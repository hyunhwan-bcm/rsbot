//! Port of PyBot's `bot/knowledge/dates.py`: when a famous thing happened.

use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

/// One `_DATES` row, its words compiled as `\bword\b`.
pub struct DateRow {
    pub names: Vec<(usize, Re)>,
    pub off: Vec<Re>,
    pub off_re: Option<&'static Re>,
    pub tell: String,
    pub long: Option<String>,
}

fn word_re(word: &str) -> Re {
    Re::new(&format!(r"\b{}\b", sh::escape(word)), 0)
}

static _DATES: LazyLock<Vec<DateRow>> = LazyLock::new(|| {
    data::get("bot.knowledge.dates._DATES")
        .pairs()
        .iter()
        .map(|(_, row)| DateRow {
            names: row.get("names").map(Value::strs).unwrap_or_default().iter().map(|n| (n.chars().count(), word_re(n))).collect(),
            off: row.get("off").map(Value::strs).unwrap_or_default().iter().map(|w| word_re(w)).collect(),
            off_re: row.get("off_re").and_then(Value::as_re),
            tell: row.get("tell").map(|v| v.str().to_string()).unwrap_or_default(),
            long: row.get("long").filter(|v| v.truthy()).map(|v| v.str().to_string()),
        })
        .collect()
});
static _LONG: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.dates._LONG"));

/// The line as plain, clean and text joined, lower-cased.
pub fn _said(reading: &Understood) -> String {
    [&reading.plain, &reading.clean, &reading.text].iter().filter(|p| !p.is_empty()).map(|p| p.as_str()).collect::<Vec<_>>().join(" ").to_lowercase()
}

/// The row whose longest name the line holds (rows whose off-words or off-shape the line has are
/// skipped).
pub fn _the_row(text: &str) -> Option<&'static DateRow> {
    let mut best: Option<&'static DateRow> = None;
    let mut best_len = 0;
    for row in _DATES.iter() {
        if row.off.iter().any(|r| r.is_match(text)) {
            continue;
        }
        if row.off_re.is_some_and(|r| r.is_match(text)) {
            continue;
        }
        for (len, name) in &row.names {
            if *len > best_len && name.is_match(text) {
                best = Some(row);
                best_len = *len;
            }
        }
    }
    best
}

/// `_named(text)`: the row the text names.
pub fn _named(text: &str) -> Option<&'static DateRow> {
    _the_row(text)
}

/// The date card for the thing the line names (its "how long" card on a length question).
pub fn the_date(_memory: &Memory, reading: &Understood) -> Option<String> {
    let text = _said(reading);
    let row = _the_row(&text)?;
    if let Some(long) = &row.long {
        if _LONG.is_match(&text) {
            return Some(long.clone());
        }
    }
    Some(row.tell.clone())
}

/// Whether the line names one of the dated things.
pub fn names_a_row(reading: &Understood) -> bool {
    _the_row(&_said(reading)).is_some()
}

fn g_names_a_row(_f: &FnRef, u: &Understood) -> bool {
    names_a_row(u)
}
fn r_the_date(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    the_date(m, u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.dates:names_a_row:154", g_names_a_row)];
pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.dates:the_date:143", r_the_date)];
