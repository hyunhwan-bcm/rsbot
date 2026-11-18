//! Port of `bot/memory/kinds.py`.
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

/// Python: bot/memory/kinds.py:170 `_the_other_end(ask, line)` (never called on the trace corpus)
pub fn _the_other_end(ask: &str, line: &str) -> bool {
    crate::unported!("bot.memory.kinds:_the_other_end")
}

/// Python: bot/memory/kinds.py:183 `_flat(text)`
pub fn _flat(text: &str) -> String {
    crate::unported!("bot.memory.kinds:_flat")
}

/// Python: bot/memory/kinds.py:187 `_words(line, length=2)` (never called on the trace corpus)
pub fn _words(line: &str, length: &Value) -> Value {
    crate::unported!("bot.memory.kinds:_words")
}

/// Python: bot/memory/kinds.py:200 `_mentions(line, wanted)` (never called on the trace corpus)
pub fn _mentions(line: &str, wanted: &Value) -> bool {
    crate::unported!("bot.memory.kinds:_mentions")
}

/// Python: bot/memory/kinds.py:212 `_their_lines(memory)` (never called on the trace corpus)
pub fn _their_lines(memory: &Memory) -> Value {
    crate::unported!("bot.memory.kinds:_their_lines")
}

/// Python: bot/memory/kinds.py:237 `the_kind_asked(text)`
pub fn the_kind_asked(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.kinds:the_kind_asked")
}

/// Python: bot/memory/kinds.py:243 `the_line_that_gives(text, lines)`
pub fn the_line_that_gives(text: &str, lines: &[String]) {
    crate::unported!("bot.memory.kinds:the_line_that_gives")
}

