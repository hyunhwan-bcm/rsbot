//! Check the MEM5 ports (bot/memory: detail, enough, pile, relations, saving, tried, trips, when)
//! function by function against the calls PyBot made (`harness/mem5_record.py` output).
//!
//!     RSBOT_TODAY=2026-11-17 mem5_check CALLS.jsonl.gz [--fn SUBSTR] [--show K]
//!
//! Per function: calls that give PyBot's result (and, for a function that writes memory, the
//! same memory after it), calls that differ, calls that reached a function another workstream
//! has not ported (UNPORTED), calls whose arguments the checker does not rebuild, and functions
//! with no entry here. A Python None and a Rust "" for a str contract are counted the same.
use rsbot::data::Value;
use rsbot::memory::{detail, enough, pile, relations as rel, saving as sv, tried, trips, when};
use rsbot::state::Memory;
use serde_json::{json, Value as J};
use std::collections::{BTreeMap, HashSet};
use std::io::BufRead;

struct Args<'a>(&'a [J]);

impl<'a> Args<'a> {
    fn s(&self, i: usize) -> String {
        match self.0.get(i) {
            Some(J::String(s)) => s.clone(),
            _ => String::new(),
        }
    }
    fn b(&self, i: usize) -> bool {
        matches!(self.0.get(i), Some(J::Bool(true)))
    }
    fn n(&self, i: usize) -> i64 {
        self.0.get(i).and_then(J::as_i64).unwrap_or(0)
    }
    fn m(&self, i: usize) -> Memory {
        match self.0.get(i) {
            Some(J::Object(m)) => Memory::from_plain(m),
            _ => Memory::new(),
        }
    }
    fn v(&self, i: usize) -> Value {
        self.0.get(i).map(Value::from_plain).unwrap_or(Value::None)
    }
    fn strs(&self, i: usize) -> Vec<String> {
        let j = match self.0.get(i) {
            Some(J::Object(o)) if o.contains_key("__set__") => o["__set__"].clone(),
            Some(j) => j.clone(),
            None => J::Null,
        };
        j.as_array().map(|a| a.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect()).unwrap_or_default()
    }
    fn set(&self, i: usize) -> HashSet<String> {
        self.strs(i).into_iter().collect()
    }
}

fn s(x: impl Into<String>) -> J {
    J::String(x.into())
}
fn os(x: Option<String>) -> J {
    x.map(J::String).unwrap_or(J::Null)
}
fn pool(x: Option<Vec<String>>) -> J {
    x.map(|v| json!(v)).unwrap_or(J::Null)
}
fn set(x: HashSet<String>) -> J {
    let mut v: Vec<String> = x.into_iter().collect();
    v.sort();
    json!({ "__set__": v })
}
fn val(x: Value) -> J {
    x.to_plain()
}
fn mem(m: &Memory) -> J {
    J::Object(m.to_plain())
}

/// The STEP_* pattern a `_read_step_tried` shape stands for, as the recorder writes a pattern.
fn shape_pattern(shape: tried::StepShape) -> J {
    let name = match shape {
        tried::StepShape::Already => "STEP_ALREADY",
        tried::StepShape::Tried => "STEP_TRIED",
        tried::StepShape::Last => "STEP_LAST",
        tried::StepShape::Done => "STEP_DONE",
        tried::StepShape::Did => "STEP_DID",
    };
    let re = rsbot::data::re(&format!("bot.memory.tried.{name}"));
    json!({ "__re__": [re.pattern(), re.flags()] })
}

/// The result, and the memory after the call when the function writes it.
fn call(name: &str, a: &Args) -> Option<(J, Option<J>)> {
    let short = name.rsplit_once(':').map(|(m, f)| (m.rsplit('.').next().unwrap(), f)).unwrap();
    macro_rules! r {
        ($e:expr) => {
            Some(($e, None))
        };
    }
    macro_rules! with_mut {
        ($i:expr, $f:expr) => {{
            let mut m = a.m($i);
            let out = $f(&mut m);
            Some((out, Some(mem(&m))))
        }};
    }
    match short {
        // tried
        ("tried", "words_of") => r!(json!(tried::words_of(&a.s(0)))),
        ("tried", "reads_as_not_worked") => r!(json!(tried::reads_as_not_worked(&a.s(0)))),
        ("tried", "the_still_state") => r!(s(tried::the_still_state(&a.s(0), &a.m(1)))),
        ("tried", "still_the_state") => r!(json!(tried::still_the_state(&a.s(0), &a.m(1)))),
        ("tried", "might_be_a_report") => r!(json!(tried::might_be_a_report(&a.s(0)))),
        ("tried", "names_a_trade") => r!(json!(tried::names_a_trade(&a.s(0)))),
        ("tried", "after_an_answer") => r!(json!(tried::after_an_answer(&a.m(0)))),
        ("tried", "last_answer_words") => r!(set(tried::last_answer_words(&a.m(0)))),
        ("tried", "asks_for_a_fix") => r!(json!(tried::asks_for_a_fix(&a.m(0)))),
        ("tried", "buying") => r!(json!(tried::buying(&a.m(0)))),
        ("tried", "person_said") => r!(s(tried::person_said(&a.m(0)))),
        ("tried", "_the_lines") => r!(json!(tried::_the_lines(&a.m(0)))),
        ("tried", "names_a_thing") => r!(json!(tried::names_a_thing(&a.m(0)))),
        ("tried", "has_evidence") => r!(json!(tried::has_evidence(&a.s(0), &a.m(1)))),
        ("tried", "asked_what_its_doing") => r!(json!(tried::asked_what_its_doing(&a.m(0)))),
        ("tried", "already_named") => r!(json!(tried::already_named(&a.s(0), &a.m(1)))),
        ("tried", "trade_for") => r!(os(tried::trade_for(&a.m(0)))),
        ("tried", "acted_pool") => r!(pool(tried::acted_pool(&a.s(0), &a.m(1)))),
        ("tried", "_up_to_the_contrast") => r!(s(tried::_up_to_the_contrast(&a.s(0)))),
        ("tried", "_read_step_tried") => r!(match tried::_read_step_tried(&a.s(0)) {
            Some((what, shape)) => json!([what, shape_pattern(shape)]),
            None => json!([null, null]),
        }),
        ("tried", "the_step_tried") => r!(os(tried::the_step_tried(&a.s(0)))),
        ("tried", "_the_past_step") => r!(os(tried::_the_past_step(&a.s(0), &a.s(1)))),
        ("tried", "_the_last_answer") => r!(s(tried::_the_last_answer(&a.m(0)))),
        ("tried", "_stem") => r!(s(tried::_stem(&a.s(0)))),
        ("tried", "_step_stems") => r!(set(tried::_step_stems(&a.s(0)))),
        ("tried", "_names_it") => r!(json!(tried::_names_it(&a.s(0), &a.set(1)))),
        ("tried", "_clauses") => r!(json!(tried::_clauses(&a.s(0)))),
        ("tried", "_a_step") => r!(s(tried::_a_step(&a.s(0)))),
        ("tried", "_opens_a_step") => r!(json!(tried::_opens_a_step(&a.s(0)))),
        ("tried", "_the_step_after") => r!(s(tried::_the_step_after(&a.s(0), &a.m(1)))),
        ("tried", "_the_steps_of") => r!(json!(tried::_the_steps_of(&a.s(0)))),
        ("tried", "_list_heads") => r!(set(tried::_list_heads(&a.s(0)))),
        ("tried", "_object_stems") => r!(set(tried::_object_stems(&a.s(0)))),
        ("tried", "in_the_answer") => r!(json!(tried::in_the_answer(&a.s(0), &a.s(1)))),
        ("tried", "_flat") => r!(s(tried::_flat(&a.s(0)))),
        ("tried", "_walk_answer") => r!(s(tried::_walk_answer(&a.s(0), &a.s(1), a.0.get(2).map_or(true, |w| w.as_bool() == Some(true))))),
        ("tried", "reads_as_a_step_done") => r!(json!(tried::reads_as_a_step_done(&a.s(0), &a.m(1)))),
        ("tried", "_as_a_lead") => r!(s(tried::_as_a_lead(&a.s(0), &a.s(1), &a.s(2)))),
        ("tried", "next_step_pool") => with_mut!(1, |m: &mut Memory| pool(tried::next_step_pool(&a.s(0), m))),
        ("tried", "_asks_for_the_next_part") => r!(json!(tried::_asks_for_the_next_part(&a.s(0)))),
        ("tried", "next_step_ask_pool") => with_mut!(1, |m: &mut Memory| pool(tried::next_step_ask_pool(&a.s(0), m))),
        ("tried", "_somebody_is_the_subject") => r!(json!(tried::_somebody_is_the_subject(&a.m(0)))),
        ("tried", "still_tail") => r!(s(tried::still_tail(&a.s(0), &a.m(1)))),
        ("tried", "a_card_is_not_the_answer") => r!(json!(tried::a_card_is_not_the_answer(&a.s(0), &a.m(1)))),
        ("tried", "still_going_on") => with_mut!(1, |m: &mut Memory| pool(tried::still_going_on(&a.s(0), m))),
        ("tried", "reply_for") => with_mut!(1, |m: &mut Memory| os(tried::reply_for(&a.s(0), m))),
        ("tried", "pool_for") => with_mut!(1, |m: &mut Memory| pool(tried::pool_for(&a.s(0), m))),
        ("tried", "_the_person_word") => r!(s(tried::_the_person_word(&a.s(0)))),
        ("tried", "_a_report_on_a_person") => r!(json!(tried::_a_report_on_a_person(&a.s(0)))),
        ("tried", "_in_care") => r!(tried::_in_care(&a.m(0)).map_or(J::Null, |(w, x, y)| json!([w, x, y]))),
        ("tried", "in_care_pool") => r!(pool(tried::in_care_pool(&a.s(0), &a.m(1)))),
        // when
        ("when", "asks_what_today_is") => r!(json!(when::asks_what_today_is(&a.s(0)))),
        ("when", "when_answer") => r!(os(when::when_answer(&a.s(0), &a.m(1)))),
        ("when", "already_answer") => r!(os(when::already_answer(&a.s(0), &a.m(1)))),
        ("when", "until_answer") => r!(os(when::until_answer(&a.s(0), &a.m(1)))),
        ("when", "how_long_answer") => r!(os(when::how_long_answer(&a.s(0), &a.m(1)))),
        ("when", "a_plan_moved_to_a_day") => r!(when::a_plan_moved_to_a_day(&a.s(0)).map_or(J::Null, val)),
        ("when", "the_plan") => r!(os(when::the_plan(&a.m(0)))),
        ("when", "_dated") => r!(json!(when::_dated(&a.m(0)).iter().map(|(l, sp)| json!([l, sp.iter().map(|(w, at)| json!([w, at])).collect::<Vec<_>>()])).collect::<Vec<_>>())),
        ("when", "_is_an_ask") => r!(json!(when::_is_an_ask(&a.s(0)))),
        ("when", "_points_back") => r!(json!(when::_points_back(&a.s(0)))),
        ("when", "_put_right") => r!({ let (x, y) = when::_put_right(&a.m(0)); json!([x, y]) }),
        ("when", "_the_line_put_right") => r!({ let (x, y) = when::_the_line_put_right(&a.m(0)); json!([x, y]) }),
        ("when", "until_ask_matches") => r!(json!(when::until_ask_matches(&a.s(0), &a.v(1)))),
        ("when", "the_part_asked") => r!(s(when::the_part_asked(&a.s(0), &a.s(1), &a.s(2), &Value::None))),
        ("when", "_no_date_yet") => r!(os(when::_no_date_yet(&a.strs(0), &a.s(1), &a.m(2)))),
        // relations
        ("relations", "_folded") => r!(s(rel::_folded(&a.s(0), &a.m(1)))),
        ("relations", "_person_from_pronoun") => with_mut!(1, |m: &mut Memory| s(rel::_person_from_pronoun(&a.s(0), m, &a.strs(2)))),
        ("relations", "_gender_of") => r!(s(rel::_gender_of(&a.s(0)))),
        ("relations", "_newest_first") => r!(json!(rel::_newest_first(&a.strs(0), &a.m(1)))),
        ("relations", "_named_person") => {
            if a.0.get(1).is_some_and(|m| m.is_object()) {
                with_mut!(1, |m: &mut Memory| s(rel::_named_person(&a.s(0), Some(m))))
            } else {
                r!(s(rel::_named_person(&a.s(0), None)))
            }
        }
        ("relations", "_a_person_in_the_store") => with_mut!(0, |m: &mut Memory| json!(rel::_a_person_in_the_store(m, &a.s(1)))),
        ("relations", "_continues_an_about_ask") => r!(json!(rel::_continues_an_about_ask(&a.m(0), &a.s(1)))),
        ("relations", "_same_as_last") => r!(json!(rel::_same_as_last(&a.s(0), &a.m(1)))),
        ("relations", "_animal_family") => r!(s(rel::_animal_family(&a.s(0)))),
        ("relations", "_their_taste") => with_mut!(0, |m: &mut Memory| os(rel::_their_taste(m, &a.s(1)))),
        ("relations", "_their_job") => with_mut!(0, |m: &mut Memory| os(rel::_their_job(m, &a.s(1)))),
        ("relations", "_whether_they_have") => with_mut!(0, |m: &mut Memory| os(rel::_whether_they_have(m, &a.s(1)))),
        ("relations", "_the_names_of_the_group") => with_mut!(0, |m: &mut Memory| os(rel::_the_names_of_the_group(m, &a.s(1)))),
        ("relations", "_relation_reply") => with_mut!(0, |m: &mut Memory| os(rel::_relation_reply(m, &a.s(1)))),
        ("relations", "_ages_reply") => with_mut!(0, |m: &mut Memory| os(rel::_ages_reply(m, &a.s(1), &a.s(2)))),
        ("relations", "_the_place_end") => r!({ let (x, y) = rel::_the_place_end(&a.s(0)); json!([x, y]) }),
        ("relations", "_the_place_word_filed") => with_mut!(0, |m: &mut Memory| s(rel::_the_place_word_filed(m, &a.s(1)))),
        ("relations", "_pet_ages") => with_mut!(0, |m: &mut Memory| json!(rel::_pet_ages(m).iter().map(|p| p.to_plain()).collect::<Vec<_>>())),
        ("relations", "_pet_age_reply") => with_mut!(0, |m: &mut Memory| os(rel::_pet_age_reply(m, &a.s(1)))),
        ("relations", "_just_gave_an_age") => r!(json!(rel::_just_gave_an_age(&a.m(0)))),
        ("relations", "_one_age") => r!(s(rel::_one_age(&a.s(0), &a.v(1), &a.s(2), &a.v(3), &a.v(4)))),
        ("relations", "_called") => r!(s(rel::_called(&a.s(0)))),
        ("relations", "_a") => r!(s(rel::_a(&a.s(0)))),
        ("relations", "_plural_person") => r!(json!(rel::_plural_person(&a.s(0)))),
        // detail
        ("detail", "the_detail") => r!(os(detail::the_detail(&a.s(0)))),
        ("detail", "_words") => r!(json!(detail::_words(&a.s(0)))),
        ("detail", "_a_verb") => r!(json!(detail::_a_verb(&a.s(0), &a.s(1)))),
        ("detail", "the_fact_the_answer_has") => r!(os(detail::the_fact_the_answer_has(&a.s(0), &a.m(1)))),
        ("detail", "the_time_in_the_line") => r!(os(detail::the_time_in_the_line(&a.s(0)))),
        ("detail", "the_step_for") => r!(detail::the_step_for(&a.s(0), &a.m(1)).map_or(J::Null, |(d, x, st, w)| json!([d, x, st, w]))),
        ("detail", "the_step_for_a_time") => r!(detail::the_step_for_a_time(&a.s(0), &a.m(1)).map_or(J::Null, |(d, x, st, w)| json!([d, x, st, w]))),
        ("detail", "_the_carrying_words") => r!(json!(detail::_the_carrying_words(&a.s(0)))),
        ("detail", "the_part_of_the_answer") => r!(s(detail::the_part_of_the_answer(&a.s(0), &a.m(1)))),
        ("detail", "answers_the_line") => r!(json!(detail::answers_the_line(&a.s(0), &a.m(1)))),
        ("detail", "answer_for") => with_mut!(1, |m: &mut Memory| os(detail::answer_for(&a.s(0), m))),
        ("detail", "the_plain_ask") => with_mut!(1, |m: &mut Memory| os(detail::the_plain_ask(&a.s(0), m))),
        ("detail", "_capital") => r!(s(detail::_capital(&a.s(0)))),
        ("detail", "stands_alone") => r!(json!(detail::stands_alone(&a.s(0)))),
        ("detail", "_report_stem") => r!(s(detail::_report_stem(&a.s(0)))),
        ("detail", "_report_content") => r!(json!(detail::_report_content(&a.s(0)))),
        ("detail", "a_report_about_the_thing") => r!(s(detail::a_report_about_the_thing(&a.s(0), &a.m(1)))),
        ("detail", "_a_step") => r!(s(detail::_a_step(&a.s(0), a.b(1)))),
        ("detail", "_first_clause") => r!(s(detail::_first_clause(&a.s(0)))),
        ("detail", "_flat") => r!(s(detail::_flat(&a.s(0)))),
        // enough, pile
        ("enough", "enough_reply") => r!(os(enough::enough_reply(&a.s(0), &a.m(1)))),
        ("enough", "_amounts") => r!(json!(enough::_amounts(&a.s(0)).iter().map(|(v, f, x)| json!([v, f, x])).collect::<Vec<_>>())),
        ("enough", "_bare_figures") => r!(json!(enough::_bare_figures(&a.s(0)))),
        ("pile", "reads_the_pile") => r!(json!(pile::reads_the_pile(&a.s(0), &a.m(1)))),
        ("pile", "reply_for") => r!(os(pile::reply_for(&a.s(0), &a.m(1)))),
        // saving
        ("saving", "_a_sum") => r!(sv::_a_sum(&a.s(0)).map_or(J::Null, |n| json!(n))),
        ("saving", "_a_date") => r!(s(sv::_a_date(&a.s(0)))),
        ("saving", "_two_sums_added") => r!(json!(sv::_two_sums_added(&a.s(0)))),
        ("saving", "the_pot_and_the_rest") => r!(val(sv::the_pot_and_the_rest(&a.s(0)))),
        ("saving", "the_pot_said_alone") => r!(json!(sv::the_pot_said_alone(&a.s(0)))),
        ("saving", "the_pot_they_are_on") => r!(json!(sv::the_pot_they_are_on(&a.s(0)))),
        ("saving", "_a_rate") => r!(json!(sv::_a_rate(&a.s(0)))),
        ("saving", "asks_how_far_off") => r!(json!(sv::asks_how_far_off(&a.s(0)))),
        ("saving", "the_gap_left") => r!(s(sv::the_gap_left(&a.s(0), &a.m(1)))),
        ("saving", "the_pot_position") => r!(s(sv::the_pot_position(&a.s(0), &a.m(1)))),
        ("saving", "_the_money_already_held") => r!(json!(sv::_the_money_already_held(&a.m(0)))),
        ("saving", "_names_a_goal") => r!(json!(sv::_names_a_goal(&a.s(0)))),
        ("saving", "asks_how_long") => r!(json!(sv::asks_how_long(&a.s(0)))),
        ("saving", "_only_a_date") => r!(s(sv::_only_a_date(&a.s(0)))),
        ("saving", "is_only_a_date") => r!(json!(sv::is_only_a_date(&a.s(0)))),
        ("saving", "_only_a_sum") => r!(json!(sv::_only_a_sum(&a.s(0)))),
        ("saving", "asks_about_saving") => r!(json!(sv::asks_about_saving(&a.s(0)))),
        ("saving", "_the_money_still_to_find") => r!(json!(sv::_the_money_still_to_find(&a.m(0)))),
        ("saving", "_the_goal_in_the_chat") => r!(json!(sv::_the_goal_in_the_chat(&a.m(0)))),
        ("saving", "_the_goal_named_in_a_line") => r!(json!(sv::_the_goal_named_in_a_line(&a.m(0)))),
        ("saving", "_the_goal_in_the_answer") => r!(json!(sv::_the_goal_in_the_answer(&a.m(0)))),
        ("saving", "another_sum_asked") => r!(s(sv::another_sum_asked(&a.s(0), &a.m(1)))),
        ("saving", "a_rate_put_right") => r!(json!(sv::a_rate_put_right(&a.s(0), &a.s(1)))),
        ("saving", "_the_rate_in_the_chat") => r!(json!(sv::_the_rate_in_the_chat(&a.m(0)))),
        ("saving", "_the_date_in_the_chat") => r!(s(sv::_the_date_in_the_chat(&a.m(0)))),
        ("saving", "asks_if_the_rate_is_enough") => r!(json!(sv::asks_if_the_rate_is_enough(&a.s(0)))),
        ("saving", "a_yes_no_about_the_rate") => r!(s(sv::a_yes_no_about_the_rate(&a.s(0), &a.m(1)))),
        ("saving", "the_extra_a_month_asked") => r!(s(sv::the_extra_a_month_asked(&a.s(0), &a.m(1)))),
        ("saving", "the_same_figure_a_month") => r!(s(sv::the_same_figure_a_month(&a.s(0), &a.m(1)))),
        ("saving", "a_rate_restated") => r!(s(sv::a_rate_restated(&a.s(0), &a.m(1)))),
        ("saving", "a_rate_put_in_with_a_verb") => r!(s(sv::a_rate_put_in_with_a_verb(&a.s(0), &a.m(1)))),
        ("saving", "a_length_at_the_rate_in_the_chat") => r!(s(sv::a_length_at_the_rate_in_the_chat(&a.s(0), &a.m(1)))),
        ("saving", "a_rate_with_no_date") => r!(s(sv::a_rate_with_no_date(&a.s(0), &a.m(1)))),
        ("saving", "asks_the_rate") => r!(json!(sv::asks_the_rate(&a.s(0), &a.m(1)))),
        ("saving", "rate_question") => r!(s(sv::rate_question(&a.s(0), &a.m(1)))),
        ("saving", "_a_rate_said") => r!(val(sv::_a_rate_said(&a.s(0)))),
        ("saving", "a_rate_for_the_goal") => r!(s(sv::a_rate_for_the_goal(&a.s(0), &a.m(1)))),
        ("saving", "a_rate_raised") => r!(s(sv::a_rate_raised(&a.s(0), &a.m(1)))),
        ("saving", "a_length_with_no_price") => r!(s(sv::a_length_with_no_price(&a.s(0), &a.m(1)))),
        ("saving", "a_price_in_an_answer") => r!(s(sv::a_price_in_an_answer(&a.s(0), &a.m(1)))),
        ("saving", "how_long_it_takes") => r!(s(sv::how_long_it_takes(&a.m(0)))),
        ("saving", "said_already") => r!(json!(sv::said_already(&a.s(0), &a.m(1)))),
        ("saving", "without_what_was_just_said") => r!(s(sv::without_what_was_just_said(&a.s(0), &a.m(1)))),
        ("saving", "_months") => r!(s(sv::_months(a.n(0)))),
        // trips
        ("trips", "reply_for") => with_mut!(1, |m: &mut Memory| os(trips::reply_for(&a.s(0), m, a.0.get(2).map_or(true, |c| c.as_bool() != Some(false))))),
        ("trips", "a_trip") => r!(trips::a_trip(&a.s(0), &a.m(1), a.0.get(2).map_or(true, |c| c.as_bool() != Some(false))).map_or(J::Null, |t| t.to_value().to_plain())),
        ("trips", "_places") => r!(s(trips::_places(&a.s(0), a.b(1)))),
        ("trips", "_low") => r!(s(trips::_low(&a.s(0)))),
        ("trips", "_month") => r!(s(trips::_month(&a.s(0)))),
        ("trips", "_when_phrase") => r!(s(trips::_when_phrase(&a.s(0)))),
        ("trips", "_who") => r!(s(trips::_who(&a.s(0)))),
        ("trips", "_how_long") => r!(s(trips::_how_long(&a.s(0)))),
        ("trips", "_how_getting_there") => r!(s(trips::_how_getting_there(&a.s(0)))),
        ("trips", "_a_carded_place") => r!(json!(trips::_a_carded_place(&a.s(0)))),
        ("trips", "_a_relation_not_a_place") => r!(json!(trips::_a_relation_not_a_place(&a.s(0)))),
        ("trips", "_a_person_visited") => r!(s(trips::_a_person_visited(&a.s(0)))),
        ("trips", "_the_trip_so_far") => with_mut!(0, |m: &mut Memory| val(trips::_the_trip_so_far(m))),
        ("trips", "the_trip_they_booked") => r!(json!(trips::the_trip_they_booked(&a.s(0)))),
        ("trips", "the_bag_beats_the_list") => with_mut!(1, |m: &mut Memory| json!(trips::the_bag_beats_the_list(&a.s(0), m))),
        ("trips", "_only_a_length") => r!(json!(trips::_only_a_length(&a.s(0)))),
        ("trips", "_as_a_visit") => r!(s(trips::_as_a_visit(&a.s(0)))),
        ("trips", "_cap") => r!(s(trips::_cap(&a.s(0)))),
        ("trips", "_seasons_we_know") => r!(s(trips::_seasons_we_know(&a.s(0)))),
        ("trips", "_somewhere_for") => with_mut!(1, |m: &mut Memory| os(trips::_somewhere_for(&a.s(0), m))),
        _ => None,
    }
}

/// Python's result as the recorder wrote it, with a None for a str contract read as "".
fn same(want: &J, got: &J) -> bool {
    want == got || (want.is_null() && got == &J::String(String::new())) || (want == &J::String(String::new()) && got.is_null())
}

fn main() {
    rsbot::runtime::quiet_panics();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut paths = Vec::new();
    let (mut fns, mut show) = (Vec::new(), 3usize);
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
            p => paths.push(p.to_string()),
        }
        i += 1;
    }
    // fn -> [same, differ, unported, shape/raised, no port]
    let mut tally: BTreeMap<String, [usize; 5]> = BTreeMap::new();
    let mut shown: BTreeMap<String, usize> = BTreeMap::new();
    let mut blockers: BTreeMap<String, usize> = BTreeMap::new();
    for path in paths {
        let file = std::fs::File::open(&path).unwrap();
        for line in std::io::BufReader::new(flate2::read::GzDecoder::new(file)).lines() {
            let Ok(line) = line else { break };
            let Ok(row) = serde_json::from_str::<J>(&line) else { break };
            if row.get("dep").is_some() {
                continue; // a dependency's answer, for the mock crate (harness/mem5_mockpatch.py)
            }
            let name = row["fn"].as_str().unwrap_or("").to_string();
            if !fns.is_empty() && !fns.iter().any(|f| name.contains(f.as_str())) {
                continue;
            }
            let t = tally.entry(name.clone()).or_default();
            let text = row["args"].to_string();
            if text.contains("__opaque__") || text.contains("__match__") || text.contains("__fn__") {
                t[3] += 1;
                continue;
            }
            let mut pos = row["args"].as_array().cloned().unwrap_or_default();
            // Keyword arguments, put where the signature has them.
            if let Some(kw) = row["kwargs"].as_object() {
                for (key, at) in [("cards", 2usize), ("wrap", 2), ("questions_are_steps", 1), ("memory", 1), ("text", 2), ("before", 1), ("pot", 3), ("named_here", 2)] {
                    if let Some(v) = kw.get(key) {
                        while pos.len() < at {
                            pos.push(J::Null);
                        }
                        if pos.len() == at {
                            pos.push(v.clone());
                        } else {
                            pos[at] = v.clone();
                        }
                    }
                }
            }
            let got = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| call(&name, &Args(&pos))));
            match got {
                Ok(None) => t[4] += 1,
                Ok(Some((j, after))) => {
                    if row["raised"].is_string() {
                        t[1] += 1;
                        if *shown.entry(name.clone()).or_default() < show {
                            *shown.get_mut(&name).unwrap() += 1;
                            println!("DIFF {name} args={}\n  python raised {}\n  rust:   {}", row["args"], row["raised"], j);
                        }
                        continue;
                    }
                    let mut ok = same(&row["result"], &j);
                    let mut detail = String::new();
                    if let (Some(after), Some(mem_index)) = (after, row.get("after")) {
                        // The memory argument after the call, normalised the same way on both sides.
                        let want_mem = mem_index
                            .as_array()
                            .and_then(|a| a.iter().find(|x| x.is_object()))
                            .map(|m| J::Object(Memory::from_plain(m.as_object().unwrap()).to_plain()));
                        if let Some(want_mem) = want_mem {
                            if want_mem != after {
                                ok = false;
                                detail = format!("\n  memory python: {want_mem}\n  memory rust:   {after}");
                            }
                        }
                    }
                    if ok {
                        t[0] += 1;
                    } else {
                        t[1] += 1;
                        let n = shown.entry(name.clone()).or_default();
                        if *n < show {
                            *n += 1;
                            println!("DIFF {name} args={}\n  python: {}\n  rust:   {}{detail}", row["args"], row["result"], j);
                        }
                    }
                }
                Err(e) => {
                    let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                    if msg.starts_with("UNPORTED") {
                        t[2] += 1;
                        *blockers.entry(msg.clone()).or_default() += 1;
                    } else if row["raised"].is_string() {
                        t[0] += 1;
                    } else {
                        t[1] += 1;
                        let n = shown.entry(name.clone()).or_default();
                        if *n < show {
                            *n += 1;
                            println!("PANIC {name} args={}\n  python: {}\n  rust panic: {msg}", row["args"], row["result"]);
                        }
                    }
                }
            }
        }
    }
    let mut total = [0usize; 5];
    println!("{:<52} {:>6} {:>6} {:>6} {:>6} {:>6}", "function", "same", "differ", "unport", "skip", "noport");
    for (name, t) in &tally {
        println!("{:<52} {:>6} {:>6} {:>6} {:>6} {:>6}", name, t[0], t[1], t[2], t[3], t[4]);
        for k in 0..5 {
            total[k] += t[k];
        }
    }
    for (name, n) in &blockers {
        println!("blocked by {name} x{n}");
    }
    println!("TOTAL same {} differ {} unported {} skipped {} no-port {}", total[0], total[1], total[2], total[3], total[4]);
}
