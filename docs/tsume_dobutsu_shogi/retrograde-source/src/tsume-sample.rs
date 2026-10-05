// A bounded solver for comparing sample positions with TsumeDobutsuCore.
// Input: one key or a tab-separated depth and key per line.
use std::collections::HashMap;
use std::io::{self, BufRead};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) struct State {
    pub(crate) board: [u8; 12],
    pub(crate) hand: [[u8; 3]; 2],
    pub(crate) turn: u8,
}

fn kind(piece: u8) -> u8 { piece & 7 }
fn side(piece: u8) -> usize { (piece >> 3) as usize }
fn piece(kind: u8, side: usize) -> u8 { kind + (side as u8) * 8 }
fn hand_index(kind: u8) -> usize { (kind - 2) as usize }
fn inside(row: i32, col: i32) -> bool { (0..4).contains(&row) && (0..3).contains(&col) }

fn directions(kind: u8) -> &'static [(i32, i32)] {
    match kind {
        1 => &[(-1,-1),(-1,0),(-1,1),(0,-1),(0,1),(1,-1),(1,0),(1,1)],
        2 => &[(-1,-1),(-1,1),(1,-1),(1,1)],
        3 => &[(-1,0),(0,-1),(0,1),(1,0)],
        4 => &[(-1,0)],
        5 => &[(-1,-1),(-1,0),(-1,1),(0,-1),(0,1),(1,0)],
        _ => &[],
    }
}

pub(crate) fn king(state: &State, player: usize) -> Option<usize> {
    state.board.iter().position(|&p| p == piece(1, player))
}

fn attacks(state: &State, player: usize, target: usize) -> bool {
    for (from, &p) in state.board.iter().enumerate() {
        if p == 0 || side(p) != player { continue; }
        for &(dr, dc) in directions(kind(p)) {
            let dr = if player == 0 { dr } else { -dr };
            let row = from as i32 / 3 + dr;
            let col = from as i32 % 3 + dc;
            if inside(row, col) && (row * 3 + col) as usize == target { return true; }
        }
    }
    false
}

fn check(state: &State, player: usize) -> bool {
    king(state, player).map_or(true, |k| attacks(state, 1 - player, k))
}

pub(crate) fn try_win(state: &State, player: usize) -> bool {
    let target_row = if player == 0 { 0 } else { 3 };
    king(state, player).is_some_and(|k| k / 3 == target_row && !attacks(state, 1 - player, k))
}

pub(crate) fn legal(state: &State) -> Vec<State> {
    let player = state.turn as usize;
    if king(state, player).is_none() { return Vec::new(); }
    let mut result = Vec::new();
    for from in 0..12 {
        let p = state.board[from];
        if p == 0 || side(p) != player { continue; }
        for &(dr, dc) in directions(kind(p)) {
            let dr = if player == 0 { dr } else { -dr };
            let row = from as i32 / 3 + dr;
            let col = from as i32 % 3 + dc;
            if !inside(row, col) { continue; }
            let to = (row * 3 + col) as usize;
            let captured = state.board[to];
            if captured != 0 && side(captured) == player { continue; }
            let mut next = *state;
            next.board[from] = 0;
            let promotion_row = if player == 0 { 0 } else { 3 };
            next.board[to] = if kind(p) == 4 && row == promotion_row {
                piece(5, player)
            } else { p };
            if captured != 0 && kind(captured) != 1 {
                let taken = if kind(captured) == 5 { 4 } else { kind(captured) };
                next.hand[player][hand_index(taken)] += 1;
            }
            next.turn = 1 - state.turn;
            if kind(captured) == 1 || !check(&next, player) { result.push(next); }
        }
    }
    for taken in 2..=4 {
        let index = hand_index(taken);
        if state.hand[player][index] == 0 { continue; }
        for to in 0..12 {
            if state.board[to] != 0 { continue; }
            let mut next = *state;
            next.board[to] = piece(taken, player);
            next.hand[player][index] -= 1;
            next.turn = 1 - state.turn;
            if !check(&next, player) { result.push(next); }
        }
    }
    result
}

fn has_try_move_after_pass(state: &State, player: usize) -> bool {
    let Some(from) = king(state, player) else { return false; };
    let pre_try_row = if player == 0 { 1 } else { 2 };
    if from / 3 != pre_try_row { return false; }
    let target_row = if player == 0 { 0 } else { 3 };
    for dc in -1..=1 {
        let col = from as i32 % 3 + dc;
        if !inside(target_row, col) { continue; }
        let to = (target_row * 3 + col) as usize;
        if state.board[to] != 0 && side(state.board[to]) == player { continue; }
        let mut next = *state;
        next.board[to] = next.board[from];
        next.board[from] = 0;
        if try_win(&next, player) { return true; }
    }
    false
}

pub(crate) fn allowed_attack(next: &State, fairy: bool) -> bool {
    fairy || check(next, 1) || try_win(next, 0) || has_try_move_after_pass(next, 0)
}

fn can_win(state: State, depth: u8, fairy: bool, memo: &mut HashMap<(State, u8), bool>) -> bool {
    if let Some(&answer) = memo.get(&(state, depth)) { return answer; }
    let answer = if king(&state, 1).is_none() || try_win(&state, 0) {
        true
    } else if king(&state, 0).is_none() || try_win(&state, 1) {
        false
    } else {
        let moves = legal(&state);
        if moves.is_empty() {
            state.turn == 1
        } else if depth == 0 {
            false
        } else if state.turn == 0 {
            moves.into_iter().any(|next| allowed_attack(&next, fairy)
                && can_win(next, depth - 1, fairy, memo))
        } else {
            moves.into_iter().all(|next| can_win(next, depth - 1, fairy, memo))
        }
    };
    memo.insert((state, depth), answer);
    answer
}

pub(crate) fn decode(text: &str) -> Result<State, String> {
    if text.len() != 15 { return Err("expected 15 hex digits".into()); }
    let packed = u64::from_str_radix(text, 16).map_err(|e| e.to_string())?;
    let mut state = State { board: [0; 12], hand: [[0; 3]; 2], turn: 0 };
    for x in 0..3 {
        for y in 0..4 {
            let code = ((packed >> ((x * 4 + y) * 4)) & 15) as u8;
            if code != 0 && !(1..=5).contains(&kind(code)) { return Err("bad piece".into()); }
            state.board[(3 - y) * 3 + 2 - x] = code;
        }
    }
    for player in 0..2 {
        for index in 0..3 {
            state.hand[player][index] = ((packed >> (48 + player * 6 + index * 2)) & 3) as u8;
        }
    }
    Ok(state)
}

pub(crate) fn pack(state: State) -> u64 {
    let mut cells = 0u64;
    let mut reflected = 0u64;
    for row in 0..4 {
        for col in 0..3 {
            let digit = state.board[row * 3 + col] as u64;
            cells |= digit << (((2 - col) * 4 + 3 - row) * 4);
            reflected |= digit << ((col * 4 + 3 - row) * 4);
        }
    }
    let mut hands = 0u64;
    for player in 0..2 {
        for index in 0..3 {
            hands |= (state.hand[player][index] as u64) << (48 + player * 6 + index * 2);
        }
    }
    hands | cells.min(reflected) | ((state.turn as u64) << 60)
}

pub(crate) fn unpack(key: u64) -> State {
    let mut state = decode(&format!("{:015x}", key & ((1u64 << 60) - 1))).expect("bad key");
    state.turn = (key >> 60) as u8;
    state
}

pub(crate) fn terminal(state: &State) -> i8 {
    if king(state, 1).is_none() || try_win(state, 0) { return 1; }
    if king(state, 0).is_none() || try_win(state, 1) { return 0; }
    if legal(state).is_empty() { return if state.turn == 1 { 1 } else { 0 }; }
    -1
}

pub(crate) fn valid_start(state: &State) -> bool {
    let mut count = [0; 3];
    for &p in &state.board {
        if p == 0 || kind(p) == 1 { continue; }
        count[hand_index(if kind(p) == 5 { 4 } else { kind(p) })] += 1;
    }
    for player in 0..2 {
        for index in 0..3 { count[index] += state.hand[player][index]; }
    }
    count == [2, 2, 2] && king(state, 0).is_some() && king(state, 1).is_some()
        && !check(state, 0) && !check(state, 1) && legal(state).len() > 1
}

fn distance(state: State, fairy: bool, max_depth: u8) -> Option<u8> {
    let mut memo = HashMap::new();
    for depth in (1..=max_depth).step_by(2) {
        if can_win(state, depth, fairy, &mut memo) { return Some(depth); }
    }
    None
}

fn main() {
    let max_depth = std::env::args().nth(1).and_then(|x| x.parse::<u8>().ok()).unwrap_or(7);
    for row in io::stdin().lock().lines() {
        let row = row.expect("input error");
        if row.starts_with('#') || row.trim().is_empty() { continue; }
        let fields: Vec<&str> = row.split_whitespace().collect();
        let key = if fields.len() > 1 { fields[1] } else { fields[0] };
        match decode(key) {
            Ok(state) if valid_start(&state) => {
                let normal = distance(state, false, max_depth);
                let fairy = distance(state, true, max_depth);
                println!("{} valid {} {}", key, normal.map_or("-".into(), |n| n.to_string()),
                    fairy.map_or("-".into(), |n| n.to_string()));
            }
            Ok(_) => println!("{} invalid - -", key),
            Err(error) => eprintln!("{}: {}", key, error),
        }
    }
}
