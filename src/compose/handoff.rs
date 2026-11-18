//! Port of `bot/compose/handoff.py`: a hand-off with nothing said after it ("one more thing",
//! "quick question") is asked to go on; one that names its subject is asked about the subject.

use crate::compose::fallback_chain::py_format;
use crate::data;
use crate::memory::narrow::memory_value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.compose.handoff.";
static HANDOFF: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}HANDOFF")));
static ABOUT: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}ABOUT")));
static _NAMES_A_QUESTION: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}_NAMES_A_QUESTION")));
static _POINTS_AT_NOTHING: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}_POINTS_AT_NOTHING")));
static _NOT_A_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}_NOT_A_SUBJECT")));
static ABOUT_THE_THING: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}ABOUT_THE_THING")));
static ASK_AWAY: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}ASK_AWAY")));
static GO_ON: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}GO_ON")));

/// `from . import pick`: its contract takes the memory as a `Value`.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, &memory_value(memory), seed_text)
}

/// Python: bot/compose/handoff.py:82 `reads_as_handoff_about_a_thing(text)`
///
/// The subject a hand-off names ("one more thing about the boiler"), or None.
pub fn reads_as_handoff_about_a_thing(text: &str) -> Option<String> {
    let clean = text::squash(&text::lower(text));
    let found = ABOUT.match_(&clean)?;
    let thing = text::strip_chars(found.name_str("thing"), " ,.?!:-");
    let first = text::split_ref(thing).first().copied().unwrap_or("");
    if thing.is_empty() || _POINTS_AT_NOTHING.contains(first) {
        return None;
    }
    if _POINTS_AT_NOTHING.contains(thing) || _NOT_A_SUBJECT.contains(first) {
        return None;
    }
    Some(thing.to_string())
}

/// Python: bot/compose/handoff.py:125 `reads_as_handoff(text)`
///
/// Is the whole line a hand-off with nothing said after it?
pub fn reads_as_handoff(text: &str) -> bool {
    HANDOFF.match_(&text::squash(&text::lower(text))).is_some()
}

/// Python: bot/compose/handoff.py:130 `takes_as_it_comes(text, memory)`
///
/// Read by the arbiter: yes to every bare hand-off and to one that names its subject.
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> bool {
    reads_as_handoff(text) || reads_as_handoff_about_a_thing(text).is_some_and(|t| !t.is_empty())
}

/// Python: bot/compose/handoff.py:139 `reply_for(text, memory)`
///
/// The reply to a hand-off, or None when the line says something after it.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&text::lower(text));
    if let Some(thing) = reads_as_handoff_about_a_thing(&clean).filter(|t| !t.is_empty()) {
        let mut chars = thing.chars();
        let first = chars.next().map(|c| c.to_uppercase().collect::<String>()).unwrap_or_default();
        let upper = first + chars.as_str();
        let lines: Vec<String> =
            ABOUT_THE_THING.iter().map(|line| py_format(line, &[("thing", &thing), ("Thing", &upper)])).collect();
        return Some(pick(&lines, memory, &clean));
    }
    if HANDOFF.match_(&clean).is_none() {
        return None;
    }
    let pool = if _NAMES_A_QUESTION.is_match(&clean) { &*ASK_AWAY } else { &*GO_ON };
    Some(pick(pool, memory, &clean))
}
