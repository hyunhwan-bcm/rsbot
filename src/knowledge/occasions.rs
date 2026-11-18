//! Port of `bot/knowledge/occasions.py`: what to bring to a do (a wake, a picnic, a potluck...).

use crate::data::FnRef;
use crate::knowledge::d2dup_g10::{crosses_a_border, globals};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

globals!("bot.knowledge.occasions",
    strs { _NOT_A_DISH, _HOST_OWN, _MAKE_ASKS, _SPREAD_WORDS, BRING_CARRY_WORDS, BRING_ASKS,
           _COOKING_ASKS, BRING_HOST_NAMES, _NOT_A_DISH_TO_CARRY, _TRIP, GENERIC_FOOD },
    string { _ANY_DISH },
    value { BRING_DISH });

static WORD: LazyLock<Re> = LazyLock::new(|| Re::new("[a-z0-9']+", 0));

/// `(names, hint)` rows of BRING_DISH, in order.
static DISHES: LazyLock<Vec<(Vec<String>, String)>> =
    LazyLock::new(|| BRING_DISH.items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect());

/// One word equals another, allowing a plural: "picnics" is "picnic".
pub fn _same(word: &str, wanted: &str) -> bool {
    let word = word.to_lowercase();
    let word = word.trim_matches('\'');
    let wanted = wanted.to_lowercase();
    if word == wanted {
        return true;
    }
    word == format!("{wanted}s") || word == format!("{wanted}es") || wanted == format!("{word}s") || wanted == format!("{word}es")
}

/// The words of the line and of its clean form, lower-cased.
pub fn _words_of(understood: &Understood) -> Vec<String> {
    let said = understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" ");
    WORD.findall_str(&format!("{said} {}", understood.clean.to_lowercase()))
}

/// The occasion the line names decides what travels.
pub fn what_to_bring(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    let has = |phrases: &[String]| phrases.iter().any(|p| clean.contains(p.as_str()));
    // A quantity is the crowd table's question.
    if has(&_NOT_A_DISH) {
        return None;
    }
    let words = _words_of(understood);
    // The host's own do: every hint below is written for a guest.
    if has(&_HOST_OWN) {
        return None;
    }
    let split = crate::py::text::split_ref(&clean);
    if has(&_MAKE_ASKS)
        && _SPREAD_WORDS.iter().any(|p| split.contains(&p.as_str()))
        && !words.iter().any(|w| ["bring", "take"].iter().any(|c| _same(w, c)))
    {
        return None;
    }
    let carry = words.iter().any(|w| BRING_CARRY_WORDS.iter().any(|c| _same(w, c)));
    // The person is asking where to go, not what to turn up with.
    if understood.wants.as_deref() == Some("place") {
        return None;
    }
    let asker = has(&BRING_ASKS);
    // A make word with no bring or take in the line is the host's, and the cooking rows answer it.
    if has(&_COOKING_ASKS) && !carry {
        return None;
    }
    for (names, hint) in DISHES.iter() {
        for name in names {
            if BRING_HOST_NAMES.contains(name) && !carry {
                continue;
            }
            let parts: Vec<&str> = crate::py::text::split_ref(name);
            if words.len() < parts.len() {
                continue;
            }
            for start in 0..=(words.len() - parts.len()) {
                if parts.iter().enumerate().all(|(offset, part)| _same(&words[start + offset], part)) {
                    return Some(hint.clone());
                }
            }
        }
    }
    // No occasion named, and the line asks what to bring or make.
    if asker && !crosses_a_border(&clean) && !has(&_NOT_A_DISH_TO_CARRY) && !has(&_TRIP)
        && words.iter().any(|w| GENERIC_FOOD.iter().any(|f| _same(w, f)))
    {
        return Some(_ANY_DISH.clone());
    }
    None
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] =
    &[("bot.knowledge.occasions:what_to_bring:185", |_: &FnRef, m: &mut Memory, u: &Understood| what_to_bring(m, u))];
