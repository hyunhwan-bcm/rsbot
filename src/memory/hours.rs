//! Port of `bot/memory/hours.py`: the turn after the hours ("does that get easier", "how do i
//! stop", "how do i plan"), answered from the family of week kept for one turn in
//! `memory["hours"]` by `bot/memory/ack.py`.

use super::mem8_py::{int, pick, py_str, rx, val};
use crate::data::Value;
use crate::py::text;
use crate::state::Memory;

const M: &str = "bot.memory.hours.";

val!(HOURS_KEY, FAMILY_NAMES, KINDS);
rx!(AWAY, FEELING);
int!(MOST, LOOK_BACK);

/// Python: bot/memory/hours.py:109 `note(memory, family)`
///
/// Keep the family of week the hours line named, for the turn after it.
pub fn note(memory: &mut Memory, family: &Value) {
    if !family.truthy() {
        return;
    }
    let turn = memory.turns.len() as i64;
    let mut held = Value::dict();
    held.set_key("family", py_str(family));
    held.set_key("turn", turn);
    memory.set(HOURS_KEY.str(), held);
}

/// Python: bot/memory/hours.py:118 `remembered(memory)`
///
/// The family named on the last turn, or None when the hours are older than that.
pub fn remembered(memory: &Memory) -> Option<String> {
    let held = memory.get(HOURS_KEY.str()).filter(|h| h.is_dict())?;
    let want = Value::Int(memory.turns.len() as i64 - *LOOK_BACK);
    if held.get("turn").unwrap_or(&Value::None) != &want {
        return None; // the hours were said longer ago than the turn before this one
    }
    let family = match held.get("family") {
        Some(v) if v.truthy() => py_str(v),
        _ => String::new(),
    };
    if FAMILY_NAMES.get(&family).is_some() { Some(family) } else { None }
}

/// Python: bot/memory/hours.py:132 `answer_for(text, memory)`
///
/// A reply to the follow-up about the hours just said, or None when this is not one.
pub fn answer_for(text: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(text);
    if clean.is_empty() || text::split_ref(&clean).len() as i64 > *MOST {
        return None;
    }
    if AWAY.is_match(&clean) || FEELING.is_match(&clean) {
        return None;
    }
    let family = remembered(memory)?;
    for kind in KINDS.items() {
        let kind = kind.items();
        let pattern = kind[1].as_re().expect("KINDS pattern");
        if !pattern.is_match(&clean) {
            continue;
        }
        // "does that get easier" is a question and "i never know when to stop" is a statement,
        // and both are the same follow-up, so neither shape decides anything here.
        let pool = kind[2].strs();
        let hours = FAMILY_NAMES.get(&family).expect("family").str();
        return Some(pick(&pool, memory, &clean).replace("{hours}", hours));
    }
    None
}
