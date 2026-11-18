//! Port of `bot/modules/words.py`.
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

/// Python: bot/modules/words.py:103 `_quoted(said)` (never called on the trace corpus)
pub fn _quoted(said: &str) -> String {
    crate::unported!("bot.modules.words:_quoted")
}

/// Python: bot/modules/words.py:144 `_names_its_own(line)` (never called on the trace corpus)
pub fn _names_its_own(line: &str) -> bool {
    crate::unported!("bot.modules.words:_names_its_own")
}

/// Python: bot/modules/words.py:168 `_the_words(text, memory)`
pub fn _the_words(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.words:_the_words")
}

/// Python: bot/modules/words.py:178 `TAKES_AS_IT_COMES(text, memory)` (never called on the trace corpus)
pub fn TAKES_AS_IT_COMES(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.words:TAKES_AS_IT_COMES")
}

/// Python: bot/modules/words.py:189 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.words:respond")
}

