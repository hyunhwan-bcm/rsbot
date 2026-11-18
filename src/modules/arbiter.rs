//! Port of `bot/modules/arbiter.py`: picks one reply from the candidates.
//!
//! Stub until the arbiter workstream lands: the first candidate (the highest PRIORITY), which
//! is what `bot/runtime.py` does without a working arbiter.

use crate::runtime::Candidate;
use crate::state::Memory;

pub fn choose(_text: &str, _memory: &Memory, _candidates: &[Candidate]) -> usize {
    0
}
