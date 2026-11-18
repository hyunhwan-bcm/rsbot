//! Port of `bot/memory/outgoings.py`.
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

/// Python: bot/memory/outgoings.py:206 `_choice(shape, line, memory=None)` (never called on the trace corpus)
pub fn _choice(shape: &Value, line: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.outgoings:_choice")
}

/// Python: bot/memory/outgoings.py:227 `_the_names(costs)` (never called on the trace corpus)
pub fn _the_names(costs: &Value) -> Value {
    crate::unported!("bot.memory.outgoings:_the_names")
}

/// Python: bot/memory/outgoings.py:241 `_sums(text)`
pub fn _sums(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.outgoings:_sums")
}

/// Python: bot/memory/outgoings.py:279 `_period(text)`
pub fn _period(text: &str) -> String {
    crate::unported!("bot.memory.outgoings:_period")
}

/// Python: bot/memory/outgoings.py:290 `_units_in(text)`
pub fn _units_in(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.outgoings:_units_in")
}

/// Python: bot/memory/outgoings.py:299 `_pounds(value)` (never called on the trace corpus)
pub fn _pounds(value: &Value) -> String {
    crate::unported!("bot.memory.outgoings:_pounds")
}

/// Python: bot/memory/outgoings.py:306 `_with_unit(value, unit)` (never called on the trace corpus)
pub fn _with_unit(value: &Value, unit: &str) -> Value {
    crate::unported!("bot.memory.outgoings:_with_unit")
}

/// Python: bot/memory/outgoings.py:312 `_in_phrase(text, start, end, pattern, span=18)` (never called on the trace corpus)
pub fn _in_phrase(text: &str, start: i64, end: i64, pattern: &Value, span: &Value) -> bool {
    crate::unported!("bot.memory.outgoings:_in_phrase")
}

/// Python: bot/memory/outgoings.py:317 `_near(text, start, end, pattern, before=30, after=30)` (never called on the trace corpus)
pub fn _near(text: &str, start: i64, end: i64, pattern: &Value, before: &str, after: &str) -> bool {
    crate::unported!("bot.memory.outgoings:_near")
}

/// Python: bot/memory/outgoings.py:323 `_the_nearest(line, cue, sums, after=20, before=6)`
pub fn _the_nearest(line: &str, cue: &Value, sums: &[String], after: i64, before: i64) {
    crate::unported!("bot.memory.outgoings:_the_nearest")
}

/// Python: bot/memory/outgoings.py:345 `_the_incomes_in_the_line(line, sums=None)`
pub fn _the_incomes_in_the_line(line: &str, sums: Option<&[String]>) -> Vec<String> {
    crate::unported!("bot.memory.outgoings:_the_incomes_in_the_line")
}

/// Python: bot/memory/outgoings.py:366 `the_income_in_the_line(text)`
pub fn the_income_in_the_line(text: &str) {
    crate::unported!("bot.memory.outgoings:the_income_in_the_line")
}

/// Python: bot/memory/outgoings.py:382 `_the_cost_the_figure_is_for(line, start, end)` (never called on the trace corpus)
pub fn _the_cost_the_figure_is_for(line: &str, start: i64, end: i64) -> Value {
    crate::unported!("bot.memory.outgoings:_the_cost_the_figure_is_for")
}

/// Python: bot/memory/outgoings.py:408 `_the_unit_beside(line, end)` (never called on the trace corpus)
pub fn _the_unit_beside(line: &str, end: i64) -> Value {
    crate::unported!("bot.memory.outgoings:_the_unit_beside")
}

/// Python: bot/memory/outgoings.py:418 `the_costs_with_units_in_the_line(text)`
pub fn the_costs_with_units_in_the_line(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.outgoings:the_costs_with_units_in_the_line")
}

/// Python: bot/memory/outgoings.py:450 `the_outgoings_in_the_line(text)` (never called on the trace corpus)
pub fn the_outgoings_in_the_line(text: &str) -> Value {
    crate::unported!("bot.memory.outgoings:the_outgoings_in_the_line")
}

/// Python: bot/memory/outgoings.py:455 `_there_is_memory(memory)`
pub fn _there_is_memory(memory: &Memory) -> bool {
    crate::unported!("bot.memory.outgoings:_there_is_memory")
}

/// Python: bot/memory/outgoings.py:472 `_the_income_before(memory)`
pub fn _the_income_before(memory: &Memory) {
    crate::unported!("bot.memory.outgoings:_the_income_before")
}

/// Python: bot/memory/outgoings.py:495 `_the_costs_before(memory)` (never called on the trace corpus)
pub fn _the_costs_before(memory: &Memory) -> Value {
    crate::unported!("bot.memory.outgoings:_the_costs_before")
}

/// Python: bot/memory/outgoings.py:518 `_remember(memory, left, unit)` (never called on the trace corpus)
pub fn _remember(memory: &mut Memory, left: &Value, unit: &str) {
    crate::unported!("bot.memory.outgoings:_remember")
}

/// Python: bot/memory/outgoings.py:524 `_the_pot_in_the_line(text)` (never called on the trace corpus)
pub fn _the_pot_in_the_line(text: &str) -> Value {
    crate::unported!("bot.memory.outgoings:_the_pot_in_the_line")
}

/// Python: bot/memory/outgoings.py:535 `_the_left_again(line, memory)` (never called on the trace corpus)
pub fn _the_left_again(line: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.outgoings:_the_left_again")
}

/// Python: bot/memory/outgoings.py:548 `_the_ask_points_back(line)` (never called on the trace corpus)
pub fn _the_ask_points_back(line: &str) -> bool {
    crate::unported!("bot.memory.outgoings:_the_ask_points_back")
}

/// Python: bot/memory/outgoings.py:555 `_the_month_was_just_set_out(memory)`
pub fn _the_month_was_just_set_out(memory: &Memory) -> bool {
    crate::unported!("bot.memory.outgoings:_the_month_was_just_set_out")
}

/// Python: bot/memory/outgoings.py:566 `_the_unit_in_the_chat(memory)` (never called on the trace corpus)
pub fn _the_unit_in_the_chat(memory: &Memory) -> Value {
    crate::unported!("bot.memory.outgoings:_the_unit_in_the_chat")
}

/// Python: bot/memory/outgoings.py:579 `_the_chat_is_about_saving(memory)` (never called on the trace corpus)
pub fn _the_chat_is_about_saving(memory: &Memory) -> bool {
    crate::unported!("bot.memory.outgoings:_the_chat_is_about_saving")
}

/// Python: bot/memory/outgoings.py:592 `reply_for(text, memory=None)`
pub fn reply_for(text: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.memory.outgoings:reply_for")
}

