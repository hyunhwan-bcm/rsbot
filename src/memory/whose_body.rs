//! Port of `bot/memory/whose_body.py`: "he cant breathe", "her leg is swollen" after the chat
//! named somebody the person looks after - a body that is somebody else's.

use super::mem8_py::rx;
use crate::state::Memory;

const M: &str = "bot.memory.whose_body.";

rx!(NAMED_SOMEBODY, PRONOUN_BODY, OWN_BODY, PET);

/// Python: bot/memory/whose_body.py:47 `somebody_was_named(memory)`
///
/// True when the chat has named somebody the person looks after, in the last few turns.
pub fn somebody_was_named(memory: &Memory) -> bool {
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(4)..].iter().rev() {
        if PET.is_match(&turn.user) {
            return false;
        }
        if NAMED_SOMEBODY.is_match(&turn.user) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/whose_body.py:66 `a_body_of_a_pronoun(clean)`
///
/// True when the line hangs a body or a hurt on he/she/they rather than on "my".
pub fn a_body_of_a_pronoun(clean: &str) -> bool {
    if OWN_BODY.is_match(clean) {
        return false;
    }
    PRONOUN_BODY.is_match(clean)
}

/// Python: bot/memory/whose_body.py:74 `somebody_elses_body(clean, memory)`
///
/// True when this line is somebody else's body, named earlier in the chat.
pub fn somebody_elses_body(clean: &str, memory: &Memory) -> bool {
    a_body_of_a_pronoun(clean) && somebody_was_named(memory)
}
