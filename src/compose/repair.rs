//! Port of `bot/compose/repair.py`: a deed the person reports doing to somebody close ("i let
//! my mum down", "i shouted at my son") and the asks that follow it, read as one repair.

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.repair.", stringify!($name))));)*
    };
}
regexes!(
    NOT_A_DEED, NOBODY_TO_REPAIR, PHYSICAL_HARM, PET_OR_PROFESSIONAL, _REGRET_FIRST, _FEELING_FIRST,
    _GRIEF, _A_BABY, AMENDS, ASKS_FOR_THE_WORDS, ASKS_WHAT_TO_DO,
);
/// `DEEDS` / `LAPSES`: (deed, pattern), in order.
fn rows(name: &str) -> Vec<(String, &'static Re)> {
    data::get(name)
        .items()
        .iter()
        .map(|row| {
            let row: &'static [Value] = row.items();
            (row[0].str().to_string(), row[1].as_re().unwrap())
        })
        .collect()
}
static DEEDS: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| rows("bot.compose.repair.DEEDS"));
static LAPSES: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| rows("bot.compose.repair.LAPSES"));
static THE_FEELING_NAMED: LazyLock<String> = LazyLock::new(|| data::string("bot.compose.repair.THE_FEELING_NAMED"));
static THE_FEELING_NAMED_DATED: LazyLock<String> =
    LazyLock::new(|| data::string("bot.compose.repair.THE_FEELING_NAMED_DATED"));

/// `" ".join(str(clean).lower().replace("’", "'").split())`
fn flat(clean: &str) -> String {
    text::squash(&text::lower(clean).replace('\u{2019}', "'"))
}

/// A reading's line: `clean.get("clean") or clean.get("text") or ""` for a dict, else the str.
fn the_words(clean: &Value) -> String {
    match clean {
        Value::Dict(_) => {
            let pick = |k: &str| clean.get(k).filter(|v| v.truthy()).map(|v| v.str().to_string());
            pick("clean").or_else(|| pick("text")).unwrap_or_default()
        }
        Value::None => String::new(),
        other => other.as_str().unwrap_or("").to_string(),
    }
}

/// `deed_of` on a line of text.
fn deed_of_text(text: &str) -> Option<String> {
    let text = flat(text);
    if text.is_empty() {
        return None;
    }
    if PET_OR_PROFESSIONAL.is_match(&text) || PHYSICAL_HARM.is_match(&text) {
        return None;
    }
    if _GRIEF.is_match(&text) || _A_BABY.is_match(&text) || _REGRET_FIRST.is_match(&text) {
        return None;
    }
    if let Some((deed, _)) = LAPSES.iter().find(|(_, pattern)| pattern.is_match(&text)) {
        return Some(deed.clone());
    }
    if NOT_A_DEED.is_match(&text) || NOBODY_TO_REPAIR.is_match(&text) {
        return None;
    }
    DEEDS.iter().find(|(_, pattern)| pattern.is_match(&text)).map(|(deed, _)| deed.clone())
}

/// Python: bot/compose/repair.py:290 `deed_of(clean)`
///
/// The deed the person reports doing to somebody, or None. `clean` is the line (a str) or a
/// reading with "clean" in it (a dict).
pub fn deed_of(clean: &Value) -> Option<String> {
    deed_of_text(&the_words(clean))
}

/// Python: bot/compose/repair.py:319 `owns_the_deed(line)`
///
/// True when the line reports something the person did to somebody close.
pub fn owns_the_deed(line: &Value) -> bool {
    deed_of(line).is_some()
}

/// Python: bot/compose/repair.py:338 `with_the_feeling_named(reply, clean)`
///
/// The deed's card under a sentence that names the feeling the line led with (a str).
pub fn with_the_feeling_named(reply: &str, clean: &str) -> String {
    if reply.is_empty() {
        return reply.to_string();
    }
    let text = flat(clean);
    if !_FEELING_FIRST.is_match(&text) {
        return reply.to_string();
    }
    let lead = if deed_of_text(&text).as_deref() == Some("forgot a date") { &*THE_FEELING_NAMED_DATED } else { &*THE_FEELING_NAMED };
    let mut chars = reply.chars();
    let first = chars.next().map(|c| c.to_lowercase().collect::<String>()).unwrap_or_default();
    format!("{lead}{first}{}", chars.as_str())
}

/// Python: bot/compose/repair.py:354 `line_of(reading)`
///
/// The line as a reading holds it: `clean`, else `text`, else the words joined.
pub fn line_of(reading: &Understood) -> String {
    if !reading.clean.is_empty() {
        return reading.clean.clone();
    }
    if !reading.text.is_empty() {
        return reading.text.clone();
    }
    reading.words.join(" ")
}

/// Python: bot/compose/repair.py:370 `amends_to(clean)`
///
/// True when the line asks how to make amends to somebody, without naming a deed.
pub fn amends_to(clean: &str) -> bool {
    let text = flat(clean);
    if text.is_empty() || !AMENDS.is_match(&text) {
        return false;
    }
    !(PET_OR_PROFESSIONAL.is_match(&text) || PHYSICAL_HARM.is_match(&text))
}

/// Python: bot/compose/repair.py:400 `a_repair_is_open(memory)`
///
/// True when the turns just said are one repair, with the asks in the middle of it.
pub fn a_repair_is_open(memory: &Memory) -> bool {
    let turns = &memory.turns[memory.turns.len().saturating_sub(3)..];
    for turn in turns.iter().rev() {
        let said = text::squash(&text::lower(&turn.user));
        if said.is_empty() {
            continue;
        }
        if deed_of_text(&said).is_some() || amends_to(&said) {
            return true;
        }
        if text::split_ref(&said).len() > 9 || !(ASKS_FOR_THE_WORDS.is_match(&said) || ASKS_WHAT_TO_DO.is_match(&said)) {
            return false;
        }
    }
    false
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
        if let Some(got) = unless_unported(|| deed_of(&Value::from("i let my mum down"))) {
            assert_eq!(got, Some("let down".to_string()));
        }
    }

    #[test]
    fn case_1() {
        if let Some(got) = unless_unported(|| amends_to("i let my mum down")) {
            assert_eq!(got, false);
        }
    }

    #[test]
    fn case_2() {
        if let Some(got) = unless_unported(|| deed_of(&Value::from("i shouted at my son"))) {
            assert_eq!(got, Some("shouted at them".to_string()));
        }
    }

    #[test]
    fn case_3() {
        if let Some(got) = unless_unported(|| amends_to("i shouted at my son")) {
            assert_eq!(got, false);
        }
    }

    #[test]
    fn case_4() {
        if let Some(got) = unless_unported(|| deed_of(&Value::from("i forgot my wifes birthday"))) {
            assert_eq!(got, Some("forgot a date".to_string()));
        }
    }

    #[test]
    fn case_5() {
        if let Some(got) = unless_unported(|| amends_to("i forgot my wifes birthday")) {
            assert_eq!(got, false);
        }
    }

    #[test]
    fn case_6() {
        if let Some(got) = unless_unported(|| deed_of(&Value::from("i hurt my back"))) {
            assert_eq!(got, None);
        }
    }

    #[test]
    fn case_7() {
        if let Some(got) = unless_unported(|| amends_to("i hurt my back")) {
            assert_eq!(got, false);
        }
    }

    #[test]
    fn case_8() {
        if let Some(got) = unless_unported(|| deed_of(&Value::from("i shouted at the dog"))) {
            assert_eq!(got, None);
        }
    }

    #[test]
    fn case_9() {
        if let Some(got) = unless_unported(|| amends_to("i shouted at the dog")) {
            assert_eq!(got, false);
        }
    }

    #[test]
    fn case_10() {
        if let Some(got) = unless_unported(|| deed_of(&Value::from("i never lied to her"))) {
            assert_eq!(got, None);
        }
    }

    #[test]
    fn case_11() {
        if let Some(got) = unless_unported(|| amends_to("i never lied to her")) {
            assert_eq!(got, false);
        }
    }

    #[test]
    fn case_12() {
        if let Some(got) = unless_unported(|| deed_of(&Value::from("i didnt call my mum back"))) {
            assert_eq!(got, Some("left it too long".to_string()));
        }
    }

    #[test]
    fn case_13() {
        if let Some(got) = unless_unported(|| amends_to("i didnt call my mum back")) {
            assert_eq!(got, false);
        }
    }

    #[test]
    fn case_14() {
        if let Some(got) = unless_unported(|| deed_of(&Value::from("i feel sick that i lied to her"))) {
            assert_eq!(got, Some("lied".to_string()));
        }
    }

    #[test]
    fn case_15() {
        if let Some(got) = unless_unported(|| amends_to("i feel sick that i lied to her")) {
            assert_eq!(got, false);
        }
    }

    #[test]
    fn case_16() {
        if let Some(got) = unless_unported(|| deed_of(&Value::from("i regret shouting at him"))) {
            assert_eq!(got, None);
        }
    }

    #[test]
    fn case_17() {
        if let Some(got) = unless_unported(|| amends_to("i regret shouting at him")) {
            assert_eq!(got, false);
        }
    }

    #[test]
    fn case_18() {
        if let Some(got) = unless_unported(|| amends_to("how do i make it up to my mum")) {
            assert_eq!(got, true);
        }
    }

    #[test]
    fn case_19() {
        if let Some(got) = unless_unported(|| amends_to("how can i make amends")) {
            assert_eq!(got, true);
        }
    }

    #[test]
    fn case_20() {
        if let Some(got) = unless_unported(|| amends_to("make it up to the dog")) {
            assert_eq!(got, false);
        }
    }

    #[test]
    fn case_21() {
        if let Some(got) = unless_unported(|| with_the_feeling_named("Say sorry soon.", "i feel sick that i lied to her")) {
            assert_eq!(got, "Guilt like that is doing its job if it moves on: say sorry soon.".to_string());
        }
    }

    #[test]
    fn case_22() {
        if let Some(got) = unless_unported(|| with_the_feeling_named("Say sorry soon.", "i lied and i feel awful")) {
            assert_eq!(got, "Say sorry soon.".to_string());
        }
    }

    #[test]
    fn case_23() {
        if let Some(got) = unless_unported(|| with_the_feeling_named("Send a card.", "i feel awful about forgetting her birthday, i forgot my wifes birthday")) {
            assert_eq!(got, "Send a card.".to_string());
        }
    }

    #[test]
    fn case_24() {
        if let Some(got) = unless_unported(|| a_repair_is_open(&mem(&[("i made my mum cry", "Oh.", "chat"), ("i dont know what to say", "Say sorry.", "chat")]))) {
            assert_eq!(got, true);
        }
    }

    #[test]
    fn case_25() {
        if let Some(got) = unless_unported(|| a_repair_is_open(&mem(&[("the garden is lovely", "Nice.", "chat")]))) {
            assert_eq!(got, false);
        }
    }
}
