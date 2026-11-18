//! Port of `bot/compose/own_ask.py`: an ask about the asker's own feeling ("how do i stop
//! feeling guilty", "what's wrong with me") gets what is true of that kind of feeling and one
//! question back, where the search hedge would have been.

use crate::data;
use crate::py::random;
use crate::py::re::{Re, I};
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.compose.own_ask.";
fn set_(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}
static SELF: LazyLock<HashSet<String>> = LazyLock::new(|| set_("SELF"));
static ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set_("ASK_WORDS"));
static ABOUT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set_("ABOUT_WORDS"));
static APPRAISAL: LazyLock<HashSet<String>> = LazyLock::new(|| set_("APPRAISAL"));
static THE_ASK: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}THE_ASK")));
static WRONG_WITH_ME: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}WRONG_WITH_ME")));
static GUILT_LINES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}GUILT_LINES")));
static SHAME_LINES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}SHAME_LINES")));
static LOW_LINES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}LOW_LINES")));
static WHAT_IS_WRONG_LINES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}WHAT_IS_WRONG_LINES")));
/// `POOLS`: (feelings, lines), in order.
static POOLS: LazyLock<Vec<(HashSet<String>, Vec<String>)>> = LazyLock::new(|| {
    data::get(&format!("{M}POOLS")).items().iter().map(|row| (row.items()[0].str_set(), row.items()[1].strs())).collect()
});
/// The patterns written inline in `_words` and `_family`.
static NOT_A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z']+", 0));
static FORGIVE_MYSELF: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:forgive|blame)\s+(?:myself|me)\b", I));

/// Python: bot/compose/own_ask.py:105 `_words(clean)`
pub fn _words(clean: &str) -> Vec<String> {
    NOT_A_WORD.split(&text::lower(clean)).into_iter().filter(|w| !w.is_empty()).collect()
}

/// Python: bot/compose/own_ask.py:109 `_named_thing(clean)`
///
/// A content word in the line that is not the ask and not the feeling, or None (only this file
/// calls it).
pub fn _named_thing(clean: &str) -> Option<String> {
    let mut feeling_seen = false;
    for word in _words(clean) {
        let word = word.strip_suffix("'s").map(str::to_string).unwrap_or(word);
        if SELF.contains(&word) || APPRAISAL.contains(&word) {
            feeling_seen = true;
            continue;
        }
        if feeling_seen && ABOUT_WORDS.contains(&word) {
            return None;
        }
        if ASK_WORDS.contains(&word) {
            continue;
        }
        if text::len(&word) < 3 && !matches!(word.as_str(), "me" | "my" | "im") {
            continue;
        }
        return Some(word);
    }
    None
}

/// Python: bot/compose/own_ask.py:214 `_family(clean)`
///
/// The pool for the feeling in the line, or the what's-wrong-with-me one, or None (a tuple of
/// lines; only this file calls it).
pub fn _family(clean: &str) -> Option<Vec<String>> {
    let words: HashSet<String> = _words(clean).into_iter().collect();
    if WRONG_WITH_ME.is_match(clean) {
        return Some(WHAT_IS_WRONG_LINES.clone());
    }
    for (feelings, lines) in POOLS.iter() {
        if words.iter().any(|w| feelings.contains(w)) {
            return Some(lines.clone());
        }
    }
    if FORGIVE_MYSELF.is_match(clean) {
        return Some(GUILT_LINES.clone());
    }
    if words.iter().any(|w| APPRAISAL.contains(w)) {
        return Some(SHAME_LINES.clone());
    }
    if ["feel", "feels", "feeling", "felt"].iter().any(|w| words.contains(*w)) {
        return Some(LOW_LINES.clone());
    }
    None
}

/// Python: bot/compose/own_ask.py:232 `answer(clean, memory=None)`
///
/// A reply for an ask about the asker's own feeling, or None. Drawn with the module-level
/// generator (`random.choice`), as the Python does.
pub fn answer(clean: &str, _memory: &Memory) -> Option<String> {
    let text = text::strip(clean);
    if text.is_empty() {
        return None;
    }
    if !(THE_ASK.is_match(text) || WRONG_WITH_ME.is_match(text)) {
        return None;
    }
    if _named_thing(text).is_some_and(|w| !w.is_empty()) {
        return None;
    }
    let lines = _family(text).filter(|l| !l.is_empty())?;
    Some(random::choice(&lines))
}

