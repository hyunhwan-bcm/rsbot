//! Port of PyBot's `bot/knowledge/odours.py`: getting a smell out of clothes, rooms and kit.

use crate::data::{self, FnRef};
use crate::knowledge::d1_dup_g7 as dup;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::understood::Understood;
use std::sync::LazyLock;

static GARMENT: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.odours.GARMENT"));
static SMELL_SAID: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.odours.SMELL_SAID"));
static _ODOUR_SOURCE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.odours._ODOUR_SOURCE"));
static _GARMENT_THEN_FOOD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.odours._GARMENT_THEN_FOOD"));

/// False when the line names a source of a smell (curry, perfume) without saying it smells: a
/// "curry stain" is the stain card's.
pub fn _a_source_is_not_a_smell(understood: &Understood) -> bool {
    let text = understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" ");
    if !_ODOUR_SOURCE.iter().any(|s| text.contains(s.as_str())) {
        return true;
    }
    SMELL_SAID.iter().any(|s| text.contains(s.as_str()))
}

/// False when the garment in the line is really food being kept (a coat of breadcrumbs, a bag
/// of cheese in a jacket pocket), so the smell card stands down.
pub fn _a_garment_is_not_a_food(understood: &Understood) -> bool {
    let text = understood.words.join(" ").to_lowercase();
    if _GARMENT_THEN_FOOD.is_match(&text) {
        return false;
    }
    if dup::a_food_is_being_kept(understood) && dup::only_a_container(understood, &GARMENT) {
        return false;
    }
    true
}

/// The rows' guard: a garment that is not food, and a smell that is said.
pub fn lambda_219(understood: &Understood) -> bool {
    _a_garment_is_not_a_food(understood) && _a_source_is_not_a_smell(understood)
}

fn g_lambda(_f: &FnRef, u: &Understood) -> bool {
    lambda_219(u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.odours:<lambda>:219", g_lambda),
    ("bot.knowledge.odours:<lambda>:264", g_lambda),
];
pub const REPLIES: &[(&str, ReplyFn)] = &[];
