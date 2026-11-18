//! Port of `bot/compose/nearby_one.py`: "is there one near me" after the person named a place
//! they were looking for, answered with the search a map app can do.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_sets, bundle_strs, format_named, inline_re};
use crate::py::text;
use crate::state::Memory;

bundle_res!("bot.compose.nearby_one"; THE_PRONOUN_ASK, THE_FRAME);
bundle_sets!("bot.compose.nearby_one"; THE_END, THE_ARTICLE, THE_ADJECTIVE, A_VERB, NOT_A_THING, NOT_A_PLACE, PLACE_HEADS,
    A_CHORE, A_TRADE, CASH_POINT, NEAR_ME_PAIR);
bundle_strs!("bot.compose.nearby_one"; NEARBY_ONE_POOL, PLACE_TAILS, PLACE_STEMS);
inline_re!(A_PLAIN_WORD, r"^[a-z][a-z'&-]*$");

/// Python: bot/compose/nearby_one.py:172 `_the_thing(line)`
///
/// The thing the person said they were looking for, in their own words, or "".
pub fn _the_thing(line: &str) -> String {
    let Some(found) = THE_FRAME.search(line) else { return String::new() };
    let words = text::split_ref(&line[found.end()..]);
    const PUNCT: &str = ".,!?;:'\"()";
    if words.first().is_some_and(|w| text::strip_chars(w, PUNCT).to_lowercase() == "to") {
        return String::new();
    }
    let mut kept: Vec<String> = Vec::new();
    for word in words {
        let bare = text::strip_chars(word, PUNCT).to_lowercase();
        if bare.is_empty() {
            continue;
        }
        if THE_END.contains(&bare) || THE_ARTICLE.contains(&bare) {
            if !kept.is_empty() {
                break;
            }
            continue;
        }
        if THE_ADJECTIVE.contains(&bare) && kept.is_empty() {
            continue;
        }
        if A_PLAIN_WORD.match_(&bare).is_none() {
            break;
        }
        kept.push(bare);
        if kept.len() == 3 {
            break;
        }
    }
    if kept.is_empty() {
        return String::new();
    }
    let joined = kept.join(" ");
    if NOT_A_THING.contains(&kept[0]) || NOT_A_THING.contains(&joined) {
        return String::new();
    }
    // The words after the frame are a verb, not a thing.
    if A_VERB.contains(&kept[0]) {
        return String::new();
    }
    if CASH_POINT.contains(&joined) || NEAR_ME_PAIR.contains(&joined) {
        return joined;
    }
    if kept.iter().any(|w| NOT_A_PLACE.contains(w) || NOT_A_THING.contains(w)) {
        return String::new();
    }
    if !_a_place(&kept) {
        return String::new();
    }
    joined
}

/// Python: bot/compose/nearby_one.py:215 `_a_place(kept)`
///
/// True when the thing the person named is somewhere a map app could show them.
pub fn _a_place<S: AsRef<str>>(kept: &[S]) -> bool {
    let last = kept[kept.len() - 1].as_ref();
    // "i need a big shop before the weekend" is a job in the diary, not an address.
    if kept.len() == 1 && A_CHORE.contains(last) {
        return false;
    }
    // The last word is the thing.
    if PLACE_HEADS.contains(last) || A_TRADE.contains(last) {
        return true;
    }
    text::ends_with_any(last, &PLACE_STEMS) || text::ends_with_any(last, &PLACE_TAILS)
}

/// Python: bot/compose/nearby_one.py:230 `_the_thread_thing(memory, cleaned)`
///
/// The thing the person said they were looking for, newest first, or "" if that was not a place.
pub fn _the_thread_thing(memory: &Memory, cleaned: &str) -> String {
    let mut seen = 0;
    for turn in memory.turns.iter().rev() {
        let said = text::strip(&turn.user).to_lowercase();
        if said.is_empty() || said == cleaned || THE_PRONOUN_ASK.search(&said).is_some() {
            continue;
        }
        seen += 1;
        if seen > 4 {
            break;
        }
        if THE_FRAME.search(&said).is_none() {
            continue;
        }
        return _the_thing(&said);
    }
    String::new()
}

/// Python: bot/compose/nearby_one.py:254 `_pick(pool, line)`
///
/// One of two wordings, the same one for the same line.
pub fn _pick(pool: &[String], line: &str) -> String {
    if pool.is_empty() {
        return String::new();
    }
    com4::by_ord(pool, line).to_string()
}

/// Python: bot/compose/nearby_one.py:261 `reply_for(line, memory=None)`
///
/// The map-app answer to a pronoun near-me ask, or None.
pub fn reply_for(line: &str, memory: &Memory) -> Option<String> {
    if line.is_empty() {
        return None;
    }
    let cleaned = text::squash(&line.to_lowercase());
    if THE_PRONOUN_ASK.search(&cleaned).is_none() {
        return None;
    }
    let thing = _the_thread_thing(memory, &cleaned);
    if thing.is_empty() {
        // Nothing in the thread this can put into a map app.
        return None;
    }
    Some(format_named(&_pick(&NEARBY_ONE_POOL, &cleaned), &[("thing", &thing)]))
}
