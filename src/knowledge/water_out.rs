//! Port of `bot/knowledge/water_out.py`: getting standing water out of a thing - a caravan, a
//! tank, a pond, a paddling pool - and the general way when the line names none.

use crate::data;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

/// `WATER_OUT_OF`: (the words for a thing, the way out of it), in the order things are told apart.
static WATER_OUT_OF: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.water_out.WATER_OUT_OF").items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect()
});

fn word_re(word: &str) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<String, &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(r) = cache.get(word) {
        return r;
    }
    let r: &'static Re = Box::leak(Box::new(Re::new(&format!(r"\b{}\b", pyre::escape(word)), 0)));
    cache.insert(word.to_string(), r);
    r
}

/// The way out of the thing the line names, or the general way when it names none.
pub fn _water_out_of_a_thing(_memory: &Memory, reading: &Understood) -> String {
    let mut line = text::lower(if reading.clean.is_empty() { &reading.text } else { &reading.clean });
    if line.is_empty() {
        line = text::lower(&reading.words.join(" "));
    }
    for (words, reply) in WATER_OUT_OF.iter() {
        if words.iter().any(|w| word_re(w).is_match(&line)) {
            return reply.clone();
        }
    }
    "A bucket and gravity: bail most of it into a bucket, then syphon the rest with a garden hose kept full of water, with the outlet end lower than the water. Never tip a lot of water down a floor drain or a shower tray, which will not take it fast enough to keep up.".into()
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.water_out:_water_out_of_a_thing:114", |_, m, u| Some(_water_out_of_a_thing(m, u)))];
