//! Port of PyBot's `bot/knowledge/caring.py`: unpaid carers, their money and their load.

use crate::data::{self, FnRef};
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::understood::Understood;
use std::sync::LazyLock;

static _AN_ANIMAL_CARED_FOR: LazyLock<&'static Re> =
    LazyLock::new(|| data::re("bot.knowledge.caring._AN_ANIMAL_CARED_FOR"));
static _AN_ANIMAL_WEARS_ME_OUT: LazyLock<&'static Re> =
    LazyLock::new(|| data::re("bot.knowledge.caring._AN_ANIMAL_WEARS_ME_OUT"));
static NOT_PLAIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' ]+", 0));
static A_CHILD: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(?:kid|kids|child|children|childminding|toddler|toddlers|babies|baby)\b", 0));
static THE_BENEFIT: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\bcarers?'?\s*(?:allowance|credit)\b|\bcaring\b|\bcarer\b", 0));

/// False when the caring in the line is for an animal: the pet files answer that.
pub fn _an_animal_is_the_one_cared_for(reading: &Understood) -> bool {
    let said = ["clean", "text", "plain"].map(|k| sh::field(reading, k)).join(" ");
    let said = format!(" {} ", NOT_PLAIN.sub(" ", &said.to_lowercase()));
    !(_AN_ANIMAL_CARED_FOR.is_match(&said) || _AN_ANIMAL_WEARS_ME_OUT.is_match(&said))
}

/// Whether the one being looked after is a grown-up (a child named is childcare), unless the
/// line names the carer's benefit or caring itself.
pub fn _the_care_is_for_a_grown_up(understood: &Understood) -> bool {
    let line = sh::clean_or_text(understood).to_lowercase();
    if !A_CHILD.is_match(&line) {
        return true;
    }
    THE_BENEFIT.is_match(&line)
}

fn g_animal(_f: &FnRef, u: &Understood) -> bool {
    _an_animal_is_the_one_cared_for(u)
}
fn g_grown_up(_f: &FnRef, u: &Understood) -> bool {
    _the_care_is_for_a_grown_up(u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.caring:_an_animal_is_the_one_cared_for:76", g_animal),
    ("bot.knowledge.caring:_the_care_is_for_a_grown_up:221", g_grown_up),
];
pub const REPLIES: &[(&str, ReplyFn)] = &[];
