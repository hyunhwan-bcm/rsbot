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
pub fn _entry(text_: &str) -> Option<&'static Entry> {
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
    deps::string("bot.memory.facts:a_moved_place", deps::args1(text_), || crate::memory::facts::a_moved_place(text_))
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
pub fn _the_topics_on_the_chat(topic: &str, memory: Option<&Memory>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for one in memory.map(records).unwrap_or(&[]).iter().rev() {
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
pub fn _the_thing_a_measure_is_of(line: &str, topic: &str, memory: Option<&Memory>) -> Option<String> {
    if memory.map(_the_last_answer).unwrap_or_default().is_empty() {
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
pub fn _a_measure_of_the_topic(line: &str, topic: &str, memory: Option<&Memory>) -> Option<String> {
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
    carry_the_ask_with(line, topic, Some(understood), Some(memory))
}

/// `carry_the_ask(line, topic, understood=None, memory=None)` with the reading and the memory
/// optional, as `_widened` calls it.
pub fn carry_the_ask_with(line: &str, topic: &str, understood: Option<&Understood>, memory: Option<&Memory>) -> Option<String> {
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
    if elided && !memory.is_some_and(|m| _the_answer_covers_it(line, m, understood)) {
        return _an_elided_ask_situated(line, topic, understood);
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
    asked.iter().any(|one| deps::truth("bot.memory.detail:answers_the_line", serde_json::json!([one, null]), || crate::memory::detail::answers_the_line(one, memory)))
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
pub(crate) fn _reply_as_str(item: &Entry) -> String {
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
        deps::truth("bot.memory.already:the_step_they_took", deps::args1(line), || crate::memory::already::the_step_they_took(line, None).py_truth())
            || deps::truth("bot.memory.repeat:_tried", deps::args1(line), || crate::memory::repeat::_tried(line))
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



static SENTENCE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SENTENCE"));
static JOINED_ON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "JOINED_ON"));
static TIME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TIME"));
pub static AMOUNT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "AMOUNT"));
static AMOUNT_ANSWER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "AMOUNT_ANSWER"));
static RATE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RATE"));
static RATE_BY_WEIGHT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RATE_BY_WEIGHT"));
static OWN_SIZE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "OWN_SIZE"));
static PLUS_MINUTES: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PLUS_MINUTES"));
static WEIGHT_IN_GRAMS: LazyLock<Vec<(String, f64)>> = LazyLock::new(|| {
    t::value(M, "WEIGHT_IN_GRAMS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_f64().unwrap())).collect()
});
static TIME_SPAN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TIME_SPAN"));
static DURATION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DURATION"));
static SPAN_ADJECTIVE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SPAN_ADJECTIVE"));
static LENGTH_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LENGTH_ASK"));
static THRESHOLD_SPAN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "THRESHOLD_SPAN"));
static WARNING_SPAN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WARNING_SPAN"));
static FREQUENCY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FREQUENCY"));
static HYPOTHETICAL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HYPOTHETICAL"));
static ANSWER_KEPT: LazyLock<i64> = LazyLock::new(|| t::int(M, "ANSWER_KEPT"));
static ACKED_IN_BETWEEN: LazyLock<i64> = LazyLock::new(|| t::int(M, "ACKED_IN_BETWEEN"));
pub static REPEAT: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "REPEAT"));
pub static REPEAT_LEADS: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| t::tuples(M, "REPEAT_LEADS"));
static ACK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ACK"));
static WARM_ACK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WARM_ACK"));
static ACK_COURTESY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ACK_COURTESY"));
static A_CLOSE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_CLOSE"));
static CLOSE_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CLOSE_FILLER"));
pub static SHORT_TURN: LazyLock<i64> = LazyLock::new(|| t::int(M, "SHORT_TURN"));
static LOOK_BACK: LazyLock<i64> = LazyLock::new(|| t::int(M, "LOOK_BACK"));
static FIGURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FIGURE"));
static POINTS_AT_THE_ANSWER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "POINTS_AT_THE_ANSWER"));
static _ASKING_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_ASKING_VERBS"));
static ASKS_A_FREQUENCY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKS_A_FREQUENCY"));
static ADVICE_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ADVICE_ASK"));
static ADVICE_NAMES_A_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ADVICE_NAMES_A_THING"));
static PAIRING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PAIRING"));
static PAIRING_OWN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PAIRING_OWN"));
pub static IMPERATIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "IMPERATIVE"));
static SENDS_YOU_ON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SENDS_YOU_ON"));
static WHEN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WHEN"));
pub static RED_FLAG: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RED_FLAG"));
static ASKS_THEM_BACK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKS_THEM_BACK"));
static A_CEILING_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_CEILING_ASK"));
static A_CEILING_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_CEILING_WORD"));
static _A_RATE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_RATE"));
static AFTERWARDS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "AFTERWARDS"));
static TOTAL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TOTAL"));
static OPTION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "OPTION"));
static QUOTE_MARK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "QUOTE_MARK"));
static OPTION_ANY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "OPTION_ANY"));
static TIME_ONLY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TIME_ONLY"));
static A_WEATHER_WINDOW: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_WEATHER_WINDOW"));
static PERIODS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| t::map_int(M, "PERIODS").into_iter().collect());
static PERIOD_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PERIOD_ASK"));
static PERIOD_LEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PERIOD_LEAD"));
static FIGURED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FIGURED"));
static MEASURED: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MEASURED"));
static _ASKS_WHAT_I_AM: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_ASKS_WHAT_I_AM"));
static STEP_ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STEP_ASK_WORDS"));
static REQUEST_A_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "REQUEST_A_THING"));
static A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_THING"));
static MOVED_KEEP: LazyLock<i64> = LazyLock::new(|| t::int(M, "MOVED_KEEP"));
static POINTS_BACK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "POINTS_BACK"));
static A_QUESTION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_QUESTION"));
static BARE_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BARE_ASK"));
static BARE_WORDS: LazyLock<i64> = LazyLock::new(|| t::int(M, "BARE_WORDS"));
static ASKS_ABOUT_THEMSELVES: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKS_ABOUT_THEMSELVES"));
static DETAIL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DETAIL"));

static HOW_MUCH: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhow much\b", crate::py::re::I));
static ONE_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bone\b", crate::py::re::I));
static SPACES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+", 0));
static LETTERS_OR_APOSTROPHE: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static LOWER_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9']+", 0));
static TRAILING_PUNCT: LazyLock<Re> = LazyLock::new(|| Re::new(r"[\s,;:]+$", 0));
static TRAILING_JOIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+(?:and|or|to)$", 0));
static NOT_A_NUMBER_CHAR: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^0-9.]", 0));
static NUMBER_OR_SPACE: LazyLock<Re> = LazyLock::new(|| Re::new(r"[\d.\s]", 0));
static NOT_A_DIGIT_OR_DOT: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^\d.]", 0));
static IN_GRAMS: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\s*(\d+(?:\.\d+)?)\s*(kg|g|lb|lbs|pound|pounds|oz|ounce|ounces)\b", crate::py::re::I)
});
static SO_SPLIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"[,;]?\s+so\b", 0));
static ABOUT_LEAD: LazyLock<Re> = LazyLock::new(|| Re::new(r"^about\s+", crate::py::re::I));

/// What another workstream's function returned, as Python's str-or-None (its contract type may
/// still be `()` while it is a stub).
trait PyStrRet {
    fn py_str(self) -> Option<String>;
}
impl PyStrRet for () {
    fn py_str(self) -> Option<String> {
        None
    }
}
impl PyStrRet for Option<String> {
    fn py_str(self) -> Option<String> {
        self
    }
}
impl PyStrRet for String {
    fn py_str(self) -> Option<String> {
        if self.is_empty() { None } else { Some(self) }
    }
}
impl PyStrRet for Value {
    fn py_str(self) -> Option<String> {
        match self {
            Value::Str(s) => Some(s.to_string()),
            _ => None,
        }
    }
}

/// The weight of one unit in grams (`WEIGHT_IN_GRAMS[unit]`).
fn _grams_of(unit: &str) -> Option<f64> {
    WEIGHT_IN_GRAMS.iter().find(|(k, _)| k == unit).map(|(_, v)| *v)
}

/// `quoted.sentences(text, SENTENCE)`.
fn _sentences(text_: &str) -> Vec<String> {
    deps::strings("bot.memory.quotes:sentences", serde_json::json!([text_, null]), || crate::memory::quotes::sentences(text_, t::value(M, "SENTENCE")))
}

/// The upper-cased first letter (`s[:1].upper() + s[1:]`).
fn _upper_first(s: &str) -> String {
    text::upper_first(s)
}

/// A turn as Python's dict `{"user", "bot", "module"}`.
fn _turn_value(turn: &crate::state::Turn) -> Value {
    let mut out = Value::dict();
    out.set_key("user", turn.user.as_str());
    out.set_key("bot", turn.bot.as_str());
    out.set_key("module", turn.module.as_deref().map(Value::from).unwrap_or(Value::None));
    out
}

/// `lst[-n:]` with Python's slice rules for n.
fn _last<T>(items: &[T], n: i64) -> &[T] {
    let len = items.len() as i64;
    let start = if n > 0 { (len - n).max(0) } else if n == 0 { 0 } else { (-n).min(len) };
    &items[start as usize..]
}

/// `remember_answer(memory, reply, keys)`: keep the whole answer an entry gave, with the words it
/// is keyed on.
pub fn remember_answer(memory: &mut Memory, reply: &str, keys: &[String]) {
    if text::strip(reply).is_empty() {
        return;
    }
    let mut sorted_keys: Vec<String> = keys.to_vec();
    sorted_keys.sort();
    let mut record = Value::dict();
    record.set_key("text", reply);
    record.set_key("keys", Value::from(sorted_keys));
    memory.set("entry_answer", record.clone());
    let mut kept: Vec<Value> = match memory.get("entry_answers") {
        Some(Value::List(l)) => l.as_ref().clone(),
        _ => Vec::new(),
    };
    kept.push(record);
    let kept = _last(&kept, *ANSWER_KEPT).to_vec();
    memory.set("entry_answers", Value::from(kept));
}

/// The kept answers (`entry_answers`, or `[entry_answer]` when there are none).
fn _kept_answers(memory: &Memory) -> Vec<Value> {
    match memory.get("entry_answers") {
        Some(Value::List(l)) if !l.is_empty() => l.as_ref().clone(),
        _ => vec![memory.get("entry_answer").cloned().unwrap_or(Value::None)],
    }
}

/// One of the last few replies opens the way this answer does.
fn _really_said(answer: &str, memory: &Memory) -> bool {
    let low = answer.to_lowercase();
    let opening = text::split(&low).into_iter().take(8).collect::<Vec<_>>().join(" ");
    _last(&memory.turns, *ANSWER_KEPT).iter().any(|turn| low_squash(&turn.bot).starts_with(&opening))
}

/// `entry_answer(memory, topic_text)`: the whole answer the entry for this topic gave, while it is
/// still what we are on.
pub fn entry_answer(memory: &Memory, topic_text: &str) -> Option<String> {
    let mine = _the_forms_of_all(keywords(topic_text));
    let mut best: Option<String> = None;
    let mut best_score = 0;
    for record in _kept_answers(memory) {
        if !is_dict(&record) {
            continue;
        }
        let said = text::strip(&rget(&record, "text")).to_string();
        if said.is_empty() {
            continue;
        }
        if !_really_said(&said, memory) {
            continue;
        }
        let keys: Vec<String> = record.get("keys").map(Value::strs).unwrap_or_default();
        if keys.is_empty() {
            continue;
        }
        let mut forms_ = _the_forms_of_all(keys);
        forms_.retain(|f| !STOPWORDS.contains(f));
        let score = mine.iter().filter(|w| forms_.contains(*w)).count();
        if score > 0 && score >= best_score {
            best = Some(said);
            best_score = score;
        }
    }
    best
}

/// `{form for word in found for form in (word, stem(word))}`.
fn _the_forms_of_all(found: impl IntoIterator<Item = String>) -> HashSet<String> {
    let mut out = HashSet::new();
    for word in found {
        out.insert(stem(&word));
        out.insert(word);
    }
    out
}

/// The turn `answer_in_hand` reads, as a turn.
fn _the_turn_in_hand(memory: &Memory) -> Option<&crate::state::Turn> {
    let turns = &memory.turns;
    if turns.is_empty() {
        return None;
    }
    let mut index = turns.len() - 1;
    let mut stepped = 0;
    while index > 0 && stepped < *ACKED_IN_BETWEEN && is_ack(&turns[index].user) {
        index -= 1;
        stepped += 1;
    }
    Some(&turns[index])
}

/// `answer_in_hand(memory)`: the bot turn that answered the subject the chat is on, reading back
/// over acknowledgements (the turn dict), or None.
pub fn answer_in_hand(memory: &Memory) -> Value {
    _the_turn_in_hand(memory).map(_turn_value).unwrap_or(Value::None)
}

/// `newest_answer(memory)`: the answer we gave most recently, while it is still one we really
/// gave.
pub fn newest_answer(memory: &Memory) -> Option<String> {
    for record in _kept_answers(memory).iter().rev() {
        if !is_dict(record) {
            continue;
        }
        let said = text::strip(&rget(record, "text")).to_string();
        if said.is_empty() {
            continue;
        }
        if _really_said(&said, memory) {
            return Some(said);
        }
    }
    None
}

/// `_carries(said, mine)`: a bot turn is this answer word for word, with or without a repeat's
/// lead-in.
pub fn _carries(said: &str, mine: &str) -> bool {
    if said.is_empty() {
        return false;
    }
    if said == mine {
        return true;
    }
    REPEAT_LEADS.iter().any(|lead| said == format!("{} {mine}", lead.join(" ")))
}

/// `_only_a_close(text)`: a line does nothing but send the person off.
pub fn _only_a_close(text_: &str) -> bool {
    let Some(found) = A_CLOSE.search(text_) else { return false };
    let rest = format!("{} {}", &text_[..found.start()], &text_[found.end()..]);
    for word in text::split(&rest) {
        let word = text::strip_chars(&word, ".,!?-\u{2014}\u{2013}\"'()").to_lowercase();
        if !word.is_empty() && !CLOSE_FILLER.contains(&word) {
            return false;
        }
    }
    true
}

/// `only_acknowledges(said)`: a bot line only acknowledges the person's own line and adds nothing.
pub fn only_acknowledges(said: &str) -> bool {
    let line = text::strip(said);
    if line.is_empty() || text::split(line).len() as i64 > *SHORT_TURN {
        return false;
    }
    if ACK.match_(line).is_some() {
        return true;
    }
    if _only_a_close(line) {
        return true;
    }
    WARM_ACK.is_match(line) && (ACK_COURTESY.is_match(line) || !line.contains(','))
}

/// `_hands_the_subject_back(said, memory)`: a turn answered a remark on the turn before.
pub fn _hands_the_subject_back(said: &str, memory: &Memory) -> bool {
    matches!(memory.get("hand_backs"), Some(Value::List(kept)) if kept.iter().any(|k| k.as_str() == Some(said)))
}

/// `_the_opening_of(reply)`: the first sentence of an answer (two when the first is short).
pub fn _the_opening_of(reply: &str) -> String {
    let body = text::strip(reply).to_string();
    let parts = _sentences(&body);
    if parts.len() < 2 {
        return body;
    }
    let n = if text::len(&parts[0]) >= 60 { 1 } else { 2 };
    let kept: Vec<String> = parts[..n].iter().map(|p| text::strip(p).to_string()).collect();
    let said = kept.join(" ");
    deps::string("bot.memory.quotes:closed", serde_json::json!([said, parts[n..], body]), || crate::memory::quotes::closed(&said, &parts[n..], &body))
}

/// `said_again(reply, memory, line)`: the reply, framed as the repeat it is when it is already on
/// the screen, or None.
pub fn said_again(reply: &str, memory: &Memory, line: &str) -> Option<String> {
    said_again_on(reply, memory, Some(line))
}

/// `said_again(reply, memory, line=None)` with the line optional.
pub fn said_again_on(reply: &str, memory: &Memory, line: Option<&str>) -> Option<String> {
    if deps::truth("bot.memory.again:carries_a_lead_in", deps::args1(reply), || crate::memory::again::carries_a_lead_in(reply)) {
        return None;
    }
    let mine = words(reply).join(" ");
    if line.is_some_and(pairing) {
        return None;
    }
    if let Some(line) = line {
        if deps::truth("bot.memory.returned:says_it_happened_again", deps::args1(line), || crate::memory::returned::says_it_happened_again(line)) {
            return None;
        }
        if keywords(line).iter().any(|w| !ASK_ONLY.contains(w) && !POINTS_AT_THE_ANSWER.contains(w)) || FIGURE.is_match(line) {
            return None;
        }
    }
    if mine.is_empty() {
        return None;
    }
    let mut times = 0;
    let mut seen = 0;
    for turn in memory.turns.iter().rev() {
        if seen >= *LOOK_BACK {
            break;
        }
        seen += 1;
        let said = words(&turn.bot).join(" ");
        if _carries(&said, &mine) || said.contains(&mine) {
            times += 1;
            continue;
        }
        if _hands_the_subject_back(&said, memory) || only_acknowledges(&turn.bot) {
            continue;
        }
        break;
    }
    if times == 0 {
        return None;
    }
    let mut reply = reply.to_string();
    let short = _the_opening_of(&reply);
    if !short.is_empty() && words(&short).join(" ") != mine {
        reply = short;
    }
    let frame = &REPEAT[(times as usize).min(REPEAT.len()) - 1];
    Some(frame.replace("{reply}", &reply))
}

/// `_said_later_than(turn, record)`: `turn` was said after `record` (both topic records).
pub fn _said_later_than(turn: &Value, record: &Value) -> bool {
    let Some(Value::Int(mine)) = turn.get("turn") else { return false };
    match record.get("turn") {
        Some(Value::Int(theirs)) => mine > theirs,
        _ => true,
    }
}

/// `_the_turn_we_lean_on(understood, memory)`: the turn the reading says this line leans on, as
/// a topic record, or None.
pub fn _the_turn_we_lean_on(understood: &Understood, memory: &Memory) -> Value {
    let said_text = understood.leans_on.as_ref().map(|l| l.text.clone()).unwrap_or_default();
    let wanted = words(&said_text).join(" ");
    if wanted.is_empty() || memory.turns.is_empty() {
        return Value::None;
    }
    let turns = &memory.turns;
    for (back, turn) in _last(turns, 8).iter().rev().enumerate() {
        let said = &turn.user;
        if words(said).join(" ") == wanted {
            let mut record = Value::dict();
            record.set_key("turn", (turns.len() - 1 - back) as i64);
            record.set_key("text", said.as_str());
            record.set_key("words", Value::from(words(said)));
            return record;
        }
    }
    Value::None
}

/// `_the_turn_that_answered(answer, memory, keep=6)`: the turn whose reply is `answer`, or None.
pub fn _the_turn_that_answered(answer: &str, memory: &Memory, keep: i64) -> Option<crate::state::Turn> {
    let wanted: Vec<String> = words(answer).into_iter().take(12).collect();
    _last(&memory.turns, keep).iter().rev().find(|turn| words(&turn.bot).into_iter().take(12).collect::<Vec<_>>() == wanted).cloned()
}

/// `_answer_about(line, memory, keep=6)`: the most recent earlier answer this line's own words
/// belong to, or None.
pub fn _answer_about(line: &str, memory: &Memory, keep: i64) -> Option<String> {
    let asking = _FRAME_ASK.is_match(&line.to_lowercase());
    let wanted: HashSet<String> = keywords(line)
        .into_iter()
        .filter(|w| !ASK_ONLY.contains(w) && !(asking && _ASKING_VERBS.contains(&stem(w))))
        .map(|w| stem(&w))
        .collect();
    if wanted.is_empty() {
        return None;
    }
    let turns = _last(&memory.turns, keep);
    if turns.len() < 2 {
        return None;
    }
    let score = |said: &str| -> usize {
        let theirs: HashSet<String> = keywords(said).iter().map(|w| stem(w)).collect();
        wanted.iter().filter(|w| theirs.contains(*w)).count()
    };
    let mut best: Option<String> = None;
    let mut best_score = score(&turns[turns.len() - 1].bot);
    for turn in &turns[..turns.len() - 1] {
        if turn.module.as_deref() != Some("answers") {
            continue;
        }
        let s = score(&turn.bot);
        if s > best_score {
            best = Some(turn.bot.clone());
            best_score = s;
        }
    }
    best
}

/// `answer_about(line, memory)`: the earlier answer this line's own words belong to, when it is
/// not the one just given.
pub fn answer_about(line: &str, memory: &Memory) -> Option<String> {
    _answer_about(line, memory, 6)
}

/// `_the_trip_of_theirs(text)`: the places a line of theirs says they are going to (a set; here
/// in the order the store read them, once each).
pub fn _the_trip_of_theirs(text_: &str) -> Vec<String> {
    let found = deps::value("bot.memory.facts:extract", deps::args1(text_), || crate::memory::facts::extract(text_, false));
    if !found.get("plan").is_some_and(Value::truthy) {
        return Vec::new();
    }
    let mut out: Vec<String> = Vec::new();
    for place in found.get("places").map(Value::items).unwrap_or(&[]) {
        let name = text::strip(&value_str(place)).to_lowercase();
        if !name.is_empty() && !out.contains(&name) {
            out.push(name);
        }
    }
    out
}

/// `the_place_they_are_on(memory)`: the place of the newest trip they have started on, or an
/// empty set.
pub fn the_place_they_are_on(memory: &Memory) -> Vec<String> {
    let turns: Vec<&crate::state::Turn> = memory.turns.iter().filter(|t_| !t_.user.is_empty()).collect();
    let corrected = memory.get("correction").is_some_and(Value::truthy);
    if !corrected {
        for turn in _last(&turns, 2).iter().rev() {
            let here = _the_trip_of_theirs(&turn.user);
            if !here.is_empty() {
                return here;
            }
        }
    }
    let plan = memory.get("user").and_then(|u| u.get("plan")).cloned().unwrap_or(Value::None);
    if let Value::Str(plan) = plan {
        if !text::strip(&plan).is_empty() {
            let here = _the_trip_of_theirs(&format!("we are going to {}", text::strip(&plan)));
            if !here.is_empty() {
                return here;
            }
        }
    }
    Vec::new()
}

/// `about_a_place_they_left_behind(answer, memory)`: the answer on the table is about a place
/// they are no longer going to.
pub fn about_a_place_they_left_behind(answer: &str, memory: &Memory) -> bool {
    let here = the_place_they_are_on(memory);
    if here.is_empty() {
        return false;
    }
    let told: HashSet<String> = memory
        .get("user")
        .and_then(|u| u.get("places"))
        .map(Value::items)
        .unwrap_or(&[])
        .iter()
        .map(|p| text::strip(&value_str(p)).to_lowercase())
        .filter(|p| !p.is_empty())
        .collect();
    let left: HashSet<String> = told.into_iter().filter(|p| !here.contains(p)).collect();
    if left.is_empty() {
        return false;
    }
    words(answer).iter().any(|w| left.contains(w))
}

/// `same_as_before(understood, memory)`: the part of the last answer that this follow-up asks
/// about, or None.
pub fn same_as_before(understood: &Understood, memory: &Memory) -> Option<String> {
    let line = if understood.clean.is_empty() { understood.text.clone() } else { understood.clean.clone() };
    if pairing(&line) {
        return None;
    }
    let last = memory.turns.last()?;
    let drawn = _the_turn_in_hand(memory).unwrap_or(last);
    if understood.kind != "question" {
        return None;
    }
    let proceed = PROCEED.iter().any(|p| line.to_lowercase().contains(p.as_str()));
    if !(leans_on_topic(understood, memory) || asks_only(understood)) {
        return None;
    }
    let record = carried_topic(memory, &line).cloned();
    if left_for_a_blank(memory, &line) {
        return None;
    }
    let asked = record.as_ref().map(|r| rget(r, "text")).unwrap_or_default();
    let mut earlier = _answer_about(&line, memory, 6);
    if earlier.is_some() && !keywords(&line).iter().any(|w| !ASK_ONLY.contains(w)) {
        let on_the_table = _entry(&asked);
        let answered_by = _the_turn_that_answered(earlier.as_deref().unwrap(), memory, 6);
        let its_subject = _entry(&answered_by.map(|t_| t_.user).unwrap_or_default());
        if let Some(on_the_table) = on_the_table {
            if !its_subject.is_some_and(|s| std::ptr::eq(s, on_the_table)) {
                earlier = None;
            }
        }
    }
    let measured = understood.asks.iter().any(|a| MEASURED.contains(a));
    let fresh_: Vec<String> = keywords(&line).into_iter().filter(|w| !ASK_ONLY.contains(w)).collect();
    if earlier.is_none() && measured && !fresh_.is_empty() {
        let theirs: HashSet<String> = key_subject(&asked, None).into_iter().collect();
        if !key_subject(&line, None).iter().any(|w| theirs.contains(w)) {
            let (mine, theirs) = (_entry(&line), _entry(&asked));
            if let (Some(mine), Some(theirs)) = (mine, theirs) {
                if !std::ptr::eq(mine, theirs) {
                    return None;
                }
            }
        }
    }
    if (!asks_only(understood) && _repeats_the_question(&line, &asked)) || HYPOTHETICAL.is_match(&line) {
        return None;
    }
    let mut whole = entry_answer(memory, &asked);
    let asked_keys = keywords(&asked);
    if !keywords(&line).iter().any(|w| asked_keys.contains(w)) {
        whole = newest_answer(memory).or(whole);
    }
    if let Some(card) = whole.as_ref().filter(|w| !w.is_empty()) {
        if asks_only(understood) {
            if let Some(moved) = _the_thing_the_chat_moved_to(card, &line, memory, *MOVED_KEEP) {
                let answer = _reply_as_str(moved);
                let part = deps::opt_string("bot.memory.narrow:narrow", serde_json::json!([answer, line]), || crate::memory::narrow::narrow(&answer, &line, "")).filter(|p| !p.is_empty());
                if let Some(part) = part {
                    let bare = text::strip_chars(&part, " .").to_lowercase();
                    for sentence in SENTENCE.split(&answer) {
                        let sentence = text::strip(&sentence);
                        if !sentence.is_empty() && !bare.is_empty() && sentence.to_lowercase().contains(&bare) {
                            return Some(sentence.to_string());
                        }
                    }
                    return Some(part);
                }
                return None;
            }
        }
    }
    if drawn.module.as_deref() != Some("answers") && whole.is_none() {
        return None;
    }
    let mut answer = match &whole {
        Some(w) if !w.is_empty() => w.clone(),
        _ => drawn.bot.clone(),
    };
    if about_a_place_they_left_behind(&answer, memory) {
        return None;
    }
    let sized = _worked_out_for_this_line(understood, &line, &answer);
    if !(leans_on_topic(understood, memory) || asks_only(understood) || sized.is_some()) {
        return None;
    }
    if let Some(worked) = sized_reply(&answer, &line, memory, &[]).filter(|w| !w.is_empty()) {
        return Some(worked);
    }
    if let Some(earlier) = earlier.filter(|e| !e.is_empty()) {
        if earlier != answer {
            answer = earlier.clone();
            whole = Some(earlier);
            if let Some(part) = deps::opt_string("bot.memory.narrow:narrow", serde_json::json!([answer, line]), || crate::memory::narrow::narrow(&answer, &line, "")).filter(|p| !p.is_empty()) {
                return Some(part);
            }
        }
    }
    let sentences = _sentences(&answer);
    if proceed {
        if _opening_instruction(&sentences[0], true) {
            return None;
        }
        let doing = _doing_part(&sentences, memory);
        if let Some(doing) = doing.filter(|d| !d.is_empty()) {
            if !_says_it_again(&doing, &sentences[0]) {
                return Some(doing);
            }
        }
        return None;
    }
    if sentences.len() < 2 && whole.is_none() {
        return None;
    }
    let part = deps::opt_string("bot.memory.narrow:part_in_the_unit", serde_json::json!([answer, line]), || crate::memory::narrow::part_in_the_unit_text(&answer, &line).py_str());
    if let Some(part) = part.filter(|p| !p.is_empty()) {
        if words(&part).join(" ") != words(&last.bot).join(" ") {
            return said_again(&part, memory, &line).or(Some(part));
        }
    }
    if ASKS_A_FREQUENCY.is_match(&line.to_lowercase()) {
        if let Some(part) = deps::opt_string("bot.memory.narrow:narrow", serde_json::json!([answer, line]), || crate::memory::narrow::narrow(&answer, &line, "")).filter(|p| !p.is_empty()) {
            if words(&part) != words(&answer) {
                return Some(part);
            }
        }
    }
    if let Some(chosen) = _pick_sentence(&line, understood, &answer, memory, false).filter(|c| !c.is_empty()) {
        let opening = _sentences(&answer);
        if !opening.is_empty() && words(&opening[0]) == words(&chosen) {
            return said_again(&chosen, memory, &line).or(Some(chosen));
        }
        return Some(chosen);
    }
    _earlier_subject_sentence(&line, understood, memory, record.as_ref(), 2)
}

/// `pairing(text)`: the line leans on the pairing verb alone, with no noun of its own.
pub fn pairing(text_: &str) -> bool {
    PAIRING.is_match(text_) && !PAIRING_OWN.is_match(text_)
}

/// `_doing_word(text)`: the first word of a sentence that is not a when.
pub fn _doing_word(text_: &str) -> String {
    for (index, word) in text::split(text_).iter().enumerate() {
        let core = text::strip_chars(word, ",;:-\u{2014}\u{2013}").to_lowercase();
        if WHEN.contains(&core) && index < 5 {
            continue;
        }
        return core;
    }
    String::new()
}

/// `_opening_instruction(sentence, allow_pointer)`: the sentence opens with an instruction.
pub fn _opening_instruction(sentence: &str, allow_pointer: bool) -> bool {
    let first = _doing_word(sentence);
    if !IMPERATIVE.contains(&first) {
        return false;
    }
    allow_pointer || !SENDS_YOU_ON.contains(&first)
}

/// `_doing_part(sentences, memory)`: the sentence of the last answer that says what to do, as it
/// was written, or None.
pub fn _doing_part(sentences: &[String], memory: &Memory) -> Option<String> {
    for allow_pointer in [false, true] {
        for sentence in sentences {
            if RED_FLAG.match_(sentence).is_some() {
                continue;
            }
            let (_, sep, tail) = text::partition(sentence, ":");
            if !sep.is_empty() && !text::strip(tail).is_empty() && _opening_instruction(tail, allow_pointer) {
                let mut said = _upper_first(text::strip(tail));
                if !(said.ends_with('.') || said.ends_with('!') || said.ends_with('?')) {
                    said.push('.');
                }
                if RED_FLAG.match_(&said).is_some() || ASKS_THEM_BACK.is_match(&said) {
                    continue;
                }
                if !already_said(memory, Some(&said), 3) {
                    return Some(said);
                }
            }
            if _opening_instruction(sentence, allow_pointer) && !ASKS_THEM_BACK.is_match(sentence) {
                if !already_said(memory, Some(sentence), 3) {
                    return Some(sentence.clone());
                }
            }
        }
    }
    None
}

/// `_says_it_again(part, sentence)`: this part of the answer is already in that sentence, nearly
/// word for word.
pub fn _says_it_again(part: &str, sentence: &str) -> bool {
    let found = LETTERS_OR_APOSTROPHE.findall_str(&part.to_lowercase());
    let said = LETTERS_OR_APOSTROPHE.findall_str(&sentence.to_lowercase());
    if found.is_empty() || said.is_empty() {
        return false;
    }
    let head: Vec<String> = found.iter().take(6).cloned().collect();
    if said.len() >= head.len() {
        for index in 0..=(said.len() - head.len()) {
            if said[index..index + head.len()] == head[..] {
                return true;
            }
        }
    }
    let shared = found.iter().filter(|w| said.contains(*w)).count();
    shared as f64 >= 0.8 * found.len() as f64
}

/// `_the_rates_in(text)`: the periods a line or sentence puts a figure over (a set; here once
/// each, in the order found).
pub fn _the_rates_in(text_: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut add = |w: String| {
        if !out.contains(&w) {
            out.push(w);
        }
    };
    for found in _A_RATE.finditer(text_) {
        for word in found.groups().into_iter().flatten() {
            if !word.is_empty() {
                add(text::rstrip_chars(&word.to_lowercase(), "s").to_string());
            }
        }
    }
    for (word, one) in [("daily", "day"), ("weekly", "week"), ("monthly", "month"), ("yearly", "year"), ("annually", "year")] {
        if crate::understanding::rx_flags(&format!(r"\b{word}\b"), crate::py::re::I).is_match(text_) {
            add(one.to_string());
        }
    }
    out
}

/// `_a_rate_that_fits(line, sentence)`: False when the line asks for a daily amount and the
/// sentence gives a single item's.
pub fn _a_rate_that_fits(line: &str, sentence: &str) -> bool {
    let asked = _the_rates_in(line);
    if asked.is_empty() {
        return true;
    }
    let given = _the_rates_in(sentence);
    asked.iter().any(|a| given.contains(a)) || given.is_empty()
}

/// `_a_comma_that_ends_a_clause(sentence, start)`: the first comma at or after `start` (a byte
/// offset) that is not inside a figure, or None.
pub fn _a_comma_that_ends_a_clause(sentence: &str, start: usize) -> Option<usize> {
    let digit = |c: Option<char>| c.is_some_and(|c| crate::understanding::pyuni::is_digit(&c.to_string()));
    let mut from = start.min(sentence.len());
    while let Some(rel) = sentence[from..].find(',') {
        let at = from + rel;
        let before = sentence[..at].chars().last();
        let after = sentence[at + 1..].chars().next();
        if !(digit(before) && digit(after)) {
            return Some(at);
        }
        from = at + 1;
    }
    None
}

/// `_pick_amount(sentences, memory, line="")`: the sentence of an answer that gives a figure in a
/// unit, or the clause with it.
pub fn _pick_amount(sentences: &[String], memory: &Memory, line: &str) -> Option<String> {
    let screen = words(&memory.turns.last().map(|t_| t_.bot.clone()).unwrap_or_default()).join(" ");
    if A_CEILING_ASK.is_match(line) {
        for sentence in sentences {
            if AMOUNT_ANSWER.is_match(sentence) && A_CEILING_WORD.is_match(sentence) {
                return Some(sentence.clone());
            }
        }
        for sentence in sentences {
            if AMOUNT_ANSWER.is_match(sentence) {
                return Some(sentence.clone());
            }
        }
    }
    let named: Vec<String> = keywords(line).into_iter().filter(|w| text::len(w) > 2 && !ASK_ONLY.contains(w)).collect();
    let names_it = |sentence: &str| -> bool {
        let theirs = keywords(sentence);
        named.iter().any(|w| theirs.contains(w))
    };
    let one_of_a_rate = |sentence: &str| -> bool {
        _the_rates_in(line).is_empty() && !_the_rates_in(sentence).is_empty() && ONE_WORD.is_match(line)
    };
    for sentence in sentences {
        if AMOUNT_ANSWER.is_match(sentence) && !screen.contains(&words(sentence).join(" ")) && _a_rate_that_fits(line, sentence) {
            if !named.is_empty() && !names_it(sentence) {
                continue;
            }
            if one_of_a_rate(sentence) {
                continue;
            }
            return Some(sentence.clone());
        }
    }
    for sentence in sentences {
        let Some(found) = AMOUNT_ANSWER.search(sentence) else { continue };
        if !_a_rate_that_fits(line, sentence) {
            continue;
        }
        if !named.is_empty() && !names_it(sentence) {
            continue;
        }
        if one_of_a_rate(sentence) {
            continue;
        }
        let head = &sentence[..found.start()];
        let before = [head.rfind(':'), head.rfind(';')].into_iter().flatten().max();
        let after = _a_comma_that_ends_a_clause(sentence, found.end());
        let start = before.map(|b| b + 1).unwrap_or(0);
        let end = after.unwrap_or(sentence.len());
        let part = text::strip_chars(&sentence[start..end.max(start)], " .,;:");
        if !part.is_empty() && text::split(part).len() >= 2 {
            return Some(format!("{}.", _upper_first(part)));
        }
        return Some(sentence.clone());
    }
    None
}

/// `_pick_sentence(line, understood, answer, memory, strict=False)`: the sentence of this answer
/// that answers this line, said on its own, or None.
pub fn _pick_sentence(line: &str, understood: &Understood, answer: &str, memory: &Memory, strict: bool) -> Option<String> {
    said_on_its_own(_the_sentence_it_answers(line, understood, answer, memory, strict).as_deref())
}

/// `said_on_its_own(sentence)`: `sentence` without the word that joined it to the sentence
/// before, or None.
pub fn said_on_its_own(sentence: Option<&str>) -> Option<String> {
    let sentence = sentence?;
    let mut said = text::strip(sentence).to_string();
    if said.is_empty() {
        return None;
    }
    if let Some(joined) = JOINED_ON.match_(&said) {
        said = said[joined.end()..].to_string();
        if !said.is_empty() {
            said = _upper_first(&said);
        }
    }
    if said.is_empty() { None } else { Some(said) }
}

/// `_the_sentence_it_answers(line, understood, answer, memory, strict=False)`: the sentence of
/// this answer that answers this line, or None.
pub fn _the_sentence_it_answers(line: &str, understood: &Understood, answer: &str, memory: &Memory, strict: bool) -> Option<String> {
    let sentences = _sentences(answer);
    if sentences.is_empty() {
        return None;
    }
    let wanted: Vec<String> = keywords(line).iter().map(|w| stem(w)).collect();
    let asks: HashSet<String> = understood.asks.iter().cloned().collect();
    if asks.contains("how_long") {
        if let Some(part) = _pick_duration(&sentences, &wanted, memory, line).filter(|p| !p.is_empty()) {
            return Some(part);
        }
    }
    if HOW_MUCH.is_match(line) {
        if let Some(part) = _pick_amount(&sentences, memory, line).filter(|p| !p.is_empty()) {
            return Some(part);
        }
    }
    if ADVICE_ASK.is_match(line) && !ADVICE_NAMES_A_THING.is_match(line) && !_names_something_new(line, answer) {
        if let Some(doing) = _doing_part(&sentences, memory).filter(|d| !d.is_empty()) {
            return Some(doing);
        }
    }
    let mut best = 0;
    let mut best_score = 0;
    for (index, sentence) in sentences.iter().enumerate() {
        let said: Vec<String> = keywords(sentence).iter().map(|w| stem(w)).collect();
        let score = wanted.iter().filter(|w| said.contains(*w)).count();
        if score > best_score {
            best = index;
            best_score = score;
        }
    }
    if best_score > 0 {
        let sentence = &sentences[best];
        if !already_said(memory, Some(sentence), 3) {
            return Some(sentence.clone());
        }
    }
    if strict {
        return None;
    }
    if asks.contains("steps") || asks.contains("how") {
        let first = &sentences[0];
        if _names_something_new(line, answer) {
            return None;
        }
        if ADVICE_ASK.is_match(line) && !ADVICE_NAMES_A_THING.is_match(line) {
            if let Some(doing) = _doing_part(&sentences, memory).filter(|d| !d.is_empty()) {
                return Some(doing);
            }
        }
        return if already_said(memory, Some(first), 3) { None } else { Some(first.clone()) };
    }
    if asks.contains("how_long") {
        let amount = AMOUNT.search(line);
        let worked = _sized_reply(answer, line, memory);
        if amount.is_some() {
            if let Some(worked) = worked.filter(|w| !w.is_empty()) {
                return Some(worked);
            }
        }
        for sentence in &sentences[1..] {
            if !(TIME.is_match(sentence) && !already_said(memory, Some(sentence), 3)) {
                continue;
            }
            if let Some(amount) = &amount {
                if !SPACES.sub("", sentence).contains(&amount.as_str().replace(' ', "")) {
                    if RATE.is_match(answer) && !already_said(memory, Some(answer), 1) {
                        return Some(answer.to_string());
                    }
                    continue;
                }
            }
            return Some(sentence.clone());
        }
        if TIME.is_match(&sentences[0]) && !already_said(memory, Some(&sentences[0]), 3) {
            let span = _time_of(&sentences[0]);
            if !span.is_empty() {
                return Some(format!("{}.", _upper_first(&span)));
            }
        }
        if let Some(part) = _pick_duration(&sentences, &wanted, memory, line).filter(|p| !p.is_empty()) {
            return Some(part);
        }
    }
    if asks.contains("how_often") {
        for sentence in &sentences {
            let Some(found) = FREQUENCY.match_(sentence) else { continue };
            if already_said(memory, Some(sentence), 3) {
                continue;
            }
            return Some(_phrase(found.as_str()));
        }
    }
    None
}

/// `_time_of(sentence)`: the time in this sentence that answers "how long?", or "".
pub fn _time_of(sentence: &str) -> String {
    let clauses: Vec<String> = CLAUSE.split(sentence).into_iter().filter(|c| !_not_a_length(c)).collect();
    let mut first = String::new();
    for clause in &clauses {
        let spans = TIME_SPAN.finditer(clause);
        let Some(last) = spans.last() else { continue };
        let span = text::strip_chars(last.as_str(), " ,;").to_string();
        if TOTAL.is_match(clause) {
            return span;
        }
        if first.is_empty() && !AFTERWARDS.is_match(clause) {
            first = span;
        }
    }
    if !first.is_empty() {
        return first;
    }
    let mut last = String::new();
    for clause in &clauses {
        if let Some(m) = TIME_SPAN.finditer(clause).last() {
            last = text::strip_chars(m.as_str(), " ,;").to_string();
        }
    }
    last
}

/// `float(s)` for the strings the option readers build, or None (Python's ValueError).
fn _py_float(s: &str) -> Option<f64> {
    if s.is_empty() || s == "." || s.matches('.').count() > 1 {
        return None;
    }
    s.parse::<f64>().ok()
}

/// `_option_range(sentence)`: "6 to 10 minutes" when the sentence offers times for several
/// results, else "".
pub fn _option_range(sentence: &str) -> String {
    let parts = CLAUSE.split(sentence);
    let mut found: Vec<(String, Option<String>)> = Vec::new();
    for part in &parts {
        if let Some(m) = OPTION.match_(part) {
            found.push((m.str(1).to_string(), m.get(2).map(str::to_string)));
        }
    }
    if found.len() < 2 {
        return String::new();
    }
    let unit = found.iter().find_map(|(_, u)| u.clone().filter(|u| !u.is_empty())).unwrap_or_default();
    if unit.is_empty() {
        return String::new();
    }
    let first = text::split(&found[0].0)[0].clone();
    let last = text::split(&found[found.len() - 1].0).last().unwrap().clone();
    if first == last {
        return String::new();
    }
    let (Some(l), Some(f)) = (_py_float(&NOT_A_NUMBER_CHAR.sub("", &last)), _py_float(&NOT_A_NUMBER_CHAR.sub("", &first))) else {
        return String::new();
    };
    if l <= f {
        return String::new();
    }
    format!("{first} to {last} {}", unit.to_lowercase())
}

/// `_quoted(part)`: 1 when this clause is inside a quotation (an odd count of quote marks).
pub fn _quoted(part: &str) -> i64 {
    (QUOTE_MARK.findall(part).len() % 2) as i64
}

/// `_the_option_for_the_topic(sentence, line, memory)`: the option in this answer that names what
/// the conversation is on, or None.
pub fn _the_option_for_the_topic(sentence: &str, line: &str, memory: &Memory) -> Option<String> {
    let found = OPTION_ANY.finditer(sentence);
    if found.len() < 2 {
        return None;
    }
    let topic = carried_topic(memory, line).map(|r| rget(r, "text")).unwrap_or_default();
    let subject_: HashSet<String> = keywords(&topic).into_iter().collect();
    if subject_.is_empty() {
        return None;
    }
    let mut hits: Vec<(usize, String, String)> = Vec::new();
    let mut unit = String::new();
    for (index, found_one) in found.iter().enumerate() {
        let end = if index + 1 < found.len() { found[index + 1].start() } else { sentence.len() };
        let mut piece = sentence[found_one.start()..end].to_string();
        for _ in 0..2 {
            piece = TRAILING_PUNCT.sub("", &piece);
            piece = TRAILING_JOIN.sub("", &piece);
        }
        if let Some(u) = found_one.get(2).filter(|u| !u.is_empty()) {
            unit = u.to_string();
        }
        if keywords(&piece).iter().any(|w| subject_.contains(w)) {
            hits.push((index, piece, unit.clone()));
        }
    }
    if hits.len() != 1 {
        return None;
    }
    let (index, mut piece, unit) = hits.pop().unwrap();
    let found_one = &found[index];
    if !unit.is_empty() && found_one.get(2).is_none_or(|u| u.is_empty()) {
        let cut = found_one.end_of(1).unwrap() - found_one.start();
        piece = format!("{} {}{}", &piece[..cut], unit.to_lowercase(), &piece[cut..]);
    }
    Some(format!("{}.", _upper_first(&piece)))
}

/// `_only_a_marker(part)`: a clause that is nothing but where a step sits in time.
pub fn _only_a_marker(part: &str) -> bool {
    let found = LOWER_WORD.findall_str(&part.to_lowercase());
    if found.is_empty() {
        return true;
    }
    found.iter().all(|w| TIME_ONLY.contains(w) || DURATION.match_(w).is_some())
}

/// `_duration_clauses(sentence)`: every clause of this sentence that carries a time, as sentences
/// of their own.
pub fn _duration_clauses(sentence: &str) -> Vec<String> {
    let mut parts = Vec::new();
    for part in _clauses_of(sentence) {
        let mut part = text::strip_chars(&part, " .").to_string();
        if _quoted(&part) != 0 {
            continue;
        }
        if part.to_lowercase().starts_with("then ") {
            part = text::slice(&part, 5, None);
        }
        if part.to_lowercase().starts_with("and ") {
            part = text::slice(&part, 4, None);
        }
        if WARNING_SPAN.is_match(&part) {
            continue;
        }
        if part.is_empty() || _only_a_marker(&part) {
            continue;
        }
        if _not_a_length(&part) {
            continue;
        }
        if DURATION.is_match(&part) || SPAN_ADJECTIVE.is_match(&part) {
            parts.push(format!("{}.", _upper_first(&part)));
        }
    }
    parts
}

/// `_only_a_cadence(part)`: the only time in this clause is how often to do it.
pub fn _only_a_cadence(part: &str) -> bool {
    FREQUENCY.is_match(part) && !DURATION.is_match(&FREQUENCY.sub(" ", part))
}

/// `_a_window_in_the_weather(part)`: every time in this clause is how long the weather has to
/// hold.
pub fn _a_window_in_the_weather(part: &str) -> bool {
    let mut starts: Vec<usize> = DURATION.finditer(part).iter().map(|m| m.start()).collect();
    starts.extend(SPAN_ADJECTIVE.finditer(part).iter().map(|m| m.start()));
    if starts.is_empty() {
        return false;
    }
    starts.sort();
    starts.iter().all(|&s| A_WEATHER_WINDOW.is_match(&part[..s]))
}

/// `_not_a_length(part)`: the time in this clause is not how long the job takes.
pub fn _not_a_length(part: &str) -> bool {
    _only_a_cadence(part) || _a_window_in_the_weather(part)
}

/// `_duration_clause(sentence)`: the first clause of this sentence that carries the time in it.
pub fn _duration_clause(sentence: &str) -> Option<String> {
    _duration_clauses(sentence).into_iter().next()
}

/// `_shorten(part)`: a long clause with a time in it comes back as the time itself.
pub fn _shorten(part: &str) -> String {
    if text::split(part).len() > 9 {
        if let Some(found) = DURATION.search(part) {
            if text::strip_chars(&part[found.end()..], " .,;:!?").is_empty() {
                let span = text::strip_chars(found.as_str(), " .");
                return format!("{}.", _upper_first(span));
            }
        }
    }
    part.to_string()
}

/// `_asked_period(line)`: the unit a line counts in ("how long each day" -> 2), or None.
pub fn _asked_period(line: &str) -> Option<i64> {
    let found = PERIOD_ASK.search(line)?;
    let unit = text::rstrip_chars(&found.str(1).to_lowercase(), "s").to_string();
    PERIODS.iter().find(|(k, _)| *k == unit).map(|(_, v)| *v)
}

/// `_in_period(part, period)`: the time in this part is counted in the period the line asked
/// for.
pub fn _in_period(part: &str, period: Option<i64>) -> bool {
    let Some(period) = period else { return true };
    if !FIGURED.is_match(part) {
        return false;
    }
    let Some(found) = DURATION.search(part) else { return false };
    let counted = found.as_str().to_lowercase();
    for (name, rank) in PERIODS.iter() {
        if counted.contains(name.as_str()) {
            return *rank <= period;
        }
    }
    false
}

/// `_pick_duration(sentences, wanted, memory, line="")`: the time in the answer this line is
/// asking about.
pub fn _pick_duration(sentences: &[String], wanted: &[String], memory: &Memory, line: &str) -> Option<String> {
    let mut sentences: Vec<String> = sentences.to_vec();
    if LENGTH_ASK.is_match(line) {
        sentences.retain(|s| DURATION.is_match(s) || SPAN_ADJECTIVE.is_match(s));
        if sentences.is_empty() {
            return None;
        }
        let spans: Vec<String> = sentences.iter().filter(|s| !THRESHOLD_SPAN.is_match(s)).cloned().collect();
        if !spans.is_empty() {
            sentences = spans;
        }
    }
    let wanted_set: HashSet<String> = wanted.iter().cloned().collect();
    let mut best_overlap = 0;
    let mut best_index = 0;
    for (index, sentence) in sentences.iter().enumerate() {
        let theirs: HashSet<String> = keywords(sentence).iter().map(|w| stem(w)).collect();
        let overlap = wanted_set.iter().filter(|w| theirs.contains(*w)).count();
        if overlap > best_overlap {
            best_overlap = overlap;
            best_index = index;
        }
    }
    if best_overlap > 0 {
        let sentence = &sentences[best_index];
        let period = _asked_period(line);
        if period.is_some() {
            let parts: Vec<String> = _duration_clauses(sentence).into_iter().filter(|p| _in_period(p, period)).collect();
            let part = _step_they_asked_for(&parts, wanted).unwrap_or_default();
            let mut part = part;
            if !part.is_empty() {
                part = PERIOD_LEAD.sub("", &part);
                let first_upper = part.chars().next().is_some_and(|c| text::is_upper(&c.to_string()));
                if !first_upper {
                    part = _upper_first(&part);
                }
            }
            return if part.is_empty() { None } else { Some(_shorten(&part)) };
        }
        if !already_said(memory, Some(sentence), 3) {
            return Some(sentence.clone());
        }
        let parts: Vec<String> = _duration_clauses(sentence).into_iter().filter(|p| !already_said(memory, Some(p), 3)).collect();
        return _step_they_asked_for(&parts, wanted).filter(|p| !p.is_empty()).map(|p| _shorten(&p));
    }
    let choices = sentences.first().map(|s| _option_range(s)).unwrap_or_default();
    if !choices.is_empty() {
        return Some(choices);
    }
    if let Some(part) = sentences.first().and_then(|s| _the_option_for_the_topic(s, line, memory)).filter(|p| !p.is_empty()) {
        return Some(part);
    }
    let period = _asked_period(line);
    let parts: Vec<String> = sentences
        .iter()
        .flat_map(|s| _duration_clauses(s))
        .filter(|p| !already_said(memory, Some(p), 3) && _in_period(p, period))
        .collect();
    _step_they_asked_for(&parts, wanted).filter(|p| !p.is_empty()).map(|p| _shorten(&p))
}

/// `_step_they_asked_for(parts, wanted)`: the last time in the answer that is not a step
/// afterwards, unless they asked for that.
pub fn _step_they_asked_for(parts: &[String], wanted: &[String]) -> Option<String> {
    let kept: Vec<&String> = parts
        .iter()
        .filter(|p| !AFTERWARDS.is_match(p) || keywords(p).iter().any(|w| wanted.contains(w)))
        .collect();
    if let Some(last) = kept.last() {
        return Some((*last).clone());
    }
    parts.last().cloned()
}

/// `_time_in_words(minutes)`: 80 -> "1 hour 20", 50 -> "50 minutes", rounded to five.
pub fn _time_in_words(minutes: f64) -> String {
    let minutes = text::round_half_even(minutes / 5.0) * 5;
    let (hours, rest) = (minutes.div_euclid(60), minutes.rem_euclid(60));
    if hours == 0 {
        return format!("{rest} minutes");
    }
    let hours_text = format!("{hours} hour{}", if hours == 1 { "" } else { "s" });
    if rest != 0 { format!("{hours_text} {rest}") } else { hours_text }
}

/// `time_for_amount(answer, line)`: the time an answer's own rate gives for the amount a
/// follow-up names, or None.
pub fn time_for_amount(answer: &str, line: &str) -> Option<String> {
    let amount = AMOUNT.search(line)?;
    let rate = RATE_BY_WEIGHT.search(answer)?;
    let (each, per, unit) = (rate.str(1), rate.str(2), rate.str(3));
    let said_unit = NUMBER_OR_SPACE.sub("", amount.as_str()).to_lowercase();
    let said_grams = _grams_of(&said_unit)?;
    let unit_grams = _grams_of(&unit.to_lowercase())?;
    let want = _py_float(&NOT_A_DIGIT_OR_DOT.sub("", amount.as_str()))? * said_grams;
    let base = per.parse::<f64>().ok()? * unit_grams;
    if base <= 0.0 || want <= 0.0 {
        return None;
    }
    let mut total = each.parse::<f64>().ok()? * want / base;
    if let Some(extra) = PLUS_MINUTES.search(answer) {
        total += extra.str(1).parse::<f64>().unwrap_or(0.0);
    }
    if !(5.0..=(12.0 * 60.0)).contains(&total) {
        return None;
    }
    Some(_time_in_words(total))
}

/// `_already_for_this_size(line, answer)`: the answer has already worked its rate out for the
/// size this line names.
pub fn _already_for_this_size(line: &str, answer: &str) -> bool {
    let (Some(asked), Some(told)) = (AMOUNT.search(line), OWN_SIZE.search(answer)) else { return false };
    let grams = _in_grams(asked.as_str());
    let said = _in_grams(&format!("{}{}", told.str(1), told.str(2)));
    grams.is_some() && grams == said
}

/// `_in_grams(text)`: an amount in grams (rounded to one place), or None when it is not a weight.
pub fn _in_grams(text_: &str) -> Option<f64> {
    let found = IN_GRAMS.match_(text_)?;
    let grams = found.str(1).parse::<f64>().ok()? * _grams_of(&found.str(2).to_lowercase())?;
    // Python's round(x, 1): the correctly rounded decimal, as Rust's formatting gives it
    format!("{grams:.1}").parse::<f64>().ok()
}

/// `_names_another_thing(line, answer, about=())`: the line names a word the answer is not about.
pub fn _names_another_thing(line: &str, answer: &str, about: &[String]) -> bool {
    let mut said: HashSet<String> = keywords(answer).iter().map(|w| stem(w)).collect();
    said.extend(about.iter().map(|w| stem(w)));
    for word in keywords(line) {
        if AMOUNT.is_match(&word) || ASK_ONLY.contains(&word) {
            continue;
        }
        if !said.contains(&stem(&word)) {
            return true;
        }
    }
    false
}

/// `sized_reply(answer, line, memory, about=())`: the time this answer's own rate gives for the
/// size this line names, or None.
pub fn sized_reply(answer: &str, line: &str, memory: &Memory, about: &[String]) -> Option<String> {
    if _names_another_thing(line, answer, about) || _already_for_this_size(line, answer) {
        return None;
    }
    _sized_reply(answer, line, memory)
}

/// `_sized_reply(answer, line, memory)`: the time the answer's own rate gives for the amount in
/// the line, said with the rule, or None.
pub fn _sized_reply(answer: &str, line: &str, memory: &Memory) -> Option<String> {
    let amount = AMOUNT.search(line)?;
    let sized = time_for_amount(answer, line).filter(|s| !s.is_empty())?;
    let rule = _rate_said(answer);
    if rule.is_empty() {
        return None;
    }
    let said = amount.as_str();
    let times = _times_asked(line, memory);
    if times >= 2 {
        return Some(format!("{} for {said}, the same rule: {rule}.", text::capitalize(&sized)));
    }
    if times == 1 {
        return Some(format!("Still {sized} for {said}: {rule}."));
    }
    Some(format!("About {sized} for {said}: {rule}."))
}

/// `_times_asked(line, memory)`: how many turns in a row just before this one asked this same
/// line, word for word.
pub fn _times_asked(line: &str, memory: &Memory) -> i64 {
    let wanted = low_squash(line);
    let mut count = 0;
    for turn in memory.turns.iter().rev() {
        if low_squash(&turn.user) != wanted {
            break;
        }
        count += 1;
    }
    count
}

/// `_rate_said(answer)`: the sentence of an answer that states a cooking rate, without the size
/// it worked out.
pub fn _rate_said(answer: &str) -> String {
    for sentence in SENTENCE.split(answer) {
        if !RATE_BY_WEIGHT.is_match(&sentence) {
            continue;
        }
        let first = match SO_SPLIT.search(&sentence) {
            Some(m) => sentence[..m.start()].to_string(),
            None => sentence.clone(),
        };
        let head = text::strip_chars(&first, " .,;").to_string();
        let head = ABOUT_LEAD.sub("", &head);
        let head = text::strip_chars(&head, " .,;");
        let starts_upper = head.chars().next().is_some_and(|c| text::is_upper(&c.to_string()));
        if starts_upper {
            let mut c = head.chars();
            let first = c.next().unwrap();
            return format!("{}{}", first.to_lowercase(), c.as_str());
        }
        return head.to_string();
    }
    String::new()
}

/// `_phrase(text)`: a phrase of an answer said as a sentence of its own.
pub fn _phrase(text_: &str) -> String {
    let said = text::strip_chars(text_, " ,;:");
    if said.is_empty() { String::new() } else { format!("{}.", _upper_first(said)) }
}

/// `has_time(text)`: a line or an answer carries a length of time in it.
pub fn has_time(text_: &str) -> bool {
    TIME.is_match(text_)
}

/// `_worked_out_for_this_line(understood, line, answer)`: the time an answer's own rate gives for
/// the amount this line names, or None.
pub fn _worked_out_for_this_line(understood: &Understood, line: &str, answer: &str) -> Option<String> {
    if !understood.asks.iter().any(|a| a == "how_long") {
        return None;
    }
    time_for_amount(answer, line)
}

/// `asks_only(understood)`: the line asks for a property and names no thing of its own.
pub fn asks_only(understood: &Understood) -> bool {
    if understood.asks.is_empty() {
        return false;
    }
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    if _ASKS_WHAT_I_AM.match_(&line.to_lowercase()).is_some() {
        return false;
    }
    keywords(line).iter().all(|w| ASK_ONLY.contains(w))
}

/// `_earlier_subject_sentence(line, understood, memory, skip, turns_back=2)`: a sentence from the
/// answer to an earlier subject this line still has a word in common with.
pub fn _earlier_subject_sentence(line: &str, understood: &Understood, memory: &Memory, skip: Option<&Value>, turns_back: i64) -> Option<String> {
    let Some(Value::List(records)) = memory.get("topics") else { return None };
    let mine: HashSet<String> = keywords(line).iter().map(|w| stem(w)).collect();
    let mut seen = 0;
    for record in records.iter().rev() {
        if !is_dict(record) || skip.is_some_and(|s| same_object(record, s)) {
            continue;
        }
        let asked = rget(record, "text");
        let bot = record.get("bot").map(value_str);
        if asked.is_empty() || already_said(memory, bot.as_deref(), 3) {
            continue;
        }
        if !keywords(&asked).iter().any(|w| mine.contains(&stem(w))) {
            continue;
        }
        seen += 1;
        if seen > turns_back {
            return None;
        }
        let answer = entry_answer(memory, &asked).filter(|a| !a.is_empty()).unwrap_or_else(|| rget(record, "bot"));
        if let Some(chosen) = _pick_sentence(line, understood, &answer, memory, true).filter(|c| !c.is_empty()) {
            return Some(chosen);
        }
    }
    None
}

/// `_repeats_the_question(line, asked)`: this line asks what the conversation is already about,
/// word for word.
pub fn _repeats_the_question(line: &str, asked: &str) -> bool {
    let wanted: Vec<String> = keywords(line).iter().map(|w| stem(w)).collect();
    if wanted.is_empty() {
        return true;
    }
    let said: Vec<String> = keywords(asked).iter().map(|w| stem(w)).collect();
    wanted.iter().filter(|w| said.contains(*w)).count() as f64 >= 0.6 * wanted.len() as f64
}

/// `already_said(memory, text, turns_back=3)`: this exact wording was one of the bot's last few
/// replies.
pub fn already_said(memory: &Memory, text_: Option<&str>, turns_back: i64) -> bool {
    let said = low_squash(text_.unwrap_or(""));
    if said.is_empty() {
        return true;
    }
    _last(&memory.turns, turns_back).iter().any(|turn| low_squash(&turn.bot) == said)
}

/// `_names_something_new(line, answer)`: the line names a thing the answer never mentions.
pub fn _names_something_new(line: &str, answer: &str) -> bool {
    let said: HashSet<String> = keywords(answer).iter().map(|w| stem(w)).collect();
    keywords(line).iter().any(|w| !STEP_ASK_WORDS.contains(w) && !said.contains(&stem(w)))
}

/// `_named_a_fresh_thing(text, record, answer="")`: the line names a thing the topic does not
/// have.
pub fn _named_a_fresh_thing(text_: &str, record: Option<&Value>, answer: &str) -> bool {
    if REQUEST_A_THING.is_match(text_) {
        return false;
    }
    let mut known: HashSet<String> = record.map(rwords).unwrap_or_default().iter().map(|w| stem(w)).collect();
    known.extend(words(answer).iter().map(|w| stem(w)));
    let said = words(text_);
    for at in 0..said.len().saturating_sub(1) {
        if A_THING.contains(&said[at].to_lowercase()) && !known.contains(&stem(&said[at + 1])) {
            return true;
        }
    }
    false
}

/// `_the_thing_the_chat_moved_to(whole, line, memory, keep=MOVED_KEEP)`: the entry of the thing
/// named after the card was given, when the chat moved to it.
pub fn _the_thing_the_chat_moved_to(whole: &str, line: &str, memory: &Memory, keep: i64) -> Option<&'static Entry> {
    let turns = _last(&memory.turns, keep);
    let wanted: Vec<String> = words(whole).into_iter().take(8).collect();
    let mut at = None;
    for (index, turn) in turns.iter().enumerate() {
        if words(&turn.bot).into_iter().take(8).collect::<Vec<_>>() == wanted {
            at = Some(index);
        }
    }
    let at = at?;
    let was = _entry(&turns[at].user)?;
    let kind = deps::opt_string("bot.memory.narrow:kind_of", deps::args1(line), || crate::memory::narrow::kind_of(line));
    let figure = kind.and_then(|k| t::value("bot.memory.narrow.", "SPANS").get(&k).and_then(Value::as_re))?;
    let line_words = words(line);
    for turn in turns[at + 1..].iter().rev() {
        let said = &turn.user;
        if said.is_empty() || words(said) == line_words {
            continue;
        }
        if turn.module.is_none() || turn.module.as_deref() == Some("answers") {
            continue;
        }
        let Some(named) = _entry(said) else { continue };
        if std::ptr::eq(named, was) {
            continue;
        }
        if figure.is_match(&_reply_as_str(named)) {
            return Some(named);
        }
    }
    None
}

/// `left_for_a_blank(memory, line)`: the chat moved to a thing named in between and got no answer
/// for it.
pub fn left_for_a_blank(memory: &Memory, line: &str) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    if words(&last.user).join(" ") == words(line).join(" ") {
        return false;
    }
    if last.module.as_deref() == Some("answers") {
        return false;
    }
    let asked = &last.user;
    if !_a_question(asked) {
        return false;
    }
    if !POINTS_BACK.is_match(line) {
        return false;
    }
    let record = carried_topic(memory, line);
    let answer = entry_answer(memory, &record.map(|r| rget(r, "text")).unwrap_or_default())
        .filter(|a| !a.is_empty())
        .unwrap_or_else(|| last.bot.clone());
    _named_a_fresh_thing(asked, record, &answer)
}

/// `_a_question(text)`: the line asks something.
pub fn _a_question(text_: &str) -> bool {
    A_QUESTION.is_match(text_)
}

/// `bare_question(understood, memory)`: a short question with no subject of its own, after a
/// topic it can lean on.
pub fn bare_question(understood: &Understood, memory: &Memory) -> bool {
    if understood.kind != "question" {
        return false;
    }
    let line = low_squash(if understood.clean.is_empty() { &understood.text } else { &understood.clean });
    if line.is_empty() || text::split(&line).len() as i64 > *BARE_WORDS || is_ack(&line) {
        return false;
    }
    if !BARE_ASK.is_match(&line) {
        return false;
    }
    if carried_topic(memory, &line).is_none() {
        return false;
    }
    if follows_on(understood, &line) {
        return false;
    }
    if ASKS_ABOUT_THEMSELVES.is_match(&line) {
        return false;
    }
    _entry(&line).is_none()
}

/// `_still_the_topic(record, memory)`: no turn since this topic's has put the chat on the bot
/// instead.
pub fn _still_the_topic(record: &Value, memory: &Memory) -> bool {
    if !is_dict(record) {
        return false;
    }
    let Some(Value::Int(index)) = record.get("turn") else { return true };
    let from = (index + 1).max(0) as usize;
    for turn in memory.turns.iter().skip(from) {
        let said = low_squash(&turn.user);
        if said.is_empty() || is_ack(&said) || GREET.contains(&said) {
            continue;
        }
        if crate::understanding::understand(&said, None).intent == "smalltalk" {
            return false;
        }
    }
    true
}

/// `leans_on_topic(understood, memory)`: this line carries on the current topic instead of
/// naming its own subject.
pub fn leans_on_topic(understood: &Understood, memory: &Memory) -> bool {
    if _after_an_emergency(memory) {
        return false;
    }
    if _after_a_correction(memory) {
        return false;
    }
    let said = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let text_ = deps::string("bot.memory.filler:without_the_shrug", deps::args1(said), || crate::memory::filler::without_the_shrug(said));
    if PROCEED.iter().any(|p| text_.to_lowercase().contains(p.as_str())) {
        return false;
    }
    let Some(record) = carried_topic(memory, &text_) else { return false };
    if text::strip(&text_).is_empty() || is_ack(&text_) {
        return false;
    }
    if opens_by_switching(said) {
        return false;
    }
    if DETAIL.is_match(&text_) {
        let known: HashSet<String> = rwords(record).into_iter().collect();
        let fresh_ = keywords(&text_).into_iter().filter(|w| !known.contains(&stem(w))).count();
        return fresh_ <= 4 && words(&text_).len() <= 14;
    }
    if follows_on(understood, &text_) {
        return true;
    }
    let carried = carry_the_ask(&text_, &rget(record, "text"), understood, memory);
    if carried.is_none() {
        return bare_question(understood, memory);
    }
    names_nothing(&text_) || _only_the_topics_own_words(&text_, record)
}

/// `_only_the_topics_own_words(text, record)`: every content word of the line is a word the topic
/// already carries.
pub fn _only_the_topics_own_words(text_: &str, record: &Value) -> bool {
    let known = rwords(record);
    if known.is_empty() {
        return false;
    }
    _content_words(text_).iter().all(|w| known.iter().any(|said| alike(w, said)))
}



static NEXT_SLOT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NEXT_SLOT"));
static ASK_SLOT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASK_SLOT"));
static ASK_WORDS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASK_WORDS"));
static TWO_PETS_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TWO_PETS_ASK"));
static PET_ARRIVING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PET_ARRIVING"));
static PET_EXISTING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PET_EXISTING"));
static WEATHER_NOISE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WEATHER_NOISE"));
static NOW_AND_HERE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOW_AND_HERE"));
static WEATHER_ASKS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WEATHER_ASKS"));
static PERIOD_ONLY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PERIOD_ONLY"));
static MEAL_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MEAL_NAMES"));
static MEAL_ONLY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MEAL_ONLY"));
static OPENS_A_FOLLOW_UP: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "OPENS_A_FOLLOW_UP"));
static A_TRIP_TAKEN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_TRIP_TAKEN"));
static IT_IS_PART_OF_THE_PHRASE: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "IT_IS_PART_OF_THE_PHRASE"));
static POINTING_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "POINTING_WORDS"));
static WITHOUT_A_DETERMINER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WITHOUT_A_DETERMINER"));
static WHAT_IT_IS_LIKE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WHAT_IT_IS_LIKE"));
static WHAT_STANDS_IN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_STANDS_IN"));
static STANDS_IN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STANDS_IN"));
static MONEY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MONEY"));
pub static YES_NO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "YES_NO"));
static PROGNOSIS: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "PROGNOSIS"));
static HOW_LONG_KEY: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "HOW_LONG_KEY"));
static LIKE_LOOKUPS: LazyLock<i64> = LazyLock::new(|| t::int(M, "LIKE_LOOKUPS"));
static LIMIT_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LIMIT_ASK"));
static THE_LIMIT_ITSELF: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THE_LIMIT_ITSELF"));
static THE_LIMIT: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "THE_LIMIT"));
static HE_OR_SHE_OPENS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HE_OR_SHE_OPENS"));
static ASKS_HOW_EARLY_THERE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKS_HOW_EARLY_THERE"));
static A_FLIGHT_IN_THE_TOPIC: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_FLIGHT_IN_THE_TOPIC"));
static THE_AIRPORT: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "THE_AIRPORT"));
static CHILD_CARE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CHILD_CARE"));
static KEEP_IT_UP: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "KEEP_IT_UP"));
static STAYING_WITH_IT: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "STAYING_WITH_IT"));
static AN_ACTIVITY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "AN_ACTIVITY"));
static LIMIT_LINE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LIMIT_LINE"));
static GATHERING_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GATHERING_WORDS"));
static TOO_SOON_TO_COOK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TOO_SOON_TO_COOK"));
static FOR_SOMEBODY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FOR_SOMEBODY"));
static PUTTING_ON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PUTTING_ON"));
static ASK_SHAPE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_SHAPE_WORDS"));
static WORDS_THIS_TURN: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "WORDS_THIS_TURN"));
static TURN_TOPIC: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "TURN_TOPIC"));
static VERB_FORMS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.kinds.", "VERB_FORMS"));
static VERBISH: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.kinds.", "VERBISH"));

static THERE_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bthere\b", crate::py::re::I));
/// `sorted(MONTH_NAMES, key=len, reverse=True)` (ties alphabetically: Python's are in set order).
static MONTHS_LONGEST_FIRST: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut v: Vec<String> = MONTH_NAMES.iter().cloned().collect();
    v.sort();
    v.sort_by_key(|s| std::cmp::Reverse(text::len(s)));
    v
});
/// `sorted(MONTH_NAMES | SEASONS, key=len, reverse=True)` (ties alphabetically).
static PERIODS_LONGEST_FIRST: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut v: Vec<String> = MONTH_NAMES.iter().chain(SEASONS.iter()).cloned().collect();
    v.sort();
    v.dedup();
    v.sort_by_key(|s| std::cmp::Reverse(text::len(s)));
    v
});
/// `sorted(MEAL_NAMES, key=len, reverse=True)` (ties alphabetically).
static MEALS_LONGEST_FIRST: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut v: Vec<String> = MEAL_NAMES.iter().cloned().collect();
    v.sort();
    v.sort_by_key(|s| std::cmp::Reverse(text::len(s)));
    v
});

/// The line a reading is read from (`understood.get("clean") or understood.get("text") or ""`).
fn _line_of(understood: &Understood) -> String {
    if understood.clean.is_empty() { understood.text.clone() } else { understood.clean.clone() }
}

/// `dict(understood)` with `clean` (and `words`) put in, and `norm_text` when Python sets it.
fn _reading_with(understood: &Understood, clean: &str, norm_text: bool) -> Understood {
    let mut reading = understood.clone();
    reading.clean = clean.to_string();
    reading.words = words(clean);
    if norm_text {
        reading.norm_text = clean.to_string();
    }
    reading
}

/// `ref.get(word)` for the pointing words the reading resolves.
fn _ref_slot<'a>(understood: &'a Understood, word: &str) -> Option<&'a str> {
    let r = &understood.reference;
    match word {
        "it" => r.it.as_deref(),
        "that" => r.that.as_deref(),
        "they" => r.they.as_deref(),
        "there" => r.there.as_deref(),
        "he" => r.he.as_deref(),
        "she" => r.she.as_deref(),
        _ => None,
    }
}

/// `memory.get("person")` is a non-empty dict.
fn _about_somebody(memory: &Memory) -> bool {
    matches!(memory.get("person"), Some(p @ Value::Dict(_)) if p.truthy())
}

/// The id of `memory["last_answer"]` when it is a dict, else None.
fn _last_answer_id(memory: &Memory) -> Option<Value> {
    match memory.get("last_answer") {
        Some(last @ Value::Dict(_)) => last.get("id").cloned(),
        _ => None,
    }
}

/// `next_slot_reading(understood, memory)`: "and for lunch" read as the last question with lunch
/// in it, or None.
pub fn next_slot_reading(understood: &Understood, memory: &Memory) -> Option<Understood> {
    let line = low_squash(&_line_of(understood));
    let slot = NEXT_SLOT.match_(&line)?;
    if understood.kind == "question" {
        return None;
    }
    let previous = memory.turns.last().map(|t_| t_.user.clone()).unwrap_or_default();
    let asked = low_squash(&previous);
    if asked.is_empty() || !ASK_WORDS.is_match(&asked) {
        return None;
    }
    let tail = ASK_SLOT.search(&asked)?;
    let thing = text::strip(slot.str(1)).to_string();
    if thing.is_empty() || thing == text::strip(tail.str(1)) {
        return None;
    }
    let mut reading = understood.clone();
    reading.clean = text::strip(&format!("{}for {thing}", &asked[..tail.start()])).to_string();
    reading.text = reading.clean.clone();
    reading.words = words(&reading.clean);
    reading.kind = "question".to_string();
    reading.intent = "question".to_string();
    Some(reading)
}

/// `_introduction_covered(new, old)`: an animal introduction card answers this pair.
pub fn _introduction_covered(new: &str, old: &str) -> bool {
    let cats = ["cat", "kitten"];
    let dogs = ["dog", "puppy"];
    (cats.contains(&new) && old == "cat") || (cats.contains(&new) && dogs.contains(&old)) || (dogs.contains(&new) && cats.contains(&old))
}

/// `PET_ONE.get(word, word)`.
fn _pet_one(word: &str) -> String {
    PET_ONE.get(word).cloned().unwrap_or_else(|| word.to_string())
}

/// `introduction_question(text, memory)`: the full introduction question for "will they get
/// on?" after a second pet, or "".
pub fn introduction_question(text_: &str, memory: &Memory) -> String {
    if !TWO_PETS_ASK.is_match(text_) {
        return String::new();
    }
    let Some(known) = memory.get("user").filter(|u| is_dict(u)) else { return String::new() };
    let mut animals: Vec<String> = Vec::new();
    for pet in known.get("pets").map(Value::items).unwrap_or(&[]) {
        let animal = if is_dict(pet) { pet.get("animal").map(value_str).unwrap_or_default() } else { String::new() };
        let animal = _pet_one(&text::strip(&animal).to_lowercase());
        if !animal.is_empty() && !animals.contains(&animal) {
            animals.push(animal);
        }
    }
    if animals.len() != 2 {
        return String::new();
    }
    let plan = known.get("plan");
    let named = match plan {
        Some(Value::Str(p)) => PET_WORD.search(p).map(|m| m.str(1).to_lowercase()),
        _ => None,
    };
    let mut new = String::new();
    if let Some(named) = named {
        new = _pet_one(&named);
    } else {
        let mut arriving: Vec<String> = Vec::new();
        for fact in memory.get("facts_log").map(Value::items).unwrap_or(&[]) {
            if !is_dict(fact) || fact.get("key").and_then(Value::as_str) != Some("pet") {
                continue;
            }
            let said = rget(fact, "text");
            let Some(found) = PET_WORD.search(&said) else { continue };
            if !PET_ARRIVING.is_match(&said) || PET_EXISTING.is_match(&said) {
                continue;
            }
            let animal = _pet_one(&found.str(1).to_lowercase());
            if !arriving.contains(&animal) {
                arriving.push(animal);
            }
        }
        if arriving.len() == 1 {
            new = arriving.remove(0);
        }
    }
    if new.is_empty() {
        return String::new();
    }
    let others: Vec<&String> = animals.iter().filter(|a| **a != new).collect();
    if others.len() != 1 {
        return String::new();
    }
    let old = others[0];
    if !_introduction_covered(&new, old) {
        return String::new();
    }
    format!("how do i introduce a new {new} to my older {old}")
}

/// `the_weather_ask` read on a line.
fn _the_weather_ask_in(line: &str) -> bool {
    let said = words(line);
    if !said.iter().any(|w| WEATHER_WORDS.contains(w)) {
        return false;
    }
    let low = line.to_lowercase();
    if !ASK_CUE.is_match(&low) && !WEATHER_ASKS.is_match(&low) {
        return false;
    }
    said.iter().all(|w| WEATHER_WORDS.contains(w) || WEATHER_NOISE.contains(w))
}

/// `the_weather_ask(understood)`: a line that only asks about the weather.
pub fn the_weather_ask(understood: &Understood) -> bool {
    _the_weather_ask_in(&_line_of(understood))
}

/// `the_place_and_month(text)`: the place and the month a line names, as the knowledge base
/// spells them, or (None, None).
pub fn the_place_and_month(text_: &str) -> (Option<String>, Option<String>) {
    let low = low_squash(text_);
    if low.is_empty() {
        return (None, None);
    }
    for name in crate::knowledge::travel::CITY_CLIMATE.longest_first() {
        let pattern = format!(r"(?<![a-z]){}(?![a-z])", crate::py::re::escape(name));
        if !crate::understanding::rx(&pattern).is_match(&low) {
            continue;
        }
        let month = MONTHS_LONGEST_FIRST.iter().find(|word| crate::understanding::rx(&format!(r"(?<![a-z]){word}")).is_match(&low)).cloned();
        return (Some(name.clone()), month);
    }
    (None, None)
}

/// `_the_period_in(text)`: the period a line names, as it is spelled there, or "".
pub fn _the_period_in(text_: &str) -> String {
    let low = low_squash(text_);
    for name in PERIODS_LONGEST_FIRST.iter() {
        for found in crate::understanding::rx(&format!(r"(?<![a-z]){name}(?![a-z])")).finditer(&low) {
            let before = text::split(&low[..found.start()]);
            let after = text::strip_chars(&low[found.end()..], " ?!.,");
            let lead = ["in", "next", "this", "the", "early", "late", "mid", "until", "til", "by", "for", "about", "of", "and"];
            if before.is_empty() || lead.contains(&before[before.len() - 1].as_str()) {
                return found.as_str().to_string();
            }
            if !["may", "march", "fall"].contains(&name.as_str()) || after.is_empty() {
                return found.as_str().to_string();
            }
        }
    }
    String::new()
}

/// `_a_period_and_nothing_else(line)`: a line that names a period and nothing else of its own.
pub fn _a_period_and_nothing_else(line: &str) -> bool {
    if _the_period_in(line).is_empty() {
        return false;
    }
    words(line).iter().all(|w| PERIOD_ONLY.contains(w) || MONTH_NAMES.contains(w) || SEASONS.contains(w))
}

/// `the_period_they_ask_about(understood, memory)`: the question on the table asked again about
/// a new month or season, or None.
pub fn the_period_they_ask_about(understood: &Understood, memory: &Memory) -> Option<String> {
    let line = _line_of(understood);
    if !_a_period_and_nothing_else(&line) {
        return None;
    }
    let period = _the_period_in(&line);
    let asked = carried_topic(memory, &line).map(|r| rget(r, "text")).unwrap_or_default();
    let was = _the_period_in(&asked);
    if was == period {
        return None;
    }
    if was.is_empty() {
        if !_the_answer_was_about_the_weather(memory) {
            return None;
        }
        let place = _the_place_the_chat_is_on(&line, memory)?;
        return Some(format!("what is the weather like in {place} in {period}"));
    }
    let low = asked.to_lowercase();
    if !ASK_CUE.is_match(&low) && !WEATHER_ASKS.is_match(&low) {
        return None;
    }
    Some(_with_the_period(&asked, &was, &period))
}

/// `_the_answer_was_about_the_weather(memory)`: the last thing the bot said was about the weather.
pub fn _the_answer_was_about_the_weather(memory: &Memory) -> bool {
    for turn in memory.turns.iter().rev() {
        if text::strip(&turn.bot).is_empty() {
            continue;
        }
        return words(&turn.bot).iter().any(|w| WEATHER_WORDS.contains(w));
    }
    false
}

/// `_the_place_the_chat_is_on(line, memory)`: the place the chat is still on, from the trip the
/// person stated, or None.
pub fn _the_place_the_chat_is_on(line: &str, memory: &Memory) -> Option<String> {
    let mine = words(line).join(" ");
    for said in _the_lines_said(memory) {
        if words(&said).join(" ") == mine {
            continue;
        }
        if _the_weather_ask_in(&said) {
            continue;
        }
        if _a_period_and_nothing_else(&said) {
            continue;
        }
        if A_TRIP_TAKEN.is_match(&low_squash(&said)) {
            return None;
        }
        let (place, _month) = the_place_and_month(&said);
        return place;
    }
    None
}

/// The first whole-word match of `was` in `asked` swapped for `now`, lower-cased and squashed.
fn _swap_word(asked: &str, was: &str, now: &str) -> String {
    let pattern = format!(r"(?<![a-z]){}(?![a-z])", crate::py::re::escape(was));
    let out = crate::understanding::rx(&pattern).sub_with(asked, 1, |_| now.to_string());
    low_squash(&out)
}

/// `_with_the_period(asked, was, now)`: the earlier question with its period swapped.
pub fn _with_the_period(asked: &str, was: &str, now: &str) -> String {
    _swap_word(asked, was, now)
}

/// `_the_meal_in(text)`: the meal a line names, or "".
pub fn _the_meal_in(text_: &str) -> String {
    let low = low_squash(text_);
    for name in MEALS_LONGEST_FIRST.iter() {
        if crate::understanding::rx(&format!(r"(?<![a-z]){name}(?![a-z])")).is_match(&low) {
            return name.clone();
        }
    }
    String::new()
}

/// `_a_meal_and_nothing_else(line)`: a line that names a meal and nothing else of its own.
pub fn _a_meal_and_nothing_else(line: &str) -> bool {
    if _the_meal_in(line).is_empty() {
        return false;
    }
    words(line).iter().all(|w| MEAL_ONLY.contains(w) || MEAL_NAMES.contains(w))
}

/// `the_meal_they_ask_about(understood, memory)`: the question on the table asked again about
/// another meal, or None.
pub fn the_meal_they_ask_about(understood: &Understood, memory: &Memory) -> Option<String> {
    let line = _line_of(understood);
    if !_a_meal_and_nothing_else(&line) {
        return None;
    }
    let meal = _the_meal_in(&line);
    let asked = carried_topic(memory, &line).map(|r| rget(r, "text")).unwrap_or_default();
    let was = _the_meal_in(&asked);
    if was.is_empty() {
        if OPENS_A_FOLLOW_UP.match_(&line).is_none() {
            return None;
        }
        return Some(format!("what can i have for {meal}"));
    }
    if was == meal {
        return None;
    }
    if !ASK_CUE.is_match(&asked.to_lowercase()) {
        return None;
    }
    Some(_with_the_meal(&asked, &was, &meal))
}

/// `meal_follow_up(understood, memory)`: the line read again with the meal it names in it, as a
/// reading, or None.
pub fn meal_follow_up(understood: &Understood, memory: &Memory) -> Option<Understood> {
    let meal = the_meal_they_ask_about(understood, memory).filter(|m| !m.is_empty())?;
    Some(_reading_with(understood, &meal, true))
}

/// `_with_the_meal(asked, was, now)`: the earlier question with its meal swapped.
pub fn _with_the_meal(asked: &str, was: &str, now: &str) -> String {
    _swap_word(asked, was, now)
}

/// `the_weather_of_the_trip(understood, memory)`: the trip's climate question, when a weather
/// follow-up can be answered with it, or None.
pub fn the_weather_of_the_trip(understood: &Understood, memory: &Memory) -> Option<String> {
    if !the_weather_ask(understood) {
        return None;
    }
    let line = _line_of(understood);
    let (place, _) = the_place_and_month(&line);
    if place.is_some() {
        return None;
    }
    if NOW_AND_HERE.is_match(&line.to_lowercase()) {
        return None;
    }
    let (place, month) = the_place_and_month(&_the_trip_said(&line, memory));
    let (place, month) = (place?, month?);
    Some(format!("what is the weather like in {place} in {month}"))
}

/// `_the_trip_said(line, memory)`: the trip the chat is on, when it names a place and a month,
/// or "".
pub fn _the_trip_said(line: &str, memory: &Memory) -> String {
    let mine = words(line).join(" ");
    for said in _the_lines_said(memory) {
        if words(&said).join(" ") == mine {
            continue;
        }
        if _the_weather_ask_in(&said) {
            continue;
        }
        let (place, month) = the_place_and_month(&said);
        if place.is_none() || month.is_none() {
            return String::new();
        }
        if A_TRIP_TAKEN.is_match(&low_squash(&said)) {
            return String::new();
        }
        return said;
    }
    String::new()
}

/// `_the_lines_said(memory)`: the lines the person has said, newest first.
pub fn _the_lines_said(memory: &Memory) -> Vec<String> {
    memory.turns.iter().rev().filter(|t_| !text::strip(&t_.user).is_empty()).map(|t_| t_.user.clone()).collect()
}

/// `the_ask_about_the_thing_it_points_at(understood, memory)`: the reading with the thing a
/// pronoun points at written in front of it, or None.
pub fn the_ask_about_the_thing_it_points_at(understood: &Understood, memory: &Memory) -> Option<Understood> {
    let _ = memory;
    let line = text::squash(&_line_of(understood));
    if line.is_empty() || !THING_ITSELF.is_match(&line) {
        return None;
    }
    let low = line.to_lowercase();
    if PROCEED.iter().any(|p| low.contains(p.as_str())) {
        return None;
    }
    if IT_IS_PART_OF_THE_PHRASE.iter().any(|p| low.contains(p.as_str())) {
        return None;
    }
    if WHAT_IT_COSTS.is_match(&line) {
        return None;
    }
    if understood.reference.kind.as_deref() != Some("thing") {
        return None;
    }
    let thing = understood.reference.it.clone().filter(|s| !s.is_empty()).or_else(|| understood.reference.that.clone())?;
    let thing = text::squash(&thing);
    if thing.is_empty() || text::split(&thing).len() > 3 || ["it", "that", "them", "they", "you", "me"].contains(&thing.as_str()) {
        return None;
    }
    if low.contains(&thing) {
        return None;
    }
    let put = format!("my {thing}");
    let filled = text::squash(&THING_ITSELF.sub_with(&line, 1, |_| put.clone()));
    Some(_reading_with(understood, &filled, false))
}

/// `the_pronoun_written_as_what_it_stands_for(understood, memory)`: this line with its pronoun
/// replaced by the thing it stands for, or None.
pub fn the_pronoun_written_as_what_it_stands_for(understood: &Understood, memory: &Memory) -> Option<Understood> {
    let line = text::squash(&_line_of(understood));
    if line.is_empty() {
        return None;
    }
    if understood.kind != "question" {
        return None;
    }
    let low = line.to_lowercase();
    if PROCEED.iter().any(|p| low.contains(p.as_str())) {
        return None;
    }
    if IT_IS_PART_OF_THE_PHRASE.iter().any(|p| low.contains(p.as_str())) {
        return None;
    }
    if _entry(&line).is_some() {
        return None;
    }
    let last_id = _last_answer_id(memory);
    let record = carried_topic(memory, &line);
    let topic_entry = record.and_then(|r| _entry(&rget(r, "text")));
    let topic_domain = topic_entry.map(|e| e.domain.clone()).filter(|d| !d.is_empty());
    for word in POINTING_WORDS.iter() {
        let Some(thing) = _ref_slot(understood, word) else { continue };
        let thing = text::squash(thing);
        if thing.is_empty() || ["it", "that", "them", "they", "you", "me", "there"].contains(&thing.to_lowercase().as_str()) {
            continue;
        }
        // a pattern made of the user's own words is compiled for this call, not kept
        let named = Re::new(&format!(r"\b{}\b", crate::py::re::escape(&thing.to_lowercase())), 0);
        if named.is_match(&low) {
            continue;
        }
        let pointer = crate::understanding::rx_flags(&format!(r"\b{word}\b"), crate::py::re::I);
        if !pointer.is_match(&line) {
            continue;
        }
        let last_word = text::split(&thing).last().unwrap().to_lowercase();
        for phrase in _the_thing_with_a_determiner(&thing) {
            let filled = low_squash(&pointer.sub_with(&line, 1, |_| phrase.clone()));
            let Some(entry) = _entry(&filled) else { continue };
            if !_the_entry_is_about_the_thing(entry, &last_word) {
                continue;
            }
            if !_the_entry_answers_the_ask(entry, &filled) {
                continue;
            }
            if let Some(domain) = &topic_domain {
                if entry.domain != *domain {
                    continue;
                }
            }
            if let Some(id) = last_id.as_ref().filter(|i| i.truthy()) {
                if Value::from(entry.id.as_str()) == *id {
                    continue;
                }
            }
            return Some(_reading_with(understood, &filled, true));
        }
    }
    None
}

/// `_the_thing_with_a_determiner(thing)`: the ways this thing can be written into a line.
pub fn _the_thing_with_a_determiner(thing: &str) -> Vec<String> {
    let mut ways = Vec::new();
    let first = text::split(thing).first().map(|f| f.to_lowercase()).unwrap_or_default();
    if !WITHOUT_A_DETERMINER.contains(&first) {
        let article = if first.chars().next().is_some_and(|c| "aeiou".contains(c)) { "an" } else { "a" };
        ways.push(format!("{article} {thing}"));
        ways.push(format!("the {thing}"));
        ways.push(format!("my {thing}"));
    }
    ways.push(thing.to_string());
    ways
}

/// `the_prognosis_the_line_asks(understood, memory)`: a bare "will it get better", read as the
/// prognosis of the thing the chat is on, or None.
pub fn the_prognosis_the_line_asks(understood: &Understood, memory: &Memory) -> Option<Understood> {
    let line = text::strip(&_line_of(understood)).to_string();
    if line.is_empty() || !YES_NO.is_match(&line) || !WHAT_STANDS_IN.is_match(&line) {
        return None;
    }
    let said = words(&line);
    let outcomes: Vec<String> = said.iter().filter_map(|w| PROGNOSIS.get(w).cloned()).collect();
    if outcomes.is_empty() {
        return None;
    }
    if said.iter().any(|w| !STOPWORDS.contains(w) && !STANDS_IN.contains(w) && !PROGNOSIS.contains_key(w) && !WHAT_IT_IS_LIKE.contains(w)) {
        return None;
    }
    if _about_somebody(memory) {
        return None;
    }
    let thing = subject(memory);
    let named = if thing.is_empty() { Vec::new() } else { words(&thing) };
    if named.len() != 1 || named.iter().any(|w| WHO.contains(w)) {
        return None;
    }
    let word = named[0].clone();
    let already = _entry(&topic_text(memory));
    let mut tried: Vec<String> = Vec::new();
    for outcome in outcomes.into_iter().chain(["last".to_string(), "get better".to_string()]) {
        if tried.contains(&outcome) {
            continue;
        }
        tried.push(outcome.clone());
        for written in _the_thing_as_a_question_names_it(&named) {
            let ask = if outcome == "last" {
                format!("how long does {written} last")
            } else {
                format!("how long does {written} take to {outcome}")
            };
            let Some(entry) = _entry(&ask) else { continue };
            if !_the_entry_is_about_the_thing(entry, &word) {
                continue;
            }
            if !_keyed_on_how_long(entry) {
                continue;
            }
            if already.is_some_and(|a| a.key == entry.key) {
                continue;
            }
            return Some(_reading_with(understood, &text::squash(&ask), true));
        }
    }
    None
}

/// `_keyed_on_how_long(entry)`: the entry is the row about how long the thing goes on.
pub fn _keyed_on_how_long(entry: &'static Entry) -> bool {
    entry.key.iter().any(|group| {
        HOW_LONG_KEY.iter().any(|phrase| match group {
            KeyPart::Word(s) => s.contains(phrase.as_str()),
            KeyPart::Any(v) => v.iter().any(|a| a == phrase),
        })
    })
}

/// `_the_thing_as_a_question_names_it(named)`: the thing as said, then with an article.
pub fn _the_thing_as_a_question_names_it(named: &[String]) -> Vec<String> {
    let whole = named.join(" ");
    if ["my", "a", "an", "the", "his", "her", "our", "their", "your"].contains(&named[0].as_str()) {
        return vec![whole];
    }
    vec![whole.clone(), format!("a {whole}"), format!("the {whole}")]
}

/// `the_property_of_the_thing(understood, memory)`: a bare yes/no about what the thing in hand is
/// like, read with that thing in it, or None.
pub fn the_property_of_the_thing(understood: &Understood, memory: &Memory) -> Option<Understood> {
    let line = text::strip(&_line_of(understood)).to_string();
    if line.is_empty() || !YES_NO.is_match(&line) || !WHAT_STANDS_IN.is_match(&line) {
        return None;
    }
    let said = words(&line);
    if !said.iter().any(|w| WHAT_IT_IS_LIKE.contains(w)) {
        return None;
    }
    if said.iter().any(|w| !WHAT_IT_IS_LIKE.contains(w) && !STOPWORDS.contains(w) && !STANDS_IN.contains(w)) {
        return None;
    }
    if _about_somebody(memory) {
        return None;
    }
    let thing = subject(memory);
    if thing.is_empty() || words(&thing).iter().any(|w| WHO.contains(w)) {
        return None;
    }
    let mut item: Option<(String, &'static Entry)> = None;
    let mut already: Option<Option<&'static Entry>> = None;
    let mut fallback: Option<(String, &'static Entry)> = None;
    let money = said.iter().any(|w| MONEY.contains(w));
    for (looked, candidate) in _fills_from_the_things(&line, understood, &thing, memory).into_iter().enumerate() {
        if looked as i64 + 1 > *LIKE_LOOKUPS {
            break;
        }
        let Some(found) = _entry(&candidate) else { continue };
        let already = *already.get_or_insert_with(|| _entry(&topic_text(memory)));
        if already.is_some_and(|a| a.key == found.key) {
            continue;
        }
        if money && !_key_words_of(found).iter().any(|w| MONEY.contains(w)) {
            if fallback.is_none() {
                fallback = Some((candidate, found));
            }
            continue;
        }
        item = Some((candidate, found));
        break;
    }
    let (filled, _) = item.or(fallback)?;
    Some(_reading_with(understood, &low_squash(&filled), true))
}

/// `the_limit_of_the_thing(understood, memory)`: a bare "how much is too much" read with the
/// thing the chat is on in it, or None.
pub fn the_limit_of_the_thing(understood: &Understood, memory: &Memory) -> Option<Understood> {
    if _memory_is_empty(memory) {
        return None;
    }
    let line = text::strip(&_line_of(understood)).to_string();
    if line.is_empty() || !LIMIT_ASK.is_match(&line) {
        return None;
    }
    if words(&line).iter().any(|w| !THE_LIMIT_ITSELF.contains(w) && !STOPWORDS.contains(w)) {
        return None;
    }
    if _about_somebody(memory) {
        return None;
    }
    let thing = subject(memory);
    if thing.is_empty() || words(&thing).iter().any(|w| WHO.contains(w)) {
        return None;
    }
    let already = _entry(&topic_text(memory));
    let mut seen: Vec<String> = Vec::new();
    for candidate in _the_limit_written_out(&line, &thing, memory) {
        if seen.contains(&candidate) {
            continue;
        }
        seen.push(candidate.clone());
        if seen.len() as i64 > *LIKE_LOOKUPS {
            break;
        }
        let Some(found) = _entry(&candidate) else { continue };
        let keyed = _key_words_of(found);
        if !["limit", "limits", "safe"].iter().any(|w| keyed.contains(*w)) {
            continue;
        }
        if already.is_some_and(|a| a.key == found.key) {
            continue;
        }
        return Some(_reading_with(understood, &candidate, true));
    }
    None
}

/// `key_subject(line, memory)` inside Python's `try`: a failure is no key at all.
fn _key_subject_or_none(line: &str, memory: &Memory) -> Vec<String> {
    crate::runtime::guarded(|| key_subject(line, Some(memory))).unwrap_or_default()
}

/// `_the_limit_written_out(line, thing, memory)`: the ask with the thing in it, the thing's own
/// name before the doing word beside it.
pub fn _the_limit_written_out(line: &str, thing: &str, memory: &Memory) -> Vec<String> {
    let whole = text::squash(thing);
    let all = text::split(&whole);
    let named: Vec<String> = all.iter().filter(|w| !_a_doing_word(w)).cloned().collect();
    let mut parts: Vec<String> = named.clone();
    parts.push(whole.clone());
    parts.extend(all.iter().filter(|w| !named.contains(w)).cloned());
    for word in _key_subject_or_none(line, memory) {
        let w = text::strip(&word).to_string();
        if !w.is_empty() && !parts.contains(&w) {
            parts.push(w);
        }
    }
    let mut out = Vec::new();
    for one in &parts {
        for said in THE_LIMIT.iter() {
            out.push(said.replace("{thing}", one));
        }
    }
    out
}

/// `_a_doing_word(word)`: a word that says what is done rather than what it is done to.
pub fn _a_doing_word(word: &str) -> bool {
    VERB_FORMS.contains(word) || VERBISH.contains(word)
}

/// `_fills_from_the_things(line, understood, thing, memory)`: every ask the line means, the gist
/// of the thing first and then the words of its key.
pub fn _fills_from_the_things(line: &str, understood: &Understood, thing: &str, memory: &Memory) -> Vec<String> {
    let mut things = vec![thing.to_string()];
    for word in _key_subject_or_none(&_line_of(understood), memory) {
        if !text::strip(&word).is_empty() {
            things.push(word);
        }
    }
    let mut out: Vec<String> = Vec::new();
    for one in &things {
        for ask in _fills(line, one) {
            if !out.contains(&ask) {
                out.push(ask);
            }
        }
    }
    out
}

/// `_fills(line, thing)`: the asks the line means, written out with the thing in it, best first.
pub fn _fills(line: &str, thing: &str) -> Vec<String> {
    let whole = text::squash(thing);
    let said = WHAT_STANDS_IN.sub_with(line, 1, |_| whole.clone());
    if words(line).iter().any(|w| MONEY.contains(w)) {
        return vec![format!("how much does {whole} cost"), format!("is {whole} expensive")];
    }
    vec![said]
}

/// `the_airport_there_is(understood, memory)`: a question about how early to be there, in a chat
/// about a flight, read as the airport.
pub fn the_airport_there_is(understood: &Understood, memory: &Memory) -> Option<Understood> {
    let line = text::strip(&_line_of(understood)).to_string();
    if line.is_empty() || !words(&line).iter().any(|w| w == "there") || !ASKS_HOW_EARLY_THERE.is_match(&line) {
        return None;
    }
    let trip = topic_text(memory);
    if trip.is_empty() || !A_FLIGHT_IN_THE_TOPIC.is_match(&trip) {
        return None;
    }
    let airport: &str = &THE_AIRPORT;
    let filled = THERE_WORD.sub_with(&line, 1, |_| airport.to_string());
    if filled == line {
        return None;
    }
    let found = _entry(&filled)?;
    if _entry(&trip).is_some_and(|a| a.key == found.key) {
        return None;
    }
    Some(_reading_with(understood, &low_squash(&filled), true))
}

/// `the_animal_the_subject_is(understood, memory)`: a line whose subject is a bare "he"/"she",
/// read with the animal the chat is on, or None.
pub fn the_animal_the_subject_is(understood: Option<&Understood>, memory: &Memory) -> Option<Understood> {
    let understood = understood?;
    let line = text::strip(&_line_of(understood)).to_string();
    if line.is_empty() || PET_WORD.is_match(&line) {
        return None;
    }
    let opens = HE_OR_SHE_OPENS.match_(&line)?;
    let who = understood
        .reference
        .he
        .clone()
        .filter(|h| !h.is_empty())
        .or_else(|| understood.reference.she.clone())
        .unwrap_or_default();
    if who.is_empty() || !PET_WORD.is_match(&who) {
        return None;
    }
    let mut thing = the_pet_the_line_before_named(memory);
    if thing.is_empty() {
        thing = _called(&who);
    }
    let filled = format!("{}{} {}", opens.name_str("lead"), thing, text::lstrip(&line[opens.end()..]));
    let item = _entry(&filled)?;
    if _entry(&line).is_some_and(|e| std::ptr::eq(e, item)) {
        return None;
    }
    let kind = text::rstrip_chars(&text::split(&thing).last().cloned().unwrap_or_default().to_lowercase(), "s").to_string();
    let keyed = _key_words_of(item);
    if !keyed.contains(&kind) && !keyed.iter().any(|w| stem(w) == kind) {
        return None;
    }
    Some(_reading_with(understood, &low_squash(&filled), false))
}

/// `_the_kind_of_animal(text)`: the animal a line or an answer names, as the family word.
pub fn _the_kind_of_animal(text_: &str) -> HashSet<String> {
    let mut found = HashSet::new();
    for word in PET_WORD.findall_str(text_) {
        let word = text::rstrip_chars(&_pet_one(&word.to_lowercase()), "s").to_string();
        found.insert(BABY_OF.get(&word).cloned().unwrap_or(word));
    }
    found
}

/// `answers_about_another_creature(answer, what_the_line_is)`: the answer in hand is about a
/// person or another animal than the one the chat is on.
pub fn answers_about_another_creature(answer: &str, what_the_line_is: &str) -> bool {
    let said = text::split(answer).into_iter().take(60).collect::<Vec<_>>().join(" ");
    if said.is_empty() {
        return false;
    }
    if CHILD_CARE.is_match(&said) {
        return true;
    }
    let kind = _the_kind_of_animal(what_the_line_is);
    let named = _the_kind_of_animal(&said);
    !named.is_empty() && !named.iter().any(|n| kind.contains(n))
}

/// `_called(who)`: "my dog" from the animal the reading points at.
pub fn _called(who: &str) -> String {
    let who = low_squash(text::strip(who));
    if who.is_empty() {
        return String::new();
    }
    let first = text::split(&who)[0].clone();
    if ["my", "our", "the", "a"].contains(&first.as_str()) { who } else { format!("my {who}") }
}

/// `_the_pet_the_turns_before_named(memory, back=3)`: "my puppy" when the puppy was named a turn
/// or two before, else "".
pub fn _the_pet_the_turns_before_named(memory: &Memory, back: i64) -> String {
    let thing = the_pet_the_line_before_named(memory);
    if !thing.is_empty() {
        return thing;
    }
    let user = memory.get("user").filter(|u| is_dict(u)).cloned().unwrap_or_else(Value::dict);
    let mut lines: Vec<String> = Vec::new();
    for turn in memory.turns.iter().rev() {
        if !text::strip(&turn.user).is_empty() {
            lines.push(turn.user.clone());
        }
        if lines.len() as i64 > back {
            break;
        }
    }
    let end = ((back + 1).max(0) as usize).min(lines.len());
    for line in lines.iter().take(end).skip(1) {
        let mut one = Memory::new();
        one.turns.push(crate::state::Turn { user: line.clone(), bot: String::new(), module: None });
        one.set("user", user.clone());
        let found = the_pet_the_line_before_named(&one);
        if !found.is_empty() {
            return found;
        }
    }
    String::new()
}

/// `a_detail_of_the_thing_we_hold(understood, memory)`: "the chewing" when that is a detail of the
/// animal the store holds.
pub fn a_detail_of_the_thing_we_hold(understood: &Understood, memory: &Memory) -> bool {
    _the_kind_the_line_leaves_out(understood, memory).is_some()
}

/// `_the_kind_the_line_leaves_out(understood, memory)`: (line, kind) when the line names a detail
/// and leaves out the animal the store holds.
pub fn _the_kind_the_line_leaves_out(understood: &Understood, memory: &Memory) -> Option<(String, String)> {
    let line = text::strip(&_line_of(understood)).to_string();
    if line.is_empty() || !ASK_CUE.is_match(&line.to_lowercase()) {
        return None;
    }
    let low = line.to_lowercase();
    if PET_WORD.is_match(&line) || PROCEED.iter().any(|p| low.contains(p.as_str())) {
        return None;
    }
    let thing = _the_pet_the_turns_before_named(memory, 3);
    if thing.is_empty() {
        return None;
    }
    let kind = text::split(&thing).last().unwrap().clone();
    if words(&line).contains(&kind) {
        return None;
    }
    Some((line, kind))
}

/// `the_thing_a_detail_belongs_to(understood, memory)`: the ask read with the kind of thing the
/// store holds, when the ask names only a detail.
pub fn the_thing_a_detail_belongs_to(understood: &Understood, memory: &Memory) -> Option<Understood> {
    let (line, kind) = _the_kind_the_line_leaves_out(understood, memory)?;
    let filled = format!("{line} {kind}");
    let item = _entry(&filled)?;
    let keyed = _key_words_of(item);
    if !keyed.contains(&kind) && !keyed.contains(&stem(&kind)) {
        return None;
    }
    let said: Vec<String> = words(&line)
        .into_iter()
        .filter(|w| !ASK_PART.contains(w) && !ASK_ONLY.contains(w) && !STAND_IN.contains(w) && !STOPWORDS.contains(w))
        .collect();
    let trouble: Vec<String> = said.iter().filter(|w| !IMPERATIVE.contains(*w)).cloned().collect();
    if !_names_it(if trouble.is_empty() { &said } else { &trouble }, item) {
        return None;
    }
    Some(_reading_with(understood, &low_squash(&filled), false))
}

/// `_carrying_on(understood, memory)`: the bare "how do i keep it up" as the carrying-on
/// question, or None.
pub fn _carrying_on(understood: &Understood, memory: &Memory) -> Option<Understood> {
    let line = _line_of(understood);
    let found = KEEP_IT_UP.search(&line)?;
    let topic = carried_topic(memory, &line).map(|r| rget(r, "text")).unwrap_or_default();
    if text::strip(&topic).is_empty() || !AN_ACTIVITY.is_match(&topic) {
        return None;
    }
    let staying: &str = &STAYING_WITH_IT;
    let said = format!("{}{}{}", &line[..found.start()], staying, &line[found.end()..]);
    Some(_reading_with(understood, &low_squash(text::strip(&said)), true))
}

/// `_what_the_turn_took_away(record, memory)`: the things a turn took off the thing in hand.
pub fn _what_the_turn_took_away(record: &Value, memory: &Memory) -> HashSet<String> {
    let _ = memory;
    if !is_dict(record) {
        return HashSet::new();
    }
    let text_or_user = record.get("text").filter(|v| v.truthy()).or_else(|| record.get("user")).map(value_str).unwrap_or_default();
    let said = low_squash(&text_or_user);
    let Some(found) = LIMIT_LINE.match_(&said) else { return HashSet::new() };
    let mut taken = HashSet::new();
    for word in text::split(found.name_str("thing")) {
        if STOPWORDS.contains(&word) || text::len(&word) < 3 {
            continue;
        }
        taken.insert(stem(&word));
    }
    taken
}

/// `_widened(line, understood, record, memory)`: this line read with the record's subject put
/// back in, or None when that reaches nothing.
pub fn _widened(line: &str, understood: &Understood, record: &Value, memory: &Memory) -> Option<String> {
    if deps::truth("bot.memory.when:asks_what_today_is", deps::args1(line), || crate::memory::when::asks_what_today_is(line)) {
        return None;
    }
    if !is_dict(record) {
        return None;
    }
    let topic_text = rget(record, "text");
    if text::strip(&topic_text).is_empty() {
        return None;
    }
    let taken = _what_the_turn_took_away(record, memory);
    if leans_on_topic(understood, memory) {
        let mut subject_words = _subject_words_to_carry(record, line, memory);
        if !taken.is_empty() {
            subject_words.retain(|w| !taken.contains(&stem(&w.to_lowercase())));
        }
        let said = words(line);
        if subject_words.is_empty() || subject_words.iter().all(|w| said.contains(w)) {
            return None;
        }
        if let Some(text_) = widen(line, &subject_words, &topic_text) {
            return Some(text_);
        }
        return carry_the_ask_with(line, &topic_text, None, None);
    }
    carry_the_ask_with(line, &topic_text, None, None)
}

/// `_the_length_of_the_aim(understood, memory)`: this turn read as the length of the aim the chat
/// is on, when it asks only a length. A reading that fails leaves the line as it was.
pub fn _the_length_of_the_aim(understood: &Understood, memory: &Memory) -> Option<Understood> {
    let line = _line_of(understood);
    if line.is_empty() {
        return None;
    }
    let asked = crate::runtime::guarded(|| crate::memory::named::a_goal_measured(&line, memory))?;
    if asked.is_empty() {
        return None;
    }
    Some(_reading_with(understood, &asked, true))
}

/// `_a_gathering_read_as_the_food(understood, line, memory)`: "we are having a barbecue" read as
/// the food for it, when its own words reach nothing.
pub fn _a_gathering_read_as_the_food(understood: &Understood, line: &str, memory: &Memory) -> Option<String> {
    let _ = memory;
    if understood.kind != "statement" || !understood.asks.is_empty() {
        return None;
    }
    let mut gathering: Vec<String> = words(line).into_iter().filter(|w| GATHERING_WORDS.contains(w)).collect();
    gathering.sort();
    gathering.dedup();
    if gathering.is_empty() || !PUTTING_ON.is_match(line) {
        return None;
    }
    if _entry(line).is_some() {
        return None;
    }
    if TOO_SOON_TO_COOK.is_match(line) {
        return None;
    }
    if FOR_SOMEBODY.is_match(line) {
        return None;
    }
    for candidate in [format!("{} food", gathering.join(" ")), "party food".to_string()] {
        if _entry(&candidate).is_some() {
            return Some(candidate);
        }
    }
    None
}

/// `_the_whole_topic(line, record_text, memory=None)`: the line read with the topic's own words,
/// phrase and all, or None when that reaches nothing.
pub fn _the_whole_topic(line: &str, record_text: &str, memory: &Memory) -> Option<String> {
    if !_leads_a_question(line) {
        return None;
    }
    let mut best: Option<String> = None;
    let mut best_score: i64 = -1;
    let last_id = _last_answer_id(memory);
    for said in _the_things_the_chat_named(record_text, memory, line) {
        let widened = format!("{line} {said}");
        let Some(reached) = _entry(&widened) else { continue };
        if last_id.as_ref().is_some_and(|id| *id == Value::from(reached.id.as_str())) {
            continue;
        }
        if _already_answered_here(reached, memory) {
            continue;
        }
        if !_the_answer_is_about_what_the_line_names(reached, line) {
            continue;
        }
        let score = _how_much_of_the_thing_the_entry_uses(reached, &said);
        if score > best_score {
            best = Some(widened);
            best_score = score;
        }
    }
    best
}

/// `_the_answer_is_about_what_the_line_names(entry, line)`: the entry's own answer talks about the
/// thing the line asks about.
pub fn _the_answer_is_about_what_the_line_names(entry: &'static Entry, line: &str) -> bool {
    let asked: Vec<String> = keywords(line).into_iter().filter(|w| text::len(w) > 3).collect();
    if asked.is_empty() {
        return true;
    }
    let named: Vec<String> = asked.into_iter().filter(|w| !ASK_SHAPE_WORDS.contains(w)).collect();
    if named.is_empty() {
        return false;
    }
    let said = words(&_reply_as_str(entry));
    named.iter().any(|w| said.iter().any(|token| alike(w, token)))
}

/// `_already_answered_here(entry, memory)`: the reply the entry gives has already been said in
/// this conversation.
pub fn _already_answered_here(entry: &'static Entry, memory: &Memory) -> bool {
    let reply = text::strip(&_reply_as_str(entry)).to_string();
    if reply.is_empty() {
        return false;
    }
    let opening = text::split(&reply.to_lowercase()).into_iter().take(8).collect::<Vec<_>>().join(" ");
    memory.turns.iter().any(|turn| {
        let said = low_squash(&turn.bot);
        !said.is_empty() && said.starts_with(&opening)
    })
}

/// `_how_much_of_the_thing_the_entry_uses(entry, said)`: how many of the thing's own content words
/// the entry is keyed on.
pub fn _how_much_of_the_thing_the_entry_uses(entry: &'static Entry, said: &str) -> i64 {
    let keyed = _key_words_of(entry);
    keywords(said).iter().filter(|w| text::len(w) > 2 && keyed.iter().any(|k| alike(w, k))).count() as i64
}

/// `_the_things_the_chat_named(record_text, memory, line="")`: the things the chat has named,
/// newest first, to read a follow-up against.
pub fn _the_things_the_chat_named(record_text: &str, memory: &Memory, line: &str) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    let mut out: Vec<String> = Vec::new();
    let mut texts = vec![record_text.to_string()];
    if _leads_a_question(line) {
        let all = records(memory);
        for record in _last(all, 4).iter().rev() {
            if is_dict(record) {
                texts.push(rget(record, "text"));
            }
        }
    }
    for t_ in texts {
        let said = low_squash(&t_);
        if said.is_empty() || text::split(&said).len() > 12 {
            continue;
        }
        let key = words(&said).join(" ");
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        out.push(said);
    }
    out
}

/// `followup_reading(understood, memory)`: this turn's reading with the current topic's subject
/// put back in, when the line leans on it.
pub fn followup_reading(understood: &Understood, memory: &Memory) -> Understood {
    if let Some(r) = _carrying_on(understood, memory) {
        return r;
    }
    if let Some(r) = next_slot_reading(understood, memory) {
        return r;
    }
    if _after_an_emergency(memory) || _after_a_correction(memory) {
        return understood.clone();
    }
    if let Some(r) = _the_length_of_the_aim(understood, memory) {
        return r;
    }
    if let Some(r) = the_pronoun_written_as_what_it_stands_for(understood, memory) {
        return r;
    }
    if let Some(r) = the_thing_it_is(understood, memory) {
        return r;
    }
    if let Some(r) = the_prognosis_the_line_asks(understood, memory) {
        return r;
    }
    if let Some(r) = the_property_of_the_thing(understood, memory) {
        return r;
    }
    if let Some(r) = the_animal_the_subject_is(Some(understood), memory) {
        return r;
    }
    if let Some(r) = the_airport_there_is(understood, memory) {
        return r;
    }
    if let Some(r) = the_limit_of_the_thing(understood, memory) {
        return r;
    }
    let said = _line_of(understood);
    let line = deps::string("bot.memory.filler:without_the_shrug", deps::args1(&said), || crate::memory::filler::without_the_shrug(&said));
    if let Some(gathering) = _a_gathering_read_as_the_food(understood, &line, memory) {
        return _reading_with(understood, &gathering, true);
    }
    if let Some(period) = the_period_they_ask_about(understood, memory) {
        return _reading_with(understood, &period, true);
    }
    if let Some(meal) = meal_follow_up(understood, memory) {
        return meal;
    }
    if let Some(trip) = the_weather_of_the_trip(understood, memory) {
        return _reading_with(understood, &trip, true);
    }
    let Some(record) = carried_topic(memory, &line).cloned() else { return understood.clone() };
    if left_for_a_blank(memory, &line) {
        return understood.clone();
    }
    let same_ask = how_much_of_the_thing_in_hand(&line, memory);
    if !same_ask.is_empty() {
        return _reading_with(understood, &same_ask, true);
    }
    if let Some(brought) = the_thing_the_subject_brings(understood, memory) {
        return _reading_with(understood, &brought, true);
    }
    let topic = rget(&record, "text");
    if elided_ask(&line) {
        // Python builds a reading from the situated ask here and drops it: only the call (and the
        // cache it fills) is kept.
        let _ = carry_the_ask(&line, &topic, understood, memory);
    }
    let leaning = _the_turn_we_lean_on(understood, memory);
    if !leaning.is_none() && _said_later_than(&leaning, &record) {
        if let Some(newer) = _widened(&line, understood, &leaning, memory) {
            return _reading_with(understood, &low_squash(&newer), true);
        }
    }
    let text_: String;
    if understood.intent == "correction" {
        if text::strip(&topic).is_empty() || words(&topic).join(" ") == words(&line).join(" ") {
            return understood.clone();
        }
        // a correction is read as the topic it corrected
        text_ = topic.clone();
    } else if bare_question(understood, memory) && carry_the_ask(&line, &topic, understood, memory).is_none() {
        let mut extra: Vec<String> = Vec::new();
        let not_doing = _what_they_are_not_doing(&topic);
        let said = words(&line);
        for word in _subject_words_to_carry(&record, &line, memory) {
            if not_doing.contains(&word) || _how_they_feel(&word) {
                continue;
            }
            if !said.contains(&word) && !extra.contains(&word) {
                extra.push(word);
            }
        }
        if extra.is_empty() {
            return understood.clone();
        }
        let widened = format!("{line} {}", extra.join(" "));
        if _entry(&widened).is_none() {
            let mut whole_words = text::split(&topic);
            whole_words.push(line.clone());
            let said_whole = whole_words.join(" ");
            let item = _entry(&said_whole);
            match item {
                Some(item) if extra.iter().any(|w| _keyed_to(&_key_words_of(item), w)) => text_ = said_whole,
                _ => return understood.clone(),
            }
        } else {
            text_ = widened;
        }
    } else if !_still_the_topic(&record, memory) {
        return understood.clone();
    } else if leans_on_topic(understood, memory) {
        let subject_words = _subject_words_to_carry(&record, &line, memory);
        let said = words(&line);
        if subject_words.is_empty() || subject_words.iter().all(|w| said.contains(w)) {
            return understood.clone();
        }
        let found = widen(&line, &subject_words, &topic)
            .or_else(|| carry_the_ask(&line, &topic, understood, memory))
            .or_else(|| _constrains_the_ask(&line, &topic))
            .or_else(|| _the_whole_topic(&line, &topic, memory));
        match found {
            Some(t_) => text_ = t_,
            None => return understood.clone(),
        }
    } else {
        let found = carry_the_ask(&line, &topic, understood, memory)
            .or_else(|| _constrains_the_ask(&line, &topic))
            .or_else(|| _the_whole_topic(&line, &topic, memory));
        match found {
            Some(t_) => text_ = t_,
            None => return understood.clone(),
        }
    }
    _reading_with(understood, &low_squash(&text_), true)
}

thread_local! {
    /// The topic record as the turn's first `note` left it, and the turn it was: the observers
    /// run before any responder, so this is the record PyBot's runtime copies for the responders
    /// (see `_the_runtime_copied`).
    static TOPIC_AFTER_NOTE: std::cell::RefCell<Option<(usize, Value)>> = const { std::cell::RefCell::new(None) };
}

/// PyBot's runtime deep-copies each memory key on its own for every responder, and a chosen
/// reply's copy becomes the memory: from then on `memory["topic"]` is no longer the same dict as
/// its entry in `memory["topics"]`. The Rust runtime's copy shares them, so when the last turn
/// had a chosen reply and the topic is still the one the observers left, the sharing is ended
/// here. A record a responder or an after hook put in place was made after the copy, and keeps
/// whatever it shares, as in Python.
fn _the_runtime_copied(memory: &mut Memory) {
    let chosen = memory.turns.last().is_some_and(|t_| t_.module.is_some());
    if !chosen {
        return;
    }
    let left = TOPIC_AFTER_NOTE.with(|c| c.borrow().clone());
    let Some((turn, left)) = left else { return };
    if turn + 1 != memory.turns.len() {
        return;
    }
    if let Some(topic) = memory.get("topic") {
        if same_object(topic, &left) {
            let copied = fresh(topic);
            memory.set("topic", copied);
        }
    }
}

/// `note(memory, text, kind=None)`: file the line, and note what filing it put on the record the
/// chat is on.
pub fn note(memory: &mut Memory, text_: &str, kind: &str) -> Value {
    _the_runtime_copied(memory);
    _the_step_back_is_undone(memory);
    let before = _the_words_before_the_turn(memory);
    let previous = current(memory).cloned();
    let record = _note(memory, text_, kind);
    _the_words_this_turn(memory, &before, &record, previous.as_ref());
    let turn = memory.turns.len();
    TOPIC_AFTER_NOTE.with(|c| {
        let mut c = c.borrow_mut();
        // only the turn's first filing: a responder's own filing works on its copy
        if c.as_ref().is_none_or(|(t_, _)| *t_ != turn) {
            *c = Some((turn, memory.get("topic").cloned().unwrap_or(Value::None)));
        }
    });
    record
}

/// `_the_reply_to_the_turn_before(memory)`: what the bot said last.
pub fn _the_reply_to_the_turn_before(memory: &Memory) -> String {
    memory.turns.iter().rev().find(|t_| !t_.bot.is_empty()).map(|t_| t_.bot.clone()).unwrap_or_default()
}

/// `_the_step_back_is_undone(memory)`: take back the words the turn before added when its reply
/// was a step back.
pub fn _the_step_back_is_undone(memory: &mut Memory) -> Option<Vec<String>> {
    if !(memory.has(&WORDS_THIS_TURN) || memory.has(&TURN_TOPIC)) {
        return None;
    }
    let said = _the_reply_to_the_turn_before(memory);
    if said.is_empty() {
        memory.remove(&WORDS_THIS_TURN);
        memory.remove(&TURN_TOPIC);
        return None;
    }
    the_step_back_takes_its_words_back(memory, &said)
}

/// `_the_words_before_the_turn(memory)`: the words each record holds, by the text of the line it
/// came from.
pub fn _the_words_before_the_turn(memory: &Memory) -> IndexMap<String, Vec<String>> {
    let mut held = IndexMap::new();
    for record in _the_records(memory) {
        held.insert(rget(&record, "text"), rwords(&record));
    }
    held
}

/// `_the_records(memory)`: the topic record and then every record in `topics`.
pub fn _the_records(memory: &Memory) -> Vec<Value> {
    let mut out = Vec::new();
    if let Some(topic) = memory.get("topic").filter(|t_| is_dict(t_)) {
        out.push(topic.clone());
    }
    for record in records(memory) {
        if is_dict(record) {
            out.push(record.clone());
        }
    }
    out
}

/// `_the_words_this_turn(memory, before, record, previous)`: note what this turn added to the
/// record, and which record it left in place.
pub fn _the_words_this_turn(memory: &mut Memory, before: &IndexMap<String, Vec<String>>, record: &Value, previous: Option<&Value>) {
    memory.remove(&WORDS_THIS_TURN);
    memory.remove(&TURN_TOPIC);
    if !is_dict(record) {
        return;
    }
    let said = rget(record, "text");
    let had = before.get(&said);
    let added: Vec<String> = rwords(record).into_iter().filter(|w| !had.is_some_and(|h| h.contains(w))).collect();
    if !added.is_empty() {
        let mut noted = Value::dict();
        noted.set_key("text", said.as_str());
        noted.set_key("words", Value::from(added));
        memory.set(&WORDS_THIS_TURN, noted);
    }
    let mut turn_topic = Value::dict();
    turn_topic.set_key("text", said.as_str());
    turn_topic.set_key("of", rget(record, "of"));
    turn_topic.set_key("was", previous.filter(|p| is_dict(p)).map(|p| rget(p, "text")).unwrap_or_default());
    memory.set(&TURN_TOPIC, turn_topic);
}

/// `_a_step_back(reply)`: the reply is one of the pools that say the bot cannot answer this.
pub fn _a_step_back(reply: &str) -> bool {
    let said = text::strip(reply);
    crate::runtime::guarded(|| deps::truth("bot.compose.fallback:is_a_hedge", deps::args1(said), || crate::compose::fallback::is_a_hedge(said))).unwrap_or(false)
}

/// `_the_detail_steps_back(memory, reply)`: put back the thing the chat was on when this turn's
/// detail about it was only stepped back from.
pub fn _the_detail_steps_back(memory: &mut Memory, reply: &str) -> Option<Value> {
    let noted = memory.remove(&TURN_TOPIC);
    let noted = noted.filter(|n| is_dict(n))?;
    if !_a_step_back(reply) {
        return None;
    }
    let mut back = rget(&noted, "of");
    if back.is_empty() {
        back = rget(&noted, "was");
    }
    if back.is_empty() || back == rget(&noted, "text") {
        return None;
    }
    let record = current(memory)?;
    if !is_dict(record) || rget(record, "text") != rget(&noted, "text") {
        return None;
    }
    let older = records(memory).iter().filter(|k| is_dict(k) && rget(k, "text") == back).last().cloned()?;
    let said = fresh(&older);
    memory.set("topic", said.clone());
    if let Some(topics) = memory.get_mut("topics").and_then(Value::items_mut) {
        topics.retain(|other| !same_object(other, &older) && rget(other, "text") != back);
        topics.push(said.clone());
    }
    Some(said)
}

/// `the_step_back_takes_its_words_back(memory, reply)`: undo this turn's word merge when the turn
/// stepped back.
pub fn the_step_back_takes_its_words_back(memory: &mut Memory, reply: &str) -> Option<Vec<String>> {
    _the_detail_steps_back(memory, reply);
    let noted = memory.remove(&WORDS_THIS_TURN);
    let noted = noted.filter(|n| is_dict(n))?;
    if !_a_step_back(reply) {
        return None;
    }
    let taken = rwords(&noted);
    if taken.is_empty() {
        return None;
    }
    let text_ = rget(&noted, "text");
    let mut took_back: Option<Vec<String>> = None;
    // Every record holding that line, the topic and the items of `topics`, each changed in place:
    // one dict held in both places is changed once and stays one dict.
    let mut done: Vec<(Value, Value)> = Vec::new();
    let mut change = |record: &mut Value, took_back: &mut Option<Vec<String>>| {
        if !is_dict(record) || rget(record, "text") != text_ {
            return;
        }
        if let Some((_, new)) = done.iter().find(|(old, _)| same_object(old, record)) {
            *took_back = Some(rwords(new));
            *record = new.clone();
            return;
        }
        let kept: Vec<String> = rwords(record).into_iter().filter(|w| !taken.contains(w)).collect();
        let old = record.clone();
        let mut new = fresh(record);
        new.set_key("words", Value::from(kept.clone()));
        done.push((old, new.clone()));
        *record = new;
        *took_back = Some(kept);
    };
    if let Some(topic) = memory.get_mut("topic") {
        change(topic, &mut took_back);
    }
    if let Some(list) = memory.get_mut("topics").and_then(Value::items_mut) {
        for record in list.iter_mut() {
            change(record, &mut took_back);
        }
    }
    took_back
}

/// Results of other workstreams' functions recorded while PyBot ran, for MEM3's function-level
/// check (`src/bin/mem3_check.rs`): a call this file and `named` / `thread` make to another file
/// is answered from here when the checker has loaded a result for exactly those arguments, so a
/// reader can be checked before the file it leans on is ported. Empty in every real run, where
/// each call goes to the function itself.
pub mod deps {
    use serde_json::{json, Value as J};
    use std::cell::RefCell;
    use std::collections::HashMap;

    thread_local! {
        static RECORDED: RefCell<HashMap<String, J>> = RefCell::new(HashMap::new());
    }

    /// Load one recorded result: the function's Python name and its simple arguments.
    pub fn record(name: &str, args: &J, result: J) {
        RECORDED.with(|r| r.borrow_mut().insert(format!("{name}|{args}"), result));
    }

    fn lookup(name: &str, args: &J) -> Option<J> {
        RECORDED.with(|r| {
            let r = r.borrow();
            if r.is_empty() {
                return None;
            }
            r.get(&format!("{name}|{args}")).cloned()
        })
    }

    pub fn string(name: &str, args: J, real: impl FnOnce() -> String) -> String {
        match lookup(name, &args) {
            Some(J::String(s)) => s,
            Some(_) => String::new(),
            None => real(),
        }
    }
    pub fn opt_string(name: &str, args: J, real: impl FnOnce() -> Option<String>) -> Option<String> {
        match lookup(name, &args) {
            Some(J::String(s)) => Some(s),
            Some(_) => None,
            None => real(),
        }
    }
    pub fn truth(name: &str, args: J, real: impl FnOnce() -> bool) -> bool {
        match lookup(name, &args) {
            Some(J::Bool(b)) => b,
            Some(J::Null) => false,
            Some(J::String(s)) => !s.is_empty(),
            Some(J::Array(a)) => !a.is_empty(),
            Some(J::Object(o)) => !o.is_empty(),
            Some(J::Number(n)) => n.as_f64() != Some(0.0),
            None => real(),
        }
    }
    pub fn strings(name: &str, args: J, real: impl FnOnce() -> Vec<String>) -> Vec<String> {
        match lookup(name, &args) {
            Some(J::Array(a)) => a.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect(),
            Some(_) => Vec::new(),
            None => real(),
        }
    }
    pub fn value(name: &str, args: J, real: impl FnOnce() -> crate::data::Value) -> crate::data::Value {
        match lookup(name, &args) {
            Some(j) => crate::data::Value::from_plain(&j),
            None => real(),
        }
    }
    /// The simple arguments as the recorder keys them.
    pub fn args1(a: &str) -> J {
        json!([a])
    }
}

/// Expected values from PyBot (`python3 harness/pycall.py`-style, one process), for branches the
/// corpus reaches rarely.
#[cfg(test)]
mod mem3_py_tests {
    use crate::memory::{deadline, named, thread, topics};
    use crate::state::{Memory, Turn};

    /// PyBot: after a turn no reply was chosen for, `memory["topic"]` is still the same dict as
    /// the newest record in `memory["topics"]`, and a follow-up's words go on both; after a chosen
    /// reply the runtime's copy has parted them and only the topic gets the words.
    #[test]
    fn py_note_topic_and_topics_share_until_the_runtime_copies() {
        for line in ["my boiler keeps losing pressure", "how often should i check the boiler pressure"] {
            topics::deps::record("bot.memory.facts:a_moved_place", &serde_json::json!([line]), serde_json::json!(""));
        }
        for (module, both) in [(None, true), (Some("answers"), false)] {
            let mut m = Memory::new();
            let l1 = "my boiler keeps losing pressure";
            topics::note(&mut m, l1, "");
            m.turns.push(Turn { user: l1.into(), bot: "Top it up to 1.5 bar.".into(), module: module.map(str::to_string) });
            let mut m = m.clone();
            topics::note(&mut m, "how often should i check the boiler pressure", "");
            let topic: Vec<String> = m.get("topic").unwrap().get("words").unwrap().strs();
            let last: Vec<String> = m.get("topics").unwrap().items().last().unwrap().get("words").unwrap().strs();
            assert_eq!(topic, vec!["boiler", "keep", "losing", "pressure", "often", "check"]);
            let want: Vec<&str> = if both { vec!["boiler", "keep", "losing", "pressure", "often", "check"] } else { vec!["boiler", "keep", "losing", "pressure"] };
            assert_eq!(last, want);
        }
    }

    #[test]
    fn py_topics___gerund() {
        assert_eq!(topics::_gerund("make"), "making");
        assert_eq!(topics::_gerund("get"), "getting");
        assert_eq!(topics::_gerund("die"), "dying");
        assert_eq!(topics::_gerund("see"), "seeing");
        assert_eq!(topics::_gerund("fix"), "fixing");
        assert_eq!(topics::_gerund("plan"), "planning");
        assert_eq!(topics::_gerund("visit"), "visitting");
        assert_eq!(topics::_gerund("bake"), "baking");
        assert_eq!(topics::_gerund("flee"), "fleeing");
    }
    #[test]
    fn py_topics___the_aim_without_its_deadline() {
        assert_eq!(topics::_the_aim_without_its_deadline("im trying to get fit before the holiday"), "Get fit");
        assert_eq!(topics::_the_aim_without_its_deadline("we're hoping to save 2000 by june"), "Save 2000");
        assert_eq!(topics::_the_aim_without_its_deadline("so im planning to move house before christmas."), "Move house");
        assert_eq!(topics::_the_aim_without_its_deadline("im going to portugal in may"), "");
        assert_eq!(topics::_the_aim_without_its_deadline("im trying to x by june"), "");
        assert_eq!(topics::_the_aim_without_its_deadline("i am learning to drive until march"), "Drive");
    }
    #[test]
    fn py_topics__return_target() {
        assert_eq!(topics::return_target("ok back to the boiler"), "the boiler");
        assert_eq!(topics::return_target("anyway, the laptop"), "the laptop");
        assert_eq!(topics::return_target("back to lisbon, whats good to see"), "lisbon");
        assert_eq!(topics::return_target("back to work tomorrow"), "");
        assert_eq!(topics::return_target("anyway my knee has been clicking up the stairs"), "");
        assert_eq!(topics::return_target("back to the promotion, would asking why hurt my chances"), "the promotion");
        assert_eq!(topics::return_target("as i was saying, rome"), "rome");
        assert_eq!(topics::return_target("anyway the"), "");
    }
    #[test]
    fn py_topics__gist() {
        assert_eq!(topics::gist("A birthday cake for my son, 8, this weekend."), "birthday cake");
        assert_eq!(topics::gist("Cook a turkey for Christmas."), "turkey");
        assert_eq!(topics::gist("the tap in the kitchen keeps dripping"), "tap kitchen keep dripping");
        assert_eq!(topics::gist("im trying to save 500 pounds for a trip to spain"), "trip spain");
        assert_eq!(topics::gist("we're getting married next year"), "getting married");
        assert_eq!(topics::gist("one more thing, my car failed its mot on a tyre"), "car failed mot tyre");
        assert_eq!(topics::gist("never mind, how do i boil an egg"), "boil egg");
        assert_eq!(topics::gist("my son's birthday is next saturday and i want to bake him a cake"), "cake");
        assert_eq!(topics::gist("im thinking of getting a dog"), "dog");
        assert_eq!(topics::gist("how do i get rid of slugs"), "slug");
        assert_eq!(topics::gist("im in the market for a sofa"), "sofa");
        assert_eq!(topics::gist("whats the weather like"), "weather");
        assert_eq!(topics::gist("i'm after a new phone, mine is four years old"), "new phone");
        assert_eq!(topics::gist("actually it's a funeral"), "funeral");
        assert_eq!(topics::gist("im trying to save 3000 for a trip to iceland next summer"), "trip iceland summer");
    }
    #[test]
    fn py_topics___room_limit() {
        assert_eq!(topics::_room_limit("my flat is small"), "my flat");
        assert_eq!(topics::_room_limit("the kitchen is really tiny"), "the kitchen");
        assert_eq!(topics::_room_limit("my kitchen is tiny and the oven is broken"), "");
        assert_eq!(topics::_room_limit("the garden is small"), "");
    }
    #[test]
    fn py_topics___option_range() {
        assert_eq!(topics::_option_range("6 minutes for a runny yolk, 7 for jammy, 9 to 10 for hard"), "6 to 10 minutes");
        assert_eq!(topics::_option_range("15 to 20 minutes for a chicken or a turkey, 10 to 20 for a joint of beef, 5 to 10 for a steak"), "");
        assert_eq!(topics::_option_range("boil it for 10 minutes"), "");
    }
    #[test]
    fn py_topics___in_grams() {
        assert_eq!(topics::_in_grams("2kg"), Some(2000.0));
        assert_eq!(topics::_in_grams("1.5lb"), Some(680.4));
        assert_eq!(topics::_in_grams("500 g"), Some(500.0));
        assert_eq!(topics::_in_grams("3 oz"), Some(85.0));
        assert_eq!(topics::_in_grams("two kilos"), None);
    }
    #[test]
    fn py_topics___rate_said() {
        assert_eq!(topics::_rate_said("Roast it at 190C. About 20 minutes per 500g plus 20 minutes, so a 2kg bird takes roughly 1 hour 40."), "20 minutes per 500g plus 20 minutes");
        assert_eq!(topics::_rate_said("Nothing here."), "");
    }
    #[test]
    fn py_topics___the_period_in() {
        assert_eq!(topics::_the_period_in("and in october"), "october");
        assert_eq!(topics::_the_period_in("i may go to spain"), "");
        assert_eq!(topics::_the_period_in("what about may"), "may");
        assert_eq!(topics::_the_period_in("in may"), "may");
        assert_eq!(topics::_the_period_in("march on"), "march");
        assert_eq!(topics::_the_period_in("the fall is nice"), "fall");
        assert_eq!(topics::_the_period_in("next summer"), "summer");
    }
    #[test]
    fn py_topics__elided_ask() {
        assert_eq!(topics::elided_ask("any games"), true);
        assert_eq!(topics::elided_ask("what about drinks"), true);
        assert_eq!(topics::elided_ask("any idea how long"), false);
        assert_eq!(topics::elided_ask("any games for a party of ten people please"), false);
        assert_eq!(topics::elided_ask("and drinks"), false);
        assert_eq!(topics::elided_ask("how about pudding"), true);
    }
    #[test]
    fn py_topics__adds_only_a_constraint() {
        assert_eq!(topics::adds_only_a_constraint("i only have ten minutes").as_deref(), Some("ten minutes"));
        assert_eq!(topics::adds_only_a_constraint("its a 2kg one").as_deref(), Some("2kg"));
        assert_eq!(topics::adds_only_a_constraint("im in a hurry").as_deref(), Some("in a hurry"));
        assert_eq!(topics::adds_only_a_constraint("i only have ten minutes and no oven").as_deref(), None);
    }
    #[test]
    fn py_topics__trouble() {
        assert_eq!(topics::trouble("my mum is out of hospital").as_deref(), Some("homeward"));
        assert_eq!(topics::trouble("my dad is out of hospital but he is still very weak").as_deref(), Some("ill"));
        assert_eq!(topics::trouble("my mum works at the hospital").as_deref(), None);
        assert_eq!(topics::trouble("she keeps forgetting where things are").as_deref(), Some("memory"));
        assert_eq!(topics::trouble("my four year old says the same story every night").as_deref(), None);
        assert_eq!(topics::trouble("me and my mum fell out").as_deref(), None);
        assert_eq!(topics::trouble("my dad fell and was fine").as_deref(), Some("falls"));
        assert_eq!(topics::trouble("he has dementia").as_deref(), Some("memory"));
        assert_eq!(topics::trouble("he was diagnosed with dementia recently").as_deref(), Some("known"));
        assert_eq!(topics::trouble("my baby is not sleeping").as_deref(), Some("sleep"));
        assert_eq!(topics::trouble("he keeps getting chest infections").as_deref(), Some("ill"));
        assert_eq!(topics::trouble("my grandad went to the hospital to visit his sister").as_deref(), Some("ill"));
    }
    #[test]
    fn py_topics___time_in_words() {
        assert_eq!(topics::_time_in_words(80.0), "1 hour 20");
        assert_eq!(topics::_time_in_words(50.0), "50 minutes");
        assert_eq!(topics::_time_in_words(62.5), "1 hour");
        assert_eq!(topics::_time_in_words(57.5), "1 hour");
        assert_eq!(topics::_time_in_words(120.0), "2 hours");
        assert_eq!(topics::_time_in_words(65.0), "1 hour 5");
    }
    #[test]
    fn py_topics__time_for_amount() {
        assert_eq!(topics::time_for_amount("About 20 minutes per 500g plus 20 minutes.", "how long for a 1.5kg one").as_deref(), Some("1 hour 20"));
        assert_eq!(topics::time_for_amount("About 20 minutes per 500g.", "how long for 700g").as_deref(), Some("30 minutes"));
        assert_eq!(topics::time_for_amount("About 45 minutes per kg.", "how long for 10kg").as_deref(), None);
        assert_eq!(topics::time_for_amount("20 minutes per pound.", "and for 3lb").as_deref(), None);
    }
    #[test]
    fn py_topics___shorten() {
        assert_eq!(topics::_shorten("you should roast it in the hot oven for about 40 minutes"), "40 minutes.");
        assert_eq!(topics::_shorten("one lesson a week is what makes the rest of it stick for good"), "one lesson a week is what makes the rest of it stick for good");
        assert_eq!(topics::_shorten("short one"), "short one");
    }
    #[test]
    fn py_topics___clauses_of() {
        assert_eq!(topics::_clauses_of("the pendulum swing (lean forward, let the arm hang, circle it slowly) a few times a day"), vec!["the pendulum swing (lean forward, let the arm hang, circle it slowly) a few times a day".to_string()]);
        assert_eq!(topics::_clauses_of("a week or two; then rest"), vec!["a week or two".to_string(), "then rest".to_string()]);
        assert_eq!(topics::_clauses_of("boil it - then drain: done"), vec!["boil it".to_string(), "then drain".to_string(), "done".to_string()]);
    }
    #[test]
    fn py_thread___past() {
        assert_eq!(thread::_past("say"), "said");
        assert_eq!(thread::_past("try"), "tried");
        assert_eq!(thread::_past("play"), "played");
        assert_eq!(thread::_past("bake"), "baked");
        assert_eq!(thread::_past("ask,"), "asked,");
        assert_eq!(thread::_past("go."), "went.");
    }
    #[test]
    fn py_thread___third() {
        assert_eq!(thread::_third("need"), "needs");
        assert_eq!(thread::_third("try"), "tries");
        assert_eq!(thread::_third("have"), "has");
        assert_eq!(thread::_third("fix"), "fixes");
        assert_eq!(thread::_third("go"), "goes");
        assert_eq!(thread::_third("play"), "plays");
        assert_eq!(thread::_third("can"), "can");
    }
    #[test]
    fn py_thread___first_clause() {
        assert_eq!(thread::_first_clause("a satay is made with nuts, and that is off the table for you. Pick another.", 13), "a satay is made with nuts");
        assert_eq!(thread::_first_clause("one; two", 13), "one");
        assert_eq!(thread::_first_clause("", 13), "");
    }
    #[test]
    fn py_thread___the_thing_they_had() {
        assert_eq!(thread::_the_thing_they_had("i had an interview today"), "interview");
        assert_eq!(thread::_the_thing_they_had("i had a dentist appointment today"), "dentist appointment");
        assert_eq!(thread::_the_thing_they_had("my first day at the new job was today"), "first day");
        assert_eq!(thread::_the_thing_they_had("i had a really bad day today"), "");
        assert_eq!(thread::_the_thing_they_had("i had my blood test done today"), "blood test");
        assert_eq!(thread::_the_thing_they_had("i had my hair cut today"), "");
        assert_eq!(thread::_the_thing_they_had("i had a go at my boss today"), "");
        assert_eq!(thread::_the_thing_they_had("i have an interview tomorrow"), "");
    }
    #[test]
    fn py_thread__only_the_arrival() {
        assert_eq!(thread::only_the_arrival("im back"), true);
        assert_eq!(thread::only_the_arrival("back from work!"), true);
        assert_eq!(thread::only_the_arrival("hi, we're back home"), true);
        assert_eq!(thread::only_the_arrival("my back hurts"), false);
    }
    #[test]
    fn py_thread__points_back() {
        assert_eq!(thread::points_back("same as before"), true);
        assert_eq!(thread::points_back("and the other one then"), true);
        assert_eq!(thread::points_back("the other flat"), false);
        assert_eq!(thread::points_back("like last time please"), true);
    }
    #[test]
    fn py_thread__points_at_a_word_alone() {
        assert_eq!(thread::points_at_a_word_alone("how do i do it"), true);
        assert_eq!(thread::points_at_a_word_alone("how do i fix a bike"), false);
        assert_eq!(thread::points_at_a_word_alone("how do i stop it"), true);
        assert_eq!(thread::points_at_a_word_alone("what about that"), false);
        assert_eq!(thread::points_at_a_word_alone("which one should i buy"), false);
        assert_eq!(thread::points_at_a_word_alone("how do i cook this chicken"), false);
    }
    #[test]
    fn py_named__takes_back() {
        assert_eq!(named::takes_back("forget the chicken"), true);
        assert_eq!(named::takes_back("take that back"), true);
        assert_eq!(named::takes_back("ok never mind"), true);
        assert_eq!(named::takes_back("drop the whole trip please"), true);
        assert_eq!(named::takes_back("forget"), true);
        assert_eq!(named::takes_back("i will forget about it all later on today"), false);
    }
    #[test]
    fn py_named__the_one_asked_about() {
        assert_eq!(named::the_one_asked_about("tell me about rex"), "rex");
        assert_eq!(named::the_one_asked_about("who is priya"), "priya");
        assert_eq!(named::the_one_asked_about("what about them"), "them");
        assert_eq!(named::the_one_asked_about("can you tell me again about the big red bike"), "");
        assert_eq!(named::the_one_asked_about("what is it"), "");
    }
    #[test]
    fn py_named__thing() {
        assert_eq!(named::thing("and a pudding"), "pudding");
        assert_eq!(named::thing("what about drinks"), "");
        assert_eq!(named::thing("and what about it"), "");
        assert_eq!(named::thing("some cake please"), "cake please");
        assert_eq!(named::thing("and how long"), "");
        assert_eq!(named::thing("two"), "two");
    }
    #[test]
    fn py_named___the_ing() {
        assert_eq!(named::_the_ing("make"), "making");
        assert_eq!(named::_the_ing("run"), "running");
        assert_eq!(named::_the_ing("swim"), "swimming");
        assert_eq!(named::_the_ing("play"), "playing");
        assert_eq!(named::_the_ing("read"), "reading");
        assert_eq!(named::_the_ing("knit"), "knitting");
    }
    #[test]
    fn py_deadline__the_deadline() {
        assert_eq!(deadline::the_deadline("by december"), "december");
        assert_eq!(deadline::the_deadline("and by the end of the year"), "the end of the year");
        assert_eq!(deadline::the_deadline("by the way"), "");
        assert_eq!(deadline::the_deadline("before christmas!"), "christmas");
        assert_eq!(deadline::the_deadline("by december i want to be in a flat"), "");
        assert_eq!(deadline::the_deadline("in two weeks"), "");
    }

    trait PyOut {
        fn out(self) -> serde_json::Value;
    }
    impl PyOut for String {
        fn out(self) -> serde_json::Value {
            serde_json::Value::String(self)
        }
    }
    impl PyOut for Option<String> {
        fn out(self) -> serde_json::Value {
            self.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null)
        }
    }
    impl PyOut for (String, i64) {
        fn out(self) -> serde_json::Value {
            serde_json::json!([self.0, self.1])
        }
    }
    impl PyOut for Vec<String> {
        fn out(self) -> serde_json::Value {
            serde_json::json!(self)
        }
    }
    impl PyOut for crate::data::Value {
        fn out(self) -> serde_json::Value {
            self.to_plain()
        }
    }
    fn mem(j: &str) -> Memory {
        Memory::from_plain(serde_json::from_str::<serde_json::Value>(j).unwrap().as_object().unwrap())
    }
    #[test]
    fn py_memory_case_0() {
        assert_eq!(serde_json::to_value(PyOut::out(thread::checking_the_verdict("is that alright", &mem(r#"{"turns": [{"user": "can i have a satay", "bot": "No: a satay is made with nuts, and that is off the table for you. Try a korma.", "module": "answers"}]}"#)))).unwrap(), serde_json::json!("Afraid not — a satay is made with nuts."));
    }
    #[test]
    fn py_memory_case_1() {
        assert_eq!(serde_json::to_value(PyOut::out(thread::checking_the_verdict("are you sure", &mem(r#"{"turns": [{"user": "x", "bot": "Afraid not — a satay is made with nuts.", "module": "chat"}]}"#)))).unwrap(), serde_json::json!("Still a no — a satay is made with nuts."));
    }
    #[test]
    fn py_memory_case_2() {
        assert_eq!(serde_json::to_value(PyOut::out(thread::checking_the_verdict("will that work", &mem(r#"{"turns": [{"user": "x", "bot": "Yes: the cake has no nuts in it, so it is fine for you.", "module": "answers"}]}"#)))).unwrap(), serde_json::json!("Yes — the cake has no nuts in it."));
    }
    #[test]
    fn py_memory_case_3() {
        assert_eq!(serde_json::to_value(PyOut::out(thread::checking_a_fact("is that right", &mem(r#"{"turns": [{"user": "That's right. It keeps 1 to 2 days.", "bot": "That's right. It keeps 1 to 2 days in the fridge.", "module": "chat"}]}"#)))).unwrap(), serde_json::json!("Yes — that's the usual guidance."));
    }
    #[test]
    fn py_memory_case_4() {
        assert_eq!(serde_json::to_value(PyOut::out(deadline::follow_up("by december", &mem(r#"{"turns": [{"user": "we want to save 3000 pounds", "bot": "Good plan.", "module": "answers"}]}"#)))).unwrap(), serde_json::json!("we want to save 3000 pounds by december"));
    }
    #[test]
    fn py_memory_case_5() {
        assert_eq!(serde_json::to_value(PyOut::out(deadline::follow_up("by december", &mem(r#"{"turns": [{"user": "my exam is in june", "bot": "Good luck.", "module": "answers"}]}"#)))).unwrap(), serde_json::json!(""));
    }
    #[test]
    fn py_memory_case_6() {
        assert_eq!(serde_json::to_value(PyOut::out(deadline::follow_up("by december", &mem(r#"{"turns": [{"user": "we want to save 3000 pounds", "bot": "Good plan.", "module": "answers"}, {"user": "we are on about 200 a month", "bot": "ok", "module": "chat"}]}"#)))).unwrap(), serde_json::json!("we are on about 200 a month by december"));
    }
    #[test]
    fn py_memory_case_7() {
        assert_eq!(serde_json::to_value(PyOut::out(named::the_name_given("its biscuit", &mem(r#"{"turns": [{"user": "we got a puppy", "bot": "Lovely! What's it called?", "module": "chat"}]}"#)))).unwrap(), serde_json::json!("biscuit"));
    }
    #[test]
    fn py_memory_case_8() {
        assert_eq!(serde_json::to_value(PyOut::out(named::the_name_given("no idea yet", &mem(r#"{"turns": [{"user": "we got a puppy", "bot": "Lovely! What's it called?", "module": "chat"}]}"#)))).unwrap(), serde_json::json!(""));
    }
    #[test]
    fn py_memory_case_9() {
        assert_eq!(serde_json::to_value(PyOut::out(named::the_name_given("she is called Rosie", &mem(r#"{"turns": [{"user": "we got a puppy", "bot": "What is her name?", "module": "chat"}]}"#)))).unwrap(), serde_json::json!("Rosie"));
    }
    #[test]
    fn py_memory_case_10() {
        assert_eq!(serde_json::to_value(PyOut::out(named::a_measure_asked_again("and a new one fitted?", &mem(r#"{"turns": [{"user": "my boiler is broken", "bot": "Sorry.", "module": "chat"}, {"user": "how long will it take to fix", "bot": "A day.", "module": "answers"}]}"#)))).unwrap(), serde_json::json!("how long does it take to fit a new boiler"));
    }
    #[test]
    fn py_memory_case_11() {
        assert_eq!(serde_json::to_value(PyOut::out(named::_the_job_before(&mem(r#"{"turns": [{"user": "im painting the fence this weekend", "bot": "Nice.", "module": "chat"}]}"#)))).unwrap(), serde_json::json!("paint"));
    }
    #[test]
    fn py_memory_case_12() {
        assert_eq!(serde_json::to_value(PyOut::out(named::_the_opening_place_at(&mem(r#"{"turns": [{"user": "tell me about lisbon", "bot": "In Lisbon: trams and tiles.", "module": "answers"}, {"user": "thanks", "bot": "In the meantime, enjoy.", "module": "chat"}]}"#)))).unwrap(), serde_json::json!(["Lisbon", 0]));
    }
    #[test]
    fn py_memory_case_13() {
        assert_eq!(serde_json::to_value(PyOut::out(topics::introduction_question("will they get on?", &mem(r#"{"user": {"pets": [{"animal": "cat"}, {"animal": "kitten"}], "plan": "getting a kitten next week"}}"#)))).unwrap(), serde_json::json!("how do i introduce a new kitten to my older cat"));
    }
    #[test]
    fn py_memory_case_14() {
        assert_eq!(serde_json::to_value(PyOut::out(topics::introduction_question("will they get on?", &mem(r#"{"user": {"pets": [{"animal": "rabbit"}, {"animal": "kitten"}], "plan": "getting a kitten"}}"#)))).unwrap(), serde_json::json!(""));
    }
    #[test]
    fn py_memory_case_15() {
        assert_eq!(topics::in_a_safety_thread(&mem(r#"{"turns": [{"user": "fire", "bot": "Ring 999.", "module": "safety"}, {"user": "ok", "bot": "Stay low.", "module": "carers_turn"}, {"user": "hi", "bot": "hello", "module": "greeting"}]}"#)), true);
    }
    #[test]
    fn py_memory_case_16() {
        assert_eq!(topics::in_a_safety_thread(&mem(r#"{"turns": [{"user": "fire", "bot": "Ring 999.", "module": "safety"}, {"user": "what now", "bot": "x", "module": "answers"}]}"#)), false);
    }
    #[test]
    fn py_memory_case_17() {
        assert_eq!(serde_json::to_value(PyOut::out(topics::context(&mem(r#"{"topics": [{"text": "a"}, {"text": ""}, {"text": "c"}]}"#), 0))).unwrap(), serde_json::json!(["a", "c"]));
    }
    #[test]
    fn py_memory_case_18() {
        assert_eq!(serde_json::to_value(PyOut::out(topics::context(&mem(r#"{"topics": [{"text": "a"}, {"text": "b"}, {"text": "c"}]}"#), -1))).unwrap(), serde_json::json!(["b", "c"]));
    }
    #[test]
    fn py_memory_case_19() {
        assert_eq!(topics::already_said(&mem(r#"{"turns": [{"user": "a", "bot": "Hello There.", "module": "chat"}]}"#), Some("hello  there."), 0), true);
    }
    #[test]
    fn py_memory_case_20() {
        assert_eq!(serde_json::to_value(PyOut::out(topics::answer_in_hand(&mem(r#"{"turns": [{"user": "how do i make ragu", "bot": "Brown the mince.", "module": "answers"}, {"user": "ok thanks", "bot": "Glad that helped.", "module": "chat"}, {"user": "cheers", "bot": "Any time.", "module": "chat"}]}"#)))).unwrap(), serde_json::json!({"user": "how do i make ragu", "bot": "Brown the mince.", "module": "answers"}));
    }

}
