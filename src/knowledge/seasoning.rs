//! Port of `bot/knowledge/seasoning.py`: how much of a spice goes in a dish.

use crate::data::{self, Value};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

fn g(name: &str) -> &'static Value {
    data::get(&format!("bot.knowledge.seasoning.{name}"))
}

/// `_SPICE_AMOUNTS`: (the patterns a spice is written as, its figure).
static _SPICE_AMOUNTS: LazyLock<Vec<(Vec<String>, String)>> =
    LazyLock::new(|| g("_SPICE_AMOUNTS").items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect());
static NOT_PLAIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9 ]+", 0));

/// `re.search(r"\b" + variant + r"\b", ...)`: the variant is a pattern of its own, not escaped.
fn variant_re(variant: &str) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<String, &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(r) = cache.get(variant) {
        return r;
    }
    let r: &'static Re = Box::leak(Box::new(Re::new(&format!(r"\b{variant}\b"), 0)));
    cache.insert(variant.to_string(), r);
    r
}

/// The figure for one spice, when the line asks how much of it goes in a dish; "" otherwise.
/// The spice named first wins, and the longer variant where two start at the same place.
pub fn _spice_amount(_memory: &Memory, understood: &Understood) -> String {
    let said = format!(" {} ", NOT_PLAIN.sub(" ", &text::lower(&understood.clean)));
    if g("_SPICE_NOT_A_DOSE").strs().iter().any(|w| said.contains(&format!(" {w} "))) {
        return String::new();
    }
    if said.contains("mulled wine") {
        return g("_MULLED_WINE").str().to_string();
    }
    if said.contains("pumpkin spice") && ["latte", "coffee", "cappuccino", "mug", "cup"].iter().any(|d| said.contains(d)) {
        return g("_PUMPKIN_LATTE").str().to_string();
    }
    let mut first: Option<(usize, usize, &String)> = None;
    for (variants, reply) in _SPICE_AMOUNTS.iter() {
        for variant in variants {
            let Some(found) = variant_re(variant).search(&said) else { continue };
            let at = text::char_index(&said, found.start());
            let length = text::len(variant);
            if first.is_none_or(|(f_at, f_len, _)| at < f_at || (at == f_at && length > f_len)) {
                first = Some((at, length, reply));
            }
        }
    }
    first.map(|(_, _, reply)| reply.clone()).unwrap_or_default()
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.seasoning:_spice_amount:214", |_, m, u| Some(_spice_amount(m, u)))];
