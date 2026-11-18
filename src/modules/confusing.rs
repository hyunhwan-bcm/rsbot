//! Port of `bot/modules/confusing.py`: a thing the person cannot follow is asked about, where the composer would only shrug.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::compose::{fallback, fallback_chain, hedge};

static WITH_A_THING_LINES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.confusing.WITH_A_THING_LINES"));
static WITHOUT_A_THING_LINES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.confusing.WITHOUT_A_THING_LINES"));
static SECOND_PERSON: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.modules.confusing.SECOND_PERSON"));
static SECOND_PERSON_RE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.confusing.SECOND_PERSON_RE"));

/// Python: bot/modules/confusing.py `_said_back(thing)`
///
/// The thing as it is said to the person: "my payslip" -> "your payslip".
pub fn _said_back(thing: &str) -> String {
    let said = SECOND_PERSON_RE.sub_fn(thing, |m| SECOND_PERSON.get(m.str(1)).cloned().unwrap_or_else(|| panic!("KeyError: {:?}", m.str(1))));
    text::strip(&said).to_string()
}

/// Python: bot/modules/confusing.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The reply to a thing the person cannot follow, or None for every other line.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let confusion = memory.understood.as_ref().and_then(|understood| understood.confusion.clone())?;
    let clean = fallback::_matchable(text);
    // Nothing is taken from an answer the composer has. A form it can help with, a medication row,
    // the foggy question for a bare "im confused": those are answers, and a question in their place
    // is worse. Only the lines it says nothing to - and the congratulation it gives a line about a
    // thing that would not go in - are this module's (bot/compose/hedge.py).
    if !ret_true(hedge::only_a_shrug(text, memory, data::get("bot.compose.hedge.MISREAD_AS_GOOD_NEWS"))) {
        return None;
    }
    let thing = _said_back(confusion.thing.as_deref().unwrap_or(""));
    // The pool is drawn from the one random stream every module shares, so answering here must not
    // spend a draw the next turn would have had: a chat that took this line and then changed the
    // reply to the line after it is worse than a chat this module never touched. So the state is
    // put back the way `bot/compose/hedge.py` puts it back after asking the composer.
    let state = crate::py::random::getstate();
    let chosen = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if !thing.is_empty() {
            let lines: Vec<String> = WITH_A_THING_LINES.iter().map(|line| fmt_named(line, &[("thing", thing.as_str())])).collect();
            return as_reply(fallback_chain::_choose(&lines, memory, &clean, None));
        }
        as_reply(fallback_chain::_choose(&WITHOUT_A_THING_LINES, memory, &clean, None))
    }));
    // `finally: random.setstate(state)`
    crate::py::random::setstate(state);
    match chosen {
        Ok(reply) => reply,
        Err(raised) => std::panic::resume_unwind(raised),
    }
}
