//! Port of `bot/memory/counts.py`: how many of a thing the person has ("i have three
//! brothers", "there are 4 of us", "a roast for 6"), kept as a number under
//! `memory["user"]["how_many"]` and given back when they ask.

use super::pyv;
use crate::data::{self, Value};
use crate::py::re::{self, Match, Re};
use crate::py::text;
use crate::state::Memory;
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.memory.counts.";
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn words_set(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}

static NUMBER_AHEAD: LazyLock<&'static Re> = LazyLock::new(|| rx("NUMBER_AHEAD"));
static THEY_HAVE: LazyLock<&'static Re> = LazyLock::new(|| rx("THEY_HAVE"));
static MY_THREE: LazyLock<&'static Re> = LazyLock::new(|| rx("MY_THREE"));
static CORRECTED: LazyLock<&'static Re> = LazyLock::new(|| rx("CORRECTED"));
static ASKING: LazyLock<&'static Re> = LazyLock::new(|| rx("ASKING"));
static IRREGULAR: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.memory.counts.IRREGULAR").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
});
static THEIRS: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("THEIRS"));
static NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("NOT_A_THING"));
static GROUP_OF_US: LazyLock<&'static Re> = LazyLock::new(|| rx("GROUP_OF_US"));
static GROUP_CORRECTED: LazyLock<&'static Re> = LazyLock::new(|| rx("GROUP_CORRECTED"));
static GROUP_OF_FOUR: LazyLock<&'static Re> = LazyLock::new(|| rx("GROUP_OF_FOUR"));
static GROUP_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("GROUP_ASK"));
static GROUP_ASK_YOU: LazyLock<&'static Re> = LazyLock::new(|| rx("GROUP_ASK_YOU"));
static BARE_GROUP_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("BARE_GROUP_ASK"));
static BARE_COUNT_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("BARE_COUNT_ASK"));
static GROUP: LazyLock<String> = LazyLock::new(|| data::string("bot.memory.counts.GROUP"));
static PARTY: LazyLock<&'static Re> = LazyLock::new(|| rx("PARTY"));
static PARTY_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("PARTY_ASK"));
static CHANGED: LazyLock<&'static Re> = LazyLock::new(|| rx("CHANGED"));
static CHANGED_BARE: LazyLock<&'static Re> = LazyLock::new(|| rx("CHANGED_BARE"));
static NOW_THE_NUMBER: LazyLock<&'static Re> = LazyLock::new(|| rx("NOW_THE_NUMBER"));
static ONE_MORE: LazyLock<&'static Re> = LazyLock::new(|| rx("ONE_MORE"));
static ONE_LESS: LazyLock<&'static Re> = LazyLock::new(|| rx("ONE_LESS"));
static ABOUT_THE_HOUSE: LazyLock<&'static Re> = LazyLock::new(|| rx("ABOUT_THE_HOUSE"));
static POINTS_AT_A_NUMBER: LazyLock<&'static Re> = LazyLock::new(|| rx("POINTS_AT_A_NUMBER"));
static PARTY_THING: LazyLock<String> = LazyLock::new(|| data::string("bot.memory.counts.PARTY_THING"));
static LOOK_BACK_FOR_A_MEAL: LazyLock<usize> =
    LazyLock::new(|| data::get("bot.memory.counts.LOOK_BACK_FOR_A_MEAL").as_int().unwrap() as usize);
static BARE_PEOPLE: LazyLock<&'static Re> = LazyLock::new(|| rx("BARE_PEOPLE"));
static A_MEAL_IN_THE_CHAT: LazyLock<&'static Re> = LazyLock::new(|| rx("A_MEAL_IN_THE_CHAT"));
static PHRASE_ENDS: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("PHRASE_ENDS"));
static CHAT_JOINERS: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("CHAT_JOINERS"));
static MAX_PHRASE: LazyLock<usize> = LazyLock::new(|| data::get("bot.memory.counts.MAX_PHRASE").as_int().unwrap() as usize);
static VERB_IN_PHRASE: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("VERB_IN_PHRASE"));
static RELATION_AHEAD: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("RELATION_AHEAD"));
static A_PERSON_TAIL: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.counts.A_PERSON_TAIL"));
static BEDS: LazyLock<&'static Re> = LazyLock::new(|| rx("BEDS"));
static A_KIND_OF_PLACE: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("A_KIND_OF_PLACE"));
static A_MEASURE_WORD: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("A_MEASURE_WORD"));
static NOT_A_COUNT: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("NOT_A_COUNT"));
static A_RANGE_AROUND: LazyLock<&'static Re> = LazyLock::new(|| rx("A_RANGE_AROUND"));
static BIGGER_NUMBERS: LazyLock<HashMap<String, i64>> = LazyLock::new(|| {
    data::get("bot.memory.counts._BIGGER_NUMBERS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap())).collect()
});
static PET_COUNT_WORDS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| {
    data::get("bot.memory.facts.PET_COUNT_WORDS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0))).collect()
});
static A_TABLE_BOOKED: LazyLock<&'static Re> = LazyLock::new(|| rx("A_TABLE_BOOKED"));
static A_TABLE_CUE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_TABLE_CUE"));
static A_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_FIGURE"));
static SAID_AS_A_CHANGE: LazyLock<&'static Re> = LazyLock::new(|| rx("SAID_AS_A_CHANGE"));
static POINTS_AT_THE_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("POINTS_AT_THE_THING"));
static A_LABEL_IN_FRONT: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("A_LABEL_IN_FRONT"));
static COUNTS_THE_THING_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("COUNTS_THE_THING_AFTER"));
static A_DUMMY_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("A_DUMMY_SUBJECT"));
static A_COPULA: LazyLock<&'static Re> = LazyLock::new(|| rx("A_COPULA"));
static WRONG_NUMBER_AFTER: LazyLock<&'static Re> = LazyLock::new(|| rx("WRONG_NUMBER_AFTER"));
static HOW_MANY: LazyLock<&'static Re> = LazyLock::new(|| rx("HOW_MANY"));
static NOT_THE_COUNT: LazyLock<&'static Re> = LazyLock::new(|| rx("NOT_THE_COUNT"));

static A_BOOKING: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:book|booked|booking|reserv\w*|table)\b", re::U));

const PUNCTUATION: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

/// `facts.PET_COUNT_WORDS.get(word)`.
fn pet_count(word: &str) -> Option<i64> {
    PET_COUNT_WORDS.iter().find(|(w, _)| w == word).map(|(_, n)| *n)
}

/// `str.isdigit()` and `int()` of it.
fn digits(said: &str) -> Option<i64> {
    if text::is_digit(said) { said.parse().ok() } else { None }
}

/// The counts the store holds (`memory["user"]["how_many"]` when it is a dict).
fn held_counts(memory: &Memory) -> Option<&Value> {
    let user = memory.get("user").filter(|u| u.truthy())?;
    Some(pyv::get(user, "how_many")).filter(|held| held.is_dict())
}

/// `isinstance(v, int)` of a held count, as the number.
fn int_of(v: &Value) -> Option<i64> {
    if pyv::is_int(v) { v.as_int() } else { None }
}

/// Python: bot/memory/counts.py:268 `_many(n)`
/// The number as a word up to twelve, as digits after that.
pub fn _many(n: i64) -> String {
    PET_COUNT_WORDS.iter().find(|(word, value)| *value == n && !word.contains(' ')).map(|(w, _)| w.clone()).unwrap_or_else(|| n.to_string())
}

/// Python: bot/memory/counts.py:276 `_head(after)`
/// The phrase a count is about, up to the word where it stops.
pub fn _head(after: &str) -> Vec<String> {
    let low = after.to_lowercase();
    let mut phrase = Vec::new();
    for word in text::split_ref(&low).into_iter().take(*MAX_PHRASE) {
        let word = text::strip_chars(word, &format!("{PUNCTUATION}\u{2019}")).to_string();
        if !text::is_alpha(&word) || PHRASE_ENDS.contains(&word) || CHAT_JOINERS.contains(&word) {
            break;
        }
        if pet_count(&word).is_some() {
            break; // the next number starts a count of its own
        }
        if VERB_IN_PHRASE.contains(&word) || word.ends_with("ly") || word.ends_with("ing") {
            break;
        }
        phrase.push(word);
    }
    phrase
}

/// Python: bot/memory/counts.py:308 `_the_relation(phrase)`
/// The singular thing a phrase counts, with the word that says which relation it is.
pub fn _the_relation(phrase: &[String]) -> String {
    let Some(last) = phrase.last() else { return String::new() };
    let thing = _thing(last);
    if thing.is_empty() || phrase.len() < 2 {
        return thing;
    }
    let before = &phrase[phrase.len() - 2];
    if RELATION_AHEAD.contains(before) {
        return format!("{before} {thing}");
    }
    if before == "great" && text::ends_with_any(&thing, &A_PERSON_TAIL) {
        return format!("great {thing}");
    }
    thing
}

/// Python: bot/memory/counts.py:322 `_thing_of(phrase)`
pub fn _thing_of(phrase: &str) -> String {
    _the_relation(&_head(phrase))
}

/// Python: bot/memory/counts.py:327 `_thing(word)`
/// The singular of a counted noun, or "" when the word cannot be a count of things.
pub fn _thing(word: &str) -> String {
    let word = text::strip_chars(&word.to_lowercase(), "-").to_string();
    if word.is_empty() || NOT_A_THING.contains(&word) || text::len(&word) < 3 {
        return String::new();
    }
    if let Some((_, one)) = IRREGULAR.iter().find(|(plural, _)| *plural == word) {
        return one.clone();
    }
    if word.ends_with("ies") && text::len(&word) > 4 {
        return format!("{}y", &word[..word.len() - 3]);
    }
    if text::ends_with_any(&word, &["ches", "shes", "sses", "xes", "zes"]) {
        return word[..word.len() - 2].to_string();
    }
    if word.ends_with('s') && !text::ends_with_any(&word, &["ss", "us", "is"]) {
        return word[..word.len() - 1].to_string();
    }
    word
}

/// Python: bot/memory/counts.py:344 `_beds(rest)`
/// How many bedrooms a phrase says the place has: "a 3 bed house" -> 3, or 0.
pub fn _beds(rest: &str) -> i64 {
    let line = text::squash(rest);
    let low = line.to_lowercase();
    let words = text::split_ref(&low);
    for found in BEDS.finditer(&line) {
        let said = found.name_str("n").to_lowercase();
        let number = digits(&said).unwrap_or_else(|| pet_count(&said).unwrap_or(0));
        if !(0 < number && number <= 20) {
            continue;
        }
        let after = text::split(text::strip_chars(&line[found.end()..], PUNCTUATION));
        if text::strip(found.name_str("join")) == "-" || !after.is_empty() || words.iter().any(|w| A_KIND_OF_PLACE.contains(*w)) {
            return number;
        }
    }
    0
}

/// Python: bot/memory/counts.py:381 `_counted(rest)`
/// {thing: number} for the counts in one possession phrase, in the order said.
pub fn _counted(rest: &str) -> IndexMap<String, i64> {
    let mut found = IndexMap::new();
    let padded = format!(" {rest}");
    // A place in the padded line, as the same character's place in `rest` (Python reads the
    // padded positions on the unpadded line).
    let at = |byte: usize| text::char_index(&padded, byte) as isize;
    for one in NUMBER_AHEAD.finditer(&padded) {
        let (n_start, _) = one.name_span("n").unwrap();
        let (who_start, who_end) = one.name_span("who").unwrap();
        let window = text::slice(rest, (at(n_start) - 16).max(0), Some(at(who_end) + 12));
        if A_RANGE_AROUND.is_match(&window) {
            continue; // a range or a choice is not the number they have
        }
        let start = at(who_start) - 1;
        let mut tail = text::slice(rest, start, None);
        if let Some((head, _)) = tail.split_once(',') {
            tail = head.to_string();
        }
        let phrase = _head(&tail);
        if phrase.iter().any(|w| NOT_A_THING.contains(w)) {
            continue; // "two left feet": no count of things
        }
        let thing = _the_relation(&phrase);
        if thing.is_empty() || THEIRS.contains(&thing) || THEIRS.contains(*text::split_ref(&thing).last().unwrap()) {
            continue;
        }
        if phrase.len() > 1 && A_MEASURE_WORD.contains(&phrase[0]) {
            continue;
        }
        let said = one.name_str("n").to_lowercase();
        let number = digits(&said).unwrap_or_else(|| pet_count(&said).unwrap_or(0));
        if number > 0 {
            found.insert(thing, number);
        }
    }
    found
}

/// Python: bot/memory/counts.py:420 `_said_number(said)`
/// The number a phrase names: "8", "eight", "twenty", "a couple of"; 0 when it names none.
pub fn _said_number(said: &str) -> i64 {
    let said = text::strip(&said.to_lowercase()).to_string();
    if let Some(n) = digits(&said) {
        return n;
    }
    if let Some(n) = pet_count(&said) {
        return n;
    }
    BIGGER_NUMBERS.get(&said).copied().unwrap_or(0)
}

/// The first of the named groups that took part (`m.group("n") or m.group("n2") or ...`).
fn first_number(found: &Match, names: &[&str]) -> String {
    names.iter().find_map(|name| found.name(name).filter(|s| !s.is_empty())).unwrap_or("").to_string()
}

/// Python: bot/memory/counts.py:431 `_the_group(said)`
/// How many people the line says are in their group, or 0.
pub fn _the_group(said: &str) -> i64 {
    for pattern in [*GROUP_OF_US, *GROUP_OF_FOUR, *GROUP_CORRECTED] {
        for found in pattern.finditer(said) {
            let number = _said_number(&first_number(&found, &["n", "n2", "n3"]));
            if number > 0 {
                return number;
            }
        }
    }
    0
}

/// Python: bot/memory/counts.py:441 `_the_party(said)`
/// How many people the line says are coming, or 0.
pub fn _the_party(said: &str) -> i64 {
    for found in PARTY.finditer(said) {
        let number = _said_number(&first_number(&found, &["n", "n2", "n3", "n4", "n5", "n6", "n7"]));
        if number > 0 {
            return number;
        }
    }
    0
}

/// Python: bot/memory/counts.py:467 `_the_chat_lines(memory)`
/// The lines of this chat a count is read from: the plan it holds and its last turns.
pub fn _the_chat_lines(memory: &Memory) -> Vec<String> {
    let user = memory.get("user").filter(|u| u.truthy()).cloned().unwrap_or_else(Value::dict);
    let mut lines = vec![pyv::str_or_empty(pyv::get(&user, "plan"))];
    let turns = &memory.turns;
    lines.extend(turns[turns.len().saturating_sub(*LOOK_BACK_FOR_A_MEAL)..].iter().map(|t| t.user.clone()));
    lines
}

/// Python: bot/memory/counts.py:474 `_the_chat_feeds_people(memory)`
pub fn _the_chat_feeds_people(memory: &Memory) -> bool {
    _the_chat_lines(memory).iter().any(|line| A_MEAL_IN_THE_CHAT.is_match(line))
}

/// Python: bot/memory/counts.py:493 `_the_chat_sets_a_table(memory)`
pub fn _the_chat_sets_a_table(memory: &Memory) -> bool {
    _the_chat_lines(memory).iter().any(|line| A_MEAL_IN_THE_CHAT.is_match(line) || A_TABLE_BOOKED.is_match(line) || A_TABLE_CUE.is_match(line))
}

/// Python: bot/memory/counts.py:500 `_the_bare_people(said)`
pub fn _the_bare_people(said: &str) -> i64 {
    for found in BARE_PEOPLE.finditer(said) {
        let number = _said_number(found.name_str("n"));
        if number > 0 {
            return number;
        }
    }
    0
}

/// Python: bot/memory/counts.py:513 `_the_number_put_right(said, memory)`
/// The number a line puts right, when the chat already holds the number at the table.
pub fn _the_number_put_right(said: &str, memory: &Memory) -> i64 {
    let Some(held_party) = held_counts(memory).and_then(|held| int_of(pyv::get(held, &PARTY_THING))).filter(|n| *n > 0) else {
        return 0;
    };
    if ABOUT_THE_HOUSE.is_match(said) {
        return 0;
    }
    if ONE_MORE.match_(said).is_some() {
        return held_party + 1;
    }
    if ONE_LESS.match_(said).is_some() {
        return if held_party > 1 { held_party - 1 } else { 0 };
    }
    for pattern in [*CHANGED_BARE, *NOW_THE_NUMBER] {
        for found in pattern.finditer(said) {
            let number = _said_number(found.name_str("n"));
            if number > 0 {
                return number;
            }
        }
    }
    0
}

/// Python: bot/memory/counts.py:569 `_forms_of(thing)`
/// The words the person may say the thing with: its singular and its plural.
pub fn _forms_of(thing: &str) -> HashSet<String> {
    if thing.is_empty() {
        return HashSet::new();
    }
    let mut forms: HashSet<String> = [thing.to_string(), format!("{thing}s"), format!("{thing}es")].into_iter().collect();
    forms.extend(IRREGULAR.iter().filter(|(_, single)| single == thing).map(|(plural, _)| plural.clone()));
    forms
}

/// Python: bot/memory/counts.py:580 `_the_figure_counts_the_thing(said, match, after_word, thing)`
/// Whether the line says the figure (whose number starts at byte `n_start`) is the count of
/// `thing`, and not of something else.
pub fn _the_figure_counts_the_thing(said: &str, n_start: usize, after_word: &str, thing: &str) -> bool {
    let before = text::strip_chars(&said[..n_start], &format!("{PUNCTUATION} ")).to_string();
    let front = text::split_ref(&before).last().map(|w| text::strip_chars(&w.to_lowercase(), PUNCTUATION).to_string()).unwrap_or_default();
    if A_LABEL_IN_FRONT.contains(&front) {
        return false;
    }
    let forms = _forms_of(thing);
    if let Some(subject) = A_COPULA.search(&before) {
        let subj = subject.name_str("subj").to_lowercase();
        if !forms.contains(&subj) && !matches!(subj.as_str(), "it" | "there" | "that" | "this") {
            return false;
        }
    }
    if !forms.contains(after_word) {
        let first = text::split_ref(said).first().map(|w| text::strip_chars(&w.to_lowercase(), PUNCTUATION).to_string()).unwrap_or_default();
        if A_DUMMY_SUBJECT.contains(&first) {
            return false;
        }
    }
    if COUNTS_THE_THING_AFTER.contains(after_word) || after_word.is_empty() {
        return true;
    }
    forms.contains(after_word)
}

/// Python: bot/memory/counts.py:610 `_the_lines(memory)`
/// The person's own lines, oldest first.
pub fn _the_lines(memory: &Memory) -> Vec<String> {
    memory.turns.iter().map(|t| t.user.clone()).filter(|line| !text::strip(line).is_empty()).collect()
}

/// Python: bot/memory/counts.py:620 `_the_thing_in_hand(memory, held)`
/// The thing the last numbered line named, when the chat keeps a count of it.
pub fn _the_thing_in_hand(memory: &Memory, held: &Value) -> String {
    for line in _the_lines(memory).iter().rev() {
        for thing in _counted(line).keys() {
            if int_of(pyv::get(held, thing)).is_some() {
                return thing.clone();
            }
        }
    }
    String::new()
}

/// Python: bot/memory/counts.py:633 `_the_figure_that_stands(said)`
/// The line up to the word that pushes a number aside, so the number that stands is read.
pub fn _the_figure_that_stands(said: &str) -> String {
    let Some(found) = WRONG_NUMBER_AFTER.search(said) else { return said.to_string() };
    let kept = text::strip(&said[..found.start()]);
    if A_FIGURE.is_match(kept) { kept.to_string() } else { said.to_string() }
}

/// Python: bot/memory/counts.py:642 `_a_figure_for_a_thing_the_chat_counts(said, memory, already=())`
/// {thing: number} for a figure said again, with the thing this chat already counts.
pub fn _a_figure_for_a_thing_the_chat_counts(said: &str, memory: &Memory, already: &IndexMap<String, i64>) -> IndexMap<String, i64> {
    let mut out = IndexMap::new();
    let Some(held) = held_counts(memory).filter(|held| held.truthy()) else { return out };
    if said.contains('?') || ASKING.match_(said).is_some() || NOT_THE_COUNT.is_match(said) {
        return out;
    }
    let said = _the_figure_that_stands(said);
    let figures: Vec<String> = A_FIGURE.finditer(&said).iter().map(|m| m.name_str("n").to_string()).collect();
    if figures.len() != 1 {
        return out; // two figures are a range or two things
    }
    let number = _said_number(&figures[0]);
    if number <= 0 {
        return out;
    }
    let named: Vec<String> = _counted(&said).into_keys().filter(|thing| held.get(thing).is_some()).collect();
    let thing = if let Some(first) = named.first() {
        first.clone()
    } else if POINTS_AT_THE_THING.is_match(&said) || SAID_AS_A_CHANGE.is_match(&said) {
        let found = A_FIGURE.search(&said).unwrap();
        let (_, n_end) = found.name_span("n").unwrap();
        let after = text::strip_chars(&said[n_end..], &format!("{PUNCTUATION} ")).to_string();
        let word = text::split_ref(&after).first().map(|w| text::strip_chars(&w.to_lowercase(), PUNCTUATION).to_string()).unwrap_or_default();
        if !already.is_empty() {
            return out; // the line already says what its figure counts
        }
        if NOT_A_THING.contains(&word) || NOT_A_COUNT.contains(&word) {
            return out;
        }
        let thing = _the_thing_in_hand(memory, held);
        if !_the_figure_counts_the_thing(&said, found.name_span("n").unwrap().0, &word, &thing) {
            return out;
        }
        thing
    } else {
        return out; // "8" on its own counts nothing until the chat says what of
    };
    if thing.is_empty() || *pyv::get(held, &thing) == Value::from(number) {
        return out;
    }
    out.insert(thing, number);
    out
}

/// Python: bot/memory/counts.py:686 `observe(text, memory)`
/// Keep how many of a thing the line says they have: `memory["user"]["how_many"][thing]`.
pub fn observe(text_: &str, memory: &mut Memory) {
    let said = text::squash(text_);
    if said.is_empty() || ASKING.match_(&said).is_some() {
        return;
    }
    let mut counts: IndexMap<String, i64> = IndexMap::new();
    let beds = _beds(&said);
    if beds != 0 {
        counts.insert("bedroom".into(), beds);
    }
    let group = _the_group(&said);
    if group != 0 {
        counts.insert(GROUP.clone(), group);
    }
    let party = _the_party(&said);
    if party != 0 {
        counts.insert(PARTY_THING.clone(), party);
    } else if _the_number_put_right(&said, memory) != 0 {
        counts.insert(PARTY_THING.clone(), _the_number_put_right(&said, memory));
    } else if _the_chat_feeds_people(memory) && _the_bare_people(&said) != 0 {
        counts.insert(PARTY_THING.clone(), _the_bare_people(&said));
    } else if group != 0 && CHANGED.is_match(&said) && !ABOUT_THE_HOUSE.is_match(&said) {
        let user = memory.get("user").filter(|u| u.truthy()).cloned().unwrap_or_else(Value::dict);
        let how_many = pyv::get(&user, "how_many");
        let held_party = if how_many.truthy() { pyv::get(how_many, &PARTY_THING).clone() } else { Value::None };
        if pyv::is_int(&held_party) {
            counts.insert(PARTY_THING.clone(), group);
        }
    }
    if let Some(put_right) = CORRECTED.match_(&said) {
        counts.extend(_counted(put_right.str(1)));
    }
    for pattern in [*THEY_HAVE, *MY_THREE] {
        for found in pattern.finditer(&said) {
            for (thing, number) in _counted(found.name_str("rest")) {
                counts.entry(thing).or_insert(number);
            }
        }
    }
    for (thing, number) in _a_figure_for_a_thing_the_chat_counts(&said, memory, &counts.clone()) {
        counts.entry(thing).or_insert(number);
    }
    if counts.is_empty() {
        return;
    }
    let user = super::user(memory);
    let mut held = match pyv::get(user, "how_many") {
        held @ Value::Dict(_) => held.clone(),
        _ => Value::dict(),
    };
    for (thing, number) in counts {
        if *pyv::get(&held, &thing) != Value::from(number) {
            held.set_key(thing.as_str(), number);
        }
    }
    user.set_key("how_many", held);
}

/// Python: bot/memory/counts.py:770 `answer(text, memory)`
/// The number of a thing they asked the count of, or None when the store holds no count.
pub fn answer(text_: &str, memory: &Memory) -> Option<String> {
    let user = memory.get("user").filter(|u| u.truthy())?;
    let held = pyv::get(user, "how_many");
    if !held.is_dict() || !held.truthy() {
        return None;
    }
    let low = text::squash(&text_.to_lowercase());
    if NOT_THE_COUNT.is_match(&low) {
        return None;
    }
    let count = |key: &str| int_of(pyv::get(held, key)).filter(|n| *n > 0);
    let of_you = GROUP_ASK_YOU.is_match(&low);
    if GROUP_ASK.is_match(&low) || BARE_GROUP_ASK.is_match(&low) || of_you {
        if let Some(number) = count(&GROUP) {
            return Some(_the_group_said_back(number));
        }
        if of_you {
            return None;
        }
        if let Some(number) = count(&PARTY_THING) {
            return Some(_the_party_said_back(number, &low, memory, low.contains("of us")));
        }
    }
    if PARTY_ASK.is_match(&low) {
        if let Some(number) = count(&PARTY_THING).filter(|_| _the_chat_sets_a_table(memory)) {
            return Some(_the_party_said_back(number, &low, memory, false));
        }
        if let Some(number) = count(&GROUP).filter(|_| POINTS_AT_A_NUMBER.is_match(&low)) {
            return Some(_the_party_said_back(number, &low, memory, true));
        }
        return None;
    }
    if BARE_COUNT_ASK.is_match(&low) {
        for (thing, of_the_group) in [(PARTY_THING.as_str(), false), (GROUP.as_str(), true)] {
            if let Some(number) = count(thing) {
                return Some(_the_party_said_back(number, &low, memory, of_the_group));
            }
        }
        return None;
    }
    let padded = format!(" {low}");
    let found = HOW_MANY.search(&padded)?;
    let mut asked = vec![_thing_of(found.name_str("what"))];
    if found.name("also").is_some_and(|a| !a.is_empty()) {
        asked.push(_thing_of(found.name_str("also")));
    }
    let asked: Vec<String> = asked.into_iter().filter(|t| !t.is_empty()).collect();
    if asked.is_empty() {
        return None;
    }
    let counted: Vec<(&String, i64)> = asked.iter().filter_map(|thing| int_of(pyv::get(held, thing)).map(|n| (thing, n))).collect();
    if counted.is_empty() {
        return None;
    }
    let parts: Vec<String> = counted
        .iter()
        .map(|(thing, number)| {
            let mut many = if *number == 1 { thing.to_string() } else { format!("{thing}s") };
            if *number != 1 {
                if let Some((plural, _)) = IRREGULAR.iter().find(|(_, one)| one == *thing) {
                    many = plural.clone();
                }
            }
            format!("{} {many}", _many(*number))
        })
        .collect();
    let said = if parts.len() == 1 { parts[0].clone() } else { format!("{} and {}", parts[..parts.len() - 1].join(" and "), parts[parts.len() - 1]) };
    if counted.len() < asked.len() {
        let missing = asked.iter().find(|thing| int_of(pyv::get(held, thing)).is_none()).unwrap();
        return Some(format!("You've got {said} - I've not been told how many {missing}s you have."));
    }
    Some(format!("You've got {said}."))
}

/// Python: bot/memory/counts.py:839 `_the_chat_books(memory)`
/// Whether the plan the chat is on is a booking, so a table is the thing to say back.
pub fn _the_chat_books(memory: &Memory) -> bool {
    let Some(user) = memory.get("user").filter(|u| u.truthy()) else { return false };
    let plans_held = pyv::get(user, "plans");
    let mut plans: Vec<Value> = if plans_held.truthy() { pyv::iter(plans_held) } else { Vec::new() };
    let plan = pyv::get(user, "plan");
    if plan.truthy() {
        plans.push(plan.clone());
    }
    plans.iter().any(|plan| A_BOOKING.is_match(&pyv::str_of(plan).to_lowercase()))
}

/// The bot's lines in this chat (as a set, in Python).
fn bot_lines(memory: &Memory) -> HashSet<&str> {
    memory.turns.iter().map(|t| t.bot.as_str()).collect()
}

/// Python: bot/memory/counts.py:849 `_the_party_said_back(number, low, memory=None, of_the_group=False)`
/// The number of people they are cooking for, said back in their own terms.
pub fn _the_party_said_back(number: i64, low: &str, memory: &Memory, of_the_group: bool) -> String {
    let many = _many(number);
    if of_the_group {
        let plain = format!("You said {many} of you.");
        return if bot_lines(memory).contains(plain.as_str()) { format!("Still {many} of you.") } else { plain };
    }
    let noun = if low.contains("guest") {
        if number == 1 { "guest" } else { "guests" }
    } else if low.contains("adult") {
        if number == 1 { "adult" } else { "adults" }
    } else if (low.contains("book") || low.contains("table") || low.contains("reserv")) && _the_chat_books(memory) {
        return format!("The table is for {many}.");
    } else if number == 1 {
        "person"
    } else {
        "people"
    };
    let plain = format!("You said {many} {noun}.");
    if bot_lines(memory).contains(plain.as_str()) {
        return format!("Still {many} {noun}.");
    }
    plain
}

/// Python: bot/memory/counts.py:878 `_the_group_said_back(number)`
/// The size of the group, said the way a person would say it back.
pub fn _the_group_said_back(number: i64) -> String {
    match number {
        1 => "Just you, then.".into(),
        2 => "The two of you, then.".into(),
        n => format!("There are {} of you.", _many(n)),
    }
}
