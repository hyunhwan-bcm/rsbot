//! Port of `bot/modules/particles.py`: the verb and the particle of a job worded after a thing ("my dress taken up").
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

static A_PARTICLE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.modules.particles.A_PARTICLE"));
static A_VERB_TAIL: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.modules.particles.A_VERB_TAIL"));

/// Python: bot/modules/particles.py `a_thing_without_its_tail(said)`
///
/// The thing said back without the job worded after it: "your dress taken up" -> "your dress".
/// Only whole words come off, and never the last one: a line that is all tail keeps what it has,
/// because an empty thing is worse than an awkward one.
pub fn a_thing_without_its_tail(said: &str) -> String {
    let mut words = text::split(said);
    while words.len() > 1 {
        let last = words[words.len() - 1].to_lowercase();
        let last = text::strip_chars(&last, ".,!?");
        if !(A_PARTICLE.contains(last) || A_VERB_TAIL.contains(last)) {
            break;
        }
        words.pop();
    }
    words.join(" ")
}

/// Python: bot/modules/particles.py `a_job_word(word)`
///
/// True when this word is the verb or particle of a job, not a thing to pick a kind of.
pub fn a_job_word(word: &str) -> bool {
    let lowered = word.to_lowercase();
    let word = text::strip_chars(&lowered, " .,!?;:'\"()");
    A_VERB_TAIL.contains(word) || A_PARTICLE.contains(word)
}
