//! Port of `bot/memory/enough.py`.
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

/// Python: bot/memory/enough.py:62 `_a_value(word)` (never called on the trace corpus)
pub fn _a_value(word: &str) -> Value {
    crate::unported!("bot.memory.enough:_a_value")
}

/// Python: bot/memory/enough.py:70 `_amount_family(unit)` (never called on the trace corpus)
pub fn _amount_family(unit: &str) -> Value {
    crate::unported!("bot.memory.enough:_amount_family")
}

/// Python: bot/memory/enough.py:92 `_period_after(text, at)` (never called on the trace corpus)
pub fn _period_after(text: &str, at: i64) -> Value {
    crate::unported!("bot.memory.enough:_period_after")
}

/// Python: bot/memory/enough.py:100 `_a_period_near(text, phrase)` (never called on the trace corpus)
pub fn _a_period_near(text: &str, phrase: &str) -> Value {
    crate::unported!("bot.memory.enough:_a_period_near")
}

/// Python: bot/memory/enough.py:108 `_the_period_of_the_answer(answer)` (never called on the trace corpus)
pub fn _the_period_of_the_answer(answer: &str) -> Value {
    crate::unported!("bot.memory.enough:_the_period_of_the_answer")
}

/// Python: bot/memory/enough.py:115 `_the_periods_the_answer_says(answer)` (never called on the trace corpus)
pub fn _the_periods_the_answer_says(answer: &str) -> Value {
    crate::unported!("bot.memory.enough:_the_periods_the_answer_says")
}

/// Python: bot/memory/enough.py:131 `_periods_disagree(mine, theirs, also=())` (never called on the trace corpus)
pub fn _periods_disagree(mine: &Memory, theirs: &Value, also: &Value) -> bool {
    crate::unported!("bot.memory.enough:_periods_disagree")
}

/// Python: bot/memory/enough.py:157 `_too_far_to_compare(value, low, high)` (never called on the trace corpus)
pub fn _too_far_to_compare(value: &Value, low: &str, high: &Value) -> bool {
    crate::unported!("bot.memory.enough:_too_far_to_compare")
}

/// Python: bot/memory/enough.py:168 `_amounts(text)`
pub fn _amounts(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.enough:_amounts")
}

/// Python: bot/memory/enough.py:198 `_target(answer, family, mine=None, bare_ok=False, period=None)` (never called on the trace corpus)
pub fn _target(answer: &str, family: &Value, mine: &Memory, bare_ok: &Value, period: &Value) -> Value {
    crate::unported!("bot.memory.enough:_target")
}

/// Python: bot/memory/enough.py:337 `_the_aim_that_is_their_number(answer, family, mine)` (never called on the trace corpus)
pub fn _the_aim_that_is_their_number(answer: &str, family: &Value, mine: &Memory) -> Value {
    crate::unported!("bot.memory.enough:_the_aim_that_is_their_number")
}

/// Python: bot/memory/enough.py:367 `_with_its_period(answer, figure, period)` (never called on the trace corpus)
pub fn _with_its_period(answer: &str, figure: &Value, period: &Value) -> Value {
    crate::unported!("bot.memory.enough:_with_its_period")
}

/// Python: bot/memory/enough.py:388 `_the_periods_of(answer, figures)` (never called on the trace corpus)
pub fn _the_periods_of(answer: &str, figures: &Value) -> Value {
    crate::unported!("bot.memory.enough:_the_periods_of")
}

/// Python: bot/memory/enough.py:404 `_aims_named(answer, mine, family)` (never called on the trace corpus)
pub fn _aims_named(answer: &str, mine: &Memory, family: &Value) -> Value {
    crate::unported!("bot.memory.enough:_aims_named")
}

/// Python: bot/memory/enough.py:463 `_counts_things(answer, at)` (never called on the trace corpus)
pub fn _counts_things(answer: &str, at: i64) -> bool {
    crate::unported!("bot.memory.enough:_counts_things")
}

/// Python: bot/memory/enough.py:487 `_figures_in(answer)` (never called on the trace corpus)
pub fn _figures_in(answer: &str) -> Value {
    crate::unported!("bot.memory.enough:_figures_in")
}

/// Python: bot/memory/enough.py:502 `_a_rate(answer, end)` (never called on the trace corpus)
pub fn _a_rate(answer: &str, end: i64) -> bool {
    crate::unported!("bot.memory.enough:_a_rate")
}

/// Python: bot/memory/enough.py:514 `_another_kind_follows(answer, end, family)` (never called on the trace corpus)
pub fn _another_kind_follows(answer: &str, end: i64, family: &Value) -> bool {
    crate::unported!("bot.memory.enough:_another_kind_follows")
}

/// Python: bot/memory/enough.py:524 `_near_bare_range(answer, mine, family=None)` (never called on the trace corpus)
pub fn _near_bare_range(answer: &str, mine: &Memory, family: &Value) -> Value {
    crate::unported!("bot.memory.enough:_near_bare_range")
}

/// Python: bot/memory/enough.py:546 `_with_its_unit(aim, family, high, answer='')` (never called on the trace corpus)
pub fn _with_its_unit(aim: &Value, family: &Value, high: &Value, answer: &str) -> Value {
    crate::unported!("bot.memory.enough:_with_its_unit")
}

/// Python: bot/memory/enough.py:573 `_the_number(value)` (never called on the trace corpus)
pub fn _the_number(value: &Value) -> String {
    crate::unported!("bot.memory.enough:_the_number")
}

/// Python: bot/memory/enough.py:578 `enough_reply(text, memory)`
pub fn enough_reply(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.enough:enough_reply")
}

/// Python: bot/memory/enough.py:629 `_enough_bare(turns, text, more, too_much)`
pub fn _enough_bare(turns: &Value, text: &str, more: bool, too_much: bool) {
    crate::unported!("bot.memory.enough:_enough_bare")
}

/// Python: bot/memory/enough.py:667 `_range_near(answer, mine)` (never called on the trace corpus)
pub fn _range_near(answer: &str, mine: &Memory) -> Value {
    crate::unported!("bot.memory.enough:_range_near")
}

/// Python: bot/memory/enough.py:689 `_bare_figures(text)`
pub fn _bare_figures(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.enough:_bare_figures")
}

/// Python: bot/memory/enough.py:704 `_candidate_answers(turns, mine_values)` (never called on the trace corpus)
pub fn _candidate_answers(turns: &[Turn], mine_values: &Value) -> Value {
    crate::unported!("bot.memory.enough:_candidate_answers")
}

/// Python: bot/memory/enough.py:724 `_enough_against(answers, given, mine_values, said, more, too_much, quoted_ok=False)` (never called on the trace corpus)
pub fn _enough_against(answers: &Value, given: &Value, mine_values: &Value, said: &str, more: &Value, too_much: &Value, quoted_ok: &Value) -> Value {
    crate::unported!("bot.memory.enough:_enough_against")
}

