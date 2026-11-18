//! Port of `bot/modules/said_fact_back.py`: their own line handed back where the store's reply to a recall ask would deny it or miss it (`bot/memory/earlier.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::memory::earlier;

/// Python: bot/modules/said_fact_back.py `observe(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// Nothing is kept here: the check reads the turns, it does not add to them.
pub fn observe(text: &str, memory: &mut Memory) {}

/// Python: bot/modules/said_fact_back.py `_the_store_would_say(text, memory)`
///
/// What the memory store answers this line, on this module's memory (so its notes are kept).
pub fn _the_store_would_say(text: &str, memory: &mut Memory) -> Option<String> {
    // this module must never lose the turn
    py_try(|| super::memory_store::respond(text, memory)).unwrap_or(None)
}

/// Python: bot/modules/said_fact_back.py `_nothing_to_give(reply)`
///
/// True for the two replies that carry no answer: the search hedge and the hand-off.
pub fn _nothing_to_give(reply: &str) -> bool {
    // a check that fails must not change the reply
    or_on_raise(|| ret_true(crate::compose::fallback::is_a_hedge(reply)) || ret_true(crate::compose::fit::is_a_punt(reply)), false)
}

/// Python: bot/modules/said_fact_back.py `_someone_answers_it(text, memory)`
///
/// Whether the knowledge base has a reply of its own for this line. A real answer always
/// outranks a quotation of an earlier line.
pub fn _someone_answers_it(text: &str, memory: &mut Memory) -> bool {
    // a check that fails must not change the reply
    match py_try(|| super::answers::respond(text, memory)) {
        Ok(Some(said)) => !text::strip(&said).is_empty(),
        _ => false,
    }
}

/// Python: bot/modules/said_fact_back.py `_another_module_gives_the_value(text, memory)`
///
/// Whether a module that answers these shapes with the value itself already has an answer. Only
/// the value the ask names counts.
pub fn _another_module_gives_the_value(text: &str, memory: &mut Memory) -> bool {
    // a check that fails must not change the reply
    let said = match py_try(|| super::attribute_said_again::respond(text, memory)) {
        Ok(Some(said)) if !text::strip(&said).is_empty() => said,
        _ => return false,
    };
    // Only the value the ask names counts: `how often did i say i go` was answered "You said every
    // day." - the dog walk - and that is a wrong answer of its own, not this module standing down.
    ret_true(earlier::gives_the_named_value(text, memory, &said))
}

/// Python: bot/modules/said_fact_back.py `_said_back_already(text, memory)`
///
/// Whether the module that owns the ask-for-it-again shapes has an answer for this line.
pub fn _said_back_already(text: &str, memory: &Memory) -> bool {
    // a check that fails must not change the reply
    or_on_raise(|| ret_true(crate::memory::lineback::reply_for(text, memory)), false)
}

/// `earlier.put_right(theirs, memory) or earlier.a_misquoted_line(theirs, memory, text)`.
fn put_right_or_misquoted(theirs: &str, memory: &Memory, text: &str) -> Value {
    let put = earlier::put_right(theirs, memory, "").val();
    if put.truthy() {
        put
    } else {
        earlier::a_misquoted_line(theirs, memory, text).val()
    }
}

/// Python: bot/modules/said_fact_back.py `TAKES_AS_IT_COMES(text, memory)`
///
/// The store claims a recall ask as its own, which is right until it denies their own line. This
/// module takes the turn only where it has read the whole line against their own words: the ask
/// is a recall ask and the store's own answer to it contradicts a line of theirs.
pub fn TAKES_AS_IT_COMES(text: &str, memory: &mut Memory) -> bool {
    if !data::re("bot.memory.earlier.RECALL").is_match(&fold(text)) {
        return false;
    }
    let theirs = match _the_store_would_say(text, memory) {
        Some(theirs) if !text::strip(&theirs).is_empty() => theirs,
        _ => return false,
    };
    let put = put_right_or_misquoted(&theirs, memory, text);
    if put.truthy() && put.as_str() != Some(theirs.as_str()) {
        return true;
    }
    ret_true(earlier::another_line_was_asked_for(text, memory, &theirs))
}

/// Python: bot/modules/said_fact_back.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// Their line handed back where the reply would deny it or miss it, else None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    if let Some(theirs) = _the_store_would_say(text, memory).filter(|theirs| !text::strip(theirs).is_empty()) {
        let put = put_right_or_misquoted(&theirs, memory, text);
        if put.truthy() {
            return Some(py_str(&put));
        }
        if ret_true(earlier::another_line_was_asked_for(text, memory, &theirs)) && !_another_module_gives_the_value(text, memory) {
            // The ask named the line it means and the store answered with another of that kind:
            // `how often did i say i go` was told "You also walk the dog every day." while the gym
            // was three times a week two lines up. The named line is the answer.
        } else if !_nothing_to_give(&theirs) {
            // The store's own answer stands, and it stands as the store's reply: a pass here
            // leaves the store's own candidate in the running, so no specialist loses its slot
            // to a copy of the words it would have got anyway.
            return None;
        }
        // A search hedge or a hand-off is the store saying it has nothing, so a recall ask is
        // still answered from their own line: "back to the bedroom, what size was it again" was
        // told the bot would rather not guess while the size was in the chat.
    }
    if _said_back_already(text, memory) {
        return None;
    }
    if !ret_true(earlier::their_own_words(text)) && _someone_answers_it(text, memory) {
        // Unless the person is asking what they themselves said or do: then their own line is the
        // answer and a general card about the subject is not.
        return None;
    }
    // The store had nothing for the line, so nothing is denied and something may still be there:
    // a recall ask ("what size did i say") and a plain one ("how old is she") both have an answer
    // when a line of theirs carries the value.
    match or_none(earlier::answer_for(text, memory)) {
        Some(answer) => Some(answer),
        None => as_reply(earlier::from_the_line(text, memory)),
    }
}
