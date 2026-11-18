//! Port of `bot/modules/chat.py`.
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

/// Python: bot/modules/chat.py:164 `_only_a_refusal(clean, tokens)` (never called on the trace corpus)
pub fn _only_a_refusal(clean: &str, tokens: &[String]) -> bool {
    crate::unported!("bot.modules.chat:_only_a_refusal")
}

/// Python: bot/modules/chat.py:414 `_keeps_failing_theirs(clean)`
pub fn _keeps_failing_theirs(clean: &str) {
    crate::unported!("bot.modules.chat:_keeps_failing_theirs")
}

/// Python: bot/modules/chat.py:431 `_letting_someone_down(clean)`
pub fn _letting_someone_down(clean: &str) {
    crate::unported!("bot.modules.chat:_letting_someone_down")
}

/// Python: bot/modules/chat.py:465 `_about_themselves(tokens)` (never called on the trace corpus)
pub fn _about_themselves(tokens: &[String]) -> bool {
    crate::unported!("bot.modules.chat:_about_themselves")
}

/// Python: bot/modules/chat.py:488 `_themselves_unfair(tokens)` (never called on the trace corpus)
pub fn _themselves_unfair(tokens: &[String]) -> bool {
    crate::unported!("bot.modules.chat:_themselves_unfair")
}

/// Python: bot/modules/chat.py:566 `_the_answer_met_it_as_news(memory)` (never called on the trace corpus)
pub fn _the_answer_met_it_as_news(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_the_answer_met_it_as_news")
}

/// Python: bot/modules/chat.py:576 `_a_loss_before(said, memory=None)` (never called on the trace corpus)
pub fn _a_loss_before(said: &str, memory: &Memory) -> Option<bool> {
    crate::unported!("bot.modules.chat:_a_loss_before")
}

/// Python: bot/modules/chat.py:600 `_a_thread_to_come_back_to(memory)` (never called on the trace corpus)
pub fn _a_thread_to_come_back_to(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_a_thread_to_come_back_to")
}

/// Python: bot/modules/chat.py:612 `_news_before(memory)`
pub fn _news_before(memory: &Memory) -> String {
    crate::unported!("bot.modules.chat:_news_before")
}

/// Python: bot/modules/chat.py:619 `_nobody_would_call_it_nice(said)` (never called on the trace corpus)
pub fn _nobody_would_call_it_nice(said: &str) -> bool {
    crate::unported!("bot.modules.chat:_nobody_would_call_it_nice")
}

/// Python: bot/modules/chat.py:666 `_the_agreement_pool(memory)` (never called on the trace corpus)
pub fn _the_agreement_pool(memory: &Memory) -> Option<String> {
    crate::unported!("bot.modules.chat:_the_agreement_pool")
}

/// Python: bot/modules/chat.py:692 `_reaction(clean, tokens, memory)`
pub fn _reaction(clean: &str, tokens: &[String], memory: &Memory) {
    crate::unported!("bot.modules.chat:_reaction")
}

/// Python: bot/modules/chat.py:700 `_reaction_pool(clean, tokens, memory)`
pub fn _reaction_pool(clean: &str, tokens: &[String], memory: &Memory) {
    crate::unported!("bot.modules.chat:_reaction_pool")
}

/// Python: bot/modules/chat.py:825 `_turns_on_news(clean)` (never called on the trace corpus)
pub fn _turns_on_news(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_turns_on_news")
}

/// Python: bot/modules/chat.py:849 `_asks_after(clean, judged)` (never called on the trace corpus)
pub fn _asks_after(clean: &str, judged: &Value) -> bool {
    crate::unported!("bot.modules.chat:_asks_after")
}

/// Python: bot/modules/chat.py:871 `_judged_as_a_noun(tokens)`
pub fn _judged_as_a_noun(tokens: &[String]) {
    crate::unported!("bot.modules.chat:_judged_as_a_noun")
}

/// Python: bot/modules/chat.py:895 `_when_not_why(rest)` (never called on the trace corpus)
pub fn _when_not_why(rest: &str) -> bool {
    crate::unported!("bot.modules.chat:_when_not_why")
}

/// Python: bot/modules/chat.py:909 `_judged_a_role(clean)` (never called on the trace corpus)
pub fn _judged_a_role(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_judged_a_role")
}

/// Python: bot/modules/chat.py:927 `_judgement(clean, tokens)`
pub fn _judgement(clean: &str, tokens: &[String]) {
    crate::unported!("bot.modules.chat:_judgement")
}

/// Python: bot/modules/chat.py:1313 `_a_job_that_went(said)`
pub fn _a_job_that_went(said: &str) -> bool {
    crate::unported!("bot.modules.chat:_a_job_that_went")
}

/// Python: bot/modules/chat.py:1327 `_the_ask_after_a_job_that_went(clean, memory)`
pub fn _the_ask_after_a_job_that_went(clean: &str, memory: &Memory) {
    crate::unported!("bot.modules.chat:_the_ask_after_a_job_that_went")
}

/// Python: bot/modules/chat.py:1467 `_not_a_night_out(place)` (never called on the trace corpus)
pub fn _not_a_night_out(place: &str) -> bool {
    crate::unported!("bot.modules.chat:_not_a_night_out")
}

/// Python: bot/modules/chat.py:1510 `_poor_word(flat)` (never called on the trace corpus)
pub fn _poor_word(flat: &Value) -> Option<String> {
    crate::unported!("bot.modules.chat:_poor_word")
}

/// Python: bot/modules/chat.py:1652 `_asked_about_the_effort(memory)` (never called on the trace corpus)
pub fn _asked_about_the_effort(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_asked_about_the_effort")
}

/// Python: bot/modules/chat.py:2267 `_as_a_pattern(line)` (never called on the trace corpus)
pub fn _as_a_pattern(line: &str) -> Value {
    crate::unported!("bot.modules.chat:_as_a_pattern")
}

/// Python: bot/modules/chat.py:2283 `hands_it_back(reply)`
pub fn hands_it_back(reply: &str) -> bool {
    crate::unported!("bot.modules.chat:hands_it_back")
}

/// Python: bot/modules/chat.py:2495 `_a_thing_in_front(clean, word)`
pub fn _a_thing_in_front(clean: &str, word: &str) -> bool {
    crate::unported!("bot.modules.chat:_a_thing_in_front")
}

/// Python: bot/modules/chat.py:2657 `_body_is_a_feeling(clean)`
pub fn _body_is_a_feeling(clean: &str) {
    crate::unported!("bot.modules.chat:_body_is_a_feeling")
}

/// Python: bot/modules/chat.py:2687 `_the_kind_of_worry(kind, text)` (never called on the trace corpus)
pub fn _the_kind_of_worry(kind: &str, text: &str) -> Value {
    crate::unported!("bot.modules.chat:_the_kind_of_worry")
}

/// Python: bot/modules/chat.py:2701 `_line_as_read(text)`
pub fn _line_as_read(text: &str) -> String {
    crate::unported!("bot.modules.chat:_line_as_read")
}

/// Python: bot/modules/chat.py:2707 `_a_limit(text, memory)`
pub fn _a_limit(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.chat:_a_limit")
}

/// Python: bot/modules/chat.py:2816 `_their_own_call(text)`
pub fn _their_own_call(text: &str) {
    crate::unported!("bot.modules.chat:_their_own_call")
}

/// Python: bot/modules/chat.py:2850 `_without_the_feeling_avoided(clean, understood)`
pub fn _without_the_feeling_avoided(clean: &str, understood: Option<&Understood>) -> String {
    crate::unported!("bot.modules.chat:_without_the_feeling_avoided")
}

/// Python: bot/modules/chat.py:2877 `_negated(clean, word)`
pub fn _negated(clean: &str, word: &str) -> bool {
    crate::unported!("bot.modules.chat:_negated")
}

/// Python: bot/modules/chat.py:2910 `_denies_an_earlier_word(clean, word)` (never called on the trace corpus)
pub fn _denies_an_earlier_word(clean: &str, word: &str) -> bool {
    crate::unported!("bot.modules.chat:_denies_an_earlier_word")
}

/// Python: bot/modules/chat.py:2992 `_fond_of_the_bot(clean)`
pub fn _fond_of_the_bot(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_fond_of_the_bot")
}

/// Python: bot/modules/chat.py:3091 `_praised_the_answer(clean)`
pub fn _praised_the_answer(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_praised_the_answer")
}

/// Python: bot/modules/chat.py:3254 `_the_same_cold_again(clean, memory)` (never called on the trace corpus)
pub fn _the_same_cold_again(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_the_same_cold_again")
}

/// Python: bot/modules/chat.py:3346 `_a_stretch_that_drags(clean)`
pub fn _a_stretch_that_drags(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_a_stretch_that_drags")
}

/// Python: bot/modules/chat.py:3550 `_doing_something_to_a_thing(clean)`
pub fn _doing_something_to_a_thing(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_doing_something_to_a_thing")
}

/// Python: bot/modules/chat.py:3592 `said_loss(memory)`
pub fn said_loss(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:said_loss")
}

/// Python: bot/modules/chat.py:3697 `_asks_about_plans(said)`
pub fn _asks_about_plans(said: &str) -> bool {
    crate::unported!("bot.modules.chat:_asks_about_plans")
}

/// Python: bot/modules/chat.py:3827 `_a_feeling_named(word)` (never called on the trace corpus)
pub fn _a_feeling_named(word: &str) -> bool {
    crate::unported!("bot.modules.chat:_a_feeling_named")
}

/// Python: bot/modules/chat.py:3861 `_a_feeling_about_me(word)` (never called on the trace corpus)
pub fn _a_feeling_about_me(word: &str) -> Value {
    crate::unported!("bot.modules.chat:_a_feeling_about_me")
}

/// Python: bot/modules/chat.py:4185 `_about_bot_life(understood)`
pub fn _about_bot_life(understood: &Understood) {
    crate::unported!("bot.modules.chat:_about_bot_life")
}

/// Python: bot/modules/chat.py:4196 `_as_meant(clean, understood)`
pub fn _as_meant(clean: &str, understood: &Understood) -> String {
    crate::unported!("bot.modules.chat:_as_meant")
}

/// Python: bot/modules/chat.py:4278 `_a_shape_no_list_knows(words)` (never called on the trace corpus)
pub fn _a_shape_no_list_knows(words: &[String]) -> bool {
    crate::unported!("bot.modules.chat:_a_shape_no_list_knows")
}

/// Python: bot/modules/chat.py:4285 `_doings_pool(words)` (never called on the trace corpus)
pub fn _doings_pool(words: &[String]) -> Value {
    crate::unported!("bot.modules.chat:_doings_pool")
}

/// Python: bot/modules/chat.py:4300 `_a_sign_off_with_a_time(understood)` (never called on the trace corpus)
pub fn _a_sign_off_with_a_time(understood: &Understood) -> bool {
    crate::unported!("bot.modules.chat:_a_sign_off_with_a_time")
}

/// Python: bot/modules/chat.py:4305 `_about_bot_doings(understood)`
pub fn _about_bot_doings(understood: &Understood) {
    crate::unported!("bot.modules.chat:_about_bot_doings")
}

/// Python: bot/modules/chat.py:4327 `_about_bot(clean, understood=None, memory=None)`
pub fn _about_bot(clean: &str, understood: &Understood, memory: &Memory) {
    crate::unported!("bot.modules.chat:_about_bot")
}

/// Python: bot/modules/chat.py:4405 `_self_only(clean, match)` (never called on the trace corpus)
pub fn _self_only(clean: &str, r#match: &Value) -> bool {
    crate::unported!("bot.modules.chat:_self_only")
}

/// Python: bot/modules/chat.py:4416 `_opinion(clean, tokens, memory)`
pub fn _opinion(clean: &str, tokens: &[String], memory: &Memory) {
    crate::unported!("bot.modules.chat:_opinion")
}

/// Python: bot/modules/chat.py:4436 `_context_follow_up(clean, memory)`
pub fn _context_follow_up(clean: &str, memory: &Memory) {
    crate::unported!("bot.modules.chat:_context_follow_up")
}

/// Python: bot/modules/chat.py:4775 `_a_trip(flat, memory)`
pub fn _a_trip(flat: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_a_trip")
}

/// Python: bot/modules/chat.py:4791 `_answers_it(text, understood, memory)`
pub fn _answers_it(text: &str, understood: &Understood, memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_answers_it")
}

/// Python: bot/modules/chat.py:4818 `_intent(clean, tokens, memory)`
pub fn _intent(clean: &str, tokens: &[String], memory: &Memory) {
    crate::unported!("bot.modules.chat:_intent")
}

/// Python: bot/modules/chat.py:4918 `_as_typed(flat, memory)`
pub fn _as_typed(flat: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.chat:_as_typed")
}

/// Python: bot/modules/chat.py:4940 `_a_detail_on_the_job(flat, memory)`
pub fn _a_detail_on_the_job(flat: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_a_detail_on_the_job")
}

/// Python: bot/modules/chat.py:4980 `_overloaded(flat, memory)`
pub fn _overloaded(flat: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_overloaded")
}

/// Python: bot/modules/chat.py:5027 `_judged(clean, memory)`
pub fn _judged(clean: &str, memory: &Memory) {
    crate::unported!("bot.modules.chat:_judged")
}

/// Python: bot/modules/chat.py:5076 `_judged_thing(clean, memory)` (never called on the trace corpus)
pub fn _judged_thing(clean: &str, memory: &Memory) -> Value {
    crate::unported!("bot.modules.chat:_judged_thing")
}

/// Python: bot/modules/chat.py:5134 `_the_steps_again(memory)` (never called on the trace corpus)
pub fn _the_steps_again(memory: &Memory) -> Value {
    crate::unported!("bot.modules.chat:_the_steps_again")
}

/// Python: bot/modules/chat.py:5155 `_a_care_turn(memory)` (never called on the trace corpus)
pub fn _a_care_turn(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_a_care_turn")
}

/// Python: bot/modules/chat.py:5169 `_short_talk(clean, tokens, memory)`
pub fn _short_talk(clean: &str, tokens: &[String], memory: &Memory) {
    crate::unported!("bot.modules.chat:_short_talk")
}

/// Python: bot/modules/chat.py:5226 `_a_forecast(memory)`
pub fn _a_forecast(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_a_forecast")
}

/// Python: bot/modules/chat.py:5268 `_tokens(text)`
pub fn _tokens(text: &str) -> Vec<String> {
    crate::unported!("bot.modules.chat:_tokens")
}

/// Python: bot/modules/chat.py:5276 `_name(memory)`
pub fn _name(memory: &Memory) {
    crate::unported!("bot.modules.chat:_name")
}

/// Python: bot/modules/chat.py:5285 `_the_lost_thread_asked(memory, kind)`
pub fn _the_lost_thread_asked(memory: &mut Memory, kind: &str) {
    crate::unported!("bot.modules.chat:_the_lost_thread_asked")
}

/// Python: bot/modules/chat.py:5297 `_after_a_lost_thread(clean, understood, memory)`
pub fn _after_a_lost_thread(clean: &str, understood: &Understood, memory: &mut Memory) {
    crate::unported!("bot.modules.chat:_after_a_lost_thread")
}

/// Python: bot/modules/chat.py:5329 `_the_weight_said(memory, feeling)`
pub fn _the_weight_said(memory: &mut Memory, feeling: &str) {
    crate::unported!("bot.modules.chat:_the_weight_said")
}

/// Python: bot/modules/chat.py:5432 `_nothing_much_span(words)` (never called on the trace corpus)
pub fn _nothing_much_span(words: &[String]) -> Value {
    crate::unported!("bot.modules.chat:_nothing_much_span")
}

/// Python: bot/modules/chat.py:5459 `_says_nothing_much(tokens, clean='')`
pub fn _says_nothing_much(tokens: &[String], clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_says_nothing_much")
}

/// Python: bot/modules/chat.py:5507 `_the_line_on_the_screen(memory)`
pub fn _the_line_on_the_screen(memory: &Memory) -> String {
    crate::unported!("bot.modules.chat:_the_line_on_the_screen")
}

/// Python: bot/modules/chat.py:5517 `_thanks_that_fit(options, memory)`
pub fn _thanks_that_fit(options: &[String], memory: &Memory) -> Vec<String> {
    crate::unported!("bot.modules.chat:_thanks_that_fit")
}

/// Python: bot/modules/chat.py:5540 `_reply(kind, memory, name=None, seed='')`
pub fn _reply(kind: &str, memory: &mut Memory, name: &str, seed: &str) -> String {
    crate::unported!("bot.modules.chat:_reply")
}

/// Python: bot/modules/chat.py:5598 `_the_part_of_day(memory)`
pub fn _the_part_of_day(memory: &Memory) {
    crate::unported!("bot.modules.chat:_the_part_of_day")
}

/// Python: bot/modules/chat.py:5608 `_says_the_time_of_day(memory)` (never called on the trace corpus)
pub fn _says_the_time_of_day(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_says_the_time_of_day")
}

/// Python: bot/modules/chat.py:5620 `_mid_chat(memory)`
pub fn _mid_chat(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_mid_chat")
}

/// Python: bot/modules/chat.py:5629 `_just_met(memory)`
pub fn _just_met(memory: &Memory) -> Value {
    crate::unported!("bot.modules.chat:_just_met")
}

/// Python: bot/modules/chat.py:5669 `_ing(word)` (never called on the trace corpus)
pub fn _ing(word: &str) -> String {
    crate::unported!("bot.modules.chat:_ing")
}

/// Python: bot/modules/chat.py:5681 `_plan_clause(plan)` (never called on the trace corpus)
pub fn _plan_clause(plan: &str) -> String {
    crate::unported!("bot.modules.chat:_plan_clause")
}

/// Python: bot/modules/chat.py:5710 `_is_greeting(tokens)`
pub fn _is_greeting(tokens: &[String]) -> bool {
    crate::unported!("bot.modules.chat:_is_greeting")
}

/// Python: bot/modules/chat.py:5785 `_a_hedge_answering_the_question(memory)`
pub fn _a_hedge_answering_the_question(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_a_hedge_answering_the_question")
}

/// Python: bot/modules/chat.py:5827 `_denies_a_good_word(tokens)` (never called on the trace corpus)
pub fn _denies_a_good_word(tokens: &[String]) -> bool {
    crate::unported!("bot.modules.chat:_denies_a_good_word")
}

/// Python: bot/modules/chat.py:5847 `_is_thanks(tokens)`
pub fn _is_thanks(tokens: &[String]) -> bool {
    crate::unported!("bot.modules.chat:_is_thanks")
}

/// Python: bot/modules/chat.py:5866 `_without_the_at_all(tokens)`
pub fn _without_the_at_all(tokens: &[String]) -> Vec<String> {
    crate::unported!("bot.modules.chat:_without_the_at_all")
}

/// Python: bot/modules/chat.py:5877 `_without_the_nod(tokens)`
pub fn _without_the_nod(tokens: &[String]) -> Vec<String> {
    crate::unported!("bot.modules.chat:_without_the_nod")
}

/// Python: bot/modules/chat.py:5929 `_is_howareyou(tokens)`
pub fn _is_howareyou(tokens: &[String]) -> bool {
    crate::unported!("bot.modules.chat:_is_howareyou")
}

/// Python: bot/modules/chat.py:5965 `_asks_about_the_make(line)` (never called on the trace corpus)
pub fn _asks_about_the_make(line: &str) -> Option<bool> {
    crate::unported!("bot.modules.chat:_asks_about_the_make")
}

/// Python: bot/modules/chat.py:5973 `_is_bye(tokens)`
pub fn _is_bye(tokens: &[String]) -> bool {
    crate::unported!("bot.modules.chat:_is_bye")
}

/// Python: bot/modules/chat.py:6022 `_leaves(tokens)`
pub fn _leaves(tokens: &[String]) -> bool {
    crate::unported!("bot.modules.chat:_leaves")
}

/// Python: bot/modules/chat.py:6070 `_bye_kind(tokens)` (never called on the trace corpus)
pub fn _bye_kind(tokens: &[String]) -> String {
    crate::unported!("bot.modules.chat:_bye_kind")
}

/// Python: bot/modules/chat.py:6075 `_ending_bye(clean, memory)`
pub fn _ending_bye(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_ending_bye")
}

/// Python: bot/modules/chat.py:6259 `_frequency_of_the_last_trouble(clean, memory=None)`
pub fn _frequency_of_the_last_trouble(clean: &str, memory: &Memory) {
    crate::unported!("bot.modules.chat:_frequency_of_the_last_trouble")
}

/// Python: bot/modules/chat.py:6308 `_answers_has_the_frequency(clean, memory)`
pub fn _answers_has_the_frequency(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_answers_has_the_frequency")
}

/// Python: bot/modules/chat.py:6338 `_frequency_is_the_turn(clean, memory=None)`
pub fn _frequency_is_the_turn(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_frequency_is_the_turn")
}

/// Python: bot/modules/chat.py:6358 `after(text, reply, memory)` - a runtime hook; its signature is the runtime's.
pub fn after(text: &str, reply: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.chat:after")
}

/// Python: bot/modules/chat.py:6387 `_early_start(clean, memory=None)`
pub fn _early_start(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    crate::unported!("bot.modules.chat:_early_start")
}

/// Python: bot/modules/chat.py:6412 `_worry_kind(clean)`
pub fn _worry_kind(clean: &str) -> String {
    crate::unported!("bot.modules.chat:_worry_kind")
}

/// Python: bot/modules/chat.py:6440 `_without_skills(clean)`
pub fn _without_skills(clean: &str) -> String {
    crate::unported!("bot.modules.chat:_without_skills")
}

/// Python: bot/modules/chat.py:6548 `_the_bot_just_asked_them(memory)`
pub fn _the_bot_just_asked_them(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_the_bot_just_asked_them")
}

/// Python: bot/modules/chat.py:6569 `_a_thank_you_taken_off(clean, memory=None)`
pub fn _a_thank_you_taken_off(clean: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.chat:_a_thank_you_taken_off")
}

/// Python: bot/modules/chat.py:6597 `_a_plain_good_answer(clean, memory=None)`
pub fn _a_plain_good_answer(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_a_plain_good_answer")
}

/// Python: bot/modules/chat.py:6630 `_bare_feeling(clean)`
pub fn _bare_feeling(clean: &str) {
    crate::unported!("bot.modules.chat:_bare_feeling")
}

/// Python: bot/modules/chat.py:6679 `_says_relief(clean)`
pub fn _says_relief(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_says_relief")
}

/// Python: bot/modules/chat.py:6706 `_starts_a_clause(after)`
pub fn _starts_a_clause(after: &[String]) -> bool {
    crate::unported!("bot.modules.chat:_starts_a_clause")
}

/// Python: bot/modules/chat.py:6711 `_in_its_clause(before)`
pub fn _in_its_clause(before: &[String]) -> Vec<String> {
    crate::unported!("bot.modules.chat:_in_its_clause")
}

/// Python: bot/modules/chat.py:6722 `_owned_by_a_thing(clean, match)`
pub fn _owned_by_a_thing(clean: &str, r#match: &Value) -> bool {
    crate::unported!("bot.modules.chat:_owned_by_a_thing")
}

/// Python: bot/modules/chat.py:6757 `_own_state(clean)`
pub fn _own_state(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_own_state")
}

/// Python: bot/modules/chat.py:6768 `_said_of_the_person(clean, match)`
pub fn _said_of_the_person(clean: &str, r#match: &Value) -> bool {
    crate::unported!("bot.modules.chat:_said_of_the_person")
}

/// Python: bot/modules/chat.py:6796 `_carries_on(word)`
pub fn _carries_on(word: &str) -> bool {
    crate::unported!("bot.modules.chat:_carries_on")
}

/// Python: bot/modules/chat.py:6804 `_owner(before)`
pub fn _owner(before: &[String]) -> Option<String> {
    crate::unported!("bot.modules.chat:_owner")
}

/// Python: bot/modules/chat.py:6845 `_hard_about_their_thing(clean)`
pub fn _hard_about_their_thing(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_hard_about_their_thing")
}

/// Python: bot/modules/chat.py:6861 `_a_person_who_wears_you_out(clean, word, match)`
pub fn _a_person_who_wears_you_out(clean: &str, word: &str, r#match: &Value) -> bool {
    crate::unported!("bot.modules.chat:_a_person_who_wears_you_out")
}

/// Python: bot/modules/chat.py:6871 `_says_how_the_person_is(clean, word, match)`
pub fn _says_how_the_person_is(clean: &str, word: &str, r#match: &Value) -> bool {
    crate::unported!("bot.modules.chat:_says_how_the_person_is")
}

/// Python: bot/modules/chat.py:7006 `_my_own_tears(clean)` (never called on the trace corpus)
pub fn _my_own_tears(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_my_own_tears")
}

/// Python: bot/modules/chat.py:7022 `_a_low_word(clean)` (never called on the trace corpus)
pub fn _a_low_word(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_a_low_word")
}

/// Python: bot/modules/chat.py:7033 `_good_news(clean)` (never called on the trace corpus)
pub fn _good_news(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_good_news")
}

/// Python: bot/modules/chat.py:7071 `_weeping(clean)`
pub fn _weeping(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_weeping")
}

/// Python: bot/modules/chat.py:7084 `_tears_said_as_a_verb(clean)`
pub fn _tears_said_as_a_verb(clean: &str) {
    crate::unported!("bot.modules.chat:_tears_said_as_a_verb")
}

/// Python: bot/modules/chat.py:7279 `_not_coping(clean)`
pub fn _not_coping(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_not_coping")
}

/// Python: bot/modules/chat.py:7321 `_speaker_going_mad(text)`
pub fn _speaker_going_mad(text: &str) -> bool {
    crate::unported!("bot.modules.chat:_speaker_going_mad")
}

/// Python: bot/modules/chat.py:7340 `_bad_day(clean)`
pub fn _bad_day(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_bad_day")
}

/// Python: bot/modules/chat.py:7378 `_scattered(clean)`
pub fn _scattered(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_scattered")
}

/// Python: bot/modules/chat.py:7448 `_a_bare_acknowledgement(clean)`
pub fn _a_bare_acknowledgement(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_a_bare_acknowledgement")
}

/// Python: bot/modules/chat.py:7483 `_the_heavy_line(memory)` (never called on the trace corpus)
pub fn _the_heavy_line(memory: &Memory) -> Value {
    crate::unported!("bot.modules.chat:_the_heavy_line")
}

/// Python: bot/modules/chat.py:7536 `_apart_from_people(clean)`
pub fn _apart_from_people(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_apart_from_people")
}

/// Python: bot/modules/chat.py:7552 `_after_lonely(clean, memory)`
pub fn _after_lonely(clean: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.chat:_after_lonely")
}

/// Python: bot/modules/chat.py:7586 `_idk_kind(memory)` (never called on the trace corpus)
pub fn _idk_kind(memory: &Memory) -> String {
    crate::unported!("bot.modules.chat:_idk_kind")
}

/// Python: bot/modules/chat.py:7619 `_offered_a_choice(said)` (never called on the trace corpus)
pub fn _offered_a_choice(said: &str) -> bool {
    crate::unported!("bot.modules.chat:_offered_a_choice")
}

/// Python: bot/modules/chat.py:7635 `_a_low_just_before(memory)`
pub fn _a_low_just_before(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_a_low_just_before")
}

/// Python: bot/modules/chat.py:7678 `_nothing_helped_just_before(memory, clean)`
pub fn _nothing_helped_just_before(memory: &Memory, clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_nothing_helped_just_before")
}

/// Python: bot/modules/chat.py:7695 `_nothing_helps(clean, memory)`
pub fn _nothing_helps(clean: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.chat:_nothing_helps")
}

/// Python: bot/modules/chat.py:7705 `_as_said(clean, understood)`
pub fn _as_said(clean: &str, understood: Option<&Understood>) -> String {
    crate::unported!("bot.modules.chat:_as_said")
}

/// Python: bot/modules/chat.py:7735 `_without_the_might(clean)`
pub fn _without_the_might(clean: &str) -> String {
    crate::unported!("bot.modules.chat:_without_the_might")
}

/// Python: bot/modules/chat.py:7775 `_feeling(clean, understood=None, bare=None, kept=False)`
pub fn _feeling(clean: &str, understood: Option<&Understood>, bare: &Value, kept: bool) -> Option<String> {
    crate::unported!("bot.modules.chat:_feeling")
}

/// Python: bot/modules/chat.py:8250 `_asked_how_it_went(line)`
pub fn _asked_how_it_went(line: &str) -> bool {
    crate::unported!("bot.modules.chat:_asked_how_it_went")
}

/// Python: bot/modules/chat.py:8292 `_a_judgement_asked(line)`
pub fn _a_judgement_asked(line: &str) -> String {
    crate::unported!("bot.modules.chat:_a_judgement_asked")
}

/// Python: bot/modules/chat.py:8322 `_is_a_nod(line)` (never called on the trace corpus)
pub fn _is_a_nod(line: &str) -> bool {
    crate::unported!("bot.modules.chat:_is_a_nod")
}

/// Python: bot/modules/chat.py:8331 `_the_nod(memory)` (never called on the trace corpus)
pub fn _the_nod(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.modules.chat:_the_nod")
}

/// Python: bot/modules/chat.py:8445 `_a_doubt_about_being_taken_on(clean, memory)`
pub fn _a_doubt_about_being_taken_on(clean: &str, memory: &Memory) {
    crate::unported!("bot.modules.chat:_a_doubt_about_being_taken_on")
}

/// Python: bot/modules/chat.py:8463 `_in_a_work_thread(memory)` (never called on the trace corpus)
pub fn _in_a_work_thread(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_in_a_work_thread")
}

/// Python: bot/modules/chat.py:8484 `_the_age_in_a_work_thread(clean, memory)`
pub fn _the_age_in_a_work_thread(clean: &str, memory: &Memory) {
    crate::unported!("bot.modules.chat:_the_age_in_a_work_thread")
}

/// Python: bot/modules/chat.py:8505 `_going_on(clean)`
pub fn _going_on(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_going_on")
}

/// Python: bot/modules/chat.py:8773 `_a_journey_not_progress(clean, match)` (never called on the trace corpus)
pub fn _a_journey_not_progress(clean: &str, r#match: &Value) -> bool {
    crate::unported!("bot.modules.chat:_a_journey_not_progress")
}

/// Python: bot/modules/chat.py:8808 `_said_about_themselves(before)` (never called on the trace corpus)
pub fn _said_about_themselves(before: &str) -> bool {
    crate::unported!("bot.modules.chat:_said_about_themselves")
}

/// Python: bot/modules/chat.py:8831 `_feel_like(clean)`
pub fn _feel_like(clean: &str) {
    crate::unported!("bot.modules.chat:_feel_like")
}

/// Python: bot/modules/chat.py:8958 `_a_feeling_riding_on_the_thread(clean, memory)` (never called on the trace corpus)
pub fn _a_feeling_riding_on_the_thread(clean: &str, memory: &Memory) -> Option<bool> {
    crate::unported!("bot.modules.chat:_a_feeling_riding_on_the_thread")
}

/// Python: bot/modules/chat.py:8971 `_against_me(clean)`
pub fn _against_me(clean: &str) {
    crate::unported!("bot.modules.chat:_against_me")
}

/// Python: bot/modules/chat.py:9059 `_worry_someone(clean, memory=None)`
pub fn _worry_someone(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    crate::unported!("bot.modules.chat:_worry_someone")
}

/// Python: bot/modules/chat.py:9083 `_loss_recent(memory)`
pub fn _loss_recent(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_loss_recent")
}

/// Python: bot/modules/chat.py:9094 `_low_mood_recent(memory)`
pub fn _low_mood_recent(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_low_mood_recent")
}

/// Python: bot/modules/chat.py:9154 `_a_thing_cannot(clean)` (never called on the trace corpus)
pub fn _a_thing_cannot(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_a_thing_cannot")
}

/// Python: bot/modules/chat.py:9392 `_checks_the_fact(text, memory)`
pub fn _checks_the_fact(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_checks_the_fact")
}

/// Python: bot/modules/chat.py:9399 `_a_follow_up_about_their_person(clean, memory)`
pub fn _a_follow_up_about_their_person(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_a_follow_up_about_their_person")
}

/// Python: bot/modules/chat.py:9427 `TAKES_AS_IT_COMES(text, memory=None)`
pub fn TAKES_AS_IT_COMES(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.chat:TAKES_AS_IT_COMES")
}

/// Python: bot/modules/chat.py:9649 `_close_nouns(text)` (never called on the trace corpus)
pub fn _close_nouns(text: &str) -> Value {
    crate::unported!("bot.modules.chat:_close_nouns")
}

/// Python: bot/modules/chat.py:9654 `_names_the_person(clean, row=None)` (never called on the trace corpus)
pub fn _names_the_person(clean: &str, row: &'static Entry) -> bool {
    crate::unported!("bot.modules.chat:_names_the_person")
}

/// Python: bot/modules/chat.py:9690 `_a_row_person(clean)`
pub fn _a_row_person(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_a_row_person")
}

/// Python: bot/modules/chat.py:9701 `_keep_the_row(clean, memory)`
pub fn _keep_the_row(clean: &str, memory: &mut Memory) -> Value {
    crate::unported!("bot.modules.chat:_keep_the_row")
}

/// Python: bot/modules/chat.py:9726 `observe(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn observe(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.chat:observe")
}

/// Python: bot/modules/chat.py:9750 `_the_base_has_an_answer(clean, memory)`
pub fn _the_base_has_an_answer(clean: &str, memory: &Memory) -> &'static Entry {
    crate::unported!("bot.modules.chat:_the_base_has_an_answer")
}

/// Python: bot/modules/chat.py:9776 `_the_row_names_its_subject(clean)`
pub fn _the_row_names_its_subject(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_the_row_names_its_subject")
}

/// Python: bot/modules/chat.py:9784 `_the_base_answers_the_row(clean, memory)`
pub fn _the_base_answers_the_row(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_the_base_answers_the_row")
}

/// Python: bot/modules/chat.py:9803 `_a_row(clean, memory=None)`
pub fn _a_row(clean: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.chat:_a_row")
}

/// Python: bot/modules/chat.py:9939 `_a_teen_has_gone_quiet(text)` (never called on the trace corpus)
pub fn _a_teen_has_gone_quiet(text: &str) -> bool {
    crate::unported!("bot.modules.chat:_a_teen_has_gone_quiet")
}

/// Python: bot/modules/chat.py:9949 `_a_quarrel_since(memory, turn)` (never called on the trace corpus)
pub fn _a_quarrel_since(memory: &Memory, turn: &Value) -> bool {
    crate::unported!("bot.modules.chat:_a_quarrel_since")
}

/// Python: bot/modules/chat.py:9958 `_the_ask_in_the_line(clean)`
pub fn _the_ask_in_the_line(clean: &str) -> String {
    crate::unported!("bot.modules.chat:_the_ask_in_the_line")
}

/// Python: bot/modules/chat.py:9973 `_a_row_next(clean, memory)`
pub fn _a_row_next(clean: &str, memory: &Memory) {
    crate::unported!("bot.modules.chat:_a_row_next")
}

/// Python: bot/modules/chat.py:10055 `_a_row_ending(clean, memory)`
pub fn _a_row_ending(clean: &str, memory: &Memory) {
    crate::unported!("bot.modules.chat:_a_row_ending")
}

/// Python: bot/modules/chat.py:10080 `_row_pronoun(clean)`
pub fn _row_pronoun(clean: &str) -> String {
    crate::unported!("bot.modules.chat:_row_pronoun")
}

/// Python: bot/modules/chat.py:10089 `_their_of(them)`
pub fn _their_of(them: &str) -> String {
    crate::unported!("bot.modules.chat:_their_of")
}

/// Python: bot/modules/chat.py:10099 `_they_of(row)` (never called on the trace corpus)
pub fn _they_of(row: &'static Entry) -> String {
    crate::unported!("bot.modules.chat:_they_of")
}

/// Python: bot/modules/chat.py:10104 `_row_subject(clean)`
pub fn _row_subject(clean: &str) -> String {
    crate::unported!("bot.modules.chat:_row_subject")
}

/// Python: bot/modules/chat.py:10113 `_the_deed_before(memory)` (never called on the trace corpus)
pub fn _the_deed_before(memory: &Memory) -> Value {
    crate::unported!("bot.modules.chat:_the_deed_before")
}

/// Python: bot/modules/chat.py:10121 `_after_a_repair(memory)`
pub fn _after_a_repair(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_after_a_repair")
}

/// Python: bot/modules/chat.py:10138 `_a_repair_is_open(memory)`
pub fn _a_repair_is_open(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_a_repair_is_open")
}

/// Python: bot/modules/chat.py:10148 `_about_someone(clean, memory)`
pub fn _about_someone(clean: &str, memory: &Memory) {
    crate::unported!("bot.modules.chat:_about_someone")
}

/// Python: bot/modules/chat.py:10263 `_after_a_plan(memory, clean)`
pub fn _after_a_plan(memory: &Memory, clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_after_a_plan")
}

/// Python: bot/modules/chat.py:10298 `_grieving(memory, clean)`
pub fn _grieving(memory: &Memory, clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_grieving")
}

/// Python: bot/modules/chat.py:10322 `_better_answer_exists(clean, memory)` (never called on the trace corpus)
pub fn _better_answer_exists(clean: &str, memory: &Memory) -> Option<bool> {
    crate::unported!("bot.modules.chat:_better_answer_exists")
}

/// Python: bot/modules/chat.py:10338 `_still_about_me(clean)` (never called on the trace corpus)
pub fn _still_about_me(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_still_about_me")
}

/// Python: bot/modules/chat.py:10349 `_about_the_person(understood)`
pub fn _about_the_person(understood: &Understood) -> bool {
    crate::unported!("bot.modules.chat:_about_the_person")
}

/// Python: bot/modules/chat.py:10357 `_my_night(clean)`
pub fn _my_night(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_my_night")
}

/// Python: bot/modules/chat.py:10365 `_stuck(clean, memory=None)`
pub fn _stuck(clean: &str, memory: &Memory) {
    crate::unported!("bot.modules.chat:_stuck")
}

/// Python: bot/modules/chat.py:10465 `_names_a_kind_of_thing(between, after)`
pub fn _names_a_kind_of_thing(between: &str, after: &str) -> bool {
    crate::unported!("bot.modules.chat:_names_a_kind_of_thing")
}

/// Python: bot/modules/chat.py:10639 `_recent_plural(memory)`
pub fn _recent_plural(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_recent_plural")
}

/// Python: bot/modules/chat.py:10653 `_pronouns(word)` (never called on the trace corpus)
pub fn _pronouns(word: &str) -> Value {
    crate::unported!("bot.modules.chat:_pronouns")
}

/// Python: bot/modules/chat.py:10680 `_other_feeling(clean, memory=None)`
pub fn _other_feeling(clean: &str, memory: &Memory) {
    crate::unported!("bot.modules.chat:_other_feeling")
}

/// Python: bot/modules/chat.py:10825 `_the_thing(phrase)` (never called on the trace corpus)
pub fn _the_thing(phrase: &str) -> Value {
    crate::unported!("bot.modules.chat:_the_thing")
}

/// Python: bot/modules/chat.py:10848 `_thing_that_stays(thing)` (never called on the trace corpus)
pub fn _thing_that_stays(thing: &str) -> bool {
    crate::unported!("bot.modules.chat:_thing_that_stays")
}

/// Python: bot/modules/chat.py:10902 `_confusion_names_a_thing(understood)`
pub fn _confusion_names_a_thing(understood: &Understood) -> bool {
    crate::unported!("bot.modules.chat:_confusion_names_a_thing")
}

/// Python: bot/modules/chat.py:10975 `_fedup_phrase(said, at)` (never called on the trace corpus)
pub fn _fedup_phrase(said: &str, at: i64) -> Value {
    crate::unported!("bot.modules.chat:_fedup_phrase")
}

/// Python: bot/modules/chat.py:10981 `_fed_up_with(clean)` (never called on the trace corpus)
pub fn _fed_up_with(clean: &str) -> Value {
    crate::unported!("bot.modules.chat:_fed_up_with")
}

/// Python: bot/modules/chat.py:11026 `_the_small_child(memory)` (never called on the trace corpus)
pub fn _the_small_child(memory: &Memory) -> Value {
    crate::unported!("bot.modules.chat:_the_small_child")
}

/// Python: bot/modules/chat.py:11045 `_someone_reply(memory, feeling, obj, poss, plural, thing='', said='')` (never called on the trace corpus)
pub fn _someone_reply(memory: &Memory, feeling: &Value, obj: &Value, poss: &Value, plural: &Value, thing: &str, said: &str) -> Value {
    crate::unported!("bot.modules.chat:_someone_reply")
}

/// Python: bot/modules/chat.py:11084 `_last_was_joke(memory)`
pub fn _last_was_joke(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_last_was_joke")
}

/// Python: bot/modules/chat.py:11091 `_drop_tail(phrase)` (never called on the trace corpus)
pub fn _drop_tail(phrase: &str) -> String {
    crate::unported!("bot.modules.chat:_drop_tail")
}

/// Python: bot/modules/chat.py:11108 `_place_of(phrase)` (never called on the trace corpus)
pub fn _place_of(phrase: &str) -> Value {
    crate::unported!("bot.modules.chat:_place_of")
}

/// Python: bot/modules/chat.py:11151 `_tidy(phrase)` (never called on the trace corpus)
pub fn _tidy(phrase: &str) -> Value {
    crate::unported!("bot.modules.chat:_tidy")
}

/// Python: bot/modules/chat.py:11169 `_numbers(values)` (never called on the trace corpus)
pub fn _numbers(values: &Value) -> String {
    crate::unported!("bot.modules.chat:_numbers")
}

/// Python: bot/modules/chat.py:11177 `_place_values(named)` (never called on the trace corpus)
pub fn _place_values(named: &Value) -> Value {
    crate::unported!("bot.modules.chat:_place_values")
}

/// Python: bot/modules/chat.py:11194 `_say(kind, memory)`
pub fn _say(kind: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.modules.chat:_say")
}

/// Python: bot/modules/chat.py:11216 `_cheer_or_the_question(flat, memory)` (never called on the trace corpus)
pub fn _cheer_or_the_question(flat: &Value, memory: &mut Memory) -> Value {
    crate::unported!("bot.modules.chat:_cheer_or_the_question")
}

/// Python: bot/modules/chat.py:11231 `_note_the_pool(memory, kind)` (never called on the trace corpus)
pub fn _note_the_pool(memory: &mut Memory, kind: &str) {
    crate::unported!("bot.modules.chat:_note_the_pool")
}

/// Python: bot/modules/chat.py:11242 `_just_named(memory, name)`
pub fn _just_named(memory: &mut Memory, name: &str) {
    crate::unported!("bot.modules.chat:_just_named")
}

/// Python: bot/modules/chat.py:11257 `_name_said(name)` (never called on the trace corpus)
pub fn _name_said(name: &str) -> Value {
    crate::unported!("bot.modules.chat:_name_said")
}

/// Python: bot/modules/chat.py:11268 `_pet_name(flat, match)`
pub fn _pet_name(flat: &str, r#match: &Value) -> String {
    crate::unported!("bot.modules.chat:_pet_name")
}

/// Python: bot/modules/chat.py:11295 `_question_coming(understood)`
pub fn _question_coming(understood: &Understood) -> bool {
    crate::unported!("bot.modules.chat:_question_coming")
}

/// Python: bot/modules/chat.py:11318 `_the_not_knowing_has_a_subject(memory)` (never called on the trace corpus)
pub fn _the_not_knowing_has_a_subject(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_the_not_knowing_has_a_subject")
}

/// Python: bot/modules/chat.py:11337 `_conversation(clean, tokens, memory)`
pub fn _conversation(clean: &str, tokens: &[String], memory: &mut Memory) {
    crate::unported!("bot.modules.chat:_conversation")
}

/// Python: bot/modules/chat.py:11426 `_the_visit_was_this_week(flat)` (never called on the trace corpus)
pub fn _the_visit_was_this_week(flat: &Value) -> bool {
    crate::unported!("bot.modules.chat:_the_visit_was_this_week")
}

/// Python: bot/modules/chat.py:11525 `_a_thing_of_care(phrase, article=True)` (never called on the trace corpus)
pub fn _a_thing_of_care(phrase: &str, article: &Value) -> Value {
    crate::unported!("bot.modules.chat:_a_thing_of_care")
}

/// Python: bot/modules/chat.py:11550 `_ends_plural(phrase)` (never called on the trace corpus)
pub fn _ends_plural(phrase: &str) -> bool {
    crate::unported!("bot.modules.chat:_ends_plural")
}

/// Python: bot/modules/chat.py:11556 `_keep_the_letters(text)` (never called on the trace corpus)
pub fn _keep_the_letters(text: &str) -> Value {
    crate::unported!("bot.modules.chat:_keep_the_letters")
}

/// Python: bot/modules/chat.py:11578 `_wearer_is_themselves(flat, match)` (never called on the trace corpus)
pub fn _wearer_is_themselves(flat: &Value, r#match: &Value) -> bool {
    crate::unported!("bot.modules.chat:_wearer_is_themselves")
}

/// Python: bot/modules/chat.py:11596 `_names_a_person(word)` (never called on the trace corpus)
pub fn _names_a_person(word: &str) -> bool {
    crate::unported!("bot.modules.chat:_names_a_person")
}

/// Python: bot/modules/chat.py:11611 `_own_medical_news(flat, memory, clean='')`
pub fn _own_medical_news(flat: &str, memory: &mut Memory, clean: &str) {
    crate::unported!("bot.modules.chat:_own_medical_news")
}

/// Python: bot/modules/chat.py:11685 `_news(clean, tokens, memory)`
pub fn _news(clean: &str, tokens: &[String], memory: &mut Memory) {
    crate::unported!("bot.modules.chat:_news")
}

/// Python: bot/modules/chat.py:11925 `_head_word(phrase)` (never called on the trace corpus)
pub fn _head_word(phrase: &str) -> Value {
    crate::unported!("bot.modules.chat:_head_word")
}

/// Python: bot/modules/chat.py:11931 `_the_poor_describer(phrase)` (never called on the trace corpus)
pub fn _the_poor_describer(phrase: &str) -> Value {
    crate::unported!("bot.modules.chat:_the_poor_describer")
}

/// Python: bot/modules/chat.py:11939 `_the_thing_this_line_names(clean)` (never called on the trace corpus)
pub fn _the_thing_this_line_names(clean: &str) -> Value {
    crate::unported!("bot.modules.chat:_the_thing_this_line_names")
}

/// Python: bot/modules/chat.py:11951 `_about_the_night_out(clean, tokens, memory)`
pub fn _about_the_night_out(clean: &str, tokens: &[String], memory: &mut Memory) {
    crate::unported!("bot.modules.chat:_about_the_night_out")
}

/// Python: bot/modules/chat.py:11983 `_saw_something(flat, memory)`
pub fn _saw_something(flat: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.chat:_saw_something")
}

/// Python: bot/modules/chat.py:12031 `_trip_place(phrase, memory)` (never called on the trace corpus)
pub fn _trip_place(phrase: &str, memory: &Memory) -> Value {
    crate::unported!("bot.modules.chat:_trip_place")
}

/// Python: bot/modules/chat.py:12078 `_told_a_trip(flat, memory)`
pub fn _told_a_trip(flat: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.chat:_told_a_trip")
}

/// Python: bot/modules/chat.py:12221 `_the_thread_is_heavy_here(memory)` (never called on the trace corpus)
pub fn _the_thread_is_heavy_here(memory: &Memory) -> Option<bool> {
    crate::unported!("bot.modules.chat:_the_thread_is_heavy_here")
}

/// Python: bot/modules/chat.py:12229 `_a_good_time_they_had(flat, memory)`
pub fn _a_good_time_they_had(flat: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.chat:_a_good_time_they_had")
}

/// Python: bot/modules/chat.py:12244 `_the_detail_of_the_good_time(flat, clean, tokens, memory)`
pub fn _the_detail_of_the_good_time(flat: &str, clean: &str, tokens: &[String], memory: &Memory) {
    crate::unported!("bot.modules.chat:_the_detail_of_the_good_time")
}

/// Python: bot/modules/chat.py:12266 `_night_out(flat, memory)`
pub fn _night_out(flat: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.chat:_night_out")
}

/// Python: bot/modules/chat.py:12333 `_a_pick_is_in_hand(memory)`
pub fn _a_pick_is_in_hand(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_a_pick_is_in_hand")
}

/// Python: bot/modules/chat.py:12350 `_asks_for_a_joke(clean, tokens, memory=None)`
pub fn _asks_for_a_joke(clean: &str, tokens: &[String], memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_asks_for_a_joke")
}

/// Python: bot/modules/chat.py:12515 `_without_a_filler_tail(thing)`
pub fn _without_a_filler_tail(thing: &str) -> String {
    crate::unported!("bot.modules.chat:_without_a_filler_tail")
}

/// Python: bot/modules/chat.py:12670 `_kindless(word)` (never called on the trace corpus)
pub fn _kindless(word: &str) -> bool {
    crate::unported!("bot.modules.chat:_kindless")
}

/// Python: bot/modules/chat.py:12753 `_a_death_just_said(memory)`
pub fn _a_death_just_said(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_a_death_just_said")
}

/// Python: bot/modules/chat.py:12779 `_in_loss(memory)`
pub fn _in_loss(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_in_loss")
}

/// Python: bot/modules/chat.py:12788 `_answers_knows(text, memory)`
pub fn _answers_knows(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.chat:_answers_knows")
}

/// Python: bot/modules/chat.py:12819 `_verb_stem(word)`
pub fn _verb_stem(word: &str) -> String {
    crate::unported!("bot.modules.chat:_verb_stem")
}

/// Python: bot/modules/chat.py:12833 `_answers_is_about_the_thing(text, thing, memory, verbs=())`
pub fn _answers_is_about_the_thing(text: &str, thing: &str, memory: &Memory, verbs: &[String]) -> bool {
    crate::unported!("bot.modules.chat:_answers_is_about_the_thing")
}

/// Python: bot/modules/chat.py:12887 `_answers_has_the_how(thing, memory)`
pub fn _answers_has_the_how(thing: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_answers_has_the_how")
}

/// Python: bot/modules/chat.py:12908 `_fallback_has_its_own_pool(text)` (never called on the trace corpus)
pub fn _fallback_has_its_own_pool(text: &str) -> Option<bool> {
    crate::unported!("bot.modules.chat:_fallback_has_its_own_pool")
}

/// Python: bot/modules/chat.py:12931 `_the_words_they_meant(clean, understood)`
pub fn _the_words_they_meant(clean: &str, understood: &Understood) -> String {
    crate::unported!("bot.modules.chat:_the_words_they_meant")
}

/// Python: bot/modules/chat.py:12961 `_their_thing(clean, tokens, memory, understood=None)`
pub fn _their_thing(clean: &str, tokens: &[String], memory: &mut Memory, understood: &Understood) -> Option<String> {
    crate::unported!("bot.modules.chat:_their_thing")
}

/// Python: bot/modules/chat.py:13250 `_asks_outright(clean)`
pub fn _asks_outright(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_asks_outright")
}

/// Python: bot/modules/chat.py:13261 `_joined_read(memory)`
pub fn _joined_read(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_joined_read")
}

/// Python: bot/modules/chat.py:13274 `_joke(clean, tokens, memory)`
pub fn _joke(clean: &str, tokens: &[String], memory: &Memory) {
    crate::unported!("bot.modules.chat:_joke")
}

/// Python: bot/modules/chat.py:13319 `_normalised_line(memory)`
pub fn _normalised_line(memory: &Memory) -> Option<String> {
    crate::unported!("bot.modules.chat:_normalised_line")
}

/// Python: bot/modules/chat.py:13505 `_a_constant_tiredness(clean)` (never called on the trace corpus)
pub fn _a_constant_tiredness(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_a_constant_tiredness")
}

/// Python: bot/modules/chat.py:13510 `_standing(clean, memory)`
pub fn _standing(clean: &str, memory: &Memory) {
    crate::unported!("bot.modules.chat:_standing")
}

/// Python: bot/modules/chat.py:13572 `_repaired_line(memory)`
pub fn _repaired_line(memory: &Memory) -> String {
    crate::unported!("bot.modules.chat:_repaired_line")
}

/// Python: bot/modules/chat.py:13603 `_says_dont_know(clean, tokens)`
pub fn _says_dont_know(clean: &str, tokens: &[String]) -> bool {
    crate::unported!("bot.modules.chat:_says_dont_know")
}

/// Python: bot/modules/chat.py:13612 `claims_instead(text, memory)` (never called on the trace corpus)
pub fn claims_instead(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.chat:claims_instead")
}

/// Python: bot/modules/chat.py:13716 `_only_the_apology(clean, match)` (never called on the trace corpus)
pub fn _only_the_apology(clean: &str, r#match: &Value) -> bool {
    crate::unported!("bot.modules.chat:_only_the_apology")
}

/// Python: bot/modules/chat.py:13750 `_chat_scatter(clean, memory)`
pub fn _chat_scatter(clean: &str, memory: &Memory) {
    crate::unported!("bot.modules.chat:_chat_scatter")
}

/// Python: bot/modules/chat.py:13999 `_the_deed_pool(deed)` (never called on the trace corpus)
pub fn _the_deed_pool(deed: &Value) -> Value {
    crate::unported!("bot.modules.chat:_the_deed_pool")
}

/// Python: bot/modules/chat.py:14008 `_the_words_pool(deed)` (never called on the trace corpus)
pub fn _the_words_pool(deed: &Value) -> Value {
    crate::unported!("bot.modules.chat:_the_words_pool")
}

/// Python: bot/modules/chat.py:14092 `_an_entry_answers_the_deed(clean, memory)` (never called on the trace corpus)
pub fn _an_entry_answers_the_deed(clean: &str, memory: &Memory) -> Option<bool> {
    crate::unported!("bot.modules.chat:_an_entry_answers_the_deed")
}

/// Python: bot/modules/chat.py:14111 `_said_wrong(clean)`
pub fn _said_wrong(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_said_wrong")
}

/// Python: bot/modules/chat.py:14140 `_own_deed(clean, start)` (never called on the trace corpus)
pub fn _own_deed(clean: &str, start: i64) -> bool {
    crate::unported!("bot.modules.chat:_own_deed")
}

/// Python: bot/modules/chat.py:14149 `_said_unkindly(clean)`
pub fn _said_unkindly(clean: &str) -> bool {
    crate::unported!("bot.modules.chat:_said_unkindly")
}

/// Python: bot/modules/chat.py:14163 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.chat:respond")
}

/// Python: bot/modules/chat.py:14288 `_a_mood_pool_line(said)` (never called on the trace corpus)
pub fn _a_mood_pool_line(said: &str) -> bool {
    crate::unported!("bot.modules.chat:_a_mood_pool_line")
}

/// Python: bot/modules/chat.py:14306 `_a_part_of_the_answer_they_have(clean, memory)`
pub fn _a_part_of_the_answer_they_have(clean: &str, memory: &Memory) -> Option<String> {
    crate::unported!("bot.modules.chat:_a_part_of_the_answer_they_have")
}

/// Python: bot/modules/chat.py:14377 `_the_newest_line_started_the_topic(memory)`
pub fn _the_newest_line_started_the_topic(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_the_newest_line_started_the_topic")
}

/// Python: bot/modules/chat.py:14480 `_the_verdict_in(clean, memory)`
pub fn _the_verdict_in(clean: &str, memory: &Memory) {
    crate::unported!("bot.modules.chat:_the_verdict_in")
}

/// Python: bot/modules/chat.py:14571 `_a_topic_still_in_this_exchange(topic, memory)`
pub fn _a_topic_still_in_this_exchange(topic: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_a_topic_still_in_this_exchange")
}

/// Python: bot/modules/chat.py:14598 `_follow_up_reading(text, memory)`
pub fn _follow_up_reading(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.chat:_follow_up_reading")
}

/// Python: bot/modules/chat.py:14846 `_reads_as_distress(line)`
pub fn _reads_as_distress(line: &str) -> bool {
    crate::unported!("bot.modules.chat:_reads_as_distress")
}

/// Python: bot/modules/chat.py:14857 `_nothing_was_heard(memory, topic)`
pub fn _nothing_was_heard(memory: &Memory, topic: &str) -> bool {
    crate::unported!("bot.modules.chat:_nothing_was_heard")
}

/// Python: bot/modules/chat.py:14879 `_asks_what_they_said(said, memory)`
pub fn _asks_what_they_said(said: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_asks_what_they_said")
}

/// Python: bot/modules/chat.py:14904 `_the_entrys_own_name(item)`
pub fn _the_entrys_own_name(item: &'static Entry) -> Vec<String> {
    crate::unported!("bot.modules.chat:_the_entrys_own_name")
}

/// Python: bot/modules/chat.py:14916 `_about_the_topic(reading, memory, topic)`
pub fn _about_the_topic(reading: &str, memory: &Memory, topic: &str) -> bool {
    crate::unported!("bot.modules.chat:_about_the_topic")
}

/// Python: bot/modules/chat.py:14965 `_says_nothing_about_the_ask(said)` (never called on the trace corpus)
pub fn _says_nothing_about_the_ask(said: &str) -> bool {
    crate::unported!("bot.modules.chat:_says_nothing_about_the_ask")
}

/// Python: bot/modules/chat.py:15036 `_names_its_own_thing(text, topic='')`
pub fn _names_its_own_thing(text: &str, topic: &str) -> bool {
    crate::unported!("bot.modules.chat:_names_its_own_thing")
}

/// Python: bot/modules/chat.py:15063 `_bare_ask(text)`
pub fn _bare_ask(text: &str) -> bool {
    crate::unported!("bot.modules.chat:_bare_ask")
}

/// Python: bot/modules/chat.py:15081 `_words_only(text)`
pub fn _words_only(text: &str) -> String {
    crate::unported!("bot.modules.chat:_words_only")
}

/// Python: bot/modules/chat.py:15101 `_the_same_answer_again(said, memory, line='')`
pub fn _the_same_answer_again(said: &str, memory: &Memory, line: &str) -> bool {
    crate::unported!("bot.modules.chat:_the_same_answer_again")
}

/// Python: bot/modules/chat.py:15165 `_said_before(said, memory)`
pub fn _said_before(said: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_said_before")
}

/// Python: bot/modules/chat.py:15202 `_says_the_kind(ask, said)`
pub fn _says_the_kind(ask: &str, said: &str) -> bool {
    crate::unported!("bot.modules.chat:_says_the_kind")
}

/// Python: bot/modules/chat.py:15279 `_work_grumble(clean, tokens, memory)`
pub fn _work_grumble(clean: &str, tokens: &[String], memory: &mut Memory) {
    crate::unported!("bot.modules.chat:_work_grumble")
}

/// Python: bot/modules/chat.py:15360 `_laughing_at(clean, memory=None)`
pub fn _laughing_at(clean: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.chat:_laughing_at")
}

/// Python: bot/modules/chat.py:15394 `_the_condolence_is_the_last_thing(memory)` (never called on the trace corpus)
pub fn _the_condolence_is_the_last_thing(memory: &Memory) -> bool {
    crate::unported!("bot.modules.chat:_the_condolence_is_the_last_thing")
}

/// Python: bot/modules/chat.py:15442 `_laughing_still(clean, memory)`
pub fn _laughing_still(clean: &str, memory: &Memory) {
    crate::unported!("bot.modules.chat:_laughing_still")
}

/// Python: bot/modules/chat.py:15498 `_somebody_thinks_i_should(clean)`
pub fn _somebody_thinks_i_should(clean: &str) {
    crate::unported!("bot.modules.chat:_somebody_thinks_i_should")
}

/// Python: bot/modules/chat.py:15520 `_thinks_pool(who)` (never called on the trace corpus)
pub fn _thinks_pool(who: &str) -> Vec<String> {
    crate::unported!("bot.modules.chat:_thinks_pool")
}

/// Python: bot/modules/chat.py:15593 `_somebody_recommended_a_thing(clean)`
pub fn _somebody_recommended_a_thing(clean: &str) {
    crate::unported!("bot.modules.chat:_somebody_recommended_a_thing")
}

/// Python: bot/modules/chat.py:15620 `_respond(text, memory, as_said=True)`
pub fn _respond(text: &str, memory: &mut Memory, as_said: bool) -> Option<String> {
    crate::unported!("bot.modules.chat:_respond")
}

