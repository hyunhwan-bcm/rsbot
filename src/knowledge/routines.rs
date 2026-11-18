//! Port of PyBot's `bot/knowledge/routines.py`: starting a routine, keeping it going after a
//! slip and stopping one you do not want (the guards that keep a machine or a pot of food that
//! "keeps" doing something, or a bill, off the habit rows).

use crate::data::{self, FnRef};
use crate::knowledge::d1_dup_g4 as dup;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

/// Module globals from the bundle, by their Python name.
macro_rules! global {
    ($name:ident: re) => {
        static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.knowledge.routines.", stringify!($name))));
    };
    ($name:ident: set) => {
        static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.knowledge.routines.", stringify!($name))));
    };
    ($name:ident: strs) => {
        static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.knowledge.routines.", stringify!($name))));
    };
}

global!(MACHINE_THAT_KEEPS: re);
global!(FOOD_THAT_KEEPS: re);
global!(_STATE_WORDS: re);
global!(GOODS_THAT_KEEP: re);
global!(KEEPS_SUBJECTS: set);
global!(KEEPS_VERB: re);
global!(_MONEY_WORDS: strs);

/// True when the thing that "keeps" doing something is a machine, a kept food, or a drink or
/// dish that keeps going cold: a fault, not a habit.
pub fn _a_machine_behaving_badly(understood: &Understood) -> bool {
    if dup::lookup_a_food_is_being_kept(understood) {
        return true;
    }
    let said = sh::clean_or_text(understood);
    if MACHINE_THAT_KEEPS.is_match(said) {
        return true;
    }
    GOODS_THAT_KEEP.is_match(said) && _STATE_WORDS.is_match(said)
}

/// True when whatever "keeps" doing something in the line is a thing (a machine, a food, hair,
/// a knee), not a person or a habit.
pub fn _a_thing_behaving_badly(understood: &Understood) -> bool {
    if _a_machine_behaving_badly(understood) {
        return true;
    }
    let value = sh::clean_or_text(understood);
    if value.is_empty() {
        return false;
    }
    let said = value.to_lowercase();
    if FOOD_THAT_KEEPS.is_match(&said) {
        return true;
    }
    let subjects = KEEPS_VERB.findall_str(&said);
    !subjects.is_empty() && !subjects.iter().all(|w| KEEPS_SUBJECTS.contains(w))
}

/// False when the line is about paying for something (a bill is not a habit).
pub fn _not_about_money(understood: &Understood) -> bool {
    // `raw` is not a key of the reading: it reads as "".
    let said = [understood.clean.as_str(), understood.norm_text.as_str(), understood.text.as_str(), ""]
        .join(" ")
        .to_lowercase();
    !sh::any_in(&said, &_MONEY_WORDS)
}

/// The keep-a-habit-going row's guard (`<lambda>:250`): no thing behaving badly.
pub fn lambda_250(understood: &Understood) -> bool {
    !_a_thing_behaving_badly(understood)
}

/// The screen-time row's guard (`<lambda>:490`): not about money, and the screens are not
/// somebody else's.
pub fn lambda_490(reading: &Understood) -> bool {
    _not_about_money(reading) && !crate::knowledge::wording::screens_are_somebody_elses(reading)
}

// ---------------------------------------------------------------------------------------------
// Registry tables.

fn g_lambda_250(_f: &FnRef, u: &Understood) -> bool { lambda_250(u) }
fn g_lambda_490(_f: &FnRef, u: &Understood) -> bool { lambda_490(u) }

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.routines:<lambda>:250", g_lambda_250),
    ("bot.knowledge.routines:<lambda>:490", g_lambda_490),
];
pub const REPLIES: &[(&str, ReplyFn)] = &[];
