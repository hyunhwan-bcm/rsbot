//! Port of `bot/memory/already.py`: a line that says the step was already taken ("i already
//! called the bank", "ive fixed it", "my boiler is fixed") is answered with the next turn of
//! the thread: the answer's own next step when it has one, else the question a friend asks
//! next, named by the kind of step.

use super::pyv;
use super::there_is_a_chat;
use crate::data::{self, Value};
use crate::py::re::{self, Re};
use crate::py::text;
use crate::state::Memory;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.memory.already.";
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn words_set(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}
fn pool(name: &str) -> Vec<String> {
    data::strs(&format!("{M}{name}"))
}

static ALREADY: LazyLock<&'static Re> = LazyLock::new(|| rx("ALREADY"));
static DID: LazyLock<&'static Re> = LazyLock::new(|| rx("DID"));
static DID_STEPS: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("DID_STEPS"));
static DID_TAIL: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("DID_TAIL"));
static TRAILING: LazyLock<&'static Re> = LazyLock::new(|| rx("TRAILING"));
static PAST: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("PAST"));
static CONTACT: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("CONTACT"));
static NOBODY_TO_ANSWER: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("NOBODY_TO_ANSWER"));
static AN_APOLOGY: LazyLock<&'static Re> = LazyLock::new(|| rx("AN_APOLOGY"));
static REPAIRED: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("REPAIRED"));
static CHANGED: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("CHANGED"));
static CHANGED_OF_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("CHANGED_OF_A_THING"));
static A_LIFE_OBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("A_LIFE_OBJECT"));
static CHANGED_NOTHING_BROKEN: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("CHANGED_NOTHING_BROKEN"));
static ASKING: LazyLock<&'static Re> = LazyLock::new(|| rx("ASKING"));
static TO_THE_BOT: LazyLock<&'static Re> = LazyLock::new(|| rx("TO_THE_BOT"));
static MOST_WORDS: LazyLock<usize> = LazyLock::new(|| data::get("bot.memory.already.MOST_WORDS").as_int().unwrap() as usize);
static SAID_TO_ME: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("SAID_TO_ME"));
static HARD: LazyLock<&'static Re> = LazyLock::new(|| rx("HARD"));
static CONTACT_REPLIES: LazyLock<Vec<String>> = LazyLock::new(|| pool("CONTACT_REPLIES"));
static OVER_REPLIES: LazyLock<Vec<String>> = LazyLock::new(|| pool("OVER_REPLIES"));
static OVER_A_JOB_DONE: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("OVER_A_JOB_DONE"));
static OVER_NAMED_REPLIES: LazyLock<Vec<String>> = LazyLock::new(|| pool("OVER_NAMED_REPLIES"));
static OVER_NAMED_JOB_REPLIES: LazyLock<Vec<String>> = LazyLock::new(|| pool("OVER_NAMED_JOB_REPLIES"));
static REPAIRED_REPLIES: LazyLock<Vec<String>> = LazyLock::new(|| pool("REPAIRED_REPLIES"));
static CHANGED_REPLIES: LazyLock<Vec<String>> = LazyLock::new(|| pool("CHANGED_REPLIES"));
static REPORT_REPLIES: LazyLock<Vec<String>> = LazyLock::new(|| pool("REPORT_REPLIES"));
static SETTLED_REPLIES: LazyLock<Vec<String>> = LazyLock::new(|| pool("SETTLED_REPLIES"));
static FINISHED_ACT: LazyLock<&'static Re> = LazyLock::new(|| rx("FINISHED_ACT"));
static FINISHED_ITSELF: LazyLock<&'static Re> = LazyLock::new(|| rx("FINISHED_ITSELF"));
static BARE_DONE: LazyLock<&'static Re> = LazyLock::new(|| rx("BARE_DONE"));
static DONE_THAT: LazyLock<&'static Re> = LazyLock::new(|| rx("DONE_THAT"));
static FINISHED_STEPS: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("FINISHED_STEPS"));
static SETTLED: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("SETTLED"));
static BARE_COMPLETION: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("BARE_COMPLETION"));
static MORE_PADDING: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("MORE_PADDING"));
static KEPT_UP: LazyLock<&'static Re> = LazyLock::new(|| rx("KEPT_UP"));
static TELLING: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("TELLING"));
static APOLOGY_STEP: LazyLock<String> = LazyLock::new(|| data::string("bot.memory.already.APOLOGY_STEP"));
static APOLOGY: LazyLock<&'static Re> = LazyLock::new(|| rx("APOLOGY"));
static POINTS_AT: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("POINTS_AT"));
static IRREGULAR: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.memory.already.IRREGULAR"));
static A_STEP_TAKEN: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("A_STEP_TAKEN"));
static SAME_AS_PRESENT: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("SAME_AS_PRESENT"));
static AN_OUTCOME: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("AN_OUTCOME"));
static OVER_WORD: LazyLock<&'static Re> = LazyLock::new(|| rx("OVER_WORD"));
static OVER_STATE: LazyLock<&'static Re> = LazyLock::new(|| rx("OVER_STATE"));
static OVER_GOT: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("OVER_GOT"));
static OVER_PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("OVER_PEOPLE"));
static OVER_NOT: LazyLock<&'static Re> = LazyLock::new(|| rx("OVER_NOT"));
static OVER_ADJECTIVE: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("OVER_ADJECTIVE"));
static OVER_NOT_A_THING_NAMED: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("OVER_NOT_A_THING_NAMED"));
static OVER_A_MEAL: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("OVER_A_MEAL"));
static OVER_STYLED: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("OVER_STYLED"));
static OVER_LEAD: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("OVER_LEAD"));
static OVER_NO_THING: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("OVER_NO_THING"));
static OVER_BETWEEN: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("OVER_BETWEEN"));
static OVER_STOP: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("OVER_STOP"));
static NOT_A_THING_SAID_BACK: LazyLock<&'static Re> = LazyLock::new(|| rx("NOT_A_THING_SAID_BACK"));
static OVER_NEVER_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("OVER_NEVER_A_THING"));
static OVER_ASKING: LazyLock<&'static Re> = LazyLock::new(|| rx("OVER_ASKING"));
static _CLAUSE_BREAK: LazyLock<&'static Re> = LazyLock::new(|| rx("_CLAUSE_BREAK"));
static STEPS_LEADS: LazyLock<Vec<String>> = LazyLock::new(|| pool("STEPS_LEADS"));
static NOT_A_THING_WORD: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("NOT_A_THING_WORD"));
static REPORT_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("REPORT_FILLER"));
static MOST_STEPS: LazyLock<usize> = LazyLock::new(|| data::get("bot.memory.already.MOST_STEPS").as_int().unwrap() as usize);
static NOT_A_STEP_START: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("NOT_A_STEP_START"));
static CONDITION_START: LazyLock<&'static Re> = LazyLock::new(|| rx("CONDITION_START"));
static SAYS_APOLOGY: LazyLock<&'static Re> = LazyLock::new(|| rx("SAYS_APOLOGY"));
static FIRST_LINE_REPLIES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.warmth.FIRST_LINE_REPLIES"));
static CRISIS_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.fallback.CRISIS_WORDS"));

static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", re::U));
static LETTERS_AND_DIGITS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9']+", re::U));
static ONLY_LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]+", re::U));
static SENTENCE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s+", re::U));
static LEADING_THEN: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:and\s+)?then\s+", re::I | re::U));
static LEADING_JOIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:and|then|but|so)\s+", re::I | re::U));

/// `s[:1].isupper()`.
fn opens_upper(s: &str) -> bool {
    s.chars().next().is_some_and(|c| text::is_upper(c.encode_utf8(&mut [0; 4])))
}

/// `" ".join(str(text or "").lower().split())`.
fn flat_low(text_: &str) -> String {
    text::squash(&text_.to_lowercase())
}

/// Python: bot/memory/already.py:62 `_the_step_after_did(low)`
/// The step in a line that says it with "did" ("i did ring them though, twice"), or None: only
/// a step of `DID_STEPS`, with nothing after it but the little words a report is padded with.
pub fn _the_step_after_did(low: &str) -> Option<String> {
    let found = DID.search(low)?;
    let step = found.str(1);
    if !DID_STEPS.contains(step) {
        return None;
    }
    if LETTERS.findall_str(&low[found.end()..]).iter().any(|word| !DID_TAIL.contains(word)) {
        return None;
    }
    Some(step.to_string())
}

/// Python: bot/memory/already.py:345 `_a_crisis_line_said(memory, words)`
/// True when the bot's own last words in this stretch named a crisis line.
pub fn _a_crisis_line_said(memory: &Memory, words: &[String]) -> bool {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(2)..].iter().rev().any(|turn| {
        let said = turn.bot.to_lowercase();
        !said.is_empty() && words.iter().any(|word| said.contains(word.as_str()))
    })
}

/// Python: bot/memory/already.py:354 `_a_hard_thread(memory)`
/// True when the conversation is about a death or an illness, where this reader stands down.
pub fn _a_hard_thread(memory: &Memory) -> bool {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(2)..].iter().any(|turn| HARD.is_match(&turn.bot) || HARD.is_match(&turn.user))
}

/// Python: bot/memory/already.py:405 `_the_step(low)`
/// The step the line says was already taken, or None: a past tense after "already" (or before
/// a trailing "already"), not one of the words that only say what somebody has.
pub fn _the_step(low: &str) -> Option<String> {
    let (found, thing_after) = match ALREADY.search(low) {
        Some(found) => (found, true),
        None => (TRAILING.search(low)?, false),
    };
    let step = found.str(1);
    if TELLING.contains(step) && APOLOGY.is_match(low) {
        return Some(APOLOGY_STEP.clone());
    }
    if !PAST.contains(step) && !step.ends_with("ed") && !A_STEP_TAKEN.contains(step) {
        return None;
    }
    // A past tense written as its present tense ("i already read that") needs a thing after it.
    if SAME_AS_PRESENT.contains(step) && thing_after {
        let tail: Vec<String> =
            LETTERS_AND_DIGITS.findall_str(&low[found.end()..]).into_iter().filter(|w| !POINTS_AT.contains(w)).collect();
        if tail.is_empty() {
            return None;
        }
    }
    Some(step.to_string())
}

/// Python: bot/memory/already.py:342 `_nothing_more(low, match)`
/// True when the job is all the line says (everything after `end` is padding).
pub fn _nothing_more(low: &str, end: usize) -> bool {
    LETTERS.findall_str(&low[end..]).iter().all(|word| MORE_PADDING.contains(word))
}

/// Python: bot/memory/already.py:348 `_a_job_with_an_end(answer)`
/// True when the answer's first sentence hands over a job that finishes, not one kept up.
pub fn _a_job_with_an_end(answer: &str) -> bool {
    let first = SENTENCE_END.split(text::strip(answer)).swap_remove(0);
    !KEPT_UP.is_match(&first)
}

/// Python: bot/memory/already.py:354 `_the_finished_step(low, memory=None)`
/// The step a report of a job finished names ("ive fixed it" -> "fixed", "its done" -> "done"),
/// or None.
pub fn _the_finished_step(low: &str, memory: Option<&Memory>) -> Option<String> {
    let mut step: Option<String> = None;
    if let Some(found) = FINISHED_ACT.search(low) {
        let word = found.str(1);
        if (FINISHED_STEPS.contains(word) || word == "did") && _nothing_more(low, found.end()) {
            step = Some(if word == "did" { "done".into() } else { word.to_string() });
        }
    }
    if step.is_none() {
        let found = FINISHED_ITSELF.search(low).or_else(|| BARE_DONE.match_(low)).or_else(|| DONE_THAT.match_(low));
        if let Some(found) = found {
            if !_nothing_more(low, found.end()) {
                return None;
            }
            step = Some(found.str(1).to_lowercase());
        }
    }
    // The bare words only close a job when the answer handed one over that ends.
    let empty = Memory::new();
    if step.as_ref().is_some_and(|s| BARE_COMPLETION.contains(s)) && !_a_job_with_an_end(&_the_last_answer(memory.unwrap_or(&empty))) {
        return None;
    }
    step
}

/// Python: bot/memory/already.py:395 `the_step_they_took(text, memory=None)`
/// The word for the step the person says they already took, or None.
pub fn the_step_they_took(text_: &str, memory: Option<&Memory>) -> Option<String> {
    let empty = Memory::new();
    let memory = memory.unwrap_or(&empty);
    let low = flat_low(text_);
    if low.is_empty() || low.contains('?') {
        return None;
    }
    if text::split_ref(&low).len() > *MOST_WORDS {
        return None;
    }
    if TO_THE_BOT.is_match(&low) {
        return None;
    }
    // The trouble itself is over: taken before the asking words, because "is" in "my boiler is
    // fixed" is a state and not a question.
    if let Some(over) = the_trouble_is_over(&low) {
        if _a_hard_thread(memory) {
            return None;
        }
        return Some(over);
    }
    // After "i", "did" is the report itself: "i did ring them" asks nothing.
    if ASKING.is_match(&low) && !DID.is_match(&low) {
        return None;
    }
    let step = _the_step(&low).or_else(|| _the_step_after_did(&low)).or_else(|| _the_finished_step(&low, Some(memory)))?;
    if _a_hard_thread(memory) {
        return None;
    }
    Some(step)
}

/// What `_the_thing_said` reads in front of the state word.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThingSaid {
    /// Python's None: the line names no thing.
    Nothing,
    /// Python's False: the word means something else about this kind of thing.
    NotMended,
    Thing(String),
}

/// Python: bot/memory/already.py:212 `_the_thing_said(low, at)`
/// The thing named in front of the word at byte `at`: "my boiler is fixed" names the boiler.
pub fn _the_thing_said(low: &str, at: usize) -> ThingSaid {
    let bare_of = |w: &str| text::strip_chars(w, ".,!'").to_string();
    let mut words = text::split_ref(&low[..at]);
    while let Some(last) = words.last() {
        let bare = bare_of(last);
        if OVER_BETWEEN.contains(&bare) || OVER_STOP.contains(&bare) {
            words.pop();
        } else {
            break;
        }
    }
    if words.is_empty() {
        return ThingSaid::Nothing;
    }
    let mut thing: Vec<String> = Vec::new();
    for raw in words.iter().rev() {
        let bare = bare_of(raw);
        if thing.is_empty() && OVER_NO_THING.contains(&bare) {
            return ThingSaid::Nothing;
        }
        if OVER_LEAD.contains(&bare) || OVER_STOP.contains(&bare) || OVER_BETWEEN.contains(&bare) || OVER_NEVER_A_THING.contains(&bare) {
            break;
        }
        if OVER_NO_THING.contains(&bare) {
            return ThingSaid::Nothing;
        }
        thing.push(bare);
        if thing.len() == 2 {
            break;
        }
    }
    if thing.is_empty() {
        return ThingSaid::Nothing;
    }
    thing.reverse();
    let thing = thing.join(" ");
    // A clause or a figure is not a thing.
    if NOT_A_THING_SAID_BACK.match_(&thing).is_some() || text::is_digit(&text::split_ref(&thing).concat()) {
        return ThingSaid::Nothing;
    }
    let head = text::split_ref(&thing).last().copied().unwrap_or("").to_string();
    if OVER_NOT_A_THING_NAMED.contains(&head) || OVER_A_MEAL.contains(&head) || OVER_STYLED.contains(&head) {
        // "the match is fixed" is rigged and "dinner is done" is cooked.
        return ThingSaid::NotMended;
    }
    ThingSaid::Thing(thing)
}

/// Python: bot/memory/already.py:254 `the_thing_that_is_mended(text)`
/// The thing a line says is fixed, or None.
pub fn the_thing_that_is_mended(text_: &str) -> Option<String> {
    let (word, at, low) = _the_over_word(text_);
    word?;
    match _the_thing_said(&low, at?) {
        ThingSaid::Thing(thing) if !thing.is_empty() => Some(thing),
        _ => None,
    }
}

/// Python: bot/memory/already.py:276 `_the_clause(low, at)`
/// The clause that holds the byte `at`, cut at the joins either side of it.
pub fn _the_clause(low: &str, at: usize) -> &str {
    let (mut start, mut end) = (0, low.len());
    for found in _CLAUSE_BREAK.finditer(low) {
        if found.end() <= at {
            start = found.end();
        } else if found.start() >= at {
            end = found.start();
            break;
        }
    }
    &low[start..end]
}

/// Python: bot/memory/already.py:294 `_the_over_word(text)`
/// The accepted word, where it starts (a byte offset into the line), and the flattened line;
/// (None, None, line) when there is none.
pub fn _the_over_word(text_: &str) -> (Option<String>, Option<usize>, String) {
    let low = flat_low(text_);
    let words = text::split_ref(&low);
    if low.is_empty() || low.contains('?') || words.len() > *MOST_WORDS {
        return (None, None, low);
    }
    if OVER_ASKING.is_match(&low) {
        return (None, None, low);
    }
    for found in OVER_WORD.finditer(&low) {
        let word = found.str(1);
        let at = text::split_ref(&low[..found.start()]).len();
        let after = words.get(at + 1).copied().unwrap_or("");
        if OVER_ADJECTIVE.contains(text::strip_chars(after, ".,!")) {
            continue;
        }
        let mut near: Vec<String> =
            words[at.saturating_sub(4)..at.min(words.len())].iter().map(|w| text::strip_chars(w, ".,!'").to_string()).collect();
        // "we got the boiler fixed": the doer in front of "got" is not the subject read here.
        if let Some(first) = near.iter().position(|w| OVER_GOT.contains(w)) {
            near = near.into_iter().enumerate().filter(|(i, w)| *i >= first || !OVER_PEOPLE.contains(w)).map(|(_, w)| w).collect();
        }
        let window = near.join(" ");
        if near.iter().any(|w| OVER_PEOPLE.contains(w)) {
            continue;
        }
        // The negation has to share a clause with the state word.
        if OVER_NOT.is_match(_the_clause(&low, found.start())) {
            continue;
        }
        if OVER_STATE.is_match(&window) {
            if _the_thing_said(&low, found.start()) == ThingSaid::NotMended {
                continue;
            }
            let (word, start) = (word.to_string(), found.start());
            return (Some(word), Some(start), low.clone());
        }
    }
    (None, None, low)
}

/// Python: bot/memory/already.py:331 `the_trouble_is_over(text)`
/// The word that says the trouble is over, or None.
pub fn the_trouble_is_over(text_: &str) -> Option<String> {
    _the_over_word(text_).0
}

/// Python: bot/memory/already.py:341 `_gave_a_step(answer)`
/// True when the answer handed a move over, whatever the move was.
pub fn _gave_a_step(answer: &str) -> bool {
    pyv::attempt(|| !crate::compose::verdict::_an_instruction_sentence(answer).is_empty()).unwrap_or(false)
}

/// Python: bot/memory/already.py:355 `_the_opening_of(answer)`
/// The first sentence of an answer, which is where the step it hands over is named.
pub fn _the_opening_of(answer: &str) -> String {
    let said = text::strip(answer);
    let bytes = said.as_bytes();
    for (index, c) in said.char_indices() {
        if ".!?".contains(c) && bytes.get(index + 1) == Some(&b' ') {
            return said[..=index].to_string();
        }
    }
    said.to_string()
}

/// Python: bot/memory/already.py:370 `_says_the_step(step, answer)`
/// True when the last answer is where the person got the step they have already taken.
pub fn _says_the_step(step: &str, answer: &str) -> bool {
    if step == APOLOGY_STEP.as_str() || step.starts_with("apolog") {
        return SAYS_APOLOGY.is_match(&answer.to_lowercase());
    }
    if AN_OUTCOME.contains(step) {
        return _gave_a_step(answer);
    }
    let word = IRREGULAR.get(step).map(String::as_str).unwrap_or(step);
    let n = text::len(word);
    let stem = if n >= 4 && word.ends_with("ed") {
        &word[..word.len() - 2]
    } else if n >= 4 && word.ends_with('e') {
        &word[..word.len() - 1]
    } else {
        word
    };
    ONLY_LETTERS
        .findall_str(&answer.to_lowercase())
        .iter()
        .any(|other| other.starts_with(stem) || (text::len(other) >= 4 && word.starts_with(other.as_str())))
}

/// Python: bot/memory/already.py:389 `takes_as_it_comes(text, memory)`
/// True when this report should be answered as it comes, whatever else proposed: only when the
/// step is one the last answer itself named, or the trouble itself is over.
pub fn takes_as_it_comes(text_: &str, memory: &Memory) -> bool {
    pyv::attempt(|| {
        if reply_for(text_, memory).is_none() {
            return false;
        }
        let Some(step) = the_step_they_took(text_, Some(memory)) else { return false };
        if the_trouble_is_over(text_).is_some() {
            return true;
        }
        _says_the_step(&step, &_the_last_answer(memory))
    })
    .unwrap_or(false)
}

/// Python: bot/memory/already.py:454 `_the_steps_after(step, answer, line='')`
/// The answer's steps after the one they took, in order, or an empty list. The step they took
/// is found by the words they used, not by its verb alone.
pub fn _the_steps_after(step: &str, answer: &str, line: &str) -> Vec<String> {
    let Some(steps) = pyv::attempt(|| {
        super::limit::_steps(answer).into_iter().map(|(one, _)| text::strip(&one).to_string()).filter(|one| !one.is_empty()).collect::<Vec<_>>()
    }) else {
        return Vec::new();
    };
    if steps.is_empty() {
        return Vec::new();
    }
    let word = IRREGULAR.get(step).map(String::as_str).unwrap_or(step).to_lowercase();
    let n = text::len(&word);
    let stem = if n > 4 && word.ends_with("ed") {
        word[..word.len() - 2].to_string()
    } else if n > 3 && word.ends_with('e') {
        word[..word.len() - 1].to_string()
    } else {
        word.clone()
    };
    let wanted: HashSet<String> = LETTERS
        .findall_str(&line.to_lowercase())
        .into_iter()
        .filter(|other| !NOT_A_THING_WORD.contains(other) && !REPORT_FILLER.contains(other) && !other.starts_with(&stem) && text::len(other) > 1)
        .collect();
    let (mut best, mut best_score) = (None, 0);
    for (index, sentence) in steps.iter().enumerate() {
        let low = sentence.to_lowercase();
        let words: HashSet<String> = LETTERS.findall_str(&low).into_iter().collect();
        let mut score = words.intersection(&wanted).count();
        if low.contains(&stem) {
            score += 1;
        }
        if score > best_score {
            best = Some(index);
            best_score = score;
        }
    }
    let Some(best) = best else { return Vec::new() };
    if !wanted.is_empty() && best_score < 2 {
        return Vec::new();
    }
    let after: Vec<String> =
        steps[best + 1..].iter().map(|one| _the_instruction_of(text::rstrip_chars(text::strip(one), " .,;"))).collect();
    // A step that opens a new sentence is the answer moving on, except a condition on the job.
    let mut kept: Vec<String> = Vec::new();
    let mut in_condition = false;
    for one in after {
        if one.is_empty() {
            break;
        }
        if kept.is_empty() && NOT_A_STEP_START.contains(text::strip_chars(&text::split_ref(&one)[0].to_lowercase(), ",:")) {
            break;
        }
        if opens_upper(&one) {
            if in_condition || CONDITION_START.match_(&one).is_none() {
                break;
            }
            in_condition = true;
        }
        kept.push(one);
        if kept.len() >= *MOST_STEPS {
            break;
        }
    }
    kept
}

/// Python: bot/memory/already.py:514 `_the_instruction_of(step)`
/// A step without the explanation hung on the end of it after a colon.
pub fn _the_instruction_of(step: &str) -> String {
    let (head, colon, _tail) = text::partition(step, ":");
    if colon.is_empty() || text::split_ref(head).len() < 2 {
        return step.to_string();
    }
    text::rstrip_chars(text::strip(head), " .,;").to_string()
}

/// `fallback._choose(options, memory, clean, seed_text)`.
fn choose(options: &[String], memory: &Memory, clean: &str, seed_text: Option<&str>) -> Option<String> {
    crate::compose::fallback_chain::_choose(options, memory, clean, seed_text)
}

/// Python: bot/memory/already.py:529 `_with_steps(after, memory)`
/// The remaining steps said as what to do next.
pub fn _with_steps(after: &[String], memory: &Memory) -> String {
    let said: Vec<String> = after
        .iter()
        .map(|step| {
            if opens_upper(step) {
                let mut chars = step.chars();
                let first = chars.next().unwrap();
                first.to_lowercase().chain(chars).collect()
            } else if step.is_empty() {
                panic!("IndexError: string index out of range")
            } else {
                step.clone()
            }
        })
        .collect();
    let said = text::rstrip_chars(&said.join(", "), " .").to_string();
    // A step the answer already put in time order keeps its own "then".
    let said = LEADING_THEN.sub("", &said);
    choose(&STEPS_LEADS, memory, &said, None).expect("AttributeError: 'NoneType' object has no attribute 'format'").replace("{said}", &said)
}

/// Python: bot/memory/already.py:542 `_the_last_answer(memory)`
/// The bot's last reply, or the answer it was one step of.
pub fn _the_last_answer(memory: &Memory) -> String {
    let last = memory.turns.iter().rev().find(|turn| !turn.bot.is_empty()).map(|turn| turn.bot.clone()).unwrap_or_default();
    match pyv::attempt(|| super::plan::the_answer_in_hand(memory)) {
        Some(answer) if !answer.is_empty() => answer,
        _ => last,
    }
}

/// Python: bot/memory/already.py:558 `tells_them_the_step_again(reply, text, memory)`
/// True when `reply` still tells the person to do the step they have just reported doing.
pub fn tells_them_the_step_again(reply: &str, text_: &str, memory: &Memory) -> bool {
    let Some(Some(step)) = pyv::attempt(|| the_step_they_took(text_, Some(memory))) else { return false };
    if the_trouble_is_over(text_).is_some() {
        // A card on the topic of a thing they say is mended is advice for a trouble still there.
        return true;
    }
    pyv::attempt(|| _says_the_step(&step, reply)).unwrap_or(false)
}

/// Python: bot/memory/already.py:589 `_of_a_thing(step, text)`
/// True when the step was done to something in the house rather than to the person's life.
pub fn _of_a_thing(step: &str, text_: &str) -> bool {
    if !CHANGED_OF_A_THING.contains(step) {
        return false;
    }
    let words = LETTERS.findall_str(&text_.to_lowercase());
    let Some(at) = words.iter().position(|w| w == step) else { return false };
    let after = &words[at + 1..];
    if after.iter().any(|w| CHANGED_NOTHING_BROKEN.contains(w)) {
        return false; // nothing was broken, so there is no repair to ask after
    }
    !after.iter().any(|w| A_LIFE_OBJECT.contains(w))
}

/// Python: bot/memory/already.py:614 `_only_finished(text)`
/// True when the finished reading is the only reading of the line.
pub fn _only_finished(text_: &str) -> bool {
    let low = flat_low(text_);
    if _the_step(&low).is_some() || _the_step_after_did(&low).is_some() {
        return false;
    }
    _the_finished_step(&low, None).is_some()
}

/// Python: bot/memory/already.py:622 `_the_part_after(answer, step_text)`
/// The answer's own "if that does not work" line after the step it has just handed back, or [].
pub fn _the_part_after(answer: &str, step_text: &str) -> Vec<String> {
    let Some(pieces) = pyv::attempt(|| {
        super::limit::_steps(answer).into_iter().map(|(one, _)| text::strip(&one).to_string()).filter(|one| !one.is_empty()).collect::<Vec<_>>()
    }) else {
        return Vec::new();
    };
    if pieces.is_empty() {
        return Vec::new();
    }
    let wanted: HashSet<String> = LETTERS
        .findall_str(&step_text.to_lowercase())
        .into_iter()
        .filter(|w| text::len(w) > 1 && !NOT_A_THING_WORD.contains(w))
        .collect();
    let (mut best, mut best_score) = (None, 0);
    for (index, piece) in pieces.iter().enumerate() {
        let words: HashSet<String> = LETTERS.findall_str(&piece.to_lowercase()).into_iter().collect();
        let score = words.intersection(&wanted).count();
        if score > best_score {
            best = Some(index);
            best_score = score;
        }
    }
    let Some(best) = best.filter(|_| best_score >= 2) else { return Vec::new() };
    for piece in &pieces[best + 1..] {
        let piece = LEADING_JOIN.sub("", piece);
        let piece = text::rstrip_chars(text::strip(&piece), " .,;").to_string();
        if piece.is_empty() {
            continue;
        }
        if CONDITION_START.match_(&piece).is_some() {
            return vec![piece];
        }
        if opens_upper(&piece) {
            break;
        }
    }
    Vec::new()
}

/// Python: bot/memory/already.py:663 `after_the_step_handed_back(text, memory)`
/// The answer's own next step when the step it handed back did not do it, or None.
pub fn after_the_step_handed_back(text_: &str, memory: &Memory) -> Option<String> {
    let turns = memory.turns.len();
    if turns == 0 {
        return None;
    }
    let step = memory.get("handed_back_text").map(pyv::str_or_empty).unwrap_or_default();
    let answer = memory.get("handed_back_answer").map(pyv::str_or_empty).unwrap_or_default();
    // The step has to be the one the bot said in its last turn.
    let current = memory.get("handed_back_step").cloned().unwrap_or(Value::None) == Value::from(turns - 1);
    if step.is_empty() || answer.is_empty() || !current {
        return None;
    }
    let after = pyv::attempt(|| {
        if !super::tried::still_the_state(text_, memory) {
            return None;
        }
        Some(_the_part_after(&answer, &step))
    })
    .flatten()?;
    if after.is_empty() {
        return None;
    }
    Some(_with_steps(&after, memory))
}

/// Python: bot/memory/already.py:695 `reply_for(text, memory)`
/// The next turn of the thread for a report that the step was already taken, or None.
pub fn reply_for(text_: &str, memory: &Memory) -> Option<String> {
    let step = match pyv::attempt(|| the_step_they_took(text_, Some(memory))) {
        None => return None,
        Some(None) => {
            // The bot named the step last turn and the person says it has not worked.
            return after_the_step_handed_back(text_, memory);
        }
        Some(Some(step)) => step,
    };
    // A line that only says the job is over names no step, so with no answer behind it the
    // job would have to be invented.
    if _the_last_answer(memory).is_empty() && _only_finished(text_) {
        return None;
    }
    // The trouble itself is over: the answer's steps are behind them.
    if let Some(over) = the_trouble_is_over(text_) {
        // Not in a thread the bot has already answered with a crisis line.
        if _a_crisis_line_said(memory, &CRISIS_WORDS) {
            return None;
        }
        let last = crate::compose::fallback::memory_bot(memory);
        // A line that names the thing is answered with a sentence that names it too.
        if let Some(thing) = the_thing_that_is_mended(text_) {
            let named = if OVER_A_JOB_DONE.contains(&over) { &*OVER_NAMED_JOB_REPLIES } else { &*OVER_NAMED_REPLIES };
            let said = choose(named, memory, &last, Some(text_)).expect("AttributeError: 'NoneType' object has no attribute 'replace'");
            return Some(said.replace("{thing}", &thing));
        }
        return choose(&OVER_REPLIES, memory, &last, Some(text_));
    }
    // Every reply below asks about a step taken earlier in the chat; on the first line there is
    // none, and the first line's own reply answers it.
    if SAID_TO_ME.contains(&step) && !there_is_a_chat(memory) {
        return Some(pyv::pick(&FIRST_LINE_REPLIES, memory, text_));
    }
    // Where the answer still has a step they have not tried, it is `repeat`'s to hand back.
    let repeat_has_it = pyv::attempt(|| {
        !BARE_COMPLETION.contains(&step)
            && super::repeat::_tried(text_)
            && super::repeat::_later_part(&Value::from(text_), &_the_last_answer(memory)).truthy()
    });
    if repeat_has_it == Some(true) {
        return None;
    }
    // Where the answer still has a step after theirs, that step is the reply.
    let after = if BARE_COMPLETION.contains(&step) {
        Vec::new()
    } else {
        pyv::attempt(|| _the_steps_after(&step, &_the_last_answer(memory), text_)).unwrap_or_default()
    };
    if !after.is_empty() {
        return Some(_with_steps(&after, memory));
    }
    // An irregular past tense is a step only when the answer named that step.
    if A_STEP_TAKEN.contains(&step) && !CONTACT.contains(&step) && !TELLING.contains(&step) {
        let answer = _the_last_answer(memory);
        if !answer.is_empty() && !_says_the_step(&step, &_the_opening_of(&_the_last_answer(memory))) {
            return None;
        }
    }
    let chosen: &[String] = if NOBODY_TO_ANSWER.contains(&step) || (CONTACT.contains(&step) && AN_APOLOGY.is_match(text_)) {
        &REPORT_REPLIES
    } else if CONTACT.contains(&step) {
        &CONTACT_REPLIES
    } else if REPAIRED.contains(&step) || _of_a_thing(&step, text_) {
        &REPAIRED_REPLIES
    } else if CHANGED.contains(&step) {
        &CHANGED_REPLIES
    } else if SETTLED.contains(&step) {
        &SETTLED_REPLIES
    } else {
        &REPORT_REPLIES
    };
    let last = crate::compose::fallback::memory_bot(memory);
    choose(chosen, memory, &last, None)
}
