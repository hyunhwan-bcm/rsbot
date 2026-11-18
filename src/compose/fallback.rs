//! Port of `bot/compose/fallback.py`.
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

/// Python: bot/compose/fallback.py:801 `_bounded(phrase)`
pub fn _bounded(phrase: &str) -> Value {
    crate::unported!("bot.compose.fallback:_bounded")
}

/// Python: bot/compose/fallback.py:810 `_has(clean, phrases)`
pub fn _has(clean: &str, phrases: &[String]) -> bool {
    crate::unported!("bot.compose.fallback:_has")
}

/// Python: bot/compose/fallback.py:824 `_matchable(text)`
pub fn _matchable(text: &str) -> String {
    crate::unported!("bot.compose.fallback:_matchable")
}

/// Python: bot/compose/fallback.py:835 `_asks_how_are_you(clean, repaired='')`
pub fn _asks_how_are_you(clean: &str, repaired: &str) -> bool {
    crate::unported!("bot.compose.fallback:_asks_how_are_you")
}

/// Python: bot/compose/fallback.py:876 `_nothing_but_tags(form, end)` (never called on the trace corpus)
pub fn _nothing_but_tags(form: &Value, end: i64) -> bool {
    crate::unported!("bot.compose.fallback:_nothing_but_tags")
}

/// Python: bot/compose/fallback.py:1124 `_the_person_ends_the_ask(rest)` (never called on the trace corpus)
pub fn _the_person_ends_the_ask(rest: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_person_ends_the_ask")
}

/// Python: bot/compose/fallback.py:1146 `_the_person_they_are_stuck_on(clean)`
pub fn _the_person_they_are_stuck_on(clean: &str) -> String {
    crate::unported!("bot.compose.fallback:_the_person_they_are_stuck_on")
}

/// Python: bot/compose/fallback.py:1222 `_a_bad_line_above(memory)` (never called on the trace corpus)
pub fn _a_bad_line_above(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_bad_line_above")
}

/// Python: bot/compose/fallback.py:1247 `_run_out(clean)`
pub fn _run_out(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_run_out")
}

/// Python: bot/compose/fallback.py:1257 `_a_dead_end(clean)`
pub fn _a_dead_end(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_dead_end")
}

/// Python: bot/compose/fallback.py:1262 `_only_the_stuck_phrase(clean)`
pub fn _only_the_stuck_phrase(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_only_the_stuck_phrase")
}

/// Python: bot/compose/fallback.py:1270 `_a_question_about_the_person(clean, memory)`
pub fn _a_question_about_the_person(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_a_question_about_the_person")
}

/// Python: bot/compose/fallback.py:1303 `_nothing_to_choose(rest)` (never called on the trace corpus)
pub fn _nothing_to_choose(rest: &str) -> bool {
    crate::unported!("bot.compose.fallback:_nothing_to_choose")
}

/// Python: bot/compose/fallback.py:1336 `_the_stretch_of_time(clean, memory)`
pub fn _the_stretch_of_time(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_the_stretch_of_time")
}

/// Python: bot/compose/fallback.py:1350 `_weighs_a_choice(clean)`
pub fn _weighs_a_choice(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_weighs_a_choice")
}

/// Python: bot/compose/fallback.py:1411 `_reads_as_dither(clean, repaired=None)`
pub fn _reads_as_dither(clean: &str, repaired: &str) -> bool {
    crate::unported!("bot.compose.fallback:_reads_as_dither")
}

/// Python: bot/compose/fallback.py:1634 `_a_plan_in_the_thread(clean, memory)` (never called on the trace corpus)
pub fn _a_plan_in_the_thread(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_plan_in_the_thread")
}

/// Python: bot/compose/fallback.py:1696 `_a_carer_refused(clean)`
pub fn _a_carer_refused(clean: &str) {
    crate::unported!("bot.compose.fallback:_a_carer_refused")
}

/// Python: bot/compose/fallback.py:1713 `_a_plan_doubt(pred, clean, repaired)`
pub fn _a_plan_doubt(pred: &Value, clean: &str, repaired: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_plan_doubt")
}

/// Python: bot/compose/fallback.py:1759 `_a_bare_worry(clean)` (never called on the trace corpus)
pub fn _a_bare_worry(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_bare_worry")
}

/// Python: bot/compose/fallback.py:1769 `_worry_is_about_the_body(clean, words)`
pub fn _worry_is_about_the_body(clean: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.fallback:_worry_is_about_the_body")
}

/// Python: bot/compose/fallback.py:1845 `_somebody_to_visit(clean, turns, look_back=3)` (never called on the trace corpus)
pub fn _somebody_to_visit(clean: &str, turns: &[Turn], look_back: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_somebody_to_visit")
}

/// Python: bot/compose/fallback.py:1857 `_someone_is_a_patient(clean, turns, look_back=4)` (never called on the trace corpus)
pub fn _someone_is_a_patient(clean: &str, turns: &[Turn], look_back: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_someone_is_a_patient")
}

/// Python: bot/compose/fallback.py:1938 `_the_animal_is_the_patient(said)`
pub fn _the_animal_is_the_patient(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_animal_is_the_patient")
}

/// Python: bot/compose/fallback.py:1958 `_the_person_is_the_patient(said)`
pub fn _the_person_is_the_patient(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_person_is_the_patient")
}

/// Python: bot/compose/fallback.py:1975 `_the_thread_is_an_animal(clean, turns, look_back=4)`
pub fn _the_thread_is_an_animal(clean: &str, turns: &Value, look_back: i64) -> bool {
    crate::unported!("bot.compose.fallback:_the_thread_is_an_animal")
}

/// Python: bot/compose/fallback.py:1993 `_thread_is_about_someone_else(memory, look_back=3)` (never called on the trace corpus)
pub fn _thread_is_about_someone_else(memory: &Memory, look_back: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_thread_is_about_someone_else")
}

/// Python: bot/compose/fallback.py:2008 `_thread_is_about_a_pet(memory, look_back=3)`
pub fn _thread_is_about_a_pet(memory: &Memory, look_back: i64) -> bool {
    crate::unported!("bot.compose.fallback:_thread_is_about_a_pet")
}

/// Python: bot/compose/fallback.py:2145 `_the_thread_said(memory, look_back=3)` (never called on the trace corpus)
pub fn _the_thread_said(memory: &Memory, look_back: &Value) -> String {
    crate::unported!("bot.compose.fallback:_the_thread_said")
}

/// Python: bot/compose/fallback.py:2151 `_a_person_in_the_thread(memory, look_back=3)`
pub fn _a_person_in_the_thread(memory: &Memory, look_back: i64) -> bool {
    crate::unported!("bot.compose.fallback:_a_person_in_the_thread")
}

/// Python: bot/compose/fallback.py:2167 `_a_hard_time_and_not_a_body(memory, line='')` (never called on the trace corpus)
pub fn _a_hard_time_and_not_a_body(memory: &Memory, line: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_hard_time_and_not_a_body")
}

/// Python: bot/compose/fallback.py:2181 `_a_pet_in_trouble(line, memory)`
pub fn _a_pet_in_trouble(line: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_pet_in_trouble")
}

/// Python: bot/compose/fallback.py:2197 `_the_pool_for_a_worry_about_them(clean, memory)`
pub fn _the_pool_for_a_worry_about_them(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_the_pool_for_a_worry_about_them")
}

/// Python: bot/compose/fallback.py:2262 `_the_speaker_is_the_subject(said)`
pub fn _the_speaker_is_the_subject(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_speaker_is_the_subject")
}

/// Python: bot/compose/fallback.py:2271 `_a_thing_they_started(memory, look_back=3)` (never called on the trace corpus)
pub fn _a_thing_they_started(memory: &Memory, look_back: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_a_thing_they_started")
}

/// Python: bot/compose/fallback.py:2285 `_names_their_age(clean)` (never called on the trace corpus)
pub fn _names_their_age(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_names_their_age")
}

/// Python: bot/compose/fallback.py:2291 `_reads_as_worry(clean, words=())`
pub fn _reads_as_worry(clean: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.fallback:_reads_as_worry")
}

/// Python: bot/compose/fallback.py:2301 `_the_weird_ask_pool(clean, memory)`
pub fn _the_weird_ask_pool(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_the_weird_ask_pool")
}

/// Python: bot/compose/fallback.py:2324 `_nothing_but_the_ask(clean, found)`
pub fn _nothing_but_the_ask(clean: &str, found: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_nothing_but_the_ask")
}

/// Python: bot/compose/fallback.py:2330 `_without_the_idioms(said)`
pub fn _without_the_idioms(said: &str) -> Vec<String> {
    crate::unported!("bot.compose.fallback:_without_the_idioms")
}

/// Python: bot/compose/fallback.py:2351 `_thread_is_about_the_body(memory, look_back=4)`
pub fn _thread_is_about_the_body(memory: &Memory, look_back: i64) -> bool {
    crate::unported!("bot.compose.fallback:_thread_is_about_the_body")
}

/// Python: bot/compose/fallback.py:2366 `_the_threads_who(memory, look_back=2)`
pub fn _the_threads_who(memory: &Memory, look_back: i64) {
    crate::unported!("bot.compose.fallback:_the_threads_who")
}

/// Python: bot/compose/fallback.py:2397 `the_threads_answer(text, memory)`
pub fn the_threads_answer(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:the_threads_answer")
}

/// Python: bot/compose/fallback.py:2449 `_thread_names_a_machine(memory, look_back=4)` (never called on the trace corpus)
pub fn _thread_names_a_machine(memory: &Memory, look_back: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_thread_names_a_machine")
}

/// Python: bot/compose/fallback.py:2490 `_the_ask_is_the_threads(clean, understood)`
pub fn _the_ask_is_the_threads(clean: &str, understood: &Understood) -> bool {
    crate::unported!("bot.compose.fallback:_the_ask_is_the_threads")
}

/// Python: bot/compose/fallback.py:2503 `_thread_is_about_a_person(memory, look_back=4)` (never called on the trace corpus)
pub fn _thread_is_about_a_person(memory: &Memory, look_back: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_thread_is_about_a_person")
}

/// Python: bot/compose/fallback.py:2513 `_a_thing_was_named(memory, look_back=4)` (never called on the trace corpus)
pub fn _a_thing_was_named(memory: &Memory, look_back: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_a_thing_was_named")
}

/// Python: bot/compose/fallback.py:2535 `_names_its_own_thing(clean)` (never called on the trace corpus)
pub fn _names_its_own_thing(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_names_its_own_thing")
}

/// Python: bot/compose/fallback.py:2554 `_money_decision(clean)`
pub fn _money_decision(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_money_decision")
}

/// Python: bot/compose/fallback.py:2564 `_about_a_sore(clean, words)`
pub fn _about_a_sore(clean: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.fallback:_about_a_sore")
}

/// Python: bot/compose/fallback.py:2583 `_a_call_for_help(clean)`
pub fn _a_call_for_help(clean: &str) {
    crate::unported!("bot.compose.fallback:_a_call_for_help")
}

/// Python: bot/compose/fallback.py:2627 `_a_small_social_decide(clean, words)`
pub fn _a_small_social_decide(clean: &str, words: &[String]) {
    crate::unported!("bot.compose.fallback:_a_small_social_decide")
}

/// Python: bot/compose/fallback.py:2691 `method_ask_pool(clean, memory=None)`
pub fn method_ask_pool(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:method_ask_pool")
}

/// Python: bot/compose/fallback.py:2719 `unknown_pool(clean, words=())`
pub fn unknown_pool(clean: &str, words: &[String]) -> Vec<String> {
    crate::unported!("bot.compose.fallback:unknown_pool")
}

/// Python: bot/compose/fallback.py:2798 `_the_thing_is_choking(clean, words)` (never called on the trace corpus)
pub fn _the_thing_is_choking(clean: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.fallback:_the_thing_is_choking")
}

/// Python: bot/compose/fallback.py:2932 `_the_line_before(memory)`
pub fn _the_line_before(memory: &Memory) -> String {
    crate::unported!("bot.compose.fallback:_the_line_before")
}

/// Python: bot/compose/fallback.py:2940 `_reads_as_heavy_news(text)`
pub fn _reads_as_heavy_news(text: &str) -> bool {
    crate::unported!("bot.compose.fallback:_reads_as_heavy_news")
}

/// Python: bot/compose/fallback.py:2954 `_the_thread_is_heavy(memory)`
pub fn _the_thread_is_heavy(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_the_thread_is_heavy")
}

/// Python: bot/compose/fallback.py:3047 `_a_switch_off_asked(memory, clean)`
pub fn _a_switch_off_asked(memory: &Memory, clean: &str) {
    crate::unported!("bot.compose.fallback:_a_switch_off_asked")
}

/// Python: bot/compose/fallback.py:3144 `names_the_thing_worth_watching(clean)` (never called on the trace corpus)
pub fn names_the_thing_worth_watching(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:names_the_thing_worth_watching")
}

/// Python: bot/compose/fallback.py:3165 `reads_as_if_i_saw(text, memory=None)`
pub fn reads_as_if_i_saw(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:reads_as_if_i_saw")
}

/// Python: bot/compose/fallback.py:3251 `_asked_about_the_day_they_had(clean)`
pub fn _asked_about_the_day_they_had(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_asked_about_the_day_they_had")
}

/// Python: bot/compose/fallback.py:3342 `_a_lost_ask(clean)`
pub fn _a_lost_ask(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_lost_ask")
}

/// Python: bot/compose/fallback.py:3394 `_a_start_ask(clean)`
pub fn _a_start_ask(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_start_ask")
}

/// Python: bot/compose/fallback.py:3400 `_said_what_it_is_about(memory)` (never called on the trace corpus)
pub fn _said_what_it_is_about(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_said_what_it_is_about")
}

/// Python: bot/compose/fallback.py:3424 `the_pool_for_a_lost_ask(memory, clean='')`
pub fn the_pool_for_a_lost_ask(memory: &Memory, clean: &str) -> Vec<String> {
    crate::unported!("bot.compose.fallback:the_pool_for_a_lost_ask")
}

/// Python: bot/compose/fallback.py:3435 `the_lost_ask_reply(memory, clean, repaired=None)`
pub fn the_lost_ask_reply(memory: &Memory, clean: &str, repaired: &str) -> Option<String> {
    crate::unported!("bot.compose.fallback:the_lost_ask_reply")
}

/// Python: bot/compose/fallback.py:3499 `_news_above(memory, heavy=False)`
pub fn _news_above(memory: &Memory, heavy: bool) -> bool {
    crate::unported!("bot.compose.fallback:_news_above")
}

/// Python: bot/compose/fallback.py:3535 `_weather_above(memory)` (never called on the trace corpus)
pub fn _weather_above(memory: &Memory) -> Option<String> {
    crate::unported!("bot.compose.fallback:_weather_above")
}

/// Python: bot/compose/fallback.py:3560 `_an_occasion_above(memory)` (never called on the trace corpus)
pub fn _an_occasion_above(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_an_occasion_above")
}

/// Python: bot/compose/fallback.py:3591 `_an_occasion_in_the_thread(memory, back=4)`
pub fn _an_occasion_in_the_thread(memory: &Memory, back: i64) -> bool {
    crate::unported!("bot.compose.fallback:_an_occasion_in_the_thread")
}

/// Python: bot/compose/fallback.py:3623 `_a_trip_above(memory)`
pub fn _a_trip_above(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_trip_above")
}

/// Python: bot/compose/fallback.py:3647 `_a_subject_above(memory)` (never called on the trace corpus)
pub fn _a_subject_above(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_subject_above")
}

/// Python: bot/compose/fallback.py:3676 `_the_ask_clause(clean, memory=None)`
pub fn _the_ask_clause(clean: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.fallback:_the_ask_clause")
}

/// Python: bot/compose/fallback.py:3693 `_bare_pick(clean, memory=None)`
pub fn _bare_pick(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_bare_pick")
}

/// Python: bot/compose/fallback.py:3792 `_opens_the_line(prefix)` (never called on the trace corpus)
pub fn _opens_the_line(prefix: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_opens_the_line")
}

/// Python: bot/compose/fallback.py:3835 `_asks_elsewhere(tail)` (never called on the trace corpus)
pub fn _asks_elsewhere(tail: &str) -> bool {
    crate::unported!("bot.compose.fallback:_asks_elsewhere")
}

/// Python: bot/compose/fallback.py:3846 `_asks_for_a_person_to_talk_to(clean)`
pub fn _asks_for_a_person_to_talk_to(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_asks_for_a_person_to_talk_to")
}

/// Python: bot/compose/fallback.py:3887 `_asks_for_help(clean, understood=None)`
pub fn _asks_for_help(clean: &str, understood: Option<&Understood>) -> bool {
    crate::unported!("bot.compose.fallback:_asks_for_help")
}

/// Python: bot/compose/fallback.py:4151 `_says_only_what_they_said(lines, clean)`
pub fn _says_only_what_they_said(lines: &[String], clean: &str) -> Vec<String> {
    crate::unported!("bot.compose.fallback:_says_only_what_they_said")
}

/// Python: bot/compose/fallback.py:4222 `_the_pronoun(clean)` (never called on the trace corpus)
pub fn _the_pronoun(clean: &str) -> String {
    crate::unported!("bot.compose.fallback:_the_pronoun")
}

/// Python: bot/compose/fallback.py:4239 `_occasion_pool(occasion, clean)` (never called on the trace corpus)
pub fn _occasion_pool(occasion: &Value, clean: &str) -> Value {
    crate::unported!("bot.compose.fallback:_occasion_pool")
}

/// Python: bot/compose/fallback.py:4265 `_their_own_slip(clean, memory=None)`
pub fn _their_own_slip(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_their_own_slip")
}

/// Python: bot/compose/fallback.py:4371 `reads_as_someone_close_is_ill(clean)` (never called on the trace corpus)
pub fn reads_as_someone_close_is_ill(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:reads_as_someone_close_is_ill")
}

/// Python: bot/compose/fallback.py:4501 `a_fresh_death_and_the_words(clean)`
pub fn a_fresh_death_and_the_words(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:a_fresh_death_and_the_words")
}

/// Python: bot/compose/fallback.py:4517 `asks_what_to_tell_a_child(clean)`
pub fn asks_what_to_tell_a_child(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:asks_what_to_tell_a_child")
}

/// Python: bot/compose/fallback.py:4622 `_an_open_result(clean)`
pub fn _an_open_result(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_an_open_result")
}

/// Python: bot/compose/fallback.py:4651 `_the_news_landed(clean)`
pub fn _the_news_landed(clean: &str) {
    crate::unported!("bot.compose.fallback:_the_news_landed")
}

/// Python: bot/compose/fallback.py:4668 `reads_as_a_result_line(text)`
pub fn reads_as_a_result_line(text: &str) -> bool {
    crate::unported!("bot.compose.fallback:reads_as_a_result_line")
}

/// Python: bot/compose/fallback.py:4683 `_a_guessed_result(clean)`
pub fn _a_guessed_result(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_guessed_result")
}

/// Python: bot/compose/fallback.py:4688 `_a_time_in_it(clean)` (never called on the trace corpus)
pub fn _a_time_in_it(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_time_in_it")
}

/// Python: bot/compose/fallback.py:4712 `_a_cost_somebody_asks(clean)`
pub fn _a_cost_somebody_asks(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_cost_somebody_asks")
}

/// Python: bot/compose/fallback.py:4867 `_the_day_described(clean)`
pub fn _the_day_described(clean: &str) {
    crate::unported!("bot.compose.fallback:_the_day_described")
}

/// Python: bot/compose/fallback.py:4961 `the_day_remarked(text)`
pub fn the_day_remarked(text: &str) {
    crate::unported!("bot.compose.fallback:the_day_remarked")
}

/// Python: bot/compose/fallback.py:5070 `_the_reason_is_on_the_body(text)` (never called on the trace corpus)
pub fn _the_reason_is_on_the_body(text: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_reason_is_on_the_body")
}

/// Python: bot/compose/fallback.py:5104 `_the_line_cancels(line)` (never called on the trace corpus)
pub fn _the_line_cancels(line: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_line_cancels")
}

/// Python: bot/compose/fallback.py:5109 `_a_plan_on_the_table(memory)` (never called on the trace corpus)
pub fn _a_plan_on_the_table(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_plan_on_the_table")
}

/// Python: bot/compose/fallback.py:5124 `_hesitant_news_before(memory, turns=3)` (never called on the trace corpus)
pub fn _hesitant_news_before(memory: &Memory, turns: &[Turn]) -> bool {
    crate::unported!("bot.compose.fallback:_hesitant_news_before")
}

/// Python: bot/compose/fallback.py:5140 `_a_hard_week(memory)` (never called on the trace corpus)
pub fn _a_hard_week(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_hard_week")
}

/// Python: bot/compose/fallback.py:5150 `_the_turn_before_was_a_heavy_one(memory, line='')` (never called on the trace corpus)
pub fn _the_turn_before_was_a_heavy_one(memory: &Memory, line: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_turn_before_was_a_heavy_one")
}

/// Python: bot/compose/fallback.py:5207 `_a_declined_plan(clean, without_a_thing=False)`
pub fn _a_declined_plan(clean: &str, without_a_thing: bool) -> bool {
    crate::unported!("bot.compose.fallback:_a_declined_plan")
}

/// Python: bot/compose/fallback.py:5275 `_someone_else_dropped_it(clean)` (never called on the trace corpus)
pub fn _someone_else_dropped_it(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_someone_else_dropped_it")
}

/// Python: bot/compose/fallback.py:5336 `_no_words(clean)`
pub fn _no_words(clean: &str) {
    crate::unported!("bot.compose.fallback:_no_words")
}

/// Python: bot/compose/fallback.py:5377 `_an_invitation(clean)`
pub fn _an_invitation(clean: &str) {
    crate::unported!("bot.compose.fallback:_an_invitation")
}

/// Python: bot/compose/fallback.py:5449 `the_bound_said(clean)` (never called on the trace corpus)
pub fn the_bound_said(clean: &str) -> String {
    crate::unported!("bot.compose.fallback:the_bound_said")
}

/// Python: bot/compose/fallback.py:5461 `reads_as_a_narrowing(clean)`
pub fn reads_as_a_narrowing(clean: &str) -> String {
    crate::unported!("bot.compose.fallback:reads_as_a_narrowing")
}

/// Python: bot/compose/fallback.py:5484 `the_limits_said(clean)`
pub fn the_limits_said(clean: &str) -> Vec<String> {
    crate::unported!("bot.compose.fallback:the_limits_said")
}

/// Python: bot/compose/fallback.py:5514 `_one_axis(line)`
pub fn _one_axis(line: &str) -> String {
    crate::unported!("bot.compose.fallback:_one_axis")
}

/// Python: bot/compose/fallback.py:5545 `_names_things_to_choose(said)` (never called on the trace corpus)
pub fn _names_things_to_choose(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_names_things_to_choose")
}

/// Python: bot/compose/fallback.py:5596 `the_answer_to_cut(memory, axis='')` (never called on the trace corpus)
pub fn the_answer_to_cut(memory: &Memory, axis: &Value) -> Value {
    crate::unported!("bot.compose.fallback:the_answer_to_cut")
}

/// Python: bot/compose/fallback.py:5683 `_the_person_already_said_it(memory)` (never called on the trace corpus)
pub fn _the_person_already_said_it(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_the_person_already_said_it")
}

/// Python: bot/compose/fallback.py:5697 `_a_care_thread_to_leave_alone(memory)`
pub fn _a_care_thread_to_leave_alone(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_care_thread_to_leave_alone")
}

/// Python: bot/compose/fallback.py:5711 `_the_last_few_lines(memory)`
pub fn _the_last_few_lines(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.compose.fallback:_the_last_few_lines")
}

/// Python: bot/compose/fallback.py:5718 `baby_loss(clean)`
pub fn baby_loss(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:baby_loss")
}

/// Python: bot/compose/fallback.py:5756 `_the_answer_above_was_a_joke(memory)` (never called on the trace corpus)
pub fn _the_answer_above_was_a_joke(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_the_answer_above_was_a_joke")
}

/// Python: bot/compose/fallback.py:5767 `_a_verdict_line(text, memory=None)`
pub fn _a_verdict_line(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_a_verdict_line")
}

/// Python: bot/compose/fallback.py:5807 `a_death_just_said(memory=None)` (never called on the trace corpus)
pub fn a_death_just_said(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:a_death_just_said")
}

/// Python: bot/compose/fallback.py:5825 `a_verdict_on_the_person(memory=None)` (never called on the trace corpus)
pub fn a_verdict_on_the_person(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:a_verdict_on_the_person")
}

/// Python: bot/compose/fallback.py:5848 `_the_last_two_lines(memory=None)` (never called on the trace corpus)
pub fn _the_last_two_lines(memory: &Memory) -> Value {
    crate::unported!("bot.compose.fallback:_the_last_two_lines")
}

/// Python: bot/compose/fallback.py:5858 `_a_loss_in_the_same_breath(memory=None, lines=())` (never called on the trace corpus)
pub fn _a_loss_in_the_same_breath(memory: &Memory, lines: &[String]) -> bool {
    crate::unported!("bot.compose.fallback:_a_loss_in_the_same_breath")
}

/// Python: bot/compose/fallback.py:5879 `a_wait_with_no_loss(memory=None)` (never called on the trace corpus)
pub fn a_wait_with_no_loss(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:a_wait_with_no_loss")
}

/// Python: bot/compose/fallback.py:5887 `a_setback_with_no_loss(memory=None)` (never called on the trace corpus)
pub fn a_setback_with_no_loss(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:a_setback_with_no_loss")
}

/// Python: bot/compose/fallback.py:5895 `_the_heavy_pool(memory=None)` (never called on the trace corpus)
pub fn _the_heavy_pool(memory: &Memory) -> Value {
    crate::unported!("bot.compose.fallback:_the_heavy_pool")
}

/// Python: bot/compose/fallback.py:5917 `a_heavy_verdict(memory=None)` (never called on the trace corpus)
pub fn a_heavy_verdict(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:a_heavy_verdict")
}

/// Python: bot/compose/fallback.py:5940 `a_happening_now(memory=None)` (never called on the trace corpus)
pub fn a_happening_now(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:a_happening_now")
}

/// Python: bot/compose/fallback.py:5959 `_a_verdict_on_something(text, memory)` (never called on the trace corpus)
pub fn _a_verdict_on_something(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_verdict_on_something")
}

/// Python: bot/compose/fallback.py:5976 `_verdict_pool(clean, memory=None)`
pub fn _verdict_pool(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_verdict_pool")
}

/// Python: bot/compose/fallback.py:6001 `a_heavy_news(memory=None)` (never called on the trace corpus)
pub fn a_heavy_news(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:a_heavy_news")
}

/// Python: bot/compose/fallback.py:6021 `a_loss_in_the_thread(memory=None)` (never called on the trace corpus)
pub fn a_loss_in_the_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:a_loss_in_the_thread")
}

/// Python: bot/compose/fallback.py:6052 `a_verdict_stands_down(memory=None)`
pub fn a_verdict_stands_down(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:a_verdict_stands_down")
}

/// Python: bot/compose/fallback.py:6093 `reads_as_a_verdict(text, memory=None)`
pub fn reads_as_a_verdict(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:reads_as_a_verdict")
}

/// Python: bot/compose/fallback.py:6122 `_reaction_pool(clean, memory=None)`
pub fn _reaction_pool(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_reaction_pool")
}

/// Python: bot/compose/fallback.py:6148 `reaction_line(text, memory=None)` (never called on the trace corpus)
pub fn reaction_line(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.fallback:reaction_line")
}

/// Python: bot/compose/fallback.py:6247 `_a_thing_done(clean)` (never called on the trace corpus)
pub fn _a_thing_done(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_thing_done")
}

/// Python: bot/compose/fallback.py:6252 `_thing_done_shape(clean)` (never called on the trace corpus)
pub fn _thing_done_shape(clean: &str) -> Value {
    crate::unported!("bot.compose.fallback:_thing_done_shape")
}

/// Python: bot/compose/fallback.py:6257 `_thing_done_pool(clean)`
pub fn _thing_done_pool(clean: &str) -> Option<Vec<String>> {
    crate::unported!("bot.compose.fallback:_thing_done_pool")
}

/// Python: bot/compose/fallback.py:6305 `_tail_complains(clean)`
pub fn _tail_complains(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_tail_complains")
}

/// Python: bot/compose/fallback.py:6319 `_thing_done_scan(clean)`
pub fn _thing_done_scan(clean: &str) -> Value {
    crate::unported!("bot.compose.fallback:_thing_done_scan")
}

/// Python: bot/compose/fallback.py:6477 `_a_thing_after(rest)`
pub fn _a_thing_after(rest: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_thing_after")
}

/// Python: bot/compose/fallback.py:6494 `_done_ing()` (never called on the trace corpus)
pub fn _done_ing() -> Value {
    crate::unported!("bot.compose.fallback:_done_ing")
}

/// Python: bot/compose/fallback.py:6568 `_a_knock_on_oneself(clean)`
pub fn _a_knock_on_oneself(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_knock_on_oneself")
}

/// Python: bot/compose/fallback.py:6591 `_a_knock_into_a_thing(clean)`
pub fn _a_knock_into_a_thing(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_knock_into_a_thing")
}

/// Python: bot/compose/fallback.py:6691 `_a_mishap_to_ask_about(text)`
pub fn _a_mishap_to_ask_about(text: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_mishap_to_ask_about")
}

/// Python: bot/compose/fallback.py:6707 `_a_mishap(clean, words)`
pub fn _a_mishap(clean: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.fallback:_a_mishap")
}

/// Python: bot/compose/fallback.py:6831 `the_dislike_ask(clean, memory)`
pub fn the_dislike_ask(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:the_dislike_ask")
}

/// Python: bot/compose/fallback.py:6862 `_own_feeling(clean)`
pub fn _own_feeling(clean: &str) {
    crate::unported!("bot.compose.fallback:_own_feeling")
}

/// Python: bot/compose/fallback.py:6882 `_summons_pool(summons, line='')` (never called on the trace corpus)
pub fn _summons_pool(summons: &Value, line: &str) -> Value {
    crate::unported!("bot.compose.fallback:_summons_pool")
}

/// Python: bot/compose/fallback.py:7011 `_a_promise_that_keeps_moving(clean)`
pub fn _a_promise_that_keeps_moving(clean: &str) {
    crate::unported!("bot.compose.fallback:_a_promise_that_keeps_moving")
}

/// Python: bot/compose/fallback.py:7087 `_reads_as_a_judgement(text)`
pub fn _reads_as_a_judgement(text: &str) -> bool {
    crate::unported!("bot.compose.fallback:_reads_as_a_judgement")
}

/// Python: bot/compose/fallback.py:7111 `_a_doable(clean, memory)`
pub fn _a_doable(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_a_doable")
}

/// Python: bot/compose/fallback.py:7121 `a_doable(text, memory)`
pub fn a_doable(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:a_doable")
}

/// Python: bot/compose/fallback.py:7133 `reads_as_a_judgement(text)`
pub fn reads_as_a_judgement(text: &str) -> bool {
    crate::unported!("bot.compose.fallback:reads_as_a_judgement")
}

/// Python: bot/compose/fallback.py:7268 `reads_as_a_change_of_medicine(text)`
pub fn reads_as_a_change_of_medicine(text: &str) -> bool {
    crate::unported!("bot.compose.fallback:reads_as_a_change_of_medicine")
}

/// Python: bot/compose/fallback.py:7345 `_plan_reported(clean, mood)`
pub fn _plan_reported(clean: &str, mood: &str) -> bool {
    crate::unported!("bot.compose.fallback:_plan_reported")
}

/// Python: bot/compose/fallback.py:7411 `a_comparison_of_standing(clean)`
pub fn a_comparison_of_standing(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:a_comparison_of_standing")
}

/// Python: bot/compose/fallback.py:7466 `_a_relationship_ending(clean)`
pub fn _a_relationship_ending(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_relationship_ending")
}

/// Python: bot/compose/fallback.py:7586 `_advice_in(last)`
pub fn _advice_in(last: &str) -> bool {
    crate::unported!("bot.compose.fallback:_advice_in")
}

/// Python: bot/compose/fallback.py:7625 `_the_words_in(advice)`
pub fn _the_words_in(advice: &str) -> Vec<String> {
    crate::unported!("bot.compose.fallback:_the_words_in")
}

/// Python: bot/compose/fallback.py:7779 `_asks_about_the_advice(clean, last)`
pub fn _asks_about_the_advice(clean: &str, last: &str) {
    crate::unported!("bot.compose.fallback:_asks_about_the_advice")
}

/// Python: bot/compose/fallback.py:7817 `_feels_better(clean)`
pub fn _feels_better(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_feels_better")
}

/// Python: bot/compose/fallback.py:7848 `_no_relief(memory, last='')` (never called on the trace corpus)
pub fn _no_relief(memory: &Memory, last: &str) -> bool {
    crate::unported!("bot.compose.fallback:_no_relief")
}

/// Python: bot/compose/fallback.py:7874 `_someone_is_arriving(clean)`
pub fn _someone_is_arriving(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_someone_is_arriving")
}

/// Python: bot/compose/fallback.py:8030 `_is_presence_check(clean)`
pub fn _is_presence_check(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_is_presence_check")
}

/// Python: bot/compose/fallback.py:8049 `_the_store_takes_it(clean)` (never called on the trace corpus)
pub fn _the_store_takes_it(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_store_takes_it")
}

/// Python: bot/compose/fallback.py:8102 `_the_answers_take_it(clean)` (never called on the trace corpus)
pub fn _the_answers_take_it(clean: &str) -> Option<bool> {
    crate::unported!("bot.compose.fallback:_the_answers_take_it")
}

/// Python: bot/compose/fallback.py:8166 `_free_tail(tail)` (never called on the trace corpus)
pub fn _free_tail(tail: &str) -> bool {
    crate::unported!("bot.compose.fallback:_free_tail")
}

/// Python: bot/compose/fallback.py:8208 `_choice_advice(clean)`
pub fn _choice_advice(clean: &str) -> String {
    crate::unported!("bot.compose.fallback:_choice_advice")
}

/// Python: bot/compose/fallback.py:8213 `_unframed(clean)`
pub fn _unframed(clean: &str) -> String {
    crate::unported!("bot.compose.fallback:_unframed")
}

/// Python: bot/compose/fallback.py:8234 `_refuses_the_pair(clean)`
pub fn _refuses_the_pair(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_refuses_the_pair")
}

/// Python: bot/compose/fallback.py:8240 `_is_a_choice(clean)`
pub fn _is_a_choice(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_is_a_choice")
}

/// Python: bot/compose/fallback.py:8401 `_an_order_of_work(sides, clean)`
pub fn _an_order_of_work(sides: &Value, clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_an_order_of_work")
}

/// Python: bot/compose/fallback.py:8419 `_choice_sides(clean)`
pub fn _choice_sides(clean: &str) {
    crate::unported!("bot.compose.fallback:_choice_sides")
}

/// Python: bot/compose/fallback.py:8441 `_capital(side)`
pub fn _capital(side: &str) -> String {
    crate::unported!("bot.compose.fallback:_capital")
}

/// Python: bot/compose/fallback.py:8447 `_named(side, memory=None)`
pub fn _named(side: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.fallback:_named")
}

/// Python: bot/compose/fallback.py:8460 `_a_pick(sides, clean, memory=None)`
pub fn _a_pick(sides: &[String], clean: &str, memory: &Memory) -> Vec<String> {
    crate::unported!("bot.compose.fallback:_a_pick")
}

/// Python: bot/compose/fallback.py:8474 `_between_people(sides, clean)`
pub fn _between_people(sides: &[String], clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_between_people")
}

/// Python: bot/compose/fallback.py:8487 `_choice_light(clean, memory=None)`
pub fn _choice_light(clean: &str, memory: &Memory) -> Vec<String> {
    crate::unported!("bot.compose.fallback:_choice_light")
}

/// Python: bot/compose/fallback.py:8499 `_choice_pool(clean, memory=None)`
pub fn _choice_pool(clean: &str, memory: &Memory) -> Vec<String> {
    crate::unported!("bot.compose.fallback:_choice_pool")
}

/// Python: bot/compose/fallback.py:8535 `_their_own_phrase(clean, things)`
pub fn _their_own_phrase(clean: &str, things: &[String]) {
    crate::unported!("bot.compose.fallback:_their_own_phrase")
}

/// Python: bot/compose/fallback.py:8593 `_a_person_is_the_thing(clean)`
pub fn _a_person_is_the_thing(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_person_is_the_thing")
}

/// Python: bot/compose/fallback.py:8711 `_their_own_bill(clean)`
pub fn _their_own_bill(clean: &str) {
    crate::unported!("bot.compose.fallback:_their_own_bill")
}

/// Python: bot/compose/fallback.py:8720 `_their_own_points(clean)`
pub fn _their_own_points(clean: &str) {
    crate::unported!("bot.compose.fallback:_their_own_points")
}

/// Python: bot/compose/fallback.py:8733 `_an_office_record(clean, asking)`
pub fn _an_office_record(clean: &str, asking: bool) {
    crate::unported!("bot.compose.fallback:_an_office_record")
}

/// Python: bot/compose/fallback.py:8763 `_their_own_record(clean, asking)`
pub fn _their_own_record(clean: &str, asking: bool) {
    crate::unported!("bot.compose.fallback:_their_own_record")
}

/// Python: bot/compose/fallback.py:8852 `_their_own_week(clean)`
pub fn _their_own_week(clean: &str) {
    crate::unported!("bot.compose.fallback:_their_own_week")
}

/// Python: bot/compose/fallback.py:8906 `_names_place(clean, places)`
pub fn _names_place(clean: &str, places: &[String]) -> bool {
    crate::unported!("bot.compose.fallback:_names_place")
}

/// Python: bot/compose/fallback.py:8918 `_their_contact_number(clean, asking)`
pub fn _their_contact_number(clean: &str, asking: bool) {
    crate::unported!("bot.compose.fallback:_their_contact_number")
}

/// Python: bot/compose/fallback.py:8966 `_their_own_day(clean)`
pub fn _their_own_day(clean: &str) {
    crate::unported!("bot.compose.fallback:_their_own_day")
}

/// Python: bot/compose/fallback.py:8999 `_a_bare_wear_ask(clean)` (never called on the trace corpus)
pub fn _a_bare_wear_ask(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_bare_wear_ask")
}

/// Python: bot/compose/fallback.py:9080 `_asks_for_something(text)` (never called on the trace corpus)
pub fn _asks_for_something(text: &str) -> bool {
    crate::unported!("bot.compose.fallback:_asks_for_something")
}

/// Python: bot/compose/fallback.py:9086 `_reads_as_why(clean)`
pub fn _reads_as_why(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_reads_as_why")
}

/// Python: bot/compose/fallback.py:9098 `_is_about_the_persons_own_life(clean)`
pub fn _is_about_the_persons_own_life(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_is_about_the_persons_own_life")
}

/// Python: bot/compose/fallback.py:9103 `_in_a_care_thread(memory)`
pub fn _in_a_care_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_in_a_care_thread")
}

/// Python: bot/compose/fallback.py:9121 `_a_lean_only_line(clean)`
pub fn _a_lean_only_line(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_lean_only_line")
}

/// Python: bot/compose/fallback.py:9126 `_a_bare_pick(clean)`
pub fn _a_bare_pick(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_bare_pick")
}

/// Python: bot/compose/fallback.py:9142 `_the_side_taken(reply, sides)`
pub fn _the_side_taken(reply: &str, sides: Option<&[String]>) {
    crate::unported!("bot.compose.fallback:_the_side_taken")
}

/// Python: bot/compose/fallback.py:9161 `_picked_a_side(reply, sides)`
pub fn _picked_a_side(reply: &str, sides: Option<&[String]>) -> bool {
    crate::unported!("bot.compose.fallback:_picked_a_side")
}

/// Python: bot/compose/fallback.py:9172 `_offered(memory)`
pub fn _offered(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.compose.fallback:_offered")
}

/// Python: bot/compose/fallback.py:9212 `_titles(said)`
pub fn _titles(said: &str) -> Value {
    crate::unported!("bot.compose.fallback:_titles")
}

/// Python: bot/compose/fallback.py:9232 `_names_in(said)` (never called on the trace corpus)
pub fn _names_in(said: &str) -> Value {
    crate::unported!("bot.compose.fallback:_names_in")
}

/// Python: bot/compose/fallback.py:9237 `_a_named_side(after)` (never called on the trace corpus)
pub fn _a_named_side(after: &str) -> Value {
    crate::unported!("bot.compose.fallback:_a_named_side")
}

/// Python: bot/compose/fallback.py:9286 `_a_trade_off(memory)` (never called on the trace corpus)
pub fn _a_trade_off(memory: &Memory) -> Value {
    crate::unported!("bot.compose.fallback:_a_trade_off")
}

/// Python: bot/compose/fallback.py:9328 `_is_a_body_or_care_trade_off(said)` (never called on the trace corpus)
pub fn _is_a_body_or_care_trade_off(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_is_a_body_or_care_trade_off")
}

/// Python: bot/compose/fallback.py:9341 `_a_leaning_side(clean, memory)`
pub fn _a_leaning_side(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_a_leaning_side")
}

/// Python: bot/compose/fallback.py:9364 `_my_pick(clean, memory)`
pub fn _my_pick(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_my_pick")
}

/// Python: bot/compose/fallback.py:9574 `_the_opening_before(match)` (never called on the trace corpus)
pub fn _the_opening_before(r#match: &Value) -> Value {
    crate::unported!("bot.compose.fallback:_the_opening_before")
}

/// Python: bot/compose/fallback.py:9582 `_a_gerund(word)` (never called on the trace corpus)
pub fn _a_gerund(word: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_gerund")
}

/// Python: bot/compose/fallback.py:9598 `_a_plan_said_back(phrase)` (never called on the trace corpus)
pub fn _a_plan_said_back(phrase: &str) -> Value {
    crate::unported!("bot.compose.fallback:_a_plan_said_back")
}

/// Python: bot/compose/fallback.py:9605 `_a_plan_phrase(said)` (never called on the trace corpus)
pub fn _a_plan_phrase(said: &str) -> Value {
    crate::unported!("bot.compose.fallback:_a_plan_phrase")
}

/// Python: bot/compose/fallback.py:9646 `_yours(phrase)` (never called on the trace corpus)
pub fn _yours(phrase: &str) -> Value {
    crate::unported!("bot.compose.fallback:_yours")
}

/// Python: bot/compose/fallback.py:9651 `_the_plan_on_the_table(memory)` (never called on the trace corpus)
pub fn _the_plan_on_the_table(memory: &Memory) -> Value {
    crate::unported!("bot.compose.fallback:_the_plan_on_the_table")
}

/// Python: bot/compose/fallback.py:9656 `_the_plan_is_the_last_thing_said(plan_line, memory)` (never called on the trace corpus)
pub fn _the_plan_is_the_last_thing_said(plan_line: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_the_plan_is_the_last_thing_said")
}

/// Python: bot/compose/fallback.py:9687 `_the_plan_and_its_line(memory)` (never called on the trace corpus)
pub fn _the_plan_and_its_line(memory: &Memory) -> Value {
    crate::unported!("bot.compose.fallback:_the_plan_and_its_line")
}

/// Python: bot/compose/fallback.py:9708 `_the_reply_above_judged(memory, past_a_question=False)` (never called on the trace corpus)
pub fn _the_reply_above_judged(memory: &Memory, past_a_question: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_the_reply_above_judged")
}

/// Python: bot/compose/fallback.py:9767 `_a_place_asked_about(clean, memory)`
pub fn _a_place_asked_about(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_a_place_asked_about")
}

/// Python: bot/compose/fallback.py:9810 `a_verdict_on_their_plan(clean, memory, past_a_question=False)` (never called on the trace corpus)
pub fn a_verdict_on_their_plan(clean: &str, memory: &Memory, past_a_question: &Value) -> Value {
    crate::unported!("bot.compose.fallback:a_verdict_on_their_plan")
}

/// Python: bot/compose/fallback.py:9822 `_a_verdict_on_their_plan(clean, memory, past_a_question=False)`
pub fn _a_verdict_on_their_plan(clean: &str, memory: &Memory, past_a_question: bool) {
    crate::unported!("bot.compose.fallback:_a_verdict_on_their_plan")
}

/// Python: bot/compose/fallback.py:9953 `_their_favourite_said(thing, memory)`
pub fn _their_favourite_said(thing: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_their_favourite_said")
}

/// Python: bot/compose/fallback.py:9968 `_their_favourite_back(thing, value)` (never called on the trace corpus)
pub fn _their_favourite_back(thing: &str, value: &Value) -> Value {
    crate::unported!("bot.compose.fallback:_their_favourite_back")
}

/// Python: bot/compose/fallback.py:9974 `_taste(clean, memory, was_taste, asking, repaired='')`
pub fn _taste(clean: &str, memory: &Memory, was_taste: bool, asking: bool, repaired: &str) {
    crate::unported!("bot.compose.fallback:_taste")
}

/// Python: bot/compose/fallback.py:10034 `_names_a_venue(clean)`
pub fn _names_a_venue(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_names_a_venue")
}

/// Python: bot/compose/fallback.py:10043 `venue_hours(text, memory=None)`
pub fn venue_hours(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:venue_hours")
}

/// Python: bot/compose/fallback.py:10061 `venue_hours_lines()` (never called on the trace corpus)
pub fn venue_hours_lines() -> Value {
    crate::unported!("bot.compose.fallback:venue_hours_lines")
}

/// Python: bot/compose/fallback.py:10109 `_the_scope_of_a_standing_ask(clean)` (never called on the trace corpus)
pub fn _the_scope_of_a_standing_ask(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_scope_of_a_standing_ask")
}

/// Python: bot/compose/fallback.py:10153 `_the_scores_line()` (never called on the trace corpus)
pub fn _the_scores_line() -> Value {
    crate::unported!("bot.compose.fallback:_the_scores_line")
}

/// Python: bot/compose/fallback.py:10170 `_the_days_news(clean, asking)`
pub fn _the_days_news(clean: &str, asking: bool) {
    crate::unported!("bot.compose.fallback:_the_days_news")
}

/// Python: bot/compose/fallback.py:10275 `_what_no_eyes_already_says(clean)` (never called on the trace corpus)
pub fn _what_no_eyes_already_says(clean: &str) -> Value {
    crate::unported!("bot.compose.fallback:_what_no_eyes_already_says")
}

/// Python: bot/compose/fallback.py:10289 `_a_no_from_the_bot(clean)`
pub fn _a_no_from_the_bot(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_no_from_the_bot")
}

/// Python: bot/compose/fallback.py:10307 `_the_live_thing_named(clean)` (never called on the trace corpus)
pub fn _the_live_thing_named(clean: &str) -> Value {
    crate::unported!("bot.compose.fallback:_the_live_thing_named")
}

/// Python: bot/compose/fallback.py:10317 `_the_bot_says_it_cannot(clean)` (never called on the trace corpus)
pub fn _the_bot_says_it_cannot(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_bot_says_it_cannot")
}

/// Python: bot/compose/fallback.py:10327 `_taste_lines()` (never called on the trace corpus)
pub fn _taste_lines() -> Value {
    crate::unported!("bot.compose.fallback:_taste_lines")
}

/// Python: bot/compose/fallback.py:10352 `reads_as_a_taste(text, memory=None)`
pub fn reads_as_a_taste(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:reads_as_a_taste")
}

/// Python: bot/compose/fallback.py:10397 `_a_trip_is_on_the_table(memory)` (never called on the trace corpus)
pub fn _a_trip_is_on_the_table(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_trip_is_on_the_table")
}

/// Python: bot/compose/fallback.py:10442 `_the_sky_and_nothing_else(clean)`
pub fn _the_sky_and_nothing_else(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_sky_and_nothing_else")
}

/// Python: bot/compose/fallback.py:10497 `_a_road_named_with_traffic(clean)`
pub fn _a_road_named_with_traffic(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_road_named_with_traffic")
}

/// Python: bot/compose/fallback.py:10526 `_their_own_count(clean)`
pub fn _their_own_count(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_their_own_count")
}

/// Python: bot/compose/fallback.py:10555 `_their_own_thing(clean)` (never called on the trace corpus)
pub fn _their_own_thing(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_their_own_thing")
}

/// Python: bot/compose/fallback.py:10561 `_their_own_count_fell(clean)`
pub fn _their_own_count_fell(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_their_own_count_fell")
}

/// Python: bot/compose/fallback.py:10571 `_live(clean, asking, memory=None)`
pub fn _live(clean: &str, asking: bool, memory: &Memory) -> Option<Vec<String>> {
    crate::unported!("bot.compose.fallback:_live")
}

/// Python: bot/compose/fallback.py:10818 `_the_ask_for_words_behind_a_doubt(clean)`
pub fn _the_ask_for_words_behind_a_doubt(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_ask_for_words_behind_a_doubt")
}

/// Python: bot/compose/fallback.py:10839 `_asks_for_words(said, words)`
pub fn _asks_for_words(said: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.fallback:_asks_for_words")
}

/// Python: bot/compose/fallback.py:10890 `_reads_as_greeting(words, text='')`
pub fn _reads_as_greeting(words: &[String], text: &str) -> bool {
    crate::unported!("bot.compose.fallback:_reads_as_greeting")
}

/// Python: bot/compose/fallback.py:10911 `_reads_as_taste(clean, words)`
pub fn _reads_as_taste(clean: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.fallback:_reads_as_taste")
}

/// Python: bot/compose/fallback.py:10920 `_blackout_words(clean, words, said)`
pub fn _blackout_words(clean: &str, words: &[String], said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_blackout_words")
}

/// Python: bot/compose/fallback.py:10936 `_action(clean, memory)`
pub fn _action(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_action")
}

/// Python: bot/compose/fallback.py:11027 `_name_given_before(clean, memory)` (never called on the trace corpus)
pub fn _name_given_before(clean: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.fallback:_name_given_before")
}

/// Python: bot/compose/fallback.py:11071 `_asks_a_name_back(clean, words=())`
pub fn _asks_a_name_back(clean: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.fallback:_asks_a_name_back")
}

/// Python: bot/compose/fallback.py:11099 `_asks_for_the_move(line)`
pub fn _asks_for_the_move(line: &str) -> bool {
    crate::unported!("bot.compose.fallback:_asks_for_the_move")
}

/// Python: bot/compose/fallback.py:11104 `_says_it_already(clean, words, memory)`
pub fn _says_it_already(clean: &str, words: &[String], memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_says_it_already")
}

/// Python: bot/compose/fallback.py:11154 `_names_a_new_thing(word, clean)`
pub fn _names_a_new_thing(word: &str, clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_names_a_new_thing")
}

/// Python: bot/compose/fallback.py:11170 `content_words(text)`
pub fn content_words(text: &str) -> Vec<String> {
    crate::unported!("bot.compose.fallback:content_words")
}

/// Python: bot/compose/fallback.py:11176 `_flat_words(text)`
pub fn _flat_words(text: &str) -> Vec<String> {
    crate::unported!("bot.compose.fallback:_flat_words")
}

/// Python: bot/compose/fallback.py:11181 `memory_bot(memory)` (never called on the trace corpus)
pub fn memory_bot(memory: &Memory) -> String {
    crate::unported!("bot.compose.fallback:memory_bot")
}

/// Python: bot/compose/fallback.py:11187 `says_it_already(text, memory)`
pub fn says_it_already(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:says_it_already")
}

/// Python: bot/compose/fallback.py:11206 `is_the_answer_shorter(reply, last)`
pub fn is_the_answer_shorter(reply: &str, last: &str) -> bool {
    crate::unported!("bot.compose.fallback:is_the_answer_shorter")
}

/// Python: bot/compose/fallback.py:11221 `is_said_it_already(reply)`
pub fn is_said_it_already(reply: &str) -> bool {
    crate::unported!("bot.compose.fallback:is_said_it_already")
}

/// Python: bot/compose/fallback.py:11269 `_points_at_nothing(clean, memory, repaired='')`
pub fn _points_at_nothing(clean: &str, memory: &Memory, repaired: &str) -> bool {
    crate::unported!("bot.compose.fallback:_points_at_nothing")
}

/// Python: bot/compose/fallback.py:11292 `_nothing_to_point_at(memory)`
pub fn _nothing_to_point_at(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_nothing_to_point_at")
}

/// Python: bot/compose/fallback.py:11318 `_a_previous_line_named_something(memory, look_back=3)` (never called on the trace corpus)
pub fn _a_previous_line_named_something(memory: &Memory, look_back: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_a_previous_line_named_something")
}

/// Python: bot/compose/fallback.py:11338 `_offers_to_fix_themselves(clean)` (never called on the trace corpus)
pub fn _offers_to_fix_themselves(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_offers_to_fix_themselves")
}

/// Python: bot/compose/fallback.py:11342 `is_a_forecast(said)`
pub fn is_a_forecast(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:is_a_forecast")
}

/// Python: bot/compose/fallback.py:11358 `_the_forecast_subject(clean, memory)` (never called on the trace corpus)
pub fn _the_forecast_subject(clean: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.fallback:_the_forecast_subject")
}

/// Python: bot/compose/fallback.py:11397 `_without_the_things_named_after_a_body(text)`
pub fn _without_the_things_named_after_a_body(text: &str) -> String {
    crate::unported!("bot.compose.fallback:_without_the_things_named_after_a_body")
}

/// Python: bot/compose/fallback.py:11402 `_whose_body(clean, recent)` (never called on the trace corpus)
pub fn _whose_body(clean: &str, recent: &Value) -> String {
    crate::unported!("bot.compose.fallback:_whose_body")
}

/// Python: bot/compose/fallback.py:11422 `_without_the_forecast_verb(line)`
pub fn _without_the_forecast_verb(line: &str) -> String {
    crate::unported!("bot.compose.fallback:_without_the_forecast_verb")
}

/// Python: bot/compose/fallback.py:11435 `_thread_said(memory, look_back=4)` (never called on the trace corpus)
pub fn _thread_said(memory: &Memory, look_back: &Value) -> String {
    crate::unported!("bot.compose.fallback:_thread_said")
}

/// Python: bot/compose/fallback.py:11441 `_a_body_in_the_thread(memory, look_back=4)` (never called on the trace corpus)
pub fn _a_body_in_the_thread(memory: &Memory, look_back: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_a_body_in_the_thread")
}

/// Python: bot/compose/fallback.py:11462 `_a_hard_patch_in_the_thread(clean, memory, look_back=3)` (never called on the trace corpus)
pub fn _a_hard_patch_in_the_thread(clean: &str, memory: &Memory, look_back: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_a_hard_patch_in_the_thread")
}

/// Python: bot/compose/fallback.py:11470 `_forecast_pool(clean, repaired, memory)`
pub fn _forecast_pool(clean: &str, repaired: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_forecast_pool")
}

/// Python: bot/compose/fallback.py:11532 `_unknown(understood, memory, clean)`
pub fn _unknown(understood: &Understood, memory: &Memory, clean: &str) -> String {
    crate::unported!("bot.compose.fallback:_unknown")
}

/// Python: bot/compose/fallback.py:11585 `_is_a_tips_ask(line)`
pub fn _is_a_tips_ask(line: &str) -> bool {
    crate::unported!("bot.compose.fallback:_is_a_tips_ask")
}

/// Python: bot/compose/fallback.py:11615 `_hedged_recently(memory, look_back=3)`
pub fn _hedged_recently(memory: &Memory, look_back: i64) -> bool {
    crate::unported!("bot.compose.fallback:_hedged_recently")
}

/// Python: bot/compose/fallback.py:11668 `_in_a_pool(text, pools=(NOT_AN_ANSWER,))`
pub fn _in_a_pool(text: &str, pools: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_in_a_pool")
}

/// Python: bot/compose/fallback.py:11732 `_the_thing_alone(thing)` (never called on the trace corpus)
pub fn _the_thing_alone(thing: &str) -> Value {
    crate::unported!("bot.compose.fallback:_the_thing_alone")
}

/// Python: bot/compose/fallback.py:11749 `_what_to_do_about(clean, memory=None)`
pub fn _what_to_do_about(clean: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.fallback:_what_to_do_about")
}

/// Python: bot/compose/fallback.py:11860 `_a_question_about_the_bot(clean, memory=None)`
pub fn _a_question_about_the_bot(clean: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.fallback:_a_question_about_the_bot")
}

/// Python: bot/compose/fallback.py:11891 `_only_the_floor(said)`
pub fn _only_the_floor(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_only_the_floor")
}

/// Python: bot/compose/fallback.py:11921 `_plain(text)`
pub fn _plain(text: &str) -> String {
    crate::unported!("bot.compose.fallback:_plain")
}

/// Python: bot/compose/fallback.py:11931 `_the_words(text)`
pub fn _the_words(text: &str) -> Vec<String> {
    crate::unported!("bot.compose.fallback:_the_words")
}

/// Python: bot/compose/fallback.py:11937 `_an_echo_of(question, sentence)` (never called on the trace corpus)
pub fn _an_echo_of(question: &str, sentence: &str) -> bool {
    crate::unported!("bot.compose.fallback:_an_echo_of")
}

/// Python: bot/compose/fallback.py:11959 `_only_a_reaction(module, said)` (never called on the trace corpus)
pub fn _only_a_reaction(module: &Value, said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_only_a_reaction")
}

/// Python: bot/compose/fallback.py:11968 `_the_answer_in_reply(question, said)` (never called on the trace corpus)
pub fn _the_answer_in_reply(question: &str, said: &str) -> Value {
    crate::unported!("bot.compose.fallback:_the_answer_in_reply")
}

/// Python: bot/compose/fallback.py:11993 `_reads_as_a_claim(said)` (never called on the trace corpus)
pub fn _reads_as_a_claim(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_reads_as_a_claim")
}

/// Python: bot/compose/fallback.py:12029 `_nothing_to_stand_behind(said)`
pub fn _nothing_to_stand_behind(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_nothing_to_stand_behind")
}

/// Python: bot/compose/fallback.py:12048 `_is_a_view_ask(clean)`
pub fn _is_a_view_ask(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_is_a_view_ask")
}

/// Python: bot/compose/fallback.py:12062 `_the_view_asked_for(clean, memory)`
pub fn _the_view_asked_for(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_the_view_asked_for")
}

/// Python: bot/compose/fallback.py:12067 `_after_a_colon(said)` (never called on the trace corpus)
pub fn _after_a_colon(said: &str) -> Value {
    crate::unported!("bot.compose.fallback:_after_a_colon")
}

/// Python: bot/compose/fallback.py:12095 `_is_an_ask_which_bit(said)`
pub fn _is_an_ask_which_bit(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_is_an_ask_which_bit")
}

/// Python: bot/compose/fallback.py:12104 `_is_a_hedge(said)`
pub fn _is_a_hedge(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_is_a_hedge")
}

/// Python: bot/compose/fallback.py:12145 `an_outcome_asked_about_someone(text, memory)`
pub fn an_outcome_asked_about_someone(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:an_outcome_asked_about_someone")
}

/// Python: bot/compose/fallback.py:12174 `is_a_hedge(said)`
pub fn is_a_hedge(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:is_a_hedge")
}

/// Python: bot/compose/fallback.py:12210 `_the_view_of(said)` (never called on the trace corpus)
pub fn _the_view_of(said: &str) -> Value {
    crate::unported!("bot.compose.fallback:_the_view_of")
}

/// Python: bot/compose/fallback.py:12255 `_a_said_claim(first)` (never called on the trace corpus)
pub fn _a_said_claim(first: &Value) -> Value {
    crate::unported!("bot.compose.fallback:_a_said_claim")
}

/// Python: bot/compose/fallback.py:12313 `asks_for_more(text, memory)`
pub fn asks_for_more(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:asks_for_more")
}

/// Python: bot/compose/fallback.py:12332 `_named_recently(named, memory, look_back=3)`
pub fn _named_recently(named: &str, memory: &Memory, look_back: i64) -> bool {
    crate::unported!("bot.compose.fallback:_named_recently")
}

/// Python: bot/compose/fallback.py:12344 `_answered_recently(memory)`
pub fn _answered_recently(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_answered_recently")
}

/// Python: bot/compose/fallback.py:12398 `_only_takes_the_answer(said)`
pub fn _only_takes_the_answer(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_only_takes_the_answer")
}

/// Python: bot/compose/fallback.py:12416 `_told_them_something(said)`
pub fn _told_them_something(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_told_them_something")
}

/// Python: bot/compose/fallback.py:12428 `_the_answer_above(memory)`
pub fn _the_answer_above(memory: &Memory) -> String {
    crate::unported!("bot.compose.fallback:_the_answer_above")
}

/// Python: bot/compose/fallback.py:12448 `_answered_already(memory)`
pub fn _answered_already(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_answered_already")
}

/// Python: bot/compose/fallback.py:12475 `_tips_spoken(text)`
pub fn _tips_spoken(text: &str) -> String {
    crate::unported!("bot.compose.fallback:_tips_spoken")
}

/// Python: bot/compose/fallback.py:12488 `_tips_subject(clean)`
pub fn _tips_subject(clean: &str) -> Option<String> {
    crate::unported!("bot.compose.fallback:_tips_subject")
}

/// Python: bot/compose/fallback.py:12515 `_tips(memory, clean, repaired=None)`
pub fn _tips(memory: &Memory, clean: &str, repaired: &str) -> String {
    crate::unported!("bot.compose.fallback:_tips")
}

/// Python: bot/compose/fallback.py:12625 `_the_figure_in(text)`
pub fn _the_figure_in(text: &str) -> String {
    crate::unported!("bot.compose.fallback:_the_figure_in")
}

/// Python: bot/compose/fallback.py:12631 `_a_price_wants_a_verdict(clean, memory)`
pub fn _a_price_wants_a_verdict(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_a_price_wants_a_verdict")
}

/// Python: bot/compose/fallback.py:12691 `_a_grown_up_asks(clean, words, said)`
pub fn _a_grown_up_asks(clean: &str, words: &[String], said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_grown_up_asks")
}

/// Python: bot/compose/fallback.py:12712 `_who_for(clean, said, words, topic=None)`
pub fn _who_for(clean: &str, said: &str, words: &[String], topic: Option<&str>) -> Option<String> {
    crate::unported!("bot.compose.fallback:_who_for")
}

/// Python: bot/compose/fallback.py:12799 `who_settles(text, memory)` (never called on the trace corpus)
pub fn who_settles(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.fallback:who_settles")
}

/// Python: bot/compose/fallback.py:12821 `_advisor(memory, clean, said, words)`
pub fn _advisor(memory: &Memory, clean: &str, said: &str, words: &[String]) -> Option<String> {
    crate::unported!("bot.compose.fallback:_advisor")
}

/// Python: bot/compose/fallback.py:12859 `_a_bad_stretch_above(memory, look_back=3)` (never called on the trace corpus)
pub fn _a_bad_stretch_above(memory: &Memory, look_back: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_a_bad_stretch_above")
}

/// Python: bot/compose/fallback.py:12877 `_thread_has_a_health_visit(memory, look_back=3)` (never called on the trace corpus)
pub fn _thread_has_a_health_visit(memory: &Memory, look_back: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_thread_has_a_health_visit")
}

/// Python: bot/compose/fallback.py:12887 `_in_a_crisis_thread(memory)`
pub fn _in_a_crisis_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_in_a_crisis_thread")
}

/// Python: bot/compose/fallback.py:12900 `_who_for_the_thread(memory, clean, understood=None)`
pub fn _who_for_the_thread(memory: &Memory, clean: &str, understood: &Understood) {
    crate::unported!("bot.compose.fallback:_who_for_the_thread")
}

/// Python: bot/compose/fallback.py:12961 `_trouble(memory, clean, words)`
pub fn _trouble(memory: &Memory, clean: &str, words: &[String]) {
    crate::unported!("bot.compose.fallback:_trouble")
}

/// Python: bot/compose/fallback.py:12984 `_is_a_form(word, evidence_word)`
pub fn _is_a_form(word: &str, evidence_word: &str) -> bool {
    crate::unported!("bot.compose.fallback:_is_a_form")
}

/// Python: bot/compose/fallback.py:13001 `_names(words, said, evidence)`
pub fn _names(words: &[String], said: &str, evidence: &[String]) -> bool {
    crate::unported!("bot.compose.fallback:_names")
}

/// Python: bot/compose/fallback.py:13016 `_reads_as_question(text, words)`
pub fn _reads_as_question(text: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.fallback:_reads_as_question")
}

/// Python: bot/compose/fallback.py:13037 `_reads_as_a_question_when_repaired(text)`
pub fn _reads_as_a_question_when_repaired(text: &str) -> bool {
    crate::unported!("bot.compose.fallback:_reads_as_a_question_when_repaired")
}

/// Python: bot/compose/fallback.py:13052 `_help_topic(clean, understood)`
pub fn _help_topic(clean: &str, understood: &Understood) {
    crate::unported!("bot.compose.fallback:_help_topic")
}

/// Python: bot/compose/fallback.py:13069 `reads_as_about_itself(text, memory=None)`
pub fn reads_as_about_itself(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:reads_as_about_itself")
}

/// Python: bot/compose/fallback.py:13113 `reads_as_a_tight_week(text, memory=None)`
pub fn reads_as_a_tight_week(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:reads_as_a_tight_week")
}

/// Python: bot/compose/fallback.py:13169 `a_continuation_of_the_tight_week(memory, text)`
pub fn a_continuation_of_the_tight_week(memory: &Memory, text: &str) -> bool {
    crate::unported!("bot.compose.fallback:a_continuation_of_the_tight_week")
}

/// Python: bot/compose/fallback.py:13186 `reads_as_small_talk(text, memory=None)`
pub fn reads_as_small_talk(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:reads_as_small_talk")
}

/// Python: bot/compose/fallback.py:13257 `_a_low_turn_above(memory)`
pub fn _a_low_turn_above(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_low_turn_above")
}

/// Python: bot/compose/fallback.py:13307 `_a_repair_already_spoken_of(memory)` (never called on the trace corpus)
pub fn _a_repair_already_spoken_of(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_repair_already_spoken_of")
}

/// Python: bot/compose/fallback.py:13343 `_self_line(text)`
pub fn _self_line(text: &str) -> String {
    crate::unported!("bot.compose.fallback:_self_line")
}

/// Python: bot/compose/fallback.py:13349 `_a_tally_of_its_own_day(understood)`
pub fn _a_tally_of_its_own_day(understood: &Understood) -> bool {
    crate::unported!("bot.compose.fallback:_a_tally_of_its_own_day")
}

/// Python: bot/compose/fallback.py:13375 `_spelled_out(clean)`
pub fn _spelled_out(clean: &str) -> String {
    crate::unported!("bot.compose.fallback:_spelled_out")
}

/// Python: bot/compose/fallback.py:13383 `_about(clean, understood=None)`
pub fn _about(clean: &str, understood: Option<&Understood>) {
    crate::unported!("bot.compose.fallback:_about")
}

/// Python: bot/compose/fallback.py:13520 `_names_the_make(clean)`
pub fn _names_the_make(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_names_the_make")
}

/// Python: bot/compose/fallback.py:13526 `about_the_bot(clean)`
pub fn about_the_bot(clean: &str) {
    crate::unported!("bot.compose.fallback:about_the_bot")
}

/// Python: bot/compose/fallback.py:13561 `_a_doubt_about_the_bot(clean)`
pub fn _a_doubt_about_the_bot(clean: &str) {
    crate::unported!("bot.compose.fallback:_a_doubt_about_the_bot")
}

/// Python: bot/compose/fallback.py:13583 `_the_bot_itself(clean)`
pub fn _the_bot_itself(clean: &str) {
    crate::unported!("bot.compose.fallback:_the_bot_itself")
}

/// Python: bot/compose/fallback.py:13603 `_about_itself(clean)`
pub fn _about_itself(clean: &str) {
    crate::unported!("bot.compose.fallback:_about_itself")
}

/// Python: bot/compose/fallback.py:13674 `_burn_line(clean, thread='')`
pub fn _burn_line(clean: &str, thread: &str) -> bool {
    crate::unported!("bot.compose.fallback:_burn_line")
}

/// Python: bot/compose/fallback.py:13762 `_the_hazard_is_gas(line)`
pub fn _the_hazard_is_gas(line: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_hazard_is_gas")
}

/// Python: bot/compose/fallback.py:13768 `_goes_back_for(clean, thread='')`
pub fn _goes_back_for(clean: &str, thread: &str) -> bool {
    crate::unported!("bot.compose.fallback:_goes_back_for")
}

/// Python: bot/compose/fallback.py:13775 `_names_a_new_part(words, clean, thread)` (never called on the trace corpus)
pub fn _names_a_new_part(words: &[String], clean: &str, thread: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_names_a_new_part")
}

/// Python: bot/compose/fallback.py:13794 `_keeps_thread(understood, clean, thread='', crisis=False)`
pub fn _keeps_thread(understood: &Understood, clean: &str, thread: &str, crisis: bool) -> bool {
    crate::unported!("bot.compose.fallback:_keeps_thread")
}

/// Python: bot/compose/fallback.py:13905 `_reads_as_fear(understood, clean)`
pub fn _reads_as_fear(understood: &Understood, clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_reads_as_fear")
}

/// Python: bot/compose/fallback.py:13927 `_only_a_worry(clean)` (never called on the trace corpus)
pub fn _only_a_worry(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_only_a_worry")
}

/// Python: bot/compose/fallback.py:13947 `_about_somebody_else(safety, memory)`
pub fn _about_somebody_else(safety: &Value, memory: &Memory) {
    crate::unported!("bot.compose.fallback:_about_somebody_else")
}

/// Python: bot/compose/fallback.py:13999 `_wrote_for_them(pool, forms)` (never called on the trace corpus)
pub fn _wrote_for_them(pool: &Value, forms: &[String]) -> Value {
    crate::unported!("bot.compose.fallback:_wrote_for_them")
}

/// Python: bot/compose/fallback.py:14053 `_red_flags_in(line)`
pub fn _red_flags_in(line: &str) -> Vec<String> {
    crate::unported!("bot.compose.fallback:_red_flags_in")
}

/// Python: bot/compose/fallback.py:14087 `_flag_events_in(line)`
pub fn _flag_events_in(line: &str) -> Vec<String> {
    crate::unported!("bot.compose.fallback:_flag_events_in")
}

/// Python: bot/compose/fallback.py:14140 `_the_threshold_words(sentence)`
pub fn _the_threshold_words(sentence: &str) -> Vec<String> {
    crate::unported!("bot.compose.fallback:_the_threshold_words")
}

/// Python: bot/compose/fallback.py:14154 `_it_is_the_answer_word(line, word)` (never called on the trace corpus)
pub fn _it_is_the_answer_word(line: &str, word: &str) -> bool {
    crate::unported!("bot.compose.fallback:_it_is_the_answer_word")
}

/// Python: bot/compose/fallback.py:14166 `_the_flags_still_standing(line)`
pub fn _the_flags_still_standing(line: &str) -> Vec<String> {
    crate::unported!("bot.compose.fallback:_the_flags_still_standing")
}

/// Python: bot/compose/fallback.py:14188 `_the_threshold_was_crossed(last, clean, turns)`
pub fn _the_threshold_was_crossed(last: &str, clean: &str, turns: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_the_threshold_was_crossed")
}

/// Python: bot/compose/fallback.py:14225 `_flag_stem(word)` (never called on the trace corpus)
pub fn _flag_stem(word: &str) -> Value {
    crate::unported!("bot.compose.fallback:_flag_stem")
}

/// Python: bot/compose/fallback.py:14240 `_it_is_this_files_call(said)` (never called on the trace corpus)
pub fn _it_is_this_files_call(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_it_is_this_files_call")
}

/// Python: bot/compose/fallback.py:14249 `_a_newer_answer_asked_them_to_call(last, turns)` (never called on the trace corpus)
pub fn _a_newer_answer_asked_them_to_call(last: &str, turns: &[Turn]) -> bool {
    crate::unported!("bot.compose.fallback:_a_newer_answer_asked_them_to_call")
}

/// Python: bot/compose/fallback.py:14278 `_the_route_the_answer_named(last)` (never called on the trace corpus)
pub fn _the_route_the_answer_named(last: &str) -> Value {
    crate::unported!("bot.compose.fallback:_the_route_the_answer_named")
}

/// Python: bot/compose/fallback.py:14296 `_the_answer_asked_them_to_call(last)`
pub fn _the_answer_asked_them_to_call(last: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_answer_asked_them_to_call")
}

/// Python: bot/compose/fallback.py:14321 `_raises_the_stakes(clean, last)` (never called on the trace corpus)
pub fn _raises_the_stakes(clean: &str, last: &str) -> bool {
    crate::unported!("bot.compose.fallback:_raises_the_stakes")
}

/// Python: bot/compose/fallback.py:14331 `_releasable(memory)`
pub fn _releasable(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_releasable")
}

/// Python: bot/compose/fallback.py:14350 `_named_draft(memory, text, words, clean)`
pub fn _named_draft(memory: &Memory, text: &str, words: &[String], clean: &str) {
    crate::unported!("bot.compose.fallback:_named_draft")
}

/// Python: bot/compose/fallback.py:14369 `_after_safety(memory, clean, understood=None)`
pub fn _after_safety(memory: &Memory, clean: &str, understood: Option<&Understood>) {
    crate::unported!("bot.compose.fallback:_after_safety")
}

/// Python: bot/compose/fallback.py:14739 `_is_a_hazard(last)` (never called on the trace corpus)
pub fn _is_a_hazard(last: &str) -> bool {
    crate::unported!("bot.compose.fallback:_is_a_hazard")
}

/// Python: bot/compose/fallback.py:14752 `_the_safety_cards(turns)` (never called on the trace corpus)
pub fn _the_safety_cards(turns: &[Turn]) -> Vec<String> {
    crate::unported!("bot.compose.fallback:_the_safety_cards")
}

/// Python: bot/compose/fallback.py:14763 `_somebody_is_coming(turns, clean)` (never called on the trace corpus)
pub fn _somebody_is_coming(turns: &[Turn], clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_somebody_is_coming")
}

/// Python: bot/compose/fallback.py:14772 `_the_answer_was_an_abuse_card(last)`
pub fn _the_answer_was_an_abuse_card(last: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_answer_was_an_abuse_card")
}

/// Python: bot/compose/fallback.py:14782 `_thread_was_closed(turns)` (never called on the trace corpus)
pub fn _thread_was_closed(turns: &[Turn]) -> bool {
    crate::unported!("bot.compose.fallback:_thread_was_closed")
}

/// Python: bot/compose/fallback.py:14796 `_hold_pool(turns, clean, last, done_report=False)` (never called on the trace corpus)
pub fn _hold_pool(turns: &[Turn], clean: &str, last: &str, done_report: &Value) -> Value {
    crate::unported!("bot.compose.fallback:_hold_pool")
}

/// Python: bot/compose/fallback.py:14824 `_a_call_is_known(turns, clean, last)` (never called on the trace corpus)
pub fn _a_call_is_known(turns: &[Turn], clean: &str, last: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_call_is_known")
}

/// Python: bot/compose/fallback.py:14851 `_the_call_is_conditional(text)` (never called on the trace corpus)
pub fn _the_call_is_conditional(text: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_call_is_conditional")
}

/// Python: bot/compose/fallback.py:14860 `_already_rang(turns)` (never called on the trace corpus)
pub fn _already_rang(turns: &[Turn]) -> bool {
    crate::unported!("bot.compose.fallback:_already_rang")
}

/// Python: bot/compose/fallback.py:14866 `_thanks_carries_emergency(words, clean)` (never called on the trace corpus)
pub fn _thanks_carries_emergency(words: &[String], clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_thanks_carries_emergency")
}

/// Python: bot/compose/fallback.py:14911 `a_date_of_ours_forgotten(clean)`
pub fn a_date_of_ours_forgotten(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:a_date_of_ours_forgotten")
}

/// Python: bot/compose/fallback.py:14922 `_impact_said(clean)`
pub fn _impact_said(clean: &str) -> Vec<String> {
    crate::unported!("bot.compose.fallback:_impact_said")
}

/// Python: bot/compose/fallback.py:14948 `_grief_thread(memory)`
pub fn _grief_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_grief_thread")
}

/// Python: bot/compose/fallback.py:14963 `_grievance_thread(memory)`
pub fn _grievance_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_grievance_thread")
}

/// Python: bot/compose/fallback.py:14981 `_someone_else_minding(clean, words, after=False)`
pub fn _someone_else_minding(clean: &str, words: &[String], after: bool) -> bool {
    crate::unported!("bot.compose.fallback:_someone_else_minding")
}

/// Python: bot/compose/fallback.py:15035 `_short_ack(words)`
pub fn _short_ack(words: &[String]) -> bool {
    crate::unported!("bot.compose.fallback:_short_ack")
}

/// Python: bot/compose/fallback.py:15135 `_hard_week_in_the_thread(memory)`
pub fn _hard_week_in_the_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_hard_week_in_the_thread")
}

/// Python: bot/compose/fallback.py:15156 `_a_verdict(clean, memory=None)`
pub fn _a_verdict(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_verdict")
}

/// Python: bot/compose/fallback.py:15260 `_dreading(clean)`
pub fn _dreading(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_dreading")
}

/// Python: bot/compose/fallback.py:15270 `_wants_it_over(clean)`
pub fn _wants_it_over(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_wants_it_over")
}

/// Python: bot/compose/fallback.py:15315 `_a_day_ahead(clean)`
pub fn _a_day_ahead(clean: &str) -> Option<Vec<String>> {
    crate::unported!("bot.compose.fallback:_a_day_ahead")
}

/// Python: bot/compose/fallback.py:15336 `_the_wait_is_for_someone_to_turn_up(text)` (never called on the trace corpus)
pub fn _the_wait_is_for_someone_to_turn_up(text: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_wait_is_for_someone_to_turn_up")
}

/// Python: bot/compose/fallback.py:15361 `_looking_forward(clean)`
pub fn _looking_forward(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_looking_forward")
}

/// Python: bot/compose/fallback.py:15373 `_a_good_time(clean, memory=None)`
pub fn _a_good_time(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_good_time")
}

/// Python: bot/compose/fallback.py:15458 `_something_funny(clean, memory=None)`
pub fn _something_funny(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_something_funny")
}

/// Python: bot/compose/fallback.py:15554 `_the_gap_is_all_there_is(clean)` (never called on the trace corpus)
pub fn _the_gap_is_all_there_is(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_the_gap_is_all_there_is")
}

/// Python: bot/compose/fallback.py:15608 `reads_as_pause(clean)`
pub fn reads_as_pause(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:reads_as_pause")
}

/// Python: bot/compose/fallback.py:15637 `_came_back(clean)`
pub fn _came_back(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_came_back")
}

/// Python: bot/compose/fallback.py:15670 `_a_chat_behind_it(memory)`
pub fn _a_chat_behind_it(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_chat_behind_it")
}

/// Python: bot/compose/fallback.py:15683 `nothing_said_yet(memory)`
pub fn nothing_said_yet(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:nothing_said_yet")
}

/// Python: bot/compose/fallback.py:15688 `nothing_yet_line()` (never called on the trace corpus)
pub fn nothing_yet_line() -> Value {
    crate::unported!("bot.compose.fallback:nothing_yet_line")
}

/// Python: bot/compose/fallback.py:15693 `brings_nothing_of_its_own(text)`
pub fn brings_nothing_of_its_own(text: &str) -> bool {
    crate::unported!("bot.compose.fallback:brings_nothing_of_its_own")
}

/// Python: bot/compose/fallback.py:15707 `assumes_an_earlier_turn(reply)`
pub fn assumes_an_earlier_turn(reply: &str) -> bool {
    crate::unported!("bot.compose.fallback:assumes_an_earlier_turn")
}

/// Python: bot/compose/fallback.py:15723 `reads_as_a_return(clean)`
pub fn reads_as_a_return(clean: &str) {
    crate::unported!("bot.compose.fallback:reads_as_a_return")
}

/// Python: bot/compose/fallback.py:15768 `_only_manners(clean, allowed)`
pub fn _only_manners(clean: &str, allowed: &[String]) -> bool {
    crate::unported!("bot.compose.fallback:_only_manners")
}

/// Python: bot/compose/fallback.py:15774 `_sorry_for_asking(clean)`
pub fn _sorry_for_asking(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_sorry_for_asking")
}

/// Python: bot/compose/fallback.py:15803 `_am_i_a_nuisance(clean)`
pub fn _am_i_a_nuisance(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_am_i_a_nuisance")
}

/// Python: bot/compose/fallback.py:15859 `_thanks_then_said(clean)`
pub fn _thanks_then_said(clean: &str) -> String {
    crate::unported!("bot.compose.fallback:_thanks_then_said")
}

/// Python: bot/compose/fallback.py:15878 `_thanks_news_went_badly(clean)`
pub fn _thanks_news_went_badly(clean: &str) -> String {
    crate::unported!("bot.compose.fallback:_thanks_news_went_badly")
}

/// Python: bot/compose/fallback.py:15914 `_acknowledges_alone(clean)`
pub fn _acknowledges_alone(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_acknowledges_alone")
}

/// Python: bot/compose/fallback.py:15924 `_thanks_with_more(clean)`
pub fn _thanks_with_more(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_thanks_with_more")
}

/// Python: bot/compose/fallback.py:15935 `reading(text, memory)`
pub fn reading(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.fallback:reading")
}

/// Python: bot/compose/fallback.py:15975 `_advice_thread_is_about_the_body(memory)`
pub fn _advice_thread_is_about_the_body(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_advice_thread_is_about_the_body")
}

/// Python: bot/compose/fallback.py:15990 `_a_step_word_in(words, steps)` (never called on the trace corpus)
pub fn _a_step_word_in(words: &[String], steps: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_a_step_word_in")
}

/// Python: bot/compose/fallback.py:15998 `_a_report_on_the_advice(clean, words, about_the_body=True, the_lot_ok=False, steps=NO_STEP)`
pub fn _a_report_on_the_advice(clean: &str, words: &[String], about_the_body: bool, the_lot_ok: bool, steps: &[String]) {
    crate::unported!("bot.compose.fallback:_a_report_on_the_advice")
}

/// Python: bot/compose/fallback.py:16055 `_the_outcome_of_the_advice(memory, clean, words)`
pub fn _the_outcome_of_the_advice(memory: &Memory, clean: &str, words: &[String]) {
    crate::unported!("bot.compose.fallback:_the_outcome_of_the_advice")
}

/// Python: bot/compose/fallback.py:16092 `the_advice(memory)`
pub fn the_advice(memory: &Memory) -> String {
    crate::unported!("bot.compose.fallback:the_advice")
}

/// Python: bot/compose/fallback.py:16102 `reads_as_a_report(text, memory)`
pub fn reads_as_a_report(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:reads_as_a_report")
}

/// Python: bot/compose/fallback.py:16115 `the_report_reply(text, memory)` (never called on the trace corpus)
pub fn the_report_reply(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.fallback:the_report_reply")
}

/// Python: bot/compose/fallback.py:16132 `a_report_that_says_something(text, memory)` (never called on the trace corpus)
pub fn a_report_that_says_something(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:a_report_that_says_something")
}

/// Python: bot/compose/fallback.py:16146 `_after_advice(memory, clean, words)`
pub fn _after_advice(memory: &Memory, clean: &str, words: &[String]) {
    crate::unported!("bot.compose.fallback:_after_advice")
}

/// Python: bot/compose/fallback.py:16161 `_the_answer_above_already_says_what_to_do(turns, memory)` (never called on the trace corpus)
pub fn _the_answer_above_already_says_what_to_do(turns: &[Turn], memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_the_answer_above_already_says_what_to_do")
}

/// Python: bot/compose/fallback.py:16176 `_a_thread_about_how_somebody_treats_them(turns)` (never called on the trace corpus)
pub fn _a_thread_about_how_somebody_treats_them(turns: &[Turn]) -> bool {
    crate::unported!("bot.compose.fallback:_a_thread_about_how_somebody_treats_them")
}

/// Python: bot/compose/fallback.py:16191 `_an_answer_above(turns)` (never called on the trace corpus)
pub fn _an_answer_above(turns: &[Turn]) -> bool {
    crate::unported!("bot.compose.fallback:_an_answer_above")
}

/// Python: bot/compose/fallback.py:16201 `_a_refusal_to_answer(clean, memory, words=None)`
pub fn _a_refusal_to_answer(clean: &str, memory: &Memory, words: &[String]) {
    crate::unported!("bot.compose.fallback:_a_refusal_to_answer")
}

/// Python: bot/compose/fallback.py:16264 `reads_as_a_refusal(text, memory)` (never called on the trace corpus)
pub fn reads_as_a_refusal(text: &str, memory: &Memory) -> Option<bool> {
    crate::unported!("bot.compose.fallback:reads_as_a_refusal")
}

/// Python: bot/compose/fallback.py:16273 `_the_next_move_after_the_advice(memory, clean, words)`
pub fn _the_next_move_after_the_advice(memory: &Memory, clean: &str, words: &[String]) {
    crate::unported!("bot.compose.fallback:_the_next_move_after_the_advice")
}

/// Python: bot/compose/fallback.py:16298 `_a_care_thread(memory)` (never called on the trace corpus)
pub fn _a_care_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_care_thread")
}

/// Python: bot/compose/fallback.py:16306 `in_a_safety_thread(memory)`
pub fn in_a_safety_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:in_a_safety_thread")
}

/// Python: bot/compose/fallback.py:16316 `_thread_answer(memory)`
pub fn _thread_answer(memory: &Memory) -> String {
    crate::unported!("bot.compose.fallback:_thread_answer")
}

/// Python: bot/compose/fallback.py:16362 `asks_about_the_last_answer(line, last, own=())` (never called on the trace corpus)
pub fn asks_about_the_last_answer(line: &str, last: &str, own: &str) -> bool {
    crate::unported!("bot.compose.fallback:asks_about_the_last_answer")
}

/// Python: bot/compose/fallback.py:16420 `asks_about_a_care_answer(line, last, memory, own=())` (never called on the trace corpus)
pub fn asks_about_a_care_answer(line: &str, last: &str, memory: &Memory, own: &str) -> bool {
    crate::unported!("bot.compose.fallback:asks_about_a_care_answer")
}

/// Python: bot/compose/fallback.py:16442 `how_much_of_the_answer_kept(reply, memory)` (never called on the trace corpus)
pub fn how_much_of_the_answer_kept(reply: &str, memory: &Memory) -> i64 {
    crate::unported!("bot.compose.fallback:how_much_of_the_answer_kept")
}

/// Python: bot/compose/fallback.py:16471 `_words_kept(reply, answer)` (never called on the trace corpus)
pub fn _words_kept(reply: &str, answer: &str) -> i64 {
    crate::unported!("bot.compose.fallback:_words_kept")
}

/// Python: bot/compose/fallback.py:16492 `careful(text, memory)`
pub fn careful(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback:careful")
}

/// Python: bot/compose/fallback.py:16505 `_in_a_loss_thread(memory)`
pub fn _in_a_loss_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_in_a_loss_thread")
}

/// Python: bot/compose/fallback.py:16555 `_a_loss_in(text)` (never called on the trace corpus)
pub fn _a_loss_in(text: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_loss_in")
}

/// Python: bot/compose/fallback.py:16585 `_dead_is_the_only_death_word(said)` (never called on the trace corpus)
pub fn _dead_is_the_only_death_word(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_dead_is_the_only_death_word")
}

/// Python: bot/compose/fallback.py:16593 `_we_condoled(line)` (never called on the trace corpus)
pub fn _we_condoled(line: &str) -> bool {
    crate::unported!("bot.compose.fallback:_we_condoled")
}

/// Python: bot/compose/fallback.py:16605 `_a_thing_stopped(clean)` (never called on the trace corpus)
pub fn _a_thing_stopped(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_thing_stopped")
}

/// Python: bot/compose/fallback.py:16610 `_last_turn(memory)` (never called on the trace corpus)
pub fn _last_turn(memory: &Memory) -> Value {
    crate::unported!("bot.compose.fallback:_last_turn")
}

/// Python: bot/compose/fallback.py:16616 `_the_last_line_was_bad_news(memory)` (never called on the trace corpus)
pub fn _the_last_line_was_bad_news(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_the_last_line_was_bad_news")
}

/// Python: bot/compose/fallback.py:16626 `_the_last_line_was_a_fault(memory)` (never called on the trace corpus)
pub fn _the_last_line_was_a_fault(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_the_last_line_was_a_fault")
}

/// Python: bot/compose/fallback.py:16641 `_a_loss_recently(memory)` (never called on the trace corpus)
pub fn _a_loss_recently(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_loss_recently")
}

/// Python: bot/compose/fallback.py:16671 `_the_last_line_was_heavy(memory)` (never called on the trace corpus)
pub fn _the_last_line_was_heavy(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_the_last_line_was_heavy")
}

/// Python: bot/compose/fallback.py:16691 `_in_sudden_loss_thread(memory)`
pub fn _in_sudden_loss_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_in_sudden_loss_thread")
}

/// Python: bot/compose/fallback.py:16701 `_reads_as_illness(clean)` (never called on the trace corpus)
pub fn _reads_as_illness(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_reads_as_illness")
}

/// Python: bot/compose/fallback.py:16734 `_in_an_aim(clean, word)`
pub fn _in_an_aim(clean: &str, word: &str) -> bool {
    crate::unported!("bot.compose.fallback:_in_an_aim")
}

/// Python: bot/compose/fallback.py:16753 `_a_mood_word(words, word)` (never called on the trace corpus)
pub fn _a_mood_word(words: &[String], word: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_mood_word")
}

/// Python: bot/compose/fallback.py:16812 `_is_rumination(clean)`
pub fn _is_rumination(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_is_rumination")
}

/// Python: bot/compose/fallback.py:16832 `_opens_a_question(words)`
pub fn _opens_a_question(words: &[String]) -> bool {
    crate::unported!("bot.compose.fallback:_opens_a_question")
}

/// Python: bot/compose/fallback.py:16901 `is_a_report_reply(said)`
pub fn is_a_report_reply(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:is_a_report_reply")
}

/// Python: bot/compose/fallback.py:16912 `_flat_pools()` (never called on the trace corpus)
pub fn _flat_pools() -> Value {
    crate::unported!("bot.compose.fallback:_flat_pools")
}

