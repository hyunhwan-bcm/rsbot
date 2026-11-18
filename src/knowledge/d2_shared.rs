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
static FOOD_NAMED: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.FOOD_NAMED"));
static BEFORE_OFF: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.BEFORE_OFF"));
static BORROWING_IN_THE_LINE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._BORROWING_IN_THE_LINE"));
static OFF_A_PERSON: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._OFF_A_PERSON"));
static THE_HEAT_OF_AN_ARGUMENT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._THE_HEAT_OF_AN_ARGUMENT"));
static FLAT_YOU_LIVE_IN: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._FLAT_YOU_LIVE_IN"));
static KEEPING_AN_EYE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._KEEPING_AN_EYE"));
static SAVING_VERB: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._SAVING_VERB"));
static SAVING_FOR_A_TRIP: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._SAVING_FOR_A_TRIP"));
static KEEPING_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.KEEPING_WORDS"));
static GONE_OFF_SIGNS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.GONE_OFF_SIGNS"));
static WHERE_PART_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.WHERE_PART_WORDS"));
static PRICE_COMPARED: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.PRICE_COMPARED"));
static GONE_INTENSIFIER: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.GONE_INTENSIFIER"));
static GONE_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.GONE_STATE"));
static OFF_STATE_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.OFF_STATE_ASKS"));
static BAD_IS_A_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup._BAD_IS_A_STATE"));
static OFF_SAID_OF_FOOD: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.OFF_SAID_OF_FOOD"));
static SMELL_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.SMELL_VERBS"));
static A_GOOD_SMELL: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.A_GOOD_SMELL"));
static A_BAKE_OR_A_PAN: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._A_BAKE_OR_A_PAN"));
static CAME_OUT_FLAT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._CAME_OUT_FLAT"));
static THE_GROUND_UNDERFOOT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._THE_GROUND_UNDERFOOT"));
static AFFORD_THE_TRIP: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._AFFORD_THE_TRIP"));
static KEPT_FOODS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.KEPT_FOODS"));
static WEIGHT_UNITS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.WEIGHT_UNITS"));
static FLAT_WORD: LazyLock<Re> = LazyLock::new(|| Re::new("\\bflat\\b", crate::py::re::I));

// DUP(bot.knowledge.lookup::the_line)
/// The line as one lower-case string: clean, normalised and plain readings side by side.
pub fn the_line(u: &Understood) -> String {
    format!("{} {} {}", u.clean, u.norm_text, u.plain).to_lowercase()
}

// DUP(bot.knowledge.lookup::said_text)
/// The line's own words as one string, from whichever reading has them ("" when neither).
/// (The Python returns None only for a non-dict reading; a reading here is always a dict.)
pub fn said_text(u: &Understood) -> Option<String> {
    if !u.words.is_empty() {
        return Some(u.words.join(" "));
    }
    if !u.norm.is_empty() {
        return Some(u.norm.join(" "));
    }
    Some(String::new())
}

// DUP(bot.knowledge.lookup::tokens)
/// The words of a text, lower-cased, curly apostrophes made straight.
pub fn tokens(text: &str) -> Vec<String> {
    let straight = text::translate(text, &[('\u{2019}', "'"), ('\u{2018}', "'"), ('\u{2bc}', "'")]);
    WORD.findall_str(&straight.to_lowercase())
}

// DUP(bot.knowledge.lookup::asks_where)
/// Whether the line asks where to get something, which is a place and not a verdict.
pub fn asks_where(u: &Understood) -> bool {
    if u.asks.iter().any(|a| a == "where") {
        return true;
    }
    let text = format!(" {} ", u.words.join(" ").to_lowercase());
    any_in(&text, &WHERE_PHRASES)
}

// DUP(bot.knowledge.lookup::asks_where_a_shop_answers)
/// Whether the line asks where to get one of the things the price answer is written for.
pub fn asks_where_a_shop_answers(u: &Understood) -> bool {
    if !asks_where(u) {
        return false;
    }
    let words: Vec<String> = u.words.iter().map(|w| w.to_lowercase()).collect();
    if words.iter().any(|w| WHERE_PART_WORDS.contains(w)) {
        return false;
    }
    let text = format!(" {} ", words.join(" "));
    if any_in(&text, &PRICE_COMPARED) {
        return true;
    }
    let reference = text::strip(opt(&u.reference.it)).to_lowercase();
    if reference.is_empty() {
        return false;
    }
    PRICE_COMPARED.iter().any(|t| reference == *t || reference.ends_with(&format!(" {t}")))
}

// DUP(bot.knowledge.lookup::a_thing_has_stopped)
/// Whether what stopped working is a thing in the line, not the person saying it.
pub fn a_thing_has_stopped(u: &Understood) -> bool {
    let words = if !u.norm.is_empty() { u.norm.join(" ") } else { u.words.join(" ") };
    let line = format!(" {} ", or(&words, or(&u.clean, &u.text)).to_lowercase());
    let line = line.replace(" | ", " ").replace('|', " ");
    if !STOPPED_WORKING.iter().any(|p| line.contains(&format!(" {p} "))) {
        return false;
    }
    let things: Vec<String> =
        u.entities.iter().filter(|e| e.kind == "thing").map(|e| e.text.to_lowercase()).collect();
    if things.is_empty() {
        return false;
    }
    let subject = opt(&u.subject).to_lowercase();
    if subject.is_empty() {
        return true;
    }
    things.iter().any(|t| !t.is_empty() && (subject.contains(t.as_str()) || t.contains(subject.as_str())))
}

// DUP(bot.knowledge.lookup::the_thing_has_gone)
/// Whether "gone"/"went" is what the thing did, rather than the state it went into.
pub fn the_thing_has_gone(u: &Understood) -> bool {
    let said: Vec<String> = u.words.iter().map(|w| w.to_lowercase()).collect();
    let line = if said.is_empty() {
        format!(" {} ", or(&u.clean, &u.text).to_lowercase())
    } else {
        format!(" {} ", said.join(" "))
    };
    for found in HAS_GONE.finditer(&line) {
        let tail = found.str(1);
        let mut words: Vec<&str> =
            text::split_ref(tail).into_iter().map(|w| text::strip_chars(w, "'.,!?")).filter(|w| !w.is_empty()).collect();
        while !words.is_empty() && GONE_INTENSIFIER.contains(words[0]) {
            words.remove(0);
        }
        if words.is_empty() || !GONE_STATE.contains(words[0]) {
            return true;
        }
    }
    false
}

// DUP(bot.knowledge.lookup::food_has_gone_off)
/// Whether the line says the food it names has gone off.
pub fn food_has_gone_off(u: &Understood) -> bool {
    let text = or3(&u.plain, &u.clean, &u.text).to_lowercase();
    let words = tokens(&text);
    if words.is_empty() || !words.iter().any(|w| FOOD_NAMED.contains(w)) {
        return false;
    }
    if OFF_STATE_ASKS.iter().any(|p| text.contains(p.as_str())) {
        return true;
    }
    for (pos, word) in words.iter().enumerate() {
        if !FOOD_NAMED.contains(word) {
            continue;
        }
        let mut at = pos + 1;
        while at < words.len() && BEFORE_OFF.contains(&words[at]) {
            at += 1;
        }
        if at >= words.len() {
            continue;
        }
        let said = &words[at];
        if said == "bad" {
            let between = &words[pos + 1..at + 1];
            if !between.iter().any(|w| BAD_IS_A_STATE.contains(w)) {
                continue;
            }
        }
        if OFF_SAID_OF_FOOD.contains(said) {
            return true;
        }
        if SMELL_VERBS.contains(said) {
            match words.get(at + 1) {
                None => return true,
                Some(after) if !A_GOOD_SMELL.contains(after) => return true,
                _ => {}
            }
        }
    }
    false
}

/// `text or clean`, else the words joined: the line the flat/heat/eye guards read.
fn text_or_words(u: &Understood) -> String {
    let said = or(&u.text, &u.clean);
    if said.is_empty() { u.words.join(" ") } else { said.to_string() }
}

// DUP(bot.knowledge.lookup::a_price_not_a_loan)
/// Whether a price row may answer: the line is a discount and not money off a person.
pub fn a_price_not_a_loan(u: &Understood) -> bool {
    let text = text_or_words(u);
    if text.is_empty() {
        return true;
    }
    !(BORROWING_IN_THE_LINE.is_match(&text) || OFF_A_PERSON.is_match(&text))
}

// DUP(bot.knowledge.lookup::not_the_heat_of_an_argument)
/// True unless the line's "heated" is people or a row rather than a room or a night.
pub fn not_the_heat_of_an_argument(u: &Understood) -> bool {
    !THE_HEAT_OF_AN_ARGUMENT.is_match(&text_or_words(u))
}

// DUP(bot.knowledge.lookup::not_the_flat_you_live_in)
/// True unless the line's "flat" is the home somebody lives in.
pub fn not_the_flat_you_live_in(u: &Understood) -> bool {
    !FLAT_YOU_LIVE_IN.is_match(&text_or_words(u))
}

// DUP(bot.knowledge.lookup::not_the_flat_of_something_made)
/// True unless the line's "flat" is how a thing somebody made turned out.
pub fn not_the_flat_of_something_made(u: &Understood) -> bool {
    let said = text_or_words(u);
    if !FLAT_WORD.is_match(&said) {
        return true;
    }
    if A_BAKE_OR_A_PAN.is_match(&said) {
        return false;
    }
    if CAME_OUT_FLAT.is_match(&said) && !THE_GROUND_UNDERFOOT.is_match(&said) {
        return false;
    }
    true
}

// DUP(bot.knowledge.lookup::not_keeping_an_eye_on_it)
/// True unless the line is about keeping an eye on something.
pub fn not_keeping_an_eye_on_it(u: &Understood) -> bool {
    !KEEPING_AN_EYE.is_match(&text_or_words(u))
}

// DUP(bot.knowledge.lookup::saving_for_a_trip)
/// False when the line is saving up for (or affording) a trip rather than a price question.
pub fn saving_for_a_trip(u: &Understood) -> bool {
    let line = or(&u.clean, &u.text);
    if AFFORD_THE_TRIP.is_match(line) {
        return false;
    }
    !(SAVING_VERB.is_match(line) && SAVING_FOR_A_TRIP.is_match(line))
}

// DUP(bot.knowledge.lookup::a_food_is_being_kept)
/// Whether the line asks about keeping a food: it names a food and a keeping word.
pub fn a_food_is_being_kept(u: &Understood) -> bool {
    let text = format!(" {} ", or(&u.norm_text, &u.clean).to_lowercase());
    if !any_in(&text, &KEEPING_WORDS) {
        return false;
    }
    let said: std::collections::HashSet<String> = u.words.iter().map(|w| w.to_lowercase()).collect();
    for food in KEPT_FOODS.iter() {
        if food.contains(' ') {
            if text.contains(food.as_str()) {
                return true;
            }
        } else if said.contains(food) || text.contains(&format!(" {food} ")) {
            return true;
        }
    }
    false
}

// DUP(bot.knowledge.lookup::a_weight)
/// Whether the line gives a weight in kilos, grams or pounds.
pub fn a_weight(u: &Understood) -> bool {
    u.numbers.iter().any(|n| WEIGHT_UNITS.contains(&opt(&n.unit).to_lowercase()))
}

// DUP(bot.knowledge.lookup::says_it_has_gone_off)
/// Whether the line says the food has gone off, or asks whether it has.
pub fn says_it_has_gone_off(clean: &str) -> bool {
    let low = format!(" {} ", clean.to_lowercase());
    GONE_OFF_SIGNS.iter().any(|s| low.contains(&format!(" {s} ")) || low.contains(&format!(" {s}")))
}

// ---------------------------------------------------------------------------------------------
// bot/compose

static OPENS_ON_A_FRAGMENT_QUESTION: LazyLock<&'static Re> =
    LazyLock::new(|| data::re("bot.compose.OPENS_ON_A_FRAGMENT_QUESTION"));
static NOT_A_FRAGMENT: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose._NOT_A_FRAGMENT"));
static PERSON_WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose._PERSON_WORD"));
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new("[a-z']+", 0));
static LETTERS_DIGITS: LazyLock<Re> = LazyLock::new(|| Re::new("[a-z0-9']+", 0));
static CLAIMS_A_PAST: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.history.CLAIMS_A_PAST"));
static GREETS_A_RETURN: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.history.GREETS_A_RETURN"));
static NO_CONTENT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.history.NO_CONTENT_WORDS"));
static ANSWERS_AN_EARLIER_TURN: LazyLock<Vec<String>> =
    LazyLock::new(|| data::strs("bot.memory.history.ANSWERS_AN_EARLIER_TURN"));

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

// DUP(bot.memory.history::brings_nothing_of_its_own)
pub fn brings_nothing_of_its_own(text: &str) -> bool {
    let words = LETTERS_DIGITS.findall_str(&text.to_lowercase());
    words.is_empty() || words.iter().all(|w| NO_CONTENT_WORDS.contains(w))
}

// DUP(bot.memory.history::an_earlier_turn)
/// Whether the conversation has a turn of theirs before this line (`text`).
pub fn an_earlier_turn(memory: &Memory, text: &str) -> bool {
    let mine = text::squash(text).to_lowercase();
    memory.turns.iter().any(|row| {
        let line = text::squash(&row.user);
        !line.is_empty() && line.to_lowercase() != mine
    })
}

// DUP(bot.memory.history::a_past_that_is_not_there)
/// Whether a reply leans on a conversation that has not happened yet.
pub fn a_past_that_is_not_there(reply: &str, memory: &Memory, said: Option<&str>) -> bool {
    if an_earlier_turn(memory, "") {
        return false;
    }
    let low = text::squash(reply).to_lowercase();
    if low.is_empty() {
        return false;
    }
    if CLAIMS_A_PAST.is_match(&low) || GREETS_A_RETURN.match_(&low).is_some() {
        return true;
    }
    let Some(said) = said else { return false };
    if !brings_nothing_of_its_own(said) {
        return false;
    }
    ANSWERS_AN_EARLIER_TURN.iter().any(|f| low.contains(f.as_str()))
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
