//! Port of `bot/understanding/cars.py`: a broken car the conversation is about, and the two entry
//! guards written on it.

#![allow(non_upper_case_globals)]

use crate::data::FnRef;
use crate::knowledge::registry::GuardFn;
use crate::py::re::Re;
use crate::py::text;
use crate::understanding::t;
use crate::understood::Understood;
use std::sync::LazyLock;

const M: &str = "bot.understanding.cars.";

static CAR: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CAR"));
static CLAUSE_BREAK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CLAUSE_BREAK"));
static FAULT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FAULT"));
static LEARNING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LEARNING"));
const LOOKBACK: i64 = 3;
static NOT_A_FAULT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_A_FAULT"));
static NOT_THE_MACHINE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_THE_MACHINE"));

/// `_a_fault_that_is_not_undone(clause)`: a fault said ("won't start") and not taken back.
fn _a_fault_that_is_not_undone(clause: &str) -> bool {
    for m in FAULT.finditer(clause) {
        let low = m.as_str().to_lowercase();
        if ["not ", "wont", "won't", "wouldnt", "wouldn't"].iter().any(|p| low.starts_with(p)) {
            return true;
        }
        let s = text::char_index(clause, m.start()) as isize;
        let e = text::char_index(clause, m.end()) as isize;
        let before = text::slice(clause, (s - 24).max(0), Some(s));
        let after = text::slice(clause, e, Some(e + 24));
        if NOT_A_FAULT.is_match(&before) || NOT_A_FAULT.is_match(&after) {
            continue;
        }
        return true;
    }
    false
}

/// `_the_car_in(clause)`: the clause names a car (not a toy or a machine).
fn _the_car_in(clause: &str) -> bool {
    for m in CAR.finditer(clause) {
        let s = text::char_index(clause, m.start()) as isize;
        let before = text::slice(clause, (s - 20).max(0), Some(s));
        if NOT_THE_MACHINE.is_match(&before) {
            continue;
        }
        return true;
    }
    false
}

/// `_says_a_broken_car(text)`.
fn _says_a_broken_car(text_: &str) -> bool {
    let low = text_.to_lowercase();
    CLAUSE_BREAK.split(&low).iter().any(|c| _the_car_in(c) && _a_fault_that_is_not_undone(c))
}

/// `a_broken_car_is_the_subject(text, earlier=())`: the line (or one of the last few) says the
/// car is broken.
pub fn a_broken_car_is_the_subject(text_: &str, earlier: &[String]) -> bool {
    let low = text_.to_lowercase();
    if LEARNING.is_match(&low) {
        return false;
    }
    if _says_a_broken_car(&low) {
        return true;
    }
    earlier[earlier.len().saturating_sub(LOOKBACK as usize)..].iter().any(|l| _says_a_broken_car(l))
}

/// `LOOKBACK`: how many earlier lines a broken car is carried from.
pub fn lookback() -> usize {
    LOOKBACK as usize
}

/// `no_broken_car(understood)`: guard - no broken car in the reading.
pub fn no_broken_car(_f: &FnRef, understood: &Understood) -> bool {
    !understood.car_fault
}

/// `no_broken_car_or_none(understood)`: guard - a broken car in the reading.
pub fn no_broken_car_or_none(_f: &FnRef, understood: &Understood) -> bool {
    understood.car_fault
}

/// The entry guards defined in cars.py.
pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.understanding.cars:no_broken_car:131", no_broken_car),
    ("bot.understanding.cars:no_broken_car_or_none:142", no_broken_car_or_none),
];

#[allow(dead_code)]
fn _unused(_: &Re) {}
