//! Port of `bot/memory/tried.py`.
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

/// Python: bot/memory/tried.py:354 `words_of(text)`
pub fn words_of(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.tried:words_of")
}

/// Python: bot/memory/tried.py:358 `reads_as_not_worked(text)`
pub fn reads_as_not_worked(text: &str) -> bool {
    crate::unported!("bot.memory.tried:reads_as_not_worked")
}

/// Python: bot/memory/tried.py:406 `the_still_state(text, memory)`
pub fn the_still_state(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.tried:the_still_state")
}

/// Python: bot/memory/tried.py:437 `still_the_state(text, memory)`
pub fn still_the_state(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.tried:still_the_state")
}

/// Python: bot/memory/tried.py:447 `might_be_a_report(text)`
pub fn might_be_a_report(text: &str) -> bool {
    crate::unported!("bot.memory.tried:might_be_a_report")
}

/// Python: bot/memory/tried.py:456 `names_a_trade(text)`
pub fn names_a_trade(text: &str) -> bool {
    crate::unported!("bot.memory.tried:names_a_trade")
}

/// Python: bot/memory/tried.py:466 `after_an_answer(memory)`
pub fn after_an_answer(memory: &Memory) -> bool {
    crate::unported!("bot.memory.tried:after_an_answer")
}

/// Python: bot/memory/tried.py:491 `last_answer_words(memory)` (never called on the trace corpus)
pub fn last_answer_words(memory: &Memory) -> Value {
    crate::unported!("bot.memory.tried:last_answer_words")
}

/// Python: bot/memory/tried.py:526 `asks_for_a_fix(memory)` (never called on the trace corpus)
pub fn asks_for_a_fix(memory: &Memory) -> bool {
    crate::unported!("bot.memory.tried:asks_for_a_fix")
}

/// Python: bot/memory/tried.py:580 `buying(memory)`
pub fn buying(memory: &Memory) -> bool {
    crate::unported!("bot.memory.tried:buying")
}

/// Python: bot/memory/tried.py:588 `person_said(memory)` (never called on the trace corpus)
pub fn person_said(memory: &Memory) -> String {
    crate::unported!("bot.memory.tried:person_said")
}

/// Python: bot/memory/tried.py:598 `_the_lines(memory)` (never called on the trace corpus)
pub fn _the_lines(memory: &Memory) -> Value {
    crate::unported!("bot.memory.tried:_the_lines")
}

/// Python: bot/memory/tried.py:618 `_names(said, blob, entry_words)` (never called on the trace corpus)
pub fn _names(said: &str, blob: &Value, entry_words: &[String]) -> bool {
    crate::unported!("bot.memory.tried:_names")
}

/// Python: bot/memory/tried.py:647 `names_a_thing(memory)` (never called on the trace corpus)
pub fn names_a_thing(memory: &Memory) -> bool {
    crate::unported!("bot.memory.tried:names_a_thing")
}

/// Python: bot/memory/tried.py:667 `has_evidence(text, memory)` (never called on the trace corpus)
pub fn has_evidence(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.tried:has_evidence")
}

/// Python: bot/memory/tried.py:687 `asked_what_its_doing(memory)`
pub fn asked_what_its_doing(memory: &Memory) -> bool {
    crate::unported!("bot.memory.tried:asked_what_its_doing")
}

/// Python: bot/memory/tried.py:696 `already_named(trade, memory)` (never called on the trace corpus)
pub fn already_named(trade: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.memory.tried:already_named")
}

/// Python: bot/memory/tried.py:714 `_the_answered_line(memory)` (never called on the trace corpus)
pub fn _the_answered_line(memory: &Memory) -> String {
    crate::unported!("bot.memory.tried:_the_answered_line")
}

/// Python: bot/memory/tried.py:720 `trade_for(memory)` (never called on the trace corpus)
pub fn trade_for(memory: &Memory) -> Value {
    crate::unported!("bot.memory.tried:trade_for")
}

/// Python: bot/memory/tried.py:773 `acted_pool(text, memory)`
pub fn acted_pool(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.tried:acted_pool")
}

/// Python: bot/memory/tried.py:806 `_up_to_the_contrast(what)`
pub fn _up_to_the_contrast(what: &str) -> String {
    crate::unported!("bot.memory.tried:_up_to_the_contrast")
}

/// Python: bot/memory/tried.py:817 `_read_step_tried(text)`
pub fn _read_step_tried(text: &str) -> Value {
    crate::unported!("bot.memory.tried:_read_step_tried")
}

/// Python: bot/memory/tried.py:850 `the_step_tried(text)` (never called on the trace corpus)
pub fn the_step_tried(text: &str) -> Value {
    crate::unported!("bot.memory.tried:the_step_tried")
}

/// Python: bot/memory/tried.py:863 `_the_past_step(text, said)`
pub fn _the_past_step(text: &str, said: &str) {
    crate::unported!("bot.memory.tried:_the_past_step")
}

/// Python: bot/memory/tried.py:890 `_the_last_answer(memory)`
pub fn _the_last_answer(memory: &Memory) -> String {
    crate::unported!("bot.memory.tried:_the_last_answer")
}

/// Python: bot/memory/tried.py:901 `_stem(word)` (never called on the trace corpus)
pub fn _stem(word: &str) -> Value {
    crate::unported!("bot.memory.tried:_stem")
}

/// Python: bot/memory/tried.py:913 `_step_stems(text)` (never called on the trace corpus)
pub fn _step_stems(text: &str) -> Value {
    crate::unported!("bot.memory.tried:_step_stems")
}

/// Python: bot/memory/tried.py:922 `_names_it(part, stems)` (never called on the trace corpus)
pub fn _names_it(part: &str, stems: &Value) -> bool {
    crate::unported!("bot.memory.tried:_names_it")
}

/// Python: bot/memory/tried.py:947 `_the_move_after(part, stems)` (never called on the trace corpus)
pub fn _the_move_after(part: &str, stems: &Value) -> Value {
    crate::unported!("bot.memory.tried:_the_move_after")
}

/// Python: bot/memory/tried.py:952 `_the_move_pieces(part, stems)` (never called on the trace corpus)
pub fn _the_move_pieces(part: &str, stems: &Value) -> Value {
    crate::unported!("bot.memory.tried:_the_move_pieces")
}

/// Python: bot/memory/tried.py:987 `_clauses(text)` (never called on the trace corpus)
pub fn _clauses(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.tried:_clauses")
}

/// Python: bot/memory/tried.py:1003 `_a_step(piece)` (never called on the trace corpus)
pub fn _a_step(piece: &str) -> Value {
    crate::unported!("bot.memory.tried:_a_step")
}

/// Python: bot/memory/tried.py:1033 `_opens_a_step(part)` (never called on the trace corpus)
pub fn _opens_a_step(part: &str) -> bool {
    crate::unported!("bot.memory.tried:_opens_a_step")
}

/// Python: bot/memory/tried.py:1052 `_the_step_after(thing, memory)`
pub fn _the_step_after(thing: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.tried:_the_step_after")
}

/// Python: bot/memory/tried.py:1068 `_the_steps_of(said)` (never called on the trace corpus)
pub fn _the_steps_of(said: &str) -> Value {
    crate::unported!("bot.memory.tried:_the_steps_of")
}

/// Python: bot/memory/tried.py:1086 `_list_heads(said)` (never called on the trace corpus)
pub fn _list_heads(said: &str) -> Value {
    crate::unported!("bot.memory.tried:_list_heads")
}

/// Python: bot/memory/tried.py:1115 `_object_stems(text)` (never called on the trace corpus)
pub fn _object_stems(text: &str) -> Value {
    crate::unported!("bot.memory.tried:_object_stems")
}

/// Python: bot/memory/tried.py:1129 `in_the_answer(thing, said)` (never called on the trace corpus)
pub fn in_the_answer(thing: &str, said: &str) -> bool {
    crate::unported!("bot.memory.tried:in_the_answer")
}

/// Python: bot/memory/tried.py:1155 `_flat(text)` (never called on the trace corpus)
pub fn _flat(text: &str) -> String {
    crate::unported!("bot.memory.tried:_flat")
}

/// Python: bot/memory/tried.py:1159 `_the_first_step_left(said, seen, after='')` (never called on the trace corpus)
pub fn _the_first_step_left(said: &str, seen: &Value, after: &str) -> Value {
    crate::unported!("bot.memory.tried:_the_first_step_left")
}

/// Python: bot/memory/tried.py:1218 `_with_its_tail(part, parts, i)` (never called on the trace corpus)
pub fn _with_its_tail(part: &str, parts: &[String], i: &Value) -> Value {
    crate::unported!("bot.memory.tried:_with_its_tail")
}

/// Python: bot/memory/tried.py:1242 `_the_advice_after(parts)` (never called on the trace corpus)
pub fn _the_advice_after(parts: &[String]) -> Value {
    crate::unported!("bot.memory.tried:_the_advice_after")
}

/// Python: bot/memory/tried.py:1264 `_walk_answer(said, thing, wrap=True)` (never called on the trace corpus)
pub fn _walk_answer(said: &str, thing: &str, wrap: &Value) -> Value {
    crate::unported!("bot.memory.tried:_walk_answer")
}

/// Python: bot/memory/tried.py:1368 `reads_as_a_step_done(text, memory)`
pub fn reads_as_a_step_done(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.tried:reads_as_a_step_done")
}

/// Python: bot/memory/tried.py:1410 `_as_a_lead(lead, thing, step)` (never called on the trace corpus)
pub fn _as_a_lead(lead: &str, thing: &str, step: &Value) -> String {
    crate::unported!("bot.memory.tried:_as_a_lead")
}

/// Python: bot/memory/tried.py:1432 `next_step_pool(text, memory)`
pub fn next_step_pool(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.tried:next_step_pool")
}

/// Python: bot/memory/tried.py:1507 `_asks_for_the_next_part(text)`
pub fn _asks_for_the_next_part(text: &str) -> bool {
    crate::unported!("bot.memory.tried:_asks_for_the_next_part")
}

/// Python: bot/memory/tried.py:1519 `next_step_ask_pool(text, memory)`
pub fn next_step_ask_pool(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.tried:next_step_ask_pool")
}

/// Python: bot/memory/tried.py:1557 `_somebody_is_the_subject(memory)` (never called on the trace corpus)
pub fn _somebody_is_the_subject(memory: &Memory) -> bool {
    crate::unported!("bot.memory.tried:_somebody_is_the_subject")
}

/// Python: bot/memory/tried.py:1584 `still_tail(text, memory)`
pub fn still_tail(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.tried:still_tail")
}

/// Python: bot/memory/tried.py:1617 `a_card_is_not_the_answer(text, memory)`
pub fn a_card_is_not_the_answer(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.tried:a_card_is_not_the_answer")
}

/// Python: bot/memory/tried.py:1639 `still_going_on(text, memory)`
pub fn still_going_on(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.tried:still_going_on")
}

/// Python: bot/memory/tried.py:1676 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.tried:reply_for")
}

/// Python: bot/memory/tried.py:1692 `pool_for(text, memory)`
pub fn pool_for(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.tried:pool_for")
}

/// Python: bot/memory/tried.py:1829 `_the_person_word(token)` (never called on the trace corpus)
pub fn _the_person_word(token: &Value) -> Value {
    crate::unported!("bot.memory.tried:_the_person_word")
}

/// Python: bot/memory/tried.py:1837 `_a_report_on_a_person(text)`
pub fn _a_report_on_a_person(text: &str) -> bool {
    crate::unported!("bot.memory.tried:_a_report_on_a_person")
}

/// Python: bot/memory/tried.py:1853 `_in_care(memory)` (never called on the trace corpus)
pub fn _in_care(memory: &Memory) -> Value {
    crate::unported!("bot.memory.tried:_in_care")
}

/// Python: bot/memory/tried.py:1905 `in_care_pool(text, memory)`
pub fn in_care_pool(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.tried:in_care_pool")
}

