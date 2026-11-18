//! Port of `bot/compose/talktime.py`: a talk asked for at a time ("can we talk tomorrow") gets
//! the honest line - no diary and no clock here - and the offer of the talk now.

use crate::compose::fallback_chain;
use crate::data;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.talktime.", stringify!($name))));)*
    };
}
regexes!(A_COMPANY_ASK, A_TALK_AT_A_TIME_ASK, A_TALK_AT_A_TIME_HARM);
static A_TALK_AT_A_TIME: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.talktime.A_TALK_AT_A_TIME"));

/// Python: bot/compose/talktime.py:108 `answer_for(text, memory=None)`
///
/// The honest line for a talk asked for at a time, or None when the line is something else.
pub fn answer_for(text: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&text::lower(text));
    if clean.is_empty() {
        return None;
    }
    if !(A_TALK_AT_A_TIME_ASK.match_(&clean).is_some() || A_COMPANY_ASK.match_(&clean).is_some()) {
        return None;
    }
    if A_TALK_AT_A_TIME_HARM.is_match(&clean) {
        return None;
    }
    fallback_chain::_choose(&A_TALK_AT_A_TIME, memory, &clean, None)
}
