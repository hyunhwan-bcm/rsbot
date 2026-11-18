//! Port of `bot/memory/a_year_of_birth.py`: a year of birth told in chat is the age, said the
//! long way.
//!
//! The year is filed as the age this chat would work out from it, and the store's own answer
//! does the rest. Only the person's own year is taken; a line that gives both a year and an age
//! keeps the age it states.

use super::mem8_py::{inline_re, pick, py_int, rx, strs};
use crate::data::Value;
use crate::py::re::I;
use crate::py::text;
use crate::state::Memory;
use chrono::{Datelike, NaiveDate};

const M: &str = "bot.memory.a_year_of_birth.";

strs!(A_BIRTH_YEAR);
rx!(_A_BORN, _SOMEONE_ELSE_BORN, _ALSO_AN_AGE);

inline_re!(
    AN_ASK,
    concat!(
        r"\?|\b(?:how|what|when|where|why|who|which|should|can|could|would|do|does|did|is|are|",
        r"will|any)\b"
    ),
    I
);

/// Python: bot/memory/a_year_of_birth.py:47 `_today()`
pub fn _today() -> NaiveDate {
    crate::py::clock::today()
}

/// Python: bot/memory/a_year_of_birth.py:51 `_the_store(memory)`
pub fn _the_store(memory: &mut Memory) -> Value {
    crate::memory::user(memory).clone()
}

/// Python: bot/memory/a_year_of_birth.py:56 `_a_slip(clean)`
///
/// True when the line is only the year of birth, with no ask after it.
pub fn _a_slip(clean: &str) -> bool {
    let mut rest = text::strip(clean).to_string();
    if let Some(born) = _A_BORN.search(&rest) {
        let cut = format!("{} {}", &rest[..born.start()], &rest[born.end()..]);
        rest = text::strip(&cut).to_string();
    }
    !AN_ASK.is_match(&rest)
}

/// Python: bot/memory/a_year_of_birth.py:67 `reply(text, memory)`
///
/// The answer to a line that only says the year they were born, or None.
pub fn reply(text: &str, memory: &mut Memory) -> Option<String> {
    let said = text;
    if said.is_empty() || !_A_BORN.is_match(said) || _SOMEONE_ELSE_BORN.is_match(said) {
        return None;
    }
    if !_a_slip(said) || text::split_ref(said).len() > 12 {
        return None;
    }
    // int(None) is a TypeError, int("x") a ValueError: both give None.
    let year = py_int(memory.get("born_year")?)?;
    let mut turned = _today().year() as i64 - year;
    // A line that says both ("i was born in 1990 and im 35") is worked out from the age it said,
    // which `observe` has already filed, so the reply does not contradict the line in front of it.
    // (`int(stated)` raising is swallowed by the Python `except Exception`.)
    let store = _the_store(memory);
    if let Some(stated) = store.get("age").filter(|v| !v.is_none()) {
        if let Some(age) = py_int(stated) {
            if 0 < age && age < 130 {
                turned = age;
            }
        }
    }
    if !(0 < turned && turned < 130) {
        return None;
    }
    let lines: Vec<String> = A_BIRTH_YEAR
        .iter()
        .map(|line| line.replace("{born}", &year.to_string()).replace("{turned}", &turned.to_string()))
        .collect();
    Some(pick(&lines, memory, said))
}

/// Python: bot/memory/a_year_of_birth.py:94 `observe(text, memory)`
///
/// File a year of birth the line gives about the person as the age it works out to.
pub fn observe(text: &str, memory: &mut Memory) {
    let said = text;
    if said.is_empty() {
        return;
    }
    let Some(born) = _A_BORN.search(said) else { return };
    if _SOMEONE_ELSE_BORN.is_match(said) {
        return;
    }
    let Some(year) = super::mem8_py::py_int_str(born.str(1)) else { return };
    let this_year = _today().year() as i64;
    if !(1900 <= year && year <= this_year) {
        return;
    }
    memory.set("born_year", year);
    if _ALSO_AN_AGE.is_match(said) {
        return;
    }
    let turned = this_year - year;
    if !(0 < turned && turned < 130) {
        return;
    }
    let facts = _the_store(memory);
    if facts.get("age").is_some() {
        return;
    }
    crate::memory::remember(memory, "age", &Value::Int(turned), said);
}
