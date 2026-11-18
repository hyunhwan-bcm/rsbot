//! Port of PyBot's `bot/knowledge/sleep_aids.py`: needing the telly, a fan or a light to sleep,
//! and noise from outside or next door.

use crate::data::{self, FnRef};
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

fn rx(name: &str) -> &'static Re {
    data::re(&format!("bot.knowledge.sleep_aids.{name}"))
}
fn s(name: &str) -> String {
    data::string(&format!("bot.knowledge.sleep_aids.{name}"))
}
static _A_SCREEN: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_SCREEN"));
static _A_SOUND: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_SOUND"));
static _A_LIGHT: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_LIGHT"));
static _A_TELLY: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_TELLY"));
static _A_SLEEP: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_SLEEP"));
static _A_DEPENDS: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_DEPENDS"));
static _A_COMPLAINT: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_COMPLAINT"));
static _A_NOISE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_NOISE"));
static _A_OUTSIDE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_OUTSIDE"));
static _A_SOMEONE_ELSES: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_SOMEONE_ELSES"));
static _A_WALL: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_WALL"));
static _A_THROUGH_A_WALL: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_THROUGH_A_WALL"));
static SCREEN_REPLY: LazyLock<String> = LazyLock::new(|| s("SCREEN_REPLY"));
static TELLY_REPLY: LazyLock<String> = LazyLock::new(|| s("TELLY_REPLY"));
static SOUND_REPLY: LazyLock<String> = LazyLock::new(|| s("SOUND_REPLY"));
static NOISE_REPLY: LazyLock<String> = LazyLock::new(|| s("NOISE_REPLY"));
static LIGHT_REPLY: LazyLock<String> = LazyLock::new(|| s("LIGHT_REPLY"));
static WALL_REPLY: LazyLock<String> = LazyLock::new(|| s("WALL_REPLY"));

/// `_words(names)`: `\b(?:name|...)\b`, case-insensitive (an import-time builder; its regexes
/// are in the bundle, so this is only for a caller that wants a new one).
pub fn _words<S: AsRef<str>>(names: &[S]) -> Re {
    let alts: Vec<String> = names.iter().map(|n| sh::escape(n.as_ref())).collect();
    Re::new(&format!(r"\b(?:{})\b", alts.join("|")), crate::py::re::I)
}

/// The person cannot sleep without a screen, a sound or a light of their own.
pub fn a_sleep_that_needs_a_thing(text: &str) -> bool {
    if _A_SOMEONE_ELSES.is_match(text) {
        return false;
    }
    (_A_SCREEN.is_match(text) || _A_SOUND.is_match(text) || _A_LIGHT.is_match(text) || _A_TELLY.is_match(text))
        && _A_SLEEP.is_match(text)
        && _A_DEPENDS.is_match(text)
        && _A_COMPLAINT.is_match(text)
}

/// The person cannot sleep for a noise outside.
pub fn a_sleep_broken_by_a_noise(text: &str) -> bool {
    _A_SLEEP.is_match(text) && _A_COMPLAINT.is_match(text) && _A_NOISE.is_match(text) && _A_OUTSIDE.is_match(text)
}

/// The person cannot sleep for something through the wall (next door's telly, a machine).
pub fn a_sleep_broken_by_a_thing_next_door(text: &str) -> bool {
    (_A_TELLY.is_match(text) || _A_SCREEN.is_match(text) || _A_SOUND.is_match(text) || _A_THROUGH_A_WALL.is_match(text))
        && _A_SLEEP.is_match(text)
        && _A_COMPLAINT.is_match(text)
        && _A_WALL.is_match(text)
}

/// Which aid the person sleeps with: "telly", "screen", "sound" or "light".
pub fn which_aid(text: &str) -> Option<&'static str> {
    if !a_sleep_that_needs_a_thing(text) {
        return None;
    }
    if _A_TELLY.is_match(text) {
        return Some("telly");
    }
    if _A_SCREEN.is_match(text) {
        return Some("screen");
    }
    if _A_SOUND.is_match(text) {
        return Some("sound");
    }
    if _A_LIGHT.is_match(text) {
        return Some("light");
    }
    None
}

/// The card for the line (the reading's text when memory has one): the aid's card, the wall
/// card, the noise card, or None.
pub fn answer_for(text: &str, memory: Option<&Memory>) -> Option<String> {
    let said = memory.and_then(|m| m.understood.as_ref()).map(|r| sh::text_or_clean(r).to_string()).unwrap_or_default();
    let line = sh::or(&said, text);
    let Some(aid) = which_aid(line) else {
        if a_sleep_broken_by_a_thing_next_door(line) {
            return Some(WALL_REPLY.clone());
        }
        if a_sleep_broken_by_a_noise(line) {
            return Some(NOISE_REPLY.clone());
        }
        return None;
    };
    Some(match aid {
        "telly" => TELLY_REPLY.clone(),
        "screen" => SCREEN_REPLY.clone(),
        "sound" => SOUND_REPLY.clone(),
        _ => LIGHT_REPLY.clone(),
    })
}

/// The aid rows' guard: the line itself says the person needs the thing to sleep.
pub fn _the_line_itself(understood: &Understood) -> bool {
    a_sleep_that_needs_a_thing(sh::text_or_clean(understood))
}

/// The noise row's guard: a noise outside, and no aid of the person's own.
pub fn _the_noise_line(understood: &Understood) -> bool {
    let said = sh::text_or_clean(understood);
    a_sleep_broken_by_a_noise(said) && which_aid(said).is_none()
}

/// The wall row's guard: something through the wall keeps the person awake.
pub fn _the_wall_line(understood: &Understood) -> bool {
    a_sleep_broken_by_a_thing_next_door(sh::text_or_clean(understood))
}

/// The telly row's guard: the person's own telly, which they cannot sleep without.
pub fn _the_telly_line(understood: &Understood) -> bool {
    let said = sh::text_or_clean(understood);
    _A_TELLY.is_match(said)
        && _A_SLEEP.is_match(said)
        && _A_DEPENDS.is_match(said)
        && _A_COMPLAINT.is_match(said)
        && !_A_SOMEONE_ELSES.is_match(said)
}

fn g_itself(_f: &FnRef, u: &Understood) -> bool {
    _the_line_itself(u)
}
fn g_noise(_f: &FnRef, u: &Understood) -> bool {
    _the_noise_line(u)
}
fn g_wall(_f: &FnRef, u: &Understood) -> bool {
    _the_wall_line(u)
}
fn g_telly(_f: &FnRef, u: &Understood) -> bool {
    _the_telly_line(u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.sleep_aids:_the_line_itself:167", g_itself),
    ("bot.knowledge.sleep_aids:_the_noise_line:172", g_noise),
    ("bot.knowledge.sleep_aids:_the_wall_line:177", g_wall),
    ("bot.knowledge.sleep_aids:_the_telly_line:182", g_telly),
];
pub const REPLIES: &[(&str, ReplyFn)] = &[];

