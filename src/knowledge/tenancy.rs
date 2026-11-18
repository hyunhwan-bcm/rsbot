//! Port of `bot/knowledge/tenancy.py`: landlord and tenancy rows, and the fences this module
//! puts on rows of `money` and `work` (a rent raise is not a pay raise, a landlord is not a manager).
//! `_fence` runs at import time (its result is in the bundle); the guard it installs is ported.

use crate::data::FnRef;
use crate::knowledge::d2dup_g10::globals;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::understood::Understood;

globals!("bot.knowledge.tenancy", re { _NOT_A_RENT_RAISE, _A_TENANCY });

/// The line, lower-cased: its clean form, or the text as typed.
pub fn _said(understood: &Understood) -> String {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    line.to_lowercase()
}

/// `_said` for a line given as a plain string.
pub fn _said_text(text: &str) -> String {
    text.to_lowercase()
}

/// True when the raise is of something that is not the rent.
pub fn _a_raise_with_an_object(understood: &Understood) -> bool {
    _NOT_A_RENT_RAISE.is_match(&_said(understood))
}

/// True when the landlord in the line makes it a tenancy question, not a manager one.
pub fn _a_tenancy_and_not_a_manager(understood: &Understood) -> bool {
    _A_TENANCY.is_match(&_said(understood))
}

/// The guard `_fence` gives a fenced row: `lambda understood, when=when: not when(understood)`.
fn fence_guard(f: &FnRef, understood: &Understood) -> bool {
    let when = f.defaults.first().and_then(|v| v.as_fn()).map(|w| w.name()).unwrap_or("");
    let said = match when {
        "bot.knowledge.tenancy:_a_raise_with_an_object" => _a_raise_with_an_object(understood),
        "bot.knowledge.tenancy:_a_tenancy_and_not_a_manager" => _a_tenancy_and_not_a_manager(understood),
        other => panic!("tenancy fence with an unknown test {other}"),
    };
    !said
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.tenancy:_fence.<locals>.<lambda>:66", fence_guard)];

pub const REPLIES: &[(&str, ReplyFn)] = &[];
