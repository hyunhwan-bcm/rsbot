//! Port of `bot/knowledge/history.py`: history rows (the cold war, votes for women, the great
//! wall) and two tables, PAST (wars, people, places) and MADE (who invented what), whose rows a
//! line names by any of their names. `names_a_row`, `names_a_row_or_a_question_after_one` and
//! `a_short_question_about_the_last_one` are guard factories: their guards read the captured table.

use crate::data::{FnRef, Value};
use crate::knowledge::d2dup_g10::globals;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};

globals!("bot.knowledge.history",
    string { COLD_WAR, COLD_WAR_WINNER },
    strs { ASKS_WHO_WON, _US, NOT_THE_THING },
    set { _WOMEN, _OTHER_SEX, _OTHER_COUNTRY, _OTHER_AGE },
    re { _HISTORY_OF_THE_VOTE, _WALK_THE_WALL, FOLLOW_ON_ASK, PAST_DEAD, PAST_WON, PAST_BUILT, PAST_LONG },
    value { PAST, MADE });

static NOT_A_LETTER: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z]+", 0));

/// `\b<word>\b`, as `_named_row` searches every name.
fn word_re(word: &str) -> Re {
    Re::new(&format!(r"\b{}\b", re::escape(word)), 0)
}

static NOT_THE_THING_RE: LazyLock<Vec<Re>> = LazyLock::new(|| NOT_THE_THING.iter().map(|w| word_re(w)).collect());

/// One row of a table, its names and "off" words compiled.
struct Row {
    row: Value,
    names: Vec<(usize, Re)>,
    off: Vec<Re>,
}

/// A table's rows, compiled once per table object.
static TABLES: LazyLock<Mutex<HashMap<usize, Arc<Vec<Row>>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn compiled(table: &Value) -> Arc<Vec<Row>> {
    let Value::Dict(inner) = table else { panic!("a history table is a dict") };
    let key = Arc::as_ptr(inner) as *const () as usize;
    let mut tables = TABLES.lock().unwrap();
    tables
        .entry(key)
        .or_insert_with(|| {
            Arc::new(
                table
                    .pairs()
                    .iter()
                    .map(|(_, row)| Row {
                        row: row.clone(),
                        names: row.get("names").map(|n| n.strs()).unwrap_or_default().iter().map(|n| (n.chars().count(), word_re(n))).collect(),
                        off: row.get("off").map(|o| o.strs()).unwrap_or_default().iter().map(|w| word_re(w)).collect(),
                    })
                    .collect(),
            )
        })
        .clone()
}

/// The Cold War, with who won it when the line asks.
pub fn _cold_war(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = format!(" {} ", NOT_A_LETTER.sub(" ", &understood.text.to_lowercase()));
    let won = ASKS_WHO_WON.iter().any(|w| said.contains(&format!(" {w} ")));
    Some(format!("{}{}", *COLD_WAR, if won { COLD_WAR_WINNER.as_str() } else { "" }))
}

/// True when the line is about women voting in Britain, the only vote this row has.
pub fn _not_the_other_sex_or_country(understood: &Understood) -> bool {
    let text = if understood.text.is_empty() { &understood.clean } else { &understood.text }.to_lowercase();
    if text.is_empty() {
        return false;
    }
    let spaced = text.replace(['?', ',', '.'], " ");
    let words: HashSet<&str> = text::split_ref(&spaced).into_iter().collect();
    if !words.iter().any(|w| _WOMEN.contains(*w)) {
        return false;
    }
    if words.iter().any(|w| _OTHER_SEX.contains(*w) || _OTHER_COUNTRY.contains(*w)) {
        return false;
    }
    if words.iter().any(|w| _OTHER_AGE.contains(*w)) {
        return false;
    }
    let padded = format!(" {text} ");
    !_US.iter().any(|p| padded.contains(p.as_str()))
}

/// A guard: the history of the vote answers a line about the change, not the age today.
pub fn _about_the_change(reading: &Understood) -> bool {
    let text = if reading.clean.is_empty() { &reading.text } else { &reading.clean }.to_lowercase();
    _HISTORY_OF_THE_VOTE.is_match(&text)
}

/// The walking time when the line asks to walk it, and the length of the wall otherwise.
pub fn _how_long_to_walk_the_wall(understood: &Understood) -> String {
    let text = format!(" {} ", if understood.clean.is_empty() { &understood.text } else { &understood.clean });
    if !_WALK_THE_WALL.is_match(&text) {
        return "About 21,196 km of wall in all, counting every branch and trench: the restored stretch most visitors walk at Badaling near Beijing is a few kilometres of that, and nobody walks the whole thing.".into();
    }
    "Two to three hours on the restored stretch at Badaling, walking it end to end at an easy pace, and most people take the cable car up at Mutianyu and walk a section in half a day. That is the wall as a visit. The whole 21,196 km is not a walk anyone does: it is broken by valleys, roads and reservoirs, and the parts in the west are crumbling earth rather than stone.".into()
}

/// The table row a line names, longest name first, or None if it names none of them.
pub fn _named_row(table: &Value, text: &str) -> Option<Value> {
    if NOT_THE_THING_RE.iter().any(|r| r.is_match(text)) {
        return None;
    }
    let mut best: Option<&Value> = None;
    let mut best_len = 0;
    let rows = compiled(table);
    for row in rows.iter() {
        if row.off.iter().any(|r| r.is_match(text)) {
            continue; // "waterloo station" is not the battle
        }
        for (len, name) in &row.names {
            if *len > best_len && name.is_match(text) {
                best = Some(&row.row);
                best_len = *len;
            }
        }
    }
    best.cloned()
}

/// The line as read, plus the words before the reader repaired them.
pub fn _said(reading: &Understood) -> String {
    [&reading.plain, &reading.clean, &reading.text]
        .iter()
        .filter(|p| !p.is_empty())
        .map(|p| p.as_str())
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn table_cell(f: &FnRef) -> &Value {
    f.cell("table").expect("history guard captures its table")
}

/// `names_a_row.<locals>.guard`: the line names a row of the captured table.
fn names_a_row_guard(f: &FnRef, reading: &Understood) -> bool {
    _named_row(table_cell(f), &_said(reading)).is_some()
}

/// Whether the line is a short question about somebody the last exchange already named.
pub fn _asks_after_a_name(text: &str) -> bool {
    FOLLOW_ON_ASK.is_match(text)
}

/// `names_a_row_or_a_question_after_one.<locals>.guard`.
fn names_a_row_or_a_question_after_one_guard(f: &FnRef, reading: &Understood) -> bool {
    let text = _said(reading);
    if _named_row(table_cell(f), &text).is_some() {
        return true;
    }
    _asks_after_a_name(&text)
}

/// `a_short_question_about_the_last_one.<locals>.guard`: a short question that names nothing.
fn a_short_question_about_the_last_one_guard(f: &FnRef, reading: &Understood) -> bool {
    let text = if reading.clean.is_empty() { &reading.text } else { &reading.clean }.to_lowercase();
    if _named_row(table_cell(f), &_said(reading)).is_some() {
        return false;
    }
    if text::split_ref(&text).len() > 6 {
        return false;
    }
    _asks_after_a_name(&text)
}

/// The table row the last exchange named, when this line asks about it and names nobody.
pub fn _the_row_behind(memory: &Memory, text: &str) -> Option<Value> {
    if _named_row(&PAST, text).is_some() {
        return None;
    }
    if !_asks_after_a_name(text) {
        return None;
    }
    let last = memory.turns.last()?;
    let before = format!("{} {}", last.user, last.bot).to_lowercase();
    _named_row(&PAST, &before)
}

/// The table's answer to the question the line asks about the thing it names.
pub fn the_past_named(memory: &mut Memory, reading: &Understood) -> Option<String> {
    let text = _said(reading);
    let row = _named_row(&PAST, &text).or_else(|| _the_row_behind(memory, &text))?;
    let field = |k: &str| row.get(k).filter(|v| v.truthy()).map(|v| v.str().to_string());
    for (key, rx) in [("dead", &*PAST_DEAD), ("won", &*PAST_WON), ("built", &*PAST_BUILT), ("long", &*PAST_LONG)] {
        if let Some(said) = field(key) {
            if rx.is_match(&text) {
                return Some(said);
            }
        }
    }
    Some(row.get("tell").expect("KeyError: tell").str().to_string())
}

/// Who made the thing the line names, or None so another entry can answer.
pub fn the_inventor(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let text = if reading.plain.is_empty() { &reading.text } else { &reading.plain };
    let row = _named_row(&MADE, &text.to_lowercase())?;
    Some(row.get("tell").expect("KeyError: tell").str().to_string())
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.history:_not_the_other_sex_or_country:59", |_: &FnRef, u: &Understood| {
        _not_the_other_sex_or_country(u)
    }),
    ("bot.knowledge.history:_about_the_change:88", |_: &FnRef, u: &Understood| _about_the_change(u)),
    ("bot.knowledge.history:names_a_row.<locals>.guard:1890", names_a_row_guard),
    ("bot.knowledge.history:names_a_row_or_a_question_after_one.<locals>.guard:1920", names_a_row_or_a_question_after_one_guard),
    ("bot.knowledge.history:a_short_question_about_the_last_one.<locals>.guard:1936", a_short_question_about_the_last_one_guard),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.history:_cold_war:26", |_: &FnRef, m: &mut Memory, u: &Understood| _cold_war(m, u)),
    ("bot.knowledge.history:<lambda>:985", |_: &FnRef, _: &mut Memory, u: &Understood| Some(_how_long_to_walk_the_wall(u))),
    ("bot.knowledge.history:the_past_named:1965", |_: &FnRef, m: &mut Memory, u: &Understood| the_past_named(m, u)),
    ("bot.knowledge.history:the_inventor:1984", |_: &FnRef, m: &mut Memory, u: &Understood| the_inventor(m, u)),
];


// Stubs appended by harness/integrate_skeleton.py: the Python functions this file does not
// define yet, with signatures from the generated contract. Port them in place (move a
// function up into the file when you port it).
pub use self::skeleton_stubs::*;
#[allow(unused_imports, unused_variables, non_snake_case, dead_code, clippy::all)]
mod skeleton_stubs {
    use super::*;
    use crate::data::{self, Value};
    use crate::knowledge::entry::Entry;
    use crate::runtime::Candidate;
    use crate::state::{Memory, Turn};
    use crate::understood::Understood;

    /// Python: bot/knowledge/history.py:1882 `names_a_row(table)` (never called on the trace corpus)
    pub fn names_a_row(table: &Value) -> Value {
        crate::unported!("bot.knowledge.history:names_a_row")
    }

    /// Python: bot/knowledge/history.py:1913 `names_a_row_or_a_question_after_one(table)` (never called on the trace corpus)
    pub fn names_a_row_or_a_question_after_one(table: &Value) -> Value {
        crate::unported!("bot.knowledge.history:names_a_row_or_a_question_after_one")
    }

    /// Python: bot/knowledge/history.py:1928 `a_short_question_about_the_last_one(table)` (never called on the trace corpus)
    pub fn a_short_question_about_the_last_one(table: &Value) -> Value {
        crate::unported!("bot.knowledge.history:a_short_question_about_the_last_one")
    }

}

