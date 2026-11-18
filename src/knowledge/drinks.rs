//! Port of `bot/knowledge/drinks.py`: which bottle goes with the food that is named, and the
//! drink itself - what it is, what it tastes like, how strong it is and what goes with it.
//!
//! The tables and the entries are the bundle's; this file holds the readers the entries call.

use crate::data::{self, Value};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

const P: &str = "bot.knowledge.drinks.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{P}{name}"))
}

/// `_PAIRS`: (foods, the bottle for them).
static _PAIRS: LazyLock<Vec<(Vec<String>, String)>> =
    LazyLock::new(|| g("_PAIRS").items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect());
static _MEALS: LazyLock<Vec<String>> = LazyLock::new(|| g("_MEALS").strs());
static _WEIGHT: LazyLock<String> = LazyLock::new(|| g("_WEIGHT").str().to_string());
static _NOT_A_TASTE: LazyLock<Vec<String>> = LazyLock::new(|| g("_NOT_A_TASTE").strs());
static _MY_OWN_CUP: LazyLock<Vec<String>> = LazyLock::new(|| g("_MY_OWN_CUP").strs());

/// One `_DRINKS` row: (names, what it is, what it tastes like, what goes with it).
pub struct Drink {
    pub names: Vec<String>,
    pub itself: String,
    pub taste: String,
    pub goes_with: String,
}
static _DRINKS: LazyLock<Vec<Drink>> = LazyLock::new(|| {
    g("_DRINKS")
        .items()
        .iter()
        .map(|r| {
            let r = r.items();
            Drink { names: r[0].strs(), itself: r[1].str().to_string(), taste: r[2].str().to_string(), goes_with: r[3].str().to_string() }
        })
        .collect()
});
/// `_STRENGTH`: {drink: how strong it is, or None}.
static _STRENGTH: LazyLock<HashMap<String, Option<String>>> =
    LazyLock::new(|| g("_STRENGTH").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_str().map(str::to_string))).collect());
/// `_DIFFERENCES`: {(drinks): what the difference is}, in the dict's order.
static _DIFFERENCES: LazyLock<Vec<(Vec<String>, String)>> =
    LazyLock::new(|| g("_DIFFERENCES").pairs().iter().map(|(k, v)| (k.strs(), v.str().to_string())).collect());

static NOT_PLAIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' ]+", 0));

/// `re.search(r"(?<![a-z0-9])" + re.escape(word) + r"(?:s|es)?(?![a-z0-9])", ...)`, compiled once.
fn word_re(word: &str) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<String, &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(r) = cache.get(word) {
        return r;
    }
    let r: &'static Re = Box::leak(Box::new(Re::new(&format!("(?<![a-z0-9]){}(?:s|es)?(?![a-z0-9])", pyre::escape(word)), 0)));
    cache.insert(word.to_string(), r);
    r
}

/// Whether the line names one of these foods, as whole words.
pub fn _named(text_: &str, words: &[String]) -> bool {
    words.iter().any(|word| word_re(word).is_match(text_))
}

/// `clean or text`, lower-cased.
fn line(reading: &Understood) -> String {
    text::lower(if reading.clean.is_empty() { &reading.text } else { &reading.clean })
}

/// The bottle that goes with the food named, or the weight rule for a meal with no food; None
/// when the line names neither.
pub fn _wine_pair(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = line(understood);
    if said.is_empty() {
        return None;
    }
    for (foods, answer) in _PAIRS.iter() {
        if _named(&said, foods) {
            return Some(answer.clone());
        }
    }
    if _named(&said, &_MEALS) {
        return Some(_WEIGHT.clone());
    }
    None
}

/// How strong the drink named in the line is, when the line asks that rather than the taste.
pub fn _how_strong(_memory: &Memory, reading: &Understood) -> Option<String> {
    let row = _a_drink(reading)?;
    let said = line(reading);
    let strength = |name: &str| _STRENGTH.get(name).cloned().flatten().filter(|s| !s.is_empty());
    for name in &row.names {
        if said.contains(name.as_str()) {
            return strength(name).or_else(|| strength(&row.names[0]));
        }
    }
    None
}

/// The line padded and cut down to letters, figures, apostrophes and spaces.
fn plain_said(reading: &Understood) -> String {
    NOT_PLAIN.sub(" ", &format!(" {} ", line(reading)))
}

/// The drink named in the line, as its row, or None (the longest name wins).
pub fn _a_drink(reading: &Understood) -> Option<&'static Drink> {
    let said = plain_said(reading);
    let has = |phrase: &String| said.contains(&format!(" {phrase} "));
    if _NOT_A_TASTE.iter().any(has) || _MY_OWN_CUP.iter().any(has) {
        return None;
    }
    let mut best: Option<(usize, &'static Drink)> = None;
    for row in _DRINKS.iter() {
        for name in &row.names {
            if has(name) && best.is_none_or(|b| text::len(name) > b.0) {
                best = Some((text::len(name), row));
            }
        }
    }
    best.map(|b| b.1)
}

/// What the drink named in the line tastes like.
pub fn _the_drink_taste(_memory: &Memory, reading: &Understood) -> Option<String> {
    _a_drink(reading).map(|row| row.taste.clone())
}

/// What the drink is, for the line that asks what it is rather than what it tastes like.
pub fn _the_drink_itself(_memory: &Memory, reading: &Understood) -> Option<String> {
    let row = _a_drink(reading)?;
    let said = format!(" {} ", line(reading));
    if said.contains(" like ") || text::rstrip(&said).ends_with(" like") {
        return Some(row.taste.clone());
    }
    Some(row.itself.clone())
}

/// What to eat with it.
pub fn _what_it_goes_with(_memory: &Memory, reading: &Understood) -> Option<String> {
    _a_drink(reading).map(|row| row.goes_with.clone())
}

/// Two drinks named in one line: what the difference is.
pub fn _the_difference(_memory: &Memory, reading: &Understood) -> Option<String> {
    let said = plain_said(reading);
    let mut found: Vec<(usize, &String)> = Vec::new();
    for row in _DRINKS.iter() {
        for name in &row.names {
            if let Some(at) = said.find(&format!(" {name} ")) {
                found.push((at, &row.names[0]));
                break;
            }
        }
    }
    found.sort();
    if found.len() < 2 {
        return None;
    }
    let pair: HashSet<&str> = [found[0].1.as_str(), found[1].1.as_str()].into_iter().collect();
    for (names, reply) in _DIFFERENCES.iter() {
        if pair.iter().all(|p| names.iter().any(|n| n == p)) {
            return Some(reply.clone());
        }
    }
    // `{row[0][0]: row for row in _DRINKS}`: the last row of a name wins
    let row = |name: &str| _DRINKS.iter().rev().find(|row| row.names[0] == name);
    let (first, second) = (row(found[0].1)?, row(found[1].1)?);
    Some(format!("{} {}", first.itself, second.itself))
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.drinks:_wine_pair:85", |_, m, u| _wine_pair(m, u)),
    ("bot.knowledge.drinks:_how_strong:625", |_, m, u| _how_strong(m, u)),
    ("bot.knowledge.drinks:_the_drink_taste:706", |_, m, u| _the_drink_taste(m, u)),
    ("bot.knowledge.drinks:_the_drink_itself:714", |_, m, u| _the_drink_itself(m, u)),
    ("bot.knowledge.drinks:_what_it_goes_with:726", |_, m, u| _what_it_goes_with(m, u)),
    ("bot.knowledge.drinks:_the_difference:791", |_, m, u| _the_difference(m, u)),
];
