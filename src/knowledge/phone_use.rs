//! Port of PyBot's `bot/knowledge/phone_use.py`: which phone to buy for what it will be used for.

use crate::data::{self, FnRef, Value};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

fn re(name: &str) -> &'static Re {
    data::re(&format!("bot.knowledge.phone_use.{name}"))
}
static _PHOTOS: LazyLock<&'static Re> = LazyLock::new(|| re("_PHOTOS"));
static _CALLS: LazyLock<&'static Re> = LazyLock::new(|| re("_CALLS"));
static _PHONE_THING: LazyLock<&'static Re> = LazyLock::new(|| re("_PHONE_THING"));
static _HEAVY: LazyLock<&'static Re> = LazyLock::new(|| re("_HEAVY"));
static _NOT_A_USE: LazyLock<&'static Re> = LazyLock::new(|| re("_NOT_A_USE"));
static _A_BUYING_QUESTION: LazyLock<&'static Re> = LazyLock::new(|| re("_A_BUYING_QUESTION"));

/// The line as typed (text, else clean).
pub fn _the_use(understood: &Understood) -> String {
    crate::knowledge::d1_shared::text_or_clean(understood).to_string()
}

/// Whether the text names a phone.
pub fn _phone_words(line: &str) -> bool {
    _PHONE_THING.is_match(line)
}

/// Whether the line names a use for a phone (not a fault or another device).
pub fn _a_use(understood: &Understood) -> bool {
    let line = _the_use(understood);
    if text::strip(&line).is_empty() {
        return false;
    }
    !_NOT_A_USE.is_match(&line)
}

/// The phone to buy for the use the line names (photos, calls, games), on a buying question
/// about a phone (here or in the topic so far); None otherwise.
pub fn _phone_for_that_use(memory: &Memory, understood: &Understood) -> Option<String> {
    let line = _the_use(understood).to_lowercase();
    let mut before = String::new();
    if let Some(topic @ Value::Dict(_)) = memory.extra.get("topic") {
        let pick = |k: &str| topic.get(k).filter(|v| v.truthy());
        before = match pick("text").or_else(|| pick("of")) {
            Some(Value::Str(s)) => s.to_string(),
            Some(v) => format!("{v:?}"),
            None => String::new(),
        };
    }
    if !_phone_words(&line) && !_phone_words(&before) {
        return None;
    }
    if !(_A_BUYING_QUESTION.is_match(&line) || _A_BUYING_QUESTION.is_match(&before)) {
        return None;
    }
    let photos = _PHOTOS.is_match(&line);
    let calls = _CALLS.is_match(&line);
    let heavy = _HEAVY.is_match(&line);
    if !(photos || calls || heavy) {
        return None;
    }
    if photos && !heavy {
        return Some("Photos are the part worth paying for: a mid-range phone at £200 to £300 takes pictures as good as a flagship did a few years ago, so put the extra into storage - 128GB at least, 256GB if you keep videos - and let cloud backup at a few pounds a month carry the rest.".into());
    }
    if calls && !photos && !heavy {
        return Some("Calls and texts need nothing special, so buy the cheapest phone that still gets security updates: about £120 to £180 new, less refurbished. Check the maker promises four or five years of updates and that the battery is 4,500mAh or more, and ignore how many cameras it has.".into());
    }
    if heavy && !photos {
        return Some("For games or work on a phone, spend on the processor and the screen: £300 to £450 buys a mid-range chip that will not stutter, and 256GB of storage, because games take a lot of room. A two-year-old flagship at the same money is the other good buy if the battery health is over 85%.".into());
    }
    Some("For photos and everything else, £200 to £300 buys a mid-range phone that covers calls, photos, maps and video, and 128GB of storage matters more than the camera. Buy it outright rather than on a contract if you can, because the contract usually costs more over two years.".into())
}

fn g_a_use(_f: &FnRef, u: &Understood) -> bool {
    _a_use(u)
}
fn r_phone(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _phone_for_that_use(m, u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.phone_use:_a_use:69", g_a_use)];
pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.phone_use:_phone_for_that_use:77", r_phone)];
