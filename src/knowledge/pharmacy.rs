//! Port of PyBot's `bot/knowledge/pharmacy.py`: what the pharmacy counter sells for a symptom.

use crate::data::{self, FnRef};
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("bot.knowledge.pharmacy.{name}"))
}
fn s(name: &str) -> String {
    data::string(&format!("bot.knowledge.pharmacy.{name}"))
}
/// `_KINDS`: (words, card).
static _KINDS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.pharmacy._KINDS").items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect()
});
static _NOT_A_SYMPTOM: LazyLock<Vec<String>> = LazyLock::new(|| strs("_NOT_A_SYMPTOM"));
static _OWN_BRAND_CARD: LazyLock<String> = LazyLock::new(|| s("_OWN_BRAND_CARD"));
static _PRESCRIBED_CARD: LazyLock<String> = LazyLock::new(|| s("_PRESCRIBED_CARD"));
static _PET_CARD: LazyLock<String> = LazyLock::new(|| s("_PET_CARD"));
static _MEDICINE_THINGS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_MEDICINE_THINGS"));
static _PETS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PETS"));
static _PRESCRIBED: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PRESCRIBED"));
static _IS_PREGNANT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_IS_PREGNANT"));
static _IS_A_CHILD: LazyLock<Vec<String>> = LazyLock::new(|| strs("_IS_A_CHILD"));
static _PREGNANT_CARD: LazyLock<String> = LazyLock::new(|| s("_PREGNANT_CARD"));
static _CHILD_CARD: LazyLock<String> = LazyLock::new(|| s("_CHILD_CARD"));
static _NOT_A_SHELF_PRICE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_NOT_A_SHELF_PRICE"));
static _NOT_A_PRESCRIPTION: LazyLock<Vec<String>> = LazyLock::new(|| strs("_NOT_A_PRESCRIPTION"));
static _PET_WORDS_FOR_AVOID: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PET_WORDS_FOR_AVOID"));

/// The line (text and clean) as one lower-cased string.
pub fn _words_of(reading: &Understood) -> String {
    let mut said = vec![reading.text.clone(), reading.clean.clone()];
    if let Some(leans) = &reading.leans_on {
        said.push(leans.text.clone());
        said.push(leans.subject.clone().unwrap_or_default());
        said.push(leans.topic.clone().unwrap_or_default());
    }
    text::squash(&said.join(" ")).to_lowercase()
}

/// The person's previous line, lower-cased.
pub fn _the_earlier(memory: Option<&Memory>) -> String {
    let earlier = memory.map(sh::last_user).unwrap_or_default();
    text::squash(&earlier).to_lowercase()
}

/// The counter card for the symptom the text names (None for no symptom, or a line that is not
/// about one).
pub fn _the_kind_in(said: &str) -> Option<String> {
    if said.is_empty() || sh::any_in(said, &_NOT_A_SYMPTOM) {
        return None;
    }
    _KINDS.iter().find(|(words, _)| sh::any_in(said, words)).map(|(_, card)| card.clone())
}

/// The card for the symptom in the line, or in the person's previous line.
pub fn _the_kind(reading: &Understood, memory: Option<&Memory>) -> Option<String> {
    _the_kind_in(&_words_of(reading)).or_else(|| _the_kind_in(&_the_earlier(memory)))
}

/// The card for someone the shelf treats differently: pregnancy, then a child.
pub fn _a_person_the_shelf_treats_differently(said: &str) -> Option<String> {
    if sh::any_in(said, &_IS_PREGNANT) {
        return Some(_PREGNANT_CARD.clone());
    }
    if sh::any_in(said, &_IS_A_CHILD) {
        return Some(_CHILD_CARD.clone());
    }
    None
}

/// What the counter sells for the symptom the line (or the line before) names; a pregnant
/// person or a child gets the card for them first.
pub fn _at_the_counter(memory: &Memory, reading: &Understood) -> Option<String> {
    let said = _words_of(reading);
    let earlier = _the_earlier(Some(memory));
    let both = format!("{said} {earlier}");
    if said.is_empty() || _names_any(&both, &_NOT_A_SYMPTOM) {
        return None;
    }
    if let Some(different) = _a_person_the_shelf_treats_differently(&both).filter(|d| !d.is_empty()) {
        return Some(different);
    }
    _the_kind_in(&said).or_else(|| _the_kind_in(&earlier))
}

/// The line as the guards read it.
pub fn _said(reading: &Understood) -> String {
    _words_of(reading)
}

/// `any(word in said for word in words)`.
pub fn _names_any<S: AsRef<str>>(said: &str, words: &[S]) -> bool {
    sh::any_in(said, words)
}

/// Own-brand medicines are the same as the branded ones.
pub fn _the_own_brand(_memory: &Memory, _reading: &Understood) -> String {
    _OWN_BRAND_CARD.clone()
}

/// Whether the line asks the price of a shelf medicine (not a prescription, the morning-after
/// pill, or a pet's).
pub fn _a_price_was_asked(understood: &Understood) -> bool {
    let said = _said(understood);
    !said.is_empty()
        && _names_any(&said, &_MEDICINE_THINGS)
        && !_names_any(&said, &_PRESCRIBED)
        && !_names_any(&said, &_NOT_A_SHELF_PRICE)
        && !_names_any(&said, &_PET_WORDS_FOR_AVOID)
}

/// Finish the prescribed course card, when the line (or the one before) names a prescription.
pub fn _the_prescribed_course(memory: &Memory, reading: &Understood) -> Option<String> {
    let said = format!("{} {}", _words_of(reading), _the_earlier(Some(memory)));
    if said.is_empty() || !_names_any(&said, &_PRESCRIBED) {
        return None;
    }
    if _names_any(&said, &_NOT_A_PRESCRIPTION) {
        return None;
    }
    Some(_PRESCRIBED_CARD.clone())
}

/// The vet's card, when the medicine is for a pet.
pub fn _a_pet_at_the_counter(memory: &Memory, reading: &Understood) -> Option<String> {
    let said = format!("{} {}", _words_of(reading), _the_earlier(Some(memory)));
    if text::strip(&said).is_empty() || !_names_any(&said, &_PETS) {
        return None;
    }
    Some(_PET_CARD.clone())
}

fn g_price(_f: &FnRef, u: &Understood) -> bool {
    _a_price_was_asked(u)
}
fn r_counter(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _at_the_counter(m, u)
}
fn r_own_brand(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_the_own_brand(m, u))
}
fn r_prescribed(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _the_prescribed_course(m, u)
}
fn r_pet(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _a_pet_at_the_counter(m, u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.pharmacy:_a_price_was_asked:339", g_price)];
pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.pharmacy:_at_the_counter:298", r_counter),
    ("bot.knowledge.pharmacy:_the_own_brand:326", r_own_brand),
    ("bot.knowledge.pharmacy:_the_prescribed_course:356", r_prescribed),
    ("bot.knowledge.pharmacy:_a_pet_at_the_counter:366", r_pet),
];
