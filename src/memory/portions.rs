//! Port of `bot/memory/portions.py`: a follow-up asking how many the answer in hand feeds is
//! answered by that answer's own count.
//!
//! "how much pasta for 4 people" / "how many does that serve" -> "That is for four - the answer
//! puts it at 400 g in total." An answer that gives an amount per person and no total asks how
//! many are coming, and the count they answer with is worked out from the figure the answer gave.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::knowledge::maths::fmt_g;
use crate::memory::history;
use crate::memory::mem7::{self, res, values};
use crate::py::re::{Match, Re};
use crate::py::text as pt;
use crate::state::Memory;
use std::sync::LazyLock;

res!("portions"; A_PORTION_ASK, FEEDS_A_COUNT, FOR_A_COUNT, PER_PERSON, AN_AMOUNT, A_TIME_AFTER_A_COUNT, A_COUNT_OF_PEOPLE, A_BARE_COUNT, A_FIGURE_AND_UNIT, CLAUSE_EDGE, _A_NUMBER_ANYWHERE, _A_UNIT_AFTER, A_CONFIRMATION);
values!("portions"; _WORDS_FOR_NUMBERS);
static A_NEW_COUNTS: LazyLock<Vec<&'static Re>> =
    LazyLock::new(|| data::get("bot.memory.portions._A_NEW_COUNTS").items().iter().map(|p| p.as_re().expect("a pattern")).collect());

/// A count found in a line: the number as said, where it ends (a byte offset), and the number it
/// replaces when the line names one ("6 not 4").
#[derive(Clone, Debug)]
pub struct Count {
    pub n: String,
    pub n_end: usize,
    pub old: Option<String>,
}

impl Count {
    fn of(m: &Match) -> Count {
        Count { n: m.name_str("n").to_string(), n_end: m.name_span("n").map_or(m.end(), |s| s.1), old: m.name("old").map(str::to_string) }
    }
}

/// `pattern.match(text, pos)`.
fn match_at<'t>(pattern: &'t Re, text: &'t str, pos: usize) -> Option<Match<'t>> {
    pattern.search_at(text, pos).filter(|m| m.start() == pos)
}

/// Python: bot/memory/portions.py:82 `_a_time_not_a_portion(said, found)`
///
/// True when the count the answer gives is a length of time, not a number of people.
pub fn _a_time_not_a_portion(said: &str, found: &Match) -> bool {
    let end = found.name_span("n").map_or(found.end(), |s| s.1);
    match_at(&A_TIME_AFTER_A_COUNT, said, end).is_some()
}

/// Python: bot/memory/portions.py:117 `_the_last_answer(memory)`
pub fn _the_last_answer(memory: &Memory) -> String {
    history::the_last_answer(memory)
}

/// Python: bot/memory/portions.py:127 `_the_clause_with_the_count(said, start, end)`
///
/// The sentence or clause of the answer around the count (byte offsets), so the reply can quote it.
pub fn _the_clause_with_the_count(said: &str, start: usize, end: usize) -> String {
    let mut left = 0;
    for edge in CLAUSE_EDGE.finditer(said) {
        if edge.end() <= start {
            left = edge.end();
        } else if edge.start() >= end {
            return pt::strip_chars(&said[left..edge.start()], " .").to_string();
        }
    }
    pt::strip_chars(&said[left..], " .").to_string()
}

/// Python: bot/memory/portions.py:138 `_the_number(word)`
pub fn _the_number(word: &str) -> String {
    pt::lower(&pt::squash(word))
}

/// Python: bot/memory/portions.py:151 `_as_a_count(word)`
///
/// The number a count was said with, or 0 when it is not one this reads.
pub fn _as_a_count(word: &str) -> i64 {
    let word = _the_number(word);
    if pt::is_digit(&word) {
        return mem7::int_str(&word).unwrap_or(0);
    }
    _WORDS_FOR_NUMBERS.get(&word).and_then(Value::as_int).unwrap_or(0)
}

/// Python: bot/memory/portions.py:159 `_in_grams(figure, unit)`
///
/// A per-head figure as grams; 0 for a unit this cannot weigh.
pub fn _in_grams(figure: f64, unit: &str) -> f64 {
    match pt::lower(unit).as_str() {
        "kg" | "kilo" | "kilos" => figure * 1000.0,
        "g" | "gram" | "grams" => figure,
        "oz" | "ounce" | "ounces" => figure * 28.35,
        "lb" | "lbs" | "pound" | "pounds" => figure * 453.6,
        "l" | "litre" | "litres" => figure * 1000.0,
        "ml" | "millilitre" | "millilitres" => figure * 1.0,
        _ => 0.0,
    }
}

/// Python: bot/memory/portions.py:176 `_counts_people(said, found)`
///
/// True when a per-person word counts people rather than the parts of a recipe.
pub fn _counts_people(said: &str, found: &Match) -> bool {
    if pt::lower(found.as_str()) != "each" {
        return true;
    }
    let before = pt::lower(pt::rstrip(&said[..found.start()]));
    !["of", "in", "for", "with"].iter().any(|w| before.ends_with(w))
}

/// The grams of a figure-and-unit match.
fn grams_of(figure: &Match) -> f64 {
    _in_grams(mem7::py_float(figure.name_str("n")).unwrap_or(0.0), figure.name_str("u"))
}

/// Python: bot/memory/portions.py:189 `_the_per_head_figure(said)`
///
/// The grams one person gets, as the answer says it, or 0 when it does not weigh one person.
pub fn _the_per_head_figure(said: &str) -> f64 {
    for found in PER_PERSON.finditer(said) {
        if !_counts_people(said, &found) {
            continue;
        }
        let head = &said[..found.start()];
        let from = [head.rfind('.'), head.rfind(','), head.rfind(';')].into_iter().flatten().max().map_or(0, |at| at + 1);
        let clause = &head[from..];
        for figure in A_FIGURE_AND_UNIT.finditer(clause).iter().rev() {
            let grams = grams_of(figure);
            if grams != 0.0 {
                return grams;
            }
        }
        for figure in A_FIGURE_AND_UNIT.finditer(said) {
            let grams = grams_of(&figure);
            if grams != 0.0 {
                return grams;
            }
        }
    }
    0.0
}

/// Python: bot/memory/portions.py:213 `_a_total(grams)`
///
/// A total weight said the way a person says one: whole kilos above a kilo, grams below.
pub fn _a_total(grams: f64) -> String {
    if grams >= 1000.0 {
        // `round(grams / 1000.0, 1)`: the decimal rounding of the exact value, half to even.
        let kilos: f64 = format!("{:.1}", grams / 1000.0).parse().unwrap_or(0.0);
        return format!("{} kg", fmt_g(kilos));
    }
    format!("{} g", fmt_g(pt::round_half_even(grams) as f64))
}

/// Python: bot/memory/portions.py:222 `the_portion_answer(memory)`
///
/// The answer in hand when it is a portion answer, or "" when it is about something else.
pub fn the_portion_answer(memory: &Memory) -> String {
    let said = _the_last_answer(memory);
    if said.is_empty() || !(AN_AMOUNT.is_match(&said) || PER_PERSON.is_match(&said) || FEEDS_A_COUNT.is_match(&said)) {
        return String::new();
    }
    said
}

/// The count the answer itself works the portion out to, when it is not a length of time.
fn the_answers_count(said: &str) -> Option<Match<'_>> {
    let counted = FEEDS_A_COUNT.search(said).or_else(|| FOR_A_COUNT.search(said))?;
    if _a_time_not_a_portion(said, &counted) {
        None
    } else {
        Some(counted)
    }
}

/// Python: bot/memory/portions.py:234 `an_ask(text, memory)`
///
/// True when this line asks how many the portion answer in hand feeds.
pub fn an_ask(text: &str, memory: &Memory) -> bool {
    if !A_PORTION_ASK.is_match(text) {
        return false;
    }
    let said = the_portion_answer(memory);
    if said.is_empty() {
        return false;
    }
    if the_answers_count(&said).is_some() {
        return true;
    }
    PER_PERSON.is_match(&said) && _the_per_head_figure(&said) != 0.0
}

/// Python: bot/memory/portions.py:259 `reply_for(text, memory)`
///
/// The reply to a line asking how many the answer in hand feeds, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn reply_for(text: &str, memory: &mut Memory) -> Option<String> {
    if !A_PORTION_ASK.is_match(text) {
        return None;
    }
    let said = the_portion_answer(memory);
    if said.is_empty() {
        return None;
    }
    if let Some(counted) = the_answers_count(&said) {
        let number = _the_number(counted.name_str("n"));
        let clause = _the_clause_with_the_count(&said, counted.start(), counted.end());
        if let Some(amount) = AN_AMOUNT.search(&clause) {
            return Some(format!("That is for {number} - the answer puts it at {} in total.", pt::squash(amount.as_str())));
        }
        return Some(format!("That is for {number}."));
    }
    if PER_PERSON.is_match(&said) {
        let grams = _the_per_head_figure(&said);
        if grams == 0.0 {
            return Some("That is per person, so it goes up with however many you are feeding.".into());
        }
        let turn = history::turns_before(memory).len() as i64;
        memory.set("portion", mem7::dict(vec![("grams", Value::Float(grams)), ("turn", Value::from(turn))]));
        return Some("That is per person, not a total - say how many you are feeding and I will work the total out for you.".into());
    }
    None
}

/// Python: bot/memory/portions.py:312 `_is_an_amount_not_a_count(text, found)`
///
/// True when the figure is an amount they want rather than a number of people.
pub fn _is_an_amount_not_a_count(text: &str, found: &Count) -> bool {
    match_at(&_A_UNIT_AFTER, text, found.n_end).is_some()
}

/// Python: bot/memory/portions.py:322 `_says_two_counts(text, found)`
///
/// True when the line works out a count of its own - "3 kids and 2 adults" - and not this one.
pub fn _says_two_counts(text: &str, found: &Count) -> bool {
    if found.old.as_deref().is_some_and(|old| !old.is_empty()) {
        return false;
    }
    _A_NUMBER_ANYWHERE.finditer(text).len() > 1
}

/// Python: bot/memory/portions.py:333 `_the_new_count(text)`
///
/// The count a correction names, as the line says it, or None.
pub fn _the_new_count(text: &str) -> Option<Count> {
    for pattern in A_NEW_COUNTS.iter() {
        if let Some(found) = pattern.search(text) {
            if _as_a_count(found.name_str("n")) != 0 {
                let count = Count::of(&found);
                if _is_an_amount_not_a_count(text, &count) || _says_two_counts(text, &count) {
                    return None;
                }
                return Some(count);
            }
        }
    }
    None
}

/// Python: bot/memory/portions.py:349 `_is_a_digit(word)`
pub fn _is_a_digit(word: &str) -> bool {
    pt::is_digit(pt::strip(word))
}

/// Python: bot/memory/portions.py:353 `_the_count_they_gave(text)`
///
/// The count the person answered the ask with, as (count, the word they used), or (0, "").
pub fn _the_count_they_gave(text: &str) -> (i64, String) {
    if let Some(found) = A_COUNT_OF_PEOPLE.search(text) {
        let word = found.name("n").filter(|w| !w.is_empty()).or(found.name("m")).unwrap_or("");
        let count = _as_a_count(word);
        if count != 0 && count <= 500 {
            return (count, word.to_string());
        }
    }
    for found in _A_NUMBER_ANYWHERE.finditer(text) {
        let word = found.as_str();
        let count = _as_a_count(word);
        if count == 0 || count > 500 {
            continue;
        }
        if _is_a_digit(word) && !A_CONFIRMATION.is_match(text) {
            continue;
        }
        return (count, word.to_string());
    }
    (0, String::new())
}

/// Python: bot/memory/portions.py:379 `_the_said_total(grams, count, count_word, old_word=None)`
///
/// The total and the per-head figure, with the number corrected said back.
pub fn _the_said_total(grams: f64, count: i64, count_word: &str, old_word: Option<&str>) -> String {
    let total = _a_total(grams * count as f64);
    match old_word.filter(|w| !w.is_empty()) {
        Some(old) => format!("Not {} - {}: about {total} in total, at {} a head.", _the_number(old), _the_number(count_word), _a_total(grams)),
        None => format!("About {total} in total for {} - that is {} a head.", _the_number(count_word), _a_total(grams)),
    }
}

/// `A_BARE_COUNT.search(text) or _the_new_count(text)`, kept when it is a count of people.
fn a_bare_count(text: &str) -> Option<Count> {
    let found = A_BARE_COUNT.search(text).map(|m| Count::of(&m)).or_else(|| _the_new_count(text))?;
    if _is_an_amount_not_a_count(text, &found) || _says_two_counts(text, &found) {
        return None;
    }
    Some(found)
}

/// The per-head figure of the answer in hand, when the answer weighs one person.
fn the_answers_head(memory: &Memory) -> f64 {
    let said = _the_last_answer(memory);
    if said.is_empty() || !PER_PERSON.finditer(&said).iter().any(|found| _counts_people(&said, found)) {
        return 0.0;
    }
    _the_per_head_figure(&said)
}

/// Python: bot/memory/portions.py:392 `_a_total_for_a_bare_count(text, memory)`
///
/// The same sum, when the line gives the number and the answer in hand gave the rest.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the line or None.
pub fn _a_total_for_a_bare_count(text: &str, memory: &Memory) -> Option<String> {
    let found = a_bare_count(text)?;
    let grams = the_answers_head(memory);
    if grams == 0.0 {
        return None;
    }
    let count = _as_a_count(&found.n);
    if count == 0 || count > 500 {
        return None;
    }
    Some(_the_said_total(grams, count, &found.n, found.old.as_deref()))
}

/// Python: bot/memory/portions.py:417 `a_bare_count_to_fill(text, memory)`
///
/// True when the line is a bare count answering the per-head figure the answer in hand gave.
pub fn a_bare_count_to_fill(text: &str, memory: &Memory) -> bool {
    let Some(found) = a_bare_count(text) else {
        return false;
    };
    if the_answers_head(memory) == 0.0 {
        return false;
    }
    let count = _as_a_count(&found.n);
    count != 0 && count <= 500
}

/// `memory["portion"]` when it holds a figure.
fn the_portion(memory: &Memory) -> Option<&Value> {
    memory.get("portion").filter(|p| p.is_dict() && mem7::get(p, "grams").truthy())
}

/// Python: bot/memory/portions.py:433 `a_count_to_fill(text, memory)`
///
/// True when this line answers the question the reply before it asked about a portion.
pub fn a_count_to_fill(text: &str, memory: &Memory) -> bool {
    let Some(portion) = the_portion(memory) else {
        return false;
    };
    if history::turns_before(memory).len() as i64 - mem7::int_or_0(portion.get("turn")) > 1 {
        return false;
    }
    _the_count_they_gave(text).0 != 0
}

/// Python: bot/memory/portions.py:445 `the_total_for(text, memory)`
///
/// The total a count of people makes of the figure the answer gave, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the line or None.
pub fn the_total_for(text: &str, memory: &mut Memory) -> Option<String> {
    let Some(portion) = the_portion(memory).cloned() else {
        return _a_total_for_a_bare_count(text, memory);
    };
    // Only the turn directly after the ask.
    if history::turns_before(memory).len() as i64 - mem7::int_or_0(portion.get("turn")) > 1 {
        memory.remove("portion");
        return None;
    }
    let (count, said) = _the_count_they_gave(text);
    if count == 0 {
        return None;
    }
    memory.remove("portion");
    let corrected = _the_new_count(text);
    let grams = mem7::get(&portion, "grams").as_f64().expect("TypeError: float() of the grams");
    Some(_the_said_total(grams, count, &said, corrected.as_ref().and_then(|c| c.old.as_deref())))
}
