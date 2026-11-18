//! Port of `bot/modules/a_cake_between_its_layers.py`: what goes between the layers of a cake, where the row the line reaches answers something else.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::knowledge::entry::Entry;
use crate::understanding::understand;
use crate::understood::Understood;

static A_CAKE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.a_cake_between_its_layers.A_CAKE"));
static BETWEEN: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.a_cake_between_its_layers.BETWEEN"));
static NOT_ASKING: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.a_cake_between_its_layers.NOT_ASKING"));
static NOT_THE_CAKE_ROW: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.modules.a_cake_between_its_layers.NOT_THE_CAKE_ROW"));
static THE_FILLING: LazyLock<String> = LazyLock::new(|| data::string("bot.modules.a_cake_between_its_layers.THE_FILLING"));

/// Python: bot/modules/a_cake_between_its_layers.py `asks_about_its_layers(said)`
///
/// True when the line asks where the filling or the split goes in a cake.
pub fn asks_about_its_layers(said: &Understood) -> bool {
    if !["question", "request", ""].contains(&said.kind.as_str()) {
        return false;
    }
    let mut line = format!("{} {}", said.clean, said.text);
    line.push(' ');
    line.push_str(&said.words.join(" "));
    if text::strip(&line).is_empty() {
        return false;
    }
    A_CAKE.is_match(&line) && BETWEEN.is_match(&line) && !NOT_ASKING.is_match(&line)
}

/// Python: bot/modules/a_cake_between_its_layers.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let said = understand(text, Some(&*memory));
    if !asks_about_its_layers(&said) {
        return None;
    }
    if let Some(row) = _the_row(text, memory) {
        if !NOT_THE_CAKE_ROW.contains(&row.id) {
            return None;
        }
    }
    Some(THE_FILLING.clone())
}

/// Python: bot/modules/a_cake_between_its_layers.py `_the_row(text, memory)`: the row the
/// knowledge base reaches on this line, or None (the matcher must never crash a turn).
pub fn _the_row(text: &str, memory: &Memory) -> Option<&'static Entry> {
    py_try(|| crate::knowledge::lookup::find(&understand(text, Some(memory)), None, None, false)).unwrap_or(None)
}
