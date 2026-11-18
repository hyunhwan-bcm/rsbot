//! Port of `bot/memory/spoken.py`: a figure said in words ("half an hour", "two and a half
//! litres", "twenty five minutes") written in digits before anything reads the line.

use super::mem8_py::{int, rx, val};
use crate::py::re::Match;
use crate::py::text;

const M: &str = "bot.memory.spoken.";

rx!(_A_SPOKEN_FRACTION, _A_SPOKEN_HALF_HOUR, _A_RATE_UNIT, _A_COUNT_AND_A_HALF, _A_WORDED_FIGURE, _A_TENS_AND_UNITS);
val!(_SPOKEN_FRACTION_VALUE, _SPOKEN_NUMBER, _ONE_TO_NINE, _UNIT_WORDS);
int!(_MINUTES_CEILING);

/// `table.get(word)` as a number.
fn number(table: &crate::data::Value, word: &str) -> Option<f64> {
    table.get(word).and_then(|v| v.as_f64())
}

/// A Python number: an int or a float.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Num {
    Int(i64),
    Float(f64),
}

/// `int(x) if float(x).is_integer() else x`.
fn whole_or(x: f64) -> Num {
    if x.fract() == 0.0 { Num::Int(x as i64) } else { Num::Float(x) }
}

/// Python: bot/memory/spoken.py:64 `_the_number(value)`
///
/// The figure as a person writes it: "2", "1.5".
pub fn _the_number(value: Num) -> String {
    match value {
        Num::Int(i) => i.to_string(),
        Num::Float(f) => match whole_or(f) {
            Num::Int(i) => i.to_string(),
            Num::Float(f) => text::float_repr(f),
        },
    }
}

/// Python: bot/memory/spoken.py:70 `_is_hours(unit)`
pub fn _is_hours(unit: &str) -> bool {
    let u = text::strip(unit).to_lowercase();
    ["hour", "hr"].contains(&u.trim_end_matches('s'))
}

/// Python `round(x, 2)` for a float: the nearest float to the correctly rounded decimal.
fn round2(x: f64) -> f64 {
    format!("{x:.2}").parse().unwrap_or(x)
}

/// Python: bot/memory/spoken.py:74 `_in_digits(amount, unit)`
///
/// The amount and its unit in digits, a fraction of an hour said in minutes.
pub fn _in_digits(amount: f64, unit: &str) -> String {
    if _is_hours(unit) && amount.fract() != 0.0 {
        let minutes = amount * 60.0;
        if minutes <= *_MINUTES_CEILING as f64 {
            return format!("{} minutes", _the_number(whole_or(minutes)));
        }
        return format!("{} hours", _the_number(Num::Float(round2(amount))));
    }
    let plural = if amount != 1.0 && !unit.ends_with('s') { "s" } else { "" };
    format!("{} {}{}", _the_number(whole_or(amount)), unit, plural)
}

/// Python: bot/memory/spoken.py:87 `_spoken_as_numbers(text)` (also `spoken_as_numbers`)
///
/// The line with its spoken figures written in digits.
pub fn _spoken_as_numbers(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let count_and_a_half = |m: &Match| {
        let count = number(&_UNIT_WORDS, &m.str(1).to_lowercase()).unwrap_or(0.0);
        if count == 0.0 {
            return m.as_str().to_string();
        }
        let part = if m.str(2).to_lowercase() == "half" { 0.5 } else { 0.25 };
        _in_digits(count + part, &m.str(3).to_lowercase())
    };
    let half_hours = |m: &Match| {
        let word = text::strip(m.str(1)).to_lowercase();
        let count = number(&_UNIT_WORDS, &word).filter(|c| *c != 0.0).unwrap_or(1.0);
        let part = if m.str(3).to_lowercase() == "half" { 0.5 } else { 0.25 };
        _in_digits(count + part, &m.str(2).to_lowercase())
    };
    let fractions = |m: &Match| match number(&_SPOKEN_FRACTION_VALUE, &m.str(1).to_lowercase()) {
        Some(amount) => _in_digits(amount, &m.str(2).to_lowercase()),
        None => m.as_str().to_string(),
    };
    let tens_and_units = |m: &Match| {
        let tens = number(&_SPOKEN_NUMBER, &m.str(1).to_lowercase()).unwrap_or(0.0) as i64;
        let units = number(&_ONE_TO_NINE, &m.str(2).to_lowercase()).unwrap_or(0.0) as i64;
        (tens + units).to_string()
    };
    let worded = |m: &Match| {
        let word = m.str(1).to_lowercase();
        let amount = if word == "a" || word == "an" {
            if _A_RATE_UNIT.match_(m.str(2)).is_some() {
                return m.as_str().to_string();
            }
            1.0
        } else {
            number(&_SPOKEN_NUMBER, &word).unwrap_or(0.0)
        };
        if amount == 0.0 {
            return m.as_str().to_string();
        }
        _in_digits(amount, &m.str(2).to_lowercase())
    };
    let text = _A_COUNT_AND_A_HALF.sub_fn(text, count_and_a_half);
    let text = _A_SPOKEN_HALF_HOUR.sub_fn(&text, half_hours);
    let text = _A_SPOKEN_FRACTION.sub_fn(&text, fractions);
    let text = _A_TENS_AND_UNITS.sub_fn(&text, tens_and_units);
    _A_WORDED_FIGURE.sub_fn(&text, worded)
}

/// Python: bot/memory/spoken.py `spoken_as_numbers = _spoken_as_numbers`
pub fn spoken_as_numbers(text: &str) -> String {
    _spoken_as_numbers(text)
}
