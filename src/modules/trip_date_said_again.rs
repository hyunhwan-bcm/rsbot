//! Port of `bot/modules/trip_date_said_again.py`: a trip's date put right with its reason in the same breath moves the trip the store holds.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::memory::facts as extract;

static A_DATE_SAID_NOW: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.trip_date_said_again.A_DATE_SAID_NOW"));
static A_MARKER_SAID: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.trip_date_said_again.A_MARKER_SAID"));

/// Python: bot/modules/trip_date_said_again.py `_a_date_said_now(text)`
///
/// The date a line moves the trip to when it says the reason in the same breath, or "". A
/// question moves nothing, and the value has to read as a date once it is read.
pub fn _a_date_said_now(text: &str) -> String {
    if text.contains('?') {
        return String::new();
    }
    let Some(found) = A_DATE_SAID_NOW.search(text) else { return String::new() };
    // `line[:found.start(1)]`: -1 for a group that took no part, which is the line less its end.
    let before = match found.start_of(1) {
        Some(at) => text[..at].to_string(),
        None => sl(text, 0, Some(-1)),
    };
    if !A_MARKER_SAID.is_match(&before) {
        return String::new();
    }
    for group in found.groups().into_iter().flatten() {
        if !group.is_empty() && ret_true(extract::_a_date(group)) {
            return ret_str(extract::_tidy(group));
        }
    }
    String::new()
}

/// Python: bot/modules/trip_date_said_again.py `_the_first_clause(text)`
///
/// The line up to its first comma, when that leaves a clause to read.
pub fn _the_first_clause(text: &str) -> String {
    let head = text::strip(text.split(',').next().unwrap_or("")).to_string();
    if text::split(&head).len() >= 2 {
        head
    } else {
        String::new()
    }
}

/// Python: bot/modules/trip_date_said_again.py `_the_trip_it_moves(memory)`
///
/// The plan the date is about, or None when there is no trip for it to move. The trip on record
/// has to be the plan in hand as well.
pub fn _the_trip_it_moves(memory: &mut Memory) -> Value {
    // `memory_store._stated_plan(memory) or memory_store.user(memory).get("plan")`
    let plan = match or_none(crate::memory::corrected::_stated_plan(memory)) {
        Some(plan) => Value::from(plan),
        None => vget(user(memory), "plan").clone(),
    };
    let plan = match plan.as_str() {
        Some(plan) if !text::strip(plan).is_empty() => plan.to_string(),
        _ => return Value::None,
    };
    let trip = crate::memory::trips::_the_trip_so_far(memory).val();
    if !trip.truthy() {
        return Value::None;
    }
    let on_record = fold(&sget(&trip, "plan"));
    if on_record.is_empty() {
        return Value::None;
    }
    if on_record == fold(&plan) {
        return Value::from(plan);
    }
    // A trip is not one line: "im going to lisbon in may with my sister" and then "we want to see
    // the old town and eat well" are the same trip, and the second is the plan on the table by the
    // time the month moves. The trip on record is in the chat's own plans, so the chat is still on
    // it - which is the thing this check is for. Without it "actually its june now, my sister moved
    // her holiday" moved nothing and the packing answer two turns later was still May.
    let said_before: Vec<String> = vget(user(memory), "plans").items().iter().map(|one| fold(&s_or(Some(one)))).collect();
    if said_before.contains(&on_record) {
        return vget(&trip, "plan").clone();
    }
    Value::None
}

/// Python: bot/modules/trip_date_said_again.py `_the_month_taken_back(memory)`
///
/// The month the trip on record says, when the store keeps a plan that says it.
pub fn _the_month_taken_back(memory: &mut Memory) -> Value {
    let trip = crate::memory::trips::_the_trip_so_far(memory).val();
    let month = fold(&sget(&trip, "month"));
    if month.is_empty() {
        return Value::None;
    }
    for said in vget(user(memory), "plans").items() {
        if fold(&s_or(Some(said))).contains(&month) {
            return Value::from(month);
        }
    }
    Value::None
}

/// Python: bot/modules/trip_date_said_again.py `_a_dated_fact_beside_it(memory)`
///
/// Whether someone else's dated fact - a birthday, an appointment - is in the store.
pub fn _a_dated_fact_beside_it(memory: &Memory) -> bool {
    mget(memory, "others").items().iter().any(|other| other.is_dict() && ["birthday", "appointment", "when", "date"].iter().any(|key| has(other, key)))
}

/// Python: bot/modules/trip_date_said_again.py `observe(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn observe(text: &str, memory: &mut Memory) {
    let moved = or_none(extract::a_trip_date_put_right(text))
        .or_else(|| or_none(extract::a_trip_date_put_right(&_the_first_clause(text))))
        .or_else(|| nz(_a_date_said_now(text)));
    let Some(moved) = moved else { return };
    if !_the_trip_it_moves(memory).truthy() {
        return;
    }
    let taken_back = if _a_dated_fact_beside_it(memory) { _the_month_taken_back(memory) } else { Value::None };
    let correction = dict_of(vec![("to", Value::from(moved)), ("from", taken_back)]);
    crate::memory::corrected::_apply_correction(memory, &correction, text);
}
