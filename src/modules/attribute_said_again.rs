//! Port of `bot/modules/attribute_said_again.py`.
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

/// Python: bot/modules/attribute_said_again.py:128 `_their_attribute(low)`
pub fn _their_attribute(low: &str) -> String {
    crate::unported!("bot.modules.attribute_said_again:_their_attribute")
}

/// Python: bot/modules/attribute_said_again.py:137 `_stem(word)`
pub fn _stem(word: &str) -> String {
    crate::unported!("bot.modules.attribute_said_again:_stem")
}

/// Python: bot/modules/attribute_said_again.py:146 `_names(tokens, attribute)`
pub fn _names(tokens: &[String], attribute: &str) -> bool {
    crate::unported!("bot.modules.attribute_said_again:_names")
}

/// Python: bot/modules/attribute_said_again.py:152 `_the_phrase_of_theirs(attribute, memory, needs_a_figure=False)`
pub fn _the_phrase_of_theirs(attribute: &str, memory: &Memory, needs_a_figure: bool) -> String {
    crate::unported!("bot.modules.attribute_said_again:_the_phrase_of_theirs")
}

/// Python: bot/modules/attribute_said_again.py:182 `_a_cost_line(line)` (never called on the trace corpus)
pub fn _a_cost_line(line: &str) -> bool {
    crate::unported!("bot.modules.attribute_said_again:_a_cost_line")
}

/// Python: bot/modules/attribute_said_again.py:188 `_the_newest_cost(memory)` (never called on the trace corpus)
pub fn _the_newest_cost(memory: &Memory) -> Value {
    crate::unported!("bot.modules.attribute_said_again:_the_newest_cost")
}

/// Python: bot/modules/attribute_said_again.py:200 `_the_cost_just_before(memory)` (never called on the trace corpus)
pub fn _the_cost_just_before(memory: &Memory) -> Value {
    crate::unported!("bot.modules.attribute_said_again:_the_cost_just_before")
}

/// Python: bot/modules/attribute_said_again.py:213 `_phrase(line)` (never called on the trace corpus)
pub fn _phrase(line: &str) -> Value {
    crate::unported!("bot.modules.attribute_said_again:_phrase")
}

/// Python: bot/modules/attribute_said_again.py:226 `_the_value(phrase)` (never called on the trace corpus)
pub fn _the_value(phrase: &str) -> String {
    crate::unported!("bot.modules.attribute_said_again:_the_value")
}

/// Python: bot/modules/attribute_said_again.py:368 `_the_slot_ask(low)`
pub fn _the_slot_ask(low: &str) -> Value {
    crate::unported!("bot.modules.attribute_said_again:_the_slot_ask")
}

/// Python: bot/modules/attribute_said_again.py:394 `_the_subject(rest)`
pub fn _the_subject(rest: &str) -> Vec<String> {
    crate::unported!("bot.modules.attribute_said_again:_the_subject")
}

/// Python: bot/modules/attribute_said_again.py:400 `_a_time(line)` (never called on the trace corpus)
pub fn _a_time(line: &str) -> Value {
    crate::unported!("bot.modules.attribute_said_again:_a_time")
}

/// Python: bot/modules/attribute_said_again.py:420 `_a_place(line)` (never called on the trace corpus)
pub fn _a_place(line: &str) -> String {
    crate::unported!("bot.modules.attribute_said_again:_a_place")
}

/// Python: bot/modules/attribute_said_again.py:431 `_a_size(line, subject)`
pub fn _a_size(line: &str, subject: &[String]) -> String {
    crate::unported!("bot.modules.attribute_said_again:_a_size")
}

/// Python: bot/modules/attribute_said_again.py:451 `_the_size_recall(subject, lines)`
pub fn _the_size_recall(subject: &[String], lines: &[String]) -> String {
    crate::unported!("bot.modules.attribute_said_again:_the_size_recall")
}

/// Python: bot/modules/attribute_said_again.py:475 `_the_money_recall(what, lines)` (never called on the trace corpus)
pub fn _the_money_recall(what: &str, lines: &[String]) -> String {
    crate::unported!("bot.modules.attribute_said_again:_the_money_recall")
}

/// Python: bot/modules/attribute_said_again.py:543 `_the_value_kind(attribute, how)` (never called on the trace corpus)
pub fn _the_value_kind(attribute: &Value, how: &str) -> Value {
    crate::unported!("bot.modules.attribute_said_again:_the_value_kind")
}

/// Python: bot/modules/attribute_said_again.py:564 `_the_line_that_says_it(text, memory)`
pub fn _the_line_that_says_it(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.attribute_said_again:_the_line_that_says_it")
}

/// Python: bot/modules/attribute_said_again.py:592 `_the_values_the_kind_takes(text, line)` (never called on the trace corpus)
pub fn _the_values_the_kind_takes(text: &str, line: &str) -> Value {
    crate::unported!("bot.modules.attribute_said_again:_the_values_the_kind_takes")
}

/// Python: bot/modules/attribute_said_again.py:619 `_the_value_said(text, memory)`
pub fn _the_value_said(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.attribute_said_again:_the_value_said")
}

/// Python: bot/modules/attribute_said_again.py:644 `_the_slot_recall(text, memory)`
pub fn _the_slot_recall(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.attribute_said_again:_the_slot_recall")
}

/// Python: bot/modules/attribute_said_again.py:700 `_the_age_they_gave(turns, low)` (never called on the trace corpus)
pub fn _the_age_they_gave(turns: &[Turn], low: &str) -> String {
    crate::unported!("bot.modules.attribute_said_again:_the_age_they_gave")
}

/// Python: bot/modules/attribute_said_again.py:743 `_the_off_with_they_gave(turns)` (never called on the trace corpus)
pub fn _the_off_with_they_gave(turns: &[Turn]) -> String {
    crate::unported!("bot.modules.attribute_said_again:_the_off_with_they_gave")
}

/// Python: bot/modules/attribute_said_again.py:757 `_the_habit_they_gave(turns)` (never called on the trace corpus)
pub fn _the_habit_they_gave(turns: &[Turn]) -> String {
    crate::unported!("bot.modules.attribute_said_again:_the_habit_they_gave")
}

/// Python: bot/modules/attribute_said_again.py:769 `_the_slot_reply(value, memory)` (never called on the trace corpus)
pub fn _the_slot_reply(value: &Value, memory: &Memory) -> Value {
    crate::unported!("bot.modules.attribute_said_again:_the_slot_reply")
}

/// Python: bot/modules/attribute_said_again.py:777 `_the_last_reply(memory)` (never called on the trace corpus)
pub fn _the_last_reply(memory: &Memory) -> String {
    crate::unported!("bot.modules.attribute_said_again:_the_last_reply")
}

/// Python: bot/modules/attribute_said_again.py:784 `_spoken(phrase)` (never called on the trace corpus)
pub fn _spoken(phrase: &str) -> Value {
    crate::unported!("bot.modules.attribute_said_again:_spoken")
}

/// Python: bot/modules/attribute_said_again.py:806 `_the_ask_names_something_new(low, memory)` (never called on the trace corpus)
pub fn _the_ask_names_something_new(low: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.attribute_said_again:_the_ask_names_something_new")
}

/// Python: bot/modules/attribute_said_again.py:823 `_the_recall(text, memory)`
pub fn _the_recall(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.attribute_said_again:_the_recall")
}

/// Python: bot/modules/attribute_said_again.py:878 `takes_as_it_comes(text, memory)` (never called on the trace corpus)
pub fn takes_as_it_comes(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.attribute_said_again:takes_as_it_comes")
}

/// Python: bot/modules/attribute_said_again.py:904 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.attribute_said_again:respond")
}

