//! Helpers the D2 knowledge modules share: small Python-behaviour shims, and private ports of
//! helpers that live in modules other workstreams own (`lookup.py`, `compose`). Each of those is
//! marked `// DUP(<python module>::<name>)` so the integrator can swap in the owner's port.
//!
//! The DUPs follow the sliced spec (`slice/spec/bot/...`): only what ran on the gold prompts.

use crate::data;
use crate::py::random::Random;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
// Python shims

/// Python `a or b` on strings: `a` unless it is empty.
pub fn or<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.is_empty() { b } else { a }
}

/// Python `a or b or c` on strings.
pub fn or3<'a>(a: &'a str, b: &'a str, c: &'a str) -> &'a str {
    or(or(a, b), c)
}

/// An optional string field as Python's `x or ""`.
pub fn opt(s: &Option<String>) -> &str {
    s.as_deref().unwrap_or("")
}

/// Python `repr(s)` of a str.
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
            c if !py_printable(c) => {
                let n = c as u32;
                if n < 0x100 {
                    out.push_str(&format!("\\x{n:02x}"));
                } else if n < 0x10000 {
                    out.push_str(&format!("\\u{n:04x}"));
                } else {
                    out.push_str(&format!("\\U{n:08x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// Python's `str.isprintable()` for one character, closely enough for the words of a line:
/// controls, separators other than the space, and the common format characters are not.
fn py_printable(c: char) -> bool {
    if c == ' ' {
        return true;
    }
    if c.is_control() || c.is_whitespace() {
        return false;
    }
    !matches!(c as u32,
        0xad | 0x600..=0x605 | 0x61c | 0x6dd | 0x70f | 0x180e | 0x200b..=0x200f | 0x202a..=0x202e
        | 0x2060..=0x2064 | 0x2066..=0x206f | 0xfeff | 0xfff9..=0xfffb | 0xe000..=0xf8ff)
}

/// Python `str(list_of_str)`, e.g. `['a', "don't"]`.
pub fn py_repr_list<S: AsRef<str>>(items: &[S]) -> String {
    let parts: Vec<String> = items.iter().map(|s| py_repr_str(s.as_ref())).collect();
    format!("[{}]", parts.join(", "))
}

/// `" ".join(words)`.
pub fn join(words: &[String]) -> String {
    words.join(" ")
}

/// `any(phrase in text for phrase in phrases)`.
pub fn any_in<S: AsRef<str>>(text: &str, phrases: &[S]) -> bool {
    phrases.iter().any(|p| text.contains(p.as_ref()))
}

/// A tuple / list of strings from the bundle, by its Python name.
pub fn strs(name: &str) -> Vec<String> {
    data::strs(name)
}

/// A compiled regex global from the bundle, by its Python name.
pub fn re(name: &str) -> &'static Re {
    data::re(name)
}

// ---------------------------------------------------------------------------------------------
// Calls into code other workstreams own

/// Run another module's registered reply by its Python id ("module:qualname:line"), for a plain
/// function (no captured values). "" when it declines, fails, or has no port yet (as
/// `registry::reply`). Use it for a reply/guard another workstream ports (e.g.
/// "bot.knowledge.meals:suggest:1997") instead of duplicating it.
pub fn call_reply(id: &str, memory: &mut crate::state::Memory, u: &Understood) -> String {
    let f = crate::data::FnRef { id: id.to_string(), cells: Vec::new(), defaults: Vec::new() };
    crate::knowledge::registry::reply(&f, memory, u)
}

/// Run another module's registered guard by its Python id (plain function, no captured values).
pub fn call_guard(id: &str, u: &Understood) -> bool {
    let f = crate::data::FnRef { id: id.to_string(), cells: Vec::new(), defaults: Vec::new() };
    crate::knowledge::registry::guard(&f, u)
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/lookup.py


// was DUP(bot.knowledge.lookup::the_line): now the real port
pub use crate::knowledge::lookup::the_line;

/// `lookup.said_text(understood)` (the real port).
pub fn said_text(u: &Understood) -> Option<String> {
    Some(crate::knowledge::lookup::said_text(u))
}

// was DUP(bot.knowledge.lookup::tokens): now the real port
pub use crate::knowledge::lookup::tokens;

// was DUP(bot.knowledge.lookup::asks_where): now the real port
pub use crate::knowledge::lookup::asks_where;

// was DUP(bot.knowledge.lookup::asks_where_a_shop_answers): now the real port
pub use crate::knowledge::lookup::asks_where_a_shop_answers;

// was DUP(bot.knowledge.lookup::a_thing_has_stopped): now the real port
pub use crate::knowledge::lookup::a_thing_has_stopped;

// was DUP(bot.knowledge.lookup::the_thing_has_gone): now the real port
pub use crate::knowledge::lookup::the_thing_has_gone;

// was DUP(bot.knowledge.lookup::food_has_gone_off): now the real port
pub use crate::knowledge::lookup::food_has_gone_off;


// was DUP(bot.knowledge.lookup::a_price_not_a_loan): now the real port
pub use crate::knowledge::lookup::a_price_not_a_loan;

// was DUP(bot.knowledge.lookup::not_the_heat_of_an_argument): now the real port
pub use crate::knowledge::lookup::not_the_heat_of_an_argument;

// was DUP(bot.knowledge.lookup::not_the_flat_you_live_in): now the real port
pub use crate::knowledge::lookup::not_the_flat_you_live_in;

// was DUP(bot.knowledge.lookup::not_the_flat_of_something_made): now the real port
pub use crate::knowledge::lookup::not_the_flat_of_something_made;

// was DUP(bot.knowledge.lookup::not_keeping_an_eye_on_it): now the real port
pub use crate::knowledge::lookup::not_keeping_an_eye_on_it;

// was DUP(bot.knowledge.lookup::saving_for_a_trip): now the real port
pub use crate::knowledge::lookup::saving_for_a_trip;

// was DUP(bot.knowledge.lookup::a_food_is_being_kept): now the real port
pub use crate::knowledge::lookup::a_food_is_being_kept;

// was DUP(bot.knowledge.lookup::a_weight): now the real port
pub use crate::knowledge::lookup::a_weight;

// was DUP(bot.knowledge.lookup::says_it_has_gone_off): now the real port
pub use crate::knowledge::lookup::says_it_has_gone_off;

// ---------------------------------------------------------------------------------------------
// bot/compose

static OPENS_ON_A_FRAGMENT_QUESTION: LazyLock<&'static Re> =
    LazyLock::new(|| data::re("bot.compose.OPENS_ON_A_FRAGMENT_QUESTION"));
static NOT_A_FRAGMENT: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose._NOT_A_FRAGMENT"));
static PERSON_WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose._PERSON_WORD"));
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new("[a-z']+", 0));

// DUP(bot.compose::_asks_about_a_thing_nobody_said)
fn asks_about_a_thing_nobody_said(option: &str, last: Option<&str>, said: &str) -> bool {
    if let Some(l) = last {
        if text::strip(l).ends_with('?') {
            return false;
        }
    }
    let Some(m) = OPENS_ON_A_FRAGMENT_QUESTION.match_(option) else { return false };
    let fragment = m.str(1).to_lowercase();
    let words: HashSet<String> = LETTERS.findall_str(&fragment).into_iter().filter(|w| w.chars().count() > 2).collect();
    if words.is_empty() || words.iter().any(|w| NOT_A_FRAGMENT.contains(w)) {
        return false;
    }
    let heard: HashSet<String> =
        LETTERS.findall_str(&format!("{} {}", last.unwrap_or(""), said).to_lowercase()).into_iter().collect();
    !words.iter().any(|w| heard.contains(w))
}

// DUP(bot.compose::said_again)
pub fn said_again(text: &str) -> String {
    PERSON_WORD.sub("@", &text::squash(text).to_lowercase())
}

// was DUP(bot.memory.history::brings_nothing_of_its_own): now the real port
pub use crate::memory::history::brings_nothing_of_its_own;

// was DUP(bot.memory.history::an_earlier_turn): now the real port
pub use crate::memory::history::an_earlier_turn;

/// `memory.history.a_past_that_is_not_there(reply, memory, said)` (the real port).
pub fn a_past_that_is_not_there(reply: &str, memory: &Memory, said: Option<&str>) -> bool {
    crate::memory::history::a_past_that_is_not_there(reply, Some(memory), said)
}

// DUP(bot.compose::pick)
/// One of the options, never the reply just given, preferring one the bot has not said lately;
/// seeded by the turn count and the person's line.
pub fn pick_in<S: AsRef<str> + Clone>(options: &[S], memory: &Memory, seed_text: &str) -> S {
    let turns = &memory.turns;
    let last: Option<&str> = turns.last().map(|t| t.bot.as_str());
    let standing: Vec<S> =
        options.iter().filter(|o| !asks_about_a_thing_nobody_said(o.as_ref(), last, seed_text)).cloned().collect();
    let mut options: Vec<S> = if !standing.is_empty() && standing.len() < options.len() { standing } else { options.to_vec() };
    if !an_earlier_turn(memory, "") {
        let backed: Vec<S> = options.iter().filter(|o| !a_past_that_is_not_there(o.as_ref(), memory, None)).cloned().collect();
        if !backed.is_empty() {
            options = backed;
        }
    }
    let mut fresh: Vec<S> = options.iter().filter(|o| Some(o.as_ref()) != last).cloned().collect();
    if fresh.is_empty() {
        fresh = options.clone();
    }
    let mut said: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for (position, turn) in turns.iter().enumerate() {
        said.entry(said_again(&turn.bot)).or_insert(position as i64);
    }
    let unsaid: Vec<S> = fresh.iter().filter(|o| !said.contains_key(&said_again(o.as_ref()))).cloned().collect();
    if !unsaid.is_empty() {
        fresh = unsaid;
    } else {
        let age = |o: &S| *said.get(&said_again(o.as_ref())).unwrap_or(&-1);
        let oldest = fresh.iter().map(age).min().unwrap_or(-1);
        fresh = fresh.iter().filter(|o| age(o) == oldest).cloned().collect();
    }
    let seed: i128 = if !seed_text.is_empty() {
        turns.len() as i128 * 7919 + crate::py::random::ord_sum(seed_text)
    } else {
        turns.len() as i128
    };
    Random::new(seed).choice(&fresh).clone()
}

/// `pick(options, memory, seed_text)` for a single-turn memory with `turns` earlier turns and no
/// history to read. Prefer [`pick_in`], which reads the conversation.
pub fn pick<S: AsRef<str> + Clone>(options: &[S], turns: usize, seed_text: &str) -> S {
    let mut memory = Memory::new();
    memory.turns = vec![Default::default(); turns];
    pick_in(options, &memory, seed_text)
}

static PET_OR_PROFESSIONAL: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.PET_OR_PROFESSIONAL"));
static PHYSICAL_HARM: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.PHYSICAL_HARM"));
static GRIEF: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair._GRIEF"));
static A_BABY: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair._A_BABY"));
static REGRET_FIRST: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair._REGRET_FIRST"));
static NOT_A_DEED: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.NOT_A_DEED"));
static NOBODY_TO_REPAIR: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.NOBODY_TO_REPAIR"));
static AMENDS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.AMENDS"));
static LAPSES: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| deed_table("bot.compose.repair.LAPSES"));
static DEEDS: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| deed_table("bot.compose.repair.DEEDS"));

fn deed_table(name: &str) -> Vec<(String, &'static Re)> {
    data::get(name)
        .items()
        .iter()
        .map(|row| {
            let pair = row.items();
            (pair[0].str().to_string(), pair[1].as_re().expect("deed pattern"))
        })
        .collect()
}

fn repair_text(clean: &str) -> String {
    text::squash(&clean.to_lowercase().replace('\u{2019}', "'"))
}

// DUP(bot.compose.repair::deed_of)
/// The deed the person reports doing to somebody, or None.
pub fn deed_of(clean: &str) -> Option<String> {
    let text = repair_text(clean);
    if text.is_empty() {
        return None;
    }
    if PET_OR_PROFESSIONAL.is_match(&text) || PHYSICAL_HARM.is_match(&text) {
        return None;
    }
    if GRIEF.is_match(&text) || A_BABY.is_match(&text) || REGRET_FIRST.is_match(&text) {
        return None;
    }
    if let Some((deed, _)) = LAPSES.iter().find(|(_, p)| p.is_match(&text)) {
        return Some(deed.clone());
    }
    if NOT_A_DEED.is_match(&text) || NOBODY_TO_REPAIR.is_match(&text) {
        return None;
    }
    DEEDS.iter().find(|(_, p)| p.is_match(&text)).map(|(d, _)| d.clone())
}

// DUP(bot.compose.repair::owns_the_deed)
pub fn owns_the_deed(clean: &str) -> bool {
    deed_of(clean).is_some()
}

/// `owns_the_deed(reading)` for a reading: its clean (or text) line.
pub fn owns_the_deed_reading(u: &Understood) -> bool {
    owns_the_deed(or(&u.clean, &u.text))
}

// DUP(bot.compose.repair::line_of)
/// The line as the reading holds it: clean, else text (None stands for "" here; the words of a
/// reading with neither are empty too).
pub fn line_of(u: &Understood) -> Option<&str> {
    if !u.clean.is_empty() {
        return Some(&u.clean);
    }
    if !u.text.is_empty() {
        return Some(&u.text);
    }
    None
}

// DUP(bot.compose.repair::amends_to)
/// Whether the line asks how to make amends to somebody, without naming a deed.
pub fn amends_to(clean: &str) -> bool {
    let text = repair_text(clean);
    if text.is_empty() || !AMENDS.is_match(&text) {
        return false;
    }
    !(PET_OR_PROFESSIONAL.is_match(&text) || PHYSICAL_HARM.is_match(&text))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reprs() {
        assert_eq!(py_repr_list(&["a", "don't", "x\"y'"]), "['a', \"don't\", 'x\"y\\'']");
        assert_eq!(py_repr_str("a\u{200b}b\u{e9}\u{a0}"), "'a\\u200bb\u{e9}\\xa0'");
    }
}
