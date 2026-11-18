//! Port of `bot/knowledge/joints.py`: a joint replacement (healing, how long it lasts, sleeping,
//! soreness) and a joint that clicks or locks.

use crate::data::FnRef;
use crate::knowledge::d2dup_g10::globals;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;

globals!("bot.knowledge.joints",
    string { SHOULDER_TIMELINE_CARD, TIMELINE_CARD, SHOULDER_IMPLANT_CARD, IMPLANT_CARD, SHOULDER_SORE_CARD,
             SORE_CARD, SHOULDER_SLEEPING_CARD, SLEEPING_CARD, TRIGGER_FINGER_CARD, LOCKED_CARD },
    strs { SHOULDER, ROAD_WORDS, PARTS, THE_JOINT_WORDS, LOCK_PARTS },
    re { A_REPLACING_A_THING },
    value { ONE });

/// The healing schedule, spoken about the joint the line named.
pub fn _timeline(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    Some(if _joint(understood) == "shoulder" { SHOULDER_TIMELINE_CARD.clone() } else { TIMELINE_CARD.clone() })
}

/// How long the implant lasts, for the joint the line named.
pub fn _implant_life(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    Some(if _joint(understood) == "shoulder" { SHOULDER_IMPLANT_CARD.clone() } else { IMPLANT_CARD.clone() })
}

/// Soreness after the operation, for the joint the line named.
pub fn _soreness(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    Some(if _joint(understood) == "shoulder" { SHOULDER_SORE_CARD.clone() } else { SORE_CARD.clone() })
}

/// Sleeping after the operation, for the joint the line named.
pub fn _sleeping(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    Some(if _joint(understood) == "shoulder" { SHOULDER_SLEEPING_CARD.clone() } else { SLEEPING_CARD.clone() })
}

/// Every word of the line, and of the turn it leans on, as a set.
pub fn _words(understood: &Understood) -> HashSet<String> {
    understood.words.iter().chain(understood.background.iter()).map(|w| w.to_lowercase()).collect()
}

/// The article for the word: "an ankle", "a knuckle".
pub fn _a(word: &str) -> String {
    let vowel = word.chars().next().is_some_and(|c| "aeiou".contains(c));
    format!("{}{word}", if vowel || word.is_empty() { "an " } else { "a " })
}

/// "shoulder" if the line named one, otherwise "replacement" for a hip or a knee.
pub fn _joint(understood: &Understood) -> &'static str {
    let words = _words(understood);
    if SHOULDER.iter().any(|w| words.contains(w)) { "shoulder" } else { "replacement" }
}

/// True unless the line is about a road that names no joint - then a roadworks line answers.
pub fn _not_a_road(understood: &Understood) -> bool {
    let words = _words(understood);
    if !ROAD_WORDS.iter().any(|w| words.contains(w)) {
        return true;
    }
    PARTS.iter().any(|w| words.contains(w))
        || [
            "hip", "knee", "shoulder", "surgeon", "operation", "physio", "hospital", "nurse", "ward", "recovery",
            "physiotherapy",
        ]
        .iter()
        .any(|w| words.contains(*w))
}

/// True unless the line replaces a thing that is not a joint - a bath, a boiler, a road.
pub fn _not_something_else_replaced(understood: &Understood) -> bool {
    let things = A_REPLACING_A_THING.findall_str(&understood.clean.to_lowercase());
    if things.is_empty() {
        return true;
    }
    things.iter().any(|w| THE_JOINT_WORDS.contains(w))
}

/// The guard `_a_replacement` gives a card with no guard of its own.
fn replacement_guard(_: &FnRef, understood: &Understood) -> bool {
    _not_a_road(understood) && _not_something_else_replaced(understood)
}

/// The guard `_a_replacement` gives a card that brought a guard (`asked`) of its own.
fn replacement_guard_asked(f: &FnRef, understood: &Understood) -> bool {
    _not_a_road(understood)
        && _not_something_else_replaced(understood)
        && f.cell("asked").and_then(|a| a.as_fn()).is_some_and(|a| crate::knowledge::registry::guard(a, understood))
}

fn one(word: &str) -> String {
    ONE.get(word).and_then(|v| v.as_str()).unwrap_or(word).to_string()
}

/// The noise answer, about the part the line named: "A knuckle that clicks...".
pub fn _noisy_joint(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = _words(understood);
    let part = PARTS.iter().find(|w| words.contains(*w)).map(|w| one(w)).unwrap_or_else(|| "joint".to_string());
    Some(text::capitalize(&_a(&part))
        + " that clicks or pops and does not hurt is almost always harmless: a tendon flicking over a bump, or gas coming out of the joint fluid, and it needs nothing doing. Keep the muscles around it working, because clicking that turns into pain is usually a strength problem rather than the noise. If it starts to hurt, swell or lock, have a physio or your GP look at it.")
}

/// The lock answer, about the part the line named, with the finger's own self-care.
pub fn _locked_joint(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = _words(understood);
    let part = LOCK_PARTS.iter().find(|w| words.contains(*w)).map(|w| one(w)).unwrap_or_else(|| "joint".to_string());
    if ["finger", "fingers", "thumb", "thumbs"].contains(&part.as_str()) {
        return Some(TRIGGER_FINGER_CARD.clone());
    }
    Some(LOCKED_CARD.replace("{part}", &part))
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.joints:_a_replacement.<locals>.<lambda>:265", replacement_guard),
    ("bot.knowledge.joints:_a_replacement.<locals>.<lambda>:268", replacement_guard_asked),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.joints:_timeline:128", |_: &FnRef, m: &mut Memory, u: &Understood| _timeline(m, u)),
    ("bot.knowledge.joints:_implant_life:135", |_: &FnRef, m: &mut Memory, u: &Understood| _implant_life(m, u)),
    ("bot.knowledge.joints:_soreness:141", |_: &FnRef, m: &mut Memory, u: &Understood| _soreness(m, u)),
    ("bot.knowledge.joints:_sleeping:147", |_: &FnRef, m: &mut Memory, u: &Understood| _sleeping(m, u)),
    ("bot.knowledge.joints:_noisy_joint:435", |_: &FnRef, m: &mut Memory, u: &Understood| _noisy_joint(m, u)),
    ("bot.knowledge.joints:_locked_joint:571", |_: &FnRef, m: &mut Memory, u: &Understood| _locked_joint(m, u)),
];


// ---------------------------------------------------------------------------------------------
// The row factory (`knowledge::built`).

/// `_a_replacement(*args, **kwargs)`: one of the cards about a joint that was replaced, which a
/// road line must not reach - `_not_a_road` and `_not_something_else_replaced` folded into its
/// guard, with the row's own guard (`asked`) after them.
#[allow(clippy::too_many_arguments)]
pub fn _a_replacement(
    id: &str,
    key: Vec<crate::knowledge::entry::KeyPart>,
    reply: crate::knowledge::entry::Reply,
    avoid: &[crate::data::Value],
    bias: i64,
    ask: Vec<crate::knowledge::entry::KeyPart>,
    asked: Option<FnRef>,
) -> crate::knowledge::entry::Entry {
    use crate::knowledge::built;
    let guard = match asked {
        None => built::function("bot.knowledge.joints:_a_replacement.<locals>.<lambda>:265"),
        Some(g) => built::closure("bot.knowledge.joints:_a_replacement.<locals>.<lambda>:268", vec![("asked", crate::data::Value::Fn(std::sync::Arc::new(g)))]),
    };
    built::entry(id, key, reply, avoid, bias, ask, Some(guard))
}
