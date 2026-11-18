//! Port of `bot/compose/pleased.py`: a line of plain pleasure ("what a lovely day", "what a
//! relief", "great news") answered in kind.

use crate::data;
use crate::memory::narrow::memory_value;
use crate::py::re::{Match, Re};
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.compose.pleased.";
static OUTSIDE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}OUTSIDE")));
static NOT_GOOD: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}NOT_GOOD")));
static WHAT_A: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}WHAT_A")));
static RELIEF_LINE: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}RELIEF_LINE")));
static RESULT_LINE: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}RESULT_LINE")));
static NEWS_LINE: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}NEWS_LINE")));
static PLEASED: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}PLEASED")));
static OUT_IN_IT: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}OUT_IN_IT")));
static RELIEF: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}RELIEF")));
static NEWS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}NEWS")));
static RESULT: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}RESULT")));

/// `from . import pick`: its contract takes the memory as a `Value`.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, &memory_value(memory), seed_text)
}

/// Python: bot/compose/pleased.py:64 `_thing(match)`: only this file calls it, with the match.
pub fn _thing(found: &Match) -> String {
    let thing = found.name("thing").filter(|t| !t.is_empty()).or_else(|| found.name("thing2")).unwrap_or("");
    text::strip(thing).to_string()
}

/// Python: bot/compose/pleased.py:68 `answer_for(text, memory=None)`
pub fn answer_for(text: &str, memory: &Memory) -> Option<String> {
    let squashed = text::squash(&text::lower(text));
    let line = text::rstrip_chars(&squashed, " .!,;:");
    if line.is_empty() || text::split_ref(line).len() > 6 {
        return None;
    }
    if line.contains('?') {
        return None;
    }
    if RELIEF_LINE.match_(line).is_some() {
        return Some(pick(&RELIEF, memory, line));
    }
    if RESULT_LINE.match_(line).is_some() {
        return Some(pick(&RESULT, memory, line));
    }
    if NEWS_LINE.match_(line).is_some() {
        return Some(pick(&NEWS, memory, line));
    }
    let found = WHAT_A.match_(line)?;
    let thing = _thing(&found);
    let words: HashSet<&str> = text::split_ref(&thing).into_iter().collect();
    if thing.is_empty() || words.iter().any(|w| NOT_GOOD.contains(*w)) {
        return None;
    }
    let pool = if words.iter().any(|w| OUTSIDE.contains(*w)) { &*OUT_IN_IT } else { &*PLEASED };
    Some(pick(pool, memory, line))
}
