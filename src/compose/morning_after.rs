//! Port of `bot/compose/morning_after.py`: a night of drinking that has already happened, the
//! person's own, with nothing after it that is a subject of its own.

use crate::data;
use crate::py::re::Re;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.morning_after.", stringify!($name))));)*
    };
}
regexes!(
    A_STRONG_STATE, A_WEAK_STATE, A_TIME_THAT_HAS_GONE, A_HANGOVER, A_MORNING, A_CONSEQUENCE,
    A_STILL_TO_COME, SOMEONE_ELSE, AN_EMERGENCY, A_QUESTION_ABOUT_IT,
);

/// Python: bot/compose/morning_after.py:117 `reads_as_a_morning_after(clean)`
///
/// True when the line reports a night of drinking that has already happened.
pub fn reads_as_a_morning_after(clean: &str) -> bool {
    if clean.is_empty() {
        return false;
    }
    if AN_EMERGENCY.is_match(clean) || A_QUESTION_ABOUT_IT.is_match(clean) {
        return false;
    }
    if A_CONSEQUENCE.is_match(clean) {
        return false;
    }
    let gone = A_TIME_THAT_HAS_GONE.search(clean);
    let strong = A_STRONG_STATE.search(clean);
    let weak = if gone.is_some() { A_WEAK_STATE.search(clean) } else { None };
    let hangover = if A_MORNING.is_match(clean) { A_HANGOVER.search(clean) } else { None };
    let Some(said) = strong.or(weak).or(hangover) else { return false };
    // The state has to be the person's own.
    if SOMEONE_ELSE.is_match(&clean[..said.start()]) {
        return false;
    }
    if gone.is_none() && A_STILL_TO_COME.is_match(clean) {
        return false;
    }
    true
}
