//! Port of `bot/memory/thread.py`.
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

/// Python: bot/memory/thread.py:106 `_asks_about_the_bot(reading)` (never called on the trace corpus)
pub fn _asks_about_the_bot(reading: &Understood) -> bool {
    crate::unported!("bot.memory.thread:_asks_about_the_bot")
}

/// Python: bot/memory/thread.py:120 `_asks_about_the_chat(text, memory, skip)` (never called on the trace corpus)
pub fn _asks_about_the_chat(text: &str, memory: &Memory, skip: &Value) -> bool {
    crate::unported!("bot.memory.thread:_asks_about_the_chat")
}

/// Python: bot/memory/thread.py:130 `_turns(memory)`
pub fn _turns(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.memory.thread:_turns")
}

/// Python: bot/memory/thread.py:135 `earlier_turn(memory)` (never called on the trace corpus)
pub fn earlier_turn(memory: &Memory) -> bool {
    crate::unported!("bot.memory.thread:earlier_turn")
}

/// Python: bot/memory/thread.py:156 `only_the_arrival(text)`
pub fn only_the_arrival(text: &str) -> bool {
    crate::unported!("bot.memory.thread:only_the_arrival")
}

/// Python: bot/memory/thread.py:170 `user_lines(memory)` (never called on the trace corpus)
pub fn user_lines(memory: &Memory) -> Value {
    crate::unported!("bot.memory.thread:user_lines")
}

/// Python: bot/memory/thread.py:194 `_is_a_receipt(reading)` (never called on the trace corpus)
pub fn _is_a_receipt(reading: &Understood) -> bool {
    crate::unported!("bot.memory.thread:_is_a_receipt")
}

/// Python: bot/memory/thread.py:202 `_is_thin(reading)` (never called on the trace corpus)
pub fn _is_thin(reading: &Understood) -> bool {
    crate::unported!("bot.memory.thread:_is_thin")
}

/// Python: bot/memory/thread.py:211 `signs_off(text)` (never called on the trace corpus)
pub fn signs_off(text: &str) -> bool {
    crate::unported!("bot.memory.thread:signs_off")
}

/// Python: bot/memory/thread.py:227 `carries_subject(text)`
pub fn carries_subject(text: &str) -> bool {
    crate::unported!("bot.memory.thread:carries_subject")
}

/// Python: bot/memory/thread.py:233 `thin(text)` (never called on the trace corpus)
pub fn thin(text: &str) -> bool {
    crate::unported!("bot.memory.thread:thin")
}

/// Python: bot/memory/thread.py:270 `points_back(text)`
pub fn points_back(text: &str) -> bool {
    crate::unported!("bot.memory.thread:points_back")
}

/// Python: bot/memory/thread.py:311 `points_at_a_word_alone(text)`
pub fn points_at_a_word_alone(text: &str) -> bool {
    crate::unported!("bot.memory.thread:points_at_a_word_alone")
}

/// Python: bot/memory/thread.py:353 `asks_what_i_said(text)`
pub fn asks_what_i_said(text: &str) -> bool {
    crate::unported!("bot.memory.thread:asks_what_i_said")
}

/// Python: bot/memory/thread.py:358 `nothing_said(memory)`
pub fn nothing_said(memory: &Memory) -> bool {
    crate::unported!("bot.memory.thread:nothing_said")
}

/// Python: bot/memory/thread.py:368 `_is_memory_line(reading)` (never called on the trace corpus)
pub fn _is_memory_line(reading: &Understood) -> bool {
    crate::unported!("bot.memory.thread:_is_memory_line")
}

/// Python: bot/memory/thread.py:376 `_named_entities(reading)` (never called on the trace corpus)
pub fn _named_entities(reading: &Understood) -> Value {
    crate::unported!("bot.memory.thread:_named_entities")
}

/// Python: bot/memory/thread.py:397 `_groups(entities, reading)` (never called on the trace corpus)
pub fn _groups(entities: &Value, reading: &Understood) -> Value {
    crate::unported!("bot.memory.thread:_groups")
}

/// Python: bot/memory/thread.py:422 `_is_the_verb(reading, word)` (never called on the trace corpus)
pub fn _is_the_verb(reading: &Understood, word: &str) -> bool {
    crate::unported!("bot.memory.thread:_is_the_verb")
}

/// Python: bot/memory/thread.py:433 `_verb_after_a_subject(reading, word)` (never called on the trace corpus)
pub fn _verb_after_a_subject(reading: &Understood, word: &str) -> bool {
    crate::unported!("bot.memory.thread:_verb_after_a_subject")
}

/// Python: bot/memory/thread.py:455 `_word_behind_prep(reading, word)` (never called on the trace corpus)
pub fn _word_behind_prep(reading: &Understood, word: &str) -> bool {
    crate::unported!("bot.memory.thread:_word_behind_prep")
}

/// Python: bot/memory/thread.py:469 `_behind_a_prep(reading, entity)` (never called on the trace corpus)
pub fn _behind_a_prep(reading: &Understood, entity: &Value) -> bool {
    crate::unported!("bot.memory.thread:_behind_a_prep")
}

/// Python: bot/memory/thread.py:481 `_the_phrase_around(reading, word)` (never called on the trace corpus)
pub fn _the_phrase_around(reading: &Understood, word: &str) -> Value {
    crate::unported!("bot.memory.thread:_the_phrase_around")
}

/// Python: bot/memory/thread.py:500 `_phrase_for(text, reading, named_only=False)` (never called on the trace corpus)
pub fn _phrase_for(text: &str, reading: &Understood, named_only: &Value) -> Value {
    crate::unported!("bot.memory.thread:_phrase_for")
}

/// Python: bot/memory/thread.py:560 `_place_case(name)` (never called on the trace corpus)
pub fn _place_case(name: &str) -> Value {
    crate::unported!("bot.memory.thread:_place_case")
}

/// Python: bot/memory/thread.py:570 `_same_spelling(text, phrase)` (never called on the trace corpus)
pub fn _same_spelling(text: &str, phrase: &str) -> Value {
    crate::unported!("bot.memory.thread:_same_spelling")
}

/// Python: bot/memory/thread.py:584 `_qualifier(reading, phrase, keywords)` (never called on the trace corpus)
pub fn _qualifier(reading: &Understood, phrase: &str, keywords: &[String]) -> Value {
    crate::unported!("bot.memory.thread:_qualifier")
}

/// Python: bot/memory/thread.py:603 `about(memory, skip=None)` (never called on the trace corpus)
pub fn about(memory: &Memory, skip: &Value) -> Value {
    crate::unported!("bot.memory.thread:about")
}

/// Python: bot/memory/thread.py:658 `_is_subject_phrase(phrase)`
pub fn _is_subject_phrase(phrase: &str) -> bool {
    crate::unported!("bot.memory.thread:_is_subject_phrase")
}

/// Python: bot/memory/thread.py:683 `_from_topics(memory)` (never called on the trace corpus)
pub fn _from_topics(memory: &Memory) -> Value {
    crate::unported!("bot.memory.thread:_from_topics")
}

/// Python: bot/memory/thread.py:710 `_is_wh(word)`
pub fn _is_wh(word: &str) -> bool {
    crate::unported!("bot.memory.thread:_is_wh")
}

/// Python: bot/memory/thread.py:723 `_with_wh(reading, words)`
pub fn _with_wh(reading: &Understood, words: &[String]) -> Vec<String> {
    crate::unported!("bot.memory.thread:_with_wh")
}

/// Python: bot/memory/thread.py:755 `_strip_lead(words)`
pub fn _strip_lead(words: &[String]) -> Vec<String> {
    crate::unported!("bot.memory.thread:_strip_lead")
}

/// Python: bot/memory/thread.py:796 `_with_you(words)` (never called on the trace corpus)
pub fn _with_you(words: &[String]) -> Value {
    crate::unported!("bot.memory.thread:_with_you")
}

/// Python: bot/memory/thread.py:812 `_first_person_aux(word)`
pub fn _first_person_aux(word: &str) -> String {
    crate::unported!("bot.memory.thread:_first_person_aux")
}

/// Python: bot/memory/thread.py:818 `_flip(words, addressee=False)`
pub fn _flip(words: &[String], addressee: bool) -> Vec<String> {
    crate::unported!("bot.memory.thread:_flip")
}

/// Python: bot/memory/thread.py:858 `your_words(text)`
pub fn your_words(text: &str) -> String {
    crate::unported!("bot.memory.thread:your_words")
}

/// Python: bot/memory/thread.py:870 `_expand(text)`
pub fn _expand(text: &str) -> String {
    crate::unported!("bot.memory.thread:_expand")
}

/// Python: bot/memory/thread.py:876 `_as_text(value)`
pub fn _as_text(value: &[String]) -> String {
    crate::unported!("bot.memory.thread:_as_text")
}

/// Python: bot/memory/thread.py:910 `_subjects_at(words, index, names=())`
pub fn _subjects_at(words: &[String], index: i64, names: &[String]) -> bool {
    crate::unported!("bot.memory.thread:_subjects_at")
}

/// Python: bot/memory/thread.py:918 `_np_end(words, start, most=3)`
pub fn _np_end(words: &[String], start: i64, most: i64) -> i64 {
    crate::unported!("bot.memory.thread:_np_end")
}

/// Python: bot/memory/thread.py:937 `_embed(words, names=())`
pub fn _embed(words: &[String], names: &[String]) -> Vec<String> {
    crate::unported!("bot.memory.thread:_embed")
}

/// Python: bot/memory/thread.py:985 `_keep_possessives(clause, clean)`
pub fn _keep_possessives(clause: &str, clean: &str) -> String {
    crate::unported!("bot.memory.thread:_keep_possessives")
}

/// Python: bot/memory/thread.py:1010 `_truncated(clause, clean)`
pub fn _truncated(clause: &str, clean: &str) -> bool {
    crate::unported!("bot.memory.thread:_truncated")
}

/// Python: bot/memory/thread.py:1026 `_numbers(text, clause)`
pub fn _numbers(text: &str, clause: &str) -> String {
    crate::unported!("bot.memory.thread:_numbers")
}

/// Python: bot/memory/thread.py:1036 `restate(text)`
pub fn restate(text: &str) -> String {
    crate::unported!("bot.memory.thread:restate")
}

/// Python: bot/memory/thread.py:1141 `_past(word)`
pub fn _past(word: &str) -> String {
    crate::unported!("bot.memory.thread:_past")
}

/// Python: bot/memory/thread.py:1155 `_agree(verb, aux)`
pub fn _agree(verb: &str, aux: &str) -> String {
    crate::unported!("bot.memory.thread:_agree")
}

/// Python: bot/memory/thread.py:1167 `_third(verb)`
pub fn _third(verb: &str) -> String {
    crate::unported!("bot.memory.thread:_third")
}

/// Python: bot/memory/thread.py:1189 `_conditional(clause)`
pub fn _conditional(clause: &str) -> String {
    crate::unported!("bot.memory.thread:_conditional")
}

/// Python: bot/memory/thread.py:1197 `_restore_case(text, clause)`
pub fn _restore_case(text: &str, clause: &str) -> String {
    crate::unported!("bot.memory.thread:_restore_case")
}

/// Python: bot/memory/thread.py:1209 `_capital(text)`
pub fn _capital(text: &str) -> String {
    crate::unported!("bot.memory.thread:_capital")
}

/// Python: bot/memory/thread.py:1231 `on_the_table(memory, limit=3)` (never called on the trace corpus)
pub fn on_the_table(memory: &Memory, limit: i64) -> Value {
    crate::unported!("bot.memory.thread:on_the_table")
}

/// Python: bot/memory/thread.py:1282 `_said_in_the_chat(phrase, turns)` (never called on the trace corpus)
pub fn _said_in_the_chat(phrase: &str, turns: &[Turn]) -> bool {
    crate::unported!("bot.memory.thread:_said_in_the_chat")
}

/// Python: bot/memory/thread.py:1328 `_said_as_a_state(head, text)` (never called on the trace corpus)
pub fn _said_as_a_state(head: &str, text: &str) -> Value {
    crate::unported!("bot.memory.thread:_said_as_a_state")
}

/// Python: bot/memory/thread.py:1349 `in_a_list(phrases)` (never called on the trace corpus)
pub fn in_a_list(phrases: &Value) -> Value {
    crate::unported!("bot.memory.thread:in_a_list")
}

/// Python: bot/memory/thread.py:1363 `topic_index(memory, skip=None)` (never called on the trace corpus)
pub fn topic_index(memory: &Memory, skip: &Value) -> Value {
    crate::unported!("bot.memory.thread:topic_index")
}

/// Python: bot/memory/thread.py:1388 `last_ask(memory, skip=None, not_before=None)` (never called on the trace corpus)
pub fn last_ask(memory: &Memory, skip: &Value, not_before: &Value) -> Value {
    crate::unported!("bot.memory.thread:last_ask")
}

/// Python: bot/memory/thread.py:1419 `_only_a_detail(text)` (never called on the trace corpus)
pub fn _only_a_detail(text: &str) -> bool {
    crate::unported!("bot.memory.thread:_only_a_detail")
}

/// Python: bot/memory/thread.py:1432 `last_said(memory, skip=None)` (never called on the trace corpus)
pub fn last_said(memory: &Memory, skip: &Value) -> Value {
    crate::unported!("bot.memory.thread:last_said")
}

/// Python: bot/memory/thread.py:1450 `last_said_past_closings(memory, skip=None)` (never called on the trace corpus)
pub fn last_said_past_closings(memory: &Memory, skip: &Value) -> Value {
    crate::unported!("bot.memory.thread:last_said_past_closings")
}

/// Python: bot/memory/thread.py:1474 `named(memory, limit=2)` (never called on the trace corpus)
pub fn named(memory: &Memory, limit: i64) -> Value {
    crate::unported!("bot.memory.thread:named")
}

/// Python: bot/memory/thread.py:1533 `checking_the_verdict(text, memory)`
pub fn checking_the_verdict(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.thread:checking_the_verdict")
}

/// Python: bot/memory/thread.py:1600 `checking_a_fact(text, memory)`
pub fn checking_a_fact(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.thread:checking_a_fact")
}

/// Python: bot/memory/thread.py:1633 `_fact_from(answer)` (never called on the trace corpus)
pub fn _fact_from(answer: &str) -> bool {
    crate::unported!("bot.memory.thread:_fact_from")
}

/// Python: bot/memory/thread.py:1639 `_a_frame_not_said(frames, memory)` (never called on the trace corpus)
pub fn _a_frame_not_said(frames: &Value, memory: &Memory) -> Value {
    crate::unported!("bot.memory.thread:_a_frame_not_said")
}

/// Python: bot/memory/thread.py:1656 `_next_frame(frames, used, said)` (never called on the trace corpus)
pub fn _next_frame(frames: &Value, used: &Value, said: &str) -> Value {
    crate::unported!("bot.memory.thread:_next_frame")
}

/// Python: bot/memory/thread.py:1665 `_first_clause(said, limit=13)` (never called on the trace corpus)
pub fn _first_clause(said: &str, limit: i64) -> Value {
    crate::unported!("bot.memory.thread:_first_clause")
}

/// Python: bot/memory/thread.py:1773 `_a_done_word(word)` (never called on the trace corpus)
pub fn _a_done_word(word: &str) -> bool {
    crate::unported!("bot.memory.thread:_a_done_word")
}

/// Python: bot/memory/thread.py:1808 `_the_verdict_they_gave(line)` (never called on the trace corpus)
pub fn _the_verdict_they_gave(line: &str) -> String {
    crate::unported!("bot.memory.thread:_the_verdict_they_gave")
}

/// Python: bot/memory/thread.py:1821 `_the_thing_they_had(line)` (never called on the trace corpus)
pub fn _the_thing_they_had(line: &str) -> Value {
    crate::unported!("bot.memory.thread:_the_thing_they_had")
}

/// Python: bot/memory/thread.py:1867 `went(text, memory)`
pub fn went(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.thread:went")
}

/// Python: bot/memory/thread.py:1912 `_asked_already(reply)` (never called on the trace corpus)
pub fn _asked_already(reply: &str) -> bool {
    crate::unported!("bot.memory.thread:_asked_already")
}

