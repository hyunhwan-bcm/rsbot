//! Workstream U's own checks of single functions against PyBot outputs dumped by the scripts in
//! the U scratchpad (`corpus/*.py`): `u_check forms FORMS.jsonl VOCAB.json`.
use rsbot::knowledge::lookup;
use std::io::BufRead;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args[0].as_str() {
        "forms" => {
            let (mut ok, mut bad) = (0, 0);
            for line in std::io::BufReader::new(std::fs::File::open(&args[1]).unwrap()).lines() {
                let v: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
                let w = v["w"].as_str().unwrap();
                let mut got: Vec<String> = lookup::_forms(w).iter().cloned().collect();
                got.sort();
                let want: Vec<String> = serde_json::from_value(v["forms"].clone()).unwrap();
                let fbs = rsbot::understanding::kinds::feeling_by_sound(w);
                let fbs_want: Option<String> = serde_json::from_value(v["fbs"].clone()).unwrap();
                if got == want && fbs == fbs_want {
                    ok += 1
                } else {
                    bad += 1;
                    if bad < 10 {
                        println!("{w}: {got:?} vs {want:?} / {fbs:?} vs {fbs_want:?}");
                    }
                }
            }
            println!("forms ok {ok} bad {bad}");
            let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&args[2]).unwrap()).unwrap();
            let voc = lookup::_vocabulary(lookup::all_entry_refs());
            let mut known: Vec<&String> = voc.known.iter().collect();
            known.sort();
            let want: Vec<String> = serde_json::from_value(v["known"].clone()).unwrap();
            println!("known: {} vs {} same={}", known.len(), want.len(), known.iter().map(|s| s.as_str()).eq(want.iter().map(|s| s.as_str())));
            let every: std::collections::HashMap<String, usize> = serde_json::from_value(v["every"].clone()).unwrap();
            println!("every same={}", every == voc.every);
            let mut ew: Vec<&String> = rsbot::understanding::lexicon::every_word().iter().collect();
            ew.sort();
            let want: Vec<String> = serde_json::from_value(v["every_word"].clone()).unwrap();
            println!("every_word: {} vs {} same={}", ew.len(), want.len(), ew.iter().map(|s| s.as_str()).eq(want.iter().map(|s| s.as_str())));
        }
        x => panic!("unknown check {x}"),
    }
}
