//! Port of `bot/compose/remark.py`.
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

/// Python: bot/compose/remark.py:411 `_a_good_night_out(clean)` (never called on the trace corpus)
pub fn _a_good_night_out(clean: &str) -> bool {
    crate::unported!("bot.compose.remark:_a_good_night_out")
}

/// Python: bot/compose/remark.py:427 `done_to_me(clean)`
pub fn done_to_me(clean: &str) -> bool {
    crate::unported!("bot.compose.remark:done_to_me")
}

/// Python: bot/compose/remark.py:544 `_a_thing_taken(clean)`
pub fn _a_thing_taken(clean: &str) -> bool {
    crate::unported!("bot.compose.remark:_a_thing_taken")
}

/// Python: bot/compose/remark.py:574 `_the_loss_was_asked(memory)`
pub fn _the_loss_was_asked(memory: &Memory) -> bool {
    crate::unported!("bot.compose.remark:_the_loss_was_asked")
}

/// Python: bot/compose/remark.py:581 `_the_answers_take_it(clean)` (never called on the trace corpus)
pub fn _the_answers_take_it(clean: &str) -> Option<bool> {
    crate::unported!("bot.compose.remark:_the_answers_take_it")
}

/// Python: bot/compose/remark.py:672 `_missed_a_person(clean)`
pub fn _missed_a_person(clean: &str) -> bool {
    crate::unported!("bot.compose.remark:_missed_a_person")
}

/// Python: bot/compose/remark.py:680 `reads_as_report(clean, words)`
pub fn reads_as_report(clean: &str, words: &[String]) {
    crate::unported!("bot.compose.remark:reads_as_report")
}

/// Python: bot/compose/remark.py:712 `_still_to_come(clean)` (never called on the trace corpus)
pub fn _still_to_come(clean: &str) -> bool {
    crate::unported!("bot.compose.remark:_still_to_come")
}

/// Python: bot/compose/remark.py:733 `_started_badly(clean)` (never called on the trace corpus)
pub fn _started_badly(clean: &str) -> bool {
    crate::unported!("bot.compose.remark:_started_badly")
}

/// Python: bot/compose/remark.py:745 `_reads_as_a_break_in(clean, words)`
pub fn _reads_as_a_break_in(clean: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.remark:_reads_as_a_break_in")
}

/// Python: bot/compose/remark.py:765 `break_in_reply_for(clean, words, memory)`
pub fn break_in_reply_for(clean: &str, words: &[String], memory: &Memory) {
    crate::unported!("bot.compose.remark:break_in_reply_for")
}

/// Python: bot/compose/remark.py:779 `report_reply_for(clean, words, memory)`
pub fn report_reply_for(clean: &str, words: &[String], memory: &Memory) {
    crate::unported!("bot.compose.remark:report_reply_for")
}

/// Python: bot/compose/remark.py:942 `reads_as_regret(clean, words=())`
pub fn reads_as_regret(clean: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.remark:reads_as_regret")
}

/// Python: bot/compose/remark.py:976 `regret_reply_for(clean, words, memory, repaired='')`
pub fn regret_reply_for(clean: &str, words: &[String], memory: &Memory, repaired: &str) {
    crate::unported!("bot.compose.remark:regret_reply_for")
}

/// Python: bot/compose/remark.py:1194 `_a_taste_they_refuse(clean, memory)`
pub fn _a_taste_they_refuse(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.remark:_a_taste_they_refuse")
}

/// Python: bot/compose/remark.py:1251 `person_pronouns(person_word)` (never called on the trace corpus)
pub fn person_pronouns(person_word: &str) -> Value {
    crate::unported!("bot.compose.remark:person_pronouns")
}

/// Python: bot/compose/remark.py:1261 `fill_person(line, person_word)` (never called on the trace corpus)
pub fn fill_person(line: &str, person_word: &str) -> Value {
    crate::unported!("bot.compose.remark:fill_person")
}

/// Python: bot/compose/remark.py:1337 `their_mood_going_on_reply_for(clean, memory)`
pub fn their_mood_going_on_reply_for(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.remark:their_mood_going_on_reply_for")
}

/// Python: bot/compose/remark.py:1359 `_pronoun_person(person)` (never called on the trace corpus)
pub fn _pronoun_person(person: &Value) -> Value {
    crate::unported!("bot.compose.remark:_pronoun_person")
}

/// Python: bot/compose/remark.py:1369 `their_mood_with_you_reply_for(clean, words, memory)`
pub fn their_mood_with_you_reply_for(clean: &str, words: &[String], memory: &mut Memory) {
    crate::unported!("bot.compose.remark:their_mood_with_you_reply_for")
}

/// Python: bot/compose/remark.py:1499 `name_a_place(said)` (never called on the trace corpus)
pub fn name_a_place(said: &str) -> String {
    crate::unported!("bot.compose.remark:name_a_place")
}

/// Python: bot/compose/remark.py:1508 `fill_age(line, n, months=False)` (never called on the trace corpus)
pub fn fill_age(line: &str, n: i64, months: &Value) -> String {
    crate::unported!("bot.compose.remark:fill_age")
}

/// Python: bot/compose/remark.py:1515 `_age_pool(n, months=False)` (never called on the trace corpus)
pub fn _age_pool(n: i64, months: &Value) -> Value {
    crate::unported!("bot.compose.remark:_age_pool")
}

/// Python: bot/compose/remark.py:1528 `_the_job_in(phrase)` (never called on the trace corpus)
pub fn _the_job_in(phrase: &str) -> Option<String> {
    crate::unported!("bot.compose.remark:_the_job_in")
}

/// Python: bot/compose/remark.py:1543 `_pick_filled(lines, memory, clean, fill)` (never called on the trace corpus)
pub fn _pick_filled(lines: &[String], memory: &Memory, clean: &str, fill: &Value) -> Value {
    crate::unported!("bot.compose.remark:_pick_filled")
}

/// Python: bot/compose/remark.py:1590 `_the_people_in(memory, name)` (never called on the trace corpus)
pub fn _the_people_in(memory: &Memory, name: &str) -> Value {
    crate::unported!("bot.compose.remark:_the_people_in")
}

/// Python: bot/compose/remark.py:1604 `_the_age_i_had(name, memory)` (never called on the trace corpus)
pub fn _the_age_i_had(name: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.remark:_the_age_i_had")
}

/// Python: bot/compose/remark.py:1619 `_an_age_put_right(clean, memory)`
pub fn _an_age_put_right(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.remark:_an_age_put_right")
}

/// Python: bot/compose/remark.py:1643 `their_people_reply_for(clean, words, memory)`
pub fn their_people_reply_for(clean: &str, words: &[String], memory: &Memory) {
    crate::unported!("bot.compose.remark:their_people_reply_for")
}

/// Python: bot/compose/remark.py:1919 `_the_thing_they_have(clean)`
pub fn _the_thing_they_have(clean: &str) {
    crate::unported!("bot.compose.remark:_the_thing_they_have")
}

/// Python: bot/compose/remark.py:1950 `_the_class_word(thing)` (never called on the trace corpus)
pub fn _the_class_word(thing: &str) -> Value {
    crate::unported!("bot.compose.remark:_the_class_word")
}

/// Python: bot/compose/remark.py:1966 `_article(thing)` (never called on the trace corpus)
pub fn _article(thing: &str) -> String {
    crate::unported!("bot.compose.remark:_article")
}

/// Python: bot/compose/remark.py:1976 `_the_fact_about(thing, kind)` (never called on the trace corpus)
pub fn _the_fact_about(thing: &str, kind: &str) -> Value {
    crate::unported!("bot.compose.remark:_the_fact_about")
}

/// Python: bot/compose/remark.py:1996 `_capped(line)` (never called on the trace corpus)
pub fn _capped(line: &str) -> Value {
    crate::unported!("bot.compose.remark:_capped")
}

/// Python: bot/compose/remark.py:2000 `a_thing_of_theirs_reply_for(clean, words, memory)`
pub fn a_thing_of_theirs_reply_for(clean: &str, words: &[String], memory: &Memory) {
    crate::unported!("bot.compose.remark:a_thing_of_theirs_reply_for")
}

/// Python: bot/compose/remark.py:2085 `the_two_of_them(clean, memory=None)`
pub fn the_two_of_them(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.remark:the_two_of_them")
}

/// Python: bot/compose/remark.py:2114 `_has_phrase(clean, phrases)`
pub fn _has_phrase(clean: &str, phrases: &[String]) -> bool {
    crate::unported!("bot.compose.remark:_has_phrase")
}

/// Python: bot/compose/remark.py:2118 `_planned_word(word)`
pub fn _planned_word(word: &str) -> bool {
    crate::unported!("bot.compose.remark:_planned_word")
}

/// Python: bot/compose/remark.py:2144 `_going_to_plan(clean)`
pub fn _going_to_plan(clean: &str) -> bool {
    crate::unported!("bot.compose.remark:_going_to_plan")
}

/// Python: bot/compose/remark.py:2166 `reads_as_plan(clean)`
pub fn reads_as_plan(clean: &str) -> bool {
    crate::unported!("bot.compose.remark:reads_as_plan")
}

/// Python: bot/compose/remark.py:2176 `_asked_last(memory)`
pub fn _asked_last(memory: &Memory) -> bool {
    crate::unported!("bot.compose.remark:_asked_last")
}

/// Python: bot/compose/remark.py:2183 `a_want_said_back(said_back, clean)`
pub fn a_want_said_back(said_back: Option<&str>, clean: &str) -> bool {
    crate::unported!("bot.compose.remark:a_want_said_back")
}

/// Python: bot/compose/remark.py:2199 `dependant_want_reply_for(clean, words, memory)`
pub fn dependant_want_reply_for(clean: &str, words: &[String], memory: &Memory) {
    crate::unported!("bot.compose.remark:dependant_want_reply_for")
}

/// Python: bot/compose/remark.py:2229 `reply_for(clean, words, memory, topics=())`
pub fn reply_for(clean: &str, words: &[String], memory: &Memory, topics: &[String]) -> Option<String> {
    crate::unported!("bot.compose.remark:reply_for")
}

/// Python: bot/compose/remark.py:2334 `reads_as_arranged(clean)`
pub fn reads_as_arranged(clean: &str) -> bool {
    crate::unported!("bot.compose.remark:reads_as_arranged")
}

/// Python: bot/compose/remark.py:2352 `_says_when_it_is(clean)` (never called on the trace corpus)
pub fn _says_when_it_is(clean: &str) -> bool {
    crate::unported!("bot.compose.remark:_says_when_it_is")
}

/// Python: bot/compose/remark.py:2364 `arranged_reply_for(clean, words, memory)`
pub fn arranged_reply_for(clean: &str, words: &[String], memory: &Memory) {
    crate::unported!("bot.compose.remark:arranged_reply_for")
}

/// Python: bot/compose/remark.py:2448 `_named(template, thing)` (never called on the trace corpus)
pub fn _named(template: &Value, thing: &str) -> String {
    crate::unported!("bot.compose.remark:_named")
}

/// Python: bot/compose/remark.py:2454 `interest_reply_for(interest, memory)`
pub fn interest_reply_for(interest: &Value, memory: &Memory) {
    crate::unported!("bot.compose.remark:interest_reply_for")
}

