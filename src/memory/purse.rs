//! Port of `bot/memory/purse.py`: "is 40 pounds enough" against the price bands the last answer
//! named ("£30 to £60 buys a sound second-hand one").

use super::mem8_py::rx;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Turn;
use std::sync::LazyLock;

const M: &str = "bot.memory.purse.";

rx!(_A_MONEY, _A_MONEY_BAND, _A_CLAUSE_END, _A_VERB_FIRST, _A_RATE_BAND);
static A_FIGURE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d+(?:,\d{3})*(?:\.\d+)?", 0));
static A_COUNT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d[\d,]*\.?\d*", 0));

/// Python `float(s)` for a figure `\d+(?:,\d{3})*(?:\.\d+)?` with its commas taken out (`\d` is
/// any Unicode decimal digit, which `float()` reads too).
fn py_float(s: &str) -> f64 {
    let ascii: String = s
        .chars()
        .map(|c| match super::mem8_py::decimal_value(c) {
            Some(d) if !c.is_ascii() => char::from_digit(d as u32, 10).unwrap(),
            _ => c,
        })
        .collect();
    ascii.parse().unwrap_or(0.0)
}

/// Python: bot/memory/purse.py:74 `_a_number(word)`
pub fn _a_number(word: &str) -> f64 {
    let word = text::strip(word).to_lowercase();
    if A_FIGURE.fullmatch(&word).is_some() {
        return py_float(&word.replace(',', ""));
    }
    0.0
}

/// Python: bot/memory/purse.py:82 `_money_in(text)`
///
/// The sums of money the line names, in order.
pub fn _money_in(text: &str) -> Vec<f64> {
    let mut found = Vec::new();
    for m in _A_MONEY.finditer(text) {
        let said = m.name("mark").filter(|s| !s.is_empty()).or_else(|| m.name("word")).unwrap_or("");
        let value = _a_number(said);
        if value != 0.0 {
            found.push(value);
        }
    }
    found
}

/// Python: bot/memory/purse.py:92 `_the_money(value)`
///
/// "£40", "£1,200", "£12.50".
pub fn _the_money(value: f64) -> String {
    if value.fract() == 0.0 {
        return format!("\u{a3}{}", text::thousands(value as i64));
    }
    format!("\u{a3}{value:.2}")
}

/// Python: bot/memory/purse.py:99 `_the_words_after(answer, at)`
///
/// The clause the answer wrote after a band, or "". (`at` is a byte offset here.)
pub fn _the_words_after(answer: &str, at: usize) -> String {
    let rest = &answer[at..];
    let clause = match _A_CLAUSE_END.search(rest) {
        Some(cut) => &rest[..cut.start()],
        None => rest,
    };
    let words = text::strip_chars(clause, " ,;:-");
    if words.is_empty() || text::split_ref(words).len() > 12 {
        return String::new();
    }
    // Another price in the clause is the next band, and a figure in it is a count, not a thing.
    if words.contains('\u{a3}') || text::split_ref(words).iter().any(|one| A_COUNT.fullmatch(one).is_some()) {
        return String::new();
    }
    words.to_string()
}

/// Python: bot/memory/purse.py:112 `_money_bands(answer)`
///
/// (low, high, what the band buys) for every price band the answer names, in order.
pub fn _money_bands(answer: &str) -> Vec<(f64, f64, String)> {
    let mut bands = Vec::new();
    for m in _A_MONEY_BAND.finditer(answer) {
        let first = m.name("a1").filter(|s| !s.is_empty()).or_else(|| m.name("b1")).unwrap_or("");
        let second = m.name("a2").filter(|s| !s.is_empty()).or_else(|| m.name("b2")).unwrap_or("");
        let (low, high) = (_a_number(first), _a_number(second));
        if low == 0.0 || high == 0.0 || low >= high {
            continue;
        }
        if _A_RATE_BAND.match_(&answer[m.end()..]).is_some() {
            continue;
        }
        bands.push((low, high, _the_words_after(answer, m.end())));
    }
    bands
}

/// Python: bot/memory/purse.py:127 `against_the_bands(text, turns)`
///
/// The person's money stood against the bands the last answer named, or None.
pub fn against_the_bands(text: &str, turns: &[Turn]) -> Option<String> {
    let mut theirs = _money_in(text);
    if theirs.is_empty() {
        for turn in turns[turns.len().saturating_sub(3)..].iter().rev() {
            theirs = _money_in(&turn.user);
            if !theirs.is_empty() {
                break;
            }
        }
    }
    let mine = *theirs.last()?;
    // The answer that priced the thing can be the one before the last; two turns back is as far
    // as the purse is read.
    let mut bands = Vec::new();
    for turn in turns[turns.len().saturating_sub(2)..].iter().rev() {
        bands = _money_bands(&turn.bot);
        if !bands.is_empty() {
            break;
        }
    }
    if bands.is_empty() {
        return None;
    }
    let spoken = _the_money(mine);
    if let Some((low, high, after)) = bands.iter().find(|b| b.0 <= mine && mine <= b.1) {
        let said = format!("{} to {}", _the_money(*low), _the_money(*high));
        if after.is_empty() {
            return Some(format!("{spoken} is inside the {said} that answer named."));
        }
        if _A_VERB_FIRST.match_(after).is_some() {
            return Some(format!("{spoken} is inside the {said} that answer named, and that {after}."));
        }
        return Some(format!("{spoken} is inside the {said} that answer named: {after}."));
    }
    // No band holds it: the cheapest one above their money is what they cannot reach.
    let mut above: Vec<&(f64, f64, String)> = bands.iter().filter(|b| b.0 > mine).collect();
    above.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let first = above.first()?;
    Some(format!("{spoken} is under the {} to {} that answer named.", _the_money(first.0), _the_money(first.1)))
}
