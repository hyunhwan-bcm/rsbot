//! Port of `bot/compose/stretches.py`: a stretch the person is still inside ("ive been on hold
//! for two hours", "ive spent all day on this form"), said back in their own time with one
//! question, in place of a placeholder.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_sets, bundle_strs, inline_re};
use crate::data;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

bundle_res!("bot.compose.stretches"; BEEN_ON_A_STRETCH, SPENT_ON_A_STRETCH, TAKEN_A_STRETCH, VIGIL, UP_WITH_A_PERSON,
    WAITING_FOR_A_PERSON, TALKING_TO_A_PLACE);
bundle_strs!("bot.compose.stretches"; POINTER_STRETCH, VISITING_STRETCH, STRETCH_IS_FINISHED, QUEUE_STRETCH, WITH_THEIR_OWN,
    WAITING_AFTER_THE_STRETCH, NOT_A_SPENT_JOB, WAITING_STRETCH, CARING_STRETCH, EFFORT_STRETCH, NOT_A_STRETCH_JOB,
    WAITING_STRETCH_SAID_BACK, EFFORT_STRETCH_SAID_BACK, CARING_STRETCH_SAID_BACK, QUEUE_STRETCH_SAID_BACK,
    VISITING_STRETCH_SAID_BACK, VISITING_SOMEBODY_SAID_BACK, SOMEBODY_ELSE_IN_IT, WAITING_ON_THEIR_OWN,
    WAITING_ON_THEIR_OWN_SAID_BACK, WAITING_TO_TALK, TALKING_STRETCH_SAID_BACK);
bundle_sets!("bot.compose.stretches"; STRETCH_STARTS_UPPER);
inline_re!(A_WORD, r"[a-z']+");

static NEUTRAL: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.fallback.NEUTRAL"));
static A_SMALL_DAY: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.fallback_data.A_SMALL_DAY"));
static A_DAY_ASKED_PLAIN: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.fallback_data.A_DAY_ASKED_PLAIN"));

/// Python: bot/compose/stretches.py:273 `_the_stretch_said_back(phrase)`
///
/// The person's own stretch, written the way the reply says it back.
pub fn _the_stretch_said_back(phrase: &str) -> String {
    let squashed = text::squash(phrase);
    let mut words: Vec<String> = text::split(text::strip_chars(&squashed, " .,!?;:"));
    if words.len() > 1 && matches!(words[0].as_str(), "for" | "since" | "all" | "most" | "the") {
        for w in words.iter_mut().skip(1) {
            if STRETCH_STARTS_UPPER.contains(&w.to_lowercase()) {
                *w = text::capitalize(w);
            }
        }
    }
    if words.is_empty() {
        return String::new();
    }
    text::upper_first(&words.join(" "))
}

/// Python: bot/compose/stretches.py:286 `_names_one_of(doing, phrases)`
///
/// Whether the doing holds one of the phrases, as whole words.
pub fn _names_one_of<S: AsRef<str>>(doing: &str, phrases: &[S]) -> bool {
    let words = text::split_ref(doing);
    phrases.iter().any(|p| {
        let p = p.as_ref();
        (p.contains(' ') && doing.contains(p)) || words.contains(&p)
    })
}

/// Python: bot/compose/stretches.py:292 `_the_shape(doing, said, after)`
///
/// Which pool this stretch belongs in, or None when it is not one of them.
pub fn _the_shape(doing: &str, said: &str, after: &str) -> Option<String> {
    let shape = |s: &str| Some(s.to_string());
    if _names_one_of(doing, &NOT_A_STRETCH_JOB) {
        return None;
    }
    if _names_one_of(doing, &CARING_STRETCH) || UP_WITH_A_PERSON.search(said).is_some() {
        return shape("caring");
    }
    if _names_one_of(doing, &QUEUE_STRETCH) || _names_one_of(said, &QUEUE_STRETCH) {
        return shape("queueing");
    }
    if _names_one_of(doing, &WAITING_STRETCH) {
        if _names_one_of(said, &WAITING_TO_TALK) && TALKING_TO_A_PLACE.search(said).is_none() {
            // The wait is for company, not for an answer.
            return shape("talking");
        }
        if _names_one_of(doing, &WAITING_ON_THEIR_OWN) || _names_one_of(after, &WAITING_ON_THEIR_OWN) || WAITING_FOR_A_PERSON.search(said).is_some() {
            return shape("waiting_on_their_own");
        }
        return shape("waiting");
    }
    if _names_one_of(said, &VISITING_STRETCH) || _names_one_of(said, &WITH_THEIR_OWN) {
        return shape("visiting");
    }
    if _names_one_of(doing, &EFFORT_STRETCH) {
        return shape("effort");
    }
    if !doing.is_empty() && _names_one_of(doing, &POINTER_STRETCH) {
        return shape("effort");
    }
    if _names_one_of(after, &WAITING_AFTER_THE_STRETCH) || _names_one_of(said, &WAITING_AFTER_THE_STRETCH) {
        return shape(if _names_one_of(said, &QUEUE_STRETCH) { "queueing" } else { "waiting" });
    }
    None
}

/// Python: bot/compose/stretches.py:326 `reads_as_a_long_stretch(clean)`
///
/// (shape, the stretch to say back) for a stretch the person is still inside, or None.
pub fn reads_as_a_long_stretch(clean: &str) -> Option<(String, String)> {
    let said = text::squash(&clean.to_lowercase());
    if said.is_empty() || said.contains('?') {
        return None;
    }
    for (pattern, spent) in [(&*BEEN_ON_A_STRETCH, false), (&*SPENT_ON_A_STRETCH, true)] {
        for met in pattern.finditer(&said) {
            let doing = text::strip_chars(met.name("doing").unwrap_or(""), " ,.;:").to_string();
            if text::split_ref(&doing).len() > 8 || doing.contains(" and ") {
                continue;
            }
            let when = _the_stretch_said_back(met.name_str("when"));
            if when.is_empty() {
                continue;
            }
            if _names_one_of(&said, &NOT_A_STRETCH_JOB) || VIGIL.search(&said).is_some() {
                continue;
            }
            if _names_one_of(&said, &STRETCH_IS_FINISHED) {
                continue;
            }
            if spent && _names_one_of(&said, &NOT_A_SPENT_JOB) {
                continue;
            }
            let shape = _the_shape(&doing, &said, &said[met.end()..]);
            // A day gone on one job takes the pool every other job takes.
            if shape.is_some() || spent {
                return Some((shape.unwrap_or_else(|| "effort".to_string()), when));
            }
        }
    }
    for met in TAKEN_A_STRETCH.finditer(&said) {
        let when = _the_stretch_said_back(met.name_str("when"));
        if when.is_empty() {
            continue;
        }
        if _names_one_of(&said, &NOT_A_STRETCH_JOB) || _names_one_of(&said, &NOT_A_SPENT_JOB) {
            continue;
        }
        if _names_one_of(&said, &STRETCH_IS_FINISHED) {
            continue;
        }
        return Some(("effort".to_string(), when));
    }
    None
}

/// Python: bot/compose/stretches.py:369 `the_stretch_pool(shape, when, said='')`
///
/// The three lines for this kind of stretch, with the person's own time in them.
pub fn the_stretch_pool(shape: &str, when: &str, said: &str) -> Vec<String> {
    let shape = if shape == "visiting" && _names_one_of(said, &SOMEBODY_ELSE_IN_IT) { "visiting_somebody" } else { shape };
    let pool: &[String] = match shape {
        "waiting" => &WAITING_STRETCH_SAID_BACK,
        "effort" => &EFFORT_STRETCH_SAID_BACK,
        "caring" => &CARING_STRETCH_SAID_BACK,
        "queueing" => &QUEUE_STRETCH_SAID_BACK,
        "visiting" => &VISITING_STRETCH_SAID_BACK,
        "visiting_somebody" => &VISITING_SOMEBODY_SAID_BACK,
        "waiting_on_their_own" => &WAITING_ON_THEIR_OWN_SAID_BACK,
        "talking" => &TALKING_STRETCH_SAID_BACK,
        _ => &[],
    };
    pool.iter().map(|l| l.replace("{When}", when)).collect()
}

/// Python: bot/compose/stretches.py:388 `_only_a_placeholder(text, said)`
///
/// True when the reply tells the person nothing of its own about the line: a stock line, a
/// hedge, a receipt, a small-day line, or the line handed back.
pub fn _only_a_placeholder(text_: &str, said: &str) -> bool {
    let said = text::squash(said);
    if said.is_empty() {
        return true;
    }
    if NEUTRAL.contains(&said) || crate::compose::fallback::_is_a_hedge(&said) || crate::memory::ack::reads_as_a_receipt(&said) {
        return true;
    }
    if A_SMALL_DAY.contains(&said) || A_DAY_ASKED_PLAIN.contains(&said) {
        return true;
    }
    let mine: HashSet<String> = A_WORD.findall_str(&text_.to_lowercase()).into_iter().collect();
    let theirs: HashSet<String> = A_WORD.findall_str(&said.to_lowercase()).into_iter().collect();
    if mine.len() < 2 {
        return false;
    }
    let shared = mine.intersection(&theirs).count();
    shared >= 3 && shared as f64 / mine.len() as f64 >= 0.6
}

/// Python: bot/compose/stretches.py:418 `the_stretch_said_back(text, memory, said)`
///
/// The reply for a stretch the person is still inside, or "" when this is not that line.
pub fn the_stretch_said_back(text_: &str, memory: &Memory, said: &str) -> String {
    if !_only_a_placeholder(text_, said) {
        return String::new();
    }
    let Some((shape, when)) = reads_as_a_long_stretch(text_) else { return String::new() };
    let clean = text::squash(&text_.to_lowercase());
    crate::compose::fallback_chain::_choose(&the_stretch_pool(&shape, &when, &clean), memory, &clean, None).unwrap_or_default()
}
