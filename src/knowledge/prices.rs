//! Port of `bot/knowledge/prices.py`: what everyday things cost, and whether a figure is a lot.

use crate::data;
use crate::data::FnRef;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{escape, Re};
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

static _JUDGING: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.prices._JUDGING"));
static _NOT_AN_ITEM: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.prices._NOT_AN_ITEM"));
static _NOT_A_PRICE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.prices._NOT_A_PRICE"));
static _A_JUDGEMENT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.prices._A_JUDGEMENT"));
static _A_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.prices._A_FIGURE"));
static _MONEY_WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.prices._MONEY_WORD"));
static _PRICE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.prices._PRICE"));

/// `_ITEMS`: each row's names (with the `\bname\b` pattern the reply builds for it) and reply.
static _ITEMS: LazyLock<Vec<(Vec<(String, Re)>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.prices._ITEMS")
        .items()
        .iter()
        .map(|row| {
            let row = row.items();
            let names = row[0]
                .strs()
                .into_iter()
                .map(|n| {
                    let rx = Re::new(&format!("\\b{}\\b", escape(&n)), 0);
                    (n, rx)
                })
                .collect();
            (names, row[1].str().to_string())
        })
        .collect()
});

/// True when the line brings a figure in money and asks whether it is a lot.
pub fn a_price_is_judged(text: &str) -> bool {
    let text = crate::py::text::squash(&text.to_lowercase());
    _JUDGING.is_match(&text)
}

/// The line as it was said, from the reading every entry is handed.
pub fn _the_text(reading: &Understood) -> String {
    let mut text = if reading.clean.is_empty() { reading.text.clone() } else { reading.clean.clone() };
    if text.is_empty() {
        text = reading.words.join(" ");
    }
    text.to_lowercase()
}

/// True when the line asks what an everyday thing costs, and nothing else about it.
pub fn _a_price_was_asked(reading: &Understood, _memory: Option<&Memory>) -> bool {
    let text = _the_text(reading);
    if text.is_empty() || _NOT_AN_ITEM.is_match(&text) || _NOT_A_PRICE.is_match(&text) {
        return false;
    }
    let judged = _A_JUDGEMENT.is_match(&text);
    let figured = _A_FIGURE.is_match(&text);
    if judged && !figured && !_MONEY_WORD.is_match(&text) {
        return false;
    }
    _PRICE.is_match(&text) || (judged && figured)
}

/// The price of the one item the line names, or the general line when it names several.
pub fn _the_item_reply(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let text = _the_text(reading);
    let mut said: Vec<&str> = Vec::new();
    let mut named: Vec<Vec<&str>> = Vec::new();
    for (names, reply) in _ITEMS.iter() {
        let hit: Vec<&str> = names.iter().filter(|(_, rx)| rx.is_match(&text)).map(|(n, _)| n.as_str()).collect();
        if !hit.is_empty() {
            named.push(hit);
            said.push(reply);
        }
    }
    if said.is_empty() {
        return None;
    }
    // max(hit, key=len): the first of the longest (by characters)
    let longest: Vec<&str> = named
        .iter()
        .map(|hit| {
            let mut best = hit[0];
            for h in &hit[1..] {
                if h.chars().count() > best.chars().count() {
                    best = h;
                }
            }
            best
        })
        .collect();
    let words: Vec<HashSet<&str>> = longest
        .iter()
        .map(|name| name.split_whitespace().filter(|w| !matches!(*w, "a" | "an" | "the")).collect())
        .collect();
    let said: Vec<&str> = said
        .iter()
        .zip(words.iter())
        .filter(|(_, mine)| !words.iter().any(|other| mine.len() < other.len() && mine.is_subset(other)))
        .map(|(r, _)| *r)
        .collect();
    if said.is_empty() {
        return None;
    }
    if said.len() > 1 {
        let n = said.len().min(3);
        return Some(format!("As a rough guide, in 2025: {}", said[..n].join(" ")));
    }
    Some(format!("{} Those are 2025 prices, and they are the sort of thing that moves.", said[0]))
}

pub const GUARDS: &[(&str, GuardFn)] =
    &[("bot.knowledge.prices:_a_price_was_asked:222", |_: &FnRef, u: &Understood| _a_price_was_asked(u, None))];

pub const REPLIES: &[(&str, ReplyFn)] = &[(
    "bot.knowledge.prices:_the_item_reply:234",
    |_: &FnRef, m: &mut Memory, u: &Understood| _the_item_reply(m, u),
)];
