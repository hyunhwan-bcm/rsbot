//! Port of `bot/compose/ruledout.py`.
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

/// Python: bot/compose/ruledout.py:219 `_named_picks(answer)` (never called on the trace corpus)
pub fn _named_picks(answer: &str) -> bool {
    crate::unported!("bot.compose.ruledout:_named_picks")
}

/// Python: bot/compose/ruledout.py:224 `_class_of(word)` (never called on the trace corpus)
pub fn _class_of(word: &str) -> Value {
    crate::unported!("bot.compose.ruledout:_class_of")
}

/// Python: bot/compose/ruledout.py:256 `_a_class_word(word)` (never called on the trace corpus)
pub fn _a_class_word(word: &str) -> bool {
    crate::unported!("bot.compose.ruledout:_a_class_word")
}

/// Python: bot/compose/ruledout.py:269 `_an_answer_of_that_class(answer, thing)` (never called on the trace corpus)
pub fn _an_answer_of_that_class(answer: &str, thing: &str) -> bool {
    crate::unported!("bot.compose.ruledout:_an_answer_of_that_class")
}

/// Python: bot/compose/ruledout.py:328 `_without_the_manners(clean)`
pub fn _without_the_manners(clean: &str) -> String {
    crate::unported!("bot.compose.ruledout:_without_the_manners")
}

/// Python: bot/compose/ruledout.py:361 `_ruled_out_thing(clean)`
pub fn _ruled_out_thing(clean: &str) -> Value {
    crate::unported!("bot.compose.ruledout:_ruled_out_thing")
}

/// Python: bot/compose/ruledout.py:416 `_a_row_thing(thing)` (never called on the trace corpus)
pub fn _a_row_thing(thing: &str) -> Value {
    crate::unported!("bot.compose.ruledout:_a_row_thing")
}

/// Python: bot/compose/ruledout.py:427 `_names_it(answer, thing)` (never called on the trace corpus)
pub fn _names_it(answer: &str, thing: &str) -> bool {
    crate::unported!("bot.compose.ruledout:_names_it")
}

/// Python: bot/compose/ruledout.py:451 `_the_whole_answer(memory)` (never called on the trace corpus)
pub fn _the_whole_answer(memory: &Memory) -> Value {
    crate::unported!("bot.compose.ruledout:_the_whole_answer")
}

/// Python: bot/compose/ruledout.py:476 `_the_standing_part(answer)` (never called on the trace corpus)
pub fn _the_standing_part(answer: &str) -> Value {
    crate::unported!("bot.compose.ruledout:_the_standing_part")
}

/// Python: bot/compose/ruledout.py:542 `_a_claim_of_failure(clean, memory)`
pub fn _a_claim_of_failure(clean: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.ruledout:_a_claim_of_failure")
}

/// Python: bot/compose/ruledout.py:607 `_a_pet_turned_down(clean, memory)` (never called on the trace corpus)
pub fn _a_pet_turned_down(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.ruledout:_a_pet_turned_down")
}

/// Python: bot/compose/ruledout.py:636 `_a_list_that_can_be_swapped(answer, thing)` (never called on the trace corpus)
pub fn _a_list_that_can_be_swapped(answer: &str, thing: &str) -> bool {
    crate::unported!("bot.compose.ruledout:_a_list_that_can_be_swapped")
}

/// Python: bot/compose/ruledout.py:644 `_rules_out_what_i_never_said(text, memory)`
pub fn _rules_out_what_i_never_said(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.ruledout:_rules_out_what_i_never_said")
}

/// Python: bot/compose/ruledout.py:665 `_ruled_out_reply(memory, clean)`
pub fn _ruled_out_reply(memory: &Memory, clean: &str) -> String {
    crate::unported!("bot.compose.ruledout:_ruled_out_reply")
}

/// Python: bot/compose/ruledout.py:685 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.ruledout:reply_for")
}

/// Python: bot/compose/ruledout.py:698 `takes_as_it_comes(text, memory)` (never called on the trace corpus)
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> Option<bool> {
    crate::unported!("bot.compose.ruledout:takes_as_it_comes")
}

