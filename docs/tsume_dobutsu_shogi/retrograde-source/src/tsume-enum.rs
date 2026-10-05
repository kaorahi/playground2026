// Enumerate positions reachable under the HTML game's unrestricted legal moves.
// The most significant bit after the 60-bit board code stores the side to move.
#[allow(dead_code)]
#[path = "tsume-sample.rs"]
mod rules;

use std::collections::HashSet;
use std::io::{self, Write};
use rules::{pack, unpack, terminal};

fn main() {
    let cap = std::env::args().nth(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(usize::MAX);
    let root_key = std::env::args().nth(2)
        .map(|s| u64::from_str_radix(&s, 16).expect("bad root key"))
        .unwrap_or(0x000b0029c41a003);
    let root = pack(unpack(root_key));
    let mut stack = vec![root];
    let mut visited = HashSet::new();
    visited.insert(root);
    let mut output = io::BufWriter::new(io::stdout().lock());
    while let Some(key) = stack.pop() {
        let state = unpack(key);
        let status = terminal(&state);
        writeln!(output, "{:016x} {}", key, status).expect("write error");
        if status == -1 {
            for child in rules::legal(&state) {
                let child_key = pack(child);
                if visited.insert(child_key) {
                    stack.push(child_key);
                    if visited.len() > cap {
                        output.flush().expect("flush error");
                        eprintln!("limit reached at {} states", visited.len());
                        std::process::exit(2);
                    }
                    if visited.len() % 1_000_000 == 0 {
                        eprintln!("discovered {} states, pending {}", visited.len(), stack.len());
                    }
                }
            }
        }
    }
    eprintln!("complete: {} states", visited.len());
}
