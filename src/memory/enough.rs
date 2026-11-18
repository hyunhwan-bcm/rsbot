//! Port of `bot/memory/enough.py`: whether the answer already on the table is the amount the
//! person asked about ("is that enough", "how much more do i need", "is 300 too much"). The
//! figure in the answer and the figure the person said are put side by side, units matched.
//! `enough_reply` is the whole surface. Module data comes from the exported bundle.
//!
//! Figures are Python floats (`f64`). Positions that PyBot does arithmetic on across strings are
//! read as code points; positions only used to slice the string they came from stay bytes.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::spoken::_spoken_as_numbers;
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text as pt;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

const I: u32 = pyre::I;

fn g(name: &str) -> &'static Value {
    data::get(&format!("bot.memory.enough.{name}"))
}
fn re(name: &str) -> &'static Re {
    g(name).as_re().unwrap_or_else(|| panic!("bot.memory.enough.{name} is not a regex"))
}

static ENOUGH_ASK: LazyLock<&'static Re> = LazyLock::new(|| re("ENOUGH_ASK"));
static HOW_MORE: LazyLock<&'static Re> = LazyLock::new(|| re("HOW_MORE"));
static _A_DOCTOR: LazyLock<&'static Re> = LazyLock::new(|| re("_A_DOCTOR"));
static _A_LIMIT: LazyLock<&'static Re> = LazyLock::new(|| re("_A_LIMIT"));
static _A_TEMPERATURE: LazyLock<&'static Re> = LazyLock::new(|| re("_A_TEMPERATURE"));
static _HOURS_AND_MINUTES: LazyLock<&'static Re> = LazyLock::new(|| re("_HOURS_AND_MINUTES"));
static _A_PERIOD: LazyLock<&'static Re> = LazyLock::new(|| re("_A_PERIOD"));
static _AIM_CUE: LazyLock<&'static Re> = LazyLock::new(|| re("_AIM_CUE"));
static _A_WORD_AFTER: LazyLock<&'static Re> = LazyLock::new(|| re("_A_WORD_AFTER"));
static _TOO_FAR_ABOVE: LazyLock<f64> = LazyLock::new(|| g("_TOO_FAR_ABOVE").as_f64().unwrap_or(3.5));
static _TOO_FAR_BELOW: LazyLock<f64> = LazyLock::new(|| g("_TOO_FAR_BELOW").as_f64().unwrap_or(20.0));
static _PERIOD_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| g("_PERIOD_WORDS").str_set());

// `bot.memory.store_lines`.
fn lines(name: &str) -> &'static Value {
    data::get(&format!("bot.memory.store_lines.{name}"))
}
/// `_AMOUNTS`: (family, pattern) in order.
static _AMOUNTS: LazyLock<Vec<(String, String)>> =
    LazyLock::new(|| lines("_AMOUNTS").items().iter().map(|pair| (pair.items()[0].str().to_string(), pair.items()[1].str().to_string())).collect());
static _AMOUNT_PATTERNS: LazyLock<Vec<(String, Re)>> =
    LazyLock::new(|| _AMOUNTS.iter().map(|(family, pattern)| (family.clone(), Re::new(pattern, I))).collect());
static _AMOUNT_WORDS: LazyLock<HashMap<String, f64>> = LazyLock::new(|| {
    lines("_AMOUNT_WORDS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_f64().unwrap_or(0.0))).collect()
});
static _COUNT_WORD: LazyLock<String> = LazyLock::new(|| lines("_COUNT_WORD").str().to_string());
/// `_CONVERT`: ((mine, theirs), factor) in order.
static _CONVERT: LazyLock<Vec<((String, String), f64)>> = LazyLock::new(|| {
    lines("_CONVERT")
        .pairs()
        .iter()
        .map(|(k, v)| ((k.items()[0].str().to_string(), k.items()[1].str().to_string()), v.as_f64().unwrap_or(0.0)))
        .collect()
});
static _UNIT_SAID: LazyLock<HashMap<String, (String, String)>> = LazyLock::new(|| {
    lines("_UNIT_SAID")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), (v.items()[0].str().to_string(), v.items()[1].str().to_string())))
        .collect()
});
static _WEEKLY: LazyLock<String> = LazyLock::new(|| lines("_WEEKLY").str().to_string());
static NUMBER_WORDS: LazyLock<HashMap<i64, String>> =
    LazyLock::new(|| lines("NUMBER_WORDS").pairs().iter().map(|(k, v)| (k.as_int().unwrap_or(0), v.str().to_string())).collect());

/// `"|".join(p for _, p in _AMOUNTS)`.
static UNITS: LazyLock<String> = LazyLock::new(|| _AMOUNTS.iter().map(|(_, p)| p.as_str()).collect::<Vec<_>>().join("|"));

// The patterns PyBot builds inline from the store's words.
static A_FIGURE_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d+(?:,\d{3})*(?:\.\d+)?", 0));
static AN_AMOUNT: LazyLock<Re> =
    LazyLock::new(|| Re::new(&format!(r"({})\s*(?:x\s*)?({})", *_COUNT_WORD, *UNITS), I));
static A_FIGURE_WITH_ITS_UNIT: LazyLock<Re> =
    LazyLock::new(|| Re::new(&format!(r"({})\s*(?:-\s*)?({})", *_COUNT_WORD, *UNITS), I));
static A_FIGURE_WITH_A_UNIT: LazyLock<Re> =
    LazyLock::new(|| Re::new(&format!(r"({})\s*(?:-\s*)?(?:{})", *_COUNT_WORD, *UNITS), I));
static A_RANGE_WITH_ITS_UNIT: LazyLock<Re> = LazyLock::new(|| {
    Re::new(&format!(r"({c})\s*(?:to|-|–|and)\s*({c})\s*(?:-\s*)?({u})", c = *_COUNT_WORD, u = *UNITS), I)
});
static A_RANGE_IN_TWO_UNITS: LazyLock<Re> = LazyLock::new(|| {
    Re::new(&format!(r"({c})\s*({u})\s*(?:to|-|–|and)\s*({c})\s*({u})", c = *_COUNT_WORD, u = *UNITS), I)
});
static A_RANGE_MAYBE_WITH_A_UNIT: LazyLock<Re> = LazyLock::new(|| {
    Re::new(&format!(r"({c})\s*(?:to|and|or|–|-)\s*({c})\s*(?:-\s*)?({u})?", c = *_COUNT_WORD, u = *UNITS), I)
});
static A_BARE_RANGE: LazyLock<Re> =
    LazyLock::new(|| Re::new(&format!(r"({c})\s*(?:to|and|or|–|-)\s*({c})\b", c = *_COUNT_WORD), I));
static A_RANGE_WITH_A_UNIT: LazyLock<Re> = LazyLock::new(|| {
    Re::new(&format!(r"({c})\s*(?:to|and|or|–|-)\s*({c})\s*(?:{u})\b", c = *_COUNT_WORD, u = *UNITS), I)
});
static A_COUNT: LazyLock<Re> = LazyLock::new(|| Re::new(&format!(r"({})", *_COUNT_WORD), I));
static A_UNIT_FOLLOWS: LazyLock<Re> = LazyLock::new(|| Re::new(&format!(r"\s*({})\b", *UNITS), I));
static AN_ANSWER_PERIOD: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?:a|per|each|every)\s+(day|week|night)\b", I));
static AN_ANSWER_PERIODS: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?:a|an|per|each|every)\s+(day|week|night|month|year)\b", I));
static A_RATE: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        r"\s*(?:%|per cent\b|percent\b|per (?:kilo|kg|kilogram|kilo of|pound|lb|100|head|person|day per)\b|g\s?/\s?kg\b|mg\s?/\s?kg\b)",
        I,
    )
});
static TOO_MUCH: LazyLock<Re> = LazyLock::new(|| Re::new(r"\btoo (?:much|often|many)\b", I));
static A_TIME_UNIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:minutes?|mins?|hours?|hrs?)\b", I));

/// A figure the answer names: (low, high, as it was said).
pub type Figure = (f64, f64, String);
/// A figure in a line: (value, family, as it was said).
pub type Amount = (f64, String, String);

/// `rx.match(text, pos)`: a match that starts at byte `pos` (the text before still counts).
fn match_at<'t>(rx: &'t Re, text: &'t str, pos: usize) -> Option<Match<'t>> {
    rx.search_at(text, pos).filter(|m| m.start() == pos)
}

/// The value of one Unicode decimal digit (`float()` reads every Nd digit).
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

/// `float(s)` for what `\d+(?:\.\d+)?` matched (any Unicode digits).
fn float_of(s: &str) -> f64 {
    let ascii: String = s.chars().map(|c| if c == '.' { '.' } else { char::from_digit(decimal_value(c).unwrap_or(0), 10).unwrap() }).collect();
    ascii.parse().unwrap_or(0.0)
}

/// `round(x, n)`: the double nearest the correctly rounded decimal.
fn round_to(x: f64, n: usize) -> f64 {
    format!("{:.*}", n, x).parse().unwrap_or(x)
}

/// `s[0].upper() + s[1:]` (an empty string is an IndexError).
fn up0(s: &str) -> String {
    if s.is_empty() {
        panic!("IndexError: string index out of range");
    }
    pt::upper_first(s)
}

/// `s[0].lower() + s[1:]` (an empty string is an IndexError).
fn low0(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => panic!("IndexError: string index out of range"),
        Some(f) => f.to_lowercase().chain(c).collect(),
    }
}

/// `s.strip(" .,!?")`.
fn trimmed(s: &str) -> String {
    pt::strip_chars(s, " .,!?").to_string()
}

/// Whether a match lies inside one of the spans.
fn inside(m: &Match, spans: &[(usize, usize)]) -> bool {
    spans.iter().any(|(start, end)| *start <= m.start() && m.end() <= *end)
}

/// `len({figure[:2] for figure in figures})`.
fn distinct_pairs<'a>(figures: impl Iterator<Item = &'a Figure>) -> usize {
    let mut seen: Vec<(f64, f64)> = Vec::new();
    for figure in figures {
        if !seen.iter().any(|pair| pair.0 == figure.0 && pair.1 == figure.1) {
            seen.push((figure.0, figure.1));
        }
    }
    seen.len()
}

/// Python: bot/memory/enough.py:62 `_a_value(word)`
///
/// The number a word stands for, from a digit or a word, or 0 when it is neither.
pub fn _a_value(word: &str) -> f64 {
    let word = pt::lower(pt::strip(word));
    if A_FIGURE_WORD.fullmatch(&word).is_some() {
        return float_of(&word.replace(',', ""));
    }
    _AMOUNT_WORDS.get(&word).copied().unwrap_or(0.0)
}

/// Python: bot/memory/enough.py:70 `_amount_family(unit)`
///
/// The family a unit word belongs to, or "" when it is not one we can compare.
pub fn _amount_family(unit: &str) -> String {
    let unit = pt::strip(unit);
    for (family, pattern) in _AMOUNT_PATTERNS.iter() {
        if pattern.fullmatch(unit).is_some() {
            return family.clone();
        }
    }
    String::new()
}

/// The period word as the store says it: "a week" for a week, "a day" for anything else.
fn as_a_period(word: &str) -> String {
    if pt::lower(word) == "week" { _WEEKLY.clone() } else { "a day".to_string() }
}

/// Python: bot/memory/enough.py:92 `_period_after(text, at)`
///
/// The period said right after the figure ending at byte `at`, as 'a day' or 'a week'.
pub fn _period_after(text: &str, at: usize) -> Option<String> {
    match_at(&_A_PERIOD, text, at).map(|m| as_a_period(m.str(1)))
}

/// Python: bot/memory/enough.py:100 `_a_period_near(text, phrase)`
///
/// The period said after `phrase` in `text`, as 'a day' or 'a week'.
pub fn _a_period_near(text: &str, phrase: &str) -> Option<String> {
    // `text.lower().find(phrase.lower())` is a code-point index into the lowered text, used as
    // one into the text itself.
    let at = pt::find(&pt::lower(text), &pt::lower(phrase));
    if at < 0 || phrase.is_empty() {
        return None;
    }
    let end = at as usize + pt::len(phrase);
    if end > pt::len(text) {
        return None; // `pattern.match(text, pos)` past the end finds nothing to match there
    }
    _period_after(text, pt::byte_at(text, end))
}

/// Python: bot/memory/enough.py:108 `_the_period_of_the_answer(answer)`
///
/// The period an answer is written on, when it says one: 'a day' or 'a week'.
pub fn _the_period_of_the_answer(answer: &str) -> Option<String> {
    AN_ANSWER_PERIOD.search(answer).map(|m| as_a_period(m.str(1)))
}

/// Python: bot/memory/enough.py:115 `_the_periods_the_answer_says(answer)`
///
/// Every period an answer is written on: {'a day', 'a week'} for a guideline that says both.
pub fn _the_periods_the_answer_says(answer: &str) -> HashSet<String> {
    AN_ANSWER_PERIODS
        .finditer(answer)
        .iter()
        .map(|m| {
            let word = pt::lower(m.str(1));
            match word.as_str() {
                "week" => _WEEKLY.clone(),
                "day" | "night" => "a day".to_string(),
                _ => format!("a {word}"),
            }
        })
        .collect()
}

/// Python: bot/memory/enough.py:131 `_periods_disagree(mine, theirs, also=())`
///
/// True when the two figures are said on periods that cannot be read against each other.
pub fn _periods_disagree(mine: Option<&str>, theirs: Option<&str>, also: &HashSet<String>) -> bool {
    let Some(mine) = mine else {
        return false;
    };
    // A guideline often gives the same aim in two periods.
    if also.contains(mine) {
        return false;
    }
    match theirs {
        None => mine == _WEEKLY.as_str(),
        Some(theirs) => mine != theirs,
    }
}

/// Python: bot/memory/enough.py:157 `_too_far_to_compare(value, low, high)`
///
/// True when the answer's figure and theirs are the same unit but not the same quantity.
pub fn _too_far_to_compare(value: f64, low: f64, high: f64) -> bool {
    if value == 0.0 || low == 0.0 || high == 0.0 {
        return false;
    }
    if value > high && value > *_TOO_FAR_ABOVE * high {
        return true;
    }
    value < low && low > *_TOO_FAR_BELOW * value
}

/// Python: bot/memory/enough.py:168 `_amounts(text)`
///
/// Every figure with a unit in the line: (value, family, as it was said).
pub fn _amounts(text: &str) -> Vec<Amount> {
    let text = _spoken_as_numbers(text);
    let mut found: Vec<Amount> = Vec::new();
    // "1 hour 40 minutes" is one figure of a hundred minutes.
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for m in _HOURS_AND_MINUTES.finditer(&text) {
        let (hours, minutes) = (_a_value(m.str(1)), _a_value(m.str(2)));
        if hours == 0.0 && minutes == 0.0 {
            continue;
        }
        found.push((hours * 60.0 + minutes, "minutes".to_string(), trimmed(m.as_str())));
        spans.push((m.start(), m.end()));
    }
    for m in AN_AMOUNT.finditer(&text) {
        let value = _a_value(m.str(1));
        if value == 0.0 {
            continue;
        }
        if inside(&m, &spans) {
            continue;
        }
        found.push((value, _amount_family(m.str(2)), trimmed(m.as_str())));
    }
    // A frequency is tried first (a stable sort, as Python's).
    found.sort_by_key(|item| if item.1 == "a week" || item.1 == "a day" { 0 } else { 1 });
    found
}

/// Python: bot/memory/enough.py:198 `_target(answer, family, mine=None, bare_ok=False, period=None)`
///
/// The figure the last answer named in the same family: (low, high, as it was said), or None.
pub fn _target(answer: &str, family: &str, mine: Option<f64>, bare_ok: bool, period: Option<&str>) -> Option<Figure> {
    let answer = _spoken_as_numbers(answer);
    let answer = answer.as_str();
    let mut ranges: Vec<(usize, Figure)> = Vec::new();
    let mut singles: Vec<(usize, Figure)> = Vec::new();
    // A figure said as hours and minutes is read whole.
    let long_spans: Vec<(usize, usize)> = _HOURS_AND_MINUTES.finditer(answer).iter().map(|m| (m.start(), m.end())).collect();
    // A range first: "1.5 to 2 litres", "six to eight glasses", "seven to nine hours".
    for m in A_RANGE_WITH_ITS_UNIT.finditer(answer) {
        if _amount_family(m.str(3)) != family {
            continue;
        }
        let (low, high) = (_a_value(m.str(1)), _a_value(m.str(2)));
        if low != 0.0 && high != 0.0 && !_a_rate(answer, m.end()) {
            ranges.push((m.start(), (low, high, trimmed(m.as_str()))));
        }
    }
    let mut spans: Vec<(usize, usize)> = A_RANGE_WITH_ITS_UNIT.finditer(answer).iter().map(|m| (m.start(), m.end())).collect();
    // A range written in two units is one range: "30 minutes to an hour a day" is 30 to 60.
    for m in A_RANGE_IN_TWO_UNITS.finditer(answer) {
        let (low_family, high_family) = (_amount_family(m.str(2)), _amount_family(m.str(4)));
        if low_family == high_family {
            continue;
        }
        let (low, high) = (_a_value(m.str(1)), _a_value(m.str(3)));
        if low == 0.0 || high == 0.0 {
            continue;
        }
        if family != low_family && family != high_family {
            continue;
        }
        let factor = |unit_family: &str| -> f64 {
            match _CONVERT.iter().find(|((a, b), _)| a == family && b == unit_family) {
                Some((_, factor)) => *factor,
                None => {
                    if unit_family == family {
                        1.0
                    } else {
                        0.0
                    }
                }
            }
        };
        let (low_in, high_in) = (low * factor(&low_family), high * factor(&high_family));
        if low_in == 0.0 || high_in == 0.0 {
            continue;
        }
        let unit = _UNIT_SAID.get(family).map(|u| u.1.clone()).unwrap_or_else(|| family.to_string());
        let said = format!("{} to {} {unit}", _the_number(low_in), _the_number(high_in));
        ranges.push((m.start(), (low_in, high_in, said)));
        spans.push((m.start(), m.end()));
    }
    // Then a plain figure: "30 minutes most days", "three sessions a week".
    let all_spans: Vec<(usize, usize)> = spans.iter().chain(long_spans.iter()).copied().collect();
    for m in A_FIGURE_WITH_ITS_UNIT.finditer(answer) {
        if _amount_family(m.str(2)) != family {
            continue;
        }
        // "two ten-minute walks" are shapes of thing, not figures to reach.
        if m.as_str().contains('-') {
            continue;
        }
        if inside(&m, &all_spans) {
            continue;
        }
        let value = _a_value(m.str(1));
        if value != 0.0 && !_a_rate(answer, m.end()) {
            singles.push((m.start(), (value, value, trimmed(m.as_str()))));
        }
    }
    // "1 hour 30 minutes" is one figure of ninety minutes.
    for m in _HOURS_AND_MINUTES.finditer(answer) {
        let (hours, minutes) = (_a_value(m.str(1)), _a_value(m.str(2)));
        if hours != 0.0 || minutes != 0.0 {
            let total = hours * 60.0 + minutes;
            singles.push((m.start(), (total, total, trimmed(m.as_str()))));
        }
    }
    ranges.sort_by_key(|item| item.0);
    singles.sort_by_key(|item| item.0);
    let is_their_number = |figure: &Figure| mine.is_some_and(|mine| figure.0 == figure.1 && figure.1 == mine);
    // A single figure that is the person's own number said back is the answer measuring them.
    let mut useful: Vec<(usize, Figure)> =
        singles.into_iter().filter(|(_, figure)| !(!ranges.is_empty() && is_their_number(figure))).collect();
    let mut figures: Vec<Figure> = ranges.iter().map(|(_, f)| f.clone()).chain(useful.iter().map(|(_, f)| f.clone())).collect();
    if let Some(period) = period {
        if distinct_pairs(figures.iter()) > 1 {
            // Only two figures said on two different periods are a thing the period settles.
            let ranges_on = _the_periods_of(answer, &ranges);
            let useful_on = _the_periods_of(answer, &useful);
            let said_on: Vec<&Option<String>> = ranges_on.iter().chain(useful_on.iter()).collect();
            if said_on.iter().any(|each| each.as_deref() == Some(period))
                && said_on.iter().any(|each| each.as_deref().is_some_and(|each| !each.is_empty() && each != period))
            {
                ranges = ranges.into_iter().zip(ranges_on).filter(|(_, each)| each.as_deref() == Some(period)).map(|(pair, _)| pair).collect();
                useful = useful.into_iter().zip(useful_on).filter(|(_, each)| each.as_deref() == Some(period)).map(|(pair, _)| pair).collect();
                figures = ranges.iter().map(|(_, f)| f.clone()).chain(useful.iter().map(|(_, f)| f.clone())).collect();
            }
        }
    }
    if distinct_pairs(figures.iter()) > 1 {
        // One range is one aim even when the answer also says a single number in passing.
        if distinct_pairs(ranges.iter().map(|(_, f)| f)) == 1 {
            let best = ranges[0].1.clone();
            if !is_their_number(&best) {
                return _with_its_period(answer, Some(best), period);
            }
        }
        return None;
    }
    let best = ranges.first().map(|(_, f)| f.clone()).or_else(|| useful.first().map(|(_, f)| f.clone()));
    if let Some(best) = &best {
        if !is_their_number(best) {
            return _with_its_period(answer, Some(best.clone()), period);
        }
    }
    // An answer can name its range without repeating the unit ("most adults need seven to
    // nine"), read only for the person's own kind of amount.
    let bare = if bare_ok { _near_bare_range(answer, mine, Some(family)) } else { None };
    if bare.is_some() {
        return bare;
    }
    match best {
        Some(best) => _with_its_period(answer, Some(best), period),
        None => None,
    }
}

/// Python: bot/memory/enough.py:337 `_the_aim_that_is_their_number(answer, family, mine)`
///
/// The figure the answer aims at when it is the number they gave: (low, high, said) or None.
pub fn _the_aim_that_is_their_number(answer: &str, family: &str, mine: f64) -> Option<Figure> {
    if mine == 0.0 {
        return None;
    }
    for m in A_FIGURE_WITH_ITS_UNIT.finditer(answer) {
        if _amount_family(m.str(2)) != family {
            continue;
        }
        if _a_value(m.str(1)) != mine {
            continue;
        }
        // `answer[max(0, start - 40):start]`, forty code points back.
        let start = pt::char_index(answer, m.start());
        let before = pt::slice(answer, start.saturating_sub(40) as isize, Some(start as isize));
        if !_AIM_CUE.is_match(&before) {
            continue;
        }
        let mut said = trimmed(m.as_str());
        // The period is part of the figure.
        if let Some(period) = _period_after(answer, m.end()) {
            said += if period == *_WEEKLY { " a week" } else { " a day" };
        }
        return Some((mine, mine, said));
    }
    None
}

/// Python: bot/memory/enough.py:367 `_with_its_period(answer, figure, period)`
///
/// The figure with the period it is said on, when the answer says it on the person's own.
pub fn _with_its_period(answer: &str, figure: Option<Figure>, period: Option<&str>) -> Option<Figure> {
    let (Some(period), Some(fig)) = (period.filter(|p| !p.is_empty()), figure.as_ref()) else {
        return figure;
    };
    let Some(at) = answer.find(fig.2.as_str()) else {
        return figure;
    };
    let Some(m) = match_at(&_A_PERIOD, answer, at + fig.2.len()) else {
        return figure;
    };
    if as_a_period(m.str(1)) != period {
        return figure;
    }
    Some((fig.0, fig.1, format!("{} {}", fig.2, trimmed(m.as_str()))))
}

/// Python: bot/memory/enough.py:388 `_the_periods_of(answer, figures)`
///
/// The period each figure (start, (low, high, said)) is said on, or None.
pub fn _the_periods_of(answer: &str, figures: &[(usize, Figure)]) -> Vec<Option<String>> {
    figures
        .iter()
        .map(|(at, figure)| {
            let m = _A_PERIOD.search_at(answer, *at)?;
            // `match.start() > at + len(figure[2]) + 12`, in code points.
            if pt::char_index(answer, m.start()) > pt::char_index(answer, *at) + pt::len(&figure.2) + 12 {
                return None;
            }
            Some(as_a_period(m.str(1)))
        })
        .collect()
}

/// Python: bot/memory/enough.py:404 `_aims_named(answer, mine, family)`
///
/// The distinct figures of this kind the answer names, a range counting as one figure.
pub fn _aims_named(answer: &str, mine: f64, family: &str) -> Vec<(f64, f64)> {
    let mut found: Vec<(f64, f64)> = Vec::new();
    let add = |pair: (f64, f64), found: &mut Vec<(f64, f64)>| {
        if !found.contains(&pair) {
            found.push(pair);
        }
    };
    let mut spans: Vec<(usize, usize)> = Vec::new();
    // A range, with its unit or without.
    for m in A_RANGE_MAYBE_WITH_A_UNIT.finditer(answer) {
        let (low, high) = (_a_value(m.str(1)), _a_value(m.str(2)));
        if !(low != 0.0 && high != 0.0 && low < high) {
            continue;
        }
        spans.push((m.start(), m.end()));
        if let Some(said) = m.get(3).filter(|said| !said.is_empty()) {
            if _amount_family(said) != family {
                continue;
            }
        }
        add((low, high), &mut found);
    }
    // Every figure said with a unit of any kind.
    for m in A_FIGURE_WITH_ITS_UNIT.finditer(answer) {
        if inside(&m, &spans) {
            continue;
        }
        spans.push((m.start(), m.end()));
        if _amount_family(m.str(2)) != family {
            continue;
        }
        let value = _a_value(m.str(1));
        if value != 0.0 {
            add((value, value), &mut found);
        }
    }
    // A figure on its own, with no unit beside it at all.
    for m in A_COUNT.finditer(answer) {
        if inside(&m, &spans) {
            continue;
        }
        let value = _a_value(m.str(1));
        if value == 0.0 || !(mine != 0.0 && mine / 3.0 <= value && value <= mine * 3.0) {
            continue;
        }
        if _counts_things(answer, m.end()) {
            continue;
        }
        add((value, value), &mut found);
    }
    found
}

/// Python: bot/memory/enough.py:463 `_counts_things(answer, at)`
///
/// True when the number ending at `at` counts things rather than measuring an amount.
///
/// PyBot asks `_amount_family(word) is not None` of the first word after the number, and
/// `_amount_family` returns "" rather than None for a word that is no unit, so that test is
/// always true and the reading always stops there: False. The words are still read in the
/// same order.
pub fn _counts_things(answer: &str, at: usize) -> bool {
    let Some(m) = match_at(&_A_WORD_AFTER, answer, at) else {
        return false;
    };
    let words = pt::split(m.as_str());
    for (at_word, word) in words.iter().enumerate() {
        let family: Option<String> = Some(_amount_family(word));
        if family.is_some() || _PERIOD_WORDS.contains(word) {
            return false;
        }
        if pt::len(word) > 3 && word.ends_with('s') {
            let rest: Vec<&str> = words[at_word + 1..].iter().map(String::as_str).chain(std::iter::once("")).collect();
            return !rest.iter().any(|other| _PERIOD_WORDS.contains(*other) || _amount_family(other) == "a week");
        }
    }
    false
}

/// Python: bot/memory/enough.py:487 `_figures_in(answer)`
///
/// Every number in the answer, with or without a unit: "20 minutes", "seven to nine".
pub fn _figures_in(answer: &str) -> Vec<f64> {
    let mut found: Vec<f64> = _amounts(answer).into_iter().map(|(value, _, _)| value).collect();
    for m in A_BARE_RANGE.finditer(answer) {
        let (low, high) = (_a_value(m.str(1)), _a_value(m.str(2)));
        if low != 0.0 && high != 0.0 && low < high {
            found.push(low);
            found.push(high);
        }
    }
    found
}

/// Python: bot/memory/enough.py:502 `_a_rate(answer, end)`
///
/// The figure is a rate rather than an amount: "0.75g per kilo", "2% of it". (`end` is a byte
/// offset; the 24 characters after it are read.)
pub fn _a_rate(answer: &str, end: usize) -> bool {
    let after: String = answer[end..].chars().take(24).collect();
    A_RATE.match_(&after).is_some()
}

/// Python: bot/memory/enough.py:514 `_another_kind_follows(answer, end, family)`
///
/// A unit of another kind sits right after the figure, so the figure is not this kind's.
pub fn _another_kind_follows(answer: &str, end: usize, family: &str) -> bool {
    match A_UNIT_FOLLOWS.match_(&answer[end..]) {
        Some(following) => _amount_family(following.str(1)) != family,
        None => false,
    }
}

/// Python: bot/memory/enough.py:524 `_near_bare_range(answer, mine, family=None)`
///
/// A range of numbers said without a unit, near enough their own figure to be about it.
pub fn _near_bare_range(answer: &str, mine: Option<f64>, family: Option<&str>) -> Option<Figure> {
    for m in A_BARE_RANGE.finditer(answer) {
        let (low, high) = (_a_value(m.str(1)), _a_value(m.str(2)));
        if !(low != 0.0 && high != 0.0 && low < high) {
            continue;
        }
        if let Some(mine) = mine {
            if !(low <= mine * 3.0 && mine <= high * 3.0) {
                continue;
            }
        }
        if let Some(family) = family.filter(|f| !f.is_empty()) {
            if _another_kind_follows(answer, m.end(), family) {
                continue;
            }
        }
        if _a_rate(answer, m.end()) {
            continue;
        }
        return Some((low, high, trimmed(m.as_str())));
    }
    None
}

/// Python: bot/memory/enough.py:546 `_with_its_unit(aim, family, high, answer='')`
///
/// The aim with its unit said, when the answer named the range without one.
pub fn _with_its_unit(aim: &str, family: &str, high: f64, answer: &str) -> String {
    // `dict(_AMOUNTS).get(family)`: the last pattern given for the family.
    let Some(pattern) = _AMOUNTS.iter().rev().find(|(f, _)| f == family).map(|(_, p)| p) else {
        return aim.to_string();
    };
    if Re::new(pattern, I).is_match(aim) {
        return aim.to_string();
    }
    // A figure that already says a length of time is a length of time.
    if (family == "minutes" || family == "hours") && A_TIME_UNIT.is_match(aim) {
        return aim.to_string();
    }
    if let Some(at) = answer.find(aim) {
        if _another_kind_follows(answer, at + aim.len(), family) {
            return aim.to_string();
        }
    }
    let Some(units) = _UNIT_SAID.get(family) else {
        return aim.to_string();
    };
    format!("{aim} {}", if high == 1.0 { &units.0 } else { &units.1 })
}

/// Python: bot/memory/enough.py:573 `_the_number(value)`
pub fn _the_number(value: f64) -> String {
    if value.fract() == 0.0 && value.is_finite() {
        format!("{:.0}", value)
    } else {
        pt::float_repr(value)
    }
}

/// Answers from another file's function, whichever Rust type its contract has today.
trait PyReply {
    fn reply(self) -> Option<String>;
}
impl PyReply for () {
    fn reply(self) -> Option<String> {
        None
    }
}
impl PyReply for Option<String> {
    fn reply(self) -> Option<String> {
        self.filter(|s| !s.is_empty())
    }
}
impl PyReply for String {
    fn reply(self) -> Option<String> {
        Some(self).filter(|s| !s.is_empty())
    }
}
impl PyReply for Value {
    fn reply(self) -> Option<String> {
        if self.truthy() { Some(crate::memory::facts::py_str(&self)) } else { None }
    }
}

/// The turns as the Python list of dicts.
fn turns_value(turns: &[Turn]) -> Value {
    Value::List(Arc::new(
        turns
            .iter()
            .map(|t| {
                Value::Dict(Arc::new(vec![
                    (Value::from("user"), Value::from(t.user.as_str())),
                    (Value::from("bot"), Value::from(t.bot.as_str())),
                    (Value::from("module"), t.module.as_deref().map(Value::from).unwrap_or(Value::None)),
                ]))
            })
            .collect(),
    ))
}

/// Python: bot/memory/enough.py:578 `enough_reply(text, memory)`
///
/// Where the amount they gave stands against the figure the last answer named, or None.
pub fn enough_reply(text: &str, memory: &Memory) -> Option<String> {
    let more = HOW_MORE.is_match(text);
    if !more && !ENOUGH_ASK.is_match(text) {
        return None;
    }
    let turns = &memory.turns;
    let last = turns.last()?;
    if last.bot.is_empty() {
        return None;
    }
    let too_much = TOO_MUCH.is_match(text);
    // Money is read on its own, before the amounts (bot/memory/purse.py).
    if let Some(purse_reply) = crate::memory::purse::against_the_bands(text, &turns_value(turns)).reply() {
        return Some(purse_reply);
    }
    // The figure they are asking about: the ask's own, or the last line they typed that gave one.
    let mut said = String::new();
    let mut given: Vec<Amount> = Vec::new();
    let candidates = std::iter::once(text.to_string()).chain(turns.iter().rev().map(|t| t.user.clone()));
    for line_text in candidates {
        let found: Vec<Amount> = _amounts(&line_text).into_iter().filter(|item| !item.1.is_empty()).collect();
        if !found.is_empty() {
            given = found;
            said = _spoken_as_numbers(&line_text);
            break;
        }
    }
    if given.is_empty() {
        // A figure said without a unit is still the amount they are asking about when the
        // answer names a range in the same breath.
        return _enough_bare(turns, text, more, too_much);
    }
    let mine_values: Vec<f64> = given.iter().map(|(value, _, _)| *value).collect();
    _enough_against(&_candidate_answers(turns, &mine_values), &given, &mine_values, &said, more, too_much, false)
}

/// Python: bot/memory/enough.py:629 `_enough_bare(turns, text, more, too_much)`
///
/// The reading for a figure the person gave without a unit, or None.
pub fn _enough_bare(turns: &[Turn], text: &str, more: bool, too_much: bool) -> Option<String> {
    let _ = too_much;
    let mut mine = 0.0;
    let candidates = std::iter::once(text.to_string()).chain(turns.iter().rev().map(|t| t.user.clone()));
    for line_text in candidates {
        if let Some(first) = _bare_figures(&line_text).into_iter().find(|value| *value >= 100.0) {
            mine = first;
            break;
        }
    }
    if mine == 0.0 {
        return None;
    }
    for answer in _candidate_answers(turns, &[mine]) {
        let Some((low, high, said)) = _range_near(&answer, mine) else {
            continue;
        };
        let aim = low0(&said);
        let mut spoken = _the_number(mine);
        if spoken == "1" {
            spoken = "one".to_string();
        }
        if more {
            let gap = low - mine;
            if gap <= 0.0 {
                return Some(format!("None - {spoken} is already at the {aim} that answer named."));
            }
            return Some(format!("{} more would put you at the {aim} that answer named.", _the_number(round_to(gap, 1))));
        }
        if mine < low {
            return Some(format!("{spoken} is short of the {aim} that answer named."));
        }
        if mine > high {
            return Some(format!("{spoken} is past the {aim} that answer named, so that side is worth watching."));
        }
        return Some(format!("No - {spoken} is inside the {aim} that answer named, so you are fine."));
    }
    None
}

/// Python: bot/memory/enough.py:667 `_range_near(answer, mine)`
///
/// The range in the answer near enough their number to be about it, its unit included.
pub fn _range_near(answer: &str, mine: f64) -> Option<Figure> {
    for pattern in [&*A_RANGE_WITH_A_UNIT, &*A_BARE_RANGE] {
        for m in pattern.finditer(answer) {
            let (low, high) = (_a_value(m.str(1)), _a_value(m.str(2)));
            if !(low != 0.0 && high != 0.0 && low < high) {
                continue;
            }
            if !(low <= mine * 3.0 && mine <= high * 3.0) {
                continue;
            }
            return Some((low, high, trimmed(m.as_str())));
        }
    }
    None
}

/// Python: bot/memory/enough.py:689 `_bare_figures(text)`
///
/// Every number in the line that is not part of an amount with a unit.
pub fn _bare_figures(text: &str) -> Vec<f64> {
    let with_unit: Vec<(usize, usize)> = A_FIGURE_WITH_A_UNIT.finditer(text).iter().map(|m| (m.start(), m.end())).collect();
    A_COUNT
        .finditer(text)
        .iter()
        .filter(|m| !inside(m, &with_unit))
        .map(|m| _a_value(m.str(1)))
        .filter(|value| *value != 0.0)
        .collect()
}

/// Python: bot/memory/enough.py:704 `_candidate_answers(turns, mine_values)`
///
/// The bot answers to read a figure out of, newest first, up to three turns back, stepping over
/// an answer whose only figures are the person's own.
pub fn _candidate_answers(turns: &[Turn], mine_values: &[f64]) -> Vec<String> {
    let mut out = Vec::new();
    for turn in turns[turns.len().saturating_sub(3)..].iter().rev() {
        let answer = &turn.bot;
        if answer.is_empty() {
            continue;
        }
        if !mine_values.is_empty() && !_figures_in(answer).iter().any(|figure| !mine_values.contains(figure)) {
            continue;
        }
        out.push(answer.clone());
    }
    out
}

/// Python: bot/memory/enough.py:724 `_enough_against(answers, given, mine_values, said, more, too_much, quoted_ok=False)`
///
/// The reply for the best of `answers`, or None when none of them names a figure to aim at.
pub fn _enough_against(
    answers: &[String],
    given: &[Amount],
    mine_values: &[f64],
    said: &str,
    more: bool,
    too_much: bool,
    quoted_ok: bool,
) -> Option<String> {
    let mut quoted: Option<(String, Vec<Amount>)> = None;
    for answer in answers {
        for (value, family, spoken) in given {
            let value = *value;
            if family.is_empty() {
                continue;
            }
            let period = _a_period_near(said, spoken);
            if _periods_disagree(period.as_deref(), _the_period_of_the_answer(answer).as_deref(), &_the_periods_the_answer_says(answer)) {
                continue;
            }
            if (family == "minutes" || family == "hours") && _A_TEMPERATURE.is_match(answer) {
                continue;
            }
            if _A_DOCTOR.is_match(answer) {
                continue;
            }
            // The family the figure is read in, which is the answer's own when it counts the
            // same thing in another unit.
            let (mut read_family, mut read_value) = (family.clone(), value);
            let mut target = _target(answer, family, Some(value), true, period.as_deref());
            if target.is_none() {
                // The answer may name their own number as the figure to aim at.
                target = _the_aim_that_is_their_number(answer, family, value);
            }
            if target.is_none() {
                // The answer may count the same thing in another unit.
                for ((mine, theirs), factor) in _CONVERT.iter() {
                    if mine != family {
                        continue;
                    }
                    let mut other = _target(answer, theirs, None, false, period.as_deref());
                    if other.is_none() {
                        other = _the_aim_that_is_their_number(answer, theirs, round_to(value / factor, 4));
                    }
                    if let Some(other) = other {
                        // The factor turns their unit into the person's.
                        target = Some((other.0 * factor, other.1 * factor, other.2));
                        read_family = theirs.clone();
                        read_value = round_to(value / factor, 4);
                        break;
                    }
                }
            }
            let Some(target) = target else {
                continue;
            };
            // A reply that quotes their own number back is stepped over for the answer before
            // it, and kept as the last thing tried.
            if !quoted_ok
                && !spoken.is_empty()
                && pt::lower(&target.2).contains(&pt::lower(spoken))
                && (target.0 - value).abs() < 1e-9
                && (target.1 - value).abs() < 1e-9
                && !_AIM_CUE.is_match(answer)
            {
                if quoted.is_none() {
                    quoted = Some((answer.clone(), vec![(value, family.clone(), spoken.clone())]));
                }
                continue;
            }
            // An answer that names more than two figures of the kind being read is a list.
            if _aims_named(answer, read_value, &read_family).len() > 2 {
                continue;
            }
            // A short answer that just says their own figure back is passed over.
            if pt::split_ref(answer).len() <= 12
                && (pt::lower(said).contains(&pt::lower(&target.2)) || pt::lower(answer).contains(&pt::lower(spoken)))
            {
                continue;
            }
            if !mine_values.is_empty() && !_figures_in(answer).iter().any(|figure| !mine_values.contains(figure)) {
                continue;
            }
            let (low, high) = (target.0, target.1);
            if _too_far_to_compare(value, low, high) {
                continue;
            }
            // The figure is quoted mid-sentence, so its first letter sits inside the line.
            let aim = low0(&target.2);
            let aim = _with_its_unit(&aim, family, high, answer);
            if more {
                // "how much more do i need" is the gap, said as an amount to add.
                let gap = low - value;
                if gap <= 0.0 {
                    return Some(format!(
                        "None - {spoken} is already {} the {aim} that answer named.",
                        if value <= high { "at" } else { "past" }
                    ));
                }
                let Some(units) = _UNIT_SAID.get(family) else {
                    continue;
                };
                let said_unit = if gap == 1.0 { &units.0 } else { &units.1 };
                let mut amount = _the_number(round_to(gap, 1));
                if amount == "1" {
                    amount = "one".to_string();
                } else if ["2", "3", "4", "5", "6", "7", "8", "9", "10"].contains(&amount.as_str()) {
                    let n: i64 = amount.parse().unwrap();
                    amount = NUMBER_WORDS.get(&n).cloned().unwrap_or(amount);
                }
                if family == "a week" || family == "a day" {
                    return Some(format!("{} more {said_unit} would put you at the {aim} that answer named.", up0(&amount)));
                }
                return Some(format!("Another {amount} {said_unit} would put you at the {aim} that answer named."));
            }
            if too_much {
                if value > high {
                    if _A_LIMIT.is_match(answer) {
                        return Some(format!("That is more than the {aim} that answer named, so trim it back towards that."));
                    }
                    return Some(format!("That is more than the {aim} that answer named."));
                }
                if low <= value && value <= high {
                    return Some(format!("No - {} is inside the {aim} that answer named, so you are fine.", up0(spoken)));
                }
                // Under the range, "is that too much" is not this module's question at all.
                continue;
            }
            if value > high {
                return Some(format!("{} is past the {aim} that answer named, so that side is worth watching.", up0(spoken)));
            }
            if value >= high {
                let at = if high > low { "at the top of" } else { "at" };
                return Some(format!("{} is {at} the {aim} that answer named, so you are there already.", up0(spoken)));
            }
            if low <= value && value < high {
                return Some(format!("{} is inside the {aim} that answer named, so you are fine.", up0(spoken)));
            }
            // Under the figure: a count of sessions is closed by doing one more.
            if family == "a week" || family == "a day" {
                return Some(format!("{} is under the {aim} that answer named - one more would put you there.", up0(spoken)));
            }
            return Some(format!("{} is short of the {aim} that answer named.", up0(spoken)));
        }
    }
    if let Some((answer, given)) = quoted {
        return _enough_against(&[answer], &given, mine_values, said, more, too_much, true);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // Values from CPython.
    #[test]
    fn figures_as_python_reads_them() {
        assert_eq!(["2,000", "twice", "1.5", ""].map(_a_value), [2000.0, 2.0, 1.5, 0.0]);
        assert_eq!([2.0, 1.5, 100.0].map(_the_number), ["2", "1.5", "100"]);
    }
}
