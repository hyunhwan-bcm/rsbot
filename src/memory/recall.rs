//! Port of `bot/memory/recall.py`: a line that points back at an earlier turn.

use crate::state::Memory;
use crate::understood::LooksBack;

/// `the_earlier_turn(text, memory=None)`.
pub fn the_earlier_turn(text: &str, memory: &Memory) -> Option<LooksBack> {
    let _ = (text, memory);
    todo!("recall::the_earlier_turn")
}
