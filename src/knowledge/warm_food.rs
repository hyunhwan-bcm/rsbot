//! Port of `bot/knowledge/warm_food.py`: warming bread and pastry - the times, the toaster, and
//! the defrost first when the pastry is frozen.

use crate::data::{self, Value};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;

fn g(name: &str) -> &'static Value {
    data::get(&format!("bot.knowledge.warm_food.{name}"))
}

fn words_of(reading: &Understood) -> HashSet<String> {
    reading.words.iter().map(|w| text::lower(w)).collect()
}

fn meets(words: &HashSet<String>, name: &str) -> bool {
    g(name).items().iter().any(|w| words.contains(w.str()))
}

/// The bread times, or the toaster ones when the line says toast.
pub fn _bread_reply(_memory: &Memory, reading: &Understood) -> String {
    let words = words_of(reading);
    if meets(&words, "_TOASTING") && !meets(&words, "_OVEN_WORDS") {
        return g("_TOAST_REPLY").str().to_string();
    }
    g("_BREAD_REPLY").str().to_string()
}

/// The pastry times, leading with the defrost when that is what the line asked for.
pub fn _pastry_reply(_memory: &Memory, reading: &Understood) -> String {
    let words = words_of(reading);
    let said = text::lower(&reading.clean);
    let cold = meets(&words, "_FROM_FROZEN") || said.contains("frozen");
    if meets(&words, "_THAWING") && !meets(&words, "_HEATING") {
        return g("_DEFROST_REPLY").str().to_string();
    }
    if cold {
        return format!("{}{}", g("_PASTRY_REPLY").str(), g("_FROZEN_CLAUSE").str());
    }
    g("_PASTRY_REPLY").str().to_string()
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.warm_food:_bread_reply:75", |_, m, u| Some(_bread_reply(m, u))),
    ("bot.knowledge.warm_food:_pastry_reply:145", |_, m, u| Some(_pastry_reply(m, u))),
];
