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

// DUP(bot.memory.topics::in_a_safety_thread)
/// True while the bot's last real answer was an emergency and nothing has answered since.
pub fn in_a_safety_thread(memory: &Memory) -> bool {
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(3);
    for turn in turns[start..].iter().rev() {
        let module = turn.module.as_deref();
        if module == Some("safety") {
            return true;
        }
        match module {
            None | Some("greeting") => continue,
            Some(m) if INSIDE_A_SAFETY_THREAD.iter().any(|x| x == m) => continue,
            _ => return false,
        }
    }
    false
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/units.py

static SPECIFIC: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.units._SPECIFIC"));
static GRAMS_PER_CUP: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| {
    data::get("bot.knowledge.units.GRAMS_PER_CUP")
        .items()
        .iter()
        .map(|t| (t.items()[0].str().to_string(), t.items()[1].as_int().unwrap_or(0)))
        .collect()
});

// DUP(bot.knowledge.units::_food_named)
/// The food this line names, and what a cup of it weighs, the longest name winning.
pub fn units_food_named(clean: &str) -> Option<(String, i64)> {
    let mut best: Option<(String, i64)> = None;
    for (name, grams) in GRAMS_PER_CUP.iter() {
        let pattern = format!(r"\b{}\b", crate::py::re::escape(name));
        if crate::knowledge::maths::rx(&pattern, 0).is_match(clean)
            && best.as_ref().is_none_or(|b| name.chars().count() > b.0.chars().count())
        {
            best = Some((name.clone(), *grams));
        }
    }
    best
}

// DUP(bot.knowledge.units::answers_more_than_a_pair)
/// True when the reply says more than how many of one unit make another.
pub fn units_answers_more_than_a_pair(clean: &str) -> bool {
    if SPECIFIC.is_match(clean) {
        return true;
    }
    units_food_named(clean).is_some()
}

// NEEDS(bot.knowledge.units::convert)
/// `units.convert(clean, memory)`: the whole conversion table (3,700 lines of units.py, owned by
/// no D workstream). Until it is ported this answers nothing, so `solve` falls through to the
/// hand-written unit facts and the cards after it, as Python does for a line the table declines.
pub fn units_convert(_clean: &str, _memory: Option<&mut Memory>) -> Option<String> {
    None
}

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

// DUP(bot.memory.saving::the_pot_and_the_rest)
/// (money already held, money still to find) when the line names both, else (0, 0).
pub fn the_pot_and_the_rest(line: &str) -> (i64, i64) {
    if text::strip(line).is_empty() {
        return (0, 0);
    }
    let (Some(further), Some(held)) = (FURTHER_SAID.search(line), HELD_SAID.search(line)) else { return (0, 0) };
    if further.start() < held.start() {
        return (0, 0);
    }
    let pot = pounds(held.get(1), held.get(2));
    let rest = pounds(further.get(1), further.get(2));
    if pot == 0 || rest == 0 {
        return (0, 0);
    }
    (pot, rest)
}

// DUP(bot.memory.saving::the_pot_said_alone)
/// The money the line says is already saved, when the line is only that, else 0.
pub fn the_pot_said_alone(line: &str) -> i64 {
    if text::strip(line).is_empty() || FURTHER_SAID.is_match(line) {
        return 0;
    }
    if AIM_AT_A_SUM.is_match(line) || BY_A_DATE.is_match(line) {
        return 0;
    }
    if let Some(held) = HELD_SAID.search(line) {
        if !POT_REPORTED.is_match(line) {
            if INTENT.is_match(line) && !HAVE_WORD.is_match(line) {
                return 0;
            }
            return pounds(held.get(1), held.get(2));
        }
    }
    for pattern in [&*ALREADY_HELD, &*ALREADY_AFTER] {
        let Some(found) = pattern.search(line) else { continue };
        let pot = pounds(found.get(1), found.get(2));
        if pot >= 50 {
            return pot;
        }
    }
    0
}

// DUP(bot.memory.saving::the_pot_they_are_on)
/// The money the line says the person is on their way to, when that is what the line says.
pub fn the_pot_they_are_on(line: &str) -> i64 {
    if text::strip(line).is_empty() || NOT_A_PRICE.is_match(line) || A_RATE.is_match(line) {
        return 0;
    }
    let Some(found) = AT_A_FIGURE.search(line) else { return 0 };
    if NOT_A_POT.match_(&line[found.end()..]).is_some() {
        return 0;
    }
    pounds(found.get(1), found.get(2))
}

// DUP(bot.memory.saving::the_money_already_held)
/// The money the person said is already in the pot, from the last few lines, or 0.
pub fn the_money_already_held(memory: &Memory) -> i64 {
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(6);
    for turn in turns[start..].iter().rev() {
        let line = turn.user.as_str();
        let (pot, _) = the_pot_and_the_rest(line);
        if pot != 0 {
            return pot;
        }
        let pot = the_pot_said_alone(line);
        if pot != 0 {
            return pot;
        }
        let pot = the_pot_they_are_on(line);
        if pot != 0 {
            return pot;
        }
    }
    0
}

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

// DUP(bot.memory.saving::a_rate_put_right)
/// The monthly rate a line puts right, or 0.
pub fn a_rate_put_right(line: &str, before: &str) -> i64 {
    if !before.is_empty()
        && A_MORE_LIKE.is_match(line)
        && A_RATE_PUT_RIGHT.is_match(line)
        && is_a_rate_line(before)
        && !OTHER_FIGURE_WORDS.is_match(line)
    {
        let figures = A_LONE_SUM.findall_str(line);
        if figures.len() == 1 {
            return a_sum_named(&figures[0]);
        }
    }
    if let Some(after_a_no) = A_NO_THEN_A_RATE.search(line) {
        if !A_SPENT_ON.is_match(&line[after_a_no.end()..]) {
            return a_sum_named(after_a_no.str(1));
        }
    }
    if !A_RATE_PUT_RIGHT.is_match(line) || OTHER_FIGURE_WORDS.is_match(line) {
        return 0;
    }
    if let Some(with_the_month) = A_SUM_A_MONTH.search(line) {
        return a_sum_named(with_the_month.str(1));
    }
    let figures = A_LONE_SUM.findall_str(line);
    if figures.len() != 1 {
        return 0;
    }
    if !A_CAN_CARRY_IT.is_match(line)
        && (A_CORRECTION_AND_A_FIGURE.match_(text::strip(line)).is_none() || NOT_THE_RATE.is_match(line))
    {
        return 0;
    }
    a_sum_named(&figures[0])
}
