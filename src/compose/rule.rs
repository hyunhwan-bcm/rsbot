//! Port of `bot/compose/rule.py`: a question about the law, a council's rule or a court letter
//! gets the one place whose wording counts.

use crate::data::{self, Value};
use crate::memory::narrow::memory_value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::{Memory, Turn};
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.rule.", stringify!($name))));)*
    };
}
regexes!(
    A_COURT_THING, A_GAME_COURT, A_LOCAL_RULE, LAW_ASKED, NOT_A_RULE_ASK, A_FOLLOW_UP,
    PERMISSION_ASKED, WHO_OWNS_IT, A_PENALTY,
);
static UNKNOWN_THE_LAW: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.rule.UNKNOWN_THE_LAW"));
static UNKNOWN_A_LOCAL_RULE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.rule.UNKNOWN_A_LOCAL_RULE"));
static UNKNOWN_A_COURT_THING: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.rule.UNKNOWN_A_COURT_THING"));

/// `from . import pick`: its contract takes the memory as a `Value`.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, &memory_value(memory), seed_text)
}

/// `turns[-look_back:]` with Python's slice rules (0 is the whole list).
fn last_turns(turns: &[Turn], look_back: i64) -> &[Turn] {
    let n = turns.len() as i64;
    let start = if look_back == 0 { 0 } else if look_back > 0 { (n - look_back).max(0) } else { (-look_back).min(n) };
    &turns[start as usize..]
}

/// Python: bot/compose/rule.py:108 `_said(text)`
pub fn _said(text: &str) -> String {
    text::squash(&text::lower(text))
}

/// Python: bot/compose/rule.py:113 `_a_court_of_law(said)`
///
/// True when a court word in the line means a court of law rather than a place you play.
pub fn _a_court_of_law(said: &str) -> bool {
    if !A_COURT_THING.is_match(said) {
        return false;
    }
    !(A_GAME_COURT.is_match(said) && !LAW_ASKED.is_match(said))
}

/// Python: bot/compose/rule.py:125 `_the_thread_said(memory, pattern, look_back=2)`
///
/// True when one of the last few things the person said matched `pattern` (a compiled pattern
/// value; `look_back` an int, None for the default 2).
pub fn _the_thread_said(memory: &Memory, pattern: &Value, look_back: &Value) -> bool {
    let pattern = pattern.as_re().expect("pattern is a compiled pattern");
    last_turns(&memory.turns, look_back.as_int().unwrap_or(2)).iter().any(|turn| pattern.is_match(&_said(&turn.user)))
}

/// Python: bot/compose/rule.py:131 `_a_court_of_law_in_the_thread(memory, look_back=2)`
pub fn _a_court_of_law_in_the_thread(memory: &Memory, look_back: i64) -> bool {
    last_turns(&memory.turns, look_back).iter().any(|turn| _a_court_of_law(&_said(&turn.user)))
}

/// Python: bot/compose/rule.py:168 `_said_a_local_rule_recently(memory, look_back=2)`
pub fn _said_a_local_rule_recently(memory: &Memory, look_back: i64) -> bool {
    last_turns(&memory.turns, look_back).iter().any(|turn| A_LOCAL_RULE.is_match(&_said(&turn.user)))
}

/// Python: bot/compose/rule.py:174 `_a_rule_ask_in_the_thread(memory, look_back=2)`
///
/// True when one of the last few lines was itself a rule ask (`look_back` None is 2).
pub fn _a_rule_ask_in_the_thread(memory: &Memory, look_back: &Value) -> bool {
    for turn in last_turns(&memory.turns, look_back.as_int().unwrap_or(2)) {
        let earlier = _said(&turn.user);
        if earlier.is_empty() || NOT_A_RULE_ASK.is_match(&earlier) {
            continue;
        }
        if LAW_ASKED.is_match(&earlier) || WHO_OWNS_IT.is_match(&earlier) {
            return true;
        }
        if A_LOCAL_RULE.is_match(&earlier) && PERMISSION_ASKED.is_match(&earlier) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/rule.py:191 `pool_for(text, memory=None)`
///
/// The pool that names who owns this rule, or None when the line is not about one.
pub fn pool_for(text: &str, memory: &Memory) -> Option<Vec<String>> {
    let said = _said(text);
    if said.is_empty() || NOT_A_RULE_ASK.is_match(&said) {
        return None;
    }
    if _a_court_of_law(&said) || _a_court_of_law_in_the_thread(memory, 2) {
        return Some(UNKNOWN_A_COURT_THING.clone());
    }
    if LAW_ASKED.is_match(&said) {
        if A_LOCAL_RULE.is_match(&said) {
            return Some(UNKNOWN_A_LOCAL_RULE.clone());
        }
        return Some(UNKNOWN_THE_LAW.clone());
    }
    if A_FOLLOW_UP.is_match(&said) && _said_a_local_rule_recently(memory, 2) {
        if A_PENALTY.is_match(&said) || _a_rule_ask_in_the_thread(memory, &Value::Int(2)) {
            return Some(UNKNOWN_A_LOCAL_RULE.clone());
        }
    }
    None
}

/// Python: bot/compose/rule.py:207 `answer_for(text, memory=None)`
///
/// One line naming the rule's owner, or None when the line is not a rule ask.
pub fn answer_for(text: &str, memory: &Memory) -> Option<String> {
    let said = _said(text);
    let pool = pool_for(&said, memory).filter(|p| !p.is_empty())?;
    Some(pick(&pool, memory, &said))
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
        if let Some(got) = unless_unported(|| pool_for("is it legal to park on the pavement", &Memory::new())) {
            assert_eq!(got, Some(vec!["Rules like this are set by your council rather than nationally, so two roads can answer differently - which is why I would only be guessing at yours. Their own site has the exact wording and any charge, and their phone line settles it in a minute.".to_string(), "This one is your council's to set and enforce, and it changes from place to place. Their site has it in writing, and Citizens Advice is free if the page is a maze, which it often is.".to_string(), "The council writes this one, so theirs is the only wording that counts - and it varies by street, so a general answer would be no use to you. It is on their site with the details; ringing them is quicker than reading, and the number is on the page.".to_string()]));
        }
    }

    #[test]
    fn case_1() {
        if let Some(got) = unless_unported(|| pool_for("is it illegal to record a call", &Memory::new())) {
            assert_eq!(got, Some(vec!["There is an answer in law to that, and it turns on the exact situation rather than the general idea - which is the part I would get wrong. gov.uk has the wording in plain English, and Citizens Advice will read it against your own case for free.".to_string(), "The general rule is written down, but where it lands depends on your side of it, and that is exactly the bit I would only be guessing at. gov.uk has the wording, and Citizens Advice is free and will apply it to your situation.".to_string(), "I can tell you where this one is kept rather than what it says: law like this is published in full, and a summary from me could be wrong on a detail that matters. It is on gov.uk in plain words, and Citizens Advice will answer the same question about your own case free of charge.".to_string()]));
        }
    }

    #[test]
    fn case_2() {
        if let Some(got) = unless_unported(|| pool_for("will they fine me", &mem(&[("i put my bin out early, am i allowed to", "Ok.", "chat")]))) {
            assert_eq!(got, Some(vec!["Rules like this are set by your council rather than nationally, so two roads can answer differently - which is why I would only be guessing at yours. Their own site has the exact wording and any charge, and their phone line settles it in a minute.".to_string(), "This one is your council's to set and enforce, and it changes from place to place. Their site has it in writing, and Citizens Advice is free if the page is a maze, which it often is.".to_string(), "The council writes this one, so theirs is the only wording that counts - and it varies by street, so a general answer would be no use to you. It is on their site with the details; ringing them is quicker than reading, and the number is on the page.".to_string()]));
        }
    }

    #[test]
    fn case_3() {
        if let Some(got) = unless_unported(|| pool_for("how long is the tennis court open", &Memory::new())) {
            assert_eq!(got, None);
        }
    }

    #[test]
    fn case_4() {
        if let Some(got) = unless_unported(|| pool_for("what happens if i ignore the letter", &mem(&[("my ex is taking me to court", "Ok.", "chat")]))) {
            assert_eq!(got, Some(vec!["The letter is the thing that matters here: it names the date and the office that can answer for it, and that office is quicker than the paperwork looks. Ring it with the reference to hand, sooner rather than later - and Citizens Advice will read the letter with you first, free.".to_string(), "Whoever sent it knows what happens next and by when, and nothing I say changes a date that is already set. Ring the office named on the letter with the reference to hand; Citizens Advice will go through the wording with you first at no cost.".to_string(), "A court's next step is on the paperwork rather than in a general answer, and the office that sent it can tell you where you stand. The reference on the letter is what gets you through - and Citizens Advice will read it with you for free if the wording is unclear.".to_string()]));
        }
    }

    #[test]
    fn case_5() {
        if let Some(got) = unless_unported(|| pool_for("is it legal to give my kids paracetamol", &Memory::new())) {
            assert_eq!(got, None);
        }
    }
}
