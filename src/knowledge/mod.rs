//! Port of PyBot's `bot/knowledge` package (`bot/knowledge/__init__.py` is this file). Each Python
//! file `bot/knowledge/<name>.py` is `<name>.rs` here.
//!
//! The knowledge base is data: every domain module's `ENTRIES` (exported in the bundle as
//! `bot.knowledge.<domain>.ENTRIES`), gathered here in Python's order by [`entries`].

pub mod a_plain_noise_word_on_a_machine;
pub mod bake_faults;
pub mod bikes;
pub mod cakes;
pub mod conditions;
pub mod cook_kit;
pub mod cooking_fats;
pub mod cooking_tables;
pub mod cooling;
pub mod crisis_replies;
pub mod desk_setup;
pub mod dish_verdicts;
pub mod drinks;
pub mod eithers;
pub mod everyday_lines;
pub mod food_left_out;
pub mod garden_tools;
pub mod how_it_works;
pub mod idioms;
pub mod ingredient_swaps;
pub mod job_times;
pub mod keeping_cooked;
pub mod kitchen;
pub mod kits_and_claims;
pub mod land;
pub mod machine_door;
pub mod meat_in_a_card;
pub mod microwave_methods;
pub mod money_words;
pub mod notable;
pub mod nursing;
pub mod parking;
pub mod pasta;
pub mod pets_health;
pub mod photography;
pub mod picks;
pub mod power_heat;
pub mod procedures;
pub mod quantity;
pub mod rice;
pub mod sauces;
pub mod school_day;
pub mod school_days;
pub mod seasoning;
pub mod spice_heat;
pub mod trades;
pub mod built;
pub mod units;
pub mod vegetables;
pub mod warm_food;
pub mod water_out;
pub mod what_damages_what;
pub mod entry;
pub mod lookup;
pub mod registry;

// D2 workstream
pub mod d2_shared;
pub mod differences;
pub mod icing;
pub mod cleaning;
pub mod consumer;
pub mod d2dup_g8;
pub mod lifeadmin;
pub mod plans;
pub mod coats;
pub mod containers_frost;
pub mod cooking_entries;
pub mod d2dup_g6;
pub mod health;
pub mod kit_lists;
pub mod prices;
pub mod whenbuy;
pub mod clothing;
pub mod d2dup_g7;
pub mod shopping;
pub mod work;
pub mod d2dup_entertainment;
pub mod entertainment;
pub mod d2dup_travel;
pub mod travel;
pub mod d2dup_money;
pub mod money;
pub mod animals;
pub mod appointments;
pub mod build_jobs;
pub mod child_work;
pub mod d2dup_g10;
pub mod fears;
pub mod friends;
pub mod fun;
pub mod habits;
pub mod history;
pub mod house_odds;
pub mod joints;
pub mod left_on;
pub mod measures;
pub mod music;
pub mod newborn;
pub mod occasions;
pub mod painting_faults;
pub mod pensions;
pub mod picking;
pub mod rubbish;
pub mod sleep;
pub mod still_ok;
pub mod swaps;
pub mod tenancy;
pub mod cooking;
pub mod d2dup_cooka;
pub mod cooking_b;
#[allow(non_snake_case)]
pub mod d2dup_cookB;
pub mod babyfood;
pub mod cooking_gaps;
pub mod costs;
pub mod d2dup_g9;
pub mod devices;
pub mod family_more;
pub mod fruit;
pub mod keeping;
pub mod opinions;
pub mod d2dup_maths;
pub mod maths;
pub mod d1_shared;
pub mod d1_dup_g1;
pub mod d1_dup_g2;
pub mod d1_dup_g3;
pub mod d1_dup_g4;
pub mod d1_dup_g5;
pub mod d1_dup_g6;
pub mod d1_dup_g7;
pub mod affording;
pub mod body;
pub mod body_energy;
pub mod business;
pub mod caring;
pub mod cars;
pub mod chilli;
pub mod comeback;
pub mod complaints;
pub mod cooking_more;
pub mod creative;
pub mod dates;
pub mod decorating;
pub mod dishes;
pub mod explains;
pub mod fabrics;
pub mod family;
pub mod fitness;
pub mod fitweek;
pub mod freshness;
pub mod garden;
pub mod geography;
pub mod home;
pub mod home_craft;
pub mod income_periods;
pub mod joint_time;
pub mod kept_animals;
pub mod kitchen_odds;
pub mod kitchen_prep;
pub mod language;
pub mod materials;
pub mod meals;
pub mod nature;
pub mod no_kit;
pub mod nutrition;
pub mod odours;
pub mod panic_talk;
pub mod pay;
pub mod pets;
pub mod pharmacy;
pub mod phone_use;
pub mod pregnancy;
pub mod routines;
pub mod schoolmaths;
pub mod science;
pub mod sleep_aids;
pub mod sport;
pub mod study;
pub mod takes;
pub mod technology;
pub mod travel_health;
pub mod vaccines;
pub mod wording;

use crate::data::{self, FnRef, Value};
use crate::knowledge::entry::Entry;
use crate::knowledge::registry::GuardFn;
use crate::understood::Understood;
use std::sync::LazyLock;

/// `domains()`: the knowledge domain names, in the order `pkgutil.iter_modules` found them
/// (the bundle keeps `_cache["domains"]`). Python maps name -> module; the port only needs names.
pub fn domains() -> &'static [String] {
    static DOMAINS: LazyLock<Vec<String>> = LazyLock::new(|| {
        let cache = data::get("bot.knowledge._cache");
        cache.get("domains").map(|d| d.pairs().iter().filter_map(|(k, _)| k.as_str().map(str::to_string)).collect()).unwrap_or_default()
    });
    &DOMAINS
}

/// `by_topic(found=None)`: {domain: the topic ids its entries answer}, as the bundle keeps it.
pub fn by_topic() -> &'static [(String, Vec<String>)] {
    static BY_TOPIC: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
        let cache = data::get("bot.knowledge._cache");
        cache.get("by_topic").map(|d| d.pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()).unwrap_or_default()
    });
    &BY_TOPIC
}

/// `topics(found=None)`: every topic id the knowledge base answers, sorted.
pub fn topics() -> Vec<String> {
    let mut all: Vec<String> = by_topic().iter().flat_map(|(_, t)| t.iter().cloned()).filter(|t| !t.is_empty()).collect();
    all.sort();
    all.dedup();
    all
}

/// `_with_one_more_test(guard, extra)`'s `guard_and_extra(understood)`: both tests pass.
pub fn guard_and_extra(f: &FnRef, understood: &Understood) -> bool {
    let run = |name: &str| match f.cell(name).and_then(Value::as_fn) {
        Some(g) => crate::knowledge::registry::guard(g, understood),
        None => panic!("TypeError: '{name}' is not callable"),
    };
    run("guard") && run("extra")
}

/// The guards defined in bot/knowledge/__init__.py.
pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge:_with_one_more_test.<locals>.guard_and_extra:142", guard_and_extra)];

/// `entries()`: every Entry in every domain, in a stable order, each carrying its domain name,
/// with the domain's `STANDS_ASIDE` and `GUARD_FOLDS` applied (the shared list in `entry`).
pub fn entries() -> &'static [Entry] {
    crate::knowledge::entry::entries()
}

/// `_with_one_more_test(guard, extra)`: a row's guard with one more condition folded in; with no
/// guard, the extra test alone. The fold is `guard_and_extra` over the two (see above).
pub fn _with_one_more_test(guard: &Value, extra: &Value) -> Value {
    if guard.is_none() {
        return extra.clone();
    }
    Value::Fn(std::sync::Arc::new(FnRef {
        id: "bot.knowledge:_with_one_more_test.<locals>.guard_and_extra:142".to_string(),
        cells: vec![("extra".to_string(), extra.clone()), ("guard".to_string(), guard.clone())],
        defaults: vec![],
    }))
}
