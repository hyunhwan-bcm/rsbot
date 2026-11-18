//! Port of `bot/modules/judge_ask.py`.
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

/// Python: bot/modules/judge_ask.py:82 `_says_nothing(said)` (never called on the trace corpus)
pub fn _says_nothing(said: &str) -> bool {
    crate::unported!("bot.modules.judge_ask:_says_nothing")
}

/// Python: bot/modules/judge_ask.py:92 `_the_composer_says_nothing(text, memory)` (never called on the trace corpus)
pub fn _the_composer_says_nothing(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.judge_ask:_the_composer_says_nothing")
}

/// Python: bot/modules/judge_ask.py:105 `reads_it(text)`
pub fn reads_it(text: &str) -> bool {
    crate::unported!("bot.modules.judge_ask:reads_it")
}

/// Python: bot/modules/judge_ask.py:116 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.judge_ask:respond")
}

