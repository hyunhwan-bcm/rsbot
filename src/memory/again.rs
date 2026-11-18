//! Port of `bot/memory/again.py`.
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

/// Python: bot/memory/again.py:166 `the_name_the_answer_says(answer, detail, card='')`
pub fn the_name_the_answer_says(answer: &str, detail: &str, card: &str) -> String {
    crate::unported!("bot.memory.again:the_name_the_answer_says")
}

/// Python: bot/memory/again.py:212 `as_a_phrase(text)`
pub fn as_a_phrase(text: &str) {
    crate::unported!("bot.memory.again:as_a_phrase")
}

/// Python: bot/memory/again.py:238 `_a_phrase_of(text)`
pub fn _a_phrase_of(text: &str) {
    crate::unported!("bot.memory.again:_a_phrase_of")
}

/// Python: bot/memory/again.py:286 `_says_i(text)` (never called on the trace corpus)
pub fn _says_i(text: &str) -> bool {
    crate::unported!("bot.memory.again:_says_i")
}

/// Python: bot/memory/again.py:292 `_for_the_person(text)` (never called on the trace corpus)
pub fn _for_the_person(text: &str) -> String {
    crate::unported!("bot.memory.again:_for_the_person")
}

/// Python: bot/memory/again.py:309 `the_same_thing_underneath(text, reply, times, memory=None)`
pub fn the_same_thing_underneath(text: &str, reply: &str, times: i64, memory: &Memory) -> String {
    crate::unported!("bot.memory.again:the_same_thing_underneath")
}

/// Python: bot/memory/again.py:342 `_lead(frame)` (never called on the trace corpus)
pub fn _lead(frame: &Value) -> Value {
    crate::unported!("bot.memory.again:_lead")
}

/// Python: bot/memory/again.py:382 `unframed(reply)`
pub fn unframed(reply: &str) -> Option<String> {
    crate::unported!("bot.memory.again:unframed")
}

/// Python: bot/memory/again.py:392 `_flat(text)`
pub fn _flat(text: &str) -> String {
    crate::unported!("bot.memory.again:_flat")
}

/// Python: bot/memory/again.py:397 `the_cause_said_again(reply, memory=None)`
pub fn the_cause_said_again(reply: &str, memory: &Memory) {
    crate::unported!("bot.memory.again:the_cause_said_again")
}

/// Python: bot/memory/again.py:444 `_a_card_behind(memory)`
pub fn _a_card_behind(memory: &Memory) -> String {
    crate::unported!("bot.memory.again:_a_card_behind")
}

/// Python: bot/memory/again.py:470 `_led_an_earlier_answer(sentence, memory)` (never called on the trace corpus)
pub fn _led_an_earlier_answer(sentence: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.again:_led_an_earlier_answer")
}

/// Python: bot/memory/again.py:488 `_the_answer_in_hand(reply, memory)`
pub fn _the_answer_in_hand(reply: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.again:_the_answer_in_hand")
}

/// Python: bot/memory/again.py:506 `_short(reply, memory=None)`
pub fn _short(reply: &str, memory: Option<&Memory>) -> String {
    crate::unported!("bot.memory.again:_short")
}

/// Python: bot/memory/again.py:525 `_the_first_sentence(reply, memory=None)` (never called on the trace corpus)
pub fn _the_first_sentence(reply: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.again:_the_first_sentence")
}

/// Python: bot/memory/again.py:539 `_carries_a_lead_in(mine)`
pub fn _carries_a_lead_in(mine: &str) -> bool {
    crate::unported!("bot.memory.again:_carries_a_lead_in")
}

/// Python: bot/memory/again.py:552 `carries_a_lead_in(reply)`
pub fn carries_a_lead_in(reply: &str) -> bool {
    crate::unported!("bot.memory.again:carries_a_lead_in")
}

/// Python: bot/memory/again.py:561 `_times_on_the_screen(mine, memory)`
pub fn _times_on_the_screen(mine: &str, memory: &Memory) -> i64 {
    crate::unported!("bot.memory.again:_times_on_the_screen")
}

/// Python: bot/memory/again.py:588 `_said_earlier(mine, memory)`
pub fn _said_earlier(mine: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.again:_said_earlier")
}

/// Python: bot/memory/again.py:600 `_said_earlier_text(mine, memory)`
pub fn _said_earlier_text(mine: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.again:_said_earlier_text")
}

/// Python: bot/memory/again.py:621 `_names_something_the_answer_says(text, mine)` (never called on the trace corpus)
pub fn _names_something_the_answer_says(text: &str, mine: &Memory) -> bool {
    crate::unported!("bot.memory.again:_names_something_the_answer_says")
}

/// Python: bot/memory/again.py:643 `_returned_thing(memory, text)`
pub fn _returned_thing(memory: &Memory, text: &str) -> String {
    crate::unported!("bot.memory.again:_returned_thing")
}

/// Python: bot/memory/again.py:663 `_the_line_asks(memory, text)` (never called on the trace corpus)
pub fn _the_line_asks(memory: &Memory, text: &str) -> bool {
    crate::unported!("bot.memory.again:_the_line_asks")
}

/// Python: bot/memory/again.py:681 `_adds_nothing_the_answer_did_not_say(thing, text, said, memory)` (never called on the trace corpus)
pub fn _adds_nothing_the_answer_did_not_say(thing: &str, text: &str, said: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.again:_adds_nothing_the_answer_did_not_say")
}

/// Python: bot/memory/again.py:718 `_places()` (never called on the trace corpus)
pub fn _places() -> Value {
    crate::unported!("bot.memory.again:_places")
}

/// Python: bot/memory/again.py:727 `_as_a_name(thing)` (never called on the trace corpus)
pub fn _as_a_name(thing: &str) -> String {
    crate::unported!("bot.memory.again:_as_a_name")
}

/// Python: bot/memory/again.py:740 `_without_the_thing_in_front(short, thing)` (never called on the trace corpus)
pub fn _without_the_thing_in_front(short: &Value, thing: &str) -> Value {
    crate::unported!("bot.memory.again:_without_the_thing_in_front")
}

/// Python: bot/memory/again.py:757 `_said_recently(mine, memory, look=3)`
pub fn _said_recently(mine: &str, memory: &Memory, look: i64) -> i64 {
    crate::unported!("bot.memory.again:_said_recently")
}

/// Python: bot/memory/again.py:782 `_says_it_came_back(text)`
pub fn _says_it_came_back(text: &str) -> bool {
    crate::unported!("bot.memory.again:_says_it_came_back")
}

/// Python: bot/memory/again.py:822 `_a_figure_asked_that_the_answer_does_not_hold(text, reply)`
pub fn _a_figure_asked_that_the_answer_does_not_hold(text: &str, reply: &str) -> bool {
    crate::unported!("bot.memory.again:_a_figure_asked_that_the_answer_does_not_hold")
}

/// Python: bot/memory/again.py:837 `_asks_for_a_detail(text)`
pub fn _asks_for_a_detail(text: &str) -> bool {
    crate::unported!("bot.memory.again:_asks_for_a_detail")
}

/// Python: bot/memory/again.py:870 `_a_report(text)`
pub fn _a_report(text: &str) -> bool {
    crate::unported!("bot.memory.again:_a_report")
}

/// Python: bot/memory/again.py:892 `_has_a_script(reply)` (never called on the trace corpus)
pub fn _has_a_script(reply: &str) -> bool {
    crate::unported!("bot.memory.again:_has_a_script")
}

/// Python: bot/memory/again.py:897 `_the_line_that_got_it(mine, memory, look=3)`
pub fn _the_line_that_got_it(mine: &str, memory: &Memory, look: i64) -> String {
    crate::unported!("bot.memory.again:_the_line_that_got_it")
}

/// Python: bot/memory/again.py:908 `_said_and_more(mine, memory)`
pub fn _said_and_more(mine: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.again:_said_and_more")
}

/// Python: bot/memory/again.py:949 `already_had(reply, memory)`
pub fn already_had(reply: &str, memory: &Memory) {
    crate::unported!("bot.memory.again:already_had")
}

/// Python: bot/memory/again.py:970 `_as_said(text)`
pub fn _as_said(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.again:_as_said")
}

/// Python: bot/memory/again.py:981 `_content_words_in_order(text)`
pub fn _content_words_in_order(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.again:_content_words_in_order")
}

/// Python: bot/memory/again.py:997 `_the_run_they_said_both_times(text, earlier)` (never called on the trace corpus)
pub fn _the_run_they_said_both_times(text: &str, earlier: &Value) -> String {
    crate::unported!("bot.memory.again:_the_run_they_said_both_times")
}

/// Python: bot/memory/again.py:1024 `_the_same_question_asked_again(mine, text, memory)`
pub fn _the_same_question_asked_again(mine: &str, text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.again:_the_same_question_asked_again")
}

/// Python: bot/memory/again.py:1057 `_the_subject_on_the_screen(memory)` (never called on the trace corpus)
pub fn _the_subject_on_the_screen(memory: &Memory) -> Value {
    crate::unported!("bot.memory.again:_the_subject_on_the_screen")
}

/// Python: bot/memory/again.py:1078 `_as_your(subject)` (never called on the trace corpus)
pub fn _as_your(subject: &str) -> Value {
    crate::unported!("bot.memory.again:_as_your")
}

/// Python: bot/memory/again.py:1085 `replied(respond)` (never called on the trace corpus)
pub fn replied(respond: &Value) -> Value {
    crate::unported!("bot.memory.again:replied")
}

/// Python: bot/memory/again.py:1122 `instead(reply, text, memory)`
pub fn instead(reply: &str, text: &str, memory: &Memory) {
    crate::unported!("bot.memory.again:instead")
}

/// Python: bot/memory/again.py:1184 `a_thing_of_the_answer(reply, text, memory)` (never called on the trace corpus)
pub fn a_thing_of_the_answer(reply: &str, text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.again:a_thing_of_the_answer")
}

/// Python: bot/memory/again.py:1221 `_another_stand_down(reply, memory)`
pub fn _another_stand_down(reply: &str, memory: &Memory) {
    crate::unported!("bot.memory.again:_another_stand_down")
}

/// Python: bot/memory/again.py:1245 `_framed_as_a_repeat(reply, mine, memory, times, text='')`
pub fn _framed_as_a_repeat(reply: &str, mine: &str, memory: &Memory, times: i64, text: &str) -> Option<String> {
    crate::unported!("bot.memory.again:_framed_as_a_repeat")
}

