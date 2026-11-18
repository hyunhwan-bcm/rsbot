//! Port of `bot/compose/hedge.py`: the composer's reply to a line, asked before a module answers
//! over it. A module just above the composer floor asks the composer first: copy the memory, put
//! the random state back afterwards, and read what `fallback.reply_for` says for the line alone.

use crate::compose::fallback_chain::{self, caught};
use crate::data::{self, Value};
use crate::py::random;
use crate::py::re::Re;
use crate::state::Memory;
use std::sync::LazyLock;

static NOTHING_SAID: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.hedge.NOTHING_SAID"));
static OUT_OF_THEIR_HANDS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.hedge.OUT_OF_THEIR_HANDS"));

/// Python: bot/compose/hedge.py:46 `what_the_composer_would_say(text, memory)` (never called on the trace corpus)
///
/// The composer's own reply to this line, or "" when it has none (always a str).
pub fn what_the_composer_would_say(text: &str, memory: &Memory) -> Value {
    let mut probe = memory.clone();
    let state = random::getstate();
    // `verdict=False` is the words on their own rather than this turn, which is what the floor
    // answers. The port's `reply_for` gives "" for Python's None, so `or ""` is already applied.
    let said = caught(|| fallback_chain::reply_for(text, &mut probe, false)).unwrap_or_default();
    random::setstate(state);
    Value::from(said)
}

/// Python: bot/compose/hedge.py:61 `only_a_shrug(text, memory, misreads=None)` (never called on the trace corpus)
///
/// True when the composer has no answer of its own for this line. `misreads` is a pattern
/// (`Value::Re`) for the answers that are worse than nothing on this kind of line, or None.
pub fn only_a_shrug(text: &str, memory: &Memory, misreads: &Value) -> bool {
    let said = what_the_composer_would_say(text, memory);
    let said = said.str();
    if said.is_empty() {
        return true;
    }
    if NOTHING_SAID.is_match(said) || OUT_OF_THEIR_HANDS.is_match(said) {
        return true;
    }
    if !misreads.is_none() && misreads.as_re().expect("misreads is a compiled pattern").is_match(said) {
        return true;
    }
    false
}
