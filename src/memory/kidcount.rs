//! Port of `bot/memory/kidcount.py`: the number of children, counted from the children the
//! chat already holds (each child relation named is one child, a child named on their own is
//! one, the same child said twice is still one).

use super::mem8_py::{py_iter, rx, s_or, strs, val};
use crate::data::Value;
use crate::py::random::Random;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;

const M: &str = "bot.memory.kidcount.";

strs!(CHILD_RELATIONS);
rx!(THE_ASK, THEIR_OWN, SOMEONE_ELSE, A_CHILD, NOT_THEIR_OWN);
val!(WORDS);

/// Python: bot/memory/kidcount.py:41 `_asks(text)`
///
/// Whether the line asks the person's own number of children.
pub fn _asks(text: &str) -> bool {
    let low = text::strip(text).to_lowercase();
    let Some(found) = THE_ASK.search(&low) else { return false };
    let rest = found.name_str("rest");
    if !A_CHILD.is_match(rest) {
        return false;
    }
    if SOMEONE_ELSE.is_match(&low) {
        return false;
    }
    if NOT_THEIR_OWN.is_match(&low) {
        return false;
    }
    // "how many kids" on its own is the ask said short; anything else has to be about them.
    THEIR_OWN.is_match(&low) || text::split_ref(rest).len() <= 2
}

/// `memory.get("user") or {}`: a dict, or a panic where Python's `.get` on it would raise.
fn user_dict(memory: &Memory) -> Value {
    match memory.get("user") {
        Some(v) if v.truthy() => {
            if !v.is_dict() {
                panic!("AttributeError: the user facts are not a dict");
            }
            v.clone()
        }
        _ => Value::dict(),
    }
}

/// `str(entry.get(key) or "").strip().lower()`.
fn field(entry: &Value, key: &str) -> String {
    text::strip(&s_or(entry.get(key))).to_lowercase()
}

/// Python: bot/memory/kidcount.py:58 `_named_children(memory)`
///
/// The children the store holds by name, each one once.
pub fn _named_children(memory: &Memory) -> HashSet<String> {
    let mut held = HashSet::new();
    for entry in py_iter(memory.get("others")) {
        if !entry.is_dict() {
            continue;
        }
        let name = field(&entry, "name");
        let who = field(&entry, "who");
        if name.is_empty() || held.contains(&name) {
            continue;
        }
        // A child who belongs to somebody else ("my sister's son") is not this person's child.
        let whose = match entry.get("of").filter(|v| v.truthy()) {
            Some(v) => s_or(Some(v)),
            None => s_or(entry.get("whose")),
        };
        if !text::strip(&whose).is_empty() {
            continue;
        }
        if CHILD_RELATIONS.contains(&who) || who.is_empty() {
            held.insert(name);
        }
    }
    held
}

/// Python: bot/memory/kidcount.py:76 `_relations(memory)`
///
/// The child relations the person named, each one once, in the order they said them.
pub fn _relations(memory: &Memory) -> Vec<String> {
    let mut held: Vec<String> = Vec::new();
    let user = user_dict(memory);
    for who in py_iter(user.get("people")) {
        let word = text::strip(&s_or(Some(&who))).to_lowercase();
        if CHILD_RELATIONS.contains(&word) && !held.contains(&word) {
            held.push(word);
        }
    }
    for entry in py_iter(memory.get("others")) {
        if !entry.is_dict() {
            continue;
        }
        let word = field(&entry, "who");
        if CHILD_RELATIONS.contains(&word) && !held.contains(&word) {
            held.push(word);
        }
    }
    held
}

/// Python: bot/memory/kidcount.py:92 `count(memory)`
///
/// How many children the chat holds, or None when it holds no number worth saying.
pub fn count(memory: &Memory) -> Option<usize> {
    let user = user_dict(memory);
    if user.get("kids").is_some_and(Value::truthy) {
        return None;
    }
    let named = _named_children(memory);
    let relations = _relations(memory);
    // The larger count is the true one: two sons called Jack and Sam are one relation and two
    // children.
    let held = named.len().max(relations.len());
    if held < 2 {
        return None;
    }
    Some(held)
}

/// Python: bot/memory/kidcount.py:112 `_vary(options, memory)`
///
/// A choice that does not repeat the last thing the bot said.
pub fn _vary(options: &[String], memory: &Memory) -> String {
    let turns = &memory.turns;
    let recent: HashSet<&str> = turns[turns.len().saturating_sub(5)..].iter().map(|t| t.bot.as_str()).collect();
    let mut fresh: Vec<&String> = options.iter().filter(|o| !recent.contains(o.as_str())).collect();
    if fresh.is_empty() {
        fresh = options.iter().collect();
    }
    Random::new(turns.len() as i128).choice(&fresh).to_string()
}

/// Python: bot/memory/kidcount.py:120 `reply_for(text, memory)`
///
/// The count said back, or None on every line that is not the ask for it.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    if !_asks(text) {
        return None;
    }
    let held = count(memory)?;
    let word = WORDS
        .pairs()
        .iter()
        .find(|(k, _)| k.as_int() == Some(held as i64))
        .map(|(_, v)| v.str().to_string())
        .unwrap_or_else(|| held.to_string());
    let noun = "children";
    let options = vec![
        format!("You've got {word} {noun}."),
        format!("{} {noun} \u{2014} that's what you've told me.", text::capitalize(&word)),
        format!("That makes {word} {noun} you've mentioned."),
    ];
    Some(_vary(&options, memory))
}
