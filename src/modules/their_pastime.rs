//! Port of `bot/modules/their_pastime.py`: a whole-line fact about one of their people - what they are into, what they do - gets a question about it.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::compose::{fallback, fallback_chain, remark};

macro_rules! rx {
    ($($name:ident),*) => {$(
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.their_pastime.", stringify!($name))));
    )*};
}
macro_rules! pool {
    ($($name:ident),*) => {$(
        static $name: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(concat!("bot.modules.their_pastime.", stringify!($name))));
    )*};
}
macro_rules! wordset {
    ($($name:ident),*) => {$(
        static $name: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(concat!("bot.modules.their_pastime.", stringify!($name))));
    )*};
}
rx!(THEIR_INTO, THEIR_LOVES, THEIR_DOES, THEIR_TOOK_UP, THEIR_KEEPS, THEIR_GOOD_AT, THEIR_VISIT, THEIR_IS_A_ROLE, HEAVY_IN_THE_THREAD);
pool!(THEIRS_ROLE_LINES, THEIRS_INTO_LINES, THEIRS_LOVES_LINES, THEIRS_DOES_LINES, THEIRS_GOOD_AT_LINES, THEIRS_VISIT_LINES, NOT_TAKEN_UP_PHRASES, PLACEHOLDER_OPENINGS);
wordset!(NOT_TAKEN_UP, NOT_PLURAL, BEFORE_AN_AUDIENCE);
static NOT_A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z\-]+", 0));
static NOT_A_LETTER: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z]+", 0));
static NOT_A_LINE_CHAR: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' ]+", 0));
static SPACES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+", 0));

/// Python: bot/modules/their_pastime.py `_starts_a_sentence(line)`
///
/// The pool line with its first letter capitalised, since it opens the reply.
pub fn _starts_a_sentence(line: &str) -> String {
    cap1(line)
}

/// Python: bot/modules/their_pastime.py `_the_turn_before_was_a_feeling(clean, memory)`
///
/// True when the last line they said was a feeling, so this fact is the reason for it. The
/// reading is the store's own (bot/memory/ack.py).
pub fn _the_turn_before_was_a_feeling(clean: &str, memory: &Memory) -> bool {
    ret_true(crate::memory::ack::_after_a_feeling(clean, memory))
}

/// Python: bot/modules/their_pastime.py `_a_thing_nobody_is_into(thing)`
///
/// True for the things that are a problem rather than a hobby ("is into drugs"). A phrase is
/// read first, because the words in "another woman" mean nothing on their own.
pub fn _a_thing_nobody_is_into(thing: &str) -> bool {
    let said = fold(thing);
    if NOT_TAKEN_UP_PHRASES.iter().any(|phrase| said.contains(phrase.as_str())) {
        return true;
    }
    NOT_A_WORD.split(&said).iter().any(|word| !word.is_empty() && NOT_TAKEN_UP.contains(word))
}

/// Python: bot/modules/their_pastime.py `_a_plural_thing(thing)`
///
/// True when the thing named is plural, so the reply says "them" and not "it".
pub fn _a_plural_thing(thing: &str) -> bool {
    let words: Vec<String> = NOT_A_WORD.split(&thing.to_lowercase()).into_iter().filter(|word| !word.is_empty()).collect();
    let Some(last) = words.last() else { return false };
    if NOT_PLURAL.contains(last) || text::len(last) < 4 {
        return false;
    }
    last.ends_with('s') && !["ss", "us", "is", "ous"].iter().any(|end| last.ends_with(end))
}

/// Python: bot/modules/their_pastime.py `_it_word(thing)`
///
/// The word the pool line uses for the thing: "them" for dinosaurs, "it" for baking.
pub fn _it_word(thing: &str) -> String {
    if _a_plural_thing(thing) { "them" } else { "it" }.to_string()
}

/// Python: bot/modules/their_pastime.py `_watched(thing)`
///
/// True when the thing named is one somebody watches as well as does.
pub fn _watched(thing: &str) -> bool {
    let words: Vec<String> = NOT_A_LETTER.split(&thing.to_lowercase()).into_iter().filter(|word| !word.is_empty()).collect();
    if words.iter().any(|word| BEFORE_AN_AUDIENCE.contains(word)) {
        return true;
    }
    BEFORE_AN_AUDIENCE.contains(&words.join(" "))
}

/// Python: bot/modules/their_pastime.py `_does_pool(thing)`
///
/// The pool for a thing they do, without the audience question for a thing done alone.
pub fn _does_pool(thing: &str) -> Vec<String> {
    if _watched(thing) {
        return THEIRS_DOES_LINES.clone();
    }
    THEIRS_DOES_LINES.iter().filter(|line| !line.contains("watch")).cloned().collect()
}

/// Python: bot/modules/their_pastime.py `_fill_it(line, person_word, it_word="it")`
///
/// The pool line with the person's pronouns, the verb that agrees, and the thing's number.
/// {they} and the rest of the pronouns are the store's own fill_person (bot/compose/remark.py);
/// {has}, {is} and {it} are this pool's.
pub fn _fill_it(line: &str, person_word: &str, it_word: &str) -> String {
    // `they, them, their, do, are = person_pronouns(person_word)`
    let pronouns = remark::person_pronouns(person_word).val();
    let (do_, are) = match pronouns.items() {
        [_, _, _, do_, are] => (py_str(do_), py_str(are)),
        _ => panic!("ValueError: not enough values to unpack (expected 5)"),
    };
    let said = ret_str(remark::fill_person(line, person_word));
    let said = said.replace("{has}", if do_ == "do" { "have" } else { "has" }).replace("{is}", if are == "are" { "are" } else { "is" }).replace("{it}", it_word);
    let said = said
        .replace("has they", "have they")
        .replace("Has they", "Have they")
        .replace("is they", "are they")
        .replace("Is they", "Are they")
        .replace("does they", "do they")
        .replace("Does they", "Do they");
    cap1(&said)
}

/// Python: bot/modules/their_pastime.py `reading(clean, words, memory)`
///
/// The pool line for a whole-line fact about one of their people, or None.
pub fn reading(clean: &str, words: &[String], memory: &Memory) -> Option<String> {
    if _the_turn_before_was_a_feeling(clean, memory) {
        return None;
    }
    if let Some(m) = THEIR_IS_A_ROLE.match_(clean) {
        if let Some(job) = or_none(remark::_the_job_in(m.name_str("role"))) {
            return Some(_fill_it(&pick(&THEIRS_ROLE_LINES, memory, clean).replace("{Job}", &job), m.name_str("person"), "it"));
        }
    }
    if let Some(m) = THEIR_INTO.match_(clean).filter(|m| !_a_thing_nobody_is_into(m.name_str("thing"))) {
        return Some(_fill_it(&pick(&THEIRS_INTO_LINES, memory, clean), m.name_str("person"), &_it_word(m.name_str("thing"))));
    }
    if let Some(m) = THEIR_LOVES.match_(clean).filter(|m| !_a_thing_nobody_is_into(m.name_str("thing"))) {
        return Some(_fill_it(&pick(&THEIRS_LOVES_LINES, memory, clean), m.name_str("person"), &_it_word(m.name_str("thing"))));
    }
    if let Some(m) = THEIR_DOES.match_(clean).filter(|m| !_a_thing_nobody_is_into(m.name_str("thing"))) {
        return Some(_fill_it(&pick(&_does_pool(m.name_str("thing")), memory, clean), m.name_str("person"), "it"));
    }
    if let Some(m) = THEIR_GOOD_AT.match_(clean).filter(|m| !_a_thing_nobody_is_into(m.name_str("thing"))) {
        return Some(_fill_it(&pick(&THEIRS_GOOD_AT_LINES, memory, clean), m.name_str("person"), "it"));
    }
    if let Some(m) = THEIR_KEEPS.match_(clean) {
        return Some(_fill_it(&pick(&THEIRS_DOES_LINES, memory, clean), m.name_str("person"), "it"));
    }
    if let Some(m) = THEIR_TOOK_UP.match_(clean).filter(|m| !_a_thing_nobody_is_into(m.name_str("thing"))) {
        return Some(_fill_it(&pick(&_does_pool(m.name_str("thing")), memory, clean), m.name_str("person"), "it"));
    }
    if let Some(m) = THEIR_VISIT.match_(clean) {
        return Some(_fill_it(&pick(&THEIRS_VISIT_LINES, memory, clean), m.name_str("person"), "it"));
    }
    None
}

/// Python: bot/modules/their_pastime.py `_is_a_placeholder(said)`
///
/// True when the composer's own reply for this line is one of the placeholder shapes.
pub fn _is_a_placeholder(said: &str) -> bool {
    let lowered = text::strip(said).to_lowercase();
    PLACEHOLDER_OPENINGS.iter().any(|opening| lowered.starts_with(opening.as_str()))
}

/// Python: bot/modules/their_pastime.py `_the_thread_is_heavy(memory)`
///
/// True when the line just before this one was a person ill, lost, bullied or in hospital.
pub fn _the_thread_is_heavy(memory: &Memory) -> bool {
    let mut before = ret_str(fallback::_the_line_before(memory));
    if before.is_empty() {
        before = last_user(memory);
    }
    let said = before.to_lowercase();
    if said.is_empty() {
        return false;
    }
    if ret_true(fallback::_reads_as_heavy_news(&said)) {
        return true;
    }
    HEAVY_IN_THE_THREAD.is_match(&said)
}

/// Python: bot/modules/their_pastime.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The turn for a fact about one of their people, when nothing better answers it.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    if text.is_empty() || text.contains('?') {
        return None;
    }
    let clean = NOT_A_LINE_CHAR.sub(" ", &text.to_lowercase());
    let clean = text::strip(&SPACES.sub(" ", &clean)).to_string();
    if !clean.starts_with("my ") {
        return None;
    }
    if _the_thread_is_heavy(memory) {
        return None;
    }
    let mine = reading(&clean, &text::split(&clean), memory).filter(|mine| !mine.is_empty())?;
    // a composer that cannot answer leaves this module its turn
    let theirs = py_try(|| or_none(fallback_chain::reply_for(text, memory, true))).unwrap_or(None);
    if let Some(theirs) = theirs {
        if !_is_a_placeholder(&theirs) {
            return None;
        }
    }
    Some(mine)
}
