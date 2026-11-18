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
