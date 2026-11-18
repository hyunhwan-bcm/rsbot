//! Port of `bot/knowledge/newborn.py`: the first fortnight with a new baby, and a birth weight.

use crate::data::FnRef;
use crate::knowledge::d2dup_g10::globals;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

globals!("bot.knowledge.newborn",
    re { _SOMEONE_ELSE_BABY, _DAYS_OLD, _WEEKS_OLD, _WEIGHT_SAID },
    strs { _NOT_THE_FORTNIGHT },
    string { _BIRTH_WEIGHT });
globals!("bot.knowledge.family", re { _BIRTH_IN_THE_CHAT });

static THIRD_PERSON_NEWS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:she|he|they)\s+(?:was|is|were|has been)\b", 0));
static MY_OR_OUR: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:my|our)\b", 0));
static OWN_BABY: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(?:my|our)\s+(?:baby|son|daughter|newborn)\b|\bi (?:had|gave)\b", 0));
static MONTHS_OLD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:[2-9]|1[0-9]|2[0-4])\s+months?\s+old\b", 0));
static YEARS_OLD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\byears?\s+old\b", 0));

/// The line's own words, lowercased: the reading rewrites some lines and the text is truest.
pub fn _the_line_said(understood: &Understood) -> String {
    for said in [&understood.text, &understood.clean, &understood.plain] {
        if !text::strip(said).is_empty() {
            return text::squash(&said.to_lowercase());
        }
    }
    String::new()
}

/// The number a `(\d{1,2})` age pattern caught.
fn age_in(rx: &Re, said: &str) -> Option<i64> {
    rx.search(said).map(|m| m.str(1).parse().unwrap_or(0))
}

/// Whether this line is the news of a new baby, or the age of one, and nothing else.
pub fn _the_line_is_the_news(understood: &Understood) -> bool {
    let said = _the_line_said(understood);
    if said.is_empty() {
        return false;
    }
    if THIRD_PERSON_NEWS.is_match(&said) && !MY_OR_OUR.is_match(&said) {
        // somebody else's news: the fortnight card is advice written for the parent
        return false;
    }
    if _SOMEONE_ELSE_BABY.is_match(&said) && !OWN_BABY.is_match(&said) {
        return false;
    }
    if _is_an_older_baby(&said) {
        return false;
    }
    if _NOT_THE_FORTNIGHT.iter().any(|w| said.contains(w.as_str())) {
        return false;
    }
    true
}

/// Whether the line gives the baby an age past the first fortnight.
pub fn _is_an_older_baby(said: &str) -> bool {
    if age_in(&_DAYS_OLD, said).is_some_and(|d| d > 14) {
        return true;
    }
    if age_in(&_WEEKS_OLD, said).is_some_and(|w| w > 2) {
        return true;
    }
    MONTHS_OLD.is_match(said) || YEARS_OLD.is_match(said)
}

/// Whether the line says an age in days, or in the first two weeks, for a baby.
pub fn _is_a_newborn(said: &str) -> bool {
    if age_in(&_DAYS_OLD, said).is_some_and(|d| d <= 14) {
        return true;
    }
    if age_in(&_WEEKS_OLD, said).is_some_and(|w| w <= 2) {
        return true;
    }
    false
}

/// Whether a baby was born in the line, or in the turn or two before it.
pub fn _a_birth_just_happened(memory: &Memory, understood: &Understood) -> bool {
    let mut said = vec![_the_line_said(understood)];
    let turns = &memory.turns;
    for turn in &turns[turns.len().saturating_sub(2)..] {
        said.push(turn.user.to_lowercase());
    }
    _BIRTH_IN_THE_CHAT.is_match(&said.join(" "))
}

/// The range a birth weight is normal in, for a weight said in a chat with a new baby.
pub fn _the_birth_weight(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = _the_line_said(understood);
    if said.is_empty() || !_WEIGHT_SAID.is_match(&said) {
        return None;
    }
    if said.contains("birth") {
        return Some(_BIRTH_WEIGHT.clone());
    }
    let days = age_in(&_DAYS_OLD, &said);
    let weeks = age_in(&_WEEKS_OLD, &said);
    // "my baby is 8 weeks old and weighs 5 kilos" is a growing baby, not a birth weight
    if days.is_some_and(|d| d > 21) || weeks.is_some_and(|w| w > 6) {
        return None;
    }
    if !_a_birth_just_happened(memory, understood) {
        return None;
    }
    Some(_BIRTH_WEIGHT.clone())
}

pub const GUARDS: &[(&str, GuardFn)] =
    &[("bot.knowledge.newborn:_the_line_is_the_news:114", |_: &FnRef, u: &Understood| _the_line_is_the_news(u))];

pub const REPLIES: &[(&str, ReplyFn)] = &[(
    "bot.knowledge.newborn:_the_birth_weight:180",
    |_: &FnRef, m: &mut Memory, u: &Understood| _the_birth_weight(m, u),
)];
