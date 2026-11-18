//! Port of `bot/memory/narrow.py`.
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

/// Python: bot/memory/narrow.py:143 `only_a_length(text)`
pub fn only_a_length(text: &str) -> bool {
    crate::unported!("bot.memory.narrow:only_a_length")
}

/// Python: bot/memory/narrow.py:218 `_the_remedy_in(said)` (never called on the trace corpus)
pub fn _the_remedy_in(said: &str) -> Value {
    crate::unported!("bot.memory.narrow:_the_remedy_in")
}

/// Python: bot/memory/narrow.py:244 `carries(reply, kind)` (never called on the trace corpus)
pub fn carries(reply: &str, kind: &str) -> bool {
    crate::unported!("bot.memory.narrow:carries")
}

/// Python: bot/memory/narrow.py:291 `has_a_date(sentence)` (never called on the trace corpus)
pub fn has_a_date(sentence: &str) -> bool {
    crate::unported!("bot.memory.narrow:has_a_date")
}

/// Python: bot/memory/narrow.py:623 `pieces_of(text, pattern)`
pub fn pieces_of(text: &str, pattern: &Value) -> Vec<String> {
    crate::unported!("bot.memory.narrow:pieces_of")
}

/// Python: bot/memory/narrow.py:659 `_same(words)` (never called on the trace corpus)
pub fn _same(words: &[String]) {
    crate::unported!("bot.memory.narrow:_same")
}

/// Python: bot/memory/narrow.py:731 `step_word(question)`
pub fn step_word(question: &str) -> Option<String> {
    crate::unported!("bot.memory.narrow:step_word")
}

/// Python: bot/memory/narrow.py:737 `kind_of(question)`
pub fn kind_of(question: &str) -> Option<String> {
    crate::unported!("bot.memory.narrow:kind_of")
}

/// Python: bot/memory/narrow.py:752 `asks_a_part(question)`
pub fn asks_a_part(question: &str) -> bool {
    crate::unported!("bot.memory.narrow:asks_a_part")
}

/// Python: bot/memory/narrow.py:760 `_named_follow_up(question)`
pub fn _named_follow_up(question: &str) -> bool {
    crate::unported!("bot.memory.narrow:_named_follow_up")
}

/// Python: bot/memory/narrow.py:777 `_gives_a_figure_in(part, unit)` (never called on the trace corpus)
pub fn _gives_a_figure_in(part: &str, unit: &str) -> bool {
    crate::unported!("bot.memory.narrow:_gives_a_figure_in")
}

/// Python: bot/memory/narrow.py:791 `_the_clause_in_the_unit(sentence, unit)` (never called on the trace corpus)
pub fn _the_clause_in_the_unit(sentence: &str, unit: &str) -> Option<String> {
    crate::unported!("bot.memory.narrow:_the_clause_in_the_unit")
}

/// Python: bot/memory/narrow.py:808 `_the_sentence_in_the_unit(reply, unit)` (never called on the trace corpus)
pub fn _the_sentence_in_the_unit(reply: &str, unit: &str) -> Value {
    crate::unported!("bot.memory.narrow:_the_sentence_in_the_unit")
}

/// Python: bot/memory/narrow.py:824 `part_in_the_unit(reply, question)`
pub fn part_in_the_unit(reply: &str, question: &str) {
    crate::unported!("bot.memory.narrow:part_in_the_unit")
}

/// Python: bot/memory/narrow.py:865 `asked_unit(question)`
pub fn asked_unit(question: &str) {
    crate::unported!("bot.memory.narrow:asked_unit")
}

/// Python: bot/memory/narrow.py:871 `_states_a_length(part)` (never called on the trace corpus)
pub fn _states_a_length(part: &str) -> bool {
    crate::unported!("bot.memory.narrow:_states_a_length")
}

/// Python: bot/memory/narrow.py:917 `the_outcome_asked(question)`
pub fn the_outcome_asked(question: &str) -> String {
    crate::unported!("bot.memory.narrow:the_outcome_asked")
}

/// Python: bot/memory/narrow.py:926 `the_part_asked(reply, question, about='')`
pub fn the_part_asked(reply: &str, question: &str, about: &str) -> Option<String> {
    crate::unported!("bot.memory.narrow:the_part_asked")
}

/// Python: bot/memory/narrow.py:952 `_a_rate_cut_from_its_figure(reply, part, question)` (never called on the trace corpus)
pub fn _a_rate_cut_from_its_figure(reply: &str, part: &str, question: &str) -> bool {
    crate::unported!("bot.memory.narrow:_a_rate_cut_from_its_figure")
}

/// Python: bot/memory/narrow.py:982 `narrow(reply, question, about='')`
pub fn narrow(reply: &str, question: &str, about: &str) -> Option<String> {
    crate::unported!("bot.memory.narrow:narrow")
}

/// Python: bot/memory/narrow.py:1029 `_a_length_of_a_doing(sentence, part, question)`
pub fn _a_length_of_a_doing(sentence: &str, part: &str, question: &str) -> bool {
    crate::unported!("bot.memory.narrow:_a_length_of_a_doing")
}

/// Python: bot/memory/narrow.py:1085 `_a_cut_off_not_a_length(sentence, part)`
pub fn _a_cut_off_not_a_length(sentence: &str, part: &str) -> bool {
    crate::unported!("bot.memory.narrow:_a_cut_off_not_a_length")
}

/// Python: bot/memory/narrow.py:1113 `_a_clause_before(said, at)` (never called on the trace corpus)
pub fn _a_clause_before(said: &str, at: i64) -> Value {
    crate::unported!("bot.memory.narrow:_a_clause_before")
}

/// Python: bot/memory/narrow.py:1129 `the_doings_length(reply, question)` (never called on the trace corpus)
pub fn the_doings_length(reply: &str, question: &str) -> Value {
    crate::unported!("bot.memory.narrow:the_doings_length")
}

/// Python: bot/memory/narrow.py:1166 `_the_clause_around(said, at)` (never called on the trace corpus)
pub fn _the_clause_around(said: &str, at: i64) -> String {
    crate::unported!("bot.memory.narrow:_the_clause_around")
}

/// Python: bot/memory/narrow.py:1183 `_at_another_place(sentence, part, question)`
pub fn _at_another_place(sentence: &str, part: &str, question: &str) -> bool {
    crate::unported!("bot.memory.narrow:_at_another_place")
}

/// Python: bot/memory/narrow.py:1206 `_gives_another_places_figure(sentence, kind, question)`
pub fn _gives_another_places_figure(sentence: &str, kind: Option<&str>, question: &str) -> bool {
    crate::unported!("bot.memory.narrow:_gives_another_places_figure")
}

/// Python: bot/memory/narrow.py:1216 `_narrow(reply, question)`
pub fn _narrow(reply: &str, question: &str) -> Option<String> {
    crate::unported!("bot.memory.narrow:_narrow")
}

/// Python: bot/memory/narrow.py:1438 `_the_start_age(sentence, figure, at)` (never called on the trace corpus)
pub fn _the_start_age(sentence: &str, figure: &Value, at: i64) -> Option<String> {
    crate::unported!("bot.memory.narrow:_the_start_age")
}

/// Python: bot/memory/narrow.py:1452 `_the_time_it_does_keep(sentences, whole)`
pub fn _the_time_it_does_keep(sentences: &[String], whole: &str) {
    crate::unported!("bot.memory.narrow:_the_time_it_does_keep")
}

/// Python: bot/memory/narrow.py:1516 `_a_period_of_extent(sentence, at)` (never called on the trace corpus)
pub fn _a_period_of_extent(sentence: &str, at: i64) -> bool {
    crate::unported!("bot.memory.narrow:_a_period_of_extent")
}

/// Python: bot/memory/narrow.py:1521 `_says_a_time(clause)` (never called on the trace corpus)
pub fn _says_a_time(clause: &str) -> bool {
    crate::unported!("bot.memory.narrow:_says_a_time")
}

/// Python: bot/memory/narrow.py:1533 `_only_a_cut_off(clause)` (never called on the trace corpus)
pub fn _only_a_cut_off(clause: &str) -> bool {
    crate::unported!("bot.memory.narrow:_only_a_cut_off")
}

/// Python: bot/memory/narrow.py:1546 `_the_answer_said_again(sentences, whole)` (never called on the trace corpus)
pub fn _the_answer_said_again(sentences: &Value, whole: &Value) -> Value {
    crate::unported!("bot.memory.narrow:_the_answer_said_again")
}

/// Python: bot/memory/narrow.py:1568 `_the_sentence_a_figure_hangs_on(part, sentence, whole, when_ask=False)`
pub fn _the_sentence_a_figure_hangs_on(part: &str, sentence: &str, whole: &str, when_ask: bool) {
    crate::unported!("bot.memory.narrow:_the_sentence_a_figure_hangs_on")
}

/// Python: bot/memory/narrow.py:1621 `_unit_stem(word)` (never called on the trace corpus)
pub fn _unit_stem(word: &str) -> Value {
    crate::unported!("bot.memory.narrow:_unit_stem")
}

/// Python: bot/memory/narrow.py:1638 `_same_word(one, other)`
pub fn _same_word(one: &str, other: &str) -> bool {
    crate::unported!("bot.memory.narrow:_same_word")
}

/// Python: bot/memory/narrow.py:1646 `_names_one_of(wanted, words)`
pub fn _names_one_of(wanted: &[String], words: &[String]) -> bool {
    crate::unported!("bot.memory.narrow:_names_one_of")
}

/// Python: bot/memory/narrow.py:1679 `_only_a_count(piece)` (never called on the trace corpus)
pub fn _only_a_count(piece: &str) -> bool {
    crate::unported!("bot.memory.narrow:_only_a_count")
}

/// Python: bot/memory/narrow.py:1732 `_how_often(sentences, question, whole)`
pub fn _how_often(sentences: &[String], question: &str, whole: &str) {
    crate::unported!("bot.memory.narrow:_how_often")
}

/// Python: bot/memory/narrow.py:1828 `_counts_on_the_answer(piece)`
pub fn _counts_on_the_answer(piece: &str) -> bool {
    crate::unported!("bot.memory.narrow:_counts_on_the_answer")
}

/// Python: bot/memory/narrow.py:1833 `_freq_clause(sentence, weak=False, wanted=frozenset())`
pub fn _freq_clause(sentence: &str, weak: bool, wanted: &[String]) -> Option<String> {
    crate::unported!("bot.memory.narrow:_freq_clause")
}

/// Python: bot/memory/narrow.py:1880 `_dated(sentences, question, whole)` (never called on the trace corpus)
pub fn _dated(sentences: &Value, question: &str, whole: &Value) -> Value {
    crate::unported!("bot.memory.narrow:_dated")
}

/// Python: bot/memory/narrow.py:1911 `_the_part_they_named(sentences, question, whole)`
pub fn _the_part_they_named(sentences: &[String], question: &str, whole: &str) {
    crate::unported!("bot.memory.narrow:_the_part_they_named")
}

/// Python: bot/memory/narrow.py:1927 `_not_a_figure(sentences, question, whole, reply=None)`
pub fn _not_a_figure(sentences: &[String], question: &str, whole: &str, reply: &str) {
    crate::unported!("bot.memory.narrow:_not_a_figure")
}

/// Python: bot/memory/narrow.py:1953 `_named_clause(answer, question)`
pub fn _named_clause(answer: &str, question: &str) -> Option<String> {
    crate::unported!("bot.memory.narrow:_named_clause")
}

/// Python: bot/memory/narrow.py:2000 `_root(word)`
pub fn _root(word: &str) -> String {
    crate::unported!("bot.memory.narrow:_root")
}

/// Python: bot/memory/narrow.py:2009 `_says(text, phrase)`
pub fn _says(text: &str, phrase: &str) -> bool {
    crate::unported!("bot.memory.narrow:_says")
}

/// Python: bot/memory/narrow.py:2015 `_informative(piece)`
pub fn _informative(piece: &str) -> bool {
    crate::unported!("bot.memory.narrow:_informative")
}

/// Python: bot/memory/narrow.py:2020 `_cuts_a_list(piece, answer)` (never called on the trace corpus)
pub fn _cuts_a_list(piece: &str, answer: &str) -> bool {
    crate::unported!("bot.memory.narrow:_cuts_a_list")
}

/// Python: bot/memory/narrow.py:2036 `_with_unit(piece, reply)`
pub fn _with_unit(piece: &str, reply: &str) -> String {
    crate::unported!("bot.memory.narrow:_with_unit")
}

/// Python: bot/memory/narrow.py:2053 `_property_ask(question)`
pub fn _property_ask(question: &str) -> bool {
    crate::unported!("bot.memory.narrow:_property_ask")
}

/// Python: bot/memory/narrow.py:2075 `_option_ask(question)`
pub fn _option_ask(question: &str) -> bool {
    crate::unported!("bot.memory.narrow:_option_ask")
}

/// Python: bot/memory/narrow.py:2082 `_stands_alone(piece)` (never called on the trace corpus)
pub fn _stands_alone(piece: &str) -> bool {
    crate::unported!("bot.memory.narrow:_stands_alone")
}

/// Python: bot/memory/narrow.py:2099 `_by_option(sentences, question)`
pub fn _by_option(sentences: &[String], question: &str) {
    crate::unported!("bot.memory.narrow:_by_option")
}

/// Python: bot/memory/narrow.py:2118 `_only_a_qualifier(clause)` (never called on the trace corpus)
pub fn _only_a_qualifier(clause: &str) -> bool {
    crate::unported!("bot.memory.narrow:_only_a_qualifier")
}

/// Python: bot/memory/narrow.py:2138 `_by_step(sentences, question)`
pub fn _by_step(sentences: &[String], question: &str) {
    crate::unported!("bot.memory.narrow:_by_step")
}

/// Python: bot/memory/narrow.py:2164 `_with_the_script(part, sentences, reply=None)` (never called on the trace corpus)
pub fn _with_the_script(part: &str, sentences: &Value, reply: &str) -> Option<String> {
    crate::unported!("bot.memory.narrow:_with_the_script")
}

/// Python: bot/memory/narrow.py:2208 `_the_quotation(sentence)` (never called on the trace corpus)
pub fn _the_quotation(sentence: &str) -> Option<String> {
    crate::unported!("bot.memory.narrow:_the_quotation")
}

/// Python: bot/memory/narrow.py:2220 `_to_the_end_of_the_script(sentences, at)` (never called on the trace corpus)
pub fn _to_the_end_of_the_script(sentences: &Value, at: i64) -> Value {
    crate::unported!("bot.memory.narrow:_to_the_end_of_the_script")
}

/// Python: bot/memory/narrow.py:2237 `_by_words(sentences, question)`
pub fn _by_words(sentences: &[String], question: &str) -> Option<String> {
    crate::unported!("bot.memory.narrow:_by_words")
}

/// Python: bot/memory/narrow.py:2260 `_as_a_length(text)`
pub fn _as_a_length(text: &str) -> String {
    crate::unported!("bot.memory.narrow:_as_a_length")
}

/// Python: bot/memory/narrow.py:2279 `_names_a_thing(question)`
pub fn _names_a_thing(question: &str) -> bool {
    crate::unported!("bot.memory.narrow:_names_a_thing")
}

/// Python: bot/memory/narrow.py:2302 `_measure(span)` (never called on the trace corpus)
pub fn _measure(span: &Value) -> String {
    crate::unported!("bot.memory.narrow:_measure")
}

/// Python: bot/memory/narrow.py:2308 `_figure(match, text)`
pub fn _figure(r#match: &Value, text: &str) -> String {
    crate::unported!("bot.memory.narrow:_figure")
}

/// Python: bot/memory/narrow.py:2329 `_a_figure_each(sentence, kind)`
pub fn _a_figure_each(sentence: &str, kind: &str) {
    crate::unported!("bot.memory.narrow:_a_figure_each")
}

/// Python: bot/memory/narrow.py:2373 `_the_item_they_named(sentence, kind, question)`
pub fn _the_item_they_named(sentence: &str, kind: &str, question: &str) {
    crate::unported!("bot.memory.narrow:_the_item_they_named")
}

/// Python: bot/memory/narrow.py:2427 `_the_half_they_named(halves, question)` (never called on the trace corpus)
pub fn _the_half_they_named(halves: &Value, question: &str) -> Value {
    crate::unported!("bot.memory.narrow:_the_half_they_named")
}

/// Python: bot/memory/narrow.py:2478 `_a_condition(sentence, match)` (never called on the trace corpus)
pub fn _a_condition(sentence: &str, r#match: &Value) -> bool {
    crate::unported!("bot.memory.narrow:_a_condition")
}

/// Python: bot/memory/narrow.py:2494 `_part(sentence, kind, question)`
pub fn _part(sentence: &str, kind: &str, question: &str) -> Option<String> {
    crate::unported!("bot.memory.narrow:_part")
}

/// Python: bot/memory/narrow.py:2631 `_canonical(word)`
pub fn _canonical(word: &str) -> String {
    crate::unported!("bot.memory.narrow:_canonical")
}

/// Python: bot/memory/narrow.py:2636 `_a_frequency(sentence, end, start=None)`
pub fn _a_frequency(sentence: &str, end: i64, start: Option<i64>) -> bool {
    crate::unported!("bot.memory.narrow:_a_frequency")
}

/// Python: bot/memory/narrow.py:2652 `_same_step(asked, words)`
pub fn _same_step(asked: &[String], words: &[String]) -> bool {
    crate::unported!("bot.memory.narrow:_same_step")
}

/// Python: bot/memory/narrow.py:2672 `_step_time(sentence, question, figure)`
pub fn _step_time(sentence: &str, question: &str, figure: &Value) -> Option<String> {
    crate::unported!("bot.memory.narrow:_step_time")
}

/// Python: bot/memory/narrow.py:2764 `_aside_held(text)`
pub fn _aside_held(text: &str) -> Value {
    crate::unported!("bot.memory.narrow:_aside_held")
}

/// Python: bot/memory/narrow.py:2775 `_aside_back(piece, kept)`
pub fn _aside_back(piece: &str, kept: &[String]) -> String {
    crate::unported!("bot.memory.narrow:_aside_back")
}

/// Python: bot/memory/narrow.py:2779 `clauses(text, pattern)`
pub fn clauses(text: &str, pattern: &Value) -> Vec<String> {
    crate::unported!("bot.memory.narrow:clauses")
}

/// Python: bot/memory/narrow.py:2789 `clauses_bare(text, pattern)`
pub fn clauses_bare(text: &str, pattern: &Value) -> Value {
    crate::unported!("bot.memory.narrow:clauses_bare")
}

/// Python: bot/memory/narrow.py:2829 `_clause(text, question)`
pub fn _clause(text: &str, question: &str) {
    crate::unported!("bot.memory.narrow:_clause")
}

/// Python: bot/memory/narrow.py:2845 `_clause_with(text, word)`
pub fn _clause_with(text: &str, word: &str) {
    crate::unported!("bot.memory.narrow:_clause_with")
}

/// Python: bot/memory/narrow.py:2852 `_clause_holding(text, span)` (never called on the trace corpus)
pub fn _clause_holding(text: &str, span: &Value) -> Value {
    crate::unported!("bot.memory.narrow:_clause_holding")
}

/// Python: bot/memory/narrow.py:2866 `words_of(text)`
pub fn words_of(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.narrow:words_of")
}

/// Python: bot/memory/narrow.py:2880 `flat(text)`
pub fn flat(text: &str) -> String {
    crate::unported!("bot.memory.narrow:flat")
}

/// Python: bot/memory/narrow.py:2884 `tidy(text)`
pub fn tidy(text: &str) -> String {
    crate::unported!("bot.memory.narrow:tidy")
}

