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
            let every: lookup::HashMap<String, usize> = serde_json::from_value(v["every"].clone()).unwrap();
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
        "topics" => {
            use rsbot::understanding::{body, breeds, cars, confusion, attempts, kinds, normalize, references, senses, topics};
            let show: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(5);
            let (mut ok, mut bad, mut shown) = (0, 0, 0);
            let mut fields: std::collections::BTreeMap<String, usize> = Default::default();
            let tj = |x: serde_json::Value| x;
            for line in std::io::BufReader::new(std::fs::File::open(&args[1]).unwrap()).lines() {
                let v: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
                let got: Option<Vec<(&str, serde_json::Value)>> = rsbot::runtime::guarded(|| {
                    let l = normalize::normalise(v["pre"].as_str().unwrap());
                    let read = kinds::classify(&l);
                    let ents = topics::detect_entities(&l, None);
                    let prev_line = v["prev_pre"].as_str().map(normalize::normalise);
                    let prev_ents = prev_line.as_ref().map(|p| topics::detect_entities(p, None)).unwrap_or_default();
                    let prev_topic = prev_line.as_ref().and_then(|p| topics::detect_topic(p, None, Some(&prev_ents), None).0);
                    let r = references::resolve(&l, &read, prev_line.as_ref(), &prev_ents, prev_topic.as_deref());
                    let topic = topics::detect_topic(&l, None, Some(&ents), Some(true)).0;
                    let mut dog = rsbot::understood::Understood::default();
                    dog.animal_in_hand = "dog".into();
                    vec![
                        ("entities", serde_json::to_value(&ents).unwrap()),
                        ("numbers", serde_json::to_value(topics::detect_numbers(&l)).unwrap()),
                        ("topic_t", serde_json::to_value(topics::detect_topic(&l, None, Some(&ents), Some(true))).unwrap()),
                        ("topic_f", serde_json::to_value(topics::detect_topic(&l, None, Some(&ents), Some(false))).unwrap()),
                        ("topic_n", serde_json::to_value(topics::detect_topic(&l, None, None, None)).unwrap()),
                        ("topic_inh", serde_json::to_value(topics::detect_topic(&l, Some("health"), Some(&ents), Some(true))).unwrap()),
                        ("rank", serde_json::to_value(topics::rank_topics(&l, None, Some(&ents), Some(true))).unwrap()),
                        ("score", serde_json::to_value(topics::score_topics(&l)).unwrap()),
                        ("subject", serde_json::to_value(topics::subject_topic(&l, Some(&ents))).unwrap()),
                        ("said", serde_json::to_value(topics::the_words_that_said(&l, topic.as_deref(), Some(&ents))).unwrap()),
                        ("warmth", serde_json::to_value(topics::the_warmth_said(&l.norm)).unwrap()),
                        ("own_output", serde_json::to_value(topics::own_output_word(&l.norm)).unwrap()),
                        ("person_names", serde_json::to_value(topics::person_names(&l)).unwrap()),
                        ("purse", serde_json::to_value(topics::purse_till_at(&l.norm, &l.text)).unwrap()),
                        ("tight", serde_json::to_value(topics::tight_on_money(&l.norm, Some(&read.kind), &l.text)).unwrap()),
                        ("unclear", serde_json::to_value(topics::unclear_subject(&l.norm, None, prev_line.as_ref())).unwrap()),
                        ("unclear_inh", serde_json::to_value(topics::unclear_subject(&l.norm, Some("money"), None)).unwrap()),
                        ("unsure", serde_json::to_value(topics::unsure_words(&l.norm, topic.as_deref())).unwrap()),
                        ("body", serde_json::to_value(body::read(&l.norm_text)).unwrap()),
                        ("ref", serde_json::to_value(&r).unwrap()),
                        ("target", match &prev_line { Some(p) => serde_json::to_value(references::_target(p, &prev_ents, prev_topic.as_deref())).unwrap(), None => serde_json::Value::Null }),
                        ("new_person", serde_json::to_value(references::person_is_new(&l, &prev_ents)).unwrap()),
                        ("in_hand", serde_json::to_value(references::the_animal_in_hand(&r, &l.words)).unwrap()),
                        ("chat_animal", serde_json::to_value(references::the_animal_the_chat_is_on(&l.words, Some(&dog))).unwrap()),
                        ("senses", serde_json::to_value(senses::senses(&l)).unwrap()),
                        ("breeds", serde_json::to_value(breeds::the_animal_a_breed_is(&l.words)).unwrap()),
                        ("car", serde_json::to_value(cars::a_broken_car_is_the_subject(&l.clean, &prev_line.as_ref().map(|p| vec![p.clean.clone()]).unwrap_or_default())).unwrap()),
                        ("confusion", serde_json::to_value(confusion::confusion_of(&l)).unwrap()),
                        ("tried", serde_json::to_value(attempts::things_tried(&l.text, Some(&l.norm))).unwrap()),
                    ]
                });
                let mut same = true;
                match got {
                    Some(got) => {
                        for (name, g) in got {
                            let g = tj(g);
                            if g != v[name] {
                                same = false;
                                *fields.entry(name.to_string()).or_default() += 1;
                                if shown < show { println!("-- {name} {:?}\n   rust {}\n   py   {}", v["text"].as_str().unwrap(), g, v[name]); shown += 1; }
                            }
                        }
                    }
                    None => {
                        same = false;
                        *fields.entry("(panic)".into()).or_default() += 1;
                        if shown < show { println!("-- panic {:?}", v["text"]); shown += 1; }
                    }
                }
                if same { ok += 1 } else { bad += 1 }
            }
            println!("topics ok {ok} bad {bad} {fields:?}");
        }
        "understand" => {
            // u_check understand gold|heldout [show]: understand(text) against PyBot's reading
            let show: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(5);
            let rows = rsbot::oracle::load(&args[1]);
            let (mut ok, mut bad, mut shown) = (0, 0, 0);
            let mut fields: std::collections::BTreeMap<String, usize> = Default::default();
            for row in rows {
                let got = rsbot::runtime::guarded(|| rsbot::understanding::understand(&row.text, None));
                let got = got.map(|u| serde_json::to_value(u).unwrap()).unwrap_or(serde_json::Value::Null);
                let want = serde_json::to_value(&row.understood).unwrap();
                let mut same = true;
                match (got.as_object(), want.as_object()) {
                    (Some(g), Some(w)) => {
                        for (k, wv) in w {
                            if k == "wants" || k == "looks_back" { continue; }
                            if g.get(k) != Some(wv) {
                                same = false;
                                *fields.entry(k.clone()).or_default() += 1;
                                if shown < show { println!("-- {} {k} {:?}\n   rust {}\n   py   {}", row.id, row.text, g.get(k).map(|x| x.to_string()).unwrap_or_default(), wv); shown += 1; }
                            }
                        }
                    }
                    _ => { same = false; *fields.entry("(panic)".into()).or_default() += 1; if shown < show { println!("-- panic {:?}", row.text); shown += 1; } }
                }
                if same { ok += 1 } else { bad += 1 }
            }
            println!("understand ok {ok} bad {bad} {fields:?}");
        }
        "find" => {
            // u_check find FIND.jsonl [show]: lookup.find on PyBot's own reading, plain and with
            // use_context, against PyBot's answers (corpus made by find/make_find.py).
            rsbot::runtime::quiet_panics();
            let show: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10);
            let (mut ok, mut bad, mut shown, mut blocked) = (0, 0, 0, 0);
            let t0 = std::time::Instant::now();
            for line in std::io::BufReader::new(std::fs::File::open(&args[1]).unwrap()).lines() {
                let v: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
                let u: rsbot::understood::Understood = match serde_json::from_value(v["understood"].clone()) {
                    Ok(u) => u,
                    Err(e) => { println!("-- unreadable {:?}: {e}", v["text"]); bad += 1; continue; }
                };
                for (key, ctx) in [("find", false), ("ctx", true)] {
                    let before: usize = rsbot::knowledge::registry::misses().values().sum();
                    let got = rsbot::runtime::guarded(|| lookup::find(&u, None, None, ctx).map(|e| e.id.clone()));
                    let got = match got { Some(Some(id)) => serde_json::Value::String(id), Some(None) => serde_json::Value::Null, None => serde_json::Value::String(format!("!panic {}", rsbot::runtime::take_last_panic().unwrap_or_default())) };
                    let missed = rsbot::knowledge::registry::misses().values().sum::<usize>() > before;
                    if got == v[key] { ok += 1 } else if missed { blocked += 1; if let Ok(path) = std::env::var("U_BLOCKED") { use std::io::Write; let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path).unwrap(); writeln!(f, "{}", serde_json::json!({"text": v["text"], "key": key, "rust": got})).unwrap(); } } else {
                        bad += 1;
                        if shown < show { println!("-- {key} {:?}\n   rust {got}\n   py   {}", v["text"], v[key]); shown += 1; }
                    }
                }
            }
            println!("find ok {ok} bad {bad} (+{blocked} differing where an unported domain function was called) in {:?}", t0.elapsed());
            let misses = rsbot::knowledge::registry::misses();
            if !misses.is_empty() { println!("unported functions called: {}", misses.len()); for (k, n) in misses.iter().take(40) { println!("  {n} {k}"); } }
        }
        "unported" => {
            // u_check unported: the guard and shape functions entries name that have no port
            fn walk(v: &rsbot::data::FnRef, out: &mut std::collections::BTreeSet<String>) {
                if !rsbot::knowledge::registry::has_guard(&v.id) { out.insert(format!("guard {}", v.id)); }
                for (_, c) in &v.cells {
                    if let Some(f) = c.as_fn() { walk(f, out) }
                    for x in c.items() { if let Some(f) = x.as_fn() { walk(f, out) } }
                }
            }
            let mut out = std::collections::BTreeSet::new();
            for e in lookup::all_entries() {
                if let Some(g) = &e.guard { walk(g, &mut out) }
                for s in &e.shape { if !rsbot::knowledge::registry::has_shape(&s.id) { out.insert(format!("shape {}", s.id)); } }
            }
            for x in out { println!("{x}"); }
        }
        "gives" => {
            // u_check gives GIVES.jsonl [show]: compose::fit::gives against PyBot (fit/make_gives.py)
            use rsbot::compose::fit;
            let show: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10);
            let (mut ok, mut bad, mut shown) = (0, 0, 0);
            let mut by_want: std::collections::BTreeMap<String, usize> = Default::default();
            for line in std::io::BufReader::new(std::fs::File::open(&args[1]).unwrap()).lines() {
                let v: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
                let reply = v["reply"].as_str().unwrap_or("");
                let said = v["text"].as_str().unwrap_or("");
                if fit::first_sentence(reply) != v["first"].as_str().unwrap_or("") { bad += 1; *by_want.entry("first_sentence".into()).or_default() += 1; } else { ok += 1 }
                for (key, l) in [("gives", Some(said)), ("none", None)] {
                    for (want, wv) in v[key].as_object().unwrap() {
                        let got = rsbot::runtime::guarded(|| fit::gives(reply, want, l));
                        let got = match got { Some(Some(b)) => serde_json::Value::Bool(b), Some(None) => serde_json::Value::Null, None => serde_json::Value::String("!panic".into()) };
                        if &got == wv { ok += 1 } else {
                            bad += 1;
                            *by_want.entry(format!("{key}:{want}")).or_default() += 1;
                            if shown < show { println!("-- {key} {want} line={said:?}\n   reply {:?}\n   rust {got} py {wv}", &reply[..reply.len().min(160)]); shown += 1; }
                        }
                    }
                }
            }
            println!("gives ok {ok} bad {bad} {by_want:?}");
        }
        "dream" => {
            // u_check dream DREAM.jsonl: memory/dream.py's readers against PyBot (dream/make.py)
            use rsbot::memory::dream as d;
            let died = rsbot::understanding::rx("\\bdied\\b");
            let (mut ok, mut bad) = (0, 0);
            for line in std::io::BufReader::new(std::fs::File::open(&args[1]).unwrap()).lines() {
                let v: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
                let t = v["text"].as_str().unwrap();
                let got = serde_json::json!({"text": t, "scene": d::the_scene(t), "named": d::named(t), "told": d::told_as_a_dream(t),
                    "not_happening": d::a_dream_is_not_happening(t), "death_first": d::a_death_came_first(t), "before_died": d::dreamed_before(t, died)});
                if got == v { ok += 1 } else { bad += 1; if bad < 10 { println!("-- {t:?}\n   rust {got}\n   py   {v}"); } }
            }
            println!("dream ok {ok} bad {bad}");
        }
        "guards" => {
            // u_check guards ONE.jsonl ID...: lookup.guard_passes for these entries on the record's reading
            let v: serde_json::Value = serde_json::from_str(std::fs::read_to_string(&args[1]).unwrap().lines().next().unwrap()).unwrap();
            let u: rsbot::understood::Understood = serde_json::from_value(v["understood"].clone()).unwrap();
            for id in &args[2..] {
                let e = rsbot::knowledge::entry::by_id(id).expect("no such entry");
                println!("{id} guard_passes={}", lookup::guard_passes(e, &u));
            }
        }
        x => panic!("unknown check {x}"),
    }
}
