//! Port of `bot/compose/their_life.py`: "do i like it", "what does my son do", "how long will
//! she be in" - a particular of the person's own life gets the honest line and one question.

use crate::data;
use crate::py::re::{Match, Re, I};
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.compose.their_life.";
macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.their_life.", stringify!($name))));)*
    };
}
regexes!(
    TASTE, AN_AGE_NAME_OR_PLACE, WORK, ABOUT_THEM, ALREADY_SAID, A_NEED_OF_THEIRS, STAY, MISSING,
    WHERE_THEY_ARE, A_PRONOUN, SAID_WHO, NOT_THIS, RECALL, A_HOW_TO,
);
static TOO_LONG: LazyLock<usize> = LazyLock::new(|| data::get(&format!("{M}TOO_LONG")).as_int().unwrap() as usize);
static STOP_ABOUT: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}STOP_ABOUT")));
static PROSE: LazyLock<&'static crate::data::Value> = LazyLock::new(|| data::get(&format!("{M}PROSE")));
/// `re.search(r"\b(?:my|our|his|her|their)\s+" + RELATION, line, re.I)`, written inline in `reads`.
static A_RELATION_NAMED: LazyLock<Re> =
    LazyLock::new(|| Re::new(&format!(r"\b(?:my|our|his|her|their)\s+{}", data::string(&format!("{M}RELATION"))), I));
/// `re.sub(r"\s+", " ", ...)`, written inline in `_bare`.
static SPACES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+", 0));

/// Python: bot/compose/their_life.py:130 `_bare(text)`
pub fn _bare(text: &str) -> String {
    let squashed = SPACES.sub(" ", text::strip(text));
    text::lower(text::strip_chars(&squashed, " .!,"))
}

/// Python: bot/compose/their_life.py:134 `_a_thing_already_said(match, memory)`
///
/// True when the noun the question turns on was said by the person earlier in the chat (only
/// this file calls it, with the `ABOUT_THEM` match).
pub fn _a_thing_already_said(found: &Match, memory: &Memory) -> bool {
    let about = text::lower(text::strip(found.name("about").unwrap_or("")));
    let words: Vec<String> = ALREADY_SAID.findall_str(&about).into_iter().filter(|w| !STOP_ABOUT.contains(w)).collect();
    if words.is_empty() {
        return false;
    }
    let turns: Vec<String> = memory.turns.iter().map(|t| text::lower(&t.user)).collect();
    let earlier = turns[..turns.len().saturating_sub(1)].join(" ");
    words.iter().any(|w| earlier.contains(w.as_str()))
}

/// Python: bot/compose/their_life.py:146 `_somebody_of_theirs(memory)`
///
/// "she", "he" and "they" are theirs when the last few turns named somebody they know.
pub fn _somebody_of_theirs(memory: &Memory) -> bool {
    memory.turns[memory.turns.len().saturating_sub(6)..].iter().any(|t| SAID_WHO.is_match(&t.user))
}

/// Python: bot/compose/their_life.py:152 `reads(text, memory=None)`
///
/// The kind of particular this is - "taste", "work", "stay", "theirs" - or None.
pub fn reads(text: &str, memory: &Memory) -> Option<String> {
    let line = _bare(text);
    if line.is_empty() {
        return None;
    }
    if text::split_ref(&line).len() > *TOO_LONG {
        return None;
    }
    if NOT_THIS.is_match(&line) || RECALL.is_match(&line) || A_HOW_TO.is_match(&line) {
        return None;
    }
    if AN_AGE_NAME_OR_PLACE.is_match(&line) {
        return None;
    }
    if MISSING.is_match(&line) || WHERE_THEY_ARE.is_match(&line) {
        return None;
    }
    if A_NEED_OF_THEIRS.is_match(&line) {
        return None;
    }
    if TASTE.is_match(&line) {
        return Some("taste".into());
    }
    let kinds: [(&str, &Re); 3] = [("work", &WORK), ("stay", &STAY), ("theirs", &ABOUT_THEM)];
    for (kind, pattern) in kinds {
        let Some(found) = pattern.search(&line) else { continue };
        if kind == "theirs" && _a_thing_already_said(&found, memory) {
            return None;
        }
        if A_PRONOUN.is_match(&line) && !A_RELATION_NAMED.is_match(&line) && !_somebody_of_theirs(memory) {
            return None;
        }
        return Some(kind.to_string());
    }
    None
}

/// Python: bot/compose/their_life.py:178 `reply_for(text, memory)`
///
/// The honest line and the one question, or None when this is not the ask.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    let kind = reads(text, memory).filter(|k| !k.is_empty())?;
    let lines = PROSE.get(&kind).expect("KeyError").strs();
    Some(lines[memory.turns.len() % lines.len()].clone())
}
