//! Port of `bot/compose/a_first_time.py`: the one thing that helps a first time at a task.

use crate::compose::fallback_chain;
use crate::data;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.a_first_time.", stringify!($name))));)*
    };
}
regexes!(A_FIRST_TIME, A_FIRST_FENCED, A_A_TASK, A_FIRST_NAMES_ITS_THING);
static FIRST_TIME: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.a_first_time.FIRST_TIME"));

/// Python: bot/compose/a_first_time.py:53 `_a_task_they_faced(text, memory)` (never called on the trace corpus)
///
/// True when the first they are talking about is a task, in this line or the one before.
pub fn _a_task_they_faced(text: &str, memory: &Memory) -> bool {
    if A_A_TASK.is_match(text) {
        return true;
    }
    if A_FIRST_NAMES_ITS_THING.is_match(text) {
        return false;
    }
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(4)..]
        .iter()
        .rev()
        .any(|turn| !turn.user.is_empty() && A_A_TASK.is_match(&turn.user))
}

/// Python: bot/compose/a_first_time.py:77 `reply_for(text, memory=None)`
///
/// The one thing that helps a first time, or None for the floor to answer as before.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    if text.is_empty() {
        return None;
    }
    let low = text::squash(&text::lower(text));
    if low.is_empty() || low.ends_with('?') {
        return None;
    }
    if !A_FIRST_TIME.is_match(&low) || A_FIRST_FENCED.is_match(&low) {
        return None;
    }
    if !_a_task_they_faced(&low, memory) {
        return None;
    }
    fallback_chain::_choose(&FIRST_TIME, memory, &low, None)
}
