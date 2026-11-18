//! Port of `bot/modules/chasing_a_body.py`: what moves a chase along when an organisation (or a
//! person of theirs) has gone quiet - the reference to have, the ask in writing, the timescale
//! and the next stage.
#![allow(non_snake_case, clippy::all)]

use super::chat::dyn_re;
use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! res {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.chasing_a_body.", stringify!($n))));)* };
}
macro_rules! vals {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<&'static Value> = LazyLock::new(|| data::get(concat!("bot.modules.chasing_a_body.", stringify!($n))));)* };
}

res!(
    NOT_DONE, HOW_TO_CHASE, RAISED, IVE_TRIED, BARE_CONTINUATION, THE_TIMESCALE_ASK,
    A_PERSON_OWES_A_REPLY, A_PRONOUN_OWES_A_REPLY, A_PRONOUN_DID_THE_SILENT_THING,
    ANSWERING_SOMETHING_ELSE, A_PERSON_DID_THE_SILENT_THING, A_PERSON_WHO_WORKS_THERE,
    A_PERSON_NOT_A_BODY,
);
vals!(
    A_PERSON_AFTER_THE_NAME, ORGANISATIONS, THE_SHAPE, THE_SHAPE_UNNAMED, IVE_TRIED_LEAD,
    THE_TIMESCALE, THE_NEXT_STEP, THE_NEXT_STEP_UNNAMED, A_PERSON_THEIRS, THE_WAIT_FOR_A_PERSON,
);

/// `fallback._choose(options, memory, clean)`, the composer's own chooser. Python passes no
/// seed (`seed_text=None`); the contract's `seed_text` is a `&str`, so it gets "".
fn choose(options: &Value, memory: &Memory, clean: &str) -> String {
    crate::compose::fallback_chain::_choose(&options.strs(), memory, clean, "")
}

/// Python: bot/modules/chasing_a_body.py:196 `_the_shape_unnamed(memory, lowered)`
pub fn _the_shape_unnamed(memory: &Memory, lowered: &str) -> String {
    choose(&THE_SHAPE_UNNAMED, memory, lowered)
}

/// The person's own lines, as typed.
fn user_lines(memory: &Memory) -> Vec<String> {
    memory.turns.iter().map(|t| t.user.clone()).collect()
}

/// Python: bot/modules/chasing_a_body.py:380 `_the_person_before(memory)`
pub fn _the_person_before(memory: &Memory) -> bool {
    let said = user_lines(memory);
    let re = dyn_re(&format!(r"\b(?:my|our)\s+{}\b", A_PERSON_THEIRS.str()), crate::py::re::I);
    said[said.len().saturating_sub(3)..].iter().any(|line| re.search(line).is_some())
}

/// Python: bot/modules/chasing_a_body.py:390 `_the_wait_for_a_person(memory, lowered)`
pub fn _the_wait_for_a_person(memory: &Memory, lowered: &str) -> Option<String> {
    let owes = A_PERSON_OWES_A_REPLY.search(lowered).is_some()
        || A_PERSON_DID_THE_SILENT_THING.search(lowered).is_some()
        || (A_PRONOUN_DID_THE_SILENT_THING.search(lowered).is_some() && _the_person_before(memory))
        || (A_PRONOUN_OWES_A_REPLY.search(lowered).is_some() && _the_person_before(memory));
    if !owes {
        return None;
    }
    if ANSWERING_SOMETHING_ELSE.search(lowered).is_some() {
        return None;
    }
    Some(choose(&THE_WAIT_FOR_A_PERSON, memory, lowered))
}

/// The person's own lines before this one: blank ones and the current line (by its text) left out.
fn lines_before(memory: &Memory, current: &str) -> Vec<String> {
    user_lines(memory)
        .into_iter()
        .filter(|line| !text::strip(line).is_empty() && text::strip(line).to_lowercase() != current)
        .collect()
}

/// Python: bot/modules/chasing_a_body.py:409 `_was_an_unnamed_chase(memory, current)`
pub fn _was_an_unnamed_chase(memory: &Memory, current: &str) -> bool {
    let said = lines_before(memory, current);
    let Some(last) = said.last() else { return false };
    let lowered = text::squash(&last.to_lowercase());
    _the_body(&lowered, &_words(&lowered)).is_none()
        && !_names_its_own_person(&lowered)
        && RAISED.search(&lowered).is_some()
        && NOT_DONE.search(&lowered).is_some()
}

static WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// Python: bot/modules/chasing_a_body.py:421 `_words(text)`
pub fn _words(text_: &str) -> HashSet<String> {
    WORD.findall_str(&text_.to_lowercase()).into_iter().collect()
}

/// Python: bot/modules/chasing_a_body.py:425 `_the_body(lowered, words)`: (the body as said, the
/// reference it goes by).
pub fn _the_body(lowered: &str, words: &HashSet<String>) -> Option<(String, String)> {
    if A_PERSON_WHO_WORKS_THERE.search(lowered).is_some() {
        return None;
    }
    for row in ORGANISATIONS.items() {
        let (triggers, said, reference) = (&row.items()[0], row.items()[1].str(), row.items()[2].str());
        for trigger in triggers.items() {
            let trigger = trigger.str();
            if trigger.contains(' ') {
                if lowered.contains(trigger) {
                    return Some((said.to_string(), reference.to_string()));
                }
            } else if words.contains(trigger) {
                if _the_person_it_belongs_to(lowered, trigger).is_some() {
                    continue;
                }
                return Some((said.to_string(), reference.to_string()));
            }
        }
    }
    None
}

/// Python: bot/modules/chasing_a_body.py:470 `_names_its_own_person(lowered)`
pub fn _names_its_own_person(lowered: &str) -> bool {
    A_PERSON_NOT_A_BODY.search(lowered).is_some()
}

/// Python: bot/modules/chasing_a_body.py:475 `_the_person_it_belongs_to(lowered, trigger)`
pub fn _the_person_it_belongs_to(lowered: &str, trigger: &str) -> Option<String> {
    let re = dyn_re(&format!(r"\b{}\s+(?:(\w+)\s+)?(\w+)", crate::py::re::escape(trigger)), 0);
    let found = re.search(lowered)?;
    for after in [found.get(1), found.get(2)].into_iter().flatten() {
        if !after.is_empty() && A_PERSON_AFTER_THE_NAME.contains_str(after) {
            return Some(after.to_string());
        }
    }
    None
}

/// Python: bot/modules/chasing_a_body.py:493 `_the_body_before(memory, current)`
pub fn _the_body_before(memory: &Memory, current: &str) -> Option<(String, String)> {
    for line in lines_before(memory, current).iter().rev() {
        let lowered = text::squash(&line.to_lowercase());
        if let Some(body) = _the_body(&lowered, &_words(&lowered)) {
            return Some(body);
        }
    }
    None
}

/// Python: bot/modules/chasing_a_body.py:511 `respond(text, memory)` - a runtime hook.
pub fn respond(text_: &str, memory: &mut Memory) -> Option<String> {
    let lowered = text::squash(&text_.to_lowercase());
    if lowered.is_empty() {
        return None;
    }
    if let Some(body) = _the_body(&lowered, &_words(&lowered)) {
        if NOT_DONE.search(&lowered).is_none() && HOW_TO_CHASE.search(&lowered).is_none() {
            return None;
        }
        return Some(_the_shape(&body, memory, &lowered));
    }
    if THE_TIMESCALE_ASK.search(&lowered).is_some() && BARE_CONTINUATION.match_(&lowered).is_none() {
        if let Some(before) = _the_body_before(memory, &lowered) {
            return Some(_the_timescale(&before, memory, &lowered));
        }
    }
    if BARE_CONTINUATION.match_(&lowered).is_some() {
        if let Some(before) = _the_body_before(memory, &lowered) {
            return Some(_the_next_step(&before, memory, &lowered));
        }
        if _was_an_unnamed_chase(memory, &lowered) {
            return Some(_the_next_step_unnamed(memory, &lowered));
        }
        return None;
    }
    if IVE_TRIED.search(&lowered).is_some() {
        if let Some(before) = _the_body_before(memory, &lowered) {
            let lead = choose(&IVE_TRIED_LEAD, memory, &lowered);
            return Some(format!("{lead} {}", _the_shape(&before, memory, &lowered)));
        }
    }
    if let Some(person_wait) = _the_wait_for_a_person(memory, &lowered) {
        return Some(person_wait);
    }
    if RAISED.search(&lowered).is_some() && NOT_DONE.search(&lowered).is_some() && !_names_its_own_person(&lowered) {
        return Some(_the_shape_unnamed(memory, &lowered));
    }
    None
}

static SENTENCE_START: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?] )([a-z])", 0));

/// `said[0].upper() + said[1:]`, then every sentence's first letter raised.
fn sentences_raised(said: &str) -> String {
    let mut c = said.chars();
    let first = c.next().unwrap_or_else(|| panic!("IndexError: string index out of range"));
    let said = first.to_uppercase().collect::<String>() + c.as_str();
    SENTENCE_START.sub_fn(&said, |hit| hit.str(1).to_uppercase())
}

/// Python: bot/modules/chasing_a_body.py:572 `_the_timescale(body, memory, lowered)`
pub fn _the_timescale(body: &(String, String), memory: &Memory, lowered: &str) -> String {
    let said = choose(&THE_TIMESCALE, memory, lowered).replace("{body}", &body.0);
    sentences_raised(&said)
}

/// Python: bot/modules/chasing_a_body.py:581 `_the_next_step(body, memory, lowered)`
pub fn _the_next_step(body: &(String, String), memory: &Memory, lowered: &str) -> String {
    let said = choose(&THE_NEXT_STEP, memory, lowered).replace("{reference}", &body.1).replace("{body}", &body.0);
    sentences_raised(&said)
}

/// Python: bot/modules/chasing_a_body.py:591 `_the_next_step_unnamed(memory, lowered)`
pub fn _the_next_step_unnamed(memory: &Memory, lowered: &str) -> String {
    sentences_raised(&choose(&THE_NEXT_STEP_UNNAMED, memory, lowered))
}

/// Python: bot/modules/chasing_a_body.py:599 `_the_shape(body, memory, lowered)`
pub fn _the_shape(body: &(String, String), memory: &Memory, lowered: &str) -> String {
    let shape = choose(&THE_SHAPE, memory, lowered);
    let said = shape.replace("{reference}", &body.1).replace("{body}", &body.0);
    sentences_raised(&said)
}
