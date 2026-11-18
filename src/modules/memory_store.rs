//! Port of `bot/modules/memory_store.py`.
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

/// Python: bot/modules/memory_store.py:396 `_a_kept_fact_read(text, memory)`
pub fn _a_kept_fact_read(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_a_kept_fact_read")
}

/// Python: bot/modules/memory_store.py:422 `_their_own_line_ask(text, memory)`
pub fn _their_own_line_ask(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_their_own_line_ask")
}

/// Python: bot/modules/memory_store.py:468 `_with_repaired(found, text, memory, kind)`
pub fn _with_repaired(found: &Value, text: &str, memory: &Memory, kind: &str) -> Value {
    crate::unported!("bot.modules.memory_store:_with_repaired")
}

/// Python: bot/modules/memory_store.py:523 `_the_places_are_written_as_names(reply, memory)`
pub fn _the_places_are_written_as_names(reply: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.modules.memory_store:_the_places_are_written_as_names")
}

/// Python: bot/modules/memory_store.py:558 `note_the_hours(text, memory)`
pub fn note_the_hours(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:note_the_hours")
}

/// Python: bot/modules/memory_store.py:587 `_a_detail_does_not_replace_the_job(found, memory)`
pub fn _a_detail_does_not_replace_the_job(found: &Value, memory: &mut Memory) -> Value {
    crate::unported!("bot.modules.memory_store:_a_detail_does_not_replace_the_job")
}

/// Python: bot/modules/memory_store.py:641 `_without_the_shrug_at_the_end(text)`
pub fn _without_the_shrug_at_the_end(text: &str) -> String {
    crate::unported!("bot.modules.memory_store:_without_the_shrug_at_the_end")
}

/// Python: bot/modules/memory_store.py:661 `observe(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn observe(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:observe")
}

/// Python: bot/modules/memory_store.py:690 `after(text, reply, memory)` - a runtime hook; its signature is the runtime's.
pub fn after(text: &str, reply: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:after")
}

/// Python: bot/modules/memory_store.py:711 `_the_ages_of_the_children(text, memory)`
pub fn _the_ages_of_the_children(text: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.modules.memory_store:_the_ages_of_the_children")
}

/// Python: bot/modules/memory_store.py:745 `_the_count_was_the_line_before(memory)` (never called on the trace corpus)
pub fn _the_count_was_the_line_before(memory: &Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_the_count_was_the_line_before")
}

/// Python: bot/modules/memory_store.py:751 `_the_pets_on_record(memory)`
pub fn _the_pets_on_record(memory: &mut Memory) -> Vec<String> {
    crate::unported!("bot.modules.memory_store:_the_pets_on_record")
}

/// Python: bot/modules/memory_store.py:764 `_note_a_pet_that_is_new(text, memory, before)`
pub fn _note_a_pet_that_is_new(text: &str, memory: &mut Memory, before: &[String]) {
    crate::unported!("bot.modules.memory_store:_note_a_pet_that_is_new")
}

/// Python: bot/modules/memory_store.py:805 `_observe(text, memory)`
pub fn _observe(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:_observe")
}

/// Python: bot/modules/memory_store.py:1048 `_a_trip_booked_named(text, memory)`
pub fn _a_trip_booked_named(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:_a_trip_booked_named")
}

/// Python: bot/modules/memory_store.py:1075 `_this_line_put_something_right(memory, text)`
pub fn _this_line_put_something_right(memory: &Memory, text: &str) -> bool {
    crate::unported!("bot.modules.memory_store:_this_line_put_something_right")
}

/// Python: bot/modules/memory_store.py:1091 `_on_record_before(memory, records)`
pub fn _on_record_before(memory: &Memory, records: &[String]) -> Vec<String> {
    crate::unported!("bot.modules.memory_store:_on_record_before")
}

/// Python: bot/modules/memory_store.py:1112 `_forget_denied(text, memory)`
pub fn _forget_denied(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:_forget_denied")
}

/// Python: bot/modules/memory_store.py:1170 `_the_back_on_the_plate(text, memory)`
pub fn _the_back_on_the_plate(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_the_back_on_the_plate")
}

/// Python: bot/modules/memory_store.py:1217 `_the_one_they_are_back_on(record, kind)` (never called on the trace corpus)
pub fn _the_one_they_are_back_on(record: &Value, kind: &str) -> Value {
    crate::unported!("bot.modules.memory_store:_the_one_they_are_back_on")
}

/// Python: bot/modules/memory_store.py:1233 `_off_their_list(memory, entry, text)` (never called on the trace corpus)
pub fn _off_their_list(memory: &mut Memory, entry: &'static Entry, text: &str) -> Value {
    crate::unported!("bot.modules.memory_store:_off_their_list")
}

/// Python: bot/modules/memory_store.py:1264 `_the_diet_they_are_back_on(memory, value)` (never called on the trace corpus)
pub fn _the_diet_they_are_back_on(memory: &Memory, value: &Value) -> Value {
    crate::unported!("bot.modules.memory_store:_the_diet_they_are_back_on")
}

/// Python: bot/modules/memory_store.py:1285 `_the_last_line(memory)`
pub fn _the_last_line(memory: &Memory) -> String {
    crate::unported!("bot.modules.memory_store:_the_last_line")
}

/// Python: bot/modules/memory_store.py:1353 `_losses_taking(text, held, memory=None)`
pub fn _losses_taking(text: &str, held: &Value, memory: &Memory) -> Vec<String> {
    crate::unported!("bot.modules.memory_store:_losses_taking")
}

/// Python: bot/modules/memory_store.py:1381 `_last_stated(memory)` (never called on the trace corpus)
pub fn _last_stated(memory: &mut Memory) -> Value {
    crate::unported!("bot.modules.memory_store:_last_stated")
}

/// Python: bot/modules/memory_store.py:1396 `_retract_last(memory, text='')` (never called on the trace corpus)
pub fn _retract_last(memory: &mut Memory, text: &str) -> Value {
    crate::unported!("bot.modules.memory_store:_retract_last")
}

/// Python: bot/modules/memory_store.py:1456 `_reads_the_store(text, memory=None)`
pub fn _reads_the_store(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_reads_the_store")
}

/// Python: bot/modules/memory_store.py:1548 `_the_plan_they_took_back(text, memory)`
pub fn _the_plan_they_took_back(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_the_plan_they_took_back")
}

/// Python: bot/modules/memory_store.py:1568 `_the_clock_in_their_line(text, memory)`
pub fn _the_clock_in_their_line(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_the_clock_in_their_line")
}

/// Python: bot/modules/memory_store.py:1600 `_an_ask_about_someone_else(text, memory)`
pub fn _an_ask_about_someone_else(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_an_ask_about_someone_else")
}

/// Python: bot/modules/memory_store.py:1632 `_the_take_back_about_someone(text, memory)`
pub fn _the_take_back_about_someone(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_the_take_back_about_someone")
}

/// Python: bot/modules/memory_store.py:1646 `_the_part_that_hurts_asked(text, memory)`
pub fn _the_part_that_hurts_asked(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_the_part_that_hurts_asked")
}

/// Python: bot/modules/memory_store.py:1661 `_the_store_readings(text, memory=None)`
pub fn _the_store_readings(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_the_store_readings")
}

/// Python: bot/modules/memory_store.py:1865 `_their_fact_ask(text, memory=None)`
pub fn _their_fact_ask(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_their_fact_ask")
}

/// Python: bot/modules/memory_store.py:1912 `_name_the_last_pet(text, memory)`
pub fn _name_the_last_pet(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:_name_the_last_pet")
}

/// Python: bot/modules/memory_store.py:1977 `_the_animals_named(text, memory)` (never called on the trace corpus)
pub fn _the_animals_named(text: &str, memory: &mut Memory) -> Value {
    crate::unported!("bot.modules.memory_store:_the_animals_named")
}

/// Python: bot/modules/memory_store.py:1990 `_the_name_it_had(animal, said, memory)` (never called on the trace corpus)
pub fn _the_name_it_had(animal: &Value, said: &str, memory: &mut Memory) -> Value {
    crate::unported!("bot.modules.memory_store:_the_name_it_had")
}

/// Python: bot/modules/memory_store.py:2004 `_names_a_pet_here(text, memory)` (never called on the trace corpus)
pub fn _names_a_pet_here(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_names_a_pet_here")
}

/// Python: bot/modules/memory_store.py:2035 `_the_figure_said(match)` (never called on the trace corpus)
pub fn _the_figure_said(r#match: &Value) -> Value {
    crate::unported!("bot.modules.memory_store:_the_figure_said")
}

/// Python: bot/modules/memory_store.py:2057 `_without_a_lead_in(text)`
pub fn _without_a_lead_in(text: &str) -> String {
    crate::unported!("bot.modules.memory_store:_without_a_lead_in")
}

/// Python: bot/modules/memory_store.py:2068 `_put_a_pets_age(pet, age, unit, text)` (never called on the trace corpus)
pub fn _put_a_pets_age(pet: &Value, age: &Value, unit: &str, text: &str) -> Value {
    crate::unported!("bot.modules.memory_store:_put_a_pets_age")
}

/// Python: bot/modules/memory_store.py:2097 `_age_the_named_pet(text, memory)`
pub fn _age_the_named_pet(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:_age_the_named_pet")
}

/// Python: bot/modules/memory_store.py:2124 `_fix_a_pets_age(text, memory)`
pub fn _fix_a_pets_age(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:_fix_a_pets_age")
}

/// Python: bot/modules/memory_store.py:2173 `_age_said_of_the_only_animal(text, memory)`
pub fn _age_said_of_the_only_animal(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:_age_said_of_the_only_animal")
}

/// Python: bot/modules/memory_store.py:2228 `_age_they_answered(text, memory)`
pub fn _age_they_answered(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:_age_they_answered")
}

/// Python: bot/modules/memory_store.py:2273 `_their_name_answered(text, memory)`
pub fn _their_name_answered(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:_their_name_answered")
}

/// Python: bot/modules/memory_store.py:2440 `_fix_a_persons_age(text, memory)`
pub fn _fix_a_persons_age(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:_fix_a_persons_age")
}

/// Python: bot/modules/memory_store.py:2478 `_fix_the_weeks_of_a_pregnancy(text, memory)`
pub fn _fix_the_weeks_of_a_pregnancy(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:_fix_the_weeks_of_a_pregnancy")
}

/// Python: bot/modules/memory_store.py:2550 `_fix_a_pets_name(text, memory)`
pub fn _fix_a_pets_name(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:_fix_a_pets_name")
}

/// Python: bot/modules/memory_store.py:2692 `_pets_of(animal, pets)` (never called on the trace corpus)
pub fn _pets_of(animal: &Value, pets: &Value) -> Value {
    crate::unported!("bot.modules.memory_store:_pets_of")
}

/// Python: bot/modules/memory_store.py:2698 `_a_pet_new_this_line(pet, memory)` (never called on the trace corpus)
pub fn _a_pet_new_this_line(pet: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_a_pet_new_this_line")
}

/// Python: bot/modules/memory_store.py:2712 `_another_animal_is_named(text)` (never called on the trace corpus)
pub fn _another_animal_is_named(text: &str) -> bool {
    crate::unported!("bot.modules.memory_store:_another_animal_is_named")
}

/// Python: bot/modules/memory_store.py:2717 `_two_of_the_animal_are_kept(pets, animal)` (never called on the trace corpus)
pub fn _two_of_the_animal_are_kept(pets: &Value, animal: &Value) -> bool {
    crate::unported!("bot.modules.memory_store:_two_of_the_animal_are_kept")
}

/// Python: bot/modules/memory_store.py:2729 `_the_turn_before_said(word, memory)` (never called on the trace corpus)
pub fn _the_turn_before_said(word: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_the_turn_before_said")
}

/// Python: bot/modules/memory_store.py:2750 `_fix_the_make_of_a_thing(text, memory)`
pub fn _fix_the_make_of_a_thing(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:_fix_the_make_of_a_thing")
}

/// Python: bot/modules/memory_store.py:2787 `_fix_a_persons_name(text, memory)`
pub fn _fix_a_persons_name(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:_fix_a_persons_name")
}

/// Python: bot/modules/memory_store.py:2870 `_the_name_they_gave_to_the_person_they_mean(text, memory)`
pub fn _the_name_they_gave_to_the_person_they_mean(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:_the_name_they_gave_to_the_person_they_mean")
}

/// Python: bot/modules/memory_store.py:3111 `_a_person(value)` (never called on the trace corpus)
pub fn _a_person(value: &Value) -> bool {
    crate::unported!("bot.modules.memory_store:_a_person")
}

/// Python: bot/modules/memory_store.py:3153 `_a_correction_without_a_marker(text, memory)`
pub fn _a_correction_without_a_marker(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:_a_correction_without_a_marker")
}

/// Python: bot/modules/memory_store.py:3181 `_a_person_called(memory, name)` (never called on the trace corpus)
pub fn _a_person_called(memory: &mut Memory, name: &str) -> bool {
    crate::unported!("bot.modules.memory_store:_a_person_called")
}

/// Python: bot/modules/memory_store.py:3221 `_stated_value(memory, key)` (never called on the trace corpus)
pub fn _stated_value(memory: &Memory, key: &str) -> Value {
    crate::unported!("bot.modules.memory_store:_stated_value")
}

/// Python: bot/modules/memory_store.py:3262 `_a_second_persons_limit(text, memory)`
pub fn _a_second_persons_limit(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.memory_store:_a_second_persons_limit")
}

/// Python: bot/modules/memory_store.py:3476 `_drop_value(memory, value, text='')` (never called on the trace corpus)
pub fn _drop_value(memory: &mut Memory, value: &Value, text: &str) -> Value {
    crate::unported!("bot.modules.memory_store:_drop_value")
}

/// Python: bot/modules/memory_store.py:3501 `_plan_names(plan, low)` (never called on the trace corpus)
pub fn _plan_names(plan: &str, low: &str) -> bool {
    crate::unported!("bot.modules.memory_store:_plan_names")
}

/// Python: bot/modules/memory_store.py:3512 `_drop_from_plan(memory, value, text='')` (never called on the trace corpus)
pub fn _drop_from_plan(memory: &mut Memory, value: &Value, text: &str) -> Value {
    crate::unported!("bot.modules.memory_store:_drop_from_plan")
}

/// Python: bot/modules/memory_store.py:3563 `_names_this(held, low)` (never called on the trace corpus)
pub fn _names_this(held: &Value, low: &str) -> bool {
    crate::unported!("bot.modules.memory_store:_names_this")
}

/// Python: bot/modules/memory_store.py:3679 `_budget_sign(text, budget)` (never called on the trace corpus)
pub fn _budget_sign(text: &str, budget: &Value) -> Value {
    crate::unported!("bot.modules.memory_store:_budget_sign")
}

/// Python: bot/modules/memory_store.py:3786 `_a_feeling_about_their_own_news(text, memory)`
pub fn _a_feeling_about_their_own_news(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_a_feeling_about_their_own_news")
}

/// Python: bot/modules/memory_store.py:3810 `takes_as_it_comes(text, memory=None)` (never called on the trace corpus)
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.memory_store:takes_as_it_comes")
}

/// Python: bot/modules/memory_store.py:3828 `_the_ask_about_how_it_went(text, memory)`
pub fn _the_ask_about_how_it_went(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_the_ask_about_how_it_went")
}

/// Python: bot/modules/memory_store.py:3847 `_a_count_from_the_date(text, memory)`
pub fn _a_count_from_the_date(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_a_count_from_the_date")
}

/// Python: bot/modules/memory_store.py:3888 `_the_unit_asked(line)`
pub fn _the_unit_asked(line: &str) {
    crate::unported!("bot.modules.memory_store:_the_unit_asked")
}

/// Python: bot/modules/memory_store.py:3956 `_the_clause(said, start, end)` (never called on the trace corpus)
pub fn _the_clause(said: &str, start: i64, end: i64) -> Value {
    crate::unported!("bot.modules.memory_store:_the_clause")
}

/// Python: bot/modules/memory_store.py:3981 `_the_money_figures(said)` (never called on the trace corpus)
pub fn _the_money_figures(said: &str) -> Value {
    crate::unported!("bot.modules.memory_store:_the_money_figures")
}

/// Python: bot/modules/memory_store.py:4001 `_the_figures_in(said, units)`
pub fn _the_figures_in(said: &str, units: &[String]) -> Vec<String> {
    crate::unported!("bot.modules.memory_store:_the_figures_in")
}

/// Python: bot/modules/memory_store.py:4043 `_the_figure_the_answer_names(text, memory)`
pub fn _the_figure_the_answer_names(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.memory_store:_the_figure_the_answer_names")
}

/// Python: bot/modules/memory_store.py:4123 `_a_figure_of_those_units(said, units)`
pub fn _a_figure_of_those_units(said: &str, units: &[String]) -> bool {
    crate::unported!("bot.modules.memory_store:_a_figure_of_those_units")
}

/// Python: bot/modules/memory_store.py:4162 `_the_time_they_put_right(memory)`
pub fn _the_time_they_put_right(memory: &Memory) {
    crate::unported!("bot.modules.memory_store:_the_time_they_put_right")
}

/// Python: bot/modules/memory_store.py:4191 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.memory_store:respond")
}

/// Python: bot/modules/memory_store.py:4361 `_asks_to_hear_it_again(text, memory)`
pub fn _asks_to_hear_it_again(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_asks_to_hear_it_again")
}

/// Python: bot/modules/memory_store.py:4503 `_asks_about_someone_else(text)`
pub fn _asks_about_someone_else(text: &str) -> bool {
    crate::unported!("bot.modules.memory_store:_asks_about_someone_else")
}

/// Python: bot/modules/memory_store.py:4607 `_note_a_corrected_fact(memory, text, entry, was)`
pub fn _note_a_corrected_fact(memory: &mut Memory, text: &str, entry: &Value, was: &str) {
    crate::unported!("bot.modules.memory_store:_note_a_corrected_fact")
}

/// Python: bot/modules/memory_store.py:4634 `_note_a_fact_moved(memory, text, entry, was)`
pub fn _note_a_fact_moved(memory: &mut Memory, text: &str, entry: &Value, was: &str) {
    crate::unported!("bot.modules.memory_store:_note_a_fact_moved")
}

/// Python: bot/modules/memory_store.py:4736 `_a_word_for_the_slot(memory, attribute)` (never called on the trace corpus)
pub fn _a_word_for_the_slot(memory: &Memory, attribute: &Value) -> bool {
    crate::unported!("bot.modules.memory_store:_a_word_for_the_slot")
}

/// Python: bot/modules/memory_store.py:4744 `_the_ask_behind_the_recall(low)`
pub fn _the_ask_behind_the_recall(low: &str) {
    crate::unported!("bot.modules.memory_store:_the_ask_behind_the_recall")
}

/// Python: bot/modules/memory_store.py:4816 `TAKES_AS_IT_COMES(text, memory=None)`
pub fn TAKES_AS_IT_COMES(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.memory_store:TAKES_AS_IT_COMES")
}

/// Python: bot/modules/memory_store.py:5019 `_about_the_verb(plans, low, found)` (never called on the trace corpus)
pub fn _about_the_verb(plans: &Value, low: &str, found: &Value) -> Value {
    crate::unported!("bot.modules.memory_store:_about_the_verb")
}

/// Python: bot/modules/memory_store.py:5120 `_remember_dislikes(text, memory, named_here=())`
pub fn _remember_dislikes(text: &str, memory: &mut Memory, named_here: &[String]) {
    crate::unported!("bot.modules.memory_store:_remember_dislikes")
}

/// Python: bot/modules/memory_store.py:5194 `_diet_reply(memory, low='')` (never called on the trace corpus)
pub fn _diet_reply(memory: &mut Memory, low: &str) -> Value {
    crate::unported!("bot.modules.memory_store:_diet_reply")
}

/// Python: bot/modules/memory_store.py:5722 `_a_fact_word(low)` (never called on the trace corpus)
pub fn _a_fact_word(low: &str) -> bool {
    crate::unported!("bot.modules.memory_store:_a_fact_word")
}

/// Python: bot/modules/memory_store.py:5729 `_a_callback_that_names_its_thing(text, memory)`
pub fn _a_callback_that_names_its_thing(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_a_callback_that_names_its_thing")
}

/// Python: bot/modules/memory_store.py:5908 `_points_back_at_nothing(text, memory)`
pub fn _points_back_at_nothing(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.memory_store:_points_back_at_nothing")
}

/// Python: bot/modules/memory_store.py:5922 `_a_recall_about_nothing(text, memory)`
pub fn _a_recall_about_nothing(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.memory_store:_a_recall_about_nothing")
}

/// Python: bot/modules/memory_store.py:5958 `_names_nothing_in_a_recall(rest)` (never called on the trace corpus)
pub fn _names_nothing_in_a_recall(rest: &str) -> bool {
    crate::unported!("bot.modules.memory_store:_names_nothing_in_a_recall")
}

/// Python: bot/modules/memory_store.py:5964 `_a_recall_frame_about_nothing(text, memory)`
pub fn _a_recall_frame_about_nothing(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.memory_store:_a_recall_frame_about_nothing")
}

/// Python: bot/modules/memory_store.py:6000 `_an_ask_about_nothing(text, memory)`
pub fn _an_ask_about_nothing(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.memory_store:_an_ask_about_nothing")
}

/// Python: bot/modules/memory_store.py:6021 `_the_last_ask_about_the_bot(memory)` (never called on the trace corpus)
pub fn _the_last_ask_about_the_bot(memory: &Memory) -> Value {
    crate::unported!("bot.modules.memory_store:_the_last_ask_about_the_bot")
}

/// Python: bot/modules/memory_store.py:6227 `_fits(answer, room)` (never called on the trace corpus)
pub fn _fits(answer: &str, room: &Value) -> Value {
    crate::unported!("bot.modules.memory_store:_fits")
}

/// Python: bot/modules/memory_store.py:6348 `_asks_whether_we_followed(text, memory)`
pub fn _asks_whether_we_followed(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_asks_whether_we_followed")
}

/// Python: bot/modules/memory_store.py:6391 `_the_group_size_ask(text, memory)`
pub fn _the_group_size_ask(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.memory_store:_the_group_size_ask")
}

