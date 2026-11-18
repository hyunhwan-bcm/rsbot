//! Helpers the D2 knowledge modules share: small Python-behaviour shims, and private ports of
//! helpers that live in modules other workstreams own (`lookup.py`, `compose`). Each of those is
//! marked `// DUP(<python module>::<name>)` so the integrator can swap in the owner's port.
//!
//! The DUPs follow the sliced spec (`slice/spec/bot/...`): only what ran on the gold prompts.

use crate::data;
use crate::py::random::Random;
use crate::py::re::Re;
use crate::py::text;
use crate::understood::Understood;
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
            c if (c as u32) < 0x20 || c as u32 == 0x7f => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push(quote);
    out
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

// NEEDS(U: bot.understanding::understand)
/// `understanding.understand(text)`: until U's engine is merged, PyBot's recorded reading when
/// RSBOT_ORACLE_UNDERSTAND is set, else a bare reading of the text.
pub fn understand(text: &str) -> Understood {
    if crate::oracle::use_oracle_understand() {
        if let Some(u) = crate::oracle::UNDERSTAND_CALLS.get(text) {
            return u.clone();
        }
    }
    Understood { text: text.to_string(), clean: text.to_lowercase(), ..Default::default() }
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/lookup.py

static WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup.WORD"));
static WHERE_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.WHERE_PHRASES"));
static STOPPED_WORKING: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.STOPPED_WORKING"));
static HAS_GONE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup.HAS_GONE"));
static FOOD_NAMED: LazyLock<std::collections::HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.FOOD_NAMED"));
static BEFORE_OFF: LazyLock<std::collections::HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.BEFORE_OFF"));
static BORROWING_IN_THE_LINE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._BORROWING_IN_THE_LINE"));
static OFF_A_PERSON: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._OFF_A_PERSON"));
static THE_HEAT_OF_AN_ARGUMENT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._THE_HEAT_OF_AN_ARGUMENT"));
static FLAT_YOU_LIVE_IN: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._FLAT_YOU_LIVE_IN"));
static KEEPING_AN_EYE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._KEEPING_AN_EYE"));
static SAVING_VERB: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._SAVING_VERB"));
static SAVING_FOR_A_TRIP: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._SAVING_FOR_A_TRIP"));
static KEEPING_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.KEEPING_WORDS"));
static GONE_OFF_SIGNS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.GONE_OFF_SIGNS"));
static FLAT_WORD: LazyLock<Re> = LazyLock::new(|| Re::new("\\bflat\\b", crate::py::re::I));

// DUP(bot.knowledge.lookup::the_line)
/// The line as one lower-case string: clean, normalised and plain readings side by side.
pub fn the_line(u: &Understood) -> String {
    format!("{} {} {}", u.clean, u.norm_text, u.plain).to_lowercase()
}

// DUP(bot.knowledge.lookup::said_text)
/// The words of the line (or its normalised words) joined; None when neither is there.
pub fn said_text(u: &Understood) -> Option<String> {
    if !u.words.is_empty() {
        return Some(u.words.join(" "));
    }
    if !u.norm.is_empty() {
        return Some(u.norm.join(" "));
    }
    None
}

// DUP(bot.knowledge.lookup::tokens)
/// The words of a text, lower-cased, curly apostrophes made straight.
pub fn tokens(text: &str) -> Vec<String> {
    let straight = text::translate(text, &[('\u{2019}', "'"), ('\u{2018}', "'"), ('\u{2bc}', "'")]);
    WORD.findall_str(&straight.to_lowercase())
}

// DUP(bot.knowledge.lookup::asks_where)
/// Whether the line asks where to buy or get something.
pub fn asks_where(u: &Understood) -> bool {
    let text = format!(" {} ", u.words.join(" ").to_lowercase());
    any_in(&text, &WHERE_PHRASES)
}

// DUP(bot.knowledge.lookup::asks_where_a_shop_answers)
/// Spec: only the "not a where-ask" branch ran on gold (falls through to None).
pub fn asks_where_a_shop_answers(u: &Understood) -> bool {
    if !asks_where(u) {
        return false;
    }
    false
}

// DUP(bot.knowledge.lookup::a_thing_has_stopped)
/// Spec: only the "no stopped-working phrase" branch ran on gold.
pub fn a_thing_has_stopped(u: &Understood) -> bool {
    let words = if !u.norm.is_empty() { u.norm.join(" ") } else { u.words.join(" ") };
    let line = format!(" {} ", or(&words, or(&u.clean, &u.text)).to_lowercase());
    let line = line.replace(" | ", " ").replace('|', " ");
    if !STOPPED_WORKING.iter().any(|p| line.contains(&format!(" {p} "))) {
        return false;
    }
    false
}

// DUP(bot.knowledge.lookup::the_thing_has_gone)
/// Spec: no tail after "gone" ever counted on gold.
pub fn the_thing_has_gone(u: &Understood) -> bool {
    let line = format!(" {} ", u.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" "));
    let _ = HAS_GONE.finditer(&line);
    false
}

// DUP(bot.knowledge.lookup::food_has_gone_off)
/// Spec: never true on gold.
pub fn food_has_gone_off(u: &Understood) -> bool {
    let text = or3(&u.plain, &u.clean, &u.text).to_lowercase();
    let words = tokens(&text);
    if words.is_empty() || !words.iter().any(|w| FOOD_NAMED.contains(w)) {
        return false;
    }
    let _ = &*BEFORE_OFF;
    false
}

// DUP(bot.knowledge.lookup::a_price_not_a_loan)
/// Whether a price line is not about borrowing or money owed by a person.
pub fn a_price_not_a_loan(u: &Understood) -> bool {
    let text = or(&u.text, &u.clean);
    !(BORROWING_IN_THE_LINE.is_match(text) || OFF_A_PERSON.is_match(text))
}

// DUP(bot.knowledge.lookup::not_the_heat_of_an_argument)
pub fn not_the_heat_of_an_argument(u: &Understood) -> bool {
    !THE_HEAT_OF_AN_ARGUMENT.is_match(or(&u.text, &u.clean))
}

// DUP(bot.knowledge.lookup::not_the_flat_you_live_in)
pub fn not_the_flat_you_live_in(u: &Understood) -> bool {
    !FLAT_YOU_LIVE_IN.is_match(or(&u.text, &u.clean))
}

// DUP(bot.knowledge.lookup::not_the_flat_of_something_made)
/// Spec: only the "no 'flat' in the line" branch ran on gold.
pub fn not_the_flat_of_something_made(u: &Understood) -> bool {
    !FLAT_WORD.is_match(or(&u.text, &u.clean))
}

// DUP(bot.knowledge.lookup::not_keeping_an_eye_on_it)
pub fn not_keeping_an_eye_on_it(u: &Understood) -> bool {
    !KEEPING_AN_EYE.is_match(or(&u.text, &u.clean))
}

// DUP(bot.knowledge.lookup::saving_for_a_trip)
pub fn saving_for_a_trip(u: &Understood) -> bool {
    let line = or(&u.clean, &u.text);
    !(SAVING_VERB.is_match(line) && SAVING_FOR_A_TRIP.is_match(line))
}

// DUP(bot.knowledge.lookup::a_food_is_being_kept)
/// Spec: only the "no keeping word" branch ran on gold.
pub fn a_food_is_being_kept(u: &Understood) -> bool {
    let text = format!(" {} ", or(&u.norm_text, &u.clean).to_lowercase());
    if !any_in(&text, &KEEPING_WORDS) {
        return false;
    }
    false
}

// DUP(bot.knowledge.lookup::a_weight)
/// Spec: never true on gold.
pub fn a_weight(_u: &Understood) -> bool {
    false
}

// DUP(bot.knowledge.lookup::says_it_has_gone_off)
pub fn says_it_has_gone_off(clean: &str) -> bool {
    let low = format!(" {} ", clean.to_lowercase());
    GONE_OFF_SIGNS.iter().any(|s| low.contains(&format!(" {s} ")) || low.contains(&format!(" {s}")))
}

// ---------------------------------------------------------------------------------------------
// bot/compose

static OPENS_ON_A_FRAGMENT_QUESTION: LazyLock<&'static Re> =
    LazyLock::new(|| data::re("bot.compose.OPENS_ON_A_FRAGMENT_QUESTION"));
static PERSON_WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose._PERSON_WORD"));
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new("[a-z']+", 0));

fn asks_about_a_thing_nobody_said(option: &str, last: Option<&str>, said: &str) -> bool {
    let Some(m) = OPENS_ON_A_FRAGMENT_QUESTION.match_(option) else { return false };
    let fragment = m.str(1).to_lowercase();
    let words: Vec<String> = LETTERS.findall_str(&fragment).into_iter().filter(|w| w.chars().count() > 2).collect();
    let heard: std::collections::HashSet<String> =
        LETTERS.findall_str(&format!("{} {}", last.unwrap_or(""), said).to_lowercase()).into_iter().collect();
    !words.iter().any(|w| heard.contains(w))
}

// DUP(bot.compose::said_again)
pub fn said_again(text: &str) -> String {
    PERSON_WORD.sub("@", &text::squash(text).to_lowercase())
}

// DUP(bot.compose::pick)
/// `pick(options, memory, seed_text)` on a single turn (no earlier turns to avoid repeating):
/// a seeded choice among the options.
pub fn pick<S: AsRef<str> + Clone>(options: &[S], turns: usize, seed_text: &str) -> S {
    let standing: Vec<S> =
        options.iter().filter(|o| !asks_about_a_thing_nobody_said(o.as_ref(), None, seed_text)).cloned().collect();
    let options: Vec<S> = if !standing.is_empty() && standing.len() < options.len() { standing } else { options.to_vec() };
    let seed: i128 = if !seed_text.is_empty() {
        turns as i128 * 7919 + crate::py::random::ord_sum(seed_text)
    } else {
        turns as i128
    };
    Random::new(seed).choice(&options).clone()
}

static PET_OR_PROFESSIONAL: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.PET_OR_PROFESSIONAL"));
static PHYSICAL_HARM: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.PHYSICAL_HARM"));
static A_BABY: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair._A_BABY"));
static AMENDS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.AMENDS"));

// DUP(bot.compose.repair::deed_of)
/// Spec: no lapse or deed pattern ever matched on gold, so it is always None.
pub fn deed_of(clean: &str) -> Option<String> {
    let text = text::squash(&clean.to_lowercase().replace('\u{2019}', "'"));
    if PET_OR_PROFESSIONAL.is_match(&text) || PHYSICAL_HARM.is_match(&text) {
        return None;
    }
    if A_BABY.is_match(&text) {
        return None;
    }
    None
}

// DUP(bot.compose.repair::owns_the_deed)
pub fn owns_the_deed(clean: &str) -> bool {
    deed_of(clean).is_some()
}

/// `deed_of(reading)` for a reading dict: its clean (or text) line.
pub fn owns_the_deed_reading(u: &Understood) -> bool {
    owns_the_deed(or(&u.clean, &u.text))
}

// DUP(bot.compose.repair::line_of)
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
/// Spec: only the "no amends" branch ran on gold.
pub fn amends_to(clean: &str) -> bool {
    let text = text::squash(&clean.to_lowercase().replace('\u{2019}', "'"));
    if text.is_empty() || !AMENDS.is_match(&text) {
        return false;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reprs() {
        assert_eq!(py_repr_list(&["a", "don't", "x\"y'"]), "['a', \"don't\", 'x\"y\\'']");
    }
}
