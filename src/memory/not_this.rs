//! Port of `bot/memory/not_this.py`: a line that is nothing but the rejection of a place ("not
//! portugal", "no rome, then", "we arent going to portugal"), answered with the place that
//! stands; otherwise the "still" reading of `bot.memory.tried`.

use super::mem8_py::{pick, py_iter, py_str, strs, val, PyVal};
use crate::data::Value;
use crate::knowledge::travel;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.not_this.";

val!(NOT_A_PLACE, _CONTRACTIONS);
strs!(STANDS, OUT, OUT_AFTER_A_QUESTION);

static A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z][a-z']*", 0));
static COUNTRIES: LazyLock<HashSet<String>> =
    LazyLock::new(|| travel::CITY_COUNTRY.keys().iter().filter_map(|k| travel::CITY_COUNTRY.get(k).cloned()).collect());

/// Python: bot/memory/not_this.py:100 `_the_place_the_chat_is_on(memory, gone)`
///
/// The place the chat holds that is not the one they ruled out, or "".
pub fn _the_place_the_chat_is_on(memory: &mut Memory, gone: &str) -> String {
    let facts = crate::memory::user(memory);
    let mut held: Vec<String> = Vec::new();
    for place in py_iter(facts.get("places")).iter().rev() {
        if let Value::Str(s) = place {
            if !text::strip(s).is_empty() {
                held.push(text::strip(s).to_string());
            }
        }
    }
    let facts = crate::memory::user(memory);
    if let Some(Value::Str(plan)) = facts.get("plan") {
        if !text::strip(plan).is_empty() {
            held.push(text::strip(plan).to_string());
        }
    }
    for place in &held {
        if place.to_lowercase() == gone.to_lowercase() {
            continue;
        }
        let named = _the_name_in(place, gone);
        if !named.is_empty() {
            return named;
        }
    }
    String::new()
}

/// Python: bot/memory/not_this.py:124 `_the_name_in(said, gone)`
///
/// The place the knowledge base knows inside a stored line, or "".
pub fn _the_name_in(said: &str, gone: &str) -> String {
    let whole = _a_place_the_chat_knows(said);
    if !whole.is_empty() && whole.to_lowercase() != gone.to_lowercase() {
        return whole;
    }
    let mut found = String::new();
    for word in A_WORD.findall_str(&said.to_lowercase()) {
        if word == gone.to_lowercase() {
            continue;
        }
        let named = _a_place_the_chat_knows(&word);
        if !named.is_empty() {
            found = named;
        }
    }
    found
}

/// Python: bot/memory/not_this.py:139 `_a_place_in_play(memory, place)`
///
/// True when the chat is on a place, so a ruled-out place is a place they were going to.
pub fn _a_place_in_play(memory: &mut Memory, place: &str) -> bool {
    let held = crate::memory::user(memory);
    if ["places", "plan", "city"].iter().any(|k| held.get(k).is_some_and(Value::truthy)) {
        return true;
    }
    let turns = &memory.turns;
    for turn in &turns[turns.len().saturating_sub(6)..] {
        for said in [&turn.user, &turn.bot] {
            for word in A_WORD.findall_str(&said.to_lowercase()) {
                if word != place.to_lowercase() && !_a_place_the_chat_knows(&word).is_empty() {
                    return true;
                }
            }
        }
    }
    false
}

/// Python: bot/memory/not_this.py:167 `_asked_last(memory)`
///
/// True when the bot's own last line asked something.
pub fn _asked_last(memory: &Memory) -> bool {
    memory.turns.last().is_some_and(|last| text::strip(&last.bot).ends_with('?'))
}

/// Python: bot/memory/not_this.py:177 `_a_place_the_chat_knows(place)`
///
/// The name written as the knowledge base holds it, or "" for a word that is no place.
pub fn _a_place_the_chat_knows(place: &str) -> String {
    let word = text::strip(place).to_lowercase();
    if word.is_empty() {
        return String::new();
    }
    if travel::CITIES.contains(&word) || travel::CITY_COUNTRY.contains(&word) {
        let named = crate::memory::facts::place_value(&word);
        return if named.truthy() { py_str(&named) } else { text::capitalize(&word) };
    }
    if COUNTRIES.contains(&word) || travel::CAPITALS.contains(&word) {
        return text::capitalize(&word);
    }
    String::new()
}

/// Python: bot/memory/not_this.py:189 `_in_plain_words(text)`
///
/// The line in long form, lower case, single-spaced and with its end stop taken off.
pub fn _in_plain_words(text: &str) -> String {
    let mut line = text::strip(text).to_lowercase().replace('\u{2019}', "'");
    for pair in _CONTRACTIONS.items() {
        let pair = pair.items();
        line = pair[0].as_re().expect("contraction").sub(pair[1].str(), &line);
    }
    text::rstrip_chars(&text::squash(&line), ".!,").to_string()
}

/// Python: bot/memory/not_this.py:197 `the_place_ruled_out(text, memory)`
///
/// The place a line of nothing but its rejection names, or "".
pub fn the_place_ruled_out(text: &str, _memory: &Memory) -> String {
    let line = _in_plain_words(text);
    let line = text::rstrip_chars(&line, ".!,");
    for shape in NOT_A_PLACE.items() {
        let Some(found) = shape.as_re().expect("NOT_A_PLACE shape").match_(line) else { continue };
        let place = _a_place_the_chat_knows(found.name_str("place"));
        if !place.is_empty() {
            return place;
        }
    }
    String::new()
}

/// Python: bot/memory/not_this.py:213 `_the_place_ruled_out(text, memory)`
///
/// The reply to a line that only rules a place out, or None.
pub fn _the_place_ruled_out(text: &str, memory: &mut Memory) -> Option<String> {
    let gone = the_place_ruled_out(text, memory);
    if gone.is_empty() {
        return None;
    }
    let stands = _the_place_the_chat_is_on(memory, &gone);
    if stands.is_empty() && !_a_place_in_play(memory, &gone) {
        return None;
    }
    let pool: &[String] = if !stands.is_empty() {
        &STANDS
    } else if _asked_last(memory) {
        &OUT_AFTER_A_QUESTION
    } else {
        &OUT
    };
    // The reply just given is dropped first, so two identical replies never come in a row.
    let lines: Vec<String> = pool.iter().map(|shape| shape.replace("{place}", &stands).replace("{gone}", &gone)).collect();
    let turns = &memory.turns;
    let said_lately: HashSet<&str> = turns[turns.len().saturating_sub(4)..].iter().map(|t| t.bot.as_str()).collect();
    let mut fresh: Vec<String> = lines.iter().filter(|l| !said_lately.contains(l.as_str())).cloned().collect();
    if fresh.is_empty() {
        fresh = lines;
    }
    Some(pick(&fresh, memory, text))
}

/// Python: bot/memory/not_this.py:238 `reply_for(text, memory)`
///
/// The reply to a line that only rules a place out, or a thing said again with "still", or None.
pub fn reply_for(text: &str, memory: &mut Memory) -> Option<String> {
    if let Some(ruled_out) = _the_place_ruled_out(text, memory).filter(|r| !r.is_empty()) {
        return Some(ruled_out);
    }
    match crate::memory::tried::reply_for(text, memory).val() {
        Value::Str(s) => Some(s.to_string()),
        _ => None,
    }
}
