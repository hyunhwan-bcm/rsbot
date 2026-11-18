//! Port of `bot/knowledge/left_on.py`: is it safe to leave the boiler, the charger, the fridge...
//! on.

use crate::data::FnRef;
use crate::knowledge::d2dup_g10::globals;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

globals!("bot.knowledge.left_on",
    re { _LEAVES_AN_OBJECT },
    string { FIRE_TEXT },
    strs { _IN_THE_THING, _AWAY, _RISK, _ONLY_HEATING },
    set { _FIRE_WORDS },
    value { _GROUPS });

/// `(group, text)` rows, in order.
static GROUPS: LazyLock<Vec<(Vec<String>, String)>> =
    LazyLock::new(|| _GROUPS.items().iter().map(|row| (row.items()[0].strs(), row.items()[1].str().to_string())).collect());

fn line(understood: &Understood) -> String {
    let said = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    text::squash(&said.to_lowercase())
}

/// False for the keeping questions that name an appliance but are not about it.
pub fn _left_on_answers_this(understood: &Understood) -> bool {
    let line = line(understood);
    if _IN_THE_THING.iter().any(|p| line.contains(p.as_str())) {
        return false;
    }
    if _LEAVES_AN_OBJECT.is_match(&line) {
        return false;
    }
    if (line.contains("fridge") || line.contains("freezer")) && !_AWAY.iter().any(|w| line.contains(w.as_str())) {
        return false;
    }
    true
}

/// False for "should i leave the heating on overnight", which is the timer's question.
pub fn _a_heating_habit_is_not_a_risk(understood: &Understood) -> bool {
    let line = line(understood);
    if _RISK.iter().any(|w| line.contains(w.as_str())) {
        return true;
    }
    !_ONLY_HEATING.iter().any(|w| line.contains(w.as_str()))
}

/// The answer for the thing that is on, and the fire answer when it is anything that heats.
pub fn _left_on_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = line(understood);
    let fire = understood.words.iter().any(|w| _FIRE_WORDS.contains(w));
    for (group, reply) in GROUPS.iter() {
        if group.iter().any(|w| line.contains(w.as_str())) {
            // a candle in front of the fairy lights is still a candle: what heats wins
            if fire && !group.iter().any(|w| w == "lights") {
                return Some(FIRE_TEXT.clone());
            }
            return Some(reply.clone());
        }
    }
    Some(FIRE_TEXT.clone())
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.left_on:_left_on_answers_this:121", |_: &FnRef, u: &Understood| _left_on_answers_this(u)),
    ("bot.knowledge.left_on:_a_heating_habit_is_not_a_risk:139", |_: &FnRef, u: &Understood| {
        _a_heating_habit_is_not_a_risk(u)
    }),
];

pub const REPLIES: &[(&str, ReplyFn)] =
    &[("bot.knowledge.left_on:_left_on_reply:155", |_: &FnRef, m: &mut Memory, u: &Understood| _left_on_reply(m, u))];
