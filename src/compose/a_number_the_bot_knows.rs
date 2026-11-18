//! Port of `bot/compose/a_number_the_bot_knows.py`: the few phone numbers this bot knows, given
//! when the person asks for one of them by name.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_vals, inline_re};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::Memory;

bundle_res!("bot.compose.a_number_the_bot_knows"; AND_THE_NEXT, A_BARE_NAME, ASKS_AGAIN, _IN_FRONT, _BEHIND);
bundle_vals!("bot.compose.a_number_the_bot_knows"; LINES, ASKS, NO_NUMBER_HERE, _REPEAT_TURNS);
inline_re!(LEADING_ARTICLE, r"^(?:the|a|an)\s+");
inline_re!(A_DIGIT, r"\d");
inline_re!(A_NUMBER, r"\d[\d ]{2,}");

/// `(names, said)` rows of LINES or NO_NUMBER_HERE.
fn rows(table: &'static crate::data::Value) -> impl Iterator<Item = (Vec<String>, &'static str)> {
    table.items().iter().map(|row| (row.items()[0].strs(), row.items()[1].str()))
}

/// Python: bot/compose/a_number_the_bot_knows.py:112 `_said_before(memory)`
///
/// The line this module gave on the last turn, if that is what the last turn was, else "".
pub fn _said_before(memory: &Memory) -> String {
    let replies: Vec<String> = _replies(memory).into_iter().filter(|r| !r.is_empty()).collect();
    let Some(last) = replies.last() else { return String::new() };
    for (_names, said) in rows(&LINES).chain(rows(&NO_NUMBER_HERE)) {
        if *last == text::squash(said) {
            return said.to_string();
        }
    }
    String::new()
}

/// Python: bot/compose/a_number_the_bot_knows.py:124 `_carried_thing(text)`
///
/// The thing a line that only points back at the last question is asking about, or "".
pub fn _carried_thing(text: &str) -> String {
    let squashed = text::squash(&text.to_lowercase().replace('\u{2019}', "'"));
    let line = text::strip_chars(&squashed, " ?!.,");
    if let Some(found) = AND_THE_NEXT.match_(line) {
        return text::strip_chars(found.str(1), " ?!.,'\"").to_string();
    }
    if A_BARE_NAME.match_(line).is_some() {
        return line.to_string();
    }
    String::new()
}

/// Python: bot/compose/a_number_the_bot_knows.py:143 `_asked_about(text)`
///
/// The thing whose number is wanted, cleaned of trailing punctuation, or "".
pub fn _asked_about(text: &str) -> String {
    let squashed = text::squash(&text.to_lowercase().replace('\u{2019}', "'"));
    let line = text::strip_chars(&squashed, " ?!.,");
    let line = _IN_FRONT.sub("", &_BEHIND.sub("", line));
    let line = text::strip_chars(&line, " ?!.,");
    if line.is_empty() {
        return String::new();
    }
    for ask in ASKS.items() {
        let ask = ask.as_re().expect("ASKS holds compiled patterns");
        if let Some(found) = ask.search(line) {
            let thing = text::strip_chars(found.str(1), " ?!.,'\"");
            let thing = LEADING_ARTICLE.sub("", thing);
            if !thing.is_empty() {
                return thing;
            }
        }
    }
    String::new()
}

/// Python: bot/compose/a_number_the_bot_knows.py:160 `_names(thing, names)`
///
/// True when the thing wanted is one of the names, as a whole word or a whole phrase. Whole
/// words, because "refuge" is inside "refugees" and "111" inside "1111".
pub fn _names(thing: &str, names: &[String]) -> bool {
    names
        .iter()
        .any(|name| Re::new(&format!(r"(?<![a-z0-9]){}(?![a-z0-9])", pyre::escape(name)), 0).search(thing).is_some())
}

/// Python: bot/compose/a_number_the_bot_knows.py:172 `_replies(memory)`
///
/// What the bot said on each turn. (Python also reads a turn kept as a `(user, bot)` pair or
/// under "reply"; the port's memory keeps every turn in the one typed shape.)
pub fn _replies(memory: &Memory) -> Vec<String> {
    memory.turns.iter().map(|turn| text::squash(&turn.bot)).collect()
}

/// Python: bot/compose/a_number_the_bot_knows.py:190 `already_said(number, memory)`
///
/// True when the bot has said this number in the last few turns.
pub fn already_said(number: &str, memory: &Memory) -> bool {
    let said = number.replace(' ', "");
    let replies = _replies(memory);
    let window = _REPEAT_TURNS.as_int().expect("_REPEAT_TURNS is an int");
    com4::tail(&replies, window).iter().any(|reply| !said.is_empty() && reply.replace(' ', "").contains(&said))
}

/// Python: bot/compose/a_number_the_bot_knows.py:199 `not_a_number(reply)`
///
/// True when the reply the bot is about to send holds no number at all. A reply with no digit
/// in it cannot be the answer to a number asked for.
pub fn not_a_number(reply: &str) -> bool {
    A_DIGIT.search(reply).is_none()
}

/// Python: bot/compose/a_number_the_bot_knows.py:210 `answer_for(text, memory=None)`
///
/// The number, when the person asked for one this bot knows. None for everything else.
pub fn answer_for(text: &str, memory: &Memory) -> Option<String> {
    let line = com4::flat(text);
    let again = ASKS_AGAIN.search(&line).is_some();
    let before = _said_before(memory);
    let mut thing = _asked_about(text);
    if thing.is_empty() && !before.is_empty() {
        thing = _carried_thing(text);
    }
    if !thing.is_empty() {
        for (names, said) in rows(&LINES) {
            if !_names(&thing, &names) {
                continue;
            }
            if let Some(number) = A_NUMBER.search(said) {
                if already_said(number.as_str(), memory) && !again {
                    return None;
                }
            }
            return Some(said.to_string());
        }
        for (names, said) in rows(&NO_NUMBER_HERE) {
            if _names(&thing, &names) {
                return Some(said.to_string());
            }
        }
    }
    if !before.is_empty() && again {
        // Nothing is named that this module knows, and the line asks again about the last one:
        // "sorry, what was it again" one line after the number, which is the same question.
        return Some(before);
    }
    None
}
