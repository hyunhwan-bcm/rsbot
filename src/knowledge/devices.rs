//! Port of `bot/knowledge/devices.py`: storage on a Mac or a PC, SD cards, sharing where you
//! are, and unfriending. Word lists, replies and entries are data from the bundle.
//!
//! Build-time only (it edits the bundle's ENTRIES at import): `_stands_down_for_food`.

use crate::data;
use crate::data::FnRef;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("bot.knowledge.devices.{name}"))
}
fn string(name: &str) -> String {
    data::string(&format!("bot.knowledge.devices.{name}"))
}
fn re(name: &str) -> &'static Re {
    data::re(&format!("bot.knowledge.devices.{name}"))
}

static MAC: LazyLock<Vec<String>> = LazyLock::new(|| strs("MAC"));
static PC: LazyLock<Vec<String>> = LazyLock::new(|| strs("PC"));
static CHECK: LazyLock<Vec<String>> = LazyLock::new(|| strs("CHECK"));
static MAC_SPACE: LazyLock<String> = LazyLock::new(|| string("MAC_SPACE"));
static PC_SPACE: LazyLock<String> = LazyLock::new(|| string("PC_SPACE"));
static BOTH_SPACE: LazyLock<String> = LazyLock::new(|| string("BOTH_SPACE"));
static BOTH_CHECK: LazyLock<String> = LazyLock::new(|| string("BOTH_CHECK"));
static MAC_CHECK: LazyLock<String> = LazyLock::new(|| string("MAC_CHECK"));
static PC_CHECK: LazyLock<String> = LazyLock::new(|| string("PC_CHECK"));
static SD_CARD_COPY: LazyLock<String> = LazyLock::new(|| string("SD_CARD_COPY"));
static SD_CARD_BUY: LazyLock<String> = LazyLock::new(|| string("SD_CARD_BUY"));
static _BUYING_A_CARD: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BUYING_A_CARD"));
static _DEVICE_OF_THE_CARD: LazyLock<Vec<Vec<String>>> =
    LazyLock::new(|| data::get("bot.knowledge.devices._DEVICE_OF_THE_CARD").items().iter().map(|w| w.strs()).collect());
/// `_CARD_IN_DEVICE`: the card's answer by the index of the device's words.
static _CARD_IN_DEVICE: LazyLock<Vec<(i64, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.devices._CARD_IN_DEVICE")
        .pairs()
        .iter()
        .map(|(k, v)| (k.as_int().unwrap_or(-1), v.str().to_string()))
        .collect()
});
static _CARD_NOT_READ: LazyLock<Vec<String>> = LazyLock::new(|| strs("_CARD_NOT_READ"));
static _CARD_NOT_READING: LazyLock<String> = LazyLock::new(|| string("_CARD_NOT_READING"));
static _CARD_FULL_IN_A_PHONE: LazyLock<String> = LazyLock::new(|| string("_CARD_FULL_IN_A_PHONE"));
static _NO_SLOT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_NO_SLOT"));
static _NO_CARD_SLOT: LazyLock<String> = LazyLock::new(|| string("_NO_CARD_SLOT"));
static _DISK_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_DISK_WORDS"));
static _ASKS_TO_STOP_SHARING: LazyLock<&'static Re> = LazyLock::new(|| re("_ASKS_TO_STOP_SHARING"));
static _ASKS_HOW_IT_WORKS: LazyLock<&'static Re> = LazyLock::new(|| re("_ASKS_HOW_IT_WORKS"));
static _ASKS_HOW_THEY_KNOW: LazyLock<&'static Re> = LazyLock::new(|| re("_ASKS_HOW_THEY_KNOW"));
static _ASKS_WHETHER_TO_SHARE: LazyLock<&'static Re> = LazyLock::new(|| re("_ASKS_WHETHER_TO_SHARE"));
static _ASKS_WHO_CAN_SEE: LazyLock<&'static Re> = LazyLock::new(|| re("_ASKS_WHO_CAN_SEE"));
static _ASKS_FOR_THE_STEPS: LazyLock<&'static Re> = LazyLock::new(|| re("_ASKS_FOR_THE_STEPS"));
static SHARE_STEPS: LazyLock<String> = LazyLock::new(|| string("SHARE_STEPS"));
static HOW_THEY_KNOW: LazyLock<String> = LazyLock::new(|| string("HOW_THEY_KNOW"));

static UNFOLLOW: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bunfollow", 0));
static BLOCK: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bblock(?:ed|ing)?\b", 0));
static UNFRIEND: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:unfriend|defriend|remove (?:a )?friend)", 0));

fn any_in<S: AsRef<str>>(text: &str, words: &[S]) -> bool {
    words.iter().any(|w| text.contains(w.as_ref()))
}

/// Which device the card is going into (its answer), or None when the line names no device.
pub fn _the_device_of_the_card(text: &str) -> Option<String> {
    for (index, words) in _DEVICE_OF_THE_CARD.iter().enumerate() {
        if any_in(text, words) {
            let answer = _CARD_IN_DEVICE.iter().find(|(k, _)| *k == index as i64).map(|(_, v)| v.clone());
            // Python: a KeyError if the index had no answer; every index has one.
            return Some(answer.expect("_CARD_IN_DEVICE has every device index"));
        }
    }
    None
}

/// A full card wants its photos copied off; a card being chosen wants a size and a speed.
pub fn _sd_card(_memory: &mut Memory, understood: &Understood) -> String {
    let text = _text(understood);
    if any_in(&text, &_NO_SLOT) {
        return _NO_CARD_SLOT.clone();
    }
    if any_in(&text, &_CARD_NOT_READ) {
        return _CARD_NOT_READING.clone();
    }
    if let Some(in_a_device) = _the_device_of_the_card(&text).filter(|s| !s.is_empty()) {
        if any_in(&text, &["full", "no room", "no space", "filled"])
            && ["phone", "android", "samsung", "tablet", "galaxy", "pixel"].iter().any(|w| text.contains(w))
        {
            return _CARD_FULL_IN_A_PHONE.clone();
        }
        return in_a_device;
    }
    if any_in(&text, &_BUYING_A_CARD) && !any_in(&text, &["full", "no room", "no space", "running out"]) {
        return SD_CARD_BUY.clone();
    }
    SD_CARD_COPY.clone()
}

/// The typed line and its cleaned reading, side by side, lower-cased.
pub fn _text(understood: &Understood) -> String {
    format!("{} {}", understood.text, understood.clean).to_lowercase()
}

fn names_one(text: &str, word: &str) -> bool {
    format!(" {text}").contains(&format!(" {word}")) || text.starts_with(word)
}

/// Whether the line names a Mac.
pub fn _names_mac(understood: &Understood) -> bool {
    let text = _text(understood);
    MAC.iter().any(|w| names_one(&text, w))
}

/// Whether the line names a Windows machine (the bare "disk" does not count).
pub fn _names_pc(understood: &Understood) -> bool {
    let text = _text(understood);
    PC.iter().filter(|w| w.as_str() != "disk").any(|w| names_one(&text, w))
}

/// Whether the line names a disk.
pub fn _names_a_disk(understood: &Understood) -> bool {
    any_in(&_text(understood), &_DISK_WORDS)
}

/// Whether the line asks how much is left rather than how to free it.
pub fn _asking_how_much(understood: &Understood) -> bool {
    any_in(&_text(understood), &CHECK)
}

/// The Mac's storage screen, only when a Mac is in the line.
pub fn _mac_space(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    if !_names_mac(understood) {
        return None;
    }
    Some(if _asking_how_much(understood) { MAC_CHECK.clone() } else { MAC_SPACE.clone() })
}

/// The PC's storage screen; a Mac gets its own, and a bare disk gets both paths.
pub fn _pc_space(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    if !(_names_pc(understood) || _names_mac(understood) || _names_a_disk(understood)) {
        return None;
    }
    let how_much = _asking_how_much(understood);
    if _names_mac(understood) && !_names_pc(understood) {
        return Some(if how_much { MAC_CHECK.clone() } else { MAC_SPACE.clone() });
    }
    if !_names_pc(understood) {
        return Some(if how_much { BOTH_CHECK.clone() } else { BOTH_SPACE.clone() });
    }
    Some(if how_much { PC_CHECK.clone() } else { PC_SPACE.clone() })
}

/// `text or clean or " ".join(words)`: the line a stand-down reads.
fn said_or_words(understood: &Understood) -> String {
    let said = if understood.text.is_empty() { &understood.clean } else { &understood.text };
    if said.is_empty() { understood.words.join(" ") } else { said.clone() }
}

/// True unless the line asks for sharing to stop, which the turn-off row answers.
pub fn _not_asking_to_stop(understood: &Understood) -> bool {
    !_ASKS_TO_STOP_SHARING.is_match(&said_or_words(understood))
}

/// The line's own words, from whichever reading a callable is handed.
pub fn _the_said(understood: &Understood) -> String {
    for value in [&understood.text, &understood.clean, &understood.norm_text, &understood.plain] {
        if !value.is_empty() {
            return value.clone();
        }
    }
    understood.words.join(" ")
}

/// True when the line asks whether to share rather than how.
pub fn _asks_whether_to_share(understood: &Understood) -> bool {
    _ASKS_WHETHER_TO_SHARE.is_match(&_the_said(understood))
}

/// True when the line asks who can already see where the person is.
pub fn _asks_who_can_see(understood: &Understood) -> bool {
    let said = _the_said(understood);
    if _ASKS_FOR_THE_STEPS.is_match(&said) {
        return false;
    }
    _ASKS_WHO_CAN_SEE.is_match(&said)
}

/// The steps for sharing where you are, or what the shape of the share is when the line asks
/// whether to.
pub fn _format_sharing_where_i_am(_memory: &mut Memory, understood: &Understood) -> String {
    if _asks_who_can_see(understood) {
        return HOW_THEY_KNOW.clone();
    }
    if _asks_whether_to_share(understood) {
        return "That is yours to weigh, and the setting decides how much they see rather than \
                whether to trust them: live sharing keeps updating for as long as the timer runs, so \
                they watch you move and still have it afterwards, while a location sent once says \
                only where you were at that moment. In WhatsApp tap the + or the paperclip, choose \
                Location, then Send your current location for the point or Share live location for \
                15 minutes, 1 hour or 8 hours; Google Maps, your picture then Location sharing, \
                lists every share you have running and stops any of them."
            .to_string();
    }
    SHARE_STEPS.clone()
}

/// True unless the line asks whether to share rather than how.
pub fn _not_asking_whether_to_share(understood: &Understood) -> bool {
    !_ASKS_WHETHER_TO_SHARE.is_match(&said_or_words(understood))
}

/// True unless the line asks how somebody came to know where the person is.
pub fn _not_asking_how_they_know(understood: &Understood) -> bool {
    !_ASKS_HOW_THEY_KNOW.is_match(&said_or_words(understood))
}

/// True only when the line asks how somebody came to know where the person is.
pub fn _asks_how_they_know(understood: &Understood) -> bool {
    !_not_asking_how_they_know(understood)
}

/// True unless the line asks how the phone knows, which the satellites card answers.
pub fn _not_asking_how_it_works(understood: &Understood) -> bool {
    !_ASKS_HOW_IT_WORKS.is_match(&said_or_words(understood))
}

/// The stand-downs the sharing row needs: not the opposite ask, not a how-do-they-know, and not
/// the how-it-works one ("who can see my location" passes).
pub fn _not_a_sharing_ask(understood: &Understood) -> bool {
    if _asks_who_can_see(understood) {
        return true;
    }
    _not_asking_to_stop(understood) && _not_asking_how_they_know(understood) && _not_asking_how_it_works(understood)
}

/// The button that was asked for first, then the other one, then how far it goes.
pub fn _unfriend_or_block(_memory: &mut Memory, understood: &Understood) -> String {
    let line = said_or_words(understood).to_lowercase();
    let unfollow = UNFOLLOW.is_match(&line);
    let block = BLOCK.is_match(&line);
    let unfriend = UNFRIEND.is_match(&line);
    if unfollow && !unfriend {
        return "On Facebook open their profile, tap Following, then Unfollow - they are not \
                told, and it takes their posts out of your feed without ending the \
                friendship. On Instagram it is the same two taps: their profile, then \
                Following, then Unfollow. If you want them gone further than that, the ... \
                menu on their profile has Block, which stops them finding you, messaging you \
                or seeing anything you post, and you can undo it in Settings > Blocked."
            .to_string();
    }
    if block && !unfriend {
        return "Blocking stops them finding you, messaging you or seeing anything you post: \
                open their profile, tap the ... menu, then Block. They are not told, and you \
                can undo it in Settings > Blocked. Unfriending is the smaller step - Friends, \
                then Unfriend - which stops the posts but still lets them message you."
            .to_string();
    }
    "On Facebook open their profile, tap Friends, then Unfriend - they are not told. \
     Blocking goes further: the ... menu on their profile, then Block. On Instagram \
     open their profile, tap Following to Unfollow, or the ... menu to Block. \
     Unfriending only stops the posts; blocking stops them finding you, messaging you \
     or seeing anything you post, and you can undo it in Settings > Blocked."
        .to_string()
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.devices:_asks_how_they_know:591", |_: &FnRef, u: &Understood| _asks_how_they_know(u)),
    ("bot.knowledge.devices:_not_a_sharing_ask:606", |_: &FnRef, u: &Understood| _not_a_sharing_ask(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.devices:_format_sharing_where_i_am:550", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_format_sharing_where_i_am(m, u))
    }),
    ("bot.knowledge.devices:_mac_space:290", |_: &FnRef, m: &mut Memory, u: &Understood| _mac_space(m, u)),
    ("bot.knowledge.devices:_pc_space:299", |_: &FnRef, m: &mut Memory, u: &Understood| _pc_space(m, u)),
    ("bot.knowledge.devices:_sd_card:239", |_: &FnRef, m: &mut Memory, u: &Understood| Some(_sd_card(m, u))),
    ("bot.knowledge.devices:_unfriend_or_block:648", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_unfriend_or_block(m, u))
    }),
];


// Stubs appended by harness/integrate_skeleton.py: the Python functions this file does not
// define yet, with signatures from the generated contract. Port them in place (move a
// function up into the file when you port it).
pub use self::skeleton_stubs::*;
#[allow(unused_imports, unused_variables, non_snake_case, dead_code, clippy::all)]
mod skeleton_stubs {
    use super::*;
    use crate::data::{self, Value};
    use crate::knowledge::entry::Entry;
    use crate::runtime::Candidate;
    use crate::state::{Memory, Turn};
    use crate::understood::Understood;

    /// Python: bot/knowledge/devices.py:348 `_stands_down_for_food(one)` (never called on the trace corpus)
    pub fn _stands_down_for_food(one: &str) -> Value {
        crate::unported!("bot.knowledge.devices:_stands_down_for_food")
    }

}

