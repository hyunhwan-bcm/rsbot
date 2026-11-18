//! Port of `bot/compose/hands.py`.
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

/// Python: bot/compose/hands.py:47 `_bare(text)`
pub fn _bare(text: &str) -> String {
    crate::unported!("bot.compose.hands:_bare")
}

/// Python: bot/compose/hands.py:196 `reads(text)`
pub fn reads(text: &str) {
    crate::unported!("bot.compose.hands:reads")
}

/// Python: bot/compose/hands.py:230 `writes_the_words(text)`
pub fn writes_the_words(text: &str) -> bool {
    crate::unported!("bot.compose.hands:writes_the_words")
}

/// Python: bot/compose/hands.py:245 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.hands:reply_for")
}

