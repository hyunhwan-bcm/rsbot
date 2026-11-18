//! Port of `bot/memory/repair.py`: they swapped the thing they were asking about ("no i meant
//! the front room"). The ask is kept and said again about the thing they meant, and the store
//! keeps the swap so the turns after it are about that thing.

use super::pyv;
use super::topics::{self, keywords};
use super::{last_user, others, remember_other};
use crate::data::{self, Value};
use crate::py::re::{self, Match, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understanding::understand;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.memory.repair.";
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn words_set(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}
fn pool(name: &str) -> Vec<String> {
    data::strs(&format!("{M}{name}"))
}

static REPAIR: LazyLock<&'static Re> = LazyLock::new(|| rx("REPAIR"));
static BARE_REPAIR: LazyLock<&'static Re> = LazyLock::new(|| rx("BARE_REPAIR"));
static NOT_A_SWAP: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("NOT_A_SWAP"));
static A_POSSESSION_REPAIR: LazyLock<&'static Re> = LazyLock::new(|| rx("A_POSSESSION_REPAIR"));
static A_VERB_IN_THE_THING: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("A_VERB_IN_THE_THING"));
static AN_INTENTION: LazyLock<&'static Re> = LazyLock::new(|| rx("AN_INTENTION"));
static A_VALUE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_VALUE"));
static A_CONTRAST: LazyLock<&'static Re> = LazyLock::new(|| rx("A_CONTRAST"));
static AN_ASK: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("AN_ASK"));
static NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("NOT_A_THING"));
static A_REMARK: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("A_REMARK"));
static NOT_THE_THING: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("NOT_THE_THING"));
static A_DETERMINER: LazyLock<&'static Re> = LazyLock::new(|| rx("A_DETERMINER"));
static A_POINTER: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("A_POINTER"));
static A_FIGURE_REPAIR: LazyLock<&'static Re> = LazyLock::new(|| rx("A_FIGURE_REPAIR"));
static FIGURE_UNITS: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("FIGURE_UNITS"));
static A_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_FIGURE"));
static FIGURE_OR_WORD: LazyLock<&'static Re> = LazyLock::new(|| rx("FIGURE_OR_WORD"));
static SENTENCE_END: LazyLock<&'static Re> = LazyLock::new(|| rx("SENTENCE_END"));
static THE_WORD_FOR: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.memory.repair.THE_WORD_FOR"));
static SWAPPED: LazyLock<Vec<String>> = LazyLock::new(|| pool("SWAPPED"));
static AN_ASK_OPENS: LazyLock<&'static Re> = LazyLock::new(|| rx("AN_ASK_OPENS"));
static HEDGE: LazyLock<&'static Re> = LazyLock::new(|| rx("HEDGE"));
static A_REAL_ANSWER: LazyLock<usize> = LazyLock::new(|| data::get("bot.memory.repair.A_REAL_ANSWER").as_int().unwrap() as usize);
static REPEAT_LEAD: LazyLock<&'static Re> = LazyLock::new(|| rx("REPEAT_LEAD"));
static SAME_AGAIN: LazyLock<Vec<String>> = LazyLock::new(|| pool("SAME_AGAIN"));
static SAME_ADVICE: LazyLock<Vec<String>> = LazyLock::new(|| pool("SAME_ADVICE"));
static THE_FACT_MOVED: LazyLock<Vec<String>> = LazyLock::new(|| pool("THE_FACT_MOVED"));
static MONTHS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.MONTHS"));
static DAYS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.DAYS"));
static RELATIONS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.RELATIONS"));

static TO_SAY: LazyLock<Re> = LazyLock::new(|| Re::new(r"^to\s+(?:say\s+|mean\s+)?", re::U));
static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9']+", re::U));
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", re::U));
static A_LEADING_DETERMINER: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"^(?:a|an|the|my|our|your|his|her|their|this|that|these|those)\b", re::I | re::U));
static A_LEADING_DETERMINER_WORD: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"^(?:a|an|the|my|our|your|his|her|their|this|that|these|those)\s+", re::I | re::U));

const THING_EDGE: &str = " ,.!?;:\"'";
const DETERMINERS: [&str; 12] = ["a", "an", "the", "my", "our", "your", "his", "her", "their", "its", "some", "any"];
const LEADS: [&str; 8] = ["the", "a", "an", "my", "our", "your", "his", "her"];

/// `[word.strip("'") for word in re.findall(r"[a-z0-9']+", text.lower())]`.
fn words_of(said: &str) -> Vec<String> {
    WORDS.findall_str(&said.to_lowercase()).into_iter().map(|w| text::strip_chars(&w, "'").to_string()).collect()
}

/// Python: bot/memory/repair.py:156 `_the_thing(text)`
/// The thing the repair names, in the words it was said in, or None.
pub fn _the_thing(text_: &str) -> Option<String> {
    let said = text::squash(text_);
    if AN_INTENTION.is_match(&said) {
        return None;
    }
    if A_CONTRAST.is_match(&said) {
        // "no, thursday not tuesday" puts one value against another: the store's to move.
        return None;
    }
    let mut bare = false;
    let found = match REPAIR.match_(&said) {
        Some(found) => found,
        None => match A_POSSESSION_REPAIR.match_(&said) {
            Some(found) => found,
            None => {
                let found = BARE_REPAIR.match_(&said)?;
                bare = true;
                found
            }
        },
    };
    // "sorry i meant to say bristol": the phrase is how they said it, not what they meant.
    let thing = TO_SAY.sub("", text::strip(found.name_str("thing")));
    let thing = text::strip_chars(&thing, THING_EDGE).to_string();
    if thing.is_empty() || A_VALUE.is_match(&thing) {
        return None;
    }
    let words = words_of(&thing);
    if words.is_empty() || words.iter().all(|w| NOT_A_THING.contains(w)) {
        return None;
    }
    if A_REMARK.contains(&words[0]) {
        return None; // "i mean its fine", "i mean whatever"
    }
    if bare && A_DETERMINER.match_(&thing).is_none() {
        // A thing named with no article right after "no" is what is missing, not a swap.
        return None;
    }
    if bare && words.iter().filter(|w| !NOT_THE_THING.contains(*w)).all(|w| NOT_A_SWAP.contains(w)) {
        return None;
    }
    if bare && words.iter().any(|w| A_VERB_IN_THE_THING.contains(w)) {
        return None; // "no the boiler is fine" is an answer about the boiler
    }
    if AN_ASK.contains(&words[0]) {
        return None; // the frame is followed by an ask, not a thing
    }
    let last = &words[words.len() - 1];
    if NOT_A_THING.contains(last) && *last != words[0] {
        // A thing that trails off into a word that names nothing.
        let nothing = words.iter().all(|w| NOT_A_THING.contains(w) || NOT_THE_THING.contains(w));
        return if nothing { None } else { Some(thing) };
    }
    Some(thing)
}

/// Python: bot/memory/repair.py:217 `_a_different_thing(old, thing)`
/// Whether the thing they meant says something the one already in the ask did not.
pub fn _a_different_thing(old: &str, thing: &str) -> bool {
    let held = _content_words(old);
    let said = _content_words(thing);
    !said.is_empty() && said.iter().any(|w| !held.contains(w))
}

/// Python: bot/memory/repair.py:234 `_content_words(text)`
/// The words a thing is named with, determiner and all left out.
pub fn _content_words(text_: &str) -> HashSet<String> {
    words_of(text_).into_iter().filter(|w| !w.is_empty() && !DETERMINERS[..10].contains(&w.as_str()) && w != "some" && w != "any").collect()
}

/// Python: bot/memory/repair.py:247 `_the_reading(line)`
/// The understanding of the line the correction is about. (Python keeps a small cache of these;
/// the reading is the same either way.)
pub fn _the_reading(line: &str) -> Understood {
    pyv::attempt(|| understand(line, Some(&Memory::new()))).unwrap_or_default()
}

/// Python: bot/memory/repair.py:265 `_the_thing_the_reading_named(line)`
/// The thing the reading finds in the line, or "" when it finds no single one.
pub fn _the_thing_the_reading_named(line: &str) -> String {
    let read = _the_reading(line);
    let mut found: Vec<(i64, String)> = read
        .entities
        .iter()
        .filter(|one| matches!(one.kind.as_str(), "thing" | "animal" | "food" | "place"))
        .map(|one| (one.at, text::squash(&one.text.to_lowercase())))
        .filter(|(_, said)| !said.is_empty())
        .collect();
    if found.is_empty() {
        return String::new();
    }
    // Words that sit next to each other are one thing; a gap means two things.
    found.sort_by_key(|(at, _)| *at);
    let mut one = found[0].1.clone();
    for index in 1..found.len() {
        let (at, said) = &found[index];
        let before = &found[index - 1];
        if *at == before.0 + text::split_ref(&before.1).len() as i64 {
            one = format!("{one} {said}");
        } else {
            return String::new();
        }
    }
    if text::squash(&line.to_lowercase()).contains(&one) { one } else { String::new() }
}

/// Python: bot/memory/repair.py:303 `_the_thing_the_last_line_named(line)`
/// The thing the ask before the repair was about: its last content word, or None.
pub fn _the_thing_the_last_line_named(line: &str) -> Option<String> {
    for word in words_of(line).into_iter().rev() {
        if text::len(&word) < 3 || NOT_THE_THING.contains(&word) || NOT_A_THING.contains(&word) {
            continue;
        }
        if MONTHS.contains(&word) || DAYS.contains(&word) || text::is_digit(&word) {
            return None;
        }
        if !keywords(&word).is_empty() {
            return Some(word);
        }
        return None;
    }
    None
}

/// Python: bot/memory/repair.py:333 `_the_noun_the_line_named(line)`
/// The thing a pointer counts: the last noun the ask named, with the verbs passed over.
pub fn _the_noun_the_line_named(line: &str) -> Option<String> {
    for word in words_of(line).into_iter().rev() {
        if text::len(&word) < 3 || NOT_THE_THING.contains(&word) || NOT_A_THING.contains(&word) {
            continue;
        }
        if A_VERB_IN_THE_THING.contains(&word) || MONTHS.contains(&word) || DAYS.contains(&word) {
            continue;
        }
        if !keywords(&word).is_empty() {
            return Some(word);
        }
        return None;
    }
    None
}

/// Python: bot/memory/repair.py:351 `_the_pointer_refines(thing, line)`
/// (the word the pointer counts, the thing to put in its place): "the lower one" about "my back
/// hurts" is ("back", "lower back").
pub fn _the_pointer_refines(thing: &str, line: &str) -> Option<(String, String)> {
    let words = words_of(thing);
    if words.is_empty() || !A_POINTER.contains(&words[words.len() - 1]) {
        return None;
    }
    let mods: Vec<&String> = words[..words.len() - 1]
        .iter()
        .filter(|w| text::len(w) >= 2 && !NOT_THE_THING.contains(*w) && !NOT_A_THING.contains(*w))
        .collect();
    if mods.is_empty() {
        return None;
    }
    let noun = _the_noun_the_line_named(line)?;
    let mods: Vec<&str> = mods.iter().map(|w| w.as_str()).collect();
    Some((noun.clone(), format!("{} {noun}", mods.join(" "))))
}

/// Python: bot/memory/repair.py:371 `_the_relation(word)`
/// The relation a word names, allowing for the way it is typed: "grans" is "gran".
pub fn _the_relation(word: &str) -> Option<String> {
    let mut plain = text::strip_chars(&text::strip(word).to_lowercase(), "'").to_string();
    if RELATIONS.contains(&plain) {
        return Some(plain);
    }
    if plain.ends_with("'s") {
        plain.truncate(plain.len() - 2);
    } else if plain.ends_with('s') {
        plain.truncate(plain.len() - 1);
    }
    if RELATIONS.contains(&plain) {
        return Some(plain);
    }
    if plain.ends_with('\'') {
        plain.truncate(plain.len() - 1);
    }
    RELATIONS.contains(&plain).then_some(plain)
}

/// Python: bot/memory/repair.py:392 `_the_person_the_line_named(line, thing)`
/// The person a correction of who is about: the person the line named that the correction does
/// not.
pub fn _the_person_the_line_named(line: &str, thing: &str) -> Option<String> {
    let named: HashSet<String> = LETTERS.findall_str(&thing.to_lowercase()).iter().filter_map(|w| _the_relation(w)).collect();
    if named.is_empty() {
        return None;
    }
    LETTERS.findall_str(&line.to_lowercase()).iter().filter_map(|w| _the_relation(w)).find(|person| !named.contains(person))
}

/// Python: bot/memory/repair.py:414 `_swapped(line, old, thing)`
/// The ask they would have said if they had said it right the first time, or "".
pub fn _swapped(line: &str, old: &str, thing: &str) -> String {
    let said = text::squash(line);
    let mut new = text::strip(thing).to_string();
    // A relation is often typed with the possessive on it: "my grans 84".
    let tail = if RELATIONS.contains(old) { r"(?:'s|s)?" } else { "" };
    let pattern = Re::new(&format!(r"\b{}{tail}\b", re::escape(old)), re::I | re::U);
    let Some(found) = pattern.search(&said) else { return String::new() };
    // The determiner belongs to the sentence, not to the thing, unless the thing opens the line.
    if !text::split_ref(&said[..found.start()]).is_empty() && A_LEADING_DETERMINER.match_(&new).is_some() {
        new = A_LEADING_DETERMINER_WORD.subn("", &new, 1);
    }
    let out = format!("{}{new}{}", &said[..found.start()], &said[found.end()..]);
    let out = _no_repeat(&text::squash(&out));
    if out.to_lowercase() == said.to_lowercase() { String::new() } else { out }
}

/// Python: bot/memory/repair.py:440 `_no_repeat(line)`
/// The line without a run of words the swap made twice: "a new new shower" -> "a new shower".
pub fn _no_repeat(line: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for word in text::split_ref(line) {
        if out.last().is_some_and(|last| last.to_lowercase() == word.to_lowercase()) {
            continue;
        }
        out.push(word);
    }
    out.join(" ")
}

/// Python: bot/memory/repair.py:457 `_the_answer(line, memory)`
/// What the answers say about the ask they would have made, read as a line of its own with the
/// facts the person stated, or None. What the answers noted about it lands in the chat.
pub fn _the_answer(line: &str, memory: &mut Memory) -> Option<String> {
    let mut fresh = pyv::a_fresh_memory(memory);
    fresh.understood = Some(understand(line, Some(&Memory::new())));
    let reply = pyv::attempt(|| crate::modules::answers::respond(line, &mut fresh));
    pyv::shared_back(memory, &fresh);
    let reply = reply??;
    if text::strip(&reply).is_empty() {
        return None;
    }
    for key in ["entry_answer", "entry_answers"] {
        if let Some(value) = fresh.get(key) {
            memory.set(key, value.clone());
        }
    }
    Some(text::strip(&reply).to_string())
}

/// Python: bot/memory/repair.py:495 `_the_plain_answer(text, memory)`
/// What the answers say about the line as it stands, read with the chat it is in.
pub fn _the_plain_answer(text_: &str, memory: &mut Memory) -> Option<String> {
    let reply = pyv::attempt(|| crate::modules::answers::respond(text_, memory))?;
    let said = text::squash(&reply.unwrap_or_default());
    if said.is_empty() { None } else { Some(said) }
}

/// Python: bot/memory/repair.py:507 `_about_the_same_thing(first, second)`
/// Whether two answers are about the same thing, by the content words they share.
pub fn _about_the_same_thing(first: &str, second: &str) -> bool {
    let left: HashSet<String> = keywords(first).into_iter().filter(|w| text::len(w) > 3).collect();
    keywords(second).iter().any(|w| text::len(w) > 3 && left.contains(w))
}

/// Python: bot/memory/repair.py:521 `_the_shape_the_swap_had(memory, line, old, thing)`
/// The corrected line written the way the store has always written it: the word at the end of
/// the line swapped for the new thing, unless that word is the state the thing is in.
pub fn _the_shape_the_swap_had(memory: &Memory, line: &str, old: &str, thing: &str) -> String {
    let last = pyv::str_or_empty(&last_user(memory, 1));
    let tail = if last.is_empty() { None } else { _the_thing_the_last_line_named(&last) };
    if let Some(tail) = tail.filter(|t| !t.is_empty()) {
        if _a_state_not_a_thing(&last, &tail) {
            return line.to_string();
        }
        if tail != old && _a_different_thing(&tail, thing) {
            let older = _swapped(&last, &tail, thing);
            if !older.is_empty() {
                return older;
            }
        }
    }
    line.to_string()
}

/// Python: bot/memory/repair.py:550 `_a_state_not_a_thing(line, tail)`
/// True when the word the line ends on is what the thing is doing or is, not a thing itself.
pub fn _a_state_not_a_thing(line: &str, tail: &str) -> bool {
    let pattern = format!(
        r"\b(?:is|are|was|were|isn'?t|aren'?t|wasn'?t|weren'?t|feels?|seems?|looks?|gets?|got|goes|went|has|have|had|keeps?|stays?|stops?)\s+{}\b",
        re::escape(tail)
    );
    Re::new(&pattern, re::I | re::U).is_match(line)
}

/// Python: bot/memory/repair.py:562 `_keep_the_swap(memory, line, old, thing)`
/// The topic and the plan are the corrected line, so the turns after it are about the thing
/// they meant rather than the one they took back.
pub fn _keep_the_swap(memory: &mut Memory, line: &str, old: &str, thing: &str) {
    let line = _the_shape_the_swap_had(memory, line, old, thing);
    if let Some(record) = topics::current(memory).cloned() {
        let mut record = record;
        record.set_key("text", line.as_str());
        record.set_key("words", Value::from(keywords(&line)));
        record.set_key("corrected", thing);
        memory.set("topic", record.clone());
        // `topics[-1] is not record`: the record replaces it (or is it, and has just changed).
        if let Some(Value::List(held)) = memory.get_mut("topics") {
            if let Some(last) = std::sync::Arc::make_mut(held).last_mut() {
                *last = record;
            }
        }
    }
    let Some(known) = memory.get_mut("user").filter(|known| known.is_dict()) else { return };
    for key in ["plan", "plans"] {
        match known.get(key).cloned() {
            Some(Value::Str(held)) => {
                let moved = _swapped(&held, old, thing);
                if !moved.is_empty() {
                    known.set_key(key, moved);
                }
            }
            Some(Value::List(held)) => {
                let moved: Vec<Value> = held
                    .iter()
                    .map(|one| {
                        let one = pyv::str_of(one);
                        let moved = _swapped(&one, old, thing);
                        Value::from(if moved.is_empty() { one } else { moved })
                    })
                    .collect();
                known.set_key(key, Value::from(moved));
            }
            _ => {}
        }
    }
    // The first six entries of the log, newest of them first.
    let Some(log) = memory.get_mut("facts_log").filter(|log| log.truthy()) else { return };
    let Some(entries) = log.items_mut() else {
        // `list(memory.get("facts_log") or [])` of a truthy non-list (a dict's keys, a str's
        // characters) holds no dict to move.
        let _ = pyv::iter(log);
        return;
    };
    let first = entries.len().min(6);
    for entry in entries[..first].iter_mut().rev() {
        if !entry.is_dict() || !matches!(pyv::get(entry, "key").as_str(), Some("plan") | Some("plans")) {
            continue;
        }
        if let Value::Str(value) = pyv::get(entry, "value").clone() {
            let moved = _swapped(&value, old, thing);
            if !moved.is_empty() {
                entry.set_key("value", moved);
            }
        }
    }
}

/// The words of a line as a set.
fn word_set(said: &str) -> HashSet<String> {
    WORDS.findall_str(&text::squash(&said.to_lowercase())).into_iter().collect()
}

/// Python: bot/memory/repair.py:595 `_the_paragraph_already_given(answer, memory, old, thing)`
/// True when this answer is the one the chat has just read, with the thing's word changed.
pub fn _the_paragraph_already_given(answer: &str, memory: &Memory, old: &str, thing: &str) -> bool {
    let said = word_set(answer);
    if said.len() < 25 {
        return false;
    }
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(4)..].iter().rev() {
        if text::split_ref(&turn.bot).len() < 20 {
            continue;
        }
        let other = word_set(&turn.bot);
        if other.is_empty() {
            continue;
        }
        let shared = said.intersection(&other).count() as f64 / said.union(&other).count() as f64;
        if shared < 0.8 {
            continue;
        }
        let (new_words, old_words) = (keywords(thing), keywords(old));
        if new_words.is_empty() || old_words.is_empty() {
            continue;
        }
        if !_names(&said, &new_words) || !_names(&other, &old_words) {
            continue;
        }
        if _names(&said, &old_words) {
            continue; // the paragraph still names the thing they took back
        }
        return true;
    }
    false
}

/// Python: bot/memory/repair.py:637 `_names(words, wanted)`
/// True when a set of words of an answer holds one of the words, plural or possessive.
pub fn _names(words: &HashSet<String>, wanted: &[String]) -> bool {
    wanted.iter().map(|one| one.to_lowercase()).filter(|one| !one.is_empty()).any(|one| {
        words.iter().any(|word| *word == one || *word == format!("{one}s") || text::rstrip_chars(word, "s") == one)
    })
}

/// Python: bot/memory/repair.py:653 `_without_the_repeat_lead(answer)`
/// The answer itself, without a lead-in that already names the thing they meant.
pub fn _without_the_repeat_lead(answer: &str) -> String {
    let said = REPEAT_LEAD.sub("", &text::squash(answer));
    if text::len(&said) >= 40 { said } else { answer.to_string() }
}

/// Python: bot/memory/repair.py:659 `_still_the_old_thing(answer, old, thing)`
/// True when the store's own answer is still about the thing they took back.
pub fn _still_the_old_thing(answer: &str, old: &str, thing: &str) -> bool {
    let said = text::squash(&answer.to_lowercase());
    if old.is_empty() {
        return false;
    }
    let words: HashSet<String> = LETTERS.findall_str(&said).into_iter().collect();
    if !keywords(old).iter().any(|w| words.contains(&w.to_lowercase())) {
        return false;
    }
    let opening: HashSet<String> = LETTERS.findall_str(&text::split_ref(&said).into_iter().take(12).collect::<Vec<_>>().join(" ")).into_iter().collect();
    !keywords(thing).iter().any(|w| opening.contains(&w.to_lowercase()))
}

/// Python: bot/memory/repair.py:680 `_bare(thing)`
/// The thing without the word in front of it, for a sentence that has one of its own.
pub fn _bare(thing: &str) -> String {
    let flat = text::squash(thing);
    let words = text::split_ref(text::strip_chars(&flat, " .,"));
    if words.len() > 1 && DETERMINERS[..9].contains(&words[0].to_lowercase().as_str()) {
        return words[1..].join(" ");
    }
    words.join(" ")
}

/// Python: bot/memory/repair.py:689 `_said_before(answer, memory)`
/// True when this answer is one the chat has already read (its opening, in any bot turn).
pub fn _said_before(answer: &str, memory: &Memory) -> bool {
    if memory.turns.is_empty() {
        return false;
    }
    let flat = text::squash(&answer.to_lowercase());
    let opening = text::split_ref(&flat).into_iter().take(10).collect::<Vec<_>>().join(" ");
    if opening.is_empty() {
        return false;
    }
    memory.turns.iter().any(|turn| text::squash(&turn.bot.to_lowercase()).contains(&opening))
}

/// Python: bot/memory/repair.py:738 `_the_same_words(one, other)`
/// Whether two sentences are the same advice once their figures are taken out of them.
pub fn _the_same_words(one: &str, other: &str) -> bool {
    let stemmed = |sentence: &str| -> Vec<String> {
        text::split(&FIGURE_OR_WORD.sub("#", &sentence.to_lowercase()))
            .into_iter()
            .map(|w| if text::len(&w) > 3 && w.ends_with('s') { w[..w.len() - 1].to_string() } else { w })
            .collect()
    };
    stemmed(one) == stemmed(other)
}

/// Python: bot/memory/repair.py:752 `_the_figure_in(line)`
/// The last figure of the line, and the unit said with it ("" when it is no unit of a figure).
pub fn _the_figure_in(line: &str) -> Option<(Match<'_>, String)> {
    let mut found = None;
    for one in A_FIGURE.finditer(line) {
        let unit = one.name_str("unit").to_lowercase();
        let unit = if FIGURE_UNITS.contains(&unit) { unit } else { String::new() };
        found = Some((one, unit));
    }
    found
}

/// Python: bot/memory/repair.py:766 `_the_figure_they_meant(text, memory)`
/// The answer worked out again for the figure they put right, or "": only the sentences whose
/// arithmetic moved, and only when the advice itself is word for word the same.
pub fn _the_figure_they_meant(text_: &str, memory: &mut Memory) -> String {
    let said_line = text::squash(text_);
    let Some(found) = A_FIGURE_REPAIR.match_(&said_line) else { return String::new() };
    let figure = found.name_str("figure").to_string();
    let unit = found.name_str("unit").to_lowercase();
    if !unit.is_empty() && !FIGURE_UNITS.contains(&unit) {
        return String::new();
    }
    let before = text::squash(&pyv::str_or_empty(&last_user(memory, 1)));
    if before.is_empty() {
        return String::new();
    }
    let Some((held, held_unit)) = _the_figure_in(&before) else { return String::new() };
    let held_figure = held.name_str("figure").to_string();
    if held_figure == figure {
        return String::new(); // the same figure said again corrects nothing
    }
    if !unit.is_empty() && unit != held_unit {
        return String::new();
    }
    let (start, end) = held.name_span("figure").unwrap();
    let line = format!("{}{figure}{}", &before[..start], &before[end..]);
    let in_hand = topics::answer_in_hand(memory);
    let said = if in_hand.is_dict() { pyv::str_or_empty(pyv::get(&in_hand, "bot")) } else { String::new() };
    if said.is_empty() || !_the_answer_says(&said, &held_figure) {
        return String::new(); // the answer on the screen was not worked out from that figure
    }
    // The answers' note goes into a scratch copy, kept only when this is the turn. The facts the
    // reader writes into are the chat's own objects in Python, so they come back either way.
    let mut scratch = memory.clone();
    let redone = _the_answer(&line, &mut scratch);
    pyv::shared_back(memory, &scratch);
    let Some(redone) = redone else { return String::new() };
    let changed = _the_figures_that_moved(&said, &redone);
    if changed.is_empty() {
        return String::new();
    }
    _keep_the_corrected_answer(memory, &scratch, &line);
    changed
}

/// Python: bot/memory/repair.py:823 `_the_answer_says(said, figure)`
/// Whether the answer on the screen was worked out from this figure, in figures or words.
pub fn _the_answer_says(said: &str, figure: &str) -> bool {
    let word = THE_WORD_FOR.get(figure).cloned().unwrap_or_default();
    [figure.to_string(), word].iter().filter(|s| !s.is_empty()).any(|spelling| {
        Re::new(&format!(r"(?<![\d.]){}(?![\d])", re::escape(spelling)), re::I | re::U).is_match(said)
    })
}

/// Python: bot/memory/repair.py:835 `_the_figures_that_moved(said, redone)`
/// The sentences of the answer worked out again that carry a figure the first one did not.
pub fn _the_figures_that_moved(said: &str, redone: &str) -> String {
    let split = |s: &str| -> Vec<String> { SENTENCE_END.split(&text::squash(s)).into_iter().filter(|one| !one.is_empty()).collect() };
    let (before, after) = (split(said), split(redone));
    if before.is_empty() || before.len() != after.len() {
        return String::new();
    }
    let mut changed: Vec<&String> = Vec::new();
    for (old, new) in before.iter().zip(&after) {
        if !_the_same_words(old, new) {
            return String::new();
        }
        if old != new {
            changed.push(new);
        }
    }
    if changed.is_empty() {
        return String::new();
    }
    let said = changed.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" ");
    text::upper_first(text::strip(&said))
}

/// Python: bot/memory/repair.py:858 `_keep_the_corrected_answer(memory, scratch, line)`
/// The answers' note and the topic are the line with the right figure in it.
pub fn _keep_the_corrected_answer(memory: &mut Memory, scratch: &Memory, line: &str) {
    for key in ["entry_answer", "entry_answers"] {
        if let Some(value) = scratch.get(key) {
            memory.set(key, value.clone());
        }
    }
    if let Some(mut record) = topics::current(memory).cloned() {
        record.set_key("text", line);
        record.set_key("words", Value::from(keywords(line)));
        memory.set("topic", record);
    }
}

/// Python: bot/memory/repair.py:947 `_a_fact_of_the_person(entry)`
/// The first fact the store holds about somebody, as a person would say it: "84".
pub fn _a_fact_of_the_person(entry: &Value) -> String {
    let age = pyv::get(entry, "age");
    if age.truthy() {
        return pyv::str_of(age);
    }
    let allergies = pyv::get(entry, "allergies");
    if allergies.truthy() {
        return format!("allergic to {}", _a_list_of(&pyv::iter(allergies)));
    }
    String::new()
}

/// Python: bot/memory/repair.py:958 `_a_list_of(things)`
/// `nuts` -> "nuts", `nuts` and `dairy` -> "nuts and dairy".
pub fn _a_list_of(things: &[Value]) -> String {
    let said: Vec<String> =
        things.iter().map(|one| text::strip(&pyv::str_of(one)).to_string()).filter(|one| !one.is_empty()).collect();
    match said.len() {
        0 => String::new(),
        1 => said[0].clone(),
        n => format!("{} and {}", said[..n - 1].join(", "), said[n - 1]),
    }
}

/// Python: bot/memory/repair.py:968 `_the_fact_moved_to_the_person_they_meant(memory, old, thing, text)`
/// The facts filed under the person they took back, moved onto the person they meant, through
/// the store's own merge. The fact said back, or "".
pub fn _the_fact_moved_to_the_person_they_meant(memory: &mut Memory, old: &str, thing: &str, text_: &str) -> String {
    let wanted = super::facts::a_person_word(&_bare(thing));
    if wanted.is_empty() || wanted == super::facts::a_person_word(old) {
        return String::new();
    }
    let people = others(memory).items().to_vec();
    for entry in people {
        if super::facts::a_person_word(&pyv::str_or_empty(pyv::get(&entry, "who"))) != super::facts::a_person_word(old) {
            continue;
        }
        let fact = _a_fact_of_the_person(&entry);
        if fact.is_empty() {
            return String::new();
        }
        let held = others(memory).items_mut().unwrap();
        let at = held.iter().position(|one| *one == entry).unwrap();
        held.remove(at);
        let mut moved = Value::dict();
        for (key, value) in entry.pairs() {
            if key.as_str() != Some("who") {
                moved.set_key(key.clone(), value.clone());
            }
        }
        moved.set_key("who", wanted.as_str());
        remember_other(memory, &moved, text_);
        return fact;
    }
    String::new()
}

/// Python: bot/memory/repair.py:994 `_the_fact_moved_line(fact, old, thing, text, memory)`
/// The reply to a correction of who, when the fact is the only thing worth saying.
pub fn _the_fact_moved_line(fact: &str, old: &str, thing: &str, text_: &str, memory: &Memory) -> Option<String> {
    if fact.is_empty() {
        return None;
    }
    let said = pyv::format(&pyv::pick(&THE_FACT_MOVED, memory, text_), &[("new", &_bare(thing)), ("old", &_bare(old)), ("fact", fact)]);
    Some(if said.ends_with(['.', '!', '?']) { said } else { said + "." })
}

/// What `_the_swap_in_the_line` finds: the last line, the words that could be the old thing,
/// the new thing, and the pointer's (word, thing) when the line refined the thing.
pub type Swap = (String, Vec<String>, String, Option<(String, String)>);

/// Python: bot/memory/repair.py:1003 `_the_swap_in_the_line(text, memory)`
/// What the line swaps, or None when the line is no correction of a thing or there is no
/// earlier line to correct. The old thing is tried in order: the person the last line named,
/// the thing its reading names, the word at the end of the line.
pub fn _the_swap_in_the_line(text_: &str, memory: &Memory) -> Option<Swap> {
    let thing = _the_thing(text_)?;
    if AN_ASK_OPENS.match_(&thing).is_some() || text::rstrip(&thing).ends_with('?') {
        return None;
    }
    let last = pyv::str_or_empty(&last_user(memory, 1));
    if last.is_empty() {
        return None;
    }
    if let Some(refined) = _the_pointer_refines(&thing, &last) {
        // "no the lower one" after "my back hurts" is "my lower back hurts".
        let (old, thing) = refined.clone();
        return Some((last, vec![old], thing, Some(refined)));
    }
    // A correction of a person moves the person, not the last noun of the line.
    let mut olds: Vec<String> = _the_person_the_line_named(&last, &thing).into_iter().collect();
    for one in [_the_thing_the_reading_named(&last), _the_thing_the_last_line_named(&last).unwrap_or_default()] {
        if !one.is_empty() && !olds.contains(&one) {
            olds.push(one);
        }
    }
    if olds.is_empty() {
        return None;
    }
    Some((last, olds, thing, None))
}

/// Python: bot/memory/repair.py:1038 `keep_the_correction(text, memory)`
/// Keep the swap the line made, for a correction another reader answered.
pub fn keep_the_correction(text_: &str, memory: &mut Memory) {
    let Some((last, olds, thing, _refined)) = _the_swap_in_the_line(text_, memory) else { return };
    for one in &olds {
        if !_a_different_thing(one, &thing) {
            continue;
        }
        let swapped = _swapped(&last, one, &thing);
        if !swapped.is_empty() {
            _keep_the_swap(memory, &swapped, one, &thing);
            return;
        }
    }
}

/// `said` with a full stop when it has no end of its own.
fn ended(said: String) -> String {
    if said.ends_with(['.', '!', '?']) { said } else { said + "." }
}

/// The thing for a sentence that puts its own determiner in front of it, and as it opens one.
fn held_and_shown(thing: &str) -> (String, String) {
    let shown = text::upper_first(thing);
    let mut held = _bare(thing);
    if LEADS.contains(&text::split_ref(&shown)[0].to_lowercase().as_str()) {
        let mut chars = shown.chars();
        held = chars.next().map(|c| c.to_lowercase().chain(chars).collect()).unwrap_or_default();
    }
    (held, shown)
}

/// `s[:1].lower() + s[1:]`.
fn lower_first(s: &str) -> String {
    let mut chars = s.chars();
    chars.next().map(|c| c.to_lowercase().chain(chars).collect()).unwrap_or_default()
}

/// Python: bot/memory/repair.py:1061 `the_thing_they_meant(text, memory)`
/// The answer about the thing they said they meant, or None. Called when the rest of the store
/// has nothing to say about the line.
pub fn the_thing_they_meant(text_: &str, memory: &mut Memory) -> Option<String> {
    let figure = _the_figure_they_meant(text_, memory);
    if !figure.is_empty() {
        return Some(figure);
    }
    let thing = _the_thing(text_)?;
    if AN_ASK_OPENS.match_(&thing).is_some() || text::rstrip(&thing).ends_with('?') {
        // The correction is the ask itself: answer it against the thread.
        let answer = _without_the_repeat_lead(&_the_answer(&thing, memory)?);
        if _said_before(&answer, memory) {
            return None;
        }
        return Some(ended(answer));
    }
    let (last, olds, thing, refined) =
        _the_swap_in_the_line(text_, memory).expect("TypeError: cannot unpack non-iterable NoneType object");
    let held = olds[0].clone();
    // The fact the last line stated belongs to the person they meant.
    let fact = _the_fact_moved_to_the_person_they_meant(memory, &held, &thing, text_);
    let (mut answer, mut line, mut old) = (String::new(), String::new(), String::new());
    for one in &olds {
        if !_a_different_thing(one, &thing) {
            continue;
        }
        let swapped = _swapped(&last, one, &thing);
        if swapped.is_empty() {
            continue;
        }
        if let Some(found) = _the_answer(&swapped, memory) {
            (answer, line, old) = (found, swapped, one.clone());
            break;
        }
    }
    if answer.is_empty() {
        return _the_fact_moved_line(&fact, &held, &thing, text_, memory);
    }
    let answer = _without_the_repeat_lead(&answer);
    let mut plain = _the_plain_answer(text_, memory);
    if plain.as_ref().is_some_and(|p| !(_about_the_same_thing(p, &thing) || _about_the_same_thing(p, &answer))) {
        // The reading of the line whole matched a word of the frame, not the thing they named.
        plain = None;
    }
    if plain.as_ref().is_some_and(|p| _without_the_repeat_lead(p) == answer) {
        // The store's own reading already answers the corrected ask; the swap is kept even so.
        _keep_the_swap(memory, &line, &old, &thing);
        return None;
    }
    if let Some(p) = &plain {
        if text::len(p) > *A_REAL_ANSWER && !HEDGE.is_match(p) && !_still_the_old_thing(p, &old, &thing) {
            // The line read as it stands has an answer of its own, given as it stands.
            if _said_before(p, memory) {
                return None;
            }
            _keep_the_swap(memory, &line, &old, &thing);
            return Some(ended(p.clone()));
        }
    }
    if plain.as_deref() == Some(answer.as_str()) {
        _keep_the_swap(memory, &line, &old, &thing);
        return None;
    }
    _keep_the_swap(memory, &line, &old, &thing);
    if refined.is_some() && !_said_before(&answer, memory) {
        // They narrowed the thing already on the table: the advice answers as it stands.
        return Some(ended(answer));
    }
    let (held_thing, shown) = held_and_shown(&thing);
    if _the_paragraph_already_given(&answer, memory, &old, &thing) {
        let said = pyv::format(&pyv::pick(&SAME_ADVICE, memory, text_), &[("said", &shown), ("thing", &held_thing), ("old", &_bare(&old))]);
        return Some(ended(said));
    }
    if _said_before(&answer, memory) {
        let said = pyv::format(
            &pyv::pick(&SAME_AGAIN, memory, text_),
            &[("said", &shown), ("thing", &held_thing), ("old", &_bare(&old)), ("answer", &lower_first(&answer))],
        );
        return Some(ended(said));
    }
    let shape = pyv::pick(&SWAPPED, memory, text_);
    let mut shown = shown;
    if !shape.starts_with("{thing}") && LEADS.contains(&text::split_ref(&shown)[0].to_lowercase().as_str()) {
        // Said inside a sentence, the determiner is not a capital.
        shown = lower_first(&shown);
    }
    let said = pyv::format(&shape, &[("thing", &shown)]);
    Some(text::strip(&format!("{said} {answer}")).to_string())
}
