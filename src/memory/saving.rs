//! Port of `bot/memory/saving.py`.
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

/// Python: bot/memory/saving.py:106 `_a_sum(text)`
pub fn _a_sum(text: &str) {
    crate::unported!("bot.memory.saving:_a_sum")
}

/// Python: bot/memory/saving.py:141 `_a_date(text)`
pub fn _a_date(text: &str) -> String {
    crate::unported!("bot.memory.saving:_a_date")
}

/// Python: bot/memory/saving.py:162 `_month_of(when)` (never called on the trace corpus)
pub fn _month_of(when: &str) -> Value {
    crate::unported!("bot.memory.saving:_month_of")
}

/// Python: bot/memory/saving.py:191 `_two_sums_added(line)`
pub fn _two_sums_added(line: &str) -> i64 {
    crate::unported!("bot.memory.saving:_two_sums_added")
}

/// Python: bot/memory/saving.py:220 `_pounds(figure, unit)` (never called on the trace corpus)
pub fn _pounds(figure: &Value, unit: &str) -> i64 {
    crate::unported!("bot.memory.saving:_pounds")
}

/// Python: bot/memory/saving.py:231 `the_pot_and_the_rest(text)`
pub fn the_pot_and_the_rest(text: &str) -> Value {
    crate::unported!("bot.memory.saving:the_pot_and_the_rest")
}

/// Python: bot/memory/saving.py:288 `the_pot_said_alone(text)`
pub fn the_pot_said_alone(text: &str) -> i64 {
    crate::unported!("bot.memory.saving:the_pot_said_alone")
}

/// Python: bot/memory/saving.py:324 `the_pot_they_are_on(text)`
pub fn the_pot_they_are_on(text: &str) -> i64 {
    crate::unported!("bot.memory.saving:the_pot_they_are_on")
}

/// Python: bot/memory/saving.py:348 `_a_rate(line)`
pub fn _a_rate(line: &str) -> bool {
    crate::unported!("bot.memory.saving:_a_rate")
}

/// Python: bot/memory/saving.py:353 `_the_pot_they_are_on(memory)` (never called on the trace corpus)
pub fn _the_pot_they_are_on(memory: &Memory) -> Value {
    crate::unported!("bot.memory.saving:_the_pot_they_are_on")
}

/// Python: bot/memory/saving.py:386 `_the_goal_they_need(memory)` (never called on the trace corpus)
pub fn _the_goal_they_need(memory: &Memory) -> Value {
    crate::unported!("bot.memory.saving:_the_goal_they_need")
}

/// Python: bot/memory/saving.py:406 `asks_how_far_off(text)`
pub fn asks_how_far_off(text: &str) -> bool {
    crate::unported!("bot.memory.saving:asks_how_far_off")
}

/// Python: bot/memory/saving.py:411 `the_gap_left(text, memory)`
pub fn the_gap_left(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:the_gap_left")
}

/// Python: bot/memory/saving.py:434 `_the_gap_said(goal, pot)` (never called on the trace corpus)
pub fn _the_gap_said(goal: &Value, pot: &Value) -> String {
    crate::unported!("bot.memory.saving:_the_gap_said")
}

/// Python: bot/memory/saving.py:442 `_the_gap_was_just_said(said, memory)` (never called on the trace corpus)
pub fn _the_gap_was_just_said(said: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.saving:_the_gap_was_just_said")
}

/// Python: bot/memory/saving.py:455 `_the_gap_again(goal, pot, memory)` (never called on the trace corpus)
pub fn _the_gap_again(goal: &Value, pot: &Value, memory: &Memory) -> Value {
    crate::unported!("bot.memory.saving:_the_gap_again")
}

/// Python: bot/memory/saving.py:472 `the_goal_and_the_pot(memory)` (never called on the trace corpus)
pub fn the_goal_and_the_pot(memory: &Memory) -> Value {
    crate::unported!("bot.memory.saving:the_goal_and_the_pot")
}

/// Python: bot/memory/saving.py:482 `the_pot_position(text, memory)`
pub fn the_pot_position(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:the_pot_position")
}

/// Python: bot/memory/saving.py:504 `_a_sum_that_is_a_goal(text)` (never called on the trace corpus)
pub fn _a_sum_that_is_a_goal(text: &str) -> bool {
    crate::unported!("bot.memory.saving:_a_sum_that_is_a_goal")
}

/// Python: bot/memory/saving.py:512 `rate_question_for(amount, when='')` (never called on the trace corpus)
pub fn rate_question_for(amount: &Value, when: &str) -> String {
    crate::unported!("bot.memory.saving:rate_question_for")
}

/// Python: bot/memory/saving.py:523 `the_money_already_held(memory)` (never called on the trace corpus)
pub fn the_money_already_held(memory: &Memory) -> Value {
    crate::unported!("bot.memory.saving:the_money_already_held")
}

/// Python: bot/memory/saving.py:528 `_the_money_already_held(memory)`
pub fn _the_money_already_held(memory: &Memory) -> i64 {
    crate::unported!("bot.memory.saving:_the_money_already_held")
}

/// Python: bot/memory/saving.py:553 `without_the_pot_opener(answer)` (never called on the trace corpus)
pub fn without_the_pot_opener(answer: &str) -> Value {
    crate::unported!("bot.memory.saving:without_the_pot_opener")
}

/// Python: bot/memory/saving.py:558 `_names_a_goal(line)`
pub fn _names_a_goal(line: &str) -> i64 {
    crate::unported!("bot.memory.saving:_names_a_goal")
}

/// Python: bot/memory/saving.py:625 `asks_how_long(text)`
pub fn asks_how_long(text: &str) -> bool {
    crate::unported!("bot.memory.saving:asks_how_long")
}

/// Python: bot/memory/saving.py:634 `how_long_it_takes(memory)` (never called on the trace corpus)
pub fn how_long_it_takes(memory: &Memory) -> Value {
    crate::unported!("bot.memory.saving:how_long_it_takes")
}

/// Python: bot/memory/saving.py:736 `_only_a_date(text)`
pub fn _only_a_date(text: &str) -> String {
    crate::unported!("bot.memory.saving:_only_a_date")
}

/// Python: bot/memory/saving.py:752 `is_only_a_date(text)`
pub fn is_only_a_date(text: &str) -> bool {
    crate::unported!("bot.memory.saving:is_only_a_date")
}

/// Python: bot/memory/saving.py:762 `_only_a_sum(text)`
pub fn _only_a_sum(text: &str) -> i64 {
    crate::unported!("bot.memory.saving:_only_a_sum")
}

/// Python: bot/memory/saving.py:788 `asks_about_saving(text)`
pub fn asks_about_saving(text: &str) -> bool {
    crate::unported!("bot.memory.saving:asks_about_saving")
}

/// Python: bot/memory/saving.py:797 `_the_money_still_to_find(memory)`
pub fn _the_money_still_to_find(memory: &Memory) -> i64 {
    crate::unported!("bot.memory.saving:_the_money_still_to_find")
}

/// Python: bot/memory/saving.py:811 `_the_goal_in_the_chat(memory)`
pub fn _the_goal_in_the_chat(memory: &Memory) -> i64 {
    crate::unported!("bot.memory.saving:_the_goal_in_the_chat")
}

/// Python: bot/memory/saving.py:816 `_the_goal_named_in_a_line(memory)`
pub fn _the_goal_named_in_a_line(memory: &Memory) -> i64 {
    crate::unported!("bot.memory.saving:_the_goal_named_in_a_line")
}

/// Python: bot/memory/saving.py:853 `_the_goal_said_as_a_price(memory)` (never called on the trace corpus)
pub fn _the_goal_said_as_a_price(memory: &Memory) -> Value {
    crate::unported!("bot.memory.saving:_the_goal_said_as_a_price")
}

/// Python: bot/memory/saving.py:908 `a_length_was_said(line)` (never called on the trace corpus)
pub fn a_length_was_said(line: &str) -> bool {
    crate::unported!("bot.memory.saving:a_length_was_said")
}

/// Python: bot/memory/saving.py:918 `_a_length_was_asked(line)` (never called on the trace corpus)
pub fn _a_length_was_asked(line: &str) -> bool {
    crate::unported!("bot.memory.saving:_a_length_was_asked")
}

/// Python: bot/memory/saving.py:923 `_the_line_before(memory)` (never called on the trace corpus)
pub fn _the_line_before(memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:_the_line_before")
}

/// Python: bot/memory/saving.py:931 `_the_goal_in_the_answer(memory)`
pub fn _the_goal_in_the_answer(memory: &Memory) -> i64 {
    crate::unported!("bot.memory.saving:_the_goal_in_the_answer")
}

/// Python: bot/memory/saving.py:965 `another_sum_asked(text, memory)`
pub fn another_sum_asked(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:another_sum_asked")
}

/// Python: bot/memory/saving.py:999 `_a_rate_is_settled(memory)` (never called on the trace corpus)
pub fn _a_rate_is_settled(memory: &Memory) -> bool {
    crate::unported!("bot.memory.saving:_a_rate_is_settled")
}

/// Python: bot/memory/saving.py:1018 `_the_length_at_that_rate(goal, rate, period, pot=0)` (never called on the trace corpus)
pub fn _the_length_at_that_rate(goal: &Value, rate: &Value, period: &Value, pot: &Value) -> String {
    crate::unported!("bot.memory.saving:_the_length_at_that_rate")
}

/// Python: bot/memory/saving.py:1037 `_a_rate_in_another_period(memory)` (never called on the trace corpus)
pub fn _a_rate_in_another_period(memory: &Memory) -> Value {
    crate::unported!("bot.memory.saving:_a_rate_in_another_period")
}

/// Python: bot/memory/saving.py:1072 `_is_an_income(line, found)` (never called on the trace corpus)
pub fn _is_an_income(line: &str, found: &Value) -> bool {
    crate::unported!("bot.memory.saving:_is_an_income")
}

/// Python: bot/memory/saving.py:1078 `_is_an_outgoing(line, found)` (never called on the trace corpus)
pub fn _is_an_outgoing(line: &str, found: &Value) -> bool {
    crate::unported!("bot.memory.saving:_is_an_outgoing")
}

/// Python: bot/memory/saving.py:1084 `_is_not_a_rate(line, found)` (never called on the trace corpus)
pub fn _is_not_a_rate(line: &str, found: &Value) -> bool {
    crate::unported!("bot.memory.saving:_is_not_a_rate")
}

/// Python: bot/memory/saving.py:1088 `_the_income_in_the_chat(memory)` (never called on the trace corpus)
pub fn _the_income_in_the_chat(memory: &Memory) -> Value {
    crate::unported!("bot.memory.saving:_the_income_in_the_chat")
}

/// Python: bot/memory/saving.py:1185 `_is_a_rate_line(line)` (never called on the trace corpus)
pub fn _is_a_rate_line(line: &str) -> bool {
    crate::unported!("bot.memory.saving:_is_a_rate_line")
}

/// Python: bot/memory/saving.py:1193 `a_rate_put_right(line, before='')`
pub fn a_rate_put_right(line: &str, before: &str) -> i64 {
    crate::unported!("bot.memory.saving:a_rate_put_right")
}

/// Python: bot/memory/saving.py:1229 `_a_sum_named(figure)` (never called on the trace corpus)
pub fn _a_sum_named(figure: &Value) -> Value {
    crate::unported!("bot.memory.saving:_a_sum_named")
}

/// Python: bot/memory/saving.py:1238 `_the_rate_in_the_chat(memory)`
pub fn _the_rate_in_the_chat(memory: &Memory) -> i64 {
    crate::unported!("bot.memory.saving:_the_rate_in_the_chat")
}

/// Python: bot/memory/saving.py:1275 `_the_date_in_the_chat(memory)`
pub fn _the_date_in_the_chat(memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:_the_date_in_the_chat")
}

/// Python: bot/memory/saving.py:1286 `_months_left(when)` (never called on the trace corpus)
pub fn _months_left(when: &str) -> Value {
    crate::unported!("bot.memory.saving:_months_left")
}

/// Python: bot/memory/saving.py:1305 `_months(n)` (never called on the trace corpus)
pub fn _months(n: i64) -> Value {
    crate::unported!("bot.memory.saving:_months")
}

/// Python: bot/memory/saving.py:1324 `a_date_on_a_settled_rate(text, memory)` (never called on the trace corpus)
pub fn a_date_on_a_settled_rate(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:a_date_on_a_settled_rate")
}

/// Python: bot/memory/saving.py:1340 `_a_date_on_a_rate(goal, rate, when)` (never called on the trace corpus)
pub fn _a_date_on_a_rate(goal: &Value, rate: &Value, when: &str) -> String {
    crate::unported!("bot.memory.saving:_a_date_on_a_rate")
}

/// Python: bot/memory/saving.py:1386 `asks_if_the_rate_is_enough(text)`
pub fn asks_if_the_rate_is_enough(text: &str) -> bool {
    crate::unported!("bot.memory.saving:asks_if_the_rate_is_enough")
}

/// Python: bot/memory/saving.py:1391 `a_yes_no_about_the_rate(text, memory)`
pub fn a_yes_no_about_the_rate(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:a_yes_no_about_the_rate")
}

/// Python: bot/memory/saving.py:1438 `asks_how_much_more_a_month(text)`
pub fn asks_how_much_more_a_month(text: &str) -> bool {
    crate::unported!("bot.memory.saving:asks_how_much_more_a_month")
}

/// Python: bot/memory/saving.py:1443 `the_extra_a_month_asked(text, memory)`
pub fn the_extra_a_month_asked(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:the_extra_a_month_asked")
}

/// Python: bot/memory/saving.py:1475 `the_same_figure_a_month(text, memory)`
pub fn the_same_figure_a_month(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:the_same_figure_a_month")
}

/// Python: bot/memory/saving.py:1511 `a_rate_restated(text, memory)`
pub fn a_rate_restated(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:a_rate_restated")
}

/// Python: bot/memory/saving.py:1558 `a_rate_put_in_with_a_verb(text, memory)`
pub fn a_rate_put_in_with_a_verb(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:a_rate_put_in_with_a_verb")
}

/// Python: bot/memory/saving.py:1606 `a_length_at_the_rate_in_the_chat(text, memory)`
pub fn a_length_at_the_rate_in_the_chat(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:a_length_at_the_rate_in_the_chat")
}

/// Python: bot/memory/saving.py:1630 `a_rate_with_no_date(text, memory)`
pub fn a_rate_with_no_date(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:a_rate_with_no_date")
}

/// Python: bot/memory/saving.py:1648 `asks_the_rate(text, memory)`
pub fn asks_the_rate(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.saving:asks_the_rate")
}

/// Python: bot/memory/saving.py:1668 `_in_the_money_readers_words(when)` (never called on the trace corpus)
pub fn _in_the_money_readers_words(when: &str) -> Value {
    crate::unported!("bot.memory.saving:_in_the_money_readers_words")
}

/// Python: bot/memory/saving.py:1700 `rate_question(text, memory)` (never called on the trace corpus)
pub fn rate_question(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:rate_question")
}

/// Python: bot/memory/saving.py:1767 `_a_rate_said(line)`
pub fn _a_rate_said(line: &str) -> Value {
    crate::unported!("bot.memory.saving:_a_rate_said")
}

/// Python: bot/memory/saving.py:1803 `a_rate_for_the_goal(text, memory)`
pub fn a_rate_for_the_goal(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:a_rate_for_the_goal")
}

/// Python: bot/memory/saving.py:1861 `_the_added_sum(line)` (never called on the trace corpus)
pub fn _the_added_sum(line: &str) -> Value {
    crate::unported!("bot.memory.saving:_the_added_sum")
}

/// Python: bot/memory/saving.py:1878 `a_rate_raised(text, memory)`
pub fn a_rate_raised(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:a_rate_raised")
}

/// Python: bot/memory/saving.py:1922 `_said_a_rate_at(rate, memory)` (never called on the trace corpus)
pub fn _said_a_rate_at(rate: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.memory.saving:_said_a_rate_at")
}

/// Python: bot/memory/saving.py:1931 `_a_rate_against_the_goal(goal, rate, memory)` (never called on the trace corpus)
pub fn _a_rate_against_the_goal(goal: &Value, rate: &Value, memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:_a_rate_against_the_goal")
}

/// Python: bot/memory/saving.py:1952 `_a_closer(memory)` (never called on the trace corpus)
pub fn _a_closer(memory: &Memory) -> Value {
    crate::unported!("bot.memory.saving:_a_closer")
}

/// Python: bot/memory/saving.py:1969 `_a_said_sentence(sentence, said)` (never called on the trace corpus)
pub fn _a_said_sentence(sentence: &str, said: &str) -> bool {
    crate::unported!("bot.memory.saving:_a_said_sentence")
}

/// Python: bot/memory/saving.py:1978 `said_already(reply, memory)` (never called on the trace corpus)
pub fn said_already(reply: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.saving:said_already")
}

/// Python: bot/memory/saving.py:2003 `_weeks_left(when)` (never called on the trace corpus)
pub fn _weeks_left(when: &str) -> Value {
    crate::unported!("bot.memory.saving:_weeks_left")
}

/// Python: bot/memory/saving.py:2021 `a_figure_each_week(memory)` (never called on the trace corpus)
pub fn a_figure_each_week(memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:a_figure_each_week")
}

/// Python: bot/memory/saving.py:2056 `the_figure_a_week(memory)` (never called on the trace corpus)
pub fn the_figure_a_week(memory: &Memory) -> Value {
    crate::unported!("bot.memory.saving:the_figure_a_week")
}

/// Python: bot/memory/saving.py:2063 `without_what_was_just_said(reply, memory)` (never called on the trace corpus)
pub fn without_what_was_just_said(reply: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.saving:without_what_was_just_said")
}

/// Python: bot/memory/saving.py:2134 `_the_rate_put_by(memory)` (never called on the trace corpus)
pub fn _the_rate_put_by(memory: &Memory) -> Value {
    crate::unported!("bot.memory.saving:_the_rate_put_by")
}

/// Python: bot/memory/saving.py:2160 `a_length_with_no_price(text, memory)`
pub fn a_length_with_no_price(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:a_length_with_no_price")
}

/// Python: bot/memory/saving.py:2194 `_already_asked_for_the_price(memory)` (never called on the trace corpus)
pub fn _already_asked_for_the_price(memory: &Memory) -> bool {
    crate::unported!("bot.memory.saving:_already_asked_for_the_price")
}

/// Python: bot/memory/saving.py:2204 `a_price_in_an_answer(text, memory)`
pub fn a_price_in_an_answer(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.saving:a_price_in_an_answer")
}

