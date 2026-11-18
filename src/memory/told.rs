//! Port of `bot/memory/told.py`.
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

/// Python: bot/memory/told.py:171 `_the_shape(verb, pred)` (never called on the trace corpus)
pub fn _the_shape(verb: &str, pred: &Value) -> Value {
    crate::unported!("bot.memory.told:_the_shape")
}

/// Python: bot/memory/told.py:189 `_the_verb(fact)` (never called on the trace corpus)
pub fn _the_verb(fact: &str) -> Value {
    crate::unported!("bot.memory.told:_the_verb")
}

/// Python: bot/memory/told.py:202 `_the_same_thing(fact, pred, claim)` (never called on the trace corpus)
pub fn _the_same_thing(fact: &str, pred: &Value, claim: &Value) -> bool {
    crate::unported!("bot.memory.told:_the_same_thing")
}

/// Python: bot/memory/told.py:361 `_tells_nothing(text)`
pub fn _tells_nothing(text: &str) -> bool {
    crate::unported!("bot.memory.told:_tells_nothing")
}

/// Python: bot/memory/told.py:374 `_handed_back(text, memory)` (never called on the trace corpus)
pub fn _handed_back(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.told:_handed_back")
}

/// Python: bot/memory/told.py:401 `is_a_boundary(text)`
pub fn is_a_boundary(text: &str) -> bool {
    crate::unported!("bot.memory.told:is_a_boundary")
}

/// Python: bot/memory/told.py:411 `at_a_boundary(text, memory=None)` (never called on the trace corpus)
pub fn at_a_boundary(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.told:at_a_boundary")
}

/// Python: bot/memory/told.py:418 `_a_boundary_in_a_claim(text)` (never called on the trace corpus)
pub fn _a_boundary_in_a_claim(text: &str) -> bool {
    crate::unported!("bot.memory.told:_a_boundary_in_a_claim")
}

/// Python: bot/memory/told.py:430 `_the_ask(text)`
pub fn _the_ask(text: &str) {
    crate::unported!("bot.memory.told:_the_ask")
}

/// Python: bot/memory/told.py:456 `ABOUT_THE_CHAT(text, memory=None)`
pub fn ABOUT_THE_CHAT(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.told:ABOUT_THE_CHAT")
}

/// Python: bot/memory/told.py:479 `answer(text, memory, facts=None, plain=None, claims=None)`
pub fn answer(text: &str, memory: &mut Memory, facts: &Value, plain: &Value, claims: &Value) -> String {
    crate::unported!("bot.memory.told:answer")
}

/// Python: bot/memory/told.py:522 `a_denial(said)` (never called on the trace corpus)
pub fn a_denial(said: &str) -> bool {
    crate::unported!("bot.memory.told:a_denial")
}

/// Python: bot/memory/told.py:539 `_lower_first(said)` (never called on the trace corpus)
pub fn _lower_first(said: &str) -> Value {
    crate::unported!("bot.memory.told:_lower_first")
}

/// Python: bot/memory/told.py:557 `_with_the_yes(said, text)`
pub fn _with_the_yes(said: &str, text: &str) -> String {
    crate::unported!("bot.memory.told:_with_the_yes")
}

/// Python: bot/memory/told.py:565 `_the_remembered(text)`
pub fn _the_remembered(text: &str) {
    crate::unported!("bot.memory.told:_the_remembered")
}

/// Python: bot/memory/told.py:577 `A_CALLBACK(text, memory=None)`
pub fn A_CALLBACK(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.told:A_CALLBACK")
}

/// Python: bot/memory/told.py:588 `a_callback_the_chat_has(text, memory)`
pub fn a_callback_the_chat_has(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.told:a_callback_the_chat_has")
}

/// Python: bot/memory/told.py:602 `_a_claim_about_the_chat(text)`
pub fn _a_claim_about_the_chat(text: &str) -> bool {
    crate::unported!("bot.memory.told:_a_claim_about_the_chat")
}

/// Python: bot/memory/told.py:608 `_an_earlier_line(memory, text)` (never called on the trace corpus)
pub fn _an_earlier_line(memory: &Memory, text: &str) -> bool {
    crate::unported!("bot.memory.told:_an_earlier_line")
}

/// Python: bot/memory/told.py:617 `callback(text, memory, facts=None, plain=None, claims=None)`
pub fn callback(text: &str, memory: &mut Memory, facts: &Value, plain: &Value, claims: &Value) {
    crate::unported!("bot.memory.told:callback")
}

/// Python: bot/memory/told.py:650 `_the_answer(text, memory, facts=None, plain=None, claims=None)`
pub fn _the_answer(text: &str, memory: &mut Memory, facts: &Value, plain: &Value, claims: &Value) {
    crate::unported!("bot.memory.told:_the_answer")
}

/// Python: bot/memory/told.py:756 `_the_property_of_the_thing(memory, what, facts)` (never called on the trace corpus)
pub fn _the_property_of_the_thing(memory: &Memory, what: &str, facts: &Value) -> Value {
    crate::unported!("bot.memory.told:_the_property_of_the_thing")
}

/// Python: bot/memory/told.py:770 `_a_clause_is_named(line)` (never called on the trace corpus)
pub fn _a_clause_is_named(line: &str) -> bool {
    crate::unported!("bot.memory.told:_a_clause_is_named")
}

/// Python: bot/memory/told.py:778 `_the_thing_the_chat_is_on(memory)` (never called on the trace corpus)
pub fn _the_thing_the_chat_is_on(memory: &Memory) -> String {
    crate::unported!("bot.memory.told:_the_thing_the_chat_is_on")
}

/// Python: bot/memory/told.py:788 `_no_note(thing, memory)` (never called on the trace corpus)
pub fn _no_note(thing: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.told:_no_note")
}

/// Python: bot/memory/told.py:824 `_the_thing_is_a_clause(thing)` (never called on the trace corpus)
pub fn _the_thing_is_a_clause(thing: &str) -> bool {
    crate::unported!("bot.memory.told:_the_thing_is_a_clause")
}

/// Python: bot/memory/told.py:847 `_quote(line, memory)` (never called on the trace corpus)
pub fn _quote(line: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.told:_quote")
}

/// Python: bot/memory/told.py:884 `_the_words_of_the_kind(what)` (never called on the trace corpus)
pub fn _the_words_of_the_kind(what: &str) -> Value {
    crate::unported!("bot.memory.told:_the_words_of_the_kind")
}

/// Python: bot/memory/told.py:914 `_yours(phrase)` (never called on the trace corpus)
pub fn _yours(phrase: &str) -> Value {
    crate::unported!("bot.memory.told:_yours")
}

/// Python: bot/memory/told.py:927 `_the_person_a_pronoun_points_at(memory, what)` (never called on the trace corpus)
pub fn _the_person_a_pronoun_points_at(memory: &mut Memory, what: &str) -> String {
    crate::unported!("bot.memory.told:_the_person_a_pronoun_points_at")
}

/// Python: bot/memory/told.py:954 `_words_of(said)` (never called on the trace corpus)
pub fn _words_of(said: &str) -> Value {
    crate::unported!("bot.memory.told:_words_of")
}

/// Python: bot/memory/told.py:960 `check(text, memory)`
pub fn check(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.told:check")
}

/// Python: bot/memory/told.py:1057 `_your_own_clause(said)` (never called on the trace corpus)
pub fn _your_own_clause(said: &str) -> Value {
    crate::unported!("bot.memory.told:_your_own_clause")
}

/// Python: bot/memory/told.py:1074 `_plain_contractions(said)` (never called on the trace corpus)
pub fn _plain_contractions(said: &str) -> Value {
    crate::unported!("bot.memory.told:_plain_contractions")
}

/// Python: bot/memory/told.py:1087 `_names_in_capitals(said)` (never called on the trace corpus)
pub fn _names_in_capitals(said: &str) -> Value {
    crate::unported!("bot.memory.told:_names_in_capitals")
}

/// Python: bot/memory/told.py:1096 `_claim_is_a_denial(claim)` (never called on the trace corpus)
pub fn _claim_is_a_denial(claim: &Value) -> bool {
    crate::unported!("bot.memory.told:_claim_is_a_denial")
}

/// Python: bot/memory/told.py:1100 `_their_lines(memory, who)` (never called on the trace corpus)
pub fn _their_lines(memory: &mut Memory, who: &str) -> Value {
    crate::unported!("bot.memory.told:_their_lines")
}

/// Python: bot/memory/told.py:1122 `_about_a_person(memory, what)` (never called on the trace corpus)
pub fn _about_a_person(memory: &mut Memory, what: &str) -> Value {
    crate::unported!("bot.memory.told:_about_a_person")
}

/// Python: bot/memory/told.py:1144 `_their_fact(who, entry)` (never called on the trace corpus)
pub fn _their_fact(who: &str, entry: &'static Entry) -> String {
    crate::unported!("bot.memory.told:_their_fact")
}

/// Python: bot/memory/told.py:1166 `_about_a_pet(memory, what, known, facts=None)` (never called on the trace corpus)
pub fn _about_a_pet(memory: &Memory, what: &str, known: &Value, facts: &Value) -> Value {
    crate::unported!("bot.memory.told:_about_a_pet")
}

/// Python: bot/memory/told.py:1209 `_the_pets(memory, pets, facts=None)` (never called on the trace corpus)
pub fn _the_pets(memory: &Memory, pets: &Value, facts: &Value) -> Value {
    crate::unported!("bot.memory.told:_the_pets")
}

/// Python: bot/memory/told.py:1221 `_the_pets_here(memory, pets, facts=None)` (never called on the trace corpus)
pub fn _the_pets_here(memory: &Memory, pets: &Value, facts: &Value) -> Value {
    crate::unported!("bot.memory.told:_the_pets_here")
}

/// Python: bot/memory/told.py:1246 `_a(word)` (never called on the trace corpus)
pub fn _a(word: &str) -> String {
    crate::unported!("bot.memory.told:_a")
}

/// Python: bot/memory/told.py:1250 `_many(pet)` (never called on the trace corpus)
pub fn _many(pet: &Value) -> String {
    crate::unported!("bot.memory.told:_many")
}

/// Python: bot/memory/told.py:1295 `_the_newest_stated_line(memory, skip='', a_question_ok=False)` (never called on the trace corpus)
pub fn _the_newest_stated_line(memory: &Memory, skip: &Value, a_question_ok: &Value) -> String {
    crate::unported!("bot.memory.told:_the_newest_stated_line")
}

/// Python: bot/memory/told.py:1316 `_a_claim_that_points(text, memory)`
pub fn _a_claim_that_points(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.told:_a_claim_that_points")
}

/// Python: bot/memory/told.py:1330 `_the_proof_they_asked_for(named, text, memory)` (never called on the trace corpus)
pub fn _the_proof_they_asked_for(named: &Value, text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.told:_the_proof_they_asked_for")
}

/// Python: bot/memory/told.py:1361 `_line_about(memory, what, whole_words=False, skip='')` (never called on the trace corpus)
pub fn _line_about(memory: &Memory, what: &str, whole_words: &[String], skip: &Value) -> Value {
    crate::unported!("bot.memory.told:_line_about")
}

/// Python: bot/memory/told.py:1410 `_a_trip_line(memory)` (never called on the trace corpus)
pub fn _a_trip_line(memory: &Memory) -> Value {
    crate::unported!("bot.memory.told:_a_trip_line")
}

/// Python: bot/memory/told.py:1426 `_flat(line)` (never called on the trace corpus)
pub fn _flat(line: &str) -> String {
    crate::unported!("bot.memory.told:_flat")
}

/// Python: bot/memory/told.py:1430 `_the_denial(who, verb, pred)` (never called on the trace corpus)
pub fn _the_denial(who: &str, verb: &str, pred: &Value) -> String {
    crate::unported!("bot.memory.told:_the_denial")
}

/// Python: bot/memory/told.py:1437 `_a_pronoun_for(who)` (never called on the trace corpus)
pub fn _a_pronoun_for(who: &str) -> Value {
    crate::unported!("bot.memory.told:_a_pronoun_for")
}

/// Python: bot/memory/told.py:1449 `_trim(line, limit=100)` (never called on the trace corpus)
pub fn _trim(line: &str, limit: i64) -> Value {
    crate::unported!("bot.memory.told:_trim")
}

/// Python: bot/memory/told.py:1468 `_stem(word)` (never called on the trace corpus)
pub fn _stem(word: &str) -> Value {
    crate::unported!("bot.memory.told:_stem")
}

/// Python: bot/memory/told.py:1485 `_kept_words(value, into=None)` (never called on the trace corpus)
pub fn _kept_words(value: &Value, into: &Value) -> Value {
    crate::unported!("bot.memory.told:_kept_words")
}

/// Python: bot/memory/told.py:1499 `_the_fact_that_names(memory, what, known, facts)` (never called on the trace corpus)
pub fn _the_fact_that_names(memory: &Memory, what: &str, known: &Value, facts: &Value) -> Value {
    crate::unported!("bot.memory.told:_the_fact_that_names")
}

/// Python: bot/memory/told.py:1521 `_the_kind_of(word)` (never called on the trace corpus)
pub fn _the_kind_of(word: &str) -> Value {
    crate::unported!("bot.memory.told:_the_kind_of")
}

/// Python: bot/memory/told.py:1531 `_names_only_a_kind(what)` (never called on the trace corpus)
pub fn _names_only_a_kind(what: &str) -> bool {
    crate::unported!("bot.memory.told:_names_only_a_kind")
}

/// Python: bot/memory/told.py:1537 `_the_fact_of_that_kind(memory, what, known, facts=None)` (never called on the trace corpus)
pub fn _the_fact_of_that_kind(memory: &Memory, what: &str, known: &Value, facts: &Value) -> Value {
    crate::unported!("bot.memory.told:_the_fact_of_that_kind")
}

