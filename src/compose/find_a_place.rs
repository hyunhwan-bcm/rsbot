//! Port of `bot/compose/find_a_place.py`: a where-to-get-it ask ("is there a launderette near
//! me") answered with the search to type, since the bot has no map.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_sets, bundle_strs, format_named, inline_re};
use crate::compose::pick;
use crate::py::text;
use crate::state::Memory;

bundle_res!("bot.compose.find_a_place"; WHERE_TO_GET, THERE_IS, NEARBY, A_RECOMMEND, A_ASKING_MY_OPINION,
    A_NAME_FOR_SOMEBODY, HOURS, SOMEBODY, A_TRADE, A_JOB_ON_A_THING, A_LEAD, A_TAIL, NOT_A_PLACE, BARE_ANY,
    ANY_ABSTRACT, CASH_MACHINE, ALREADY_A_PLAN, ALREADY_A_ROUTE, A_HAD_IT, A_THING_TO_HELP_WITH);
bundle_sets!("bot.compose.find_a_place"; NO_THING, _ROUTE_HEADS);
bundle_strs!("bot.compose.find_a_place"; NO_MAP, NO_MAP_UNNAMED, A_HAND, THE_HOURS);
inline_re!(DOES_THE_THING, r"\b(?:that|which|who) (?:does|sells|fixes|repairs|makes|keeps|stocks)\s+([a-z][a-z' ]{1,28})$");
inline_re!(GET_THE_THING, concat!(
    r"\b(?:get|buy|find|hire|pick up|source)\s+(?:hold of\s+)?(?:a|an|the|some|any|my|our)?\s*",
    r"([a-z][a-z' ]{1,28}?)(?:\s+(?:from|near|round|around|close|locally|in|please|then|",
    r"today|now|cheaply|cheap|asap)$|\s*$)"));
inline_re!(ANY_THING, concat!(r"(?:any|some)\s+([a-z][a-z' ]{1,28}?)(?:\s+(?:near|round|around|open|",
    r"in|that|to|please|today|now|locally)\b|$)"));
inline_re!(NEAREST_THING, concat!(r"\b(?:nearest|closest)\s+([a-z][a-z' ]{1,28}?)(?:\s+(?:near|to|for|in|",
    r"that|please|today|now|open)\b|$)"));
inline_re!(IS_THERE_A_THING, concat!(r"\b(?:is|are) there (?:a|an|any|somewhere|anywhere|any good|a decent)",
    r"\s+([a-z][a-z' ]{1,28}?)(?:\s+(?:near|round|around|close|open|",
    r"locally|in|that|to)\b|$)"));
inline_re!(MY_OUR_THE, r"\b(?:my|our|the)\b");
inline_re!(WHO_DOES, r"\bwho (?:does|sells|fixes|repairs|makes)\b");
inline_re!(A_GET_VERB, r"\b(?:get|buy|find|hire|pick up|source)\b");

/// Python: bot/compose/find_a_place.py:196 `already_a_route(reply)`
///
/// True when a reply already names the finder, the register or the body that keeps it.
pub fn already_a_route(reply: &str) -> bool {
    if ALREADY_A_ROUTE.search(reply).is_some() {
        return true;
    }
    _ROUTE_HEADS.iter().any(|head| !head.is_empty() && reply.starts_with(head.as_str()))
}

/// Python: bot/compose/find_a_place.py:208 `_clean(text)`
pub fn _clean(text: &str) -> String {
    text::strip_chars(&text::squash(&text.to_lowercase().replace('\u{2019}', "'")), " ?.!,").to_string()
}

/// Python: bot/compose/find_a_place.py:212 `_a_thing(thing)`
///
/// The thing trimmed of the ask and the place around it, or None when it names nothing.
pub fn _a_thing(thing: &str) -> Option<String> {
    let mut thing = text::strip(thing).to_string();
    loop {
        let trimmed = text::strip(&A_TAIL.sub("", &thing)).to_string();
        if trimmed == thing {
            break;
        }
        thing = trimmed;
    }
    if thing.is_empty() || NO_THING.contains(&thing) || text::split_ref(&thing).len() > 4 {
        return None;
    }
    Some(thing)
}

/// `_a_thing(A_LEAD.sub("", found.group(1).strip()))`.
fn led(found: &str) -> Option<String> {
    _a_thing(&A_LEAD.sub("", text::strip(found)))
}

/// Python: bot/compose/find_a_place.py:225 `_the_thing(clean)`
///
/// The thing the line asks for, in the person's own words, or None when it names none.
pub fn _the_thing(clean: &str) -> Option<String> {
    // "anywhere round here that does dry cleaning": the thing is what the place does.
    if let Some(found) = DOES_THE_THING.search(clean) {
        return _a_thing(found.str(1));
    }
    if let Some(found) = GET_THE_THING.search(clean) {
        return led(found.str(1));
    }
    if let Some(found) = ANY_THING.match_(clean) {
        return led(found.str(1));
    }
    // "where is the nearest atm": the thing is what the near-word is about.
    if let Some(found) = NEAREST_THING.search(clean) {
        if let Some(thing) = led(found.str(1)) {
            if !matches!(thing.as_str(), "place" | "one" | "spot") {
                return Some(thing);
            }
        }
    }
    if let Some(found) = IS_THERE_A_THING.search(clean) {
        return led(found.str(1));
    }
    None
}

/// Python: bot/compose/find_a_place.py:298 `_a_person_for_the_job(clean)`
///
/// True when the person is asking for somebody to do the work rather than a shop.
pub fn _a_person_for_the_job(clean: &str) -> bool {
    if SOMEBODY.search(clean).is_some() || A_TRADE.search(clean).is_some() {
        return true;
    }
    A_JOB_ON_A_THING.search(clean).is_some() && MY_OUR_THE.search(clean).is_some()
}

/// Python: bot/compose/find_a_place.py:305 `answer_for(text, memory)`
///
/// A reply for a where-to-get-it ask, or None.
pub fn answer_for(text_: &str, memory: &Memory) -> Option<String> {
    let clean = _clean(text_);
    if clean.is_empty() {
        return None;
    }
    let n = text::split_ref(&clean).len();
    if !(3..=16).contains(&n) {
        return None;
    }
    let has = |rx: &crate::py::re::Re| rx.search(&clean).is_some();
    let bare_any = BARE_ANY.match_(&clean).is_some();
    let for_the_place_test = CASH_MACHINE.sub("cash_machine", &clean);
    if bare_any && !has(&THERE_IS) && ANY_ABSTRACT.search(&for_the_place_test).is_some() {
        return None;
    }
    if NOT_A_PLACE.search(&for_the_place_test).is_some() || has(&ALREADY_A_PLAN) {
        return None;
    }
    let asking_where = has(&WHERE_TO_GET);
    let mut asking_if_there_is = (has(&THERE_IS) || bare_any) && has(&NEARBY);
    if bare_any && !asking_if_there_is {
        // "any chemist open now" names no place word but is the same question.
        asking_if_there_is = has(&HOURS);
    }
    let asking_for_a_name = has(&A_RECOMMEND)
        && !has(&A_ASKING_MY_OPINION)
        && !has(&A_HAD_IT)
        && (has(&A_NAME_FOR_SOMEBODY)
            || has(&A_TRADE)
            || has(&A_THING_TO_HELP_WITH)
            || (has(&SOMEBODY) && has(&A_JOB_ON_A_THING)));
    if !asking_where && !asking_if_there_is && !asking_for_a_name {
        return None;
    }
    if asking_for_a_name {
        return Some(pick(&A_HAND, memory, &clean));
    }
    let who_does = has(&WHO_DOES) && has(&NEARBY);
    if asking_where && !who_does && !has(&A_GET_VERB) && !has(&NEARBY) {
        return None;
    }
    if has(&HOURS) {
        return Some(pick(&THE_HOURS, memory, &clean));
    }
    if _a_person_for_the_job(&clean) {
        return Some(pick(&A_HAND, memory, &clean));
    }
    if let Some(thing) = _the_thing(&clean) {
        return Some(format_named(&pick(&NO_MAP, memory, &clean), &[("thing", &thing)]));
    }
    Some(pick(&NO_MAP_UNNAMED, memory, &clean))
}
