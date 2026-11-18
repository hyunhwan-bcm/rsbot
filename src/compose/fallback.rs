//! Port of `bot/compose/fallback.py`: the composer's safety net, a fit reply when no module
//! answered the line, and the many readings of a line and its thread the other composers share.
//!
//! Every module-level table, pool and compiled regex comes from the data bundle under its
//! Python name in this module's namespace (`bot.compose.fallback.X`), which is the binding the
//! running module sees, including the names re-imported at the end of the Python file.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, clippy::all)]

use crate::compose::fallback_chain::{_choose, _mood, _words};
use crate::data::{self, Value};
use crate::knowledge::entry::Entry;
use crate::py::re::{Match, Re};
use crate::py::text;
use crate::runtime::Candidate;
use crate::state::{Memory, Turn};
use crate::understanding::rx;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.compose.fallback.";

/// Compiled regexes of this module's namespace, by Python name.
macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.fallback.", stringify!($name))));)*
    };
}

/// String sets (frozensets, sets, dict keys) of this module's namespace, for membership tests.
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.compose.fallback.", stringify!($name))));)*
    };
}

/// String sequences (reply pools, ordered word lists) of this module's namespace, in order.
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.fallback.", stringify!($name))));)*
    };
}

/// Plain strings of this module's namespace.
macro_rules! strings {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<String> =
            LazyLock::new(|| data::string(concat!("bot.compose.fallback.", stringify!($name))));)*
    };
}

/// Any other value of this module's namespace (dicts, tuples of tuples).
macro_rules! values {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Value> =
            LazyLock::new(|| data::get(concat!("bot.compose.fallback.", stringify!($name))));)*
    };
}

/// `re.findall(r"[a-z']+", s)`: the runs of lowercase ASCII letters and apostrophes.
fn az_words(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in s.chars() {
        if c.is_ascii_lowercase() || c == '\'' {
            cur.push(c);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// `re.findall(r"[a-z0-9']+", s)`.
fn az09_words(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in s.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '\'' {
            cur.push(c);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// `set(re.findall(r"[a-z']+", s))`.
fn az_set(s: &str) -> HashSet<String> {
    az_words(s).into_iter().collect()
}

/// Python's `seq[-n:]` for a count `n` (`seq[-0:]` is the whole sequence, as in Python).
fn last_n<T>(seq: &[T], n: i64) -> &[T] {
    let len = seq.len() as i64;
    let start = -n;
    let start = if start < 0 { (len + start).max(0) } else { start.min(len) };
    &seq[start as usize..]
}

/// A look-back count passed as a Python value (an int, or None for the default).
fn look_back_of(v: &Value, default: i64) -> i64 {
    v.as_int().unwrap_or(default)
}

/// `str(x or "").lower()`.
fn lower(s: &str) -> String {
    text::lower(s)
}

/// `any(word in s for word in set)` written as `set(words) & other`.
fn meets(words: &HashSet<String>, other: &HashSet<String>) -> bool {
    if words.len() <= other.len() { words.iter().any(|w| other.contains(w)) } else { other.iter().any(|w| words.contains(w)) }
}

/// `random.Random(seed).choice(...)`-free pick: `_choose` with no seed text.
fn choose(options: &[String], memory: &Memory, clean: &str) -> Option<String> {
    let picked = _choose(options, memory, clean, "");
    if picked.is_empty() { None } else { Some(picked) }
}

/// `_choose(options, memory, clean, seed_text)` with a seed text.
fn choose_seeded(options: &[String], memory: &Memory, clean: &str, seed_text: &str) -> Option<String> {
    let picked = _choose(options, memory, clean, seed_text);
    if picked.is_empty() { None } else { Some(picked) }
}

/// Python's `str.format` with named fields: `{name}` replaced, `{{`/`}}` unescaped.
fn format_named(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut rest = template;
    while let Some(i) = rest.find(['{', '}']) {
        out.push_str(&rest[..i]);
        let c = rest.as_bytes()[i];
        if c == b'{' && rest[i + 1..].starts_with('{') {
            out.push('{');
            rest = &rest[i + 2..];
            continue;
        }
        if c == b'}' {
            out.push('}');
            rest = if rest[i + 1..].starts_with('}') { &rest[i + 2..] } else { &rest[i + 1..] };
            continue;
        }
        let end = rest[i..].find('}').map(|j| i + j).unwrap_or(rest.len());
        let key = &rest[i + 1..end];
        match fields.iter().find(|(k, _)| *k == key) {
            Some((_, v)) => out.push_str(v),
            None => panic!("KeyError: {key}"),
        }
        rest = if end < rest.len() { &rest[end + 1..] } else { "" };
    }
    out.push_str(rest);
    out
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
pub fn _an_order_of_work(sides: Option<&[String]>, clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_an_order_of_work")
}

/// Python: bot/compose/fallback.py:8419 `_choice_sides(clean)`
pub fn _choice_sides(clean: &str) -> Option<Vec<String>> {
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
pub fn _yours(phrase: &str) -> String {
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
pub fn an_outcome_asked_about_someone(text: &str, memory: &Memory) -> Option<String> {
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

// ---- fallback.py:801-1770: phrase matching, the manners ask, the stuck and dithering lines ----

regexes!(HOW_WAS_YOUR_TIME, YOUR_DAY_ASK, YOUR_TIME_ASK);
pools!(HOW_ARE_YOU_ASKS, HOW_ARE_YOU_GOING_ON);
word_sets!(HOW_ARE_YOU_GOING_ON_AFTER, HOW_ARE_YOU_TAGS);

/// Python: bot/compose/fallback.py:801 `_bounded(phrase)`
/// A phrase matched as whole words, so "raining" is not "training". (Python keeps the compiled
/// patterns in `_BOUNDED`; `rx` is the same cache.)
pub fn _bounded(phrase: &str) -> &'static Re {
    rx(&format!("(?<![a-z]){}(?![a-z])", crate::py::re::escape(phrase)))
}

/// Python: bot/compose/fallback.py:810 `_has(clean, phrases)`
pub fn _has(clean: &str, phrases: &[String]) -> bool {
    phrases.iter().any(|phrase| _bounded(phrase).is_match(clean))
}

/// Python: bot/compose/fallback.py:824 `_matchable(text)`
/// The line as a phrase table can see it: lowercase, no apostrophes, single spaces.
pub fn _matchable(text: &str) -> String {
    let t = lower(text).replace('\u{2019}', "'").replace('\'', "");
    text::squash(&t)
}

/// Python: bot/compose/fallback.py:835 `_asks_how_are_you(clean, repaired='')`
pub fn _asks_how_are_you(clean: &str, repaired: &str) -> bool {
    let mut forms = vec![_matchable(clean)];
    if !repaired.is_empty() {
        let r = _matchable(repaired);
        if r != forms[0] {
            forms.push(r);
        }
    }
    for form in &forms {
        for m in HOW_WAS_YOUR_TIME.finditer(form) {
            if _nothing_but_tags(form, m.end()) {
                return true;
            }
        }
        for phrase in HOW_ARE_YOU_ASKS.iter() {
            for m in _bounded(&_matchable(phrase)).finditer(form) {
                if _nothing_but_tags(form, m.end()) {
                    return true;
                }
            }
        }
        for phrase in HOW_ARE_YOU_GOING_ON.iter() {
            for m in _bounded(&_matchable(phrase)).finditer(form) {
                let tail = &form[m.end()..];
                // "my daughter is 7 going on 8": the figure after the phrase is an age.
                if rx(r"\d").is_match(tail) {
                    continue;
                }
                let after = az_words(tail).join(" ");
                if HOW_ARE_YOU_GOING_ON_AFTER.contains(&after) {
                    return true;
                }
            }
        }
        for pattern in [&*YOUR_DAY_ASK, &*YOUR_TIME_ASK] {
            for m in pattern.finditer(form) {
                if _nothing_but_tags(form, m.end()) {
                    return true;
                }
            }
        }
    }
    false
}

/// Python: bot/compose/fallback.py:876 `_nothing_but_tags(form, end)` (never called on the trace corpus)
/// True when what follows the phrase is only a tag. `end` is a byte offset into `form`.
pub fn _nothing_but_tags(form: &str, end: usize) -> bool {
    az_words(&form[end..]).iter().all(|w| HOW_ARE_YOU_TAGS.contains(w))
}

regexes!(STUCK_ON_A_PERSON, STUCK_PERSON_TAIL_PHRASE, STUCK_A_CLAUSE_ABOUT_THEM, STUCK_AND_ANOTHER_PERSON, STUCK_DESCRIBES_THE_PERSON);

/// Python: bot/compose/fallback.py:1124 `_the_person_ends_the_ask(rest)` (never called on the trace corpus)
/// True when what follows the person is about the person, not a thing of theirs.
pub fn _the_person_ends_the_ask(rest: &str) -> bool {
    let rest = rest.trim_start_matches(' ').trim_end_matches([' ', '?', '.', '!', ',']);
    if rest.trim_matches([' ', ',', '?', '!', '.']).is_empty() {
        return true;
    }
    if STUCK_AND_ANOTHER_PERSON.match_(rest).is_some() {
        return true;
    }
    if STUCK_DESCRIBES_THE_PERSON.match_(rest).is_some() {
        return true;
    }
    if STUCK_A_CLAUSE_ABOUT_THEM.match_(rest).is_some() {
        return true;
    }
    let mut left: &str = rest;
    loop {
        let Some(found) = STUCK_PERSON_TAIL_PHRASE.match_(left) else { break };
        left = left[found.end()..].trim_start_matches([' ', ',', '?', '!', '.']);
    }
    left.is_empty()
}

/// Python: bot/compose/fallback.py:1146 `_the_person_they_are_stuck_on(clean)`
/// The person named in "i dont know what to do about my mum", faced back at them ("" for none).
pub fn _the_person_they_are_stuck_on(clean: &str) -> String {
    let Some(found) = STUCK_ON_A_PERSON.search(clean) else { return String::new() };
    if !_the_person_ends_the_ask(found.name_str("rest")) {
        return String::new();
    }
    let mut who = _yours(found.name_str("who"));
    if lower(&who).starts_with("our ") {
        who = format!("your {}", text::slice(&who, 4, None));
    }
    who
}

word_sets!(STUCK_ONLY_WORDS, RUN_OUT_WORDS);
regexes!(RUN_OUT, RUN_OUT_DOUBT, RUN_OUT_EMPTY, STUCK_PHRASES);
pools!(DITHER_ABOUT_A_PERSON, A_FREE_TIME_ASK);

/// Python: bot/compose/fallback.py:1222 `_a_bad_line_above(memory)` (never called on the trace corpus)
/// True when the line before this one was a bad one by its own words.
pub fn _a_bad_line_above(memory: &Memory) -> bool {
    let Some(turn) = memory.turns.last() else { return false };
    let last = &turn.user;
    if text::strip(last).is_empty() {
        return false;
    }
    _mood(&_words(last), last) == "negative"
}

/// Python: bot/compose/fallback.py:1247 `_run_out(clean)`
pub fn _run_out(clean: &str) -> bool {
    let words = az_words(clean);
    if words.is_empty() || !RUN_OUT.is_match(clean) {
        return false;
    }
    if !(RUN_OUT_DOUBT.is_match(clean) || RUN_OUT_EMPTY.is_match(clean)) {
        return false;
    }
    words.iter().all(|w| RUN_OUT_WORDS.contains(text::strip_chars(w, "'")))
}

/// Python: bot/compose/fallback.py:1257 `_a_dead_end(clean)`
pub fn _a_dead_end(clean: &str) -> bool {
    _only_the_stuck_phrase(clean) || _run_out(clean)
}

/// Python: bot/compose/fallback.py:1262 `_only_the_stuck_phrase(clean)`
pub fn _only_the_stuck_phrase(clean: &str) -> bool {
    let words = az_words(clean);
    if words.is_empty() || !STUCK_PHRASES.is_match(clean) {
        return false;
    }
    words.iter().all(|w| STUCK_ONLY_WORDS.contains(text::strip_chars(w, "'")))
}

/// Python: bot/compose/fallback.py:1270 `_a_question_about_the_person(clean, memory)`
/// A question about the person they are stuck on, or None when no person is named.
/// (The skeleton's `()` return could not carry the reply; it returns what Python returns.)
pub fn _a_question_about_the_person(clean: &str, memory: &Memory) -> Option<String> {
    let who = _the_person_they_are_stuck_on(clean);
    if who.is_empty() {
        return None;
    }
    let first = text::slice(&who, 0, Some(1));
    let cap = format!("{}{}", text::upper(&first), text::slice(&who, 1, None));
    let lines: Vec<String> =
        DITHER_ABOUT_A_PERSON.iter().map(|line| format_named(line, &[("who", &who), ("Who", &cap)])).collect();
    choose(&lines, memory, clean)
}

regexes!(SAYS_THE_SIDES, DITHER_TWO_WAY, LIFE_DIRECTION, A_STRETCH_OF_TIME);
word_sets!(HOW_TO_DECIDE_FILLER);

/// Python: bot/compose/fallback.py:1303 `_nothing_to_choose(rest)` (never called on the trace corpus)
/// True when what follows "how do i decide/pick/choose" names no subject of its own.
pub fn _nothing_to_choose(rest: &str) -> bool {
    if SAYS_THE_SIDES.match_(rest).is_some() {
        return true;
    }
    !az09_words(&lower(rest)).iter().any(|w| !HOW_TO_DECIDE_FILLER.contains(w))
}

/// Python: bot/compose/fallback.py:1336 `_the_stretch_of_time(clean, memory)`
/// A question about the free afternoon, or None when the line is not that.
pub fn _the_stretch_of_time(clean: &str, memory: &Memory) -> Option<String> {
    if !A_STRETCH_OF_TIME.is_match(clean) {
        return None;
    }
    choose(&A_FREE_TIME_ASK, memory, clean)
}

/// Python: bot/compose/fallback.py:1350 `_weighs_a_choice(clean)`
pub fn _weighs_a_choice(clean: &str) -> bool {
    if DITHER_TWO_WAY.is_match(clean) {
        return true;
    }
    !(LIFE_DIRECTION.is_match(clean) || A_STRETCH_OF_TIME.is_match(clean))
}

regexes!(FACT_BETWEEN, HELP_SERVICE, PERSON_NOT_A_DECISION, PRONOUN_NOT_A_DECISION, DITHER, DITHER_STRONG, DITHER_SUBJECT);

/// Python: bot/compose/fallback.py:1411 `_reads_as_dither(clean, repaired=None)`
pub fn _reads_as_dither(clean: &str, repaired: &str) -> bool {
    let mut lines = vec![clean];
    if !repaired.is_empty() && repaired != clean {
        lines.push(repaired);
    }
    for line in lines {
        if FACT_BETWEEN.is_match(line) {
            continue;
        }
        if PERSON_NOT_A_DECISION.is_match(line) || PRONOUN_NOT_A_DECISION.is_match(line) || HELP_SERVICE.is_match(line) {
            continue;
        }
        if DITHER.is_match(line) && (DITHER_STRONG.is_match(line) || DITHER_SUBJECT.is_match(line)) {
            return true;
        }
    }
    false
}

regexes!(PLAN_WORDS, TAKING_UP);

/// Python: bot/compose/fallback.py:1634 `_a_plan_in_the_thread(clean, memory)` (never called on the trace corpus)
/// True when the plan the doubt is about is in the line or the turn before it.
pub fn _a_plan_in_the_thread(clean: &str, memory: &Memory) -> bool {
    for t in [clean.to_string(), _the_line_before(memory)] {
        let said = lower(&t);
        if !said.is_empty() && (PLAN_WORDS.is_match(&said) || TAKING_UP.is_match(&said)) {
            return true;
        }
    }
    _a_thing_they_started(memory, &Value::None)
}

regexes!(REFUSES_IT, A_CHILD, SOMEONE_CLOSE, SOMEONE_ELSE, A_CLINIC, AN_AID, THE_DOCTOR, HELP_IN);

/// Python: bot/compose/fallback.py:1696 `_a_carer_refused(clean)`
/// Which of the refusals "my mum wont ..." is ("clinic", "aid", "doctor", "help"), or None.
pub fn _a_carer_refused(clean: &str) -> Option<String> {
    if !REFUSES_IT.is_match(clean) || A_CHILD.is_match(clean) {
        return None;
    }
    if !(SOMEONE_CLOSE.is_match(clean) || SOMEONE_ELSE.is_match(clean)) {
        return None;
    }
    let kind = if A_CLINIC.is_match(clean) {
        "clinic"
    } else if AN_AID.is_match(clean) {
        "aid"
    } else if THE_DOCTOR.is_match(clean) {
        "doctor"
    } else if HELP_IN.is_match(clean) {
        "help"
    } else {
        return None;
    };
    Some(kind.to_string())
}

/// Python: bot/compose/fallback.py:1713 `_a_plan_doubt(pred, clean, repaired)`
/// `pred` is a compiled regex's `search` (the only predicate PyBot passes, `REALISTIC.search`).
pub fn _a_plan_doubt(pred: &Value, clean: &str, repaired: &str) -> bool {
    let rx_ = pred.as_re().unwrap_or_else(|| panic!("_a_plan_doubt: predicate is not a regex: {pred:?}"));
    _a_plan_doubt_with(|t| rx_.is_match(t), clean, repaired)
}

/// `_a_plan_doubt` with any predicate.
pub fn _a_plan_doubt_with(pred: impl Fn(&str) -> bool, clean: &str, repaired: &str) -> bool {
    for t in [clean, repaired] {
        if pred(t) {
            return true;
        }
        let tail = if t.contains(',') { text::strip(t.rsplit(',').next().unwrap_or("")) } else { "" };
        if !tail.is_empty() && pred(tail) {
            return true;
        }
    }
    false
}

regexes!(SHOULD_WORRY, SHOULD_WORRY_FILLER);
word_sets!(BODY_PARTS, HEALTH_WORDS, SYMPTOM_WORDS);
pools!(CHECK_PHRASES);

/// Python: bot/compose/fallback.py:1759 `_a_bare_worry(clean)` (never called on the trace corpus)
/// True when "should i worry" is the whole ask, not attached to a thing they describe.
pub fn _a_bare_worry(clean: &str) -> bool {
    let Some(found) = SHOULD_WORRY.search(clean) else { return false };
    let rest = format!("{} {}", &clean[..found.start()], &clean[found.end()..]);
    let rest = SHOULD_WORRY_FILLER.sub(" ", &rest);
    !rest.chars().any(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

/// Python: bot/compose/fallback.py:1769 `_worry_is_about_the_body(clean, words)`
pub fn _worry_is_about_the_body(clean: &str, words: &[String]) -> bool {
    words.iter().any(|w| BODY_PARTS.contains(w) || HEALTH_WORDS.contains(w) || SYMPTOM_WORDS.contains(w))
        || CHECK_PHRASES.iter().any(|p| clean.contains(p.as_str()))
}

// ---- fallback.py:1782-2760: whose trouble the thread is, the worry about somebody, unknown pools ----

pools!(
    UNKNOWN_HOW_IT_GOES_FOR_SOMEONE,
    UNKNOWN_HOW_LONG_IN_HOSPITAL,
    UNKNOWN_HOW_IT_GOES_FOR_A_HARD_TIME,
    UNKNOWN_HOW_IT_GOES_FOR_A_PET,
    UNKNOWN_IF_THEY_STILL_CAN,
);
word_sets!(THE_PEOPLE, VISIT_PRONOUNS, NOT_PEOPLE_WORDS, ANIMALS, WILD_BIRDS, MESS_PLACES);
regexes!(IN_HOSPITAL);

/// `{w[:-2] if w.endswith("'s") else w for w in re.findall(r"[a-z']+", said)}`.
fn words_without_possessive(said: &str) -> HashSet<String> {
    az_words(said)
        .into_iter()
        .map(|w| if w.ends_with("'s") { w[..w.len() - 2].to_string() } else { w })
        .collect()
}

/// Python: bot/compose/fallback.py:1845 `_somebody_to_visit(clean, turns, look_back=3)` (never called on the trace corpus)
/// True when the line or the last few turns name a person the visit could be to.
pub fn _somebody_to_visit(clean: &str, turns: &[Turn], look_back: i64) -> bool {
    let is_person = |w: &String| THE_PEOPLE.contains(w) || VISIT_PRONOUNS.contains(w);
    if az_words(clean).iter().any(is_person) {
        return true;
    }
    for turn in last_n(turns, look_back) {
        if az_words(&lower(&turn.user)).iter().any(is_person) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:1857 `_someone_is_a_patient(clean, turns, look_back=4)` (never called on the trace corpus)
pub fn _someone_is_a_patient(clean: &str, turns: &[Turn], look_back: i64) -> bool {
    if IN_HOSPITAL.is_match(clean) {
        return true;
    }
    last_n(turns, look_back).iter().any(|turn| IN_HOSPITAL.is_match(&lower(&turn.user)))
}

regexes!(AN_ANIMAL_DID_IT, AN_ANIMAL_DID_IT_TO_ANYONE, A_PERSON_ELSE_IS_HURT, A_PERSONS_TREATMENT, A_PERSON_IS_BLEEDING, AN_A_HARM_VERB, AN_ANIMAL_WORD);

/// Python: bot/compose/fallback.py:1938 `_the_animal_is_the_patient(said)`
pub fn _the_animal_is_the_patient(said: &str) -> bool {
    let t = lower(said);
    let animals: Vec<usize> = AN_ANIMAL_WORD.finditer(&t).iter().map(|m| m.start()).collect();
    if animals.is_empty() {
        return false;
    }
    if AN_ANIMAL_DID_IT.is_match(&t) {
        return false;
    }
    let Some(verb) = AN_A_HARM_VERB.search(&t) else { return false };
    if *animals.iter().min().unwrap() > verb.start() {
        return false;
    }
    // tail & (THE_PEOPLE - ANIMALS)
    !az_words(&t[verb.end()..]).iter().any(|w| THE_PEOPLE.contains(w) && !ANIMALS.contains(w))
}

/// Python: bot/compose/fallback.py:1958 `_the_person_is_the_patient(said)`
pub fn _the_person_is_the_patient(said: &str) -> bool {
    if said.is_empty() || _the_animal_is_the_patient(said) {
        return false;
    }
    AN_ANIMAL_DID_IT.is_match(said)
        || AN_ANIMAL_DID_IT_TO_ANYONE.is_match(said)
        || A_PERSON_ELSE_IS_HURT.is_match(said)
        || A_PERSONS_TREATMENT.is_match(said)
        || A_PERSON_IS_BLEEDING.is_match(said)
}

/// Python: bot/compose/fallback.py:1975 `_the_thread_is_an_animal(clean, turns, look_back=4)`
pub fn _the_thread_is_an_animal(clean: &str, turns: &[Turn], look_back: i64) -> bool {
    let said = lower(clean);
    let recent: Vec<String> = last_n(turns, look_back).iter().map(|t| lower(&t.user)).collect();
    if _the_person_is_the_patient(&said) || recent.iter().any(|x| _the_person_is_the_patient(x)) {
        return false;
    }
    if az_words(&said).iter().any(|w| NOT_PEOPLE_WORDS.contains(w)) {
        return true;
    }
    recent.iter().any(|x| az_words(x).iter().any(|w| NOT_PEOPLE_WORDS.contains(w)))
}

static TROUBLE_OTHERS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.trouble.OTHERS"));

/// Python: bot/compose/fallback.py:1993 `_thread_is_about_someone_else(memory, look_back=3)` (never called on the trace corpus)
/// (`look_back`: an int, or None for the default.)
pub fn _thread_is_about_someone_else(memory: &Memory, look_back: &Value) -> bool {
    for turn in last_n(&memory.turns, look_back_of(look_back, 3)) {
        let words = words_without_possessive(&lower(&turn.user));
        if words.iter().any(|w| TROUBLE_OTHERS.contains(w) && !NOT_PEOPLE_WORDS.contains(w)) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:2008 `_thread_is_about_a_pet(memory, look_back=3)`
pub fn _thread_is_about_a_pet(memory: &Memory, look_back: i64) -> bool {
    for turn in last_n(&memory.turns, look_back).iter().rev() {
        let said = lower(&turn.user);
        let words = words_without_possessive(&said);
        let animals: Vec<&String> = words.iter().filter(|w| ANIMALS.contains(*w)).collect();
        if animals.is_empty() {
            if _the_speaker_is_the_subject(&said) {
                return false;
            }
            continue;
        }
        if animals.iter().all(|a| WILD_BIRDS.contains(*a)) && words.iter().any(|w| MESS_PLACES.contains(w)) {
            continue;
        }
        if crate::compose::loss::reads_as_loss(&said, None, false) || crate::compose::badnews::reads_as_sudden_loss(&said) {
            return false;
        }
        return true;
    }
    false
}

regexes!(
    ASKED_ABOUT,
    WHETHER_IT_GOES_WELL,
    WHETHER_THEY_CAN_STILL,
    WHETHER_IT_GOES_WELL_ITSELF,
    WHETHER_IT_LONGER_HAS_GOT,
    AN_ILLNESS,
    A_PET_IN_TROUBLE,
    WHETHER_IT_IS_THEIR_AGE,
    A_PATIENT_STATE,
    A_HARD_TIME,
    A_MEDICAL_THREAD,
);
word_sets!(_MY_PEOPLE);

/// Python: bot/compose/fallback.py:2145 `_the_thread_said(memory, look_back=3)` (never called on the trace corpus)
/// The last few things the person said, as one lowercase string.
pub fn _the_thread_said(memory: &Memory, look_back: i64) -> String {
    last_n(&memory.turns, look_back).iter().map(|t| lower(&t.user)).collect::<Vec<_>>().join(" ")
}

/// Python: bot/compose/fallback.py:2151 `_a_person_in_the_thread(memory, look_back=3)`
pub fn _a_person_in_the_thread(memory: &Memory, look_back: i64) -> bool {
    for turn in last_n(&memory.turns, look_back) {
        let said = lower(&turn.user);
        let words = words_without_possessive(&said);
        if words.iter().any(|w| _MY_PEOPLE.contains(w.trim_end_matches('s'))) || A_PATIENT_STATE.is_match(&said) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:2167 `_a_hard_time_and_not_a_body(memory, line='')` (never called on the trace corpus)
pub fn _a_hard_time_and_not_a_body(memory: &Memory, line: &str) -> bool {
    let said = format!("{} {}", _the_thread_said(memory, 3), line);
    if A_MEDICAL_THREAD.is_match(&said) || AN_ILLNESS.is_match(&said) {
        return false;
    }
    A_HARD_TIME.is_match(&_the_thread_said(memory, 3))
}

/// Python: bot/compose/fallback.py:2181 `_a_pet_in_trouble(line, memory)`
pub fn _a_pet_in_trouble(line: &str, memory: &Memory) -> bool {
    if !_thread_is_about_a_pet(memory, 3) || _a_person_in_the_thread(memory, 2) {
        return false;
    }
    let said = _the_thread_said(memory, 3);
    A_PET_IN_TROUBLE.is_match(line) || A_PET_IN_TROUBLE.is_match(&said) || A_HARD_TIME.is_match(&said)
}

/// Python: bot/compose/fallback.py:2197 `_the_pool_for_a_worry_about_them(clean, memory)`
/// The pool for "is he going to be ok" and its kind, or None. (The skeleton's `()` return could
/// not carry the pool.)
pub fn _the_pool_for_a_worry_about_them(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let line = rx(r"[^a-z0-9' ]").sub(" ", &lower(clean));
    if an_outcome_asked_about_someone(clean, memory).is_some() || the_threads_answer(clean, memory).is_some() {
        return None;
    }
    let still = WHETHER_THEY_CAN_STILL.is_match(&line);
    let itself = WHETHER_IT_GOES_WELL_ITSELF.is_match(&line) || WHETHER_IT_LONGER_HAS_GOT.is_match(&line);
    if itself {
        if _a_pet_in_trouble(&line, memory) {
            return Some(UNKNOWN_HOW_IT_GOES_FOR_A_PET.clone());
        }
        if !_a_person_in_the_thread(memory, 3) {
            return None;
        }
        if !(WHETHER_IT_LONGER_HAS_GOT.is_match(&line) || AN_ILLNESS.is_match(&_the_thread_said(memory, 3))) {
            return None;
        }
    }
    if !(still || itself || WHETHER_IT_GOES_WELL.is_match(&line) || WHETHER_IT_IS_THEIR_AGE.is_match(&line)) {
        return None;
    }
    let named = ASKED_ABOUT.search(&line).map(|m| m.as_str().to_string()).unwrap_or_default();
    if text::split_ref(&named).iter().any(|w| ANIMALS.contains(*w) || NOT_PEOPLE_WORDS.contains(*w)) {
        return Some(UNKNOWN_HOW_IT_GOES_FOR_A_PET.clone());
    }
    if named.is_empty() || rx(r"(?:he|she|they)\b").match_(&named).is_some() {
        if _a_pet_in_trouble(&line, memory) {
            return Some(UNKNOWN_HOW_IT_GOES_FOR_A_PET.clone());
        }
        if !_a_person_in_the_thread(memory, 3) {
            return None;
        }
    }
    if still {
        return Some(UNKNOWN_IF_THEY_STILL_CAN.clone());
    }
    if _a_hard_time_and_not_a_body(memory, &line) {
        return Some(UNKNOWN_HOW_IT_GOES_FOR_A_HARD_TIME.clone());
    }
    if _someone_is_a_patient(&line, &memory.turns, 4) {
        return Some(UNKNOWN_HOW_LONG_IN_HOSPITAL.clone());
    }
    Some(UNKNOWN_HOW_IT_GOES_FOR_SOMEONE.clone())
}

regexes!(THEIR_OWN_STATE);

/// Python: bot/compose/fallback.py:2262 `_the_speaker_is_the_subject(said)`
pub fn _the_speaker_is_the_subject(said: &str) -> bool {
    THEIR_OWN_STATE.is_match(said)
}

/// Python: bot/compose/fallback.py:2271 `_a_thing_they_started(memory, look_back=3)` (never called on the trace corpus)
/// (`look_back`: an int, or None for the default.)
pub fn _a_thing_they_started(memory: &Memory, look_back: &Value) -> bool {
    last_n(&memory.turns, look_back_of(look_back, 3)).iter().any(|t| TAKING_UP.is_match(&lower(&t.user)))
}

regexes!(_AGE_DOUBT, WORRY_SELF, WEIRD_ASK, FEELING_SAID);
pools!(UNKNOWN_WORRY, UNKNOWN_WEIRD_BARE);

/// Python: bot/compose/fallback.py:2285 `_names_their_age(clean)` (never called on the trace corpus)
pub fn _names_their_age(clean: &str) -> bool {
    rx(r"\b\d{1,2}\b").is_match(clean) || _AGE_DOUBT.is_match(clean)
}

/// Python: bot/compose/fallback.py:2291 `_reads_as_worry(clean, words=())`
pub fn _reads_as_worry(clean: &str, words: &[String]) -> bool {
    let split;
    let words = if words.is_empty() {
        split = text::split(clean);
        &split[..]
    } else {
        words
    };
    if _worry_is_about_the_body(clean, words) {
        return false;
    }
    if WORRY_SELF.is_match(clean) {
        return true;
    }
    SHOULD_WORRY.is_match(clean) && _a_bare_worry(clean)
}

/// Python: bot/compose/fallback.py:2301 `_the_weird_ask_pool(clean, memory)`
/// The pool for "is that weird / normal", or None when the line is a question of fact.
pub fn _the_weird_ask_pool(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let found = WEIRD_ASK.search(clean)?;
    if _worry_is_about_the_body(clean, &text::split(clean)) || _thread_is_about_the_body(memory, 4) {
        return None;
    }
    if FEELING_SAID.is_match(clean) || FEELING_SAID.is_match(&_the_line_before(memory)) {
        return Some(UNKNOWN_WORRY.clone());
    }
    if _the_line_before(memory).is_empty() && _nothing_but_the_ask(clean, &found) {
        return Some(UNKNOWN_WEIRD_BARE.clone());
    }
    None
}

/// Python: bot/compose/fallback.py:2324 `_nothing_but_the_ask(clean, found)`
pub fn _nothing_but_the_ask(clean: &str, found: &Match) -> bool {
    let rest = format!("{} {}", &clean[..found.start()], &clean[found.end()..]);
    !SHOULD_WORRY_FILLER.sub(" ", &rest).chars().any(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

static IDIOM_PHRASES: LazyLock<Vec<Vec<String>>> =
    LazyLock::new(|| data::get("bot.compose.fallback.IDIOM_PHRASES").items().iter().map(Value::strs).collect());

/// Python: bot/compose/fallback.py:2330 `_without_the_idioms(said)`
/// The words of a line with the phrases in `IDIOM_PHRASES` taken out (a set).
pub fn _without_the_idioms(said: &str) -> HashSet<String> {
    let words = az_words(said);
    if words.is_empty() {
        return HashSet::new();
    }
    let mut taken = vec![false; words.len()];
    for phrase in IDIOM_PHRASES.iter() {
        let size = phrase.len();
        if size > words.len() {
            continue;
        }
        for at in 0..=(words.len() - size) {
            if words[at..at + size] == phrase[..] {
                for t in &mut taken[at..at + size] {
                    *t = true;
                }
            }
        }
    }
    words.into_iter().enumerate().filter(|(at, _)| !taken[*at]).map(|(_, w)| w).collect()
}

/// Python: bot/compose/fallback.py:2351 `_thread_is_about_the_body(memory, look_back=4)`
pub fn _thread_is_about_the_body(memory: &Memory, look_back: i64) -> bool {
    for turn in last_n(&memory.turns, look_back) {
        let said = lower(&turn.user);
        if said.is_empty() {
            continue;
        }
        let said = _without_the_things_named_after_a_body(&_without_the_forecast_verb(&said));
        let words = _without_the_idioms(&said);
        if words.iter().any(|w| SYMPTOM_WORDS.contains(w) || BODY_PARTS.contains(w) || HEALTH_WORDS.contains(w)) {
            return true;
        }
    }
    false
}

strings!(HEALTH_WHO);

/// Python: bot/compose/fallback.py:2366 `_the_threads_who(memory, look_back=2)`
/// The person the thread's own line hands the question to, or None.
pub fn _the_threads_who(memory: &Memory, look_back: i64) -> Option<String> {
    for turn in last_n(&memory.turns, look_back).iter().rev() {
        let said = format!(" {} ", rx(r"[^a-z0-9']").sub(" ", &lower(&turn.user)));
        let words = text::split(&said.replace('\'', ""));
        if words.is_empty() {
            continue;
        }
        let who = _who_for(text::strip(&said), &said, &words, None);
        if let Some(who) = who {
            if !who.is_empty() && who != *HEALTH_WHO {
                return Some(who);
            }
        }
    }
    None
}

regexes!(_LEANS_ON_THE_THREAD, _A_CANDIDATE_OF_ITS_OWN, ASK_SHAPE);
pools!(UNKNOWN_WHO, UNKNOWN_HEALTH);

/// `understand(text, memory)` inside Python's `try: ... except Exception: understood = {}`.
fn understand_or_empty(text_: &str, memory: Option<&Memory>) -> Understood {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| crate::understanding::understand(text_, memory)))
        .unwrap_or_default()
}

/// Python: bot/compose/fallback.py:2397 `the_threads_answer(text, memory)`
/// The line for a follow-up that only points at the thread, or None. (The skeleton's `()`
/// return could not carry the line.)
pub fn the_threads_answer(text_: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(text_);
    if clean.is_empty() || _hedged_recently(memory, 3) {
        return None;
    }
    let understood = understand_or_empty(&clean, Some(memory));
    if !_the_ask_is_the_threads(&clean, &understood) {
        return None;
    }
    if let Some(who) = _the_threads_who(memory, 2) {
        let lines: Vec<String> = UNKNOWN_WHO.iter().map(|l| format_named(l, &[("who", &who)])).collect();
        return choose(&lines, memory, &clean);
    }
    let leans = understood.leans_on.as_ref().and_then(|l| l.topic.as_deref());
    if leans == Some("health") || _thread_is_about_the_body(memory, 1) {
        return choose(&UNKNOWN_HEALTH, memory, &clean);
    }
    None
}

word_sets!(_THREAD_MACHINES);

/// Python: bot/compose/fallback.py:2449 `_thread_names_a_machine(memory, look_back=4)` (never called on the trace corpus)
/// (`look_back`: an int, or None for the default.)
pub fn _thread_names_a_machine(memory: &Memory, look_back: &Value) -> bool {
    let body = |w: &str| BODY_PARTS.contains(w) || SYMPTOM_WORDS.contains(w) || HEALTH_WORDS.contains(w);
    for turn in last_n(&memory.turns, look_back_of(look_back, 4)).iter().rev() {
        let tokens = az_words(&lower(&turn.user));
        let machine_at: Vec<usize> = tokens.iter().enumerate().filter(|(_, w)| _THREAD_MACHINES.contains(*w)).map(|(i, _)| i).collect();
        if machine_at.is_empty() {
            if tokens.iter().any(|w| body(w)) {
                return false;
            }
            continue;
        }
        let body_at: Vec<usize> = tokens.iter().enumerate().filter(|(_, w)| body(w)).map(|(i, _)| i).collect();
        if body_at.iter().all(|&i| machine_at.iter().any(|&j| (i as i64 - j as i64).abs() == 1)) {
            return true;
        }
    }
    false
}

static _BODY_TOPICS: LazyLock<Vec<Option<String>>> = LazyLock::new(|| {
    data::get("bot.compose.fallback._BODY_TOPICS").items().iter().map(|v| v.as_str().map(str::to_string)).collect()
});

/// Python: bot/compose/fallback.py:2490 `_the_ask_is_the_threads(clean, understood)`
pub fn _the_ask_is_the_threads(clean: &str, understood: &Understood) -> bool {
    if !(ASK_SHAPE.is_match(clean) && _LEANS_ON_THE_THREAD.is_match(clean)) {
        return false;
    }
    if _A_CANDIDATE_OF_ITS_OWN.is_match(&lower(clean)) {
        return false;
    }
    _BODY_TOPICS.contains(&understood.topic)
}

word_sets!(_THREAD_PEOPLE, _NOT_AN_OWN_THING, MONEY_WORDS, _MONEY_NOUN, SORE_WORDS, SORE_AMBIGUOUS);
regexes!(REPAIR_PERSON, REPAIR_DECISION);

/// Python: bot/compose/fallback.py:2503 `_thread_is_about_a_person(memory, look_back=4)` (never called on the trace corpus)
/// (`look_back`: an int, or None for the default.)
pub fn _thread_is_about_a_person(memory: &Memory, look_back: &Value) -> bool {
    for turn in last_n(&memory.turns, look_back_of(look_back, 4)) {
        if az_words(&lower(&turn.user)).iter().any(|w| _THREAD_PEOPLE.contains(w)) || REPAIR_PERSON.is_match(&turn.user) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:2513 `_a_thing_was_named(memory, look_back=4)` (never called on the trace corpus)
/// (`look_back`: an int, or None for the default.)
pub fn _a_thing_was_named(memory: &Memory, look_back: &Value) -> bool {
    for turn in last_n(&memory.turns, look_back_of(look_back, 4)) {
        let said = &turn.user;
        if text::strip(said).is_empty() {
            continue;
        }
        if crate::understanding::understand(said, None).entities.iter().any(|e| e.kind == "thing") {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:2535 `_names_its_own_thing(clean)` (never called on the trace corpus)
pub fn _names_its_own_thing(clean: &str) -> bool {
    let Some(m) = REPAIR_DECISION.search(clean) else { return false };
    for word in az_words(&clean[m.end()..]) {
        if _NOT_AN_OWN_THING.contains(&word) {
            continue;
        }
        if BODY_PARTS.contains(&word) || HEALTH_WORDS.contains(&word) || SYMPTOM_WORDS.contains(&word) {
            continue;
        }
        if REPAIR_PERSON.is_match(&word) {
            continue;
        }
        return true;
    }
    false
}

regexes!(_PAY_DEBT, _CARD_MONEY, _VALUE_OF_THING, SORE_DOING, SORE_PICKING, SORE_ONLY_A_BODY, SORE_PLACE);

/// Python: bot/compose/fallback.py:2554 `_money_decision(clean)`
pub fn _money_decision(clean: &str) -> bool {
    let said = az_set(clean);
    if !said.iter().any(|w| MONEY_WORDS.contains(w)) && !_PAY_DEBT.is_match(clean) && !_CARD_MONEY.is_match(clean) {
        return false;
    }
    if _VALUE_OF_THING.is_match(clean) && !said.iter().any(|w| _MONEY_NOUN.contains(w)) {
        return false;
    }
    true
}

/// Python: bot/compose/fallback.py:2564 `_about_a_sore(clean, words)`
pub fn _about_a_sore(clean: &str, words: &[String]) -> bool {
    if !SORE_DOING.is_match(clean) {
        return false;
    }
    let named: HashSet<&String> = words.iter().filter(|w| SORE_WORDS.contains(*w)).collect();
    if named.is_empty() {
        return false;
    }
    if named.iter().any(|w| !SORE_AMBIGUOUS.contains(*w)) {
        return true;
    }
    if !SORE_PICKING.is_match(clean) || SORE_ONLY_A_BODY.is_match(clean) {
        return true;
    }
    SORE_PLACE.is_match(clean)
}

regexes!(
    _ASKS_TO_ACT,
    _A_PLACE_TO_BE_SEEN,
    _A_CALL_WORD,
    DANGER_SYMPTOM,
    MINOR_HARM,
    EMERGENCY_CALL,
    POLICE_CALL,
    REACHING_THE_POLICE,
    _ASKING_FOR_THE_POLICE,
    SOMETHING_HAPPENING,
);
pools!(RING_FOR_HELP, GO_TO_BE_SEEN, RING_THE_POLICE);

/// Python: bot/compose/fallback.py:2583 `_a_call_for_help(clean)`
/// The pool for a line asking whether to ring for help, or None when it is another question.
pub fn _a_call_for_help(clean: &str) -> Option<Vec<String>> {
    if !_ASKS_TO_ACT.is_match(clean) {
        return None;
    }
    if _A_PLACE_TO_BE_SEEN.is_match(clean) && !_A_CALL_WORD.is_match(clean) {
        if DANGER_SYMPTOM.is_match(clean) {
            return Some(RING_FOR_HELP.clone());
        }
        if MINOR_HARM.is_match(clean) {
            return None;
        }
        return Some(GO_TO_BE_SEEN.clone());
    }
    if EMERGENCY_CALL.is_match(clean) {
        if MINOR_HARM.is_match(clean) && !DANGER_SYMPTOM.is_match(clean) {
            return None;
        }
        return Some(RING_FOR_HELP.clone());
    }
    if POLICE_CALL.is_match(clean)
        && (REACHING_THE_POLICE.is_match(clean) || _ASKING_FOR_THE_POLICE.is_match(clean) || SOMETHING_HAPPENING.is_match(clean))
    {
        return Some(RING_THE_POLICE.clone());
    }
    None
}

regexes!(REPAIR_HOW, ASKED_OUT, AN_APOLOGY, _CONTACT, _CALLING, _GOING, _NOT_INVITED);
pools!(REPAIR_ANSWER, ASK_OUT_ANSWER, APOLOGY_ANSWER, CALL_ANSWER, CONTACT_ANSWER, GO_ANSWER);
word_sets!(_A_PERSON, _A_SOCIAL_THING);

/// Python: bot/compose/fallback.py:2627 `_a_small_social_decide(clean, words)`
/// The pool for an everyday social decision, or None when the line is a different question.
pub fn _a_small_social_decide(clean: &str, words: &[String]) -> Option<Vec<String>> {
    if ASKED_OUT.is_match(clean) {
        return Some(ASK_OUT_ANSWER.clone());
    }
    if AN_APOLOGY.is_match(clean) {
        return Some(APOLOGY_ANSWER.clone());
    }
    if _CONTACT.is_match(clean) && words.iter().any(|w| _A_PERSON.contains(w)) {
        if _CALLING.is_match(clean) {
            return Some(CALL_ANSWER.clone());
        }
        return Some(CONTACT_ANSWER.clone());
    }
    if words.iter().any(|w| _A_SOCIAL_THING.contains(w)) && _GOING.is_match(clean) && !_NOT_INVITED.is_match(clean) {
        return Some(GO_ANSWER.clone());
    }
    None
}

regexes!(METHOD_OPENING, HOW_TO_TELL, NOT_A_METHOD, _DEFINITION);
word_sets!(_A_PRODUCT);
pools!(HELP_ALREADY, TIPS_LINES);

/// Python: bot/compose/fallback.py:2691 `method_ask_pool(clean, memory=None)`
/// The pool for a method asked for when nothing else could answer it, else None. (The
/// skeleton's `()` return could not carry the pool.)
pub fn method_ask_pool(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    if METHOD_OPENING.match_(clean).is_none() {
        return None;
    }
    if HOW_TO_TELL.is_match(clean) || NOT_A_METHOD.is_match(clean) {
        return None;
    }
    if az_words(clean).iter().any(|w| _A_PRODUCT.contains(w)) {
        return None;
    }
    if _DEFINITION.is_match(clean) {
        return None;
    }
    if _answered_already(memory) {
        return Some(HELP_ALREADY.clone());
    }
    Some(TIPS_LINES.clone())
}

regexes!(HANDS_IT_BACK, PLASTER_ON, DECIDE_OPENING, DECIDE_SANITY);
pools!(HANDS_BACK, SORE_HANDS, PLASTER_IT, WORD_ANYWHERE, UNKNOWN_WORD, ORDER_OF_WORK, UNKNOWN_MONEY, UNKNOWN_DECIDE, UNKNOWN);
word_sets!(SORE_SERIOUS, _CREATIVE);

/// Python: bot/compose/fallback.py:2719 `unknown_pool(clean, words=())`
/// The pool for a question the bot cannot answer, chosen by the kind of question it is.
pub fn unknown_pool(clean: &str, words: &[String]) -> Vec<String> {
    let split;
    let words = if words.is_empty() {
        split = text::split(clean);
        &split[..]
    } else {
        words
    };
    if HANDS_IT_BACK.match_(clean).is_some() {
        return HANDS_BACK.clone();
    }
    if let Some(calling) = _a_call_for_help(clean) {
        if !calling.is_empty() {
            return calling;
        }
    }
    if REPAIR_HOW.is_match(clean) && words.iter().any(|w| _A_PERSON.contains(w)) {
        return REPAIR_ANSWER.clone();
    }
    if _reads_as_worry(clean, words) {
        return UNKNOWN_WORRY.clone();
    }
    if _about_a_sore(clean, words) {
        if words.iter().any(|w| SORE_SERIOUS.contains(w)) {
            return UNKNOWN_HEALTH.clone();
        }
        return SORE_HANDS.clone();
    }
    if PLASTER_ON.is_match(clean) {
        return PLASTER_IT.clone();
    }
    if WORD_ANYWHERE.iter().any(|p| clean.contains(p.as_str())) && !words.iter().any(|w| _CREATIVE.contains(w)) {
        return UNKNOWN_WORD.clone();
    }
    if !_DEFINITION.is_match(clean) && _an_order_of_work(_choice_sides(clean).as_deref(), clean) {
        return ORDER_OF_WORK.clone();
    }
    if (DECIDE_OPENING.is_match(clean) || DECIDE_SANITY.is_match(clean)) && !_DEFINITION.is_match(clean) {
        if words.iter().any(|w| HEALTH_WORDS.contains(w) || SORE_SERIOUS.contains(w)) || CHECK_PHRASES.iter().any(|p| clean.contains(p.as_str())) {
            return UNKNOWN_HEALTH.clone();
        }
        if _money_decision(clean) {
            return UNKNOWN_MONEY.clone();
        }
        if let Some(social) = _a_small_social_decide(clean, words) {
            if !social.is_empty() {
                return social;
            }
        }
        return UNKNOWN_DECIDE.clone();
    }
    UNKNOWN.clone()
}

// ---- fallback.py:2795-3420: a choking thing, the line before, switching off, "did you see" ----

pools!(THING_STATES);
word_sets!(BODY_SUBJECT, _COPULA, THING_SUBJECT);

/// Python: bot/compose/fallback.py:2798 `_the_thing_is_choking(clean, words)` (never called on the trace corpus)
/// True when a choking state belongs to a thing: the engine, the drain, the bike chain.
pub fn _the_thing_is_choking(clean: &str, words: &[String]) -> bool {
    if !THING_STATES.iter().any(|p| clean.contains(p.as_str())) {
        return false;
    }
    if words.iter().any(|w| BODY_SUBJECT.contains(w)) {
        return false;
    }
    let tokens = text::split(&clean.replace('\'', ""));
    for (i, word) in tokens.iter().enumerate() {
        if word != "choking" {
            continue;
        }
        let mut before: Vec<&String> = tokens[i.saturating_sub(4)..i].iter().collect();
        while before.last().is_some_and(|w| _COPULA.contains(*w)) {
            before.pop();
        }
        if before.last().is_some_and(|w| THING_SUBJECT.contains(*w)) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:2932 `_the_line_before(memory)`
/// What the person said on the turn before this one, or "" on the first turn.
pub fn _the_line_before(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.user.clone()).unwrap_or_default()
}

/// Python: bot/compose/fallback.py:2940 `_reads_as_heavy_news(text)`
/// True when a line is heavy news - a death, a diagnosis, a loss - rather than an ordinary one.
pub fn _reads_as_heavy_news(text_: &str) -> bool {
    let said = lower(text_);
    !said.is_empty()
        && (crate::compose::loss::reads_as_loss(&said, None, false)
            || crate::compose::badnews::reads_as_bad_news(&said)
            || crate::compose::badnews::reads_as_sudden_loss(&said))
}

/// Python: bot/compose/fallback.py:2954 `_the_thread_is_heavy(memory)`
pub fn _the_thread_is_heavy(memory: &Memory) -> bool {
    _reads_as_heavy_news(&_the_line_before(memory))
}

regexes!(SWITCH_OFF_ASK, SWITCH_OFF_GET_OUT, SMOKE_ABOUT, ALREADY_SAID_OFF);
word_sets!(SWITCH_OFF_THINGS, KEEP_IT_RUNNING, SWITCH_OFF_FAULT);
pools!(SWITCH_OFF_ALREADY, SWITCH_OFF_ADVISE, SWITCH_OFF_BATTERY);

/// Python: bot/compose/fallback.py:3047 `_a_switch_off_asked(memory, clean)`
/// The pool for "should i turn it off" about a machine in a thread the bot has just answered,
/// or None. (The skeleton's `()` return could not carry the pool.)
pub fn _a_switch_off_asked(memory: &Memory, clean: &str) -> Option<Vec<String>> {
    if !SWITCH_OFF_ASK.is_match(clean) {
        return None;
    }
    let said = format!(" {} ", lower(&_the_line_before(memory)));
    let answer = format!(" {} ", lower(&memory_bot(memory)));
    let thread = format!("{said}{answer}");
    let things = az_set(&thread);
    if !things.iter().any(|w| SWITCH_OFF_THINGS.contains(w)) {
        return None;
    }
    if things.iter().any(|w| KEEP_IT_RUNNING.contains(w)) {
        return None;
    }
    if clean.contains("battery") && ["alarm", "alarms", "smoke"].iter().any(|w| things.contains(*w)) {
        if ["beeping", "beeps", "chirping", "chirps", "chirp", "beep"].iter().any(|w| things.contains(*w)) {
            return Some(SWITCH_OFF_BATTERY.clone());
        }
        return None;
    }
    if SWITCH_OFF_GET_OUT.is_match(&said) {
        return None;
    }
    if SWITCH_OFF_GET_OUT.is_match(&answer) || answer.contains("999") || answer.contains("911") {
        return None;
    }
    if ALREADY_SAID_OFF.is_match(&answer) {
        return Some(SWITCH_OFF_ALREADY.clone());
    }
    if SMOKE_ABOUT.is_match(&said) {
        return None;
    }
    if things.iter().any(|w| SWITCH_OFF_FAULT.contains(w)) {
        return Some(SWITCH_OFF_ADVISE.clone());
    }
    None
}

regexes!(ASKED_IF_I_SAW, NOT_THE_NEWS, LOOKED_UP, SAW_IT, WATCHABLE, SAW_NEWS, SAW_IT_OR_THE);

/// Python: bot/compose/fallback.py:3144 `names_the_thing_worth_watching(clean)` (never called on the trace corpus)
pub fn names_the_thing_worth_watching(clean: &str) -> bool {
    let Some(said) = SAW_IT_OR_THE.match_(clean) else { return false };
    let tail = text::split_ref(&clean[said.end()..]).into_iter().take(5).collect::<Vec<_>>().join(" ");
    WATCHABLE.is_match(&tail) || SAW_NEWS.is_match(&tail)
}

/// Python: bot/compose/fallback.py:3165 `reads_as_if_i_saw(text, memory=None)`
pub fn reads_as_if_i_saw(text_: &str, memory: &Memory) -> bool {
    let clean = text::squash(&lower(text_).replace('\u{2019}', "'"));
    if clean.is_empty() || NOT_THE_NEWS.is_match(&clean) || LOOKED_UP.is_match(&clean) {
        return false;
    }
    if SAW_IT.match_(&clean).is_some() {
        let last = crate::memory::last_user(memory, 1);
        return WATCHABLE.is_match(&lower(last.as_str().unwrap_or("")));
    }
    if ASKED_IF_I_SAW.match_(&clean).is_some() {
        return names_the_thing_worth_watching(&clean);
    }
    false
}

regexes!(
    WAS_THEY_THERE,
    WAS_A_NAMED_ONE_THERE,
    DID_THEY_GO,
    A_THING_OR_A_PERSON,
    A_LOOK_BACK,
    WAS_IT_ANY_GOOD,
    WENT_AND_WERE_ASKED,
    WHO_WAS_THERE,
    HURT_IN_THE_LINE,
);
word_sets!(PEOPLE_WHO_WENT, NOT_A_NAME);

/// Python: bot/compose/fallback.py:3251 `_asked_about_the_day_they_had(clean)`
pub fn _asked_about_the_day_they_had(clean: &str) -> bool {
    if HURT_IN_THE_LINE.is_match(clean) || LOOKED_UP.is_match(clean) {
        return false;
    }
    if WAS_THEY_THERE.match_(clean).is_some() || WAS_IT_ANY_GOOD.match_(clean).is_some() || WHO_WAS_THERE.match_(clean).is_some() {
        return true;
    }
    if let Some(went) = DID_THEY_GO.match_(clean) {
        let verb = clean[went.start()..went.end()].trim_end_matches(text::is_space);
        if verb.ends_with("think") {
            if verb.starts_with("do") || verb.starts_with("does") {
                let said = &clean[went.end()..];
                if !A_LOOK_BACK.is_match(said) || !A_THING_OR_A_PERSON.is_match(said) {
                    return false;
                }
            }
        }
        return true;
    }
    if let Some(m) = WENT_AND_WERE_ASKED.match_(clean) {
        if PEOPLE_WHO_WENT.contains(m.name_str("noun")) {
            return true;
        }
    }
    let Some(named) = WAS_A_NAMED_ONE_THERE.match_(clean) else { return false };
    let who = named.name_str("who");
    let also = named.name("also");
    if NOT_A_NAME.contains(who) || also.is_some_and(|a| !a.is_empty() && NOT_A_NAME.contains(a)) {
        return false;
    }
    true
}

regexes!(BARE_PICK, BARE_PICK_LOST, BARE_PICK_LOST_FIRST, BARE_PICK_START, BARE_PICK_START_CLAUSE);
pools!(LOST_IN_A_THREAD);

/// Python: bot/compose/fallback.py:3342 `_a_lost_ask(clean)`
pub fn _a_lost_ask(clean: &str) -> bool {
    BARE_PICK_LOST.match_(clean).is_some() || BARE_PICK_LOST_FIRST.match_(clean).is_some()
}

/// Python: bot/compose/fallback.py:3394 `_a_start_ask(clean)`
pub fn _a_start_ask(clean: &str) -> bool {
    BARE_PICK_START.match_(clean).is_some() || BARE_PICK_START_CLAUSE.is_match(clean)
}

/// Python: bot/compose/fallback.py:3400 `_said_what_it_is_about(memory)` (never called on the trace corpus)
/// True when a line before this one gave a reply something to name.
pub fn _said_what_it_is_about(memory: &Memory) -> bool {
    for turn in last_n(&memory.turns, 2) {
        let said = text::squash(&lower(&turn.user));
        if said.split(' ').filter(|w| w.chars().count() > 2).count() >= 3 {
            return true;
        }
    }
    false
}

// ---- fallback.py:3424-3990: the lost ask, the bare pick, asking for a person or a hand ----

pools!(START_ASK, BARE_PICK_AFTER_NEWS, BARE_PICK_DO);

/// Python's `try: ... except Exception: ...`: a panic is caught and given back as None, except
/// an UNPORTED one (a function another workstream has not ported yet), which keeps unwinding so
/// the checkers count it.
fn py_try<T>(f: impl FnOnce() -> T) -> Option<T> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(v) => Some(v),
        Err(e) => {
            let msg = e
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| e.downcast_ref::<&str>().copied())
                .unwrap_or("");
            if msg.starts_with("UNPORTED") {
                std::panic::resume_unwind(e)
            }
            None
        }
    }
}

/// Python: bot/compose/fallback.py:3424 `the_pool_for_a_lost_ask(memory, clean='')`
/// Which offer "i dont know what to do" gets: the news one, the thread one, or the bare one.
pub fn the_pool_for_a_lost_ask(memory: &Memory, clean: &str) -> Vec<String> {
    if _news_above(memory, false) {
        return BARE_PICK_AFTER_NEWS.clone();
    }
    if _a_start_ask(clean) {
        return START_ASK.clone();
    }
    if _said_what_it_is_about(memory) {
        return LOST_IN_A_THREAD.clone();
    }
    BARE_PICK_DO.clone()
}

/// Python: bot/compose/fallback.py:3435 `the_lost_ask_reply(memory, clean, repaired=None)`
/// The offer for "i dont know what to do", or None when this line is not that ask.
pub fn the_lost_ask_reply(memory: &Memory, clean: &str, repaired: &str) -> Option<String> {
    if !the_advice(memory).is_empty() {
        return None;
    }
    if !(_a_lost_ask(clean) || _a_lost_ask(repaired) || _a_start_ask(clean) || _a_start_ask(repaired)) {
        return None;
    }
    let pick = the_pool_for_a_lost_ask(memory, if _a_start_ask(clean) { clean } else { "" });
    if pick.is_empty() {
        return None;
    }
    choose(&pick, memory, clean)
}

regexes!(WEATHER_COLD, WEATHER_HOT, WEATHER_WET, HEAVY_ABOVE, WORK_NEWS);

/// Python: bot/compose/fallback.py:3499 `_news_above(memory, heavy=False)`
/// True when the line before this one said something heavy.
pub fn _news_above(memory: &Memory, heavy: bool) -> bool {
    if _in_a_care_thread(memory) {
        return true;
    }
    for turn in last_n(&memory.turns, 2) {
        let low = text::squash(&lower(&turn.user));
        if low.is_empty() {
            continue;
        }
        if HEAVY_ABOVE.is_match(&low) {
            return true;
        }
        let news = if heavy {
            crate::compose::badnews::reads_as_heavy_news(&low)
        } else {
            crate::compose::badnews::reads_as_bad_news(&low) && !WORK_NEWS.is_match(&low)
        };
        if news {
            return true;
        }
        if py_try(|| crate::compose::loss::reads_as_loss(&low, None, false)) == Some(true) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:3535 `_weather_above(memory)` (never called on the trace corpus)
/// The weather the person named in the last two lines, as "cold", "hot", "wet" or None.
pub fn _weather_above(memory: &Memory) -> Option<String> {
    for turn in last_n(&memory.turns, 2).iter().rev() {
        let said = &turn.user;
        if said.is_empty() {
            continue;
        }
        if WEATHER_WET.is_match(said) {
            return Some("wet".into());
        }
        if WEATHER_COLD.is_match(said) {
            return Some("cold".into());
        }
        if WEATHER_HOT.is_match(said) {
            return Some("hot".into());
        }
    }
    None
}

regexes!(OCCASION_ABOVE, THE_DAY_IS_NAMED, AN_OCCASION_AHEAD, TRIP_ABOVE);

/// Python: bot/compose/fallback.py:3560 `_an_occasion_above(memory)` (never called on the trace corpus)
pub fn _an_occasion_above(memory: &Memory) -> bool {
    last_n(&memory.turns, 2).iter().any(|t| OCCASION_ABOVE.is_match(&t.user))
}

/// Python: bot/compose/fallback.py:3591 `_an_occasion_in_the_thread(memory, back=4)`
pub fn _an_occasion_in_the_thread(memory: &Memory, back: i64) -> bool {
    last_n(&memory.turns, back).iter().any(|t| AN_OCCASION_AHEAD.is_match(&t.user))
}

/// Python: bot/compose/fallback.py:3623 `_a_trip_above(memory)`
pub fn _a_trip_above(memory: &Memory) -> bool {
    last_n(&memory.turns, 2).iter().any(|t| TRIP_ABOVE.is_match(&t.user))
}

word_sets!(BARE_PICK_NEEDS_THE_THING, THREAD_NOTHING);

/// Python: bot/compose/fallback.py:3647 `_a_subject_above(memory)` (never called on the trace corpus)
/// True when the last line the person sent named a thing, so a bare ask is a follow-up. (The
/// second of the two definitions in the Python file, the one the module keeps.)
pub fn _a_subject_above(memory: &Memory) -> bool {
    for turn in last_n(&memory.turns, 2) {
        let said = &turn.user;
        if text::strip(said).is_empty() {
            continue;
        }
        let words = az_words(&lower(said));
        if words.iter().any(|w| !THREAD_NOTHING.contains(text::strip_chars(w, "'")) && w.chars().count() > 2) {
            return true;
        }
    }
    false
}

regexes!(_AFTER_A_CLAUSE);

/// Python: bot/compose/fallback.py:3676 `_the_ask_clause(clean, memory=None)`
/// The ask a line ends with when something sits in front of it, or "" when nothing does.
pub fn _the_ask_clause(clean: &str, memory: &Memory) -> String {
    let said = text::squash(&lower(clean));
    if said.is_empty() {
        return String::new();
    }
    let mut ask = String::new();
    if let Some(reading) = &memory.understood {
        let focus = text::split(&reading.focus.join(" "));
        let joined = focus.join(" ");
        if focus.len() >= 3 && said.ends_with(&joined) && joined != said {
            ask = joined;
        }
    }
    if ask.is_empty() {
        ask = _AFTER_A_CLAUSE.search(&said).map(|m| text::squash(m.str(1))).unwrap_or_default();
    }
    if text::split_ref(&ask).len() >= 3 { ask } else { String::new() }
}

static BARE_PICK_KIND: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.BARE_PICK_KIND").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
pools!(BARE_PICK_WHICH, BARE_PICK_WEAR, BARE_PICK_WEAR_WET, BARE_PICK_WEAR_COLD, BARE_PICK_WEAR_HOT, BARE_PICK_BRING, BARE_PICK_TAKE);

/// Python: bot/compose/fallback.py:3693 `_bare_pick(clean, memory=None)`
/// The pool for a line that asks the bot to choose and names nothing to choose between, or None.
/// (The skeleton's `()` return could not carry the pool.) Python tells the pools apart by
/// identity (`pool is BARE_PICK_WEAR`); every pool here has its own lines, so equality is the same.
pub fn _bare_pick(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let mut found = BARE_PICK.match_(clean);
    let mut from_the_clause = false;
    let ask;
    if found.is_none() {
        ask = _the_ask_clause(clean, memory);
        found = if ask.is_empty() { None } else { BARE_PICK.match_(&ask) };
        from_the_clause = found.is_some();
    }
    let found = found?;
    if lower(text::strip(found.as_str())).starts_with("which") {
        return Some(BARE_PICK_WHICH.clone());
    }
    let verb = text::squash(&lower(found.name_str("verb")));
    let pool = BARE_PICK_KIND.iter().find(|(k, _)| *k == verb).map(|(_, v)| v.clone()).unwrap_or_else(|| BARE_PICK_DO.clone());
    if from_the_clause && (OCCASION_ABOVE.is_match(clean) || THE_DAY_IS_NAMED.is_match(clean) || _an_occasion_in_the_thread(memory, 4)) {
        return None;
    }
    if from_the_clause && pool == *BARE_PICK_DO {
        return None;
    }
    if BARE_PICK_NEEDS_THE_THING.contains(&verb) && _a_subject_above(memory) {
        return None;
    }
    if pool == *BARE_PICK_WEAR && !_an_occasion_above(memory) {
        match _weather_above(memory).as_deref() {
            Some("wet") => return Some(BARE_PICK_WEAR_WET.clone()),
            Some("cold") => return Some(BARE_PICK_WEAR_COLD.clone()),
            Some("hot") => return Some(BARE_PICK_WEAR_HOT.clone()),
            _ => {}
        }
    }
    if pool == *BARE_PICK_BRING && _a_trip_above(memory) {
        return Some(BARE_PICK_TAKE.clone());
    }
    if pool == *BARE_PICK_DO && _news_above(memory, true) {
        return Some(BARE_PICK_AFTER_NEWS.clone());
    }
    Some(pool)
}

regexes!(HUMAN_LEAD, HUMAN_QUOTED, HUMAN_HELPS, HUMAN_TAIL, HUMAN_TALK_ASK, HUMAN_TO_ME, HUMAN_OTHER_END, HAND_ASK);
word_sets!(HUMAN_EMPTY);

/// Python: bot/compose/fallback.py:3792 `_opens_the_line(prefix)` (never called on the trace corpus)
/// True when what comes before the human/person ask is only a greeting or a question opener.
pub fn _opens_the_line(prefix: &str) -> bool {
    HUMAN_LEAD.match_(prefix).is_some() || HUMAN_QUOTED.is_match(prefix)
}

/// Python: bot/compose/fallback.py:3835 `_asks_elsewhere(tail)` (never called on the trace corpus)
pub fn _asks_elsewhere(tail: &str) -> bool {
    if HUMAN_HELPS.match_(tail).is_some() {
        return false;
    }
    if HUMAN_TAIL.match_(tail).is_some() {
        return false;
    }
    if HUMAN_OTHER_END.match_(tail).is_some() {
        return false;
    }
    rx("[a-z]+").findall_str(&lower(tail)).iter().any(|w| !HUMAN_EMPTY.contains(w))
}

/// Python: bot/compose/fallback.py:3846 `_asks_for_a_person_to_talk_to(clean)`
pub fn _asks_for_a_person_to_talk_to(clean: &str) -> bool {
    let Some(matched) = HUMAN_TALK_ASK.search(clean) else { return false };
    if !_opens_the_line(&clean[..matched.start()]) {
        return false;
    }
    let tail = &clean[matched.end()..];
    HUMAN_TAIL.match_(tail).is_some() || HUMAN_TO_ME.match_(tail).is_some() || HUMAN_OTHER_END.match_(tail).is_some()
}

/// Python: bot/compose/fallback.py:3887 `_asks_for_help(clean, understood=None)`
/// A bare ask for a hand as the whole line, commas and all ("hi, can you help me").
pub fn _asks_for_help(clean: &str, understood: Option<&Understood>) -> bool {
    let mut lines = vec![clean.to_string()];
    if let Some(u) = understood {
        if !u.plain.is_empty() && u.plain != clean {
            lines.push(u.plain.clone());
        }
    }
    for line in &lines {
        let bare = text::squash(&rx("[^a-z0-9 ]+").sub(" ", &lower(line)));
        if HAND_ASK.match_(&bare).is_some() {
            return true;
        }
    }
    false
}

// ---- fallback.py:4158-4316: warm lines that only say what was said, the occasion, a slip ----

static DETAILS_IN_A_WARM_LINE: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.DETAILS_IN_A_WARM_LINE")
        .items()
        .iter()
        .map(|p| (p.items()[0].str().to_string(), p.items()[1].strs()))
        .collect()
});

/// Python: bot/compose/fallback.py:4151 `_says_only_what_they_said(lines, clean)`
/// The warm lines whose own detail the person's line carries, or the plain ones if none do.
pub fn _says_only_what_they_said(lines: &[String], clean: &str) -> Vec<String> {
    let said = az_set(&lower(clean));
    let unsupported = |line: &String| -> bool {
        let low = lower(line);
        for (phrase, support) in DETAILS_IN_A_WARM_LINE.iter() {
            if low.contains(phrase.as_str()) {
                return !support.iter().any(|w| said.contains(w));
            }
        }
        false
    };
    let kept: Vec<String> = lines.iter().filter(|l| !unsupported(l)).cloned().collect();
    if !kept.is_empty() {
        return kept;
    }
    lines.to_vec()
}

regexes!(HAPPENING_NOW, DATE_GIVEN, _APART, _PLACES_THEM);
word_sets!(_SHE, _HE);

/// Python: bot/compose/fallback.py:4222 `_the_pronoun(clean)` (never called on the trace corpus)
/// her or him for the one person the day belongs to, "" when the line names none or both.
pub fn _the_pronoun(clean: &str) -> String {
    let words = az_words(&lower(clean).replace('\u{2019}', "'"));
    let (mut she, mut he) = (false, false);
    for word in &words {
        let cut = &word[..word.len().saturating_sub(1)];
        let stem: &str = if word.ends_with('s') && (_SHE.contains(cut) || _HE.contains(cut)) { cut } else { word };
        if _SHE.contains(stem) {
            she = true;
        } else if _HE.contains(stem) {
            he = true;
        }
    }
    if she && !he {
        return "her".into();
    }
    if he && !she {
        return "him".into();
    }
    String::new()
}

static SOMEONE_ELSES_DAY: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.SOMEONE_ELSES_DAY").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
static OCCASION: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.OCCASION").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});

/// Python: bot/compose/fallback.py:4239 `_occasion_pool(occasion, clean)` (never called on the trace corpus)
/// The lines for this occasion (a name such as "birthday"), as a tuple value.
pub fn _occasion_pool(occasion: &Value, clean: &str) -> Value {
    Value::Tuple(std::sync::Arc::new(occasion_pool(occasion.str(), clean).into_iter().map(Value::from).collect()))
}

/// `_occasion_pool` with the occasion as a name and the pool as lines.
pub fn occasion_pool(occasion: &str, clean: &str) -> Vec<String> {
    if crate::compose::goodnews::someone_elses_occasion(clean) {
        if let Some((_, pool)) = SOMEONE_ELSES_DAY.iter().find(|(k, _)| k == occasion) {
            if !pool.is_empty() {
                let apart = _APART.is_match(clean);
                let mut kept: Vec<String> = pool.iter().filter(|l| apart || !_PLACES_THEM.is_match(l)).cloned().collect();
                let pronoun = _the_pronoun(clean);
                if !pronoun.is_empty() {
                    kept = kept.iter().map(|l| rx(r"\bthem\b").sub(&pronoun, l)).collect();
                }
                return if kept.is_empty() { pool.clone() } else { kept };
            }
        }
    }
    let pool = OCCASION
        .iter()
        .find(|(k, _)| k == occasion)
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| panic!("KeyError: {occasion:?} (OCCASION)"));
    let mut drop: Vec<&str> = Vec::new();
    if HAPPENING_NOW.is_match(clean) {
        drop.push("when it comes");
    }
    if DATE_GIVEN.is_match(clean) {
        drop.push("how far off");
    }
    if !drop.is_empty() {
        let kept: Vec<String> = pool.iter().filter(|l| !drop.iter().any(|p| lower(l).contains(p))).cloned().collect();
        if !kept.is_empty() {
            return kept;
        }
    }
    pool
}

regexes!(
    SERIOUS_NEWS,
    BABY_LOSS,
    SELF_BLAME_ASKS,
    A_VERDICT_ON_THE_PERSON,
    GOING_TO_DIE,
    HARM_IN_A_LAUGH,
    LAUGH_STOPPED,
    A_CROWD_LAUGHED,
    A_PERSON_LAUGHED,
    A_OWN_LAUGH,
    LAUGHED_AT_THEM,
    SOMEONE_ELSES_SHAME,
    DIE_OF_IT,
    THE_GROUND_OPENS,
    EMBARRASSMENT_WORD,
    THE_MOMENT_IS_THEIRS,
    A_SLIP_IS_NOT_A_FALL,
    A_SOCIAL_SLIP,
    A_SLIP,
    IN_A_ROOM,
);
pools!(LAUGHED_AT, EMBARRASSED);

/// Python: bot/compose/fallback.py:4265 `_their_own_slip(clean, memory=None)`
/// Which pool an awkward moment gets: "at them" or "of their own", or None. (The skeleton's
/// `()` return could not carry the pool.)
pub fn _their_own_slip(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    if clean.contains('?') {
        return None;
    }
    if HEAVY_ABOVE.is_match(clean) || SERIOUS_NEWS.is_match(clean) || BABY_LOSS.is_match(clean) {
        return None;
    }
    if SELF_BLAME_ASKS.is_match(clean) || A_VERDICT_ON_THE_PERSON.is_match(clean) {
        return None;
    }
    if GOING_TO_DIE.is_match(clean) {
        return None;
    }
    if HARM_IN_A_LAUGH.is_match(clean) || LAUGH_STOPPED.is_match(clean) {
        return None;
    }
    if _in_a_care_thread(memory) {
        return None;
    }
    if A_CROWD_LAUGHED.is_match(clean) {
        return Some(LAUGHED_AT.clone());
    }
    if A_PERSON_LAUGHED.is_match(clean) || A_OWN_LAUGH.is_match(clean) {
        return None;
    }
    if LAUGHED_AT_THEM.is_match(clean) {
        return Some(LAUGHED_AT.clone());
    }
    if SOMEONE_ELSES_SHAME.is_match(clean) {
        return None;
    }
    if DIE_OF_IT.is_match(clean) || THE_GROUND_OPENS.is_match(clean) {
        return Some(EMBARRASSED.clone());
    }
    if EMBARRASSMENT_WORD.is_match(clean) && THE_MOMENT_IS_THEIRS.is_match(clean) {
        return Some(EMBARRASSED.clone());
    }
    if A_SLIP_IS_NOT_A_FALL.is_match(clean) {
        return None;
    }
    if A_SOCIAL_SLIP.is_match(clean) || (A_SLIP.is_match(clean) && IN_A_ROOM.is_match(clean)) {
        return Some(EMBARRASSED.clone());
    }
    None
}

// ---- fallback.py:4374-4720: serious news about someone close, a death and the words, results ----

regexes!(CLOSE_OWNER, NOT_THEIR_NEWS, NOT_YET_NEWS);

/// Python: bot/compose/fallback.py:4371 `reads_as_someone_close_is_ill(clean)` (never called on the trace corpus)
/// Whether a cleaned, lowercased line is serious news about someone the person loves.
pub fn reads_as_someone_close_is_ill(clean: &str) -> bool {
    if clean.is_empty() || NOT_THEIR_NEWS.is_match(clean) {
        return false;
    }
    if !CLOSE_OWNER.is_match(clean) {
        return false;
    }
    let Some(serious) = SERIOUS_NEWS.search(clean) else { return false };
    if let Some(doubt) = NOT_YET_NEWS.search(clean) {
        if doubt.start() < serious.start() {
            return false;
        }
    }
    true
}

regexes!(TELLING_A_CHILD, A_DEATH_FROM_TODAY, A_CHILD_DIED, A_DEATH_IN_THE_LINE, A_SUICIDE_IN_THE_LINE);

/// Python: bot/compose/fallback.py:4501 `a_fresh_death_and_the_words(clean)`
pub fn a_fresh_death_and_the_words(clean: &str) -> bool {
    let t = text::squash(&lower(clean));
    TELLING_A_CHILD.is_match(&t)
        && A_DEATH_IN_THE_LINE.is_match(&t)
        && A_DEATH_FROM_TODAY.is_match(&t)
        && !A_CHILD_DIED.is_match(&t)
        && !A_SUICIDE_IN_THE_LINE.is_match(&t)
}

/// Python: bot/compose/fallback.py:4517 `asks_what_to_tell_a_child(clean)`
pub fn asks_what_to_tell_a_child(clean: &str) -> bool {
    let t = text::squash(&lower(clean));
    TELLING_A_CHILD.is_match(&t) && A_DEATH_IN_THE_LINE.is_match(&t) && !A_DEATH_FROM_TODAY.is_match(&t) && !A_CHILD_DIED.is_match(&t)
}

regexes!(_RESULT, _WAITING, _DOUBT, _GUESSED_RESULT, _RESOLVED, _A_TIME, _ANSWER_GOOD, _ANSWER_BAD, _LANDED);

/// Python: bot/compose/fallback.py:4622 `_an_open_result(clean)`
/// True when the line reports a result the person has not heard yet.
pub fn _an_open_result(clean: &str) -> bool {
    if _RESOLVED.is_match(clean) {
        return false;
    }
    if _WAITING.is_match(clean) {
        return true;
    }
    _RESULT.is_match(clean) && _DOUBT.is_match(clean)
}

/// Python: bot/compose/fallback.py:4651 `_the_news_landed(clean)`
/// "bad", "good" or "landed" when the answer came back, None while it has not. (The skeleton's
/// `()` return could not carry the answer.)
pub fn _the_news_landed(clean: &str) -> Option<String> {
    if !_LANDED.is_match(clean) {
        return None;
    }
    if _ANSWER_BAD.is_match(clean) {
        return Some("bad".into());
    }
    if _ANSWER_GOOD.is_match(clean) {
        return Some("good".into());
    }
    Some("landed".into())
}

/// Python: bot/compose/fallback.py:4668 `reads_as_a_result_line(text)`
pub fn reads_as_a_result_line(text_: &str) -> bool {
    let clean = _plain(&_matchable(text_));
    if clean.is_empty() {
        return false;
    }
    _the_news_landed(&clean).is_some() || _an_open_result(&clean) || _a_guessed_result(&clean)
}

/// Python: bot/compose/fallback.py:4683 `_a_guessed_result(clean)`
pub fn _a_guessed_result(clean: &str) -> bool {
    _GUESSED_RESULT.is_match(clean)
}

/// Python: bot/compose/fallback.py:4688 `_a_time_in_it(clean)` (never called on the trace corpus)
pub fn _a_time_in_it(clean: &str) -> bool {
    _A_TIME.is_match(clean)
}

regexes!(A_COST, SOMEBODY_ASKS, A_ROLE_IN_SOMEBODY_ELSES_DAY);

/// Python: bot/compose/fallback.py:4712 `_a_cost_somebody_asks(clean)`
/// True for a bill somebody else has put on the person for a shared occasion.
pub fn _a_cost_somebody_asks(clean: &str) -> bool {
    let t = text::squash(&lower(clean));
    if t.is_empty() || t.contains('?') || t.split(' ').count() > 24 {
        return false;
    }
    if !A_COST.is_match(&t) {
        return false;
    }
    if !(SOMEBODY_ASKS.is_match(&t) || A_ROLE_IN_SOMEBODY_ELSES_DAY.is_match(&t)) {
        return false;
    }
    !rx(r"\bmy\s+(?:own\s+)?(?:wedding|holiday|party|trip)\b").is_match(&t)
}

// ---- fallback.py:4867-5590: the day remarked on, a plan dropped, no words, invitations, limits ----

static DAY_WORDS: LazyLock<Vec<(String, HashSet<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.DAY_WORDS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_set())).collect()
});
static THE_DAY: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.THE_DAY").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
static THE_DAY_AHEAD: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.THE_DAY_AHEAD").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
static DAY_SAID_AS_A_KIND: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.DAY_SAID_AS_A_KIND").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].str().to_string())).collect()
});
word_sets!(
    DAY_GLUE,
    NEGATIONS,
    NEGATION_SOFTENERS,
    DAY_TO_COME,
    DAY_VERDICTS,
    DAY_ASKS,
    DAY_THEIR_OWN_DAY,
    DAY_JOINED_ON,
    DAY_NEEDS_A_DAY_NAMED,
    DAY_NAMED,
);
pools!(DAY_ON_THE_WAY, DAY_AS_ANYTHING, DAY_TO_COME_PHRASES, DAY_TO_COME_PAST, DAY_NOT_THE_DAY, DAY_IN_HERE);
regexes!(DAY_MET_ME);

/// `dict.get(key)` on a table of pools.
fn pool_of(table: &[(String, Vec<String>)], key: &str) -> Option<Vec<String>> {
    table.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

/// `re.sub(r"\b" + r"\s+".join(phrase.split()) + r"\b", repl, line)`.
fn sub_phrase(line: &str, phrase: &str, repl: &str) -> String {
    let pattern = format!(r"\b{}\b", text::split_ref(phrase).join(r"\s+"));
    rx(&pattern).sub(repl, line)
}

/// Python: bot/compose/fallback.py:4867 `_the_day_described(clean)`
/// The pool for a line that says what the day is like, or None. (The skeleton's `()` return
/// could not carry the pool.)
pub fn _the_day_described(clean: &str) -> Option<Vec<String>> {
    let mut line = text::squash(&lower(clean));
    if line.is_empty() || line.contains('?') {
        return None;
    }
    line = rx(r"^what\s+an?\s+").sub("", &line);
    if DAY_MET_ME.match_(&line).is_some() {
        line = DAY_MET_ME.sub("", &line);
        for phrase in DAY_ON_THE_WAY.iter() {
            line = sub_phrase(&line, phrase, " ");
        }
        line = text::squash(&line);
    }
    for phrase in DAY_AS_ANYTHING.iter() {
        line = sub_phrase(&line, phrase, " ");
    }
    for (phrase, word) in DAY_SAID_AS_A_KIND.iter() {
        line = sub_phrase(&line, phrase, word);
    }
    line = text::squash(&line);
    let words: Vec<String> = text::split_ref(&line)
        .iter()
        .map(|w| w.trim_matches(['.', ',', '!', ';', ':', '\'', '\u{2019}', '"']).to_string())
        .filter(|w| !w.is_empty())
        .collect();
    if words.is_empty() || words.len() > 8 {
        return None;
    }
    if DAY_ASKS.contains(&words[0]) || DAY_THEIR_OWN_DAY.contains(&words[0]) || DAY_JOINED_ON.contains(&words[0]) {
        return None;
    }
    if DAY_NOT_THE_DAY.iter().any(|p| line.contains(p.as_str())) {
        return None;
    }
    if DAY_IN_HERE.iter().any(|p| line.contains(p.as_str())) {
        return None;
    }
    let in_a_pool = |w: &str| DAY_WORDS.iter().any(|(_, pool)| pool.contains(w));
    if !words.iter().any(|w| DAY_NAMED.contains(w) || DAY_TO_COME.contains(w)) {
        let day_words: Vec<&String> = words.iter().filter(|w| in_a_pool(w)).collect();
        if !day_words.is_empty() && day_words.iter().all(|w| DAY_NEEDS_A_DAY_NAMED.contains(*w)) {
            return None;
        }
    }
    if words.iter().any(|w| !DAY_GLUE.contains(w) && !in_a_pool(w)) {
        return None;
    }
    if DAY_TO_COME_PAST.iter().any(|p| line.contains(p.as_str())) {
        return None;
    }
    for (index, word) in words.iter().enumerate() {
        if index > 0 && index + 1 < words.len() && in_a_pool(word) {
            if words[index - 1] == "the" && (words[index + 1] == "one" || words[index + 1] == "ones") {
                return None;
            }
        }
    }
    let day_words: Vec<&String> = words.iter().filter(|w| in_a_pool(w)).collect();
    if (words.iter().any(|w| w == "was") || words.iter().any(|w| w == "were"))
        && !day_words.is_empty()
        && day_words.iter().all(|w| DAY_VERDICTS.contains(*w))
        && !words.iter().any(|w| DAY_TO_COME.contains(w))
    {
        return None;
    }
    let ruled_out = |index: usize| -> bool {
        let mut back = index as i64 - 1;
        while back >= 0 && NEGATION_SOFTENERS.contains(&words[back as usize]) {
            back -= 1;
        }
        back >= 0 && NEGATIONS.contains(&words[back as usize])
    };
    let (mut best, mut best_count): (Option<&str>, usize) = (None, 0);
    let (mut ruled, mut ruled_count): (Option<&str>, usize) = (None, 0);
    for (kind, pool) in DAY_WORDS.iter() {
        let count = words.iter().enumerate().filter(|(i, w)| pool.contains(*w) && !ruled_out(*i)).count();
        if count > best_count {
            best = Some(kind);
            best_count = count;
        }
        let cleared = words.iter().enumerate().filter(|(i, w)| pool.contains(*w) && ruled_out(*i)).count();
        if cleared > ruled_count {
            ruled = Some(kind);
            ruled_count = cleared;
        }
    }
    let Some(best) = best else {
        if ruled.is_some() {
            return pool_of(&THE_DAY, "mild");
        }
        return None;
    };
    if words.iter().any(|w| DAY_TO_COME.contains(w)) || DAY_TO_COME_PHRASES.iter().any(|p| line.contains(p.as_str())) {
        return pool_of(&THE_DAY_AHEAD, best).filter(|p| !p.is_empty()).or_else(|| pool_of(&THE_DAY, best));
    }
    pool_of(&THE_DAY, best)
}

/// Python: bot/compose/fallback.py:4961 `the_day_remarked(text)`
/// The pool for a remark about the day as it is or as it is coming, or None. (The skeleton's
/// `()` return could not carry the pool.)
pub fn the_day_remarked(text_: &str) -> Option<Vec<String>> {
    let line = text::squash(&lower(text_));
    let pool = _the_day_described(&line)?;
    if pool.is_empty() {
        return None;
    }
    if rx(r"\b(?:was|were)\b").is_match(&line) {
        return None;
    }
    Some(pool)
}

regexes!(
    DECLINE_ABOUT_THE_BOT,
    DECLINE_SOMEONE_ELSE,
    DECLINE_VERB,
    DECLINE_MUST,
    DECLINE_ASK,
    DECLINE_TIME,
    DECLINE_OCCASION,
    DECLINE_DATE,
    DECLINE_OTHER,
    DECLINE_OTHER_SUBJECT,
    DECLINE_BE_THERE,
    DECLINE_AFTER,
    BODY_SAYS_WHY_PHRASE,
    BABY_ON_THE_WAY,
    VERB_PARTICLE,
    LATE_NOT_A_TIME,
    PLAN_ON_THE_TABLE,
    LINE_CANCELS,
);
word_sets!(BODY_SAYS_WHY);

/// Python: bot/compose/fallback.py:5070 `_the_reason_is_on_the_body(text)` (never called on the trace corpus)
pub fn _the_reason_is_on_the_body(text_: &str) -> bool {
    if text_.is_empty() {
        return false;
    }
    let words: Vec<String> = az_set(text_).into_iter().collect();
    words.iter().any(|w| BODY_SAYS_WHY.contains(w))
        || _worry_is_about_the_body(text_, &words)
        || LATE_NOT_A_TIME.is_match(text_)
        || BABY_ON_THE_WAY.is_match(text_)
        || BODY_SAYS_WHY_PHRASE.is_match(text_)
}

/// Python: bot/compose/fallback.py:5104 `_the_line_cancels(line)` (never called on the trace corpus)
pub fn _the_line_cancels(line: &str) -> bool {
    LINE_CANCELS.is_match(line)
}

/// Python: bot/compose/fallback.py:5109 `_a_plan_on_the_table(memory)` (never called on the trace corpus)
pub fn _a_plan_on_the_table(memory: &Memory) -> bool {
    last_n(&memory.turns, 3).iter().any(|t| !t.user.is_empty() && PLAN_ON_THE_TABLE.is_match(&t.user))
}

/// Python: bot/compose/fallback.py:5124 `_hesitant_news_before(memory, turns=3)` (never called on the trace corpus)
/// (`turns` is the count of the person's lines to read, as in Python.)
pub fn _hesitant_news_before(memory: &Memory, turns: i64) -> bool {
    last_n(&memory.turns, turns).iter().any(|t| {
        let said = lower(&t.user);
        !said.is_empty() && crate::compose::badnews::reads_as_bad_news(&said)
    })
}

/// Python: bot/compose/fallback.py:5140 `_a_hard_week(memory)` (never called on the trace corpus)
pub fn _a_hard_week(memory: &Memory) -> bool {
    _in_a_loss_thread(memory) || _a_loss_recently(memory) || _hesitant_news_before(memory, 3)
}

/// Python: bot/compose/fallback.py:5150 `_the_turn_before_was_a_heavy_one(memory, line='')` (never called on the trace corpus)
pub fn _the_turn_before_was_a_heavy_one(memory: &Memory, line: &str) -> bool {
    let plain = if line.is_empty() { String::new() } else { _plain(line) };
    if !plain.is_empty() && !(_a_declined_plan(&plain, true) && !DECLINE_TIME.is_match(&plain)) {
        return false;
    }
    let Some(last) = memory.turns.last() else { return false };
    let said = lower(text::strip(&last.user));
    if said.is_empty() {
        return false;
    }
    let read = py_try(|| crate::understanding::understand(&said, None));
    if let Some(read) = &read {
        if read.intent == "question" || read.intent == "ask" || read.kind == "question" {
            return false;
        }
    }
    if let Some(note) = memory.get("heavy_reply") {
        if note.is_dict() && note.get("turn").and_then(Value::as_int) == Some(memory.turns.len() as i64 - 1) {
            return true;
        }
    }
    if crate::compose::badnews::reads_like_a_symptom(&said) || _the_reason_is_on_the_body(&said) {
        return true;
    }
    let Some(read) = read else { return false };
    if read.intent == "feeling" && read.polarity < 0 {
        return true;
    }
    read.emotion.as_deref().is_some_and(|e| !e.is_empty()) || read.body.as_deref().is_some_and(|b| !b.is_empty())
}

/// The line `_a_declined_plan` and `_someone_else_dropped_it` read: no apostrophes, one space.
fn decline_line(clean: &str) -> String {
    text::squash(text::strip(&clean.replace('\u{2019}', "'").replace('\'', "")))
}

/// The first decline in the line that is not about the bot: (is it `DECLINE_MUST`, the match).
fn the_decline<'t>(line: &'t str) -> Option<(bool, Match<'t>)> {
    for (must, pattern) in [(false, &*DECLINE_VERB), (true, &*DECLINE_MUST)] {
        for m in pattern.finditer(line) {
            if DECLINE_ABOUT_THE_BOT.is_match(&line[..m.start()]) {
                continue;
            }
            return Some((must, m));
        }
    }
    None
}

/// Python: bot/compose/fallback.py:5207 `_a_declined_plan(clean, without_a_thing=False)`
/// True when the line is the person saying they cannot make a plan.
pub fn _a_declined_plan(clean: &str, without_a_thing: bool) -> bool {
    let line = decline_line(clean);
    if line.is_empty() || DECLINE_ASK.match_(&line).is_some() || line.ends_with('?') {
        return false;
    }
    let Some((must, found)) = the_decline(&line) else { return false };
    let group1 = found.str(1);
    if _the_reason_is_on_the_body(&line[..found.start()]) || _the_reason_is_on_the_body(&VERB_PARTICLE.sub(" ", group1)) {
        return false;
    }
    let stripped = group1.trim_matches([' ', ',', '.', '-', '!', '?']);
    let tail_owned = DECLINE_AFTER.sub("", stripped);
    let tail = tail_owned.trim_matches([' ', ',', '.', '-', '!', '?']);
    if tail.is_empty() {
        return true;
    }
    if tail.starts_with("for ") {
        let head = &line[..found.end() - group1.len()];
        if DECLINE_BE_THERE.is_match(head.trim_end_matches(text::is_space)) {
            return false;
        }
    }
    if must && !DECLINE_TIME.is_match(tail) {
        return false;
    }
    if DECLINE_DATE.is_match(tail) {
        return true;
    }
    if DECLINE_TIME.is_match(tail) {
        if DECLINE_OTHER.match_(tail).is_none() || DECLINE_OTHER_SUBJECT.match_(&line).is_none() {
            return true;
        }
    }
    if DECLINE_OTHER.match_(tail).is_some() {
        return false;
    }
    if tail == "it" || tail == "there" || tail == "along" {
        return true;
    }
    if without_a_thing {
        return false;
    }
    if ["to ", "for ", "it to ", "it on ", "it for ", "it at ", "it next "].iter().any(|p| tail.starts_with(p)) {
        return true;
    }
    DECLINE_OCCASION.is_match(tail)
}

/// Python: bot/compose/fallback.py:5275 `_someone_else_dropped_it(clean)` (never called on the trace corpus)
pub fn _someone_else_dropped_it(clean: &str) -> bool {
    let line = decline_line(clean);
    if line.is_empty() || !_a_declined_plan(clean, false) {
        return false;
    }
    match the_decline(&line) {
        Some((_, m)) => DECLINE_SOMEONE_ELSE.is_match(&line[..m.start()]),
        None => false,
    }
}

regexes!(NO_WORDS, NO_WORDS_NOT, NO_WORDS_TO, NO_WORDS_SHE, NO_WORDS_CROWD, NO_WORDS_ONE, NO_WORDS_CARE, NO_WORDS_ASK);
pools!(NO_WORDS_ASKING, NO_WORDS_AT_THE_CARE, NO_WORDS_SOMEONE, NO_WORDS_STUCK);

/// Python: bot/compose/fallback.py:5336 `_no_words(clean)`
/// The pool for a line that cannot find the words, or None. (The skeleton's `()` return could
/// not carry the pool.)
pub fn _no_words(clean: &str) -> Option<Vec<String>> {
    if !NO_WORDS.is_match(clean) || NO_WORDS_NOT.is_match(clean) {
        return None;
    }
    if NO_WORDS_ASK.is_match(clean) {
        return Some(NO_WORDS_ASKING.clone());
    }
    if NO_WORDS_CARE.is_match(clean) {
        return Some(NO_WORDS_AT_THE_CARE.clone());
    }
    if NO_WORDS_TO.is_match(clean) || NO_WORDS_SHE.is_match(clean) {
        if NO_WORDS_CROWD.is_match(clean) && !NO_WORDS_ONE.is_match(clean) {
            return None;
        }
        return Some(NO_WORDS_SOMEONE.clone());
    }
    Some(NO_WORDS_STUCK.clone())
}

regexes!(INVITE_TALK, INVITE_OPENER, INVITE_COMPANY, INVITE_NOT_A_DOING, INVITE_WHEN);
word_sets!(INVITE_NEVER, INVITE_FILLER, INVITE_PRONOUN, INVITE_ACTIVITY);

/// Python: bot/compose/fallback.py:5377 `_an_invitation(clean)`
/// 'talk', 'go' or None: which invitation this line is, if it is one. (The skeleton's `()`
/// return could not carry the answer.)
pub fn _an_invitation(clean: &str) -> Option<String> {
    let line = text::squash(&rx("[^a-z0-9 ']+").sub(" ", &lower(clean)));
    let split = text::split_ref(&line);
    if line.is_empty() || split.len() < 2 {
        return None;
    }
    if split.iter().any(|w| INVITE_NEVER.contains(*w)) {
        return None;
    }
    let talk_or_go = || Some(if INVITE_TALK.is_match(&line) { "talk".to_string() } else { "go".to_string() });
    if INVITE_COMPANY.match_(&line).is_some() {
        return talk_or_go();
    }
    let found = INVITE_OPENER.match_(&line)?;
    let rest = text::split_ref(found.str(1));
    let mut tail: Vec<&str> = rest.iter().copied().filter(|w| !INVITE_FILLER.contains(*w)).collect();
    if tail.is_empty() {
        tail = rest.iter().copied().filter(|w| !["tonight", "today", "tomorrow", "later", "now", "soon", "after"].contains(w)).collect();
    }
    let said = tail.join(" ");
    if tail.is_empty() || INVITE_NOT_A_DOING.match_(&said).is_some() || INVITE_WHEN.match_(&said).is_some() {
        return None;
    }
    if tail.iter().all(|w| INVITE_PRONOUN.contains(*w)) {
        return None;
    }
    if !tail.iter().any(|w| INVITE_ACTIVITY.contains(*w)) && !INVITE_TALK.is_match(&line) {
        return None;
    }
    talk_or_go()
}

regexes!(BOUND_SAID, NARROW_JOIN, A_FAULT);
static NARROW_AXES: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.NARROW_AXES")
        .items()
        .iter()
        .map(|p| (p.items()[0].str().to_string(), p.items()[1].as_re().unwrap()))
        .collect()
});
static NARROW_WORDS: LazyLock<i64> = LazyLock::new(|| data::get("bot.compose.fallback.NARROW_WORDS").as_int().unwrap());
static NARROW_SUBSTANCE: LazyLock<i64> = LazyLock::new(|| data::get("bot.compose.fallback.NARROW_SUBSTANCE").as_int().unwrap());
word_sets!(NARROW_ANSWERED);
static AXIS_SAID: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.AXIS_SAID").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_re().unwrap())).collect()
});
static AXIS_MIN: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.AXIS_MIN").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap())).collect()
});

/// Python: bot/compose/fallback.py:5449 `the_bound_said(clean)` (never called on the trace corpus)
/// The bound the line itself names, said back the way they said it, or "".
pub fn the_bound_said(clean: &str) -> String {
    let line = text::squash(text::strip(clean));
    let Some(found) = BOUND_SAID.search(&line) else { return String::new() };
    let rest = &line[found.start()..];
    let rest = rx(r"[^\w\s£$€.,-]").sub(" ", rest);
    let rest = rest.trim_matches([' ', '.', ',']);
    let words: Vec<&str> = text::split_ref(rest).into_iter().take(5).collect();
    words.join(" ").trim_end_matches([' ', '.', ',']).to_string()
}

/// Python: bot/compose/fallback.py:5461 `reads_as_a_narrowing(clean)`
/// The axis this line narrows on, or "" when it is not only a constraint.
pub fn reads_as_a_narrowing(clean: &str) -> String {
    the_limits_said(clean).into_iter().next().map(|(axis, _)| axis).unwrap_or_default()
}

/// Python: bot/compose/fallback.py:5484 `the_limits_said(clean)`
/// The limits a line of joined limits names, as `(axis, part)` pairs in the order a reply should
/// try them. (The skeleton's `Vec<String>` could not carry the pairs Python returns.)
pub fn the_limits_said(clean: &str) -> Vec<(String, String)> {
    let line = text::squash(text::strip(clean));
    if line.is_empty() || text::rstrip(&line).ends_with('?') || text::split_ref(&line).len() as i64 > *NARROW_WORDS {
        return Vec::new();
    }
    let parts: Vec<String> = NARROW_JOIN.split(&line).into_iter().filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return Vec::new();
    }
    let axes: Vec<String> = parts.iter().map(|p| _one_axis(p)).collect();
    if axes.iter().any(|a| a.is_empty()) {
        return Vec::new();
    }
    let mut with_bound = Vec::new();
    let mut without = Vec::new();
    for (axis, part) in axes.into_iter().zip(parts) {
        if the_bound_said(&part).is_empty() {
            without.push((axis, part));
        } else {
            with_bound.push((axis, part));
        }
    }
    with_bound.reverse();
    without.reverse();
    with_bound.extend(without);
    with_bound
}

/// Python: bot/compose/fallback.py:5514 `_one_axis(line)`
/// The one axis a piece of a line narrows on, or "".
pub fn _one_axis(line: &str) -> String {
    for (axis, pattern) in NARROW_AXES.iter() {
        if pattern.match_(line).is_some() {
            return axis.clone();
        }
    }
    String::new()
}

/// Python's `str.isupper()` for one character.
fn char_is_upper(c: char) -> bool {
    c.is_uppercase()
}

/// Python: bot/compose/fallback.py:5545 `_names_things_to_choose(said)` (never called on the trace corpus)
/// True when the answer puts two or more named things in front of the person, to pick from.
pub fn _names_things_to_choose(said: &str) -> bool {
    if !rx_flags_i(r"\bor\b").is_match(said) {
        return false;
    }
    let words = text::split_ref(said);
    let mut named: HashSet<&str> = HashSet::new();
    for (index, word) in words.iter().enumerate() {
        let bare = word.trim_matches(['.', ',', ';', ':', '!', '?', '*', '(', ')', '\u{2014}', '-', '\u{2019}', '\'', '"']);
        let mut chars = bare.chars();
        let Some(first) = chars.next() else { continue };
        let rest = chars.as_str();
        if bare.chars().count() < 2 || !char_is_upper(first) || !text::is_lower(rest) {
            continue;
        }
        if index == 0 {
            continue;
        }
        let before = text::rstrip(words[index - 1]);
        if before.ends_with(['.', '!', '?', ':']) && !before.ends_with("...") {
            continue;
        }
        named.insert(bare);
    }
    named.len() >= 2
}

/// `re.compile(pattern, re.I)` through the shared cache.
fn rx_flags_i(pattern: &str) -> &'static Re {
    crate::understanding::rx_flags(pattern, crate::py::re::I)
}

/// Python: bot/compose/fallback.py:5596 `the_answer_to_cut(memory, axis='')` (never called on the trace corpus)
/// The answer still on the screen (the last turn, as a dict of user, bot and module), or {}.
pub fn the_answer_to_cut(memory: &Memory, axis: &Value) -> Value {
    match answer_to_cut(memory, axis.as_str().unwrap_or("")) {
        Some(turn) => {
            let mut d = Value::dict();
            d.set_key("user", turn.user.as_str());
            d.set_key("bot", turn.bot.as_str());
            d.set_key("module", turn.module.as_deref().map(Value::from).unwrap_or(Value::None));
            d
        }
        None => Value::dict(),
    }
}

/// `the_answer_to_cut` as the turn itself (None for Python's `{}`).
pub fn answer_to_cut<'m>(memory: &'m Memory, axis: &str) -> Option<&'m Turn> {
    let last = memory.turns.last()?;
    if !last.module.as_deref().is_some_and(|m| NARROW_ANSWERED.contains(m)) {
        return None;
    }
    let said = &last.bot;
    if (text::split_ref(said).len() as i64) < *NARROW_SUBSTANCE {
        return None;
    }
    if text::rstrip(said).ends_with('?') {
        return None;
    }
    if !axis.is_empty() {
        let raised = match AXIS_SAID.iter().find(|(k, _)| k == axis) {
            Some((_, rx_)) => rx_.finditer(said).len() as i64,
            None => 0,
        };
        let min = AXIS_MIN.iter().find(|(k, _)| k == axis).map(|(_, n)| *n).unwrap_or(1);
        if raised < min && !_names_things_to_choose(said) {
            return None;
        }
    }
    if A_FAULT.is_match(&last.user) {
        return None;
    }
    Some(last)
}

// ---- fallback.py:5683-6127: reactions and verdicts on the thing before ----

/// `" ".join(str(turn.get("user") or "").lower().split())` for the person's last `n` lines.
fn last_lines(memory: &Memory, n: i64) -> Vec<String> {
    last_n(&memory.turns, n).iter().map(|t| text::squash(&lower(&t.user))).collect()
}

/// Python: bot/compose/fallback.py:5683 `_the_person_already_said_it(memory)` (never called on the trace corpus)
pub fn _the_person_already_said_it(memory: &Memory) -> bool {
    let last = crate::memory::last_user(memory, 1);
    let said = text::strip(last.as_str().unwrap_or(""));
    if said.is_empty() || text::rstrip(said).ends_with('?') {
        return false;
    }
    text::split_ref(said).len() >= 4
}

/// Python: bot/compose/fallback.py:5697 `_a_care_thread_to_leave_alone(memory)`
pub fn _a_care_thread_to_leave_alone(memory: &Memory) -> bool {
    if _in_a_care_thread(memory) || _in_sudden_loss_thread(memory) || _in_a_loss_thread(memory) {
        return true;
    }
    _the_last_few_lines(memory).iter().any(|said| baby_loss(said))
}

/// Python: bot/compose/fallback.py:5711 `_the_last_few_lines(memory)`
/// The person's own last few lines, as the thread left them.
pub fn _the_last_few_lines(memory: &Memory) -> Vec<String> {
    last_n(&memory.turns, 4).iter().filter(|t| !text::strip(&t.user).is_empty()).map(|t| text::squash(&lower(&t.user))).collect()
}

/// Python: bot/compose/fallback.py:5718 `baby_loss(clean)`
pub fn baby_loss(clean: &str) -> bool {
    BABY_LOSS.is_match(clean)
}

regexes!(VERDICT_SAID, VERDICT_BARE, JOKE_WANTED, JOKE_UNWANTED);
pools!(REACTION_OUCH);

/// Python: bot/compose/fallback.py:5756 `_the_answer_above_was_a_joke(memory)` (never called on the trace corpus)
pub fn _the_answer_above_was_a_joke(memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    let asked = text::squash(&lower(&last.user));
    if JOKE_UNWANTED.is_match(&asked) {
        return false;
    }
    JOKE_WANTED.is_match(&asked)
}

/// Python: bot/compose/fallback.py:5767 `_a_verdict_line(text, memory=None)`
/// The pool for a line that is only a verdict on the thing before it, or None. (The skeleton's
/// `()` return could not carry the pool.)
pub fn _a_verdict_line(text_: &str, memory: &Memory) -> Option<Vec<String>> {
    let t = text::squash(&lower(text_));
    let found = VERDICT_SAID.match_(&t);
    if found.is_none() && VERDICT_BARE.match_(&t).is_none() {
        return None;
    }
    if let Some(found) = &found {
        if found.name("noun") == Some("joke") && _the_answer_above_was_a_joke(memory) {
            return None;
        }
    }
    Some(REACTION_OUCH.clone())
}

regexes!(VERDICT_LINE, STATE_ABOVE, WAITING_NEWS, JOB_GONE_NEWS, GRIEF_ABOVE, VERDICT_POINTS_BACK, HEAVY_NEWS, HARD_WEEK);

/// Python: bot/compose/fallback.py:5807 `a_death_just_said(memory=None)` (never called on the trace corpus)
pub fn a_death_just_said(memory: &Memory) -> bool {
    for line in last_lines(memory, 2) {
        if line.is_empty() {
            continue;
        }
        if py_try(|| crate::compose::loss::reads_as_loss(&line, None, false)) == Some(true) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:5825 `a_verdict_on_the_person(memory=None)` (never called on the trace corpus)
pub fn a_verdict_on_the_person(memory: &Memory) -> bool {
    last_lines(memory, 2).iter().any(|l| !l.is_empty() && STATE_ABOVE.is_match(l))
}

/// Python: bot/compose/fallback.py:5848 `_the_last_two_lines(memory=None)` (never called on the trace corpus)
/// The person's own lines for this turn and the one before - never the bot's wording.
pub fn _the_last_two_lines(memory: &Memory) -> Vec<String> {
    last_lines(memory, 2).into_iter().filter(|l| !l.is_empty()).collect()
}

/// Python: bot/compose/fallback.py:5858 `_a_loss_in_the_same_breath(memory=None, lines=())` (never called on the trace corpus)
pub fn _a_loss_in_the_same_breath(memory: &Memory, lines: &[String]) -> bool {
    if a_death_just_said(memory) || a_heavy_verdict(memory) {
        return true;
    }
    for line in lines {
        if HEAVY_ABOVE.is_match(line) {
            return true;
        }
        let heavy = py_try(|| {
            crate::compose::badnews::reads_as_sudden_loss(line)
                || crate::compose::badnews::reads_as_the_leaving(line)
                || crate::compose::loss::reads_as_loss(line, None, false)
        });
        if heavy == Some(true) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:5879 `a_wait_with_no_loss(memory=None)` (never called on the trace corpus)
pub fn a_wait_with_no_loss(memory: &Memory) -> bool {
    let lines = _the_last_two_lines(memory);
    if !lines.iter().any(|l| WAITING_NEWS.is_match(l)) {
        return false;
    }
    !_a_loss_in_the_same_breath(memory, &lines)
}

/// Python: bot/compose/fallback.py:5887 `a_setback_with_no_loss(memory=None)` (never called on the trace corpus)
pub fn a_setback_with_no_loss(memory: &Memory) -> bool {
    let lines = _the_last_two_lines(memory);
    if !lines.iter().any(|l| JOB_GONE_NEWS.is_match(l)) {
        return false;
    }
    !_a_loss_in_the_same_breath(memory, &lines)
}

pools!(VERDICT_NUMB, VERDICT_QUEUE, VERDICT_SETBACK, VERDICT_HEAVY);

/// Python: bot/compose/fallback.py:5895 `_the_heavy_pool(memory=None)` (never called on the trace corpus)
/// The pool for a verdict on news with no repair in it, as a tuple value.
pub fn _the_heavy_pool(memory: &Memory) -> Value {
    Value::from(the_heavy_pool(memory))
}

/// `_the_heavy_pool` as lines.
pub fn the_heavy_pool(memory: &Memory) -> Vec<String> {
    if a_death_just_said(memory) {
        return VERDICT_NUMB.clone();
    }
    if a_wait_with_no_loss(memory) {
        return VERDICT_QUEUE.clone();
    }
    if a_setback_with_no_loss(memory) {
        return VERDICT_SETBACK.clone();
    }
    VERDICT_HEAVY.clone()
}

/// Python: bot/compose/fallback.py:5917 `a_heavy_verdict(memory=None)` (never called on the trace corpus)
pub fn a_heavy_verdict(memory: &Memory) -> bool {
    last_lines(memory, 2).iter().any(|l| !l.is_empty() && GRIEF_ABOVE.is_match(l))
}

/// Python: bot/compose/fallback.py:5940 `a_happening_now(memory=None)` (never called on the trace corpus)
pub fn a_happening_now(memory: &Memory) -> bool {
    last_lines(memory, 2).iter().any(|l| !l.is_empty() && HAPPENING_NOW.is_match(l))
}

/// Python: bot/compose/fallback.py:5959 `_a_verdict_on_something(text, memory)` (never called on the trace corpus)
pub fn _a_verdict_on_something(text_: &str, memory: &Memory) -> bool {
    if VERDICT_POINTS_BACK.match_(text_).is_some() {
        return true;
    }
    !memory.turns.is_empty()
}

pools!(VERDICT_SHAME, VERDICT_LUCK, VERDICT_ROUGH, VERDICT_ANNOY);

/// Python: bot/compose/fallback.py:5976 `_verdict_pool(clean, memory=None)`
/// The pool for a verdict on the news, or None when the line is anything more. (The skeleton's
/// `()` return could not carry the pool.)
pub fn _verdict_pool(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let t = text::squash(&lower(clean));
    if t.is_empty() || VERDICT_LINE.match_(&t).is_none() || t.contains('?') {
        return None;
    }
    if !_a_verdict_on_something(&t, memory) {
        return None;
    }
    if memory.turns.is_empty() {
        return None;
    }
    if rx(r"\b(?:shame|letdown)\b").is_match(&t) {
        return Some(VERDICT_SHAME.clone());
    }
    if rx(r"\b(?:typical|figures|luck|law|of course)\b").is_match(&t) {
        return Some(VERDICT_LUCK.clone());
    }
    if a_verdict_on_the_person(memory) {
        return Some(VERDICT_ROUGH.clone());
    }
    Some(VERDICT_ANNOY.clone())
}

/// Python: bot/compose/fallback.py:6001 `a_heavy_news(memory=None)` (never called on the trace corpus)
pub fn a_heavy_news(memory: &Memory) -> bool {
    for line in last_lines(memory, 2) {
        if line.is_empty() {
            continue;
        }
        if HEAVY_NEWS.is_match(&line) {
            return true;
        }
        if py_try(|| crate::compose::badnews::reads_as_the_leaving(&line)) == Some(true) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:6021 `a_loss_in_the_thread(memory=None)` (never called on the trace corpus)
pub fn a_loss_in_the_thread(memory: &Memory) -> bool {
    if a_heavy_verdict(memory) || a_heavy_news(memory) {
        return true;
    }
    match py_try(|| _in_a_loss_thread(memory) || _grief_thread(memory)) {
        Some(true) => return true,
        Some(false) => {}
        None => return false,
    }
    for line in last_lines(memory, 2) {
        if line.is_empty() {
            continue;
        }
        if HEAVY_ABOVE.is_match(&line) {
            return true;
        }
        if py_try(|| crate::compose::badnews::reads_as_sudden_loss(&line)) == Some(true) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:6052 `a_verdict_stands_down(memory=None)`
/// (Python's `if not memory` is never true of a turn's memory, which always holds its turns.)
pub fn a_verdict_stands_down(memory: &Memory) -> bool {
    match py_try(|| in_a_safety_thread(memory) || _in_a_loss_thread(memory) || _grief_thread(memory)) {
        Some(true) => return true,
        Some(false) => {}
        None => return false,
    }
    for line in last_lines(memory, 2) {
        if line.is_empty() {
            continue;
        }
        if HARD_WEEK.is_match(&line) || HEAVY_ABOVE.is_match(&line) {
            return true;
        }
        if py_try(|| crate::compose::badnews::reads_as_sudden_loss(&line)) == Some(true) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:6093 `reads_as_a_verdict(text, memory=None)`
pub fn reads_as_a_verdict(text_: &str, memory: &Memory) -> bool {
    py_try(|| _verdict_pool(text_, memory).is_some()).unwrap_or(false)
}

regexes!(FIRST_TURN_VERDICT, REACTION_LINE);
pools!(VERDICT_FIRST, REACTION_REPEAT, REACTION_REPEAT_FIRST, REACTIONS_CARE);
static REACTION_SIGNS: LazyLock<Vec<(Vec<String>, &'static Re)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.REACTION_SIGNS").items().iter().map(|p| (p.items()[0].strs(), p.items()[1].as_re().unwrap())).collect()
});
static REACTIONS_SAID: LazyLock<Vec<(Vec<String>, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.REACTIONS_SAID").pairs().iter().map(|(k, v)| (k.strs(), v.strs())).collect()
});

/// Python: bot/compose/fallback.py:6122 `_reaction_pool(clean, memory=None)`
/// The pool for a bare reaction, or None when the line is anything more than one. (The
/// skeleton's `()` return could not carry the pool.)
pub fn _reaction_pool(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let t = text::squash(&lower(clean));
    if let Some(verdict) = _a_verdict_line(&t, memory) {
        if !verdict.is_empty() {
            return Some(verdict);
        }
    }
    if FIRST_TURN_VERDICT.match_(&t).is_some() && memory.turns.is_empty() {
        return Some(VERDICT_FIRST.clone());
    }
    if t.is_empty() || REACTION_LINE.match_(&t).is_none() {
        return None;
    }
    for (pool, sign) in REACTION_SIGNS.iter() {
        if sign.is_match(&t) {
            return Some(pool.clone());
        }
    }
    if rx(r"[\w']*\s*(?:what|pardon)\W*").fullmatch(&t).is_some() {
        if crate::memory::last_bot(memory).is_empty() {
            return Some(REACTION_REPEAT_FIRST.clone());
        }
        return Some(REACTION_REPEAT.clone());
    }
    None
}

/// Python: bot/compose/fallback.py:6148 `reaction_line(text, memory=None)` (never called on the trace corpus)
/// The composer's own line for a verdict on the thing before it, or None.
pub fn reaction_line(text_: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&lower(text_));
    let mut pool = _a_verdict_line(&clean, memory)?;
    if pool.is_empty() {
        return None;
    }
    if _a_care_thread_to_leave_alone(memory) {
        pool = REACTIONS_CARE.clone();
    } else if _the_person_already_said_it(memory) {
        if let Some((_, said)) = REACTIONS_SAID.iter().find(|(k, _)| *k == pool) {
            pool = said.clone();
        }
    }
    choose(&pool, memory, &clean)
}

// ---- fallback.py:6250-6505: a thing done ----

/// Python truthiness of what another module's function returns, whatever type its port gives
/// it (a reader whose skeleton says `()` is still a stub, and never returns).
pub trait Truthy {
    fn truthy(&self) -> bool;
}
impl Truthy for () {
    fn truthy(&self) -> bool {
        false
    }
}
impl Truthy for bool {
    fn truthy(&self) -> bool {
        *self
    }
}
impl Truthy for String {
    fn truthy(&self) -> bool {
        !self.is_empty()
    }
}
impl Truthy for &str {
    fn truthy(&self) -> bool {
        !self.is_empty()
    }
}
impl<T> Truthy for Vec<T> {
    fn truthy(&self) -> bool {
        !self.is_empty()
    }
}
impl<T: Truthy> Truthy for Option<T> {
    fn truthy(&self) -> bool {
        self.as_ref().is_some_and(Truthy::truthy)
    }
}
impl Truthy for Value {
    fn truthy(&self) -> bool {
        Value::truthy(self)
    }
}

pools!(A_THING_ON, A_THING_AHEAD, A_THING_GONE, A_THING_SEEN_TO, A_THING_BOUGHT, A_THING_DONE);
word_sets!(GONE_VERB, BOUGHT_VERB, ALIVE, NOT_CHOSEN, SEEN_TO_VERB);

/// Python: bot/compose/fallback.py:6247 `_a_thing_done(clean)` (never called on the trace corpus)
pub fn _a_thing_done(clean: &str) -> bool {
    _thing_done_shape(clean).is_some()
}

/// Python: bot/compose/fallback.py:6252 `_thing_done_shape(clean)` (never called on the trace corpus)
/// Just the shape: see `_thing_done_scan`.
pub fn _thing_done_shape(clean: &str) -> Option<String> {
    thing_done_scan(clean).0
}

/// Python: bot/compose/fallback.py:6257 `_thing_done_pool(clean)`
/// The pool for a thing done, or None when the line is not one.
pub fn _thing_done_pool(clean: &str) -> Option<Vec<String>> {
    if crate::compose::fallback_chain::_a_thing_they_regret(clean) {
        return None;
    }
    let (shape, done_verb) = thing_done_scan(clean);
    let shape = shape?;
    if shape.is_empty() {
        return None;
    }
    if shape == "going" {
        return Some(A_THING_ON.clone());
    }
    if shape == "soon" || shape == "at" {
        return Some(A_THING_AHEAD.clone());
    }
    if GONE_VERB.contains(&done_verb) {
        return Some(A_THING_GONE.clone());
    }
    if BOUGHT_VERB.contains(&done_verb) {
        let split = text::split_ref(clean);
        if split.iter().any(|w| ALIVE.contains(*w)) {
            return None;
        }
        if split.iter().any(|w| NOT_CHOSEN.contains(*w)) {
            return Some(A_THING_SEEN_TO.clone());
        }
        return Some(A_THING_BOUGHT.clone());
    }
    if SEEN_TO_VERB.contains(&done_verb) {
        return Some(A_THING_SEEN_TO.clone());
    }
    Some(A_THING_DONE.clone())
}

regexes!(
    HARM_AFTER,
    TAIL_JOIN,
    TAIL_COMPLAINT,
    TAIL_ASK,
    TAIL_GRUMBLE,
    TAIL_TROUBLE,
    NEGATIVE_LINE,
    HARM_NAMED,
    HARM_ON_ME,
    FEELING_OFF,
    A_TROUBLE_IN_THE_TAIL,
    HARM_ANYWHERE,
    WENT_AT_IT,
    DID_AT_IT,
    SPENT_TIME,
    BEEN_AT_IT,
    TIME_SPAN,
    AT_IT,
    HABIT_TIME,
    FUTURE_TIME,
);
word_sets!(
    QUESTION_OPENERS,
    DONE_VERB,
    DONE_NOT_THIS,
    DONE_ONLY_A_THING,
    DONE_NOT_A_PERSON,
    DONE_ING,
    AT_IT_EXTRA,
    BECOMING,
    BECOMING_STATE,
    AT_IT_NOT,
    THING_AFTER,
);

/// Python: bot/compose/fallback.py:6305 `_tail_complains(clean)`
pub fn _tail_complains(clean: &str) -> bool {
    match TAIL_JOIN.search(clean) {
        Some(first) => TAIL_COMPLAINT.is_match(&clean[first.end()..]),
        None => false,
    }
}

/// Python: bot/compose/fallback.py:6319 `_thing_done_scan(clean)`
/// `(shape, verb)` for this line, or `(None, "")` when it is not a thing done, as a tuple value.
pub fn _thing_done_scan(clean: &str) -> Value {
    let (shape, verb) = thing_done_scan(clean);
    Value::Tuple(std::sync::Arc::new(vec![shape.map(Value::from).unwrap_or(Value::None), Value::from(verb)]))
}

/// `_thing_done_scan` as a pair.
pub fn thing_done_scan(clean: &str) -> (Option<String>, String) {
    let none = (None, String::new());
    if clean.contains('?')
        || clean.contains(" but ")
        || TAIL_ASK.is_match(clean)
        || TAIL_GRUMBLE.is_match(clean)
        || TAIL_TROUBLE.is_match(clean)
        || _tail_complains(clean)
    {
        return none;
    }
    if QUESTION_OPENERS.contains(text::split_ref(clean).first().copied().unwrap_or("")) {
        return none;
    }
    if NEGATIVE_LINE.is_match(clean) {
        return none;
    }
    let tokens = az_words(clean);
    if crate::compose::trouble::reads_as_trouble(clean, Some(&tokens[..]), clean).truthy() {
        return none;
    }
    if HARM_AFTER.is_match(clean) {
        return none;
    }
    if HARM_NAMED.is_match(clean) || HARM_ON_ME.is_match(clean) {
        return none;
    }
    if FEELING_OFF.is_match(clean) || A_TROUBLE_IN_THE_TAIL.is_match(clean) {
        return none;
    }
    let t = rx(r"\b(?:ive|i've)\b").sub("i have", clean);
    let t = rx(r"\b(?:weve|we've)\b").sub("we have", &t);
    let t = rx(r"\b(i|we)'d\b").sub(r"\1 had", &t);
    let words = az_words(&t);
    for i in 0..words.len().saturating_sub(1) {
        if words[i] != "i" && words[i] != "we" {
            continue;
        }
        let mut j = i + 1;
        while j < words.len() && ["just", "also", "finally", "only", "actually", "have", "had", "been"].contains(&words[j].as_str()) {
            j += 1;
        }
        if j >= words.len() || !DONE_VERB.contains(&words[j]) {
            continue;
        }
        let rest = &words[j + 1..(j + 3).min(words.len())];
        if rest.iter().any(|w| DONE_NOT_THIS.contains(w)) {
            return none;
        }
        if DONE_ONLY_A_THING.contains(&words[j]) && words[(j + 1).min(words.len())..(j + 4).min(words.len())].iter().any(|w| DONE_NOT_A_PERSON.contains(w)) {
            return none;
        }
        return (Some("done".into()), words[j].clone());
    }
    if !words.is_empty() && DONE_VERB.contains(&words[0]) && !HARM_ANYWHERE.is_match(clean) {
        let rest = &words[1..3.min(words.len())];
        if !rest.iter().any(|w| DONE_NOT_THIS.contains(w)) {
            return (Some("done".into()), words[0].clone());
        }
    }
    if WENT_AT_IT.is_match(&t) {
        return (Some("gone".into()), "went".into());
    }
    if DID_AT_IT.is_match(&t) {
        return (Some("gone".into()), "did".into());
    }
    if let Some(spent) = SPENT_TIME.search(&t) {
        if DONE_ING.contains(spent.str(1)) {
            return (Some("going".into()), spent.str(1).to_string());
        }
    }
    if let Some(been) = BEEN_AT_IT.search(&t) {
        if DONE_ING.contains(been.str(1)) && TIME_SPAN.is_match(&t) {
            return (Some("going".into()), been.str(1).to_string());
        }
    }
    if let Some(at_it) = AT_IT.search(&t) {
        let verb = at_it.str(1);
        if DONE_ING.contains(verb) || AT_IT_EXTRA.contains(verb) {
            let rest = &t[at_it.end()..];
            if BECOMING.contains(verb) && BECOMING_STATE.contains(text::split_ref(rest).first().copied().unwrap_or("")) {
                return none;
            }
            if _a_thing_after(rest) && !HABIT_TIME.is_match(rest) && !az_words(rest).iter().any(|w| AT_IT_NOT.contains(w)) {
                let shape = if FUTURE_TIME.is_match(rest) { "soon" } else { "at" };
                return (Some(shape.into()), verb.to_string());
            }
        }
    }
    none
}

/// Python: bot/compose/fallback.py:6477 `_a_thing_after(rest)`
/// True when the words after the verb name the thing the person is working on.
pub fn _a_thing_after(rest: &str) -> bool {
    az_words(rest).iter().any(|w| !THING_AFTER.contains(w) && w.chars().count() > 2)
}

/// Python: bot/compose/fallback.py:6494 `_done_ing()` (never called on the trace corpus)
/// The "-ing" forms of DONE_VERB (the module keeps the result as `DONE_ING`).
pub fn _done_ing() -> HashSet<String> {
    let mut out = HashSet::new();
    for verb in DONE_VERB.iter() {
        let stem: &str = if verb.ends_with("ed") { &verb[..verb.len() - 2] } else { verb };
        out.insert(format!("{stem}ing"));
        out.insert(format!("{verb}ing"));
        if stem.chars().count() > 2 {
            let last = stem.chars().last().unwrap();
            out.insert(format!("{stem}{last}ing"));
        }
    }
    for w in [
        "running", "swimming", "making", "writing", "cutting", "digging", "hanging", "building", "sweeping", "freezing", "sitting", "getting",
        "decorating", "repainting", "reorganising", "reorganizing",
    ] {
        out.insert(w.to_string());
    }
    out
}

// ---- fallback.py:6568-7130: knocks and mishaps, dislikes, own feeling, summons, promises, doable ----

/// What another module's reader returns, as Python's `str | None`, whatever type its port gives
/// it (a reader whose skeleton says `()` is still a stub, and never returns).
pub trait PyOptStr {
    fn py_opt_str(self) -> Option<String>;
}
impl PyOptStr for () {
    fn py_opt_str(self) -> Option<String> {
        None
    }
}
impl PyOptStr for String {
    fn py_opt_str(self) -> Option<String> {
        Some(self)
    }
}
impl PyOptStr for Option<String> {
    fn py_opt_str(self) -> Option<String> {
        self
    }
}
impl PyOptStr for Value {
    fn py_opt_str(self) -> Option<String> {
        self.as_str().map(str::to_string)
    }
}

regexes!(KNOCK_ON_ME, A_SPAN_OF_TIME, WALKED_INTO, MISHAP_FELL, MISHAP_UNDERFOOT, SENSE_GONE, MISHAP_ANYWHERE, MISHAP_OVER);

/// Python: bot/compose/fallback.py:6568 `_a_knock_on_oneself(clean)`
pub fn _a_knock_on_oneself(clean: &str) -> bool {
    if clean.is_empty() || clean.contains('?') || clean.contains(" but ") {
        return false;
    }
    if !KNOCK_ON_ME.is_match(clean) {
        return false;
    }
    !A_SPAN_OF_TIME.is_match(clean)
}

/// Python: bot/compose/fallback.py:6591 `_a_knock_into_a_thing(clean)`
pub fn _a_knock_into_a_thing(clean: &str) -> bool {
    if clean.is_empty() || clean.contains('?') || clean.contains(" but ") {
        return false;
    }
    WALKED_INTO.is_match(clean)
}

/// Python: bot/compose/fallback.py:6691 `_a_mishap_to_ask_about(text)`
pub fn _a_mishap_to_ask_about(text_: &str) -> bool {
    if SENSE_GONE.is_match(text_) {
        return true;
    }
    if !MISHAP_ANYWHERE.is_match(text_) {
        return false;
    }
    !MISHAP_OVER.is_match(text_)
}

/// Python: bot/compose/fallback.py:6707 `_a_mishap(clean, words)`
pub fn _a_mishap(clean: &str, words: &[String]) -> bool {
    if clean.is_empty() || clean.contains('?') || words.len() > 24 {
        return false;
    }
    if MISHAP_OVER.is_match(clean) {
        return false;
    }
    MISHAP_FELL.is_match(clean) || MISHAP_UNDERFOOT.is_match(clean)
}

regexes!(NOT_ENOUGH, NO_DIRECTION, NOT_SELF, DISLIKE_HAS_TWO_HALVES, DISLIKE_TAIL, DISLIKE_WITHOUT_AN_I);
pools!(DISLIKE_ASKS, NOT_ENOUGH_REPLY, NO_DIRECTION_REPLY);
word_sets!(NOT_A_THING_ASKED);
static DISLIKE_TOO_LONG: LazyLock<usize> = LazyLock::new(|| data::get("bot.compose.fallback.DISLIKE_TOO_LONG").as_int().unwrap() as usize);

/// `[frame.format(thing=thing, Thing=thing[:1].upper() + thing[1:]) for frame in DISLIKE_ASKS]`.
fn dislike_asks(thing: &str) -> Vec<String> {
    let cap = format!("{}{}", text::upper(&text::slice(thing, 0, Some(1))), text::slice(thing, 1, None));
    DISLIKE_ASKS.iter().map(|f| format_named(f, &[("thing", thing), ("Thing", &cap)])).collect()
}

/// Python: bot/compose/fallback.py:6831 `the_dislike_ask(clean, memory)`
/// A question about the thing the person has just said they do not like, or None. (The
/// skeleton's `()` return could not carry the question.)
pub fn the_dislike_ask(clean: &str, memory: &Memory) -> Option<String> {
    let mut thing = crate::memory::obstacle::the_thing(clean).py_opt_str().unwrap_or_default();
    let strip_ends = |s: &str| s.trim_matches([' ', ',', '.', ';', ':', '-']).to_string();
    if thing.is_empty() {
        if let Some(found) = DISLIKE_WITHOUT_AN_I.match_(&text::squash(clean)) {
            let mut t = strip_ends(found.name_str("thing"));
            if t.is_empty() || NOT_A_THING_ASKED.contains(&lower(&t)) {
                return None;
            }
            t = strip_ends(&DISLIKE_TAIL.sub("", &t));
            if t.is_empty() || text::split_ref(&t).len() > *DISLIKE_TOO_LONG {
                return None;
            }
            if DISLIKE_HAS_TWO_HALVES.is_match(&t) {
                return None;
            }
            let t = _yours(&t);
            return choose(&dislike_asks(&t), memory, clean);
        }
    }
    if !thing.is_empty() {
        thing = _yours(&strip_ends(&DISLIKE_TAIL.sub("", &thing)));
    }
    if thing.is_empty() || text::split_ref(&thing).len() > *DISLIKE_TOO_LONG {
        return None;
    }
    if DISLIKE_HAS_TWO_HALVES.is_match(&thing) {
        return None;
    }
    choose(&dislike_asks(&thing), memory, clean)
}

/// Python: bot/compose/fallback.py:6862 `_own_feeling(clean)`
/// The pool for a statement about the speaker's own worth or direction, or None. (The
/// skeleton's `()` return could not carry the pool.)
pub fn _own_feeling(clean: &str) -> Option<Vec<String>> {
    if text::rstrip(clean).ends_with('?') || NOT_SELF.is_match(clean) {
        return None;
    }
    if NOT_ENOUGH.is_match(clean) {
        return Some(NOT_ENOUGH_REPLY.clone());
    }
    if NO_DIRECTION.is_match(clean) {
        return Some(NO_DIRECTION_REPLY.clone());
    }
    None
}

regexes!(SUMMONS_ASKS_A_CALL);
pools!(SUMMONS_RANG, SUMMONS_WRITTEN, SUMMONS_SEEN);

/// Python: bot/compose/fallback.py:6882 `_summons_pool(summons, line='')` (never called on the trace corpus)
/// The lines for a summons, by shape: a meeting, a letter, or a phone call (as a tuple value).
/// `summons` is the reading's `summons` dict.
pub fn _summons_pool(summons: &Value, line: &str) -> Value {
    let shape = if summons.is_dict() { summons.get("shape").and_then(Value::as_str) } else { None };
    Value::from(summons_pool(shape, line))
}

/// `_summons_pool` by the summons' shape.
pub fn summons_pool(shape: Option<&str>, line: &str) -> Vec<String> {
    match shape {
        Some("written") => {
            if SUMMONS_ASKS_A_CALL.is_match(line) {
                SUMMONS_RANG.clone()
            } else {
                SUMMONS_WRITTEN.clone()
            }
        }
        Some("rang") => SUMMONS_RANG.clone(),
        _ => SUMMONS_SEEN.clone(),
    }
}

regexes!(
    PROMISE_OFFICE,
    PROMISE_A_PERSON_NAMED,
    PROMISE_REPORTED,
    PROMISE_FUTURE,
    PROMISE_BUT_NEVER,
    PROMISE_NEVER_WHAT,
    PROMISE_RISK,
    PROMISE_RECUR,
    PROMISE_NOT_A_PERSON,
    PROMISE_MONEY,
    PROMISE_A_HE_OR_SHE,
);
pools!(PROMISE_A_PERSON, PROMISE_AN_OFFICE);
word_sets!(GRIEVANCE_HEALTH, GRIEVANCE_HARM);

/// Python: bot/compose/fallback.py:7011 `_a_promise_that_keeps_moving(clean)`
/// The reply pool for a promise that keeps not happening, or None. (The skeleton's `()` return
/// could not carry the pool.)
pub fn _a_promise_that_keeps_moving(clean: &str) -> Option<Vec<String>> {
    let reported = PROMISE_REPORTED.search(clean);
    if let Some(device) = PROMISE_NOT_A_PERSON.search(clean) {
        if reported.as_ref().is_none_or(|r| device.start() < r.start()) {
            return None;
        }
    }
    if PROMISE_MONEY.is_match(clean) || PROMISE_RISK.is_match(clean) {
        return None;
    }
    let said = az_set(&clean.replace('\u{2019}', "'"));
    let office_subject = PROMISE_OFFICE.is_match(clean) && !PROMISE_A_HE_OR_SHE.is_match(clean) && !PROMISE_A_PERSON_NAMED.is_match(clean);
    if !office_subject && said.iter().any(|w| GRIEVANCE_HEALTH.contains(w) || GRIEVANCE_HARM.contains(w)) {
        return None;
    }
    let broken = PROMISE_BUT_NEVER.search(clean);
    let promised = PROMISE_NEVER_WHAT.is_match(clean)
        || reported.as_ref().is_some_and(|r| {
            (PROMISE_RECUR.is_match(clean) && PROMISE_FUTURE.is_match(clean)) || broken.as_ref().is_some_and(|b| b.start() > r.end())
        });
    if !promised {
        return None;
    }
    Some(if office_subject { PROMISE_AN_OFFICE.clone() } else { PROMISE_A_PERSON.clone() })
}

regexes!(DOABLE, DOABLE_NOT, DOABLE_FOLD);
word_sets!(DOABLE_WORDS, DOABLE_BREATHERS);
pools!(DOABLE_REPLIES);

/// Python: bot/compose/fallback.py:7087 `_reads_as_a_judgement(text)`
/// True when the line is nothing but a judgement that what was said is doable.
pub fn _reads_as_a_judgement(text_: &str) -> bool {
    let mut clean = text::squash(&lower(text_));
    while DOABLE_FOLD.match_(&clean).is_some() {
        clean = text::strip(&DOABLE_FOLD.subn("", &clean, 1)).to_string();
    }
    if clean.is_empty() || clean.contains('?') {
        return false;
    }
    let words = text::split_ref(&clean);
    if words.len() > 8 {
        return false;
    }
    if !DOABLE.is_match(&clean) || DOABLE_NOT.is_match(&clean) {
        return false;
    }
    words.iter().map(|w| w.trim_matches(['.', ',', '!', '?', ';', ':', '\'', '\u{2019}'])).all(|w| DOABLE_WORDS.contains(w) || DOABLE_BREATHERS.contains(w))
}

/// Python: bot/compose/fallback.py:7111 `_a_doable(clean, memory)`
/// The reply for a short "that's doable" about the advice just given, or None. (The skeleton's
/// `()` return could not carry the reply.)
pub fn _a_doable(clean: &str, memory: &Memory) -> Option<String> {
    if !_reads_as_a_judgement(clean) {
        return None;
    }
    if the_advice(memory).is_empty() {
        return None;
    }
    choose(&DOABLE_REPLIES, memory, clean)
}

/// Python: bot/compose/fallback.py:7121 `a_doable(text, memory)`
/// Read by greeting: the reply when this line is a judgement of the answer just given, or None.
/// (The skeleton's `()` return could not carry the reply.)
pub fn a_doable(text_: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&lower(text_));
    _a_doable(&clean, memory)
}
