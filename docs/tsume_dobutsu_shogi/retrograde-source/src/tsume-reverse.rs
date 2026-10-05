// Generate candidate predecessors of a position, then verify each forward edge.
#[allow(dead_code)]
#[path = "tsume-sample.rs"]
pub(crate) mod rules;

use std::collections::HashSet;
use rules::State;

fn piece_kind(piece: u8) -> u8 { piece & 7 }
fn piece_side(piece: u8) -> usize { (piece >> 3) as usize }

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

fn offer(candidate: State, target: u64, fairy: bool, reachable: &HashSet<u64>, output: &mut HashSet<u64>) {
    let key = rules::pack(candidate);
    if !reachable.contains(&key) || !output.insert(key) { return; }
    if rules::terminal(&candidate) != -1 || !rules::legal(&candidate).into_iter().any(|next| {
        (fairy || candidate.turn != 0 || rules::allowed_attack(&next, false))
            && rules::pack(next) == target
    }) {
        output.remove(&key);
    }
}

pub(crate) fn predecessors(target: u64, fairy: bool, reachable: &HashSet<u64>) -> Vec<u64> {
    let after = rules::unpack(target);
    let mover = (1 - after.turn) as usize;
    let opponent = 1 - mover;
    let mut found = HashSet::new();
    for to in 0..12 {
        let current = after.board[to];
        if current == 0 || piece_side(current) != mover { continue; }
        let current_kind = piece_kind(current);
        if (2..=4).contains(&current_kind) && after.hand[mover][(current_kind - 2) as usize] < 2 {
            let mut before = after;
            before.turn = mover as u8;
            before.board[to] = 0;
            before.hand[mover][(current_kind - 2) as usize] += 1;
            offer(before, target, fairy, reachable, &mut found);
        }
        for original_kind in [current_kind, if current_kind == 5 { 4 } else { 0 }] {
            if original_kind == 0 { continue; }
            for &(dr, dc) in directions(original_kind) {
                let dr = if mover == 0 { dr } else { -dr };
                let row = to as i32 / 3 - dr;
                let col = to as i32 % 3 - dc;
                if !(0..4).contains(&row) || !(0..3).contains(&col) { continue; }
                let from = (row * 3 + col) as usize;
                if after.board[from] != 0 { continue; }
                let promotion_row = if mover == 0 { 0 } else { 3 };
                let promoted = original_kind == 4 && to / 3 == promotion_row;
                if (if promoted { 5 } else { original_kind }) != current_kind { continue; }
                let mut before = after;
                before.turn = mover as u8;
                before.board[from] = original_kind + (mover as u8) * 8;
                before.board[to] = 0;
                offer(before, target, fairy, reachable, &mut found);
                for captured_kind in [1u8, 2, 3, 4, 5] {
                    let mut captured = before;
                    if captured_kind != 1 {
                        let hand_kind = if captured_kind == 5 { 4 } else { captured_kind };
                        let index = (hand_kind - 2) as usize;
                        if captured.hand[mover][index] == 0 { continue; }
                        captured.hand[mover][index] -= 1;
                    }
                    captured.board[to] = captured_kind + (opponent as u8) * 8;
                    offer(captured, target, fairy, reachable, &mut found);
                }
            }
        }
    }
    found.into_iter().collect()
}
