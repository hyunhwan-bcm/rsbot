//! Port of PyBot's `bot/knowledge/decorating.py`: paint going wrong, and the order of jobs.

use crate::data::{self, FnRef};
use crate::knowledge::d1_dup_g7 as dup;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

static _PAINT_WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.decorating._PAINT_WORD"));
static _DECORATED_SURFACE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.decorating._DECORATED_SURFACE"));
static _STEPS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.decorating._STEPS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
});
static _ORDER_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.decorating._ORDER_WORDS"));
static _NOT_WOODWORK: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.decorating._NOT_WOODWORK"));
static _ASKS_THE_MATERIAL: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.decorating._ASKS_THE_MATERIAL"));

/// Whether paint or a painted surface is what the line is about (not food being kept).
pub fn _paint_is_the_subject(understood: &Understood) -> bool {
    if dup::a_food_is_being_kept(understood) {
        return false;
    }
    let mut said = understood.words.join(" ").to_lowercase();
    if said.is_empty() {
        said = sh::clean_or_text(understood).to_lowercase();
    }
    _PAINT_WORD.is_match(&said) || _DECORATED_SURFACE.is_match(&said)
}

/// Whether the line asks the order of two decorating steps (fill, sand, prime, paint) on walls
/// or woodwork, not floors and not which grit to buy.
pub fn _the_order_of_two_steps(understood: &Understood) -> bool {
    let text = sh::text_or_clean(understood).to_lowercase();
    if sh::any_in(&text, &_NOT_WOODWORK) || sh::any_in(&text, &_ASKS_THE_MATERIAL) {
        return false;
    }
    if !sh::any_in(&text, &_ORDER_WORDS) {
        return false;
    }
    _STEPS.iter().filter(|(word, _)| text.contains(word.as_str())).map(|(_, step)| step.as_str()).collect::<HashSet<_>>().len() >= 2
}

/// Whether the line asks about putting up a shelf and its brackets (which first, how to hang).
pub fn _a_shelf_and_its_brackets(understood: &Understood) -> bool {
    let text = sh::text_or_clean(understood).to_lowercase();
    if !text.contains("shelf") && !text.contains("shelves") {
        return false;
    }
    if !text.contains("bracket") && !text.contains("brackets") {
        return false;
    }
    ["first", "before", "or", "then", "put up", "hang"].iter().any(|w| text.contains(w))
}

fn g_paint(_f: &FnRef, u: &Understood) -> bool {
    _paint_is_the_subject(u)
}
fn g_order(_f: &FnRef, u: &Understood) -> bool {
    _the_order_of_two_steps(u)
}
fn g_shelf(_f: &FnRef, u: &Understood) -> bool {
    _a_shelf_and_its_brackets(u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.decorating:_paint_is_the_subject:93", g_paint),
    ("bot.knowledge.decorating:_the_order_of_two_steps:148", g_order),
    ("bot.knowledge.decorating:_a_shelf_and_its_brackets:162", g_shelf),
];
pub const REPLIES: &[(&str, ReplyFn)] = &[];
