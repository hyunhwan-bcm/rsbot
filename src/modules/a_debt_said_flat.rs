//! Port of `bot/modules/a_debt_said_flat.py`: a borrowing said flat - a rate, a balance - said back with what it means.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

macro_rules! rx {
    ($($name:ident),*) => {$(
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.a_debt_said_flat.", stringify!($name))));
    )*};
}
macro_rules! pool {
    ($($name:ident),*) => {$(
        static $name: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(concat!("bot.modules.a_debt_said_flat.", stringify!($name))));
    )*};
}
rx!(_A_RATE, _A_BORROWING, _A_JOINED, _A_BALANCE, _A_DEBT_DONE, _A_NOT_THIS_DEBT, _A_PAYMENT, _A_DEBT_NAMED, _A_GREETING_CARD, _A_HAVE, _A_THINKING, _A_ASK, A_BORROWING_WORDS);
pool!(A_DEBT_RATE, A_DEBT_BALANCE, A_DEBT_CHEAP, A_DEBT_STORE, A_DEBT_BNPL, A_DEBT_NO_FIGURE);

/// Python: bot/modules/a_debt_said_flat.py `observe(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// Nothing to keep: the turn is read off the line itself, every time.
pub fn observe(text: &str, memory: &mut Memory) {}

/// Python: bot/modules/a_debt_said_flat.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The rate or the balance said back with what it means, or None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let clean = fold(text);
    if clean.is_empty() || clean.contains('?') {
        return None;
    }
    if !_A_BORROWING.is_match(&clean) {
        return None;
    }
    if _A_DEBT_DONE.is_match(&clean) {
        return None;
    }
    if _A_ASK.match_(&clean).is_some() && !_A_RATE.is_match(&clean) {
        return None;
    }
    if _A_JOINED.match_(&clean).is_some() {
        return None;
    }
    if _A_NOT_THIS_DEBT.is_match(&clean) {
        return None;
    }
    if _A_GREETING_CARD.is_match(&clean) {
        return None;
    }
    if let Some(rate) = _A_RATE.search(&clean) {
        let percent = rate.str(1);
        // `float(percent)`: a ValueError answers nothing
        let value = py_float(percent)?;
        if value >= 8.0 {
            return Some(fmt_named(&pick(&A_DEBT_RATE, memory, &clean), &[("rate", percent)]));
        }
        return Some(fmt_named(&pick(&A_DEBT_CHEAP, memory, &clean), &[("rate", percent)]));
    }
    if _A_BALANCE.is_match(&clean) {
        if _A_PAYMENT.is_match(&clean) {
            return None;
        }
        return Some(pick(&A_DEBT_BALANCE, memory, &clean));
    }
    if _A_DEBT_NAMED.is_match(&clean) && _A_HAVE.is_match(&clean) && !_A_THINKING.is_match(&clean) {
        return Some(match A_BORROWING_WORDS.search(&clean) {
            Some(kind) if ["bnpl", "klarna", "buy"].iter().any(|opener| kind.as_str().starts_with(opener)) => pick(&A_DEBT_BNPL, memory, &clean),
            Some(_) => pick(&A_DEBT_STORE, memory, &clean),
            None => pick(&A_DEBT_NO_FIGURE, memory, &clean),
        });
    }
    None
}
