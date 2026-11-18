//! Port of `bot/memory/returns.py`: a line that takes the chat back to something said earlier,
//! and asks about it.
//!
//! "back to lisbon, what should i see" reads as two parts: the opener is the return, the thing
//! after it is what the question is about, and the ask is the rest. The ask is put back together
//! with the thing in it, and only a version that reaches a real entry, names the thing, and is
//! not the entry the whole line already reached, is handed on.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::knowledge::lookup::{self, Entry};
use crate::memory::mem7::{self, inline_re, res, sets};
use crate::memory::{again as same_again, topics};
use crate::py::re;
use crate::py::text as pt;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

// ASK_OPENS is defined in PyBot and never read.
res!("returns"; OPENERS, STORE_FRAME, QUANTITY_ASK, A_NUMBER);
sets!("returns"; POINTER, NOTHING_NAMED);
inline_re! {
    ASK_AFTER_THE_THING = r"\s+(?=(?:what|which|where|when|who|why|how|should|can|could|do|does|is|are|will|would)\b)", re::I;
    MEASURE = r"^(how\s+(?:much|many))\s+(.+)$", re::I;
    A_HOW = r"^how\b", re::I;
    SENTENCE_END = r"(?<=[.!?])\s+", 0;
    AN_ASK_WORD = r"\b(?:what|which|where|when|who|why|how|should|can|could|do|does|did|is|are|will|would)\b", re::I;
}
static AGAIN: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.again.AGAIN"));
static DETAIL_AGAIN: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.again.DETAIL_AGAIN"));

/// `topics._entry(text)`: the knowledge entry a bare text reaches (the same call topics.rs makes;
/// its own copy is private to that file).
fn entry_of(text: &str) -> Option<&'static Entry> {
    let reading = Understood { clean: text.to_string(), words: topics::words(text), ..Default::default() };
    lookup::find(&reading, None, None, false)
}

fn same_entry(one: Option<&'static Entry>, other: Option<&'static Entry>) -> bool {
    matches!((one, other), (Some(a), Some(b)) if std::ptr::eq(a, b))
}

/// Python: bot/memory/returns.py:73 `_the_chat_has(words, memory)`
///
/// True when the chat has said one of these words, so the return points at something real.
pub fn _the_chat_has(words: &[String], memory: &Memory) -> bool {
    memory.turns.iter().any(|turn| {
        let said: HashSet<String> = topics::words(&turn.user).into_iter().chain(topics::words(&turn.bot)).collect();
        words.iter().any(|word| said.contains(word))
    })
}

/// The words of the thing that name it: `[w for w in topics.words(thing) if w not in POINTER]`,
/// or None when they name nothing.
fn the_named(thing: &str) -> Option<Vec<String>> {
    let named: Vec<String> = topics::words(thing).into_iter().filter(|word| !POINTER.contains(word)).collect();
    if named.is_empty() || named.iter().all(|word| NOTHING_NAMED.contains(word)) {
        return None;
    }
    Some(named)
}

/// Python: bot/memory/returns.py:86 `_the_parts(text)`
///
/// The thing the line goes back to (its naming words), and the ask after it, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the pair or None.
pub fn _the_parts(text: &str) -> Option<(Vec<String>, String)> {
    let unframed = STORE_FRAME.sub("", text);
    let m = OPENERS.match_(&unframed)?;
    let rest = pt::strip(m.name_str("rest"));
    if rest.is_empty() {
        return None;
    }
    let (thing, ask) = if rest.contains(',') {
        let (thing, _, ask) = pt::partition(rest, ",");
        (thing, ask)
    } else {
        let found = ASK_AFTER_THE_THING.search(rest)?;
        (&rest[..found.start()], &rest[found.start()..])
    };
    let thing = pt::strip_chars(thing, " ,.:;?!");
    let ask = pt::strip_chars(ask, " ,.:;?!");
    if thing.is_empty() || ask.is_empty() {
        return None;
    }
    if pt::strip(&pt::lower(m.name_str("opener"))) == "about" && !rest.contains(',') {
        // A bare "about" is only a return when the thing and the ask are split off from it.
        return None;
    }
    Some((the_named(thing)?, ask.to_string()))
}

/// Python: bot/memory/returns.py:118 `_the_lines(named, ask)`
///
/// The ask with the thing written into it, in the orders worth trying.
pub fn _the_lines(named: &[String], ask: &str) -> Vec<String> {
    let thing = named.join(" ");
    if let Some(measure) = MEASURE.match_(ask) {
        return vec![pt::strip(&format!("{} {} {}", measure.str(1), thing, measure.str(2))).to_string()];
    }
    let (first, _, after) = pt::partition(ask, " ");
    if matches!(pt::lower(first).as_str(), "what" | "which" | "whose") {
        return vec![pt::strip(&format!("{first} {thing} {after}")).to_string(), pt::strip(&format!("{ask} {thing}")).to_string()];
    }
    if A_HOW.is_match(ask) {
        return Vec::new();
    }
    vec![pt::strip(&format!("{ask} {thing}")).to_string()]
}

/// Python: bot/memory/returns.py:142 `_the_domain_of_the_chat(named, memory)`
///
/// The domain of the entry the chat's own line about this thing reached, or "".
pub fn _the_domain_of_the_chat(named: &[String], memory: &Memory) -> String {
    for turn in memory.turns.iter().rev() {
        let line = turn.user.as_str();
        let words = topics::words(line);
        if !named.iter().any(|word| words.contains(word)) {
            continue;
        }
        // A line that cannot be read tells us nothing.
        return match mem7::py_try(|| entry_of(line)) {
            Some(Some(item)) => item.domain.clone(),
            _ => String::new(),
        };
    }
    String::new()
}

/// `topics._names_it(line, item)`: PyBot hands it the line itself, and `set(found)` of a str is
/// its characters, so the characters are what is passed.
fn names_it(line: &str, item: &'static Entry) -> bool {
    let chars: Vec<String> = line.chars().map(|c| c.to_string()).collect();
    topics::_names_it(&chars, item)
}

/// Python: bot/memory/returns.py:166 `a_return_to_the_topic(text, memory)`
///
/// The ask with the thing the line goes back to written into it, or "".
pub fn a_return_to_the_topic(text: &str, memory: &Memory) -> String {
    let Some((named, ask)) = _the_parts(text) else {
        return String::new();
    };
    if !_the_chat_has(&named, memory) {
        return String::new();
    }
    let Some((whole, without)) = mem7::py_try(|| (entry_of(text), entry_of(&ask))) else {
        return String::new(); // a line the knowledge base cannot read is not answered
    };
    let domain = _the_domain_of_the_chat(&named, memory);
    for line in _the_lines(&named, &ask) {
        let Some(Some(item)) = mem7::py_try(|| entry_of(&line)) else {
            continue;
        };
        if same_entry(whole, Some(item)) {
            // The line already reached this entry, so the thing was never what it was about.
            continue;
        }
        if !names_it(&line, item) && same_entry(without, Some(item)) {
            // The thing was carried along and did no work.
            continue;
        }
        if !domain.is_empty() && item.domain != domain {
            // Another sense of the thing's word: a house you buy, and house music.
            continue;
        }
        return line;
    }
    String::new()
}

/// Python: bot/memory/returns.py:208 `the_thing_named(text)`
///
/// The thing the line goes back to, as the person typed it, or "".
pub fn the_thing_named(text: &str) -> String {
    let Some(m) = OPENERS.match_(text) else {
        return String::new();
    };
    let rest = pt::strip(m.name_str("rest"));
    if !rest.contains(',') {
        return String::new();
    }
    pt::strip_chars(pt::partition(rest, ",").0, " ,.:;?!").to_string()
}

/// Python: bot/memory/returns.py:232 `_the_sentence_with_the_amount(answer)`
///
/// The sentence of `answer` that gives the amount, or "" when none of them does.
pub fn _the_sentence_with_the_amount(answer: &str) -> String {
    for sentence in SENTENCE_END.split(answer) {
        if !pt::strip(&sentence).is_empty() && A_NUMBER.is_match(&sentence) {
            return pt::strip(&sentence).to_string();
        }
    }
    String::new()
}

/// Python: bot/memory/returns.py:240 `as_the_same_answer(reply, text, short=None)`
///
/// The answer framed as the same one, for a return that lands on what is already on screen.
pub fn as_the_same_answer(reply: &str, text: &str, short: &Value) -> String {
    let thing = Some(the_thing_named(text)).filter(|thing| !thing.is_empty()).unwrap_or_else(|| "that".into());
    let mut short = mem7::text_of(Some(short));
    let ask = _the_parts(text).map(|(_, ask)| ask).unwrap_or_default();
    if !ask.is_empty() && QUANTITY_ASK.match_(&ask).is_some() && !A_NUMBER.is_match(&short) {
        // A "how much" ask is not answered by a sentence without a number in it.
        let amount = _the_sentence_with_the_amount(reply);
        if !amount.is_empty() {
            short = amount;
        }
    }
    let base = if !short.is_empty() {
        short
    } else {
        same_again::unframed(reply).filter(|unframed| !unframed.is_empty()).unwrap_or_else(|| reply.to_string())
    };
    let short = same_again::_short(&base, None);
    // The lead-in may only name what the answer names.
    let named = same_again::the_name_the_answer_says(&short, &thing, reply);
    if named.is_empty() || pt::lower(&short).contains(&pt::lower(&named)) {
        return mem7::fmt(&AGAIN[0], &[("reply", &short)]);
    }
    mem7::fmt(&DETAIL_AGAIN[1], &[("detail", &named), ("reply", &short)])
}

/// Python: bot/memory/returns.py:283 `a_bare_return(text, memory=None)`
///
/// True when the line only takes the chat back to a thing and asks nothing about it.
///
/// `memory` None is PyBot's `memory=None` (no check that the chat has the thing).
pub fn a_bare_return(text: &str, memory: Option<&Memory>) -> bool {
    let unframed = STORE_FRAME.sub("", text);
    let Some(m) = OPENERS.match_(&unframed) else {
        return false;
    };
    let rest = pt::strip(m.name_str("rest"));
    if rest.is_empty() || rest.contains(',') || AN_ASK_WORD.is_match(rest) {
        return false;
    }
    let Some(named) = the_named(rest) else {
        return false;
    };
    memory.map_or(true, |memory| _the_chat_has(&named, memory))
}
