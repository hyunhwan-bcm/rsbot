//! Port of `bot/compose/fallback_chain.py`.
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

/// Python: bot/compose/fallback_chain.py:890 `_body_state(clean)`
pub fn _body_state(clean: &str) {
    crate::unported!("bot.compose.fallback_chain:_body_state")
}

/// Python: bot/compose/fallback_chain.py:938 `_a_part_in_a_state(clean)`
pub fn _a_part_in_a_state(clean: &str) {
    crate::unported!("bot.compose.fallback_chain:_a_part_in_a_state")
}

/// Python: bot/compose/fallback_chain.py:976 `_a_part_is_cold(clean)` (never called on the trace corpus)
pub fn _a_part_is_cold(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_part_is_cold")
}

/// Python: bot/compose/fallback_chain.py:985 `_state_from(rest)`
pub fn _state_from(rest: &str) -> Value {
    crate::unported!("bot.compose.fallback_chain:_state_from")
}

/// Python: bot/compose/fallback_chain.py:1006 `_the_state(word)`
pub fn _the_state(word: &str) {
    crate::unported!("bot.compose.fallback_chain:_the_state")
}

/// Python: bot/compose/fallback_chain.py:1052 `_habit_is_a_problem(clean)` (never called on the trace corpus)
pub fn _habit_is_a_problem(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_habit_is_a_problem")
}

/// Python: bot/compose/fallback_chain.py:1057 `reads_as_a_flat_habit(text, understood=None)`
pub fn reads_as_a_flat_habit(text: &str, understood: &Understood) -> bool {
    crate::unported!("bot.compose.fallback_chain:reads_as_a_flat_habit")
}

/// Python: bot/compose/fallback_chain.py:1081 `_flat_habit_stated(raw)`
pub fn _flat_habit_stated(raw: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_flat_habit_stated")
}

/// Python: bot/compose/fallback_chain.py:1102 `_a_claim_about_a_thing(before)`
pub fn _a_claim_about_a_thing(before: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_claim_about_a_thing")
}

/// Python: bot/compose/fallback_chain.py:1350 `_an_order_it_cannot_do(clean)`
pub fn _an_order_it_cannot_do(clean: &str) {
    crate::unported!("bot.compose.fallback_chain:_an_order_it_cannot_do")
}

/// Python: bot/compose/fallback_chain.py:1376 `_act_asked_of_me(clean)`
pub fn _act_asked_of_me(clean: &str) {
    crate::unported!("bot.compose.fallback_chain:_act_asked_of_me")
}

/// Python: bot/compose/fallback_chain.py:1403 `_the_store_has_the_recall(clean, memory)` (never called on the trace corpus)
pub fn _the_store_has_the_recall(clean: &str, memory: &Memory) -> Option<bool> {
    crate::unported!("bot.compose.fallback_chain:_the_store_has_the_recall")
}

/// Python: bot/compose/fallback_chain.py:1423 `_acts_for_me(clean, memory=None)`
pub fn _acts_for_me(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback_chain:_acts_for_me")
}

/// Python: bot/compose/fallback_chain.py:1472 `_reads_as_a_bad_stretch(clean)`
pub fn _reads_as_a_bad_stretch(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_reads_as_a_bad_stretch")
}

/// Python: bot/compose/fallback_chain.py:1489 `_cue_negated(words, cue)`
pub fn _cue_negated(words: &[String], cue: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_cue_negated")
}

/// Python: bot/compose/fallback_chain.py:1511 `_a_state_in_the_past(words, index)` (never called on the trace corpus)
pub fn _a_state_in_the_past(words: &[String], index: i64) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_state_in_the_past")
}

/// Python: bot/compose/fallback_chain.py:1516 `_the_win_is_someone_elses(words, index)`
pub fn _the_win_is_someone_elses(words: &[String], index: i64) -> bool {
    crate::unported!("bot.compose.fallback_chain:_the_win_is_someone_elses")
}

/// Python: bot/compose/fallback_chain.py:1620 `reads_as_no_pleasure(clean)`
pub fn reads_as_no_pleasure(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:reads_as_no_pleasure")
}

/// Python: bot/compose/fallback_chain.py:1664 `reads_as_a_question_about_the_bot(clean)`
pub fn reads_as_a_question_about_the_bot(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:reads_as_a_question_about_the_bot")
}

/// Python: bot/compose/fallback_chain.py:1693 `_a_thing_stopping_after(words, i)`
pub fn _a_thing_stopping_after(words: &[String], i: i64) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_thing_stopping_after")
}

/// Python: bot/compose/fallback_chain.py:1906 `_a_harm_said(clean)`
pub fn _a_harm_said(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_harm_said")
}

/// Python: bot/compose/fallback_chain.py:1948 `_mood(words, clean='')`
pub fn _mood(words: &[String], clean: &str) -> String {
    crate::unported!("bot.compose.fallback_chain:_mood")
}

/// Python: bot/compose/fallback_chain.py:2063 `_no_such_thing_before(words, i)`
pub fn _no_such_thing_before(words: &[String], i: i64) -> bool {
    crate::unported!("bot.compose.fallback_chain:_no_such_thing_before")
}

/// Python: bot/compose/fallback_chain.py:2082 `_mixed_seed(text)` (never called on the trace corpus)
pub fn _mixed_seed(text: &str) -> Value {
    crate::unported!("bot.compose.fallback_chain:_mixed_seed")
}

/// Python: bot/compose/fallback_chain.py:2115 `_only_what_the_line_gives(pool, clean)` (never called on the trace corpus)
pub fn _only_what_the_line_gives(pool: &Value, clean: &str) -> Value {
    crate::unported!("bot.compose.fallback_chain:_only_what_the_line_gives")
}

/// Python: bot/compose/fallback_chain.py:2133 `_choose(options, memory, clean, seed_text=None)`
pub fn _choose(options: &[String], memory: &Memory, clean: &str, seed_text: &str) -> String {
    crate::unported!("bot.compose.fallback_chain:_choose")
}

/// Python: bot/compose/fallback_chain.py:2161 `_a_birthday_still_to_come(clean, memory)` (never called on the trace corpus)
pub fn _a_birthday_still_to_come(clean: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.fallback_chain:_a_birthday_still_to_come")
}

/// Python: bot/compose/fallback_chain.py:2186 `_the_birthday_said_back(birthday)` (never called on the trace corpus)
pub fn _the_birthday_said_back(birthday: &Value) -> Value {
    crate::unported!("bot.compose.fallback_chain:_the_birthday_said_back")
}

/// Python: bot/compose/fallback_chain.py:2211 `_somebody_elses_day_in_thread(memory)`
pub fn _somebody_elses_day_in_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback_chain:_somebody_elses_day_in_thread")
}

/// Python: bot/compose/fallback_chain.py:2244 `_the_person_named_a_list(memory)`
pub fn _the_person_named_a_list(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback_chain:_the_person_named_a_list")
}

/// Python: bot/compose/fallback_chain.py:2257 `_routine_pool(clean, words, memory=None)`
pub fn _routine_pool(clean: &str, words: &[String], memory: &Memory) -> Option<Vec<String>> {
    crate::unported!("bot.compose.fallback_chain:_routine_pool")
}

/// Python: bot/compose/fallback_chain.py:2298 `_asks_the_bot(understood)`
pub fn _asks_the_bot(understood: &Understood) -> bool {
    crate::unported!("bot.compose.fallback_chain:_asks_the_bot")
}

/// Python: bot/compose/fallback_chain.py:2306 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    crate::unported!("bot.compose.fallback_chain:_words")
}

/// Python: bot/compose/fallback_chain.py:2311 `_reads(pred, clean, repaired, lower=False, memory=None)`
pub fn _reads(pred: &Value, clean: &str, repaired: &str, lower: bool, memory: Option<&Memory>) -> bool {
    crate::unported!("bot.compose.fallback_chain:_reads")
}

/// Python: bot/compose/fallback_chain.py:2341 `_restated(understood)`
pub fn _restated(understood: &Understood) -> String {
    crate::unported!("bot.compose.fallback_chain:_restated")
}

/// Python: bot/compose/fallback_chain.py:2360 `_answers_the_correction(restated, memory)`
pub fn _answers_the_correction(restated: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback_chain:_answers_the_correction")
}

/// Python: bot/compose/fallback_chain.py:2430 `_a_near_miss(clean)`
pub fn _a_near_miss(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_near_miss")
}

/// Python: bot/compose/fallback_chain.py:2435 `_a_things_state(memory, clean)`
pub fn _a_things_state(memory: &Memory, clean: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_things_state")
}

/// Python: bot/compose/fallback_chain.py:2487 `_a_chat_to_correct(memory)` (never called on the trace corpus)
pub fn _a_chat_to_correct(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_chat_to_correct")
}

/// Python: bot/compose/fallback_chain.py:2496 `_takes_something_back(clean, memory=None)`
pub fn _takes_something_back(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback_chain:_takes_something_back")
}

/// Python: bot/compose/fallback_chain.py:2520 `_a_lateness_report(clean, normalised='')`
pub fn _a_lateness_report(clean: &str, normalised: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_lateness_report")
}

/// Python: bot/compose/fallback_chain.py:2596 `_somebody_hurt(clean, memory, understood)`
pub fn _somebody_hurt(clean: &str, memory: &Memory, understood: &Understood) {
    crate::unported!("bot.compose.fallback_chain:_somebody_hurt")
}

/// Python: bot/compose/fallback_chain.py:2652 `_a_low_day(clean, understood)`
pub fn _a_low_day(clean: &str, understood: &Understood) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_low_day")
}

/// Python: bot/compose/fallback_chain.py:2697 `_loves_people(clean)`
pub fn _loves_people(clean: &str) -> i64 {
    crate::unported!("bot.compose.fallback_chain:_loves_people")
}

/// Python: bot/compose/fallback_chain.py:2717 `_loves_someone(clean)`
pub fn _loves_someone(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_loves_someone")
}

/// Python: bot/compose/fallback_chain.py:2801 `_the_clock_pool(at)` (never called on the trace corpus)
pub fn _the_clock_pool(at: i64) -> Value {
    crate::unported!("bot.compose.fallback_chain:_the_clock_pool")
}

/// Python: bot/compose/fallback_chain.py:2809 `_the_time_said(clean)`
pub fn _the_time_said(clean: &str) {
    crate::unported!("bot.compose.fallback_chain:_the_time_said")
}

/// Python: bot/compose/fallback_chain.py:2818 `_the_morning_has_a_clock(clean)`
pub fn _the_morning_has_a_clock(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_the_morning_has_a_clock")
}

/// Python: bot/compose/fallback_chain.py:2834 `reply_for(text, memory, verdict=True)`
pub fn reply_for(text: &str, memory: &mut Memory, verdict: bool) -> String {
    crate::unported!("bot.compose.fallback_chain:reply_for")
}

/// Python: bot/compose/fallback_chain.py:2898 `_options_already_up(memory)` (never called on the trace corpus)
pub fn _options_already_up(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback_chain:_options_already_up")
}

/// Python: bot/compose/fallback_chain.py:2912 `_reads_value(pred, clean, repaired, memory=None)`
pub fn _reads_value(pred: &Value, clean: &str, repaired: &str, memory: Option<&Memory>) -> Value {
    crate::unported!("bot.compose.fallback_chain:_reads_value")
}

/// Python: bot/compose/fallback_chain.py:2958 `_a_countdown_in_a_care_thread(memory, clean, repaired, understood)`
pub fn _a_countdown_in_a_care_thread(memory: &Memory, clean: &str, repaired: &str, understood: &Understood) {
    crate::unported!("bot.compose.fallback_chain:_a_countdown_in_a_care_thread")
}

/// Python: bot/compose/fallback_chain.py:2986 `_reply_for(text, memory, understood=None, verdict=True)`
pub fn _reply_for(text: &str, memory: &mut Memory, understood: &Understood, verdict: bool) -> String {
    crate::unported!("bot.compose.fallback_chain:_reply_for")
}

/// Python: bot/compose/fallback_chain.py:5028 `_a_betrayal(clean)`
pub fn _a_betrayal(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_betrayal")
}

/// Python: bot/compose/fallback_chain.py:5068 `_a_thing_they_regret(clean)`
pub fn _a_thing_they_regret(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_thing_they_regret")
}

/// Python: bot/compose/fallback_chain.py:5131 `_a_care_home_line(understood, memory)`
pub fn _a_care_home_line(understood: &Understood, memory: &Memory) {
    crate::unported!("bot.compose.fallback_chain:_a_care_home_line")
}

/// Python: bot/compose/fallback_chain.py:5167 `_a_day_told_instead_of_the_receipt(said_back, clean, memory)`
pub fn _a_day_told_instead_of_the_receipt(said_back: Option<&str>, clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback_chain:_a_day_told_instead_of_the_receipt")
}

/// Python: bot/compose/fallback_chain.py:5295 `_the_crowd_is_ahead(clean)`
pub fn _the_crowd_is_ahead(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_the_crowd_is_ahead")
}

/// Python: bot/compose/fallback_chain.py:5319 `_the_rest_of_the_day(clean, memory, mood='neutral')`
pub fn _the_rest_of_the_day(clean: &str, memory: &Memory, mood: &str) {
    crate::unported!("bot.compose.fallback_chain:_the_rest_of_the_day")
}

/// Python: bot/compose/fallback_chain.py:5348 `_an_emotion(clean)` (never called on the trace corpus)
pub fn _an_emotion(clean: &str) -> Value {
    crate::unported!("bot.compose.fallback_chain:_an_emotion")
}

/// Python: bot/compose/fallback_chain.py:5356 `_just_asked_about_the_day(memory)`
pub fn _just_asked_about_the_day(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback_chain:_just_asked_about_the_day")
}

/// Python: bot/compose/fallback_chain.py:5452 `_a_day_of_someone_else(text)`
pub fn _a_day_of_someone_else(text: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_day_of_someone_else")
}

/// Python: bot/compose/fallback_chain.py:5480 `_a_thing_happened(text)`
pub fn _a_thing_happened(text: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_thing_happened")
}

/// Python: bot/compose/fallback_chain.py:5518 `_a_thing_got_ahead(clean)`
pub fn _a_thing_got_ahead(clean: &str) {
    crate::unported!("bot.compose.fallback_chain:_a_thing_got_ahead")
}

/// Python: bot/compose/fallback_chain.py:5588 `_a_problem_stated(clean, repaired=None)`
pub fn _a_problem_stated(clean: &str, repaired: Option<&str>) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_problem_stated")
}

/// Python: bot/compose/fallback_chain.py:5646 `_says_there_is_nothing_to_wear(clean, repaired=None, text=None)`
pub fn _says_there_is_nothing_to_wear(clean: &str, repaired: &str, text: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_says_there_is_nothing_to_wear")
}

/// Python: bot/compose/fallback_chain.py:5655 `_a_wear_ask_about_a_day(clean, repaired=None, text=None)`
pub fn _a_wear_ask_about_a_day(clean: &str, repaired: &str, text: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_wear_ask_about_a_day")
}

/// Python: bot/compose/fallback_chain.py:5695 `_the_wear_pool_for_the_line(clean, repaired=None, text=None)`
pub fn _the_wear_pool_for_the_line(clean: &str, repaired: &str, text: &str) -> Vec<String> {
    crate::unported!("bot.compose.fallback_chain:_the_wear_pool_for_the_line")
}

/// Python: bot/compose/fallback_chain.py:5708 `_a_small_day(clean, memory=None)`
pub fn _a_small_day(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_small_day")
}

/// Python: bot/compose/fallback_chain.py:5900 `_a_detail_about_what_was_said(text, memory)`
pub fn _a_detail_about_what_was_said(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_detail_about_what_was_said")
}

/// Python: bot/compose/fallback_chain.py:6019 `_a_dish_name(clean, words, memory)` (never called on the trace corpus)
pub fn _a_dish_name(clean: &str, words: &[String], memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_dish_name")
}

/// Python: bot/compose/fallback_chain.py:6039 `_a_report_with_no_pronoun(clean, text, memory)`
pub fn _a_report_with_no_pronoun(clean: &str, text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_report_with_no_pronoun")
}

/// Python: bot/compose/fallback_chain.py:6080 `_repaired_of(text, memory)`
pub fn _repaired_of(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.fallback_chain:_repaired_of")
}

/// Python: bot/compose/fallback_chain.py:6096 `takes_as_it_comes(text, memory)`
pub fn takes_as_it_comes(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.compose.fallback_chain:takes_as_it_comes")
}

/// Python: bot/compose/fallback_chain.py:6223 `_a_name_told(clean)`
pub fn _a_name_told(clean: &str) {
    crate::unported!("bot.compose.fallback_chain:_a_name_told")
}

/// Python: bot/compose/fallback_chain.py:6277 `_named_after_reply(clean, memory)`
pub fn _named_after_reply(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback_chain:_named_after_reply")
}

/// Python: bot/compose/fallback_chain.py:6296 `_a_name_told_reply(clean, memory)`
pub fn _a_name_told_reply(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback_chain:_a_name_told_reply")
}

/// Python: bot/compose/fallback_chain.py:6343 `_a_state_of_theirs(text)`
pub fn _a_state_of_theirs(text: &str) -> Option<String> {
    crate::unported!("bot.compose.fallback_chain:_a_state_of_theirs")
}

/// Python: bot/compose/fallback_chain.py:6422 `_a_day_said_badly(text)`
pub fn _a_day_said_badly(text: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_day_said_badly")
}

/// Python: bot/compose/fallback_chain.py:6430 `_the_last_exchange(memory)`
pub fn _the_last_exchange(memory: &Memory) -> Value {
    crate::unported!("bot.compose.fallback_chain:_the_last_exchange")
}

/// Python: bot/compose/fallback_chain.py:6446 `_the_day_was_already_bad(memory)`
pub fn _the_day_was_already_bad(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback_chain:_the_day_was_already_bad")
}

/// Python: bot/compose/fallback_chain.py:6478 `_a_detail_of_a_bad_day(memory, text=None)`
pub fn _a_detail_of_a_bad_day(memory: &Memory, text: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_detail_of_a_bad_day")
}

/// Python: bot/compose/fallback_chain.py:6567 `_money_came_to_them(text, words)`
pub fn _money_came_to_them(text: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.fallback_chain:_money_came_to_them")
}

/// Python: bot/compose/fallback_chain.py:6574 `_a_sum_with_nothing_beside_it(clean, memory=None)`
pub fn _a_sum_with_nothing_beside_it(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback_chain:_a_sum_with_nothing_beside_it")
}

/// Python: bot/compose/fallback_chain.py:6653 `_a_report_from_elsewhere(clean)`
pub fn _a_report_from_elsewhere(clean: &str) {
    crate::unported!("bot.compose.fallback_chain:_a_report_from_elsewhere")
}

/// Python: bot/compose/fallback_chain.py:6679 `_a_small_day_shape(clean)`
pub fn _a_small_day_shape(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_small_day_shape")
}

/// Python: bot/compose/fallback_chain.py:6787 `_a_bad_time_told(clean, memory=None)`
pub fn _a_bad_time_told(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_bad_time_told")
}

/// Python: bot/compose/fallback_chain.py:6827 `_a_day_asked_without_a_cheer(clean, memory=None)`
pub fn _a_day_asked_without_a_cheer(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback_chain:_a_day_asked_without_a_cheer")
}

