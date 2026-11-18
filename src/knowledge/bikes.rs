//! Port of `bot/knowledge/bikes.py`: the bicycle - punctures, chains, brakes, an e-bike's
//! battery, what to wear and how the bars and the saddle are set. The rows are the bundle's;
//! this file holds the readers they call.

use crate::data::{self, Value};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

const P: &str = "bot.knowledge.bikes.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{P}{name}"))
}
fn s(name: &str) -> String {
    g(name).str().to_string()
}
fn strs(name: &str) -> Vec<String> {
    g(name).strs()
}
fn re(name: &str) -> &'static Re {
    g(name).as_re().unwrap_or_else(|| panic!("{P}{name} is not a regex"))
}

static _PRICE_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PRICE_ASK"));
static _RANGE_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_RANGE_ASK"));
static _LIFE_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_LIFE_ASK"));
static _IS_A_BIKE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_IS_A_BIKE"));
static _BAR_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| g("_BAR_WORDS").str_set());
static _SEAT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| g("_SEAT_WORDS").str_set());

/// The problem's own words, as the key part. The bicycle itself is the entry's `ask`.
pub fn _bike(needs: &[String]) -> Vec<String> {
    needs.to_vec()
}

fn any_in(said: &str, words: &[String]) -> bool {
    words.iter().any(|w| said.contains(w.as_str()))
}

/// The pack answer the line asked for: a price, a range, a life, or what to do about a fault.
pub fn _bike_battery(_memory: &Memory, understood: &Understood) -> String {
    let clean = text::lower(&understood.clean);
    let (price, range, life, care) = (s("_BIKE_PACK_PRICE"), s("_BIKE_PACK_RANGE"), s("_BIKE_PACK_LIFE"), s("_BIKE_PACK_CARE"));
    if any_in(&clean, &_PRICE_ASK) {
        return [price, range, care].join(" ");
    }
    if any_in(&clean, &_RANGE_ASK) {
        return [range, life, price].join(" ");
    }
    if any_in(&clean, &_LIFE_ASK) {
        return [life, price, care].join(" ");
    }
    s("_BIKE_PACK_FAULT")
}

/// The same answers for a follow-up, only while the chat is still about a bicycle.
pub fn _bike_battery_earlier(memory: &Memory, understood: &Understood) -> String {
    let mut said = vec![understood.clean.clone()];
    said.extend(memory.turns.iter().map(|t| t.user.clone()));
    let said = text::lower(&said.join(" "));
    if !any_in(&said, &_IS_A_BIKE) {
        return String::new();
    }
    _bike_battery(memory, understood)
}

/// The clothes for the day: the same answer, with the hot one when the line names one.
pub fn _bike_wear(_memory: &Memory, understood: &Understood) -> String {
    if re("_HOT_WEATHER_RE").is_match(&text::lower(&understood.clean)) {
        return s("_BIKE_WEAR_HOT");
    }
    s("_BIKE_WEAR_ANY")
}

/// Which bars to get, how to move the bars you have, or the saddle first.
pub fn _bike_fit(_memory: &Memory, understood: &Understood) -> String {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let mut said = text::lower(line);
    if said.is_empty() {
        said = text::lower(&understood.words.join(" "));
    }
    let words: HashSet<String> = understood.words.iter().map(|w| text::lower(w)).collect();
    let names_bars = words.iter().any(|w| _BAR_WORDS.contains(w)) || re("_A_BAR_NAMED").is_match(&said);
    let names_seat = words.iter().any(|w| _SEAT_WORDS.contains(w));
    if names_bars && !names_seat {
        if re("_BAR_MOVE").is_match(&said) && !re("_BAR_CHOICE").is_match(&said) {
            return s("MOVING_BARS");
        }
        return s("WHICH_BARS");
    }
    s("SADDLE_FIRST")
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.bikes:_bike_battery:147", |_, m, u| Some(_bike_battery(m, u))),
    ("bot.knowledge.bikes:_bike_battery_earlier:159", |_, m, u| Some(_bike_battery_earlier(m, u))),
    ("bot.knowledge.bikes:_bike_wear:200", |_, m, u| Some(_bike_wear(m, u))),
    ("bot.knowledge.bikes:_bike_fit:308", |_, m, u| Some(_bike_fit(m, u))),
];
