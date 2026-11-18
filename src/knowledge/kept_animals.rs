//! Port of PyBot's `bot/knowledge/kept_animals.py`: looking after a kept animal (not cooking it).

use crate::data::{self, FnRef};
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::understood::Understood;
use std::sync::LazyLock;

static _AVOID: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.kept_animals._AVOID"));
static _ASKS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.kept_animals._ASKS"));

/// Whether the line asks how to keep or look after an animal, and is not a recipe line.
pub fn keeping_an_animal(understood: &Understood) -> bool {
    let joined;
    let mut said = sh::text_or_clean(understood);
    if said.is_empty() {
        joined = sh::words_line(understood);
        said = &joined;
    }
    let text = said.to_lowercase();
    if text.is_empty() {
        return false;
    }
    if _AVOID.is_match(&text) {
        return false;
    }
    _ASKS.is_match(&text)
}

fn g_keeping_an_animal(_f: &FnRef, u: &Understood) -> bool {
    keeping_an_animal(u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.kept_animals:keeping_an_animal:48", g_keeping_an_animal)];
pub const REPLIES: &[(&str, ReplyFn)] = &[];
