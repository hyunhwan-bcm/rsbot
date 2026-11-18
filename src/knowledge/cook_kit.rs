//! Port of `bot/knowledge/cook_kit.py`: kitchen kit - how much water in the kettle, a pan or a
//! tin with burnt food on it, and a cracked, warped or worn-out piece of kit.
//!
//! The word lists and replies are the bundle's (the ones `cooking` owns are read under
//! `bot.knowledge.cook_kit.*`, where the module imported them); `_says` and `_words_and_text`
//! are cooking's own ports.

use crate::data::{self, Value};
use crate::knowledge::cooking_b::{_says, _words_and_text};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::{py_str_repr, Num, Understood};
use std::collections::HashSet;
use std::sync::LazyLock;

const P: &str = "bot.knowledge.cook_kit.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{P}{name}"))
}
fn s(name: &str) -> String {
    g(name).str().to_string()
}

macro_rules! strs {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<Vec<String>> = LazyLock::new(|| g(stringify!($n)).strs());)* };
}
macro_rules! res {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<&'static Re> = LazyLock::new(|| g(stringify!($n)).as_re().unwrap());)* };
}
strs!(_FOOD, _PAN_CLEAN, _NOT_A_PAN, _A_PAN, _PAN_STUCK, _PAN_SOMETHING, _PAN_BURNT, _BURNT_EATING_SAFETY, _PAN_DISH);
res!(_CUPS_SAID, _FULL_KETTLE_ASK, _BLANK_STUFF, _VESSEL_PHRASES);
static ML_PER_MUG: LazyLock<i64> = LazyLock::new(|| g("ML_PER_MUG").as_int().unwrap());

/// A millilitre figure as a person would say it: 600 ml, then 1.2 litres.
pub fn _ml(value: i64) -> String {
    if value >= 1000 {
        let litres = format!("{:.1}", value as f64 / 1000.0);
        return format!("{} litres", litres.trim_end_matches('0').trim_end_matches('.'));
    }
    format!("{value} ml")
}

/// (count, "cups" or "people") from the line itself, or (0, "cups").
pub fn _cups_said(text_: &str) -> (i64, &'static str) {
    let lowered = text::lower(text_);
    let Some(m) = _CUPS_SAID.search(&lowered) else { return (0, "cups") };
    let said = m.str(1);
    let count = said.parse::<i64>().unwrap_or_else(|_| g("_COUNT_WORDS").get(said).and_then(Value::as_int).unwrap_or(0));
    let unit = if ["people", "person", "of us"].iter().any(|p| m.str(2).starts_with(p)) { "people" } else { "cups" };
    (count.max(0), unit)
}

/// `clean or text`.
fn line(understood: &Understood) -> &str {
    if understood.clean.is_empty() { &understood.text } else { &understood.clean }
}

/// How much water to boil, and how long it takes, from the line.
pub fn _tea_water(_memory: &Memory, understood: &Understood) -> String {
    let (mut count, mut unit) = (0, "cups");
    for number in &understood.numbers {
        let said = number.unit.as_deref().unwrap_or("");
        // `int(value)`: a float is cut to its whole part, a missing value is skipped
        let value = match &number.value {
            Some(Num::Int(i)) => *i,
            Some(Num::Float(f)) if f.is_finite() => f.trunc() as i64,
            _ => continue,
        };
        if value < 1 {
            continue;
        }
        if ["cup", "cups", "mug", "mugs", "cuppa", "cuppas"].contains(&said) {
            (count, unit) = (value, "cups");
            break;
        }
        if ["people", "person", "of us", "guest", "guests"].contains(&said) {
            (count, unit) = (value, "people");
            break;
        }
    }
    if count == 0 {
        (count, unit) = _cups_said(line(understood));
    }
    let said = text::lower(line(understood));
    if count == 0 && _FULL_KETTLE_ASK.is_match(&said) {
        return s("_FULL_KETTLE");
    }
    if count == 0 {
        return "A mug of tea is about 300 ml, so count the mugs you are making and fill the kettle just past that: two cups is 600 ml, and a full 1.7-litre kettle is about five mugs. Boiling only what you will drink is the cheapest habit in the kitchen.".into();
    }
    let amount = _ml(count * *ML_PER_MUG);
    let what = if unit == "people" {
        format!("{count} people, a mug each")
    } else if count == 1 {
        "one cup".to_string()
    } else {
        format!("{count} cups")
    };
    format!(
        "About {amount} for {what}, since a mug holds roughly 300 ml and a splash more covers the element. Boiling only what you drink is the cheapest habit in the kitchen."
    )
}

/// The soak, said for the thing that burnt on (a tin, a tray or a shelf), or "" for a line
/// whose food is what burnt.
pub fn _burnt_tin(_memory: &Memory, reading: &Understood) -> String {
    let said = _words_and_text(reading);
    let words: HashSet<String> = reading.words.iter().map(|w| text::lower(w)).collect();
    let baked = _BLANK_STUFF.sub(" ", &_VESSEL_PHRASES.sub(" ", &said));
    let mut cleaned: Vec<String> = _PAN_CLEAN.clone();
    cleaned.extend(["off my tray", "off the tray", "off my tin", "off the tin", "get the burnt off", "off the sheet"].map(String::from));
    if _says(&baked, &_FOOD) && !_says(&said, &cleaned) {
        return String::new();
    }
    let any = |list: &[&str]| list.iter().any(|w| words.contains(*w));
    if any(&["coating", "coatings", "non", "nonstick", "teflon", "scratch", "scratched", "scratches", "scouring", "scour", "scourer", "wool", "scraper"]) {
        return "Bicarbonate and a wooden spoon will not lift a non-stick coating - it is wire wool and metal scourers that do. If the coating is already scratched through to the metal, that tin is done for anything that sticks, so line it with baking paper and keep it for bakes that turn out of the tin anyway.".into();
    }
    if any(&["shelf", "shelves", "rack", "racks"]) {
        return "Soak it in hot water and two tablespoons of bicarbonate of soda for half an hour - the bath if the shelf will not fit in the sink - then lift the burnt patches off with a plastic scourer or a ball of foil. Anything really black takes a paste of bicarbonate and water left on overnight. Dry it before it goes back in, and put a sheet of foil on the shelf under the tin next time, which is what catches the drips.".into();
    }
    if any(&["tray", "trays", "sheet", "sheets"]) {
        return "Soak it in the sink with hot water and two tablespoons of bicarbonate of soda for half an hour, then lift the burnt patches off with a wooden spoon or a plastic scourer. Anything really black takes a paste of bicarbonate and water left on overnight. Dry it properly before it goes away, and line it with baking paper next time, which is what stops it coming back.".into();
    }
    "Soak it in the sink with hot water and two tablespoons of bicarbonate of soda for half an hour, then lift the burnt patches off with a wooden spoon or a plastic scourer: tins are thin and scouring takes the non-stick off for good. Anything really black takes a paste of bicarbonate and water left on overnight. Dry it properly before it goes away, and line it with baking paper next time, which is what stops it coming back.".into()
}

/// `str(value or "")` of `norm`: Python's repr of the word list, or "" for an empty one.
fn norm_said(understood: &Understood) -> String {
    if understood.norm.is_empty() {
        return String::new();
    }
    format!("[{}]", understood.norm.iter().map(|w| py_str_repr(w)).collect::<Vec<_>>().join(", "))
}

/// The pan's answer for a pan line, and nothing for a line whose dish is what burnt.
pub fn _burnt_pan(_memory: &Memory, understood: &Understood) -> String {
    let said = text::lower(&[understood.clean.clone(), understood.norm_text.clone(), norm_said(understood)].join(" "));
    if _says(&said, &_BURNT_EATING_SAFETY) {
        return s("_BURNT_EATING_REPLY");
    }
    if _says(&said, &_NOT_A_PAN) && !_says(&said, &_A_PAN) {
        return String::new();
    }
    if _says(&said, &_PAN_STUCK) && !_says(&said, &_PAN_BURNT) && !_says(&said, &_PAN_SOMETHING) {
        return String::new();
    }
    if _says(&said, &_PAN_CLEAN) {
        return s("_BURNT_PAN_REPLY");
    }
    if _says(&said, &_PAN_DISH) {
        return String::new(); // a dish is burnt, not the pan: fix-burnt has that line
    }
    s("_BURNT_PAN_REPLY")
}

/// A cracked, warped or worn-out piece of kitchen kit: whether it can be used, fixed or binned.
/// The answer follows the material.
pub fn _kit_damaged(_memory: &Memory, understood: &Understood) -> String {
    let said = format!(" {} ", [&understood.text, &understood.norm_text, &understood.clean].map(|t| text::lower(t)).join(" "));
    let has = |phrases: &[&str]| phrases.iter().any(|p| said.contains(&format!(" {p}")));
    if has(&["microwave plate", "turntable", "glass plate", "plate in the microwave"]) {
        return "Replace the part rather than the machine: a microwave plate or turntable ring is about \u{a3}10 to \u{a3}20 and sold by the model number on the sticker inside the door, or stamped on the plate itself. A cracked plate can shatter the next time it heats, so keep it out of the microwave until the new one comes.".into();
    }
    if has(&["jug", "blender", "carafe", "glass lid", "lid"]) {
        return "A cracked jug, blender jug or glass lid goes: it can let go all at once with something hot in it, and a crack you cannot scrub out of is not a food surface. Blenders and kettles take a spare jug or lid by model number, and a jam jar is a free substitute while you wait.".into();
    }
    if has(&["board", "chopping board", "cutting board", "wooden board"]) {
        return "Sand it back and oil it if the split is shallow, because that closes the grain again; a crack that goes right through traps food where you cannot wash it out, and that board should go. Never soak a wooden board or put it in the dishwasher - wash it, stand it on edge to dry and oil it once a month.".into();
    }
    if has(&["enamel", "enamelled", "chipped", "chipping"]) {
        return "A chip on the inside of an enamelled pan will rust and flake into the food, so that one is finished; a chip on the outside only matters for looks. Enamel repair paints are not food safe, so do not touch one in, and do not put an enamelled pan in the dishwasher, which is what chips the rim.".into();
    }
    if has(&["coating", "non stick", "non-stick", "teflon", "flaking", "peeling"]) {
        return "Once the coating is flaking the pan is done: the flakes come off into the food and the bare metal scorches what it touches. Re-coating is not a home job, so replace it - and washing it by hand, never heating it empty and never scouring it are what make the next one last three to five years.".into();
    }
    if has(&[
        "warped", "warping", "buckled", "buckling", "dented", "bent", "wobbles", "wobbling", "rocks", "rocking", "spins", "doesnt sit flat", "doesn't sit flat",
        "wont sit flat", "won't sit flat", "not flat", "bowed", "dished", "domed", "bulging", "bulged",
    ]) {
        if has(&["tin", "tins", "tray", "trays", "sheet", "springform", "mould", "muffin"]) {
            return "Stand it on the worktop and press it flat again with the heel of your hand: a baking tin that has bent in the wash still bakes the same, and the only thing you notice is the wobble. Set it on a flat baking sheet if the batter runs to one side. Replace it when the springform clip or the base no longer seats flat, because then it leaks.".into();
        }
        return "A warped base only really matters on a flat hob: on gas the pan still heats, but on induction, ceramic or solid plate it will not sit flat and the heat goes patchy. Check it against a straight edge on the worktop. It cannot be flat again for certain, so a heavy-based pan is the replacement - thin ones go in one hot empty minute or a cold rinse in a hot pan.".into();
    }
    if has(&["kettle"]) {
        return "A cracked kettle goes: water reaches the element and the connections, which is a shock risk rather than a leak to mop up. Unplug it and replace it - a new kettle is \u{a3}15 to \u{a3}30, and descaling the next one monthly is what keeps it from running hot.".into();
    }
    if has(&["slow cooker", "casserole", "stoneware", "ceramic", "pot", "pots", "dish", "dish", "baking dish", "oven dish"]) {
        return "Retire it if the crack is in the pot itself: stoneware fails right through when it heats, and a crack holds food you cannot wash out of it. Slow cooker inserts and casserole dishes are sold on their own by model number or size, \u{a3}10 to \u{a3}25, and a lidded oven dish that fits inside will do until the new one comes.".into();
    }
    "If the crack is right through, replace it: a split pot, dish or plate holds food and bacteria where you cannot wash, and it can fail completely the next time it heats. A chip or crack on an outside face that never touches food can usually be lived with.".into()
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.cook_kit:_tea_water:85", |_, m, u| Some(_tea_water(m, u))),
    ("bot.knowledge.cook_kit:_burnt_tin:134", |_, m, u| Some(_burnt_tin(m, u))),
    ("bot.knowledge.cook_kit:_burnt_pan:243", |_, m, u| Some(_burnt_pan(m, u))),
    ("bot.knowledge.cook_kit:_kit_damaged:297", |_, m, u| Some(_kit_damaged(m, u))),
];
