//! Port of `bot/modules/talkto.py`: somebody asking for the bot's ear gets the yes and the one question, where no card answers the thing named (`bot/compose/talkto.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::knowledge::entry::Entry;
use crate::knowledge::lookup::find as find_entry;
use crate::understood::Understood;

static _A_GENERIC_OPENER: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.modules.talkto._A_GENERIC_OPENER"));
static _ROUTES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.talkto._ROUTES"));
static _A_LIFE_AT_RISK: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.talkto._A_LIFE_AT_RISK"));

/// Python: bot/modules/talkto.py `_the_entry_for(text)`
///
/// The entry the line's own words reach, or None when nothing fits. Asked about the line
/// `bot/compose/talkto.py` read, with the named day taken out.
pub fn _the_entry_for(text: &str) -> Option<&'static Entry> {
    // a reading that fails is the line as typed
    let line = py_try(|| ret_str(crate::compose::talkto::the_line_as_read(text))).unwrap_or_else(|()| text.to_string());
    let words = fold(&line);
    if words.is_empty() {
        return None;
    }
    // `find_entry({"clean": words, "words": words.split()})`: a reading with those two keys only.
    let reading = Understood { clean: words.clone(), words: text::split(&words), ..Understood::default() };
    // a lookup that raises finds nothing
    py_try(|| find_entry(&reading, None, None, false)).unwrap_or(None)
}

/// Python: bot/modules/talkto.py `DEFERS_TO(reply, text)`
///
/// Yes for every entry that answers the line: the opener row is the one exception.
pub fn DEFERS_TO(reply: &str, text: &str) -> bool {
    match _the_entry_for(text) {
        None => true,
        Some(found) => !_A_GENERIC_OPENER.contains(&found.id),
    }
}

/// Python: bot/modules/talkto.py `_the_entry_the_line_reached(memory)`
///
/// The entry the understanding engine's reading reaches, or None.
pub fn _the_entry_the_line_reached(memory: &Memory) -> Option<&'static Entry> {
    let understood = memory.understood.as_ref()?;
    // a lookup that raises reaches nothing
    py_try(|| find_entry(understood, None, None, false)).unwrap_or(None)
}

/// Python: bot/modules/talkto.py `_is_the_opener(entry)`
///
/// True when the only thing the line reached is the small-talk opener row.
pub fn _is_the_opener(entry: Option<&'static Entry>) -> bool {
    entry.map_or(false, |entry| _A_GENERIC_OPENER.contains(&entry.id))
}

/// Python: bot/modules/talkto.py `_a_route_is_named(reply)`
pub fn _a_route_is_named(reply: &str) -> bool {
    let said = text::strip(reply);
    !said.is_empty() && _ROUTES.iter().any(|who| said.starts_with(who.as_str()))
}

/// Python: bot/modules/talkto.py `_the_composer_says_a_route(text, memory)`
///
/// True when the composer's own reply to this line names somebody to ask.
pub fn _the_composer_says_a_route(text: &str, memory: &Memory) -> bool {
    // a composer that raises names no route; `dict(memory)` is a copy
    let Ok(said) = py_try(|| super::greeting::respond(text, &mut memory.clone())) else { return false };
    _a_route_is_named(&said.unwrap_or_default())
}

/// Python: bot/modules/talkto.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The yes and the one question, or None for every other line.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // A body in trouble is not an ask for the bot's ear, and that module sits below this one:
    // "can i talk to you about my son swallowing a battery" is a 999 line, not an opener.
    if ret_true(crate::compose::urgent::reply_for(text, memory)) {
        return None;
    }
    if _A_LIFE_AT_RISK.is_match(text) {
        return None;
    }
    let entry = _the_entry_the_line_reached(memory);
    // A card that answers the thing named is the better answer, and this module only ever fills
    // the gap where nothing does: the childcare card, the tenants' advice, the carers card.
    if entry.is_some() && !_is_the_opener(entry) {
        return None;
    }
    // The understanding engine's reading loses a line with a day in it ("can we talk tomorrow about
    // my mum" is read as asking for the ear and nothing else), so where it reached nothing the
    // knowledge base is asked about the line this module read - the same line, with the day taken
    // out - and a card that answers the thing named keeps the turn. Only asked where the reading
    // reached nothing: on every other line this module behaves exactly as it did.
    if entry.is_none() {
        let found = _the_entry_for(text);
        if found.is_some() && !_is_the_opener(found) {
            return None;
        }
    }
    // And a route to somebody who does know beats an opener as well: Citizens Advice for the tax or
    // the landlord, MoneyHelper for the money. The composer has already said it, so there is
    // nothing to add. The opener row is the exception, because a route is not what was asked for
    // on a line read as somebody asking for the bot's ear: "can i talk to you about my medication"
    // was "a pharmacist or your GP can answer that today", which is the shrug in a better suit.
    if !_is_the_opener(entry) && _the_composer_says_a_route(text, memory) {
        return None;
    }
    // The understanding engine's own reading of the line, because a person types "i wana talk
    // about my job" and the ask is the same ask: the engine has already made it "want".
    let clean = memory.understood.as_ref().map(|understood| understood.clean.clone()).unwrap_or_default();
    or_none(crate::compose::talkto::answer_for(if clean.is_empty() { text } else { &clean }, memory))
}
