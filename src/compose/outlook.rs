//! Port of `bot/compose/outlook.py`: "will it get better" pointed at the thread above, and the
//! pools for the threads a forecast line does not fit (a crisis, an emergency, abuse).

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.compose.outlook.";
static _WORDS: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}_WORDS")));
static _POINTS_AT_THE_THREAD: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}_POINTS_AT_THE_THREAD")));
static _OUTCOME: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}_OUTCOME")));
static _THE_THING_ITSELF: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}_THE_THING_ITSELF")));
fn pool(name: &str) -> Value {
    data::get(&format!("{M}{name}")).clone()
}

/// Python: bot/compose/outlook.py:105 `points_at_the_thread(text)`
///
/// True for a short line whose whole subject is a pronoun standing for the thread above.
pub fn points_at_the_thread(text: &str) -> bool {
    let clean = text::squash(&text::lower(text));
    if clean.is_empty() || text::split_ref(&clean).len() > 12 {
        return false;
    }
    _POINTS_AT_THE_THREAD.match_(&clean).is_some()
}

/// Python: bot/compose/outlook.py:113 `asks_the_outlook(text, clean=None)`
///
/// True when the line asks how the thing turns out, pointing at it with a pronoun. The
/// contract's `clean` is a str; every caller passes the normalised line, which Python reads in
/// place of `text` (so `text` is not read here).
pub fn asks_the_outlook(text: &str, clean: &str) -> bool {
    let said = text::squash(&text::lower(clean));
    if said.is_empty() || text::split_ref(&said).len() > 9 {
        return false;
    }
    if !_OUTCOME.is_match(&said) {
        return false;
    }
    if !_THE_THING_ITSELF.is_empty() && _WORDS.findall_str(&said).iter().any(|w| _THE_THING_ITSELF.contains(w)) {
        return false;
    }
    _POINTS_AT_THE_THREAD.match_(&said).is_some()
}

/// Python: bot/compose/outlook.py:127 `the_pool(in_crisis, help_is_coming, asked_them_to_call=False, abuse=False)`
///
/// The pool (a tuple of lines) for the threads a forecast line does not fit, or None.
pub fn the_pool(in_crisis: &Value, help_is_coming: &Value, asked_them_to_call: &Value, abuse: &Value) -> Value {
    if abuse.truthy() {
        return pool("LINES_WHEN_SOMEONE_IS_HURTING_YOU");
    }
    if in_crisis.truthy() {
        return pool(if help_is_coming.truthy() { "LINES_WITH_HELP" } else { "LINES_IN_CRISIS" });
    }
    if asked_them_to_call.truthy() {
        return pool(if help_is_coming.truthy() { "LINES_WITH_HELP" } else { "LINES_IN_AN_EMERGENCY" });
    }
    Value::None
}
