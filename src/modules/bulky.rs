//! Port of `bot/modules/bulky.py`: a finished bulky thing - a carpet, a sofa, a fridge - and how it goes out.
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
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.bulky.", stringify!($name))));
    )*};
}
rx!(FLOOR, FURNITURE, ELECTRICAL, GARDEN, DISPOSAL, REPLACING, FINISHED, POINTING, NOT_OURS, KEEPS_ITS_ANSWER);

/// Python: bot/modules/bulky.py `_clean(text)`
pub fn _clean(text: &str) -> String {
    fold(text)
}

/// Python: bot/modules/bulky.py `_group(text)`: the (group, thing) a line names, or ("", "").
pub fn _group(text: &str) -> (String, String) {
    for (name, pattern) in [("floor", &*FLOOR), ("furniture", &*FURNITURE), ("electrical", &*ELECTRICAL), ("garden", &*GARDEN)] {
        if let Some(found) = pattern.search(text) {
            return (name.to_string(), found.as_str().to_lowercase());
        }
    }
    (String::new(), String::new())
}

/// Python: bot/modules/bulky.py `_the_thing(text, memory)`
///
/// The thing the line is about, from this line or the one before it.
pub fn _the_thing(text: &str, memory: &Memory) -> (String, String) {
    let (group, thing) = _group(text);
    if !thing.is_empty() {
        return (group, thing);
    }
    if !POINTING.is_match(text) {
        return (String::new(), String::new());
    }
    for said in recent(memory, 3).iter().rev() {
        let (group, thing) = _group(&said.user);
        if !thing.is_empty() {
            return (group, thing);
        }
    }
    (String::new(), String::new())
}

/// Python: bot/modules/bulky.py `_reply(group, thing)`
pub fn _reply(group: &str, thing: &str) -> String {
    if group == "floor" {
        return format!(
            "Roll the {0} up and cut it into lengths you can lift, then take it to the recycling \
             centre - most tips take {0} in the general waste skip. If it will not fit in a car, \
             book a bulky waste collection with your council; most charge twenty to forty pounds \
             for a doorstep pickup. Check your council's site for the fee and the booking, \
             because I can't see your council's charges from here.",
            thing
        );
    }
    if group == "furniture" {
        return format!(
            "Take the {0} apart if it will come apart, then book a bulky waste collection with \
             your council - most charge twenty to forty pounds - or take it to the recycling \
             centre. If it is still usable, a charity or a local reuse group may collect it; the \
             British Heart Foundation collects furniture in many areas. Check your council's \
             site for the fee and the collection, because I can't see your council's charges \
             from here.",
            thing
        );
    }
    if group == "electrical" {
        return format!(
            "Anything with a plug or a battery is electrical waste, so it cannot go in the bin: \
             a shop that sells the same kind of thing has to take it back, and the recycling \
             centre has an electricals point. A {0} is also a bulky collection if you cannot \
             move it - ask the shop delivering a new one to take the old one. Check your \
             council's site for the fee, because I can't see your council's charges from here.",
            thing
        );
    }
    format!(
        "Dismantle the {0} as far as it will come apart and take it to the recycling centre, or \
         book a bulky waste collection with your council; most charge twenty to forty pounds. \
         The metal goes in the scrap metal skip and the rest in general waste, and green garden \
         waste goes in the council's green bin. Check your council's site for the fee and any \
         permit, because I can't see your council's charges from here.",
        thing
    )
}

/// Python: bot/modules/bulky.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let clean = _clean(text);
    if clean.is_empty() {
        return None;
    }
    if !DISPOSAL.is_match(&clean) && !REPLACING.is_match(&clean) {
        return None;
    }
    if NOT_OURS.is_match(&clean) || KEEPS_ITS_ANSWER.is_match(&clean) {
        return None;
    }
    let (group, thing) = _the_thing(&clean, memory);
    if thing.is_empty() {
        return None;
    }
    if REPLACING.is_match(&clean) && !DISPOSAL.is_match(&clean) {
        // Only a floor covering has an old one to get out; a replaced boiler or a fitted kitchen
        // is another kind of job and keeps its own answer.
        if group != "floor" {
            return None;
        }
        return Some("Good. What are you doing with the old one - tip or a council collection?".to_string());
    }
    if !FINISHED.is_match(&clean) && !POINTING.is_match(&clean) {
        return None;
    }
    Some(_reply(&group, &thing))
}
