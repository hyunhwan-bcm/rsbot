//! Private ports of the helpers `bot/knowledge/maths.py` calls from modules other workstreams
//! own (the memory store's saving reader and topics, and the units table). Each is marked so the
//! integrator can swap in the owner's port; the data comes from the bundle by Python name.

use crate::data;
use crate::py::re::{Match, Re};
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

fn saving(name: &str) -> &'static Re {
    data::re(&format!("bot.memory.saving.{name}"))
}

// ---------------------------------------------------------------------------------------------
// bot/memory/topics.py

static INSIDE_A_SAFETY_THREAD: LazyLock<Vec<String>> =
    LazyLock::new(|| data::strs("bot.memory.topics.INSIDE_A_SAFETY_THREAD"));

// was DUP(bot.memory.topics::in_a_safety_thread): now the real port
pub use crate::memory::topics::in_a_safety_thread;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/units.py

// ---------------------------------------------------------------------------------------------
// bot/memory/saving.py

static FURTHER_SAID: LazyLock<&'static Re> = LazyLock::new(|| saving("FURTHER_SAID"));
static HELD_SAID: LazyLock<&'static Re> = LazyLock::new(|| saving("HELD_SAID"));
static POT_REPORTED: LazyLock<&'static Re> = LazyLock::new(|| saving("POT_REPORTED"));
static ALREADY_HELD: LazyLock<&'static Re> = LazyLock::new(|| saving("ALREADY_HELD"));
static ALREADY_AFTER: LazyLock<&'static Re> = LazyLock::new(|| saving("ALREADY_AFTER"));
static AIM_AT_A_SUM: LazyLock<&'static Re> = LazyLock::new(|| saving("AIM_AT_A_SUM"));
static BY_A_DATE: LazyLock<&'static Re> = LazyLock::new(|| saving("BY_A_DATE"));
static INTENT: LazyLock<&'static Re> = LazyLock::new(|| saving("INTENT"));
static AT_A_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| saving("AT_A_FIGURE"));
static NOT_A_PRICE: LazyLock<&'static Re> = LazyLock::new(|| saving("NOT_A_PRICE"));
static NOT_A_POT: LazyLock<&'static Re> = LazyLock::new(|| saving("NOT_A_POT"));
static A_RATE: LazyLock<&'static Re> = LazyLock::new(|| saving("A_RATE"));
static INCOME_FRAME: LazyLock<&'static Re> = LazyLock::new(|| saving("INCOME_FRAME"));
static OUTGOING_FRAME: LazyLock<&'static Re> = LazyLock::new(|| saving("OUTGOING_FRAME"));
static CLAUSE_EDGE: LazyLock<&'static Re> = LazyLock::new(|| saving("CLAUSE_EDGE"));
static A_RATE_PUT_RIGHT: LazyLock<&'static Re> = LazyLock::new(|| saving("A_RATE_PUT_RIGHT"));
static A_CAN_CARRY_IT: LazyLock<&'static Re> = LazyLock::new(|| saving("A_CAN_CARRY_IT"));
static A_SUM_A_MONTH: LazyLock<&'static Re> = LazyLock::new(|| saving("A_SUM_A_MONTH"));
static OTHER_FIGURE_WORDS: LazyLock<&'static Re> = LazyLock::new(|| saving("_OTHER_FIGURE_WORDS"));
static A_LONE_SUM: LazyLock<&'static Re> = LazyLock::new(|| saving("A_LONE_SUM"));
static A_CORRECTION_AND_A_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| saving("A_CORRECTION_AND_A_FIGURE"));
static NOT_THE_RATE: LazyLock<&'static Re> = LazyLock::new(|| saving("_NOT_THE_RATE"));
static A_NO_THEN_A_RATE: LazyLock<&'static Re> = LazyLock::new(|| saving("A_NO_THEN_A_RATE"));
static A_SPENT_ON: LazyLock<&'static Re> = LazyLock::new(|| saving("A_SPENT_ON"));
static A_MORE_LIKE: LazyLock<&'static Re> = LazyLock::new(|| saving("A_MORE_LIKE"));
static HAVE_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:have|has|got|got\s+\w+)\b", crate::py::re::I));

// DUP(bot.memory.saving::_pounds)
/// A figure written as people write it ("2", "2k", "2 grand") as a number, or 0.
fn pounds(figure: Option<&str>, unit: Option<&str>) -> i64 {
    let raw = figure.unwrap_or("").replace(',', "");
    let t = raw.trim();
    if t.is_empty() || !t.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return 0;
    }
    let Ok(mut value) = t.parse::<f64>() else { return 0 };
    if matches!(unit.unwrap_or("").to_lowercase().as_str(), "k" | "grand" | "thousand") {
        value *= 1000.0;
    }
    if value >= 20.0 { value.round_ties_even() as i64 } else { 0 }
}

/// `memory.saving.the_pot_and_the_rest(text)` (the real port), as (pot, rest).
pub fn the_pot_and_the_rest(line: &str) -> (i64, i64) {
    let both = crate::memory::saving::the_pot_and_the_rest(line);
    let items = both.items();
    (items[0].as_int().unwrap_or(0), items[1].as_int().unwrap_or(0))
}

// was DUP(bot.memory.saving::the_pot_said_alone): now the real port
pub use crate::memory::saving::the_pot_said_alone;

// was DUP(bot.memory.saving::the_pot_they_are_on): now the real port
pub use crate::memory::saving::the_pot_they_are_on;

// was DUP(bot.memory.saving::the_money_already_held): now the real port
pub use crate::memory::saving::the_money_already_held;

/// `line[:found.start()]` split at clause edges: the last clause before the figure.
fn is_an_income(line: &str, found: &Match) -> bool {
    let before = CLAUSE_EDGE.split(&line[..found.start()]);
    INCOME_FRAME.is_match(before.last().map(String::as_str).unwrap_or(""))
}

fn is_an_outgoing(line: &str, found: &Match) -> bool {
    let start = text::char_index(line, found.start()).saturating_sub(40);
    let end = text::char_index(line, found.end()) + 40;
    let window = text::slice(line, start as isize, Some(end as isize));
    OUTGOING_FRAME.is_match(&window)
}

/// True when the line puts money by at a rate, whatever else it says.
fn is_a_rate_line(line: &str) -> bool {
    A_RATE.finditer(line).iter().any(|found| !(is_an_income(line, found) || is_an_outgoing(line, found)))
}

/// One sum as a number, or 0 when it is not a rate anybody could put by.
fn a_sum_named(figure: &str) -> i64 {
    match figure.replace(',', "").trim().parse::<i64>() {
        Ok(v) if v >= 10 => v,
        _ => 0,
    }
}

// was DUP(bot.memory.saving::a_rate_put_right): now the real port
pub use crate::memory::saving::a_rate_put_right;
