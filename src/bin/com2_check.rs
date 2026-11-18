//! Replay the calls `harness/com2_calls.py` recorded against the Rust port of the COM2 files
//! (bot/compose/verdict.py, onscreen.py, fallback_chain.py).
//!
//!     com2_check CALLS.jsonl.gz [--fn SUBSTRING] [--show K]
//!
//! Per function: calls that give PyBot's result (and, for a function that writes memory, PyBot's
//! memory after it), calls that differ, and calls that reached a stub not ported yet.
use flate2::read::GzDecoder;
use rsbot::compose::{fallback_chain as fc, onscreen as os, verdict as v};
use rsbot::runtime::{guarded, quiet_panics, take_last_panic};
use rsbot::state::{Memory, Turn};
use rsbot::understood::Understood;
use serde_json::{json, Map, Value as J};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::BufRead;

struct Call<'a> {
    args: Vec<Option<&'a J>>,
    mems: &'a HashMap<String, Map<String, J>>,
}

impl<'a> Call<'a> {
    fn get(&self, i: usize) -> Option<&'a J> {
        self.args.get(i).copied().flatten()
    }
    fn s(&self, i: usize) -> String {
        match self.get(i) {
            Some(J::String(s)) => s.clone(),
            _ => String::new(),
        }
    }
    fn os(&self, i: usize) -> Option<String> {
        match self.get(i) {
            Some(J::String(s)) => Some(s.clone()),
            _ => None,
        }
    }
    fn b(&self, i: usize, default: bool) -> bool {
        match self.get(i) {
            Some(J::Bool(b)) => *b,
            Some(J::Null) => false,
            None => default,
            Some(_) => true,
        }
    }
    fn n(&self, i: usize, default: i64) -> i64 {
        self.get(i).and_then(|x| x.as_i64()).unwrap_or(default)
    }
    fn strs(&self, i: usize) -> Vec<String> {
        match self.get(i) {
            Some(J::Array(a)) => a.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect(),
            _ => vec![],
        }
    }
    fn mem(&self, i: usize) -> Memory {
        match self.get(i) {
            Some(J::Object(o)) if o.contains_key("$memory") => {
                let h = o["$memory"].as_str().unwrap_or("");
                self.mems.get(h).map(Memory::from_plain).unwrap_or_default()
            }
            _ => Memory::new(),
        }
    }
    fn has_mem(&self, i: usize) -> bool {
        matches!(self.get(i), Some(J::Object(o)) if o.contains_key("$memory"))
    }
    fn und(&self, i: usize) -> Understood {
        match self.get(i) {
            Some(J::Object(o)) if o.contains_key("$understood") => serde_json::from_value(o["$understood"].clone()).unwrap_or_default(),
            _ => Understood::default(),
        }
    }
    fn has_und(&self, i: usize) -> bool {
        matches!(self.get(i), Some(J::Object(o)) if o.contains_key("$understood"))
    }
    fn turn(&self, i: usize) -> Turn {
        turn_of(self.get(i).unwrap_or(&J::Null))
    }
    fn turns(&self, i: usize) -> Vec<Turn> {
        match self.get(i) {
            Some(J::Array(a)) => a.iter().map(turn_of).collect(),
            _ => vec![],
        }
    }
}

fn turn_of(t: &J) -> Turn {
    Turn {
        user: t.get("user").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        bot: t.get("bot").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        module: t.get("module").and_then(|x| x.as_str()).map(str::to_string),
    }
}

fn turns_json(turns: &[Turn]) -> J {
    J::Array(turns.iter().map(|t| json!({"user": t.user, "bot": t.bot, "module": t.module})).collect())
}

fn sorted(v: J) -> J {
    match v {
        J::Array(mut a) => {
            a.sort_by_key(|x| x.to_string());
            J::Array(a)
        }
        x => x,
    }
}

/// Values that mean "absent" in Python reads, as `turn_check` reads them.
fn blank(v: &J) -> bool {
    match v {
        J::Null => true,
        J::Bool(b) => !b,
        J::String(s) => s.is_empty(),
        J::Array(a) => a.is_empty(),
        J::Object(o) => o.is_empty(),
        J::Number(n) => n.as_f64() == Some(0.0),
    }
}
fn norm(v: &J) -> J {
    match v {
        J::Object(o) => J::Object(o.iter().filter(|(_, x)| !blank(x)).map(|(k, x)| (k.clone(), norm(x))).collect()),
        J::Array(a) => J::Array(a.iter().map(norm).collect()),
        x => x.clone(),
    }
}

/// What a call gave back, as JSON, and the memory after it when the function writes memory.
type Out = (J, Option<Memory>);

fn opt(s: Option<String>) -> J {
    s.map(J::String).unwrap_or(J::Null)
}
fn opt_pool(p: Option<Vec<String>>) -> J {
    p.map(|v| json!(v)).unwrap_or(J::Null)
}
fn set_json(s: HashSet<String>) -> J {
    sorted(json!(s.into_iter().collect::<Vec<_>>()))
}
fn idx(i: Option<usize>) -> J {
    i.map(|x| json!(x)).unwrap_or(J::Null)
}

/// Run one recorded call. None when the function has no replay here.
fn run(name: &str, c: &Call) -> Option<Out> {
    let r = |j: J| Some((j, None));
    match name {
        // ---- onscreen ----
        "bot.compose.onscreen:_stem" => r(json!(os::_stem(&c.s(0)))),
        "bot.compose.onscreen:_the_same_word" => r(json!(os::_the_same_word(&c.s(0), &c.strs(1)))),
        "bot.compose.onscreen:_stems" => r(sorted(json!(os::_stems(&c.strs(0))))),
        "bot.compose.onscreen:_names_something_new" => r(json!(os::_names_something_new(&c.strs(0), &c.strs(1), &c.strs(2)))),
        "bot.compose.onscreen:_the_words_in_the_chat" => r(sorted(json!(os::_the_words_in_the_chat(&c.mem(0), &c.s(1))))),
        "bot.compose.onscreen:_the_answer_to_hand_back" => r(json!(os::_the_answer_to_hand_back(&c.mem(0)))),
        "bot.compose.onscreen:_the_answer_is_a_card" => r(json!(os::_the_answer_is_a_card(&c.mem(0)))),
        "bot.compose.onscreen:_the_parts" => r(json!(os::_the_parts(&c.s(0)))),
        "bot.compose.onscreen:_the_sentence_to_hand_back" => r(json!(os::_the_sentence_to_hand_back(&c.s(0), &c.strs(1), &c.strs(2)))),
        "bot.compose.onscreen:the_answer_on_screen" => r(json!(os::the_answer_on_screen(&c.s(0), &c.mem(1)))),
        "bot.compose.onscreen:earlier_turn" => r(json!(os::earlier_turn(&c.mem(0)))),
        "bot.compose.onscreen:talks_about_the_conversation" => r(json!(os::talks_about_the_conversation(&c.s(0)))),
        // ---- verdict ----
        "bot.compose.verdict:_a_job_somewhere" => r(json!(v::_a_job_somewhere(&c.s(0)))),
        "bot.compose.verdict:the_fix_asked" => r(json!(v::the_fix_asked(&c.s(0), &c.mem(1)))),
        "bot.compose.verdict:_the_list_before_the_call" => r(json!(v::_the_list_before_the_call(&c.s(0)))),
        "bot.compose.verdict:_the_call_and_its_reason" => r(json!(v::_the_call_and_its_reason(&c.s(0)))),
        "bot.compose.verdict:_the_sign_words" => r(set_json(v::_the_sign_words(&c.s(0)))),
        "bot.compose.verdict:_they_named_the_sign_themselves" => r(json!(v::_they_named_the_sign_themselves(&c.s(0), &c.mem(1)))),
        "bot.compose.verdict:_the_person_for" => r(json!(v::_the_person_for(c.os(0).as_deref(), c.os(1).as_deref()))),
        "bot.compose.verdict:_about_a_body" => {
            let texts: Vec<String> = c.args.iter().map(|a| a.and_then(|x| x.as_str()).unwrap_or("").to_string()).collect();
            let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
            r(json!(v::_about_a_body(&refs)))
        }
        "bot.compose.verdict:_a_doubt_about_money" => r(json!(v::_a_doubt_about_money(&c.mem(0)))),
        "bot.compose.verdict:_is_a_comparison" => r(json!(v::_is_a_comparison(&c.s(0)))),
        "bot.compose.verdict:_a_figure_owed" => r(json!(v::_a_figure_owed(&c.s(0)))),
        "bot.compose.verdict:_is_an_explanation" => r(json!(v::_is_an_explanation(&c.s(0)))),
        "bot.compose.verdict:_own_pattern" => r(json!(v::_own_pattern(&c.s(0)))),
        "bot.compose.verdict:takes_a_fact_back" => r(json!(v::takes_a_fact_back(&c.s(0)))),
        "bot.compose.verdict:_say" => r(json!(v::_say(&c.strs(0), &c.mem(1), &c.s(2)))),
        "bot.compose.verdict:_a_report_line" => r(json!(v::_a_report_line(&c.s(0)))),
        "bot.compose.verdict:_flat" => r(json!(v::_flat(&c.s(0)))),
        "bot.compose.verdict:rules_on_the_doubt" => r(json!(v::rules_on_the_doubt(&c.s(0)))),
        "bot.compose.verdict:opens_on_a_job" => r(json!(v::opens_on_a_job(&c.s(0)))),
        "bot.compose.verdict:carries_the_answer" => r(json!(v::carries_the_answer(&c.s(0), &c.mem(1)))),
        "bot.compose.verdict:_a_punt" => r(json!(v::_a_punt(&c.turn(0)))),
        "bot.compose.verdict:_skip_the_derived" => {
            let turns = c.turns(0);
            r(turns_json(v::_skip_the_derived(&turns, &c.mem(1))))
        }
        "bot.compose.verdict:answers_the_ask" => r(json!(v::answers_the_ask(&c.s(0), &c.mem(1)))),
        "bot.compose.verdict:_the_news_just_before" => r(json!(v::_the_news_just_before(&c.mem(0)))),
        "bot.compose.verdict:_last_reply" => r(json!(v::_last_reply(&c.mem(0), c.b(1, false), c.b(2, false)))),
        "bot.compose.verdict:_words" => r(json!(v::_words(&c.s(0)))),
        "bot.compose.verdict:_is_an_echo" => r(json!(v::_is_an_echo(&c.turn(0)))),
        "bot.compose.verdict:_is_a_recap" => r(json!(v::_is_a_recap(&c.turn(0), &c.turn(1)))),
        "bot.compose.verdict:_last_line_that_named_a_call" => r(json!(v::_last_line_that_named_a_call(&c.mem(0)))),
        "bot.compose.verdict:_a_call_for_now" => r(json!(v::_a_call_for_now(&c.s(0)))),
        "bot.compose.verdict:_the_call_sentence" => r(json!(v::_the_call_sentence(&c.s(0)))),
        "bot.compose.verdict:_last_person_line" => r(json!(v::_last_person_line(&c.mem(0)))),
        "bot.compose.verdict:_the_worst_thing_they_said" => r(json!(v::_the_worst_thing_they_said(&c.mem(0)))),
        "bot.compose.verdict:_as_written" => {
            let (a, b) = v::_as_written(&c.s(0));
            r(json!([a, b]))
        }
        "bot.compose.verdict:_a_thing_they_own" => r(json!(v::_a_thing_they_own(&c.mem(0)))),
        "bot.compose.verdict:_a_person_they_know" => r(json!(v::_a_person_they_know(&c.mem(0), &c.s(1)))),
        "bot.compose.verdict:_the_line_behind" => r(json!(v::_the_line_behind(&c.mem(0)))),
        "bot.compose.verdict:_the_plan_they_brought" => r(json!(v::_the_plan_they_brought(&c.mem(0), c.n(1, 2) as usize))),
        "bot.compose.verdict:_considering" => r(json!(v::_considering(&c.mem(0)))),
        "bot.compose.verdict:_the_last_line_was_mine" => r(json!(v::_the_last_line_was_mine(&c.mem(0)))),
        "bot.compose.verdict:_last_module" => r(json!(v::_last_module(&c.mem(0), c.b(1, false)))),
        "bot.compose.verdict:_the_plan_can_hurt" => r(opt(v::_the_plan_can_hurt(&c.s(0)))),
        "bot.compose.verdict:_the_answer_already_warned" => r(json!(v::_the_answer_already_warned(&c.s(0)))),
        "bot.compose.verdict:_a_plan_doubt" => r(opt(v::_a_plan_doubt(&c.mem(0)))),
        "bot.compose.verdict:_about_the_person" => r(json!(v::_about_the_person(&c.mem(0)))),
        "bot.compose.verdict:_own_doing" => r(json!(v::_own_doing(&c.mem(0)))),
        "bot.compose.verdict:_a_plan_or_a_choice" => r(json!(v::_a_plan_or_a_choice(&c.mem(0)))),
        "bot.compose.verdict:_good_news" => r(json!(v::_good_news(&c.mem(0)))),
        "bot.compose.verdict:_an_event_that_is_over" => r(json!(v::_an_event_that_is_over(&c.mem(0)))),
        "bot.compose.verdict:_nothing_to_be_long_about" => r(json!(v::_nothing_to_be_long_about(&c.mem(0)))),
        "bot.compose.verdict:_first" => match c.get(0) {
            Some(J::String(s)) => r(idx(v::_first(s, &c.strs(1)))),
            _ => None,
        },
        "bot.compose.verdict:_a_throw_away" => {
            let (a, t) = v::_a_throw_away(&c.s(0));
            r(json!([a, t]))
        }
        "bot.compose.verdict:_the_person_line" => r(json!(v::_the_person_line(&c.mem(0), &c.s(1)))),
        "bot.compose.verdict:_said_no" => r(opt(v::_said_no(&c.s(0)))),
        "bot.compose.verdict:_said_enough" => r(opt(v::_said_enough(&c.s(0)))),
        "bot.compose.verdict:_the_person_gave_the_sign" => r(json!(v::_the_person_gave_the_sign(&c.mem(0), &c.s(1)))),
        "bot.compose.verdict:_care_for_now" => r(json!(v::_care_for_now(&c.s(0)))),
        "bot.compose.verdict:_sentence_at" => r(json!(v::_sentence_at(&c.s(0), c.n(1, 0) as usize))),
        "bot.compose.verdict:_behind_a_description" => r(json!(v::_behind_a_description(&c.s(0), c.n(1, 0) as usize))),
        "bot.compose.verdict:_not_fenced" => r(json!(v::_not_fenced(&c.s(0), c.n(1, 0) as usize, c.get(2).and_then(|x| x.as_i64()).map(|x| x as usize)))),
        "bot.compose.verdict:_said_without_a_not" => r(json!(v::_said_without_a_not(&c.s(0), c.n(1, 0) as usize, c.n(2, 0) as usize))),
        "bot.compose.verdict:_first_positive" => r(idx(v::_first_positive(&c.s(0), &c.strs(1)))),
        "bot.compose.verdict:_unconditional_care" => r(idx(v::_unconditional_care(&c.s(0)))),
        "bot.compose.verdict:_the_fence_is_met" => r(json!(v::_the_fence_is_met(&c.s(0), &c.s(1), &c.mem(2)))),
        "bot.compose.verdict:_a_frequency_already_said" => r(json!(c.has_mem(2) && v::_a_frequency_already_said(&c.s(0), &c.s(1), &c.mem(2)))),
        "bot.compose.verdict:_condition_after_care" => {
            let m = c.mem(2);
            let at = c.get(1).and_then(|x| x.as_i64()).filter(|x| *x >= 0).map(|x| x as usize);
            r(json!(v::_condition_after_care(&c.s(0), at, if c.has_mem(2) { Some(&m) } else { None })))
        }
        "bot.compose.verdict:_cut_condition" => r(json!(v::_cut_condition(&c.s(0), c.n(1, 22) as usize))),
        "bot.compose.verdict:_first_sentence_end" => r(json!(v::_first_sentence_end(&c.s(0)))),
        "bot.compose.verdict:_told_to_act" => r(json!(v::_told_to_act(&c.s(0), c.n(1, 0) as usize))),
        "bot.compose.verdict:_referral" => r(rsbot::data::Value::to_plain(&v::_referral(&c.s(0)))),
        "bot.compose.verdict:_a_plain_go" => r(json!(v::_a_plain_go(c.n(0, 0) as usize, &c.s(1)))),
        "bot.compose.verdict:_opening" => r(json!(v::_opening(&c.s(0)))),
        "bot.compose.verdict:_the_thing" => r(set_json(v::_the_thing(&c.s(0)))),
        "bot.compose.verdict:_referral_about_the_thing" => r(json!(v::_referral_about_the_thing(&c.s(0), &c.s(1)))),
        "bot.compose.verdict:_echoes_the_person" => r(json!(v::_echoes_the_person(&c.s(0), &c.mem(1)))),
        "bot.compose.verdict:_kept_step" => r(json!(v::_kept_step(&c.s(0)))),
        "bot.compose.verdict:_only_a_verdict" => r(json!(v::_only_a_verdict(&c.s(0)))),
        "bot.compose.verdict:_the_line_just_given" => r(json!(v::_the_line_just_given(&c.mem(0)))),
        "bot.compose.verdict:_the_answer_before_that" => r(json!(v::_the_answer_before_that(&c.mem(0)))),
        "bot.compose.verdict:_says_it_again" => r(json!(v::_says_it_again(&c.s(0), &c.s(1)))),
        "bot.compose.verdict:_already_asked_for_a_step" => r(json!(v::_already_asked_for_a_step(&c.mem(0)))),
        "bot.compose.verdict:_the_referral_sentence" => r(json!(v::the_referral_sentence(&c.s(0)))),
        "bot.compose.verdict:_the_line_is_the_threshold" => r(json!(v::_the_line_is_the_threshold(&c.s(0), &c.s(1)))),
        "bot.compose.verdict:_most_of_the_line" => r(json!(v::_most_of_the_line(&c.s(0), &c.s(1)))),
        "bot.compose.verdict:_step" => r(opt(v::the_step(&c.s(0)))),
        "bot.compose.verdict:_listed" => r(opt(v::_listed(&c.s(0)))),
        "bot.compose.verdict:_cut_on" => r(json!(v::_cut_on(&c.s(0), &c.strs(1)))),
        "bot.compose.verdict:_shorten" => r(json!(v::_shorten(&c.s(0), &c.strs(1)))),
        "bot.compose.verdict:_cut_at_a_join" => r(json!(v::_cut_at_a_join(&c.s(0)))),
        "bot.compose.verdict:_first_clause" => r(json!(v::_first_clause(&c.s(0), c.b(1, true)))),
        "bot.compose.verdict:_an_offer_is_waiting" => r(json!(v::_an_offer_is_waiting(&c.s(0)))),
        "bot.compose.verdict:_after_a_comfort_line" => r(json!(v::_after_a_comfort_line(&c.mem(0)))),
        "bot.compose.verdict:_the_question_asked" => r(json!(v::_the_question_asked(&c.s(0)))),
        "bot.compose.verdict:_ends_on_a_question" => r(json!(v::_ends_on_a_question(&c.s(0)))),
        "bot.compose.verdict:_is_a_question" => r(json!(v::_is_a_question(&c.s(0)))),
        "bot.compose.verdict:_steps" => r(json!(v::the_steps(&c.s(0)))),
        "bot.compose.verdict:_doing_word" => r(json!(v::_doing_word(&c.strs(0)))),
        "bot.compose.verdict:_action_count" => r(json!(v::_action_count(&c.s(0)))),
        "bot.compose.verdict:_no_hanging_end" => r(json!(v::_no_hanging_end(&c.s(0)))),
        "bot.compose.verdict:_described_clause" => r(match v::_described_clause(&c.s(0)) {
            Some((a, b, w)) => json!([a, b, w]),
            None => J::Null,
        }),
        "bot.compose.verdict:_in_a_described_clause" => r(json!(v::_in_a_described_clause(&c.s(0), c.n(1, 0) as usize))),
        "bot.compose.verdict:_a_described_condition" => r(json!(v::_a_described_condition(&c.s(0)))),
        "bot.compose.verdict:_an_if_that_is_a_condition" => r(json!(v::_an_if_that_is_a_condition(&c.s(0)))),
        "bot.compose.verdict:_a_judgement_fence" => r(json!(v::_a_judgement_fence(&c.s(0)))),
        "bot.compose.verdict:_after_the_if" => r(json!(v::_after_the_if(&c.s(0)))),
        "bot.compose.verdict:_when" => r(json!(v::_when(&c.s(0)))),
        "bot.compose.verdict:_stated_before_the_judgement" => r(json!(v::_stated_before_the_judgement(&c.s(0)))),
        "bot.compose.verdict:_stated_referral" => {
            let (a, b) = v::_stated_referral(&c.s(0));
            r(json!([a, b]))
        }
        "bot.compose.verdict:_danger" => r(idx(v::_danger(&c.s(0)))),
        "bot.compose.verdict:_a_how_to" => r(json!(v::_a_how_to(&c.s(0)))),
        "bot.compose.verdict:_round_robin" => r(json!(v::_round_robin(&c.strs(0), &c.s(1)))),
        "bot.compose.verdict:_says_what_it_is" => r(json!(v::_says_what_it_is(&c.s(0)))),
        "bot.compose.verdict:_the_line_it_answered" => r(json!(v::_the_line_it_answered(&c.s(0), &c.mem(1)))),
        "bot.compose.verdict:_past_the_timescale" => r(json!(v::_past_the_timescale(&c.s(0), &c.mem(1)))),
        "bot.compose.verdict:_subject_words" => r(set_json(v::_subject_words(&c.s(0)))),
        "bot.compose.verdict:_the_span_further_up" => r(json!(v::_the_span_further_up(&c.s(0), &c.mem(1)))),
        "bot.compose.verdict:_stems" => r(set_json(v::_stems(&c.s(0)))),
        "bot.compose.verdict:_the_fence_is_their_own_line" => r(json!(v::_the_fence_is_their_own_line(&c.s(0), &c.mem(1)))),
        "bot.compose.verdict:_a_mood" => r(json!(v::_a_mood(&c.mem(0)))),
        "bot.compose.verdict:_about_a_death" => r(json!(v::_about_a_death(&c.mem(0)))),
        "bot.compose.verdict:_the_last_person_lines" => r(json!(v::_the_last_person_lines(&c.mem(0), c.n(1, 0) as usize))),
        "bot.compose.verdict:_a_job_to_do" => r(json!(v::_a_job_to_do(&c.s(0)))),
        "bot.compose.verdict:_the_advice_itself" => r(json!(v::_the_advice_itself(&c.s(0)))),
        "bot.compose.verdict:_an_ask_for_a_call" => r(json!(v::_an_ask_for_a_call(&c.mem(0)))),
        "bot.compose.verdict:_a_fix_with_no_warning" => r(json!(v::_a_fix_with_no_warning(&c.s(0)))),
        "bot.compose.verdict:_somebody_owes_them" => r(json!(v::_somebody_owes_them(&c.mem(0), c.n(1, 3) as usize))),
        "bot.compose.verdict:_a_reading_of_the_feeling" => r(opt(v::_a_reading_of_the_feeling(&c.mem(0), &c.s(1)))),
        "bot.compose.verdict:_care_pool" => r(json!(v::_care_pool(&c.s(0)))),
        "bot.compose.verdict:_the_knowledge_has_this" => r(opt(v::_the_knowledge_has_this(&c.s(0), &c.mem(1), &c.s(2)))),
        "bot.compose.verdict:a_check_on_the_reply" => r(json!(v::a_check_on_the_reply(&c.s(0)))),
        "bot.compose.verdict:_an_amount_asked" => r(json!(v::_an_amount_asked(&c.s(0)))),
        "bot.compose.verdict:_the_last_thing_said_is_a_nod" => r(json!(v::_the_last_thing_said_is_a_nod(&c.mem(0)))),
        "bot.compose.verdict:_a_check_on_the_method" => r(opt(v::_a_check_on_the_method(&c.s(0), &c.mem(1)))),
        "bot.compose.verdict:reply_for" => r(opt(v::reply_for(&c.s(0), &c.mem(1)))),
        "bot.compose.verdict:_the_referral_it_closed_on" => r(json!(v::_the_referral_it_closed_on(&c.s(0)))),
        "bot.compose.verdict:_the_range_it_gave" => r(json!(v::_the_range_it_gave(&c.s(0)))),
        "bot.compose.verdict:_an_instruction_and_nothing_else" => r(json!(v::_an_instruction_and_nothing_else(&c.s(0), &c.mem(1)))),
        "bot.compose.verdict:_the_floor_asked_them_something" => r(json!(v::_the_floor_asked_them_something(&c.mem(0)))),
        "bot.compose.verdict:_a_feeling_of_their_own" => r(json!(v::_a_feeling_of_their_own(&c.s(0)))),
        "bot.compose.verdict:_the_other_thing_in_the_answer" => r(json!(v::_the_other_thing_in_the_answer(&c.s(0), c.os(1).as_deref(), &c.mem(2)))),
        "bot.compose.verdict:_step_sentence" => r(json!(v::_step_sentence(&c.s(0), &c.s(1)))),
        "bot.compose.verdict:_up_to_a_break" => r(json!(v::_up_to_a_break(&c.s(0), &c.s(1), c.n(2, 0) as usize))),
        "bot.compose.verdict:_an_instruction_sentence" => r(json!(v::_an_instruction_sentence(&c.s(0)))),
        "bot.compose.verdict:_the_answer_under_a_frame" => r(json!(v::_the_answer_under_a_frame(&c.mem(0)))),
        "bot.compose.verdict:_the_front_names_what_is_in_hand" => r(json!(v::_the_front_names_what_is_in_hand(&c.s(0), &c.s(1), &c.mem(2)))),
        "bot.compose.verdict:next_step" => r(opt(v::next_step(&c.s(0), &c.mem(1)))),
        // ---- fallback_chain ----
        "bot.compose.fallback_chain:_body_state" => r(opt(fc::_body_state(&c.s(0)))),
        "bot.compose.fallback_chain:_a_part_in_a_state" => r(opt(fc::_a_part_in_a_state(&c.s(0)))),
        "bot.compose.fallback_chain:_a_part_is_cold" => r(json!(fc::_a_part_is_cold(&c.s(0)))),
        "bot.compose.fallback_chain:_state_from" => {
            let (s, t) = fc::_state_from(&c.s(0));
            r(json!([s, t]))
        }
        "bot.compose.fallback_chain:_the_state" => r(opt(fc::_the_state(&c.s(0)))),
        "bot.compose.fallback_chain:_habit_is_a_problem" => r(json!(fc::_habit_is_a_problem(&c.s(0)))),
        "bot.compose.fallback_chain:reads_as_a_flat_habit" => r(json!(fc::reads_as_a_flat_habit(&c.s(0), &c.und(1)))),
        "bot.compose.fallback_chain:_flat_habit_stated" => r(json!(fc::_flat_habit_stated(&c.s(0)))),
        "bot.compose.fallback_chain:_a_claim_about_a_thing" => r(json!(fc::_a_claim_about_a_thing(&c.s(0)))),
        "bot.compose.fallback_chain:_an_order_it_cannot_do" => r(opt_pool(fc::_an_order_it_cannot_do(&c.s(0)))),
        "bot.compose.fallback_chain:_act_asked_of_me" => r(opt_pool(fc::_act_asked_of_me(&c.s(0)))),
        "bot.compose.fallback_chain:_the_store_has_the_recall" => {
            let mut m = c.mem(1);
            let got = fc::_the_store_has_the_recall(&c.s(0), Some(&mut m));
            Some((json!(got), Some(m)))
        }
        "bot.compose.fallback_chain:_acts_for_me" => {
            let mut m = c.mem(1);
            let has = c.has_mem(1);
            let got = fc::_acts_for_me(&c.s(0), if has { Some(&mut m) } else { None });
            Some((json!(got), if has { Some(m) } else { None }))
        }
        "bot.compose.fallback_chain:_reads_as_a_bad_stretch" => r(json!(fc::_reads_as_a_bad_stretch(&c.s(0)))),
        "bot.compose.fallback_chain:_cue_negated" => r(json!(fc::_cue_negated(&c.strs(0), &c.s(1)))),
        "bot.compose.fallback_chain:_a_state_in_the_past" => r(json!(fc::_a_state_in_the_past(&c.strs(0), c.n(1, 0)))),
        "bot.compose.fallback_chain:_the_win_is_someone_elses" => r(json!(fc::_the_win_is_someone_elses(&c.strs(0), c.n(1, 0)))),
        "bot.compose.fallback_chain:reads_as_no_pleasure" => r(json!(fc::reads_as_no_pleasure(&c.s(0)))),
        "bot.compose.fallback_chain:reads_as_a_question_about_the_bot" => r(json!(fc::reads_as_a_question_about_the_bot(&c.s(0)))),
        "bot.compose.fallback_chain:_a_thing_stopping_after" => r(json!(fc::_a_thing_stopping_after(&c.strs(0), c.n(1, 0)))),
        "bot.compose.fallback_chain:_a_harm_said" => r(json!(fc::_a_harm_said(&c.s(0)))),
        "bot.compose.fallback_chain:_mood" => r(json!(fc::_mood(&c.strs(0), &c.s(1)))),
        "bot.compose.fallback_chain:_no_such_thing_before" => r(json!(fc::_no_such_thing_before(&c.strs(0), c.n(1, 0)))),
        "bot.compose.fallback_chain:_mixed_seed" => r(json!(fc::_mixed_seed(&c.s(0)) as i64)),
        "bot.compose.fallback_chain:_only_what_the_line_gives" => r(json!(fc::_only_what_the_line_gives(&c.strs(0), &c.s(1)))),
        "bot.compose.fallback_chain:_choose" => r(opt(fc::_choose_opt(&c.strs(0), &c.mem(1), &c.s(2), c.os(3).as_deref()))),
        "bot.compose.fallback_chain:_somebody_elses_day_in_thread" => r(json!(fc::_somebody_elses_day_in_thread(&c.mem(0)))),
        "bot.compose.fallback_chain:_the_person_named_a_list" => r(json!(fc::_the_person_named_a_list(&c.mem(0)))),
        "bot.compose.fallback_chain:_routine_pool" => {
            let m = c.mem(2);
            r(opt_pool(fc::_routine_pool(&c.s(0), &c.strs(1), if c.has_mem(2) { Some(&m) } else { None })))
        }
        "bot.compose.fallback_chain:_asks_the_bot" => r(json!(fc::_asks_the_bot(&c.und(0)))),
        "bot.compose.fallback_chain:_words" => r(json!(fc::_words(&c.s(0)))),
        "bot.compose.fallback_chain:_restated" => r(json!(fc::_restated(&c.und(0)))),
        "bot.compose.fallback_chain:_a_near_miss" => r(json!(fc::_a_near_miss(&c.s(0)))),
        "bot.compose.fallback_chain:_a_things_state" => r(json!(fc::_a_things_state(&c.mem(0), &c.s(1)))),
        "bot.compose.fallback_chain:_a_chat_to_correct" => r(json!(fc::_a_chat_to_correct(&c.mem(0)))),
        "bot.compose.fallback_chain:_takes_something_back" => r(json!(fc::_takes_something_back(&c.s(0), &c.mem(1)))),
        "bot.compose.fallback_chain:_a_lateness_report" => r(json!(fc::_a_lateness_report(&c.s(0), &c.s(1)))),
        "bot.compose.fallback_chain:_somebody_hurt" => r(opt(fc::_somebody_hurt(&c.s(0), &c.mem(1), &c.und(2)))),
        "bot.compose.fallback_chain:_a_low_day" => r(json!(fc::_a_low_day(&c.s(0), &c.und(1)))),
        "bot.compose.fallback_chain:_loves_people" => r(json!(fc::_loves_people(&c.s(0)))),
        "bot.compose.fallback_chain:_loves_someone" => r(json!(fc::_loves_someone(&c.s(0)))),
        "bot.compose.fallback_chain:_the_clock_pool" => r(json!(fc::_the_clock_pool(&c.s(0)))),
        "bot.compose.fallback_chain:_the_morning_has_a_clock" => r(json!(fc::_the_morning_has_a_clock(&c.s(0)))),
        "bot.compose.fallback_chain:_options_already_up" => r(json!(fc::_options_already_up(&c.mem(0)))),
        "bot.compose.fallback_chain:_a_countdown_in_a_care_thread" => {
            r(opt(fc::_a_countdown_in_a_care_thread(&c.mem(0), &c.s(1), &c.s(2), &c.und(3))))
        }
        "bot.compose.fallback_chain:reply_for" => {
            let mut m = c.mem(1);
            let got = fc::reply_for(&c.s(0), &mut m, c.b(2, true));
            Some((json!(got), Some(m)))
        }
        "bot.compose.fallback_chain:_reply_for" => {
            let mut m = c.mem(1);
            let u = c.und(2);
            let got = fc::_reply_for(&c.s(0), &mut m, if c.has_und(2) { Some(&u) } else { None }, c.b(3, true));
            Some((opt(got), Some(m)))
        }
        "bot.compose.fallback_chain:_a_betrayal" => r(json!(fc::_a_betrayal(&c.s(0)))),
        "bot.compose.fallback_chain:_a_thing_they_regret" => r(json!(fc::_a_thing_they_regret(&c.s(0)))),
        "bot.compose.fallback_chain:_a_care_home_line" => {
            let u = c.und(0);
            r(opt(fc::_a_care_home_line(if c.has_und(0) { Some(&u) } else { None }, &c.mem(1))))
        }
        "bot.compose.fallback_chain:_a_day_told_instead_of_the_receipt" => {
            r(opt(fc::_a_day_told_instead_of_the_receipt(c.os(0).as_deref(), &c.s(1), &c.mem(2))))
        }
        "bot.compose.fallback_chain:_the_crowd_is_ahead" => r(json!(fc::_the_crowd_is_ahead(&c.s(0)))),
        "bot.compose.fallback_chain:_the_rest_of_the_day" => {
            let mood = c.os(2).unwrap_or_else(|| "neutral".into());
            r(opt_pool(fc::_the_rest_of_the_day(&c.s(0), &c.mem(1), &mood)))
        }
        "bot.compose.fallback_chain:_an_emotion" => r(opt(fc::_an_emotion(&c.s(0)))),
        "bot.compose.fallback_chain:_just_asked_about_the_day" => r(json!(fc::_just_asked_about_the_day(&c.mem(0)))),
        "bot.compose.fallback_chain:_a_day_of_someone_else" => r(json!(fc::_a_day_of_someone_else(&c.s(0)))),
        "bot.compose.fallback_chain:_a_thing_happened" => r(json!(fc::_a_thing_happened(&c.s(0)))),
        "bot.compose.fallback_chain:_a_thing_got_ahead" => r(opt_pool(fc::_a_thing_got_ahead(&c.s(0)))),
        "bot.compose.fallback_chain:_a_problem_stated" => r(json!(fc::_a_problem_stated(&c.s(0), c.os(1).as_deref()))),
        "bot.compose.fallback_chain:_says_there_is_nothing_to_wear" => r(json!(fc::_says_there_is_nothing_to_wear(&c.s(0), &c.s(1), &c.s(2)))),
        "bot.compose.fallback_chain:_a_wear_ask_about_a_day" => r(json!(fc::_a_wear_ask_about_a_day(&c.s(0), &c.s(1), &c.s(2)))),
        "bot.compose.fallback_chain:_the_wear_pool_for_the_line" => r(json!(fc::_the_wear_pool_for_the_line(&c.s(0), &c.s(1), &c.s(2)))),
        "bot.compose.fallback_chain:_a_small_day" => r(json!(fc::_a_small_day(&c.s(0), &c.mem(1)))),
        "bot.compose.fallback_chain:_a_detail_about_what_was_said" => r(json!(fc::_a_detail_about_what_was_said(&c.s(0), &c.mem(1)))),
        "bot.compose.fallback_chain:_a_dish_name" => r(json!(fc::_a_dish_name(&c.s(0), &c.strs(1), &c.mem(2)))),
        "bot.compose.fallback_chain:_a_report_with_no_pronoun" => {
            let mut m = c.mem(2);
            let got = fc::_a_report_with_no_pronoun(&c.s(0), &c.s(1), &mut m);
            Some((json!(got), Some(m)))
        }
        "bot.compose.fallback_chain:_repaired_of" => r(json!(fc::_repaired_of(&c.s(0), &c.mem(1)))),
        "bot.compose.fallback_chain:takes_as_it_comes" => {
            let mut m = c.mem(1);
            let got = fc::takes_as_it_comes(&c.s(0), &mut m);
            Some((json!(got), Some(m)))
        }
        "bot.compose.fallback_chain:_a_name_told" => r(match fc::_a_name_told(&c.s(0)) {
            Some((a, b, p)) => json!([a, b, p]),
            None => J::Null,
        }),
        "bot.compose.fallback_chain:_named_after_reply" => r(opt(fc::_named_after_reply(&c.s(0), &c.mem(1)))),
        "bot.compose.fallback_chain:_a_name_told_reply" => r(opt(fc::_a_name_told_reply(&c.s(0), &c.mem(1)))),
        "bot.compose.fallback_chain:_a_state_of_theirs" => r(opt(fc::_a_state_of_theirs(&c.s(0)))),
        "bot.compose.fallback_chain:_a_day_said_badly" => r(json!(fc::_a_day_said_badly(&c.s(0)))),
        "bot.compose.fallback_chain:_the_last_exchange" => {
            let m = c.mem(0);
            r(fc::_the_last_exchange(&m).map(|t| json!({"user": t.user, "bot": t.bot, "module": t.module})).unwrap_or(J::Null))
        }
        "bot.compose.fallback_chain:_the_day_was_already_bad" => r(json!(fc::_the_day_was_already_bad(&c.mem(0)))),
        "bot.compose.fallback_chain:_a_detail_of_a_bad_day" => r(json!(fc::_a_detail_of_a_bad_day(&c.mem(0), c.os(1).as_deref()))),
        "bot.compose.fallback_chain:_money_came_to_them" => r(json!(fc::_money_came_to_them(&c.s(0), &c.strs(1)))),
        "bot.compose.fallback_chain:_a_sum_with_nothing_beside_it" => {
            let m = c.mem(1);
            r(opt(fc::_a_sum_with_nothing_beside_it(&c.s(0), if c.has_mem(1) { Some(&m) } else { None })))
        }
        "bot.compose.fallback_chain:_a_report_from_elsewhere" => r(opt_pool(fc::_a_report_from_elsewhere(&c.s(0)))),
        "bot.compose.fallback_chain:_a_small_day_shape" => r(json!(fc::_a_small_day_shape(&c.s(0)))),
        "bot.compose.fallback_chain:_a_bad_time_told" => r(json!(fc::_a_bad_time_told(&c.s(0), &c.mem(1)))),
        "bot.compose.fallback_chain:_a_day_asked_without_a_cheer" => r(json!(fc::_a_day_asked_without_a_cheer(&c.s(0), &c.mem(1)))),
        _ => None,
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("usage: com2_check CALLS.jsonl.gz [--fn SUBSTRING] [--show K]");
    let (mut only, mut show) = (String::new(), 3usize);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--fn" => only = args.next().unwrap_or_default(),
            "--show" => show = args.next().and_then(|x| x.parse().ok()).unwrap_or(3),
            _ => {}
        }
    }
    quiet_panics();
    let reader = std::io::BufReader::new(GzDecoder::new(std::fs::File::open(&path).expect("calls file")));
    let mut mems: HashMap<String, Map<String, J>> = HashMap::new();
    let mut calls: Vec<J> = Vec::new();
    for line in reader.lines() {
        let Ok(line) = line else { break };
        let d: J = serde_json::from_str(&line).unwrap();
        if let Some(h) = d.get("memory").and_then(|x| x.as_str()) {
            mems.insert(h.to_string(), d["plain"].as_object().cloned().unwrap_or_default());
        } else {
            calls.push(d);
        }
    }
    #[derive(Default)]
    struct Tally {
        ok: usize,
        bad: usize,
        skipped: usize,
        unported: BTreeMap<String, usize>,
        shown: Vec<String>,
    }
    let mut tally: BTreeMap<String, Tally> = BTreeMap::new();
    for d in &calls {
        let name = d["fn"].as_str().unwrap_or("").replace(":TAKES_AS_IT_COMES", ":takes_as_it_comes");
        if !only.is_empty() && !name.contains(&only) {
            continue;
        }
        let params: Vec<String> = d["params"].as_array().map(|a| a.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect()).unwrap_or_default();
        let pos = d["args"].as_array().cloned().unwrap_or_default();
        let kw = d["kwargs"].as_object().cloned().unwrap_or_default();
        let mut argv: Vec<Option<&J>> = Vec::new();
        let n = params.len().max(pos.len());
        let pos_ref = d["args"].as_array().unwrap();
        for i in 0..n {
            if i < pos.len() {
                argv.push(Some(&pos_ref[i]));
            } else if let Some(p) = params.get(i) {
                argv.push(d["kwargs"].as_object().and_then(|o| o.get(p)));
            } else {
                argv.push(None);
            }
        }
        let _ = kw;
        let t = tally.entry(name.clone()).or_default();
        if pos.iter().chain(d["kwargs"].as_object().map(|o| o.values().collect::<Vec<_>>()).unwrap_or_default().into_iter().map(|x| &*x).collect::<Vec<_>>().into_iter()).any(|a| {
            a.as_object().is_some_and(|o| o.contains_key("$callable") || o.contains_key("$opaque"))
        }) || d["raised"].is_string()
        {
            t.skipped += 1;
            continue;
        }
        let call = Call { args: argv, mems: &mems };
        let _ = take_last_panic();
        let got = guarded(|| run(&name, &call));
        match got {
            Some(Some((result, mem_after))) => {
                let want = &d["result"];
                let mut same = if d["type"] == "set" { sorted(want.clone()) == sorted(result.clone()) } else { norm(want) == norm(&result) };
                // Python's None and "" are both "no line" for the readers that return text.
                if !same && (want.is_null() && result == json!("") || want == &json!("") && result.is_null()) {
                    same = true;
                }
                let mut mem_note = String::new();
                if same {
                    if let (Some(m), Some(h)) = (mem_after, d.get("mem_after").and_then(|x| x.as_str())) {
                        if let Some(py) = mems.get(h) {
                            let ours = J::Object(m.to_plain());
                            if norm(&J::Object(py.clone())) != norm(&ours) {
                                same = false;
                                mem_note = " (memory after differs)".into();
                            }
                        }
                    }
                }
                if same {
                    t.ok += 1;
                } else {
                    t.bad += 1;
                    if t.shown.len() < show {
                        t.shown.push(format!("  args {}\n    python {}\n    rust   {}{}", J::Array(pos.clone()).to_string().chars().take(400).collect::<String>(), want.to_string().chars().take(300).collect::<String>(), result.to_string().chars().take(300).collect::<String>(), mem_note));
                    }
                }
            }
            Some(None) => t.skipped += 1,
            None => {
                let msg = take_last_panic().unwrap_or_default();
                if let Some(stub) = msg.strip_prefix("UNPORTED ") {
                    *t.unported.entry(stub.to_string()).or_default() += 1;
                } else {
                    t.bad += 1;
                    if t.shown.len() < show {
                        t.shown.push(format!("  args {}\n    panic {}", J::Array(pos.clone()).to_string().chars().take(400).collect::<String>(), msg));
                    }
                }
            }
        }
    }
    let (mut ok, mut bad, mut unp) = (0, 0, 0);
    for (name, t) in &tally {
        let u: usize = t.unported.values().sum();
        ok += t.ok;
        bad += t.bad;
        unp += u;
        println!("{name}: ok {} bad {} unported {} skipped {}", t.ok, t.bad, u, t.skipped);
        if u > 0 {
            let top: Vec<String> = t.unported.iter().map(|(k, v)| format!("{k} x{v}")).take(4).collect();
            println!("    stubs: {}", top.join(", "));
        }
        for s in &t.shown {
            println!("{s}");
        }
    }
    println!("TOTAL ok {ok} bad {bad} unported {unp}");
}
