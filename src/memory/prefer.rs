//! Port of `bot/memory/prefer.py`: a taste said after a list of picks ("i like crime", "in the
//! mood for something light") written out as the ask it is ("any good crime books"), with the
//! medium read off the turn before.

use super::mem8_py::{int, rx, strs, val};
use crate::py::text;
use crate::state::Memory;

const M: &str = "bot.memory.prefer.";

rx!(TASTE, WANTS, SOMETHING, _VAGUE_LEAD, _ASKS_SOMETHING, REFUSED, _TITLE, _LEAD);
val!(MEDIUM, MEDIUM_WORDS);
strs!(POINTS_BACK);
int!(MOST_WORDS);

/// Python: bot/memory/prefer.py:79 `_offered_choices(reply)`
///
/// Whether the answer offered a list of picks (two titles at least).
pub fn _offered_choices(reply: &str) -> bool {
    _TITLE.finditer(reply).len() >= 2
}

/// Python: bot/memory/prefer.py:84 `_said_the_medium(said)`
///
/// The medium ("books", "films") a line names first, or "".
pub fn _said_the_medium(said: &str) -> String {
    let low = text::squash(&said.to_lowercase());
    if low.is_empty() {
        return String::new();
    }
    let mut best = String::new();
    let mut at: Option<usize> = None;
    for pair in MEDIUM.items() {
        let pair = pair.items();
        if let Some(found) = pair[1].as_re().expect("MEDIUM pattern").search(&low) {
            if at.is_none_or(|a| found.start() < a) {
                best = pair[0].str().to_string();
                at = Some(found.start());
            }
        }
    }
    best
}

/// Python: bot/memory/prefer.py:97 `_last_turn(memory)`
///
/// (what the person said last, what the bot said back), or ("", "").
pub fn _last_turn(memory: &Memory) -> (String, String) {
    match memory.turns.last() {
        Some(t) => (t.user.clone(), t.bot.clone()),
        None => (String::new(), String::new()),
    }
}

/// Python: bot/memory/prefer.py:105 `_taste(text)`
///
/// The taste the whole line is, or "".
pub fn _taste(text: &str) -> String {
    let squashed = text::squash(&text.to_lowercase());
    let said = text::strip_chars(&squashed, " .,!?");
    if said.is_empty() || text::split_ref(said).len() > 8 {
        return String::new();
    }
    if _ASKS_SOMETHING.is_match(said) {
        return String::new();
    }
    let found = TASTE.match_(said).or_else(|| WANTS.match_(said)).or_else(|| SOMETHING.match_(said));
    let Some(found) = found else { return String::new() };
    let mut taste = text::strip_chars(found.str(1), " .,!?").to_string();
    if REFUSED.is_match(said) {
        return String::new();
    }
    while _LEAD.match_(&taste).is_some() {
        taste = _LEAD.sub("", &taste);
    }
    while _VAGUE_LEAD.match_(&taste).is_some() {
        taste = _VAGUE_LEAD.sub("", &taste);
    }
    let words = text::split_ref(&taste);
    if words.is_empty() || words.len() as i64 > *MOST_WORDS {
        return String::new();
    }
    if words.iter().any(|w| POINTS_BACK.iter().any(|p| p == w)) {
        return String::new();
    }
    // "i like crime books" already names the medium, so there is nothing to carry.
    if !_said_the_medium(&taste).is_empty() {
        return String::new();
    }
    taste
}

/// Python: bot/memory/prefer.py:148 `about_the_medium(answer, memory)`
///
/// Whether the answer a taste bought is about the medium the taste was put in. True when there
/// is no medium to check against.
pub fn about_the_medium(answer: &str, memory: &Memory) -> bool {
    let (asked, answered) = _last_turn(memory);
    let mut medium = _said_the_medium(&asked);
    if medium.is_empty() {
        medium = _said_the_medium(&answered);
    }
    let pattern = if medium.is_empty() { None } else { MEDIUM_WORDS.get(&medium).and_then(|v| v.as_re()) };
    let Some(pattern) = pattern else { return true };
    _TITLE.is_match(answer) || pattern.is_match(answer)
}

/// Python: bot/memory/prefer.py:162 `served_already(text, memory)`
///
/// Whether the taste is the one the turn before was served with its list already.
pub fn served_already(text: &str, memory: &Memory) -> bool {
    let taste = _taste(text);
    if taste.is_empty() {
        return false;
    }
    let (asked, answered) = _last_turn(memory);
    _taste(&asked) == taste && _offered_choices(&answered)
}

/// Python: bot/memory/prefer.py:177 `follow_up(text, memory)`
///
/// "any good <taste> <medium>" for a taste said after a list of picks, or "".
pub fn follow_up(text: &str, memory: &Memory) -> String {
    let taste = _taste(text);
    if taste.is_empty() {
        return String::new();
    }
    if served_already(text, memory) {
        return String::new();
    }
    let (asked, answered) = _last_turn(memory);
    if !_offered_choices(&answered) {
        return String::new();
    }
    let mut medium = _said_the_medium(&asked);
    if medium.is_empty() {
        medium = _said_the_medium(&answered);
    }
    if medium.is_empty() {
        return String::new();
    }
    format!("any good {taste} {medium}")
}
