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
use std::sync::{Arc, LazyLock};

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


// ---------------------------------------------------------------------------------------------
// The rest of bot/memory/topics.py (workstream MEM3), in the Python file's order.
//
// A topic record is a Python dict ({"turn", "text", "words", ...}) and stays a `Value` here.
// Python's `memory["topic"]` is often the very same dict object as an item of
// `memory["topics"]`, so a write to one shows in the other. The port keeps that: a record held
// in both places shares one `Arc` (`same_object`), and a write to the topic record goes to every
// place that holds the same object (`_write_record`).

static NOT_DOING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_DOING"));
static ACKS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ACKS"));
static GREET: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GREET"));
static REFERRING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "REFERRING"));
static ORDERING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ORDERING"));
static ABOUT_ANOTHER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ABOUT_ANOTHER"));
static PART_OF_THE_ONE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PART_OF_THE_ONE"));
static MEASURE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MEASURE_WORDS"));
static PROPERTY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PROPERTY"));
static KIND_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "KIND_ASK"));
static SWITCHERS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "SWITCHERS"));
static RETURN_ASKS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RETURN_ASKS"));
static AIM_FRAME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "AIM_FRAME"));
static A_DEADLINE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_DEADLINE"));
static ASPECT_OF_IT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASPECT_OF_IT"));
static ASPECT_IS_A_PRONOUN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASPECT_IS_A_PRONOUN"));
static OWN_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "OWN_THING"));
static JOINS_THE_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "JOINS_THE_THING"));
static ABOUT_THE_PERSON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ABOUT_THE_PERSON"));
static PRONOUN_PERSON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PRONOUN_PERSON"));
static MEASURE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MEASURE"));
static _LEADS_IN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_LEADS_IN"));
static CORRECTION_NOISE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CORRECTION_NOISE"));
static WHO: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WHO"));
static RECENT_TURNS: LazyLock<i64> = LazyLock::new(|| t::int(M, "RECENT_TURNS"));

static TO_LEAD: LazyLock<Re> = LazyLock::new(|| Re::new(r"^\s*to\s+", crate::py::re::I));
static THREE_LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]{3}", crate::py::re::I));

// ---- record helpers ----

/// `a is b` for two records: the same dict object (one `Arc`).
fn same_object(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Dict(x), Value::Dict(y)) => Arc::ptr_eq(x, y),
        (Value::List(x), Value::List(y)) => Arc::ptr_eq(x, y),
        _ => false,
    }
}

/// `dict(record)` / `list(x)`: a shallow copy, a new object.
fn fresh(v: &Value) -> Value {
    match v {
        Value::Dict(d) => Value::Dict(Arc::new(d.as_ref().clone())),
        Value::List(l) => Value::List(Arc::new(l.as_ref().clone())),
        other => other.clone(),
    }
}

/// `str(record.get(key) or "")`.
fn rget(record: &Value, key: &str) -> String {
    record.get(key).map(value_str).unwrap_or_default()
}

/// `list(record.get("words") or [])`.
fn rwords(record: &Value) -> Vec<String> {
    record.get("words").map(Value::strs).unwrap_or_default()
}

fn is_dict(v: &Value) -> bool {
    matches!(v, Value::Dict(_))
}

/// `record.get("text")` is truthy.
fn has_text(record: &Value) -> bool {
    record.get("text").is_some_and(Value::truthy)
}

/// `record[key] = value` on a record memory holds: the new record goes everywhere the old object
/// is held (`memory["topic"]` and the items of `memory["topics"]` that are the same object).
/// Returns the record as it is now.
fn _write_record(memory: &mut Memory, record: &Value, key: &str, value: Value) -> Value {
    let mut new = fresh(record);
    new.set_key(key, value);
    _replace_object(memory, record, &new);
    new
}

/// Every place memory holds `old` (by identity) now holds `new`.
fn _replace_object(memory: &mut Memory, old: &Value, new: &Value) {
    if !is_dict(old) {
        return;
    }
    if memory.get("topic").is_some_and(|t_| same_object(t_, old)) {
        memory.set("topic", new.clone());
    }
    if let Some(list) = memory.get_mut("topics").and_then(Value::items_mut) {
        for item in list.iter_mut() {
            if same_object(item, old) {
                *item = new.clone();
            }
        }
    }
}

/// `record["words"] = (list(record.get("words") or []) + extra[:4])[:12]`, the extra words being
/// those of `found` the record does not hold yet.
fn _add_words(memory: &mut Memory, record: &Value, found: Vec<String>) -> Value {
    let have = rwords(record);
    let extra: Vec<String> = found.into_iter().filter(|w| !have.contains(w)).collect();
    let mut grown = have.clone();
    grown.extend(extra.into_iter().take(4));
    grown.truncate(12);
    _write_record(memory, record, "words", Value::from(grown))
}

/// The record a line opens: it becomes `memory["topic"]` and joins `memory["topics"]` unless the
/// last one already has its text. The topic in hand is kept when it is the same line.
fn _open_record(memory: &mut Memory, record: Value) -> Value {
    let text_ = record.get("text").cloned().unwrap_or(Value::None);
    if let Some(previous) = memory.get("topic") {
        if is_dict(previous) && previous.get("text").cloned().unwrap_or(Value::None) == text_ {
            return previous.clone();
        }
    }
    memory.set("topic", record.clone());
    if !matches!(memory.get("topics"), Some(Value::List(_))) {
        memory.set("topics", Value::list());
    }
    let topics = memory.get_mut("topics").unwrap();
    let last_text = topics.items().last().map(|l| l.get("text").cloned().unwrap_or(Value::None));
    if last_text.is_none_or(|l| l != text_) {
        topics.push(record.clone());
    }
    record
}

fn new_record(turn: usize, text_: &str, found: Vec<String>) -> Value {
    let mut record = Value::dict();
    record.set_key("turn", turn as i64);
    record.set_key("text", text::strip(text_));
    record.set_key("words", Value::from(found));
    record
}

/// `memory.get("understood")`.
fn understood_of(memory: &Memory) -> Option<&Understood> {
    memory.understood.as_ref()
}

fn stems_in(found: &[String], set: &HashSet<String>) -> bool {
    found.iter().any(|w| set.contains(&stem(w)))
}

/// The bot's last reply (`turns[-1]["bot"]`), or "".
fn last_bot(memory: &Memory) -> String {
    memory.turns.last().map(|t_| t_.bot.clone()).unwrap_or_default()
}

/// `" ".join(str(s or "").lower().split())`.
fn low_squash(s: &str) -> String {
    text::squash(&s.to_lowercase())
}

/// `_what_they_are_not_doing(text)`: the words a line puts after a negation ("wont listen" gives
/// {"listen"}), once each.
pub fn _what_they_are_not_doing(text_: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let line = low_squash(text_);
    for found in NOT_DOING.finditer(&line) {
        let word = found.str(1);
        if !word.is_empty() && !STOPWORDS.contains(word) && text::len(word) >= 3 {
            let s = stem(word);
            if !out.contains(&s) {
                out.push(s);
            }
        }
    }
    out
}

/// `content(text)`: the words a line is about, with the verb endings taken off and the grammar
/// dropped.
pub fn content(text_: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in words(text_) {
        if text::len(&word) < 3 || crate::understanding::pyuni::is_digit(&word) {
            continue;
        }
        if STOPWORDS.contains(&word) {
            continue;
        }
        let key = stem(&word);
        if STOPWORDS.contains(&key) || forms(&key).iter().any(|f| STOPWORDS.contains(f)) {
            continue;
        }
        if !out.contains(&key) {
            out.push(key);
        }
    }
    out
}

/// `is_ack(text)`: a short reply that answers the bot's last turn rather than starting a topic.
pub fn is_ack(text_: &str) -> bool {
    let clean = words(text_).join(" ");
    if clean.is_empty() {
        return true;
    }
    if ACKS.contains(&clean) || GREET.contains(&clean) {
        return true;
    }
    let parts = text::split(&clean);
    if parts.len() <= 4 && parts.iter().any(|w| ACKS.contains(w) || GREET.contains(w)) {
        return parts
            .iter()
            .all(|w| ACKS.contains(w) || GREET.contains(w) || STOPWORDS.contains(w) || text::len(w) <= 3);
    }
    false
}

/// `is_small_talk(text, kind=None)` (kind "" for None).
pub fn is_small_talk(text_: &str, kind: &str) -> bool {
    if ["empty", "greeting", "thanks"].contains(&kind) {
        return true;
    }
    is_ack(text_)
}

/// `_a_place_the_line_moves(text)`: the place a line moves a plan to (`facts.a_moved_place`).
pub fn _a_place_the_line_moves(text_: &str) -> String {
    crate::memory::facts::a_moved_place(text_)
}

/// `_asks_about_the_bot(read)`: the reading says the line is a question about the bot itself.
pub fn _asks_about_the_bot(read: Option<&Understood>) -> bool {
    read.is_some_and(|r| r.about_bot)
}

/// `_note(memory, text, kind=None)`: record the line as the current topic when it is about
/// something. Returns the record (a dict) or None.
pub fn _note(memory: &mut Memory, text_: &str, kind: &str) -> Value {
    let mut text_ = text_.to_string();
    if let Some(read) = memory.understood.clone() {
        if let Some(aim_length) = _the_length_of_the_aim(&read, memory) {
            memory.understood = Some(aim_length);
        }
    }
    // `read` as the function holds it from here on
    let read: Option<Understood> = memory.understood.clone();
    if let Some(read) = &read {
        if read.change_of_subject {
            let kept = text::squash(&read.clean);
            if !kept.is_empty() {
                text_ = kept;
            }
        }
    }
    let text_ = text_.as_str();
    _keep_person(memory, text_);
    if is_small_talk(text_, kind) {
        return Value::None;
    }
    if _asks_about_the_bot(read.as_ref()) {
        return Value::None;
    }
    let moved_to = _a_place_the_line_moves(text_);
    if !moved_to.is_empty() {
        let record = new_record(memory.turns.len(), text_, keywords(text_));
        return _open_record(memory, record);
    }
    let target = return_target(text_);
    if !target.is_empty() {
        if let Some(record) = find(memory, &target).cloned() {
            _resume(memory, &record, &target, text_);
            return memory.get("topic").cloned().unwrap_or(Value::None);
        }
        if !_asks(&target) {
            return current(memory).cloned().unwrap_or(Value::None);
        }
    }
    let leans = read.as_ref().is_some_and(|r| r.followup);
    if is_follow_up(text_, memory) || leans {
        let record = current(memory).cloned();
        if let Some(record) = &record {
            if !_after_an_emergency(memory) {
                if let Some(back) = _the_record_they_came_back_to(text_, memory, record) {
                    return _back_to(memory, &back, text_);
                }
            }
        }
        if let Some(record) = &record {
            if !_after_an_emergency(memory)
                && (!_names_another_subject(text_, record, memory) || _a_part_of_the_record(read.as_ref(), record))
            {
                if !_switches(text_) || _only_details(text_, record) {
                    return _add_words(memory, record, keywords(&_without_switchers(text_)));
                }
            }
        }
    }
    let constraint = _only_names_the_constraint(text_, memory);
    if _adds_a_detail(text_, memory)
        || _only_adds_a_person_to_the_thing(text_, &current(memory).cloned().unwrap_or(Value::None))
        || constraint
    {
        if let Some(record) = current(memory).cloned() {
            if !_after_an_emergency(memory) {
                if constraint {
                    return record;
                }
                return _add_words(memory, &record, keywords(text_));
            }
        }
    }
    if _is_a_when_or_how_often(text_, memory) {
        if let Some(record) = current(memory).cloned() {
            if !_after_an_emergency(memory) {
                return _add_words(memory, &record, keywords(text_));
            }
        }
    }
    if _asks_which_kind(text_, memory) {
        let record = _about_record(memory);
        if record.truthy() && !_after_an_emergency(memory) && !_reaches_another_thing(text_, memory) {
            return _add_words(memory, &record, keywords(text_));
        }
    }
    if _asks_an_aspect_of_the_thing(text_, memory) {
        if let Some(record) = current(memory).cloned() {
            if !_after_an_emergency(memory) && !_names_a_place_of_its_own(text_, &record, Some(memory)) {
                return _add_words(memory, &record, keywords(text_));
            }
        }
    }
    if let Some(kept) = _the_plan_a_correction_renames(memory, text_) {
        return kept;
    }
    let keys = keywords(text_);
    if keys.len() < 2 && words(text_).len() < 4 {
        let labels: Vec<String> = understood_of(memory)
            .map(|r| r.topics.iter().filter(|l| !l.is_empty()).cloned().collect())
            .unwrap_or_default();
        if labels.is_empty() {
            return Value::None;
        }
        let mut record = new_record(memory.turns.len(), text_, keys);
        record.set_key("id", labels[0].as_str());
        record.set_key("label", labels[0].as_str());
        return _open_record(memory, record);
    }
    let detail = _a_detail_of_what_we_are_on(text_, memory);
    let mut record = new_record(memory.turns.len(), text_, keys);
    let aim = _the_aim_without_its_deadline(text_);
    if !aim.is_empty() {
        record.set_key("text", aim.as_str());
        record.set_key("words", Value::from(keywords(&aim)));
    }
    if let Some(detail) = detail {
        record.set_key("of", rget(&detail, "text"));
    }
    _open_record(memory, record)
}

/// `_the_record_they_came_back_to(text, memory, record)`: an earlier record of this chat that
/// the line names, when the one in hand says nothing.
pub fn _the_record_they_came_back_to(text_: &str, memory: &Memory, record: &Value) -> Option<Value> {
    let found: HashSet<String> = keywords(text_).into_iter().collect();
    let held: HashSet<String> = rwords(record).into_iter().collect();
    if found.is_empty() || found.iter().any(|w| held.contains(w)) {
        return None;
    }
    if found.iter().all(|w| ORDERING.contains(w)) {
        return None;
    }
    let Some(Value::List(topics)) = memory.get("topics") else { return None };
    let start = topics.len().saturating_sub(4);
    let text_or_empty = |r: &Value| r.get("text").filter(|v| v.truthy()).cloned().unwrap_or_else(|| Value::from(""));
    let record_text = text_or_empty(record);
    for older in topics[start..].iter().rev() {
        if same_object(older, record) || text_or_empty(older) == record_text {
            continue;
        }
        let theirs: HashSet<String> = rwords(older).into_iter().filter(|w| !REFERRING.contains(w)).collect();
        if found.iter().any(|w| theirs.contains(w)) {
            return Some(older.clone());
        }
    }
    None
}

/// `_back_to(memory, record, text)`: make an earlier record the current one again, with the
/// line's words added to it.
pub fn _back_to(memory: &mut Memory, record: &Value, text_: &str) -> Value {
    let mut said = fresh(record);
    let kept = rwords(record);
    let mut grown = kept.clone();
    grown.extend(keywords(text_).into_iter().filter(|w| !kept.contains(w)));
    grown.truncate(12);
    said.set_key("words", Value::from(grown));
    memory.set("topic", said.clone());
    let text_or_empty = |r: &Value| r.get("text").filter(|v| v.truthy()).cloned().unwrap_or_else(|| Value::from(""));
    let said_text = text_or_empty(&said);
    if let Some(topics) = memory.get_mut("topics").and_then(Value::items_mut) {
        topics.retain(|other| !same_object(other, record) && text_or_empty(other) != said_text);
        topics.push(said.clone());
    }
    said
}

/// `_the_aim_without_its_deadline(text)`: the aim a line states, without the deadline clause it
/// is said against, or "".
pub fn _the_aim_without_its_deadline(text_: &str) -> String {
    let line = text::squash(text_);
    let Some(m) = AIM_FRAME.match_(&line) else { return String::new() };
    let rest = &line[m.end()..];
    let Some(cut) = A_DEADLINE.search(rest) else { return String::new() };
    let head = TO_LEAD.sub("", &rest[..cut.start()]);
    let aim = text::strip_chars(&head, " .,;:-").to_string();
    let n = text::split(&aim).len();
    if !(1..=8).contains(&n) || !THREE_LETTERS.is_match(&aim) {
        return String::new();
    }
    if aim.chars().next().is_some_and(|c| text::is_lower(&c.to_string())) {
        text::upper_first(&aim)
    } else {
        aim
    }
}

/// `_asks_which_kind(text, memory)`: the line asks which kind of the thing in hand.
pub fn _asks_which_kind(text_: &str, memory: &Memory) -> bool {
    if !KIND_ASK.is_match(&low_squash(text_)) {
        return false;
    }
    !subject(memory).is_empty()
}

/// `_asks_an_aspect_of_the_thing(text, memory)`: the line asks about a side of the thing the
/// chat is on, naming nothing of its own.
pub fn _asks_an_aspect_of_the_thing(text_: &str, memory: &Memory) -> bool {
    if !ASPECT_OF_IT.is_match(&low_squash(text_)) {
        return false;
    }
    if person(text_).is_some_and(|p| !p.is_empty()) {
        return false;
    }
    let found = words(text_);
    if found.iter().any(|w| ASPECT_IS_A_PRONOUN.contains(w)) {
        return false;
    }
    if stems_in(&found, &WHO) {
        return false;
    }
    !subject(memory).is_empty()
}

/// `_reaches_another_thing(text, memory)`: the line reaches an entry that is not about the
/// conversation's thing.
pub fn _reaches_another_thing(text_: &str, memory: &Memory) -> bool {
    let Some(mine) = _entry(text_) else { return false };
    let thing = subject(memory);
    if thing.is_empty() {
        return false;
    }
    !_names_it(&words(&thing), mine)
}

/// `_names_a_thing_of_its_own(text, record, memory=None)`.
pub fn _names_a_thing_of_its_own(text_: &str, record: &Value, memory: Option<&Memory>) -> bool {
    let known: HashSet<String> = rwords(record).into_iter().collect();
    let line = format!(" {} ", words(text_).join(" "));
    for found in OWN_THING.finditer(&line) {
        let word = stem(found.str(1));
        if WHO.contains(&word) || known.contains(&word) {
            continue;
        }
        return true;
    }
    _names_a_place_of_its_own(text_, record, memory)
}

/// `_names_a_place_of_its_own(text, record, memory=None)`: the line says where they are, about a
/// place the chat is not already on.
pub fn _names_a_place_of_its_own(text_: &str, record: &Value, memory: Option<&Memory>) -> bool {
    let _ = text_; // Python takes the line and reads only the record and the reading
    let Some(read) = memory.and_then(understood_of) else { return false };
    if read.kind == "question" {
        return false;
    }
    let known: HashSet<String> = rwords(record).into_iter().collect();
    let said: HashSet<String> = words(&rget(record, "text")).into_iter().collect();
    for found in &read.entities {
        if found.kind != "place" {
            continue;
        }
        let name = text::strip(&found.text).to_lowercase();
        if name.is_empty() {
            continue;
        }
        let parts = text::split(&name);
        if parts.iter().any(|p| known.contains(&stem(p))) {
            continue;
        }
        if parts.iter().any(|p| said.contains(p)) {
            continue;
        }
        return true;
    }
    false
}

/// `_only_adds_a_person_to_the_thing(text, record)`.
pub fn _only_adds_a_person_to_the_thing(text_: &str, record: &Value) -> bool {
    if !is_dict(record) || !has_text(record) {
        return false;
    }
    let line = words(text_).join(" ");
    if line.is_empty() || text::split(&line).len() > 14 {
        return false;
    }
    if _entry(text_).is_some() {
        return false;
    }
    let named = person(text_);
    let theirs = person(&rget(record, "text"));
    if named.is_none() && theirs.is_none() {
        return false;
    }
    if JOINS_THE_THING.is_match(&line) && named.is_some() {
        return true;
    }
    ABOUT_THE_PERSON.is_match(&line) && theirs.is_some() && PRONOUN_PERSON.is_match(&line)
}

/// `_another_area(text, said)`: the two lines are read as different kinds of thing.
pub fn _another_area(text_: &str, said: &str) -> bool {
    if said.is_empty() {
        return false;
    }
    let there = crate::understanding::understand(said, None).topic_own;
    let mine = crate::understanding::understand(text_, None).topic_own;
    if mine.as_ref().is_some_and(|m| MEASURE.contains(m)) {
        return false;
    }
    match (there, mine) {
        (Some(a), Some(b)) => !a.is_empty() && !b.is_empty() && a != b,
        _ => false,
    }
}

/// `_a_detail_of_what_we_are_on(text, memory)`: the record this line only says a detail about,
/// or None when it names a thing of its own.
pub fn _a_detail_of_what_we_are_on(text_: &str, memory: &Memory) -> Option<Value> {
    let record = current(memory)?.clone();
    if !is_dict(&record) || !has_text(&record) {
        return None;
    }
    if _this_line(&record, memory) || _after_an_emergency(memory) {
        return None;
    }
    if _names_a_thing_of_its_own(text_, &record, Some(memory)) {
        return None;
    }
    if !_recent(&record, memory) {
        return None;
    }
    let thing = gist(&rget(&record, "text"));
    if thing.is_empty() {
        return None;
    }
    let widened = _entry(&format!("{text_} {thing}"))?;
    let keyed = _key_words_of(widened);
    let thing_words: Vec<String> = keywords(&thing).into_iter().filter(|w| text::len(w) > 2).collect();
    if thing_words.is_empty() || !thing_words.iter().any(|w| keyed.iter().any(|k| alike(w, k))) {
        return None;
    }
    let own = _entry(text_);
    if own.is_some()
        && !keywords(text_).iter().any(|w| keyed.iter().any(|k| alike(w, k)))
        && _another_area(text_, &rget(&record, "text"))
    {
        return None;
    }
    if own.is_some_and(|o| std::ptr::eq(o, widened)) {
        return None;
    }
    if let Some(own) = own {
        if !_key_words_of(own).iter().any(|w| keyed.iter().any(|k| alike(w, k))) {
            return None;
        }
    }
    if keeps_away(&format!("{text_} {thing}"), &rget(&record, "text"), widened) {
        return None;
    }
    Some(record)
}

/// `_a_part_of_the_record(read, record)`: the reading says this line hangs off the record.
pub fn _a_part_of_the_record(read: Option<&Understood>, record: &Value) -> bool {
    let Some(read) = read else { return false };
    if !is_dict(record) {
        return false;
    }
    let leaned = words(read.leans_on.as_ref().map(|l| l.text.as_str()).unwrap_or("")).join(" ");
    !leaned.is_empty() && leaned == words(&rget(record, "text")).join(" ")
}

/// `_names_another_subject(text, record, memory=None)`: a line that leans back is really about a
/// different thing of its own.
pub fn _names_another_subject(text_: &str, record: &Value, memory: &Memory) -> bool {
    if _leads_a_question(text_) && _leans_back(text_, record) && _points_at_the_record(text_, record, memory) {
        return false;
    }
    let Some(mine) = _entry(text_) else { return false };
    let theirs = _entry(&rget(record, "text"));
    theirs.is_some_and(|t_| t_.id != mine.id)
}

/// `_leans_back(text, record)`: the line points at the thing the chat is on.
pub fn _leans_back(text_: &str, record: &Value) -> bool {
    let found = words(text_);
    if found.iter().any(|w| REFERRING.contains(w)) {
        return true;
    }
    let joined = found.join(" ");
    if PROPERTY.is_match(&joined) || PART_OF_THE_ONE.is_match(&joined) || ABOUT_ANOTHER.is_match(&joined) {
        return true;
    }
    let held = rwords(record);
    keywords(text_).iter().any(|w| held.contains(w))
}

/// `_points_at_the_record(text, record, memory=None)`: the line points at the thing the chat is
/// on, not merely reads as a property ask.
pub fn _points_at_the_record(text_: &str, record: &Value, memory: &Memory) -> bool {
    let mut found = words(text_);
    if a_placeholder_pronoun(text_) {
        found.retain(|w| w != "it");
    }
    if found.iter().any(|w| REFERRING.contains(w)) {
        return true;
    }
    let joined = found.join(" ");
    if PART_OF_THE_ONE.is_match(&joined) || ABOUT_ANOTHER.is_match(&joined) {
        return true;
    }
    let mine: HashSet<String> = keywords(text_).into_iter().collect();
    if mine.iter().all(|w| MEASURE_WORDS.contains(w)) {
        return true;
    }
    let held = rwords(record);
    if mine.iter().any(|w| held.contains(w)) {
        return true;
    }
    let said = last_bot(memory);
    if mine.is_empty() || said.is_empty() {
        return false;
    }
    let bot_words: HashSet<String> = words(&said).into_iter().collect();
    mine.iter().any(|w| bot_words.contains(w))
}

/// `_leads_a_question(text)`: the line opens a question.
pub fn _leads_a_question(text_: &str) -> bool {
    let found = words(text_);
    text_.contains('?') || found.first().is_some_and(|w| RETURN_ASKS.contains(w))
}

/// `_switches(text)`: the line turns to a different subject ("actually it's a funeral").
pub fn _switches(text_: &str) -> bool {
    let low = words(text_).join(" ");
    let padded = format!(" {low} ");
    SWITCHERS.iter().any(|s| low.starts_with(s.as_str()) || padded.contains(&format!(" {s} ")))
}

/// `opens_by_switching(text)`: the line opens by turning to another subject.
pub fn opens_by_switching(text_: &str) -> bool {
    let low = _LEADS_IN.sub("", &words(text_).join(" "));
    SWITCHERS.iter().any(|s| low.starts_with(s.as_str()))
}

/// `_without_switchers(text)`: the line without the words that announce or excuse a correction.
pub fn _without_switchers(text_: &str) -> String {
    let parts: Vec<String> = SWITCHERS.iter().flat_map(|s| text::split(s)).collect();
    words(text_)
        .into_iter()
        .filter(|w| !parts.contains(w) && !CORRECTION_NOISE.contains(w) && !ACKS.contains(w))
        .collect::<Vec<_>>()
        .join(" ")
}



static DETAIL_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DETAIL_WORDS"));
static A_THING_ON_ITS_WAY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_THING_ON_ITS_WAY"));
static NAMES_THE_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NAMES_THE_THING"));
static FRESH_NAMES: LazyLock<i64> = LazyLock::new(|| t::int(M, "FRESH_NAMES"));
static MY_THING_IS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MY_THING_IS"));
static SHORTHAND: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SHORTHAND"));
static FRESH_WORDS: LazyLock<i64> = LazyLock::new(|| t::int(M, "FRESH_WORDS"));
static WHEN_OR_AMOUNT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHEN_OR_AMOUNT"));
static NAMED_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NAMED_THING"));
static CONSTRAINT_IS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONSTRAINT_IS"));
static ON_A_BUDGET: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ON_A_BUDGET"));
static WHO_SAID: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHO_SAID"));
static WHO_SAID_THIRD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHO_SAID_THIRD"));
static A_BARE_PRONOUN_SUBJECT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_BARE_PRONOUN_SUBJECT"));
static A_QUESTION_THEY_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_QUESTION_THEY_ASK"));
static PERSON_TROUBLE: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| t::str_re_pairs(M, "PERSON_TROUBLE"));
static YOUNG_PERSON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "YOUNG_PERSON"));
static FELL_OUT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FELL_OUT"));
static FALLS_FINE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FALLS_FINE"));
static A_JOB_AT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_JOB_AT"));
static SOMETHING_WRONG: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SOMETHING_WRONG"));
static CHANGE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CHANGE"));
static A_WORRY_AFTER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_WORRY_AFTER"));
static A_CHILDS_PERSON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_CHILDS_PERSON"));
static A_GROWN_AGE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_GROWN_AGE"));
static REPEATED_SAID: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "REPEATED_SAID"));
static CORRECTION_LINE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CORRECTION_LINE"));
static INSIDE_A_SAFETY_THREAD: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "INSIDE_A_SAFETY_THREAD"));
static WH_WORD_WITHOUT_ITS_APOSTROPHE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WH_WORD_WITHOUT_ITS_APOSTROPHE"));
static RETURN_LEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RETURN_LEAD"));
static CASUAL_LEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CASUAL_LEAD"));
static CASUAL_OPEN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CASUAL_OPEN"));
static CASUAL_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CASUAL_VERB"));
static RETURN_TIME: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RETURN_TIME"));
static HELPER_ASKS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HELPER_ASKS"));
static ING_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ING_VERBS"));
static VERB_PHRASE_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "VERB_PHRASE_AFTER"));
static MODALS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MODALS"));
static LEAD_ACKS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LEAD_ACKS"));
static CONTINUATION_START: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CONTINUATION_START"));
static ASK_ONLY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_ONLY"));
static _LEAD_QUESTION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_LEAD_QUESTION"));
static _RELATION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_RELATION"));
static _FRAME_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_FRAME_ASK"));
static _WITHOUT_CLAUSE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_WITHOUT_CLAUSE"));
static _PHRASAL_RE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_PHRASAL_RE"));
static _WISH_RE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_WISH_RE"));
static _LEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_LEAD"));
static _CORRECTION_LEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_CORRECTION_LEAD"));
static _TIME_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_TIME_WORDS"));
static _CONSIDERING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_CONSIDERING"));
static _BUYING_A_STATE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_BUYING_A_STATE"));
static _MAX_SUBJECT: LazyLock<i64> = LazyLock::new(|| t::int(M, "_MAX_SUBJECT"));
static _A_FIGURE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_FIGURE"));
static _HANDOFF_THROAT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_HANDOFF_THROAT"));
static _DATE_OCCASIONS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_DATE_OCCASIONS"));
static _DATE_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_DATE_FILLER"));
static _MAKING_A_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_MAKING_A_THING"));
static _ROOM_LIMIT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_ROOM_LIMIT"));
static _HOME_SPACES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_HOME_SPACES"));
static DURATION_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DURATION_ASK"));
static DURATION_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DURATION_WORDS"));
static ACTION: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ACTION"));
static MAX_ACTION: LazyLock<i64> = LazyLock::new(|| t::int(M, "MAX_ACTION"));
static PERSON_LEAD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PERSON_LEAD"));
static ACTION_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ACTION_AFTER"));
static ONE_ATTRIBUTE: LazyLock<Vec<(&'static Re, String)>> = LazyLock::new(|| t::re_str_pairs(M, "ONE_ATTRIBUTE"));
static ATTRIBUTE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ATTRIBUTE_WORDS"));
static NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_THING"));
static A_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_STATE"));
/// `sorted(lexicon.HANDOFF_LEADS, key=len, reverse=True)`.
static HANDOFF_LEADS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut v = t::strs("bot.understanding.lexicon.", "HANDOFF_LEADS");
    v.sort_by_key(|s| std::cmp::Reverse(text::len(s)));
    v
});
/// `sorted(SWITCHERS, key=len, reverse=True)`.
static SWITCHERS_LONGEST_FIRST: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut v = SWITCHERS.clone();
    v.sort_by_key(|s| std::cmp::Reverse(text::len(s)));
    v
});
/// `r"[,?!;]|\b(?:" + "|".join(sorted(RETURN_ASKS)) + r")\b"`.
static THE_ASK_IN_A_RETURN: LazyLock<Re> = LazyLock::new(|| {
    let mut asks: Vec<&String> = RETURN_ASKS.iter().collect();
    asks.sort();
    let joined: Vec<&str> = asks.iter().map(|s| s.as_str()).collect();
    Re::new(&format!(r"[,?!;]|\b(?:{})\b", joined.join("|")), 0)
});
static DATE_SPLIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:and|but)\b|[,;]", 0));
static A_TIME_SPLIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:for|tonight|today|tomorrow|yesterday|this|next)\b", 0));
static COMMA_OR_SEMI: LazyLock<Re> = LazyLock::new(|| Re::new(r"[,;]", 0));
static ACTION_SPLIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:and|but|because|so|then|when|if|before|after)\b", 0));
static A_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d", 0));

/// `_only_details(text, record)`: everything a correction brings is a detail of the subject.
pub fn _only_details(text_: &str, record: &Value) -> bool {
    let have = rwords(record);
    keywords(&_without_switchers(text_)).iter().filter(|k| !have.contains(k)).all(|k| DETAIL_WORDS.contains(k))
}

/// `_the_plan_a_correction_renames(memory, text)`: the record the line corrects, when it names a
/// different thing inside a plan of theirs. The record gains the new name's words.
pub fn _the_plan_a_correction_renames(memory: &mut Memory, text_: &str) -> Option<Value> {
    let record = current(memory)?.clone();
    if !is_dict(&record) || _after_an_emergency(memory) {
        return None;
    }
    if NAMES_THE_THING.match_(&low_squash(text_)).is_none() {
        return None;
    }
    if !A_THING_ON_ITS_WAY.is_match(&rget(&record, "text")) {
        return None;
    }
    let have = rwords(&record);
    let extra: Vec<String> = keywords(text_).into_iter().filter(|k| !have.contains(k)).collect();
    if extra.is_empty() {
        return None;
    }
    let mut grown = have.clone();
    grown.extend(extra.into_iter().take(*FRESH_NAMES as usize));
    grown.truncate(12);
    Some(_write_record(memory, &record, "words", Value::from(grown)))
}

/// `_is_a_quantity(reading)`: the numbers in the line carry a measure ("6000 pounds", "3k").
pub fn _is_a_quantity(reading: &Understood) -> bool {
    for number in &reading.numbers {
        if number.unit.as_deref().is_some_and(|u| !u.is_empty()) {
            return true;
        }
        if SHORTHAND.match_(text::strip(&number.text)).is_some() {
            return true;
        }
    }
    false
}

/// The record in hand was written within the last few turns.
fn _recent(record: &Value, memory: &Memory) -> bool {
    match record.get("turn") {
        Some(Value::Int(turn)) => memory.turns.len() as i64 - turn <= *RECENT_TURNS,
        Some(Value::Bool(b)) => memory.turns.len() as i64 - (*b as i64) <= *RECENT_TURNS,
        _ => true,
    }
}

/// `_is_a_when_or_how_often(text, memory)`: the line only adds a when or a how-often.
pub fn _is_a_when_or_how_often(text_: &str, memory: &Memory) -> bool {
    let Some(read) = understood_of(memory) else { return false };
    if read.kind != "statement" {
        return false;
    }
    let found = words(text_);
    if found.is_empty() || found.len() > 8 {
        return false;
    }
    if !WHEN_OR_AMOUNT.is_match(&found.join(" ")) {
        return false;
    }
    if NAMED_THING.is_match(&format!(" {} ", found.join(" "))) {
        return false;
    }
    if keywords(text_).len() > 4 {
        return false;
    }
    if _entry(text_).is_some() {
        return false;
    }
    let Some(record) = current(memory) else { return false };
    _recent(record, memory)
}

/// `_only_names_the_constraint(text, memory)`: the line puts a number on a constraint of the
/// thing in hand, and nothing else.
pub fn _only_names_the_constraint(text_: &str, memory: &Memory) -> bool {
    let Some(read) = understood_of(memory) else { return false };
    if read.kind != "statement" {
        return false;
    }
    let said = words(text_).join(" ");
    let on_a_budget = ON_A_BUDGET.match_(&said).is_some();
    if CONSTRAINT_IS.match_(&said).is_none() && !on_a_budget {
        return false;
    }
    if on_a_budget {
        if !A_DIGIT.is_match(&said) {
            return false;
        }
    } else if !_is_a_quantity(read) {
        return false;
    }
    let Some(record) = current(memory) else { return false };
    _recent(record, memory)
}

/// `_adds_a_detail(text, memory)`: the line only says a how-much about what the conversation is
/// already on.
pub fn _adds_a_detail(text_: &str, memory: &Memory) -> bool {
    let Some(read) = understood_of(memory) else { return false };
    if read.kind != "statement" {
        return false;
    }
    if !_is_a_quantity(read) {
        return false;
    }
    let Some(record) = current(memory) else { return false };
    if !_recent(record, memory) {
        return false;
    }
    let held = rwords(record);
    if let Some(named) = MY_THING_IS.match_(&format!("{} ", words(text_).join(" "))) {
        if !held.contains(&stem(named.str(1))) {
            return false;
        }
    }
    let mut numeric: HashSet<String> = HashSet::new();
    for number in &read.numbers {
        numeric.extend(words(&number.text).iter().map(|w| stem(w)));
    }
    let fresh_: Vec<String> = keywords(text_).into_iter().filter(|w| !held.contains(w) && !numeric.contains(w)).collect();
    fresh_.len() as i64 <= *FRESH_WORDS
}

/// `person(text)`: the person this speaker names ("my mum" -> "mum"), or None.
pub fn person(text_: &str) -> Option<String> {
    WHO_SAID.search(&words(text_).join(" ")).map(|m| m.str(1).to_string())
}

/// `_named_inside_a_question(text)`: this line's pronoun subject is asking a question that names
/// somebody else.
pub fn _named_inside_a_question(text_: &str) -> bool {
    let line = words(&text_.to_lowercase().replace('\u{2019}', "'")).join(" ");
    if !A_BARE_PRONOUN_SUBJECT.is_match(&line) {
        return false;
    }
    A_QUESTION_THEY_ASK.is_match(&line)
}

/// `person_named(text, memory=None)`: the person the line is about.
pub fn person_named(text_: &str, memory: &Memory) -> Option<String> {
    if let Some(who) = person(text_).filter(|w| !w.is_empty()) {
        if about(memory).truthy() && _named_inside_a_question(text_) {
            return None;
        }
        return Some(who);
    }
    if about(memory).truthy() {
        return None;
    }
    WHO_SAID_THIRD.search(&words(text_).join(" ")).map(|m| m.str(1).to_string())
}

/// `about(memory)`: the person the chat is about, as a record, or None.
pub fn about(memory: &Memory) -> Value {
    match memory.get("person") {
        Some(record) if is_dict(record) && record.get("who").is_some_and(Value::truthy) => record.clone(),
        _ => Value::None,
    }
}

/// `trouble(text)`: which kind of trouble this line reports about the person, or None.
pub fn trouble(text_: &str) -> Option<String> {
    let low = text::squash(&text_.to_lowercase().replace('\u{2019}', "'"));
    if A_JOB_AT.is_match(&low) && !SOMETHING_WRONG.is_match(&low) {
        return None;
    }
    for (kind, pattern) in PERSON_TROUBLE.iter() {
        if !pattern.is_match(&low) {
            continue;
        }
        if kind == "known" && !CHANGE.is_match(&low) {
            continue;
        }
        if kind == "memory" && REPEATED_SAID.is_match(&low) && A_CHILDS_PERSON.is_match(&low) && !A_GROWN_AGE.is_match(&low) {
            continue;
        }
        if kind == "homeward" && A_WORRY_AFTER.is_match(&low) {
            continue;
        }
        if kind == "falls" && (YOUNG_PERSON.is_match(&low) || FALLS_FINE.is_match(&low) || FELL_OUT.is_match(&low)) {
            continue;
        }
        return Some(kind.clone());
    }
    None
}

/// `_keep_person(memory, text)`: record the person this line is about, and keep the last one
/// when the line says "she". Returns the person record or None.
pub fn _keep_person(memory: &mut Memory, text_: &str) -> Value {
    let turn = memory.turns.len() as i64;
    let who = person_named(text_, memory);
    let kind = trouble(text_);
    let said = text::strip(text_).to_string();
    let mut record = match who.filter(|w| !w.is_empty()) {
        Some(who) => {
            let kept = about(memory);
            if kept.get("who").cloned().unwrap_or(Value::None) != Value::from(who.as_str())
                || kept.get("said").cloned().unwrap_or(Value::None) != Value::from(said.as_str())
            {
                let mut fresh_ = Value::dict();
                fresh_.set_key("who", who.as_str());
                fresh_.set_key("said", said.as_str());
                fresh_.set_key("turn", turn);
                memory.set("person", fresh_);
            }
            memory.get("person").cloned().unwrap_or(Value::None)
        }
        None => {
            let mut record = about(memory);
            if !record.is_none() {
                record.set_key("turn", turn);
                memory.set("person", record.clone());
            }
            record
        }
    };
    if !record.is_none() {
        if let Some(kind) = kind.filter(|k| !k.is_empty()) {
            if record.get("trouble").cloned().unwrap_or(Value::None) != Value::from(kind.as_str()) {
                record.set_key("trouble_turn", turn);
            }
            record.set_key("trouble", kind.as_str());
            memory.set("person", record.clone());
        }
    }
    record
}

/// `_after_an_emergency(memory)`: the bot's last answer was an emergency.
pub fn _after_an_emergency(memory: &Memory) -> bool {
    memory.turns.last().is_some_and(|t_| t_.module.as_deref() == Some("safety"))
}

/// `_after_a_correction(memory)`: the line before this one corrected something.
pub fn _after_a_correction(memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    let said = &last.user;
    if text::strip(said).is_empty() {
        return false;
    }
    CORRECTION_LINE.is_match(&low_squash(said))
}

/// `in_a_safety_thread(memory)`: the bot's last real answer was an emergency and nothing has
/// answered since.
pub fn in_a_safety_thread(memory: &Memory) -> bool {
    let start = memory.turns.len().saturating_sub(3);
    for turn in memory.turns[start..].iter().rev() {
        let module = turn.module.as_deref();
        if module == Some("safety") {
            return true;
        }
        match module {
            None | Some("greeting") => continue,
            Some(m) if INSIDE_A_SAFETY_THREAD.iter().any(|x| x == m) => continue,
            _ => return false,
        }
    }
    false
}

/// `_the_apostrophe_back(low)`: "whats good to see" read as "what is good to see".
pub fn _the_apostrophe_back(low: &str) -> String {
    WH_WORD_WITHOUT_ITS_APOSTROPHE.sub(r"\1 is", low)
}

/// `return_target(text)`: what a line like "ok back to the boiler" or "anyway, the laptop"
/// names, or "".
pub fn return_target(text_: &str) -> String {
    let low = _the_apostrophe_back(&words(text_).join(" "));
    if let Some(found) = RETURN_LEAD.match_(&low) {
        let rest = text::strip_chars(&low[found.end()..], " ,.?!");
        return _the_thing(rest);
    }
    let Some(casual) = CASUAL_LEAD.match_(&low) else { return String::new() };
    let target = text::strip_chars(&low[casual.end()..], " ,.?!");
    let parts = text::split(target);
    if parts.is_empty() || !CASUAL_OPEN.contains(&parts[0]) {
        return String::new();
    }
    _the_thing(target)
}

/// `_a_question_after_the_thing(rest, split)`: the words from `split` on are a question about
/// the thing rather than its verb.
pub fn _a_question_after_the_thing(rest: &str, split: usize) -> bool {
    let parts = text::split(rest);
    if split >= parts.len() || !HELPER_ASKS.contains(&parts[split]) {
        return true;
    }
    let after = if split + 1 < parts.len() { text::strip_chars(&parts[split + 1], ".,!?;:").to_string() } else { String::new() };
    if after.is_empty() {
        return true;
    }
    if MODALS.contains(&parts[split]) && (ING_VERBS.contains(&after) || after.ends_with("ed")) {
        return true;
    }
    !(VERB_PHRASE_AFTER.contains(&after) || ING_VERBS.contains(&after) || after.ends_with("ed"))
}

/// `_the_thing(rest)`: "the laptop, which would you get?" names the laptop; "work tomorrow"
/// names nothing.
pub fn _the_thing(rest: &str) -> String {
    let asked = THE_ASK_IN_A_RETURN.search(rest);
    let head = match &asked {
        Some(m) => &rest[..m.start()],
        None => rest,
    };
    if asked.is_some() && !_a_question_after_the_thing(rest, text::split(head).len()) {
        return String::new();
    }
    let mut parts = text::split(head);
    while parts.last().is_some_and(|p| CASUAL_OPEN.contains(p) || STOPWORDS.contains(p)) {
        parts.pop();
    }
    if parts.is_empty() || parts.len() > 3 {
        return String::new();
    }
    if parts.iter().any(|p| CASUAL_VERB.contains(p) || RETURN_TIME.contains(p)) {
        return String::new();
    }
    parts.join(" ")
}

/// `_asks(text)`.
pub fn _asks(text_: &str) -> bool {
    if text_.contains('?') {
        return true;
    }
    text::split(text_).iter().any(|w| RETURN_ASKS.contains(w))
}

/// `asks_after(line, thing)`: does the line ask something after the thing it went back to?
pub fn asks_after(line: &str, thing: &str) -> bool {
    let mut low = _the_apostrophe_back(&words(line).join(" "));
    let named = words(thing).join(" ");
    if !named.is_empty() {
        if let Some(at) = low.find(&named) {
            low = low[at + named.len()..].to_string();
        }
    }
    _asks(&low)
}

/// `the_thing_asked_again(memory, text="")`: the line a person went back to, with the ask they
/// made on it, or "".
pub fn the_thing_asked_again(memory: &Memory, text_: &str) -> String {
    let Some(record) = resumed(memory) else { return String::new() };
    if !record.get("asks").is_some_and(Value::truthy) {
        return String::new();
    }
    let thing = low_squash(&rget(&record, "asked"));
    let line = low_squash(text_);
    if thing.is_empty() || line.is_empty() {
        return String::new();
    }
    let Some(at) = line.find(&thing) else { return String::new() };
    let tail = text::strip_chars(&line[at + thing.len()..], " ,;:.-");
    if tail.is_empty() {
        return String::new();
    }
    format!("{thing} {tail}")
}

/// `the_thing_went_back_to(memory)`: the thing named after a return opener.
pub fn the_thing_went_back_to(memory: &Memory) -> String {
    let Some(record) = resumed(memory) else { return String::new() };
    if !record.get("asks").is_some_and(Value::truthy) {
        return String::new();
    }
    low_squash(&rget(&record, "asked"))
}

/// `_resume(memory, record, target, line="")`: make the earlier topic current again, and note
/// the turn so a reply can greet the return.
pub fn _resume(memory: &mut Memory, record: &Value, target: &str, line: &str) -> Value {
    let mut said = fresh(record);
    let subject_ = if !target.is_empty() {
        Value::from(words(target).join(" "))
    } else {
        said.get("subject").filter(|v| v.truthy()).cloned().unwrap_or_else(|| Value::from(""))
    };
    said.set_key("subject", subject_);
    memory.set("topic", said.clone());
    if !matches!(memory.get("topics"), Some(Value::List(_))) {
        memory.set("topics", Value::list());
    }
    let said_text = said.get("text").filter(|v| v.truthy()).cloned().unwrap_or_else(|| Value::from(""));
    {
        let topics = memory.get_mut("topics").unwrap().items_mut().unwrap();
        topics.retain(|other| {
            let other_text = other.get("text").filter(|v| v.truthy()).cloned().unwrap_or_else(|| Value::from(""));
            !same_object(other, record) && other_text != said_text
        });
        topics.push(said.clone());
    }
    let text_or_target = said.get("text").filter(|v| v.truthy()).cloned().unwrap_or_else(|| Value::from(target));
    memory.set("return_to", text_or_target.clone());
    let mut returned = Value::dict();
    returned.set_key("text", text_or_target);
    returned.set_key("asked", target);
    returned.set_key("asks", asks_after(line, target));
    returned.set_key("turn", memory.turns.len() as i64);
    memory.set("returned", returned);
    _the_answer_on_the_return(memory, &said);
    said
}

/// `{form for word in words for form in (word, stem(word))} - STOPWORDS`.
fn _the_forms_of(found: impl IntoIterator<Item = String>) -> HashSet<String> {
    let mut out = HashSet::new();
    for word in found {
        let s = stem(&word);
        out.insert(word);
        out.insert(s);
    }
    out.retain(|w| !STOPWORDS.contains(w));
    out
}

/// `_the_answer_on_the_return(memory, said)`: put the answer that belongs to the topic gone back
/// to on top of the kept answers.
pub fn _the_answer_on_the_return(memory: &mut Memory, said: &Value) {
    let Some(Value::List(kept)) = memory.get("entry_answers") else { return };
    if kept.len() < 2 {
        return;
    }
    let wanted = _the_forms_of(keywords(&rget(said, "text")));
    if wanted.is_empty() {
        return;
    }
    let mut best: Option<usize> = None;
    let mut best_score = 0;
    for (i, record) in kept.iter().enumerate() {
        if !is_dict(record) {
            continue;
        }
        let keys: Vec<String> = record.get("keys").map(Value::strs).unwrap_or_default();
        let forms_ = _the_forms_of(keys);
        let score = wanted.iter().filter(|w| forms_.contains(*w)).count();
        if score > best_score {
            best = Some(i);
            best_score = score;
        }
    }
    let Some(best) = best else { return };
    let best_value = kept[best].clone();
    if !best_value.truthy() {
        return;
    }
    let last = kept.last().unwrap();
    if best == kept.len() - 1 || same_object(&best_value, last) {
        return;
    }
    let list = memory.get_mut("entry_answers").unwrap().items_mut().unwrap();
    let first_equal = list.iter().position(|x| same_object(x, &best_value) || *x == best_value).unwrap();
    let moved = list.remove(first_equal);
    list.push(moved);
    memory.set("entry_answer", best_value);
}

/// `resumed(memory)`: the earlier topic the user went back to on this turn, or None.
pub fn resumed(memory: &Memory) -> Option<Value> {
    let record = memory.get("returned")?;
    if !is_dict(record) || !has_text(record) {
        return None;
    }
    if record.get("turn").cloned().unwrap_or(Value::None) != Value::Int(memory.turns.len() as i64) {
        return None;
    }
    Some(record.clone())
}

/// `is_follow_up(text, memory)`: a line leans on the current topic.
pub fn is_follow_up(text_: &str, memory: &Memory) -> bool {
    let Some(record) = current(memory) else { return false };
    if _this_line(record, memory) {
        return false;
    }
    let mut clean = words(text_).join(" ");
    if clean.is_empty() || is_ack(text_) || GREET.contains(&clean) {
        return false;
    }
    let mut rest = words(text_);
    while rest.first().is_some_and(|w| LEAD_ACKS.contains(w)) {
        rest.remove(0);
    }
    if !rest.is_empty() {
        clean = rest.join(" ");
    }
    let low = clean.clone();
    if _switches(&clean) {
        return false;
    }
    let found = words(text_);
    if found.len() > 16 {
        return false;
    }
    let topic_words: HashSet<String> = rwords(record).into_iter().collect();
    let fresh_: Vec<String> = keywords(text_).into_iter().filter(|k| !topic_words.contains(k)).collect();
    if KIND_ASK.is_match(&low) && found.len() <= 10 {
        return true;
    }
    if CONTINUATION_START.contains(&found[0]) {
        if fresh_.len() <= 2 && found.len() <= 8 {
            return true;
        }
        if PROPERTY.is_match(&low) && fresh_.len() <= 3 && found.len() <= 10 {
            return true;
        }
    }
    if found.iter().any(|w| REFERRING.contains(w)) && fresh_.len() <= 3 {
        return true;
    }
    fresh_.len() <= 1 && found.len() <= 8
}

/// `context(memory, n=2)`: the last n topic texts, oldest first.
pub fn context(memory: &Memory, n: i64) -> Vec<String> {
    let Some(Value::List(records)) = memory.get("topics") else { return Vec::new() };
    // records[-n:], with Python's slice rules (n=0 is the whole list)
    let len = records.len() as i64;
    let start = if n > 0 { (len - n).max(0) } else if n == 0 { 0 } else { (-n).min(len) };
    let mut out = Vec::new();
    for r in &records[start as usize..] {
        if !is_dict(r) {
            panic!("AttributeError: topic record is not a dict");
        }
        if r.get("text").is_some_and(Value::truthy) {
            out.push(rget(r, "text"));
        }
    }
    out
}

/// `expand(text, memory)`: the line plus the topic it follows.
pub fn expand(text_: &str, memory: &Memory) -> Value {
    if let Some(record) = current(memory) {
        if is_follow_up(text_, memory) {
            let mut earlier = context(memory, 2);
            if earlier.is_empty() {
                earlier = vec![rget(record, "text")];
            }
            return Value::from(format!("{text_} (earlier: {})", earlier.join(" / ")));
        }
    }
    Value::from(text_)
}

/// `_gerund(verb)`: the -ing form of a verb.
pub fn _gerund(verb: &str) -> String {
    let c: Vec<char> = verb.chars().collect();
    if verb.ends_with("ie") {
        return format!("{}ying", &verb[..verb.len() - 2]);
    }
    if verb.ends_with('e') && !(verb.ends_with("ee") || verb.ends_with("ye") || verb.ends_with("oe")) {
        return format!("{}ing", &verb[..verb.len() - 1]);
    }
    let n = c.len();
    if n > 2 && !"aeiouwxy".contains(c[n - 1]) && "aeiou".contains(c[n - 2]) && !"aeiou".contains(c[n - 3]) {
        return format!("{verb}{}ing", c[n - 1]);
    }
    format!("{verb}ing")
}

/// `_strip_considering(source)`: the line without the words that only say they are thinking of
/// getting the thing.
pub fn _strip_considering(source: &str) -> String {
    let mut source = source.to_string();
    loop {
        let stripped = _CONSIDERING.sub("", &source);
        if stripped == source {
            return source;
        }
        source = stripped;
    }
}

/// `_only_a_handoff(head)`: a clause only announces that something is coming.
pub fn _only_a_handoff(head: &str) -> bool {
    let said = low_squash(head);
    let said = text::strip_chars(&said, " ,.;:!?-");
    let said = _HANDOFF_THROAT.sub("", said);
    let said = text::strip_chars(&said, " ,.;:!?-");
    for lead in HANDOFF_LEADS.iter() {
        if said == lead || (said.starts_with(lead.as_str()) && keywords(&said[lead.len()..]).is_empty()) {
            return true;
        }
    }
    false
}

/// `_the_people()`: the words that name a person in the user's life, in the forms keywords
/// keeps (facts.RELATIONS and their stems).
pub fn _the_people() -> &'static HashSet<String> {
    static PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| {
        let mut out = HashSet::new();
        for word in t::strs("bot.memory.facts.", "RELATIONS") {
            out.insert(stem(&word));
            out.insert(word);
        }
        out
    });
    &PEOPLE
}

/// `_a_word_of_a_date(word)`: a content word can be part of a date and nothing else.
pub fn _a_word_of_a_date(word: &str) -> bool {
    let forms_ = [word.to_string(), stem(word)];
    forms_.iter().any(|f| _DATE_OCCASIONS.contains(f))
        || forms_.iter().any(|f| _TIME_WORDS.contains(f))
        || _DATE_FILLER.contains(word)
        || forms_.iter().any(|f| _A_FIGURE.contains(f))
        || forms_.iter().any(|f| _the_people().contains(f))
}

/// `_only_a_date(clause)`: a clause only says when a day is.
pub fn _only_a_date(clause: &str) -> bool {
    let kept = keywords(clause);
    if kept.is_empty() || !kept.iter().any(|w| _DATE_OCCASIONS.contains(&stem(w))) {
        return false;
    }
    kept.iter().all(|w| _a_word_of_a_date(w))
}

/// `_the_thing_after_a_date(text)`: the clause that names the thing a dated line is about, or "".
pub fn _the_thing_after_a_date(text_: &str) -> String {
    let Some(m) = DATE_SPLIT.search(text_) else { return String::new() };
    let (head, tail) = (&text_[..m.start()], &text_[m.end()..]);
    if !_only_a_date(head) || keywords(tail).is_empty() {
        return String::new();
    }
    if _MAKING_A_THING.is_match(tail) { tail.to_string() } else { String::new() }
}

/// `gist(text)`: the short noun phrase a line is really about.
pub fn gist(text_: &str) -> String {
    let mut source = low_squash(text_);
    source = _CORRECTION_LEAD.sub("", &source);
    for switcher in SWITCHERS_LONGEST_FIRST.iter() {
        if source.starts_with(switcher.as_str()) {
            source = text::lstrip_chars(&source[switcher.len()..], " ,.!-").to_string();
            break;
        }
    }
    let dated = _the_thing_after_a_date(&source);
    if !dated.is_empty() {
        source = dated;
    }
    source = text::squash(&_PHRASAL_RE.sub(" ", &source));
    source = _RELATION.split(&source).into_iter().next().unwrap_or_default();
    let whole = source.clone();
    if _FRAME_ASK.is_match(&source) {
        let trimmed = _WITHOUT_CLAUSE.sub("", &source);
        let trimmed = text::strip_chars(&trimmed, " ,.;:!?");
        if !trimmed.is_empty() && !keywords(trimmed).is_empty() {
            source = trimmed.to_string();
        }
    }
    source = _strip_considering(&source);
    let buying = _BUYING_A_STATE.match_(&source).is_some();
    let mut wished = false;
    for _ in 0..(if buying { 0 } else { 3 }) {
        let stripped = _WISH_RE.sub("", &source);
        if stripped == source || text::strip_chars(&stripped, " ,.;:!?").is_empty() {
            break;
        }
        source = stripped;
        wished = true;
    }
    let before_split = source.clone();
    let segments = A_TIME_SPLIT.split(&source);
    source = segments[0].clone();
    if !wished && !buying {
        source = _LEAD.sub("", text::strip_chars(&source, " ,.;:!?"));
        loop {
            let trimmed = _LEAD_QUESTION.sub("", &source);
            if trimmed == source || text::strip(&trimmed).is_empty() {
                break;
            }
            source = trimmed;
        }
    }
    source = _strip_considering(&source);
    let (head, _, after_handoff) = text::partition(&source, ",");
    source = if !text::strip(after_handoff).is_empty() && _only_a_handoff(head) {
        after_handoff.to_string()
    } else {
        COMMA_OR_SEMI.split(&source).into_iter().next().unwrap_or_default()
    };
    let mut kept: Vec<String> = keywords(&source).into_iter().filter(|w| !ASK_ONLY.contains(w)).collect();
    while kept.last().is_some_and(|w| _TIME_WORDS.contains(w)) {
        kept.pop();
    }
    if !kept.is_empty() && kept.iter().all(|w| _A_FIGURE.contains(w)) {
        for part in &segments[1..] {
            let later: Vec<String> =
                keywords(part).into_iter().filter(|w| !ASK_ONLY.contains(w) && !_TIME_WORDS.contains(w)).collect();
            if !later.is_empty() && !later.iter().all(|w| _A_FIGURE.contains(w)) {
                kept = later;
                break;
            }
        }
    }
    if kept.is_empty() && wished {
        for source in [&before_split, &whole] {
            kept = keywords(source).into_iter().filter(|w| !ASK_ONLY.contains(w) && !_TIME_WORDS.contains(w)).collect();
            if !kept.is_empty() {
                break;
            }
        }
    }
    if kept.is_empty() {
        return String::new();
    }
    kept.truncate(*_MAX_SUBJECT as usize);
    kept.join(" ")
}

/// `_room_limit(text)`: the home space a line only says the size of, or "".
pub fn _room_limit(text_: &str) -> String {
    let said = low_squash(text_);
    let Some(found) = _ROOM_LIMIT.match_(&said) else { return String::new() };
    let head = text::strip_chars(found.name_str("head"), " ,.;:!?-");
    if head.is_empty() {
        return String::new();
    }
    if text::split(head).last().is_some_and(|w| _HOME_SPACES.contains(w)) { head.to_string() } else { String::new() }
}

/// `_about_record(memory)`: the last topic that names something, rather than only saying how
/// much room there is. `{}` when there is none.
pub fn _about_record(memory: &Memory) -> Value {
    let Some(Value::List(records)) = memory.get("topics") else {
        return current(memory).cloned().unwrap_or_else(Value::dict);
    };
    for record in records.iter().rev() {
        if !is_dict(record) || !_room_limit(&rget(record, "text")).is_empty() {
            continue;
        }
        if _names_nothing(record) {
            continue;
        }
        return record.clone();
    }
    current(memory).cloned().unwrap_or_else(Value::dict)
}

/// `_names_nothing(record)`: a topic record says nothing of its own about what is being talked
/// about.
pub fn _names_nothing(record: &Value) -> bool {
    if !text::squash(&rget(record, "subject")).is_empty() {
        return false;
    }
    gist(&rget(record, "text")).is_empty()
}

/// `subject(memory)`: what the conversation is about now, as a noun phrase.
pub fn subject(memory: &Memory) -> String {
    let known = memory.get("user").filter(|u| is_dict(u)).cloned().unwrap_or_else(Value::dict);
    let mut record = _about_record(memory);
    if let Some(thing) = _thing_of(memory, &record) {
        record = thing.clone();
    }
    let mut now = text::squash(&rget(&record, "subject"));
    if now.is_empty() {
        let said = rget(&record, "text");
        now = gist(&if said.is_empty() { topic_text(memory) } else { said });
    }
    let planned = gist(&rget(&known, "plan"));
    if !planned.is_empty() && !now.is_empty() && now.contains(&planned) {
        return planned;
    }
    if now.is_empty() { planned } else { now }
}

/// `topic_text(memory)`: the current topic's text, or "".
pub fn topic_text(memory: &Memory) -> String {
    current(memory).map(|r| rget(r, "text")).unwrap_or_default()
}

/// `_the_action_word(line)`: the first word of the line that is the person's own action, and
/// its base form.
pub fn _the_action_word(line: &[String]) -> Option<(usize, String)> {
    for (index, word) in line.iter().enumerate() {
        let mut found: Vec<String> = forms(word).into_iter().filter(|f| ACTION.contains(f)).collect();
        if found.is_empty() {
            continue;
        }
        let before = &line[..index];
        if before.last().is_some_and(|b| ACTION_AFTER.contains(b)) {
            // "the best way to learn spanish" is the person's own doing
        } else if before.iter().any(|b| !PERSON_LEAD.contains(b)) {
            continue;
        }
        // sorted(found, key=len)[0]; Python breaks a tie in set order, here alphabetically
        found.sort_by(|a, b| text::len(a).cmp(&text::len(b)).then_with(|| a.cmp(b)));
        return Some((index, found.swap_remove(0)));
    }
    None
}

/// `action(memory)`: the action the conversation is on ("lose a stone"), or "".
pub fn action(memory: &Memory) -> String {
    let said = words(&topic_text(memory)).join(" ");
    if said.is_empty() {
        return String::new();
    }
    let line = text::split(&said);
    let Some((index, base)) = _the_action_word(&line) else { return String::new() };
    let tail = line[index..].join(" ");
    let tail = ACTION_SPLIT.split(&tail).into_iter().next().unwrap_or_default();
    let mut parts: Vec<String> = text::split(&tail).into_iter().take(*MAX_ACTION as usize).collect();
    parts[0] = base;
    text::strip(&parts.join(" ")).to_string()
}

/// `written_out(text, memory)`: a bare duration ask written out with the action in it, or None.
pub fn written_out(text_: &str, memory: &Memory) -> Option<String> {
    let low = words(text_).join(" ");
    if low.is_empty() || !DURATION_ASK.is_match(&low) {
        return None;
    }
    if text::split(&low).iter().any(|w| !DURATION_WORDS.contains(w)) {
        return None;
    }
    let done = action(memory);
    if done.is_empty() {
        return None;
    }
    let said = text::squash(text_);
    let said = text::rstrip_chars(&said, "?.! ");
    Some(format!("{said} to {done}?"))
}

/// `_named_a_thing(line)`: the thing a line named, or "" when it named an attribute or a state.
pub fn _named_a_thing(line: &str) -> String {
    let thing = gist(line);
    if thing.is_empty() || NOT_A_THING.contains(&thing) {
        return String::new();
    }
    let parts = text::split(&thing);
    if parts.iter().all(|w| ATTRIBUTE_WORDS.contains(w)) {
        return String::new();
    }
    if parts.last().is_some_and(|w| A_STATE.contains(w)) {
        return String::new();
    }
    thing
}

/// `_the_thing_on_the_table(memory)`: the thing the last few lines named, or "".
pub fn _the_thing_on_the_table(memory: &Memory) -> String {
    for turn in memory.turns.iter().rev().take(4) {
        let thing = _named_a_thing(&turn.user);
        if !thing.is_empty() {
            return thing;
        }
    }
    String::new()
}

/// `attribute_of_the_thing(text, memory)`: a line that asks one attribute of the thing on the
/// table, written out with the thing, or None.
pub fn attribute_of_the_thing(text_: &str, memory: &Memory) -> Option<String> {
    let low = words(text_).join(" ");
    let parts = text::split(&low);
    if low.is_empty() || parts.len() > 5 {
        return None;
    }
    if parts.iter().any(|w| !ATTRIBUTE_WORDS.contains(w)) {
        return None;
    }
    let frame = ONE_ATTRIBUTE.iter().find(|(pattern, _)| pattern.is_match(&low)).map(|(_, said)| said.clone())?;
    let thing = _the_thing_on_the_table(memory);
    if thing.is_empty() {
        return None;
    }
    Some(frame.replace("{thing}", &thing))
}



static PRICE_ASK: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PRICE_ASK"));
static ASKS_FOR_ADVICE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASKS_FOR_ADVICE"));
static CAUSE_OF_THE_COMPLAINT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CAUSE_OF_THE_COMPLAINT"));
static REFERRING_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "REFERRING_WORDS"));
static DUMMY_IT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DUMMY_IT"));
static ASK_MORE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_MORE"));
static STAND_IN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STAND_IN"));
static ABOUT_THE_CHAT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ABOUT_THE_CHAT"));
static ELIDED_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ELIDED_ASK"));
static SAME_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SAME_ASK"));
static COUNTING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COUNTING"));
static THE_TAIL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "THE_TAIL"));
static LONG_IT_TAKES: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LONG_IT_TAKES"));
static TOPIC_OPENS_WITH: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TOPIC_OPENS_WITH"));
static STAY_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "STAY_ASK"));
static STAY_ONLY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STAY_ONLY"));
static TRIP_FRAME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TRIP_FRAME"));
pub static LIVES_THERE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LIVES_THERE"));
static POINTING_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "POINTING_WORD"));
static A_VERDICT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_VERDICT"));
static A_BARE_MEASURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_BARE_MEASURE"));
static ABOUT_A_BARE_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ABOUT_A_BARE_THING"));
static A_FREQUENCY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_FREQUENCY"));
static PROPERTY_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PROPERTY_ASK"));
static MONTH_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MONTH_NAMES"));
static SEASONS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SEASONS"));
static WEATHER_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WEATHER_WORDS"));
static ABOUT_NAMED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ABOUT_NAMED"));
static _ABOUT_TAIL_PRONOUN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_ABOUT_TAIL_PRONOUN"));
static ABOUT_MOST_WORDS: LazyLock<i64> = LazyLock::new(|| t::int(M, "ABOUT_MOST_WORDS"));
static DEFINITE_OPEN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DEFINITE_OPEN"));
static ANSWER_OPENING: LazyLock<i64> = LazyLock::new(|| t::int(M, "ANSWER_OPENING"));
static CONSTRAINT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONSTRAINT"));
static CONSTRAINT_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CONSTRAINT_FILLER"));
static CLAUSE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CLAUSE"));
static ASIDE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASIDE"));
static SHIELD: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "SHIELD"));

static ELIDED_LEAD: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"^(?:(?:ok|okay|so|and|also|plus|or|then|right|yeah|yes|oh|em|um)\s+)*(?:how|what)\s+about\s+", 0)
});
static AN_ASK_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:how|when|where|why|which|who)\b", 0));
static A_WORD_OF_LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[A-Za-z']+", 0));
static A_OR_SOME: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:a|an|some)\b", crate::py::re::I));

/// Python's truth of whatever another workstream's function returns (its contract type may still
/// be `()` while it is a stub).
trait PyTruth {
    fn py_truth(&self) -> bool;
}
impl PyTruth for () {
    fn py_truth(&self) -> bool {
        false
    }
}
impl PyTruth for bool {
    fn py_truth(&self) -> bool {
        *self
    }
}
impl PyTruth for String {
    fn py_truth(&self) -> bool {
        !self.is_empty()
    }
}
impl PyTruth for Value {
    fn py_truth(&self) -> bool {
        self.truthy()
    }
}
impl<T: PyTruth> PyTruth for Option<T> {
    fn py_truth(&self) -> bool {
        self.as_ref().is_some_and(PyTruth::py_truth)
    }
}
impl<T> PyTruth for Vec<T> {
    fn py_truth(&self) -> bool {
        !self.is_empty()
    }
}

/// The words of every alternative of every part of a key (`for part in key: for phrase in
/// (part if tuple else (part,)): words(phrase)`).
fn _words_of_parts(parts: &[KeyPart]) -> HashSet<String> {
    let mut found = HashSet::new();
    for part in parts {
        for phrase in lookup::alternatives(part) {
            found.extend(words(phrase));
        }
    }
    found
}

/// `word in keyed or stem(word) in keyed or _said(word, keyed)`.
fn _keyed_word(word: &str, keyed: &HashSet<String>) -> bool {
    if keyed.contains(word) || keyed.contains(&stem(word)) {
        return true;
    }
    let as_list: Vec<String> = keyed.iter().cloned().collect();
    _said(word, &as_list)
}

/// `len(a & b)` for two word sets.
fn _shared(a: &HashSet<String>, b: &HashSet<String>) -> usize {
    a.iter().filter(|w| b.contains(*w)).count()
}

/// `_the_part_a_complaint_is_about(text)`: the body part a line complains about, or None.
pub fn _the_part_a_complaint_is_about(text_: &str) -> Option<String> {
    crate::knowledge::body::the_part_that_hurts(text_)
}

/// `_the_cause_words(text)`: the words after the cause in a complaint, or an empty set.
pub fn _the_cause_words(text_: &str) -> HashSet<String> {
    let low = text_.to_lowercase();
    let Some(found) = CAUSE_OF_THE_COMPLAINT.search(&low) else { return HashSet::new() };
    words(&low[found.end()..]).into_iter().collect()
}

/// `_subject_words_to_carry(record, text, memory)`: the topic's words a follow-up carries, with a
/// complaint's cause left behind.
pub fn _subject_words_to_carry(record: &Value, text_: &str, memory: &Memory) -> Vec<String> {
    let topic_text = if is_dict(record) { rget(record, "text") } else { String::new() };
    let subject_ = key_subject(&topic_text, Some(memory));
    let part = _the_part_a_complaint_is_about(&topic_text).filter(|p| !p.is_empty());
    let cause = _the_cause_words(&topic_text);
    if let Some(part) = part {
        if !cause.is_empty()
            && !words(text_).contains(&part)
            && !_the_part_a_complaint_is_about(text_).is_some_and(|p| !p.is_empty())
        {
            let mut kept: Vec<String> = subject_.into_iter().filter(|w| !cause.contains(w) || *w == part).collect();
            if !kept.contains(&part) {
                kept.insert(0, part);
            }
            return kept;
        }
    }
    subject_
}

/// `_where_the_topic_lands(topic)`: the entry the topic is about, read past the verb it happens to
/// share with an entry's ask.
pub fn _where_the_topic_lands(topic: &str) -> Option<&'static Entry> {
    let found = _entry(topic)?;
    let asked = _words_of_parts(&found.ask);
    if asked.is_empty() {
        return Some(found);
    }
    let all = words(topic);
    let kept: Vec<String> = all.iter().filter(|w| !asked.contains(&stem(w)) && !asked.contains(*w)).cloned().collect();
    if kept.len() == all.len() {
        return Some(found);
    }
    let Some(other) = _entry(&kept.join(" ")) else { return Some(found) };
    if std::ptr::eq(other, found) {
        return Some(found);
    }
    let topic_keys: HashSet<String> = keywords(topic).into_iter().collect();
    if _shared(&_key_words_of(other), &topic_keys) < 2 {
        return Some(found);
    }
    Some(other)
}

/// `an_entry_for(text)`: the knowledge entry a line reaches on its own (the exported entry
/// object), or None.
pub fn an_entry_for(text_: &str) -> Value {
    _entry(text_).map(|e| e.raw.clone()).unwrap_or(Value::None)
}

/// `_the_last_question(memory)`: the last line of the chat that asked something, or "".
pub fn _the_last_question(memory: &Memory) -> String {
    for turn in memory.turns.iter().rev() {
        let said = text::strip(&turn.user).to_string();
        if said.contains('?') || !crate::understanding::understand(&said, Some(memory)).asks.is_empty() {
            return said;
        }
    }
    String::new()
}

/// `how_much_of_the_thing_in_hand(line, memory)`: the last question in the chat, with this
/// line's count of the thing in hand in it, or "".
pub fn how_much_of_the_thing_in_hand(line: &str, memory: &Memory) -> String {
    let said = text::strip(line).to_string();
    if SAME_ASK.match_(&said).is_none() {
        return String::new();
    }
    let body = SAME_ASK.sub("", &said);
    let counted = words(&gist(&body));
    if counted.is_empty() || !counted.iter().all(|w| COUNTING.contains(w) || crate::understanding::pyuni::is_digit(w)) {
        return String::new();
    }
    let asked = _the_last_question(memory);
    if asked.is_empty() {
        return String::new();
    }
    let mut head = String::new();
    for m in THE_TAIL.finditer(&asked) {
        head = asked[..m.end()].to_string();
    }
    if head.is_empty() || crate::understanding::understand(&format!("{head} {body}"), Some(memory)).asks.is_empty() {
        return String::new();
    }
    let before = memory.turns.last().map(|t_| t_.user.clone()).unwrap_or_default();
    let head_words = words(&head);
    let said_words = words(&said);
    let thing: Vec<String> =
        words(&gist(&before)).into_iter().filter(|w| !head_words.contains(w) && !said_words.contains(w)).collect();
    if thing.is_empty() {
        return String::new();
    }
    let mut all = head_words.clone();
    all.extend(counted);
    all.extend(thing);
    let read = all.join(" ");
    if read.is_empty() || _entry(&read).is_none() {
        return String::new();
    }
    read
}

/// `elided_ask(text)`: the line asks about a thing it names with the asking left off.
pub fn elided_ask(text_: &str) -> bool {
    let said = text::strip(&text_.to_lowercase()).to_string();
    if said.is_empty() || words(&said).len() > 5 {
        return false;
    }
    if ELIDED_ASK.match_(&said).is_none() {
        return false;
    }
    let asked = ELIDED_LEAD.sub("", &said);
    !AN_ASK_WORD.is_match(&asked)
}

/// `names_nothing(text)`: a line asks something and names nothing it could be about.
pub fn names_nothing(text_: &str) -> bool {
    _content_words(text_).is_empty()
}

/// `_content_words(text)`: the words of a line that could name a thing of its own.
pub fn _content_words(text_: &str) -> Vec<String> {
    if ABOUT_THE_CHAT.is_match(&text_.to_lowercase()) {
        return Vec::new();
    }
    words(text_)
        .into_iter()
        .filter(|w| !STOPWORDS.contains(w) && !ASK_ONLY.contains(w) && !ASK_MORE.contains(w) && !STAND_IN.contains(w))
        .collect()
}

/// `_the_topic_phrase(topic)`: the topic as the phrase a frame can take ("im buying a house" ->
/// "buying a house"), or "".
pub fn _the_topic_phrase(topic: &str) -> String {
    let said = low_squash(topic);
    let said = text::strip_chars(&said, " .,;:!?");
    let said = TOPIC_OPENS_WITH.sub("", said);
    let said = text::strip(&said);
    if text::split(said).len() >= 2 { said.to_string() } else { String::new() }
}

/// `_a_length_of_the_job(item)`: the entry is keyed to "how long ... take".
pub fn _a_length_of_the_job(item: &'static Entry) -> bool {
    for part in &item.key {
        let said = _words_of_parts(std::slice::from_ref(part));
        if ["take", "takes", "taking"].iter().any(|w| said.contains(*w)) && ["how", "long", "time"].iter().any(|w| said.contains(*w)) {
            return true;
        }
    }
    false
}

/// `the_city_of(topic)`: the city of the days-per-city table the topic names, or None.
pub fn the_city_of(topic: &str) -> Option<String> {
    use crate::knowledge::travel::{_find_in, CITY_DAYS};
    _find_in(&topic.to_lowercase(), &CITY_DAYS).map(str::to_string)
}

/// `the_country_of(topic)`: the country of the where-to-go table the topic names, or None.
pub fn the_country_of(topic: &str) -> Option<String> {
    use crate::knowledge::travel::{_find_in, WHERE_TO_GO};
    _find_in(&topic.to_lowercase(), &WHERE_TO_GO).map(str::to_string)
}

/// `asks_how_long_to_stay(line)`: the line is shaped like an ask for how long.
pub fn asks_how_long_to_stay(line: &str) -> bool {
    STAY_ASK.is_match(&line.to_lowercase())
}

/// `going_there(place, topic)`: the topic says the person is going to the place, not living in
/// it.
pub fn going_there(place: &str, topic: &str) -> bool {
    let said = topic.to_lowercase();
    for m in LIVES_THERE.finditer(&said) {
        let after: String = said[m.end()..].chars().take(40).collect();
        if after.contains(place) {
            return false;
        }
    }
    TRIP_FRAME.is_match(&said)
}

/// `a_stay_with_the_place(line, topic)`: the stay-length ask with the place the topic names in
/// it, or None.
pub fn a_stay_with_the_place(line: &str, topic: &str) -> Option<String> {
    use crate::knowledge::travel::{_find_in, CITY_DAYS};
    let lowered = line.to_lowercase();
    if !STAY_ASK.is_match(&lowered) {
        return None;
    }
    if words(&lowered).iter().any(|w| !STAY_ONLY.contains(w)) {
        return None;
    }
    if _find_in(&lowered, &CITY_DAYS).is_some() {
        return None;
    }
    let place = the_city_of(topic).filter(|p| !p.is_empty()).or_else(|| the_country_of(topic)).filter(|p| !p.is_empty())?;
    if !going_there(&place, topic) {
        return None;
    }
    let said = text::strip(text::rstrip_chars(text::strip(&lowered), "?.!"));
    Some(format!("{said} in {place}"))
}

/// `_the_ways_of_saying_it(stem_of, line, cands)`: (tried, carried) for each candidate, said the
/// two ways it can be said.
pub fn _the_ways_of_saying_it(stem_of: &str, line: &str, cands: &[Vec<String>]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for cand in cands {
        let carried = cand.join(" ");
        out.push((format!("{stem_of} {carried}"), carried));
    }
    if POINTING_WORD.is_match(line) {
        for cand in cands {
            let carried = cand.join(" ");
            let put = POINTING_WORD.sub_with(line, 1, |_| carried.clone());
            out.push((low_squash(&put), carried));
        }
    }
    out
}

/// `_the_last_answer(memory)`: the bot's last reply, or "".
pub fn _the_last_answer(memory: &Memory) -> String {
    memory.turns.iter().rev().map(|t_| t_.bot.clone()).find(|b| !b.is_empty()).unwrap_or_default()
}

/// `_the_topics_on_the_chat(topic=None, memory=None)`: the topic in hand first, then the ones the
/// chat has been on, newest first.
pub fn _the_topics_on_the_chat(topic: &str, memory: &Memory) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for one in records(memory).iter().rev() {
        let said = rget(one, "text");
        if !said.is_empty() && !out.contains(&said) {
            out.push(said);
        }
    }
    if let Some(i) = out.iter().position(|o| o == topic) {
        out.remove(i);
    }
    if !topic.is_empty() {
        out.insert(0, topic.to_string());
    }
    out
}

/// `_the_thing_a_measure_is_of(line, topic=None, memory=None)`: the bare thing a measure ask
/// names, when the chat has an answer to be a follow-up on.
pub fn _the_thing_a_measure_is_of(line: &str, topic: &str, memory: &Memory) -> Option<String> {
    if _the_last_answer(memory).is_empty() {
        return None;
    }
    if !A_BARE_MEASURE.is_match(&line.to_lowercase()) {
        return None;
    }
    let mut on_the_chat: HashSet<String> = HashSet::new();
    for one in _the_topics_on_the_chat(topic, memory) {
        let phrase = _the_topic_phrase(&one);
        on_the_chat.extend(keywords(if phrase.is_empty() { &one } else { &phrase }));
    }
    keywords(line).into_iter().find(|w| !MEASURE_WORDS.contains(w) && !on_the_chat.contains(w))
}

/// `_with_the_word_before(line, word, thing)`: the line with `word` put in front of the thing it
/// asks about, or None.
pub fn _with_the_word_before(line: &str, word: &str, thing: &str) -> Option<String> {
    for found in A_WORD_OF_LETTERS.finditer(line) {
        if stem(&found.as_str().to_lowercase()) == thing {
            let put = format!("{}{} {}", &line[..found.start()], word, &line[found.start()..]);
            return Some(text::strip(&put).to_string());
        }
    }
    None
}

/// `_how_they_feel(word)`: the word says how the person feels about the thing.
pub fn _how_they_feel(word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    if A_VERDICT.contains(word) {
        return true;
    }
    forms(word).iter().any(|f| A_VERDICT.contains(f))
}

/// `_keyed_to(keyed, word)`: a key holds the word, in any of the forms it takes.
pub fn _keyed_to(keyed: &HashSet<String>, word: &str) -> bool {
    if keyed.contains(word) || keyed.contains(&stem(word)) {
        return true;
    }
    keyed.iter().any(|key| forms(key).contains(word))
}

/// `_a_measure_of_the_topic(line, topic=None, memory=None)`: the measure ask widened with a topic
/// word, or None.
pub fn _a_measure_of_the_topic(line: &str, topic: &str, memory: &Memory) -> Option<String> {
    let thing = _the_thing_a_measure_is_of(line, topic, memory)?;
    let said: HashSet<String> = words(line).into_iter().collect();
    for one in _the_topics_on_the_chat(topic, memory) {
        let phrase = _the_topic_phrase(&one);
        for word in keywords(if phrase.is_empty() { &one } else { &phrase }) {
            if said.contains(&word) || MEASURE_WORDS.contains(&word) {
                continue;
            }
            let Some(widened) = _with_the_word_before(line, &word, &thing) else { continue };
            if widened.is_empty() {
                continue;
            }
            let Some(found) = _entry(&widened) else { continue };
            let keyed = _key_words_of(found);
            if keyed.contains(&thing) && _keyed_to(&keyed, &word) {
                return Some(widened);
            }
        }
    }
    None
}

/// `carry_the_ask(line, topic, understood=None, memory=None)`: a follow-up that names nothing
/// takes the topic's own words with it, or None.
pub fn carry_the_ask(line: &str, topic: &str, understood: &Understood, memory: &Memory) -> Option<String> {
    let lowered = line.to_lowercase();
    if ABOUT_THE_CHAT.is_match(&lowered) {
        return None;
    }
    if PROCEED.iter().any(|p| lowered.contains(p.as_str())) {
        return None;
    }
    let elided = elided_ask(line);
    if !ASK_CUE.is_match(&lowered) && !elided {
        return None;
    }
    if let Some(carried) = _carry_the_words(line, topic) {
        return Some(carried);
    }
    if let Some(doing) = _the_ask_with_its_doing(line, topic, None) {
        return Some(doing);
    }
    if let Some(measured) = _a_measure_of_the_topic(line, topic, memory) {
        return Some(measured);
    }
    if elided && !_the_answer_covers_it(line, memory, Some(understood)) {
        return _an_elided_ask_situated(line, topic, Some(understood));
    }
    None
}

/// `not memory` for a memory dict: nothing in it at all.
fn _memory_is_empty(memory: &Memory) -> bool {
    memory.understood.is_none()
        && memory.intent_wants.is_none()
        && !memory.extra_had_intent
        && memory.asked_now.is_none()
        && memory.turns.is_empty()
        && memory.extra.is_empty()
}

/// `_the_answer_covers_it(line, memory, understood=None)`: the answer just given already has the
/// step this line narrows.
pub fn _the_answer_covers_it(line: &str, memory: &Memory, understood: Option<&Understood>) -> bool {
    if _memory_is_empty(memory) {
        return false;
    }
    let mut asked = vec![line.to_string()];
    let typed = understood.map(|u| u.text.clone()).unwrap_or_default();
    if !typed.is_empty() && !asked.contains(&typed) {
        asked.push(typed);
    }
    let put_back: Vec<String> = asked.iter().filter_map(|one| _with_the_article(one)).filter(|p| !p.is_empty()).collect();
    asked.extend(put_back);
    asked.iter().any(|one| crate::memory::detail::answers_the_line(one, memory))
}

/// `_with_the_article(line)`: the line with "the" put back before a bare thing ("what about
/// ceiling"), or None.
pub fn _with_the_article(line: &str) -> Option<String> {
    let found = ABOUT_A_BARE_THING.match_(line)?;
    let (start, _) = found.name_span("thing")?;
    Some(format!("{} the {}", found.name_str("open"), &line[start..]))
}

/// `_only_said_as_a_frequency(topic)`: the words of the topic it says only inside a frequency
/// phrase.
pub fn _only_said_as_a_frequency(topic: &str) -> HashSet<String> {
    let left: HashSet<String> = words(&A_FREQUENCY.sub(" ", topic)).into_iter().collect();
    words(topic).into_iter().filter(|w| !left.contains(w)).collect()
}

/// `_asks_a_property_and_nothing_else(text)`: "what temperature", "and how hot".
pub fn _asks_a_property_and_nothing_else(text_: &str) -> bool {
    let low = text::strip(text_).to_lowercase();
    if PROPERTY_ASK.match_(&low).is_none() {
        return false;
    }
    let found = keywords(&low);
    !found.is_empty() && found.iter().all(|w| MEASURE_WORDS.contains(w))
}

/// `_carry_the_words(line, topic)`: the line widened with the topic's own words, or None when
/// they reach no entry.
pub fn _carry_the_words(line: &str, topic: &str) -> Option<String> {
    if let Some(long_take) = LONG_IT_TAKES.match_(line) {
        let phrase = _the_topic_phrase(topic);
        let tried = if phrase.is_empty() { String::new() } else { format!("{} {} take", long_take.name_str("head"), phrase) };
        let found = if tried.is_empty() { None } else { _entry(&tried) };
        if let Some(found) = found {
            if _a_length_of_the_job(found) && !keeps_away(&tried, &phrase, found) {
                let keyed = _key_words_of(found);
                let carried: HashSet<String> = keywords(&phrase).into_iter().collect();
                if carried.len() >= 2
                    && carried.iter().all(|w| keyed.contains(w) || keyed.contains(&stem(w)) || forms(w).iter().any(|f| keyed.contains(f)))
                {
                    return Some(tried);
                }
            }
        }
    }
    let lowered = line.to_lowercase();
    if (the_city_of(topic).is_some() || the_country_of(topic).is_some()) && STAY_ASK.is_match(&lowered) {
        return a_stay_with_the_place(line, topic);
    }
    let said = words(line);
    let have: HashSet<String> = said.iter().cloned().collect();
    let elsewhere = !_the_trip_of_theirs(topic).is_empty() && !_a_sky_ask(line);
    let lends_no_property = _asks_a_property_and_nothing_else(topic);
    let known: Vec<String> = keywords(topic)
        .into_iter()
        .filter(|w| {
            !have.contains(w)
                && !(lends_no_property && MEASURE_WORDS.contains(w))
                && !(elsewhere && (MONTH_NAMES.contains(w) || SEASONS.contains(w)))
        })
        .collect();
    if known.is_empty() {
        return None;
    }
    let mut anchor = if names_nothing(line) { _where_the_topic_lands(topic) } else { _entry(topic) };
    let extra: Vec<String> = match anchor {
        Some(found) => {
            let keyed = _key_words_of(found);
            let mut extra: Vec<String> = known.iter().filter(|w| _keyed_word(w, &keyed)).cloned().collect();
            let frequency = _only_said_as_a_frequency(topic);
            if !extra.is_empty() && !frequency.is_empty() && extra.iter().all(|w| frequency.contains(w)) {
                extra.clear();
            }
            if extra.is_empty() {
                anchor = None;
                known.clone()
            } else {
                extra
            }
        }
        None => known.clone(),
    };
    if extra.is_empty() {
        return None;
    }
    let stem_of = text::strip(text::rstrip_chars(text::strip(line), "?.!")).to_string();
    let mut cands: Vec<Vec<String>> = vec![extra.clone()];
    for one in &extra {
        for two in &extra {
            if one != two {
                cands.push(vec![one.clone(), two.clone()]);
            }
        }
    }
    for one in &extra {
        cands.push(vec![one.clone()]);
    }
    let mut text_: Option<String> = None;
    let mut item: Option<&'static Entry> = None;
    let mut borrowed = String::new();
    for (tried, carried) in _the_ways_of_saying_it(&stem_of, line, &cands) {
        borrowed = carried.clone();
        let Some(found) = _entry(&tried) else { continue };
        let keyed = _key_words_of(found);
        if !text::split(&carried).iter().all(|w| _keyed_word(w, &keyed)) {
            continue;
        }
        text_ = Some(tried);
        item = Some(found);
        break;
    }
    let mut item = item?;
    let mut text_ = text_.unwrap();
    let keyed = _key_words_of(item);
    let borrowed_words = text::split(&borrowed);
    let lean: Vec<String> = borrowed_words.iter().filter(|w| _keyed_word(w, &keyed)).cloned().collect();
    if !lean.is_empty() && lean.len() < borrowed_words.len() {
        let tried = format!("{stem_of} {}", lean.join(" "));
        if let Some(found) = _entry(&tried) {
            text_ = tried;
            item = found;
            borrowed = lean.join(" ");
        }
    }
    let is_anchor = anchor.is_some_and(|a| std::ptr::eq(a, item));
    if !_names_it(&said, item) && !names_nothing(line) && !is_anchor {
        return None;
    }
    if let Some(anchor) = anchor.filter(|_| !is_anchor) {
        if _shared(&_key_words_of(item), &_key_words_of(anchor)) < 2 {
            return None;
        }
    } else if anchor.is_none() {
        let topic_keys: HashSet<String> = keywords(topic).into_iter().collect();
        if _shared(&_key_words_of(item), &topic_keys) < 2 && topic_keys.len() > 1 {
            return None;
        }
    }
    if keeps_away(&text_, &borrowed, item) {
        return None;
    }
    Some(text_)
}

type SituatedKey = (String, String, Option<String>);

/// `_elided_situated_cache`: the last situated readings (cleared past 32), kept for the life of
/// the process as in Python.
static _ELIDED_SITUATED_CACHE: LazyLock<std::sync::Mutex<IndexMap<SituatedKey, String>>> =
    LazyLock::new(|| std::sync::Mutex::new(IndexMap::new()));

/// `_an_elided_ask_situated(line, topic, understood=None)`: the line read with the situation the
/// chat is on ("any games kid"), or None.
pub fn _an_elided_ask_situated(line: &str, topic: &str, understood: Option<&Understood>) -> Option<String> {
    let said = words(line);
    let have: HashSet<String> = said.iter().cloned().collect();
    let known: Vec<String> = keywords(topic).into_iter().filter(|w| !have.contains(w)).collect();
    if known.is_empty() {
        return None;
    }
    let understood = understood?;
    let stem_of = text::strip(text::rstrip_chars(text::strip(line), "?.!")).to_string();
    let first: Vec<String> = known.iter().take(8).cloned().collect();
    let situation: SituatedKey = (stem_of.clone(), first.join(" "), understood.topic.clone());
    if let Some(memo) = _ELIDED_SITUATED_CACHE.lock().unwrap().get(&situation) {
        return if memo.is_empty() { None } else { Some(memo.clone()) };
    }
    let mut found: Option<String> = None;
    for word in &first {
        let tried = format!("{stem_of} {word}");
        let Some(item) = _entry(&tried) else { continue };
        if !_names_it(&said, item) {
            continue;
        }
        if !lookup::about_the_topic(item, understood) {
            continue;
        }
        if keeps_away(&tried, word, item) {
            continue;
        }
        found = Some(tried);
        break;
    }
    let mut cache = _ELIDED_SITUATED_CACHE.lock().unwrap();
    if cache.len() > 32 {
        cache.clear();
    }
    cache.insert(situation, found.clone().unwrap_or_default());
    found
}

/// Python's `str(entry.reply)`: the text, or the repr of a tuple / list of lines, or a
/// function's repr.
fn _reply_as_str(item: &Entry) -> String {
    use crate::knowledge::entry::Reply;
    let raw = item.raw.attr("reply");
    match &item.reply {
        Reply::Text(s) => s.clone(),
        Reply::Choices(lines) => {
            let shown: Vec<String> = lines.iter().map(|l| crate::understood::py_str_repr(l)).collect();
            match raw {
                Some(Value::List(_)) => format!("[{}]", shown.join(", ")),
                _ if shown.len() == 1 => format!("({},)", shown[0]),
                _ => format!("({})", shown.join(", ")),
            }
        }
        // `<function name at 0x...>`: never the opening of a reply anyone was given
        Reply::Fn(f) => format!("<function {}>", f.name().rsplit(':').next().unwrap_or("")),
    }
}

/// `_the_same_answer_again(item, memory)`: the entry's answer is the answer the person already
/// has, or opens as it does.
pub fn _the_same_answer_again(item: &'static Entry, memory: &Memory) -> bool {
    let reply = low_squash(&_reply_as_str(item));
    if reply.is_empty() {
        return false;
    }
    let n = *ANSWER_OPENING as isize;
    let opening = text::slice(&reply, 0, Some(n));
    let start = memory.turns.len().saturating_sub(2);
    for turn in &memory.turns[start..] {
        let said = low_squash(&turn.bot);
        if !said.is_empty() && (said.contains(&opening) || reply.contains(&text::slice(&said, 0, Some(n)))) {
            return true;
        }
    }
    false
}

/// `the_thing_the_subject_brings(understood, memory)`: the widened line for a line that asks
/// about a thing of the subject ("shoes for running"), or None.
pub fn the_thing_the_subject_brings(understood: &Understood, memory: &Memory) -> Option<String> {
    let clean_or_text = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let text_or_clean = if understood.text.is_empty() { &understood.clean } else { &understood.text };
    let text_ = low_squash(clean_or_text);
    let raw = low_squash(text_or_clean);
    let mut lines = vec![text_.clone(), raw];
    if let Some(turn) = memory.turns.last() {
        let typed = low_squash(&turn.user);
        if !typed.is_empty() {
            lines.insert(0, typed);
        }
    }
    let mut hit = None;
    for said_as in &lines {
        if let Some(found) = ABOUT_NAMED.match_(said_as) {
            hit = Some((said_as.clone(), found.name_str("thing").to_string(), found.end()));
            break;
        }
    }
    let (said_as, thing_said, end) = hit?;
    let thing = _ABOUT_TAIL_PRONOUN.sub("", text::strip(&thing_said));
    let thing = text::strip(&thing).to_string();
    if thing.is_empty() || text::split(&thing).len() as i64 > *ABOUT_MOST_WORDS {
        return None;
    }
    let typed = _ABOUT_TAIL_PRONOUN.sub("", text::strip(&said_as[end..]));
    if DEFINITE_OPEN.match_(&thing).is_some() || DEFINITE_OPEN.match_(&typed).is_some() {
        return None;
    }
    let kind = if typed.is_empty() { thing.clone() } else { typed.clone() };
    if !A_OR_SOME.is_match(&kind) {
        let last = text::split(&kind).last().cloned().unwrap_or_default();
        if !(last.ends_with('s') && !last.ends_with("ss") && text::len(&last) > 3) {
            return None;
        }
    }
    let thing_words: Vec<String> =
        words(&thing).into_iter().filter(|w| !STOPWORDS.contains(w) && !ASK_ONLY.contains(w) && text::len(w) > 2).collect();
    if thing_words.is_empty() {
        return None;
    }
    let record = carried_topic(memory, &text_)?.clone();
    let mut spoke_of: HashSet<String> = HashSet::new();
    let start = memory.turns.len().saturating_sub(2);
    for turn in &memory.turns[start..] {
        spoke_of.extend(words(&turn.bot).iter().map(|w| stem(w)));
    }
    if thing_words.iter().any(|w| spoke_of.contains(&stem(w))) {
        return None;
    }
    let subject_words: Vec<String> =
        key_subject(&rget(&record, "text"), Some(memory)).into_iter().filter(|w| !thing_words.contains(w)).collect();
    if subject_words.is_empty() {
        return None;
    }
    for subject_ in &subject_words {
        for candidate in [format!("{thing} for {subject_}"), format!("{thing} {subject_}")] {
            let Some(item) = _entry(&candidate) else { continue };
            let keyed = _key_words_of(item);
            let named = thing_words.iter().any(|w| _keyed_word(w, &keyed));
            let about_the_subject = subject_words.iter().any(|w| _keyed_word(w, &keyed));
            if !(named && about_the_subject) {
                continue;
            }
            if let Some(anchor) = _entry(&rget(&record, "text")) {
                if !std::ptr::eq(anchor, item) && _the_same_answer_again(item, memory) {
                    continue;
                }
            }
            return Some(candidate);
        }
    }
    None
}

/// `follows_on(understood, text)`: the engine read the line as carrying on, and the line itself
/// points back at something.
pub fn follows_on(understood: &Understood, text_: &str) -> bool {
    let said = DUMMY_IT.sub(" ", text_);
    if !words(&said).iter().any(|w| REFERRING_WORDS.contains(w)) {
        return false;
    }
    if understood.followup {
        return true;
    }
    understood.reference.followup
}

/// `keeps_away(line, topic, item)`: the entry the widened line reaches keeps itself away from a
/// word of the topic.
pub fn keeps_away(line: &str, topic: &str, item: &'static Entry) -> bool {
    keeps_away_from(line, topic, Some(item))
}

/// `keeps_away(line, topic, item=None)` with the entry left to be looked up from the line.
pub fn keeps_away_from(line: &str, topic: &str, item: Option<&'static Entry>) -> bool {
    let item = match item {
        Some(i) => Some(i),
        None => _entry(line),
    };
    let Some(item) = item else { return false };
    let said = words(topic);
    item.avoid.iter().any(|phrase| _kept_away_from(phrase, &said))
}

/// `_kept_away_from(phrase, said)`: the entry's avoidance phrase is one the topic says.
pub fn _kept_away_from(phrase: &str, said: &[String]) -> bool {
    let phrase_words = words(phrase);
    if !phrase_words.iter().any(|w| STOPWORDS.contains(w)) {
        return _said(phrase, said);
    }
    let wanted: Vec<String> = phrase_words.iter().map(|w| stem(w)).collect();
    let have: Vec<String> = said.iter().map(|w| stem(w)).collect();
    if wanted.is_empty() || !wanted.iter().all(|w| have.contains(w)) {
        return false;
    }
    _said(phrase, said)
}

/// `_names_it(found, item)`: the line itself says a word of the entry key.
pub fn _names_it(found: &[String], item: &'static Entry) -> bool {
    let asks = |w: &String| ASK_PART.contains(w) || ASKS_FOR_ADVICE.contains(w);
    let content: HashSet<&String> = found.iter().filter(|w| !STOPWORDS.contains(*w)).collect();
    let asked_only = content.iter().all(|w| asks(w));
    for part in &item.key {
        for phrase in lookup::alternatives(part) {
            let named: HashSet<String> = words(phrase).into_iter().filter(|w| !STOPWORDS.contains(w)).collect();
            if !named.is_empty() && named.iter().all(asks) && !asked_only {
                continue;
            }
            if _said(phrase, found) {
                return true;
            }
        }
    }
    false
}

/// `_key_words_of(item)`: every word an entry's key says it is about.
pub fn _key_words_of(item: &'static Entry) -> HashSet<String> {
    _words_of_parts(&item.key)
}

/// `_names_it_as_written(found, item)`: the line says a word of the entry's key as it is written,
/// not a form of it.
pub fn _names_it_as_written(found: &[String], item: &'static Entry) -> bool {
    let have: HashSet<String> = found.iter().map(|w| w.to_lowercase()).collect();
    for part in &item.key {
        for phrase in lookup::alternatives(part) {
            if !_said(phrase, found) {
                continue;
            }
            if words(phrase).iter().any(|w| have.contains(&w.to_lowercase())) {
                return true;
            }
        }
    }
    false
}

/// `_asked_only(found, item)`: the only parts of the entry key the line matches are asking words.
pub fn _asked_only(found: &[String], item: &'static Entry) -> bool {
    let mut named = false;
    for part in &item.key {
        for phrase in lookup::alternatives(part) {
            if !_said(phrase, found) {
                continue;
            }
            if words(phrase).iter().all(|w| ASK_PART.contains(w) || PRICE_ASK.contains(w)) {
                continue;
            }
            named = true;
        }
    }
    !named
}

/// `_a_sky_ask(line)`: a line asking about the weather.
pub fn _a_sky_ask(line: &str) -> bool {
    words(line).iter().any(|w| WEATHER_WORDS.contains(w))
}

/// `_the_step_they_report(line)`: the line reports a step it has taken rather than asking about
/// the topic. A reading that fails counts as no report, as Python's `except` has it.
pub fn _the_step_they_report(line: &str) -> bool {
    crate::runtime::guarded(|| {
        crate::memory::already::the_step_they_took(line, None).py_truth() || crate::memory::repeat::_tried(line)
    })
    .unwrap_or(false)
}

/// `widen(line, subject_words, topic_text)`: the line with the whole of the topic's subject on the
/// end, when that reaches an entry.
pub fn widen(line: &str, subject_words: &[String], topic_text: &str) -> Option<String> {
    if a_placeholder_pronoun(line) {
        return None;
    }
    if _the_step_they_report(line) {
        return None;
    }
    let topic_entry = _entry(topic_text);
    let have = words(line);
    let elsewhere = !_the_trip_of_theirs(topic_text).is_empty() && !_a_sky_ask(line);
    let extra: Vec<String> = subject_words
        .iter()
        .filter(|w| !have.contains(w) && !(elsewhere && (MONTH_NAMES.contains(*w) || SEASONS.contains(*w))))
        .cloned()
        .collect();
    if extra.is_empty() {
        return None;
    }
    let text_ = format!("{line} {}", extra.join(" "));
    let item = _entry(&text_)?;
    if topic_entry.is_some_and(|t_| std::ptr::eq(t_, item)) {
        return None;
    }
    if !_names_it(&have, item) {
        return None;
    }
    if _asked_only(&have, item) && !_names_it_as_written(&extra, item) {
        return None;
    }
    if keeps_away(&text_, topic_text, item) {
        return None;
    }
    Some(text_)
}

/// `adds_only_a_constraint(line)`: the limit the line names, when the line is made of nothing but
/// that limit.
pub fn adds_only_a_constraint(line: &str) -> Option<String> {
    let m = CONSTRAINT.search(line)?;
    let rest = words(&format!("{} {}", &line[..m.start()], &line[m.end()..]));
    if rest.iter().any(|w| !CONSTRAINT_FILLER.contains(w) && !crate::understanding::pyuni::is_digit(w)) {
        return None;
    }
    Some(m.as_str().to_string())
}

/// `_constrains_the_ask(line, topic_text)`: the question just answered, asked again with this
/// line's limit on the end.
pub fn _constrains_the_ask(line: &str, topic_text: &str) -> Option<String> {
    adds_only_a_constraint(line)?;
    let asked = text::strip(topic_text);
    if asked.is_empty() {
        return None;
    }
    let mine = _entry(asked);
    let text_ = format!("{asked} {}", text::strip(line));
    let item = _entry(&text_)?;
    if mine.is_some_and(|m| !std::ptr::eq(m, item)) {
        return None;
    }
    Some(text_)
}

/// `_clauses_of(sentence)`: the clauses of a sentence, with a bracketed aside kept whole.
pub fn _clauses_of(sentence: &str) -> Vec<String> {
    let shield: &str = &SHIELD;
    let shielded = ASIDE.sub_fn(sentence, |m| m.as_str().replace(',', shield));
    CLAUSE.split(&shielded).into_iter().map(|piece| piece.replace(shield, ",")).collect()
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
    pub fn _the_length_of_the_aim(understood: &Understood, memory: &Memory) -> Option<Understood> {
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

