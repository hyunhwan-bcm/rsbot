//! Port of `bot/memory/trips.py`.
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

/// Python: bot/memory/trips.py:189 `_one_thing_they_need(low)` (never called on the trace corpus)
pub fn _one_thing_they_need(low: &str) -> Value {
    crate::unported!("bot.memory.trips:_one_thing_they_need")
}

/// Python: bot/memory/trips.py:541 `_start(said)`
pub fn _start(said: &str) -> String {
    crate::unported!("bot.memory.trips:_start")
}

/// Python: bot/memory/trips.py:547 `_low(text)`
pub fn _low(text: &str) -> String {
    crate::unported!("bot.memory.trips:_low")
}

/// Python: bot/memory/trips.py:590 `_a_relation_not_a_place(place)`
pub fn _a_relation_not_a_place(place: &str) -> bool {
    crate::unported!("bot.memory.trips:_a_relation_not_a_place")
}

/// Python: bot/memory/trips.py:612 `_a_person_visited(line)`
pub fn _a_person_visited(line: &str) -> String {
    crate::unported!("bot.memory.trips:_a_person_visited")
}

/// Python: bot/memory/trips.py:631 `_places(line, a_visit=False)`
pub fn _places(line: &str, a_visit: bool) -> String {
    crate::unported!("bot.memory.trips:_places")
}

/// Python: bot/memory/trips.py:717 `_the_thing_booked(low)`
pub fn _the_thing_booked(low: &str) -> bool {
    crate::unported!("bot.memory.trips:_the_thing_booked")
}

/// Python: bot/memory/trips.py:732 `_a_thing_booked_for_when(low)`
pub fn _a_thing_booked_for_when(low: &str) -> bool {
    crate::unported!("bot.memory.trips:_a_thing_booked_for_when")
}

/// Python: bot/memory/trips.py:743 `_month(line)`
pub fn _month(line: &str) -> String {
    crate::unported!("bot.memory.trips:_month")
}

/// Python: bot/memory/trips.py:764 `_when_phrase(line)`
pub fn _when_phrase(line: &str) -> String {
    crate::unported!("bot.memory.trips:_when_phrase")
}

/// Python: bot/memory/trips.py:789 `_who(line)`
pub fn _who(line: &str) -> String {
    crate::unported!("bot.memory.trips:_who")
}

/// Python: bot/memory/trips.py:797 `_how_long(line)`
pub fn _how_long(line: &str) -> String {
    crate::unported!("bot.memory.trips:_how_long")
}

/// Python: bot/memory/trips.py:807 `_how_getting_there(line)`
pub fn _how_getting_there(line: &str) -> String {
    crate::unported!("bot.memory.trips:_how_getting_there")
}

/// Python: bot/memory/trips.py:812 `_by_the_sea(place)` (never called on the trace corpus)
pub fn _by_the_sea(place: &str) -> bool {
    crate::unported!("bot.memory.trips:_by_the_sea")
}

/// Python: bot/memory/trips.py:822 `_a_carded_place(place)`
pub fn _a_carded_place(place: &str) -> bool {
    crate::unported!("bot.memory.trips:_a_carded_place")
}

/// Python: bot/memory/trips.py:846 `a_moved_destination(line, memory)`
pub fn a_moved_destination(line: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.trips:a_moved_destination")
}

/// Python: bot/memory/trips.py:875 `_the_last_line_named_an_animal(memory)` (never called on the trace corpus)
pub fn _the_last_line_named_an_animal(memory: &Memory) -> bool {
    crate::unported!("bot.memory.trips:_the_last_line_named_an_animal")
}

/// Python: bot/memory/trips.py:894 `_keep_the_new_place(memory, place, old='', said='')` (never called on the trace corpus)
pub fn _keep_the_new_place(memory: &mut Memory, place: &str, old: &Value, said: &str) {
    crate::unported!("bot.memory.trips:_keep_the_new_place")
}

/// Python: bot/memory/trips.py:921 `_the_place_put_right(entries, old, place)` (never called on the trace corpus)
pub fn _the_place_put_right(entries: &Value, old: &Value, place: &str) -> Value {
    crate::unported!("bot.memory.trips:_the_place_put_right")
}

/// Python: bot/memory/trips.py:945 `a_trip(text, memory, cards=True)`
pub fn a_trip(text: &str, memory: &Memory, cards: bool) -> Value {
    crate::unported!("bot.memory.trips:a_trip")
}

/// Python: bot/memory/trips.py:999 `the_trip_they_booked(text)`
pub fn the_trip_they_booked(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.trips:the_trip_they_booked")
}

/// Python: bot/memory/trips.py:1016 `_the_trip_so_far(memory)`
pub fn _the_trip_so_far(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.trips:_the_trip_so_far")
}

/// Python: bot/memory/trips.py:1046 `_asked_last(memory)`
pub fn _asked_last(memory: &Memory) -> bool {
    crate::unported!("bot.memory.trips:_asked_last")
}

/// Python: bot/memory/trips.py:1051 `_told(text, memory)`
pub fn _told(text: &str, memory: &Memory) -> Vec<String> {
    crate::unported!("bot.memory.trips:_told")
}

/// Python: bot/memory/trips.py:1057 `_pick(options, memory, text)`
pub fn _pick(options: &[String], memory: &Memory, text: &str) -> String {
    crate::unported!("bot.memory.trips:_pick")
}

/// Python: bot/memory/trips.py:1075 `_join(parts)`
pub fn _join(parts: &[String]) -> String {
    crate::unported!("bot.memory.trips:_join")
}

/// Python: bot/memory/trips.py:1089 `_with_them(named, who, memory, line)` (never called on the trace corpus)
pub fn _with_them(named: &Value, who: &str, memory: &Memory, line: &str) -> String {
    crate::unported!("bot.memory.trips:_with_them")
}

/// Python: bot/memory/trips.py:1096 `_yours(who)` (never called on the trace corpus)
pub fn _yours(who: &str) -> String {
    crate::unported!("bot.memory.trips:_yours")
}

/// Python: bot/memory/trips.py:1114 `_mid(named)`
pub fn _mid(named: &str) -> String {
    crate::unported!("bot.memory.trips:_mid")
}

/// Python: bot/memory/trips.py:1120 `_cap(text)`
pub fn _cap(text: &str) -> String {
    crate::unported!("bot.memory.trips:_cap")
}

/// Python: bot/memory/trips.py:1137 `_seasons_we_know(place)`
pub fn _seasons_we_know(place: &str) -> String {
    crate::unported!("bot.memory.trips:_seasons_we_know")
}

/// Python: bot/memory/trips.py:1161 `_their_month(note, month)` (never called on the trace corpus)
pub fn _their_month(note: &str, month: &Value) -> String {
    crate::unported!("bot.memory.trips:_their_month")
}

/// Python: bot/memory/trips.py:1170 `_in_its_season(place, month)` (never called on the trace corpus)
pub fn _in_its_season(place: &str, month: &Value) -> bool {
    crate::unported!("bot.memory.trips:_in_its_season")
}

/// Python: bot/memory/trips.py:1193 `_a_winter_place(place)` (never called on the trace corpus)
pub fn _a_winter_place(place: &str) -> bool {
    crate::unported!("bot.memory.trips:_a_winter_place")
}

/// Python: bot/memory/trips.py:1201 `_when_note(month, memory, text, booked=False, place='')`
pub fn _when_note(month: &str, memory: &Memory, text: &str, booked: bool, place: &str) -> String {
    crate::unported!("bot.memory.trips:_when_note")
}

/// Python: bot/memory/trips.py:1219 `_the_visit_length(how_long, memory, text)` (never called on the trace corpus)
pub fn _the_visit_length(how_long: &Value, memory: &Memory, text: &str) -> Value {
    crate::unported!("bot.memory.trips:_the_visit_length")
}

/// Python: bot/memory/trips.py:1233 `_who_note(who, how_long, memory, text, place=None)`
pub fn _who_note(who: &str, how_long: &str, memory: &Memory, text: &str, place: &str) -> String {
    crate::unported!("bot.memory.trips:_who_note")
}

/// Python: bot/memory/trips.py:1260 `_a_note(month, who, how_long, memory, text, booked=False, place='')`
pub fn _a_note(month: &str, who: &str, how_long: &str, memory: &Memory, text: &str, booked: bool, place: &str) -> String {
    crate::unported!("bot.memory.trips:_a_note")
}

/// Python: bot/memory/trips.py:1288 `_a_known_city(place)` (never called on the trace corpus)
pub fn _a_known_city(place: &str) -> bool {
    crate::unported!("bot.memory.trips:_a_known_city")
}

/// Python: bot/memory/trips.py:1347 `_the_areas_of(place)` (never called on the trace corpus)
pub fn _the_areas_of(place: &str) -> String {
    crate::unported!("bot.memory.trips:_the_areas_of")
}

/// Python: bot/memory/trips.py:1356 `_somewhere_for(line, memory)`
pub fn _somewhere_for(line: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.trips:_somewhere_for")
}

/// Python: bot/memory/trips.py:1417 `_the_kind_of_trip(how_long)` (never called on the trace corpus)
pub fn _the_kind_of_trip(how_long: &Value) -> String {
    crate::unported!("bot.memory.trips:_the_kind_of_trip")
}

/// Python: bot/memory/trips.py:1433 `_the_warm_week_is_out_of_season(month)` (never called on the trace corpus)
pub fn _the_warm_week_is_out_of_season(month: &Value) -> bool {
    crate::unported!("bot.memory.trips:_the_warm_week_is_out_of_season")
}

/// Python: bot/memory/trips.py:1438 `_an_event_not_a_place(place)` (never called on the trace corpus)
pub fn _an_event_not_a_place(place: &str) -> bool {
    crate::unported!("bot.memory.trips:_an_event_not_a_place")
}

/// Python: bot/memory/trips.py:1448 `a_packing_question_about_the_trip(text, memory)`
pub fn a_packing_question_about_the_trip(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.trips:a_packing_question_about_the_trip")
}

/// Python: bot/memory/trips.py:1496 `the_bag_beats_the_list(text, memory)`
pub fn the_bag_beats_the_list(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.trips:the_bag_beats_the_list")
}

/// Python: bot/memory/trips.py:1508 `reply_for(text, memory, cards=True)`
pub fn reply_for(text: &str, memory: &mut Memory, cards: bool) -> Option<String> {
    crate::unported!("bot.memory.trips:reply_for")
}

/// Python: bot/memory/trips.py:1578 `_only_a_length(line)`
pub fn _only_a_length(line: &str) -> bool {
    crate::unported!("bot.memory.trips:_only_a_length")
}

/// Python: bot/memory/trips.py:1594 `_the_month_answered(line, memory)`
pub fn _the_month_answered(line: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.trips:_the_month_answered")
}

/// Python: bot/memory/trips.py:1612 `_as_a_visit(place)`
pub fn _as_a_visit(place: &str) -> String {
    crate::unported!("bot.memory.trips:_as_a_visit")
}

/// Python: bot/memory/trips.py:1640 `_a_new_trip(line, trip, memory)`
pub fn _a_new_trip(line: &str, trip: &Value, memory: &Memory) -> String {
    crate::unported!("bot.memory.trips:_a_new_trip")
}

/// Python: bot/memory/trips.py:1703 `_a_length_said(memory)` (never called on the trace corpus)
pub fn _a_length_said(memory: &Memory) -> Value {
    crate::unported!("bot.memory.trips:_a_length_said")
}

/// Python: bot/memory/trips.py:1724 `_the_length_asked(trip, memory, line)` (never called on the trace corpus)
pub fn _the_length_asked(trip: &Value, memory: &Memory, line: &str) -> String {
    crate::unported!("bot.memory.trips:_the_length_asked")
}

/// Python: bot/memory/trips.py:1735 `_a_detail(line, trip, memory)`
pub fn _a_detail(line: &str, trip: &Value, memory: &Memory) {
    crate::unported!("bot.memory.trips:_a_detail")
}

