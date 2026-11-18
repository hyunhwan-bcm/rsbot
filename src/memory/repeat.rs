//! Port of `bot/memory/repeat.py`.
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

/// Python: bot/memory/repeat.py:84 `_doings(text)`
pub fn _doings(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.repeat:_doings")
}

/// Python: bot/memory/repeat.py:90 `asks_for_the_same_work(text, memory)` (never called on the trace corpus)
pub fn asks_for_the_same_work(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.repeat:asks_for_the_same_work")
}

/// Python: bot/memory/repeat.py:153 `without_a_reset(text)`
pub fn without_a_reset(text: &str) -> String {
    crate::unported!("bot.memory.repeat:without_a_reset")
}

/// Python: bot/memory/repeat.py:492 `_the_first_instruction_in_a_long_piece(head, verbs)`
pub fn _the_first_instruction_in_a_long_piece(head: &str, verbs: &[String]) {
    crate::unported!("bot.memory.repeat:_the_first_instruction_in_a_long_piece")
}

/// Python: bot/memory/repeat.py:509 `_a_verb_after_a_leading_phrase(head, verbs)`
pub fn _a_verb_after_a_leading_phrase(head: &str, verbs: &[String]) -> bool {
    crate::unported!("bot.memory.repeat:_a_verb_after_a_leading_phrase")
}

/// Python: bot/memory/repeat.py:631 `only_said_it_back(line, said)`
pub fn only_said_it_back(line: &str, said: Option<&str>) -> bool {
    crate::unported!("bot.memory.repeat:only_said_it_back")
}

/// Python: bot/memory/repeat.py:655 `_an_echo(turn)` (never called on the trace corpus)
pub fn _an_echo(turn: &Value) -> bool {
    crate::unported!("bot.memory.repeat:_an_echo")
}

/// Python: bot/memory/repeat.py:663 `_names_no_topic(turn)` (never called on the trace corpus)
pub fn _names_no_topic(turn: &Value) -> Option<bool> {
    crate::unported!("bot.memory.repeat:_names_no_topic")
}

/// Python: bot/memory/repeat.py:675 `_only_echoes_between(memory, back)` (never called on the trace corpus)
pub fn _only_echoes_between(memory: &Memory, back: &Value) -> bool {
    crate::unported!("bot.memory.repeat:_only_echoes_between")
}

/// Python: bot/memory/repeat.py:721 `_a_help_ask(text)`
pub fn _a_help_ask(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:_a_help_ask")
}

/// Python: bot/memory/repeat.py:751 `asks_for_the_move(text)`
pub fn asks_for_the_move(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:asks_for_the_move")
}

/// Python: bot/memory/repeat.py:762 `names_nothing(text)`
pub fn names_nothing(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:names_nothing")
}

/// Python: bot/memory/repeat.py:793 `_names_a_thing_of_its_own(text)`
pub fn _names_a_thing_of_its_own(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.repeat:_names_a_thing_of_its_own")
}

/// Python: bot/memory/repeat.py:830 `asks_to_put_it_right(text)`
pub fn asks_to_put_it_right(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:asks_to_put_it_right")
}

/// Python: bot/memory/repeat.py:849 `a_step_of_the_same_answer(text)`
pub fn a_step_of_the_same_answer(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:a_step_of_the_same_answer")
}

/// Python: bot/memory/repeat.py:866 `_recent_answers(memory)`
pub fn _recent_answers(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.memory.repeat:_recent_answers")
}

/// Python: bot/memory/repeat.py:903 `_the_answer_about_what_we_are_on(memory, recent)`
pub fn _the_answer_about_what_we_are_on(memory: &Memory, recent: &Value) -> Value {
    crate::unported!("bot.memory.repeat:_the_answer_about_what_we_are_on")
}

/// Python: bot/memory/repeat.py:949 `_nothing_but_acknowledgements(back, memory)` (never called on the trace corpus)
pub fn _nothing_but_acknowledgements(back: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:_nothing_but_acknowledgements")
}

/// Python: bot/memory/repeat.py:969 `_about_the_same_thing(asked, said, memory)`
pub fn _about_the_same_thing(asked: &str, said: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:_about_the_same_thing")
}

/// Python: bot/memory/repeat.py:1038 `_short(said)` (never called on the trace corpus)
pub fn _short(said: &str) -> Value {
    crate::unported!("bot.memory.repeat:_short")
}

/// Python: bot/memory/repeat.py:1084 `_explains(sentence)`
pub fn _explains(sentence: &str) -> bool {
    crate::unported!("bot.memory.repeat:_explains")
}

/// Python: bot/memory/repeat.py:1093 `_a_step_comes_after(part)` (never called on the trace corpus)
pub fn _a_step_comes_after(part: &str) -> bool {
    crate::unported!("bot.memory.repeat:_a_step_comes_after")
}

/// Python: bot/memory/repeat.py:1108 `_opens_an_instruction(text)` (never called on the trace corpus)
pub fn _opens_an_instruction(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:_opens_an_instruction")
}

/// Python: bot/memory/repeat.py:1145 `_opening_move(part)`
pub fn _opening_move(part: &str) {
    crate::unported!("bot.memory.repeat:_opening_move")
}

/// Python: bot/memory/repeat.py:1254 `_a_question(said)` (never called on the trace corpus)
pub fn _a_question(said: &str) -> bool {
    crate::unported!("bot.memory.repeat:_a_question")
}

/// Python: bot/memory/repeat.py:1269 `_a_move(said, questions_are_steps=False)`
pub fn _a_move(said: &str, questions_are_steps: bool) {
    crate::unported!("bot.memory.repeat:_a_move")
}

/// Python: bot/memory/repeat.py:1287 `_a_move_in(said, questions_are_steps=False)`
pub fn _a_move_in(said: &str, questions_are_steps: bool) {
    crate::unported!("bot.memory.repeat:_a_move_in")
}

/// Python: bot/memory/repeat.py:1411 `_a_conditional_move(said)` (never called on the trace corpus)
pub fn _a_conditional_move(said: &str) -> Option<String> {
    crate::unported!("bot.memory.repeat:_a_conditional_move")
}

/// Python: bot/memory/repeat.py:1457 `asks_for_more(text)`
pub fn asks_for_more(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:asks_for_more")
}

/// Python: bot/memory/repeat.py:1470 `a_hand_back(said, memory)`
pub fn a_hand_back(said: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:a_hand_back")
}

/// Python: bot/memory/repeat.py:1484 `_handed_back_last(memory)` (never called on the trace corpus)
pub fn _handed_back_last(memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:_handed_back_last")
}

/// Python: bot/memory/repeat.py:1492 `_another_part(said, memory)` (never called on the trace corpus)
pub fn _another_part(said: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.repeat:_another_part")
}

/// Python: bot/memory/repeat.py:1557 `_said_in_a_turn(short, memory)` (never called on the trace corpus)
pub fn _said_in_a_turn(short: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:_said_in_a_turn")
}

/// Python: bot/memory/repeat.py:1568 `_says_what_to_do(part)` (never called on the trace corpus)
pub fn _says_what_to_do(part: &str) -> Value {
    crate::unported!("bot.memory.repeat:_says_what_to_do")
}

/// Python: bot/memory/repeat.py:1607 `_said_in_the_chat(said, memory)` (never called on the trace corpus)
pub fn _said_in_the_chat(said: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:_said_in_the_chat")
}

/// Python: bot/memory/repeat.py:1623 `_claims_what_it_never_said(reply, memory)`
pub fn _claims_what_it_never_said(reply: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.repeat:_claims_what_it_never_said")
}

/// Python: bot/memory/repeat.py:1637 `_handed_back(memory, reply)`
pub fn _handed_back(memory: &mut Memory, reply: &str) -> String {
    crate::unported!("bot.memory.repeat:_handed_back")
}

/// Python: bot/memory/repeat.py:1656 `_frame_heads()` (never called on the trace corpus)
pub fn _frame_heads() -> Value {
    crate::unported!("bot.memory.repeat:_frame_heads")
}

/// Python: bot/memory/repeat.py:1681 `framed(said)` (never called on the trace corpus)
pub fn framed(said: &str) -> bool {
    crate::unported!("bot.memory.repeat:framed")
}

/// Python: bot/memory/repeat.py:1693 `handed_back(memory, reply)` (never called on the trace corpus)
pub fn handed_back(memory: &mut Memory, reply: &str) -> Value {
    crate::unported!("bot.memory.repeat:handed_back")
}

/// Python: bot/memory/repeat.py:1702 `_already_repeated(short, memory)`
pub fn _already_repeated(short: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:_already_repeated")
}

/// Python: bot/memory/repeat.py:1724 `_handed_back_already(step, memory)`
pub fn _handed_back_already(step: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:_handed_back_already")
}

/// Python: bot/memory/repeat.py:1806 `_a_lead_of_its_own(short)` (never called on the trace corpus)
pub fn _a_lead_of_its_own(short: &Value) -> bool {
    crate::unported!("bot.memory.repeat:_a_lead_of_its_own")
}

/// Python: bot/memory/repeat.py:1826 `_kept_clause(text)` (never called on the trace corpus)
pub fn _kept_clause(text: &str) -> String {
    crate::unported!("bot.memory.repeat:_kept_clause")
}

/// Python: bot/memory/repeat.py:1849 `asks_for_less(text)`
pub fn asks_for_less(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:asks_for_less")
}

/// Python: bot/memory/repeat.py:1920 `_recent_lines(memory)` (never called on the trace corpus)
pub fn _recent_lines(memory: &Memory) -> Value {
    crate::unported!("bot.memory.repeat:_recent_lines")
}

/// Python: bot/memory/repeat.py:1955 `_sentences(said)`
pub fn _sentences(said: &str) -> Vec<String> {
    crate::unported!("bot.memory.repeat:_sentences")
}

/// Python: bot/memory/repeat.py:1979 `_most_condensed(said, memory=None)` (never called on the trace corpus)
pub fn _most_condensed(said: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.repeat:_most_condensed")
}

/// Python: bot/memory/repeat.py:2017 `_short_enough(cut)` (never called on the trace corpus)
pub fn _short_enough(cut: &Value) -> Value {
    crate::unported!("bot.memory.repeat:_short_enough")
}

/// Python: bot/memory/repeat.py:2047 `_ended(cut)` (never called on the trace corpus)
pub fn _ended(cut: &Value) -> Value {
    crate::unported!("bot.memory.repeat:_ended")
}

/// Python: bot/memory/repeat.py:2062 `_unbalanced(piece)`
pub fn _unbalanced(piece: &str) -> bool {
    crate::unported!("bot.memory.repeat:_unbalanced")
}

/// Python: bot/memory/repeat.py:2078 `_the_quotation(said)` (never called on the trace corpus)
pub fn _the_quotation(said: &str) -> Value {
    crate::unported!("bot.memory.repeat:_the_quotation")
}

/// Python: bot/memory/repeat.py:2103 `_closing_the_quote(cut, said)` (never called on the trace corpus)
pub fn _closing_the_quote(cut: &Value, said: &str) -> Value {
    crate::unported!("bot.memory.repeat:_closing_the_quote")
}

/// Python: bot/memory/repeat.py:2123 `_read_up_to_the_colon(said)` (never called on the trace corpus)
pub fn _read_up_to_the_colon(said: &str) -> String {
    crate::unported!("bot.memory.repeat:_read_up_to_the_colon")
}

/// Python: bot/memory/repeat.py:2131 `_keeps(said, what)` (never called on the trace corpus)
pub fn _keeps(said: &str, what: &str) -> Value {
    crate::unported!("bot.memory.repeat:_keeps")
}

/// Python: bot/memory/repeat.py:2152 `shorter_for(text, memory)`
pub fn shorter_for(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.repeat:shorter_for")
}

/// Python: bot/memory/repeat.py:2233 `_a_bare_time(part)`
pub fn _a_bare_time(part: Option<&str>) -> Option<String> {
    crate::unported!("bot.memory.repeat:_a_bare_time")
}

/// Python: bot/memory/repeat.py:2268 `_names_no_thing_of_its_own(text)` (never called on the trace corpus)
pub fn _names_no_thing_of_its_own(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:_names_no_thing_of_its_own")
}

/// Python: bot/memory/repeat.py:2277 `_the_price_the_answer_gave(text, memory)`
pub fn _the_price_the_answer_gave(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.repeat:_the_price_the_answer_gave")
}

/// Python: bot/memory/repeat.py:2298 `asks_what_that_costs(text, memory)`
pub fn asks_what_that_costs(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:asks_what_that_costs")
}

/// Python: bot/memory/repeat.py:2308 `_the_price_in_hand(text, memory)`
pub fn _the_price_in_hand(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.repeat:_the_price_in_hand")
}

/// Python: bot/memory/repeat.py:2336 `_the_time_an_earlier_answer_gave(text, memory)`
pub fn _the_time_an_earlier_answer_gave(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.repeat:_the_time_an_earlier_answer_gave")
}

/// Python: bot/memory/repeat.py:2377 `_a_length_already_handed(part, memory)` (never called on the trace corpus)
pub fn _a_length_already_handed(part: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:_a_length_already_handed")
}

/// Python: bot/memory/repeat.py:2392 `_a_what_to_do_answer(memory)` (never called on the trace corpus)
pub fn _a_what_to_do_answer(memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:_a_what_to_do_answer")
}

/// Python: bot/memory/repeat.py:2421 `_a_city_guide_is_the_answer_in_front(memory)`
pub fn _a_city_guide_is_the_answer_in_front(memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:_a_city_guide_is_the_answer_in_front")
}

/// Python: bot/memory/repeat.py:2438 `answer_for(text, memory)`
pub fn answer_for(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.repeat:answer_for")
}

/// Python: bot/memory/repeat.py:2625 `_an_entry_answers_it(text, memory)` (never called on the trace corpus)
pub fn _an_entry_answers_it(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:_an_entry_answers_it")
}

/// Python: bot/memory/repeat.py:2658 `_words(text)` (never called on the trace corpus)
pub fn _words(text: &str) -> Value {
    crate::unported!("bot.memory.repeat:_words")
}

/// Python: bot/memory/repeat.py:2663 `_a_bare_ask(text)` (never called on the trace corpus)
pub fn _a_bare_ask(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:_a_bare_ask")
}

/// Python: bot/memory/repeat.py:2675 `_a_lead_and_a_step(lead, step)`
pub fn _a_lead_and_a_step(lead: &str, step: &str) -> String {
    crate::unported!("bot.memory.repeat:_a_lead_and_a_step")
}

/// Python: bot/memory/repeat.py:2694 `_the_lead_for(text, step='')` (never called on the trace corpus)
pub fn _the_lead_for(text: &str, step: &Value) -> Value {
    crate::unported!("bot.memory.repeat:_the_lead_for")
}

/// Python: bot/memory/repeat.py:2709 `_content(text)`
pub fn _content(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.repeat:_content")
}

/// Python: bot/memory/repeat.py:2714 `_trouble(text)`
pub fn _trouble(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:_trouble")
}

/// Python: bot/memory/repeat.py:2758 `_a_thing_of_its_own(line, said)`
pub fn _a_thing_of_its_own(line: &str, said: &str) -> bool {
    crate::unported!("bot.memory.repeat:_a_thing_of_its_own")
}

/// Python: bot/memory/repeat.py:2789 `after_an_ack(text)`
pub fn after_an_ack(text: &str) -> String {
    crate::unported!("bot.memory.repeat:after_an_ack")
}

/// Python: bot/memory/repeat.py:2842 `_a_feeling(text)`
pub fn _a_feeling(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:_a_feeling")
}

/// Python: bot/memory/repeat.py:2860 `_the_doing(sentence)`
pub fn _the_doing(sentence: &str) -> String {
    crate::unported!("bot.memory.repeat:_the_doing")
}

/// Python: bot/memory/repeat.py:2886 `_the_advice(said)`
pub fn _the_advice(said: &str) -> String {
    crate::unported!("bot.memory.repeat:_the_advice")
}

/// Python: bot/memory/repeat.py:2908 `the_next_move(memory, text='')` (never called on the trace corpus)
pub fn the_next_move(memory: &Memory, text: &str) -> String {
    crate::unported!("bot.memory.repeat:the_next_move")
}

/// Python: bot/memory/repeat.py:2933 `_next_step(said)` (never called on the trace corpus)
pub fn _next_step(said: &str) -> Value {
    crate::unported!("bot.memory.repeat:_next_step")
}

/// Python: bot/memory/repeat.py:2942 `_later_part(tried, said)` (never called on the trace corpus)
pub fn _later_part(tried: &Value, said: &str) -> Value {
    crate::unported!("bot.memory.repeat:_later_part")
}

/// Python: bot/memory/repeat.py:2992 `_opens_a_doing(sentence)`
pub fn _opens_a_doing(sentence: &str) -> bool {
    crate::unported!("bot.memory.repeat:_opens_a_doing")
}

/// Python: bot/memory/repeat.py:3018 `_the_untried(tried, memory)`
pub fn _the_untried(tried: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.repeat:_the_untried")
}

/// Python: bot/memory/repeat.py:3055 `_tried(text)`
pub fn _tried(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:_tried")
}

/// Python: bot/memory/repeat.py:3059 `_says_they_agree(clean)` (never called on the trace corpus)
pub fn _says_they_agree(clean: &str) -> bool {
    crate::unported!("bot.memory.repeat:_says_they_agree")
}

/// Python: bot/memory/repeat.py:3079 `_the_thing_they_named(clean, part)` (never called on the trace corpus)
pub fn _the_thing_they_named(clean: &str, part: &str) -> String {
    crate::unported!("bot.memory.repeat:_the_thing_they_named")
}

/// Python: bot/memory/repeat.py:3112 `agreement_with_the_answer(text, memory)`
pub fn agreement_with_the_answer(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.repeat:agreement_with_the_answer")
}

/// Python: bot/memory/repeat.py:3148 `_agreed(named, memory)` (never called on the trace corpus)
pub fn _agreed(named: &Value, memory: &Memory) -> Value {
    crate::unported!("bot.memory.repeat:_agreed")
}

/// Python: bot/memory/repeat.py:3186 `_the_stems(word)`
pub fn _the_stems(word: &str) -> Vec<String> {
    crate::unported!("bot.memory.repeat:_the_stems")
}

/// Python: bot/memory/repeat.py:3203 `_the_act(word)`
pub fn _the_act(word: &str) {
    crate::unported!("bot.memory.repeat:_the_act")
}

/// Python: bot/memory/repeat.py:3227 `_the_object(clean, at)` (never called on the trace corpus)
pub fn _the_object(clean: &str, at: i64) -> Value {
    crate::unported!("bot.memory.repeat:_the_object")
}

/// Python: bot/memory/repeat.py:3247 `_the_steps_done(text)`
pub fn _the_steps_done(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.repeat:_the_steps_done")
}

/// Python: bot/memory/repeat.py:3261 `_the_answer_gives(said, act)` (never called on the trace corpus)
pub fn _the_answer_gives(said: &str, act: &Value) -> bool {
    crate::unported!("bot.memory.repeat:_the_answer_gives")
}

/// Python: bot/memory/repeat.py:3270 `_a_step_done(text, said)`
pub fn _a_step_done(text: &str, said: &str) -> bool {
    crate::unported!("bot.memory.repeat:_a_step_done")
}

/// Python: bot/memory/repeat.py:3303 `_the_step_just_given(text, memory)`
pub fn _the_step_just_given(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:_the_step_just_given")
}

/// Python: bot/memory/repeat.py:3317 `carried_on(text, memory)`
pub fn carried_on(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.repeat:carried_on")
}

/// Python: bot/memory/repeat.py:3495 `_starting_out_names_a_thing(line)`
pub fn _starting_out_names_a_thing(line: &str) -> bool {
    crate::unported!("bot.memory.repeat:_starting_out_names_a_thing")
}

/// Python: bot/memory/repeat.py:3506 `says_where_to_start(text)`
pub fn says_where_to_start(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:says_where_to_start")
}

/// Python: bot/memory/repeat.py:3563 `a_more_ask(text)`
pub fn a_more_ask(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:a_more_ask")
}

/// Python: bot/memory/repeat.py:3588 `_places_to_cut(head)`
pub fn _places_to_cut(head: &str) -> Value {
    crate::unported!("bot.memory.repeat:_places_to_cut")
}

/// Python: bot/memory/repeat.py:3609 `_inside_a_bracket(text, at)`
pub fn _inside_a_bracket(text: &str, at: i64) -> bool {
    crate::unported!("bot.memory.repeat:_inside_a_bracket")
}

/// Python: bot/memory/repeat.py:3633 `_a_list_item(part)`
pub fn _a_list_item(part: &str) -> String {
    crate::unported!("bot.memory.repeat:_a_list_item")
}

/// Python: bot/memory/repeat.py:3670 `_pieces(part)`
pub fn _pieces(part: &str) -> Vec<String> {
    crate::unported!("bot.memory.repeat:_pieces")
}

/// Python: bot/memory/repeat.py:3722 `_the_tail_asks_a_question(part, marks, index, found)` (never called on the trace corpus)
pub fn _the_tail_asks_a_question(part: &str, marks: &Value, index: i64, found: &Value) -> bool {
    crate::unported!("bot.memory.repeat:_the_tail_asks_a_question")
}

/// Python: bot/memory/repeat.py:3728 `_the_colon_tail_kept(part, marks, index, start, found)`
pub fn _the_colon_tail_kept(part: &str, marks: &Value, index: i64, start: i64, found: &Value) -> Value {
    crate::unported!("bot.memory.repeat:_the_colon_tail_kept")
}

/// Python: bot/memory/repeat.py:3773 `_opens_on_a_verb(text)`
pub fn _opens_on_a_verb(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:_opens_on_a_verb")
}

/// Python: bot/memory/repeat.py:3799 `_the_steps(said)`
pub fn _the_steps(said: &str) -> Option<String> {
    crate::unported!("bot.memory.repeat:_the_steps")
}

/// Python: bot/memory/repeat.py:3903 `_the_next_step(said, memory)`
pub fn _the_next_step(said: &str, memory: &Memory) -> Option<String> {
    crate::unported!("bot.memory.repeat:_the_next_step")
}

/// Python: bot/memory/repeat.py:3927 `_the_first_step(said)`
pub fn _the_first_step(said: &str) -> String {
    crate::unported!("bot.memory.repeat:_the_first_step")
}

/// Python: bot/memory/repeat.py:3951 `_a_new_work(text, asked)`
pub fn _a_new_work(text: &str, asked: &str) -> bool {
    crate::unported!("bot.memory.repeat:_a_new_work")
}

/// Python: bot/memory/repeat.py:3970 `asks_which_part(text)`
pub fn asks_which_part(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:asks_which_part")
}

/// Python: bot/memory/repeat.py:3985 `_leads_for(text)`
pub fn _leads_for(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.repeat:_leads_for")
}

/// Python: bot/memory/repeat.py:3990 `_a_lead_for(leads, step, memory, text)`
pub fn _a_lead_for(leads: &[String], step: &str, memory: &Memory, text: &str) -> String {
    crate::unported!("bot.memory.repeat:_a_lead_for")
}

/// Python: bot/memory/repeat.py:4007 `the_move(text, said, memory)`
pub fn the_move(text: &str, said: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.memory.repeat:the_move")
}

/// Python: bot/memory/repeat.py:4038 `_they_say_they_tried(step, memory)`
pub fn _they_say_they_tried(step: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:_they_say_they_tried")
}

/// Python: bot/memory/repeat.py:4120 `the_figure_asked(text, memory)`
pub fn the_figure_asked(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.repeat:the_figure_asked")
}

/// Python: bot/memory/repeat.py:4228 `_the_keys_of(said, memory)`
pub fn _the_keys_of(said: &str, memory: &Memory) -> Vec<String> {
    crate::unported!("bot.memory.repeat:_the_keys_of")
}

/// Python: bot/memory/repeat.py:4255 `answers_nothing(reply)`
pub fn answers_nothing(reply: &str) -> bool {
    crate::unported!("bot.memory.repeat:answers_nothing")
}

/// Python: bot/memory/repeat.py:4280 `the_advice_was_to_get_seen(memory)`
pub fn the_advice_was_to_get_seen(memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:the_advice_was_to_get_seen")
}

/// Python: bot/memory/repeat.py:4290 `the_part_asked(text, memory)`
pub fn the_part_asked(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.repeat:the_part_asked")
}

/// Python: bot/memory/repeat.py:4534 `_whether_to(clean)`
pub fn _whether_to(clean: &str) -> String {
    crate::unported!("bot.memory.repeat:_whether_to")
}

/// Python: bot/memory/repeat.py:4576 `_a_stem(word)`
pub fn _a_stem(word: &str) -> String {
    crate::unported!("bot.memory.repeat:_a_stem")
}

/// Python: bot/memory/repeat.py:4603 `_the_chat_is_still_on(said, memory)` (never called on the trace corpus)
pub fn _the_chat_is_still_on(said: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:_the_chat_is_still_on")
}

/// Python: bot/memory/repeat.py:4632 `_where_the_rest_is(said, memory, turn, ask='')`
pub fn _where_the_rest_is(said: &str, memory: &Memory, turn: &Value, ask: &str) -> String {
    crate::unported!("bot.memory.repeat:_where_the_rest_is")
}

/// Python: bot/memory/repeat.py:4673 `what_it_knows(text, memory)`
pub fn what_it_knows(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.repeat:what_it_knows")
}

/// Python: bot/memory/repeat.py:4976 `the_plain_line(line)`
pub fn the_plain_line(line: &str) -> String {
    crate::unported!("bot.memory.repeat:the_plain_line")
}

/// Python: bot/memory/repeat.py:5078 `how_long_its_been(text)`
pub fn how_long_its_been(text: &str) -> bool {
    crate::unported!("bot.memory.repeat:how_long_its_been")
}

/// Python: bot/memory/repeat.py:5107 `_a_line_that_corrects(line)` (never called on the trace corpus)
pub fn _a_line_that_corrects(line: &str) -> bool {
    crate::unported!("bot.memory.repeat:_a_line_that_corrects")
}

/// Python: bot/memory/repeat.py:5165 `_a_list_of_ages(line)` (never called on the trace corpus)
pub fn _a_list_of_ages(line: &str) -> bool {
    crate::unported!("bot.memory.repeat:_a_list_of_ages")
}

/// Python: bot/memory/repeat.py:5170 `_something_gone(lines, at, words)` (never called on the trace corpus)
pub fn _something_gone(lines: &[String], at: i64, words: &[String]) -> bool {
    crate::unported!("bot.memory.repeat:_something_gone")
}

/// Python: bot/memory/repeat.py:5217 `_the_ask_gave_that_figure(number, clean)` (never called on the trace corpus)
pub fn _the_ask_gave_that_figure(number: &Value, clean: &str) -> bool {
    crate::unported!("bot.memory.repeat:_the_ask_gave_that_figure")
}

/// Python: bot/memory/repeat.py:5236 `_a_count_of_one_with_a_plural(number, noun)` (never called on the trace corpus)
pub fn _a_count_of_one_with_a_plural(number: &Value, noun: &str) -> bool {
    crate::unported!("bot.memory.repeat:_a_count_of_one_with_a_plural")
}

/// Python: bot/memory/repeat.py:5246 `_the_thing_counted(clean, said)` (never called on the trace corpus)
pub fn _the_thing_counted(clean: &str, said: &str) -> Value {
    crate::unported!("bot.memory.repeat:_the_thing_counted")
}

/// Python: bot/memory/repeat.py:5269 `_figures_in(line)`
pub fn _figures_in(line: &str) -> Vec<String> {
    crate::unported!("bot.memory.repeat:_figures_in")
}

/// Python: bot/memory/repeat.py:5356 `_money_paid_for_a_thing(clean, line)` (never called on the trace corpus)
pub fn _money_paid_for_a_thing(clean: &str, line: &str) -> bool {
    crate::unported!("bot.memory.repeat:_money_paid_for_a_thing")
}

/// Python: bot/memory/repeat.py:5363 `_one_figure(found, kind)`
pub fn _one_figure(found: &[String], kind: Option<&str>) -> Vec<String> {
    crate::unported!("bot.memory.repeat:_one_figure")
}

/// Python: bot/memory/repeat.py:5374 `_a_bare_count(figure)` (never called on the trace corpus)
pub fn _a_bare_count(figure: &Value) -> bool {
    crate::unported!("bot.memory.repeat:_a_bare_count")
}

/// Python: bot/memory/repeat.py:5385 `_the_same_word(one, other)`
pub fn _the_same_word(one: &str, other: &str) -> bool {
    crate::unported!("bot.memory.repeat:_the_same_word")
}

/// Python: bot/memory/repeat.py:5409 `_the_corrected(lines, at, kind, found)` (never called on the trace corpus)
pub fn _the_corrected(lines: &[String], at: i64, kind: &str, found: &Value) -> Value {
    crate::unported!("bot.memory.repeat:_the_corrected")
}

/// Python: bot/memory/repeat.py:5443 `_the_length_they_mean(clean, lines, spoken=())` (never called on the trace corpus)
pub fn _the_length_they_mean(clean: &str, lines: &[String], spoken: &Value) -> Value {
    crate::unported!("bot.memory.repeat:_the_length_they_mean")
}

/// Python: bot/memory/repeat.py:5498 `_the_figure_of_a_goal(lines, kind, wanted=())` (never called on the trace corpus)
pub fn _the_figure_of_a_goal(lines: &[String], kind: &str, wanted: &Value) -> Value {
    crate::unported!("bot.memory.repeat:_the_figure_of_a_goal")
}

/// Python: bot/memory/repeat.py:5525 `_the_figure_counts_the_thing(text, figure)` (never called on the trace corpus)
pub fn _the_figure_counts_the_thing(text: &str, figure: &Value) -> bool {
    crate::unported!("bot.memory.repeat:_the_figure_counts_the_thing")
}

/// Python: bot/memory/repeat.py:5570 `a_narrowing_and_its_line(line, memory)` (never called on the trace corpus)
pub fn a_narrowing_and_its_line(line: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.repeat:a_narrowing_and_its_line")
}

/// Python: bot/memory/repeat.py:5604 `_the_turn_that_names_it(text, memory)` (never called on the trace corpus)
pub fn _the_turn_that_names_it(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.repeat:_the_turn_that_names_it")
}

/// Python: bot/memory/repeat.py:5623 `the_figure_they_gave(text, memory)`
pub fn the_figure_they_gave(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.repeat:the_figure_they_gave")
}

/// Python: bot/memory/repeat.py:5863 `_the_store_keeps_this_figure(clean, memory)`
pub fn _the_store_keeps_this_figure(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.repeat:_the_store_keeps_this_figure")
}

/// Python: bot/memory/repeat.py:5926 `_the_figure_beside_the_thing(figures, words)` (never called on the trace corpus)
pub fn _the_figure_beside_the_thing(figures: &Value, words: &[String]) -> Value {
    crate::unported!("bot.memory.repeat:_the_figure_beside_the_thing")
}

/// Python: bot/memory/repeat.py:5942 `_every_figure_the_ask_names(wanted, line, kind)` (never called on the trace corpus)
pub fn _every_figure_the_ask_names(wanted: &Value, line: &str, kind: &str) -> String {
    crate::unported!("bot.memory.repeat:_every_figure_the_ask_names")
}

/// Python: bot/memory/repeat.py:5972 `asked_back(text, memory)`
pub fn asked_back(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.repeat:asked_back")
}

/// Python: bot/memory/repeat.py:6009 `carries_the_figure(figure, reply)` (never called on the trace corpus)
pub fn carries_the_figure(figure: &Value, reply: &str) -> bool {
    crate::unported!("bot.memory.repeat:carries_the_figure")
}

/// Python: bot/memory/repeat.py:6054 `_the_same_words(one, other)`
pub fn _the_same_words(one: &str, other: &str) -> bool {
    crate::unported!("bot.memory.repeat:_the_same_words")
}

/// Python: bot/memory/repeat.py:6061 `_shortened(said)` (never called on the trace corpus)
pub fn _shortened(said: &str) -> Value {
    crate::unported!("bot.memory.repeat:_shortened")
}

/// Python: bot/memory/repeat.py:6074 `_the_answer_again(said, memory)` (never called on the trace corpus)
pub fn _the_answer_again(said: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.repeat:_the_answer_again")
}

/// Python: bot/memory/repeat.py:6090 `the_same_question_again(text, memory)`
pub fn the_same_question_again(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.repeat:the_same_question_again")
}

