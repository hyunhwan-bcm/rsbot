//! Port of `bot/understanding/car_glass.py`: a car window in the line is a damage card only when
//! nobody did it and a fault is named.

#![allow(non_upper_case_globals)]

use crate::data::FnRef;
use crate::knowledge::registry::GuardFn;
use crate::py::re::Re;
use crate::understanding::t;
use crate::understood::Understood;
use std::sync::LazyLock;

const M: &str = "bot.understanding.car_glass.";

static A_WINDOW_ON_A_CAR: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_WINDOW_ON_A_CAR"));
static _A_FAULT: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "_A_FAULT"));  // t, 105 items
static _SOMEBODY_DID_IT: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "_SOMEBODY_DID_IT"));  // t, 27 items

/// `_names_one_of(said, parts)`.
fn _names_one_of(said: &str, parts: &[String]) -> bool {
    let padded = format!(" {said}");
    parts.iter().any(|p| padded.contains(&format!(" {p}")))
}

/// `a_window_on_a_car_is_not_a_damage(understood)`: guard.
pub fn a_window_on_a_car_is_not_a_damage(_f: &FnRef, understood: &Understood) -> bool {
    let raw = understood.extra.get("raw").and_then(|v| v.as_str()).unwrap_or("");
    let words = understood.words.join(" ");
    let said: Vec<&str> = [understood.text.as_str(), understood.clean.as_str(), raw, words.as_str()].into_iter().filter(|p| !p.is_empty()).collect();
    let said = said.join(" ").to_lowercase();
    if crate::py::text::strip(&said).is_empty() || !A_WINDOW_ON_A_CAR.is_match(&said) {
        return true;
    }
    if _names_one_of(&said, &_SOMEBODY_DID_IT) {
        return true;
    }
    !_names_one_of(&said, &_A_FAULT)
}

/// The entry guards defined in car_glass.py.
pub const GUARDS: &[(&str, GuardFn)] = &[("bot.understanding.car_glass:a_window_on_a_car_is_not_a_damage:61", a_window_on_a_car_is_not_a_damage)];

#[allow(dead_code)]
fn _unused(_: &Re) {}
