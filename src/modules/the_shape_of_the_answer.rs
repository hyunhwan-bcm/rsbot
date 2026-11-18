//! Port of `bot/modules/the_shape_of_the_answer.py`: the shape of an answer the bot cannot give the fact of, said where the floor would only hedge (`bot/compose/the_shape_without_the_fact.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::compose::{fallback, fallback_chain};

/// `_A_HEADS`: how much of a hedge's opening is compared.
const _A_HEADS: isize = 40;
/// `fallback._thread_is_about_the_body`'s own default for `look_back`.
const THE_BODY_LOOK_BACK: i64 = 4;

/// Python: bot/modules/the_shape_of_the_answer.py `_the_reply_is_only_a_placeholder(reply)`
pub fn _the_reply_is_only_a_placeholder(reply: &str) -> bool {
    let said = fold(reply);
    if said.is_empty() {
        return false;
    }
    let neutral = Value::Tuple(Arc::new(vec![data::get("bot.compose.fallback.NEUTRAL").clone()]));
    if ret_true(fallback::_in_a_pool(reply, &neutral)) {
        return true;
    }
    let head = sl(&said, 0, Some(_A_HEADS));
    data::strs("bot.compose.fallback.UNKNOWN").iter().any(|line| sl(&line.to_lowercase(), 0, Some(_A_HEADS)) == head)
}

/// Python: bot/modules/the_shape_of_the_answer.py `_the_composer_would_hedge(text, memory)`
///
/// True when the composer's own line for this turn is a placeholder, not an answer.
pub fn _the_composer_would_hedge(text: &str, memory: &mut Memory) -> bool {
    // a composer that cannot be asked claims the turn
    let Ok(reply) = py_try(|| fallback_chain::reply_for(text, memory, true).val()) else { return false };
    _the_reply_is_only_a_placeholder(&s_or(Some(&reply)))
}

/// Python: bot/modules/the_shape_of_the_answer.py `_the_floor_would_say_a_placeholder(text, memory)`
///
/// True when `greeting` - the module that says the hedge - would say a placeholder this turn. The
/// module that would otherwise answer is asked, on a copy of memory, and its own line decides.
pub fn _the_floor_would_say_a_placeholder(text: &str, memory: &Memory) -> bool {
    // a floor that cannot be asked does not open the turn
    let Ok(reply) = py_try(|| super::greeting::respond(text, &mut _a_copy_of(memory))) else { return false };
    _the_reply_is_only_a_placeholder(&reply.unwrap_or_default())
}

/// Python: bot/modules/the_shape_of_the_answer.py `_a_copy_of(memory)`
///
/// Memory as a module may read it: everything copied, the turns shared, as the runtime does (the
/// turns are never written by a module, so a plain copy is the same thing here).
pub fn _a_copy_of(memory: &Memory) -> Memory {
    memory.clone()
}

/// Python: bot/modules/the_shape_of_the_answer.py `_a_thread_with_a_person_in_trouble(memory)`
///
/// A worry about somebody's health, a death, a crisis, or a line about the body. A reader that
/// cannot be asked does not fence.
pub fn _a_thread_with_a_person_in_trouble(memory: &Memory) -> bool {
    let readers: [&dyn Fn() -> bool; 4] = [
        &|| ret_true(fallback::_in_a_care_thread(memory)),
        &|| ret_true(fallback::_in_a_loss_thread(memory)),
        &|| ret_true(fallback::_in_a_crisis_thread(memory)),
        // `_thread_is_about_the_body(memory)`: `look_back` at its default
        &|| ret_true(fallback::_thread_is_about_the_body(memory, THE_BODY_LOOK_BACK)),
    ];
    readers.iter().any(|reader| py_try(|| reader()).unwrap_or(false))
}

/// Python: bot/modules/the_shape_of_the_answer.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let understood = memory.understood.clone()?;
    // The reader comes first because it is cheap and says no to almost every line: the composer is
    // only asked when this line has a shape the bot could say something true about.
    let pool = match crate::compose::the_shape_without_the_fact::the_shape_without_the_fact(&understood, memory).val() {
        Value::None => return None,
        pool => py_strs(&pool),
    };
    if _a_thread_with_a_person_in_trouble(memory) {
        return None;
    }
    if !_the_composer_would_hedge(text, memory) {
        return None;
    }
    if !_the_floor_would_say_a_placeholder(text, memory) {
        return None;
    }
    let clean = if !understood.clean.is_empty() { understood.clean.clone() } else { fold(text) };
    as_reply(fallback_chain::_choose(&pool, memory, &clean, None))
}
