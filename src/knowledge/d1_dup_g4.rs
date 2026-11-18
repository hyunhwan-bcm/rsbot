//! DUP ports (group 4 of workstream D1): private ports of helpers from modules other
//! workstreams own, each marked `DUP(<python module>::<name>)` so the integrator can replace it.
//! Also two small Python-compat helpers the group's modules share (`py_repr_str_list`,
//! `py_int`).

use crate::data::{self, Value};
use crate::knowledge::d1_shared as sh;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
// Python-compat helpers.

/// `repr(s)` for a Python str: single quotes unless the text holds a single quote and no double
/// quote, with backslash, the quote and control characters escaped as CPython does.
pub fn py_repr_str(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
    let mut out = String::with_capacity(s.len() + 2);
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => out.push_str(&format!("\\x{:02x}", c as u32)),
            c if (0x80..0xa0).contains(&(c as u32)) => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// `str(list_of_str)`: Python's repr of a list of strings, e.g. `['a', "don't"]`.
pub fn py_repr_str_list<S: AsRef<str>>(items: &[S]) -> String {
    let parts: Vec<String> = items.iter().map(|s| py_repr_str(s.as_ref())).collect();
    format!("[{}]", parts.join(", "))
}

/// `int(s)` for a string of ASCII digits (as the bot's `\d` captures and `isdigit()` words are,
/// in practice); None where Python's `int()` would raise. Values too large for i64 saturate.
pub fn py_int(s: &str) -> Option<i64> {
    let t = text::strip(s);
    let (neg, digits) = match t.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let digits: String = digits.chars().filter(|c| *c != '_').collect();
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let v = digits.parse::<i64>().unwrap_or(i64::MAX);
    Some(if neg { -v } else { v })
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.pets

// ---------------------------------------------------------------------------------------------
// bot.knowledge.health

/// DUP(bot.knowledge.health::tablets_for_a_pregnancy): a reply's "paracetamol or ibuprofen"
/// made paracetamol alone, with the pregnancy note, when the line says the person is pregnant.
pub fn health_tablets_for_a_pregnancy(reply: &str, understood: &Understood) -> String {
    static PREGNANT_AT_ALL: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.health._PREGNANT_AT_ALL"));
    static PREGNANT_DENIED: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.health._PREGNANT_DENIED"));
    static TABLETS_SAFE: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
        data::get("bot.knowledge.health.TABLETS_SAFE")
            .items()
            .iter()
            .map(|p| (p.items()[0].str().to_string(), p.items()[1].str().to_string()))
            .collect()
    });
    static NOTE: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.health.PREGNANT_TABLETS_NOTE"));
    let text = sh::squash_lower(sh::clean_or_text(understood));
    if !PREGNANT_AT_ALL.is_match(&text) || PREGNANT_DENIED.is_match(&text) {
        return reply.to_string();
    }
    let mut said = reply.to_string();
    let mut swapped = false;
    for (from, to) in TABLETS_SAFE.iter() {
        if said.contains(from.as_str()) {
            said = said.replace(from.as_str(), to);
            swapped = true;
        }
    }
    let lowered = said.to_lowercase();
    if ["ibuprofen", "nurofen"].iter().any(|w| lowered.contains(w)) {
        swapped = true;
    }
    if swapped {
        said.push_str(&NOTE);
    }
    said
}

// ---------------------------------------------------------------------------------------------
// bot.compose.repair / bot.knowledge.habits

/// DUP(bot.compose.repair::line_of): the line as the reading holds it (clean, else text, else
/// the words).
pub fn repair_line_of(reading: &Understood) -> String {
    if !reading.clean.is_empty() {
        return reading.clean.clone();
    }
    if !reading.text.is_empty() {
        return reading.text.clone();
    }
    reading.words.join(" ")
}

// was DUP(bot.knowledge.habits::_words_of): now the real port
pub use crate::knowledge::habits::_words_of as habits_words_of;

/// DUP(bot.compose.repair::deed_of): the deed the person reports doing to somebody close, or
/// None.
pub fn repair_deed_of(clean: &str) -> Option<String> {
    static PET_OR_PROFESSIONAL: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.PET_OR_PROFESSIONAL"));
    static PHYSICAL_HARM: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.PHYSICAL_HARM"));
    static GRIEF: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair._GRIEF"));
    static A_BABY: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair._A_BABY"));
    static REGRET_FIRST: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair._REGRET_FIRST"));
    static NOT_A_DEED: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.NOT_A_DEED"));
    static NOBODY_TO_REPAIR: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.NOBODY_TO_REPAIR"));
    fn table(name: &str) -> Vec<(String, &'static Re)> {
        data::get(name)
            .items()
            .iter()
            .map(|p| (p.items()[0].str().to_string(), p.items()[1].as_re().expect("deed pattern")))
            .collect()
    }
    static LAPSES: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| table("bot.compose.repair.LAPSES"));
    static DEEDS: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| table("bot.compose.repair.DEEDS"));
    let text = text::squash(&clean.to_lowercase().replace('\u{2019}', "'"));
    if text.is_empty() {
        return None;
    }
    if PET_OR_PROFESSIONAL.is_match(&text) || PHYSICAL_HARM.is_match(&text) {
        return None;
    }
    if GRIEF.is_match(&text) || A_BABY.is_match(&text) || REGRET_FIRST.is_match(&text) {
        return None;
    }
    for (deed, pattern) in LAPSES.iter() {
        if pattern.is_match(&text) {
            return Some(deed.clone());
        }
    }
    if NOT_A_DEED.is_match(&text) || NOBODY_TO_REPAIR.is_match(&text) {
        return None;
    }
    for (deed, pattern) in DEEDS.iter() {
        if pattern.is_match(&text) {
            return Some(deed.clone());
        }
    }
    None
}

/// DUP(bot.compose.repair::owns_the_deed): the line reports something the person did to
/// somebody close (read from the reading's clean line, else its text).
pub fn repair_owns_the_deed(reading: &Understood) -> bool {
    repair_deed_of(sh::clean_or_text(reading)).is_some()
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.lookup

// was DUP(bot.knowledge.lookup::names_a_thing): now the real port
pub use crate::knowledge::lookup::names_a_thing as lookup_names_a_thing;

// was DUP(bot.knowledge.lookup::a_food_is_being_kept): now the real port
pub use crate::knowledge::lookup::a_food_is_being_kept as lookup_a_food_is_being_kept;

// ---------------------------------------------------------------------------------------------
// bot.knowledge.wording

// ---------------------------------------------------------------------------------------------
// bot.memory.thread

/// The texts of `memory.thread.user_lines(memory)` (the real port; the readings are not read).
pub fn thread_user_lines(memory: &Memory) -> Vec<String> {
    crate::memory::thread::user_lines(memory).items().iter().map(|pair| pair.items()[0].str().to_string()).collect()
}

/// `memory.get(key)` for a key a module keeps in `extra`, as Python truthiness.
pub fn memory_truthy(memory: &Memory, key: &str) -> bool {
    memory.extra.get(key).is_some_and(Value::truthy)
}
