//! Port of `bot/knowledge/house_odds.py`: odd household questions (cooker hoods, watch straps,
//! a machine that bangs).

use crate::data::FnRef;
use crate::knowledge::d2dup_g10::globals;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::state::Memory;
use crate::understood::Understood;

globals!("bot.knowledge.house_odds",
    re { _STRAP, _SOAKING },
    string { HOOD_NOISE_TEXT, HOOD_GREASE_TEXT },
    set { HOOD_NOISE_WORDS },
    strs { _THE_MACHINE, _THE_PLUMBING });

/// The grease answer for a grease question, the noise answer for a noise one.
pub fn _hood_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    if understood.words.iter().any(|w| HOOD_NOISE_WORDS.contains(w)) {
        return Some(HOOD_NOISE_TEXT.clone());
    }
    Some(HOOD_GREASE_TEXT.clone())
}

/// True for the strap itself; False for a watch that got washed or wet.
pub fn _a_strap_not_the_device(understood: &Understood) -> bool {
    let text = understood.clean.to_lowercase();
    _STRAP.is_match(&text) || !_SOAKING.is_match(&text)
}

/// A machine banging on its own (not the plumbing behind it). Python takes
/// `(memory_or_understood, understood=None)`; the reading is what it reads.
pub fn _the_machine_its_own_bang(understood: &Understood) -> bool {
    let words = format!(" {} ", understood.words.join(" "));
    if !_THE_MACHINE.iter().any(|m| words.contains(&format!(" {m} "))) {
        return false;
    }
    !_THE_PLUMBING.iter().any(|p| words.contains(&format!(" {p} ")))
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.house_odds:_a_strap_not_the_device:87", |_: &FnRef, u: &Understood| _a_strap_not_the_device(u)),
    ("bot.knowledge.house_odds:<lambda>:335", |_: &FnRef, u: &Understood| !_the_machine_its_own_bang(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] =
    &[("bot.knowledge.house_odds:_hood_reply:67", |_: &FnRef, m: &mut Memory, u: &Understood| _hood_reply(m, u))];
