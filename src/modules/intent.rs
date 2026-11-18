//! Port of `bot/modules/intent.py`: writes the shape of answer the line wants.

use crate::state::Memory;

pub const PRIORITY: f64 = 9500.0;

/// `observe(text, memory)`: `memory["intent_wants"]` and `memory["understood"]["wants"]`.
pub fn observe(_text: &str, memory: &mut Memory) {
    let Some(read) = memory.understood.as_ref() else { return };
    let before = memory.intent_wants.clone();
    let want = crate::compose::fit::wants(read, before.as_deref());
    let want = crate::compose::clause::shape(read, want);
    let want = crate::compose::what_it_asks::the_shape_it_asks(read, want);
    memory.intent_wants = want.clone();
    let read = memory.understood.as_mut().unwrap();
    read.wants = want.filter(|w| !w.is_empty());
}
