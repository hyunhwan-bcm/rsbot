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
