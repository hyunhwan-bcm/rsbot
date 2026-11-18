//! Port of `bot/knowledge/containers_frost.py`: bringing pots in before a frost. The entry is
//! data; this is its guard.

use crate::data;
use crate::data::FnRef;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::understood::Understood;
use std::sync::LazyLock;

static _HAS_ALREADY_FROZEN: LazyLock<&'static Re> =
    LazyLock::new(|| data::re("bot.knowledge.containers_frost._HAS_ALREADY_FROZEN"));

/// True while the frost is still ahead of the line, which is when moving a pot is the answer.
pub fn _the_night_is_still_coming(understood: &Understood) -> bool {
    let mut said = if understood.text.is_empty() { understood.clean.clone() } else { understood.text.clone() };
    if said.is_empty() {
        said = understood.words.join(" ");
    }
    !_HAS_ALREADY_FROZEN.is_match(&said)
}

pub const GUARDS: &[(&str, GuardFn)] = &[(
    "bot.knowledge.containers_frost:_the_night_is_still_coming:53",
    |_: &FnRef, u: &Understood| _the_night_is_still_coming(u),
)];

pub const REPLIES: &[(&str, ReplyFn)] = &[];
