//! Port of `bot/memory/price.py`.
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

/// Python: bot/memory/price.py:81 `_money(value)` (never called on the trace corpus)
pub fn _money(value: &Value) -> String {
    crate::unported!("bot.memory.price:_money")
}

/// Python: bot/memory/price.py:88 `_a_value(token)`
pub fn _a_value(token: &str) -> f64 {
    crate::unported!("bot.memory.price:_a_value")
}

/// Python: bot/memory/price.py:96 `_the_money_said_back(said, text)`
pub fn _the_money_said_back(said: &str, text: &str) -> String {
    crate::unported!("bot.memory.price:_the_money_said_back")
}

/// Python: bot/memory/price.py:107 `_the_price_said(value)` (never called on the trace corpus)
pub fn _the_price_said(value: &Value) -> String {
    crate::unported!("bot.memory.price:_the_price_said")
}

/// Python: bot/memory/price.py:124 `_the_figure_in_the_range_s_style(mine, said, range_said)` (never called on the trace corpus)
pub fn _the_figure_in_the_range_s_style(mine: &Memory, said: &str, range_said: &str) -> Value {
    crate::unported!("bot.memory.price:_the_figure_in_the_range_s_style")
}

/// Python: bot/memory/price.py:137 `_number_tokens(text)`
pub fn _number_tokens(text: &str) -> Value {
    crate::unported!("bot.memory.price:_number_tokens")
}

/// Python: bot/memory/price.py:142 `_prices_in(text)`
pub fn _prices_in(text: &str) -> Value {
    crate::unported!("bot.memory.price:_prices_in")
}

/// Python: bot/memory/price.py:172 `_a_price_at_a_rate(text)` (never called on the trace corpus)
pub fn _a_price_at_a_rate(text: &str) -> bool {
    crate::unported!("bot.memory.price:_a_price_at_a_rate")
}

/// Python: bot/memory/price.py:185 `_price_ranges_in(text, period=None)`
pub fn _price_ranges_in(text: &str, period: &str) -> Vec<String> {
    crate::unported!("bot.memory.price:_price_ranges_in")
}

/// Python: bot/memory/price.py:233 `_the_price_they_gave(text, memory)`
pub fn _the_price_they_gave(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.price:_the_price_they_gave")
}

/// Python: bot/memory/price.py:260 `_a_bare_price_in_the_thread(text, memory)`
pub fn _a_bare_price_in_the_thread(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.price:_a_bare_price_in_the_thread")
}

/// Python: bot/memory/price.py:279 `_answers_in(turns)`
pub fn _answers_in(turns: &Value) -> Vec<String> {
    crate::unported!("bot.memory.price:_answers_in")
}

/// Python: bot/memory/price.py:291 `_the_range_that_is_about_it(answers, price, period=None)`
pub fn _the_range_that_is_about_it(answers: &[String], price: f64, period: &str) {
    crate::unported!("bot.memory.price:_the_range_that_is_about_it")
}

/// Python: bot/memory/price.py:375 `_the_between_verdict(mine, said, below, above, memory)` (never called on the trace corpus)
pub fn _the_between_verdict(mine: &Memory, said: &str, below: &Value, above: &Value, memory: &Memory) -> Option<String> {
    crate::unported!("bot.memory.price:_the_between_verdict")
}

/// Python: bot/memory/price.py:388 `_the_verdict(mine, said, low, high, spoken, memory, own=False)` (never called on the trace corpus)
pub fn _the_verdict(mine: &Memory, said: &str, low: &str, high: &Value, spoken: &Value, memory: &Memory, own: &str) -> Option<String> {
    crate::unported!("bot.memory.price:_the_verdict")
}

/// Python: bot/memory/price.py:444 `_the_thing_priced(text)`
pub fn _the_thing_priced(text: &str) -> String {
    crate::unported!("bot.memory.price:_the_thing_priced")
}

/// Python: bot/memory/price.py:452 `_the_price_they_mean(text, memory)`
pub fn _the_price_they_mean(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.price:_the_price_they_mean")
}

/// Python: bot/memory/price.py:460 `_the_period_they_said(text, memory)`
pub fn _the_period_they_said(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.price:_the_period_they_said")
}

/// Python: bot/memory/price.py:495 `_a_price_ask(text)`
pub fn _a_price_ask(text: &str) {
    crate::unported!("bot.memory.price:_a_price_ask")
}

/// Python: bot/memory/price.py:514 `_a_rate_is_all_the_answer_names(answers)` (never called on the trace corpus)
pub fn _a_rate_is_all_the_answer_names(answers: &Value) -> bool {
    crate::unported!("bot.memory.price:_a_rate_is_all_the_answer_names")
}

/// Python: bot/memory/price.py:523 `_a_second_quote_is_the_test(memory)` (never called on the trace corpus)
pub fn _a_second_quote_is_the_test(memory: &Memory) -> Value {
    crate::unported!("bot.memory.price:_a_second_quote_is_the_test")
}

/// Python: bot/memory/price.py:529 `judged(text, memory)`
pub fn judged(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.price:judged")
}

/// Python: bot/memory/price.py:557 `judged_with_the_card(text, answer, memory)`
pub fn judged_with_the_card(text: &str, answer: Option<&str>, memory: &Memory) {
    crate::unported!("bot.memory.price:judged_with_the_card")
}

/// Python: bot/memory/price.py:630 `_the_ranges_that_are_not_rates(answer, ranges)`
pub fn _the_ranges_that_are_not_rates(answer: &str, ranges: &[String]) -> Vec<String> {
    crate::unported!("bot.memory.price:_the_ranges_that_are_not_rates")
}

/// Python: bot/memory/price.py:646 `_the_verdict_against_a_card(mine, said, answer, memory, definite_only=False)` (never called on the trace corpus)
pub fn _the_verdict_against_a_card(mine: &Memory, said: &str, answer: &str, memory: &Memory, definite_only: &Value) -> Option<String> {
    crate::unported!("bot.memory.price:_the_verdict_against_a_card")
}

