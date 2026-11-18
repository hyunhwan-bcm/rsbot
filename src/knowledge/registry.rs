//! Knowledge entries' guards and function replies, by the Python function they port.
//!
//! An entry's `guard` and `reply` can be Python functions (1,442 guards, 1,791 replies). The
//! bundle names each one as "module:qualname:line" with the values a closure captured
//! ([`FnRef`]). Each ported knowledge module lists its ports in `GUARDS` / `REPLIES` tables,
//! keyed by that id, and this registry runs them:
//!
//! ```ignore
//! pub const GUARDS: &[(&str, GuardFn)] = &[
//!     ("bot.knowledge.cooking:_a_made_pot_is_not_its_recipe:4321", a_made_pot_is_not_its_recipe),
//! ];
//! ```
//!
//! Python semantics: `guard_passes` treats a guard that raises as failing, and `_payload` treats a
//! reply that is not a string as declining (""). A function that has no port yet behaves the
//! same way (guard fails, reply declines) and is counted in [`misses`], so a check run shows
//! which ports are missing.

use crate::data::FnRef;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{BTreeMap, HashMap};
use std::sync::{LazyLock, Mutex};

/// `guard(reading) -> bool`.
pub type GuardFn = fn(&FnRef, &Understood) -> bool;
/// `reply(memory, reading) -> str | None`; None (or anything Python would not return as a str)
/// means the entry declines.
pub type ReplyFn = fn(&FnRef, &mut Memory, &Understood) -> Option<String>;

static GUARDS: LazyLock<HashMap<&'static str, GuardFn>> = LazyLock::new(|| {
    let mut m: HashMap<&'static str, GuardFn> = HashMap::new();
    for table in guard_tables() {
        for (k, f) in table.iter() {
            m.insert(k, *f);
        }
    }
    m
});

static REPLIES: LazyLock<HashMap<&'static str, ReplyFn>> = LazyLock::new(|| {
    let mut m: HashMap<&'static str, ReplyFn> = HashMap::new();
    for table in reply_tables() {
        for (k, f) in table.iter() {
            m.insert(k, *f);
        }
    }
    m
});

/// Every ported module's guard table. A workstream adds its module's line here.
fn guard_tables() -> Vec<&'static [(&'static str, GuardFn)]> {
    vec![
        // e.g. crate::knowledge::cooking::GUARDS,
        // U: the guards defined in bot/knowledge/__init__.py, lookup.py and understanding/
        crate::knowledge::GUARDS,
        crate::knowledge::lookup::GUARDS,
        U_LINKED_GUARDS,
        crate::understanding::cars::GUARDS,
        crate::understanding::car_glass::GUARDS,
        // D2
        crate::knowledge::differences::GUARDS,
        crate::knowledge::maths::GUARDS,
        crate::knowledge::opinions::GUARDS,
        crate::knowledge::keeping::GUARDS,
        crate::knowledge::fruit::GUARDS,
        crate::knowledge::family_more::GUARDS,
        crate::knowledge::devices::GUARDS,
        crate::knowledge::costs::GUARDS,
        crate::knowledge::cooking_gaps::GUARDS,
        crate::knowledge::babyfood::GUARDS,
        crate::knowledge::vegetables::GUARDS,
        crate::knowledge::drinks::GUARDS,
        crate::knowledge::rice::GUARDS,
        crate::knowledge::cooking_b::GUARDS,
        crate::knowledge::cooking::GUARDS,
        crate::knowledge::tenancy::GUARDS,
        crate::knowledge::swaps::GUARDS,
        crate::knowledge::still_ok::GUARDS,
        crate::knowledge::sleep::GUARDS,
        crate::knowledge::rubbish::GUARDS,
        crate::knowledge::picking::GUARDS,
        crate::knowledge::pensions::GUARDS,
        crate::knowledge::painting_faults::GUARDS,
        crate::knowledge::occasions::GUARDS,
        crate::knowledge::newborn::GUARDS,
        crate::knowledge::music::GUARDS,
        crate::knowledge::measures::GUARDS,
        crate::knowledge::left_on::GUARDS,
        crate::knowledge::joints::GUARDS,
        crate::knowledge::house_odds::GUARDS,
        crate::knowledge::history::GUARDS,
        crate::knowledge::habits::GUARDS,
        crate::knowledge::fun::GUARDS,
        crate::knowledge::friends::GUARDS,
        crate::knowledge::fears::GUARDS,
        crate::knowledge::child_work::GUARDS,
        crate::knowledge::build_jobs::GUARDS,
        crate::knowledge::appointments::GUARDS,
        crate::knowledge::animals::GUARDS,
        crate::knowledge::money::GUARDS,
        crate::knowledge::travel::GUARDS,
        crate::knowledge::entertainment::GUARDS,
        crate::knowledge::work::GUARDS,
        crate::knowledge::shopping::GUARDS,
        crate::knowledge::clothing::GUARDS,
        crate::knowledge::whenbuy::GUARDS,
        crate::knowledge::prices::GUARDS,
        crate::knowledge::kit_lists::GUARDS,
        crate::knowledge::health::GUARDS,
        crate::knowledge::cooking_entries::GUARDS,
        crate::knowledge::containers_frost::GUARDS,
        crate::knowledge::coats::GUARDS,
        crate::knowledge::plans::GUARDS,
        crate::knowledge::lifeadmin::GUARDS,
        crate::knowledge::consumer::GUARDS,
        crate::knowledge::cleaning::GUARDS,
        crate::knowledge::icing::GUARDS,
        crate::knowledge::affording::GUARDS,
        crate::knowledge::body::GUARDS,
        crate::knowledge::body_energy::GUARDS,
        crate::knowledge::business::GUARDS,
        crate::knowledge::caring::GUARDS,
        crate::knowledge::cars::GUARDS,
        crate::knowledge::chilli::GUARDS,
        crate::knowledge::comeback::GUARDS,
        crate::knowledge::complaints::GUARDS,
        crate::knowledge::cooking_more::GUARDS,
        crate::knowledge::creative::GUARDS,
        crate::knowledge::dates::GUARDS,
        crate::knowledge::decorating::GUARDS,
        crate::knowledge::dishes::GUARDS,
        crate::knowledge::explains::GUARDS,
        crate::knowledge::fabrics::GUARDS,
        crate::knowledge::family::GUARDS,
        crate::knowledge::fitness::GUARDS,
        crate::knowledge::fitweek::GUARDS,
        crate::knowledge::freshness::GUARDS,
        crate::knowledge::garden::GUARDS,
        crate::knowledge::geography::GUARDS,
        crate::knowledge::home::GUARDS,
        crate::knowledge::home_craft::GUARDS,
        crate::knowledge::income_periods::GUARDS,
        crate::knowledge::joint_time::GUARDS,
        crate::knowledge::kept_animals::GUARDS,
        crate::knowledge::kitchen_odds::GUARDS,
        crate::knowledge::kitchen_prep::GUARDS,
        crate::knowledge::language::GUARDS,
        crate::knowledge::materials::GUARDS,
        crate::knowledge::meals::GUARDS,
        crate::knowledge::nature::GUARDS,
        crate::knowledge::no_kit::GUARDS,
        crate::knowledge::nutrition::GUARDS,
        crate::knowledge::odours::GUARDS,
        crate::knowledge::panic_talk::GUARDS,
        crate::knowledge::pay::GUARDS,
        crate::knowledge::pets::GUARDS,
        crate::knowledge::pharmacy::GUARDS,
        crate::knowledge::phone_use::GUARDS,
        crate::knowledge::pregnancy::GUARDS,
        crate::knowledge::routines::GUARDS,
        crate::knowledge::schoolmaths::GUARDS,
        crate::knowledge::science::GUARDS,
        crate::knowledge::sleep_aids::GUARDS,
        crate::knowledge::sport::GUARDS,
        crate::knowledge::study::GUARDS,
        crate::knowledge::takes::GUARDS,
        crate::knowledge::technology::GUARDS,
        crate::knowledge::travel_health::GUARDS,
        crate::knowledge::vaccines::GUARDS,
        crate::knowledge::wording::GUARDS,
    ]
}

/// Every ported module's reply table. A workstream adds its module's line here.
fn reply_tables() -> Vec<&'static [(&'static str, ReplyFn)]> {
    vec![
        // U: the replies lookup.py builds (about_them, the_fault)
        crate::knowledge::lookup::REPLIES,
        // e.g. crate::knowledge::cooking::REPLIES,
        // D2
        crate::knowledge::differences::REPLIES,
        crate::knowledge::maths::REPLIES,
        crate::knowledge::opinions::REPLIES,
        crate::knowledge::keeping::REPLIES,
        crate::knowledge::fruit::REPLIES,
        crate::knowledge::family_more::REPLIES,
        crate::knowledge::devices::REPLIES,
        crate::knowledge::costs::REPLIES,
        crate::knowledge::cooking_gaps::REPLIES,
        crate::knowledge::babyfood::REPLIES,
        crate::knowledge::vegetables::REPLIES,
        crate::knowledge::drinks::REPLIES,
        crate::knowledge::rice::REPLIES,
        crate::knowledge::cooking_b::REPLIES,
        crate::knowledge::cooking::REPLIES,
        crate::knowledge::tenancy::REPLIES,
        crate::knowledge::swaps::REPLIES,
        crate::knowledge::still_ok::REPLIES,
        crate::knowledge::sleep::REPLIES,
        crate::knowledge::rubbish::REPLIES,
        crate::knowledge::picking::REPLIES,
        crate::knowledge::pensions::REPLIES,
        crate::knowledge::painting_faults::REPLIES,
        crate::knowledge::occasions::REPLIES,
        crate::knowledge::newborn::REPLIES,
        crate::knowledge::music::REPLIES,
        crate::knowledge::measures::REPLIES,
        crate::knowledge::left_on::REPLIES,
        crate::knowledge::joints::REPLIES,
        crate::knowledge::house_odds::REPLIES,
        crate::knowledge::history::REPLIES,
        crate::knowledge::habits::REPLIES,
        crate::knowledge::fun::REPLIES,
        crate::knowledge::friends::REPLIES,
        crate::knowledge::fears::REPLIES,
        crate::knowledge::child_work::REPLIES,
        crate::knowledge::build_jobs::REPLIES,
        crate::knowledge::appointments::REPLIES,
        crate::knowledge::animals::REPLIES,
        crate::knowledge::money::REPLIES,
        crate::knowledge::travel::REPLIES,
        crate::knowledge::entertainment::REPLIES,
        crate::knowledge::work::REPLIES,
        crate::knowledge::shopping::REPLIES,
        crate::knowledge::clothing::REPLIES,
        crate::knowledge::whenbuy::REPLIES,
        crate::knowledge::prices::REPLIES,
        crate::knowledge::kit_lists::REPLIES,
        crate::knowledge::health::REPLIES,
        crate::knowledge::cooking_entries::REPLIES,
        crate::knowledge::containers_frost::REPLIES,
        crate::knowledge::coats::REPLIES,
        crate::knowledge::plans::REPLIES,
        crate::knowledge::lifeadmin::REPLIES,
        crate::knowledge::consumer::REPLIES,
        crate::knowledge::cleaning::REPLIES,
        crate::knowledge::icing::REPLIES,
        crate::knowledge::affording::REPLIES,
        crate::knowledge::body::REPLIES,
        crate::knowledge::body_energy::REPLIES,
        crate::knowledge::business::REPLIES,
        crate::knowledge::caring::REPLIES,
        crate::knowledge::cars::REPLIES,
        crate::knowledge::chilli::REPLIES,
        crate::knowledge::comeback::REPLIES,
        crate::knowledge::complaints::REPLIES,
        crate::knowledge::cooking_more::REPLIES,
        crate::knowledge::creative::REPLIES,
        crate::knowledge::dates::REPLIES,
        crate::knowledge::decorating::REPLIES,
        crate::knowledge::dishes::REPLIES,
        crate::knowledge::explains::REPLIES,
        crate::knowledge::fabrics::REPLIES,
        crate::knowledge::family::REPLIES,
        crate::knowledge::fitness::REPLIES,
        crate::knowledge::fitweek::REPLIES,
        crate::knowledge::freshness::REPLIES,
        crate::knowledge::garden::REPLIES,
        crate::knowledge::geography::REPLIES,
        crate::knowledge::home::REPLIES,
        crate::knowledge::home_craft::REPLIES,
        crate::knowledge::income_periods::REPLIES,
        crate::knowledge::joint_time::REPLIES,
        crate::knowledge::kept_animals::REPLIES,
        crate::knowledge::kitchen_odds::REPLIES,
        crate::knowledge::kitchen_prep::REPLIES,
        crate::knowledge::language::REPLIES,
        crate::knowledge::materials::REPLIES,
        crate::knowledge::meals::REPLIES,
        crate::knowledge::nature::REPLIES,
        crate::knowledge::no_kit::REPLIES,
        crate::knowledge::nutrition::REPLIES,
        crate::knowledge::odours::REPLIES,
        crate::knowledge::panic_talk::REPLIES,
        crate::knowledge::pay::REPLIES,
        crate::knowledge::pets::REPLIES,
        crate::knowledge::pharmacy::REPLIES,
        crate::knowledge::phone_use::REPLIES,
        crate::knowledge::pregnancy::REPLIES,
        crate::knowledge::routines::REPLIES,
        crate::knowledge::schoolmaths::REPLIES,
        crate::knowledge::science::REPLIES,
        crate::knowledge::sleep_aids::REPLIES,
        crate::knowledge::sport::REPLIES,
        crate::knowledge::study::REPLIES,
        crate::knowledge::takes::REPLIES,
        crate::knowledge::technology::REPLIES,
        crate::knowledge::travel_health::REPLIES,
        crate::knowledge::vaccines::REPLIES,
        crate::knowledge::wording::REPLIES,
    ]
}

static MISSES: LazyLock<Mutex<BTreeMap<String, usize>>> = LazyLock::new(|| Mutex::new(BTreeMap::new()));

fn miss(id: &str) {
    *MISSES.lock().unwrap().entry(id.to_string()).or_default() += 1;
}

/// `guard_passes(item, reading)` for a function guard: false when it fails, panics, or has no port.
pub fn guard(f: &FnRef, reading: &Understood) -> bool {
    match GUARDS.get(f.id.as_str()) {
        Some(g) => crate::runtime::guarded(|| g(f, reading)).unwrap_or(false),
        None => {
            miss(&f.id);
            false
        }
    }
}

/// `_payload` for a function reply: the reply, or "" when it declines, panics, or has no port.
pub fn reply(f: &FnRef, memory: &mut Memory, reading: &Understood) -> String {
    match REPLIES.get(f.id.as_str()) {
        Some(r) => crate::runtime::guarded(|| r(f, memory, reading)).flatten().unwrap_or_default(),
        None => {
            miss(&f.id);
            String::new()
        }
    }
}

/// `reply(memory, reading)` as Python calls it inside a `try`: `Err` when it raises (or has no
/// port), else what it returned (None for a reply that is not a string).
pub fn try_reply(f: &FnRef, memory: &mut Memory, reading: &Understood) -> Result<Option<String>, ()> {
    match REPLIES.get(f.id.as_str()) {
        Some(r) => crate::runtime::guarded(|| r(f, memory, reading)).ok_or(()),
        None => {
            miss(&f.id);
            Err(())
        }
    }
}

/// `shape(line) -> bool`: an entry's figure reader (`Entry.shape`).
pub type ShapeFn = fn(&FnRef, &str) -> bool;

static SHAPES: LazyLock<HashMap<&'static str, ShapeFn>> = LazyLock::new(|| {
    let mut m: HashMap<&'static str, ShapeFn> = HashMap::new();
    for table in shape_tables() {
        for (k, f) in table.iter() {
            m.insert(k, *f);
        }
    }
    m
});

/// Every ported module's figure-shape readers.
fn shape_tables() -> Vec<&'static [(&'static str, ShapeFn)]> {
    vec![
        U_LINKED_SHAPES,
        crate::understanding::kinds::SHAPES,
        crate::knowledge::takes::SHAPES,
        crate::knowledge::family::SHAPES,
        crate::knowledge::pets::SHAPES,
        crate::knowledge::body::SHAPES,
        crate::knowledge::wording::SHAPES,
        crate::knowledge::pregnancy::SHAPES,
        crate::knowledge::cars::SHAPES,
        crate::knowledge::technology::SHAPES,
        crate::knowledge::language::SHAPES,
    ]
}

/// `_says_a_shape`'s `shape(line)` inside its `try`: false when it raises or has no port.
pub fn shape(f: &FnRef, line: &str) -> bool {
    match SHAPES.get(f.id.as_str()) {
        Some(s) => crate::runtime::guarded(|| s(f, line)).unwrap_or(false),
        None => {
            miss(&f.id);
            false
        }
    }
}

/// Whether a function has a port (for checks).
pub fn has_guard(id: &str) -> bool {
    GUARDS.contains_key(id)
}
pub fn has_shape(id: &str) -> bool {
    SHAPES.contains_key(id)
}
pub fn has_reply(id: &str) -> bool {
    REPLIES.contains_key(id)
}

/// Functions that were called but have no port, with how often.
pub fn misses() -> BTreeMap<String, usize> {
    MISSES.lock().unwrap().clone()
}

// ---------------------------------------------------------------------------------------------
// U (integration): domain functions that entries name as a guard or a shape and that the domain
// ports have (or are one-line lambdas over the domain's data) but no table registered. Linked here
// so `lookup.find` runs them; a domain table registering the same id later takes precedence only
// if it is listed after these (the maps keep the last insert), and both run the same port.

fn u_pets_stands_aside(_: &FnRef, u: &Understood) -> bool {
    crate::knowledge::pets::STANDS_ASIDE(u)
}
fn u_travel_not_a_wage(_: &FnRef, u: &Understood) -> bool {
    crate::knowledge::travel::_not_a_wage(u)
}

/// Guards linked by U (see above).
pub const U_LINKED_GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.pets:STANDS_ASIDE:3595", u_pets_stands_aside),
    ("bot.knowledge.travel:_not_a_wage:5929", u_travel_not_a_wage),
];

fn u_consumer_a_stretch_of_waiting(_: &FnRef, line: &str) -> bool {
    crate::knowledge::consumer::_a_stretch_of_waiting(line)
}
fn u_health_a_high_reading(_: &FnRef, line: &str) -> bool {
    crate::knowledge::health::_a_high_reading(line)
}
fn u_shopping_names_a_childs_age(_: &FnRef, line: &str) -> bool {
    crate::knowledge::shopping::_names_a_childs_age(line)
}
fn u_work_asks_what_to_do(_: &FnRef, line: &str) -> bool {
    crate::knowledge::work::_asks_what_to_do(line)
}
/// health.py L2291 `lambda words: bool(_SAID_COLD.search(words))`.
fn u_health_said_cold(_: &FnRef, line: &str) -> bool {
    crate::data::re("bot.knowledge.health._SAID_COLD").is_match(line)
}
/// health.py L2320 `lambda words: bool(_SAID_HOT.search(words))`.
fn u_health_said_hot(_: &FnRef, line: &str) -> bool {
    crate::data::re("bot.knowledge.health._SAID_HOT").is_match(line)
}
/// health.py L7745 `lambda words: bool(re.search(r"\d", (words or "")))`.
fn u_health_a_figure(_: &FnRef, line: &str) -> bool {
    crate::understanding::rx("\\d").is_match(line)
}
/// cooking.py L12363 `_alone(line, _only=tuple(_names))`.
fn u_cooking_alone(f: &FnRef, line: &str) -> bool {
    let only: Vec<String> = f.defaults.first().map(|v| v.strs()).unwrap_or_default();
    crate::knowledge::cooking_b::_names_nothing_but_the_dish(line, &only)
}

/// Shapes linked by U (see above).
pub const U_LINKED_SHAPES: &[(&str, ShapeFn)] = &[
    ("bot.knowledge.consumer:_a_stretch_of_waiting:1505", u_consumer_a_stretch_of_waiting),
    ("bot.knowledge.health:_a_high_reading:1611", u_health_a_high_reading),
    ("bot.knowledge.shopping:_names_a_childs_age:1649", u_shopping_names_a_childs_age),
    ("bot.knowledge.work:_asks_what_to_do:1707", u_work_asks_what_to_do),
    ("bot.knowledge.health:<lambda>:2291", u_health_said_cold),
    ("bot.knowledge.health:<lambda>:2320", u_health_said_hot),
    ("bot.knowledge.health:<lambda>:7745", u_health_a_figure),
    ("bot.knowledge.cooking:_alone:12363", u_cooking_alone),
];
