//! Port of `bot/compose/reading.py`.
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

/// Python: bot/compose/reading.py:59 `_number(clean)`
pub fn _number(clean: &str) {
    crate::unported!("bot.compose.reading:_number")
}

/// Python: bot/compose/reading.py:76 `_range(said)` (never called on the trace corpus)
pub fn _range(said: &str) -> Value {
    crate::unported!("bot.compose.reading:_range")
}

/// Python: bot/compose/reading.py:88 `_say(value)` (never called on the trace corpus)
pub fn _say(value: &Value) -> String {
    crate::unported!("bot.compose.reading:_say")
}

/// Python: bot/compose/reading.py:93 `_unit(unit)` (never called on the trace corpus)
pub fn _unit(unit: &str) -> String {
    crate::unported!("bot.compose.reading:_unit")
}

/// Python: bot/compose/reading.py:128 `_word_number(word)` (never called on the trace corpus)
pub fn _word_number(word: &str) -> Value {
    crate::unported!("bot.compose.reading:_word_number")
}

/// Python: bot/compose/reading.py:142 `_plural(unit, high)` (never called on the trace corpus)
pub fn _plural(unit: &str, high: &Value) -> Value {
    crate::unported!("bot.compose.reading:_plural")
}

/// Python: bot/compose/reading.py:160 `_usual_unit(text, before=None)` (never called on the trace corpus)
pub fn _usual_unit(text: &str, before: &str) -> Value {
    crate::unported!("bot.compose.reading:_usual_unit")
}

/// Python: bot/compose/reading.py:182 `_add_measured(spans, low, high, unit, usual, at=None, matched='')` (never called on the trace corpus)
pub fn _add_measured(spans: &Value, low: &str, high: &Value, unit: &str, usual: &Value, at: i64, matched: &Value) {
    crate::unported!("bot.compose.reading:_add_measured")
}

/// Python: bot/compose/reading.py:201 `_same_measure(left, right)` (never called on the trace corpus)
pub fn _same_measure(left: &Value, right: &Value) -> bool {
    crate::unported!("bot.compose.reading:_same_measure")
}

/// Python: bot/compose/reading.py:212 `_where(value, low, high, unit)` (never called on the trace corpus)
pub fn _where(value: &Value, low: &str, high: &Value, unit: &str) -> String {
    crate::unported!("bot.compose.reading:_where")
}

/// Python: bot/compose/reading.py:221 `_step(said)` (never called on the trace corpus)
pub fn _step(said: &str) -> Value {
    crate::unported!("bot.compose.reading:_step")
}

/// Python: bot/compose/reading.py:235 `_said(memory, clean)` (never called on the trace corpus)
pub fn _said(memory: &Memory, clean: &str) -> Value {
    crate::unported!("bot.compose.reading:_said")
}

/// Python: bot/compose/reading.py:445 `_the_figure_in_the_doubt(rest)`
pub fn _the_figure_in_the_doubt(rest: &str) -> Value {
    crate::unported!("bot.compose.reading:_the_figure_in_the_doubt")
}

/// Python: bot/compose/reading.py:472 `_doubt(text)`
pub fn _doubt(text: &str) {
    crate::unported!("bot.compose.reading:_doubt")
}

/// Python: bot/compose/reading.py:580 `_doubt_ranges(said)` (never called on the trace corpus)
pub fn _doubt_ranges(said: &str) -> Value {
    crate::unported!("bot.compose.reading:_doubt_ranges")
}

/// Python: bot/compose/reading.py:630 `_an_age(text, span)` (never called on the trace corpus)
pub fn _an_age(text: &str, span: &Value) -> bool {
    crate::unported!("bot.compose.reading:_an_age")
}

/// Python: bot/compose/reading.py:638 `_money(value, mark='£')` (never called on the trace corpus)
pub fn _money(value: &Value, mark: &Value) -> String {
    crate::unported!("bot.compose.reading:_money")
}

/// Python: bot/compose/reading.py:762 `_they_named_the_tier(qualifier, mine)` (never called on the trace corpus)
pub fn _they_named_the_tier(qualifier: &Value, mine: &Memory) -> bool {
    crate::unported!("bot.compose.reading:_they_named_the_tier")
}

/// Python: bot/compose/reading.py:775 `_the_range_for(value, spans, said, mine='')` (never called on the trace corpus)
pub fn _the_range_for(value: &Value, spans: &Value, said: &str, mine: &Memory) -> Value {
    crate::unported!("bot.compose.reading:_the_range_for")
}

/// Python: bot/compose/reading.py:822 `_the_person_words(memory, line='')` (never called on the trace corpus)
pub fn _the_person_words(memory: &Memory, line: &str) -> String {
    crate::unported!("bot.compose.reading:_the_person_words")
}

/// Python: bot/compose/reading.py:836 `_the_span(doubt, said, mine='')` (never called on the trace corpus)
pub fn _the_span(doubt: &Value, said: &str, mine: &Memory) -> Value {
    crate::unported!("bot.compose.reading:_the_span")
}

/// Python: bot/compose/reading.py:921 `_the_side_asked(word)` (never called on the trace corpus)
pub fn _the_side_asked(word: &str) -> Option<String> {
    crate::unported!("bot.compose.reading:_the_side_asked")
}

/// Python: bot/compose/reading.py:931 `_the_side_decides(options, doubt, value, low, high, low_side, found_side)` (never called on the trace corpus)
pub fn _the_side_decides(options: &Value, doubt: &Value, value: &Value, low: &str, high: &Value, low_side: &Value, found_side: &Value) -> Value {
    crate::unported!("bot.compose.reading:_the_side_decides")
}

/// Python: bot/compose/reading.py:951 `_judged(doubt, said, mine='')` (never called on the trace corpus)
pub fn _judged(doubt: &Value, said: &str, mine: &Memory) -> Value {
    crate::unported!("bot.compose.reading:_judged")
}

/// Python: bot/compose/reading.py:1008 `_the_band_aside(said, span_text)` (never called on the trace corpus)
pub fn _the_band_aside(said: &str, span_text: &str) -> String {
    crate::unported!("bot.compose.reading:_the_band_aside")
}

/// Python: bot/compose/reading.py:1026 `_with_the_aside(line, span_text, aside)` (never called on the trace corpus)
pub fn _with_the_aside(line: &str, span_text: &str, aside: &Value) -> Value {
    crate::unported!("bot.compose.reading:_with_the_aside")
}

/// Python: bot/compose/reading.py:1057 `_in_the_unit(value, unit, wanted)` (never called on the trace corpus)
pub fn _in_the_unit(value: &Value, unit: &str, wanted: &Value) -> Value {
    crate::unported!("bot.compose.reading:_in_the_unit")
}

/// Python: bot/compose/reading.py:1066 `_a_stretch_of_time(unit)` (never called on the trace corpus)
pub fn _a_stretch_of_time(unit: &str) -> bool {
    crate::unported!("bot.compose.reading:_a_stretch_of_time")
}

/// Python: bot/compose/reading.py:1082 `_the_usual_is_the_answer(doubt, said)` (never called on the trace corpus)
pub fn _the_usual_is_the_answer(doubt: &Value, said: &str) -> Value {
    crate::unported!("bot.compose.reading:_the_usual_is_the_answer")
}

/// Python: bot/compose/reading.py:1123 `_a_word_figure(said)` (never called on the trace corpus)
pub fn _a_word_figure(said: &str) -> Value {
    crate::unported!("bot.compose.reading:_a_word_figure")
}

/// Python: bot/compose/reading.py:1144 `_the_last_figure_in(said)` (never called on the trace corpus)
pub fn _the_last_figure_in(said: &str) -> Value {
    crate::unported!("bot.compose.reading:_the_last_figure_in")
}

/// Python: bot/compose/reading.py:1155 `_in_the_same_league(value, said)` (never called on the trace corpus)
pub fn _in_the_same_league(value: &Value, said: &str) -> bool {
    crate::unported!("bot.compose.reading:_in_the_same_league")
}

/// Python: bot/compose/reading.py:1167 `_answer_before(memory, value=None)` (never called on the trace corpus)
pub fn _answer_before(memory: &Memory, value: &Value) -> Value {
    crate::unported!("bot.compose.reading:_answer_before")
}

/// Python: bot/compose/reading.py:1205 `_figure_since(memory, after, narrow=False)` (never called on the trace corpus)
pub fn _figure_since(memory: &Memory, after: &str, narrow: &Value) -> Value {
    crate::unported!("bot.compose.reading:_figure_since")
}

/// Python: bot/compose/reading.py:1248 `_a_clock_time(said, found, value)` (never called on the trace corpus)
pub fn _a_clock_time(said: &str, found: &Value, value: &Value) -> bool {
    crate::unported!("bot.compose.reading:_a_clock_time")
}

/// Python: bot/compose/reading.py:1259 `_the_figure_in(said, found)` (never called on the trace corpus)
pub fn _the_figure_in(said: &str, found: &Value) -> Value {
    crate::unported!("bot.compose.reading:_the_figure_in")
}

/// Python: bot/compose/reading.py:1336 `_a_lone_price(said)` (never called on the trace corpus)
pub fn _a_lone_price(said: &str) -> String {
    crate::unported!("bot.compose.reading:_a_lone_price")
}

/// Python: bot/compose/reading.py:1414 `asks_for_the_figure(doubt, said)` (never called on the trace corpus)
pub fn asks_for_the_figure(doubt: &Value, said: &str) -> Value {
    crate::unported!("bot.compose.reading:asks_for_the_figure")
}

/// Python: bot/compose/reading.py:1474 `_doubt_and_answer(clean, memory)`
pub fn _doubt_and_answer(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.reading:_doubt_and_answer")
}

/// Python: bot/compose/reading.py:1496 `_the_answer_is_about(said, subject)` (never called on the trace corpus)
pub fn _the_answer_is_about(said: &str, subject: &str) -> bool {
    crate::unported!("bot.compose.reading:_the_answer_is_about")
}

/// Python: bot/compose/reading.py:1513 `_only_says_that(doubt)` (never called on the trace corpus)
pub fn _only_says_that(doubt: &Value) -> bool {
    crate::unported!("bot.compose.reading:_only_says_that")
}

/// Python: bot/compose/reading.py:1528 `_the_usuals(subject)` (never called on the trace corpus)
pub fn _the_usuals(subject: &str) -> Value {
    crate::unported!("bot.compose.reading:_the_usuals")
}

/// Python: bot/compose/reading.py:1564 `_the_opening_sentence(reply)` (never called on the trace corpus)
pub fn _the_opening_sentence(reply: &str) -> Value {
    crate::unported!("bot.compose.reading:_the_opening_sentence")
}

/// Python: bot/compose/reading.py:1611 `_judged_on_its_subject(doubt)` (never called on the trace corpus)
pub fn _judged_on_its_subject(doubt: &Value) -> Value {
    crate::unported!("bot.compose.reading:_judged_on_its_subject")
}

/// Python: bot/compose/reading.py:1645 `_the_thing_the_chat_is_on(memory)` (never called on the trace corpus)
pub fn _the_thing_the_chat_is_on(memory: &Memory) -> Value {
    crate::unported!("bot.compose.reading:_the_thing_the_chat_is_on")
}

/// Python: bot/compose/reading.py:1659 `_the_doubt_with_its_thing(clean, memory)`
pub fn _the_doubt_with_its_thing(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.reading:_the_doubt_with_its_thing")
}

/// Python: bot/compose/reading.py:1678 `_its_own_subject_judges_it(clean, memory)`
pub fn _its_own_subject_judges_it(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.reading:_its_own_subject_judges_it")
}

/// Python: bot/compose/reading.py:1699 `_a_card_answers(text)` (never called on the trace corpus)
pub fn _a_card_answers(text: &str) -> Option<bool> {
    crate::unported!("bot.compose.reading:_a_card_answers")
}

/// Python: bot/compose/reading.py:1711 `judges_a_figure(clean, memory)`
pub fn judges_a_figure(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.reading:judges_a_figure")
}

/// Python: bot/compose/reading.py:1726 `judges_the_figure_on_the_table(clean, memory)`
pub fn judges_the_figure_on_the_table(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.reading:judges_the_figure_on_the_table")
}

/// Python: bot/compose/reading.py:1758 `_a_rate_of_theirs(memory)` (never called on the trace corpus)
pub fn _a_rate_of_theirs(memory: &Memory) -> bool {
    crate::unported!("bot.compose.reading:_a_rate_of_theirs")
}

/// Python: bot/compose/reading.py:1770 `_the_verdict_for_the_table(clean, memory)`
pub fn _the_verdict_for_the_table(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.reading:_the_verdict_for_the_table")
}

/// Python: bot/compose/reading.py:1833 `_the_price_verdict(clean, memory)` (never called on the trace corpus)
pub fn _the_price_verdict(clean: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.reading:_the_price_verdict")
}

/// Python: bot/compose/reading.py:1859 `_the_store_found_it_outside(clean, memory, a_price)` (never called on the trace corpus)
pub fn _the_store_found_it_outside(clean: &str, memory: &Memory, a_price: &Value) -> Option<bool> {
    crate::unported!("bot.compose.reading:_the_store_found_it_outside")
}

/// Python: bot/compose/reading.py:1880 `reply_to_a_doubt(clean, memory)`
pub fn reply_to_a_doubt(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.reading:reply_to_a_doubt")
}

/// Python: bot/compose/reading.py:1901 `_the_answer_about_this_figure(doubt, memory, at)` (never called on the trace corpus)
pub fn _the_answer_about_this_figure(doubt: &Value, memory: &Memory, at: i64) -> Value {
    crate::unported!("bot.compose.reading:_the_answer_about_this_figure")
}

/// Python: bot/compose/reading.py:1965 `_figure_words(number, unit, mark='')` (never called on the trace corpus)
pub fn _figure_words(number: &Value, unit: &str, mark: &Value) -> Value {
    crate::unported!("bot.compose.reading:_figure_words")
}

/// Python: bot/compose/reading.py:2007 `_their_figure(memory, back=1)` (never called on the trace corpus)
pub fn _their_figure(memory: &Memory, back: &Value) -> Value {
    crate::unported!("bot.compose.reading:_their_figure")
}

/// Python: bot/compose/reading.py:2091 `_already_judged(memory)` (never called on the trace corpus)
pub fn _already_judged(memory: &Memory) -> bool {
    crate::unported!("bot.compose.reading:_already_judged")
}

/// Python: bot/compose/reading.py:2106 `_a_bare_doubt(clean, memory)`
pub fn _a_bare_doubt(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.reading:_a_bare_doubt")
}

/// Python: bot/compose/reading.py:2141 `reply_to_a_bare_doubt(clean, memory)`
pub fn reply_to_a_bare_doubt(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.reading:reply_to_a_bare_doubt")
}

/// Python: bot/compose/reading.py:2159 `reply_for(clean, memory)`
pub fn reply_for(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.reading:reply_for")
}

/// Python: bot/compose/reading.py:2222 `pick_inside(value, span_text, said)` (never called on the trace corpus)
pub fn pick_inside(value: &Value, span_text: &str, said: &str) -> Value {
    crate::unported!("bot.compose.reading:pick_inside")
}

/// Python: bot/compose/reading.py:2227 `_figure_of_the_answer(memory, at)` (never called on the trace corpus)
pub fn _figure_of_the_answer(memory: &Memory, at: i64) -> Value {
    crate::unported!("bot.compose.reading:_figure_of_the_answer")
}

