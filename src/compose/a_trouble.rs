//! Port of `bot/compose/a_trouble.py`: a thing not doing its job ("my airpods keep cutting
//! out"), asked about with the one question that narrows it.
#![allow(non_upper_case_globals)]

use crate::compose::aboutness::{is_a_hedge as _a_hedge, restates_the_line, unlike_the_line};
use crate::compose::com4::{self, bundle_res, bundle_sets, bundle_strs, bundle_vals};
use crate::compose::fallback;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;

bundle_res!("bot.compose.a_trouble"; A_TROUBLE, NOT_A_MACHINE, FIRE_WORD, ITS_THE_WEATHER, _ALREADY_NARROWING,
    _A_BARE_OFFER, STALL);
bundle_sets!("bot.compose.a_trouble"; _NOT_THE_THING, _FUNCTION, _STOP);
bundle_strs!("bot.compose.a_trouble"; _REPLIES, _DETERMINERS);
bundle_vals!("bot.compose.a_trouble"; _STOCK_LENGTH);

/// Python: bot/compose/a_trouble.py:118 `the_thing(clean)`
///
/// The most specific thing the line is about, with its determiner, or "" when there is none.
pub fn the_thing(clean: &str) -> String {
    let words = text::split(clean);
    // The words of the trouble itself are what is wrong, not what it is wrong with: "the start"
    // out of "my car wont start" is not a thing.
    let spans: Vec<(usize, usize)> = A_TROUBLE.finditer(clean).iter().map(|m| (m.start(), m.end())).collect();
    let mut trouble_words: HashSet<&str> = HashSet::new();
    for &(start, end) in &spans {
        trouble_words.extend(text::split_ref(&clean[start..end]));
    }
    let found: Vec<String> = words
        .iter()
        .filter(|w| {
            text::len(w) >= 3
                && !_FUNCTION.contains(*w)
                && !_NOT_THE_THING.contains(*w)
                && !trouble_words.contains(w.as_str())
                && text::is_alpha(w)
        })
        .cloned()
        .collect();
    if found.is_empty() {
        return String::new();
    }
    _named(clean, &words, &found, &spans)
}

/// Python: bot/compose/a_trouble.py:137 `_named(clean, words, found, spans)`
///
/// The first thing named is the thing: "my car wont start in the cold" is about the car, not
/// about the cold, and "my washing machine" is two words of one thing.
pub fn _named(clean: &str, words: &[String], found: &[String], spans: &[(usize, usize)]) -> String {
    let _ = (clean, spans);
    let best = &found[0];
    let at = words.iter().position(|w| w == best).expect("list.index: the word is in the line");
    let mut name = best.clone();
    let after = words.get(at + 1).map(String::as_str).unwrap_or("");
    if !after.is_empty() && text::is_alpha(after) && text::len(after) >= 4 && found.iter().any(|f| f == after) && !_STOP.contains(after) {
        name = format!("{best} {after}");
    }
    let before = if at > 0 { words[at - 1].as_str() } else { "" };
    if _DETERMINERS.iter().any(|d| d == before) {
        return format!("{before} {name}");
    }
    format!("the {name}")
}

/// Python: bot/compose/a_trouble.py:153 `_heavy(memory)`
///
/// True when the thread is heavy news, read through the composer's own test for it.
pub fn _heavy(memory: &Memory) -> bool {
    let turns = &memory.turns;
    let Some(last) = turns.last() else { return false };
    if crate::compose::badnews::reads_as_heavy_news(&text::squash(&last.user)) {
        return true;
    }
    let only_turns = Memory { turns: turns.clone(), ..Memory::new() };
    fallback::_the_thread_is_heavy(&only_turns)
}

/// Python: bot/compose/a_trouble.py:163 `trouble_pool(clean, memory)`
///
/// The one question this trouble raises, or None when this is not a trouble.
pub fn trouble_pool(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let clean = com4::flat(clean);
    if _heavy(memory) {
        return None; // the care readers own a heavy thread, whatever the line says
    }
    if crate::compose::badnews::reads_as_heavy_news(&clean) {
        return None; // and they own a heavy line on its own, before there is any thread
    }
    if NOT_A_MACHINE.search(&clean).is_some() || FIRE_WORD.search(&clean).is_some() {
        return None; // a person or a hazard, not a thing that is not doing its job
    }
    if ITS_THE_WEATHER.search(&clean).is_some() {
        return None; // the weather is no fault of anything
    }
    if clean.is_empty() || clean.contains('?') || text::split_ref(&clean).len() < 3 {
        return None; // a question has its own answers; a bare word names nothing to ask about
    }
    if A_TROUBLE.search(&clean).is_none() {
        return None;
    }
    let thing = the_thing(&clean);
    if thing.is_empty() {
        return None;
    }
    let pool: Vec<String> = _REPLIES.iter().map(|line| text::upper_first(&line.replacen("%s", &thing, 1))).collect();
    if pool.iter().all(|line| {
        let n = text::split_ref(line).len();
        2 < n && n < 22
    }) {
        Some(pool)
    } else {
        None
    }
}

/// Python: bot/compose/a_trouble.py:207 `_a_bare_offer(reply)`
pub fn _a_bare_offer(reply: &str) -> bool {
    _A_BARE_OFFER.search(reply).is_some()
}

/// Python: bot/compose/a_trouble.py:220 `says_nothing_about_the_line(reply, text)`
///
/// True when the composer's own reply is a hedge, the line given back, or a stock line.
pub fn says_nothing_about_the_line(reply: &str, text_: &str) -> bool {
    let reply = com4::straight(reply);
    if text::strip(&reply).is_empty() {
        return true;
    }
    if STALL.search(&reply).is_some() {
        return true; // a stall is a stall however long it is
    }
    if text::split_ref(&reply).len() as i64 > _STOCK_LENGTH.as_int().expect("_STOCK_LENGTH is an int") {
        return false; // a real answer, however it is worded; only a short line is a placeholder
    }
    if text::rstrip(&reply).ends_with('?') && !_a_bare_offer(&reply) {
        return false; // a question of its own already moves the thread along
    }
    if _ALREADY_NARROWING.search(&reply).is_some() {
        return false; // a question about the trouble is already an answer to it
    }
    if _a_hedge(&reply) || restates_the_line(&reply, text_) {
        return true;
    }
    let pair = unlike_the_line(&reply, text_);
    let (about, said) = (&pair.items()[0], &pair.items()[1]);
    about.truthy() && !said.truthy()
}
