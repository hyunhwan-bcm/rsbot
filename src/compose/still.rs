//! Port of `bot/compose/still.py`: a bare nudge ("go on", "and?") after an answer that has said
//! all it has, answered with the honest end of it rather than a repeat.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_sets, bundle_strs, bundle_vals, inline_re, IsNone};
use crate::compose::{fallback, verdict};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::HashSet;
use std::sync::LazyLock;

bundle_sets!("bot.compose.still"; NUDGES, LEADING);
bundle_strs!("bot.compose.still"; SYMPATHY_OPEN, STILL_END, STILL_AFTER_A_LOSS, STILL_AFTER_A_HARD_DAY, SORROW_MODULES, HARD_MODULES);
bundle_vals!("bot.compose.still"; SHORT_BACK);
inline_re!(SENTENCE_BREAK, r"(?<=[.!?]) ");

static UNREADABLE: LazyLock<HashSet<String>> = LazyLock::new(|| crate::data::set("bot.compose.verdict.UNREADABLE"));

/// Python: bot/compose/still.py:92 `_flat(text)`
pub fn _flat(text_: &str) -> String {
    text::squash(&com4::straight(&text_.to_lowercase()))
}

/// Python: bot/compose/still.py:96 `_is_a_nudge(flat)`
///
/// True for a line that only says "go on", with any "ok"/"so" in front of it taken off.
pub fn _is_a_nudge(flat: &str) -> bool {
    if flat.is_empty() || text::split_ref(flat).len() > 4 {
        return false;
    }
    let mut words: Vec<&str> = text::split_ref(flat).into_iter().map(|w| text::strip_chars(w, ".,!?;:-")).collect();
    let whole = words.iter().filter(|w| !w.is_empty()).copied().collect::<Vec<_>>().join(" ");
    if NUDGES.contains(&whole) {
        return true; // "then what" is whole before any "then" is taken off the front
    }
    while words.len() > 1 && LEADING.contains(words[0]) {
        words.remove(0);
    }
    let rest = words.into_iter().filter(|w| !w.is_empty()).collect::<Vec<_>>().join(" ");
    NUDGES.contains(&rest)
}

/// Python: bot/compose/still.py:109 `_ours(said)`
///
/// True for a line this module wrote: the floor's end line is not the answer.
pub fn _ours(said: &str) -> bool {
    let low = said.to_lowercase();
    STILL_END.iter().any(|template| {
        let head = text::strip(template.split('{').next().unwrap_or("")).to_lowercase();
        !head.is_empty() && low.starts_with(&head)
    })
}

/// `_flat(said).startswith(SYMPATHY_OPEN)`.
fn opens_on_sympathy(said: &str) -> bool {
    text::starts_with_any(&_flat(said), &SYMPATHY_OPEN)
}

/// `fallback._hedged_recently(memory)` inside Python's `try`: a reading that fails stands down.
fn hedged(memory: &Memory) -> bool {
    com4::attempt(|| fallback::_hedged_recently(memory, 3)).unwrap_or(true)
}

/// Python: bot/compose/still.py:119 `_answer_behind(memory)`
///
/// Whether the last thing the bot said gave the person something to do, and what it was ("" for
/// nothing). The walk goes back over this module's own end lines, nods and floor lines.
pub fn _answer_behind(memory: &Memory) -> String {
    // A hedge is not an answer.
    if hedged(memory) {
        return String::new();
    }
    let walked: Vec<&Turn> = memory.turns.iter().rev().collect();
    for (index, turn) in walked.iter().enumerate() {
        let module = turn.module.as_deref();
        if module.is_some_and(|m| UNREADABLE.contains(m)) {
            return String::new();
        }
        // The floor's own line is not an answer.
        let said = &turn.bot;
        if text::strip(said).is_empty() || _ours(said) {
            continue;
        }
        if module == Some("greeting") {
            // A floor line that ends on a question is the composer asking the person back.
            if text::strip(said).ends_with('?') {
                return String::new();
            }
            if !_continues_an_answer(&walked[index + 1..]) {
                return String::new();
            }
            continue;
        }
        // An answer that ends by asking the person something has not finished.
        if text::strip(said).ends_with('?') || opens_on_sympathy(said) {
            return String::new();
        }
        // The other half of an answer handed back is an answer.
        if module == Some("memory_store") && _a_hand_back(said) {
            return said.clone();
        }
        // An answer that gave something to do, or a list of things: finished with "that's the
        // lot of it". A one-line nod is neither.
        let steps = verdict::_steps(said).truthy();
        return if steps || SENTENCE_BREAK.split(text::strip(said)).len() > 1 { said.clone() } else { String::new() };
    }
    String::new()
}

/// Python: bot/compose/still.py:188 `_continues_an_answer(behind)`
///
/// True when one of the turns behind this floor line is an answer with something in it.
pub fn _continues_an_answer(behind: &[&Turn]) -> bool {
    behind.iter().any(|turn| {
        let module = turn.module.as_deref();
        if module == Some("greeting") || module.is_some_and(|m| UNREADABLE.contains(m)) {
            return false;
        }
        text::split_ref(&turn.bot).len() >= 5
    })
}

/// Python: bot/compose/still.py:204 `_a_hand_back(said)`
///
/// True when the memory store's line is part of an answer rather than a nod of its own: its first
/// sentence is six words or more.
pub fn _a_hand_back(said: &str) -> bool {
    let sentence = crate::data::re("bot.memory.repeat.SENTENCE");
    let parts: Vec<String> = sentence.split(text::strip(said)).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    !parts.is_empty() && text::split_ref(&parts[0]).len() >= 6
}

/// Python: bot/compose/still.py:222 `_sorrow_behind(memory)`
///
/// Whether the last thing the bot said was a condolence, and which kind: "loss", "hard" or "".
pub fn _sorrow_behind(memory: &Memory) -> String {
    if memory.turns.is_empty() || hedged(memory) {
        return String::new();
    }
    let last = memory.turns.last().unwrap();
    let module = last.module.as_deref().unwrap_or("");
    if last.module.is_some() && HARD_MODULES.iter().any(|m| m == module) {
        return String::new();
    }
    if last.module.is_some() && SORROW_MODULES.iter().any(|m| m == module) {
        return "loss".to_string();
    }
    if opens_on_sympathy(&last.bot) {
        return "hard".to_string();
    }
    String::new()
}

/// Python: bot/compose/still.py:251 `_the_rest_is_anothers(clean, memory=None)`
///
/// True when `bot/memory/another.py` has the rest of the answer for this line (it is the better
/// answer). A reading that fails leaves the line here.
pub fn _the_rest_is_anothers(clean: &str, memory: &Memory) -> bool {
    com4::attempt(|| !crate::memory::another::other_option(clean, memory).is_py_none()).unwrap_or(false)
}

/// Python: bot/compose/still.py:271 `takes_as_it_comes(text, memory)`
///
/// Only on a nudge with an answer behind it: the line is read whole, and nowhere else.
pub fn takes_as_it_comes(text_: &str, memory: &Memory) -> bool {
    com4::attempt(|| {
        if _the_rest_is_anothers(text_, memory) {
            return false;
        }
        _is_a_nudge(&_flat(text_)) && (!_answer_behind(memory).is_empty() || !_sorrow_behind(memory).is_empty())
    })
    .unwrap_or(false)
}

/// Python: bot/compose/still.py:281 `reply_for(clean, memory)`
///
/// The honest end of the answer for a nudge that names nothing of its own. Else None.
pub fn reply_for(clean: &str, memory: &Memory) -> Option<String> {
    com4::attempt(|| {
        if !takes_as_it_comes(clean, memory) {
            return None;
        }
        let pool: &Vec<String> = match _sorrow_behind(memory).as_str() {
            "loss" => &STILL_AFTER_A_LOSS,
            "hard" => &STILL_AFTER_A_HARD_DAY,
            _ => &STILL_END,
        };
        Some(verdict::_say(pool, memory, &_flat(clean)))
    })
    .flatten()
}

#[allow(dead_code)]
fn _short_back() -> i64 {
    SHORT_BACK.as_int().unwrap_or(0)
}
