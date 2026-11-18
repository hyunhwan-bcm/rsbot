//! Port of `bot/memory/resize.py`: a follow-up that only changes a figure - the same question,
//! with the new figure in it.
//!
//! "how long do i cook a 2kg chicken for" / "what about a 3kg one" is read as "how long do i cook
//! a 3kg chicken for"; "how much rice for 4 people" / "actually only 3 people" as the question for
//! three; "i can put away 50 a month" / "what if i do 100" as the line at 100 a month. This offers
//! a reading; the caller checks it reaches an entry.
#![allow(non_snake_case, clippy::all)]

use crate::data::Value;
use crate::knowledge::maths::fmt_g;
use crate::memory::mem7::{self, inline_re, res, sets, values};
use crate::py::text as pt;

res!("resize"; QUANTITY, HEADCOUNT, _A_PORTION_ASK, RATE, A_REPLACEMENT, SOMEONE_OUT, SOME_OF_THEM, STILL_AT_THE_TABLE, _TWO_COUNTS, AIMS_AT_A_FIGURE, A_CORRECTION);
sets!("resize"; ONLY_A_COUNT, ONLY_A_RATE);
values!("resize"; RATE_UNIT, _WORD_VALUE, FAMILY);
inline_re! {
    A_BARE_FIGURE = r"(?<![\w.])£?(\d[\d,]*(?:\.\d+)?)\s*(?:pounds?|quid)?", 0;
    ASKS = r"\b(?:how|what|which)\b", 0;
}
static A_NUMBER: std::sync::LazyLock<crate::py::re::Re> = std::sync::LazyLock::new(|| {
    crate::py::re::Re::new(&format!(r"(?<![\w.])({})\b", crate::data::string("bot.memory.resize.COUNT")), 0)
});

/// A figure found in a line: (value, unit, span as byte offsets, what it is a figure of).
pub type Figure = (f64, String, (usize, usize), String);

/// Python: bot/memory/resize.py:112 `quantity(text)`
///
/// The figure in a line and what it is a figure of: "a 3kg one" -> (3.0, "kg", span, "weight").
pub fn quantity(text: &str) -> Option<Figure> {
    let said = pt::lower(&pt::squash(text));
    let found = QUANTITY.search(&said)?;
    let value = mem7::py_float(found.str(1))?;
    let unit = found.str(2).to_string();
    let family = FAMILY.get(&unit).map_or(String::new(), |f| f.str().to_string());
    Some((value, unit, (found.start(), found.end()), family))
}

/// `bot.memory.recent(memory, n)`: the last n turns, read off the memory dict.
fn recent(memory: &Value, n: usize) -> Vec<Value> {
    if !memory.truthy() {
        return Vec::new();
    }
    match memory.get("turns") {
        Some(Value::List(turns)) => mem7::tail(turns, n).to_vec(),
        _ => Vec::new(),
    }
}

/// Python: bot/memory/resize.py:126 `_earlier_lines(memory, n=4)`
///
/// The person's own lines before this one (the memory as the dict PyBot reads).
pub fn _earlier_lines(memory: &Value, n: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for turn in recent(memory, n + 1) {
        let said = if turn.is_dict() { mem7::text_of(turn.get("user")) } else { String::new() };
        let said = pt::squash(&said);
        if !said.is_empty() {
            lines.push(said);
        }
    }
    lines.truncate(n);
    lines
}

/// Python: bot/memory/resize.py:139 `rate(text)`
///
/// The money a line puts by each period: "50 a month" -> (50.0, "month", span, "rate").
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the quadruple or None. Its caller
/// in store_replies.rs (`_a_rate_changed`) is adapted to read the Option.
pub fn rate(text: &str) -> Option<Figure> {
    let said = pt::lower(&pt::squash(text));
    let found = RATE.search(&said)?;
    let value = mem7::py_float(&found.str(1).replace(',', ""))?;
    Some((value, found.str(3).to_string(), (found.start(), found.end()), "rate".to_string()))
}

/// Python: bot/memory/resize.py:161 `_a_bare_rate(text)`
///
/// The figure in a line that says nothing but a new figure: "what if i do 100" -> 100.0.
pub fn _a_bare_rate(text: &str) -> Option<f64> {
    let said = pt::lower(&pt::squash(text));
    if said.is_empty() || mem7::word_count(&said) > 8 {
        return None;
    }
    let found = A_BARE_FIGURE.search(&said)?;
    let others: Vec<&str> = pt::split_ref(&said).into_iter().map(|w| pt::strip_chars(w, ".,!?;:\"'")).filter(|w| !w.is_empty() && !ONLY_A_RATE.contains(*w)).collect();
    if others != [pt::strip(found.as_str())] {
        return None;
    }
    if AIMS_AT_A_FIGURE.is_match(&said) && A_CORRECTION.is_match(&said) {
        return None; // a new pot named with an aim word
    }
    mem7::py_float(&found.str(1).replace(',', ""))
}

/// Python: bot/memory/resize.py:186 `_with_a_new_rate(said, value, unit, memory)`
///
/// The earlier line's rate swapped for this one, or None.
pub fn _with_a_new_rate(said: &str, value: f64, unit: Option<&str>, memory: &Value) -> Option<String> {
    for earlier in _earlier_lines(memory, 4) {
        let Some(there) = rate(&earlier) else {
            continue;
        };
        if unit.is_none() && there.0 == value {
            continue;
        }
        let period = unit.unwrap_or(&there.1).to_string();
        let (start, end) = there.2;
        let low = pt::lower(&pt::squash(&earlier));
        let per = RATE_UNIT.get(&period).unwrap_or_else(|| panic!("KeyError: {period:?}")).str();
        let candidate = pt::strip(&format!("{}{} {}{}", &low[..start], fmt_g(value), per, &low[end..])).to_string();
        if candidate != pt::lower(said) {
            return Some(candidate);
        }
    }
    None
}

/// Python: bot/memory/resize.py:208 `_value(raw)`
///
/// The number a word or a figure stands for: "three" and "3" are both 3.
pub fn _value(raw: &str) -> Option<i64> {
    let raw = pt::lower(pt::strip(raw));
    if pt::is_digit(&raw) {
        return mem7::int_str(&raw);
    }
    _WORD_VALUE.get(&raw).and_then(Value::as_int)
}

/// Python: bot/memory/resize.py:216 `headcount(text)`
///
/// The number of people a line gives and where it stands (byte offsets): "only 3 people" -> (3, 5, 6).
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the triple or None.
pub fn headcount(text: &str) -> Option<(i64, usize, usize)> {
    let said = pt::lower(&pt::squash(text));
    if said.is_empty() {
        return None;
    }
    let groups = _TWO_COUNTS.finditer(&said);
    if groups.len() > 1 {
        // A table counted in two groups: "3 kids and 2 adults" is five at the table.
        let total: i64 = groups.iter().map(|g| _value(g.str(1)).unwrap_or(0)).sum();
        if total != 0 {
            return Some((total, groups[0].start_of(1).expect("group 1"), groups.last().expect("two").end_of(1).expect("group 1")));
        }
    }
    if let Some(found) = HEADCOUNT.search(&said) {
        let (start, end) = found.span(1).expect("group 1");
        return _value(found.str(1)).filter(|v| *v != 0).map(|v| (v, start, end));
    }
    if mem7::word_count(&said) > 6 {
        return None;
    }
    let number = A_NUMBER.search(&said)?;
    let others = pt::split_ref(&said).into_iter().map(|w| pt::strip_chars(w, ".,!?;:\"'")).filter(|w| !w.is_empty() && !ONLY_A_COUNT.contains(*w)).count();
    if others != 1 {
        return None;
    }
    let (start, end) = number.span(1).expect("group 1");
    _value(number.str(1)).filter(|v| *v != 0).map(|v| (v, start, end))
}

/// Python: bot/memory/resize.py:254 `_a_question(earlier)`
///
/// Whether the earlier line asked something, so the rewrite is an answer rather than a fact.
pub fn _a_question(earlier: &str) -> bool {
    let said = pt::lower(&pt::squash(earlier));
    ASKS.is_match(&said) || said.ends_with('?')
}

/// Python: bot/memory/resize.py:260 `_the_ones_left(value, memory)`
///
/// The table with the people who are out taken off it, or None when there is no table.
pub fn _the_ones_left(value: i64, memory: &Value) -> Option<i64> {
    for earlier in _earlier_lines(memory, 4) {
        let Some(there) = headcount(&earlier) else {
            continue;
        };
        let left = there.0 - value;
        return if left >= 1 { Some(left) } else { None };
    }
    None
}

/// Python: bot/memory/resize.py:277 `_with_a_new_table(said, value, memory)`
///
/// The portion question on the table, asked for a new number of people, or None.
pub fn _with_a_new_table(said: &str, value: i64, memory: &Value) -> Option<String> {
    let lines = _earlier_lines(memory, 4);
    for earlier in &lines {
        if !_a_question(earlier) {
            continue;
        }
        let Some(there) = headcount(earlier) else {
            continue;
        };
        if there.0 == value {
            continue;
        }
        let low = pt::lower(&pt::squash(earlier));
        let candidate = pt::strip(&format!("{}{}{}", &low[..there.1], value, &low[there.2..])).to_string();
        if candidate != pt::lower(said) {
            return Some(candidate);
        }
    }
    // The table was named a line earlier than the question: the question is the line to ask again.
    let earlier_ask = lines.iter().find(|line| _a_question(line) && _A_PORTION_ASK.is_match(&pt::lower(line)) && headcount(line).is_none())?;
    Some(format!("{} for {} people", pt::rstrip_chars(&pt::lower(&pt::squash(earlier_ask)), "?. "), value))
}

/// Python: bot/memory/resize.py:307 `_with_a_new_weight(said, value, memory)`
///
/// The earlier question with its figure swapped for this one, or None.
pub fn _with_a_new_weight(said: &str, value: Option<f64>, memory: &Value) -> Option<String> {
    let mut value = value;
    let mut unit: Option<String> = None;
    let mut family: Option<String> = None;
    if let Some((v, u, _span, f)) = quantity(said) {
        value = Some(v);
        unit = Some(u);
        family = Some(f);
    }
    for earlier in _earlier_lines(memory, 4) {
        let Some(there) = quantity(&earlier) else {
            continue;
        };
        if Some(there.0) == value {
            continue;
        }
        let this_unit = match &unit {
            None => {
                if !_a_question(&earlier) {
                    continue;
                }
                unit = Some(there.1.clone());
                family = Some(there.3.clone());
                there.1.clone()
            }
            Some(u) => {
                if Some(&there.3) != family.as_ref() {
                    continue;
                }
                u.clone()
            }
        };
        let (start, end) = there.2;
        let low = pt::lower(&pt::squash(&earlier));
        // `"%g" % value` (value is None only for a line with no figure, which never gets here
        // with no unit of its own)
        let figure = fmt_g(value.expect("TypeError: a figure"));
        let candidate = pt::strip(&format!("{}{}{}{}", &low[..start], figure, this_unit, &low[end..])).to_string();
        if candidate != pt::lower(said) {
            return Some(candidate);
        }
    }
    None
}

/// Python: bot/memory/resize.py:338 `rewritten(text, memory)`
///
/// The line read with the earlier line's figure swapped for this one, or None. `memory` is the
/// memory as the dict PyBot reads (its callers hand it over that way).
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the line or None.
pub fn rewritten(text: &str, memory: &Value) -> Option<String> {
    let said = pt::squash(text);
    if said.is_empty() || mem7::word_count(&said) > 10 {
        return None;
    }
    if let Some(now) = rate(&said) {
        return _with_a_new_rate(&said, now.0, Some(&now.1), memory);
    }
    if let Some(bare) = _a_bare_rate(&said) {
        // "what if i do 100" after "i can put away 50 a month": a new rate for the same goal.
        if let Some(resized) = _with_a_new_rate(&said, bare, None, memory).filter(|r| !r.is_empty()) {
            return Some(resized);
        }
    }
    if quantity(&said).is_some() {
        return _with_a_new_weight(&said, None, memory);
    }
    let (value, _, _) = headcount(&said)?;
    if SOMEONE_OUT.is_match(&said) {
        let left = _the_ones_left(value, memory)?;
        return _with_a_new_table(&said, left, memory);
    }
    if SOME_OF_THEM.is_match(&said) && !STILL_AT_THE_TABLE.is_match(&said) {
        return None; // "two of them are kids" says who is at the table, not how many
    }
    if HEADCOUNT.is_match(&pt::lower(&said)) {
        return _with_a_new_table(&said, value, memory);
    }
    if !A_REPLACEMENT.is_match(&said) {
        return None;
    }
    // A bare figure corrects whatever figure is on the table.
    _with_a_new_table(&said, value, memory)
        .or_else(|| _with_a_new_weight(&said, Some(value as f64), memory))
        .or_else(|| _with_a_new_rate(&said, value as f64, None, memory))
}
