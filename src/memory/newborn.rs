//! Port of `bot/memory/newborn.py`: whether the last few turns are about a baby arriving, so a
//! present asked for is the newborn kind.

use super::mem8_py::{int, rx, strset};
use crate::data::Value;
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.newborn.";

strset!(BABY_WORDS);
rx!(A_BIRTH, JUST_BORN, A_YEAR, A_BIRTH_WEIGHT, NOT_THE_BABY);
int!(HOW_MANY_TURNS);
static LETTER_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// A str for `" ".join(...)`: Python raises TypeError on anything else, which nothing catches.
fn joinable(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        other => panic!("TypeError: sequence item: expected str instance, {other:?} found"),
    }
}

/// Python: bot/memory/newborn.py:42 `_the_lines(memory)`
///
/// The last few turns as words, newest first, without the line being answered.
pub fn _the_lines(memory: &Memory) -> Vec<String> {
    let mut lines: Vec<&Value> = Vec::new();
    if let Some(current) = memory.get("topic").filter(|c| c.is_dict()) {
        lines.push(current);
    }
    let records: &[Value] = match memory.get("topics") {
        Some(v) if v.truthy() => v.items(),
        _ => &[],
    };
    let from = records.len().saturating_sub(*HOW_MANY_TURNS as usize);
    for record in records[from..].iter().rev() {
        if record.is_dict() {
            lines.push(record);
        }
    }
    let mut said = Vec::new();
    for record in lines {
        let text = match record.get("text") {
            Some(v) if v.truthy() => joinable(v),
            _ => String::new(),
        };
        // The line as it was typed is read beside the words a topic keeps.
        let mut parts = vec![text];
        if let Some(Value::List(words)) = record.get("words") {
            parts.extend(words.iter().map(joinable));
        }
        said.push(parts.join(" "));
    }
    said
}

/// Python: bot/memory/newborn.py:62 `the_thread_has_a_new_baby(memory, understood=None)`
///
/// True when the last few turns are about a baby arriving.
pub fn the_thread_has_a_new_baby(memory: &Memory, understood: Option<&Understood>) -> bool {
    let line = match understood {
        Some(u) if !u.clean.is_empty() => u.clean.clone(),
        Some(u) => u.text.clone(),
        None => String::new(),
    };
    if !line.is_empty() && NOT_THE_BABY.is_match(&line) {
        return false;
    }
    let said = _the_lines(memory);
    if said.is_empty() {
        return false;
    }
    let together = said.join(" ");
    if NOT_THE_BABY.is_match(&together) {
        // a puppy and a baby in the same chat: the last thing named wins, and it is not a baby.
        return false;
    }
    let words: HashSet<String> = LETTER_WORDS.findall_str(&together.to_lowercase()).into_iter().collect();
    if JUST_BORN.is_match(&together) && !A_YEAR.is_match(&together) {
        return true;
    }
    if words.iter().any(|w| BABY_WORDS.contains(w)) && (A_BIRTH.is_match(&together) || A_BIRTH_WEIGHT.is_match(&together)) {
        return true;
    }
    // "a girl, 7lb 2oz" right after "had her baby": the weight is what says how old it is.
    A_BIRTH_WEIGHT.is_match(&together) && A_BIRTH.is_match(&together)
}

/// Python: bot/memory/newborn.py:88 `the_baby_is_new(memory, understood=None)`
///
/// True when the baby in the chat is a newborn, so a present is the newborn kind.
pub fn the_baby_is_new(memory: &Memory, understood: Option<&Understood>) -> bool {
    the_thread_has_a_new_baby(memory, understood)
}
