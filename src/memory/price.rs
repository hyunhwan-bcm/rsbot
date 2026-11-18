//! Port of `bot/memory/price.py`: a price the person gives ("the cheapest quote was 2400" / "is
//! that a lot"), read against the range the bot itself named a turn ago; and a price asked with
//! its thing ("is 6000 too much for a new boiler") read against the card that answers it.

use super::pyv;
use crate::data;
use crate::py::re::{self, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.price.";
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn pool(name: &str) -> Vec<String> {
    data::strs(&format!("{M}{name}"))
}

static A_PRICE_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("A_PRICE_ASK"));
static _PRICE_SYMBOL: LazyLock<&'static Re> = LazyLock::new(|| rx("_PRICE_SYMBOL"));
static _PRICE_WORD: LazyLock<&'static Re> = LazyLock::new(|| rx("_PRICE_WORD"));
static _AFTER_A_RATE: LazyLock<&'static Re> = LazyLock::new(|| rx("_AFTER_A_RATE"));
static _A_MONEY_CEILING: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_MONEY_CEILING"));
static _PRICE_RANGE: LazyLock<&'static Re> = LazyLock::new(|| rx("_PRICE_RANGE"));
static A_PRICE_SYMBOL_ONLY: LazyLock<&'static Re> = LazyLock::new(|| rx("A_PRICE_SYMBOL_ONLY"));
static A_PRICE_INSIDE: LazyLock<Vec<String>> = LazyLock::new(|| pool("A_PRICE_INSIDE"));
static A_PRICE_OVER: LazyLock<Vec<String>> = LazyLock::new(|| pool("A_PRICE_OVER"));
static A_PRICE_UNDER: LazyLock<Vec<String>> = LazyLock::new(|| pool("A_PRICE_UNDER"));
static A_PRICE_BETWEEN: LazyLock<Vec<String>> = LazyLock::new(|| pool("A_PRICE_BETWEEN"));
static A_PRICE_ASK_WITH_THE_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("A_PRICE_ASK_WITH_THE_THING"));
static _A_PRICE_FOR_THE_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_PRICE_FOR_THE_THING"));
static _ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.price._ASK_WORDS"));
static _A_MONEY_AFTER_THE_ASK: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.price._A_MONEY_AFTER_THE_ASK"));
static _A_PHRASE_AFTER_THE_ASK: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.price._A_PHRASE_AFTER_THE_ASK"));
static _A_RATE_IN_THE_SENTENCE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_RATE_IN_THE_SENTENCE"));
static A_JOB_JUDGED: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.prices.A_JOB_JUDGED"));

static A_SYMBOL_FIRST: LazyLock<Re> = LazyLock::new(|| Re::new(r"[£$€]", re::U));
static A_NUMBER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d[\d,]*(?:\.\d+)?", re::U));
static A_MONEY_UNIT_AFTER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s?(?:quid|pounds|euros?|dollars?|grand|k)\b", re::I | re::U));
static THOUSANDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s?(k|grand|thousand)\b", re::I | re::U));
static A_FIGURE_ON_ITS_OWN: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?:£|\$|€)?\s?\d[\d,]*(?:\.\d+)?(?:\s?(?:k|pounds|quid))?", re::U));
static SPACES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+", re::U));
static A_PERIOD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:a|an|per|each|every|this|last)\s+(day|week|month|year|hour|night)\b", re::I | re::U));
static OF_IN_FRONT: LazyLock<Re> = LazyLock::new(|| Re::new(r"^\s*of\b", re::I | re::U));
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", re::U));
static DIGITS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d[\d,.]*", re::U));
static FIRST_SENTENCE: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s", re::U));

/// The last `n` characters of `s` (`s[max(0, len - n):]`).
fn last_chars(s: &str, n: usize) -> &str {
    let count = s.chars().count();
    &s[text::byte_at(s, count.saturating_sub(n))..]
}

/// The first `n` characters of `s`.
fn first_chars(s: &str, n: usize) -> &str {
    &s[..text::byte_at(s, n)]
}

/// Python: bot/memory/price.py:91 `_money(value)`
/// The figure as it is said on the screen: 2400 in a range is 2,400.
pub fn _money(value: f64) -> String {
    let whole = value.fract() == 0.0;
    if value >= 1000.0 {
        if whole {
            return text::thousands(value as i64);
        }
        let fixed = format!("{value:.2}");
        let (int, frac) = fixed.split_once('.').unwrap();
        return format!("{}.{frac}", text::thousands(int.parse().unwrap()));
    }
    if whole { format!("{}", value.trunc() as i64) } else { format!("{value:.2}") }
}

/// Python: bot/memory/price.py:98 `_a_value(token)`
/// A figure as it was written: "2,400" and "2400" are the same number (0 when it is none).
pub fn _a_value(token: &str) -> f64 {
    text::strip(&token.replace(',', "")).parse::<f64>().unwrap_or(0.0)
}

/// Python: bot/memory/price.py:106 `_the_money_said_back(said, text)`
/// A figure keeps the symbol the line wrote; with none written it is the pound it was asked in.
pub fn _the_money_said_back(said: &str, text_: &str) -> String {
    let said = text::strip(said);
    if A_SYMBOL_FIRST.match_(said).is_some() {
        return said.to_string();
    }
    for mark in ["£", "$", "€"] {
        if text_.contains(mark) {
            return format!("{mark}{said}");
        }
    }
    format!("£{said}")
}

/// Python: bot/memory/price.py:117 `_the_price_said(value)`
/// "2400" said back the way it is written: "£2,400".
pub fn _the_price_said(value: f64) -> String {
    format!("£{}", _money(value))
}

/// Python: bot/memory/price.py:131 `_the_figure_in_the_range_s_style(mine, said, range_said)`
/// The person's figure written the way the range beside it is written.
pub fn _the_figure_in_the_range_s_style(mine: f64, said: &str, range_said: &str) -> String {
    let figure = if said.is_empty() { _money(mine) } else { said.to_string() };
    let figure = A_NUMBER.sub_fn(&figure, |one| _money(_a_value(one.as_str())));
    match A_PRICE_SYMBOL_ONLY.search(range_said) {
        Some(mark) if !A_PRICE_SYMBOL_ONLY.is_match(&figure) => format!("{}{figure}", mark.as_str()),
        _ => figure,
    }
}

/// Python: bot/memory/price.py:142 `_number_tokens(text)`
/// The figures in the line, with where each starts (a byte offset).
pub fn _number_tokens(text_: &str) -> Vec<(String, usize)> {
    A_NUMBER.finditer(text_).iter().map(|m| (m.as_str().to_string(), m.start())).collect()
}

/// Python: bot/memory/price.py:147 `_prices_in(text)`
/// [(value, spoken)] for the figures in the line that are marked as money.
pub fn _prices_in(text_: &str) -> Vec<(f64, String)> {
    let said = text_;
    let mut found = Vec::new();
    for (token, start) in _number_tokens(said) {
        let mut value = _a_value(&token);
        if value == 0.0 {
            continue;
        }
        let before = last_chars(&said[..start], 24);
        let after = first_chars(&said[start + token.len()..], 20);
        let marked = _PRICE_SYMBOL.is_match(&format!("{before}{token}{}", first_chars(after, 1)))
            || _PRICE_WORD.is_match(before)
            || A_MONEY_UNIT_AFTER.match_(after).is_some();
        if !marked {
            continue;
        }
        if _AFTER_A_RATE.match_(after).is_some() {
            continue;
        }
        let mut spoken = text::strip_chars(&token, " ,").to_string();
        if let Some(thousands) = THOUSANDS.match_(after) {
            value *= 1000.0;
            let end = start + token.len() + thousands.as_str().len();
            spoken = text::strip_chars(&said[start..end.min(said.len())], " ,").to_string();
        }
        found.push((value, spoken));
    }
    found
}

/// Python: bot/memory/price.py:175 `_a_price_at_a_rate(text)`
/// Whether the only figures in the line come with a period on them ("£200 to £300 a day").
pub fn _a_price_at_a_rate(text_: &str) -> bool {
    let figures = A_FIGURE_ON_ITS_OWN.finditer(text_);
    if figures.is_empty() {
        return false;
    }
    figures.iter().all(|m| _AFTER_A_RATE.match_(first_chars(&text_[m.end()..], 20)).is_some())
}

/// A money range in an answer: where it is (byte offsets), its low and high, and how it was said.
#[derive(Clone, Debug, PartialEq)]
pub struct PriceRange {
    pub start: usize,
    pub end: usize,
    pub low: f64,
    pub high: f64,
    pub spoken: String,
}

/// Python: bot/memory/price.py:188 `_price_ranges_in(text, period=None)`
/// The money ranges in the answer; a range with a period on it only when the person's figure
/// was said in the same period.
pub fn _price_ranges_in(text_: &str, period: Option<&str>) -> Vec<PriceRange> {
    let said = text_;
    let mut found = Vec::new();
    for one in _PRICE_RANGE.finditer(said) {
        let (mut low, mut high) = (_a_value(one.name_str("low_n")), _a_value(one.name_str("high_n")));
        if low == 0.0 || high == 0.0 || high <= low {
            continue;
        }
        let tail = first_chars(&said[one.end()..], 20);
        let head = last_chars(&said[..one.start()], 24);
        let unit = one.name_str("unit");
        let around = {
            let a = text::char_index(said, one.start()).saturating_sub(3);
            let b = text::char_index(said, one.end()) + 3;
            text::slice(said, a as isize, Some(b as isize))
        };
        let marked = one.name("low").is_some_and(|s| !s.is_empty())
            || one.name("high").is_some_and(|s| !s.is_empty())
            || (!unit.is_empty() && unit.to_lowercase() != "k")
            || _PRICE_SYMBOL.is_match(&around)
            || _PRICE_WORD.is_match(head)
            || _PRICE_WORD.is_match(tail);
        if !marked {
            continue;
        }
        if let Some(rate) = _AFTER_A_RATE.match_(tail) {
            if period.unwrap_or("") != rate.str(1).to_lowercase() {
                continue;
            }
        }
        if unit.to_lowercase() == "k" {
            low *= 1000.0;
            high *= 1000.0;
        }
        let spoken = SPACES.sub(" ", text::strip(one.as_str()));
        let spoken = text::strip_chars(&spoken, " ,.;:").to_string();
        if _A_MONEY_CEILING.is_match(head) {
            continue;
        }
        found.push(PriceRange { start: one.start(), end: one.end(), low, high, spoken });
    }
    found
}

/// The person's line and the three before it, newest first.
fn lines_back<'a>(text_: &'a str, memory: &'a Memory) -> Vec<&'a str> {
    let mut lines = vec![text_];
    lines.extend(memory.turns.iter().rev().take(3).map(|t| t.user.as_str()));
    lines
}

/// Python: bot/memory/price.py:236 `_the_price_they_gave(text, memory)`
/// (value, spoken) for the price the person gave, from this line or one of the last three.
pub fn _the_price_they_gave(text_: &str, memory: &Memory) -> (Option<f64>, String) {
    for line in lines_back(text_, memory) {
        let marked = _prices_in(line);
        if let Some((value, spoken)) = marked.last() {
            return (Some(*value), spoken.clone());
        }
        let bare: Vec<(f64, String)> = _number_tokens(line)
            .into_iter()
            .map(|(token, _)| (_a_value(&token), text::strip_chars(&token, " ,").to_string()))
            .filter(|(value, _)| *value >= 40.0)
            .collect();
        if bare.is_empty() {
            continue;
        }
        if _PRICE_WORD.is_match(line) {
            let (value, spoken) = bare.last().unwrap().clone();
            return (Some(value), spoken);
        }
        return (None, String::new());
    }
    (None, String::new())
}

/// Python: bot/memory/price.py:258 `_a_bare_price_in_the_thread(text, memory)`
/// The price they gave with no unit on it, read only when no line marked one as money.
pub fn _a_bare_price_in_the_thread(text_: &str, memory: &Memory) -> (Option<f64>, String) {
    for line in lines_back(text_, memory) {
        if !_prices_in(line).is_empty() {
            continue;
        }
        if let Some(value) = _number_tokens(line).iter().map(|(token, _)| _a_value(token)).find(|v| *v >= 40.0) {
            return (Some(value), _the_price_said(value));
        }
    }
    (None, String::new())
}

/// Python: bot/memory/price.py:276 `_answers_in(turns)`
/// The answers the bot gave, newest first.
pub fn _answers_in(turns: &[Turn]) -> Vec<String> {
    turns.iter().rev().filter(|turn| !text::strip(&turn.bot).is_empty()).map(|turn| turn.bot.clone()).collect()
}

/// Python's ordering of the tuples these readers sort.
fn by_number(a: f64, b: f64) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

/// `min(items, key=key)` / `max(...)`: the first of the smallest / largest, as Python's are.
fn first_min<T>(items: &[T], key: impl Fn(&T) -> f64) -> &T {
    items.iter().fold(&items[0], |best, one| if key(one) < key(best) { one } else { best })
}
fn first_max<T>(items: &[T], key: impl Fn(&T) -> f64) -> &T {
    items.iter().fold(&items[0], |best, one| if key(one) > key(best) { one } else { best })
}

/// Python: bot/memory/price.py:287 `_the_range_that_is_about_it(answers, price, period=None)`
/// (low, high, spoken) for the range in the newest answer that is about this price, or None.
pub fn _the_range_that_is_about_it(answers: &[String], price: f64, period: Option<&str>) -> Option<(f64, f64, String)> {
    for said in answers.iter().take(2) {
        let found = _the_ranges_that_are_not_rates(said, &_price_ranges_in(said, period));
        if found.is_empty() {
            continue;
        }
        let bands: Vec<(f64, f64, String)> = found.iter().map(|r| (r.low, r.high, r.spoken.clone())).collect();
        if bands.iter().all(|(low, _, _)| price < *low) {
            // min(bands): the tuples compared whole.
            let best = bands.iter().fold(&bands[0], |best, one| {
                let order = by_number(one.0, best.0).then(by_number(one.1, best.1)).then(one.2.cmp(&best.2));
                if order == Ordering::Less { one } else { best }
            });
            return Some(best.clone());
        }
        if bands.iter().all(|(_, high, _)| price > *high) {
            return Some(first_max(&bands, |band| band.1).clone());
        }
        let mut scored: Vec<(f64, f64, f64, String)> = bands
            .iter()
            .map(|(low, high, spoken)| {
                let distance = if *low <= price && price <= *high {
                    0.0
                } else if price > *high {
                    price - high
                } else {
                    low - price
                };
                (distance, *low, *high, spoken.clone())
            })
            .collect();
        scored.sort_by(|a, b| by_number(a.0, b.0).then(by_number(a.1, b.1)).then(by_number(a.2, b.2)).then(a.3.cmp(&b.3)));
        let best = &scored[0];
        if price > best.2 && price > best.2 * 3.0 {
            continue;
        }
        if price < best.1 && price * 3.0 < best.1 {
            continue;
        }
        if scored.len() > 1 {
            let (near, second) = (scored[0].0, scored[1].0);
            if near != 0.0 && second < near * 2.0 {
                continue;
            }
        }
        return Some((best.1, best.2, best.3.clone()));
    }
    None
}

/// Python: bot/memory/price.py:375 `_the_between_verdict(mine, said, below, above, memory)`
/// The verdict for a figure that falls between two bands the card prices.
pub fn _the_between_verdict(mine: f64, said: &str, below: &str, above: &str, memory: &Memory) -> Option<String> {
    let price = _the_figure_in_the_range_s_style(mine, said, below);
    let line = pyv::pick(&A_PRICE_BETWEEN, memory, &format!("between{}", py_float(mine)));
    let upper = text::upper_first(&price);
    Some(pyv::format(&line, &[("price", &price), ("below", below), ("above", above), ("Price", &upper)]))
}

/// `str(x)` of a float.
fn py_float(x: f64) -> String {
    text::float_repr(x)
}

/// Python: bot/memory/price.py:388 `_the_verdict(mine, said, low, high, spoken, memory, own=False)`
/// The judgement said in the answer's own words: inside, above or below its range.
pub fn _the_verdict(mine: f64, said: &str, low: f64, high: f64, spoken: &str, memory: &Memory, own: bool) -> Option<String> {
    let price = _the_figure_in_the_range_s_style(mine, said, spoken);
    let (lines, told) = if low <= mine && mine <= high {
        (&*A_PRICE_INSIDE, "inside")
    } else if mine > high {
        (&*A_PRICE_OVER, "over")
    } else {
        (&*A_PRICE_UNDER, "under")
    };
    let mut lines = lines.clone();
    if own {
        // Nothing says "the answer there" about a card that was never shown.
        let stood: Vec<String> = lines.iter().filter(|line| !line.contains("the answer")).cloned().collect();
        if !stood.is_empty() {
            lines = stood;
        }
    }
    let line = pyv::pick(&lines, memory, &format!("{told}{}", py_float(mine)));
    let upper = text::upper_first(&price);
    Some(pyv::format(&line, &[("aim", spoken), ("price", &price), ("Price", &upper)]))
}

/// Python: bot/memory/price.py:440 `_the_thing_priced(text)`
/// The thing the line prices ("a haircut"), or "" when the line names no thing at all.
pub fn _the_thing_priced(text_: &str) -> String {
    let left = _PRICE_SYMBOL.sub(" ", text_);
    let left = DIGITS.sub(" ", &left).to_lowercase();
    LETTERS.findall_str(&left).into_iter().filter(|w| !_ASK_WORDS.contains(w)).collect::<Vec<_>>().join(" ")
}

/// Python: bot/memory/price.py:448 `_the_price_they_mean(text, memory)`
/// (value, spoken) for the figure the line is asking a judgement on, or (None, "").
pub fn _the_price_they_mean(text_: &str, memory: &Memory) -> (Option<f64>, String) {
    let (mine, said) = _the_price_they_gave(text_, memory);
    if mine.is_some_and(|m| m != 0.0) {
        return (mine, said);
    }
    _a_bare_price_in_the_thread(text_, memory)
}

/// Python: bot/memory/price.py:456 `_the_period_they_said(text, memory)`
/// The period the person put their own figure in - "a month", "a week" - or None.
pub fn _the_period_they_said(text_: &str, memory: &Memory) -> Option<String> {
    lines_back(text_, memory).into_iter().find_map(|line| A_PERIOD.search(line).map(|found| found.str(1).to_lowercase()))
}

/// Python: bot/memory/price.py:483 `_a_price_ask(text)`
/// Whether the ask is about a figure (None when the words after it name another quantity).
pub fn _a_price_ask(text_: &str) -> bool {
    let Some(found) = A_PRICE_ASK.search(text_) else { return false };
    let tail = OF_IN_FRONT.sub("", &text_[found.end()..]);
    let words = LETTERS.findall_str(&tail.to_lowercase());
    let Some(first) = words.first() else { return true };
    if found.as_str().to_lowercase().ends_with("a lot") && first == "of" {
        return words.len() > 1 && _A_MONEY_AFTER_THE_ASK.contains(&words[1]);
    }
    _A_MONEY_AFTER_THE_ASK.contains(first) || _A_PHRASE_AFTER_THE_ASK.contains(first)
}

/// Python: bot/memory/price.py:500 `_a_rate_is_all_the_answer_names(answers)`
/// Whether the newest answer prices only a day or an hour of the work, and never the job.
pub fn _a_rate_is_all_the_answer_names(answers: &[String]) -> bool {
    answers.iter().take(1).any(|said| {
        let found = _price_ranges_in(said, None);
        !found.is_empty() && _the_ranges_that_are_not_rates(said, &found).is_empty()
    })
}

/// Python: bot/memory/price.py:508 `_a_second_quote_is_the_test(memory)`
/// What settles a figure for a whole job: the same job priced a second time.
pub fn _a_second_quote_is_the_test(memory: &Memory) -> String {
    pyv::pick(&A_JOB_JUDGED, memory, "jobjudged")
}

/// Python: bot/memory/price.py:514 `judged(text, memory)`
/// A line asking whether a figure they gave is high or low, from the answer already given.
pub fn judged(text_: &str, memory: &Memory) -> Option<String> {
    if !_a_price_ask(text_) {
        return None;
    }
    let answers = _answers_in(&memory.turns);
    if answers.is_empty() {
        return None;
    }
    let (mine, said) = _the_price_they_mean(text_, memory);
    let mine = mine.filter(|m| *m != 0.0)?;
    let said = _the_money_said_back(&said, text_);
    let period = _the_period_they_said(text_, memory);
    match _the_range_that_is_about_it(&answers, mine, period.as_deref()) {
        Some((low, high, spoken)) => _the_verdict(mine, &said, low, high, &spoken, memory, false),
        None => {
            if period.is_none() && _a_rate_is_all_the_answer_names(&answers) {
                return Some(_a_second_quote_is_the_test(memory));
            }
            None
        }
    }
}

/// Python: bot/memory/price.py:541 `judged_with_the_card(text, answer, memory)`
/// A price the line gives for a thing, judged against the range the answer for it names.
pub fn judged_with_the_card(text_: &str, answer: Option<&str>, memory: &Memory) -> Option<String> {
    let answer = answer?;
    let asked = A_PRICE_ASK_WITH_THE_THING.search(text_)?;
    if _the_thing_priced(text_).is_empty() {
        return None;
    }
    let figure = asked.name_str("figure");
    let tokens = _number_tokens(figure);
    let mine = if let Some((token, start)) = tokens.last() {
        let mine = _a_value(token);
        // Python reads the token's place in the figure as a place in the whole line.
        let at = text::char_index(figure, *start) as isize;
        let after = text::slice(text_, at, Some(at + 12));
        if mine != 0.0 && _AFTER_A_RATE.match_(&after).is_some() {
            return None;
        }
        mine
    } else {
        let marked = _prices_in(text_);
        let mine = marked.last().map(|(value, _)| *value).filter(|v| *v != 0.0)?;
        if !_A_PRICE_FOR_THE_THING.is_match(text_) {
            return None;
        }
        mine
    };
    if mine == 0.0 {
        return None;
    }
    let said = _the_price_said(mine);
    if let Some(verdict) = _the_verdict_against_a_card(mine, &said, answer, memory, true) {
        return Some(verdict);
    }
    let stripped = text::strip(answer);
    let opening = FIRST_SENTENCE.split(stripped).swap_remove(0);
    if opening != answer {
        if let Some(verdict) = _the_verdict_against_a_card(mine, &said, &opening, memory, false) {
            return Some(verdict);
        }
    }
    _the_verdict_against_a_card(mine, &said, answer, memory, false)
}

/// Python: bot/memory/price.py:607 `_the_ranges_that_are_not_rates(answer, ranges)`
/// The card's ranges that price the thing, leaving out a rate for a day or an hour of it.
pub fn _the_ranges_that_are_not_rates(answer: &str, ranges: &[PriceRange]) -> Vec<PriceRange> {
    let mut kept = Vec::new();
    for item in ranges {
        let (start, end) = (item.start, item.end);
        let left = [answer[..start].rfind('.'), answer[..start].rfind('\n')].into_iter().flatten().max();
        let after: Vec<usize> = [answer[end..].find('.'), answer[end..].find('\n')].into_iter().flatten().map(|at| at + end).collect();
        let right = after.iter().min().copied().unwrap_or(answer.len());
        let from = left.map(|at| at + 1).unwrap_or(0);
        let sentence = &answer[from..right.max(from)];
        if _A_RATE_IN_THE_SENTENCE.is_match(sentence) && _AFTER_A_RATE.match_(first_chars(&answer[end..], 20)).is_none() {
            continue;
        }
        kept.push(item.clone());
    }
    kept
}

/// Python: bot/memory/price.py:622 `_the_verdict_against_a_card(mine, said, answer, memory, definite_only=False)`
/// The verdict when the card's ranges say one thing about the figure, else None.
pub fn _the_verdict_against_a_card(mine: f64, said: &str, answer: &str, memory: &Memory, definite_only: bool) -> Option<String> {
    let ranges = _the_ranges_that_are_not_rates(answer, &_price_ranges_in(answer, None));
    if ranges.is_empty() {
        return None;
    }
    let verdict = |r: &PriceRange| _the_verdict(mine, said, r.low, r.high, &r.spoken, memory, true);
    if let Some(holds) = ranges.iter().find(|r| r.low <= mine && mine <= r.high) {
        return verdict(holds);
    }
    if ranges.iter().all(|r| r.low > mine) {
        return verdict(first_min(&ranges, |r| r.low));
    }
    if ranges.iter().all(|r| r.high < mine) {
        return verdict(first_max(&ranges, |r| r.high));
    }
    if definite_only {
        return None;
    }
    let under: Vec<PriceRange> = ranges.iter().filter(|r| r.high < mine).cloned().collect();
    let over: Vec<PriceRange> = ranges.iter().filter(|r| r.low > mine).cloned().collect();
    if !under.is_empty() && !over.is_empty() {
        let near = first_max(&under, |r| r.high);
        let far = first_min(&over, |r| r.low);
        if far.low - near.high <= near.high - near.low {
            return _the_between_verdict(mine, said, &near.spoken, &far.spoken, memory);
        }
        return verdict(near);
    }
    None
}
