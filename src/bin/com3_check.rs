#![allow(dead_code, unused_imports)]
//! Replay the calls `harness/com3_calls.py` recorded against the Rust port of the COM3 files
//! (13 files of bot/compose).
//!
//!     com3_check CALLS.jsonl.gz [--fn SUBSTRING] [--show K]
//!
//! Per function: calls that give PyBot's result (and, for a function that writes memory, PyBot's
//! memory after it), calls that differ, and calls that reached a stub not ported yet.
use flate2::read::GzDecoder;
use rsbot::compose::{reading as rd, remark as rm, goodnews as gn, check_with_you as cwy, checking as ck, fallback_pools as fp, finding_out as fo, loss, onsubject as os, quantity as qt, ruledout as ro, trouble as tr, what_i_do_know as wk};
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


fn optj<T: serde::Serialize>(v: Option<T>) -> J {
    v.map(|x| json!(x)).unwrap_or(J::Null)
}
fn words_arg(c: &Call, i: usize) -> Option<Vec<String>> {
    match c.get(i) {
        Some(J::Array(_)) => Some(c.strs(i)),
        _ => None,
    }
}

/// Run one recorded call. None when the function has no replay here.
fn run(name: &str, c: &Call) -> Option<Out> {
    let r = |j: J| Some((j, None));
    let und_opt = |i: usize| if c.has_und(i) { Some(c.und(i)) } else { None };
    match name {
        // ---- goodnews ----
        "bot.compose.goodnews:reads_as_good_news" => r(json!(gn::reads_as_good_news(&c.s(0)))),
        "bot.compose.goodnews:reads_as_a_pass" => r(json!(gn::reads_as_a_pass(&c.s(0)))),
        "bot.compose.goodnews:reads_as_news_to_come" => r(json!(gn::reads_as_news_to_come(&c.s(0)))),
        "bot.compose.goodnews:reads_as_an_occasion" => r(optj(gn::reads_as_an_occasion(&c.s(0)))),
        "bot.compose.goodnews:reads_as_recovery" => r(json!(gn::reads_as_recovery(&c.s(0)))),
        "bot.compose.goodnews:reads_as_settled" => r(json!(gn::reads_as_settled(&c.s(0)))),
        "bot.compose.goodnews:reads_as_a_clear_result" => r(json!(gn::reads_as_a_clear_result(&c.s(0)))),
        "bot.compose.goodnews:reads_as_a_win_nobody_noticed" => r(json!(gn::reads_as_a_win_nobody_noticed(&c.s(0)))),
        "bot.compose.goodnews:reads_as_a_win_a_cost_came_with" => r(json!(gn::reads_as_a_win_a_cost_came_with(&c.s(0)))),
        "bot.compose.goodnews:reads_as_a_prize_in_a_message" => r(json!(gn::reads_as_a_prize_in_a_message(&c.s(0)))),
        "bot.compose.goodnews:asks_for_news" => r(json!(gn::asks_for_news(&c.s(0)))),
        "bot.compose.goodnews:new_thing" => r(gn::new_thing(&c.s(0)).map(|v| v.to_plain()).unwrap_or(J::Null)),
        "bot.compose.goodnews:a_detail_of_the_news" => r(json!(if c.has_mem(1) { gn::a_detail_of_the_news(&c.s(0), &c.mem(1)) } else { gn::a_detail_of_the_news_in(&c.s(0), None) })),
        "bot.compose.goodnews:reads_as_a_detail_of_the_news" => r(json!(gn::reads_as_a_detail_of_the_news(&c.s(0), &c.mem(1)))),
        "bot.compose.goodnews:reply_for_a_year_ago" => r(optj(gn::reply_for_a_year_ago(&c.s(0), &c.s(1), &c.mem(2)))),
        "bot.compose.goodnews:the_cheer_was_this_pool" => r(json!(gn::the_cheer_was_this_pool(&c.mem(0)))),
        "bot.compose.goodnews:the_reply_just_cheered" => r(json!(gn::the_reply_just_cheered(&c.mem(0)))),
        "bot.compose.goodnews:reads_as_a_congratulation" => r(json!(gn::reads_as_a_congratulation(&c.s(0)))),
        "bot.compose.goodnews:_a_hand_over" => r(json!(gn::_a_hand_over(&c.strs(0)))),
        "bot.compose.goodnews:worry_on_the_table" => r(json!(gn::worry_on_the_table(&c.mem(0), &rsbot::data::Value::Int(c.n(1, 4))))),
        "bot.compose.goodnews:a_year_behind" => r(json!(gn::a_year_behind(&c.s(0), None))),
        // ---- remark ----
        "bot.compose.remark:reads_as_regret" => r(json!(rm::reads_as_regret(&c.s(0), &c.strs(1)))),
        "bot.compose.remark:regret_reply_for" => r(optj(rm::regret_reply_for(&c.s(0), &c.strs(1), &c.mem(2), &c.s(3)))),
        "bot.compose.remark:reads_as_report" => r(optj(rm::reads_as_report(&c.s(0), &c.strs(1)))),
        "bot.compose.remark:report_reply_for" => r(optj(rm::report_reply_for(&c.s(0), &c.strs(1), &c.mem(2)))),
        "bot.compose.remark:break_in_reply_for" => r(optj(rm::break_in_reply_for(&c.s(0), &c.strs(1), &c.mem(2)))),
        "bot.compose.remark:reply_for" => r(optj(rm::reply_for(&c.s(0), &c.strs(1), &c.mem(2), &c.strs(3)))),
        "bot.compose.remark:arranged_reply_for" => r(optj(rm::arranged_reply_for(&c.s(0), &c.strs(1), &c.mem(2)))),
        "bot.compose.remark:reads_as_arranged" => r(json!(rm::reads_as_arranged(&c.s(0)))),
        "bot.compose.remark:reads_as_plan" => r(json!(rm::reads_as_plan(&c.s(0)))),
        "bot.compose.remark:the_two_of_them" => r(optj(rm::the_two_of_them(&c.s(0), &c.mem(1)))),
        "bot.compose.remark:their_people_reply_for" => r(optj(rm::their_people_reply_for(&c.s(0), &c.strs(1), &c.mem(2)))),
        "bot.compose.remark:their_mood_going_on_reply_for" => r(optj(rm::their_mood_going_on_reply_for(&c.s(0), &c.mem(1)))),
        "bot.compose.remark:their_mood_with_you_reply_for" => {
            let mut m = c.mem(2);
            let out = rm::their_mood_with_you_reply_for(&c.s(0), &c.strs(1), &mut m);
            Some((optj(out), Some(m)))
        }
        "bot.compose.remark:a_want_said_back" => r(json!(rm::a_want_said_back(c.os(0).as_deref(), &c.s(1)))),
        "bot.compose.remark:dependant_want_reply_for" => r(optj(rm::dependant_want_reply_for(&c.s(0), &c.strs(1), &c.mem(2)))),
        "bot.compose.remark:a_thing_of_theirs_reply_for" => r(optj(rm::a_thing_of_theirs_reply_for(&c.s(0), &c.strs(1), &c.mem(2)))),
        "bot.compose.remark:done_to_me" => r(json!(rm::done_to_me(&c.s(0)))),
        "bot.compose.remark:_has_phrase" => r(json!(rm::_has_phrase(&c.s(0), &c.strs(1)))),
        "bot.compose.remark:interest_reply_for" => r(optj(rm::interest_reply_for(&c.get(0).map(rsbot::data::Value::from_plain).unwrap_or(rsbot::data::Value::None), &c.mem(1)))),
        // ---- reading ----
        "bot.compose.reading:reply_for" => r(optj(rd::reply_for(&c.s(0), &c.mem(1)))),
        "bot.compose.reading:reply_to_a_doubt" => r(optj(rd::reply_to_a_doubt(&c.s(0), &c.mem(1)))),
        "bot.compose.reading:reply_to_a_bare_doubt" => r(optj(rd::reply_to_a_bare_doubt(&c.s(0), &c.mem(1)))),
        "bot.compose.reading:judges_a_figure" => r(json!(rd::judges_a_figure(&c.s(0), &c.mem(1)))),
        "bot.compose.reading:judges_the_figure_on_the_table" => r(json!(rd::judges_the_figure_on_the_table(&c.s(0), &c.mem(1)))),
        "bot.compose.reading:_the_verdict_for_the_table" => r(optj(rd::_the_verdict_for_the_table(&c.s(0), &c.mem(1)))),
        "bot.compose.reading:_its_own_subject_judges_it" => r(json!(rd::_its_own_subject_judges_it(&c.s(0), &c.mem(1)))),
        "bot.compose.reading:_a_bare_doubt" => r(rd::_a_bare_doubt(&c.s(0), &c.mem(1)).map(|(v, u, t, m)| json!([v, u, t, m])).unwrap_or(J::Null)),
        "bot.compose.reading:_doubt" => r(rd::_doubt(&c.s(0)).map(|d| json!({"word": d.word, "value": d.value, "money": d.money, "unit": d.unit, "subject": d.subject, "rate": d.rate})).unwrap_or(J::Null)),
        "bot.compose.reading:_number" => r(rd::_number(&c.s(0)).map(|(v, u)| json!([v, u])).unwrap_or(J::Null)),
        "bot.compose.reading:_their_figure" => r(rd::_their_figure(&c.mem(0), &rsbot::data::Value::Int(c.n(1, 1))).to_plain()),
        "bot.compose.reading:_the_figure_in_the_doubt" => { let (v, m, u, rest, rate) = rd::_the_figure_in_the_doubt(&c.s(0)); r(json!([v, m, u, rest, rate])) }
        // ---- loss ----
        "bot.compose.loss:reads_as_loss" => { let w = words_arg(c, 1); r(json!(loss::reads_as_loss(&c.s(0), w.as_deref(), c.b(2, false)))) }
        "bot.compose.loss:reply_for" => r(opt_pool(loss::reply_for(&c.s(0), &c.mem(1), &c.s(2), &c.strs(3), c.os(4).as_deref()))),
        "bot.compose.loss:continues" => r(json!(loss::continues(&c.mem(0)))),
        "bot.compose.loss:is_practical_ask" => r(json!(loss::is_practical_ask(&c.s(0), c.os(1).as_deref()))),
        "bot.compose.loss:asks_anything" => r(json!(loss::asks_anything(&c.s(0), c.os(1).as_deref()))),
        "bot.compose.loss:arrangement_cost_pool" => r(opt_pool(loss::arrangement_cost_pool(&c.s(0), c.os(1).as_deref(), &c.mem(2)))),
        "bot.compose.loss:lost_in_a_place" => r(json!(loss::lost_in_a_place(&c.s(0)))),
        "bot.compose.loss:lost_thing_to_find" => r(json!(loss::lost_thing_to_find(&c.s(0)))),
        "bot.compose.loss:keeps_losing_thing" => r(json!(loss::keeps_losing_thing(&c.s(0)))),
        "bot.compose.loss:locked_out" => r(json!(loss::locked_out(&c.s(0)))),
        "bot.compose.loss:found_what_was_lost" => r(json!(loss::found_what_was_lost(&c.s(0)))),
        "bot.compose.loss:marks_a_day" => r(json!(loss::marks_a_day(&c.s(0), c.b(1, false)))),
        "bot.compose.loss:says_the_last_drug" => r(json!(loss::says_the_last_drug(&c.s(0)))),
        "bot.compose.loss:words_of" => r(json!(loss::words_of(&c.s(0)))),
        "bot.compose.loss:_plain" => r(json!(loss::_plain(&c.s(0)))),
        "bot.compose.loss:_tight" => r(json!(loss::_tight(&c.s(0)))),
        "bot.compose.loss:_spoken" => r(json!(loss::_spoken(&c.s(0)))),
        "bot.compose.loss:_the_end_is_near" => { let w = words_arg(c, 1); r(json!(loss::_the_end_is_near(&c.s(0), w.as_deref()))) }
        "bot.compose.loss:_the_family_is_told" => r(json!(loss::_the_family_is_told(&c.s(0)))),
        "bot.compose.loss:_missing_someone" => r(json!(loss::_missing_someone(&c.s(0)))),
        "bot.compose.loss:_loss_is_the_thread" => r(json!(loss::_loss_is_the_thread(&c.mem(0)))),
        "bot.compose.loss:_an_ending_in_the_chat" => r(json!(loss::_an_ending_in_the_chat(&c.mem(0)))),
        "bot.compose.loss:_keeping_going" => r(opt_pool(loss::_keeping_going(&c.s(0), &c.strs(1), &c.mem(2)))),
        "bot.compose.loss:_a_swap_of_the_question_before" => r(json!(loss::_a_swap_of_the_question_before(&c.s(0), &c.mem(1)))),
        "bot.compose.loss:_names_a_stored_pet" => r(json!(loss::_names_a_stored_pet(&c.s(0), &c.mem(1)))),
        "bot.compose.loss:_has_person" => r(json!(loss::_has_person(&c.strs(0)))),
        "bot.compose.loss:_unnamed_person" => r(json!(loss::_unnamed_person(&c.strs(0), &c.s(1)))),
        "bot.compose.loss:_a_thing_of_theirs" => r(json!(loss::_a_thing_of_theirs(&c.strs(0)))),
        "bot.compose.loss:_only_the_proceedings" => r(json!(loss::_only_the_proceedings(&c.s(0), &c.strs(1)))),
        "bot.compose.loss:_accident_death" => r(json!(loss::_accident_death(&c.s(0)))),
        "bot.compose.loss:_passed_on" => r(json!(loss::_passed_on(&c.s(0), c.b(1, false)))),
        "bot.compose.loss:_passed_alone" => r(json!(loss::_passed_alone(&c.s(0), c.b(1, false)))),
        "bot.compose.loss:_death_noun" => r(json!(loss::_death_noun(&c.s(0), &c.strs(1), c.b(2, false)))),
        "bot.compose.loss:_buried_person" => r(json!(loss::_buried_person(&c.s(0), &c.strs(1), c.b(2, false)))),
        "bot.compose.loss:_put_down" => r(json!(loss::_put_down(&c.s(0), &c.strs(1)))),
        "bot.compose.loss:_where_i_put_it" => r(json!(loss::_where_i_put_it(&c.s(0)))),
        "bot.compose.loss:_readable" => r(json!(loss::_readable(&c.s(0)))),
        "bot.compose.loss:_only_a_new_subject" => r(json!(loss::_only_a_new_subject(&c.s(0)))),
        "bot.compose.loss:_telling_a_child" => r(json!(loss::_telling_a_child(&c.s(0)))),
        "bot.compose.loss:_the_visit_ask" => r(json!(loss::_the_visit_ask(&c.s(0)))),
        "bot.compose.loss:_a_gift_for_someone_in_care" => r(json!(loss::_a_gift_for_someone_in_care(&c.s(0)))),
        "bot.compose.loss:_a_colour_is_worn" => r(json!(loss::_a_colour_is_worn(&c.s(0)))),
        "bot.compose.loss:_brought_in_prey" => r(json!(loss::_brought_in_prey(&c.s(0), &c.strs(1)))),
        "bot.compose.loss:_a_pronoun_is_what_died" => r(json!(loss::_a_pronoun_is_what_died(&c.strs(0), &c.s(1)))),
        "bot.compose.loss:_the_give_up_without_a_care_word" => r(json!(loss::_the_give_up_without_a_care_word(&c.s(0), c.b(1, false)))),
        "bot.compose.loss:is_dying_now" => r(json!(loss::is_dying_now(&c.s(0)))),
        "bot.compose.loss:whose" => r(json!(loss::whose(&c.s(0)))),
        "bot.compose.loss:the_animal" => r(json!(loss::the_animal(&c.s(0)))),
        "bot.compose.loss:a_pet_died" => r(json!(loss::a_pet_died(&c.s(0)))),
        // ---- trouble ----
        "bot.compose.trouble:reads_as_trouble" => { let w = words_arg(c, 1); r(optj(tr::reads_as_trouble(&c.s(0), w.as_deref(), &c.s(2)))) }
        "bot.compose.trouble:_shape" => { let w = words_arg(c, 1); r(optj(tr::_shape(&c.s(0), w.as_deref()))) }
        "bot.compose.trouble:carried_on" => {
            let w = words_arg(c, 1);
            r(opt_pool(tr::carried_on_with(&c.s(0), w.as_deref(), &c.mem(2))))
        }
        "bot.compose.trouble:someone_elses" => r(json!(tr::someone_elses(&c.strs(0)))),
        "bot.compose.trouble:_keeps_doing_it" => r(json!(tr::_keeps_doing_it(&c.s(0)))),
        "bot.compose.trouble:_the_fault_hangs_on" => r(sorted(json!(tr::_the_fault_hangs_on(&c.s(0))))),
        "bot.compose.trouble:_a_fault_of_a_thing" => r(json!(tr::_a_fault_of_a_thing(&c.s(0)))),
        "bot.compose.trouble:_without_a_dead_that_only_means_very" => r(json!(tr::_without_a_dead_that_only_means_very(&c.s(0)))),
        "bot.compose.trouble:_without_wanting_or_praise" => r(json!(tr::_without_wanting_or_praise(&c.s(0)))),
        "bot.compose.trouble:_shape_on_the_table" => r(optj(tr::_shape_on_the_table(&c.mem(0)))),
        "bot.compose.trouble:_shape_answered" => r(optj(tr::_shape_answered(&c.mem(0)))),
        "bot.compose.trouble:answers_a_strain_question" => r(json!(tr::answers_a_strain_question(&c.s(0), &c.mem(1)))),
        "bot.compose.trouble:_flat" => r(json!(tr::_flat(&c.s(0)))),
        // ---- check_with_you ----
        "bot.compose.check_with_you:reads_as_a_check" => r(json!(cwy::reads_as_a_check(&c.s(0)))),
        // ---- what_i_do_know ----
        "bot.compose.what_i_do_know:_kind_of" => r(json!(wk::_kind_of(&c.s(0)))),
        "bot.compose.what_i_do_know:_parts_of" => r(json!(wk::_parts_of(&c.s(0)))),
        "bot.compose.what_i_do_know:_a_thing_from" => r(json!(wk::_a_thing_from(&c.s(0), c.b(1, false)))),
        "bot.compose.what_i_do_know:_thing_named_in" => r(json!(wk::_thing_named_in(&c.s(0)))),
        "bot.compose.what_i_do_know:_without_a_particle" => r(json!(wk::_without_a_particle(&c.s(0)))),
        "bot.compose.what_i_do_know:_the_thing" => r(json!(wk::_the_thing(&c.und(0), &c.s(1), &c.mem(2)))),
        "bot.compose.what_i_do_know:before_text_from" => r(json!(wk::before_text_from(&c.mem(0)))),
        "bot.compose.what_i_do_know:_kind_from_the_owning_line" => r(json!(wk::_kind_from_the_owning_line(&c.mem(0), &c.s(1), &c.s(2)))),
        "bot.compose.what_i_do_know:_known" => r(json!(wk::_known(&c.s(0), &c.s(1)))),
        "bot.compose.what_i_do_know:_that" => r(json!(wk::_that(&c.s(0)))),
        "bot.compose.what_i_do_know:reply_for" => r(optj(wk::reply_for(&c.s(0), &c.mem(1), &c.s(2)))),
        // ---- ruledout ----
        "bot.compose.ruledout:_ruled_out_thing" => { let (t, k) = ro::_ruled_out_thing(&c.s(0)); r(json!([t, k])) }
        "bot.compose.ruledout:_rules_out_what_i_never_said" => { let (t, k, o) = ro::_rules_out_what_i_never_said(&c.s(0), &c.mem(1)); r(json!([t, k, o])) }
        "bot.compose.ruledout:_a_claim_of_failure" => r(json!(ro::_a_claim_of_failure(&c.s(0), &c.mem(1)))),
        "bot.compose.ruledout:_without_the_manners" => r(json!(ro::_without_the_manners(&c.s(0)))),
        "bot.compose.ruledout:reply_for" => r(json!(ro::reply_for(&c.s(0), &c.mem(1)))),
        "bot.compose.ruledout:_ruled_out_reply" => r(json!(ro::_ruled_out_reply(&c.mem(0), &c.s(1)))),
        // ---- onsubject ----
        "bot.compose.onsubject:_bare_follow_up" => r(optj(os::_bare_follow_up(&c.s(0)))),
        "bot.compose.onsubject:_words" => r(json!(os::_words(&c.s(0)))),
        "bot.compose.onsubject:_stem" => r(json!(os::_stem(&c.s(0)))),
        "bot.compose.onsubject:_named" => r(json!(os::_named(&c.s(0), c.b(1, false), c.b(2, true)).into_iter().map(|(a, b)| json!([a, b])).collect::<Vec<_>>())),
        "bot.compose.onsubject:_the_subject" => { let (a, b, l) = os::_the_subject(&c.mem(0), &c.s(1)); r(json!([a, b, l])) }
        "bot.compose.onsubject:_shape" => r(json!(os::_shape(&c.s(0)))),
        "bot.compose.onsubject:_the_place" => r(optj(os::_the_place(&c.s(0)))),
        "bot.compose.onsubject:pool" => r(optj(os::pool(&c.s(0), &c.und(1), &c.mem(2)))),
        // ---- finding_out ----
        "bot.compose.finding_out:for_the_ask" => r(opt_pool(fo::for_the_ask_with(&c.s(0), &c.s(1), und_opt(2).as_ref()))),
        "bot.compose.finding_out:the_wrecked_form_reply" => r(opt_pool(fo::the_wrecked_form_reply(&c.s(0)))),
        "bot.compose.finding_out:_has" => r(json!(fo::_has(&c.s(0), &c.strs(1)))),
        "bot.compose.finding_out:_names_one_of" => r(json!(fo::_names_one_of(&c.s(0), &c.strs(1)))),
        "bot.compose.finding_out:_the_trade" => r(json!(fo::_the_trade(&c.s(0)))),
        "bot.compose.finding_out:_the_thing" => r(json!(fo::_the_thing(&c.s(0)))),
        "bot.compose.finding_out:_the_service" => r(fo::_the_service(&c.s(0), &c.s(1)).map(|(a, b)| json!([a, b])).unwrap_or(J::Null)),
        "bot.compose.finding_out:_the_picked_thing" => r(json!(fo::_the_picked_thing(&c.s(0)))),
        // ---- checking ----
        "bot.compose.checking:answer_for" => r(optj(ck::answer_for(&c.s(0), &c.mem(1)))),
        "bot.compose.checking:the_checks_already_named" => r(optj(ck::the_checks_already_named(&c.s(0), &c.mem(1)))),
        "bot.compose.checking:_about_a_journey" => r(optj(ck::_about_a_journey(&c.s(0)))),
        "bot.compose.checking:_about_a_shop" => r(json!(ck::_about_a_shop(&c.s(0)))),
        "bot.compose.checking:_named_service" => { let (a, b) = ck::_named_service(&c.s(0)); r(json!([a, b])) }
        "bot.compose.checking:_a_day_out" => r(json!(ck::_a_day_out(&c.s(0), &c.s(1)))),
        "bot.compose.checking:_about_a_pick" => r(optj(ck::_about_a_pick(&c.s(0), &c.s(1)))),
        "bot.compose.checking:_the_line_before" => r(json!(ck::_the_line_before(&c.mem(0)))),
        "bot.compose.checking:_the_checks_in" => r(json!(ck::_the_checks_in(&c.s(0)))),
        "bot.compose.checking:_a_named_country_in" => r(json!(ck::_a_named_country_in(&c.s(0)))),
        // ---- quantity ----
        "bot.compose.quantity:reply_for" => r(optj(qt::reply_for(&c.s(0), &c.mem(1)))),
        "bot.compose.quantity:the_rate_they_asked_about" => r(optj(qt::the_rate_they_asked_about(&c.s(0), &c.mem(1)))),
        "bot.compose.quantity:the_figure_against_the_answer" => r(optj(qt::the_figure_against_the_answer(&c.s(0), &c.mem(1)))),
        "bot.compose.quantity:_read" => r(match qt::_read(&c.s(0), &c.mem(1)) {
            Some(x) => json!([x.value, x.family, x.unit, x.period, x.phrase, x.tail]),
            None => json!([null, null, null, null, null, null]),
        }),
        "bot.compose.quantity:_amount" => r(optj(qt::_amount(&c.s(0)))),
        // ---- fallback_pools ----
        "bot.compose.fallback_pools:_unknown" => r(json!(fp::_unknown(&c.und(0), &c.mem(1), &c.s(2)))),
        "bot.compose.fallback_pools:_the_view_asked_for" => r(optj(fp::_the_view_asked_for(&c.s(0), &c.mem(1)))),
        "bot.compose.fallback_pools:_a_missed_step" => r(opt_pool(fp::_a_missed_step(&c.s(0), &c.s(1), &c.mem(2), &c.strs(3)))),
        "bot.compose.fallback_pools:_the_rule_for_it_for_the_turn" => r(opt_pool(fp::_the_rule_for_it_for_the_turn(&c.s(0), &c.s(1), &c.mem(2)))),
        "bot.compose.fallback_pools:_a_care_thread" => r(json!(fp::_a_care_thread(&c.mem(0), c.n(1, 3)))),
        "bot.compose.fallback_pools:_the_repeat_answer" => r(json!(fp::_the_repeat_answer(&c.mem(0)))),
        _ => None,
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("usage: com3_check CALLS.jsonl.gz [--fn SUBSTRING] [--show K]");
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
