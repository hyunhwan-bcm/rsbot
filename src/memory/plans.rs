//! Port of `bot/memory/plans.py`: a plan announced is the question the plan implies.
//!
//! "im getting married in june" is read as "how do i plan a wedding" and answered from the entry
//! it reaches, when the line is the person's own plan said as a statement and the rewrite reaches
//! an entry about planning. The turn after, a question about one part of the plan gets that
//! part's sentence of the paragraph rather than the paragraph again.
#![allow(non_snake_case, clippy::all)]

use crate::knowledge::entry::{Entry, KeyPart, Reply};
use crate::knowledge::lookup;
use crate::memory::mem7::{self, inline_re, res, sets};
use crate::py::text as pt;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;

res!("plans"; CALENDAR_NOW, _PLANNING, _MOVING, _MARRYING, _NEW_JOB, _WORK_WHEN, _ASKING, _TURNED, _DENIED, _ABOUT_A_DEATH, _A_LENGTH_ASK, _SPAN_HEAD, _A_CLAUSE_BREAK, _A_FIRST_ASK, _PLAN_ASK);
sets!("plans"; PLANNING_KEYS, _PART_WORDS, _ASK_WORDS, _ORDER_WORDS);
inline_re! {
    NOT_A_WORD_CHAR = r#"[\s.,!?;:'"()]"#, 0;
    TRAILING_TIME = r"\s+(?:next|this|in|on|for)\s+\w+.*$", 0;
    CLAUSES = r",\s*|;\s*|\s+and\s+", 0;
    WORDS = r"[a-z']+", 0;
    SENTENCE_END = r"(?<=[.!?])\s+", 0;
}

/// Python: bot/memory/plans.py:61 `_a_first_day(text)`
///
/// True when the line is a first day at a new job, not a shift or a job on the house.
pub fn _a_first_day(text: &str) -> bool {
    let Some(found) = _NEW_JOB.search(text) else {
        return false;
    };
    if pt::lower(found.name_str("what")) == "job" {
        return true;
    }
    let after = found.name_str("after");
    if NOT_A_WORD_CHAR.sub("", after).is_empty() {
        return true;
    }
    _WORK_WHEN.match_(after).is_some()
}

/// Python: bot/memory/plans.py:100 `as_question(text)`
///
/// The question a plan announcement implies, or None when the line is not one.
pub fn as_question(text: &str) -> Option<String> {
    _implied(text).0
}

/// Python: bot/memory/plans.py:106 `_implied(text)`
///
/// The question the line implies, and whether it is one of the kinds read by hand.
pub fn _implied(text: &str) -> (Option<String>, bool) {
    let clean = pt::squash(&pt::lower(text));
    if clean.is_empty() || _ASKING.is_match(&clean) || _TURNED.is_match(&clean) || _DENIED.is_match(&clean) || _ABOUT_A_DEATH.is_match(&clean) {
        return (None, false);
    }
    // `_ASK_FOR`, in its order.
    if _MARRYING.is_match(&clean) {
        return (Some("how do i plan a wedding".into()), true);
    }
    if _MOVING.is_match(&clean) {
        return (Some("what do i need to do to move house".into()), true);
    }
    if _a_first_day(&clean) {
        return (Some("first day at a new job".into()), true);
    }
    if let Some(found) = _PLANNING.search(&clean) {
        let thing = pt::strip_chars(found.name_str("thing"), " .!,");
        if thing.starts_with("to ") {
            return (None, false); // an intention, not a thing to plan
        }
        let thing = pt::strip(&TRAILING_TIME.subn("", thing, 1)).to_string();
        if !thing.is_empty() {
            if ["my ", "our ", "the "].iter().any(|p| thing.starts_with(p)) {
                return (Some(format!("how do i plan {thing}")), false);
            }
            return (Some(format!("how do i plan a {thing}")), false);
        }
    }
    (None, false)
}

/// The reading `understand(question, memory)` gives.
fn reading_of(question: &str, memory: &Memory) -> Understood {
    crate::understanding::understand(question, Some(memory))
}

/// Python: bot/memory/plans.py:135 `_the_reply(entry, question, memory)`
///
/// The entry's answer said, the way the composer will say it, or "".
///
/// CONTRACT: `memory` is `&Memory` (pending.rs calls it so); an entry's reply function may write
/// to the memory it is handed, so it gets a copy here, and `answer` uses `the_reply_on`, which
/// hands it the real one as PyBot does.
pub fn _the_reply(entry: &'static Entry, question: &str, memory: &Memory) -> String {
    let mut copy = memory.clone();
    the_reply_on(entry, question, &mut copy)
}

/// `_the_reply` with the memory the reply function may write to.
fn the_reply_on(entry: &'static Entry, question: &str, memory: &mut Memory) -> String {
    match &entry.reply {
        Reply::Text(text) => text.clone(),
        Reply::Choices(options) => mem7::pick(options, memory, ""),
        Reply::Fn(f) => {
            let reading = match mem7::py_try(|| reading_of(question, memory)) {
                Some(reading) => reading,
                None => return String::new(),
            };
            match crate::knowledge::registry::try_reply(f, memory, &reading) {
                Ok(Some(reply)) => reply,
                _ => String::new(),
            }
        }
    }
}

/// Python: bot/memory/plans.py:155 `answer(text, memory=None)`
///
/// The entry's answer to the question the plan implies, or None to leave the line as it was.
///
/// CONTRACT: the skeleton gave this no return type and `&Memory`; PyBot returns the answer or None,
/// and an entry's reply function is handed the real memory (it may write to it), so it is
/// `&mut Memory`. Its caller (store_replies) holds `&mut Memory`.
pub fn answer(text: &str, memory: &mut Memory) -> Option<String> {
    let (question, by_hand) = _implied(text);
    let question = question?;
    let entry = mem7::py_try(|| lookup::find(&reading_of(&question, memory), None, None, false))??;
    if !by_hand && !_about_planning(entry) {
        // The entry is about the thing, not about planning it.
        return None;
    }
    let reply = the_reply_on(entry, &question, memory);
    if pt::strip(&reply).is_empty() {
        return None;
    }
    // A reply that opens with a fragment leans on the question's words.
    if mem7::is_lower(mem7::first_chars(&reply, 1)) {
        return None;
    }
    Some(pt::strip(&reply).to_string())
}

/// Python: bot/memory/plans.py:191 `_about_planning(entry)`
///
/// Whether the entry was written for a question about planning the thing.
pub fn _about_planning(entry: &'static Entry) -> bool {
    let mut words: HashSet<String> = HashSet::new();
    for part in &entry.key {
        let pieces: Vec<String> = match part {
            KeyPart::Word(word) => vec![word.clone()],
            KeyPart::Any(words) => words.clone(),
        };
        for piece in pieces {
            words.extend(pt::split(&pt::lower(&piece).replace('\'', "")));
        }
    }
    words.iter().any(|w| PLANNING_KEYS.contains(w))
}

/// Python: bot/memory/plans.py:232 `_the_clauses(detail)`
pub fn _the_clauses(detail: &str) -> Vec<String> {
    CLAUSES.split(detail).iter().map(|part| pt::strip_chars(part, " .;").to_string()).filter(|part| !part.is_empty()).collect()
}

/// Python: bot/memory/plans.py:237 `_the_step(detail, wanted, first_only=False)`
///
/// The clause of a sentence that names the part asked about, else the first clause.
pub fn _the_step(detail: &str, wanted: &HashSet<String>, first_only: bool) -> String {
    let clauses = _the_clauses(detail);
    let fallback = || clauses.first().cloned().unwrap_or_else(|| pt::strip_chars(detail, " .").to_string());
    if first_only {
        return fallback();
    }
    let (mut best, mut best_hit) = (None, 0);
    for clause in &clauses {
        let hit = _part_words(clause).iter().filter(|w| wanted.contains(*w)).count();
        if hit > best_hit {
            best = Some(clause.clone());
            best_hit = hit;
        }
    }
    best.unwrap_or_else(fallback)
}

/// Python: bot/memory/plans.py:254 `_the_first_step(sentence)`
///
/// The opening step of the paragraph, said as a step rather than as the paragraph again.
pub fn _the_first_step(sentence: &str) -> Option<String> {
    let text = pt::rstrip_chars(&pt::squash(sentence), ".").to_string();
    if text.is_empty() {
        return None;
    }
    let mut step = String::new();
    for part in _A_CLAUSE_BREAK.split(&text).iter().map(|p| pt::strip(p).to_string()).filter(|p| !p.is_empty()) {
        step = if step.is_empty() { part } else { format!("{step}, {part}") };
        if mem7::word_count(&step) >= 4 {
            break;
        }
    }
    if step.is_empty() {
        return None;
    }
    if mem7::word_count(&step) >= mem7::word_count(&text) {
        step = text;
    }
    Some(format!("Start with this: {}.", mem7::low1(&step)))
}

/// Python: bot/memory/plans.py:279 `_a_countdown_step(head, detail, wanted, about_the_start)`
///
/// A span heading with its step, short enough not to be the paragraph read out again.
pub fn _a_countdown_step(head: &str, detail: &str, wanted: &HashSet<String>, about_the_start: bool) -> Option<String> {
    let step = _the_step(detail, wanted, about_the_start);
    if step.is_empty() {
        return None;
    }
    let said = format!("{}{}", pt::rstrip_chars(pt::strip(head), ":"), if about_the_start { " is when to start:" } else { ":" });
    Some(format!("{said} {}.", mem7::low1(&step)))
}

/// Python: bot/memory/plans.py:299 `_another_thing(wanted, said_words)`
///
/// True when the line names something the paragraph itself never mentions.
pub fn _another_thing(wanted: &HashSet<String>, said_words: &HashSet<String>) -> bool {
    wanted.iter().any(|w| !_ASK_WORDS.contains(w) && pt::len(w) > 3 && !said_words.contains(w))
}

/// Python: bot/memory/plans.py:322 `_part_words(text)`
pub fn _part_words(text: &str) -> HashSet<String> {
    WORDS.findall_str(&pt::squash(&pt::lower(text))).into_iter().filter(|w| !_PART_WORDS.contains(w) && pt::len(w) > 2).collect()
}

/// Python: bot/memory/plans.py:327 `_was_a_plan(line)`
///
/// True when the person's line was a plan announced or a question about planning one.
pub fn _was_a_plan(line: &str) -> bool {
    let clean = pt::squash(&pt::lower(line));
    if clean.is_empty() {
        return false;
    }
    _implied(&clean).0.is_some() || _PLAN_ASK.is_match(&clean)
}

/// Python: bot/memory/plans.py:341 `follow_up(text, memory)`
///
/// The sentence of the plan answer this line asks about, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the sentence or None.
pub fn follow_up(text: &str, memory: &Memory) -> Option<String> {
    // "what day is it" asks about today, not a part of the plan.
    if CALENDAR_NOW.match_(pt::strip_chars(&pt::squash(text), " .!?;,")).is_some() {
        return None;
    }
    let last = memory.turns.last()?;
    let reply = pt::squash(&pt::lower(&last.bot));
    let mut said = String::new();
    if let Some(kept) = memory.get("plan_answer").filter(|k| k.is_dict()) {
        if _was_a_plan(&mem7::text_of(kept.get("user"))) {
            let announced = pt::squash(&pt::lower(&mem7::text_of(kept.get("user"))));
            if !announced.is_empty() && announced == pt::squash(&pt::lower(&last.user)) {
                said = pt::squash(&mem7::text_of(kept.get("text")));
            }
        }
    }
    if said.is_empty() {
        let told = memory.get("entry_answer").filter(|t| t.truthy());
        said = mem7::text_of(told.and_then(|t| t.get("text")));
        let whole = pt::squash(&pt::lower(&said));
        if said.is_empty() || reply.is_empty() || !(reply.contains(mem7::first_chars(&whole, 60)) || whole.contains(mem7::first_chars(&reply, 60))) {
            return None; // the paragraph was not what they were given
        }
        if last.module.as_deref() != Some("memory_store") && !_was_a_plan(&last.user) {
            return None;
        }
    }
    if !(pt::rstrip(text).ends_with('?') || _ASKING.is_match(text) || _A_FIRST_ASK.is_match(text)) {
        return None; // a statement is not asking about a part of it
    }
    let wanted = _part_words(text);
    let sentences: Vec<String> = SENTENCE_END.split(&said).iter().map(|p| pt::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    let (head, colon, detail) = match sentences.first() {
        Some(first) => {
            let (h, c, d) = pt::partition(first, ":");
            (h.to_string(), !c.is_empty(), d.to_string())
        }
        None => (String::new(), false, String::new()),
    };
    // "what do i do first" asks for where the paragraph starts.
    let said_words = _part_words(&said);
    if _A_FIRST_ASK.is_match(text) {
        let named: HashSet<String> = wanted.iter().filter(|w| !_ORDER_WORDS.contains(*w)).cloned().collect();
        if !_another_thing(&named, &said_words) {
            if colon && _SPAN_HEAD.is_match(&head) && !pt::strip(&detail).is_empty() {
                return _a_countdown_step(&head, &detail, &HashSet::new(), false);
            }
            return sentences.first().and_then(|first| _the_first_step(first));
        }
    }
    if wanted.is_empty() {
        return None;
    }
    let span = colon && _SPAN_HEAD.is_match(&head);
    let mut opening: Option<String> = None;
    let mut asked_about: Vec<(String, HashSet<String>)> = Vec::new();
    if colon && !pt::strip(&detail).is_empty() {
        let o = pt::upper_first(pt::strip(&detail));
        let mut words = _part_words(&detail);
        words.extend(_part_words(&head));
        asked_about.push((o.clone(), words));
        opening = Some(o);
    }
    asked_about.extend(sentences.iter().skip(1).map(|s| (s.clone(), _part_words(s))));
    let (mut best, mut best_hit): (Option<String>, usize) = (None, 0);
    for (sentence, words) in &asked_about {
        if reply.starts_with(mem7::first_chars(&pt::squash(&pt::lower(sentence)), 40)) {
            continue; // that sentence is what they have just been given
        }
        let hit = words.iter().filter(|w| wanted.contains(*w)).count();
        if hit > 0 && (best.is_none() || hit >= best_hit) {
            best = Some(sentence.clone());
            best_hit = hit;
        }
    }
    let Some(best) = best else {
        // Nothing names the part, and the line asks how long: the countdown's first step.
        if span && _A_LENGTH_ASK.is_match(text) && !_another_thing(&wanted, &said_words) {
            return _a_countdown_step(&head, &detail, &wanted, true);
        }
        return None;
    };
    if opening.as_ref() == Some(&best) && span {
        // The heading with the clause that names the part.
        return _a_countdown_step(&head, &detail, &wanted, _A_LENGTH_ASK.is_match(text));
    }
    Some(best)
}
