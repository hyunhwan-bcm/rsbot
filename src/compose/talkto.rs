//! Port of `bot/compose/talkto.py`: "can we talk about my mum", "i need to think through my
//! options" gets a yes and the one question that starts it.

use crate::compose::fallback_chain::py_format;
use crate::data;
use crate::memory::narrow::memory_value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.compose.talkto.";
macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.talkto.", stringify!($name))));)*
    };
}
regexes!(
    _SUBJECT_TAIL, _A_TIME_BEFORE_THE_THING, _SUBJECT_HEAD, _SOMETHING_ELSE, _SUBJECT_END,
    _CLAUSE_START, _SUBJECT_END_CLAUSE, _HEDGED_END, _A_BARE_INVITATION, _A_LEAD_IN_FRONT,
    _REPEATED_WORD,
);
static TALK_ASKS: LazyLock<Vec<&'static Re>> =
    LazyLock::new(|| data::get(&format!("{M}TALK_ASKS")).items().iter().map(|v| v.as_re().unwrap()).collect());
static BLANK_SUBJECTS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}BLANK_SUBJECTS")));
static WEIGHING_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}WEIGHING_WORDS")));
static INVITED: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}INVITED")));
static POOL: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}POOL")));
static POOL_CLAUSE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}POOL_CLAUSE")));
static WEIGHING: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}WEIGHING")));
static _OWNERS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get(&format!("{M}_OWNERS")).items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].str().to_string())).collect()
});
static _TYPED_SHORT_DICT: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map(&format!("{M}_TYPED_SHORT_DICT")));

/// `from . import pick`: its contract takes the memory as a `Value`.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, &memory_value(memory), seed_text)
}

/// Python: bot/compose/talkto.py:152 `_as_typed(line)`
///
/// The line with the chat-speak the frames use spelled out and a doubled word collapsed.
pub fn _as_typed(line: &str) -> String {
    let line = _REPEATED_WORD.sub(r"\1", line);
    text::split_ref(&line).iter().map(|w| _TYPED_SHORT_DICT.get(*w).cloned().unwrap_or_else(|| w.to_string())).collect::<Vec<_>>().join(" ")
}

/// Python: bot/compose/talkto.py:162 `the_line_as_read(text)`
///
/// The line the frames are matched against: chat-speak spelled out, a named day dropped.
pub fn the_line_as_read(text: &str) -> String {
    _A_TIME_BEFORE_THE_THING.sub(r"\1 ", &_as_typed(text))
}

/// Python: bot/compose/talkto.py:173 `_subject(text)`
///
/// The thing named after the talk frame, or "" when none is named.
pub fn _subject(text: &str) -> String {
    let text = the_line_as_read(text);
    for pattern in TALK_ASKS.iter() {
        let Some(found) = pattern.match_(&text) else { continue };
        let subject = format!(" {}", text::rstrip_chars(text::strip(found.name_str("subject")), ".!?,"));
        let subject = text::strip(&_SUBJECT_TAIL.sub("", &subject)).to_string();
        let subject = text::strip(&_SUBJECT_HEAD.sub("", &subject)).to_string();
        let subject = text::strip(&_SOMETHING_ELSE.sub("", &subject)).to_string();
        let clause = _CLAUSE_START.match_(&subject).is_some();
        let end = if clause { &_SUBJECT_END_CLAUSE } else { &_SUBJECT_END };
        let subject = text::strip(&end.sub("", &subject)).to_string();
        let subject = text::strip(&_HEDGED_END.sub("", &subject)).to_string();
        if BLANK_SUBJECTS.contains(&subject) {
            return String::new();
        }
        let mut words: Vec<String> = text::split(&subject);
        if words.is_empty() || words.len() > if clause { 9 } else { 5 } {
            return String::new();
        }
        if let Some((_, theirs)) = _OWNERS.iter().find(|(mine, _)| *mine == words[0]) {
            words[0] = theirs.clone();
        }
        return words.join(" ");
    }
    String::new()
}

/// Python: bot/compose/talkto.py:198 `answer_for(text, memory)`
///
/// A yes and the one question that starts it, or "" when the line is not this ask.
pub fn answer_for(text: &str, memory: &Memory) -> String {
    let line = text::squash(&text::lower(text));
    if line.is_empty() {
        return String::new();
    }
    let subject = _subject(&line);
    if subject.is_empty() {
        let typed = _as_typed(&line);
        if _A_BARE_INVITATION.match_(&typed).is_some()
            && (_A_LEAD_IN_FRONT.match_(&typed).is_some() || text::rstrip(&typed).ends_with('?'))
        {
            return pick(&INVITED, memory, text);
        }
        return String::new();
    }
    if text::split_ref(&subject).iter().any(|w| WEIGHING_WORDS.contains(*w)) {
        return pick(&WEIGHING, memory, text);
    }
    let frames = if _CLAUSE_START.match_(&subject).is_some() { &*POOL_CLAUSE } else { &*POOL };
    let upper = text::upper(&text::slice(&subject, 0, Some(1))) + &text::slice(&subject, 1, None);
    let lines: Vec<String> = frames.iter().map(|one| py_format(one, &[("thing", &subject), ("Thing", &upper)])).collect();
    pick(&lines, memory, text)
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
        if let Some(got) = unless_unported(|| _subject("can we talk about my mum")) {
            assert_eq!(got, "your mum".to_string());
        }
    }

    #[test]
    fn case_1() {
        if let Some(got) = unless_unported(|| the_line_as_read("can we talk about my mum")) {
            assert_eq!(got, "can we talk about my mum".to_string());
        }
    }

    #[test]
    fn case_2() {
        if let Some(got) = unless_unported(|| _subject("i need to think through my options")) {
            assert_eq!(got, "your options".to_string());
        }
    }

    #[test]
    fn case_3() {
        if let Some(got) = unless_unported(|| the_line_as_read("i need to think through my options")) {
            assert_eq!(got, "i need to think through my options".to_string());
        }
    }

    #[test]
    fn case_4() {
        if let Some(got) = unless_unported(|| _subject("can we talk tomorrow about my mum")) {
            assert_eq!(got, "your mum".to_string());
        }
    }

    #[test]
    fn case_5() {
        if let Some(got) = unless_unported(|| the_line_as_read("can we talk tomorrow about my mum")) {
            assert_eq!(got, "can we talk about my mum".to_string());
        }
    }

    #[test]
    fn case_6() {
        if let Some(got) = unless_unported(|| _subject("can u talk to me about it")) {
            assert_eq!(got, "".to_string());
        }
    }

    #[test]
    fn case_7() {
        if let Some(got) = unless_unported(|| the_line_as_read("can u talk to me about it")) {
            assert_eq!(got, "can you talk to me about it".to_string());
        }
    }

    #[test]
    fn case_8() {
        if let Some(got) = unless_unported(|| _subject("i wanna wanna talk about my job and stuff")) {
            assert_eq!(got, "your job".to_string());
        }
    }

    #[test]
    fn case_9() {
        if let Some(got) = unless_unported(|| the_line_as_read("i wanna wanna talk about my job and stuff")) {
            assert_eq!(got, "i want talk about my job and stuff".to_string());
        }
    }

    #[test]
    fn case_10() {
        if let Some(got) = unless_unported(|| _subject("can you help me think about how to tell her")) {
            assert_eq!(got, "how to tell her".to_string());
        }
    }

    #[test]
    fn case_11() {
        if let Some(got) = unless_unported(|| the_line_as_read("can you help me think about how to tell her")) {
            assert_eq!(got, "can you help me think about how to tell her".to_string());
        }
    }

    #[test]
    fn case_12() {
        if let Some(got) = unless_unported(|| _subject("can we talk")) {
            assert_eq!(got, "".to_string());
        }
    }

    #[test]
    fn case_13() {
        if let Some(got) = unless_unported(|| the_line_as_read("can we talk")) {
            assert_eq!(got, "can we talk".to_string());
        }
    }
}
