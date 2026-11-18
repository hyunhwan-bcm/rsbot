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
        // D2
        crate::knowledge::differences::GUARDS,
        crate::knowledge::icing::GUARDS,
    ]
}

/// Every ported module's reply table. A workstream adds its module's line here.
fn reply_tables() -> Vec<&'static [(&'static str, ReplyFn)]> {
    vec![
        // e.g. crate::knowledge::cooking::REPLIES,
        // D2
        crate::knowledge::differences::REPLIES,
        crate::knowledge::icing::REPLIES,
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

/// Whether a function has a port (for checks).
pub fn has_guard(id: &str) -> bool {
    GUARDS.contains_key(id)
}
pub fn has_reply(id: &str) -> bool {
    REPLIES.contains_key(id)
}

/// Functions that were called but have no port, with how often.
pub fn misses() -> BTreeMap<String, usize> {
    MISSES.lock().unwrap().clone()
}
