//! Port of `bot/compose/moving.py`: a move said ahead without saying it is a glad one is asked
//! about ("is it a move you wanted?") rather than congratulated.

use crate::compose::fallback_chain::{self, caught};
use crate::compose::goodnews;
use crate::data;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.moving.", stringify!($name))));)*
    };
}
regexes!(A_MOVE, A_MOVE_NOUN, GLAD, NOT_GLAD, ADVICE, ITS_OWN_NEWS);
static MANY: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.moving.MANY"));

/// Python: bot/compose/moving.py:61 `reads_as_a_move(text)`
///
/// True when the line is a move said without saying it is a glad one (nor plainly not glad).
pub fn reads_as_a_move(text: &str) -> bool {
    let clean = text::squash(&text::lower(text));
    if clean.is_empty() || clean.contains('?') {
        return false;
    }
    if !(A_MOVE.is_match(&clean) || A_MOVE_NOUN.is_match(&clean)) {
        return false;
    }
    if GLAD.is_match(&clean) || ITS_OWN_NEWS.is_match(&clean) {
        return false;
    }
    if NOT_GLAD.is_match(&clean) || ADVICE.is_match(&clean) {
        return false;
    }
    true
}

/// Python: bot/compose/moving.py:80 `reads_as_a_move_cheered(text)`
///
/// The move the composer would have congratulated: a move, and one still ahead of now.
pub fn reads_as_a_move_cheered(text: &str) -> bool {
    let clean = text::squash(&text::lower(text));
    if !reads_as_a_move(&clean) {
        return false;
    }
    caught(|| goodnews::ahead_of_now(&clean)).unwrap_or(false)
}

/// Python: bot/compose/moving.py:96 `reply_for(text, memory=None)`
///
/// One of the pool's lines for a move, or None when the line is something else.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    if !reads_as_a_move_cheered(text) {
        return None;
    }
    let clean = text::squash(&text::lower(text));
    caught(|| fallback_chain::_choose(&MANY, memory, &clean, None)).flatten()
}
