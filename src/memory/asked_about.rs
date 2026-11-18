//! Port of `bot/memory/asked_about.py`: the answer this chat already gave, when a later line
//! names the thing it was about.
//!
//! `topics.answer_about` finds the earlier card; the reply is the sentence of that card which
//! carries the words the line asked with, said as the return it is ("Back to sleep: ...").

use super::mem8_py::{int, py_str, rx, strs, strset, PyVal};
use super::topics;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;

const M: &str = "bot.memory.asked_about.";

rx!(SENTENCE, _A_CONTINUATION);
strs!(RETURN, RETURN_UNNAMED);
int!(SHORT, MOST);
strset!(_NOT_A_THING);
static ASK_ONLY: std::sync::LazyLock<HashSet<String>> =
    std::sync::LazyLock::new(|| crate::data::set("bot.memory.topics.ASK_ONLY"));

/// Python: bot/memory/asked_about.py:64 `_sentences(answer)`
pub fn _sentences(answer: &str) -> Vec<String> {
    SENTENCE
        .split(text::strip(answer))
        .iter()
        .map(|one| text::strip(one).to_string())
        .filter(|one| !one.is_empty())
        .collect()
}

/// Python: bot/memory/asked_about.py:79 `_the_words_asked(text)`
///
/// The content words of the line, without the asking: "how do i get some sleep" -> sleep.
pub fn _the_words_asked(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in topics::keywords(text) {
        if ASK_ONLY.contains(&word) || _NOT_A_THING.contains(&word) || out.contains(&word) {
            continue;
        }
        out.push(word);
    }
    out
}

/// Python: bot/memory/asked_about.py:97 `_the_part_that_answers(answer, wanted, said_before)`
///
/// The sentence of the card that carries the words the line asked with, and its neighbour.
pub fn _the_part_that_answers(answer: &str, wanted: &[String], said_before: &str) -> String {
    let parts = _sentences(answer);
    if parts.is_empty() {
        return String::new();
    }
    let wanted_set: HashSet<&str> = wanted.iter().map(String::as_str).collect();
    let mut scored: Vec<(usize, usize, usize)> = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        let here: HashSet<String> = topics::keywords(part).iter().map(|word| topics::stem(word)).collect();
        let score = here.iter().filter(|w| wanted_set.contains(w.as_str())).count();
        let stands = if _A_CONTINUATION.match_(part).is_some() { 0 } else { 1 };
        scored.push((score, stands, index));
    }
    // A sentence that stands on its own beats one that opens mid-thought with the same words in
    // it, so the return reads as an answer and not as the tail of one.
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)).then(a.2.cmp(&b.2)));
    let best = scored[0];
    let index = if best.0 > 0 && best.1 > 0 {
        best.2
    } else if best.0 > 0 {
        // The only sentences with the line's words in them open mid-thought: the card is still
        // about the thing asked, so its own first whole sentence is the answer.
        parts.iter().position(|part| _A_CONTINUATION.match_(part).is_none()).unwrap_or(best.2)
    } else {
        return String::new(); // the card says none of the words the line asked with
    };
    let mut out = parts[index].clone();
    if (text::split_ref(&out).len() as i64) < *SHORT && index + 1 < parts.len() {
        out = format!("{} {}", out, parts[index + 1]);
    }
    if text::split_ref(&out).len() as i64 > *MOST {
        let words = text::split_ref(&out);
        let cut = words[..*MOST as usize].join(" ");
        out = format!("{}.", text::rstrip_chars(&cut, " ,;:"));
    }
    // The bot may not say the same sentence twice in a row: if this is the one it has just said,
    // nothing is handed back - the hedge is better than the same way twice.
    let flat_before = text::squash(&said_before.to_lowercase());
    if !flat_before.is_empty() && flat_before.contains(&text::slice(&text::squash(&out.to_lowercase()), 0, Some(60))) {
        return String::new();
    }
    out
}

/// Python: bot/memory/asked_about.py:134 `the_answer_this_chat_gave(text, memory)`
///
/// The earlier answer of this chat about the thing the line names, as the return it is.
pub fn the_answer_this_chat_gave(text: &str, memory: &Memory) -> Option<String> {
    let wanted = _the_words_asked(text);
    if wanted.is_empty() {
        return None;
    }
    let answer = topics::answer_about(text, memory).val();
    if !answer.truthy() {
        return None;
    }
    let turns = &memory.turns;
    let said_before = turns.last().map(|t| t.bot.clone()).unwrap_or_default();
    let part = _the_part_that_answers(&py_str(&answer), &wanted, &said_before);
    if part.is_empty() {
        return None;
    }
    let said = text::squash(&part.to_lowercase());
    let just_said = turns.last().map(|t| text::squash(&t.bot.to_lowercase())).unwrap_or_default();
    let head = text::slice(&said, 0, Some(60));
    if !head.is_empty() && just_said.contains(&head) {
        return None; // the sentence just said is not a return, it is the same way twice
    }
    let frame = if wanted.len() == 1 {
        &RETURN[turns.len() % RETURN.len()]
    } else {
        &RETURN_UNNAMED[turns.len() % RETURN_UNNAMED.len()]
    };
    Some(frame.replace("{thing}", &wanted[0]).replace("{reply}", &part))
}
