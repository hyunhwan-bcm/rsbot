//! Port of `bot/compose/setting_off.py`: "what time should i leave" gets the method for working
//! it out (back from the hour you must be there), in place of a pointer to search.

use crate::data;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.setting_off.", stringify!($name))));)*
    };
}
regexes!(
    ASK_SHAPE, ASK_LATEST, FIRST_PERSON, THEIR_OWN_CHOICE, SOMEBODY_ELSE, SET_OFF_A_THING,
    THEIR_TIMETABLE, A_TIME_GIVEN, THE_HOUR_IS_FIXED, A_DESTINATION,
);
macro_rules! strs {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.setting_off.", stringify!($name))));)*
    };
}
strs!(SETTING_OFF, NOT_A_JOURNEY, POINTERS, NEEDS_THE_HOUR, THE_HOUR_IS_KNOWN, THE_HOUR_THEY_SAID, A_PLACE_NAMED);
/// `re.sub(r"^(?:at|by|for|before|until|till)\s+", "", ...)`, written inline.
static A_PREPOSITION: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:at|by|for|before|until|till)\s+", 0));

/// Python: bot/compose/setting_off.py:150 `_the_hour_already_said(text, memory)`
///
/// The hour the person fixed a line or two back, with the preposition off it ("9", "2.30pm"),
/// or "" (only this file calls it).
pub fn _the_hour_already_said(_text: &str, memory: &Memory) -> String {
    let turns = &memory.turns[memory.turns.len().saturating_sub(2)..];
    for turn in turns.iter().rev() {
        let said = text::squash(&text::lower(&turn.user));
        if said.is_empty() || !THE_HOUR_IS_FIXED.is_match(&said) {
            continue;
        }
        let Some(found) = A_TIME_GIVEN.search(&said) else { continue };
        return A_PREPOSITION.sub("", text::strip(found.as_str()));
    }
    String::new()
}

/// Python: bot/compose/setting_off.py:166 `is_a_pointer(reply)`
///
/// True when the composer's line names where to look rather than giving an answer.
pub fn is_a_pointer(reply: &str) -> bool {
    let said = text::lower(reply);
    POINTERS.iter().any(|shape| said.contains(shape.as_str()))
}

/// Python: bot/compose/setting_off.py:172 `reads_it(text)`
///
/// True for a question about what time the person should set off.
pub fn reads_it(text: &str) -> bool {
    let said = text::squash(&text::lower(text));
    if said.is_empty() {
        return false;
    }
    if !(ASK_SHAPE.is_match(&said) || ASK_LATEST.is_match(&said)) {
        return false;
    }
    if !(FIRST_PERSON.is_match(&said) || (THEIR_OWN_CHOICE.is_match(&said) && !SOMEBODY_ELSE.is_match(&said))) {
        return false;
    }
    if !SETTING_OFF.iter().any(|verb| said.contains(verb.as_str())) {
        return false;
    }
    if NOT_A_JOURNEY.iter().any(|fence| said.contains(fence.as_str())) {
        return false;
    }
    if SET_OFF_A_THING.is_match(&said) {
        return false;
    }
    !THEIR_TIMETABLE.is_match(&said)
}

/// Python: bot/compose/setting_off.py:192 `_choose(pool, memory)`
///
/// The first line whose opening the bot's last reply does not already carry (only this file
/// calls it).
pub fn _choose(pool: &[String], memory: &Memory) -> String {
    let last = memory.turns.last().map(|t| t.bot.as_str()).unwrap_or("");
    for line in pool {
        if last.is_empty() || !last.contains(&text::slice(line, 0, Some(30))) {
            return line.clone();
        }
    }
    pool[0].clone()
}

/// Python: bot/compose/setting_off.py:203 `answer_for(text, memory=None)`
///
/// The method for working out a leaving time, or None.
pub fn answer_for(text: &str, memory: &Memory) -> Option<String> {
    if !reads_it(text) {
        return None;
    }
    let said = text::squash(&text::lower(text));
    if A_TIME_GIVEN.is_match(&said) {
        return Some(_choose(&THE_HOUR_IS_KNOWN, memory));
    }
    let hour = _the_hour_already_said(text, memory);
    if !hour.is_empty() {
        // `line % hour`: each line holds one "%s".
        let lines: Vec<String> = THE_HOUR_THEY_SAID.iter().map(|line| line.replacen("%s", &hour, 1)).collect();
        return Some(_choose(&lines, memory));
    }
    if A_DESTINATION.is_match(&said) {
        return Some(_choose(&A_PLACE_NAMED, memory));
    }
    Some(_choose(&NEEDS_THE_HOUR, memory))
}
