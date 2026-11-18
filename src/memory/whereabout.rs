//! Port of `bot/memory/whereabout.py`: a line that is only a place ("near leeds", "somewhere
//! warm") put on the question a turn or two back that asked for one.

use super::mem8_py::{rx, strset};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.whereabout.";

rx!(PLACE_LINE, QUALIFIED_LINE, ASKS_FOR_A_PLACE);
strset!(NOT_A_PLACE);
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// Python: bot/memory/whereabout.py:81 `the_place(line)`
///
/// The place a line that is only a place says, or "" for any other line.
pub fn the_place(line: &str) -> String {
    let said = text::strip_chars(text::strip(line), " .!,").to_lowercase();
    if said.is_empty() || text::split_ref(&said).len() > 5 {
        return String::new();
    }
    let Some(found) = PLACE_LINE.match_(&said).or_else(|| QUALIFIED_LINE.match_(&said)) else { return String::new() };
    let place = text::strip_chars(found.name_str("place"), " .").to_string();
    if place.is_empty() {
        return String::new();
    }
    let first = text::split_ref(&place)[0].to_string();
    if NOT_A_PLACE.contains(&first) || NOT_A_PLACE.contains(&place) {
        return String::new();
    }
    // "in a minute" is a when.
    if first == "a" || first == "an" {
        return String::new();
    }
    // "in the oven" is a room, not a place to go.
    if let Some(rest) = place.strip_prefix("the ") {
        if NOT_A_PLACE.contains(rest) {
            return String::new();
        }
    }
    place
}

/// Python: bot/memory/whereabout.py:113 `the_question_with_the_place(text, memory)`
///
/// The question this line puts a place on, with the place in it; or "".
pub fn the_question_with_the_place(text: &str, memory: &Memory) -> String {
    let place = the_place(text);
    if place.is_empty() {
        return String::new();
    }
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(3)..].iter().rev() {
        let asked = text::strip(&turn.user);
        if asked.is_empty() || !ASKS_FOR_A_PLACE.is_match(asked) {
            continue;
        }
        // The place has to be new to this conversation.
        let said: HashSet<String> = LETTERS.findall_str(&turn.bot.to_lowercase()).into_iter().collect();
        if said.contains(text::split_ref(&place)[0]) {
            return String::new();
        }
        return format!("{} {}", text::rstrip_chars(asked, " ?."), text::strip_chars(text::strip(text), " .!"));
    }
    String::new()
}
