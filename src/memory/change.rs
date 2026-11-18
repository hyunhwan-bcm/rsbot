//! Port of `bot/memory/change.py`: a line that reports a change to a fact the store keeps is
//! news, not a question about it.
//!
//! "ive changed my diet" was answered "You haven't told me about your diet." What is missing
//! after it is what the diet is now, so the honest reply asks for that. The subject is the
//! caller's: it passes the word the fact is kept under and gets back how the line stands to it.
#![allow(non_snake_case, clippy::all)]

use crate::memory::mem7::{self, res, sets, texts, values};
use crate::py::re::{self, Re};
use crate::py::text as pt;

res!("change"; CHANGED, SUBJECT_CHANGED, WANTS, HOW_TO, WHEN_TAIL, NOT_WANTING);
sets!("change"; FILLER);
values!("change"; ASKS, WISHES);
texts!("change"; KEPT, DROPPING, CHANGED_ASK, WISHED);
mem7::inline_re! {
    A_WORD = r"[a-z']+", 0;
}

/// `r"\b" + r"\s+".join(re.escape(part) for part in words.split()) + r"\b"`, case-blind.
fn the_subject_pattern(words: &str) -> Re {
    let parts: Vec<String> = pt::split_ref(words).into_iter().map(re::escape).collect();
    Re::new(&format!(r"\b{}\b", parts.join(r"\s+")), re::I)
}

/// Python: bot/memory/change.py:88 `_names(line, subject)`
///
/// Whether the line names the fact: the subject's own words, in any case.
pub fn _names(line: &str, subject: &str) -> bool {
    let words = pt::strip(subject);
    if words.is_empty() {
        return false;
    }
    the_subject_pattern(words).is_match(line)
}

/// Python: bot/memory/change.py:97 `_left_over(line, subject)`
///
/// The words of the line that are neither the fact, nor the change, nor filler.
pub fn _left_over(line: &str, subject: &str) -> Vec<String> {
    let mut mine = line.to_string();
    let words = pt::strip(subject);
    if !words.is_empty() {
        mine = the_subject_pattern(words).sub(" ", &mine);
    }
    mine = WHEN_TAIL.sub(" ", &mine);
    for pattern in [*CHANGED, *SUBJECT_CHANGED, *WANTS] {
        mine = pattern.sub(" ", &mine);
    }
    let of_the_subject = pt::split(&pt::lower(words));
    A_WORD
        .findall_str(&pt::lower(&mine))
        .into_iter()
        .filter(|word| !FILLER.contains(word) && !of_the_subject.contains(word))
        .collect()
}

/// Python: bot/memory/change.py:111 `reported(text, subject)`
///
/// How the line stands to the fact: "changed", "wanting", or None for anything else. The line
/// has to name the subject, and a line with a new value in it is left to the readers that keep
/// facts.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns "changed" / "wanting" / None.
pub fn reported(text: &str, subject: &str) -> Option<&'static str> {
    let line = pt::strip(text);
    if line.is_empty() || !_names(line, subject) {
        return None;
    }
    if HOW_TO.is_match(line) || NOT_WANTING.is_match(line) {
        return None;
    }
    let moved = CHANGED.search(line).or_else(|| SUBJECT_CHANGED.search(line))?;
    if !_left_over(line, subject).is_empty() {
        return None; // the line says more than the change: a value, a plan, a reason of its own
    }
    let before = &line[..moved.end()];
    if WANTS.is_match(before) {
        return Some("wanting");
    }
    Some("changed")
}

/// Python: bot/memory/change.py:135 `reply(kind, subject, was=None)`
///
/// What to say to a line that reports a change to the fact, or asks for one. `was` is the value
/// the store holds, as `str(was)`, or "" when it holds none.
pub fn reply(kind: &str, subject: &str, was: &str) -> String {
    let key = pt::lower(pt::strip(subject));
    if kind == "wanting" {
        return match WISHES.get(&key) {
            Some(wish) if wish.truthy() => wish.str().to_string(),
            _ => WISHED.clone(),
        };
    }
    let ask = match ASKS.get(&key) {
        Some(ask) if ask.truthy() => ask.str().to_string(),
        _ => CHANGED_ASK.clone(),
    };
    if was.is_empty() {
        return format!("{}{}", *KEPT, ask);
    }
    let settled = pt::rstrip_chars(pt::strip(was), ".");
    let settled = match settled.strip_prefix("no ") {
        Some(rest) => format!("You don't eat {rest}"),
        None => format!("You were {settled}"),
    };
    // `ask[0].lower() + ask[1:]` (an ask is never empty)
    mem7::fmt(&DROPPING, &[("was", &settled), ("ask", &mem7::low1(&ask))])
}
