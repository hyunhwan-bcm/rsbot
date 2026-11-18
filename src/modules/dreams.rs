//! Port of `bot/modules/dreams.py`: a dream somebody is telling, answered as a dream and not as the thing it shows (`bot/memory/dream.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::memory::dream as _dream;

macro_rules! rx {
    ($($name:ident),*) => {$(
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.dreams.", stringify!($name))));
    )*};
}
macro_rules! pool {
    ($($name:ident),*) => {$(
        static $name: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(concat!("bot.modules.dreams.", stringify!($name))));
    )*};
}
rx!(SOMEONE_DIED, A_FIRE, A_BREAK_IN, CHASED_OR_FALLING, ILL_IN_THE_DREAM, A_GOOD_ONE, DIED_MYSELF, ATTACKED, TEETH, AN_EX, A_BABY, PEOPLE_HURT, A_VISIT);
pool!(DEATH_LINES, BREAK_IN_LINES, FIRE_LINES, CHASE_LINES, TEETH_LINES, EX_LINES, BABY_LINES, GOOD_LINES, OWN_DEATH_LINES, ATTACK_LINES, VISIT_LINES, PEOPLE_HURT_LINES, ILL_LINES, PLAIN_LINES, SETTLED_LINES);

/// Python: bot/modules/dreams.py `_settled(line, choices)`
///
/// One of the lines, picked by the words so the same dream gets the same answer.
pub fn _settled(line: &str, choices: &[String]) -> String {
    choices[(crate::py::random::ord_sum(line) as usize) % choices.len()].clone()
}

/// Python: bot/modules/dreams.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// An answer for a dream somebody is telling, or None for every other line.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let line = fold(text);
    let scene = ret_str(_dream::the_scene(&line));
    if !scene.is_empty() && !ret_true(_dream::a_dream_is_not_happening(&line)) {
        return None;
    }
    if scene.is_empty() {
        if data::re("bot.memory.dream.SETTLED_IT").is_match(&line) && ret_true(_dream::told_in_the_turn_before(memory)) {
            return Some(_settled(&line, &SETTLED_LINES));
        }
        return None;
    }
    // `line[:line.find(scene)]`: -1 for a scene that is not in the line, which is the line less
    // its last character.
    let before = match line.find(&scene) {
        Some(at) => line[..at].to_string(),
        None => sl(&line, 0, Some(-1)),
    };
    if SOMEONE_DIED.is_match(&before) || A_FIRE.is_match(&before) {
        return None;
    }
    let scene = scene.as_str();
    let lines: &[String] = if DIED_MYSELF.is_match(scene) {
        &OWN_DEATH_LINES
    } else if ATTACKED.is_match(scene) {
        &ATTACK_LINES
    } else if ILL_IN_THE_DREAM.is_match(scene) && !PEOPLE_HURT.is_match(scene) {
        &ILL_LINES
    } else if SOMEONE_DIED.is_match(scene) {
        &DEATH_LINES
    } else if TEETH.is_match(scene) {
        &TEETH_LINES
    } else if A_FIRE.is_match(scene) {
        &FIRE_LINES
    } else if A_BREAK_IN.is_match(scene) {
        &BREAK_IN_LINES
    } else if A_BABY.is_match(scene) {
        &BABY_LINES
    } else if AN_EX.is_match(scene) {
        &EX_LINES
    } else if A_VISIT.is_match(scene) {
        &VISIT_LINES
    } else if PEOPLE_HURT.is_match(scene) {
        &PEOPLE_HURT_LINES
    } else if CHASED_OR_FALLING.is_match(scene) {
        &CHASE_LINES
    } else if A_GOOD_ONE.is_match(scene) {
        &GOOD_LINES
    } else {
        &PLAIN_LINES
    };
    Some(_settled(&line, lines))
}

/// Python: bot/modules/dreams.py `CLAIMS_INSTEAD(text, memory)`
///
/// True on the dream lines another reader claims as it comes, where this module has the answer.
pub fn CLAIMS_INSTEAD(text: &str, memory: &Memory) -> Option<bool> {
    // a module that cannot answer claims nothing
    Some(or_on_raise(|| respond(text, &mut memory.clone()).map_or(false, |reply| !reply.is_empty()), false))
}
