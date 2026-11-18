//! Port of PyBot's `bot/knowledge/travel_health.py`: travel jabs and medicines for a trip.

use crate::data::{self, FnRef};
use crate::knowledge::d1_dup_g7::pct;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

fn rx(name: &str) -> &'static Re {
    data::re(&format!("bot.knowledge.travel_health.{name}"))
}
fn s(name: &str) -> String {
    data::string(&format!("bot.knowledge.travel_health.{name}"))
}
fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("bot.knowledge.travel_health.{name}"))
}
static _ROUTINE_PLACE: LazyLock<&'static Re> = LazyLock::new(|| rx("_ROUTINE_PLACE"));
static _AWAY: LazyLock<&'static Re> = LazyLock::new(|| rx("_AWAY"));
static _PLACE: LazyLock<&'static Re> = LazyLock::new(|| rx("_PLACE"));
static _ANIMAL: LazyLock<&'static Re> = LazyLock::new(|| rx("_ANIMAL"));
static _NOT_A_TRIP: LazyLock<&'static Re> = LazyLock::new(|| rx("_NOT_A_TRIP"));
static _A_BARE_NEED: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_BARE_NEED"));
static _ROUTINE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ROUTINE"));
static _YELLOW_FEVER: LazyLock<Vec<String>> = LazyLock::new(|| strs("_YELLOW_FEVER"));
static _MALARIA: LazyLock<Vec<String>> = LazyLock::new(|| strs("_MALARIA"));
static _A_TRAVEL_CLINIC: LazyLock<String> = LazyLock::new(|| s("_A_TRAVEL_CLINIC"));
static _THE_REST_OF_THE_LIST: LazyLock<String> = LazyLock::new(|| s("_THE_REST_OF_THE_LIST"));
static _CHECK_IT: LazyLock<String> = LazyLock::new(|| s("_CHECK_IT"));
static _WHEN_TO_BOOK_BARE: LazyLock<String> = LazyLock::new(|| s("_WHEN_TO_BOOK_BARE"));
static _OTHER_ONES: LazyLock<String> = LazyLock::new(|| s("_OTHER_ONES"));
static _YELLOW_SENTENCE: LazyLock<String> = LazyLock::new(|| s("_YELLOW_SENTENCE"));
static _MALARIA_SENTENCE: LazyLock<String> = LazyLock::new(|| s("_MALARIA_SENTENCE"));
static _MALARIA_ASKED: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"(?<![a-z])(?:malaria|antimalarial|antimalarials)(?![a-z])", re::I));
static _YELLOW_ASKED: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<![a-z])(?:yellow fever|certificate)(?![a-z])", re::I));

/// Whether the line is about a trip (a place, going abroad, or a bare "do I need jabs"), not a
/// pet's jabs.
pub fn _a_trip_is_named(understood: &Understood) -> bool {
    let text = &understood.clean;
    if _ANIMAL.is_match(text) {
        return false;
    }
    if _PLACE.is_match(text) || _AWAY.is_match(text) {
        return true;
    }
    _A_BARE_NEED.is_match(text) && !_NOT_A_TRIP.is_match(text)
}

/// The place the line names, lower-cased (a routine place first).
pub fn _place_named(text: &str) -> Option<String> {
    for pattern in [*_ROUTINE_PLACE, *_PLACE] {
        if let Some(there) = pattern.search(text) {
            return Some(text::strip(there.str(1)).to_lowercase());
        }
    }
    None
}

/// The jabs card for the place the line names: routine places need nothing more; malaria and
/// yellow fever places get their sentences, the one asked about first.
pub fn _the_jabs_reply(_memory: &Memory, understood: &Understood) -> String {
    let text = &understood.clean;
    let Some(place) = _place_named(text) else {
        return format!("{} {}", *_A_TRAVEL_CLINIC, *_THE_REST_OF_THE_LIST);
    };
    let name = text::title(&place);
    let n = name.as_str();
    if _ROUTINE.contains(&place) {
        return pct("Nothing beyond the routine jabs for %s - no travel clinic needed unless you are staying somewhere unusual or going on somewhere else. Make sure the routine ones are up to date (MMR, tetanus, polio), and check %s on the NHS Fit for Travel page if you want it in writing.", &[n, n]);
    }
    let malaria = _MALARIA.contains(&place);
    let yellow = _YELLOW_FEVER.contains(&place);
    let asks_malaria = malaria && _MALARIA_ASKED.is_match(text);
    let asks_yellow = yellow && _YELLOW_ASKED.is_match(text);
    let mut said: Vec<String> = Vec::new();
    if asks_malaria {
        said.push(pct("Malaria tablets are the decision for %s, and it is the area and the season that decide them, so take the clinic's advice for exactly where you are staying.", &[n]));
        said.push(_WHEN_TO_BOOK_BARE.clone());
        said.push(pct(&_OTHER_ONES, &[n]));
        if yellow {
            said.push(pct(&_YELLOW_SENTENCE, &[n]));
        }
    } else if asks_yellow {
        said.push(pct("%s is in the yellow fever area: the vaccine is for the areas with risk, and the certificate is what a border asks for when you arrive from one, so the clinic needs your route and not only the country.", &[n]));
        said.push(_WHEN_TO_BOOK_BARE.clone());
        said.push(pct(&_OTHER_ONES, &[n]));
        if malaria {
            said.push(pct(&_MALARIA_SENTENCE, &[n]));
        }
    } else {
        said.push(pct("Plan on hepatitis A and typhoid for %s, and book a travel clinic or a pharmacy that does travel jabs four to six weeks before you fly: some courses are two or three doses a month apart, and most of them take a fortnight to work.", &[n]));
        if yellow {
            said.push(pct(&_YELLOW_SENTENCE, &[n]));
        }
        if malaria {
            said.push(pct(&_MALARIA_SENTENCE, &[n]));
        }
    }
    said.push(pct(&_CHECK_IT, &[n]));
    said.join(" ")
}

fn g_trip(_f: &FnRef, u: &Understood) -> bool {
    _a_trip_is_named(u)
}
fn r_jabs(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_the_jabs_reply(m, u))
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.travel_health:_a_trip_is_named:168", g_trip)];
pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.travel_health:_the_jabs_reply:235", r_jabs)];
