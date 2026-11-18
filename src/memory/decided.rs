//! Port of `bot/memory/decided.py`: what they settled on ("actually a dog", "lets go with
//! spain"), kept as a decision of its own, and the ask that wants it back ("what did i decide
//! to get").

use super::pyv;
use crate::data::{self, Value};
use crate::py::re::{self, Re};
use crate::py::text;
use crate::state::Memory;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.memory.decided.";
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn rxs(name: &str) -> Vec<&'static Re> {
    data::get(&format!("{M}{name}")).items().iter().map(|v| v.as_re().unwrap()).collect()
}
fn words_set(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}

static LEAD_IN: LazyLock<&'static Re> = LazyLock::new(|| rx("LEAD_IN"));
static SETTLING: LazyLock<Vec<&'static Re>> = LazyLock::new(|| rxs("SETTLING"));
static GOING: LazyLock<Vec<&'static Re>> = LazyLock::new(|| rxs("GOING"));
static ACTIVITY: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("ACTIVITY"));
static CORRECTING_LINE: LazyLock<Vec<&'static Re>> = LazyLock::new(|| rxs("CORRECTING_LINE"));
static CHOOSING: LazyLock<Vec<&'static Re>> = LazyLock::new(|| rxs("CHOOSING"));
static OPTIONS: LazyLock<&'static Re> = LazyLock::new(|| rx("OPTIONS"));
static NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("NOT_A_THING"));
static CLAUSE_CHOICE: LazyLock<&'static Re> = LazyLock::new(|| rx("CLAUSE_CHOICE"));
static CORRECTING: LazyLock<&'static Re> = LazyLock::new(|| rx("CORRECTING"));
static TAIL: LazyLock<&'static Re> = LazyLock::new(|| rx("TAIL"));
static IDIOM: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("IDIOM"));
static MOST_WORDS: LazyLock<usize> = LazyLock::new(|| data::get("bot.memory.decided.MOST_WORDS").as_int().unwrap() as usize);
static ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("ASK"));
static GUESSING_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("GUESSING_ASK"));
static TO_YOU: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.memory.decided.TO_YOU"));
static INNER_DECIDE: LazyLock<&'static Re> = LazyLock::new(|| rx("INNER_DECIDE"));
static OPTION_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("OPTION_FILLER"));
static REACHES_FOR_A_CHOICE: LazyLock<&'static Re> = LazyLock::new(|| rx("REACHES_FOR_A_CHOICE"));
static OPENING_QUESTION: LazyLock<&'static Re> = LazyLock::new(|| rx("OPENING_QUESTION"));
static DECIDING_WORD: LazyLock<&'static Re> = LazyLock::new(|| rx("DECIDING_WORD"));
static ASK_FRAME: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("ASK_FRAME"));
static NOT_A_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("NOT_A_SUBJECT"));
static DOING: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("DOING"));
static COMPANION: LazyLock<&'static Re> = LazyLock::new(|| rx("COMPANION"));

static A_FIGURE: LazyLock<Re> = LazyLock::new(|| Re::new(r"[\d.,]+", re::U));

const EDGE: &str = " ,.;:!?";
const PUNCTUATION: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

/// `pattern.sub("", phrase).strip(EDGE)` until nothing changes.
fn strip_all(phrase: String, pattern: &Re) -> String {
    let mut phrase = phrase;
    loop {
        let trimmed = text::strip_chars(&pattern.sub("", &phrase), EDGE).to_string();
        if trimmed == phrase {
            return phrase;
        }
        phrase = trimmed;
    }
}

/// The lines they said in the last `n` turns, newest first.
fn users_back(memory: &Memory, n: usize) -> impl Iterator<Item = &str> {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(n)..].iter().rev().map(|turn| turn.user.as_str())
}

/// Python: bot/memory/decided.py:174 `_the_thing(phrase)`
/// The chosen thing as the store should keep it, or None when the phrase is not one.
pub fn _the_thing(phrase: &str) -> Option<String> {
    let phrase = text::strip_chars(&text::squash(phrase), EDGE).to_string();
    let phrase = strip_all(phrase, &CORRECTING);
    let phrase = strip_all(phrase, &INNER_DECIDE);
    let phrase = strip_all(phrase, &TAIL);
    if IDIOM.contains(&phrase.to_lowercase()) {
        return None;
    }
    if A_FIGURE.fullmatch(&phrase).is_some() {
        return None;
    }
    let words = text::split_ref(&phrase);
    let verb_phrase = words.len() > 1 && words[0].to_lowercase() == "to";
    if words.is_empty() || words.len() > if verb_phrase { *MOST_WORDS + 4 } else { *MOST_WORDS } {
        return None;
    }
    if NOT_A_THING.contains(&words[0].to_lowercase()) {
        return None;
    }
    if words[1..].iter().any(|w| NOT_A_THING.contains(&w.to_lowercase())) {
        return None;
    }
    Some(phrase)
}

/// Python: bot/memory/decided.py:213 `_two_things(said)`
/// The two things a line put side by side, as "X or Y", or None.
pub fn _two_things(said: &str) -> Option<String> {
    let flat = text::squash(&said.to_lowercase());
    let low = text::strip_chars(&flat, " ?.!,");
    let (left, right) = if low.contains(" between ") && low.split_once(" between ").unwrap().1.contains(" and ") {
        let head = low.split_once(" between ").unwrap().1;
        head.split_once(" and ").unwrap()
    } else if let Some(cut) = low.rfind(" or ") {
        (&low[..cut], &low[cut + 4..])
    } else {
        return None;
    };
    let mut left_words = text::split_ref(left);
    while left_words.first().is_some_and(|w| OPTION_FILLER.contains(text::strip_chars(w, ",;:"))) {
        left_words.remove(0);
    }
    let mut right_words: Vec<&str> = text::split_ref(right).into_iter().take(3).collect();
    while right_words.last().is_some_and(|w| OPTION_FILLER.contains(text::strip_chars(w, ",;:"))) {
        right_words.pop();
    }
    let left = text::strip_chars(&left_words[left_words.len().saturating_sub(3)..].join(" "), " ,;:").to_string();
    let right = text::strip_chars(&right_words.join(" "), " ,;:").to_string();
    if left.is_empty() || right.is_empty() {
        return None;
    }
    if IDIOM.contains(&left) || IDIOM.contains(&right) || NOT_A_THING.contains(&left) || NOT_A_THING.contains(&right) {
        return None;
    }
    Some(format!("{left} or {right}"))
}

/// Python: bot/memory/decided.py:243 `_options_since(memory, line)`
/// The pair put on the table after the choice on the book was made, or None.
pub fn _options_since(memory: &Memory, line: &str) -> Option<String> {
    let key = text::squash(&line.to_lowercase());
    if key.is_empty() {
        return None;
    }
    for said in users_back(memory, 6) {
        let said = text::squash(said);
        if said.is_empty() {
            continue;
        }
        if said.to_lowercase() == key {
            break;
        }
        if asks(&said) || !OPTIONS.is_match(&said) {
            continue;
        }
        if let Some(pair) = _two_things(&said) {
            return Some(pair);
        }
    }
    None
}

/// Python: bot/memory/decided.py:263 `_the_pair_left_open(memory)`
/// "italy or spain" for the last line that put two things in front of them, or None.
pub fn _the_pair_left_open(memory: &Memory) -> Option<String> {
    for said in users_back(memory, 3) {
        if said.is_empty() || asks(said) || !OPTIONS.is_match(said) {
            continue;
        }
        if let Some(pair) = _two_things(said) {
            return Some(pair);
        }
    }
    None
}

/// Python: bot/memory/decided.py:274 `_as_typed(text, thing)`
/// The thing with the case the person typed it in.
pub fn _as_typed(text_: &str, thing: &str) -> String {
    let words = text::split_ref(thing);
    if words.is_empty() {
        return thing.to_string();
    }
    let pattern = format!(r"\b{}\b", words.iter().map(|w| re::escape(w)).collect::<Vec<_>>().join(r"\s+"));
    Re::new(&pattern, re::I | re::U).search(text_).map(|m| m.as_str().to_string()).unwrap_or_else(|| thing.to_string())
}

/// What `read` finds in a line: the thing, whether the line settles the choice by itself, and
/// whether it is a correction ("no, the blue one").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    pub what: String,
    pub settled: bool,
    pub corrects: bool,
}

/// Python: bot/memory/decided.py:283 `read(text)`
/// The choice a line makes, or None.
pub fn read(text_: &str) -> Option<Reading> {
    let mut line = text::squash(&text_.to_lowercase());
    if line.is_empty() || line.contains('?') || text::split_ref(&line).len() > 14 {
        return None;
    }
    if asks(&line) {
        return None;
    }
    loop {
        let trimmed = LEAD_IN.sub("", &line);
        if trimmed == line {
            break;
        }
        line = trimmed;
    }
    let reading = |what: String, settled: bool, corrects: bool| Reading { what: _as_typed(text_, &what), settled, corrects };
    if let Some(clause) = CLAUSE_CHOICE.search(&line) {
        if let Some(thing) = _the_thing(clause.name_str("what")).filter(|t| !_an_activity(t)) {
            return Some(reading(thing, true, false));
        }
    }
    for pattern in CORRECTING_LINE.iter() {
        if let Some(found) = pattern.search(&line) {
            if let Some(thing) = _the_thing(found.name_str("what")) {
                return Some(reading(thing, false, true));
            }
        }
    }
    for pattern in GOING.iter() {
        let Some(found) = pattern.search(&line) else { continue };
        if let Some(thing) = _the_thing(found.name_str("what")).filter(|t| !_an_activity(t)) {
            return Some(reading(thing, true, false));
        }
    }
    for (patterns, settled) in [(&*SETTLING, true), (&*CHOOSING, false)] {
        for pattern in patterns.iter() {
            let Some(found) = pattern.search(&line) else { continue };
            if let Some(thing) = _the_thing(found.name_str("what")) {
                return Some(reading(thing, settled, false));
            }
        }
    }
    None
}

/// Python: bot/memory/decided.py:331 `extract(text)`
/// The thing a choosing line settled on, or None.
pub fn extract(text_: &str) -> Option<String> {
    read(text_).map(|reading| reading.what)
}

/// Python: bot/memory/decided.py:341 `_an_activity(thing)`
/// True when the thing after a going verb is something a person does, not something chosen.
pub fn _an_activity(thing: &str) -> bool {
    let mut words: Vec<String> = text::split_ref(thing).iter().map(|w| text::strip_chars(w, EDGE).to_lowercase()).collect();
    while words.first().is_some_and(|w| matches!(w.as_str(), "a" | "an" | "the" | "some" | "my" | "our")) {
        words.remove(0);
    }
    words.first().is_some_and(|w| ACTIVITY.contains(w))
}

/// Python: bot/memory/decided.py:349 `_a_bare_thing(said)`
/// True when a line is one short thing and nothing else: "cat", "the blue one".
pub fn _a_bare_thing(said: &str) -> bool {
    let flat = text::squash(&said.to_lowercase());
    let line = text::strip_chars(&flat, EDGE);
    if line.is_empty() || line.contains('?') {
        return false;
    }
    let words = text::split_ref(line);
    if words.len() > 2 || words.iter().any(|w| NOT_A_THING.contains(*w)) {
        return false;
    }
    words.iter().all(|w| text::is_alpha(w)) && text::len(words[words.len() - 1]) > 2
}

/// Python: bot/memory/decided.py:359 `_options_on_the_table(memory)`
/// Whether the last thing said put a choice in front of them (the two turns before).
pub fn _options_on_the_table(memory: &Memory) -> bool {
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(2)..].iter().rev() {
        let said = turn.user.as_str();
        if !turn.bot.is_empty() && said.is_empty() {
            continue;
        }
        if _a_bare_thing(said) {
            return true;
        }
        if asks(said) {
            continue;
        }
        if OPTIONS.is_match(said) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/decided.py:399 `_a_choice_reached_for(memory, kept_line=None)`
/// Whether the last thing they said is a choice this reader could not name. `kept_line` "" is
/// None.
pub fn _a_choice_reached_for(memory: &Memory, kept_line: &str) -> bool {
    for said in users_back(memory, 3) {
        let said = text::squash(said);
        if said.is_empty() || asks(&said) {
            continue;
        }
        if said.contains('?') || OPENING_QUESTION.match_(&said).is_some() {
            continue;
        }
        if !kept_line.is_empty() && said.to_lowercase() == kept_line.to_lowercase() {
            return false;
        }
        return REACHES_FOR_A_CHOICE.is_match(&said);
    }
    false
}

/// Python: bot/memory/decided.py:425 `_a_plan_since(memory)`
/// Whether the last thing they said was a plan still going: "im going for a run".
pub fn _a_plan_since(memory: &Memory) -> bool {
    for said in users_back(memory, 3) {
        let said = text::squash(said);
        if said.is_empty() || asks(&said) || said.contains('?') {
            continue;
        }
        let low = said.to_lowercase();
        for pattern in GOING.iter() {
            if let Some(found) = pattern.search(&low) {
                return _an_activity(&_the_thing(found.name_str("what")).unwrap_or_default());
            }
        }
        return false;
    }
    false
}

/// Python: bot/memory/decided.py:442 `observe(text, memory)`
/// Keep the thing they settled on (`memory["decided"]`); a later choice is the one they settled on.
pub fn observe(text_: &str, memory: &mut Memory) -> Option<Value> {
    let reading = read(text_)?;
    if reading.corrects && !(the_decision(memory).is_some() || _options_on_the_table(memory)) {
        return None;
    }
    if !reading.settled && !reading.corrects && !_options_on_the_table(memory) {
        return None;
    }
    let mut decided = Value::dict();
    decided.set_key("what", reading.what.as_str());
    decided.set_key("line", text::squash(&text_.to_lowercase()));
    memory.set("decided", decided.clone());
    Some(decided)
}

/// Python: bot/memory/decided.py:463 `_the_plan_on_record(memory)`
/// The plan the chat has already said back to them, latest first, or None.
pub fn _the_plan_on_record(memory: &Memory) -> Option<String> {
    let user = memory.get("user").filter(|u| u.is_dict())?;
    let plan = pyv::get(user, "plan");
    let mut plans: Vec<Value> = if plan.truthy() { vec![plan.clone()] } else { Vec::new() };
    if let Value::List(held) = pyv::get(user, "plans") {
        let mut all = held.to_vec();
        all.extend(plans);
        plans = all;
    }
    for plan in plans.iter().rev().filter(|p| p.truthy()) {
        let plan = text::squash(&pyv::str_or_empty(plan));
        let words: Vec<String> = text::split(&plan.to_lowercase().replace(',', " ")).into_iter().filter(|w| text::len(w) > 2).collect();
        if !words.is_empty() && said_back(memory, &words) {
            return Some(plan);
        }
    }
    None
}

/// Python: bot/memory/decided.py:484 `said_back(memory, words)`
/// Whether an earlier reply of the bot already said these words.
pub fn said_back(memory: &Memory, words: &[String]) -> bool {
    memory.turns.iter().any(|turn| {
        let said = turn.bot.to_lowercase();
        !said.is_empty() && words.iter().all(|w| said.contains(w.as_str()))
    })
}

/// Python: bot/memory/decided.py:492 `the_decision(memory)`
/// The choice on record, or None.
pub fn the_decision(memory: &Memory) -> Option<Value> {
    memory.get("decided").filter(|held| held.is_dict() && !text::strip(&pyv::str_or_empty(pyv::get(held, "what"))).is_empty()).cloned()
}

/// Python: bot/memory/decided.py:500 `asks(text)`
/// Whether the line is asking back what the chat settled on.
pub fn asks(text_: &str) -> bool {
    ASK.is_match(&text::squash(&text_.to_lowercase()))
}

/// Python: bot/memory/decided.py:506 `the_guess(text)`
/// The thing an ask expects to hear back ("did we settle on spain"), or None.
pub fn the_guess(text_: &str) -> Option<String> {
    let flat = text::squash(&text_.to_lowercase());
    let low = text::strip_chars(&flat, " ?.!,");
    let found = GUESSING_ASK.search(low)?;
    _the_thing(found.name_str("what"))
}

/// Python: bot/memory/decided.py:515 `_the_same_thing(said, held)`
/// Whether the thing guessed is the thing on record: "spain" is "spain in may".
pub fn _the_same_thing(said: &str, held: &str) -> bool {
    let one = text::strip_chars(&text::squash(&said.to_lowercase()), " ?.!,").to_string();
    let other = text::strip_chars(&text::squash(&held.to_lowercase()), " ?.!,").to_string();
    if one.is_empty() || other.is_empty() {
        return false;
    }
    let one = one.strip_prefix("to ").unwrap_or(&one).to_string();
    let other = other.strip_prefix("to ").unwrap_or(&other).to_string();
    if one == other || other.contains(&one) || one.contains(&other) {
        return true;
    }
    let (first, second) = (text::split_ref(&one), text::split_ref(&other));
    if first.len() > 2 && second.len() > 2 && first[1..] == second[1..] {
        let (verb_one, verb_two) = (first[0], second[0]);
        return text::len(verb_one) > 2 && text::len(verb_two) > 2 && text::slice(verb_one, 0, Some(3)) == text::slice(verb_two, 0, Some(3));
    }
    false
}

/// Python: bot/memory/decided.py:537 `_said_to_you(thing)`
/// The thing in second person, in the case they typed it: "fixing mine" -> "fixing yours".
pub fn _said_to_you(thing: &str) -> String {
    text::split_ref(thing)
        .into_iter()
        .map(|word| TO_YOU.get(&text::strip_chars(word, ".,!?;:").to_lowercase()).cloned().unwrap_or_else(|| word.to_string()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Python: bot/memory/decided.py:546 `_a_verb_phrase(thing)`
pub fn _a_verb_phrase(thing: &str) -> bool {
    let words = text::split_ref(thing);
    words.len() > 1 && words[0].to_lowercase() == "to"
}

/// Python: bot/memory/decided.py:552 `_the_choice_said(thing)`
pub fn _the_choice_said(thing: &str) -> String {
    if _a_verb_phrase(thing) { format!("You decided {thing}.") } else { format!("You went with {thing}.") }
}

/// Python: bot/memory/decided.py:559 `_at_the_start(thing)`
pub fn _at_the_start(thing: &str) -> String {
    text::upper_first(thing)
}

/// Python: bot/memory/decided.py:589 `_asks_about_another_thing(text, held)`
/// True when the ask names a thing the choice on record is not about.
pub fn _asks_about_another_thing(text_: &str, held: Option<&Value>) -> bool {
    let said = text::squash(&text_.to_lowercase());
    let held_text = held.map(|h| text::squash(&pyv::str_or_empty(pyv::get(h, "what")).to_lowercase())).unwrap_or_default();
    if COMPANION.is_match(&said) {
        return true;
    }
    let named: Vec<String> = text::split_ref(&said)
        .iter()
        .map(|w| text::strip_chars(w, PUNCTUATION).to_string())
        .filter(|w| {
            !w.is_empty() && !ASK_FRAME.contains(w) && !NOT_A_SUBJECT.contains(w) && !DOING.contains(w) && text::len(w) > 2 && !text::is_digit(w)
        })
        .collect();
    if named.is_empty() {
        return false;
    }
    !named.iter().any(|w| held_text.contains(w.as_str()) || (text::len(w) > 3 && held_text.starts_with(&text::slice(w, 0, Some(4)))))
}

/// `pick(options, memory, seed_text=text)` on a list of lines.
fn pick(options: &[String], memory: &Memory, text_: &str) -> String {
    pyv::pick(options, memory, text_)
}

/// Python: bot/memory/decided.py:612 `answer(text, memory)`
/// The reply to an ask that wants the choice back, or None when the line is not one.
pub fn answer(text_: &str, memory: &Memory) -> Option<String> {
    if !asks(text_) {
        return None;
    }
    let held = the_decision(memory);
    let held_line = held.as_ref().map(|h| pyv::str_or_empty(pyv::get(h, "line"))).unwrap_or_default();
    let lines = |frames: &[&str], tail: &str| -> Vec<String> { frames.iter().map(|f| format!("{f}{tail}")).collect() };
    if held.is_some() {
        if let Some(moved_on) = _options_since(memory, &held_line) {
            if !_a_choice_reached_for(memory, &held_line) {
                let tail = format!("{moved_on}.");
                return Some(pick(&lines(&["Nothing settled yet: ", "Not yet: ", "Neither one yet: "], &tail), memory, text_));
            }
        }
    }
    if held.is_some() && the_guess(text_).is_none() && _asks_about_another_thing(text_, held.as_ref()) {
        return None;
    }
    if held.is_some() && _a_choice_reached_for(memory, &held_line) {
        return None;
    }
    let Some(held) = held else {
        if _asks_about_another_thing(text_, None) && the_guess(text_).is_none() {
            return None;
        }
        if let Some(plan) = _the_plan_on_record(memory) {
            let mut as_held = Value::dict();
            as_held.set_key("what", plan.as_str());
            if DECIDING_WORD.is_match(text_) && !_asks_about_another_thing(text_, Some(&as_held)) {
                let tail = format!("{plan}.");
                return Some(pick(&lines(&["You settled on ", "That's settled: ", "The chat settled it: "], &tail), memory, text_));
            }
        }
        if _a_choice_reached_for(memory, "") && !(DECIDING_WORD.is_match(text_) && _a_plan_since(memory)) {
            return None;
        }
        if let Some(pair) = _the_pair_left_open(memory) {
            let tail = format!("{pair}.");
            return Some(pick(&lines(&["Nothing settled yet: ", "Not yet: ", "Neither one yet: "], &tail), memory, text_));
        }
        return Some(pick(&lines(&["Nothing settled yet.", "You haven't picked one yet.", "You haven't settled on one yet."], ""), memory, text_));
    };
    let what = pyv::str_or_empty(pyv::get(&held, "what"));
    let thing = _said_to_you(&what);
    let guess = the_guess(text_);
    if _a_verb_phrase(&what) {
        if let Some(guess) = guess {
            if _the_same_thing(&guess, &what) {
                return Some(pick(&[format!("That's right: you decided {thing}."), format!("Yes - you decided {thing}.")], memory, text_));
            }
            return Some(pick(&[format!("Not quite: you decided {thing}."), format!("The other one: you decided {thing}.")], memory, text_));
        }
        return Some(pick(&[format!("You decided {thing}."), format!("You decided {thing}, in the end.")], memory, text_));
    }
    if let Some(guess) = guess {
        if _the_same_thing(&guess, &what) {
            return Some(pick(&[format!("Yes - {thing}, in the end."), format!("That's right: {thing}."), format!("Yes, {thing}.")], memory, text_));
        }
        return Some(pick(&[format!("No - {thing}, in the end."), format!("Not quite: {thing}, in the end."), format!("The other one: {thing}.")], memory, text_));
    }
    Some(pick(&[format!("{}, in the end.", _at_the_start(&thing)), format!("You settled on {thing}."), _the_choice_said(&thing)], memory, text_))
}
