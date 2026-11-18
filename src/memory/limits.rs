//! Port of `bot/memory/limits.py`: a limit on what somebody can do, said out loud ("she cant
//! walk far", "he is in a wheelchair"), read as a fact about the plan and answered with what it
//! means for the trip. The limit is kept in `memory["limits"]` for the plan modules.

use super::mem8_py::{mget, pick, rx, strs};
use crate::data::Value;
use crate::py::text;
use crate::state::{Memory, Turn};

const M: &str = "bot.memory.limits.";

rx!(
    _LIMIT_VERB, _AID, _IN_A_CHAIR, _FLAT, _NO_WALKS, _RULE_LEAD, _RULE_TAIL, _STAIRS_SAID, _REFUSES_AID,
    _DISLIKES_THE_WALK, _PUSHCHAIR, _FLAT_IS_A_HOME, _ILLNESS, _PET, _ASKS, _A_TRIP
);
strs!(WALK_REPLIES, STAIRS_REPLIES, CHAIR_REPLIES);

/// Python: bot/memory/limits.py:162 `_the_trip_is_on(clean, memory)`
///
/// Whether a trip or a day out is on the table for this limit to shape.
pub fn _the_trip_is_on(clean: &str, memory: &Memory) -> bool {
    if _A_TRIP.is_match(clean) {
        return true;
    }
    if _FLAT.is_match(clean) && !_FLAT_IS_A_HOME.is_match(clean) {
        return true;
    }
    let known = mget(memory, "user");
    if known.is_dict() {
        if let Some(places) = known.get("places").filter(|p| p.truthy()) {
            // `for place in places`: a str iterates its characters, a dict its keys.
            let any = match places {
                Value::Str(s) => s.chars().any(|c| !text::is_space(c)),
                Value::Dict(d) => d.iter().any(|(k, _)| !text::strip(&super::mem8_py::py_str(k)).is_empty()),
                other => other.items().iter().any(|p| !text::strip(&super::mem8_py::py_str(p)).is_empty()),
            };
            if any {
                return true;
            }
        }
        if _A_TRIP.is_match(&super::mem8_py::s_or(known.get("plan"))) {
            return true;
        }
    }
    let turns = _turns(memory);
    for turn in &turns[turns.len().saturating_sub(4)..] {
        if _A_TRIP.is_match(&turn.user) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/limits.py:218 `_kind(clean)`
///
/// Which limit the line is ("chair", "walk", "stairs"), or None. The aid comes first.
pub fn _kind(clean: &str) -> Option<String> {
    if _IN_A_CHAIR.is_match(clean) {
        return Some("chair".into());
    }
    if let Some(aid) = _AID.search(clean) {
        let aid = aid.as_str().to_lowercase();
        if aid.contains("wheelchair") || aid.contains("scooter") {
            return Some("chair".into());
        }
        return Some("walk".into());
    }
    if let Some(said) = _LIMIT_VERB.search(clean) {
        if said.as_str().to_lowercase().contains("stairs") {
            return Some("stairs".into());
        }
        return Some("walk".into());
    }
    if _DISLIKES_THE_WALK.is_match(clean) {
        return Some("walk".into());
    }
    if _FLAT.is_match(clean) {
        return Some("walk".into());
    }
    let lead = _RULE_LEAD.match_(clean).map(|m| m.end()).unwrap_or(0);
    // `_NO_WALKS.match(clean, pos)`: a search from pos whose match starts at pos is the same match.
    if let Some(rule) = _NO_WALKS.search_at(clean, lead).filter(|m| m.start() == lead) {
        if _RULE_TAIL.match_(&clean[rule.end()..]).is_some() {
            // Nobody in the line, so the rule is about the ground.
            return Some(if _STAIRS_SAID.is_match(rule.as_str()) { "stairs" } else { "walk" }.into());
        }
    }
    None
}

/// Python: bot/memory/limits.py:245 `names_a_limit(text)`
///
/// The kind of limit the line names, or None.
pub fn names_a_limit(text: &str) -> Option<String> {
    _kind(&text::squash(&text.replace('\u{2019}', "'")))
}

/// Python: bot/memory/limits.py:255 `reply(text, memory)`
///
/// A reply that turns a limit into a plan, or None when the line is not one.
pub fn reply(text: &str, memory: &mut Memory) -> Option<String> {
    let clean = text::squash(&text.replace('\u{2019}', "'"));
    if clean.is_empty() || text::split_ref(&clean).len() > 18 {
        return None;
    }
    if _PET.is_match(&clean) || _ILLNESS.is_match(&clean) || _REFUSES_AID.is_match(&clean) || _PUSHCHAIR.is_match(&clean) {
        return None;
    }
    let kind = _kind(&clean)?;
    // A question with a limit in it is somebody else's turn: it asks for a fact, not the plan.
    if text::rstrip(&clean).ends_with('?') || _ASKS.match_(&clean).is_some() {
        return None;
    }
    if !_the_trip_is_on(&clean, memory) {
        return None;
    }
    let mut kept = Value::dict();
    kept.set_key("kind", kind.as_str());
    kept.set_key("text", clean.as_str());
    kept.set_key("turn", _turns(memory).len());
    // The limit holds for the rest of the chat; the list keeps every one said (the last six).
    let mut limits = match memory.get("limits") {
        Some(Value::List(l)) => l.as_ref().clone(),
        _ => Vec::new(),
    };
    limits.push(kept);
    let from = limits.len().saturating_sub(6);
    memory.set("limits", Value::from(limits[from..].to_vec()));
    let pool: &[String] = match kind.as_str() {
        "walk" => &WALK_REPLIES,
        "stairs" => &STAIRS_REPLIES,
        _ => &CHAIR_REPLIES,
    };
    Some(pick(pool, memory, text))
}

/// Python: bot/memory/limits.py:288 `_turns(memory)`
pub fn _turns(memory: &Memory) -> &[Turn] {
    &memory.turns
}

/// Python: bot/memory/limits.py:293 `said(memory)`
///
/// The last limit told in this chat, or None.
pub fn said(memory: &Memory) -> Option<Value> {
    match memory.get("limits") {
        Some(Value::List(l)) => l.last().filter(|last| last.is_dict()).cloned(),
        _ => None,
    }
}
