//! Port of `bot/compose/colour_on_the_body.py`: a colour on the person's own skin ("my neck is
//! all blotchy") or a blush, answered as what it is rather than a card the words matched.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_sets, bundle_strs};
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;

bundle_res!("bot.compose.colour_on_the_body"; COLOUR, BLUSH, WORD);
bundle_sets!("bot.compose.colour_on_the_body"; BODY_PARTS, ALSO_A_SYMPTOM, A_BURN, INSIDE, SOMEWHERE_ELSE, SOMEONE_ELSE,
    PETS, NOT_A_PATCH);
bundle_strs!("bot.compose.colour_on_the_body"; PATCH_POOL, BLUSH_POOL);

fn meets(words: &HashSet<String>, set: &HashSet<String>) -> bool {
    words.iter().any(|w| set.contains(w))
}

/// Python: bot/compose/colour_on_the_body.py:165 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    WORD.findall_str(&com4::straight(&text.to_lowercase()))
}

/// Python: bot/compose/colour_on_the_body.py:169 `_last_reply(memory)`
///
/// What the bot said on the turn before, if the store has it.
pub fn _last_reply(memory: &Memory) -> String {
    memory.turns.iter().rev().find(|t| !t.bot.is_empty()).map(|t| text::squash(&t.bot)).unwrap_or_default()
}

/// Python: bot/compose/colour_on_the_body.py:178 `_pick(pool, line, memory=None)`
///
/// A wording for this line, never the whole of what the bot said last time: the wording steps
/// on when it would be the last one said again.
pub fn _pick(pool: &[String], line: &str, memory: &Memory) -> String {
    if pool.is_empty() {
        return String::new();
    }
    let start = (com4::ord_sum(&line.to_lowercase()) % pool.len() as i128) as usize;
    let last = _last_reply(memory);
    for step in 0..pool.len() {
        let said = &pool[(start + step) % pool.len()];
        if last.is_empty() || !last.contains(&text::squash(said)) {
            return said.clone();
        }
    }
    pool[start].clone()
}

/// Python: bot/compose/colour_on_the_body.py:198 `_someone_elses(words)`
///
/// True when the line says whose body it is and it is not the person typing. The plural is
/// read by dropping one trailing s, and only when what is left is a person word.
pub fn _someone_elses<S: AsRef<str>>(words: &[S]) -> bool {
    let either = |w: &str| SOMEONE_ELSE.contains(w) || PETS.contains(w);
    for word in words {
        let word = word.as_ref();
        if either(word) {
            return true;
        }
        for suffix in ["'s", "'"] {
            if let Some(stem) = word.strip_suffix(suffix) {
                if either(stem) {
                    return true;
                }
            }
        }
        if let Some(stem) = word.strip_suffix('s') {
            if either(stem) {
                return true;
            }
        }
    }
    false
}

/// Python: bot/compose/colour_on_the_body.py:217 `_thread_names_a_pet(memory)`
///
/// True when the last few turns were about an animal, whose skin is not the person's.
pub fn _thread_names_a_pet(memory: &Memory) -> bool {
    let mut seen = 0;
    for turn in memory.turns.iter().rev() {
        let said = text::strip(&turn.user).to_lowercase();
        if said.is_empty() {
            continue;
        }
        seen += 1;
        if seen > 4 {
            break;
        }
        if _words(&said).iter().any(|w| PETS.contains(w)) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/colour_on_the_body.py:235 `reply_for(text, memory=None)`
///
/// The answer to a colour on the person's own body or a blush, or None.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&com4::straight(&text.to_lowercase()));
    if clean.is_empty() {
        return None;
    }
    let words: HashSet<String> = _words(&clean).into_iter().collect();
    let word_list: Vec<&String> = words.iter().collect();

    // A blush is a moment, not a patch, and it is the one shape that needs no body part in the
    // line: "i go bright red when i talk to people".
    if BLUSH.search(&clean).is_some() {
        // "why do we blush" is a question about people in general.
        if clean.starts_with("why") && !["i", "my", "me", "im", "myself"].iter().any(|w| words.contains(*w)) {
            return None;
        }
        if meets(&words, &SOMEWHERE_ELSE) || meets(&words, &INSIDE) {
            return None;
        }
        if _someone_elses(&word_list) {
            return None;
        }
        return Some(_pick(&BLUSH_POOL, &clean, memory));
    }

    if COLOUR.search(&clean).is_none() {
        return None;
    }
    if !meets(&words, &BODY_PARTS) || meets(&words, &INSIDE) || meets(&words, &SOMEWHERE_ELSE) {
        return None;
    }
    if _someone_elses(&word_list) {
        return None;
    }
    if meets(&words, &ALSO_A_SYMPTOM) || meets(&words, &NOT_A_PATCH) || meets(&words, &A_BURN) {
        return None;
    }
    if _thread_names_a_pet(memory) {
        return None;
    }
    Some(_pick(&PATCH_POOL, &clean, memory))
}
