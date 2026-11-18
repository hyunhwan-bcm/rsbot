//! Port of PyBot's `bot/knowledge/chilli.py`: chillies (keeping, freezing, heat) and a pot of
//! chilli, and how hot each curry on the menu is.

use crate::data::{self, FnRef};
use crate::knowledge::d1_dup_g7 as dup;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

fn key(name: &str) -> String {
    format!("bot.knowledge.chilli.{name}")
}
static _CHILLI_IN_THE_LINE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&key("_CHILLI_IN_THE_LINE")));
/// `CURRY_HEAT`: (names, place on the heat scale).
static CURRY_HEAT: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get(&key("CURRY_HEAT")).items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect()
});
static CURRY_ORDER_LINE: LazyLock<String> = LazyLock::new(|| data::string(&key("CURRY_ORDER_LINE")));
static _A_FAULT_IN_THE_DISH: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&key("_A_FAULT_IN_THE_DISH")));
static _SHORTAGE: LazyLock<&'static Re> = LazyLock::new(|| data::re(&key("_SHORTAGE")));
static _BEAN_STEMS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&key("_BEAN_STEMS")));
static A_COOK_CLOCK_VERB: LazyLock<&'static Re> = LazyLock::new(|| data::re(&key("A_COOK_CLOCK_VERB")));
static A_KEEPING_CUE: LazyLock<&'static Re> = LazyLock::new(|| data::re(&key("A_KEEPING_CUE")));
static A_POT_OF_CHILLI_KEEPS: LazyLock<String> = LazyLock::new(|| data::string(&key("A_POT_OF_CHILLI_KEEPS")));
static NOT_PLAIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' ]+", 0));

/// How the chilli row answers: the vegetable's own answer when the line names a chilli; the pot
/// of chilli's answer when a dish is in the chat and it has one.
pub fn _the_vegetables_or_the_pot(
    memory: &Memory,
    understood: &Understood,
    vegetable_answer: &str,
    pot_answer: fn(&Memory, &Understood) -> Option<String>,
) -> String {
    if _a_chilli_in_this_line(memory, understood) {
        return vegetable_answer.to_string();
    }
    if _a_dish_in_the_chat(memory, understood) {
        if let Some(pot) = pot_answer(memory, understood).filter(|p| !crate::py::text::strip(p).is_empty()) {
            return pot;
        }
    }
    vegetable_answer.to_string()
}

/// Whether a dish (a pot of chilli) is in the line or the chat so far.
pub fn _a_dish_in_the_chat(memory: &Memory, understood: &Understood) -> bool {
    crate::runtime::guarded(|| crate::knowledge::dishes::dish_for(memory, understood).is_some_and(|d| !d.is_empty()))
        .unwrap_or(false)
}

/// The dish plan's keeping answer.
pub fn _the_plans_keeping_answer(memory: &Memory, understood: &Understood) -> Option<String> {
    crate::knowledge::dishes::follow_up(memory, understood)
}

/// The cooked dish's freezing answer.
pub fn _the_cooked_dish_freezing_answer(memory: &Memory, understood: &Understood) -> Option<String> {
    Some(dup::freeze_cooked_answer(memory, understood))
}

/// Whether the line as typed names a chilli (the vegetable).
pub fn _a_chilli_in_this_line(memory: &Memory, understood: &Understood) -> bool {
    let text = dup::said_in_this_line(understood, Some(memory)).to_lowercase();
    sh::any_in(&text, &_CHILLI_IN_THE_LINE)
}

/// Keeping chillies, or a pot of chilli when that is what the chat is about.
pub fn lambda_218(memory: &Memory, understood: &Understood) -> String {
    _the_vegetables_or_the_pot(
        memory,
        understood,
        "Loose in the fridge drawer they keep a week or two, and longer in a paper bag than in plastic: the bag holds the moisture that turns them soft. A glut is better frozen or dried than left to go wrinkly - both keep the heat - and a handful also becomes chilli oil, which is the fastest way to use up twenty at once.",
        _the_plans_keeping_answer,
    )
}

/// Freezing chillies, or a cooked dish when that is what the chat is about.
pub fn lambda_311(memory: &Memory, understood: &Understood) -> String {
    _the_vegetables_or_the_pot(
        memory,
        understood,
        "Freeze them whole in a bag and slice them still frozen: they go soft, so they are for cooking rather than for a salad, but the heat survives a few months. Chopped into an ice-cube tray with a little oil is the other way, one cube per curry, and there is no need to blanch them first.",
        _the_cooked_dish_freezing_answer,
    )
}

/// How hot the curry the line names is, on the usual menu's scale (None for a dish that came
/// out too hot).
pub fn _curry_heat(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" ");
    let flat = format!(" {} ", sh::clean_or_text(understood).to_lowercase());
    let flat = NOT_PLAIN.sub(" ", &flat);
    if sh::any_in(&flat, &_A_FAULT_IN_THE_DISH) {
        return None;
    }
    for (names, place) in CURRY_HEAT.iter() {
        for name in names {
            if said.contains(name.as_str()) {
                return Some(format!(
                    "A {name} is {place}. Mildest to hottest the usual menu runs {}, so ask for it mild and the kitchen leaves the chilli out - and if you want heat with flavour, step one up the list rather than two.",
                    *CURRY_ORDER_LINE
                ));
            }
        }
    }
    Some(format!(
        "Mildest to hottest the usual takeaway menu runs {}. A korma or pasanda has no chilli heat at all, a tikka masala is mild, and a jalfrezi and up are the hot end; ask for any of them mild and the kitchen makes it without the chilli.",
        *CURRY_ORDER_LINE
    ))
}

/// False when the line is short of something other than beans ("no tomatoes"): the chilli
/// without beans row is for beans.
pub fn _short_of_something_else(understood: &Understood) -> bool {
    for m in _SHORTAGE.finditer(&sh::said_text(understood)) {
        let near: Vec<&str> = crate::py::text::split_ref(m.str(1)).into_iter().take(3).collect();
        if !near.iter().any(|w| _BEAN_STEMS.iter().any(|s| w.starts_with(s.as_str()))) {
            return false;
        }
    }
    true
}

/// How long a pot of chilli keeps ("" on a cooking-time line).
pub fn _the_pot_keeps(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::clean_or_text(understood).to_lowercase();
    if A_COOK_CLOCK_VERB.is_match(&said) && !A_KEEPING_CUE.is_match(&said) {
        return String::new();
    }
    A_POT_OF_CHILLI_KEEPS.clone()
}

fn g_short(_f: &FnRef, u: &Understood) -> bool {
    _short_of_something_else(u)
}
fn r_218(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(lambda_218(m, u))
}
fn r_311(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(lambda_311(m, u))
}
fn r_curry(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _curry_heat(m, u)
}
fn r_pot(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_the_pot_keeps(m, u))
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.chilli:_short_of_something_else:170", g_short)];
pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.chilli:<lambda>:218", r_218),
    ("bot.knowledge.chilli:<lambda>:311", r_311),
    ("bot.knowledge.chilli:_curry_heat:139", r_curry),
    ("bot.knowledge.chilli:_the_pot_keeps:201", r_pot),
];
