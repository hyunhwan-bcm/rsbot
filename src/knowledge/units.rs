//! Port of `bot/knowledge/units.py`.
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

/// Python: bot/knowledge/units.py:257 `_the_component_is_the_whole_measure(clean)` (never called on the trace corpus)
pub fn _the_component_is_the_whole_measure(clean: &str) -> bool {
    crate::unported!("bot.knowledge.units:_the_component_is_the_whole_measure")
}

/// Python: bot/knowledge/units.py:282 `_the_thing_measured_by_itself(clean)`
pub fn _the_thing_measured_by_itself(clean: &str) {
    crate::unported!("bot.knowledge.units:_the_thing_measured_by_itself")
}

/// Python: bot/knowledge/units.py:341 `_object_weight(clean)`
pub fn _object_weight(clean: &str) {
    crate::unported!("bot.knowledge.units:_object_weight")
}

/// Python: bot/knowledge/units.py:369 `_many(name)` (never called on the trace corpus)
pub fn _many(name: &str) -> String {
    crate::unported!("bot.knowledge.units:_many")
}

/// Python: bot/knowledge/units.py:378 `_kilos(grams)` (never called on the trace corpus)
pub fn _kilos(grams: &Value) -> String {
    crate::unported!("bot.knowledge.units:_kilos")
}

/// Python: bot/knowledge/units.py:404 `_build()` (never called on the trace corpus)
pub fn _build() -> Value {
    crate::unported!("bot.knowledge.units:_build")
}

/// Python: bot/knowledge/units.py:452 `_factor(family, unit)`
pub fn _factor(family: &str, unit: &str) -> f64 {
    crate::unported!("bot.knowledge.units:_factor")
}

/// Python: bot/knowledge/units.py:459 `_words(unit)` (never called on the trace corpus)
pub fn _words(unit: &str) -> Value {
    crate::unported!("bot.knowledge.units:_words")
}

/// Python: bot/knowledge/units.py:467 `_grams(value)` (never called on the trace corpus)
pub fn _grams(value: &Value) -> String {
    crate::unported!("bot.knowledge.units:_grams")
}

/// Python: bot/knowledge/units.py:473 `_fmt(value)`
pub fn _fmt(value: f64) -> String {
    crate::unported!("bot.knowledge.units:_fmt")
}

/// Python: bot/knowledge/units.py:489 `_value(raw)`
pub fn _value(raw: &str) -> f64 {
    crate::unported!("bot.knowledge.units:_value")
}

/// Python: bot/knowledge/units.py:498 `_mentions(clean)`
pub fn _mentions(clean: &str) -> Value {
    crate::unported!("bot.knowledge.units:_mentions")
}

/// Python: bot/knowledge/units.py:511 `_number_before(clean, start)`
pub fn _number_before(clean: &str, start: i64) {
    crate::unported!("bot.knowledge.units:_number_before")
}

/// Python: bot/knowledge/units.py:521 `_say(value, unit, one=None)`
pub fn _say(value: &Value, unit: &str, one: Option<bool>) -> String {
    crate::unported!("bot.knowledge.units:_say")
}

/// Python: bot/knowledge/units.py:535 `_split_time(result, to_unit)`
pub fn _split_time(result: f64, to_unit: &str) {
    crate::unported!("bot.knowledge.units:_split_time")
}

/// Python: bot/knowledge/units.py:549 `_sentence(value, family, from_unit, to_unit, in_phrase=False)`
pub fn _sentence(value: f64, family: &str, from_unit: &str, to_unit: &str, in_phrase: bool) -> String {
    crate::unported!("bot.knowledge.units:_sentence")
}

/// Python: bot/knowledge/units.py:569 `_temperature(clean)`
pub fn _temperature(clean: &str) {
    crate::unported!("bot.knowledge.units:_temperature")
}

/// Python: bot/knowledge/units.py:607 `_temperature_rule(clean)`
pub fn _temperature_rule(clean: &str) {
    crate::unported!("bot.knowledge.units:_temperature_rule")
}

/// Python: bot/knowledge/units.py:625 `_mark_for(celsius)` (never called on the trace corpus)
pub fn _mark_for(celsius: &Value) -> Value {
    crate::unported!("bot.knowledge.units:_mark_for")
}

/// Python: bot/knowledge/units.py:630 `_oven(clean)`
pub fn _oven(clean: &str) {
    crate::unported!("bot.knowledge.units:_oven")
}

/// Python: bot/knowledge/units.py:693 `_butter(clean)`
pub fn _butter(clean: &str) {
    crate::unported!("bot.knowledge.units:_butter")
}

/// Python: bot/knowledge/units.py:775 `_pack(clean)`
pub fn _pack(clean: &str) {
    crate::unported!("bot.knowledge.units:_pack")
}

/// Python: bot/knowledge/units.py:833 `_singular(name)` (never called on the trace corpus)
pub fn _singular(name: &str) -> Value {
    crate::unported!("bot.knowledge.units:_singular")
}

/// Python: bot/knowledge/units.py:844 `_table_grams(name, table)` (never called on the trace corpus)
pub fn _table_grams(name: &str, table: &Value) -> Value {
    crate::unported!("bot.knowledge.units:_table_grams")
}

/// Python: bot/knowledge/units.py:854 `_pack_count(clean)`
pub fn _pack_count(clean: &str) {
    crate::unported!("bot.knowledge.units:_pack_count")
}

/// Python: bot/knowledge/units.py:944 `_in_a_vessel(clean)`
pub fn _in_a_vessel(clean: &str) {
    crate::unported!("bot.knowledge.units:_in_a_vessel")
}

/// Python: bot/knowledge/units.py:962 `_loose(clean)`
pub fn _loose(clean: &str) {
    crate::unported!("bot.knowledge.units:_loose")
}

/// Python: bot/knowledge/units.py:991 `_liquid_weight(clean)`
pub fn _liquid_weight(clean: &str) {
    crate::unported!("bot.knowledge.units:_liquid_weight")
}

/// Python: bot/knowledge/units.py:1025 `_mass_said(grams)` (never called on the trace corpus)
pub fn _mass_said(grams: &Value) -> String {
    crate::unported!("bot.knowledge.units:_mass_said")
}

/// Python: bot/knowledge/units.py:1179 `_a_dish_for_a_table(clean)`
pub fn _a_dish_for_a_table(clean: &str) {
    crate::unported!("bot.knowledge.units:_a_dish_for_a_table")
}

/// Python: bot/knowledge/units.py:1232 `_an_allowance(names)` (never called on the trace corpus)
pub fn _an_allowance(names: &[String]) -> Value {
    crate::unported!("bot.knowledge.units:_an_allowance")
}

/// Python: bot/knowledge/units.py:1242 `_a_weight(grams)` (never called on the trace corpus)
pub fn _a_weight(grams: &Value) -> String {
    crate::unported!("bot.knowledge.units:_a_weight")
}

/// Python: bot/knowledge/units.py:1367 `people_in_groups(text)`
pub fn people_in_groups(text: &str) {
    crate::unported!("bot.knowledge.units:people_in_groups")
}

/// Python: bot/knowledge/units.py:1383 `_people_asked(clean)`
pub fn _people_asked(clean: &str) {
    crate::unported!("bot.knowledge.units:_people_asked")
}

/// Python: bot/knowledge/units.py:1400 `_people_before(memory, statements_only=False, asked_for_an_amount=False)`
pub fn _people_before(memory: &Value, statements_only: bool, asked_for_an_amount: bool) -> Option<i64> {
    crate::unported!("bot.knowledge.units:_people_before")
}

/// Python: bot/knowledge/units.py:1467 `_a_count_that_still_stands(older)`
pub fn _a_count_that_still_stands(older: &[String]) {
    crate::unported!("bot.knowledge.units:_a_count_that_still_stands")
}

/// Python: bot/knowledge/units.py:1500 `_a_corrected_figure(text)`
pub fn _a_corrected_figure(text: &str) {
    crate::unported!("bot.knowledge.units:_a_corrected_figure")
}

/// Python: bot/knowledge/units.py:1518 `_a_table_without_people(text)`
pub fn _a_table_without_people(text: &str) -> bool {
    crate::unported!("bot.knowledge.units:_a_table_without_people")
}

/// Python: bot/knowledge/units.py:1535 `_per_person(clean, memory=None)`
pub fn _per_person(clean: &str, memory: &Memory) {
    crate::unported!("bot.knowledge.units:_per_person")
}

/// Python: bot/knowledge/units.py:1673 `_said_people(people)` (never called on the trace corpus)
pub fn _said_people(people: &Value) -> Value {
    crate::unported!("bot.knowledge.units:_said_people")
}

/// Python: bot/knowledge/units.py:1678 `_mass_of(grams)` (never called on the trace corpus)
pub fn _mass_of(grams: &Value) -> String {
    crate::unported!("bot.knowledge.units:_mass_of")
}

/// Python: bot/knowledge/units.py:1686 `_litres_of(ml)` (never called on the trace corpus)
pub fn _litres_of(ml: &Value) -> String {
    crate::unported!("bot.knowledge.units:_litres_of")
}

/// Python: bot/knowledge/units.py:1738 `_count_said(ratio)` (never called on the trace corpus)
pub fn _count_said(ratio: &Value) -> String {
    crate::unported!("bot.knowledge.units:_count_said")
}

/// Python: bot/knowledge/units.py:1753 `_area_pictures(square_metres)` (never called on the trace corpus)
pub fn _area_pictures(square_metres: &Value) -> String {
    crate::unported!("bot.knowledge.units:_area_pictures")
}

/// Python: bot/knowledge/units.py:1770 `_share_before(clean, start)` (never called on the trace corpus)
pub fn _share_before(clean: &str, start: i64) -> Value {
    crate::unported!("bot.knowledge.units:_share_before")
}

/// Python: bot/knowledge/units.py:1782 `_land_in_pictures(value, unit, clean='')` (never called on the trace corpus)
pub fn _land_in_pictures(value: &Value, unit: &str, clean: &str) -> Option<String> {
    crate::unported!("bot.knowledge.units:_land_in_pictures")
}

/// Python: bot/knowledge/units.py:1805 `_land_size(clean, mentions)`
pub fn _land_size(clean: &str, mentions: &Value) {
    crate::unported!("bot.knowledge.units:_land_size")
}

/// Python: bot/knowledge/units.py:1834 `_height(clean, mentions)`
pub fn _height(clean: &str, mentions: &Value) {
    crate::unported!("bot.knowledge.units:_height")
}

/// Python: bot/knowledge/units.py:1871 `_is_a_distance(clean)`
pub fn _is_a_distance(clean: &str) -> bool {
    crate::unported!("bot.knowledge.units:_is_a_distance")
}

/// Python: bot/knowledge/units.py:1911 `_asks_something_else(clean)`
pub fn _asks_something_else(clean: &str) -> bool {
    crate::unported!("bot.knowledge.units:_asks_something_else")
}

/// Python: bot/knowledge/units.py:1928 `_own_height(clean)`
pub fn _own_height(clean: &str) {
    crate::unported!("bot.knowledge.units:_own_height")
}

/// Python: bot/knowledge/units.py:1969 `_metres_in_feet(clean)`
pub fn _metres_in_feet(clean: &str) {
    crate::unported!("bot.knowledge.units:_metres_in_feet")
}

/// Python: bot/knowledge/units.py:1997 `_cup_of_something(clean)`
pub fn _cup_of_something(clean: &str) {
    crate::unported!("bot.knowledge.units:_cup_of_something")
}

/// Python: bot/knowledge/units.py:2035 `_handful(clean)`
pub fn _handful(clean: &str) {
    crate::unported!("bot.knowledge.units:_handful")
}

/// Python: bot/knowledge/units.py:2093 `_asks_for_weight(clean)`
pub fn _asks_for_weight(clean: &str) -> bool {
    crate::unported!("bot.knowledge.units:_asks_for_weight")
}

/// Python: bot/knowledge/units.py:2108 `_names_unit(clean, unit)`
pub fn _names_unit(clean: &str, unit: &str) -> bool {
    crate::unported!("bot.knowledge.units:_names_unit")
}

/// Python: bot/knowledge/units.py:2118 `_frac(clean)`
pub fn _frac(clean: &str) -> String {
    crate::unported!("bot.knowledge.units:_frac")
}

/// Python: bot/knowledge/units.py:2150 `_rule_sentence(family, from_unit, to_unit)` (never called on the trace corpus)
pub fn _rule_sentence(family: &Value, from_unit: &Value, to_unit: &Value) -> Option<String> {
    crate::unported!("bot.knowledge.units:_rule_sentence")
}

/// Python: bot/knowledge/units.py:2184 `_is_bare_pair(clean, mentions)`
pub fn _is_bare_pair(clean: &str, mentions: &Value) -> bool {
    crate::unported!("bot.knowledge.units:_is_bare_pair")
}

/// Python: bot/knowledge/units.py:2208 `_the_ounce_pair(family, unit, other_family, other_unit)`
pub fn _the_ounce_pair(family: &str, unit: &str, other_family: &str, other_unit: &str) -> Vec<String> {
    crate::unported!("bot.knowledge.units:_the_ounce_pair")
}

/// Python: bot/knowledge/units.py:2221 `_how_to_convert(clean, mentions)`
pub fn _how_to_convert(clean: &str, mentions: &Value) {
    crate::unported!("bot.knowledge.units:_how_to_convert")
}

/// Python: bot/knowledge/units.py:2252 `_tail_is_about_the_measure(tail)`
pub fn _tail_is_about_the_measure(tail: &str) -> bool {
    crate::unported!("bot.knowledge.units:_tail_is_about_the_measure")
}

/// Python: bot/knowledge/units.py:2296 `_share_value(word)` (never called on the trace corpus)
pub fn _share_value(word: &str) -> Value {
    crate::unported!("bot.knowledge.units:_share_value")
}

/// Python: bot/knowledge/units.py:2306 `_share_name(value)` (never called on the trace corpus)
pub fn _share_name(value: &Value) -> Value {
    crate::unported!("bot.knowledge.units:_share_name")
}

/// Python: bot/knowledge/units.py:2316 `_share_spoons(value, per)` (never called on the trace corpus)
pub fn _share_spoons(value: &Value, per: &Value) -> Value {
    crate::unported!("bot.knowledge.units:_share_spoons")
}

/// Python: bot/knowledge/units.py:2331 `_share_of_a_measure(clean, tail=True)`
pub fn _share_of_a_measure(clean: &str, tail: bool) {
    crate::unported!("bot.knowledge.units:_share_of_a_measure")
}

/// Python: bot/knowledge/units.py:2373 `_tsp_phrase(count)` (never called on the trace corpus)
pub fn _tsp_phrase(count: i64) -> String {
    crate::unported!("bot.knowledge.units:_tsp_phrase")
}

/// Python: bot/knowledge/units.py:2453 `_a_fraction_value(word)` (never called on the trace corpus)
pub fn _a_fraction_value(word: &str) -> Value {
    crate::unported!("bot.knowledge.units:_a_fraction_value")
}

/// Python: bot/knowledge/units.py:2477 `_a_fraction_name(value)` (never called on the trace corpus)
pub fn _a_fraction_name(value: &Value) -> Value {
    crate::unported!("bot.knowledge.units:_a_fraction_name")
}

/// Python: bot/knowledge/units.py:2493 `_a_span_amount(amount, name)` (never called on the trace corpus)
pub fn _a_span_amount(amount: &Value, name: &str) -> String {
    crate::unported!("bot.knowledge.units:_a_span_amount")
}

/// Python: bot/knowledge/units.py:2503 `_a_number_word(number)` (never called on the trace corpus)
pub fn _a_number_word(number: &Value) -> Value {
    crate::unported!("bot.knowledge.units:_a_number_word")
}

/// Python: bot/knowledge/units.py:2510 `_a_share_of_a_unit(value, unit)` (never called on the trace corpus)
pub fn _a_share_of_a_unit(value: &Value, unit: &str) -> String {
    crate::unported!("bot.knowledge.units:_a_share_of_a_unit")
}

/// Python: bot/knowledge/units.py:2540 `_a_fraction_of_a_time_unit(clean)`
pub fn _a_fraction_of_a_time_unit(clean: &str) {
    crate::unported!("bot.knowledge.units:_a_fraction_of_a_time_unit")
}

/// Python: bot/knowledge/units.py:2577 `_the_span_said(span_seconds, unit, clean)` (never called on the trace corpus)
pub fn _the_span_said(span_seconds: &Value, unit: &str, clean: &str) -> String {
    crate::unported!("bot.knowledge.units:_the_span_said")
}

/// Python: bot/knowledge/units.py:2588 `_a_mixed_parts(match)` (never called on the trace corpus)
pub fn _a_mixed_parts(r#match: &Value) -> Value {
    crate::unported!("bot.knowledge.units:_a_mixed_parts")
}

/// Python: bot/knowledge/units.py:2600 `_a_span_of_time(clean)`
pub fn _a_span_of_time(clean: &str) {
    crate::unported!("bot.knowledge.units:_a_span_of_time")
}

/// Python: bot/knowledge/units.py:2673 `_share_carried(clean, memory)`
pub fn _share_carried(clean: &str, memory: &Memory) {
    crate::unported!("bot.knowledge.units:_share_carried")
}

/// Python: bot/knowledge/units.py:2697 `convert(clean, memory=None)`
pub fn convert(clean: &str, memory: &Memory) {
    crate::unported!("bot.knowledge.units:convert")
}

/// Python: bot/knowledge/units.py:2728 `_count_in_a_line(clean)`
pub fn _count_in_a_line(clean: &str) {
    crate::unported!("bot.knowledge.units:_count_in_a_line")
}

/// Python: bot/knowledge/units.py:2756 `_portions_in_a_weight(clean)`
pub fn _portions_in_a_weight(clean: &str) {
    crate::unported!("bot.knowledge.units:_portions_in_a_weight")
}

/// Python: bot/knowledge/units.py:2877 `_a_table_word(count)` (never called on the trace corpus)
pub fn _a_table_word(count: i64) -> Value {
    crate::unported!("bot.knowledge.units:_a_table_word")
}

/// Python: bot/knowledge/units.py:2887 `_birds_for_a_table(clean)`
pub fn _birds_for_a_table(clean: &str) {
    crate::unported!("bot.knowledge.units:_birds_for_a_table")
}

/// Python: bot/knowledge/units.py:2939 `_people_in_a_count(word)` (never called on the trace corpus)
pub fn _people_in_a_count(word: &str) -> Value {
    crate::unported!("bot.knowledge.units:_people_in_a_count")
}

/// Python: bot/knowledge/units.py:2948 `_a_whole_thing_feeds(clean, memory=None)`
pub fn _a_whole_thing_feeds(clean: &str, memory: &Memory) {
    crate::unported!("bot.knowledge.units:_a_whole_thing_feeds")
}

/// Python: bot/knowledge/units.py:3014 `_answer(clean, memory=None)`
pub fn _answer(clean: &str, memory: &Memory) {
    crate::unported!("bot.knowledge.units:_answer")
}

/// Python: bot/knowledge/units.py:3193 `_unit_words(text)`
pub fn _unit_words(text: &str) -> Value {
    crate::unported!("bot.knowledge.units:_unit_words")
}

/// Python: bot/knowledge/units.py:3199 `_bare_measure(text)`
pub fn _bare_measure(text: &str) {
    crate::unported!("bot.knowledge.units:_bare_measure")
}

/// Python: bot/knowledge/units.py:3224 `_previous_measure(memory)`
pub fn _previous_measure(memory: &Memory) {
    crate::unported!("bot.knowledge.units:_previous_measure")
}

/// Python: bot/knowledge/units.py:3256 `_previous_portion(memory)` (never called on the trace corpus)
pub fn _previous_portion(memory: &Memory) -> Value {
    crate::unported!("bot.knowledge.units:_previous_portion")
}

/// Python: bot/knowledge/units.py:3269 `_carried_people(clean, memory)`
pub fn _carried_people(clean: &str, memory: &Memory) {
    crate::unported!("bot.knowledge.units:_carried_people")
}

/// Python: bot/knowledge/units.py:3314 `_carried_weight(clean, memory)`
pub fn _carried_weight(clean: &str, memory: &Memory) {
    crate::unported!("bot.knowledge.units:_carried_weight")
}

/// Python: bot/knowledge/units.py:3369 `_names_a_portion_food(line)` (never called on the trace corpus)
pub fn _names_a_portion_food(line: &str) -> bool {
    crate::unported!("bot.knowledge.units:_names_a_portion_food")
}

/// Python: bot/knowledge/units.py:3378 `_portion_after_correction(memory)` (never called on the trace corpus)
pub fn _portion_after_correction(memory: &Memory) -> Value {
    crate::unported!("bot.knowledge.units:_portion_after_correction")
}

/// Python: bot/knowledge/units.py:3416 `_carried_measure(clean, memory)`
pub fn _carried_measure(clean: &str, memory: &Memory) {
    crate::unported!("bot.knowledge.units:_carried_measure")
}

/// Python: bot/knowledge/units.py:3451 `_a_or_an(word)` (never called on the trace corpus)
pub fn _a_or_an(word: &str) -> String {
    crate::unported!("bot.knowledge.units:_a_or_an")
}

/// Python: bot/knowledge/units.py:3460 `_amount_phrase(value, unit)` (never called on the trace corpus)
pub fn _amount_phrase(value: &Value, unit: &str) -> Value {
    crate::unported!("bot.knowledge.units:_amount_phrase")
}

/// Python: bot/knowledge/units.py:3472 `_cups_phrase(amount, unit, spoons=False)`
pub fn _cups_phrase(amount: f64, unit: &str, spoons: bool) -> String {
    crate::unported!("bot.knowledge.units:_cups_phrase")
}

/// Python: bot/knowledge/units.py:3497 `_spoons(count)` (never called on the trace corpus)
pub fn _spoons(count: i64) -> String {
    crate::unported!("bot.knowledge.units:_spoons")
}

/// Python: bot/knowledge/units.py:3510 `answers_more_than_a_pair(clean)`
pub fn answers_more_than_a_pair(clean: &str) -> bool {
    crate::unported!("bot.knowledge.units:answers_more_than_a_pair")
}

/// Python: bot/knowledge/units.py:3524 `_food_named(clean)`
pub fn _food_named(clean: &str) {
    crate::unported!("bot.knowledge.units:_food_named")
}

/// Python: bot/knowledge/units.py:3538 `_named_ingredient(clean, mentions, measured)`
pub fn _named_ingredient(clean: &str, mentions: &Value, measured: &[String]) {
    crate::unported!("bot.knowledge.units:_named_ingredient")
}

/// Python: bot/knowledge/units.py:3576 `_plural_of(name)`
pub fn _plural_of(name: &str) -> String {
    crate::unported!("bot.knowledge.units:_plural_of")
}

/// Python: bot/knowledge/units.py:3588 `_a(name)` (never called on the trace corpus)
pub fn _a(name: &str) -> String {
    crate::unported!("bot.knowledge.units:_a")
}

/// Python: bot/knowledge/units.py:3596 `_counted_as(word)` (never called on the trace corpus)
pub fn _counted_as(word: &str) -> Value {
    crate::unported!("bot.knowledge.units:_counted_as")
}

/// Python: bot/knowledge/units.py:3611 `_piece_named(clean)`
pub fn _piece_named(clean: &str) {
    crate::unported!("bot.knowledge.units:_piece_named")
}

/// Python: bot/knowledge/units.py:3640 `_piece_weight(clean)`
pub fn _piece_weight(clean: &str) {
    crate::unported!("bot.knowledge.units:_piece_weight")
}

/// Python: bot/knowledge/units.py:3667 `_count_phrase(count)` (never called on the trace corpus)
pub fn _count_phrase(count: i64) -> Value {
    crate::unported!("bot.knowledge.units:_count_phrase")
}

/// Python: bot/knowledge/units.py:3683 `_named_unit(text)`
pub fn _named_unit(text: &str) -> Vec<String> {
    crate::unported!("bot.knowledge.units:_named_unit")
}

