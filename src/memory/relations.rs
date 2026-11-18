//! Port of `bot/memory/relations.py`: the people and animals in the store - a relation, the group
//! it belongs to, the name they were given, the age they were said to be, and the answer to a
//! question about them.
//!
//! Module data (patterns, word lists, group tables) comes from the exported bundle.
//!
//! Order notes: `facts.KIN_WORDS` is `sorted(set(KIN), key=len, reverse=True)` in PyBot, so the
//! order of two relation words of the same length follows CPython's per-process string hashing;
//! the bundle holds the order of the export run, and `_named_person` reads it in that order.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::facts::{self as extract, py_str};
use crate::py::re::{self as pyre, Re};
use crate::py::text as pt;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

const I: u32 = pyre::I;

fn g(name: &str) -> &'static Value {
    data::get(&format!("bot.memory.relations.{name}"))
}
fn re(name: &str) -> &'static Re {
    g(name).as_re().unwrap_or_else(|| panic!("bot.memory.relations.{name} is not a regex"))
}

macro_rules! patterns {
    ($($name:ident),* $(,)?) => {
        $(pub static $name: LazyLock<&'static Re> = LazyLock::new(|| re(stringify!($name)));)*
    };
}
patterns!(
    CHAT_TAIL, PERSON_AFTER, ABOUT_PERSON_ASK, ANOTHER_PERSON, AGE_ASK, AGE_RULE_WORD, AGE_YOU_SAY,
    AGE_COPULA, AGE_ADVERB, AGE_DETERMINER, AGE_PERSON_WORD, AGE_AGAIN, NAME_AGAIN, AGE_SAID,
    THE_KIDS, CHILD_WORD, AGE_WORD, AGE_AS_A_CAUSE, GROUP_ASK, THEIR_NAME, THEIR_AGE,
    GROUP_NAME_ASK, THEIR_TASTE_WHAT, THEIR_TASTE_INTO, THEIR_TASTE_NOT, THEIR_TASTE_NOT_INTO,
    THEIR_TASTE_A_YES_NO, THEIR_JOB, HAVE_ANY, GROUP_NAMES_TYPED, THEY_ARE_NAMED,
    SEVERAL_PEOPLE_NAMED, TWO_OF_KIN, _A_PLACE_ASK, PET_ASK, BREED_ASK, _PET_WORD, PLURAL_ANIMAL,
);

fn str_map(value: &Value) -> HashMap<String, String> {
    value.pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
}
fn group_map(value: &Value) -> HashMap<String, Vec<String>> {
    value.pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
}

static _IRREGULAR_PLURAL: LazyLock<HashMap<String, String>> = LazyLock::new(|| str_map(g("_IRREGULAR_PLURAL")));
static _PLURAL: LazyLock<HashSet<String>> = LazyLock::new(|| g("_PLURAL").str_set());
static PEOPLE_MANY: LazyLock<String> = LazyLock::new(|| g("PEOPLE_MANY").str().to_string());
static _THE_YOUNGER_END: LazyLock<HashSet<String>> = LazyLock::new(|| g("_THE_YOUNGER_END").str_set());
static _THE_PLACE_STANDS_ALONE: LazyLock<HashSet<String>> = LazyLock::new(|| g("_THE_PLACE_STANDS_ALONE").str_set());
static MANY_OF_THEM: LazyLock<HashMap<String, String>> = LazyLock::new(|| str_map(g("MANY_OF_THEM")));
static ALREADY_MANY: LazyLock<HashSet<String>> = LazyLock::new(|| g("ALREADY_MANY").str_set());

// `bot.memory.store_lines`: the words the store files people under.
fn lines(name: &str) -> &'static Value {
    data::get(&format!("bot.memory.store_lines.{name}"))
}
static CHILD: LazyLock<Vec<String>> = LazyLock::new(|| lines("CHILD").strs());
static CHILD_OF: LazyLock<HashMap<String, String>> = LazyLock::new(|| str_map(lines("CHILD_OF")));
static CHILD_RELATIONS: LazyLock<Vec<String>> = LazyLock::new(|| lines("CHILD_RELATIONS").strs());
static FAMILY_GROUP: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| group_map(lines("FAMILY_GROUP")));
static GROUP_OF: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| group_map(lines("GROUP_OF")));
static HER_WORD: LazyLock<HashMap<String, String>> = LazyLock::new(|| str_map(lines("HER_WORD")));
static PERSON_GROUP: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| group_map(lines("PERSON_GROUP")));
static PRONOUN_PEOPLE: LazyLock<Vec<String>> = LazyLock::new(|| lines("PRONOUN_PEOPLE").strs());
static RELATION_ALIKE: LazyLock<HashMap<String, String>> = LazyLock::new(|| str_map(lines("RELATION_ALIKE")));

// `bot.memory.facts` (imported as `extract`).
fn facts(name: &str) -> &'static Value {
    data::get(&format!("bot.memory.facts.{name}"))
}
static SHE: LazyLock<HashSet<String>> = LazyLock::new(|| facts("SHE").str_set());
static HE: LazyLock<HashSet<String>> = LazyLock::new(|| facts("HE").str_set());
static KIN: LazyLock<Vec<String>> = LazyLock::new(|| facts("KIN").strs());
static KIN_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| facts("KIN_WORDS").strs());
static ALONE_PERSON: LazyLock<Vec<String>> = LazyLock::new(|| facts("ALONE_PERSON").strs());
static RELATIONS: LazyLock<Vec<String>> = LazyLock::new(|| facts("RELATIONS").strs());
static TITLE_GENDER: LazyLock<HashMap<String, String>> = LazyLock::new(|| str_map(facts("TITLE_GENDER")));

/// `_named_person`'s pattern for each relation word, in `KIN_WORDS + ALONE_PERSON` order.
static A_RELATION_NAMED: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    KIN_WORDS
        .iter()
        .chain(ALONE_PERSON.iter())
        .map(|relation| (relation.clone(), Re::new(&format!(r"\b{relation}(?:\s+in\s+laws?)?(?:'s|s)?\b"), I)))
        .collect()
});

// The patterns PyBot writes inline.
static SHE_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:she|her)(?:'s|s)?\b", 0));
static HE_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:he|his)(?:'s|s)?\b", 0));
static DASH_OR_SPACE: LazyLock<Re> = LazyLock::new(|| Re::new(r"[-\s]+", 0));
static IN_LAWS_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bin laws$", 0));
static S_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"s$", 0));
static A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", I));
static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static POSSESSIVE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?:'s|s'|s)$", 0));
static RELATION_SPLIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+and\s+|\s+|,", 0));
static HAVE_ANY_SPLIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s*(?:,|\bor\b|\band\b)\s*", 0));
static NAMES_SPLIT: LazyLock<Re> = LazyLock::new(|| Re::new(r",|\band\b|&", 0));
static NAMES_AFTER_CALLED: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"(?:\bcalled\b|\bnamed\b|,)\s*(?P<names>[^.!?]{3,60})$", I));
static NAMES_AFTER_ARE: LazyLock<Re> = LazyLock::new(|| {
    Re::new(&format!(r"\b(?:{})\s+(?:are|were)\s+(?P<names>[^.!?]{{3,60}})$", PEOPLE_MANY.as_str()), I)
});
static MY_OUR_THE: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:my|our|the)\s+", I));
static THEY_THEM: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:they|them)\b", 0));
static ANY_PRONOUN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:she|he|her|his|they|them)\b", 0));
static SHE_HER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:she|her)\b", 0));
static ONE_PRONOUN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:she|he|her|his)\b", 0));
static THEY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bthey\b", 0));
static WHAT_IS_USUAL: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:normal|typical|usual|common|expected)\b", 0));
static CHILDREN_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:kids?|children|child|family)\b", 0));
static A_PERSON_PRONOUN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:she|her|hers|he|him|his)\b", 0));
static PLACE_ENDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[.,;:!?]", 0));
static AN_ANIMAL_PRONOUN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:she|he|it|they|his|her|its|their)\b", 0));
static SHE_ASKED: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:she|her|hers)\b", 0));
static HE_ASKED: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:he|him|his)\b", 0));

static NONE: Value = Value::None;

/// `entry.get(key)` (None when missing, or when the entry is not a dict).
fn eget<'a>(entry: &'a Value, key: &str) -> &'a Value {
    entry.get(key).unwrap_or(&NONE)
}

/// `str(entry.get(key) or "")`.
fn sget(entry: &Value, key: &str) -> String {
    s_or(eget(entry, key))
}

/// `str(value or "")`.
fn s_or(value: &Value) -> String {
    if value.truthy() { py_str(value) } else { String::new() }
}

/// `isinstance(value, int)` (a bool is an int in Python).
fn is_int(value: &Value) -> bool {
    matches!(value, Value::Int(_) | Value::Bool(_))
}

/// `int(value)` for a value that is an int.
fn int_of(value: &Value) -> i64 {
    value.as_int().unwrap_or(0)
}

/// `s[:1].upper() + s[1:]`.
fn up1(s: &str) -> String {
    pt::upper_first(s)
}

/// `s[0].upper() + s[1:]`: an empty string is an IndexError.
fn up0(s: &str) -> String {
    if s.is_empty() {
        panic!("IndexError: string index out of range");
    }
    pt::upper_first(s)
}

/// `s[:1].lower() + s[1:]`.
fn low1(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().chain(c).collect(),
    }
}

/// `s.split()[0]`: a string with no word in it is an IndexError.
fn first_word(s: &str) -> &str {
    match pt::split_ref(s).first() {
        Some(word) => word,
        None => panic!("IndexError: list index out of range"),
    }
}

/// `bot.memory.user(memory)`: the user's facts, created on first use (a copy of the dict).
fn user(memory: &mut Memory) -> Value {
    if !memory.get("user").is_some_and(Value::is_dict) {
        memory.set("user", Value::dict());
    }
    memory.get("user").cloned().unwrap_or(Value::None)
}

/// `bot.memory.others(memory)`: facts about other people, created on first use (a copy).
fn others(memory: &mut Memory) -> Vec<Value> {
    if !matches!(memory.get("others"), Some(Value::List(_))) {
        memory.set("others", Value::list());
    }
    memory.get("others").map(|v| v.items().to_vec()).unwrap_or_default()
}

/// `user(memory).get(key) or []`, as the items of the list.
fn user_list(memory: &mut Memory, key: &str) -> Vec<Value> {
    let held = user(memory);
    let value = eget(&held, key);
    if value.truthy() { value.items().to_vec() } else { Vec::new() }
}

/// The memory as a Python-like dict value (for the contract functions that take one).
fn memory_value(memory: &Memory) -> Value {
    Value::from_plain(&serde_json::Value::Object(memory.to_plain()))
}

/// `pick(options, memory)` from `bot.compose` (no seed text).
fn pick(options: &[String], memory: &Memory) -> String {
    crate::compose::pick(options, &memory_value(memory), "")
}

/// A Python `try` whose `except Exception` gives a fallback. A function another workstream has
/// not ported yet is not an exception PyBot would have raised, so it passes through.
fn py_try<T>(f: impl FnOnce() -> T) -> Option<T> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(v) => Some(v),
        Err(e) => {
            let msg = e.downcast_ref::<String>().map(String::as_str).or_else(|| e.downcast_ref::<&str>().copied()).unwrap_or("");
            if msg.starts_with("UNPORTED") {
                std::panic::resume_unwind(e);
            }
            None
        }
    }
}

/// An argument for another file's contract parameter, whichever Rust type it has today: a
/// skeleton stub types a Python int default as whatever the trace saw.
trait PyIntArg<'a> {
    fn of(n: i64, as_value: &'a Value) -> Self;
}
impl<'a> PyIntArg<'a> for i64 {
    fn of(n: i64, _: &'a Value) -> Self {
        n
    }
}
impl<'a> PyIntArg<'a> for usize {
    fn of(n: i64, _: &'a Value) -> Self {
        n as usize
    }
}
impl<'a> PyIntArg<'a> for Option<i64> {
    fn of(n: i64, _: &'a Value) -> Self {
        Some(n)
    }
}
impl<'a> PyIntArg<'a> for &'a Value {
    fn of(_: i64, as_value: &'a Value) -> Self {
        as_value
    }
}
impl<'a> PyIntArg<'a> for &'a [String] {
    /// The stub's type cannot hold the int; the stub panics UNPORTED before reading it.
    fn of(_: i64, _: &'a Value) -> Self {
        &[]
    }
}

/// A list of lines another file's function returned, whichever Rust type its contract has.
trait PyLines {
    fn lines(self) -> Vec<String>;
}
impl PyLines for Value {
    fn lines(self) -> Vec<String> {
        self.strs()
    }
}
impl PyLines for Vec<String> {
    fn lines(self) -> Vec<String> {
        self
    }
}

/// `history.the_lines_before(memory)` with its default look-back.
fn the_lines_before(memory: &Memory) -> Vec<String> {
    let look_back = data::get("bot.memory.history.LOOK_BACK");
    crate::memory::history::the_lines_before(memory, PyIntArg::of(look_back.as_int().unwrap_or(0), look_back)).lines()
}

/// `history.is_their_news(line)` with its default word count.
fn is_their_news(line: &str) -> bool {
    static THREE: Value = Value::Int(3);
    crate::memory::history::is_their_news(line, PyIntArg::of(3, &THREE))
}

/// Python: bot/memory/relations.py:40 `_folded(text, memory=None)`
///
/// The line with chat-speak folded to plain words ("whats my name" -> "what is my name"), from
/// the understanding engine when it has read this very line, else read here. (A memory with no
/// reading in it is Python's `memory=None`.)
pub fn _folded(text: &str, memory: &Memory) -> String {
    let low = pt::squash(&pt::lower(text));
    if low.is_empty() {
        return String::new();
    }
    let tidy = |folded: &str| -> String {
        let plain = pt::squash(&pt::lower(&folded.replace(" | ", " ")));
        pt::strip_chars(&CHAT_TAIL.sub("", &plain), " ,.;:!?").to_string()
    };
    if let Some(seen) = &memory.understood {
        if pt::squash(&pt::lower(&seen.text)) == low && !seen.norm_text.is_empty() {
            return tidy(&seen.norm_text);
        }
    }
    let folded = py_try(|| crate::understanding::normalize::normalise(text).norm_text).unwrap_or_default();
    // The chat itself named after the ask is a wrapper, not content: every reader works on the
    // ask inside it, so it comes off here, once, for all of them.
    tidy(if folded.is_empty() { &low } else { &folded })
}

/// Python: bot/memory/relations.py:63 `_person_from_pronoun(text, memory, named_here=())`
///
/// Who a bare "she"/"he" in this line is: the last person named whose pronoun fits.
pub fn _person_from_pronoun(text: &str, memory: &mut Memory, named_here: &[String]) -> String {
    let low = pt::lower(text);
    let mut people: Vec<String> =
        user_list(memory, "people").iter().filter_map(|who| who.as_str().map(str::to_string)).collect();
    // A person the store already keeps a fact about is someone "she"/"he"/"they" can point at,
    // even when their name went into the fact rather than into the list of people.
    people.extend(others(memory).iter().filter(|entry| eget(entry, "who").truthy()).map(|entry| py_str(eget(entry, "who"))));
    let people: Vec<String> = named_here.iter().cloned().chain(people).collect();
    let genders: [(&Re, &HashSet<String>); 2] = [(&SHE_SAID, &SHE), (&HE_SAID, &HE)];
    for (pronoun, fits) in genders {
        // "he's 8" and "shes 5" are the same pronoun as "he" and "she".
        if pronoun.is_match(&low) {
            for who in _newest_first(&people, memory) {
                // A plural is not one person.
                if who.ends_with('s') && !fits.contains(&who) {
                    continue;
                }
                // "mother in law" is a she though the phrase is not a word in the list.
                if pronoun_fits(&who, fits) {
                    return who;
                }
            }
            // A "he" when the only people in the chat are her mum and her sister is not one of
            // them, and answering about the newest one anyway puts a fact on the wrong person.
            return String::new();
        }
    }
    _newest_first(&people, memory).into_iter().next().unwrap_or_default()
}

/// Python: bot/memory/relations.py:98 `_gender_of(who)`
///
/// The gender a person word carries, or "" when it carries none ("partner", "kid").
pub fn _gender_of(who: &str) -> String {
    let word = pt::lower(pt::strip(who));
    if word.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = DASH_OR_SPACE.split(&word).into_iter().filter(|part| !part.is_empty()).collect();
    for part in std::iter::once(&word).chain(parts.iter()) {
        if SHE.contains(part) && HE.contains(part) {
            return String::new(); // partner, friend, colleague: either
        }
        if SHE.contains(part) {
            return "she".to_string();
        }
        if HE.contains(part) {
            return "he".to_string();
        }
    }
    // "stepmum", "granddad", "godmother": the gender sits at the end of a word this file has no
    // name for. (The longest ending wins in PyBot; every root of one list gives the same answer.)
    let roots: [(&HashSet<String>, &str); 2] = [(&SHE, "she"), (&HE, "he")];
    for (roots, gender) in roots {
        if roots.iter().any(|root| pt::len(root) > 2 && word.ends_with(root.as_str())) {
            return gender.to_string();
        }
    }
    String::new()
}

/// `_pronoun_fits` with the list of words itself (`extract.SHE` or `extract.HE`).
fn pronoun_fits(who: &str, fits: &HashSet<String>) -> bool {
    let who = pt::lower(pt::strip(who));
    if fits.contains(&who) {
        return true;
    }
    let gender = _gender_of(&who);
    if gender.is_empty() {
        return true;
    }
    // `(extract.SHE if gender == "she" else extract.HE) is fits`
    *(if gender == "she" { &*SHE } else { &*HE }) == *fits
}

/// Python: bot/memory/relations.py:120 `_pronoun_fits(who, fits)`
///
/// Whether a pronoun of that gender can point at this person. `fits` is `extract.SHE` or
/// `extract.HE`; PyBot tells them apart by identity, read here by their words.
pub fn _pronoun_fits(who: &str, fits: &[String]) -> bool {
    pronoun_fits(who, &fits.iter().cloned().collect())
}

/// Python: bot/memory/relations.py:136 `_newest_first(people, memory)`
///
/// The people, the one named last in the chat first, each kept once.
pub fn _newest_first(people: &[String], memory: &Memory) -> Vec<String> {
    let mut order: Vec<String> = Vec::new();
    let log = memory.get("facts_log").filter(|v| v.truthy()).map(|v| v.items().to_vec()).unwrap_or_default();
    for entry in log.iter().rev() {
        if !entry.is_dict() {
            continue;
        }
        let text = pt::lower(&sget(entry, "text"));
        for who in people {
            if !who.is_empty()
                && !order.contains(who)
                && Re::new(&format!(r"\b{}s?\b", pyre::escape(who)), 0).is_match(&text)
            {
                order.push(who.clone());
            }
        }
    }
    for who in people.iter().rev() {
        if !who.is_empty() && !order.contains(who) {
            order.push(who.clone());
        }
    }
    order
}

/// Python: bot/memory/relations.py:165 `_named_person(low, memory=None)`
///
/// The relation a line names as a person, or "" when it names their thing ("how old is my sons
/// school" names nobody). A word wider than the store's oldest list only counts when the chat
/// was told about that person - which is only asked when there is a memory to ask.
pub fn _named_person(low: &str, mut memory: Option<&mut Memory>) -> String {
    // A pair of people named as one ("my partner's mum") is the person the store filed the age
    // against, and it is looked for first.
    let chained = extract::a_chained_person(low);
    if !chained.is_empty() {
        return extract::a_person_word(&chained);
    }
    for (relation, pattern) in A_RELATION_NAMED.iter() {
        // "my mother in law" is a person of her own and not a mother: the second words are kept.
        for found in pattern.finditer(low) {
            if PERSON_AFTER.match_(&low[found.end()..]).is_none() {
                continue;
            }
            if let Some(memory) = memory.as_deref_mut() {
                if !RELATIONS.contains(relation) && !_a_person_in_the_store(memory, relation) {
                    continue;
                }
            }
            let name = pt::squash(&pt::lower(found.as_str()).replace("'s", ""));
            let name = IN_LAWS_END.sub("in law", &name);
            let name = if !name.ends_with("law") { S_END.sub("", &name) } else { name };
            return if name == *relation { relation.clone() } else { name };
        }
    }
    String::new()
}

/// Python: bot/memory/relations.py:204 `_a_person_in_the_store(memory, who)`
///
/// Whether the chat was ever told about this person, by the word the line used.
pub fn _a_person_in_the_store(memory: &mut Memory, who: &str) -> bool {
    let who = pt::lower(who);
    if who.is_empty() {
        return false;
    }
    for entry in others(memory) {
        if pt::lower(&sget(&entry, "who")) == who {
            return true;
        }
    }
    user_list(memory, "people").iter().any(|person| pt::lower(&py_str(person)) == who)
}

/// Python: bot/memory/relations.py:239 `_continues_an_about_ask(memory, low)`
///
/// True when the line is another person named after an ask about a person.
pub fn _continues_an_about_ask(memory: &Memory, low: &str) -> bool {
    let line = pt::strip(low);
    if ANOTHER_PERSON.match_(line).is_none() || _named_person(line, None).is_empty() {
        return false;
    }
    let mut users: Vec<&str> =
        _every_line(memory).iter().filter(|turn| !turn.user.is_empty()).map(|turn| turn.user.as_str()).collect();
    // The line being answered may already be in the log, so the ask before it is the last one
    // that is not this line.
    let here = pt::lower(line);
    while users.last().is_some_and(|last| pt::lower(pt::strip(last)) == here) {
        users.pop();
    }
    users.last().is_some_and(|last| ABOUT_PERSON_ASK.is_match(&pt::lower(last)))
}

/// Python: bot/memory/relations.py:284 `_a_word_the_store_knows(memory, word)`
///
/// Whether the word is the name of a person or an animal this chat was told about.
pub fn _a_word_the_store_knows(memory: &mut Memory, word: &str) -> bool {
    let word = pt::lower(word);
    if word.is_empty() {
        return false;
    }
    for entry in others(memory) {
        if word == pt::lower(&sget(&entry, "name")) || word == pt::lower(&sget(&entry, "who")) {
            return true;
        }
    }
    for pet in user_list(memory, "pets") {
        if pet.is_dict() && pt::lower(&sget(&pet, "name")) == word {
            return true;
        }
    }
    user_list(memory, "people").iter().any(|person| pt::lower(&py_str(person)) == word)
}

/// Python: bot/memory/relations.py:298 `_age_is_about_a_person(low, memory)`
///
/// Whether an age ask is about somebody in the chat rather than about the world: the person
/// has to be what the ask asks ("what age do kids stop napping" is a rule).
pub fn _age_is_about_a_person(low: &str, memory: &mut Memory) -> bool {
    if !_the_place_end(low).1.is_empty() {
        // "how old is the eldest", "how old is the little one": a child named by their place in
        // the family is a person the chat can have been told an age for.
        return true;
    }
    let Some(ask) = AGE_ASK.search(low) else {
        return true;
    };
    if !extract::a_chained_person(low).is_empty() {
        // "how old is my partner's mum" is a person, not a rule about mothers.
        return true;
    }
    let mut rest = pt::strip(&low[ask.end()..]);
    while let Some(adverb) = AGE_ADVERB.match_(rest) {
        rest = pt::strip(&rest[adverb.end()..]);
    }
    if let Some(rule) = AGE_RULE_WORD.match_(rest) {
        // "how old did you say he was" asks the store what it was told, so the asking verb
        // keeps the ask about the person named after it.
        let after = pt::strip(&rest[rule.end()..]);
        if AGE_YOU_SAY.match_(after).is_none() {
            return false;
        }
        rest = after;
    }
    if let Some(copula) = AGE_COPULA.match_(rest) {
        rest = pt::strip(&rest[copula.end()..]);
    }
    if let Some(determiner) = AGE_DETERMINER.match_(rest) {
        // "a kitten" and "an animal" name the kind, not one of theirs.
        if matches!(pt::lower(determiner.as_str()).as_str(), "a" | "an") {
            return false;
        }
        rest = pt::strip(&rest[determiner.end()..]);
    }
    let Some(word) = A_WORD.match_(rest) else {
        return false;
    };
    let subject = word.as_str();
    if AGE_PERSON_WORD.match_(subject).is_some() || KIN_WORDS.contains(&pt::lower(subject)) {
        return true;
    }
    if _PET_WORD.fullmatch(subject).is_some() || _PET_WORD.fullmatch(pt::rstrip_chars(subject, "s")).is_some() {
        return true;
    }
    _a_word_the_store_knows(memory, subject)
}

/// Python: bot/memory/relations.py:398 `_tidy_name(value)`
///
/// "rufus" is the name they mean, and a name reads "Rufus" at the start of a sentence.
pub fn _tidy_name(value: &Value) -> String {
    tidy_name(&s_or(value))
}

fn tidy_name(value: &str) -> String {
    let value = pt::strip_chars(value, " .,!?;:");
    if value.chars().next().is_some_and(char::is_lowercase) { up1(value) } else { value.to_string() }
}

/// Python: bot/memory/relations.py:404 `_same_as_last(reply, memory)`
///
/// True when this exact sentence was the bot's line in the turn just before.
pub fn _same_as_last(reply: &str, memory: &Memory) -> bool {
    for turn in memory.turns.iter().rev() {
        let said = pt::strip(&turn.bot);
        if !said.is_empty() {
            return said == pt::strip(reply);
        }
    }
    false
}

/// Python: bot/memory/relations.py:414 `_animal_family(animal)`
///
/// The animal a pet word belongs to: a puppy is a dog, a kitten is a cat.
pub fn _animal_family(animal: &str) -> String {
    let word = pt::lower(animal);
    let family = match word.as_str() {
        "puppy" | "dog" => "dog",
        "kitten" | "cat" => "cat",
        "bunny" | "rabbit" => "rabbit",
        "goldfish" | "fish" => "fish",
        "pony" | "horse" => "horse",
        "bird" | "parrot" => "parrot",
        _ => return word,
    };
    family.to_string()
}

/// Python: bot/memory/relations.py:423 `_a(word)`
pub fn _a(word: &str) -> String {
    // `word[:1].lower() in "aeiou"`: true for an empty word too.
    let first: String = word.chars().take(1).flat_map(char::to_lowercase).collect();
    format!("{}{}", if "aeiou".contains(first.as_str()) { "an " } else { "a " }, word)
}

/// Python: bot/memory/relations.py:498 `_their_taste(memory, low)`
///
/// What a person the chat knows likes, said back for an ask about it. Only a fact the store
/// holds is answered, and only for a person it can name.
pub fn _their_taste(memory: &mut Memory, low: &str) -> Option<String> {
    let mut how = String::new();
    let mut said = [&*THEIR_TASTE_A_YES_NO, &*THEIR_TASTE_WHAT, &*THEIR_TASTE_INTO]
        .into_iter()
        .find_map(|pattern| pattern.search(low));
    if said.is_none() {
        // The same fact asked for with the "not" in front of the person: the answer is what is
        // on their dislike list.
        said = [&*THEIR_TASTE_NOT, &*THEIR_TASTE_NOT_INTO].into_iter().find_map(|pattern| pattern.search(low));
        if said.is_some() {
            how = "not like".to_string();
        }
    }
    let said = said?;
    if how.is_empty() {
        how = pt::lower(said.name_str("how"));
    }
    let thing = pt::lower(pt::strip(said.name_str("thing")));
    let mut who = pt::lower(said.name_str("who"));
    let pronoun = pt::lower(said.name_str("pronoun"));
    let mut known: Option<Value> = None;
    if !who.is_empty() {
        known = others(memory).into_iter().find(|entry| pt::lower(&sget(entry, "who")) == who);
    } else if !pronoun.is_empty() {
        let fits = if pronoun == "she" || pronoun == "her" { &*SHE } else { &*HE };
        known = others(memory).into_iter().rev().find(|entry| {
            fits.contains(&pt::lower(&sget(entry, "who")))
                && (eget(entry, "likes").truthy() || eget(entry, "dislikes").truthy())
        });
        if let Some(known) = &known {
            who = pt::lower(&sget(known, "who"));
        }
    }
    let known = known?;
    let listed = |key: &str| -> Vec<String> {
        let items = eget(&known, key);
        if !items.truthy() {
            return Vec::new();
        }
        items.items().iter().map(py_str).filter(|item| !pt::strip(item).is_empty()).collect()
    };
    let likes = listed("likes");
    let dislikes = listed("dislikes");
    if !thing.is_empty() {
        // "does my mum like gardening" is about that one thing, and the answer is about it too.
        for item in &likes {
            if _same_thing(item, &thing) {
                return Some(_they_like_sentence(&who, std::slice::from_ref(item), "yes", &pronoun));
            }
        }
        for item in &dislikes {
            if _same_thing(item, &thing) {
                return Some(_they_like_sentence(&who, std::slice::from_ref(item), "no", &pronoun));
            }
        }
        return None;
    }
    if matches!(how.as_str(), "hate" | "dislike" | "not like") {
        if dislikes.is_empty() {
            if !likes.is_empty() {
                // The list she had was taken back, so the ask is answered with what is on the
                // other list now - the honest answer is nothing.
                let said = _they_like_sentence(&who, &likes, "yes", &pronoun);
                return Some(format!("Nothing now \u{2014} {}", low1(&said)));
            }
            return Some(format!("You haven't told me what your {who} doesn't like yet."));
        }
        return Some(_they_like_sentence(&who, &dislikes, "no", &pronoun));
    }
    if likes.is_empty() {
        // The store holds the person but not this list, and the honest line is better than the
        // search shrug.
        return Some(format!("You haven't told me what your {who} likes yet."));
    }
    Some(_they_like_sentence(&who, &likes, "yes", &pronoun))
}

/// `one.split()[-1]`.
fn last_word(s: &str) -> &str {
    match pt::split_ref(s).last() {
        Some(word) => word,
        None => panic!("IndexError: list index out of range"),
    }
}

/// Python: bot/memory/relations.py:571 `_same_thing(one, other)`
///
/// Whether two ways of naming a taste are the same thing ("cooking", "cook").
pub fn _same_thing(one: &str, other: &str) -> bool {
    let one = pt::lower(one);
    let one = pt::strip(&one);
    let other = pt::lower(other);
    let other = pt::strip(&other);
    if one.is_empty() || other.is_empty() {
        return false;
    }
    if one == other {
        return true;
    }
    crate::memory::topics::stem(last_word(one)) == crate::memory::topics::stem(last_word(other))
}

/// Python: bot/memory/relations.py:586 `_plural_person(who)`
pub fn _plural_person(who: &str) -> bool {
    let who = pt::lower(who);
    _PLURAL.contains(&who) || (who.ends_with('s') && who != "boss" && who != "christmas")
}

/// Python: bot/memory/relations.py:591 `_they_like_sentence(who, things, how, pronoun='')`
///
/// What they like, said back short: "Your sister loves cooking." / "Your son doesn't like
/// mushrooms." The person comes first when the store can name them, and the pronoun they used
/// when it cannot.
pub fn _they_like_sentence(who: &str, things: &[String], how: &str, pronoun: &str) -> String {
    let listed = things[..things.len().min(2)].join(" and ");
    let (head, some) = if !who.is_empty() {
        (format!("Your {who} "), _plural_person(who))
    } else {
        let head = match pronoun {
            "they" => "They ",
            "she" => "She ",
            _ => "He ",
        };
        (head.to_string(), pronoun == "they")
    };
    if how == "no" {
        return format!("{head}{}{listed}.", if some { "don't like " } else { "doesn't like " });
    }
    format!("{head}{}{listed}.", if some { "love " } else { "loves " })
}

/// Python: bot/memory/relations.py:621 `_their_job(memory, low)`
///
/// What a person the chat knows does, said back for an ask about it. Only a role the store
/// holds is answered; the person is the one the ask names, or the one a pronoun points at.
pub fn _their_job(memory: &mut Memory, low: &str) -> Option<String> {
    let found = THEIR_JOB.search(low)?;
    let first_of = |names: [&str; 3]| -> String {
        pt::lower(names.into_iter().map(|name| found.name_str(name)).find(|said| !said.is_empty()).unwrap_or(""))
    };
    let mut who = first_of(["who", "who2", "who3"]);
    let pronoun = first_of(["pronoun", "pronoun2", "pronoun3"]);
    let mut known: Option<Value> = None;
    if !who.is_empty() {
        known = _a_kept_person(memory, &who);
        if let Some(known) = &known {
            who = pt::lower(&sget(known, "who"));
        }
    } else if !pronoun.is_empty() && pronoun != "they" {
        let fits = if pronoun == "she" { &*SHE } else { &*HE };
        known = others(memory)
            .into_iter()
            .rev()
            .find(|entry| fits.contains(&pt::lower(&sget(entry, "who"))) && eget(entry, "job").truthy());
        if let Some(known) = &known {
            who = pt::lower(&sget(known, "who"));
        }
    }
    let Some(known) = known else {
        let who_kept = !who.is_empty()
            && pronoun.is_empty()
            && (KIN.contains(&who) || _kin_forms(&who).iter().any(|form| KIN.contains(form)));
        if who_kept {
            // A question about somebody the store has nothing on is answered honestly, the same
            // way it answers a name or an age it was never told.
            return Some(format!("You haven't told me what your {who} does yet."));
        }
        return None;
    };
    let job = sget(&known, "job");
    let job = pt::strip(&job);
    if job.is_empty() {
        return if !who.is_empty() { Some(format!("You haven't told me what your {who} does yet.")) } else { None };
    }
    if !who.is_empty() && _plural_person(&who) {
        return Some(format!("Your {who} work as {}.", _a(job)));
    }
    if !who.is_empty() {
        return Some(format!("Your {who} is {}.", _a(job)));
    }
    Some(format!("{}{}.", if pronoun == "she" { "She's " } else { "He's " }, _a(job)))
}

/// Python: bot/memory/relations.py:665 `_kin_forms(who)`
///
/// The relation words a possessive ask can name: "sisters" and "sister's" are "sister".
pub fn _kin_forms(who: &str) -> HashSet<String> {
    let word = pt::lower(who);
    let word = pt::strip(&word).to_string();
    let mut forms = HashSet::from([word.clone()]);
    let bare = POSSESSIVE_END.sub("", &word);
    if !bare.is_empty() && bare != word {
        forms.insert(bare);
    }
    forms.insert(format!("{word}s"));
    forms
}

/// Python: bot/memory/relations.py:680 `_a_kept_person(memory, who)`
///
/// The entry the store holds for the person an ask names, if it holds one.
pub fn _a_kept_person(memory: &mut Memory, who: &str) -> Option<Value> {
    let forms = _kin_forms(who);
    others(memory).into_iter().find(|entry| forms.contains(&pt::lower(&sget(entry, "who"))))
}

/// Python: bot/memory/relations.py:689 `_same_name_however_they_wrote_it(entry, line)`
///
/// Whether the line names the person, under the name they have now or the one put right.
pub fn _same_name_however_they_wrote_it(entry: &Value, line: &str) -> bool {
    if same_name(&sget(entry, "name"), line) {
        return true;
    }
    let was = eget(entry, "was");
    was.truthy() && was.items().iter().any(|old| same_name(&s_or(old), line))
}

/// Python: bot/memory/relations.py:701 `_same_name(known, line)`
///
/// Whether the person the store knows is the one the line names: "how old is janet". The line
/// arrives folded, so both sides are folded and compared.
pub fn _same_name(known: &Value, line: &str) -> bool {
    same_name(&s_or(known), line)
}

fn same_name(known: &str, line: &str) -> bool {
    let known = pt::lower(pt::strip(known));
    if known.is_empty() {
        return false;
    }
    let nobody = Memory::new();
    for token in WORDS.findall_str(&pt::lower(line)) {
        if token == known {
            return true;
        }
        if _folded(&token, &nobody) == _folded(&known, &nobody) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/relations.py:721 `_more_than_one(who)`
///
/// Whether the word for the person takes "are": "my kids are 4 and 7", "my mum is 72".
pub fn _more_than_one(who: &str) -> bool {
    _plural_person(who)
}

/// Python: bot/memory/relations.py:730 `_canon_relation(word)`
///
/// The relation a word names, in the word the store files it under: "mother" is "mum".
pub fn _canon_relation(word: &str) -> String {
    let word = pt::lower(pt::strip_chars(word, " .,"));
    let word = RELATION_ALIKE.get(&word).cloned().unwrap_or(word);
    if let Some(one) = word.strip_suffix('s') {
        if let Some(alike) = RELATION_ALIKE.get(one) {
            return alike.clone();
        }
        if CHILD_RELATIONS.iter().any(|relation| relation == one) {
            return one.to_string();
        }
    }
    word
}

/// Python: bot/memory/relations.py:741 `_relations_of(who)`
///
/// The relations a stored word for people stands for: "kids" is every child relation.
pub fn _relations_of(who: &str) -> Vec<String> {
    let mut found = Vec::new();
    for word in RELATION_SPLIT.split(&pt::lower(pt::strip(who))) {
        let canon = _canon_relation(&word);
        if canon.is_empty() {
            continue;
        }
        if matches!(canon.as_str(), "child" | "children" | "kid" | "kids") {
            found.extend(CHILD_RELATIONS.iter().cloned());
        } else if canon == "parent" || canon == "parents" {
            found.extend(GROUP_OF["parents"].iter().cloned());
        } else {
            found.push(canon);
        }
    }
    found
}

/// `_in_the_group` with the group as the words themselves.
fn in_the_group(who: &str, group: &[String]) -> bool {
    _relations_of(who).iter().any(|word| group.contains(word))
}

/// Python: bot/memory/relations.py:757 `_in_the_group(who, group)`
///
/// Whether a relation the store holds is one of the people the ask named.
pub fn _in_the_group(who: &str, group: &Value) -> bool {
    in_the_group(who, &group.strs())
}

/// Python: bot/memory/relations.py:762 `_a_list_of_names(names)`
///
/// Three names read "Jack, Emma and Ada", not "Jack and Emma and Ada".
pub fn _a_list_of_names(names: &[String]) -> String {
    let said: Vec<&str> = names.iter().map(String::as_str).filter(|name| !name.is_empty()).collect();
    if said.len() < 3 {
        return said.join(" and ");
    }
    format!("{} and {}", said[..said.len() - 1].join(", "), said[said.len() - 1])
}

/// Python: bot/memory/relations.py:770 `_mentioned_the_group(memory, group, word)`
///
/// Whether the chat was told the group exists, even with no name filed for any of them.
pub fn _mentioned_the_group(memory: &mut Memory, group: &[String], word: &str) -> bool {
    let held = user(memory);
    if matches!(word, "children" | "kids" | "childs") && eget(&held, "kids").truthy() {
        return true;
    }
    let people = eget(&held, "people");
    if people.truthy() && people.items().iter().any(|person| in_the_group(&s_or(person), group)) {
        return true;
    }
    // What the store holds is not the whole of what was said: only a group nobody ever
    // mentioned may be answered "you haven't told me about your children".
    _the_group_was_said(word, memory)
}

/// `(CHILD_RELATIONS + GROUP_OF[a] + GROUP_OF[b] + ...)`.
fn the_family(groups: &[&str]) -> Vec<String> {
    let mut family = CHILD_RELATIONS.clone();
    for name in groups {
        family.extend(GROUP_OF[*name].iter().cloned());
    }
    family
}

/// Python: bot/memory/relations.py:795 `_whether_they_have(memory, low)`
///
/// Whether they have any of a kind of person, answered from the people on file. The answer is
/// what was said and no more - it does not guess that a missing brother does not exist.
pub fn _whether_they_have(memory: &mut Memory, low: &str) -> Option<String> {
    let found_ask = HAVE_ANY.match_(pt::strip_chars(low, " ,.!?;:"))?;
    let words: Vec<String> =
        HAVE_ANY_SPLIT.split(&pt::lower(found_ask.name_str("what"))).into_iter().filter(|word| !word.is_empty()).collect();
    if words.is_empty() {
        return None;
    }
    let held: Vec<Value> = others(memory).into_iter().filter(Value::is_dict).collect();
    // A relation they named without a name ("i have a sister") is kept as a word and not as a
    // person, so it still counts as a sister the chat has been told about.
    let plain: Vec<String> = user_list(memory, "people").iter().map(|word| pt::lower(&py_str(word))).collect();
    // `counts`: a dict, in the order its words were first set.
    let mut counts: Vec<(String, i64)> = Vec::new();
    let mut set_count = |counts: &mut Vec<(String, i64)>, word: &str, count: i64| match counts.iter_mut().find(|(w, _)| w == word) {
        Some(slot) => slot.1 = count,
        None => counts.push((word.to_string(), count)),
    };
    let mut found: Vec<(String, Value)> = Vec::new();
    let mut said_without_a_name: Vec<String> = Vec::new();
    let mut empty: Vec<String> = Vec::new();
    for word in &words {
        let group: Vec<String> = if word == "family" {
            the_family(&["parents", "siblings", "grandparents"])
        } else {
            match GROUP_OF.get(word).filter(|group| !group.is_empty()) {
                Some(group) => group.clone(),
                None => vec![_canon_relation(word)],
            }
        };
        if let Some(mine) = held.iter().find(|entry| in_the_group(&s_or(eget(entry, "who")), &group)) {
            found.push((word.clone(), mine.clone()));
            continue;
        }
        let hinted: Vec<String> = plain.iter().filter(|one| in_the_group(one, &group)).map(|one| _canon_relation(one)).collect();
        let held_user = user(memory);
        let count = eget(&held_user, "kids");
        if !hinted.is_empty() {
            said_without_a_name.extend(hinted);
            continue;
        }
        if matches!(count, Value::Int(0)) && in_the_group("child", &group) {
            // A count of zero is an answer, not a gap.
            said_without_a_name.push(word.clone());
            set_count(&mut counts, word, 0);
            continue;
        }
        if is_int(count) && int_of(count) > 0 && in_the_group("child", &group) {
            // "i have two kids" told the chat how many there are, so "do i have any children"
            // is answered with the number and the name that is still missing.
            let word = if matches!(word.as_str(), "kids" | "children" | "childs") { word.as_str() } else { "child" };
            said_without_a_name.push(word.to_string());
            set_count(&mut counts, word, int_of(count));
            continue;
        }
        empty.push(word.clone());
    }
    if found.is_empty() && said_without_a_name.is_empty() {
        let asked = _a_plural_list(&words);
        return Some(pick(
            &[
                format!("You haven't told me about {} yet.", _a_or_any(&words)),
                format!("Nothing on your {asked} yet - tell me and I'll keep it."),
            ],
            memory,
        ));
    }
    let mut said: Vec<String> = Vec::new();
    // `list(dict.fromkeys(said_without_a_name))`
    let mut bare: Vec<String> = Vec::new();
    for word in &said_without_a_name {
        if !bare.contains(word) {
            bare.push(word.clone());
        }
    }
    for (word, entry) in &found {
        let who_kept = eget(entry, "who");
        let who = if who_kept.truthy() { py_str(who_kept) } else { word.clone() };
        let who = pt::strip(&who).to_string();
        let name = sget(entry, "name");
        let name = pt::strip(&name);
        let names_kept = eget(entry, "names");
        let names: Vec<String> = if names_kept.truthy() {
            names_kept.items().iter().map(py_str).filter(|one| !pt::strip(one).is_empty()).collect()
        } else {
            Vec::new()
        };
        if name.is_empty() && names.len() > 1 {
            said.push(format!("your {who} are called {}", _a_list_of_names(&names)));
        } else if !name.is_empty() {
            said.push(format!("your {who} {}", _called(name)));
        } else if !bare.contains(&who) {
            bare.push(who);
        }
    }
    let mut line = String::new();
    if !said.is_empty() {
        line = format!("You've mentioned {}", _a_list_of_names(&said));
    }
    if !bare.is_empty() {
        let none_of_them = !counts.is_empty() && !counts.iter().any(|(_, count)| *count != 0);
        let how_many = counts.iter().map(|(_, count)| *count).find(|count| *count != 0).unwrap_or(0);
        let gap = if line.is_empty() { "" } else { " " };
        if none_of_them {
            // Said as none, not as a number that never arrived.
            line += &format!("{gap}You told me you don't have any {}.", _a_list_of_words(&bare));
        } else {
            let said_them = if how_many != 0 {
                format!("{how_many} {}", bare[0])
            } else {
                _a_list_of_words(&bare.iter().map(|word| _with_article(word)).collect::<Vec<_>>())
            };
            let names = if bare.len() > 1 || how_many > 1 {
                "their names".to_string()
            } else {
                format!("{} name", HER_WORD.get(&bare[0]).map(String::as_str).unwrap_or("their"))
            };
            line += &format!("{gap}You've told me you have {said_them}, but not {names} yet.");
        }
    }
    if !empty.is_empty() {
        let tail = format!("no {} that you've told me about", _a_list_of_words(&empty));
        if !line.is_empty() {
            line = format!("{} - and {tail}.", pt::rstrip_chars(&line, "."));
        } else {
            line = format!("You haven't mentioned any {} yet.", _a_list_of_words(&empty));
        }
    }
    Some(if line.ends_with('.') { line } else { line + "." })
}

/// `[str(word) for word in words if str(word)]`.
fn said_words(words: &[String]) -> Vec<&str> {
    words.iter().map(String::as_str).filter(|word| !word.is_empty()).collect()
}

/// Python: bot/memory/relations.py:893 `_a_list_of_words(words)`
///
/// A list of words for people, read "a sister and a brother".
pub fn _a_list_of_words(words: &[String]) -> String {
    let said = said_words(words);
    match said.len() {
        0 | 1 => said.concat(),
        2 => format!("{} and {}", said[0], said[1]),
        n => format!("{} and {}", said[..n - 1].join(", "), said[n - 1]),
    }
}

/// Python: bot/memory/relations.py:903 `_more_than_one_word(word)`
///
/// Whether a word for people is a plural one: "brothers", "children".
pub fn _more_than_one_word(word: &str) -> bool {
    let word = pt::lower(pt::strip(word));
    word.ends_with('s') || matches!(word.as_str(), "children" | "kids" | "family")
}

/// Python: bot/memory/relations.py:909 `_with_article(word)`
///
/// One kind of person, with the article the word takes: "a brother", "any sisters".
pub fn _with_article(word: &str) -> String {
    format!("{}{word}", if _more_than_one_word(word) { "any " } else { "a " })
}

/// Python: bot/memory/relations.py:915 `_a_or_any(words)`
///
/// An ask about a kind of person reads "a brother" or "any brothers or sisters".
pub fn _a_or_any(words: &[String]) -> String {
    let said = said_words(words);
    if said.len() == 1 {
        return _with_article(said[0]);
    }
    let Some(first) = said.first() else {
        panic!("IndexError: list index out of range");
    };
    let rest = said[1..].join(" or ");
    format!("{}{first} or {rest}", if _more_than_one_word(first) { "any " } else { "a " })
}

/// Python: bot/memory/relations.py:925 `_a_plural_list(words)`
///
/// Two or three kinds of person read "brothers or sisters", not "and".
pub fn _a_plural_list(words: &[String]) -> String {
    let said = said_words(words);
    match said.len() {
        0 => String::new(),
        1 => said[0].to_string(),
        2 => format!("{} or {}", said[0], said[1]),
        n => format!("{} or {}", said[..n - 1].join(", "), said[n - 1]),
    }
}

/// The names in the words that follow "called", "named" or a comma: plain words of 2 to 14
/// letters that are nobody's kin word.
fn the_plain_names(said: &str) -> Vec<String> {
    NAMES_SPLIT
        .split(said)
        .iter()
        .map(|one| pt::strip_chars(one, " .,").to_string())
        .filter(|one| !one.is_empty())
        .filter(|one| (2..=14).contains(&pt::len(one)) && pt::is_alpha(one) && !KIN_WORDS.contains(&pt::lower(one)))
        .collect()
}

/// Python: bot/memory/relations.py:944 `_the_group_names_typed(word, memory)`
///
/// The names a line of theirs gave for a group, as a list: "my kids are tom and ada". Only two
/// or three names count, and the last line that gives them wins.
pub fn _the_group_names_typed(word: &str, memory: &Memory) -> Vec<String> {
    let mut said: Vec<String> = Vec::new();
    let has_kids = memory.get("user").filter(|v| v.truthy()).is_some_and(|user| eget(user, "kids").truthy());
    for turn in &memory.turns {
        let line = &turn.user;
        let Some(found) = GROUP_NAMES_TYPED.search(line) else {
            // "i have two kids" then "they are called tom and ada": the pronoun gives the names
            // for the group the line before named.
            let names = _the_names_a_pronoun_gave(line);
            if !names.is_empty() && has_kids {
                said = names;
            }
            continue;
        };
        let group = GROUP_OF.get(word).cloned().unwrap_or_default();
        if !in_the_group(&pt::lower(found.name_str("who")), &group) {
            continue;
        }
        let names = the_plain_names(found.name_str("names"));
        if (2..=3).contains(&names.len()) {
            said = names;
        }
    }
    said
}

/// Python: bot/memory/relations.py:981 `_the_names_a_pronoun_gave(line)`
///
/// The names a "they are called ..." line gives, as a list, or [].
pub fn _the_names_a_pronoun_gave(line: &str) -> Vec<String> {
    match THEY_ARE_NAMED.search(pt::strip(line)) {
        Some(found) => _the_names_from(found.name_str("names")),
        None => Vec::new(),
    }
}

/// Python: bot/memory/relations.py:989 `_the_names_from(said)`
///
/// Two or three names out of the words that follow "called", "named" or a comma.
pub fn _the_names_from(said: &str) -> Vec<String> {
    let names = the_plain_names(said);
    if (2..=3).contains(&names.len()) { names } else { Vec::new() }
}

/// Python: bot/memory/relations.py:1027 `_the_several_named_before(memory)`
///
/// The newest earlier line that named more than one person: (word, names, phrase).
pub fn _the_several_named_before(memory: &Memory) -> (String, Vec<String>, String) {
    for line in the_lines_before(memory).iter().rev() {
        // The line has to be one that named them, not one that asked about them.
        if !is_their_news(line) {
            continue;
        }
        let found = SEVERAL_PEOPLE_NAMED.search(line);
        let names = _the_names_from(&_the_names_tail(line));
        let pair = if found.is_none() { TWO_OF_KIN.search(line) } else { None };
        if found.is_none() && pair.is_none() && names.is_empty() {
            continue;
        }
        let mut word = String::new();
        let mut phrase = String::new();
        if let Some(found) = &found {
            let said = ["counted", "group", "bare"].into_iter().map(|name| found.name_str(name)).find(|said| !said.is_empty());
            word = pt::lower(said.unwrap_or(""));
            // The words the line used, as it used them.
            phrase = pt::squash(found.as_str());
        }
        if let Some(pair) = &pair {
            let (one, other) = (pt::lower(pair.name_str("a")), pt::lower(pair.name_str("b")));
            word = format!("{one} and {other}");
            phrase = format!("your {one} and your {other}");
        }
        return (word, names, phrase);
    }
    (String::new(), Vec::new(), String::new())
}

/// Python: bot/memory/relations.py:1061 `_the_names_tail(line)`
///
/// The words after "called", "named", a comma or the "are" of a group, where names go.
pub fn _the_names_tail(line: &str) -> String {
    if let Some(found) = NAMES_AFTER_CALLED.search(line) {
        return found.name_str("names").to_string();
    }
    NAMES_AFTER_ARE.search(line).map(|found| found.name_str("names").to_string()).unwrap_or_default()
}

/// `_a_list_of_names([_called(one) for one in names])`.
fn the_names_called(names: &[String]) -> String {
    _a_list_of_names(&names.iter().map(|one| _called(one)).collect::<Vec<_>>())
}

/// Python: bot/memory/relations.py:1077 `_the_names_of_them_all(memory)`
///
/// The names of the several people an earlier line named, for a plural pronoun ask.
pub fn _the_names_of_them_all(memory: &Memory) -> Option<String> {
    let (word, names, phrase) = _the_several_named_before(memory);
    if !names.is_empty() {
        return Some(the_names_called(&names) + ".");
    }
    if !word.is_empty() {
        let told = MY_OUR_THE.sub_fn(&phrase, |_| "your ".to_string());
        let told = if told.is_empty() { word } else { told };
        return Some(pick(
            &[
                format!("You told me about {told}, but not their names - what are they called?"),
                format!("I've got {told} from you, but not their names. Tell me and I'll keep them."),
            ],
            memory,
        ));
    }
    None
}

/// Python: bot/memory/relations.py:1098 `_the_names_they_typed(word, memory)`
///
/// The names a line of theirs gave for a group, said back: "Tom and Sam, then - your kids".
pub fn _the_names_they_typed(word: &str, memory: &Memory) -> Option<String> {
    let said = _the_group_names_typed(word, memory);
    if said.is_empty() {
        return None;
    }
    Some(format!("{}, then - I'll keep those as your {word}' names.", the_names_called(&said)))
}

/// `int(entry.get("count") or 1)`.
fn count_of(entry: &Value) -> i64 {
    let count = eget(entry, "count");
    if !count.truthy() {
        return 1;
    }
    match count {
        Value::Int(n) => *n,
        Value::Bool(b) => *b as i64,
        Value::Float(f) => f.trunc() as i64,
        Value::Str(s) => match pt::strip(s).parse::<i64>() {
            Ok(n) => n,
            Err(_) => panic!("ValueError: invalid literal for int()"),
        },
        _ => panic!("TypeError: int() argument"),
    }
}

/// The names kept for an entry as a list: `[str(n) for n in (entry.get("names") or []) if
/// str(n).strip()]`.
fn names_of(entry: &Value) -> Vec<String> {
    let names = eget(entry, "names");
    if !names.truthy() {
        return Vec::new();
    }
    names.items().iter().map(py_str).filter(|one| !pt::strip(one).is_empty()).collect()
}

/// Python: bot/memory/relations.py:1107 `_the_names_of_the_group(memory, low)`
///
/// The names of more than one person they told us about: "Your son is called Jack, and ..."
/// A plural ask is about the people the chat already holds, so it is answered from them and
/// from nowhere else.
pub fn _the_names_of_the_group(memory: &mut Memory, low: &str) -> Option<String> {
    let ask = GROUP_NAME_ASK.search(low)?;
    let word =
        pt::lower(["a", "b", "c", "d"].into_iter().map(|name| ask.name_str(name)).find(|said| !said.is_empty()).unwrap_or(""));
    let mut known: Vec<Value> = others(memory).into_iter().filter(Value::is_dict).collect();
    if !word.is_empty() {
        let group = if word != "family" {
            GROUP_OF.get(&word).cloned().unwrap_or_default()
        } else {
            the_family(&["parents", "siblings", "grandparents", "grandchildren", "aunts", "uncles", "cousins"])
        };
        known.retain(|entry| in_the_group(&s_or(eget(entry, "who")), &group));
        if known.is_empty() && !_mentioned_the_group(memory, &group, &word) {
            return Some(pick(
                &[
                    format!("You haven't told me about your {word} yet."),
                    format!("I've got nothing about your {word} yet - tell me and I'll keep it."),
                ],
                memory,
            ));
        }
    } else if known.len() < 2 {
        // A plural pronoun is more than one person: "their" can never be answered with the one
        // person a line happened to file beside the people it named. Two entries are two
        // people, whatever each one is called; the filter is for the one entry a line about
        // several people leaves behind.
        known.retain(|entry| {
            _more_than_one(&s_or(eget(entry, "who"))) || names_of(entry).len() > 1 || count_of(entry) > 1
        });
    }
    // (who, what they are called, whether it is a pair said in one breath)
    let mut said: Vec<(String, String, bool)> = Vec::new();
    for entry in known.iter().take(4) {
        let who = sget(entry, "who");
        let who = pt::strip(&who).to_string();
        let name = eget(entry, "name");
        let names = names_of(entry);
        // "my parents are called janet and brian" files both the pair and the first name, and
        // the pair is the whole fact.
        if names.len() > 1 {
            // One fact about two people: the line never said which name is whose.
            said.push((who, _a_list_of_names(&names), true));
        } else if let Some(name) = name.as_str().filter(|name| !name.is_empty()) {
            said.push((who, _called(name), false));
        }
    }
    if said.is_empty() {
        if word.is_empty() {
            // The people a plural pronoun points at are in an earlier turn or nowhere: the chat
            // itself is the only place to look, and it answers this or nothing.
            if let Some(forgotten) = _the_names_of_them_all(memory).filter(|said| !said.is_empty()) {
                return Some(forgotten);
            }
            // The chat holds more than one person and a name on none of them: the group is the
            // people it holds, and the honest line is about all of them.
            let group = known
                .iter()
                .find(|entry| _more_than_one(&s_or(eget(entry, "who"))))
                .map(|entry| pt::strip(&sget(entry, "who")).to_string())
                .unwrap_or_default();
            if !group.is_empty() {
                return Some(pick(
                    &[
                        format!("You haven't told me a name for your {group} yet - what are they called?"),
                        format!("I've got your {group} but no names for them - what are they called?"),
                    ],
                    memory,
                ));
            }
        }
        if _the_group_was_said(&word, memory) {
            // The line that named them is in the chat and the readers did not file it: what the
            // store has is the line itself, so it says the names back and takes them.
            if let Some(typed) = _the_names_they_typed(&word, memory).filter(|said| !said.is_empty()) {
                return Some(typed);
            }
            // Named, and the names could not be read out of the line: asking for them keeps the
            // chat moving.
            return Some(pick(
                &[
                    format!("What are your {word} called? I'll keep their names."),
                    format!("Tell me what your {word} are called and I'll keep it."),
                ],
                memory,
            ));
        }
        if word.is_empty() {
            // Nothing to name: "a name for your yet" is not a sentence. The readers below can
            // still answer it, so this one declines.
            return None;
        }
        return Some(pick(
            &[
                format!("You haven't told me a name for your {word} yet."),
                format!("I don't have a name for your {word} yet."),
            ],
            memory,
        ));
    }
    // Two people filed under the same word are one sentence: "my kids are tom 8 and ada 5" files
    // Tom and Ada separately, and they are said as one group of two children.
    let mut grouped: Vec<(String, String, bool)> = Vec::new();
    for (who, value, paired) in &said {
        match grouped.iter_mut().find(|(held_who, _, held_paired)| !paired && !held_paired && held_who == who) {
            Some(held) => *held = (who.clone(), format!("{}\0{value}", held.1), *paired),
            None => grouped.push((who.clone(), value.clone(), *paired)),
        }
    }
    let mut parts: Vec<String> = Vec::new();
    for (who, value, paired) in &grouped {
        if *paired {
            parts.push(format!("you told me your {who} are called {value}"));
        } else {
            let names: Vec<String> = value.split('\0').map(str::to_string).collect();
            parts.push(format!(
                "your {who}{} called {}",
                if _more_than_one(who) { " are" } else { " is" },
                _a_list_of_names(&names)
            ));
        }
    }
    if parts.len() == 1 {
        return Some(up0(&parts[0]) + ".");
    }
    if parts.len() == 2 && !said.iter().any(|(_, _, paired)| *paired) {
        return Some(format!("{}, and {}.", up0(&parts[0]), parts[1]));
    }
    Some(parts.iter().map(|part| up0(part)).collect::<Vec<_>>().join(". ") + ".")
}

/// Python: bot/memory/relations.py:1224 `_did_you_mean(entry, property_, ask_gender='')`
///
/// The one person the store holds, offered back as a question rather than stated as a fact.
/// When the ask's own pronoun is the other one, the mismatch is said out loud.
pub fn _did_you_mean(entry: &Value, property_: &str, ask_gender: &str) -> String {
    let who = sget(entry, "who");
    let gender = _gender_of(&who);
    let pronoun = match gender.as_str() {
        "she" => "She",
        "he" => "He",
        _ => "They",
    };
    let plural = pronoun == "They";
    let (verb, fact) = if property_ == "name" {
        (if plural { "are called " } else { "is called " }, _called(&sget(entry, "name")))
    } else if eget(entry, "turns").truthy() {
        (if plural { "turn " } else { "turns " }, format!("{} {}", sget(entry, "age"), py_str(eget(entry, "turns"))))
    } else {
        (if plural { "are " } else { "is " }, sget(entry, "age"))
    };
    if !ask_gender.is_empty() && !gender.is_empty() && gender != ask_gender {
        let him = if gender == "she" { "her" } else { "him" };
        return format!("I've only got your {who} down, and {} {verb}{fact}. Did you mean {him}?", pt::lower(pronoun));
    }
    format!("Did you mean your {who}? {pronoun} {verb}{fact}.")
}

/// Whether the entry holds the fact asked for: `entry.get(property_) or (property_ == "age" and
/// entry.get("months"))`.
fn has_the_fact(entry: &Value, property_: &str) -> bool {
    eget(entry, property_).truthy() || (property_ == "age" && eget(entry, "months").truthy())
}

/// Python: bot/memory/relations.py:1252 `_relation_reply(memory, low)`
///
/// A fact about someone they mentioned, said back: "Your mum is called Janet." The person is
/// the one the ask names, the one it points at with a pronoun, or the last person the store
/// heard about. Only a fact the store holds is answered.
pub fn _relation_reply(memory: &mut Memory, low: &str) -> Option<String> {
    if let Some(tasted) = _their_taste(memory, low).filter(|said| !said.is_empty()) {
        return Some(tasted);
    }
    if let Some(worked) = _their_job(memory, low).filter(|said| !said.is_empty()) {
        return Some(worked);
    }
    if let Some(have_any) = _whether_they_have(memory, low).filter(|said| !said.is_empty()) {
        return Some(have_any);
    }
    if let Some(grouped) = _the_names_of_the_group(memory, low).filter(|said| !said.is_empty()) {
        return Some(grouped);
    }
    let wants_name = THEIR_NAME.is_match(low) || NAME_AGAIN.match_(low).is_some();
    let wants_age = THEIR_AGE.is_match(low);
    let mut by_name = false;
    if !(wants_name || wants_age) {
        return None;
    }
    let property_ = if wants_name { "name" } else { "age" };
    let mut who = _named_person(low, Some(&mut *memory));
    if property_ == "age"
        && !who.is_empty()
        && !who.ends_with('s')
        && Re::new(&format!(r"\b{}s\b", pyre::escape(&who)), 0).is_match(low)
    {
        // "how old are my daughters" asks about the group; `_ages_reply` says every age the
        // chat was given for the word they used.
        return None;
    }
    let mut known: Option<Value> = None;
    for entry in others(memory) {
        if !who.is_empty() && eget(&entry, "who").as_str() == Some(who.as_str()) {
            known = Some(entry);
            break;
        }
        if who.is_empty() && _same_name_however_they_wrote_it(&entry, low) {
            // "how old is janet" names the person the store knows by name, whether or not it
            // holds the thing asked for. The answer uses their name.
            by_name = eget(&entry, "name").truthy();
            who = s_or(eget(&entry, "who"));
            known = Some(entry);
            break;
        }
    }
    if known.is_none() && who.is_empty() && property_ == "age" && THEY_THEM.is_match(low) {
        // "how old are they" is every person the chat has an age for, not one of them:
        // `_ages_reply` answers the plural with all the ages.
        return None;
    }
    if known.is_none() && who.is_empty() && ANY_PRONOUN.is_match(low) {
        // "how old is she" after "my mum is 72" is the mum: the last person the store heard
        // about is the one a pronoun points at. A pronoun only points at a person it fits.
        let fits = if SHE_HER.is_match(low) { &*SHE } else { &*HE };
        for entry in others(memory).into_iter().rev() {
            if has_the_fact(&entry, property_) && pronoun_fits(&s_or(eget(&entry, "who")), fits) {
                who = s_or(eget(&entry, "who"));
                known = Some(entry);
                break;
            }
        }
    }
    if known.is_none() && who.is_empty() && ONE_PRONOUN.is_match(low) {
        // The pronoun fits nobody the store holds: when exactly one person holds this fact, the
        // honest reply offers that person as a question.
        let held: Vec<Value> =
            others(memory).into_iter().filter(|entry| entry.is_dict() && has_the_fact(entry, property_)).collect();
        if held.len() == 1 {
            let offered = _did_you_mean(&held[0], property_, &_ask_gender(low));
            if !offered.is_empty() {
                return Some(offered);
            }
        }
    }
    if known.is_none() && !who.is_empty() {
        // A pair given in one breath ("my mum and dad are dave and sue") is kept whole, because
        // the line does not say which name is whose. So the answer says what was said and asks
        // for the one thing that settles it.
        for entry in others(memory) {
            let kept_who = s_or(eget(&entry, "who"));
            let words: Vec<String> = kept_who.split(" and ").map(|w| pt::strip(w).to_string()).collect();
            let names_kept = eget(&entry, "names");
            let names: Vec<String> = if names_kept.truthy() { names_kept.items().iter().map(py_str).collect() } else { Vec::new() };
            let grouped = _relations_of(&kept_who);
            if names.len() > 1 && (words.contains(&who) || grouped.contains(&_canon_relation(&who))) {
                return Some(format!(
                    "You told me your {} are {} - which one is your {who}?",
                    py_str(eget(&entry, "who")),
                    _a_list_of_names(&names)
                ));
            }
        }
    }
    if known.is_none() && wants_name && !who.is_empty() {
        // An everyday memory question deserves the honest answer, not a shrug.
        return Some(format!("You haven't told me a name for your {who} yet."));
    }
    if let Some(known) = known.as_ref() {
        if !eget(known, property_).truthy() && property_ == "age" && eget(known, "name").truthy() {
            // They told us who she is but not how old she is: answered with what the store has.
            let pronoun = _pronoun_for(&who, known);
            // A "they" takes "are".
            return Some(format!(
                "Your {who} is called {}, but you haven't told me how old {pronoun}{}",
                _called(&py_str(eget(known, "name"))),
                if pronoun == "they" { " are." } else { " is." }
            ));
        }
        if property_ == "age" && eget(known, "months").truthy() && !eget(known, "age").truthy() {
            // A baby's age was said in months, so it is said back in months.
            let pronoun = sget(known, "pronoun");
            let (name, plural) = if !who.is_empty() {
                (format!("Your {who}"), who.ends_with('s'))
            } else {
                // An age the chat only ever said with a pronoun: the pronoun is the name.
                let name = match pronoun.as_str() {
                    "she" => "She",
                    "he" => "He",
                    _ => "They",
                };
                (name.to_string(), pronoun.is_empty() || pronoun == "they")
            };
            return Some(format!("{name}{}{} months old.", if plural { " are " } else { " is " }, py_str(eget(known, "months"))));
        }
    }
    let known = known.filter(|known| eget(known, property_).truthy())?;
    let kept = py_str(eget(&known, property_));
    let said = if property_ == "name" { _called(&kept) } else { kept };
    if who.is_empty() {
        return Some(format!("{}{said}.", if THEY.is_match(low) { "Their " } else { "That's " }));
    }
    if eget(&known, "name").truthy() && (by_name || same_name(&py_str(eget(&known, "name")), low)) {
        // The ask named the person by their own name, so the answer is about that person and not
        // about the word they were filed under: "Ada is 7.", never "Your kid is 7."
        let named_one = _called(&py_str(eget(&known, "name")));
        return Some(format!("{named_one}{}{said}.", if property_ == "name" { " is called " } else { " is " }));
    }
    // A word for more than one person takes "are".
    let mine = format!("Your {who} {}", if _more_than_one(&who) { "are " } else { "is " });
    if property_ == "name" {
        return Some(format!("{mine}called {said}."));
    }
    if eget(&known, "turns").truthy() {
        // An age still to come ("she's 4 next month") is said as a birthday.
        return Some(_one_age(&who, eget(&known, "age"), &py_str(eget(&known, "turns")), eget(&known, "to_come"), &NONE));
    }
    Some(format!("{mine}{said}."))
}

/// Python: bot/memory/relations.py:1398 `_pronoun_for(who, known=None)`
///
/// The pronoun a person takes: "she", "he" or "they". A title is evidence and wins over the
/// word: Mr is a he.
pub fn _pronoun_for(who: &str, known: &Value) -> String {
    let name = if known.truthy() { sget(known, "name") } else { String::new() };
    // `name.split(" ")[0].strip(".").lower() if " " in name or name else ""`
    let title = pt::lower(pt::strip_chars(name.split(' ').next().unwrap_or(""), "."));
    if let Some(gender) = TITLE_GENDER.get(&title) {
        return gender.clone();
    }
    if SHE.contains(who) && !HE.contains(who) {
        return "she".to_string();
    }
    if HE.contains(who) && !SHE.contains(who) {
        return "he".to_string();
    }
    "they".to_string()
}

/// Python: bot/memory/relations.py:1416 `_the_group_word(who)`
///
/// The group word the person used, in the spelling the answer can say out loud.
pub fn _the_group_word(who: &str) -> String {
    let who = pt::lower(who);
    let forms = [
        who.clone(),
        who.strip_suffix("es").unwrap_or("").to_string(),
        who.strip_suffix('s').unwrap_or("").to_string(),
        CHILD_OF.get(&who).cloned().unwrap_or_default(),
        format!("{who}s"),
    ];
    forms.into_iter().find(|word| !word.is_empty() && PERSON_GROUP.contains_key(word)).unwrap_or_default()
}

/// Python: bot/memory/relations.py:1430 `_the_group_was_said(who, memory)`
///
/// Whether any of the person's own lines named this group, in any spelling of it.
pub fn _the_group_was_said(who: &str, memory: &Memory) -> bool {
    let word = _the_group_word(who);
    if word.is_empty() {
        return false;
    }
    let forms: HashSet<String> =
        [word.clone(), format!("{word}s"), format!("{word}es"), CHILD_OF.get(&word).cloned().unwrap_or_default()]
            .into_iter()
            .filter(|one| !one.is_empty() && pt::len(one) > 2)
            .collect();
    let patterns: Vec<Re> =
        forms.iter().map(|one| Re::new(&format!(r"(?<![a-z]){}(?![a-z])", pyre::escape(one)), 0)).collect();
    memory.turns.iter().any(|turn| {
        let said = pt::lower(&turn.user);
        patterns.iter().any(|pattern| pattern.is_match(&said))
    })
}

/// One age the chat was given: (who it was said of, the age, when they turn it, the age still
/// to come, the name they have).
#[derive(Clone, Debug)]
pub struct Told {
    pub who: String,
    pub age: Value,
    pub turns: String,
    pub to_come: Value,
    pub held: String,
}

fn one_age_of(who: &str, told: &Told) -> String {
    _one_age(who, &told.age, &told.turns, &told.to_come, &Value::from(told.held.as_str()))
}

/// Python: bot/memory/relations.py:1452 `_ages_reply(memory, low, text='')`
///
/// How old a person they told us about is, said back: "Your son is 7." Their ages are filed
/// with the person, so the ask is about what the chat has been told, not about the world.
pub fn _ages_reply(memory: &mut Memory, low: &str, text: &str) -> Option<String> {
    // "and my dad" after "what do you know about my mum" is that ask about another person, not
    // the age question.
    if _continues_an_about_ask(memory, low) {
        return None;
    }
    if !(AGE_ASK.is_match(low) || AGE_WORD.is_match(low)) {
        // "and my daughter?" is only an ask about an age when it follows one.
        let said = if text.is_empty() { low } else { text };
        let said = pt::lower(pt::rstrip_chars(pt::strip(said), "?.!"));
        if !(_just_gave_an_age(memory) && AGE_AGAIN.match_(&said).is_some()) {
            return None;
        }
    }
    // An age named as the reason for something is not a request for the number.
    if AGE_AS_A_CAUSE.is_match(low) && !AGE_ASK.is_match(low) {
        return None;
    }
    // An age a rule turns on is not an age of theirs: the person has to be what the ask asks.
    if AGE_ASK.is_match(low) && !_age_is_about_a_person(low, memory) {
        return None;
    }
    // "is that normal for his age" is not the ask "how old is he": the age is the measure the
    // question turns on. Only when the store does hold an age.
    if !AGE_ASK.is_match(low)
        && WHAT_IS_USUAL.is_match(low)
        && others(memory)
            .iter()
            .any(|entry| is_int(eget(entry, "age")) || is_int(eget(entry, "months")) || eget(entry, "ages").truthy())
    {
        return None;
    }
    let mut told: Vec<Told> = Vec::new();
    for entry in others(memory) {
        let who_said = sget(&entry, "who");
        let turns = sget(&entry, "turns");
        // A child named as well as aged is said back by their name; the name rides along with
        // the age, beside the birthday still to come.
        let held = sget(&entry, "name");
        let to_come = eget(&entry, "to_come");
        if is_int(eget(&entry, "age")) {
            told.push(Told {
                who: who_said.clone(),
                age: eget(&entry, "age").clone(),
                turns: turns.clone(),
                to_come: if is_int(to_come) { to_come.clone() } else { Value::None },
                held: held.clone(),
            });
        }
        // The children said as a group keep a list of ages, each said back in the order told.
        let ages = eget(&entry, "ages");
        if ages.truthy() {
            for age in ages.items().iter().filter(|age| is_int(age)) {
                told.push(Told { who: who_said.clone(), age: age.clone(), turns: turns.clone(), to_come: Value::None, held: String::new() });
            }
        }
        // A baby's age is kept in months; the figure carries its unit into the answer.
        let months = eget(&entry, "months");
        if is_int(months) {
            told.push(Told {
                who: who_said.clone(),
                age: Value::from(format!("{} months", py_str(months))),
                turns: turns.clone(),
                to_come: Value::None,
                held: held.clone(),
            });
        }
    }
    // The plural names the same person; a possessive in front of another noun is not the person.
    if _named_person(low, Some(&mut *memory)).is_empty() {
        // A child named by their place in the family, which the store files under other words.
        let placed = _the_place_word_filed(memory, low);
        if !placed.is_empty() {
            return Some(placed);
        }
    }
    let mut who = _named_person(low, Some(&mut *memory));
    // The ask's own word decides the number.
    let asked_plural = !who.is_empty()
        && (who.ends_with('s') || Re::new(&format!(r"\b{}s\b", pyre::escape(&who)), 0).is_match(low));
    if let Some(group) = GROUP_ASK.search(low) {
        // "how old are my parents" is about the family the line names, not about one member of
        // it: the answer says every age the store holds for that family, in the words the chat
        // used.
        let word = pt::lower(group.str(1));
        let family = match FAMILY_GROUP.get(&word) {
            Some(family) => family,
            None => panic!("KeyError: '{word}'"),
        };
        let members: Vec<Told> = told
            .iter()
            .filter(|entry| {
                let first = first_word(&entry.who);
                family.iter().any(|one| one == first)
                    || first == word
                    || CHILD_OF.get(first).is_some_and(|child| family.contains(child))
            })
            .cloned()
            .collect();
        if !members.is_empty() {
            return Some(_ages_sentence(&members));
        }
        if matches!(word.as_str(), "kids" | "children") && told.is_empty() {
            return Some("You haven't told me how old they are.".to_string());
        }
        return Some(format!("You haven't told me how old your {word} are."));
    }
    if who.is_empty() && THE_KIDS.is_match(low) {
        let children: Vec<Told> = told
            .iter()
            .filter(|entry| {
                let child = CHILD_OF.get(&entry.who).cloned().unwrap_or_else(|| pt::rstrip_chars(&entry.who, "s").to_string());
                CHILD.contains(&child)
            })
            .cloned()
            .collect();
        if CHILD_WORD.is_match(low) {
            // "my kids", "the children": the people who are children, not everyone the chat
            // knows. With no child's age on file the honest line is that.
            if children.is_empty() {
                return Some("You haven't told me how old they are.".to_string());
            }
            return Some(_ages_sentence(&children));
        }
        // "they", "them", "their", "both", "everyone": the ages in front of the chat, children
        // first, and whatever person it does know when there are no children in it.
        let told = if children.is_empty() { told } else { children };
        if told.is_empty() && !CHILDREN_WORD.is_match(low) {
            // "how old are they" straight after "my dog is 5 and my cat is 12" is about the
            // animals.
            let ages = _pet_ages(memory);
            if !ages.is_empty() {
                return Some(_pet_ages_sentence(&ages));
            }
        }
        if told.is_empty() {
            if let Some(theirs) = memory.get("their_age").filter(|theirs| theirs.is_dict() && eget(theirs, "age").truthy()) {
                // An age the line gave for "they" and never for anybody named.
                return Some(_one_age("", eget(theirs, "age"), &s_or(eget(theirs, "turns")), &NONE, &NONE));
            }
            return Some("You haven't told me how old they are.".to_string());
        }
        return Some(_ages_sentence(&told));
    }
    if who.is_empty() && A_PERSON_PRONOUN.is_match(low) {
        who = _person_from_pronoun(low, memory, &[]);
        if who.is_empty() {
            // An age the chat keeps against the pronoun itself ("hes 8 next month", said after a
            // line about somebody else): the pronoun is who the store was given.
            let pronoun = if SHE_HER.is_match(low) { "she" } else { "he" };
            for entry in others(memory) {
                if sget(&entry, "pronoun") != pronoun {
                    continue;
                }
                let age = eget(&entry, "age");
                let months = eget(&entry, "months");
                if !is_int(age) && !is_int(months) {
                    continue;
                }
                let he = pronoun == "he";
                if is_int(months) {
                    // "shes 4 months" is said back in months.
                    return Some(format!("{}{} months.", if he { "He is " } else { "She is " }, py_str(months)));
                }
                let when = sget(&entry, "turns");
                if !when.is_empty() {
                    return Some(format!("{}{} {when}.", if he { "He turns " } else { "She turns " }, py_str(age)));
                }
                return Some(format!("{}{}.", if he { "He is " } else { "She is " }, py_str(age)));
            }
        }
    }
    if who.is_empty() {
        // An animal they told us about has an age the chat holds the same way. Nothing in the
        // line says whose age otherwise, and the modules that know about the world answer it.
        return _pet_age_reply(memory, low).filter(|said| !said.is_empty());
    }
    // A child named in the line is the person the ask is about, and their age is the one filed
    // with their name.
    for one in &told {
        if !one.held.is_empty() && Re::new(&format!(r"\b{}\b", pyre::escape(&pt::lower(&one.held))), 0).is_match(low) {
            return Some(one_age_of(&who, one));
        }
    }
    // A plural ask is about the group, not the one person the store filed under the same word.
    if asked_plural {
        let word = if who.ends_with('s') { who.clone() } else { format!("{who}s") };
        let family: Vec<Told> = told.iter().filter(|one| first_word(&one.who) == word).cloned().collect();
        if family.len() > 1 {
            return Some(_ages_sentence(&family));
        }
    }
    for one in &told {
        if one.who == who || (who.ends_with('s') && one.who == who[..who.len() - 1]) {
            return Some(one_age_of(&who, one));
        }
    }
    // The store keeps a plural word for the people a plural line named as one group, so a
    // singular ask with that word in it is about the same people.
    let plural = format!("{who}s");
    let irregular = _IRREGULAR_PLURAL.get(&who).cloned().unwrap_or_else(|| plural.clone());
    let family: Vec<Told> = told
        .iter()
        .filter(|one| {
            let first = first_word(&one.who);
            first == plural || first == irregular
        })
        .cloned()
        .collect();
    if !family.is_empty() {
        return Some(_ages_sentence(&family));
    }
    // A person was named, so the answer is about them and nobody else.
    Some(format!("You haven't told me how old your {who}{}", if who.ends_with('s') { " are." } else { " is." }))
}

/// Python: bot/memory/relations.py:1661 `_the_place_end(low)`
///
/// ("young" | "old" | "", the place word): the end of the family the ask's place word names.
/// The place word has to be the whole of the noun phrase.
pub fn _the_place_end(low: &str) -> (String, String) {
    let nothing = (String::new(), String::new());
    let Some(asked) = _A_PLACE_ASK.search(low) else {
        return nothing;
    };
    let after = &low[asked.end()..];
    let after = match PLACE_ENDS.search(after) {
        Some(cut) => &after[..cut.start()],
        None => after,
    };
    let after = pt::strip(after);
    let next_word = pt::split_ref(after).first().map(|word| pt::lower(word)).unwrap_or_default();
    if !next_word.is_empty() && !_THE_PLACE_STANDS_ALONE.contains(&next_word) {
        return nothing;
    }
    let word = pt::squash(&pt::lower(asked.name_str("who")));
    let end = if _THE_YOUNGER_END.contains(&word) { "young" } else { "old" };
    (end.to_string(), word)
}

/// Python: bot/memory/relations.py:1679 `_the_child_ages_on_file(memory)`
///
/// The ages the chat holds for children of theirs, in the order filed, as (word, age, turns).
pub fn _the_child_ages_on_file(memory: &mut Memory) -> Vec<(String, Value, String)> {
    let mut found = Vec::new();
    for entry in others(memory) {
        if !entry.is_dict() {
            continue;
        }
        let who = pt::lower(&sget(&entry, "who"));
        let child = CHILD_OF.get(&who).cloned().unwrap_or_else(|| pt::rstrip_chars(&who, "s").to_string());
        if !CHILD.contains(&child) && !CHILD.contains(&who) {
            continue;
        }
        let turns = sget(&entry, "turns");
        if is_int(eget(&entry, "age")) {
            found.push((who.clone(), eget(&entry, "age").clone(), turns.clone()));
        }
        let ages = eget(&entry, "ages");
        if ages.truthy() {
            for age in ages.items().iter().filter(|age| is_int(age)) {
                found.push((who.clone(), age.clone(), turns.clone()));
            }
        }
    }
    found
}

/// Python: bot/memory/relations.py:1697 `_the_place_word_filed(memory, low)`
///
/// How old the child at the end of the family the ask names is, or "" when that end is empty.
pub fn _the_place_word_filed(memory: &mut Memory, low: &str) -> String {
    let (end, word) = _the_place_end(low);
    if end.is_empty() || word.is_empty() {
        return String::new();
    }
    let mut ages = _the_child_ages_on_file(memory);
    if ages.is_empty() {
        return String::new();
    }
    ages.sort_by_key(|one| int_of(&one.1)); // stable, as Python's sort is
    let (_who, age, turns) = if end == "young" { ages[0].clone() } else { ages[ages.len() - 1].clone() };
    if end == "old" && ages[..ages.len() - 1].iter().any(|one| one.1 == age) {
        return String::new();
    }
    _one_age(&word, &age, &turns, &NONE, &NONE)
}

/// Python: bot/memory/relations.py:1732 `_pet_ages(memory)`
///
/// The animals the store holds an age for.
pub fn _pet_ages(memory: &mut Memory) -> Vec<Value> {
    user_list(memory, "pets").into_iter().filter(|pet| pet.is_dict() && is_int(eget(pet, "age"))).collect()
}

/// Python: bot/memory/relations.py:1738 `the_pronoun_said_with_a_name(memory, age)`
///
/// The age a pronoun answer is about, said with the animal's name, or None. The animal is only
/// named when the age about to be said is the age the store filed for it, and for exactly one
/// animal.
pub fn the_pronoun_said_with_a_name(memory: &mut Memory, age: &Value) -> Option<String> {
    if !is_int(age) {
        return None;
    }
    let aged: Vec<Value> = _pet_ages(memory).into_iter().filter(|pet| eget(pet, "age") == age).collect();
    if aged.len() != 1 {
        return None;
    }
    Some(_pet_ages_sentence(&aged))
}

/// Python: bot/memory/relations.py:1761 `_pet_age_reply(memory, low)`
///
/// How old an animal they told us about is: "Max is 4.", "Your dog is 5." When no age was ever
/// given the store says so about that animal rather than guessing or searching.
pub fn _pet_age_reply(memory: &mut Memory, low: &str) -> Option<String> {
    let pets: Vec<Value> = user_list(memory, "pets").into_iter().filter(Value::is_dict).collect();
    let mut asked: Vec<Value> = Vec::new();
    let mut family = PET_ASK.search(low).map(|animal| _animal_family(animal.str(1))).unwrap_or_default();
    if family.is_empty() {
        if let Some(breed) = BREED_ASK.search(low) {
            family = _animal_family(&crate::understanding::breeds::the_kind_of(last_word(breed.as_str())));
        }
    }
    if !family.is_empty() {
        asked = pets.iter().filter(|pet| _animal_family(&s_or(eget(pet, "animal"))) == family).cloned().collect();
        if asked.is_empty() {
            // The line says the animal is theirs, so the world is not the place to look.
            let an = if family.chars().next().is_some_and(|c| "aeiou".contains(c)) { "an" } else { "a" };
            return Some(format!("You haven't told me about {an} {family}."));
        }
    } else {
        for pet in pets.iter().rev() {
            let name = sget(pet, "name");
            let name = pt::strip(&name);
            if pt::len(name) > 2 && Re::new(&format!(r"\b{}\b", pyre::escape(&pt::lower(name))), 0).is_match(low) {
                asked.push(pet.clone());
            }
        }
    }
    if asked.is_empty() && AN_ANIMAL_PRONOUN.is_match(low) {
        // "how old is she" is about the one animal the chat holds an age for, and the pronoun is
        // the only thing that says which animal is meant.
        let aged = _pet_ages(memory);
        if !aged.is_empty() {
            asked = aged;
        }
    }
    if asked.is_empty() {
        return None;
    }
    let aged: Vec<Value> = asked.iter().filter(|pet| is_int(eget(pet, "age"))).cloned().collect();
    if aged.is_empty() {
        let family = _animal_family(&s_or(eget(&asked[0], "animal")));
        if asked.len() > 1 || count_of(&asked[0]) > 1 {
            return Some(format!("You haven't told me how old your {family}s are."));
        }
        return Some(format!("You haven't told me how old {} is.", _pet_named(&asked[0])));
    }
    let said = _pet_ages_sentence(&aged);
    if _same_as_last(&said, memory) {
        // "how old is my cat" right after "how old is she" is the same age: say it short.
        let ages: Vec<String> = aged.iter().take(4).map(_a_pet_age).collect();
        return Some(format!("Still {}.", ages.join(" and ")));
    }
    Some(said)
}

/// Python: bot/memory/relations.py:1815 `_pet_named(pet)`
///
/// How to name one animal: by the name they gave it ("Max"), or by what they call it.
pub fn _pet_named(pet: &Value) -> String {
    let name = sget(pet, "name");
    let name = pt::strip(&name);
    if !name.is_empty() {
        return tidy_name(name);
    }
    let breed = eget(pet, "breed");
    let animal = s_or(if breed.truthy() { breed } else { eget(pet, "animal") });
    let animal = pt::strip(&animal);
    if !animal.is_empty() && !PLURAL_ANIMAL.is_match(animal) {
        return format!("your {animal}");
    }
    format!("your {}", _animal_family(animal))
}

/// Python: bot/memory/relations.py:1826 `_a_pet_age(pet)`
///
/// The age as it was given: "5", "4 months old".
pub fn _a_pet_age(pet: &Value) -> String {
    let unit = sget(pet, "unit");
    let Some(age) = pet.get("age") else {
        panic!("KeyError: 'age'");
    };
    if unit.is_empty() { py_str(age) } else { format!("{} {unit} old", py_str(age)) }
}

/// Python: bot/memory/relations.py:1832 `_pet_ages_sentence(pets)`
///
/// Every age the chat was given for these animals, said back as one sentence.
pub fn _pet_ages_sentence(pets: &[Value]) -> String {
    let parts: Vec<String> = pets
        .iter()
        .take(4)
        .map(|pet| {
            let who = _pet_named(pet);
            let verb = if who.starts_with("your ") && who.ends_with('s') { " are " } else { " is " };
            format!("{who}{verb}{}", _a_pet_age(pet))
        })
        .collect();
    if parts.is_empty() {
        panic!("IndexError: list index out of range");
    }
    up0(&parts.join(" and ")) + "."
}

/// Python: bot/memory/relations.py:1845 `_one_age(who, age, turns='', to_come=None, held='')`
///
/// One person's age, said back: a birthday still to come is said as one, and with both numbers
/// on file the answer is both. A person with a name of their own is said by that name.
/// (`turns` is the words for when they turn that age: a str, which the skeleton had typed as
/// the chat's turns.)
pub fn _one_age(who: &str, age: &Value, turns: &str, to_come: &Value, held: &Value) -> String {
    let has_a_name = held.truthy();
    let name = if has_a_name {
        up1(&py_str(held))
    } else if PRONOUN_PEOPLE.iter().any(|pronoun| pronoun == who) {
        // A person the chat only pointed at: the pronoun is the name.
        who.to_string()
    } else if !who.is_empty() {
        // One person keeps the word they came in.
        format!("your {who}")
    } else {
        "they".to_string()
    };
    // A child said by their own name is one person, whatever group word they arrived in.
    let plural = !has_a_name && (who.ends_with('s') || who.is_empty() || who == "they" || who == "them");
    let age = py_str(age);
    let said = if !turns.is_empty() && is_int(to_come) && py_str(to_come) != age {
        format!("{name}{}{age}, and turns {} {turns}", if plural { " are " } else { " is " }, py_str(to_come))
    } else if !turns.is_empty() {
        format!("{name}{}{age} {turns}", if plural { " turn " } else { " turns " })
    } else {
        format!("{name}{}{age}", if plural { " are " } else { " is " })
    };
    up0(&said) + "."
}

/// Python: bot/memory/relations.py:1879 `_just_gave_an_age(memory)`
///
/// True when the bot's last reply was an age it said back ("Your son is 7.").
pub fn _just_gave_an_age(memory: &Memory) -> bool {
    AGE_SAID.is_match(pt::strip(&crate::memory::last_bot(memory)))
}

/// Python: bot/memory/relations.py:1892 `_the_many(who)`
///
/// The word for the group of them, in the shape the sentence needs ("daughter" -> "daughters").
pub fn _the_many(who: &str) -> String {
    let word = pt::lower(pt::strip(who));
    if word.is_empty() {
        return String::new();
    }
    if let Some(many) = MANY_OF_THEM.get(&word) {
        return many.clone();
    }
    if ALREADY_MANY.contains(&word) || word.ends_with('s') {
        return word;
    }
    if let Some(root) = word.strip_suffix('y') {
        // `word[-2:-1] not in "aeiou"`: the letter before the "y" (nothing, for a one-letter word,
        // which is "in" any string).
        let before = root.chars().last();
        if before.is_some_and(|c| !"aeiou".contains(c)) {
            return format!("{root}ies");
        }
    }
    word + "s"
}

/// Python: bot/memory/relations.py:1907 `_ages_sentence(told)`
///
/// Every age the chat has been given, as one sentence, oldest fact first. Ages given for the
/// same word are said once, in the order they were told.
pub fn _ages_sentence(told: &[Told]) -> String {
    struct Group {
        who: String,
        ages: Vec<Value>,
        turns: String,
        to_come: Value,
        held: String,
    }
    let mut groups: Vec<Group> = Vec::new();
    for one in told.iter().take(4) {
        match groups.iter_mut().find(|group| group.who == one.who && group.held == one.held && group.turns.is_empty() && one.turns.is_empty()) {
            Some(group) => group.ages.push(one.age.clone()),
            None => groups.push(Group {
                who: one.who.clone(),
                ages: vec![one.age.clone()],
                turns: one.turns.clone(),
                to_come: one.to_come.clone(),
                held: one.held.clone(),
            }),
        }
    }
    let mut parts: Vec<String> = Vec::new();
    for group in &groups {
        if group.ages.len() == 1 {
            let said = _one_age(&group.who, &group.ages[0], &group.turns, &group.to_come, &Value::from(group.held.as_str()));
            parts.push(pt::rstrip_chars(&said, ".").to_string());
            continue;
        }
        let name = if group.who.is_empty() { "they".to_string() } else { format!("your {}", _the_many(&group.who)) };
        let n = group.ages.len();
        let said = group.ages[..n - 1].iter().map(py_str).collect::<Vec<_>>().join(", ");
        parts.push(format!("{name} are {said} and {}", py_str(&group.ages[n - 1])));
    }
    // A part that starts with a name keeps its capital: "Ada is 7 and Sam is 4", never "and sam".
    let starts_with_a_name = |part: &str| {
        groups.iter().any(|group| {
            // `part.startswith(str(held)[:1].upper())`
            !group.held.is_empty() && part.starts_with(&group.held.chars().take(1).flat_map(char::to_uppercase).collect::<String>())
        })
    };
    let parts: Vec<String> = parts
        .iter()
        .enumerate()
        .map(|(i, part)| if i == 0 || starts_with_a_name(part) { part.clone() } else { low1(part) })
        .collect();
    if parts.len() == 1 {
        return up0(&parts[0]) + ".";
    }
    if parts.is_empty() {
        panic!("IndexError: list index out of range");
    }
    let said = format!("{} and {}", parts[..parts.len() - 1].join(", "), parts[parts.len() - 1]);
    up0(&said) + "."
}

/// Python: bot/memory/relations.py:1939 `_ask_gender(low)`
///
/// The gender the line's own pronoun names, or "" when it names none.
pub fn _ask_gender(low: &str) -> String {
    if SHE_ASKED.is_match(low) {
        return "she".to_string();
    }
    if HE_ASKED.is_match(low) {
        return "he".to_string();
    }
    String::new()
}

/// Python: bot/memory/relations.py:1948 `_called(name)`
///
/// A name as it is written back: "biscuit" reads "Biscuit", "mr patel" reads "Mr Patel".
pub fn _called(name: &str) -> String {
    let spaced = name.replace('.', " ");
    let words = pt::split_ref(&spaced);
    // A title and a name are two words, and both are written the way a name is written.
    words.iter().map(|word| up1(word)).collect::<Vec<_>>().join(" ")
}

/// Python: bot/memory/relations.py:1961 `_every_line(memory)`
///
/// Every line the person has said, oldest first, as the turns they were said in.
pub fn _every_line(memory: &Memory) -> &[Turn] {
    &memory.turns
}
