//! Port of `bot/memory/taking.py`: a bare "what can i take" is a question about the thing the
//! chat is on.
//!
//! "my back is killing me" / "what can i take" is asked of the knowledge base in the words its
//! entries are keyed on ("what can i take for back pain"), and answered only when the line is the
//! bare ask, the chat is not a trip, the chat says a complaint of the person's own, and the answer
//! names it and is not the answer already on the screen.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::mem7::{self, inline_re, lists, res, sets, values};
use crate::memory::topics;
use crate::py::re::{self, Re};
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

res!("taking"; FILLER, NOTHING_NAMED, TRIP, THEIRS, A_FORM_OF_A_VERB, IN_PAIN, FEEDING, PREGNANT, A_SENTENCE);
sets!("taking"; NOT_A_THING);
lists!("taking"; FORMULAS, PAINKILLER_ASKS);
values!("taking"; A_CARD_IS_LONG, THE_SUBJECTS_A_CHAT_HAS, THE_WORDS_A_SUBJECT_HAS, THE_ASKS_A_LINE_MAKES);
inline_re! {
    LETTERS_ONLY = r"^[a-z]+$", 0;
    WORDS = r"[a-z']+", 0;
}

fn int_of(v: &Value) -> usize {
    v.as_int().expect("an int global") as usize
}

/// Python: bot/memory/taking.py:111 `_flat(text)`
pub fn _flat(text: &str) -> String {
    pt::squash(text)
}

/// Python: bot/memory/taking.py:115 `_asks_what_to_take(text)`
///
/// True when the line is the bare ask and names nothing to take or to take it for.
pub fn _asks_what_to_take(text: &str) -> bool {
    let said = FILLER.sub("", &pt::lower(&_flat(text)));
    let said = pt::strip(&said);
    !said.is_empty() && NOTHING_NAMED.match_(said).is_some()
}

/// Python: bot/memory/taking.py:123 `_complaint_words()`
///
/// The words that make a line a complaint, taken from the health knowledge where it has them.
pub fn _complaint_words() -> HashSet<String> {
    let mut words: HashSet<String> = HashSet::new();
    // The knowledge base is read for its words and nothing else: a missing table narrows this.
    for name in ["bot.knowledge.body.HURT", "bot.knowledge.body._SORE_ANYWHERE"] {
        if let Some(table) = data::try_get(name) {
            words.extend(mem7::iterate(table).iter().map(mem7::py_str));
        }
    }
    words.extend(
        pt::split(
            "heartburn indigestion reflux nausea migraine headache toothache earache backache stomachache
        cough coughing cold flu covid temperature fever cystitis thrush hayfever rash hives eczema
        psoriasis dandruff ulcer blister sunburn burn sprain cramp cramping constipation diarrhoea
        diarrea sick vomiting warts verruca sciatica gout arthritis ibs period swollen swelling
        infected infection throat tooth teeth gum gums ear ears stomach tummy belly chest queasy
        nauseous vomit vomiting dizzy faint fainted bleeding rash pregnancy pregnant",
        ),
    );
    words.into_iter().filter(|w| !w.is_empty()).collect()
}

/// `\b(?:a|b|...)\b` with the longest first.
fn longest_first(words: Vec<String>) -> Re {
    let mut words = words;
    words.sort_by(|a, b| pt::len(b).cmp(&pt::len(a)));
    Re::new(&format!(r"\b(?:{})\b", words.iter().map(|w| re::escape(w)).collect::<Vec<_>>().join("|")), re::I)
}

/// Python: bot/memory/taking.py:157 `_someone_else()`
///
/// The words that make a complaint someone else's ("my son", "his back"), or None.
pub fn _someone_else() -> Option<Re> {
    let phrases = data::try_get("bot.knowledge.body.THIRD_PARTY")?;
    Some(longest_first(mem7::iterate(phrases).iter().map(mem7::py_str).collect()))
}

static COMPLAINT: LazyLock<Re> = LazyLock::new(|| longest_first(_complaint_words().into_iter().collect()));
static NOT_THEIRS: LazyLock<Option<Re>> = LazyLock::new(_someone_else);

/// Python: bot/memory/taking.py:168 `_a_complaint(subject)`
///
/// True when the person's own line says something is wrong with them.
pub fn _a_complaint(subject: &str) -> bool {
    let said = _flat(subject);
    if NOT_THEIRS.as_ref().is_some_and(|not_theirs| not_theirs.is_match(&said)) {
        return false; // a cough in their son is not a cough in them
    }
    COMPLAINT.is_match(&said)
}

/// Python: bot/memory/taking.py:182 `_subjects(memory)`
///
/// The things the chat has been on, newest first, as the person's own lines.
pub fn _subjects(memory: &Memory) -> Vec<String> {
    let mut seen: Vec<Value> = Vec::new();
    if let Some(record) = memory.get("topic").filter(|r| r.is_dict()) {
        seen.push(record.clone());
    }
    if let Some(Value::List(others)) = memory.get("topics") {
        seen.extend(others.iter().rev().filter(|o| o.is_dict()).cloned());
    }
    let mut out: Vec<String> = Vec::new();
    for other in seen {
        let said = _flat(&mem7::text_of(other.get("text")));
        if !said.is_empty() && !out.contains(&said) {
            out.push(said);
        }
    }
    out.truncate(int_of(&THE_SUBJECTS_A_CHAT_HAS));
    out
}

/// Python: bot/memory/taking.py:200 `_the_things_to_take_for(subject)`
///
/// The words of the person's line that name something to take something for.
pub fn _the_things_to_take_for(subject: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    for word in topics::keywords(subject) {
        if pt::len(&word) < 3 || NOT_A_THING.contains(&word) || LETTERS_ONLY.match_(&word).is_none() {
            continue;
        }
        let mut word = word;
        if A_FORM_OF_A_VERB.match_(&word).is_some() {
            // "coughing" is a cough.
            word.truncate(word.len() - if word.ends_with("ing") { 3 } else { 2 });
            let chars: Vec<char> = word.chars().collect();
            if chars.len() > 3 && chars[chars.len() - 1] == chars[chars.len() - 2] {
                word.pop();
            }
            if pt::len(&word) < 3 || NOT_A_THING.contains(&word) {
                continue;
            }
        }
        if !words.contains(&word) {
            words.push(word);
        }
    }
    words.truncate(int_of(&THE_WORDS_A_SUBJECT_HAS));
    words
}

/// Python: bot/memory/taking.py:219 `_a_trip_above(memory, text)`
///
/// True when the chat is about going somewhere, where the packing answer is the right one.
pub fn _a_trip_above(memory: &Memory, text: &str) -> bool {
    TRIP.is_match(&_flat(text)) || mem7::tail(&memory.turns, 2).iter().any(|turn| TRIP.is_match(&_flat(&turn.user)))
}

/// `answers.respond(text, memory) or ""`, with a failure read as "".
fn answers_respond(text: &str, memory: &mut Memory) -> String {
    mem7::py_try(|| crate::modules::answers::respond(text, memory)).flatten().unwrap_or_default()
}

/// Python: bot/memory/taking.py:229 `_the_answer(text, memory)`
///
/// What the knowledge base says to this line, asked on its own (a fresh memory that keeps only
/// the person's facts).
pub fn _the_answer(text: &str, memory: &Memory) -> String {
    let mut fresh = Memory::new();
    fresh.set("aside", true);
    for key in ["user", "others", "facts_log"] {
        if let Some(value) = memory.get(key) {
            fresh.set(key, value.clone());
        }
    }
    answers_respond(text, &mut fresh)
}

/// Python: bot/memory/taking.py:243 `_the_said_in_the_chat(memory)`
///
/// Every line the person has typed in this chat, as one string.
pub fn _the_said_in_the_chat(memory: &Memory) -> String {
    memory.turns.iter().map(|turn| _flat(&turn.user)).collect::<Vec<_>>().join(" ")
}

/// Python: bot/memory/taking.py:251 `_the_last_reply(memory)`
pub fn _the_last_reply(memory: &Memory) -> String {
    memory.turns.last().map_or(String::new(), |turn| _flat(&turn.bot))
}

/// Python: bot/memory/taking.py:258 `_the_situation_this_chat_is_in(reply, memory)`
///
/// The card, cut to the situation the person is actually in.
pub fn _the_situation_this_chat_is_in(reply: &str, memory: &Memory) -> String {
    let said = _the_said_in_the_chat(memory);
    if FEEDING.is_match(&said) || !PREGNANT.is_match(&said) || !FEEDING.is_match(reply) {
        return reply.to_string();
    }
    let sentences = A_SENTENCE.findall_str(reply);
    let kept: Vec<&String> = sentences.iter().filter(|s| !FEEDING.is_match(s)).collect();
    if kept.is_empty() || kept.len() == sentences.len() {
        return reply.to_string();
    }
    pt::squash(&kept.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" "))
}

/// Python: bot/memory/taking.py:274 `_the_answer_today(text, memory)`
///
/// The answer the knowledge base gives this line in this chat, as the line stands. (PyBot hands
/// it a shallow copy of the memory; here it is a copy.)
pub fn _the_answer_today(text: &str, memory: &Memory) -> String {
    let mut copy = memory.clone();
    answers_respond(text, &mut copy)
}

/// Python: bot/memory/taking.py:284 `_the_same_answer(reply, text, memory, today)`
///
/// True when the knowledge base answers the line as asked with these same words. `today` caches
/// the answer to the line for the turn.
pub fn _the_same_answer(reply: &str, text: &str, memory: &Memory, today: &mut Option<String>) -> bool {
    let answer = today.get_or_insert_with(|| _the_answer_today(text, memory));
    _flat(answer) == _flat(reply) && pt::len(&_the_last_reply(memory)) >= int_of(&A_CARD_IS_LONG)
}

/// Python: bot/memory/taking.py:301 `_names(reply, word)`
///
/// True when the answer says the word back, a plural or a form of it counting.
pub fn _names(reply: &str, word: &str) -> bool {
    let said = pt::lower(&_flat(reply));
    !said.is_empty() && WORDS.findall_str(&said).iter().any(|other| other == word || topics::alike(other, word))
}

/// Python: bot/memory/taking.py:312 `_the_answer_above(reply, memory)`
///
/// True when this is the answer the person has just read, in its own opening words.
pub fn _the_answer_above(reply: &str, memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else {
        return false;
    };
    let low = pt::lower(&_flat(reply));
    let words = pt::split_ref(&low);
    let opening = words[..words.len().min(10)].join(" ");
    !opening.is_empty() && pt::lower(&_flat(&last.bot)).contains(&opening)
}

/// Python: bot/memory/taking.py:329 `_the_subjects_to_answer(memory, text)`
///
/// The things in the chat it is worth answering a bare ask about, newest first.
pub fn _the_subjects_to_answer(memory: &Memory, _text: &str) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    for subject in _subjects(memory) {
        if !THEIRS.is_match(&subject) || subject.contains('?') || !_a_complaint(&subject) {
            continue;
        }
        let words = _the_things_to_take_for(&subject);
        if !words.is_empty() {
            out.push((subject, words));
        }
    }
    out
}

/// Python: bot/memory/taking.py:348 `the_ask_and_the_subject(text, memory)`
///
/// True when the line is the bare ask and the chat has something to answer it about. (A reading
/// that fails claims nothing.)
pub fn the_ask_and_the_subject(text: &str, memory: &Memory) -> Option<bool> {
    let claimed = mem7::py_try(|| {
        _asks_what_to_take(text) && !_a_trip_above(memory, text) && !_the_subjects_to_answer(memory, text).is_empty()
    });
    Some(claimed.unwrap_or(false))
}

/// Python: bot/memory/taking.py:364 `reply_for(text, memory)`
///
/// The answer to a bare "what can i take", about the thing the chat is on, or "".
pub fn reply_for(text: &str, memory: &Memory) -> String {
    if !_asks_what_to_take(text) || _a_trip_above(memory, text) {
        return String::new();
    }
    let mut asked = 0;
    let mut today: Option<String> = None;
    let mut already = String::new();
    let mut in_pain = String::new();
    for subject in _subjects(memory) {
        if !THEIRS.is_match(&subject) || subject.contains('?') || !_a_complaint(&subject) {
            continue;
        }
        for word in _the_things_to_take_for(&subject) {
            for formula in FORMULAS.iter() {
                if asked >= int_of(&THE_ASKS_A_LINE_MAKES) {
                    return String::new();
                }
                asked += 1;
                let reply = _the_answer(&formula.replace("%s", &word), memory);
                if reply.is_empty() || !_names(&reply, &word) {
                    continue;
                }
                if _the_answer_above(&reply, memory) {
                    if already.is_empty() {
                        already = reply;
                        in_pain = subject.clone();
                    }
                    continue;
                }
                if _the_same_answer(&reply, text, memory, &mut today) {
                    continue; // the line gets this answer on its own
                }
                return _the_situation_this_chat_is_in(&reply, memory);
            }
        }
    }
    if !in_pain.is_empty() && IN_PAIN.is_match(&in_pain) {
        // What they are asking is what to take for the pain.
        let reply = _the_answer(&PAINKILLER_ASKS[0], memory);
        if !reply.is_empty() && !_the_answer_above(&reply, memory) && !_the_same_answer(&reply, text, memory, &mut today) {
            return reply;
        }
    }
    if !already.is_empty() && _the_same_answer(&already, text, memory, &mut today) {
        return String::new();
    }
    already
}
