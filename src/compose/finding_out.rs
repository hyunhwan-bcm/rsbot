//! Port of `bot/compose/finding_out.py`.
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

/// Python: bot/compose/finding_out.py:34 `_has(line, phrases)`
pub fn _has(line: &str, phrases: &[String]) -> bool {
    crate::unported!("bot.compose.finding_out:_has")
}

/// Python: bot/compose/finding_out.py:46 `_words(line)`
pub fn _words(line: &str) -> Vec<String> {
    crate::unported!("bot.compose.finding_out:_words")
}

/// Python: bot/compose/finding_out.py:50 `_names_one_of(line, phrases)`
pub fn _names_one_of(line: &str, phrases: &[String]) -> bool {
    crate::unported!("bot.compose.finding_out:_names_one_of")
}

/// Python: bot/compose/finding_out.py:491 `_the_trade(line)`
pub fn _the_trade(line: &str) -> String {
    crate::unported!("bot.compose.finding_out:_the_trade")
}

/// Python: bot/compose/finding_out.py:499 `_the_thing(line)`
pub fn _the_thing(line: &str) -> String {
    crate::unported!("bot.compose.finding_out:_the_thing")
}

/// Python: bot/compose/finding_out.py:520 `_trade_route(line, ctx)`
pub fn _trade_route(line: &str, ctx: &Value) {
    crate::unported!("bot.compose.finding_out:_trade_route")
}

/// Python: bot/compose/finding_out.py:532 `_buy_route(line, ctx)`
pub fn _buy_route(line: &str, ctx: &Value) {
    crate::unported!("bot.compose.finding_out:_buy_route")
}

/// Python: bot/compose/finding_out.py:547 `_the_service(line, pointed='')`
pub fn _the_service(line: &str, pointed: &str) {
    crate::unported!("bot.compose.finding_out:_the_service")
}

/// Python: bot/compose/finding_out.py:593 `_service_route(line, ctx)`
pub fn _service_route(line: &str, ctx: &Value) {
    crate::unported!("bot.compose.finding_out:_service_route")
}

/// Python: bot/compose/finding_out.py:602 `_about_a_place(line, ctx)`
pub fn _about_a_place(line: &str, ctx: &Value) -> bool {
    crate::unported!("bot.compose.finding_out:_about_a_place")
}

/// Python: bot/compose/finding_out.py:618 `_the_picked_thing(line)`
pub fn _the_picked_thing(line: &str) -> String {
    crate::unported!("bot.compose.finding_out:_the_picked_thing")
}

/// Python: bot/compose/finding_out.py:636 `_pick_route(line, ctx)`
pub fn _pick_route(line: &str, ctx: &Value) {
    crate::unported!("bot.compose.finding_out:_pick_route")
}

/// Python: bot/compose/finding_out.py:661 `_how_many_route(line, ctx)`
pub fn _how_many_route(line: &str, ctx: &Value) {
    crate::unported!("bot.compose.finding_out:_how_many_route")
}

/// Python: bot/compose/finding_out.py:669 `_place_route(line, ctx)`
pub fn _place_route(line: &str, ctx: &Value) {
    crate::unported!("bot.compose.finding_out:_place_route")
}

/// Python: bot/compose/finding_out.py:683 `_route_route(line, ctx)`
pub fn _route_route(line: &str, ctx: &Value) {
    crate::unported!("bot.compose.finding_out:_route_route")
}

/// Python: bot/compose/finding_out.py:729 `_open_chemist_route(line, ctx)`
pub fn _open_chemist_route(line: &str, ctx: &Value) {
    crate::unported!("bot.compose.finding_out:_open_chemist_route")
}

/// Python: bot/compose/finding_out.py:741 `_who_route(line, ctx)`
pub fn _who_route(line: &str, ctx: &Value) {
    crate::unported!("bot.compose.finding_out:_who_route")
}

/// Python: bot/compose/finding_out.py:755 `_tide_route(line, ctx)`
pub fn _tide_route(line: &str, ctx: &Value) {
    crate::unported!("bot.compose.finding_out:_tide_route")
}

/// Python: bot/compose/finding_out.py:770 `_figure_route(line, ctx)`
pub fn _figure_route(line: &str, ctx: &Value) {
    crate::unported!("bot.compose.finding_out:_figure_route")
}

/// Python: bot/compose/finding_out.py:808 `_record_route(line, ctx)`
pub fn _record_route(line: &str, ctx: &Value) {
    crate::unported!("bot.compose.finding_out:_record_route")
}

/// Python: bot/compose/finding_out.py:839 `_the_reading(understood)`
pub fn _the_reading(understood: &Understood) -> Value {
    crate::unported!("bot.compose.finding_out:_the_reading")
}

/// Python: bot/compose/finding_out.py:964 `the_wrecked_form_reply(line)`
pub fn the_wrecked_form_reply(line: &str) {
    crate::unported!("bot.compose.finding_out:the_wrecked_form_reply")
}

/// Python: bot/compose/finding_out.py:997 `for_the_ask(clean, repaired='', understood=None)`
pub fn for_the_ask(clean: &str, repaired: &str, understood: &Understood) -> Option<Vec<String>> {
    crate::unported!("bot.compose.finding_out:for_the_ask")
}

