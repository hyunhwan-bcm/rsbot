//! Port of `bot/compose/nottheirbest.py`: a day denied ("not my best day") or graded low ("could
//! be better") gets the bad-week pool instead of a congratulation, a shrug or a hedge.

use crate::compose::fallback;
use crate::data;
use crate::memory::narrow::memory_value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.nottheirbest.", stringify!($name))));)*
    };
}
regexes!(A_DAY_DENIED, A_HAD_BETTER, A_DAY_OWNS_THE_GRADE, A_DAY_GRADED_ALONE, A_CHEER, A_SHRUG);
static QUESTION_OPENERS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.nottheirbest.QUESTION_OPENERS"));
static BAD_STRETCH_REPLIES: LazyLock<Vec<String>> =
    LazyLock::new(|| data::strs("bot.compose.nottheirbest.BAD_STRETCH_REPLIES"));

/// `from . import pick`: its contract takes the memory as a `Value`.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, &memory_value(memory), seed_text)
}

/// Python: bot/compose/nottheirbest.py:69 `reads_as_a_low_day(text)`
///
/// True when the line is the person's own day or week, denied or graded low by comparison.
pub fn reads_as_a_low_day(text: &str) -> bool {
    let clean = text::squash(&text::lower(text));
    if clean.is_empty() {
        return false;
    }
    let first = text::split_ref(&clean).first().copied().unwrap_or("");
    A_DAY_GRADED_ALONE.is_match(&clean)
        || ((A_DAY_DENIED.is_match(&clean) || A_HAD_BETTER.is_match(&clean) || A_DAY_OWNS_THE_GRADE.is_match(&clean))
            && !QUESTION_OPENERS.contains(first))
}

/// Python: bot/compose/nottheirbest.py:80 `_may_replace(reply)`
///
/// True when the composer's line says the wrong thing about a day like this: a hedge, a
/// congratulation or a shrug of an ack.
pub fn _may_replace(reply: &str) -> bool {
    if reply.is_empty() {
        return false;
    }
    if fallback::is_a_hedge(reply) {
        return true;
    }
    if reply.starts_with("Noted:") {
        return true;
    }
    A_CHEER.is_match(reply) || A_SHRUG.is_match(reply)
}

/// Python: bot/compose/nottheirbest.py:95 `answer_for(text, reply, memory=None)`
///
/// The bad-week pool for a day denied or graded low, or None.
pub fn answer_for(text: &str, reply: &str, memory: &Memory) -> Option<String> {
    if !reads_as_a_low_day(text) {
        return None;
    }
    if !_may_replace(reply) {
        return None;
    }
    Some(pick(&BAD_STRETCH_REPLIES, memory, text))
}
