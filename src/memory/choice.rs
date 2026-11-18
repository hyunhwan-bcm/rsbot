//! Port of `bot/memory/choice.py`.
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

/// Python: bot/memory/choice.py:60 `_clean_option(piece)`
pub fn _clean_option(piece: &str) -> String {
    crate::unported!("bot.memory.choice:_clean_option")
}

/// Python: bot/memory/choice.py:75 `_from_italics(text)`
pub fn _from_italics(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.choice:_from_italics")
}

/// Python: bot/memory/choice.py:90 `_segment(text)`
pub fn _segment(text: &str) -> String {
    crate::unported!("bot.memory.choice:_segment")
}

/// Python: bot/memory/choice.py:100 `_from_list(text)`
pub fn _from_list(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.choice:_from_list")
}

/// Python: bot/memory/choice.py:140 `_from_labels(text)`
pub fn _from_labels(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.choice:_from_labels")
}

/// Python: bot/memory/choice.py:156 `_from_question(text)`
pub fn _from_question(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.choice:_from_question")
}

/// Python: bot/memory/choice.py:169 `options_in(reply)`
pub fn options_in(reply: &str) -> Vec<String> {
    crate::unported!("bot.memory.choice:options_in")
}

/// Python: bot/memory/choice.py:188 `asks_for_a_pick(text)`
pub fn asks_for_a_pick(text: &str) -> bool {
    crate::unported!("bot.memory.choice:asks_for_a_pick")
}

/// Python: bot/memory/choice.py:210 `_from_user_line(memory)` (never called on the trace corpus)
pub fn _from_user_line(memory: &Memory) -> Value {
    crate::unported!("bot.memory.choice:_from_user_line")
}

/// Python: bot/memory/choice.py:243 `_asked_for_help(memory, n=4)` (never called on the trace corpus)
pub fn _asked_for_help(memory: &Memory, n: i64) -> bool {
    crate::unported!("bot.memory.choice:_asked_for_help")
}

/// Python: bot/memory/choice.py:314 `_peeled(option)` (never called on the trace corpus)
pub fn _peeled(option: &Value) -> Value {
    crate::unported!("bot.memory.choice:_peeled")
}

/// Python: bot/memory/choice.py:331 `_readable(options)` (never called on the trace corpus)
pub fn _readable(options: &Value) -> Value {
    crate::unported!("bot.memory.choice:_readable")
}

/// Python: bot/memory/choice.py:347 `_already_picked(memory, back=3)` (never called on the trace corpus)
pub fn _already_picked(memory: &Memory, back: &Value) -> Value {
    crate::unported!("bot.memory.choice:_already_picked")
}

/// Python: bot/memory/choice.py:363 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.choice:reply_for")
}

/// Python: bot/memory/choice.py:520 `_the_thing_in(line)` (never called on the trace corpus)
pub fn _the_thing_in(line: &str) -> Value {
    crate::unported!("bot.memory.choice:_the_thing_in")
}

/// Python: bot/memory/choice.py:586 `_hands_the_choice_over(before)` (never called on the trace corpus)
pub fn _hands_the_choice_over(before: &str) -> bool {
    crate::unported!("bot.memory.choice:_hands_the_choice_over")
}

/// Python: bot/memory/choice.py:594 `settled_on(memory)` (never called on the trace corpus)
pub fn settled_on(memory: &Memory) -> Value {
    crate::unported!("bot.memory.choice:settled_on")
}

/// Python: bot/memory/choice.py:614 `_bare_change(line)` (never called on the trace corpus)
pub fn _bare_change(line: &str) -> String {
    crate::unported!("bot.memory.choice:_bare_change")
}

/// Python: bot/memory/choice.py:627 `_a_choice_in_hand(memory, this)` (never called on the trace corpus)
pub fn _a_choice_in_hand(memory: &Memory, this: &Value) -> bool {
    crate::unported!("bot.memory.choice:_a_choice_in_hand")
}

/// Python: bot/memory/choice.py:638 `the_decision(text, memory)`
pub fn the_decision(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.choice:the_decision")
}

/// Python: bot/memory/choice.py:650 `_a_way_of_saying_it(thing, memory)` (never called on the trace corpus)
pub fn _a_way_of_saying_it(thing: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.choice:_a_way_of_saying_it")
}

/// Python: bot/memory/choice.py:658 `offered(memory, n=1)` (never called on the trace corpus)
pub fn offered(memory: &Memory, n: i64) -> Value {
    crate::unported!("bot.memory.choice:offered")
}

