//! Port of `bot/knowledge/animals.py`: facts about animals (how long they live, what they eat,
//! the word for a young one or a group, the biggest and fastest). The rows are data; `_fact` and
//! `_the_word` are factories whose replies read the fact or word they captured.

use crate::data::{FnRef, Value};
use crate::knowledge::d2dup_g10::{globals, re_cached};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self, Re};
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

globals!("bot.knowledge.animals", value { ANIMALS, WORD_ROWS, _WORD_SENTENCE, EATS, EATS_ITSELF, RECORDS });

/// A or an, from the letter the word starts with: "an unkindness", "a murder".
pub fn _article(word: &str) -> &'static str {
    let first = word.chars().next().map(|c| c.to_lowercase().to_string()).unwrap_or_default();
    if "aeiou".contains(first.as_str()) { "an" } else { "a" }
}

/// The repaired line, lower case, with a space at each end, for a whole-word search.
pub fn _line(understood: &Understood) -> String {
    let text = if understood.plain.is_empty() { &understood.clean } else { &understood.plain };
    format!(" {} ", text.to_lowercase())
}

fn names_of(row: &Value) -> Vec<String> {
    row.get("names").map(|n| n.strs()).unwrap_or_default()
}

/// The animal row the line names, or None: the longest matching name wins.
pub fn _named<'a>(understood: &Understood, rows: &'a Value) -> Option<&'a Value> {
    let text = _line(understood);
    let mut found = None;
    let mut best = 0usize;
    for row in rows.items() {
        for name in names_of(row) {
            if name.chars().count() > best && re_cached(&format!(r"\b{}\b", re::escape(&name)), 0).is_match(&text) {
                found = Some(row);
                best = name.chars().count();
            }
        }
    }
    found
}

/// `_the_word.<locals>.reply`: the word for this kind of name (cell "kind"), from the row the
/// line names, in one sentence; None when the row lacks it.
fn the_word_reply(f: &FnRef, _memory: &mut Memory, understood: &Understood) -> Option<String> {
    let kind = f.cell("kind").map(|v| v.str()).unwrap_or("");
    let row = _named(understood, &WORD_ROWS)?;
    let word = row.get(kind).filter(|w| w.truthy())?.str().to_string();
    let names = names_of(row);
    let many = if names.len() > 1 { names[1].clone() } else { format!("{}s", names[0]) };
    let sentence = _WORD_SENTENCE.get(kind).map(|v| v.str()).expect("KeyError: _WORD_SENTENCE");
    Some(
        sentence
            .replace("{one}", &names[0])
            .replace("{many}", &many)
            .replace("{article}", _article(&word))
            .replace("{word}", &word),
    )
}

/// `_fact.<locals>.reply`: the fact (cell "fact") from the animal row the line names.
fn fact_reply(f: &FnRef, _memory: &mut Memory, understood: &Understood) -> Option<String> {
    let fact = f.cell("fact").map(|v| v.str()).unwrap_or("");
    let row = _named(understood, &ANIMALS)?;
    row.get(fact).and_then(|v| v.as_str()).map(str::to_string)
}

/// The food for the animal the line names, or None when its row has no line about food.
pub fn _eats(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let row = _named(understood, &ANIMALS)?;
    let text = _line(understood);
    for name in names_of(row) {
        if let Some(fence) = EATS_ITSELF.get(&name) {
            if fence.truthy() && fence.strs().iter().any(|w| text.contains(w.as_str())) {
                // the line is about eating the animal, keeping it, or riding it
                return None;
            }
        }
        if let Some(food) = EATS.get(&name) {
            return food.as_str().map(str::to_string);
        }
    }
    None
}

static BIRD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bbird|birds\b", 0));
static SEA: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(fish|sea|ocean|water)\b", 0));
static LAND: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(land|on land)\b", 0));
static LIFE: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\b(longest lived|longest living|longest-lived|oldest|longest|lives the longest|live the longest)\b", 0)
});
static SPEED: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(fastest|quickest|speed)\b", 0));
static TALL: LazyLock<Re> = LazyLock::new(|| Re::new(r"\btall\b|\btallest\b", 0));
static STRONG: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(strongest|weakest)\b", 0));
static DANGER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(dangerous|deadliest|kill)\b", 0));

/// `RECORDS.get((what, where))`.
fn record(what: &str, place: &str) -> Option<&'static Value> {
    RECORDS.pairs().iter().find(|(k, _)| k.items().len() == 2 && k.items()[0].str() == what && k.items()[1].str() == place).map(|(_, v)| v)
}

/// The record the line asks about: what it is the most of, and where.
pub fn _record(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = _line(understood);
    let place = if BIRD.is_match(&text) {
        "bird"
    } else if SEA.is_match(&text) {
        "sea"
    } else if LAND.is_match(&text) {
        "land"
    } else {
        "any"
    };
    let what = if LIFE.is_match(&text) {
        "life"
    } else if SPEED.is_match(&text) {
        "speed"
    } else if TALL.is_match(&text) {
        "tall"
    } else if STRONG.is_match(&text) {
        if text.contains("strong") { "strong" } else { "weak" }
    } else if DANGER.is_match(&text) {
        "danger"
    } else {
        "big"
    };
    record(what, place)
        .filter(|v| v.truthy())
        .or_else(|| record(what, "any"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.animals:_the_word.<locals>.reply:847", the_word_reply),
    ("bot.knowledge.animals:_fact.<locals>.reply:861", fact_reply),
    ("bot.knowledge.animals:_eats:867", |_: &FnRef, m: &mut Memory, u: &Understood| _eats(m, u)),
    ("bot.knowledge.animals:_record:884", |_: &FnRef, m: &mut Memory, u: &Understood| _record(m, u)),
];
