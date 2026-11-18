//! Port of `bot/knowledge/cooking.py` (functions defined at lines 1-5999 of the original; the
//! rest of the file is `cooking_b.rs`). Cooking and food: everyday how-tos, timings, fixes and
//! food safety. The entries and every table are data from the bundle; this is the logic their
//! guards and replies run.

#![allow(non_snake_case)]

use crate::data::{self, FnRef, Value};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::{Num, Understood};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

// The cooking functions defined from line 6000 on are in cooking_b.rs; re-exported here, so
// `crate::knowledge::cooking::X` reaches every function of cooking.py.
pub use super::cooking_b::*;
// Helpers of other modules, until their owners' ports are merged.
use super::d2dup_cooka::{
    dishes_ask_kind, dishes_asks_for_a_side_of_a_food, dishes_dish_in, dishes_general_sides,
    dishes_names_an_appliance_and_a_cut, dishes_plan, dishes_recent_texts, dishes_reply, garden_done_not,
    garden_harvest_crops, lookup_find_id, people_in_groups,
};
use super::prices::a_price_is_judged;

// ---------------------------------------------------------------------------------------------
// Data access: every table is the bundle's, by its Python name.

const MODULE: &str = "bot.knowledge.cooking.";

pub(crate) fn g(name: &str) -> &'static Value {
    data::get(&format!("{MODULE}{name}"))
}
fn strs(name: &str) -> Vec<String> {
    g(name).strs()
}
fn set(name: &str) -> HashSet<String> {
    g(name).str_set()
}
fn rx(name: &str) -> &'static Re {
    g(name).as_re().unwrap_or_else(|| panic!("{name} is not a regex"))
}
fn st(name: &str) -> String {
    g(name).str().to_string()
}

macro_rules! lazy {
    ($name:ident: Vec<String> = $py:expr) => {
        static $name: LazyLock<Vec<String>> = LazyLock::new(|| strs($py));
    };
    ($name:ident: HashSet<String> = $py:expr) => {
        static $name: LazyLock<HashSet<String>> = LazyLock::new(|| set($py));
    };
    ($name:ident: Re = $py:expr) => {
        static $name: LazyLock<&'static Re> = LazyLock::new(|| rx($py));
    };
    ($name:ident: String = $py:expr) => {
        static $name: LazyLock<String> = LazyLock::new(|| st($py));
    };
}

/// An inline pattern, compiled once.
macro_rules! re_lazy {
    ($name:ident = $pat:expr) => {
        static $name: LazyLock<Re> = LazyLock::new(|| Re::new($pat, 0));
    };
    ($name:ident = $pat:expr, $flags:expr) => {
        static $name: LazyLock<Re> = LazyLock::new(|| Re::new($pat, $flags));
    };
}

// ---------------------------------------------------------------------------------------------
// Python helpers

/// `any(phrase in text for phrase in phrases)`.
fn any_in<S: AsRef<str>>(text: &str, phrases: &[S]) -> bool {
    phrases.iter().any(|p| text.contains(p.as_ref()))
}

/// Python `a or b` on strings.
fn or<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.is_empty() { b } else { a }
}

/// The words of the reading, lower-cased (`[str(w).lower() for w in words]`).
fn lower_words(u: &Understood) -> Vec<String> {
    u.words.iter().map(|w| w.to_lowercase()).collect()
}

/// `re.search(r"\b" + re.escape(name) + r"\b", text)`.
fn word_in(text: &str, name: &str) -> bool {
    Re::new(&format!("\\b{}\\b", crate::py::re::escape(name)), 0).is_match(text)
}

/// Python's `format(x, "g")`: six significant digits, trailing zeros dropped.
pub(crate) fn py_g(x: f64) -> String {
    if x == 0.0 {
        return "0".into();
    }
    if !x.is_finite() {
        return text::float_repr(x);
    }
    let sci = format!("{:.5e}", x);
    let (mant, exp) = sci.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    if (-4..6).contains(&exp) {
        let decimals = (5 - exp).max(0) as usize;
        let s = format!("{:.*}", decimals, x);
        let s = if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s };
        s
    } else {
        let m = if mant.contains('.') { mant.trim_end_matches('0').trim_end_matches('.') } else { mant };
        format!("{}e{}{:02}", m, if exp < 0 { '-' } else { '+' }, exp.abs())
    }
}

/// A Python number as f64, and whether it was an int.
fn num(v: &Option<Num>) -> Option<f64> {
    v.as_ref().map(Num::as_f64)
}

/// Python's `round(x)` for an int-or-float value.
fn py_round(v: &Num) -> i64 {
    match v {
        Num::Int(i) => *i,
        Num::Float(f) => text::round_half_even(*f),
    }
}

// ---------------------------------------------------------------------------------------------
// Keeping a jar, a lid, the small baked things

lazy!(CONTAINER_FOODS: Vec<String> = "CONTAINER_FOODS");
lazy!(CONTAINER_MONTHS: Vec<String> = "CONTAINER_MONTHS");

/// The keeping sentence for the container named a turn ago, or "" when none was named.
pub fn _container_keep(memory: &mut Memory, _reading: &Understood) -> Option<String> {
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(2);
    for turn in turns[start..].iter().rev() {
        let said = turn.user.to_lowercase();
        let named: Vec<&String> = CONTAINER_FOODS.iter().filter(|w| said.contains(w.as_str())).collect();
        if let Some(first) = named.first() {
            let food = text::capitalize(first);
            if CONTAINER_MONTHS.contains(first) {
                let keep = if food.ends_with('s') { "keep" } else { "keeps" };
                return Some(format!(
                    "{food} {keep} for months in the fridge once it is opened, and an opened sauce or pesto 1 to 2 weeks. The label's once-opened line beats any general rule, so bin it if it smells sharp or the lid bulges."
                ));
            }
            return Some(format!(
                "An opened {first} is best within 1 to 2 weeks, kept in the fridge, and the label's once-opened line beats any general rule. Bin it if it smells sharp, looks odd or the lid bulges."
            ));
        }
    }
    Some(String::new())
}

/// The lid answer, with the keeping sentence when the jar holds something worth keeping.
pub fn _stuck_jar(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let line = or(&reading.text, &reading.clean).to_lowercase();
    let mut said = String::from(
        "Tap the edge of the lid all the way round with the handle of a knife, then grip it through a rubber glove, a silicone mat or a wide rubber band - the grip is usually what is missing. If it still will not turn, stand the jar in a bowl of hot water for two minutes so the metal lid swells, or slide the back of a spoon under the rim to let air in.",
    );
    if any_in(&line, &CONTAINER_FOODS) {
        said.push_str(" Once open, jam, pickles and chutney keep for months, an opened sauce 1 to 2 weeks.");
    }
    Some(said)
}

/// `_baked_keep.<locals>.reply`: the keeping answer for the small baked thing the line names.
pub fn _baked_keep_reply(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let line = or(&reading.text, &reading.clean).to_lowercase();
    let goods = g("BAKED_GOODS");
    let mut names: Vec<&str> = goods.pairs().iter().filter_map(|(k, _)| k.as_str()).collect();
    names.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()));
    for name in names {
        if line.contains(name) {
            let row = goods.get(name).unwrap().strs();
            return Some(format!("{} keep {}. {} {}", row[0], row[1], row[2], row[3]));
        }
    }
    Some(String::new())
}

// ---------------------------------------------------------------------------------------------
// Staples

/// Sweetcorn, leading with the way the line asks for it.
pub fn _corn(understood: &Understood) -> String {
    let said = format!(" {} ", lower_words(understood).join(" "));
    let has = |cues: &[&str]| cues.iter().any(|c| said.contains(c));
    if has(&[" microwave ", " microwaved ", " microwaving ", " microwaves ", " micro "]) {
        return "In the microwave a cob takes 3 to 4 minutes on full power for one and 5 to 6 for two: leave it in its husk, or wrap it in a damp piece of kitchen paper, and turn it once. It carries on cooking as it stands, so give it a minute before you butter it. Loose kernels in a covered bowl with a splash of water take 2 to 3 minutes, stirred halfway.".into();
    }
    if has(&[" air fryer ", " airfryer ", " air fried ", " airfry "]) {
        return "In the air fryer rub the cobs with a little oil and give them 10 to 12 minutes at 190C, turning once: the kernels blister at the tips and that is the point. Two cobs fit in a normal basket; more than that and they steam instead of browning.".into();
    }
    if has(&[" grill ", " grilled ", " grilling ", " barbecue ", " bbq ", " char ", " charred "]) {
        return "Under a hot grill or on the barbecue, 10 to 12 minutes, turning every few minutes until the kernels start to char in patches - rub them with a little butter or oil first so they colour rather than dry. Boiled instead, 6 to 8 minutes in unsalted water; salt toughens it.".into();
    }
    "Boil it in unsalted water for 6 to 8 minutes - salt toughens it - or grill it, turning it now and then, for 10 to 12 minutes until the kernels start to char. In the microwave a cob takes 3 to 4 minutes on full power. Butter, salt and pepper, and it needs nothing else.".into()
}

/// The general "learn to cook" card, unless the line names a food (its own row answers that).
pub fn _learn_to_cook(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let foods = _foods_said();
    if understood.words.iter().any(|w| foods.contains(&_fold(&w.to_lowercase()))) {
        return None;
    }
    Some("Learn four things and the rest follows: pasta with a sauce, an omelette, a tray of roasted vegetables, and a one-pot curry or stew. Cook the same one twice in a week rather than a new one each night, and taste as you go - food that tastes flat usually wants salt, a squeeze of lemon, or more time in the pan. A cheap thermometer takes the fear out of chicken and pork.".into())
}

lazy!(_REST_BODY: Vec<String> = "_REST_BODY");
lazy!(_REST_CUT: Vec<String> = "_REST_CUT");
lazy!(_REST_CUT_PHRASE: Vec<String> = "_REST_CUT_PHRASE");

/// True when the rest in the line is a body part's rest, not the meat's.
pub fn _resting_a_body_part(said: &str) -> bool {
    let body = _REST_BODY.iter().any(|w| said.contains(&format!(" {w} ")));
    let cut = _REST_CUT.iter().any(|w| said.contains(&format!(" {w} ")));
    if body && !any_in(said, &_REST_CUT_PHRASE) {
        return true;
    }
    body && !cut
}

/// Why and how long to rest the meat.
pub fn _rest_the_meat(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = format!(" {} ", understood.words.join(" "));
    if said.contains(" the rest ") || said.contains(" rest of ") {
        return None;
    }
    if _resting_a_body_part(&said) {
        return None;
    }
    if dishes_ask_kind(understood).as_deref() == Some("rest")
        && dishes_reply("rest", memory, understood).is_some_and(|r| !r.is_empty())
    {
        let next_row = lookup_find_id(understood, &["rest-meat"]);
        if next_row.as_deref() == Some("plan-followup") {
            return None;
        }
    }
    let minutes = "loosely under foil and off the heat, not back in the oven: 15 to 20 minutes for a chicken or a turkey, 10 to 20 for a joint of beef, lamb or pork, 5 to 10 for a steak or chops, and 20 to 30 for a brisket";
    let reason = "standing lets the heat that has built up round the outside move back into the middle, so the juice stays in the meat when you carve it instead of running out onto the board";
    if understood.asks.iter().any(|a| a == "why") {
        return Some(format!(
            "Rest it because {reason} - {minutes}. It stays hot that long under the foil, so nothing is lost by waiting."
        ));
    }
    Some(format!(
        "{}. {}. It stays hot that long under the foil, so nothing is lost by waiting.",
        text::upper_first(minutes),
        text::upper_first(reason)
    ))
}

lazy!(_TOO_THICK_SAID: Vec<String> = "_TOO_THICK_SAID");
lazy!(_BODY_TOO_THICK: String = "_BODY_TOO_THICK");
lazy!(_BODY_TOO_THIN: String = "_BODY_TOO_THIN");

/// A sauce too thick or too thin: thin unless the line complains of thick and is not thickening.
pub fn _body(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = format!(" {} ", understood.words.iter().map(|w| _fold(w)).collect::<Vec<_>>().join(" "));
    let thickening = said.contains("thicken") || said.contains("thicker") || said.contains("thickening");
    let complaining = any_in(&said, &_TOO_THICK_SAID);
    if complaining && !thickening {
        return Some(_BODY_TOO_THICK.clone());
    }
    Some(_BODY_TOO_THIN.clone())
}

lazy!(CHILD_WORDS: Vec<String> = "CHILD_WORDS");
lazy!(ADULT_VEG: String = "ADULT_VEG");
lazy!(KIDS_VEG: String = "KIDS_VEG");

/// Whether the line, or one of the last three, is about a child.
pub fn _about_a_child(memory: &Memory, understood: &Understood) -> bool {
    let line = format!(" {} ", understood.clean);
    if any_in(&line, &CHILD_WORDS) {
        return true;
    }
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(3);
    for turn in turns[start..].iter().rev() {
        let said = format!(" {} ", turn.user);
        if any_in(&said, &CHILD_WORDS) {
            return true;
        }
    }
    false
}

/// More veg: the grown-up card, or the children's card unless it was just given.
pub fn _more_veg(memory: &mut Memory, understood: &Understood) -> Option<String> {
    if !_about_a_child(memory, understood) {
        return Some(ADULT_VEG.clone());
    }
    if _the_veg_card_was_just_said(memory) { None } else { Some(KIDS_VEG.clone()) }
}

/// Whether one of the last two replies was the children's veg card.
pub fn _the_veg_card_was_just_said(memory: &Memory) -> bool {
    let opening = text::slice(&KIDS_VEG, 0, Some(40)).to_lowercase();
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(2);
    turns[start..].iter().any(|t| t.bot.to_lowercase().contains(&opening))
}

/// The children's veg card, only about a child and not twice running.
pub fn _kids_veg(memory: &mut Memory, understood: &Understood) -> Option<String> {
    if !_about_a_child(memory, understood) {
        return None;
    }
    if _the_veg_card_was_just_said(memory) { None } else { Some(KIDS_VEG.clone()) }
}

/// Every crop with a harvest row (the garden's).
pub fn _harvest_crops() -> HashSet<String> {
    garden_harvest_crops()
}

/// The words that say a crop is being cooked rather than grown (the garden's).
pub fn _done_words() -> Vec<String> {
    garden_done_not()
}

lazy!(_A_HOW_MUCH: Re = "_A_HOW_MUCH");
lazy!(_A_HEAT_OR_TIME: Re = "_A_HEAT_OR_TIME");
lazy!(_A_SIDE_SAID: Re = "_A_SIDE_SAID");

/// False when the line asks how much to make, or what to serve beside it, and not a heat or time.
pub fn _not_a_portion_ask(understood: &Understood) -> bool {
    let joined = understood.words.join(" ");
    let text = or(or(&understood.clean, &understood.norm_text), &joined).to_lowercase();
    if _A_HOW_MUCH.is_match(&text) && !_A_HEAT_OR_TIME.is_match(&text) {
        return false;
    }
    if _A_SIDE_SAID.is_match(&text) && !_A_HEAT_OR_TIME.is_match(&text) {
        return false;
    }
    true
}

lazy!(NOT_A_METHOD: Vec<String> = "NOT_A_METHOD");
lazy!(KNOWS_READY: Vec<String> = "KNOWS_READY");
lazy!(DISH_BEFORE: HashSet<String> = "DISH_BEFORE");
lazy!(DISH_AFTER: HashSet<String> = "DISH_AFTER");
lazy!(_AN_APPLIANCE: Vec<String> = "_AN_APPLIANCE");

/// `NOT_WANTED.get(name, ())`.
fn not_wanted(name: &str) -> Vec<String> {
    g("NOT_WANTED").get(name).map(|v| v.strs()).unwrap_or_default()
}

/// A callable reply in a table (e.g. `_corn` in BASICS), run on the reading.
fn call_table_reply(f: &FnRef, understood: &Understood) -> Option<String> {
    match f.id.as_str() {
        "bot.knowledge.cooking:_corn:203" => Some(_corn(understood)),
        _ => None,
    }
}

/// The plain method for an everyday staple the line names, or None.
pub fn _staple(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said: Vec<String> = understood.words.iter().map(|w| _fold(&w.to_lowercase())).collect();
    let joined = format!(" {} ", said.join(" "));
    if any_in(&joined, &NOT_A_METHOD) {
        return None;
    }
    if any_in(&joined, &KNOWS_READY) {
        let done = _done_words();
        if !done.iter().any(|w| joined.contains(&format!(" {w} "))) {
            let harvest = _harvest_crops();
            if harvest.iter().any(|c| joined.contains(&format!(" {c} "))) {
                return None;
            }
        }
    }
    for row in g("BASICS").items() {
        let row = row.items();
        let names = row[0].strs();
        let reply = &row[1];
        for name in &names {
            let parts: Vec<String> = text::split(name).iter().map(|p| _fold(p)).collect();
            let size = parts.len();
            if said.len() + 1 < size + 1 {
                continue;
            }
            for start in 0..(said.len() + 1 - size) {
                if said[start..start + size] != parts[..] {
                    continue;
                }
                let before = if start > 0 { said[start - 1].as_str() } else { "" };
                let after = if start + size < said.len() { said[start + size].as_str() } else { "" };
                if DISH_BEFORE.contains(before) || DISH_AFTER.contains(after) {
                    continue;
                }
                if !["white sauce", "cheese sauce", "bechamel", "béchamel"].contains(&name.as_str())
                    && (said.iter().any(|w| w == "sauce") || said.iter().any(|w| w == "saucy"))
                {
                    continue;
                }
                let rest: Vec<String> = said[..start].iter().chain(said[start + size..].iter()).cloned().collect();
                let mut guards = not_wanted(&names[0]);
                if *name != names[0] {
                    guards.extend(not_wanted(name));
                }
                let own = match reply.as_str() {
                    Some(r) => _fold(&r.to_lowercase()),
                    None => String::new(),
                };
                if _a_guard_said(&rest, &joined, &guards, &own) {
                    continue;
                }
                return match reply {
                    Value::Fn(f) => call_table_reply(f, understood),
                    other => Some(other.str().to_string()),
                };
            }
        }
    }
    None
}

/// Whether the line names an appliance together with a cut of meat (the dish rows' own case).
pub fn _an_appliance_has_the_line(text: &str) -> bool {
    dishes_names_an_appliance_and_a_cut(text)
}

/// True when one of a staple row's "not wanted" words is in the line.
pub fn _a_guard_said(rest: &[String], joined: &str, guards: &[String], own: &str) -> bool {
    for guard in guards {
        if guard.contains(' ') {
            let phrase = _fold(guard);
            if joined.contains(&phrase) && !own.contains(&phrase) {
                if !_AN_APPLIANCE.contains(guard) || !_an_appliance_has_the_line(joined) {
                    return true;
                }
            }
        } else if rest.iter().any(|w| w == guard) {
            return true;
        }
    }
    false
}

/// Pancakes, thin or American.
pub fn _pancakes(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words: HashSet<&str> = understood.words.iter().map(String::as_str).collect();
    let any = |xs: &[&str]| xs.iter().any(|x| words.contains(x));
    if words.contains("mix") && !any(&["make", "made", "making", "recipe", "batter", "whisk", "homemade", "from"]) {
        return None;
    }
    if any(&["american", "fluffy", "thick", "buttermilk", "scotch"]) {
        return Some("For a dozen thick American pancakes, whisk 150g plain flour, 2 teaspoons of baking powder, a tablespoon of sugar, 2 eggs and 250ml milk into a thick batter that holds its shape, and let it rest 10 minutes. Cook them in a buttered pan on a medium heat, 2 minutes a side, and turn them once, when bubbles pop and stay open. The baking powder is what lifts them; for thin pancakes leave it out and use 300ml milk.".into());
    }
    Some("For about 12 thin pancakes, whisk 100g plain flour, 2 eggs and 300ml milk with a pinch of salt, then let the batter rest 20 minutes. Cook them in a hot non-stick pan with a little butter, about a minute a side, until bubbles pop on top and the edges look dry. The rest is what stops them tearing; American ones want a teaspoon of baking powder and a thicker batter. Flat ones are a batter that was too thin, or an old raising agent.".into())
}

/// Pancakes for a number of people: the batter scaled to the table.
pub fn _pancake_batch(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let mut people: Option<i64> = None;
    for number in &understood.numbers {
        let Some(v) = &number.value else { continue };
        let value = v.as_f64();
        if !(2.0..=40.0).contains(&value) {
            continue;
        }
        let unit = number.unit.as_deref();
        if matches!(
            unit,
            None | Some("") | Some("people") | Some("person") | Some("persons") | Some("adults") | Some("adult")
                | Some("kids") | Some("guests") | Some("portions") | Some("servings")
        ) {
            people = Some(py_round(v));
            break;
        }
    }
    let Some(people) = people else {
        return Some("For four people allow 200g plain flour, 3 eggs and 600ml milk, which makes about 16 thin pancakes; a family with small children is happy on 150g. Cook them a minute a side in a hot pan until the bubbles pop and the edges look dry.".into());
    };
    let grams = 50 * people;
    let flour = if grams % 1000 == 0 {
        format!("{}kg", grams.div_euclid(1000))
    } else if grams > 1000 && grams % 500 == 0 {
        format!("{} and a half kg", grams.div_euclid(1000))
    } else {
        format!("{grams}g")
    };
    let eggs = 0.75 * people as f64;
    let whole = eggs.trunc() as i64;
    let frac = eggs - whole as f64;
    let egg_words = if (0.4..=0.6).contains(&frac) {
        format!("{} or {} eggs", whole, whole + 1)
    } else {
        format!("{} eggs", text::round_half_even(eggs))
    };
    let millilitres = 150 * people;
    let milk = if millilitres < 1000 {
        format!("{millilitres}ml of milk")
    } else if millilitres % 1000 == 0 {
        format!("{} litres of milk", millilitres.div_euclid(1000))
    } else {
        format!("{:.1} litres of milk", millilitres as f64 / 1000.0).replace(".0 litres", " litres")
    };
    Some(format!(
        "For {} people allow {} plain flour, {} and {}, which makes about {} thin pancakes. Cook them a minute a side in a hot non-stick pan with a little butter, until the bubbles pop and the edges look dry; for thick American ones add a teaspoon of baking powder per 200g of flour.",
        people,
        flour,
        egg_words,
        milk,
        4 * people
    ))
}

/// Sweet potato, by the way the line cooks it.
pub fn _sweet_potato(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words: HashSet<&str> = understood.words.iter().map(String::as_str).collect();
    let any = |xs: &[&str]| xs.iter().any(|x| words.contains(x));
    if any(&["boil", "boiled", "boiling", "steam", "steamed", "steaming", "mash", "mashed", "puree", "pureed"]) {
        return Some("Boil peeled chunks for 12 to 15 minutes, until a knife slides in with no resistance: sweet potato softens faster than white potato, so test it at 10. Drain and steam it dry for a minute, then mash it with butter and a pinch of salt and go easy on the milk, because it is wetter than white potato.".into());
    }
    if any(&["microwave", "microwaved", "microwaving", "quick", "quickly", "fast", "fastest", "hurry", "hurried", "hungry", "rush"]) {
        return Some("One medium sweet potato, pricked all over with a fork, takes 5 to 8 minutes on high: give it 5, turn it, then a minute at a time until a knife slides in with no resistance. Two take 10 to 12 minutes, and chunks for mash take 8. It steams in there rather than crisping, so give the skin 5 minutes in a hot oven or a dry pan afterwards if you want it crisp.".into());
    }
    if any(&["fryer", "airfryer", "airfried"]) {
        return Some("In the air fryer, wedges cut into 2cm pieces take 15 to 18 minutes at 190C, shaken once halfway: toss them in a little oil, salt and paprika first and do not crowd the basket, or they steam instead of crisping. Whole ones take 35 to 40 minutes at 180C, and fries cut thin take 12 to 15 minutes.".into());
    }
    Some("For wedges, cut them into 2cm pieces, toss them in oil, salt and paprika and roast at 200C (180C fan) for 25 to 30 minutes. Whole, they bake in about 45 minutes at 200C, or 8 minutes in the microwave for one on its own. Boiled for mash, chunks take 12 to 15 minutes.".into())
}

/// Roast chicken thighs, with lemon and potatoes when the line says so.
pub fn _thighs(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = understood.words.join(" ");
    if words.contains("lemon") || words.contains("potato") || words.contains("potatoes") {
        return Some("Halve the potatoes, toss everything with oil, lemon slices and salt, and roast at 200C (180C fan) for 35 to 40 minutes, skin side up, until the skin is crisp and the juices run clear. Squeeze the roasted lemon over before serving.".into());
    }
    Some("Roast chicken thighs at 200C (180C fan) for 35 to 40 minutes, skin side up and spaced out, until the skin is crisp and the juices run clear. Salt the skin before they go in and don't crowd the tray, or they steam instead of crisping.".into())
}

/// A weight in words: grams under a kilo, else kilos.
pub fn _weight_words(kilos: f64) -> String {
    if kilos < 1.0 {
        return format!("{}g", text::round_half_even(kilos * 1000.0));
    }
    format!("{}kg", py_g(kilos))
}

/// Chicken legs or drumsticks, with the frozen note.
pub fn _legs(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let mut reply = _legs_method(understood);
    let words = understood.words.join(" ");
    if words.contains("frozen") || words.contains("freezer") {
        reply.push_str(" From frozen, add about 10 minutes and check at the bone before you serve it.");
    }
    Some(reply)
}

/// The method for chicken legs or drumsticks, by how the line cooks them.
pub fn _legs_method(understood: &Understood) -> String {
    let words = understood.words.join(" ");
    let has = |xs: &[&str]| xs.iter().any(|x| words.contains(x));
    let small = has(&["drumstick", "drumsticks"]);
    let done = "It is done when the juices at the thickest part run clear and the meat pulls away from the bone - if you have a thermometer, 74C.";
    if has(&["boil", "boiling", "poach", "poached", "simmer", "simmering"]) {
        return format!(
            "Simmer {} in salted water for {} minutes. {done}",
            if small { "drumsticks" } else { "the legs" },
            if small { "20 to 25" } else { "25 to 30" }
        );
    }
    if has(&["grill", "grilled", "grilling", "barbecue", "barbeque", "bbq"]) {
        return format!(
            "Over medium heat, a {} takes about {}, turning once and moving it off the hottest spot if it colours too fast. {done}",
            if small { "drumstick" } else { "leg" },
            if !small { "8 to 10 minutes a side" } else { "10 minutes a side" }
        );
    }
    if has(&["slow", "crockpot", "crock"]) {
        return format!(
            "In a slow cooker, legs take 3 to 4 hours on low with a splash of stock or a tin of tomatoes; drumsticks are done nearer 2 to 3. {done}"
        );
    }
    if words.contains("air") {
        return format!(
            "In an air fryer, {} take {} minutes at 190C, turned once, until the skin is crisp. {done}",
            if small { "drumsticks" } else { "legs" },
            if small { "20 to 25" } else { "25 to 30" }
        );
    }
    if small {
        return "Roast drumsticks at 200C (180C fan) for 25 to 30 minutes, skin side up and spaced out, until the skin is crisp and the juices at the bone run clear. Salt the skin first, and give them a turn halfway if your oven has a hot spot.".into();
    }
    "A whole chicken leg takes 35 to 40 minutes at 200C (180C fan), skin side up: a drumstick alone is 25 to 30. It is done when the juices at the thickest part run clear and the meat pulls away from the bone - if you have a thermometer, 74C. Salt the skin and don't crowd the tray, or it steams instead of crisping.".into()
}

lazy!(TENDER_ADJECTIVE_BEFORE: HashSet<String> = "TENDER_ADJECTIVE_BEFORE");

/// True when the line is about chicken tenders (the pieces), not tender chicken.
pub fn _tenders_are_a_piece(understood: &Understood) -> bool {
    let words = lower_words(understood);
    if words.iter().any(|w| w == "tenders" || w == "strips" || w == "goujons") {
        return true;
    }
    for (i, word) in words.iter().enumerate() {
        if word == "chicken" && i + 1 < words.len() && ["tender", "strip", "goujon"].contains(&words[i + 1].as_str()) {
            let before = if i > 0 { words[i - 1].as_str() } else { "" };
            return !TENDER_ADJECTIVE_BEFORE.contains(before);
        }
    }
    false
}

/// Chicken tenders, by how the line cooks them.
pub fn _tenders_method(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = understood.words.join(" ");
    let has = |xs: &[&str]| xs.iter().any(|x| words.contains(x));
    let done = "They are done when the middle is 74C and no pink is left - cut the thickest one open if you have no thermometer.";
    if has(&["air", "airfryer", "airfryers"]) {
        return Some(format!("In an air fryer, chicken tenders take 9 to 10 minutes at 190C in a single layer, shaken once halfway. {done}"));
    }
    if has(&["grill", "grilled", "grilling"]) {
        return Some(format!("Under a hot grill, chicken tenders take 4 to 5 minutes a side, turned once. {done}"));
    }
    if has(&["fry", "fried", "frying", "pan", "shallow"]) {
        return Some(format!("In a frying pan over a medium heat, chicken tenders take 3 to 4 minutes a side, until golden outside and white in the middle. {done}"));
    }
    Some(format!("Chicken tenders are thin, so 10 to 12 minutes in a 200C (180C fan) oven is enough: spread out on an oiled tray and turned once halfway. {done}"))
}

re_lazy!(WEIGHT_SAID = r"\d|\b(?:kilo\w*|kg|g|lb|lbs|pound|pounds|oz|ounce|ounces)\b");
re_lazy!(A_OR_AN = r"(?:a|an)\s+(.+)$");

/// The weight of the bird the line gives (kilos, as said), or None.
pub fn _bird_weight(understood: &Understood) -> Option<(f64, String)> {
    for number in &understood.numbers {
        let Some(value) = num(&number.value) else { continue };
        if value <= 0.0 {
            continue;
        }
        let kilos = match number.unit.as_deref() {
            Some("kg") => value,
            Some("g") => value / 1000.0,
            Some("lb") => value * 0.4536,
            Some("oz") => value * 0.02835,
            _ => continue,
        };
        if kilos != 0.0 && (0.4..=12.0).contains(&kilos) {
            let mut said = number.text.clone();
            if !WEIGHT_SAID.is_match(&said.to_lowercase()) {
                said = _weight_words(kilos);
            }
            let low = said.to_lowercase();
            if let Some(m) = A_OR_AN.match_(&low) {
                said = m.str(1).to_string();
            }
            return Some((kilos, said));
        }
    }
    None
}

/// Hours in words, to the nearest three, as days where it reads better.
pub fn _hours_words(hours: f64) -> String {
    let hours = text::round_half_even(hours / 3.0) * 3;
    if hours < 24 {
        return format!("{hours} hours");
    }
    let (days, rest) = (hours.div_euclid(24), hours.rem_euclid(24));
    let whole = match days {
        1 => Some("a day"),
        2 => Some("two days"),
        3 => Some("three days"),
        4 => Some("four days"),
        5 => Some("five days"),
        _ => None,
    };
    let half = match days {
        1 => Some("a day and a half"),
        2 => Some("two and a half days"),
        3 => Some("three and a half days"),
        _ => None,
    };
    if rest == 0 {
        if let Some(w) = whole {
            return w.into();
        }
    }
    if rest == 12 {
        if let Some(h) = half {
            return h.into();
        }
    }
    format!("{hours} hours")
}

/// Minutes in words, to the nearest five ("1 hour 20 minutes").
pub fn _in_words(minutes: f64) -> String {
    let minutes = text::round_half_even(minutes / 5.0) * 5;
    let (hours, rest) = (minutes.div_euclid(60), minutes.rem_euclid(60));
    if hours == 0 {
        return format!("{rest} minutes");
    }
    let hours_text = format!("{hours} hour{}", if hours == 1 { "" } else { "s" });
    if rest != 0 { format!("{hours_text} {rest} minutes") } else { hours_text }
}

/// How long a whole chicken roasts, from the weight the line gives (2kg when it gives none).
pub fn _chicken_time(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said_line = or(&understood.clean, &understood.text);
    if a_price_is_judged(said_line) {
        return None;
    }
    let check = "Check the thigh reads 74C on a thermometer and the juices run clear, then rest it for 15 minutes before carving.";
    let (kilos, said) = _bird_weight(understood).unwrap_or((2.0, "2kg".to_string()));
    Some(format!(
        "About 20 minutes per 500g plus 20 minutes at 190C (170C fan), so a {said} bird takes roughly {}. {check}",
        _in_words(20.0 * (kilos / 0.5) + 20.0)
    ))
}

// ---------------------------------------------------------------------------------------------
// Seasonings, a table of people, sides

lazy!(_A_RUB: String = "_A_RUB");
re_lazy!(A_RUB_ASK = r"\b(?:dry rub|spice rub|make a rub|what goes in a rub|rub recipe)\b");

/// The seasoning row for the longest seasoning name the line says, or the rub, or None.
pub fn _a_seasoning(text_: &str) -> Option<String> {
    let mut best: Option<(usize, String)> = None;
    for row in g("SEASONINGS").items() {
        let row = row.items();
        for name in row[0].strs() {
            let n = name.chars().count();
            if word_in(text_, &name) && best.as_ref().is_none_or(|b| n > b.0) {
                best = Some((n, row[1].str().to_string()));
            }
        }
    }
    if let Some((_, reply)) = best {
        return Some(reply);
    }
    if A_RUB_ASK.is_match(text_) {
        return Some(_A_RUB.clone());
    }
    None
}

/// The spelled-out counts two to a hundred ("twenty five": 25).
pub fn _spelled_counts() -> HashMap<String, i64> {
    let ones = g("_ONES").pairs();
    let teens = g("_TEENS").pairs();
    let tens = g("_TENS").pairs();
    let mut counts = HashMap::new();
    for table in [ones, teens, tens] {
        for (value, word) in table {
            let v = value.as_int().unwrap_or(0);
            if v >= 2 {
                counts.insert(word.str().to_string(), v);
            }
        }
    }
    for (t, tens_word) in tens {
        for (o, one_word) in ones {
            counts.insert(format!("{} {}", tens_word.str(), one_word.str()), t.as_int().unwrap_or(0) + o.as_int().unwrap_or(0));
        }
    }
    counts.insert("hundred".into(), 100);
    counts.insert("a hundred".into(), 100);
    counts
}

lazy!(_NOT_A_TABLE: Vec<String> = "_NOT_A_TABLE");

/// True unless the word after a count says it counts something other than people.
pub fn _people_after(text_: &str, end: usize) -> bool {
    let rest = text::split(text::strip(&text_[end..]));
    let after = rest.first().map(|w| text::strip_chars(w, ".,!?;:").to_lowercase()).unwrap_or_default();
    !_NOT_A_TABLE.contains(&after)
}

lazy!(_CROWD_FOR: Re = "_CROWD_FOR");
lazy!(_CROWD_WITHIN: Re = "_CROWD_WITHIN");
lazy!(_CROWD_PHRASES: Vec<String> = "_CROWD_PHRASES");
re_lazy!(FOR_A_COUNT = r"\bfor\s+(\d{1,3})\b");
re_lazy!(A_COUNT_OF_PEOPLE = r"\b(\d{1,3})\s+(?:people|of us|adults|guests|coming)\b");

fn crowd_number(word: &str) -> i64 {
    g("_CROWD_NUMBERS").get(word).and_then(Value::as_int).unwrap_or_else(|| panic!("KeyError: {word}"))
}

/// How many people the line cooks for (the largest count it gives), 6 for a crowd word, or 0.
pub fn _crowd(text_: &str) -> i64 {
    let text_ = text_.to_lowercase().replace('-', " ");
    let mut numbers: Vec<i64> = FOR_A_COUNT
        .finditer(&text_)
        .into_iter()
        .filter(|m| _people_after(&text_, m.end()))
        .map(|m| m.str(1).parse::<i64>().unwrap())
        .collect();
    for m in _CROWD_FOR.finditer(&text_) {
        if _people_after(&text_, m.end()) {
            numbers.push(crowd_number(m.str(1)));
        }
    }
    for word in _CROWD_WITHIN.findall_str(&text_) {
        numbers.push(crowd_number(&word));
    }
    for count in A_COUNT_OF_PEOPLE.findall_str(&text_) {
        numbers.push(count.parse().unwrap());
    }
    if let Some(grouped) = people_in_groups(&text_) {
        numbers.push(grouped);
    }
    if let Some(m) = numbers.iter().max() {
        return *m;
    }
    if any_in(&text_, &_CROWD_PHRASES) {
        return 6;
    }
    0
}

lazy!(_NAMED_TABLE: Re = "_NAMED_TABLE");

/// True when the line names a table of people without a number ("the whole family").
pub fn _named_table(text_: &str) -> bool {
    _NAMED_TABLE.is_match(text_)
}

lazy!(_TABLE_PHRASES: Vec<String> = "_TABLE_PHRASES");

/// True when the line is cooking for a table of people.
pub fn _table_text(text_: &str) -> bool {
    let text_ = text_.to_lowercase();
    let count = _crowd(&text_);
    if count >= 6 {
        return true;
    }
    if count == 0 {
        return _named_table(&text_);
    }
    if count >= 3 && (any_in(&text_, &_CROWD_PHRASES) || text_.contains("of us")) {
        return true;
    }
    let has = |xs: &[&str]| xs.iter().any(|x| text_.contains(x));
    if any_in(&text_, &_TABLE_PHRASES) && has(&["cook", "cooking", "make", "feed", "serve"]) {
        return true;
    }
    count >= 3 && any_in(&text_, &_TABLE_PHRASES) && has(&["lunch", "lunches", "meal", "meals", "eat"])
}

lazy!(_CROWD_VERBS: Vec<String> = "_CROWD_VERBS");
lazy!(_CROWD_NOT_A_MEAL: Vec<String> = "_CROWD_NOT_A_MEAL");
lazy!(_CROWD_FOOD: Vec<String> = "_CROWD_FOOD");
lazy!(_CROWD_QUANTITY: Vec<String> = "_CROWD_QUANTITY");

/// True when the line asks what to cook for a table of people.
pub fn _crowd_ask(clean: &str) -> bool {
    let text_ = clean.to_lowercase();
    if !_table_text(&text_) {
        return false;
    }
    if any_in(&text_, &_CROWD_VERBS) {
        return true;
    }
    if any_in(&text_, &_CROWD_NOT_A_MEAL) {
        return false;
    }
    if any_in(&text_, &_CROWD_FOOD) {
        return !any_in(&text_, &_CROWD_QUANTITY);
    }
    false
}

fn crowd_words(count: i64) -> String {
    g("_CROWD_WORDS")
        .pairs()
        .iter()
        .find(|(k, _)| k.as_int() == Some(count))
        .map(|(_, v)| v.str().to_string())
        .unwrap_or_else(|| count.to_string())
}

/// How many trays the crowd dish takes, for this many people.
pub fn _trays(count: i64) -> String {
    if count >= 13 {
        return "two or three trays of it, on the top and middle shelves, or one dish made in the biggest pot you own".into();
    }
    if count >= 9 {
        return format!("two trays of it, on the top and middle shelves and swapped halfway, to feed {}", crowd_words(count));
    }
    if count != 0 && count <= 5 {
        return format!("one tray of it is plenty for {}", crowd_words(count));
    }
    if count == 0 {
        return "which does four to six, so count the potatoes yourself".into();
    }
    "which feeds eight from one oven".into()
}

/// One big oven dish for a table, meat-free when the eaters ruled meat out.
pub fn _crowd_meal(banned: &HashSet<String>, occasion: bool, count: i64) -> String {
    let meats = [
        "meat", "chicken", "beef", "pork", "lamb", "mince", "bacon", "sausage", "sausages", "ham", "turkey", "fish",
        "salmon", "cod", "tuna", "prawn", "prawns",
    ];
    let vegetarian = meats.iter().any(|m| banned.contains(*m));
    let (dish, amount) = if vegetarian {
        (
            "a big mushroom and lentil lasagne or a chickpea and vegetable traybake, made in the largest dish you own",
            "about 300g of the dish per person",
        )
    } else {
        (
            "a tray of chicken thighs or sausages with halved potatoes, peppers and onions at 200C (180C fan) for 45 minutes",
            "about 200g of meat and a big potato each",
        )
    };
    let mut reply = format!(
        "Cook one big oven dish rather than several pans: {dish}, {}. Allow {amount}, add a loaf of bread and a bowl of salad, and write the times down so nothing needs the oven at once.",
        _trays(count)
    );
    if occasion {
        reply.push_str(" Anything that can be made that morning - a chilli, a stew, a lasagne - just gets reheated, which leaves the day free.");
    }
    reply
}

lazy!(_OCCASIONS: Vec<String> = "_OCCASIONS");
re_lazy!(AN_ORDINAL_DAY = r"\b\d{1,3}(?:st|nd|rd|th)\b");

/// True when the line is a day that matters: a birthday, a wake, or a "70th".
pub fn _occasion(text_: &str) -> bool {
    if any_in(text_, &_OCCASIONS) {
        return true;
    }
    AN_ORDINAL_DAY.is_match(text_)
}

/// The fallback list, unless the line is a meal for a table or an occasion.
pub fn _crowd_or_any(text_: &str, banned: &HashSet<String>) -> String {
    if _table_text(text_) {
        return _crowd_meal(banned, _occasion(text_), _crowd(text_));
    }
    let asks = ["what should i cook", "what can i cook", "what shall i cook", "what to cook", "what to make", "meal", "dinner", "cook", "no idea"];
    if _occasion(text_) && asks.iter().any(|p| text_.contains(p)) {
        return _crowd_meal(banned, true, _crowd(text_));
    }
    _any_meal(banned)
}

re_lazy!(NOT_A_SIDE = r"\bpain\b|\bstitch\b|\bside effects?\b|\bleft side\b|\bright side\b");
lazy!(A_TIME_OR_HEAT_ASK: Re = "A_TIME_OR_HEAT_ASK");
lazy!(SEASONING_ASKS: Re = "SEASONING_ASKS");

/// What to serve with a dish: the seasoning, the plan's sides, the table's row or the pairing rule.
pub fn _sides(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let joined = understood.words.join(" ");
    let text_ = or(&understood.clean, &joined).to_lowercase();
    if NOT_A_SIDE.is_match(&text_) {
        return None;
    }
    if A_TIME_OR_HEAT_ASK.is_match(&text_) {
        return None;
    }
    if SEASONING_ASKS.is_match(&text_) {
        if let Some(seasoned) = _a_seasoning(&text_) {
            if !seasoned.is_empty() {
                return Some(seasoned);
            }
        }
    }
    let table = _table_side(&text_);
    let plan = _plan_side(Some(memory), &text_);
    if let Some((_, sides)) = plan {
        return Some(sides);
    }
    if let Some((_, sides)) = table {
        return Some(sides);
    }
    if dishes_asks_for_a_side_of_a_food(&text_) {
        return Some(dishes_general_sides(&text_));
    }
    None
}

/// The sides of the dish being planned, or None.
pub fn plan_side(memory: Option<&Memory>, text_: &str) -> Option<String> {
    _plan_side(memory, &text_.to_lowercase()).map(|(_, s)| s)
}

/// The side for the dish in this line or the last few, or None.
pub fn side_for(memory: &Memory, text_: &str) -> Option<String> {
    if let Some(side) = plan_side(Some(memory), text_) {
        if !side.is_empty() {
            return Some(side);
        }
    }
    let mut lines = vec![text_.to_string()];
    lines.extend(dishes_recent_texts(memory).into_iter().take(3));
    for line in lines {
        if let Some((_, side)) = _table_side(&line.to_lowercase()) {
            return Some(side);
        }
    }
    None
}

/// The sides row for the longest dish name the line says: (length, reply).
pub fn _table_side(text_: &str) -> Option<(usize, String)> {
    let mut best: Option<(usize, String)> = None;
    for row in g("SIDES").items() {
        let row = row.items();
        for name in row[0].strs() {
            let n = name.chars().count();
            if word_in(text_, &name) && best.as_ref().is_none_or(|b| n > b.0) {
                best = Some((n, row[1].str().to_string()));
            }
        }
    }
    best
}

/// The sides of the plan for the dish this line (or one of the last two) names.
pub fn _plan_side(memory: Option<&Memory>, text_: &str) -> Option<(usize, String)> {
    let mut lines = vec![text_.to_string()];
    if let Some(m) = memory {
        lines.extend(dishes_recent_texts(m).into_iter().take(2));
    }
    for line in lines {
        let name = dishes_dish_in(&line);
        let Some(plan) = dishes_plan(name.as_deref().unwrap_or("")) else { continue };
        if !plan.truthy() {
            continue;
        }
        let sides = plan.get("sides");
        if !sides.is_some_and(Value::truthy) {
            return None;
        }
        let name = name.unwrap_or_default();
        return Some((name.chars().count(), sides.unwrap().str().to_string()));
    }
    None
}

lazy!(DIABETES_WORDS: Vec<String> = "DIABETES_WORDS");
re_lazy!(NOT_ALNUM = r"[^a-z0-9]+");

/// True when the line says diabetes in one of its forms.
pub fn _says_diabetes(text_: &str) -> bool {
    let folded = format!(" {} ", NOT_ALNUM.sub(" ", &text_.to_lowercase()).trim_matches(text::is_space));
    DIABETES_WORDS.iter().any(|w| folded.contains(&format!(" {w} ")))
}

re_lazy!(MINUTES_SAID = r"\b(\d{1,3})\s*(?:minutes?|mins?)\b");

/// The minutes the line gives, in figures or words, or None.
pub fn _minutes_named(said: &str) -> Option<i64> {
    if let Some(m) = MINUTES_SAID.search(said) {
        return m.str(1).parse().ok();
    }
    for row in g("TIME_PHRASES").items() {
        let row = row.items();
        if said.contains(row[0].str()) {
            return row[1].as_int();
        }
    }
    for row in g("TIME_WORDS").items() {
        let row = row.items();
        if Re::new(&format!("\\b{}\\b", row[0].str()), 0).is_match(said) {
            return row[1].as_int();
        }
    }
    None
}

lazy!(ANY_MEAL: String = "ANY_MEAL");
lazy!(TIME_TWENTY: String = "TIME_TWENTY");
lazy!(TIME_SOON: String = "TIME_SOON");
lazy!(TIME_MIDDLING: String = "TIME_MIDDLING");

/// Dinners that fit the minutes the line gives.
pub fn _time_budget(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = understood.words.iter().map(|w| _fold(w)).collect::<Vec<_>>().join(" ");
    let minutes = _minutes_named(&said)?;
    if minutes <= 15 {
        return Some(ANY_MEAL.clone());
    }
    if minutes <= 22 {
        return Some(TIME_TWENTY.clone());
    }
    if minutes <= 28 {
        return Some(TIME_SOON.clone());
    }
    if minutes <= 90 {
        return Some(TIME_MIDDLING.clone());
    }
    None
}

lazy!(CHEAP_DISHES: Vec<String> = "CHEAP_DISHES");
lazy!(CHEAP_TAIL: String = "CHEAP_TAIL");
lazy!(CHEAP_TAIL_MEATFREE: String = "CHEAP_TAIL_MEATFREE");

/// Cheap dinners for four, leaving out what the eater has ruled out.
pub fn _cheap_dinners(banned: &HashSet<String>) -> String {
    let banned_list: Vec<String> = banned.iter().cloned().collect();
    let mut dishes: Vec<String> = CHEAP_DISHES
        .iter()
        .filter(|one| !_mentions(one, &banned_list) && !_bans(&text::split(one), one, banned))
        .cloned()
        .collect();
    if dishes.is_empty() {
        dishes = CHEAP_DISHES.clone();
    }
    let tail = if _mentions(&CHEAP_TAIL, &banned_list) || _bans(&["mince".to_string()], &CHEAP_TAIL, banned) {
        CHEAP_TAIL_MEATFREE.clone()
    } else {
        CHEAP_TAIL.clone()
    };
    let listed = format!("{} or {}", dishes[..dishes.len() - 1].join(", "), dishes[dishes.len() - 1]);
    format!("Cheap dinners that feed four for about a fiver: {listed}. {tail}")
}

lazy!(PROTEIN_COMPLETE: String = "PROTEIN_COMPLETE");
lazy!(PROTEIN_SOURCES: String = "PROTEIN_SOURCES");
re_lazy!(COMPLETE = r"\bcomplete\b");

/// Protein without meat, or what a complete protein is.
pub fn _protein(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text_ = understood.clean.to_lowercase();
    if COMPLETE.is_match(&text_) {
        return Some(PROTEIN_COMPLETE.clone());
    }
    Some(PROTEIN_SOURCES.clone())
}

lazy!(_SOFT_FRUIT_WANTS_THE_OVEN: Re = "_SOFT_FRUIT_WANTS_THE_OVEN");
lazy!(_SOFT_FRUIT: Re = "_SOFT_FRUIT");

/// A soft fruit is a cooking question only when it is going in the oven.
pub fn _a_soft_fruit_is_still_a_cooking_question(understood: &Understood) -> bool {
    let text_ = understood.clean.to_lowercase();
    _SOFT_FRUIT_WANTS_THE_OVEN.is_match(&text_) || !_SOFT_FRUIT.is_match(&text_)
}

/// Which fruit the bake is about (apple when the line names none of the others).
pub fn _the_baked_fruit(text_: &str) -> &'static str {
    static NAMES: LazyLock<Vec<(Re, &'static str)>> = LazyLock::new(|| {
        [("quince", "quince"), ("nectarine", "peach"), ("peach", "peach"), ("plum", "plum"), ("pear", "pear")]
            .iter()
            .map(|(n, f)| (Re::new(&format!("\\b{n}(?:es|s)?\\b"), 0), *f))
            .collect()
    });
    for (re, fruit) in NAMES.iter() {
        if re.is_match(text_) {
            return fruit;
        }
    }
    "apple"
}

re_lazy!(STEWING = r"\b(?:stew|stews|stewed|stewing|boil|boils|boiled|boiling|simmer|simmered|simmering|poach|poached|poaching|puree|pureed)\b");
re_lazy!(MICROWAVE_WORD = r"\bmicrowave\b");

/// Baked, stewed or microwaved fruit.
pub fn _bake_fruit(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text_ = understood.clean.to_lowercase();
    let fruit = _the_baked_fruit(&text_);
    let lookup = |table: &str| g(table).get(fruit).map(|v| v.str().to_string());
    if STEWING.is_match(&text_) {
        return Some(lookup("STEWED_FRUIT").unwrap_or_else(|| panic!("KeyError: {fruit}")));
    }
    if MICROWAVE_WORD.is_match(&text_) {
        return Some(lookup("MICROWAVED_FRUIT").unwrap_or_else(|| panic!("KeyError: {fruit}")));
    }
    Some(lookup("BAKED_FRUIT").unwrap_or_else(|| st("BAKED_APPLES")))
}

re_lazy!(JACKET_FILLING = r"\b(fill|filling|fillings|topping|toppings|goes with|what with|put in|beans|cheese|tuna|coleslaw|chilli)\b");

/// A jacket potato: the fillings, or how to bake it.
pub fn _jacket(understood: &Understood) -> String {
    let said = understood.clean.to_lowercase();
    if JACKET_FILLING.is_match(&said) {
        return st("JACKET_FILL");
    }
    st("JACKET_BAKE")
}

/// Jam, or chutney when the line says so.
pub fn _jam_make(understood: &Understood) -> String {
    let said = understood.clean.to_lowercase();
    if said.contains("chutney") || said.contains("preserve") {
        return st("CHUTNEY_RECIPE");
    }
    st("JAM_RECIPE")
}

/// Poaching fish, naming the fish the line names.
pub fn _poach_fish(understood: &Understood) -> String {
    let text_ = understood.clean.to_lowercase();
    let template = st("POACH_FISH");
    for name in ["haddock", "cod", "trout", "salmon"] {
        if word_in(&text_, name) {
            return template.replace("{fish}", name);
        }
    }
    template.replace("{fish}", "salmon")
}

re_lazy!(PEEL_WORD = r"\bpeel");
re_lazy!(SKIN_WORD = r"\bskin\b");

/// Mushrooms: peeling or washing.
pub fn _mushroom_wash(understood: &Understood) -> String {
    let said = understood.clean.to_lowercase();
    if PEEL_WORD.is_match(&said) || SKIN_WORD.is_match(&said) {
        return st("MUSHROOM_PEEL");
    }
    st("MUSHROOM_WASH")
}

lazy!(MONTHS_SAID: Re = "MONTHS_SAID");
lazy!(GOING_SAID: Re = "GOING_SAID");
lazy!(EATING_WORDS: HashSet<String> = "EATING_WORDS");
lazy!(PLAN_WORDS: HashSet<String> = "PLAN_WORDS");
lazy!(CHEAP_PLAN_WORDS: HashSet<String> = "CHEAP_PLAN_WORDS");
re_lazy!(LETTER_WORDS = r"[a-z']+");
re_lazy!(FREEZE_OR_BATCH = r"\bfreez|\bbatch");

/// A meal plan for the week: batch, cheap or plain.
pub fn _meal_plan(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let mut parts = vec![understood.clean.to_lowercase()];
    parts.extend(_recent_user_turns(Some(memory), 4));
    let text_ = parts.join(" ");
    let mut said: Vec<String> = words.iter().map(|w| _fold(w)).collect();
    said.extend(LETTER_WORDS.findall_str(&text_).iter().map(|w| _fold(w)));
    if (MONTHS_SAID.is_match(&text_) || GOING_SAID.is_match(&text_)) && !said.iter().any(|w| EATING_WORDS.contains(w)) {
        return None;
    }
    if said.iter().any(|w| PLAN_WORDS.contains(w)) || FREEZE_OR_BATCH.is_match(&text_) {
        return Some(st("PLAN_BATCH"));
    }
    if said.iter().any(|w| CHEAP_PLAN_WORDS.contains(w)) {
        return Some(st("PLAN_CHEAP"));
    }
    Some(st("PLAN_WEEK"))
}

// ---------------------------------------------------------------------------------------------
// Faults: dry, falling apart, a dough

/// A word is the wanted one or starts with it.
pub fn _same(word: &str, wanted: &str) -> bool {
    !word.is_empty() && !wanted.is_empty() && (word == wanted || word.starts_with(wanted))
}

/// The bake the words name, and whether it is said as a plural ("", false when none).
pub fn _the_bake_named(words: &[String]) -> (String, bool) {
    for row in g("_BAKE_NAMES").items() {
        let row = row.items();
        let (one, many) = (row[0].str(), row[1].str());
        if words.iter().any(|w| _same(w, many) || _same(w, one)) {
            return if one == "bread" || one == "pastry" { (one.to_string(), false) } else { (many.to_string(), true) };
        }
    }
    (String::new(), false)
}

/// Whether the words ask why it went wrong.
pub fn _asks_why(words: &[String]) -> bool {
    let said = words.join(" ");
    words.iter().any(|w| w == "why" || w == "howcome")
        || said.contains("what did i do wrong")
        || said.contains("what went wrong")
        || said.contains("what am i doing wrong")
}

lazy!(_BAKE_DISH: Vec<String> = "_BAKE_DISH");
lazy!(_BAKED: Vec<String> = "_BAKED");
lazy!(_RICE_DISH: Vec<String> = "_RICE_DISH");
lazy!(_DIP_DISH: Vec<String> = "_DIP_DISH");
lazy!(_SAUCE_DISH: Vec<String> = "_SAUCE_DISH");

fn any_same(words: &[String], wanted: &[impl AsRef<str>]) -> bool {
    words.iter().any(|w| wanted.iter().any(|x| _same(w, x.as_ref())))
}

fn any_word(words: &[String], set: &[&str]) -> bool {
    words.iter().any(|w| set.contains(&w.as_str()))
}

/// Something that came out dry: the fix for the dish it is.
pub fn _dry(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text_ = understood.clean.to_lowercase();
    let words = lower_words(understood);
    if any_same(&words, &["biscuit", "biscuits", "cookie", "cookies", "shortbread"]) {
        return None;
    }
    let mut rescue = "Serve it with something wet — gravy, a spoon of yoghurt, a squeeze of lemon and oil — and next time take it out 5 minutes earlier and rest it under foil. Dry meat or fish is nearly always overcooked, and resting is what keeps the juice in.";
    if _asks_why(&words) {
        rescue = "Dry meat or fish is nearly always overcooked: it went past the point where the juices set, or it was cooked hard and fast without a rest, so the juice ran out of it. Take it out 5 minutes earlier next time and rest it under foil, and serve this one with something wet - gravy, a spoon of yoghurt, a squeeze of lemon and oil.";
    }
    let said = words.join(" ");
    if any_word(&words, &["pasta", "spaghetti", "penne", "macaroni", "tagliatelle", "fusilli", "noodles", "noodle", "rice noodles", "udon"])
        && any_word(&words, &["overcooked", "over-cooked", "over cooked", "mushy", "soggy"])
    {
        return Some("Overcooked pasta cannot be brought back, but it can be used: drain it, toss it in oil so it stops sticking, and fold it into a bake, a frittata or a soup where soft is fine. Next time salt the water properly, take it off a minute before the packet says, and taste a piece rather than trusting the clock, because a big pan of boiling water comes back to the boil slowly and the first minute is not cooking.".into());
    }
    let named = any_same(&words, &_BAKE_DISH) || _BAKE_DISH.iter().any(|w| w.contains(' ') && said.contains(w.as_str()));
    if named && any_word(&words, &["bake", "baked", "baking", "oven", "overcooked", "over-cooked", "dry"]) {
        return Some("A bake or a pie dries out when it goes in uncovered or stays in too long: stir a splash of milk or a spoon of passata through the sauce, cover it with foil and give it 10 minutes at 180C to come back. Next time keep a little bechamel back to pour over the top, and cover it for the first 20 minutes.".into());
    }
    if any_same(&words, &_BAKED) {
        let (thing, plural) = _the_bake_named(&words);
        let head = if !thing.is_empty() {
            format!("Dry {thing}{}", if plural { " are" } else { " is" })
        } else {
            "A dry bake is".to_string()
        };
        return Some(format!("{head} usually a minute or two too long in the oven, or a little too much flour. Take it out as soon as a skewer comes out with a crumb on it, weigh the flour rather than scooping it with a cup, and don't overbeat - and a dry bake is still good with cream, custard or something wet on it."));
    }
    if any_word(&words, &["rice", "ricecooker"]) || any_same(&words, &_RICE_DISH) {
        return Some("Dry rice means too little water or too long on the heat: add a splash of hot water, put the lid back on and steam it for 5 minutes on the lowest heat. Next time use one and a half parts water to one of rice, lid on, 10 minutes, then rest it off the heat.".into());
    }
    if words.iter().any(|w| _DIP_DISH.contains(w)) {
        return Some("Loosen it a spoonful at a time rather than all at once: a splash of cold water and a squeeze of lemon for hummus, or a little more oil for a pesto, blitzed or stirred through until it is soft again. A dull, dry taste is usually not enough salt or acid rather than not enough oil.".into());
    }
    if any_same(&words, &_SAUCE_DISH) {
        return Some("A sauce that has gone dry wants liquid back, not less time in the pan: a little stock, the water the pasta cooked in, a spoon of tinned tomatoes or a splash of milk, then the lid on and five minutes at a bare simmer. A stew, a curry or a chilli tightens as it stands, so loosen it at the end rather than watering it at the start, and keep the heat low enough that it does not catch.".into());
    }
    if words.iter().any(|w| _same(w, "pastry")) {
        return Some("Dry pastry wants a sprinkle of cold water and a 20 minute rest in the fridge, then roll it between two sheets of baking paper. Next time keep the fat cold and add water a tablespoon at a time, because pastry dries out fast once the flour is in.".into());
    }
    if text_.contains("safe to eat") || text_.contains("ok to eat") || any_word(&words, &["safe", "safely", "food-poisoning", "ill"]) {
        return Some(format!("Dry is a texture problem, not a safety one: if it was cooked through (75C inside, no pink meat and clear juices) it is fine cold, or reheated once until steaming. {rescue}"));
    }
    Some(rescue.to_string())
}

/// A loaf of bread: white, or wholemeal and brown.
pub fn _bread(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let said = words.join(" ");
    let wholemeal = any_word(&words, &["wholemeal", "wholewheat", "whole"])
        || ["whole wheat", "whole-wheat", "wholemeal flour"].iter().any(|p| said.contains(p));
    let brown = any_word(&words, &["brown", "granary", "malted"]);
    if wholemeal || brown {
        return Some("Use 500g of wholemeal flour (or 300g wholemeal flour and 200g strong white flour, which rises more easily), 7g of dried yeast, 10g of salt and 350ml of warm water: wholemeal drinks more and takes longer, so leave it somewhere warm 2 to 3 hours until it has doubled rather than 1 to 2. Knead 10 minutes, shape it into a tin, leave it 45 minutes more, then bake at 220C (200C fan) for 35 to 40 minutes and tap the base: a hollow sound means it is done.".into());
    }
    Some("Put 500g strong white flour, 7g of dried yeast, 10g of salt and 320ml of warm water in a bowl, mix it to a sticky dough and knead 10 minutes until it stops tearing. Leave it somewhere warm 1 to 2 hours until it has doubled, shape it into a tin, leave it another 45 minutes, then bake at 220C (200C fan) for 30 to 35 minutes and tap the base: a hollow sound means it is done.".into())
}

lazy!(_FISH_NAMES: Vec<String> = "_FISH_NAMES");
lazy!(_COMING_APART_ON_PURPOSE: Vec<String> = "_COMING_APART_ON_PURPOSE");
lazy!(_MUSHY_OWN_ROW: Vec<String> = "_MUSHY_OWN_ROW");

/// Food that falls apart: the fix for the thing it is.
pub fn _falls_apart(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let said = format!(" {} ", words.join(" "));
    let text_ = understood.clean.to_lowercase();
    let whole = format!("{said} {text_} ");
    let why = _asks_why(&words);
    let names = |wanted: &[&str]| any_same(&words, wanted);
    if any_same(&words, &_FISH_NAMES) {
        if names(&["pie", "pies", "fish pie"]) {
            return Some("A fish pie comes apart because the sauce is thin, not because the fish broke: make the white sauce thick enough to hold a spoon (about 40g butter and 40g flour to 400ml milk) and let the pie stand 10 minutes out of the oven before you cut it, so the starch sets. A crumble or a pastry lid that slides off is the same fault - too much liquid under the crust.".into());
        }
        if why {
            return Some("A fillet comes apart for two reasons: it went into the pan wet, so it steamed instead of sealing, or it was moved before the skin had set. Dry it with kitchen paper, get the pan properly hot, lay it in skin side down, press it flat for ten seconds and leave it four minutes before you turn it once. A fillet that still breaks up was probably overcooked, so take it out the moment the thickest part just flakes.".into());
        }
        return Some("Pat the fillet dry with kitchen paper and get the pan properly hot before it goes in: wet fish on a cool pan steams instead of sealing, and that is what makes it break up. Lay it in skin side down, press it flat for ten seconds, then leave it alone for four minutes - it releases itself when the skin has set - turn once and give it two more, and take it out the moment the thickest part just flakes. Baking at 200C (180C fan) for 12 to 15 minutes on baking paper is the gentler way, and a fish stew is meant to break up: stir it in at the end.".into());
    }
    if names(&["meatballs", "meatball", "kofta", "koftas", "rissoles", "falafel", "sausage meat", "meatloaf", "patties", "patty"]) {
        return Some("Something ground and shaped needs a binder and a rest: one egg and a handful of breadcrumbs per 500g, mix it just until it holds - overworking makes it springy - then chill it 20 minutes before it meets the pan, because cold holds together and warm does not. Sear it in a hot pan without moving it for two minutes a side so a crust forms, then finish it in a 200C oven rather than turning it again. Too lean a mince is the other cause: 20% fat is what keeps it in one piece.".into());
    }
    if names(&["tofu", "paneer", "halloumi", "tempeh"]) {
        return Some("Press the water out first: wrap the block in a clean tea towel with a weight on it for 15 minutes, then dust it in cornflour before it goes in the pan. That dry coat is what holds a crust together, and a hot pan with a little oil that you leave alone for three minutes does the rest. In a stir-fry, add it last and fold rather than stir.".into());
    }
    if names(&["potatoes", "potato", "mash"]) {
        return Some("Floury potatoes break up and waxy ones hold their shape, so buy the right one: Maris Piper or King Edward for mash and roast, Charlotte or Jersey Royal for a salad or a tray of boiled ones. Cut them the same size, cook them from cold water at a bare simmer rather than a rolling boil, and drain them in a colander for a minute to steam dry before they go in the oven.".into());
    }
    if names(&["dumplings", "dumpling", "gnocchi"]) {
        return Some("A dumpling falls apart when the dough is too wet or the pot is at a rolling boil: use 100g of suet to 200g of self-raising flour with just enough cold water to bring it together, drop them onto the stew, put the lid on and keep it at a bare simmer for 20 minutes without lifting the lid. Gnocchi want the same gentle heat, and they are done a minute after they float.".into());
    }
    if names(&[
        "cake", "cakes", "sponge", "bread", "loaf", "loaves", "pastry", "scone", "scones", "biscuit", "biscuits", "cookie",
        "cookies", "brownie", "brownies", "muffin", "muffins", "traybake", "yorkshire pudding", "yorkshires", "burger",
        "burgers", "rice", "peas", "pasta", "noodles", "omelette", "pancake", "pancakes",
    ]) {
        return None;
    }
    if names(&["chicken", "turkey", "lamb", "beef", "pork", "roast", "joint", "shoulder", "brisket", "ham", "gammon"]) {
        return Some("Coming apart is how you know it is done: a shoulder, a brisket or a chicken thigh is ready when it pulls apart with no resistance, and a whole bird is done at 74C in the thickest part of the thigh. If it is coming apart before it is cooked through, the heat was too high - turn it down to 150C (130C fan), add a splash of water to the tin and give it another hour covered, because slow and low is what turns the connective tissue to gelatine rather than drying the meat out.".into());
    }
    if names(&["stew", "casserole", "tagine", "goulash", "braise", "pulled"]) {
        return Some("Nothing has gone wrong: a stew, a braise and a slow-cooked shoulder are meant to come apart, and that is the sign the connective tissue has turned to gelatine. A shoulder at 150C takes 3 to 4 hours; if it is coming apart before you want it to, take it off the heat 20 minutes early and let it sit covered, or lift the pieces out and reduce the liquid on its own.".into());
    }
    if any_in(&whole, &_COMING_APART_ON_PURPOSE) {
        return Some("Nothing has gone wrong if it is a stew, a braise or a slow-cooked shoulder: they are meant to come apart, and that is the connective tissue turning to gelatine. A shoulder at 150C takes 3 to 4 hours. If it is breaking up before you want it to, take it off the heat early and let it sit covered.".into());
    }
    if any_same(&words, &_MUSHY_OWN_ROW) {
        return None;
    }
    if why {
        return Some("It is nearly always one of two things: it was cooked too long, so what held it together gave up, or it was turned and moved before it had a crust to hold onto. Take it off the heat a little earlier next time and leave it alone in the pan until it releases itself.".into());
    }
    Some("Handle it less and cook it a little less: food comes apart when it is overcooked, or when it is moved in the pan before it has sealed. Dry the surface, get the pan hot, leave it alone until it releases itself, and turn it once. If it is already in pieces, a bake, a soup or a sauce is the way to serve it rather than a plate.".into())
}

lazy!(_DOUGH_NOT: Vec<String> = "_DOUGH_NOT");
lazy!(_DOUGH_WET: Vec<String> = "_DOUGH_WET");
lazy!(_DOUGH_DRY: Vec<String> = "_DOUGH_DRY");

/// A dough too wet or too dry: which way to go, read from the last fault word and its negation.
pub fn _dough(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let mut direction = "";
    for (index, word) in words.iter().enumerate() {
        let negated = words[index.saturating_sub(2)..index].iter().any(|e| _DOUGH_NOT.contains(e) || e.ends_with("n't"));
        if _DOUGH_WET.contains(word) {
            direction = if !negated { "more flour" } else { "more water" };
        } else if _DOUGH_DRY.contains(word) {
            direction = if !negated { "more water" } else { "more flour" };
        }
    }
    if direction == "more flour" {
        return Some("Add flour a spoonful at a time and knead it in before you judge, because a dough is wettest in the first few minutes: it should cling to your hand and come away clean. Wetter is better for an open crumb, so stop as soon as it stops running and keep a little of the water back next time, since flour alone can be 10% different in how much it drinks.".into());
    }
    Some("Add water a spoonful at a time and knead a minute after each one: a stiff dough reads drier than it is, because the flour takes a few minutes to take up what is already in there, and you only feel the difference after kneading. A bread dough should be tacky rather than sticky - it clings to your hand but comes away clean - and it is ready when a piece stretches thin enough to see light through without tearing. If you overshoot, a spoonful of flour brings it back.".into())
}

// ---------------------------------------------------------------------------------------------
// Watery, lumpy, eggs, pasta, popcorn, tough, salty

/// Whether the words name one of these (by `_same`, or a spaced name inside the line).
fn names_table(words: &[String], said: &str, names: &[String]) -> bool {
    any_same(words, names) || names.iter().any(|w| w.contains(' ') && said.contains(w.as_str()))
}

/// Something came out watery: the fix for the dish it is, or "" for a wet line that is no dish.
pub fn _watery(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let all_words = lower_words(understood);
    let said = all_words.join(" ");
    let words: Vec<String> = all_words.into_iter().filter(|w| !said.contains(&format!("{w} milk"))).collect();
    if any_in(&said, &strs("_PLANT_MILK")) && !any_in(&said, &strs("_PLANT_MILK_DISH")) {
        return Some("A carton of oat, almond or soy milk that pours thin has separated: shake it hard for half a minute before you open it and it comes back together, and a cheap barista one holds up better in coffee. Homemade oat milk is thin by nature - blend 1 part oats to 4 of water, strain it lightly, and add a pinch of salt; it thickens as it stands in the fridge.".into());
    }
    let hit = |table: &str| names_table(&words, &said, &strs(table));
    if hit("_MASH_DISH") {
        return Some("Mash goes watery when the potatoes come out of wet water or the milk goes in cold: drain them, put the pan back on a low heat for a minute to steam the rest off, then mash with warm milk and butter. If it has already gone loose, warm it through in the pan and stir rather than adding anything - it firms up as the extra water steams away.".into());
    }
    if hit("_CHICKPEA_WET") {
        return Some("A hummus goes watery from water or ice going in with the chickpeas, or from too much lemon: drain the tin well, add a spoon more tahini or a few more chickpeas and blitz again, and serve it chilled, because it thickens as it sits. Loosen it later with oil, not water.".into());
    }
    if hit("_OLIVE_DIP_WET") {
        return Some("Tapenade goes watery from olives that came out of the brine wet, or from too much oil: drain them, dry them on a tea towel, and pound or pulse them with the capers and anchovy before the oil goes in. A thin tapenade wants more olives and a few capers, not less oil, and it firms up in the fridge.".into());
    }
    if hit("_GREEN_DIP_WET") {
        return Some("Pesto goes watery from basil that was still wet, or from too much oil in one go: dry the leaves in a tea towel, pound or pulse them with the nuts and cheese first and add the oil last, a little at a time. A thin pesto in the jar wants more grated cheese and nuts rather than less oil.".into());
    }
    if hit("_DIP_DISH_WET") {
        return Some("A dip goes watery from water in the base: drain the tin or squeeze out the grated vegetable, then thicken it with a spoon of the same solid - chickpeas, yogurt, cream cheese - and taste it again, because thinning it dilutes the salt and the acid too.".into());
    }
    if hit("_AVOCADO_WET") {
        return Some("Guacamole goes watery from over-ripe avocado or too much lime: mash it with a fork rather than a blender, drain any liquid off and stir in a little more avocado, and leave the stone in if you are keeping it - it slows the browning, not the water.".into());
    }
    if hit("_BAKE_WET") {
        return Some("A bake goes watery when the sauce was thin before it was layered, or the vegetables still had water in them. Next time drain and squeeze the veg and let the sauce come to a simmer with a spoon of cornflour slaked in cold water before it goes in. For this one, give it 10 to 15 minutes uncovered at the same heat, then rest it 10 minutes before you cut it, and the sauce will firm up.".into());
    }
    if hit("_DUMPLING_WET") {
        return Some("A dumpling mix that is too wet wants plain flour a spoonful at a time until it stops sticking to the bowl, then shape them with wet hands and drop them onto a stew that is simmering, put the lid on and give them 20 minutes without lifting it. Too wet and they spread out into the gravy instead of sitting on top.".into());
    }
    if hit("_FRY_MIX_WET") {
        return Some("A mix that will not hold together is too wet to fry: squeeze grated onion or veg in a clean tea towel, stir in a spoonful of plain or gram flour, and chill it for 30 minutes so the starch takes up the water. Make sure the oil is properly hot before the first one goes in - a cool pan is what turns a soft mix into a pancake.".into());
    }
    if hit("_BREAD_DOUGH_WET") {
        return _dough(memory, understood);
    }
    if hit("_BATTER_WET") {
        return Some("A batter that is too wet wants flour, not heat: whisk in a spoonful at a time until it coats the back of the spoon, then let it rest 20 minutes so the flour takes up the liquid. Too thin and it will not hold the bubbles, so pancakes come out pale and flat rather than risen, and a cake will not hold its rise either.".into());
    }
    if hit("_PORRIDGE_WET") {
        return Some("Porridge that has gone too loose wants a minute more on the heat with the lid off and a stir, and less milk next time: 40g of oats to 300ml of milk or water is enough for one, and it thickens as it stands. Do not add more oats once it is cooking, because they will not soften in time.".into());
    }
    if hit("_RISOTTO_WET") {
        return Some("Risotto that will not thicken wants more stirring, not more heat: keep it moving and add the stock a ladle at a time, waiting for each one to be taken up, then beat in cold butter and grated cheese off the heat at the end - that is what makes it creamy and loose at the same time.".into());
    }
    if hit("_COUSCOUS_WET") {
        return Some("Couscous goes watery from too much water: pour it into a wide dish, fork it through to break up the clumps and leave it uncovered for 5 minutes so the steam escapes. Next time use the same volume of couscous and boiling water, cover it for 5 minutes, then fork it with a knob of butter rather than stirring it.".into());
    }
    if hit("_PASTRY_WET") {
        return Some("A quiche or a pie goes watery when the filling is too wet or the egg is undercooked: give it 5 to 10 minutes more in a 180C oven and it will set, and next time cook the vegetables or the fruit down first and let them cool, then blind-bake the case before the filling goes in.".into());
    }
    if hit("_PASTRY_DOUGH_WET") {
        return Some("Pastry is a dough, not a filling: if it is wet and sticky, work in a little more flour a spoon at a time, wrap it and give it 20 minutes in the fridge, and roll it between two sheets of baking paper rather than dusting it with more flour. Shortcrust wants about half its weight in fat to flour, and cold hands.".into());
    }
    if hit("_RICE_WET") {
        return Some("Rice goes watery from too much water or the lid coming off: tip it into a sieve, then back in the pan with the lid on and 5 minutes on the lowest heat to steam dry. Next time one and a half parts water to one of rice, lid on, and leave it alone.".into());
    }
    if hit("_WET_CUSTARD") {
        return Some("Custard that is thin has not had enough heat: stir a teaspoon of cornflour into a splash of cold milk, whisk it in and give it 2 more minutes on a low heat until it coats the back of the spoon. It also thickens as it cools, so take it off before it looks set rather than after.".into());
    }
    if hit("_WET_ICING") {
        return Some("Icing that is too thin wants icing sugar, a spoon at a time, beaten in until it holds a soft peak: a teaspoon of water or milk goes a long way, so add that in drops. A glaze is meant to be thin - it should coat the back of the spoon and run off in ribbons, which is a different job from icing a cake.".into());
    }
    if hit("_WET_DRINK") {
        return Some("A thin smoothie or shake wants less liquid and something solid: half a banana, a spoon of oats or yoghurt, or a handful of frozen fruit, blended again - the fruit and the ice are what give it body. Pour the milk or juice in last, a splash at a time, instead of all at the start.".into());
    }
    if hit("_WET_YOGHURT") {
        let pot = "A pot of yoghurt that has gone watery has separated, and that is harmless: stir it back together, or spoon the whey off the top, or strain it through muslin or a clean tea towel in a sieve for an hour for a thicker pot.";
        let dip = "A yoghurt dip goes watery because the vegetables in it were wet: salt the grated cucumber or carrot for 10 minutes, squeeze it out in a tea towel, and fold it into thick yoghurt at the end - a spoon of cream cheese or labneh brings back one that has already split.";
        if any_in(&said, &strs("_WET_DIP_WORDS")) {
            return Some(format!("{dip} {pot}"));
        }
        return Some(format!("{pot} {dip}"));
    }
    if hit("_WET_DAL") {
        return Some("A thin dal wants the lid off for 10 more minutes, stirring now and then: it thickens as the lentils break up, and mashing a ladleful against the side of the pan and stirring it back in thickens it at once. Add water a splash at a time at the end, because dal sets as it cools.".into());
    }
    if hit("_WET_SLAW") {
        return Some("A watery salad is nearly always salt pulling the water out of the vegetables: salt the shredded cabbage or the cucumber for 10 minutes, squeeze it in a clean tea towel, and dress it just before it goes to the table. Mayo loosened with a splash of vinegar coats better than a thick spoonful.".into());
    }
    let pan_food = any_word(
        &words,
        &[
            "sauce", "sauces", "stew", "curry", "chilli", "chili", "gravy", "ragu", "bolognese", "bolognaise", "daal", "dhal",
            "dal", "tagine", "goulash", "casserole", "cassoulet", "broth", "stock", "soup", "soups", "jus",
        ],
    );
    let thin = pan_food
        || any_word(&words, &["watery", "runny", "thin", "sloppy", "liquid", "thickens", "thicken", "reduce", "reduces"])
        || ["never thickens", "wont thicken", "won't thicken", "not thickening", "doesnt thicken", "doesn't thicken"]
            .iter()
            .any(|p| said.contains(p));
    if !thin && ["too wet", "a bit wet", "is wet", "went wet", "bit wet"].iter().any(|p| said.contains(p)) {
        return Some(String::new());
    }
    Some("Take the lid off and let it simmer for 10 to 15 minutes: most of the water goes off in the steam. To thicken it properly, stir in a spoon of tomato puree, a handful of red lentils, or cornflour slaked in cold water, and give it 5 more minutes.".into())
}

/// Lumpy mash, custard, porridge or sauce.
pub fn _lumpy(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    if words.iter().any(|w| _same(w, "mash") || w == "potato" || w == "potatoes") {
        return Some("Mash them while they're hot with plenty of butter and warm milk, and beat with a wooden spoon rather than a fork. Lumps come from cold potatoes or too little fat; if they're stubborn, push the mash through a sieve or a potato ricer.".into());
    }
    if words.iter().any(|w| _same(w, "custard")) {
        return Some("Take it off the heat and whisk hard, then push it through a sieve. Custard goes lumpy when the heat is too high or the eggs go in too fast, so next time pour the hot milk into the eggs a splash at a time and cook it slowly, stirring.".into());
    }
    if words.iter().any(|w| _same(w, "porridge") || _same(w, "oats")) {
        return Some("Beat in a splash more milk or water and whisk it hard; porridge lumps come from dry oats hitting hot liquid too fast. Next time stir the oats into cold liquid and bring it up slowly, stirring all the time.".into());
    }
    Some("Whisk it hard with the pan off the heat and push it through a sieve, because lumps break up far better off the ring. Next time stir the flour into the fat before any liquid goes in, then add the liquid a splash at a time.".into())
}

lazy!(_EGG_STORE: Vec<String> = "_EGG_STORE");
lazy!(_EGG_TEST_TEXT: String = "_EGG_TEST_TEXT");
lazy!(_EGG_STORE_TEXT: String = "_EGG_STORE_TEXT");

/// Eggs: the freshness test first when the line asks whether they are still good.
pub fn _eggs(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let testing = any_word(
        &words,
        &[
            "fresh", "off", "bad", "rotten", "float", "test", "edible", "safe", "date", "dated", "expired", "expiry", "ok", "okay",
            "fine", "use", "usable", "eat", "eaten", "yesterday", "week", "weeks", "old",
        ],
    );
    if testing && !words.iter().any(|w| _EGG_STORE.contains(w)) {
        return Some(format!("{}{}", *_EGG_TEST_TEXT, text::strip(&_EGG_STORE_TEXT)));
    }
    Some(format!("{}{}", *_EGG_STORE_TEXT, text::strip(&_EGG_TEST_TEXT)))
}

lazy!(_CLUMP_WORDS: Vec<String> = "_CLUMP_WORDS");

/// Cooking pasta, with the clumping note when the line says it clumps.
pub fn _pasta_cook(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let mut text_ = String::from("Use a big pan of well-salted water, about 10g of salt per litre, and cook the pasta one minute less than the packet says. Keep a mug of the cooking water: a splash of it makes any sauce cling to the pasta.");
    if words.iter().any(|w| _CLUMP_WORDS.contains(w)) {
        text_.push_str(" Clumping is the pasta sitting still in water that is not deep or hot enough, so stir it for the first minute and again every couple of minutes, and leave the oil out: it stops the sauce clinging.");
    }
    Some(text_)
}

/// Making popcorn: in a pan, the microwave, or without oil.
pub fn _popcorn_make(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let pan = "Three kernels and a tablespoon of oil in a big lidded pan over medium heat, and when those three pop, add a single layer of kernels and put the lid back on. Shake the pan every so often and take it off the heat when the pops slow to one every few seconds, about 3 minutes for four tablespoons of kernels. Tip it into a bowl, salt it while it is hot, and leave the lid off so the steam gets away and it stays crisp.";
    let microwave = "Put a quarter of a cup of kernels in a paper bag or a bowl with a plate over it and give it 2 to 3 minutes on full power, stopping the moment the pops slow to one every few seconds; a bag is the easy way because the steam escapes. Salt it while it is hot and leave the lid off the bowl so it stays crisp.";
    let dry = "Dry kernels pop without any oil: shake a single layer of them in a heavy lidded pan over medium heat, lifting the pan off the ring every few seconds so the bottom does not scorch, until the pops slow. Or use an air popper, which is the same thing with the stirring done for you.";
    if any_same(&words, &["microwave", "microwaved", "microwaving", "microwaves", "bag", "bags"]) {
        if words.iter().any(|w| w == "hob" || w == "pan") {
            return Some(format!("{microwave} {pan}"));
        }
        return Some(microwave.into());
    }
    if any_same(&words, &["oil", "air", "popper", "diet", "healthy"]) {
        return Some(dry.into());
    }
    Some(pan.into())
}

/// Keeping popcorn, or why it came out chewy.
pub fn _popcorn_keep(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    if any_same(&words, &["chewy", "soggy", "squidgy", "tough", "rubbery"]) {
        return Some("Chewy popcorn was either stale to start with or it did not get hot enough: use kernels under a year old, keep them in a sealed jar rather than the fridge, and give the pan a few more seconds after the pops slow to one every few seconds. Leave the lid off the bowl once it is out, because the steam it sits in is what turns the rest soft.".into());
    }
    Some("Kernels keep for about a year in a sealed jar in a cool cupboard, and popped corn goes soft within a day or two, so an airtight tin is what keeps it crisp. Popcorn that comes out chewy was stale to start with or did not get hot enough: use kernels under a year old and give the pan a few more seconds after the pops slow.".into())
}

/// Stewed fruit that stays hard: the fruit's own reason.
pub fn _stewed_fruit(words: &[String]) -> String {
    let mut named = "fruit".to_string();
    let mut why = "a firm one holds its shape where a ripe one collapses, so use the ripe ones for a compote".to_string();
    for row in g("_STEWED_WHY").items() {
        let row = row.items();
        if any_same(words, &row[0].strs()) {
            named = row[1].str().to_string();
            why = row[2].str().to_string();
            break;
        }
    }
    format!(
        "A stewed {named} that stays hard was the fruit rather than the cooking: {why}. Give it 10 more minutes with a splash of water and taste as it goes, because it goes from firm to pulp quickly once it starts."
    )
}

lazy!(_STEWED_FRUIT: Vec<String> = "_STEWED_FRUIT");

/// Something tough or rubbery: the fix for the thing it is.
pub fn _tough(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    if words.iter().any(|w| _same(w, "squid")) {
        return Some("Squid is either 2 minutes or 40: flash it in a very hot pan, or stew it slowly with tomatoes and wine for 40 minutes. Anything in between turns it to rubber.".into());
    }
    if any_same(&words, &_STEWED_FRUIT) && any_same(&words, &["stewed", "stew", "poached", "compote", "baked", "puree"]) {
        return Some(_stewed_fruit(&words));
    }
    if any_same(&words, &["stew", "stews", "casserole", "braise", "braised", "braising", "goulash", "hotpot", "tagine", "ragu", "chilli", "chili"])
        && !any_same(&words, &_STEWED_FRUIT)
    {
        return Some("A tough stew is an underdone one rather than an overcooked one: another 30 to 45 minutes at a bare simmer with the lid on, and top the liquid up if it is running low. A simmer that is really a boil tightens the meat instead of softening it, so keep it just trembling and test a piece with a fork - it is ready when it gives way without resistance.".into());
    }
    if any_same(
        &words,
        &[
            "steak", "meat", "beef", "pork", "lamb", "chicken", "chop", "chops", "turkey", "sausage", "sausages", "burger",
            "burgers", "roast", "gammon", "duck", "ribs",
        ],
    ) {
        if _asks_why(&words) {
            return Some("It went tough from cooking too fast or from not resting: the muscle fibres contract and squeeze the juice out, and a cheap cut needs longer at a lower heat to break down instead - 2 hours at 160C with liquid in the pan. This piece is best sliced thinly across the grain and served with something wet.".into());
        }
        return Some("Slice it thinly across the grain and serve it with something wet; that rescues more tough meat than anything else. It went tough from cooking too fast or not resting, so next time go slower: a cheap cut wants 2 hours at 160C with liquid in the pan.".into());
    }
    if any_same(&words, &["pasta", "noodle", "noodles", "rice"]) {
        return Some("Tough here is underdone rather than overcooked: give it 5 more minutes in boiling water and test it with the tip of a knife, and cut the pieces the same size so they finish together. A pan that never came back to the boil is the usual reason it is still hard in the middle when the outside is done.".into());
    }
    if any_same(
        &words,
        &[
            "cake", "cakes", "sponge", "sponges", "muffin", "muffins", "cupcake", "cupcakes", "scone", "scones", "brownies",
            "brownie", "bun", "buns", "traybake", "traybakes", "loaf", "loaves",
        ],
    ) {
        let (thing, plural) = _the_bake_named(&words);
        let head = if !thing.is_empty() { format!("Tough {thing}{}", if plural { " are" } else { " is" }) } else { "A tough bake is".to_string() };
        return Some(format!("{head} mixed too much or a little too long in the oven, or short of fat. Stir only until the flour disappears, fold rather than beat, and weigh the butter rather than eyeballing it, because a sponge wants as much fat as eggs by weight. One that is already tough is best eaten warm with something wet, or torn up for a trifle."));
    }
    if any_same(&words, &["pastry", "shortcrust", "shortcrust pastry", "pie crust", "pie", "tart", "quiche", "crumble"]) {
        return Some("Tough pastry is over-worked or too wet: keep the fat cold, rub it in with your fingertips only until it looks like fine breadcrumbs, add iced water a spoon at a time and bring it together fast. Shortcrust wants about half its weight in fat to flour, and 20 minutes in the fridge before it is rolled; roll it between two sheets of greaseproof so you never stretch it. Pastry that is already baked cannot be rescued, so serve it with something wet.".into());
    }
    if any_same(&words, &["bread", "loaf", "roll", "rolls", "baguette", "pitta", "naan", "bun", "buns", "bagel", "bagels"]) {
        return Some("Tough bread is usually over-kneaded or baked too slowly: knead until the dough springs back and stops tearing, prove it somewhere warm until it has doubled, and bake it hot (220C) for the first 10 minutes so it rises before the crust sets. Bread that has already gone leathery comes back in a hot oven for 5 minutes with a splash of water in the tin.".into());
    }
    if any_same(&words, &["egg", "eggs", "omelette", "scrambled"]) {
        return Some("Rubbery eggs have been cooked too long, and in a microwave that happens in seconds: beat them, give them 30 seconds at a time on medium, stir hard between each go, and stop while they still look a little wet, because they carry on setting as they stand. The same rule in a pan - low heat, slow stirring, off the heat early.".into());
    }
    Some("Rubbery is overcooked, or a batter that was beaten too much: pancakes and batters want a lumpy mix and a minute less in the pan. It's hard to rescue on the plate, so serve it with something wet and cook it less next time.".into())
}

lazy!(_BAKE: Vec<String> = "_BAKE");

/// Too salty: bulk it out, or for a bake balance it.
pub fn _salty(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    if any_same(&words, &_BAKE) {
        return Some("Baking is the one place salt cannot be cooked out or bulked out: leave it and cut the salt from whatever goes with it, or add a little more sugar and fat to push it into the background. Next time weigh the salt with the measuring spoons rather than shaking it in, and remember salted butter is already about a quarter salt.".into());
    }
    Some("Don't add water: bulk it out with more of what's already in it — potatoes, rice, tinned tomatoes, cream or unsalted stock — and finish with a squeeze of lemon. In a soup or stew, a peeled potato simmered in it for 15 minutes takes up some of the salt too.".into())
}

/// Scrambled eggs: the method, or the fix for watery or rubbery ones.
pub fn _scramble(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let asks_how = understood.kind == "question" || words.iter().any(|w| _same(w, "how"));
    if asks_how
        && !any_same(&words, &["watery", "wet", "sloppy", "rubbery", "rubber", "tough", "dry", "hard", "overcooked", "chewy", "curdled", "mushy"])
    {
        return Some("Beat 2 or 3 eggs with a pinch of salt, melt a knob of butter in a pan over a low heat, pour them in and stir slowly with a spatula, pushing the set egg in from the edges. Take them off while they still look a little wet, after 2 or 3 minutes, and stir in a little more butter off the heat.".into());
    }
    if any_same(&words, &["watery", "wet", "sloppy"]) {
        return Some("Watery scrambled eggs have been cooked too long: the curds squeeze out their water as they tighten. Cook them low and slow, stirring, and take them off while they still look a little loose, then stop the heat with a knob of cold butter or a spoon of creme fraiche.".into());
    }
    Some("Rubbery scrambled eggs come from too much heat: cook them low and slow, stirring, and take them off while they still look slightly wet, because they carry on setting. A knob of butter at the end stops them going dry.".into())
}

/// An egg dish gone wrong: a split sauce, a rubbery quiche, a fried egg, an omelette.
pub fn _egg_fault(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    if words.is_empty() {
        return None;
    }
    let said = words.join(" ");
    let named = |table: &str| any_in(&said, &strs(table));
    let fault = |names: &[&str]| any_same(&words, names) || names.iter().any(|n| said.contains(n));
    if named("_EGG_SAUCE") && fault(&["grainy", "gritty", "split", "splits", "curdled", "curdles", "lumpy", "scrambled"]) {
        return Some("A grainy or split creme anglaise has curdled, because the eggs went above about 82C. Take it off the heat at once, whisk in a splash of cold milk to stop it cooking, then strain it through a sieve and it comes back smooth. Next time cook it over a low heat, stirring all the time, and pull it off as soon as it coats the back of the spoon.".into());
    }
    if named("_EGG_SAUCE") {
        return None;
    }
    if named("_EGG_QUICHE") {
        if fault(&["rubbery", "rubber", "tough", "dense", "overcooked", "over cooked"]) {
            return Some("A rubbery quiche is a custard that stayed in too long: the filling is firm at about 80C, so bake it at 180C for 30 to 35 minutes, until the middle still has a slight wobble, and let it cool for 20 minutes, because it firms as it stands. Two eggs to 200ml of double cream in a 20cm tin sets soft; more egg than that sets like rubber.".into());
        }
        return None;
    }
    if named("_EGG_FRIED") {
        return Some("Fried eggs go rubbery when the white sets hard before the yolk is warm: cook them on a medium heat with a lid on for 2 to 3 minutes, so the top sets in the steam, and take them out while the yolk still wobbles. On a high heat the white blisters and toughens long before the yolk is hot.".into());
    }
    if named("_EGG_FRITTATA") || named("_EGG_PAN") {
        if fault(&["fell apart", "falls apart", "falling apart", "wouldnt fold", "wont fold", "broke", "tears", "tore"]) {
            return Some("An omelette tears when the eggs are too thin for the pan, so use a 20cm pan and no more than three eggs, let the base set for 45 seconds before you fold, and fold it with a spatula in the pan instead of lifting it out. A frittata is finished under the grill and never folded, which is the same dish without the part that breaks.".into());
        }
        if fault(&["soggy", "watery", "wet"]) {
            return Some("An omelette or frittata that stays wet is a vegetable problem: mushrooms, courgette, spinach and onion all give out water as they cook. Fry them first, on their own, until their water has gone and they are starting to colour, and let them cool a little before the eggs go in. A pinch of salt in the beaten egg also helps it set.".into());
        }
        if fault(&["rubbery", "rubber", "tough", "leathery", "overcooked", "over cooked", "chewy"]) {
            return Some("An omelette goes rubbery from the pan and the clock rather than the eggs: a 20cm pan on a medium heat, two beaten eggs with a pinch of salt, and pull the set edges into the middle for 60 to 90 seconds, then fold and slide it out while the top still looks a little wet. A small pan steams it instead of setting it, and one more minute is enough to make it leathery.".into());
        }
        return None;
    }
    None
}

/// Cookies that spread too far, or not at all.
pub fn _cookies(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = lower_words(understood).join(" ");
    let no_spread = [
        "didnt spread", "didn't spread", "did not spread", "not spread", "not spreading", "wont spread", "won't spread",
        "stayed in a ball", "stayed in balls", "too thick", "too small", "didnt flatten", "didn't flatten",
        "did not flatten", "stayed round",
    ];
    if no_spread.iter().any(|p| said.contains(p)) {
        return Some("Cookies that stay in a ball have usually had too much flour or butter that was too cold: weigh the flour rather than scooping the cup, use soft butter, and press each ball down to about 1cm with the back of a spoon before it goes in. A tablespoon of milk or an extra yolk loosens a dry dough, and the oven wants to be properly hot, 180C (160C fan), so they set before they can spread.".into());
    }
    Some("Cookies spread when the dough is warm or the butter is too soft: chill the dough 30 minutes and put it on a cool tray, and leave a good 5cm between them. Too much sugar or a greased tin makes them spread further, and a fan oven needs 20C less than the recipe.".into())
}

re_lazy!(WHITE_BEFORE_CHOCOLATE = r"\bwhite\s+(?=chocolate\b)");

/// Chocolate: bloom, health, what it is, tempering, dipping, seizing or melting.
pub fn _chocolate(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let said = words.join(" ");
    let told_apart = text::split(&WHITE_BEFORE_CHOCOLATE.sub("", &said));
    if any_same(&told_apart, &["bloom", "white", "streak", "streaks", "grey", "gray"]) {
        return Some("White streaks on chocolate are bloom, and it is harmless: either cocoa butter or sugar has come to the surface because the chocolate got warm and then cool again. It still tastes the same and it is fine to eat or to melt. To stop it happening, store it cool and steady, around 15 to 18C, and never in the fridge.".into());
    }
    let health = ["healthy", "good for you", "bad for you", "better for you", "fattening", "sugar", "calorie", "calories", "diet"];
    if health.iter().any(|p| said.contains(p)) {
        return Some("As a treat, chocolate is fine: dark with 70% cocoa or more has the least sugar, so it is the one to pick if you are watching it, and two squares is a normal portion. White chocolate is the sweetest of the three because it is cocoa butter, milk and sugar with no cocoa solids at all.".into());
    }
    let what = [
        "real chocolate", "is it chocolate", "actually chocolate", "made of", "made from", "contain", "contains",
        "what is chocolate", "is white chocolate", "whats in chocolate", "what's in chocolate",
    ];
    if what.iter().any(|p| said.contains(p)) {
        if words.iter().any(|w| w == "white") {
            return Some("White chocolate is cocoa butter, milk powder and sugar, with no cocoa solids at all, which is why it tastes of vanilla and milk rather than of chocolate. It is still real chocolate by the labelling rules - at least 20% cocoa butter in Europe - just a different thing from dark, and it is the sweetest of the three.".into());
        }
        return Some("What most people mean by chocolate is dark: cocoa solids and cocoa butter with sugar, and the higher the percentage the less sugar there is. Milk chocolate adds milk powder and more sugar, and white chocolate is cocoa butter, milk and sugar with no cocoa solids at all.".into());
    }
    if words.iter().any(|w| _same(w, "temper")) {
        return Some("To temper chocolate, melt two thirds of it in a bowl over barely simmering water to 45C (or 40C for white), take it off the heat and stir in the rest, chopped, until it drops to 27C, then warm it gently back to 31C for dark or 29C for milk and white. That sets it glossy and hard at room temperature. A thermometer earns its keep; without one, a clean spoon dipped in should set firm in 5 minutes.".into());
    }
    if any_same(&words, &["dip", "dipped", "dipping", "strawberry", "strawberries"]) && !any_same(&words, &["melt", "melting", "melted"]) {
        return Some("Melt it slowly, then dip: dry the fruit first so the chocolate grips, dip about two thirds of the way in, let the extra drip back into the bowl, and set them on baking paper in the fridge for 10 minutes to set hard. If the chocolate feels thin, stir in a handful more chopped chocolate off the heat.".into());
    }
    if any_same(&words, &["seize", "seized", "seizing", "grainy", "lumpy", "stiff"]) {
        return Some("Chocolate seizes when even a drop of water gets in, and the fix is more water, not less: stir in a teaspoon of boiling water at a time, off the heat, until it goes smooth again. A tablespoon of warm cream does the same and leaves you with a ganache.".into());
    }
    Some("Melt chocolate in a bowl over barely simmering water, with the bowl not touching the water, or in 20-second bursts in the microwave, stirring between them. Keep every spoon and bowl dry, because a drop of water makes it seize, and take it off while most of it is melted - the rest will go as you stir.".into())
}

// ---------------------------------------------------------------------------------------------
// Sugar, soup, flat bakes, frozen meat, turkey, joints, mince

/// Grainy icing, fudge, custard, caramel or sugar.
pub fn _grainy(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let has = |names: &[&str]| any_same(&words, names);
    if has(&["icing", "frosting", "buttercream", "glaze", "fondant", "drizzle"]) {
        return Some("Grainy icing is undissolved sugar: sift the icing sugar first, then add the liquid a teaspoon at a time, and if it is already made, beat in a spoonful of warm milk or a few drops of lemon juice and it goes smooth. Cold liquid and coarse sugar are what leave the grain, so warm the milk slightly next time and give it a full minute of beating.".into());
    }
    if has(&["fudge", "tablet", "praline", "brittle", "candy"]) {
        return Some("Grainy fudge has sugar crystals in it, which happens when it is stirred while it boils or a crust of sugar from the pan gets back in. Warm it very gently with two tablespoons of milk and beat it hard until it is smooth again, or press it out flat and call it tablet. Next time stir only until the sugar dissolves, then wipe the pan sides down with a wet pastry brush and leave it to 116C.".into());
    }
    if has(&["custard", "creme", "anglaise"]) {
        return Some("Grainy custard has curdled: the eggs got too hot too fast. Take it off the heat at once, whisk in a splash of cold milk to stop it cooking, then strain it through a sieve into a jug and it comes back smooth. Next time cook it over a low heat, stirring all the time, and take it off at about 82C or as soon as it coats the back of the spoon.".into());
    }
    if has(&["toffee", "caramel"]) {
        return Some("Grainy caramel or toffee has crystallised: add a tablespoon of water and warm it very gently without stirring until the crystals melt, then let it cool slowly. It crystallises when the sugar is stirred once it is boiling, or when one crystal from the side of the pan gets back in, so stir only until the sugar has dissolved and keep the sides of the pan wiped clean.".into());
    }
    Some("Sugar that sets grainy has crystallised, from stirring once it boils or a crystal from the pan getting back in. Warm it gently with a tablespoon of water until it melts back, and next time stir only until the sugar dissolves, wipe the pan sides with a wet pastry brush, and add a spoonful of golden syrup or a squeeze of lemon at the start.".into())
}

lazy!(_SUGAR_TROUBLE: Vec<String> = "_SUGAR_TROUBLE");
lazy!(_SUGAR_MAKING: Vec<String> = "_SUGAR_MAKING");

/// Sugar work: bitter caramel, a set bag, crystallising, dissolving, toffee and fudge, caramel.
pub fn _sugar_work(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let has = |names: &[&str]| any_same(&words, names);
    if has(&["bitter", "burnt", "burned", "burning", "smoky", "smells"]) {
        if _burnt_on_the_pan(&words.join(" ")) {
            return Some(String::new());
        }
        return Some("A bitter caramel has gone past the colour: sugar keeps cooking after it leaves the heat, so it went from amber to dark before it came off. Start again and pull it at the colour of a copper penny, and have the cream ready to add off the heat so it stops at once. One that tastes of the pan is not worth saving.".into());
    }
    if has(&["hard", "hardened", "hardens", "block", "solid", "lumps", "lumpy", "stuck", "soften", "softens", "softening"]) {
        return Some("Sugar that has set hard has taken up damp and dried out again: put the bag in the microwave for 10 seconds at a time, or stand a slice of bread or a peeled apple in the jar overnight, and it breaks up. Keep it in an airtight jar away from the steam of the kettle, because it is the damp that sets it into a block.".into());
    }
    let melting = ["dissolve", "dissolves", "dissolving", "dissolved", "melt", "melts", "melting", "melted"];
    if any_same(&words, &_SUGAR_TROUBLE) && !has(&melting) {
        return Some("Sugar crystallises when it is stirred once it is boiling, or when one grain from the side of the pan drops back in. Melt it back with a tablespoon of water over a low heat without stirring, and next time stir only until the sugar has dissolved, wipe the pan sides down with a wet pastry brush, and add a squeeze of lemon or a spoonful of golden syrup at the start.".into());
    }
    if has(&["syrup", "coffee", "cocktail", "cocktails", "tea", "dissolve", "dissolves", "dissolving", "dissolved", "melt", "melts", "melting", "melted"])
        || !any_same(&words, &_SUGAR_MAKING)
    {
        return Some("Sugar dissolves fastest in hot liquid: warm the water or milk first, stir until the liquid is clear rather than for a set time, and use about 200g of sugar to 100ml of water for a syrup. Caster sugar goes in quicker than granulated and icing sugar dissolves almost at once. A spoonful of lemon juice or golden syrup stops a syrup crystallising as it cools.".into());
    }
    if has(&["toffee", "toffees", "fudge", "praline", "brittle", "candy"]) {
        return Some("Caramel is the base of all of them, then the temperature sets what you get: 200g sugar in a dry heavy pan, swirled not stirred, to the colour of a copper penny. Toffee is that plus 60g butter and a splash of cream stirred in off the heat, cooked on to 150C (hard crack) and poured to set; fudge comes off at 116C (soft ball), then you beat it in the pan as it cools so it stays smooth instead of grainy. A sugar thermometer earns its keep here.".into());
    }
    Some("Caramel is sugar cooked until it changes colour, and nothing else: 200g sugar in a dry heavy pan over a medium heat, not stirred but swirled, until it is the colour of a copper penny - then straight off the heat, because it keeps cooking. For a sauce, add a splash of warm cream off the heat, as it spits. A wet caramel is 200g sugar with 60ml water and a squeeze of lemon, boiled without stirring.".into())
}

/// Soup: making it, or making a dull one better.
pub fn _soup(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    if any_same(&words, &["better", "flavour", "improve", "boring", "lifeless", "interesting"]) {
        return Some("Add salt first, then a squeeze of lemon or a splash of vinegar: most dull soup is short of both, not of spice. After that something savoury - a stock cube, a spoon of miso, a parmesan rind or a knob of butter - and roast the vegetables next time rather than boiling them.".into());
    }
    Some("Soften an onion and a carrot in butter with a pinch of salt, add whatever vegetables or pulses you have and enough stock to cover them, and simmer 20 minutes. Blend it smooth or leave it chunky, taste it, and finish with salt, pepper and a squeeze of lemon.".into())
}

/// Something that came out flat: pancakes, Yorkshires, scones or a bake.
pub fn _flat(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let has = |names: &[&str]| any_same(&words, names);
    if has(&["pancake", "pancakes", "batter", "crepe", "crepes"]) {
        return Some("Flat pancakes are usually old baking powder or a batter that was beaten too much: use a fresh tin, stir until it is just combined, and let the batter rest 10 minutes before it hits the pan. A properly hot pan matters too - the first one is always the test.".into());
    }
    if has(&["yorkshire", "puddings"]) {
        return Some("Yorkshire puddings want a smoking tin and cold batter: put a little oil in each hole, heat it in a 220C oven until it shimmers, pour the batter in and don't open the door for 20 minutes.".into());
    }
    if has(&["scon", "scones"]) {
        return Some("Flat scones come from handling: keep the butter cold, rub it in quickly, pat the dough out rather than rolling it hard, and cut straight down with a sharp cutter so the edges stay free to rise.".into());
    }
    Some("Something that came out flat is short of rise, because the baking powder was past its date, the flour was beaten after it went in, or the oven was not up to heat. Check the baking powder is in date (about half a teaspoon to 100g of flour), stop beating once the flour is in, and let the oven come fully up to heat before the tin goes in.".into())
}

lazy!(_FROM_FROZEN: Vec<String> = "_FROM_FROZEN");
lazy!(_FROZEN_WHAT: Vec<String> = "_FROZEN_WHAT");
lazy!(_FROZEN_TIME: Vec<String> = "_FROZEN_TIME");
lazy!(_FROZEN_ASK: Vec<String> = "_FROZEN_ASK");

/// True when the line asks whether something can be cooked straight from the bag.
pub fn _frozen_from_the_bag(understood: &Understood) -> bool {
    let text_ = text::squash(&understood.clean.to_lowercase());
    if ["turkey", "joint", "whole bird", "roasting"].iter().any(|w| text_.contains(w)) {
        return false;
    }
    if !any_in(&text_, &_FROM_FROZEN) {
        return false;
    }
    if any_in(&text_, &_FROZEN_WHAT) {
        return false;
    }
    if any_in(&text_, &_FROZEN_TIME) {
        return false;
    }
    if any_in(&text_, &_FROZEN_ASK) {
        return true;
    }
    ["from frozen", "cooked from frozen", "cook it frozen", "without defrosting", "without thawing", "no need to defrost", "skip the defrosting"]
        .iter()
        .any(|w| text_.contains(w))
}

/// Cooking from frozen: mince, a named cut, or small cuts in general.
pub fn _frozen_cook_reply(understood: &Understood) -> String {
    let text_ = text::squash(&understood.clean.to_lowercase());
    let mut words: Vec<String> = text::split(&text_);
    let cut_words = g("_FROZEN_CUT_WORDS").strs();
    if !cut_words.iter().any(|cut| words.contains(cut)) {
        // "can i just cook it from frozen" names no meat: the cut is in the turn before, and
        // only the cut words are taken from the subject the line leans on.
        if let Some(lean) = &understood.leans_on {
            let subject = text::split(&lean.subject.as_deref().unwrap_or("").to_lowercase());
            words.extend(subject.into_iter().filter(|w| cut_words.contains(w)));
        }
    }
    let led = if g("_FROZEN_CUTS").items()[0].strs().iter().any(|cut| words.contains(cut)) {
        "Mince can go straight into the pan from frozen: brown it over a high heat, break it up as it loosens and give it a few extra minutes before the sauce goes in.".to_string()
    } else {
        let mut found: Option<(String, bool)> = None;
        for row in g("_FROZEN_WHOLE").items() {
            let row = row.items();
            if words.iter().any(|w| w == row[0].str()) {
                found = Some((row[1].str().to_string(), row[2].truthy()));
                break;
            }
        }
        match found {
            Some((said, one)) if !said.is_empty() => {
                let template = st(if one { "_FROZEN_ONE" } else { "_FROZEN_MANY" });
                template.replacen("{}", &said, 1)
            }
            _ => "Usually yes for small cuts - mince, sausages, bacon, chops: they can go straight into a hot pan or a sauce from frozen, a few minutes longer than thawed and piping hot all the way through.".to_string(),
        }
    };
    format!("{led} A whole bird or a big joint is the exception: it has to thaw first, or the outside overcooks while the middle stays raw. Do not refreeze it once it has started to thaw.")
}

/// Defrosting meat: from frozen, a turkey, a joint, or small cuts.
pub fn _meat_defrost(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text_ = text::squash(&understood.clean.to_lowercase());
    if any_in(&text_, &_FROZEN_WHAT) && (text_.contains("frozen") || text_.contains("freezer")) {
        return None;
    }
    if _frozen_from_the_bag(understood) {
        return Some(_frozen_cook_reply(understood));
    }
    let small = ["mince", "sausage", "sausages", "chop", "chops", "steak", "steaks", "bacon", "liver", "kidney", "kidneys"];
    let has_small = small.iter().any(|w| text_.contains(w));
    if !has_small && (text_.contains("turkey") || text_.contains("bird")) {
        let weight = _bird_weight(understood);
        let time = match &weight {
            Some((kilos, _)) if *kilos != 0.0 => _hours_words(24.0 * kilos / 2.0),
            _ => "two and a half days".to_string(),
        };
        return Some(format!("A turkey wants {time} in the fridge. That is 24 hours for every 2kg: leave it in its wrapper in a tray on the bottom shelf and never on the worktop, and once it is thawed cook it within 2 days. A crown thaws in 24 hours."));
    }
    if !has_small && text_.contains("joint") {
        let weight = _bird_weight(understood);
        let hours = match &weight {
            Some((kilos, _)) if *kilos != 0.0 => format!("about {}", _hours_words(24.0 * kilos / 2.0)),
            _ => "a day for every 2kg".to_string(),
        };
        let said = match &weight {
            Some((_, s)) if !s.is_empty() => s.clone(),
            _ => "2kg".to_string(),
        };
        return Some(format!("Give it {hours} in the fridge. A {said} one goes in the night before: leave it in its wrapper in a tray on the bottom shelf, never on the worktop, and cook it within 2 days of thawing. A rolled or boned one is quicker than one on the bone."));
    }
    if !has_small {
        return Some("In the fridge, on a plate on the bottom shelf, never on the worktop or in warm water. A turkey or a big joint wants a day or two there; mince, sausages and chops thaw overnight, and the microwave's defrost setting is the shortcut if you are short of time - cook the meat straight away when you use it.".into());
    }
    Some("In the fridge, on a plate on the bottom shelf: mince, sausages and chops thaw overnight. If you are short of time use the microwave's defrost setting and cook the meat straight away, and never thaw it on the worktop or in warm water.".into())
}

/// The words and the typed line, lower-cased, as one string.
fn words_and_typed(understood: &Understood) -> String {
    let mut parts = lower_words(understood);
    parts.push(understood.text.to_lowercase());
    parts.join(" ")
}

/// Roasting a whole turkey (from its weight) or a crown.
pub fn _turkey(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = words_and_typed(understood);
    if !said.contains("crown") {
        let (kilos, weight_said) = _bird_weight(understood).unwrap_or((5.0, "5kg".to_string()));
        return Some(format!(
            "A whole turkey roasts at 180C (160C fan) for 20 minutes per kg plus 70 minutes: a {weight_said} bird takes about {}. Cook the stuffing in a separate tray, because a stuffed bird needs longer and the middle of the stuffing is the last thing to get hot. Check the thigh at the deepest point, not the breast, for 74C, and rest it 30 to 45 minutes under foil before carving.",
            _in_words(20.0 * kilos + 70.0)
        ));
    }
    Some("Roast a turkey crown at 180C (160C fan) for 20 minutes per 500g plus 20 minutes, until a thermometer in the thickest part reads 74C, then rest it 30 minutes under foil. Salt it the night before or push butter under the skin, and cover the top with foil if it browns too fast.".into())
}

/// "so a 2kg one is about 1 hour 40 minutes." from the weight in the line, or the example.
pub fn _cut_time(understood: &Understood, per_kilo: f64, extra: f64, example: &str) -> String {
    match _bird_weight(understood) {
        None => example.to_string(),
        Some((kilos, said)) => format!("so a {} one is about {}.", said, _in_words(kilos * per_kilo + extra)),
    }
}

/// A cut of turkey: spatchcock, wings, drumsticks, legs, thighs, fillets or a breast.
pub fn _turkey_cut(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = words_and_typed(understood);
    let has = |words: &[&str]| words.iter().any(|w| said.contains(w));
    let done = "Take it to 74C in the thickest part and rest it 10 minutes under foil.";
    let dark = "Take the thickest part to 80C, where dark meat goes tender rather than tight, and rest it 10 minutes under foil.";
    if has(&["spatchcock"]) {
        return Some(format!(
            "A spatchcocked bird is flat, so it roasts fast and even: 200C (180C fan) at 20 minutes per kg, {} {done}",
            _cut_time(understood, 20.0, 0.0, "so a 4kg one is about 1 hour 20 minutes.")
        ));
    }
    if has(&["wing", "wings"]) {
        return Some("Turkey wings roast at 200C (180C fan) in 25 to 30 minutes, turned once, and they are done at 74C in the thickest part, so rest them 5 minutes under foil.".into());
    }
    if has(&["drumstick", "drumsticks"]) {
        return Some(format!("Turkey drumsticks take 35 to 40 minutes at 200C (180C fan), turned once so the skin browns all over. {dark}"));
    }
    if has(&["leg", "legs"]) {
        return Some(format!("Turkey legs take 45 to 50 minutes at 190C (170C fan), skin side up, turned once. {dark}"));
    }
    if has(&["thigh", "thighs"]) {
        return Some(format!("Boneless turkey thighs take 40 to 45 minutes at 190C (170C fan); bone-in ones want 10 minutes more. {dark}"));
    }
    if has(&["fillet", "fillets", "steak", "steaks", "scallop", "scallops"]) {
        return Some("Turkey fillets, steaks and escalopes are thin and quick: 20 to 25 minutes at 190C (170C fan), or 8 to 10 minutes in a hot pan. They are done at 74C, so check one and rest them 5 minutes under foil.".into());
    }
    if has(&["boneless", "rolled", "butterflied", "flat"]) {
        return Some(format!(
            "A boneless, rolled turkey breast wants 30 minutes per 500g at 190C (170C fan), {} {done}",
            _cut_time(understood, 60.0, 0.0, "so a 1.5kg one takes about 1 hour 30 minutes.")
        ));
    }
    Some(format!(
        "A bone-in turkey breast takes 20 minutes per 500g plus 20 at 190C (170C fan), {} {done}",
        _cut_time(understood, 40.0, 20.0, "so a 2kg one is about 1 hour 40 minutes.")
    ))
}

/// Hummus, with or without tahini.
pub fn _hummus(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = words_and_typed(understood);
    let no_tahini = said.contains("no tahini")
        || said.contains("without tahini")
        || said.contains("tahini free")
        || (said.contains("tahini")
            && ["dont have", "don't have", "ran out", "out of", "none", "substitute", "instead of", "replace"].iter().any(|p| said.contains(p)));
    if no_tahini {
        return Some("Without tahini, blitz the drained tin of chickpeas with the juice of a lemon, a small garlic clove, a good pinch of salt and 2 tablespoons of olive oil, then loosen it with iced water a spoon at a time. A spoon of yogurt or a second squeeze of lemon stands in for the richness the tahini would give.".into());
    }
    Some("Blitz a drained tin of chickpeas with 2 tablespoons of tahini, the juice of a lemon, a small garlic clove and half a teaspoon of salt, then loosen it with iced water a spoon at a time - that is what makes it fluffy. A pinch of bicarbonate of soda in the pan softens the skins if you boil your own chickpeas.".into())
}

lazy!(GOING_OFF_ASK: Vec<String> = "GOING_OFF_ASK");
lazy!(KEEPING_VERB: Vec<String> = "KEEPING_VERB");
lazy!(IN_THE_OVEN: Vec<String> = "IN_THE_OVEN");

/// Roasting a joint: pork, beef or lamb, or both cards.
pub fn _roast_joint(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = text::squash(&understood.text.to_lowercase());
    if any_in(&line, &GOING_OFF_ASK) {
        return None;
    }
    if any_in(&line, &KEEPING_VERB) && !any_in(&line, &IN_THE_OVEN) {
        return None;
    }
    let words = lower_words(understood);
    if any_same(&words, &["pork", "gammon", "ham"]) {
        return Some(st("JOINT_PORK"));
    }
    if any_same(&words, &["beef", "lamb", "mutton"]) {
        return Some(st("JOINT_BEEF"));
    }
    Some(format!("{} {}", st("JOINT_BEEF"), st("JOINT_PORK")))
}

lazy!(KEEPING_SENSE: Vec<String> = "KEEPING_SENSE");
lazy!(_MINCE_PIE_PHRASE: Re = "_MINCE_PIE_PHRASE");
lazy!(_BOLOGNESE_WORDS: Vec<String> = "_BOLOGNESE_WORDS");

/// A grainy sauce from the mince.
pub fn _mince_grainy(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    if any_same(&words, &KEEPING_SENSE) {
        return None;
    }
    if _MINCE_PIE_PHRASE.is_match(&words.join(" ")) {
        return None;
    }
    if any_same(&words, &_BOLOGNESE_WORDS) && !words.iter().any(|w| _same(w, "sauce")) {
        return None;
    }
    Some("A grainy sauce is usually the mince: it went into a pan that was not hot enough, so it boiled in its own juice instead of browning. Heat a heavy pan until a drop of water sizzles, then brown the mince in two batches. Leave it alone for 2 to 3 minutes at a time so it colours before the liquid goes in. For the pot you have, give it a slow half hour with the lid on, which softens the grains, and stir in a spoon of butter at the end.".into())
}

/// Browning mince, or mince that went tough.
pub fn _mince(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    if any_same(&words, &KEEPING_SENSE) {
        return None;
    }
    if any_same(&words, &["tough", "rubbery", "chewy", "dry"]) {
        return Some("Minced meat goes tough and grainy when it is cooked too long or boiled in its own juice: brown it fast in a hot pan in batches, then add liquid only once it has coloured. A pinch of bicarb (half a teaspoon per 500g) keeps it soft in a long bolognese.".into());
    }
    Some("Brown mince in batches over a high heat and don't stir it much: it should sizzle and colour, not steam. Pale, grey mince is mince that was crowded in the pan.".into())
}

/// `understanding.line_as_read`: the repaired line, else the clean one, else the typed one.
// DUP(bot.understanding::line_as_read)
pub fn line_as_read(u: &Understood) -> String {
    if !text::strip(&u.plain).is_empty() {
        return u.plain.clone();
    }
    or(&u.clean, &u.text).to_string()
}

/// What the person has said so far: their last four lines and this one.
pub fn _said_so_far(understood: &Understood, memory: &Memory) -> String {
    let line = line_as_read(understood);
    let start = memory.turns.len().saturating_sub(4);
    let mut parts: Vec<String> = memory.turns[start..].iter().map(|t| t.user.clone()).collect();
    parts.push(line);
    parts.join(" ")
}

static HEAD_COUNT_SHAPES: LazyLock<Vec<Re>> = LazyLock::new(|| {
    let people = st("_PEOPLE_AFTER");
    let words = st("_COUNT_WORD_SHAPES");
    let when = r"(?:on\s+\w+day|tonight|tomorrow|this weekend|next week|at the weekend|this christmas|for christmas)";
    [
        format!(r"\bfor\s+(\d{{1,2}})\s*{people}"),
        format!(r"\bfor\s+({words})\s*{people}"),
        format!(r"\b(\d{{1,2}})\s*{people}"),
        format!(r"\b({words})\s*{people}"),
        format!(r"\bfor\s+(\d{{1,2}})\s*{when}"),
        format!(r"\bfor\s+({words})\s*{when}"),
        format!(r"\bfor\s+(\d{{1,2}})\s*(?=[,.!?]|$)"),
        format!(r"\bfor\s+({words})\s*(?=[,.!?]|$)"),
        format!(r"\bwe(?:'re| are)\s+(\d{{1,2}})\b"),
        format!(r"\bwe(?:'re| are)\s+({words})\b"),
    ]
    .iter()
    .map(|p| Re::new(p, 0))
    .collect()
});

/// How many people the line cooks for (2 to 20), or None.
pub fn _head_count(text_: &str) -> Option<i64> {
    let text_ = text_.to_lowercase();
    for shape in HEAD_COUNT_SHAPES.iter() {
        let Some(found) = shape.search(&text_) else { continue };
        let value = found.str(1);
        let number = if !value.is_empty() && text::is_digit(value) {
            value.parse::<i64>().ok()
        } else {
            g("_COUNT_VALUE").get(value).and_then(Value::as_int)
        };
        if let Some(n) = number {
            if n != 0 && (2..=20).contains(&n) {
                return Some(n);
            }
        }
    }
    None
}

/// The shopping for a roast: the bird or joint for the number of people, and the trimmings.
pub fn _roast_shopping(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text_ = or(&understood.clean, &understood.text).to_lowercase();
    let people = _head_count(&text_);
    let trimmings = "potatoes, carrots, parsnips or swede, and something green like cabbage or sprouts, plus oil, and stock and flour for the gravy";
    let bird_grams = g("BIRD_GRAMS").as_int().unwrap_or(500);
    let (body, bird, timing);
    if text_.contains("turkey") || text_.contains("christmas dinner") || text_.contains("xmas dinner") {
        let per = bird_grams;
        bird = "turkey";
        timing = "20 minutes per 500g plus 90";
        body = match people {
            Some(p) => format!("About {}kg of turkey for {p}, at {per}g a person on the bone", py_g((p * per) as f64 / 1000.0)),
            None => "Most people do turkey: about 500g a person on the bone, so a 4kg bird feeds eight".to_string(),
        };
    } else if text_.contains("chicken") {
        let per = bird_grams;
        bird = "chicken";
        timing = "20 minutes per 500g plus 20";
        body = match people {
            Some(p) => format!("About {}kg of chicken for {p}, at {per}g a person on the bone", py_g((p * per) as f64 / 1000.0)),
            None => format!("About {per}g of chicken a person on the bone, so a 3kg bird feeds six and a 1.5kg one feeds three"),
        };
    } else {
        let per = 250;
        bird = "";
        timing = "20 minutes per 500g plus 20";
        body = match people {
            Some(p) => format!("About {}kg of raw joint for {p}, which is 250g a person", py_g((p * per) as f64 / 1000.0)),
            None => "For a roast: about 250g of raw joint a person, so 1.5kg feeds six".to_string(),
        };
    }
    let mut text_out = format!("{body}. Then {trimmings}. For timing, work on {timing} at 200C (180C fan), and rest it 15 minutes before carving.");
    if bird.is_empty() {
        text_out.push_str(" Add flour, eggs and milk for Yorkshire puddings, and mint or apple sauce if it is lamb or pork.");
    }
    Some(text_out)
}

// ---------------------------------------------------------------------------------------------
// What the eater rules out: diets, allergies, conditions, and the answers about one food

/// A text window of up to `n` characters before a byte offset (Python `text[max(0, at-n):at]`).
fn chars_before(text_: &str, at: usize, n: usize) -> &str {
    let ci = text::char_index(text_, at);
    &text_[text::byte_at(text_, ci.saturating_sub(n))..at]
}

/// A text window of up to `n` characters after a byte offset (Python `text[at:at+n]`).
fn chars_after(text_: &str, at: usize, n: usize) -> &str {
    let ci = text::char_index(text_, at);
    &text_[at..text::byte_at(text_, ci + n)]
}

fn meets(a: &HashSet<String>, b: &HashSet<String>) -> bool {
    a.iter().any(|x| b.contains(x))
}

lazy!(MEAT: HashSet<String> = "MEAT");
lazy!(FISH: HashSet<String> = "FISH");
lazy!(DAIRY: HashSet<String> = "DAIRY");
lazy!(EGGS: HashSet<String> = "EGGS");
lazy!(NUTS: HashSet<String> = "NUTS");
lazy!(GLUTEN: HashSet<String> = "GLUTEN");
lazy!(SHELLFISH: HashSet<String> = "SHELLFISH");
lazy!(CHEESES: HashSet<String> = "CHEESES");
lazy!(NO_MEAT_DIET: HashSet<String> = "NO_MEAT_DIET");

/// The food families in ANY_FAMILY order: NUTS, GLUTEN, DAIRY, EGGS, MEAT, FISH, SHELLFISH. A
/// family is its index here; the id-keyed tables (FAMILY_SAY, SELF_SAY, BAN_SAY_NAME) are read
/// through the ids FAMILY_SAY lists in the same order.
static ANY_FAMILY: LazyLock<Vec<HashSet<String>>> = LazyLock::new(|| g("ANY_FAMILY").items().iter().map(Value::str_set).collect());
const F_NUTS: usize = 0;
const F_MEAT: usize = 4;
const F_FISH: usize = 5;

/// The Python id FAMILY_SAY gives a family (its key, in ANY_FAMILY order).
fn family_id(i: usize) -> Option<i64> {
    g("FAMILY_SAY").pairs().get(i).and_then(|(k, _)| k.as_int())
}

fn by_family_id<'a>(table: &'a Value, i: usize) -> Option<&'a Value> {
    let id = family_id(i)?;
    table.pairs().iter().find(|(k, _)| k.as_int() == Some(id)).map(|(_, v)| v)
}

fn family_say(i: usize) -> (String, String) {
    let row = by_family_id(g("FAMILY_SAY"), i).expect("FAMILY_SAY").strs();
    (row[0].clone(), row[1].clone())
}

/// `FAMILIES.get(word)`.
fn families_get(word: &str) -> Option<HashSet<String>> {
    g("FAMILIES").get(word).map(Value::str_set)
}

/// `MADE_OF.get(food, frozenset())`.
fn made_of(food: &str) -> HashSet<String> {
    g("MADE_OF").get(food).map(Value::str_set).unwrap_or_default()
}

fn made_of_has(food: &str) -> bool {
    g("MADE_OF").get(food).is_some()
}

lazy!(_DIET_ASKED: Re = "_DIET_ASKED");
lazy!(_DIET_ASK_NOT_A_FOOD: HashSet<String> = "_DIET_ASK_NOT_A_FOOD");

/// True when the diet word at `at` sits in a question about a food ("is honey vegan").
pub fn _ask_about_the_food(text_: &str, at: usize) -> bool {
    let head = &text_[..at.min(text_.len())];
    for m in _DIET_ASKED.finditer(head) {
        if m.end() <= at && !text::split(m.str(1)).iter().any(|w| _DIET_ASK_NOT_A_FOOD.contains(w)) {
            return true;
        }
    }
    false
}

lazy!(AVOIDED: Re = "AVOIDED");
lazy!(FREEFROM: Re = "FREEFROM");
lazy!(TROUBLE: Re = "TROUBLE");
re_lazy!(VEGETARIAN_WORD = r"\b(?:vegetarian|veggie|vego)\b");
re_lazy!(VEGAN_WORD = r"\b(?:vegan|plant based|plantbased)\b");
static CONDITION_PATTERNS: LazyLock<Vec<(Re, HashSet<String>)>> = LazyLock::new(|| {
    g("CONDITION_BANS")
        .pairs()
        .iter()
        .map(|(k, v)| (Re::new(&format!("\\b{}\\b", k.str().replace(' ', "\\s+")), 0), v.str_set()))
        .collect()
});

/// Every food the line rules out (by diet, allergy, "free from", trouble or condition), with a
/// later "not" taking one back.
pub fn _excluded(text_: &str) -> HashSet<String> {
    let text_ = text_.to_lowercase().replace('\u{2019}', "'").replace('-', " ");
    let mut events: Vec<(usize, HashSet<String>, bool)> = Vec::new();
    for m in VEGETARIAN_WORD.finditer(&text_) {
        if _ask_about_the_food(&text_, m.start()) {
            continue;
        }
        events.push((m.start(), NO_MEAT_DIET.clone(), !_denied_at(&text_, m.start())));
    }
    for m in VEGAN_WORD.finditer(&text_) {
        if _ask_about_the_food(&text_, m.start()) {
            continue;
        }
        let mut fams = NO_MEAT_DIET.clone();
        fams.extend(DAIRY.iter().cloned());
        fams.extend(EGGS.iter().cloned());
        events.push((m.start(), fams, !_denied_at(&text_, m.start())));
    }
    for (which, pattern) in [(0, &**AVOIDED), (1, &**FREEFROM), (2, &**TROUBLE)] {
        for m in pattern.finditer(&text_) {
            let word = m.groups().into_iter().flatten().find(|p| !p.is_empty()).unwrap_or("");
            if word.is_empty() {
                continue;
            }
            let families = _families_of(word);
            let denied = _denied_at(&text_, m.start());
            events.push((m.start(), families, !denied));
            events.push((m.start(), _foods_in_list(&_list_words(chars_after(&text_, m.end(), 80), false)), !denied));
            if which == 1 {
                events.push((m.start(), _foods_in_list(&_list_words(chars_before(&text_, m.start(), 80), true)), !denied));
            }
        }
    }
    for (pattern, foods) in CONDITION_PATTERNS.iter() {
        for m in pattern.finditer(&text_) {
            events.push((m.start(), foods.clone(), !_denied_at(&text_, m.start())));
        }
    }
    events.sort_by_key(|e| e.0);
    let mut state: HashMap<String, bool> = HashMap::new();
    for (_, families, positive) in &events {
        for word in families {
            state.insert(word.clone(), *positive);
        }
    }
    let mut banned: HashSet<String> = state.into_iter().filter(|(_, on)| *on).map(|(w, _)| w).collect();
    if banned.contains("cheese") || meets(&DAIRY, &banned) {
        banned.extend(CHEESES.iter().cloned());
    }
    banned
}

lazy!(_LACTOSE_WORDS: Vec<String> = "_LACTOSE_WORDS");
lazy!(_NOT_LACTOSE_WORDS: Vec<String> = "_NOT_LACTOSE_WORDS");

/// True when the text says lactose intolerance (and not lactose-free food).
pub fn _a_lactose_intolerance(said: &str) -> bool {
    if !any_in(said, &_LACTOSE_WORDS) {
        return false;
    }
    let plain = said.replace('\u{2019}', "'");
    !any_in(&plain, &_NOT_LACTOSE_WORDS)
}

/// The strings a Python value holds: itself if a str, its items if a list/tuple/set.
fn value_strings(v: &Value) -> Vec<String> {
    match v {
        Value::Str(s) => vec![s.to_string()],
        Value::List(_) | Value::Tuple(_) | Value::Set(_) => v.items().iter().map(value_str).collect(),
        _ => Vec::new(),
    }
}

/// Python `str(x)` for the plain values a fact record holds.
fn value_str(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        Value::Bool(b) => if *b { "True".into() } else { "False".into() },
        Value::None => "None".into(),
        other => format!("{other:?}"),
    }
}

/// Iterating a fact value as Python would (`for item in value or ()`): a str gives its characters.
fn value_iter(v: Option<&Value>) -> Vec<String> {
    match v {
        None => Vec::new(),
        Some(Value::Str(s)) => s.chars().map(|c| c.to_string()).collect(),
        Some(v) if !v.truthy() => Vec::new(),
        Some(v) => v.items().iter().map(value_str).collect(),
    }
}

/// True when a fact record (allergies, diet, conditions) says lactose intolerance.
pub fn _record_says_lactose(known: &Value) -> bool {
    if !matches!(known, Value::Dict(_)) {
        return false;
    }
    let mut said: Vec<String> = Vec::new();
    for key in ["allergies", "diet", "diets", "condition", "conditions"] {
        if let Some(v) = known.get(key) {
            said.extend(value_strings(v));
        }
    }
    _a_lactose_intolerance(&said.join(" ").to_lowercase().replace('-', " "))
}

/// `memory_store.user(memory)`: the user's facts, created (empty) on first use.
// DUP(bot.memory::user)
fn store_user(memory: &mut Memory) -> Value {
    match memory.extra.get("user") {
        Some(v @ Value::Dict(_)) => v.clone(),
        _ => {
            let empty = Value::Dict(std::sync::Arc::new(Vec::new()));
            memory.extra.insert("user".into(), empty.clone());
            empty
        }
    }
}

/// True when the stored facts about the user say lactose intolerance.
pub fn _their_diet_is_a_lactose_intolerance(memory: &mut Memory) -> bool {
    let known = store_user(memory);
    _record_says_lactose(&known)
}

/// The foods a word stands for: its family, or the word itself.
pub fn _families_of(word: &str) -> HashSet<String> {
    if word == "lactose" {
        return DAIRY.clone();
    }
    if word == "wheat" {
        return GLUTEN.clone();
    }
    families_get(word).unwrap_or_else(|| HashSet::from([word.to_string()]))
}

lazy!(LIST_JOINERS: HashSet<String> = "LIST_JOINERS");
lazy!(LIST_FOODS: HashSet<String> = "LIST_FOODS");

/// The foods of a list ("nuts, dairy and eggs"), up to the first word that is not one.
pub fn _foods_in_list(words: &[String]) -> HashSet<String> {
    let mut found = HashSet::new();
    for word in words {
        let word = text::strip_chars(word, ".,!?;:'");
        if word.is_empty() || LIST_JOINERS.contains(word) {
            continue;
        }
        if !LIST_FOODS.contains(word) {
            break;
        }
        found.extend(_families_of(word));
    }
    found
}

re_lazy!(LIST_WORD = r"[a-z']+|[&+,]");

/// The words and list joiners of a text, in order or backwards.
pub fn _list_words(text_: &str, backwards: bool) -> Vec<String> {
    let mut words = LIST_WORD.findall_str(&text_.to_lowercase());
    if backwards {
        words.reverse();
    }
    words
}

lazy!(NOT_THIS: Re = "NOT_THIS");

/// True when the 30 characters before `start` take the food back ("not vegetarian").
pub fn _denied_at(text_: &str, start: usize) -> bool {
    NOT_THIS.is_match(chars_before(text_, start, 30))
}

/// The first food (two words, then one) the text names that MADE_OF knows.
pub fn _food_named(text_: &str) -> Option<String> {
    let words = LETTER_WORDS.findall_str(&text_.to_lowercase());
    for size in [2usize, 1] {
        if words.len() < size {
            continue;
        }
        for start in 0..=(words.len() - size) {
            let phrase = words[start..start + size].join(" ");
            if made_of_has(&phrase) {
                return Some(phrase);
            }
        }
    }
    None
}

/// Every food the text names that MADE_OF knows, two-word names first.
pub fn _foods_named(text_: &str) -> Vec<String> {
    let words = LETTER_WORDS.findall_str(&text_.to_lowercase());
    let mut found = Vec::new();
    let mut taken: HashSet<usize> = HashSet::new();
    for size in [2usize, 1] {
        if words.len() < size {
            continue;
        }
        for start in 0..=(words.len() - size) {
            if (start..start + size).any(|at| taken.contains(&at)) {
                continue;
            }
            let phrase = words[start..start + size].join(" ");
            if made_of_has(&phrase) {
                found.push(phrase);
                taken.extend(start..start + size);
            }
        }
    }
    found
}

lazy!(SINGULAR_IN_S: HashSet<String> = "SINGULAR_IN_S");

/// True when the food is said as a plural ("crackers", not "couscous").
pub fn _is_plural(food: &str) -> bool {
    let words = text::split(food);
    let word = words.last().map(String::as_str).unwrap_or("");
    if SINGULAR_IN_S.contains(word) || word.ends_with("ss") {
        return false;
    }
    word.ends_with('s')
}

re_lazy!(AFTER_A_FOOD = r"[\s-]+([a-z']+)");
re_lazy!(FREE_BEFORE = r"([a-z]+)\s+free$");
re_lazy!(LAST_WORD = r"([a-z']+)$");

/// The food as the line names it, with its article and a "gluten free" in front kept.
pub fn _named_as(text_: &str, food: &str) -> String {
    let text_ = text_.to_lowercase();
    let pattern = text::split(food).iter().map(|p| crate::py::re::escape(p)).collect::<Vec<_>>().join("\\s+");
    let free_of = g("FREE_OF");
    for m in Re::new(&format!("\\b{pattern}\\b"), 0).finditer(&text_) {
        if let Some(after) = AFTER_A_FOOD.match_(&text_[m.end()..]) {
            if made_of_has(after.str(1)) {
                continue;
            }
        }
        let mut before = text::rstrip(&text_[..m.start()]).to_string();
        let mut qualifier = String::new();
        if let Some(free) = FREE_BEFORE.search(&before) {
            if free_of.get(free.str(1)).is_some() {
                qualifier = format!("{} free ", free.str(1));
                let cut = free.start();
                before = text::rstrip(&before[..cut]).to_string();
            }
        }
        if let Some(article) = LAST_WORD.search(&before) {
            if ["a", "an", "the"].contains(&article.str(1)) {
                return format!("{} {}{}", article.str(1), qualifier, food);
            }
        }
        return format!("{qualifier}{food}");
    }
    food.to_string()
}

/// Everything the user rules out: the stored facts, what they said about themselves, lactose.
pub fn _bans_for(understood: &Understood, memory: &mut Memory) -> HashSet<String> {
    let mut banned = _bans_from_store(memory);
    banned.extend(_excluded(&_mine_only(&_said_so_far(understood, memory))));
    let line = or(&understood.clean, &understood.text).to_lowercase();
    if any_in(&line, &_NOT_LACTOSE_WORDS) {
        // a lactose-free food is not an intolerance
    } else if _a_lactose_intolerance(&line) || _their_diet_is_a_lactose_intolerance(memory) {
        banned.insert("lactose".into());
    }
    banned
}

re_lazy!(CLAUSE_SPLIT = r"(?<=[.!?;])\s+|\s*,\s*|\s+and\s+|\s+but\s+");
lazy!(THIRD_PERSON: Re = "THIRD_PERSON");

/// The clauses of a text that are about the speaker, not somebody else.
pub fn _mine_only(text_: &str) -> String {
    let parts: Vec<String> =
        CLAUSE_SPLIT.split(text_).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    parts.into_iter().filter(|p| THIRD_PERSON.match_(p).is_none()).collect::<Vec<_>>().join(" ")
}

/// What the stored facts about the user rule out.
pub fn _bans_from_store(memory: &Memory) -> HashSet<String> {
    match memory.extra.get("user") {
        Some(user @ Value::Dict(_)) => _bans_of(Some(user)),
        _ => HashSet::new(),
    }
}

/// What a fact record (allergies, conditions, diet) rules out.
pub fn _bans_of(facts: Option<&Value>) -> HashSet<String> {
    let mut banned = HashSet::new();
    let Some(facts @ Value::Dict(_)) = facts else { return banned };
    for item in value_iter(facts.get("allergies")) {
        let word = item.to_lowercase();
        match families_get(&word) {
            Some(f) if !f.is_empty() => banned.extend(f),
            _ => {
                banned.insert(word);
            }
        }
    }
    for item in value_iter(facts.get("condition")) {
        let said = item.to_lowercase();
        for (condition, foods) in g("CONDITION_BANS").pairs() {
            if said.contains(condition.str()) {
                banned.extend(foods.str_set());
            }
        }
    }
    let diet = facts.get("diet").filter(|v| v.truthy()).map(value_str).unwrap_or_default().to_lowercase();
    if ["vegetarian", "veggie", "vego"].contains(&diet.as_str()) {
        banned.extend(NO_MEAT_DIET.iter().cloned());
    } else if ["vegan", "plant based", "plantbased"].contains(&diet.as_str()) {
        banned.extend(NO_MEAT_DIET.iter().cloned());
        banned.extend(DAIRY.iter().cloned());
        banned.extend(EGGS.iter().cloned());
    } else if let Some(rest) = diet.strip_prefix("no ") {
        let word = text::strip(rest).to_string();
        match families_get(&word) {
            Some(f) if !f.is_empty() => banned.extend(f),
            _ => {
                banned.insert(word);
            }
        }
    }
    banned
}

/// BAN_SAY rows: (family index, the word that names it).
fn ban_say() -> Vec<(usize, String)> {
    g("BAN_SAY").items().iter().enumerate().map(|(i, row)| (i, row.items()[1].str().to_string())).collect()
}

/// The families the eater has named outright ("nuts", "gluten", ...).
pub fn _named_families(banned: &HashSet<String>) -> Vec<usize> {
    ban_say().into_iter().filter(|(_, name)| banned.contains(name)).map(|(f, _)| f).collect()
}

/// The names of the families ruled out, as the reply says them.
pub fn _bans_said(banned: &HashSet<String>) -> Vec<String> {
    let named = _named_families(banned);
    let mut names: Vec<String> = ban_say().into_iter().filter(|(f, _)| named.contains(f)).map(|(_, n)| n).collect();
    if names.is_empty() {
        names = ban_say().into_iter().filter(|(f, _)| meets(&ANY_FAMILY[*f], banned)).map(|(_, n)| n).collect();
    }
    if names.iter().any(|n| n == "meat") && names.iter().any(|n| n == "shellfish") {
        let at = names.iter().position(|n| n == "shellfish").unwrap();
        names.remove(at);
    }
    if names.iter().any(|n| n == "meat") && names.iter().any(|n| n == "fish") {
        let at = names.iter().position(|n| n == "fish").unwrap();
        names.remove(at);
    }
    names
}

/// The one other person the line is about, from what memory holds about others, or None.
pub fn _the_guest(memory: &Memory, text_: &str) -> Option<Value> {
    let others = memory.extra.get("others")?;
    let Value::List(list) = others else { return None };
    if list.is_empty() {
        return None;
    }
    let low = text::squash(&text_.to_lowercase());
    let named: Vec<&Value> = list
        .iter()
        .filter(|o| {
            matches!(o, Value::Dict(_))
                && o.get("who").is_some_and(Value::truthy)
                && Re::new(&format!("\\b{}\\b", crate::py::re::escape(&value_str(o.get("who").unwrap()).to_lowercase())), 0).is_match(&low)
        })
        .collect();
    if named.len() == 1 {
        return Some(named[0].clone());
    }
    if !named.is_empty() {
        return None;
    }
    if list.len() == 1 && matches!(list[0], Value::Dict(_)) { Some(list[0].clone()) } else { None }
}

/// The first food (two words, then one) the text names that MAY_HAVE knows.
pub fn _food_may(text_: &str) -> Option<String> {
    let words = LETTER_WORDS.findall_str(&text_.to_lowercase());
    let may = g("MAY_HAVE");
    for size in [2usize, 1] {
        if words.len() < size {
            continue;
        }
        for start in 0..=(words.len() - size) {
            let phrase = words[start..start + size].join(" ");
            if may.get(&phrase).is_some() {
                return Some(phrase);
            }
        }
    }
    None
}

/// "Can my sister eat X": the answer from what memory holds about her allergies or conditions.
pub fn _someones_else_can_eat(groups: &[Option<&str>], line: &str, memory: &Memory) -> Option<String> {
    let guest = _the_guest(memory, line)?;
    if !(guest.get("allergies").is_some_and(Value::truthy) || guest.get("condition").is_some_and(Value::truthy)) {
        return None;
    }
    let said = groups.iter().flatten().filter(|p| !p.is_empty()).copied().collect::<Vec<_>>().join(" ");
    let food = _food_named(&said).or_else(|| _food_may(&said))?;
    let banned = _bans_of(Some(&guest));
    if banned.is_empty() {
        return None;
    }
    if _record_says_lactose(&guest) && !any_in(&said.to_lowercase(), &_NOT_LACTOSE_WORDS) {
        if let Some(answer) = _lactose_answer(&food, &said) {
            return Some(answer);
        }
    }
    _not_for_them(&food, &banned, Some(&guest), &said)
}

/// The answer for a food somebody else cannot have, or None when nothing in it is ruled out.
pub fn _not_for_them(food: &str, banned: &HashSet<String>, guest: Option<&Value>, text_: &str) -> Option<String> {
    let named = _named_as(text_, food);
    let who = guest.and_then(|g| g.get("who")).filter(|v| v.truthy()).map(value_str).unwrap_or_default();
    let who = text::strip(&who).to_lowercase();
    let them = if !who.is_empty() { format!("your {who}") } else { "them".to_string() };
    let named_here = _named_families(banned);
    let food_made_of = made_of(food);
    let mut hits: Vec<usize> = (0..ANY_FAMILY.len()).filter(|&f| meets(&ANY_FAMILY[f], &food_made_of) && meets(&ANY_FAMILY[f], banned)).collect();
    hits.sort_by_key(|f| if named_here.contains(f) { 0 } else { 1 });
    // FREE_OF at run time is the str-keyed table (a later definition replaces the id-keyed one),
    // so the id lookups below always give their default.
    if let Some(&first) = hits.first() {
        let mut said = family_say(first).0;
        if _is_plural(food) {
            said = IS_WORD.sub("are", &said);
        }
        return Some(format!(
            "No: {named} {said}, so it is off the table for {them}. A pack or a menu that says free from it is the only check that settles it."
        ));
    }
    let may_set = g("MAY_HAVE").get(food).map(Value::str_set).unwrap_or_default();
    let may: Vec<usize> = (0..ANY_FAMILY.len()).filter(|&f| meets(&ANY_FAMILY[f], &may_set) && meets(&ANY_FAMILY[f], banned)).collect();
    let &family = may.first()?;
    let what = if family == F_NUTS {
        "nuts".to_string()
    } else {
        by_family_id(g("BAN_SAY_NAME"), family).map(|v| v.str().to_string()).unwrap_or_else(|| "it".into())
    };
    Some(format!(
        "{} can be made without {what}, but the bought or restaurant version often has {} in, so for {them} the pack or the menu has to say free from it.",
        text::capitalize(&named),
        if what == "nuts" { "them" } else { "it" }
    ))
}

re_lazy!(IS_WORD = r"\bis\b");
lazy!(_POINTING: HashSet<String> = "_POINTING");
lazy!(_LEAVE_IT_OUT: HashSet<String> = "_LEAVE_IT_OUT");

/// The foods a short line points at ("and the cake?"), or nothing.
pub fn _foods_a_line_points_at(line: &str) -> Vec<String> {
    let words = LETTER_WORDS.findall_str(&line.to_lowercase());
    if words.is_empty() || words.len() > 7 {
        return Vec::new();
    }
    let foods = _foods_named(line);
    if foods.is_empty() {
        return _food_may(line).into_iter().collect();
    }
    let spare: HashSet<String> = text::split(&foods.join(" ")).into_iter().collect();
    if words.iter().any(|w| !spare.contains(w) && !_POINTING.contains(w)) {
        return Vec::new();
    }
    if words.iter().any(|w| _LEAVE_IT_OUT.contains(w)) {
        return Vec::new();
    }
    foods
}

/// The answer for the one food a short line points at, against what the eater rules out.
pub fn pointed_at_a_food(understood: &Understood, memory: &mut Memory) -> Option<String> {
    let line = or(&understood.clean, &understood.text).to_string();
    let foods = _foods_a_line_points_at(&line);
    if foods.is_empty() {
        return None;
    }
    let guest = _the_guest(memory, &line);
    let guest_bans = guest.as_ref().map(|g| _bans_of(Some(g))).unwrap_or_default();
    if !guest_bans.is_empty() {
        let hits: Vec<&String> = foods.iter().filter(|f| made_of_has(f) && _bans_it(f, &guest_bans)).collect();
        return if hits.len() == 1 { _not_for_them(hits[0], &guest_bans, guest.as_ref(), &line) } else { None };
    }
    let banned = _bans_for(understood, memory);
    if _bans_said(&banned).is_empty() {
        return None;
    }
    let hits: Vec<&String> = foods.iter().filter(|f| made_of_has(f) && _bans_it(f, &banned)).collect();
    if hits.len() == 1 { _one_food_answer(hits[0], &banned, &line, false) } else { None }
}

lazy!(EAT_ANY: Re = "EAT_ANY");
lazy!(EAT_ANY_WORDS: HashSet<String> = "EAT_ANY_WORDS");
lazy!(EAT_ASK_THEM: Re = "EAT_ASK_THEM");
lazy!(EAT_ASK: Re = "EAT_ASK");
lazy!(EAT_ASK_IF: Re = "EAT_ASK_IF");
lazy!(EAT_ASK_BARE: Re = "EAT_ASK_BARE");
lazy!(FANCY_ASK: Re = "FANCY_ASK");

/// "Can I eat X?" against what the eater has ruled out, or None.
pub fn suits_them(understood: &Understood, memory: &mut Memory) -> Option<String> {
    let line = or(&understood.clean, &understood.text).to_string();
    if EAT_ANY.is_match(&line) {
        let said_here = LETTER_WORDS.findall_str(&line.to_lowercase()).into_iter().any(|w| !EAT_ANY_WORDS.contains(&w));
        if said_here {
            return None;
        }
        let banned = _bans_for(understood, memory);
        if _bans_said(&banned).is_empty() {
            return None;
        }
        return Some(_any_meal(&banned));
    }
    if let Some(about_them) = EAT_ASK_THEM.search(&line) {
        let groups = about_them.groups();
        return _someones_else_can_eat(&groups, &line, memory);
    }
    if let Some(pointed) = pointed_at_a_food(understood, memory) {
        if !pointed.is_empty() {
            return Some(pointed);
        }
    }
    let banned = _bans_for(understood, memory);
    if _bans_said(&banned).is_empty() {
        return None;
    }
    let mut bare = false;
    let asked = match EAT_ASK.search(&line).or_else(|| EAT_ASK_IF.search(&line)) {
        Some(m) => m,
        None => {
            if !EAT_ASK_BARE.is_match(&line) || line.to_lowercase().contains("still") {
                return None;
            }
            bare = true;
            EAT_ASK_BARE.search(&line).unwrap()
        }
    };
    let asked_text = asked.as_str().to_string();
    let mut named_foods = _foods_named(&asked_text);
    if named_foods.is_empty() {
        return None;
    }
    let distinct: HashSet<&String> = named_foods.iter().collect();
    if distinct.len() > 1 {
        named_foods.retain(|f| _bans_it(f, &banned));
        if named_foods.is_empty() {
            return None;
        }
    }
    _one_food_answer(&named_foods[0], &banned, &asked_text, bare)
}

/// "I fancy X": whether X fits what the eater rules out, or None.
pub fn considered(understood: &Understood, memory: &mut Memory) -> Option<String> {
    let line = or(&understood.clean, &understood.text).to_string();
    let considering = FANCY_ASK.search(&line)?;
    let end = considering.end();
    let banned = _bans_for(understood, memory);
    if _bans_said(&banned).is_empty() {
        return None;
    }
    let food = _food_they_avoid(&_foods_named(&line[end..]), &banned)?;
    _one_food_answer(&food, &banned, &line, false)
}

/// The family set a line says the food is free of ("gluten free pasta"), or empty.
pub fn _free_of(text_: &str, food: &str) -> HashSet<String> {
    let text_ = text_.to_lowercase().replace('\u{2019}', "'").replace('-', " ");
    let phrase = text::split(food).iter().map(|p| crate::py::re::escape(p)).collect::<Vec<_>>().join("\\s+");
    let free_of = g("FREE_OF");
    for m in Re::new(&format!("\\b([a-z]+)\\s+free\\s+{phrase}\\b"), 0).finditer(&text_) {
        if let Some(v) = free_of.get(m.str(1)) {
            return v.str_set();
        }
    }
    for m in Re::new(&format!("{phrase}\\s+(?:is\\s+|are\\s+)?([a-z]+)\\s+free\\b"), 0).finditer(&text_) {
        if let Some(v) = free_of.get(m.str(1)) {
            return v.str_set();
        }
    }
    HashSet::new()
}

/// True when a family in the food is one the eater rules out.
pub fn _bans_it(food: &str, banned: &HashSet<String>) -> bool {
    let food_made_of = g("MADE_OF").get(food).map(Value::str_set).unwrap_or_else(|| panic!("KeyError: {food}"));
    ANY_FAMILY.iter().any(|f| meets(f, &food_made_of) && meets(f, banned))
}

/// The first of these foods the eater rules out, or None.
pub fn _food_they_avoid(named_foods: &[String], banned: &HashSet<String>) -> Option<String> {
    named_foods.iter().find(|f| _bans_it(f, banned)).cloned()
}

lazy!(_LACTOSE_FREE_ASK: Re = "_LACTOSE_FREE_ASK");

/// The lactose answer for a food, or None.
pub fn _lactose_answer(food: &str, text_: &str) -> Option<String> {
    if _LACTOSE_FREE_ASK.is_match(text_) {
        return Some("Yes: lactose-free milk is ordinary milk with the milk sugar already split, so it does the same job in tea, on cereal and in cooking. There are lactose-free yoghurts and soft cheeses too, though a hard cheese or butter has almost none left in it anyway.".into());
    }
    if let Some(v) = g("_LACTOSE_SAYS").get(food) {
        return Some(v.str().to_string());
    }
    if meets(&DAIRY, &made_of(food)) {
        return Some("It depends how much milk is in it: milk, soft cheese, yoghurt and ice cream hold most of the lactose, while a hard cheese or butter has almost none left. Try a small portion with a meal, and lactose-free milk and lactase tablets are there for the ones that catch you out.".into());
    }
    None
}

/// Whether one food fits what the eater rules out, said for them.
pub fn _one_food_answer(food: &str, banned: &HashSet<String>, text_: &str, bare: bool) -> Option<String> {
    let avoid = _bans_said(banned);
    if banned.contains("lactose") {
        if let Some(answer) = _lactose_answer(food, text_) {
            if !answer.is_empty() {
                return Some(answer);
            }
        }
    }
    let base = g("MADE_OF").get(food).map(Value::str_set).unwrap_or_else(|| panic!("KeyError: {food}"));
    let free = _free_of(text_, food);
    let food_made_of: HashSet<String> = base.difference(&free).cloned().collect();
    let named_here = _named_families(banned);
    let mut hits: Vec<usize> = (0..ANY_FAMILY.len()).filter(|&f| meets(&ANY_FAMILY[f], &food_made_of) && meets(&ANY_FAMILY[f], banned)).collect();
    hits.sort_by_key(|f| if named_here.contains(f) { 0 } else { 1 });
    if bare && hits.is_empty() {
        return None;
    }
    if hits.is_empty() {
        if FISH.contains(food) && named_here.contains(&F_MEAT) && !named_here.contains(&F_FISH) {
            return Some(st("FISH_SPLIT"));
        }
        let what = if avoid.len() == 1 {
            avoid[0].clone()
        } else {
            let last = avoid.last().unwrap_or_else(|| panic!("IndexError: list index out of range"));
            format!("{} or {}", avoid[..avoid.len() - 1].join(", "), last)
        };
        let named = _named_as(text_, food);
        if food == "honey" && ["meat", "dairy", "egg"].iter().all(|w| banned.contains(*w)) {
            return Some("Some vegans avoid honey, so it depends how strict you are - sugar or golden syrup does the same job in a recipe.".into());
        }
        if !food_made_of.is_empty() {
            let them = if what == "nuts" { "them" } else { "it" };
            return Some(format!("Nothing in {named} needs {what}, but a bought one can have {them} in - check the label."));
        }
        if avoid.iter().any(|a| a == "gluten") {
            if let Some(v) = g("NOT_SETTLED_BY_THE_RECIPE").get(food) {
                return Some(v.str().to_string());
            }
        }
        if _is_plural(food) {
            return Some(format!("Yes: {named} have no {what} in them, so they are fine for you."));
        }
        return Some(format!("Yes: {named} has no {what} in it, so it is fine for you."));
    }
    let (mut said, swap) = family_say(hits[0]);
    let named = _named_as(text_, food);
    if ANY_FAMILY[hits[0]].contains(food) {
        if let Some(self_say) = by_family_id(g("SELF_SAY"), hits[0]) {
            return Some(format!("No: {}, and it is off the table for you. {}.", self_say.str(), text::upper_first(&swap)));
        }
    }
    if _is_plural(food) {
        said = IS_WORD.sub("are", &said);
    }
    Some(format!("No: {named} {said}, and that is off the table for you. {}.", text::upper_first(&swap)))
}

/// A ban on meat carries fish too.
pub fn _no_meat_also_means_fish(banned: &HashSet<String>) -> HashSet<String> {
    if meets(&MEAT, banned) {
        let mut out = banned.clone();
        out.extend(FISH.iter().cloned());
        out
    } else {
        banned.clone()
    }
}

/// True when an idea (or one of its names) has a food the eater rules out.
pub fn _bans(names: &[String], idea: &str, banned: &HashSet<String>) -> bool {
    for name in names {
        if banned.contains(name) {
            return true;
        }
        if text::split(name).iter().any(|part| banned.contains(part)) {
            return true;
        }
    }
    let text_ = idea.to_lowercase();
    banned.iter().any(|word| word_in(&text_, word))
}

lazy!(SWAP_BEFORE: Re = "SWAP_BEFORE");
lazy!(SWAP_AFTER: Re = "SWAP_AFTER");

/// True when the food at `at` is named as a swap ("a flax egg"), not as part of the dish.
pub fn _a_swap_not_a_dish(text_: &str, at: usize, word: &str) -> bool {
    let window = g("SWAP_WINDOW").as_int().unwrap_or(34) as usize;
    let before = chars_before(text_, at, window);
    let after = chars_after(text_, at + word.len(), window);
    SWAP_BEFORE.is_match(before) || SWAP_AFTER.match_(after).is_some()
}

lazy!(DAIRY_SAYS: HashSet<String> = "DAIRY_SAYS");
lazy!(NOT_THE_DAIRY: Re = "NOT_THE_DAIRY");
re_lazy!(BUTTER_BEANS = r"\s+beans?\b", crate::py::re::I);

/// The banned-food words a text says as part of a dish (not a swap, not a butter bean).
pub fn _says_a_food<S: AsRef<str>>(text_: &str, words: &[S]) -> Vec<String> {
    let mut found = Vec::new();
    for word in words {
        let word = word.as_ref();
        let re = Re::new(&format!("\\b{}(?:s|es)?\\b", crate::py::re::escape(word)), crate::py::re::I);
        for m in re.finditer(text_) {
            if DAIRY_SAYS.contains(word) && NOT_THE_DAIRY.is_match(&text_[..m.start()]) {
                continue;
            }
            if word == "butter" && BUTTER_BEANS.match_(&text_[m.end()..]).is_some() {
                continue;
            }
            if _a_swap_not_a_dish(text_, m.start(), m.as_str()) {
                continue;
            }
            found.push(word.to_string());
            break;
        }
    }
    found
}

/// Everything the user rules out, read from this line (or the stored reading) and memory.
pub fn avoids(memory: &mut Memory, understood: Option<&Understood>) -> HashSet<String> {
    let read = match understood {
        Some(u) => u.clone(),
        None => memory.understood.clone().unwrap_or_default(),
    };
    crate::runtime::guarded(|| _bans_for(&read, memory)).unwrap_or_default()
}

/// The foods a stored allergy word stands for.
pub fn _family_of(word: &str) -> HashSet<String> {
    let word = text::strip(word).to_lowercase();
    if word.is_empty() {
        return HashSet::new();
    }
    let stem = if word.ends_with('s') { word[..word.len() - 1].to_string() } else { word.clone() };
    for key in [&word, &stem] {
        if let Some(f) = families_get(key) {
            return f;
        }
    }
    HashSet::from([word])
}

/// The allergies memory holds for the person the conversation is about.
pub fn avoids_for_the_one_talked_about(memory: &Memory) -> HashSet<String> {
    let Some(person @ Value::Dict(_)) = memory.extra.get("person") else { return HashSet::new() };
    let who = person.get("who").filter(|v| v.truthy()).map(value_str).unwrap_or_default();
    let who = text::strip(&who).to_lowercase();
    if who.is_empty() {
        return HashSet::new();
    }
    let mut banned = HashSet::new();
    if let Some(others) = memory.extra.get("others") {
        for record in others.items() {
            if !matches!(record, Value::Dict(_)) {
                continue;
            }
            let rw = record.get("who").filter(|v| v.truthy()).map(value_str).unwrap_or_default();
            if text::strip(&rw).to_lowercase() != who {
                continue;
            }
            for item in value_iter(record.get("allergies")) {
                banned.extend(_family_of(&item));
            }
            break;
        }
    }
    banned
}

/// The words a reply would say for the foods the eater rules out.
pub fn _the_banned_words(banned: &HashSet<String>) -> HashSet<String> {
    let mut words = HashSet::new();
    if meets(&MEAT, banned) || meets(&FISH, banned) {
        words.extend(set("MEAT_SAYS"));
    }
    if meets(&FISH, banned) {
        words.extend(set("FISH_SAYS"));
    }
    if meets(&DAIRY, banned) && !banned.contains("lactose") {
        words.extend(DAIRY_SAYS.iter().cloned());
    }
    if meets(&EGGS, banned) {
        words.extend(set("EGG_SAYS"));
    }
    if meets(&NUTS, banned) {
        words.extend(set("NUT_SAYS"));
    }
    if meets(&SHELLFISH, banned) {
        words.extend(set("SHELLFISH_SAYS"));
    }
    words
}

re_lazy!(SWAP_PIECES = r"([.!?:]\s+|,\s+)");
lazy!(SWAP_CLAUSE: Re = "SWAP_CLAUSE");

/// The text with `pattern` swapped outside the clauses that already name a swap.
pub fn _rewritten_outside_swaps(text_: &str, pattern: &Re, swap: &str, leave: Option<&Re>) -> String {
    let mut parts = SWAP_PIECES.split(text_);
    for i in (0..parts.len()).step_by(2) {
        if !SWAP_CLAUSE.is_match(&parts[i]) && !leave.is_some_and(|l| l.is_match(&parts[i])) {
            parts[i] = pattern.sub(swap, &parts[i]);
        }
    }
    parts.concat()
}

lazy!(CAUTION_SAID: Re = "CAUTION_SAID");
re_lazy!(SENTENCE_BREAK = r"(?<=[.!?])\s+");

/// The banned words a text says outside its sentences of caution.
pub fn _uncautioned<S: AsRef<str>>(text_: &str, words: &[S]) -> HashSet<String> {
    if !CAUTION_SAID.is_match(text_) {
        return _says_a_food(text_, words).into_iter().collect();
    }
    let mut fresh = HashSet::new();
    for sentence in SENTENCE_BREAK.split(text_) {
        if CAUTION_SAID.is_match(&sentence) {
            continue;
        }
        fresh.extend(_says_a_food(&sentence, words));
    }
    fresh
}

lazy!(SAYS_THE_FOOD_IS_OUT: Re = "SAYS_THE_FOOD_IS_OUT");

/// The nut swaps made in a reply, when that is safe and leaves no banned food.
pub fn _nut_swapped(text_: &str, banned: &HashSet<String>, line: &str) -> String {
    if !meets(&NUTS, banned) || !_a_nut_swap_is_safe(text_, line) {
        return text_.to_string();
    }
    let mut swapped = text_.to_string();
    for row in g("NUT_SWAPS").items() {
        let row = row.items();
        swapped = _rewritten_outside_swaps(&swapped, row[0].as_re().unwrap(), row[1].str(), None);
    }
    if swapped == text_ || names_a_banned_food(Some(&swapped), banned, line) {
        return text_.to_string();
    }
    swapped
}

/// True unless the reply already says the food is out, or the line itself names nuts.
pub fn _a_nut_swap_is_safe(text_: &str, line: &str) -> bool {
    if SAYS_THE_FOOD_IS_OUT.is_match(text_) {
        return false;
    }
    _says_a_food(line, &set_vec("NUT_SAYS")).is_empty()
}

fn set_vec(name: &str) -> Vec<String> {
    strs(name)
}

fn diet_variant(text_: &str) -> String {
    g("DIET_VARIANTS").get(text::strip(text_)).map(|v| v.str().to_string()).unwrap_or_else(|| text_.to_string())
}

/// A reply fitted to what the person talked about rules out, or None when it cannot be.
pub fn fitted_for_the_one_talked_about(reply: Option<&str>, banned: &HashSet<String>, line: &str) -> Option<String> {
    let Some(reply) = reply else { return None };
    if text::strip(reply).is_empty() || banned.is_empty() {
        return Some(reply.to_string());
    }
    let text_ = _nut_swapped(&diet_variant(reply), banned, line);
    if names_a_banned_food(Some(&text_), banned, line) {
        return None;
    }
    Some(text_)
}

/// True when a reply names a food the eater rules out (one the line did not name itself).
pub fn names_a_banned_food(reply: Option<&str>, banned: &HashSet<String>, line: &str) -> bool {
    let Some(reply) = reply else { return false };
    if text::strip(reply).is_empty() || banned.is_empty() {
        return false;
    }
    let words: Vec<String> = _the_banned_words(banned).into_iter().collect();
    if words.is_empty() {
        return false;
    }
    let mut said_already: HashSet<String> = _says_a_food(line, &words).into_iter().collect();
    let stems: Vec<String> = said_already.iter().filter(|w| w.ends_with('s')).map(|w| w[..w.len() - 1].to_string()).collect();
    said_already.extend(stems);
    _uncautioned(reply, &words).iter().any(|w| !said_already.contains(w) && !said_already.contains(drop_last(w)))
}

/// `word[:-1]`.
fn drop_last(w: &str) -> &str {
    match w.char_indices().last() {
        Some((i, _)) => &w[..i],
        None => w,
    }
}

lazy!(GLUTEN_FREE_ALREADY: Re = "GLUTEN_FREE_ALREADY");
lazy!(PLANT_SAID: Re = "PLANT_SAID");
lazy!(SUBSTITUTE_ASK: Re = "SUBSTITUTE_ASK");

/// A reply fitted to the eater's diet (swaps made, a sentence dropped), or None when it cannot be.
pub fn fits_the_diet(reply: Option<&str>, banned: &HashSet<String>, line: &str) -> Option<String> {
    let Some(reply) = reply else { return None };
    if text::strip(reply).is_empty() || banned.is_empty() {
        return Some(reply.to_string());
    }
    let mut text_ = reply.to_string();
    let mut probe: Vec<String> = _the_banned_words(banned).into_iter().collect();
    for w in strs("GLUTEN_WORDS") {
        if !probe.contains(&w) {
            probe.push(w);
        }
    }
    let line_words: HashSet<String> = _says_a_food(line, &probe).into_iter().collect();
    if meets(&GLUTEN, banned) {
        for row in g("GLUTEN_SWAPS").items() {
            let row = row.items();
            if !line_words.contains(row[0].str()) {
                text_ = _rewritten_outside_swaps(&text_, row[1].as_re().unwrap(), row[2].str(), Some(&GLUTEN_FREE_ALREADY));
            }
        }
    }
    let words: Vec<String> = _the_banned_words(banned).into_iter().collect();
    if words.is_empty() {
        return Some(text_);
    }
    let low_line = line.to_lowercase();
    if _ask_about_the_food(&low_line, low_line.len()) {
        return Some(text_);
    }
    let mut said_already: HashSet<String> = _says_a_food(line, &words).into_iter().collect();
    let stems: Vec<String> = said_already.iter().filter(|w| w.ends_with('s')).map(|w| w[..w.len() - 1].to_string()).collect();
    said_already.extend(stems);
    let left: Vec<String> = _uncautioned(&text_, &words)
        .into_iter()
        .filter(|w| !said_already.contains(w) && !said_already.contains(drop_last(w)))
        .collect();
    if left.is_empty() {
        return Some(text_);
    }
    let nuts_only = meets(&NUTS, banned) && banned.iter().all(|w| NUTS.contains(w));
    if meets(&MEAT, banned) || meets(&FISH, banned) || nuts_only {
        text_ = diet_variant(&text_);
    }
    if meets(&DAIRY, banned) && !banned.contains("lactose") && !PLANT_SAID.is_match(&text_) {
        for row in g("DAIRY_SWAPS").items() {
            let row = row.items();
            text_ = _rewritten_outside_swaps(&text_, row[0].as_re().unwrap(), row[1].str(), None);
        }
    }
    text_ = _nut_swapped(&text_, banned, line);
    let mut said = _says_a_food(&text_, &words);
    if PLANT_SAID.is_match(&text_) {
        let (meat, fish, egg) = (set("MEAT_SAYS"), set("FISH_SAYS"), set("EGG_SAYS"));
        said.retain(|w| !DAIRY_SAYS.contains(w) && !egg.contains(w) && !meat.contains(w) && !fish.contains(w));
    }
    let left: Vec<String> = said.into_iter().filter(|w| !said_already.contains(w)).collect();
    if left.is_empty() {
        return Some(text_);
    }
    if SUBSTITUTE_ASK.is_match(&low_line) {
        return Some(text_);
    }
    if SAYS_THE_FOOD_IS_OUT.is_match(&text_) {
        return Some(text_);
    }
    if let Some(without) = _the_card_without_that_sentence(&text_, &left, &words) {
        if !without.is_empty() {
            return Some(without);
        }
    }
    let stripped = text::strip(&text_).to_string();
    let first = match SENTENCE_BREAK.search(&stripped) {
        Some(m) => stripped[..m.start()].to_string(),
        None => stripped.clone(),
    };
    if !_says_a_food(&first, &words).iter().any(|w| left.contains(w) || left.iter().any(|l| l == drop_last(w))) {
        return Some(text_);
    }
    None
}

re_lazy!(CARD_PIECES = r"([.!?]+\s+|;\s+|,\s+(?:then|and)\s+)");

/// The card with the clauses that name a banned food dropped, when enough of it is left.
pub fn _the_card_without_that_sentence(text_: &str, left: &[String], words: &[String]) -> Option<String> {
    let bits = CARD_PIECES.split(text::strip(text_));
    let mut keep: Vec<(usize, &str)> = Vec::new();
    let mut dropped = 0;
    for index in (0..bits.len()).step_by(2) {
        let part = bits[index].as_str();
        let names = _says_a_food(part, words);
        if names.iter().any(|w| left.contains(w) || left.iter().any(|l| l == drop_last(w))) {
            dropped += 1;
            continue;
        }
        keep.push((index, part));
    }
    if dropped == 0 || keep.is_empty() {
        return None;
    }
    if text::strip_chars(keep[0].1, " ,.;:").is_empty() || keep[0].0 != 0 {
        return None;
    }
    let pieces: Vec<String> = keep
        .iter()
        .enumerate()
        .filter(|(_, (_, part))| !text::strip_chars(part, " ,.;:").is_empty())
        .map(|(index, (_, part))| {
            if index != 0 { text::strip_chars(part, " ,.;:").to_lowercase() } else { text::strip_chars(part, " ,;").to_string() }
        })
        .collect();
    let said = pieces.join(", ");
    if said.is_empty() {
        return None;
    }
    let n = said.chars().count() as f64;
    if n < 40.0 || n < 0.55 * text::strip(text_).chars().count() as f64 {
        return None;
    }
    let said = text::upper_first(&said);
    if said.ends_with('.') || said.ends_with('!') || said.ends_with('?') { Some(said) } else { Some(format!("{said}.")) }
}

/// Ten-minute dinners that keep clear of what the eater rules out.
pub fn _any_meal(banned: &HashSet<String>) -> String {
    let no_egg = banned.contains("egg") || banned.contains("eggs");
    if meets(&GLUTEN, banned) {
        let egg = if no_egg { "" } else { " and an egg" };
        let cheese = if meets(&DAIRY, banned) { "hummus" } else { "cheese" };
        return format!("Ten-minute dinners, all gluten free: rice fried with frozen veg{egg}; a jacket potato with beans and {cheese}; or a chickpea and tomato curry with rice. Tell me what is in the fridge and I will pick one.");
    }
    if no_egg {
        if meets(&DAIRY, banned) {
            return "Ten-minute dinners: a chickpea and tomato curry with rice; pasta with garlic, chilli and olive oil; or beans on toast with hummus. Tell me what is in the fridge and I will pick one.".into();
        }
        return "Ten-minute dinners: a chickpea and tomato curry with rice; pasta with garlic, chilli and olive oil; or beans on toast with a little cheese or hummus. Tell me what is in the fridge and I will pick one.".into();
    }
    if MEAT.iter().all(|w| banned.contains(w)) {
        return "Ten-minute dinners: pasta with garlic, chilli and olive oil; a chickpea and tomato curry with rice; or eggs on toast. Tell me what is in the fridge and I will pick one.".into();
    }
    ANY_MEAL.clone()
}

// ---------------------------------------------------------------------------------------------
// Cauliflower rice, pulses, a plant or a pet that is not dinner, meal ideas, the pantry

/// Cauliflower rice, with the carb count when the line is about carbs.
pub fn _cauliflower_rice(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words: HashSet<String> = understood.words.iter().map(|w| _fold(w)).collect();
    let method = "Grate or blitz a head of cauliflower into rice-sized bits, wrap it in a tea towel and squeeze out the water, then fry it in a little oil for 4 to 5 minutes with salt until it is hot but still has a bite. Do not boil it: that is what makes it soggy. One head makes four good portions and keeps three days in the fridge.";
    if ["carb", "carbs", "keto", "diet", "diabetes", "sugar", "low"].iter().any(|w| words.contains(*w)) {
        return Some(format!("Cauliflower is one of the lowest-carb vegetables there is: about 3g of carbs per 100g, against about 28g in cooked rice, so it works on a low-carb or diabetic plate. {method}"));
    }
    Some(method.into())
}

re_lazy!(RECIPE_TAIL = r"\brecipe\b(?:\s+(?:for|of|to make))?\s+(.*)$");
lazy!(_NOT_A_DISH: HashSet<String> = "_NOT_A_DISH");
lazy!(_FOOD: Vec<String> = "_FOOD");

/// True when the line asks for a recipe for a dish none of whose words the tables know.
pub fn _recipe_for_unknown_dish(lowered: &str) -> bool {
    let Some(m) = RECIPE_TAIL.search(lowered) else { return false };
    let tail = LETTER_WORDS.findall_str(m.str(1));
    if tail.is_empty() {
        return false;
    }
    if tail.iter().any(|w| ["idea", "ideas", "meal", "dinner", "tea", "supper"].contains(&w.as_str())) {
        return false;
    }
    let food_words: Vec<&String> = tail.iter().filter(|w| !_NOT_A_DISH.contains(*w)).take(3).collect();
    if food_words.is_empty() {
        return false;
    }
    let ingredients = g("_INGREDIENT_WORDS");
    food_words.iter().all(|w| ingredients.get(w).is_none() && !_FOOD.contains(w))
}

/// The first sentence of an idea.
pub fn _first_sentence(idea: &str) -> String {
    let stripped = text::strip(idea);
    match SENTENCE_BREAK.search(stripped) {
        Some(m) => stripped[..m.start()].to_string(),
        None => stripped.to_string(),
    }
}

lazy!(LEFTOVER_WORDS: HashSet<String> = "LEFTOVER_WORDS");
lazy!(LEFTOVER_PHRASES: Vec<String> = "LEFTOVER_PHRASES");

/// True when the line is about leftovers.
pub fn _leftover(folded: &[String], clean: &str) -> bool {
    folded.iter().any(|w| LEFTOVER_WORDS.contains(w)) || any_in(clean, &LEFTOVER_PHRASES)
}

lazy!(_PULSES_FRESH: Re = "_PULSES_FRESH");
lazy!(_PULSES_DRIED: Re = "_PULSES_DRIED");
lazy!(_PULSES_TIN: Re = "_PULSES_TIN");
lazy!(_PULSES_HARD: Re = "_PULSES_HARD");
lazy!(_PULSES_APPLIANCE: Re = "_PULSES_APPLIANCE");
lazy!(_PULSES_KIDNEY: Re = "_PULSES_KIDNEY");
lazy!(_PULSES_NO_SOAK: Re = "_PULSES_NO_SOAK");

/// Dried beans and chickpeas: soaking, cooking, and beans that stay hard.
pub fn _pulses(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = _words_and_text(understood);
    if _PULSES_FRESH.is_match(&said) && !said.contains("split peas") && !_PULSES_DRIED.is_match(&said) {
        return None;
    }
    if _PULSES_TIN.is_match(&said) && !_PULSES_HARD.is_match(&said) {
        return None;
    }
    if _PULSES_APPLIANCE.is_match(&said) && !_PULSES_KIDNEY.is_match(&said) {
        return None;
    }
    if _PULSES_HARD.is_match(&said) {
        return Some("Old beans are usually the reason: a bag that has sat in the cupboard for years takes twice as long however you cook it. Keep it at a bare simmer with the lid just off, top it up with hot water, and give it another 20 to 30 minutes with a half teaspoon of bicarbonate of soda in the pan - and leave the salt, tomato and vinegar until the beans are soft, because acid early is the other thing that keeps the skins tough. Hard water does the same, so use filtered water if yours is very chalky.".into());
    }
    if _PULSES_NO_SOAK.is_match(&said) {
        return Some("You can skip the soaking: boil them hard for 2 minutes, take the pan off the heat and leave it covered for an hour, then simmer as usual - or cook them straight from the bag for 30 to 45 minutes longer and top up with hot water. Kidney beans are the one exception: boil them hard for 10 minutes before the simmer, and never cook them raw in a slow cooker.".into());
    }
    Some("Dried chickpeas and beans want 8 hours of cold water, then 45 to 60 minutes of hard simmering; lentils need no soaking at all, just 20 to 30 minutes in the pan. Kidney beans must be boiled hard for 10 minutes, never slow cooked raw.".into())
}

/// Cooking lentils.
pub fn _lentils(_memory: &mut Memory, _understood: &Understood) -> Option<String> {
    Some("No soaking: rinse and simmer in plenty of water. Red lentils split and go soft in 15 to 20 minutes, which soups and dal want; green or brown stay whole at 20 to 25; puy take about 25 and hold their shape in salads. Salt near the end, not at the start, and taste at 15 minutes: a minute either way is firm against mushy. One mug of dried makes two cooked, and a bay leaf and half an onion in the water do more for flavour than anything added later.".into())
}

/// True when the line names exactly one ingredient and the meal idea does not use it.
pub fn _keeps_its_own_name(memory: &Memory, understood: &Understood, idea: &str) -> bool {
    let words: Vec<String> = understood.words.iter().map(|w| _fold(w)).collect();
    let ingredients = g("_INGREDIENT_WORDS");
    let mut named: Vec<String> = Vec::new();
    for (place, word) in words.iter().enumerate() {
        let Some(name) = ingredients.get(word) else { continue };
        let name = name.str();
        if !name.is_empty() && _fold(name) == *word && !named.contains(word) {
            named.push(word.clone());
        }
        if place + 1 < words.len() {
            let pair = format!("{} {}", word, words[place + 1]);
            if let Some(p) = ingredients.get(&pair) {
                if p.truthy() && _fold(p.str()) == pair {
                    named.push(pair);
                }
            }
        }
    }
    if named.len() != 1 {
        return false;
    }
    let text_ = words.join(" ");
    if words.iter().any(|w| LEFTOVER_WORDS.contains(w)) || any_in(&text_, &LEFTOVER_PHRASES) {
        return false;
    }
    if !meals_diet_said(memory, None).is_empty() {
        return false;
    }
    if ["old", "stale", "dry", "yesterday"].iter().any(|m| text_.contains(&format!("{m} {}", named[0]))) {
        return false;
    }
    if meals_is_the_ruled_out_food(&named[0], &words.join(" ")) {
        return false;
    }
    let said: Vec<String> = LETTER_WORDS.findall_str(&idea.to_lowercase()).iter().map(|w| _fold(w)).collect();
    !said.contains(&named[0])
}

lazy!(_ORDER_NOT_A_MEAL: Vec<String> = "_ORDER_NOT_A_MEAL");
lazy!(_MILK_IN_TEA_NOT_A_MEAL: Vec<String> = "_MILK_IN_TEA_NOT_A_MEAL");

/// True when the line is a drink order or milk in a brew, not a dinner.
pub fn _a_brew_not_a_dinner(line: &str) -> bool {
    let low = format!(" {} ", line.to_lowercase());
    if _ORDER_NOT_A_MEAL.iter().any(|p| low.contains(&format!(" {p} "))) {
        return true;
    }
    if !low.contains(" milk ") && !low.starts_with(" milk") {
        return false;
    }
    _MILK_IN_TEA_NOT_A_MEAL.iter().any(|p| low.contains(&format!(" {p} ")) || low.contains(&format!(" {p}.")))
}

/// True when the line is an order at a counter, not a dinner.
pub fn _an_order_not_a_dinner(line: &str) -> bool {
    let low = format!(" {} ", line.to_lowercase());
    _ORDER_NOT_A_MEAL.iter().any(|p| low.contains(&format!(" {p} ")))
}

/// True when the garden's entries answer this line (the plant is a garden plant).
pub fn _the_garden_has_this_plant(line: &str) -> bool {
    lookup_find_in_garden(line)
}

/// The garden table's own matcher (`find(reading, entries=garden.ENTRIES) is not None`).
fn lookup_find_in_garden(line: &str) -> bool {
    super::d2dup_cooka::lookup_find_in_garden(line)
}

lazy!(_PLANT_WOODY: Vec<String> = "_PLANT_WOODY");
lazy!(_PLANT_PURCHASE: Vec<String> = "_PLANT_PURCHASE");

/// True when the line is a plant bought at a garden centre.
pub fn _a_plant_from_the_shop(line: &str) -> bool {
    let low = format!(" {} ", line.to_lowercase().replace('\'', " "));
    _PLANT_WOODY.iter().any(|w| low.contains(&format!(" {w} "))) && _PLANT_PURCHASE.iter().any(|w| low.contains(&format!(" {w} ")))
}

lazy!(_PLANT_NOT_A_DINNER: Vec<String> = "_PLANT_NOT_A_DINNER");
lazy!(_PLANT_KITCHEN_PLANT: Vec<String> = "_PLANT_KITCHEN_PLANT");
lazy!(_PLANT_GARDEN_VERB: Vec<String> = "_PLANT_GARDEN_VERB");
lazy!(_PLANT_BEARS: Vec<String> = "_PLANT_BEARS");
lazy!(_PLANT_COOK_ASK: Vec<String> = "_PLANT_COOK_ASK");
lazy!(_PLANT_DO_WITH: Vec<String> = "_PLANT_DO_WITH");
lazy!(_PLANT_SOURCE: Re = "_PLANT_SOURCE");

/// True when the line is about a plant in the garden, not a vegetable for dinner.
pub fn _a_plant_not_a_dinner(line: &str) -> bool {
    let low = format!(" {} ", line.to_lowercase().replace('\'', " "));
    let has = |xs: &[String]| xs.iter().any(|w| low.contains(&format!(" {w} ")));
    if !has(&_PLANT_NOT_A_DINNER) {
        return false;
    }
    if has(&_PLANT_KITCHEN_PLANT) && ["bought", "buy", "buying", "got", "picked up"].iter().any(|w| low.contains(&format!(" {w} "))) {
        return false;
    }
    if has(&_PLANT_GARDEN_VERB) {
        return _the_garden_has_this_plant(line) || _a_plant_from_the_shop(line);
    }
    if has(&_PLANT_BEARS) {
        return false;
    }
    let named: usize = _PLANT_NOT_A_DINNER.iter().map(|w| text::count(&low, &format!(" {w} "))).sum();
    let asked = has(&_PLANT_COOK_ASK) || has(&_PLANT_DO_WITH);
    if named > 0 && asked && _PLANT_SOURCE.findall(&low.replace("  ", " ")).len() >= named {
        return false;
    }
    if has(&_PLANT_COOK_ASK) {
        return false;
    }
    _the_garden_has_this_plant(line) || _a_plant_from_the_shop(line)
}

lazy!(FEEDING_NOT_FOOD: Vec<String> = "FEEDING_NOT_FOOD");
lazy!(FEEDING_EATS: Re = "FEEDING_EATS");
lazy!(FEEDING_GIVEN: Re = "FEEDING_GIVEN");
lazy!(FEEDING_SAFE_FOR: Re = "FEEDING_SAFE_FOR");

/// True when the line asks what an animal or a baby can be fed.
pub fn _feeding_ask(line: &str) -> bool {
    let low = format!(" {} ", line.to_lowercase());
    if any_in(&low, &FEEDING_NOT_FOOD) {
        return false;
    }
    FEEDING_EATS.is_match(&low) || FEEDING_GIVEN.is_match(&low) || FEEDING_SAFE_FOR.is_match(&low)
}

lazy!(PET_ANIMALS: Vec<String> = "PET_ANIMALS");
lazy!(_PET_OWNED: Vec<String> = "_PET_OWNED");
lazy!(_PET_EATING: Vec<String> = "_PET_EATING");

/// True when the animal in the line is a pet (owned, or eating), not dinner.
pub fn _pet_not_dinner(line: &str) -> bool {
    let low = format!(" {} ", line.to_lowercase());
    let animals: Vec<&String> = PET_ANIMALS.iter().filter(|a| low.contains(&format!(" {a} "))).collect();
    if animals.is_empty() {
        return false;
    }
    if animals.iter().any(|a| _PET_OWNED.iter().any(|o| low.contains(&format!(" {o}{a} ")))) {
        return true;
    }
    _PET_EATING.iter().any(|v| low.contains(&format!(" {v} ")))
}

lazy!(PET_NOT_FOOD: Vec<String> = "PET_NOT_FOOD");
lazy!(PET_MEAT_ANIMALS: Vec<String> = "PET_MEAT_ANIMALS");
lazy!(PET_HOME: Re = "PET_HOME");

/// True when the line reads as about a pet.
pub fn _reads_as_a_pet(line: &str) -> bool {
    let low = format!(" {} ", line.to_lowercase());
    if any_in(&low, &PET_NOT_FOOD) {
        return false;
    }
    if _pet_not_dinner(line) {
        return true;
    }
    let owned = PET_MEAT_ANIMALS.iter().any(|a| low.contains(&format!("my {a} ")) || low.contains(&format!("our {a} ")));
    owned && PET_HOME.is_match(&low)
}

/// "How do I cook my rabbit": the pet answer, with the meat's own idea if that is what was meant.
pub fn _pet_or_meat(line: &str) -> Option<String> {
    let low = line.to_lowercase();
    if !any_in(&low, &strs("PET_COOKING_ASK")) {
        return None;
    }
    if any_in(&low, &strs("PET_CARE")) {
        return None;
    }
    if !_reads_as_a_pet(&low) {
        return None;
    }
    for (name, row) in g("PANTRY").pairs() {
        let name = name.str();
        if !PET_ANIMALS.iter().any(|a| a == name) && !PET_MEAT_ANIMALS.iter().any(|a| a == name) {
            continue;
        }
        if !low.contains(&format!(" {name} ")) && !low.contains(&format!(" {name}s ")) {
            continue;
        }
        let meat = row.items()[1].str().to_string();
        let (head, _, rest) = text::partition(&meat, ": ");
        let mut rest = rest.to_string();
        let head_first = text::split(&head.to_lowercase()).into_iter().next();
        if rest.is_empty() || head.is_empty() || !name.contains(head_first.expect("IndexError").as_str()) {
            rest = meat.clone();
        }
        let rest = text::strip(&rest).to_string();
        if rest.is_empty() {
            return None;
        }
        let mut chars = rest.chars();
        let first: String = chars.next().unwrap().to_lowercase().collect();
        return Some(format!(
            "That reads as a pet rather than a dinner, and I would keep it off the plate. If you meant the meat, {first}{}",
            chars.as_str()
        ));
    }
    None
}

lazy!(_PLAN_ASK: Re = "_PLAN_ASK");
lazy!(MEAL_ASK_PHRASES: Vec<String> = "MEAL_ASK_PHRASES");
re_lazy!(MINUTES_FOR = r"\bfor\s+\d{1,3}\s*(?:min|mins|minute|minutes|hour|hours|sec|seconds)\b");
re_lazy!(WITHOUT_WORD = r"\bwithout\b");

/// Dinner ideas: a pet or a brew stands down, a table gets the crowd dish, else the meal
/// planner's idea, the pantry's, or ten-minute dinners that fit what the eater rules out.
pub fn _meal_ideas(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = &understood.words;
    let joined = words.join(" ");
    let line = or(&understood.clean, &joined).to_string();
    let mut text_ = line.clone();
    // An empty turn list is Python's absent "turns" (the opening line).
    if !memory.turns.is_empty() {
        let start = memory.turns.len().saturating_sub(4);
        let earlier: Vec<String> = memory.turns[start..].iter().map(|t| t.user.clone()).collect();
        text_.push(' ');
        text_.push_str(&earlier.join(" "));
    }
    let low_line = line.to_lowercase();
    if _pet_not_dinner(&line) {
        return _pet_or_meat(&line);
    }
    if _a_brew_not_a_dinner(&line) {
        return None;
    }
    if _PLAN_ASK.is_match(&low_line) {
        return Some("Pick four dinners you already cook and put them on the fridge, then shop for just those - one double batch goes in the freezer for the night you are late home. Keep one night loose for eggs on toast or a takeaway, and plan whatever spoils first for the day after you shop.".into());
    }
    if ["dessert", "puddings", "pudding", "afters", "ice cream", "sweet thing"].iter().any(|w| low_line.contains(w)) {
        return None;
    }
    let mut banned = _no_meat_also_means_fish(&_excluded(&text_));
    banned.extend(_short_of_food(&text_));
    banned.extend(negated_ingredients(understood));
    let table = MINUTES_FOR.sub(" ", &text_).to_lowercase();
    let asking_words = [
        "what", "idea", "ideas", "something", "suggest", "suggestion", "dinner", "dinners", "meal", "meals", "menu", "tea",
        "supper", "lunch", "cook for", "make for", "feed", "recipe", "anything", "any", "need", "want", "party", "guests",
        "everyone",
    ];
    let asking = understood.intent != "news"
        && (understood.kind == "question"
            || understood.intent == "question"
            || understood.intent == "request"
            || asking_words.iter().any(|w| low_line.contains(w)));
    let table_count = _crowd(&table);
    let cook_asks = ["what should i cook", "what can i cook", "what shall i cook", "what to cook", "what to make", "meal", "dinner", "cook", "no idea"];
    let at_table = asking
        && !meals_says_a_diet(&line)
        && (table_count >= 6
            || (table_count >= 3 && any_in(&low_line, &_CROWD_PHRASES))
            || (_occasion(&table) && cook_asks.iter().any(|p| low_line.contains(p))));
    if at_table {
        return Some(_crowd_or_any(&table, &banned));
    }
    let mut idea = meals_suggest(memory, understood);
    if !idea.is_empty() && !banned.is_empty() && _bans(&[], &idea, &banned) {
        idea = String::new();
    }
    if !idea.is_empty() && _keeps_its_own_name(memory, understood, &idea) {
        let mut all: Vec<String> = words.clone();
        all.extend(text::split(&understood.clean));
        if let Some((_, own)) = _pantry_idea(&all, &banned, &low_line) {
            if !text::strip(&own).is_empty() {
                idea = own;
            }
        }
    }
    if !idea.is_empty() {
        return Some(idea);
    }
    let asks_food = matches!(understood.topic.as_deref(), Some("cooking") | Some("food-and-drink")) || any_in(&low_line, &MEAL_ASK_PHRASES);
    if asks_food {
        if let Some(from_pantry) = meal(understood, MealArgs { banned: Some(banned.clone()), memory: Some(memory), ..Default::default() }) {
            if !text::strip(&from_pantry).is_empty() && !_bans(&[], &from_pantry, &banned) {
                return Some(from_pantry);
            }
        }
    }
    if meals_unknown_food(understood) {
        return None;
    }
    if banned.is_empty() && WITHOUT_WORD.is_match(&low_line) && !meals_says_a_diet(&line) {
        return None;
    }
    if _says_diabetes(&text_) {
        return Some(st("DIABETIC_PLATE"));
    }
    Some(_any_meal(&banned))
}

/// The pantry's quick idea for one food name, or None.
pub fn pantry_idea(name: &str) -> Option<String> {
    let folded = _fold(&name.to_lowercase());
    if folded.is_empty() {
        return None;
    }
    let folded_s = folded.trim_end_matches('s');
    for (_, row) in g("PANTRY").pairs() {
        let row = row.items();
        for alias in row[0].strs() {
            let key = _fold(&alias.to_lowercase());
            if key == folded || key == folded_s || key.trim_end_matches('s') == folded {
                return Some(row[1].str().to_string());
            }
        }
    }
    None
}

/// The pantry idea for the food the words name first (longest name on a tie), skipping what
/// the eater rules out: (where it was named, the idea).
pub fn _pantry_idea(words: &[String], banned: &HashSet<String>, clean: &str) -> Option<(usize, String)> {
    let mut found: Option<(usize, String)> = None;
    let mut reached: Option<(usize, i64)> = None;
    let not_this = g("NOT_THIS_FOOD");
    let folded_words: Vec<String> = words.iter().map(|w| _fold(w)).collect();
    for (name, row) in g("PANTRY").pairs() {
        let row = row.items();
        let aliases = row[0].strs();
        let idea = row[1].str();
        if _bans(&aliases, idea, banned) {
            continue;
        }
        if not_this.get(name.str()).is_some_and(|v| any_in(clean, &v.strs())) {
            continue;
        }
        for alias in &aliases {
            let parts: Vec<String> = text::split(alias).iter().map(|p| _fold(p)).collect();
            if folded_words.len() < parts.len() {
                continue;
            }
            for start in 0..=(folded_words.len() - parts.len()) {
                if (0..parts.len()).all(|o| folded_words[start + o] == parts[o]) {
                    let spot = (start, -(parts.len() as i64));
                    if reached.is_none_or(|r| spot < r) {
                        reached = Some(spot);
                        found = Some((start, idea.to_string()));
                    }
                    break;
                }
            }
        }
    }
    found
}

/// Porridge made to fit what the eater rules out.
pub fn _porridge(banned: &HashSet<String>) -> String {
    if meets(&GLUTEN, banned) {
        let milk = if meets(&DAIRY, banned) { "oat milk" } else { "milk" };
        return format!("Porridge made with gluten free oats and {milk}");
    }
    if meets(&DAIRY, banned) {
        return "Porridge made with oat milk and a spoon of jam".into();
    }
    "Porridge with a banana and a spoon of jam".into()
}

/// BREAKFAST_CHOICES' `lambda banned: _porridge(banned)` (bot.knowledge.cooking:<lambda>:5984).
pub fn breakfast_porridge_lambda_5984(banned: &HashSet<String>) -> String {
    _porridge(banned)
}

// ---------------------------------------------------------------------------------------------
// The meals module (D1), until it is merged

fn meals_suggest(memory: &mut Memory, understood: &Understood) -> String {
    crate::knowledge::d2_shared::call_reply("bot.knowledge.meals:suggest:1997", memory, understood)
}

use super::d2dup_cooka::{meals_diet_said, meals_is_the_ruled_out_food, meals_says_a_diet, meals_unknown_food};

// ---------------------------------------------------------------------------------------------
// The registry tables: every guard and reply of an entry defined in lines 1-5999.

macro_rules! reply {
    ($id:expr, $f:path) => {
        ($id, |_: &FnRef, m: &mut Memory, u: &Understood| $f(m, u))
    };
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.cooking:_a_soft_fruit_is_still_a_cooking_question:1827", |_: &FnRef, u: &Understood| _a_soft_fruit_is_still_a_cooking_question(u)),
    ("bot.knowledge.cooking:_not_a_portion_ask:719", |_: &FnRef, u: &Understood| _not_a_portion_ask(u)),
    ("bot.knowledge.cooking:_tenders_are_a_piece:1006", |_: &FnRef, u: &Understood| _tenders_are_a_piece(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    reply!("bot.knowledge.cooking:_bake_fruit:1842", _bake_fruit),
    reply!("bot.knowledge.cooking:_baked_keep.<locals>.reply:134", _baked_keep_reply),
    reply!("bot.knowledge.cooking:_body:617", _body),
    reply!("bot.knowledge.cooking:_bread:2115", _bread),
    reply!("bot.knowledge.cooking:_cauliflower_rice:5282", _cauliflower_rice),
    reply!("bot.knowledge.cooking:_chicken_time:1093", _chicken_time),
    reply!("bot.knowledge.cooking:_chocolate:2991", _chocolate),
    reply!("bot.knowledge.cooking:_container_keep:99", _container_keep),
    reply!("bot.knowledge.cooking:_cookies:2969", _cookies),
    reply!("bot.knowledge.cooking:_dough:2288", _dough),
    reply!("bot.knowledge.cooking:_dry:2034", _dry),
    reply!("bot.knowledge.cooking:_egg_fault:2899", _egg_fault),
    reply!("bot.knowledge.cooking:_eggs:2629", _eggs),
    reply!("bot.knowledge.cooking:_falls_apart:2164", _falls_apart),
    reply!("bot.knowledge.cooking:_flat:3184", _flat),
    reply!("bot.knowledge.cooking:_grainy:3060", _grainy),
    reply!("bot.knowledge.cooking:_kids_veg:680", _kids_veg),
    reply!("bot.knowledge.cooking:_learn_to_cook:457", _learn_to_cook),
    reply!("bot.knowledge.cooking:_legs:952", _legs),
    reply!("bot.knowledge.cooking:_lentils:5405", _lentils),
    reply!("bot.knowledge.cooking:_lumpy:2585", _lumpy),
    reply!("bot.knowledge.cooking:_meal_ideas:5752", _meal_ideas),
    reply!("bot.knowledge.cooking:_meal_plan:1929", _meal_plan),
    reply!("bot.knowledge.cooking:_meat_defrost:3307", _meat_defrost),
    reply!("bot.knowledge.cooking:_mince:3564", _mince),
    reply!("bot.knowledge.cooking:_mince_grainy:3538", _mince_grainy),
    reply!("bot.knowledge.cooking:_more_veg:664", _more_veg),
    reply!("bot.knowledge.cooking:_pancake_batch:847", _pancake_batch),
    reply!("bot.knowledge.cooking:_pancakes:820", _pancakes),
    reply!("bot.knowledge.cooking:_pasta_cook:2640", _pasta_cook),
    reply!("bot.knowledge.cooking:_popcorn_keep:2682", _popcorn_keep),
    reply!("bot.knowledge.cooking:_popcorn_make:2653", _popcorn_make),
    reply!("bot.knowledge.cooking:_protein:1734", _protein),
    reply!("bot.knowledge.cooking:_pulses:5371", _pulses),
    reply!("bot.knowledge.cooking:_rest_the_meat:575", _rest_the_meat),
    reply!("bot.knowledge.cooking:_roast_joint:3479", _roast_joint),
    reply!("bot.knowledge.cooking:_roast_shopping:3722", _roast_shopping),
    reply!("bot.knowledge.cooking:_salty:2838", _salty),
    reply!("bot.knowledge.cooking:_scramble:2851", _scramble),
    reply!("bot.knowledge.cooking:_sides:1465", _sides),
    reply!("bot.knowledge.cooking:_soup:3169", _soup),
    reply!("bot.knowledge.cooking:_staple:730", _staple),
    reply!("bot.knowledge.cooking:_stuck_jar:119", _stuck_jar),
    reply!("bot.knowledge.cooking:_sugar_work:3112", _sugar_work),
    reply!("bot.knowledge.cooking:_sweet_potato:888", _sweet_potato),
    reply!("bot.knowledge.cooking:_tenders_method:1023", _tenders_method),
    reply!("bot.knowledge.cooking:_thighs:935", _thighs),
    reply!("bot.knowledge.cooking:_time_budget:1669", _time_budget),
    reply!("bot.knowledge.cooking:_tough:2740", _tough),
    reply!("bot.knowledge.cooking:_turkey:3357", _turkey),
    reply!("bot.knowledge.cooking:_turkey_cut:3394", _turkey_cut),
    reply!("bot.knowledge.cooking:_watery:2406", _watery),
];


// ---------------------------------------------------------------------------------------------
// The factories: the closures they build (as `FnRef`s the registry runs) and the import-time fold
// of the meal ideas into the pantry, whose result is the bundle's `PANTRY`.

/// `_baked_keep()`: the keeping answer for the small baked things (`_baked_keep_reply`).
pub fn _baked_keep() -> crate::data::FnRef {
    crate::knowledge::built::function("bot.knowledge.cooking:_baked_keep.<locals>.reply:134")
}

/// `_bread_rise_answer()`: why a loaf is slow, worded for the prove the person is doing.
pub fn _bread_rise_answer() -> crate::data::FnRef {
    crate::knowledge::built::function("bot.knowledge.cooking:_bread_rise_answer.<locals>.answer:8098")
}

/// `_fold_meal_ideas()`: fold the older meal-ideas list into the pantry table, so one table
/// answers every way of asking. Python mutates `PANTRY` in place at import; this folds the
/// `meal_ideas` rows into the `pantry` given ({name: (aliases, idea)}, in order).
pub fn _fold_meal_ideas(
    pantry: &mut indexmap::IndexMap<String, (Vec<String>, String)>,
    meal_ideas: &[(Vec<String>, String)],
    vague_words: &[String],
) {
    use crate::knowledge::cooking_b::_fold;
    let vague: HashSet<String> = vague_words.iter().map(|w| _fold(w)).collect();
    let names: Vec<String> = pantry.keys().cloned().collect();
    for name in names {
        let (aliases, idea) = pantry[&name].clone();
        let kept: Vec<String> = aliases.iter().filter(|a| !vague.contains(&_fold(a))).cloned().collect();
        if !kept.is_empty() && kept != aliases {
            pantry.insert(name, (kept, idea));
        }
    }
    let mut claimed: indexmap::IndexMap<String, String> = indexmap::IndexMap::new();
    for (name, (aliases, _idea)) in pantry.iter() {
        claimed.entry(name.clone()).or_insert_with(|| name.clone());
        for alias in aliases {
            claimed.entry(_fold(alias)).or_insert_with(|| name.clone());
        }
    }
    for (names, idea) in meal_ideas {
        let group: Vec<String> = names.iter().filter(|n| vague.contains(&_fold(n))).cloned().collect();
        let solid: Vec<&String> = names.iter().filter(|n| !vague.contains(&_fold(n))).collect();
        if !group.is_empty() && solid.iter().all(|n| claimed.contains_key(&_fold(n))) {
            let key = _fold(&group[0]);
            pantry.entry(key.clone()).or_insert_with(|| (group.clone(), idea.clone()));
            for name in names {
                claimed.entry(_fold(name)).or_insert_with(|| key.clone());
            }
            continue;
        }
        let mut key: Option<String> = None;
        for name in names {
            if vague.contains(&_fold(name)) {
                continue;
            }
            key = claimed.get(&_fold(name)).cloned();
            break;
        }
        let key = match key {
            None => {
                let key = _fold(&names[0]);
                let alone: Vec<String> = names.iter().filter(|n| !claimed.contains_key(&_fold(n)) || **n == names[0]).cloned().collect();
                pantry.insert(key.clone(), (alone, idea.clone()));
                key
            }
            Some(key) => {
                let (aliases, short) = pantry.get(&key).cloned().unwrap_or_else(|| panic!("KeyError: {key:?}"));
                let mut aliases = aliases;
                aliases.extend(names.iter().filter(|n| !claimed.contains_key(&_fold(n)) && !vague.contains(&_fold(n))).cloned());
                pantry.insert(key.clone(), (aliases, short));
                key
            }
        };
        for name in names {
            claimed.entry(_fold(name)).or_insert_with(|| key.clone());
        }
    }
}

/// `_recipe_only_when_asked_for_this(reply, names=(), no_yeast_ok=False)`: this method, unless
/// the line is not asking for it, wants a swap, or another way.
pub fn _recipe_only_when_asked_for_this(reply: &str, names: &[String], no_yeast_ok: bool) -> crate::data::FnRef {
    let total = crate::knowledge::cooking_b::_recipe_total(reply);
    crate::knowledge::built::closure(
        "bot.knowledge.cooking:_recipe_only_when_asked_for_this.<locals>.answer:12250",
        vec![
            ("names", crate::knowledge::built::words(names)),
            ("no_yeast_ok", Value::Bool(no_yeast_ok)),
            ("reply", Value::from(reply)),
            ("total", Value::from(total.as_str())),
        ],
    )
}

#[cfg(test)]
mod factory_tests {
    use super::*;

    /// Every recipe row's closure is what the factory makes from the row's own reply and names.
    #[test]
    fn recipe_rows_rebuild() {
        data::load();
        let mut n = 0;
        for row in crate::knowledge::entries() {
            let crate::knowledge::entry::Reply::Fn(f) = &row.reply else { continue };
            if !f.id.starts_with("bot.knowledge.cooking:_recipe_only_when_asked_for_this") {
                continue;
            }
            let ours = _recipe_only_when_asked_for_this(f.cell("reply").unwrap().str(), &f.cell("names").unwrap().strs(), f.cell("no_yeast_ok").unwrap().truthy());
            assert_eq!(ours.cell("total"), f.cell("total"), "{}", row.id);
            n += 1;
        }
        assert!(n > 50, "{n} recipe rows");
    }
}
