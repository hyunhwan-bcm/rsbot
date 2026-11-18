//! Port of `bot/compose/facts.py`: whether a line asks when something is, and whether a reply
//! is the date it asked for.

use crate::data::{self, Value};
use crate::memory::narrow;
use crate::py::re::Re;
use crate::py::text;
use std::sync::{Arc, LazyLock};

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.facts.", stringify!($name))));)*
    };
}
regexes!(WHEN_ASK, HOW_TO_MOMENT, A_CLOCK, DATE_TOKEN);
static MOST_WORDS: LazyLock<usize> = LazyLock::new(|| data::get("bot.compose.facts.MOST_WORDS").as_int().unwrap() as usize);

/// Python: bot/compose/facts.py:40 `asks_for_a_when(text)`
///
/// Whether the line asks when something is, rather than how to do it.
pub fn asks_for_a_when(text: &str) -> bool {
    if HOW_TO_MOMENT.is_match(text) {
        return false;
    }
    WHEN_ASK.is_match(text)
}

/// Python: bot/compose/facts.py:47 `holds_a_date(text)`
///
/// Whether a reply carries a date, a weekday or a clock time.
pub fn holds_a_date(text: &str) -> bool {
    narrow::has_a_date(text) || A_CLOCK.is_match(text)
}

/// The date words as a list, first seen first (the set Python builds, in insertion order).
fn date_words(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in DATE_TOKEN.findall_str(text) {
        let word = text::rstrip_chars(&text::lower(&word), ".").to_string();
        if !out.contains(&word) {
            out.push(word);
        }
    }
    out
}

/// Python: bot/compose/facts.py:72 `_date_words(text)`: a set of str.
pub fn _date_words(text: &str) -> Value {
    Value::Set(Arc::new(date_words(text).into_iter().map(Value::from).collect()))
}

/// Python: bot/compose/facts.py:76 `the_same_date(one, two)`
///
/// Whether two replies name the same date. `two` is a str or None (None reads as "").
pub fn the_same_date(one: &str, two: &Value) -> bool {
    let first = date_words(one);
    if first.is_empty() {
        return false;
    }
    let second = date_words(two.as_str().unwrap_or(""));
    first.iter().any(|word| second.contains(word))
}

/// Python: bot/compose/facts.py:82 `the_fact_asked_for(text, reply)`
///
/// Whether this reply is the date the line asked for.
pub fn the_fact_asked_for(text: &str, reply: &str) -> bool {
    if !asks_for_a_when(text) {
        return false;
    }
    if text::split_ref(reply).len() > *MOST_WORDS {
        return false;
    }
    holds_a_date(reply)
}
