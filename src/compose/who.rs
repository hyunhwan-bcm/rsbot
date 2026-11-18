//! Port of `bot/compose/who.py`: a bare "who do i ring" after an answer, answered with the person
//! the answer (or the thread) named and the one line on how to get to them.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_sets, bundle_vals, inline_re};
use crate::compose::pick;
use crate::memory::holds as earlier;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

bundle_res!("bot.compose.who"; WHO_ASK, PREAMBLE);
bundle_sets!("bot.compose.who"; QUIET, CARE_TOPICS);
bundle_vals!("bot.compose.who"; EXPERTS, LEAD, BY_TOPIC, THREAD, EARLY);
inline_re!(WHITESPACE, r"\s+");
inline_re!(SENTENCE_BREAK, r"(?<=[.!?]) ");

/// EXPERTS rows: (name, the words that name them, the ways in).
static EXPERT_ROWS: LazyLock<Vec<(String, &'static Re, Vec<String>)>> = LazyLock::new(|| {
    EXPERTS
        .items()
        .iter()
        .map(|row| (row.items()[0].str().to_string(), row.items()[1].as_re().expect("a pattern"), row.items()[2].strs()))
        .collect()
});

/// A person named in an answer: (name, the sentence naming them, the person's character offset
/// in that sentence).
pub type Named = (String, String, usize);

/// Python: bot/compose/who.py:134 `_flat(text)`
pub fn _flat(text_: &str) -> String {
    let t = text::strip(text_).to_lowercase().replace(['\u{2019}', '\u{2018}'], "'");
    WHITESPACE.sub(" ", &t)
}

/// Python: bot/compose/who.py:140 `asks_who(text)`
///
/// True when the line is nothing but a request for the person to see.
pub fn asks_who(text_: &str) -> bool {
    WHO_ASK.match_(text::strip_chars(&_flat(text_), " .!?")).is_some()
}

/// Python: bot/compose/who.py:145 `_last_module(memory)`
pub fn _last_module(memory: &Memory) -> String {
    earlier::last_module(memory)
}

/// Python: bot/compose/who.py:149 `_last_reply(memory)`
pub fn _last_reply(memory: &Memory) -> String {
    earlier::last_reply(memory).str().to_string()
}

/// Python: bot/compose/who.py:153 `_names_no_subject(line)`
///
/// True for a line that only says to go on: what is left once the grammar is gone is nothing.
pub fn _names_no_subject(line: &str) -> bool {
    if asks_who(line) {
        return true;
    }
    let read = crate::understanding::understand(&_flat(line), Some(&Memory::new()));
    !read.keywords.iter().any(|w| text::len(w) > 2)
}

/// Python: bot/compose/who.py:165 `_the_subject(memory)`
///
/// (turn index, the answer to it, the line) for the last line that named a subject.
pub fn _the_subject(memory: &Memory) -> Option<(usize, String, String)> {
    for (index, turn) in memory.turns.iter().enumerate().rev() {
        let line = &turn.user;
        if line.is_empty() || _names_no_subject(line) {
            continue;
        }
        return Some((index, turn.bot.clone(), line.clone()));
    }
    None
}

/// Python: bot/compose/who.py:188 `_thread_text(memory, answer='')`
///
/// What the person said over the last few lines, which is what the thread is about. (Python
/// walks `holds.turns(memory)`, which is every turn that is a dict: the memory's turns here.)
pub fn _thread_text(memory: &Memory, answer: &str) -> String {
    let turns = &memory.turns;
    let mut said = vec![answer.to_string()];
    said.extend(turns[turns.len().saturating_sub(3)..].iter().map(|t| t.user.clone()));
    said.into_iter().filter(|p| !p.is_empty()).collect::<Vec<_>>().join(" ")
}

/// Python: bot/compose/who.py:207 `_sentence_at(text, at)`
///
/// (the sentence of `text` the byte offset `at` sits in, the byte where that sentence starts).
pub fn _sentence_at(text_: &str, at: usize) -> (String, usize) {
    let mut begin = 0;
    for found in SENTENCE_BREAK.finditer(text_) {
        if found.start() >= at {
            return (text_[begin..found.start()].to_string(), begin);
        }
        begin = found.end();
    }
    (text_[begin..].to_string(), begin)
}

/// Python: bot/compose/who.py:217 `_person_in(said)`
///
/// (person, the sentence that names them, offset of the person in it) from an answer, or None.
pub fn _person_in(said: &str) -> Option<Named> {
    let mut best: Option<(String, String, usize, usize)> = None;
    for (name, words, _way) in EXPERT_ROWS.iter() {
        let Some(found) = words.search(said) else { continue };
        if best.as_ref().is_some_and(|b| found.start() >= b.3) {
            continue;
        }
        // Where the person sits inside their sentence.
        let (sentence, start) = _sentence_at(said, found.start());
        let at = if found.start() > start { text::len(&said[start..found.start()]) } else { 0 };
        best = Some((name.clone(), sentence, at, found.start()));
    }
    best.map(|(name, sentence, at, _)| (name, sentence, at))
}

/// Python: bot/compose/who.py:235 `_cut_the_preamble(sentence, at)`
///
/// The referral from the person's own clause in, without the run-up to it; and where the cut
/// was made (character offsets).
pub fn _cut_the_preamble(sentence: &str, at: usize) -> (String, usize) {
    let mut start = 0usize;
    let mut start_byte = 0usize;
    for found in PREAMBLE.finditer(sentence) {
        let end = text::char_index(sentence, found.end());
        if end > at {
            break;
        }
        if at - end <= 80 {
            start = end;
            start_byte = found.end();
        }
    }
    (text::strip(&sentence[start_byte..]).to_string(), start)
}

/// Python: bot/compose/who.py:251 `_tidy(clause)`
pub fn _tidy(clause: &str) -> String {
    let mut c = text::rstrip_chars(text::strip(text::strip_chars(text::strip(clause), "\"")), " ,;:").to_string();
    if !c.is_empty() && !c.ends_with(['.', '!', '?']) {
        c.push('.');
    }
    if text::is_lower(&c.chars().take(1).collect::<String>()) {
        c = text::upper_first(&c);
    }
    c
}

/// Python: bot/compose/who.py:260 `_lead_of(name)`
pub fn _lead_of(name: &str) -> String {
    LEAD.get(name).map(|v| v.str().to_string()).unwrap_or_else(|| name.to_string())
}

/// Python: bot/compose/who.py:264 `_the_way_in(name, memory, said, clause='')`
///
/// The one line of the reply that is not the answer's: how to get to the person. A way the
/// answer already gave is not said again.
pub fn _the_way_in(name: &str, memory: &Memory, said: &str, clause: &str) -> String {
    let ways: Vec<String> = EXPERT_ROWS.iter().filter(|(other, _, _)| other == name).map(|(_, _, w)| w.clone()).last().unwrap_or_default();
    if ways.is_empty() {
        return String::new();
    }
    let said_flat = _flat(clause);
    let fresh: Vec<String> = ways.iter().filter(|w| !_says_it_again(w, &said_flat)).cloned().collect();
    pick(if fresh.is_empty() { &ways } else { &fresh }, memory, said)
}

/// Python: bot/compose/who.py:281 `_says_it_again(way, said)`
///
/// True when the way in repeats a run of three words the clause already used.
pub fn _says_it_again(way: &str, said: &str) -> bool {
    let flat = _flat(way);
    let words = text::split_ref(&flat);
    (0..words.len().saturating_sub(2)).any(|at| said.contains(&words[at..at + 3].join(" ")))
}

/// Python: bot/compose/who.py:290 `_person_for_the_topic(memory, said='', subject='')`
///
/// The person the thread is about, when the answer named nobody.
pub fn _person_for_the_topic(memory: &Memory, said: &str, subject: &str) -> String {
    let mut topics: Vec<String> = Vec::new();
    if let Some(read) = memory.understood.as_ref() {
        topics = read.topics.iter().map(|t| t.to_lowercase()).collect();
        if let Some(one) = read.topic.as_ref().map(|t| t.to_lowercase()).filter(|t| !t.is_empty()) {
            topics.push(one);
        }
    }
    if !subject.is_empty() {
        let theirs = crate::understanding::understand(&_flat(subject), Some(&Memory::new()));
        if !theirs.topics.is_empty() {
            topics = theirs.topics.iter().map(|t| t.to_lowercase()).collect();
        }
    }
    if topics.iter().any(|t| CARE_TOPICS.contains(t)) {
        let thread = _thread_text(memory, said);
        for row in THREAD.items() {
            if row.items()[1].as_re().expect("a pattern").search(&thread).is_some() {
                return row.items()[0].str().to_string();
            }
        }
    }
    for row in BY_TOPIC.items() {
        if topics.iter().any(|t| t == row.items()[0].str()) {
            return row.items()[1].str().to_string();
        }
    }
    String::new()
}

/// Python: bot/compose/who.py:320 `_from_the_answer(name, sentence, at, memory, said, repeat=True)`
///
/// The person the answer named, with the one line on how to get to them. `repeat` is False for
/// an answer the person has already heard.
pub fn _from_the_answer(name: &str, sentence: &str, at: usize, memory: &Memory, said: &str, repeat: bool) -> Option<String> {
    let (clause, start) = _cut_the_preamble(sentence, at);
    let way = _the_way_in(name, memory, said, &clause);
    if way.is_empty() {
        return None;
    }
    let early = EARLY.as_int().expect("an int") as usize;
    if repeat && at - start <= early {
        // The sentence says who to see at the front of it: hand it back with the way in after.
        let mut body = _tidy(&clause);
        if !_says_it_again(&way, &_flat(&body)) {
            body = format!("{body} {}.", text::upper_first(&way));
        }
        return Some(body);
    }
    // The person is named at the end of a condition: the reply is the person and the way in.
    Some(format!("{}: {way}.", _lead_of(name)))
}

/// Python: bot/compose/who.py:343 `reply_for(text, memory)`
///
/// The person to see, or None on every line that is not a bare ask for one.
pub fn reply_for(text_: &str, memory: &Memory) -> Option<String> {
    if !asks_who(text_) || QUIET.contains(&_last_module(memory)) {
        return None;
    }
    let answer = _last_reply(memory);
    if answer.is_empty() {
        return None;
    }
    // An answer that already named someone is the one to hand back, condition and all.
    if let Some((name, sentence, at)) = _person_in(&answer) {
        if let Some(reply) = _from_the_answer(&name, &sentence, at, memory, text_, true).filter(|r| !r.is_empty()) {
            return Some(reply);
        }
    }
    // The last answer named nobody: the person the thread already named.
    let subject = _the_subject(memory);
    if let Some((index, bot, _)) = subject.as_ref() {
        if !bot.is_empty() && *index != memory.turns.len() - 1 {
            if let Some((name, sentence, at)) = _person_in(bot) {
                if let Some(reply) = _from_the_answer(&name, &sentence, at, memory, text_, false).filter(|r| !r.is_empty()) {
                    return Some(reply);
                }
            }
        }
    }
    let name = _person_for_the_topic(memory, text_, subject.as_ref().map(|s| s.2.as_str()).unwrap_or(""));
    if name.is_empty() {
        return None;
    }
    let way = _the_way_in(&name, memory, text_, "");
    if way.is_empty() {
        return None;
    }
    Some(format!("{}: {way}.", _lead_of(&name)))
}

/// Python: bot/compose/who.py:377 `takes_as_it_comes(text, memory)`
///
/// Only on a line that is nothing but the ask, over a thread that has a person to name.
pub fn takes_as_it_comes(text_: &str, memory: &Memory) -> Option<bool> {
    Some(crate::compose::com4::attempt(|| reply_for(text_, memory).is_some()).unwrap_or(false))
}
