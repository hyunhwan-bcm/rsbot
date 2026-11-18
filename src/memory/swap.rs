//! Port of `bot/memory/swap.py`.
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

/// Python: bot/memory/swap.py:57 `_after_a_take_back(said)`
pub fn _after_a_take_back(said: &str) {
    crate::unported!("bot.memory.swap:_after_a_take_back")
}

/// Python: bot/memory/swap.py:125 `_said(text)`
pub fn _said(text: &str) -> String {
    crate::unported!("bot.memory.swap:_said")
}

/// Python: bot/memory/swap.py:129 `_stem(word)` (never called on the trace corpus)
pub fn _stem(word: &str) -> Value {
    crate::unported!("bot.memory.swap:_stem")
}

/// Python: bot/memory/swap.py:137 `thing(text)`
pub fn thing(text: &str) {
    crate::unported!("bot.memory.swap:thing")
}

/// Python: bot/memory/swap.py:229 `someone(text)`
pub fn someone(text: &str) {
    crate::unported!("bot.memory.swap:someone")
}

/// Python: bot/memory/swap.py:257 `_earlier(memory, text, n=4, wide=False)` (never called on the trace corpus)
pub fn _earlier(memory: &Memory, text: &str, n: i64, wide: &Value) -> Value {
    crate::unported!("bot.memory.swap:_earlier")
}

/// Python: bot/memory/swap.py:291 `_about(question)` (never called on the trace corpus)
pub fn _about(question: &str) -> Option<String> {
    crate::unported!("bot.memory.swap:_about")
}

/// Python: bot/memory/swap.py:324 `_reads_as_a_question(line)` (never called on the trace corpus)
pub fn _reads_as_a_question(line: &str) -> bool {
    crate::unported!("bot.memory.swap:_reads_as_a_question")
}

/// Python: bot/memory/swap.py:340 `_word(question, answer, new_words, taken_up)` (never called on the trace corpus)
pub fn _word(question: &str, answer: &str, new_words: &[String], taken_up: &Value) -> Value {
    crate::unported!("bot.memory.swap:_word")
}

/// Python: bot/memory/swap.py:355 `_replace(question, word, new)` (never called on the trace corpus)
pub fn _replace(question: &str, word: &str, new: &Value) -> Value {
    crate::unported!("bot.memory.swap:_replace")
}

/// Python: bot/memory/swap.py:384 `_spans(question, subject)` (never called on the trace corpus)
pub fn _spans(question: &str, subject: &str) -> Value {
    crate::unported!("bot.memory.swap:_spans")
}

/// Python: bot/memory/swap.py:420 `is_repair(text)`
pub fn is_repair(text: &str) -> bool {
    crate::unported!("bot.memory.swap:is_repair")
}

/// Python: bot/memory/swap.py:426 `candidates(text, memory, most=3)` (never called on the trace corpus)
pub fn candidates(text: &str, memory: &Memory, most: &Value) -> Value {
    crate::unported!("bot.memory.swap:candidates")
}

/// Python: bot/memory/swap.py:475 `rewritten(text, memory)`
pub fn rewritten(text: &str, memory: &Value) {
    crate::unported!("bot.memory.swap:rewritten")
}

/// Python: bot/memory/swap.py:540 `_named_somebody(memory, text, n=4)` (never called on the trace corpus)
pub fn _named_somebody(memory: &Memory, text: &str, n: i64) -> Value {
    crate::unported!("bot.memory.swap:_named_somebody")
}

/// Python: bot/memory/swap.py:564 `rewritten_person(text, memory)`
pub fn rewritten_person(text: &str, memory: &Value) {
    crate::unported!("bot.memory.swap:rewritten_person")
}

/// Python: bot/memory/swap.py:593 `names_person(old, answer, text, last='')` (never called on the trace corpus)
pub fn names_person(old: &Value, answer: &str, text: &str, last: &str) -> bool {
    crate::unported!("bot.memory.swap:names_person")
}

/// Python: bot/memory/swap.py:624 `_the_preposition_of(new)` (never called on the trace corpus)
pub fn _the_preposition_of(new: &Value) -> Value {
    crate::unported!("bot.memory.swap:_the_preposition_of")
}

/// Python: bot/memory/swap.py:633 `_the_place_after_the_same_preposition(question, new)` (never called on the trace corpus)
pub fn _the_place_after_the_same_preposition(question: &str, new: &Value) -> Option<String> {
    crate::unported!("bot.memory.swap:_the_place_after_the_same_preposition")
}

/// Python: bot/memory/swap.py:664 `_a_second_place(question, line, new)` (never called on the trace corpus)
pub fn _a_second_place(question: &str, line: &str, new: &Value) -> bool {
    crate::unported!("bot.memory.swap:_a_second_place")
}

/// Python: bot/memory/swap.py:682 `placed(question, old, new)` (never called on the trace corpus)
pub fn placed(question: &str, old: &Value, new: &Value) -> Value {
    crate::unported!("bot.memory.swap:placed")
}

/// Python: bot/memory/swap.py:729 `_said_again(answer, memory)` (never called on the trace corpus)
pub fn _said_again(answer: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.swap:_said_again")
}

/// Python: bot/memory/swap.py:748 `_brings_its_own_place(new)` (never called on the trace corpus)
pub fn _brings_its_own_place(new: &Value) -> bool {
    crate::unported!("bot.memory.swap:_brings_its_own_place")
}

/// Python: bot/memory/swap.py:754 `names(old, answer, text, memory=None, question=None)` (never called on the trace corpus)
pub fn names(old: &Value, answer: &str, text: &str, memory: &Memory, question: &str) -> bool {
    crate::unported!("bot.memory.swap:names")
}

