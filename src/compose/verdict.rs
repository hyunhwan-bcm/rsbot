//! Port of `bot/compose/verdict.py`.
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

/// Python: bot/compose/verdict.py:412 `_a_job_somewhere(reply)`
pub fn _a_job_somewhere(reply: &str) -> bool {
    crate::unported!("bot.compose.verdict:_a_job_somewhere")
}

/// Python: bot/compose/verdict.py:438 `the_fix_asked(clean, memory)`
pub fn the_fix_asked(clean: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.verdict:the_fix_asked")
}

/// Python: bot/compose/verdict.py:565 `_the_list_before_the_call(said)` (never called on the trace corpus)
pub fn _the_list_before_the_call(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_the_list_before_the_call")
}

/// Python: bot/compose/verdict.py:602 `_the_call_and_its_reason(said)` (never called on the trace corpus)
pub fn _the_call_and_its_reason(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_the_call_and_its_reason")
}

/// Python: bot/compose/verdict.py:651 `_the_sign_words(call)` (never called on the trace corpus)
pub fn _the_sign_words(call: &Value) -> Value {
    crate::unported!("bot.compose.verdict:_the_sign_words")
}

/// Python: bot/compose/verdict.py:658 `_they_named_the_sign_themselves(call, memory)` (never called on the trace corpus)
pub fn _they_named_the_sign_themselves(call: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_they_named_the_sign_themselves")
}

/// Python: bot/compose/verdict.py:667 `_the_person_for(who, quick)` (never called on the trace corpus)
pub fn _the_person_for(who: &str, quick: &Value) -> Value {
    crate::unported!("bot.compose.verdict:_the_person_for")
}

/// Python: bot/compose/verdict.py:868 `_about_a_body()` (never called on the trace corpus)
pub fn _about_a_body(texts: &[Value]) -> bool {
    crate::unported!("bot.compose.verdict:_about_a_body")
}

/// Python: bot/compose/verdict.py:974 `_a_doubt_about_money(memory)` (never called on the trace corpus)
pub fn _a_doubt_about_money(memory: &Memory) -> Value {
    crate::unported!("bot.compose.verdict:_a_doubt_about_money")
}

/// Python: bot/compose/verdict.py:1072 `_is_a_comparison(said)` (never called on the trace corpus)
pub fn _is_a_comparison(said: &str) -> bool {
    crate::unported!("bot.compose.verdict:_is_a_comparison")
}

/// Python: bot/compose/verdict.py:1096 `_a_figure_owed(line)` (never called on the trace corpus)
pub fn _a_figure_owed(line: &str) -> bool {
    crate::unported!("bot.compose.verdict:_a_figure_owed")
}

/// Python: bot/compose/verdict.py:1895 `_is_an_explanation(sentence)` (never called on the trace corpus)
pub fn _is_an_explanation(sentence: &str) -> bool {
    crate::unported!("bot.compose.verdict:_is_an_explanation")
}

/// Python: bot/compose/verdict.py:2047 `_own_pattern(line)`
pub fn _own_pattern(line: &str) -> String {
    crate::unported!("bot.compose.verdict:_own_pattern")
}

/// Python: bot/compose/verdict.py:2075 `takes_a_fact_back(said)`
pub fn takes_a_fact_back(said: &str) -> bool {
    crate::unported!("bot.compose.verdict:takes_a_fact_back")
}

/// Python: bot/compose/verdict.py:2097 `_say(pool, memory, flat)`
pub fn _say(pool: &[String], memory: &Memory, flat: &str) -> String {
    crate::unported!("bot.compose.verdict:_say")
}

/// Python: bot/compose/verdict.py:2114 `_a_report_line(said)`
pub fn _a_report_line(said: &str) -> bool {
    crate::unported!("bot.compose.verdict:_a_report_line")
}

/// Python: bot/compose/verdict.py:2129 `_flat(text)`
pub fn _flat(text: &str) -> String {
    crate::unported!("bot.compose.verdict:_flat")
}

/// Python: bot/compose/verdict.py:2165 `rules_on_the_doubt(said)`
pub fn rules_on_the_doubt(said: &str) -> bool {
    crate::unported!("bot.compose.verdict:rules_on_the_doubt")
}

/// Python: bot/compose/verdict.py:2192 `opens_on_a_job(reply)`
pub fn opens_on_a_job(reply: &str) -> bool {
    crate::unported!("bot.compose.verdict:opens_on_a_job")
}

/// Python: bot/compose/verdict.py:2220 `carries_the_answer(reply, memory)`
pub fn carries_the_answer(reply: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:carries_the_answer")
}

/// Python: bot/compose/verdict.py:2261 `_a_punt(last)`
pub fn _a_punt(last: &Value) -> bool {
    crate::unported!("bot.compose.verdict:_a_punt")
}

/// Python: bot/compose/verdict.py:2283 `_skip_the_derived(turns, memory)`
pub fn _skip_the_derived(turns: &Value, memory: &Memory) -> Value {
    crate::unported!("bot.compose.verdict:_skip_the_derived")
}

/// Python: bot/compose/verdict.py:2298 `answers_the_ask(clean, memory)`
pub fn answers_the_ask(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:answers_the_ask")
}

/// Python: bot/compose/verdict.py:2316 `_the_news_just_before(memory)` (never called on the trace corpus)
pub fn _the_news_just_before(memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_the_news_just_before")
}

/// Python: bot/compose/verdict.py:2337 `_last_reply(memory, past_the_nod=False, past_the_verdict=False)`
pub fn _last_reply(memory: &Memory, past_the_nod: bool, past_the_verdict: bool) -> String {
    crate::unported!("bot.compose.verdict:_last_reply")
}

/// Python: bot/compose/verdict.py:2448 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    crate::unported!("bot.compose.verdict:_words")
}

/// Python: bot/compose/verdict.py:2456 `_is_an_echo(turn)`
pub fn _is_an_echo(turn: &Value) -> bool {
    crate::unported!("bot.compose.verdict:_is_an_echo")
}

/// Python: bot/compose/verdict.py:2479 `_is_a_recap(turn, before)`
pub fn _is_a_recap(turn: &Value, before: &Value) -> bool {
    crate::unported!("bot.compose.verdict:_is_a_recap")
}

/// Python: bot/compose/verdict.py:2495 `_last_line_that_named_a_call(memory)` (never called on the trace corpus)
pub fn _last_line_that_named_a_call(memory: &Memory) -> Value {
    crate::unported!("bot.compose.verdict:_last_line_that_named_a_call")
}

/// Python: bot/compose/verdict.py:2528 `_a_call_for_now(said)` (never called on the trace corpus)
pub fn _a_call_for_now(said: &str) -> bool {
    crate::unported!("bot.compose.verdict:_a_call_for_now")
}

/// Python: bot/compose/verdict.py:2582 `_the_call_sentence(said)` (never called on the trace corpus)
pub fn _the_call_sentence(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_the_call_sentence")
}

/// Python: bot/compose/verdict.py:2624 `_last_person_line(memory)` (never called on the trace corpus)
pub fn _last_person_line(memory: &Memory) -> Value {
    crate::unported!("bot.compose.verdict:_last_person_line")
}

/// Python: bot/compose/verdict.py:2639 `_the_worst_thing_they_said(memory)` (never called on the trace corpus)
pub fn _the_worst_thing_they_said(memory: &Memory) -> Value {
    crate::unported!("bot.compose.verdict:_the_worst_thing_they_said")
}

/// Python: bot/compose/verdict.py:2658 `_as_written(line)` (never called on the trace corpus)
pub fn _as_written(line: &str) -> Vec<String> {
    crate::unported!("bot.compose.verdict:_as_written")
}

/// Python: bot/compose/verdict.py:2679 `_a_thing_they_own(memory)` (never called on the trace corpus)
pub fn _a_thing_they_own(memory: &Memory) -> String {
    crate::unported!("bot.compose.verdict:_a_thing_they_own")
}

/// Python: bot/compose/verdict.py:2696 `_a_person_they_know(memory, said)` (never called on the trace corpus)
pub fn _a_person_they_know(memory: &Memory, said: &str) -> String {
    crate::unported!("bot.compose.verdict:_a_person_they_know")
}

/// Python: bot/compose/verdict.py:2715 `_the_line_behind(memory)` (never called on the trace corpus)
pub fn _the_line_behind(memory: &Memory) -> String {
    crate::unported!("bot.compose.verdict:_the_line_behind")
}

/// Python: bot/compose/verdict.py:2756 `_the_plan_they_brought(memory, back=2)` (never called on the trace corpus)
pub fn _the_plan_they_brought(memory: &Memory, back: &Value) -> Value {
    crate::unported!("bot.compose.verdict:_the_plan_they_brought")
}

/// Python: bot/compose/verdict.py:2822 `_considering(memory)` (never called on the trace corpus)
pub fn _considering(memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_considering")
}

/// Python: bot/compose/verdict.py:2836 `_the_last_line_was_mine(memory)` (never called on the trace corpus)
pub fn _the_last_line_was_mine(memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_the_last_line_was_mine")
}

/// Python: bot/compose/verdict.py:2858 `_last_module(memory, past_the_nod=False)`
pub fn _last_module(memory: &Memory, past_the_nod: bool) -> String {
    crate::unported!("bot.compose.verdict:_last_module")
}

/// Python: bot/compose/verdict.py:3021 `_the_plan_can_hurt(said)` (never called on the trace corpus)
pub fn _the_plan_can_hurt(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_the_plan_can_hurt")
}

/// Python: bot/compose/verdict.py:3040 `_the_answer_already_warned(said)` (never called on the trace corpus)
pub fn _the_answer_already_warned(said: &str) -> String {
    crate::unported!("bot.compose.verdict:_the_answer_already_warned")
}

/// Python: bot/compose/verdict.py:3061 `_a_plan_doubt(memory)` (never called on the trace corpus)
pub fn _a_plan_doubt(memory: &Memory) -> Value {
    crate::unported!("bot.compose.verdict:_a_plan_doubt")
}

/// Python: bot/compose/verdict.py:3082 `_about_the_person(memory)` (never called on the trace corpus)
pub fn _about_the_person(memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_about_the_person")
}

/// Python: bot/compose/verdict.py:3105 `_own_doing(memory)` (never called on the trace corpus)
pub fn _own_doing(memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_own_doing")
}

/// Python: bot/compose/verdict.py:3120 `_a_plan_or_a_choice(memory)` (never called on the trace corpus)
pub fn _a_plan_or_a_choice(memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_a_plan_or_a_choice")
}

/// Python: bot/compose/verdict.py:3137 `_good_news(memory)` (never called on the trace corpus)
pub fn _good_news(memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_good_news")
}

/// Python: bot/compose/verdict.py:3174 `_an_event_that_is_over(memory)` (never called on the trace corpus)
pub fn _an_event_that_is_over(memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_an_event_that_is_over")
}

/// Python: bot/compose/verdict.py:3197 `_nothing_to_be_long_about(memory)` (never called on the trace corpus)
pub fn _nothing_to_be_long_about(memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_nothing_to_be_long_about")
}

/// Python: bot/compose/verdict.py:3207 `_first(text, words)` (never called on the trace corpus)
pub fn _first(text: &str, words: &[String]) -> Value {
    crate::unported!("bot.compose.verdict:_first")
}

/// Python: bot/compose/verdict.py:3222 `_a_throw_away(said)` (never called on the trace corpus)
pub fn _a_throw_away(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_a_throw_away")
}

/// Python: bot/compose/verdict.py:3258 `_the_person_line(memory, said)` (never called on the trace corpus)
pub fn _the_person_line(memory: &Memory, said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_the_person_line")
}

/// Python: bot/compose/verdict.py:3268 `_said_no(said)` (never called on the trace corpus)
pub fn _said_no(said: &str) -> Option<String> {
    crate::unported!("bot.compose.verdict:_said_no")
}

/// Python: bot/compose/verdict.py:3286 `_said_enough(said)` (never called on the trace corpus)
pub fn _said_enough(said: &str) -> Option<String> {
    crate::unported!("bot.compose.verdict:_said_enough")
}

/// Python: bot/compose/verdict.py:3307 `_the_person_gave_the_sign(memory, said)` (never called on the trace corpus)
pub fn _the_person_gave_the_sign(memory: &Memory, said: &str) -> bool {
    crate::unported!("bot.compose.verdict:_the_person_gave_the_sign")
}

/// Python: bot/compose/verdict.py:3327 `_care_for_now(said)` (never called on the trace corpus)
pub fn _care_for_now(said: &str) -> bool {
    crate::unported!("bot.compose.verdict:_care_for_now")
}

/// Python: bot/compose/verdict.py:3345 `_sentence_at(text, at)` (never called on the trace corpus)
pub fn _sentence_at(text: &str, at: i64) -> Value {
    crate::unported!("bot.compose.verdict:_sentence_at")
}

/// Python: bot/compose/verdict.py:3355 `_behind_a_description(said, at)` (never called on the trace corpus)
pub fn _behind_a_description(said: &str, at: i64) -> bool {
    crate::unported!("bot.compose.verdict:_behind_a_description")
}

/// Python: bot/compose/verdict.py:3376 `_not_fenced(said, start, end=None)` (never called on the trace corpus)
pub fn _not_fenced(said: &str, start: i64, end: i64) -> bool {
    crate::unported!("bot.compose.verdict:_not_fenced")
}

/// Python: bot/compose/verdict.py:3408 `_said_without_a_not(text, at, length)` (never called on the trace corpus)
pub fn _said_without_a_not(text: &str, at: i64, length: &Value) -> bool {
    crate::unported!("bot.compose.verdict:_said_without_a_not")
}

/// Python: bot/compose/verdict.py:3419 `_first_positive(text, words)` (never called on the trace corpus)
pub fn _first_positive(text: &str, words: &[String]) -> Value {
    crate::unported!("bot.compose.verdict:_first_positive")
}

/// Python: bot/compose/verdict.py:3443 `_unconditional_care(said)` (never called on the trace corpus)
pub fn _unconditional_care(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_unconditional_care")
}

/// Python: bot/compose/verdict.py:3474 `_the_fence_is_met(fence, said, memory)` (never called on the trace corpus)
pub fn _the_fence_is_met(fence: &Value, said: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_the_fence_is_met")
}

/// Python: bot/compose/verdict.py:3506 `_a_frequency_already_said(fence, said, memory)` (never called on the trace corpus)
pub fn _a_frequency_already_said(fence: &Value, said: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_a_frequency_already_said")
}

/// Python: bot/compose/verdict.py:3538 `_condition_after_care(said, care_at, memory=None)` (never called on the trace corpus)
pub fn _condition_after_care(said: &str, care_at: &Value, memory: &Memory) -> Value {
    crate::unported!("bot.compose.verdict:_condition_after_care")
}

/// Python: bot/compose/verdict.py:3615 `_cut_condition(when, most=22)` (never called on the trace corpus)
pub fn _cut_condition(when: &str, most: &Value) -> Value {
    crate::unported!("bot.compose.verdict:_cut_condition")
}

/// Python: bot/compose/verdict.py:3622 `_first_sentence_end(said)` (never called on the trace corpus)
pub fn _first_sentence_end(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_first_sentence_end")
}

/// Python: bot/compose/verdict.py:3628 `_told_to_act(said, at)` (never called on the trace corpus)
pub fn _told_to_act(said: &str, at: i64) -> bool {
    crate::unported!("bot.compose.verdict:_told_to_act")
}

/// Python: bot/compose/verdict.py:3645 `_referral(said)` (never called on the trace corpus)
pub fn _referral(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_referral")
}

/// Python: bot/compose/verdict.py:3674 `_a_plain_go(at, said)` (never called on the trace corpus)
pub fn _a_plain_go(at: i64, said: &str) -> bool {
    crate::unported!("bot.compose.verdict:_a_plain_go")
}

/// Python: bot/compose/verdict.py:3688 `_opening(said)` (never called on the trace corpus)
pub fn _opening(said: &str) -> String {
    crate::unported!("bot.compose.verdict:_opening")
}

/// Python: bot/compose/verdict.py:3723 `_the_thing(line)` (never called on the trace corpus)
pub fn _the_thing(line: &str) -> Value {
    crate::unported!("bot.compose.verdict:_the_thing")
}

/// Python: bot/compose/verdict.py:3728 `_referral_about_the_thing(said, line)` (never called on the trace corpus)
pub fn _referral_about_the_thing(said: &str, line: &str) -> bool {
    crate::unported!("bot.compose.verdict:_referral_about_the_thing")
}

/// Python: bot/compose/verdict.py:3743 `_echoes_the_person(step, memory)` (never called on the trace corpus)
pub fn _echoes_the_person(step: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_echoes_the_person")
}

/// Python: bot/compose/verdict.py:3768 `_kept_step(reply)`
pub fn _kept_step(reply: &str) -> bool {
    crate::unported!("bot.compose.verdict:_kept_step")
}

/// Python: bot/compose/verdict.py:3775 `_only_a_verdict(line)` (never called on the trace corpus)
pub fn _only_a_verdict(line: &str) -> Option<bool> {
    crate::unported!("bot.compose.verdict:_only_a_verdict")
}

/// Python: bot/compose/verdict.py:3790 `_the_line_just_given(memory)` (never called on the trace corpus)
pub fn _the_line_just_given(memory: &Memory) -> String {
    crate::unported!("bot.compose.verdict:_the_line_just_given")
}

/// Python: bot/compose/verdict.py:3801 `_the_answer_before_that(memory)` (never called on the trace corpus)
pub fn _the_answer_before_that(memory: &Memory) -> String {
    crate::unported!("bot.compose.verdict:_the_answer_before_that")
}

/// Python: bot/compose/verdict.py:3819 `_word_runs(text, size=5)` (never called on the trace corpus)
pub fn _word_runs(text: &str, size: i64) -> Value {
    crate::unported!("bot.compose.verdict:_word_runs")
}

/// Python: bot/compose/verdict.py:3824 `_says_it_again(line, said)` (never called on the trace corpus)
pub fn _says_it_again(line: &str, said: &str) -> bool {
    crate::unported!("bot.compose.verdict:_says_it_again")
}

/// Python: bot/compose/verdict.py:3843 `_already_asked_for_a_step(memory)` (never called on the trace corpus)
pub fn _already_asked_for_a_step(memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_already_asked_for_a_step")
}

/// Python: bot/compose/verdict.py:3854 `_the_referral_sentence(said)` (never called on the trace corpus)
pub fn _the_referral_sentence(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_the_referral_sentence")
}

/// Python: bot/compose/verdict.py:3864 `_the_line_is_the_threshold(stated, line)` (never called on the trace corpus)
pub fn _the_line_is_the_threshold(stated: &Value, line: &str) -> bool {
    crate::unported!("bot.compose.verdict:_the_line_is_the_threshold")
}

/// Python: bot/compose/verdict.py:3878 `_most_of_the_line(step, heard)` (never called on the trace corpus)
pub fn _most_of_the_line(step: &Value, heard: &Value) -> bool {
    crate::unported!("bot.compose.verdict:_most_of_the_line")
}

/// Python: bot/compose/verdict.py:3893 `_step(said)` (never called on the trace corpus)
pub fn _step(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_step")
}

/// Python: bot/compose/verdict.py:3927 `_listed(said)` (never called on the trace corpus)
pub fn _listed(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_listed")
}

/// Python: bot/compose/verdict.py:4022 `_cut_on(head, joins)` (never called on the trace corpus)
pub fn _cut_on(head: &str, joins: &Value) -> Value {
    crate::unported!("bot.compose.verdict:_cut_on")
}

/// Python: bot/compose/verdict.py:4062 `_shorten(head, words)` (never called on the trace corpus)
pub fn _shorten(head: &str, words: &[String]) -> Value {
    crate::unported!("bot.compose.verdict:_shorten")
}

/// Python: bot/compose/verdict.py:4098 `_cut_at_a_join(part)` (never called on the trace corpus)
pub fn _cut_at_a_join(part: &str) -> Value {
    crate::unported!("bot.compose.verdict:_cut_at_a_join")
}

/// Python: bot/compose/verdict.py:4111 `_first_clause(part, and_break=True)` (never called on the trace corpus)
pub fn _first_clause(part: &str, and_break: &Value) -> Value {
    crate::unported!("bot.compose.verdict:_first_clause")
}

/// Python: bot/compose/verdict.py:4172 `_an_offer_is_waiting(said)` (never called on the trace corpus)
pub fn _an_offer_is_waiting(said: &str) -> bool {
    crate::unported!("bot.compose.verdict:_an_offer_is_waiting")
}

/// Python: bot/compose/verdict.py:4204 `_after_a_comfort_line(memory)` (never called on the trace corpus)
pub fn _after_a_comfort_line(memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_after_a_comfort_line")
}

/// Python: bot/compose/verdict.py:4239 `_the_question_asked(said)` (never called on the trace corpus)
pub fn _the_question_asked(said: &str) -> String {
    crate::unported!("bot.compose.verdict:_the_question_asked")
}

/// Python: bot/compose/verdict.py:4273 `_ends_on_a_question(said)` (never called on the trace corpus)
pub fn _ends_on_a_question(said: &str) -> bool {
    crate::unported!("bot.compose.verdict:_ends_on_a_question")
}

/// Python: bot/compose/verdict.py:4282 `_is_a_question(sentence)` (never called on the trace corpus)
pub fn _is_a_question(sentence: &str) -> bool {
    crate::unported!("bot.compose.verdict:_is_a_question")
}

/// Python: bot/compose/verdict.py:4292 `_steps(said)` (never called on the trace corpus)
pub fn _steps(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_steps")
}

/// Python: bot/compose/verdict.py:4425 `_doing_word(words)` (never called on the trace corpus)
pub fn _doing_word(words: &[String]) -> Value {
    crate::unported!("bot.compose.verdict:_doing_word")
}

/// Python: bot/compose/verdict.py:4440 `_action_count(said)` (never called on the trace corpus)
pub fn _action_count(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_action_count")
}

/// Python: bot/compose/verdict.py:4456 `_no_hanging_end(when)` (never called on the trace corpus)
pub fn _no_hanging_end(when: &str) -> Value {
    crate::unported!("bot.compose.verdict:_no_hanging_end")
}

/// Python: bot/compose/verdict.py:4505 `_described_clause(sentence)` (never called on the trace corpus)
pub fn _described_clause(sentence: &str) -> Value {
    crate::unported!("bot.compose.verdict:_described_clause")
}

/// Python: bot/compose/verdict.py:4546 `_in_a_described_clause(sentence, at)` (never called on the trace corpus)
pub fn _in_a_described_clause(sentence: &str, at: i64) -> bool {
    crate::unported!("bot.compose.verdict:_in_a_described_clause")
}

/// Python: bot/compose/verdict.py:4552 `_a_described_condition(sentence)` (never called on the trace corpus)
pub fn _a_described_condition(sentence: &str) -> String {
    crate::unported!("bot.compose.verdict:_a_described_condition")
}

/// Python: bot/compose/verdict.py:4563 `_an_if_that_is_a_condition(sentence)` (never called on the trace corpus)
pub fn _an_if_that_is_a_condition(sentence: &str) -> Value {
    crate::unported!("bot.compose.verdict:_an_if_that_is_a_condition")
}

/// Python: bot/compose/verdict.py:4592 `_a_judgement_fence(said)` (never called on the trace corpus)
pub fn _a_judgement_fence(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_a_judgement_fence")
}

/// Python: bot/compose/verdict.py:4618 `_after_the_if(sentence)` (never called on the trace corpus)
pub fn _after_the_if(sentence: &str) -> Value {
    crate::unported!("bot.compose.verdict:_after_the_if")
}

/// Python: bot/compose/verdict.py:4652 `_when(said)` (never called on the trace corpus)
pub fn _when(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_when")
}

/// Python: bot/compose/verdict.py:4710 `_stated_before_the_judgement(sentence)` (never called on the trace corpus)
pub fn _stated_before_the_judgement(sentence: &str) -> Value {
    crate::unported!("bot.compose.verdict:_stated_before_the_judgement")
}

/// Python: bot/compose/verdict.py:4734 `_stated_referral(said)` (never called on the trace corpus)
pub fn _stated_referral(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_stated_referral")
}

/// Python: bot/compose/verdict.py:4790 `_danger(said)` (never called on the trace corpus)
pub fn _danger(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_danger")
}

/// Python: bot/compose/verdict.py:4812 `_a_how_to(said)` (never called on the trace corpus)
pub fn _a_how_to(said: &str) -> bool {
    crate::unported!("bot.compose.verdict:_a_how_to")
}

/// Python: bot/compose/verdict.py:4838 `_round_robin(pool, said)` (never called on the trace corpus)
pub fn _round_robin(pool: &Value, said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_round_robin")
}

/// Python: bot/compose/verdict.py:4853 `_says_what_it_is(said)` (never called on the trace corpus)
pub fn _says_what_it_is(said: &str) -> bool {
    crate::unported!("bot.compose.verdict:_says_what_it_is")
}

/// Python: bot/compose/verdict.py:4911 `_the_line_it_answered(said, memory)` (never called on the trace corpus)
pub fn _the_line_it_answered(said: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.verdict:_the_line_it_answered")
}

/// Python: bot/compose/verdict.py:4931 `_past_the_timescale(said, memory)` (never called on the trace corpus)
pub fn _past_the_timescale(said: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_past_the_timescale")
}

/// Python: bot/compose/verdict.py:4954 `_subject_words(text)` (never called on the trace corpus)
pub fn _subject_words(text: &str) -> Value {
    crate::unported!("bot.compose.verdict:_subject_words")
}

/// Python: bot/compose/verdict.py:4960 `_the_span_further_up(said, memory)` (never called on the trace corpus)
pub fn _the_span_further_up(said: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.verdict:_the_span_further_up")
}

/// Python: bot/compose/verdict.py:5004 `_stems(text)` (never called on the trace corpus)
pub fn _stems(text: &str) -> Value {
    crate::unported!("bot.compose.verdict:_stems")
}

/// Python: bot/compose/verdict.py:5019 `_the_fence_is_their_own_line(said, memory)` (never called on the trace corpus)
pub fn _the_fence_is_their_own_line(said: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_the_fence_is_their_own_line")
}

/// Python: bot/compose/verdict.py:5060 `_a_mood(memory)` (never called on the trace corpus)
pub fn _a_mood(memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_a_mood")
}

/// Python: bot/compose/verdict.py:5065 `_about_a_death(memory)` (never called on the trace corpus)
pub fn _about_a_death(memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_about_a_death")
}

/// Python: bot/compose/verdict.py:5085 `_the_last_person_lines(memory, count)` (never called on the trace corpus)
pub fn _the_last_person_lines(memory: &Memory, count: i64) -> Value {
    crate::unported!("bot.compose.verdict:_the_last_person_lines")
}

/// Python: bot/compose/verdict.py:5113 `_a_job_to_do(text)` (never called on the trace corpus)
pub fn _a_job_to_do(text: &str) -> bool {
    crate::unported!("bot.compose.verdict:_a_job_to_do")
}

/// Python: bot/compose/verdict.py:5130 `_the_advice_itself(said)` (never called on the trace corpus)
pub fn _the_advice_itself(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_the_advice_itself")
}

/// Python: bot/compose/verdict.py:5165 `_an_ask_for_a_call(memory)` (never called on the trace corpus)
pub fn _an_ask_for_a_call(memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_an_ask_for_a_call")
}

/// Python: bot/compose/verdict.py:5171 `_a_fix_with_no_warning(said)` (never called on the trace corpus)
pub fn _a_fix_with_no_warning(said: &str) -> bool {
    crate::unported!("bot.compose.verdict:_a_fix_with_no_warning")
}

/// Python: bot/compose/verdict.py:5224 `_somebody_owes_them(memory, back=3)` (never called on the trace corpus)
pub fn _somebody_owes_them(memory: &Memory, back: &Value) -> bool {
    crate::unported!("bot.compose.verdict:_somebody_owes_them")
}

/// Python: bot/compose/verdict.py:5238 `_a_reading_of_the_feeling(memory, flat)` (never called on the trace corpus)
pub fn _a_reading_of_the_feeling(memory: &Memory, flat: &Value) -> Value {
    crate::unported!("bot.compose.verdict:_a_reading_of_the_feeling")
}

/// Python: bot/compose/verdict.py:5257 `_care_pool(said)` (never called on the trace corpus)
pub fn _care_pool(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_care_pool")
}

/// Python: bot/compose/verdict.py:5291 `_the_knowledge_has_this(clean, memory, above)` (never called on the trace corpus)
pub fn _the_knowledge_has_this(clean: &str, memory: &Memory, above: &Value) -> Value {
    crate::unported!("bot.compose.verdict:_the_knowledge_has_this")
}

/// Python: bot/compose/verdict.py:5317 `a_check_on_the_reply(reply)`
pub fn a_check_on_the_reply(reply: &str) -> bool {
    crate::unported!("bot.compose.verdict:a_check_on_the_reply")
}

/// Python: bot/compose/verdict.py:5332 `_an_amount_asked(flat)` (never called on the trace corpus)
pub fn _an_amount_asked(flat: &Value) -> bool {
    crate::unported!("bot.compose.verdict:_an_amount_asked")
}

/// Python: bot/compose/verdict.py:5338 `_the_last_thing_said_is_a_nod(memory)` (never called on the trace corpus)
pub fn _the_last_thing_said_is_a_nod(memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_the_last_thing_said_is_a_nod")
}

/// Python: bot/compose/verdict.py:5347 `_a_check_on_the_method(flat, memory)` (never called on the trace corpus)
pub fn _a_check_on_the_method(flat: &Value, memory: &Memory) -> Value {
    crate::unported!("bot.compose.verdict:_a_check_on_the_method")
}

/// Python: bot/compose/verdict.py:5422 `reply_for(clean, memory)`
pub fn reply_for(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.verdict:reply_for")
}

/// Python: bot/compose/verdict.py:6161 `_the_referral_it_closed_on(said)` (never called on the trace corpus)
pub fn _the_referral_it_closed_on(said: &str) -> Value {
    crate::unported!("bot.compose.verdict:_the_referral_it_closed_on")
}

/// Python: bot/compose/verdict.py:6193 `_the_range_it_gave(said)` (never called on the trace corpus)
pub fn _the_range_it_gave(said: &str) -> String {
    crate::unported!("bot.compose.verdict:_the_range_it_gave")
}

/// Python: bot/compose/verdict.py:6218 `_an_instruction_and_nothing_else(said, memory)` (never called on the trace corpus)
pub fn _an_instruction_and_nothing_else(said: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_an_instruction_and_nothing_else")
}

/// Python: bot/compose/verdict.py:6277 `_the_floor_asked_them_something(memory)` (never called on the trace corpus)
pub fn _the_floor_asked_them_something(memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_the_floor_asked_them_something")
}

/// Python: bot/compose/verdict.py:6300 `_a_feeling_of_their_own(line)` (never called on the trace corpus)
pub fn _a_feeling_of_their_own(line: &str) -> Value {
    crate::unported!("bot.compose.verdict:_a_feeling_of_their_own")
}

/// Python: bot/compose/verdict.py:6322 `_the_other_thing_in_the_answer(said, step, memory)` (never called on the trace corpus)
pub fn _the_other_thing_in_the_answer(said: &str, step: &Value, memory: &Memory) -> Value {
    crate::unported!("bot.compose.verdict:_the_other_thing_in_the_answer")
}

/// Python: bot/compose/verdict.py:6363 `_step_sentence(said, step)` (never called on the trace corpus)
pub fn _step_sentence(said: &str, step: &Value) -> String {
    crate::unported!("bot.compose.verdict:_step_sentence")
}

/// Python: bot/compose/verdict.py:6383 `_up_to_a_break(said, step, within)` (never called on the trace corpus)
pub fn _up_to_a_break(said: &str, step: &Value, within: &Value) -> Value {
    crate::unported!("bot.compose.verdict:_up_to_a_break")
}

/// Python: bot/compose/verdict.py:6415 `_an_instruction_sentence(said)` (never called on the trace corpus)
pub fn _an_instruction_sentence(said: &str) -> String {
    crate::unported!("bot.compose.verdict:_an_instruction_sentence")
}

/// Python: bot/compose/verdict.py:6446 `_the_answer_under_a_frame(memory)` (never called on the trace corpus)
pub fn _the_answer_under_a_frame(memory: &Memory) -> Value {
    crate::unported!("bot.compose.verdict:_the_answer_under_a_frame")
}

/// Python: bot/compose/verdict.py:6482 `_the_front_names_what_is_in_hand(head, line, memory)` (never called on the trace corpus)
pub fn _the_front_names_what_is_in_hand(head: &str, line: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.verdict:_the_front_names_what_is_in_hand")
}

/// Python: bot/compose/verdict.py:6511 `next_step(clean, memory)`
pub fn next_step(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.verdict:next_step")
}

