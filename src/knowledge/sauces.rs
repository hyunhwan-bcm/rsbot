//! Port of `bot/knowledge/sauces.py`: the sauces made at home (bread sauce, mint, horseradish,
//! peppercorn, hollandaise, garlic ...) - the recipe, or how long it keeps when that is the ask.

use crate::data::{self, FnRef, Value};
use crate::knowledge::built;
use crate::knowledge::entry::{Entry, KeyPart, Reply};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

const P: &str = "bot.knowledge.sauces.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{P}{name}"))
}
fn re(name: &str) -> &'static Re {
    g(name).as_re().unwrap_or_else(|| panic!("{P}{name} is not a regex"))
}

/// The oil emulsion, or the spoonful-in-yoghurt version when the line says yoghurt.
pub fn _garlic_sauce(_memory: &Memory, understood: &Understood) -> String {
    let line = text::lower(if understood.clean.is_empty() { &understood.text } else { &understood.clean });
    if line.contains("yoghurt") || line.contains("yogurt") {
        return g("_GARLIC_YOGHURT").str().to_string();
    }
    g("_GARLIC_EMULSION").str().to_string()
}

/// `_keeps(reply, keeping)`: the sauce's own keeping answer when the line asks how long it
/// lasts, else the recipe (`_keeps_answer`).
pub fn _keeps(reply: &Value, keeping: &str) -> FnRef {
    built::closure("bot.knowledge.sauces:_keeps.<locals>.answer:260", vec![("keeping", Value::from(keeping)), ("reply", reply.clone())])
}

/// The recipe: a fixed reply, or the sauce's own function.
fn the_recipe(reply: &Value, memory: &mut Memory, reading: &Understood) -> Option<String> {
    match reply {
        Value::Fn(f) => match f.name() {
            "bot.knowledge.sauces:_garlic_sauce" => Some(_garlic_sauce(memory, reading)),
            _ => Some(crate::knowledge::registry::reply(f, memory, reading)),
        },
        other => other.as_str().map(str::to_string),
    }
}

/// `_keeps(reply, keeping).<locals>.answer`.
pub fn _keeps_answer(reply: &Value, keeping: &str, memory: &mut Memory, reading: &Understood) -> Option<String> {
    let line = [&reading.norm_text, &reading.clean, &reading.text].into_iter().find(|s| !s.is_empty()).cloned().unwrap_or_default();
    let keeping = (!keeping.is_empty()).then(|| keeping.to_string());
    if re("_A_KEEPING_WORD").is_match(&line) {
        return keeping;
    }
    if re("_AN_UNTIL").is_match(&line) || re("_A_STEP_LENGTH").is_match(&line) {
        return the_recipe(reply, memory, reading);
    }
    if re("_A_COOKING_TIME").is_match(&line) {
        return None;
    }
    if keeping.is_some() && re("_KEEPING").is_match(&line) {
        return keeping;
    }
    if keeping.is_none() && re("_KEEPING").is_match(&line) {
        return None;
    }
    the_recipe(reply, memory, reading)
}

/// `_rows()`: one entry per sauce, the first one leaning a little harder.
pub fn _rows() -> Vec<Entry> {
    let made_in_the_pan = g("_MADE_IN_THE_PAN").strs();
    let keeps = g("_KEEPS");
    let mut entries = Vec::new();
    for (number, row) in g("_SAUCES").items().iter().enumerate() {
        let row = row.items();
        let (names, reply, avoid) = (row[0].strs(), &row[1], &row[2]);
        let gate = if row.len() > 3 { row[3].strs() } else { g("_IS_A_SAUCE").strs() };
        let not_a_pan: Vec<String> = if made_in_the_pan.contains(&names[0]) {
            Vec::new()
        } else {
            ["meat", "steak", "joint", "roasting tin", "gravy", "stock"].iter().map(|s| s.to_string()).collect()
        };
        entries.push(built::entry(
            &format!("sauce-{}", names[0].replace(' ', "-")),
            vec![built::part(&names), built::part(&g("_ASKING").strs())],
            Reply::Fn(_keeps(reply, keeps.get(&names[0]).map(Value::str).unwrap_or(""))),
            &[avoid.clone(), g("_NOT_A_RECIPE").clone(), built::words(&not_a_pan)],
            2 + if number == 0 { 1 } else { 0 },
            gate.into_iter().map(KeyPart::Word).collect(),
            None,
        ));
    }
    entries
}

fn keeps_reply(f: &FnRef, memory: &mut Memory, reading: &Understood) -> Option<String> {
    let reply = f.cell("reply").cloned().unwrap_or(Value::None);
    let keeping = f.cell("keeping").map(Value::str).unwrap_or("").to_string();
    _keeps_answer(&reply, &keeping, memory, reading)
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.sauces:_keeps.<locals>.answer:260", keeps_reply),
    ("bot.knowledge.sauces:_garlic_sauce:158", |_, m, u| Some(_garlic_sauce(m, u))),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_rebuild() {
        data::load();
        let exported = built::exported(&format!("{P}ENTRIES"));
        let ours = _rows();
        assert_eq!(ours.len(), exported.len());
        for (mine, theirs) in ours.iter().zip(exported.iter()) {
            assert_eq!(built::same_entry(mine, theirs), None);
        }
    }
}
