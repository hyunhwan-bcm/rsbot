//! Port of `bot/memory/passing_the_time.py`: a line about the moment a person is in ("on the
//! bus", "waiting at the dentist", "lazy day today"), answered as the small talk it is - unless
//! the chat is carrying something heavy, or the line is a habit or an appointment.

use super::mem8_py::{pick, rx, strs};
use crate::py::text;
use crate::state::Memory;

const M: &str = "bot.memory.passing_the_time.";

rx!(
    _WAITING, _MOVING, _IN_THE_CAR, _SLOW, _IN_BED, _SOFA, _ON_A_BREAK, _AT_WORK, _AT_THE_GYM, _AT_A_DOCTOR,
    _AT_A_PLACE, _NOT_A_PLACE, _HEAVY, A_STRETCH_OF_TIME, A_DAY_TO_COME
);
strs!(
    WAITING_REPLIES, MOVING_REPLIES, IN_THE_CAR_REPLIES, SLOW_REPLIES, IN_BED_REPLIES, SOFA_REPLIES,
    BREAK_REPLIES, AT_WORK_REPLIES, AT_THE_GYM_REPLIES, AT_A_DOCTOR_REPLIES, AT_A_PLACE_REPLIES
);

/// Python: bot/memory/passing_the_time.py:259 `_the_chat_is_heavy(memory)`
///
/// Whether one of the last two lines the person said was a disclosure or a strong feeling.
pub fn _the_chat_is_heavy(memory: &Memory) -> bool {
    let earlier: Vec<&str> =
        memory.turns.iter().map(|t| t.user.as_str()).filter(|u| !text::strip(u).is_empty()).collect();
    earlier[earlier.len().saturating_sub(2)..].iter().any(|said| _HEAVY.is_match(&text::squash(said)))
}

/// Python: bot/memory/passing_the_time.py:277 `reply(text, memory)`
///
/// The reply to a line about the moment they are in, or None.
pub fn reply(text: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&text.replace('\u{2019}', "'"));
    if clean.is_empty() || text::split_ref(&clean).len() > 9 {
        return None;
    }
    if _the_chat_is_heavy(memory) {
        // Left to the module that carries the feeling on.
        return None;
    }
    if A_STRETCH_OF_TIME.is_match(&clean) || A_DAY_TO_COME.is_match(&clean) {
        // A habit or an appointment, not the moment they are in: the store keeps the fact.
        return None;
    }
    let seed = text;
    // Read in this order, so a line with a better reading earlier keeps it.
    let readings: [(&crate::py::re::Re, &[String]); 10] = [
        (*_ON_A_BREAK, &BREAK_REPLIES),
        (*_WAITING, &WAITING_REPLIES),
        (*_IN_THE_CAR, &IN_THE_CAR_REPLIES),
        (*_MOVING, &MOVING_REPLIES),
        (*_IN_BED, &IN_BED_REPLIES),
        (*_SOFA, &SOFA_REPLIES),
        (*_SLOW, &SLOW_REPLIES),
        (*_AT_WORK, &AT_WORK_REPLIES),
        (*_AT_THE_GYM, &AT_THE_GYM_REPLIES),
        (*_AT_A_DOCTOR, &AT_A_DOCTOR_REPLIES),
    ];
    for (pattern, pool) in readings {
        if pattern.match_(&clean).is_some() {
            return Some(pick(pool, memory, seed));
        }
    }
    if _AT_A_PLACE.match_(&clean).is_some() && !_NOT_A_PLACE.is_match(&clean) {
        return Some(pick(&AT_A_PLACE_REPLIES, memory, seed));
    }
    None
}
