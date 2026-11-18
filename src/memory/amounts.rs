//! Port of `bot/memory/amounts.py`: a count the person reports, read against the count the last
//! answer named.
//!
//! "six to eight glasses a day" / "i have about 4 glasses" -> where they stand against it. Narrow:
//! the line is a count they say they do, one figure of a thing in the table, not a length spent
//! on something else; and the answer names one figure of that same thing.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::mem7::{self, inline_re, res};
use crate::py::re::{self, Re};
use crate::py::text as pt;
use std::sync::LazyLock;

res!("amounts"; A_THEIR_OWN, A_AN_ASK, A_A_LENGTH_OF, A_CEILING);
inline_re! {
    PERIOD = r"\s*(?:a|an|per|each|every|most)\s+(day|week|night|month)\b", re::I;
}
static COUNT: LazyLock<String> = LazyLock::new(|| data::string("bot.memory.amounts._COUNT"));
static THINGS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.memory.amounts._THINGS").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].str().to_string())).collect()
});
/// `_COUNT_WORDS`, 1..=12 in order.
static COUNT_WORDS: LazyLock<Vec<(i64, String)>> = LazyLock::new(|| {
    data::get("bot.memory.amounts._COUNT_WORDS").pairs().iter().map(|(k, v)| (k.as_int().unwrap_or(0), v.str().to_string())).collect()
});
/// The single-figure and range patterns for each thing of the table.
static SINGLE: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    THINGS.iter().map(|(thing, p)| (thing.clone(), Re::new(&format!(r"({})\s*(?:-\s*)?(?:{})\b", *COUNT, p), re::I))).collect()
});
static RANGE: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    THINGS
        .iter()
        .map(|(thing, p)| (thing.clone(), Re::new(&format!("({c})\\s*(?:to|-|\u{2013}|and|or)\\s*({c})\\s*(?:-\\s*)?(?:{p})\\b", c = *COUNT), re::I)))
        .collect()
});

/// Python: bot/memory/amounts.py:73 `_a_value(word)`
///
/// The number a count says: "4", "7,000", "six". None when it is not a count.
pub fn _a_value(word: &str) -> Option<f64> {
    let word = pt::lower(pt::strip(word)).replace(',', "");
    if let Some(value) = mem7::py_float(&word) {
        return Some(value);
    }
    COUNT_WORDS.iter().position(|(_, w)| *w == word).map(|i| (i + 1) as f64)
}

/// Python: bot/memory/amounts.py:84 `_a_number(value)`
///
/// A count as the reply says it: "two", "1,000".
pub fn _a_number(value: f64) -> String {
    if value == value.trunc() {
        let int = value as i64;
        if let Some((_, word)) = COUNT_WORDS.iter().find(|(n, _)| *n == int) {
            return word.clone();
        }
        return pt::thousands(int);
    }
    pt::float_repr(value)
}

/// Python: bot/memory/amounts.py:91 `_counts(text)`
///
/// Every figure said with a unit of the table, in the order it was said, as
/// {"at", "value", "thing", "said", "period"}.
pub fn _counts(text: &str) -> Vec<Value> {
    let mut found: Vec<(usize, Value)> = Vec::new();
    for (thing, pattern) in SINGLE.iter() {
        for m in pattern.finditer(text) {
            let Some(value) = _a_value(m.str(1)) else {
                continue;
            };
            let item = mem7::dict(vec![
                ("at", Value::from(pt::char_index(text, m.start()))),
                ("value", Value::Float(value)),
                ("thing", Value::from(thing.as_str())),
                ("said", Value::from(pt::strip_chars(m.as_str(), " .,!?"))),
                ("period", Value::from(_the_period_at(text, m.end()))),
            ]);
            found.push((m.start(), item));
        }
    }
    found.sort_by_key(|(at, _)| *at);
    found.into_iter().map(|(_, item)| item).collect()
}

/// Python: bot/memory/amounts.py:107 `_the_period_at(text, at)`
///
/// The period said right after the figure ending at `at` (a byte offset), as "a day" or "a week".
pub fn _the_period_at(text: &str, at: usize) -> Option<String> {
    let m = PERIOD.match_(&text[at..])?;
    Some(if matches!(pt::lower(m.str(1)).as_str(), "week" | "month") { "a week".into() } else { "a day".into() })
}

/// Python: bot/memory/amounts.py:116 `a_count_they_report(text)`
///
/// {"at", "value", "thing", "said", "period"} for a count the person reports, or None.
pub fn a_count_they_report(text: &str) -> Option<Value> {
    if pt::strip(text).is_empty() {
        return None;
    }
    if A_AN_ASK.is_match(text) || !A_THEIR_OWN.is_match(text) || A_A_LENGTH_OF.is_match(text) {
        return None;
    }
    let mut found = _counts(text);
    // One figure only: two counts in the line is a comparison of their own or two things.
    if found.len() != 1 {
        return None;
    }
    found.pop()
}

/// Python: bot/memory/amounts.py:132 `_the_figure_named(answer, thing, mine)`
///
/// The one figure of `thing` the answer names, as {"low", "high", "said", "period"}, or None.
pub fn _the_figure_named(answer: &str, thing: &str, mine: Option<f64>) -> Option<Value> {
    let single = &SINGLE.iter().find(|(t, _)| t == thing)?.1;
    let range = &RANGE.iter().find(|(t, _)| t == thing)?.1;
    if answer.is_empty() {
        return None;
    }
    let ranges = range.finditer(answer);
    let spans: Vec<(usize, usize)> = ranges.iter().map(|m| (m.start(), m.end())).collect();
    let figure = |low: f64, high: f64, said: &str, end: usize| {
        mem7::dict(vec![
            ("low", Value::Float(low)),
            ("high", Value::Float(high)),
            ("said", Value::from(pt::strip_chars(said, " .,!?"))),
            ("period", Value::from(_the_period_at(answer, end))),
        ])
    };
    let mut figures: Vec<(f64, f64, Value)> = Vec::new();
    for m in &ranges {
        let (Some(low), Some(high)) = (_a_value(m.str(1)), _a_value(m.str(2))) else {
            continue;
        };
        if low >= high {
            continue;
        }
        figures.push((low, high, figure(low, high, m.as_str(), m.end())));
    }
    for m in single.finditer(answer) {
        if spans.iter().any(|(start, end)| *start <= m.start() && m.end() <= *end) {
            continue;
        }
        let Some(value) = _a_value(m.str(1)) else {
            continue;
        };
        figures.push((value, value, figure(value, value, m.as_str(), m.end())));
    }
    // A guideline names one figure or one range: two is a list of choices.
    let named = figures.len();
    // The answer's own figure, said back to them, is not a figure to aim at.
    let figures: Vec<Value> = figures.into_iter().filter(|(low, high, _)| !(low == high && Some(*high) == mine)).map(|(_, _, f)| f).collect();
    if named > 1 && !figures.is_empty() {
        return None;
    }
    if figures.len() != 1 {
        return None;
    }
    figures.into_iter().next()
}

/// Python: bot/memory/amounts.py:178 `_the_periods_disagree(mine, theirs)`
///
/// True when the two figures are said on periods that cannot be read against each other.
pub fn _the_periods_disagree(mine: Option<&str>, theirs: Option<&str>) -> bool {
    match (mine, theirs) {
        (Some(mine), Some(theirs)) => mine != theirs,
        _ => false,
    }
}

/// A number field of a dict as f64.
fn num(value: &Value, key: &str) -> f64 {
    mem7::get(value, key).as_f64().unwrap_or(0.0)
}

/// Python: bot/memory/amounts.py:185 `against_the_last_answer(said, answer)`
///
/// The count they reported against the figure the last answer named, or None.
pub fn against_the_last_answer(said: &Value, answer: &str) -> Option<Value> {
    if !said.is_dict() {
        return None;
    }
    let figure = _the_figure_named(answer, mem7::get_str(said, "thing"), mem7::get(said, "value").as_f64())?;
    if _the_periods_disagree(mem7::get(said, "period").as_str(), mem7::get(&figure, "period").as_str()) {
        return None;
    }
    if num(&figure, "low") <= 0.0 {
        return None;
    }
    Some(mem7::dict(vec![
        ("thing", mem7::get(said, "thing").clone()),
        ("mine", mem7::get(said, "value").clone()),
        ("mine_said", mem7::get(said, "said").clone()),
        ("low", mem7::get(&figure, "low").clone()),
        ("high", mem7::get(&figure, "high").clone()),
        ("figure", mem7::get(&figure, "said").clone()),
        ("figure_period", mem7::get(&figure, "period").clone()),
        ("a_ceiling", Value::from(A_CEILING.is_match(answer))),
    ]))
}

/// Python: bot/memory/amounts.py:202 `_the_figure_said(against)`
pub fn _the_figure_said(against: &Value) -> String {
    let mut said = format!("the {}", _a_mid_sentence(mem7::get_str(against, "figure")));
    if let Some(period) = mem7::get(against, "figure_period").as_str().filter(|p| !p.is_empty()) {
        if !said.contains(period) {
            said = format!("{said} {period}");
        }
    }
    said
}

/// Python: bot/memory/amounts.py:210 `_a_mid_sentence(figure)`
///
/// The answer's own figure, written mid-sentence: "six to eight glasses a day".
pub fn _a_mid_sentence(figure: &str) -> String {
    let word = figure.split(' ').next().unwrap_or("");
    if mem7::is_upper(mem7::first_chars(word, 1)) && COUNT_WORDS.iter().any(|(_, w)| *w == pt::lower(word)) {
        return mem7::low1(figure);
    }
    figure.to_string()
}

/// Python: bot/memory/amounts.py:229 `_the_aim_said(against)`
///
/// The answer's figure, written mid-sentence, with its unit kept and its period dropped.
pub fn _the_aim_said(against: &Value) -> String {
    let aim = format!("the {}", _a_mid_sentence(mem7::get_str(against, "figure")));
    if let Some(period) = mem7::get(against, "figure_period").as_str().filter(|p| !p.is_empty()) {
        let tail = format!(" {period}");
        if aim.ends_with(&tail) {
            return aim[..aim.len() - tail.len()].to_string();
        }
    }
    aim
}

/// Python: bot/memory/amounts.py:254 `the_comparison(against)`
///
/// Where the count they reported stands against the figure the answer named.
pub fn the_comparison(against: &Value) -> String {
    let (mine, low, high) = (num(against, "mine"), num(against, "low"), num(against, "high"));
    let aim = _the_aim_said(against);
    if mem7::get(against, "a_ceiling").truthy() {
        // A figure the answer frames as a limit is not a figure to reach.
        if mine > high {
            return format!("That is over {aim}, which is worth trimming back.");
        }
        if mine < low {
            return format!("That is under {aim}, which is where you want to be.");
        }
        return format!("That is inside {aim}, which is fine - that is a ceiling, not a target.");
    }
    if mine < low {
        return format!("That is a fair way under {aim} most people need.");
    }
    if mine > high {
        return format!("That is past {aim}, which is fine - more than that is no problem.");
    }
    format!("That is inside {aim} most people need.")
}
