//! Port of `bot/memory/detail.py`: a line that adds a detail to the thing the last answer was
//! about ("its on the bedroom ones mainly"), answered with the one step of that answer for it;
//! and the readers beside it - a time the thing happens, a part of the answer asked about, a
//! plain ask the answer covers, a report about the thing. Module data comes from the bundle.
//!
//! Module-level names PyBot defines twice (`A_WHEN`) are read from the bundle, which holds the
//! value the module ends with - the one every function sees at run time.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::repeat;
use crate::py::re::Re;
use crate::py::text as pt;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

fn g(name: &str) -> &'static Value {
    data::get(&format!("bot.memory.detail.{name}"))
}
fn re(name: &str) -> &'static Re {
    g(name).as_re().unwrap_or_else(|| panic!("bot.memory.detail.{name} is not a regex"))
}

macro_rules! patterns {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> = LazyLock::new(|| re(stringify!($name)));)*
    };
}
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> = LazyLock::new(|| g(stringify!($name)).strs());)*
    };
}
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> = LazyLock::new(|| g(stringify!($name)).str_set());)*
    };
}
macro_rules! counts {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<usize> = LazyLock::new(|| g(stringify!($name)).as_int().unwrap_or(0) as usize);)*
    };
}
patterns!(
    JOIN_OPEN, COPULA_OPEN, ABOUT_OPEN, PRONOUN_OPEN, PREPOSITION_OPEN, LIKE_OPEN, A_PERSON_BODY,
    AND_A_CLAUSE, DETAIL_BODY, QUANTITY_BODY, MEASURE_BODY, HAVE_COUNT, JOIN_LEAD, PREPOSITION,
    A_SUBJECT_OF_ITS_OWN, NAMES_NOTHING, A_WHEN, SHOWS_OUT, SHOWS_FROM, FALTER, SPAN_ONLY, A_FIGURE,
    THE_PERSON, ASKING_ME, A_GREETING, COUNTED_DETAIL, FIGURED, ASKS_A_PART, PLAIN_ASK,
    A_QUANTITY_ASK, CLAUSE, LEAD_AND, HEADING, A_DURATION, REPORT_SENTENCE,
    REPORT_OPENS_ON_THE_THING, REPORT_ASKS, REPORT_BEHAVIOUR, REPORT_NEGATES, A_REFERRAL,
    OPENS_MID_THOUGHT,
);
pools!(LEADS_LEAD, LEADS_FOR, LEADS_FOR_A_TIME, PLAIN_LEADS, STEP_ANSWERS, NOT_AN_ANSWER, A_TIME_IDIOM);
word_sets!(
    TIME_WORDS, VAGUE, STATE, DETERMINER, TIME_OF_DAY, FREQUENCY, A_QUALIFIER, A_DANGLING,
    A_COUNT_WORD, A_SMALL_WORD, ASKING, ASK_ONLY, NAME_ONLY, NOT_A_THING, SPAN_WORDS,
    SPAN_WORDS_ABOUT, NUMBER_WORDS, CLAUSE_WORDS, REPORT_FILLER,
);
counts!(MOST_WORDS, STEP_WORDS, SHORT_STEP, FACT_SENTENCE_WORDS, A_PART_WORDS, A_STEP_ANSWER, HEADING_WORDS, REPORT_WORDS);

static VERBISH: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.understanding.kinds.VERBISH"));
static VERB_FORMS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.understanding.kinds.VERB_FORMS"));
static IMPERATIVES: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.verdict.IMPERATIVES"));
static TOPICS_STOPWORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.topics.STOPWORDS"));
static NEXT_STEP_EMERGENCY: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.repeat.NEXT_STEP_EMERGENCY"));
static REPEAT_SENTENCE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.repeat.SENTENCE"));

// The patterns PyBot writes inline.
static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static SENTENCE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s+", 0));
static SPACES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+", 0));
static A_JOINING_WORD: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"^(?:but|and|so|oh|also|well|yeah|yep|ok|okay|right|alright)[\s,]+", crate::py::re::I));
static DASH_GROUPS: LazyLock<Re> = LazyLock::new(|| Re::new("\\s+-\\s+|\\s+\u{2014}\\s+", 0));
static BRACKETED: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s*\([^)]*\)", 0));
static AN_INDEFINITE: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?:a|an|some)\b", crate::py::re::I));
static NOT_A_LETTER: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z]", 0));

/// What another file's function returned, read with Python truthiness, whatever its contract type.
trait PyTrue {
    fn py_true(&self) -> bool;
}
impl PyTrue for () {
    fn py_true(&self) -> bool {
        false
    }
}
impl PyTrue for bool {
    fn py_true(&self) -> bool {
        *self
    }
}
impl PyTrue for Value {
    fn py_true(&self) -> bool {
        self.truthy()
    }
}
impl PyTrue for String {
    fn py_true(&self) -> bool {
        !self.is_empty()
    }
}
impl PyTrue for Option<String> {
    fn py_true(&self) -> bool {
        self.as_deref().is_some_and(|s| !s.is_empty())
    }
}

/// `trouble.reads_as_trouble(clean)`.
fn reads_as_trouble(clean: &str) -> bool {
    crate::compose::trouble::reads_as_trouble(clean, None, "").py_true()
}

/// A Python `try` whose `except Exception` gives a fallback; UNPORTED passes through.
fn py_try<T>(f: impl FnOnce() -> T) -> Option<T> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(v) => Some(v),
        Err(e) => {
            let msg = e.downcast_ref::<String>().map(String::as_str).or_else(|| e.downcast_ref::<&str>().copied()).unwrap_or("");
            if msg.starts_with("UNPORTED") {
                std::panic::resume_unwind(e);
            }
            None
        }
    }
}

/// `not memory`: the dict has no key at all.
fn is_empty(memory: &Memory) -> bool {
    memory.understood.is_none()
        && memory.intent_wants.is_none()
        && !memory.extra_had_intent
        && memory.asked_now.is_none()
        && memory.extra.is_empty()
        && memory.turns.is_empty()
}

/// `pick(options, memory, seed_text)` from `bot.compose` (`repeat.pick`).
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    let memory_value = Value::from_plain(&serde_json::Value::Object(memory.to_plain()));
    crate::compose::pick(options, &memory_value, seed_text)
}

/// `str.format` for the `{name}` fields these leads use (unused fields are ignored).
fn fmt(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 64);
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find('}') {
            Some(close) => {
                let name = &after[..close];
                match fields.iter().find(|(k, _)| *k == name) {
                    Some((_, v)) => out.push_str(v),
                    None => panic!("KeyError: '{name}'"),
                }
                rest = &after[close + 1..];
            }
            None => {
                out.push_str(&rest[open..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

/// `re.split(r"(?<=[.!?])\s+", text)`.
fn sentences_of(text: &str) -> Vec<String> {
    SENTENCE_END.split(text)
}

/// `len(s.split())`.
fn n_words(s: &str) -> usize {
    pt::split_ref(s).len()
}

/// `s[:1].lower() + s[1:]`.
fn low1(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().chain(c).collect(),
    }
}

/// Python: bot/memory/detail.py:236 `_a_verb(word, before='')`
///
/// True for a word that makes the line a report rather than a detail. A verb form behind a
/// determiner is the thing's adjective ("a rented flat").
pub fn _a_verb(word: &str, before: &str) -> bool {
    if DETERMINER.contains(before) {
        return false;
    }
    VERBISH.contains(word) || VERB_FORMS.contains(word)
}

/// Python: bot/memory/detail.py:249 `_flat(text)`
pub fn _flat(text: &str) -> String {
    pt::squash(text)
}

/// Python: bot/memory/detail.py:253 `_the_same_answer_as_the_one_it_narrows(step, said)`
///
/// True when the whole reply found for the detail is the answer it narrows: the first 60
/// characters are enough.
pub fn _the_same_answer_as_the_one_it_narrows(step: &str, said: &str) -> bool {
    let here = pt::slice(&pt::lower(&_flat(step)), 0, Some(60));
    let there = pt::slice(&pt::lower(&_flat(said)), 0, Some(60));
    !here.is_empty() && here == there
}

/// Python: bot/memory/detail.py:263 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    WORDS.findall_str(&pt::lower(&_flat(text)))
}

/// The opener a detail was read with (Python compares the pattern objects with `is`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Opener {
    About,
    Like,
    Copula,
    Pronoun,
    HaveCount,
}

/// `" ".join(SHOWS_OUT.sub("from ", SHOWS_FROM.sub("", said)).split())`: how the trouble shows
/// is part of the asking.
fn without_the_showing(said: &str) -> String {
    pt::squash(&SHOWS_OUT.sub("from ", &SHOWS_FROM.sub("", said)))
}

/// Python: bot/memory/detail.py:279 `the_detail(text)`
///
/// The detail as a phrase to put in front of the step ("on the bedroom ones mainly"), or None
/// when the line is not a detail of the thing the answer was about.
pub fn the_detail(text: &str) -> Option<String> {
    let said = _flat(text);
    if said.is_empty() || said.contains('?') {
        return None;
    }
    let mut said = without_the_showing(&said);
    let joined = pt::strip(&JOIN_OPEN.sub("", &said)).to_string();
    if !joined.is_empty() && joined != said && PREPOSITION.match_(&joined).is_some() {
        said = joined;
    }
    if PREPOSITION.match_(&said).is_some() && (_names_nothing(&said) || A_SUBJECT_OF_ITS_OWN.is_match(&said)) {
        // A fixed phrase and a clause of the person's own are a remark, not a narrowing.
        return None;
    }
    if n_words(&said) > *MOST_WORDS {
        return None;
    }
    // A span said on its own: "its been about three weeks now".
    if let Some(span) = SPAN_ONLY.match_(&said) {
        return Some(format!("for {}", pt::squash(span.name_str("span"))));
    }
    // The count said as the person has it is read before the verb test.
    let counted_rest: Option<String> = HAVE_COUNT
        .match_(&said)
        .filter(|counted| QUANTITY_BODY.match_(counted.name_str("count")).is_some())
        .map(|counted| pt::strip(counted.name_str("count")).to_string());
    // How the thing is going wrong, and where: the place is the part of the thing.
    if let Some(falter) = FALTER.match_(&said) {
        return Some(pt::strip(falter.name_str("place")).to_string());
    }
    let not_a_copula = |w: &String| !matches!(w.as_str(), "is" | "are" | "was" | "were" | "be" | "been");
    let mut body_words: Vec<String> = _words(&said).into_iter().filter(not_a_copula).collect();
    if let Some(like_head) = LIKE_OPEN.match_(&said) {
        body_words = _words(&said[like_head.end()..]).into_iter().filter(not_a_copula).collect();
    }
    let befores = std::iter::once("").chain(body_words.iter().map(String::as_str));
    if counted_rest.is_none() && befores.zip(body_words.iter()).any(|(before, w)| _a_verb(w, before)) {
        return None;
    }
    let openers: [(Opener, &Re); 4] =
        [(Opener::About, *ABOUT_OPEN), (Opener::Like, *LIKE_OPEN), (Opener::Copula, *COPULA_OPEN), (Opener::Pronoun, *PRONOUN_OPEN)];
    // As in Python, the loop variable keeps the last opener tried when none of them breaks.
    let mut opener = Opener::Pronoun;
    let mut rest: Option<String> = None;
    for (which, pattern) in openers {
        opener = which;
        let stripped = pt::strip(&pattern.sub("", &said)).to_string();
        if stripped.is_empty() || stripped == said {
            continue;
        }
        if which != Opener::Pronoun
            && which != Opener::Like
            && !(DETAIL_BODY.match_(&stripped).is_some()
                || (which == Opener::Copula && (QUANTITY_BODY.match_(&stripped).is_some() || MEASURE_BODY.match_(&stripped).is_some())))
        {
            return None;
        }
        rest = Some(stripped);
        break;
    }
    let mut rest = match rest {
        Some(rest) => rest,
        None => {
            if let Some(counted) = counted_rest {
                opener = Opener::HaveCount;
                counted
            } else if PREPOSITION_OPEN.match_(&said).is_none() {
                return None;
            } else {
                let mut rest = said.clone();
                // "Like" introduces the thing rather than being part of it.
                if LIKE_OPEN.match_(&rest).is_some() {
                    rest = pt::strip_chars(&LIKE_OPEN.sub("", &rest), " .,;:").to_string();
                    if rest.is_empty() {
                        return None;
                    }
                }
                rest
            }
        }
    };
    // A pronoun with no preposition is the person's own state ("im thirsty").
    if opener == Opener::Pronoun && PREPOSITION.match_(&rest).is_none() {
        return None;
    }
    rest = pt::strip_chars(&rest, " .,!;:").to_string();
    if rest.is_empty() || AND_A_CLAUSE.is_match(&rest) {
        return None;
    }
    let content: Vec<String> = _words(&PREPOSITION.sub("", &rest)).into_iter().filter(|w| !VAGUE.contains(w)).collect();
    if content.is_empty() {
        return None;
    }
    if content.iter().all(|word| TIME_WORDS.contains(word)) {
        return None; // how often or how long, not which thing
    }
    if A_PERSON_BODY.match_(&rest).is_some() {
        return None; // a person is a different subject
    }
    if content.iter().all(|word| STATE.contains(word)) {
        return None; // only how it is, not which part it is
    }
    Some(rest)
}

/// Python: bot/memory/detail.py:413 `_a_figure(word)`
///
/// The figure as it can be compared: the number and the unit without the space ("5k").
pub fn _a_figure(word: &str) -> String {
    pt::rstrip_chars(&SPACES.sub("", &pt::lower(word)), ".,").to_string()
}

/// The answer one turn back, the newest the repeat reader would hand back.
fn the_answer_one_turn_back(memory: &Memory) -> String {
    for (back, _asked, answer) in repeat::recent_answers(memory) {
        if back > 1 {
            break;
        }
        if !answer.is_empty() {
            return answer;
        }
    }
    String::new()
}

/// Python: bot/memory/detail.py:418 `the_fact_the_answer_has(text, memory)`
///
/// The sentence of the answer just given that speaks of a figure the person names, or None.
pub fn the_fact_the_answer_has(text: &str, memory: &Memory) -> Option<String> {
    if is_empty(memory) {
        return None;
    }
    let said = _flat(text);
    if said.is_empty() || said.contains('?') || n_words(&said) > *MOST_WORDS {
        return None;
    }
    if !THE_PERSON.is_match(&said) || ASKING_ME.match_(&said).is_some() {
        return None;
    }
    let wanted: HashSet<String> = A_FIGURE.finditer(&said).iter().map(|m| _a_figure(m.as_str())).collect();
    if wanted.is_empty() {
        return None;
    }
    let reply = the_answer_one_turn_back(memory);
    if reply.is_empty() {
        return None;
    }
    let sentences: Vec<String> = sentences_of(&reply).iter().map(|p| pt::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    // One sentence is the whole answer: handing it back is repeating, not answering.
    if sentences.len() < 2 {
        return None;
    }
    for sentence in &sentences {
        if n_words(sentence) < *FACT_SENTENCE_WORDS {
            continue;
        }
        let there: HashSet<String> = A_FIGURE.finditer(sentence).iter().map(|m| _a_figure(m.as_str())).collect();
        if !there.is_disjoint(&wanted) {
            let said_back = if pt::ends_with_any(sentence, &[".", "!", "?"]) { sentence.clone() } else { format!("{sentence}.") };
            return Some(pt::upper_first(&said_back));
        }
    }
    None
}

/// What `the_step_for` found: the detail, the answer it narrows, the step, and whether the step
/// is a whole reply rather than a step of the answer.
pub type Found = (String, String, String, bool);

/// Python: bot/memory/detail.py:464 `the_step_for(text, memory)`
///
/// The detail, the answer it narrows and the step to hand back, or None.
pub fn the_step_for(text: &str, memory: &Memory) -> Option<Found> {
    if is_empty(memory) {
        return None;
    }
    let detail = the_detail(text)?;
    if detail.is_empty() {
        return None;
    }
    // A line about the person's own body or a broken thing is not a detail of an answer.
    if reads_as_trouble(&_flat(text)) {
        return None;
    }
    let mut own: Option<String> = None;
    let mut own_read = false;
    for (back, asked, said) in repeat::recent_answers(memory) {
        if back > 2 {
            continue;
        }
        if back > 1 && !repeat::_about_the_same_thing(&asked, &said, memory) {
            continue;
        }
        let plain = _without_branches(&said);
        if plain.is_empty() {
            // An answer made only of cases has no move of its own to hand over, but a thing the
            // line names may still have a card of its own.
            if !own_read {
                own = _the_thing_itself(&detail, memory);
                own_read = true;
            }
            if let Some(own) = own.clone().filter(|o| !o.is_empty()) {
                return Some((detail, said, own, true));
            }
            if _another_thing(&detail, &said) {
                if let Some(worth) = _what_it_is_worth(&detail, memory).filter(|w| !w.is_empty()) {
                    return Some((detail, said, worth, true));
                }
            }
            continue;
        }
        let mut step = _step_about(&plain, &detail).unwrap_or_default();
        if _a_count(&detail) {
            // A count changes the figure: the sentence carrying it is the step.
            if let Some(figure) = _the_figure(&plain).filter(|f| !f.is_empty()) {
                step = figure;
            }
        }
        if step.is_empty() {
            if !own_read {
                own = _the_thing_itself(&detail, memory);
                own_read = true;
            }
            if let Some(own) = own.clone().filter(|o| !o.is_empty()) {
                return Some((detail, said, own, true));
            }
            // A thing the answer never speaks of is a question of its own.
            if _another_thing(&detail, &said) {
                if let Some(worth) = _what_it_is_worth(&detail, memory).filter(|w| !w.is_empty()) {
                    return Some((detail, said, worth, true));
                }
            }
            if _asks_about_a_new_thing(text, &detail, &said) {
                continue;
            }
            if _is_a_span(&detail) {
                step = _a_span_step(&plain, &detail);
                if step.is_empty() {
                    step = _first_instruction(&plain);
                }
                if step.is_empty() {
                    step = _opening_said(&plain);
                }
            } else {
                step = _a_step(&plain, false);
                if step.is_empty() && _says_where(&detail) {
                    step = _first_instruction(&plain);
                    if step.is_empty() {
                        step = _opening_said(&plain);
                    }
                }
            }
        }
        if step.is_empty() {
            continue;
        }
        if repeat::_already_repeated(&step, memory) {
            // A second detail in the same answer gets the first clause only.
            step = _first_clause(&step);
            if step.is_empty() || repeat::_already_repeated(&step, memory) {
                return None;
            }
        }
        return Some((detail, said, step, false));
    }
    None
}

/// Python: bot/memory/detail.py:635 `the_time_in_the_line(text)`
///
/// The time or frequency the line names and no thing, or None.
pub fn the_time_in_the_line(text: &str) -> Option<String> {
    let said = _flat(text);
    if said.is_empty() || said.contains('?') || n_words(&said) > *MOST_WORDS {
        return None;
    }
    let said = without_the_showing(&said);
    let mut body = said.clone();
    for pattern in [&*ABOUT_OPEN, &*LIKE_OPEN, &*COPULA_OPEN, &*PRONOUN_OPEN] {
        let stripped = pt::strip(&pattern.sub("", &body)).to_string();
        if !stripped.is_empty() && stripped != body {
            body = stripped;
            break;
        }
    }
    let body = pt::strip(&A_JOINING_WORD.sub("", &body)).to_string();
    let words = _words(&body);
    if words.is_empty() || words.len() > *MOST_WORDS {
        return None;
    }
    let befores = std::iter::once("").chain(words.iter().map(String::as_str));
    if befores.zip(words.iter()).any(|(before, word)| _a_verb(word, before)) {
        return None;
    }
    if A_GREETING.match_(pt::strip_chars(&body, " .,!;:")).is_some() {
        return None;
    }
    if words.last().is_some_and(|last| A_DANGLING.contains(last)) {
        return None;
    }
    if words.iter().any(|word| A_COUNT_WORD.contains(word) || pt::is_digit(word)) {
        return None;
    }
    for idiom in A_TIME_IDIOM.iter() {
        if pt::lower(pt::strip(&body)) == *idiom {
            return Some(pt::strip_chars(&body, " .,!;:").to_string());
        }
    }
    let content: Vec<&String> = words
        .iter()
        .filter(|w| !VAGUE.contains(*w) && !STATE.contains(*w) && !A_QUALIFIER.contains(*w) && !A_SMALL_WORD.contains(*w))
        .collect();
    if content.is_empty() || !content.iter().all(|w| TIME_WORDS.contains(*w)) {
        return None;
    }
    // A time of day or a frequency, and not only a span.
    if !content.iter().any(|w| TIME_OF_DAY.contains(*w) || FREQUENCY.contains(*w)) {
        return None;
    }
    Some(pt::strip_chars(&body, " .,!;:").to_string())
}

/// Python: bot/memory/detail.py:679 `_a_time_step(said, detail)`
///
/// The sentence of the answer that speaks of the same time, or None.
pub fn _a_time_step(said: &str, detail: &str) -> Option<String> {
    let named: Vec<String> = _words(detail).into_iter().filter(|w| TIME_WORDS.contains(w)).collect();
    if named.is_empty() {
        return None;
    }
    for sentence in sentences_of(&_flat(said)) {
        let here = _words(&sentence);
        if named.iter().any(|word| here.contains(word)) {
            let sentence = pt::strip(&sentence);
            if !sentence.is_empty() && n_words(sentence) >= 3 {
                return Some(sentence.to_string());
            }
        }
    }
    None
}

/// Python: bot/memory/detail.py:700 `_the_answer_before(memory, text)`
///
/// What the bot said to the turn before this one, or "" when it has said nothing yet.
pub fn _the_answer_before(memory: &Memory, text: &str) -> String {
    let here = pt::rstrip_chars(&pt::lower(&_flat(text)), ".!?, ").to_string();
    for turn in memory.turns.iter().rev() {
        let said = pt::rstrip_chars(&pt::lower(&_flat(&turn.user)), ".!?, ").to_string();
        if said.is_empty() || (!here.is_empty() && said == here) {
            continue;
        }
        let reply = _flat(&turn.bot);
        if !reply.is_empty() {
            return reply;
        }
    }
    String::new()
}

/// Python: bot/memory/detail.py:723 `the_step_for_a_time(text, memory)`
///
/// The time named, the answer it belongs to and the step to hand back, or None.
pub fn the_step_for_a_time(text: &str, memory: &Memory) -> Option<Found> {
    if is_empty(memory) {
        return None;
    }
    let detail = the_time_in_the_line(text).filter(|d| !d.is_empty())?;
    if reads_as_trouble(&_flat(text)) {
        return None;
    }
    let said = _the_answer_before(memory, text);
    if said.is_empty() {
        return None;
    }
    if NEXT_STEP_EMERGENCY.is_match(&said) {
        // The answer before is the emergency route: nothing lesser is handed back.
        return None;
    }
    let plain = {
        let without = _without_branches(&said);
        if without.is_empty() { _flat(&said) } else { without }
    };
    let mut step = _a_time_step(&plain, &detail).unwrap_or_default();
    if step.is_empty() {
        step = _first_instruction(&plain);
    }
    if step.is_empty() {
        step = _a_step(&plain, false);
    }
    if step.is_empty() {
        return None;
    }
    if repeat::_already_repeated(&step, memory) {
        step = _first_clause(&step);
        if step.is_empty() || repeat::_already_repeated(&step, memory) {
            return None;
        }
    }
    Some((detail, said, step, false))
}

/// Python: bot/memory/detail.py:771 `_a_count(detail)`
///
/// Whether the detail is a number of people or portions, not a part of the thing.
pub fn _a_count(detail: &str) -> bool {
    COUNTED_DETAIL.match_(&_flat(detail)).is_some()
}

/// Python: bot/memory/detail.py:776 `_the_figure(said)`
///
/// The sentence of the answer that carries the figure, or None.
pub fn _the_figure(said: &str) -> Option<String> {
    sentences_of(&_flat(said)).iter().map(|s| pt::strip(s).to_string()).find(|s| !s.is_empty() && FIGURED.is_match(s))
}

/// Python: bot/memory/detail.py:809 `_the_carrying_words(text)`
///
/// The words of the line that name something, with a plural ending taken off.
pub fn _the_carrying_words(text: &str) -> Vec<String> {
    if !_words(text).iter().any(|w| !ASKING.contains(w) && pt::len(w) > 3) {
        return Vec::new();
    }
    _words(text)
        .into_iter()
        .filter(|w| !ASKING.contains(w) && pt::len(w) > 2)
        .map(|w| pt::rstrip_chars(&w, "s").to_string())
        .collect()
}

/// `{word.rstrip("s") for word in _words(text)}`.
fn stems_of(text: &str) -> HashSet<String> {
    _words(text).iter().map(|w| pt::rstrip_chars(w, "s").to_string()).collect()
}

/// Python: bot/memory/detail.py:824 `_the_clause_that_holds(words, said)`
///
/// The sentence, or the clause of it, that holds the words asked about, or "".
pub fn _the_clause_that_holds(words: &[String], said: &str) -> String {
    let wanted: HashSet<String> = words.iter().cloned().collect();
    for sentence in sentences_of(&_flat(said)) {
        if wanted.is_disjoint(&stems_of(&sentence)) {
            continue;
        }
        let clauses: Vec<String> =
            _clauses(&sentence).iter().map(|p| pt::strip_chars(p, " ,;:-").to_string()).filter(|p| !p.is_empty()).collect();
        // `max(..., default="")`: the first of the clauses that hold the most.
        let mut best = String::new();
        let mut most: Option<usize> = None;
        for piece in &clauses {
            let held = wanted.intersection(&stems_of(piece)).count();
            if most.map_or(true, |m| held > m) {
                most = Some(held);
                best = piece.clone();
            }
        }
        if best.is_empty() || wanted.is_disjoint(&stems_of(&best)) {
            continue;
        }
        let mut best = pt::strip(&JOIN_LEAD.sub("", &best)).to_string();
        if n_words(&best) < 4 {
            // A clause this short is half a sentence: the sentence is the smallest thing.
            best = pt::strip_chars(&JOIN_LEAD.sub("", &_flat(&sentence)), " ,;:-").to_string();
        }
        if best.is_empty() || n_words(&best) > *A_PART_WORDS {
            continue;
        }
        return pt::upper_first(&best);
    }
    String::new()
}

/// Python: bot/memory/detail.py:848 `the_part_of_the_answer(text, memory)`
///
/// The reply when the line asks about a part of the answer one turn back, or "".
pub fn the_part_of_the_answer(text: &str, memory: &Memory) -> String {
    let said = _flat(text);
    if said.is_empty() || n_words(&said) > *MOST_WORDS || ASKS_A_PART.match_(&said).is_none() {
        return String::new();
    }
    let carrying = _the_carrying_words(&said);
    if carrying.is_empty() {
        return String::new();
    }
    if reads_as_trouble(&said) {
        return String::new();
    }
    let answer = the_answer_one_turn_back(memory);
    if answer.is_empty() {
        return String::new();
    }
    let there = stems_of(&answer);
    if carrying.iter().any(|word| !there.contains(word)) {
        return String::new();
    }
    let clause = _the_clause_that_holds(&carrying, &answer);
    if clause.is_empty() || repeat::_already_repeated(&clause, memory) {
        return String::new();
    }
    if pt::ends_with_any(&clause, &[".", "!", "?"]) { clause } else { format!("{clause}.") }
}

/// Python: bot/memory/detail.py:884 `answers_the_line(text, memory)`
///
/// True when `answer_for` will answer this line: the answer just given has its step.
pub fn answers_the_line(text: &str, memory: &Memory) -> bool {
    the_step_for(text, memory).is_some() || the_step_for_a_time(text, memory).is_some()
}

/// Python: bot/memory/detail.py:889 `answer_for(text, memory)`
///
/// The reply for a detail added to the answer just given, or None.
pub fn answer_for(text: &str, memory: &mut Memory) -> Option<String> {
    let mut found = the_step_for(text, memory);
    let mut a_time = false;
    if found.is_none() {
        // The line names no thing, only when the thing happens.
        found = the_step_for_a_time(text, memory);
        a_time = found.is_some();
    }
    let Some((detail, said, step, whole)) = found else {
        // The line names no thing of its own, only words the answer just gave.
        let a_part = the_part_of_the_answer(text, memory);
        if !a_part.is_empty() {
            return Some(repeat::_handed_back(memory, &a_part));
        }
        return None;
    };
    if whole && _the_same_answer_as_the_one_it_narrows(&step, &said) {
        // The same paragraph again: not a detail of the answer.
        return None;
    }
    if whole {
        // A whole reply: framed as the repeat it is, and the card itself remembered as said.
        let framed = crate::memory::again::a_thing_of_the_answer(&step, text, memory);
        repeat::_handed_back(memory, &step);
        return Some(if framed.truthy() { crate::memory::facts::py_str(&framed) } else { step });
    }
    let thing = {
        let bare = pt::strip(&PREPOSITION.sub("", &detail)).to_string();
        if bare.is_empty() { detail.clone() } else { bare }
    };
    if a_time {
        let reply = fmt(
            &pick(&LEADS_FOR_A_TIME, memory, text),
            &[("detail_cap", &_capital(&detail)), ("thing", &thing), ("step", &_capital(&step)), ("step_low", &low1(&step))],
        );
        return Some(repeat::_handed_back(memory, &reply));
    }
    let leads = if thing != detail { &*LEADS_LEAD } else { &*LEADS_FOR };
    let reply = fmt(&pick(leads, memory, text), &[("detail_cap", &_capital(&detail)), ("thing", &thing), ("step", &_capital(&step))]);
    Some(repeat::_handed_back(memory, &reply))
}

/// Python: bot/memory/detail.py:1015 `_the_answer_just_given(memory)`
///
/// The bot's last reply when it was an answer with a step in it, or "".
pub fn _the_answer_just_given(memory: &Memory) -> String {
    let Some(last) = memory.turns.last() else {
        return String::new();
    };
    if !last.module.as_deref().is_some_and(|m| STEP_ANSWERS.iter().any(|a| a == m)) {
        return String::new();
    }
    let said = pt::strip(&last.bot);
    if pt::len(said) < *A_STEP_ANSWER || pt::rstrip(said).ends_with('?') {
        return String::new();
    }
    said.to_string()
}

/// Python: bot/memory/detail.py:1029 `_bare_step(step)`
///
/// The step with its capital taken off where it carries on a sentence; an opening in capitals
/// is a name and stays.
pub fn _bare_step(step: &str) -> String {
    let head = pt::slice(step, 0, Some(2));
    if pt::len(step) > 1 && head == pt::upper(&head) {
        return step.to_string();
    }
    low1(step)
}

/// Python: bot/memory/detail.py:1043 `the_plain_ask(text, memory)`
///
/// The step of the answer just given, when the line asks about it and names no thing, or None.
pub fn the_plain_ask(text: &str, memory: &mut Memory) -> Option<String> {
    if is_empty(memory) {
        return None;
    }
    let said = _flat(text);
    let words = pt::split(&pt::lower(&said));
    if words.is_empty() || words.len() > *MOST_WORDS || PLAIN_ASK.match_(&said).is_none() {
        return None;
    }
    if A_QUANTITY_ASK.is_match(&said) {
        return None;
    }
    if words.iter().any(|word| !ASK_ONLY.contains(pt::strip_chars(word, "?,.!;:"))) {
        return None;
    }
    let answer = _the_answer_just_given(memory);
    if answer.is_empty() {
        return None;
    }
    let mut step = _a_step(&answer, false);
    if step.is_empty() {
        step = _first_instruction(&answer);
    }
    if step.is_empty() || n_words(&step) < 2 {
        return None;
    }
    if repeat::_handed_back_already(&step, memory) {
        return None;
    }
    let reply = match A_WHEN.search(&said) {
        // They said when, so the reply opens on their own word.
        Some(when) => format!("{} - {}", _capital(when.str(1)), _bare_step(&step)),
        None => fmt(&pick(&PLAIN_LEADS, memory, text), &[("step", &_bare_step(&step))]),
    };
    Some(repeat::_handed_back(memory, &reply))
}

/// Python: bot/memory/detail.py:1084 `_clauses(move)`
///
/// The move cut where a clause ends, ignoring a comma inside brackets.
pub fn _clauses(r#move: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut piece: Vec<String> = Vec::new();
    let mut depth: i64 = 0;
    for chunk in CLAUSE.split(r#move) {
        depth += pt::count(&chunk, "(") as i64 - pt::count(&chunk, ")") as i64;
        if !piece.is_empty() && depth != 0 {
            piece.push(chunk);
            continue;
        }
        if !piece.is_empty() {
            out.push(piece.join(", "));
        }
        piece = vec![chunk];
    }
    if !piece.is_empty() {
        out.push(piece.join(", "));
    }
    out
}

/// Python: bot/memory/detail.py:1101 `_first_clause(step)`
///
/// The shortest form of a step: its first clause, with a full stop.
pub fn _first_clause(step: &str) -> String {
    _clauses(pt::rstrip_chars(step, "."))
        .iter()
        .map(|p| pt::strip_chars(p, " ,;:-.").to_string())
        .find(|p| !p.is_empty())
        .map(|first| format!("{first}."))
        .unwrap_or_default()
}

/// Python: bot/memory/detail.py:1124 `_one_of_a_list(clauses, groups, at)`
///
/// Whether a clause is one item of a list of things, or a thing a dash-group qualifies.
pub fn _one_of_a_list(clauses: &[String], groups: &[String], at: usize) -> bool {
    clauses.len() > 1 || at + 1 < groups.len()
}

/// Python: bot/memory/detail.py:1136 `_head_word(detail)`
///
/// The word the detail names its thing with ("in a rented flat" -> "flat").
pub fn _head_word(detail: &str) -> String {
    _words(&PREPOSITION.sub("", detail)).into_iter().filter(|w| !VAGUE.contains(w)).last().unwrap_or_default()
}

/// Python: bot/memory/detail.py:1143 `_subject_words(memory)`
///
/// The words the chat's thing can be named by, best first (at most six).
pub fn _subject_words(memory: &Memory) -> Vec<String> {
    let (asked, said) = match memory.turns.last() {
        Some(last) => (last.user.clone(), last.bot.clone()),
        None => (String::new(), String::new()),
    };
    let mut out: Vec<String> = Vec::new();
    for source in [crate::memory::topics::subject(memory), asked] {
        for word in pt::split(&source) {
            let word = NOT_A_LETTER.sub("", &pt::lower(&word));
            if pt::len(&word) > 2 && !VAGUE.contains(&word) && !TOPICS_STOPWORDS.contains(&word) {
                out.push(word);
            }
        }
    }
    let said = pt::lower(&said);
    let (mut seen, mut both, mut rest) = (HashSet::new(), Vec::new(), Vec::new());
    for word in out {
        if !seen.insert(word.clone()) {
            continue;
        }
        if Re::new(&format!(r"\b{}\b", crate::py::re::escape(&word)), 0).is_match(&said) {
            both.push(word);
        } else {
            rest.push(word);
        }
    }
    both.into_iter().chain(rest).take(6).collect()
}

/// Python: bot/memory/detail.py:1170 `_the_entry_reply(found)`
///
/// The text of an entry, or "" when there is none to say (a callable is not read here).
pub fn _the_entry_reply(found: Option<&crate::knowledge::entry::Entry>) -> String {
    match found.map(|entry| &entry.reply) {
        Some(crate::knowledge::entry::Reply::Text(text)) => text.clone(),
        _ => String::new(),
    }
}

/// `{word.rstrip("s") or word for word in _words(text) if ...}`.
fn plural_stems(words: impl Iterator<Item = String>) -> HashSet<String> {
    words.map(|w| {
        let bare = pt::rstrip_chars(&w, "s").to_string();
        if bare.is_empty() { w } else { bare }
    })
    .collect()
}

/// Python: bot/memory/detail.py:1176 `_another_thing(detail, said)`
///
/// True when the detail names something the answer in hand never speaks of.
pub fn _another_thing(detail: &str, said: &str) -> bool {
    let head = _head_word(detail);
    if head.is_empty() || NOT_A_THING.contains(&head) {
        return false;
    }
    // A detail that opens on a preposition says where the thing is or what it is for.
    if PREPOSITION.match_(&_flat(detail)).is_some() {
        return false;
    }
    let want = plural_stems(_words(detail).into_iter().filter(|w| !VAGUE.contains(w)));
    let have = plural_stems(_words(said).into_iter());
    !(want.is_empty() || !want.is_disjoint(&have))
}

/// Python: bot/memory/detail.py:1199 `_what_it_is_worth(detail, memory)`
///
/// "What about a dehumidifier" is the question "is one worth it", when the base keeps a card so.
pub fn _what_it_is_worth(detail: &str, memory: &Memory) -> Option<String> {
    let head = _head_word(detail);
    if head.is_empty() {
        return None;
    }
    let reply = py_try(|| {
        let understood: Understood = crate::understanding::understand(&format!("should i get {detail}"), Some(&Memory::new()));
        let found = crate::knowledge::lookup::find(&understood, None, None, false)?;
        Some(crate::modules::answers::_payload(found, memory, &understood))
    })
    .flatten()?;
    if !_says_about(&reply, &head) {
        return None;
    }
    if repeat::_already_repeated(&reply, memory) {
        return None;
    }
    Some(reply)
}

/// Python: bot/memory/detail.py:1226 `_says_about(reply, head)`
///
/// True when an entry's answer is about the thing itself, and says so as it opens.
pub fn _says_about(reply: &str, head: &str) -> bool {
    if pt::len(reply) < 40 {
        return false;
    }
    let first = sentences_of(pt::strip(reply)).into_iter().next().unwrap_or_default();
    pt::lower(&first).contains(head)
}

/// Python: bot/memory/detail.py:1234 `_the_thing_itself(detail, memory)`
///
/// The knowledge base's answer for the thing the detail names, asked beside the subject.
pub fn _the_thing_itself(detail: &str, memory: &Memory) -> Option<String> {
    let head = _head_word(detail);
    if head.is_empty() || NOT_A_THING.contains(&head) {
        return None;
    }
    for word in _subject_words(memory) {
        if word == head {
            continue;
        }
        let Some(reply) = py_try(|| {
            let understood = crate::understanding::understand(&format!("{word} {head}"), Some(&Memory::new()));
            _the_entry_reply(crate::knowledge::lookup::find(&understood, None, None, false))
        }) else {
            continue;
        };
        // The entry has to answer about the thing itself, and say so as it opens.
        let first = sentences_of(pt::strip(&reply)).into_iter().next().unwrap_or_default();
        if pt::len(&reply) < 40 || !pt::lower(&first).contains(&head) {
            continue;
        }
        if repeat::_already_repeated(&reply, memory) {
            continue;
        }
        return Some(reply);
    }
    None
}

/// Python: bot/memory/detail.py:1275 `_asks_about_a_new_thing(text, detail, said)`
///
/// True for "what about a case" whose thing the answer never speaks of.
pub fn _asks_about_a_new_thing(text: &str, detail: &str, said: &str) -> bool {
    let flat = _flat(text);
    let Some(opened) = ABOUT_OPEN.match_(&flat) else {
        return false;
    };
    // Only a thing said with an indefinite article is a thing of its own.
    if AN_INDEFINITE.match_(&flat[opened.end()..]).is_none() {
        return false;
    }
    use crate::memory::topics::stem;
    let want: HashSet<String> =
        _words(detail).iter().filter(|w| !VAGUE.contains(*w) && !NAME_ONLY.contains(*w)).map(|w| stem(w)).collect();
    if want.is_empty() {
        return true;
    }
    let have: HashSet<String> = _words(said).iter().map(|w| stem(w)).collect();
    want.is_disjoint(&have)
}

/// Python: bot/memory/detail.py:1302 `_step_about(said, detail)`
///
/// The part of the answer that is about the thing the detail names, or None.
pub fn _step_about(said: &str, detail: &str) -> Option<String> {
    let want: HashSet<String> = _words(detail).into_iter().filter(|w| !VAGUE.contains(w) && !NAME_ONLY.contains(w)).collect();
    if want.is_empty() {
        return None;
    }
    if _is_a_span(detail) {
        return None; // a span narrows the answer by how long, not by name
    }
    let sentences: Vec<String> = sentences_of(&_flat(said)).iter().map(|s| pt::strip(s).to_string()).filter(|s| !s.is_empty()).collect();
    for sentence in &sentences {
        let groups: Vec<String> = DASH_GROUPS.split(sentence).iter().map(|g| pt::strip(g).to_string()).collect();
        for (at, group) in groups.iter().enumerate() {
            let clauses: Vec<String> = group.split(',').map(|c| pt::strip(c).to_string()).filter(|c| !c.is_empty()).collect();
            for (pos, clause) in clauses.iter().enumerate() {
                let have: HashSet<String> = _words(clause).into_iter().collect();
                let met = have.intersection(&want).count();
                if !(want.is_subset(&have) || met * 2 > want.len()) {
                    continue;
                }
                if !_one_of_a_list(&clauses, &groups, at) {
                    continue;
                }
                if pos == clauses.len() - 1 && at + 1 < groups.len() {
                    let qualifier = pt::strip_chars(&groups[at + 1], " .,;:-").to_string();
                    if (2..=*STEP_WORDS).contains(&n_words(&qualifier)) {
                        return Some(format!("{qualifier}."));
                    }
                }
                let mut clause = clause.clone();
                let name_only: HashSet<String> = want.union(&NAME_ONLY).cloned().collect();
                if have.is_subset(&name_only) && pos + 1 < clauses.len() {
                    // The clause is only the name, so the clause after it is the step.
                    clause = format!("{}, {}", clauses[pos], clauses[pos + 1]);
                }
                let clause = JOIN_LEAD.sub("", pt::strip_chars(&clause, " .,;:-"));
                if clause.is_empty() || n_words(&clause) > *STEP_WORDS {
                    return None;
                }
                return Some(format!("{clause}."));
            }
        }
    }
    None
}

/// Python: bot/memory/detail.py:1356 `_a_branch(sentence)`
///
/// Whether the sentence is a case and what happens in it, rather than a move.
pub fn _a_branch(sentence: &str) -> bool {
    !crate::memory::condition::conditions(sentence).is_empty()
}

/// Python: bot/memory/detail.py:1367 `_without_branches(said)`
///
/// The answer with the sentences that are cases of it taken out.
pub fn _without_branches(said: &str) -> String {
    sentences_of(&_flat(said))
        .iter()
        .map(|s| pt::strip(s).to_string())
        .filter(|s| !s.is_empty())
        .filter(|s| !_a_branch(s))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Python: bot/memory/detail.py:1374 `_the_move_itself(move)`
///
/// The move without the half a colon puts after it.
pub fn _the_move_itself(r#move: &str) -> String {
    let head = pt::strip_chars(r#move.split(':').next().unwrap_or(""), " ,;.-");
    if n_words(head) >= 2 { head.to_string() } else { r#move.to_string() }
}

/// Python: bot/memory/detail.py:1425 `_the_heading(said)`
///
/// The label a card opens with ("Air fryer:", "Slow cooker:"), or "" when it has none.
pub fn _the_heading(said: &str) -> String {
    let text = pt::strip(said);
    let Some(found) = HEADING.match_(text) else {
        return String::new();
    };
    let label = pt::rstrip_chars(pt::rstrip(found.as_str()), ":").to_string();
    let words = pt::split_ref(&label);
    if words.is_empty() || words.len() > *HEADING_WORDS {
        return String::new();
    }
    if words.iter().any(|w| CLAUSE_WORDS.contains(pt::strip_chars(&pt::lower(w), "*_\"',"))) {
        return String::new();
    }
    label
}

/// Python: bot/memory/detail.py:1440 `_is_the_label_itself(text, label)`
///
/// True when the words are the label and nothing else, whatever the punctuation.
pub fn _is_the_label_itself(text: &str, label: &str) -> bool {
    pt::strip_chars(&_flat(text), " .,;:") == pt::strip_chars(&_flat(label), " .,;:")
}

/// Python: bot/memory/detail.py:1445 `_without_the_heading(said)`
///
/// The answer without the label in front of it, or the answer itself when it has none.
pub fn _without_the_heading(said: &str) -> String {
    let label = _the_heading(said);
    if label.is_empty() {
        return said.to_string();
    }
    let stripped = pt::strip(said);
    let after = pt::slice(stripped, pt::len(&label) as isize, None);
    pt::lstrip(pt::lstrip_chars(pt::lstrip(&after), ":")).to_string()
}

/// Python: bot/memory/detail.py:1460 `_first_instruction(said)`
///
/// The answer's first sentence that opens on something to do, cut to its first clause, or "".
pub fn _first_instruction(said: &str) -> String {
    for sentence in sentences_of(&_flat(said)) {
        let clauses: Vec<String> = _clauses(&sentence).iter().map(|c| pt::strip_chars(c, " ,;:.-").to_string()).collect();
        let mut first = clauses.into_iter().find(|c| !c.is_empty()).unwrap_or_default();
        if first.is_empty() || !IMPERATIVES.contains(&pt::lower(pt::split_ref(&first)[0])) {
            continue;
        }
        if n_words(&first) > *STEP_WORDS {
            // The method in brackets is not the step.
            let bare = pt::squash(&BRACKETED.sub("", &first));
            if !bare.is_empty() {
                first = bare;
            }
        }
        if (2..=*STEP_WORDS).contains(&n_words(&first)) {
            return format!("{first}.");
        }
    }
    String::new()
}

/// Python: bot/memory/detail.py:1482 `_opening_said(said)`
///
/// The answer's first sentence, or its first clause when that is long.
pub fn _opening_said(said: &str) -> String {
    let first = sentences_of(&_flat(said)).into_iter().next().unwrap_or_default();
    let first = pt::strip(&first);
    if first.is_empty() {
        return String::new();
    }
    if n_words(first) <= *STEP_WORDS {
        return if pt::ends_with_any(first, &[".", "!", "?"]) { first.to_string() } else { format!("{first}.") };
    }
    _first_clause(first)
}

/// Python: bot/memory/detail.py:1492 `_is_a_span(detail)`
///
/// True when the detail is only how long a thing has been going on, with no name in it.
pub fn _is_a_span(detail: &str) -> bool {
    let want: Vec<String> = _words(detail).into_iter().filter(|w| !VAGUE.contains(w) && !NAME_ONLY.contains(w)).collect();
    !want.is_empty() && want.iter().all(|w| SPAN_WORDS.contains(w) || NUMBER_WORDS.contains(w) || SPAN_WORDS_ABOUT.contains(w))
}

/// Python: bot/memory/detail.py:1498 `_a_span_step(said, detail)`
///
/// The move in the answer that speaks of a span, when the detail is a span, or "".
pub fn _a_span_step(said: &str, detail: &str) -> String {
    if !_words(detail).iter().any(|w| SPAN_WORDS.contains(w)) {
        return String::new();
    }
    for sentence in sentences_of(&_flat(said)).iter().rev() {
        if !A_DURATION.is_match(sentence) {
            continue;
        }
        for piece in _clauses(sentence) {
            let piece = pt::strip_chars(&LEAD_AND.sub("", pt::strip_chars(&piece, " ,;:.-")), " ,;:.-").to_string();
            if (2..=*STEP_WORDS).contains(&n_words(&piece)) {
                return format!("{piece}.");
            }
        }
    }
    String::new()
}

/// Python: bot/memory/detail.py:1518 `_opening_of(said)`
///
/// The first clause of the answer's opening sentence, when that sentence says what to do.
pub fn _opening_of(said: &str) -> String {
    let parts: Vec<String> = REPEAT_SENTENCE.split(pt::strip(said)).iter().map(|p| pt::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    if parts.len() < 2 {
        return String::new();
    }
    let mut first = _clauses(&parts[0]).iter().map(|c| pt::strip_chars(c, " ,;:.-").to_string()).find(|c| !c.is_empty()).unwrap_or_default();
    if !first.is_empty() && n_words(&first) > *STEP_WORDS {
        let bare = pt::squash(&BRACKETED.sub("", &first));
        if !bare.is_empty() {
            first = bare;
        }
    }
    if first.is_empty() || n_words(&first) < 2 || n_words(&first) > *STEP_WORDS {
        return String::new();
    }
    if !IMPERATIVES.contains(&pt::lower(pt::split_ref(&first)[0])) {
        return String::new();
    }
    if pt::ends_with_any(&first, &[".", "!", "?"]) { first } else { format!("{first}.") }
}

/// Python: bot/memory/detail.py:1539 `_names_nothing(detail)`
///
/// True when the phrase opens on a preposition and names no place and no use.
pub fn _names_nothing(detail: &str) -> bool {
    let flat = _flat(detail);
    let Some(opening) = PREPOSITION.match_(&flat) else {
        return false;
    };
    NAMES_NOTHING.match_(pt::strip(&flat[opening.end()..])).is_some()
}

/// Python: bot/memory/detail.py:1555 `_says_where(detail)`
///
/// True when the detail narrows the answer by where or by what for, and not by when.
pub fn _says_where(detail: &str) -> bool {
    let flat = _flat(detail);
    PREPOSITION.match_(&flat).is_some() && A_WHEN.match_(&flat).is_none() && !_names_nothing(&flat) && !A_SUBJECT_OF_ITS_OWN.is_match(&flat)
}

/// Python: bot/memory/detail.py:1568 `_a_step(said, questions_are_steps=False)`
///
/// The one move out of the answer, cut at a clause boundary so it is not the answer again.
/// (Contract: `questions_are_steps` is a bool and the step a str, "" for none.)
pub fn _a_step(said: &str, questions_are_steps: bool) -> String {
    let found = repeat::a_move(said, questions_are_steps).unwrap_or_default();
    let mut r#move = pt::strip_chars(&found, " ,;:.-").to_string();
    if r#move.is_empty() {
        return String::new();
    }
    // The first thing to do is the opening sentence's first clause when the answer opens on an
    // instruction.
    let label = _the_heading(said);
    let opening = _opening_of(said);
    if !opening.is_empty() && !(!label.is_empty() && _is_the_label_itself(&opening, &label)) {
        r#move = opening;
    }
    if !label.is_empty() {
        // The card's heading is not the move.
        let without = _without_the_heading(&r#move);
        if !without.is_empty() {
            r#move = without;
        }
        if _is_the_label_itself(&r#move, &label) {
            let itself = _the_move_itself(&_without_the_heading(said));
            if !itself.is_empty() {
                r#move = itself;
            }
        }
    }
    let pieces: Vec<String> = _clauses(&_the_move_itself(&r#move)).iter().map(|p| pt::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    let mut kept = pieces.first().cloned().unwrap_or_else(|| r#move.clone());
    for piece in pieces.iter().skip(1) {
        if n_words(&kept) >= *SHORT_STEP {
            break;
        }
        let trial = format!("{kept}, {piece}");
        if n_words(&trial) > *STEP_WORDS {
            break;
        }
        kept = trial;
    }
    let kept = pt::strip_chars(&kept, " ,;:-.").to_string();
    let label = _the_heading(said);
    if !kept.is_empty() && !label.is_empty() && _is_the_label_itself(&kept, &label) {
        // The step is the card's own heading and nothing else: read it off the body instead.
        let body = _without_the_heading(said);
        if !body.is_empty() && body != said {
            return _a_step(&body, questions_are_steps);
        }
    }
    if kept.is_empty() { String::new() } else { format!("{kept}.") }
}

/// Python: bot/memory/detail.py:1617 `_capital(said)`
pub fn _capital(said: &str) -> String {
    pt::upper_first(pt::strip(said))
}

/// Python: bot/memory/detail.py:1719 `stands_alone(sentence)`
///
/// Whether a sentence can be read as the reply, with nothing before it on the screen.
pub fn stands_alone(sentence: &str) -> bool {
    let said = pt::strip(sentence);
    !said.is_empty() && OPENS_MID_THOUGHT.match_(said).is_none()
}

/// Python: bot/memory/detail.py:1725 `_report_stem(word)`
///
/// The word without the ending a verb or a plural wears, so "drops" meets "dropping".
pub fn _report_stem(word: &str) -> String {
    let mut word = word.to_string();
    for ending in ["ing", "ed", "es", "s"] {
        if word.ends_with(ending) && pt::len(&word) > ending.len() + 2 {
            word.truncate(word.len() - ending.len());
            break;
        }
    }
    let chars: Vec<char> = word.chars().collect();
    let n = chars.len();
    if n > 3 && chars[n - 1] == chars[n - 2] && !"aeiou".contains(chars[n - 1]) {
        word = chars[..n - 1].iter().collect();
    }
    word
}

/// Python: bot/memory/detail.py:1736 `_report_content(text)`
pub fn _report_content(text: &str) -> Vec<String> {
    _words(text).iter().filter(|w| !REPORT_FILLER.contains(*w) && pt::len(w) > 2).map(|w| _report_stem(w)).collect()
}

/// Python: bot/memory/detail.py:1741 `a_report_about_the_thing(text, memory)`
///
/// The sentence of the last answer that speaks to this report about its thing, or "".
pub fn a_report_about_the_thing(text: &str, memory: &Memory) -> String {
    let said = pt::lower(&_flat(text));
    if said.is_empty() || n_words(&said) > *REPORT_WORDS {
        return String::new();
    }
    if REPORT_ASKS.is_match(&said) || REPORT_OPENS_ON_THE_THING.match_(&said).is_none() {
        return String::new();
    }
    if REPORT_NEGATES.is_match(&said) {
        return String::new();
    }
    let turns = &memory.turns;
    let Some(last) = turns.last() else {
        return String::new();
    };
    let answer = &last.bot;
    if pt::len(answer) < 60 {
        return String::new();
    }
    let lowered = pt::lower(answer);
    if NOT_AN_ANSWER.iter().any(|hedge| lowered.contains(hedge.as_str())) {
        return String::new();
    }
    let about: HashSet<String> = _report_content(&said).into_iter().filter(|w| pt::len(w) > 3).collect();
    if about.is_empty() {
        return String::new();
    }
    let mut best = String::new();
    let mut best_shared = 0usize;
    for sentence in REPORT_SENTENCE.split(answer) {
        let sentence = pt::strip(&sentence).to_string();
        if n_words(&sentence) < 5 {
            continue;
        }
        if sentence.ends_with('?') || !REPORT_BEHAVIOUR.is_match(&sentence) {
            continue;
        }
        if !stands_alone(&sentence) {
            continue;
        }
        if A_REFERRAL.is_match(&sentence) {
            continue;
        }
        let shared = about.intersection(&_report_content(&sentence).into_iter().collect()).count();
        if shared > best_shared {
            best = sentence;
            best_shared = shared;
        }
    }
    if best.is_empty() || best_shared < 1 {
        return String::new();
    }
    // A sentence the person has already had on its own is not handed back a second time.
    let flat_best = pt::lower(&_flat(&best));
    if turns[turns.len().saturating_sub(2)..].iter().any(|t| pt::lower(&_flat(&t.bot)) == flat_best) {
        return String::new();
    }
    _capital(&best)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Values from CPython.
    #[test]
    fn details_as_python_reads_them() {
        let read = |s: &str| the_detail(s);
        assert_eq!(read("its on the bedroom ones mainly").as_deref(), Some("on the bedroom ones mainly"));
        assert_eq!(read("with two kids").as_deref(), Some("with two kids"));
        assert_eq!(read("its been about three weeks now").as_deref(), Some("for about three weeks now"));
        assert_eq!(read("in the end i called someone"), None);
        assert_eq!(read("like keys and names").as_deref(), Some("keys and names"));
        assert_eq!(read("we only have 3 days"), None);
        assert_eq!(read("its 2kg").as_deref(), Some("2kg"));
        assert_eq!(read("it catches at the top").as_deref(), Some("at the top"));
        assert_eq!(read("im thirsty"), None);
        assert_eq!(_clauses("unscrew it (bowl down, then pull), and rinse; dry it"), ["unscrew it (bowl down", "then pull)", "and rinse", "dry it"]);
        assert_eq!(["dropping", "drops", "stopped", "running"].map(_report_stem), ["drop", "drop", "stop", "run"]);
    }
}
