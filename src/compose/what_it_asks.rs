//! Port of `bot/compose/what_it_asks.py`: a last look at the line for a shape the reading misses
//! ("what should i spend it on" is a pick, "how much do they charge" an amount).

#![allow(non_upper_case_globals)]

use crate::py::re::{Match, Re};
use crate::py::text;
use crate::understanding::{rx, t};
use crate::understood::Understood;
use std::sync::LazyLock;

const M: &str = "bot.compose.what_it_asks.";

static ALTERNATIVES: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ALTERNATIVES"));
static AMOUNT: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "AMOUNT"));
static MONEY_VERB_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MONEY_VERB_ASK"));
static NOT_A_PRESENT_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_A_PRESENT_ASK"));
static OPTIONS: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "OPTIONS"));
static PICK: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "PICK"));
static PRESENT_LOOKING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PRESENT_LOOKING"));
static SPEND_IT_ON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SPEND_IT_ON"));

/// `_names_alternatives(line, match)`: the clause around the money verb names alternatives.
fn _names_alternatives(line: &str, m: &Match) -> bool {
    let s = text::char_index(line, m.start()) as isize;
    let e = text::char_index(line, m.end()) as isize;
    let lo = (s - 80).max(0);
    let window = text::slice(line, lo, Some(e + 80));
    let seat = (s - lo) as usize;
    let mut spent = 0usize;
    for part in rx("[,;:?.!]").split(&window) {
        let n = text::len(&part);
        if spent <= seat && seat <= spent + n {
            return ALTERNATIVES.is_match(&part);
        }
        spent += n + 1;
    }
    ALTERNATIVES.is_match(&window)
}

/// `_the_line_once(understood)`: the line (plain, text or clean) lower-cased.
fn _the_line_once(u: &Understood) -> String {
    for value in [&u.plain, &u.text, &u.clean] {
        if !text::strip(value).is_empty() {
            return text::squash(&value.to_lowercase().replace('\u{2019}', "'"));
        }
    }
    String::new()
}

/// `the_shape_it_asks(understood, want)`.
pub fn the_shape_it_asks(u: &Understood, want: Option<String>) -> Option<String> {
    let line = _the_line_once(u);
    if line.is_empty() {
        return want;
    }
    if SPEND_IT_ON.is_match(&line) {
        return Some(PICK.to_string());
    }
    if let Some(money) = MONEY_VERB_ASK.search(&line) {
        if !_names_alternatives(&line, &money) {
            return Some(AMOUNT.to_string());
        }
    }
    if want.is_none() && PRESENT_LOOKING.is_match(&line) && !NOT_A_PRESENT_ASK.is_match(&line) {
        return Some(OPTIONS.to_string());
    }
    want
}

#[allow(dead_code)]
fn _unused(_: &Re) {}
