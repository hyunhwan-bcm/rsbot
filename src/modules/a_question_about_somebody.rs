//! Port of `bot/modules/a_question_about_somebody.py`: an ask about a third party the chat has never been told about is answered by asking who they mean.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

macro_rules! rx {
    ($($name:ident),*) => {$(
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.a_question_about_somebody.", stringify!($name))));
    )*};
}
rx!(A_THIRD_PARTY, SOMEBODY_IN_THE_LINE, NAMES_THEM, A_FACT_ABOUT_THEM, AN_ASK);
static WHO_IS_IT: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.a_question_about_somebody.WHO_IS_IT"));

/// Python: bot/modules/a_question_about_somebody.py `_there_is_a_chat(memory)`
///
/// Whether the person said anything before this line.
pub fn _there_is_a_chat(memory: &Memory) -> bool {
    memory.turns.iter().any(|turn| !text::strip(&turn.user).is_empty())
}

/// Python: bot/modules/a_question_about_somebody.py `_flat(text)`
///
/// The line with its case and spacing taken off.
pub fn _flat(text: &str) -> String {
    text::squash(&text.to_lowercase().replace('\u{2019}', "'"))
}

/// Python: bot/modules/a_question_about_somebody.py `about_somebody_unnamed(text, memory)`
///
/// Is this an ask about a third party the chat has never been told about?
pub fn about_somebody_unnamed(text: &str, memory: &Memory) -> bool {
    if _there_is_a_chat(memory) {
        return false;
    }
    let said = _flat(text);
    if said.is_empty() {
        return false;
    }
    if !AN_ASK.is_match(&said) && !said.contains('?') {
        return false;
    }
    if !A_THIRD_PARTY.is_match(&said) || !A_FACT_ABOUT_THEM.is_match(&said) {
        return false;
    }
    !SOMEBODY_IN_THE_LINE.is_match(&said) && !NAMES_THEM.is_match(&said)
}

/// Python: bot/modules/a_question_about_somebody.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// Asking who they mean, or None for every other line.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    if !about_somebody_unnamed(text, memory) {
        return None;
    }
    // a pick that fails still has a wording
    match py_try(|| pick(&WHO_IS_IT, memory, &_flat(text))) {
        Ok(line) => Some(line),
        Err(()) => Some(WHO_IS_IT[0].clone()),
    }
}
