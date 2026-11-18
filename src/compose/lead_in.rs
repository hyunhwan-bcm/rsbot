//! Port of `bot/compose/lead_in.py`: a question behind a lead-in ("and is b&q open on sunday")
//! gets the answer the question has on its own, when the lead-in left the composer with only
//! the floor, a nod or an echo.

use crate::compose::{fallback, fallback_chain};
use crate::data;
use crate::memory::ack;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.compose.lead_in.";
static LEAD_IN: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}LEAD_IN")));
static A_QUESTION: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}A_QUESTION")));
static FEWEST_WORDS: LazyLock<usize> = LazyLock::new(|| data::get(&format!("{M}FEWEST_WORDS")).as_int().unwrap() as usize);
static THE_FLOOR_OR_A_NOD: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}THE_FLOOR_OR_A_NOD")));
static NAMED_HELP_NOW: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}NAMED_HELP_NOW")));
static NO_NEWS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}NO_NEWS")));
/// `re.findall(r"[a-z0-9&']+", ...)`, written inline in `_an_echo_of_the_ask`.
static NEWS_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9&']+", 0));

/// Python: bot/compose/lead_in.py:56 `_only_the_floor_or_a_nod(said)`
///
/// True when the reply is nothing but the floor, a nod or a receipt.
pub fn _only_the_floor_or_a_nod(said: &str) -> bool {
    let text = text::strip(said);
    if text.is_empty() {
        return true;
    }
    if fallback::is_a_hedge(text) {
        return true;
    }
    // A receipt test that cannot run must not stop the module.
    if fallback_chain::caught(|| ack::reads_as_a_receipt(text)).unwrap_or(false) {
        return true;
    }
    THE_FLOOR_OR_A_NOD.match_(text).is_some()
}

/// Python: bot/compose/lead_in.py:71 `_the_rest_after_the_lead_in(line)`
///
/// The question the line asks once the lead-in is off it, or "" when there is no lead-in.
pub fn _the_rest_after_the_lead_in(line: &str) -> String {
    let Some(found) = LEAD_IN.match_(line) else { return String::new() };
    let rest = text::strip(&line[found.end()..]);
    if rest.is_empty() || A_QUESTION.match_(rest).is_none() {
        return String::new();
    }
    if text::split_ref(rest).len() < *FEWEST_WORDS {
        return String::new();
    }
    rest.to_string()
}

/// Python: bot/compose/lead_in.py:84 `_a_copy(memory)`
///
/// A copy of memory to ask a question of, the way the runtime makes one for a module (only this
/// file calls it).
pub fn _a_copy(memory: &Memory) -> Memory {
    memory.clone()
}

/// Python: bot/compose/lead_in.py:93 `_read_as_its_own_line(rest, memory)`
///
/// The conversation's memory, with the understanding reread for the shorter line (only this file
/// calls it).
pub fn _read_as_its_own_line(rest: &str, memory: &Memory) -> Memory {
    let mut mine = _a_copy(memory);
    // Without the reread the composer sees the whole line; a failure keeps what was written.
    let _ = fallback_chain::caught(|| crate::modules::understand::observe(rest, &mut mine));
    mine
}

/// Python: bot/compose/lead_in.py:126 `_an_echo_of_the_ask(said, line)`
///
/// True when the reply says nothing the ask did not already say.
pub fn _an_echo_of_the_ask(said: &str, line: &str) -> bool {
    let asked: HashSet<String> = NEWS_WORDS.findall_str(&text::lower(line)).into_iter().collect();
    NEWS_WORDS.findall_str(&text::lower(said)).iter().all(|word| NO_NEWS.contains(word) || asked.contains(word))
}

/// Python: bot/compose/lead_in.py:140 `_a_reply_of_its_own(said)`
pub fn _a_reply_of_its_own(said: &str) -> bool {
    !said.is_empty() && !_only_the_floor_or_a_nod(said)
}

/// `fallback.reply_for(text, memory)` (verdict=True); "" is Python's None.
fn reply_for(text: &str, mut memory: Memory) -> String {
    fallback_chain::reply_for(text, &mut memory, true)
}

/// Python: bot/compose/lead_in.py:145 `the_answer_the_lead_in_hid(text, memory)`
///
/// The reply the ask would have had without the lead-in, or None.
pub fn the_answer_the_lead_in_hid(text: &str, memory: &Memory) -> Option<String> {
    let line = text::squash(text::strip(text));
    let rest = _the_rest_after_the_lead_in(&line);
    if rest.is_empty() {
        return None;
    }
    let whole = reply_for(&line, _a_copy(memory));
    if !_only_the_floor_or_a_nod(&whole) && !_an_echo_of_the_ask(&whole, &line) {
        return None;
    }
    let mut without = reply_for(&rest, _read_as_its_own_line(&rest, memory));
    if !_a_reply_of_its_own(&without) || without == whole {
        let mut alone = memory.clone();
        alone.turns.clear();
        let fresh = reply_for(&rest, _read_as_its_own_line(&rest, &alone));
        if !fresh.is_empty() && _a_reply_of_its_own(&fresh) && fresh != whole {
            without = fresh;
        }
    }
    if without.is_empty() {
        return None;
    }
    if without == whole {
        return None; // the lead-in changed nothing
    }
    if NAMED_HELP_NOW.is_match(&whole) || NAMED_HELP_NOW.is_match(&without) {
        return None; // an emergency reply is never replaced, and never replaces one
    }
    if !_a_reply_of_its_own(&without) {
        return None;
    }
    Some(without)
}
