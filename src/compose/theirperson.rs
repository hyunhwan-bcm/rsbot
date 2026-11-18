//! Port of `bot/compose/theirperson.py`: "my best friend is having a really hard time" - a
//! person of theirs, as the subject, in a rough patch (or with a condition named) - gets a
//! friend's reply rather than the nod.

use crate::compose::fallback;
use crate::data::{self, Value};
use crate::py::re::{Match, Re, I};
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.theirperson.", stringify!($name))));)*
    };
}
regexes!(A_PERSON_OF_THEIRS, A_ROUGH_PATCH, A_NOT_COPING, A_LOW, A_CONDITION, A_WONDERING, STANDS_DOWN);
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.theirperson.", stringify!($name))));)*
    };
}
pools!(A_CONDITION_STATED, A_CONDITION_WONDERED, POOL, NAMED);
static NEUTRAL: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.fallback.NEUTRAL"));
/// The patterns written inline in `_said`, `_who` and `reply_for`.
static SAID_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9'?!]+", 0));
static OWNER: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:my|our)\s+", I));
static MIGHT_HAVE: LazyLock<Re> = LazyLock::new(|| Re::new(r"^\s*(?:could|might|may|must)\s+have\b", 0));
static AND_ME: LazyLock<Re> = LazyLock::new(|| Re::new(r"^\s*(?:and|&)\s+(?:i|me)\b", 0));
static PLURAL: LazyLock<Re> = LazyLock::new(|| Re::new(r"^\s*(?:are|were|have|have\s+been|were\s+being)\b", 0));

/// `from . import pick`: its contract takes the memory as a `Value`.
fn pick(options: &[String], memory: &Value, seed_text: &str) -> String {
    // Python hands pick a dict that is not the chat's memory (a reading or a one-turn dict);
    // pick reads it as one, so it is read here as a Memory built from that dict.
    let as_memory = match memory.to_plain() {
        serde_json::Value::Object(m) => Memory::from_plain(&m),
        _ => Memory::new(),
    };
    crate::compose::pick(options, &as_memory, seed_text)
}

/// Python: bot/compose/theirperson.py:157 `_said(text)`
///
/// The line's words, lowercased, with a space at each end, or "".
pub fn _said(text: &str) -> String {
    let words = SAID_WORDS.findall_str(&text::lower(text));
    if words.is_empty() {
        return String::new();
    }
    format!(" {} ", words.join(" "))
}

/// Python: bot/compose/theirperson.py:162 `_who(person, said)`
///
/// The person as the reply names them: "my best friend" is "best friend" (only this file calls
/// it, with the `A_PERSON_OF_THEIRS` match).
pub fn _who(person: &Match, said: &str) -> String {
    let phrase = text::strip(&said[person.start()..person.end()]);
    OWNER.subn("", phrase, 1)
}

/// Python: bot/compose/theirperson.py:169 `reply_for(text, memory=None)`
///
/// The friend's reply to somebody else's rough patch, or None when the line is not that.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    let said = _said(text);
    if text::strip(&said).is_empty() {
        return None;
    }
    if STANDS_DOWN.is_match(&said) {
        return None;
    }
    let person = A_PERSON_OF_THEIRS.search(&said)?;
    let tail = &said[person.end()..];
    let a_condition = A_CONDITION.match_(tail).is_some();
    if a_condition && MIGHT_HAVE.match_(tail).is_some() {
        return None;
    }
    if !(a_condition || A_ROUGH_PATCH.match_(tail).is_some() || A_NOT_COPING.match_(tail).is_some() || A_LOW.match_(tail).is_some()) {
        return None;
    }
    if AND_ME.match_(tail).is_some() {
        return None;
    }
    // `read = memory.get("understood")`, a dict or {}
    let read = match &memory.understood {
        Some(u) => Value::from_plain(&serde_json::to_value(u).unwrap()),
        None => Value::dict(),
    };
    let plural = PLURAL.match_(tail).is_some();
    let pool: Vec<String> = if a_condition {
        if A_WONDERING.is_match(tail) { A_CONDITION_WONDERED.clone() } else { A_CONDITION_STATED.clone() }
    } else if plural {
        POOL.clone()
    } else {
        POOL.iter().chain(NAMED.iter()).cloned().collect()
    };
    let who = _who(&person, &said);
    let mut worded: Vec<String> = pool.iter().map(|line| line.replace("{who}", &who)).collect();
    if let Some(last) = memory.turns.last().map(|t| t.bot.as_str()).filter(|l| !l.is_empty()) {
        if worded.iter().any(|w| w == last) && worded.len() > 1 {
            worded.retain(|line| line != last);
        }
    }
    Some(pick(&worded, &read, text::strip(&said)))
}

/// Python: bot/compose/theirperson.py:218 `may_replace(reply, text="")`
///
/// True when the bot's line for this turn says nothing about what the person brought.
pub fn may_replace(reply: &str, text: &str) -> bool {
    let said = text::strip(reply);
    if said.is_empty() {
        return false;
    }
    if fallback::_is_a_hedge(said) || fallback::in_a_pool(said, &[&NEUTRAL[..]]) {
        return true;
    }
    let mine: HashSet<String> = fallback::_the_words(text).into_iter().collect();
    let theirs: HashSet<String> = fallback::_the_words(said).into_iter().collect();
    if mine.len() < 2 || theirs.is_empty() {
        return false;
    }
    let shared = mine.intersection(&theirs).count();
    shared >= 2 && shared as f64 / mine.len() as f64 >= 0.6
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
        if let Some(got) = unless_unported(|| _said("My best friend is having a really hard time!")) {
            assert_eq!(got, " my best friend is having a really hard time! ".to_string());
        }
    }

    #[test]
    fn case_1() {
        if let Some(got) = unless_unported(|| _said("my mum is ill")) {
            assert_eq!(got, " my mum is ill ".to_string());
        }
    }

    #[test]
    fn case_2() {
        if let Some(got) = unless_unported(|| _said("my son has adhd")) {
            assert_eq!(got, " my son has adhd ".to_string());
        }
    }

    #[test]
    fn case_3() {
        if let Some(got) = unless_unported(|| may_replace("I'm listening — say more if it helps.", "my mum is struggling")) {
            assert_eq!(got, true);
        }
    }

    #[test]
    fn case_4() {
        if let Some(got) = unless_unported(|| may_replace("Your mum is struggling, noted.", "my mum is struggling")) {
            assert_eq!(got, true);
        }
    }

    #[test]
    fn case_5() {
        if let Some(got) = unless_unported(|| may_replace("Lovely.", "my mum is struggling")) {
            assert_eq!(got, false);
        }
    }
}
