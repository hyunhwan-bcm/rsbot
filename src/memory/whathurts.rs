//! Port of `bot/memory/whathurts.py`: "which one", "what did i say was hurting" - the part of
//! them the store holds as sore, said back.

use super::mem8_py::{rx, strset, user_or_empty, s_or};
use crate::data::Value;
use crate::py::random::Random;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.whathurts.";

rx!(BODY_PART, WHICH_PART, AFTER_WHICH_ONE, HURTING_ASK, ASKED_ABOUT_A_PART);
strset!(A_SIDE_TAIL);
static FACTS_BODY_PART: LazyLock<&'static Re> = LazyLock::new(|| crate::data::re("bot.memory.facts.BODY_PART"));
static FACTS_BODY_HURTS: LazyLock<&'static Re> = LazyLock::new(|| crate::data::re("bot.memory.facts.BODY_HURTS"));
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static SAID_WHAT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bsa(?:y|id)\b|\bwhat .*\b(?:hurt|sore|pain)\b", 0));

/// Python: bot/memory/whathurts.py:37 `_vary(options, memory)`
///
/// pick(), but over the whole reply: never one of the bot's last five lines while another is left.
pub fn _vary(options: &[String], memory: &Memory) -> String {
    let turns = &memory.turns;
    let recent: HashSet<&str> = turns[turns.len().saturating_sub(5)..].iter().map(|t| t.bot.as_str()).collect();
    let mut fresh: Vec<&String> = options.iter().filter(|o| !recent.contains(o.as_str())).collect();
    if fresh.is_empty() {
        fresh = options.iter().collect();
    }
    Random::new(turns.len() as i128).choice(&fresh).to_string()
}

/// Python: bot/memory/whathurts.py:46 `which_one_is_a_side(low)`
///
/// False when "which one" is the whole of a question about choosing, not about a side.
pub fn which_one_is_a_side(low: &str) -> bool {
    let Some(found) = AFTER_WHICH_ONE.search(low) else { return true };
    LETTERS.findall_str(&found.str(1).to_lowercase()).iter().all(|w| A_SIDE_TAIL.contains(w))
}

/// `(memory.get("user") or {}).get("hurts")`.
fn hurts(memory: &Memory) -> Option<Value> {
    user_or_empty(memory).get("hurts").cloned()
}

/// Python: bot/memory/whathurts.py:55 `before(memory)`
///
/// The part the store holds before this line is read, carried across `observe`.
pub fn before(memory: &Memory) -> String {
    match hurts(memory) {
        Some(held) if held.truthy() => {
            if held.is_dict() { s_or(held.get("part")) } else { String::new() }
        }
        _ => String::new(),
    }
}

/// Python: bot/memory/whathurts.py:61 `after(memory, held_before)`
///
/// A part named where another was held is a put-right, not a second ache: the store keeps the
/// part it came from (`hurts["was"]`).
pub fn after(memory: &mut Memory, held_before: &str) {
    let Some(held) = hurts(memory).filter(|h| h.is_dict()) else { return };
    if held_before.is_empty() || s_or(held.get("part")) == held_before {
        return;
    }
    if let Some(user) = memory.get_mut("user") {
        if let Some(h) = user.get_mut("hurts") {
            h.set_key("was", held_before);
        }
    }
}

/// Python: bot/memory/whathurts.py:77 `_a_question_about_a_part(line)`
pub fn _a_question_about_a_part(line: &str) -> bool {
    ASKED_ABOUT_A_PART.is_match(line)
}

/// `bot.memory.last_user(memory)`: the person's last line, "" without one.
fn last_user(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.user.clone()).unwrap_or_default()
}

/// Python: bot/memory/whathurts.py:81 `what_is_hurting_reply(memory, low)`
///
/// "what did i say was hurting": the part the store holds now, or None.
pub fn what_is_hurting_reply(memory: &Memory, low: &str) -> Option<String> {
    if !HURTING_ASK.is_match(low) || !SAID_WHAT.is_match(low) {
        return None;
    }
    let held = hurts(memory).filter(|h| h.is_dict() && h.get("part").is_some_and(Value::truthy));
    let Some(held) = held else {
        let previous = text::squash(&last_user(memory));
        if _a_question_about_a_part(&previous) {
            // The part is what the line asked about, not a complaint of theirs.
            let low_prev = previous.to_lowercase();
            let what = FACTS_BODY_PART.search(&low_prev).map(|m| m.str(1).to_lowercase()).unwrap_or_else(|| "it".into());
            return Some(_vary(
                &[
                    format!("You asked about a sore {what} - you haven't told me about anything of your own hurting."),
                    format!("Nothing of yours - you were asking about a sore {what}."),
                ],
                memory,
            ));
        }
        let low_prev = previous.to_lowercase();
        if FACTS_BODY_PART.is_match(&low_prev) && FACTS_BODY_HURTS.is_match(&low_prev) {
            // Somebody else's body: the line is what was said, and saying it back is honest.
            return Some(super::mem8_py::py_str(&super::told::_quote(&previous, memory)));
        }
        return Some(_vary(
            &[
                "You haven't told me about anything hurting.".to_string(),
                "Nothing of yours is hurting as far as I know.".to_string(),
                "You haven't said that anything is hurting.".to_string(),
            ],
            memory,
        ));
    };
    let part = s_or(held.get("part"));
    let was = s_or(held.get("was"));
    let side = s_or(held.get("side"));
    let one = if side.is_empty() { format!("your {part}") } else { format!("your {side} {part}") };
    let verb = if part.ends_with('s') { "were" } else { "was" };
    let cap = text::upper_first(&one);
    if !was.is_empty() {
        return Some(_vary(
            &[
                format!("It is {one} - you put the {was} right."),
                format!("{cap}, since you put the {was} right."),
                format!("You said {one} in the end, not the {was}."),
            ],
            memory,
        ));
    }
    Some(_vary(
        &[format!("It is {one} that you said {verb} hurting."), format!("{cap} - that is the one."), format!("You said {one}.")],
        memory,
    ))
}

/// Python: bot/memory/whathurts.py:124 `the_part_asked(memory, low)`
///
/// The part of them that is sore, said back to a question about which one it is, or None.
pub fn the_part_asked(memory: &Memory, low: &str) -> Option<String> {
    if !WHICH_PART.is_match(low) {
        return None;
    }
    if !which_one_is_a_side(low) {
        return None;
    }
    let held = hurts(memory).filter(|h| h.is_dict() && h.get("part").is_some_and(Value::truthy))?;
    let part = text::strip(&s_or(held.get("part"))).to_lowercase();
    let singular = part.trim_end_matches('s').to_string();
    match BODY_PART.search(low) {
        Some(asked) => {
            let asked = asked.str(1).to_lowercase();
            if asked != part && asked != singular && asked != format!("{part}s") {
                // A different part from the one the store holds.
                return None;
            }
        }
        None => {
            // "which one": the part has to be the one in front of the chat.
            let previous = text::squash(&last_user(memory).to_lowercase());
            if !previous.contains(&part) && !previous.contains(&singular) {
                return None;
            }
        }
    }
    let side = text::strip(&s_or(held.get("side"))).to_string();
    let place = text::strip(&s_or(held.get("place"))).to_string();
    if !side.is_empty() {
        let side_cap = text::upper_first(&side);
        let mut said = _vary(
            &[format!("{side_cap} {part}."), format!("Your {side} {part} - that is what you said."), format!("The {side} one.")],
            memory,
        );
        if !place.is_empty() {
            said = _vary(&[format!("{side_cap} {part}, {place} side."), format!("Your {side} {part}, the {place} of it.")], memory);
        }
        return Some(said);
    }
    if !place.is_empty() {
        return Some(_vary(
            &[
                format!("The {place} of it - you did not say which side."),
                format!("The {place} of your {part}, you said. Left or right you did not say."),
                format!("You said the {place} of it. Not which side."),
            ],
            memory,
        ));
    }
    Some(_vary(
        &[format!("You said your {part}, without saying which side."), format!("Just your {part} - you did not say left or right.")],
        memory,
    ))
}
