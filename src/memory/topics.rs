//! Port of `bot/memory/topics.py`: the parts the understand observer calls - a pronoun in a
//! short ask filled with the thing it stands for (`the_thing_it_is`), and a bare advice ask given
//! the doing the chat is about (`the_advice_ask_with_its_doing`), with the word helpers they use.

#![allow(non_upper_case_globals)]

use crate::data::Value;
use crate::knowledge::lookup::{self, Entry, KeyPart};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understanding::t;
use crate::understood::Understood;
use indexmap::IndexMap;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.topics.";

static WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WORD"));
static ENDINGS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "ENDINGS"));  // t, 2 items
static STOPWORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STOPWORDS"));
static ASK_PART: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_PART"));
static ADVICE_ONLY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ADVICE_ONLY"));
static NAMES_THE_RUNNING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NAMES_THE_RUNNING"));
static SOMETHING_IS_BROKEN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SOMETHING_IS_BROKEN"));
static THEIR_OWN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "THEIR_OWN"));
static _SMALL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_SMALL"));
static NOT_THEIR_OWN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_THEIR_OWN"));
static PLACEHOLDER_PRONOUN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PLACEHOLDER_PRONOUN"));
static A_NEW_ONE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_NEW_ONE"));
static PET_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PET_WORD"));
static PET_ONE: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "PET_ONE"));
static BABY_OF: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "BABY_OF"));
static ASK_CUE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASK_CUE"));
static THING_ITSELF: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "THING_ITSELF"));
static PROCEED: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "PROCEED"));  // t, 7 items
static WORTH_DECIDING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WORTH_DECIDING"));
static WHAT_IT_COSTS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_IT_COSTS"));
static BREED_RE: LazyLock<&'static Re> = LazyLock::new(|| t::re("bot.understanding.breeds.", "BREED_RE"));

/// `words(text)`: the lower-cased words, possessive 's and apostrophes dropped.
pub fn words(text_: &str) -> Vec<String> {
    let low = text_.to_lowercase().replace('\u{2019}', "'");
    WORD.findall_str(&low)
        .into_iter()
        .map(|w| w.strip_suffix("'s").map(str::to_string).unwrap_or(w).replace('\'', ""))
        .filter(|w| !w.is_empty())
        .collect()
}

/// `stem(word)`: the word without a plural ending.
pub fn stem(word: &str) -> String {
    let n = text::len(word);
    if n > 4 && word.ends_with("ies") {
        return format!("{}y", &word[..word.len() - 3]);
    }
    if n > 3 && word.ends_with("es") && word[..word.len() - 2].ends_with(['s', 'x', 'z', 'h']) {
        return word[..word.len() - 2].to_string();
    }
    if n > 3 && word.ends_with('s') && !word.ends_with("ss") {
        return word[..word.len() - 1].to_string();
    }
    word.to_string()
}

/// `forms(word)`: the word and its roots.
pub fn forms(word: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    out.insert(word.to_string());
    for ending in ENDINGS.iter() {
        if text::len(word) > text::len(ending) + 2 && word.ends_with(ending.as_str()) {
            let root = &word[..word.len() - ending.len()];
            if text::len(root) >= 4 {
                out.insert(root.to_string());
            }
            out.insert(format!("{root}e"));
            let r: Vec<char> = root.chars().collect();
            if r.len() > 3 && r[r.len() - 1] == r[r.len() - 2] && !"aeiou".contains(r[r.len() - 1]) {
                out.insert(r[..r.len() - 1].iter().collect());
            }
        }
    }
    out
}

/// `keywords(text)`: the content words, stemmed, once each.
pub fn keywords(text_: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in words(text_) {
        if STOPWORDS.contains(&word) || text::len(&word) < 3 || crate::understanding::pyuni::is_digit(&word) {
            continue;
        }
        let key = stem(&word);
        if !out.contains(&key) {
            out.push(key);
        }
    }
    out
}

/// `memory.get(key)` for a key kept in `extra`.
fn mem<'a>(memory: &'a Memory, key: &str) -> Option<&'a Value> {
    memory.extra.get(key)
}

/// `current(memory)`: the topic record, when it has a text.
pub fn current(memory: &Memory) -> Option<&Value> {
    let record = mem(memory, "topic")?;
    if matches!(record, Value::Dict(_)) && record.get("text").is_some_and(Value::truthy) { Some(record) } else { None }
}

/// `_forms(word)`.
fn _forms(word: &str) -> Vec<String> {
    let n = text::len(word);
    let mut out = vec![word.to_string()];
    if n > 5 && word.ends_with("ing") {
        let base = &word[..word.len() - 3];
        out.push(base.to_string());
        out.push(format!("{base}e"));
    } else if n > 4 && word.ends_with("ed") {
        let base = &word[..word.len() - 2];
        out.push(base.to_string());
        out.push(format!("{base}e"));
    }
    out
}

/// `alike(one, other)`: two words that are forms of one another.
pub fn alike(one: &str, other: &str) -> bool {
    for left in _forms(one) {
        for right in _forms(other) {
            if left == right {
                return true;
            }
            let (ln, rn) = (text::len(&left), text::len(&right));
            if ln >= 4 && rn >= 4 && (left.starts_with(&right) || right.starts_with(&left)) {
                return true;
            }
            if ln >= 5 && rn >= 5 && text::slice(&left, 0, Some(5)) == text::slice(&right, 0, Some(5)) {
                return true;
            }
        }
    }
    false
}

/// The `topics` records (a list of dicts) in memory.
fn records(memory: &Memory) -> &[Value] {
    mem(memory, "topics").map(Value::items).unwrap_or(&[])
}

/// `find(memory, text)`: the latest topic record that shares a word with the text.
pub fn find<'a>(memory: &'a Memory, text_: &str) -> Option<&'a Value> {
    let wanted = keywords(text_);
    if wanted.is_empty() {
        return None;
    }
    for record in records(memory).iter().rev() {
        let have: Vec<String> = record.get("words").map(|w| w.strs()).unwrap_or_default();
        if wanted.iter().any(|w| have.iter().any(|o| alike(w, o))) {
            return Some(record);
        }
    }
    None
}

/// `_this_line(record, memory)`: the record was made on this turn.
fn _this_line(record: &Value, memory: &Memory) -> bool {
    record.get("turn").and_then(Value::as_int) == Some(memory.turns.len() as i64)
}

/// `_thing_of(memory, record)`: the record the record's `of` points back at.
fn _thing_of<'a>(memory: &'a Memory, record: &Value) -> Option<&'a Value> {
    if !matches!(record, Value::Dict(_)) {
        return None;
    }
    let back = record.get("of").map(value_str).unwrap_or_default();
    if back.is_empty() {
        return None;
    }
    records(memory).iter().rev().find(|o| matches!(o, Value::Dict(_)) && o.get("text").map(value_str).unwrap_or_default() == back)
}

/// `str(x or '')` for a value from a record.
fn value_str(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        Value::None => String::new(),
        other if !other.truthy() => String::new(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        Value::Bool(_) => "True".to_string(),
        other => format!("{other:?}"),
    }
}

/// `carried_topic(memory, text)`: the topic record the line carries on from.
pub fn carried_topic<'a>(memory: &'a Memory, text_: &str) -> Option<&'a Value> {
    let line = words(text_).join(" ");
    if let Some(Value::List(list)) = mem(memory, "topics") {
        for record in list.iter().rev() {
            if !matches!(record, Value::Dict(_)) {
                continue;
            }
            let rt = record.get("text").map(value_str).unwrap_or_default();
            if words(&rt).join(" ") != line {
                return _thing_of(memory, record).or(Some(record));
            }
        }
    }
    current(memory)
}

/// `key_subject(text, memory=None)`: the words that name the thing a line is about.
pub fn key_subject(text_: &str, memory: Option<&Memory>) -> Vec<String> {
    if let Some(memory) = memory {
        for record in records(memory).iter().rev() {
            if !matches!(record, Value::Dict(_)) || record.get("text").map(value_str).unwrap_or_default() != text_ {
                continue;
            }
            if _this_line(record, memory) {
                break;
            }
            if let Some(thing) = _thing_of(memory, record) {
                if let Some(found) = _key_subject(&thing.get("text").map(value_str).unwrap_or_default()) {
                    return found;
                }
            }
            break;
        }
    }
    if let Some(found) = _key_subject(text_) {
        return found;
    }
    if let Some(memory) = memory {
        for record in records(memory).iter().rev() {
            let t_ = if matches!(record, Value::Dict(_)) { record.get("text").map(value_str).unwrap_or_default() } else { String::new() };
            if let Some(found) = _key_subject(&t_) {
                return found;
            }
        }
    }
    keywords(text_).into_iter().take(3).collect()
}

/// `_key_subject(text)`: the key words of the entry the text reaches, as said.
fn _key_subject(text_: &str) -> Option<Vec<String>> {
    let found = words(text_);
    let item = _entry(text_)?;
    let mut out: Vec<String> = Vec::new();
    for part in &item.key {
        for alternative in lookup::alternatives(part) {
            if !_said(alternative, &found) {
                continue;
            }
            let part_words = words(alternative);
            if !part_words.iter().any(|w| ASK_PART.contains(w)) {
                out.extend(part_words);
            }
            break;
        }
    }
    if out.is_empty() { None } else { Some(out) }
}

/// `_said(phrase, found)`: the phrase's content words appear in order in the words.
fn _said(phrase: &str, found: &[String]) -> bool {
    let wanted: Vec<String> = words(phrase).into_iter().filter(|w| !STOPWORDS.contains(w)).map(|w| stem(&w)).collect();
    if wanted.is_empty() {
        return false;
    }
    let have: Vec<String> = found.iter().map(|w| stem(w)).collect();
    let n = wanted.len();
    if n > have.len() {
        return false;
    }
    (0..=have.len() - n).any(|i| have[i..i + n] == wanted[..])
}

/// `_entry(text)`: the knowledge entry a bare text reaches.
fn _entry(text_: &str) -> Option<&'static Entry> {
    let reading = Understood { clean: text_.to_string(), words: words(text_), ..Default::default() };
    lookup::find(&reading, None, None, false)
}

/// `_the_ask_with_its_doing(line, topic, last_id=None)`: "should i stop" + the doing the chat is
/// about ("should i stop running").
fn _the_ask_with_its_doing(line: &str, topic: &str, last_id: Option<&str>) -> Option<String> {
    if ADVICE_ONLY.match_(line).is_none() {
        return None;
    }
    let whole = _entry(topic);
    let said = words(line);
    for word in keywords(topic) {
        if said.contains(&word) {
            continue;
        }
        let tried = format!("{} {}", text::strip(line), word);
        let Some(found) = _entry(&tried) else { continue };
        if whole.is_some_and(|w| std::ptr::eq(w, found)) {
            continue;
        }
        if last_id.is_some_and(|l| !l.is_empty() && found.id == l) {
            continue;
        }
        return Some(tried);
    }
    None
}

/// The id of `memory["last_answer"]`, when it is a dict.
fn last_answer_id(memory: &Memory) -> Option<String> {
    let last = mem(memory, "last_answer")?;
    if !matches!(last, Value::Dict(_)) {
        return None;
    }
    last.get("id").filter(|v| v.truthy()).map(value_str)
}

/// The reading with `clean` and `words` taken from a filled line.
fn filled_reading(understood: &Understood, filled: &str) -> Understood {
    let mut reading = understood.clone();
    reading.clean = text::squash(&filled.to_lowercase());
    reading.words = words(&reading.clean);
    reading
}

/// `the_advice_ask_with_its_doing(understood, memory)`.
pub fn the_advice_ask_with_its_doing(understood: &Understood, memory: &Memory) -> Option<Understood> {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let line = text::strip(line);
    if line.is_empty() {
        return None;
    }
    let record = mem(memory, "topic")?;
    if !matches!(record, Value::Dict(_)) || !record.get("text").is_some_and(Value::truthy) {
        return None;
    }
    let last_id = last_answer_id(memory);
    let topic_text = record.get("text").map(value_str).unwrap_or_default();
    let doing = _the_ask_with_its_doing(line, &topic_text, last_id.as_deref())?;
    Some(filled_reading(understood, &doing))
}

/// `a_placeholder_pronoun(line)`: "it" that stands for nothing ("it's raining").
pub fn a_placeholder_pronoun(line: &str) -> bool {
    PLACEHOLDER_PRONOUN.is_match(line) && !A_NEW_ONE.is_match(line)
}

/// `the_thing_it_is(understood, memory)`: the reading with "it" filled in from the store, or None.
pub fn the_thing_it_is(understood: &Understood, memory: &Memory) -> Option<Understood> {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let line = text::strip(line);
    if line.is_empty() || !ASK_CUE.is_match(&line.to_lowercase()) || !THING_ITSELF.is_match(line) {
        return None;
    }
    let low = line.to_lowercase();
    if PROCEED.iter().any(|p| low.contains(p.as_str())) {
        return None;
    }
    if WORTH_DECIDING.is_match(line) || a_placeholder_pronoun(line) {
        return None;
    }
    if !WHAT_IT_COSTS.is_match(line) {
        let thing = the_pet_the_line_before_named(memory);
        if thing.is_empty() {
            return None;
        }
        let filled = THING_ITSELF.subn(&thing, line, 1);
        _entry(&filled)?;
        return Some(filled_reading(understood, &filled));
    }
    if _entry(line).is_some() {
        return None;
    }
    let last_id = last_answer_id(memory);
    for (thing, word) in _the_things_the_pronoun_stands_for(memory, line) {
        let filled = THING_ITSELF.subn(&thing, line, 1);
        let Some(entry) = _entry(&filled) else { continue };
        if !_the_entry_is_about_the_thing(entry, &word) {
            continue;
        }
        if !_the_entry_answers_the_ask(entry, &filled) {
            continue;
        }
        if last_id.as_deref().is_some_and(|l| !l.is_empty() && entry.id == l) {
            continue;
        }
        return Some(filled_reading(understood, &filled));
    }
    None
}

/// `_the_things_the_pronoun_stands_for(memory, line)`: (phrase, word) pairs, nearest first.
fn _the_things_the_pronoun_stands_for(memory: &Memory, line: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let pet = the_pet_the_line_before_named(memory);
    if !pet.is_empty() {
        let last = text::split_ref(&pet).last().unwrap().to_string();
        out.push((pet.clone(), last));
    }
    let record = carried_topic(memory, line);
    let said_text = record.and_then(|r| if matches!(r, Value::Dict(_)) { r.get("text").map(value_str) } else { None }).unwrap_or_default();
    let said_text = text::squash(&said_text.to_lowercase());
    if said_text.is_empty() || SOMETHING_IS_BROKEN.is_match(&said_text) {
        return out;
    }
    let said = format!(" {said_text}");
    let mine = _what_they_called_theirs(&said);
    if !mine.is_empty() {
        for pair in mine {
            if !out.contains(&pair) {
                out.push(pair);
            }
        }
        return out;
    }
    let line_words = words(line);
    for word in key_subject(&said_text, Some(memory)) {
        if line_words.contains(&word) {
            continue;
        }
        let mut phrase = word.clone();
        if said.contains(&format!(" my {word}")) || said_text.starts_with(&format!("my {word}")) {
            phrase = format!("my {word}");
        } else if said.contains(&format!(" the {word}")) || said_text.starts_with(&format!("the {word}")) {
            phrase = format!("the {word}");
        }
        let pair = (phrase, word);
        if !out.contains(&pair) {
            out.push(pair);
        }
    }
    out
}

/// `_what_they_called_theirs(text)`: "my car", "our boiler" said in the line.
fn _what_they_called_theirs(said: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for m in THEIR_OWN.finditer(said) {
        let (lead, word) = (m.str(1), m.str(2));
        if text::len(word) < 3 || _SMALL.contains(word) || NOT_THEIR_OWN.contains(word) {
            continue;
        }
        let pair = (format!("{} {}", lead.to_lowercase(), word), word.to_string());
        if !out.contains(&pair) {
            out.push(pair);
        }
    }
    out.truncate(3);
    out
}

/// `_the_entry_answers_the_ask(entry, line)`.
fn _the_entry_answers_the_ask(entry: &Entry, line: &str) -> bool {
    if entry.id.ends_with("running-cost") || entry.id.ends_with("running-costs") {
        return NAMES_THE_RUNNING.is_match(line);
    }
    true
}

/// `_the_entry_is_about_the_thing(entry, word)`: `word in group` for a key part (a substring of
/// a phrase, or one of a tuple's alternatives).
fn _the_entry_is_about_the_thing(entry: &Entry, word: &str) -> bool {
    entry.key.iter().any(|part| match part {
        KeyPart::Word(s) => s.contains(word),
        KeyPart::Any(v) => v.iter().any(|a| a == word),
    })
}

/// `the_pet_the_line_before_named(memory)`: "my dog" when the last line named the person's pet.
pub fn the_pet_the_line_before_named(memory: &Memory) -> String {
    let Some(before) = memory.turns.iter().rev().map(|t_| t_.user.clone()).find(|u| !text::strip(u).is_empty()) else { return String::new() };
    let mut named: IndexMap<String, String> = IndexMap::new();
    for word in PET_WORD.findall_str(&before) {
        let low = word.to_lowercase();
        let word = PET_ONE.get(&low).cloned().unwrap_or(low);
        let key = BABY_OF.get(&word).cloned().unwrap_or_else(|| word.clone());
        named.entry(key).or_insert(word);
    }
    let user = mem(memory, "user").filter(|u| matches!(u, Value::Dict(_)));
    let pets: &[Value] = user.and_then(|u| u.get("pets")).filter(|p| p.truthy()).map(Value::items).unwrap_or(&[]);
    let mut babies: HashSet<String> = HashSet::new();
    for pet in pets {
        if !matches!(pet, Value::Dict(_)) {
            continue;
        }
        let kind = text::strip(&pet.get("animal").map(value_str).unwrap_or_default()).to_lowercase().trim_end_matches('s').to_string();
        if let Some(b) = BABY_OF.get(&kind) {
            babies.insert(b.clone());
        }
    }
    for m in BREED_RE.finditer(&before) {
        if !crate::understanding::breeds::a_frame_before(&before, m.start()) {
            continue;
        }
        let kind = crate::understanding::breeds::the_kind_of(m.as_str());
        if !kind.is_empty() && !babies.contains(&kind) {
            named.entry(kind.clone()).or_insert(kind);
        }
    }
    if named.is_empty() {
        return String::new();
    }
    for pet in pets.iter().rev() {
        if !matches!(pet, Value::Dict(_)) {
            continue;
        }
        let Some(kind) = pet.get("animal").and_then(Value::as_str) else { continue };
        if text::strip(kind).is_empty() {
            continue;
        }
        let k = text::strip(kind).to_lowercase().trim_end_matches('s').to_string();
        let k = BABY_OF.get(&k).cloned().unwrap_or(k);
        if let Some(n) = named.get(&k) {
            return format!("my {n}");
        }
    }
    String::new()
}


// Stubs appended by harness/integrate_skeleton.py: the Python functions this file does not
// define yet, with signatures from the generated contract. Port them in place (move a
// function up into the file when you port it).
pub use self::skeleton_stubs::*;
#[allow(unused_imports, unused_variables, non_snake_case, dead_code, clippy::all)]
mod skeleton_stubs {
    use super::*;
    use crate::data::{self, Value};
    use crate::knowledge::entry::Entry;
    use crate::runtime::Candidate;
    use crate::state::{Memory, Turn};
    use crate::understood::Understood;

    /// Python: bot/memory/topics.py:212 `_what_they_are_not_doing(text)`
    pub fn _what_they_are_not_doing(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.topics:_what_they_are_not_doing")
    }

    /// Python: bot/memory/topics.py:248 `content(text)`
    pub fn content(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.topics:content")
    }

    /// Python: bot/memory/topics.py:274 `is_ack(text)`
    pub fn is_ack(text: &str) -> bool {
        crate::unported!("bot.memory.topics:is_ack")
    }

    /// Python: bot/memory/topics.py:287 `is_small_talk(text, kind=None)`
    pub fn is_small_talk(text: &str, kind: &str) -> bool {
        crate::unported!("bot.memory.topics:is_small_talk")
    }

    /// Python: bot/memory/topics.py:293 `_a_place_the_line_moves(text)`
    pub fn _a_place_the_line_moves(text: &str) -> String {
        crate::unported!("bot.memory.topics:_a_place_the_line_moves")
    }

    /// Python: bot/memory/topics.py:303 `_asks_about_the_bot(read)`
    pub fn _asks_about_the_bot(read: &Understood) -> bool {
        crate::unported!("bot.memory.topics:_asks_about_the_bot")
    }

    /// Python: bot/memory/topics.py:308 `_note(memory, text, kind=None)`
    pub fn _note(memory: &mut Memory, text: &str, kind: &str) -> Value {
        crate::unported!("bot.memory.topics:_note")
    }

    /// Python: bot/memory/topics.py:490 `_the_record_they_came_back_to(text, memory, record)`
    pub fn _the_record_they_came_back_to(text: &str, memory: &Memory, record: &Value) -> Value {
        crate::unported!("bot.memory.topics:_the_record_they_came_back_to")
    }

    /// Python: bot/memory/topics.py:525 `_back_to(memory, record, text)`
    pub fn _back_to(memory: &mut Memory, record: &Value, text: &str) -> Value {
        crate::unported!("bot.memory.topics:_back_to")
    }

    /// Python: bot/memory/topics.py:552 `_the_aim_without_its_deadline(text)`
    pub fn _the_aim_without_its_deadline(text: &str) -> String {
        crate::unported!("bot.memory.topics:_the_aim_without_its_deadline")
    }

    /// Python: bot/memory/topics.py:573 `_asks_which_kind(text, memory)`
    pub fn _asks_which_kind(text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:_asks_which_kind")
    }

    /// Python: bot/memory/topics.py:614 `_asks_an_aspect_of_the_thing(text, memory)`
    pub fn _asks_an_aspect_of_the_thing(text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:_asks_an_aspect_of_the_thing")
    }

    /// Python: bot/memory/topics.py:636 `_reaches_another_thing(text, memory)`
    pub fn _reaches_another_thing(text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:_reaches_another_thing")
    }

    /// Python: bot/memory/topics.py:660 `_names_a_thing_of_its_own(text, record, memory=None)`
    pub fn _names_a_thing_of_its_own(text: &str, record: &Value, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:_names_a_thing_of_its_own")
    }

    /// Python: bot/memory/topics.py:671 `_names_a_place_of_its_own(text, record, memory=None)`
    pub fn _names_a_place_of_its_own(text: &str, record: &Value, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:_names_a_place_of_its_own")
    }

    /// Python: bot/memory/topics.py:703 `_only_adds_a_person_to_the_thing(text, record)`
    pub fn _only_adds_a_person_to_the_thing(text: &str, record: &Value) -> bool {
        crate::unported!("bot.memory.topics:_only_adds_a_person_to_the_thing")
    }

    /// Python: bot/memory/topics.py:755 `_another_area(text, said)`
    pub fn _another_area(text: &str, said: &str) -> bool {
        crate::unported!("bot.memory.topics:_another_area")
    }

    /// Python: bot/memory/topics.py:772 `_a_detail_of_what_we_are_on(text, memory)`
    pub fn _a_detail_of_what_we_are_on(text: &str, memory: &Memory) {
        crate::unported!("bot.memory.topics:_a_detail_of_what_we_are_on")
    }

    /// Python: bot/memory/topics.py:830 `_a_part_of_the_record(read, record)`
    pub fn _a_part_of_the_record(read: &Understood, record: &Value) -> bool {
        crate::unported!("bot.memory.topics:_a_part_of_the_record")
    }

    /// Python: bot/memory/topics.py:846 `_names_another_subject(text, record, memory=None)`
    pub fn _names_another_subject(text: &str, record: &Value, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:_names_another_subject")
    }

    /// Python: bot/memory/topics.py:874 `_leans_back(text, record)`
    pub fn _leans_back(text: &str, record: &Value) -> bool {
        crate::unported!("bot.memory.topics:_leans_back")
    }

    /// Python: bot/memory/topics.py:894 `_points_at_the_record(text, record, memory=None)`
    pub fn _points_at_the_record(text: &str, record: &Value, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:_points_at_the_record")
    }

    /// Python: bot/memory/topics.py:935 `_leads_a_question(text)`
    pub fn _leads_a_question(text: &str) -> bool {
        crate::unported!("bot.memory.topics:_leads_a_question")
    }

    /// Python: bot/memory/topics.py:941 `_switches(text)`
    pub fn _switches(text: &str) -> bool {
        crate::unported!("bot.memory.topics:_switches")
    }

    /// Python: bot/memory/topics.py:960 `opens_by_switching(text)`
    pub fn opens_by_switching(text: &str) -> bool {
        crate::unported!("bot.memory.topics:opens_by_switching")
    }

    /// Python: bot/memory/topics.py:995 `_without_switchers(text)`
    pub fn _without_switchers(text: &str) -> String {
        crate::unported!("bot.memory.topics:_without_switchers")
    }

    /// Python: bot/memory/topics.py:1003 `_only_details(text, record)`
    pub fn _only_details(text: &str, record: &Value) -> bool {
        crate::unported!("bot.memory.topics:_only_details")
    }

    /// Python: bot/memory/topics.py:1043 `_the_plan_a_correction_renames(memory, text)`
    pub fn _the_plan_a_correction_renames(memory: &Memory, text: &str) {
        crate::unported!("bot.memory.topics:_the_plan_a_correction_renames")
    }

    /// Python: bot/memory/topics.py:1072 `_is_a_quantity(reading)`
    pub fn _is_a_quantity(reading: &Understood) -> bool {
        crate::unported!("bot.memory.topics:_is_a_quantity")
    }

    /// Python: bot/memory/topics.py:1104 `_is_a_when_or_how_often(text, memory)`
    pub fn _is_a_when_or_how_often(text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:_is_a_when_or_how_often")
    }

    /// Python: bot/memory/topics.py:1147 `_only_names_the_constraint(text, memory)`
    pub fn _only_names_the_constraint(text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:_only_names_the_constraint")
    }

    /// Python: bot/memory/topics.py:1178 `_adds_a_detail(text, memory)`
    pub fn _adds_a_detail(text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:_adds_a_detail")
    }

    /// Python: bot/memory/topics.py:1248 `person(text)`
    pub fn person(text: &str) -> Option<String> {
        crate::unported!("bot.memory.topics:person")
    }

    /// Python: bot/memory/topics.py:1269 `_named_inside_a_question(text)`
    pub fn _named_inside_a_question(text: &str) -> bool {
        crate::unported!("bot.memory.topics:_named_inside_a_question")
    }

    /// Python: bot/memory/topics.py:1277 `person_named(text, memory=None)`
    pub fn person_named(text: &str, memory: &Memory) -> Option<String> {
        crate::unported!("bot.memory.topics:person_named")
    }

    /// Python: bot/memory/topics.py:1297 `about(memory)`
    pub fn about(memory: &Memory) -> Value {
        crate::unported!("bot.memory.topics:about")
    }

    /// Python: bot/memory/topics.py:1493 `trouble(text)`
    pub fn trouble(text: &str) -> Option<String> {
        crate::unported!("bot.memory.topics:trouble")
    }

    /// Python: bot/memory/topics.py:1525 `_keep_person(memory, text)`
    pub fn _keep_person(memory: &mut Memory, text: &str) -> Value {
        crate::unported!("bot.memory.topics:_keep_person")
    }

    /// Python: bot/memory/topics.py:1551 `_after_an_emergency(memory)`
    pub fn _after_an_emergency(memory: &Value) -> bool {
        crate::unported!("bot.memory.topics:_after_an_emergency")
    }

    /// Python: bot/memory/topics.py:1574 `_after_a_correction(memory)`
    pub fn _after_a_correction(memory: &Value) -> bool {
        crate::unported!("bot.memory.topics:_after_a_correction")
    }

    /// Python: bot/memory/topics.py:1597 `in_a_safety_thread(memory)`
    pub fn in_a_safety_thread(memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:in_a_safety_thread")
    }

    /// Python: bot/memory/topics.py:1633 `_the_apostrophe_back(low)`
    pub fn _the_apostrophe_back(low: &str) -> String {
        crate::unported!("bot.memory.topics:_the_apostrophe_back")
    }

    /// Python: bot/memory/topics.py:1638 `return_target(text)`
    pub fn return_target(text: &str) -> String {
        crate::unported!("bot.memory.topics:return_target")
    }

    /// Python: bot/memory/topics.py:1698 `_a_question_after_the_thing(rest, split)` (never called on the trace corpus)
    pub fn _a_question_after_the_thing(rest: &str, split: &Value) -> bool {
        crate::unported!("bot.memory.topics:_a_question_after_the_thing")
    }

    /// Python: bot/memory/topics.py:1711 `_the_thing(rest)` (never called on the trace corpus)
    pub fn _the_thing(rest: &str) -> String {
        crate::unported!("bot.memory.topics:_the_thing")
    }

    /// Python: bot/memory/topics.py:1727 `_asks(text)` (never called on the trace corpus)
    pub fn _asks(text: &str) -> bool {
        crate::unported!("bot.memory.topics:_asks")
    }

    /// Python: bot/memory/topics.py:1733 `asks_after(line, thing)` (never called on the trace corpus)
    pub fn asks_after(line: &str, thing: &str) -> bool {
        crate::unported!("bot.memory.topics:asks_after")
    }

    /// Python: bot/memory/topics.py:1751 `the_thing_asked_again(memory, text='')`
    pub fn the_thing_asked_again(memory: &Memory, text: &str) -> String {
        crate::unported!("bot.memory.topics:the_thing_asked_again")
    }

    /// Python: bot/memory/topics.py:1778 `the_thing_went_back_to(memory)` (never called on the trace corpus)
    pub fn the_thing_went_back_to(memory: &Memory) -> String {
        crate::unported!("bot.memory.topics:the_thing_went_back_to")
    }

    /// Python: bot/memory/topics.py:1786 `_resume(memory, record, target, line='')` (never called on the trace corpus)
    pub fn _resume(memory: &mut Memory, record: &Value, target: &Value, line: &str) -> Value {
        crate::unported!("bot.memory.topics:_resume")
    }

    /// Python: bot/memory/topics.py:1812 `_the_answer_on_the_return(memory, said)` (never called on the trace corpus)
    pub fn _the_answer_on_the_return(memory: &mut Memory, said: &str) {
        crate::unported!("bot.memory.topics:_the_answer_on_the_return")
    }

    /// Python: bot/memory/topics.py:1855 `resumed(memory)`
    pub fn resumed(memory: &Memory) {
        crate::unported!("bot.memory.topics:resumed")
    }

    /// Python: bot/memory/topics.py:1925 `is_follow_up(text, memory)`
    pub fn is_follow_up(text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:is_follow_up")
    }

    /// Python: bot/memory/topics.py:1964 `context(memory, n=2)`
    pub fn context(memory: &Memory, n: i64) -> Vec<String> {
        crate::unported!("bot.memory.topics:context")
    }

    /// Python: bot/memory/topics.py:1972 `expand(text, memory)` (never called on the trace corpus)
    pub fn expand(text: &str, memory: &Memory) -> Value {
        crate::unported!("bot.memory.topics:expand")
    }

    /// Python: bot/memory/topics.py:2049 `_gerund(verb)` (never called on the trace corpus)
    pub fn _gerund(verb: &str) -> String {
        crate::unported!("bot.memory.topics:_gerund")
    }

    /// Python: bot/memory/topics.py:2116 `_strip_considering(source)`
    pub fn _strip_considering(source: &str) -> String {
        crate::unported!("bot.memory.topics:_strip_considering")
    }

    /// Python: bot/memory/topics.py:2138 `_only_a_handoff(head)`
    pub fn _only_a_handoff(head: &str) -> bool {
        crate::unported!("bot.memory.topics:_only_a_handoff")
    }

    /// Python: bot/memory/topics.py:2172 `_the_people()`
    pub fn _the_people() -> Vec<String> {
        crate::unported!("bot.memory.topics:_the_people")
    }

    /// Python: bot/memory/topics.py:2186 `_a_word_of_a_date(word)`
    pub fn _a_word_of_a_date(word: &str) -> bool {
        crate::unported!("bot.memory.topics:_a_word_of_a_date")
    }

    /// Python: bot/memory/topics.py:2201 `_only_a_date(clause)`
    pub fn _only_a_date(clause: &str) -> bool {
        crate::unported!("bot.memory.topics:_only_a_date")
    }

    /// Python: bot/memory/topics.py:2214 `_the_thing_after_a_date(text)`
    pub fn _the_thing_after_a_date(text: &str) -> String {
        crate::unported!("bot.memory.topics:_the_thing_after_a_date")
    }

    /// Python: bot/memory/topics.py:2234 `gist(text)`
    pub fn gist(text: &str) -> String {
        crate::unported!("bot.memory.topics:gist")
    }

    /// Python: bot/memory/topics.py:2354 `_room_limit(text)`
    pub fn _room_limit(text: &str) -> String {
        crate::unported!("bot.memory.topics:_room_limit")
    }

    /// Python: bot/memory/topics.py:2392 `_about_record(memory)`
    pub fn _about_record(memory: &Memory) -> Value {
        crate::unported!("bot.memory.topics:_about_record")
    }

    /// Python: bot/memory/topics.py:2415 `_names_nothing(record)`
    pub fn _names_nothing(record: &Value) -> bool {
        crate::unported!("bot.memory.topics:_names_nothing")
    }

    /// Python: bot/memory/topics.py:2422 `subject(memory)`
    pub fn subject(memory: &Memory) -> String {
        crate::unported!("bot.memory.topics:subject")
    }

    /// Python: bot/memory/topics.py:2446 `topic_text(memory)`
    pub fn topic_text(memory: &Memory) -> String {
        crate::unported!("bot.memory.topics:topic_text")
    }

    /// Python: bot/memory/topics.py:2493 `_the_action_word(line)` (never called on the trace corpus)
    pub fn _the_action_word(line: &str) -> Value {
        crate::unported!("bot.memory.topics:_the_action_word")
    }

    /// Python: bot/memory/topics.py:2515 `action(memory)` (never called on the trace corpus)
    pub fn action(memory: &Memory) -> String {
        crate::unported!("bot.memory.topics:action")
    }

    /// Python: bot/memory/topics.py:2536 `written_out(text, memory)`
    pub fn written_out(text: &str, memory: &Memory) {
        crate::unported!("bot.memory.topics:written_out")
    }

    /// Python: bot/memory/topics.py:2586 `_named_a_thing(line)` (never called on the trace corpus)
    pub fn _named_a_thing(line: &str) -> Value {
        crate::unported!("bot.memory.topics:_named_a_thing")
    }

    /// Python: bot/memory/topics.py:2600 `_the_thing_on_the_table(memory)` (never called on the trace corpus)
    pub fn _the_thing_on_the_table(memory: &Memory) -> Value {
        crate::unported!("bot.memory.topics:_the_thing_on_the_table")
    }

    /// Python: bot/memory/topics.py:2613 `attribute_of_the_thing(text, memory)`
    pub fn attribute_of_the_thing(text: &str, memory: &Memory) {
        crate::unported!("bot.memory.topics:attribute_of_the_thing")
    }

    /// Python: bot/memory/topics.py:2708 `_the_part_a_complaint_is_about(text)`
    pub fn _the_part_a_complaint_is_about(text: &str) {
        crate::unported!("bot.memory.topics:_the_part_a_complaint_is_about")
    }

    /// Python: bot/memory/topics.py:2724 `_the_cause_words(text)`
    pub fn _the_cause_words(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.topics:_the_cause_words")
    }

    /// Python: bot/memory/topics.py:2732 `_subject_words_to_carry(record, text, memory)`
    pub fn _subject_words_to_carry(record: &Value, text: &str, memory: &Memory) -> Vec<String> {
        crate::unported!("bot.memory.topics:_subject_words_to_carry")
    }

    /// Python: bot/memory/topics.py:2800 `_where_the_topic_lands(topic)`
    pub fn _where_the_topic_lands(topic: &str) -> &'static Entry {
        crate::unported!("bot.memory.topics:_where_the_topic_lands")
    }

    /// Python: bot/memory/topics.py:2833 `an_entry_for(text)` (never called on the trace corpus)
    pub fn an_entry_for(text: &str) -> Value {
        crate::unported!("bot.memory.topics:an_entry_for")
    }

    /// Python: bot/memory/topics.py:2943 `_the_last_question(memory)` (never called on the trace corpus)
    pub fn _the_last_question(memory: &Memory) -> Value {
        crate::unported!("bot.memory.topics:_the_last_question")
    }

    /// Python: bot/memory/topics.py:2952 `how_much_of_the_thing_in_hand(line, memory)`
    pub fn how_much_of_the_thing_in_hand(line: &str, memory: &Memory) -> String {
        crate::unported!("bot.memory.topics:how_much_of_the_thing_in_hand")
    }

    /// Python: bot/memory/topics.py:2996 `elided_ask(text)`
    pub fn elided_ask(text: &str) -> bool {
        crate::unported!("bot.memory.topics:elided_ask")
    }

    /// Python: bot/memory/topics.py:3016 `names_nothing(text)`
    pub fn names_nothing(text: &str) -> bool {
        crate::unported!("bot.memory.topics:names_nothing")
    }

    /// Python: bot/memory/topics.py:3026 `_content_words(text)`
    pub fn _content_words(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.topics:_content_words")
    }

    /// Python: bot/memory/topics.py:3053 `_the_topic_phrase(topic)`
    pub fn _the_topic_phrase(topic: &str) -> String {
        crate::unported!("bot.memory.topics:_the_topic_phrase")
    }

    /// Python: bot/memory/topics.py:3060 `_a_length_of_the_job(item)` (never called on the trace corpus)
    pub fn _a_length_of_the_job(item: &'static Entry) -> bool {
        crate::unported!("bot.memory.topics:_a_length_of_the_job")
    }

    /// Python: bot/memory/topics.py:3098 `the_city_of(topic)`
    pub fn the_city_of(topic: &str) {
        crate::unported!("bot.memory.topics:the_city_of")
    }

    /// Python: bot/memory/topics.py:3104 `the_country_of(topic)`
    pub fn the_country_of(topic: &str) {
        crate::unported!("bot.memory.topics:the_country_of")
    }

    /// Python: bot/memory/topics.py:3115 `asks_how_long_to_stay(line)`
    pub fn asks_how_long_to_stay(line: &str) -> bool {
        crate::unported!("bot.memory.topics:asks_how_long_to_stay")
    }

    /// Python: bot/memory/topics.py:3120 `going_there(place, topic)`
    pub fn going_there(place: &str, topic: &str) -> bool {
        crate::unported!("bot.memory.topics:going_there")
    }

    /// Python: bot/memory/topics.py:3136 `a_stay_with_the_place(line, topic)`
    pub fn a_stay_with_the_place(line: &str, topic: &str) {
        crate::unported!("bot.memory.topics:a_stay_with_the_place")
    }

    /// Python: bot/memory/topics.py:3161 `_the_ways_of_saying_it(stem_of, line, cands)`
    pub fn _the_ways_of_saying_it(stem_of: &str, line: &str, cands: &Value) -> Option<Vec<String>> {
        crate::unported!("bot.memory.topics:_the_ways_of_saying_it")
    }

    /// Python: bot/memory/topics.py:3196 `_the_last_answer(memory)`
    pub fn _the_last_answer(memory: &Memory) -> String {
        crate::unported!("bot.memory.topics:_the_last_answer")
    }

    /// Python: bot/memory/topics.py:3206 `_the_topics_on_the_chat(topic=None, memory=None)`
    pub fn _the_topics_on_the_chat(topic: &str, memory: &Memory) -> Vec<String> {
        crate::unported!("bot.memory.topics:_the_topics_on_the_chat")
    }

    /// Python: bot/memory/topics.py:3221 `_the_thing_a_measure_is_of(line, topic=None, memory=None)`
    pub fn _the_thing_a_measure_is_of(line: &str, topic: &str, memory: &Memory) {
        crate::unported!("bot.memory.topics:_the_thing_a_measure_is_of")
    }

    /// Python: bot/memory/topics.py:3238 `_with_the_word_before(line, word, thing)`
    pub fn _with_the_word_before(line: &str, word: &str, thing: &str) -> String {
        crate::unported!("bot.memory.topics:_with_the_word_before")
    }

    /// Python: bot/memory/topics.py:3247 `_how_they_feel(word)`
    pub fn _how_they_feel(word: &str) -> bool {
        crate::unported!("bot.memory.topics:_how_they_feel")
    }

    /// Python: bot/memory/topics.py:3265 `_keyed_to(keyed, word)` (never called on the trace corpus)
    pub fn _keyed_to(keyed: &Value, word: &str) -> bool {
        crate::unported!("bot.memory.topics:_keyed_to")
    }

    /// Python: bot/memory/topics.py:3273 `_a_measure_of_the_topic(line, topic=None, memory=None)`
    pub fn _a_measure_of_the_topic(line: &str, topic: &str, memory: &Memory) {
        crate::unported!("bot.memory.topics:_a_measure_of_the_topic")
    }

    /// Python: bot/memory/topics.py:3301 `carry_the_ask(line, topic, understood=None, memory=None)`
    pub fn carry_the_ask(line: &str, topic: &str, understood: &Understood, memory: &Memory) -> Option<String> {
        crate::unported!("bot.memory.topics:carry_the_ask")
    }

    /// Python: bot/memory/topics.py:3420 `_the_answer_covers_it(line, memory, understood=None)`
    pub fn _the_answer_covers_it(line: &str, memory: &Memory, understood: &Understood) -> bool {
        crate::unported!("bot.memory.topics:_the_answer_covers_it")
    }

    /// Python: bot/memory/topics.py:3456 `_with_the_article(line)` (never called on the trace corpus)
    pub fn _with_the_article(line: &str) -> Option<String> {
        crate::unported!("bot.memory.topics:_with_the_article")
    }

    /// Python: bot/memory/topics.py:3474 `_only_said_as_a_frequency(topic)`
    pub fn _only_said_as_a_frequency(topic: &str) -> Vec<String> {
        crate::unported!("bot.memory.topics:_only_said_as_a_frequency")
    }

    /// Python: bot/memory/topics.py:3484 `_asks_a_property_and_nothing_else(text)`
    pub fn _asks_a_property_and_nothing_else(text: &str) -> bool {
        crate::unported!("bot.memory.topics:_asks_a_property_and_nothing_else")
    }

    /// Python: bot/memory/topics.py:3498 `_carry_the_words(line, topic)`
    pub fn _carry_the_words(line: &str, topic: &str) -> Option<String> {
        crate::unported!("bot.memory.topics:_carry_the_words")
    }

    /// Python: bot/memory/topics.py:3669 `_an_elided_ask_situated(line, topic, understood=None)`
    pub fn _an_elided_ask_situated(line: &str, topic: &str, understood: &Understood) {
        crate::unported!("bot.memory.topics:_an_elided_ask_situated")
    }

    /// Python: bot/memory/topics.py:3741 `_the_same_answer_again(item, memory)` (never called on the trace corpus)
    pub fn _the_same_answer_again(item: &'static Entry, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:_the_same_answer_again")
    }

    /// Python: bot/memory/topics.py:3754 `the_thing_the_subject_brings(understood, memory)`
    pub fn the_thing_the_subject_brings(understood: &Understood, memory: &Memory) {
        crate::unported!("bot.memory.topics:the_thing_the_subject_brings")
    }

    /// Python: bot/memory/topics.py:3850 `follows_on(understood, text)`
    pub fn follows_on(understood: &Understood, text: &str) -> bool {
        crate::unported!("bot.memory.topics:follows_on")
    }

    /// Python: bot/memory/topics.py:3865 `keeps_away(line, topic, item=None)`
    pub fn keeps_away(line: &str, topic: &str, item: &'static Entry) -> bool {
        crate::unported!("bot.memory.topics:keeps_away")
    }

    /// Python: bot/memory/topics.py:3880 `_kept_away_from(phrase, said)`
    pub fn _kept_away_from(phrase: &str, said: &[String]) -> bool {
        crate::unported!("bot.memory.topics:_kept_away_from")
    }

    /// Python: bot/memory/topics.py:3897 `_names_it(found, item)`
    pub fn _names_it(found: &[String], item: &'static Entry) -> bool {
        crate::unported!("bot.memory.topics:_names_it")
    }

    /// Python: bot/memory/topics.py:3923 `_key_words_of(item)`
    pub fn _key_words_of(item: &'static Entry) -> Vec<String> {
        crate::unported!("bot.memory.topics:_key_words_of")
    }

    /// Python: bot/memory/topics.py:3933 `_names_it_as_written(found, item)` (never called on the trace corpus)
    pub fn _names_it_as_written(found: &Value, item: &'static Entry) -> bool {
        crate::unported!("bot.memory.topics:_names_it_as_written")
    }

    /// Python: bot/memory/topics.py:3952 `_asked_only(found, item)`
    pub fn _asked_only(found: &[String], item: &'static Entry) -> bool {
        crate::unported!("bot.memory.topics:_asked_only")
    }

    /// Python: bot/memory/topics.py:3971 `_a_sky_ask(line)`
    pub fn _a_sky_ask(line: &str) -> bool {
        crate::unported!("bot.memory.topics:_a_sky_ask")
    }

    /// Python: bot/memory/topics.py:3979 `_the_step_they_report(line)`
    pub fn _the_step_they_report(line: &str) -> bool {
        crate::unported!("bot.memory.topics:_the_step_they_report")
    }

    /// Python: bot/memory/topics.py:3995 `widen(line, subject_words, topic_text)`
    pub fn widen(line: &str, subject_words: &[String], topic_text: &str) -> Option<String> {
        crate::unported!("bot.memory.topics:widen")
    }

    /// Python: bot/memory/topics.py:4057 `adds_only_a_constraint(line)`
    pub fn adds_only_a_constraint(line: &str) {
        crate::unported!("bot.memory.topics:adds_only_a_constraint")
    }

    /// Python: bot/memory/topics.py:4069 `_constrains_the_ask(line, topic_text)`
    pub fn _constrains_the_ask(line: &str, topic_text: &str) {
        crate::unported!("bot.memory.topics:_constrains_the_ask")
    }

    /// Python: bot/memory/topics.py:4195 `_clauses_of(sentence)` (never called on the trace corpus)
    pub fn _clauses_of(sentence: &str) -> Vec<String> {
        crate::unported!("bot.memory.topics:_clauses_of")
    }

    /// Python: bot/memory/topics.py:4248 `remember_answer(memory, reply, keys)`
    pub fn remember_answer(memory: &mut Memory, reply: &str, keys: &[String]) {
        crate::unported!("bot.memory.topics:remember_answer")
    }

    /// Python: bot/memory/topics.py:4273 `entry_answer(memory, topic_text)`
    pub fn entry_answer(memory: &Memory, topic_text: &str) -> Option<String> {
        crate::unported!("bot.memory.topics:entry_answer")
    }

    /// Python: bot/memory/topics.py:4317 `answer_in_hand(memory)`
    pub fn answer_in_hand(memory: &Memory) -> Value {
        crate::unported!("bot.memory.topics:answer_in_hand")
    }

    /// Python: bot/memory/topics.py:4337 `newest_answer(memory)`
    pub fn newest_answer(memory: &Memory) -> Option<String> {
        crate::unported!("bot.memory.topics:newest_answer")
    }

    /// Python: bot/memory/topics.py:4379 `_carries(said, mine)`
    pub fn _carries(said: &str, mine: &str) -> bool {
        crate::unported!("bot.memory.topics:_carries")
    }

    /// Python: bot/memory/topics.py:4421 `_only_a_close(text)`
    pub fn _only_a_close(text: &str) -> bool {
        crate::unported!("bot.memory.topics:_only_a_close")
    }

    /// Python: bot/memory/topics.py:4437 `only_acknowledges(said)`
    pub fn only_acknowledges(said: &str) -> bool {
        crate::unported!("bot.memory.topics:only_acknowledges")
    }

    /// Python: bot/memory/topics.py:4459 `_hands_the_subject_back(said, memory)`
    pub fn _hands_the_subject_back(said: &str, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:_hands_the_subject_back")
    }

    /// Python: bot/memory/topics.py:4489 `_the_opening_of(reply)`
    pub fn _the_opening_of(reply: &str) -> String {
        crate::unported!("bot.memory.topics:_the_opening_of")
    }

    /// Python: bot/memory/topics.py:4514 `said_again(reply, memory, line=None)`
    pub fn said_again(reply: &str, memory: &Memory, line: &str) {
        crate::unported!("bot.memory.topics:said_again")
    }

    /// Python: bot/memory/topics.py:4579 `_said_later_than(turn, record)`
    pub fn _said_later_than(turn: &Value, record: &Value) -> bool {
        crate::unported!("bot.memory.topics:_said_later_than")
    }

    /// Python: bot/memory/topics.py:4590 `_the_turn_we_lean_on(understood, memory)`
    pub fn _the_turn_we_lean_on(understood: &Understood, memory: &Memory) -> Value {
        crate::unported!("bot.memory.topics:_the_turn_we_lean_on")
    }

    /// Python: bot/memory/topics.py:4615 `_the_turn_that_answered(answer, memory, keep=6)` (never called on the trace corpus)
    pub fn _the_turn_that_answered(answer: &str, memory: &Memory, keep: i64) -> Value {
        crate::unported!("bot.memory.topics:_the_turn_that_answered")
    }

    /// Python: bot/memory/topics.py:4634 `_answer_about(line, memory, keep=6)`
    pub fn _answer_about(line: &str, memory: &Memory, keep: i64) {
        crate::unported!("bot.memory.topics:_answer_about")
    }

    /// Python: bot/memory/topics.py:4668 `answer_about(line, memory)`
    pub fn answer_about(line: &str, memory: &Memory) {
        crate::unported!("bot.memory.topics:answer_about")
    }

    /// Python: bot/memory/topics.py:4679 `_the_trip_of_theirs(text)`
    pub fn _the_trip_of_theirs(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.topics:_the_trip_of_theirs")
    }

    /// Python: bot/memory/topics.py:4694 `the_place_they_are_on(memory)`
    pub fn the_place_they_are_on(memory: &Memory) -> Vec<String> {
        crate::unported!("bot.memory.topics:the_place_they_are_on")
    }

    /// Python: bot/memory/topics.py:4723 `about_a_place_they_left_behind(answer, memory)`
    pub fn about_a_place_they_left_behind(answer: &str, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:about_a_place_they_left_behind")
    }

    /// Python: bot/memory/topics.py:4748 `same_as_before(understood, memory)`
    pub fn same_as_before(understood: &Understood, memory: &Value) {
        crate::unported!("bot.memory.topics:same_as_before")
    }

    /// Python: bot/memory/topics.py:4990 `pairing(text)`
    pub fn pairing(text: &str) -> bool {
        crate::unported!("bot.memory.topics:pairing")
    }

    /// Python: bot/memory/topics.py:5012 `_doing_word(text)` (never called on the trace corpus)
    pub fn _doing_word(text: &str) -> Value {
        crate::unported!("bot.memory.topics:_doing_word")
    }

    /// Python: bot/memory/topics.py:5022 `_opening_instruction(sentence, allow_pointer)` (never called on the trace corpus)
    pub fn _opening_instruction(sentence: &str, allow_pointer: &Value) -> Value {
        crate::unported!("bot.memory.topics:_opening_instruction")
    }

    /// Python: bot/memory/topics.py:5057 `_doing_part(sentences, memory)` (never called on the trace corpus)
    pub fn _doing_part(sentences: &Value, memory: &Memory) -> Value {
        crate::unported!("bot.memory.topics:_doing_part")
    }

    /// Python: bot/memory/topics.py:5088 `_says_it_again(part, sentence)` (never called on the trace corpus)
    pub fn _says_it_again(part: &str, sentence: &str) -> bool {
        crate::unported!("bot.memory.topics:_says_it_again")
    }

    /// Python: bot/memory/topics.py:5118 `_the_rates_in(text)`
    pub fn _the_rates_in(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.topics:_the_rates_in")
    }

    /// Python: bot/memory/topics.py:5132 `_a_rate_that_fits(line, sentence)` (never called on the trace corpus)
    pub fn _a_rate_that_fits(line: &str, sentence: &str) -> bool {
        crate::unported!("bot.memory.topics:_a_rate_that_fits")
    }

    /// Python: bot/memory/topics.py:5147 `_a_comma_that_ends_a_clause(sentence, start)` (never called on the trace corpus)
    pub fn _a_comma_that_ends_a_clause(sentence: &str, start: i64) -> Value {
        crate::unported!("bot.memory.topics:_a_comma_that_ends_a_clause")
    }

    /// Python: bot/memory/topics.py:5164 `_pick_amount(sentences, memory, line='')`
    pub fn _pick_amount(sentences: &[String], memory: &Memory, line: &str) {
        crate::unported!("bot.memory.topics:_pick_amount")
    }

    /// Python: bot/memory/topics.py:5221 `_pick_sentence(line, understood, answer, memory, strict=False)`
    pub fn _pick_sentence(line: &str, understood: &Understood, answer: &str, memory: &Memory, strict: bool) -> Option<String> {
        crate::unported!("bot.memory.topics:_pick_sentence")
    }

    /// Python: bot/memory/topics.py:5226 `said_on_its_own(sentence)`
    pub fn said_on_its_own(sentence: Option<&str>) -> Option<String> {
        crate::unported!("bot.memory.topics:said_on_its_own")
    }

    /// Python: bot/memory/topics.py:5245 `_the_sentence_it_answers(line, understood, answer, memory, strict=False)`
    pub fn _the_sentence_it_answers(line: &str, understood: &Understood, answer: &str, memory: &Memory, strict: bool) -> Option<String> {
        crate::unported!("bot.memory.topics:_the_sentence_it_answers")
    }

    /// Python: bot/memory/topics.py:5363 `_time_of(sentence)` (never called on the trace corpus)
    pub fn _time_of(sentence: &str) -> Value {
        crate::unported!("bot.memory.topics:_time_of")
    }

    /// Python: bot/memory/topics.py:5390 `_option_range(sentence)` (never called on the trace corpus)
    pub fn _option_range(sentence: &str) -> String {
        crate::unported!("bot.memory.topics:_option_range")
    }

    /// Python: bot/memory/topics.py:5425 `_quoted(part)` (never called on the trace corpus)
    pub fn _quoted(part: &str) -> i64 {
        crate::unported!("bot.memory.topics:_quoted")
    }

    /// Python: bot/memory/topics.py:5439 `_the_option_for_the_topic(sentence, line, memory)` (never called on the trace corpus)
    pub fn _the_option_for_the_topic(sentence: &str, line: &str, memory: &Memory) -> Option<String> {
        crate::unported!("bot.memory.topics:_the_option_for_the_topic")
    }

    /// Python: bot/memory/topics.py:5483 `_only_a_marker(part)` (never called on the trace corpus)
    pub fn _only_a_marker(part: &str) -> bool {
        crate::unported!("bot.memory.topics:_only_a_marker")
    }

    /// Python: bot/memory/topics.py:5491 `_duration_clauses(sentence)` (never called on the trace corpus)
    pub fn _duration_clauses(sentence: &str) -> Value {
        crate::unported!("bot.memory.topics:_duration_clauses")
    }

    /// Python: bot/memory/topics.py:5516 `_only_a_cadence(part)` (never called on the trace corpus)
    pub fn _only_a_cadence(part: &str) -> bool {
        crate::unported!("bot.memory.topics:_only_a_cadence")
    }

    /// Python: bot/memory/topics.py:5536 `_a_window_in_the_weather(part)` (never called on the trace corpus)
    pub fn _a_window_in_the_weather(part: &str) -> bool {
        crate::unported!("bot.memory.topics:_a_window_in_the_weather")
    }

    /// Python: bot/memory/topics.py:5548 `_not_a_length(part)` (never called on the trace corpus)
    pub fn _not_a_length(part: &str) -> bool {
        crate::unported!("bot.memory.topics:_not_a_length")
    }

    /// Python: bot/memory/topics.py:5560 `_duration_clause(sentence)` (never called on the trace corpus)
    pub fn _duration_clause(sentence: &str) -> Value {
        crate::unported!("bot.memory.topics:_duration_clause")
    }

    /// Python: bot/memory/topics.py:5566 `_shorten(part)` (never called on the trace corpus)
    pub fn _shorten(part: &str) -> Value {
        crate::unported!("bot.memory.topics:_shorten")
    }

    /// Python: bot/memory/topics.py:5600 `_asked_period(line)` (never called on the trace corpus)
    pub fn _asked_period(line: &str) -> Value {
        crate::unported!("bot.memory.topics:_asked_period")
    }

    /// Python: bot/memory/topics.py:5618 `_in_period(part, period)` (never called on the trace corpus)
    pub fn _in_period(part: &str, period: &Value) -> bool {
        crate::unported!("bot.memory.topics:_in_period")
    }

    /// Python: bot/memory/topics.py:5638 `_pick_duration(sentences, wanted, memory, line='')` (never called on the trace corpus)
    pub fn _pick_duration(sentences: &Value, wanted: &Value, memory: &Memory, line: &str) -> Value {
        crate::unported!("bot.memory.topics:_pick_duration")
    }

    /// Python: bot/memory/topics.py:5713 `_step_they_asked_for(parts, wanted)` (never called on the trace corpus)
    pub fn _step_they_asked_for(parts: &[String], wanted: &Value) -> Value {
        crate::unported!("bot.memory.topics:_step_they_asked_for")
    }

    /// Python: bot/memory/topics.py:5726 `_time_in_words(minutes)` (never called on the trace corpus)
    pub fn _time_in_words(minutes: &Value) -> Value {
        crate::unported!("bot.memory.topics:_time_in_words")
    }

    /// Python: bot/memory/topics.py:5736 `time_for_amount(answer, line)` (never called on the trace corpus)
    pub fn time_for_amount(answer: &str, line: &str) -> Value {
        crate::unported!("bot.memory.topics:time_for_amount")
    }

    /// Python: bot/memory/topics.py:5766 `_already_for_this_size(line, answer)`
    pub fn _already_for_this_size(line: &str, answer: &str) -> bool {
        crate::unported!("bot.memory.topics:_already_for_this_size")
    }

    /// Python: bot/memory/topics.py:5780 `_in_grams(text)` (never called on the trace corpus)
    pub fn _in_grams(text: &str) -> Value {
        crate::unported!("bot.memory.topics:_in_grams")
    }

    /// Python: bot/memory/topics.py:5789 `_names_another_thing(line, answer, about=())`
    pub fn _names_another_thing(line: &str, answer: &str, about: &[String]) -> bool {
        crate::unported!("bot.memory.topics:_names_another_thing")
    }

    /// Python: bot/memory/topics.py:5806 `sized_reply(answer, line, memory, about=())`
    pub fn sized_reply(answer: &str, line: &str, memory: &Memory, about: &[String]) {
        crate::unported!("bot.memory.topics:sized_reply")
    }

    /// Python: bot/memory/topics.py:5820 `_sized_reply(answer, line, memory)`
    pub fn _sized_reply(answer: &str, line: &str, memory: &Memory) {
        crate::unported!("bot.memory.topics:_sized_reply")
    }

    /// Python: bot/memory/topics.py:5845 `_times_asked(line, memory)` (never called on the trace corpus)
    pub fn _times_asked(line: &str, memory: &Memory) -> Value {
        crate::unported!("bot.memory.topics:_times_asked")
    }

    /// Python: bot/memory/topics.py:5858 `_rate_said(answer)` (never called on the trace corpus)
    pub fn _rate_said(answer: &str) -> Value {
        crate::unported!("bot.memory.topics:_rate_said")
    }

    /// Python: bot/memory/topics.py:5869 `_phrase(text)` (never called on the trace corpus)
    pub fn _phrase(text: &str) -> String {
        crate::unported!("bot.memory.topics:_phrase")
    }

    /// Python: bot/memory/topics.py:5875 `has_time(text)`
    pub fn has_time(text: &str) -> bool {
        crate::unported!("bot.memory.topics:has_time")
    }

    /// Python: bot/memory/topics.py:5880 `_worked_out_for_this_line(understood, line, answer)`
    pub fn _worked_out_for_this_line(understood: &Understood, line: &str, answer: &str) {
        crate::unported!("bot.memory.topics:_worked_out_for_this_line")
    }

    /// Python: bot/memory/topics.py:5905 `asks_only(understood)`
    pub fn asks_only(understood: &Understood) -> bool {
        crate::unported!("bot.memory.topics:asks_only")
    }

    /// Python: bot/memory/topics.py:5927 `_earlier_subject_sentence(line, understood, memory, skip, turns_back=2)`
    pub fn _earlier_subject_sentence(line: &str, understood: &Understood, memory: &Memory, skip: &Value, turns_back: i64) {
        crate::unported!("bot.memory.topics:_earlier_subject_sentence")
    }

    /// Python: bot/memory/topics.py:5952 `_repeats_the_question(line, asked)`
    pub fn _repeats_the_question(line: &str, asked: &str) -> bool {
        crate::unported!("bot.memory.topics:_repeats_the_question")
    }

    /// Python: bot/memory/topics.py:5961 `already_said(memory, text, turns_back=3)`
    pub fn already_said(memory: &Memory, text: Option<&str>, turns_back: i64) -> bool {
        crate::unported!("bot.memory.topics:already_said")
    }

    /// Python: bot/memory/topics.py:5981 `_names_something_new(line, answer)`
    pub fn _names_something_new(line: &str, answer: &str) -> bool {
        crate::unported!("bot.memory.topics:_names_something_new")
    }

    /// Python: bot/memory/topics.py:6002 `_named_a_fresh_thing(text, record, answer='')`
    pub fn _named_a_fresh_thing(text: &str, record: &Value, answer: &str) -> bool {
        crate::unported!("bot.memory.topics:_named_a_fresh_thing")
    }

    /// Python: bot/memory/topics.py:6025 `_the_thing_the_chat_moved_to(whole, line, memory, keep=MOVED_KEEP)` (never called on the trace corpus)
    pub fn _the_thing_the_chat_moved_to(whole: &Value, line: &str, memory: &Memory, keep: i64) -> Value {
        crate::unported!("bot.memory.topics:_the_thing_the_chat_moved_to")
    }

    /// Python: bot/memory/topics.py:6070 `left_for_a_blank(memory, line)`
    pub fn left_for_a_blank(memory: &Value, line: &str) -> bool {
        crate::unported!("bot.memory.topics:left_for_a_blank")
    }

    /// Python: bot/memory/topics.py:6124 `_a_question(text)`
    pub fn _a_question(text: &str) -> bool {
        crate::unported!("bot.memory.topics:_a_question")
    }

    /// Python: bot/memory/topics.py:6139 `bare_question(understood, memory)`
    pub fn bare_question(understood: &Understood, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:bare_question")
    }

    /// Python: bot/memory/topics.py:6183 `_still_the_topic(record, memory)`
    pub fn _still_the_topic(record: &Value, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:_still_the_topic")
    }

    /// Python: bot/memory/topics.py:6224 `leans_on_topic(understood, memory)`
    pub fn leans_on_topic(understood: &Understood, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:leans_on_topic")
    }

    /// Python: bot/memory/topics.py:6272 `_only_the_topics_own_words(text, record)`
    pub fn _only_the_topics_own_words(text: &str, record: &Value) -> bool {
        crate::unported!("bot.memory.topics:_only_the_topics_own_words")
    }

    /// Python: bot/memory/topics.py:6289 `next_slot_reading(understood, memory)`
    pub fn next_slot_reading(understood: &Understood, memory: &Value) {
        crate::unported!("bot.memory.topics:next_slot_reading")
    }

    /// Python: bot/memory/topics.py:6352 `_introduction_covered(new, old)` (never called on the trace corpus)
    pub fn _introduction_covered(new: &Value, old: &Value) -> bool {
        crate::unported!("bot.memory.topics:_introduction_covered")
    }

    /// Python: bot/memory/topics.py:6367 `introduction_question(text, memory)`
    pub fn introduction_question(text: &str, memory: &Memory) -> String {
        crate::unported!("bot.memory.topics:introduction_question")
    }

    /// Python: bot/memory/topics.py:6462 `the_weather_ask(understood)`
    pub fn the_weather_ask(understood: &Understood) -> bool {
        crate::unported!("bot.memory.topics:the_weather_ask")
    }

    /// Python: bot/memory/topics.py:6478 `the_place_and_month(text)`
    pub fn the_place_and_month(text: &str) -> Value {
        crate::unported!("bot.memory.topics:the_place_and_month")
    }

    /// Python: bot/memory/topics.py:6507 `_the_period_in(text)`
    pub fn _the_period_in(text: &str) -> String {
        crate::unported!("bot.memory.topics:_the_period_in")
    }

    /// Python: bot/memory/topics.py:6527 `_a_period_and_nothing_else(line)`
    pub fn _a_period_and_nothing_else(line: &str) -> bool {
        crate::unported!("bot.memory.topics:_a_period_and_nothing_else")
    }

    /// Python: bot/memory/topics.py:6535 `the_period_they_ask_about(understood, memory)`
    pub fn the_period_they_ask_about(understood: &Understood, memory: &Value) {
        crate::unported!("bot.memory.topics:the_period_they_ask_about")
    }

    /// Python: bot/memory/topics.py:6571 `_the_answer_was_about_the_weather(memory)`
    pub fn _the_answer_was_about_the_weather(memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:_the_answer_was_about_the_weather")
    }

    /// Python: bot/memory/topics.py:6583 `_the_place_the_chat_is_on(line, memory)` (never called on the trace corpus)
    pub fn _the_place_the_chat_is_on(line: &str, memory: &Memory) -> Value {
        crate::unported!("bot.memory.topics:_the_place_the_chat_is_on")
    }

    /// Python: bot/memory/topics.py:6607 `_with_the_period(asked, was, now)` (never called on the trace corpus)
    pub fn _with_the_period(asked: &str, was: &str, now: &Value) -> String {
        crate::unported!("bot.memory.topics:_with_the_period")
    }

    /// Python: bot/memory/topics.py:6628 `_the_meal_in(text)`
    pub fn _the_meal_in(text: &str) -> String {
        crate::unported!("bot.memory.topics:_the_meal_in")
    }

    /// Python: bot/memory/topics.py:6637 `_a_meal_and_nothing_else(line)`
    pub fn _a_meal_and_nothing_else(line: &str) -> bool {
        crate::unported!("bot.memory.topics:_a_meal_and_nothing_else")
    }

    /// Python: bot/memory/topics.py:6645 `the_meal_they_ask_about(understood, memory)`
    pub fn the_meal_they_ask_about(understood: &Understood, memory: &Memory) {
        crate::unported!("bot.memory.topics:the_meal_they_ask_about")
    }

    /// Python: bot/memory/topics.py:6675 `meal_follow_up(understood, memory)`
    pub fn meal_follow_up(understood: &Understood, memory: &Memory) {
        crate::unported!("bot.memory.topics:meal_follow_up")
    }

    /// Python: bot/memory/topics.py:6693 `_with_the_meal(asked, was, now)` (never called on the trace corpus)
    pub fn _with_the_meal(asked: &str, was: &str, now: &Value) -> String {
        crate::unported!("bot.memory.topics:_with_the_meal")
    }

    /// Python: bot/memory/topics.py:6699 `the_weather_of_the_trip(understood, memory)`
    pub fn the_weather_of_the_trip(understood: &Understood, memory: &Value) {
        crate::unported!("bot.memory.topics:the_weather_of_the_trip")
    }

    /// Python: bot/memory/topics.py:6728 `_the_trip_said(line, memory)` (never called on the trace corpus)
    pub fn _the_trip_said(line: &str, memory: &Memory) -> Value {
        crate::unported!("bot.memory.topics:_the_trip_said")
    }

    /// Python: bot/memory/topics.py:6755 `_the_lines_said(memory)` (never called on the trace corpus)
    pub fn _the_lines_said(memory: &Memory) -> Value {
        crate::unported!("bot.memory.topics:_the_lines_said")
    }

    /// Python: bot/memory/topics.py:6814 `the_ask_about_the_thing_it_points_at(understood, memory)`
    pub fn the_ask_about_the_thing_it_points_at(understood: &Understood, memory: &Value) {
        crate::unported!("bot.memory.topics:the_ask_about_the_thing_it_points_at")
    }

    /// Python: bot/memory/topics.py:6867 `the_pronoun_written_as_what_it_stands_for(understood, memory)`
    pub fn the_pronoun_written_as_what_it_stands_for(understood: &Understood, memory: &Value) {
        crate::unported!("bot.memory.topics:the_pronoun_written_as_what_it_stands_for")
    }

    /// Python: bot/memory/topics.py:6948 `_the_thing_with_a_determiner(thing)`
    pub fn _the_thing_with_a_determiner(thing: &str) -> Vec<String> {
        crate::unported!("bot.memory.topics:_the_thing_with_a_determiner")
    }

    /// Python: bot/memory/topics.py:7177 `the_prognosis_the_line_asks(understood, memory)`
    pub fn the_prognosis_the_line_asks(understood: &Understood, memory: &Value) {
        crate::unported!("bot.memory.topics:the_prognosis_the_line_asks")
    }

    /// Python: bot/memory/topics.py:7264 `_keyed_on_how_long(entry)` (never called on the trace corpus)
    pub fn _keyed_on_how_long(entry: &'static Entry) -> bool {
        crate::unported!("bot.memory.topics:_keyed_on_how_long")
    }

    /// Python: bot/memory/topics.py:7270 `_the_thing_as_a_question_names_it(named)` (never called on the trace corpus)
    pub fn _the_thing_as_a_question_names_it(named: &Value) -> Value {
        crate::unported!("bot.memory.topics:_the_thing_as_a_question_names_it")
    }

    /// Python: bot/memory/topics.py:7282 `the_property_of_the_thing(understood, memory)`
    pub fn the_property_of_the_thing(understood: &Understood, memory: &Value) {
        crate::unported!("bot.memory.topics:the_property_of_the_thing")
    }

    /// Python: bot/memory/topics.py:7381 `the_limit_of_the_thing(understood, memory)`
    pub fn the_limit_of_the_thing(understood: &Understood, memory: &Value) {
        crate::unported!("bot.memory.topics:the_limit_of_the_thing")
    }

    /// Python: bot/memory/topics.py:7434 `_the_limit_written_out(line, thing, memory)` (never called on the trace corpus)
    pub fn _the_limit_written_out(line: &str, thing: &str, memory: &Memory) -> Value {
        crate::unported!("bot.memory.topics:_the_limit_written_out")
    }

    /// Python: bot/memory/topics.py:7456 `_a_doing_word(word)` (never called on the trace corpus)
    pub fn _a_doing_word(word: &str) -> bool {
        crate::unported!("bot.memory.topics:_a_doing_word")
    }

    /// Python: bot/memory/topics.py:7462 `_fills_from_the_things(line, understood, thing, memory)` (never called on the trace corpus)
    pub fn _fills_from_the_things(line: &str, understood: &Understood, thing: &str, memory: &Memory) -> Value {
        crate::unported!("bot.memory.topics:_fills_from_the_things")
    }

    /// Python: bot/memory/topics.py:7487 `_fills(line, thing)` (never called on the trace corpus)
    pub fn _fills(line: &str, thing: &str) -> Value {
        crate::unported!("bot.memory.topics:_fills")
    }

    /// Python: bot/memory/topics.py:7528 `the_airport_there_is(understood, memory)`
    pub fn the_airport_there_is(understood: &Understood, memory: &Value) {
        crate::unported!("bot.memory.topics:the_airport_there_is")
    }

    /// Python: bot/memory/topics.py:7562 `the_animal_the_subject_is(understood, memory)`
    pub fn the_animal_the_subject_is(understood: Option<&Understood>, memory: &Memory) {
        crate::unported!("bot.memory.topics:the_animal_the_subject_is")
    }

    /// Python: bot/memory/topics.py:7617 `_the_kind_of_animal(text)` (never called on the trace corpus)
    pub fn _the_kind_of_animal(text: &str) -> Value {
        crate::unported!("bot.memory.topics:_the_kind_of_animal")
    }

    /// Python: bot/memory/topics.py:7626 `answers_about_another_creature(answer, what_the_line_is)` (never called on the trace corpus)
    pub fn answers_about_another_creature(answer: &str, what_the_line_is: &Value) -> bool {
        crate::unported!("bot.memory.topics:answers_about_another_creature")
    }

    /// Python: bot/memory/topics.py:7651 `_called(who)` (never called on the trace corpus)
    pub fn _called(who: &str) -> Value {
        crate::unported!("bot.memory.topics:_called")
    }

    /// Python: bot/memory/topics.py:7661 `_the_pet_the_turns_before_named(memory, back=3)`
    pub fn _the_pet_the_turns_before_named(memory: &Memory, back: i64) -> String {
        crate::unported!("bot.memory.topics:_the_pet_the_turns_before_named")
    }

    /// Python: bot/memory/topics.py:7689 `a_detail_of_the_thing_we_hold(understood, memory)`
    pub fn a_detail_of_the_thing_we_hold(understood: &Understood, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:a_detail_of_the_thing_we_hold")
    }

    /// Python: bot/memory/topics.py:7698 `_the_kind_the_line_leaves_out(understood, memory)`
    pub fn _the_kind_the_line_leaves_out(understood: &Understood, memory: &Memory) {
        crate::unported!("bot.memory.topics:_the_kind_the_line_leaves_out")
    }

    /// Python: bot/memory/topics.py:7717 `the_thing_a_detail_belongs_to(understood, memory)`
    pub fn the_thing_a_detail_belongs_to(understood: &Understood, memory: &Memory) {
        crate::unported!("bot.memory.topics:the_thing_a_detail_belongs_to")
    }

    /// Python: bot/memory/topics.py:7860 `_carrying_on(understood, memory)`
    pub fn _carrying_on(understood: &Understood, memory: &Value) {
        crate::unported!("bot.memory.topics:_carrying_on")
    }

    /// Python: bot/memory/topics.py:7891 `_what_the_turn_took_away(record, memory)`
    pub fn _what_the_turn_took_away(record: &Value, memory: &Memory) -> Vec<String> {
        crate::unported!("bot.memory.topics:_what_the_turn_took_away")
    }

    /// Python: bot/memory/topics.py:7913 `_widened(line, understood, record, memory)`
    pub fn _widened(line: &str, understood: &Understood, record: &Value, memory: &Memory) -> Option<String> {
        crate::unported!("bot.memory.topics:_widened")
    }

    /// Python: bot/memory/topics.py:7946 `_the_length_of_the_aim(understood, memory)`
    pub fn _the_length_of_the_aim(understood: &Understood, memory: &Memory) {
        crate::unported!("bot.memory.topics:_the_length_of_the_aim")
    }

    /// Python: bot/memory/topics.py:7994 `_a_gathering_read_as_the_food(understood, line, memory)`
    pub fn _a_gathering_read_as_the_food(understood: &Understood, line: &str, memory: &Value) {
        crate::unported!("bot.memory.topics:_a_gathering_read_as_the_food")
    }

    /// Python: bot/memory/topics.py:8024 `_the_whole_topic(line, record_text, memory=None)`
    pub fn _the_whole_topic(line: &str, record_text: &str, memory: &Memory) -> Option<String> {
        crate::unported!("bot.memory.topics:_the_whole_topic")
    }

    /// Python: bot/memory/topics.py:8081 `_the_answer_is_about_what_the_line_names(entry, line)`
    pub fn _the_answer_is_about_what_the_line_names(entry: &'static Entry, line: &str) -> bool {
        crate::unported!("bot.memory.topics:_the_answer_is_about_what_the_line_names")
    }

    /// Python: bot/memory/topics.py:8110 `_already_answered_here(entry, memory)`
    pub fn _already_answered_here(entry: &'static Entry, memory: &Memory) -> bool {
        crate::unported!("bot.memory.topics:_already_answered_here")
    }

    /// Python: bot/memory/topics.py:8125 `_how_much_of_the_thing_the_entry_uses(entry, said)`
    pub fn _how_much_of_the_thing_the_entry_uses(entry: &'static Entry, said: &str) -> i64 {
        crate::unported!("bot.memory.topics:_how_much_of_the_thing_the_entry_uses")
    }

    /// Python: bot/memory/topics.py:8132 `_the_things_the_chat_named(record_text, memory, line='')`
    pub fn _the_things_the_chat_named(record_text: &str, memory: &Memory, line: &str) -> Option<String> {
        crate::unported!("bot.memory.topics:_the_things_the_chat_named")
    }

    /// Python: bot/memory/topics.py:8161 `followup_reading(understood, memory)`
    pub fn followup_reading(understood: &Understood, memory: &Value) -> Understood {
        crate::unported!("bot.memory.topics:followup_reading")
    }

    /// Python: bot/memory/topics.py:8393 `note(memory, text, kind=None)`
    pub fn note(memory: &mut Memory, text: &str, kind: &str) -> Value {
        crate::unported!("bot.memory.topics:note")
    }

    /// Python: bot/memory/topics.py:8410 `_the_reply_to_the_turn_before(memory)`
    pub fn _the_reply_to_the_turn_before(memory: &Memory) -> String {
        crate::unported!("bot.memory.topics:_the_reply_to_the_turn_before")
    }

    /// Python: bot/memory/topics.py:8418 `_the_step_back_is_undone(memory)`
    pub fn _the_step_back_is_undone(memory: &mut Memory) {
        crate::unported!("bot.memory.topics:_the_step_back_is_undone")
    }

    /// Python: bot/memory/topics.py:8436 `_the_words_before_the_turn(memory)`
    pub fn _the_words_before_the_turn(memory: &Memory) -> Value {
        crate::unported!("bot.memory.topics:_the_words_before_the_turn")
    }

    /// Python: bot/memory/topics.py:8444 `_the_records(memory)`
    pub fn _the_records(memory: &Memory) {
        crate::unported!("bot.memory.topics:_the_records")
    }

    /// Python: bot/memory/topics.py:8450 `_the_words_this_turn(memory, before, record, previous)`
    pub fn _the_words_this_turn(memory: &mut Memory, before: &Value, record: &Value, previous: &Value) {
        crate::unported!("bot.memory.topics:_the_words_this_turn")
    }

    /// Python: bot/memory/topics.py:8467 `_a_step_back(reply)`
    pub fn _a_step_back(reply: &str) -> bool {
        crate::unported!("bot.memory.topics:_a_step_back")
    }

    /// Python: bot/memory/topics.py:8476 `_the_detail_steps_back(memory, reply)`
    pub fn _the_detail_steps_back(memory: &mut Memory, reply: &str) -> Value {
        crate::unported!("bot.memory.topics:_the_detail_steps_back")
    }

    /// Python: bot/memory/topics.py:8503 `the_step_back_takes_its_words_back(memory, reply)`
    pub fn the_step_back_takes_its_words_back(memory: &mut Memory, reply: &str) -> Option<Vec<String>> {
        crate::unported!("bot.memory.topics:the_step_back_takes_its_words_back")
    }

}

