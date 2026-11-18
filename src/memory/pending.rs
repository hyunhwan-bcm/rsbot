//! Port of `bot/memory/pending.py`: a thing not got yet for an occasion ("still havent got an
//! outfit for the wedding in 3 weeks") written out as the question it is ("what should i wear to
//! wedding in 3 weeks") and answered from the knowledge base.

use super::mem8_py::{strset, val};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.pending.";

val!(_NOT_GOT);
static _OCCASION: LazyLock<&'static Re> = LazyLock::new(|| crate::data::re("bot.memory.pending._OCCASION"));
static _OCCASION_HEAD: LazyLock<&'static Re> = LazyLock::new(|| crate::data::re("bot.memory.pending._OCCASION_HEAD"));
strset!(WORN, GIVEN, GENERIC, STOP);

static CLAUSE_SPLIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s*(?:,|;|\band\b|\bbut\b|\bso\b|\bbecause\b)\s*", 0));
static A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static ARTICLE: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:a|an|the|my|our|any|some|another)\s+", 0));
static THING_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+(?:for|to|in|at|on|from)\s+|\s+yet\b", 0));
static FOR_TO: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+(?:for|to)\s+", 0));
static CUT_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+(?:in|on|at|for|to|next|this|the|with|of|yet)$", 0));

/// Python: bot/memory/pending.py:76 `_clauses(text)`
pub fn _clauses(text: &str) -> Vec<String> {
    CLAUSE_SPLIT
        .split(&text.to_lowercase())
        .iter()
        .filter(|part| !text::strip(part).is_empty())
        .map(|part| text::strip_chars(part, " .!?,").to_string())
        .collect()
}

/// Python: bot/memory/pending.py:85 `_content(text)`
///
/// The words that carry meaning, for asking whether two phrases are the same thing.
pub fn _content(text: &str) -> HashSet<String> {
    A_WORD.findall_str(&text.to_lowercase()).into_iter().filter(|w| !STOP.contains(w)).collect()
}

/// Python: bot/memory/pending.py:89 `_what_is_missing(text)`
///
/// (the thing, the phrase it was said in), or None.
pub fn _what_is_missing(text: &str) -> Option<(String, String)> {
    for clause in _clauses(text) {
        for frame in _NOT_GOT.items() {
            let Some(found) = frame.as_re().expect("_NOT_GOT frame").search(&clause) else { continue };
            let phrase = text::strip(found.name_str("thing")).to_string();
            let thing = ARTICLE.subn("", &phrase, 1);
            // The thing ends where the preposition for the occasion starts.
            let thing = text::strip(&THING_END.split(&thing)[0]).to_string();
            if !thing.is_empty() {
                return Some((thing, phrase));
            }
        }
    }
    None
}

/// Python: bot/memory/pending.py:105 `_the_occasion(text, missing)`
///
/// The occasion the line names, said as it was said, or None.
pub fn _the_occasion(text: &str, missing: &str) -> Option<String> {
    let mut found_all: Vec<String> = Vec::new();
    for clause in _clauses(text) {
        // Every phrase the clause offers is tried.
        for found in _OCCASION.finditer(&clause) {
            found_all.push(found.name_str("occ").to_string());
        }
        if let Some(head) = _OCCASION_HEAD.match_(&clause) {
            found_all.push(head.name_str("occ").to_string());
        }
    }
    let missing_words = _content(missing);
    for occ in found_all {
        let mut occ = text::strip_chars(&occ, " -").to_string();
        // What follows the preposition is the occasion.
        let after = FOR_TO.split(&occ);
        if after.len() > 1 {
            occ = after[after.len() - 1].clone();
        }
        // A trailing preposition is where the phrase was cut, not part of the occasion.
        let occ = text::strip_chars(&CUT_END.sub("", &occ), " -").to_string();
        if occ.is_empty() || !_content(&occ).is_disjoint(&missing_words) {
            continue; // the thing itself, not the occasion it is for
        }
        if text::split_ref(&occ).len() > 7 {
            continue; // a whole sentence is not an occasion
        }
        return Some(if occ.to_lowercase().starts_with("a ") { text::slice(&occ, 2, None) } else { occ });
    }
    None
}

/// Python: bot/memory/pending.py:133 `as_question(text)`
///
/// The line written out as the question it is, or None.
pub fn as_question(text: &str) -> Option<String> {
    if text::strip(text).is_empty() {
        return None;
    }
    let said = text::squash(&text.to_lowercase());
    if said.contains('?') {
        return None;
    }
    let (thing, phrase) = _what_is_missing(&said)?;
    let mut words = _content(&phrase);
    words.extend(_content(&thing));
    let occasion = _the_occasion(&said, &thing)?;
    if words.iter().any(|w| GIVEN.contains(w)) {
        return Some(format!("what should i get for {occasion}"));
    }
    if words.iter().any(|w| WORN.contains(w)) || words.contains("wear") {
        return Some(format!("what should i wear to {occasion}"));
    }
    if words.iter().any(|w| GENERIC.contains(w)) {
        return Some(format!("how do i plan {occasion}"));
    }
    None
}

/// Python: bot/memory/pending.py:157 `answer(text, memory=None)`
///
/// The knowledge base's answer to the question the line is, or None.
pub fn answer(text: &str, memory: &Memory) -> Option<String> {
    let question = as_question(text)?;
    // `except Exception: return None` around the reading and the lookup.
    let entry = crate::runtime::guarded(|| {
        let reading = crate::understanding::understand(&question, Some(memory));
        crate::knowledge::lookup::find(&reading, None, None, false)
    })??;
    let reply = crate::memory::plans::_the_reply(entry, &question, memory);
    if text::strip(&reply).is_empty() || crate::understanding::pyuni::is_lower(&text::slice(&reply, 0, Some(1))) {
        return None;
    }
    Some(text::strip(&reply).to_string())
}
