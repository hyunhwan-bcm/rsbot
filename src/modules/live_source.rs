//! Port of `bot/modules/live_source.py`: an ask whose answer is live somewhere else - a bank holiday, a term date, a draw, a queue - and where it is kept.
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
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.live_source.", stringify!($name))));
    )*};
}
macro_rules! words {
    ($($name:ident),*) => {$(
        static $name: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(concat!("bot.modules.live_source.", stringify!($name))));
    )*};
}
rx!(CALENDAR_OPEN, CALENDAR_PLAN, DRAW_ASK, LEAD_IN, ASKS);
words!(BANK_HOLIDAY, TERM, CALENDAR_NOT, CALENDAR_SCHOOL, NATIONAL_DRAW, LOCAL_DRAW, MONTHLY_DRAW, DRAW, WAIT, AIRPORT, A_E);
words!(BANK_LINES, TERM_LINES, AIRPORT_LINES, A_E_LINES, NATIONAL_LINES, LOCAL_LINES, TICKET_LINES, MONTHLY_LINES);

/// Python: bot/modules/live_source.py `_bounded(phrase)`
///
/// A phrase matched as whole words, so "raining" is not "training". (Python keeps the compiled
/// pattern in `_BOUNDED`; the pattern is the same one each time.)
pub fn _bounded(phrase: &str) -> Re {
    Re::new(&format!("(?<![a-z]){}(?![a-z])", pyre::escape(phrase)), 0)
}

/// Python: bot/modules/live_source.py `_has(clean, phrases)`
pub fn _has(clean: &str, phrases: &[String]) -> bool {
    phrases.iter().any(|phrase| _bounded(phrase).is_match(clean))
}

/// Python: bot/modules/live_source.py `_pick(clean, lines)`
///
/// One of the two ways of saying it, the same one every time for the same line.
pub fn _pick(clean: &str, lines: &[String]) -> String {
    let sum: usize = clean.as_bytes().iter().map(|byte| *byte as usize).sum();
    lines[sum % lines.len()].clone()
}

/// Python: bot/modules/live_source.py `_clean(text)`
pub fn _clean(text: &str) -> String {
    fold(text)
}

/// Python: bot/modules/live_source.py `_a_date_ask(clean)`
///
/// The pool for a date ask, or () when the line is not one.
pub fn _a_date_ask(clean: &str) -> Vec<String> {
    if CALENDAR_OPEN.match_(&_without_lead_in(clean)).is_none() {
        return Vec::new();
    }
    if CALENDAR_PLAN.is_match(clean) {
        return Vec::new(); // a plan for the week, not the week's date
    }
    if _has(clean, &CALENDAR_NOT) && !_has(clean, &CALENDAR_SCHOOL) {
        return Vec::new(); // the post, the tip, a shift: a service word, not the calendar
    }
    if _has(clean, &TERM) {
        return TERM_LINES.clone();
    }
    if _has(clean, &BANK_HOLIDAY) {
        return BANK_LINES.clone();
    }
    Vec::new()
}

/// Python: bot/modules/live_source.py `_without_lead_in(clean)`
///
/// The line with a spoken lead-in taken off the front, for reading the ask.
pub fn _without_lead_in(clean: &str) -> String {
    let mut said = clean.to_string();
    loop {
        let shorter = LEAD_IN.subn("", &said, 1);
        if shorter == said || text::strip(&shorter).is_empty() {
            return said;
        }
        said = shorter;
    }
}

/// Python: bot/modules/live_source.py `_a_draw_ask(clean)`
///
/// The pool for a ticket check, or () when the line is not one.
pub fn _a_draw_ask(clean: &str) -> Vec<String> {
    if !_has(clean, &DRAW) || !DRAW_ASK.is_match(clean) {
        return Vec::new();
    }
    if _has(clean, &MONTHLY_DRAW) {
        return MONTHLY_LINES.clone();
    }
    if _has(clean, &LOCAL_DRAW) && !_has(clean, &NATIONAL_DRAW) {
        return LOCAL_LINES.clone();
    }
    if _has(clean, &NATIONAL_DRAW) {
        return NATIONAL_LINES.clone();
    }
    TICKET_LINES.clone()
}

/// Python: bot/modules/live_source.py `_a_queue(clean)`
pub fn _a_queue(clean: &str) -> Vec<String> {
    if !_has(clean, &WAIT) {
        return Vec::new();
    }
    if _has(clean, &A_E) {
        return A_E_LINES.clone();
    }
    if _has(clean, &AIRPORT) {
        return AIRPORT_LINES.clone();
    }
    Vec::new()
}

/// Python: bot/modules/live_source.py `respond(text, memory=None)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let clean = _clean(text);
    let said = _without_lead_in(&clean);
    if clean.is_empty() || !(ASKS.match_(&said).is_some() || text.contains('?')) {
        return None;
    }
    let lines = _a_queue(&clean);
    if !lines.is_empty() {
        return Some(_pick(&clean, &lines));
    }
    let draw = _a_draw_ask(&clean);
    if !draw.is_empty() {
        return Some(_pick(&clean, &draw));
    }
    let lines = _a_date_ask(&clean);
    if !lines.is_empty() {
        return Some(_pick(&clean, &lines));
    }
    None
}
