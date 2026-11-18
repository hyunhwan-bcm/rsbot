//! Port of `bot/modules/name_taken_back.py`: a name the person says is wrong comes off the record, with an apology and the question.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

static NAME_TAKEN_BACK_SAY: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.name_taken_back.NAME_TAKEN_BACK_SAY"));
static NAME_TAKEN_BACK_BARE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.name_taken_back.NAME_TAKEN_BACK_BARE"));

/// Python: bot/modules/name_taken_back.py `_the_name_i_had(memory)`
///
/// The name the store holds for the person, or "" when it holds none.
pub fn _the_name_i_had(memory: &mut Memory) -> String {
    text::strip(&sget(user(memory), "name")).to_string()
}

/// Python: bot/modules/name_taken_back.py `_note(text)`
///
/// The line as the store keeps it for comparing one turn with the next.
pub fn _note(text: &str) -> String {
    fold(text)
}

/// Python: bot/modules/name_taken_back.py `observe(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// Take the name off the record, and leave the note the reply is written from. The name goes
/// here rather than in `respond` because the store keeps what a line says whatever its own reply
/// is.
pub fn observe(text: &str, memory: &mut Memory) {
    let known = _the_name_i_had(memory);
    let taken = crate::memory::facts::the_name_they_take_back(text, &known).val();
    if taken.is_none() {
        return;
    }
    user(memory).del_key("name");
    memory.set("name_taken_back", dict_of(vec![("name", Value::from(py_str(&taken))), ("said", Value::from(_note(text)))]));
    // The bot is about to ask for the name, so the store has to know it asked: a bare word on the
    // next line is the answer to this question and nothing else.
    memory.set("asked_for", "name");
}

/// Python: bot/modules/name_taken_back.py `_this_line(memory, text)`
///
/// The note left by `observe` when it was made on this line, else None.
pub fn _this_line(memory: &Memory, text: &str) -> Option<Value> {
    let held = mget(memory, "name_taken_back");
    if !held.is_dict() || vget(held, "said").as_str() != Some(_note(text).as_str()) {
        return None;
    }
    Some(held.clone())
}

/// Python: bot/modules/name_taken_back.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let held = _this_line(memory, text)?;
    let name = text::strip(&sget(&held, "name")).to_string();
    if name.is_empty() {
        return Some(pick(&NAME_TAKEN_BACK_BARE, memory, ""));
    }
    let lines: Vec<String> = NAME_TAKEN_BACK_SAY.iter().map(|line| fmt_named(line, &[("name", name.as_str())])).collect();
    Some(pick(&lines, memory, ""))
}

/// Python: bot/modules/name_taken_back.py `takes_as_it_comes(text, memory)` (also bound as
/// `TAKES_AS_IT_COMES`).
///
/// True only on a line that denies the name on record: nothing else has this answer.
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> bool {
    _this_line(memory, text).is_some()
}
