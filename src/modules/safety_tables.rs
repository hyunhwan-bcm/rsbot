//! Port of `bot/modules/safety_tables.py`: the pattern tables `bot/modules/safety.py` reads (the tables themselves are in the exported bundle).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/safety_tables.py:7662 `_with_streak_fever(pats)`
///
/// The same streak shapes, read only when the line also has a fever in it.
pub fn _with_streak_fever(pats: &Value) -> Value {
    let extra = format!(r"(?=[\s\S]*(?:{}))", data::strs("bot.modules.safety_tables.STREAK_FEVER").join("|"));
    Value::Tuple(Arc::new(pats.items().iter().map(|p| Value::from(format!("{}{}", py_str(p), extra))).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streak_fever_is_the_exported_table() {
        // WOUND_STREAK_FEVER = _with_streak_fever(WOUND_STREAK), computed at import in Python
        let built = _with_streak_fever(data::get("bot.modules.safety_tables.WOUND_STREAK"));
        assert_eq!(&built, data::get("bot.modules.safety_tables.WOUND_STREAK_FEVER"));
    }
}
