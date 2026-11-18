//! Port of `bot/memory/will_it_work.py`: "will that work?" after a card - the yes, with the
//! card's own account of what the method achieves.

use super::mem8_py::{int, py_str, rx, strs, strset, PyVal};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understanding::pyuni;
use std::sync::LazyLock;

const M: &str = "bot.memory.will_it_work.";

rx!(WORK_CHECK, ACHIEVES, NOT_ACHIEVING, SENTENCE);
strset!(JOINS_ON);
strs!(YES_IN_FRONT, TRAILING_FILLER);
int!(THE_MOST_QUOTED);
static CLAUSE_CUT: LazyLock<Re> = LazyLock::new(|| Re::new(r"[,;:]\s+|\s+-\s+", 0));
static SPACES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+", 0));

/// Python: bot/memory/will_it_work.py:80 `asks_if_it_will_work(line)`
///
/// True when the line is nothing but a check that the advice works.
pub fn asks_if_it_will_work(line: &str) -> bool {
    let squashed = text::squash(line);
    let said = text::strip_chars(&squashed, " ?.!,");
    !said.is_empty() && WORK_CHECK.match_(said).is_some()
}

/// Python: bot/memory/will_it_work.py:86 `_the_clauses(sentence)`
///
/// The clauses of a sentence, in order.
pub fn _the_clauses(sentence: &str) -> Vec<String> {
    CLAUSE_CUT
        .split(sentence)
        .iter()
        .map(|part| text::strip_chars(part, " .").to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

/// Python: bot/memory/will_it_work.py:93 `_without_filler(clause)`
///
/// The clause without the words that only join it to the rest of the card ("... help too").
pub fn _without_filler(clause: &str) -> String {
    let mut said = text::strip_chars(clause, " .").to_string();
    for filler in TRAILING_FILLER.iter() {
        if said.to_lowercase().ends_with(&format!(" {filler}")) {
            let keep = text::len(&said) as isize - text::len(filler) as isize - 1;
            said = text::rstrip_chars(&text::slice(&said, 0, Some(keep)), " ,.").to_string();
        }
    }
    said
}

/// Python: bot/memory/will_it_work.py:102 `_what_it_achieves(card)`
///
/// The clause of the card that says what the method achieves, or "".
pub fn _what_it_achieves(card: &str) -> String {
    let sentences: Vec<String> =
        SENTENCE.split(card).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    for sentence in sentences.iter().rev() {
        if !ACHIEVES.is_match(sentence) || NOT_ACHIEVING.is_match(sentence) {
            continue;
        }
        for clause in _the_clauses(sentence) {
            if !ACHIEVES.is_match(&clause) || NOT_ACHIEVING.is_match(&clause) {
                continue;
            }
            let first = SPACES.split(&clause).swap_remove(0).to_lowercase();
            if JOINS_ON.contains(text::strip_chars(&first, ",.;:")) {
                continue;
            }
            let clause = _without_filler(&clause);
            let n = text::split_ref(&clause).len() as i64;
            if 4 <= n && n <= *THE_MOST_QUOTED {
                return clause;
            }
        }
    }
    String::new()
}

/// Python: bot/memory/will_it_work.py:124 `reply_for(text, memory)`
///
/// The yes with the card's own account of why, or None.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    if !asks_if_it_will_work(text) {
        return None;
    }
    let card = super::measure::_the_card_said_last(memory).val();
    if !card.truthy() {
        return None;
    }
    let mut said = _what_it_achieves(&py_str(&card));
    if said.is_empty() {
        return None;
    }
    // The card's verdict lines already have an owner (`thread.checking_the_verdict`).
    let low = text::lstrip(&said).to_lowercase();
    if low.starts_with("yes") || low.starts_with("no") {
        return None;
    }
    let one = text::slice(&said, 0, Some(1));
    let two = text::slice(&said, 1, Some(2));
    let three = text::slice(&said, 2, Some(3));
    if pyuni::is_upper(&one) && (pyuni::is_lower(&two) || (two == " " && pyuni::is_lower(&three))) {
        said = super::mem8_py::lower_first(&said);
    }
    let said = text::rstrip_chars(&said, " .");
    let at = text.chars().map(|c| c as usize).sum::<usize>() % YES_IN_FRONT.len();
    Some(format!("{} \u{2014} {}.", YES_IN_FRONT[at], said))
}
