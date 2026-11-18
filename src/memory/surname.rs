//! Port of `bot/memory/surname.py`: the family name a person gives, and the ask that wants it
//! back.
//!
//! A family name is kept on its own (`user["surname"]`), filed only when the line says the word
//! is one ("my last name is okafor"), and answered only when the line asks for their own family
//! name and is not a how-to ("how do i change my surname").
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::mem7::{self, inline_re, res, sets};
use crate::py::random::Random;
use crate::py::re::Re;
use crate::py::text as pt;
use crate::state::Memory;
use std::sync::LazyLock;

res!("surname"; A_SURNAME_WORD, SAYS_A_SURNAME, NOT_THE_ASK, ASKS_THE_FIRST, ASKS_THE_FULL, ASKS_AGAIN, POINTS_BACK);
sets!("surname"; NAME_PARTS, DESCRIBES_A_NAME, NEVER_A_NAME, LOWER_PARTS);
inline_re! {
    APOSTROPHE_LETTER = "(['\u{2019}])([a-z])", 0;
}
static ASKS_THE_SURNAME: LazyLock<Vec<&'static Re>> =
    LazyLock::new(|| data::get("bot.memory.surname.ASKS_THE_SURNAME").items().iter().map(|p| p.as_re().expect("a pattern")).collect());

/// Python: bot/memory/surname.py:94 `_vary(options, memory)`
///
/// A choice that does not repeat the last thing the bot said.
pub fn _vary(options: &[String], memory: &Memory) -> String {
    let recent: Vec<&str> = mem7::tail(&memory.turns, 5).iter().map(|turn| turn.bot.as_str()).collect();
    let mut fresh: Vec<String> = options.iter().filter(|o| !recent.contains(&o.as_str())).cloned().collect();
    if fresh.is_empty() {
        fresh = options.to_vec();
    }
    Random::new(memory.turns.len() as i128).choice(&fresh).clone()
}

/// Python: bot/memory/surname.py:102 `_the_name_they_said(word)`
///
/// The name the way it is written: "okafor" -> "Okafor", "van der berg" -> "van der Berg".
pub fn _the_name_they_said(word: &str) -> String {
    let parts = pt::split(word);
    if parts.is_empty() {
        return String::new();
    }
    let written: Vec<String> = parts
        .iter()
        .enumerate()
        .map(|(index, part)| {
            if LOWER_PARTS.contains(&pt::lower(part)) && index < parts.len() - 1 {
                pt::lower(part)
            } else {
                pt::upper_first(part)
            }
        })
        .collect();
    APOSTROPHE_LETTER.sub_fn(&written.join(" "), |m| format!("{}{}", m.str(1), pt::upper(m.str(2))))
}

/// Python: bot/memory/surname.py:121 `the_surname(memory)`
///
/// The family name this chat was told, or "".
pub fn the_surname(memory: &Memory) -> String {
    match memory.get("user").filter(|u| u.is_dict()).and_then(|u| u.get("surname")) {
        Some(Value::Str(said)) => pt::strip(said).to_string(),
        _ => String::new(),
    }
}

/// Python: bot/memory/surname.py:130 `_given_one(text)`
///
/// The family name the line gives, or "" when the line gives none.
pub fn _given_one(text: &str) -> String {
    for m in SAYS_A_SURNAME.finditer(text) {
        let name = m.name_str("name");
        let words = pt::split_ref(name);
        let first = words[0];
        let low = pt::lower(pt::strip(first));
        if low.is_empty() || DESCRIBES_A_NAME.contains(&low) || NEVER_A_NAME.contains(&low) || pt::len(&low) < 2 {
            continue;
        }
        let mut parts = vec![first.to_string()];
        for word in &words[1..] {
            // Only a part of a name joins the name.
            let last_is_a_part = NAME_PARTS.contains(&pt::lower(parts.last().expect("not empty")));
            if NAME_PARTS.contains(&pt::lower(word)) || mem7::is_upper(mem7::first_chars(word, 1)) || (last_is_a_part && pt::is_alpha(word)) {
                parts.push(word.to_string());
                continue;
            }
            break;
        }
        return _the_name_they_said(&parts.join(" "));
    }
    String::new()
}

/// Python: bot/memory/surname.py:151 `observe(text, memory)`
///
/// File a family name the line gives. True when one was kept.
pub fn observe(text: &str, memory: &mut Memory) -> bool {
    let said = _given_one(text);
    if said.is_empty() {
        return false;
    }
    let held = memory.setdefault("user", Value::dict());
    if !held.is_dict() {
        return false; // a store written by something else keeps its shape
    }
    held.set_key("surname", said);
    true
}

/// Python: bot/memory/surname.py:165 `_asks_for_it(text)`
///
/// True when the line is the person asking for their own family name.
pub fn _asks_for_it(text: &str) -> bool {
    let said = pt::squash(text);
    if said.is_empty() || !A_SURNAME_WORD.is_match(&said) || mem7::word_count(&said) > 12 || NOT_THE_ASK.is_match(&said) {
        return false;
    }
    ASKS_THE_SURNAME.iter().any(|p| p.is_match(&said))
}

/// Python: bot/memory/surname.py:188 `_the_first_name(memory)`
pub fn _the_first_name(memory: &mut Memory) -> String {
    crate::memory::name(memory)
}

/// Python: bot/memory/surname.py:193 `a_note(text, memory)`
///
/// A short word back when the whole line is the family name and nothing else, or None.
pub fn a_note(text: &str, memory: &Memory) -> Option<String> {
    let said = pt::squash(text);
    if said.is_empty() {
        return None;
    }
    let m = SAYS_A_SURNAME.match_(&said)?;
    if !pt::strip_chars(&said[m.end()..], " .!?,;-").is_empty() {
        return None;
    }
    if _given_one(&said).is_empty() {
        return None;
    }
    let held = the_surname(memory);
    if held.is_empty() {
        return None;
    }
    Some(_vary(&[format!("{held} \u{2014} noted."), format!("Got it \u{2014} {held}."), format!("{held}, I'll remember that.")], memory))
}

/// Python: bot/memory/surname.py:226 `_the_lines_before(memory, back=3)`
///
/// What the last few turns said, to see whether the chat is still on the surname.
pub fn _the_lines_before(memory: &Memory, back: usize) -> String {
    let turns: Vec<&crate::state::Turn> = memory.turns.iter().filter(|turn| !pt::strip(&turn.user).is_empty()).collect();
    let lines: Vec<String> = mem7::tail(&turns, back).iter().map(|turn| format!("{} {}", turn.user, turn.bot)).collect();
    pt::lower(&lines.join(" "))
}

/// Python: bot/memory/surname.py:234 `a_follow_up(text, memory)`
///
/// The family name for a follow-up that points back at it, or None.
pub fn a_follow_up(text: &str, memory: &Memory) -> Option<String> {
    let said = pt::lower(&_plain(text));
    if said.is_empty() || the_surname(memory).is_empty() || mem7::word_count(&said) > 7 {
        return None;
    }
    if NOT_THE_ASK.is_match(&said) || A_SURNAME_WORD.is_match(&said) {
        return None;
    }
    if !(ASKS_AGAIN.is_match(&said) && POINTS_BACK.is_match(&said)) {
        return None;
    }
    if !A_SURNAME_WORD.is_match(&_the_lines_before(memory, 3)) {
        return None;
    }
    let held = the_surname(memory);
    Some(_vary(&[format!("It's {held}."), format!("{held} \u{2014} as you told me."), format!("Still {held}.")], memory))
}

/// Python: bot/memory/surname.py:254 `there_is_a_chat(memory)`
pub fn there_is_a_chat(memory: &Memory) -> bool {
    crate::memory::there_is_a_chat(memory)
}

/// Python: bot/memory/surname.py:260 `reply_for(text, memory)`
///
/// The family name said back, or an honest line when the chat never gave one.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn reply_for(text: &str, memory: &mut Memory) -> Option<String> {
    if let Some(note) = a_note(text, memory).filter(|n| !n.is_empty()) {
        return Some(note);
    }
    if !there_is_a_chat(memory) {
        // A memory reply needs a memory: on the first message the modules that answer lines answer.
        return None;
    }
    if ASKS_THE_FULL.is_match(&_plain(text)) {
        return Some(_the_full_name(memory));
    }
    if ASKS_THE_FIRST.is_match(&_plain(text)) {
        return Some(_the_first_name_reply(memory));
    }
    if !_asks_for_it(text) {
        return a_follow_up(text, memory);
    }
    if !_given_one(text).is_empty() {
        return None; // the give is the answer it belongs to
    }
    let held = the_surname(memory);
    if held.is_empty() {
        memory.set("asked_for", "surname");
        let options = ["You haven't told me your surname yet.", "I don't have it \u{2014} you've not said your surname.", "You haven't given me a surname, so I'd only be guessing."];
        return Some(_vary(&options.map(String::from), memory));
    }
    Some(_vary(&[format!("It's {held}."), format!("{held} \u{2014} that's the one you gave me."), format!("Your surname's {held}.")], memory))
}

/// Python: bot/memory/surname.py:295 `_plain(text)`
pub fn _plain(text: &str) -> String {
    pt::squash(text)
}

/// Python: bot/memory/surname.py:299 `_the_full_name(memory)`
///
/// Both names, when the chat holds them: "Sam Okafor".
pub fn _the_full_name(memory: &mut Memory) -> String {
    let first = _the_first_name(memory);
    let family = the_surname(memory);
    if !first.is_empty() && !family.is_empty() {
        return _vary(&[format!("{first} {family}."), format!("You're {first} {family}."), format!("{first} {family} \u{2014} that's both of the names you gave.")], memory);
    }
    if !first.is_empty() {
        return format!("I have your first name, {first}, but you haven't given me a surname.");
    }
    if !family.is_empty() {
        return format!("I have your surname, {family}, but you've not told me your first name.");
    }
    memory.set("asked_for", "name");
    _vary(&["You haven't told me your name yet.".to_string(), "I don't have a name from you \u{2014} you've not said.".to_string()], memory)
}

/// Python: bot/memory/surname.py:314 `_the_first_name_reply(memory)`
pub fn _the_first_name_reply(memory: &mut Memory) -> String {
    let first = _the_first_name(memory);
    if !first.is_empty() {
        return _vary(&[format!("{first} \u{2014} you told me."), format!("It's {first}."), format!("Your first name's {first}.")], memory);
    }
    memory.set("asked_for", "name");
    _vary(&["You haven't told me your first name yet.".to_string(), "I don't have it \u{2014} you've not said your first name.".to_string()], memory)
}
