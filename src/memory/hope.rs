//! Port of `bot/memory/hope.py`: a hopeful aside after the advice ("fingers crossed", "with
//! luck", "hope it works", "good luck"), said back briefly in the person's own idiom.

use super::mem8_py::{rx, val};
use crate::data::Value;
use crate::py::text;
use crate::state::Memory;

const M: &str = "bot.memory.hope.";

rx!(WISH, _FINGERS, _WOOD, _LUCK, _GOOD_LUCK, FOR_SOMEONE);
val!(GROUPS);

/// Python: bot/memory/hope.py:104 `_group(line)`
pub fn _group(line: &str) -> String {
    if _GOOD_LUCK.is_match(line) {
        return "goodluck".into();
    }
    if _FINGERS.is_match(line) {
        return "fingers".into();
    }
    if _WOOD.is_match(line) {
        return "wood".into();
    }
    if _LUCK.is_match(line) {
        return "luck".into();
    }
    "hope".into()
}

/// Python: bot/memory/hope.py:121 `reading(text)`
///
/// Whether the whole line is a hopeful aside about the thing in hand.
pub fn reading(text: &str) -> bool {
    let line = text::strip(text);
    if line.is_empty() || line.contains('?') {
        return false;
    }
    if FOR_SOMEONE.is_match(line) {
        return false;
    }
    WISH.match_(line).is_some()
}

/// Python: bot/memory/hope.py:131 `answer_for(text, memory)`
///
/// The brief reply to a hopeful aside, or None when the line is not one.
pub fn answer_for(text: &str, memory: &mut Memory) -> Option<String> {
    if !reading(text) {
        return None;
    }
    let pool = GROUPS.get(&_group(text)).expect("hope group").strs();
    // Counted in the conversation, so a second wish in the same chat is said a different way.
    // (isinstance(count, int) holds for a bool too, as in Python.)
    let count = match memory.get("hope_said").and_then(Value::as_int) {
        Some(count) => count + 1,
        None => 1,
    };
    memory.set("hope_said", count);
    Some(pool[(count - 1).rem_euclid(pool.len() as i64) as usize].clone())
}
