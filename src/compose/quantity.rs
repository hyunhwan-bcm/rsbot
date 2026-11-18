//! Port of `bot/compose/quantity.py`.
//!
//! GENERATED SKELETON: every function is a stub until its workstream ports it. The
//! signatures are the contract other files call against; keep them.
#![allow(unused_variables, unused_imports, non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::knowledge::entry::Entry;
use crate::py::re::Re;
use crate::runtime::Candidate;
use crate::state::{Memory, Turn};
use crate::understood::Understood;
use std::sync::LazyLock;

/// Python: bot/compose/quantity.py:370 `_number_at(words, at)`
pub fn _number_at(words: &[String], at: i64) -> Value {
    crate::unported!("bot.compose.quantity:_number_at")
}

/// Python: bot/compose/quantity.py:391 `_amount(word)`
pub fn _amount(word: &str) {
    crate::unported!("bot.compose.quantity:_amount")
}

/// Python: bot/compose/quantity.py:407 `_fmt(value)`
pub fn _fmt(value: f64) -> String {
    crate::unported!("bot.compose.quantity:_fmt")
}

/// Python: bot/compose/quantity.py:414 `_minutes(total)` (never called on the trace corpus)
pub fn _minutes(total: &Value) -> String {
    crate::unported!("bot.compose.quantity:_minutes")
}

/// Python: bot/compose/quantity.py:438 `_the_sum_inside_the_word(text)`
pub fn _the_sum_inside_the_word(text: &str) -> Vec<String> {
    crate::unported!("bot.compose.quantity:_the_sum_inside_the_word")
}

/// Python: bot/compose/quantity.py:468 `_the_money_words(text)` (never called on the trace corpus)
pub fn _the_money_words(text: &str) -> bool {
    crate::unported!("bot.compose.quantity:_the_money_words")
}

/// Python: bot/compose/quantity.py:472 `_about_a_distance(clean, memory)` (never called on the trace corpus)
pub fn _about_a_distance(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.quantity:_about_a_distance")
}

/// Python: bot/compose/quantity.py:487 `_read(clean, memory=None)`
pub fn _read(clean: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.quantity:_read")
}

/// Python: bot/compose/quantity.py:614 `_q(phrase, unit)` (never called on the trace corpus)
pub fn _q(phrase: &str, unit: &str) -> Value {
    crate::unported!("bot.compose.quantity:_q")
}

/// Python: bot/compose/quantity.py:694 `_a_length_they_have_bounded(clean, unit, body, period)`
pub fn _a_length_they_have_bounded(clean: &str, unit: &str, body: &[String], period: &Value) -> bool {
    crate::unported!("bot.compose.quantity:_a_length_they_have_bounded")
}

/// Python: bot/compose/quantity.py:714 `_a_bound_not_a_budget(clean)` (never called on the trace corpus)
pub fn _a_bound_not_a_budget(clean: &str) -> bool {
    crate::unported!("bot.compose.quantity:_a_bound_not_a_budget")
}

/// Python: bot/compose/quantity.py:720 `_the_turn_above(memory)` (never called on the trace corpus)
pub fn _the_turn_above(memory: &Memory) -> Value {
    crate::unported!("bot.compose.quantity:_the_turn_above")
}

/// Python: bot/compose/quantity.py:739 `_the_change_is_the_line(clean)` (never called on the trace corpus)
pub fn _the_change_is_the_line(clean: &str) -> bool {
    crate::unported!("bot.compose.quantity:_the_change_is_the_line")
}

/// Python: bot/compose/quantity.py:744 `_a_weight_they_mean_themselves(memory, clean='')` (never called on the trace corpus)
pub fn _a_weight_they_mean_themselves(memory: &Memory, clean: &str) -> bool {
    crate::unported!("bot.compose.quantity:_a_weight_they_mean_themselves")
}

/// Python: bot/compose/quantity.py:764 `_a_thing_was_named(memory, clean='')` (never called on the trace corpus)
pub fn _a_thing_was_named(memory: &Memory, clean: &str) -> bool {
    crate::unported!("bot.compose.quantity:_a_thing_was_named")
}

/// Python: bot/compose/quantity.py:785 `_a_span_they_are_in(clean, memory)` (never called on the trace corpus)
pub fn _a_span_they_are_in(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.quantity:_a_span_they_are_in")
}

/// Python: bot/compose/quantity.py:804 `_a_count_the_question_asked_for(clean, memory)` (never called on the trace corpus)
pub fn _a_count_the_question_asked_for(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.quantity:_a_count_the_question_asked_for")
}

/// Python: bot/compose/quantity.py:821 `_a_fact_about_a_length(memory)` (never called on the trace corpus)
pub fn _a_fact_about_a_length(memory: &Memory) -> bool {
    crate::unported!("bot.compose.quantity:_a_fact_about_a_length")
}

/// Python: bot/compose/quantity.py:828 `reply_for(clean, memory)`
pub fn reply_for(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.quantity:reply_for")
}

/// Python: bot/compose/quantity.py:866 `the_thing_they_pay_for(clean)` (never called on the trace corpus)
pub fn the_thing_they_pay_for(clean: &str) -> Value {
    crate::unported!("bot.compose.quantity:the_thing_they_pay_for")
}

/// Python: bot/compose/quantity.py:882 `the_rate_they_asked_about(clean, memory=None)`
pub fn the_rate_they_asked_about(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.quantity:the_rate_they_asked_about")
}

/// Python: bot/compose/quantity.py:908 `_without_a_pointing_tail(words)`
pub fn _without_a_pointing_tail(words: &[String]) -> Vec<String> {
    crate::unported!("bot.compose.quantity:_without_a_pointing_tail")
}

/// Python: bot/compose/quantity.py:921 `_without_a_hedge(words)`
pub fn _without_a_hedge(words: &[String]) -> Vec<String> {
    crate::unported!("bot.compose.quantity:_without_a_hedge")
}

/// Python: bot/compose/quantity.py:945 `_without_the_household(words)`
pub fn _without_the_household(words: &[String]) -> Vec<String> {
    crate::unported!("bot.compose.quantity:_without_the_household")
}

/// Python: bot/compose/quantity.py:963 `_a_take_without_home(lead)` (never called on the trace corpus)
pub fn _a_take_without_home(lead: &str) -> bool {
    crate::unported!("bot.compose.quantity:_a_take_without_home")
}

/// Python: bot/compose/quantity.py:969 `_without_a_remark(words)`
pub fn _without_a_remark(words: &[String]) -> Vec<String> {
    crate::unported!("bot.compose.quantity:_without_a_remark")
}

/// Python: bot/compose/quantity.py:979 `_in_a_conversation(memory)` (never called on the trace corpus)
pub fn _in_a_conversation(memory: &Memory) -> bool {
    crate::unported!("bot.compose.quantity:_in_a_conversation")
}

/// Python: bot/compose/quantity.py:992 `_not_short(pool, value)` (never called on the trace corpus)
pub fn _not_short(pool: &Value, value: &Value) -> Value {
    crate::unported!("bot.compose.quantity:_not_short")
}

/// Python: bot/compose/quantity.py:1001 `_without_a_question(pool)` (never called on the trace corpus)
pub fn _without_a_question(pool: &Value) -> Value {
    crate::unported!("bot.compose.quantity:_without_a_question")
}

/// Python: bot/compose/quantity.py:1008 `_say(family, q, value, period, tail, memory, clean, unit=None)` (never called on the trace corpus)
pub fn _say(family: &Value, q: &Value, value: &Value, period: &Value, tail: &str, memory: &Memory, clean: &str, unit: &str) -> Option<String> {
    crate::unported!("bot.compose.quantity:_say")
}

/// Python: bot/compose/quantity.py:1223 `_the_unit_of_one(unit)` (never called on the trace corpus)
pub fn _the_unit_of_one(unit: &str) -> Value {
    crate::unported!("bot.compose.quantity:_the_unit_of_one")
}

/// Python: bot/compose/quantity.py:1235 `_the_answer_range(above)` (never called on the trace corpus)
pub fn _the_answer_range(above: &Value) -> Value {
    crate::unported!("bot.compose.quantity:_the_answer_range")
}

/// Python: bot/compose/quantity.py:1267 `_the_rule_said(low, high, unit, period)` (never called on the trace corpus)
pub fn _the_rule_said(low: &str, high: &Value, unit: &str, period: &Value) -> String {
    crate::unported!("bot.compose.quantity:_the_rule_said")
}

/// Python: bot/compose/quantity.py:1275 `_the_family(unit)` (never called on the trace corpus)
pub fn _the_family(unit: &str) -> Value {
    crate::unported!("bot.compose.quantity:_the_family")
}

/// Python: bot/compose/quantity.py:1282 `the_figure_against_the_answer(clean, memory)`
pub fn the_figure_against_the_answer(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.quantity:the_figure_against_the_answer")
}

