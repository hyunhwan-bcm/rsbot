//! Check py::random against CPython reference draws (harness/random_check.py output).
use rsbot::py::random::Random;
use std::io::BufRead;
fn main() {
    let path = std::env::args().nth(1).unwrap();
    let (mut ok, mut bad) = (0, 0);
    for line in std::io::BufReader::new(std::fs::File::open(path).unwrap()).lines() {
        let v: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let seed = v["seed"].as_i64().unwrap() as i128;
        let mut r = Random::new(seed);
        let mut got = serde_json::json!({"seed": v["seed"], "random": r.random()});
        got["choice"] = [1usize, 2, 3, 5, 7, 10, 17, 100, 1000]
            .iter()
            .map(|&n| *Random::new(seed).choice(&(0..n).collect::<Vec<_>>()))
            .collect::<Vec<_>>()
            .into();
        let mut r = Random::new(seed);
        got["bits1"] = (0..5).map(|_| r.getrandbits(1)).collect::<Vec<_>>().into();
        got["sample3_10"] = Random::new(seed).sample(&(0..10).collect::<Vec<_>>(), 3).into();
        got["sample3_40"] = Random::new(seed).sample(&(0..40).collect::<Vec<_>>(), 3).into();
        got["sample8_100"] = Random::new(seed).sample(&(0..100).collect::<Vec<_>>(), 8).into();
        let mut xs: Vec<usize> = (0..9).collect();
        Random::new(seed).shuffle(&mut xs);
        got["shuffle9"] = xs.into();
        if got == v { ok += 1 } else { bad += 1; if bad < 4 { eprintln!("want {v}\n got {got}") } }
    }
    println!("random: {ok} seeds identical, {bad} differ");
}
