//! Port of `bot/memory/lineback.py`.
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

/// Python: bot/memory/lineback.py:148 `flat(text)`
pub fn flat(text: &str) -> String {
    crate::unported!("bot.memory.lineback:flat")
}

/// Python: bot/memory/lineback.py:153 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.lineback:_words")
}

/// Python: bot/memory/lineback.py:157 `_stem(word)`
pub fn _stem(word: &str) -> String {
    crate::unported!("bot.memory.lineback:_stem")
}

/// Python: bot/memory/lineback.py:165 `about(text)`
pub fn about(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.lineback:about")
}

/// Python: bot/memory/lineback.py:196 `_a_kind_named(text)` (never called on the trace corpus)
pub fn _a_kind_named(text: &str) -> Value {
    crate::unported!("bot.memory.lineback:_a_kind_named")
}

/// Python: bot/memory/lineback.py:204 `is_a_recall_ask(text)`
pub fn is_a_recall_ask(text: &str) -> bool {
    crate::unported!("bot.memory.lineback:is_a_recall_ask")
}

/// Python: bot/memory/lineback.py:223 `_the_last_reply(memory)` (never called on the trace corpus)
pub fn _the_last_reply(memory: &Memory) -> String {
    crate::unported!("bot.memory.lineback:_the_last_reply")
}

/// Python: bot/memory/lineback.py:232 `the_lines(memory)`
pub fn the_lines(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.memory.lineback:the_lines")
}

/// Python: bot/memory/lineback.py:245 `_the_best_line(wanted, lines, questions=False)`
pub fn _the_best_line(wanted: &[String], lines: &[String], questions: bool) {
    crate::unported!("bot.memory.lineback:_the_best_line")
}

/// Python: bot/memory/lineback.py:265 `about_any(text)`
pub fn about_any(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.lineback:about_any")
}

/// Python: bot/memory/lineback.py:301 `_the_kind_asked(said)`
pub fn _the_kind_asked(said: &str) {
    crate::unported!("bot.memory.lineback:_the_kind_asked")
}

/// Python: bot/memory/lineback.py:318 `_a_value_of_the_kind(kind, lines)` (never called on the trace corpus)
pub fn _a_value_of_the_kind(kind: &str, lines: &[String]) -> Value {
    crate::unported!("bot.memory.lineback:_a_value_of_the_kind")
}

/// Python: bot/memory/lineback.py:335 `_the_value(line)` (never called on the trace corpus)
pub fn _the_value(line: &str) -> Value {
    crate::unported!("bot.memory.lineback:_the_value")
}

/// Python: bot/memory/lineback.py:346 `_carried(line, wanted)` (never called on the trace corpus)
pub fn _carried(line: &str, wanted: &Value) -> Value {
    crate::unported!("bot.memory.lineback:_carried")
}

/// Python: bot/memory/lineback.py:355 `_the_value_of_the_kind(said, lines, memory)`
pub fn _the_value_of_the_kind(said: &str, lines: &[String], memory: &Memory) {
    crate::unported!("bot.memory.lineback:_the_value_of_the_kind")
}

/// Python: bot/memory/lineback.py:380 `_the_figure_for_a_thing(said, lines, memory)`
pub fn _the_figure_for_a_thing(said: &str, lines: &[String], memory: &Memory) {
    crate::unported!("bot.memory.lineback:_the_figure_for_a_thing")
}

/// Python: bot/memory/lineback.py:401 `_the_line_itself(said, lines, memory)`
pub fn _the_line_itself(said: &str, lines: &[String], memory: &Memory) {
    crate::unported!("bot.memory.lineback:_the_line_itself")
}

/// Python: bot/memory/lineback.py:441 `_the_kind_they_gave(said, lines, memory)`
pub fn _the_kind_they_gave(said: &str, lines: &[String], memory: &Memory) {
    crate::unported!("bot.memory.lineback:_the_kind_they_gave")
}

/// Python: bot/memory/lineback.py:457 `the_kind_they_gave(text, memory)`
pub fn the_kind_they_gave(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.lineback:the_kind_they_gave")
}

/// Python: bot/memory/lineback.py:471 `an_earlier_turn(text, memory)`
pub fn an_earlier_turn(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.lineback:an_earlier_turn")
}

/// Python: bot/memory/lineback.py:494 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.lineback:reply_for")
}

