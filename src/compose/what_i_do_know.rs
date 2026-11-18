//! Port of `bot/compose/what_i_do_know.py`: the half of an answer a person can have - the thing,
//! what is known about that kind of thing, and the one place that keeps the rest - when the
//! composer would otherwise only hedge.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.what_i_do_know.", stringify!($name))));)*
    };
}
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.compose.what_i_do_know.", stringify!($name))));)*
    };
}

regexes!(
    A_METHOD, A_TIME, A_PLACE, A_FREQUENCY, A_FIGURE, A_REASON, COOKING, A_PURCHASE, A_PERSON, GOT_IT,
    ANIMAL_AS_A_SOURCE, REFERS_BACK, A_DETERMINER, A_BUYING_NOW, NUTRITION, CORRECTION, LIVING, OWNED_ANYWAY,
    ON_THE_BODY, CLINICAL,
);
word_sets!(MADE, NOT_A_THING, SINGULAR_BUT_ENDS_IN_S, MODIFIERS, PARTICLES, NOT_AN_OBJECT, PART_OF_SOMETHING);

fn named(name: &str) -> &'static Value {
    data::get(&format!("bot.compose.what_i_do_know.{name}"))
}

/// `KINDS`: (kind, pattern) in order.
static KINDS: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    named("KINDS")
        .items()
        .iter()
        .map(|pair| (pair.items()[0].str().to_string(), pair.items()[1].as_re().expect("KINDS: a pattern")))
        .collect()
});
static WHERE_IT_LIVES: LazyLock<HashMap<String, String>> = LazyLock::new(|| named("WHERE_IT_LIVES").str_map());
static PART_WORD: LazyLock<HashMap<String, String>> = LazyLock::new(|| named("PART_WORD").str_map());
static HOLDS: LazyLock<HashMap<String, HashSet<String>>> = LazyLock::new(|| {
    named("HOLDS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_set())).collect()
});
static GENERAL_BY_PART: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| named("GENERAL_BY_PART").str_list_map());
static BOUGHT_GENERAL: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| named("BOUGHT_GENERAL").str_list_map());
static TAIL: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| named("TAIL").str_list_map());
/// `BY_KIND`, keyed by (kind, part).
static BY_KIND: LazyLock<HashMap<(String, String), Vec<String>>> = LazyLock::new(|| {
    named("BY_KIND")
        .pairs()
        .iter()
        .map(|(k, v)| ((k.items()[0].str().to_string(), k.items()[1].str().to_string()), v.strs()))
        .collect()
});

static A_NAMED_THING: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(?:my|our|the|this|these|a|an)\s+([a-z][a-z'-]+(?:\s+[a-z][a-z'-]+){0,3})", 0));
static A_PLAIN_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z][a-z'-]+", 0));

/// Python: bot/compose/what_i_do_know.py:394 `_parts_of(said)`
/// The part of the answer the line is after, by the first shape that matches.
pub fn _parts_of(said: &str) -> String {
    let rules: [(&str, &Re); 6] = [
        ("method", &A_METHOD),
        ("time", &A_TIME),
        ("place", &A_PLACE),
        ("frequency", &A_FREQUENCY),
        ("figure", &A_FIGURE),
        ("reason", &A_REASON),
    ];
    rules.iter().find(|(_, rule)| rule.is_match(said)).map(|(name, _)| name.to_string()).unwrap_or_default()
}

/// Python: bot/compose/what_i_do_know.py:416 `_kind_of(said)`
/// The kind of thing the line names, or "" when it names none of them.
pub fn _kind_of(said: &str) -> String {
    if ANIMAL_AS_A_SOURCE.is_match(said) {
        return String::new();
    }
    KINDS.iter().find(|(_, rule)| rule.is_match(said)).map(|(kind, _)| kind.clone()).unwrap_or_default()
}

/// Python: bot/compose/what_i_do_know.py:440 `_that(thing)`
/// The word to put in front of the thing, singular or plural, from how people say it.
pub fn _that(thing: &str) -> String {
    if thing.ends_with('s') && !SINGULAR_BUT_ENDS_IN_S.contains(thing) {
        return "those".to_string();
    }
    "that".to_string()
}

/// Python: bot/compose/what_i_do_know.py:481 `_without_a_particle(phrase)`
pub fn _without_a_particle(phrase: &str) -> String {
    let mut words = text::split_ref(phrase);
    while words.last().is_some_and(|w| PARTICLES.contains(*w)) {
        words.pop();
    }
    words.join(" ")
}

/// Python: bot/compose/what_i_do_know.py:488 `_a_thing_from(said, determiner_only=False)`
/// The thing a person's own line names, by the words around it, or "": the noun phrase after a
/// determiner, else the last run of ordinary words that is not a function word.
pub fn _a_thing_from(said: &str, determiner_only: bool) -> String {
    for found in A_NAMED_THING.finditer(said) {
        // The words people put in front of the noun point at it and are not part of its name, and
        // the noun phrase stops where the person stops naming and starts saying something about it.
        let mut words = text::split_ref(found.str(1));
        while words.first().is_some_and(|w| MODIFIERS.contains(*w)) {
            words.remove(0);
        }
        let mut kept: Vec<&str> = Vec::new();
        for word in words {
            if MODIFIERS.contains(word) || NOT_A_THING.contains(word) {
                break;
            }
            kept.push(word);
            if kept.len() == 2 {
                break;
            }
        }
        let kept = _without_a_particle(&kept.join(" "));
        if !kept.is_empty() {
            return kept;
        }
    }
    if determiner_only {
        return String::new();
    }
    let words = A_PLAIN_WORD.findall_str(said);
    let mut run: Vec<String> = Vec::new();
    let mut best = String::new();
    for word in words {
        if NOT_A_THING.contains(&word) {
            if !run.is_empty() {
                best = run[run.len().saturating_sub(2)..].join(" ");
                run.clear();
            }
            continue;
        }
        run.push(word);
    }
    if !run.is_empty() {
        best = run[run.len().saturating_sub(2)..].join(" ");
    }
    let best = _without_a_particle(text::strip(&best));
    if text::len(&best) < 3 || !text::split_ref(&best).iter().all(|word| text::is_alpha(word)) {
        return String::new();
    }
    best
}

/// Python: bot/compose/what_i_do_know.py:542 `_thing_named_in(text)`
/// The thing a line names, read the way the line means it: a line that corrects itself means
/// the second thing it says.
pub fn _thing_named_in(text_: &str) -> String {
    if let Some(mark) = CORRECTION.search(text_) {
        let corrected = _a_thing_from(&text_[mark.end()..], true);
        if !corrected.is_empty() {
            return corrected;
        }
    }
    _a_thing_from(text_, false)
}

/// The engine's subject, then its first entity, as a thing, or "".
fn subject_or_entity(subject: Option<&str>, entities: &[String]) -> String {
    let subject = text::lower(text::strip(subject.unwrap_or("")));
    if !subject.is_empty() && !NOT_A_THING.contains(&subject) {
        return subject;
    }
    for ent in entities {
        let ent = text::lower(text::strip(ent));
        if !ent.is_empty() && !NOT_A_THING.contains(&ent) {
            return ent;
        }
    }
    String::new()
}

/// Python: bot/compose/what_i_do_know.py:559 `_the_thing(read, said, memory)`
/// The thing the line is about, said the way the person said it, or "": a pronoun takes the
/// thing the line before named; then a correction, a determiner phrase, the engine's subject or
/// entity, and last the line before, read word by word. (The line before is only the words of
/// the turn; it carries no subject or entities.)
pub fn _the_thing(read: &Understood, said: &str, memory: &Memory) -> String {
    let before = _the_line_before(memory);
    let before_text = before.clone().unwrap_or_default();
    let named_before = if before_text.is_empty() { String::new() } else { _thing_named_in(&before_text) };
    // A pronoun is the person telling you where to look; the determiner only wins when it comes
    // first - "the filter, how do i clean it" - which is the line naming the thing outright.
    let points_back = REFERS_BACK.search(said).is_some_and(|back| !A_DETERMINER.is_match(&said[..back.start()]));
    if points_back && !named_before.is_empty() {
        return named_before;
    }
    if let Some(mark) = CORRECTION.search(said) {
        let corrected = _a_thing_from(&said[mark.end()..], true);
        if !corrected.is_empty() {
            return corrected;
        }
    }
    let named_now = _a_thing_from(said, true);
    if !named_now.is_empty() {
        return named_now;
    }
    let entities: Vec<String> = read.entities.iter().map(|e| e.text.clone()).collect();
    let from_read = subject_or_entity(read.subject.as_deref(), &entities);
    if !from_read.is_empty() {
        return from_read;
    }
    // The turn before, and only there, is read word by word: that is the line that named the
    // thing, and a question's own words ("how often do i clean it") are not a thing.
    if let Some(before) = before {
        let named = _thing_named_in(&before);
        if !named.is_empty() {
            return named;
        }
    }
    String::new()
}

/// Python: bot/compose/what_i_do_know.py:617 `before_text_from(memory)`
/// The words of the person's previous line, or "".
pub fn before_text_from(memory: &Memory) -> String {
    _the_line_before(memory).unwrap_or_default()
}

/// Python: bot/compose/what_i_do_know.py:622 `_the_line_before(memory)`
/// The person's previous line, word for word, for a follow-up that names "it" (Python's
/// `{"text": said}`), or None.
pub fn _the_line_before(memory: &Memory) -> Option<String> {
    let now = memory.understood.as_ref().map(|u| text::lower(text::strip(&u.text))).unwrap_or_default();
    for turn in memory.turns.iter().rev() {
        let said = text::strip(&turn.user);
        if !said.is_empty() && text::lower(said) != now {
            return Some(said.to_string());
        }
    }
    None
}

/// Python: bot/compose/what_i_do_know.py:712 `_kind_from_the_owning_line(memory, thing, said="")`
/// The kind of thing, when the person's own line says they got it, or "".
pub fn _kind_from_the_owning_line(memory: &Memory, thing: &str, said: &str) -> String {
    if thing.is_empty() {
        return String::new();
    }
    let sources = [
        memory.understood.as_ref().map(|u| u.text.clone()).unwrap_or_default(),
        _the_line_before(memory).unwrap_or_default(),
        said.to_string(),
    ];
    for source in sources {
        let text_ = text::lower(&source);
        if !text_.contains(thing) {
            continue;
        }
        if text::split_ref(thing).iter().all(|word| NOT_AN_OBJECT.contains(*word)) {
            continue;
        }
        // "my humidifier", "our bread maker": the person has said it is theirs, so the maker's page
        // and the manual are the pages that hold it, whatever the thing turns out to be.
        if text_.contains(&format!("my {thing}"))
            || text_.contains(&format!("our {thing}"))
            || GOT_IT.is_match(&text_)
            || OWNED_ANYWAY.is_match(&text_)
        {
            // A thing that is alive, or is a home for something alive, keeps its own calendar.
            if LIVING.is_match(thing) {
                return "living".to_string();
            }
            if CLINICAL.is_match(thing) {
                return "clinical".to_string();
            }
            // Nothing on the body came in a box, whatever the line says about having it.
            if ON_THE_BODY.is_match(thing) {
                return String::new();
            }
            return "bought".to_string();
        }
    }
    String::new()
}

/// Python: bot/compose/what_i_do_know.py:763 `_known(kind, part)`
/// What this file can say about the kind of thing, before it says where the rest is.
pub fn _known(kind: &str, part: &str) -> Vec<String> {
    if let Some(known) = BY_KIND.get(&(kind.to_string(), part.to_string())) {
        return known.clone();
    }
    if MADE.contains(kind) {
        return GENERAL_BY_PART.get(part).cloned().unwrap_or_default();
    }
    if kind == "bought" {
        return BOUGHT_GENERAL.get(part).cloned().unwrap_or_default();
    }
    Vec::new()
}

/// `compose.pick(options, read, said)`: the memory `pick` is handed is the reading itself.
fn pick(options: &[String], read: Option<&Understood>, said: &str) -> String {
    // A reading holds no "turns", so `pick` reads it as a memory with no conversation in it.
    let _ = read;
    crate::compose::pick(options, &crate::state::Memory::new(), said)
}

/// Python: bot/compose/what_i_do_know.py:779 `reply_for(text, memory=None, reply="")`
/// What is known about the thing, where the rest of it lives, and one question - or None.
pub fn reply_for(text_: &str, memory: &Memory, reply: &str) -> Option<String> {
    if !crate::compose::fallback::is_a_hedge(reply) {
        return None;
    }
    if crate::compose::fallback::in_a_safety_thread(memory) {
        return None;
    }
    let said = text::lower(text_);
    if text::strip(&said).is_empty() {
        return None;
    }
    if A_PERSON.is_match(&said) {
        return None;
    }
    if crate::compose::fallback::_thread_is_about_the_body(memory, 4) {
        return None;
    }
    let read = memory.understood.as_ref();
    let empty = Understood::default();
    if COOKING.is_match(&said)
        || A_PURCHASE.is_match(&said)
        || NUTRITION.is_match(&said)
        || A_BUYING_NOW.is_match(&before_text_from(memory))
    {
        return None;
    }
    if ANIMAL_AS_A_SOURCE.is_match(&said) {
        return None;
    }
    let thing = _the_thing(read.unwrap_or(&empty), &said, memory);
    if thing.is_empty() {
        return None;
    }
    if ANIMAL_AS_A_SOURCE.is_match(&thing) || A_PERSON.is_match(&thing) {
        return None;
    }
    let mut kind = if text::split_ref(&thing).iter().all(|w| PART_OF_SOMETHING.contains(*w)) {
        "part".to_string()
    } else {
        String::new()
    };
    if kind.is_empty() {
        kind = _kind_of(&said);
    }
    if kind.is_empty() {
        kind = _kind_of(&thing);
    }
    if kind.is_empty() {
        kind = _kind_of(&read.map(|u| u.text.clone()).unwrap_or_default());
        if kind.is_empty() {
            kind = _kind_of(&_the_line_before(memory).unwrap_or_default());
        }
    }
    if kind.is_empty() {
        kind = _kind_from_the_owning_line(memory, &thing, &said);
    }
    if kind.is_empty() {
        return None;
    }
    let part = _parts_of(&said);
    if part.is_empty() {
        return None;
    }
    if !HOLDS.get(&kind).is_some_and(|parts| parts.contains(&part)) {
        return None;
    }
    let knowns = _known(&kind, &part);
    let (Some(place), Some(word), Some(tails)) = (WHERE_IT_LIVES.get(&kind), PART_WORD.get(&part), TAIL.get(&part)) else {
        return None;
    };
    if place.is_empty() || knowns.is_empty() || word.is_empty() || tails.is_empty() {
        return None;
    }
    // The part it cannot check said in a few words, then what it does know about the kind of
    // thing, then the one place that keeps the rest, then a question that keeps the turn going.
    let dem = _that(&thing);
    let mut options = Vec::new();
    for known in &knowns {
        for tail in tails {
            options.push(format!("I can't check {word} for {dem} {thing}, sorry. What I can say is that {known}. {place} {tail}"));
        }
    }
    Some(pick(&options, read, &said))
}
