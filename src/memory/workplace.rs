//! Port of `bot/memory/workplace.py`: "i work in a primary school" when the chat already holds
//! the job - "A teacher in a primary school - ..." with the place said back the way it came.

use super::mem8_py::{pick, rx, s_or, strs};
use crate::data::Value;
use crate::py::re::{Re, I};
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

const M: &str = "bot.memory.workplace.";

rx!(I_WORK, NOT_MINE, A_QUESTION, NOT_A_PLACE, A_PLACE, WITH_THE_DETAIL, USED_TO, A_SHIFT_NOT_A_TRADE);
strs!(_TAILS, _FROM_HOME_TAILS);
static NOT_A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9 ]+", 0));
static A_CLAUSE: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(?:and|but|because|so|when|while|although|though|which|where|if)\b", 0));
static TRAILING_PUNCT: LazyLock<Re> = LazyLock::new(|| Re::new("[\\s,.;:!?'\u{2019}]+$", 0));
static TRAILING_FILLER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+(?:anyway|now|though|too|still)$", I));

/// Python: bot/memory/workplace.py:94 `_the_same_line(a, b)`
///
/// Whether two lines are the same after case and spacing.
pub fn _the_same_line(a: &str, b: &str) -> bool {
    let clean = |said: &str| text::split(&NOT_A_WORD.sub(" ", &said.to_lowercase()));
    clean(a) == clean(b)
}

/// Python: bot/memory/workplace.py:101 `_the_job_the_chat_holds(memory, text='')`
///
/// The job the person said of themselves on an earlier turn, in the words they used, or "".
pub fn _the_job_the_chat_holds(memory: &Memory, text: &str) -> String {
    if let Some(Value::List(entries)) = memory.get("facts_log") {
        for entry in entries.iter().rev() {
            if !entry.is_dict() || entry.get("key").and_then(Value::as_str) != Some("job") {
                continue;
            }
            let Some(value) = entry.get("value").filter(|v| v.truthy()) else { continue };
            if !text.is_empty() && _the_same_line(&s_or(entry.get("text")), text) {
                continue; // the job this very line filed, which is the place it named
            }
            let value = super::mem8_py::py_str(value);
            let value = text::strip(&value);
            if A_SHIFT_NOT_A_TRADE.match_(value).is_some() {
                continue; // "i work nights" files the hours, not the trade
            }
            return value.to_string();
        }
        return String::new();
    }
    match memory.get("user").filter(|u| u.is_dict()) {
        Some(user) => text::strip(&s_or(user.get("job"))).to_string(),
        None => String::new(),
    }
}

/// Python: bot/memory/workplace.py:126 `_with_an_article(job)`
///
/// The job the way it opens a sentence: "a teacher", "an engineer".
pub fn _with_an_article(job: &str) -> String {
    let word = text::strip(job);
    if word.is_empty() {
        return String::new();
    }
    let first = text::slice(word, 0, Some(1)).to_lowercase();
    let article = if "aeiou".contains(first.as_str()) { "an" } else { "a" };
    format!("{article} {word}")
}

/// Python: bot/memory/workplace.py:135 `_a_clean_place(place)`
///
/// The place with its trailing punctuation and filler taken off: "a primary school".
pub fn _a_clean_place(place: &str) -> String {
    // A clause after the place is not part of it.
    let word = A_CLAUSE.split(place).swap_remove(0);
    let word = TRAILING_PUNCT.sub("", text::strip(&word));
    let mut word = TRAILING_FILLER.sub("", &word);
    let words = text::split_ref(&word);
    if words.len() > 5 {
        word = words[..5].join(" ");
    }
    text::strip(&word).to_string()
}

/// Python: bot/memory/workplace.py:148 `_the_place_the_line_said(text)`
///
/// (the preposition, the place), when the line says where the person's work is.
pub fn _the_place_the_line_said(text: &str) -> Option<(String, String)> {
    let said = text::strip(text);
    if said.is_empty() || !I_WORK.is_match(said) {
        return None;
    }
    if A_QUESTION.match_(said).is_some() || said.ends_with('?') || USED_TO.is_match(said) {
        return None;
    }
    if NOT_MINE.is_match(said) {
        return None;
    }
    let found = I_WORK.search(said)?;
    let rest = text::strip(found.name_str("rest"));
    if rest.is_empty() || NOT_A_PLACE.match_(rest).is_some() {
        return None;
    }
    let place = A_PLACE.match_(rest)?;
    if WITH_THE_DETAIL.is_match(place.name_str("place")) {
        return None;
    }
    let clean = _a_clean_place(place.name_str("place"));
    if text::len(&clean) < 3 {
        return None;
    }
    // The place has to be the end of the line or near it.
    let tail = text::strip_chars(&rest[place.end()..], " ,.;:!?'\u{2019}");
    if text::split_ref(tail).len() > 3 {
        return None;
    }
    Some((place.name_str("prep").to_lowercase(), clean))
}

/// Python: bot/memory/workplace.py:178 `the_place_the_person_works(text, memory)`
///
/// The reply to a line that says where the person's work is, or None.
pub fn the_place_the_person_works(text: &str, memory: &Memory) -> Option<String> {
    let (prep, place) = _the_place_the_line_said(text)?;
    let job = _the_job_the_chat_holds(memory, text);
    if job.is_empty() {
        // With no job on file the reply would be a guess at the trade.
        return None;
    }
    let lead = text::capitalize(&_with_an_article(&job));
    let home = prep == "from" || ["home", "the house"].contains(&place.to_lowercase().as_str());
    if home {
        return Some(format!("{lead} working from home - {}", pick(&_FROM_HOME_TAILS, memory, text)));
    }
    // The line's own preposition, so the place goes back the way it came.
    Some(format!("{lead} {prep} {place} - {}", pick(&_TAILS, memory, text)))
}
