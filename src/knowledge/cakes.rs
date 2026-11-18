//! Port of `bot/knowledge/cakes.py`: the party cake (which tin for how many people) and how many
//! eggs go in a bake. The `cake-size` and `cake-eggs` rows that answer with these live in
//! `dishes`; their replies are registered here, under these functions' ids.

use crate::data::{self, Value};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

const P: &str = "bot.knowledge.cakes.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{P}{name}"))
}
fn s(name: &str) -> String {
    g(name).str().to_string()
}

/// One `_CAKE_SHAPES` row: (one tin, the tins, the fewest it serves, the most).
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    pub one: String,
    pub many: String,
    pub low: i64,
    pub high: i64,
}
static _CAKE_SHAPES: LazyLock<Vec<Shape>> = LazyLock::new(|| {
    g("_CAKE_SHAPES")
        .items()
        .iter()
        .map(|r| {
            let r = r.items();
            Shape { one: r[0].str().to_string(), many: r[1].str().to_string(), low: r[2].as_int().unwrap(), high: r[3].as_int().unwrap() }
        })
        .collect()
});
static _CAKE_COUNTS: LazyLock<Vec<(String, i64)>> =
    LazyLock::new(|| g("_CAKE_COUNTS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap())).collect());
static _CAKE_CLOCK: LazyLock<Vec<String>> = LazyLock::new(|| g("_CAKE_CLOCK").strs());
static _CAKE_NUMBERS: LazyLock<Vec<String>> = LazyLock::new(|| g("_CAKE_NUMBERS").strs());

/// A pattern built from text at run time, compiled once.
fn rx(pattern: String) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<String, &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(r) = cache.get(&pattern) {
        return r;
    }
    let r: &'static Re = Box::leak(Box::new(Re::new(&pattern, 0)));
    cache.insert(pattern, r);
    r
}

static COUNT_PATTERNS: LazyLock<Vec<Re>> = LazyLock::new(|| {
    [
        r"\b(\d{1,3})\s+(?:people|guests|kids|children|adults|persons|portions|slices|of us|head)\b",
        r"\bfor\s+(\d{1,3})\b",
        r"\b(?:feeds|feed|serves|serve)\s+(\d{1,3})\b",
        r"\bparty\s+of\s+(\d{1,3})\b",
    ]
    .iter()
    .map(|p| Re::new(p, 0))
    .collect()
});

/// How many people the cake in this line is for, or 0 when the line names no number.
pub fn _cake_count(text_: &str) -> i64 {
    let said = format!(" {} ", text::lower(text_));
    let mut found: Vec<i64> = Vec::new();
    for pattern in COUNT_PATTERNS.iter() {
        for m in pattern.finditer(&said) {
            let after = text::strip(&said[m.end()..]);
            let first = text::strip_chars(after.split(' ').next().unwrap_or(""), ",.");
            if _CAKE_CLOCK.iter().any(|w| w == first) {
                continue;
            }
            found.push(m.str(1).parse().expect("int()"));
        }
    }
    for (words, value) in _CAKE_COUNTS.iter() {
        if rx(format!(r"\b{words}\b")).is_match(&said) {
            found.push(*value);
        }
    }
    found.into_iter().max().unwrap_or(0)
}

/// Whether the line is one of these kinds of ask - by whole words, so "iced" is not "ice".
pub fn _cake_asks<S: AsRef<str>>(text_: &str, words: &[S]) -> bool {
    let said = format!(" {} ", text::lower(text_));
    words.iter().any(|word| {
        let word = word.as_ref();
        if word.contains(' ') { said.contains(word) } else { rx(format!(r"\b{word}\b")).is_match(&said) }
    })
}

static THIRTY_BY_TWENTY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b30\s*x\s*20\s*cm\b", 0));

/// The tin a line names on its own - "how many will a 25cm cake serve" - or None.
pub fn _cake_tin_size(text_: &str) -> Option<Shape> {
    let said = format!(" {} ", text::lower(text_).replace(" cm", "cm"));
    if THIRTY_BY_TWENTY.is_match(&said) {
        return Some(_CAKE_SHAPES[3].clone());
    }
    for shape in &_CAKE_SHAPES[..3] {
        let size = shape.one.split("cm").next().unwrap_or(""); // "20cm tin" -> "20"
        if rx(format!(r"\b{size}\s*cm\b")).is_match(&said) {
            return Some(shape.clone());
        }
    }
    None
}

/// The fewest tins that still feed this many people, and what they feed between them:
/// (score, counts of each tin, the fewest served, the most). Fewest tins first, then fewest
/// kinds of tin.
pub fn _cake_tins(count: i64) -> Option<((i64, i64, i64), [i64; 4], i64, i64)> {
    let mut best: Option<((i64, i64, i64), [i64; 4], i64, i64)> = None;
    for n20 in 0..9 {
        for n23 in 0..9 {
            for n25 in 0..9 {
                for ntr in 0..9 {
                    let counts = [n20, n23, n25, ntr];
                    let tins: i64 = counts.iter().sum();
                    if tins == 0 || tins > 12 {
                        continue;
                    }
                    let low: i64 = counts.iter().zip(_CAKE_SHAPES.iter()).map(|(n, s)| n * s.low).sum();
                    let high: i64 = counts.iter().zip(_CAKE_SHAPES.iter()).map(|(n, s)| n * s.high).sum();
                    if high < count {
                        continue; // it has to cover everyone at the top of a serve
                    }
                    let kinds = counts.iter().filter(|n| **n != 0).count() as i64;
                    let score = (tins, kinds, high - count);
                    if best.is_none_or(|b| score < b.0) {
                        best = Some((score, counts, low, high));
                    }
                }
            }
        }
    }
    best
}

/// `" ".join(parts).strip()`.
fn joined(parts: &[&str]) -> String {
    text::strip(&parts.join(" ")).to_string()
}

/// The party cake answer: the tin for the number in the line, or the table when it names none.
pub fn cake_plan(_memory: &Memory, understood: &Understood) -> Option<String> {
    let line = format!(" {} ", understood.clean);
    let count = _cake_count(&line);
    let (day_before, double) = (s("_CAKE_DAY_BEFORE"), s("_CAKE_DOUBLE"));
    if count == 0 {
        if let Some(shape) = _cake_tin_size(&line) {
            let said = if shape.one.contains("traybake") {
                format!("A {} cut into squares feeds {} to {}. ", shape.one, shape.low, shape.high)
            } else {
                format!("A {} two-layer sponge feeds {} to {}.", shape.one.replace(" tin", ""), shape.low, shape.high)
            };
            return Some(joined(&[&said, &day_before, &double]));
        }
        if _cake_asks(&line, &g("_CAKE_DOUBLE_WORDS").strs()) {
            return Some(text::strip(&double).to_string());
        }
        if _cake_asks(&line, &g("_CAKE_DECOR_WORDS").strs()) {
            return Some(text::strip(&s("_CAKE_DECOR")).to_string());
        }
        if _cake_asks(&line, &g("_CUPCAKE_WORDS").strs()) {
            return Some(s("_CUPCAKE_KEEP"));
        }
        if _cake_asks(&line, &g("_CAKE_SIZE_WORDS").strs()) {
            return Some(joined(&[&s("_CAKE_SIZE"), &day_before, &double]));
        }
        return Some(s("_CAKE_BASE"));
    }
    if _cake_asks(&line, &g("_CUPCAKE_WORDS").strs()) {
        let trays = -((-count * 2).div_euclid(24));
        let mut said = format!("{} cupcakes for {count} people: two a head, baked at 180C for 12 to 18 minutes and iced on the day.", count * 2);
        if trays > 1 {
            said += &format!(" That is {trays} trays of 24, and a muffin tray takes 12 at a time.");
        }
        return Some(said);
    }
    let mut plan: Option<([i64; 4], i64, i64)> = None;
    if count <= 20 {
        for (index, shape) in _CAKE_SHAPES[..3].iter().enumerate() {
            if count <= shape.high {
                let mut counts = [0; 4];
                counts[index] = 1;
                plan = Some((counts, shape.low, shape.high));
                break;
            }
        }
    }
    let (counts, low, high) = match plan {
        Some(plan) => plan,
        None => {
            let (_score, counts, low, high) = _cake_tins(count)?;
            (counts, low, high)
        }
    };
    let tins: i64 = counts.iter().sum();
    let number = |n: i64| _CAKE_NUMBERS.get(n as usize).cloned().unwrap_or_else(|| panic!("IndexError: tuple index out of range"));
    let parts: Vec<String> = counts
        .iter()
        .enumerate()
        .filter(|(_, n)| **n != 0)
        .map(|(i, n)| format!("{} {}", number(*n), if *n == 1 { &_CAKE_SHAPES[i].one } else { &_CAKE_SHAPES[i].many }))
        .collect();
    let said = if tins == 1 {
        let shape = parts[0].split_once(' ').map(|(_, s)| s).unwrap_or(""); // "one 25cm tin" -> "25cm tin"
        if shape.contains("traybake") {
            format!("One {shape} cut into squares feeds {low} to {high}, which covers {count}.")
        } else {
            format!("A {} two-layer sponge feeds {low} to {high}, so one tin covers {count}.", shape.replace(" tin", ""))
        }
    } else {
        let listed = if parts.len() > 1 { format!("{} and {}", parts[..parts.len() - 1].join(", "), parts[parts.len() - 1]) } else { parts[0].clone() };
        format!("{} {} {low} to {high} between them, which covers {count}.", text::upper_first(&listed), if tins > 1 { "feed" } else { "feeds" })
    };
    Some(joined(&[&said, &day_before, "Double every weight and split it between the tins with the same heat and time."]))
}

/// How many eggs go in a bake: the count is the weight of everything else.
pub fn cake_eggs(_memory: &Memory, understood: &Understood) -> String {
    let line = format!(" {} ", understood.clean);
    let asks = |words: &[&str]| _cake_asks(&line, words);
    if asks(&["needed", "necessary", "essential", "required", "have to", "can i skip", "skip the egg", "skip eggs", "do without"]) {
        return s("_EGG_SKIP");
    }
    if asks(&["yorkshire", "batter", "toad in the hole", "popover"]) {
        return s("_EGG_BATTER");
    }
    if asks(&["pancake", "pancakes", "crepe", "crepes", "drop scone"]) {
        return s("_EGG_PANCAKE");
    }
    if asks(&["meringue", "meringues", "pavlova"]) {
        return s("_EGG_MERINGUE");
    }
    s("_EGG_SPONGE")
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.cakes:cake_plan:147", |_, m, u| cake_plan(m, u)),
    ("bot.knowledge.cakes:cake_eggs:268", |_, m, u| Some(cake_eggs(m, u))),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Expected values from `pycall.py bot.knowledge.cakes _cake_tins` / `_cake_count`.
    #[test]
    fn tins_and_counts() {
        data::load();
        assert_eq!(_cake_tins(45).map(|b| (b.1, b.2, b.3)), Some(([0, 0, 0, 3], 45, 54)));
        assert_eq!(_cake_count("a cake for 30 people"), 30);
        assert_eq!(_cake_count("bake for 30 minutes for twelve"), 12);
    }
}
