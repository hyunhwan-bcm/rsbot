//! Replays PyBot's own calls to the COM5 files (55 small readers of bot/compose) against the port.
//!
//!     com5_check CALLS.jsonl.gz [--fn NAME]... [--show K]
//!
//! CALLS is what `harness/com5_calls.py` recorded while PyBot ran whole conversations: every
//! distinct call with its arguments, its result, the global RNG state when the call used it, and
//! the memory after it when the call changed it. Each call is made again in Rust and the result
//! (and memory) compared. A call that reaches a function another workstream has not ported yet
//! is counted apart ("unported"), as is a Rust panic that stands for a Python exception.

use rsbot::compose::*;
use rsbot::data::Value;
use rsbot::runtime::Candidate;
use rsbot::state::Memory;
use serde_json::{Map, Value as J};
use std::collections::BTreeMap;
use std::io::BufRead;

struct A<'a> {
    args: &'a [J],
    kwargs: &'a Map<String, J>,
    params: Vec<String>,
}

impl<'a> A<'a> {
    fn get(&self, i: usize) -> Option<&'a J> {
        if let Some(v) = self.args.get(i) {
            return Some(v);
        }
        self.params.get(i).and_then(|name| self.kwargs.get(name))
    }
    fn s(&self, i: usize) -> String {
        match self.get(i) {
            Some(J::String(s)) => s.clone(),
            Some(J::Null) | None => String::new(),
            Some(other) => other.to_string(),
        }
    }
    fn opt_s(&self, i: usize) -> Option<String> {
        match self.get(i) {
            Some(J::String(s)) => Some(s.clone()),
            _ => None,
        }
    }
    fn b(&self, i: usize) -> bool {
        match self.get(i) {
            Some(J::Bool(b)) => *b,
            Some(J::Null) | None => false,
            Some(J::Number(n)) => n.as_f64() != Some(0.0),
            Some(J::String(s)) => !s.is_empty(),
            Some(J::Array(a)) => !a.is_empty(),
            Some(_) => true,
        }
    }
    fn n(&self, i: usize) -> i64 {
        self.get(i).and_then(|v| v.as_i64()).unwrap_or(0)
    }
    fn strs(&self, i: usize) -> Vec<String> {
        match self.get(i) {
            Some(J::Array(a)) => a.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect(),
            _ => Vec::new(),
        }
    }
    fn set(&self, i: usize) -> std::collections::HashSet<String> {
        self.strs(i).into_iter().collect()
    }
    fn mem(&self, i: usize) -> Memory {
        match self.get(i) {
            Some(J::Object(o)) => match o.get("__memory__") {
                Some(J::Object(m)) => Memory::from_plain(m),
                _ => Memory::from_plain(o),
            },
            _ => Memory::new(),
        }
    }
    fn value(&self, i: usize) -> rsbot::data::Value {
        fn bare(v: &J) -> J {
            match v {
                J::Object(o) if o.len() == 1 && o.contains_key("__dict__") => bare(&o["__dict__"]),
                J::Object(o) => J::Object(o.iter().map(|(k, v)| (k.clone(), bare(v))).collect()),
                J::Array(a) => J::Array(a.iter().map(bare).collect()),
                other => other.clone(),
            }
        }
        match self.get(i) {
            Some(v) => rsbot::data::Value::from_plain(&bare(v)),
            None => rsbot::data::Value::None,
        }
    }
    fn re(&self, i: usize) -> rsbot::py::re::Re {
        match self.get(i) {
            Some(J::Object(o)) if o.contains_key("__re__") => {
                rsbot::py::re::Re::new(o["__re__"].as_str().unwrap(), o["flags"].as_u64().unwrap_or(0) as u32)
            }
            Some(J::String(s)) => rsbot::py::re::Re::new(s, 0),
            _ => panic!("not a pattern"),
        }
    }
}

/// How a result is compared: as Python truthiness, as text (None and "" alike), or exactly.
enum Out {
    Bool(bool),
    Text(Option<String>),
    Exact(J),
    /// The lengths in order, and the same strings (Python sorts a set by length: ties fall in
    /// hash order, which changes from run to run).
    ByLength(Vec<String>),
    /// A set: the same strings in any order.
    Set(Vec<String>),
}

fn text(s: String) -> Out {
    Out::Text(Some(s))
}
fn opt(s: Option<String>) -> Out {
    Out::Text(s)
}
fn strs_out(v: Vec<String>) -> Out {
    Out::Exact(J::Array(v.into_iter().map(J::String).collect()))
}


/// A pool or None, compared exactly (an empty pool is not None).
fn pool(v: Option<Vec<String>>) -> Out {
    Out::Exact(v.map(|p| J::Array(p.into_iter().map(J::String).collect())).unwrap_or(J::Null))
}
/// A str or None, compared exactly ("" is not None).
fn opt_exact(v: Option<String>) -> Out {
    Out::Exact(v.map(J::String).unwrap_or(J::Null))
}
fn value(v: Value) -> Out {
    Out::Exact(v.to_plain())
}
fn set(v: impl IntoIterator<Item = String>) -> Out {
    Out::Set(v.into_iter().collect())
}
fn mem_out(memory: Memory, memory_after: &mut Option<Memory>) {
    *memory_after = Some(memory);
}

impl<'a> A<'a> {
    fn pattern_or_none(&self, i: usize) -> Value {
        match self.get(i) {
            Some(J::Object(o)) if o.contains_key("__re__") => Value::Re(std::sync::Arc::new(self.re(i))),
            _ => Value::None,
        }
    }
    fn candidates(&self, i: usize) -> Vec<Candidate> {
        let Some(J::Array(items)) = self.get(i) else { return Vec::new() };
        items
            .iter()
            .map(|c| {
                let c = c.get("__dict__").unwrap_or(c);
                Candidate {
                    module: Box::leak(c["module"].as_str().unwrap_or("").to_string().into_boxed_str()),
                    priority: c["priority"].as_f64().unwrap_or(0.0),
                    reply: c["reply"].as_str().unwrap_or("").to_string(),
                }
            })
            .collect()
    }
    fn present(&self, i: usize) -> bool {
        self.get(i).is_some_and(|v| !v.is_null())
    }
}

/// Runs one recorded call; None when this checker has no adapter for the function.
fn call(fid: &str, a: &A, memory_after: &mut Option<Memory>) -> Option<Out> {
    let (module, name) = fid.split_once(':').unwrap();
    let module = module.rsplit('.').next().unwrap();
    let b = Out::Bool;
    let out = match (module, name) {
        ("a_count_of_packets", "is_a_dose") => b(a_count_of_packets::is_a_dose(&a.s(0))),
        ("a_document_lives_with_an_office", "a_map_app_line") => b(a_document_lives_with_an_office::a_map_app_line(&a.s(0))),
        ("a_document_lives_with_an_office", "_the_kind") => text(a_document_lives_with_an_office::_the_kind(&a.s(0))),
        ("a_document_lives_with_an_office", "_names_the_office") => b(a_document_lives_with_an_office::_names_the_office(&a.s(0), &a.s(1))),
        ("a_document_lives_with_an_office", "a_lost_record") => b(a_document_lives_with_an_office::a_lost_record(&a.s(0), &a.s(1))),
        ("a_document_lives_with_an_office", "answer_for") => opt_exact(a_document_lives_with_an_office::answer_for(&a.s(0), &a.mem(1), &a.s(2))),
        ("a_first_time", "_a_task_they_faced") => b(a_first_time::_a_task_they_faced(&a.s(0), &a.mem(1))),
        ("a_first_time", "reply_for") => opt_exact(a_first_time::reply_for(&a.s(0), &a.mem(1))),
        ("a_thing_wants_doing", "reads") => opt_exact(a_thing_wants_doing::reads(&a.s(0), &a.mem(1))),
        ("a_thing_wants_doing", "pool") => pool(a_thing_wants_doing::pool(&a.s(0), &a.mem(1))),
        ("aboutness", "after_a_change_of_subject") => text(aboutness::after_a_change_of_subject(&a.s(0))),
        ("aboutness", "subject_words") => set(aboutness::subject_words(&a.s(0))),
        ("aboutness", "names_the_subject") => set(aboutness::names_the_subject(&a.s(0), &a.strs(1))),
        ("aboutness", "restates_the_line") => b(aboutness::restates_the_line(&a.s(0), &a.s(1))),
        ("aboutness", "is_a_hedge") => b(aboutness::is_a_hedge(&a.s(0))),
        ("aboutness", "unlike_the_line") => {
            let (about, said) = aboutness::unlike_the_line_words(&a.s(0), &a.s(1));
            let mut about = about;
            let mut said = said;
            about.sort();
            said.sort();
            Out::Set(vec![format!("about:{about:?}"), format!("said:{said:?}")])
        }
        ("aboutness", "the_better_answer") => {
            let need = if a.present(4) { a.value(4) } else { Value::Int(2) };
            value(aboutness::the_better_answer(&a.s(0), &a.candidates(1), &a.s(2), &a.value(3), &need))
        }
        ("breakup", "_thread_said") => text(breakup::_thread_said(&a.mem(0), &a.s(1))),
        ("breakup", "answer_for") => opt_exact(breakup::answer_for(&a.s(0), &a.s(1), &a.mem(2))),
        ("childwords", "_the_tail_after_the_child") => opt_exact(childwords::_the_tail_after_the_child(&a.s(0))),
        ("childwords", "_the_tail_is_about_the_death") => b(childwords::_the_tail_is_about_the_death(&a.s(0))),
        ("childwords", "a_death_in_the_thread") => b(childwords::a_death_in_the_thread(&a.mem(0), &a.value(1))),
        ("childwords", "asks_the_words_for_a_child") => b(childwords::asks_the_words_for_a_child(&a.s(0), &a.mem(1))),
        ("childwords", "_the_last_few_lines") => text(childwords::_the_last_few_lines(&a.mem(0), &a.value(1))),
        ("circumstance", "_line") => text(circumstance::_line(&a.s(0))),
        ("circumstance", "reads_as_situation") => opt_exact(circumstance::reads_as_situation(&a.s(0))),
        ("circumstance", "pool") => pool(circumstance::pool(&a.s(0))),
        ("doubt", "_last_bot") => {
            let (t, m) = doubt::_last_bot(&a.mem(0));
            Out::Exact(J::Array(vec![J::String(t), J::String(m)]))
        }
        ("doubt", "reads_as_a_doubt") => b(doubt::reads_as_a_doubt(&a.s(0))),
        ("doubt", "_the_line_behind") => text(doubt::_the_line_behind(&a.mem(0))),
        ("doubt", "reply_for") => opt_exact(doubt::reply_for(&a.s(0), &a.mem(1))),
        ("facts", "asks_for_a_when") => b(facts::asks_for_a_when(&a.s(0))),
        ("facts", "holds_a_date") => b(facts::holds_a_date(&a.s(0))),
        ("facts", "_date_words") => set(facts::_date_words(&a.s(0)).strs()),
        ("facts", "the_same_date") => b(facts::the_same_date(&a.s(0), &a.value(1))),
        ("facts", "the_fact_asked_for") => b(facts::the_fact_asked_for(&a.s(0), &a.s(1))),
        ("fallback_data", "the_place_the_answer_is_kept") => pool(fallback_data::the_place_the_answer_is_kept(&a.s(0))),
        ("fallback_data", "the_bots_own_answer") => pool(fallback_data::the_bots_own_answer(&a.s(0))),
        ("for_the_day", "for_the_ask") => pool(for_the_day::for_the_ask(&a.s(0), &a.s(1))),
        ("griefplea", "_said") => text(griefplea::_said(&a.s(0))),
        ("griefplea", "reply_for") => opt_exact(griefplea::reply_for(&a.s(0), &a.s(1), &a.mem(2))),
        ("handoff", "reads_as_handoff_about_a_thing") => opt_exact(handoff::reads_as_handoff_about_a_thing(&a.s(0))),
        ("handoff", "reads_as_handoff") => b(handoff::reads_as_handoff(&a.s(0))),
        ("handoff", "takes_as_it_comes") => b(handoff::takes_as_it_comes(&a.s(0), &a.mem(1))),
        ("handoff", "reply_for") => opt_exact(handoff::reply_for(&a.s(0), &a.mem(1))),
        ("hands", "_bare") => text(hands::_bare(&a.s(0))),
        ("hands", "reads") => opt_exact(hands::reads(&a.s(0))),
        ("hands", "writes_the_words") => b(hands::writes_the_words(&a.s(0))),
        ("hands", "reply_for") => opt_exact(hands::reply_for(&a.s(0), &a.mem(1))),
        ("hedge", "what_the_composer_would_say") => value(hedge::what_the_composer_would_say(&a.s(0), &a.mem(1))),
        ("hedge", "only_a_shrug") => b(hedge::only_a_shrug(&a.s(0), &a.mem(1), &a.pattern_or_none(2))),
        ("invitation", "_the_invitation") => pool(invitation::_the_invitation(&a.s(0))),
        ("invitation", "reply_for") => opt_exact(invitation::reply_for(&a.s(0), &a.mem(1))),
        ("judgement", "asks_for_a_judgement") => b(judgement::asks_for_a_judgement(&a.s(0), &a.mem(1)).unwrap()),
        ("kept_nearby", "_clean") => text(kept_nearby::_clean(&a.s(0))),
        ("kept_nearby", "_without_plurals") => text(kept_nearby::_without_plurals(&a.s(0))),
        ("kept_nearby", "is_the_plain_hedge") => b(kept_nearby::is_the_plain_hedge(&a.s(0))),
        ("kept_nearby", "_inheritable") => b(kept_nearby::_inheritable(&a.strs(0))),
        ("kept_nearby", "_the_kind") => pool(kept_nearby::_the_kind(&a.s(0), a.b(1))),
        ("kept_nearby", "answer_for") => pool(kept_nearby::answer_for(&a.s(0), &a.mem(1))),
        ("lead_in", "_only_the_floor_or_a_nod") => b(lead_in::_only_the_floor_or_a_nod(&a.s(0))),
        ("lead_in", "_the_rest_after_the_lead_in") => text(lead_in::_the_rest_after_the_lead_in(&a.s(0))),
        ("lead_in", "_an_echo_of_the_ask") => b(lead_in::_an_echo_of_the_ask(&a.s(0), &a.s(1))),
        ("lead_in", "_a_reply_of_its_own") => b(lead_in::_a_reply_of_its_own(&a.s(0))),
        ("lead_in", "the_answer_the_lead_in_hid") => opt_exact(lead_in::the_answer_the_lead_in_hid(&a.s(0), &a.mem(1))),
        ("liveask", "answer_for") => opt_exact(liveask::answer_for(&a.s(0), &a.mem(1), &a.s(2))),
        ("morning_after", "reads_as_a_morning_after") => b(morning_after::reads_as_a_morning_after(&a.s(0))),
        ("moving", "reads_as_a_move") => b(moving::reads_as_a_move(&a.s(0))),
        ("moving", "reads_as_a_move_cheered") => b(moving::reads_as_a_move_cheered(&a.s(0))),
        ("moving", "reply_for") => opt_exact(moving::reply_for(&a.s(0), &a.mem(1))),
        ("news_ask", "reads_as_an_ask_for_the_news") => b(news_ask::reads_as_an_ask_for_the_news(&a.s(0))),
        ("no_advice", "_what_is_left_after_the_decline") => text(no_advice::_what_is_left_after_the_decline(&a.s(0))),
        ("no_advice", "asks_something_of_its_own") => b(no_advice::asks_something_of_its_own(&a.s(0))),
        ("no_advice", "_plain") => text(no_advice::_plain(&a.s(0))),
        ("no_advice", "says_a_death") => b(no_advice::says_a_death(&a.s(0))),
        ("no_advice", "asks_for_a_view") => b(no_advice::asks_for_a_view(&a.s(0))),
        ("no_advice", "says_no_advice") => b(no_advice::says_no_advice(&a.s(0))),
        ("no_advice", "pool_name") => opt_exact(no_advice::pool_name(&a.s(0))),
        ("no_advice", "lines_for") => strs_out(no_advice::lines_for(&a.s(0))),
        ("no_advice", "reply_for") => text(no_advice::reply_for(&a.s(0), &a.s(1), &a.mem(2))),
        ("no_page", "_bare") => text(no_page::_bare(&a.s(0))),
        ("no_page", "reads") => opt_exact(no_page::reads(&a.s(0))),
        ("no_page", "reply_for") => opt_exact(no_page::reply_for(&a.s(0), &a.mem(1))),
        ("nottheirbest", "reads_as_a_low_day") => b(nottheirbest::reads_as_a_low_day(&a.s(0))),
        ("nottheirbest", "_may_replace") => b(nottheirbest::_may_replace(&a.s(0))),
        ("nottheirbest", "answer_for") => opt_exact(nottheirbest::answer_for(&a.s(0), &a.s(1), &a.mem(2))),
        ("onmybehalf", "_facing_them") => text(onmybehalf::_facing_them(&a.s(0))),
        ("onmybehalf", "_what_was_asked_for") => text(onmybehalf::_what_was_asked_for(&a.s(0))),
        ("onmybehalf", "_is_a_flat_read") => b(onmybehalf::_is_a_flat_read(&a.s(0))),
        ("onmybehalf", "_is_a_hedge") => b(onmybehalf::_is_a_hedge(&a.s(0))),
        ("onmybehalf", "_about_an_errand") => pool(onmybehalf::_about_an_errand(&a.s(0))),
        ("onmybehalf", "answer_for") => opt_exact(onmybehalf::answer_for(&a.s(0), &a.mem(1), &a.s(2))),
        ("outlook", "points_at_the_thread") => b(outlook::points_at_the_thread(&a.s(0))),
        ("outlook", "asks_the_outlook") => {
            let clean = if a.present(1) { a.s(1) } else { a.s(0) };
            b(outlook::asks_the_outlook(&a.s(0), &clean))
        }
        ("outlook", "the_pool") => value(outlook::the_pool(&a.value(0), &a.value(1), &a.value(2), &a.value(3))),
        ("own_ask", "_words") => strs_out(own_ask::_words(&a.s(0))),
        ("own_ask", "_named_thing") => opt_exact(own_ask::_named_thing(&a.s(0))),
        ("own_ask", "_family") => pool(own_ask::_family(&a.s(0))),
        ("own_ask", "answer") => opt_exact(own_ask::answer(&a.s(0), &a.mem(1))),
        ("place_facts", "_has") => b(place_facts::_has(&a.s(0), &a.strs(1))),
        ("place_facts", "_a_place_they_are_bringing_somebody_to") => b(place_facts::_a_place_they_are_bringing_somebody_to(&a.s(0))),
        ("place_facts", "the_line") => pool(place_facts::the_line(&a.s(0))),
        ("place_facts", "_a_day_ask") => pool(place_facts::_a_day_ask(&a.s(0))),
        ("pleased", "answer_for") => opt_exact(pleased::answer_for(&a.s(0), &a.mem(1))),
        ("postcode", "a_postcode_lookup") => pool(postcode::a_postcode_lookup(&a.s(0))),
        ("prices", "_a_balance_ask") => b(prices::_a_balance_ask(&a.s(0))),
        ("prices", "_has") => b(prices::_has(&a.s(0), &a.strs(1))),
        ("prices", "the_price_reply") => pool(prices::the_price_reply(&a.s(0), &a.strs(1), &a.s(2), &a.strs(3))),
        ("recall", "_is_a_death") => b(recall::_is_a_death(&a.s(0))),
        ("recall", "_said_on_the_last_turn") => b(recall::_said_on_the_last_turn(&a.s(0), &a.mem(1))),
        ("recall", "_names_a_thing") => b(recall::_names_a_thing(&a.s(0))),
        ("recall", "with_a_tail") => text(recall::with_a_tail(&a.s(0), &a.mem(1))),
        ("receiving_end", "_the_role_said_back") => text(receiving_end::_the_role_said_back(&a.s(0))),
        ("receiving_end", "_a_body_line_above") => b(receiving_end::_a_body_line_above(&a.mem(0))),
        ("receiving_end", "_a_person_who_keeps") => opt_exact(receiving_end::_a_person_who_keeps(&a.s(0), &a.s(1), &a.mem(2))),
        ("receiving_end", "reply_for") => opt_exact(receiving_end::reply_for(&a.s(0), &a.s(1), &a.mem(2))),
        ("repair", "deed_of") => opt_exact(repair::deed_of(&a.value(0))),
        ("repair", "owns_the_deed") => b(repair::owns_the_deed(&a.value(0))),
        ("repair", "with_the_feeling_named") => text(repair::with_the_feeling_named(&a.s(0), &a.s(1))),
        ("repair", "amends_to") => match a.get(0) {
            Some(J::Object(_)) => b(repair::amends_to(&a.value(0).get("clean").map(|v| v.str().to_string()).unwrap_or_default())),
            _ => b(repair::amends_to(&a.s(0))),
        },
        ("repair", "a_repair_is_open") => b(repair::a_repair_is_open(&a.mem(0))),
        ("rule", "_said") => text(rule::_said(&a.s(0))),
        ("rule", "_a_court_of_law") => b(rule::_a_court_of_law(&a.s(0))),
        ("rule", "_a_court_of_law_in_the_thread") => b(rule::_a_court_of_law_in_the_thread(&a.mem(0), if a.present(1) { a.n(1) } else { 2 })),
        ("rule", "_said_a_local_rule_recently") => b(rule::_said_a_local_rule_recently(&a.mem(0), if a.present(1) { a.n(1) } else { 2 })),
        ("rule", "_a_rule_ask_in_the_thread") => b(rule::_a_rule_ask_in_the_thread(&a.mem(0), &a.value(1))),
        ("rule", "pool_for") => pool(rule::pool_for(&a.s(0), &a.mem(1))),
        ("rule", "answer_for") => opt_exact(rule::answer_for(&a.s(0), &a.mem(1))),
        ("safety_doubt", "_a_bare_verdict_ask") => b(safety_doubt::_a_bare_verdict_ask(&a.s(0), &a.mem(1), &a.s(2))),
        ("safety_doubt", "reply_for") => opt_exact(safety_doubt::reply_for(&a.s(0), &a.mem(1))),
        ("safety_doubt", "_the_answer_behind") => {
            let (behind, after) = safety_doubt::_the_answer_behind(&a.mem(0));
            Out::Exact(J::Array(vec![behind.map(J::String).unwrap_or(J::Null), J::Array(after.into_iter().map(J::String).collect())]))
        }
        ("safetyask", "reply_for") => pool(safetyask::reply_for(&a.s(0), &a.mem(1))),
        ("safetyask", "_a_safety_word") => b(safetyask::_a_safety_word(&a.s(0))),
        ("safetyask", "_the_thing_is_described_faulty") => b(safetyask::_the_thing_is_described_faulty(&a.mem(0), &a.s(1))),
        ("safetyask", "_a_shrug") => b(safetyask::_a_shrug(&a.s(0))),
        ("safetyask", "_the_thing_on_the_table") => text(safetyask::_the_thing_on_the_table(&a.s(0), &a.mem(1))),
        ("safetyask", "the_bot_already_ruled") => b(safetyask::the_bot_already_ruled(&a.mem(0))),
        ("safetyask", "_the_answers_have_this") => opt_exact(safetyask::_the_answers_have_this(&a.s(0), &a.mem(1))),
        ("setting_off", "_the_hour_already_said") => text(setting_off::_the_hour_already_said(&a.s(0), &a.mem(1))),
        ("setting_off", "is_a_pointer") => b(setting_off::is_a_pointer(&a.s(0))),
        ("setting_off", "reads_it") => b(setting_off::reads_it(&a.s(0))),
        ("setting_off", "_choose") => text(setting_off::_choose(&a.strs(0), &a.mem(1))),
        ("setting_off", "answer_for") => opt_exact(setting_off::answer_for(&a.s(0), &a.mem(1))),
        ("stepwords", "_ing") => text(stepwords::_ing(&a.s(0))),
        ("stepwords", "a_step") => b(stepwords::a_step(&a.s(0))),
        ("stepwords", "said_after_a_to") => text(stepwords::said_after_a_to(&a.s(0), &Value::None)),
        ("talktime", "answer_for") => opt_exact(talktime::answer_for(&a.s(0), &a.mem(1))),
        ("talkto", "_as_typed") => text(talkto::_as_typed(&a.s(0))),
        ("talkto", "the_line_as_read") => text(talkto::the_line_as_read(&a.s(0))),
        ("talkto", "_subject") => text(talkto::_subject(&a.s(0))),
        ("talkto", "answer_for") => text(talkto::answer_for(&a.s(0), &a.mem(1))),
        ("the_rule_for_it", "the_rule_for_it") => pool(the_rule_for_it::the_rule_for_it(&a.s(0), &a.s(1))),
        ("the_trouble_named", "_has") => b(the_trouble_named::_has(&a.s(0), &a.strs(1))),
        ("the_trouble_named", "_family") => opt_exact(the_trouble_named::_family(&a.s(0))),
        ("the_trouble_named", "_the_wording") => text(the_trouble_named::_the_wording(&a.mem(0), &a.s(1))),
        ("the_trouble_named", "reply_for") => {
            let mut memory = a.mem(1);
            let out = the_trouble_named::reply_for_mut(&a.s(0), &mut memory);
            mem_out(memory, memory_after);
            opt_exact(out)
        }
        ("the_turn_after", "_the_limit_just_given") => {
            let (just, phrases) = the_turn_after::_the_limit_just_given(&a.mem(0));
            Out::Exact(J::Array(vec![
                just.map(|p| J::Array(p.into_iter().map(J::String).collect())).unwrap_or(J::Null),
                J::Array(phrases.into_iter().map(J::String).collect()),
            ]))
        }
        ("the_turn_after", "_the_hours_pool") => strs_out(the_turn_after::_the_hours_pool()),
        ("the_turn_after", "_words_of") => set(the_turn_after::_words_of(&a.strs(0))),
        ("the_turn_after", "the_day_after") => opt_exact(the_turn_after::the_day_after(&a.s(0), &a.mem(1))),
        ("the_turn_after", "asked_to_check") => opt_exact(the_turn_after::asked_to_check(&a.s(0), &a.mem(1))),
        ("the_turn_after", "answer_for") => opt(the_turn_after::answer_for(&a.s(0), &a.mem(1))),
        ("their_life", "_bare") => text(their_life::_bare(&a.s(0))),
        ("their_life", "_somebody_of_theirs") => b(their_life::_somebody_of_theirs(&a.mem(0))),
        ("their_life", "reads") => opt_exact(their_life::reads(&a.s(0), &a.mem(1))),
        ("their_life", "reply_for") => opt_exact(their_life::reply_for(&a.s(0), &a.mem(1))),
        ("theirperson", "_said") => text(theirperson::_said(&a.s(0))),
        ("theirperson", "reply_for") => opt_exact(theirperson::reply_for(&a.s(0), &a.mem(1))),
        ("theirperson", "may_replace") => b(theirperson::may_replace(&a.s(0), &a.s(1))),
        ("toknow", "_the_thing_asked_about") => text(toknow::_the_thing_asked_about(&a.s(0))),
        ("toknow", "_a_card_about") => text(toknow::_a_card_about(&a.s(0))),
        ("toknow", "ask_about_what_to_look_out_for") => opt_exact(toknow::ask_about_what_to_look_out_for(&a.s(0), &a.mem(1))),
        ("unseen", "reads") => b(unseen::reads(&a.s(0))),
        ("waited_for", "asks_for_a_date") => b(waited_for::asks_for_a_date(&a.s(0), &a.s(1))),
        ("waited_for", "_the_thread") => text(waited_for::_the_thread(&a.s(0), &a.mem(1))),
        ("waited_for", "_the_kind") => opt_exact(waited_for::_the_kind(&a.s(0), &a.s(1))),
        ("waited_for", "for_the_ask") => pool(waited_for::for_the_ask(&a.s(0), &a.s(1), &a.mem(2))),
        ("ward", "_whom") => text(ward::_whom(&a.s(0))),
        ("ward", "_the_place_before") => text(ward::_the_place_before(&a.mem(0))),
        ("ward", "answer_for") => opt_exact(ward::answer_for(&a.s(0), &a.mem(1))),
        ("whole", "a_whole_cut") => text(whole::a_whole_cut(&a.s(0), a.n(1))),
        ("worn", "pool") => value(worn::pool(&a.value(0))),
        ("worn", "reads_as_worn") => opt_exact(worn::reads_as_worn(&a.s(0), &a.strs(1), &a.s(2))),
        ("worn", "_shape") => {
            let words = a.strs(1);
            opt_exact(worn::_shape(&a.s(0), if words.is_empty() { None } else { Some(&words) }))
        }
        ("worn", "_content_words") => strs_out(worn::_content_words(&a.s(0))),
        ("worn", "_is_a_role") => b(worn::_is_a_role(&a.s(0))),
        ("worn", "_names_a_job") => b(worn::_names_a_job(&a.s(0))),
        _ => return None,
    };
    Some(out)
}

fn sorted(set: std::collections::HashSet<String>) -> Vec<String> {
    let mut v: Vec<String> = set.into_iter().collect();
    v.sort();
    v
}

fn truthy(v: &J) -> bool {
    match v {
        J::Null => false,
        J::Bool(b) => *b,
        J::Number(n) => n.as_f64() != Some(0.0),
        J::String(s) => !s.is_empty(),
        J::Array(a) => !a.is_empty(),
        J::Object(o) => !o.is_empty(),
    }
}

fn same(py: &J, out: &Out) -> bool {
    match out {
        Out::Bool(b) => truthy(py) == *b,
        Out::Text(t) => {
            let p = match py {
                J::String(s) if !s.is_empty() => Some(s.as_str()),
                _ => None,
            };
            p == t.as_deref().filter(|s| !s.is_empty())
        }
        Out::Exact(j) => py == j,
        Out::Set(v) => {
            let mut theirs: Vec<String> = match py {
                // unlike_the_line's (set, set): compared as the two sorted lists
                J::Array(a) if a.len() == 2 && a.iter().all(|x| x.is_array()) && v.len() == 2 && v[0].starts_with("about:") => {
                    let sorted = |x: &J| {
                        let mut s: Vec<String> = x.as_array().unwrap().iter().map(|s| s.as_str().unwrap_or("").to_string()).collect();
                        s.sort();
                        s
                    };
                    vec![format!("about:{:?}", sorted(&a[0])), format!("said:{:?}", sorted(&a[1]))]
                }
                J::Array(a) => a.iter().map(|s| s.as_str().unwrap_or("").to_string()).collect(),
                _ => Vec::new(),
            };
            let mut mine = v.clone();
            theirs.sort();
            mine.sort();
            theirs == mine
        }
        Out::ByLength(v) => {
            let theirs: Vec<String> =
                py.as_array().map(|a| a.iter().map(|s| s.as_str().unwrap_or("").to_string()).collect()).unwrap_or_default();
            let lens = |v: &[String]| v.iter().map(|s| s.chars().count()).collect::<Vec<_>>();
            let (mut a, mut b) = (theirs.clone(), v.clone());
            a.sort();
            b.sort();
            lens(&theirs) == lens(v) && a == b
        }
    }
}

fn show(out: &Out) -> String {
    match out {
        Out::Bool(b) => b.to_string(),
        Out::Text(t) => format!("{t:?}"),
        Out::Exact(j) => j.to_string(),
        Out::ByLength(v) => format!("{v:?}"),
        Out::Set(v) => format!("{v:?}"),
    }
}

fn main() {
    rsbot::runtime::quiet_panics();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut files = Vec::new();
    let mut only: Vec<String> = Vec::new();
    let mut show_k = 3usize;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--fn" => {
                only.push(args[i + 1].clone());
                i += 1;
            }
            "--show" => {
                show_k = args[i + 1].parse().unwrap();
                i += 1;
            }
            other => files.push(other.to_string()),
        }
        i += 1;
    }
    // (ok, mismatch, unported, panic, no adapter)
    let mut stats: BTreeMap<String, [usize; 5]> = BTreeMap::new();
    let mut shown: BTreeMap<String, usize> = BTreeMap::new();
    let mut missing: BTreeMap<String, usize> = BTreeMap::new();
    for file in &files {
        let f = std::fs::File::open(file).expect("calls file");
        let reader = std::io::BufReader::new(flate2::read::GzDecoder::new(f));
        for line in reader.lines() {
            let line = line.unwrap();
            let rec: J = serde_json::from_str(&line).unwrap();
            let fid = rec["fn"].as_str().unwrap().to_string();
            if !only.is_empty() && !only.iter().any(|o| fid.ends_with(o.as_str())) {
                continue;
            }
            let empty = Map::new();
            let a = A {
                args: rec["args"].as_array().map(|v| v.as_slice()).unwrap_or(&[]),
                kwargs: rec["kwargs"].as_object().unwrap_or(&empty),
                params: rec["params"].as_array().map(|v| v.iter().map(|p| p.as_str().unwrap_or("").to_string()).collect()).unwrap_or_default(),
            };
            if let Some(J::Array(state)) = rec.get("rng") {
                let st: Vec<u64> = state.iter().map(|v| v.as_u64().unwrap()).collect();
                rsbot::py::random::setstate(rsbot::py::random::Random::from_python_state(&st));
            } else {
                rsbot::py::random::seed(0);
            }
            let entry = stats.entry(fid.clone()).or_default();
            let mut memory_after: Option<Memory> = None;
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| call(&fid, &a, &mut memory_after)));
            let raised = rec["raised"].as_str();
            match res {
                Ok(None) => entry[4] += 1,
                Ok(Some(out)) => {
                    let mut ok = raised.is_none() && same(&rec["result"], &out);
                    let mut why = String::new();
                    if ok {
                        if let (Some(m), Some(J::Object(want))) = (&memory_after, rec.get("mem_after")) {
                            let got = m.to_plain();
                            let differs: Vec<&String> = want
                                .iter()
                                .filter(|(k, v)| k.as_str() != "understood" && got.get(k.as_str()) != Some(v))
                                .map(|(k, _)| k)
                                .collect();
                            if !differs.is_empty() {
                                ok = false;
                                why = format!(" memory differs at {differs:?}");
                            }
                        }
                    }
                    if ok {
                        entry[0] += 1;
                    } else {
                        entry[1] += 1;
                        let n = shown.entry(fid.clone()).or_default();
                        if *n < show_k {
                            *n += 1;
                            let a_text = serde_json::to_string(&rec["args"]).unwrap();
                            let a_text: String = a_text.chars().take(400).collect();
                            println!(
                                "MISMATCH {fid}{why}\n  args: {a_text}\n  py:   {} {}\n  rs:   {}",
                                rec["result"],
                                raised.unwrap_or(""),
                                show(&out)
                            );
                        }
                    }
                }
                Err(_) => {
                    let msg = rsbot::runtime::take_last_panic().unwrap_or_default();
                    if msg.starts_with("UNPORTED") {
                        entry[2] += 1;
                        *missing.entry(msg.clone()).or_default() += 1;
                    } else if raised.is_some() {
                        entry[0] += 1; // Python raised, and so did the port
                    } else {
                        entry[3] += 1;
                        let n = shown.entry(fid.clone()).or_default();
                        if *n < show_k {
                            *n += 1;
                            let a_text: String = serde_json::to_string(&rec["args"]).unwrap().chars().take(300).collect();
                            println!("PANIC {fid}: {msg}\n  args: {a_text}");
                        }
                    }
                }
            }
        }
    }
    let mut total = [0usize; 5];
    println!("\n{:<55} {:>6} {:>6} {:>8} {:>6} {:>6}", "function", "ok", "diff", "unported", "panic", "n/a");
    for (fid, s) in &stats {
        println!("{:<55} {:>6} {:>6} {:>8} {:>6} {:>6}", fid, s[0], s[1], s[2], s[3], s[4]);
        for k in 0..5 {
            total[k] += s[k];
        }
    }
    println!("{:<55} {:>6} {:>6} {:>8} {:>6} {:>6}", "TOTAL", total[0], total[1], total[2], total[3], total[4]);
    let mut missing: Vec<(String, usize)> = missing.into_iter().collect();
    missing.sort_by(|a, b| b.1.cmp(&a.1));
    println!("\nunported functions reached (other workstreams'):");
    for (name, n) in missing.iter().take(25) {
        println!("  {n:6} {name}");
    }
}
