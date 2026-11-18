//! Port of `bot/memory/named.py`.
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

/// Python: bot/memory/named.py:35 `_answers_the_bot(text)`
pub fn _answers_the_bot(text: &str) -> bool {
    crate::unported!("bot.memory.named:_answers_the_bot")
}

/// Python: bot/memory/named.py:111 `_previous_ask(memory)`
pub fn _previous_ask(memory: &Memory) -> String {
    crate::unported!("bot.memory.named:_previous_ask")
}

/// Python: bot/memory/named.py:129 `takes_back(text)` (never called on the trace corpus)
pub fn takes_back(text: &str) -> bool {
    crate::unported!("bot.memory.named:takes_back")
}

/// Python: bot/memory/named.py:199 `the_one_asked_about(text)`
pub fn the_one_asked_about(text: &str) -> String {
    crate::unported!("bot.memory.named:the_one_asked_about")
}

/// Python: bot/memory/named.py:238 `thing(text)`
pub fn thing(text: &str) -> String {
    crate::unported!("bot.memory.named:thing")
}

/// Python: bot/memory/named.py:272 `_looks_like_a_thing(phrase)`
pub fn _looks_like_a_thing(phrase: &str) -> bool {
    crate::unported!("bot.memory.named:_looks_like_a_thing")
}

/// Python: bot/memory/named.py:295 `carries_a_name(text)` (never called on the trace corpus)
pub fn carries_a_name(text: &str) -> bool {
    crate::unported!("bot.memory.named:carries_a_name")
}

/// Python: bot/memory/named.py:300 `the_name_in(value)` (never called on the trace corpus)
pub fn the_name_in(value: &Value) -> String {
    crate::unported!("bot.memory.named:the_name_in")
}

/// Python: bot/memory/named.py:348 `asked_for_a_name(memory)`
pub fn asked_for_a_name(memory: &Memory) -> bool {
    crate::unported!("bot.memory.named:asked_for_a_name")
}

/// Python: bot/memory/named.py:357 `the_name_given(text, memory=None)`
pub fn the_name_given(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.named:the_name_given")
}

/// Python: bot/memory/named.py:411 `_not_got(text, memory=None)`
pub fn _not_got(text: &str, memory: &Value) -> String {
    crate::unported!("bot.memory.named:_not_got")
}

/// Python: bot/memory/named.py:469 `_said_the_food(memory, food)` (never called on the trace corpus)
pub fn _said_the_food(memory: &Memory, food: &Value) -> bool {
    crate::unported!("bot.memory.named:_said_the_food")
}

/// Python: bot/memory/named.py:475 `_last_reply(memory)` (never called on the trace corpus)
pub fn _last_reply(memory: &Memory) -> Value {
    crate::unported!("bot.memory.named:_last_reply")
}

/// Python: bot/memory/named.py:491 `the_thing_named(memory)`
pub fn the_thing_named(memory: &Memory) -> String {
    crate::unported!("bot.memory.named:the_thing_named")
}

/// Python: bot/memory/named.py:502 `about_the_thing(text, memory, owner='')`
pub fn about_the_thing(text: &str, memory: &Value, owner: &str) -> String {
    crate::unported!("bot.memory.named:about_the_thing")
}

/// Python: bot/memory/named.py:583 `_the_thing_they_named(memory)`
pub fn _the_thing_they_named(memory: &Memory) -> String {
    crate::unported!("bot.memory.named:_the_thing_they_named")
}

/// Python: bot/memory/named.py:596 `a_measure_asked_again(text, memory)`
pub fn a_measure_asked_again(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.named:a_measure_asked_again")
}

/// Python: bot/memory/named.py:704 `_a_length_ask(text)`
pub fn _a_length_ask(text: &str) -> bool {
    crate::unported!("bot.memory.named:_a_length_ask")
}

/// Python: bot/memory/named.py:727 `_a_length_ask_of_the_aim(text)`
pub fn _a_length_ask_of_the_aim(text: &str) -> bool {
    crate::unported!("bot.memory.named:_a_length_ask_of_the_aim")
}

/// Python: bot/memory/named.py:772 `_answers_a_pace(ask, memory)` (never called on the trace corpus)
pub fn _answers_a_pace(ask: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.named:_answers_a_pace")
}

/// Python: bot/memory/named.py:780 `_the_entry(ask, memory)` (never called on the trace corpus)
pub fn _the_entry(ask: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.named:_the_entry")
}

/// Python: bot/memory/named.py:791 `_answers_a_length(ask, memory)` (never called on the trace corpus)
pub fn _answers_a_length(ask: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.named:_answers_a_length")
}

/// Python: bot/memory/named.py:806 `_the_length_entry_about(ask, memory, aim_words)` (never called on the trace corpus)
pub fn _the_length_entry_about(ask: &str, memory: &Memory, aim_words: &[String]) -> Value {
    crate::unported!("bot.memory.named:_the_length_entry_about")
}

/// Python: bot/memory/named.py:829 `_answers_a_length_about(ask, memory, aim_words)` (never called on the trace corpus)
pub fn _answers_a_length_about(ask: &str, memory: &Memory, aim_words: &[String]) -> bool {
    crate::unported!("bot.memory.named:_answers_a_length_about")
}

/// Python: bot/memory/named.py:834 `_an_aim_before_the_side_question(turns, memory)` (never called on the trace corpus)
pub fn _an_aim_before_the_side_question(turns: &[Turn], memory: &Memory) -> Value {
    crate::unported!("bot.memory.named:_an_aim_before_the_side_question")
}

/// Python: bot/memory/named.py:854 `_the_aim_and_its_length_question(text, memory)`
pub fn _the_aim_and_its_length_question(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.named:_the_aim_and_its_length_question")
}

/// Python: bot/memory/named.py:956 `a_goal_measured(text, memory)`
pub fn a_goal_measured(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.named:a_goal_measured")
}

/// Python: bot/memory/named.py:965 `a_length_about_an_aim_on_a_deadline(text, memory)` (never called on the trace corpus)
pub fn a_length_about_an_aim_on_a_deadline(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.named:a_length_about_an_aim_on_a_deadline")
}

/// Python: bot/memory/named.py:990 `_no_thanks(text)`
pub fn _no_thanks(text: &str) -> bool {
    crate::unported!("bot.memory.named:_no_thanks")
}

/// Python: bot/memory/named.py:995 `follow_up(text, memory)`
pub fn follow_up(text: &str, memory: &Value) -> String {
    crate::unported!("bot.memory.named:follow_up")
}

/// Python: bot/memory/named.py:1072 `names_nothing_to_be_about(said)`
pub fn names_nothing_to_be_about(said: &str) -> bool {
    crate::unported!("bot.memory.named:names_nothing_to_be_about")
}

/// Python: bot/memory/named.py:1098 `_only_how_they_feel(text, memory)`
pub fn _only_how_they_feel(text: &str, memory: &Value) -> bool {
    crate::unported!("bot.memory.named:_only_how_they_feel")
}

/// Python: bot/memory/named.py:1134 `about_it(text, memory)`
pub fn about_it(text: &str, memory: &Value) -> String {
    crate::unported!("bot.memory.named:about_it")
}

/// Python: bot/memory/named.py:1231 `a_figure_on_the_doing(memory, word)` (never called on the trace corpus)
pub fn a_figure_on_the_doing(memory: &Memory, word: &str) -> Value {
    crate::unported!("bot.memory.named:a_figure_on_the_doing")
}

/// Python: bot/memory/named.py:1245 `is_a_progress_ask(text)` (never called on the trace corpus)
pub fn is_a_progress_ask(text: &str) -> bool {
    crate::unported!("bot.memory.named:is_a_progress_ask")
}

/// Python: bot/memory/named.py:1254 `_the_doing(word, memory)` (never called on the trace corpus)
pub fn _the_doing(word: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.named:_the_doing")
}

/// Python: bot/memory/named.py:1306 `the_doing_the_plan_named(word, memory)` (never called on the trace corpus)
pub fn the_doing_the_plan_named(word: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.named:the_doing_the_plan_named")
}

/// Python: bot/memory/named.py:1314 `_the_plan_said(memory, word)` (never called on the trace corpus)
pub fn _the_plan_said(memory: &Memory, word: &str) -> String {
    crate::unported!("bot.memory.named:_the_plan_said")
}

/// Python: bot/memory/named.py:1333 `_the_ing(word)` (never called on the trace corpus)
pub fn _the_ing(word: &str) -> Value {
    crate::unported!("bot.memory.named:_the_ing")
}

/// Python: bot/memory/named.py:1434 `_the_doing_in_hand(memory)`
pub fn _the_doing_in_hand(memory: &Memory) -> String {
    crate::unported!("bot.memory.named:_the_doing_in_hand")
}

/// Python: bot/memory/named.py:1477 `the_duration_for_the_doing(text, memory)`
pub fn the_duration_for_the_doing(text: &str, memory: &Value) -> String {
    crate::unported!("bot.memory.named:the_duration_for_the_doing")
}

/// Python: bot/memory/named.py:1511 `_the_place_belongs_to_the_doing(said, doing, memory)` (never called on the trace corpus)
pub fn _the_place_belongs_to_the_doing(said: &str, doing: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.memory.named:_the_place_belongs_to_the_doing")
}

/// Python: bot/memory/named.py:1556 `_the_aspect_asked(low, thing)`
pub fn _the_aspect_asked(low: &[String], thing: &str) -> String {
    crate::unported!("bot.memory.named:_the_aspect_asked")
}

/// Python: bot/memory/named.py:1593 `_the_job_before(memory)` (never called on the trace corpus)
pub fn _the_job_before(memory: &Memory) -> Value {
    crate::unported!("bot.memory.named:_the_job_before")
}

/// Python: bot/memory/named.py:1614 `the_best_time_to_do_it(text, understood, memory=None)`
pub fn the_best_time_to_do_it(text: &str, understood: &Understood, memory: &Memory) -> String {
    crate::unported!("bot.memory.named:the_best_time_to_do_it")
}

/// Python: bot/memory/named.py:1670 `_only_the_measure(low)`
pub fn _only_the_measure(low: &[String]) -> bool {
    crate::unported!("bot.memory.named:_only_the_measure")
}

/// Python: bot/memory/named.py:1682 `about_the_subject(text, memory)`
pub fn about_the_subject(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.named:about_the_subject")
}

/// Python: bot/memory/named.py:1793 `_without_and_at_the_start(said)`
pub fn _without_and_at_the_start(said: &str) -> String {
    crate::unported!("bot.memory.named:_without_and_at_the_start")
}

/// Python: bot/memory/named.py:1827 `_the_forms(base)` (never called on the trace corpus)
pub fn _the_forms(base: &Value) -> Value {
    crate::unported!("bot.memory.named:_the_forms")
}

/// Python: bot/memory/named.py:1842 `_the_doing_word(text)` (never called on the trace corpus)
pub fn _the_doing_word(text: &str) -> Value {
    crate::unported!("bot.memory.named:_the_doing_word")
}

/// Python: bot/memory/named.py:1851 `_a_doing_word(text)` (never called on the trace corpus)
pub fn _a_doing_word(text: &str) -> bool {
    crate::unported!("bot.memory.named:_a_doing_word")
}

/// Python: bot/memory/named.py:1855 `_what_the_chat_is_on(memory, this_line)` (never called on the trace corpus)
pub fn _what_the_chat_is_on(memory: &Memory, this_line: &str) -> Value {
    crate::unported!("bot.memory.named:_what_the_chat_is_on")
}

/// Python: bot/memory/named.py:1871 `_the_thing_in_hand(memory, this_line)` (never called on the trace corpus)
pub fn _the_thing_in_hand(memory: &Memory, this_line: &str) -> Value {
    crate::unported!("bot.memory.named:_the_thing_in_hand")
}

/// Python: bot/memory/named.py:1883 `_the_place_phrase(low)`
pub fn _the_place_phrase(low: &[String]) -> String {
    crate::unported!("bot.memory.named:_the_place_phrase")
}

/// Python: bot/memory/named.py:1898 `_about_the_thing_in_the_place(low, place, said_the_thing, this_line, memory)` (never called on the trace corpus)
pub fn _about_the_thing_in_the_place(low: &str, place: &str, said_the_thing: &Value, this_line: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.named:_about_the_thing_in_the_place")
}

/// Python: bot/memory/named.py:1934 `_the_rewrite_earns_its_place(alone, written, memory)` (never called on the trace corpus)
pub fn _the_rewrite_earns_its_place(alone: &Value, written: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.memory.named:_the_rewrite_earns_its_place")
}

/// Python: bot/memory/named.py:1975 `_thing_in_hand(memory)`
pub fn _thing_in_hand(memory: &Memory) -> Value {
    crate::unported!("bot.memory.named:_thing_in_hand")
}

/// Python: bot/memory/named.py:2015 `_only_a_particle(word, text)`
pub fn _only_a_particle(word: &str, text: &str) -> bool {
    crate::unported!("bot.memory.named:_only_a_particle")
}

/// Python: bot/memory/named.py:2042 `_the_answers_own_subject(answer)`
pub fn _the_answers_own_subject(answer: &Value) -> Vec<String> {
    crate::unported!("bot.memory.named:_the_answers_own_subject")
}

/// Python: bot/memory/named.py:2059 `reported_states(text, memory)`
pub fn reported_states(text: &str, memory: &Value) -> Vec<String> {
    crate::unported!("bot.memory.named:reported_states")
}

/// Python: bot/memory/named.py:2146 `_the_persons_own_words(memory)` (never called on the trace corpus)
pub fn _the_persons_own_words(memory: &Memory) -> Value {
    crate::unported!("bot.memory.named:_the_persons_own_words")
}

/// Python: bot/memory/named.py:2274 `stated_about_the_person(text, memory)`
pub fn stated_about_the_person(text: &str, memory: &Value) -> Vec<String> {
    crate::unported!("bot.memory.named:stated_about_the_person")
}

/// Python: bot/memory/named.py:2330 `_a_place_of_its_own(memory)`
pub fn _a_place_of_its_own(memory: &Memory) -> bool {
    crate::unported!("bot.memory.named:_a_place_of_its_own")
}

/// Python: bot/memory/named.py:2341 `subject_words(memory, text='')`
pub fn subject_words(memory: &Memory, text: &str) -> Vec<String> {
    crate::unported!("bot.memory.named:subject_words")
}

/// Python: bot/memory/named.py:2372 `care_reports(text, memory)`
pub fn care_reports(text: &str, memory: &Value) -> Vec<String> {
    crate::unported!("bot.memory.named:care_reports")
}

/// Python: bot/memory/named.py:2424 `a_faulty_thing(memory)`
pub fn a_faulty_thing(memory: &Memory) -> bool {
    crate::unported!("bot.memory.named:a_faulty_thing")
}

/// Python: bot/memory/named.py:2452 `_the_answer_gave_a_time(memory)`
pub fn _the_answer_gave_a_time(memory: &Memory) -> bool {
    crate::unported!("bot.memory.named:_the_answer_gave_a_time")
}

/// Python: bot/memory/named.py:2460 `the_thing(memory)` (never called on the trace corpus)
pub fn the_thing(memory: &Memory) -> String {
    crate::unported!("bot.memory.named:the_thing")
}

/// Python: bot/memory/named.py:2477 `subject_word(memory)`
pub fn subject_word(memory: &Value) -> String {
    crate::unported!("bot.memory.named:subject_word")
}

/// Python: bot/memory/named.py:2504 `_the_noun_in(said)`
pub fn _the_noun_in(said: &str) -> Vec<String> {
    crate::unported!("bot.memory.named:_the_noun_in")
}

/// Python: bot/memory/named.py:2513 `_the_record_before(memory)`
pub fn _the_record_before(memory: &Memory) {
    crate::unported!("bot.memory.named:_the_record_before")
}

/// Python: bot/memory/named.py:2539 `a_newer_thing(memory, word)`
pub fn a_newer_thing(memory: &Memory, word: &str) -> String {
    crate::unported!("bot.memory.named:a_newer_thing")
}

/// Python: bot/memory/named.py:2632 `_the_trip_place(memory)`
pub fn _the_trip_place(memory: &Memory) -> String {
    crate::unported!("bot.memory.named:_the_trip_place")
}

/// Python: bot/memory/named.py:2690 `_the_opening_place(memory)`
pub fn _the_opening_place(memory: &Memory) -> String {
    crate::unported!("bot.memory.named:_the_opening_place")
}

/// Python: bot/memory/named.py:2700 `_the_opening_place_at(memory)`
pub fn _the_opening_place_at(memory: &Memory) -> Value {
    crate::unported!("bot.memory.named:_the_opening_place_at")
}

/// Python: bot/memory/named.py:2731 `_the_place_the_bot_just_named(memory)`
pub fn _the_place_the_bot_just_named(memory: &Memory) -> String {
    crate::unported!("bot.memory.named:_the_place_the_bot_just_named")
}

/// Python: bot/memory/named.py:2750 `a_part_of_the_trip(text, memory)`
pub fn a_part_of_the_trip(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.named:a_part_of_the_trip")
}

/// Python: bot/memory/named.py:2794 `asks_a_part_of_the_trip(text, memory)`
pub fn asks_a_part_of_the_trip(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.named:asks_a_part_of_the_trip")
}

/// Python: bot/memory/named.py:2821 `the_food_it_is_about(understood, memory)`
pub fn the_food_it_is_about(understood: &Understood, memory: &Memory) -> String {
    crate::unported!("bot.memory.named:the_food_it_is_about")
}

