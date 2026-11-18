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
        "norm" => {
            // u_check norm LINES.jsonl [--show K]: the three pre-passes and normalise(pre)
            let show: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(5);
            let (mut ok, mut bad, mut shown) = (0, 0, 0);
            let mut fields: std::collections::BTreeMap<String, usize> = Default::default();
            for line in std::io::BufReader::new(std::fs::File::open(&args[1]).unwrap()).lines() {
                let v: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
                let text = v["text"].as_str().unwrap();
                let got_pre = rsbot::runtime::guarded(|| {
                    use rsbot::understanding::normalize as n;
                    n::_the_sugar_that_is_blood(&n::_the_part_that_hurts(&n::without_the_figure_of_speech(text)))
                });
                let want_pre = v["pre"].as_str().unwrap();
                let mut same = true;
                if got_pre.as_deref() != Some(want_pre) {
                    same = false;
                    *fields.entry("(pre)".into()).or_default() += 1;
                    if shown < show { println!("-- pre {text:?}\n   rust {got_pre:?}\n   py   {want_pre:?}"); shown += 1; }
                }
                let got = rsbot::runtime::guarded(|| rsbot::understanding::normalize::normalise(want_pre));
                let got = got.map(|l| serde_json::to_value(l).unwrap()).unwrap_or(serde_json::Value::Null);
                let want = &v["line"];
                if let (Some(g), Some(w)) = (got.as_object(), want.as_object()) {
                    for (k, wv) in w {
                        if g.get(k) != Some(wv) {
                            same = false;
                            *fields.entry(k.clone()).or_default() += 1;
                            if shown < show { println!("-- {k} {text:?}\n   rust {}\n   py   {}", g.get(k).map(|x| x.to_string()).unwrap_or_default(), wv); shown += 1; }
                        }
                    }
                } else {
                    same = false;
                    *fields.entry("(panic)".into()).or_default() += 1;
                    if shown < show { println!("-- panic {text:?}"); shown += 1; }
                }
                if same { ok += 1 } else { bad += 1 }
            }
            println!("norm ok {ok} bad {bad} {fields:?}");
        }
        "kinds" => {
            // u_check kinds LINES.jsonl [show]: classify(Python's line) against Python's read
            let show: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(5);
            let (mut ok, mut bad, mut shown) = (0, 0, 0);
            let mut fields: std::collections::BTreeMap<String, usize> = Default::default();
            for line in std::io::BufReader::new(std::fs::File::open(&args[1]).unwrap()).lines() {
                let v: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
                let l: rsbot::understanding::normalize::Line = serde_json::from_value(v["line"].clone()).unwrap();
                let got = rsbot::runtime::guarded(|| rsbot::understanding::kinds::classify(&l));
                let got = got.map(|r| serde_json::to_value(r).unwrap()).unwrap_or(serde_json::Value::Null);
                let want = &v["read"];
                let mut same = true;
                match (got.as_object(), want.as_object()) {
                    (Some(g), Some(w)) => {
                        for (k, wv) in w {
                            if g.get(k) != Some(wv) {
                                same = false;
                                *fields.entry(k.clone()).or_default() += 1;
                                if shown < show { println!("-- {k} {:?}\n   rust {}\n   py   {}", l.norm_text, g.get(k).map(|x| x.to_string()).unwrap_or_default(), wv); shown += 1; }
                            }
                        }
                    }
                    _ => {
                        same = false;
                        *fields.entry("(panic)".into()).or_default() += 1;
                        if shown < show { println!("-- panic {:?}", l.norm_text); shown += 1; }
                    }
                }
                if same { ok += 1 } else { bad += 1 }
            }
            println!("kinds ok {ok} bad {bad} {fields:?}");
        }
        "kindsfns" => {
            use rsbot::understanding::kinds as k;
            let show: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(5);
            let (mut ok, mut bad, mut shown) = (0, 0, 0);
            let mut fields: std::collections::BTreeMap<String, usize> = Default::default();
            for line in std::io::BufReader::new(std::fs::File::open(&args[1]).unwrap()).lines() {
                let v: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
                let l: rsbot::understanding::normalize::Line = serde_json::from_value(v["line"].clone()).unwrap();
                let read: k::Read = serde_json::from_value(v["read"].clone()).unwrap();
                let earlier: Vec<String> = serde_json::from_value(v["earlier"].clone()).unwrap();
                let mut as_read = rsbot::understood::Understood::default();
                as_read.kind = read.kind.clone();
                as_read.intent = read.intent.clone();
                as_read.asks = read.asks.clone();
                as_read.followup = true;
                as_read.text = v["text"].as_str().unwrap().to_string();
                let mut prev = rsbot::understood::Understood::default();
                prev.over_time = Some(true);
                prev.over_time_what = Some("price".into());
                prev.text = "earlier".into();
                let j = |x: serde_json::Value| x;
                let got: Vec<(&str, serde_json::Value)> = vec![
                    ("over_time", j(serde_json::to_value(k::over_time(&l, &read, None, false)).unwrap())),
                    ("carrying", j(serde_json::to_value(k::over_time(&l, &as_read, Some(&prev), true)).unwrap())),
                    ("taking_up", serde_json::to_value(k::taking_up(&l, Some(&read))).unwrap()),
                    ("care_home", serde_json::to_value(k::a_care_home(&l.clean)).unwrap()),
                    ("carried_home", serde_json::to_value(k::a_home_carried_forward(&l.clean, &earlier)).unwrap()),
                    ("setup", serde_json::to_value(k::the_setup(&l)).unwrap()),
                    ("routine", serde_json::to_value(k::is_a_routine(&l, Some(&read))).unwrap()),
                    ("shift", serde_json::to_value(k::shift_worked(&l.clean)).unwrap()),
                    ("after_any", serde_json::to_value(k::after_any_link(&l.norm)).unwrap()),
                    ("how_i_am", serde_json::to_value(k::asks_how_i_am(&l.norm)).unwrap()),
                    ("noun_req", serde_json::to_value(k::noun_phrase_request(&l)).unwrap()),
                    ("opening", serde_json::to_value(k::opening_line(&l.norm)).unwrap()),
                    ("talk", serde_json::to_value(k::talk_frame(&l.norm)).unwrap()),
                    ("courtesy", serde_json::to_value(k::the_line_after_a_courtesy(&l.norm)).unwrap()),
                    ("emotion", serde_json::to_value(k::emotion_of(&l.norm_text)).unwrap()),
                    ("asks_back", serde_json::to_value(k::_asks_back(&l.norm)).unwrap()),
                    ("fall", serde_json::to_value(k::_a_fall_that_is_not_one(&l.norm)).unwrap()),
                    ("step", serde_json::to_value(k::_a_step_done(&l.norm)).unwrap()),
                    ("wrong", serde_json::to_value(k::_the_step_went_wrong(&l.norm)).unwrap()),
                ];
                let mut same = true;
                for (name, g) in got {
                    if g != v[name] {
                        same = false;
                        *fields.entry(name.to_string()).or_default() += 1;
                        if shown < show { println!("-- {name} {:?}\n   rust {}\n   py   {}", l.norm_text, g, v[name]); shown += 1; }
                    }
                }
                if same { ok += 1 } else { bad += 1 }
            }
            println!("kindsfns ok {ok} bad {bad} {fields:?}");
        }
        x => panic!("unknown check {x}"),
    }
}
