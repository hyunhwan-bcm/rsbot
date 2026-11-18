//! Function-level checks of the MEM8 ports against PyBot: each case in `mem8_cases.jsonl` was
//! evaluated by PyBot (`python3 harness/pycall.py`-style, clock pinned to 2026-11-17) and is run
//! here through the Rust port. A case that reaches a function another workstream has not ported
//! yet (UNPORTED) is counted as skipped, not failed.

use crate::data::Value;
use crate::state::{Memory, Turn};
use serde_json::{json, Value as J};

fn s(a: &[J], i: usize) -> String {
    a[i].as_str().unwrap_or("").to_string()
}
fn mem(a: &[J], i: usize) -> Memory {
    Memory::from_plain(a[i].as_object().expect("memory dict"))
}
fn strs(a: &[J], i: usize) -> Vec<String> {
    a[i].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect()
}
fn turns(a: &[J], i: usize) -> Vec<Turn> {
    let mut m = serde_json::Map::new();
    m.insert("turns".into(), a[i].clone());
    Memory::from_plain(&m).turns
}
fn opt(o: Option<String>) -> J {
    o.map(J::String).unwrap_or(J::Null)
}
fn pair<A: Into<J>, B: Into<J>>(o: Option<(A, B)>) -> J {
    o.map(|(x, y)| json!([x.into(), y.into()])).unwrap_or(J::Null)
}

fn run(m: &str, f: &str, a: &[J]) -> J {
    use super::*;
    match (m, f) {
        ("a_year_of_birth", "_a_slip") => json!(a_year_of_birth::_a_slip(&s(a, 0))),
        ("asked_about", "_the_words_asked") => json!(asked_about::_the_words_asked(&s(a, 0))),
        ("asked_about", "_sentences") => json!(asked_about::_sentences(&s(a, 0))),
        ("asked_about", "_the_part_that_answers") => json!(asked_about::_the_part_that_answers(&s(a, 0), &strs(a, 1), &s(a, 2))),
        ("baking", "written_out") => opt(baking::written_out(&s(a, 0), &mem(a, 1))),
        ("baking", "_the_name") => json!(baking::_the_name(&s(a, 0))),
        ("doing_it", "_is_doing") => json!(doing_it::_is_doing(&s(a, 0))),
        ("doing_it", "doing_word") => opt(doing_it::doing_word(&s(a, 0))),
        ("doing_it", "asks_about_doing") => json!(doing_it::asks_about_doing(&s(a, 0))),
        ("doing_it", "written_out") => json!(doing_it::written_out(&s(a, 0), &mem(a, 1))),
        ("doing_it", "the_ask_behind_a_moan") => json!(doing_it::the_ask_behind_a_moan(&s(a, 0))),
        ("doubt", "the_doubt") => json!(doubt::the_doubt(&s(a, 0), &mem(a, 1))),
        ("filler", "without_filler") => json!(filler::without_filler(&s(a, 0))),
        ("filler", "without_the_end_shrug") => json!(filler::without_the_end_shrug(&s(a, 0))),
        ("filler", "without_the_shrug") => json!(filler::without_the_shrug(&s(a, 0))),
        ("goals", "_tidy") => json!(goals::_tidy(&s(a, 0))),
        ("goals", "the_loss_that_was_the_plan") => json!(goals::the_loss_that_was_the_plan(&s(a, 0), &mut mem(a, 1))),
        ("goals", "a_progress_asked") => json!(goals::a_progress_asked(&s(a, 0), &mem(a, 1))),
        ("groupages", "the_list_given") => pair(groupages::the_list_given(&s(a, 0))),
        ("groupages", "reply_for") => json!(groupages::reply_for(&s(a, 0), &mem(a, 1))),
        ("hope", "reading") => json!(hope::reading(&s(a, 0))),
        ("hope", "answer_for") => opt(hope::answer_for(&s(a, 0), &mut mem(a, 1))),
        ("hours", "remembered") => opt(hours::remembered(&mem(a, 0))),
        ("kidcount", "_asks") => json!(kidcount::_asks(&s(a, 0))),
        ("kidcount", "reply_for") => opt(kidcount::reply_for(&s(a, 0), &mem(a, 1))),
        ("kinds", "the_kind_asked") => json!(kinds::the_kind_asked(&s(a, 0))),
        ("kinds", "the_line_that_gives") => kinds::the_line_that_gives(&s(a, 0), &strs(a, 1)).map(|v| v.to_plain()).unwrap_or(J::Null),
        ("limits", "names_a_limit") => opt(limits::names_a_limit(&s(a, 0))),
        ("limits", "_the_trip_is_on") => json!(limits::_the_trip_is_on(&s(a, 0), &mem(a, 1))),
        ("more", "asks_for_more") => json!(more::asks_for_more(&s(a, 0))),
        ("more", "_about_what") => json!(more::_about_what(&s(a, 0))),
        ("more", "_is_what_was_asked") => json!(more::_is_what_was_asked(&s(a, 0), &s(a, 1))),
        ("more", "_its_ask") => json!(more::_its_ask(&s(a, 0))),
        ("more", "_as_a_line") => json!(more::_as_a_line(&s(a, 0))),
        ("more", "_flat") => json!(more::_flat(&s(a, 0))),
        ("not_a_trip", "a_doing_after_to") => json!(not_a_trip::a_doing_after_to(&strs(a, 0))),
        ("not_a_trip", "a_place_of_ordinary_life") => json!(not_a_trip::a_place_of_ordinary_life(&s(a, 0))),
        ("not_a_trip", "a_person") => json!(not_a_trip::a_person(&s(a, 0))),
        ("not_a_trip", "the_visit") => json!(not_a_trip::the_visit(&s(a, 0))),
        ("not_this", "_in_plain_words") => json!(not_this::_in_plain_words(&s(a, 0))),
        ("not_this", "the_place_ruled_out") => json!(not_this::the_place_ruled_out(&s(a, 0), &mem(a, 1))),
        ("not_this", "_a_place_the_chat_knows") => json!(not_this::_a_place_the_chat_knows(&s(a, 0))),
        ("onset", "reads_as_onset") => onset::reads_as_onset(&s(a, 0)).map(|(c, d)| json!([c, d])).unwrap_or(J::Null),
        ("onset", "_clause") => json!(onset::_clause(&s(a, 0))),
        ("opening", "_who") => {
            let (w, b) = opening::_who(&s(a, 0));
            json!([w, b])
        }
        ("opening", "_names_my") => json!(opening::_names_my(&s(a, 0), &s(a, 1))),
        ("opening", "_said_before") => json!(opening::_said_before(&mem(a, 0))),
        ("passing_the_time", "_the_chat_is_heavy") => json!(passing_the_time::_the_chat_is_heavy(&mem(a, 0))),
        ("pending", "_clauses") => json!(pending::_clauses(&s(a, 0))),
        ("pending", "_what_is_missing") => pair(pending::_what_is_missing(&s(a, 0))),
        ("pending", "_the_occasion") => opt(pending::_the_occasion(&s(a, 0), &s(a, 1))),
        ("pending", "as_question") => opt(pending::as_question(&s(a, 0))),
        ("prefer", "_taste") => json!(prefer::_taste(&s(a, 0))),
        ("prefer", "_said_the_medium") => json!(prefer::_said_the_medium(&s(a, 0))),
        ("prefer", "follow_up") => json!(prefer::follow_up(&s(a, 0), &mem(a, 1))),
        ("purse", "_money_in") => json!(purse::_money_in(&s(a, 0))),
        ("purse", "_the_money") => json!(purse::_the_money(a[0].as_f64().unwrap())),
        ("purse", "_money_bands") => json!(purse::_money_bands(&s(a, 0)).into_iter().map(|(l, h, w)| json!([l, h, w])).collect::<Vec<_>>()),
        ("purse", "against_the_bands") => opt(purse::against_the_bands(&s(a, 0), &turns(a, 1))),
        ("quotes", "left_open") => json!(quotes::left_open(&s(a, 0))),
        ("quotes", "closed") => json!(quotes::closed(&s(a, 0), &strs(a, 1), &s(a, 2))),
        ("remedy", "_the_thing") => json!(remedy::_the_thing(&s(a, 0), &s(a, 1))),
        ("remedy", "reply_for") => json!(remedy::reply_for(&s(a, 0), &mem(a, 1))),
        ("reminded", "_trim") => json!(reminded::_trim(&s(a, 0))),
        ("reminded", "_the_thing_itself") => json!(reminded::_the_thing_itself(&s(a, 0))),
        ("returned", "reads_as_returning") => json!(returned::reads_as_returning(&s(a, 0))),
        ("returned", "the_next_step") => opt(returned::the_next_step(&s(a, 0))),
        ("returned", "says_it_happened_again") => json!(returned::says_it_happened_again(&s(a, 0))),
        ("returned", "pool_for") => opt(returned::pool_for(&s(a, 0), &mem(a, 1))),
        ("setback", "reading") => setback::reading(&s(a, 0)).map(|(n, t)| json!([n, t])).unwrap_or(J::Null),
        ("setback", "pool") => json!(setback::pool(&s(a, 0))),
        ("size", "_the_rule") => json!(size::_the_rule(&s(a, 0))),
        ("size", "_one_of_ours") => json!(size::_one_of_ours(&s(a, 0))),
        ("spoken", "_spoken_as_numbers") => json!(spoken::_spoken_as_numbers(&s(a, 0))),
        ("stuck", "the_ask_in") => json!(stuck::the_ask_in(&s(a, 0))),
        ("stuck", "_opens_with_an_emergency") => json!(stuck::_opens_with_an_emergency(&s(a, 0))),
        ("stuck", "pool") => json!(stuck::pool(&s(a, 0), &mem(a, 1))),
        ("stuck", "_the_answer_gave_a_step") => json!(stuck::_the_answer_gave_a_step(&mem(a, 0))),
        ("the_verdict", "answer_for") => opt(the_verdict::answer_for(&s(a, 0), &mem(a, 1))),
        ("the_years", "_the_turn") => opt(the_years::_the_turn(&s(a, 0))),
        ("their_feeling", "_says_something") => json!(their_feeling::_says_something(&s(a, 0))),
        ("too_late", "_two_clauses") => json!(too_late::_two_clauses(&s(a, 0))),
        ("warmth", "_nothing_said_before") => json!(warmth::_nothing_said_before(&mem(a, 0))),
        ("whathurts", "which_one_is_a_side") => json!(whathurts::which_one_is_a_side(&s(a, 0))),
        ("whathurts", "the_part_asked") => opt(whathurts::the_part_asked(&mem(a, 0), &s(a, 1))),
        ("whathurts", "what_is_hurting_reply") => opt(whathurts::what_is_hurting_reply(&mem(a, 0), &s(a, 1))),
        ("whereabout", "the_place") => json!(whereabout::the_place(&s(a, 0))),
        ("whereabout", "the_question_with_the_place") => json!(whereabout::the_question_with_the_place(&s(a, 0), &mem(a, 1))),
        ("whose_body", "somebody_elses_body") => json!(whose_body::somebody_elses_body(&s(a, 0), &mem(a, 1))),
        ("whose_body", "a_body_of_a_pronoun") => json!(whose_body::a_body_of_a_pronoun(&s(a, 0))),
        ("will_it_work", "asks_if_it_will_work") => json!(will_it_work::asks_if_it_will_work(&s(a, 0))),
        ("will_it_work", "_what_it_achieves") => json!(will_it_work::_what_it_achieves(&s(a, 0))),
        ("workplace", "_the_place_the_line_said") => pair(workplace::_the_place_the_line_said(&s(a, 0))),
        ("workplace", "_a_clean_place") => json!(workplace::_a_clean_place(&s(a, 0))),
        ("workplace", "the_place_the_person_works") => opt(workplace::the_place_the_person_works(&s(a, 0), &mem(a, 1))),
        ("workplace", "_with_an_article") => json!(workplace::_with_an_article(&s(a, 0))),
        _ => panic!("no dispatcher for {m}.{f}"),
    }
}

/// Numbers compare as numbers (Python's 1200.0 and Rust's 1200.0 may print differently).
fn same(a: &J, b: &J) -> bool {
    match (a, b) {
        (J::Number(x), J::Number(y)) => x.as_f64() == y.as_f64(),
        (J::Array(x), J::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same(p, q)),
        _ => a == b,
    }
}

#[test]
fn mem8_cases_match_pybot() {
    let _ = Value::None;
    let (mut ok, mut skipped, mut bad) = (0, 0, Vec::new());
    for line in include_str!("mem8_cases.jsonl").lines() {
        let case: J = serde_json::from_str(line).unwrap();
        let (m, f) = (case["m"].as_str().unwrap(), case["f"].as_str().unwrap());
        let args = case["a"].as_array().unwrap().clone();
        let got = std::panic::catch_unwind(|| run(m, f, &args));
        match got {
            Ok(got) if same(&got, &case["r"]) => ok += 1,
            Ok(got) => bad.push(format!("{m}.{f}({}) = {got}, PyBot {}", case["a"], case["r"])),
            Err(e) => {
                let msg = e
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_default();
                if msg.starts_with("UNPORTED") {
                    skipped += 1;
                } else {
                    bad.push(format!("{m}.{f}({}) panicked: {msg}", case["a"]));
                }
            }
        }
    }
    eprintln!("MEM8 cases: {ok} ok, {skipped} skipped (unported deps), {} differ", bad.len());
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
