//! Port of `bot/compose/no_page.py`: "how do i handle my mum", "how do i get through christmas",
//! "how do i cope with it all" have no page to point at, so the reply names the parts the
//! question is made of.

use crate::compose::fallback_chain::py_format;
use crate::data;
use crate::py::re::{Match, Re};
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.no_page.", stringify!($name))));)*
    };
}
regexes!(DOING, PERSON, STRETCH_WORDS, OCCASION, WHOLE, NOT_THIS);
const M: &str = "bot.compose.no_page.";
/// `re.search(r"\b" + word + r"\b", line)` for each of `BODY`.
static BODY: LazyLock<Vec<Re>> =
    LazyLock::new(|| data::strs(&format!("{M}BODY")).iter().map(|w| Re::new(&format!(r"\b{w}\b"), 0)).collect());
static FEMALE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}FEMALE_WORDS")));
static PLURAL_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}PLURAL_WORDS")));
static TAIL_OK: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}TAIL_OK")));
static PROSE: LazyLock<&'static crate::data::Value> = LazyLock::new(|| data::get(&format!("{M}PROSE")));
/// The patterns written inline in `_bare` and `_who`.
static SPACES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+", 0));
static OWNER: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:my|our|the)\s+", 0));
static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z-]+", 0));

/// Python: bot/compose/no_page.py:157 `_bare(text)`
pub fn _bare(text: &str) -> String {
    let squashed = SPACES.sub(" ", text::strip(text));
    text::lower(text::strip_chars(&squashed, " .!,"))
}

/// Python: bot/compose/no_page.py:161 `_who(match)`
///
/// The person as they were named, and the pronoun that goes with them (only this file calls it,
/// with the `PERSON` match).
pub fn _who(found: &Match) -> (String, String) {
    let phrase = text::strip(found.name("who").unwrap_or(""));
    let phrase = OWNER.subn("your ", phrase, 1);
    let words = WORDS.findall_str(&text::lower(&phrase));
    let mut them = "them";
    if !words.iter().any(|w| PLURAL_WORDS.contains(w)) {
        them = if words.iter().any(|w| FEMALE_WORDS.contains(w)) {
            "her"
        } else if words.iter().any(|w| w == "ex" || w == "partner") {
            "them"
        } else {
            "him"
        };
    }
    (phrase, them.to_string())
}

/// Python: bot/compose/no_page.py:186 `_tail_ok(match, line)` (only this file calls it).
pub fn _tail_ok(found: &Match, line: &str) -> bool {
    let tail = text::strip_chars(&line[found.end()..], " .!,");
    TAIL_OK.contains(tail) || tail.starts_with("when ")
}

/// Python: bot/compose/no_page.py:191 `reads(text)`
///
/// The kind of ask this is - "person", "occasion", "stretch", "whole" - or None.
pub fn reads(text: &str) -> Option<String> {
    let line = _bare(text);
    if line.is_empty() || !DOING.is_match(&line) {
        return None;
    }
    if NOT_THIS.is_match(&line) {
        return None;
    }
    if BODY.iter().any(|word| word.is_match(&line)) {
        return None;
    }
    if let Some(person) = PERSON.search(&line) {
        if _tail_ok(&person, &line) {
            return Some("person".into());
        }
    }
    if let Some(occasion) = OCCASION.search(&line) {
        if _tail_ok(&occasion, &line) {
            if STRETCH_WORDS.is_match(&line) {
                return Some("stretch".into());
            }
            return Some("occasion".into());
        }
    }
    if WHOLE.is_match(&line) {
        return Some("whole".into());
    }
    None
}

/// Python: bot/compose/no_page.py:211 `reply_for(text, memory=None)`
///
/// The parts it is made of and the one question back, or None when this is not the ask.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    let kind = reads(text).filter(|k| !k.is_empty())?;
    let pool = PROSE.get(&kind).expect("KeyError").strs();
    let line = &pool[memory.turns.len() % pool.len()];
    let bare = _bare(text);
    if kind == "person" {
        let (who, them) = _who(&PERSON.search(&bare).expect("AttributeError"));
        return Some(py_format(line, &[("who", &who), ("them", &them)]));
    }
    if kind == "occasion" || kind == "stretch" {
        let found = OCCASION.search(&bare).expect("AttributeError");
        let said = py_format(line, &[("what", found.name_str("what"))]);
        let mut chars = said.chars();
        let first = chars.next().map(|c| c.to_uppercase().collect::<String>()).expect("IndexError");
        return Some(first + chars.as_str());
    }
    Some(line.clone())
}
