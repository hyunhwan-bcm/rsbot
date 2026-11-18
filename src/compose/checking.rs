//! Port of `bot/compose/checking.py`.
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

/// Python: bot/compose/checking.py:76 `_a_named_country_in(text)`
pub fn _a_named_country_in(text: &str) -> bool {
    crate::unported!("bot.compose.checking:_a_named_country_in")
}

/// Python: bot/compose/checking.py:309 `_a_time_of_day(said)`
pub fn _a_time_of_day(said: &str) {
    crate::unported!("bot.compose.checking:_a_time_of_day")
}

/// Python: bot/compose/checking.py:543 `_the_route_ends(said)` (never called on the trace corpus)
pub fn _the_route_ends(said: &str) -> Value {
    crate::unported!("bot.compose.checking:_the_route_ends")
}

/// Python: bot/compose/checking.py:575 `_without_a_qualifier(run)` (never called on the trace corpus)
pub fn _without_a_qualifier(run: &Value) -> Value {
    crate::unported!("bot.compose.checking:_without_a_qualifier")
}

/// Python: bot/compose/checking.py:586 `_a_place_name(run)` (never called on the trace corpus)
pub fn _a_place_name(run: &Value) -> String {
    crate::unported!("bot.compose.checking:_a_place_name")
}

/// Python: bot/compose/checking.py:722 `_said(text)`
pub fn _said(text: &str) -> String {
    crate::unported!("bot.compose.checking:_said")
}

/// Python: bot/compose/checking.py:726 `_the_line_before(memory)`
pub fn _the_line_before(memory: &Memory) -> String {
    crate::unported!("bot.compose.checking:_the_line_before")
}

/// Python: bot/compose/checking.py:742 `_the_answers_before(memory, count=4)`
pub fn _the_answers_before(memory: &Memory, count: i64) -> Vec<String> {
    crate::unported!("bot.compose.checking:_the_answers_before")
}

/// Python: bot/compose/checking.py:756 `_the_checks_in(answer)`
pub fn _the_checks_in(answer: &str) -> String {
    crate::unported!("bot.compose.checking:_the_checks_in")
}

/// Python: bot/compose/checking.py:783 `_about_a_check(said, understood, memory)`
pub fn _about_a_check(said: &str, understood: &Understood, memory: &Memory) {
    crate::unported!("bot.compose.checking:_about_a_check")
}

/// Python: bot/compose/checking.py:804 `_named_service(said)`
pub fn _named_service(said: &str) -> Vec<String> {
    crate::unported!("bot.compose.checking:_named_service")
}

/// Python: bot/compose/checking.py:817 `_about_a_service(said, understood)`
pub fn _about_a_service(said: &str, understood: &Understood) {
    crate::unported!("bot.compose.checking:_about_a_service")
}

/// Python: bot/compose/checking.py:838 `_about_a_shop(said)`
pub fn _about_a_shop(said: &str) -> String {
    crate::unported!("bot.compose.checking:_about_a_shop")
}

/// Python: bot/compose/checking.py:857 `_the_mode_named(mode)` (never called on the trace corpus)
pub fn _the_mode_named(mode: &Value) -> Value {
    crate::unported!("bot.compose.checking:_the_mode_named")
}

/// Python: bot/compose/checking.py:870 `_about_a_journey(said)`
pub fn _about_a_journey(said: &str) {
    crate::unported!("bot.compose.checking:_about_a_journey")
}

/// Python: bot/compose/checking.py:963 `_a_country_not_home(text)` (never called on the trace corpus)
pub fn _a_country_not_home(text: &str) -> Value {
    crate::unported!("bot.compose.checking:_a_country_not_home")
}

/// Python: bot/compose/checking.py:975 `_about_the_subject_named(said, understood, before='')`
pub fn _about_the_subject_named(said: &str, understood: &Understood, before: &str) {
    crate::unported!("bot.compose.checking:_about_the_subject_named")
}

/// Python: bot/compose/checking.py:1032 `_about_a_pick(said, before='')`
pub fn _about_a_pick(said: &str, before: &str) {
    crate::unported!("bot.compose.checking:_about_a_pick")
}

/// Python: bot/compose/checking.py:1058 `_the_home_town(memory)`
pub fn _the_home_town(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.compose.checking:_the_home_town")
}

/// Python: bot/compose/checking.py:1096 `_the_lines_of_this_question(said, memory)` (never called on the trace corpus)
pub fn _the_lines_of_this_question(said: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.checking:_the_lines_of_this_question")
}

/// Python: bot/compose/checking.py:1110 `_the_move_was_to_their_own_town(said, memory)`
pub fn _the_move_was_to_their_own_town(said: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.checking:_the_move_was_to_their_own_town")
}

/// Python: bot/compose/checking.py:1140 `_a_day_out(said, before='')`
pub fn _a_day_out(said: &str, before: &str) -> bool {
    crate::unported!("bot.compose.checking:_a_day_out")
}

/// Python: bot/compose/checking.py:1162 `_about_the_rules(said, understood, before='', memory=None)`
pub fn _about_the_rules(said: &str, understood: &Understood, before: &str, memory: &Memory) {
    crate::unported!("bot.compose.checking:_about_the_rules")
}

/// Python: bot/compose/checking.py:1234 `_a_form(said, understood)`
pub fn _a_form(said: &str, understood: &Understood) {
    crate::unported!("bot.compose.checking:_a_form")
}

/// Python: bot/compose/checking.py:1245 `_a_choice(said, understood)`
pub fn _a_choice(said: &str, understood: &Understood) {
    crate::unported!("bot.compose.checking:_a_choice")
}

/// Python: bot/compose/checking.py:1257 `_a_shortlist(said, understood)`
pub fn _a_shortlist(said: &str, understood: &Understood) {
    crate::unported!("bot.compose.checking:_a_shortlist")
}

/// Python: bot/compose/checking.py:1264 `_about_the_sky(said, understood)`
pub fn _about_the_sky(said: &str, understood: &Understood) {
    crate::unported!("bot.compose.checking:_about_the_sky")
}

/// Python: bot/compose/checking.py:1340 `the_checks_already_named(text, memory=None)`
pub fn the_checks_already_named(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.checking:the_checks_already_named")
}

/// Python: bot/compose/checking.py:1360 `answer_for(text, memory=None)`
pub fn answer_for(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.checking:answer_for")
}

