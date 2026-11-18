//! Port of `bot/understanding/confusion.py`: the person says they are confused, and by what.

#![allow(non_upper_case_globals)]

use crate::py::re::Re;
use crate::understanding::normalize::Line;
use crate::understanding::t;
use crate::understood::Confusion;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.understanding.confusion.";

static CONFUSION_BY_A_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONFUSION_BY_A_THING"));
static CONFUSION_CANNOT_FOLLOW: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONFUSION_CANNOT_FOLLOW"));
static CONFUSION_CUT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CONFUSION_CUT"));
static CONFUSION_HARD_TO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONFUSION_HARD_TO"));
static CONFUSION_HEAD_NOR_TAIL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONFUSION_HEAD_NOR_TAIL"));
static CONFUSION_HEAD_SPINNING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONFUSION_HEAD_SPINNING"));
static CONFUSION_HELPER_NOT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONFUSION_HELPER_NOT"));
static CONFUSION_MAKES_NO_SENSE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONFUSION_MAKES_NO_SENSE"));
static CONFUSION_POINTS_AT_NOTHING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONFUSION_POINTS_AT_NOTHING"));
static CONFUSION_SAID_OF_A_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONFUSION_SAID_OF_A_THING"));
static CONFUSION_TRAILING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CONFUSION_TRAILING"));
static DENY_BEFORE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DENY_BEFORE"));

/// `_a_confusion_ruled_out(before_cue)`: "i'm not confused".
fn _a_confusion_ruled_out(before_cue: &str) -> bool {
    DENY_BEFORE.is_match(&CONFUSION_HELPER_NOT.sub("", before_cue))
}

/// `_a_thing_trimmed(thing)`: the thing they are confused by, trimmed (None when too short).
fn _a_thing_trimmed(thing: Option<&str>) -> Option<String> {
    let mut words: Vec<&str> = Vec::new();
    for word in crate::py::text::split_ref(thing.unwrap_or("")) {
        if CONFUSION_CUT.contains(word) {
            break;
        }
        words.push(word);
    }
    while words.last().is_some_and(|w| CONFUSION_TRAILING.contains(*w)) {
        words.pop();
    }
    if words.len() < 2 || !words[1..].iter().any(|w| !CONFUSION_TRAILING.contains(*w)) {
        return None;
    }
    Some(words.join(" "))
}

/// `confusion_of(line)`: how the line says the person is confused, or None.
pub fn confusion_of(line: &Line) -> Option<Confusion> {
    let shapes: [(&Re, &str); 7] = [
        (&CONFUSION_HARD_TO, "hard_to"),
        (&CONFUSION_SAID_OF_A_THING, "said_of_a_thing"),
        (&CONFUSION_MAKES_NO_SENSE, "said_of_a_thing"),
        (&CONFUSION_CANNOT_FOLLOW, "cannot_follow"),
        (&CONFUSION_HEAD_NOR_TAIL, "cannot_follow"),
        (&CONFUSION_BY_A_THING, "by_a_thing"),
        (&CONFUSION_HEAD_SPINNING, "head_spinning"),
    ];
    for part in line.norm_text.split('|') {
        let clause = crate::py::text::strip(part);
        if clause.is_empty() {
            continue;
        }
        if CONFUSION_POINTS_AT_NOTHING.is_match(clause) {
            return Some(Confusion { shape: "nothing_named".to_string(), thing: None });
        }
        for (pattern, shape) in shapes {
            let Some(found) = pattern.search(clause) else { continue };
            let cut = match found.name_span("cue") {
                Some((s, _)) => s,
                None if pattern.pattern().contains("(?P<cue>") => found.start(),
                None => found.start(),
            };
            // `found.start('cue')` is -1 when the group did not take part: clause[:-1]
            let before_cue = if found.name_span("cue").is_none() && pattern.pattern().contains("(?P<cue>") {
                crate::py::text::slice(clause, 0, Some(-1))
            } else {
                clause[..cut].to_string()
            };
            if _a_confusion_ruled_out(&before_cue) {
                continue;
            }
            let thing = _a_thing_trimmed(found.name("thing"));
            return Some(Confusion { shape: shape.to_string(), thing });
        }
    }
    None
}

#[allow(dead_code)]
fn _unused(_: &HashSet<String>) {}
