//! Port of `bot/modules/thing_i_did_with.py`: "which coat was it i ordered": the line of theirs that says what they did with a thing, handed back.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

static DID_WITH: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.thing_i_did_with.DID_WITH"));
static WAS_IT_I: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.thing_i_did_with.WAS_IT_I"));
static ATTRIBUTE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.modules.thing_i_did_with.ATTRIBUTE"));
static NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.modules.thing_i_did_with.NOT_A_THING"));
static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// Python: bot/modules/thing_i_did_with.py `_thing(low)`
///
/// The thing the ask is about, or "" when the line is not this shape.
pub fn _thing(low: &str) -> String {
    for pattern in [&*WAS_IT_I, &*DID_WITH] {
        let Some(m) = pattern.match_(low) else { continue };
        let mut words = text::split(m.name_str("thing"));
        while words.len() > 1 && ATTRIBUTE.contains(&words[0]) {
            words.remove(0);
        }
        let thing = text::strip(&words.join(" ")).to_string();
        if thing.is_empty() || ATTRIBUTE.contains(&thing) || NOT_A_THING.contains(&thing) {
            return String::new();
        }
        return thing;
    }
    String::new()
}

/// Python: bot/modules/thing_i_did_with.py `_did(low)`
///
/// The doing in the ask: "what coat did i order" -> "order".
pub fn _did(low: &str) -> String {
    for pattern in [&*WAS_IT_I, &*DID_WITH] {
        if let Some(m) = pattern.match_(low) {
            return m.name_str("did").to_string();
        }
    }
    String::new()
}

/// Python: bot/modules/thing_i_did_with.py `_stem(word)`
///
/// The word with a plural or a participle taken off, so "coat" finds "coats".
pub fn _stem(word: &str) -> String {
    let lowered = word.to_lowercase();
    let word = text::strip_chars(&lowered, "'");
    for suffix in ["ing", "ed", "es", "s"] {
        if text::len(word) > suffix.len() + 3 && word.ends_with(suffix) {
            return word[..word.len() - suffix.len()].to_string();
        }
    }
    word.to_string()
}

/// Python: bot/modules/thing_i_did_with.py `_says_it(tokens, thing)`
///
/// True when every word of the thing is one of these tokens, in any of its forms.
pub fn _says_it(tokens: &[String], thing: &str) -> bool {
    let stems: HashSet<String> = tokens.iter().map(|token| _stem(token)).collect();
    text::split(thing).iter().all(|word| stems.contains(&_stem(word)))
}

/// Python: bot/modules/thing_i_did_with.py `_the_line(thing, did, memory)`
///
/// The line of theirs that names the thing, or "". A line that asks something is skipped. A line
/// with the doing in it wins over a later line that only names the thing; the newest line is the
/// fallback.
pub fn _the_line(thing: &str, did: &str, memory: &Memory) -> String {
    let mut lines: Vec<String> = Vec::new();
    for turn in memory.turns.iter().rev() {
        let line = text::strip(&turn.user).to_string();
        if line.is_empty() {
            continue;
        }
        if !_says_it(&WORDS.findall_str(&line.to_lowercase()), thing) {
            continue;
        }
        if line.ends_with('?') || !_thing(&fold(&line)).is_empty() {
            continue;
        }
        lines.push(line);
    }
    if lines.is_empty() {
        return String::new();
    }
    let doing = _stem(did);
    for line in &lines {
        if WORDS.findall_str(&line.to_lowercase()).iter().any(|token| _stem(token) == doing) {
            return line.clone();
        }
    }
    lines[0].clone()
}

/// Python: bot/modules/thing_i_did_with.py `_times_said(line, memory)`
///
/// How many of the last few replies already carry this line.
pub fn _times_said(line: &str, memory: &Memory) -> usize {
    let said = line.to_lowercase();
    recent(memory, 4).iter().filter(|turn| turn.bot.to_lowercase().contains(&said)).count()
}

/// Python: bot/modules/thing_i_did_with.py `_the_recall(text, memory)`
///
/// The reply for a line that asks after what they did with a thing, or None.
pub fn _the_recall(text: &str, memory: &Memory) -> Option<String> {
    let low = fold(text);
    if low.is_empty() {
        return None;
    }
    let thing = _thing(&low);
    if thing.is_empty() {
        return None;
    }
    let line = _the_line(&thing, &_did(&low), memory);
    if line.is_empty() {
        return None;
    }
    let quoted = format!("\"{line}\"");
    // Asking again on the next turn is the same line, not a new one, and the same sentence twice
    // in a row reads like a machine: the frame changes each time it is asked, so the reply is
    // never the one already on the screen.
    let times = _times_said(&line, memory);
    if times > 0 {
        return Some(format!("{}{}", ["Still the same one: ", "Same line again: "][times % 2], quoted));
    }
    Some(format!("You said: {quoted}"))
}

/// Python: bot/modules/thing_i_did_with.py `takes_as_it_comes(text, memory)` (also bound as
/// `TAKES_AS_IT_COMES`).
///
/// True on exactly the lines `_the_recall` answers, and nowhere else.
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> bool {
    _the_recall(text, memory).is_some()
}

/// Python: bot/modules/thing_i_did_with.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The thing they did with, when they told the chat about it, else None. Never a line the store
/// can already answer: the reader that already has it wins.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let folded = fold(text);
    let low = text::strip_chars(&folded, " ,.;:!?");
    if or_none(super::memory_store::respond(low, memory)).is_some() {
        return None;
    }
    _the_recall(text, memory)
}
