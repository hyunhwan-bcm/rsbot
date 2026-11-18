//! Port of `bot/modules/done_ask.py`: a doneness ask names a state and no dish ("and how do i know
//! its cooked"), so it is read against the dish the chat is on: the dish's own word goes in front
//! of the line in `memory["understood"]`, where every module below reads it. The module has no
//! `respond`; it only changes how a line is read.

use crate::data;
use crate::knowledge::entry::{Entry, KeyPart};
use crate::memory::topics;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

fn name(n: &str) -> String {
    format!("bot.modules.done_ask.{n}")
}

static A_DONENESS_ASK: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("A_DONENESS_ASK")));
static DONENESS_WORDS: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("DONENESS_WORDS")));
static DONENESS_KEYS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&name("DONENESS_KEYS")));
/// `_topics.MEASURE_WORDS`.
static MEASURE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.topics.MEASURE_WORDS"));

/// Python: bot/modules/done_ask.py:60 `_entry(text)`: the knowledge entry a line reaches, or None.
pub fn _entry(text: &str) -> Option<&'static Entry> {
    let said = text::squash(&text::lower(text));
    if said.is_empty() {
        return None;
    }
    let reading = Understood { words: topics::words(&said), clean: said, ..Understood::default() };
    crate::knowledge::lookup::find(&reading, None, None, false)
}

/// Python: bot/modules/done_ask.py:69 `_the_answer_to_lean_on(memory)`: the bot's last reply,
/// or "".
pub fn _the_answer_to_lean_on(memory: &Memory) -> String {
    memory.turns.iter().rev().map(|one| one.bot.clone()).find(|said| !said.is_empty()).unwrap_or_default()
}

/// Python: bot/modules/done_ask.py:78 `_the_topics_on_the_chat(memory)`: the topic in hand first,
/// then the ones the chat has been on, newest first.
pub fn _the_topics_on_the_chat(memory: &Memory) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Some(kept) = memory.get("topics") {
        for one in kept.items().iter().rev() {
            // `(one or {}).get("text")`: a falsy entry reads as {}, anything else must be a dict.
            if !one.truthy() {
                continue;
            }
            assert!(one.is_dict(), "a topic record that is not a dict");
            let said = one.get("text").filter(|v| v.truthy()).map(|v| v.str().to_string()).unwrap_or_default();
            if !said.is_empty() && !out.contains(&said) {
                out.push(said);
            }
        }
    }
    let said = match memory.get("topic") {
        Some(record) if record.is_dict() => record.get("text").filter(|v| v.truthy()).map(|v| v.str().to_string()).unwrap_or_default(),
        _ => String::new(),
    };
    if let Some(at) = out.iter().position(|one| *one == said) {
        out.remove(at);
    }
    if !said.is_empty() {
        out.insert(0, said);
    }
    out
}

/// Python: bot/modules/done_ask.py:94 `_key_groups(item)`: the groups of phrases an entry is keyed
/// by, in the order the entry says them.
pub fn _key_groups(item: &'static Entry) -> Vec<Vec<String>> {
    item.key
        .iter()
        .map(|part| match part {
            KeyPart::Word(word) => vec![word.clone()],
            KeyPart::Any(words) => words.clone(),
        })
        .collect()
}

/// Python: bot/modules/done_ask.py:105 `_keys_the_word(phrases, word)`: a key group holds the
/// word, in any of the forms it takes.
pub fn _keys_the_word(phrases: &[String], word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    let mut said: HashSet<String> = HashSet::new();
    for phrase in phrases {
        said.extend(topics::words(phrase));
    }
    if said.contains(word) {
        return true;
    }
    for one in &said {
        // A word the stemmer does not like simply does not match.
        let matched = crate::runtime::guarded(|| topics::forms(one).contains(word) || topics::forms(word).contains(one)).unwrap_or(false);
        if matched {
            return true;
        }
    }
    false
}

/// Python: bot/modules/done_ask.py:124 `_about_the_thing(item, word)`: the entry's first key group
/// is the word, as the thing the entry is about.
pub fn _about_the_thing(item: &'static Entry, word: &str) -> bool {
    let groups = _key_groups(item);
    !groups.is_empty() && _keys_the_word(&groups[0], word)
}

/// Python: bot/modules/done_ask.py:136 `_says_how_it_turns_out(item)`: the entry is keyed to a
/// done state, every word of the state in one group.
pub fn _says_how_it_turns_out(item: &'static Entry) -> bool {
    for group in _key_groups(item) {
        let mut said: HashSet<String> = HashSet::new();
        for phrase in &group {
            said.extend(topics::words(phrase));
        }
        for one in DONENESS_KEYS.iter() {
            if topics::words(one).iter().all(|word| said.contains(word)) {
                return true;
            }
        }
    }
    false
}

/// Python: bot/modules/done_ask.py:153 `_the_thing_words(memory, said_words)`: the words of the
/// things the chat has been on, newest first, that the line does not say.
pub fn _the_thing_words(memory: &Memory, said_words: &HashSet<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for topic in _the_topics_on_the_chat(memory) {
        for word in topics::keywords(&topic) {
            if said_words.contains(&word) || MEASURE_WORDS.contains(&word) || out.contains(&word) {
                continue;
            }
            out.push(word);
        }
    }
    out
}

/// Python: bot/modules/done_ask.py:164 `_the_chat_has_moved_on(found, memory)`: the newest topic
/// that reaches an entry at all reaches one in another domain.
pub fn _the_chat_has_moved_on(found: &'static Entry, memory: &Memory) -> bool {
    let said = text::squash(memory.understood.as_ref().map(|u| u.clean.as_str()).unwrap_or(""));
    let mine = found.domain.as_str();
    if mine.is_empty() {
        return false;
    }
    for topic in _the_topics_on_the_chat(memory) {
        let topic = text::squash(&topic);
        if topic.is_empty() || topic == said {
            continue;
        }
        let Some(here) = _entry(&topic) else { continue };
        let theirs = here.domain.as_str();
        return !theirs.is_empty() && theirs != mine;
    }
    false
}

/// Python: bot/modules/done_ask.py:193 `the_done_state_of_the_thing_in_hand(line, memory)`: the
/// doneness ask with the word of the thing the chat is on in it, or "".
pub fn the_done_state_of_the_thing_in_hand(line: &str, memory: &Memory) -> String {
    let said = text::squash(&text::lower(line));
    if said.is_empty() || !A_DONENESS_ASK.is_match(&said) || !DONENESS_WORDS.is_match(&said) {
        return String::new();
    }
    if _the_answer_to_lean_on(memory).is_empty() {
        // Nothing answered yet: a first message is answered from its own words.
        return String::new();
    }
    let said_words: HashSet<String> = topics::words(&said).into_iter().collect();
    for word in _the_thing_words(memory, &said_words) {
        for filled in [format!("{word} {said}"), format!("{} {word}", text::rstrip_chars(&said, "?.! "))] {
            let Some(found) = _entry(&filled) else { continue };
            if !_about_the_thing(found, &word) {
                continue;
            }
            if !_says_how_it_turns_out(found) {
                continue;
            }
            if _the_chat_has_moved_on(found, memory) {
                return String::new();
            }
            return filled;
        }
    }
    String::new()
}

/// Python: bot/modules/done_ask.py:224 `observe(line, memory)`: read a doneness ask against the
/// dish the chat is on, and write that reading down.
pub fn observe(text: &str, memory: &mut Memory) {
    let Some(read) = memory.understood.as_ref() else { return };
    let said = if !read.clean.is_empty() {
        read.clean.clone()
    } else if !read.text.is_empty() {
        read.text.clone()
    } else {
        text.to_string()
    };
    let filled = the_done_state_of_the_thing_in_hand(&said, memory);
    if filled.is_empty() {
        return;
    }
    let mut reading = read.clone();
    reading.words = topics::words(&filled);
    // `if "norm_text" in reading`: the reading always carries it.
    reading.norm_text = filled.clone();
    reading.clean = filled;
    memory.understood = Some(reading);
}
