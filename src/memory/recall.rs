//! Port of `bot/memory/recall.py`: a line that points back at an earlier turn ("what did i say
//! about my budget", "remember my dog's name?"), and what it asks the store for.

#![allow(non_upper_case_globals)]

use crate::memory::topics::{stem, words};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understanding::{breeds, rx_flags, t};
use crate::understood::LooksBack;
use std::collections::{BTreeSet, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.memory.recall.";

static OPENS_WITH: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "OPENS_WITH"));
static SUBJECT_CONTRACTION: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUBJECT_CONTRACTION"));
static QUESTION_START: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "QUESTION_START"));
static REMEMBERING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "REMEMBERING"));
static LEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LEAD"));
static RETURNING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RETURNING"));
static WHY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHY"));
static KEEPING_TRACK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "KEEPING_TRACK"));
static LIMIT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LIMIT"));
static ADVICE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ADVICE"));
static SAY_NOW: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SAY_NOW"));
static RECALLING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RECALLING"));
static ACTION_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ACTION_ASK"));
static HOW_TO_TELL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HOW_TO_TELL"));
static TELLS_APART: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TELLS_APART"));
static DRIVE_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DRIVE_ASK"));
static SPENT_ON_A_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SPENT_ON_A_THING"));
static SPENDING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SPENDING"));
static CALL_NAME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CALL_NAME"));
static A_PERSON_BY_WHAT_THEY_DO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_PERSON_BY_WHAT_THEY_DO"));
static AVOID: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "AVOID"));
static USER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "USER"));
static SCAFFOLD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SCAFFOLD"));
static TIME_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TIME_WORDS"));
static WANTS_THE_FACT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WANTS_THE_FACT"));
static EMPLOYER_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "EMPLOYER_WORDS"));
static HOW_KNOW: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HOW_KNOW"));
static ME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ME"));
static KNOWING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "KNOWING"));
static KNOWN_OBJECT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "KNOWN_OBJECT"));
static LIVING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LIVING"));
static PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PEOPLE"));
static LIMIT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LIMIT_WORDS"));
static RECALL_MODIFIERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RECALL_MODIFIERS"));
static HOW_OLD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HOW_OLD"));
static DECIDING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DECIDING"));
static DECIDED_ABOUT: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "DECIDED_ABOUT"));  // t, 2 items
static WORLD_SUBJECT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WORLD_SUBJECT"));
static PERSONAL: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "PERSONAL"));  // t, 9 items
static CALL_A_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CALL_A_THING"));
static A_PERSON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_PERSON"));
static LOOKS_BACK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LOOKS_BACK"));
static _ASKING_FRAME: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_ASKING_FRAME"));
/// `SUBJECTS`: (key, the words that name it), in Python's order.
static SUBJECTS: LazyLock<Vec<(String, Vec<String>)>> =
    LazyLock::new(|| t::value(M, "SUBJECTS").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].strs())).collect());

/// `_a_contraction_opens(text)`: "i'm", "it's" opening the line.
fn _a_contraction_opens(text_: &str) -> bool {
    let Some(found) = OPENS_WITH.match_(text_) else { return false };
    let g = found.str(1).to_lowercase().replace('\u{2019}', "'");
    SUBJECT_CONTRACTION.contains(g.split('\'').next().unwrap_or(""))
}

/// `_asked(text, low)`: the line is a question.
fn _asked(text_: &str, low: &[String]) -> bool {
    if text_.contains('?') {
        return true;
    }
    if low.is_empty() {
        return false;
    }
    if QUESTION_START.contains(&low[0]) && _a_contraction_opens(text_) {
        return false;
    }
    if REMEMBERING.match_(&low[0]).is_some() {
        return true;
    }
    let two = low[..low.len().min(2)].join(" ");
    if matches!(two.as_str(), "do you" | "did you" | "you know" | "did i" | "do i" | "can you") {
        return true;
    }
    QUESTION_START.contains(&low[0])
}

/// `_asked_them(line)`.
fn _asked_them(line: &str) -> bool {
    let low = words(line);
    if low.is_empty() {
        return false;
    }
    if matches!(low[0].as_str(), "do" | "did" | "are" | "can" | "could" | "would" | "will") && low.iter().take(2).any(|w| w == "you") {
        return false;
    }
    true
}

/// `_key(word)`: the store key a word names ("dog" -> pet, "budget" -> budget).
fn _key(word: &str) -> Option<String> {
    if !breeds::the_kind_of(word).is_empty() {
        return Some("pet".to_string());
    }
    for (key, names) in SUBJECTS.iter() {
        if names.iter().any(|n| n == word) {
            return Some(key.clone());
        }
        if text::len(word) > 4
            && names.iter().any(|name| {
                let head = text::slice(name, 0, Some(4));
                word.starts_with(&head) && (head == "alle" || head == "budg")
            })
        {
            return Some(key.clone());
        }
    }
    None
}

/// `_stems(said)`.
fn _stems(said: &str) -> BTreeSet<String> {
    words(said).into_iter().filter(|w| !w.is_empty()).map(|w| stem(&w)).collect()
}

/// `_the_slot(line)`: "the dog", "the budget".
fn _the_slot(line: &str) -> bool {
    let low = words(line);
    (0..low.len()).any(|i| low[i] == "the" && i + 1 < low.len() && _key(&low[i + 1]).is_some())
}

/// `_without_the_lead(text)`: the line without its lead-in, and the key a "back to" names.
fn _without_the_lead(text_: &str) -> (String, Option<String>) {
    let mut line = text::strip_chars(&LEAD.sub("", text_), " ,.!?;:-").to_string();
    let mut named: Option<String> = None;
    for _ in 0..3 {
        let Some(cut) = RETURNING.match_(&line) else { break };
        let said = words(cut.as_str());
        if let Some(last) = said.last() {
            named = _key(last).or(named);
        }
        line = text::strip_chars(&line[cut.end()..], " ,.!?;:-").to_string();
    }
    (line, named)
}

/// `ask(text, why=False)`: the store key a recall question asks for, or None.
pub fn ask(text_: &str, why: bool) -> Option<String> {
    let (line, returning) = _without_the_lead(&text::squash(text_));
    let mut line = line;
    if line.is_empty() || text::split_ref(&line).len() > 12 {
        return None;
    }
    if WHY.is_match(&line) && !why {
        return None;
    }
    if KEEPING_TRACK.is_match(&line) {
        return None;
    }
    let limit = LIMIT.is_match(&line);
    if (ADVICE.is_match(&line) || SAY_NOW.is_match(&line)) && !RECALLING.is_match(&line) && !limit {
        return None;
    }
    if ACTION_ASK.is_match(&line) && !RECALLING.is_match(&line) && !limit && !crate::understanding::rx(r"\bhow (?:much|many)\b|\bfor a living\b").is_match(&line) {
        return None;
    }
    if let Some(how_to) = HOW_TO_TELL.search(&line) {
        if TELLS_APART.is_match(&line[how_to.end()..]) {
            return None;
        }
    }
    if DRIVE_ASK.is_match(&line) {
        return Some("car".to_string());
    }
    if SPENDING.is_match(&line) {
        if let Some(tail) = SPENT_ON_A_THING.search(&line) {
            let head = text::strip_chars(&line[..tail.start()], " ,.").to_string();
            if !head.is_empty() {
                line = head;
            }
        }
    }
    let low = words(&line);
    if low.is_empty() || !_asked(&line, &low) {
        return None;
    }
    if CALL_NAME.is_match(&line) && !asks_for_a_word(&line) && !A_PERSON_BY_WHAT_THEY_DO.is_match(&line) {
        return Some("name".to_string());
    }
    if AVOID.is_match(&line) && USER.is_match(&line) {
        return Some("allergy".to_string());
    }
    if rx_flags(r"\bdate of birth\b|\bbirth date\b", crate::py::re::I).is_match(&line) {
        return Some("birthday".to_string());
    }
    let mut content: Vec<String> = low.iter().filter(|w| !SCAFFOLD.contains(*w) && !TIME_WORDS.contains(*w) && text::len(w) > 1 && REMEMBERING.match_(w).is_none()).cloned().collect();
    if !content.is_empty() && !WANTS_THE_FACT.is_match(&line) && content.iter().all(|w| EMPLOYER_WORDS.contains(w)) {
        return None;
    }
    if content.is_empty() {
        if returning.is_some() {
            return returning;
        }
        if limit {
            return Some("limit".to_string());
        }
        if HOW_KNOW.is_match(&line) {
            return None;
        }
        if ME.is_match(&line) {
            return Some("me".to_string());
        }
        if USER.is_match(&line) && _asked_them(&line) && KNOWING.is_match(&line) && KNOWN_OBJECT.is_match(&line) {
            return Some("me".to_string());
        }
        return None;
    }
    if !USER.is_match(&line) && !(RECALLING.is_match(&line) && _the_slot(&line)) {
        return None;
    }
    if LIVING.is_match(&line) {
        return Some("job".to_string());
    }
    if content.iter().any(|w| PEOPLE.contains(w)) {
        return None;
    }
    if limit && content.iter().all(|w| LIMIT_WORDS.contains(w)) {
        return Some("limit".to_string());
    }
    let mut modifierless = Vec::new();
    for (at, word) in content.iter().enumerate() {
        let after = content.get(at + 1).map(String::as_str).unwrap_or("");
        if RECALL_MODIFIERS.contains(word) && !after.is_empty() && _key(after).is_some() && !((word == "old" || word == "older") && HOW_OLD.is_match(&line)) {
            continue;
        }
        modifierless.push(word.clone());
    }
    content = modifierless;
    let mut keys = Vec::new();
    for word in &content {
        keys.push(_key(word)?);
    }
    if DECIDING.is_match(&line) && !RECALLING.is_match(&line) && DECIDED_ABOUT.iter().any(|k| keys.contains(k)) {
        return None;
    }
    if WORLD_SUBJECT.match_(&line).is_some() && !RECALLING.is_match(&line) && keys.iter().any(|k| !PERSONAL.contains(k)) {
        return None;
    }
    SUBJECTS.iter().find(|(k, _)| keys.contains(k)).map(|(k, _)| k.clone())
}

/// `asks_for_a_word(line)`: "what do you call a ...".
pub fn asks_for_a_word(line: &str) -> bool {
    CALL_A_THING.is_match(line) && !(A_PERSON.is_match(line) || A_PERSON_BY_WHAT_THEY_DO.is_match(line))
}

/// `looks_back(text)`: the cue that points back ("you said", "earlier"), or "".
pub fn looks_back(text_: &str) -> String {
    LOOKS_BACK.search(&text::squash(&text_.to_lowercase())).map(|m| text::strip(m.as_str()).to_string()).unwrap_or_default()
}

/// `the_earlier_turn(text, memory=None)`: the earlier lines a look-back cue points at.
pub fn the_earlier_turn(text_: &str, memory: &Memory) -> Option<LooksBack> {
    let cue = looks_back(text_);
    if cue.is_empty() {
        return None;
    }
    let thing = words(text_).into_iter().filter(|w| !_ASKING_FRAME.contains(w)).collect::<Vec<_>>().join(" ");
    let want = _stems(&thing);
    // Python: `turns` is only a list once a turn has been answered; an empty list reads the same
    if memory.turns.is_empty() {
        return None;
    }
    let turns = &memory.turns;
    if want.is_empty() {
        let said: Vec<String> = turns.iter().rev().filter(|t_| !t_.user.is_empty()).map(|t_| t_.user.clone()).take(3).collect();
        if said.is_empty() {
            return None;
        }
        return Some(LooksBack { cue, thing: ask(text_, false).unwrap_or_default(), said });
    }
    let mut everywhere: BTreeSet<String> = BTreeSet::new();
    for turn in turns {
        everywhere.extend(_stems(&turn.user));
    }
    let unsaid: HashSet<&String> = want.iter().filter(|w| !everywhere.contains(*w)).collect();
    let mut said: Vec<String> = Vec::new();
    for turn in turns.iter().rev() {
        let line = &turn.user;
        let have = if line.is_empty() { BTreeSet::new() } else { _stems(line) };
        let common = want.iter().filter(|w| have.contains(*w)).count();
        if common >= 2 && want.iter().filter(|w| !have.contains(*w)).all(|w| unsaid.contains(w)) {
            said.push(line.clone());
        } else if !line.is_empty() && want.iter().all(|w| have.contains(w)) {
            said.push(line.clone());
        }
        if said.len() == 3 {
            break;
        }
    }
    if said.is_empty() {
        return None;
    }
    Some(LooksBack { cue, thing, said })
}

#[allow(dead_code)]
fn _unused(_: &Re) {}
