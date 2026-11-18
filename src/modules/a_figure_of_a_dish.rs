//! Port of `bot/modules/a_figure_of_a_dish.py`: a figure asked of a dish they made: the recipe and the portion set it, not a general figure.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

static A_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.a_figure_of_a_dish.A_FIGURE"));
static DISH: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.a_figure_of_a_dish.DISH"));
static THEIRS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.a_figure_of_a_dish.THEIRS"));
static POOL: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.a_figure_of_a_dish.POOL"));

/// Python: bot/modules/a_figure_of_a_dish.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let clean = fold(text);
    if clean.is_empty() {
        return None;
    }
    if !A_FIGURE.is_match(&clean) {
        return None;
    }
    if !(DISH.is_match(&clean) || THEIRS.is_match(&clean)) {
        return None;
    }
    Some(POOL[0].clone())
}
