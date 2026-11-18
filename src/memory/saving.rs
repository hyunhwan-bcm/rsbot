//! Port of `bot/memory/saving.py`: a saving goal carries its own arithmetic, and the sum is a
//! follow-up waiting to happen. The goal, the date, the pot and the rate are read out of the
//! chat, and the line is answered with the figure it asks for, or handed back as the plain money
//! question the knowledge base answers. Module data comes from the exported bundle; names PyBot
//! defines twice (`A_RATE`, `A_THING_NAMED`, `NOT_A_THING`) hold the value the module ends with,
//! which is the one every function sees at run time.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

const I: u32 = crate::py::re::I;

fn g(name: &str) -> &'static Value {
    data::get(&format!("bot.memory.saving.{name}"))
}
fn re(name: &str) -> &'static Re {
    g(name).as_re().unwrap_or_else(|| panic!("bot.memory.saving.{name} is not a regex"))
}

macro_rules! patterns {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> = LazyLock::new(|| re(stringify!($name)));)*
    };
}
patterns!(
    SAVING, SUM, NOT_A_POT, BARE_FIGURE, DATE, CORRECTING, END_OF_RELATIVE, BARE_END_OF_RELATIVE,
    BARE_DATE, ALREADY, INTENT, PAST_SAVING, HELD, TOGETHER, HELD_SAID, FURTHER_SAID,
    POT_REPORTED, ALREADY_HELD, ALREADY_AFTER, AIM_AT_A_SUM, BY_A_DATE, AT_A_FIGURE, A_RATE,
    HOW_FAR_OFF, NEED_A_FIGURE, POT_OPENER, HOW_LONG_ASK, HOW_LONG_UNTIL, WHEN_AT_THE_GOAL,
    PRICE_WORDS, A_TRAILING_SUM, NOT_A_PRICE, A_GOAL_STATED, A_LENGTH_WAS_ASKED, A_LENGTH_SAID,
    ANOTHER_SUM, A_GOAL_SUM, A_RATE_ANY_OTHER_PERIOD, INCOME_FRAME, OUTGOING_FRAME, CLAUSE_EDGE,
    A_RATE_PUT_RIGHT, A_CAN_CARRY_IT, A_SUM_A_MONTH, _OTHER_FIGURE_WORDS, A_LONE_SUM,
    A_CORRECTION_AND_A_FIGURE, _NOT_THE_RATE, A_NO_THEN_A_RATE, A_SPENT_ON, A_MORE_LIKE, HOW_MUCH,
    A_WEEK_FIGURE, ASKS_IF_ENOUGH, ASKS_HOW_MUCH_MORE, A_NEW_RATE, A_RATE_WITH_A_VERB,
    ASKS_A_LENGTH, OTHER_POT, RATE_GIVEN, RATE_VERB, FILLER_LEAD, BARE_RATE, SAVE_VERB,
    CONDITIONAL, ADDED, A_RESTATED_RATE, A_MONTH_FIGURE, ASKS_A_LENGTH_FOR_A_THING,
    THE_THING_WANTED, A_THING_NAMED, PUT_BY, MONEY_OUT,
);

static MONTHS: LazyLock<Vec<String>> = LazyLock::new(|| g("MONTHS").strs());
static NAMED_WHEN: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    g("NAMED_WHEN").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].str().to_string())).collect()
});
static MONTHS_IN_A: LazyLock<HashMap<String, f64>> =
    LazyLock::new(|| g("MONTHS_IN_A").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_f64().unwrap_or(0.0))).collect());
static KNOWN_TO_THE_MONEY_ENTRY: LazyLock<Vec<String>> = LazyLock::new(|| g("_KNOWN_TO_THE_MONEY_ENTRY").strs());
static THE_END_OF_A_SEASON: LazyLock<HashMap<String, String>> =
    LazyLock::new(|| g("_THE_END_OF_A_SEASON").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect());
static SAVING_CLOSERS: LazyLock<Vec<String>> = LazyLock::new(|| g("SAVING_CLOSERS").strs());
static A_WEEK_ASK: LazyLock<String> = LazyLock::new(|| g("A_WEEK_ASK").str().to_string());
static NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| g("NOT_A_THING").str_set());

// The patterns PyBot writes inline.
static HAVE_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:have|has|got|got\s+\w+)\b", I));
static NOT_A_LETTER: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z]", 0));
static NOT_A_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^0-9]", 0));
static A_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d", 0));
static MONEY_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(r"£|\bpounds?\b|\bquid\b", I));
static FOR_A_LENGTH: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bfor\s+(?:\d|a|an|two|three|four|five|six)", I));
static A_FIGURE_ADDED: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?:£\s*)?(\d[\d,]*(?:\.\d+)?)\s*(k|grand|thousand)?", 0));
static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static SENTENCE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s+", 0));
static PRICE_ASKED: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\bwhat(?:'s| is) (?:the |it |that )?(?:.* )?going for\b|\bwhat'?s the price\b|\bwhat does it cost\b|\bhow much is it\b", 0)
});

/// The value of one Unicode decimal digit (`int()` and `float()` read every Nd digit).
fn decimal_value(c: char) -> Option<u32> {
    if c.is_ascii_digit() {
        return c.to_digit(10);
    }
    if !crate::understanding::pyuni::ch_decimal(c) {
        return None;
    }
    let mut back = 0u32;
    let mut cp = c as u32;
    while cp > 0 {
        match char::from_u32(cp - 1) {
            Some(p) if crate::understanding::pyuni::ch_decimal(p) => {
                back += 1;
                cp -= 1;
            }
            _ => break,
        }
    }
    Some(back % 10)
}

/// `float(s)` for digits with an optional point; None where Python raises ValueError.
fn py_float(s: &str) -> Option<f64> {
    let mut ascii = String::new();
    for c in s.chars() {
        if c == '.' {
            ascii.push('.');
        } else {
            ascii.push(char::from_digit(decimal_value(c)?, 10)?);
        }
    }
    if ascii.is_empty() || ascii.starts_with('.') && ascii.len() == 1 {
        return None;
    }
    ascii.parse().ok()
}

/// `int(s)` for a string of digits; None where Python raises ValueError.
fn py_int(s: &str) -> Option<i64> {
    if s.is_empty() {
        return None;
    }
    let mut n: i64 = 0;
    for c in s.chars() {
        n = n.checked_mul(10)?.checked_add(decimal_value(c)? as i64)?;
    }
    Some(n)
}

/// `round(x)` (half to even) as an int.
fn py_round(x: f64) -> i64 {
    pt::round_half_even(x)
}

/// `format(n, ",")`.
fn commas(n: i64) -> String {
    pt::thousands(n)
}

/// `-(-a // b)`: division rounded up.
fn ceil_div(a: i64, b: i64) -> i64 {
    -((-a).div_euclid(b))
}

/// The person's lines, oldest first.
fn users(memory: &Memory) -> Vec<String> {
    memory.turns.iter().map(|t| t.user.clone()).collect()
}

/// The last `n` of a list.
fn last_n<T>(items: &[T], n: usize) -> &[T] {
    &items[items.len().saturating_sub(n)..]
}

/// The bot's last line, of the turns that have one.
fn last_bot_with_a_reply(memory: &Memory) -> Option<String> {
    memory.turns.iter().rev().find(|t| !t.bot.is_empty()).map(|t| t.bot.clone())
}

/// Python: bot/memory/saving.py:106 `_a_sum(text)`
///
/// The sum of money in the line, as a plain number, or None.
pub fn _a_sum(text: &str) -> Option<i64> {
    let line = text;
    for found in SUM.finditer(line) {
        let figure = found.get(1).filter(|s| !s.is_empty()).or_else(|| found.get(3)).unwrap_or("");
        let unit = pt::lower(found.get(2).filter(|s| !s.is_empty()).or_else(|| found.get(4)).unwrap_or(""));
        if figure.is_empty() {
            continue;
        }
        if NOT_A_POT.match_(&line[found.end()..]).is_some() {
            continue; // a saving on a bill, not a pot
        }
        let Some(mut value) = py_float(&figure.replace(',', "")) else {
            continue;
        };
        if matches!(unit.as_str(), "k" | "grand" | "thousand") {
            value *= 1000.0;
        }
        if value < 20.0 {
            continue; // a habit, not a goal
        }
        return Some(py_round(value));
    }
    // A goal written without its unit: "i need to save 3000".
    for found in BARE_FIGURE.finditer(line) {
        let Some(value) = py_int(found.str(1)) else {
            continue;
        };
        if value < 100 || (1990..=2100).contains(&value) {
            continue;
        }
        if NOT_A_POT.match_(&line[found.end()..]).is_some() {
            continue;
        }
        return Some(value);
    }
    None
}

/// `_a_sum` as Python's truthiness reads it (`0` and None alike).
fn a_sum(text: &str) -> i64 {
    _a_sum(text).unwrap_or(0)
}

/// Python: bot/memory/saving.py:141 `_a_date(text)`
///
/// The date the line puts the goal in, in the person's own words ("june", "christmas"), or "".
pub fn _a_date(text: &str) -> String {
    if let Some(relative) = END_OF_RELATIVE.search(text) {
        return format!("the end of {}", pt::squash(&pt::lower(relative.str(1))));
    }
    let Some(found) = DATE.search(text) else {
        return String::new();
    };
    let word = pt::lower(found.str(1));
    if pt::lower(found.as_str()).contains("end of") {
        return format!("the end of {word}");
    }
    word
}

/// Python: bot/memory/saving.py:162 `_month_of(when)`
///
/// The month a date falls in, so a named one can be counted: "christmas" is December.
pub fn _month_of(when: &str) -> String {
    let mut word = pt::strip(&pt::lower(when)).to_string();
    if let Some(rest) = word.strip_prefix("the end of ") {
        word = pt::strip(rest).to_string();
    }
    if word == "next month" || word == "this month" {
        let step: i64 = if word == "this month" { 1 } else { 2 };
        use chrono::Datelike;
        let month = crate::py::clock::today().month() as i64;
        return MONTHS[(month + step - 2).rem_euclid(12) as usize].clone();
    }
    if MONTHS.contains(&word) {
        return word;
    }
    for (name, month) in NAMED_WHEN.iter() {
        if word.contains(name.as_str()) {
            return month.clone();
        }
    }
    String::new()
}

/// A figure and its unit, as `float(figure) * 1000` for a thousand-word.
fn figure_and_unit(figure: &str, unit: &str) -> Option<f64> {
    let mut value = py_float(&figure.replace(',', ""))?;
    if matches!(pt::lower(unit).as_str(), "k" | "grand" | "thousand") {
        value *= 1000.0;
    }
    Some(value)
}

/// Python: bot/memory/saving.py:191 `_two_sums_added(line)`
///
/// The two sums the line adds together, or 0 when it names only one.
pub fn _two_sums_added(line: &str) -> i64 {
    let Some(found) = TOGETHER.search(line) else {
        return 0;
    };
    let mut total = 0;
    for (figure, unit) in [(found.str(1), found.str(2)), (found.str(3), found.str(4))] {
        let Some(value) = figure_and_unit(figure, unit) else {
            return 0;
        };
        total += py_round(value);
    }
    if total >= 20 { total } else { 0 }
}

/// Python: bot/memory/saving.py:220 `_pounds(figure, unit)`
///
/// A figure written as people write it ("2", "2k", "2 grand") as a number, or 0.
pub fn _pounds(figure: &str, unit: &str) -> i64 {
    match figure_and_unit(figure, unit) {
        Some(value) if value >= 20.0 => py_round(value),
        _ => 0,
    }
}

/// `the_pot_and_the_rest` as numbers.
fn pot_and_rest(text: &str) -> (i64, i64) {
    if pt::strip(text).is_empty() {
        return (0, 0);
    }
    let (Some(further), Some(held)) = (FURTHER_SAID.search(text), HELD_SAID.search(text)) else {
        return (0, 0);
    };
    if further.start() < held.start() {
        return (0, 0); // the other way round, which `_two_sums_added` reads
    }
    let pot = _pounds(held.str(1), held.str(2));
    let rest = _pounds(further.str(1), further.str(2));
    if pot == 0 || rest == 0 {
        return (0, 0);
    }
    (pot, rest)
}

/// Python: bot/memory/saving.py:231 `the_pot_and_the_rest(text)`
///
/// (money already held, money still to find) when the line names both, else (0, 0).
pub fn the_pot_and_the_rest(text: &str) -> Value {
    let (pot, rest) = pot_and_rest(text);
    Value::Tuple(Arc::new(vec![Value::Int(pot), Value::Int(rest)]))
}

/// Python: bot/memory/saving.py:288 `the_pot_said_alone(text)`
///
/// The money the line says is already saved, when the line is only that, else 0.
pub fn the_pot_said_alone(text: &str) -> i64 {
    let line = text;
    if pt::strip(line).is_empty() || FURTHER_SAID.is_match(line) {
        return 0;
    }
    if AIM_AT_A_SUM.is_match(line) || BY_A_DATE.is_match(line) {
        return 0;
    }
    if let Some(held) = HELD_SAID.search(line) {
        if !POT_REPORTED.is_match(line) {
            if INTENT.is_match(line) && !HAVE_WORD.is_match(line) {
                return 0;
            }
            return _pounds(held.str(1), held.str(2));
        }
    }
    for pattern in [&*ALREADY_HELD, &*ALREADY_AFTER] {
        let Some(found) = pattern.search(line) else {
            continue;
        };
        let pot = _pounds(found.str(1), found.str(2));
        // A small figure beside "already have" is not a pot.
        if pot >= 50 {
            return pot;
        }
    }
    0
}

/// Python: bot/memory/saving.py:324 `the_pot_they_are_on(text)`
///
/// The money the line says the person is on their way to, when that is what the line says.
pub fn the_pot_they_are_on(text: &str) -> i64 {
    let line = text;
    if pt::strip(line).is_empty() || NOT_A_PRICE.is_match(line) || _a_rate(line) {
        return 0;
    }
    let Some(found) = AT_A_FIGURE.search(line) else {
        return 0;
    };
    if NOT_A_POT.match_(&line[found.end()..]).is_some() {
        return 0;
    }
    _pounds(found.str(1), found.str(2))
}

/// Python: bot/memory/saving.py:348 `_a_rate(line)`
///
/// True when the figure in the line is a rate rather than a sum.
pub fn _a_rate(line: &str) -> bool {
    A_RATE.is_match(line)
}

/// Python: bot/memory/saving.py:353 `_the_pot_they_are_on(memory)`
///
/// The last position the person said they were on towards the goal, or 0.
pub fn _the_pot_they_are_on(memory: &Memory) -> i64 {
    let lines = users(memory);
    last_n(&lines, 6).iter().rev().map(|line| the_pot_they_are_on(line)).find(|pot| *pot != 0).unwrap_or(0)
}

/// Python: bot/memory/saving.py:386 `_the_goal_they_need(memory)`
///
/// The sum the chat says they need, when the chat has already said what they save for.
pub fn _the_goal_they_need(memory: &Memory) -> i64 {
    let lines = users(memory);
    let lines = last_n(&lines, 6);
    if !lines.iter().any(|line| SAVING.is_match(line) && (a_sum(line) == 0 || _a_rate(line))) {
        return 0;
    }
    for line in lines.iter().rev() {
        if the_pot_they_are_on(line) != 0 || the_pot_said_alone(line) != 0 || _a_rate(line) {
            continue;
        }
        if NEED_A_FIGURE.is_match(line) {
            let wanted = a_sum(line);
            if wanted != 0 {
                return wanted;
            }
        }
    }
    0
}

/// Python: bot/memory/saving.py:406 `asks_how_far_off(text)`
pub fn asks_how_far_off(text: &str) -> bool {
    HOW_FAR_OFF.is_match(text)
}

/// Python: bot/memory/saving.py:411 `the_gap_left(text, memory)`
///
/// The money between the pot and the goal, said back, or "".
pub fn the_gap_left(text: &str, memory: &Memory) -> String {
    let line = text;
    if !asks_how_far_off(line) || a_sum(line) != 0 || _a_rate(line) {
        return String::new();
    }
    let (goal, pot) = goal_and_pot(memory);
    if goal == 0 || pot == 0 || pot >= goal {
        return String::new();
    }
    let said = _the_gap_said(goal, pot);
    if _the_gap_was_just_said(&said, memory) {
        return _the_gap_again(goal, pot, memory);
    }
    said
}

/// Python: bot/memory/saving.py:434 `_the_gap_said(goal, pot)`
///
/// The two figures side by side: what is left, and how much of it is already there.
pub fn _the_gap_said(goal: i64, pot: i64) -> String {
    let left = goal - pot;
    let share = py_round(100.0 * pot as f64 / goal as f64);
    format!("£{} to go of £{} - you're £{} in, {share}% of the way there.", commas(left), commas(goal), commas(pot))
}

/// Python: bot/memory/saving.py:442 `_the_gap_was_just_said(said, memory)`
///
/// True when the bot's own last line was this same sentence.
pub fn _the_gap_was_just_said(said: &str, memory: &Memory) -> bool {
    match memory.turns.last() {
        Some(last) => pt::squash(&last.bot) == pt::squash(said),
        None => false,
    }
}

/// Python: bot/memory/saving.py:455 `_the_gap_again(goal, pot, memory)`
///
/// The same gap said another way: the length at the rate they settled on, else a restatement.
pub fn _the_gap_again(goal: i64, pot: i64, memory: &Memory) -> String {
    let rate = _the_rate_in_the_chat(memory);
    if rate != 0 {
        let said = _the_length_at_that_rate(goal - pot, rate, "month", pot);
        if !said.is_empty() {
            return said;
        }
    }
    format!("Still £{} to go - you've £{} of the £{} in the pot.", commas(goal - pot), commas(pot), commas(goal))
}

/// `the_goal_and_the_pot` as numbers.
fn goal_and_pot(memory: &Memory) -> (i64, i64) {
    let mut goal = _the_goal_in_the_chat(memory);
    if goal == 0 {
        goal = _the_goal_said_as_a_price(memory);
    }
    if goal == 0 {
        goal = _the_goal_they_need(memory);
    }
    if goal == 0 {
        return (0, 0);
    }
    let mut pot = _the_money_already_held(memory);
    if pot == 0 {
        pot = _the_pot_they_are_on(memory);
    }
    (goal, pot)
}

/// Python: bot/memory/saving.py:472 `the_goal_and_the_pot(memory)`
///
/// (goal, pot) as the chat holds them, or (0, 0).
pub fn the_goal_and_the_pot(memory: &Memory) -> Value {
    let (goal, pot) = goal_and_pot(memory);
    Value::Tuple(Arc::new(vec![Value::Int(goal), Value::Int(pot)]))
}

/// Python: bot/memory/saving.py:482 `the_pot_position(text, memory)`
///
/// The gap said back when the line reports the position itself, or "".
pub fn the_pot_position(text: &str, memory: &Memory) -> String {
    if the_pot_they_are_on(text) == 0 || asks_how_far_off(text) || _a_sum_that_is_a_goal(text) {
        return String::new();
    }
    if the_pot_said_alone(text) != 0 {
        return String::new();
    }
    let (goal, pot) = goal_and_pot(memory);
    // The position is in the line being read.
    let on = the_pot_they_are_on(text);
    let pot = if on != 0 { on } else { pot };
    if goal == 0 || pot == 0 || pot >= goal {
        return String::new();
    }
    _the_gap_said(goal, pot)
}

/// Python: bot/memory/saving.py:504 `_a_sum_that_is_a_goal(text)`
///
/// True when the line carries a goal of its own, so it is not only a position.
pub fn _a_sum_that_is_a_goal(text: &str) -> bool {
    if _a_rate(text) {
        return false;
    }
    NEED_A_FIGURE.is_match(text) && a_sum(text) != 0
}

/// Python: bot/memory/saving.py:512 `rate_question_for(amount, when='')`
///
/// The money question about a sum the module works out itself, or "".
pub fn rate_question_for(amount: &Value, when: &str) -> String {
    let amount = amount.as_int().unwrap_or(0);
    if amount == 0 || amount < 20 {
        return String::new();
    }
    if !when.is_empty() {
        return format!(
            "How much do i need to put by a week to have {} pounds by {}?",
            commas(amount),
            _in_the_money_readers_words(when)
        );
    }
    format!("How much do i need to put by a week to have {} pounds?", commas(amount))
}

/// Python: bot/memory/saving.py:523 `the_money_already_held(memory)`
pub fn the_money_already_held(memory: &Memory) -> i64 {
    _the_money_already_held(memory)
}

/// Python: bot/memory/saving.py:528 `_the_money_already_held(memory)`
///
/// The money the person said is already in the pot, from the last few lines, or 0.
pub fn _the_money_already_held(memory: &Memory) -> i64 {
    let lines = users(memory);
    for line in last_n(&lines, 6).iter().rev() {
        let (pot, _rest) = pot_and_rest(line);
        if pot != 0 {
            return pot;
        }
        let pot = the_pot_said_alone(line);
        if pot != 0 {
            return pot;
        }
        let pot = the_pot_they_are_on(line);
        if pot != 0 {
            return pot;
        }
    }
    0
}

/// Python: bot/memory/saving.py:553 `without_the_pot_opener(answer)`
///
/// The answer with its "For £2,000: " opener taken off.
pub fn without_the_pot_opener(answer: &str) -> String {
    POT_OPENER.subn("", answer, 1)
}

/// Python: bot/memory/saving.py:558 `_names_a_goal(line)`
///
/// The sum when the line is aiming at money rather than reporting money already put by, or 0.
pub fn _names_a_goal(line: &str) -> i64 {
    let added = _two_sums_added(line);
    if added != 0 && SAVING.is_match(line) {
        return added;
    }
    let (pot, rest) = pot_and_rest(line);
    if pot != 0 && rest != 0 && SAVING.is_match(line) {
        return pot + rest;
    }
    if INTENT.is_match(line) && SAVING.is_match(line) {
        let aim = a_sum(line);
        if aim != 0 {
            return aim;
        }
    }
    if !(SAVING.is_match(line) && a_sum(line) != 0) {
        return 0;
    }
    if INTENT.is_match(line) {
        return a_sum(line);
    }
    if ALREADY.is_match(line) || PAST_SAVING.is_match(line) || HELD.is_match(line) {
        return 0;
    }
    a_sum(line)
}

/// Python: bot/memory/saving.py:625 `asks_how_long(text)`
///
/// True when the line asks how long the goal takes and names no figure itself.
pub fn asks_how_long(text: &str) -> bool {
    let line = pt::squash(&pt::lower(text));
    if line.is_empty() || pt::split_ref(&line).len() > 8 || A_DIGIT.is_match(&line) {
        return false;
    }
    HOW_LONG_ASK.is_match(&line) || HOW_LONG_UNTIL.match_(&line).is_some() || WHEN_AT_THE_GOAL.match_(&line).is_some()
}

/// `not memory`: the dict has no key at all.
fn is_empty(memory: &Memory) -> bool {
    memory.understood.is_none()
        && memory.intent_wants.is_none()
        && !memory.extra_had_intent
        && memory.asked_now.is_none()
        && memory.extra.is_empty()
        && memory.turns.is_empty()
}

/// Python: bot/memory/saving.py:634 `how_long_it_takes(memory)`
///
/// The goal said as a length of time, at the gentle rate and one faster, or "".
pub fn how_long_it_takes(memory: &Memory) -> String {
    if is_empty(memory) {
        return String::new();
    }
    let previous = _the_line_before(memory);
    let mut goal = _the_goal_in_the_chat(memory);
    let priced = if goal != 0 { 0 } else { _the_goal_said_as_a_price(memory) };
    if goal == 0 {
        goal = priced;
    }
    let mut from_the_answer = false;
    if goal == 0 {
        // The goal the answer above worked out; an answer that already took the pot off is the
        // money left, not the goal.
        goal = _the_goal_in_the_answer(memory);
        let last_bot = memory.turns.last().map(|t| t.bot.clone()).unwrap_or_default();
        from_the_answer = goal != 0 && pt::lower(&last_bot).contains("still to find");
    }
    let mut pot = if !from_the_answer { _the_money_already_held(memory) } else { 0 };
    if goal != 0 && 0 < pot && pot < goal {
        goal -= pot;
    } else {
        pot = 0;
    }
    if goal == 0 || goal < 100 {
        return String::new();
    }
    if !previous.is_empty() && _a_length_was_asked(&previous) {
        return String::new(); // asked twice: the answer above already gave the length
    }
    if _the_rate_in_the_chat(memory) != 0 || !_the_date_in_the_chat(memory).is_empty() {
        let rate = _the_rate_in_the_chat(memory);
        if rate != 0 && goal != 0 && priced != 0 && _the_date_in_the_chat(memory).is_empty() {
            return _the_length_at_that_rate(goal, rate, "month", pot);
        }
        if rate != 0 && rate < goal {
            let months = ceil_div(goal, rate);
            let last = memory.turns.last().map(|t| t.bot.clone()).unwrap_or_default();
            if !Re::new(&format!(r"\b{months}\s+months?\b"), 0).is_match(&last) {
                return without_what_was_just_said(&_a_rate_against_the_goal(goal, rate, memory), memory);
            }
        }
        return String::new();
    }
    let (rate, period) = a_rate_in_another_period(memory);
    if rate != 0 {
        return _the_length_at_that_rate(goal, rate, &period, pot);
    }
    let slow = ceil_div(goal, 100);
    let fast = ceil_div(goal, 250);
    let income = _the_income_in_the_chat(memory);
    if income != 0 {
        let said_goal = if pot != 0 { format!("the £{} still to find", commas(goal)) } else { format!("£{}", commas(goal)) };
        return format!(
            "How much of the £{} can you put by each month? At £100 a month {said_goal} takes {}.",
            commas(income),
            _months(slow)
        );
    }
    let said = if pot != 0 { format!("the £{} still to find takes ", commas(goal)) } else { format!("£{} takes ", commas(goal)) };
    if fast == slow {
        return format!(
            "At £100 a month, {said}{}. Pick the rate your pay can carry and set the standing order for the day after payday.",
            _months(slow)
        );
    }
    format!("At £100 a month, {said}{} - or {} at £250 a month. Pick the one your pay can carry.", _months(slow), _months(fast))
}

/// Python: bot/memory/saving.py:736 `_only_a_date(text)`
///
/// The month when the line is nothing but a date, or "".
pub fn _only_a_date(text: &str) -> String {
    let bare = pt::strip(text);
    let bare = pt::strip(&CORRECTING.sub("", bare)).to_string();
    if let Some(relative) = BARE_END_OF_RELATIVE.match_(&bare) {
        return format!("the end of {}", pt::squash(&pt::lower(relative.str(1))));
    }
    let Some(found) = BARE_DATE.match_(&bare) else {
        return String::new();
    };
    let word = pt::lower(found.str(1));
    if pt::lower(found.as_str()).contains("end of") {
        return format!("the end of {word}");
    }
    word
}

/// Python: bot/memory/saving.py:752 `is_only_a_date(text)`
pub fn is_only_a_date(text: &str) -> bool {
    !_only_a_date(text).is_empty()
}

/// Python: bot/memory/saving.py:762 `_only_a_sum(text)`
///
/// The sum when the line corrects a figure with nothing but a figure, or 0.
pub fn _only_a_sum(text: &str) -> i64 {
    let bare = pt::strip(&CORRECTING.sub("", text)).to_string();
    if bare.is_empty() || pt::split_ref(&bare).len() > 4 {
        return 0;
    }
    let aim = a_sum(&bare);
    let rest = NOT_A_LETTER.sub("", &pt::lower(&SUM.sub("", &bare)));
    if !pt::strip(&rest).is_empty() {
        return 0; // the line says more than the figure
    }
    if aim == 0 {
        let figure = NOT_A_DIGIT.sub("", &bare);
        if pt::len(&figure) >= 3 {
            if let Some(n) = py_int(&figure) {
                if n >= 20 {
                    return n;
                }
            }
        }
        return 0;
    }
    aim
}

/// Python: bot/memory/saving.py:788 `asks_about_saving(text)`
pub fn asks_about_saving(text: &str) -> bool {
    _names_a_goal(text) != 0
}

/// Python: bot/memory/saving.py:797 `_the_money_still_to_find(memory)`
///
/// The goal less the money already in the pot.
pub fn _the_money_still_to_find(memory: &Memory) -> i64 {
    let whole = _the_goal_in_the_chat(memory);
    let held = _the_money_already_held(memory);
    if whole != 0 && 0 < held && held < whole {
        return whole - held;
    }
    whole
}

/// Python: bot/memory/saving.py:811 `_the_goal_in_the_chat(memory)`
pub fn _the_goal_in_the_chat(memory: &Memory) -> i64 {
    _the_goal_named_in_a_line(memory)
}

/// Python: bot/memory/saving.py:816 `_the_goal_named_in_a_line(memory)`
///
/// The sum a line puts beside a saving word ("im saving 3000 for a car"), or 0.
pub fn _the_goal_named_in_a_line(memory: &Memory) -> i64 {
    let lines = users(memory);
    for line in last_n(&lines, 6).iter().rev() {
        let corrected = _only_a_sum(line);
        if corrected != 0 {
            return corrected;
        }
        if the_pot_said_alone(line) != 0 {
            continue;
        }
        let aim = _names_a_goal(line);
        if aim != 0 {
            return aim;
        }
    }
    0
}

/// Python: bot/memory/saving.py:853 `_the_goal_said_as_a_price(memory)`
///
/// The sum said of the thing they are saving for, when no line named a goal, or 0.
pub fn _the_goal_said_as_a_price(memory: &Memory) -> i64 {
    let lines = users(memory);
    let recent = last_n(&lines, 6);
    if !recent.iter().any(|line| SAVING.is_match(line) && (a_sum(line) == 0 || _a_rate(line))) {
        return 0;
    }
    for line in recent.iter().rev() {
        if line.is_empty() || the_pot_said_alone(line) != 0 {
            continue;
        }
        if the_pot_they_are_on(line) != 0 {
            continue;
        }
        if A_RATE.is_match(line) || A_RATE_ANY_OTHER_PERIOD.is_match(line) {
            continue; // a rate, not a price
        }
        if !(PRICE_WORDS.is_match(line) || A_TRAILING_SUM.is_match(line)) {
            continue;
        }
        if NOT_A_PRICE.is_match(line) {
            continue;
        }
        let aim = a_sum(line);
        if aim != 0 && aim >= 100 {
            return aim;
        }
    }
    0
}

/// Python: bot/memory/saving.py:908 `a_length_was_said(line)`
pub fn a_length_was_said(line: &str) -> bool {
    A_LENGTH_SAID.is_match(line)
}

/// Python: bot/memory/saving.py:918 `_a_length_was_asked(line)`
pub fn _a_length_was_asked(line: &str) -> bool {
    A_LENGTH_WAS_ASKED.is_match(line)
}

/// Python: bot/memory/saving.py:923 `_the_line_before(memory)`
pub fn _the_line_before(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.user.clone()).unwrap_or_default()
}

/// Python: bot/memory/saving.py:931 `_the_goal_in_the_answer(memory)`
///
/// The pot stated in the bot's last few answers, or 0.
pub fn _the_goal_in_the_answer(memory: &Memory) -> i64 {
    let answered: Vec<&String> = memory.turns.iter().filter(|t| !t.bot.is_empty()).map(|t| &t.bot).collect();
    for bot in last_n(&answered, 3).iter().rev() {
        for found in A_GOAL_STATED.finditer(bot) {
            let Some(mut value) = py_float(&found.str(1).replace(',', "")) else {
                continue;
            };
            if pt::lower(found.str(2)) == "k" {
                value *= 1000.0;
            }
            let value = value.trunc() as i64;
            if value >= 100 {
                return value;
            }
        }
    }
    0
}

/// Python: bot/memory/saving.py:965 `another_sum_asked(text, memory)`
///
/// The line read as a saving goal on a new sum, or "".
pub fn another_sum_asked(text: &str, memory: &Memory) -> String {
    if ANOTHER_SUM.match_(text).is_none() && A_GOAL_SUM.match_(text).is_none() {
        return String::new();
    }
    let lines = users(memory);
    let talked_money = last_n(&lines, 2).iter().any(|line| _only_a_sum(line) != 0 || _names_a_goal(line) != 0);
    if !talked_money {
        return String::new();
    }
    let figure = a_sum(text);
    if figure == 0 || figure < 100 {
        return String::new();
    }
    format!("save £{}", commas(figure))
}

/// Python: bot/memory/saving.py:999 `_a_rate_is_settled(memory)`
///
/// True when a rate the person gave is already in the chat.
pub fn _a_rate_is_settled(memory: &Memory) -> bool {
    let lines = users(memory);
    last_n(&lines, 4).iter().any(|line| A_RATE.is_match(line))
}

/// Python: bot/memory/saving.py:1018 `_the_length_at_that_rate(goal, rate, period, pot=0)`
///
/// The goal at a rate given in weeks, fortnights or years (or months), in that same unit.
pub fn _the_length_at_that_rate(goal: i64, rate: i64, period: &str, pot: i64) -> String {
    let count = ceil_div(goal, rate);
    let said = if pot != 0 { format!("the £{} still to find takes ", commas(goal)) } else { format!("£{} takes ", commas(goal)) };
    let unit = if count == 1 { period.to_string() } else { format!("{period}s") };
    let mut line = format!("At £{} a {period}, {said}{count} {unit}", commas(rate));
    if period == "year" {
        return line + ".";
    }
    let Some(factor) = MONTHS_IN_A.get(period) else {
        panic!("KeyError: '{period}'");
    };
    let months = py_round(count as f64 * factor);
    if period != "month" && months >= 2 {
        line += &format!(" (about {})", _months(months));
    }
    line + "."
}

/// `_a_rate_in_another_period` as (rate, period).
fn a_rate_in_another_period(memory: &Memory) -> (i64, String) {
    let lines = users(memory);
    for line in last_n(&lines, 6).iter().rev() {
        let Some(found) = A_RATE_ANY_OTHER_PERIOD.search(line) else {
            continue;
        };
        let Some(value) = py_int(&found.str(1).replace(',', "")) else {
            continue;
        };
        if value >= 5 {
            return (value, pt::lower(found.str(2)));
        }
    }
    (0, String::new())
}

/// Python: bot/memory/saving.py:1037 `_a_rate_in_another_period(memory)`
///
/// The rate and the period when the person gave one in weeks, fortnights or years.
pub fn _a_rate_in_another_period(memory: &Memory) -> Value {
    let (rate, period) = a_rate_in_another_period(memory);
    Value::Tuple(Arc::new(vec![Value::Int(rate), Value::from(period)]))
}

/// Python: bot/memory/saving.py:1072 `_is_an_income(line, found)`
///
/// True when the figure starting at byte `start` is what the person is paid.
pub fn _is_an_income(line: &str, start: usize) -> bool {
    let before = CLAUSE_EDGE.split(&line[..start]).pop().unwrap_or_default();
    INCOME_FRAME.is_match(&before)
}

/// Python: bot/memory/saving.py:1078 `_is_an_outgoing(line, found)`
///
/// True when the figure spanning bytes `start..end` is money that goes out each month.
pub fn _is_an_outgoing(line: &str, start: usize, end: usize) -> bool {
    // `text[max(0, start - 40):end + 40]`, forty code points either side.
    let s = pt::char_index(line, start) as isize;
    let e = pt::char_index(line, end) as isize;
    OUTGOING_FRAME.is_match(&pt::slice(line, (s - 40).max(0), Some(e + 40)))
}

/// Python: bot/memory/saving.py:1084 `_is_not_a_rate(line, found)`
pub fn _is_not_a_rate(line: &str, start: usize, end: usize) -> bool {
    _is_an_income(line, start) || _is_an_outgoing(line, start, end)
}

/// Python: bot/memory/saving.py:1088 `_the_income_in_the_chat(memory)`
///
/// The month's money the person says they are paid, or 0.
pub fn _the_income_in_the_chat(memory: &Memory) -> i64 {
    let lines = users(memory);
    for line in last_n(&lines, 6).iter().rev() {
        for found in A_RATE.finditer(line) {
            if !_is_an_income(line, found.start()) {
                continue;
            }
            let value = _a_sum_named(found.str(1));
            if value != 0 {
                return value;
            }
        }
    }
    0
}

/// Python: bot/memory/saving.py:1185 `_is_a_rate_line(line)`
///
/// True when the line puts money by at a rate, whatever else it says.
pub fn _is_a_rate_line(line: &str) -> bool {
    A_RATE.finditer(line).iter().any(|found| !_is_not_a_rate(line, found.start(), found.end()))
}

/// Python: bot/memory/saving.py:1193 `a_rate_put_right(line, before='')`
///
/// The monthly rate a line puts right, or 0.
pub fn a_rate_put_right(line: &str, before: &str) -> i64 {
    if !before.is_empty()
        && A_MORE_LIKE.is_match(line)
        && A_RATE_PUT_RIGHT.is_match(line)
        && _is_a_rate_line(before)
        && !_OTHER_FIGURE_WORDS.is_match(line)
    {
        let figures = A_LONE_SUM.findall_str(line);
        if figures.len() == 1 {
            return _a_sum_named(&figures[0]);
        }
    }
    if let Some(after_a_no) = A_NO_THEN_A_RATE.search(line) {
        if !A_SPENT_ON.is_match(&line[after_a_no.end()..]) {
            return _a_sum_named(after_a_no.str(1));
        }
    }
    if !A_RATE_PUT_RIGHT.is_match(line) || _OTHER_FIGURE_WORDS.is_match(line) {
        return 0;
    }
    if let Some(with_the_month) = A_SUM_A_MONTH.search(line) {
        return _a_sum_named(with_the_month.str(1));
    }
    let figures = A_LONE_SUM.findall_str(line);
    if figures.len() != 1 {
        return 0;
    }
    if !A_CAN_CARRY_IT.is_match(line) {
        // Only a correction that names the figure and nothing else is still this chat's rate.
        if A_CORRECTION_AND_A_FIGURE.match_(pt::strip(line)).is_none() || _NOT_THE_RATE.is_match(line) {
            return 0;
        }
    }
    _a_sum_named(&figures[0])
}

/// Python: bot/memory/saving.py:1229 `_a_sum_named(figure)`
///
/// One sum as a number, or 0 when it is not a rate anybody could put by.
pub fn _a_sum_named(figure: &str) -> i64 {
    match py_int(&figure.replace(',', "")) {
        Some(value) if value >= 10 => value,
        _ => 0,
    }
}

/// Python: bot/memory/saving.py:1238 `_the_rate_in_the_chat(memory)`
///
/// The sum a month the person has settled on, reading the last few lines in order, or 0.
pub fn _the_rate_in_the_chat(memory: &Memory) -> i64 {
    let lines = users(memory);
    let mut settled = 0;
    let mut before = String::new();
    for line in last_n(&lines, 6) {
        for found in A_RATE.finditer(line) {
            if _is_not_a_rate(line, found.start(), found.end()) {
                continue;
            }
            let value = _a_sum_named(found.str(1));
            if value != 0 {
                settled = value;
            }
        }
        if settled != 0 {
            let put_right = a_rate_put_right(line, &before);
            if put_right != 0 {
                settled = put_right;
            }
        }
        before = line.clone();
    }
    settled
}

/// Python: bot/memory/saving.py:1275 `_the_date_in_the_chat(memory)`
///
/// The month the goal was put in, from the last few lines, or "".
pub fn _the_date_in_the_chat(memory: &Memory) -> String {
    let lines = users(memory);
    for line in last_n(&lines, 6).iter().rev() {
        let mut when = _only_a_date(line);
        if when.is_empty() {
            when = _a_date(line);
        }
        if !when.is_empty() {
            return when;
        }
    }
    String::new()
}

/// Python: bot/memory/saving.py:1286 `_months_left(when)`
///
/// The months from this one to the month the person named.
pub fn _months_left(when: &str) -> i64 {
    use chrono::Datelike;
    let today = crate::py::clock::today();
    let word = pt::lower(when);
    if word.contains("this month") || word.contains("this week") {
        return 0;
    }
    if word.contains("next month") {
        return 1;
    }
    let month = _month_of(when);
    if month.is_empty() {
        return 0;
    }
    let number = MONTHS.iter().position(|m| *m == month).unwrap() as i64 + 1;
    let this = today.month() as i64;
    let year_ahead = if number <= this { 1 } else { 0 };
    (year_ahead * 12 + (number - this)).max(0)
}

/// Python: bot/memory/saving.py:1305 `_months(n)`
///
/// The length as months, with the years in it once it is over a year.
pub fn _months(n: i64) -> String {
    let said = format!("{} month{}", commas(n), if n == 1 { "" } else { "s" });
    if n < 12 {
        return said;
    }
    let (years, months) = (n.div_euclid(12), n.rem_euclid(12));
    let mut parts = Vec::new();
    if years != 0 {
        parts.push(format!("{} year{}", commas(years), if years == 1 { "" } else { "s" }));
    }
    if months != 0 {
        parts.push(format!("{months} month{}", if months == 1 { "" } else { "s" }));
    }
    format!("{said}, about {}", parts.join(" and "))
}

/// Python: bot/memory/saving.py:1324 `a_date_on_a_settled_rate(text, memory)`
///
/// The answer to a bare date asked after the rate was already chosen, or "".
pub fn a_date_on_a_settled_rate(text: &str, memory: &Memory) -> String {
    let when = _only_a_date(text);
    if when.is_empty() || !_a_rate_is_settled(memory) {
        return String::new();
    }
    let (rate, goal) = (_the_rate_in_the_chat(memory), _the_money_still_to_find(memory));
    _a_date_on_a_rate(goal, rate, &when)
}

/// Python: bot/memory/saving.py:1340 `_a_date_on_a_rate(goal, rate, when)`
///
/// The rate against the month it has to reach: whether it gets there, and what would.
pub fn _a_date_on_a_rate(goal: i64, rate: i64, when: &str) -> String {
    let left = _months_left(when);
    if rate == 0 || goal == 0 || goal <= rate || left < 1 {
        return String::new();
    }
    let months = ceil_div(goal, rate);
    let name = if when.starts_with("the end of ") { when.to_string() } else { pt::capitalize(when) };
    let goal_text = format!("£{}", commas(goal));
    let rate_text = format!("£{} a month", commas(rate));
    if months <= left {
        return format!(
            "{goal_text} at {rate_text} takes {} and {name} is {} away, so that rate gets you there. Set the standing order for the day after payday and it looks after itself \u{2014} a bonus or a refund just brings the date forward.",
            _months(months),
            _months(left)
        );
    }
    let need = ceil_div(goal, left);
    let need = ceil_div(need, 10) * 10;
    let part = (rate * left).min(goal);
    format!(
        "{goal_text} at {rate_text} takes {} and {name} is {} away, so it does not get you there \u{2014} that would take about £{} a month. At {rate_text} you have £{} of it by then; a bonus, a bill that drops or a later date closes the gap.",
        _months(months),
        _months(left),
        commas(need),
        commas(part)
    )
}

/// Python: bot/memory/saving.py:1386 `asks_if_the_rate_is_enough(text)`
pub fn asks_if_the_rate_is_enough(text: &str) -> bool {
    ASKS_IF_ENOUGH.match_(text).is_some()
}

/// Python: bot/memory/saving.py:1391 `a_yes_no_about_the_rate(text, memory)`
///
/// The verdict on the rate the chat holds, in its short form, or "".
pub fn a_yes_no_about_the_rate(text: &str, memory: &Memory) -> String {
    if !asks_if_the_rate_is_enough(text) {
        return String::new();
    }
    let (goal, rate) = (_the_money_still_to_find(memory), _the_rate_in_the_chat(memory));
    if goal == 0 || rate == 0 {
        return String::new();
    }
    let months = ceil_div(goal, rate);
    let when = _the_date_in_the_chat(memory);
    let left = if when.is_empty() { 0 } else { _months_left(&when) };
    let goal_text = format!("£{}", commas(goal));
    let rate_only = format!("£{} a month", commas(rate));
    let rate_text = format!("at {rate_only}");
    if when.is_empty() || left == 0 {
        return format!(
            "{goal_text} {rate_text} takes {}. Name the month you want it by and I will tell you whether that is soon enough.",
            _months(months)
        );
    }
    if months <= left {
        let spare = left - months;
        let room = if spare != 0 { format!("{spare} month{} to spare", if spare == 1 { "" } else { "s" }) } else { "nothing to spare".to_string() };
        return format!(
            "Yes \u{2014} {rate_only} gets you to {goal_text} by {}, {room}. Leave the standing order where it is.",
            pt::capitalize(&when)
        );
    }
    let part = (rate * left).min(goal);
    format!(
        "No \u{2014} {rate_text} you have £{} of {goal_text} by {}. The whole {goal_text} takes {} at that rate, so the rest wants more a month or a later date.",
        commas(part),
        pt::capitalize(&when),
        _months(months)
    )
}

/// Python: bot/memory/saving.py:1438 `asks_how_much_more_a_month(text)`
pub fn asks_how_much_more_a_month(text: &str) -> bool {
    ASKS_HOW_MUCH_MORE.match_(text).is_some()
}

/// Python: bot/memory/saving.py:1443 `the_extra_a_month_asked(text, memory)`
///
/// The extra a month the date needs, or "".
pub fn the_extra_a_month_asked(text: &str, memory: &Memory) -> String {
    if !asks_how_much_more_a_month(text) {
        return String::new();
    }
    let (goal, rate) = (_the_money_still_to_find(memory), _the_rate_in_the_chat(memory));
    let when = _the_date_in_the_chat(memory);
    if goal == 0 || rate == 0 || when.is_empty() {
        return String::new();
    }
    let left = _months_left(&when);
    if left == 0 {
        return String::new();
    }
    let months = ceil_div(goal, rate);
    if months <= left {
        let spare = left - months;
        return format!(
            "Nothing more \u{2014} £{} a month already gets you to £{} by {} with {spare} month{} to spare.",
            commas(rate),
            commas(goal),
            pt::capitalize(&when),
            if spare == 1 { "" } else { "s" }
        );
    }
    let need = ceil_div(ceil_div(goal, left), 10) * 10;
    let more = (need - rate).max(0);
    let left_text = if left == 1 { "the month that is left".to_string() } else { format!("the {} left", _months(left)) };
    format!(
        "About £{} a month more: £{} a month for {left_text}, rather than the £{} you have. Or give it longer \u{2014} at £{} a month the whole £{} takes {}.",
        commas(more),
        commas(need),
        commas(rate),
        commas(rate),
        commas(goal),
        _months(months)
    )
}

/// Python: bot/memory/saving.py:1475 `the_same_figure_a_month(text, memory)`
///
/// The rate asked for in months, said in months, when the chat already gave it in weeks, or "".
pub fn the_same_figure_a_month(text: &str, memory: &Memory) -> String {
    if !HOW_MUCH.is_match(text) {
        return String::new();
    }
    for turn in last_n(&memory.turns, 4).iter().rev() {
        let Some(found) = A_WEEK_FIGURE.search(&turn.bot) else {
            continue;
        };
        let Some(a_week) = py_int(&found.str(1).replace(',', "")) else {
            return String::new();
        };
        if a_week < 5 {
            return String::new();
        }
        let a_month = py_round(a_week as f64 * 52.0 / 12.0 / 10.0) * 10;
        return format!(
            "£{} a week is about £{} a month. Set the standing order for the day after payday and it runs itself \u{2014} a bonus or a refund only shortens the time.",
            commas(a_week),
            commas(a_month)
        );
    }
    String::new()
}

/// A rate read off a match: `float(group(1))`, a thousand for "k", rounded.
fn rate_of(figure: &str, k: &str) -> Option<i64> {
    let mut rate = py_float(&figure.replace(',', ""))?;
    if !k.is_empty() {
        rate *= 1000.0;
    }
    Some(py_round(rate))
}

/// Python: bot/memory/saving.py:1511 `a_rate_restated(text, memory)`
///
/// The goal worked out at the rate the line puts in place of the old one, or "".
pub fn a_rate_restated(text: &str, memory: &Memory) -> String {
    let line = text;
    let Some(said) = A_NEW_RATE.match_(line) else {
        return String::new();
    };
    if PAST_SAVING.is_match(line) || OTHER_POT.is_match(line) {
        return String::new();
    }
    if !_only_a_date(line).is_empty() || !_a_date(line).is_empty() {
        return String::new();
    }
    let Some(rate) = rate_of(said.str(1), said.str(2)) else {
        return String::new();
    };
    if rate < 20 {
        return String::new();
    }
    let aim = _the_goal_in_the_chat(memory);
    if aim == 0 || rate >= aim {
        return String::new();
    }
    if rate == _the_rate_in_the_chat(memory) {
        return String::new();
    }
    without_what_was_just_said(&_a_rate_against_the_goal(aim, rate, memory), memory)
}

/// Python: bot/memory/saving.py:1558 `a_rate_put_in_with_a_verb(text, memory)`
///
/// The goal at a new rate when the figure comes in without its unit, or "".
pub fn a_rate_put_in_with_a_verb(text: &str, memory: &Memory) -> String {
    let line = text;
    if pt::strip(line).is_empty() || A_RATE.is_match(line) || A_NEW_RATE.match_(line).is_some() {
        return String::new();
    }
    if !_only_a_date(line).is_empty() || !_a_date(line).is_empty() {
        return String::new();
    }
    let Some(said) = A_RATE_WITH_A_VERB.match_(line) else {
        return String::new();
    };
    if PAST_SAVING.is_match(line) || OTHER_POT.is_match(line) {
        return String::new();
    }
    if !_a_rate_is_settled(memory) {
        return String::new();
    }
    let Some(rate) = rate_of(said.str(1), said.str(2)) else {
        return String::new();
    };
    if rate < 20 {
        return String::new();
    }
    let mut aim = _the_goal_in_the_chat(memory);
    if aim == 0 {
        aim = _the_goal_in_the_answer(memory);
    }
    if aim == 0 || rate >= aim {
        return String::new();
    }
    if rate == _the_rate_in_the_chat(memory) {
        return String::new();
    }
    without_what_was_just_said(&_a_rate_against_the_goal(aim, rate, memory), memory)
}

/// Python: bot/memory/saving.py:1606 `a_length_at_the_rate_in_the_chat(text, memory)`
///
/// The money question about the goal in the line at the rate already in the chat, or "".
pub fn a_length_at_the_rate_in_the_chat(text: &str, memory: &Memory) -> String {
    let line = text;
    if pt::strip(line).is_empty() || !ASKS_A_LENGTH.is_match(line) {
        return String::new();
    }
    if A_RATE.is_match(line) || A_NEW_RATE.match_(line).is_some() || A_RATE_WITH_A_VERB.match_(line).is_some() {
        return String::new();
    }
    if !_the_date_in_the_chat(memory).is_empty() {
        return String::new();
    }
    let said = a_sum(line);
    if said == 0 {
        return String::new();
    }
    let rate = _the_rate_in_the_chat(memory);
    if rate == 0 || rate >= said {
        return String::new();
    }
    format!("How long will it take to save {} at {} a month?", commas(said), commas(rate))
}

/// Python: bot/memory/saving.py:1630 `a_rate_with_no_date(text, memory)`
///
/// The rate asked for when the goal is in the chat but no date is, or "".
pub fn a_rate_with_no_date(text: &str, memory: &Memory) -> String {
    let line = text;
    if !HOW_MUCH.is_match(line) || a_sum(line) != 0 || !_only_a_date(line).is_empty() || !_a_date(line).is_empty() {
        return String::new();
    }
    let aim = _the_goal_in_the_chat(memory);
    if aim == 0 || !_the_date_in_the_chat(memory).is_empty() {
        return String::new();
    }
    format!(
        "£{} is a different rate over six months than over a year, so name the month you want it by and I will work out what to put by each month \u{2014} and whether a standing order gets you there.",
        commas(aim)
    )
}

/// Python: bot/memory/saving.py:1648 `asks_the_rate(text, memory)`
///
/// True when the line asks for the rate and names neither the goal nor the date.
pub fn asks_the_rate(text: &str, memory: &Memory) -> bool {
    if !HOW_MUCH.is_match(text) || a_sum(text) != 0 {
        return false;
    }
    if !_only_a_date(text).is_empty() || !_a_date(text).is_empty() {
        return false;
    }
    _the_goal_in_the_chat(memory) != 0
}

/// Python: bot/memory/saving.py:1668 `_in_the_money_readers_words(when)`
///
/// The date as the knowledge base can count it: a named day becomes the month it falls in.
pub fn _in_the_money_readers_words(when: &str) -> String {
    let word = pt::strip(&pt::lower(when)).to_string();
    if word.is_empty() || MONTHS.contains(&word) || KNOWN_TO_THE_MONEY_ENTRY.contains(&word) {
        return word;
    }
    if let Some(rest) = word.strip_prefix("the end of ") {
        let rest = pt::strip(rest).to_string();
        if matches!(rest.as_str(), "next year" | "this year" | "the year") {
            return word;
        }
        let season = rest.strip_prefix("the ").unwrap_or(&rest).to_string();
        let month = match THE_END_OF_A_SEASON.get(&season).filter(|m| !m.is_empty()) {
            Some(month) => month.clone(),
            None => _in_the_money_readers_words(&rest),
        };
        return if month.is_empty() { word } else { format!("the end of {month}") };
    }
    for (name, month) in NAMED_WHEN.iter() {
        if word.contains(name.as_str()) {
            return month.clone();
        }
    }
    word
}

/// Python: bot/memory/saving.py:1700 `rate_question(text, memory)`
///
/// The money question this line really is, or "" when it is not one.
pub fn rate_question(text: &str, memory: &Memory) -> String {
    let line = text;
    if pt::strip(line).is_empty() {
        return String::new();
    }
    if HOW_MUCH.is_match(line) && a_sum(line) == 0 {
        let (aim, when) = (_the_goal_in_the_chat(memory), _the_date_in_the_chat(memory));
        if aim == 0 {
            return String::new();
        }
        if !when.is_empty() {
            return format!(
                "How much do i need to put by a week to have {} pounds by {}?",
                commas(aim),
                _in_the_money_readers_words(&when)
            );
        }
        return format!("How much do i need to put by a week to have {} pounds?", commas(aim));
    }
    let (mut aim, when);
    if _names_a_goal(line) != 0 {
        aim = _names_a_goal(line);
        when = _a_date(line);
    } else {
        when = _only_a_date(line);
        if when.is_empty() || _a_rate_is_settled(memory) {
            return String::new();
        }
        aim = _the_goal_in_the_chat(memory);
    }
    if aim == 0 {
        return String::new();
    }
    let mut held = pot_and_rest(line).0;
    if held == 0 {
        held = _the_money_already_held(memory);
    }
    if held != 0 && 0 < held && held < aim {
        aim -= held;
    }
    if !when.is_empty() {
        return format!(
            "How much do i need to put by a week to have {} pounds by {}?",
            commas(aim),
            _in_the_money_readers_words(&when)
        );
    }
    format!("How much do i need to put by a week to have {} pounds?", commas(aim))
}

/// `_a_rate_said` as (rate, whether the period was named).
fn a_rate_said(line: &str) -> (i64, bool) {
    if let Some(found) = A_RATE.search(line) {
        return match py_int(&found.str(1).replace(',', "")) {
            Some(rate) => (rate, true),
            None => (0, false),
        };
    }
    if INTENT.is_match(line) || MONEY_SAID.is_match(line) {
        return (0, false);
    }
    let rest = pt::strip_chars(&FILLER_LEAD.sub("", line), " ,.").to_string();
    let Some(verb) = RATE_GIVEN.match_(&rest).or_else(|| RATE_VERB.match_(&rest)) else {
        return (0, false);
    };
    if SAVE_VERB.match_(&rest).is_some() && !CONDITIONAL.is_match(line) {
        return (0, false);
    }
    let rest = pt::strip_chars(&rest[verb.end()..], " ,.").to_string();
    let Some(found) = BARE_RATE.fullmatch(&rest) else {
        return (0, false);
    };
    match py_int(&found.str(1).replace(',', "")) {
        Some(rate) => (rate, false),
        None => (0, false),
    }
}

/// Python: bot/memory/saving.py:1767 `_a_rate_said(line)`
///
/// The sum a month the person says they will put by, and whether they named the period.
pub fn _a_rate_said(line: &str) -> Value {
    let (rate, named) = a_rate_said(line);
    Value::Tuple(Arc::new(vec![Value::Int(rate), Value::Bool(named)]))
}

/// Python: bot/memory/saving.py:1803 `a_rate_for_the_goal(text, memory)`
///
/// The arithmetic for a rate the person states, when the goal is already in the chat, or "".
pub fn a_rate_for_the_goal(text: &str, memory: &Memory) -> String {
    let line = text;
    if pt::strip(line).is_empty() {
        return String::new();
    }
    if !RATE_GIVEN.is_match(line) && !RATE_VERB.is_match(line) {
        return String::new();
    }
    if _only_a_sum(line) != 0 || !_only_a_date(line).is_empty() {
        return String::new();
    }
    if FOR_A_LENGTH.is_match(line) {
        return String::new(); // a rate and a length: the money entry's multiplication
    }
    let (rate, _period) = a_rate_said(line);
    if rate == 0 {
        return String::new();
    }
    if let Some(found) = A_RATE.search(line) {
        let tail = &line[found.end()..];
        if NOT_A_POT.match_(tail).is_some() || OTHER_POT.match_(tail).is_some() {
            return String::new();
        }
    }
    let goal = _the_money_still_to_find(memory);
    if goal <= rate {
        return String::new();
    }
    let when = _the_date_in_the_chat(memory);
    if !when.is_empty() {
        let against = _a_date_on_a_rate(goal, rate, &when);
        if !against.is_empty() {
            return against;
        }
    }
    let months = ceil_div(goal, rate);
    let closing = _a_closer(memory);
    let said = format!("£{} at £{} a month is {}.", commas(goal), commas(rate), _months(months));
    if closing.is_empty() { said } else { format!("{said} {closing}") }
}

/// Python: bot/memory/saving.py:1861 `_the_added_sum(line)`
///
/// The sum the line adds to the rate, or 0 when it names none.
pub fn _the_added_sum(line: &str) -> i64 {
    let Some(found) = A_FIGURE_ADDED.search(line) else {
        return 0;
    };
    let Some(value) = figure_and_unit(found.str(1), found.str(2)) else {
        return 0;
    };
    let value = value.trunc() as i64;
    if !(5..=5000).contains(&value) {
        return 0;
    }
    value
}

/// Python: bot/memory/saving.py:1878 `a_rate_raised(text, memory)`
///
/// The goal worked out again at the rate the line raises it to, or "".
pub fn a_rate_raised(text: &str, memory: &Memory) -> String {
    let line = text;
    if pt::strip(line).is_empty() {
        return String::new();
    }
    let mut goal = _the_money_still_to_find(memory);
    if goal == 0 {
        goal = _the_goal_in_the_answer(memory);
    }
    let rate = _the_rate_in_the_chat(memory);
    if goal == 0 || rate == 0 || goal <= rate {
        return String::new();
    }
    if let Some(restated) = A_RESTATED_RATE.match_(line) {
        let Some(named) = py_int(&restated.str(1).replace(',', "")) else {
            return String::new();
        };
        if named <= rate || named >= goal {
            return String::new();
        }
        if _said_a_rate_at(named, memory) {
            return format!(
                "That is the rate: put the standing order in for £{} the day after payday and the £{} takes care of itself from there.",
                commas(named),
                commas(goal)
            );
        }
        return _a_rate_against_the_goal(goal, named, memory);
    }
    if !ADDED.is_match(line) {
        return String::new();
    }
    if _names_a_goal(line) != 0 {
        return String::new();
    }
    let added = _the_added_sum(line);
    if added == 0 {
        return String::new();
    }
    let raised = rate + added;
    if raised >= goal {
        return String::new();
    }
    _a_rate_against_the_goal(goal, raised, memory)
}

/// Python: bot/memory/saving.py:1922 `_said_a_rate_at(rate, memory)`
///
/// True when the bot's last answer already worked the goal out at this rate.
pub fn _said_a_rate_at(rate: i64, memory: &Memory) -> bool {
    let Some(last) = last_bot_with_a_reply(memory) else {
        return false;
    };
    Re::new(&format!(r"£\s*{}\s*a\s+month", commas(rate).replace(',', ",?")), 0).is_match(&last)
}

/// Python: bot/memory/saving.py:1931 `_a_rate_against_the_goal(goal, rate, memory)`
///
/// The months the goal takes at this rate, and a closing line the reply above did not use.
pub fn _a_rate_against_the_goal(goal: i64, rate: i64, memory: &Memory) -> String {
    let months = ceil_div(goal, rate);
    let said = format!("£{} at £{} a month is {}.", commas(goal), commas(rate), _months(months));
    let closing = _a_closer(memory);
    if closing.is_empty() { said } else { format!("{said} {closing}") }
}

/// Python: bot/memory/saving.py:1952 `_a_closer(memory)`
///
/// A closing line the reply just above did not already use, or "".
pub fn _a_closer(memory: &Memory) -> String {
    let last = pt::lower(&last_bot_with_a_reply(memory).unwrap_or_default());
    let said: HashSet<String> = WORDS.findall_str(&last).into_iter().collect();
    for closing in SAVING_CLOSERS.iter() {
        let words: Vec<String> = WORDS.findall_str(&pt::lower(closing)).into_iter().filter(|w| pt::len(w) > 3).collect();
        let here = words.iter().filter(|w| said.contains(*w)).count();
        if !words.is_empty() && here * 2 >= words.len() {
            continue;
        }
        return closing.clone();
    }
    String::new()
}

/// Python: bot/memory/saving.py:1969 `_a_said_sentence(sentence, said)`
///
/// True when most of the words of this sentence are in the reply above.
pub fn _a_said_sentence(sentence: &str, said: &HashSet<String>) -> bool {
    let words: Vec<String> = WORDS.findall_str(&pt::lower(sentence)).into_iter().filter(|w| pt::len(w) > 3).collect();
    if words.len() < 4 {
        return false;
    }
    let here = words.iter().filter(|w| said.contains(*w)).count();
    here * 4 >= words.len() * 3
}

/// The words of the bot's last line that has a reply.
fn words_said_last(memory: &Memory) -> Option<HashSet<String>> {
    let last = last_bot_with_a_reply(memory)?;
    Some(WORDS.findall_str(&pt::lower(&last)).into_iter().collect())
}

/// Python: bot/memory/saving.py:1978 `said_already(reply, memory)`
///
/// True when the first sentence of `reply` is one the answer above already said.
pub fn said_already(reply: &str, memory: &Memory) -> bool {
    let first = SENTENCE_END.split(pt::strip(reply)).into_iter().next().unwrap_or_default();
    if first.is_empty() {
        return false;
    }
    let Some(said) = words_said_last(memory) else {
        return false;
    };
    !said.is_empty() && _a_said_sentence(&first, &said)
}

/// Python: bot/memory/saving.py:2003 `_weeks_left(when)`
///
/// The weeks from this one to the month the person named, rounded up.
pub fn _weeks_left(when: &str) -> i64 {
    use chrono::Datelike;
    let month = _month_of(when);
    if month.is_empty() {
        return 0;
    }
    let today = crate::py::clock::today();
    let number = MONTHS.iter().position(|m| *m == month).unwrap() as u32 + 1;
    let year = today.year() + if number <= today.month() { 1 } else { 0 };
    let first = chrono::NaiveDate::from_ymd_opt(year, number, 1).unwrap();
    let days = (first - today).num_days();
    ceil_div(days, 7).max(0)
}

/// Python: bot/memory/saving.py:2021 `a_figure_each_week(memory)`
///
/// The goal divided by the weeks left, said as one figure, or "".
pub fn a_figure_each_week(memory: &Memory) -> String {
    let (aim, when) = (_the_money_still_to_find(memory), _the_date_in_the_chat(memory));
    let weeks = _weeks_left(&when);
    if aim == 0 || weeks < 2 {
        return String::new();
    }
    let a_week = ceil_div(aim, weeks);
    let found = last_bot_with_a_reply(memory).and_then(|last| A_MONTH_FIGURE.search(&last).map(|m| m.str(1).to_string()));
    let mut said = match found {
        Some(figure) => format!("£{figure} a month is"),
        None => format!("£{} by {} is", commas(aim), pt::capitalize(&when)),
    };
    said += &format!(" about £{} a week over the {} weeks left", commas(a_week), commas(weeks));
    let closing = _a_closer(memory);
    if !closing.is_empty() {
        let mut c = closing.chars();
        let lowered: String = c.next().map(|f| f.to_lowercase().chain(c).collect()).unwrap_or_default();
        return format!("{said} \u{2014} {lowered}");
    }
    said + "."
}

/// Python: bot/memory/saving.py:2056 `the_figure_a_week(memory)`
///
/// The question that gets the figure each week, or "" when there is nothing to divide.
pub fn the_figure_a_week(memory: &Memory) -> String {
    if _the_goal_in_the_chat(memory) == 0 || _the_date_in_the_chat(memory).is_empty() {
        return String::new();
    }
    A_WEEK_ASK.clone()
}

/// Python: bot/memory/saving.py:2063 `without_what_was_just_said(reply, memory)`
///
/// `reply` with any sentence of it the reply above already said taken off (never the first).
pub fn without_what_was_just_said(reply: &str, memory: &Memory) -> String {
    if reply.is_empty() || reply.contains('\n') {
        return reply.to_string();
    }
    let Some(said) = words_said_last(memory) else {
        return reply.to_string();
    };
    if said.is_empty() {
        return reply.to_string();
    }
    let parts: Vec<String> = SENTENCE_END.split(pt::strip(reply)).into_iter().filter(|p| !pt::strip(p).is_empty()).collect();
    if parts.len() < 2 {
        return reply.to_string();
    }
    let mut kept = vec![parts[0].clone()];
    kept.extend(parts[1..].iter().filter(|p| !_a_said_sentence(p, &said)).cloned());
    kept.join(" ")
}

/// Python: bot/memory/saving.py:2134 `_the_rate_put_by(memory)`
///
/// The sum a month the person put by, from the last few lines, or 0.
pub fn _the_rate_put_by(memory: &Memory) -> i64 {
    let lines = users(memory);
    let window = last_n(&lines, 6);
    if !window.iter().any(|line| PUT_BY.is_match(line)) {
        return 0;
    }
    for line in window.iter().rev() {
        let Some(found) = A_RATE.search(line) else {
            continue;
        };
        if MONEY_OUT.is_match(line) {
            continue;
        }
        let Some(value) = py_int(&found.str(1).replace(',', "")) else {
            continue;
        };
        if value >= 10 {
            return value;
        }
    }
    0
}

/// Python: bot/memory/saving.py:2160 `a_length_with_no_price(text, memory)`
///
/// The rate said as the money it becomes, and the price asked for, or "".
pub fn a_length_with_no_price(text: &str, memory: &Memory) -> String {
    let line = pt::squash(&pt::lower(text));
    if line.is_empty() {
        return String::new();
    }
    if !ASKS_A_LENGTH_FOR_A_THING.is_match(&line) || !THE_THING_WANTED.is_match(&line) {
        return String::new();
    }
    if A_DIGIT.is_match(&line) {
        return String::new();
    }
    let rate = _the_rate_put_by(memory);
    if rate < 10 {
        return String::new();
    }
    if _the_goal_in_the_chat(memory) != 0 || _the_goal_in_the_answer(memory) != 0 {
        return String::new();
    }
    let said = format!("£{} a month is £{} every six months, £{} in a year.", commas(rate), commas(rate * 6), commas(rate * 12));
    let thing = match A_THING_NAMED.search(&line) {
        Some(named) if !NOT_A_THING.contains(&pt::lower(named.str(1))) => format!(" {}", pt::lower(named.str(1))),
        _ => String::new(),
    };
    let asked = if thing.is_empty() { "What's the price on it?".to_string() } else { format!("What's the{thing} going for?") };
    if _already_asked_for_the_price(memory) {
        return format!("{said} Tell me the price and I'll do the weeks.");
    }
    format!("{said} {asked}")
}

/// Python: bot/memory/saving.py:2194 `_already_asked_for_the_price(memory)`
///
/// True when the answer above already asked what the thing costs.
pub fn _already_asked_for_the_price(memory: &Memory) -> bool {
    match last_bot_with_a_reply(memory) {
        Some(above) => PRICE_ASKED.is_match(&pt::lower(&above)),
        None => false,
    }
}

/// Python: bot/memory/saving.py:2204 `a_price_in_an_answer(text, memory)`
///
/// The arithmetic for a price given in reply to "what's it going for?", or "".
pub fn a_price_in_an_answer(text: &str, memory: &Memory) -> String {
    let line = pt::squash(&pt::lower(text));
    if line.is_empty() || pt::split_ref(&line).len() > 7 {
        return String::new();
    }
    let price = a_sum(&line);
    if price == 0 || !_only_a_date(&line).is_empty() || !_a_date(&line).is_empty() {
        return String::new();
    }
    let rate = _the_rate_put_by(memory);
    if rate == 0 || rate >= price {
        return String::new();
    }
    if !_already_asked_for_the_price(memory) {
        return String::new();
    }
    let months = ceil_div(price, rate);
    format!("£{} at £{} a month is {}.", commas(price), commas(rate), _months(months))
}
