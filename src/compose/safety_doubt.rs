//! Port of `bot/compose/safety_doubt.py`: "is it serious?" right after a safety answer that said
//! to ring somebody gets the answer's own yes, not a hedge.

use crate::compose::fallback;
use crate::data::{self, Value};
use crate::py::random::{ord_sum, Random};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.compose.safety_doubt.";
static YES_LINES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}YES_LINES")));
static AFTER_THE_CALL_LINES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}AFTER_THE_CALL_LINES")));
static _CONTINUATION: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}_CONTINUATION")));
static _EMERGENCY_CASES: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}_EMERGENCY_CASES")));
static _CALLS_SOMEBODY: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}_CALLS_SOMEBODY")));
static _HELP_IS_COMING: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}_HELP_IS_COMING")));
static _AGREES_IT_IS_BAD: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}_AGREES_IT_IS_BAD")));

/// Python: bot/compose/safety_doubt.py:62 `_the_answer_behind(memory)`
///
/// The safety answer the doubt is about (lowercased) and what was said after it, or
/// (None, []) - only this file calls it.
pub fn _the_answer_behind(memory: &Memory) -> (Option<String>, Vec<String>) {
    let turns = &memory.turns[memory.turns.len().saturating_sub(6)..];
    let mut after = Vec::new();
    for turn in turns.iter().rev() {
        let module = turn.module.as_deref();
        if module == Some("safety") {
            after.push(text::lower(&turn.user));
            return (Some(text::lower(&turn.bot)), after);
        }
        if !module.is_some_and(|m| _CONTINUATION.iter().any(|c| c == m)) {
            return (None, Vec::new());
        }
        after.push(text::lower(&format!("{} {}", turn.user, turn.bot)));
    }
    (None, Vec::new())
}

/// Python: bot/compose/safety_doubt.py:93 `_a_bare_verdict_ask(text, memory, clean)`
pub fn _a_bare_verdict_ask(text: &str, memory: &Memory, clean: &str) -> bool {
    if memory.understood.as_ref().is_some_and(|read| read.asks_a_verdict) {
        return true;
    }
    if _AGREES_IT_IS_BAD.match_(clean).is_some() {
        return true;
    }
    fallback::reads_as_a_verdict(text, memory)
}

/// Python: bot/compose/safety_doubt.py:102 `reply_for(text, memory)`
///
/// The answer's own yes to a bare doubt in a safety thread, or None.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&text::lower(text));
    if clean.is_empty() {
        return None;
    }
    let (behind, after) = _the_answer_behind(memory);
    let behind = behind.filter(|b| !b.is_empty())?;
    let case = memory.get("safety_case").and_then(Value::as_str);
    if !case.is_some_and(|c| _EMERGENCY_CASES.contains(c)) {
        return None;
    }
    if matches!(memory.get("safety_safe"), Some(Value::Bool(true))) {
        return None;
    }
    if !_CALLS_SOMEBODY.is_match(&behind) {
        return None;
    }
    if !_a_bare_verdict_ask(text, memory, &clean) {
        return None;
    }
    let pool = if after.iter().any(|said| _HELP_IS_COMING.is_match(said)) { &*AFTER_THE_CALL_LINES } else { &*YES_LINES };
    let recent: Vec<&str> = memory.turns[memory.turns.len().saturating_sub(5)..].iter().map(|t| t.bot.as_str()).collect();
    let mut options: Vec<&String> = pool.iter().filter(|line| !recent.contains(&line.as_str())).collect();
    if options.is_empty() {
        options = pool.iter().collect();
    }
    let seed = memory.turns.len() as i128 * 7919 + ord_sum(&format!("{clean}{behind}"));
    Some((*Random::new(seed).choice(&options)).clone())
}
