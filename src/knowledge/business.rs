//! Port of PyBot's `bot/knowledge/business.py`: starting and running a small business.

use crate::data::{self, FnRef};
use crate::knowledge::d1_dup_g7 as dup;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::understood::Understood;
use std::sync::LazyLock;

static TRADES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.business.TRADES"));
static _GOING_ALONE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.business._GOING_ALONE"));
static _A_BUSINESS_NAMED: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut v = _GOING_ALONE.clone();
    v.extend(["business", "trade", "company", "firm", "shop", "sole trader"].map(String::from));
    v
});

/// Whether a trade named in the line is the person's own business (going self-employed), not a
/// tradesman they are hiring.
pub fn _a_trade_is_their_own(understood: &Understood) -> bool {
    let text = sh::padded(&sh::clean_or_text(understood).to_lowercase());
    if !sh::any_in(&text, &TRADES) {
        return true;
    }
    sh::any_in(&text, &_GOING_ALONE)
}

/// Whether "starting out" in the line is not starting out at some other thing ("start out
/// running"), unless the business is named as well.
pub fn _the_start_out_names_no_thing(understood: &Understood) -> bool {
    let text = sh::padded(&sh::clean_or_text(understood).to_lowercase());
    if !dup::_starting_out_names_a_thing(&text) {
        return true;
    }
    sh::any_in(&text, &_A_BUSINESS_NAMED)
}

/// The row answers the person's own business, started as a business.
pub fn _the_person_is_the_row(understood: &Understood) -> bool {
    _a_trade_is_their_own(understood) && _the_start_out_names_no_thing(understood)
}

fn g_the_person_is_the_row(_f: &FnRef, u: &Understood) -> bool {
    _the_person_is_the_row(u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.business:_the_person_is_the_row:121", g_the_person_is_the_row)];
pub const REPLIES: &[(&str, ReplyFn)] = &[];
