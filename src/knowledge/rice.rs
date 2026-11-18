//! Port of `bot/knowledge/rice.py`.
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

/// Python: bot/knowledge/rice.py:179 `_said(understood)` (never called on the trace corpus)
pub fn _said(understood: &Understood) -> String {
    crate::unported!("bot.knowledge.rice:_said")
}

/// Python: bot/knowledge/rice.py:187 `_sort_named(said, names=None)` (never called on the trace corpus)
pub fn _sort_named(said: &str, names: &[String]) -> Value {
    crate::unported!("bot.knowledge.rice:_sort_named")
}

/// Python: bot/knowledge/rice.py:208 `_dish_recent(memory)` (never called on the trace corpus)
pub fn _dish_recent(memory: &Memory) -> Value {
    crate::unported!("bot.knowledge.rice:_dish_recent")
}

/// Python: bot/knowledge/rice.py:223 `_dish_named(said)` (never called on the trace corpus)
pub fn _dish_named(said: &str) -> Value {
    crate::unported!("bot.knowledge.rice:_dish_named")
}

/// Python: bot/knowledge/rice.py:231 `_swap(memory, understood)` (never called on the trace corpus)
pub fn _swap(memory: &Memory, understood: &Understood) -> Value {
    crate::unported!("bot.knowledge.rice:_swap")
}

/// Python: bot/knowledge/rice.py:310 `_dish(memory, understood)` (never called on the trace corpus)
pub fn _dish(memory: &Memory, understood: &Understood) -> Value {
    crate::unported!("bot.knowledge.rice:_dish")
}

/// Python: bot/knowledge/rice.py:315 `_swap_reply(memory, understood)` (never called on the trace corpus)
pub fn _swap_reply(memory: &Memory, understood: &Understood) -> Value {
    crate::unported!("bot.knowledge.rice:_swap_reply")
}

/// Python: bot/knowledge/rice.py:319 `_dish_reply(memory, understood)` (never called on the trace corpus)
pub fn _dish_reply(memory: &Memory, understood: &Understood) -> Value {
    crate::unported!("bot.knowledge.rice:_dish_reply")
}

