//! Port of `bot/modules/figure_asked_again.py`: a yes/no about a figure the bot gave (`bot/memory/stands.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::memory::stands;

/// Python: bot/modules/figure_asked_again.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The reply to a yes/no about a figure the bot gave, or None for every other line. Two shapes: a
/// figure in the line, and a change with no figure in it.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // `stands.reply_for(text, memory) or stands.change_reply(text, memory)`
    match or_none(stands::reply_for(text, memory)) {
        Some(reply) => Some(reply),
        None => as_reply(stands::change_reply(text, memory)),
    }
}

/// Python: bot/modules/figure_asked_again.py `takes_as_it_comes(text, memory)` (also bound as
/// `TAKES_AS_IT_COMES`).
///
/// Only on a yes/no about a figure of this chat's own answers, and nowhere else.
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> Option<bool> {
    // a hook that raises claims nothing
    Some(or_on_raise(|| ret_true(stands::the_figure_asked_about(text, memory)) || ret_true(stands::the_change_asked_about(text, memory)), false))
}
