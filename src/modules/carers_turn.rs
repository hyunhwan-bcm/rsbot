//! Port of `bot/modules/carers_turn.py`.
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

/// Python: bot/modules/carers_turn.py:388 `_clean(text)`
pub fn _clean(text: &str) -> String {
    crate::unported!("bot.modules.carers_turn:_clean")
}

/// Python: bot/modules/carers_turn.py:392 `_turns(memory)`
pub fn _turns(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.modules.carers_turn:_turns")
}

/// Python: bot/modules/carers_turn.py:399 `_the_answer_above(memory)`
pub fn _the_answer_above(memory: &Memory) {
    crate::unported!("bot.modules.carers_turn:_the_answer_above")
}

/// Python: bot/modules/carers_turn.py:424 `_reads_as_easing(clean)` (never called on the trace corpus)
pub fn _reads_as_easing(clean: &str) -> bool {
    crate::unported!("bot.modules.carers_turn:_reads_as_easing")
}

/// Python: bot/modules/carers_turn.py:431 `_reads_as_easing_but_worse(clean)` (never called on the trace corpus)
pub fn _reads_as_easing_but_worse(clean: &str) -> bool {
    crate::unported!("bot.modules.carers_turn:_reads_as_easing_but_worse")
}

/// Python: bot/modules/carers_turn.py:438 `_an_abuse_thread(last)`
pub fn _an_abuse_thread(last: &str) -> bool {
    crate::unported!("bot.modules.carers_turn:_an_abuse_thread")
}

/// Python: bot/modules/carers_turn.py:443 `_pick(pool, forms, clean, memory=None)` (never called on the trace corpus)
pub fn _pick(pool: &Value, forms: &[String], clean: &str, memory: &Memory) -> Value {
    crate::unported!("bot.modules.carers_turn:_pick")
}

/// Python: bot/modules/carers_turn.py:466 `_the_thread(memory, last)` (never called on the trace corpus)
pub fn _the_thread(memory: &Memory, last: &str) -> String {
    crate::unported!("bot.modules.carers_turn:_the_thread")
}

/// Python: bot/modules/carers_turn.py:475 `_the_answer_sent_them_somewhere(last)` (never called on the trace corpus)
pub fn _the_answer_sent_them_somewhere(last: &str) -> bool {
    crate::unported!("bot.modules.carers_turn:_the_answer_sent_them_somewhere")
}

/// Python: bot/modules/carers_turn.py:480 `_the_words_for_the_one_who_did_it(memory)` (never called on the trace corpus)
pub fn _the_words_for_the_one_who_did_it(memory: &Memory) -> Value {
    crate::unported!("bot.modules.carers_turn:_the_words_for_the_one_who_did_it")
}

/// Python: bot/modules/carers_turn.py:563 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.carers_turn:respond")
}

