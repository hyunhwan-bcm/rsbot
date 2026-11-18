//! Port of `bot/memory/relations.py`.
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

/// Python: bot/memory/relations.py:40 `_folded(text, memory=None)`
pub fn _folded(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.relations:_folded")
}

/// Python: bot/memory/relations.py:63 `_person_from_pronoun(text, memory, named_here=())`
pub fn _person_from_pronoun(text: &str, memory: &mut Memory, named_here: &[String]) -> String {
    crate::unported!("bot.memory.relations:_person_from_pronoun")
}

/// Python: bot/memory/relations.py:98 `_gender_of(who)` (never called on the trace corpus)
pub fn _gender_of(who: &str) -> Value {
    crate::unported!("bot.memory.relations:_gender_of")
}

/// Python: bot/memory/relations.py:120 `_pronoun_fits(who, fits)`
pub fn _pronoun_fits(who: &str, fits: &[String]) -> bool {
    crate::unported!("bot.memory.relations:_pronoun_fits")
}

/// Python: bot/memory/relations.py:136 `_newest_first(people, memory)`
pub fn _newest_first(people: &[String], memory: &Memory) -> Vec<String> {
    crate::unported!("bot.memory.relations:_newest_first")
}

/// Python: bot/memory/relations.py:165 `_named_person(low, memory=None)`
pub fn _named_person(low: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.memory.relations:_named_person")
}

/// Python: bot/memory/relations.py:204 `_a_person_in_the_store(memory, who)`
pub fn _a_person_in_the_store(memory: &mut Memory, who: &str) -> bool {
    crate::unported!("bot.memory.relations:_a_person_in_the_store")
}

/// Python: bot/memory/relations.py:239 `_continues_an_about_ask(memory, low)`
pub fn _continues_an_about_ask(memory: &Memory, low: &str) -> bool {
    crate::unported!("bot.memory.relations:_continues_an_about_ask")
}

/// Python: bot/memory/relations.py:284 `_a_word_the_store_knows(memory, word)` (never called on the trace corpus)
pub fn _a_word_the_store_knows(memory: &mut Memory, word: &str) -> bool {
    crate::unported!("bot.memory.relations:_a_word_the_store_knows")
}

/// Python: bot/memory/relations.py:298 `_age_is_about_a_person(low, memory)` (never called on the trace corpus)
pub fn _age_is_about_a_person(low: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.relations:_age_is_about_a_person")
}

/// Python: bot/memory/relations.py:398 `_tidy_name(value)` (never called on the trace corpus)
pub fn _tidy_name(value: &Value) -> Value {
    crate::unported!("bot.memory.relations:_tidy_name")
}

/// Python: bot/memory/relations.py:404 `_same_as_last(reply, memory)`
pub fn _same_as_last(reply: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.relations:_same_as_last")
}

/// Python: bot/memory/relations.py:414 `_animal_family(animal)`
pub fn _animal_family(animal: &str) -> String {
    crate::unported!("bot.memory.relations:_animal_family")
}

/// Python: bot/memory/relations.py:423 `_a(word)` (never called on the trace corpus)
pub fn _a(word: &str) -> String {
    crate::unported!("bot.memory.relations:_a")
}

/// Python: bot/memory/relations.py:498 `_their_taste(memory, low)`
pub fn _their_taste(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.relations:_their_taste")
}

/// Python: bot/memory/relations.py:571 `_same_thing(one, other)` (never called on the trace corpus)
pub fn _same_thing(one: &str, other: &str) -> bool {
    crate::unported!("bot.memory.relations:_same_thing")
}

/// Python: bot/memory/relations.py:586 `_plural_person(who)` (never called on the trace corpus)
pub fn _plural_person(who: &str) -> bool {
    crate::unported!("bot.memory.relations:_plural_person")
}

/// Python: bot/memory/relations.py:591 `_they_like_sentence(who, things, how, pronoun='')` (never called on the trace corpus)
pub fn _they_like_sentence(who: &str, things: &Value, how: &str, pronoun: &Value) -> String {
    crate::unported!("bot.memory.relations:_they_like_sentence")
}

/// Python: bot/memory/relations.py:621 `_their_job(memory, low)`
pub fn _their_job(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.relations:_their_job")
}

/// Python: bot/memory/relations.py:665 `_kin_forms(who)` (never called on the trace corpus)
pub fn _kin_forms(who: &str) -> Value {
    crate::unported!("bot.memory.relations:_kin_forms")
}

/// Python: bot/memory/relations.py:680 `_a_kept_person(memory, who)` (never called on the trace corpus)
pub fn _a_kept_person(memory: &mut Memory, who: &str) -> Value {
    crate::unported!("bot.memory.relations:_a_kept_person")
}

/// Python: bot/memory/relations.py:689 `_same_name_however_they_wrote_it(entry, line)` (never called on the trace corpus)
pub fn _same_name_however_they_wrote_it(entry: &'static Entry, line: &str) -> bool {
    crate::unported!("bot.memory.relations:_same_name_however_they_wrote_it")
}

/// Python: bot/memory/relations.py:701 `_same_name(known, line)` (never called on the trace corpus)
pub fn _same_name(known: &Value, line: &str) -> bool {
    crate::unported!("bot.memory.relations:_same_name")
}

/// Python: bot/memory/relations.py:721 `_more_than_one(who)` (never called on the trace corpus)
pub fn _more_than_one(who: &str) -> bool {
    crate::unported!("bot.memory.relations:_more_than_one")
}

/// Python: bot/memory/relations.py:730 `_canon_relation(word)` (never called on the trace corpus)
pub fn _canon_relation(word: &str) -> Value {
    crate::unported!("bot.memory.relations:_canon_relation")
}

/// Python: bot/memory/relations.py:741 `_relations_of(who)` (never called on the trace corpus)
pub fn _relations_of(who: &str) -> Value {
    crate::unported!("bot.memory.relations:_relations_of")
}

/// Python: bot/memory/relations.py:757 `_in_the_group(who, group)` (never called on the trace corpus)
pub fn _in_the_group(who: &str, group: &Value) -> bool {
    crate::unported!("bot.memory.relations:_in_the_group")
}

/// Python: bot/memory/relations.py:762 `_a_list_of_names(names)` (never called on the trace corpus)
pub fn _a_list_of_names(names: &[String]) -> String {
    crate::unported!("bot.memory.relations:_a_list_of_names")
}

/// Python: bot/memory/relations.py:770 `_mentioned_the_group(memory, group, word)` (never called on the trace corpus)
pub fn _mentioned_the_group(memory: &mut Memory, group: &Value, word: &str) -> bool {
    crate::unported!("bot.memory.relations:_mentioned_the_group")
}

/// Python: bot/memory/relations.py:795 `_whether_they_have(memory, low)`
pub fn _whether_they_have(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.relations:_whether_they_have")
}

/// Python: bot/memory/relations.py:893 `_a_list_of_words(words)` (never called on the trace corpus)
pub fn _a_list_of_words(words: &[String]) -> String {
    crate::unported!("bot.memory.relations:_a_list_of_words")
}

/// Python: bot/memory/relations.py:903 `_more_than_one_word(word)` (never called on the trace corpus)
pub fn _more_than_one_word(word: &str) -> bool {
    crate::unported!("bot.memory.relations:_more_than_one_word")
}

/// Python: bot/memory/relations.py:909 `_with_article(word)` (never called on the trace corpus)
pub fn _with_article(word: &str) -> String {
    crate::unported!("bot.memory.relations:_with_article")
}

/// Python: bot/memory/relations.py:915 `_a_or_any(words)` (never called on the trace corpus)
pub fn _a_or_any(words: &[String]) -> String {
    crate::unported!("bot.memory.relations:_a_or_any")
}

/// Python: bot/memory/relations.py:925 `_a_plural_list(words)` (never called on the trace corpus)
pub fn _a_plural_list(words: &[String]) -> Value {
    crate::unported!("bot.memory.relations:_a_plural_list")
}

/// Python: bot/memory/relations.py:944 `_the_group_names_typed(word, memory)` (never called on the trace corpus)
pub fn _the_group_names_typed(word: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.relations:_the_group_names_typed")
}

/// Python: bot/memory/relations.py:981 `_the_names_a_pronoun_gave(line)` (never called on the trace corpus)
pub fn _the_names_a_pronoun_gave(line: &str) -> Value {
    crate::unported!("bot.memory.relations:_the_names_a_pronoun_gave")
}

/// Python: bot/memory/relations.py:989 `_the_names_from(said)` (never called on the trace corpus)
pub fn _the_names_from(said: &str) -> Value {
    crate::unported!("bot.memory.relations:_the_names_from")
}

/// Python: bot/memory/relations.py:1027 `_the_several_named_before(memory)` (never called on the trace corpus)
pub fn _the_several_named_before(memory: &Memory) -> Value {
    crate::unported!("bot.memory.relations:_the_several_named_before")
}

/// Python: bot/memory/relations.py:1061 `_the_names_tail(line)` (never called on the trace corpus)
pub fn _the_names_tail(line: &str) -> Value {
    crate::unported!("bot.memory.relations:_the_names_tail")
}

/// Python: bot/memory/relations.py:1077 `_the_names_of_them_all(memory)` (never called on the trace corpus)
pub fn _the_names_of_them_all(memory: &Memory) -> Value {
    crate::unported!("bot.memory.relations:_the_names_of_them_all")
}

/// Python: bot/memory/relations.py:1098 `_the_names_they_typed(word, memory)` (never called on the trace corpus)
pub fn _the_names_they_typed(word: &str, memory: &Memory) -> Option<String> {
    crate::unported!("bot.memory.relations:_the_names_they_typed")
}

/// Python: bot/memory/relations.py:1107 `_the_names_of_the_group(memory, low)`
pub fn _the_names_of_the_group(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.relations:_the_names_of_the_group")
}

/// Python: bot/memory/relations.py:1224 `_did_you_mean(entry, property_, ask_gender='')` (never called on the trace corpus)
pub fn _did_you_mean(entry: &'static Entry, property_: &Value, ask_gender: &Value) -> String {
    crate::unported!("bot.memory.relations:_did_you_mean")
}

/// Python: bot/memory/relations.py:1252 `_relation_reply(memory, low)`
pub fn _relation_reply(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.relations:_relation_reply")
}

/// Python: bot/memory/relations.py:1398 `_pronoun_for(who, known=None)` (never called on the trace corpus)
pub fn _pronoun_for(who: &str, known: &Value) -> Value {
    crate::unported!("bot.memory.relations:_pronoun_for")
}

/// Python: bot/memory/relations.py:1416 `_the_group_word(who)` (never called on the trace corpus)
pub fn _the_group_word(who: &str) -> Value {
    crate::unported!("bot.memory.relations:_the_group_word")
}

/// Python: bot/memory/relations.py:1430 `_the_group_was_said(who, memory)` (never called on the trace corpus)
pub fn _the_group_was_said(who: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.relations:_the_group_was_said")
}

/// Python: bot/memory/relations.py:1452 `_ages_reply(memory, low, text='')`
pub fn _ages_reply(memory: &mut Memory, low: &str, text: &str) {
    crate::unported!("bot.memory.relations:_ages_reply")
}

/// Python: bot/memory/relations.py:1661 `_the_place_end(low)`
pub fn _the_place_end(low: &str) -> Vec<String> {
    crate::unported!("bot.memory.relations:_the_place_end")
}

/// Python: bot/memory/relations.py:1679 `_the_child_ages_on_file(memory)` (never called on the trace corpus)
pub fn _the_child_ages_on_file(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.relations:_the_child_ages_on_file")
}

/// Python: bot/memory/relations.py:1697 `_the_place_word_filed(memory, low)`
pub fn _the_place_word_filed(memory: &mut Memory, low: &str) -> String {
    crate::unported!("bot.memory.relations:_the_place_word_filed")
}

/// Python: bot/memory/relations.py:1732 `_pet_ages(memory)`
pub fn _pet_ages(memory: &mut Memory) -> Vec<String> {
    crate::unported!("bot.memory.relations:_pet_ages")
}

/// Python: bot/memory/relations.py:1738 `the_pronoun_said_with_a_name(memory, age)` (never called on the trace corpus)
pub fn the_pronoun_said_with_a_name(memory: &mut Memory, age: &Value) -> Option<String> {
    crate::unported!("bot.memory.relations:the_pronoun_said_with_a_name")
}

/// Python: bot/memory/relations.py:1761 `_pet_age_reply(memory, low)`
pub fn _pet_age_reply(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.relations:_pet_age_reply")
}

/// Python: bot/memory/relations.py:1815 `_pet_named(pet)` (never called on the trace corpus)
pub fn _pet_named(pet: &Value) -> Value {
    crate::unported!("bot.memory.relations:_pet_named")
}

/// Python: bot/memory/relations.py:1826 `_a_pet_age(pet)` (never called on the trace corpus)
pub fn _a_pet_age(pet: &Value) -> String {
    crate::unported!("bot.memory.relations:_a_pet_age")
}

/// Python: bot/memory/relations.py:1832 `_pet_ages_sentence(pets)` (never called on the trace corpus)
pub fn _pet_ages_sentence(pets: &Value) -> String {
    crate::unported!("bot.memory.relations:_pet_ages_sentence")
}

/// Python: bot/memory/relations.py:1845 `_one_age(who, age, turns='', to_come=None, held='')` (never called on the trace corpus)
pub fn _one_age(who: &str, age: &Value, turns: &[Turn], to_come: &Value, held: &Value) -> String {
    crate::unported!("bot.memory.relations:_one_age")
}

/// Python: bot/memory/relations.py:1879 `_just_gave_an_age(memory)`
pub fn _just_gave_an_age(memory: &Memory) -> bool {
    crate::unported!("bot.memory.relations:_just_gave_an_age")
}

/// Python: bot/memory/relations.py:1892 `_the_many(who)` (never called on the trace corpus)
pub fn _the_many(who: &str) -> Value {
    crate::unported!("bot.memory.relations:_the_many")
}

/// Python: bot/memory/relations.py:1907 `_ages_sentence(told)` (never called on the trace corpus)
pub fn _ages_sentence(told: &Value) -> String {
    crate::unported!("bot.memory.relations:_ages_sentence")
}

/// Python: bot/memory/relations.py:1939 `_ask_gender(low)` (never called on the trace corpus)
pub fn _ask_gender(low: &str) -> String {
    crate::unported!("bot.memory.relations:_ask_gender")
}

/// Python: bot/memory/relations.py:1948 `_called(name)` (never called on the trace corpus)
pub fn _called(name: &str) -> String {
    crate::unported!("bot.memory.relations:_called")
}

/// Python: bot/memory/relations.py:1961 `_every_line(memory)` (never called on the trace corpus)
pub fn _every_line(memory: &Memory) -> Value {
    crate::unported!("bot.memory.relations:_every_line")
}

