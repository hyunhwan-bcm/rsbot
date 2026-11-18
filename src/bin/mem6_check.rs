//! Check the MEM6 ports (bot/memory/__init__, already, carry, cause, choice, counts, decided,
//! history, limit, outgoings, price, repair, spent, swap, told) function by function against the
//! calls PyBot made (`harness/mem6_record.py` output).
//!
//!     RSBOT_TODAY=2026-11-17 mem6_check CALLS.jsonl.gz [...] [--fn SUBSTR] [--show K]
//!
//! Per function: calls that give PyBot's result (and leave memory as PyBot left it), calls that
//! differ, calls that reached a function another workstream has not ported yet (UNPORTED), and
//! calls not compared (the call moved the global generator, or an argument the checker does not
//! rebuild). A Python None and a Rust "" are the same answer (several contracts say None as "").
use rsbot::data::Value;
use rsbot::memory as m;
use rsbot::state::{Memory, Turn};
use serde_json::{json, Value as J};
use std::collections::BTreeMap;
use std::io::BufRead;

struct Args<'a>(&'a serde_json::Map<String, J>);

impl<'a> Args<'a> {
    fn j(&self, name: &str) -> &'a J {
        static NULL: J = J::Null;
        self.0.get(name).unwrap_or(&NULL)
    }
    fn s(&self, name: &str) -> String {
        match self.j(name) {
            J::String(s) => s.clone(),
            _ => String::new(),
        }
    }
    fn n(&self, name: &str) -> i64 {
        self.j(name).as_i64().unwrap_or(0)
    }
    fn f(&self, name: &str) -> f64 {
        self.j(name).as_f64().unwrap_or(0.0)
    }
    fn v(&self, name: &str) -> Value {
        Value::from_plain(self.j(name))
    }
    fn strs(&self, name: &str) -> Vec<String> {
        self.j(name).as_array().map(|a| a.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect()).unwrap_or_default()
    }
    fn mem(&self, name: &str) -> Memory {
        match self.j(name) {
            J::Object(o) => Memory::from_plain(o),
            _ => Memory::new(),
        }
    }
    fn has_mem(&self, name: &str) -> bool {
        self.j(name).is_object()
    }
}

fn s(x: impl Into<String>) -> J {
    J::String(x.into())
}
fn os(x: Option<String>) -> J {
    x.map(J::String).unwrap_or(J::Null)
}
fn pair((a, b): (String, String)) -> J {
    json!([a, b])
}
fn turn(t: &Turn) -> J {
    json!({"user": t.user, "bot": t.bot, "module": t.module})
}

/// The memory keys a reader writes, as plain JSON, for comparing with PyBot's after the call.
fn written(mem: &Memory) -> J {
    let mut plain = mem.to_plain();
    for key in ["understood", "turns", "intent_wants", "asked_now"] {
        plain.remove(key);
    }
    J::Object(plain)
}
fn written_j(j: &J) -> J {
    let mut plain = j.as_object().cloned().unwrap_or_default();
    for key in ["understood", "turns", "intent_wants", "asked_now"] {
        plain.remove(key);
    }
    J::Object(plain)
}

/// Run the Rust port of one recorded call: Some((result, memory after)) or None when the
/// function has no case here.
fn call(name: &str, a: &Args) -> Option<(J, Option<J>)> {
    let (module, func) = name.split_once(':')?;
    let module = module.trim_start_matches("bot.memory").trim_start_matches('.');
    let text = || a.s("text");
    let mut mem = a.mem("memory");
    let opt_mem = |mem: &Memory| if a.has_mem("memory") { Some(mem.clone()) } else { None };
    let r: J = match (module, func) {
        ("", "you") => s(m::you(&text())),
        ("", "plan_sentence") => s(m::plan_sentence(&a.s("plan"), &a.s("said_line"))),
        ("", "_the_plan_said_back") => s(m::_the_plan_said_back(&a.s("plan"), &a.s("said_line"))),
        ("", "_statement") => s(m::_statement(&text())),
        ("", "reads_as_a_plan") => J::Bool(m::reads_as_a_plan(&m::pyv::str_of(&a.v("value")), &a.s("text"))),
        ("", "_the_thing_the_plan_is") => s(m::_the_thing_the_plan_is(&a.s("plan"))),
        ("", "named_days") => s(m::named_days(&a.s("said"))),
        ("", "appointment_sentence") => s(m::appointment_sentence(&text())),
        ("", "_mid") => s(m::_mid(&text())),
        ("", "_to_the_bot") => s(m::_to_the_bot(&text())),
        ("", "_from_the_bot") => s(m::_from_the_bot(&text())),
        ("", "without_the_apostrophe") => s(m::without_the_apostrophe(&text())),
        ("", "_the_relation_possessive") => s(m::_the_relation_possessive(&text())),
        ("", "_flip_to_the_second_person") => s(m::_flip_to_the_second_person(&text())),
        ("", "there_is_a_chat") => J::Bool(m::there_is_a_chat(&mem)),
        ("", "recent") => J::Array(m::recent(opt_mem(&mem).as_ref(), a.n("n")).iter().map(turn).collect()),
        ("", "last_user") => m::last_user(&mem, a.n("n")).to_plain(),
        ("", "last_bot") => s(m::last_bot(&mem)),
        ("", "last_module") => os(m::last_module(&mem)),
        ("", "user") => m::user(&mut mem).to_plain(),
        ("", "log") => m::log(&mut mem).to_plain(),
        ("", "others") => m::others(&mut mem).to_plain(),
        ("", "name") => s(m::name(&mut mem)),
        ("", "plan_line") => s(m::plan_line(&mut mem, &m::pyv::str_or_empty(&a.v("plan")))),
        ("", "remember") => {
            m::remember(&mut mem, &a.s("key"), &a.v("value"), &a.s("text"));
            J::Null
        }
        ("", "remember_other") => {
            m::remember_other(&mut mem, &a.v("entry"), &a.s("text"));
            J::Null
        }
        ("", "forget") => Value::from(m::forget(&mut mem, &a.s("key"), &a.v("values"), &a.s("why"))).to_plain(),
        ("choice", "reply_for") => os(m::choice::reply_for(&text(), &mem)),
        ("choice", "the_decision") => os(m::choice::the_decision(&text(), &mem)),
        ("choice", "options_in") => json!(m::choice::options_in(&a.s("reply"))),
        ("choice", "asks_for_a_pick") => J::Bool(m::choice::asks_for_a_pick(&text())),
        ("choice", "_clean_option") => s(m::choice::_clean_option(&a.s("piece"))),
        ("choice", "_from_italics") => json!(m::choice::_from_italics(&text())),
        ("choice", "_from_labels") => json!(m::choice::_from_labels(&text())),
        ("choice", "_from_list") => json!(m::choice::_from_list(&text())),
        ("choice", "_from_question") => json!(m::choice::_from_question(&text())),
        ("choice", "_segment") => s(m::choice::_segment(&text())),
        ("choice", "_the_thing_in") => s(m::choice::_the_thing_in(&a.s("line"))),
        ("choice", "settled_on") => s(m::choice::settled_on(&mem)),
        ("swap", "thing") => os(m::swap::thing(&text())),
        ("swap", "someone") => os(m::swap::someone(&text())),
        ("swap", "_said") => s(m::swap::_said(&text())),
        ("swap", "_after_a_take_back") => os(m::swap::_after_a_take_back(&a.s("said"))),
        ("swap", "is_repair") => J::Bool(m::swap::is_repair(&text())),
        ("swap", "rewritten") => m::swap::rewritten(&text(), &mem).map(pair).unwrap_or(J::Null),
        ("swap", "rewritten_person") => m::swap::rewritten_person(&text(), &mem).map(pair).unwrap_or(J::Null),
        ("swap", "candidates") => J::Array(m::swap::candidates(&text(), &mem, a.n("most") as usize).into_iter().map(pair).collect()),
        ("swap", "names") => J::Bool(m::swap::names(&a.s("old"), &a.s("answer"), &text(), opt_mem(&mem).as_ref(), &a.s("question"))),
        ("swap", "names_person") => J::Bool(m::swap::names_person(&a.s("old"), &a.s("answer"), &text(), &a.s("last"))),
        ("swap", "placed") => os(m::swap::placed(&a.s("question"), &a.s("old"), &a.s("new"))),
        ("swap", "_about") => os(m::swap::_about(&a.s("question"))),
        ("history", "asked") => os(m::history::asked(&text(), None)),
        ("history", "the_thing_named") => s(m::history::the_thing_named(&text())),
        ("history", "the_thing_they_named") => s(m::history::the_thing_they_named(&text())),
        ("history", "_flat") => s(m::history::_flat(&text())),
        ("history", "_nothing_said") => J::Bool(m::history::_nothing_said(&text())),
        ("history", "about_the_chat") => J::Bool(m::history::about_the_chat(&text())),
        ("history", "not_a_subject") => J::Bool(m::history::not_a_subject(&text())),
        ("history", "brings_nothing_of_its_own") => J::Bool(m::history::brings_nothing_of_its_own(&text())),
        ("history", "an_earlier_turn") => J::Bool(m::history::an_earlier_turn(&mem, &text())),
        ("history", "a_past_that_is_not_there") => {
            let said = a.j("said").as_str().map(str::to_string);
            J::Bool(m::history::a_past_that_is_not_there(&a.s("reply"), Some(&mem), said.as_deref()))
        }
        ("history", "_earlier_turns") => J::Array(m::history::_earlier_turns(Some(&mem), &text()).into_iter().map(turn).collect()),
        ("history", "turns_before") => J::Array(m::history::turns_before(&mem).iter().map(turn).collect()),
        ("history", "the_answer_on_screen") => s(m::history::the_answer_on_screen(&mem, a.n("within"), &a.strs("modules"))),
        ("history", "_a_hedge") => J::Bool(m::history::_a_hedge(&a.s("said"))),
        ("history", "_a_note_not_an_answer") => J::Bool(m::history::_a_note_not_an_answer(&a.s("said"))),
        ("history", "topic_lines") => json!(m::history::topic_lines(&mem, a.n("limit") as usize)),
        ("history", "first_bot_line") => s(m::history::first_bot_line(&mem)),
        ("cause", "answer_for") => os(m::cause::answer_for(&text(), &mem)),
        ("cause", "expand") => os(m::cause::expand(&text(), &mem)),
        ("cause", "_a_thing") => os(m::cause::_a_thing(&text())),
        ("cause", "_flat") => s(m::cause::_flat(&text())),
        ("cause", "_only_a_nod") => J::Bool(m::cause::_only_a_nod(&text())),
        ("cause", "_already_has_it") => J::Bool(m::cause::_already_has_it(&a.s("said"), &a.s("thing"))),
        ("cause", "_the_state_they_reported") => match m::cause::_the_state_they_reported(&mem, a.n("back") as usize) {
            Some((line, word)) => json!([line, word]),
            None => json!([null, null]),
        },
        ("cause", "_the_bot_just_spoke") => s(m::cause::_the_bot_just_spoke(&mem)),
        ("cause", "_asks_about_the_bot") => J::Bool(m::cause::_asks_about_the_bot(&text(), &mem)),
        ("limit", "words_of") => json!(m::limit::words_of(&text())),
        ("limit", "reads_as_a_limit") => match m::limit::reads_as_a_limit(&text(), opt_mem(&mem).as_ref()) {
            Some((k, t, b)) => json!([k, t, b]),
            None => J::Null,
        },
        ("limit", "reads_the_answer") => J::Bool(m::limit::reads_the_answer(&text(), &mem)),
        ("limit", "after_an_answer") => J::Bool(m::limit::after_an_answer(&mem)),
        ("limit", "reply_for") => os(m::limit::reply_for(&text(), &mut mem)),
        ("limit", "move_after_a_limit") => os(m::limit::move_after_a_limit(&text(), &mut mem)),
        ("limit", "_limit_reply") => os(m::limit::_limit_reply(&text(), &mem)),
        ("limit", "only_a_limit") => J::Bool(m::limit::only_a_limit(&text(), None)),
        ("limit", "_steps") => J::Array(m::limit::_steps(&a.s("answer")).into_iter().map(pair).collect()),
        ("already", "the_step_they_took") => os(m::already::the_step_they_took(&text(), opt_mem(&mem).as_ref())),
        ("already", "the_trouble_is_over") => os(m::already::the_trouble_is_over(&text())),
        ("already", "reply_for") => os(m::already::reply_for(&text(), &mem)),
        ("already", "takes_as_it_comes") => J::Bool(m::already::takes_as_it_comes(&text(), &mem)),
        ("already", "tells_them_the_step_again") => J::Bool(m::already::tells_them_the_step_again(&a.s("reply"), &text(), &mem)),
        ("already", "after_the_step_handed_back") => os(m::already::after_the_step_handed_back(&text(), &mem)),
        ("already", "_the_step") => os(m::already::_the_step(&a.s("low"))),
        ("already", "_the_step_after_did") => os(m::already::_the_step_after_did(&a.s("low"))),
        ("already", "_the_finished_step") => os(m::already::_the_finished_step(&a.s("low"), opt_mem(&mem).as_ref())),
        ("already", "_the_over_word") => {
            let (word, at, low) = m::already::_the_over_word(&text());
            let at = at.map(|b| rsbot::py::text::char_index(&low, b) as i64).unwrap_or(-1);
            json!([word, at, low])
        }
        ("carry", "the_subject_in_the_condition") => os(m::carry::the_subject_in_the_condition(&text(), &mut mem)),
        ("carry", "the_verdict_in_the_chat") => os(m::carry::the_verdict_in_the_chat(&text(), &mut mem)),
        ("carry", "the_quality_in_the_chat") => os(m::carry::the_quality_in_the_chat(&text(), &mut mem)),
        ("carry", "the_limit_in_the_chat") => os(m::carry::the_limit_in_the_chat(&text(), &mut mem)),
        ("carry", "_a_verdict_asked") => J::Bool(m::carry::_a_verdict_asked(&a.s("clean"))),
        ("carry", "_the_quality") => os(m::carry::_the_quality(&a.s("line"))),
        ("repair", "the_thing_they_meant") => os(m::repair::the_thing_they_meant(&text(), &mut mem)),
        ("repair", "keep_the_correction") => {
            m::repair::keep_the_correction(&text(), &mut mem);
            J::Null
        }
        ("repair", "_the_thing") => os(m::repair::_the_thing(&text())),
        ("repair", "_the_thing_the_last_line_named") => os(m::repair::_the_thing_the_last_line_named(&a.s("line"))),
        ("repair", "_swapped") => s(m::repair::_swapped(&a.s("line"), &a.s("old"), &a.s("thing"))),
        ("told", "is_a_boundary") => J::Bool(m::told::is_a_boundary(&text())),
        ("told", "_tells_nothing") => J::Bool(m::told::_tells_nothing(&text())),
        ("told", "_the_ask") => os(m::told::_the_ask(&text())),
        ("told", "_the_remembered") => os(m::told::_the_remembered(&text())),
        ("told", "_a_claim_about_the_chat") => J::Bool(m::told::_a_claim_about_the_chat(&text())),
        ("told", "ABOUT_THE_CHAT") => J::Bool(m::told::ABOUT_THE_CHAT(&text(), None)),
        ("told", "A_CALLBACK") => J::Bool(m::told::A_CALLBACK(&text(), None)),
        ("told", "a_callback_the_chat_has") => J::Bool(m::told::a_callback_the_chat_has(&text(), &mem)),
        ("told", "check") => os(m::told::check(&text(), &mut mem)),
        ("told", "_a_claim_that_points") => os(m::told::_a_claim_that_points(&text(), &mem)),
        ("told", "_with_the_yes") => s(m::told::_with_the_yes(&a.s("said"), &text())),
        ("told", "answer") | ("told", "callback") | ("told", "_the_answer") => {
            let facts = rsbot::memory::store_replies::_told_readers();
            let plain = Value::Fn(std::sync::Arc::new(rsbot::data::FnRef {
                id: "bot.memory.store_replies:_respond".into(),
                cells: Vec::new(),
                defaults: Vec::new(),
            }));
            let claim_text = a.j("claims").get("cells").and_then(|c| c.get("text")).and_then(J::as_str).unwrap_or("").to_string();
            let claims = Value::Fn(std::sync::Arc::new(rsbot::data::FnRef {
                id: "bot.memory.store_replies:_about_this_chat".into(),
                cells: vec![("text".into(), Value::from(claim_text))],
                defaults: Vec::new(),
            }));
            match func {
                "answer" => s(m::told::answer(&text(), &mut mem, &facts, &plain, &claims)),
                "callback" => os(m::told::callback(&text(), &mut mem, &facts, &plain, &claims)),
                _ => os(m::told::_the_answer(&text(), &mut mem, &facts, &plain, &claims)),
            }
        }
        ("decided", "answer") => os(m::decided::answer(&text(), &mem)),
        ("decided", "asks") => J::Bool(m::decided::asks(&text())),
        ("decided", "extract") => os(m::decided::extract(&text())),
        ("decided", "read") => match m::decided::read(&text()) {
            Some(r) if r.corrects => json!({"what": r.what, "settled": r.settled, "corrects": true}),
            Some(r) => json!({"what": r.what, "settled": r.settled}),
            None => J::Null,
        },
        ("decided", "observe") => m::decided::observe(&text(), &mut mem).map(|v| v.to_plain()).unwrap_or(J::Null),
        ("decided", "the_guess") => os(m::decided::the_guess(&text())),
        ("decided", "_the_thing") => os(m::decided::_the_thing(&a.s("phrase"))),
        ("counts", "observe") => {
            m::counts::observe(&text(), &mut mem);
            J::Null
        }
        ("counts", "answer") => os(m::counts::answer(&text(), &mem)),
        ("counts", "_counted") => json!(m::counts::_counted(&a.s("rest"))),
        ("counts", "_head") => json!(m::counts::_head(&a.s("after"))),
        ("counts", "_thing") => s(m::counts::_thing(&a.s("word"))),
        ("counts", "_said_number") => json!(m::counts::_said_number(&a.s("said"))),
        ("counts", "_the_group") => json!(m::counts::_the_group(&a.s("said"))),
        ("counts", "_the_party") => json!(m::counts::_the_party(&a.s("said"))),
        ("price", "judged") => os(m::price::judged(&text(), &mem)),
        ("price", "judged_with_the_card") => os(m::price::judged_with_the_card(&text(), a.j("answer").as_str(), &mem)),
        ("price", "_prices_in") => J::Array(m::price::_prices_in(&text()).into_iter().map(|(v, s)| json!([v, s])).collect()),
        ("price", "_a_price_ask") => return Some((J::Bool(m::price::_a_price_ask(&text())), None)),
        ("price", "_money") => s(m::price::_money(a.f("value"))),
        ("price", "_the_thing_priced") => s(m::price::_the_thing_priced(&text())),
        ("spent", "answer") => os(m::spent::answer(&text(), &mem)),
        ("spent", "what_is_left") => os(m::spent::what_is_left(&text(), &mem)),
        ("spent", "readings") => J::Array(m::spent::readings(&a.s("line")).into_iter().map(|s| json!({"amount": s.amount, "thing": s.thing})).collect()),
        ("spent", "_held_in") => os(m::spent::_held_in(&a.s("line"))),
        ("spent", "_plain") => s(m::spent::_plain(&text())),
        ("outgoings", "reply_for") => {
            let had = a.has_mem("memory");
            s(m::outgoings::reply_for(&text(), if had { Some(&mut mem) } else { None }))
        }
        ("outgoings", "the_income_in_the_line") => m::outgoings::the_income_in_the_line(&text()).map(|(v, u)| json!([v, u])).unwrap_or(J::Null),
        ("outgoings", "_sums") => J::Array(m::outgoings::_sums(&text()).into_iter().map(|s| json!([s.value, s.start, s.end])).collect()),
        ("outgoings", "the_outgoings_in_the_line") => J::Array(m::outgoings::the_outgoings_in_the_line(&text()).into_iter().map(|(n, v)| json!([n, v])).collect()),
        ("outgoings", "_pounds") => s(m::outgoings::_pounds(a.f("value"))),
        _ => return None,
    };
    let after = if a.has_mem("memory") { Some(written(&mem)) } else { None };
    Some((r, after))
}

/// Python's value and Rust's are the same answer: numbers by value, None as "", a truthy
/// match object as true.
fn same(a: &J, b: &J) -> bool {
    match (a, b) {
        (J::Null, J::String(s)) | (J::String(s), J::Null) => s.is_empty(),
        (J::Number(x), J::Number(y)) => x.as_f64() == y.as_f64(),
        (J::Array(x), J::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same(p, q)),
        (J::Object(x), J::Object(y)) => x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| same(v, w))),
        (J::Object(_), J::Bool(true)) => true,
        (J::Null, J::Bool(false)) => true,
        _ => a == b,
    }
}

fn main() {
    rsbot::runtime::quiet_panics();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut paths, mut fns, mut show) = (Vec::new(), Vec::new(), 3usize);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--fn" => {
                fns.push(args[i + 1].clone());
                i += 1;
            }
            "--show" => {
                show = args[i + 1].parse().unwrap();
                i += 1;
            }
            x => paths.push(x.to_string()),
        }
        i += 1;
    }
    // fn -> [same, differ, unported, not compared, no case]
    let mut tally: BTreeMap<String, [usize; 5]> = BTreeMap::new();
    let mut shown: BTreeMap<String, usize> = BTreeMap::new();
    for path in &paths {
        let reader = std::io::BufReader::new(flate2::read::GzDecoder::new(std::fs::File::open(path).unwrap()));
        for line in reader.lines() {
            let Ok(line) = line else { break };
            let Ok(row) = serde_json::from_str::<J>(&line) else { break };
            let name = row["fn"].as_str().unwrap().to_string();
            if name.contains("<locals>") || (!fns.is_empty() && !fns.iter().any(|f| name.contains(f.as_str()))) {
                continue;
            }
            let t = tally.entry(name.clone()).or_default();
            if row.get("rng").is_some() {
                t[3] += 1;
                continue;
            }
            let empty = serde_json::Map::new();
            let a = Args(row["args"].as_object().unwrap_or(&empty));
            let got = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| call(&name, &a)));
            let raised = row["raised"].is_string();
            let mut report = |kind: &str, detail: String| {
                let n = shown.entry(name.clone()).or_default();
                if *n < show {
                    *n += 1;
                    println!("{kind} {name} args={}\n{detail}", serde_json::to_string(&row["args"]).unwrap().chars().take(600).collect::<String>());
                }
            };
            match got {
                Ok(None) => t[4] += 1,
                Ok(Some((j, after))) => {
                    let want = &row["result"];
                    let want_after = row.get("after").and_then(|a| a.get("memory")).map(written_j).or_else(|| a.0.get("memory").filter(|m| m.is_object()).map(written_j));
                    let after_ok = match (&after, &want_after) {
                        (Some(x), Some(y)) => same(x, y),
                        _ => true,
                    };
                    if !raised && same(want, &j) && after_ok {
                        t[0] += 1;
                    } else {
                        t[1] += 1;
                        let mut detail = format!("  python: {}{}\n  rust:   {}", if raised { "RAISED " } else { "" }, want, j);
                        if !after_ok {
                            detail += &format!("\n  python memory: {}\n  rust memory:   {}", want_after.unwrap(), after.unwrap());
                        }
                        report("DIFF", detail);
                    }
                }
                Err(e) => {
                    let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                    if msg.contains("UNPORTED") {
                        t[2] += 1;
                    } else if raised {
                        t[0] += 1;
                    } else {
                        t[1] += 1;
                        report("PANIC", format!("  {msg}"));
                    }
                }
            }
        }
    }
    let mut total = [0usize; 5];
    println!("\n{:<58} {:>6} {:>6} {:>8} {:>6} {:>6}", "function", "same", "differ", "unported", "notcmp", "nocase");
    for (name, t) in &tally {
        for k in 0..5 {
            total[k] += t[k];
        }
        if t[0] + t[1] + t[2] > 0 {
            println!("{:<58} {:>6} {:>6} {:>8} {:>6} {:>6}", name, t[0], t[1], t[2], t[3], t[4]);
        }
    }
    println!("{:<58} {:>6} {:>6} {:>8} {:>6} {:>6}", "TOTAL", total[0], total[1], total[2], total[3], total[4]);
}
