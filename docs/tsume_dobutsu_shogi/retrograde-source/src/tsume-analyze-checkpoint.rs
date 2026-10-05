// Complete each compressed depth layer before beginning the next one.
#[allow(dead_code)]
#[path = "tsume-reverse.rs"]
mod reverse;

use reverse::rules;
use std::collections::HashSet;
use std::fs;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

fn layer_path(dir: &Path, depth: usize) -> PathBuf {
    dir.join(format!("depth{:02}.txt.xz", depth))
}

fn write_layer(dir: &Path, depth: usize, keys: &[u64]) {
    let target = layer_path(dir, depth);
    let temporary = dir.join(format!("depth{:02}.pending.xz", depth));
    let file = fs::File::create(&temporary).expect("create checkpoint");
    let mut compressor = Command::new("xz").args(["-T1", "-1", "-c"])
        .stdin(Stdio::piped()).stdout(Stdio::from(file)).spawn().expect("start xz");
    {
        let mut output = BufWriter::new(compressor.stdin.take().expect("xz stdin"));
        for &key in keys {
            writeln!(output, "{:016x} {}", key, depth).expect("write checkpoint");
        }
    }
    assert!(compressor.wait().expect("wait for xz").success(), "xz failed");
    fs::rename(temporary, target).expect("commit checkpoint");
}

fn read_layer(dir: &Path, depth: usize) -> Vec<u64> {
    let mut decoder = Command::new("xz").arg("-dc").arg(layer_path(dir, depth))
        .stdout(Stdio::piped()).spawn().expect("start xz decoder");
    let mut keys = Vec::new();
    for line in BufReader::new(decoder.stdout.take().expect("xz stdout")).lines() {
        let line = line.expect("read checkpoint");
        let mut fields = line.split_whitespace();
        let key = u64::from_str_radix(fields.next().expect("checkpoint key"), 16)
            .expect("invalid checkpoint key");
        assert_eq!(fields.next().expect("checkpoint depth").parse::<usize>().unwrap(), depth);
        assert!(fields.next().is_none(), "extra checkpoint field");
        keys.push(key);
    }
    assert!(decoder.wait().expect("wait for decoder").success(), "bad checkpoint");
    keys
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 4, "usage: analyzer normal|fairy max_depth checkpoint_dir");
    let fairy = match args[1].as_str() {
        "normal" => false,
        "fairy" => true,
        _ => panic!("mode must be normal or fairy"),
    };
    let max_depth: usize = if args[2] == "all" { usize::MAX }
        else { args[2].parse().expect("invalid depth") };
    let dir = Path::new(&args[3]);
    fs::create_dir_all(dir).expect("create checkpoint directory");
    let started = Instant::now();

    let mut reachable = HashSet::new();
    let mut winning = HashSet::new();
    let mut frontier = Vec::new();
    for row in io::stdin().lock().lines() {
        let row = row.expect("input error");
        let mut fields = row.split_whitespace();
        let key = u64::from_str_radix(fields.next().expect("missing key"), 16).expect("bad key");
        let status = fields.next().expect("missing status");
        assert!(reachable.insert(key), "duplicate input key");
        if status == "1" {
            assert!(winning.insert(key));
            frontier.push(key);
        } else {
            assert!(status == "0" || status == "-1", "bad status");
        }
    }
    eprintln!("loaded {} reachable positions and {} win seeds in {:.1}s",
        reachable.len(), frontier.len(), started.elapsed().as_secs_f64());
    if layer_path(dir, 0).exists() {
        let seeds = read_layer(dir, 0);
        assert_eq!(seeds.len(), frontier.len(), "checkpoint seed count differs");
        assert!(seeds.iter().all(|key| winning.contains(key)), "checkpoint seeds differ");
    } else {
        write_layer(dir, 0, &frontier);
        eprintln!("checkpoint depth 0: {} wins", frontier.len());
    }

    let mut depth = 0usize;
    while depth < max_depth && layer_path(dir, depth + 1).exists() {
        let next = read_layer(dir, depth + 1);
        for &key in &next {
            assert!(reachable.contains(&key) && winning.insert(key), "invalid checkpoint key");
        }
        depth += 1;
        frontier = next;
        eprintln!("resumed depth {}: {} total, elapsed {:.1}s",
            depth, winning.len(), started.elapsed().as_secs_f64());
    }
    let mut last_count = frontier.len();
    while depth < max_depth && !frontier.is_empty() {
        let layer_started = Instant::now();
        let mut candidates = HashSet::new();
        for &key in &frontier {
            for predecessor in reverse::predecessors(key, fairy, &reachable) {
                if !winning.contains(&predecessor) { candidates.insert(predecessor); }
            }
        }
        let mut next = Vec::new();
        for key in candidates {
            let state = rules::unpack(key);
            let won = if state.turn == 0 {
                true
            } else {
                let moves = rules::legal(&state);
                !moves.is_empty() && moves.into_iter().all(|child| winning.contains(&rules::pack(child)))
            };
            if won { next.push(key); }
        }
        depth += 1;
        write_layer(dir, depth, &next);
        for &key in &next { winning.insert(key); }
        let seconds = layer_started.elapsed().as_secs_f64();
        let ratio = if last_count == 0 { 0.0 } else { next.len() as f64 / last_count as f64 };
        let estimate = if ratio > 0.0 && ratio < 1.0 {
            format!("{:.1}s (rough, assuming the latest layer ratio persists)",
                seconds * ratio / (1.0 - ratio))
        } else if next.is_empty() { "0s (converged)".to_string() }
        else { "unknown".to_string() };
        eprintln!("checkpoint depth {}: {} wins, {} total, layer {:.1}s, elapsed {:.1}s, remaining {}",
            depth, next.len(), winning.len(), seconds, started.elapsed().as_secs_f64(), estimate);
        last_count = next.len();
        frontier = next;
    }
    eprintln!("analyzed through depth {}: {} winning positions", depth, winning.len());
}
