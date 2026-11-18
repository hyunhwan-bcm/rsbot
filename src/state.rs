//! The conversation memory a turn reads and writes: Python's `memory` dict. (The Python
//! package `bot/memory` is `crate::memory`; this struct is `crate::state::Memory`.)
//!
//! This port answers one line at a time (as SmolTalkBench asks), so `turns` is empty when a
//! turn starts and the keys that only matter on a later turn are not kept. What a module writes
//! for its own use within the turn (an entry it answered from, a case it recorded) goes in
//! `extra`, under the Python key.

use crate::data::Value;
use crate::understood::Understood;
use indexmap::IndexMap;

#[derive(Clone, Debug, Default)]
pub struct Turn {
    pub user: String,
    pub bot: String,
    pub module: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Memory {
    /// The understanding engine's reading of this line (`memory["understood"]`).
    pub understood: Option<Understood>,
    /// The shape of answer the line asks for (`memory["intent_wants"]`).
    pub intent_wants: Option<String>,
    /// The line as typed, kept by the answers module (`memory["asked_now"]`).
    pub asked_now: Option<String>,
    /// Earlier turns (`memory["turns"]`); empty at the start of a single-turn reply.
    pub turns: Vec<Turn>,
    /// Anything else a module keeps for itself, by its Python key.
    pub extra: IndexMap<String, Value>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory::default()
    }
    /// `len(memory.get("turns") or [])`, the count most seeded pickers use.
    pub fn turn_count(&self) -> usize {
        self.turns.len()
    }
}
