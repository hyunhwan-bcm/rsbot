//! Port of `bot/modules/record_due.py`: the person's own record - a parcel, an order, a library loan - and who holds its state.
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
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.record_due.", stringify!($name))));
    )*};
}
macro_rules! pool {
    ($($name:ident),*) => {$(
        static $name: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(concat!("bot.modules.record_due.", stringify!($name))));
    )*};
}
rx!(_RECORD, _QUALIFIED, _ASK, _STATE_AFTER, _NOT_THE_STATE, _DELIVERY_STATE, _LATE, _BARE_STATE, _LOST, _TROUBLE, _AGAIN);
pool!(_LIBRARY_CARD, _LIBRARY_LOAN, _DELIVERY, LIBRARY_CARD_LINES, LIBRARY_LOAN_LINES, LIBRARY_LATE_LINES, LIBRARY_LOST_LINES, DELIVERY_LINES, DELIVERY_TROUBLE_LINES, DELIVERY_AGAIN_LINES);

/// Python: bot/modules/record_due.py `_the_record(clean)`
///
/// The person's own record the line names, or None.
pub fn _the_record(clean: &str) -> Option<String> {
    for m in _RECORD.finditer(clean) {
        // `_QUALIFIED.match(clean, match.end())`
        if _QUALIFIED.match_(&clean[m.end()..]).is_none() {
            return Some(text::squash(&m.name_str("thing").to_lowercase()));
        }
    }
    None
}

/// Python: bot/modules/record_due.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let clean = fold(text);
    if clean.is_empty() || !(_ASK.is_match(&clean) || _STATE_AFTER.is_match(&clean) || _BARE_STATE.is_match(&clean)) {
        return None;
    }
    let record = _the_record(&clean)?;
    if _NOT_THE_STATE.is_match(&clean) {
        return None;
    }
    let card = _LIBRARY_CARD.contains(&record);
    let loan = _LIBRARY_LOAN.contains(&record);
    if (card || loan) && _LOST.is_match(&clean) {
        return Some(_pick(&LIBRARY_LOST_LINES, memory));
    }
    if card {
        return Some(_pick(&LIBRARY_CARD_LINES, memory));
    }
    if loan {
        if _LATE.is_match(&clean) {
            return Some(_pick(&LIBRARY_LATE_LINES, memory));
        }
        return Some(_pick(&LIBRARY_LOAN_LINES, memory));
    }
    if _DELIVERY.contains(&record) {
        // A state ask about a delivery is the tracker's: "will it come before friday", "where is
        // my parcel", "has my order arrived". A request to fetch or post it is not this answer.
        if !_DELIVERY_STATE.is_match(&clean) {
            return None;
        }
        if _TROUBLE.is_match(&clean) {
            if _AGAIN.is_match(&clean) {
                return Some(_pick(&DELIVERY_AGAIN_LINES, memory));
            }
            return Some(_pick(&DELIVERY_TROUBLE_LINES, memory));
        }
        return Some(_pick(&DELIVERY_LINES, memory));
    }
    None
}

/// Python: bot/modules/record_due.py `_pick(lines, memory)`
///
/// Two wordings, so a second record ask does not repeat the first reply word for word.
pub fn _pick(lines: &[String], memory: &Memory) -> String {
    let last = last_bot(memory);
    lines.iter().find(|line| **line != last).unwrap_or(&lines[0]).clone()
}
