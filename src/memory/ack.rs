//! Port of `bot/memory/ack.py`: say back the thing they just told you, in their own words.
//!
//! A person who says "my sister is called Meg" or "we're flying to Rome on Monday" is handing
//! you something. A friend answers with the thing itself - "Meg, got it" or "Rome on Monday,
//! nice" - not one stock line for every line, and not a question at the end every time either.
//! Every rule here is a guard against taking a turn that is not that (see the Python module
//! docstring): only a statement, only a fact or a detail of the thing in hand, never a greeting,
//! a feeling, a loss, good news, a body that hurts; and the echo is short.
//!
//! CONTRACT notes: functions the generated contract types `()` keep that signature and gain a
//! `*_text` variant that carries the value (`a_limit_reply_text`, `pet_given_away_text`, ...).
#![allow(non_snake_case)]

use crate::data::{self, Value};
use crate::memory::narrow::{memory_value, py_format, py_str, py_str_or_empty, py_try, re_cached};
use crate::memory::{facts, named, topics};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use crate::understanding::pyuni;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

macro_rules! data_re {
    ($($name:ident),* $(,)?) => {
        $( static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.memory.ack.", stringify!($name)))); )*
    };
}
macro_rules! data_set {
    ($($name:ident),* $(,)?) => {
        $( static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.memory.ack.", stringify!($name)))); )*
    };
}
macro_rules! data_strs {
    ($($name:ident),* $(,)?) => {
        $( static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.memory.ack.", stringify!($name)))); )*
    };
}
macro_rules! data_map {
    ($($name:ident),* $(,)?) => {
        $( static $name: LazyLock<HashMap<String, String>> =
            LazyLock::new(|| data::str_map(concat!("bot.memory.ack.", stringify!($name)))); )*
    };
}
macro_rules! data_pools {
    ($($name:ident),* $(,)?) => {
        $( static $name: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
            data::get(concat!("bot.memory.ack.", stringify!($name)))
                .pairs()
                .iter()
                .map(|(k, v)| (py_str(k), v.strs()))
                .collect()
        }); )*
    };
}
macro_rules! inline_re {
    ($name:ident, $pattern:expr, $flags:expr) => {
        static $name: LazyLock<Re> = LazyLock::new(|| Re::new($pattern, $flags));
    };
}

// ---------------------------------------------------------------------------------------------
// Helpers shared by the readers below.

/// `len(s.split())`.
fn n_words(s: &str) -> usize {
    text::split_ref(s).len()
}

/// `" ".join(s.split())`.
fn squash(s: &str) -> String {
    text::squash(s)
}

/// `" ".join(s.lower().split())`.
fn flat(s: &str) -> String {
    text::squash(&text::lower(s))
}

/// `s[:1].upper() + s[1:]`.
fn upper_first(s: &str) -> String {
    text::upper_first(s)
}

/// `pick(options, memory, seed_text)` from the composer.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, &memory_value(memory), seed_text)
}

/// The turns before this one.
fn turns(memory: &Memory) -> &[Turn] {
    &memory.turns
}

/// `turns[-n:]`.
fn last_turns(memory: &Memory, n: usize) -> &[Turn] {
    let all = turns(memory);
    &all[all.len().saturating_sub(n)..]
}

/// `bot.memory.last_user(memory)`: the person's last line, or "".
fn last_user(memory: &Memory) -> String {
    turns(memory).last().map(|t| t.user.clone()).unwrap_or_default()
}

/// `bot.memory.there_is_a_chat(memory)`: whether the person said anything before this line.
fn there_is_a_chat(memory: &Memory) -> bool {
    turns(memory).iter().any(|t| !text::strip(&t.user).is_empty())
}

/// `bot.memory.others(memory)`: the store's list of other people, made an empty list when it is
/// missing, as Python's `others` does. (The contract's `others` is typed `Vec<String>`; the list
/// holds dicts, so it is read here.)
fn the_others(memory: &Memory) -> Vec<Value> {
    match memory.get("others") {
        Some(Value::List(held)) => held.to_vec(),
        _ => Vec::new(),
    }
}

/// `the_others(memory)` for a caller that may write: Python sets `memory["others"] = []` when the
/// key is missing.
fn the_others_mut(memory: &mut Memory) -> Vec<Value> {
    if !matches!(memory.get("others"), Some(Value::List(_))) {
        memory.set("others", Value::list());
    }
    the_others(memory)
}

/// `str(entry.get(key) or "")`.
fn field(entry: &Value, key: &str) -> String {
    py_str_or_empty(entry.get(key))
}

/// `re.escape(word)` built into a pattern, compiled once.
fn word_re(pattern: &str) -> &'static Re {
    re_cached(pattern, 0)
}

// Data of other modules this one reads.
static RELATIONS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.facts.RELATIONS"));
static RELATION_SET: LazyLock<HashSet<String>> = LazyLock::new(|| RELATIONS.iter().cloned().collect());
static HE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.HE"));
static SHE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.SHE"));

data_strs!(
    FACTS, EXPLAIN_TAIL, SOMEONE_TAIL, VISIT_TAIL, SOMEONE_MILESTONE_TAIL, A_PERSON_ALONE_LINES, A_KNOWN_AGE_LINES,
    A_TWO_ALONE_LINES, A_GROUP_ALONE_LINES, FRAMES, THE_LIMIT_GOES_FIRST, SOON_TAIL, FAR_TAIL, SORRY_TAIL,
    WORK_FRAMES, HABIT_TAIL, GOING_TAIL, TRIP_TAIL, OCCASION_TAIL, PET_TAIL_ONE, PET_TAIL_MANY,
    OCCASION_FLAT_TAIL, DIARY_TAIL, DIARY_TIME_TAIL, MEETING_TAIL, MEETING_TIME_TAIL, VISIT_TAIL_ONE,
    VISIT_TAIL_MANY,
);
data_set!(NOT_A_THING, A_POSITION_MAYBE_A_THING, WORK_PLACE, PLURAL_KIN, OCCASIONS, SHOWER, PLACES_WITH_THE, THING_BEFORE, PAST_PLAN, NOT_A_TRIP);
data_map!(A_GROUP_ALONE, A_POSITION);
data_pools!(A_POSITION_LINES);
data_re!(
    HAVE_THING, AN_AIM, A_FEELING_ABOUT_IT, SOMEONE_GOOD, SOMEONE_COMING, NOT_A_VISIT, SOMEONE_MILESTONE,
    A_SECOND_PERSON, A_BIRTHDAY_TOLD, A_PLAN_VERB, A_DATE_TOLD, A_POINTED_AT, A_PERSON_ALONE, WHICH_ONE,
    STILL_TO_DO, NOT_ONE_THING, _A_PLAN_WAS_GIVEN, DAY_SOON, DAY_BEEN_AND_GONE, A_DOING, A_FIGURE,
    A_DOING_TURNED_DOWN, DAY_JUST_A_STATE, DAY_FAR, A_DAY_ASKED, NOT_A_DAY_TO_LOOK_FORWARD_TO, _SENTENCE,
    _SPACED_DASH, DIARY_THING, DIARY_TIME, COMING_TO_STAY, A_COMING_WORD, TRIP_VERB, DESTINATION,
    ARRIVE_VERB, DESTINATION_ARRIVAL, DATE_AHEAD, DREADED_OCCASION, HOW_LONG_ALREADY,
);

/// `_SAID_BACK`: each frame cut into its fixed front and back.
static _SAID_BACK: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.memory.ack._SAID_BACK")
        .items()
        .iter()
        .map(|pair| (pair.items()[0].str().to_string(), pair.items()[1].str().to_string()))
        .collect()
});

inline_re!(RE_DIGIT, r"\d", 0);
inline_re!(RE_ARE_PLURAL, r"\b(?:are|were|were both|are both)\b", 0);
inline_re!(RE_AND_ME, r"\band (?:i|me|we|us)\b", 0);
inline_re!(RE_AND_OR_AMP, r"\band\b|&", 0);

// ---------------------------------------------------------------------------------------------
// A person named on their own.

/// Python: bot/memory/ack.py:241 `_a_relation(word)`
///
/// The relation a bare possessive names ("dads" -> "dad"), or "" when it names none.
pub fn _a_relation(word: &str) -> String {
    let said = text::strip_chars(&flat(word), " .,;:!?'").to_string();
    let without_s = if said.ends_with('s') { said[..said.len() - 1].to_string() } else { String::new() };
    for candidate in [said.clone(), without_s, said.replace(' ', "")] {
        if RELATION_SET.contains(&candidate) {
            return candidate;
        }
    }
    String::new()
}

/// Python: bot/memory/ack.py:250 `_the_pronoun(relation)`
///
/// "he" or "she" for a relation that says it, and "" for one that does not.
pub fn _the_pronoun(relation: &str) -> String {
    if HE.contains(relation) {
        return "he".into();
    }
    if SHE.contains(relation) {
        return "she".into();
    }
    String::new()
}

/// Python: bot/memory/ack.py:259 `_the_ages_held(memory)`
///
/// {relation: age} for the people the chat has already been given an age for.
pub fn _the_ages_held(memory: &mut Memory) -> HashMap<String, Value> {
    let mut held = HashMap::new();
    for entry in the_others_mut(memory) {
        if entry.is_dict() && entry.get("age").is_some_and(|a| !a.is_none()) {
            held.insert(text::lower(&field(&entry, "who")), entry.get("age").cloned().unwrap());
        }
    }
    held
}

/// Python: bot/memory/ack.py:269 `_people_in_chat(memory)`
///
/// "the youngest" and "the other one" are people only in a chat that has people in it.
pub fn _people_in_chat(memory: &mut Memory) -> bool {
    if let Some(user) = memory.get("user") {
        if user.is_dict() && (user.get("people").is_some_and(Value::truthy) || user.get("kids").is_some_and(Value::truthy)) {
            return true;
        }
    }
    if !the_others_mut(memory).is_empty() {
        return true;
    }
    for turn in last_turns(memory, 4) {
        let said = text::lower(&turn.user);
        if RELATIONS.iter().any(|one| word_re(&format!(r"\bmy\s+{}\b", pyre::escape(one))).is_match(&said)) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/ack.py:286 `a_person_named_alone(text, memory)`
///
/// The question a person named on its own asks for, or "".
pub fn a_person_named_alone(text_: &str, memory: &mut Memory) -> String {
    // The little words in front of the person are read off by the frames every ask about a
    // person uses (`named.the_one_asked_about`).
    let asked = named::the_one_asked_about(text_);
    let only = if asked.is_empty() { squash(text_) } else { asked };
    let Some(found) = A_PERSON_ALONE.match_(&squash(&only)).map(|m| (m.name_str("what").to_string(), m.name_str("det").to_string()))
    else {
        return String::new();
    };
    let (what, det) = found;
    let said = text::strip_chars(&flat(&what), " .,;:!?'").to_string();
    let said = WHICH_ONE.sub("", &said);
    // "your son" for a person who said "my son", and "the twins" for "the twins".
    let ours = ["my", "our"].contains(&text::lower(&det).as_str());
    let stem = if said.ends_with('s') { &said[..said.len() - 1] } else { "" };
    let several = said.ends_with('s') && RELATION_SET.contains(stem);
    let word = if several { stem.to_string() } else { said.clone() };
    // A person named by the place they hold rather than by a relation word.
    if let Some(place) = A_POSITION.get(&flat(&what)).filter(|p| !p.is_empty()) {
        if A_POSITION_MAYBE_A_THING.contains(place) && !_people_in_chat(memory) {
            return String::new();
        }
        let pool = A_POSITION_LINES
            .iter()
            .find(|(k, _)| k == place)
            .map(|(_, v)| v.clone())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| A_PERSON_ALONE_LINES.clone());
        let who = format!("{}{}", if ours { "Your " } else { "The " }, place);
        return pick(&pool, memory, &said)
            .replace("{Who}", &who)
            .replace("{they}", "they")
            .replace("{them}", "them")
            .replace("{do}", "do");
    }
    let group = match A_GROUP_ALONE.get(&said).filter(|g| !g.is_empty()) {
        Some(g) => g.clone(),
        None if several => format!("{}s", stem),
        None => String::new(),
    };
    if !group.is_empty() {
        let who = format!("{}{}", if ours { "Your " } else { "The " }, group);
        return py_format(&pick(&A_GROUP_ALONE_LINES, memory, &said), &[("Who", &who)]);
    }
    let relation = _a_relation(&word);
    if relation.is_empty() {
        return String::new();
    }
    _the_question_about(&relation, &said, ours, memory, "")
}

/// Python: bot/memory/ack.py:335 `_the_question_about(relation, said, ours, memory, name='')`
///
/// The question a friend asks about this person, said back their way.
pub fn _the_question_about(relation: &str, said: &str, ours: bool, memory: &mut Memory, name: &str) -> String {
    let pronoun = _the_pronoun(relation);
    let (they, them) = if pronoun.is_empty() {
        ("they".to_string(), "them".to_string())
    } else {
        let them = if pronoun == "he" { "him" } else { "her" };
        (pronoun.clone(), them.to_string())
    };
    let who = if name.is_empty() { format!("{}{}", if ours { "Your " } else { "The " }, relation) } else { name.to_string() };
    let known = _the_ages_held(memory).contains_key(relation); // an age they gave is not the question
    let pool = if known { &*A_KNOWN_AGE_LINES } else { &*A_PERSON_ALONE_LINES };
    let line = pick(pool, memory, said);
    let does = if they == "he" || they == "she" { "does" } else { "do" };
    line.replace("{Who}", &who).replace("{they}", &they).replace("{them}", &them).replace("{do}", does)
}

const POINTED_ONE: [&str; 6] = ["him", "her", "them", "they", "that one", "this one"];

/// Python: bot/memory/ack.py:359 `a_person_pointed_at(text, memory)`
///
/// The question about the person a "him" or "her" or "them" points at, or "".
pub fn a_person_pointed_at(text_: &str, memory: &mut Memory) -> String {
    let only = named::the_one_asked_about(text_);
    if only.is_empty() || A_POINTED_AT.match_(&flat(&only)).is_none() {
        return String::new();
    }
    let word = text::strip(&text::lower(&only)).to_string();
    // "the two of them" is two people, and the chat has to hold two before it means anybody.
    if !POINTED_ONE.contains(&word.as_str()) && _how_many_people(memory) < 2 {
        return String::new();
    }
    if !POINTED_ONE.contains(&word.as_str()) {
        let all_named = _the_people_pointed_at(memory);
        if all_named.len() >= 2 {
            let who = all_named[..2].join(" and ");
            let pool = if all_named[..2].iter().all(|one| _an_age_held(memory, one)) {
                &*A_TWO_ALONE_LINES
            } else {
                &*A_GROUP_ALONE_LINES
            };
            return py_format(&pick(pool, memory, &who), &[("Who", &who)]);
        }
    }
    let Some((relation, name)) = _the_person_pointed_at(&word, memory) else { return String::new() };
    _the_question_about(&relation, &relation, true, memory, &name)
}

/// Python: bot/memory/ack.py:389 `_how_many_people(memory)`
///
/// How many people of theirs the chat holds.
pub fn _how_many_people(memory: &mut Memory) -> usize {
    the_others_mut(memory)
        .iter()
        .filter(|entry| entry.is_dict() && RELATION_SET.contains(&text::lower(&field(entry, "who"))))
        .count()
}

/// Python: bot/memory/ack.py:396 `_the_people_pointed_at(memory)`
///
/// The names of the people the chat holds, in the order they were given, named only.
pub fn _the_people_pointed_at(memory: &mut Memory) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for entry in the_others_mut(memory) {
        if !entry.is_dict() {
            continue;
        }
        if !RELATION_SET.contains(&text::lower(&field(&entry, "who"))) {
            continue;
        }
        let name = text::strip(&field(&entry, "name")).to_string();
        if name.is_empty() || !name.chars().next().is_some_and(pyuni::ch_alpha) {
            continue;
        }
        let name = upper_first_char(&name);
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

/// `name[0].upper() + name[1:]`.
fn upper_first_char(s: &str) -> String {
    upper_first(s)
}

/// Python: bot/memory/ack.py:414 `_an_age_held(memory, name)`
///
/// Whether the chat already has an age for the person of this name.
pub fn _an_age_held(memory: &mut Memory, name: &str) -> bool {
    let wanted = text::lower(text::strip(name));
    the_others_mut(memory).iter().any(|entry| {
        entry.is_dict() && text::lower(text::strip(&field(entry, "name"))) == wanted && entry.get("age").is_some_and(Value::truthy)
    })
}

/// Python: bot/memory/ack.py:426 `_the_person_pointed_at(word, memory)`
///
/// (relation, name) for the person the word points at, or None when the chat has nobody.
pub fn _the_person_pointed_at(word: &str, memory: &mut Memory) -> Option<(String, String)> {
    let wanted: Option<&HashSet<String>> = match word {
        "him" => Some(&HE),
        "her" => Some(&SHE),
        _ => None,
    };
    let people: Vec<Value> = the_others_mut(memory).into_iter().filter(|e| e.is_dict()).collect();
    for entry in people.iter().rev() {
        let relation = text::lower(text::strip(&field(entry, "who")));
        if !RELATION_SET.contains(&relation) {
            continue;
        }
        if let Some(wanted) = wanted {
            if !wanted.contains(&relation) {
                continue;
            }
        }
        let mut name = text::strip(&field(entry, "name")).to_string();
        if !name.is_empty() && name.chars().next().is_some_and(pyuni::ch_alpha) {
            name = upper_first_char(&name);
        }
        return Some((relation, name));
    }
    None
}

/// Python: bot/memory/ack.py:444 `a_second_person(text, memory)`
///
/// The ask for the date of the fact this line leaves out, or "".
pub fn a_second_person(text_: &str, memory: &Memory) -> String {
    let Some(found) = A_SECOND_PERSON.match_(&squash(text_)).map(|m| m.name_str("who").to_string()) else {
        return String::new();
    };
    let who = _a_relation(&found);
    if who.is_empty() {
        return String::new();
    }
    let before = flat(&last_user(memory));
    if before.is_empty() {
        return String::new();
    }
    // The turn before has to be about a person, and about somebody else.
    let first = RELATIONS
        .iter()
        .find(|one| word_re(&format!(r"\bmy\s+{}", pyre::escape(one))).is_match(&before))
        .cloned()
        .unwrap_or_default();
    if first.is_empty() || first == who {
        return String::new();
    }
    if !(A_BIRTHDAY_TOLD.is_match(&before) || A_DATE_TOLD.is_match(&before)) {
        return String::new();
    }
    let said = format!("your {who}");
    if A_BIRTHDAY_TOLD.is_match(&before) {
        return format!("And {said}'s birthday \u{2014} when is that?");
    }
    let verb = A_PLAN_VERB.search(&before).map(|m| text::lower(m.name_str("verb")));
    let pronoun = _the_pronoun(&who);
    if let Some(verb) = verb {
        if !pronoun.is_empty() {
            return format!("And {said} \u{2014} when's {pronoun} {verb}?");
        }
    }
    format!("And {said} \u{2014} when is that?")
}

/// Python: bot/memory/ack.py:586 `_the_last_reply_gave_a_plan(memory)`
///
/// Whether the bot's last reply told the person something to do, or how to choose it.
pub fn _the_last_reply_gave_a_plan(memory: &Memory) -> bool {
    let said = turns(memory).iter().rev().find(|t| !text::strip(&t.bot).is_empty()).map(|t| flat(&t.bot)).unwrap_or_default();
    if said.is_empty() {
        return false;
    }
    if RE_DIGIT.is_match(&said) || said.contains(':') {
        return true;
    }
    if _A_PLAN_WAS_GIVEN.is_match(&said) {
        return true;
    }
    crate::memory::stuck::_the_answer_gave_a_step(memory)
}

/// Python: bot/memory/ack.py:603 `_a_limit_on_the_plan(read, memory)`
///
/// CONTRACT: typed `()`; the value is [`a_limit_on_the_plan_text`].
pub fn _a_limit_on_the_plan(read: &str, memory: &Memory) {
    let _ = a_limit_on_the_plan_text(read, memory);
}

/// The line's own limit put in front of the plan the last reply gave, or None.
pub fn a_limit_on_the_plan_text(read: &str, memory: &Memory) -> Option<String> {
    if !_the_last_reply_gave_a_plan(memory) {
        return None;
    }
    let found = STILL_TO_DO.search(read)?;
    let thing = text::strip_chars(&squash(found.str(1)), " .,!?:;").to_string();
    let words = n_words(&thing);
    // A whole clause after the verb is a story, not the one thing going first.
    if !(1..=4).contains(&words) || NOT_ONE_THING.is_match(&thing) {
        return None;
    }
    let pool: Vec<String> = THE_LIMIT_GOES_FIRST.iter().map(|line| py_format(line, &[("thing", &thing)])).collect();
    Some(pick(&pool, memory, read))
}

/// Python: bot/memory/ack.py:739 `_a_day_coming(clean)`
///
/// Which pool a line about a day ahead wants, or None when it is not one. (The Python function
/// has three unreachable lines after its last `return`; they do nothing.)
pub fn _a_day_coming(clean: &str) -> Option<Vec<String>> {
    let said = clean;
    if A_DAY_ASKED.is_match(said) || DAY_BEEN_AND_GONE.is_match(said) {
        return None;
    }
    if DAY_JUST_A_STATE.is_match(said) || A_FIGURE.is_match(said) {
        return None;
    }
    if A_DOING_TURNED_DOWN.is_match(said) {
        return None;
    }
    if !A_DOING.is_match(said) {
        return None;
    }
    if NOT_A_DAY_TO_LOOK_FORWARD_TO.is_match(said) {
        return Some(SORRY_TAIL.clone());
    }
    if DAY_FAR.is_match(said) {
        return Some(FAR_TAIL.clone());
    }
    if DAY_SOON.is_match(said) {
        return Some(SOON_TAIL.clone());
    }
    None
}

/// Python: bot/memory/ack.py:785 `reads_as_a_receipt(text)`
///
/// True when a reply is one of the frames with the person's words in it.
pub fn reads_as_a_receipt(text_: &str) -> bool {
    let said = flat(text_);
    if said.is_empty() {
        return false;
    }
    let readings = [said.clone(), _SPACED_DASH.sub(", ", &said)];
    if readings.iter().any(|reading| _one_frame(reading)) {
        return true;
    }
    for reading in &readings {
        let first = _SENTENCE.split(reading).into_iter().next().unwrap_or_default();
        if &first != reading && _one_frame(&first) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/ack.py:813 `_one_frame(said)`
///
/// True when the whole line is one frame with the person's words inside it.
pub fn _one_frame(said: &str) -> bool {
    for (front, back) in _SAID_BACK.iter() {
        if !front.is_empty() && !said.starts_with(front.as_str()) {
            continue;
        }
        if !back.is_empty() && !said.ends_with(back.as_str()) {
            continue;
        }
        // said[len(front):len(said) - len(back)], in characters
        let n = text::len(said);
        let (a, b) = (text::len(front), n.saturating_sub(text::len(back)));
        let middle = if a < b { text::slice(said, a as isize, Some(b as isize)) } else { String::new() };
        if !text::strip_chars(&middle, " \u{2014}-.:,;").is_empty() {
            return true;
        }
    }
    false
}

/// Python: bot/memory/ack.py:978 `_more_than_one_coming(said, start, word)`
///
/// True when the line names more than one person coming, from where "my"/"our" starts (`start`
/// is a byte offset into `said`).
pub fn _more_than_one_coming(said: &str, start: usize, word: &str) -> bool {
    if PLURAL_KIN.contains(word) {
        return true;
    }
    // A kin word said in the plural: the plural verb is what makes the "s" a plural.
    if word.ends_with('s') && RELATION_SET.contains(&word[..word.len() - 1]) && !RELATION_SET.contains(word) {
        return RE_ARE_PLURAL.is_match(said);
    }
    let mut rest = &said[start.min(said.len())..];
    if let Some(cut) = A_COMING_WORD.search(rest) {
        rest = &rest[..cut.start()];
    }
    // "and i"/"and we" is the speaker going with them, not a pair of visitors.
    if RE_AND_ME.is_match(rest) {
        return false;
    }
    RE_AND_OR_AMP.is_match(rest)
}

// ---------------------------------------------------------------------------------------------
// Trips, likes and limits.

data_re!(
    OBLIGED, A_STAY_PLAN, A_TRIP_PLAN, A_STAY_PERSON, _OPENS_WITH_AN_ASK, A_LIKE_OF_ONE_THING, A_LIKE_ECHO,
    A_LIKE_WITH_TWO_HALVES, A_PERSON_LIKED, _AN_ABILITY, WANTS_IT, _BULLIED, HURTS_A_PERSON, A_NAME_WAS_VIOLENT,
    AN_ASK, WHEN_TODAY, WHEN_TOMORROW, WHEN_DAYS, COUNTED, STEP_OPENS, A_EVENT, A_CLEAR, A_RESULT, A_MOVED,
    VALUE_UNIT_GLUED,
);
data_set!(_LIMIT_ONLY, BODY_STATE_WORDS, BODY_STATE_PHRASES, BODY_STATE_TAIL, VALUE_UNIT_WORDS, WHOLE_FREQUENCY);
data_pools!(LIMIT_FRAMES, LIMIT_AFTER_THE_LINE);
data_map!(A_MEASURE);

static PLAN_TURNS: LazyLock<usize> = LazyLock::new(|| data::get("bot.memory.ack.PLAN_TURNS").as_int().unwrap() as usize);
static SAID_BACK_WORDS: LazyLock<usize> =
    LazyLock::new(|| data::get("bot.memory.ack.SAID_BACK_WORDS").as_int().unwrap() as usize);
static FACTS_PLAN: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.facts.PLAN"));

/// `{word: number}` tables (`COUNT`, `CLOCK_WORDS`).
fn int_map(name: &str) -> HashMap<String, i64> {
    data::get(name).pairs().iter().map(|(k, v)| (py_str(k), v.as_int().unwrap_or(0))).collect()
}
static COUNT: LazyLock<HashMap<String, i64>> = LazyLock::new(|| int_map("bot.memory.ack.COUNT"));
static CLOCK_WORDS: LazyLock<HashMap<String, i64>> = LazyLock::new(|| int_map("bot.memory.ack.CLOCK_WORDS"));

inline_re!(RE_AZ_WORDS, r"[a-z']+", 0);
inline_re!(RE_AZ_CURLY_WORDS, "[a-z'\u{2019}]+", 0);
inline_re!(RE_CLOCK_TOKEN, r"\d{1,2}|[a-z]+", 0);
inline_re!(RE_FIRST_SENTENCE, r"(?<=[.!?])\s+", 0);

/// `facts.a_limit_on_the_plan(text)`: the limit a line puts on a plan - "move", "money",
/// "other" - or None.
pub(crate) fn facts_a_limit_on_the_plan(text_: &str) -> Option<String> {
    facts::a_limit_on_the_plan(text_)
}

/// Python: bot/memory/ack.py:1085 `reads_as_a_trip(clean)`
///
/// "trip", "occasion" (or "occasion_flat") or None: a plan that is coming, told to a friend.
pub fn reads_as_a_trip(clean: &str) -> Option<String> {
    if text::strip(clean).is_empty() {
        return None;
    }
    let flat_ = flat(clean);
    if !DATE_AHEAD.is_match(&flat_) || OBLIGED.is_match(&flat_) {
        return None;
    }
    if RE_AZ_WORDS.findall_str(&flat_).iter().any(|w| PAST_PLAN.contains(w)) {
        return None;
    }
    if text::starts_with_any(&flat_, &["my ", "her ", "his ", "their ", "she ", "he "]) {
        return None;
    }
    if !TRIP_VERB.is_match(&flat_) {
        return None;
    }
    let mut destination = DESTINATION.search(&flat_).map(|m| (m.start(), m.str(1).to_string()));
    if destination.is_none() && ARRIVE_VERB.is_match(&flat_) {
        destination = DESTINATION_ARRIVAL.search(&flat_).map(|m| (m.start(), m.str(1).to_string()));
    }
    let (start, place) = destination?;
    if NOT_A_TRIP.contains(&place) {
        return None;
    }
    // The four words after "to" are where an appointment, a relative or a job is named.
    let after_to: Vec<String> = RE_AZ_CURLY_WORDS.findall_str(&flat_[start..]).into_iter().skip(1).take(4).collect();
    let first = after_to.first().cloned().unwrap_or_default();
    if crate::memory::not_a_trip::not_a_destination(&after_to, &first) {
        return None; // the word after "to" is a doing, not a place
    }
    if after_to.iter().any(|w| NOT_A_TRIP.contains(w)) {
        return None;
    }
    if after_to.iter().any(|w| OCCASIONS.contains(w)) {
        if after_to.iter().any(|w| w == "shower") && !after_to.iter().any(|w| SHOWER.contains(w)) {
            return None;
        }
        return Some(if DREADED_OCCASION.is_match(&flat_) { "occasion_flat" } else { "occasion" }.into());
    }
    // A word that is not a place: "to the tip", "to my mum's", except the places with "the".
    let head = first;
    if THING_BEFORE.contains(&head) || text::ends_with_any(&head, &["'s", "\u{2019}s", "s'"]) {
        let named_ = after_to.get(1).cloned().unwrap_or_default();
        let two_word = after_to.iter().skip(1).take(2).cloned().collect::<Vec<_>>().join(" ");
        if !(head == "the" && (PLACES_WITH_THE.contains(&named_) || PLACES_WITH_THE.contains(&two_word))) {
            return None;
        }
    }
    if head.is_empty() {
        return None;
    }
    Some("trip".into())
}

/// Python: bot/memory/ack.py:1345 `_the_plan_in_hand(memory, clean='')`
///
/// Which plan the last few turns are on: "stay", "trip", or None.
pub fn _the_plan_in_hand(memory: &Memory, clean: &str) -> Option<String> {
    let mut lines: Vec<String> = Vec::new();
    if !text::strip(clean).is_empty() {
        lines.push(clean.to_string());
    }
    for turn in last_turns(memory, *PLAN_TURNS).iter().rev() {
        if !text::strip(&turn.user).is_empty() {
            lines.push(squash(&turn.user));
        }
    }
    if lines.iter().any(|said| A_STAY_PLAN.is_match(said) && A_STAY_PERSON.is_match(said)) {
        return Some("stay".into());
    }
    if lines.iter().any(|said| A_TRIP_PLAN.is_match(said) && facts_a_limit_on_the_plan(said).is_none()) {
        return Some("trip".into());
    }
    None
}

/// Python: bot/memory/ack.py:1423 `_a_like_of_one_thing(clean, memory=None)`
///
/// Whether the line's whole echo is one thing the person likes.
pub fn _a_like_of_one_thing(clean: &str, memory: &Memory) -> bool {
    let echo = _echo(clean, Some(memory));
    let Some(thing) = A_LIKE_ECHO.match_(&echo).map(|m| m.name_str("thing").to_string()) else { return false };
    if A_LIKE_WITH_TWO_HALVES.is_match(&thing) || A_PERSON_LIKED.is_match(&thing) {
        return false;
    }
    !RELATIONS.iter().any(|word| re_cached(&format!(r"\b{}\b", pyre::escape(word)), pyre::I).is_match(&thing))
}

/// Python: bot/memory/ack.py:1436 `_only_a_liking(found)`
///
/// Whether the line filed a like and nothing else worth an answer of its own.
pub fn _only_a_liking(found: &Value) -> bool {
    let rest = found
        .pairs()
        .iter()
        .any(|(k, v)| v.truthy() && !["tastes", "pets"].contains(&k.str()) && !k.str().ends_with("_frame"));
    found.get("tastes").is_some_and(Value::truthy) && !rest
}

/// Python: bot/memory/ack.py:1448 `a_liked_thing(found)`
///
/// The thing a stated like is about, or "" when the line filed no like.
pub fn a_liked_thing(found: &Value) -> String {
    let Some(tastes) = found.get("tastes") else { return String::new() };
    for entry in tastes.items() {
        if !entry.is_dict() || field(entry, "how") != "yes" {
            continue;
        }
        let thing = text::strip_chars(&squash(&field(entry, "thing")), " ,.:;!?").to_string();
        if !thing.is_empty()
            && !thing.contains('{')
            && !thing.contains('}')
            && n_words(&thing) <= 4
            && A_LIKE_OF_ONE_THING.match_(&thing).is_some()
        {
            return thing;
        }
    }
    String::new()
}

/// Python: bot/memory/ack.py:1485 `a_limit_reply(text, memory=None)`
///
/// CONTRACT: typed `()`; the value is [`a_limit_reply_text`].
pub fn a_limit_reply(text_: &str, memory: &Memory) {
    let _ = a_limit_reply_text(text_, memory);
}

/// The limit said back with what the bot will do with it, or None when the line is more.
pub fn a_limit_reply_text(text_: &str, memory: &Memory) -> Option<String> {
    let clean = squash(text_);
    if clean.is_empty() || clean.contains('?') {
        return None;
    }
    if _OPENS_WITH_AN_ASK.match_(&clean).is_some() {
        return None;
    }
    let family = facts_a_limit_on_the_plan(&clean)?;
    if !_AN_ABILITY.is_match(&clean) && !_a_plan_on_the_table(memory, &clean) {
        return None; // nothing is being planned: a remark the pools answer better
    }
    let found = _learned(text_, Some(memory));
    if found.truthy() && !found.pairs().iter().all(|(k, _)| _LIMIT_ONLY.contains(k.str())) {
        return None;
    }
    Some(_a_limit_reply(&clean, &family, memory))
}

/// Python: bot/memory/ack.py:1519 `_a_plan_on_the_table(memory, clean='')`
///
/// Whether there is a plan for the limit to be about.
pub fn _a_plan_on_the_table(memory: &Memory, clean: &str) -> bool {
    if WANTS_IT.is_match(clean) {
        return true;
    }
    let mut seen = 0;
    for turn in turns(memory).iter().rev() {
        let said = &turn.user;
        if text::strip(said).is_empty() {
            continue;
        }
        seen += 1;
        if seen > *PLAN_TURNS {
            return false;
        }
        if FACTS_PLAN.is_match(said) && facts_a_limit_on_the_plan(said).is_none() {
            return true;
        }
        if facts::extract(said, false).get("plan").is_some_and(Value::truthy) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/ack.py:1576 `_a_limit_reply(clean, family, memory)`
///
/// The limit said back, with what the bot will do with it.
pub fn _a_limit_reply(clean: &str, family: &str, memory: &Memory) -> String {
    let echo = _echo(clean, None);
    let body = if echo.is_empty() { clean.to_string() } else { echo };
    let mut family = family.to_string();
    if family == "move" && _the_plan_in_hand(memory, clean).as_deref() == Some("stay") {
        family = "move_stay".into();
    }
    if n_words(&body) > *SAID_BACK_WORDS {
        // A whole sentence is not a label for the limit: the reply starts with what it will do.
        if let Some((_, after)) = LIMIT_AFTER_THE_LINE.iter().find(|(k, _)| *k == family) {
            if !after.is_empty() {
                return _pick_ending(after, memory, &[]);
            }
        }
    }
    let frames = LIMIT_FRAMES
        .iter()
        .find(|(k, _)| *k == family)
        .map(|(_, v)| v.clone())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| LIMIT_FRAMES.iter().find(|(k, _)| k == "other").map(|(_, v)| v.clone()).expect("KeyError: 'other'"));
    let thing_cap = upper_first(&body);
    _pick_ending(&frames, memory, &[("thing", &body), ("Thing", &thing_cap)])
}

/// Python: bot/memory/ack.py:1591 `a_limit_for_the_plan(text, memory=None)`
///
/// CONTRACT: typed `()`; the value is [`a_limit_for_the_plan_text`].
pub fn a_limit_for_the_plan(text_: &str, memory: &Memory) {
    let _ = a_limit_for_the_plan_text(text_, memory);
}

/// A limit on the plan in hand, or None when no plan is in hand or no frame fits it.
pub fn a_limit_for_the_plan_text(text_: &str, memory: &Memory) -> Option<String> {
    let clean = squash(text_);
    if clean.is_empty() || clean.contains('?') || _OPENS_WITH_AN_ASK.match_(&clean).is_some() {
        return None;
    }
    let family = facts_a_limit_on_the_plan(&clean)?;
    // A reply about the plan needs a plan from an earlier turn.
    if !there_is_a_chat(memory) {
        return None;
    }
    // Only a guest who is staying.
    if _the_plan_in_hand(memory, "").as_deref() != Some("stay") {
        return None;
    }
    Some(_a_limit_reply(&clean, &family, memory))
}

/// Python: bot/memory/ack.py:1698 `reads_as_cruelty(said)`
///
/// Whether this line says somebody is doing the person harm.
pub fn reads_as_cruelty(said: &str) -> bool {
    let text_ = squash(said);
    if text_.is_empty() {
        return false;
    }
    if _BULLIED.is_match(&text_) || A_NAME_WAS_VIOLENT.is_match(&text_) {
        return true;
    }
    HURTS_A_PERSON.is_match(&text_)
}

/// Python: bot/memory/ack.py:1844 `reads_as_an_ask(line)`
///
/// Whether this line asks something, in a shape a person types.
pub fn reads_as_an_ask(line: &str) -> bool {
    line.contains('?') || AN_ASK.match_(text::strip(line)).is_some()
}

/// Python: bot/memory/ack.py:1895 `_shape_of(when)`
///
/// Which kind of clock this deadline is: today, tomorrow, days, or room.
pub fn _shape_of(when: &str) -> String {
    if WHEN_TODAY.is_match(when) {
        return "today".into();
    }
    if WHEN_TOMORROW.is_match(when) {
        return "tomorrow".into();
    }
    if let Some(found) = COUNTED.search(when) {
        let said = found.str(1);
        let number: i64 = match COUNT.get(&text::lower(said)) {
            Some(n) => *n,
            None => said.parse().expect("ValueError: invalid literal for int()"),
        };
        let unit = text::lower(found.str(2));
        if unit.starts_with("month") {
            return "room".into();
        }
        let most = if unit.starts_with("day") { 5 } else { 1 };
        return if number <= most { "days".into() } else { "room".into() };
    }
    if WHEN_DAYS.is_match(when) {
        return "days".into();
    }
    "room".into()
}

/// Python: bot/memory/ack.py:2038 `_clock_shape(when)`
///
/// Which half of the day the hour is in: "morning" or "after".
pub fn _clock_shape(when: &str) -> String {
    let said = text::lower(when);
    let Some(found) = RE_CLOCK_TOKEN.search(&said) else { return "morning".into() };
    let token = found.as_str();
    let hour = if pyuni::is_digit(token) { token.parse::<i64>().ok() } else { CLOCK_WORDS.get(token).copied() };
    let Some(hour) = hour else { return "morning".into() };
    if said.contains("pm") || said.contains("p.m") {
        return "after".into();
    }
    if said.contains("am") || said.contains("a.m") {
        return "morning".into();
    }
    if hour <= 3 {
        return "after".into();
    }
    if hour < 12 { "morning".into() } else { "after".into() }
}

/// Python: bot/memory/ack.py:2074 `_tells_them_what_to_do(said)`
///
/// Whether the answer just given was advice they can act on, rather than a list of facts.
pub fn _tells_them_what_to_do(said: &str) -> bool {
    let stripped = text::strip(said);
    let opening = match RE_FIRST_SENTENCE.search(stripped) {
        Some(m) => &stripped[..m.start()],
        None => stripped,
    };
    STEP_OPENS.match_(text::strip(opening)).is_some()
}

/// Python: bot/memory/ack.py:2120 `_a_bare_state(clean)`
///
/// Whether the line is nothing but a state of the body ("im bunged up", "im shaky").
pub fn _a_bare_state(clean: &str) -> bool {
    let replaced = clean.replace('-', " ");
    let mut words: Vec<String> =
        text::split_ref(&replaced).iter().map(|w| text::strip_chars(w, ".,!?;:").to_string()).filter(|w| !w.is_empty()).collect();
    if words.first().is_some_and(|w| w == "im" || w == "i'm" || w == "iam") {
        words.remove(0);
    } else if words.len() >= 2 && words[0] == "i" && words[1] == "am" {
        words.drain(..2);
    } else {
        return false;
    }
    if words.is_empty() {
        return false;
    }
    if BODY_STATE_PHRASES.contains(&words.join(" ")) {
        return true;
    }
    words.iter().any(|w| BODY_STATE_WORDS.contains(w)) && words.iter().all(|w| BODY_STATE_WORDS.contains(w) || BODY_STATE_TAIL.contains(w))
}

/// Python: bot/memory/ack.py:2236 `has_news(text)`
///
/// Whether the line carries something that happened, as opposed to how they feel.
pub fn has_news(text_: &str) -> bool {
    A_EVENT.is_match(text_) || A_CLEAR.is_match(text_) || A_RESULT.is_match(text_) || A_MOVED.is_match(text_)
}

/// Python: bot/memory/ack.py:2365 `_repaired(clean, memory)`
///
/// The line with the words that were wrong repaired, for the guards that match English: the
/// engine's own reading (`understood["plain"]`), and only when that reading is of this line.
pub fn _repaired(clean: &str, memory: &Memory) -> String {
    let Some(understood) = memory.understood.as_ref() else { return clean.to_string() };
    if understood.clean != text::lower(clean) {
        return clean.to_string(); // a reading of some other line: never decide on it
    }
    if understood.plain.is_empty() { clean.to_string() } else { understood.plain.clone() }
}

/// Python: bot/memory/ack.py:2451 `_the_values_measures(value)`
///
/// The kinds of measure the value is already said in: "10 km" -> {"distance"}.
pub fn _the_values_measures(value: &str) -> HashSet<String> {
    let mut found = HashSet::new();
    for word in text::split_ref(value) {
        let mut word = text::lower(text::strip_chars(word, ".,!?;:'\u{2019}"));
        if WHOLE_FREQUENCY.contains(&word) {
            found.insert("time".to_string());
        }
        if let Some(glued) = VALUE_UNIT_GLUED.match_(&word).map(|m| text::lower(m.str(1))) {
            word = glued;
        }
        if VALUE_UNIT_WORDS.contains(&word) {
            found.insert(A_MEASURE.get(&word).cloned().unwrap_or(word));
        }
    }
    found
}

// ---------------------------------------------------------------------------------------------
// A value put right.

data_re!(MAKE_A_VALUE, GIVEN_AWAY, GIVEN_ANIMAL, NOT_THEIRS, OFFERS_A_CHAT, A_DATE_ALONE, A_MONTH_ALONE, A_WEEKDAY, A_WORRIED_ABOUT_IT, A_MILESTONE, GREETING);
data_set!(VALUE_WORDS, UNITS_AFTER, DAYS, MONTHS);
data_strs!(
    CORRECTED_WITH_UNIT, CORRECTED_ALONE, GIVEN_AWAY_REPLIES, CHAT_OFFER_REPLY, A_DATE_ON_A_PLAN, A_DAY_ON_A_PLAN,
    A_DATE_ON_A_HARD_PLAN, A_DAY_ON_A_HARD_PLAN,
);

/// A frame phrase of `PUT_RIGHT_LEAD` / `PUT_RIGHT_TAIL`: its words, or None for an entry that
/// is a bare string in Python (`("it's")` is not a tuple), which never equals a tuple of words.
fn frame_phrases(name: &str) -> Vec<Option<Vec<String>>> {
    data::get(name).items().iter().map(|p| if p.is_str() { None } else { Some(p.strs()) }).collect()
}
static PUT_RIGHT_LEAD: LazyLock<Vec<Option<Vec<String>>>> = LazyLock::new(|| frame_phrases("bot.memory.ack.PUT_RIGHT_LEAD"));
static PUT_RIGHT_TAIL: LazyLock<Vec<Option<Vec<String>>>> = LazyLock::new(|| frame_phrases("bot.memory.ack.PUT_RIGHT_TAIL"));

inline_re!(RE_SPACE_COMMA, r"[\s,]+", 0);
inline_re!(RE_I_OR_WE_OPENS, r"\s*(?:i|we)\b", pyre::I);

/// Python: bot/memory/ack.py:2473 `_the_units_measure(unit)`
///
/// The kinds of measure the unit of the line before names: "every morning" -> {"time"}.
pub fn _the_units_measure(unit: &str) -> HashSet<String> {
    text::split_ref(unit)
        .iter()
        .filter_map(|w| A_MEASURE.get(&text::lower(text::strip_chars(w, ".,!?;:'\u{2019}"))).cloned())
        .filter(|m| !m.is_empty())
        .collect()
}

/// Python: bot/memory/ack.py:2483 `_the_unit_is_said_twice(value, unit)`
///
/// Whether saying the line's unit after this value would measure the same thing twice.
pub fn _the_unit_is_said_twice(value: &str, unit: &str) -> bool {
    let here = _the_values_measures(value);
    let there = _the_units_measure(unit);
    if !here.is_empty() && !there.is_empty() {
        return here.intersection(&there).next().is_some();
    }
    false
}

/// Python: bot/memory/ack.py:2521 `_without_a_frame(words, phrases, back=False)`
///
/// The words left when a frame phrase is taken off one end, and whether one was taken.
pub fn _without_a_frame(words: &[String], phrases: &[Option<Vec<String>>], back: bool) -> (Vec<String>, bool) {
    let mut words: Vec<String> = words.to_vec();
    let mut took = false;
    'outer: while !words.is_empty() {
        for phrase in phrases {
            let Some(phrase) = phrase else { continue };
            let n = phrase.len();
            let end: &[String] = if back { &words[words.len().saturating_sub(n)..] } else { &words[..n.min(words.len())] };
            let said: Vec<String> = end.iter().map(|w| text::lower(text::strip_chars(w, ".,!?;:'\u{2019}"))).collect();
            if &said == phrase {
                if back {
                    let keep = words.len().saturating_sub(n);
                    words.truncate(keep);
                } else {
                    words.drain(..n.min(words.len()));
                }
                took = true;
                continue 'outer;
            }
        }
        break;
    }
    (words, took)
}

/// Python: bot/memory/ack.py:2538 `_without_a_not(words)`
///
/// The words with a trailing "not <figure>" taken off, which is the figure being taken back.
pub fn _without_a_not(words: &[String]) -> (Vec<String>, bool) {
    let n = words.len();
    if n >= 3 && text::lower(text::strip_chars(&words[n - 2], ".,!?")) == "not" && RE_DIGIT.is_match(&words[n - 1]) {
        return (words[..n - 2].to_vec(), true);
    }
    (words.to_vec(), false)
}

/// Python: bot/memory/ack.py:2546 `_the_bare_value(clean)`
///
/// The figure a correction names when it has no verb in it, or "" when it is not one.
pub fn _the_bare_value(clean: &str) -> String {
    let words: Vec<String> =
        RE_SPACE_COMMA.split(text::strip_chars(clean, " .!?")).into_iter().filter(|w| !w.is_empty()).collect();
    if words.is_empty() {
        return String::new();
    }
    let (words, led) = _without_a_frame(&words, &PUT_RIGHT_LEAD, false);
    let (words, _) = _without_a_not(&words);
    let (words, tailed) = _without_a_frame(&words, &PUT_RIGHT_TAIL, true);
    if !(led || tailed) {
        return String::new();
    }
    if words.is_empty() || words.len() > 4 {
        return String::new();
    }
    text::strip_chars(&words.join(" "), " .,!?").to_string()
}

/// Python: bot/memory/ack.py:2566 `value_put_right(clean, memory)`
///
/// The correction said back, or "" when the line is not one that only puts a value right.
pub fn value_put_right(clean: &str, memory: &Memory) -> String {
    let made = MAKE_A_VALUE.match_(clean).map(|m| m.name_str("value").to_string());
    let mut bare = String::new();
    if made.is_none() {
        bare = _the_bare_value(clean);
        if bare.is_empty() {
            return String::new();
        }
    }
    if _the_line_before(memory).is_empty() {
        return String::new(); // nothing said before it, so there is no value to put right
    }
    let value = text::strip_chars(&squash(made.as_deref().unwrap_or(&bare)), " .").to_string();
    if value.is_empty() || n_words(&value) > 4 {
        return String::new();
    }
    let words: Vec<String> = text::split_ref(&value).iter().map(|w| text::lower(text::strip_chars(w, ".,!?'\u{2019}"))).collect();
    if !bare.is_empty() {
        // A correction with no verb has to be a figure or a frequency said plainly.
        if !(RE_DIGIT.is_match(&value) || words.iter().all(|w| VALUE_WORDS.contains(w) || WHOLE_FREQUENCY.contains(w))) {
            return String::new();
        }
    } else if !(RE_DIGIT.is_match(&value)
        || words.iter().all(|w| VALUE_WORDS.contains(w) || DAYS.contains(w) || MONTHS.contains(w) || WHOLE_FREQUENCY.contains(w)))
    {
        return String::new();
    }
    let unit = _the_unit_before(memory);
    if unit.is_empty() {
        return String::new(); // nothing in the line before is measured in anything
    }
    let unit = _without_a_unit_already_in_the_value(&value, &unit);
    let value_cap = upper_first(&value);
    if _the_unit_is_said_twice(&value, &unit) {
        return py_format(&pick(&CORRECTED_ALONE, memory, clean), &[("value", &value), ("Value", &value_cap)]);
    }
    py_format(&pick(&CORRECTED_WITH_UNIT, memory, clean), &[("value", &value), ("Value", &value_cap), ("unit", &unit)])
}

/// Python: bot/memory/ack.py:2615 `_without_a_unit_already_in_the_value(value, unit)`
///
/// The unit of the line before with the word the value already ends in taken off.
pub fn _without_a_unit_already_in_the_value(value: &str, unit: &str) -> String {
    let unit_words = text::split_ref(unit);
    let value_words = text::split_ref(value);
    if unit_words.len() < 2 || value_words.is_empty() {
        return unit.to_string();
    }
    let last = text::lower(text::strip_chars(value_words[value_words.len() - 1], ".,!?;:'\u{2019}"));
    if last == text::lower(text::strip_chars(unit_words[0], ".,!?;:'\u{2019}")) {
        return unit_words[1..].join(" ");
    }
    unit.to_string()
}

/// Python: bot/memory/ack.py:2633 `_the_line_before(memory)`
///
/// The line the person said just before, or "" when there is none.
pub fn _the_line_before(memory: &Memory) -> String {
    last_user(memory)
}

/// Python: bot/memory/ack.py:2645 `_the_unit_before(memory)`
///
/// The words the line before measures its figure in: "three times a week" -> "times a week".
pub fn _the_unit_before(memory: &Memory) -> String {
    let said = _the_line_before(memory);
    if text::strip(&said).is_empty() {
        return String::new();
    }
    let words: Vec<String> =
        text::split_ref(&said).iter().map(|w| text::strip_chars(w, ".,!?;:").to_string()).filter(|w| !w.is_empty()).collect();
    for index in 0..words.len().saturating_sub(1) {
        let figure = text::lower(&words[index]);
        if ["a", "an", "the"].contains(&figure.as_str()) {
            continue; // "a month" is the unit of "twice a month", not the figure
        }
        if !(RE_DIGIT.is_match(&figure) || VALUE_WORDS.contains(&figure) || DAYS.contains(&figure) || MONTHS.contains(&figure)) {
            continue;
        }
        let mut tail: Vec<String> = Vec::new();
        for following in words.iter().skip(index + 1).take(3) {
            let low = text::lower(following);
            if UNITS_AFTER.contains(&low) && tail.len() < 3 {
                tail.push(low);
                continue;
            }
            break;
        }
        if !tail.is_empty() {
            return tail.join(" ");
        }
    }
    String::new()
}

/// `kind[:-1] if kind.endswith("s") and not kind.endswith("ss") else kind`.
fn singular(kind: &str) -> String {
    if kind.ends_with('s') && !kind.ends_with("ss") { kind[..kind.len() - 1].to_string() } else { kind.to_string() }
}

/// Python: bot/memory/ack.py:2710 `_the_animal_given_away(text, memory=None)`
///
/// The animal the line hands over, as a word to put in a sentence ("dog"), or "".
pub fn _the_animal_given_away(text_: &str, memory: &Memory) -> String {
    if let Some(found) = GIVEN_ANIMAL.search(text_) {
        return singular(&text::lower(found.str(1)));
    }
    let said = text::lower(text_);
    let records = memory.get("denial").filter(|d| d.is_dict()).and_then(|d| d.get("pets"));
    for record in records.map(|r| r.items()).unwrap_or(&[]) {
        if !record.is_dict() {
            continue;
        }
        let name = text::lower(text::strip(&field(record, "name")));
        if !name.is_empty() && word_re(&format!(r"\b{}\b", pyre::escape(&name))).is_match(&said) {
            let kind = field(record, "animal");
            let kind = text::lower(if kind.is_empty() { "pet" } else { &kind });
            return singular(&kind);
        }
    }
    String::new()
}

/// Python: bot/memory/ack.py:2734 `pet_given_away(text, memory=None)`
///
/// CONTRACT: typed `()`; the value is [`pet_given_away_text`].
pub fn pet_given_away(text_: &str, memory: &Memory) {
    let _ = pet_given_away_text(text_, memory);
}

/// The warm line for an animal given to a new home, or None when this is another line.
pub fn pet_given_away_text(text_: &str, memory: &Memory) -> Option<String> {
    let clean = squash(text_);
    if clean.is_empty() || !GIVEN_AWAY.is_match(&clean) {
        return None;
    }
    if NOT_THEIRS.is_match(&clean) && RE_I_OR_WE_OPENS.match_(&clean).is_none() {
        return None; // somebody else's pet, told as their news rather than the user's
    }
    let kind = _the_animal_given_away(&clean, memory);
    if kind.is_empty() {
        return None;
    }
    Some(py_format(&pick(&GIVEN_AWAY_REPLIES, memory, &clean), &[("animal", &kind)]))
}

/// Python: bot/memory/ack.py:2785 `_offers_a_chat(clean, memory=None)`
///
/// CONTRACT: typed `()`; the value is [`offers_a_chat_text`].
pub fn _offers_a_chat(clean: &str, memory: &Memory) {
    let _ = offers_a_chat_text(clean, memory);
}

/// A short warm yes to a line that came to talk, never the line said back.
pub fn offers_a_chat_text(clean: &str, memory: &Memory) -> Option<String> {
    OFFERS_A_CHAT.match_(clean)?;
    for turn in last_turns(memory, 3) {
        let said = text::lower(&turn.user);
        if !said.is_empty()
            && (crate::compose::badnews::reads_as_bad_news(&said) || crate::compose::loss::reads_as_loss(&said, None, false))
        {
            return None;
        }
    }
    Some(pick(&CHAT_OFFER_REPLY, memory, ""))
}

/// Python: bot/memory/ack.py:2859 `_the_plan_a_date_belongs_to(text)`
///
/// The word for the plan a bare date is the date of, and its kind, or ("", "") for none.
pub fn _the_plan_a_date_belongs_to(text_: &str) -> (String, String) {
    let said = flat(text_);
    if said.is_empty() {
        return (String::new(), String::new());
    }
    let words = RE_AZ_WORDS.findall_str(&said);
    if let Some(occasion) = words.iter().find(|w| OCCASIONS.contains(*w)) {
        return (occasion.clone(), "occasion".into());
    }
    if A_MILESTONE.is_match(&said) && word_re(r"\b(?:my|our)\b").is_match(&said) {
        return ("birthday".into(), "occasion".into());
    }
    if let Some(diary) = DIARY_THING.search(&said) {
        return (text::lower(diary.as_str()), "diary".into());
    }
    // A trip with the date left out: the same guards as `reads_as_a_trip`.
    if TRIP_VERB.is_match(&said)
        && DESTINATION.is_match(&said)
        && !OBLIGED.is_match(&said)
        && !words.iter().any(|w| PAST_PLAN.contains(w))
    {
        let head = text::lower(DESTINATION.search(&said).unwrap().str(1));
        if !NOT_A_TRIP.contains(&head) && !THING_BEFORE.contains(&head) {
            return ("holiday".into(), "trip".into());
        }
    }
    (String::new(), String::new())
}

/// Python: bot/memory/ack.py:2888 `_a_date_on_the_thing_in_hand(clean, memory)`
///
/// CONTRACT: typed `()`; the value is [`a_date_on_the_thing_in_hand_text`].
pub fn _a_date_on_the_thing_in_hand(clean: &str, memory: &Memory) {
    let _ = a_date_on_the_thing_in_hand_text(clean, memory);
}

/// The plan's own date, said back as what the time means for it, or None.
pub fn a_date_on_the_thing_in_hand_text(clean: &str, memory: &Memory) -> Option<String> {
    let said = flat(clean);
    let when = A_DATE_ALONE.match_(&said).map(|m| flat(m.name_str("when")))?;
    let record = topics::current(memory);
    let text_ = flat(&py_str_or_empty(record.and_then(|r| r.get("text"))));
    if text_.is_empty() {
        return None;
    }
    if when.is_empty() || RE_AZ_WORDS.findall_str(&text_).contains(&when) {
        return None; // the date is already in the record's own words
    }
    let (plan, kind) = _the_plan_a_date_belongs_to(&text_);
    if plan.is_empty() {
        return None;
    }
    let thing = upper_first(&when);
    let mut so_far = vec![text_.clone()];
    so_far.extend(last_turns(memory, 4).iter().map(|t| text::lower(&t.user)));
    let worried = A_WORRIED_ABOUT_IT.is_match(&so_far.join(" "));
    let a_day = A_MONTH_ALONE.match_(&when).is_some();
    let made = if kind == "diary" {
        if A_WEEKDAY.match_(&when).is_some() {
            format!("the {plan} on {thing}")
        } else if a_day {
            format!("the {plan} in {thing}")
        } else {
            format!("the {plan} {thing}")
        }
    } else if a_day {
        let first = thing.chars().next().map(|c| c.to_lowercase().to_string()).unwrap_or_default();
        let article = if "aeiou".contains(first.as_str()) && !first.is_empty() { "an " } else { "a " };
        format!("{article}{thing} {plan}")
    } else {
        format!("the {plan} {}", text::lower(&thing))
    };
    let made = upper_first(&made);
    let pool = if a_day {
        if worried { &*A_DATE_ON_A_HARD_PLAN } else { &*A_DATE_ON_A_PLAN }
    } else if worried {
        &*A_DAY_ON_A_HARD_PLAN
    } else {
        &*A_DAY_ON_A_PLAN
    };
    Some(py_format(&pick(pool, memory, ""), &[("Thing", &made)]))
}

/// Python: bot/memory/ack.py:2947 `_after_the_hello(clean)`
///
/// What the line says after the hello, or "" when the hello is all of it.
pub fn _after_the_hello(clean: &str) -> String {
    match GREETING.match_(clean) {
        Some(found) => text::lstrip_chars(&clean[found.end()..], " ,.!-:;").to_string(),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------------------------
// The echo itself.

data_re!(
    MEET_GREET, SELF_INTRO, WANTS_ANSWER, DEATH_WORDS, WOULD_HAVE_BEEN, BODY, BLEED, A_FEELING, GOOD_MOOD,
    A_CHILD_IN_THE_LINE, AS_WELL, REMARK_BACK, REMARK_STILL_TO_COME, REMARK_ALREADY_PAID,
    GROWING, REMARK_TWO_SIDED,
    REMARK_JUDGES_THE_PRICE, REMARK_FIGURE, _HOLE, _NOT_WORDS, NO_REASON,
    A_CLAUSE_OF_ITS_OWN, AN_AGE_ALONE, THIRD_PERSON, THE_OLD_ONE, REPLACING, NOT_THE_HOURS, WHEN_DETAIL,
);
data_strs!(
    THE_STORY_GOES_ON, WISH_FOR_AN_ANIMAL, ANIMAL_COMING, A_BREED_BEING_CHOSEN, ANIMAL_HAD_ONCE,
    ANIMAL_TAKEN_BACK, PET_TAIL_COMING, PET_TAIL_COMING_MANY, REMARK_PRICE_AHEAD, REMARK_FIGURE_REPLIES,
    LENGTH_TAIL, SINCE_TAIL, OLD_ONE_TAIL, WHEN_TODAY_TAIL, WHEN_TOMORROW_TAIL, WHEN_DAYS_TAIL,
    WHEN_ROOMY_TAIL,
);
data_set!(PERSON_WORDS, LENGTH_KEEPS);
data_pools!(REMARK_REPLIES);
data_map!(CONDITION_FOOD);

static RELATION_WHOS_AGE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.facts.RELATION_WHOS_AGE"));
static A_DISMISSAL: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.fallback_data.A_DISMISSAL"));
static NARROW_A_LENGTH: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.narrow.A_LENGTH"));
static LENGTH_NAMED_DAYS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.narrow.LENGTH_NAMED_DAYS"));

/// `REMARK_KINDS`: the remark patterns with the kind each one reads, in order.
static REMARK_KINDS: LazyLock<Vec<(&'static Re, String)>> = LazyLock::new(|| {
    data::get("bot.memory.ack.REMARK_KINDS")
        .items()
        .iter()
        .map(|pair| (pair.items()[0].as_re().unwrap(), pair.items()[1].str().to_string()))
        .collect()
});

inline_re!(RE_HAND_BACK_WORDS, r"[a-z0-9']+", 0);

fn facts_reported_clause(clean: &str) -> Option<String> {
    facts::_reported_clause(clean)
}

/// `trouble.reads_as_trouble(read)`: the reading's shape, or None.
fn trouble_reading(read: &str) -> Option<String> {
    crate::compose::trouble::reads_as_trouble(read, None, "")
}

/// CONTRACT: `opening::the_opening` is typed `()`; Python returns the opening question or None.
/// The call is made and its value read as None until the contract carries one.
fn opening_the_opening(thing: &str, clean: &str, memory: &Memory) -> Option<String> {
    crate::memory::opening::the_opening(thing, clean, memory);
    None
}

/// `word in (facts.extract(...) or {})`: the keys of a facts dict.
fn keys_of(found: &Value) -> Vec<String> {
    found.pairs().iter().map(|(k, _)| py_str(k)).collect()
}

/// Python: bot/memory/ack.py:2955 `reply(text, memory)`
///
/// A short echo of the fact they just gave, or None when this is not that kind of line.
pub fn reply(text_: &str, memory: &mut Memory) -> Option<String> {
    let mut text_ = text_.to_string();
    let mut clean = squash(&text_);
    if clean.is_empty() || clean.contains('?') {
        return None;
    }
    if let Some(dated) = a_date_on_the_thing_in_hand_text(&clean, memory).filter(|d| !d.is_empty()) {
        return Some(dated); // "its in july" after "im going to a wedding"
    }
    // A clock time somebody has to be somewhere by is the composer's.
    if crate::compose::fallback_chain::_the_morning_has_a_clock(&text::lower(&clean)) {
        return None;
    }
    // A line that takes the last thing back is the composer's dismissal.
    if A_DISMISSAL.match_(&clean).is_some() {
        return None;
    }
    // "i told you my interview is in june" is the fact said again with a frame around it.
    if let Some(reported) = facts_reported_clause(&clean).filter(|r| !r.is_empty()) {
        if facts::extract(&reported, false).truthy() {
            clean = squash(&reported);
            text_ = clean.clone();
        }
    }
    // Advice somebody else gave them, and an errand asked of the bot, are not facts to file.
    if let Some(understood) = memory.understood.as_ref() {
        if understood.suggestion.is_some() {
            return None;
        }
        if understood.errand.as_deref().is_some_and(|e| !e.is_empty()) {
            return None;
        }
    }
    let mut read = _repaired(&clean, memory);
    // A hello in front of a fact is the hello and the fact.
    if GREETING.is_match(&clean) {
        let rest = _after_the_hello(&clean);
        if rest.is_empty() || !facts::about_themselves(&rest).is_some_and(|v| !v.is_empty()) {
            return None;
        }
        clean = rest.clone();
        text_ = rest;
        read = _repaired(&clean, memory);
    } else if MEET_GREET.is_match(&clean) || SELF_INTRO.is_match(&clean) {
        return None;
    }
    if let Some(had) = the_job_they_had_text(&clean, memory).filter(|h| !h.is_empty()) {
        return Some(had); // the job they had, not a day to give a verdict on
    }
    let alone = a_person_named_alone(&clean, memory);
    if !alone.is_empty() {
        return Some(alone);
    }
    if WANTS_ANSWER.is_match(&read) && !RELATION_WHOS_AGE.is_match(&read) {
        return None; // asking, not telling
    }
    if _a_result_is_on_the_table(&clean) {
        return None; // a result is the composer's: "What did they say?"
    }
    if let Some(limit) = a_limit_on_the_plan_text(&read, memory).filter(|l| !l.is_empty()) {
        return Some(limit);
    }
    let second = a_second_person(&clean, memory);
    if !second.is_empty() {
        return Some(second);
    }
    let corrected = value_put_right(&clean, memory);
    if !corrected.is_empty() {
        return Some(corrected);
    }
    if let Some(aged) = the_old_one_aged_text(&clean, memory).filter(|a| !a.is_empty()) {
        return Some(aged);
    }
    if !_statement(memory) {
        return None;
    }
    if let Some(given) = pet_given_away_text(&clean, memory).filter(|g| !g.is_empty()) {
        return Some(given);
    }
    if trouble_reading(&read).is_some() {
        return None; // a fault to troubleshoot is the composer's turn
    }
    if let Some(limit) = a_limit_reply_text(&text_, memory) {
        return Some(limit);
    }
    if _no_subject(&clean, memory) {
        return None;
    }
    if crate::compose::badnews::reads_as_bad_news(&text::lower(&read)) || _a_loss(&text_, memory, &read) {
        return None;
    }
    if _bad_news_just_before(&read, memory) {
        return Some(pick(&THE_STORY_GOES_ON, memory, &read));
    }
    if crate::compose::badnews::reads_like_a_symptom(&text::lower(&read)) {
        return None;
    }
    if crate::compose::goodnews::reads_as_good_news(&text::lower(&read)) {
        return None;
    }
    if crate::compose::goodnews::reads_as_settled(&text::lower(&clean))
        && crate::compose::goodnews::worry_on_the_table(memory, &Value::Int(4))
    {
        return None;
    }
    if DEATH_WORDS.is_match(&read) {
        return None;
    }
    if WOULD_HAVE_BEEN.is_match(&read) || BODY.is_match(&read) || BLEED.is_match(&read) {
        return None;
    }
    if A_FEELING.is_match(&clean) {
        return None;
    }
    if _said_to_the_bot(&clean) {
        return None;
    }
    if _a_bare_state(&text::lower(&clean)) {
        return None;
    }
    if GOOD_MOOD.is_match(&clean) && !has_news(&clean) {
        return None;
    }
    if let Some(pool) = nobody_around_pool(&text::lower(&clean), memory) {
        return Some(pick(&pool, memory, &clean));
    }
    if let Some(deadline) = a_deadline_text(&clean, &read, memory).filter(|d| !d.is_empty()) {
        return Some(deadline);
    }
    if let Some(clock) = a_clock_time_text(&clean, &read, memory).filter(|c| !c.is_empty()) {
        return Some(clock);
    }
    if _after_a_feeling(&clean, memory) {
        // The reason for the feeling they just gave.
        let thing = _echo(&clean, None);
        if !thing.is_empty() {
            let line = pick(&EXPLAIN_TAIL, memory, &clean);
            return Some(py_format(&line, &[("thing", &thing), ("Thing", &upper_first(&thing))]));
        }
    }
    if let Some(lasting) = the_length_text(&clean, memory).filter(|l| !l.is_empty()) {
        return Some(lasting);
    }
    if let Some(hours) = the_hours_text(&clean, &read, memory).filter(|h| !h.is_empty()) {
        return Some(hours);
    }
    if let Some(chat) = offers_a_chat_text(&clean, memory).filter(|c| !c.is_empty()) {
        return Some(chat);
    }
    if let Some(job) = _a_job_to_do(&clean, memory).filter(|j| !j.is_empty()) {
        return Some(job);
    }
    let found = _learned(&text_, Some(memory));
    if found.truthy() && _only_a_wish(&found) {
        // An animal they would like is named back and asked about.
        let wanted = format!("a {}", _the_wished_animal(&found, &clean));
        let line = pick(&WISH_FOR_AN_ANIMAL, memory, &clean);
        return Some(py_format(&line, &[("Thing", &upper_first(&wanted)), ("thing", &wanted)]));
    }
    if found.truthy() {
        if let Some(nobody) = who_the_figure_is_text(&found, memory, &text::lower(&clean)).filter(|n| !n.is_empty()) {
            return Some(nobody);
        }
    }
    if found.truthy() && keys_of(&found).iter().all(|k| k == "age" || k == "height") && !_only_the_figure(&clean) {
        return None; // an age said with a whole life around it is not the thing to say back
    }
    let animal_wanted = _an_animal_wanted(&clean);
    if !animal_wanted.is_empty() && !_on_record(&animal_wanted, memory) {
        return Some(_what_a_wanted_animal_needs(&animal_wanted, &clean, memory));
    }
    let coming = if found.truthy() { _the_animal_coming(&found) } else { String::new() };
    if !coming.is_empty() {
        let line = pick(&ANIMAL_COMING, memory, &coming);
        return Some(py_format(&line, &[("animal", &coming)]));
    }
    let planned = _an_animal_only_planned(&found, &clean, memory);
    if !planned.is_empty() {
        let first_word = text::split_ref(&planned).first().copied().unwrap_or("");
        let said = if first_word == "a" || first_word == "an" {
            planned.clone()
        } else if planned.chars().next().is_some_and(|c| "aeiou".contains(c)) {
            format!("an {planned}")
        } else {
            format!("a {planned}")
        };
        let line = pick(&A_BREED_BEING_CHOSEN, memory, &clean);
        return Some(py_format(&line, &[("animal", &said), ("Animal", &upper_first(&said))]));
    }
    if found.truthy() && (_an_animal_taken_back(&found) || _an_animal_turned_down(&found, memory)) {
        let animal = _the_taken_back_animal(&found);
        if !animal.is_empty() && !_on_record(&animal, memory) {
            let fields = [("animal", animal.as_str()), ("Animal", &upper_first(&animal))];
            if _an_animal_they_had_once(&clean) {
                return Some(py_format(&pick(&ANIMAL_HAD_ONCE, memory, &animal), &fields));
            }
            return Some(py_format(&pick(&ANIMAL_TAKEN_BACK, memory, &animal), &fields));
        }
        return None;
    }
    if let Some(remark) = _a_remark_on_what_was_said(&clean, memory).filter(|r| !r.is_empty()) {
        // A remark on the turn before is answered, and noted as one that hands the subject back.
        _a_hand_back(&remark, memory);
        return Some(remark);
    }
    if let Some(another) = another_of_what_we_said_text(&clean, memory).filter(|a| !a.is_empty()) {
        return Some(another);
    }
    let more = _adds_one_more_allergen(&text_, Some(memory));
    let routine = memory.understood.as_ref().is_some_and(|u| u.routine);
    if more.is_empty() && routine {
        if let Some(habit) = _plain(&clean, memory).filter(|h| !h.is_empty()) {
            return Some(habit);
        }
    }
    let nothing_to_say_back = !found.truthy() || (more.is_empty() && !_tells_it(&clean, &found));
    let said = if nothing_to_say_back { a_thing_they_have_text(&clean, memory) } else { None };
    if let Some(said) = said.filter(|s| !s.is_empty()) {
        return Some(_the_opening_instead(Some(&said), &clean, memory).unwrap_or(said));
    }
    if nothing_to_say_back {
        // Nothing in the line to file, or a fact that cannot be said back: a habit still is.
        let plain = _plain(&clean, memory);
        return _the_opening_instead(plain.as_deref(), &clean, memory).or(plain);
    }
    let pets = found.get("pets").cloned().unwrap_or(Value::None);
    let only_pets = keys_of(&found) == vec!["pets".to_string()];
    let pet_line = pets.truthy() && more.is_empty() && only_pets && !AS_WELL.is_match(&clean) && !A_CHILD_IN_THE_LINE.is_match(&clean);
    let ill = !A_CHILD_IN_THE_LINE.is_match(&clean) && crate::compose::badnews::reads_for_a_pet(&text::lower(&clean));
    if pet_line && ill {
        return None; // the vet's line answers it
    }
    if pet_line && !_nothing_added_but_the_pet(&clean, &pets) {
        return None; // the line says how the animal is
    }
    if pet_line && !_a_pet_that_is_all_right(&clean, &pets) {
        let (said, many) = _the_pets_said(&pets, &clean);
        if !said.is_empty() {
            let pool = if _the_pets_are_coming(&pets, memory) {
                if many { &*PET_TAIL_COMING_MANY } else { &*PET_TAIL_COMING }
            } else if many {
                &*PET_TAIL_MANY
            } else {
                &*PET_TAIL_ONE
            };
            let line = pick(pool, memory, &clean);
            return Some(py_format(&line, &[("thing", &said), ("Thing", &upper_first(&said))]));
        }
    }
    let kids = _children_with_ages(&text_);
    let allergies: Vec<String> = found.get("allergies").map(|a| a.strs()).unwrap_or_default();
    let thing = if !kids.is_empty() {
        _children_said(&kids)
    } else if !more.is_empty() && allergies == more {
        format!("you're allergic to {} too", more.join(" and "))
    } else {
        _echo(&clean, Some(memory))
    };
    if thing.is_empty() {
        return None;
    }
    let condition = text::lower(&py_str_or_empty(found.get("condition")));
    let food = CONDITION_FOOD.get(&condition).cloned().unwrap_or_else(|| "that".to_string());
    let thing_cap = upper_first(&thing);
    let fields = [("thing", thing.as_str()), ("Thing", thing_cap.as_str()), ("food", food.as_str())];
    if let Some(plan) = _plan_tails(&found, &clean, memory).filter(|p| !p.is_empty()) {
        return Some(_pick_ending(&plan, memory, &fields));
    }
    let frames = _frames(&found, _heavy(memory), &clean, memory);
    if frames == *FRAMES {
        // The first line of a chat gets the question a friend asks about the fact itself.
        if let Some(opened) = opening_the_opening(&thing, &clean, memory).filter(|o| !o.is_empty()) {
            return Some(opened);
        }
    }
    Some(_pick_ending(&frames, memory, &fields))
}

/// Python: bot/memory/ack.py:3451 `_the_opening_instead(said, clean, memory)`
///
/// The question about the fact, where a receipt of the line is all the reply would be.
pub fn _the_opening_instead(said: Option<&str>, clean: &str, memory: &Memory) -> Option<String> {
    let said = said.filter(|s| !s.is_empty())?;
    if !reads_as_a_receipt(said) {
        return None;
    }
    let thing = _echo(clean, Some(memory));
    if thing.is_empty() {
        return None;
    }
    opening_the_opening(&thing, clean, memory)
}

/// Python: bot/memory/ack.py:3465 `_a_hand_back(reply, memory)`
///
/// Note a reply that answered a remark, so the turn after it can tell it from a new subject.
pub fn _a_hand_back(reply: &str, memory: &mut Memory) {
    let low = text::lower(reply).replace('\u{2019}', "'");
    let said = RE_HAND_BACK_WORDS.findall_str(&low).join(" ");
    if said.is_empty() {
        return;
    }
    if !matches!(memory.get("hand_backs"), Some(Value::List(_))) {
        memory.set("hand_backs", Value::list());
    }
    let kept = memory.get_mut("hand_backs").unwrap();
    kept.push(said);
    if let Some(items) = kept.items_mut() {
        let extra = items.len().saturating_sub(4);
        items.drain(..extra);
    }
}

/// Python: bot/memory/ack.py:3488 `_a_remark_on_what_was_said(clean, memory)`
///
/// The reply to a one-line remark about the turn before, or None when the line is not one.
pub fn _a_remark_on_what_was_said(clean: &str, memory: &Memory) -> Option<String> {
    if turns(memory).is_empty() {
        return None;
    }
    let body = text::strip(clean);
    if n_words(body) > 9 {
        return None;
    }
    if REMARK_BACK.match_(body).is_none() || body.contains('?') {
        return None;
    }
    if REMARK_TWO_SIDED.is_match(body) {
        return None;
    }
    let topic_text = match memory.get("topic") {
        Some(topic) if topic.is_dict() => field(topic, "text"),
        _ => String::new(),
    };
    for (pattern, kind) in REMARK_KINDS.iter() {
        if !pattern.is_match(body) {
            continue;
        }
        if kind == "gone" && !GROWING.is_match(&format!("{} {}", body, topic_text)) {
            continue;
        }
        if kind == "expense" && !REMARK_JUDGES_THE_PRICE.is_match(body) {
            // A figure on its own is answered as the figure.
            if !REMARK_FIGURE.is_match(body) {
                continue;
            }
            return Some(pick(&REMARK_FIGURE_REPLIES, memory, body));
        }
        let mut options: Vec<String> =
            REMARK_REPLIES.iter().find(|(k, _)| k == kind).map(|(_, v)| v.clone()).expect("KeyError: remark kind");
        if kind == "expense" && _a_price_still_to_come(body, &topic_text) {
            options = REMARK_PRICE_AHEAD.clone();
        }
        if _asked_last(memory) {
            let kept: Vec<String> = options.iter().filter(|one| !one.ends_with('?')).cloned().collect();
            if !kept.is_empty() {
                options = kept;
            }
        }
        return Some(pick(&options, memory, body));
    }
    None
}

/// Python: bot/memory/ack.py:3529 `_a_price_still_to_come(body, topic_text)`
///
/// Whether an expense remark is about a price nobody has paid yet.
pub fn _a_price_still_to_come(body: &str, topic_text: &str) -> bool {
    let said = [body, topic_text].iter().filter(|p| !p.is_empty()).cloned().collect::<Vec<_>>().join(" ");
    if REMARK_STILL_TO_COME.is_match(&said) {
        return true;
    }
    !REMARK_ALREADY_PAID.is_match(&said)
}

/// Python: bot/memory/ack.py:3547 `_a_result_is_on_the_table(clean)`
///
/// Whether this line is a result the person is waiting on or has just heard (the composer's
/// reading).
pub fn _a_result_is_on_the_table(clean: &str) -> bool {
    crate::compose::fallback::reads_as_a_result_line(clean)
}

/// Python: bot/memory/ack.py:3565 `_ending_of(frame)`
///
/// The words a frame adds after the thing said back: the part that repeats between turns.
pub fn _ending_of(frame: &str) -> String {
    let parts = _HOLE.split(frame);
    if parts.len() > 1 { squash(&_NOT_WORDS.sub(" ", &text::lower(&parts[1]))) } else { String::new() }
}

/// Python: bot/memory/ack.py:3571 `_pick_ending(frames, memory, **fields)`
///
/// One of the endings, never the one the reply before it ended with, filled with `fields`.
pub fn _pick_ending(frames: &[String], memory: &Memory, fields: &[(&str, &str)]) -> String {
    let last = match turns(memory).last() {
        Some(turn) => squash(&_NOT_WORDS.sub(" ", &text::lower(&turn.bot))),
        None => String::new(),
    };
    let fresh: Vec<String> = frames
        .iter()
        .filter(|frame| {
            let ending = _ending_of(frame);
            !(!ending.is_empty() && last.contains(&ending))
        })
        .cloned()
        .collect();
    let pool = if fresh.is_empty() { frames.to_vec() } else { fresh };
    py_format(&pick(&pool, memory, ""), fields)
}

/// The engine's reading of a line, as `understand(line)` with no memory.
fn understanding(line: &str) -> Understood {
    crate::understanding::understand(line, None)
}

/// Python: bot/memory/ack.py:3626 `_after_a_feeling(clean, memory)`
///
/// Whether this line is the reason for a feeling they said on the turn before.
pub fn _after_a_feeling(clean: &str, memory: &Memory) -> bool {
    if AN_AGE_ALONE.match_(text::strip(clean)).is_some() {
        return false; // an age is not a reason for anything here
    }
    let all = turns(memory);
    if all.is_empty() {
        return false;
    }
    let mut index = all.len() - 1;
    let mut said = all[index].user.clone();
    if text::strip(&said).is_empty() {
        return false;
    }
    if NO_REASON.match_(text::strip(&said)).is_some() && A_CLAUSE_OF_ITS_OWN.is_match(clean) {
        // "im low" then "dont know why": the reason belongs to the feeling one turn back.
        if index == 0 {
            return false;
        }
        index -= 1;
        said = all[index].user.clone();
        if text::strip(&said).is_empty() {
            return false;
        }
    }
    let read = understanding(&said);
    if read.emotion.as_deref() == Some("confused") {
        return false; // losing the thread is not a mood the next line explains
    }
    if AN_AIM.is_match(&said) {
        // An aim, not a mood, unless a feeling is said now.
        let now = understanding(clean);
        if !(now.intent == "feeling" && now.polarity < 0) && !A_FEELING_ABOUT_IT.is_match(clean) {
            return false;
        }
    }
    if read.intent != "feeling" || read.polarity > 0 {
        // "im fine" is a feeling too, and nothing follows from it.
        if !_a_heavy_reply_just_before(memory, index as i64) {
            return false;
        }
    }
    let words = text::split_ref(clean);
    if words.is_empty() || words.len() > 12 {
        return false;
    }
    let first = text::strip_chars(&text::lower(words[0]), ",;:!").to_string();
    if ["i", "im", "i'm", "ive", "i've", "we", "we're", "were"].contains(&first.as_str()) {
        return true;
    }
    words.iter().any(|w| PERSON_WORDS.contains(text::strip_chars(&text::lower(w), ",;:!")))
}

/// Python: bot/memory/ack.py:3692 `_a_heavy_reply_just_before(memory, index)`
///
/// Whether the reply they got on turn `index` was about something heavy they are carrying.
pub fn _a_heavy_reply_just_before(memory: &Memory, index: i64) -> bool {
    match memory.get("heavy_reply") {
        Some(note) if note.is_dict() => note.get("turn").is_some_and(|t| *t == Value::Int(index)),
        _ => false,
    }
}

/// Python: bot/memory/ack.py:3730 `_bad_news_just_before(clean, memory)`
///
/// Whether the line carries on a hard story the last turn or two began.
pub fn _bad_news_just_before(clean: &str, memory: &Memory) -> bool {
    if THIRD_PERSON.match_(&_strip_lead_in(clean, false)).is_none() {
        return false;
    }
    last_turns(memory, 2).iter().any(|turn| {
        let said = text::lower(&turn.user);
        crate::compose::badnews::reads_as_bad_news(&said) || crate::compose::loss::reads_as_loss(&said, None, false)
    })
}

/// Python: bot/memory/ack.py:3759 `carries_the_story_on(text, memory)`
///
/// Whether this line is the next line of a hard story the last turn or two began.
pub fn carries_the_story_on(text_: &str, memory: &Memory) -> bool {
    if text_.contains('?') {
        return false;
    }
    let clean = squash(text_);
    if clean.is_empty() {
        return false;
    }
    _bad_news_just_before(&clean, memory)
}

/// Python: bot/memory/ack.py:3828 `the_old_one_aged(clean, memory)`
///
/// CONTRACT: typed `()`; the value is [`the_old_one_aged_text`].
pub fn the_old_one_aged(clean: &str, memory: &Memory) {
    let _ = the_old_one_aged_text(clean, memory);
}

/// The reply to a line giving the age of the thing they are replacing, or None.
pub fn the_old_one_aged_text(clean: &str, memory: &Memory) -> Option<String> {
    let said_age = THE_OLD_ONE.match_(clean).map(|m| text::lower(m.name_str("n")))?;
    if n_words(clean) > 9 {
        return None;
    }
    let age: i64 = if pyuni::is_digit(&said_age) {
        said_age.parse().ok()?
    } else {
        match data::get("bot.memory.facts.WORD_NUMBERS").get(&said_age) {
            Some(Value::Int(n)) => *n,
            _ => return None,
        }
    };
    if !(5..=40).contains(&age) {
        return None; // "two years old" is a bad run, not a good one
    }
    let last = turns(memory).last()?;
    if !REPLACING.is_match(&last.user) {
        return None;
    }
    if A_FEELING.is_match(clean) || DEATH_WORDS.is_match(clean) || BLEED.is_match(clean) {
        return None;
    }
    Some(py_format(&pick(&OLD_ONE_TAIL, memory, clean), &[("n", &age.to_string())]))
}

/// Python: bot/memory/ack.py:3866 `_the_length(clean, memory)`
///
/// CONTRACT: typed `()`; the value is [`the_length_text`].
pub fn _the_length(clean: &str, memory: &Memory) {
    let _ = the_length_text(clean, memory);
}

/// The reply to a line that says only how long the thing has been going, or None.
pub fn the_length_text(clean: &str, memory: &Memory) -> Option<String> {
    if !crate::memory::narrow::only_a_length(clean) {
        return None;
    }
    if !crate::memory::tried::after_an_answer(memory) {
        return None;
    }
    let low = text::lower(clean);
    if crate::compose::badnews::reads_like_a_symptom(&low) || crate::compose::badnews::reads_as_bad_news(&low) {
        return None;
    }
    if A_FEELING.is_match(clean) || DEATH_WORDS.is_match(clean) {
        return None;
    }
    let found = NARROW_A_LENGTH.search(clean).expect("AttributeError: 'NoneType' object has no attribute 'group'");
    let lead = text::lower(found.name_str("lead"));
    let amount = found.name("amount");
    let unit = text::lower(found.name_str("unit"));
    if lead == "since" {
        // A weekday is a name and keeps its capital.
        let when = if LENGTH_NAMED_DAYS.contains(&unit) { upper_first(&unit) } else { unit };
        return Some(py_format(&pick(&SINCE_TAIL, memory, clean), &[("when", &when)]));
    }
    let mut length = match amount.filter(|a| !a.is_empty()) {
        Some(amount) => format!("{amount} {unit}"),
        None => unit,
    };
    if LENGTH_KEEPS.contains(&lead) {
        length = format!("{lead} {length}");
    }
    Some(py_format(&pick(&LENGTH_TAIL, memory, clean), &[("length", &upper_first(&length))]))
}

/// Python: bot/memory/ack.py:3903 `_the_hours(clean, read, memory)`
///
/// CONTRACT: typed `()`; the value is [`the_hours_text`].
pub fn _the_hours(clean: &str, read: &str, memory: &mut Memory) {
    let _ = the_hours_text(clean, read, memory);
}

/// The reply to a line about the hours they work, or None.
pub fn the_hours_text(clean: &str, read: &str, memory: &mut Memory) -> Option<String> {
    if clean.contains('?') || WANTS_ANSWER.is_match(read) {
        return None;
    }
    if NOT_THE_HOURS.is_match(read) {
        return None;
    }
    if DEATH_WORDS.is_match(read) || A_FEELING.is_match(clean) || BLEED.is_match(read) {
        return None;
    }
    let low = text::lower(read);
    if crate::compose::badnews::reads_like_a_symptom(&low) || crate::compose::badnews::reads_as_bad_news(&low) {
        return None;
    }
    // A death in the last few turns is not a chat about the rota.
    let recent = last_turns(memory, 3).iter().map(|t| t.user.clone()).collect::<Vec<_>>().join(" ");
    if !recent.is_empty() && DEATH_WORDS.is_match(&recent) {
        return None;
    }
    if let Some(last) = turns(memory).last() {
        if last.module.as_deref() == Some("answers") && reads_as_an_ask(&last.user) {
            return None; // a detail added to the answer just given
        }
    }
    let (family, pool) = hours_family(read);
    let pool = pool?;
    // The kind of week is kept for the turn after it (bot/memory/hours.py).
    crate::memory::hours::note(memory, &Value::from(family));
    Some(pick(&pool, memory, clean))
}

/// Python: bot/memory/ack.py:3944 `_a_deadline(clean, read, memory)`
///
/// CONTRACT: typed `()`; the value is [`a_deadline_text`].
pub fn _a_deadline(clean: &str, read: &str, memory: &Memory) {
    let _ = a_deadline_text(clean, read, memory);
}

/// The reply to a line that only says when the thing we just answered has to happen.
pub fn a_deadline_text(clean: &str, read: &str, memory: &Memory) -> Option<String> {
    let _ = read;
    let when = WHEN_DETAIL.search(clean).map(|m| text::strip(m.name_str("when")).to_string())?;
    if n_words(clean) > 7 {
        return None;
    }
    let all = turns(memory);
    let last = all.last()?;
    let said = last.bot.clone();
    if last.module.as_deref() != Some("answers") || n_words(&said) < 18 {
        return None;
    }
    if !_tells_them_what_to_do(&said) {
        return None; // a list of facts has no first step
    }
    let recent = format!("{} {}", said, last_turns(memory, 3).iter().map(|t| t.user.clone()).collect::<Vec<_>>().join(" "));
    if DEATH_WORDS.is_match(&recent) || BLEED.is_match(&recent) || A_FEELING.is_match(clean) {
        return None;
    }
    let pool = match _shape_of(&when).as_str() {
        "today" => &*WHEN_TODAY_TAIL,
        "tomorrow" => &*WHEN_TOMORROW_TAIL,
        "days" => &*WHEN_DAYS_TAIL,
        _ => &*WHEN_ROOMY_TAIL,
    };
    Some(py_format(&pick(pool, memory, clean), &[("when", &when), ("When", &upper_first(&when))]))
}

// ---------------------------------------------------------------------------------------------
// Hours, clocks and pets.

data_re!(
    CLOCK_EVENT, CLOCK_FLAT, CLOCK_DETAIL, CLOCK_AT, THE_NOUN_IS_AT, A_MARKED_HOUR, ON_NIGHTS, SHIFT_LATE,
    SHIFT_EARLY, EARLY_RISE, LATE_FINISH, LONG_SHIFT, SHIFTS_ALL_OVER, WORK_FROM_HOME, PET_ADDED, PET_ALL_RIGHT,
    INTRODUCED_BEFORE, A_NAME_AFTER, A_VAGUE_MANY, A_WANTED_ANIMAL, A_WANT_TURNED_DOWN, A_HAD_AN_ANIMAL,
);
data_strs!(
    CLOCK_FLAT_TAIL, CLOCK_AFTER_TAIL, CLOCK_MORNING_TAIL, NIGHT_TAIL, LATE_TAIL, EARLY_TAIL, LONG_TAIL,
    SPREAD_TAIL, HOME_TAIL, A_PET_WANTED, A_WANTED_ANIMAL_TAIL,
);

static WHEN_EVENT_WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.when.EVENT_WORD"));
inline_re!(RE_HOUR_HEAD, r"\s*(\d{1,2})(?!\d)", 0);

/// Python: bot/memory/ack.py:3977 `_an_hour_on_the_day(text)`
///
/// Which pool an hour on the day belongs to: "nerves" for an appointment, "flat" for a shift.
pub fn _an_hour_on_the_day(text_: &str) -> Option<String> {
    if text_.is_empty() {
        return None;
    }
    if CLOCK_EVENT.is_match(text_) {
        return Some("nerves".into());
    }
    if CLOCK_FLAT.is_match(text_) {
        return Some("flat".into());
    }
    if WHEN_EVENT_WORD.is_match(text_) { Some("nerves".into()) } else { None }
}

/// Python: bot/memory/ack.py:4006 `_the_figure_is_an_hour(when)`
///
/// True when the figure is an hour at all: "40" is a quantity, and a clock stops at 24.
pub fn _the_figure_is_an_hour(when: &str) -> bool {
    match RE_HOUR_HEAD.match_(when) {
        None => true, // a spelled hour is a time
        Some(head) => {
            let hour: i64 = head.str(1).parse().unwrap_or(0);
            (1..=24).contains(&hour)
        }
    }
}

/// Python: bot/memory/ack.py:4014 `_a_clock_time(clean, read, memory)`
///
/// CONTRACT: typed `()`; the value is [`a_clock_time_text`].
pub fn _a_clock_time(clean: &str, read: &str, memory: &Memory) {
    let _ = a_clock_time_text(clean, read, memory);
}

/// The reply to a line that only gives the hour the thing we just answered is at, or None.
pub fn a_clock_time_text(clean: &str, read: &str, memory: &Memory) -> Option<String> {
    let _ = read;
    if n_words(clean) > 8 {
        return None;
    }
    let found = CLOCK_DETAIL.search(clean).or_else(|| CLOCK_AT.search(clean))?;
    let when = text::strip(found.name_str("when")).to_string();
    let last = turns(memory).last()?;
    let said = last.bot.clone();
    let module = last.module.as_deref();
    let theirs = last_turns(memory, 3).iter().map(|t| t.user.clone()).collect::<Vec<_>>().join(" ");
    if module == Some("greeting") && n_words(&said) >= 5 {
        // The composer's own acknowledgment counts when either side names a diary thing.
        if _an_hour_on_the_day(&said).is_none() && _an_hour_on_the_day(&theirs).is_none() {
            return None;
        }
    } else if !matches!(module, Some("answers") | Some("chat") | Some("memory_store")) || n_words(&said) < 5 {
        return None;
    }
    // What they said, not what the answer said.
    let kind = _an_hour_on_the_day(&theirs)?;
    if DEATH_WORDS.is_match(&theirs) || BLEED.is_match(clean) || A_FEELING.is_match(clean) {
        return None;
    }
    if !_the_figure_is_an_hour(&when) {
        return None;
    }
    if let Some(named_) = THE_NOUN_IS_AT.search(clean) {
        if !A_MARKED_HOUR.is_match(&when) && _an_hour_on_the_day(named_.name_str("noun")).is_none() {
            return None; // "the total is at 40" is a figure on a total
        }
    }
    let at = format!("At {when}");
    let pool = if kind == "flat" {
        &*CLOCK_FLAT_TAIL
    } else if _clock_shape(&when) == "after" {
        &*CLOCK_AFTER_TAIL
    } else {
        &*CLOCK_MORNING_TAIL
    };
    Some(py_format(&pick(pool, memory, clean), &[("At", &upper_first(&at)), ("When", &upper_first(&when)), ("when", &when)]))
}

/// Python: bot/memory/ack.py:4075 `_hours_family(read)`
///
/// Which kind of week the line names, and the pool that answers it, as a (family, pool) tuple
/// (None, None when it names none).
pub fn _hours_family(read: &str) -> Value {
    let (family, pool) = hours_family(read);
    Value::Tuple(std::sync::Arc::new(vec![Value::from(family), pool.map(Value::from).unwrap_or(Value::None)]))
}

/// `_hours_family(read)` with Rust types.
pub fn hours_family(read: &str) -> (Option<String>, Option<Vec<String>>) {
    let found = |family: &str, pool: &Vec<String>| (Some(family.to_string()), Some(pool.clone()));
    if ON_NIGHTS.is_match(read) {
        return found("nights", &NIGHT_TAIL);
    }
    if SHIFT_LATE.is_match(read) {
        return found("late", &LATE_TAIL);
    }
    if SHIFT_EARLY.is_match(read) {
        return found("early", &EARLY_TAIL);
    }
    if EARLY_RISE.is_match(read) {
        return found("early", &EARLY_TAIL);
    }
    if LATE_FINISH.is_match(read) {
        return found("late", &LATE_TAIL);
    }
    if LONG_SHIFT.is_match(read) {
        return found("long", &LONG_TAIL);
    }
    if SHIFTS_ALL_OVER.is_match(read) {
        return found("spread", &SPREAD_TAIL);
    }
    if WORK_FROM_HOME.is_match(read) {
        return found("home", &HOME_TAIL);
    }
    (None, None)
}

/// The word the person used for a pet record: its breed, or its animal.
fn pet_word(pet: &Value) -> String {
    let breed = field(pet, "breed");
    let word = if breed.is_empty() { field(pet, "animal") } else { breed };
    text::lower(text::strip(&word))
}

/// Python: bot/memory/ack.py:4143 `_a_pet_that_is_all_right(clean, kept)`
///
/// Whether the line says the animal is fine, rather than how it is ill.
pub fn _a_pet_that_is_all_right(clean: &str, kept: &Value) -> bool {
    for pet in kept.items() {
        if !pet.is_dict() {
            continue;
        }
        let word = pet_word(pet);
        if word.is_empty() {
            continue;
        }
        if let Some(found) = re_cached(&format!(r"\b{}s?\b", pyre::escape(&word)), pyre::I).search(clean) {
            if PET_ALL_RIGHT.is_match(&clean[found.end()..]) {
                return true;
            }
        }
    }
    false
}

/// Python: bot/memory/ack.py:4174 `_nothing_added_but_the_pet(clean, kept)`
///
/// Whether the words after the animal add nothing but its name, count, age or breed.
pub fn _nothing_added_but_the_pet(clean: &str, kept: &Value) -> bool {
    for pet in kept.items() {
        if !pet.is_dict() {
            continue;
        }
        let word = pet_word(pet);
        if word.is_empty() {
            continue;
        }
        let Some(found) = re_cached(&format!(r"\b{}s?\b", pyre::escape(&word)), pyre::I).search(clean) else { continue };
        let after = &clean[found.end()..];
        if PET_ALL_RIGHT.is_match(after) {
            continue; // "my dog is fine now": a fact to say back
        }
        if INTRODUCED_BEFORE.is_match(&clean[..found.start()]) && A_NAME_AFTER.match_(after).is_some() {
            continue; // "we named the cat Biscuit"
        }
        if PET_ADDED.match_(after).is_none() {
            return false;
        }
    }
    true
}

/// Python: bot/memory/ack.py:4255 `_a_wanted_animal(clean)`
///
/// True when the line asks for an animal rather than turning one down or remembering one.
pub fn _a_wanted_animal(clean: &str) -> bool {
    if A_WANT_TURNED_DOWN.is_match(clean) {
        return false;
    }
    A_WANTED_ANIMAL.is_match(clean)
}

// ---------------------------------------------------------------------------------------------
// Animals wanted, coming, taken back and kept.

data_re!(PET_WORD, HABIT, GOING, TROUBLE, FEELING, GOING_BAD);

static FACTS_PETS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.facts.PETS"));
static ANIMAL_KINDS: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.memory.facts.ANIMAL_KINDS"));
static NAMES_THE_THING: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.topics.NAMES_THE_THING"));
static A_THING_ON_ITS_WAY: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.topics.A_THING_ON_ITS_WAY"));

/// `memory.get("user").get("pets")`: the store's own animals.
fn user_pets(memory: &Memory) -> Vec<Value> {
    match memory.get("user") {
        Some(user) if user.is_dict() => user.get("pets").map(|p| p.items().to_vec()).unwrap_or_default(),
        _ => Vec::new(),
    }
}

/// `found.get("pets") or []`, the dicts only.
fn found_pets(found: &Value) -> Vec<Value> {
    found.get("pets").map(|p| p.items().iter().filter(|x| x.is_dict()).cloned().collect()).unwrap_or_default()
}

/// Python: bot/memory/ack.py:4263 `_an_animal_wanted(clean)`
///
/// The animal a line wants, or "".
pub fn _an_animal_wanted(clean: &str) -> String {
    if !_a_wanted_animal(clean) {
        return String::new();
    }
    if let Some(found) = FACTS_PETS.search(clean) {
        return text::lower(found.str(1));
    }
    // "the kids want a pet" names no animal at all.
    if PET_WORD.is_match(clean) {
        return "pet".into();
    }
    String::new()
}

/// Python: bot/memory/ack.py:4281 `_an_animal_they_had_once(clean)`
///
/// True when the line is an animal the person used to have, not one they are asking for.
pub fn _an_animal_they_had_once(clean: &str) -> bool {
    if _a_wanted_animal(clean) {
        return false;
    }
    A_HAD_AN_ANIMAL.is_match(clean)
}

/// Python: bot/memory/ack.py:4292 `_what_a_wanted_animal_needs(animal, clean, memory)`
///
/// What having one of these takes, from the knowledge base, or the want's next question. (The
/// entry's reply gets the real memory, as in Python, so this takes it mutably.)
pub fn _what_a_wanted_animal_needs(animal: &str, clean: &str, memory: &mut Memory) -> String {
    let looked = py_try(|| {
        let reading = understanding(&format!("what do i need for a {animal}"));
        let found = crate::knowledge::lookup::find(&reading, None, None, false);
        (reading, found)
    });
    if let Some((reading, Some(entry))) = looked {
        let said = match &entry.reply {
            crate::knowledge::entry::Reply::Text(s) => Some(s.clone()),
            crate::knowledge::entry::Reply::Fn(f) => crate::knowledge::registry::try_reply(f, memory, &reading).ok().flatten(),
            crate::knowledge::entry::Reply::Choices(_) => None,
        };
        if let Some(said) = said {
            if !text::strip(&said).is_empty() {
                return text::strip(&said).to_string();
            }
        }
    }
    if animal == "pet" {
        return pick(&A_PET_WANTED, memory, clean);
    }
    py_format(&pick(&A_WANTED_ANIMAL_TAIL, memory, clean), &[("animal", animal), ("Animal", &upper_first(animal))])
}

/// Python: bot/memory/ack.py:4347 `_the_pets_are_coming(pets, memory)`
///
/// Whether the animals this line names are ones the store already has on their way.
pub fn _the_pets_are_coming(pets: &Value, memory: &Memory) -> bool {
    let kind_of = |pet: &Value| ANIMAL_KINDS.get(&text::lower(text::strip(&field(pet, "animal")))).cloned().unwrap_or_default();
    let coming: HashSet<String> = user_pets(memory)
        .iter()
        .filter(|pet| pet.is_dict() && pet.get("coming").is_some_and(Value::truthy))
        .map(kind_of)
        .filter(|k| !k.is_empty())
        .collect();
    if coming.is_empty() {
        return false;
    }
    for pet in pets.items() {
        if !pet.is_dict() {
            continue;
        }
        let count = pet.get("count").and_then(Value::as_f64).unwrap_or(0.0);
        if pet.get("name").is_some_and(Value::truthy) || count > 1.0 || !pet.get("breed").is_some_and(Value::truthy) {
            continue; // a named animal, or more than one, is not the one on its way
        }
        if coming.contains(&kind_of(pet)) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/ack.py:4381 `_the_animal_coming(found)`
///
/// The animal word of an animal the line says is on its way, or "".
pub fn _the_animal_coming(found: &Value) -> String {
    for pet in found_pets(found) {
        if pet.get("coming").is_some_and(Value::truthy) {
            let animal = text::lower(text::strip(&field(&pet, "animal")));
            if !animal.is_empty() {
                return animal;
            }
        }
    }
    String::new()
}

/// Python: bot/memory/ack.py:4392 `_the_taken_back_animal(found)`
///
/// The animal word the take-back names, or "".
pub fn _the_taken_back_animal(found: &Value) -> String {
    for pet in found_pets(found) {
        if pet.get("not_theirs").is_some_and(Value::truthy) {
            let animal = text::lower(text::strip(&field(&pet, "animal")));
            if !animal.is_empty() {
                return animal;
            }
        }
    }
    String::new()
}

/// Python: bot/memory/ack.py:4402 `_an_animal_turned_down(found, memory)`
///
/// True when the line is a "no" on an animal they do not have.
pub fn _an_animal_turned_down(found: &Value, memory: &Memory) -> bool {
    if !found.is_dict() {
        return false;
    }
    let pets = found_pets(found);
    if pets.is_empty() || !pets.iter().all(|pet| pet.get("not_theirs").is_some_and(Value::truthy)) {
        return false;
    }
    if keys_of(found).iter().any(|k| k != "pets" && k != "tastes") {
        return false;
    }
    let animal = _the_taken_back_animal(found);
    if animal.is_empty() || _on_record(&animal, memory) {
        return false;
    }
    let mut named: HashSet<String> = HashSet::new();
    for pet in &pets {
        for key in ["animal", "breed"] {
            named.insert(text::lower(text::strip(&field(pet, key))));
        }
    }
    named.remove("");
    let tastes: Vec<Value> = found.get("tastes").map(|t| t.items().iter().filter(|x| x.is_dict()).cloned().collect()).unwrap_or_default();
    !tastes.is_empty() && tastes.iter().all(|taste| named.contains(&text::lower(text::strip(&field(taste, "thing")))))
}

/// Python: bot/memory/ack.py:4431 `_on_record(animal, memory)`
///
/// True when the store holds an animal of that kind as one of theirs.
pub fn _on_record(animal: &str, memory: &Memory) -> bool {
    user_pets(memory).iter().any(|pet| pet.is_dict() && text::lower(text::strip(&field(pet, "animal"))) == animal)
}

/// Python: bot/memory/ack.py:4460 `_an_animal_only_planned(found, clean, memory)`
///
/// The animal word a line renames inside a plan to get one, or "".
pub fn _an_animal_only_planned(found: &Value, clean: &str, memory: &Memory) -> String {
    let pets = found_pets(found);
    if pets.len() != 1 {
        return String::new();
    }
    let pet = &pets[0];
    if ["wanted", "coming", "not_theirs"].iter().any(|k| pet.get(k).is_some_and(Value::truthy)) {
        return String::new();
    }
    if NAMES_THE_THING.match_(&flat(clean)).is_none() {
        return String::new();
    }
    let plan = match memory.get("topic") {
        Some(record) if record.is_dict() => field(record, "text"),
        _ => String::new(),
    };
    if plan.is_empty() || text::lower(text::strip(&plan)) == text::lower(text::strip(clean)) {
        return String::new();
    }
    if !A_THING_ON_ITS_WAY.is_match(&plan) {
        return String::new();
    }
    text::lower(text::strip(&pet_word_raw(pet)))
}

/// `pet.get("breed") or pet.get("animal") or ""`, as written.
fn pet_word_raw(pet: &Value) -> String {
    let breed = field(pet, "breed");
    if breed.is_empty() { field(pet, "animal") } else { breed }
}

/// Python: bot/memory/ack.py:4485 `_the_wished_animal(found, clean='')`
///
/// The animal the line wishes for, as they named it - the breed if they gave one.
pub fn _the_wished_animal(found: &Value, clean: &str) -> String {
    for pet in found_pets(found) {
        if pet.get("wanted").is_some_and(Value::truthy) {
            let said = text::lower(text::strip(&pet_word_raw(&pet)));
            if !said.is_empty() {
                return said;
            }
        }
    }
    let words: Vec<&str> = text::split_ref(clean).into_iter().filter(|w| pyuni::is_alpha(w)).collect();
    words.last().map(|w| w.to_string()).unwrap_or_else(|| "one".into())
}

/// Python: bot/memory/ack.py:4496 `_only_a_wish(found)`
///
/// True when everything the line gave the store is an animal the person would like.
pub fn _only_a_wish(found: &Value) -> bool {
    if !found.is_dict() || keys_of(found).iter().any(|k| k != "pets") {
        return false;
    }
    let pets = found_pets(found);
    !pets.is_empty() && pets.iter().all(|pet| pet.get("wanted").is_some_and(Value::truthy))
}

/// Python: bot/memory/ack.py:4504 `_an_animal_taken_back(found)`
///
/// True when the only thing the line gave the store is an animal being taken back.
pub fn _an_animal_taken_back(found: &Value) -> bool {
    if !found.is_dict() || keys_of(found).iter().any(|k| k != "pets") {
        return false;
    }
    let pets = found_pets(found);
    if pets.iter().any(|pet| pet.get("wanted").is_some_and(Value::truthy)) {
        return false;
    }
    !pets.is_empty() && pets.iter().all(|pet| pet.get("not_theirs").is_some_and(Value::truthy))
}

/// `int(pet.get("count") or 1)`, 1 when it cannot be read.
fn pet_count(pet: &Value) -> i64 {
    match pet.get("count") {
        Some(v) if v.truthy() => match v {
            Value::Int(n) => *n,
            Value::Bool(b) => *b as i64,
            Value::Float(f) => f.trunc() as i64,
            Value::Str(s) => text::strip(s).parse().unwrap_or(1),
            _ => 1,
        },
        _ => 1,
    }
}

/// Python: bot/memory/ack.py:4520 `_the_pets_said(kept, clean='')`
///
/// The pets the store kept, as a person would say them, and whether there is more than one.
pub fn _the_pets_said(kept: &Value, clean: &str) -> (String, bool) {
    let number = |n: i64| -> String {
        match n {
            1 => "one",
            2 => "two",
            3 => "three",
            4 => "four",
            5 => "five",
            6 => "six",
            7 => "seven",
            8 => "eight",
            9 => "nine",
            10 => "ten",
            _ => "",
        }
        .to_string()
    };
    let mut groups: Vec<(String, i64)> = Vec::new();
    for pet in kept.items() {
        if !pet.is_dict() {
            continue;
        }
        let count = pet_count(pet);
        // A breed named beside a group is the breed of one of them.
        let breed_word = if pet.get("breed_of_one").is_some_and(Value::truthy) { String::new() } else { field(pet, "breed") };
        let animal = text::lower(text::strip(&if breed_word.is_empty() { field(pet, "animal") } else { breed_word }));
        if animal.is_empty() {
            continue;
        }
        match groups.iter_mut().find(|g| g.0 == animal) {
            Some(group) => group.1 += count,
            None => groups.push((animal, count)),
        }
    }
    if groups.is_empty() {
        return (String::new(), false);
    }
    if groups.len() == 1 && groups[0].1 == 1 && A_VAGUE_MANY.is_match(clean) {
        return (String::new(), false);
    }
    let parts: Vec<String> = groups
        .iter()
        .map(|(animal, count)| {
            if *count == 1 {
                let article = if animal.chars().next().is_some_and(|c| "aeiou".contains(c)) { "an " } else { "a " };
                format!("{article}{animal}")
            } else {
                let word = if animal.ends_with('s') { animal.clone() } else { format!("{animal}s") };
                let said = number(*count);
                format!("{} {}", if said.is_empty() { count.to_string() } else { said }, word)
            }
        })
        .collect();
    let mut said = if parts.len() > 1 {
        format!("{} and {}", parts[..parts.len() - 1].join(", "), parts[parts.len() - 1])
    } else {
        parts[0].clone()
    };
    let names: Vec<String> = kept
        .items()
        .iter()
        .filter(|p| p.is_dict())
        .map(|p| text::strip(&field(p, "name")).to_string())
        .filter(|n| !n.is_empty())
        .map(|n| upper_first(&n))
        .collect();
    if names.len() == 1 {
        said.push_str(&format!(" called {}", names[0]));
    } else if !names.is_empty() {
        said.push_str(&format!(", {} and {}", names[..names.len() - 1].join(", "), names[names.len() - 1]));
    }
    let many = groups.iter().map(|g| g.1).sum::<i64>() > 1;
    (said, many)
}

/// Python: bot/memory/ack.py:4576 `_plain(clean, memory)`
///
/// An echo of a line with no fact in it, with a tail that fits what kind of line it is.
pub fn _plain(clean: &str, memory: &Memory) -> Option<String> {
    let body = _strip_lead_in(clean, false);
    if body.is_empty() || trouble_reading(clean).is_some() {
        return None;
    }
    if !_about_them(&body) {
        return None;
    }
    if !HABIT.is_match(&body) && !GOING.is_match(&body) {
        return None;
    }
    if OBLIGED.is_match(&body) {
        return None;
    }
    if TROUBLE.is_match(&body) || FEELING.is_match(&body) {
        return None; // a feeling or a row is not a routine to admire
    }
    let thing = _echo(clean, Some(memory));
    if thing.is_empty() {
        return None;
    }
    let options = if HABIT.is_match(&body) {
        &*HABIT_TAIL
    } else if GOING.is_match(&body) && !GOING_BAD.is_match(&body) {
        &*GOING_TAIL
    } else {
        return None;
    };
    Some(py_format(&pick(options, memory, ""), &[("thing", &thing), ("Thing", &upper_first(&thing))]))
}

/// Python: bot/memory/ack.py:4607 `_about_them(body)`
///
/// Whether the line is about their own life rather than the world or someone else's.
pub fn _about_them(body: &str) -> bool {
    let first = text::split_ref(body).first().map(|w| text::strip_chars(&text::lower(w), ",;:").to_string()).unwrap_or_default();
    ["i", "i'm", "im", "i've", "ive", "i'd", "we", "we're", "were", "my", "our"].contains(&first.as_str())
}

// ---------------------------------------------------------------------------------------------
// Somebody getting better, nobody around, and plans in the diary.

data_re!(
    NOBODY_AROUND, NOBODY_AROUND_ASK, ANYBODY_AT_ALL, GOING_HOME, ON_THE_MEND, BACK_AT_IT, A_MOVE,
    _THE_BOT_AS_SUBJECT, _SOMEONE_ELSE_IN_IT, _WORDS, PLACED_IN_THE_PAST,
);
data_set!(OTHER_PERSON, A_RELATION, LEADERS);
data_strs!(
    GOING_HOME_TAIL, ON_THE_MEND_TAIL, BACK_AT_IT_TAIL, GOING_HOME_MINE, ON_THE_MEND_MINE, BACK_AT_IT_MINE,
    ALONE_AFTER_A_MOVE, ALONE_TAIL, ALONE_AFTER_A_LOSS,
);

inline_re!(
    RE_SOMETHING_TO_GET_BETTER_FROM,
    r"\b(?:ill|sick|hospital|operation|op|surgery|broke|broken|stroke|chemo|treatment|injury|injured|fell|fall|accident|recovering|recovery|bug|flu|pneumonia|infection)\b",
    0
);
inline_re!(
    RE_THE_TIME,
    r"\b(next week|this week|tomorrow|soon|on (?:monday|tuesday|wednesday|thursday|friday|saturday|sunday)|in a (?:few|couple of) days|by the weekend)\b",
    0
);
inline_re!(RE_MY_OR_OUR_WORD, r"\b(?:my|our)\s+([a-z]+)", 0);
inline_re!(RE_POSSESSIVE_END, r"'s$", 0);
inline_re!(RE_COMING_HOME, r"\b(?:coming|heading|getting|moving|flying) (?:back )?home\b", 0);
inline_re!(RE_MEETING, r"\b(?:meet|meeting|seeing)\b", 0);

/// `compose.negated(clean, phrase)`.
fn negated(clean: &str, phrase: &str) -> bool {
    crate::compose::negated(clean, phrase, 3)
}

/// Python: bot/memory/ack.py:4699 `_who_is_getting_better(low)`
///
/// "them" for somebody else in the line, "me" for the speaker, "" for nobody.
pub fn _who_is_getting_better(low: &str) -> String {
    let words: HashSet<String> = RE_AZ_WORDS.findall_str(low).into_iter().collect();
    if words.iter().any(|w| OTHER_PERSON.contains(w) || A_RELATION.contains(w)) {
        return "them".into();
    }
    if words.iter().any(|w| ["i", "im", "ive", "me", "my", "myself"].contains(&w.as_str())) {
        return "me".into();
    }
    String::new()
}

/// Python: bot/memory/ack.py:4709 `a_person_getting_better(clean, memory=None)`
///
/// CONTRACT: typed `()`; the value is [`a_person_getting_better_pool`].
pub fn a_person_getting_better(clean: &str, memory: &Memory) {
    let _ = a_person_getting_better_pool(clean, memory);
}

/// The pool for news that somebody is getting well, or None.
pub fn a_person_getting_better_pool(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let plain = clean.replace(['\u{2019}', '\u{2018}'], "'");
    let low = text::lower(&plain);
    if text::strip(&low).is_empty() || DEATH_WORDS.is_match(&low) {
        return None;
    }
    let who = _who_is_getting_better(&low);
    if who.is_empty() {
        return None;
    }
    let mut said = vec![low.clone()];
    said.extend(_the_words_of_the_last_two_turns(memory));
    if !RE_SOMETHING_TO_GET_BETTER_FROM.is_match(&said.join(" ")) {
        return None; // nothing in the chat says there was anything to get better from
    }
    let mine = who == "me";
    for (pattern, theirs, ours) in [
        (&*GOING_HOME, &*GOING_HOME_TAIL, &*GOING_HOME_MINE),
        (&*ON_THE_MEND, &*ON_THE_MEND_TAIL, &*ON_THE_MEND_MINE),
        (&*BACK_AT_IT, &*BACK_AT_IT_TAIL, &*BACK_AT_IT_MINE),
    ] {
        if let Some(found) = pattern.search(&low) {
            if !negated(&low, found.as_str()) {
                return Some(_with_the_time(if mine { ours } else { theirs }, &low));
            }
        }
    }
    None
}

/// Python: bot/memory/ack.py:4734 `_the_words_of_the_last_two_turns(memory)`
///
/// The person's own lines from the last two turns, lowercased, oldest first.
pub fn _the_words_of_the_last_two_turns(memory: &Memory) -> Vec<String> {
    let words: Vec<String> = turns(memory).iter().map(|t| text::lower(&t.user)).collect();
    words[words.len().saturating_sub(2)..].to_vec()
}

/// Python: bot/memory/ack.py:4745 `_with_the_time(pool, low)`
///
/// The pool with the time the line gave in it ("out next week" -> "out next week").
pub fn _with_the_time(pool: &[String], low: &str) -> Vec<String> {
    let when = RE_THE_TIME.search(low).map(|m| m.str(1).to_string()).unwrap_or_else(|| "soon".into());
    pool.iter().map(|line| line.replace("{soon}", &when)).collect()
}

/// Python: bot/memory/ack.py:4789 `nobody_around(clean, memory)`
///
/// CONTRACT: typed `()`; the value is [`nobody_around_pool`].
pub fn nobody_around(clean: &str, memory: &Memory) {
    let _ = nobody_around_pool(clean, memory);
}

/// The pool for a line that says, as a fact, that they have nobody, or None.
pub fn nobody_around_pool(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let plain = clean.replace(['\u{2019}', '\u{2018}'], "'");
    let hit = NOBODY_AROUND.search(&plain)?;
    if NOBODY_AROUND_ASK.is_match(&plain) || ANYBODY_AT_ALL.is_match(&plain) {
        return None;
    }
    // A line about somebody else's empty week is not this line.
    if text::split_ref(&plain[..hit.start()]).iter().any(|w| OTHER_PERSON.contains(*w)) {
        return None;
    }
    let head: String = hit.as_str().chars().take(12).collect();
    if negated(&plain, &head) {
        return None; // a doubled denial is not this line
    }
    // `str(one or "")` of a turn dict is its repr: the words in it are read as they stand.
    let mut said = vec![plain.clone()];
    said.extend(last_turns(memory, 2).iter().map(turn_repr));
    let said = said.join(" ");
    if DEATH_WORDS.is_match(&said) {
        return Some(ALONE_AFTER_A_LOSS.clone());
    }
    Some(if A_MOVE.is_match(&said) { ALONE_AFTER_A_MOVE.clone() } else { ALONE_TAIL.clone() })
}

/// `str(turn)` for a turn dict, as Python writes it.
fn turn_repr(turn: &Turn) -> String {
    let module = match &turn.module {
        Some(m) => crate::memory::narrow::py_repr(&Value::from(m.as_str())),
        None => "None".into(),
    };
    format!(
        "{{'user': {}, 'bot': {}, 'module': {}}}",
        crate::memory::narrow::py_repr(&Value::from(turn.user.as_str())),
        crate::memory::narrow::py_repr(&Value::from(turn.bot.as_str())),
        module
    )
}

/// Python: bot/memory/ack.py:4827 `_said_to_the_bot(clean)`
///
/// Whether the line opens on the bot and has nothing of the person's life in it.
pub fn _said_to_the_bot(clean: &str) -> bool {
    if _THE_BOT_AS_SUBJECT.match_(clean).is_none() {
        return false;
    }
    !_SOMEONE_ELSE_IN_IT.is_match(clean)
}

/// Python: bot/memory/ack.py:4834 `_no_subject(clean, memory=None)`
///
/// Whether the line opens with a helper and nothing that can be a subject: "was out all day".
pub fn _no_subject(clean: &str, memory: &Memory) -> bool {
    if memory.understood.as_ref().is_some_and(|u| u.dropped_subject) {
        return true;
    }
    let mut words: Vec<String> = _WORDS.findall_str(&text::lower(&_repaired(clean, memory))).into_iter().filter(|w| !w.is_empty()).collect();
    while words.first().is_some_and(|w| LEADERS.contains(w)) {
        words.remove(0);
    }
    !words.is_empty() && crate::understanding::normalize::opens_without_subject(&words, 0)
}

/// Python: bot/memory/ack.py:4852 `_asked_last(memory)`
///
/// Whether the bot's last reply ended on a question, so this one must not ask another.
pub fn _asked_last(memory: &Memory) -> bool {
    turns(memory).last().is_some_and(|last| text::strip(&last.bot).ends_with('?'))
}

/// Python: bot/memory/ack.py:4861 `_a_diary_plan(found, clean)`
///
/// The endings an appointment or a visit wants, or None when the plan is something else.
pub fn _a_diary_plan(found: &Value, clean: &str) -> Option<Vec<String>> {
    if !found.is_dict() || !found.get("plan").is_some_and(Value::truthy) {
        return None;
    }
    let said = flat(clean);
    if said.is_empty() || !DATE_AHEAD.is_match(&said) {
        return None;
    }
    if RE_AZ_WORDS.findall_str(&said).iter().any(|w| PAST_PLAN.contains(w)) {
        return None;
    }
    let mut who: Option<&str> = None;
    if let Some(named_) = RE_MY_OR_OUR_WORD.search(&said) {
        let word = named_.str(1).to_string();
        let candidates = [word.clone(), RE_POSSESSIVE_END.sub("", &word), text::slice(&word, 0, Some(-1))];
        let person = candidates.into_iter().find(|one| RELATION_SET.contains(one)).unwrap_or_default();
        // Coming home is not a visit.
        let at_home = RE_COMING_HOME.is_match(&said);
        if RELATION_SET.contains(&person) && COMING_TO_STAY.is_match(&said) && !at_home {
            who = Some(if _more_than_one_coming(&said, named_.end(), &word) {
                "they"
            } else if SHE.contains(&person) && !HE.contains(&person) {
                "she"
            } else if HE.contains(&person) && !SHE.contains(&person) {
                "he"
            } else {
                "they"
            });
        }
    }
    if who == Some("they") {
        return Some(VISIT_TAIL_MANY.clone());
    }
    if let Some(who) = who {
        return Some(VISIT_TAIL_ONE.iter().map(|one| one.replace("{who}", who)).collect());
    }
    if DIARY_THING.is_match(&said) {
        return Some(if DIARY_TIME.is_match(&said) { DIARY_TIME_TAIL.clone() } else { DIARY_TAIL.clone() });
    }
    if RE_MEETING.is_match(&said) {
        return Some(if DIARY_TIME.is_match(&said) { MEETING_TAIL.clone() } else { MEETING_TIME_TAIL.clone() });
    }
    if RE_AZ_WORDS.findall_str(&said).iter().any(|w| OCCASIONS.contains(w)) {
        return Some(OCCASION_TAIL.clone());
    }
    None
}

/// Python: bot/memory/ack.py:4907 `_plan_tails(found, clean, memory)`
///
/// The endings a trip or an occasion wants, or None when the line is another kind of plan.
pub fn _plan_tails(found: &Value, clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let mut options = match reads_as_a_trip(clean).as_deref() {
        Some("trip") => TRIP_TAIL.clone(),
        Some("occasion") => OCCASION_TAIL.clone(),
        Some("occasion_flat") => OCCASION_FLAT_TAIL.clone(),
        _ => _a_diary_plan(found, clean).filter(|o| !o.is_empty())?,
    };
    if HOW_LONG_ALREADY.is_match(clean) {
        let kept: Vec<String> = options.iter().filter(|one| !text::lower(one).contains("how long")).cloned().collect();
        if !kept.is_empty() {
            options = kept;
        }
    }
    if _asked_last(memory) {
        // Two questions in a row is an interrogation.
        let kept: Vec<String> = options.iter().filter(|one| !one.ends_with('?')).cloned().collect();
        if !kept.is_empty() {
            options = kept;
        }
    }
    Some(options)
}

/// Python: bot/memory/ack.py:4945 `_about_a_body(read, memory=None)`
///
/// Whether the line is the person's own body or health, however it words it.
pub fn _about_a_body(read: &str, memory: &Memory) -> bool {
    py_try(|| crate::understanding::understand(read, Some(memory)).topic.as_deref() == Some("health")).unwrap_or(false)
}

// ---------------------------------------------------------------------------------------------
// Jobs, pointers and conditions wondered about.

data_re!(JOB_TO_DO, JOB_HAS_A_DAY, JOB_BEFORE, HAVE_A, POINTS_BACK, A_COUNT_OF_THEM, A_DATE_STILL_TO_COME, A_WONDERED_CONDITION, A_WONDER_SAID_NO, A_CONDITION_KIND);
data_strs!(JOB_TAIL, PAST_JOB, PAST_JOB_PLAIN, WHO_THE_FIGURE_IS, WHO_IS_IT, WHAT_OF_THEM, WHICH_ONES, HEAVY_FRAMES, CHILD);
data_set!(WITH_AN_ARTICLE, CONDITION_INITIALS);

static JOB_WAS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.facts.JOB_WAS"));
static WAS_NOT_A_JOB: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.WAS_NOT_A_JOB"));
static PERSON_GROUP_KEYS: LazyLock<HashSet<String>> =
    LazyLock::new(|| data::get("bot.memory.ack.PERSON_GROUP").pairs().iter().map(|(k, _)| py_str(k)).collect());
/// `CONDITION_FAMILIES`: (the family's pattern or None for the rest, its frames).
static CONDITION_FAMILIES: LazyLock<Vec<(Option<&'static Re>, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.memory.ack.CONDITION_FAMILIES")
        .items()
        .iter()
        .map(|row| (row.items()[0].as_re(), row.items()[1].strs()))
        .collect()
});

inline_re!(RE_SHE_WORDS, r"\b(?:she|shes|she's|her)\b", pyre::I);
inline_re!(RE_HE_WORDS, r"\b(?:he|hes|he's|his)\b", pyre::I);
inline_re!(RE_THEY_WORDS, r"\b(?:they|theyre|they're|them|their|theirs|those|these)\b", pyre::I);

/// Python: bot/memory/ack.py:5012 `the_job_they_had(clean, memory)`
///
/// CONTRACT: typed `()`; the value is [`the_job_they_had_text`].
pub fn the_job_they_had(clean: &str, memory: &Memory) {
    let _ = the_job_they_had_text(clean, memory);
}

/// A job the person said they used to do, answered as that, or None.
pub fn the_job_they_had_text(clean: &str, memory: &Memory) -> Option<String> {
    if clean.contains('?') || !JOB_WAS.is_match(clean) {
        return None;
    }
    let job = squash(&py_str_or_empty(facts::extract(clean, false).get("job")));
    if job.is_empty() {
        return None;
    }
    if WAS_NOT_A_JOB.contains(&text::lower(text::split_ref(&job)[0])) {
        return None;
    }
    let pool = if JOB_BEFORE.is_match(clean) { &*PAST_JOB } else { &*PAST_JOB_PLAIN };
    Some(py_format(&pick(pool, memory, clean), &[("x", &job), ("X", &upper_first(&job))]))
}

/// Python: bot/memory/ack.py:5033 `a_job_to_do(text, memory=None)`
///
/// A thing with something still to be done to it, as the job it is, or None (a reading that
/// cannot be made answers nothing).
pub fn a_job_to_do(text_: &str, memory: &Memory) -> Option<String> {
    py_try(|| _a_job_to_do(text_, memory)).flatten()
}

/// Python: bot/memory/ack.py:5045 `_a_job_to_do(clean, memory=None)`
///
/// A thing with something still to be done to it, answered as the job it is.
pub fn _a_job_to_do(clean: &str, memory: &Memory) -> Option<String> {
    let spoken = _spoken(&_repairs(clean, memory));
    let read = if spoken.is_empty() { clean.to_string() } else { spoken };
    if !JOB_TO_DO.is_match(&read) {
        return None;
    }
    let found = HAVE_THING.search(&read)?;
    let thing = text::strip_chars(&text::lower(found.str(1)), "-'").to_string();
    if thing.is_empty() || NOT_A_THING.contains(&thing) || pyuni::is_digit(&thing) {
        return None;
    }
    if _about_a_body(&read, memory) || PLACED_IN_THE_PAST.is_match(&read) {
        return None;
    }
    if HABIT.is_match(&read) {
        return None; // a routine has its own reply
    }
    let what = _echo(clean, Some(memory));
    if what.is_empty() {
        return None;
    }
    // The bot says it in the person's own contraction: "you've got a wardrobe to put up".
    let what = HAVE_A.sub(r"\1've got a", &what);
    let tails: Vec<String> = if JOB_HAS_A_DAY.is_match(&read) { JOB_TAIL[1..].to_vec() } else { JOB_TAIL.clone() };
    Some(py_format(&pick(&tails, memory, &read), &[("thing", &what), ("Thing", &upper_first(&what))]))
}

/// Python: bot/memory/ack.py:5127 `_the_chat_holds_something(memory)`
///
/// Whether a line was said before this one, so a pointer in this line can point at it.
pub fn _the_chat_holds_something(memory: &Memory) -> bool {
    there_is_a_chat(memory)
}

/// Python: bot/memory/ack.py:5139 `_the_pronoun_points_at_somebody(low, memory)`
///
/// Whether a pronoun in this line can point at a person or an animal the chat has named.
pub fn _the_pronoun_points_at_somebody(low: &str, memory: &mut Memory) -> bool {
    if !crate::memory::relations::_person_from_pronoun(low, memory, &[]).is_empty() {
        return true;
    }
    if user_pets(memory).iter().any(|pet| pet.is_dict()) {
        return true;
    }
    // A "he"/"she" points at the person the chat has already named.
    let fits: Option<&HashSet<String>> = if RE_SHE_WORDS.is_match(low) {
        Some(&SHE)
    } else if RE_HE_WORDS.is_match(low) {
        Some(&HE)
    } else {
        None
    };
    if let Some(fits) = fits {
        let said = last_turns(memory, 4).iter().map(|t| text::lower(&t.user)).collect::<Vec<_>>().join(" ");
        let fits_list: Vec<String> = fits.iter().cloned().collect();
        let mut words: Vec<String> = PERSON_WORDS.iter().chain(CHILD.iter()).cloned().collect::<HashSet<_>>().into_iter().collect();
        words.sort_by(|a, b| text::len(b).cmp(&text::len(a)).then_with(|| a.cmp(b)));
        for word in &words {
            if !word_re(&format!(r"\b{word}\b")).is_match(&said) {
                continue;
            }
            if crate::memory::relations::_pronoun_fits(word, &fits_list) {
                return true;
            }
        }
    }
    if !RE_THEY_WORDS.is_match(low) {
        return false;
    }
    let mut words: HashSet<String> = RE_AZ_WORDS.findall_str(low).into_iter().collect();
    for turn in last_turns(memory, 4) {
        words.extend(RE_AZ_WORDS.findall_str(&text::lower(&turn.user)));
    }
    if words.iter().any(|w| PERSON_GROUP_KEYS.contains(w) || CHILD.contains(w)) {
        return true;
    }
    memory
        .get("others")
        .map(|o| o.items().to_vec())
        .unwrap_or_default()
        .iter()
        .filter(|e| e.is_dict())
        .any(|entry| !text::strip(&field(entry, "who")).is_empty() && !field(entry, "who").ends_with('s'))
}

/// Python: bot/memory/ack.py:5187 `_somebody_was_being_hurt(memory)`
///
/// Whether the line before this one said somebody is doing the person harm.
pub fn _somebody_was_being_hurt(memory: &Memory) -> bool {
    match turns(memory).last() {
        Some(last) => reads_as_cruelty(&last.user),
        None => false,
    }
}

/// Python: bot/memory/ack.py:5201 `_who_the_figure_is(found, memory, low)`
///
/// CONTRACT: typed `()`; the value is [`who_the_figure_is_text`].
pub fn _who_the_figure_is(found: &Value, memory: &mut Memory, low: &str) {
    let _ = who_the_figure_is_text(found, memory, low);
}

/// The question that asks who an age or a height belongs to, or None.
pub fn who_the_figure_is_text(found: &Value, memory: &mut Memory, low: &str) -> Option<String> {
    let entries = found.get("others").filter(|e| e.truthy())?.items().to_vec();
    if SOMEONE_MILESTONE.is_match(low) || A_DATE_STILL_TO_COME.is_match(low) {
        return None; // a milestone somebody is looking forward to
    }
    for entry in &entries {
        if !entry.is_dict() || entry.get("who").is_some_and(Value::truthy) {
            continue;
        }
        let figure = match entry.get("age").filter(|a| a.truthy()) {
            Some(a) => a.clone(),
            None => entry.get("height").cloned().unwrap_or(Value::None),
        };
        if !figure.truthy() {
            continue;
        }
        let pronoun = entry.get("pronoun").is_some_and(Value::truthy);
        if pronoun && _the_pronoun_points_at_somebody(low, memory) {
            return None; // somebody in the chat fits the pronoun
        }
        if pronoun && _somebody_was_being_hurt(memory) {
            // The room says there is exactly one person this is about: the age is theirs.
            let pron = text::lower(text::strip(&field(entry, "pronoun")));
            if let Value::Int(n) = figure {
                let said = format!("{pron}'s {n}");
                return Some(py_format(&pick(&HEAVY_FRAMES, memory, low), &[("thing", &said), ("Thing", &upper_first(&said))]));
            }
            return None;
        }
        if let Value::Int(n) = figure {
            if n != 0 {
                let said = n.to_string();
                return Some(py_format(&pick(&WHO_THE_FIGURE_IS, memory, low), &[("figure", &said), ("Figure", &said)]));
            }
        }
        return Some(pick(&WHO_IS_IT, memory, low));
    }
    None
}

/// Python: bot/memory/ack.py:5245 `_what_the_pointer_is(read, memory)`
///
/// The question that asks what a bare "them" names, or None. (`read` is the line's text, as
/// Python's callers pass it.)
pub fn _what_the_pointer_is(read: &str, memory: &Memory) -> Option<String> {
    if _the_chat_holds_something(memory) {
        return None;
    }
    if !POINTS_BACK.is_match(read) {
        return None;
    }
    if let Some(count) = A_COUNT_OF_THEM.search(read) {
        let counted = count.str(1);
        return Some(py_format(&pick(&WHAT_OF_THEM, memory, read), &[("Count", &upper_first(counted))]));
    }
    Some(pick(&WHICH_ONES, memory, read))
}

/// Python: bot/memory/ack.py:5322 `_the_route_for(thing)`
///
/// The frames that suit the kind of condition being wondered about.
pub fn _the_route_for(thing: &str) -> Vec<String> {
    let low = text::lower(thing);
    for (family, frames) in CONDITION_FAMILIES.iter() {
        if let Some(family) = family {
            if family.is_match(&low) {
                return frames.clone();
            }
        }
    }
    CONDITION_FAMILIES.last().map(|(_, f)| f.clone()).unwrap_or_default()
}

/// Python: bot/memory/ack.py:5331 `_the_entry_names(entry, head)`
///
/// Whether the entry that answered the line is about this thing at all.
pub fn _the_entry_names(entry: &'static crate::knowledge::entry::Entry, head: &str) -> bool {
    use crate::knowledge::entry::KeyPart;
    if entry.key.is_empty() {
        return true;
    }
    let words: Vec<&str> = entry.key.iter().filter_map(|k| if let KeyPart::Word(w) = k { Some(w.as_str()) } else { None }).collect();
    if words.contains(&head) {
        return true;
    }
    let prefix = |s: &str, n: usize| -> String { s.chars().take(n).collect() };
    words.iter().any(|word| text::len(word) > 3 && (prefix(word, 4) == prefix(head, 4) || prefix(word, 5) == prefix(head, 5)))
}

data_re!(RAISED, PERIOD, A_SPAN_OF_TIME, FOR_A_THING, BUYING, MONEY, THEIRS, SEPARATION);
data_strs!(SPENDING_FRAMES, BUDGET_FRAMES, CONDITION_FRAMES, DRINK_FRAMES, FOOD_FRAMES, STILL_FRAMES, OTHER_LIMIT_FRAMES, LIKES_FRAMES);
data_set!(NUMBER_WORDS, VERBISH);

static HEAVY_TURNS: LazyLock<usize> = LazyLock::new(|| data::get("bot.memory.ack.HEAVY_TURNS").as_int().unwrap() as usize);
static FAMILY: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.ack.FAMILY"));
inline_re!(RE_ASK_OPENS, r"\s*(?:when|what|how|why|where|who|is|are|does|do|will|did)\b", pyre::I);

/// CONTRACT: `loss::reply_for` is typed `()`; Python returns the loss reply or None, and only
/// whether it is None matters here. The call is made and read as None until the contract
/// carries the value.
fn loss_reply_for(text_: &str, memory: &Memory, clean: &str, topics_: &[String]) -> Option<String> {
    crate::compose::loss::reply_for(text_, memory, clean, topics_, None);
    None
}

/// `facts._kids_named_with_ages(text)`: the {name, age} dicts of the children a line names.
fn facts_kids_named_with_ages(text_: &str) -> Vec<Value> {
    facts::_kids_named_with_ages(text_)
}

/// `topics.an_entry_for(text)` names `head`: the entry's own key words, read off the exported
/// entry (`_the_entry_names` for a value).
fn entry_value_names(entry: &Value, head: &str) -> bool {
    let key = entry.attr("key").or_else(|| entry.get("key"));
    let words: Vec<String> = key.map(|k| k.items().iter().filter_map(|w| w.as_str().map(str::to_string)).collect()).unwrap_or_default();
    let has_key = key.is_some_and(Value::truthy);
    if !has_key {
        return true;
    }
    if words.iter().any(|w| w == head) {
        return true;
    }
    let prefix = |s: &str, n: usize| -> String { s.chars().take(n).collect() };
    words.iter().any(|word| text::len(word) > 3 && (prefix(word, 4) == prefix(head, 4) || prefix(word, 5) == prefix(head, 5)))
}

/// Python: bot/memory/ack.py:5349 `_a_condition_they_wonder_about(clean, memory=None)`
///
/// The answer to "i think i have <condition>", or "" when this is not that line.
pub fn _a_condition_they_wonder_about(clean: &str, memory: &Memory) -> String {
    if clean.is_empty() || A_WONDER_SAID_NO.is_match(clean) {
        return String::new();
    }
    let Some(found) = A_WONDERED_CONDITION.search(clean).map(|m| m.name_str("thing").to_string()) else { return String::new() };
    let mut words: Vec<String> = text::split(text::strip_chars(&flat(&found), "-' "));
    if words.is_empty() {
        return String::new();
    }
    let article = if WITH_AN_ARTICLE.contains(&words[0]) { words.remove(0) } else { String::new() };
    if words.is_empty() {
        return String::new();
    }
    let head = words[words.len() - 1].clone();
    if NOT_A_THING.contains(&head) || pyuni::is_digit(&head) {
        return String::new(); // not a condition at all
    }
    if words.iter().any(|w| NUMBER_WORDS.contains(w)) || head.chars().any(pyuni::ch_digit) {
        return String::new(); // a count, not a thing they have
    }
    if words.iter().any(|w| VERBISH.contains(w)) && !A_CONDITION_KIND.is_match(&head) {
        return String::new(); // something they did, not a condition they wonder about
    }
    let initials = CONDITION_INITIALS.contains(&head);
    if !article.is_empty() && !(A_CONDITION_KIND.is_match(&head) || initials) {
        return String::new();
    }
    if crate::compose::badnews::reads_like_a_symptom(&text::lower(clean)) {
        return String::new();
    }
    let entry = topics::an_entry_for(clean);
    if !entry.is_none() && entry_value_names(&entry, &head) {
        return String::new(); // an entry already answers this line
    }
    let mut phrase = words.iter().map(|w| if initials && *w == head { text::upper(w) } else { w.clone() }).collect::<Vec<_>>().join(" ");
    if !article.is_empty() {
        phrase = format!("{article} {phrase}");
    }
    py_format(&pick(&_the_route_for(&phrase), memory, &phrase), &[("thing", &phrase)])
}

/// Python: bot/memory/ack.py:5396 `_a_thing_they_have(clean, memory=None)`
///
/// CONTRACT: typed `()`; the value is [`a_thing_they_have_text`].
pub fn _a_thing_they_have(clean: &str, memory: &Memory) {
    let _ = a_thing_they_have_text(clean, memory);
}

/// A thing they have, said back: "i have a bike" -> "Noted: you have a bike".
pub fn a_thing_they_have_text(clean: &str, memory: &Memory) -> Option<String> {
    let spoken = _spoken(&_repairs(clean, memory));
    let read = if spoken.is_empty() { clean.to_string() } else { spoken };
    let found = HAVE_THING.search(&read)?;
    let thing = text::strip_chars(&text::lower(found.str(1)), "-'").to_string();
    if thing.is_empty() || NOT_A_THING.contains(&thing) || pyuni::is_digit(&thing) {
        return None;
    }
    if _about_a_body(&read, memory) || PLACED_IN_THE_PAST.is_match(&read) {
        return None;
    }
    if HABIT.is_match(&read) {
        return None; // a repeating time makes the line a routine
    }
    let what = _echo(clean, Some(memory));
    if what.is_empty() {
        return None;
    }
    if let Some(pointer) = _what_the_pointer_is(&read, memory).filter(|p| !p.is_empty()) {
        return Some(pointer);
    }
    Some(py_format(&pick(&FRAMES, memory, ""), &[("thing", &what), ("Thing", &upper_first(&what))]))
}

/// Python: bot/memory/ack.py:5437 `_a_spending_budget(clean, memory)`
///
/// Whether a stated budget is money to spend on a thing rather than a month to live inside.
pub fn _a_spending_budget(clean: &str, memory: &Memory) -> bool {
    if PERIOD.is_match(clean) || A_SPAN_OF_TIME.is_match(clean) {
        return false;
    }
    let re = |name: &str| data::re(&format!("bot.memory.facts.{name}"));
    if re("INTENDED_SPEND").is_match(clean) || re("THE_THING_MAY_COST").is_match(clean) || re("ALL_I_CAN_DO").is_match(clean) {
        return true;
    }
    if FOR_A_THING.is_match(clean) {
        return true; // the line names what the money is for
    }
    let mut recent: Vec<String> = vec![clean.to_string()];
    recent.extend(last_turns(memory, 3).iter().map(|t| t.user.clone()));
    if let Some(Value::List(topics_)) = memory.get("topics") {
        for topic in &topics_[topics_.len().saturating_sub(2)..] {
            if topic.is_dict() {
                recent.push(field(topic, "text"));
            }
        }
    }
    let said = text::lower(text::strip(clean));
    let others: Vec<&String> = recent.iter().filter(|line| text::lower(text::strip(line)) != said).collect();
    if RAISED.is_match(clean) {
        for line in &others {
            if FOR_A_THING.is_match(line) || BUYING.is_match(line) {
                return true;
            }
            if PERIOD.is_match(line) || A_SPAN_OF_TIME.is_match(line) {
                return false;
            }
        }
    }
    if MONEY.is_match(&text::lower(&others.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" "))) {
        return false;
    }
    BUYING.is_match(&text::lower(&recent.join(" ")))
}

/// Python: bot/memory/ack.py:5476 `_frames(found, heavy=False, clean='', memory=None)`
///
/// Which set of endings this fact wants: a limit is answered with what the bot does with it.
pub fn _frames(found: &Value, heavy: bool, clean: &str, memory: &Memory) -> Vec<String> {
    if found.get("budget").is_some_and(Value::truthy) {
        return if _a_spending_budget(clean, memory) { SPENDING_FRAMES.clone() } else { BUDGET_FRAMES.clone() };
    }
    let condition = text::lower(&py_str_or_empty(found.get("condition")));
    if CONDITION_FOOD.contains_key(&condition) {
        return CONDITION_FRAMES.clone();
    }
    if found.get("diet").is_some_and(Value::truthy) || found.get("allergies").is_some_and(Value::truthy) {
        if text::lower(&py_str_or_empty(found.get("diet"))).contains("alcohol") {
            return DRINK_FRAMES.clone();
        }
        return FOOD_FRAMES.clone();
    }
    if heavy && !_just_a_liking(found) {
        return if _about_the_family(found) { HEAVY_FRAMES.clone() } else { STILL_FRAMES.clone() };
    }
    if found.get("others").map(|o| o.items().iter().any(|e| e.is_dict() && e.get("allergies").is_some_and(Value::truthy))).unwrap_or(false) {
        return OTHER_LIMIT_FRAMES.clone();
    }
    if _about_someone_else(found, clean) {
        if SOMEONE_GOOD.is_match(clean) {
            return SOMEONE_TAIL.clone();
        }
        if _the_milestone(clean) {
            return SOMEONE_MILESTONE_TAIL.clone();
        }
        if _a_visit(clean) {
            return VISIT_TAIL.clone();
        }
    }
    if _a_workplace(found, heavy, memory) {
        // The line's own ending, rotated by its length.
        let step = text::len(clean) % WORK_FRAMES.len();
        return WORK_FRAMES[step..].iter().chain(WORK_FRAMES[..step].iter()).cloned().collect();
    }
    if let Some(coming) = _a_day_coming(clean) {
        return coming;
    }
    if _only_a_liking(found) && !a_liked_thing(found).is_empty() && _a_like_of_one_thing(clean, memory) {
        let step = text::len(clean) % LIKES_FRAMES.len();
        return LIKES_FRAMES[step..].iter().chain(LIKES_FRAMES[..step].iter()).cloned().collect();
    }
    FRAMES.clone()
}

/// Python: bot/memory/ack.py:5533 `_about_someone_else(found, clean='')`
///
/// Whether the line's fact is about a person of theirs rather than about the person typing.
pub fn _about_someone_else(found: &Value, clean: &str) -> bool {
    if ["people", "others", "kids", "relations"].iter().any(|k| found.get(k).is_some_and(Value::truthy)) {
        return true;
    }
    THEIRS.is_match(clean)
}

/// Python: bot/memory/ack.py:5545 `_a_visit(clean)`
///
/// Whether the line is a visit announced, rather than a lift or a question.
pub fn _a_visit(clean: &str) -> bool {
    if text::strip(clean).is_empty() || text::rstrip(clean).ends_with('?') || NOT_A_VISIT.is_match(clean) {
        return false;
    }
    if RE_ASK_OPENS.match_(clean).is_some() {
        return false;
    }
    SOMEONE_COMING.is_match(clean)
}

/// Python: bot/memory/ack.py:5559 `_the_milestone(clean)`
///
/// A round birthday or an anniversary of somebody else's, which is not the same as an age.
pub fn _the_milestone(clean: &str) -> bool {
    SOMEONE_MILESTONE.is_match(clean)
}

/// Python: bot/memory/ack.py:5568 `_a_workplace(found, heavy=False, memory=None)`
///
/// Whether the line told the bot where the person works, and nothing else.
pub fn _a_workplace(found: &Value, heavy: bool, memory: &Memory) -> bool {
    if heavy || !found.is_dict() || !found.get("job").is_some_and(Value::truthy) {
        return false;
    }
    let rest = found.pairs().iter().any(|(k, v)| v.truthy() && k.str() != "job" && !k.str().ends_with("_frame"));
    if rest {
        return false;
    }
    let frame = match memory.get("user") {
        Some(user) if user.is_dict() => text::lower(text::strip(&field(user, "job_frame"))),
        _ => String::new(),
    };
    !frame.is_empty() && WORK_PLACE.contains(text::split_ref(&frame)[0])
}

/// Python: bot/memory/ack.py:5594 `_about_the_family(found)`
///
/// Whether the fact filed is about a person of theirs rather than a pet, a job or a taste.
pub fn _about_the_family(found: &Value) -> bool {
    FAMILY.iter().any(|k| found.get(k).is_some_and(Value::truthy))
}

/// Python: bot/memory/ack.py:5604 `_just_a_liking(found)`
///
/// Whether the only thing this line filed is a taste, which is not something that lands.
pub fn _just_a_liking(found: &Value) -> bool {
    let rest = found.pairs().iter().any(|(k, v)| v.truthy() && k.str() != "tastes" && !k.str().ends_with("_frame"));
    found.get("tastes").is_some_and(Value::truthy) && !rest
}

/// Python: bot/memory/ack.py:5617 `_heavy(memory)`
///
/// Whether the last few turns hold a death, a separation or harm somebody is doing.
pub fn _heavy(memory: &Memory) -> bool {
    for turn in last_turns(memory, *HEAVY_TURNS) {
        let said = squash(&turn.user);
        if said.is_empty() {
            continue;
        }
        if crate::compose::loss::reads_as_loss(&said, None, false) || SEPARATION.is_match(&text::lower(&said)) || reads_as_cruelty(&said) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/ack.py:5635 `_statement(memory)`
///
/// Whether the understanding engine read the line as telling the bot something.
pub fn _statement(memory: &Memory) -> bool {
    match memory.understood.as_ref().map(|u| u.kind.as_str()) {
        Some(kind) if !kind.is_empty() => ["statement", "tell", "news"].contains(&kind),
        _ => true,
    }
}

/// Python: bot/memory/ack.py:5644 `_a_loss(text, memory, clean)`
///
/// Whether bot/compose/loss.py reads a death or a grief in the line: that is its turn.
pub fn _a_loss(text_: &str, memory: &Memory, clean: &str) -> bool {
    let topics_: Vec<String> = memory.understood.as_ref().map(|u| u.topics.clone()).unwrap_or_default();
    loss_reply_for(text_, memory, clean, &topics_).is_some()
}

/// Python: bot/memory/ack.py:5651 `_learned(text, memory=None)`
///
/// The facts in the line for the store to keep (a dict), or an empty dict when there are none.
pub fn _learned(text_: &str, memory: Option<&Memory>) -> Value {
    let mut found = facts::extract(&_spoken(text_), false);
    if !found.truthy() {
        found = facts::extract(text_, false);
    }
    let kept: Vec<(Value, Value)> =
        found.pairs().iter().filter(|(k, v)| FACTS.iter().any(|f| f == k.str()) && v.truthy()).cloned().collect();
    if !kept.is_empty() {
        return Value::Dict(std::sync::Arc::new(kept));
    }
    let more = _adds_one_more_allergen(text_, memory);
    if !more.is_empty() {
        let mut out = Value::dict();
        out.set_key("allergies", Value::from(more));
        return out;
    }
    if _adds_a_detail(text_, memory) {
        let mut out = Value::dict();
        out.set_key("detail", true);
        return out;
    }
    Value::dict()
}

/// Python: bot/memory/ack.py:5673 `_children_with_ages(text)`
///
/// The children a line names with an age: "my kids are ada 7 and sam 4" (dicts of name, age).
pub fn _children_with_ages(text_: &str) -> Vec<Value> {
    py_try(|| facts_kids_named_with_ages(text_)).unwrap_or_default()
}

/// Python: bot/memory/ack.py:5682 `_children_said(kids)`
///
/// The children said back by name and age: "Ada is 7 and Sam is 4".
pub fn _children_said(kids: &[Value]) -> String {
    let mut parts = Vec::new();
    for kid in kids.iter().take(4) {
        let name = field(kid, "name");
        if name.is_empty() {
            continue;
        }
        let age = py_str(kid.get("age").unwrap_or(&Value::None));
        parts.push(format!("{} is {}", upper_first(&name), age));
    }
    parts.join(" and ")
}

/// Python: bot/memory/ack.py:5693 `_adds_one_more_allergen(text, memory)`
///
/// The allergy a line adds to the ones on record, or [].
pub fn _adds_one_more_allergen(text_: &str, memory: Option<&Memory>) -> Vec<String> {
    let Some(memory) = memory else { return Vec::new() };
    let held = match memory.get("user") {
        Some(user) if user.is_dict() => user.get("allergies").cloned().unwrap_or(Value::None),
        _ => Value::None,
    };
    if !matches!(held, Value::List(ref l) if !l.is_empty()) {
        return Vec::new();
    }
    facts::one_more(text_)
}

// ---------------------------------------------------------------------------------------------
// A detail of the thing in hand, and pointers back at it.

data_re!(
    _NOT_A_DETAIL, _ANSWER_OPENER, _ASKED_FOR, _SAID_YES_NO, _ASKS_MIDWAY, _FAULT, _NOT_A_DETAIL_SUBJECT,
    DIGIT, REPAIR_FRAME, WORD, _OPENS_WITH_A_LINK, MONEY_ONLY, KIN_POSSESSIVE, A_COUNT_NOT_A_STATE, WHOS,
    POINTS_AT_ANOTHER, A_DECISION, REPORTS_AN_ACTION, SOMEONE_ELSE_POINTS_AT_ANOTHER, PRONOUN_POINTS_AT_ANOTHER,
    A_NAME_GIVEN, A_DETERMINER, HAVE_GOT, STOP, POLITE, STATE, FIGURE_LEAD, A_CONTINUATION, A_CLAUSE, A_SOMEONE,
    NO_THEN_A_LINE_ONLY, NO_LEAD, _REACTION_CLAUSE, COUNTED_STARS, ONE_STAR, COUNTED_LETTERS, A_DEGREE,
    DEGREE_CLASS, WITH_SLASH, TWO_TO, TWO_PLACE, TWO_VERB, CONTRAST, CONTRAST_SAID_AGAIN, PICK_FRAME,
    A_REASON_TAIL, A_CORRECTION_IN_FRONT, VALUE,
);
data_set!(
    _EMPTY, VAGUE, UNITS_OF_MEASURE, POINTER_FILLER, NOT_A_THING_TO_HAVE_ANOTHER, ENDS_THE_THING_VERB,
    ENDS_THE_THING, NOT_A_SECOND_OF, FIGURE_FILLER, FIGURE_UNIT, FIGURE_WORD, REACTION_PHRASES, REACTION_WORDS,
    YES_AFTER, ITS_OWNS_AFTER, NOT_A_VALUE, BETWEEN_VALUES,
);
data_strs!(NO_REFERENT, ANOTHER_ONE, ANOTHER_ONES, BEFORE_ONE, ANOTHER_ONE_FOR_SOMEONE, ANOTHER_SOME_FOR_SOMEONE, LEAD_IN);
data_map!(SPEAK, SHORT);

static MOST: LazyLock<usize> = LazyLock::new(|| data::get("bot.memory.ack.MOST").as_int().unwrap() as usize);
static CLAUSES: LazyLock<usize> = LazyLock::new(|| data::get("bot.memory.ack.CLAUSES").as_int().unwrap() as usize);
static FRAGMENT: LazyLock<usize> = LazyLock::new(|| data::get("bot.memory.ack.FRAGMENT").as_int().unwrap() as usize);
static NAMED_NOT_THE_THING: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.named.NOT_THE_THING"));
static FACTS_DIED: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.facts.DIED"));
static DISLIKE_IN: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.taste.DISLIKE_IN"));
static FILLER_TAIL_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.facts.FILLER_TAIL_PHRASES"));
/// `LEFT_OFF_CONTRACTION`: a word typed flat and the words it stands for ("were" -> we are).
static LEFT_OFF_CONTRACTION: LazyLock<HashMap<String, Vec<String>>> =
    LazyLock::new(|| data::get("bot.memory.ack.LEFT_OFF_CONTRACTION").pairs().iter().map(|(k, v)| (py_str(k), v.strs())).collect());
/// `{int: str}` tables (`STAR_COUNT`, `LETTER_COUNT`).
fn count_words(name: &str) -> HashMap<i64, String> {
    data::get(name).pairs().iter().map(|(k, v)| (k.as_int().unwrap_or(0), py_str(v))).collect()
}
static STAR_COUNT: LazyLock<HashMap<i64, String>> = LazyLock::new(|| count_words("bot.memory.ack.STAR_COUNT"));
static LETTER_COUNT: LazyLock<HashMap<i64, String>> = LazyLock::new(|| count_words("bot.memory.ack.LETTER_COUNT"));
/// `sorted(LEAD_IN, key=len, reverse=True)` (a stable sort, as Python's).
static LEADS_LONGEST_FIRST: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut leads = LEAD_IN.clone();
    leads.sort_by(|a, b| text::len(b).cmp(&text::len(a)));
    leads
});

inline_re!(RE_A_LETTER, "[A-Za-z\u{00c0}-\u{024f}]", 0);
inline_re!(RE_NOT_AZ, r"[^a-z]+", 0);
inline_re!(RE_ALNUM_ANY, r"[a-z0-9]", pyre::I);
inline_re!(RE_SPACES, r"\s+", 0);
inline_re!(RE_FIGURE_WORD_SHAPE, r"\d{1,3}(?:st|nd|rd|th)?(?:(?:cm|m|ft|in|kg|st|lbs?)s?)?", 0);

/// `trouble.reads_as_trouble(read, words)`: the reading's shape, or None.
fn trouble_reading_with_words(read: &str) -> Option<String> {
    let words = text::split(read);
    crate::compose::trouble::reads_as_trouble(read, Some(&words), "")
}

/// Python: bot/memory/ack.py:5712 `_adds_a_detail(text, memory)`
///
/// Whether the line says something new about the thing we are on, with nothing to store.
pub fn _adds_a_detail(text_: &str, memory: Option<&Memory>) -> bool {
    let Some(memory) = memory else { return false };
    // The topic it belongs to is the one we were on before the line (topics.carried_topic).
    let Some(record) = topics::carried_topic(memory, text_).or_else(|| topics::current(memory)).filter(|r| r.truthy()) else {
        return false;
    };
    let clean = squash(text_);
    let read = _repaired(&clean, memory);
    if clean.contains('?') || WANTS_ANSWER.is_match(&read) || TROUBLE.is_match(&read) {
        return false;
    }
    if trouble_reading_with_words(&read).is_some() {
        return false;
    }
    let said_words = _words(&clean);
    if said_words.len() < 2 {
        return false;
    }
    // A bare figure is a figure, not a detail.
    if !RE_A_LETTER.is_match(&_strip_lead_in(&clean, false)) {
        return false;
    }
    if !RE_A_LETTER.is_match(&REPAIR_FRAME.sub("", &clean)) {
        return false;
    }
    if n_words(&clean) <= 2 && !DIGIT.is_match(&clean) {
        return false;
    }
    let carried = format!("{} {}", field(record, "text"), last_turns(memory, 3).iter().map(|t| t.user.clone()).collect::<Vec<_>>().join(" "));
    if DEATH_WORDS.is_match(&carried) || _NOT_A_DETAIL_SUBJECT.is_match(&carried) {
        return false;
    }
    if last_turns(memory, 2).iter().any(|t| crate::compose::badnews::reads_as_bad_news(&t.user)) {
        return false;
    }
    let mut known: HashSet<String> = _words(&field(record, "text")).into_iter().collect();
    if let Some(details) = record.get("details") {
        for earlier in details.items() {
            known.extend(_words(&py_str(earlier)));
        }
    }
    if _NOT_A_DETAIL.is_match(&read)
        || _ANSWER_OPENER.match_(&read).is_some()
        || _ASKED_FOR.match_(&read).is_some()
        || _SAID_YES_NO.match_(&read).is_some()
        || _ASKS_MIDWAY.is_match(&read)
        || _FAULT.is_match(&read)
    {
        return false;
    }
    said_words.iter().any(|w| !known.contains(w))
}

/// Python: bot/memory/ack.py:5780 `_words(text)`
///
/// The words of `text` that carry meaning: no articles, no auxiliaries, lower case.
pub fn _words(text_: &str) -> Vec<String> {
    WORD.findall_str(&text::lower(text_)).into_iter().filter(|w| !_EMPTY.contains(w)).collect()
}

/// Python: bot/memory/ack.py:5798 `_names_nothing(body)`
///
/// True when the words left are only filler and vague ones, so there is nothing to say back.
pub fn _names_nothing(body: &str) -> bool {
    _words(body).iter().all(|w| VAGUE.contains(w))
}

/// Python: bot/memory/ack.py:5817 `_repairs(clean, memory)`
///
/// The line with the understanding engine's own corrections in it.
pub fn _repairs(clean: &str, memory: &Memory) -> String {
    repairs_with(clean, Some(memory))
}

fn repairs_with(clean: &str, memory: Option<&Memory>) -> String {
    let Some(plain) = memory.and_then(|m| m.understood.as_ref()).map(|u| u.plain.clone()) else { return clean.to_string() };
    if text::strip(&plain).is_empty() {
        return clean.to_string();
    }
    let typed = text::split(clean);
    let fixed = text::split(&plain);
    if typed.is_empty() {
        return clean.to_string();
    }
    let bare_alpha = |s: &str| pyuni::is_alpha(&s.replace(['\'', '\u{2019}'], ""));
    let mut pairs: Vec<(String, String)> = Vec::new();
    let (mut was_at, mut now_at) = (0usize, 0usize);
    while was_at < typed.len() && now_at < fixed.len() {
        let words = LEFT_OFF_CONTRACTION.get(&text::lower(&typed[was_at])).filter(|w| !w.is_empty());
        if let Some(words) = words.filter(|w| fixed.get(now_at..now_at + w.len()).is_some_and(|f| f == w.as_slice())) {
            pairs.push((typed[was_at].clone(), words.join(" ")));
            now_at += words.len();
        } else if text::lower(&typed[was_at]) == text::lower(&fixed[now_at]) || (pyuni::is_alpha(&typed[was_at]) && bare_alpha(&fixed[now_at])) {
            pairs.push((typed[was_at].clone(), fixed[now_at].clone()));
            now_at += 1;
        } else {
            return clean.to_string();
        }
        was_at += 1;
    }
    if was_at != typed.len() || now_at != fixed.len() {
        return clean.to_string();
    }
    let words: Vec<String> = pairs
        .into_iter()
        .map(|(was, now)| {
            if was != now && pyuni::is_alpha(&was) && (bare_alpha(&now) || now.contains(' ')) && text::lower(&was) != text::lower(&now) {
                if was.chars().next().is_some_and(pyuni::ch_upper) { upper_first(&now) } else { now }
            } else {
                was
            }
        })
        .collect();
    words.join(" ")
}

/// Python: bot/memory/ack.py:5907 `_says_something(body)`
///
/// Whether the words left after a repair frame say something, rather than naming a bare value.
pub fn _says_something(body: &str) -> bool {
    facts::_tail_has_a_verb(&RE_AZ_WORDS.findall_str(&text::lower(body)))
}

/// Python: bot/memory/ack.py:6003 `_the_person_the_pointer_is_about(body, memory)`
///
/// The person the pointer line is about ("wife", "mum", "sister"), or "" for the person talking.
pub fn _the_person_the_pointer_is_about(body: &str, memory: &Memory) -> String {
    if let Some(found) = SOMEONE_ELSE_POINTS_AT_ANOTHER.search(body) {
        let word = text::lower(text::strip(found.str(1)));
        return if word == "own" || word == "self" { String::new() } else { word };
    }
    if PRONOUN_POINTS_AT_ANOTHER.is_match(body) {
        // "she got one too": the person is the last relation the chat named.
        for turn in last_turns(memory, 3).iter().rev() {
            let said = text::lower(&turn.user);
            for word in RELATIONS.iter() {
                if word_re(&format!(r"\bmy\s+{}\b", pyre::escape(word))).is_match(&said) {
                    return word.clone();
                }
            }
        }
    }
    String::new()
}

/// Python: bot/memory/ack.py:6022 `_the_thing_the_chat_is_on(memory)`
///
/// The thing the chat is about, in the person's word, whether they said several, and the plural.
pub fn _the_thing_the_chat_is_on(memory: &Memory) -> (String, bool, String) {
    let nothing = (String::new(), false, String::new());
    let phrase = text::lower(&squash(&topics::subject(memory)));
    if phrase.is_empty() {
        return nothing;
    }
    let mut named_words: HashSet<String> = HashSet::new();
    for turn in last_turns(memory, 3).iter().rev() {
        named_words.extend(_a_thing_the_line_named(&turn.user));
    }
    named_words.retain(|w| !NOT_A_SECOND_OF.contains(w));
    if named_words.is_empty() {
        return nothing;
    }
    let stems: HashSet<String> = named_words.iter().map(|w| topics::stem(w)).collect();
    let mut stem = String::new();
    for word in RE_NOT_AZ.split(&phrase) {
        if text::len(&word) > 2
            && !NAMED_NOT_THE_THING.contains(&word)
            && !NOT_A_THING_TO_HAVE_ANOTHER.contains(&word)
            && (named_words.contains(&word) || stems.contains(&topics::stem(&word)))
        {
            stem = word;
        }
    }
    if stem.is_empty() || NOT_A_THING_TO_HAVE_ANOTHER.contains(&stem) || RELATION_SET.contains(&stem) {
        return nothing;
    }
    if _something_they_avoid(memory, &stem) || _a_name_the_store_holds(memory, &stem) {
        return nothing;
    }
    let mut said = String::new();
    'turns: for turn in last_turns(memory, 3).iter().rev() {
        for word in RE_AZ_WORDS.findall_str(&text::lower(&turn.user)) {
            if word == stem || topics::stem(&word) == stem {
                said = word;
                break 'turns;
            }
        }
    }
    if said.is_empty() {
        said = stem.clone();
    }
    let plural = _plural(&said);
    let several = said != stem;
    (said, several, plural)
}

/// Python: bot/memory/ack.py:6073 `_a_name_the_store_holds(memory, stem)`
///
/// Whether the word is a name somebody or some animal was given, not a thing of its own.
pub fn _a_name_the_store_holds(memory: &Memory, stem: &str) -> bool {
    let mut names: Vec<String> = Vec::new();
    for pet in user_pets(memory) {
        if pet.is_dict() && pet.get("name").is_some_and(Value::truthy) {
            names.push(field(&pet, "name"));
        }
    }
    if let Some(others) = memory.get("others") {
        for other in others.items() {
            if other.is_dict() && other.get("name").is_some_and(Value::truthy) {
                names.push(field(other, "name"));
            }
        }
    }
    if let Some(person) = memory.get("person").filter(|p| p.is_dict()) {
        if person.get("name").is_some_and(Value::truthy) {
            names.push(field(person, "name"));
        }
    }
    for turn in last_turns(memory, 3) {
        if let Some(found) = A_NAME_GIVEN.search(&turn.user) {
            names.push(found.str(1).to_string());
        }
    }
    names.iter().filter(|n| !n.is_empty()).any(|name| topics::stem(name) == stem)
}

/// Python: bot/memory/ack.py:6102 `_something_they_avoid(memory, stem)`
///
/// Whether the thing the chat is on is something the person is allergic to or avoiding.
pub fn _something_they_avoid(memory: &Memory, stem: &str) -> bool {
    let mut words: Vec<Value> = Vec::new();
    if let Some(user) = memory.get("user").filter(|u| u.is_dict()) {
        if let Some(a) = user.get("allergies").filter(|a| a.truthy()) {
            words.extend(a.items().iter().cloned());
        }
    }
    if let Some(meal) = memory.get("meal").filter(|m| m.is_dict()) {
        if let Some(a) = meal.get("avoid").filter(|a| a.truthy()) {
            words.extend(a.items().iter().cloned());
        }
    }
    words.iter().any(|w| topics::stem(&py_str(w)) == stem)
}

/// Python: bot/memory/ack.py:6120 `_plural(word)`
///
/// The plural of a thing the person named, so a reply can say "two puppies".
pub fn _plural(word: &str) -> String {
    if word.ends_with('s') {
        return word.to_string();
    }
    if text::ends_with_any(word, &["x", "z", "ch", "sh"]) {
        return format!("{word}es");
    }
    let chars: Vec<char> = word.chars().collect();
    if chars.len() > 2 && word.ends_with('y') && !"aeiou".contains(chars[chars.len() - 2]) {
        return format!("{}ies", &word[..word.len() - 1]);
    }
    format!("{word}s")
}

/// Python: bot/memory/ack.py:6147 `_is_a_contraction(word)`
///
/// Whether the word is a contraction ("isn't", "wont", "didnt") rather than a thing.
pub fn _is_a_contraction(word: &str) -> bool {
    word.contains("n't") || ENDS_THE_THING_VERB.contains(word)
}

/// Python: bot/memory/ack.py:6178 `_a_thing_the_line_named(text)`
///
/// The head words of the noun phrases a line names behind a determiner.
pub fn _a_thing_the_line_named(text_: &str) -> HashSet<String> {
    let words = RE_AZ_WORDS.findall_str(&text::lower(text_));
    let mut out = HashSet::new();
    for (at, word) in words.iter().enumerate() {
        if A_DETERMINER.fullmatch(word).is_none() {
            continue;
        }
        let mut head = String::new();
        for after in words.iter().skip(at + 1).take(3) {
            if ENDS_THE_THING.contains(after) || text::len(after) <= 2 || _is_a_contraction(after) {
                break;
            }
            head = after.clone();
        }
        if !head.is_empty() {
            out.insert(head);
        }
    }
    out
}

/// Python: bot/memory/ack.py:6197 `_a_loss_just_before(memory)`
///
/// Whether the last turn or two is a death: that turn belongs to the feelings reader.
pub fn _a_loss_just_before(memory: &Memory) -> bool {
    last_turns(memory, 2).iter().any(|t| FACTS_DIED.is_match(&t.user))
}

/// Python: bot/memory/ack.py:6208 `_another_of_what_we_said(clean, memory)`
///
/// CONTRACT: typed `()`; the value is [`another_of_what_we_said_text`].
pub fn _another_of_what_we_said(clean: &str, memory: &Memory) {
    let _ = another_of_what_we_said_text(clean, memory);
}

/// The reply to a line that only points at another of the thing the chat is about, or None.
pub fn another_of_what_we_said_text(clean: &str, memory: &Memory) -> Option<String> {
    let body = text::strip(clean);
    if body.is_empty() || body.contains('?') || n_words(body) > 8 {
        return None;
    }
    if A_DECISION.is_match(body) {
        return None;
    }
    let found = text::lower(POINTS_AT_ANOTHER.search(body)?.as_str());
    let left = RE_AZ_WORDS.findall_str(&text::lower(&POINTS_AT_ANOTHER.sub(" ", body))).into_iter().any(|w| !POINTER_FILLER.contains(&w));
    if left {
        return None; // the line names a thing of its own
    }
    if REPORTS_AN_ACTION.is_match(body) && found != "one before" && found != "the same one" {
        return None;
    }
    if _a_loss_just_before(memory) {
        return None;
    }
    let (one, several, plural) = _the_thing_the_chat_is_on(memory);
    if one.is_empty() {
        return Some(pick(&NO_REFERENT, memory, body));
    }
    let person = _the_person_the_pointer_is_about(body, memory);
    if !person.is_empty() {
        let pool = if several { &*ANOTHER_SOME_FOR_SOMEONE } else { &*ANOTHER_ONE_FOR_SOMEONE };
        return Some(py_format(&pick(pool, memory, body), &[("person", &person), ("thing", &one), ("things", &plural)]));
    }
    if found == "one before" {
        return Some(py_format(&pick(&BEFORE_ONE, memory, body), &[("thing", &one)]));
    }
    if several {
        return Some(py_format(&pick(&ANOTHER_ONES, memory, body), &[("things", &plural)]));
    }
    Some(py_format(&pick(&ANOTHER_ONE, memory, body), &[("thing", &one), ("things", &plural)]))
}

/// Python: bot/memory/ack.py:6258 `_only_the_correction_in_front(kept, body)`
///
/// True when `kept` is `body` with nothing in front of it but the correction's own opener.
pub fn _only_the_correction_in_front(kept: &str, body: &str) -> bool {
    let kept = squash(kept);
    let body = squash(body);
    if kept.is_empty() || body.is_empty() || kept == body {
        return false;
    }
    if !text::lower(&kept).ends_with(&text::lower(&body)) {
        return false;
    }
    let front = text::slice(&kept, 0, Some((text::len(&kept) - text::len(&body)) as isize));
    !text::strip(&front).is_empty() && facts::is_a_correction_opener(&front)
}

/// Python: bot/memory/ack.py:6281 `_echo(clean, memory=None)`
///
/// What they said, said back and short ("" for nothing to say back).
pub fn _echo(clean: &str, memory: Option<&Memory>) -> String {
    let clean = crate::understanding::normalize::without_a_leading_exclamation(clean);
    let said_it = repairs_with(&clean, memory);
    let mut body = _strip_lead_in(&said_it, false);
    if !body.is_empty() && !_says_something(&body) {
        // A bare value keeps the repair's own words in front: "you mean three".
        let kept = _strip_lead_in(&said_it, true);
        if kept != body && !_only_the_correction_in_front(&kept, &body) {
            body = kept;
        }
    }
    if body.is_empty() || _names_nothing(&body) {
        return String::new();
    }
    if _OPENS_WITH_A_LINK.match_(&body).is_some() && n_words(&body) <= 3 && MONEY_ONLY.match_(text::strip_chars(&body, " .,!?")).is_none() {
        return String::new(); // half a sentence
    }
    let said = if text::len(&body) <= *MOST {
        body.clone()
    } else {
        let clauses: Vec<String> = STOP.split(&body).iter().map(|c| text::strip(c).to_string()).filter(|c| !c.is_empty()).collect();
        if clauses.is_empty() { body.clone() } else { clauses.iter().take(*CLAUSES).cloned().collect::<Vec<_>>().join(", ") }
    };
    let said = text::strip_chars(&_cut(&said), " ,;:-!?.\u{2014}").to_string();
    let said = text::strip_chars(&POLITE.sub("", &said), " ,;:-").to_string();
    let said = _tail_off(&said);
    let said = text::strip_chars(&_contrast_off(&said), " ,;:-").to_string();
    let said = text::strip_chars(&_reason_off(&said, &clean), " ,;:-").to_string();
    let said = text::strip_chars(&_pick_off(&said), " ,;:-").to_string();
    if said.is_empty() {
        return String::new();
    }
    let you = crate::memory::you(&_spoken(&said));
    let back = _tidy(&KIN_POSSESSIVE.sub(r"\1's", text::strip_chars(&you, " ,;:-")));
    if back.is_empty() {
        return String::new();
    }
    let mut back = HAVE_GOT.sub(r"\1've got", &back);
    // A count of things after "i'm" is what they have, not what they are.
    if let Some((start, unit)) = A_COUNT_NOT_A_STATE.match_(&back).map(|m| (m.start_of(1).unwrap(), text::lower(m.str(2)))) {
        if !UNITS_OF_MEASURE.contains(&unit) {
            back = format!("you've got {}", &back[start..]);
        }
    }
    WHOS.sub("who's", &back)
}

/// Python: bot/memory/ack.py:6335 `said_back(clean, memory=None)`
///
/// Their line in the bot's own words, short (None when there is nothing to say back).
pub fn said_back(clean: &str, memory: &Memory) -> Value {
    let said = _echo(clean, Some(memory));
    if said.is_empty() { Value::None } else { Value::from(said) }
}

/// Python: bot/memory/ack.py:6343 `_only_a_dislike(body, found)`
///
/// True when the line is a dislike of a thing and nothing else.
pub fn _only_a_dislike(body: &str, found: &Value) -> bool {
    if !found.get("tastes").is_some_and(Value::truthy) {
        return false;
    }
    !TROUBLE.is_match(&DISLIKE_IN.sub(" ", body))
}

/// Python: bot/memory/ack.py:6356 `only_a_dislike(text, memory=None)`
///
/// True when the line is a stated dislike of a thing and nothing else.
pub fn only_a_dislike(text_: &str, memory: &Memory) -> bool {
    let clean = squash(text_);
    if clean.is_empty() {
        return false;
    }
    _only_a_dislike(&_strip_lead_in(&clean, false), &_learned(&clean, Some(memory)))
}

/// Python: bot/memory/ack.py:6369 `_tells_it(clean, found=None)`
///
/// Whether the line can be said back: a state of their world, not someone's misbehaviour.
pub fn _tells_it(clean: &str, found: &Value) -> bool {
    let body = _strip_lead_in(clean, false);
    let others: Vec<Value> = found.get("others").map(|o| o.items().iter().filter(|e| e.is_dict()).cloned().collect()).unwrap_or_default();
    if others.iter().any(|e| e.get("name").is_some() || e.get("age").is_some()) {
        return true; // a fact about them, however long the sentence is
    }
    if !facts::foods_ruled_out(found).is_empty() {
        return true;
    }
    if TROUBLE.is_match(&body) && !_a_diet(found) && !_only_a_dislike(&body, found) {
        return false;
    }
    if found.get("done").is_some_and(Value::truthy) {
        return true;
    }
    STATE.is_match(&body) || text::len(&body) <= *FRAGMENT
}

/// Python: bot/memory/ack.py:6390 `_a_diet(found)`
///
/// Whether the store read a diet in the line, so a negation in it is not a complaint.
pub fn _a_diet(found: &Value) -> bool {
    if !found.is_dict() {
        return false;
    }
    if found.get("diet").is_some_and(Value::truthy) || found.get("no_meat").is_some_and(Value::truthy) {
        return true;
    }
    found.get("others").map(|o| o.items().iter().any(|e| e.is_dict() && e.get("diet").is_some_and(Value::truthy))).unwrap_or(false)
}

/// Python: bot/memory/ack.py:6413 `_only_the_figure(clean)`
///
/// Whether the line says nothing but an age or a height: "im 34", "im 6 foot 2".
pub fn _only_the_figure(clean: &str) -> bool {
    let body = squash(&FIGURE_LEAD.sub("", &_strip_lead_in(clean, false)));
    text::split_ref(&body)
        .iter()
        .map(|w| text::strip_chars(w, ".,!;:"))
        .filter(|w| !w.is_empty() && !FIGURE_FILLER.contains(&text::lower(w)))
        .all(|w| RE_FIGURE_WORD_SHAPE.fullmatch(w).is_some() || FIGURE_UNIT.contains(&text::lower(w)) || FIGURE_WORD.contains(&text::lower(w)))
}

/// Python: bot/memory/ack.py:6446 `_the_rest_of_a_question(clean, before='')`
///
/// Whether the line is the tail of the question before it, with no fact of its own.
pub fn _the_rest_of_a_question(clean: &str, before: &str) -> bool {
    let line = squash(clean);
    let Some(found) = A_CONTINUATION.match_(&line) else { return false };
    let rest = text::strip_chars(&line[found.end()..], " ,.;:-!?");
    if rest.is_empty() {
        return false;
    }
    if A_CLAUSE.is_match(rest) || A_SOMEONE.is_match(rest) {
        return false;
    }
    reads_as_an_ask(before)
}

/// Python: bot/memory/ack.py:6548 `_after_a_reaction(text)`
///
/// `text` without a short reaction clause in front of it ("lucky you.", "fair enough,").
pub fn _after_a_reaction(text_: &str) -> String {
    let Some(found) = _REACTION_CLAUSE.match_(text_) else { return text_.to_string() };
    let clause = text::lower(&squash(found.str(1)));
    let words: Vec<String> = text::split_ref(found.str(1)).iter().map(|w| text::strip_chars(w, "'\u{2019}").to_string()).collect();
    if words.is_empty() || words.len() > 5 {
        return text_.to_string();
    }
    if !REACTION_PHRASES.contains(&clause) && !words.iter().all(|w| REACTION_WORDS.contains(&text::lower(w))) {
        return text_.to_string();
    }
    text::strip(&text_[found.end()..]).to_string()
}

/// Python: bot/memory/ack.py:6563 `_strip_lead_in(clean, keep_the_repair=False)`
///
/// `clean` without the "hi, so," and the like in front of the fact.
pub fn _strip_lead_in(clean: &str, keep_the_repair: bool) -> String {
    let mut text_ = text::strip_chars(clean, " ,;:-").to_string();
    // The bare "no" comes off either way.
    text_ = NO_THEN_A_LINE_ONLY.sub("", &text_);
    if !keep_the_repair {
        text_ = NO_LEAD.sub("", &text_);
    }
    let mut changed = true;
    while changed {
        changed = false;
        let bare = if keep_the_repair { text_.clone() } else { NO_LEAD.sub("", &text_) };
        if bare != text_ {
            text_ = bare;
            changed = true;
            continue;
        }
        for lead in LEADS_LONGEST_FIRST.iter() {
            let low = text::lower(&text_);
            let n = text::len(lead);
            if low.starts_with(lead.as_str()) {
                let next = text::slice(&low, n as isize, Some(n as isize + 1));
                if ["", " ", ",", ";", ":", "-", "!", "?", "."].contains(&next.as_str()) {
                    let rest = text::slice(&text_, n as isize, None);
                    text_ = text::lstrip_chars(&rest, " ,;:-!?.").to_string();
                    changed = true;
                    break;
                }
            }
        }
        if !changed {
            let done = _after_a_reaction(&text_);
            if done != text_ {
                text_ = done;
                changed = true;
            }
        }
    }
    text_
}

/// Python: bot/memory/ack.py:6600 `without_the_frame(clean)`
///
/// "sorry, i mean saturday" with the apology and the repair taken off: "saturday".
pub fn without_the_frame(clean: &str) -> Value {
    Value::from(_strip_lead_in(&squash(clean), false))
}

/// Python: bot/memory/ack.py:6632 `_spoken(said)`
///
/// `said` with the chat-speak folded, so the echo does not read "im" or "dont" or "yrs".
pub fn _spoken(said: &str) -> String {
    let said = COUNTED_STARS.sub_fn(said, |m| {
        let n: i64 = m.name_str("count").parse().unwrap_or(0);
        STAR_COUNT.get(&n).cloned().expect("KeyError: STAR_COUNT")
    });
    let said = ONE_STAR.sub_fn(&said, |_| "an A*".to_string());
    let said = COUNTED_LETTERS.sub_fn(&said, |m| {
        let n: i64 = m.name_str("count").parse().unwrap_or(0);
        format!("{} {}s", LETTER_COUNT.get(&n).cloned().expect("KeyError: LETTER_COUNT"), text::upper(m.name_str("letter")))
    });
    let said = if A_DEGREE.is_match(&said) {
        DEGREE_CLASS.sub_fn(&said, |m| format!("{}:{}", m.name_str("first"), m.name_str("second")))
    } else {
        said
    };
    let said = WITH_SLASH.sub_fn(&said, |_| "with ".to_string());
    let said = TWO_TO.sub_fn(&said, |m| {
        let whole = m.as_str();
        format!("{}to", &whole[..whole.len() - 1])
    });
    let said = TWO_PLACE.sub_fn(&said, |_| "to ".to_string());
    let said = TWO_VERB.sub_fn(&said, |_| "to ".to_string());
    let words = text::split_ref(&said);
    let mut out: Vec<String> = Vec::new();
    for (place, word) in words.iter().enumerate() {
        let bare = text::lower(text::strip_chars(word, ".,;:!?"));
        let tail = &word[text::rstrip_chars(word, ".,;:!?").len()..];
        let after = words.get(place + 1).map(|w| text::lower(text::strip_chars(w, ".,;:!?"))).unwrap_or_default();
        let before = if place > 0 { text::lower(text::strip_chars(words[place - 1], ".,;:!?")) } else { String::new() };
        if bare == "yer" {
            let fold = if after.is_empty() || YES_AFTER.contains(&after) { "yeah" } else { "your" };
            out.push(format!("{fold}{tail}"));
        } else if bare == "its" && ITS_OWNS_AFTER.contains(&before) {
            out.push(word.to_string()); // the possessive: "got its screen"
        } else if let Some(spoken) = SPEAK.get(&bare) {
            out.push(format!("{spoken}{tail}"));
        } else if let Some(short) = SHORT.get(&bare) {
            out.push(format!("{short}{tail}"));
        } else {
            out.push(word.to_string());
        }
    }
    out.join(" ")
}

/// Python: bot/memory/ack.py:6711 `_pick_off(said)`
///
/// `said` without the words that say a choice was made: "make it a dog" -> "a dog".
pub fn _pick_off(said: &str) -> String {
    let Some(found) = PICK_FRAME.match_(said) else { return said.to_string() };
    let rest = text::strip_chars(&said[found.end()..], " ,;:-.!?\u{2014}");
    if n_words(rest) < 1 {
        return said.to_string();
    }
    rest.to_string()
}

/// Python: bot/memory/ack.py:6727 `_tail_off(said)`
///
/// `said` without the chat-speak hanging off the end, so the echo is the fact and not the filler.
pub fn _tail_off(said: &str) -> String {
    let said = crate::memory::filler::without_filler(said);
    let mut words: Vec<&str> = text::split_ref(&said);
    while !words.is_empty() {
        let low = text::strip_chars(&text::lower(&words.join(" ")), " ,;:-").to_string();
        let cut = FILLER_TAIL_PHRASES.iter().find(|phrase| low.ends_with(phrase.as_str()));
        let Some(cut) = cut else { break };
        let n = n_words(cut);
        if words.len() <= n {
            break;
        }
        words.truncate(words.len() - n);
    }
    text::strip_chars(&words.join(" "), " ,;:-").to_string()
}

/// Python: bot/memory/ack.py:6744 `_contrast_off(said)`
///
/// `said` without the wrong value it replaced: "my budget is 400 a month not 300" -> "my budget
/// is 400 a month".
pub fn _contrast_off(said: &str) -> String {
    if let Some(again) = CONTRAST_SAID_AGAIN.match_(text::strip(said)) {
        if _a_value(again.name_str("old")) {
            return format!("{} {}", again.name_str("thing"), again.name_str("new"));
        }
    }
    let Some(found) = CONTRAST.search(said) else { return said.to_string() };
    let old_text = found.name_str("old").to_string();
    let start = found.start();
    let mut old: Vec<String> = RE_SPACES.split(text::strip_chars(&old_text, " .,;:-")).into_iter().filter(|w| !w.is_empty()).collect();
    if old.first().is_some_and(|w| ["the", "a", "an"].contains(&text::lower(w).as_str())) {
        old.remove(0); // "not the 5th" corrects a day
    }
    if old.is_empty() || !RE_ALNUM_ANY.is_match(&old[0]) {
        return said.to_string();
    }
    let kept: Vec<String> = RE_SPACES.split(text::strip_chars(&said[..start], " .,;:-")).into_iter().filter(|w| !w.is_empty()).collect();
    if NOT_A_VALUE.contains(&text::lower(&old[0])) {
        return said.to_string();
    }
    if !(_a_value(&old[0]) || kept.last().is_some_and(|k| _a_value(k))) {
        return said.to_string();
    }
    let restated = _the_value_put_right(said, start, &old_text);
    if !restated.is_empty() {
        return restated;
    }
    said[..start].to_string()
}

/// Python: bot/memory/ack.py:6784 `_reason_off(said, clean)`
///
/// `said` without the reason a correction gave for itself: "you mean 75 because I lost some".
pub fn _reason_off(said: &str, clean: &str) -> String {
    if A_CORRECTION_IN_FRONT.match_(clean).is_none() {
        return said.to_string();
    }
    match A_REASON_TAIL.search(said) {
        Some(found) if !text::strip_chars(&said[..found.start()], " ,;:-").is_empty() => said[..found.start()].to_string(),
        _ => said.to_string(),
    }
}

/// Python: bot/memory/ack.py:6810 `_the_value_put_right(said, match)`
///
/// The line with the value a restated correction puts right, or "" for any other shape. The
/// match is the `CONTRAST` match: where it starts in `said`, and its "old" group.
pub fn _the_value_put_right(said: &str, start: usize, old: &str) -> String {
    let head = text::strip_chars(&squash(&said[..start]), " ,;:-\u{2014}").to_string();
    if head.is_empty() {
        return String::new();
    }
    if RE_SPACES.split(&head).iter().any(|w| _a_value(w)) {
        return String::new(); // the value on this side is the one said back
    }
    let words: Vec<String> = RE_SPACES.split(text::strip_chars(old, " .,;:-")).into_iter().filter(|w| !w.is_empty()).collect();
    if words.len() < 3 {
        return String::new();
    }
    let Some(second) = (1..words.len()).find(|&at| _a_value(&words[at])) else { return String::new() };
    let between: Vec<String> = words[1..second].iter().map(|w| text::lower(text::strip_chars(w, ".,!?'\u{2019}"))).collect();
    if between.is_empty() || between.iter().any(|w| !BETWEEN_VALUES.contains(w)) {
        return String::new();
    }
    format!("{} {}", head, words[second..].join(" "))
}

/// Python: bot/memory/ack.py:6873 `_a_value(word)`
///
/// Whether this word is a value a correction can replace ("300", "june", "£40").
pub fn _a_value(word: &str) -> bool {
    let low = text::lower(text::strip_chars(word, " .,;:!?\u{2019}'"));
    if low.is_empty() {
        return false;
    }
    if VALUE.match_(&low).is_some() {
        return true;
    }
    DAYS.contains(&low) || MONTHS.contains(&low)
}

/// Python: bot/memory/ack.py:6883 `_cut(said)`
///
/// `said` down to MOST characters at a word boundary, so an echo stays an echo.
pub fn _cut(said: &str) -> String {
    if text::len(said) <= *MOST {
        return said.to_string();
    }
    let head = text::slice(said, 0, Some(*MOST as isize));
    let kept = match head.rfind(' ') {
        Some(at) => text::rstrip_chars(&head[..at], " ,;:-").to_string(),
        None => text::rstrip_chars(&head, " ,;:-").to_string(),
    };
    if kept.contains(' ') { kept } else { head }
}

/// Python: bot/memory/ack.py:6905 `_tidy(text)`
///
/// `text` as one sentence piece: the second person in lower case, and nothing else changed.
pub fn _tidy(text_: &str) -> String {
    let out: Vec<String> = text::split_ref(text_)
        .iter()
        .enumerate()
        .map(|(index, word)| {
            let low = text::lower(word);
            if ["you", "your", "you're", "you've", "we", "our", "yourself"].contains(&low.as_str()) {
                low
            } else if index == 0 && FIRST_WORD.contains(&low) {
                low
            } else {
                word.to_string()
            }
        })
        .collect();
    out.join(" ")
}

data_set!(FIRST_WORD);

#[cfg(test)]
mod tests {
    use super::*;

    fn chat(lines: &[(&str, &str, &str)]) -> Memory {
        let mut memory = Memory::new();
        memory.turns = lines
            .iter()
            .map(|(user, bot, module)| Turn { user: user.to_string(), bot: bot.to_string(), module: Some(module.to_string()) })
            .collect();
        memory
    }

    /// Expected values from PyBot (`harness/pycall.py`).
    #[test]
    fn readers_of_a_line() {
        let birthday = chat(&[("my mums birthday is in may", "Noted.", "memory_store")]);
        assert_eq!(a_second_person("and my dads", &birthday), "And your dad's birthday \u{2014} when is that?");
        let wedding = chat(&[("my brother is getting married in june", "Noted.", "memory_store")]);
        assert_eq!(a_second_person("and my sister", &wedding), "And your sister \u{2014} when's she getting?");
        let gym = chat(&[("i go to the gym three times a week", "ok", "chat")]);
        assert_eq!(_the_unit_before(&gym), "times a week");
        assert_eq!(_the_bare_value("sorry 1150, i got it wrong"), "1150");
        assert_eq!(_the_bare_value("1150, not 1200"), "");
        assert_eq!(_strip_lead_in("ok so, no wait, its tuesday", false), "its tuesday");
        assert_eq!(_contrast_off("hes not 4 hes 5"), "hes 5");
        assert_eq!(_contrast_off("my budget is 400 a month not 300"), "my budget is 400 a month");
        assert_eq!(_contrast_off("my son is not 4 hes 5"), "my son is 5");
        assert_eq!(
            _spoken("im 2 dogs n my son got 3 a stars and 2 as, a 2 1 in his degree, going 2 spain w/ the kids"),
            "i'm two dogs and my son got three A*s and two As, a 2:1 in his degree, going to spain with the kids"
        );
        assert_eq!(_a_day_coming("the vets at 4 tomorrow and we are going"), Some(SOON_TAIL.clone()));
    }
}
