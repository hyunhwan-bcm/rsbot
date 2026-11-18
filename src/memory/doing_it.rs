//! Port of `bot/memory/doing_it.py`: an ask about how long to do a thing, written out with the
//! thing the chat is about ("how long should i practise" -> "how long should i practise
//! guitar?"), and the how-to ask behind a moan ("i always get rice wrong").

use super::mem8_py::{int, mget, rx, strset, val};
use super::topics;
use crate::data::Value;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.doing_it.";

rx!(LENGTH_ASK, A_MOAN);
strset!(ASK_WORDS, TIME_WORDS, NOT_A_THING_DONE);
int!(LOOKBACK);
val!(MOAN_VERB);
static ACTION: LazyLock<HashSet<String>> = LazyLock::new(|| crate::data::set("bot.memory.topics.ACTION"));

/// Python: bot/memory/doing_it.py:51 `_is_doing(word)`
///
/// Whether this word is one of the things a person does, in the form it is typed in.
pub fn _is_doing(word: &str) -> bool {
    if NOT_A_THING_DONE.contains(word) || NOT_A_THING_DONE.contains(&topics::stem(word)) {
        return false;
    }
    if ACTION.contains(word) {
        return true;
    }
    for doing in ACTION.iter() {
        if text::len(doing) < 4 || text::len(word) <= text::len(doing) {
            continue;
        }
        if word.starts_with(doing.as_str()) {
            return true;
        }
        if doing.ends_with('e') && word.starts_with(&doing[..doing.len() - 1]) {
            return true; // "practising" is a practice, and "practic" is not a word of its own
        }
    }
    false
}

/// Python: bot/memory/doing_it.py:72 `doing_word(text)`
///
/// The doing word in this line, in the words it is typed in, or None.
pub fn doing_word(text: &str) -> Option<String> {
    topics::words(text).into_iter().find(|word| _is_doing(word))
}

/// Python: bot/memory/doing_it.py:80 `asks_about_doing(text)`
///
/// Whether this is a length ask about doing something, in the words it is typed in.
pub fn asks_about_doing(text: &str) -> bool {
    if !LENGTH_ASK.is_match(text) {
        return false;
    }
    doing_word(text).is_some()
}

/// Python: bot/memory/doing_it.py:88 `_the_things(memory)`
///
/// The things this chat has been about, newest first, as the words a line can be written on.
pub fn _the_things(memory: &Memory) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let records = mget(memory, "topics");
    let records: &[Value] = if records.truthy() { records.items() } else { &[] };
    let from = records.len().saturating_sub(*LOOKBACK as usize);
    for record in records[from..].iter().rev() {
        let Some(said) = record.get("text").and_then(Value::as_str) else { continue };
        if text::strip(said).is_empty() {
            continue;
        }
        if doing_word(said).is_none() {
            continue; // a state, not a thing the chat has been doing something with
        }
        for word in topics::keywords(said) {
            if text::len(&word) < 4 || TIME_WORDS.contains(&word) || _is_doing(&word) {
                continue;
            }
            if !out.contains(&word) {
                out.push(word);
            }
        }
    }
    out
}

/// Python: bot/memory/doing_it.py:113 `written_out(text, memory)`
///
/// The line written out with each thing the chat is about, newest thing first.
pub fn written_out(text: &str, memory: &Memory) -> Vec<String> {
    let squashed = text::squash(text);
    let said = text::rstrip_chars(&squashed, "?.! ");
    if !asks_about_doing(said) {
        return Vec::new();
    }
    if text::split_ref(&said.to_lowercase()).iter().any(|word| !ASK_WORDS.contains(*word) && !_is_doing(word)) {
        // the line names a thing itself, so it is not the bare ask this module writes out
        return Vec::new();
    }
    let mut out = Vec::new();
    for thing in _the_things(memory) {
        out.push(format!("{said} {thing}?"));
        out.push(format!("{said} {thing} each day?"));
    }
    out
}

/// Python: bot/memory/doing_it.py:156 `the_ask_behind_a_moan(text)`
///
/// The how-to ask behind "i always get rice wrong", or "" when the line is not that.
pub fn the_ask_behind_a_moan(text: &str) -> String {
    let said = text::squash(text);
    if said.is_empty() || text::split_ref(&said).len() > 12 {
        return String::new();
    }
    let Some(found) = A_MOAN.search(&said) else { return String::new() };
    let thing = text::strip_chars(found.name_str("thing"), " ,");
    let words = text::split_ref(thing).len();
    if words == 0 || words > 4 {
        return String::new();
    }
    if topics::keywords(thing).is_empty() {
        return String::new(); // a thing made of grammar only ("it", "that") names nothing to look up
    }
    let verb = found.name("verb").filter(|v| !v.is_empty()).unwrap_or("get").to_lowercase();
    let asked = MOAN_VERB.get(&verb).map(|v| v.str().to_string()).unwrap_or(verb);
    format!("how do i {asked} {thing} right?")
}
