//! Port of `bot/memory/topics.py`: the parts the understand observer calls.

use crate::state::Memory;
use crate::understood::Understood;

/// `the_thing_it_is(understood, memory)`: the reading with a pronoun filled in from the store,
/// or None.
pub fn the_thing_it_is(understood: &Understood, memory: &Memory) -> Option<Understood> {
    let _ = (understood, memory);
    todo!("memory::topics::the_thing_it_is")
}

/// `the_advice_ask_with_its_doing(understood, memory)`.
pub fn the_advice_ask_with_its_doing(understood: &Understood, memory: &Memory) -> Option<Understood> {
    let _ = (understood, memory);
    todo!("memory::topics::the_advice_ask_with_its_doing")
}
