//! Port of `bot/memory/told.py`: whether they already told the bot about a thing, answered from
//! the chat ("have I told you about my sister?", "remember when i said my knee hurt", "i told
//! you already"), and the boundary a person sets on the chat ("i don't want to talk about it").
//!
//! The store's own readers for each kind of fact come in as `facts`, a dict of function
//! references made by `store_replies::_told_readers` and run by `store_replies::run_told_reader`;
//! `plain` is the store's `_respond` and `claims` its `_about_this_chat` with the line in it.

#![allow(non_snake_case)] // ABOUT_THE_CHAT and A_CALLBACK keep their Python names

use super::pyv::{self, OptStr};
use super::{history, others, user};
use crate::data::{self, Value};
use crate::py::re::{self, Match, Re};
use crate::py::text;
use crate::state::Memory;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.memory.told.";
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn words_set(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}
fn pool(name: &str) -> Vec<String> {
    data::strs(&format!("{M}{name}"))
}

static TOLD_ABOUT: LazyLock<&'static Re> = LazyLock::new(|| rx("TOLD_ABOUT"));
static KNOW_ABOUT: LazyLock<&'static Re> = LazyLock::new(|| rx("KNOW_ABOUT"));
static A_STATEMENT: LazyLock<&'static Re> = LazyLock::new(|| rx("A_STATEMENT"));
static NOTHING_NAMED: LazyLock<&'static Re> = LazyLock::new(|| rx("NOTHING_NAMED"));
static POINTING: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("POINTING"));
static IN_QUESTION: LazyLock<Vec<(Re, String)>> = LazyLock::new(|| {
    data::get("bot.memory.told.IN_QUESTION")
        .items()
        .iter()
        .map(|pair| (Re::new(pair.items()[0].str(), re::I | re::U), pair.items()[1].str().to_string()))
        .collect()
});
static A_KIND: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.memory.told.A_KIND").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
});
static CHECK_OF_SOMEONE: LazyLock<&'static Re> = LazyLock::new(|| rx("CHECK_OF_SOMEONE"));
static CORRECTION_WORDS: LazyLock<String> = LazyLock::new(|| data::string("bot.memory.told.CORRECTION_WORDS"));
static CORRECTING: LazyLock<&'static Re> = LazyLock::new(|| rx("CORRECTING"));
static SAID_OF_SOMEONE: LazyLock<&'static Re> = LazyLock::new(|| rx("SAID_OF_SOMEONE"));
static SAID_OF_PRONOUN: LazyLock<&'static Re> = LazyLock::new(|| rx("SAID_OF_PRONOUN"));
static BARE_DENIAL: LazyLock<&'static Re> = LazyLock::new(|| rx("BARE_DENIAL"));
static BARE_DENIAL_PRONOUN: LazyLock<&'static Re> = LazyLock::new(|| rx("BARE_DENIAL_PRONOUN"));
static A_NUMBER: LazyLock<&'static Re> = LazyLock::new(|| rx("A_NUMBER"));
static A_NAME: LazyLock<&'static Re> = LazyLock::new(|| rx("A_NAME"));
static WITH_AN_ARTICLE: LazyLock<&'static Re> = LazyLock::new(|| rx("WITH_AN_ARTICLE"));
static CLAIM_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("CLAIM_FILLER"));
static CHECKED_ELSEWHERE: LazyLock<&'static Re> = LazyLock::new(|| rx("CHECKED_ELSEWHERE"));
static ANIMAL: LazyLock<&'static Re> = LazyLock::new(|| rx("ANIMAL"));
static WHERE_I_LIVE: LazyLock<&'static Re> = LazyLock::new(|| rx("WHERE_I_LIVE"));
static NOT_MINE: LazyLock<&'static Re> = LazyLock::new(|| rx("NOT_MINE"));
static ALREADY: LazyLock<&'static Re> = LazyLock::new(|| rx("ALREADY"));
static ASKS: LazyLock<&'static Re> = LazyLock::new(|| rx("ASKS"));
static REMEMBERED: LazyLock<&'static Re> = LazyLock::new(|| rx("REMEMBERED"));
static CALLBACK_MISSED: LazyLock<Vec<String>> = LazyLock::new(|| pool("CALLBACK_MISSED"));
/// The module's last `A_DENIAL` (Python binds the name twice; the second one is the global).
static A_DENIAL: LazyLock<&'static Re> = LazyLock::new(|| rx("A_DENIAL"));
static CLAIMED_ABOUT: LazyLock<&'static Re> = LazyLock::new(|| rx("CLAIMED_ABOUT"));
static A_CLAIM_WITHOUT_YOU: LazyLock<&'static Re> = LazyLock::new(|| rx("A_CLAIM_WITHOUT_YOU"));
static A_TRAILING_TIME: LazyLock<&'static Re> = LazyLock::new(|| rx("A_TRAILING_TIME"));
static REMEMBER_TELLING: LazyLock<&'static Re> = LazyLock::new(|| rx("REMEMBER_TELLING"));
static A_CLAUSE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_CLAUSE"));
static A_BOUNDARY: LazyLock<&'static Re> = LazyLock::new(|| rx("A_BOUNDARY"));
static A_CLAIM_BEFORE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_CLAIM_BEFORE"));
static LEFT_THERE: LazyLock<Vec<String>> = LazyLock::new(|| pool("LEFT_THERE"));
static A_LEADING_TIME: LazyLock<&'static Re> = LazyLock::new(|| rx("A_LEADING_TIME"));
static A_TIME_ONLY: LazyLock<&'static Re> = LazyLock::new(|| rx("A_TIME_ONLY"));
static A_BARE_CLAIM: LazyLock<&'static Re> = LazyLock::new(|| rx("A_BARE_CLAIM"));
static SOMETHING_ABOUT: LazyLock<&'static Re> = LazyLock::new(|| rx("SOMETHING_ABOUT"));
static SAID_AS_IT_IS: LazyLock<&'static Re> = LazyLock::new(|| rx("SAID_AS_IT_IS"));
static LOWERABLE: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("LOWERABLE"));
static A_QUOTE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_QUOTE"));
static A_DATE_PROPERTY: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("A_DATE_PROPERTY"));
static A_PLACE_PROPERTY: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("A_PLACE_PROPERTY"));
static A_CLAUSE_OPENING: LazyLock<&'static Re> = LazyLock::new(|| rx("A_CLAUSE_OPENING"));
static REPORTED: LazyLock<&'static Re> = LazyLock::new(|| rx("REPORTED"));
static TELLING_ALREADY: LazyLock<&'static Re> = LazyLock::new(|| rx("TELLING_ALREADY"));
static MINE: LazyLock<&'static Re> = LazyLock::new(|| rx("MINE"));
static YOURS: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.memory.told.YOURS"));
static A_VERB_IN_A_CLAUSE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_VERB_IN_A_CLAUSE"));
static A_CLAUSE_OF_MINE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_CLAUSE_OF_MINE"));
static CONTRACTIONS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.memory.told.CONTRACTIONS").pairs().iter().map(|(k, v)| (k.strs(), v.str().to_string())).collect()
});
static CALLED_NAME: LazyLock<&'static Re> = LazyLock::new(|| rx("CALLED_NAME"));
static COUNT_WORDS: LazyLock<HashMap<i64, String>> = LazyLock::new(|| {
    data::get("bot.memory.told.COUNT_WORDS").pairs().iter().map(|(k, v)| (k.as_int().unwrap(), v.str().to_string())).collect()
});
static ONLY_A_POINTING_TIME: LazyLock<&'static Re> = LazyLock::new(|| rx("ONLY_A_POINTING_TIME"));
static POINTING_TAIL: LazyLock<&'static Re> = LazyLock::new(|| rx("POINTING_TAIL"));
static POINTING_SAID: LazyLock<&'static Re> = LazyLock::new(|| rx("POINTING_SAID"));
static A_TRIP: LazyLock<&'static Re> = LazyLock::new(|| rx("A_TRIP"));
static CLAIMS_TOLD: LazyLock<&'static Re> = LazyLock::new(|| rx("CLAIMS_TOLD"));
static NOTHING_SAID: LazyLock<&'static Re> = LazyLock::new(|| rx("NOTHING_SAID"));
static THE_KINDS_KEPT: LazyLock<Vec<String>> = LazyLock::new(|| pool("THE_KINDS_KEPT"));
static RELATIONS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.RELATIONS"));
static SHE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.SHE"));
static HE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.HE"));
static BREED_RE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.understanding.breeds.BREED_RE"));
static BREED_KIND: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.understanding.breeds.BREED_KIND"));

static A_LEADING_DETERMINER: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:the|my|our|a|an)\s+", re::U));
static LOWER_LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]+", re::U));
static WORDS_AND_DIGITS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9']+", re::U));
static SPLIT_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[\s,]+", re::U));
static NOT_LETTERS_AZ: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-zA-Z]+", re::U));
static NOT_LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z]+", re::U));
static NON_LETTER: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z]", re::U));
static I_M: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bi\s*m\b", re::I | re::U));
static I_VE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bi\s*ve\b", re::I | re::U));

// ---------------------------------------------------------------------------------------------
// The store's readers, handed in as function references.

/// The reader `facts[key]`, when the dict holds one.
fn has_reader(facts: &Value, key: &str) -> bool {
    facts.get(key).is_some_and(Value::truthy)
}

/// `(facts or {}).get(key) or (lambda memory: "")` run on the memory (`arg` is the animal for
/// "pets", the word for "family").
fn run_reader(facts: &Value, key: &str, memory: &mut Memory, arg: &str) -> String {
    if !has_reader(facts, key) {
        return if key == "family" { arg.to_string() } else { String::new() };
    }
    pyv::str_or_empty(&crate::memory::store_replies::run_told_reader(key, memory, arg))
}

/// `plain(asked)`: the store's answer in its own words.
fn run_plain(plain: &Value, asked: &str, memory: &mut Memory) -> Option<String> {
    match plain.as_fn() {
        Some(f) if f.name() == "bot.memory.store_replies:_respond" => crate::memory::store_replies::_respond(asked, memory),
        Some(f) => panic!("told: no plain reader {}", f.id),
        None => None,
    }
}

/// `claims()`: whether the store reads the line as being about what it keeps.
fn run_claims(claims: &Value, memory: &mut Memory) -> bool {
    match claims.as_fn() {
        Some(f) => {
            let line = f.cell("text").map(|t| t.str().to_string()).unwrap_or_default();
            crate::memory::store_replies::_about_this_chat(&line, memory)
        }
        None => panic!("TypeError: 'NoneType' object is not callable"),
    }
}

/// `pick(options, memory, seed_text=...)`.
fn pick(options: &[String], memory: &Memory, seed: &str) -> String {
    pyv::pick(options, memory, seed)
}

/// `s[0].lower() + s[1:]`.
fn lower_first(s: &str) -> String {
    let mut chars = s.chars();
    chars.next().map(|c| c.to_lowercase().chain(chars).collect()).unwrap_or_default()
}

/// `s[0].upper() + s[1:]`.
fn upper_first(s: &str) -> String {
    text::upper_first(s)
}

// ---------------------------------------------------------------------------------------------

/// Python: bot/memory/told.py:203 `_the_shape(verb, pred)`
/// What a fact is about, in the coarsest words that tell one fact from another.
pub fn _the_shape(verb: &str, pred: &str) -> (String, &'static str) {
    let pred = text::strip(pred);
    let kind = if A_NUMBER.match_(pred).is_some() {
        "number"
    } else if A_NAME.match_(pred).is_some() {
        "called"
    } else if WITH_AN_ARTICLE.match_(pred).is_some() {
        "a"
    } else {
        "said"
    };
    (verb.to_string(), kind)
}

/// Python: bot/memory/told.py:222 `_the_verb(fact)`
/// A fact's verb, with a denial read as the thing it denies: "isnt" is "is".
pub fn _the_verb(fact: &Match) -> String {
    if let Some(verb) = fact.name("verb").filter(|v| !v.is_empty()) {
        return verb.to_lowercase();
    }
    let said = fact.name_str("not").to_lowercase().replace(" n't", "nt");
    for one in ["is", "was", "are", "does", "did", "has"] {
        if said.starts_with(one) {
            return one.to_string();
        }
    }
    said
}

/// Python: bot/memory/told.py:234 `_the_same_thing(fact, pred, claim)`
/// Whether a fact is about the same thing as the claim, said with a different value.
pub fn _the_same_thing(fact: &Match, pred: &str, claim: &str) -> bool {
    _the_shape(&_the_verb(fact), pred) == _the_shape("is", claim)
}

/// Python: bot/memory/told.py:361 `_tells_nothing(text)`
/// True when the line claims the bot was told and names nothing that it was told.
pub fn _tells_nothing(text_: &str) -> bool {
    if A_BARE_CLAIM.match_(text_).is_some() {
        return true;
    }
    let Some(found) = CLAIMED_ABOUT.match_(text_).or_else(|| A_CLAIM_WITHOUT_YOU.match_(text_)) else { return false };
    let what = A_TRAILING_TIME.sub("", &text::squash(found.name_str("what")));
    let what = A_LEADING_TIME.sub("", &what);
    let what = text::strip_chars(&what, " .,!");
    what.is_empty() || A_TIME_ONLY.match_(what).is_some()
}

/// Python: bot/memory/told.py:374 `_handed_back(text, memory)`
/// "So you did - you said your knee hurts.", from the last thing they said, or None.
pub fn _handed_back(text_: &str, memory: &Memory) -> Option<String> {
    let mine = text::squash(&text_.to_lowercase());
    let lines: Vec<String> = memory.turns.iter().filter(|t| !text::strip(&t.user).is_empty()).map(|t| text::squash(&t.user)).collect();
    for said in lines.iter().rev() {
        if said.to_lowercase() == mine {
            continue; // this turn is not the chat it is following on from
        }
        let quoted = _quote(said, memory);
        if quoted.is_empty() {
            continue;
        }
        let quoted = lower_first(&quoted);
        let frames: Vec<String> = ["So you did - ", "You're right - ", "You did - ", "Fair enough - "].iter().map(|f| format!("{f}{quoted}")).collect();
        return Some(pick(&frames, memory, ""));
    }
    None
}

/// Python: bot/memory/told.py:401 `is_a_boundary(text)`
/// True when the line is the person closing a subject rather than asking about one.
pub fn is_a_boundary(text_: &str) -> bool {
    let mut line = text::squash(text_);
    if line.is_empty() {
        return false;
    }
    if A_CLAIM_BEFORE.match_(&line).is_some() {
        line = A_CLAIM_BEFORE.subn("", &line, 1);
    }
    A_BOUNDARY.match_(&line).is_some()
}

/// Python: bot/memory/told.py:410 `at_a_boundary(text, memory=None)`
/// The short answer to a boundary, or None when the line is not one.
pub fn at_a_boundary(text_: &str, memory: &Memory) -> Option<String> {
    if !is_a_boundary(text_) {
        return None;
    }
    Some(pick(&LEFT_THERE, memory, ""))
}

/// Python: bot/memory/told.py:417 `_a_boundary_in_a_claim(text)`
pub fn _a_boundary_in_a_claim(text_: &str) -> bool {
    A_CLAIM_BEFORE.match_(&text::squash(text_)).is_some()
}

/// Which pattern `_the_ask` read the line with.
#[derive(Clone, Copy, PartialEq, Eq)]
enum AskShape {
    Told,
    Know,
    Claimed,
    ClaimWithoutYou,
    RememberTelling,
}

/// The ask's shape and its "what", or None.
fn the_ask_match(line: &str) -> Option<(AskShape, Match<'_>)> {
    let shapes: [(AskShape, &Re); 5] = [
        (AskShape::Told, *TOLD_ABOUT),
        (AskShape::Know, *KNOW_ABOUT),
        (AskShape::Claimed, *CLAIMED_ABOUT),
        (AskShape::ClaimWithoutYou, *A_CLAIM_WITHOUT_YOU),
        (AskShape::RememberTelling, *REMEMBER_TELLING),
    ];
    shapes.into_iter().find_map(|(shape, pattern)| pattern.match_(line).map(|m| (shape, m)))
}

/// Python: bot/memory/told.py:430 `_the_ask(text)`
/// The thing they ask about ("my sister", "the trip"), or None when the line is not this ask.
pub fn _the_ask(text_: &str) -> Option<String> {
    if is_a_boundary(text_) {
        return None; // a limit on the chat, not a thing asked about in it
    }
    let (shape, found) = the_ask_match(text_)?;
    let squashed = text::squash(found.name_str("what"));
    let said = text::strip_chars(&A_TRAILING_TIME.sub("", &squashed), " .,!").to_string();
    if said.is_empty() {
        return None;
    }
    if matches!(shape, AskShape::Claimed | AskShape::ClaimWithoutYou | AskShape::RememberTelling)
        && POINTING.contains(&text::split_ref(&said)[0].to_lowercase())
    {
        return None; // "i mentioned it before" points at the chat itself
    }
    // "have i told you my name is sam" states the fact: it is news.
    if shape == AskShape::Told && found.name("about").is_none_or(str::is_empty) && A_STATEMENT.is_match(&said) {
        return None;
    }
    Some(said)
}

/// Python: bot/memory/told.py:456 `ABOUT_THE_CHAT(text, memory=None)`
/// True when the line is this ask, whatever word in it another module would claim.
pub fn ABOUT_THE_CHAT(text_: &str, _memory: Option<&Memory>) -> bool {
    if _tells_nothing(text_) || is_a_boundary(text_) {
        return true;
    }
    let Some(said) = _the_ask(text_) else { return false };
    let what = A_LEADING_DETERMINER.sub("", &said);
    let what = ALREADY.sub("", text::strip_chars(&what, " .,!"));
    !what.is_empty() && !POINTING.contains(&what.to_lowercase())
}

/// Python: bot/memory/told.py:479 `answer(text, memory, facts=None, plain=None, claims=None)`
/// The thing they ask about, said back from the chat; "" when the line is not this ask.
pub fn answer(text_: &str, memory: &mut Memory, facts: &Value, plain: &Value, claims: &Value) -> String {
    if is_a_boundary(text_) {
        return at_a_boundary(text_, memory).unwrap_or_default();
    }
    if _tells_nothing(text_) {
        if let Some(handed) = _handed_back(text_, memory) {
            return handed;
        }
    }
    if _a_claim_about_the_chat(text_) && !_an_earlier_line(memory, text_) {
        return String::new(); // a telling, not a claim about what the notes hold
    }
    if let Some(checked) = check(text_, memory) {
        return checked;
    }
    if let Some(pointed) = _a_claim_that_points(text_, memory) {
        return pointed;
    }
    let said = _the_answer(text_, memory, facts, plain, claims);
    if let Some(s) = &said {
        if A_DENIAL.match_(s).is_some() && A_CLAUSE.match_(&_the_ask(text_).unwrap_or_default()).is_some() {
            return pick(&CALLBACK_MISSED, memory, "");
        }
    }
    match said {
        Some(s) if !s.is_empty() && SAID_AS_IT_IS.match_(&s).is_some() => s,
        other => _with_the_yes(&other.unwrap_or_default(), text_),
    }
}

/// Python: bot/memory/told.py:523 `a_denial(said)`
/// True when the sentence says the chat does not hold the thing - a miss, not a recall.
pub fn a_denial(said: &str) -> bool {
    A_DENIAL.match_(said).is_some() || A_QUOTE.is_match(said) || SAID_AS_IT_IS.match_(said).is_some()
}

/// Python: bot/memory/told.py:541 `_lower_first(said)`
/// The sentence with a common opener lowercased, ready to follow "Yes — ".
pub fn _lower_first(said: &str) -> String {
    if said.is_empty() {
        return String::new();
    }
    let first_word = said.split(' ').next().unwrap_or("");
    let first = text::strip_chars(first_word, ".,!?\"'—-").to_lowercase();
    let first = first.split('\'').next().unwrap_or("");
    if LOWERABLE.contains(first) { lower_first(said) } else { said.to_string() }
}

/// Python: bot/memory/told.py:557 `_with_the_yes(said, text)`
/// `said` with "Yes — " in front of it when the line is a yes/no question about the chat.
pub fn _with_the_yes(said: &str, text_: &str) -> String {
    if said.is_empty() || A_QUOTE.is_match(said) || TOLD_ABOUT.match_(text_).is_none() {
        return said.to_string();
    }
    format!("Yes — {}", _lower_first(said))
}

/// Python: bot/memory/told.py:565 `_the_remembered(text)`
/// "my knee hurt" out of "remember when i said my knee hurt", or None.
pub fn _the_remembered(text_: &str) -> Option<String> {
    let found = REMEMBERED.match_(text_)?;
    let squashed = text::squash(found.name_str("what"));
    let said = text::strip_chars(&squashed, " .,!");
    let said = text::strip(&ALREADY.sub("", said)).to_string();
    if said.is_empty() || POINTING.contains(&said.to_lowercase()) {
        return None;
    }
    Some(said)
}

/// Python: bot/memory/told.py:577 `A_CALLBACK(text, memory=None)`
/// True when the line asks the bot to remember a thing they said.
pub fn A_CALLBACK(text_: &str, _memory: Option<&Memory>) -> bool {
    _the_remembered(text_).is_some()
}

/// Python: bot/memory/told.py:588 `a_callback_the_chat_has(text, memory)`
/// True when the line is a callback to something this chat already holds.
pub fn a_callback_the_chat_has(text_: &str, memory: &Memory) -> bool {
    let Some(said) = _the_remembered(text_) else { return false };
    let what = A_LEADING_DETERMINER.sub("", &said);
    let what = text::strip_chars(&what, " .,!");
    !_line_about(memory, if what.is_empty() { &said } else { what }, false, text_).is_empty()
}

/// Python: bot/memory/told.py:602 `_a_claim_about_the_chat(text)`
pub fn _a_claim_about_the_chat(text_: &str) -> bool {
    CLAIMED_ABOUT.match_(text_).is_some() || A_CLAIM_WITHOUT_YOU.match_(text_).is_some()
}

/// Python: bot/memory/told.py:608 `_an_earlier_line(memory, text)`
pub fn _an_earlier_line(memory: &Memory, text_: &str) -> bool {
    history::an_earlier_turn(memory, text_)
}

/// Python: bot/memory/told.py:617 `callback(text, memory, facts=None, plain=None, claims=None)`
/// The thing they said, handed back to them, or None when the line is not this.
pub fn callback(text_: &str, memory: &mut Memory, facts: &Value, plain: &Value, claims: &Value) -> Option<String> {
    let said = _the_remembered(text_)?;
    if is_a_boundary(text_) {
        return at_a_boundary(text_, memory);
    }
    if _a_claim_about_the_chat(text_) && !_an_earlier_line(memory, text_) {
        return None;
    }
    let recalled = answer(&format!("have i told you about {said}"), memory, facts, plain, claims);
    if !recalled.is_empty() && A_DENIAL.match_(&recalled).is_none() {
        return Some(recalled);
    }
    Some(pick(&CALLBACK_MISSED, memory, ""))
}

/// Python: bot/memory/told.py:650 `_the_answer(text, memory, facts=None, plain=None, claims=None)`
/// The readers that answer one thing they ask about, in the order that reads best.
pub fn _the_answer(text_: &str, memory: &mut Memory, facts: &Value, plain: &Value, claims: &Value) -> Option<String> {
    let said = _the_ask(text_)?;
    let mut bare = text::strip(&SOMETHING_ABOUT.sub("", &said)).to_string();
    if let Some(pointed) = _the_proof_they_asked_for(&bare, text_, memory) {
        return Some(pointed);
    }
    let know = KNOW_ABOUT.match_(text_).is_some() && NOTHING_NAMED.match_(&bare).is_none();
    if NOTHING_NAMED.match_(&bare).is_some() {
        return None; // the store's own quoting reader says the last line back
    }
    if know {
        // The fact asked in a question is turned into the phrase the store's reader knows.
        if let Some((_, plain_ask)) = IN_QUESTION.iter().find(|(pattern, _)| pattern.match_(&bare).is_some()) {
            bare = plain_ask.clone();
        }
    }
    if plain.truthy() && know && !POINTING.contains(&bare.to_lowercase()) {
        if let Some(kept) = run_plain(plain, &bare, memory).filter(|k| !k.is_empty()) {
            return Some(kept);
        }
    }
    let what = A_LEADING_DETERMINER.sub("", &bare);
    let what = text::strip(&ALREADY.sub("", text::strip_chars(&what, " .,!"))).to_string();
    if what.is_empty() || POINTING.contains(&what.to_lowercase()) {
        return None;
    }
    let known = user(memory).clone();
    if let Some(theirs) = _about_a_person(memory, &what) {
        return Some(theirs);
    }
    let someone = _the_person_a_pronoun_points_at(memory, &what);
    if !someone.is_empty() {
        return _about_a_person(memory, &someone);
    }
    if let Some(pet) = _about_a_pet(memory, &what, &known, facts) {
        return Some(pet);
    }
    if WHERE_I_LIVE.match_(&what).is_some() {
        let said = run_reader(facts, "places", memory, "");
        if !said.is_empty() {
            return Some(said);
        }
    }
    if _names_only_a_kind(&what) {
        let kept = _the_fact_of_that_kind(memory, &what, &known, facts);
        if !kept.is_empty() {
            return Some(kept);
        }
    }
    // A kind word is not the word they used: the line about that fact is found under either word.
    let kind_words = _the_words_of_the_kind(&what);
    if !kind_words.is_empty() {
        let mut sorted: Vec<&String> = kind_words.iter().collect();
        sorted.sort();
        let joined = sorted.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" ");
        let quoted = _line_about(memory, &joined, true, text_);
        if !quoted.is_empty() {
            return Some(_quote(&quoted, memory));
        }
    }
    let kept = _the_fact_of_that_kind(memory, &what, &known, facts);
    if !kept.is_empty() {
        return Some(kept);
    }
    if let Some(named) = _the_fact_that_names(memory, &what, &known, facts) {
        return Some(named);
    }
    let quoted = _line_about(memory, &what, false, text_);
    if !quoted.is_empty() {
        return Some(_quote(&quoted, memory));
    }
    if know && !claims.is_none() && !run_claims(claims, memory) {
        return None; // a question about the world in the knowing frame
    }
    if let Some(dated) = _the_property_of_the_thing(memory, &bare, facts) {
        return Some(dated);
    }
    Some(_no_note(&_yours(&bare), memory))
}

/// Python: bot/memory/told.py:765 `_the_property_of_the_thing(memory, what, facts)`
/// What the chat holds for the property the ask names ("when", "where"), when it names only that.
pub fn _the_property_of_the_thing(memory: &mut Memory, what: &str, facts: &Value) -> Option<String> {
    let asked = text::strip_chars(&text::squash(&what.to_lowercase()), " .,!?").to_string();
    if A_DATE_PROPERTY.contains(&asked) {
        return super::when::when_answer("when is it", memory).opt_str();
    }
    if A_PLACE_PROPERTY.contains(&asked) {
        return run_reader(facts, "places", memory, "").opt_str();
    }
    None
}

/// Python: bot/memory/told.py:779 `_a_clause_is_named(line)`
/// True when a claim about the chat names a sentence as well as the telling.
pub fn _a_clause_is_named(line: &str) -> bool {
    let Some(found) = CLAIMED_ABOUT.match_(line).or_else(|| A_CLAIM_WITHOUT_YOU.match_(line)) else { return false };
    A_CLAUSE.match_(&text::squash(found.name_str("what"))).is_some()
}

/// Python: bot/memory/told.py:787 `_the_thing_the_chat_is_on(memory)`
pub fn _the_thing_the_chat_is_on(memory: &Memory) -> String {
    crate::memory::store_replies::_the_thing_the_chat_is_on(memory)
}

/// Python: bot/memory/told.py:797 `_no_note(thing, memory)`
/// The denial, said about the note rather than about the chat.
pub fn _no_note(thing: &str, memory: &Memory) -> String {
    let frames: Vec<String> = if _the_thing_is_a_clause(thing) {
        vec![
            format!("Nothing in my notes says \u{201c}{thing}\u{201d} yet — tell me and I'll keep it."),
            format!("I don't have \u{201c}{thing}\u{201d} in my notes — say it and I'll hold on to it."),
            format!("\u{201c}{thing}\u{201d} isn't in what I keep yet — tell me and I'll remember it."),
        ]
    } else {
        vec![
            format!("I haven't kept a note of {thing} — tell me and I'll remember it."),
            format!("There's nothing on {thing} in what I keep — say it and I'll hold on to it."),
            format!("Nothing on {thing} in my notes yet — tell me and I'll keep it."),
        ]
    };
    pick(&frames, memory, "")
}

/// Python: bot/memory/told.py:836 `_the_thing_is_a_clause(thing)`
/// True when what they handed back is a sentence they said, not the name of a thing.
pub fn _the_thing_is_a_clause(thing: &str) -> bool {
    let words = text::split_ref(thing);
    if words.is_empty() {
        return false;
    }
    if words.len() > 3 {
        return true;
    }
    A_CLAUSE_OPENING.is_match(&words.join(" "))
}

/// Python: bot/memory/told.py:863 `_quote(line, memory)`
/// Their own line, said back to them in the second person.
pub fn _quote(line: &str, memory: &Memory) -> String {
    let line = match REPORTED.match_(line) {
        Some(reported) => reported.name_str("clause").to_string(),
        None => line.to_string(),
    };
    let opened = I_M.sub("i'm", &line);
    let opened = I_VE.sub("i've", &opened);
    let said = text::rstrip_chars(text::strip(&super::you(&opened)), ".").to_string();
    if said.is_empty() {
        return pick(&["You told me that already".to_string(), "You did say that".to_string()], memory, "");
    }
    let said = format!("{}.", lower_first(&said));
    if TELLING_ALREADY.match_(&said).is_some() {
        return upper_first(&said);
    }
    let frames: Vec<String> = ["You said ", "You told me ", "You did say "].iter().map(|f| format!("{f}{said}")).collect();
    pick(&frames, memory, "")
}

/// Python: bot/memory/told.py:900 `_the_words_of_the_kind(what)`
/// Every word the store files under the kind they asked about, or an empty set.
pub fn _the_words_of_the_kind(what: &str) -> HashSet<String> {
    let mut kinds = HashSet::new();
    for word in SPLIT_WORDS.split(&what.to_lowercase()) {
        let kind = _the_kind_of(&word);
        if kind.is_empty() {
            return HashSet::new();
        }
        kinds.insert(kind);
    }
    if kinds.len() != 1 {
        return HashSet::new();
    }
    let kind = kinds.into_iter().next().unwrap();
    A_KIND.iter().filter(|(_, key)| *key == kind).map(|(word, _)| word.clone()).collect()
}

/// Python: bot/memory/told.py:931 `_yours(phrase)`
/// The thing asked about, said from the bot's side: "my back" -> "your back"; a clause is
/// handed back as it was said.
pub fn _yours(phrase: &str) -> String {
    if A_VERB_IN_A_CLAUSE.is_match(phrase) {
        return phrase.to_string();
    }
    MINE.sub_fn(phrase, |m| YOURS.get(&m.str(1).to_lowercase()).cloned().unwrap_or_else(|| m.str(1).to_string()))
}

/// Python: bot/memory/told.py:945 `_the_person_a_pronoun_points_at(memory, what)`
/// Who "her", "him" or "them" is: the last person the chat named, or "".
pub fn _the_person_a_pronoun_points_at(memory: &mut Memory, what: &str) -> String {
    let word = text::rstrip_chars(&text::strip(what).to_lowercase(), ".,!?").to_string();
    if !matches!(word.as_str(), "her" | "him" | "them" | "they") {
        return String::new();
    }
    let entry = others(memory).items().iter().rev().find(|kept| kept.is_dict() && pyv::get(kept, "who").truthy()).cloned();
    if let Some(entry) = entry {
        return text::rstrip_chars(&pyv::str_of(pyv::get(&entry, "who")).to_lowercase(), "s").to_string();
    }
    for turn in memory.turns.iter().rev() {
        for other in LOWER_LETTERS.findall_str(&turn.user.to_lowercase()) {
            let bare = text::rstrip_chars(&other, "s");
            if RELATIONS.contains(bare) {
                return bare.to_string();
            }
        }
    }
    String::new()
}

/// Python: bot/memory/told.py:973 `_words_of(said)`
/// The words a claim or a fact is made of: no articles, no punctuation, no filler.
pub fn _words_of(said: &str) -> HashSet<String> {
    WORDS_AND_DIGITS
        .findall_str(&said.to_lowercase())
        .into_iter()
        .filter(|w| (!CLAIM_FILLER.contains(w) && text::len(w) > 1) || text::is_digit(w))
        .collect()
}

/// Python: bot/memory/told.py:979 `check(text, memory)`
/// A yes/no check of a fact the chat gave about someone else, answered from that line.
pub fn check(text_: &str, memory: &mut Memory) -> Option<String> {
    let found = CHECK_OF_SOMEONE.match_(text_)?;
    let claim = found.name_str("claim").to_string();
    if CHECKED_ELSEWHERE.is_match(&claim) {
        return None;
    }
    let asked = _words_of(&claim);
    if asked.is_empty() {
        return None;
    }
    let who = LOWER_LETTERS.findall_str(&found.as_str().to_lowercase()).into_iter().find(|w| RELATIONS.contains(w))?;
    // A newer line about the same thing, said with a different value, is the last word on it.
    let mut newer: Option<String> = None;
    let mut newer_said = String::new();
    let mut taken_back = false;
    for said in _their_lines(memory, &who).iter().rev() {
        let mut fact = SAID_OF_SOMEONE.match_(said);
        let mut by_pronoun = false;
        if fact.is_none() {
            // A take-back with nothing after it is read before the pronoun fact.
            if let Some(bare) = BARE_DENIAL_PRONOUN.match_(said).or_else(|| BARE_DENIAL.match_(said)) {
                let named = text::rstrip_chars(&text::strip(bare.name_str("who")).to_lowercase(), "s").to_string();
                if named.is_empty() || named == who {
                    taken_back = true;
                }
                continue;
            }
            fact = SAID_OF_PRONOUN.match_(said);
            by_pronoun = fact.is_some();
        }
        let Some(fact) = fact else { continue };
        let pred = fact.name("pred").filter(|p| !p.is_empty()).or_else(|| fact.name("pred2")).unwrap_or("").to_string();
        if !asked.is_subset(&_words_of(&pred)) {
            if newer.is_none() && _the_same_thing(&fact, &pred, &claim) {
                newer = Some(text::squash(&pred));
                newer_said = said.clone();
            }
            continue;
        }
        if let Some(newer) = &newer {
            if *newer != text::squash(&pred) {
                return Some(format!("You've told me your {who} is {newer}, and before that {}. Which is right?", text::squash(&pred)));
            }
        }
        let denial = fact.name("not").is_some_and(|n| !n.is_empty()) != taken_back;
        let said_back = if by_pronoun {
            let verb = fact.name("not").filter(|n| !n.is_empty()).or_else(|| fact.name("verb")).unwrap_or("");
            _names_in_capitals(&_plain_contractions(text::strip(&format!("your {who} {verb} {pred}"))))
        } else if taken_back {
            _the_denial(&who, &_the_verb(&fact), &pred)
        } else {
            _names_in_capitals(&_plain_contractions(text::strip_chars(&_your_own_clause(said), " .,!")))
        };
        if said_back.is_empty() {
            continue;
        }
        let opening = if denial != _claim_is_a_denial(&claim) { "No — " } else { "Yes — " };
        return Some(format!("{opening}{}.", lower_first(&said_back)));
    }
    // The same fact said with another value: the fact as they gave it, with no yes or no.
    if let Some(newer) = newer.filter(|n| !n.is_empty()) {
        if !newer_said.is_empty() && _the_shape("is", &newer).1 != "said" {
            let said_back = _names_in_capitals(&_plain_contractions(text::strip_chars(&_your_own_clause(&newer_said), " .,!")));
            if !said_back.is_empty() {
                return Some(format!("{}.", upper_first(&said_back)));
            }
        }
    }
    None
}

/// Python: bot/memory/told.py:1085 `_your_own_clause(said)`
/// Their line about their own person, said from the bot's side: "your mum is 71".
pub fn _your_own_clause(said: &str) -> String {
    let line = CORRECTING.sub("", text::strip(said));
    match A_CLAUSE_OF_MINE.match_(&line) {
        Some(found) => {
            let yours = YOURS.get(&found.str(1).to_lowercase()).cloned().unwrap_or_else(|| "your".into());
            format!("{yours}{}", &line[found.end()..])
        }
        None => line,
    }
}

/// Python: bot/memory/told.py:1101 `_plain_contractions(said)`
/// The line with the words a person writes as one word put back together.
pub fn _plain_contractions(said: &str) -> String {
    let mut said = said.to_string();
    for (words, joined) in CONTRACTIONS.iter() {
        let pattern = format!(r"\b{}\b", words.join(r"\s+"));
        said = Re::new(&pattern, re::I | re::U).sub(&joined.replace('\\', "\\\\"), &said);
    }
    said
}

/// Python: bot/memory/told.py:1113 `_names_in_capitals(said)`
pub fn _names_in_capitals(said: &str) -> String {
    CALLED_NAME.sub_fn(said, |m| format!("called {}", upper_first(m.str(1))))
}

/// Python: bot/memory/told.py:1122 `_claim_is_a_denial(claim)`
pub fn _claim_is_a_denial(claim: &str) -> bool {
    A_DENIAL.match_(claim).is_some()
}

/// Python: bot/memory/told.py:1126 `_their_lines(memory, who)`
/// Every line of theirs about that person, oldest first, and the ones said of them by pronoun.
pub fn _their_lines(memory: &mut Memory, who: &str) -> Vec<String> {
    let named = Re::new(&format!(r"\b{}s?\b", re::escape(who)), re::U);
    let pronoun = _a_pronoun_for(who);
    let mut lines: Vec<String> = Vec::new();
    for turn in &memory.turns {
        let said = &turn.user;
        if named.is_match(&said.to_lowercase()) {
            lines.push(_trim(said, 80));
        } else if !lines.is_empty() && pronoun.is_match(said) {
            lines.push(_trim(said, 80));
        }
    }
    let kept = pyv::get(user(memory), "person").clone();
    if kept.is_dict() {
        if let Value::Str(said) = pyv::get(&kept, "said") {
            lines.push(_trim(said, 80));
        }
    }
    lines
}

/// Python: bot/memory/told.py:1147 `_about_a_person(memory, what)`
/// What the store holds about a relation of theirs, or None when the ask names no one.
pub fn _about_a_person(memory: &mut Memory, what: &str) -> Option<String> {
    for word in text::split_ref(what) {
        let word = text::strip_chars(word, ".,!?");
        let who = text::rstrip_chars(&word.to_lowercase(), "s").to_string();
        if !RELATIONS.contains(&who) {
            continue;
        }
        let entries = others(memory).items().to_vec();
        for entry in &entries {
            if text::rstrip_chars(&pyv::str_or_empty(pyv::get(entry, "who")).to_lowercase(), "s") != who {
                continue;
            }
            let fact = _their_fact(&who, entry);
            if !fact.is_empty() {
                return Some(fact);
            }
        }
        // Their own words still answer the question, and that beats denying it.
        let quoted = _line_about(memory, &who, true, "");
        if !quoted.is_empty() {
            return Some(_quote(&quoted, memory));
        }
        return Some(_no_note(&format!("your {who}"), memory));
    }
    None
}

/// Python: bot/memory/told.py:1170 `_their_fact(who, entry)`
/// One fact about someone else, said with the relation in it.
pub fn _their_fact(who: &str, entry: &Value) -> String {
    let allergies = pyv::get(entry, "allergies");
    if allergies.truthy() {
        let items: Vec<Value> = if allergies.is_list() { allergies.items().to_vec() } else { vec![allergies.clone()] };
        return format!("You said your {who} is allergic to {}.", items.iter().map(pyv::str_of).collect::<Vec<_>>().join(", "));
    }
    let diet = pyv::get(entry, "diet");
    if diet.truthy() {
        return format!("You said your {who} is {}.", pyv::str_of(diet));
    }
    if pyv::get(entry, "no_meat").truthy() {
        return format!("You said your {who} doesn't eat meat.");
    }
    let name = pyv::get(entry, "name");
    if name.truthy() {
        let name = text::split_ref(&pyv::str_of(name)).iter().map(|w| upper_first(w)).collect::<Vec<_>>().join(" ");
        return format!("Your {who} is called {name}.");
    }
    let condition = pyv::get(entry, "condition");
    if condition.truthy() {
        return format!("You said your {who} has {}.", pyv::str_of(condition));
    }
    String::new()
}

/// Python: bot/memory/told.py:1192 `_about_a_pet(memory, what, known, facts=None)`
/// A pet they ask about by its kind: "have I told you about the dog?".
pub fn _about_a_pet(memory: &mut Memory, what: &str, known: &Value, facts: &Value) -> Option<String> {
    if NOT_MINE.is_match(&format!("{what} ")) {
        return None;
    }
    let name = match ANIMAL.search(what) {
        Some(kind) => run_reader(facts, "family", memory, text::rstrip_chars(&kind.as_str().to_lowercase(), "s")),
        None => {
            // A breed is the animal it is.
            let breed = BREED_RE.search(what)?;
            let name = BREED_KIND.get(&crate::understanding::breeds::the_breed_named(breed.as_str())).cloned().unwrap_or_default();
            if name.is_empty() {
                return None;
            }
            name
        }
    };
    let held = pyv::get(known, "pets");
    let pets: Vec<Value> = if held.truthy() { pyv::iter(held).into_iter().filter(Value::is_dict).collect() } else { Vec::new() };
    if pets.is_empty() {
        return None;
    }
    if name == "pet" || name == "animal" {
        return _the_pets(memory, &pets, facts);
    }
    let mut mine = Vec::new();
    for pet in &pets {
        let animal = text::rstrip_chars(&pyv::str_or_empty(pyv::get(pet, "animal")).to_lowercase(), "s").to_string();
        if run_reader(facts, "family", memory, &animal) == name {
            mine.push(pet.clone());
        }
    }
    if mine.is_empty() {
        return Some(_no_note(&format!("a {name}"), memory));
    }
    let said = run_reader(facts, "pets", memory, &name);
    if !said.is_empty() && mine.iter().any(|pet| pyv::get(pet, "name").truthy()) {
        return Some(said);
    }
    let pet = &mine[mine.len() - 1];
    if pyv::get(pet, "coming").truthy() {
        return Some(format!("You're getting {}.", _many(pet)));
    }
    Some(format!("You've got {}.", _many(pet)))
}

/// Python: bot/memory/told.py:1235 `_the_pets(memory, pets, facts=None)`
/// All the pets the store holds, in the store's own sentence when they are one kind.
pub fn _the_pets(memory: &mut Memory, pets: &[Value], facts: &Value) -> Option<String> {
    let here: Vec<Value> = pets.iter().filter(|p| !pyv::get(p, "coming").truthy()).cloned().collect();
    let coming: Vec<Value> = pets.iter().filter(|p| pyv::get(p, "coming").truthy()).cloned().collect();
    let said = _the_pets_here(memory, &here, facts);
    let owed = _the_pets_here(memory, &coming, facts).map(|owed| owed.replacen("You've got ", "You're getting ", 1));
    let parts: Vec<String> = [said, owed].into_iter().flatten().filter(|one| !one.is_empty()).collect();
    if parts.is_empty() { None } else { Some(parts.join(" ")) }
}

/// Python: bot/memory/told.py:1247 `_the_pets_here(memory, pets, facts=None)`
pub fn _the_pets_here(memory: &mut Memory, pets: &[Value], facts: &Value) -> Option<String> {
    let mut kinds: Vec<String> = Vec::new();
    for pet in pets {
        let bare = text::rstrip_chars(&pyv::str_or_empty(pyv::get(pet, "animal")).to_lowercase(), "s").to_string();
        let animal = run_reader(facts, "family", memory, &bare);
        if !animal.is_empty() && !kinds.contains(&animal) {
            kinds.push(animal);
        }
    }
    if kinds.len() == 1 {
        let said = run_reader(facts, "pets", memory, &kinds[0]);
        if !said.is_empty() {
            return Some(said);
        }
    }
    let said: Vec<String> = kinds
        .iter()
        .map(|kind| {
            let of_kind: Vec<&Value> = pets
                .iter()
                .filter(|pet| {
                    let animal = text::rstrip_chars(&pyv::str_or_empty(pyv::get(pet, "animal")).to_lowercase(), "s").to_string();
                    animal == *kind || animal == format!("{kind}s")
                })
                .collect();
            _many(of_kind.last().expect("IndexError: list index out of range"))
        })
        .filter(|one| !one.is_empty())
        .collect();
    match said.len() {
        0 => None,
        1 => Some(format!("You've got {}.", said[0])),
        n => Some(format!("You've got {} and {}.", said[..n - 1].join(", "), said[n - 1])),
    }
}

/// Python: bot/memory/told.py:1272 `_a(word)`
pub fn _a(word: &str) -> String {
    let vowel = word.chars().next().is_some_and(|c| "aeiou".contains(c.to_lowercase().next().unwrap()));
    format!("{}{word}", if vowel || word.is_empty() { "an " } else { "a " })
}

/// Python: bot/memory/told.py:1276 `_many(pet)`
/// A pet said with its name or its count: "a cat called Milo", "two cats", "a dog".
pub fn _many(pet: &Value) -> String {
    let kind = {
        let animal = pyv::get(pet, "animal");
        if animal.truthy() { pyv::str_of(animal) } else { "pet".to_string() }
    };
    let breed = pyv::get(pet, "breed");
    let animal = if pyv::get(pet, "breed_of_one").truthy() || !breed.truthy() { kind.clone() } else { pyv::str_of(breed) };
    let many = if text::ends_with_any(&kind, &["fish", "sheep", "deer"]) { kind.clone() } else { format!("{kind}s") };
    let name = pyv::get(pet, "name");
    if name.truthy() {
        return format!("{} called {}", _a(&animal), upper_first(&pyv::str_of(name)));
    }
    // `int(pet.get("count") or 1)`, 1 when it is not a number.
    let count = pyv::get(pet, "count");
    let count = if !count.truthy() {
        1
    } else {
        match count {
            Value::Int(i) => *i,
            Value::Bool(b) => *b as i64,
            Value::Float(f) => f.trunc() as i64,
            Value::Str(s) => text::strip(s).parse().unwrap_or(1),
            _ => 1,
        }
    };
    if count > 1 {
        return format!("{} {many}", COUNT_WORDS.get(&count).cloned().unwrap_or_else(|| count.to_string()));
    }
    _a(&animal)
}

/// Python: bot/memory/told.py:1335 `_the_newest_stated_line(memory, skip='', a_question_ok=False)`
/// The last line of theirs that states something, or "".
pub fn _the_newest_stated_line(memory: &Memory, skip: &str, a_question_ok: bool) -> String {
    let asked = _flat(skip);
    for turn in memory.turns.iter().rev() {
        let said = &turn.user;
        if text::strip(said).is_empty() || _flat(said) == asked {
            continue;
        }
        if !a_question_ok && (text::rstrip(said).ends_with('?') || ASKS.match_(said).is_some()) {
            continue;
        }
        if _a_claim_about_the_chat(said) {
            continue;
        }
        return text::squash(said);
    }
    String::new()
}

/// Python: bot/memory/told.py:1356 `_a_claim_that_points(text, memory)`
/// The proof for a claim that names the chat rather than a thing ("i said that already").
pub fn _a_claim_that_points(text_: &str, memory: &Memory) -> Option<String> {
    if POINTING_SAID.match_(text_).is_some() {
        return _the_proof_they_asked_for("", text_, memory);
    }
    let found = CLAIMED_ABOUT.match_(text_).or_else(|| A_CLAIM_WITHOUT_YOU.match_(text_))?;
    let said = text::strip_chars(&A_TRAILING_TIME.sub("", &text::squash(found.name_str("what"))), " .,!").to_string();
    if !said.is_empty() && !POINTING.contains(&said.to_lowercase()) {
        return None;
    }
    _the_proof_they_asked_for(&said, text_, memory)
}

/// Python: bot/memory/told.py:1371 `_the_proof_they_asked_for(named, text, memory)`
/// The line they are asking for, when the claim points at the chat instead of naming a thing.
pub fn _the_proof_they_asked_for(named: &str, text_: &str, memory: &Memory) -> Option<String> {
    let said = text::squash(named);
    if !said.is_empty() && ONLY_A_POINTING_TIME.match_(text::strip(&POINTING_TAIL.sub("", &said))).is_none() {
        return None;
    }
    let told_about = _the_thing_the_chat_is_on(memory);
    let mut quoted = if told_about.is_empty() { String::new() } else { _line_about(memory, &told_about, false, text_) };
    if quoted.is_empty() {
        quoted = _the_newest_stated_line(memory, text_, false);
    }
    if quoted.is_empty() {
        let asked_line = _the_newest_stated_line(memory, text_, true);
        if asked_line.is_empty() {
            return None;
        }
        return Some(pick(
            &[
                format!("All I have from you so far is \"{asked_line}\" — tell me the rest and I'll keep it."),
                format!("What I have is \"{asked_line}\". Say it in your own words and it is in."),
            ],
            memory,
            "",
        ));
    }
    Some(pick(
        &[
            format!("You did — \"{quoted}\" is what I kept."),
            format!("\"{quoted}\" — that one I have."),
            format!("I have it: \"{quoted}\"."),
        ],
        memory,
        "",
    ))
}

/// Python: bot/memory/told.py:1404 `_line_about(memory, what, whole_words=False, skip='')`
/// The last line of theirs about the thing, trimmed, or "" when they never said one.
pub fn _line_about(memory: &Memory, what: &str, whole_words: bool, skip: &str) -> String {
    let words: Vec<String> = SPLIT_WORDS.split(&what.to_lowercase()).into_iter().filter(|w| text::len(w) > 2).collect();
    if words.is_empty() {
        return String::new();
    }
    let asked = _flat(skip);
    for turn in memory.turns.iter().rev() {
        let said = &turn.user;
        if text::strip(said).is_empty() || _flat(said) == asked {
            continue;
        }
        if text::rstrip(said).ends_with('?') || ASKS.match_(said).is_some() {
            continue;
        }
        if _a_claim_about_the_chat(said) && !_a_clause_is_named(said) {
            continue;
        }
        if super::recall::ask(said, false).as_deref() == Some("me") {
            continue;
        }
        if TOLD_ABOUT.match_(said).is_some() || CLAIMS_TOLD.is_match(said) || REMEMBERED.match_(said).is_some() {
            continue;
        }
        let low = said.to_lowercase();
        let found = if whole_words {
            words.iter().any(|word| Re::new(&format!(r"\b{}s?\b", re::escape(word)), re::U).is_match(&low))
        } else {
            words.iter().any(|word| low.contains(word.as_str()))
        };
        if found {
            return _trim(said, 100);
        }
    }
    String::new()
}

/// Python: bot/memory/told.py:1447 `_a_trip_line(memory)`
/// The last line of theirs about going somewhere, trimmed, or "".
pub fn _a_trip_line(memory: &Memory) -> String {
    for turn in memory.turns.iter().rev() {
        let said = &turn.user;
        if text::strip(said).is_empty() || text::rstrip(said).ends_with('?') {
            continue;
        }
        if A_TRIP.is_match(said) {
            return _trim(said, 100);
        }
    }
    String::new()
}

/// Python: bot/memory/told.py:1463 `_flat(line)`
pub fn _flat(line: &str) -> String {
    text::strip_chars(&text::squash(&line.to_lowercase()), " .,!").to_string()
}

/// Python: bot/memory/told.py:1467 `_the_denial(who, verb, pred)`
/// "your mum isn't coeliac": the fact said the other way round, with the relation named.
pub fn _the_denial(who: &str, verb: &str, pred: &str) -> String {
    let word = match verb {
        "is" => "isn't",
        "was" => "wasn't",
        "are" => "aren't",
        "does" => "doesn't",
        "did" => "didn't",
        "has" => "hasn't",
        _ => "isn't",
    };
    format!("your {who} {word} {}", text::squash(pred))
}

/// Python: bot/memory/told.py:1474 `_a_pronoun_for(who)`
/// The pronouns that can point at that person: "she" for a mum, and "they" for anybody.
pub fn _a_pronoun_for(who: &str) -> Re {
    let word = text::strip(who).to_lowercase();
    let mut fits = vec!["they"];
    if SHE.contains(&word) {
        fits.push("she");
    }
    if HE.contains(&word) {
        fits.push("he");
    }
    Re::new(&format!(r"^\W*(?:{}\b[,.!;:]?\s*){{0,2}}(?:{})(?:'s|s)?\s+\S", *CORRECTION_WORDS, fits.join("|")), re::I | re::U)
}

/// Python: bot/memory/told.py:1486 `_trim(line, limit=100)`
/// Their line as a quote: one line, short enough to read inside a reply.
pub fn _trim(line: &str, limit: usize) -> String {
    let said = text::strip_chars(&text::squash(line), " .,!").to_string();
    if text::len(&said) > limit {
        let head = text::slice(&said, 0, Some(limit as isize));
        let cut = match head.rfind(' ') {
            Some(at) => head[..at].to_string(),
            None => head,
        };
        return format!("{cut}\u{2026}");
    }
    said
}

/// Python: bot/memory/told.py:1504 `_stem(word)`
/// A word without its endings, so "savings", "saving" and "save" are one thing to match on.
pub fn _stem(word: &str) -> String {
    let mut word = NON_LETTER.sub("", &word.to_lowercase());
    for _ in 0..3 {
        let ending = ["ings", "ing", "ies", "es", "ed", "s", "e", "y"]
            .into_iter()
            .find(|ending| text::len(&word) > ending.len() + 2 && word.ends_with(ending));
        match ending {
            Some(ending) => word.truncate(word.len() - ending.len()),
            None => break,
        }
    }
    word
}

/// Python: bot/memory/told.py:1520 `_kept_words(value, into=None)`
/// Every word of a kept fact, so a plan or a list of allergies can be searched for one.
pub fn _kept_words(value: &Value, into: &mut HashSet<String>) {
    match value {
        Value::Str(s) => into.extend(NOT_LETTERS.split(&s.to_lowercase()).into_iter().filter(|w| !w.is_empty())),
        Value::Dict(d) => d.iter().for_each(|(_, item)| _kept_words(item, into)),
        Value::List(items) | Value::Tuple(items) | Value::Set(items) => items.iter().for_each(|item| _kept_words(item, into)),
        _ => {}
    }
}

/// Python: bot/memory/told.py:1534 `_the_fact_that_names(memory, what, known, facts)`
/// The fact the ask names by one of its words, said the store's way, or None.
pub fn _the_fact_that_names(memory: &mut Memory, what: &str, known: &Value, facts: &Value) -> Option<String> {
    if !facts.truthy() {
        return None;
    }
    let stems: HashSet<String> =
        NOT_LETTERS_AZ.split(what).iter().filter(|w| text::len(w) > 2).map(|w| _stem(w)).filter(|s| !s.is_empty()).collect();
    if stems.is_empty() {
        return None;
    }
    for key in THE_KINDS_KEPT.iter() {
        let value = pyv::get(known, key);
        if !has_reader(facts, key) || !value.truthy() {
            continue;
        }
        let mut words = HashSet::new();
        _kept_words(value, &mut words);
        let kept: HashSet<String> = words.iter().map(|w| _stem(w)).collect();
        if stems.is_disjoint(&kept) {
            continue;
        }
        let said = run_reader(facts, key, memory, "");
        if !said.is_empty() && NOTHING_SAID.match_(&said).is_none() {
            return Some(said);
        }
    }
    None
}

/// Python: bot/memory/told.py:1555 `_the_kind_of(word)`
/// "plans" and "plan" are the same kind of fact, and so are "jobs" and "job".
pub fn _the_kind_of(word: &str) -> String {
    let word = text::strip_chars(word, ".,!?").to_lowercase();
    let n = text::len(&word);
    let candidates = [
        word.clone(),
        text::slice(&word, 0, Some(n as isize - 1)),
        if word.ends_with("es") { text::slice(&word, 0, Some(n as isize - 2)) } else { word.clone() },
    ];
    for candidate in candidates {
        if let Some((_, key)) = A_KIND.iter().find(|(kind, key)| *kind == candidate && !key.is_empty()) {
            return key.clone();
        }
    }
    String::new()
}

/// Python: bot/memory/told.py:1564 `_names_only_a_kind(what)`
/// True when every word of the thing asked about names a kind of fact ("my name", "the money").
pub fn _names_only_a_kind(what: &str) -> bool {
    let words: Vec<String> = SPLIT_WORDS
        .split(&what.to_lowercase())
        .into_iter()
        .filter(|w| !w.is_empty())
        .map(|w| text::strip_chars(&w, ".,!?").to_string())
        .collect();
    !words.is_empty() && words.iter().all(|w| !_the_kind_of(w).is_empty())
}

/// Python: bot/memory/told.py:1570 `_the_fact_of_that_kind(memory, what, known, facts=None)`
/// The fact a kind-naming ask asks about ("have i told you about the trip" -> the plan), in the
/// store's own sentence, or "".
pub fn _the_fact_of_that_kind(memory: &mut Memory, what: &str, known: &Value, facts: &Value) -> String {
    for word in SPLIT_WORDS.split(&what.to_lowercase()) {
        let key = _the_kind_of(&word);
        if key.is_empty() {
            continue;
        }
        let value = pyv::get(known, &key).clone();
        if !value.truthy() {
            // A trip they took is as much a trip as one they are planning.
            if key == "plan" {
                let places = pyv::get(known, "places");
                let places: Vec<Value> = if places.truthy() { pyv::iter(places) } else { Vec::new() };
                for place in places.iter().rev() {
                    if let Value::Str(place) = place {
                        let said = _line_about(memory, place, false, "");
                        if !said.is_empty() {
                            return _quote(&said, memory);
                        }
                    }
                }
                let said = _a_trip_line(memory);
                if !said.is_empty() {
                    return _quote(&said, memory);
                }
            }
            return String::new();
        }
        let said = run_reader(facts, &key, memory, "");
        if !said.is_empty() {
            return said;
        }
        if key == "plan" {
            return format!("You said: \"{}\"", _trim(&pyv::str_of(&value), 100));
        }
        let values: Vec<Value> = if matches!(value, Value::List(_)) { value.items().to_vec() } else { vec![value] };
        return format!("You told me: {}.", values.iter().map(pyv::str_of).collect::<Vec<_>>().join(", "));
    }
    String::new()
}
