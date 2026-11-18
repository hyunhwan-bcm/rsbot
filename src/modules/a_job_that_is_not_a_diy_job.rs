//! Port of `bot/modules/a_job_that_is_not_a_diy_job.py`: a job that is not one to try yourself - gas, a chimney, a load-bearing wall - and who does it.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

macro_rules! words {
    ($($name:ident),*) => {$(
        static $name: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(concat!("bot.modules.a_job_that_is_not_a_diy_job.", stringify!($name))));
    )*};
}
words!(SELF_WORDS, NOT_TRYING_IT, NOW_WORDS, NOT_A_TREE, CARRIED_AND_UNPLUGGED, THE_WIRING_ITSELF, NOT_A_JOB_TO_TRY_POOL);
static NOT_A_JOB_TO_TRY: LazyLock<&'static Value> = LazyLock::new(|| data::get("bot.modules.a_job_that_is_not_a_diy_job.NOT_A_JOB_TO_TRY"));
static TRY_IT_MYSELF: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.a_job_that_is_not_a_diy_job.TRY_IT_MYSELF"));
static A_HAZARD_NAMING_THE_JOB: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.a_job_that_is_not_a_diy_job.A_HAZARD_NAMING_THE_JOB"));
static REGISTER_HINT: LazyLock<&'static Value> = LazyLock::new(|| data::get("bot.modules.a_job_that_is_not_a_diy_job.REGISTER_HINT"));

/// Python: bot/modules/a_job_that_is_not_a_diy_job.py `_has(line, phrases)`
///
/// True when one of the phrases is in the line as a whole word or phrase.
pub fn _has(line: &str, phrases: &[String]) -> bool {
    phrases.iter().any(|phrase| Re::new(&format!("(?<![a-z0-9]){}(?![a-z0-9])", pyre::escape(phrase)), 0).is_match(line))
}

/// Python: bot/modules/a_job_that_is_not_a_diy_job.py `_the_job(said)`
///
/// (who does it, why it is not a weekend's job) for the job this line names, or None.
pub fn _the_job(said: &str) -> Option<(String, String)> {
    NOT_A_JOB_TO_TRY.items().iter().find(|job| job.items()[0].as_re().unwrap().is_match(said)).map(|job| (job.items()[1].str().to_string(), job.items()[2].str().to_string()))
}

/// Python: bot/modules/a_job_that_is_not_a_diy_job.py `respond(text, memory=None)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let said = fold(text);
    if said.is_empty() {
        return None;
    }
    if _has(&said, &NOT_TRYING_IT) || _has(&A_HAZARD_NAMING_THE_JOB.sub("", &said), &NOW_WORDS) {
        return None;
    }
    if !TRY_IT_MYSELF.is_match(&said) && !_has(&said, &SELF_WORDS) {
        return None;
    }
    if _has(&said, &NOT_A_TREE) {
        return None;
    }
    if _has(&said, &CARRIED_AND_UNPLUGGED) && !_has(&said, &THE_WIRING_ITSELF) {
        return None;
    }
    let (who, why) = _the_job(&said)?;
    let who_cap = cap1(&who);
    let mut said_pool: Vec<String> =
        NOT_A_JOB_TO_TRY_POOL.iter().map(|line| fmt_named(line, &[("who", who.as_str()), ("who_cap", who_cap.as_str()), ("why", why.as_str())])).collect();
    if let Some(hint) = REGISTER_HINT.get(&who).filter(|hint| hint.truthy()) {
        said_pool = said_pool.iter().map(|line| format!("{} {}", line, hint.str())).collect();
    }
    let last = text::strip(&last_bot(memory)).to_string();
    let fresh: Vec<String> = said_pool.iter().filter(|line| **line != last).cloned().collect();
    // `random.choice`: the shared stream, as in Python
    Some(crate::py::random::choice(if fresh.is_empty() { &said_pool } else { &fresh }))
}
