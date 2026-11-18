//! Port of `bot/modules/a_letter_about_money.py`: a letter, a notice or a bill about money that has arrived (or has not): what to find on it first.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

macro_rules! rx {
    ($($name:ident),*) => {$(
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.a_letter_about_money.", stringify!($name))));
    )*};
}
macro_rules! pool {
    ($($name:ident),*) => {$(
        static $name: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(concat!("bot.modules.a_letter_about_money.", stringify!($name))));
    )*};
}
rx!(_ARRIVED, _NOT_YET, _OUTGOING, _MONEY, _AMOUNT, _DATE);
pool!(LETTER_LINES, BILL_LINES, NOT_YET_LINES);
static _SPELLINGS: LazyLock<&'static Value> = LazyLock::new(|| data::get("bot.modules.a_letter_about_money._SPELLINGS"));
static A_LETTER_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:letter|notice|bill)\b", pyre::I));

/// Python: bot/modules/a_letter_about_money.py `_ask_for_what_is_missing(line, clean, thing)`
///
/// Point the question at the detail the line does not already carry.
pub fn _ask_for_what_is_missing(line: &str, clean: &str, thing: &str) -> String {
    let has_amount = _AMOUNT.is_match(clean);
    let has_date = _DATE.is_match(clean);
    if !has_amount {
        if !has_date {
            return line.to_string();
        }
        return line.replace("Tell me the amount and the dates on it and I'll help you check it.", "Tell me the amount on it and I'll help you check it.");
    }
    if !has_date {
        return line
            .replace("Tell me the amount and the dates on it and I'll help you check it.", "Tell me the dates it covers and I'll help you check it.")
            .replace("What is it for, and how much?", "What is it for, and what dates does it cover?");
    }
    line.replace("Tell me the amount and the dates on it and I'll help you check it.", "Tell me the reference and I'll help you check it.")
        .replace("What does it say the amount is?", "What is the reference on it?")
        .replace("What is it for, and how much?", "What is it for, and what is the reference?")
}

/// Python: bot/modules/a_letter_about_money.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let mut clean = fold(text);
    if clean.is_empty() {
        return None;
    }
    for spelling in _SPELLINGS.items() {
        clean = spelling.items()[0].as_re().unwrap().sub(spelling.items()[1].str(), &clean);
    }
    let Some(m) = _ARRIVED.search(&clean) else {
        if _NOT_YET.is_match(&clean) && _MONEY.is_match(&clean) && A_LETTER_WORD.is_match(&clean) && !_OUTGOING.is_match(&clean) {
            return Some(_pick(&NOT_YET_LINES, memory));
        }
        return None;
    };
    if _OUTGOING.is_match(&clean[..m.start()]) {
        return None;
    }
    let thing = ["thing", "thing2", "thing3", "thing4", "thing5"].iter().map(|name| m.name_str(name)).find(|thing| !thing.is_empty()).unwrap_or("").to_lowercase();
    if thing.is_empty() {
        return None;
    }
    if !_MONEY.is_match(&clean) {
        return None;
    }
    if _NOT_YET.is_match(&clean) {
        return Some(_pick(&NOT_YET_LINES, memory));
    }
    if thing == "bill" {
        return Some(_ask_for_what_is_missing(&_pick(&BILL_LINES, memory), &clean, &thing));
    }
    Some(_ask_for_what_is_missing(&_pick(&LETTER_LINES, memory), &clean, &thing))
}

/// Python: bot/modules/a_letter_about_money.py `_pick(lines, memory)`
///
/// Two wordings, so a second letter ask does not repeat the first reply word for word.
pub fn _pick(lines: &[String], memory: &Memory) -> String {
    let last = last_bot(memory);
    lines.iter().find(|line| **line != last).unwrap_or(&lines[0]).clone()
}
