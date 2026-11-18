//! Port of `bot/memory/onset.py`: "it started when i moved house", "it started a month ago":
//! the time a thing began, read as the cause or the length of the thing just discussed, and
//! answered about the life event or the length rather than as a new subject.

use super::mem8_py::{int, pick, py_int_str, rx, strs, val};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

const M: &str = "bot.memory.onset.";

rx!(ONSET, LENGTH, BARE_UNIT);
// The per-cause pools are reached through BY_CAUSE (MOVED_REPLIES, SPLIT_REPLIES, MONEY_REPLIES)
// or not at all (JOB, BABY, DIED are left to other modules), as in the Python.
strs!(BODY, THE_PERSON, NAMED, OTHER_REPLIES, BODY_REPLIES, LENGTH_REPLIES, LEAVE_IT);
val!(CAUSES, NUMBER_WORDS, BY_CAUSE, UNIT_DAYS);
int!(NAMED_DAYS, LONG_ENOUGH);

/// `re.search(_pattern(TABLE), ...)`: the one-item tuples compiled once (no flags, as Python's).
static BODY_RE: LazyLock<Re> = LazyLock::new(|| Re::new(&_pattern(&BODY), 0));
static THE_PERSON_RE: LazyLock<Re> = LazyLock::new(|| Re::new(&_pattern(&THE_PERSON), 0));
static NAMED_RE: LazyLock<Re> = LazyLock::new(|| Re::new(&_pattern(&NAMED), 0));
static CAUSE_RES: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    CAUSES
        .items()
        .iter()
        .map(|pair| {
            let pair = pair.items();
            (pair[0].str().to_string(), Re::new(&_pattern(&pair[1].strs()), 0))
        })
        .collect()
});
static SPACES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+", 0));

/// `UNIT_DAYS.get(unit, 1)`.
fn unit_days(unit: &str) -> i64 {
    UNIT_DAYS.get(unit).and_then(|v| v.as_int()).unwrap_or(1)
}

/// Python: bot/memory/onset.py:241 `_clean(line)`
pub fn _clean(line: &str) -> String {
    text::strip(&line.to_lowercase()).to_string()
}

/// Python: bot/memory/onset.py:245 `_pattern(found)`
///
/// The regex itself: the tables are written as one-item tuples.
pub fn _pattern(found: &[String]) -> String {
    found[0].clone()
}

/// Python: bot/memory/onset.py:250 `_days(match)`
///
/// How long, in days, the length in the line is worth. (`found` is a match of LENGTH.)
pub fn _days(found: &crate::py::re::Match) -> Option<i64> {
    let word = SPACES.sub(" ", text::strip(found.str(1)));
    let count = match NUMBER_WORDS.get(&word).and_then(|v| v.as_int()) {
        Some(count) => count,
        None => py_int_str(&word)?,
    };
    Some(count * unit_days(found.str(2)))
}

/// Python: bot/memory/onset.py:262 `_clause(line)`
///
/// The words after the verb of beginning, or "" when the line has no onset frame.
pub fn _clause(line: &str) -> String {
    match ONSET.search(line) {
        Some(found) => text::strip_chars(&line[found.end()..], " .,!;:-").to_string(),
        None => String::new(),
    }
}

/// A length of time read off the clause: a bare unit, or a counted length.
fn the_length(clause: &str) -> Option<(String, Option<i64>)> {
    if let Some(unit) = BARE_UNIT.match_(clause) {
        return Some(("time".into(), Some(unit_days(unit.str(1)))));
    }
    if let Some(length) = LENGTH.search(clause) {
        if let Some(days) = _days(&length) {
            return Some(("time".into(), Some(days)));
        }
    }
    None
}

/// Python: bot/memory/onset.py:270 `reads_as_onset(line)`
///
/// (cause, days or None), the cause one of "move", "job", "baby", "died", "split", "money",
/// "time", "other"; None for every line that is not an onset statement.
pub fn reads_as_onset(line: &str) -> Option<(String, Option<i64>)> {
    let clean = _clean(line);
    if clean.is_empty() || clean.contains('?') {
        return None;
    }
    let clause = _clause(&clean);
    if clause.is_empty() {
        return None;
    }
    // The frame that matched on a bare count is no frame at all: a match that consumed nothing
    // is only ever a length of time.
    if let Some(found) = ONSET.search(&clean) {
        if found.end() == found.start() {
            return the_length(&clause);
        }
    }
    for (name, pattern) in CAUSE_RES.iter() {
        if pattern.is_match(&clause) {
            return Some((name.clone(), None));
        }
    }
    // Nothing named the cause: the clause is the length of time alone, or a time with a name.
    if NAMED_RE.is_match(&clean) {
        return Some(("time".into(), Some(*NAMED_DAYS)));
    }
    the_length(&clause)
}

/// `_clean((turns[-1] or {}).get("user"))`, or None without a turn.
fn last_user(memory: &Memory) -> Option<String> {
    memory.turns.last().map(|t| _clean(&t.user))
}

/// Python: bot/memory/onset.py:314 `_body_before(memory)`
///
/// True when the line before this one was about a body, not a life.
pub fn _body_before(memory: &Memory) -> bool {
    last_user(memory).is_some_and(|said| BODY_RE.is_match(&said))
}

/// Python: bot/memory/onset.py:326 `_the_person_before(memory)`
///
/// True when the line before this one was about the person themselves, not about a thing.
pub fn _the_person_before(memory: &Memory) -> bool {
    match last_user(memory) {
        Some(said) if !said.is_empty() => BODY_RE.is_match(&said) || THE_PERSON_RE.is_match(&said),
        _ => false,
    }
}

/// Python: bot/memory/onset.py:340 `_answered_before(memory)`
///
/// True when the bot has just answered something: a card, not a nod.
pub fn _answered_before(memory: &Memory) -> bool {
    memory.turns.last().is_some_and(|t| text::len(text::strip(&t.bot)) >= 60)
}

/// Python: bot/memory/onset.py:352 `reply(line, memory)`
///
/// The answer to an onset statement about the thing just discussed, or None to pass.
pub fn reply(line: &str, memory: &Memory) -> Option<String> {
    let (cause, days) = reads_as_onset(line)?;
    if !_answered_before(memory) {
        // Nothing was being discussed: the modules for the line itself decide what it is.
        return None;
    }
    if LEAVE_IT.contains(&cause) {
        return None;
    }
    if _body_before(memory) {
        // A move that hurt their back is a strain rather than a life event.
        return if cause != "time" { Some(pick(&BODY_REPLIES, memory, line)) } else { None };
    }
    if cause == "time" {
        if days.is_none_or(|d| d < *LONG_ENOUGH) {
            return None; // "it started yesterday" needs no plan; the card already said what to do
        }
        if !(_body_before(memory) || _the_person_before(memory)) {
            // The length is only a length: the reply here is written for a person's own state.
            return None;
        }
        return Some(pick(&LENGTH_REPLIES, memory, line));
    }
    let pool = match BY_CAUSE.get(&cause) {
        Some(p) if p.truthy() => p.strs(),
        _ => OTHER_REPLIES.clone(),
    };
    Some(pick(&pool, memory, line))
}
