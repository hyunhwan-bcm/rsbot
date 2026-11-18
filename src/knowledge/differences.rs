//! Port of `bot/knowledge/differences.py`: "what is the difference between X and Y". The pairs
//! and their entries are data (built at import time by `_pair` / `_slug`, exported in the
//! bundle); the runtime logic is the cream guard.

use crate::data::FnRef;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::understood::Understood;

/// The line says cream somewhere: "single or double cream" is the same question as the pair.
pub fn _with_cream(understood: &Understood) -> bool {
    understood.clean.to_lowercase().contains("cream")
}

pub const GUARDS: &[(&str, GuardFn)] =
    &[("bot.knowledge.differences:_with_cream:274", |_: &FnRef, u: &Understood| _with_cream(u))];

pub const REPLIES: &[(&str, ReplyFn)] = &[];
