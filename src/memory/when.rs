//! Port of `bot/memory/when.py`.
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

/// Python: bot/memory/when.py:171 `_is_a_trip_line(said)` (never called on the trace corpus)
pub fn _is_a_trip_line(said: &str) -> bool {
    crate::unported!("bot.memory.when:_is_a_trip_line")
}

/// Python: bot/memory/when.py:200 `_names_that_event(word, other)` (never called on the trace corpus)
pub fn _names_that_event(word: &str, other: &str) -> bool {
    crate::unported!("bot.memory.when:_names_that_event")
}

/// Python: bot/memory/when.py:275 `asks_what_today_is(text)`
pub fn asks_what_today_is(text: &str) -> bool {
    crate::unported!("bot.memory.when:asks_what_today_is")
}

/// Python: bot/memory/when.py:288 `_more_than_one_day(memory)` (never called on the trace corpus)
pub fn _more_than_one_day(memory: &Memory) -> bool {
    crate::unported!("bot.memory.when:_more_than_one_day")
}

/// Python: bot/memory/when.py:301 `when_answer(text, memory)`
pub fn when_answer(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.when:when_answer")
}

/// Python: bot/memory/when.py:395 `_is_an_ask(line)`
pub fn _is_an_ask(line: &str) -> bool {
    crate::unported!("bot.memory.when:_is_an_ask")
}

/// Python: bot/memory/when.py:403 `_backs_off(line)` (never called on the trace corpus)
pub fn _backs_off(line: &str) -> bool {
    crate::unported!("bot.memory.when:_backs_off")
}

/// Python: bot/memory/when.py:415 `_no_date_yet(asked, line, memory)`
pub fn _no_date_yet(asked: &[String], line: &str, memory: &Memory) {
    crate::unported!("bot.memory.when:_no_date_yet")
}

/// Python: bot/memory/when.py:457 `_a_day_and_a_month(day, month)` (never called on the trace corpus)
pub fn _a_day_and_a_month(day: &Value, month: &Value) -> String {
    crate::unported!("bot.memory.when:_a_day_and_a_month")
}

/// Python: bot/memory/when.py:473 `_points_back(line)`
pub fn _points_back(line: &str) -> bool {
    crate::unported!("bot.memory.when:_points_back")
}

/// Python: bot/memory/when.py:483 `the_plan(memory)`
pub fn the_plan(memory: &Memory) -> Option<String> {
    crate::unported!("bot.memory.when:the_plan")
}

/// Python: bot/memory/when.py:501 `_said_in_the_last_few_turns(memory, said)` (never called on the trace corpus)
pub fn _said_in_the_last_few_turns(memory: &Memory, said: &str) -> bool {
    crate::unported!("bot.memory.when:_said_in_the_last_few_turns")
}

/// Python: bot/memory/when.py:509 `_names_a_thing(text, spans)` (never called on the trace corpus)
pub fn _names_a_thing(text: &str, spans: &Value) -> bool {
    crate::unported!("bot.memory.when:_names_a_thing")
}

/// Python: bot/memory/when.py:533 `_the_thing_before_the_date(memory, said, asked)` (never called on the trace corpus)
pub fn _the_thing_before_the_date(memory: &Memory, said: &str, asked: &str) -> Value {
    crate::unported!("bot.memory.when:_the_thing_before_the_date")
}

/// Python: bot/memory/when.py:580 `_the_line_put_right(memory)`
pub fn _the_line_put_right(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.memory.when:_the_line_put_right")
}

/// Python: bot/memory/when.py:618 `_dated(memory)`
pub fn _dated(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.memory.when:_dated")
}

/// Python: bot/memory/when.py:696 `_lead(found)` (never called on the trace corpus)
pub fn _lead(found: &Value) -> Value {
    crate::unported!("bot.memory.when:_lead")
}

/// Python: bot/memory/when.py:717 `_time_for(asked, said, spans)` (never called on the trace corpus)
pub fn _time_for(asked: &str, said: &str, spans: &Value) -> Value {
    crate::unported!("bot.memory.when:_time_for")
}

/// Python: bot/memory/when.py:758 `_with_the_month_after(said, found)` (never called on the trace corpus)
pub fn _with_the_month_after(said: &str, found: &Value) -> Value {
    crate::unported!("bot.memory.when:_with_the_month_after")
}

/// Python: bot/memory/when.py:769 `_names_a_thing_of_its_own(line, said)` (never called on the trace corpus)
pub fn _names_a_thing_of_its_own(line: &str, said: &str) -> bool {
    crate::unported!("bot.memory.when:_names_a_thing_of_its_own")
}

/// Python: bot/memory/when.py:816 `_a_doing_word(words)` (never called on the trace corpus)
pub fn _a_doing_word(words: &[String]) -> bool {
    crate::unported!("bot.memory.when:_a_doing_word")
}

/// Python: bot/memory/when.py:827 `_plain(when)` (never called on the trace corpus)
pub fn _plain(when: &str) -> String {
    crate::unported!("bot.memory.when:_plain")
}

/// Python: bot/memory/when.py:835 `_put_right(memory)`
pub fn _put_right(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.memory.when:_put_right")
}

/// Python: bot/memory/when.py:869 `_a_date_said(value)` (never called on the trace corpus)
pub fn _a_date_said(value: &Value) -> String {
    crate::unported!("bot.memory.when:_a_date_said")
}

/// Python: bot/memory/when.py:890 `_same(word, other)` (never called on the trace corpus)
pub fn _same(word: &str, other: &str) -> bool {
    crate::unported!("bot.memory.when:_same")
}

/// Python: bot/memory/when.py:902 `_root(word)` (never called on the trace corpus)
pub fn _root(word: &str) -> Value {
    crate::unported!("bot.memory.when:_root")
}

/// Python: bot/memory/when.py:916 `_a_plural_stem(word)` (never called on the trace corpus)
pub fn _a_plural_stem(word: &str) -> Value {
    crate::unported!("bot.memory.when:_a_plural_stem")
}

/// Python: bot/memory/when.py:960 `the_part_asked(plan, low, line=None, is_a_place=None)` (never called on the trace corpus)
pub fn the_part_asked(plan: &str, low: &str, line: &str, is_a_place: &Value) -> String {
    crate::unported!("bot.memory.when:the_part_asked")
}

/// Python: bot/memory/when.py:991 `_the_place(plan, line, is_a_place=None)` (never called on the trace corpus)
pub fn _the_place(plan: &str, line: &str, is_a_place: &Value) -> Value {
    crate::unported!("bot.memory.when:_the_place")
}

/// Python: bot/memory/when.py:1020 `_a_name(place)` (never called on the trace corpus)
pub fn _a_name(place: &str) -> String {
    crate::unported!("bot.memory.when:_a_name")
}

/// Python: bot/memory/when.py:1038 `_without_the_date(text)` (never called on the trace corpus)
pub fn _without_the_date(text: &str) -> String {
    crate::unported!("bot.memory.when:_without_the_date")
}

/// Python: bot/memory/when.py:1046 `_names_a_place(place)` (never called on the trace corpus)
pub fn _names_a_place(place: &str) -> bool {
    crate::unported!("bot.memory.when:_names_a_place")
}

/// Python: bot/memory/when.py:1059 `_the_dates_in(text)` (never called on the trace corpus)
pub fn _the_dates_in(text: &str) -> Value {
    crate::unported!("bot.memory.when:_the_dates_in")
}

/// Python: bot/memory/when.py:1071 `_the_date_asked(ask, dates)` (never called on the trace corpus)
pub fn _the_date_asked(ask: &str, dates: &Value) -> Value {
    crate::unported!("bot.memory.when:_the_date_asked")
}

/// Python: bot/memory/when.py:1116 `already_answer(text, memory)`
pub fn already_answer(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.when:already_answer")
}

/// Python: bot/memory/when.py:1157 `_has_happened(said)` (never called on the trace corpus)
pub fn _has_happened(said: &str) -> bool {
    crate::unported!("bot.memory.when:_has_happened")
}

/// Python: bot/memory/when.py:1167 `_was_sentence(when)` (never called on the trace corpus)
pub fn _was_sentence(when: &str) -> String {
    crate::unported!("bot.memory.when:_was_sentence")
}

/// Python: bot/memory/when.py:1176 `_sentence(when)` (never called on the trace corpus)
pub fn _sentence(when: &str) -> String {
    crate::unported!("bot.memory.when:_sentence")
}

/// Python: bot/memory/when.py:1221 `_the_interval_it_means(line, memory)` (never called on the trace corpus)
pub fn _the_interval_it_means(line: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.when:_the_interval_it_means")
}

/// Python: bot/memory/when.py:1253 `_said_before_now(memory, turn)` (never called on the trace corpus)
pub fn _said_before_now(memory: &Memory, turn: &Value) -> String {
    crate::unported!("bot.memory.when:_said_before_now")
}

/// Python: bot/memory/when.py:1285 `_said_in_the_past(line)` (never called on the trace corpus)
pub fn _said_in_the_past(line: &str) -> bool {
    crate::unported!("bot.memory.when:_said_in_the_past")
}

/// Python: bot/memory/when.py:1338 `_a_trip_is_named(dated)` (never called on the trace corpus)
pub fn _a_trip_is_named(dated: &Value) -> bool {
    crate::unported!("bot.memory.when:_a_trip_is_named")
}

/// Python: bot/memory/when.py:1350 `_a_bare_unit_ask_wants_a_length(line, dated)` (never called on the trace corpus)
pub fn _a_bare_unit_ask_wants_a_length(line: &str, dated: &Value) -> bool {
    crate::unported!("bot.memory.when:_a_bare_unit_ask_wants_a_length")
}

/// Python: bot/memory/when.py:1372 `until_ask_matches(line, again=None)`
pub fn until_ask_matches(line: &str, again: &Value) -> bool {
    crate::unported!("bot.memory.when:until_ask_matches")
}

/// Python: bot/memory/when.py:1393 `_today()` (never called on the trace corpus)
pub fn _today() -> Value {
    crate::unported!("bot.memory.when:_today")
}

/// Python: bot/memory/when.py:1398 `until_answer(text, memory)`
pub fn until_answer(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.when:until_answer")
}

/// Python: bot/memory/when.py:1452 `_the_date_it_means(line, memory)` (never called on the trace corpus)
pub fn _the_date_it_means(line: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.when:_the_date_it_means")
}

/// Python: bot/memory/when.py:1483 `_the_newest_date(memory)` (never called on the trace corpus)
pub fn _the_newest_date(memory: &Memory) -> Value {
    crate::unported!("bot.memory.when:_the_newest_date")
}

/// Python: bot/memory/when.py:1501 `_a_day_but_no_month(when)` (never called on the trace corpus)
pub fn _a_day_but_no_month(when: &str) -> bool {
    crate::unported!("bot.memory.when:_a_day_but_no_month")
}

/// Python: bot/memory/when.py:1518 `_the_month_in_play(when, memory)` (never called on the trace corpus)
pub fn _the_month_in_play(when: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.when:_the_month_in_play")
}

/// Python: bot/memory/when.py:1561 `_a_day_number(word)` (never called on the trace corpus)
pub fn _a_day_number(word: &str) -> bool {
    crate::unported!("bot.memory.when:_a_day_number")
}

/// Python: bot/memory/when.py:1565 `_the_day(when, today=None)` (never called on the trace corpus)
pub fn _the_day(when: &str, today: &Value) -> Value {
    crate::unported!("bot.memory.when:_the_day")
}

/// Python: bot/memory/when.py:1649 `_this_month_or_next(today, day)` (never called on the trace corpus)
pub fn _this_month_or_next(today: &Value, day: &Value) -> Value {
    crate::unported!("bot.memory.when:_this_month_or_next")
}

/// Python: bot/memory/when.py:1660 `_a_year_for(today, month, day)` (never called on the trace corpus)
pub fn _a_year_for(today: &Value, month: &Value, day: &Value) -> Value {
    crate::unported!("bot.memory.when:_a_year_for")
}

/// Python: bot/memory/when.py:1676 `_days_in(year, month)` (never called on the trace corpus)
pub fn _days_in(year: &Value, month: &Value) -> Value {
    crate::unported!("bot.memory.when:_days_in")
}

/// Python: bot/memory/when.py:1682 `_last_day(first_of_month)` (never called on the trace corpus)
pub fn _last_day(first_of_month: &Value) -> String {
    crate::unported!("bot.memory.when:_last_day")
}

/// Python: bot/memory/when.py:1686 `_add_months(day, months)` (never called on the trace corpus)
pub fn _add_months(day: &Value, months: &Value) -> Value {
    crate::unported!("bot.memory.when:_add_months")
}

/// Python: bot/memory/when.py:1695 `_how_long(day, kind, line)` (never called on the trace corpus)
pub fn _how_long(day: &Value, kind: &str, line: &str) -> Option<String> {
    crate::unported!("bot.memory.when:_how_long")
}

/// Python: bot/memory/when.py:1740 `_how_long_ago(day, kind, line)` (never called on the trace corpus)
pub fn _how_long_ago(day: &Value, kind: &str, line: &str) -> Option<String> {
    crate::unported!("bot.memory.when:_how_long_ago")
}

/// Python: bot/memory/when.py:1786 `_a_day_in_the_past(day, today)` (never called on the trace corpus)
pub fn _a_day_in_the_past(day: &Value, today: &Value) -> Value {
    crate::unported!("bot.memory.when:_a_day_in_the_past")
}

/// Python: bot/memory/when.py:1799 `_the_day_said(day, kind)` (never called on the trace corpus)
pub fn _the_day_said(day: &Value, kind: &str) -> String {
    crate::unported!("bot.memory.when:_the_day_said")
}

/// Python: bot/memory/when.py:1831 `a_plan_moved_to_a_day(text)`
pub fn a_plan_moved_to_a_day(text: &str) {
    crate::unported!("bot.memory.when:a_plan_moved_to_a_day")
}

/// Python: bot/memory/when.py:1958 `how_long_answer(text, memory)`
pub fn how_long_answer(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.when:how_long_answer")
}

/// Python: bot/memory/when.py:1997 `_a_day_put_right(memory)` (never called on the trace corpus)
pub fn _a_day_put_right(memory: &Memory) -> Value {
    crate::unported!("bot.memory.when:_a_day_put_right")
}

/// Python: bot/memory/when.py:2022 `_the_lines_that_began(memory)` (never called on the trace corpus)
pub fn _the_lines_that_began(memory: &Memory) -> Value {
    crate::unported!("bot.memory.when:_the_lines_that_began")
}

/// Python: bot/memory/when.py:2048 `_names_it(named, said)` (never called on the trace corpus)
pub fn _names_it(named: &Value, said: &str) -> bool {
    crate::unported!("bot.memory.when:_names_it")
}

/// Python: bot/memory/when.py:2062 `_the_length_from(said)` (never called on the trace corpus)
pub fn _the_length_from(said: &str) -> Value {
    crate::unported!("bot.memory.when:_the_length_from")
}

/// Python: bot/memory/when.py:2117 `memory_of_one(said)` (never called on the trace corpus)
pub fn memory_of_one(said: &str) -> Value {
    crate::unported!("bot.memory.when:memory_of_one")
}

/// Python: bot/memory/when.py:2122 `_a_count(said)` (never called on the trace corpus)
pub fn _a_count(said: &str) -> Value {
    crate::unported!("bot.memory.when:_a_count")
}

/// Python: bot/memory/when.py:2130 `_the_day_a_span_names(span)` (never called on the trace corpus)
pub fn _the_day_a_span_names(span: &Value) -> Value {
    crate::unported!("bot.memory.when:_the_day_a_span_names")
}

/// Python: bot/memory/when.py:2164 `_the_day_a_weekday_names(word)` (never called on the trace corpus)
pub fn _the_day_a_weekday_names(word: &str) -> Value {
    crate::unported!("bot.memory.when:_the_day_a_weekday_names")
}

/// Python: bot/memory/when.py:2173 `_the_month_number(word)` (never called on the trace corpus)
pub fn _the_month_number(word: &str) -> Value {
    crate::unported!("bot.memory.when:_the_month_number")
}

/// Python: bot/memory/when.py:2183 `_the_month_back(months)` (never called on the trace corpus)
pub fn _the_month_back(months: &Value) -> Value {
    crate::unported!("bot.memory.when:_the_month_back")
}

/// Python: bot/memory/when.py:2194 `_the_month_name(number)` (never called on the trace corpus)
pub fn _the_month_name(number: &Value) -> Value {
    crate::unported!("bot.memory.when:_the_month_name")
}

/// Python: bot/memory/when.py:2200 `_a_day_said(day)` (never called on the trace corpus)
pub fn _a_day_said(day: &Value) -> Value {
    crate::unported!("bot.memory.when:_a_day_said")
}

/// Python: bot/memory/when.py:2207 `_an_ordinal_end(number)` (never called on the trace corpus)
pub fn _an_ordinal_end(number: &Value) -> Value {
    crate::unported!("bot.memory.when:_an_ordinal_end")
}

/// Python: bot/memory/when.py:2220 `_a_count_said(number)` (never called on the trace corpus)
pub fn _a_count_said(number: &Value) -> Value {
    crate::unported!("bot.memory.when:_a_count_said")
}

/// Python: bot/memory/when.py:2226 `_a_length_in_words(days)` (never called on the trace corpus)
pub fn _a_length_in_words(days: &Value) -> String {
    crate::unported!("bot.memory.when:_a_length_in_words")
}

/// Python: bot/memory/when.py:2244 `_about(length, when)` (never called on the trace corpus)
pub fn _about(length: &Value, when: &str) -> Option<String> {
    crate::unported!("bot.memory.when:_about")
}

/// Python: bot/memory/when.py:2254 `_tidy(said)` (never called on the trace corpus)
pub fn _tidy(said: &str) -> Value {
    crate::unported!("bot.memory.when:_tidy")
}

