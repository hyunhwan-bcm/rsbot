//! Port of `bot/memory/favourite.py`: a favourite named in a line, and the ask for it back.
//!
//! "my favourite colour is green" is kept in one general slot (`memory["user"]["favourites"]`),
//! and "what is my favourite colour" names the thing, so the answer is that one fact. The value
//! has to be a thing rather than a clause or a shrug, and somebody else's favourite is not kept.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::mem7::{self, inline_re, lists, res, sets};
use crate::py::text as pt;
use crate::state::Memory;
use std::sync::LazyLock;

res!("favourite"; LEAD, SAYS, SOMEONE_ELSE, BEST_PERSON, IS_MINE, ASKS, WHICH_IS, CORRECTS, PREFERS_NOW, ASKS_ALL, AN_ANSWER, TAIL, AN_EVENT, A_CLAUSE, ASKS_SOMETHING, STUCK, SENTENCE_TOO, A_QUESTION_AFTER, KIN_APOSTROPHE);
sets!("favourite"; ACKNOWLEDGES, A_STATE, NO_VALUE, NOT_A_THING, ASK_TAIL);
lists!("favourite"; NO_VALUE_PHRASES);
inline_re! {
    CLAUSE_BREAK = "\\s*[,;]\\s*|\\s+[\u{2014}-]\\s+", 0;
    A_LETTER = r"[a-z0-9]", 0;
}
static MY_WORDS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.memory.favourite.MY_WORDS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
});

const EDGE: &str = " \t,.;:!-\u{2014}";

/// Python: bot/memory/favourite.py:165 `_plain(value)`
pub fn _plain(value: &str) -> String {
    let text = pt::squash(value);
    let text = pt::strip_chars(&text, EDGE);
    let text = TAIL.sub("", text);
    pt::strip_chars(&text, EDGE).to_string()
}

/// Python: bot/memory/favourite.py:172 `_a_value(value)`
///
/// What follows "is", when it names a favourite rather than a clause or a shrug; else "".
pub fn _a_value(value: &str) -> String {
    let text = _plain(value);
    if text.is_empty() || mem7::word_count(&text) > 6 {
        return String::new();
    }
    let low = pt::lower(&text).replace('\'', "").replace('\u{2019}', "");
    if NO_VALUE.contains(&low) || pt::split_ref(&low).iter().all(|w| NO_VALUE.contains(*w)) {
        return String::new();
    }
    if NO_VALUE_PHRASES.iter().any(|phrase| low == *phrase || low.starts_with(&format!("{phrase} "))) {
        return String::new();
    }
    if A_CLAUSE.match_(&low).is_some() {
        return String::new();
    }
    if A_STATE.contains(&low) && mem7::word_count(&low) == 1 {
        return String::new(); // what it is doing, not what they like
    }
    if AN_EVENT.is_match(&low) {
        return String::new(); // the thing now, not the favourite
    }
    if !A_LETTER.is_match(&low) {
        return String::new();
    }
    text
}

/// Python: bot/memory/favourite.py:200 `_a_thing(thing)`
///
/// The word(s) the favourite is of, kept lower-case, or "" when it names no thing.
pub fn _a_thing(thing: &str) -> String {
    let text = pt::lower(pt::strip_chars(&pt::squash(thing), " \t,.;:!-"));
    if text.is_empty() || mem7::word_count(&text) > 3 {
        return String::new();
    }
    let words: Vec<&str> = pt::split_ref(&text).into_iter().filter(|w| !ASK_TAIL.contains(*w)).collect();
    let Some(first) = words.first() else {
        return String::new();
    };
    if ASK_TAIL.contains(*first) || matches!(*first, "thing" | "things" | "one" | "which" | "what" | "who") {
        return String::new(); // "my favourite thing is ..." names no thing
    }
    words.join(" ")
}

/// Python: bot/memory/favourite.py:248 `_clauses(text)`
///
/// Each clause of the line, then the whole line.
pub fn _clauses(text: &str) -> Vec<String> {
    let said = pt::squash(text);
    let out: Vec<String> = CLAUSE_BREAK.split(&said).iter().map(|p| pt::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    if out.len() > 1 {
        let mut out = out;
        out.push(said);
        return out;
    }
    vec![said]
}

/// `{"thing": thing, "value": value}`.
fn a_favourite(thing: &str, value: &str) -> Value {
    mem7::dict(vec![("thing", Value::from(thing)), ("value", Value::from(value))])
}

/// Python: bot/memory/favourite.py:263 `read(line)`
///
/// The favourite a line states: {"thing": "colour", "value": "green"}, or None.
pub fn read(line: &str) -> Option<Value> {
    let text = pt::squash(line);
    if text.is_empty() || text.contains('?') {
        return None;
    }
    if asked(&text).is_some() || SOMEONE_ELSE.is_match(&text) || BEST_PERSON.is_match(&text) {
        return None;
    }
    let plain = LEAD.sub("", &text);
    let mut parts = _clauses(&plain);
    let before_the_question = pt::strip(&A_QUESTION_AFTER.split(&plain)[0]).to_string();
    if !before_the_question.is_empty() && before_the_question != plain {
        parts.insert(0, before_the_question);
    }
    for said in &parts {
        let Some(found) = SAYS.match_(said).or_else(|| IS_MINE.match_(said)) else {
            continue;
        };
        let thing = _a_thing(found.name_str("thing"));
        let value = _a_value(found.name_str("value"));
        if !thing.is_empty() && !value.is_empty() {
            return Some(a_favourite(&thing, &value));
        }
    }
    None
}

/// Python: bot/memory/favourite.py:294 `asks_something_too(line)`
///
/// True when the line states a favourite and asks something else as well.
pub fn asks_something_too(line: &str) -> bool {
    let text = pt::squash(line);
    if text.is_empty() {
        return false;
    }
    let parts: Vec<String> = SENTENCE_TOO.split(&LEAD.sub("", &text)).iter().map(|p| pt::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    if parts.len() < 2 {
        return false;
    }
    for part in &parts {
        let plain = pt::strip(&LEAD.sub("", part)).to_string();
        if plain.is_empty() {
            continue;
        }
        if SAYS.match_(&plain).is_some() || IS_MINE.match_(&plain).is_some() {
            continue; // this clause is the favourite itself
        }
        if plain.contains('?') || ASKS_SOMETHING.match_(&plain).is_some() || STUCK.is_match(&plain) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/favourite.py:320 `asked(line)`
///
/// The thing a line asks their favourite of ("colour"), or None when it asks something else.
pub fn asked(line: &str) -> Option<String> {
    let text = pt::squash(line);
    if text.is_empty() {
        return None;
    }
    let low = pt::lower(&text).replace('\u{2019}', "'");
    if !low.contains("favourite") && !low.contains("favorite") && !low.contains(" fav") {
        return None;
    }
    if low.contains("your favourite") || low.contains("your favorite") {
        return None; // a question about the bot
    }
    let unled = LEAD.sub("", &text);
    let thing = match ASKS.search(&unled) {
        Some(found) => found.name_str("thing").to_string(),
        None => WHICH_IS.search(&text)?.name_str("thing").to_string(),
    };
    Some(_a_thing(&thing)).filter(|t| !t.is_empty())
}

/// Python: bot/memory/favourite.py:338 `_singular(thing)`
///
/// The thing with a plural "s" taken off, for "what are my favourite films".
pub fn _singular(thing: &str) -> String {
    if pt::len(thing) > 3 && thing.ends_with('s') && !thing.ends_with("ss") {
        return thing[..thing.len() - 1].to_string();
    }
    thing.to_string()
}

/// Python: bot/memory/favourite.py:346 `asks_all(line)`
///
/// Whether the line asks for every favourite at once ("what are my favourites").
pub fn asks_all(line: &str) -> bool {
    let text = pt::squash(line);
    if text.is_empty() {
        return false;
    }
    let low = pt::lower(&text).replace('\u{2019}', "'");
    if low.contains("your favourite") || low.contains("your favorite") {
        return false;
    }
    ASKS_ALL.is_match(&low)
}

/// Python: bot/memory/favourite.py:357 `prefers_now(line)`
///
/// The value a "i prefer X now" line puts in place of the favourite last named, or "".
pub fn prefers_now(line: &str) -> String {
    let text = pt::squash(line);
    if text.is_empty() || text.contains('?') {
        return String::new();
    }
    match PREFERS_NOW.match_(&LEAD.sub("", &text)) {
        Some(found) => _a_value(found.name_str("value")),
        None => String::new(),
    }
}

/// Python: bot/memory/favourite.py:368 `an_answer(line, thing)`
///
/// The value somebody gives when the bot has just asked them for a favourite, or "".
pub fn an_answer(line: &str, thing: &str) -> String {
    let text = pt::squash(line);
    if text.is_empty() || text.contains('?') || thing.is_empty() {
        return String::new();
    }
    let plain = LEAD.sub("", &text);
    if AN_ANSWER.match_(&plain).is_none() {
        return String::new();
    }
    let low = pt::lower(&plain).replace('.', "").replace('!', "").replace(',', "");
    let words = pt::split_ref(&low);
    if words.is_empty() || words.iter().any(|w| ASK_TAIL.contains(*w) || NOT_A_THING.contains(*w)) {
        return String::new();
    }
    if words.iter().all(|w| ACKNOWLEDGES.contains(*w) || w.starts_with("thank")) {
        return String::new(); // "thanks", "no thanks", "ok" is not the favourite asked for
    }
    _a_value(&plain)
}

/// Python: bot/memory/favourite.py:384 `corrected(line, thing)`
///
/// The value a correction puts in place of a favourite just given, or "".
pub fn corrected(line: &str, thing: &str) -> String {
    let text = pt::squash(line);
    if text.is_empty() || text.contains('?') || thing.is_empty() {
        return String::new();
    }
    match CORRECTS.match_(&text) {
        Some(found) => _a_value(found.name_str("value")),
        None => String::new(),
    }
}

/// `(memory.get("user") or {}).get("favourites")` when it is a dict.
fn held(memory: &Memory) -> Option<&Value> {
    memory.get("user").filter(|u| u.truthy()).and_then(|u| u.get("favourites")).filter(|h| h.is_dict())
}

/// Python: bot/memory/favourite.py:395 `kept(memory, thing)`
///
/// The favourite on record for that thing, or "" when they have not said one.
pub fn kept(memory: &Memory, thing: &str) -> String {
    if thing.is_empty() {
        return String::new();
    }
    let Some(held) = held(memory) else {
        return String::new();
    };
    match held.get(thing) {
        Some(Value::Str(value)) => value.to_string(),
        _ => match held.get(&_singular(thing)) {
            Some(Value::Str(value)) => value.to_string(),
            _ => String::new(),
        },
    }
}

/// Python: bot/memory/favourite.py:408 `the_favourites(memory)`
///
/// Every favourite on record, in the order they were given, as (thing, value).
pub fn the_favourites(memory: &Memory) -> Vec<(String, String)> {
    held(memory)
        .map(|held| held.pairs().iter().filter_map(|(thing, value)| value.as_str().map(|v| (mem7::py_str(thing), v.to_string()))).collect())
        .unwrap_or_default()
}

/// Python: bot/memory/favourite.py:418 `store(memory, found)`
///
/// Keep one favourite in the store, replacing the same thing said again.
pub fn store(memory: &mut Memory, found: &Value) {
    if !found.is_dict() {
        return;
    }
    let thing = mem7::get(found, "thing").clone();
    let value = mem7::get(found, "value").clone();
    if !thing.truthy() || !value.truthy() {
        return;
    }
    let known = memory.setdefault("user", Value::dict());
    if !known.is_dict() {
        panic!("AttributeError: memory['user'] has no attribute 'get'");
    }
    let mut favourites = match known.get("favourites") {
        Some(held) if held.is_dict() => held.clone(),
        _ => Value::dict(),
    };
    favourites.set_key(thing, value);
    known.set_key("favourites", favourites);
}

/// Python: bot/memory/favourite.py:452 `their_words(value)`
///
/// The person's own words about themselves, said back: "my mums curry" -> "your mum's curry".
pub fn their_words(value: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for word in pt::split_ref(value) {
        let low = pt::strip_chars(&pt::lower(word), ".,!?").to_string();
        let Some((_, swapped)) = MY_WORDS.iter().find(|(k, _)| *k == low) else {
            out.push(word.to_string());
            continue;
        };
        // `word[len(low):]`
        let tail = mem7::from_char(word, pt::len(&low));
        let mut said = if mem7::is_upper(mem7::first_chars(word, 1)) { pt::upper_first(swapped) } else { swapped.clone() };
        said.push_str(tail);
        out.push(said);
    }
    KIN_APOSTROPHE.sub(r"\1's", &out.join(" "))
}

/// Python: bot/memory/favourite.py:467 `watch(text, memory)`
///
/// Keep the favourite a line names, answers, corrects or changes, and mark it for the reply.
pub fn watch(text: &str, memory: &mut Memory) {
    let turns = mem7::turn_count(memory);
    let mut said = read(text);
    if said.is_none() {
        let pending = memory.remove("favourite_asked");
        if let Some(Value::Str(pending)) = pending.filter(|p| p.truthy()) {
            let answered = an_answer(text, &pending);
            if !answered.is_empty() {
                // The bot asked for the favourite, and this short line is the answer.
                let found = a_favourite(&pending, &answered);
                memory.set("favourite_told", found.clone());
                said = Some(found);
            }
        }
    }
    let recent = memory.get("favourite_recent").filter(|r| r.is_dict() && mem7::get(r, "thing").truthy()).cloned();
    if said.is_none() {
        // A correction the turn after a favourite was given replaces it.
        if let Some(recent) = &recent {
            if *mem7::get(recent, "turn") == Value::from(turns - 1) {
                let thing = mem7::py_str(mem7::get(recent, "thing"));
                let put_right = corrected(text, &thing);
                if !put_right.is_empty() {
                    let found = a_favourite(&thing, &put_right);
                    memory.set("favourite_told", found.clone());
                    said = Some(found);
                }
            }
        }
    }
    if said.is_none() {
        // "i prefer blue now" a turn or two after the favourite is the favourite changing.
        if let Some(recent) = &recent {
            if turns - mem7::int_or_0(recent.get("turn")) <= 2 {
                let thing = mem7::py_str(mem7::get(recent, "thing"));
                let moved = prefers_now(text);
                if !moved.is_empty() && moved != kept(memory, &thing) {
                    let was = kept(memory, &thing);
                    said = Some(a_favourite(&thing, &moved));
                    memory.set("favourite_changed", mem7::dict(vec![("thing", Value::from(thing.as_str())), ("value", Value::from(moved)), ("was", Value::from(was))]));
                }
            }
        }
    }
    if let Some(said) = said {
        store(memory, &said);
        memory.set("favourite_recent", mem7::dict(vec![("thing", mem7::get(&said, "thing").clone()), ("turn", Value::from(turns))]));
    }
}
