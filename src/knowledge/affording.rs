//! Port of PyBot's `bot/knowledge/affording.py`: an invitation the person cannot afford.

use crate::data::{self, FnRef};
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::understood::Understood;
use std::sync::LazyLock;

static _A_PERSON_OR_AN_INVITATION: LazyLock<&'static Re> =
    LazyLock::new(|| data::re("bot.knowledge.affording._A_PERSON_OR_AN_INVITATION"));
static _PLANNING: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.affording._PLANNING"));

/// Whether the line is about an invitation (a person, a trip) the person cannot afford, and not
/// about planning or budgeting one.
pub fn _an_invitation_is_what_they_cannot_afford(understood: &Understood) -> bool {
    let words = sh::words_line(understood);
    let text = sh::or(sh::clean_or_text(understood), &words);
    if !_A_PERSON_OR_AN_INVITATION.is_match(text) {
        return false;
    }
    !_PLANNING.is_match(text)
}

fn g_invitation(_f: &FnRef, u: &Understood) -> bool {
    _an_invitation_is_what_they_cannot_afford(u)
}

pub const GUARDS: &[(&str, GuardFn)] =
    &[("bot.knowledge.affording:_an_invitation_is_what_they_cannot_afford:63", g_invitation)];
pub const REPLIES: &[(&str, ReplyFn)] = &[];
