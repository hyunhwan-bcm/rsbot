//! Port of `bot/compose/what_i_do_know.py`.
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

/// Python: bot/compose/what_i_do_know.py:394 `_parts_of(said)`
pub fn _parts_of(said: &str) -> String {
    crate::unported!("bot.compose.what_i_do_know:_parts_of")
}

/// Python: bot/compose/what_i_do_know.py:416 `_kind_of(said)`
pub fn _kind_of(said: &str) -> String {
    crate::unported!("bot.compose.what_i_do_know:_kind_of")
}

/// Python: bot/compose/what_i_do_know.py:440 `_that(thing)`
pub fn _that(thing: &str) -> String {
    crate::unported!("bot.compose.what_i_do_know:_that")
}

/// Python: bot/compose/what_i_do_know.py:481 `_without_a_particle(phrase)`
pub fn _without_a_particle(phrase: &str) -> String {
    crate::unported!("bot.compose.what_i_do_know:_without_a_particle")
}

/// Python: bot/compose/what_i_do_know.py:488 `_a_thing_from(said, determiner_only=False)`
pub fn _a_thing_from(said: &str, determiner_only: bool) -> String {
    crate::unported!("bot.compose.what_i_do_know:_a_thing_from")
}

/// Python: bot/compose/what_i_do_know.py:542 `_thing_named_in(text)`
pub fn _thing_named_in(text: &str) -> String {
    crate::unported!("bot.compose.what_i_do_know:_thing_named_in")
}

/// Python: bot/compose/what_i_do_know.py:559 `_the_thing(read, said, memory)`
pub fn _the_thing(read: &Understood, said: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.what_i_do_know:_the_thing")
}

/// Python: bot/compose/what_i_do_know.py:617 `before_text_from(memory)`
pub fn before_text_from(memory: &Memory) -> String {
    crate::unported!("bot.compose.what_i_do_know:before_text_from")
}

/// Python: bot/compose/what_i_do_know.py:622 `_the_line_before(memory)`
pub fn _the_line_before(memory: &Memory) {
    crate::unported!("bot.compose.what_i_do_know:_the_line_before")
}

/// Python: bot/compose/what_i_do_know.py:712 `_kind_from_the_owning_line(memory, thing, said='')`
pub fn _kind_from_the_owning_line(memory: &Memory, thing: &str, said: &str) -> String {
    crate::unported!("bot.compose.what_i_do_know:_kind_from_the_owning_line")
}

/// Python: bot/compose/what_i_do_know.py:763 `_known(kind, part)`
pub fn _known(kind: &str, part: &str) -> Vec<String> {
    crate::unported!("bot.compose.what_i_do_know:_known")
}

/// Python: bot/compose/what_i_do_know.py:779 `reply_for(text, memory=None, reply='')`
pub fn reply_for(text: &str, memory: &Memory, reply: &str) {
    crate::unported!("bot.compose.what_i_do_know:reply_for")
}

