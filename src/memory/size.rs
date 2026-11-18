//! Port of `bot/memory/size.py`: "what about a bigger one" after a cooking time - the same rule
//! at another size, or a few minutes more or less.

use super::mem8_py::{py_int, rx, strs};
use crate::py::re::{Re, I};
use crate::py::text;
use crate::state::Memory;
use crate::understanding::pyuni;
use std::sync::LazyLock;

const M: &str = "bot.memory.size.";

rx!(_ANOTHER_SIZE, _POINTS_BACK, _A_TIME, _RULE_STOPS, _A_RATE, _SENTENCE, _BY_WEIGHT, _SIZED_THING, _SIZED_ASKED);
strs!(BIGGER, SMALLER, SAID_BIGGER, SAID_SMALLER, MORE, LESS, _OUR_SHORT_LINES, _OUR_RULE_LEADS);
static SMALLER_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:small|smaller|little|littler|thinner|narrower)\b", I));

/// Python: bot/memory/size.py:75 `_flat(text)`
pub fn _flat(text: &str) -> String {
    text::squash(text.trim_end_matches('.')).to_lowercase()
}

/// Python: bot/memory/size.py:93 `_one_of_ours(answer)`
///
/// True when the line is one of this module's own shapes.
pub fn _one_of_ours(answer: &str) -> bool {
    let flat = _flat(answer);
    if _OUR_SHORT_LINES.contains(&flat) {
        return true;
    }
    text::starts_with_any(&flat, &_OUR_RULE_LEADS)
}

/// Python: bot/memory/size.py:123 `_the_thing_asked(memory, text)`
///
/// The line that the answer above answered, word for word, or "".
pub fn _the_thing_asked(memory: &Memory, text: &str) -> String {
    let here = _flat(text);
    for turn in memory.turns.iter().rev() {
        let asked = text::squash(&turn.user);
        if asked.is_empty() || _flat(&asked) == here {
            continue;
        }
        if _one_of_ours(&turn.bot) {
            continue;
        }
        return asked;
    }
    String::new()
}

/// Python: bot/memory/size.py:143 `_the_last_line(memory)`
///
/// The bot's last line, word for word, or "".
pub fn _the_last_line(memory: &Memory) -> String {
    memory.turns.iter().rev().find(|t| !t.bot.is_empty()).map(|t| text::strip(&t.bot).to_string()).unwrap_or_default()
}

/// Python: bot/memory/size.py:152 `_the_answer(memory)`
///
/// The last answer with a time in it, walking back past this module's own lines.
pub fn _the_answer(memory: &Memory) -> String {
    for turn in memory.turns.iter().rev() {
        if turn.bot.is_empty() {
            continue;
        }
        let answer = text::strip(&turn.bot);
        if _one_of_ours(answer) {
            continue;
        }
        if _A_TIME.is_match(answer) {
            return answer.to_string();
        }
        return String::new();
    }
    String::new()
}

/// Python: bot/memory/size.py:171 `_the_rule(answer)`
///
/// The sentence of the answer that is the rule for any size, or "".
pub fn _the_rule(answer: &str) -> String {
    for sentence in _SENTENCE.finditer(answer) {
        let said = text::strip(sentence.as_str());
        if !(_A_RATE.is_match(said) && _A_TIME.is_match(said)) {
            continue;
        }
        // The part after the last colon is tried first, and taken only while it is still a rule.
        let after = match said.rsplit_once(": ") {
            Some((_, after)) => text::strip(after).to_string(),
            None => String::new(),
        };
        for piece in [after.as_str(), said] {
            if !(_A_RATE.is_match(piece) && _A_TIME.is_match(piece)) {
                continue;
            }
            let piece = match _RULE_STOPS.search(piece) {
                Some(stop) => text::strip(&piece[..stop.start()]),
                None => piece,
            };
            if !piece.is_empty() {
                return piece.to_string();
            }
        }
    }
    String::new()
}

/// Python: bot/memory/size.py:195 `answer_for(text, memory=None)`
///
/// The same thing in another size, when that is what the line asks; otherwise None.
pub fn answer_for(text: &str, memory: &mut Memory) -> Option<String> {
    let said = text::squash(text);
    if said.is_empty() || text::split_ref(&said).len() > 8 {
        return None;
    }
    if !(_ANOTHER_SIZE.is_match(&said) && _POINTS_BACK.is_match(&said)) {
        return None;
    }
    let answer = _the_answer(memory);
    if answer.is_empty() {
        return None;
    }
    if !(_BY_WEIGHT.is_match(&answer) || _SIZED_THING.is_match(&answer) || _SIZED_ASKED.is_match(&_the_thing_asked(memory, &said)))
    {
        return None; // nothing in this chat comes in sizes, so "one" means something else
    }
    let bigger = !SMALLER_SAID.is_match(&said);
    // int(memory.get("size_said") or 0): a value int() cannot read is a ValueError nothing catches.
    let which = match memory.get("size_said") {
        Some(v) if v.truthy() => py_int(v).unwrap_or_else(|| panic!("ValueError: int({v:?})")),
        _ => 0,
    };
    memory.set("size_said", which + 1);
    let rule = _the_rule(&answer);
    let at = |n: usize| which.rem_euclid(n as i64) as usize;
    if !rule.is_empty() {
        if _one_of_ours(&_the_last_line(memory)) {
            // The rule was said in the last line, so this one says what is left.
            let shapes = if bigger { &*SAID_BIGGER } else { &*SAID_SMALLER };
            return Some(shapes[at(shapes.len())].clone());
        }
        let shapes = if bigger { &*BIGGER } else { &*SMALLER };
        let mut said = rule.trim_end_matches('.').to_string();
        let first = text::split_ref(&said).first().map(|s| s.to_string()).unwrap_or_default();
        if !first.is_empty() && !(pyuni::is_upper(&first) || pyuni::is_digit(&text::slice(&first, 0, Some(1)))) {
            // The rule sits after a colon, so its first letter is an ordinary one again.
            said = super::mem8_py::lower_first(&said);
        }
        let reply = shapes[at(shapes.len())].replace("{rule}", &said);
        return Some(format!("{}.", text::upper_first(&reply)));
    }
    let shapes = if bigger { &*MORE } else { &*LESS };
    Some(shapes[at(shapes.len())].clone())
}
