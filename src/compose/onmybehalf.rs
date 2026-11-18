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
    crate::compose::pick(options, memory, seed_text)
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

// ---- COM5-CASES (generated by harness/com5_cases.py; expected values from PyBot) ----
#[cfg(test)]
mod pybot_cases {
    #![allow(unused_imports, dead_code)]
    use super::*;
    use crate::data::Value;
    use crate::state::{Memory, Turn};

    fn mem(turns: &[(&str, &str, &str)]) -> Memory {
        let mut m = Memory::new();
        m.turns = turns
            .iter()
            .map(|(u, b, md)| Turn { user: u.to_string(), bot: b.to_string(), module: (!md.is_empty()).then(|| md.to_string()) })
            .collect();
        m
    }

    /// The call's result, or None when it reached a function another workstream has not ported yet.
    fn unless_unported<T>(f: impl FnOnce() -> T) -> Option<T> {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
            Ok(v) => Some(v),
            Err(e) => {
                let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                if msg.starts_with("UNPORTED") {
                    eprintln!("skipped: {msg}");
                    return None;
                }
                std::panic::resume_unwind(e)
            }
        }
    }

    /// True when the call raises (panics), as the Python does.
    fn raises<T>(f: impl FnOnce() -> T) -> bool {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err()
    }

    #[test]
    fn case_0() {
        if let Some(got) = unless_unported(|| _what_was_asked_for("book me a table")) {
            assert_eq!(got, "a table".to_string());
        }
    }

    #[test]
    fn case_1() {
        if let Some(got) = unless_unported(|| _about_an_errand("book me a table")) {
            assert_eq!(got, Some(vec!["I can't book, order or pay for anything - no account, no card, no hands. Do it in their app or on their site, and tell me the price first: I'll say whether it looks fair and what to check before you pay.".to_string(), "Nothing on my side can hold a booking or take money. Read their cancellation line before you click buy - it is the one part of this that costs you later - and say what you are buying and I'll say what to watch.".to_string(), "That one needs your account, not me: I can't reach a basket or a booking form. Tell me what it costs and what it covers, and I'll help you work out whether the dearer one earns its money.".to_string()]));
        }
    }

    #[test]
    fn case_2() {
        if let Some(got) = unless_unported(|| _what_was_asked_for("google our flight times for me")) {
            assert_eq!(got, "our flight times".to_string());
        }
    }

    #[test]
    fn case_3() {
        if let Some(got) = unless_unported(|| _about_an_errand("google our flight times for me")) {
            assert_eq!(got, Some(vec!["Nothing here can search the web - no browser on this side. Say what matters most about {what} and I'll help you work out what to compare; the site or app itself has the rest.".to_string(), "I can't look anything up live, sorry. Tell me what you need to know about {what} and I'll say what to look for and what to ignore; whoever sells it has today's version.".to_string(), "No search from here - I only know what is in my own head, and none of it changes today. Name the details that matter about {what} and I'll narrow it down before you open the site.".to_string()]));
        }
    }

    #[test]
    fn case_4() {
        if let Some(got) = unless_unported(|| _what_was_asked_for("send this to my boss")) {
            assert_eq!(got, "this to my boss".to_string());
        }
    }

    #[test]
    fn case_5() {
        if let Some(got) = unless_unported(|| _about_an_errand("send this to my boss")) {
            assert_eq!(got, Some(vec!["I can't send or print anything - nothing leaves this chat. I'll get {what} right here instead, in full; copying it across is the one press on your side.".to_string(), "No email, no text, no printer on my side, sorry. What I can do is get {what} exactly right here; pasting it over is the last step, and it is yours.".to_string(), "Nothing here can post or send. I'll write {what} out so it is ready to go - the copy and the send are the two presses on your end.".to_string()]));
        }
    }

    #[test]
    fn case_6() {
        if let Some(got) = unless_unported(|| _what_was_asked_for("can you search for cheap flights please")) {
            assert_eq!(got, "cheap flights".to_string());
        }
    }

    #[test]
    fn case_7() {
        if let Some(got) = unless_unported(|| _about_an_errand("can you search for cheap flights please")) {
            assert_eq!(got, Some(vec!["Nothing here can search the web - no browser on this side. Say what matters most about {what} and I'll help you work out what to compare; the site or app itself has the rest.".to_string(), "I can't look anything up live, sorry. Tell me what you need to know about {what} and I'll say what to look for and what to ignore; whoever sells it has today's version.".to_string(), "No search from here - I only know what is in my own head, and none of it changes today. Name the details that matter about {what} and I'll narrow it down before you open the site.".to_string()]));
        }
    }

    #[test]
    fn case_8() {
        if let Some(got) = unless_unported(|| _what_was_asked_for("save it")) {
            assert_eq!(got, "it".to_string());
        }
    }

    #[test]
    fn case_9() {
        if let Some(got) = unless_unported(|| _about_an_errand("save it")) {
            assert_eq!(got, Some(vec!["Nothing here keeps a list between chats - every chat starts fresh, so what you put here does not come back tomorrow. Write it where you'll see it (your phone's notes, a paper by the door) and tell me the items: I'll help you sort them.".to_string(), "I can't save anything, sorry - there is nowhere on my side for it to live. Say the items out loud here and I'll help you order and trim the list; then keep it in your own notes.".to_string(), "No memory of a list once we stop talking. Give me the lot and I'll group them by where they are in the shop; your notes app is the thing that will still have it tomorrow.".to_string()]));
        }
    }

    #[test]
    fn case_10() {
        if let Some(got) = unless_unported(|| _what_was_asked_for("how do i book a table")) {
            assert_eq!(got, "how do i book a table".to_string());
        }
    }

    #[test]
    fn case_11() {
        if let Some(got) = unless_unported(|| _about_an_errand("how do i book a table")) {
            assert_eq!(got, None);
        }
    }

    #[test]
    fn case_12() {
        if let Some(got) = unless_unported(|| _what_was_asked_for("remind me to call mum")) {
            assert_eq!(got, "remind me to call mum".to_string());
        }
    }

    #[test]
    fn case_13() {
        if let Some(got) = unless_unported(|| _about_an_errand("remind me to call mum")) {
            assert_eq!(got, Some(vec!["Nothing here keeps a list between chats - every chat starts fresh, so what you put here does not come back tomorrow. Write it where you'll see it (your phone's notes, a paper by the door) and tell me the items: I'll help you sort them.".to_string(), "I can't save anything, sorry - there is nowhere on my side for it to live. Say the items out loud here and I'll help you order and trim the list; then keep it in your own notes.".to_string(), "No memory of a list once we stop talking. Give me the lot and I'll group them by where they are in the shop; your notes app is the thing that will still have it tomorrow.".to_string()]));
        }
    }

    #[test]
    fn case_14() {
        if let Some(got) = unless_unported(|| _what_was_asked_for("order my prescription")) {
            assert_eq!(got, "my prescription".to_string());
        }
    }

    #[test]
    fn case_15() {
        if let Some(got) = unless_unported(|| _about_an_errand("order my prescription")) {
            assert_eq!(got, Some(vec!["I can't book, order or pay for anything - no account, no card, no hands. Do it in their app or on their site, and tell me the price first: I'll say whether it looks fair and what to check before you pay.".to_string(), "Nothing on my side can hold a booking or take money. Read their cancellation line before you click buy - it is the one part of this that costs you later - and say what you are buying and I'll say what to watch.".to_string(), "That one needs your account, not me: I can't reach a basket or a booking form. Tell me what it costs and what it covers, and I'll help you work out whether the dearer one earns its money.".to_string()]));
        }
    }
}
