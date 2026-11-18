//! Port of `bot/compose/onmybehalf.py`: an errand handed to a bot with no hands ("book me a
//! table", "google the train times", "send this to my boss") gets the honest limit and the half
//! the bot can do, in place of the composer's flat reading or its hedge.

use crate::compose::fallback;
use crate::compose::fallback_chain::py_format;
use crate::data::{self, Value};
use crate::memory::narrow::memory_value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.onmybehalf.", stringify!($name))));)*
    };
}
regexes!(
    _A_SEARCH, _A_SPEND, _A_MESSAGE_OUT, _A_LIST, _THEIR_ERRAND, _KEEP_OUT, _THE_VERB_AND_THEN,
    _A_LEADING_PREPOSITION, _THE_ADDRESSEE, _THE_TAIL_OF_THE_ASK, _MINE,
);
const M: &str = "bot.compose.onmybehalf.";
static _FLAT: LazyLock<Vec<Vec<String>>> =
    LazyLock::new(|| data::get(&format!("{M}_FLAT")).items().iter().map(Value::strs).collect());
static A_NO_SEARCH: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}A_NO_SEARCH")));
static A_NO_SPEND: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}A_NO_SPEND")));
static A_NO_SEND: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}A_NO_SEND")));
static A_NO_LIST: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}A_NO_LIST")));
static _A_MERE_PRONOUN: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}_A_MERE_PRONOUN")));
static _YOURS: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map(&format!("{M}_YOURS")));
static POINTS_BACK_NOTHING: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.store_lines.POINTS_BACK_NOTHING"));
static POINTING_WORD_NOTHING: LazyLock<Vec<String>> =
    LazyLock::new(|| data::strs("bot.memory.store_lines.POINTING_WORD_NOTHING"));

/// `from . import pick`: its contract takes the memory as a `Value`.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, &memory_value(memory), seed_text)
}

/// Python: bot/compose/onmybehalf.py:153 `_facing_them(phrase)`
///
/// The person's own phrase turned to face them: "our flight times" -> "your flight times" (a str;
/// only this file calls it).
pub fn _facing_them(phrase: &str) -> String {
    _MINE.sub_fn(phrase, |found| _YOURS[&text::lower(found.str(1))].clone())
}

/// Python: bot/compose/onmybehalf.py:158 `_what_was_asked_for(clean)`
///
/// The thing the person named in their errand, or "it" when they named nothing.
pub fn _what_was_asked_for(clean: &str) -> String {
    let after = _THE_VERB_AND_THEN.subn("", clean, 1);
    let mut after = _THE_ADDRESSEE.subn("", &after, 1);
    for _ in 0..4 {
        let before = after.clone();
        after = text::strip(&_A_LEADING_PREPOSITION.subn("", &after, 1)).to_string();
        after = text::strip_chars(&_THE_TAIL_OF_THE_ASK.sub("", &after), " .!?,").to_string();
        if after == before {
            break;
        }
    }
    let words = text::split_ref(&after);
    if words.is_empty() || words.len() > 8 || (words.len() == 1 && _A_MERE_PRONOUN.contains(words[0])) {
        return "it".into();
    }
    words.join(" ")
}

/// Python: bot/compose/onmybehalf.py:174 `_is_a_flat_read(reply)`
///
/// True for the composer's flat readings this module replaces, and nothing else.
pub fn _is_a_flat_read(reply: &str) -> bool {
    _FLAT.iter().any(|pool| pool.iter().any(|line| line == reply))
}

/// Python: bot/compose/onmybehalf.py:179 `_is_a_hedge(reply)`
///
/// True for the composer's lines that say it cannot answer, or that nothing was said yet.
pub fn _is_a_hedge(reply: &str) -> bool {
    if fallback::is_a_hedge(reply) {
        return true;
    }
    POINTS_BACK_NOTHING.iter().chain(POINTING_WORD_NOTHING.iter()).any(|line| line == reply)
}

/// Python: bot/compose/onmybehalf.py:200 `_about_an_errand(clean)`
///
/// The pool for the errand the line hands over, or None (only this file calls it).
pub fn _about_an_errand(clean: &str) -> Option<Vec<String>> {
    if text::split_ref(clean).len() > 14 {
        return None;
    }
    let a_search = _A_SEARCH.is_match(clean);
    if !a_search && !_THEIR_ERRAND.is_match(clean) {
        return None;
    }
    if _KEEP_OUT.is_match(clean) {
        return None;
    }
    if a_search {
        return Some(A_NO_SEARCH.clone());
    }
    if _A_SPEND.is_match(clean) {
        return Some(A_NO_SPEND.clone());
    }
    if _A_MESSAGE_OUT.is_match(clean) {
        return Some(A_NO_SEND.clone());
    }
    if _A_LIST.is_match(clean) {
        return Some(A_NO_LIST.clone());
    }
    None
}

/// Python: bot/compose/onmybehalf.py:219 `answer_for(text, memory, reply)`
///
/// The honest line for an errand handed to a bot with no hands, or None.
pub fn answer_for(text: &str, memory: &Memory, reply: &str) -> Option<String> {
    if reply.is_empty() {
        return None;
    }
    let clean = text::squash(&text::lower(text));
    if clean.is_empty() {
        return None;
    }
    let pool = _about_an_errand(&clean).filter(|p| !p.is_empty())?;
    // `pool is A_NO_SEARCH`: the four pools differ, so equality is identity here.
    let a_search = pool == *A_NO_SEARCH;
    if !_is_a_flat_read(reply) && !(a_search && _is_a_hedge(reply)) {
        return None;
    }
    let line = pick(&pool, memory, text);
    Some(py_format(&line, &[("what", &_facing_them(&_what_was_asked_for(&clean)))]))
}
