//! Helpers the g8 modules (consumer, plans, cleaning, lifeadmin) need from code they do not own,
//! and a cache for regexes PyBot builds on the fly. Every port of someone else's function is
//! marked `DUP(...)`, `DUP-D2(...)` or `NEEDS(...)` for the integrator.

use crate::data;
use crate::py::re::{escape, Re};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

// ---------------------------------------------------------------------------------------------
// Regexes built at run time

static CACHE: LazyLock<Mutex<HashMap<(String, u32), &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// `re.compile(pattern, flags)` for a pattern PyBot builds from data at run time, built once.
pub fn cached_re(pattern: &str, flags: u32) -> &'static Re {
    let key = (pattern.to_string(), flags);
    if let Some(r) = CACHE.lock().unwrap().get(&key) {
        return r;
    }
    let r: &'static Re = Box::leak(Box::new(Re::new(pattern, flags)));
    CACHE.lock().unwrap().insert(key, r);
    r
}

/// `re.search(r"\b" + re.escape(word) + r"\b", text)`.
pub fn has_bounded_word(text: &str, word: &str) -> bool {
    cached_re(&format!("\\b{}\\b", escape(word)), 0).is_match(text)
}

/// `re.search(r"(?<![a-z0-9])" + re.escape(word) + r"(?![a-z0-9])", text)`.
pub fn has_fenced_word(text: &str, word: &str) -> bool {
    cached_re(&format!("(?<![a-z0-9]){}(?![a-z0-9])", escape(word)), 0).is_match(text)
}

// ---------------------------------------------------------------------------------------------
// DUP-D2(bot.knowledge.cooking::without) and the helpers it runs

macro_rules! cooking {
    ($t:ty, $name:ident, $f:path) => {
        static $name: LazyLock<$t> = LazyLock::new(|| $f(concat!("bot.knowledge.cooking.", stringify!($name))));
    };
}
cooking!(String, _GRIDDLE_TAIL, data::string);

// was DUP-D2(bot.knowledge.cooking::_line): now the real port
pub use crate::knowledge::cooking::_line as cooking_line;

// was DUP-D2(bot.knowledge.cooking::missing): now the real port
pub use crate::knowledge::cooking::missing as cooking_missing;

// was DUP-D2(bot.knowledge.cooking::available): now the real port
pub use crate::knowledge::cooking::available as cooking_available;

// was DUP-D2(bot.knowledge.cooking::without): now the real port
pub use crate::knowledge::cooking::without as cooking_without;

// ---------------------------------------------------------------------------------------------
// DUP(bot.knowledge.dishes::a_dish_is_already_the_plan)

static PLAN_MARKER: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.dishes._PLAN_MARKER"));
static IDEA_ASK: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.dishes._IDEA_ASK"));

// NEEDS(bot.knowledge.dishes::dish_in)
/// The plan key a piece of text names, or None. Needs D1's `dishes._named` (the dish-name
/// matcher, with `understanding.topics.says_another_thing` and lookup's typo rules); until that
/// is merged it names nothing.
pub fn dish_in(text: &str) -> Option<String> {
    crate::knowledge::dishes::dish_in(text).map(str::to_string)
}

// DUP(bot.knowledge.dishes::a_dish_is_already_the_plan)
/// True when the line names a dish and says that is what is being cooked.
pub fn a_dish_is_already_the_plan(text: &str) -> bool {
    let text = text.to_lowercase();
    // `dish_in` is free of side effects, so the cheap tests go first
    if text.is_empty() {
        return false;
    }
    if !PLAN_MARKER.iter().any(|m| text.contains(m.as_str())) {
        return false;
    }
    if IDEA_ASK.iter().any(|p| text.contains(p.as_str())) {
        return false;
    }
    dish_in(&text).is_some()
}
