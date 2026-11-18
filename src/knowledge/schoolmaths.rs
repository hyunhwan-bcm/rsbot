//! Port of PyBot's `bot/knowledge/schoolmaths.py`: school maths terms (brackets in a sum).

use crate::data::{self, FnRef};
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::understood::Understood;
use std::sync::LazyLock;

static _BRACKET_MATHS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.schoolmaths._BRACKET_MATHS"));

/// Whether the brackets the line asks about are the ones in a sum (BODMAS), not punctuation.
pub fn _a_bracket_is_a_sum(understood: &Understood) -> bool {
    let mut said = "";
    for name in ["plain", "clean", "text"] {
        said = sh::field(understood, name);
        if !said.is_empty() {
            break;
        }
    }
    _BRACKET_MATHS.is_match(&said.to_lowercase())
}

fn g_a_bracket_is_a_sum(_f: &FnRef, u: &Understood) -> bool {
    _a_bracket_is_a_sum(u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.schoolmaths:_a_bracket_is_a_sum:34", g_a_bracket_is_a_sum)];
pub const REPLIES: &[(&str, ReplyFn)] = &[];
