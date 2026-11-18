//! Port of `bot/knowledge/coats.py`: whether to take a coat out today. The entry is data; this is
//! its guard.

use crate::data;
use crate::data::FnRef;
use crate::knowledge::d2_shared::py_repr_list;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::understood::Understood;
use std::sync::LazyLock;

static _NOT_ABOUT_TODAY: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.coats._NOT_ABOUT_TODAY"));
static _OUT_WHEN: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.coats._OUT_WHEN"));
static _LOST_OR_TAKEN: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.coats._LOST_OR_TAKEN"));
static _A_CLOCK: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.coats._A_CLOCK"));

/// True when the line is a yes or no about taking a garment out today.
pub fn _a_garment_to_take(understood: &Understood) -> bool {
    // " ".join(str(understood.get(key) or "") for key in ("words", "clean", "text")): the words
    // list goes in as its Python repr.
    let words = if understood.words.is_empty() { String::new() } else { py_repr_list(&understood.words) };
    let mut said = format!("{} {} {}", words, understood.clean, understood.text).to_lowercase();
    for filler in ['[', ']', ',', '\''] {
        said = said.replace(filler, " ");
    }
    let lead = ["how do i", "how do you", "how can i", "how to"];
    if lead.iter().any(|w| said.contains(w)) || _NOT_ABOUT_TODAY.iter().any(|w| said.contains(w.as_str())) {
        return false;
    }
    if _LOST_OR_TAKEN.is_match(&said) {
        return false;
    }
    _OUT_WHEN.is_match(&said) || _A_CLOCK.is_match(&said)
}

pub const GUARDS: &[(&str, GuardFn)] =
    &[("bot.knowledge.coats:_a_garment_to_take:106", |_: &FnRef, u: &Understood| _a_garment_to_take(u))];

pub const REPLIES: &[(&str, ReplyFn)] = &[];
