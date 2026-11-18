//! Port of `bot/memory/pile.py`: how much of a thing the person has, said as the whole line
//! ("ive got too many", "weve got plenty"). A pile is not a day: what a friend does with a pile
//! is cut it down to a shortlist. Module data comes from the exported bundle.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::py::re::{Match, Re};
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

fn g(name: &str) -> &'static Value {
    data::get(&format!("bot.memory.pile.{name}"))
}

static AN_AMOUNT_THEY_HAVE: LazyLock<&'static Re> = LazyLock::new(|| g("AN_AMOUNT_THEY_HAVE").as_re().unwrap());
static NOT_A_PILE: LazyLock<&'static Re> = LazyLock::new(|| g("NOT_A_PILE").as_re().unwrap());
static A_DAY: LazyLock<&'static Re> = LazyLock::new(|| g("A_DAY").as_re().unwrap());
static A_BACK_REFERENCE: LazyLock<HashSet<String>> = LazyLock::new(|| g("A_BACK_REFERENCE").str_set());
static SHORT: LazyLock<usize> = LazyLock::new(|| g("SHORT").as_int().unwrap_or(8) as usize);
static A_PILE_TOO_BIG: LazyLock<Vec<String>> = LazyLock::new(|| g("A_PILE_TOO_BIG").strs());

/// Python: bot/memory/pile.py:94 `_the_thing(found)`
pub fn _the_thing(found: &Match) -> String {
    pt::lower(&pt::squash(found.name_str("thing")))
}

/// Python: bot/memory/pile.py:98 `reads_the_pile(text, memory=None)`
///
/// The reading, for a caller that wants to know without a reply: True for a pile line.
pub fn reads_the_pile(text: &str, _memory: &Memory) -> bool {
    let line = pt::squash(text);
    if line.is_empty() || line.contains('?') || pt::split_ref(&line).len() > *SHORT {
        return false;
    }
    let Some(found) = AN_AMOUNT_THEY_HAVE.match_(&line) else {
        return false;
    };
    if A_DAY.is_match(&line) {
        return false;
    }
    let thing = _the_thing(&found);
    if !A_BACK_REFERENCE.contains(&thing) {
        return false;
    }
    if !thing.is_empty() && NOT_A_PILE.is_match(&thing) {
        return false;
    }
    true
}

/// Python: bot/memory/pile.py:116 `reply_for(text, memory=None)`
///
/// A reply to a pile, or None for every other line. (A memory with nothing in it is Python's
/// `memory=None`, which `pick` reads as `{}`.)
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    if !reads_the_pile(text, memory) {
        return None;
    }
    let memory_value = Value::from_plain(&serde_json::Value::Object(memory.to_plain()));
    Some(crate::compose::pick(&A_PILE_TOO_BIG, &memory_value, text))
}
