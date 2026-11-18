//! Port of PyBot's `bot/knowledge/no_kit.py`: what to use when a piece of kitchen kit is missing.

use crate::data::{self, FnRef};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

/// `STAND_IN` rows: (name, other names, reply).
static STAND_IN: LazyLock<Vec<(String, Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.no_kit.STAND_IN")
        .items()
        .iter()
        .map(|row| {
            let r = row.items();
            (r[0].str().to_string(), r[1].strs(), r[2].str().to_string())
        })
        .collect()
});
static _WITHOUT_A_MANNER: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.no_kit._WITHOUT_A_MANNER"));
static _A_TOOL_IS_STILL_MISSING: LazyLock<HashSet<String>> =
    LazyLock::new(|| data::set("bot.knowledge.no_kit._A_TOOL_IS_STILL_MISSING"));

/// The line's words, lower-cased.
pub fn _words(reading: &Understood) -> Vec<String> {
    reading.words.iter().map(|w| w.to_lowercase()).collect()
}

/// The name a `STAND_IN` row goes by.
pub fn _the_word_for(row: &(String, Vec<String>, String)) -> &str {
    &row.0
}

/// The stand-in card for the longest piece of kit the line names ("" when it names none).
pub fn _no_kit(_memory: &Memory, reading: &Understood) -> String {
    let text = format!(" {} ", reading.clean.to_lowercase());
    let words: HashSet<String> = _words(reading).into_iter().collect();
    let mut best: Option<(usize, &(String, Vec<String>, String))> = None;
    for row in STAND_IN.iter() {
        for name in std::iter::once(&row.0).chain(row.1.iter()) {
            let len = name.chars().count();
            let longer = best.is_none_or(|(b, _)| len > b);
            if name.contains(' ') {
                if text.contains(name.as_str()) && longer {
                    best = Some((len, row));
                }
            } else if words.contains(name) && longer {
                best = Some((len, row));
            }
        }
    }
    best.map(|(_, row)| row.2.clone()).unwrap_or_default()
}

/// Whether the line is short of a tool: "without stirring" is a way of cooking, not a missing
/// tool, unless the "-ing" word is having or using one.
pub fn _is_a_shortage(reading: &Understood) -> bool {
    let text = format!(" {} ", reading.clean.to_lowercase());
    let manners: Vec<&str> = _WITHOUT_A_MANNER.finditer(&text).iter().map(|m| m.str(1)).collect();
    if manners.is_empty() {
        return true;
    }
    manners.iter().any(|w| _A_TOOL_IS_STILL_MISSING.contains(*w))
}

fn g_is_a_shortage(_f: &FnRef, u: &Understood) -> bool {
    _is_a_shortage(u)
}
fn r_no_kit(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_no_kit(m, u))
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.no_kit:_is_a_shortage:318", g_is_a_shortage)];
pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.no_kit:_no_kit:275", r_no_kit)];
