//! Port of `bot/modules/familyask.py`: a question about the bot's own family, answered honestly: it is a program and has none.
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
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.familyask.", stringify!($name))));
    )*};
}
rx!(THE_RELATION, A_HAVE_RELATION, A_RELATION, HOW_MANY_RELATION, A_QUESTION, AND_ANOTHER, NOT_ABOUT_THE_BOT, A_PAST_ASK, _NO_HAND_BACK, A_CHOICE);
static RELATION: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.modules.familyask.RELATION"));
static NO_ONE: LazyLock<&'static Value> = LazyLock::new(|| data::get("bot.modules.familyask.NO_ONE"));
static _HOW_IT_WENT: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.familyask._HOW_IT_WENT"));
static _THE_PEOPLE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.familyask._THE_PEOPLE"));

/// `THE_RELATION.search(said) or A_HAVE_RELATION.match(said) or HOW_MANY_RELATION.match(said)`.
fn the_match(said: &str) -> Option<Match<'_>> {
    THE_RELATION.search(said).or_else(|| A_HAVE_RELATION.match_(said)).or_else(|| HOW_MANY_RELATION.match_(said))
}

/// The words of the matched ask, lowered, with the marks on their ends taken off.
fn matched_words(m: &Match) -> Vec<String> {
    text::split(m.as_str()).iter().map(|w| text::strip_chars(&w.to_lowercase(), ",.'").to_string()).collect()
}

/// Python: bot/modules/familyask.py `the_relation(text)`
///
/// The relation the line asks about, echoed as the person wrote it, or None. "your mum and dad"
/// comes back as "mum and dad" because that is the pair they asked about.
pub fn the_relation(text: &str) -> Option<String> {
    let said = text::squash(text);
    let m = the_match(&said)?;
    let found: Vec<String> = matched_words(&m).into_iter().filter(|w| RELATION.contains_key(w)).collect();
    let first = found.first()?;
    let after = &said[m.end()..];
    if AND_ANOTHER.is_match(after) {
        if let Some(rest) = A_RELATION.search(after) {
            if rest.as_str().to_lowercase() != *first {
                return Some(format!("{} and {}", first, rest.as_str().to_lowercase()));
            }
        }
    }
    Some(first.clone())
}

/// Python: bot/modules/familyask.py `the_class(text)`
///
/// Which class of relation the line named, or None.
pub fn the_class(text: &str) -> Option<String> {
    let said = text::squash(text);
    let m = the_match(&said)?;
    matched_words(&m).iter().find_map(|word| RELATION.get(word).cloned())
}

/// Python: bot/modules/familyask.py `is_a_family_ask(text)`
///
/// True when the line asks the bot about a relation of its own, and asks at all.
pub fn is_a_family_ask(text: &str) -> bool {
    let said = text::squash(text);
    if said.is_empty() {
        return false;
    }
    if !THE_RELATION.is_match(&said) && A_HAVE_RELATION.match_(&said).is_none() && HOW_MANY_RELATION.match_(&said).is_none() {
        return false;
    }
    if NOT_ABOUT_THE_BOT.is_match(&said) {
        return false;
    }
    if A_CHOICE.is_match(&said) {
        return false;
    }
    A_QUESTION.match_(&said).is_some() || A_HAVE_RELATION.match_(&said).is_some() || HOW_MANY_RELATION.match_(&said).is_some() || said.ends_with('?')
}

/// Python: bot/modules/familyask.py `reply_for(text, memory=None)`
///
/// The honest answer to a question about the bot's own family, or None.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    if !is_a_family_ask(text) {
        return None;
    }
    let said = text::squash(text);
    let kind = the_class(&said).filter(|kind| !kind.is_empty()).unwrap_or_else(|| "family".to_string());
    let told = NO_ONE.get(&kind).unwrap_or_else(|| panic!("KeyError: {kind:?}")).strs();
    let said_of: &[String] = if A_PAST_ASK.is_match(&said) { &_HOW_IT_WENT } else { &_THE_PEOPLE };
    let seed = fold(text);
    let honest = pick(&told, memory, &seed);
    if _NO_HAND_BACK.is_match(&said) {
        return Some(honest);
    }
    Some(format!("{} {}", honest, pick(said_of, memory, &seed)))
}

/// Python: bot/modules/familyask.py `TAKES_AS_IT_COMES(text, memory)`
///
/// A question about the bot's own family is this module's line whole.
pub fn TAKES_AS_IT_COMES(text: &str, memory: &Memory) -> bool {
    is_a_family_ask(text)
}

/// Python: bot/modules/familyask.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The line a question about the bot's family gets; None for every other line.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    reply_for(text, memory)
}
