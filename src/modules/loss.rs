//! Port of `bot/modules/loss.py`: the reply a death asks for, and when an entry that says the sad thing itself keeps the turn (`bot/compose/loss.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::compose::{fit, loss};
use crate::understanding::understand;
use crate::understood::Understood;

/// Python: bot/modules/loss.py `defers_to(reply, line)` (also bound as `DEFERS_TO`).
///
/// True when the entry's reply should keep the turn. Five things have to hold: the entry says
/// the sad thing itself, the death in the line is a pet's, the line asks what to do rather than
/// how the person feels, something is asked at all, and the ask is one the entry answers.
pub fn defers_to(reply: &str, line: &str) -> bool {
    if ret_true(loss::acknowledges(reply))
        && ret_true(loss::a_pet_died(line))
        && !ret_true(loss::names_a_feeling(line))
        && ret_true(loss::asks_for_a_decision(line))
        && ret_true(loss::asks_about_the_remains(line, &Value::None))
    {
        return true;
    }
    // A death line carries its paperwork in the same breath as the death: "my mum died, is the
    // cremation expensive", "my husband passed away last month and i dont know where to start with
    // probate". The condolence answers the death and leaves the question, so an entry that answers
    // that question keeps the turn. Three things have to hold: the line names the administration
    // rather than the person's feelings, the entry's reply gives the kind of answer the line asked
    // for (a figure for a price, steps for "where do i start") - the composer's own fit check, so
    // this reads no phrasing of its own - and the reply is about that same piece of paperwork, not
    // a right-shaped card on the next topic over (bot/compose/loss.py, `is_about_the_paperwork`).
    if !ret_true(loss::asks_the_administration(line)) || ret_true(loss::names_a_feeling(line)) {
        return false;
    }
    if !ret_true(loss::is_about_the_paperwork(reply, line)) {
        return false;
    }
    let Some(want) = or_none(fit::wants(&understand(line, None), None)) else { return false };
    // `fit.gives` is read by attribute: the reading `bot/modules/on_record.py` rebinds at import.
    super::on_record::gives(reply, &want, Some(line)) == Some(true)
}

/// Python: bot/modules/loss.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The reply a death asks for, or None when this is an ordinary line. A follow-up is read the
/// same way while the bot's own last replies were about the loss; a change of subject ends it.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let understood = _reading(text, memory);
    if understood.as_ref().map_or(false, |understood| understood.about_bot) {
        // "do you miss me", "what do you think of me": asked inside a bereavement thread it is
        // still a line about the bot, and the bot's own answer is the one that fits - the thread
        // answered it with "that sounds heavy to be carrying", which is about the wrong person.
        return None;
    }
    let clean: Option<String> = understood.as_ref().map(|understood| understood.clean.clone());
    let words: Option<Vec<String>> = understood.as_ref().map(|understood| understood.words.clone());
    let options = _options(text, memory, clean.as_deref(), understood.as_ref(), words.as_deref());
    if !options.truthy() {
        return None;
    }
    let reply = pick(&py_strs(&options), memory, text);
    _the_missing_said(memory, &reply);
    Some(reply)
}

/// Python: bot/modules/loss.py `_the_missing_said(memory, reply)`
///
/// Note that the missing was met, for the turn after it (bot/memory/ack.py). Written where the
/// reply is chosen, so it lands in this module's copy of the memory.
pub fn _the_missing_said(memory: &mut Memory, reply: &str) {
    if !data::get("bot.compose.loss.MISS_REPLIES").contains_str(reply) {
        return;
    }
    let turn = memory.turns.len() as i64;
    memory.set("heavy_reply", dict_of(vec![("feeling", Value::from("missing")), ("turn", Value::Int(turn))]));
}

/// Python: bot/modules/loss.py `_reading(text, memory)`
///
/// The understanding engine's reading of *this* line: the stored one when it is of this line,
/// else the line read again. None is Python's `{}`, for a reading that fails.
pub fn _reading(text: &str, memory: &Memory) -> Option<Understood> {
    if let Some(stored) = memory.understood.as_ref() {
        if _same(&stored.clean, text) {
            return Some(stored.clone());
        }
    }
    // a reading that fails leaves the pools to the words
    py_try(|| understand(text, Some(memory))).ok()
}

/// Python: bot/modules/loss.py `_same(stored, text)`
///
/// True when the stored reading is of this line (the words, not the punctuation).
pub fn _same(stored: &str, text: &str) -> bool {
    if text::strip(stored).is_empty() {
        return false;
    }
    loss::words_of(stored).val() == loss::words_of(text).val()
}

/// Python: bot/modules/loss.py `_options(text, memory, clean, understood, words)`: the pool the
/// line asks for, or None (a reading that fails just passes the line on).
///
/// CONTRACT NOTE: `loss.reply_for` takes `clean: &str` and `topics: &[String]`, so Python's None
/// for either goes over as "" / an empty list.
pub fn _options(text: &str, memory: &Memory, clean: Option<&str>, understood: Option<&Understood>, words: Option<&[String]>) -> Value {
    or_on_raise(
        || {
            let words: Vec<String> = match words {
                Some(words) => words.to_vec(),
                None => py_strs(&loss::words_of(text).val()),
            };
            let topics: Vec<String> = understood.map(|understood| understood.topics.clone()).unwrap_or_default();
            loss::reply_for(text, memory, clean.unwrap_or(""), &topics, _mood(&words).as_deref()).val()
        },
        Value::None,
    )
}

/// Python: bot/modules/loss.py `_mood(words)`
///
/// 'negative' for a hard follow-up ("it was awful"), so it gets company, not a stock ack.
pub fn _mood(words: &[String]) -> Option<String> {
    let cues = py_try(_cues).ok()?;
    if words.iter().any(|word| cues.contains(word)) {
        Some("negative".to_string())
    } else {
        None
    }
}

/// Python: bot/modules/loss.py `_cues()`: the composer's one list of hard words.
pub fn _cues() -> HashSet<String> {
    data::set("bot.compose.fallback.NEGATIVE_CUES")
}
