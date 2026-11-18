//! Port of `bot/modules/chasing_a_body.py`.
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

/// Python: bot/modules/chasing_a_body.py:196 `_the_shape_unnamed(memory, lowered)` (never called on the trace corpus)
pub fn _the_shape_unnamed(memory: &Memory, lowered: &str) -> Value {
    crate::unported!("bot.modules.chasing_a_body:_the_shape_unnamed")
}

/// Python: bot/modules/chasing_a_body.py:380 `_the_person_before(memory)` (never called on the trace corpus)
pub fn _the_person_before(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chasing_a_body:_the_person_before")
}

/// Python: bot/modules/chasing_a_body.py:390 `_the_wait_for_a_person(memory, lowered)`
pub fn _the_wait_for_a_person(memory: &Memory, lowered: &str) {
    crate::unported!("bot.modules.chasing_a_body:_the_wait_for_a_person")
}

/// Python: bot/modules/chasing_a_body.py:409 `_was_an_unnamed_chase(memory, current)` (never called on the trace corpus)
pub fn _was_an_unnamed_chase(memory: &Memory, current: &Value) -> bool {
    crate::unported!("bot.modules.chasing_a_body:_was_an_unnamed_chase")
}

/// Python: bot/modules/chasing_a_body.py:421 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    crate::unported!("bot.modules.chasing_a_body:_words")
}

/// Python: bot/modules/chasing_a_body.py:425 `_the_body(lowered, words)`
pub fn _the_body(lowered: &str, words: &[String]) -> Option<Vec<String>> {
    crate::unported!("bot.modules.chasing_a_body:_the_body")
}

/// Python: bot/modules/chasing_a_body.py:470 `_names_its_own_person(lowered)` (never called on the trace corpus)
pub fn _names_its_own_person(lowered: &str) -> bool {
    crate::unported!("bot.modules.chasing_a_body:_names_its_own_person")
}

/// Python: bot/modules/chasing_a_body.py:475 `_the_person_it_belongs_to(lowered, trigger)`
pub fn _the_person_it_belongs_to(lowered: &str, trigger: &str) {
    crate::unported!("bot.modules.chasing_a_body:_the_person_it_belongs_to")
}

/// Python: bot/modules/chasing_a_body.py:493 `_the_body_before(memory, current)`
pub fn _the_body_before(memory: &Memory, current: &str) {
    crate::unported!("bot.modules.chasing_a_body:_the_body_before")
}

/// Python: bot/modules/chasing_a_body.py:511 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.chasing_a_body:respond")
}

/// Python: bot/modules/chasing_a_body.py:572 `_the_timescale(body, memory, lowered)` (never called on the trace corpus)
pub fn _the_timescale(body: &str, memory: &Memory, lowered: &str) -> String {
    crate::unported!("bot.modules.chasing_a_body:_the_timescale")
}

/// Python: bot/modules/chasing_a_body.py:581 `_the_next_step(body, memory, lowered)` (never called on the trace corpus)
pub fn _the_next_step(body: &str, memory: &Memory, lowered: &str) -> String {
    crate::unported!("bot.modules.chasing_a_body:_the_next_step")
}

/// Python: bot/modules/chasing_a_body.py:591 `_the_next_step_unnamed(memory, lowered)` (never called on the trace corpus)
pub fn _the_next_step_unnamed(memory: &Memory, lowered: &str) -> String {
    crate::unported!("bot.modules.chasing_a_body:_the_next_step_unnamed")
}

/// Python: bot/modules/chasing_a_body.py:599 `_the_shape(body, memory, lowered)` (never called on the trace corpus)
pub fn _the_shape(body: &str, memory: &Memory, lowered: &str) -> Value {
    crate::unported!("bot.modules.chasing_a_body:_the_shape")
}

