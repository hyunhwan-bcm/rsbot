//! Port of `bot/memory/goals.py`: a loss the chat already has the reason for is not a loss
//! that needs checking.
//!
//! "i lost 2 kg" a turn after "im trying to lose weight" is the plan working, and the figure is
//! said back; "how am i doing" is answered from the figure the chat holds.

use super::mem8_py::{lower_first, mget, py_str, rx, strs};
use crate::data::Value;
use crate::py::text;
use crate::state::Memory;

const M: &str = "bot.memory.goals.";

rx!(A_LOSS, A_PLAN_TO_LOSE, GLUED_UNIT, HOW_GOING);
strs!(SAID_BACK, FIRST, PROGRESS_REPLY);

/// Python: bot/memory/goals.py:75 `_tidy(how)`
///
/// "2kg" -> "2 kg", "half a stone" -> "Half a stone".
pub fn _tidy(how: &str) -> String {
    let said = GLUED_UNIT.sub_fn(&text::squash(how), |m| format!(" {}", m.str(1)));
    text::upper_first(&text::squash(&said))
}

/// Python: bot/memory/goals.py:82 `_the_plan_to_lose(memory)`
///
/// True when the chat holds an aim to lose weight, in the store's own words.
pub fn _the_plan_to_lose(memory: &Memory) -> bool {
    let user = mget(memory, "user");
    let mut said: Vec<String> = Vec::new();
    for key in ["plan", "goal", "aim", "diet"] {
        if let Some(Value::Str(value)) = user.get(key) {
            said.push(value.to_string());
        }
    }
    for key in ["plans", "goals"] {
        if let Some(value) = user.get(key).filter(|v| v.is_list()) {
            said.extend(value.items().iter().map(py_str));
        }
    }
    said.iter().any(|text_| !text_.is_empty() && A_PLAN_TO_LOSE.is_match(text_))
}

/// Python: bot/memory/goals.py:99 `_the_turn_that_said_it(memory)`
///
/// How many times the chat has been told about a loss, this line included.
pub fn _the_turn_that_said_it(memory: &Memory) -> usize {
    memory.turns.iter().filter(|turn| A_LOSS.is_match(&turn.user)).count()
}

/// Python: bot/memory/goals.py:109 `the_loss_that_was_the_plan(text, memory)`
///
/// The reply to a loss said in a chat that was already trying to lose it, or "".
pub fn the_loss_that_was_the_plan(text: &str, memory: &mut Memory) -> String {
    let Some(found) = A_LOSS.search(text) else { return String::new() };
    if !_the_plan_to_lose(memory) {
        return String::new();
    }
    let how = _tidy(found.name_str("how"));
    let pool = if _the_turn_that_said_it(memory) <= 1 { &*FIRST } else { &*SAID_BACK };
    let mut reply = pool[memory.turns.len() % pool.len()].clone();
    if reply.contains("{X}") {
        reply = reply.replace("{X}", &how);
    } else {
        reply = reply.replace("{x2}", &lower_first(&how));
    }
    if let Some(user) = memory.get_mut("user").filter(|u| u.is_dict()) {
        // The figure is the chat's own record of the plan working, and the store has no other
        // place for it: what has been lost is not the person's weight, it is progress.
        let mut progress = Value::dict();
        progress.set_key("lost", how.as_str());
        user.set_key("progress", progress);
    }
    reply
}

/// Python: bot/memory/goals.py:131 `_the_progress(memory)`
///
/// The figure the chat holds for the plan in hand, or "".
pub fn _the_progress(memory: &Memory) -> String {
    let user = mget(memory, "user");
    let lost = match user.get("progress").filter(|p| p.is_dict()).and_then(|p| p.get("lost")) {
        Some(v) if v.truthy() => text::strip(&py_str(v)).to_string(),
        _ => String::new(),
    };
    if !lost.is_empty() && !["weight", "some weight"].contains(&lost.to_lowercase().as_str()) {
        lost
    } else {
        String::new()
    }
}

/// Python: bot/memory/goals.py:141 `a_progress_asked(text, memory)`
///
/// The reply to "how am i doing" when the chat holds a figure for the plan, or "".
pub fn a_progress_asked(text: &str, memory: &Memory) -> String {
    if HOW_GOING.match_(&text::squash(&text.to_lowercase())).is_none() {
        return String::new();
    }
    let lost = _the_progress(memory);
    if lost.is_empty() {
        return String::new();
    }
    let reply = &PROGRESS_REPLY[memory.turns.len() % PROGRESS_REPLY.len()];
    reply.replace("{X}", &lost).replace("{x2}", &lower_first(&lost))
}
