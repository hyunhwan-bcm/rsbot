//! PyBot's own values for the pure readers of the MOD4 modules, on lines the turn oracles do not
//! reach. The cases are in `mod4_cases.json`, written by `harness/mod4_cases.py` (which calls the
//! Python function of the same name on the same arguments); this calls the Rust ports.

use super::*;
use crate::data::Value;
use crate::state::Memory;
use crate::understood::Understood;
use serde_json::{json, Value as J};

fn s(a: &[J], i: usize) -> &str {
    a[i].as_str().unwrap_or("")
}

fn strs(a: &[J], i: usize) -> Vec<String> {
    a.get(i).and_then(J::as_array).map(|v| v.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect()).unwrap_or_default()
}

/// The memory a case hands over: the turns and the store as they are, and a reading with the
/// one or two keys the case sets.
fn mem(a: &[J], i: usize) -> Memory {
    let mut plain = a[i].as_object().cloned().unwrap_or_default();
    let understood = plain.remove("understood");
    let mut memory = Memory::from_plain(&plain);
    if let Some(read) = understood {
        let mut reading = Understood::default();
        if let Some(kind) = read.get("kind").and_then(J::as_str) {
            reading.kind = kind.to_string();
        }
        reading.about_bot_kind = read.get("about_bot_kind").and_then(J::as_str).map(str::to_string);
        memory.understood = Some(reading);
    }
    memory
}

fn sorted(set: std::collections::HashSet<String>) -> J {
    let mut words: Vec<String> = set.into_iter().collect();
    words.sort();
    json!(words)
}

fn pairs(figures: Vec<(String, bool)>) -> J {
    json!(figures.into_iter().map(|(words, definite)| json!([words, definite])).collect::<Vec<_>>())
}

fn call(name: &str, a: &[J]) -> J {
    match name {
        "particles.a_thing_without_its_tail" => json!(particles::a_thing_without_its_tail(s(a, 0))),
        "particles.a_job_word" => json!(particles::a_job_word(s(a, 0))),
        "memory_store._the_unit_asked" => json!(memory_store::_the_unit_asked(s(a, 0))),
        "memory_store._budget_sign" => json!(memory_store::_budget_sign(s(a, 0), s(a, 1))),
        "memory_store._names_this" => json!(memory_store::_names_this(s(a, 0), s(a, 1))),
        "memory_store._plan_names" => json!(memory_store::_plan_names(s(a, 0), s(a, 1))),
        "memory_store._without_a_lead_in" => json!(memory_store::_without_a_lead_in(s(a, 0))),
        "memory_store._the_ask_behind_the_recall" => json!(memory_store::_the_ask_behind_the_recall(s(a, 0)).map(|(ask, slot, who)| vec![ask, slot, who])),
        "memory_store._a_fact_word" => json!(memory_store::_a_fact_word(s(a, 0))),
        "memory_store._names_nothing_in_a_recall" => json!(memory_store::_names_nothing_in_a_recall(s(a, 0))),
        "memory_store._asks_about_someone_else" => json!(memory_store::_asks_about_someone_else(s(a, 0))),
        "memory_store._another_animal_is_named" => json!(memory_store::_another_animal_is_named(s(a, 0))),
        "memory_store._the_money_figures" => pairs(memory_store::_the_money_figures(s(a, 0))),
        "memory_store._the_figures_in" => pairs(memory_store::_the_figures_in(s(a, 0), &strs(a, 1))),
        "memory_store._a_figure_of_those_units" => json!(memory_store::_a_figure_of_those_units(s(a, 0), &strs(a, 1))),
        "memory_store._a_person" => json!(memory_store::_a_person(&a[0].as_str().map(Value::from).unwrap_or(Value::None))),
        "memory_store._the_clause" => json!(memory_store::_the_clause(s(a, 0), a[1].as_u64().unwrap() as usize, a[2].as_u64().unwrap() as usize)),
        "bulky._group" => {
            let (group, thing) = bulky::_group(s(a, 0));
            json!([group, thing])
        }
        "bulky._reply" => json!(bulky::_reply(s(a, 0), s(a, 1))),
        "bulky.respond" => json!(bulky::respond(s(a, 0), &mut mem(a, 1))),
        "a_figure_of_a_dish.respond" => json!(a_figure_of_a_dish::respond(s(a, 0), &mut mem(a, 1))),
        "record_due._the_record" => json!(record_due::_the_record(s(a, 0))),
        "record_due.respond" => json!(record_due::respond(s(a, 0), &mut mem(a, 1))),
        "live_source.respond" => json!(live_source::respond(s(a, 0), &mut mem(a, 1))),
        "live_source._without_lead_in" => json!(live_source::_without_lead_in(s(a, 0))),
        "a_letter_about_money.respond" => json!(a_letter_about_money::respond(s(a, 0), &mut mem(a, 1))),
        "a_letter_about_money._ask_for_what_is_missing" => json!(a_letter_about_money::_ask_for_what_is_missing(s(a, 0), s(a, 1), s(a, 2))),
        "judge_ask.reads_it" => json!(judge_ask::reads_it(s(a, 0))),
        "judge_ask._says_nothing" => json!(judge_ask::_says_nothing(s(a, 0))),
        "what_they_said._details" => sorted(what_they_said::_details(s(a, 0))),
        "what_they_said._the_kind_of_answer" => json!(what_they_said::_the_kind_of_answer(s(a, 0))),
        "what_they_said._the_reported_clause" => json!(what_they_said::_the_reported_clause(s(a, 0))),
        "what_they_said._the_figure_the_line_gave" => json!(what_they_said::_the_figure_the_line_gave(s(a, 0))),
        "words._quoted" => json!(words::_quoted(s(a, 0))),
        "words._names_its_own" => json!(words::_names_its_own(s(a, 0))),
        "hardpast._the_hard_thing" => json!(hardpast::_the_hard_thing(s(a, 0), &mem(a, 1))),
        "how_long_a_stay._the_opening" => json!(how_long_a_stay::_the_opening(s(a, 0))),
        "how_long_a_stay._a_subject_of_its_own" => json!(how_long_a_stay::_a_subject_of_its_own(s(a, 0))),
        "how_long_a_stay._the_length_said" => json!(how_long_a_stay::_the_length_said(&mem(a, 0))),
        "familyask.the_relation" => json!(familyask::the_relation(s(a, 0))),
        "familyask.the_class" => json!(familyask::the_class(s(a, 0))),
        "familyask.is_a_family_ask" => json!(familyask::is_a_family_ask(s(a, 0))),
        "thing_i_did_with._thing" => json!(thing_i_did_with::_thing(s(a, 0))),
        "thing_i_did_with._did" => json!(thing_i_did_with::_did(s(a, 0))),
        "thing_i_did_with._stem" => json!(thing_i_did_with::_stem(s(a, 0))),
        "a_fact_asked_again._the_date_in" => json!(a_fact_asked_again::_the_date_in(s(a, 0))),
        "a_fact_asked_again._the_noun_asked" => json!(a_fact_asked_again::_the_noun_asked(s(a, 0))),
        "a_fact_asked_again._a_name" => json!(a_fact_asked_again::_a_name(s(a, 0))),
        "a_fact_asked_again.respond" => json!(a_fact_asked_again::respond(s(a, 0), &mut mem(a, 1))),
        "a_job_that_is_not_a_diy_job._the_job" => json!(a_job_that_is_not_a_diy_job::_the_job(s(a, 0)).map(|(who, why)| vec![who, why])),
        "a_job_that_is_not_a_diy_job._has" => json!(a_job_that_is_not_a_diy_job::_has(s(a, 0), &strs(a, 1))),
        "a_shock_at_a_bill._the_ending_and_the_page" => json!(a_shock_at_a_bill::_the_ending_and_the_page(s(a, 0))),
        "a_shock_at_a_bill._a_complaint_added_after_the_ending" => json!(a_shock_at_a_bill::_a_complaint_added_after_the_ending(s(a, 0))),
        "a_shock_at_a_bill._the_ending_is_somebody_elses" => json!(a_shock_at_a_bill::_the_ending_is_somebody_elses(s(a, 0))),
        "a_shock_at_a_bill._a_kind_word_in_front_of_the_page" => json!(a_shock_at_a_bill::_a_kind_word_in_front_of_the_page(s(a, 0))),
        "a_shock_at_a_bill._a_bill_of_another_kind" => json!(a_shock_at_a_bill::_a_bill_of_another_kind(s(a, 0))),
        "a_shock_at_a_bill._ask_for_what_is_missing" => json!(a_shock_at_a_bill::_ask_for_what_is_missing(&mem(a, 0), s(a, 1))),
        "a_figure_beside_the_state.a_figure_of_heat" => json!(a_figure_beside_the_state::a_figure_of_heat(s(a, 0))),
        "a_figure_beside_the_state.without_the_figure" => json!(a_figure_beside_the_state::without_the_figure(s(a, 0))),
        "keeping._seed" => json!(keeping::_seed(s(a, 0))),
        "keeping._rotate" => json!(keeping::_rotate(&strs(a, 0), s(a, 1))),
        "keeping._pick" => json!(keeping::_pick(&strs(a, 0), s(a, 1))),
        "keeping.respond" => json!(keeping::respond(s(a, 0), &mut mem(a, 1))),
        "the_wait_about_a_body.is_generic" => json!(the_wait_about_a_body::is_generic(s(a, 0))),
        "the_wait_about_a_body._words" => json!(the_wait_about_a_body::_words(s(a, 0))),
        "the_wait_about_a_body.said_back" => json!(the_wait_about_a_body::said_back(s(a, 0), s(a, 1))),
        "a_question_about_somebody._flat" => json!(a_question_about_somebody::_flat(s(a, 0))),
        "a_question_about_somebody.about_somebody_unnamed" => json!(a_question_about_somebody::about_somebody_unnamed(s(a, 0), &mem(a, 1))),
        "dreams._settled" => json!(dreams::_settled(s(a, 0), &strs(a, 1))),
        "trip_date_said_again._the_first_clause" => json!(trip_date_said_again::_the_first_clause(s(a, 0))),
        "a_day_somebody_else_keeps.respond" => json!(a_day_somebody_else_keeps::respond(s(a, 0), &mut mem(a, 1))),
        "a_day_somebody_else_keeps._choose" => json!(a_day_somebody_else_keeps::_choose(&strs(a, 0), &mem(a, 1), s(a, 2))),
        "safety_phrases._both_here" => {
            let groups: Vec<Value> = a.iter().map(Value::from_plain).collect();
            json!(safety_phrases::_both_here(&groups))
        }
        "the_kind_one_line_up._answered_by_the_fallback" => json!(the_kind_one_line_up::_answered_by_the_fallback(&mem(a, 0))),
        "first_line_open._has_a_turn_behind_it" => json!(first_line_open::_has_a_turn_behind_it(&mem(a, 0))),
        "doubt._the_question" => json!(doubt::_the_question(s(a, 0), s(a, 1))),
        "confusing._said_back" => json!(confusing::_said_back(s(a, 0))),
        "name_taken_back._note" => json!(name_taken_back::_note(s(a, 0))),
        "name_taken_back.respond" => json!(name_taken_back::respond(s(a, 0), &mut mem(a, 1))),
        "the_shape_of_the_answer._the_reply_is_only_a_placeholder" => json!(the_shape_of_the_answer::_the_reply_is_only_a_placeholder(s(a, 0))),
        "talkto._a_route_is_named" => json!(talkto::_a_route_is_named(s(a, 0))),
        "their_pastime._a_thing_nobody_is_into" => json!(their_pastime::_a_thing_nobody_is_into(s(a, 0))),
        "their_pastime._a_plural_thing" => json!(their_pastime::_a_plural_thing(s(a, 0))),
        "their_pastime._it_word" => json!(their_pastime::_it_word(s(a, 0))),
        "their_pastime._watched" => json!(their_pastime::_watched(s(a, 0))),
        "their_pastime._does_pool" => json!(their_pastime::_does_pool(s(a, 0))),
        "their_pastime._is_a_placeholder" => json!(their_pastime::_is_a_placeholder(s(a, 0))),
        "their_pastime._starts_a_sentence" => json!(their_pastime::_starts_a_sentence(s(a, 0))),
        "where_it_answers._content" => {
            let skip: std::collections::HashSet<String> = strs(a, 1).into_iter().collect();
            sorted(where_it_answers::_content(s(a, 0), if a.len() > 1 { Some(&skip) } else { None }))
        }
        "said_fact_back._nothing_to_give" => json!(said_fact_back::_nothing_to_give(s(a, 0))),
        other => panic!("no Rust call for {other}"),
    }
}

#[test]
fn the_ports_give_pybots_values() {
    crate::data::load();
    let cases: Vec<J> = serde_json::from_str(include_str!("mod4_cases.json")).unwrap();
    let mut wrong = Vec::new();
    let mut waiting = 0;
    for case in &cases {
        let name = case["fn"].as_str().unwrap();
        let args = case["args"].as_array().unwrap().clone();
        let got = std::panic::catch_unwind(|| call(name, &args));
        match got {
            Ok(got) if got == case["want"] => {}
            Ok(got) => wrong.push(format!("{name}{} -> {got}, PyBot: {}", case["args"], case["want"])),
            // a reader another workstream has not ported yet is not this port's difference
            Err(raised) => {
                let said = raised.downcast_ref::<String>().cloned().or_else(|| raised.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                if said.starts_with("UNPORTED") {
                    waiting += 1;
                } else {
                    wrong.push(format!("{name}{} raised {said}, PyBot: {}", case["args"], case["want"]));
                }
            }
        }
    }
    eprintln!("{} cases, {} waiting on readers not yet ported", cases.len(), waiting);
    assert!(wrong.is_empty(), "{} of {} differ:\n{}", wrong.len(), cases.len(), wrong.join("\n"));
}
