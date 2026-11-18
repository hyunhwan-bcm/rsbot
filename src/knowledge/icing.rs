//! Port of `bot/knowledge/icing.py`: keeping icing and frosting. The entries are data; this is
//! the one guard they use.

use crate::data;
use crate::data::FnRef;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::understood::Understood;
use std::sync::LazyLock;

static _FROSTING_UP: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.icing._FROSTING_UP"));

/// False when the line is a freezer frosting up rather than an icing being kept.
pub fn _not_a_freezer_frosting_up(understood: &Understood) -> bool {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    !_FROSTING_UP.is_match(line)
}

pub const GUARDS: &[(&str, GuardFn)] = &[(
    "bot.knowledge.icing:_not_a_freezer_frosting_up:52",
    |_: &FnRef, u: &Understood| _not_a_freezer_frosting_up(u),
)];

pub const REPLIES: &[(&str, ReplyFn)] = &[];
