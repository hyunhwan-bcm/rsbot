//! Port of `bot/memory/limit.py`.
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

/// Python: bot/memory/limit.py:159 `words_of(text)`
pub fn words_of(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.limit:words_of")
}

/// Python: bot/memory/limit.py:176 `_a_fault_in_the_thing(said, found)` (never called on the trace corpus)
pub fn _a_fault_in_the_thing(said: &str, found: &Value) -> bool {
    crate::unported!("bot.memory.limit:_a_fault_in_the_thing")
}

/// Python: bot/memory/limit.py:186 `_shown(match)` (never called on the trace corpus)
pub fn _shown(r#match: &Value) -> Value {
    crate::unported!("bot.memory.limit:_shown")
}

/// Python: bot/memory/limit.py:226 `_without_a_reason(said)` (never called on the trace corpus)
pub fn _without_a_reason(said: &str) -> String {
    crate::unported!("bot.memory.limit:_without_a_reason")
}

/// Python: bot/memory/limit.py:239 `_a_pet_in_the_reason(said, found)` (never called on the trace corpus)
pub fn _a_pet_in_the_reason(said: &str, found: &Value) -> Value {
    crate::unported!("bot.memory.limit:_a_pet_in_the_reason")
}

/// Python: bot/memory/limit.py:246 `_bare(said)` (never called on the trace corpus)
pub fn _bare(said: &str) -> Value {
    crate::unported!("bot.memory.limit:_bare")
}

/// Python: bot/memory/limit.py:253 `_the_read(said, memory)`
pub fn _the_read(said: &str, memory: &Memory) -> Understood {
    crate::unported!("bot.memory.limit:_the_read")
}

/// Python: bot/memory/limit.py:261 `reads_as_a_limit(text, memory=None)`
pub fn reads_as_a_limit(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.limit:reads_as_a_limit")
}

/// Python: bot/memory/limit.py:344 `only_a_limit(text, memory=None)` (never called on the trace corpus)
pub fn only_a_limit(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.limit:only_a_limit")
}

/// Python: bot/memory/limit.py:367 `_everyone_in_the_house(line, found)` (never called on the trace corpus)
pub fn _everyone_in_the_house(line: &str, found: &Value) -> String {
    crate::unported!("bot.memory.limit:_everyone_in_the_house")
}

/// Python: bot/memory/limit.py:388 `_pieces(answer)` (never called on the trace corpus)
pub fn _pieces(answer: &str) -> Value {
    crate::unported!("bot.memory.limit:_pieces")
}

/// Python: bot/memory/limit.py:400 `_steps(answer)` (never called on the trace corpus)
pub fn _steps(answer: &str) -> Value {
    crate::unported!("bot.memory.limit:_steps")
}

/// Python: bot/memory/limit.py:419 `_names(clause, thing)` (never called on the trace corpus)
pub fn _names(clause: &str, thing: &str) -> bool {
    crate::unported!("bot.memory.limit:_names")
}

/// Python: bot/memory/limit.py:441 `_first_with(answer, thing)` (never called on the trace corpus)
pub fn _first_with(answer: &str, thing: &str) -> Value {
    crate::unported!("bot.memory.limit:_first_with")
}

/// Python: bot/memory/limit.py:476 `_the_amount_of_it(clause, thing)` (never called on the trace corpus)
pub fn _the_amount_of_it(clause: &str, thing: &str) -> bool {
    crate::unported!("bot.memory.limit:_the_amount_of_it")
}

/// Python: bot/memory/limit.py:489 `_among_measured_things(clause, thing)` (never called on the trace corpus)
pub fn _among_measured_things(clause: &str, thing: &str) -> bool {
    crate::unported!("bot.memory.limit:_among_measured_things")
}

/// Python: bot/memory/limit.py:508 `is_a_thing_on_a_list(clause, thing='')` (never called on the trace corpus)
pub fn is_a_thing_on_a_list(clause: &str, thing: &str) -> bool {
    crate::unported!("bot.memory.limit:is_a_thing_on_a_list")
}

/// Python: bot/memory/limit.py:533 `after_an_answer(memory)`
pub fn after_an_answer(memory: &Memory) -> bool {
    crate::unported!("bot.memory.limit:after_an_answer")
}

/// Python: bot/memory/limit.py:538 `_last_answer(memory)` (never called on the trace corpus)
pub fn _last_answer(memory: &Memory) -> Value {
    crate::unported!("bot.memory.limit:_last_answer")
}

/// Python: bot/memory/limit.py:548 `reads_the_answer(text, memory)`
pub fn reads_the_answer(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.limit:reads_the_answer")
}

/// Python: bot/memory/limit.py:580 `_the_line_answered(memory)` (never called on the trace corpus)
pub fn _the_line_answered(memory: &Memory) -> String {
    crate::unported!("bot.memory.limit:_the_line_answered")
}

/// Python: bot/memory/limit.py:589 `_a_fault_in_the_line_answered(memory)` (never called on the trace corpus)
pub fn _a_fault_in_the_line_answered(memory: &Memory) -> bool {
    crate::unported!("bot.memory.limit:_a_fault_in_the_line_answered")
}

/// Python: bot/memory/limit.py:595 `_about_the_job(bare, memory)` (never called on the trace corpus)
pub fn _about_the_job(bare: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.memory.limit:_about_the_job")
}

/// Python: bot/memory/limit.py:613 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.limit:reply_for")
}

/// Python: bot/memory/limit.py:628 `move_after_a_limit(text, memory)`
pub fn move_after_a_limit(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.limit:move_after_a_limit")
}

/// Python: bot/memory/limit.py:656 `_names_no_thing(line)` (never called on the trace corpus)
pub fn _names_no_thing(line: &str) -> bool {
    crate::unported!("bot.memory.limit:_names_no_thing")
}

/// Python: bot/memory/limit.py:668 `_steps_the_limit_left(text, memory)` (never called on the trace corpus)
pub fn _steps_the_limit_left(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.limit:_steps_the_limit_left")
}

/// Python: bot/memory/limit.py:680 `a_detail_of_a_fault(memory, bare)` (never called on the trace corpus)
pub fn a_detail_of_a_fault(memory: &Memory, bare: &Value) -> bool {
    crate::unported!("bot.memory.limit:a_detail_of_a_fault")
}

/// Python: bot/memory/limit.py:718 `_without_the_appliance(thing, answer, memory)` (never called on the trace corpus)
pub fn _without_the_appliance(thing: &str, answer: &str, memory: &Memory) -> Option<String> {
    crate::unported!("bot.memory.limit:_without_the_appliance")
}

/// Python: bot/memory/limit.py:738 `_limit_reply(text, memory)`
pub fn _limit_reply(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.limit:_limit_reply")
}

/// Python: bot/memory/limit.py:809 `_kept(answer, bare)` (never called on the trace corpus)
pub fn _kept(answer: &str, bare: &Value) -> Value {
    crate::unported!("bot.memory.limit:_kept")
}

/// Python: bot/memory/limit.py:840 `_without_a_repeated_time(sentences)` (never called on the trace corpus)
pub fn _without_a_repeated_time(sentences: &Value) -> Value {
    crate::unported!("bot.memory.limit:_without_a_repeated_time")
}

/// Python: bot/memory/limit.py:850 `_up_to_words(sentences)` (never called on the trace corpus)
pub fn _up_to_words(sentences: &Value) -> Value {
    crate::unported!("bot.memory.limit:_up_to_words")
}

/// Python: bot/memory/limit.py:862 `_holds_on(answer, bare)` (never called on the trace corpus)
pub fn _holds_on(answer: &str, bare: &Value) -> bool {
    crate::unported!("bot.memory.limit:_holds_on")
}

/// Python: bot/memory/limit.py:868 `_needs_it(thing, answer)` (never called on the trace corpus)
pub fn _needs_it(thing: &str, answer: &str) -> String {
    crate::unported!("bot.memory.limit:_needs_it")
}

/// Python: bot/memory/limit.py:878 `_bare_phrase(named, bare)` (never called on the trace corpus)
pub fn _bare_phrase(named: &Value, bare: &Value) -> Value {
    crate::unported!("bot.memory.limit:_bare_phrase")
}

/// Python: bot/memory/limit.py:887 `_with_a_pet(answer, who)` (never called on the trace corpus)
pub fn _with_a_pet(answer: &str, who: &str) -> Option<String> {
    crate::unported!("bot.memory.limit:_with_a_pet")
}

/// Python: bot/memory/limit.py:899 `_say(lead, steps)` (never called on the trace corpus)
pub fn _say(lead: &str, steps: &Value) -> String {
    crate::unported!("bot.memory.limit:_say")
}

/// Python: bot/memory/limit.py:913 `_tidy(step)` (never called on the trace corpus)
pub fn _tidy(step: &Value) -> String {
    crate::unported!("bot.memory.limit:_tidy")
}

