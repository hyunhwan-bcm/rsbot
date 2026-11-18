//! Port of `bot/modules/understand.py`: runs the understanding engine before any module replies.

use crate::state::Memory;

pub const PRIORITY: f64 = 10000.0;

/// `observe(text, memory)`: writes `memory["understood"]` (with `looks_back`).
pub fn observe(text: &str, memory: &mut Memory) {
    let read = crate::understanding::understand(text, Some(memory));
    let mut filled = crate::memory::topics::the_thing_it_is(&read, memory);
    if filled.is_none() {
        filled = crate::memory::topics::the_advice_ask_with_its_doing(&read, memory);
    }
    let mut understood = filled.unwrap_or(read);
    understood.looks_back = crate::memory::recall::the_earlier_turn(text, memory);
    memory.understood = Some(understood);
}
