//! Replays PyBot's own calls to the MEM4 files (narrow, callback, repeat, ack) against the port.
//!
//!     mem4_check CALLS.jsonl.gz [--fn NAME]... [--show K]
//!
//! CALLS is what `harness/mem4_calls.py` recorded while PyBot ran whole conversations: every
//! distinct call with its arguments, its result, the global RNG state when the call used it, and
//! the memory after it when the call changed it. Each call is made again in Rust and the result
//! (and memory) compared. A call that reaches a function another workstream has not ported yet
//! is counted apart ("unported"), as is a Rust panic that stands for a Python exception.

use rsbot::memory::{ack, callback, narrow, repeat};
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

/// Runs one recorded call; None when this checker has no adapter for the function.
fn call(fid: &str, a: &A, memory_after: &mut Option<Memory>) -> Option<Out> {
    let (module, name) = fid.split_once(':').unwrap();
    let module = module.rsplit('.').next().unwrap();
    let out = match (module, name) {
        // ---- narrow
        ("narrow", "only_a_length") => Out::Bool(narrow::only_a_length(&a.s(0))),
        ("narrow", "_the_remedy_in") => opt(narrow::_the_remedy_in(&a.s(0))),
        ("narrow", "carries") => Out::Bool(narrow::carries(&a.s(0), &a.s(1))),
        ("narrow", "has_a_date") => Out::Bool(narrow::has_a_date(&a.s(0))),
        ("narrow", "pieces_of") => strs_out(narrow::pieces_of(&a.s(0), &a.re(1))),
        ("narrow", "step_word") => opt(narrow::step_word(&a.s(0))),
        ("narrow", "kind_of") => opt(narrow::kind_of(&a.s(0))),
        ("narrow", "asks_a_part") => Out::Bool(narrow::asks_a_part(&a.s(0))),
        ("narrow", "_named_follow_up") => Out::Bool(narrow::_named_follow_up(&a.s(0))),
        ("narrow", "_gives_a_figure_in") => Out::Bool(narrow::_gives_a_figure_in(&a.s(0), &a.s(1))),
        ("narrow", "_the_clause_in_the_unit") => opt(narrow::_the_clause_in_the_unit(&a.s(0), &a.s(1))),
        ("narrow", "_the_sentence_in_the_unit") => opt(narrow::_the_sentence_in_the_unit(&a.s(0), &a.s(1))),
        ("narrow", "part_in_the_unit") => opt(narrow::part_in_the_unit_text(&a.s(0), &a.s(1))),
        ("narrow", "asked_unit") => opt(narrow::asked_unit(&a.s(0))),
        ("narrow", "_states_a_length") => Out::Bool(narrow::_states_a_length(&a.s(0))),
        ("narrow", "the_outcome_asked") => text(narrow::the_outcome_asked(&a.s(0))),
        ("narrow", "the_part_asked") => opt(narrow::the_part_asked(&a.s(0), &a.s(1), &a.s(2))),
        ("narrow", "_a_rate_cut_from_its_figure") => Out::Bool(narrow::_a_rate_cut_from_its_figure(&a.s(0), &a.s(1), &a.s(2))),
        ("narrow", "narrow") => opt(narrow::narrow(&a.s(0), &a.s(1), &a.s(2))),
        ("narrow", "_narrow") => opt(narrow::_narrow(&a.s(0), &a.s(1))),
        ("narrow", "_a_length_of_a_doing") => Out::Bool(narrow::_a_length_of_a_doing(&a.s(0), &a.s(1), &a.s(2))),
        ("narrow", "_a_cut_off_not_a_length") => Out::Bool(narrow::_a_cut_off_not_a_length(&a.s(0), &a.s(1))),
        ("narrow", "the_doings_length") => text(narrow::the_doings_length(&a.s(0), &a.s(1))),
        ("narrow", "_at_another_place") => Out::Bool(narrow::_at_another_place(&a.s(0), &a.s(1), &a.s(2))),
        ("narrow", "_gives_another_places_figure") => {
            Out::Bool(narrow::_gives_another_places_figure(&a.s(0), a.opt_s(1).as_deref(), &a.s(2)))
        }
        ("narrow", "_the_time_it_does_keep") => opt(narrow::_the_time_it_does_keep(&a.strs(0), &a.s(1))),
        ("narrow", "_says_a_time") => Out::Bool(narrow::_says_a_time(&a.s(0))),
        ("narrow", "_only_a_cut_off") => Out::Bool(narrow::_only_a_cut_off(&a.s(0))),
        ("narrow", "_the_answer_said_again") => opt(narrow::_the_answer_said_again(&a.strs(0), &a.s(1))),
        ("narrow", "_the_sentence_a_figure_hangs_on") => {
            opt(narrow::_the_sentence_a_figure_hangs_on(&a.s(0), &a.s(1), &a.s(2), a.b(3)))
        }
        ("narrow", "_unit_stem") => text(narrow::_unit_stem(&a.s(0))),
        ("narrow", "_same_word") => Out::Bool(narrow::_same_word(&a.s(0), &a.s(1))),
        ("narrow", "_names_one_of") => Out::Bool(narrow::_names_one_of(&a.set(0), &a.strs(1))),
        ("narrow", "_only_a_count") => Out::Bool(narrow::_only_a_count(&a.s(0))),
        ("narrow", "_how_often") => opt(narrow::_how_often(&a.strs(0), &a.s(1), &a.s(2))),
        ("narrow", "_counts_on_the_answer") => Out::Bool(narrow::_counts_on_the_answer(&a.s(0))),
        ("narrow", "_freq_clause") => opt(narrow::_freq_clause(&a.s(0), a.b(1), &a.set(2))),
        ("narrow", "_dated") => opt(narrow::_dated(&a.strs(0), &a.s(1), &a.s(2))),
        ("narrow", "_the_part_they_named") => opt(narrow::_the_part_they_named(&a.strs(0), &a.s(1), &a.s(2))),
        ("narrow", "_not_a_figure") => opt(narrow::_not_a_figure(&a.strs(0), &a.s(1), &a.s(2), &a.s(3))),
        ("narrow", "_named_clause") => opt(narrow::_named_clause(&a.s(0), &a.s(1))),
        ("narrow", "_root") => text(narrow::_root(&a.s(0))),
        ("narrow", "_says") => Out::Bool(narrow::_says(&a.s(0), &a.s(1))),
        ("narrow", "_informative") => Out::Bool(narrow::_informative(&a.s(0))),
        ("narrow", "_cuts_a_list") => Out::Bool(narrow::_cuts_a_list(&a.s(0), &a.s(1))),
        ("narrow", "_with_unit") => text(narrow::_with_unit(&a.s(0), &a.s(1))),
        ("narrow", "_property_ask") => Out::Bool(narrow::_property_ask(&a.s(0))),
        ("narrow", "_option_ask") => Out::Bool(narrow::_option_ask(&a.s(0))),
        ("narrow", "_stands_alone") => Out::Bool(narrow::_stands_alone(&a.s(0))),
        ("narrow", "_by_option") => opt(narrow::_by_option(&a.strs(0), &a.s(1))),
        ("narrow", "_only_a_qualifier") => Out::Bool(narrow::_only_a_qualifier(&a.s(0))),
        ("narrow", "_by_step") => opt(narrow::_by_step(&a.strs(0), &a.s(1))),
        ("narrow", "_with_the_script") => opt(narrow::_with_the_script(&a.s(0), &a.strs(1), &a.s(2))),
        ("narrow", "_the_quotation") => opt(narrow::_the_quotation(&a.s(0))),
        ("narrow", "_to_the_end_of_the_script") => text(narrow::_to_the_end_of_the_script(&a.strs(0), a.n(1) as usize)),
        ("narrow", "_by_words") => opt(narrow::_by_words(&a.strs(0), &a.s(1))),
        ("narrow", "_as_a_length") => text(narrow::_as_a_length(&a.s(0))),
        ("narrow", "_names_a_thing") => Out::Bool(narrow::_names_a_thing(&a.s(0))),
        ("narrow", "_measure") => text(narrow::_measure(&a.s(0))),
        ("narrow", "_a_figure_each") => match narrow::_a_figure_each(&a.s(0), &a.s(1)) {
            Some(v) => strs_out(v),
            None => Out::Exact(J::Null),
        },
        ("narrow", "_the_item_they_named") => opt(narrow::_the_item_they_named(&a.s(0), &a.s(1), &a.s(2))),
        ("narrow", "_the_half_they_named") => opt(narrow::_the_half_they_named(&a.strs(0), &a.s(1))),
        ("narrow", "_part") => match narrow::_part(&a.s(0), &a.s(1), &a.s(2)) {
            narrow::Part::Text(t) => Out::Exact(J::String(t)),
            narrow::Part::Nothing => Out::Exact(J::Null),
            narrow::Part::NoLength => Out::Exact(J::String("NO_LENGTH".into())),
        },
        ("narrow", "_canonical") => text(narrow::_canonical(&a.s(0))),
        ("narrow", "_same_step") => Out::Bool(narrow::_same_step(&a.set(0), &a.set(1))),
        ("narrow", "_aside_back") => text(narrow::_aside_back(&a.s(0), &a.strs(1))),
        ("narrow", "clauses") => strs_out(narrow::clauses(&a.s(0), &a.re(1))),
        ("narrow", "clauses_bare") => {
            let (p, b) = narrow::clauses_bare(&a.s(0), &a.re(1));
            Out::Exact(J::Array(vec![
                J::Array(p.into_iter().map(J::String).collect()),
                J::Array(b.into_iter().map(J::String).collect()),
            ]))
        }
        ("narrow", "_aside_held") => {
            let (h, k) = narrow::_aside_held(&a.s(0));
            Out::Exact(J::Array(vec![J::String(h), J::Array(k.into_iter().map(J::String).collect())]))
        }
        ("narrow", "_clause") => opt(narrow::_clause(&a.s(0), &a.s(1))),
        ("narrow", "_clause_with") => opt(narrow::_clause_with(&a.s(0), &a.s(1))),
        ("narrow", "_clause_holding") => opt(narrow::_clause_holding(&a.s(0), &a.s(1))),
        ("narrow", "words_of") => strs_out(narrow::words_of(&a.s(0))),
        ("narrow", "flat") => text(narrow::flat(&a.s(0))),
        ("narrow", "tidy") => opt(narrow::tidy(&a.s(0))),
        // ---- callback
        ("callback", "reply_for") => {
            let mut m = a.mem(1);
            let r = callback::reply_for(&a.s(0), &mut m);
            *memory_after = Some(m);
            text(r)
        }
        ("callback", "claims") => {
            let mut m = a.mem(1);
            let r = callback::claims(&a.s(0), &mut m);
            *memory_after = Some(m);
            Out::Bool(r)
        }
        ("callback", "_a_medical_episode") => Out::Bool(callback::_a_medical_episode(&a.mem(0))),
        ("callback", "_the_pronouns") => strs_out(callback::_the_pronouns(&a.s(0))),
        ("callback", "_the_news_about") => text(callback::_the_news_about(&a.mem(0), &a.s(1), &a.s(2))),
        // ---- repeat
        ("repeat", "asks_for_the_same_work") => Out::Bool(repeat::asks_for_the_same_work(&a.s(0), &a.mem(1))),
        ("repeat", "without_a_reset") => text(repeat::without_a_reset(&a.s(0))),
        ("repeat", "only_said_it_back") => Out::Bool(repeat::only_said_it_back(&a.s(0), a.opt_s(1).as_deref())),
        ("repeat", "_a_help_ask") => Out::Bool(repeat::_a_help_ask(&a.s(0))),
        ("repeat", "asks_for_the_move") => Out::Bool(repeat::asks_for_the_move(&a.s(0))),
        ("repeat", "names_nothing") => Out::Bool(repeat::names_nothing(&a.s(0))),
        ("repeat", "_names_a_thing_of_its_own") => strs_out(repeat::_names_a_thing_of_its_own(&a.s(0))),
        ("repeat", "asks_to_put_it_right") => Out::Bool(repeat::asks_to_put_it_right(&a.s(0))),
        ("repeat", "a_step_of_the_same_answer") => Out::Bool(repeat::a_step_of_the_same_answer(&a.s(0))),
        ("repeat", "_recent_answers") => Out::Exact(J::Array(
            repeat::recent_answers(&a.mem(0))
                .into_iter()
                .map(|(b, q, s)| J::Array(vec![J::from(b as i64), J::String(q), J::String(s)]))
                .collect(),
        )),
        ("repeat", "_nothing_but_acknowledgements") => {
            Out::Bool(repeat::_nothing_but_acknowledgements(a.n(0) as usize, &a.mem(1)))
        }
        ("repeat", "_about_the_same_thing") => Out::Bool(repeat::_about_the_same_thing(&a.s(0), &a.s(1), &a.mem(2))),
        ("repeat", "_short") => text(repeat::_short(&a.s(0))),
        ("repeat", "_explains") => Out::Bool(repeat::_explains(&a.s(0))),
        ("repeat", "_a_step_comes_after") => Out::Bool(repeat::_a_step_comes_after(&a.s(0))),
        ("repeat", "_opens_an_instruction") => Out::Bool(repeat::_opens_an_instruction(&a.s(0))),
        ("repeat", "_opening_move") => opt(repeat::_opening_move(&a.s(0))),
        ("repeat", "_a_question") => Out::Bool(repeat::_a_question(&a.s(0))),
        ("repeat", "_a_move") => opt(repeat::a_move(&a.s(0), a.b(1))),
        ("repeat", "_a_move_in") => opt(repeat::_a_move_in(&a.s(0), a.b(1))),
        ("repeat", "_a_conditional_move") => opt(repeat::_a_conditional_move(&a.s(0))),
        ("repeat", "asks_for_more") => Out::Bool(repeat::asks_for_more(&a.s(0))),
        ("repeat", "a_hand_back") => Out::Bool(repeat::a_hand_back(&a.s(0), &a.mem(1))),
        ("repeat", "_handed_back_last") => Out::Bool(repeat::_handed_back_last(&a.mem(0))),
        ("repeat", "_another_part") => {
            let (p, d) = repeat::_another_part(&a.s(0), &a.mem(1));
            Out::Exact(J::Array(vec![J::String(p), J::Bool(d)]))
        }
        ("repeat", "_said_in_a_turn") => Out::Bool(repeat::_said_in_a_turn(&a.s(0), &a.mem(1))),
        ("repeat", "_says_what_to_do") => Out::Bool(repeat::_says_what_to_do(&a.s(0)).truthy()),
        ("repeat", "_said_in_the_chat") => Out::Bool(repeat::_said_in_the_chat(&a.s(0), &a.mem(1))),
        ("repeat", "_claims_what_it_never_said") => text(repeat::_claims_what_it_never_said(&a.s(0), &a.mem(1))),
        ("repeat", "_handed_back") | ("repeat", "handed_back") => {
            let mut m = a.mem(0);
            let r = repeat::_handed_back(&mut m, &a.s(1));
            *memory_after = Some(m);
            text(r)
        }
        ("repeat", "framed") => Out::Bool(repeat::framed(&a.s(0))),
        ("repeat", "_already_repeated") => Out::Bool(repeat::_already_repeated(&a.s(0), &a.mem(1))),
        ("repeat", "_handed_back_already") => Out::Bool(repeat::_handed_back_already(&a.s(0), &a.mem(1))),
        ("repeat", "_a_lead_of_its_own") => Out::Bool(repeat::_a_lead_of_its_own(&a.s(0))),
        ("repeat", "_kept_clause") => text(repeat::_kept_clause(&a.s(0))),
        ("repeat", "asks_for_less") => Out::Bool(repeat::asks_for_less(&a.s(0))),
        ("repeat", "_sentences") => strs_out(repeat::_sentences(&a.s(0))),
        ("repeat", "_most_condensed") => text(repeat::_most_condensed(&a.s(0), Some(&a.mem(1)))),
        ("repeat", "_short_enough") => text(repeat::_short_enough(&a.s(0))),
        ("repeat", "_ended") => text(repeat::_ended(&a.s(0))),
        ("repeat", "_unbalanced") => Out::Bool(repeat::_unbalanced(&a.s(0))),
        ("repeat", "_the_quotation") => text(repeat::_the_quotation(&a.s(0))),
        ("repeat", "_closing_the_quote") => text(repeat::_closing_the_quote(&a.s(0), &a.s(1))),
        ("repeat", "_read_up_to_the_colon") => text(repeat::_read_up_to_the_colon(&a.s(0))),
        ("repeat", "_keeps") => text(repeat::_keeps(&a.s(0), &a.s(1))),
        ("repeat", "shorter_for") => {
            let mut m = a.mem(1);
            let r = repeat::shorter_for_text(&a.s(0), &mut m);
            *memory_after = Some(m);
            opt(r)
        }
        ("repeat", "_a_bare_time") => opt(repeat::_a_bare_time(a.opt_s(0).as_deref())),
        ("repeat", "_names_no_thing_of_its_own") => Out::Bool(repeat::_names_no_thing_of_its_own(&a.s(0))),
        ("repeat", "_the_price_the_answer_gave") => {
            let mut m = a.mem(1);
            let r = repeat::_the_price_the_answer_gave(&a.s(0), &mut m);
            *memory_after = Some(m);
            opt(r)
        }
        ("repeat", "asks_what_that_costs") => Out::Bool(repeat::asks_what_that_costs(&a.s(0), &a.mem(1))),
        ("repeat", "_the_price_in_hand") => opt(repeat::_the_price_in_hand(&a.s(0), &a.mem(1))),
        ("repeat", "_the_time_an_earlier_answer_gave") => {
            let mut m = a.mem(1);
            let r = repeat::_the_time_an_earlier_answer_gave(&a.s(0), &mut m);
            *memory_after = Some(m);
            opt(r)
        }
        ("repeat", "_a_length_already_handed") => Out::Bool(repeat::_a_length_already_handed(&a.s(0), &a.mem(1))),
        ("repeat", "_a_what_to_do_answer") => Out::Bool(repeat::_a_what_to_do_answer(&a.mem(0))),
        ("repeat", "answer_for") => {
            let mut m = a.mem(1);
            let r = repeat::answer_for_text(&a.s(0), &mut m);
            *memory_after = Some(m);
            opt(r)
        }
        ("repeat", "_words") => strs_out(repeat::_words(&a.s(0))),
        ("repeat", "_a_bare_ask") => Out::Bool(repeat::_a_bare_ask(&a.s(0))),
        ("repeat", "_a_lead_and_a_step") => text(repeat::_a_lead_and_a_step(&a.s(0), &a.s(1))),
        ("repeat", "_the_lead_for") => strs_out(repeat::_the_lead_for(&a.s(0), &a.s(1))),
        ("repeat", "_trouble") => Out::Bool(repeat::_trouble(&a.s(0))),
        ("repeat", "_a_thing_of_its_own") => Out::Bool(repeat::_a_thing_of_its_own(&a.s(0), &a.s(1))),
        ("repeat", "after_an_ack") => text(repeat::after_an_ack(&a.s(0))),
        ("repeat", "_a_feeling") => Out::Bool(repeat::_a_feeling(&a.s(0))),
        ("repeat", "_the_doing") => text(repeat::_the_doing(&a.s(0))),
        ("repeat", "_the_advice") => text(repeat::_the_advice(&a.s(0))),
        ("repeat", "the_next_move") => text(repeat::the_next_move(&a.mem(0), &a.s(1))),
        ("repeat", "_next_step") => text(repeat::_next_step(&a.s(0))),
        ("repeat", "_later_part") => text(repeat::later_part(&a.s(0), &a.s(1))),
        ("repeat", "_opens_a_doing") => Out::Bool(repeat::_opens_a_doing(&a.s(0))),
        ("repeat", "_the_untried") => {
            let mut m = a.mem(1);
            let r = repeat::_the_untried(&a.s(0), &mut m);
            *memory_after = Some(m);
            opt(r)
        }
        ("repeat", "_tried") => Out::Bool(repeat::_tried(&a.s(0))),
        ("repeat", "_says_they_agree") => Out::Bool(repeat::_says_they_agree(&a.s(0))),
        ("repeat", "_the_thing_they_named") => text(repeat::_the_thing_they_named(&a.s(0), &a.s(1))),
        ("repeat", "agreement_with_the_answer") => text(repeat::agreement_with_the_answer(&a.s(0), &a.mem(1))),
        ("repeat", "_agreed") => text(repeat::_agreed(&a.s(0), &a.mem(1))),
        ("repeat", "_the_act") => opt(repeat::_the_act(&a.s(0))),
        ("repeat", "_the_answer_gives") => Out::Bool(repeat::_the_answer_gives(&a.s(0), &a.s(1))),
        ("repeat", "_a_step_done") => Out::Bool(repeat::_a_step_done(&a.s(0), &a.s(1))),
        ("repeat", "_the_step_just_given") => Out::Bool(repeat::_the_step_just_given(&a.s(0), &a.mem(1))),
        ("repeat", "carried_on") => {
            let mut m = a.mem(1);
            let r = repeat::carried_on_text(&a.s(0), &mut m);
            *memory_after = Some(m);
            opt(r)
        }
        ("repeat", "_starting_out_names_a_thing") => Out::Bool(repeat::_starting_out_names_a_thing(&a.s(0))),
        ("repeat", "says_where_to_start") => Out::Bool(repeat::says_where_to_start(&a.s(0))),
        ("repeat", "a_more_ask") => Out::Bool(repeat::a_more_ask(&a.s(0))),
        ("repeat", "_a_list_item") => text(repeat::_a_list_item(&a.s(0))),
        ("repeat", "_pieces") => strs_out(repeat::_pieces(&a.s(0))),
        ("repeat", "_opens_on_a_verb") => Out::Bool(repeat::_opens_on_a_verb(&a.s(0))),
        ("repeat", "_the_next_step") => opt(repeat::_the_next_step(&a.s(0), &a.mem(1))),
        ("repeat", "_the_first_step") => opt(repeat::_the_first_step(&a.s(0))),
        ("repeat", "_a_new_work") => Out::Bool(repeat::_a_new_work(&a.s(0), &a.s(1))),
        ("repeat", "asks_which_part") => Out::Bool(repeat::asks_which_part(&a.s(0))),
        ("repeat", "_leads_for") => strs_out(repeat::_leads_for(&a.s(0))),
        ("repeat", "the_move") => {
            let mut m = a.mem(2);
            let r = repeat::the_move(&a.s(0), &a.s(1), &mut m);
            *memory_after = Some(m);
            opt(r)
        }
        ("repeat", "_they_say_they_tried") => Out::Bool(repeat::_they_say_they_tried(&a.s(0), &a.mem(1))),
        ("repeat", "the_figure_asked") => opt(repeat::the_figure_asked_text(&a.s(0), &a.mem(1))),
        ("repeat", "answers_nothing") => Out::Bool(repeat::answers_nothing(&a.s(0))),
        ("repeat", "the_advice_was_to_get_seen") => Out::Bool(repeat::the_advice_was_to_get_seen(&a.mem(0))),
        ("repeat", "the_part_asked") => opt(repeat::the_part_asked_text(&a.s(0), &a.mem(1))),
        ("repeat", "_whether_to") => text(repeat::_whether_to(&a.s(0))),
        ("repeat", "_the_steps_done") => {
            let mut acts: Vec<J> = repeat::_the_steps_done(&a.s(0))
                .into_iter()
                .map(|(act, thing)| {
                    let thing = match thing {
                        Some(t) => J::Array(sorted(t).into_iter().map(J::String).collect()),
                        None => J::Null,
                    };
                    J::Array(vec![J::String(act), thing])
                })
                .collect();
            acts.sort_by_key(|j| j.to_string());
            Out::Exact(J::Array(acts))
        }
        ("repeat", "_the_stems") => strs_out(sorted(repeat::_the_stems(&a.s(0)).into_iter().collect())),
        ("repeat", "_a_stem") => text(repeat::_a_stem(&a.s(0))),
        ("repeat", "_the_chat_is_still_on") => Out::Bool(repeat::_the_chat_is_still_on(&a.s(0), &a.mem(1))),
        ("repeat", "what_it_knows") => opt(repeat::what_it_knows_text(&a.s(0), &a.mem(1))),
        ("repeat", "the_plain_line") => text(repeat::the_plain_line(&a.s(0))),
        ("repeat", "how_long_its_been") => Out::Bool(repeat::how_long_its_been(&a.s(0))),
        ("repeat", "_a_line_that_corrects") => Out::Bool(repeat::_a_line_that_corrects(&a.s(0))),
        ("repeat", "_a_list_of_ages") => Out::Bool(repeat::_a_list_of_ages(&a.s(0))),
        ("repeat", "_something_gone") => Out::Bool(repeat::_something_gone(&a.strs(0), a.n(1) as usize, &a.strs(2))),
        ("repeat", "_the_ask_gave_that_figure") => Out::Bool(repeat::_the_ask_gave_that_figure(&a.s(0), &a.s(1))),
        ("repeat", "_a_count_of_one_with_a_plural") => {
            Out::Bool(repeat::_a_count_of_one_with_a_plural(&a.s(0), &a.s(1)))
        }
        ("repeat", "_the_thing_counted") => text(repeat::_the_thing_counted(&a.s(0), &a.s(1))),
        ("callback", "_the_ask") => {
            let mut m = a.mem(1);
            match callback::_the_ask(&a.s(0), &mut m) {
                Some((relation, name, age)) => Out::Exact(J::Array(vec![J::String(relation), J::String(name), age.to_plain()])),
                None => Out::Text(None),
            }
        }
        ("narrow", "_a_frequency") => {
            let sentence = a.s(0);
            let end = rsbot::py::text::byte_at(&sentence, a.n(1) as usize);
            let start = a.get(2).and_then(|v| v.as_i64()).map(|n| rsbot::py::text::byte_at(&sentence, n as usize));
            Out::Bool(narrow::_a_frequency(&sentence, end, start))
        }
        ("narrow", "_step_time") => {
            let figure = match a.get(2) {
                Some(J::Object(_)) => Some(a.re(2)),
                _ => None,
            };
            opt(narrow::_step_time(&a.s(0), &a.s(1), figure.as_ref()))
        }
        ("repeat", "_a_city_guide_is_the_answer_in_front") => Out::Bool(repeat::_a_city_guide_is_the_answer_in_front(&a.mem(0))),
        ("repeat", "_a_lead_for") => text(repeat::_a_lead_for(&a.strs(0), &a.s(1), &a.mem(2), &a.s(3))),
        ("repeat", "_a_verb_after_a_leading_phrase") => Out::Bool(repeat::_a_verb_after_a_leading_phrase(&a.s(0), &a.set(1))),
        ("repeat", "_the_first_instruction_in_a_long_piece") => {
            opt(repeat::_the_first_instruction_in_a_long_piece(&a.s(0), &a.set(1)))
        }
        ("repeat", "_content") => strs_out(sorted(repeat::_content(&a.s(0)))),
        ("repeat", "_the_keys_of") => strs_out(sorted(repeat::_the_keys_of(&a.s(0), &a.mem(1)))),
        ("repeat", "_doings") => {
            let mut found: Vec<usize> = repeat::_doings(&a.s(0)).into_iter().collect();
            found.sort();
            Out::Exact(J::Array(found.into_iter().map(|i| J::from(i as i64)).collect()))
        }
        ("repeat", "_frame_heads") => Out::ByLength(repeat::_frame_heads().to_vec()),
        ("repeat", "_inside_a_bracket") => {
            let line = a.s(0);
            Out::Bool(repeat::_inside_a_bracket(&line, rsbot::py::text::byte_at(&line, a.n(1) as usize)))
        }
        ("repeat", "_places_to_cut") => {
            let head = a.s(0);
            Out::Exact(J::Array(
                repeat::_places_to_cut(&head).into_iter().map(|b| J::from(rsbot::py::text::char_index(&head, b) as i64)).collect(),
            ))
        }
        ("repeat", "_one_figure") => {
            let found: Vec<repeat::Figure> = match a.get(0) {
                Some(J::Array(fs)) => fs
                    .iter()
                    .map(|f| {
                        let s = |i: usize| f[i].as_str().unwrap_or("").to_string();
                        (s(0), s(1), s(2), f[3].as_bool().unwrap_or(false))
                    })
                    .collect(),
                _ => Vec::new(),
            };
            match repeat::_one_figure(&found, a.opt_s(1).as_deref()) {
                Some((k, w, n, c)) => Out::Exact(J::Array(vec![J::String(k), J::String(w), J::String(n), J::Bool(c)])),
                None => Out::Exact(J::Array(Vec::new())),
            }
        }
        ("repeat", "_the_answer_about_what_we_are_on") => {
            let recent: Vec<(usize, String, String)> = match a.get(1) {
                Some(J::Array(rs)) => rs
                    .iter()
                    .map(|r| (r[0].as_u64().unwrap_or(0) as usize, r[1].as_str().unwrap_or("").to_string(), r[2].as_str().unwrap_or("").to_string()))
                    .collect(),
                _ => Vec::new(),
            };
            Out::Exact(J::Array(
                repeat::_the_answer_about_what_we_are_on(&a.mem(0), recent)
                    .into_iter()
                    .map(|(b, q, s)| J::Array(vec![J::from(b as i64), J::String(q), J::String(s)]))
                    .collect(),
            ))
        }
        ("repeat", "_where_the_rest_is") => {
            let turn = a.value(2);
            let field = |k: &str| turn.get(k).and_then(|v| v.as_str().map(str::to_string));
            let turn = rsbot::state::Turn {
                user: field("user").unwrap_or_default(),
                bot: field("bot").unwrap_or_default(),
                module: field("module"),
            };
            text(repeat::_where_the_rest_is(&a.s(0), &a.mem(1), &turn, &a.s(3)))
        }
        ("repeat", "_figures_in") => Out::Exact(J::Array(
            repeat::_figures_in(&a.s(0))
                .into_iter()
                .map(|(k, s, n, c)| J::Array(vec![J::String(k), J::String(s), J::String(n), J::Bool(c)]))
                .collect(),
        )),
        ("repeat", "_money_paid_for_a_thing") => Out::Bool(repeat::_money_paid_for_a_thing(&a.s(0), &a.s(1))),
        ("repeat", "_a_bare_count") => Out::Bool(repeat::_a_bare_count(&a.s(0))),
        ("repeat", "_the_same_word") => Out::Bool(repeat::_the_same_word(&a.s(0), &a.s(1))),
        ("repeat", "_the_length_they_mean") => text(repeat::_the_length_they_mean(&a.s(0), &a.strs(1), &a.strs(2))),
        ("repeat", "_the_figure_counts_the_thing") => Out::Bool(repeat::_the_figure_counts_the_thing(&a.s(0), &a.s(1))),
        ("repeat", "a_narrowing_and_its_line") => {
            Out::Exact(repeat::a_narrowing_and_its_line(&a.s(0), &a.mem(1)).to_plain())
        }
        ("repeat", "_the_turn_that_names_it") => opt(repeat::_the_turn_that_names_it(&a.s(0), &a.mem(1))),
        ("repeat", "the_figure_they_gave") => opt(repeat::the_figure_they_gave_text(&a.s(0), &a.mem(1))),
        ("repeat", "_the_store_keeps_this_figure") => Out::Bool(repeat::_the_store_keeps_this_figure(&a.s(0), &a.mem(1))),
        ("repeat", "asked_back") => opt(repeat::asked_back_text(&a.s(0), &a.mem(1))),
        ("repeat", "carries_the_figure") => Out::Bool(repeat::carries_the_figure(&a.value(0), &a.s(1))),
        ("repeat", "_the_same_words") => Out::Bool(repeat::_the_same_words(&a.s(0), &a.s(1))),
        ("repeat", "_shortened") => text(repeat::_shortened(&a.s(0))),
        ("repeat", "_the_answer_again") => text(repeat::_the_answer_again(&a.s(0), &a.mem(1))),
        ("repeat", "the_same_question_again") => opt(repeat::the_same_question_again_text(&a.s(0), &a.mem(1))),
        _ => return ack_call(name, a, memory_after),
    };
    Some(out)
}

/// The ack adapters (kept apart, so the list above stays readable).
fn ack_call(name: &str, a: &A, memory_after: &mut Option<Memory>) -> Option<Out> {
    macro_rules! with_mem {
        ($i:expr, $m:ident, $e:expr) => {{
            let mut $m = a.mem($i);
            let r = $e;
            *memory_after = Some($m);
            r
        }};
    }
    let out = match name {
        "_a_relation" => text(ack::_a_relation(&a.s(0))),
        "_the_pronoun" => text(ack::_the_pronoun(&a.s(0))),
        "a_person_named_alone" => text(with_mem!(1, m, ack::a_person_named_alone(&a.s(0), &mut m))),
        "a_person_pointed_at" => text(with_mem!(1, m, ack::a_person_pointed_at(&a.s(0), &mut m))),
        "a_second_person" => text(ack::a_second_person(&a.s(0), &a.mem(1))),
        "_the_last_reply_gave_a_plan" => Out::Bool(ack::_the_last_reply_gave_a_plan(&a.mem(0))),
        "_a_limit_on_the_plan" => opt(ack::a_limit_on_the_plan_text(&a.s(0), &a.mem(1))),
        "_a_day_coming" => match ack::_a_day_coming(&a.s(0)) {
            Some(v) => strs_out(v),
            None => Out::Exact(J::Null),
        },
        "reads_as_a_receipt" => Out::Bool(ack::reads_as_a_receipt(&a.s(0))),
        "_one_frame" => Out::Bool(ack::_one_frame(&a.s(0))),
        "reads_as_a_trip" => opt(ack::reads_as_a_trip(&a.s(0))),
        "_the_plan_in_hand" => opt(ack::_the_plan_in_hand(&a.mem(0), &a.s(1))),
        "_a_like_of_one_thing" => Out::Bool(ack::_a_like_of_one_thing(&a.s(0), &a.mem(1))),
        "_only_a_liking" => Out::Bool(ack::_only_a_liking(&a.value(0))),
        "a_liked_thing" => text(ack::a_liked_thing(&a.value(0))),
        "a_limit_reply" => opt(ack::a_limit_reply_text(&a.s(0), &a.mem(1))),
        "_a_plan_on_the_table" => Out::Bool(ack::_a_plan_on_the_table(&a.mem(0), &a.s(1))),
        "a_limit_for_the_plan" => opt(ack::a_limit_for_the_plan_text(&a.s(0), &a.mem(1))),
        "reads_as_cruelty" => Out::Bool(ack::reads_as_cruelty(&a.s(0))),
        "reads_as_an_ask" => Out::Bool(ack::reads_as_an_ask(&a.s(0))),
        "_shape_of" => text(ack::_shape_of(&a.s(0))),
        "_clock_shape" => text(ack::_clock_shape(&a.s(0))),
        "_tells_them_what_to_do" => Out::Bool(ack::_tells_them_what_to_do(&a.s(0))),
        "_a_bare_state" => Out::Bool(ack::_a_bare_state(&a.s(0))),
        "has_news" => Out::Bool(ack::has_news(&a.s(0))),
        "_repaired" => text(ack::_repaired(&a.s(0), &a.mem(1))),
        "_the_bare_value" => text(ack::_the_bare_value(&a.s(0))),
        "value_put_right" => text(ack::value_put_right(&a.s(0), &a.mem(1))),
        "_the_line_before" => text(ack::_the_line_before(&a.mem(0))),
        "_the_unit_before" => text(ack::_the_unit_before(&a.mem(0))),
        "pet_given_away" => opt(ack::pet_given_away_text(&a.s(0), &a.mem(1))),
        "_offers_a_chat" => opt(ack::offers_a_chat_text(&a.s(0), &a.mem(1))),
        "_a_date_on_the_thing_in_hand" => opt(ack::a_date_on_the_thing_in_hand_text(&a.s(0), &a.mem(1))),
        "_after_the_hello" => text(ack::_after_the_hello(&a.s(0))),
        "reply" => opt(with_mem!(1, m, ack::reply(&a.s(0), &mut m))),
        "_a_remark_on_what_was_said" => opt(ack::_a_remark_on_what_was_said(&a.s(0), &a.mem(1))),
        "_a_price_still_to_come" => Out::Bool(ack::_a_price_still_to_come(&a.s(0), &a.s(1))),
        "_a_result_is_on_the_table" => Out::Bool(ack::_a_result_is_on_the_table(&a.s(0))),
        "_ending_of" => text(ack::_ending_of(&a.s(0))),
        "_after_a_feeling" => Out::Bool(ack::_after_a_feeling(&a.s(0), &a.mem(1))),
        "_a_heavy_reply_just_before" => Out::Bool(ack::_a_heavy_reply_just_before(&a.mem(0), a.n(1))),
        "_bad_news_just_before" => Out::Bool(ack::_bad_news_just_before(&a.s(0), &a.mem(1))),
        "carries_the_story_on" => Out::Bool(ack::carries_the_story_on(&a.s(0), &a.mem(1))),
        "the_old_one_aged" => opt(ack::the_old_one_aged_text(&a.s(0), &a.mem(1))),
        "_the_length" => opt(ack::the_length_text(&a.s(0), &a.mem(1))),
        "_the_hours" => opt(with_mem!(2, m, ack::the_hours_text(&a.s(0), &a.s(1), &mut m))),
        "_a_deadline" => opt(ack::a_deadline_text(&a.s(0), &a.s(1), &a.mem(2))),
        "_an_hour_on_the_day" => opt(ack::_an_hour_on_the_day(&a.s(0))),
        "_the_figure_is_an_hour" => Out::Bool(ack::_the_figure_is_an_hour(&a.s(0))),
        "_a_clock_time" => opt(ack::a_clock_time_text(&a.s(0), &a.s(1), &a.mem(2))),
        "_nothing_added_but_the_pet" => Out::Bool(ack::_nothing_added_but_the_pet(&a.s(0), &a.value(1))),
        "_a_pet_that_is_all_right" => Out::Bool(ack::_a_pet_that_is_all_right(&a.s(0), &a.value(1))),
        "_a_wanted_animal" => Out::Bool(ack::_a_wanted_animal(&a.s(0))),
        "_an_animal_wanted" => text(ack::_an_animal_wanted(&a.s(0))),
        "_an_animal_they_had_once" => Out::Bool(ack::_an_animal_they_had_once(&a.s(0))),
        "_the_animal_coming" => text(ack::_the_animal_coming(&a.value(0))),
        "_an_animal_turned_down" => Out::Bool(ack::_an_animal_turned_down(&a.value(0), &a.mem(1))),
        "_on_record" => Out::Bool(ack::_on_record(&a.s(0), &a.mem(1))),
        "_an_animal_only_planned" => text(ack::_an_animal_only_planned(&a.value(0), &a.s(1), &a.mem(2))),
        "_only_a_wish" => Out::Bool(ack::_only_a_wish(&a.value(0))),
        "_an_animal_taken_back" => Out::Bool(ack::_an_animal_taken_back(&a.value(0))),
        "_plain" => opt(ack::_plain(&a.s(0), &a.mem(1))),
        "_about_them" => Out::Bool(ack::_about_them(&a.s(0))),
        "_who_is_getting_better" => text(ack::_who_is_getting_better(&a.s(0))),
        "a_person_getting_better" => match ack::a_person_getting_better_pool(&a.s(0), &a.mem(1)) {
            Some(v) => strs_out(v),
            None => Out::Exact(J::Null),
        },
        "_the_words_of_the_last_two_turns" => strs_out(ack::_the_words_of_the_last_two_turns(&a.mem(0))),
        "nobody_around" => match ack::nobody_around_pool(&a.s(0), &a.mem(1)) {
            Some(v) => strs_out(v),
            None => Out::Exact(J::Null),
        },
        "_said_to_the_bot" => Out::Bool(ack::_said_to_the_bot(&a.s(0))),
        "_no_subject" => Out::Bool(ack::_no_subject(&a.s(0), &a.mem(1))),
        "_asked_last" => Out::Bool(ack::_asked_last(&a.mem(0))),
        "_a_diary_plan" => match ack::_a_diary_plan(&a.value(0), &a.s(1)) {
            Some(v) => strs_out(v),
            None => Out::Exact(J::Null),
        },
        "_plan_tails" => match ack::_plan_tails(&a.value(0), &a.s(1), &a.mem(2)) {
            Some(v) => strs_out(v),
            None => Out::Exact(J::Null),
        },
        "the_job_they_had" => opt(ack::the_job_they_had_text(&a.s(0), &a.mem(1))),
        "a_job_to_do" => opt(ack::a_job_to_do(&a.s(0), &a.mem(1))),
        "_a_job_to_do" => opt(ack::_a_job_to_do(&a.s(0), &a.mem(1))),
        "_a_loss" => Out::Bool(ack::_a_loss(&a.s(0), &a.mem(1), &a.s(2))),
        "_about_a_body" => Out::Bool(ack::_about_a_body(&a.s(0), &a.mem(1))),
        "_children_with_ages" => Out::Exact(J::Array(ack::_children_with_ages(&a.s(0)).iter().map(|v| v.to_plain()).collect())),
        "_hours_family" => Out::Exact(ack::_hours_family(&a.s(0)).to_plain()),
        "_pick_ending" => {
            let fields: Vec<(String, String)> =
                a.kwargs.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect();
            let fields: Vec<(&str, &str)> = fields.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
            text(ack::_pick_ending(&a.strs(0), &a.mem(1), &fields))
        }
        "_the_opening_instead" => opt(ack::_the_opening_instead(a.opt_s(0).as_deref(), &a.s(1), &a.mem(2))),
        "_without_a_frame" => {
            let phrases: Vec<Option<Vec<String>>> = match a.get(1) {
                Some(J::Array(ps)) => ps
                    .iter()
                    .map(|p| p.as_array().map(|ws| ws.iter().map(|w| w.as_str().unwrap_or("").to_string()).collect()))
                    .collect(),
                _ => Vec::new(),
            };
            let (words, took) = ack::_without_a_frame(&a.strs(0), &phrases, a.b(2));
            Out::Exact(J::Array(vec![J::Array(words.into_iter().map(J::String).collect()), J::Bool(took)]))
        }
        "_without_a_not" => {
            let (words, took) = ack::_without_a_not(&a.strs(0));
            Out::Exact(J::Array(vec![J::Array(words.into_iter().map(J::String).collect()), J::Bool(took)]))
        }
        "_somebody_was_being_hurt" => Out::Bool(ack::_somebody_was_being_hurt(&a.mem(0))),
        "_who_the_figure_is" => opt(with_mem!(1, m, ack::who_the_figure_is_text(&a.value(0), &mut m, &a.s(2)))),
        "_a_condition_they_wonder_about" => text(ack::_a_condition_they_wonder_about(&a.s(0), &a.mem(1))),
        "_a_thing_they_have" => opt(ack::a_thing_they_have_text(&a.s(0), &a.mem(1))),
        "_a_spending_budget" => Out::Bool(ack::_a_spending_budget(&a.s(0), &a.mem(1))),
        "_frames" => strs_out(ack::_frames(&a.value(0), a.b(1), &a.s(2), &a.mem(3))),
        "_about_someone_else" => Out::Bool(ack::_about_someone_else(&a.value(0), &a.s(1))),
        "_a_visit" => Out::Bool(ack::_a_visit(&a.s(0))),
        "_the_milestone" => Out::Bool(ack::_the_milestone(&a.s(0))),
        "_a_workplace" => Out::Bool(ack::_a_workplace(&a.value(0), a.b(1), &a.mem(2))),
        "_about_the_family" => Out::Bool(ack::_about_the_family(&a.value(0))),
        "_just_a_liking" => Out::Bool(ack::_just_a_liking(&a.value(0))),
        "_heavy" => Out::Bool(ack::_heavy(&a.mem(0))),
        "_statement" => Out::Bool(ack::_statement(&a.mem(0))),
        "_learned" => Out::Exact(ack::_learned(&a.s(0), Some(&a.mem(1))).to_plain()),
        "_adds_one_more_allergen" => strs_out(ack::_adds_one_more_allergen(&a.s(0), Some(&a.mem(1)))),
        "_adds_a_detail" => Out::Bool(ack::_adds_a_detail(&a.s(0), Some(&a.mem(1)))),
        "_words" => strs_out(ack::_words(&a.s(0))),
        "_names_nothing" => Out::Bool(ack::_names_nothing(&a.s(0))),
        "_repairs" => text(ack::_repairs(&a.s(0), &a.mem(1))),
        "_says_something" => Out::Bool(ack::_says_something(&a.s(0))),
        "_the_person_the_pointer_is_about" => text(ack::_the_person_the_pointer_is_about(&a.s(0), &a.mem(1))),
        "_plural" => text(ack::_plural(&a.s(0))),
        "_is_a_contraction" => Out::Bool(ack::_is_a_contraction(&a.s(0))),
        "_a_loss_just_before" => Out::Bool(ack::_a_loss_just_before(&a.mem(0))),
        "_another_of_what_we_said" => opt(ack::another_of_what_we_said_text(&a.s(0), &a.mem(1))),
        "_only_the_correction_in_front" => Out::Bool(ack::_only_the_correction_in_front(&a.s(0), &a.s(1))),
        "_echo" => text(ack::_echo(&a.s(0), Some(&a.mem(1)))),
        "said_back" => Out::Exact(ack::said_back(&a.s(0), &a.mem(1)).to_plain()),
        "_only_a_dislike" => Out::Bool(ack::_only_a_dislike(&a.s(0), &a.value(1))),
        "only_a_dislike" => Out::Bool(ack::only_a_dislike(&a.s(0), &a.mem(1))),
        "_tells_it" => Out::Bool(ack::_tells_it(&a.s(0), &a.value(1))),
        "_a_diet" => Out::Bool(ack::_a_diet(&a.value(0))),
        "_only_the_figure" => Out::Bool(ack::_only_the_figure(&a.s(0))),
        "_the_rest_of_a_question" => Out::Bool(ack::_the_rest_of_a_question(&a.s(0), &a.s(1))),
        "_after_a_reaction" => text(ack::_after_a_reaction(&a.s(0))),
        "_strip_lead_in" => text(ack::_strip_lead_in(&a.s(0), a.b(1))),
        "without_the_frame" => Out::Exact(ack::without_the_frame(&a.s(0)).to_plain()),
        "_spoken" => text(ack::_spoken(&a.s(0))),
        "_pick_off" => text(ack::_pick_off(&a.s(0))),
        "_tail_off" => text(ack::_tail_off(&a.s(0))),
        "_contrast_off" => text(ack::_contrast_off(&a.s(0))),
        "_reason_off" => text(ack::_reason_off(&a.s(0), &a.s(1))),
        "_a_value" => Out::Bool(ack::_a_value(&a.s(0))),
        "_cut" => text(ack::_cut(&a.s(0))),
        "_tidy" => text(ack::_tidy(&a.s(0))),
        "_a_thing_the_line_named" => strs_out(sorted(ack::_a_thing_the_line_named(&a.s(0)))),
        "_the_units_measure" => strs_out(sorted(ack::_the_units_measure(&a.s(0)))),
        "_the_values_measures" => strs_out(sorted(ack::_the_values_measures(&a.s(0)))),
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
