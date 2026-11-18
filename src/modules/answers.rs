//! Port of `bot/modules/answers.py`.
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

/// Python: bot/modules/answers.py:101 `_the_new_part(reply, memory, keep=3)`
pub fn _the_new_part(reply: &str, memory: &Memory, keep: i64) {
    crate::unported!("bot.modules.answers:_the_new_part")
}

/// Python: bot/modules/answers.py:137 `_without_the_verdict(text)`
pub fn _without_the_verdict(text: &str) -> String {
    crate::unported!("bot.modules.answers:_without_the_verdict")
}

/// Python: bot/modules/answers.py:155 `_card_openings()` (never called on the trace corpus)
pub fn _card_openings() -> Value {
    crate::unported!("bot.modules.answers:_card_openings")
}

/// Python: bot/modules/answers.py:174 `_without_a_lead(text)`
pub fn _without_a_lead(text: &str) -> String {
    crate::unported!("bot.modules.answers:_without_a_lead")
}

/// Python: bot/modules/answers.py:195 `_opening(text, words=8)`
pub fn _opening(text: &str, words: i64) -> String {
    crate::unported!("bot.modules.answers:_opening")
}

/// Python: bot/modules/answers.py:211 `_most_of_it_said(reply, memory, keep=3, least=40)`
pub fn _most_of_it_said(reply: &str, memory: &Memory, keep: i64, least: i64) -> bool {
    crate::unported!("bot.modules.answers:_most_of_it_said")
}

/// Python: bot/modules/answers.py:232 `_said_before(reply, memory, keep=3)`
pub fn _said_before(reply: &str, memory: &Memory, keep: i64) -> bool {
    crate::unported!("bot.modules.answers:_said_before")
}

/// Python: bot/modules/answers.py:441 `_in_short(reply)`
pub fn _in_short(reply: &str) -> String {
    crate::unported!("bot.modules.answers:_in_short")
}

/// Python: bot/modules/answers.py:460 `_same_as_the_last_answer(text, memory)` (never called on the trace corpus)
pub fn _same_as_the_last_answer(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_same_as_the_last_answer")
}

/// Python: bot/modules/answers.py:482 `_the_same_answer_in_other_words(text, memory)` (never called on the trace corpus)
pub fn _the_same_answer_in_other_words(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_same_answer_in_other_words")
}

/// Python: bot/modules/answers.py:498 `_flat(text)`
pub fn _flat(text: &str) -> String {
    crate::unported!("bot.modules.answers:_flat")
}

/// Python: bot/modules/answers.py:528 `_says_something(clause)`
pub fn _says_something(clause: &str) -> bool {
    crate::unported!("bot.modules.answers:_says_something")
}

/// Python: bot/modules/answers.py:533 `_the_part_in_the_answer(answer, detail)`
pub fn _the_part_in_the_answer(answer: &str, detail: &str) -> String {
    crate::unported!("bot.modules.answers:_the_part_in_the_answer")
}

/// Python: bot/modules/answers.py:579 `_names_it(word, last)`
pub fn _names_it(word: &str, last: &str) -> bool {
    crate::unported!("bot.modules.answers:_names_it")
}

/// Python: bot/modules/answers.py:612 `_a_part_named_in_the_line(text, memory, answer)`
pub fn _a_part_named_in_the_line(text: &str, memory: &Memory, answer: &str) -> String {
    crate::unported!("bot.modules.answers:_a_part_named_in_the_line")
}

/// Python: bot/modules/answers.py:656 `_short_form_of_the_last(answer, memory)`
pub fn _short_form_of_the_last(answer: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_short_form_of_the_last")
}

/// Python: bot/modules/answers.py:667 `_answer_said(plan, memory)`
pub fn _answer_said(plan: &str, memory: &Memory) -> i64 {
    crate::unported!("bot.modules.answers:_answer_said")
}

/// Python: bot/modules/answers.py:701 `_carries(text)`
pub fn _carries(text: &str) -> Vec<String> {
    crate::unported!("bot.modules.answers:_carries")
}

/// Python: bot/modules/answers.py:707 `_the_sentence_asked(reply, text)`
pub fn _the_sentence_asked(reply: &str, text: &str) -> String {
    crate::unported!("bot.modules.answers:_the_sentence_asked")
}

/// Python: bot/modules/answers.py:741 `_the_sentence_naming(reply, text, memory=None)`
pub fn _the_sentence_naming(reply: &str, text: &str, memory: &Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:_the_sentence_naming")
}

/// Python: bot/modules/answers.py:771 `_repeat_short(reply, text, memory=None)`
pub fn _repeat_short(reply: &str, text: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_repeat_short")
}

/// Python: bot/modules/answers.py:820 `_first_step(reply)` (never called on the trace corpus)
pub fn _first_step(reply: &str) -> Value {
    crate::unported!("bot.modules.answers:_first_step")
}

/// Python: bot/modules/answers.py:840 `_figure_missing(reply, text)`
pub fn _figure_missing(reply: &str, text: &str) {
    crate::unported!("bot.modules.answers:_figure_missing")
}

/// Python: bot/modules/answers.py:863 `_figures_changed(reply, said)`
pub fn _figures_changed(reply: &str, said: &str) -> bool {
    crate::unported!("bot.modules.answers:_figures_changed")
}

/// Python: bot/modules/answers.py:870 `_same_answer(reply, said)`
pub fn _same_answer(reply: &str, said: &str) -> bool {
    crate::unported!("bot.modules.answers:_same_answer")
}

/// Python: bot/modules/answers.py:905 `_asks_of_another_subject(text, understood)`
pub fn _asks_of_another_subject(text: &str, understood: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_asks_of_another_subject")
}

/// Python: bot/modules/answers.py:921 `_asked_again(text, reply, memory)`
pub fn _asked_again(text: &str, reply: &str, memory: &Memory) -> i64 {
    crate::unported!("bot.modules.answers:_asked_again")
}

/// Python: bot/modules/answers.py:946 `_ask_verb(text)`
pub fn _ask_verb(text: &str) -> String {
    crate::unported!("bot.modules.answers:_ask_verb")
}

/// Python: bot/modules/answers.py:952 `_the_outcome_owed(item, text)`
pub fn _the_outcome_owed(item: &'static Entry, text: &str) {
    crate::unported!("bot.modules.answers:_the_outcome_owed")
}

/// Python: bot/modules/answers.py:969 `_said_before_now(text, memory)`
pub fn _said_before_now(text: &str, memory: &Memory) -> Vec<String> {
    crate::unported!("bot.modules.answers:_said_before_now")
}

/// Python: bot/modules/answers.py:987 `_a_fact_about_the_person(memory)`
pub fn _a_fact_about_the_person(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_a_fact_about_the_person")
}

/// Python: bot/modules/answers.py:1003 `_the_detail_named(text, memory=None, with_quality=False)`
pub fn _the_detail_named(text: &str, memory: &Memory, with_quality: bool) -> String {
    crate::unported!("bot.modules.answers:_the_detail_named")
}

/// Python: bot/modules/answers.py:1020 `_a_restatement_of_the_answer(memory)`
pub fn _a_restatement_of_the_answer(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_a_restatement_of_the_answer")
}

/// Python: bot/modules/answers.py:1033 `_all_of_it_in_the_answer(named, memory)`
pub fn _all_of_it_in_the_answer(named: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_all_of_it_in_the_answer")
}

/// Python: bot/modules/answers.py:1051 `_the_clause_s_verb(raw, index)`
pub fn _the_clause_s_verb(raw: &[String], index: i64) -> bool {
    crate::unported!("bot.modules.answers:_the_clause_s_verb")
}

/// Python: bot/modules/answers.py:1061 `_the_part_named(text, memory=None, with_quality=False)`
pub fn _the_part_named(text: &str, memory: &Memory, with_quality: bool) -> String {
    crate::unported!("bot.modules.answers:_the_part_named")
}

/// Python: bot/modules/answers.py:1159 `_only_a_quality_name(text, memory=None)` (never called on the trace corpus)
pub fn _only_a_quality_name(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_only_a_quality_name")
}

/// Python: bot/modules/answers.py:1181 `_named_whole(raw, index)`
pub fn _named_whole(raw: &[String], index: i64) -> String {
    crate::unported!("bot.modules.answers:_named_whole")
}

/// Python: bot/modules/answers.py:1222 `_their_own_state(text)`
pub fn _their_own_state(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_their_own_state")
}

/// Python: bot/modules/answers.py:1237 `_named_in(part, detail)`
pub fn _named_in(part: &str, detail: &str) -> bool {
    crate::unported!("bot.modules.answers:_named_in")
}

/// Python: bot/modules/answers.py:1259 `similar(word, other)`
pub fn similar(word: &str, other: &str) -> bool {
    crate::unported!("bot.modules.answers:similar")
}

/// Python: bot/modules/answers.py:1272 `_entry_words(memory)`
pub fn _entry_words(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.modules.answers:_entry_words")
}

/// Python: bot/modules/answers.py:1292 `_what_they_typed(text, memory)`
pub fn _what_they_typed(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_what_they_typed")
}

/// Python: bot/modules/answers.py:1312 `_covered(text, reply, memory=None, understood=None)`
pub fn _covered(text: &str, reply: &str, memory: &Memory, understood: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_covered")
}

/// Python: bot/modules/answers.py:1364 `_a_question_about_another_thing(text, reply, understood, memory, own=None, item=None)`
pub fn _a_question_about_another_thing(text: &str, reply: &str, understood: &Understood, memory: &Memory, own: &str, item: &'static Entry) -> bool {
    crate::unported!("bot.modules.answers:_a_question_about_another_thing")
}

/// Python: bot/modules/answers.py:1444 `_small_letter(body)`
pub fn _small_letter(body: &str) -> String {
    crate::unported!("bot.modules.answers:_small_letter")
}

/// Python: bot/modules/answers.py:1460 `_times_the_answer_was_said(reply, memory)`
pub fn _times_the_answer_was_said(reply: &str, memory: &Memory) -> i64 {
    crate::unported!("bot.modules.answers:_times_the_answer_was_said")
}

/// Python: bot/modules/answers.py:1470 `_with_detail(detail, part, times, text='', memory=None, card='')`
pub fn _with_detail(detail: &str, part: &str, times: i64, text: &str, memory: &Memory, card: &str) -> String {
    crate::unported!("bot.modules.answers:_with_detail")
}

/// Python: bot/modules/answers.py:1506 `_a_yes_no_ask(memory)`
pub fn _a_yes_no_ask(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_a_yes_no_ask")
}

/// Python: bot/modules/answers.py:1532 `_asks_for_more(text)`
pub fn _asks_for_more(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_asks_for_more")
}

/// Python: bot/modules/answers.py:1537 `_nothing_more_said_again(reply, text, memory)`
pub fn _nothing_more_said_again(reply: &str, text: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_nothing_more_said_again")
}

/// Python: bot/modules/answers.py:1555 `_the_detail_said_again(reply, text, memory)`
pub fn _the_detail_said_again(reply: &str, text: &str, memory: &Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:_the_detail_said_again")
}

/// Python: bot/modules/answers.py:1591 `_detail_lead_in(text, part, memory)` (never called on the trace corpus)
pub fn _detail_lead_in(text: &str, part: &str, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_detail_lead_in")
}

/// Python: bot/modules/answers.py:1610 `_the_answer_on_screen(memory)` (never called on the trace corpus)
pub fn _the_answer_on_screen(memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_the_answer_on_screen")
}

/// Python: bot/modules/answers.py:1619 `_the_part_said_again(part, text, understood, memory)`
pub fn _the_part_said_again(part: &str, text: &str, understood: &Understood, memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_the_part_said_again")
}

/// Python: bot/modules/answers.py:1633 `_already_said(reply, memory, keep=3)`
pub fn _already_said(reply: &str, memory: &Memory, keep: i64) -> bool {
    crate::unported!("bot.modules.answers:_already_said")
}

/// Python: bot/modules/answers.py:1654 `_the_same_sentences(said, mine)`
pub fn _the_same_sentences(said: &str, mine: &str) -> bool {
    crate::unported!("bot.modules.answers:_the_same_sentences")
}

/// Python: bot/modules/answers.py:1694 `_asks_who_the_bot_is(understood, text)`
pub fn _asks_who_the_bot_is(understood: Option<&Understood>, text: &str) -> bool {
    crate::unported!("bot.modules.answers:_asks_who_the_bot_is")
}

/// Python: bot/modules/answers.py:1739 `_answers_a_length(item, reply)`
pub fn _answers_a_length(item: &'static Entry, reply: &str) -> bool {
    crate::unported!("bot.modules.answers:_answers_a_length")
}

/// Python: bot/modules/answers.py:1755 `_written_ask_with_an_answer(text, memory)`
pub fn _written_ask_with_an_answer(text: &str, memory: &Value) {
    crate::unported!("bot.modules.answers:_written_ask_with_an_answer")
}

/// Python: bot/modules/answers.py:1779 `_the_article(subject, memory)`
pub fn _the_article(subject: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_the_article")
}

/// Python: bot/modules/answers.py:1809 `_with_subject(text, memory, about=None)`
pub fn _with_subject(text: &str, memory: &Value, about: &Value) -> String {
    crate::unported!("bot.modules.answers:_with_subject")
}

/// Python: bot/modules/answers.py:1843 `_about_subject(text, memory, about=None)`
pub fn _about_subject(text: &str, memory: &Memory, about: &Value) -> String {
    crate::unported!("bot.modules.answers:_about_subject")
}

/// Python: bot/modules/answers.py:1917 `_keyed_on_the_person(item, who)` (never called on the trace corpus)
pub fn _keyed_on_the_person(item: &'static Entry, who: &str) -> bool {
    crate::unported!("bot.modules.answers:_keyed_on_the_person")
}

/// Python: bot/modules/answers.py:1934 `_something_for(text)`
pub fn _something_for(text: &str) -> String {
    crate::unported!("bot.modules.answers:_something_for")
}

/// Python: bot/modules/answers.py:1986 `_keeps_a_plan(memory)` (never called on the trace corpus)
pub fn _keeps_a_plan(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_keeps_a_plan")
}

/// Python: bot/modules/answers.py:2041 `_with_subject_tail(text, memory, understood=None)` (never called on the trace corpus)
pub fn _with_subject_tail(text: &str, memory: &Memory, understood: &Understood) -> Value {
    crate::unported!("bot.modules.answers:_with_subject_tail")
}

/// Python: bot/modules/answers.py:2049 `_subject_tails(text, memory, understood=None)`
pub fn _subject_tails(text: &str, memory: &Memory, understood: &Understood) -> Vec<String> {
    crate::unported!("bot.modules.answers:_subject_tails")
}

/// Python: bot/modules/answers.py:2121 `_bare_subject(subject)`
pub fn _bare_subject(subject: &str) -> String {
    crate::unported!("bot.modules.answers:_bare_subject")
}

/// Python: bot/modules/answers.py:2154 `_the_month(text)`
pub fn _the_month(text: &str) -> String {
    crate::unported!("bot.modules.answers:_the_month")
}

/// Python: bot/modules/answers.py:2165 `_a_trip(memory)` (never called on the trace corpus)
pub fn _a_trip(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_a_trip")
}

/// Python: bot/modules/answers.py:2193 `_a_clause(subject)`
pub fn _a_clause(subject: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_clause")
}

/// Python: bot/modules/answers.py:2198 `_choosing(text, memory)`
pub fn _choosing(text: &str, memory: &Value) {
    crate::unported!("bot.modules.answers:_choosing")
}

/// Python: bot/modules/answers.py:2285 `_only_a_size_of(item, text)`
pub fn _only_a_size_of(item: &'static Entry, text: &str) -> bool {
    crate::unported!("bot.modules.answers:_only_a_size_of")
}

/// Python: bot/modules/answers.py:2310 `_choice_words(text, memory)` (never called on the trace corpus)
pub fn _choice_words(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_choice_words")
}

/// Python: bot/modules/answers.py:2343 `_qualifier_rewrites(text, memory)`
pub fn _qualifier_rewrites(text: &str, memory: &Value) -> Vec<String> {
    crate::unported!("bot.modules.answers:_qualifier_rewrites")
}

/// Python: bot/modules/answers.py:2433 `_a_lack_not_a_refusal(flat, end)` (never called on the trace corpus)
pub fn _a_lack_not_a_refusal(flat: &Value, end: i64) -> bool {
    crate::unported!("bot.modules.answers:_a_lack_not_a_refusal")
}

/// Python: bot/modules/answers.py:2442 `_refused(text)`
pub fn _refused(text: &str) -> String {
    crate::unported!("bot.modules.answers:_refused")
}

/// Python: bot/modules/answers.py:2476 `_says_it_leaves_it_out(reply, refused)` (never called on the trace corpus)
pub fn _says_it_leaves_it_out(reply: &str, refused: &Value) -> bool {
    crate::unported!("bot.modules.answers:_says_it_leaves_it_out")
}

/// Python: bot/modules/answers.py:2504 `_hands_back_the_refusal(item, refused)`
pub fn _hands_back_the_refusal(item: &'static Entry, refused: &str) -> bool {
    crate::unported!("bot.modules.answers:_hands_back_the_refusal")
}

/// Python: bot/modules/answers.py:2528 `_about_a_lack(item)` (never called on the trace corpus)
pub fn _about_a_lack(item: &'static Entry) -> bool {
    crate::unported!("bot.modules.answers:_about_a_lack")
}

/// Python: bot/modules/answers.py:2538 `_key_words(item)`
pub fn _key_words(item: &'static Entry) -> Vec<String> {
    crate::unported!("bot.modules.answers:_key_words")
}

/// Python: bot/modules/answers.py:2573 `_names_a_thing(understood)`
pub fn _names_a_thing(understood: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_names_a_thing")
}

/// Python: bot/modules/answers.py:2582 `_own_thing(understood)`
pub fn _own_thing(understood: &Understood) -> String {
    crate::unported!("bot.modules.answers:_own_thing")
}

/// Python: bot/modules/answers.py:2637 `_thing_named_in(understood, span)`
pub fn _thing_named_in(understood: &Understood, span: &[String]) -> String {
    crate::unported!("bot.modules.answers:_thing_named_in")
}

/// Python: bot/modules/answers.py:2654 `_new_thing(own, answer)`
pub fn _new_thing(own: &str, answer: &str) -> bool {
    crate::unported!("bot.modules.answers:_new_thing")
}

/// Python: bot/modules/answers.py:2666 `_the_run_said(memory, extra='')`
pub fn _the_run_said(memory: &Memory, extra: &str) -> String {
    crate::unported!("bot.modules.answers:_the_run_said")
}

/// Python: bot/modules/answers.py:2693 `_head_thing(subject)`
pub fn _head_thing(subject: &str) -> String {
    crate::unported!("bot.modules.answers:_head_thing")
}

/// Python: bot/modules/answers.py:2708 `_the_same_kind_of_thing(item, memory, text='')`
pub fn _the_same_kind_of_thing(item: &'static Entry, memory: &Memory, text: &str) -> bool {
    crate::unported!("bot.modules.answers:_the_same_kind_of_thing")
}

/// Python: bot/modules/answers.py:2737 `_reaches_another_subject(text, item, memory)`
pub fn _reaches_another_subject(text: &str, item: &'static Entry, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_reaches_another_subject")
}

/// Python: bot/modules/answers.py:2851 `_off_the_topic(understood, memory)`
pub fn _off_the_topic(understood: &Understood, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_off_the_topic")
}

/// Python: bot/modules/answers.py:2903 `_from_another_domain(item, off_topic)` (never called on the trace corpus)
pub fn _from_another_domain(item: &'static Entry, off_topic: &Value) -> bool {
    crate::unported!("bot.modules.answers:_from_another_domain")
}

/// Python: bot/modules/answers.py:2911 `_names_a_topic_word(item, wanted)` (never called on the trace corpus)
pub fn _names_a_topic_word(item: &'static Entry, wanted: &Value) -> bool {
    crate::unported!("bot.modules.answers:_names_a_topic_word")
}

/// Python: bot/modules/answers.py:2951 `_topic_words(line, memory)`
pub fn _topic_words(line: &str, memory: &Memory) -> Vec<String> {
    crate::unported!("bot.modules.answers:_topic_words")
}

/// Python: bot/modules/answers.py:2960 `_lent_words(reading, memory)`
pub fn _lent_words(reading: &Understood, memory: &Memory) -> Vec<String> {
    crate::unported!("bot.modules.answers:_lent_words")
}

/// Python: bot/modules/answers.py:2978 `_same_word(word, said)`
pub fn _same_word(word: &str, said: &str) -> bool {
    crate::unported!("bot.modules.answers:_same_word")
}

/// Python: bot/modules/answers.py:2988 `_writes_its_own_reply(item)`
pub fn _writes_its_own_reply(item: &'static Entry) -> bool {
    crate::unported!("bot.modules.answers:_writes_its_own_reply")
}

/// Python: bot/modules/answers.py:3009 `_leans_on(reading, item, memory=None)`
pub fn _leans_on(reading: &Understood, item: &'static Entry, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_leans_on")
}

/// Python: bot/modules/answers.py:3118 `_is_about(item, subject)`
pub fn _is_about(item: &'static Entry, subject: &str) -> bool {
    crate::unported!("bot.modules.answers:_is_about")
}

/// Python: bot/modules/answers.py:3174 `_only_a_property(words)`
pub fn _only_a_property(words: &[String]) -> bool {
    crate::unported!("bot.modules.answers:_only_a_property")
}

/// Python: bot/modules/answers.py:3197 `_a_property_word_names_a_thing(text)`
pub fn _a_property_word_names_a_thing(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_property_word_names_a_thing")
}

/// Python: bot/modules/answers.py:3215 `_a_money_question(own, keyed)`
pub fn _a_money_question(own: &[String], keyed: &[String]) -> bool {
    crate::unported!("bot.modules.answers:_a_money_question")
}

/// Python: bot/modules/answers.py:3224 `_remember_answer(memory, item, reply, keys=())`
pub fn _remember_answer(memory: &mut Memory, item: Option<&'static Entry>, reply: &str, keys: &[String]) {
    crate::unported!("bot.modules.answers:_remember_answer")
}

/// Python: bot/modules/answers.py:3242 `_last_answer(memory)`
pub fn _last_answer(memory: &Memory) {
    crate::unported!("bot.modules.answers:_last_answer")
}

/// Python: bot/modules/answers.py:3274 `_asks_part(text)`
pub fn _asks_part(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_asks_part")
}

/// Python: bot/modules/answers.py:3323 `_the_sentence_about(text, memory)`
pub fn _the_sentence_about(text: &str, memory: &Value) {
    crate::unported!("bot.modules.answers:_the_sentence_about")
}

/// Python: bot/modules/answers.py:3426 `_a_stock_acknowledgement(turns, back)` (never called on the trace corpus)
pub fn _a_stock_acknowledgement(turns: &[Turn], back: &Value) -> bool {
    crate::unported!("bot.modules.answers:_a_stock_acknowledgement")
}

/// Python: bot/modules/answers.py:3452 `_the_person_to_call(text, memory)`
pub fn _the_person_to_call(text: &str, memory: &Value) {
    crate::unported!("bot.modules.answers:_the_person_to_call")
}

/// Python: bot/modules/answers.py:3530 `_the_clause_that_says_who(part)` (never called on the trace corpus)
pub fn _the_clause_that_says_who(part: &str) -> Value {
    crate::unported!("bot.modules.answers:_the_clause_that_says_who")
}

/// Python: bot/modules/answers.py:3600 `_the_clause_that_picks(sentence)` (never called on the trace corpus)
pub fn _the_clause_that_picks(sentence: &str) -> Value {
    crate::unported!("bot.modules.answers:_the_clause_that_picks")
}

/// Python: bot/modules/answers.py:3644 `_a_list_of_things(tail)` (never called on the trace corpus)
pub fn _a_list_of_things(tail: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_list_of_things")
}

/// Python: bot/modules/answers.py:3659 `_the_thing_that_picks(sentence)` (never called on the trace corpus)
pub fn _the_thing_that_picks(sentence: &str) -> Value {
    crate::unported!("bot.modules.answers:_the_thing_that_picks")
}

/// Python: bot/modules/answers.py:3729 `_a_price_and_nothing_else(span)` (never called on the trace corpus)
pub fn _a_price_and_nothing_else(span: &Value) -> bool {
    crate::unported!("bot.modules.answers:_a_price_and_nothing_else")
}

/// Python: bot/modules/answers.py:3750 `_the_composer_has_options(memory)` (never called on the trace corpus)
pub fn _the_composer_has_options(memory: &Memory) -> Option<bool> {
    crate::unported!("bot.modules.answers:_the_composer_has_options")
}

/// Python: bot/modules/answers.py:3770 `_the_option_that_decides(said, reply)` (never called on the trace corpus)
pub fn _the_option_that_decides(said: &str, reply: &str) -> Value {
    crate::unported!("bot.modules.answers:_the_option_that_decides")
}

/// Python: bot/modules/answers.py:3781 `_the_answer_to_a_choice_behind(memory)` (never called on the trace corpus)
pub fn _the_answer_to_a_choice_behind(memory: &Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:_the_answer_to_a_choice_behind")
}

/// Python: bot/modules/answers.py:3807 `_the_one_to_pick(text, memory)`
pub fn _the_one_to_pick(text: &str, memory: &Value) {
    crate::unported!("bot.modules.answers:_the_one_to_pick")
}

/// Python: bot/modules/answers.py:3900 `_one_total(rest, whole_sentence=False)` (never called on the trace corpus)
pub fn _one_total(rest: &str, whole_sentence: &Value) -> bool {
    crate::unported!("bot.modules.answers:_one_total")
}

/// Python: bot/modules/answers.py:3924 `_the_dish_time(understood, memory)`
pub fn _the_dish_time(understood: &Understood, memory: &Memory) {
    crate::unported!("bot.modules.answers:_the_dish_time")
}

/// Python: bot/modules/answers.py:4006 `_the_unit_last_said(memory, reply)` (never called on the trace corpus)
pub fn _the_unit_last_said(memory: &Memory, reply: &str) -> Option<String> {
    crate::unported!("bot.modules.answers:_the_unit_last_said")
}

/// Python: bot/modules/answers.py:4026 `_the_figure_they_meant(text, memory, reply)`
pub fn _the_figure_they_meant(text: &str, memory: &Memory, reply: &str) {
    crate::unported!("bot.modules.answers:_the_figure_they_meant")
}

/// Python: bot/modules/answers.py:4050 `_amount_asked(text, memory)`
pub fn _amount_asked(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.answers:_amount_asked")
}

/// Python: bot/modules/answers.py:4133 `_the_amount_they_named_again(text, memory)`
pub fn _the_amount_they_named_again(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.answers:_the_amount_they_named_again")
}

/// Python: bot/modules/answers.py:4195 `_the_machine_follow_up(text, reading, memory)`
pub fn _the_machine_follow_up(text: &str, reading: &Understood, memory: &Memory) {
    crate::unported!("bot.modules.answers:_the_machine_follow_up")
}

/// Python: bot/modules/answers.py:4247 `_part_asked(text, memory)`
pub fn _part_asked(text: &str, memory: &Value) {
    crate::unported!("bot.modules.answers:_part_asked")
}

/// Python: bot/modules/answers.py:4541 `_amount_for_the_name(clause, named)`
pub fn _amount_for_the_name(clause: &str, named: &[String]) -> bool {
    crate::unported!("bot.modules.answers:_amount_for_the_name")
}

/// Python: bot/modules/answers.py:4596 `_sentence_naming_the_part(reply, text)`
pub fn _sentence_naming_the_part(reply: &str, text: &str) -> Option<String> {
    crate::unported!("bot.modules.answers:_sentence_naming_the_part")
}

/// Python: bot/modules/answers.py:4671 `_names_no_cause(head)`
pub fn _names_no_cause(head: &str) -> bool {
    crate::unported!("bot.modules.answers:_names_no_cause")
}

/// Python: bot/modules/answers.py:4679 `_cause_part(part)`
pub fn _cause_part(part: &str) -> String {
    crate::unported!("bot.modules.answers:_cause_part")
}

/// Python: bot/modules/answers.py:4700 `_reason_again(reason, last)`
pub fn _reason_again(reason: &str, last: &str) -> String {
    crate::unported!("bot.modules.answers:_reason_again")
}

/// Python: bot/modules/answers.py:4762 `_asked_figure(text, reply, other_reply)`
pub fn _asked_figure(text: &str, reply: &str, other_reply: Option<&str>) -> String {
    crate::unported!("bot.modules.answers:_asked_figure")
}

/// Python: bot/modules/answers.py:4787 `_the_figure(text, reply)`
pub fn _the_figure(text: &str, reply: &str) -> String {
    crate::unported!("bot.modules.answers:_the_figure")
}

/// Python: bot/modules/answers.py:4792 `_sentence_with(reply, part)` (never called on the trace corpus)
pub fn _sentence_with(reply: &str, part: &str) -> String {
    crate::unported!("bot.modules.answers:_sentence_with")
}

/// Python: bot/modules/answers.py:4805 `_part_not_repeated(part, whole, memory)`
pub fn _part_not_repeated(part: Option<&str>, whole: &str, memory: &Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:_part_not_repeated")
}

/// Python: bot/modules/answers.py:4856 `_names_a_property(text)`
pub fn _names_a_property(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_names_a_property")
}

/// Python: bot/modules/answers.py:4869 `_with_bare_subject(text, memory)`
pub fn _with_bare_subject(text: &str, memory: &Value) -> String {
    crate::unported!("bot.modules.answers:_with_bare_subject")
}

/// Python: bot/modules/answers.py:4883 `_the_things_to_try(understood, memory)` (never called on the trace corpus)
pub fn _the_things_to_try(understood: &Understood, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_the_things_to_try")
}

/// Python: bot/modules/answers.py:4916 `_the_answer_from_thing(text, thing, memory)` (never called on the trace corpus)
pub fn _the_answer_from_thing(text: &str, thing: &str, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_the_answer_from_thing")
}

/// Python: bot/modules/answers.py:4932 `_with_the_thing(text, thing)` (never called on the trace corpus)
pub fn _with_the_thing(text: &str, thing: &str) -> Value {
    crate::unported!("bot.modules.answers:_with_the_thing")
}

/// Python: bot/modules/answers.py:4946 `_care_holds(text, memory)`
pub fn _care_holds(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_care_holds")
}

/// Python: bot/modules/answers.py:4962 `_named_by(item, subject)`
pub fn _named_by(item: &'static Entry, subject: &str) -> bool {
    crate::unported!("bot.modules.answers:_named_by")
}

/// Python: bot/modules/answers.py:5005 `_the_answer_was_about_the_subject(memory)`
pub fn _the_answer_was_about_the_subject(memory: &Value) -> bool {
    crate::unported!("bot.modules.answers:_the_answer_was_about_the_subject")
}

/// Python: bot/modules/answers.py:5053 `_minutes_in(text)`
pub fn _minutes_in(text: Option<&str>) -> Value {
    crate::unported!("bot.modules.answers:_minutes_in")
}

/// Python: bot/modules/answers.py:5070 `_a_figure_apart(mine, theirs)`
pub fn _a_figure_apart(mine: &Memory, theirs: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_figure_apart")
}

/// Python: bot/modules/answers.py:5086 `_only_a_figure(part)`
pub fn _only_a_figure(part: &str) -> bool {
    crate::unported!("bot.modules.answers:_only_a_figure")
}

/// Python: bot/modules/answers.py:5097 `_figures(text)`
pub fn _figures(text: &str) -> Vec<String> {
    crate::unported!("bot.modules.answers:_figures")
}

/// Python: bot/modules/answers.py:5119 `_name_words()`
pub fn _name_words() -> Vec<String> {
    crate::unported!("bot.modules.answers:_name_words")
}

/// Python: bot/modules/answers.py:5149 `_filler(word)`
pub fn _filler(word: &str) -> bool {
    crate::unported!("bot.modules.answers:_filler")
}

/// Python: bot/modules/answers.py:5161 `_left_out(item, text)`
pub fn _left_out(item: &'static Entry, text: &str) -> bool {
    crate::unported!("bot.modules.answers:_left_out")
}

/// Python: bot/modules/answers.py:5193 `_keyed_on_the_line(item, text)`
pub fn _keyed_on_the_line(item: &'static Entry, text: &str) -> bool {
    crate::unported!("bot.modules.answers:_keyed_on_the_line")
}

/// Python: bot/modules/answers.py:5209 `_shares_a_word(item, subject)` (never called on the trace corpus)
pub fn _shares_a_word(item: &'static Entry, subject: &str) -> bool {
    crate::unported!("bot.modules.answers:_shares_a_word")
}

/// Python: bot/modules/answers.py:5225 `_answers_the_figure(text, reply)`
pub fn _answers_the_figure(text: &str, reply: &str) -> bool {
    crate::unported!("bot.modules.answers:_answers_the_figure")
}

/// Python: bot/modules/answers.py:5281 `_asks_how_to_say_it(text)`
pub fn _asks_how_to_say_it(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_asks_how_to_say_it")
}

/// Python: bot/modules/answers.py:5286 `_in_the_subjects_field(item, subject)`
pub fn _in_the_subjects_field(item: &'static Entry, subject: &str) -> bool {
    crate::unported!("bot.modules.answers:_in_the_subjects_field")
}

/// Python: bot/modules/answers.py:5310 `_own_thing_words(text, memory)`
pub fn _own_thing_words(text: &str, memory: &Value) -> Vec<String> {
    crate::unported!("bot.modules.answers:_own_thing_words")
}

/// Python: bot/modules/answers.py:5332 `_about_the_talk(item, text, memory, reading)`
pub fn _about_the_talk(item: &'static Entry, text: &str, memory: &Memory, reading: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_about_the_talk")
}

/// Python: bot/modules/answers.py:5412 `_agrees_with_its_own_field(item, reading)`
pub fn _agrees_with_its_own_field(item: &'static Entry, reading: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_agrees_with_its_own_field")
}

/// Python: bot/modules/answers.py:5423 `_the_field_on_the_screen(item, text, reading, memory)`
pub fn _the_field_on_the_screen(item: &'static Entry, text: &str, reading: &Understood, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_field_on_the_screen")
}

/// Python: bot/modules/answers.py:5455 `_last_turn_answered_from_the_field(memory)`
pub fn _last_turn_answered_from_the_field(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_last_turn_answered_from_the_field")
}

/// Python: bot/modules/answers.py:5476 `_says_a_subject_phrase(item, text)`
pub fn _says_a_subject_phrase(item: &'static Entry, text: &str) -> bool {
    crate::unported!("bot.modules.answers:_says_a_subject_phrase")
}

/// Python: bot/modules/answers.py:5524 `_names_the_entry(text, item)`
pub fn _names_the_entry(text: &str, item: &'static Entry) -> bool {
    crate::unported!("bot.modules.answers:_names_the_entry")
}

/// Python: bot/modules/answers.py:5564 `_a_definition_of_the_entry(text, item)`
pub fn _a_definition_of_the_entry(text: &str, item: &'static Entry) -> bool {
    crate::unported!("bot.modules.answers:_a_definition_of_the_entry")
}

/// Python: bot/modules/answers.py:5611 `_says_the_ask_whole(text, item)`
pub fn _says_the_ask_whole(text: &str, item: &'static Entry) -> bool {
    crate::unported!("bot.modules.answers:_says_the_ask_whole")
}

/// Python: bot/modules/answers.py:5633 `_a_why_line_that_names(item, reading)`
pub fn _a_why_line_that_names(item: &'static Entry, reading: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_a_why_line_that_names")
}

/// Python: bot/modules/answers.py:5640 `_names_the_thing_apart_from_the_talk(item, text, subject, reading)`
pub fn _names_the_thing_apart_from_the_talk(item: &'static Entry, text: &str, subject: &str, reading: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_names_the_thing_apart_from_the_talk")
}

/// Python: bot/modules/answers.py:5678 `_a_choice_with_one_thing_left_out(text)`
pub fn _a_choice_with_one_thing_left_out(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_choice_with_one_thing_left_out")
}

/// Python: bot/modules/answers.py:5735 `_describes_the_thing(word)`
pub fn _describes_the_thing(word: &str) -> bool {
    crate::unported!("bot.modules.answers:_describes_the_thing")
}

/// Python: bot/modules/answers.py:5750 `_said_in_the_talk(memory)`
pub fn _said_in_the_talk(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.modules.answers:_said_in_the_talk")
}

/// Python: bot/modules/answers.py:5761 `_already_in_the_talk(word, said)`
pub fn _already_in_the_talk(word: &str, said: &[String]) -> bool {
    crate::unported!("bot.modules.answers:_already_in_the_talk")
}

/// Python: bot/modules/answers.py:5772 `_names_its_own_thing(text, memory=None)`
pub fn _names_its_own_thing(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_names_its_own_thing")
}

/// Python: bot/modules/answers.py:5816 `_the_topic_rules_it_out(item, memory)` (never called on the trace corpus)
pub fn _the_topic_rules_it_out(item: &'static Entry, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_topic_rules_it_out")
}

/// Python: bot/modules/answers.py:5829 `_in_context(text, memory, reject=None)`
pub fn _in_context(text: &str, memory: &Value, reject: &Value) {
    crate::unported!("bot.modules.answers:_in_context")
}

/// Python: bot/modules/answers.py:6005 `_from_rewrite(fuller, text, memory, reject, added='', domain=None)` (never called on the trace corpus)
pub fn _from_rewrite(fuller: &Value, text: &str, memory: &Memory, reject: &Value, added: &Value, domain: &Value) -> Value {
    crate::unported!("bot.modules.answers:_from_rewrite")
}

/// Python: bot/modules/answers.py:6049 `_asks_storing(memory)`
pub fn _asks_storing(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_asks_storing")
}

/// Python: bot/modules/answers.py:6060 `_keeping_thing(memory)` (never called on the trace corpus)
pub fn _keeping_thing(memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_keeping_thing")
}

/// Python: bot/modules/answers.py:6068 `_the_plan_keeps(understood, memory)` (never called on the trace corpus)
pub fn _the_plan_keeps(understood: &Understood, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_the_plan_keeps")
}

/// Python: bot/modules/answers.py:6085 `_keeping_entry(text, memory, thing=None, storing=False)` (never called on the trace corpus)
pub fn _keeping_entry(text: &str, memory: &Memory, thing: &str, storing: &Value) -> Value {
    crate::unported!("bot.modules.answers:_keeping_entry")
}

/// Python: bot/modules/answers.py:6140 `_in_this_conversation(reject, reading)`
pub fn _in_this_conversation(reject: &Value, reading: &Understood) -> Value {
    crate::unported!("bot.modules.answers:_in_this_conversation")
}

/// Python: bot/modules/answers.py:6155 `_in_bare_context(text, memory, reject=None)`
pub fn _in_bare_context(text: &str, memory: &Value, reject: &Value) {
    crate::unported!("bot.modules.answers:_in_bare_context")
}

/// Python: bot/modules/answers.py:6178 `_reply_of(item, reading, memory)`
pub fn _reply_of(item: &'static Entry, reading: &Understood, memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_reply_of")
}

/// Python: bot/modules/answers.py:6198 `_figures_of(said)`
pub fn _figures_of(said: &str) -> Vec<String> {
    crate::unported!("bot.modules.answers:_figures_of")
}

/// Python: bot/modules/answers.py:6203 `_new_figure(fresh, before)`
pub fn _new_figure(fresh: &str, before: &str) -> bool {
    crate::unported!("bot.modules.answers:_new_figure")
}

/// Python: bot/modules/answers.py:6213 `_adds_a_figure(text)`
pub fn _adds_a_figure(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_adds_a_figure")
}

/// Python: bot/modules/answers.py:6233 `_a_figure_put_right(text)`
pub fn _a_figure_put_right(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_figure_put_right")
}

/// Python: bot/modules/answers.py:6239 `_without_the_figures(line)` (never called on the trace corpus)
pub fn _without_the_figures(line: &str) -> String {
    crate::unported!("bot.modules.answers:_without_the_figures")
}

/// Python: bot/modules/answers.py:6244 `_the_entry_asked_again(text, memory)`
pub fn _the_entry_asked_again(text: &str, memory: &Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:_the_entry_asked_again")
}

/// Python: bot/modules/answers.py:6294 `_the_part_a_longer_variant_adds(fresh, said)`
pub fn _the_part_a_longer_variant_adds(fresh: &str, said: &str) {
    crate::unported!("bot.modules.answers:_the_part_a_longer_variant_adds")
}

/// Python: bot/modules/answers.py:6319 `_payload(item, memory, reading)`
pub fn _payload(item: &'static Entry, memory: &Memory, reading: &Understood) -> String {
    crate::unported!("bot.modules.answers:_payload")
}

/// Python: bot/modules/answers.py:6329 `_the_sentence_owed(text, reply, memory)`
pub fn _the_sentence_owed(text: &str, reply: &str, memory: &Memory) {
    crate::unported!("bot.modules.answers:_the_sentence_owed")
}

/// Python: bot/modules/answers.py:6402 `_wants_a_stand_in(reading)`
pub fn _wants_a_stand_in(reading: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_wants_a_stand_in")
}

/// Python: bot/modules/answers.py:6428 `_a_swap_row(item)` (never called on the trace corpus)
pub fn _a_swap_row(item: &'static Entry) -> bool {
    crate::unported!("bot.modules.answers:_a_swap_row")
}

/// Python: bot/modules/answers.py:6484 `_price_limit(text)`
pub fn _price_limit(text: &str) -> Value {
    crate::unported!("bot.modules.answers:_price_limit")
}

/// Python: bot/modules/answers.py:6512 `_only_a_limit(text)`
pub fn _only_a_limit(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_only_a_limit")
}

/// Python: bot/modules/answers.py:6519 `_a_gift_entry(reading)`
pub fn _a_gift_entry(reading: &Understood) {
    crate::unported!("bot.modules.answers:_a_gift_entry")
}

/// Python: bot/modules/answers.py:6525 `_a_gift_just_answered(memory)` (never called on the trace corpus)
pub fn _a_gift_just_answered(memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_a_gift_just_answered")
}

/// Python: bot/modules/answers.py:6540 `_asks_what_to_give(reading)` (never called on the trace corpus)
pub fn _asks_what_to_give(reading: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_asks_what_to_give")
}

/// Python: bot/modules/answers.py:6551 `_the_interests(text, memory, item)`
pub fn _the_interests(text: &str, memory: &Memory, item: &'static Entry) -> Vec<String> {
    crate::unported!("bot.modules.answers:_the_interests")
}

/// Python: bot/modules/answers.py:6580 `_the_entry_reply(item, memory, reading)` (never called on the trace corpus)
pub fn _the_entry_reply(item: &'static Entry, memory: &Memory, reading: &Understood) -> Value {
    crate::unported!("bot.modules.answers:_the_entry_reply")
}

/// Python: bot/modules/answers.py:6590 `_the_present_with_a_price(text, reading, memory)`
pub fn _the_present_with_a_price(text: &str, reading: &Understood, memory: &Memory) {
    crate::unported!("bot.modules.answers:_the_present_with_a_price")
}

/// Python: bot/modules/answers.py:6666 `_asks_a_price(text)`
pub fn _asks_a_price(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_asks_a_price")
}

/// Python: bot/modules/answers.py:6687 `_names_what_it_wants_priced(text)`
pub fn _names_what_it_wants_priced(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_names_what_it_wants_priced")
}

/// Python: bot/modules/answers.py:6759 `_same_where_they_asked(text, said)` (never called on the trace corpus)
pub fn _same_where_they_asked(text: &str, said: &str) -> String {
    crate::unported!("bot.modules.answers:_same_where_they_asked")
}

/// Python: bot/modules/answers.py:6816 `_mid_sentence(part)` (never called on the trace corpus)
pub fn _mid_sentence(part: &str) -> Value {
    crate::unported!("bot.modules.answers:_mid_sentence")
}

/// Python: bot/modules/answers.py:6847 `_holds_where_they_asked(part, asked)` (never called on the trace corpus)
pub fn _holds_where_they_asked(part: &str, asked: &str) -> bool {
    crate::unported!("bot.modules.answers:_holds_where_they_asked")
}

/// Python: bot/modules/answers.py:6862 `_the_method_in(part)` (never called on the trace corpus)
pub fn _the_method_in(part: &str) -> Value {
    crate::unported!("bot.modules.answers:_the_method_in")
}

/// Python: bot/modules/answers.py:6876 `_room_they_asked_about(line, asked, room)` (never called on the trace corpus)
pub fn _room_they_asked_about(line: &str, asked: &str, room: &Value) -> Value {
    crate::unported!("bot.modules.answers:_room_they_asked_about")
}

/// Python: bot/modules/answers.py:6898 `_a_place_of_the_same_kind(earlier, text)` (never called on the trace corpus)
pub fn _a_place_of_the_same_kind(earlier: &Value, text: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_place_of_the_same_kind")
}

/// Python: bot/modules/answers.py:6914 `_a_place_they_asked(text)`
pub fn _a_place_they_asked(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_place_they_asked")
}

/// Python: bot/modules/answers.py:6925 `_key_terms(item)` (never called on the trace corpus)
pub fn _key_terms(item: &'static Entry) -> Value {
    crate::unported!("bot.modules.answers:_key_terms")
}

/// Python: bot/modules/answers.py:6947 `_written_for(item, thing)` (never called on the trace corpus)
pub fn _written_for(item: &'static Entry, thing: &str) -> bool {
    crate::unported!("bot.modules.answers:_written_for")
}

/// Python: bot/modules/answers.py:6967 `_more_than_the_thing(item, rest)` (never called on the trace corpus)
pub fn _more_than_the_thing(item: &'static Entry, rest: &str) -> bool {
    crate::unported!("bot.modules.answers:_more_than_the_thing")
}

/// Python: bot/modules/answers.py:6989 `_about_the_rest_of_the_question(item, question, new)` (never called on the trace corpus)
pub fn _about_the_rest_of_the_question(item: &'static Entry, question: &str, new: &Value) -> bool {
    crate::unported!("bot.modules.answers:_about_the_rest_of_the_question")
}

/// Python: bot/modules/answers.py:7021 `_offer(reader, memory, reading)`
pub fn _offer(reader: &Value, memory: &Memory, reading: &Understood) -> Value {
    crate::unported!("bot.modules.answers:_offer")
}

/// Python: bot/modules/answers.py:7124 `_keeping_in_the_answer_before(memory)` (never called on the trace corpus)
pub fn _keeping_in_the_answer_before(memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_keeping_in_the_answer_before")
}

/// Python: bot/modules/answers.py:7167 `_keeping_reader(reading, memory)`
pub fn _keeping_reader(reading: &Understood, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_keeping_reader")
}

/// Python: bot/modules/answers.py:7251 `_cooked_thing(memory)`
pub fn _cooked_thing(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_cooked_thing")
}

/// Python: bot/modules/answers.py:7256 `_the_plan_for(text, memory, understood)` (never called on the trace corpus)
pub fn _the_plan_for(text: &str, memory: &Memory, understood: &Understood) -> Value {
    crate::unported!("bot.modules.answers:_the_plan_for")
}

/// Python: bot/modules/answers.py:7297 `_names_another_food(understood, subject)`
pub fn _names_another_food(understood: &Understood, subject: &str) -> bool {
    crate::unported!("bot.modules.answers:_names_another_food")
}

/// Python: bot/modules/answers.py:7334 `_the_food_named(subject)`
pub fn _the_food_named(subject: &str) -> String {
    crate::unported!("bot.modules.answers:_the_food_named")
}

/// Python: bot/modules/answers.py:7340 `_a_cooking_ask_after_a_keeping_answer(understood, memory)`
pub fn _a_cooking_ask_after_a_keeping_answer(understood: &Understood, memory: &Memory) {
    crate::unported!("bot.modules.answers:_a_cooking_ask_after_a_keeping_answer")
}

/// Python: bot/modules/answers.py:7405 `_the_plan_answers_this(memory)`
pub fn _the_plan_answers_this(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_plan_answers_this")
}

/// Python: bot/modules/answers.py:7421 `_the_plan_said_again(understood, text, memory)`
pub fn _the_plan_said_again(understood: &Understood, text: &str, memory: &Memory) {
    crate::unported!("bot.modules.answers:_the_plan_said_again")
}

/// Python: bot/modules/answers.py:7454 `_filling_in_an_emergency(understood, memory)`
pub fn _filling_in_an_emergency(understood: &Understood, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_filling_in_an_emergency")
}

/// Python: bot/modules/answers.py:7469 `_usable(item, reading, memory)`
pub fn _usable(item: &'static Entry, reading: &Understood, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_usable")
}

/// Python: bot/modules/answers.py:7479 `_next_usable(item, reading, memory)` (never called on the trace corpus)
pub fn _next_usable(item: &'static Entry, reading: &Understood, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_next_usable")
}

/// Python: bot/modules/answers.py:7485 `_without_names(reading)`
pub fn _without_names(reading: &Understood) -> Understood {
    crate::unported!("bot.modules.answers:_without_names")
}

/// Python: bot/modules/answers.py:7570 `_only_the_tone(rest)` (never called on the trace corpus)
pub fn _only_the_tone(rest: &str) -> bool {
    crate::unported!("bot.modules.answers:_only_the_tone")
}

/// Python: bot/modules/answers.py:7604 `_reword_ask(text)`
pub fn _reword_ask(text: &str) {
    crate::unported!("bot.modules.answers:_reword_ask")
}

/// Python: bot/modules/answers.py:7647 `_tone_of(text)` (never called on the trace corpus)
pub fn _tone_of(text: &str) -> Value {
    crate::unported!("bot.modules.answers:_tone_of")
}

/// Python: bot/modules/answers.py:7660 `_softened(sentence, tones)` (never called on the trace corpus)
pub fn _softened(sentence: &str, tones: &Value) -> Value {
    crate::unported!("bot.modules.answers:_softened")
}

/// Python: bot/modules/answers.py:7714 `_last_rewrite(memory)` (never called on the trace corpus)
pub fn _last_rewrite(memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_last_rewrite")
}

/// Python: bot/modules/answers.py:7747 `_hard_to_say(asked)` (never called on the trace corpus)
pub fn _hard_to_say(asked: &str) -> Value {
    crate::unported!("bot.modules.answers:_hard_to_say")
}

/// Python: bot/modules/answers.py:7756 `_rewritten(asked, tones, from_followup=False)` (never called on the trace corpus)
pub fn _rewritten(asked: &str, tones: &Value, from_followup: &Value) -> String {
    crate::unported!("bot.modules.answers:_rewritten")
}

/// Python: bot/modules/answers.py:7845 `_asks_about_a_food(low)` (never called on the trace corpus)
pub fn _asks_about_a_food(low: &str) -> bool {
    crate::unported!("bot.modules.answers:_asks_about_a_food")
}

/// Python: bot/modules/answers.py:7857 `_someones_else_diet(text, memory, understood)`
pub fn _someones_else_diet(text: &str, memory: &Memory, understood: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_someones_else_diet")
}

/// Python: bot/modules/answers.py:7895 `_a_guest_restriction(text, memory, understood)` (never called on the trace corpus)
pub fn _a_guest_restriction(text: &str, memory: &Memory, understood: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_a_guest_restriction")
}

/// Python: bot/modules/answers.py:7921 `_names_someone_else(low)`
pub fn _names_someone_else(low: &str) -> bool {
    crate::unported!("bot.modules.answers:_names_someone_else")
}

/// Python: bot/modules/answers.py:7939 `_a_writing_job(understood)`
pub fn _a_writing_job(understood: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_a_writing_job")
}

/// Python: bot/modules/answers.py:7963 `_asks_about_their_taste(text)`
pub fn _asks_about_their_taste(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_asks_about_their_taste")
}

/// Python: bot/modules/answers.py:7968 `_about_them_alone(low)` (never called on the trace corpus)
pub fn _about_them_alone(low: &str) -> bool {
    crate::unported!("bot.modules.answers:_about_them_alone")
}

/// Python: bot/modules/answers.py:7982 `_health_entries()`
pub fn _health_entries() -> Value {
    crate::unported!("bot.modules.answers:_health_entries")
}

/// Python: bot/modules/answers.py:7990 `_names_a_typed_word(item, text)` (never called on the trace corpus)
pub fn _names_a_typed_word(item: &'static Entry, text: &str) -> bool {
    crate::unported!("bot.modules.answers:_names_a_typed_word")
}

/// Python: bot/modules/answers.py:8018 `_circumstances(text)`
pub fn _circumstances(text: &str) -> Vec<String> {
    crate::unported!("bot.modules.answers:_circumstances")
}

/// Python: bot/modules/answers.py:8041 `_detail_frame(part)` (never called on the trace corpus)
pub fn _detail_frame(part: &str) -> String {
    crate::unported!("bot.modules.answers:_detail_frame")
}

/// Python: bot/modules/answers.py:8045 `_a_figure_the_answer_has(text, memory)`
pub fn _a_figure_the_answer_has(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.answers:_a_figure_the_answer_has")
}

/// Python: bot/modules/answers.py:8054 `_a_detail_answered(text, memory)`
pub fn _a_detail_answered(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.answers:_a_detail_answered")
}

/// Python: bot/modules/answers.py:8102 `_a_complaint_answered(text, understood, memory)`
pub fn _a_complaint_answered(text: &str, understood: &Understood, memory: &mut Memory) {
    crate::unported!("bot.modules.answers:_a_complaint_answered")
}

/// Python: bot/modules/answers.py:8134 `_a_dish_was_offered(memory)`
pub fn _a_dish_was_offered(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_a_dish_was_offered")
}

/// Python: bot/modules/answers.py:8147 `observe(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn observe(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.answers:observe")
}

/// Python: bot/modules/answers.py:8178 `_asks_only_the_length(text)`
pub fn _asks_only_the_length(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_asks_only_the_length")
}

/// Python: bot/modules/answers.py:8187 `TAKES_AS_IT_COMES(text, memory=None)`
pub fn TAKES_AS_IT_COMES(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.answers:TAKES_AS_IT_COMES")
}

/// Python: bot/modules/answers.py:8201 `claims_instead(text, memory=None)` (never called on the trace corpus)
pub fn claims_instead(text: &str, memory: &mut Memory) -> Option<bool> {
    crate::unported!("bot.modules.answers:claims_instead")
}

/// Python: bot/modules/answers.py:8224 `_a_limit_on_the_plan(text, memory)`
pub fn _a_limit_on_the_plan(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_a_limit_on_the_plan")
}

/// Python: bot/modules/answers.py:8250 `_a_bare_thanks(understood)`
pub fn _a_bare_thanks(understood: Option<&Understood>) -> bool {
    crate::unported!("bot.modules.answers:_a_bare_thanks")
}

/// Python: bot/modules/answers.py:8265 `_a_judgement_of_the_figure_on_the_table(text, memory)`
pub fn _a_judgement_of_the_figure_on_the_table(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_a_judgement_of_the_figure_on_the_table")
}

/// Python: bot/modules/answers.py:8282 `_the_bot_has_something_to_go_over(memory)` (never called on the trace corpus)
pub fn _the_bot_has_something_to_go_over(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_bot_has_something_to_go_over")
}

/// Python: bot/modules/answers.py:8315 `_the_line_asks_what_this_one_meant(memory)`
pub fn _the_line_asks_what_this_one_meant(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_line_asks_what_this_one_meant")
}

/// Python: bot/modules/answers.py:8331 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:respond")
}

/// Python: bot/modules/answers.py:8580 `_the_question_with_the_place_on_it(text, memory, reply)`
pub fn _the_question_with_the_place_on_it(text: &str, memory: &Memory, reply: Option<&str>) -> String {
    crate::unported!("bot.modules.answers:_the_question_with_the_place_on_it")
}

/// Python: bot/modules/answers.py:8623 `_of_its_own(memory)` (never called on the trace corpus)
pub fn _of_its_own(memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_of_its_own")
}

/// Python: bot/modules/answers.py:8631 `_keep_the_answer(text, reply, memory)`
pub fn _keep_the_answer(text: &str, reply: Option<&str>, memory: &mut Memory) {
    crate::unported!("bot.modules.answers:_keep_the_answer")
}

/// Python: bot/modules/answers.py:8655 `_the_sentence_it_names(reply, text, memory)`
pub fn _the_sentence_it_names(reply: &str, text: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_the_sentence_it_names")
}

/// Python: bot/modules/answers.py:8685 `_fitted_to_what_they_eat(reply, text, memory)`
pub fn _fitted_to_what_they_eat(reply: &str, text: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_fitted_to_what_they_eat")
}

/// Python: bot/modules/answers.py:8726 `_the_dish_by_name(answer, memory)`
pub fn _the_dish_by_name(answer: Option<&str>, memory: &Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:_the_dish_by_name")
}

/// Python: bot/modules/answers.py:8762 `_the_same_answer_again(answer, memory)` (never called on the trace corpus)
pub fn _the_same_answer_again(answer: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_same_answer_again")
}

/// Python: bot/modules/answers.py:8774 `_the_figure_the_thing_we_are_on_holds(text, memory)` (never called on the trace corpus)
pub fn _the_figure_the_thing_we_are_on_holds(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_the_figure_the_thing_we_are_on_holds")
}

/// Python: bot/modules/answers.py:8799 `_a_length_ask_with_no_figure(text, memory, answer='')` (never called on the trace corpus)
pub fn _a_length_ask_with_no_figure(text: &str, memory: &Memory, answer: &str) -> Value {
    crate::unported!("bot.modules.answers:_a_length_ask_with_no_figure")
}

/// Python: bot/modules/answers.py:8828 `_a_city_card_asks_which_first(text, memory)` (never called on the trace corpus)
pub fn _a_city_card_asks_which_first(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_a_city_card_asks_which_first")
}

/// Python: bot/modules/answers.py:8839 `_the_answer_in_front(memory)` (never called on the trace corpus)
pub fn _the_answer_in_front(memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_the_answer_in_front")
}

/// Python: bot/modules/answers.py:8920 `_a_lead_time(text)`
pub fn _a_lead_time(text: &str) {
    crate::unported!("bot.modules.answers:_a_lead_time")
}

/// Python: bot/modules/answers.py:8950 `_the_thing_has_got_better(text, memory)`
pub fn _the_thing_has_got_better(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_thing_has_got_better")
}

/// Python: bot/modules/answers.py:8983 `_a_case_of_the_answer_in_hand(text, memory)`
pub fn _a_case_of_the_answer_in_hand(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_a_case_of_the_answer_in_hand")
}

/// Python: bot/modules/answers.py:9060 `_answer(text, memory)`
pub fn _answer(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:_answer")
}

/// Python: bot/modules/answers.py:9644 `_a_measure_about_the_subject(measure, text, memory, alone='')`
pub fn _a_measure_about_the_subject(measure: &str, text: &str, memory: &mut Memory, alone: &str) -> String {
    crate::unported!("bot.modules.answers:_a_measure_about_the_subject")
}

/// Python: bot/modules/answers.py:9803 `_the_line_already_has_its_answer(alone, text)` (never called on the trace corpus)
pub fn _the_line_already_has_its_answer(alone: &Value, text: &str) -> bool {
    crate::unported!("bot.modules.answers:_the_line_already_has_its_answer")
}

/// Python: bot/modules/answers.py:9833 `_names_a_repair(entry)` (never called on the trace corpus)
pub fn _names_a_repair(entry: &'static Entry) -> bool {
    crate::unported!("bot.modules.answers:_names_a_repair")
}

/// Python: bot/modules/answers.py:9841 `_the_entry_names(entry, word)` (never called on the trace corpus)
pub fn _the_entry_names(entry: &'static Entry, word: &str) -> bool {
    crate::unported!("bot.modules.answers:_the_entry_names")
}

/// Python: bot/modules/answers.py:9849 `_the_plan_said(memory)` (never called on the trace corpus)
pub fn _the_plan_said(memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_the_plan_said")
}

/// Python: bot/modules/answers.py:9860 `_the_part_is_what_answers(part, memory)`
pub fn _the_part_is_what_answers(part: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_part_is_what_answers")
}

/// Python: bot/modules/answers.py:9891 `_the_part_of_the_thing_they_named(text, memory)`
pub fn _the_part_of_the_thing_they_named(text: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.modules.answers:_the_part_of_the_thing_they_named")
}

/// Python: bot/modules/answers.py:9931 `_the_report_is_answered(text, entry)`
pub fn _the_report_is_answered(text: &str, entry: Option<&'static Entry>) -> bool {
    crate::unported!("bot.modules.answers:_the_report_is_answered")
}

/// Python: bot/modules/answers.py:9975 `_the_fact_is_what_is_answered(text, stated, entry, memory=None)`
pub fn _the_fact_is_what_is_answered(text: &str, stated: &str, entry: Option<&'static Entry>, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_fact_is_what_is_answered")
}

/// Python: bot/modules/answers.py:10032 `_ask_a_line_of_its_own(line, memory)` (never called on the trace corpus)
pub fn _ask_a_line_of_its_own(line: &str, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_ask_a_line_of_its_own")
}

/// Python: bot/modules/answers.py:10046 `_the_entry_key_words(entry)` (never called on the trace corpus)
pub fn _the_entry_key_words(entry: &'static Entry) -> Value {
    crate::unported!("bot.modules.answers:_the_entry_key_words")
}

/// Python: bot/modules/answers.py:10056 `_the_practice_is_answered(practice, entry)` (never called on the trace corpus)
pub fn _the_practice_is_answered(practice: &Value, entry: &'static Entry) -> bool {
    crate::unported!("bot.modules.answers:_the_practice_is_answered")
}

/// Python: bot/modules/answers.py:10099 `_the_ask_behind_a_moan(text, memory)`
pub fn _the_ask_behind_a_moan(text: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.modules.answers:_the_ask_behind_a_moan")
}

/// Python: bot/modules/answers.py:10135 `_ask_a_rewritten_line(line, memory)`
pub fn _ask_a_rewritten_line(line: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:_ask_a_rewritten_line")
}

/// Python: bot/modules/answers.py:10177 `_turns_on_this_line(text, memory)`
pub fn _turns_on_this_line(text: &str, memory: &Memory) -> i64 {
    crate::unported!("bot.modules.answers:_turns_on_this_line")
}

/// Python: bot/modules/answers.py:10192 `_without_lead_in(answer)`
pub fn _without_lead_in(answer: &str) -> String {
    crate::unported!("bot.modules.answers:_without_lead_in")
}

/// Python: bot/modules/answers.py:10212 `_the_line_again(text, memory)`
pub fn _the_line_again(text: &str, memory: &Value) {
    crate::unported!("bot.modules.answers:_the_line_again")
}

/// Python: bot/modules/answers.py:10281 `_stated_with_copula(text, value)` (never called on the trace corpus)
pub fn _stated_with_copula(text: &str, value: &Value) -> bool {
    crate::unported!("bot.modules.answers:_stated_with_copula")
}

/// Python: bot/modules/answers.py:10298 `_nothing_but_the_slot(said, value_words, about)` (never called on the trace corpus)
pub fn _nothing_but_the_slot(said: &str, value_words: &[String], about: &Value) -> bool {
    crate::unported!("bot.modules.answers:_nothing_but_the_slot")
}

/// Python: bot/modules/answers.py:10320 `_the_food_they_do_not_eat(text, understood)`
pub fn _the_food_they_do_not_eat(text: &str, understood: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_the_food_they_do_not_eat")
}

/// Python: bot/modules/answers.py:10344 `_store_keeps_the_line(text, understood)`
pub fn _store_keeps_the_line(text: &str, understood: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_store_keeps_the_line")
}

/// Python: bot/modules/answers.py:10418 `_says_line(text, words)`
pub fn _says_line(text: &str, words: &[String]) -> bool {
    crate::unported!("bot.modules.answers:_says_line")
}

/// Python: bot/modules/answers.py:10436 `_a_pick_is_asked(text)`
pub fn _a_pick_is_asked(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_pick_is_asked")
}

/// Python: bot/modules/answers.py:10441 `_the_line_as_said(text)`
pub fn _the_line_as_said(text: &str) -> String {
    crate::unported!("bot.modules.answers:_the_line_as_said")
}

/// Python: bot/modules/answers.py:10477 `_only_the_office(memory)`
pub fn _only_the_office(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_only_the_office")
}

/// Python: bot/modules/answers.py:10492 `_a_food_named_of_its_own(understood, food)`
pub fn _a_food_named_of_its_own(understood: &Understood, food: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_food_named_of_its_own")
}

/// Python: bot/modules/answers.py:10512 `_keeping_about_the_food_there(memory, understood)`
pub fn _keeping_about_the_food_there(memory: &Memory, understood: &Understood) {
    crate::unported!("bot.modules.answers:_keeping_about_the_food_there")
}

/// Python: bot/modules/answers.py:10556 `_the_answer_on_the_table(understood, memory)`
pub fn _the_answer_on_the_table(understood: &Understood, memory: &Value) {
    crate::unported!("bot.modules.answers:_the_answer_on_the_table")
}

/// Python: bot/modules/answers.py:10595 `_already_on_the_screen(reply, memory, turns=4)` (never called on the trace corpus)
pub fn _already_on_the_screen(reply: &str, memory: &Memory, turns: &[Turn]) -> bool {
    crate::unported!("bot.modules.answers:_already_on_the_screen")
}

/// Python: bot/modules/answers.py:10624 `_the_verse_said_again(text, memory)`
pub fn _the_verse_said_again(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.answers:_the_verse_said_again")
}

/// Python: bot/modules/answers.py:10664 `_the_sentence_around(fragment, text)` (never called on the trace corpus)
pub fn _the_sentence_around(fragment: &str, text: &str) -> String {
    crate::unported!("bot.modules.answers:_the_sentence_around")
}

/// Python: bot/modules/answers.py:10691 `_the_sentence_with_a_figure(text)` (never called on the trace corpus)
pub fn _the_sentence_with_a_figure(text: &str) -> Value {
    crate::unported!("bot.modules.answers:_the_sentence_with_a_figure")
}

/// Python: bot/modules/answers.py:10741 `_puts_it_to_someone(reply)` (never called on the trace corpus)
pub fn _puts_it_to_someone(reply: &str) -> bool {
    crate::unported!("bot.modules.answers:_puts_it_to_someone")
}

/// Python: bot/modules/answers.py:10751 `_the_ask_is_for_words(reading)`
pub fn _the_ask_is_for_words(reading: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_the_ask_is_for_words")
}

/// Python: bot/modules/answers.py:10759 `_respond(text, memory)`
pub fn _respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:_respond")
}

