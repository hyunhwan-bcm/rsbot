//! Compare every PyBot regex against CPython's matches (harness/regex_oracle.py output).
//!
//!     regex_check ORACLE.jsonl CORPUS.jsonl [pcre2|fancy]
use rsbot::py::re::{set_engine, Engine, Re};
use std::io::BufRead;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let eng = if args.get(3).map(String::as_str) == Some("fancy") { Engine::Fancy } else { Engine::Pcre2 };
    set_engine(eng);
    let corpus: Vec<String> = std::io::BufReader::new(std::fs::File::open(&args[2]).unwrap())
        .lines()
        .map(|l| serde_json::from_str(&l.unwrap()).unwrap())
        .collect();
    // byte offset -> code point offset, per corpus line
    let char_at: Vec<Vec<usize>> = corpus
        .iter()
        .map(|t| {
            let mut v = vec![0; t.len() + 1];
            let mut n = 0;
            for (b, ch) in t.char_indices() {
                v[b] = n;
                for k in 1..ch.len_utf8() {
                    v[b + k] = n;
                }
                n += 1;
            }
            v[t.len()] = n;
            v
        })
        .collect();
    let (mut total, mut compile_err, mut mismatch, mut ok) = (0, 0, 0, 0);
    let mut shown = 0;
    let t0 = Instant::now();
    let mut search_time = std::time::Duration::ZERO;
    for line in std::io::BufReader::new(std::fs::File::open(&args[1]).unwrap()).lines() {
        let v: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        total += 1;
        let p = v["p"].as_str().unwrap();
        let f = v["f"].as_u64().unwrap() as u32;
        let rx = Re::new(p, f);
        if let Err(e) = rx.check() {
            compile_err += 1;
            if shown < 15 {
                eprintln!("COMPILE {f} {e}");
                shown += 1;
            }
            continue;
        }
        let want: Vec<(usize, usize, usize)> = v["m"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| (t[0].as_u64().unwrap() as usize, t[1].as_u64().unwrap() as usize, t[2].as_u64().unwrap() as usize))
            .collect();
        let ts = Instant::now();
        let mut got = Vec::new();
        for (row, text) in corpus.iter().enumerate() {
            for m in rx.finditer(text) {
                got.push((row, char_at[row][m.start()], char_at[row][m.end()]));
            }
        }
        search_time += ts.elapsed();
        if got == want {
            ok += 1;
        } else {
            mismatch += 1;
            if shown < 40 {
                let first = got.iter().zip(want.iter()).position(|(a, b)| a != b).unwrap_or(got.len().min(want.len()));
                let row = got.get(first).or(want.get(first)).map(|t| t.0).unwrap_or(0);
                eprintln!(
                    "MISMATCH flags={f} pat={:?}\n   line={:?}\n   rust={:?} py={:?} (n rust {} py {})",
                    &p.chars().take(160).collect::<String>(),
                    &corpus[row].chars().take(120).collect::<String>(),
                    got.get(first),
                    want.get(first),
                    got.len(),
                    want.len()
                );
                shown += 1;
            }
        }
    }
    println!(
        "{:?}: {total} patterns, {ok} identical, {mismatch} differ, {compile_err} failed to compile; search {:.1}s, total {:.1}s",
        eng,
        search_time.as_secs_f64(),
        t0.elapsed().as_secs_f64()
    );
}
