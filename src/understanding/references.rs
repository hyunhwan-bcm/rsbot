//! Port of `bot/understanding/references.py`: what "it", "that", "they" point back at.

#![allow(non_upper_case_globals)]

use crate::py::re::Re;
use crate::py::text;
use crate::understanding::kinds::Read;
use crate::understanding::normalize::Line;
use crate::understanding::{breeds, rx, t, topics};
use crate::understood::{Entity, Ref, Understood};
use indexmap::IndexMap;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.understanding.references.";

static ANIMALS_IN_HAND: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ANIMALS_IN_HAND"));
static ANIMAL_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ANIMAL_NAMES"));
static ASK_OPENERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_OPENERS"));
static CLAUSE_SUBJECTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CLAUSE_SUBJECTS"));
static COORDINATORS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COORDINATORS"));
static DETERMINERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DETERMINERS"));
static FOLLOWUP_STARTS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "FOLLOWUP_STARTS"));  // t, 40 items
static LEAVES_SOMETHING_OUT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LEAVES_SOMETHING_OUT"));
static MINE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MINE"));
static NAMES_A_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NAMES_A_THING"));
static ONE_PERSON: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "ONE_PERSON"));  // t, 6 items
static OWN_PERSONS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "OWN_PERSONS"));  // t, 3 items
static OWN_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OWN_SUBJECT"));
static PLACEHOLDER_IT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PLACEHOLDER_IT"));
static POINTING_BACK: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "POINTING_BACK"));
static POSSESSIVE_PRONOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "POSSESSIVE_PRONOUNS"));
static PRONOUNS: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| t::map_strs(M, "PRONOUNS"));
static PRONOUN_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PRONOUN_WORDS"));
static STANDS_FOR_SOMETHING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STANDS_FOR_SOMETHING"));
static THIRD_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THIRD_PERSON"));
static WITHOUT_THE_APOSTROPHE: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| t::map_strs(M, "WITHOUT_THE_APOSTROPHE"));
static BREED_KIND: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map("bot.understanding.breeds.", "BREED_KIND"));

/// `_target(prev_line, prev_entities, prev_topic)`: `(name, kind)` the last line was about.
pub fn _target(_prev_line: &Line, prev_entities: &[Entity], prev_topic: Option<&str>) -> (Option<String>, Option<String>) {
    let things: Vec<&Entity> = prev_entities.iter().filter(|e| e.kind == "thing").collect();
    if let Some(first) = things.first() {
        let mut words = vec![first.text.clone()];
        let mut at = first.at;
        for later in &things[1..] {
            if later.at == at + 1 {
                words.push(later.text.clone());
                at = later.at;
            }
        }
        return (Some(words.join(" ").to_lowercase()), Some("thing".to_string()));
    }
    for kind in ["title", "name", "place", "person"] {
        if let Some(e) = prev_entities.iter().find(|e| e.kind == kind) {
            return (Some(e.text.to_lowercase()), Some(kind.to_string()));
        }
    }
    if let Some(topic) = prev_topic.filter(|t| !t.is_empty()) {
        return (Some(topic.replace('-', " ")), Some("topic".to_string()));
    }
    (None, None)
}

/// `_person(prev_entities, fallback)`.
fn _person(prev_entities: &[Entity], fallback: Option<String>) -> Option<String> {
    for kind in ["name", "title", "person"] {
        if let Some(e) = prev_entities.iter().find(|e| e.kind == kind) {
            return Some(e.text.to_lowercase());
        }
    }
    fallback
}

/// `_place(prev_entities, prev_topic)`.
fn _place(prev_entities: &[Entity], prev_topic: Option<&str>) -> Option<String> {
    if let Some(e) = prev_entities.iter().find(|e| e.kind == "place") {
        return Some(e.text.to_lowercase());
    }
    if prev_topic == Some("geography-and-travel") { Some("geography and travel".to_string()) } else { None }
}

/// `_moved(read, prev_topic)`: classify's reading has no topic, so it never moved.
fn _moved(_read: &Read, _prev_topic: Option<&str>) -> bool {
    false
}

/// `_has_moved(line, prev_topic)`: the line's own topic is another than the last line's.
fn _has_moved(line: &Line, prev_topic: Option<&str>) -> bool {
    let Some(prev) = prev_topic.filter(|t| !t.is_empty()) else { return false };
    let entities = topics::detect_entities(line, None);
    let (own, _) = topics::detect_topic(line, None, Some(&entities), None);
    own.is_some_and(|o| !o.is_empty() && o != prev)
}

/// `_a_thing_of_its_own(norm, at, word)`.
fn _a_thing_of_its_own(norm: &[String], at: usize, word: &str) -> bool {
    let Some(opened) = _last_noun_phrase(norm, at) else { return false };
    if POSSESSIVE_PRONOUNS.contains(word) {
        return true;
    }
    norm[opened..at].iter().any(|o| COORDINATORS.contains(o))
}

/// `_last_noun_phrase(norm, at)`.
fn _last_noun_phrase(norm: &[String], at: usize) -> Option<usize> {
    let mut index = at as isize - 2;
    let stop = (at as isize - 12).max(-1);
    while index > stop {
        let word = &norm[index as usize];
        if DETERMINERS.contains(word) && (index as usize) + 1 < at {
            return Some(index as usize);
        }
        if PRONOUN_WORDS.contains(word) {
            return None;
        }
        index -= 1;
    }
    None
}

fn pronouns(name: &str) -> &'static [String] {
    PRONOUNS.get(name).map(Vec::as_slice).unwrap_or_else(|| panic!("KeyError: {name}"))
}

/// `_named_before(line, names, moved=True)`: the pronouns whose thing the line names itself.
fn _named_before(line: &Line, names: &[&str], moved: bool) -> Vec<String> {
    let norm = &line.norm;
    let people: Vec<i64> = topics::detect_entities(line, None).iter().filter(|e| OWN_PERSONS.contains(&e.kind)).map(|e| e.at).collect();
    let mut found = Vec::new();
    for name in names {
        let aliases = pronouns(name);
        let Some(at) = norm.iter().position(|w| aliases.contains(w)) else { continue };
        if at > 0 && POINTING_BACK.contains(&norm[at - 1]) {
            continue;
        }
        if _asks_about(line, &[name]) {
            continue;
        }
        let mut named = people.iter().any(|&spot| spot < at as i64);
        if !named && moved {
            named = _a_thing_of_its_own(norm, at, &norm[at]);
        }
        if !named {
            continue;
        }
        found.push(name.to_string());
    }
    found
}

/// `_asks_about(line, names)`: "what is it", "where are they".
fn _asks_about(line: &Line, names: &[&str]) -> bool {
    let aliases: HashSet<&String> = names.iter().flat_map(|n| pronouns(n).iter()).collect();
    line.clauses.iter().any(|c| !c.is_empty() && ASK_OPENERS.contains(&c[0]) && c.iter().take(3).any(|w| aliases.contains(w)))
}

/// `person_is_new(line, prev_entities)`: the line brings in a person the last line did not.
pub fn person_is_new(line: &Line, prev_entities: &[Entity]) -> bool {
    let people_here = !topics::person_names(line).is_empty() || topics::detect_entities(line, None).iter().any(|e| e.kind == "person");
    if !people_here && !line.norm.iter().any(|w| ONE_PERSON.contains(w)) {
        return false;
    }
    !prev_entities.iter().any(|e| e.kind == "person")
}

/// `_only_the_purpose(line)`: "to fix it" carries on.
fn _only_the_purpose(line: &Line) -> bool {
    let norm = &line.norm;
    if norm.len() < 2 || norm[0] != "to" {
        return false;
    }
    !norm[1..].iter().any(|w| CLAUSE_SUBJECTS.contains(w))
}

/// `_a_placeholder_it(line)`: "it is raining", "it's fine" - an it that stands for nothing.
fn _a_placeholder_it(line: &Line) -> bool {
    let said = line.norm.join(" ");
    let Some(frame) = PLACEHOLDER_IT.match_(&said) else { return false };
    let rest = &said[frame.end()..];
    if NAMES_A_THING.is_match(rest) {
        return false;
    }
    let rest_words = text::split_ref(rest);
    if rest_words.iter().any(|w| STANDS_FOR_SOMETHING.contains(*w)) {
        return false;
    }
    !rest_words.last().is_some_and(|w| LEAVES_SOMETHING_OUT.contains(*w))
}

/// `_the_breed_in(text)`: the animal a breed named in the text is.
fn _the_breed_in(text_: &str) -> String {
    let whole = text::strip(text_).to_lowercase();
    if let Some(k) = BREED_KIND.get(&whole).filter(|k| !k.is_empty()) {
        return k.clone();
    }
    let said: Vec<String> = rx("[a-z']+").findall_str(&text_.to_lowercase());
    if said.is_empty() {
        return String::new();
    }
    let filled = breeds::the_animal_a_breed_is(&said);
    if filled.len() <= said.len() {
        return String::new();
    }
    let mut index = 0;
    for piece in filled {
        if index < said.len() && piece == said[index] {
            index += 1;
            continue;
        }
        return piece.to_lowercase();
    }
    String::new()
}

/// `_the_pronouns(name)`.
fn _the_pronouns(name: &str) -> HashSet<&'static str> {
    let mut s: HashSet<&'static str> = pronouns(name).iter().map(String::as_str).collect();
    if let Some(v) = WITHOUT_THE_APOSTROPHE.get(name) {
        s.extend(v.iter().map(String::as_str));
    }
    s
}

/// The bare words of a list: `str(word).lower().strip(string.punctuation + '’').split("'")[0]`.
fn bare_words(words: &[String]) -> HashSet<String> {
    const PUNCT: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~\u{2019}";
    words.iter().map(|w| text::strip_chars(&w.to_lowercase(), PUNCT).split('\'').next().unwrap_or("").to_string()).collect()
}

/// `the_animal_the_chat_is_on(words, last_read)`: the animal the conversation is about, when
/// the line points at it with "he"/"she"/"it" and names no other.
pub fn the_animal_the_chat_is_on(words: &[String], last_read: Option<&Understood>) -> String {
    let Some(last) = last_read else { return String::new() };
    let animal = last.animal_in_hand.to_lowercase();
    if animal.is_empty() {
        return String::new();
    }
    let bare = bare_words(words);
    if bare.iter().any(|w| ANIMAL_NAMES.contains(w) || MINE.contains(w)) {
        return String::new();
    }
    if !bare.iter().any(|w| THIRD_PERSON.contains(w)) {
        return String::new();
    }
    animal
}

/// `the_animal_in_hand(ref, words)`: the animal a follow-up's pronoun points at.
pub fn the_animal_in_hand(reference: &Ref, words: &[String]) -> String {
    if !reference.followup {
        return String::new();
    }
    let bare = bare_words(words);
    if bare.iter().any(|w| OWN_SUBJECT.contains(w) || ANIMAL_NAMES.contains(w)) {
        return String::new();
    }
    for name in PRONOUNS.keys() {
        let theirs = _the_pronouns(name);
        if !bare.iter().any(|w| theirs.contains(w.as_str())) {
            continue;
        }
        let said = match name.as_str() {
            "it" => reference.it.clone(),
            "that" => reference.that.clone(),
            "they" => reference.they.clone(),
            "there" => reference.there.clone(),
            "he" => reference.he.clone(),
            "she" => reference.she.clone(),
            "kind" => reference.kind.clone(),
            _ => None,
        }
        .unwrap_or_default();
        if let Some(animal) = ANIMALS_IN_HAND.search(&said) {
            return animal.as_str().to_lowercase();
        }
        let breed = _the_breed_in(&said);
        if !breed.is_empty() {
            return breed;
        }
    }
    String::new()
}

/// `resolve(line, read, prev_line=None, prev_entities=None, prev_topic=None)`: what each
/// pronoun points back at, and whether the line follows on from the last one.
pub fn resolve(line: &Line, read: &Read, prev_line: Option<&Line>, prev_entities: &[Entity], prev_topic: Option<&str>) -> Ref {
    let words: HashSet<&str> = line.norm.iter().map(String::as_str).collect();
    let used = |name: &str| pronouns(name).iter().any(|p| words.contains(p.as_str()));
    let mut r = Ref::default();
    let Some(prev_line) = prev_line else { return r };
    let (target, kind) = _target(prev_line, prev_entities, prev_topic);
    let person = _person(prev_entities, target.clone());
    let others_used = used("that") || used("they") || used("there") || used("he") || used("she");
    let mut reference = used("it") || others_used;
    let placeholder = _a_placeholder_it(line);
    let starts = line.norm.iter().take(4).cloned().collect::<Vec<_>>().join(" ");
    let mut opens_like_followup = FOLLOWUP_STARTS.iter().any(|p| starts == *p || starts.starts_with(&format!("{p} "))) || _only_the_purpose(line);
    if placeholder && !others_used {
        opens_like_followup = false;
    }
    let short = line.norm.len() <= 3 && read.kind != "greeting" && read.kind != "thanks";
    let entities = topics::detect_entities(line, None);
    let adds_topic = topics::subject_topic(line, Some(&entities)).is_some();
    let asks_without_topic = read.kind == "question" && !adds_topic;
    r.kind = kind;
    if reference && placeholder && !others_used {
        reference = false;
    }
    if reference {
        r.it = target.clone();
        r.that = target.clone();
        r.they = target;
        r.he = person.clone();
        r.she = person;
    }
    r.there = _place(prev_entities, prev_topic);
    r.followup = reference || opens_like_followup || short || asks_without_topic;
    let moved = _moved(read, prev_topic) || _has_moved(line, prev_topic);
    let named_first = _named_before(line, &["it", "that", "they", "there"], moved);
    for name in &named_first {
        match name.as_str() {
            "it" => r.it = None,
            "that" => r.that = None,
            "they" => r.they = None,
            "there" => r.there = None,
            _ => {}
        }
        if name == "they" || name == "that" {
            r.he = None;
            r.she = None;
        }
    }
    let pointed_back: HashSet<&str> = ["it", "that", "they", "there"].into_iter().filter(|n| used(n)).collect();
    let named_set: HashSet<&str> = named_first.iter().map(String::as_str).collect();
    if !pointed_back.is_empty() && pointed_back == named_set && !(used("he") || used("she")) {
        r.followup = opens_like_followup || short || asks_without_topic;
    }
    r
}
