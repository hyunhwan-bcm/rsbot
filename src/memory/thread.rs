//! Port of `bot/memory/thread.py`: the thread of the conversation - what we are on (`about`),
//! and the user's earlier lines said again inside a sentence (`restate`).
//!
//! A reader that takes Python's `skip` (a function telling a line about the chat itself) has a
//! `_with` form taking a Rust closure; the contract form takes the function as a `Value`
//! (`Value::Fn`, resolved by its Python name, or `Value::None`).

use crate::data::Value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::{Memory, Turn};
use crate::understanding::t;
use crate::understood::{Entity, Understood};
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

const M: &str = "bot.memory.thread.";

static NEGATIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NEGATIVE"));
static POSITIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "POSITIVE"));
static VERB_FORMS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "VERB_FORMS"));
static IRREGULAR_PARTICIPLES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "IRREGULAR_PARTICIPLES"));
static PLACE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLACE_WORDS"));
static THIN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THIN"));
static SIGNS_OFF: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SIGNS_OFF"));
static WEAK: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WEAK"));
static NON_NOUN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NON_NOUN"));
static LEAD: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "LEAD"));
static MODAL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MODAL"));
static AUX: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AUX"));
static WH: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WH"));
static SUBJECTISH: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUBJECTISH"));
static FLIP: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "FLIP"));
static PLACE_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLACE_NAMES"));
static CALENDAR: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CALENDAR"));
static CONTRACTIONS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CONTRACTIONS"));
static _ONLY_THE_ARRIVAL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_ONLY_THE_ARRIVAL"));
static RECEIPT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RECEIPT_WORDS"));
static RECEIPT_LONGEST: LazyLock<i64> = LazyLock::new(|| t::int(M, "RECEIPT_LONGEST"));
static POINTS_BACK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "POINTS_BACK"));
static POINTING_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "POINTING_WORD"));
static ASK_WORD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_WORD"));
static ASK_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_VERB"));
static A_CHOICE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_CHOICE"));
static WHAT_I_SAID: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_I_SAID"));
static VERB_BEFORE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "VERB_BEFORE"));
static DETERMINERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DETERMINERS"));
static BARE_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BARE_VERBS"));
static NUMBER_WORD_HEADS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NUMBER_WORD_HEADS"));
static ASK_WORDS: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| t::map_strs(M, "ASK_WORDS"));
static ADDRESSEE: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "ADDRESSEE"));
static SUBJECT_YOU: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "SUBJECT_YOU"));
static DROPPED_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DROPPED_SUBJECT"));
static SECOND_PERSON: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| t::map_strs(M, "SECOND_PERSON"));
static FIRST_PERSON_AUX: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "FIRST_PERSON_AUX"));
static EXPAND: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "EXPAND"));
static PREDICATE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PREDICATE"));
static PREP: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PREP"));
static MEASURE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MEASURE"));
static ADVERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ADVERB"));
static POSSESSIVE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "POSSESSIVE"));
static IS_NOT_POSSESSIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "IS_NOT_POSSESSIVE"));
static NUMBER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NUMBER"));
static PAST: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "PAST"));
static THIRD: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "THIRD"));
static NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_THING"));
static ON_THE_TABLE_TURNS: LazyLock<i64> = LazyLock::new(|| t::int(M, "ON_THE_TABLE_TURNS"));
static ON_THE_TABLE_MISSES: LazyLock<i64> = LazyLock::new(|| t::int(M, "ON_THE_TABLE_MISSES"));
static COPULA: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COPULA"));
static STATE_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STATE_FILLER"));
static VERB_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "VERB_WORDS"));
static DETAIL_OPENER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DETAIL_OPENER"));
static CHECK_VERDICT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CHECK_VERDICT"));
static VERDICT_SAID: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "VERDICT_SAID"));
static NO_FRAMES: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "NO_FRAMES"));
static YES_FRAMES: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "YES_FRAMES"));
static NO_USED: LazyLock<Vec<(String, usize)>> = LazyLock::new(|| _used("NO_USED"));
static YES_USED: LazyLock<Vec<(String, usize)>> = LazyLock::new(|| _used("YES_USED"));
static CHECK_A_FACT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CHECK_A_FACT"));
static NO_FACT_SAID: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NO_FACT_SAID"));
static FACT_FRAMES: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "FACT_FRAMES"));
static NOT_A_FACT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_A_FACT"));
static WENT_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WENT_ASK"));
static NOT_THE_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_THE_THING"));
static NEVER_A_THING_THAT_WENT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NEVER_A_THING_THAT_WENT"));
static _A_STATE_ITSELF: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_STATE_ITSELF"));
static _A_NAME_FOR_IT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_NAME_FOR_IT"));
static _A_DESCRIPTION: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_DESCRIPTION"));
static HAPPENED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HAPPENED"));
static HAD_IT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HAD_IT"));
static IT_WAS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "IT_WAS"));
static THING_END: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THING_END"));
static _A_THING_DONE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_THING_DONE"));
static THING_PARTICLE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THING_PARTICLE"));
static DONE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DONE_WORDS"));
static WENT_FRAMES: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "WENT_FRAMES"));
static WENT_AGAIN: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "WENT_AGAIN"));
static A_VERDICT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_VERDICT"));
static _WENT_WELL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_WENT_WELL"));
static WENT_TOLD_WELL: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "WENT_TOLD_WELL"));
static WENT_TOLD_BAD: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "WENT_TOLD_BAD"));
static GREET: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.memory.topics.", "GREET"));
static ACKS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.memory.topics.", "ACKS"));

static ARRIVAL_PUNCT: LazyLock<Re> = LazyLock::new(|| Re::new(r"[.,!?]+", 0));
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]+", 0));
static FIRST_CLAUSE_CUT: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s|;\s|,\s+(?:and|so|but)\s", 0));
static HOW_IT_WENT: LazyLock<Re> = LazyLock::new(|| Re::new(r"how (?:did|was) (?:it|that|the|your)\b.*\bgo\b", 0));

fn _used(name: &str) -> Vec<(String, usize)> {
    t::value(M, name).items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].as_int().unwrap() as usize)).collect()
}

/// `word.strip(".,!?;:")`.
fn bare(word: &str) -> &str {
    text::strip_chars(word, ".,!?;:")
}

/// `word[len(core):]`: what follows the stripped core, counted in characters as Python does.
fn tail_of(word: &str, core: &str) -> String {
    text::slice(word, text::len(core) as isize, None)
}

/// `understand(text, {})`, kept per line: the reading of a line on its own is the same every
/// time it is asked for, and the thread readers ask for the same lines over and over.
fn read(text_: &str) -> Understood {
    static CACHE: LazyLock<Mutex<HashMap<String, Understood>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    if let Some(r) = CACHE.lock().unwrap().get(text_) {
        return r.clone();
    }
    let r = crate::understanding::understand(text_, None);
    let mut cache = CACHE.lock().unwrap();
    if cache.len() > 4096 {
        cache.clear();
    }
    cache.insert(text_.to_string(), r.clone());
    r
}

/// The words of a reading with `strip(chars)` applied, lower-cased.
fn reading_words(reading: &Understood, chars: &str) -> Vec<String> {
    reading.words.iter().map(|w| text::strip_chars(w, chars).to_lowercase()).collect()
}

/// Python's `skip(text, memory)` as a Rust function, from the `Value` a contract caller passes:
/// the store's `_about_this_chat` or the recap's `_not_a_recap`, by Python name.
pub fn skip_of(skip: &Value) -> Option<fn(&str, &Memory) -> bool> {
    let f = skip.as_fn()?;
    match f.name() {
        "bot.memory.store_replies:_about_this_chat" => Some(|t_, m| {
            // the store's reader may write memory; a reader here only reads, as the copy shows
            let mut copy = m.clone();
            crate::memory::store_replies::_about_this_chat(t_, &mut copy)
        }),
        "bot.modules.recap:_not_a_recap" => Some(|t_, m| crate::modules::recap::_not_a_recap(t_, m).unwrap_or(false)),
        other => panic!("thread: no Rust reader for skip function {other}"),
    }
}

/// A skip reader as the `_with` functions take it.
pub type Skip<'a> = Option<&'a dyn Fn(&str, &Memory) -> bool>;

/// `_asks_about_the_bot(reading)`: the line asks about the bot itself.
pub fn _asks_about_the_bot(reading: &Understood) -> bool {
    reading.about_bot
}

/// `_asks_about_the_chat(text, memory, skip)`: the store says this line is about the
/// conversation, not about the world.
pub fn _asks_about_the_chat(text_: &str, memory: &Memory, skip: &Value) -> bool {
    match skip_of(skip) {
        Some(f) => f(text_, memory),
        None => false,
    }
}

fn _asks_about_the_chat_with(text_: &str, memory: &Memory, skip: Skip) -> bool {
    skip.is_some_and(|f| f(text_, memory))
}

/// `_turns(memory)`: the turns.
pub fn _turns(memory: &Memory) -> &[Turn] {
    &memory.turns
}

/// `earlier_turn(memory)`: there is an earlier turn for a reply to point back at.
pub fn earlier_turn(memory: &Memory) -> bool {
    _turns(memory).iter().any(|turn| !turn.user.is_empty())
}

/// `only_the_arrival(text)`: the line says nothing but that the person is back.
pub fn only_the_arrival(text_: &str) -> bool {
    let said = text::strip(text_).to_lowercase().replace('\'', "");
    let said = ARRIVAL_PUNCT.sub(" ", &said);
    _ONLY_THE_ARRIVAL.match_(&text::squash(&said)).is_some()
}

/// `user_lines(memory)`: the user's earlier lines, oldest first, with their readings.
pub fn _user_lines(memory: &Memory) -> Vec<(String, Understood)> {
    _turns(memory)
        .iter()
        .filter(|turn| !text::strip(&turn.user).is_empty())
        .map(|turn| (turn.user.clone(), read(&turn.user)))
        .collect()
}

/// `user_lines(memory)` as Python's list of `(text, reading)` pairs (the reading as its plain
/// dict).
pub fn user_lines(memory: &Memory) -> Value {
    let pairs: Vec<Value> = _user_lines(memory)
        .into_iter()
        .map(|(said, reading)| {
            let plain = Value::from_plain(&serde_json::to_value(&reading).unwrap());
            Value::Tuple(std::sync::Arc::new(vec![Value::from(said), plain]))
        })
        .collect();
    Value::from(pairs)
}

/// `_is_a_receipt(reading)`: the line only takes the reply in and says nothing else.
pub fn _is_a_receipt(reading: &Understood) -> bool {
    let found: Vec<String> = reading
        .words
        .iter()
        .map(|w| text::strip_chars(w, ".,!?-\u{2014}\u{2013}()").to_string())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect();
    !found.is_empty() && found.len() as i64 <= *RECEIPT_LONGEST && found.iter().all(|w| RECEIPT_WORDS.contains(w))
}

/// `_is_thin(reading)`.
pub fn _is_thin(reading: &Understood) -> bool {
    let found: Vec<&str> = reading.words.iter().map(|w| text::strip_chars(w, ".,!?")).collect();
    if found.is_empty() {
        return true;
    }
    if _is_a_receipt(reading) {
        return true;
    }
    found.len() <= 2 && found.iter().all(|w| THIN.contains(*w))
}

/// `signs_off(text)`: a line that is only a closing ("night", "im off to bed").
pub fn signs_off(text_: &str) -> bool {
    let reading = read(text_);
    let found: Vec<&str> = reading.words.iter().map(|w| text::strip_chars(w, ".,!?")).collect();
    if found.is_empty() {
        return false;
    }
    if reading.intent == "farewell" {
        return true;
    }
    SIGNS_OFF.match_(&found.join(" ")).is_some()
}

/// `carries_subject(text)`: the line can be the thing the chat was on.
pub fn carries_subject(text_: &str) -> bool {
    let said = restate(text_);
    !said.is_empty() && _is_subject_phrase(&said)
}

/// `thin(text)`: a line that carries no subject ("hi", "ok", "lol").
pub fn thin(text_: &str) -> bool {
    let reading = read(text_);
    _is_thin(&reading) || _is_memory_line(&reading)
}

/// `points_back(text)`: a line that names nothing of its own and points at what was said before.
pub fn points_back(text_: &str) -> bool {
    let low = text::squash(&text_.to_lowercase().replace('\'', ""));
    let low = text::strip_chars(&low, " .!?,");
    !low.is_empty() && text::split(low).len() <= 6 && POINTS_BACK.match_(low).is_some()
}

/// `points_at_a_word_alone(text)`: an ask whose only subject is a pointing word.
pub fn points_at_a_word_alone(text_: &str) -> bool {
    let low = text::squash(&text_.to_lowercase().replace('\'', ""));
    let low = text::strip_chars(&low, " .!?,");
    let found = LETTERS.findall_str(low);
    if found.is_empty() || !POINTING_WORD.is_match(low) {
        return false;
    }
    if A_CHOICE.is_match(low) {
        return false;
    }
    let other: Vec<&String> = found.iter().filter(|w| !ASK_WORD.contains(*w) && !ASK_VERB.contains(*w)).collect();
    if other.is_empty() {
        return found.iter().any(|w| ASK_VERB.contains(w));
    }
    if other.len() > 1 {
        return false;
    }
    let at = found.iter().position(|w| w == other[0]).unwrap();
    at + 1 < found.len() && POINTING_WORD.fullmatch(&found[at + 1]).is_some()
}

/// `asks_what_i_said(text)`: an ask about what was said or felt in this chat.
pub fn asks_what_i_said(text_: &str) -> bool {
    WHAT_I_SAID.is_match(text_)
}

/// `nothing_said(memory)`: no earlier line of theirs has carried a subject yet.
pub fn nothing_said(memory: &Memory) -> bool {
    !_turns(memory).iter().any(|turn| carries_subject(&turn.user))
}

/// `_is_memory_line(reading)`: a line about the conversation itself.
pub fn _is_memory_line(reading: &Understood) -> bool {
    matches!(reading.topic_own.as_deref(), Some("about-the-bot") | Some("memory"))
        || ["about_bot", "greeting"].contains(&reading.intent.as_str())
        || ["greeting", "memory"].contains(&reading.kind.as_str())
}

/// `_named_entities(reading)`: the entities worth naming.
pub fn _named_entities(reading: &Understood) -> Vec<Entity> {
    let mut out = Vec::new();
    if _is_memory_line(reading) {
        return out;
    }
    for entity in &reading.entities {
        let name = text::strip(&entity.text);
        let core = name.to_lowercase();
        if name.is_empty() || WEAK.contains(&core) || FLIP.contains_key(&core) || THIN.contains(&core) || text::len(&core) <= 2 {
            continue;
        }
        if CONTRACTIONS.contains(&text::split(&core)[0]) {
            continue;
        }
        if MEASURE.contains(&core) || name.chars().next().is_some_and(|c| crate::understanding::pyuni::is_digit(&c.to_string())) {
            continue;
        }
        out.push(entity.clone());
    }
    out
}

/// `_groups(entities, reading)`: entities that sit next to each other in the line, joined.
pub fn _groups(entities: &[Entity], reading: &Understood) -> Vec<Vec<Entity>> {
    let norm = &reading.norm_text;
    let mut groups: Vec<Vec<Entity>> = Vec::new();
    for entity in entities {
        let name = text::strip(&entity.text);
        if let Some(last) = groups.last_mut() {
            let joined = format!("{} {}", text::strip(&last[last.len() - 1].text), name);
            if norm.contains(&joined.to_lowercase()) {
                last.push(entity.clone());
                continue;
            }
        }
        groups.push(vec![entity.clone()]);
    }
    groups
}

/// `_is_the_verb(reading, word)`: `word` sits where the sentence puts its verb.
pub fn _is_the_verb(reading: &Understood, word: &str) -> bool {
    let found = reading_words(reading, ".,!?");
    let low = word.to_lowercase();
    for index in (0..found.len()).rev() {
        let token = &found[index];
        if *token != low && !token.starts_with(&low) {
            continue;
        }
        return index > 0 && VERB_BEFORE.contains(&found[index - 1]);
    }
    false
}

/// `_verb_after_a_subject(reading, word)`: the word is the verb of "my back hurts".
pub fn _verb_after_a_subject(reading: &Understood, word: &str) -> bool {
    if !word.ends_with('s') || word.ends_with("ss") || text::len(word) < 3 {
        return false;
    }
    let found = reading_words(reading, ".,!?");
    let low = word.to_lowercase();
    for index in (0..found.len()).rev() {
        let token = &found[index];
        if *token != low && !token.starts_with(&low) {
            continue;
        }
        if index < 2 {
            return false;
        }
        let owner = &found[index - 2];
        return DETERMINERS.contains(owner) || ["my", "our", "your", "his", "her", "their"].contains(&owner.as_str());
    }
    false
}

/// `_word_behind_prep(reading, word)`: the only place this word holds is behind a preposition.
pub fn _word_behind_prep(reading: &Understood, word: &str) -> bool {
    let found: Vec<String> = reading.words.iter().map(|w| w.to_lowercase()).collect();
    let low = word.to_lowercase();
    for index in (0..found.len()).rev() {
        if found[index] != low {
            continue;
        }
        let mut before = index as i64 - 1;
        while before >= 0 && DETERMINERS.contains(&found[before as usize]) {
            before -= 1;
        }
        if before >= 0 && PREP.contains(&found[before as usize]) {
            return true;
        }
    }
    false
}

/// `_behind_a_prep(reading, entity)`: the words just before this entity are a preposition and its
/// article.
pub fn _behind_a_prep(reading: &Understood, entity: &Entity) -> bool {
    let found = &reading.words;
    let at = entity.at;
    if at <= 0 || at as usize > found.len() {
        return false;
    }
    let at = at as usize;
    let mut before: Vec<String> = found[at.saturating_sub(2)..at].iter().map(|w| w.to_lowercase()).collect();
    while before.last().is_some_and(|w| DETERMINERS.contains(w)) {
        before.pop();
    }
    before.last().is_some_and(|w| PREP.contains(w))
}

/// `_the_phrase_around(reading, word)`: the phrase `word` sits in, as (determiner, word), or None.
pub fn _the_phrase_around(reading: &Understood, word: &str) -> Option<(String, String)> {
    let found = reading_words(reading, ".,!?");
    let low = word.to_lowercase();
    for (index, token) in found.iter().enumerate() {
        if *token != low && !token.starts_with(&low) {
            continue;
        }
        let before = if index > 0 { found[index - 1].as_str() } else { "" };
        if ["my", "our", "your"].contains(&before) {
            return Some(("your".to_string(), word.to_string()));
        }
        if DETERMINERS.contains(before) {
            return Some((before.to_string(), word.to_string()));
        }
        return None;
    }
    None
}

/// `_phrase_for(text, reading, named_only=False)`: the thing this line is about, as a phrase with
/// its article, or None.
pub fn _phrase_for(text_: &str, reading: &Understood, named_only: bool) -> Option<String> {
    let keywords: Vec<String> = reading.keywords.iter().filter(|w| !WEAK.contains(*w) && !THIN.contains(*w)).cloned().collect();
    let entities = _named_entities(reading);
    let mut chosen: Option<Vec<Entity>> = None;
    for kind in ["place", "person", "name", "thing"] {
        let of_kind: Vec<Entity> = entities.iter().filter(|e| e.kind == kind).cloned().collect();
        let mut groups = _groups(&of_kind, reading);
        if kind == "person" || kind == "thing" {
            groups.retain(|g| !_behind_a_prep(reading, &g[0]));
        }
        if let Some(last) = groups.pop() {
            chosen = Some(last);
            break;
        }
    }
    if let Some(chosen) = chosen {
        let mut name = chosen.iter().map(|e| text::strip(&e.text).to_string()).collect::<Vec<_>>().join(" ");
        if let Some(before) = _qualifier(reading, &name, &keywords) {
            name = format!("{before} {name}");
        }
        name = _same_spelling(text_, &name);
        let kind = chosen[0].kind.as_str();
        if name.to_lowercase().starts_with("my ") {
            name = format!("your {}", text::slice(&name, 3, None));
        }
        if PLACE_NAMES.contains(kind) || kind == "name" || name.chars().next().is_some_and(|c| text::is_upper(&c.to_string())) {
            return Some(_place_case(&name));
        }
        let low = name.to_lowercase();
        if ["your ", "a ", "an ", "the "].iter().any(|p| low.starts_with(p)) {
            return Some(name);
        }
        return Some(format!("the {name}"));
    }
    let quality = |w: &String| POSITIVE.contains(w) || NEGATIVE.contains(w);
    let mut nouns: Vec<String> = keywords
        .iter()
        .filter(|w| {
            !(w.ends_with("ing") || w.ends_with("ed"))
                && !NON_NOUN.contains(*w)
                && !quality(w)
                && !CALENDAR.contains(*w)
                && !_is_the_verb(reading, w)
                && !_verb_after_a_subject(reading, w)
        })
        .cloned()
        .collect();
    if nouns.is_empty() {
        return None;
    }
    let things: Vec<String> = nouns.iter().filter(|w| !VERB_FORMS.contains(*w)).cloned().collect();
    if !things.is_empty() {
        nouns = things;
    }
    let kept: Vec<String> = nouns.iter().filter(|w| PLACE_WORDS.contains(*w) || !_word_behind_prep(reading, w)).cloned().collect();
    if !kept.is_empty() && kept.len() < nouns.len() {
        nouns = kept;
    }
    let last = nouns.last().unwrap().clone();
    if PLACE_WORDS.contains(&last) {
        return Some(_place_case(&_same_spelling(text_, &last)));
    }
    let around = _the_phrase_around(reading, &last);
    if named_only && around.is_none() {
        return None;
    }
    if around.is_some_and(|a| a.0 == "your") {
        return Some(format!("your {last}"));
    }
    Some(format!("the {last}"))
}

/// `_place_case(name)`: a place keeps its capital however it was typed.
pub fn _place_case(name: &str) -> String {
    let parts = text::split(name);
    if parts.is_empty() {
        return name.to_string();
    }
    if PLACE_WORDS.contains(&name.to_lowercase()) || parts.iter().all(|w| PLACE_WORDS.contains(&w.to_lowercase())) {
        return parts.iter().map(|w| _capital(w)).collect::<Vec<_>>().join(" ");
    }
    name.to_string()
}

/// `_same_spelling(text, phrase)`: `phrase` as it was typed.
pub fn _same_spelling(text_: &str, phrase: &str) -> String {
    let index = text::find(&text_.to_lowercase(), &phrase.to_lowercase());
    if index >= 0 {
        return text::slice(text_, index, Some(index + text::len(phrase) as isize));
    }
    phrase.to_string()
}

/// `_qualifier(reading, phrase, keywords)`: the word that belongs to `phrase` ("birthday cake").
pub fn _qualifier(reading: &Understood, phrase: &str, keywords: &[String]) -> Option<String> {
    let norm = &reading.norm_text;
    let index = norm.find(&phrase.to_lowercase())?;
    if index == 0 {
        return None;
    }
    let found = text::split(&norm[..index]);
    if found.len() < 2 {
        return None;
    }
    let before = text::strip_chars(&found[found.len() - 1], ".,!?").to_string();
    if !keywords.contains(&before) || !DETERMINERS.contains(text::strip_chars(&found[found.len() - 2], ".,!?")) {
        return None;
    }
    Some(before)
}

/// `about(memory, skip=None)`: the thing the conversation is on, as a phrase, or None.
pub fn about(memory: &Memory, skip: &Value) -> Value {
    let f = skip_of(skip);
    about_with(memory, f.as_ref().map(|f| f as &dyn Fn(&str, &Memory) -> bool)).map(Value::from).unwrap_or(Value::None)
}

/// `about(memory, skip)` with the skip reader as a closure.
pub fn about_with(memory: &Memory, skip: Skip) -> Option<String> {
    let lines: Vec<(String, Understood)> = _user_lines(memory)
        .into_iter()
        .filter(|(said, reading)| {
            !_is_thin(reading)
                && !_is_memory_line(reading)
                && !_asks_about_the_chat_with(said, memory, skip)
                && !_asks_about_the_bot(reading)
                && !signs_off(said)
        })
        .collect();
    for pass_kind in ["entity", "statement", "any"] {
        for (said, reading) in lines.iter().rev() {
            let names_something = !_named_entities(reading).is_empty();
            if pass_kind == "entity" && !names_something {
                continue;
            }
            if pass_kind == "statement" && reading.kind == "question" && !names_something {
                continue;
            }
            if pass_kind != "entity" && (reading.intent == "feeling" || reading.topic_own.as_deref() == Some("feelings")) {
                continue;
            }
            if let Some(phrase) = _phrase_for(said, reading, pass_kind == "any") {
                if _is_subject_phrase(&phrase) {
                    return Some(phrase);
                }
            }
        }
    }
    _from_topics(memory)
}

/// `_is_subject_phrase(phrase)`: the phrase can stand after "we were on".
pub fn _is_subject_phrase(phrase: &str) -> bool {
    let mut found: Vec<&str> = text::split_ref(phrase).into_iter().map(|w| text::strip_chars(w, ",.;:")).collect();
    if found.len() > 1 && DETERMINERS.contains(&found[0].to_lowercase()) {
        found.remove(0);
    }
    let head = found.first().map(|w| w.to_lowercase()).unwrap_or_default();
    if head.is_empty() {
        return false;
    }
    if crate::understanding::pyuni::is_digit(&head) || NUMBER_WORD_HEADS.contains(&head) {
        return false;
    }
    !(GREET.contains(&head) || ACKS.contains(&head) || BARE_VERBS.contains(&head))
}

/// `_from_topics(memory)`: what the memory store calls the subject, with the article a sentence
/// needs.
pub fn _from_topics(memory: &Memory) -> Option<String> {
    let phrase = text::squash(&crate::memory::topics::subject(memory));
    if phrase.is_empty() || !_is_subject_phrase(&phrase) {
        return None;
    }
    let first = text::split(&phrase)[0].clone();
    if DETERMINERS.contains(&first.to_lowercase()) || first.chars().next().is_some_and(|c| text::is_upper(&c.to_string())) {
        return Some(phrase);
    }
    Some(format!("the {phrase}"))
}

/// `_is_wh(word)`: the word is a question word, contraction and all.
pub fn _is_wh(word: &str) -> bool {
    let core = text::strip_chars(word, ".,!?").to_lowercase().replace('\'', "").replace('\u{2019}', "");
    if WH.contains(&core) {
        return true;
    }
    core.ends_with('s') && WH.contains(&core[..core.len() - 1])
}

/// `_with_wh(reading, words)`: the question word the reading kept out of the focus, put back.
pub fn _with_wh(reading: &Understood, words: &[String]) -> Vec<String> {
    if reading.kind != "question" || words.is_empty() {
        return words.to_vec();
    }
    if _is_wh(&words[0]) {
        return words.to_vec();
    }
    let held: HashSet<String> = words.iter().map(|w| bare(w).to_lowercase()).collect();
    let norm: Vec<String> = reading.norm.iter().map(|w| text::strip_chars(w, ".,!?").to_lowercase()).collect();
    for (index, word) in norm.iter().enumerate() {
        if !WH.contains(word) {
            continue;
        }
        let mut phrase: Vec<String> = Vec::new();
        for word2 in norm.iter().skip(index).take(3) {
            if !phrase.is_empty() && (AUX.contains(word2) || SUBJECTISH.contains(word2) || ["the", "a", "an"].contains(&word2.as_str())) {
                break;
            }
            phrase.push(word2.clone());
        }
        if !phrase.is_empty() && !held.contains(&phrase[0].to_lowercase()) {
            phrase.extend(words.iter().cloned());
            return phrase;
        }
        break;
    }
    for ask in &reading.asks {
        let phrase: Vec<String> = match ASK_WORDS.get(ask) {
            Some(p) if !p.is_empty() => p.clone(),
            _ if WH.contains(ask) => vec![ask.clone()],
            _ => Vec::new(),
        };
        if !phrase.is_empty() && !held.contains(&phrase[0].to_lowercase()) {
            let mut out = phrase;
            out.extend(words.iter().cloned());
            return out;
        }
    }
    words.to_vec()
}

/// `_strip_lead(words)`: drop the discourse markers that open a follow-up.
pub fn _strip_lead(words: &[String]) -> Vec<String> {
    let mut words = words.to_vec();
    let mut changed = true;
    while changed && !words.is_empty() {
        changed = false;
        for lead in LEAD.iter() {
            let parts = text::split(lead);
            if words.len() >= parts.len() && words[..parts.len()] == parts[..] {
                words = words[parts.len()..].to_vec();
                changed = true;
            }
        }
    }
    words
}

/// `_with_you(words)`: the subject a line that opens on an auxiliary left out.
pub fn _with_you(words: &[String]) -> Vec<String> {
    if words.is_empty() {
        return vec!["you".to_string()];
    }
    let first = text::strip_chars(&words[0], ".,!?;:").to_lowercase();
    if let Some(lead) = SECOND_PERSON.get(&first) {
        let mut out = lead.clone();
        out.extend(words[1..].iter().cloned());
        return out;
    }
    let mut out = vec!["you".to_string()];
    out.extend(words.iter().cloned());
    out
}

/// `_first_person_aux(word)`: "are" -> "am", "were" -> "was", with the punctuation kept.
pub fn _first_person_aux(word: &str) -> String {
    let core = bare(word);
    let tail = tail_of(word, core);
    let moved = FIRST_PERSON_AUX.get(&core.to_lowercase()).cloned().unwrap_or_else(|| core.to_string());
    format!("{moved}{tail}")
}

/// `_flip(words, addressee=False)`: first person the user used becomes second person.
pub fn _flip(words: &[String], addressee: bool) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (index, word) in words.iter().enumerate() {
        let core = bare(word);
        let tail = tail_of(word, core);
        if addressee && ADDRESSEE.contains_key(core) {
            let before = if index > 0 { bare(&words[index - 1]).to_lowercase() } else { String::new() };
            if core == "you" && (index == 0 || AUX.contains(&before) || WH.contains(&before)) {
                if let Some(last) = out.last_mut() {
                    *last = _first_person_aux(last);
                }
                out.push(format!("{}{}", *SUBJECT_YOU, tail));
            } else {
                out.push(format!("{}{}", ADDRESSEE[core], tail));
            }
            continue;
        }
        let flipped = FLIP.get(core).map(String::as_str).unwrap_or(core);
        out.push(format!("{flipped}{tail}"));
    }
    out
}

/// `your_words(text)`: the person's own phrase in the second person.
pub fn your_words(text_: &str) -> String {
    _flip(&text::split(text_), false).join(" ")
}

/// `_expand(text)`: the line with its contractions opened and no full stop at the end.
pub fn _expand(text_: &str) -> String {
    let found: Vec<String> = text::split(text_)
        .into_iter()
        .map(|w| EXPAND.get(&bare(&w).to_lowercase()).cloned().unwrap_or(w))
        .collect();
    let joined = found.join(" ");
    text::rstrip_chars(text::strip_chars(&joined, " ,;:"), ".").to_string()
}

/// `_as_text(value)`: the focus words as one string.
pub fn _as_text(value: &[String]) -> String {
    text::strip(&value.join(" ")).to_string()
}

/// `_subjects_at(words, index, names=())`: words[index] can be the subject right after an
/// auxiliary.
pub fn _subjects_at(words: &[String], index: i64, names: &[String]) -> bool {
    if index < 0 || index as usize >= words.len() {
        return false;
    }
    let core = text::strip_chars(&words[index as usize], ".,!?").to_lowercase();
    SUBJECTISH.contains(&core) || names.contains(&core)
}

/// `_np_end(words, start, most=3)`: where the subject after an auxiliary ends.
pub fn _np_end(words: &[String], start: i64, most: i64) -> i64 {
    let n = words.len() as i64;
    if start >= n {
        return start;
    }
    if !DETERMINERS.contains(&bare(&words[start as usize]).to_lowercase()) {
        return start + 1;
    }
    let mut end = start + 1;
    while end < n && end - start < most {
        let core = bare(&words[end as usize]).to_lowercase();
        if AUX.contains(&core) || WH.contains(&core) || PREDICATE.contains(&core) || PREP.contains(&core) || ADVERB.contains(&core) {
            break;
        }
        end += 1;
    }
    end.max(start + 1)
}

/// `_embed(words, names=())`: the question with its auxiliary behind the subject.
pub fn _embed(words: &[String], names: &[String]) -> Vec<String> {
    if words.len() < 2 {
        return words.to_vec();
    }
    for index in 0..words.len() - 1 {
        let aux = bare(&words[index]).to_lowercase();
        if !AUX.contains(&aux) {
            continue;
        }
        let start = index + 1;
        if !_subjects_at(words, start as i64, names) {
            continue;
        }
        let end = (_np_end(words, start as i64, 3) as usize).min(words.len());
        let mut subject: Vec<String> = words[start..end].to_vec();
        let mut tail: Vec<String> = words[end..].to_vec();
        if MODAL.contains(&aux) && index > 0 && subject.len() > 1 && tail.is_empty() {
            tail = vec![subject.pop().unwrap()];
        }
        let mut out: Vec<String> = words[..index].to_vec();
        if ["do", "does", "did"].contains(&aux.as_str()) {
            let mut verb = 0;
            while verb < tail.len() && ADVERB.contains(&bare(&tail[verb]).to_lowercase()) {
                verb += 1;
            }
            if verb < tail.len() {
                tail[verb] = _agree(&tail[verb], &aux);
            } else if subject.len() > 1 && {
                let last = bare(&subject[subject.len() - 1]).to_lowercase();
                !(DETERMINERS.contains(&last) || PREDICATE.contains(&last) || MEASURE.contains(&last))
            } {
                let last = subject.pop().unwrap();
                tail = vec![_agree(&last, &aux)];
            }
            out.extend(subject);
            out.extend(tail);
            return out;
        }
        out.extend(subject);
        out.push(words[index].clone());
        out.extend(tail);
        return out;
    }
    words.to_vec()
}

/// `_keep_possessives(clause, clean)`: put back the "'s" the reading drops.
pub fn _keep_possessives(clause: &str, clean: &str) -> String {
    let stems: HashSet<String> = POSSESSIVE.finditer(clean).iter().map(|m| m.str(1).to_lowercase()).collect();
    if stems.is_empty() {
        return clause.to_string();
    }
    let found = text::split(clause);
    let mut out = Vec::new();
    for (index, word) in found.iter().enumerate() {
        let core = bare(word);
        if index == found.len() - 1 {
            out.push(word.clone());
            continue;
        }
        let low = core.to_lowercase();
        if stems.contains(&low) && !core.contains('\'') && !core.contains('\u{2019}') && !IS_NOT_POSSESSIVE.contains(&low) {
            out.push(format!("{core}'s{}", tail_of(word, core)));
        } else {
            out.push(word.clone());
        }
    }
    out.join(" ")
}

/// `_truncated(clause, clean)`: the reading's focus stops before the line does.
pub fn _truncated(clause: &str, clean: &str) -> bool {
    if clause.is_empty() || clean.is_empty() {
        return false;
    }
    let focus_words = text::split(&clause.replace('?', "").to_lowercase());
    let clean_words = text::split(&clean.replace('?', "").to_lowercase());
    focus_words.len() < clean_words.len() && clean_words[..focus_words.len()] == focus_words[..]
}

/// `_numbers(text, clause)`: the numbers in `clause` as the person typed them.
pub fn _numbers(text_: &str, clause: &str) -> String {
    let mut typed: HashMap<String, String> = HashMap::new();
    for m in NUMBER.finditer(text_) {
        typed.insert(m.as_str().replace(',', ""), m.as_str().to_string());
    }
    if typed.is_empty() {
        return clause.to_string();
    }
    NUMBER.sub_fn(clause, |m| typed.get(m.as_str()).filter(|s| !s.is_empty()).cloned().unwrap_or_else(|| m.as_str().to_string()))
}

/// The contract form of `restate`: "" for Python's None.
pub fn restate(text_: &str) -> String {
    _restate(text_).unwrap_or_default()
}

/// `restate(text)`: the user's line said again inside a sentence, in their words, or None.
pub fn _restate(text_: &str) -> Option<String> {
    let reading = read(text_);
    let clean = text::strip(&reading.clean).to_string();
    let mut clause = _as_text(&reading.focus);
    if clause.is_empty() {
        clause = text::strip(reading.norm_text.rsplit(" | ").next().unwrap_or("")).to_string();
    }
    if _truncated(&clause, &clean) {
        clause = clean.clone();
    }
    if clause.is_empty() {
        clause = clean.clone();
    }
    let first_word = text::split(&clause).first().map(|w| bare(w).to_lowercase()).unwrap_or_default();
    let line_words: HashSet<String> = text::split(&clean).iter().map(|w| bare(w).to_lowercase()).collect();
    if !clean.is_empty() && (PREP.contains(&first_word) || (!first_word.is_empty() && !line_words.contains(&first_word))) {
        clause = _expand(&clean);
    }
    let mut words = _strip_lead(&text::split(&clause.replace('?', "")));
    if words.is_empty() {
        return None;
    }
    words = _with_wh(&reading, &words);
    let typed: Vec<String> = text::split(text_).iter().map(|w| bare(w).to_lowercase()).collect();
    if let Some(typed_aux) = typed.iter().find(|w| ["do", "does", "did"].contains(&w.as_str())) {
        for word in words.iter_mut() {
            let core = bare(word).to_lowercase();
            if ["do", "does", "did"].contains(&core.as_str()) && core != *typed_aux {
                let rest = tail_of(word, bare(word));
                *word = format!("{typed_aux}{rest}");
                break;
            }
        }
    }
    let first = text::strip_chars(&words[0], ".,!?").to_lowercase();
    let names: Vec<String> = reading.entities.iter().map(|e| text::strip(&e.text).to_lowercase()).collect();
    if AUX.contains(&first) || WH.contains(&first) || DROPPED_SUBJECT.contains(&first) {
        words = _flip(&words, true);
        if DROPPED_SUBJECT.contains(&first) && reading.kind != "question" {
            words = _with_you(&words);
            clause = _embed(&words, &names).join(" ");
        } else {
            let said = _embed(&words, &names).join(" ");
            clause = if AUX.contains(&first) { format!("whether {said}") } else { said };
        }
    } else {
        let to_bot = crate::memory::_to_the_bot(&words.join(" "));
        clause = crate::memory::_from_the_bot(&_flip(&text::split(&to_bot), false).join(" "));
    }
    clause = _conditional(&clause);
    clause = _keep_possessives(&clause, &clean);
    if reading.kind == "question" {
        let core: HashSet<String> = text::split(&clause).iter().map(|w| bare(w).to_lowercase()).collect();
        let opening = text::split(&clause).first().cloned().unwrap_or_default();
        if !core.iter().any(|w| AUX.contains(w) || WH.contains(w)) && !_is_wh(&opening) && !core.contains("whether") {
            return None;
        }
    }
    clause = _numbers(text_, &clause);
    clause = crate::memory::facts::no_tone(&clause);
    if clause.is_empty() {
        return None;
    }
    let out = _restore_case(text_, text::strip_chars(&clause, " ,;:"));
    if out.is_empty() { None } else { Some(out) }
}

/// `_past(word)`: the past tense of a verb.
pub fn _past(word: &str) -> String {
    let core = bare(word);
    let suffix = tail_of(word, core);
    if let Some(p) = PAST.get(core) {
        return format!("{p}{suffix}");
    }
    if core.is_empty() || !core.chars().last().is_some_and(char::is_alphabetic) {
        return word.to_string();
    }
    if core.ends_with('e') {
        return format!("{core}d{suffix}");
    }
    let c: Vec<char> = core.chars().collect();
    if c.len() > 2 && core.ends_with('y') && !"aeiou".contains(c[c.len() - 2]) {
        return format!("{}ied{suffix}", c[..c.len() - 1].iter().collect::<String>());
    }
    format!("{core}ed{suffix}")
}

/// `_agree(verb, aux)`: the verb form that goes with the subject once "do/does/did" is dropped.
pub fn _agree(verb: &str, aux: &str) -> String {
    match aux {
        "did" => _past(verb),
        "does" => _third(verb),
        _ => verb.to_string(),
    }
}

/// `_third(verb)`: a base verb in the third person.
pub fn _third(verb: &str) -> String {
    let core = bare(verb);
    let suffix = tail_of(verb, core);
    let low = core.to_lowercase();
    if core.is_empty() || !core.chars().last().is_some_and(char::is_alphabetic) {
        return verb.to_string();
    }
    if let Some(t_) = THIRD.get(&low) {
        return format!("{t_}{suffix}");
    }
    if AUX.contains(&low) {
        return verb.to_string();
    }
    if ["s", "sh", "ch", "x", "z", "o"].iter().any(|e| low.ends_with(e)) {
        return format!("{core}es{suffix}");
    }
    let c: Vec<char> = low.chars().collect();
    if c.len() > 2 && low.ends_with('y') && !"aeiou".contains(c[c.len() - 2]) {
        let cc: Vec<char> = core.chars().collect();
        return format!("{}ies{suffix}", cc[..cc.len() - 1].iter().collect::<String>());
    }
    format!("{core}s{suffix}")
}

/// `_conditional(clause)`: "if i stuff it" -> "what happens if you stuff it".
pub fn _conditional(clause: &str) -> String {
    if clause.starts_with("if ") { format!("what happens {clause}") } else { clause.to_string() }
}

/// `_restore_case(text, clause)`: put back the capitals the reading lowers.
pub fn _restore_case(text_: &str, clause: &str) -> String {
    let typed: Vec<String> = text::split(text_).iter().map(|w| bare(w).to_string()).collect();
    let capitals: HashSet<String> = typed
        .iter()
        .skip(1)
        .filter(|w| w.chars().next().is_some_and(|c| text::is_upper(&c.to_string())))
        .map(|w| w.to_lowercase())
        .collect();
    text::split(clause)
        .iter()
        .map(|word| {
            let core = bare(word);
            if capitals.contains(core) { format!("{}{}", _capital(core), tail_of(word, core)) } else { word.clone() }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `_capital(text)`: the first letter upper-cased.
pub fn _capital(text_: &str) -> String {
    text::upper_first(text_)
}

/// `on_the_table(memory, limit=3)`: the things the chat is holding, newest first.
pub fn on_the_table(memory: &Memory, limit: i64) -> Value {
    Value::from(_on_the_table(memory, limit))
}

fn _on_the_table(memory: &Memory, limit: i64) -> Vec<String> {
    let turns = _turns(memory);
    if turns.is_empty() {
        return Vec::new();
    }
    let start = turns.len().saturating_sub(*ON_THE_TABLE_TURNS as usize);
    let recent = &turns[start..];
    let mut out: Vec<String> = Vec::new();
    let mut misses = 0;
    for size in (1..=recent.len()).rev() {
        let mut shorter = memory.clone();
        shorter.turns = recent[..size].to_vec();
        let Some(phrase) = about_with(&shorter, None).filter(|p| !p.is_empty()) else { continue };
        let head = text::strip_chars(text::split(&phrase).last().map(String::as_str).unwrap_or(""), ".,!?").to_string();
        if NOT_A_THING.contains(&head) {
            continue;
        }
        if !_said_in_the_chat(&phrase, &recent[..size]) {
            continue;
        }
        if out.iter().any(|one| one.to_lowercase() == phrase.to_lowercase()) {
            misses += 1;
            if misses >= *ON_THE_TABLE_MISSES {
                break;
            }
            continue;
        }
        out.push(phrase);
        misses = 0;
        if out.len() as i64 >= limit {
            break;
        }
    }
    out
}

/// `_said_in_the_chat(phrase, turns)`: the phrase's words sit side by side in something the
/// person said.
pub fn _said_in_the_chat(phrase: &str, turns: &[Turn]) -> bool {
    let mut found = text::split(&phrase.to_lowercase());
    if found.is_empty() {
        return false;
    }
    if found[0] == "your" {
        found[0] = "my".to_string();
    }
    let head = found[found.len() - 1].clone();
    for turn in turns {
        let said = text::split(&turn.user.to_lowercase());
        let run = found.len();
        if said.len() >= run && (0..=said.len() - run).any(|start| said[start..start + run] == found[..]) {
            return true;
        }
        if VERB_WORDS.contains(&head) {
            continue;
        }
        for entity in &read(&turn.user).entities {
            if entity.kind != "thing" && entity.kind != "person" {
                continue;
            }
            if entity.text.to_lowercase() != head {
                continue;
            }
            if _said_as_a_state(&head, &turn.user) {
                continue;
            }
            return true;
        }
    }
    false
}

/// `_said_as_a_state(head, text)`: every time the chat said this word, it was after a copula.
pub fn _said_as_a_state(head: &str, text_: &str) -> bool {
    let said = text::split(&text_.to_lowercase().replace('\u{2019}', "'"));
    let clean = |w: &str| text::strip_chars(w, ".,!?;:\"'").to_string();
    let mut seen = false;
    for (i, word) in said.iter().enumerate() {
        if clean(word) != head {
            continue;
        }
        seen = true;
        let mut j = i as i64 - 1;
        while j >= 0 && STATE_FILLER.contains(&clean(&said[j as usize])) {
            j -= 1;
        }
        if j < 0 || !COPULA.contains(&clean(&said[j as usize])) {
            return false;
        }
    }
    seen
}

/// `in_a_list(phrases)`: the things as one clause ("your sister, the mince and your manager").
pub fn in_a_list(phrases: &Value) -> Value {
    let items: Vec<String> = phrases.items().iter().map(|p| text::strip(&value_text(p)).to_string()).filter(|p| !p.is_empty()).collect();
    Value::from(_in_a_list(&items))
}

fn _in_a_list(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        n => format!("{} and {}", items[..n - 1].join(", "), items[n - 1]),
    }
}

/// `str(x)` for a value in a list of phrases.
fn value_text(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        Value::None => "None".to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        other => format!("{other:?}"),
    }
}

/// `topic_index(memory, skip=None)`: where in `user_lines` the newest line that names the thing
/// we are on sits, or None.
pub fn topic_index(memory: &Memory, skip: &Value) -> Value {
    let f = skip_of(skip);
    topic_index_with(memory, f.as_ref().map(|f| f as &dyn Fn(&str, &Memory) -> bool)).map(Value::from).unwrap_or(Value::None)
}

/// `topic_index(memory, skip)` with the skip reader as a closure.
pub fn topic_index_with(memory: &Memory, skip: Skip) -> Option<usize> {
    let mut keep: Vec<(usize, String, Understood)> = Vec::new();
    for (index, (said, reading)) in _user_lines(memory).into_iter().enumerate() {
        if _is_thin(&reading) || _is_memory_line(&reading) {
            continue;
        }
        if _asks_about_the_chat_with(&said, memory, skip) {
            continue;
        }
        keep.push((index, said, reading));
    }
    for require_entity in [true, false] {
        for (index, said, reading) in keep.iter().rev() {
            if require_entity && _named_entities(reading).is_empty() {
                continue;
            }
            if !require_entity && (reading.intent == "feeling" || reading.topic_own.as_deref() == Some("feelings")) {
                continue;
            }
            if _phrase_for(said, reading, false).is_some_and(|p| !p.is_empty()) {
                return Some(*index);
            }
        }
    }
    None
}

/// `last_ask(memory, skip=None, not_before=None)`: the last thing the user asked, restated, or
/// None.
pub fn last_ask(memory: &Memory, skip: &Value, not_before: &Value) -> Value {
    let f = skip_of(skip);
    let not_before = not_before.as_int();
    last_ask_with(memory, f.as_ref().map(|f| f as &dyn Fn(&str, &Memory) -> bool), not_before).map(Value::from).unwrap_or(Value::None)
}

/// `last_ask(memory, skip, not_before)` with the skip reader as a closure.
pub fn last_ask_with(memory: &Memory, skip: Skip, not_before: Option<i64>) -> Option<String> {
    let lines = _user_lines(memory);
    for index in (0..lines.len()).rev() {
        if not_before.is_some_and(|n| (index as i64) < n) {
            break;
        }
        let (said, reading) = &lines[index];
        if _is_memory_line(reading) || _asks_about_the_chat_with(said, memory, skip) {
            continue;
        }
        if reading.kind == "question" || said.contains('?') {
            if let Some(out) = _restate(said) {
                return Some(out);
            }
        }
    }
    None
}

/// What another file's function returned, as Python's str-or-None (its contract type may still
/// be `()` while it is a stub).
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

/// `_only_a_detail(text)`: the whole line is a detail about something already on the table.
pub fn _only_a_detail(text_: &str) -> bool {
    let line = text::squash(text_);
    let line = text::strip_chars(&line, " .!?");
    let rest = DETAIL_OPENER.sub("", line);
    if rest == line {
        return false;
    }
    let Some(got) = crate::memory::detail::the_detail(line).py_str().filter(|g| !g.is_empty()) else { return false };
    let lower_words = |t_: &str| text::split(t_).iter().map(|w| w.to_lowercase()).collect::<Vec<_>>();
    lower_words(&got) == lower_words(&rest)
}

/// `last_said(memory, skip=None)`: the last substantial thing the user told the bot, restated,
/// or None.
pub fn last_said(memory: &Memory, skip: &Value) -> Value {
    let f = skip_of(skip);
    last_said_with(memory, f.as_ref().map(|f| f as &dyn Fn(&str, &Memory) -> bool)).map(Value::from).unwrap_or(Value::None)
}

/// `last_said(memory, skip)` with the skip reader as a closure.
pub fn last_said_with(memory: &Memory, skip: Skip) -> Option<String> {
    let (said, past_a_closing) = last_said_past_closings_with(memory, skip);
    if past_a_closing && said.is_some() && about_with(memory, skip).is_some_and(|a| !a.is_empty()) {
        return None;
    }
    said
}

/// `last_said_past_closings(memory, skip=None)`: the last thing said, plus whether a closing had
/// to be walked past to reach it (Python's tuple).
pub fn last_said_past_closings(memory: &Memory, skip: &Value) -> Value {
    let f = skip_of(skip);
    let (said, past) = last_said_past_closings_with(memory, f.as_ref().map(|f| f as &dyn Fn(&str, &Memory) -> bool));
    Value::Tuple(std::sync::Arc::new(vec![Value::from(said), Value::from(past)]))
}

/// `last_said_past_closings(memory, skip)` with the skip reader as a closure.
pub fn last_said_past_closings_with(memory: &Memory, skip: Skip) -> (Option<String>, bool) {
    let mut past_a_closing = false;
    for (said, reading) in _user_lines(memory).iter().rev() {
        if _is_thin(reading) || reading.kind == "question" || _is_memory_line(reading) {
            continue;
        }
        if signs_off(said) {
            past_a_closing = true;
            continue;
        }
        if _asks_about_the_chat_with(said, memory, skip) {
            continue;
        }
        if _only_a_detail(said) {
            continue;
        }
        if let Some(out) = _restate(said) {
            return (Some(out), past_a_closing);
        }
    }
    (None, past_a_closing)
}

/// `named(memory, limit=2)`: the things and places the user has named, newest first, each used
/// once.
pub fn named(memory: &Memory, limit: i64) -> Value {
    let mut out: Vec<String> = Vec::new();
    for (_said, reading) in _user_lines(memory).iter().rev() {
        for entity in &reading.entities {
            let mut name = text::strip(&entity.text).to_string();
            if entity.kind == "thing" || name.is_empty() || WEAK.contains(&name.to_lowercase()) {
                continue;
            }
            if FLIP.contains_key(&name.to_lowercase()) || text::len(text::strip_chars(&name, ".,")) <= 2 {
                continue;
            }
            if name.to_lowercase().starts_with("my ") {
                name = format!("your {}", text::slice(&name, 3, None));
            }
            if out.iter().any(|n| n.to_lowercase() == name.to_lowercase()) {
                continue;
            }
            out.push(name);
            if out.len() as i64 >= limit {
                return Value::from(out);
            }
        }
    }
    Value::from(out)
}

/// `checking_the_verdict(text, memory)`: the verdict said again, or None.
pub fn checking_the_verdict(text_: &str, memory: &Memory) -> Option<String> {
    let said = text::squash(&text_.to_lowercase());
    let said = text::strip_chars(&said, " .!?,");
    if said.is_empty() || text::split(said).len() > 9 || CHECK_VERDICT.match_(said).is_none() {
        return None;
    }
    let last = memory.turns.last().map(|t_| text::strip(&t_.bot).to_string()).unwrap_or_default();
    let verdict = VERDICT_SAID.match_(&last)?;
    let reason = _first_clause(verdict.str(2), 13);
    if reason.is_empty() {
        return None;
    }
    let frame = if verdict.str(1).to_lowercase() == "yes" {
        _next_frame(&YES_FRAMES, &YES_USED, &last)
    } else {
        _next_frame(&NO_FRAMES, &NO_USED, &last)
    };
    Some(frame.replace("{reason}", &reason))
}

/// `checking_a_fact(text, memory)`: the answer on screen confirmed, or None.
pub fn checking_a_fact(text_: &str, memory: &Memory) -> Option<String> {
    let said = text::squash(&text_.to_lowercase());
    let said = text::strip_chars(&said, " .!?,");
    if said.is_empty() || text::split(said).len() > 7 || CHECK_A_FACT.match_(said).is_none() {
        return None;
    }
    let last = memory.turns.last()?;
    let answer = text::squash(&last.bot);
    if answer.is_empty() || answer.ends_with('?') {
        return None;
    }
    if !_fact_from(&answer) {
        if text::split(&answer).len() < 8 || NO_FACT_SAID.match_(&answer).is_some() || NOT_A_FACT.is_match(&answer) {
            return None;
        }
        if crate::memory::topics::an_entry_for(&last.user).is_none() {
            return None;
        }
    }
    Some(_a_frame_not_said(&FACT_FRAMES, memory))
}

/// `_fact_from(answer)`: the answer on screen is a confirmation this reader gave a turn ago.
pub fn _fact_from(answer: &str) -> bool {
    let low = answer.to_lowercase();
    FACT_FRAMES.iter().any(|frame| low.starts_with(text::rstrip_chars(&frame.to_lowercase(), ".")))
}

/// `_a_frame_not_said(frames, memory)`: the first of `frames` the bot has not said in this
/// conversation yet.
pub fn _a_frame_not_said(frames: &[String], memory: &Memory) -> String {
    let all: Vec<String> = memory.turns.iter().map(|t_| text::squash(&t_.bot.to_lowercase())).collect();
    let said = &all[all.len().saturating_sub(6)..];
    for frame in frames {
        let lowered = frame.to_lowercase();
        let stem = text::rstrip_chars(&lowered, ".");
        if !said.iter().any(|line| line.starts_with(stem)) {
            return frame.clone();
        }
    }
    frames[0].clone()
}

/// `_next_frame(frames, used, said)`: the frame this verdict is said in, moving on from the one
/// the bot used last time.
pub fn _next_frame(frames: &[String], used: &[(String, usize)], said: &str) -> String {
    let stem = text::squash(said).to_lowercase();
    for (lead, going_to) in used {
        if stem.starts_with(lead.as_str()) {
            return frames[*going_to].clone();
        }
    }
    frames[0].clone()
}

/// `_first_clause(said, limit=13)`: the first reason in a verdict.
pub fn _first_clause(said: &str, limit: i64) -> String {
    let said = text::squash(said);
    if said.is_empty() {
        return String::new();
    }
    let first = match FIRST_CLAUSE_CUT.search(&said) {
        Some(m) => said[..m.start()].to_string(),
        None => said.clone(),
    };
    let mut cut = text::strip_chars(&first, " .,;:").to_string();
    let found = text::split(&cut);
    if found.len() as i64 > limit {
        let joined = found[..limit as usize].join(" ");
        cut = text::rstrip_chars(&joined, ",;:").to_string();
    }
    cut
}

/// `_a_done_word(word)`: this word is a participle.
pub fn _a_done_word(word: &str) -> bool {
    word.ends_with("ed") || DONE_WORDS.contains(word) || IRREGULAR_PARTICIPLES.contains(word)
}

/// `_the_verdict_they_gave(line)`: the word the report used for how it went, or "".
pub fn _the_verdict_they_gave(line: &str) -> String {
    let low = text::squash(&line.to_lowercase());
    let Some(m) = A_VERDICT.search(&low) else { return String::new() };
    let v = m.name("v").filter(|s| !s.is_empty()).or_else(|| m.name("w")).unwrap_or("");
    v.to_lowercase()
}

/// `_the_thing_they_had(line)`: the thing the line says they had, did or went to, or "".
pub fn _the_thing_they_had(line: &str) -> String {
    let low = text::squash(&line.to_lowercase());
    if low.is_empty() || !HAPPENED.is_match(&low) {
        return String::new();
    }
    for pattern in [*IT_WAS, *HAD_IT] {
        let Some(m) = pattern.search(&low) else { continue };
        let mut found: Vec<String> = Vec::new();
        for word in text::split(m.name_str("thing")) {
            if THING_END.contains(&word) {
                break;
            }
            found.push(word);
        }
        let mut thing = text::strip_chars(&found.join(" "), " ,;:-").to_string();
        if thing.is_empty() || NOT_THE_THING.contains(&thing) {
            continue;
        }
        if found.first().is_some_and(|w| _A_THING_DONE.contains(w)) {
            continue;
        }
        if found.iter().any(|w| NEVER_A_THING_THAT_WENT.contains(w)) {
            continue;
        }
        if found.last().is_some_and(|w| _A_STATE_ITSELF.contains(w)) && !found.iter().any(|w| _A_NAME_FOR_IT.contains(w)) {
            continue;
        }
        if found.iter().any(|w| _A_DESCRIPTION.contains(w)) {
            continue;
        }
        if found.iter().any(|w| THING_PARTICLE.contains(w)) {
            continue;
        }
        if found.last().is_some_and(|w| _a_done_word(w)) {
            found.pop();
            thing = text::strip_chars(&found.join(" "), " ,;:-").to_string();
            if found.len() < 2 || NOT_THE_THING.contains(&thing) {
                continue;
            }
        } else if found.len() > 1 && found[..found.len() - 1].iter().any(|w| _a_done_word(w)) {
            continue;
        }
        return thing;
    }
    String::new()
}

/// `pick(options, memory)` from the compose package, given the memory as the plain dict its
/// contract takes.
fn _pick(options: &[String], memory: &Memory) -> String {
    let plain = Value::from_plain(&serde_json::Value::Object(memory.to_plain()));
    crate::compose::pick(options, &plain, "")
}

/// `went(text, memory)`: the reply to "how did it go" after they told the chat about a thing they
/// had today, or None.
pub fn went(text_: &str, memory: &Memory) -> Option<String> {
    let said = text::squash(&text_.to_lowercase());
    let said = text::strip_chars(&said, " ?.!,");
    let found = WENT_ASK.match_(said)?;
    let turns = _turns(memory);
    let last = turns.last()?;
    let asked_before = _asked_already(&last.bot);
    let mut thing = String::new();
    let mut verdict = String::new();
    for turn in turns.iter().rev() {
        thing = _the_thing_they_had(&turn.user);
        if !thing.is_empty() {
            verdict = _the_verdict_they_gave(&turn.user);
            break;
        }
    }
    let asked_about = text::strip(found.name_str("thing")).to_string();
    if !asked_about.is_empty() {
        if thing.is_empty() || NOT_THE_THING.contains(&asked_about) || _A_THING_DONE.contains(&asked_about) {
            return None;
        }
        thing = asked_about;
    }
    if thing.is_empty() {
        return None;
    }
    if !verdict.is_empty() {
        let frames: &[String] = if _WENT_WELL.contains(&verdict) { &WENT_TOLD_WELL } else { &WENT_TOLD_BAD };
        let options: Vec<String> =
            frames.iter().map(|f| f.replace("{v}", &verdict).replace("{V}", &text::capitalize(&verdict))).collect();
        return Some(_pick(&options, memory));
    }
    if asked_before {
        return Some(_pick(&WENT_AGAIN, memory));
    }
    let options: Vec<String> = WENT_FRAMES.iter().map(|f| f.replace("{thing}", &thing)).collect();
    Some(_pick(&options, memory))
}

/// `_asked_already(reply)`: the bot's own line was this question.
pub fn _asked_already(reply: &str) -> bool {
    let said = text::squash(&reply.to_lowercase());
    if !said.ends_with('?') {
        return false;
    }
    WENT_ASK.match_(text::strip(text::rstrip_chars(&said, "?"))).is_some() || HOW_IT_WENT.is_match(&said)
}
