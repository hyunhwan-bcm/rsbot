//! Port of `bot/memory/ack.py`.
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

/// Python: bot/memory/ack.py:241 `_a_relation(word)`
pub fn _a_relation(word: &str) -> String {
    crate::unported!("bot.memory.ack:_a_relation")
}

/// Python: bot/memory/ack.py:250 `_the_pronoun(relation)` (never called on the trace corpus)
pub fn _the_pronoun(relation: &Value) -> String {
    crate::unported!("bot.memory.ack:_the_pronoun")
}

/// Python: bot/memory/ack.py:259 `_the_ages_held(memory)` (never called on the trace corpus)
pub fn _the_ages_held(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.ack:_the_ages_held")
}

/// Python: bot/memory/ack.py:269 `_people_in_chat(memory)` (never called on the trace corpus)
pub fn _people_in_chat(memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.ack:_people_in_chat")
}

/// Python: bot/memory/ack.py:286 `a_person_named_alone(text, memory)`
pub fn a_person_named_alone(text: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.memory.ack:a_person_named_alone")
}

/// Python: bot/memory/ack.py:335 `_the_question_about(relation, said, ours, memory, name='')` (never called on the trace corpus)
pub fn _the_question_about(relation: &Value, said: &str, ours: &Value, memory: &mut Memory, name: &str) -> String {
    crate::unported!("bot.memory.ack:_the_question_about")
}

/// Python: bot/memory/ack.py:359 `a_person_pointed_at(text, memory)`
pub fn a_person_pointed_at(text: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.memory.ack:a_person_pointed_at")
}

/// Python: bot/memory/ack.py:389 `_how_many_people(memory)` (never called on the trace corpus)
pub fn _how_many_people(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.ack:_how_many_people")
}

/// Python: bot/memory/ack.py:396 `_the_people_pointed_at(memory)` (never called on the trace corpus)
pub fn _the_people_pointed_at(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.ack:_the_people_pointed_at")
}

/// Python: bot/memory/ack.py:414 `_an_age_held(memory, name)` (never called on the trace corpus)
pub fn _an_age_held(memory: &mut Memory, name: &str) -> bool {
    crate::unported!("bot.memory.ack:_an_age_held")
}

/// Python: bot/memory/ack.py:426 `_the_person_pointed_at(word, memory)` (never called on the trace corpus)
pub fn _the_person_pointed_at(word: &str, memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.ack:_the_person_pointed_at")
}

/// Python: bot/memory/ack.py:444 `a_second_person(text, memory)`
pub fn a_second_person(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.ack:a_second_person")
}

/// Python: bot/memory/ack.py:586 `_the_last_reply_gave_a_plan(memory)`
pub fn _the_last_reply_gave_a_plan(memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:_the_last_reply_gave_a_plan")
}

/// Python: bot/memory/ack.py:603 `_a_limit_on_the_plan(read, memory)`
pub fn _a_limit_on_the_plan(read: &str, memory: &Memory) {
    crate::unported!("bot.memory.ack:_a_limit_on_the_plan")
}

/// Python: bot/memory/ack.py:739 `_a_day_coming(clean)`
pub fn _a_day_coming(clean: &str) -> Option<Vec<String>> {
    crate::unported!("bot.memory.ack:_a_day_coming")
}

/// Python: bot/memory/ack.py:785 `reads_as_a_receipt(text)`
pub fn reads_as_a_receipt(text: &str) -> bool {
    crate::unported!("bot.memory.ack:reads_as_a_receipt")
}

/// Python: bot/memory/ack.py:813 `_one_frame(said)`
pub fn _one_frame(said: &str) -> bool {
    crate::unported!("bot.memory.ack:_one_frame")
}

/// Python: bot/memory/ack.py:978 `_more_than_one_coming(said, start, word)` (never called on the trace corpus)
pub fn _more_than_one_coming(said: &str, start: i64, word: &str) -> bool {
    crate::unported!("bot.memory.ack:_more_than_one_coming")
}

/// Python: bot/memory/ack.py:1085 `reads_as_a_trip(clean)`
pub fn reads_as_a_trip(clean: &str) -> Option<String> {
    crate::unported!("bot.memory.ack:reads_as_a_trip")
}

/// Python: bot/memory/ack.py:1345 `_the_plan_in_hand(memory, clean='')` (never called on the trace corpus)
pub fn _the_plan_in_hand(memory: &Memory, clean: &str) -> Option<String> {
    crate::unported!("bot.memory.ack:_the_plan_in_hand")
}

/// Python: bot/memory/ack.py:1423 `_a_like_of_one_thing(clean, memory=None)`
pub fn _a_like_of_one_thing(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:_a_like_of_one_thing")
}

/// Python: bot/memory/ack.py:1436 `_only_a_liking(found)`
pub fn _only_a_liking(found: &Value) -> bool {
    crate::unported!("bot.memory.ack:_only_a_liking")
}

/// Python: bot/memory/ack.py:1448 `a_liked_thing(found)`
pub fn a_liked_thing(found: &Value) -> String {
    crate::unported!("bot.memory.ack:a_liked_thing")
}

/// Python: bot/memory/ack.py:1485 `a_limit_reply(text, memory=None)`
pub fn a_limit_reply(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.ack:a_limit_reply")
}

/// Python: bot/memory/ack.py:1519 `_a_plan_on_the_table(memory, clean='')`
pub fn _a_plan_on_the_table(memory: &Memory, clean: &str) -> bool {
    crate::unported!("bot.memory.ack:_a_plan_on_the_table")
}

/// Python: bot/memory/ack.py:1576 `_a_limit_reply(clean, family, memory)` (never called on the trace corpus)
pub fn _a_limit_reply(clean: &str, family: &Value, memory: &Memory) -> String {
    crate::unported!("bot.memory.ack:_a_limit_reply")
}

/// Python: bot/memory/ack.py:1591 `a_limit_for_the_plan(text, memory=None)`
pub fn a_limit_for_the_plan(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.ack:a_limit_for_the_plan")
}

/// Python: bot/memory/ack.py:1698 `reads_as_cruelty(said)`
pub fn reads_as_cruelty(said: &str) -> bool {
    crate::unported!("bot.memory.ack:reads_as_cruelty")
}

/// Python: bot/memory/ack.py:1844 `reads_as_an_ask(line)`
pub fn reads_as_an_ask(line: &str) -> bool {
    crate::unported!("bot.memory.ack:reads_as_an_ask")
}

/// Python: bot/memory/ack.py:1895 `_shape_of(when)` (never called on the trace corpus)
pub fn _shape_of(when: &str) -> String {
    crate::unported!("bot.memory.ack:_shape_of")
}

/// Python: bot/memory/ack.py:2038 `_clock_shape(when)` (never called on the trace corpus)
pub fn _clock_shape(when: &str) -> String {
    crate::unported!("bot.memory.ack:_clock_shape")
}

/// Python: bot/memory/ack.py:2074 `_tells_them_what_to_do(said)` (never called on the trace corpus)
pub fn _tells_them_what_to_do(said: &str) -> bool {
    crate::unported!("bot.memory.ack:_tells_them_what_to_do")
}

/// Python: bot/memory/ack.py:2120 `_a_bare_state(clean)`
pub fn _a_bare_state(clean: &str) -> bool {
    crate::unported!("bot.memory.ack:_a_bare_state")
}

/// Python: bot/memory/ack.py:2236 `has_news(text)`
pub fn has_news(text: &str) -> bool {
    crate::unported!("bot.memory.ack:has_news")
}

/// Python: bot/memory/ack.py:2365 `_repaired(clean, memory)`
pub fn _repaired(clean: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.ack:_repaired")
}

/// Python: bot/memory/ack.py:2451 `_the_values_measures(value)` (never called on the trace corpus)
pub fn _the_values_measures(value: &Value) -> Value {
    crate::unported!("bot.memory.ack:_the_values_measures")
}

/// Python: bot/memory/ack.py:2473 `_the_units_measure(unit)` (never called on the trace corpus)
pub fn _the_units_measure(unit: &str) -> Value {
    crate::unported!("bot.memory.ack:_the_units_measure")
}

/// Python: bot/memory/ack.py:2483 `_the_unit_is_said_twice(value, unit)` (never called on the trace corpus)
pub fn _the_unit_is_said_twice(value: &Value, unit: &str) -> bool {
    crate::unported!("bot.memory.ack:_the_unit_is_said_twice")
}

/// Python: bot/memory/ack.py:2521 `_without_a_frame(words, phrases, back=False)`
pub fn _without_a_frame(words: &[String], phrases: &Value, back: bool) -> Value {
    crate::unported!("bot.memory.ack:_without_a_frame")
}

/// Python: bot/memory/ack.py:2538 `_without_a_not(words)`
pub fn _without_a_not(words: &[String]) -> Value {
    crate::unported!("bot.memory.ack:_without_a_not")
}

/// Python: bot/memory/ack.py:2546 `_the_bare_value(clean)`
pub fn _the_bare_value(clean: &str) -> String {
    crate::unported!("bot.memory.ack:_the_bare_value")
}

/// Python: bot/memory/ack.py:2566 `value_put_right(clean, memory)`
pub fn value_put_right(clean: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.ack:value_put_right")
}

/// Python: bot/memory/ack.py:2615 `_without_a_unit_already_in_the_value(value, unit)` (never called on the trace corpus)
pub fn _without_a_unit_already_in_the_value(value: &Value, unit: &str) -> Value {
    crate::unported!("bot.memory.ack:_without_a_unit_already_in_the_value")
}

/// Python: bot/memory/ack.py:2633 `_the_line_before(memory)`
pub fn _the_line_before(memory: &Memory) -> String {
    crate::unported!("bot.memory.ack:_the_line_before")
}

/// Python: bot/memory/ack.py:2645 `_the_unit_before(memory)` (never called on the trace corpus)
pub fn _the_unit_before(memory: &Memory) -> String {
    crate::unported!("bot.memory.ack:_the_unit_before")
}

/// Python: bot/memory/ack.py:2710 `_the_animal_given_away(text, memory=None)` (never called on the trace corpus)
pub fn _the_animal_given_away(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.ack:_the_animal_given_away")
}

/// Python: bot/memory/ack.py:2734 `pet_given_away(text, memory=None)`
pub fn pet_given_away(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.ack:pet_given_away")
}

/// Python: bot/memory/ack.py:2785 `_offers_a_chat(clean, memory=None)`
pub fn _offers_a_chat(clean: &str, memory: &Memory) {
    crate::unported!("bot.memory.ack:_offers_a_chat")
}

/// Python: bot/memory/ack.py:2859 `_the_plan_a_date_belongs_to(text)` (never called on the trace corpus)
pub fn _the_plan_a_date_belongs_to(text: &str) -> Value {
    crate::unported!("bot.memory.ack:_the_plan_a_date_belongs_to")
}

/// Python: bot/memory/ack.py:2888 `_a_date_on_the_thing_in_hand(clean, memory)`
pub fn _a_date_on_the_thing_in_hand(clean: &str, memory: &Memory) {
    crate::unported!("bot.memory.ack:_a_date_on_the_thing_in_hand")
}

/// Python: bot/memory/ack.py:2947 `_after_the_hello(clean)` (never called on the trace corpus)
pub fn _after_the_hello(clean: &str) -> String {
    crate::unported!("bot.memory.ack:_after_the_hello")
}

/// Python: bot/memory/ack.py:2955 `reply(text, memory)`
pub fn reply(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.memory.ack:reply")
}

/// Python: bot/memory/ack.py:3451 `_the_opening_instead(said, clean, memory)`
pub fn _the_opening_instead(said: Option<&str>, clean: &str, memory: &Memory) {
    crate::unported!("bot.memory.ack:_the_opening_instead")
}

/// Python: bot/memory/ack.py:3465 `_a_hand_back(reply, memory)`
pub fn _a_hand_back(reply: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.ack:_a_hand_back")
}

/// Python: bot/memory/ack.py:3488 `_a_remark_on_what_was_said(clean, memory)`
pub fn _a_remark_on_what_was_said(clean: &str, memory: &Memory) -> Option<String> {
    crate::unported!("bot.memory.ack:_a_remark_on_what_was_said")
}

/// Python: bot/memory/ack.py:3529 `_a_price_still_to_come(body, topic_text)` (never called on the trace corpus)
pub fn _a_price_still_to_come(body: &str, topic_text: &str) -> bool {
    crate::unported!("bot.memory.ack:_a_price_still_to_come")
}

/// Python: bot/memory/ack.py:3547 `_a_result_is_on_the_table(clean)`
pub fn _a_result_is_on_the_table(clean: &str) -> bool {
    crate::unported!("bot.memory.ack:_a_result_is_on_the_table")
}

/// Python: bot/memory/ack.py:3565 `_ending_of(frame)`
pub fn _ending_of(frame: &str) -> String {
    crate::unported!("bot.memory.ack:_ending_of")
}

/// Python: bot/memory/ack.py:3571 `_pick_ending(frames, memory)`
pub fn _pick_ending(frames: &[String], memory: &Memory) -> String {
    crate::unported!("bot.memory.ack:_pick_ending")
}

/// Python: bot/memory/ack.py:3626 `_after_a_feeling(clean, memory)`
pub fn _after_a_feeling(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:_after_a_feeling")
}

/// Python: bot/memory/ack.py:3692 `_a_heavy_reply_just_before(memory, index)`
pub fn _a_heavy_reply_just_before(memory: &Memory, index: i64) -> bool {
    crate::unported!("bot.memory.ack:_a_heavy_reply_just_before")
}

/// Python: bot/memory/ack.py:3730 `_bad_news_just_before(clean, memory)`
pub fn _bad_news_just_before(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:_bad_news_just_before")
}

/// Python: bot/memory/ack.py:3759 `carries_the_story_on(text, memory)`
pub fn carries_the_story_on(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:carries_the_story_on")
}

/// Python: bot/memory/ack.py:3828 `the_old_one_aged(clean, memory)`
pub fn the_old_one_aged(clean: &str, memory: &Memory) {
    crate::unported!("bot.memory.ack:the_old_one_aged")
}

/// Python: bot/memory/ack.py:3866 `_the_length(clean, memory)`
pub fn _the_length(clean: &str, memory: &Memory) {
    crate::unported!("bot.memory.ack:_the_length")
}

/// Python: bot/memory/ack.py:3903 `_the_hours(clean, read, memory)`
pub fn _the_hours(clean: &str, read: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.ack:_the_hours")
}

/// Python: bot/memory/ack.py:3944 `_a_deadline(clean, read, memory)`
pub fn _a_deadline(clean: &str, read: &str, memory: &Memory) {
    crate::unported!("bot.memory.ack:_a_deadline")
}

/// Python: bot/memory/ack.py:3977 `_an_hour_on_the_day(text)` (never called on the trace corpus)
pub fn _an_hour_on_the_day(text: &str) -> Option<String> {
    crate::unported!("bot.memory.ack:_an_hour_on_the_day")
}

/// Python: bot/memory/ack.py:4006 `_the_figure_is_an_hour(when)` (never called on the trace corpus)
pub fn _the_figure_is_an_hour(when: &str) -> bool {
    crate::unported!("bot.memory.ack:_the_figure_is_an_hour")
}

/// Python: bot/memory/ack.py:4014 `_a_clock_time(clean, read, memory)`
pub fn _a_clock_time(clean: &str, read: &str, memory: &Memory) {
    crate::unported!("bot.memory.ack:_a_clock_time")
}

/// Python: bot/memory/ack.py:4075 `_hours_family(read)`
pub fn _hours_family(read: &str) -> Value {
    crate::unported!("bot.memory.ack:_hours_family")
}

/// Python: bot/memory/ack.py:4143 `_a_pet_that_is_all_right(clean, kept)` (never called on the trace corpus)
pub fn _a_pet_that_is_all_right(clean: &str, kept: &Value) -> bool {
    crate::unported!("bot.memory.ack:_a_pet_that_is_all_right")
}

/// Python: bot/memory/ack.py:4174 `_nothing_added_but_the_pet(clean, kept)`
pub fn _nothing_added_but_the_pet(clean: &str, kept: &Value) -> bool {
    crate::unported!("bot.memory.ack:_nothing_added_but_the_pet")
}

/// Python: bot/memory/ack.py:4255 `_a_wanted_animal(clean)`
pub fn _a_wanted_animal(clean: &str) -> bool {
    crate::unported!("bot.memory.ack:_a_wanted_animal")
}

/// Python: bot/memory/ack.py:4263 `_an_animal_wanted(clean)`
pub fn _an_animal_wanted(clean: &str) -> String {
    crate::unported!("bot.memory.ack:_an_animal_wanted")
}

/// Python: bot/memory/ack.py:4281 `_an_animal_they_had_once(clean)` (never called on the trace corpus)
pub fn _an_animal_they_had_once(clean: &str) -> bool {
    crate::unported!("bot.memory.ack:_an_animal_they_had_once")
}

/// Python: bot/memory/ack.py:4292 `_what_a_wanted_animal_needs(animal, clean, memory)`
pub fn _what_a_wanted_animal_needs(animal: &str, clean: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.ack:_what_a_wanted_animal_needs")
}

/// Python: bot/memory/ack.py:4347 `_the_pets_are_coming(pets, memory)` (never called on the trace corpus)
pub fn _the_pets_are_coming(pets: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:_the_pets_are_coming")
}

/// Python: bot/memory/ack.py:4381 `_the_animal_coming(found)`
pub fn _the_animal_coming(found: &Value) -> String {
    crate::unported!("bot.memory.ack:_the_animal_coming")
}

/// Python: bot/memory/ack.py:4392 `_the_taken_back_animal(found)` (never called on the trace corpus)
pub fn _the_taken_back_animal(found: &Value) -> Value {
    crate::unported!("bot.memory.ack:_the_taken_back_animal")
}

/// Python: bot/memory/ack.py:4402 `_an_animal_turned_down(found, memory)`
pub fn _an_animal_turned_down(found: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:_an_animal_turned_down")
}

/// Python: bot/memory/ack.py:4431 `_on_record(animal, memory)`
pub fn _on_record(animal: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:_on_record")
}

/// Python: bot/memory/ack.py:4460 `_an_animal_only_planned(found, clean, memory)`
pub fn _an_animal_only_planned(found: &Value, clean: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.ack:_an_animal_only_planned")
}

/// Python: bot/memory/ack.py:4485 `_the_wished_animal(found, clean='')` (never called on the trace corpus)
pub fn _the_wished_animal(found: &Value, clean: &str) -> Value {
    crate::unported!("bot.memory.ack:_the_wished_animal")
}

/// Python: bot/memory/ack.py:4496 `_only_a_wish(found)`
pub fn _only_a_wish(found: &Value) -> bool {
    crate::unported!("bot.memory.ack:_only_a_wish")
}

/// Python: bot/memory/ack.py:4504 `_an_animal_taken_back(found)`
pub fn _an_animal_taken_back(found: &Value) -> bool {
    crate::unported!("bot.memory.ack:_an_animal_taken_back")
}

/// Python: bot/memory/ack.py:4520 `_the_pets_said(kept, clean='')` (never called on the trace corpus)
pub fn _the_pets_said(kept: &Value, clean: &str) -> Value {
    crate::unported!("bot.memory.ack:_the_pets_said")
}

/// Python: bot/memory/ack.py:4576 `_plain(clean, memory)`
pub fn _plain(clean: &str, memory: &Memory) -> Option<String> {
    crate::unported!("bot.memory.ack:_plain")
}

/// Python: bot/memory/ack.py:4607 `_about_them(body)`
pub fn _about_them(body: &str) -> bool {
    crate::unported!("bot.memory.ack:_about_them")
}

/// Python: bot/memory/ack.py:4699 `_who_is_getting_better(low)`
pub fn _who_is_getting_better(low: &str) -> String {
    crate::unported!("bot.memory.ack:_who_is_getting_better")
}

/// Python: bot/memory/ack.py:4709 `a_person_getting_better(clean, memory=None)`
pub fn a_person_getting_better(clean: &str, memory: &Memory) {
    crate::unported!("bot.memory.ack:a_person_getting_better")
}

/// Python: bot/memory/ack.py:4734 `_the_words_of_the_last_two_turns(memory)`
pub fn _the_words_of_the_last_two_turns(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.memory.ack:_the_words_of_the_last_two_turns")
}

/// Python: bot/memory/ack.py:4745 `_with_the_time(pool, low)` (never called on the trace corpus)
pub fn _with_the_time(pool: &Value, low: &str) -> Value {
    crate::unported!("bot.memory.ack:_with_the_time")
}

/// Python: bot/memory/ack.py:4789 `nobody_around(clean, memory)`
pub fn nobody_around(clean: &str, memory: &Memory) {
    crate::unported!("bot.memory.ack:nobody_around")
}

/// Python: bot/memory/ack.py:4827 `_said_to_the_bot(clean)`
pub fn _said_to_the_bot(clean: &str) -> bool {
    crate::unported!("bot.memory.ack:_said_to_the_bot")
}

/// Python: bot/memory/ack.py:4834 `_no_subject(clean, memory=None)`
pub fn _no_subject(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:_no_subject")
}

/// Python: bot/memory/ack.py:4852 `_asked_last(memory)`
pub fn _asked_last(memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:_asked_last")
}

/// Python: bot/memory/ack.py:4861 `_a_diary_plan(found, clean)`
pub fn _a_diary_plan(found: &Value, clean: &str) {
    crate::unported!("bot.memory.ack:_a_diary_plan")
}

/// Python: bot/memory/ack.py:4907 `_plan_tails(found, clean, memory)`
pub fn _plan_tails(found: &Value, clean: &str, memory: &Memory) {
    crate::unported!("bot.memory.ack:_plan_tails")
}

/// Python: bot/memory/ack.py:4945 `_about_a_body(read, memory=None)`
pub fn _about_a_body(read: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:_about_a_body")
}

/// Python: bot/memory/ack.py:5012 `the_job_they_had(clean, memory)`
pub fn the_job_they_had(clean: &str, memory: &Memory) {
    crate::unported!("bot.memory.ack:the_job_they_had")
}

/// Python: bot/memory/ack.py:5033 `a_job_to_do(text, memory=None)`
pub fn a_job_to_do(text: &str, memory: &Memory) -> Option<String> {
    crate::unported!("bot.memory.ack:a_job_to_do")
}

/// Python: bot/memory/ack.py:5045 `_a_job_to_do(clean, memory=None)`
pub fn _a_job_to_do(clean: &str, memory: &Memory) -> Option<String> {
    crate::unported!("bot.memory.ack:_a_job_to_do")
}

/// Python: bot/memory/ack.py:5127 `_the_chat_holds_something(memory)` (never called on the trace corpus)
pub fn _the_chat_holds_something(memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:_the_chat_holds_something")
}

/// Python: bot/memory/ack.py:5139 `_the_pronoun_points_at_somebody(low, memory)` (never called on the trace corpus)
pub fn _the_pronoun_points_at_somebody(low: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.ack:_the_pronoun_points_at_somebody")
}

/// Python: bot/memory/ack.py:5187 `_somebody_was_being_hurt(memory)` (never called on the trace corpus)
pub fn _somebody_was_being_hurt(memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:_somebody_was_being_hurt")
}

/// Python: bot/memory/ack.py:5201 `_who_the_figure_is(found, memory, low)`
pub fn _who_the_figure_is(found: &Value, memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.ack:_who_the_figure_is")
}

/// Python: bot/memory/ack.py:5245 `_what_the_pointer_is(read, memory)` (never called on the trace corpus)
pub fn _what_the_pointer_is(read: &Understood, memory: &Memory) -> Value {
    crate::unported!("bot.memory.ack:_what_the_pointer_is")
}

/// Python: bot/memory/ack.py:5322 `_the_route_for(thing)` (never called on the trace corpus)
pub fn _the_route_for(thing: &str) -> Value {
    crate::unported!("bot.memory.ack:_the_route_for")
}

/// Python: bot/memory/ack.py:5331 `_the_entry_names(entry, head)` (never called on the trace corpus)
pub fn _the_entry_names(entry: &'static Entry, head: &str) -> bool {
    crate::unported!("bot.memory.ack:_the_entry_names")
}

/// Python: bot/memory/ack.py:5349 `_a_condition_they_wonder_about(clean, memory=None)`
pub fn _a_condition_they_wonder_about(clean: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.ack:_a_condition_they_wonder_about")
}

/// Python: bot/memory/ack.py:5396 `_a_thing_they_have(clean, memory=None)`
pub fn _a_thing_they_have(clean: &str, memory: &Memory) {
    crate::unported!("bot.memory.ack:_a_thing_they_have")
}

/// Python: bot/memory/ack.py:5437 `_a_spending_budget(clean, memory)` (never called on the trace corpus)
pub fn _a_spending_budget(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:_a_spending_budget")
}

/// Python: bot/memory/ack.py:5476 `_frames(found, heavy=False, clean='', memory=None)`
pub fn _frames(found: &Value, heavy: bool, clean: &str, memory: &Memory) -> Vec<String> {
    crate::unported!("bot.memory.ack:_frames")
}

/// Python: bot/memory/ack.py:5533 `_about_someone_else(found, clean='')`
pub fn _about_someone_else(found: &Value, clean: &str) -> bool {
    crate::unported!("bot.memory.ack:_about_someone_else")
}

/// Python: bot/memory/ack.py:5545 `_a_visit(clean)` (never called on the trace corpus)
pub fn _a_visit(clean: &str) -> bool {
    crate::unported!("bot.memory.ack:_a_visit")
}

/// Python: bot/memory/ack.py:5559 `_the_milestone(clean)` (never called on the trace corpus)
pub fn _the_milestone(clean: &str) -> bool {
    crate::unported!("bot.memory.ack:_the_milestone")
}

/// Python: bot/memory/ack.py:5568 `_a_workplace(found, heavy=False, memory=None)`
pub fn _a_workplace(found: &Value, heavy: bool, memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:_a_workplace")
}

/// Python: bot/memory/ack.py:5594 `_about_the_family(found)` (never called on the trace corpus)
pub fn _about_the_family(found: &Value) -> bool {
    crate::unported!("bot.memory.ack:_about_the_family")
}

/// Python: bot/memory/ack.py:5604 `_just_a_liking(found)` (never called on the trace corpus)
pub fn _just_a_liking(found: &Value) -> bool {
    crate::unported!("bot.memory.ack:_just_a_liking")
}

/// Python: bot/memory/ack.py:5617 `_heavy(memory)`
pub fn _heavy(memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:_heavy")
}

/// Python: bot/memory/ack.py:5635 `_statement(memory)`
pub fn _statement(memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:_statement")
}

/// Python: bot/memory/ack.py:5644 `_a_loss(text, memory, clean)`
pub fn _a_loss(text: &str, memory: &Memory, clean: &str) -> bool {
    crate::unported!("bot.memory.ack:_a_loss")
}

/// Python: bot/memory/ack.py:5651 `_learned(text, memory=None)`
pub fn _learned(text: &str, memory: Option<&Memory>) -> Value {
    crate::unported!("bot.memory.ack:_learned")
}

/// Python: bot/memory/ack.py:5673 `_children_with_ages(text)`
pub fn _children_with_ages(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.ack:_children_with_ages")
}

/// Python: bot/memory/ack.py:5682 `_children_said(kids)` (never called on the trace corpus)
pub fn _children_said(kids: &Value) -> String {
    crate::unported!("bot.memory.ack:_children_said")
}

/// Python: bot/memory/ack.py:5693 `_adds_one_more_allergen(text, memory)`
pub fn _adds_one_more_allergen(text: &str, memory: Option<&Memory>) -> Vec<String> {
    crate::unported!("bot.memory.ack:_adds_one_more_allergen")
}

/// Python: bot/memory/ack.py:5712 `_adds_a_detail(text, memory)`
pub fn _adds_a_detail(text: &str, memory: Option<&Memory>) -> bool {
    crate::unported!("bot.memory.ack:_adds_a_detail")
}

/// Python: bot/memory/ack.py:5780 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.ack:_words")
}

/// Python: bot/memory/ack.py:5798 `_names_nothing(body)`
pub fn _names_nothing(body: &str) -> bool {
    crate::unported!("bot.memory.ack:_names_nothing")
}

/// Python: bot/memory/ack.py:5817 `_repairs(clean, memory)`
pub fn _repairs(clean: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.ack:_repairs")
}

/// Python: bot/memory/ack.py:5907 `_says_something(body)`
pub fn _says_something(body: &str) -> bool {
    crate::unported!("bot.memory.ack:_says_something")
}

/// Python: bot/memory/ack.py:6003 `_the_person_the_pointer_is_about(body, memory)` (never called on the trace corpus)
pub fn _the_person_the_pointer_is_about(body: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.ack:_the_person_the_pointer_is_about")
}

/// Python: bot/memory/ack.py:6022 `_the_thing_the_chat_is_on(memory)` (never called on the trace corpus)
pub fn _the_thing_the_chat_is_on(memory: &Memory) -> Value {
    crate::unported!("bot.memory.ack:_the_thing_the_chat_is_on")
}

/// Python: bot/memory/ack.py:6073 `_a_name_the_store_holds(memory, stem)` (never called on the trace corpus)
pub fn _a_name_the_store_holds(memory: &Memory, stem: &str) -> bool {
    crate::unported!("bot.memory.ack:_a_name_the_store_holds")
}

/// Python: bot/memory/ack.py:6102 `_something_they_avoid(memory, stem)` (never called on the trace corpus)
pub fn _something_they_avoid(memory: &Memory, stem: &str) -> bool {
    crate::unported!("bot.memory.ack:_something_they_avoid")
}

/// Python: bot/memory/ack.py:6120 `_plural(word)` (never called on the trace corpus)
pub fn _plural(word: &str) -> Value {
    crate::unported!("bot.memory.ack:_plural")
}

/// Python: bot/memory/ack.py:6147 `_is_a_contraction(word)` (never called on the trace corpus)
pub fn _is_a_contraction(word: &str) -> bool {
    crate::unported!("bot.memory.ack:_is_a_contraction")
}

/// Python: bot/memory/ack.py:6178 `_a_thing_the_line_named(text)` (never called on the trace corpus)
pub fn _a_thing_the_line_named(text: &str) -> Value {
    crate::unported!("bot.memory.ack:_a_thing_the_line_named")
}

/// Python: bot/memory/ack.py:6197 `_a_loss_just_before(memory)` (never called on the trace corpus)
pub fn _a_loss_just_before(memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:_a_loss_just_before")
}

/// Python: bot/memory/ack.py:6208 `_another_of_what_we_said(clean, memory)`
pub fn _another_of_what_we_said(clean: &str, memory: &Memory) {
    crate::unported!("bot.memory.ack:_another_of_what_we_said")
}

/// Python: bot/memory/ack.py:6258 `_only_the_correction_in_front(kept, body)` (never called on the trace corpus)
pub fn _only_the_correction_in_front(kept: &Value, body: &str) -> bool {
    crate::unported!("bot.memory.ack:_only_the_correction_in_front")
}

/// Python: bot/memory/ack.py:6281 `_echo(clean, memory=None)`
pub fn _echo(clean: &str, memory: Option<&Memory>) -> String {
    crate::unported!("bot.memory.ack:_echo")
}

/// Python: bot/memory/ack.py:6335 `said_back(clean, memory=None)` (never called on the trace corpus)
pub fn said_back(clean: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.ack:said_back")
}

/// Python: bot/memory/ack.py:6343 `_only_a_dislike(body, found)`
pub fn _only_a_dislike(body: &str, found: &Value) -> bool {
    crate::unported!("bot.memory.ack:_only_a_dislike")
}

/// Python: bot/memory/ack.py:6356 `only_a_dislike(text, memory=None)`
pub fn only_a_dislike(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.ack:only_a_dislike")
}

/// Python: bot/memory/ack.py:6369 `_tells_it(clean, found=None)`
pub fn _tells_it(clean: &str, found: &Value) -> bool {
    crate::unported!("bot.memory.ack:_tells_it")
}

/// Python: bot/memory/ack.py:6390 `_a_diet(found)` (never called on the trace corpus)
pub fn _a_diet(found: &Value) -> bool {
    crate::unported!("bot.memory.ack:_a_diet")
}

/// Python: bot/memory/ack.py:6413 `_only_the_figure(clean)` (never called on the trace corpus)
pub fn _only_the_figure(clean: &str) -> bool {
    crate::unported!("bot.memory.ack:_only_the_figure")
}

/// Python: bot/memory/ack.py:6446 `_the_rest_of_a_question(clean, before='')` (never called on the trace corpus)
pub fn _the_rest_of_a_question(clean: &str, before: &str) -> bool {
    crate::unported!("bot.memory.ack:_the_rest_of_a_question")
}

/// Python: bot/memory/ack.py:6548 `_after_a_reaction(text)`
pub fn _after_a_reaction(text: &str) -> String {
    crate::unported!("bot.memory.ack:_after_a_reaction")
}

/// Python: bot/memory/ack.py:6563 `_strip_lead_in(clean, keep_the_repair=False)`
pub fn _strip_lead_in(clean: &str, keep_the_repair: bool) -> String {
    crate::unported!("bot.memory.ack:_strip_lead_in")
}

/// Python: bot/memory/ack.py:6600 `without_the_frame(clean)` (never called on the trace corpus)
pub fn without_the_frame(clean: &str) -> Value {
    crate::unported!("bot.memory.ack:without_the_frame")
}

/// Python: bot/memory/ack.py:6632 `_spoken(said)`
pub fn _spoken(said: &str) -> String {
    crate::unported!("bot.memory.ack:_spoken")
}

/// Python: bot/memory/ack.py:6711 `_pick_off(said)`
pub fn _pick_off(said: &str) -> String {
    crate::unported!("bot.memory.ack:_pick_off")
}

/// Python: bot/memory/ack.py:6727 `_tail_off(said)`
pub fn _tail_off(said: &str) -> String {
    crate::unported!("bot.memory.ack:_tail_off")
}

/// Python: bot/memory/ack.py:6744 `_contrast_off(said)`
pub fn _contrast_off(said: &str) -> String {
    crate::unported!("bot.memory.ack:_contrast_off")
}

/// Python: bot/memory/ack.py:6784 `_reason_off(said, clean)`
pub fn _reason_off(said: &str, clean: &str) -> String {
    crate::unported!("bot.memory.ack:_reason_off")
}

/// Python: bot/memory/ack.py:6810 `_the_value_put_right(said, match)` (never called on the trace corpus)
pub fn _the_value_put_right(said: &str, r#match: &Value) -> String {
    crate::unported!("bot.memory.ack:_the_value_put_right")
}

/// Python: bot/memory/ack.py:6873 `_a_value(word)` (never called on the trace corpus)
pub fn _a_value(word: &str) -> bool {
    crate::unported!("bot.memory.ack:_a_value")
}

/// Python: bot/memory/ack.py:6883 `_cut(said)`
pub fn _cut(said: &str) -> String {
    crate::unported!("bot.memory.ack:_cut")
}

/// Python: bot/memory/ack.py:6905 `_tidy(text)`
pub fn _tidy(text: &str) -> String {
    crate::unported!("bot.memory.ack:_tidy")
}

