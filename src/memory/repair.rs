//! Port of `bot/memory/repair.py`.
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

/// Python: bot/memory/repair.py:156 `_the_thing(text)`
pub fn _the_thing(text: &str) {
    crate::unported!("bot.memory.repair:_the_thing")
}

/// Python: bot/memory/repair.py:217 `_a_different_thing(old, thing)` (never called on the trace corpus)
pub fn _a_different_thing(old: &Value, thing: &str) -> bool {
    crate::unported!("bot.memory.repair:_a_different_thing")
}

/// Python: bot/memory/repair.py:233 `_content_words(text)` (never called on the trace corpus)
pub fn _content_words(text: &str) -> Value {
    crate::unported!("bot.memory.repair:_content_words")
}

/// Python: bot/memory/repair.py:246 `_the_reading(line)` (never called on the trace corpus)
pub fn _the_reading(line: &str) -> Value {
    crate::unported!("bot.memory.repair:_the_reading")
}

/// Python: bot/memory/repair.py:264 `_the_thing_the_reading_named(line)` (never called on the trace corpus)
pub fn _the_thing_the_reading_named(line: &str) -> Value {
    crate::unported!("bot.memory.repair:_the_thing_the_reading_named")
}

/// Python: bot/memory/repair.py:305 `_the_thing_the_last_line_named(line)` (never called on the trace corpus)
pub fn _the_thing_the_last_line_named(line: &str) -> Value {
    crate::unported!("bot.memory.repair:_the_thing_the_last_line_named")
}

/// Python: bot/memory/repair.py:335 `_the_noun_the_line_named(line)` (never called on the trace corpus)
pub fn _the_noun_the_line_named(line: &str) -> Value {
    crate::unported!("bot.memory.repair:_the_noun_the_line_named")
}

/// Python: bot/memory/repair.py:354 `_the_pointer_refines(thing, line)` (never called on the trace corpus)
pub fn _the_pointer_refines(thing: &str, line: &str) -> Value {
    crate::unported!("bot.memory.repair:_the_pointer_refines")
}

/// Python: bot/memory/repair.py:373 `_the_relation(word)` (never called on the trace corpus)
pub fn _the_relation(word: &str) -> Value {
    crate::unported!("bot.memory.repair:_the_relation")
}

/// Python: bot/memory/repair.py:395 `_the_person_the_line_named(line, thing)` (never called on the trace corpus)
pub fn _the_person_the_line_named(line: &str, thing: &str) -> Value {
    crate::unported!("bot.memory.repair:_the_person_the_line_named")
}

/// Python: bot/memory/repair.py:415 `_swapped(line, old, thing)` (never called on the trace corpus)
pub fn _swapped(line: &str, old: &Value, thing: &str) -> Value {
    crate::unported!("bot.memory.repair:_swapped")
}

/// Python: bot/memory/repair.py:439 `_no_repeat(line)` (never called on the trace corpus)
pub fn _no_repeat(line: &str) -> String {
    crate::unported!("bot.memory.repair:_no_repeat")
}

/// Python: bot/memory/repair.py:456 `_the_answer(line, memory)` (never called on the trace corpus)
pub fn _the_answer(line: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.memory.repair:_the_answer")
}

/// Python: bot/memory/repair.py:494 `_the_plain_answer(text, memory)` (never called on the trace corpus)
pub fn _the_plain_answer(text: &str, memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.repair:_the_plain_answer")
}

/// Python: bot/memory/repair.py:506 `_about_the_same_thing(first, second)` (never called on the trace corpus)
pub fn _about_the_same_thing(first: &Value, second: &Value) -> bool {
    crate::unported!("bot.memory.repair:_about_the_same_thing")
}

/// Python: bot/memory/repair.py:520 `_the_shape_the_swap_had(memory, line, old, thing)` (never called on the trace corpus)
pub fn _the_shape_the_swap_had(memory: &Memory, line: &str, old: &Value, thing: &str) -> Value {
    crate::unported!("bot.memory.repair:_the_shape_the_swap_had")
}

/// Python: bot/memory/repair.py:549 `_a_state_not_a_thing(line, tail)` (never called on the trace corpus)
pub fn _a_state_not_a_thing(line: &str, tail: &str) -> bool {
    crate::unported!("bot.memory.repair:_a_state_not_a_thing")
}

/// Python: bot/memory/repair.py:561 `_keep_the_swap(memory, line, old, thing)` (never called on the trace corpus)
pub fn _keep_the_swap(memory: &mut Memory, line: &str, old: &Value, thing: &str) {
    crate::unported!("bot.memory.repair:_keep_the_swap")
}

/// Python: bot/memory/repair.py:594 `_the_paragraph_already_given(answer, memory, old, thing)` (never called on the trace corpus)
pub fn _the_paragraph_already_given(answer: &str, memory: &Memory, old: &Value, thing: &str) -> bool {
    crate::unported!("bot.memory.repair:_the_paragraph_already_given")
}

/// Python: bot/memory/repair.py:636 `_names(words, wanted)` (never called on the trace corpus)
pub fn _names(words: &[String], wanted: &Value) -> bool {
    crate::unported!("bot.memory.repair:_names")
}

/// Python: bot/memory/repair.py:652 `_without_the_repeat_lead(answer)` (never called on the trace corpus)
pub fn _without_the_repeat_lead(answer: &str) -> Value {
    crate::unported!("bot.memory.repair:_without_the_repeat_lead")
}

/// Python: bot/memory/repair.py:658 `_still_the_old_thing(answer, old, thing)` (never called on the trace corpus)
pub fn _still_the_old_thing(answer: &str, old: &Value, thing: &str) -> bool {
    crate::unported!("bot.memory.repair:_still_the_old_thing")
}

/// Python: bot/memory/repair.py:679 `_bare(thing)` (never called on the trace corpus)
pub fn _bare(thing: &str) -> String {
    crate::unported!("bot.memory.repair:_bare")
}

/// Python: bot/memory/repair.py:688 `_said_before(answer, memory)` (never called on the trace corpus)
pub fn _said_before(answer: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.repair:_said_before")
}

/// Python: bot/memory/repair.py:737 `_the_same_words(one, other)` (never called on the trace corpus)
pub fn _the_same_words(one: &str, other: &str) -> bool {
    crate::unported!("bot.memory.repair:_the_same_words")
}

/// Python: bot/memory/repair.py:751 `_the_figure_in(line)` (never called on the trace corpus)
pub fn _the_figure_in(line: &str) -> Value {
    crate::unported!("bot.memory.repair:_the_figure_in")
}

/// Python: bot/memory/repair.py:765 `_the_figure_they_meant(text, memory)`
pub fn _the_figure_they_meant(text: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.memory.repair:_the_figure_they_meant")
}

/// Python: bot/memory/repair.py:822 `_the_answer_says(said, figure)` (never called on the trace corpus)
pub fn _the_answer_says(said: &str, figure: &Value) -> bool {
    crate::unported!("bot.memory.repair:_the_answer_says")
}

/// Python: bot/memory/repair.py:834 `_the_figures_that_moved(said, redone)` (never called on the trace corpus)
pub fn _the_figures_that_moved(said: &str, redone: &Value) -> String {
    crate::unported!("bot.memory.repair:_the_figures_that_moved")
}

/// Python: bot/memory/repair.py:857 `_keep_the_corrected_answer(memory, scratch, line)` (never called on the trace corpus)
pub fn _keep_the_corrected_answer(memory: &mut Memory, scratch: &Value, line: &str) {
    crate::unported!("bot.memory.repair:_keep_the_corrected_answer")
}

/// Python: bot/memory/repair.py:946 `_a_fact_of_the_person(entry)` (never called on the trace corpus)
pub fn _a_fact_of_the_person(entry: &'static Entry) -> String {
    crate::unported!("bot.memory.repair:_a_fact_of_the_person")
}

/// Python: bot/memory/repair.py:957 `_a_list_of(things)` (never called on the trace corpus)
pub fn _a_list_of(things: &Value) -> Value {
    crate::unported!("bot.memory.repair:_a_list_of")
}

/// Python: bot/memory/repair.py:967 `_the_fact_moved_to_the_person_they_meant(memory, old, thing, text)` (never called on the trace corpus)
pub fn _the_fact_moved_to_the_person_they_meant(memory: &mut Memory, old: &Value, thing: &str, text: &str) -> Value {
    crate::unported!("bot.memory.repair:_the_fact_moved_to_the_person_they_meant")
}

/// Python: bot/memory/repair.py:993 `_the_fact_moved_line(fact, old, thing, text, memory)` (never called on the trace corpus)
pub fn _the_fact_moved_line(fact: &str, old: &Value, thing: &str, text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.repair:_the_fact_moved_line")
}

/// Python: bot/memory/repair.py:1002 `_the_swap_in_the_line(text, memory)`
pub fn _the_swap_in_the_line(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.repair:_the_swap_in_the_line")
}

/// Python: bot/memory/repair.py:1037 `keep_the_correction(text, memory)`
pub fn keep_the_correction(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.repair:keep_the_correction")
}

/// Python: bot/memory/repair.py:1060 `the_thing_they_meant(text, memory)`
pub fn the_thing_they_meant(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.repair:the_thing_they_meant")
}

