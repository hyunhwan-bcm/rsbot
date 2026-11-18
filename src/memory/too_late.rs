//! Port of `bot/memory/too_late.py`: "is it too late", "am i too old to get any good" after an
//! answer that says anybody can - the sentence that answers the doubt, handed back.

use super::mem8_py::{pick, py_str, rx, strs, PyVal};
use super::meant;
use crate::data::Value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

const M: &str = "bot.memory.too_late.";

rx!(DOUBT, ANYONE, GET_ANY_GOOD, SAYS_IT);
strs!(LEADS, WITHOUT_A_SUBJECT);
static CLAUSE: LazyLock<&'static Re> = LazyLock::new(|| crate::data::re("bot.memory.meant.CLAUSE"));
static HEDGE: LazyLock<&'static Re> = LazyLock::new(|| crate::data::re("bot.memory.repeat.HEDGE"));
static LOOK_BACK: LazyLock<&'static Value> = LazyLock::new(|| crate::data::get("bot.memory.meant.LOOK_BACK"));

/// Python: bot/memory/too_late.py:109 `_the_sentence(said)`
///
/// The sentence of the answer that says anybody can, cut down to its gist, or None.
pub fn _the_sentence(said: &str) -> Option<String> {
    for sentence in meant::_sentences(said).val().items() {
        let sentence = py_str(sentence);
        if text::strip(&sentence).ends_with('?') {
            continue;
        }
        if !ANYONE.is_match(&sentence) {
            continue;
        }
        let mut gist = py_str(&meant::_gist(&sentence).val());
        if !ANYONE.is_match(&gist) {
            // The sentence says it in a clause the gist cut away: keep both clauses.
            let two = _two_clauses(&sentence);
            if !two.is_empty() {
                gist = two;
            }
        }
        if text::split_ref(&gist).len() < 4 {
            continue;
        }
        return Some(gist);
    }
    None
}

/// Python: bot/memory/too_late.py:131 `_two_clauses(sentence)`
///
/// A sentence cut after its second clause, as a sentence, or "".
pub fn _two_clauses(sentence: &str) -> String {
    let cuts: Vec<usize> = CLAUSE.finditer(sentence).iter().map(|m| m.start()).collect();
    if cuts.len() >= 2 {
        return format!("{}.", text::strip_chars(&sentence[..cuts[1]], " ,;:-."));
    }
    if cuts.len() == 1 && text::split_ref(sentence).len() <= 40 {
        return format!("{}.", text::rstrip_chars(text::strip(sentence), " ,;:-."));
    }
    String::new()
}

/// Python: bot/memory/too_late.py:145 `_the_reply(sentence, memory, clean)`
///
/// The sentence handed back under a plain lead, or on its own when it says it already.
pub fn _the_reply(sentence: &str, memory: &Memory, clean: &str) -> String {
    let body = text::rstrip_chars(sentence, ".!?");
    let end = &sentence[body.len()..];
    let end = if end.is_empty() { "." } else { end };
    if SAYS_IT.is_match(body) {
        return format!("{}{}", text::upper_first(body), end);
    }
    let said = format!("{}{}", super::mem8_py::lower_first(body), end);
    pick(&LEADS, memory, clean).replace("{said}", &said)
}

/// Python: bot/memory/too_late.py:154 `answer_for(text, memory)`
///
/// The sentence that answers the doubt, or None when this is not that turn.
pub fn answer_for(text: &str, memory: &Memory) -> Option<String> {
    let clean = text::strip(text);
    if DOUBT.match_(clean).is_none() {
        return None;
    }
    let answers = meant::_answers(memory, &LOOK_BACK).val();
    let answers = answers.items();
    // A hedge is never answered from itself.
    if let Some(first) = answers.first() {
        if HEDGE.is_match(&py_str(&first.items()[0])) {
            return None;
        }
    }
    for pair in answers {
        let said = py_str(&pair.items()[0]);
        if HEDGE.is_match(&said) {
            continue;
        }
        if let Some(sentence) = _the_sentence(&said) {
            return Some(_the_reply(&sentence, memory, clean));
        }
    }
    if GET_ANY_GOOD.is_match(clean) {
        return Some(pick(&WITHOUT_A_SUBJECT, memory, clean));
    }
    None
}
