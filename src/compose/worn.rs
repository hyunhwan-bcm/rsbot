//! Port of `bot/compose/worn.py`: "this spreadsheet is killing me", "so much to do" - a thing
//! that wears, a pile of things to do, or the work itself - gets a question about that.

use crate::compose::fallback_chain::truthy;
use crate::compose::trouble;
use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.compose.worn.";
static WEAR: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}WEAR")));
static PILE: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}PILE")));
static CONTENT: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}CONTENT")));
macro_rules! sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.compose.worn.", stringify!($name))));)*
    };
}
sets!(MONEY, WEATHER, A_STRETCH, EXTRA_BODY, NOT_A_JOB, POINTING, ROLE);
static PLEA: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.trouble.PLEA"));
static BODY_PART: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.trouble.BODY_PART"));

/// Python: bot/compose/worn.py:111 `pool(shape)`
///
/// The replies for a shape: "pile", "role", or (anything else) one job that fights back.
pub fn pool(shape: &Value) -> Value {
    let name = match shape.as_str() {
        Some("pile") => "PILE_REPLIES",
        Some("role") => "ROLE_REPLIES",
        _ => "JOB_REPLIES",
    };
    data::get(&format!("{M}{name}")).clone()
}

/// Python: bot/compose/worn.py:121 `reads_as_worn(clean, words=None, repaired="")`
///
/// "pile", "role" or "job" for the wear the line names, or None. `words` empty is Python's None
/// (read from the line); `repaired` is only consulted when the line as typed is not one.
pub fn reads_as_worn(clean: &str, words: &[String], repaired: &str) -> Option<String> {
    let text = text::squash(&text::lower(clean));
    if text.is_empty() || text.contains('?') {
        return None; // a question asks something back and is answered as one
    }
    let words = if words.is_empty() { None } else { Some(words) };
    let mut shape = _shape(&text, words);
    if shape.is_none() && !repaired.is_empty() && repaired != text {
        shape = _shape(repaired, None);
    }
    shape
}

/// Python: bot/compose/worn.py:134 `_shape(text, words=None)`
///
/// The shape of the wear in this one text, or None.
pub fn _shape(text: &str, words: Option<&[String]>) -> Option<String> {
    // `set(words) if words else set(CONTENT.findall(text))`, first seen first
    let mut set: Vec<String> = Vec::new();
    let source = match words.filter(|w| !w.is_empty()) {
        Some(w) => w.to_vec(),
        None => CONTENT.findall_str(text),
    };
    for word in source {
        if !set.contains(&word) {
            set.push(word);
        }
    }
    if set.is_empty() {
        return None;
    }
    if trouble::someone_elses(&set) {
        return None;
    }
    if truthy(trouble::reads_as_trouble(text, Some(&set), "")) {
        return None;
    }
    if PLEA.is_match(text) {
        return None;
    }
    if set.iter().any(|w| WEATHER.contains(w)) || set.iter().any(|w| MONEY.contains(w)) {
        return None;
    }
    if !PILE.finditer(text).is_empty() && _names_a_job(text) {
        return Some("pile".into());
    }
    for found in WEAR.finditer(text) {
        let subject = &text[..found.start()];
        if CONTENT.findall_str(subject).iter().any(|w| BODY_PART.contains(w) || EXTRA_BODY.contains(w)) {
            continue;
        }
        if _is_a_role(subject) {
            return Some("role".into());
        }
        if _names_a_job(subject) {
            return Some("job".into());
        }
    }
    None
}

/// Python: bot/compose/worn.py:160 `_content_words(clause)`
///
/// The words of a clause that name something, without the ones that only point at it.
pub fn _content_words(clause: &str) -> Vec<String> {
    CONTENT.findall_str(clause).into_iter().filter(|w| !POINTING.contains(w) && !A_STRETCH.contains(w)).collect()
}

/// Python: bot/compose/worn.py:166 `_is_a_role(clause)`
///
/// True when the wear is about their work or their course, and nothing else.
pub fn _is_a_role(clause: &str) -> bool {
    let content = _content_words(clause);
    !content.is_empty() && content.iter().all(|w| ROLE.contains(w))
}

/// Python: bot/compose/worn.py:175 `_names_a_job(clause)`
///
/// True when the clause names something to be done, not just a stretch of time.
pub fn _names_a_job(clause: &str) -> bool {
    let content = _content_words(clause);
    !content.is_empty() && !content.iter().all(|w| NOT_A_JOB.contains(w))
}
