//! Port of `bot/memory/outgoings.py`: what comes in and what goes out ("im on 1800 a month and
//! rent is 700"), answered with arithmetic on the person's own figures: what is left, or the
//! share of a take-home to put by in a chat that is already about saving.
//!
//! Python's positions here are character indices and its windows are counted in characters, so
//! the figures carry character positions (`Sum`) and the windows are cut with `text::slice`.

use crate::data::{self, Value};
use crate::py::re::{self, Match, Re};
use crate::py::text;
use crate::state::Memory;
use std::collections::{BTreeSet, HashMap};
use std::sync::LazyLock;

const M: &str = "bot.memory.outgoings.";
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn pool(name: &str) -> Vec<String> {
    data::strs(&format!("{M}{name}"))
}

static SUM: LazyLock<&'static Re> = LazyLock::new(|| rx("SUM"));
static BARE_NUMBER: LazyLock<&'static Re> = LazyLock::new(|| rx("BARE_NUMBER"));
static INCOME: LazyLock<&'static Re> = LazyLock::new(|| rx("INCOME"));
static FIXED_COST: LazyLock<&'static Re> = LazyLock::new(|| rx("FIXED_COST"));
static AFTER_THE_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| rx("AFTER_THE_FIGURE"));
static NOT_A_COST_NAME: LazyLock<&'static Re> = LazyLock::new(|| rx("NOT_A_COST_NAME"));
static MONTH: LazyLock<&'static Re> = LazyLock::new(|| rx("MONTH"));
static WEEK: LazyLock<&'static Re> = LazyLock::new(|| rx("WEEK"));
static YEAR: LazyLock<&'static Re> = LazyLock::new(|| rx("YEAR"));
static NOT_A_BUDGET_LINE: LazyLock<&'static Re> = LazyLock::new(|| rx("NOT_A_BUDGET_LINE"));
static AN_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("AN_ASK"));
static A_MEASURE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_MEASURE"));
static A_CODE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_CODE"));
static A_HAND_HELD: LazyLock<&'static Re> = LazyLock::new(|| rx("A_HAND_HELD"));
static _THE_COST: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.memory.outgoings._THE_COST"));
static LOW_SHARE: LazyLock<f64> = LazyLock::new(|| data::get("bot.memory.outgoings.LOW_SHARE").as_f64().unwrap());
static HIGH_SHARE: LazyLock<f64> = LazyLock::new(|| data::get("bot.memory.outgoings.HIGH_SHARE").as_f64().unwrap());
static LEFT: LazyLock<Vec<String>> = LazyLock::new(|| pool("LEFT"));
static A_YEAR_AND_A_MONTH: LazyLock<Vec<String>> = LazyLock::new(|| pool("A_YEAR_AND_A_MONTH"));
static FIRST_MOVE: LazyLock<Vec<String>> = LazyLock::new(|| pool("FIRST_MOVE"));
static A_FIRST_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("A_FIRST_ASK"));
static A_LEFT_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("A_LEFT_ASK"));
static LEFT_AGAIN: LazyLock<Vec<String>> = LazyLock::new(|| pool("LEFT_AGAIN"));
static A_STEP_ASK_POINTS_BACK: LazyLock<&'static Re> = LazyLock::new(|| rx("A_STEP_ASK_POINTS_BACK"));
static A_SHARE: LazyLock<Vec<String>> = LazyLock::new(|| pool("A_SHARE"));

static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[A-Za-z']+", re::U));
static THE_SUM_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bleaves \u{a3}|\bleft after the\b", re::U));
static ABOUT_SAVING: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\b(?:save|saves|saving|savings|put\s+(?:by|aside|away)|budget|deposit|putting\s+by)\b", re::I | re::U)
});

/// A figure that can be money: its value and where it is (character indices, as Python's).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sum {
    pub value: f64,
    pub start: usize,
    pub end: usize,
}

/// `s[a:b]` with Python's character indices (a is never negative here).
fn cut(s: &str, a: isize, b: Option<isize>) -> String {
    text::slice(s, a.max(0), b)
}

/// A match's start and end as character indices.
fn chars_of(s: &str, m: &Match) -> (usize, usize) {
    (text::char_index(s, m.start()), text::char_index(s, m.end()))
}

/// `"%s and the %s"`, `"{:,}"` and the like for pounds.
fn with_commas(int_part: i64) -> String {
    text::thousands(int_part)
}

/// Python: bot/memory/outgoings.py:207 `_choice(shape, line, memory=None)`
/// A shape for this line, never the one just used.
pub fn _choice(shape: &[String], line: &str, memory: Option<&Memory>) -> String {
    let was = memory.and_then(|m| m.turns.last()).map(|t| t.bot.clone()).unwrap_or_default();
    let start = (line.chars().map(|c| c as usize).sum::<usize>()) % shape.len();
    for step in 0..shape.len() {
        let candidate = &shape[(start + step) % shape.len()];
        let opener = text::strip(candidate.split('{').next().unwrap_or("")).to_string();
        if opener.is_empty() || !was.contains(&text::slice(&opener, 0, Some(28))) {
            return candidate.clone();
        }
    }
    shape[start].clone()
}

/// Python: bot/memory/outgoings.py:226 `_the_names(costs)`
/// The costs as one phrase: "rent", "rent and the childcare", "rent, the nursery and the loan".
pub fn _the_names(costs: &[(String, f64)]) -> String {
    let mut names: Vec<String> = Vec::new();
    for (name, _) in costs {
        let name = _THE_COST.get(name).cloned().unwrap_or_else(|| name.clone());
        if !names.contains(&name) {
            names.push(name);
        }
    }
    match names.len() {
        1 => names[0].clone(),
        2 => format!("{} and the {}", names[0], names[1]),
        n => format!("{} and the {}", names[..n - 1].join(", "), names[n - 1]),
    }
}

/// Whether the character after a figure is a letter, or a measure follows it.
fn a_word_after(text_: &str, end: usize) -> bool {
    let next = cut(text_, end as isize, Some(end as isize + 1));
    text::is_alpha(&next) || A_MEASURE.match_(&cut(text_, end as isize, None)).is_some()
}

/// Python: bot/memory/outgoings.py:240 `_sums(text)`
/// Every figure in the line that can be money, in order.
pub fn _sums(text_: &str) -> Vec<Sum> {
    let mut found: Vec<Sum> = Vec::new();
    let mut taken: Vec<(usize, usize)> = Vec::new();
    for one in SUM.finditer(text_) {
        let value = one.name("marked").filter(|v| !v.is_empty()).or(one.name("bare")).unwrap_or("");
        let unit = one.name("marked_unit").filter(|v| !v.is_empty()).or(one.name("bare_unit")).unwrap_or("").to_lowercase();
        let Ok(mut number) = value.replace(',', "").parse::<f64>() else { continue };
        if matches!(unit.as_str(), "k" | "grand" | "thousand") {
            number *= 1000.0;
        }
        let (start, end) = chars_of(text_, &one);
        if a_word_after(text_, end) {
            continue;
        }
        if number >= 20.0 {
            found.push(Sum { value: number, start, end });
            taken.push((start, end));
        }
    }
    for one in BARE_NUMBER.finditer(text_) {
        let (start, end) = chars_of(text_, &one);
        if taken.iter().any(|(a, b)| *a <= start && start < *b) {
            continue;
        }
        if a_word_after(text_, end) {
            continue;
        }
        let Ok(number) = one.as_str().replace(',', "").parse::<f64>() else { continue };
        if number >= 20.0 {
            found.push(Sum { value: number, start, end });
        }
    }
    found.sort_by_key(|s| s.start);
    found
}

/// Python: bot/memory/outgoings.py:280 `_period(text)`
/// The unit a figure is said in: "a month" unless the line says a week or a year.
pub fn _period(text_: &str) -> String {
    if WEEK.is_match(text_) {
        "a week".into()
    } else if YEAR.is_match(text_) {
        "a year".into()
    } else if MONTH.is_match(text_) {
        "a month".into()
    } else {
        String::new()
    }
}

/// Python: bot/memory/outgoings.py:291 `_units_in(text)`
pub fn _units_in(text_: &str) -> BTreeSet<&'static str> {
    [(*MONTH, "a month"), (*WEEK, "a week"), (*YEAR, "a year")].into_iter().filter(|(p, _)| p.is_match(text_)).map(|(_, u)| u).collect()
}

/// Python: bot/memory/outgoings.py:300 `_pounds(value)`
/// £1,100 / £180.50 - the mark on, and the figure the way people write it.
pub fn _pounds(value: f64) -> String {
    let whole = text::round_half_even(value);
    if (value - whole as f64).abs() < 0.005 {
        return format!("£{}", with_commas(whole));
    }
    let fixed = format!("{value:.2}");
    let (int, frac) = fixed.split_once('.').unwrap();
    let int: i64 = int.parse().unwrap();
    let sign = if value < 0.0 && int == 0 { "-" } else { "" };
    format!("£{sign}{}.{frac}", with_commas(int))
}

/// Python: bot/memory/outgoings.py:307 `_with_unit(value, unit)`
pub fn _with_unit(value: f64, unit: &str) -> String {
    let figure = _pounds(value);
    if unit.is_empty() { figure } else { format!("{figure} {unit}") }
}

/// Python: bot/memory/outgoings.py:313 `_in_phrase(text, start, end, pattern, span=18)`
/// Whether `pattern` runs around the figure, over it.
pub fn _in_phrase(text_: &str, start: usize, end: usize, pattern: &Re, span: usize) -> bool {
    pattern.is_match(&cut(text_, start as isize - span as isize, Some((end + span) as isize)))
}

/// Python: bot/memory/outgoings.py:318 `_near(text, start, end, pattern, before=30, after=30)`
/// Whether `pattern` sits within those characters of the figure, either side of it.
pub fn _near(text_: &str, start: usize, end: usize, pattern: &Re, before: usize, after: usize) -> bool {
    pattern.is_match(&cut(text_, start as isize - before as isize, Some(start as isize)))
        || pattern.is_match(&cut(text_, end as isize, Some((end + after) as isize)))
}

/// Python: bot/memory/outgoings.py:324 `_the_nearest(line, cue, sums, after=20, before=6)`
/// The sum a word is talking about: the first one after it, else the last one before it, as
/// (distance, sum). `cue` is the word's (start, end) in characters.
pub fn _the_nearest(cue: (usize, usize), sums: &[Sum], after: isize, before: isize) -> Option<(isize, Sum)> {
    let mut best: Option<(isize, Sum)> = None;
    for sum in sums {
        let gap = sum.start as isize - cue.1 as isize;
        let distance = if 0 <= gap && gap <= after {
            gap
        } else {
            let gap = cue.0 as isize - sum.end as isize;
            if !(0 <= gap && gap <= before) {
                continue;
            }
            gap
        };
        if best.is_none_or(|(d, _)| distance < d) {
            best = Some((distance, *sum));
        }
    }
    best
}

/// Python: bot/memory/outgoings.py:345 `_the_incomes_in_the_line(line, sums=None)`
/// Every figure the line calls money coming in.
pub fn _the_incomes_in_the_line(line: &str, sums: Option<&[Sum]>) -> Vec<Sum> {
    if line.is_empty() || !INCOME.is_match(line) || A_CODE.is_match(line) {
        return Vec::new();
    }
    let owned;
    let sums = match sums {
        Some(sums) => sums,
        None => {
            owned = _sums(line);
            &owned
        }
    };
    let mut found = Vec::new();
    let mut seen = Vec::new();
    for cue in INCOME.finditer(line) {
        let Some((_, sum)) = _the_nearest(chars_of(line, &cue), sums, 20, 8) else { continue };
        if seen.contains(&sum.start) {
            continue;
        }
        seen.push(sum.start);
        found.push(sum);
    }
    found
}

/// Python: bot/memory/outgoings.py:366 `the_income_in_the_line(text)`
/// (value, period) for "im on 1800 a month" and "i take home 1800", or None.
pub fn the_income_in_the_line(text_: &str) -> Option<(f64, String)> {
    if text_.is_empty() {
        return None;
    }
    let incomes = _the_incomes_in_the_line(text_, None);
    if incomes.is_empty() {
        return None;
    }
    Some((incomes.iter().map(|s| s.value).sum(), _period(text_)))
}

/// Python: bot/memory/outgoings.py:383 `_the_cost_the_figure_is_for(line, start, end)`
/// The cost a figure belongs to, or "" when nothing beside it says money is going out.
pub fn _the_cost_the_figure_is_for(line: &str, start: usize, end: usize) -> String {
    let before = cut(line, start as isize - 26, Some(start as isize));
    let after = cut(line, end as isize, Some(end as isize + 26));
    let mut best: Option<(usize, String)> = None;
    let before_len = text::len(&before);
    for found in FIXED_COST.finditer(&before) {
        let distance = before_len - text::char_index(&before, found.end());
        if best.as_ref().is_none_or(|(d, _)| distance < *d) {
            best = Some((distance, found.as_str().to_lowercase()));
        }
    }
    for found in FIXED_COST.finditer(&after) {
        let distance = text::char_index(&after, found.start());
        if best.as_ref().is_none_or(|(d, _)| distance < *d) {
            best = Some((distance, found.as_str().to_lowercase()));
        }
    }
    if let Some((_, name)) = best {
        return name;
    }
    match AFTER_THE_FIGURE.match_(&after) {
        Some(on_shape) if NOT_A_COST_NAME.match_(on_shape.name_str("noun")).is_none() => on_shape.name_str("noun").to_lowercase(),
        _ => String::new(),
    }
}

/// Python: bot/memory/outgoings.py:410 `_the_unit_beside(line, end)`
/// The period said right after a figure: "900 a month" is a month, "900" says nothing.
pub fn _the_unit_beside(line: &str, end: usize) -> String {
    let tail = cut(line, end as isize, Some(end as isize + 16));
    for (pattern, unit) in [(*MONTH, "a month"), (*WEEK, "a week"), (*YEAR, "a year")] {
        if let Some(found) = pattern.search(&tail) {
            if text::strip_chars(&tail[..found.start()], " ,-").is_empty() {
                return unit.into();
            }
        }
    }
    String::new()
}

/// Python: bot/memory/outgoings.py:420 `the_costs_with_units_in_the_line(text)`
/// [(name, value, unit)] for the costs the line names; a figure nothing explains takes the whole
/// reading with it.
pub fn the_costs_with_units_in_the_line(text_: &str) -> Vec<(String, f64, String)> {
    if text_.is_empty() {
        return Vec::new();
    }
    let sums = _sums(text_);
    let incomes: Vec<usize> = _the_incomes_in_the_line(text_, Some(&sums)).iter().map(|s| s.start).collect();
    let mut found = Vec::new();
    for sum in &sums {
        if incomes.contains(&sum.start) {
            continue;
        }
        if _in_phrase(text_, sum.start, sum.end, &A_HAND_HELD, 18) {
            continue;
        }
        let name = _the_cost_the_figure_is_for(text_, sum.start, sum.end);
        if name.is_empty() {
            return Vec::new();
        }
        found.push((name, sum.value, _the_unit_beside(text_, sum.end)));
    }
    found
}

/// Python: bot/memory/outgoings.py:452 `the_outgoings_in_the_line(text)`
pub fn the_outgoings_in_the_line(text_: &str) -> Vec<(String, f64)> {
    the_costs_with_units_in_the_line(text_).into_iter().map(|(name, value, _)| (name, value)).collect()
}

/// Python: bot/memory/outgoings.py:457 `_there_is_memory(memory)`
/// Whether the chat has said anything before this line.
pub fn _there_is_memory(memory: Option<&Memory>) -> bool {
    memory.is_some_and(|m| m.turns.iter().any(|t| !text::strip(&t.user).is_empty() || !text::strip(&t.bot).is_empty()))
}

/// The person's lines from the two turns before (that said something), newest first.
fn two_lines_back(memory: Option<&Memory>) -> Vec<&str> {
    let Some(memory) = memory else { return Vec::new() };
    memory.turns.iter().rev().map(|t| t.user.as_str()).filter(|line| !text::strip(line).is_empty()).take(2).collect()
}

/// Python: bot/memory/outgoings.py:472 `_the_income_before(memory)`
/// (value, period) from the two lines before, when this one says only what goes out.
pub fn _the_income_before(memory: Option<&Memory>) -> Option<(f64, String)> {
    two_lines_back(memory).into_iter().find_map(the_income_in_the_line)
}

/// Python: bot/memory/outgoings.py:491 `_the_costs_before(memory)`
/// The costs the two lines before named with a figure, oldest first.
pub fn _the_costs_before(memory: Option<&Memory>) -> Vec<(String, f64)> {
    let mut found = Vec::new();
    for line in two_lines_back(memory) {
        let mut earlier = the_outgoings_in_the_line(line);
        earlier.extend(found);
        found = earlier;
    }
    found
}

/// Python: bot/memory/outgoings.py:513 `_remember(memory, left, unit)`
/// Leave a note that this turn was the month set out, for the line that asks what comes next.
pub fn _remember(memory: Option<&mut Memory>, left: f64, unit: &str) {
    if let Some(memory) = memory {
        let mut month = Value::dict();
        month.set_key("left", left);
        month.set_key("unit", unit);
        memory.set("the_month_set_out", month);
    }
}

/// Python: bot/memory/outgoings.py:519 `_the_pot_in_the_line(text)`
/// The figure the line puts by, or 0: "rent is 750 and i put 200 aside".
pub fn _the_pot_in_the_line(text_: &str) -> f64 {
    _sums(text_).into_iter().find(|s| _in_phrase(text_, s.start, s.end, &A_HAND_HELD, 18)).map(|s| s.value).unwrap_or(0.0)
}

/// Python: bot/memory/outgoings.py:528 `_the_left_again(line, memory)`
/// The figure the month left, said again to the line that asks what is left of it.
pub fn _the_left_again(line: &str, memory: Option<&Memory>) -> String {
    let Some(month) = memory.and_then(|m| m.get("the_month_set_out")).filter(|m| m.is_dict()) else { return String::new() };
    let left = month.get("left").cloned().unwrap_or(Value::None);
    let unit = month.get("unit").filter(|u| u.truthy()).map(|u| super::pyv::str_of(u)).unwrap_or_else(|| "a month".into());
    if !left.truthy() || super::pyv::num(&left) <= 0.0 {
        return String::new();
    }
    let figure = format!("\u{a3}{}", with_commas(text::round_half_even(super::pyv::num(&left))));
    super::pyv::format(&_choice(&LEFT_AGAIN, line, memory), &[("left", &figure), ("unit", &unit)])
}

/// Python: bot/memory/outgoings.py:541 `_the_ask_points_back(line)`
pub fn _the_ask_points_back(line: &str) -> bool {
    A_STEP_ASK_POINTS_BACK.is_match(line) || WORDS.findall_str(line).len() <= 4
}

/// Python: bot/memory/outgoings.py:548 `_the_month_was_just_set_out(memory)`
pub fn _the_month_was_just_set_out(memory: Option<&Memory>) -> bool {
    let Some(memory) = memory else { return false };
    if !memory.get("the_month_set_out").is_some_and(Value::truthy) {
        return false;
    }
    memory.turns.last().is_some_and(|turn| THE_SUM_SAID.is_match(&turn.bot))
}

/// Python: bot/memory/outgoings.py:559 `_the_unit_in_the_chat(memory)`
/// The unit the chat is speaking in: "how much should i save a month" sets the month.
pub fn _the_unit_in_the_chat(memory: Option<&Memory>) -> String {
    let Some(memory) = memory else { return String::new() };
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(2)..].iter().rev().map(|t| _period(&t.user)).find(|u| !u.is_empty()).unwrap_or_default()
}

/// Python: bot/memory/outgoings.py:571 `_the_chat_is_about_saving(memory)`
pub fn _the_chat_is_about_saving(memory: Option<&Memory>) -> bool {
    let Some(memory) = memory else { return false };
    let turns = &memory.turns;
    let said: Vec<&str> = turns[turns.len().saturating_sub(2)..].iter().flat_map(|t| [t.user.as_str(), t.bot.as_str()]).collect();
    ABOUT_SAVING.is_match(&said.join(" "))
}

/// Python: bot/memory/outgoings.py:584 `reply_for(text, memory=None)`
/// The reply a line about the month gets, or "" when it is not one.
pub fn reply_for(text_: &str, mut memory: Option<&mut Memory>) -> String {
    let line = text::strip(text_).to_string();
    if line.is_empty() || text::len(&line) > 220 {
        return String::new();
    }
    let view = memory.as_deref();
    if A_LEFT_ASK.is_match(&line) && _the_month_was_just_set_out(view) {
        return _the_left_again(&line, view);
    }
    if A_FIRST_ASK.is_match(&line) && _the_month_was_just_set_out(view) && _the_ask_points_back(&line) {
        return _choice(&FIRST_MOVE, &line, view);
    }
    if NOT_A_BUDGET_LINE.is_match(&line) || AN_ASK.is_match(&line) {
        return String::new();
    }
    if _units_in(&line).len() > 1 {
        return String::new();
    }
    let unit = _period(&line);
    let mut income = the_income_in_the_line(&line);
    let with_units = the_costs_with_units_in_the_line(&line);
    let mut costs: Vec<(String, f64)> = with_units.iter().map(|(name, value, _)| (name.clone(), *value)).collect();
    if !_there_is_memory(view) && !(income.is_some() && !costs.is_empty()) {
        return String::new();
    }
    if let Some((value, said_unit)) = &income {
        if unit.is_empty() {
            income = Some((*value, _the_unit_in_the_chat(view)));
        } else {
            income = Some((*value, said_unit.clone()));
        }
    }
    let own_month = income.is_some();
    if income.is_none() {
        let Some(earlier) = _the_income_before(view) else { return String::new() };
        if costs.is_empty() {
            return String::new();
        }
        let earlier_unit = if !earlier.1.is_empty() {
            earlier.1.clone()
        } else if !unit.is_empty() {
            unit.clone()
        } else {
            _the_unit_in_the_chat(view)
        };
        income = Some((earlier.0, earlier_unit));
    }
    let (value, unit) = income.unwrap();
    if !costs.is_empty() {
        let mut before = Vec::new();
        if !own_month {
            let named: Vec<String> = costs.iter().map(|(name, _)| name.clone()).collect();
            before = _the_costs_before(view).into_iter().filter(|(name, _)| !named.contains(name)).collect();
        }
        before.extend(costs);
        costs = before;
        let named = _the_names(&costs);
        let cost_unit = with_units.last().map(|(_, _, u)| u.clone()).unwrap_or_default();
        if costs.len() == 1 && !cost_unit.is_empty() && cost_unit != unit {
            return String::new();
        }
        let out: f64 = costs.iter().map(|(_, cost)| cost).sum();
        if unit == "a year" && cost_unit != "a year" {
            let a_month = text::round_half_even(value / 12.0) as f64;
            let left = a_month - out;
            if left <= 0.0 || left < a_month * 0.05 {
                return String::new();
            }
            let view = memory.as_deref();
            let reply = super::pyv::format(
                &_choice(&A_YEAR_AND_A_MONTH, &line, view),
                &[("in_amount", &_pounds(value)), ("a_month", &_pounds(a_month)), ("named", &named), ("left", &_with_unit(left, "a month"))],
            );
            _remember(memory.as_deref_mut(), left, "a month");
            return reply;
        }
        let left = value - out;
        if left <= 0.0 || left < value * 0.05 {
            return String::new();
        }
        let pot = _the_pot_in_the_line(&line);
        let view = memory.as_deref();
        let mut reply = super::pyv::format(
            &_choice(&LEFT, &line, view),
            &[("left", &_with_unit(left, &unit)), ("named", &named), ("in_amount", &_pounds(value)), ("out_amount", &_pounds(out))],
        );
        if pot != 0.0 {
            reply += &format!(" The {} you put by comes out of that.", _pounds(pot));
        }
        _remember(memory.as_deref_mut(), left, &unit);
        return reply;
    }
    let view = memory.as_deref();
    if !_the_chat_is_about_saving(view) || unit.is_empty() {
        return String::new();
    }
    super::pyv::format(
        &_choice(&A_SHARE, &line, view),
        &[("in_amount", &_pounds(value)), ("low", &_pounds(value * *LOW_SHARE)), ("high", &_pounds(value * *HIGH_SHARE)), ("period", &unit)],
    )
}
