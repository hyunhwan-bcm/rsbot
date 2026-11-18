//! Port of `bot/memory/store_replies.py`.
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

/// Python: bot/memory/store_replies.py:374 `_a_count_ask(low)`
pub fn _a_count_ask(low: &str) {
    crate::unported!("bot.memory.store_replies:_a_count_ask")
}

/// Python: bot/memory/store_replies.py:595 `_bare(folded)`
pub fn _bare(folded: &str) -> String {
    crate::unported!("bot.memory.store_replies:_bare")
}

/// Python: bot/memory/store_replies.py:617 `_the_limits_on(entry)` (never called on the trace corpus)
pub fn _the_limits_on(entry: &'static Entry) -> Value {
    crate::unported!("bot.memory.store_replies:_the_limits_on")
}

/// Python: bot/memory/store_replies.py:623 `_the_limit_they_share(text, memory)`
pub fn _the_limit_they_share(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_the_limit_they_share")
}

/// Python: bot/memory/store_replies.py:671 `_is_or_are(words)` (never called on the trace corpus)
pub fn _is_or_are(words: &[String]) -> String {
    crate::unported!("bot.memory.store_replies:_is_or_are")
}

/// Python: bot/memory/store_replies.py:678 `_the_things(words)` (never called on the trace corpus)
pub fn _the_things(words: &[String]) -> Value {
    crate::unported!("bot.memory.store_replies:_the_things")
}

/// Python: bot/memory/store_replies.py:687 `_said_of_a_shared_limit(limits, who, memory)` (never called on the trace corpus)
pub fn _said_of_a_shared_limit(limits: &Value, who: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_said_of_a_shared_limit")
}

/// Python: bot/memory/store_replies.py:728 `_their_own(who)` (never called on the trace corpus)
pub fn _their_own(who: &str) -> String {
    crate::unported!("bot.memory.store_replies:_their_own")
}

/// Python: bot/memory/store_replies.py:739 `_the_back_this_turn(text, memory)`
pub fn _the_back_this_turn(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_the_back_this_turn")
}

/// Python: bot/memory/store_replies.py:764 `_still_holding(memory, animals)` (never called on the trace corpus)
pub fn _still_holding(memory: &Memory, animals: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_still_holding")
}

/// Python: bot/memory/store_replies.py:798 `_a_size(text)`
pub fn _a_size(text: &str) {
    crate::unported!("bot.memory.store_replies:_a_size")
}

/// Python: bot/memory/store_replies.py:805 `_only_names_the_size(text)`
pub fn _only_names_the_size(text: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_only_names_the_size")
}

/// Python: bot/memory/store_replies.py:814 `_the_clause_that_works_it_out(answer, size)` (never called on the trace corpus)
pub fn _the_clause_that_works_it_out(answer: &str, size: i64) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_the_clause_that_works_it_out")
}

/// Python: bot/memory/store_replies.py:852 `_the_people_that_size_feeds(text, memory)`
pub fn _the_people_that_size_feeds(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_the_people_that_size_feeds")
}

/// Python: bot/memory/store_replies.py:897 `_the_amount_they_made(text, memory)`
pub fn _the_amount_they_made(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_the_amount_they_made")
}

/// Python: bot/memory/store_replies.py:938 `_the_portion_reply(size, portion, count)` (never called on the trace corpus)
pub fn _the_portion_reply(size: i64, portion: &Value, count: i64) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_the_portion_reply")
}

/// Python: bot/memory/store_replies.py:962 `_the_portion_for_the_food(held, answers)` (never called on the trace corpus)
pub fn _the_portion_for_the_food(held: &Value, answers: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_the_portion_for_the_food")
}

/// Python: bot/memory/store_replies.py:994 `_the_size_on_the_screen(turns)` (never called on the trace corpus)
pub fn _the_size_on_the_screen(turns: &[Turn]) -> Value {
    crate::unported!("bot.memory.store_replies:_the_size_on_the_screen")
}

/// Python: bot/memory/store_replies.py:1012 `_size_in_grams(size)` (never called on the trace corpus)
pub fn _size_in_grams(size: i64) -> Value {
    crate::unported!("bot.memory.store_replies:_size_in_grams")
}

/// Python: bot/memory/store_replies.py:1025 `_the_figure_for_the_size_they_named(text, memory)`
pub fn _the_figure_for_the_size_they_named(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_the_figure_for_the_size_they_named")
}

/// Python: bot/memory/store_replies.py:1059 `_denial_this_turn(text, memory)`
pub fn _denial_this_turn(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_denial_this_turn")
}

/// Python: bot/memory/store_replies.py:1134 `_retraction_this_turn(text, memory)`
pub fn _retraction_this_turn(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_retraction_this_turn")
}

/// Python: bot/memory/store_replies.py:1166 `_told_readers()`
pub fn _told_readers() -> Value {
    crate::unported!("bot.memory.store_replies:_told_readers")
}

/// Python: bot/memory/store_replies.py:1202 `_the_line_before_is_their_own_animal(memory, back=3)` (never called on the trace corpus)
pub fn _the_line_before_is_their_own_animal(memory: &Memory, back: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_the_line_before_is_their_own_animal")
}

/// Python: bot/memory/store_replies.py:1224 `_the_names_again(folded, low, memory)`
pub fn _the_names_again(folded: &str, low: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.memory.store_replies:_the_names_again")
}

/// Python: bot/memory/store_replies.py:1255 `_the_one_animal_with_a_name(memory)` (never called on the trace corpus)
pub fn _the_one_animal_with_a_name(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_the_one_animal_with_a_name")
}

/// Python: bot/memory/store_replies.py:1269 `_told_pet_reply(memory, animal)` (never called on the trace corpus)
pub fn _told_pet_reply(memory: &mut Memory, animal: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_told_pet_reply")
}

/// Python: bot/memory/store_replies.py:1289 `_says_more_than_the_ask(low)` (never called on the trace corpus)
pub fn _says_more_than_the_ask(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_says_more_than_the_ask")
}

/// Python: bot/memory/store_replies.py:1316 `_how_far_along(memory, low)`
pub fn _how_far_along(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_how_far_along")
}

/// Python: bot/memory/store_replies.py:1352 `_a_length_i_have(memory, low)`
pub fn _a_length_i_have(memory: &Memory, low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_length_i_have")
}

/// Python: bot/memory/store_replies.py:1373 `_a_limit_kept_for_them(low, memory)` (never called on the trace corpus)
pub fn _a_limit_kept_for_them(low: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_a_limit_kept_for_them")
}

/// Python: bot/memory/store_replies.py:1397 `_the_pick_asked_again(text, memory)`
pub fn _the_pick_asked_again(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_the_pick_asked_again")
}

/// Python: bot/memory/store_replies.py:1442 `_what_the_pet_is_called(text, memory)`
pub fn _what_the_pet_is_called(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_what_the_pet_is_called")
}

/// Python: bot/memory/store_replies.py:1474 `_the_name_they_picked(text, memory)`
pub fn _the_name_they_picked(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_the_name_they_picked")
}

/// Python: bot/memory/store_replies.py:1506 `_offered_here(memory)`
pub fn _offered_here(memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_offered_here")
}

/// Python: bot/memory/store_replies.py:1518 `_name_they_answered(text, memory)`
pub fn _name_they_answered(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_name_they_answered")
}

/// Python: bot/memory/store_replies.py:1531 `_the_animal_of_the_pick(memory)` (never called on the trace corpus)
pub fn _the_animal_of_the_pick(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_the_animal_of_the_pick")
}

/// Python: bot/memory/store_replies.py:1552 `_correction_this_turn(text, memory)`
pub fn _correction_this_turn(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_correction_this_turn")
}

/// Python: bot/memory/store_replies.py:1561 `_the_second_pet_said(fixed, memory)` (never called on the trace corpus)
pub fn _the_second_pet_said(fixed: &Value, memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_the_second_pet_said")
}

/// Python: bot/memory/store_replies.py:1584 `_correction_said(fixed, memory=None)` (never called on the trace corpus)
pub fn _correction_said(fixed: &Value, memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_correction_said")
}

/// Python: bot/memory/store_replies.py:1665 `_job_note(value, known)` (never called on the trace corpus)
pub fn _job_note(value: &Value, known: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_job_note")
}

/// Python: bot/memory/store_replies.py:1689 `_the_limit_the_last_line_gave(memory)` (never called on the trace corpus)
pub fn _the_limit_the_last_line_gave(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_the_limit_the_last_line_gave")
}

/// Python: bot/memory/store_replies.py:1712 `_not_the_same_reply(memory, low)`
pub fn _not_the_same_reply(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_not_the_same_reply")
}

/// Python: bot/memory/store_replies.py:1793 `_with_an_in_law(who, low)` (never called on the trace corpus)
pub fn _with_an_in_law(who: &str, low: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_with_an_in_law")
}

/// Python: bot/memory/store_replies.py:1815 `_kids_ask(low)`
pub fn _kids_ask(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_kids_ask")
}

/// Python: bot/memory/store_replies.py:1910 `_the_ask_after_a_statement(low, memory=None)`
pub fn _the_ask_after_a_statement(low: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_the_ask_after_a_statement")
}

/// Python: bot/memory/store_replies.py:1939 `_asks_about_someone(low, text, memory=None)`
pub fn _asks_about_someone(low: &str, text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_asks_about_someone")
}

/// Python: bot/memory/store_replies.py:1965 `_asks_the_line(low, text)`
pub fn _asks_the_line(low: &str, text: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_asks_the_line")
}

/// Python: bot/memory/store_replies.py:2011 `_about_the_bot_here(memory)`
pub fn _about_the_bot_here(memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_about_the_bot_here")
}

/// Python: bot/memory/store_replies.py:2033 `_about_this_chat(text, memory=None)`
pub fn _about_this_chat(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_about_this_chat")
}

/// Python: bot/memory/store_replies.py:2225 `_ask_about_the_job(ask)` (never called on the trace corpus)
pub fn _ask_about_the_job(ask: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_ask_about_the_job")
}

/// Python: bot/memory/store_replies.py:2234 `_job_just_said(memory, ask='')` (never called on the trace corpus)
pub fn _job_just_said(memory: &mut Memory, ask: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_job_just_said")
}

/// Python: bot/memory/store_replies.py:2271 `_doing_again(text, memory)` (never called on the trace corpus)
pub fn _doing_again(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_doing_again")
}

/// Python: bot/memory/store_replies.py:2308 `_a_work_detail(line)` (never called on the trace corpus)
pub fn _a_work_detail(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_work_detail")
}

/// Python: bot/memory/store_replies.py:2330 `_job_words_seen(job, line)` (never called on the trace corpus)
pub fn _job_words_seen(job: &Value, line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_job_words_seen")
}

/// Python: bot/memory/store_replies.py:2351 `_the_place_said(place, line)` (never called on the trace corpus)
pub fn _the_place_said(place: &str, line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_the_place_said")
}

/// Python: bot/memory/store_replies.py:2371 `_only_the_job(line, job, place='')` (never called on the trace corpus)
pub fn _only_the_job(line: &str, job: &Value, place: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_only_the_job")
}

/// Python: bot/memory/store_replies.py:2415 `_summary_ask(low)`
pub fn _summary_ask(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_summary_ask")
}

/// Python: bot/memory/store_replies.py:2448 `_an_attribute_they_gave(low)`
pub fn _an_attribute_they_gave(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_an_attribute_they_gave")
}

/// Python: bot/memory/store_replies.py:2470 `_an_open_attribute_ask(low, memory)`
pub fn _an_open_attribute_ask(low: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_an_open_attribute_ask")
}

/// Python: bot/memory/store_replies.py:2500 `_which_thing_was_it(low, memory)`
pub fn _which_thing_was_it(low: &str, memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_which_thing_was_it")
}

/// Python: bot/memory/store_replies.py:2534 `_without_the_when(line)` (never called on the trace corpus)
pub fn _without_the_when(line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_without_the_when")
}

/// Python: bot/memory/store_replies.py:2550 `_a_clock_ask(line)`
pub fn _a_clock_ask(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_clock_ask")
}

/// Python: bot/memory/store_replies.py:2555 `_said_word(word, memory)` (never called on the trace corpus)
pub fn _said_word(word: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_said_word")
}

/// Python: bot/memory/store_replies.py:2577 `_recall_in_progress(memory)` (never called on the trace corpus)
pub fn _recall_in_progress(memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_recall_in_progress")
}

/// Python: bot/memory/store_replies.py:2591 `_elided_ask(text, memory, low='')`
pub fn _elided_ask(text: &str, memory: &Memory, low: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_elided_ask")
}

/// Python: bot/memory/store_replies.py:2638 `_named_slots(line)` (never called on the trace corpus)
pub fn _named_slots(line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_named_slots")
}

/// Python: bot/memory/store_replies.py:2642 `_stated_slot(memory, turns, back=1)` (never called on the trace corpus)
pub fn _stated_slot(memory: &Memory, turns: &[Turn], back: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_stated_slot")
}

/// Python: bot/memory/store_replies.py:2677 `_remember_asked(memory, slot)`
pub fn _remember_asked(memory: &mut Memory, slot: &str) {
    crate::unported!("bot.memory.store_replies:_remember_asked")
}

/// Python: bot/memory/store_replies.py:2683 `_the_case_they_just_gave(text, memory)`
pub fn _the_case_they_just_gave(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_the_case_they_just_gave")
}

/// Python: bot/memory/store_replies.py:2731 `_said_another_way(reply)` (never called on the trace corpus)
pub fn _said_another_way(reply: &str) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_said_another_way")
}

/// Python: bot/memory/store_replies.py:2767 `_slot_words(slot, memory=None)` (never called on the trace corpus)
pub fn _slot_words(slot: &Value, memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_slot_words")
}

/// Python: bot/memory/store_replies.py:2784 `_slot_of(low)`
pub fn _slot_of(low: &str) -> String {
    crate::unported!("bot.memory.store_replies:_slot_of")
}

/// Python: bot/memory/store_replies.py:2815 `_slot_named_in(low)` (never called on the trace corpus)
pub fn _slot_named_in(low: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_slot_named_in")
}

/// Python: bot/memory/store_replies.py:2822 `_slot_held(slot, memory)` (never called on the trace corpus)
pub fn _slot_held(slot: &Value, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_slot_held")
}

/// Python: bot/memory/store_replies.py:2830 `_bare_ask_naming_a_fact(low, memory)`
pub fn _bare_ask_naming_a_fact(low: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.memory.store_replies:_bare_ask_naming_a_fact")
}

/// Python: bot/memory/store_replies.py:2914 `_is_a_question(previous)` (never called on the trace corpus)
pub fn _is_a_question(previous: &Value) -> Option<bool> {
    crate::unported!("bot.memory.store_replies:_is_a_question")
}

/// Python: bot/memory/store_replies.py:2937 `_again(who)` (never called on the trace corpus)
pub fn _again(who: &str) -> String {
    crate::unported!("bot.memory.store_replies:_again")
}

/// Python: bot/memory/store_replies.py:2946 `_the_thing_before_the_feelings(memory)` (never called on the trace corpus)
pub fn _the_thing_before_the_feelings(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_the_thing_before_the_feelings")
}

/// Python: bot/memory/store_replies.py:2970 `_a_feeling_about_what_they_said(memory, low)`
pub fn _a_feeling_about_what_they_said(memory: &Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_a_feeling_about_what_they_said")
}

/// Python: bot/memory/store_replies.py:3082 `_said_back_already(reply, said)` (never called on the trace corpus)
pub fn _said_back_already(reply: &str, said: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_said_back_already")
}

/// Python: bot/memory/store_replies.py:3096 `_the_bot_name(memory, low)`
pub fn _the_bot_name(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_the_bot_name")
}

/// Python: bot/memory/store_replies.py:3108 `_the_store_reply(text, memory)`
pub fn _the_store_reply(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_the_store_reply")
}

/// Python: bot/memory/store_replies.py:3380 `_the_answer_already_gives(detail, memory)` (never called on the trace corpus)
pub fn _the_answer_already_gives(detail: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_the_answer_already_gives")
}

/// Python: bot/memory/store_replies.py:3445 `_turned(question)` (never called on the trace corpus)
pub fn _turned(question: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_turned")
}

/// Python: bot/memory/store_replies.py:3455 `_a_choice_from_what_was_said(line, memory)`
pub fn _a_choice_from_what_was_said(line: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_a_choice_from_what_was_said")
}

/// Python: bot/memory/store_replies.py:3475 `_detail_they_left_out(memory, low)`
pub fn _detail_they_left_out(memory: &Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_detail_they_left_out")
}

/// Python: bot/memory/store_replies.py:3600 `_a_held_part(memory, line)` (never called on the trace corpus)
pub fn _a_held_part(memory: &mut Memory, line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_held_part")
}

/// Python: bot/memory/store_replies.py:3608 `_a_part_of_the_job_never_said(memory, low)` (never called on the trace corpus)
pub fn _a_part_of_the_job_never_said(memory: &mut Memory, low: &str) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_a_part_of_the_job_never_said")
}

/// Python: bot/memory/store_replies.py:3624 `_holds_nothing(known)`
pub fn _holds_nothing(known: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_holds_nothing")
}

/// Python: bot/memory/store_replies.py:3627 `_a_thing_it_keeps(memory, mine)`
pub fn _a_thing_it_keeps(memory: &mut Memory, mine: &str) {
    crate::unported!("bot.memory.store_replies:_a_thing_it_keeps")
}

/// Python: bot/memory/store_replies.py:3643 `_the_part_it_never_said(memory, low)`
pub fn _the_part_it_never_said(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_the_part_it_never_said")
}

/// Python: bot/memory/store_replies.py:3675 `_a_kept_thing_behind(memory)` (never called on the trace corpus)
pub fn _a_kept_thing_behind(memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_a_kept_thing_behind")
}

/// Python: bot/memory/store_replies.py:3703 `_diet_how(diet)` (never called on the trace corpus)
pub fn _diet_how(diet: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_diet_how")
}

/// Python: bot/memory/store_replies.py:3713 `_the_fact_just_asked(memory, low)`
pub fn _the_fact_just_asked(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_the_fact_just_asked")
}

/// Python: bot/memory/store_replies.py:3740 `_yes_then(reply)` (never called on the trace corpus)
pub fn _yes_then(reply: &str) -> String {
    crate::unported!("bot.memory.store_replies:_yes_then")
}

/// Python: bot/memory/store_replies.py:3766 `_after_the_yes_no_frame(text)`
pub fn _after_the_yes_no_frame(text: &str) -> String {
    crate::unported!("bot.memory.store_replies:_after_the_yes_no_frame")
}

/// Python: bot/memory/store_replies.py:3784 `_statement(text)`
pub fn _statement(text: &str) {
    crate::unported!("bot.memory.store_replies:_statement")
}

/// Python: bot/memory/store_replies.py:3804 `_same(left, right)` (never called on the trace corpus)
pub fn _same(left: &Value, right: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_same")
}

/// Python: bot/memory/store_replies.py:3807 `_clause(key, value, known=None)` (never called on the trace corpus)
pub fn _clause(key: &str, value: &Value, known: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_clause")
}

/// Python: bot/memory/store_replies.py:3842 `_their_allergy(who, theirs)` (never called on the trace corpus)
pub fn _their_allergy(who: &str, theirs: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_their_allergy")
}

/// Python: bot/memory/store_replies.py:3849 `_said_matches(key, said, known)` (never called on the trace corpus)
pub fn _said_matches(key: &str, said: &str, known: &Value) -> Option<bool> {
    crate::unported!("bot.memory.store_replies:_said_matches")
}

/// Python: bot/memory/store_replies.py:3878 `_names(key, text)` (never called on the trace corpus)
pub fn _names(key: &str, text: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_names")
}

/// Python: bot/memory/store_replies.py:3902 `_restated(text, typed=None)`
pub fn _restated(text: &str, typed: &str) {
    crate::unported!("bot.memory.store_replies:_restated")
}

/// Python: bot/memory/store_replies.py:3930 `_behind_a_remembering_frame(text)`
pub fn _behind_a_remembering_frame(text: &str) {
    crate::unported!("bot.memory.store_replies:_behind_a_remembering_frame")
}

/// Python: bot/memory/store_replies.py:3964 `_remembers_fact(text, memory)`
pub fn _remembers_fact(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_remembers_fact")
}

/// Python: bot/memory/store_replies.py:4016 `_someone_said_back(said, memory)` (never called on the trace corpus)
pub fn _someone_said_back(said: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_someone_said_back")
}

/// Python: bot/memory/store_replies.py:4100 `_the_food_named(rest)` (never called on the trace corpus)
pub fn _the_food_named(rest: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_food_named")
}

/// Python: bot/memory/store_replies.py:4113 `_the_rule_that_rules_it_out(known, category)` (never called on the trace corpus)
pub fn _the_rule_that_rules_it_out(known: &Value, category: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_the_rule_that_rules_it_out")
}

/// Python: bot/memory/store_replies.py:4135 `_a_rule_on_eating(text, memory)`
pub fn _a_rule_on_eating(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_a_rule_on_eating")
}

/// Python: bot/memory/store_replies.py:4159 `_confirmation(text, memory)`
pub fn _confirmation(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_confirmation")
}

/// Python: bot/memory/store_replies.py:4222 `_old_values(memory, told)` (never called on the trace corpus)
pub fn _old_values(memory: &Memory, told: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_old_values")
}

/// Python: bot/memory/store_replies.py:4251 `_a_figure(word)` (never called on the trace corpus)
pub fn _a_figure(word: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_figure")
}

/// Python: bot/memory/store_replies.py:4256 `_swapped_lines(memory, fixed)` (never called on the trace corpus)
pub fn _swapped_lines(memory: &Memory, fixed: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_swapped_lines")
}

/// Python: bot/memory/store_replies.py:4308 `_the_time_that_moved(told, fixed, value)` (never called on the trace corpus)
pub fn _the_time_that_moved(told: &Value, fixed: &Value, value: &Value) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_the_time_that_moved")
}

/// Python: bot/memory/store_replies.py:4341 `_a_rate_changed(text, memory)` (never called on the trace corpus)
pub fn _a_rate_changed(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_a_rate_changed")
}

/// Python: bot/memory/store_replies.py:4348 `_answer_the_new_one(memory, fixed)` (never called on the trace corpus)
pub fn _answer_the_new_one(memory: &mut Memory, fixed: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_answer_the_new_one")
}

/// Python: bot/memory/store_replies.py:4386 `_name_fixed_this_turn(text, memory)`
pub fn _name_fixed_this_turn(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_name_fixed_this_turn")
}

/// Python: bot/memory/store_replies.py:4410 `_a_number_word(number)` (never called on the trace corpus)
pub fn _a_number_word(number: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_a_number_word")
}

/// Python: bot/memory/store_replies.py:4429 `_fact_moved_this_turn(text, memory)`
pub fn _fact_moved_this_turn(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_fact_moved_this_turn")
}

/// Python: bot/memory/store_replies.py:4453 `_fact_fixed_this_turn(text, memory)`
pub fn _fact_fixed_this_turn(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_fact_fixed_this_turn")
}

/// Python: bot/memory/store_replies.py:4479 `_respond(text, memory)`
pub fn _respond(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_respond")
}

/// Python: bot/memory/store_replies.py:5073 `_a_place_new_to_the_chat(memory, low)` (never called on the trace corpus)
pub fn _a_place_new_to_the_chat(memory: &Memory, low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_place_new_to_the_chat")
}

/// Python: bot/memory/store_replies.py:5095 `_about_what_they_said(text, memory)`
pub fn _about_what_they_said(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_about_what_they_said")
}

/// Python: bot/memory/store_replies.py:5154 `_the_job_was_just_said(memory)` (never called on the trace corpus)
pub fn _the_job_was_just_said(memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_the_job_was_just_said")
}

/// Python: bot/memory/store_replies.py:5185 `_nothing_but_the_job(said, job, memory)` (never called on the trace corpus)
pub fn _nothing_but_the_job(said: &str, job: &Value, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_nothing_but_the_job")
}

/// Python: bot/memory/store_replies.py:5204 `_the_allergy_is_what_is_asked(text, memory)` (never called on the trace corpus)
pub fn _the_allergy_is_what_is_asked(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_the_allergy_is_what_is_asked")
}

/// Python: bot/memory/store_replies.py:5229 `_general_recall(text, memory)`
pub fn _general_recall(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_general_recall")
}

/// Python: bot/memory/store_replies.py:5312 `_pet_names(family)` (never called on the trace corpus)
pub fn _pet_names(family: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_pet_names")
}

/// Python: bot/memory/store_replies.py:5326 `_points_at_something_else(low)` (never called on the trace corpus)
pub fn _points_at_something_else(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_points_at_something_else")
}

/// Python: bot/memory/store_replies.py:5340 `_the_pet_in_play(low, memory)` (never called on the trace corpus)
pub fn _the_pet_in_play(low: &str, memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_the_pet_in_play")
}

/// Python: bot/memory/store_replies.py:5355 `_a_pet_of_theirs_is_named(low, family, memory)` (never called on the trace corpus)
pub fn _a_pet_of_theirs_is_named(low: &str, family: &Value, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_a_pet_of_theirs_is_named")
}

/// Python: bot/memory/store_replies.py:5365 `_a_name_for_the_pet(text, memory, low)`
pub fn _a_name_for_the_pet(text: &str, memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_a_name_for_the_pet")
}

/// Python: bot/memory/store_replies.py:5387 `_count_of_a_kind(pets)` (never called on the trace corpus)
pub fn _count_of_a_kind(pets: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_count_of_a_kind")
}

/// Python: bot/memory/store_replies.py:5400 `_return_reply(text, memory)`
pub fn _return_reply(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_return_reply")
}

/// Python: bot/memory/store_replies.py:5416 `_same_case(thing, text)` (never called on the trace corpus)
pub fn _same_case(thing: &str, text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_same_case")
}

/// Python: bot/memory/store_replies.py:5423 `_name_reply(memory, low='')` (never called on the trace corpus)
pub fn _name_reply(memory: &mut Memory, low: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_name_reply")
}

/// Python: bot/memory/store_replies.py:5439 `_the_day_of(plan)` (never called on the trace corpus)
pub fn _the_day_of(plan: &str) -> String {
    crate::unported!("bot.memory.store_replies:_the_day_of")
}

/// Python: bot/memory/store_replies.py:5455 `_at_the_clock(plan)` (never called on the trace corpus)
pub fn _at_the_clock(plan: &str) -> String {
    crate::unported!("bot.memory.store_replies:_at_the_clock")
}

/// Python: bot/memory/store_replies.py:5478 `_the_day_before(plan)` (never called on the trace corpus)
pub fn _the_day_before(plan: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_day_before")
}

/// Python: bot/memory/store_replies.py:5494 `_the_prep_day(memory, low)`
pub fn _the_prep_day(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_the_prep_day")
}

/// Python: bot/memory/store_replies.py:5526 `_a_plan_answers(memory, low)` (never called on the trace corpus)
pub fn _a_plan_answers(memory: &mut Memory, low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_plan_answers")
}

/// Python: bot/memory/store_replies.py:5545 `_the_date_behind_that(memory, low, plans)`
pub fn _the_date_behind_that(memory: &Memory, low: &str, plans: &[String]) {
    crate::unported!("bot.memory.store_replies:_the_date_behind_that")
}

/// Python: bot/memory/store_replies.py:5568 `_the_whole_word_in(text, word)` (never called on the trace corpus)
pub fn _the_whole_word_in(text: &str, word: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_whole_word_in")
}

/// Python: bot/memory/store_replies.py:5582 `_third_person(word)` (never called on the trace corpus)
pub fn _third_person(word: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_third_person")
}

/// Python: bot/memory/store_replies.py:5596 `_the_line_that_said(memory, plan)` (never called on the trace corpus)
pub fn _the_line_that_said(memory: &Memory, plan: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_line_that_said")
}

/// Python: bot/memory/store_replies.py:5610 `_the_verb_from_the_line(plan, line)` (never called on the trace corpus)
pub fn _the_verb_from_the_line(plan: &str, line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_verb_from_the_line")
}

/// Python: bot/memory/store_replies.py:5647 `_the_verb_they_asked(plan, low, line='')` (never called on the trace corpus)
pub fn _the_verb_they_asked(plan: &str, low: &str, line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_verb_they_asked")
}

/// Python: bot/memory/store_replies.py:5702 `_plans_reply(memory, low)`
pub fn _plans_reply(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_plans_reply")
}

/// Python: bot/memory/store_replies.py:5871 `_their_place(low)`
pub fn _their_place(low: &str) {
    crate::unported!("bot.memory.store_replies:_their_place")
}

/// Python: bot/memory/store_replies.py:5881 `_naming_words(text)` (never called on the trace corpus)
pub fn _naming_words(text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_naming_words")
}

/// Python: bot/memory/store_replies.py:5886 `_the_day_is_the_whole_date(plan)` (never called on the trace corpus)
pub fn _the_day_is_the_whole_date(plan: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_the_day_is_the_whole_date")
}

/// Python: bot/memory/store_replies.py:5898 `_the_plan_was_just_said_back(memory, plan)` (never called on the trace corpus)
pub fn _the_plan_was_just_said_back(memory: &Memory, plan: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_the_plan_was_just_said_back")
}

/// Python: bot/memory/store_replies.py:5912 `_the_answer_about_it_is_on_the_screen(memory, plan, text)` (never called on the trace corpus)
pub fn _the_answer_about_it_is_on_the_screen(memory: &Memory, plan: &str, text: &str) -> Option<bool> {
    crate::unported!("bot.memory.store_replies:_the_answer_about_it_is_on_the_screen")
}

/// Python: bot/memory/store_replies.py:5966 `_a_time_for_the_job_itself(low)` (never called on the trace corpus)
pub fn _a_time_for_the_job_itself(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_time_for_the_job_itself")
}

/// Python: bot/memory/store_replies.py:5977 `_thing_is_a_quality(low)` (never called on the trace corpus)
pub fn _thing_is_a_quality(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_thing_is_a_quality")
}

/// Python: bot/memory/store_replies.py:5992 `_names_something_of_its_own(low, start)` (never called on the trace corpus)
pub fn _names_something_of_its_own(low: &str, start: i64) -> bool {
    crate::unported!("bot.memory.store_replies:_names_something_of_its_own")
}

/// Python: bot/memory/store_replies.py:6001 `_when_thing(low, wide=False)`
pub fn _when_thing(low: &str, wide: bool) -> String {
    crate::unported!("bot.memory.store_replies:_when_thing")
}

/// Python: bot/memory/store_replies.py:6034 `_the_one_asked(matching, rest)`
pub fn _the_one_asked(matching: &[String], rest: &str) -> String {
    crate::unported!("bot.memory.store_replies:_the_one_asked")
}

/// Python: bot/memory/store_replies.py:6050 `_ask_verb(low)`
pub fn _ask_verb(low: &str) -> String {
    crate::unported!("bot.memory.store_replies:_ask_verb")
}

/// Python: bot/memory/store_replies.py:6055 `_asks_the_end(low)`
pub fn _asks_the_end(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_asks_the_end")
}

/// Python: bot/memory/store_replies.py:6060 `_asks_the_start(low)`
pub fn _asks_the_start(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_asks_the_start")
}

/// Python: bot/memory/store_replies.py:6065 `_says_the_end(plan)` (never called on the trace corpus)
pub fn _says_the_end(plan: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_says_the_end")
}

/// Python: bot/memory/store_replies.py:6070 `_answers_the_end_asked(plan, low)` (never called on the trace corpus)
pub fn _answers_the_end_asked(plan: &str, low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_answers_the_end_asked")
}

/// Python: bot/memory/store_replies.py:6082 `_names_the_thing(plan, thing, memory=None)` (never called on the trace corpus)
pub fn _names_the_thing(plan: &str, thing: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_names_the_thing")
}

/// Python: bot/memory/store_replies.py:6099 `_says_the_verb(plan, verb)` (never called on the trace corpus)
pub fn _says_the_verb(plan: &str, verb: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_says_the_verb")
}

/// Python: bot/memory/store_replies.py:6125 `_job_reply(memory, where=False)` (never called on the trace corpus)
pub fn _job_reply(memory: &mut Memory, r#where: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_job_reply")
}

/// Python: bot/memory/store_replies.py:6147 `_yours(value)` (never called on the trace corpus)
pub fn _yours(value: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_yours")
}

/// Python: bot/memory/store_replies.py:6161 `_a_circumstance(job)` (never called on the trace corpus)
pub fn _a_circumstance(job: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_a_circumstance")
}

/// Python: bot/memory/store_replies.py:6177 `_job_said(known)` (never called on the trace corpus)
pub fn _job_said(known: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_job_said")
}

/// Python: bot/memory/store_replies.py:6217 `_place_phrase(place)` (never called on the trace corpus)
pub fn _place_phrase(place: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_place_phrase")
}

/// Python: bot/memory/store_replies.py:6232 `_job_phrase(known)` (never called on the trace corpus)
pub fn _job_phrase(known: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_job_phrase")
}

/// Python: bot/memory/store_replies.py:6246 `_places_reply(memory, low)` (never called on the trace corpus)
pub fn _places_reply(memory: &mut Memory, low: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_places_reply")
}

/// Python: bot/memory/store_replies.py:6321 `_grown_up_reply(memory)` (never called on the trace corpus)
pub fn _grown_up_reply(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_grown_up_reply")
}

/// Python: bot/memory/store_replies.py:6334 `_from_reply(memory)` (never called on the trace corpus)
pub fn _from_reply(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_from_reply")
}

/// Python: bot/memory/store_replies.py:6351 `_is_are(who)` (never called on the trace corpus)
pub fn _is_are(who: &str) -> String {
    crate::unported!("bot.memory.store_replies:_is_are")
}

/// Python: bot/memory/store_replies.py:6354 `_eats(who)` (never called on the trace corpus)
pub fn _eats(who: &str) -> String {
    crate::unported!("bot.memory.store_replies:_eats")
}

/// Python: bot/memory/store_replies.py:6357 `_does_not(who)` (never called on the trace corpus)
pub fn _does_not(who: &str) -> String {
    crate::unported!("bot.memory.store_replies:_does_not")
}

/// Python: bot/memory/store_replies.py:6360 `_they(who)` (never called on the trace corpus)
pub fn _they(who: &str) -> String {
    crate::unported!("bot.memory.store_replies:_they")
}

/// Python: bot/memory/store_replies.py:6369 `_a_diet_predicate(who, diet)` (never called on the trace corpus)
pub fn _a_diet_predicate(who: &str, diet: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_a_diet_predicate")
}

/// Python: bot/memory/store_replies.py:6385 `_their_pronoun(who)` (never called on the trace corpus)
pub fn _their_pronoun(who: &str) -> String {
    crate::unported!("bot.memory.store_replies:_their_pronoun")
}

/// Python: bot/memory/store_replies.py:6400 `_no_diet(who)` (never called on the trace corpus)
pub fn _no_diet(who: &str) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_no_diet")
}

/// Python: bot/memory/store_replies.py:6414 `_a_condition_rules_out(condition)` (never called on the trace corpus)
pub fn _a_condition_rules_out(condition: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_a_condition_rules_out")
}

/// Python: bot/memory/store_replies.py:6423 `_both_food_facts(who, allergies, diet)` (never called on the trace corpus)
pub fn _both_food_facts(who: &str, allergies: &Value, diet: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_both_food_facts")
}

/// Python: bot/memory/store_replies.py:6441 `_allergy_is_their_food(who, allergies, only=True, low='')` (never called on the trace corpus)
pub fn _allergy_is_their_food(who: &str, allergies: &Value, only: &Value, low: &str) -> String {
    crate::unported!("bot.memory.store_replies:_allergy_is_their_food")
}

/// Python: bot/memory/store_replies.py:6467 `_what_the_food_has(low, ruled)` (never called on the trace corpus)
pub fn _what_the_food_has(low: &str, ruled: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_what_the_food_has")
}

/// Python: bot/memory/store_replies.py:6491 `_diet_is_their_food(who, diet, low='', plain=False)` (never called on the trace corpus)
pub fn _diet_is_their_food(who: &str, diet: &Value, low: &str, plain: &Value) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_diet_is_their_food")
}

/// Python: bot/memory/store_replies.py:6521 `_other_fact(memory, key)`
pub fn _other_fact(memory: &mut Memory, key: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_other_fact")
}

/// Python: bot/memory/store_replies.py:6536 `_last_answer(memory)`
pub fn _last_answer(memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_last_answer")
}

/// Python: bot/memory/store_replies.py:6547 `_one_half_of_the_dish(food, taste)` (never called on the trace corpus)
pub fn _one_half_of_the_dish(food: &Value, taste: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_one_half_of_the_dish")
}

/// Python: bot/memory/store_replies.py:6567 `_dislikes_here(text)`
pub fn _dislikes_here(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.store_replies:_dislikes_here")
}

/// Python: bot/memory/store_replies.py:6581 `_a_dislike_told(text, memory)`
pub fn _a_dislike_told(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_a_dislike_told")
}

/// Python: bot/memory/store_replies.py:6641 `_the_take_back(memory, key)`
pub fn _the_take_back(memory: &Memory, key: &str) -> Vec<String> {
    crate::unported!("bot.memory.store_replies:_the_take_back")
}

/// Python: bot/memory/store_replies.py:6662 `_a_take_back_said_back(line, gone)` (never called on the trace corpus)
pub fn _a_take_back_said_back(line: &str, gone: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_a_take_back_said_back")
}

/// Python: bot/memory/store_replies.py:6687 `_a_negation_said_back(gone, line='')` (never called on the trace corpus)
pub fn _a_negation_said_back(gone: &Value, line: &str) -> String {
    crate::unported!("bot.memory.store_replies:_a_negation_said_back")
}

/// Python: bot/memory/store_replies.py:6701 `_says_the_fact(line, key)` (never called on the trace corpus)
pub fn _says_the_fact(line: &str, key: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_says_the_fact")
}

/// Python: bot/memory/store_replies.py:6736 `_the_line_about(memory, key, own=None)`
pub fn _the_line_about(memory: &Memory, key: &str, own: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_the_line_about")
}

/// Python: bot/memory/store_replies.py:6771 `_said_of_someone_else(line, wanted)` (never called on the trace corpus)
pub fn _said_of_someone_else(line: &str, wanted: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_said_of_someone_else")
}

/// Python: bot/memory/store_replies.py:6786 `_deny(memory, key, denial, own=None)`
pub fn _deny(memory: &Memory, key: &str, denial: &str, own: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_deny")
}

/// Python: bot/memory/store_replies.py:6793 `_their_own_diet(line)` (never called on the trace corpus)
pub fn _their_own_diet(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_their_own_diet")
}

/// Python: bot/memory/store_replies.py:6817 `_diet_words(value)` (never called on the trace corpus)
pub fn _diet_words(value: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_diet_words")
}

/// Python: bot/memory/store_replies.py:6823 `_a_diet_put_back(shown)` (never called on the trace corpus)
pub fn _a_diet_put_back(shown: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_a_diet_put_back")
}

/// Python: bot/memory/store_replies.py:6847 `_diet_reply(memory, low='')`
pub fn _diet_reply(memory: &mut Memory, low: &str) -> String {
    crate::unported!("bot.memory.store_replies:_diet_reply")
}

/// Python: bot/memory/store_replies.py:6970 `_about_people(low)` (never called on the trace corpus)
pub fn _about_people(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_about_people")
}

/// Python: bot/memory/store_replies.py:6982 `_age_compare(memory, low)`
pub fn _age_compare(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_age_compare")
}

/// Python: bot/memory/store_replies.py:7091 `_the_day_a_plan_names(plan)` (never called on the trace corpus)
pub fn _the_day_a_plan_names(plan: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_day_a_plan_names")
}

/// Python: bot/memory/store_replies.py:7128 `_the_plans_in_date_order(memory)` (never called on the trace corpus)
pub fn _the_plans_in_date_order(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_the_plans_in_date_order")
}

/// Python: bot/memory/store_replies.py:7175 `_which_comes_first(memory, low)`
pub fn _which_comes_first(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_which_comes_first")
}

/// Python: bot/memory/store_replies.py:7201 `_the_pronoun_is_one_of_the_two(by_who, low)` (never called on the trace corpus)
pub fn _the_pronoun_is_one_of_the_two(by_who: &Value, low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_the_pronoun_is_one_of_the_two")
}

/// Python: bot/memory/store_replies.py:7217 `_pointed_at(aged, low)` (never called on the trace corpus)
pub fn _pointed_at(aged: &Value, low: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_pointed_at")
}

/// Python: bot/memory/store_replies.py:7249 `_a_food(value)` (never called on the trace corpus)
pub fn _a_food(value: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_a_food")
}

/// Python: bot/memory/store_replies.py:7262 `_diet_food(diet)` (never called on the trace corpus)
pub fn _diet_food(diet: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_diet_food")
}

/// Python: bot/memory/store_replies.py:7274 `_said_a_limit(memory)` (never called on the trace corpus)
pub fn _said_a_limit(memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_said_a_limit")
}

/// Python: bot/memory/store_replies.py:7284 `_said_list(items)` (never called on the trace corpus)
pub fn _said_list(items: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_said_list")
}

/// Python: bot/memory/store_replies.py:7299 `_limit_reply(memory, low='')` (never called on the trace corpus)
pub fn _limit_reply(memory: &mut Memory, low: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_limit_reply")
}

/// Python: bot/memory/store_replies.py:7386 `_the_answer_the_ask_gets(text, memory)`
pub fn _the_answer_the_ask_gets(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_the_answer_the_ask_gets")
}

/// Python: bot/memory/store_replies.py:7407 `_the_ask_beats_the_receipt(text, memory)`
pub fn _the_ask_beats_the_receipt(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_the_ask_beats_the_receipt")
}

/// Python: bot/memory/store_replies.py:7421 `_a_diet_told(text, memory, low)`
pub fn _a_diet_told(text: &str, memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_a_diet_told")
}

/// Python: bot/memory/store_replies.py:7451 `_object_pronoun(who, known=None)` (never called on the trace corpus)
pub fn _object_pronoun(who: &str, known: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_object_pronoun")
}

/// Python: bot/memory/store_replies.py:7456 `_who_the_pronoun_means(low, memory, pronoun)` (never called on the trace corpus)
pub fn _who_the_pronoun_means(low: &str, memory: &mut Memory, pronoun: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_who_the_pronoun_means")
}

/// Python: bot/memory/store_replies.py:7498 `_a_relations_name_reply(memory, low, text='')`
pub fn _a_relations_name_reply(memory: &mut Memory, low: &str, text: &str) {
    crate::unported!("bot.memory.store_replies:_a_relations_name_reply")
}

/// Python: bot/memory/store_replies.py:7613 `_the_group(who)` (never called on the trace corpus)
pub fn _the_group(who: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_group")
}

/// Python: bot/memory/store_replies.py:7621 `_in_group(who, group)` (never called on the trace corpus)
pub fn _in_group(who: &str, group: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_in_group")
}

/// Python: bot/memory/store_replies.py:7627 `_the_people(entries)` (never called on the trace corpus)
pub fn _the_people(entries: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_the_people")
}

/// Python: bot/memory/store_replies.py:7648 `_which_one_of_them(memory, who)` (never called on the trace corpus)
pub fn _which_one_of_them(memory: &mut Memory, who: &str) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_which_one_of_them")
}

/// Python: bot/memory/store_replies.py:7670 `_what_the_group_is_called(group, memory)` (never called on the trace corpus)
pub fn _what_the_group_is_called(group: &Value, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_what_the_group_is_called")
}

/// Python: bot/memory/store_replies.py:7711 `_everyone_they_mean(memory)` (never called on the trace corpus)
pub fn _everyone_they_mean(memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_everyone_they_mean")
}

/// Python: bot/memory/store_replies.py:7723 `_who_is_the_name_reply(memory, low)`
pub fn _who_is_the_name_reply(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_who_is_the_name_reply")
}

/// Python: bot/memory/store_replies.py:7814 `_kids_reply(memory, low)`
pub fn _kids_reply(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_kids_reply")
}

/// Python: bot/memory/store_replies.py:7859 `_and_list(words)` (never called on the trace corpus)
pub fn _and_list(words: &[String]) -> String {
    crate::unported!("bot.memory.store_replies:_and_list")
}

/// Python: bot/memory/store_replies.py:7866 `_lines_a_person_moved(memory)` (never called on the trace corpus)
pub fn _lines_a_person_moved(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_lines_a_person_moved")
}

/// Python: bot/memory/store_replies.py:7897 `_only_a_pointer(flat)` (never called on the trace corpus)
pub fn _only_a_pointer(flat: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_only_a_pointer")
}

/// Python: bot/memory/store_replies.py:7916 `_a_known_person(memory, who)` (never called on the trace corpus)
pub fn _a_known_person(memory: &mut Memory, who: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_a_known_person")
}

/// Python: bot/memory/store_replies.py:7990 `_only_the_name(line, named, who)` (never called on the trace corpus)
pub fn _only_the_name(line: &str, named: &Value, who: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_only_the_name")
}

/// Python: bot/memory/store_replies.py:8009 `_a_name_in_a_pronoun_line(line)` (never called on the trace corpus)
pub fn _a_name_in_a_pronoun_line(line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_a_name_in_a_pronoun_line")
}

/// Python: bot/memory/store_replies.py:8027 `_named_back(who, line)` (never called on the trace corpus)
pub fn _named_back(who: &str, line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_named_back")
}

/// Python: bot/memory/store_replies.py:8056 `_a_relation_in(line)` (never called on the trace corpus)
pub fn _a_relation_in(line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_a_relation_in")
}

/// Python: bot/memory/store_replies.py:8076 `_person_clauses(who, entry)` (never called on the trace corpus)
pub fn _person_clauses(who: &str, entry: &'static Entry) -> Value {
    crate::unported!("bot.memory.store_replies:_person_clauses")
}

/// Python: bot/memory/store_replies.py:8103 `_date_written_as_a_date(value)` (never called on the trace corpus)
pub fn _date_written_as_a_date(value: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_date_written_as_a_date")
}

/// Python: bot/memory/store_replies.py:8115 `_pronoun_person(text, memory)` (never called on the trace corpus)
pub fn _pronoun_person(text: &str, memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_pronoun_person")
}

/// Python: bot/memory/store_replies.py:8134 `_a_line_about(memory, who)` (never called on the trace corpus)
pub fn _a_line_about(memory: &Memory, who: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_a_line_about")
}

/// Python: bot/memory/store_replies.py:8173 `_a_that_fits(name)` (never called on the trace corpus)
pub fn _a_that_fits(name: &str) -> String {
    crate::unported!("bot.memory.store_replies:_a_that_fits")
}

/// Python: bot/memory/store_replies.py:8184 `_a_thing_i_own_reply(memory, low)`
pub fn _a_thing_i_own_reply(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_a_thing_i_own_reply")
}

/// Python: bot/memory/store_replies.py:8228 `_about_a_person_reply(memory, low, text='')`
pub fn _about_a_person_reply(memory: &mut Memory, low: &str, text: &str) {
    crate::unported!("bot.memory.store_replies:_about_a_person_reply")
}

/// Python: bot/memory/store_replies.py:8347 `_about_the_children_reply(memory, low)` (never called on the trace corpus)
pub fn _about_the_children_reply(memory: &mut Memory, low: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_about_the_children_reply")
}

/// Python: bot/memory/store_replies.py:8430 `_a_list_of_clauses(clauses)` (never called on the trace corpus)
pub fn _a_list_of_clauses(clauses: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_a_list_of_clauses")
}

/// Python: bot/memory/store_replies.py:8437 `_the_person_called(memory, line)`
pub fn _the_person_called(memory: &mut Memory, line: &str) -> String {
    crate::unported!("bot.memory.store_replies:_the_person_called")
}

/// Python: bot/memory/store_replies.py:8454 `_covers(said, clauses)` (never called on the trace corpus)
pub fn _covers(said: &str, clauses: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_covers")
}

/// Python: bot/memory/store_replies.py:8463 `_tokens(text)` (never called on the trace corpus)
pub fn _tokens(text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_tokens")
}

/// Python: bot/memory/store_replies.py:8473 `_said_everything(answer, memory)` (never called on the trace corpus)
pub fn _said_everything(answer: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_said_everything")
}

/// Python: bot/memory/store_replies.py:8490 `_a_pregnancy_of_their_own(text, memory, low)`
pub fn _a_pregnancy_of_their_own(text: &str, memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_a_pregnancy_of_their_own")
}

/// Python: bot/memory/store_replies.py:8515 `_a_state_told(text, memory, low)`
pub fn _a_state_told(text: &str, memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_a_state_told")
}

/// Python: bot/memory/store_replies.py:8560 `_a_state_that_was(text, memory, low)`
pub fn _a_state_that_was(text: &str, memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_a_state_that_was")
}

/// Python: bot/memory/store_replies.py:8583 `_pet_part_reply(memory, low)`
pub fn _pet_part_reply(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_pet_part_reply")
}

/// Python: bot/memory/store_replies.py:8593 `_pet_breed_reply(memory, low)`
pub fn _pet_breed_reply(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_pet_breed_reply")
}

/// Python: bot/memory/store_replies.py:8637 `_a_number(value)` (never called on the trace corpus)
pub fn _a_number(value: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_a_number")
}

/// Python: bot/memory/store_replies.py:8644 `_a_pronoun_age(memory, low)` (never called on the trace corpus)
pub fn _a_pronoun_age(memory: &mut Memory, low: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_a_pronoun_age")
}

/// Python: bot/memory/store_replies.py:8710 `_a_pronoun_word(text)` (never called on the trace corpus)
pub fn _a_pronoun_word(text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_a_pronoun_word")
}

/// Python: bot/memory/store_replies.py:8720 `_someone_else_named(low, memory)`
pub fn _someone_else_named(low: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.memory.store_replies:_someone_else_named")
}

/// Python: bot/memory/store_replies.py:8776 `_the_same_fact_about_someone(text, memory)`
pub fn _the_same_fact_about_someone(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_the_same_fact_about_someone")
}

/// Python: bot/memory/store_replies.py:8839 `_the_fact_said_against(kind, value, rest)` (never called on the trace corpus)
pub fn _the_fact_said_against(kind: &str, value: &Value, rest: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_the_fact_said_against")
}

/// Python: bot/memory/store_replies.py:8857 `_the_person_last_given_a_fact(memory)` (never called on the trace corpus)
pub fn _the_person_last_given_a_fact(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_the_person_last_given_a_fact")
}

/// Python: bot/memory/store_replies.py:8867 `_take_the_fact_back(memory, who, kind)` (never called on the trace corpus)
pub fn _take_the_fact_back(memory: &mut Memory, who: &str, kind: &str) {
    crate::unported!("bot.memory.store_replies:_take_the_fact_back")
}

/// Python: bot/memory/store_replies.py:8880 `_ive_taken_it_back(who, kind, value)` (never called on the trace corpus)
pub fn _ive_taken_it_back(who: &str, kind: &str, value: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_ive_taken_it_back")
}

/// Python: bot/memory/store_replies.py:8898 `_the_fact_on_record(memory)` (never called on the trace corpus)
pub fn _the_fact_on_record(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_the_fact_on_record")
}

/// Python: bot/memory/store_replies.py:8912 `_both_of_you(who)` (never called on the trace corpus)
pub fn _both_of_you(who: &str) -> String {
    crate::unported!("bot.memory.store_replies:_both_of_you")
}

/// Python: bot/memory/store_replies.py:8916 `_this_too(who, kind, value)` (never called on the trace corpus)
pub fn _this_too(who: &str, kind: &str, value: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_this_too")
}

/// Python: bot/memory/store_replies.py:8927 `_isnt_this_too(who, kind, value)` (never called on the trace corpus)
pub fn _isnt_this_too(who: &str, kind: &str, value: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_isnt_this_too")
}

/// Python: bot/memory/store_replies.py:8941 `_the_person_still_in_hand(low, memory)`
pub fn _the_person_still_in_hand(low: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.memory.store_replies:_the_person_still_in_hand")
}

/// Python: bot/memory/store_replies.py:8954 `_a_food_word(low, entry)` (never called on the trace corpus)
pub fn _a_food_word(low: &str, entry: &'static Entry) -> bool {
    crate::unported!("bot.memory.store_replies:_a_food_word")
}

/// Python: bot/memory/store_replies.py:8969 `_other_person_reply(memory, low)`
pub fn _other_person_reply(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_other_person_reply")
}

/// Python: bot/memory/store_replies.py:9118 `_asks_for_a_limit_not_advice(low)`
pub fn _asks_for_a_limit_not_advice(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_asks_for_a_limit_not_advice")
}

/// Python: bot/memory/store_replies.py:9139 `_the_pronoun_in(low)`
pub fn _the_pronoun_in(low: &str) -> String {
    crate::unported!("bot.memory.store_replies:_the_pronoun_in")
}

/// Python: bot/memory/store_replies.py:9153 `_nobody_else_kept(memory)`
pub fn _nobody_else_kept(memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_nobody_else_kept")
}

/// Python: bot/memory/store_replies.py:9171 `_a_bare_limit_ask(low, memory)`
pub fn _a_bare_limit_ask(low: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_a_bare_limit_ask")
}

/// Python: bot/memory/store_replies.py:9191 `_nothing_kept_about_them(low)` (never called on the trace corpus)
pub fn _nothing_kept_about_them(low: &str) -> String {
    crate::unported!("bot.memory.store_replies:_nothing_kept_about_them")
}

/// Python: bot/memory/store_replies.py:9209 `_did_you_mean_the_person(memory, low)`
pub fn _did_you_mean_the_person(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_did_you_mean_the_person")
}

/// Python: bot/memory/store_replies.py:9251 `_condition_is_their_food(who, condition, low='')` (never called on the trace corpus)
pub fn _condition_is_their_food(who: &str, condition: &Value, low: &str) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_condition_is_their_food")
}

/// Python: bot/memory/store_replies.py:9281 `_diet_sentence(who, diet)` (never called on the trace corpus)
pub fn _diet_sentence(who: &str, diet: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_diet_sentence")
}

/// Python: bot/memory/store_replies.py:9310 `car_line(car)` (never called on the trace corpus)
pub fn car_line(car: &Value) -> String {
    crate::unported!("bot.memory.store_replies:car_line")
}

/// Python: bot/memory/store_replies.py:9333 `_birthday_in_words(memory)` (never called on the trace corpus)
pub fn _birthday_in_words(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_birthday_in_words")
}

/// Python: bot/memory/store_replies.py:9356 `_birthday_reply(memory)` (never called on the trace corpus)
pub fn _birthday_reply(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_birthday_reply")
}

/// Python: bot/memory/store_replies.py:9386 `_the_same_relative(one, other)` (never called on the trace corpus)
pub fn _the_same_relative(one: &str, other: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_the_same_relative")
}

/// Python: bot/memory/store_replies.py:9396 `_a_persons_birthday_reply(memory, low)`
pub fn _a_persons_birthday_reply(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_a_persons_birthday_reply")
}

/// Python: bot/memory/store_replies.py:9459 `_weight_reply(memory)` (never called on the trace corpus)
pub fn _weight_reply(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_weight_reply")
}

/// Python: bot/memory/store_replies.py:9498 `_height_reply(memory)` (never called on the trace corpus)
pub fn _height_reply(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_height_reply")
}

/// Python: bot/memory/store_replies.py:9514 `_fact_reply(memory, low, key=None)`
pub fn _fact_reply(memory: &mut Memory, low: &str, key: &str) -> String {
    crate::unported!("bot.memory.store_replies:_fact_reply")
}

/// Python: bot/memory/store_replies.py:9684 `_was_called(named)` (never called on the trace corpus)
pub fn _was_called(named: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_was_called")
}

/// Python: bot/memory/store_replies.py:9692 `_a_pet_that_is_gone(gone)` (never called on the trace corpus)
pub fn _a_pet_that_is_gone(gone: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_a_pet_that_is_gone")
}

/// Python: bot/memory/store_replies.py:9715 `_put_together(parts)` (never called on the trace corpus)
pub fn _put_together(parts: &[String]) -> Value {
    crate::unported!("bot.memory.store_replies:_put_together")
}

/// Python: bot/memory/store_replies.py:9724 `_pets_together(pets)` (never called on the trace corpus)
pub fn _pets_together(pets: &Value) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_pets_together")
}

/// Python: bot/memory/store_replies.py:9755 `_a_pet(pet)` (never called on the trace corpus)
pub fn _a_pet(pet: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_a_pet")
}

/// Python: bot/memory/store_replies.py:9783 `_a_why_line_they_have_answered(text, memory)`
pub fn _a_why_line_they_have_answered(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_a_why_line_they_have_answered")
}

/// Python: bot/memory/store_replies.py:9788 `_holds_the_subject(memory, text)`
pub fn _holds_the_subject(memory: &mut Memory, text: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_holds_the_subject")
}

/// Python: bot/memory/store_replies.py:9804 `_asks_for_a_fact(low, text='', memory=None)`
pub fn _asks_for_a_fact(low: &str, text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_asks_for_a_fact")
}

/// Python: bot/memory/store_replies.py:9829 `_vary(options, memory)`
pub fn _vary(options: &[String], memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_vary")
}

/// Python: bot/memory/store_replies.py:9837 `_turn_lines(memory)` (never called on the trace corpus)
pub fn _turn_lines(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_turn_lines")
}

/// Python: bot/memory/store_replies.py:9845 `_repeat_ask(text, low, memory=None)`
pub fn _repeat_ask(text: &str, low: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_repeat_ask")
}

/// Python: bot/memory/store_replies.py:9866 `asks_again(text, memory=None)`
pub fn asks_again(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:asks_again")
}

/// Python: bot/memory/store_replies.py:9877 `_repeat_reply(memory, text='')` (never called on the trace corpus)
pub fn _repeat_reply(memory: &Memory, text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_repeat_reply")
}

/// Python: bot/memory/store_replies.py:9946 `_a_whole_answer_to_hand_back(memory, text='')` (never called on the trace corpus)
pub fn _a_whole_answer_to_hand_back(memory: &Memory, text: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_whole_answer_to_hand_back")
}

/// Python: bot/memory/store_replies.py:9962 `_carries_something(turn)` (never called on the trace corpus)
pub fn _carries_something(turn: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_carries_something")
}

/// Python: bot/memory/store_replies.py:9985 `_only_a_nod(line)` (never called on the trace corpus)
pub fn _only_a_nod(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_only_a_nod")
}

/// Python: bot/memory/store_replies.py:9992 `_hearing_again(said, line)` (never called on the trace corpus)
pub fn _hearing_again(said: &str, line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_hearing_again")
}

/// Python: bot/memory/store_replies.py:10032 `_just_a_hello(line)` (never called on the trace corpus)
pub fn _just_a_hello(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_just_a_hello")
}

/// Python: bot/memory/store_replies.py:10045 `_worth_reporting(line)` (never called on the trace corpus)
pub fn _worth_reporting(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_worth_reporting")
}

/// Python: bot/memory/store_replies.py:10056 `_first_reply(memory)` (never called on the trace corpus)
pub fn _first_reply(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_first_reply")
}

/// Python: bot/memory/store_replies.py:10075 `_what_i_said_about(memory, thing)` (never called on the trace corpus)
pub fn _what_i_said_about(memory: &Memory, thing: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_what_i_said_about")
}

/// Python: bot/memory/store_replies.py:10126 `_adds_something(said, asked)` (never called on the trace corpus)
pub fn _adds_something(said: &str, asked: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_adds_something")
}

/// Python: bot/memory/store_replies.py:10137 `_first_bot_reply(memory)` (never called on the trace corpus)
pub fn _first_bot_reply(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_first_bot_reply")
}

/// Python: bot/memory/store_replies.py:10146 `_asked_and_answered(memory)` (never called on the trace corpus)
pub fn _asked_and_answered(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_asked_and_answered")
}

/// Python: bot/memory/store_replies.py:10162 `_said_reply(memory)` (never called on the trace corpus)
pub fn _said_reply(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_said_reply")
}

/// Python: bot/memory/store_replies.py:10242 `_corrects(line)` (never called on the trace corpus)
pub fn _corrects(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_corrects")
}

/// Python: bot/memory/store_replies.py:10246 `_last_real(memory)` (never called on the trace corpus)
pub fn _last_real(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_last_real")
}

/// Python: bot/memory/store_replies.py:10265 `_the_fact_is_kept(key, memory)` (never called on the trace corpus)
pub fn _the_fact_is_kept(key: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_the_fact_is_kept")
}

/// Python: bot/memory/store_replies.py:10270 `_a_denial(reply)` (never called on the trace corpus)
pub fn _a_denial(reply: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_denial")
}

/// Python: bot/memory/store_replies.py:10274 `_names_its_thing(low)`
pub fn _names_its_thing(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_names_its_thing")
}

/// Python: bot/memory/store_replies.py:10290 `_say_back(line, memory=None)` (never called on the trace corpus)
pub fn _say_back(line: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_say_back")
}

/// Python: bot/memory/store_replies.py:10306 `_recall_reply(memory, low, text='', spoken=False)` (never called on the trace corpus)
pub fn _recall_reply(memory: &mut Memory, low: &str, text: &str, spoken: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_recall_reply")
}

/// Python: bot/memory/store_replies.py:10487 `_tagged(line)` (never called on the trace corpus)
pub fn _tagged(line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_tagged")
}

/// Python: bot/memory/store_replies.py:10492 `_the_same_person(who)` (never called on the trace corpus)
pub fn _the_same_person(who: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_same_person")
}

/// Python: bot/memory/store_replies.py:10513 `_the_name_pattern(who)` (never called on the trace corpus)
pub fn _the_name_pattern(who: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_name_pattern")
}

/// Python: bot/memory/store_replies.py:10535 `_the_word_for_them(who)` (never called on the trace corpus)
pub fn _the_word_for_them(who: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_word_for_them")
}

/// Python: bot/memory/store_replies.py:10574 `_a_stay_in_a_place(text)` (never called on the trace corpus)
pub fn _a_stay_in_a_place(text: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_stay_in_a_place")
}

/// Python: bot/memory/store_replies.py:10598 `_kind(thing)` (never called on the trace corpus)
pub fn _kind(thing: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_kind")
}

/// Python: bot/memory/store_replies.py:10605 `_says_where_they_live(text)` (never called on the trace corpus)
pub fn _says_where_they_live(text: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_says_where_they_live")
}

/// Python: bot/memory/store_replies.py:10620 `_a_place_they_go(text)` (never called on the trace corpus)
pub fn _a_place_they_go(text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_a_place_they_go")
}

/// Python: bot/memory/store_replies.py:10634 `_about_kind(thing, text, memory=None, phrase=False)` (never called on the trace corpus)
pub fn _about_kind(thing: &str, text: &str, memory: &mut Memory, phrase: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_about_kind")
}

/// Python: bot/memory/store_replies.py:10666 `_seen(line)` (never called on the trace corpus)
pub fn _seen(line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_seen")
}

/// Python: bot/memory/store_replies.py:10681 `_alike(word, seen)` (never called on the trace corpus)
pub fn _alike(word: &str, seen: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_alike")
}

/// Python: bot/memory/store_replies.py:10704 `_recall_quote(line, memory=None, asked=False)` (never called on the trace corpus)
pub fn _recall_quote(line: &str, memory: &Memory, asked: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_recall_quote")
}

/// Python: bot/memory/store_replies.py:10739 `_a_figure_put_right(line, memory)` (never called on the trace corpus)
pub fn _a_figure_put_right(line: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_a_figure_put_right")
}

/// Python: bot/memory/store_replies.py:10778 `_a_narrowing_put_after(line, memory)` (never called on the trace corpus)
pub fn _a_narrowing_put_after(line: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_a_narrowing_put_after")
}

/// Python: bot/memory/store_replies.py:10795 `_put_right_after_it(line, memory)` (never called on the trace corpus)
pub fn _put_right_after_it(line: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_put_right_after_it")
}

/// Python: bot/memory/store_replies.py:10824 `_age_reply(memory)` (never called on the trace corpus)
pub fn _age_reply(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_age_reply")
}

/// Python: bot/memory/store_replies.py:10833 `_content_words(line)` (never called on the trace corpus)
pub fn _content_words(line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_content_words")
}

/// Python: bot/memory/store_replies.py:10837 `_the_line_before_was_a_plan(memory)` (never called on the trace corpus)
pub fn _the_line_before_was_a_plan(memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_the_line_before_was_a_plan")
}

/// Python: bot/memory/store_replies.py:10854 `_plan_was_corrected(memory)` (never called on the trace corpus)
pub fn _plan_was_corrected(memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_plan_was_corrected")
}

/// Python: bot/memory/store_replies.py:10871 `_about_the_same(line, line_words)` (never called on the trace corpus)
pub fn _about_the_same(line: &str, line_words: &[String]) -> bool {
    crate::unported!("bot.memory.store_replies:_about_the_same")
}

/// Python: bot/memory/store_replies.py:10875 `_activity_line(memory)` (never called on the trace corpus)
pub fn _activity_line(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_activity_line")
}

/// Python: bot/memory/store_replies.py:10906 `_the_making_line_they_said(memory, back=8)` (never called on the trace corpus)
pub fn _the_making_line_they_said(memory: &Memory, back: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_the_making_line_they_said")
}

/// Python: bot/memory/store_replies.py:10924 `_recap(line)` (never called on the trace corpus)
pub fn _recap(line: &str) -> String {
    crate::unported!("bot.memory.store_replies:_recap")
}

/// Python: bot/memory/store_replies.py:10940 `_tidy_words(text)` (never called on the trace corpus)
pub fn _tidy_words(text: &str) -> String {
    crate::unported!("bot.memory.store_replies:_tidy_words")
}

/// Python: bot/memory/store_replies.py:10961 `_a_thing(name)` (never called on the trace corpus)
pub fn _a_thing(name: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_thing")
}

/// Python: bot/memory/store_replies.py:10983 `_says_the_thing(said, thing)` (never called on the trace corpus)
pub fn _says_the_thing(said: &str, thing: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_says_the_thing")
}

/// Python: bot/memory/store_replies.py:10993 `_a_description(thing)` (never called on the trace corpus)
pub fn _a_description(thing: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_description")
}

/// Python: bot/memory/store_replies.py:11005 `_the_fixed_fact(memory)` (never called on the trace corpus)
pub fn _the_fixed_fact(memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_the_fixed_fact")
}

/// Python: bot/memory/store_replies.py:11028 `_the_line_named(memory, low)` (never called on the trace corpus)
pub fn _the_line_named(memory: &Memory, low: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_line_named")
}

/// Python: bot/memory/store_replies.py:11051 `_topic_reply(memory, low, again=False)` (never called on the trace corpus)
pub fn _topic_reply(memory: &mut Memory, low: &str, again: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_topic_reply")
}

/// Python: bot/memory/store_replies.py:11157 `_the_last_ask_about_the_bot(memory)` (never called on the trace corpus)
pub fn _the_last_ask_about_the_bot(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_the_last_ask_about_the_bot")
}

/// Python: bot/memory/store_replies.py:11173 `_asked_reply(memory)` (never called on the trace corpus)
pub fn _asked_reply(memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_asked_reply")
}

/// Python: bot/memory/store_replies.py:11198 `_last_told(memory)` (never called on the trace corpus)
pub fn _last_told(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_last_told")
}

/// Python: bot/memory/store_replies.py:11226 `_as_repaired(word)` (never called on the trace corpus)
pub fn _as_repaired(word: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_as_repaired")
}

/// Python: bot/memory/store_replies.py:11230 `_with_the_name(said, raw)` (never called on the trace corpus)
pub fn _with_the_name(said: &str, raw: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_with_the_name")
}

/// Python: bot/memory/store_replies.py:11276 `_reads_as_a_correction(line)` (never called on the trace corpus)
pub fn _reads_as_a_correction(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_reads_as_a_correction")
}

/// Python: bot/memory/store_replies.py:11286 `_lines_that_moved(memory)` (never called on the trace corpus)
pub fn _lines_that_moved(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_lines_that_moved")
}

/// Python: bot/memory/store_replies.py:11325 `_the_summary_already_says(memory, flat)` (never called on the trace corpus)
pub fn _the_summary_already_says(memory: &mut Memory, flat: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_the_summary_already_says")
}

/// Python: bot/memory/store_replies.py:11343 `_safety_lines(memory)` (never called on the trace corpus)
pub fn _safety_lines(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_safety_lines")
}

/// Python: bot/memory/store_replies.py:11371 `_shape(line)` (never called on the trace corpus)
pub fn _shape(line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_shape")
}

/// Python: bot/memory/store_replies.py:11383 `_out_of_date_lines(memory)` (never called on the trace corpus)
pub fn _out_of_date_lines(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_out_of_date_lines")
}

/// Python: bot/memory/store_replies.py:11427 `_root(word)` (never called on the trace corpus)
pub fn _root(word: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_root")
}

/// Python: bot/memory/store_replies.py:11440 `_one_word(first, second)` (never called on the trace corpus)
pub fn _one_word(first: &Value, second: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_one_word")
}

/// Python: bot/memory/store_replies.py:11475 `_judging_themselves(line)` (never called on the trace corpus)
pub fn _judging_themselves(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_judging_themselves")
}

/// Python: bot/memory/store_replies.py:11480 `_recent_told(memory, limit=3)` (never called on the trace corpus)
pub fn _recent_told(memory: &mut Memory, limit: i64) -> Value {
    crate::unported!("bot.memory.store_replies:_recent_told")
}

/// Python: bot/memory/store_replies.py:11567 `_all_about_the_bot(memory)` (never called on the trace corpus)
pub fn _all_about_the_bot(memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_all_about_the_bot")
}

/// Python: bot/memory/store_replies.py:11582 `_a_figure_the_store_could_not_keep(line, memory)` (never called on the trace corpus)
pub fn _a_figure_the_store_could_not_keep(line: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_a_figure_the_store_could_not_keep")
}

/// Python: bot/memory/store_replies.py:11597 `_the_thing_the_chat_is_on(memory)` (never called on the trace corpus)
pub fn _the_thing_the_chat_is_on(memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_the_thing_the_chat_is_on")
}

/// Python: bot/memory/store_replies.py:11618 `_a_fact_of_its_own(clause)` (never called on the trace corpus)
pub fn _a_fact_of_its_own(clause: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_fact_of_its_own")
}

/// Python: bot/memory/store_replies.py:11623 `_told_phrase(text)` (never called on the trace corpus)
pub fn _told_phrase(text: &str) -> String {
    crate::unported!("bot.memory.store_replies:_told_phrase")
}

/// Python: bot/memory/store_replies.py:11661 `_expanded(text)` (never called on the trace corpus)
pub fn _expanded(text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_expanded")
}

/// Python: bot/memory/store_replies.py:11678 `_a_judgement_was_said(memory)` (never called on the trace corpus)
pub fn _a_judgement_was_said(memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_a_judgement_was_said")
}

/// Python: bot/memory/store_replies.py:11696 `_plain_words(text)` (never called on the trace corpus)
pub fn _plain_words(text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_plain_words")
}

/// Python: bot/memory/store_replies.py:11700 `_says_nothing_new(phrase, told_words, body_words)` (never called on the trace corpus)
pub fn _says_nothing_new(phrase: &str, told_words: &[String], body_words: &[String]) -> bool {
    crate::unported!("bot.memory.store_replies:_says_nothing_new")
}

/// Python: bot/memory/store_replies.py:11713 `_summary_words(text)` (never called on the trace corpus)
pub fn _summary_words(text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_summary_words")
}

/// Python: bot/memory/store_replies.py:11730 `_already_said(words, said)` (never called on the trace corpus)
pub fn _already_said(words: &[String], said: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_already_said")
}

/// Python: bot/memory/store_replies.py:11748 `_taste_reply(memory, low)` (never called on the trace corpus)
pub fn _taste_reply(memory: &mut Memory, low: &str) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_taste_reply")
}

/// Python: bot/memory/store_replies.py:11781 `_tastes_said(memory)` (never called on the trace corpus)
pub fn _tastes_said(memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_tastes_said")
}

/// Python: bot/memory/store_replies.py:11813 `_without_the_age(line)` (never called on the trace corpus)
pub fn _without_the_age(line: &str) -> String {
    crate::unported!("bot.memory.store_replies:_without_the_age")
}

/// Python: bot/memory/store_replies.py:11818 `_the_words_of_a_fact(value)` (never called on the trace corpus)
pub fn _the_words_of_a_fact(value: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_the_words_of_a_fact")
}

/// Python: bot/memory/store_replies.py:11833 `_a_fact_the_recap_does_not_say(memory, phrase, body_words)` (never called on the trace corpus)
pub fn _a_fact_the_recap_does_not_say(memory: &mut Memory, phrase: &str, body_words: &[String]) -> bool {
    crate::unported!("bot.memory.store_replies:_a_fact_the_recap_does_not_say")
}

/// Python: bot/memory/store_replies.py:11855 `_kids_told(memory)` (never called on the trace corpus)
pub fn _kids_told(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_kids_told")
}

/// Python: bot/memory/store_replies.py:11871 `_a_child_is_described(memory)` (never called on the trace corpus)
pub fn _a_child_is_described(memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_a_child_is_described")
}

/// Python: bot/memory/store_replies.py:11880 `_a_few_children(count)` (never called on the trace corpus)
pub fn _a_few_children(count: i64) -> String {
    crate::unported!("bot.memory.store_replies:_a_few_children")
}

/// Python: bot/memory/store_replies.py:11909 `_summed_up(low)`
pub fn _summed_up(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_summed_up")
}

/// Python: bot/memory/store_replies.py:11913 `_summary_reply(memory)` (never called on the trace corpus)
pub fn _summary_reply(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_summary_reply")
}

/// Python: bot/memory/store_replies.py:12170 `_quote(text, limit=90)` (never called on the trace corpus)
pub fn _quote(text: &str, limit: i64) -> String {
    crate::unported!("bot.memory.store_replies:_quote")
}

/// Python: bot/memory/store_replies.py:12192 `_heavy(line)` (never called on the trace corpus)
pub fn _heavy(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_heavy")
}

/// Python: bot/memory/store_replies.py:12196 `_topics_reply(memory)` (never called on the trace corpus)
pub fn _topics_reply(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_topics_reply")
}

/// Python: bot/memory/store_replies.py:12216 `_first_sentence(text, limit=70)` (never called on the trace corpus)
pub fn _first_sentence(text: &str, limit: i64) -> String {
    crate::unported!("bot.memory.store_replies:_first_sentence")
}

/// Python: bot/memory/store_replies.py:12295 `_sentences(answer)` (never called on the trace corpus)
pub fn _sentences(answer: &str) -> Vec<String> {
    crate::unported!("bot.memory.store_replies:_sentences")
}

/// Python: bot/memory/store_replies.py:12299 `_halves(sentence)` (never called on the trace corpus)
pub fn _halves(sentence: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_halves")
}

/// Python: bot/memory/store_replies.py:12308 `_says_something(part)` (never called on the trace corpus)
pub fn _says_something(part: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_says_something")
}

/// Python: bot/memory/store_replies.py:12320 `_seconds(said)` (never called on the trace corpus)
pub fn _seconds(said: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_seconds")
}

/// Python: bot/memory/store_replies.py:12345 `_names_minutes(said)` (never called on the trace corpus)
pub fn _names_minutes(said: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_names_minutes")
}

/// Python: bot/memory/store_replies.py:12350 `_longest(said, bare=False)` (never called on the trace corpus)
pub fn _longest(said: &str, bare: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_longest")
}

/// Python: bot/memory/store_replies.py:12381 `_a_remark_about_a_length(answer)` (never called on the trace corpus)
pub fn _a_remark_about_a_length(answer: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_a_remark_about_a_length")
}

/// Python: bot/memory/store_replies.py:12397 `_moves(answer)` (never called on the trace corpus)
pub fn _moves(answer: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_moves")
}

/// Python: bot/memory/store_replies.py:12408 `_up_to_the_move(answer, part)` (never called on the trace corpus)
pub fn _up_to_the_move(answer: &str, part: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_up_to_the_move")
}

/// Python: bot/memory/store_replies.py:12430 `_the_move_that_fits(moves, room)` (never called on the trace corpus)
pub fn _the_move_that_fits(moves: &Value, room: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_the_move_that_fits")
}

/// Python: bot/memory/store_replies.py:12456 `_left_out(moves, room)` (never called on the trace corpus)
pub fn _left_out(moves: &Value, room: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_left_out")
}

/// Python: bot/memory/store_replies.py:12460 `_the_lead(answer)` (never called on the trace corpus)
pub fn _the_lead(answer: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_lead")
}

/// Python: bot/memory/store_replies.py:12478 `_a_rate_of_the_same_length(answer, room)` (never called on the trace corpus)
pub fn _a_rate_of_the_same_length(answer: &str, room: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_a_rate_of_the_same_length")
}

/// Python: bot/memory/store_replies.py:12489 `_the_budget_on_the_answer(text, memory)`
pub fn _the_budget_on_the_answer(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_the_budget_on_the_answer")
}

/// Python: bot/memory/store_replies.py:12611 `_theirs(length, mid=False)` (never called on the trace corpus)
pub fn _theirs(length: &Value, mid: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_theirs")
}

/// Python: bot/memory/store_replies.py:12620 `_tidy(part)` (never called on the trace corpus)
pub fn _tidy(part: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_tidy")
}

/// Python: bot/memory/store_replies.py:12633 `_clauses(reply)` (never called on the trace corpus)
pub fn _clauses(reply: &str) -> Vec<String> {
    crate::unported!("bot.memory.store_replies:_clauses")
}

/// Python: bot/memory/store_replies.py:12638 `_the_limit(reply)` (never called on the trace corpus)
pub fn _the_limit(reply: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_limit")
}

/// Python: bot/memory/store_replies.py:12648 `_the_condition(reply)` (never called on the trace corpus)
pub fn _the_condition(reply: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_condition")
}

/// Python: bot/memory/store_replies.py:12662 `_recent_answers(memory, back=3)` (never called on the trace corpus)
pub fn _recent_answers(memory: &Memory, back: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_recent_answers")
}

/// Python: bot/memory/store_replies.py:12683 `_shares_a_word(low, said)` (never called on the trace corpus)
pub fn _shares_a_word(low: &str, said: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_shares_a_word")
}

/// Python: bot/memory/store_replies.py:12694 `_the_call_from_the_advice(text, memory)`
pub fn _the_call_from_the_advice(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_the_call_from_the_advice")
}

/// Python: bot/memory/store_replies.py:12743 `_trim(text, limit=90)` (never called on the trace corpus)
pub fn _trim(text: &str, limit: i64) -> Value {
    crate::unported!("bot.memory.store_replies:_trim")
}

/// Python: bot/memory/store_replies.py:12751 `_food_facts(memory)`
pub fn _food_facts(memory: &mut Memory) -> Vec<String> {
    crate::unported!("bot.memory.store_replies:_food_facts")
}

/// Python: bot/memory/store_replies.py:12773 `_the_food_a_condition_rules_out(condition)` (never called on the trace corpus)
pub fn _the_food_a_condition_rules_out(condition: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_the_food_a_condition_rules_out")
}

/// Python: bot/memory/store_replies.py:12788 `_what_a_dinner_has_to_leave_out(memory)` (never called on the trace corpus)
pub fn _what_a_dinner_has_to_leave_out(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_what_a_dinner_has_to_leave_out")
}

/// Python: bot/memory/store_replies.py:12820 `_all_of_them(count)` (never called on the trace corpus)
pub fn _all_of_them(count: i64) -> String {
    crate::unported!("bot.memory.store_replies:_all_of_them")
}

/// Python: bot/memory/store_replies.py:12834 `_say_a_list(items)` (never called on the trace corpus)
pub fn _say_a_list(items: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_say_a_list")
}

/// Python: bot/memory/store_replies.py:12943 `_reads_as_the_invitation(text, memory)`
pub fn _reads_as_the_invitation(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_reads_as_the_invitation")
}

/// Python: bot/memory/store_replies.py:12958 `_the_module_behind(memory, said)` (never called on the trace corpus)
pub fn _the_module_behind(memory: &Memory, said: &str) -> String {
    crate::unported!("bot.memory.store_replies:_the_module_behind")
}

/// Python: bot/memory/store_replies.py:12966 `_the_line_before_the_check(memory, line)` (never called on the trace corpus)
pub fn _the_line_before_the_check(memory: &Memory, line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_line_before_the_check")
}

/// Python: bot/memory/store_replies.py:12980 `_a_check_choice(options, memory, salt)` (never called on the trace corpus)
pub fn _a_check_choice(options: &Value, memory: &Memory, salt: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_a_check_choice")
}

/// Python: bot/memory/store_replies.py:12987 `_a_check_came_before(memory, line)` (never called on the trace corpus)
pub fn _a_check_came_before(memory: &Memory, line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_check_came_before")
}

/// Python: bot/memory/store_replies.py:13007 `_the_followed_check(text, memory)`
pub fn _the_followed_check(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_the_followed_check")
}

/// Python: bot/memory/store_replies.py:13085 `_plain_line(text)`
pub fn _plain_line(text: &str) -> String {
    crate::unported!("bot.memory.store_replies:_plain_line")
}

/// Python: bot/memory/store_replies.py:13115 `_a_subject_to_leave_alone(low)`
pub fn _a_subject_to_leave_alone(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_subject_to_leave_alone")
}

/// Python: bot/memory/store_replies.py:13140 `_closes_the_subject(text)`
pub fn _closes_the_subject(text: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_closes_the_subject")
}

/// Python: bot/memory/store_replies.py:13161 `_closes_by_a_frame(low)`
pub fn _closes_by_a_frame(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_closes_by_a_frame")
}

/// Python: bot/memory/store_replies.py:13170 `_a_plain_frame_closes(low)` (never called on the trace corpus)
pub fn _a_plain_frame_closes(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_plain_frame_closes")
}

/// Python: bot/memory/store_replies.py:13182 `_held_thread(memory)` (never called on the trace corpus)
pub fn _held_thread(memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_held_thread")
}

/// Python: bot/memory/store_replies.py:13193 `_gentle_subject(memory)` (never called on the trace corpus)
pub fn _gentle_subject(memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_gentle_subject")
}

/// Python: bot/memory/store_replies.py:13205 `_can_close(text, memory)`
pub fn _can_close(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_can_close")
}

/// Python: bot/memory/store_replies.py:13226 `_the_close(text, memory)`
pub fn _the_close(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_the_close")
}

