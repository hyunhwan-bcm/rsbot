//! Port of `bot/memory/detail.py`.
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

/// Python: bot/memory/detail.py:236 `_a_verb(word, before='')`
pub fn _a_verb(word: &str, before: &str) -> bool {
    crate::unported!("bot.memory.detail:_a_verb")
}

/// Python: bot/memory/detail.py:249 `_flat(text)`
pub fn _flat(text: &str) -> String {
    crate::unported!("bot.memory.detail:_flat")
}

/// Python: bot/memory/detail.py:253 `_the_same_answer_as_the_one_it_narrows(step, said)` (never called on the trace corpus)
pub fn _the_same_answer_as_the_one_it_narrows(step: &Value, said: &str) -> bool {
    crate::unported!("bot.memory.detail:_the_same_answer_as_the_one_it_narrows")
}

/// Python: bot/memory/detail.py:263 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.detail:_words")
}

/// Python: bot/memory/detail.py:279 `the_detail(text)`
pub fn the_detail(text: &str) {
    crate::unported!("bot.memory.detail:the_detail")
}

/// Python: bot/memory/detail.py:413 `_a_figure(word)` (never called on the trace corpus)
pub fn _a_figure(word: &str) -> String {
    crate::unported!("bot.memory.detail:_a_figure")
}

/// Python: bot/memory/detail.py:418 `the_fact_the_answer_has(text, memory)`
pub fn the_fact_the_answer_has(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.detail:the_fact_the_answer_has")
}

/// Python: bot/memory/detail.py:464 `the_step_for(text, memory)`
pub fn the_step_for(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.detail:the_step_for")
}

/// Python: bot/memory/detail.py:635 `the_time_in_the_line(text)`
pub fn the_time_in_the_line(text: &str) {
    crate::unported!("bot.memory.detail:the_time_in_the_line")
}

/// Python: bot/memory/detail.py:679 `_a_time_step(said, detail)` (never called on the trace corpus)
pub fn _a_time_step(said: &str, detail: &Value) -> Value {
    crate::unported!("bot.memory.detail:_a_time_step")
}

/// Python: bot/memory/detail.py:700 `_the_answer_before(memory, text)` (never called on the trace corpus)
pub fn _the_answer_before(memory: &Memory, text: &str) -> Value {
    crate::unported!("bot.memory.detail:_the_answer_before")
}

/// Python: bot/memory/detail.py:723 `the_step_for_a_time(text, memory)`
pub fn the_step_for_a_time(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.detail:the_step_for_a_time")
}

/// Python: bot/memory/detail.py:771 `_a_count(detail)` (never called on the trace corpus)
pub fn _a_count(detail: &Value) -> bool {
    crate::unported!("bot.memory.detail:_a_count")
}

/// Python: bot/memory/detail.py:776 `_the_figure(said)` (never called on the trace corpus)
pub fn _the_figure(said: &str) -> Value {
    crate::unported!("bot.memory.detail:_the_figure")
}

/// Python: bot/memory/detail.py:809 `_the_carrying_words(text)`
pub fn _the_carrying_words(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.detail:_the_carrying_words")
}

/// Python: bot/memory/detail.py:824 `_the_clause_that_holds(words, said)` (never called on the trace corpus)
pub fn _the_clause_that_holds(words: &[String], said: &str) -> String {
    crate::unported!("bot.memory.detail:_the_clause_that_holds")
}

/// Python: bot/memory/detail.py:848 `the_part_of_the_answer(text, memory)`
pub fn the_part_of_the_answer(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.detail:the_part_of_the_answer")
}

/// Python: bot/memory/detail.py:884 `answers_the_line(text, memory)` (never called on the trace corpus)
pub fn answers_the_line(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.detail:answers_the_line")
}

/// Python: bot/memory/detail.py:889 `answer_for(text, memory)`
pub fn answer_for(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.detail:answer_for")
}

/// Python: bot/memory/detail.py:1015 `_the_answer_just_given(memory)` (never called on the trace corpus)
pub fn _the_answer_just_given(memory: &Memory) -> Value {
    crate::unported!("bot.memory.detail:_the_answer_just_given")
}

/// Python: bot/memory/detail.py:1029 `_bare_step(step)` (never called on the trace corpus)
pub fn _bare_step(step: &Value) -> Value {
    crate::unported!("bot.memory.detail:_bare_step")
}

/// Python: bot/memory/detail.py:1043 `the_plain_ask(text, memory)`
pub fn the_plain_ask(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.detail:the_plain_ask")
}

/// Python: bot/memory/detail.py:1084 `_clauses(move)` (never called on the trace corpus)
pub fn _clauses(r#move: &Value) -> Value {
    crate::unported!("bot.memory.detail:_clauses")
}

/// Python: bot/memory/detail.py:1101 `_first_clause(step)` (never called on the trace corpus)
pub fn _first_clause(step: &Value) -> String {
    crate::unported!("bot.memory.detail:_first_clause")
}

/// Python: bot/memory/detail.py:1124 `_one_of_a_list(clauses, groups, at)` (never called on the trace corpus)
pub fn _one_of_a_list(clauses: &Value, groups: &Value, at: i64) -> bool {
    crate::unported!("bot.memory.detail:_one_of_a_list")
}

/// Python: bot/memory/detail.py:1136 `_head_word(detail)` (never called on the trace corpus)
pub fn _head_word(detail: &Value) -> Value {
    crate::unported!("bot.memory.detail:_head_word")
}

/// Python: bot/memory/detail.py:1143 `_subject_words(memory)` (never called on the trace corpus)
pub fn _subject_words(memory: &Memory) -> Value {
    crate::unported!("bot.memory.detail:_subject_words")
}

/// Python: bot/memory/detail.py:1170 `_the_entry_reply(found)` (never called on the trace corpus)
pub fn _the_entry_reply(found: &Value) -> Value {
    crate::unported!("bot.memory.detail:_the_entry_reply")
}

/// Python: bot/memory/detail.py:1176 `_another_thing(detail, said)` (never called on the trace corpus)
pub fn _another_thing(detail: &Value, said: &str) -> bool {
    crate::unported!("bot.memory.detail:_another_thing")
}

/// Python: bot/memory/detail.py:1199 `_what_it_is_worth(detail, memory)` (never called on the trace corpus)
pub fn _what_it_is_worth(detail: &Value, memory: &Memory) -> Value {
    crate::unported!("bot.memory.detail:_what_it_is_worth")
}

/// Python: bot/memory/detail.py:1226 `_says_about(reply, head)` (never called on the trace corpus)
pub fn _says_about(reply: &str, head: &str) -> bool {
    crate::unported!("bot.memory.detail:_says_about")
}

/// Python: bot/memory/detail.py:1234 `_the_thing_itself(detail, memory)` (never called on the trace corpus)
pub fn _the_thing_itself(detail: &Value, memory: &Memory) -> Value {
    crate::unported!("bot.memory.detail:_the_thing_itself")
}

/// Python: bot/memory/detail.py:1275 `_asks_about_a_new_thing(text, detail, said)` (never called on the trace corpus)
pub fn _asks_about_a_new_thing(text: &str, detail: &Value, said: &str) -> bool {
    crate::unported!("bot.memory.detail:_asks_about_a_new_thing")
}

/// Python: bot/memory/detail.py:1302 `_step_about(said, detail)` (never called on the trace corpus)
pub fn _step_about(said: &str, detail: &Value) -> Option<String> {
    crate::unported!("bot.memory.detail:_step_about")
}

/// Python: bot/memory/detail.py:1356 `_a_branch(sentence)` (never called on the trace corpus)
pub fn _a_branch(sentence: &str) -> bool {
    crate::unported!("bot.memory.detail:_a_branch")
}

/// Python: bot/memory/detail.py:1367 `_without_branches(said)` (never called on the trace corpus)
pub fn _without_branches(said: &str) -> String {
    crate::unported!("bot.memory.detail:_without_branches")
}

/// Python: bot/memory/detail.py:1374 `_the_move_itself(move)` (never called on the trace corpus)
pub fn _the_move_itself(r#move: &Value) -> Value {
    crate::unported!("bot.memory.detail:_the_move_itself")
}

/// Python: bot/memory/detail.py:1425 `_the_heading(said)` (never called on the trace corpus)
pub fn _the_heading(said: &str) -> Value {
    crate::unported!("bot.memory.detail:_the_heading")
}

/// Python: bot/memory/detail.py:1440 `_is_the_label_itself(text, label)` (never called on the trace corpus)
pub fn _is_the_label_itself(text: &str, label: &str) -> bool {
    crate::unported!("bot.memory.detail:_is_the_label_itself")
}

/// Python: bot/memory/detail.py:1445 `_without_the_heading(said)` (never called on the trace corpus)
pub fn _without_the_heading(said: &str) -> String {
    crate::unported!("bot.memory.detail:_without_the_heading")
}

/// Python: bot/memory/detail.py:1460 `_first_instruction(said)` (never called on the trace corpus)
pub fn _first_instruction(said: &str) -> String {
    crate::unported!("bot.memory.detail:_first_instruction")
}

/// Python: bot/memory/detail.py:1482 `_opening_said(said)` (never called on the trace corpus)
pub fn _opening_said(said: &str) -> Value {
    crate::unported!("bot.memory.detail:_opening_said")
}

/// Python: bot/memory/detail.py:1492 `_is_a_span(detail)` (never called on the trace corpus)
pub fn _is_a_span(detail: &Value) -> bool {
    crate::unported!("bot.memory.detail:_is_a_span")
}

/// Python: bot/memory/detail.py:1498 `_a_span_step(said, detail)` (never called on the trace corpus)
pub fn _a_span_step(said: &str, detail: &Value) -> String {
    crate::unported!("bot.memory.detail:_a_span_step")
}

/// Python: bot/memory/detail.py:1518 `_opening_of(said)` (never called on the trace corpus)
pub fn _opening_of(said: &str) -> Value {
    crate::unported!("bot.memory.detail:_opening_of")
}

/// Python: bot/memory/detail.py:1539 `_names_nothing(detail)` (never called on the trace corpus)
pub fn _names_nothing(detail: &Value) -> bool {
    crate::unported!("bot.memory.detail:_names_nothing")
}

/// Python: bot/memory/detail.py:1555 `_says_where(detail)` (never called on the trace corpus)
pub fn _says_where(detail: &Value) -> bool {
    crate::unported!("bot.memory.detail:_says_where")
}

/// Python: bot/memory/detail.py:1568 `_a_step(said, questions_are_steps=False)` (never called on the trace corpus)
pub fn _a_step(said: &str, questions_are_steps: &Value) -> Value {
    crate::unported!("bot.memory.detail:_a_step")
}

/// Python: bot/memory/detail.py:1617 `_capital(said)`
pub fn _capital(said: &str) -> String {
    crate::unported!("bot.memory.detail:_capital")
}

/// Python: bot/memory/detail.py:1719 `stands_alone(sentence)`
pub fn stands_alone(sentence: &str) -> bool {
    crate::unported!("bot.memory.detail:stands_alone")
}

/// Python: bot/memory/detail.py:1725 `_report_stem(word)`
pub fn _report_stem(word: &str) -> String {
    crate::unported!("bot.memory.detail:_report_stem")
}

/// Python: bot/memory/detail.py:1736 `_report_content(text)`
pub fn _report_content(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.detail:_report_content")
}

/// Python: bot/memory/detail.py:1741 `a_report_about_the_thing(text, memory)`
pub fn a_report_about_the_thing(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.detail:a_report_about_the_thing")
}

