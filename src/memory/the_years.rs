//! Port of `bot/memory/the_years.py`: "i feel so old", "im 40 next month", "im past it" - a
//! remark about the years, answered as one, unless the line has a bigger half of its own.

use super::mem8_py::{int, pick, rx, strs};
use super::passing_the_time::_the_chat_is_heavy;
use crate::py::re::{Re, I};
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

const M: &str = "bot.memory.the_years.";

rx!(_FORGETTING, _TREATMENT, _OWN_SUBJECT, _DISLIKE, _FEELING, _SELF, _OTHER, _BODY_CLAIM, _ASK_START, _BIRTHDAY);
int!(_LIMIT);
strs!(OLD_LINES, PAST_IT_LINES, TOO_OLD_LINES, MY_AGE_LINES, BIRTHDAY_LINES, COMPARISON_LINES);

static ITS_THE_YEARS: LazyLock<Re> = LazyLock::new(|| Re::new(r"^\W*(?:the years|it is|its|age)\b", I));
static PAST_IT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:past|passed) it\b|\bover the hill\b", I));
static TOO_OLD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\btoo old\b", I));
static COMPARED: LazyLock<Re> = LazyLock::new(|| Re::new(r"look at|young people|teenagers|schoolchildren|all these kids", I));
static MY_AGE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bmy age\b", I));

/// Python: bot/memory/the_years.py:158 `_the_turn(clean)`
///
/// The birthday the line names ("40 next month", "turning 50"), or None.
pub fn _the_turn(clean: &str) -> Option<String> {
    _BIRTHDAY.search(clean).map(|found| text::squash(found.as_str()))
}

/// Python: bot/memory/the_years.py:166 `reply(text, memory)`
///
/// The reply to a remark about their years, or None.
pub fn reply(text: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&text.replace('\u{2019}', "'"));
    if clean.is_empty() || text::split_ref(&clean).len() as i64 > *_LIMIT {
        return None;
    }
    if clean.contains('?') || _ASK_START.match_(&clean).is_some() {
        return None;
    }
    if !_FEELING.is_match(&clean) {
        return None;
    }
    let said = _SELF.is_match(&clean) || ITS_THE_YEARS.match_(&clean).is_some();
    if !said || _OTHER.is_match(&clean) || _BODY_CLAIM.is_match(&clean) {
        return None;
    }
    if _FORGETTING.is_match(&clean) {
        // The forgetting has a card of its own.
        return None;
    }
    if _TREATMENT.is_match(&clean) || _OWN_SUBJECT.is_match(&clean) || _DISLIKE.is_match(&clean) {
        // The rest of the line has an answer of its own, and it is the bigger half.
        return None;
    }
    if _the_chat_is_heavy(memory) {
        return None;
    }
    let seed = text;
    if let Some(turn) = _the_turn(&clean).filter(|t| !t.is_empty()) {
        // The birthday is part of the line, so the frame is filled before the pool is picked.
        let turn = text::upper_first(&turn);
        let pool: Vec<String> = BIRTHDAY_LINES.iter().map(|line| line.replace("{turn}", &turn)).collect();
        return Some(pick(&pool, memory, seed));
    }
    let pool: &[String] = if PAST_IT.is_match(&clean) {
        &PAST_IT_LINES
    } else if TOO_OLD.is_match(&clean) {
        &TOO_OLD_LINES
    } else if COMPARED.is_match(&clean) {
        &COMPARISON_LINES
    } else if MY_AGE.is_match(&clean) {
        &MY_AGE_LINES
    } else {
        &OLD_LINES
    };
    Some(pick(pool, memory, seed))
}
