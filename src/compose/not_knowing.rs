//! Port of `bot/compose/not_knowing.py`: when the bot cannot answer, what can still be said
//! about that kind of question and where the rest lives, instead of a bare hedge.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_sets, bundle_strs, inline_re};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;

bundle_res!("bot.compose.not_knowing"; LIVE_TODAY, LIVE_ASK, A_MACHINE_OF_THEIRS, GET_READY_FOR, RULES_FOR_THEIR_CASE,
    HOW_MUCH_TO_PUT_IN, A_PRICE_THEY_WILL_PAY, WHAT_TO_TAKE, HOW_TO_TEACH, LEAVING_SOMEONE, IS_IT_OK, A_BODY_IN_IT,
    A_FOOD_IN_IT, WATCH_OUT, A_USUAL_AMOUNT, NEWS_RESULT, SOMEONE_ELSES_OWN, DOCUMENT_NAMED, POINTER_HEDGE, ASKS_TO_DECIDE,
    WHERE_TO_START_ASKS, WHAT_YOU_NEED_ASKS, ROUTE_IN_ASKS, FIRST_ATTEMPT_ASKS, GET_READY_ASKS, NAMES_WHERE_TO_ASK,
    ABOUT_THIS_CHAT);
bundle_strs!("bot.compose.not_knowing"; GET_READY_LEADS, GET_READY_ROUTES, PUT_IN_LEADS, PUT_IN_ROUTES, WHAT_TO_TAKE_LEADS,
    WHAT_TO_TAKE_ROUTES, HOW_TO_TEACH_LEADS, HOW_TO_TEACH_ROUTES, LEAVING_SOMEONE_LEADS, LEAVING_SOMEONE_ROUTES,
    IS_IT_OK_LEADS, IS_IT_OK_PLAIN_LEADS, IS_IT_OK_ROUTES, IS_IT_OK_PLAIN_ROUTES, IS_IT_OK_FOOD_ROUTES, LIVE_TODAY_LEADS,
    LIVE_TODAY_ROUTES, A_MACHINE_LEADS, A_MACHINE_ROUTES, RULES_LEADS, RULES_ROUTES, RULES_PLAIN_ROUTES, A_PRICE_LEADS,
    A_PRICE_ROUTES, SOMEONE_ELSES_LEADS, SOMEONE_ELSES_ROUTES, USUAL_AMOUNT_LEADS, WATCH_OUT_LEADS, WATCH_OUT_BODY_LEADS,
    WATCH_OUT_BODY_ROUTES, WATCH_OUT_ROUTES, USUAL_AMOUNT_ROUTES, NEWS_LEADS, NEWS_ROUTES, WHERE_TO_START_LINES,
    WHAT_YOU_NEED_LINES, FIRST_ATTEMPT_LINES, ROUTE_IN_LINES, GET_READY_LINES);
bundle_sets!("bot.compose.not_knowing"; ASK_ONLY_WORDS);
inline_re!(A_WORD, r"[a-z']+");

/// Python: bot/compose/not_knowing.py:27 `_choose(options, memory, clean)`
///
/// The pool's own chooser (`fallback_chain._choose`). Python concatenates what it returns, so a
/// None from it raises there; here it panics the same way.
pub fn _choose(options: &[String], memory: &Memory, clean: &str) -> String {
    crate::compose::fallback_chain::_choose(options, memory, clean, None)
        .unwrap_or_else(|| panic!("TypeError: the chooser gave None"))
}

/// Python: bot/compose/not_knowing.py:398 `_the_words_of(said)`
///
/// The words of a line, lowercased, with the punctuation and the apostrophes dropped.
pub fn _the_words_of(said: &str) -> Vec<String> {
    A_WORD.findall_str(&said.to_lowercase()).into_iter().map(|w| w.replace('\'', "")).collect()
}

/// Python: bot/compose/not_knowing.py:408 `_says_nothing_of_its_own(clean)`
///
/// True when every word of the line is an ask or a filler: a follow-up with no subject.
pub fn _says_nothing_of_its_own(clean: &str) -> bool {
    let words = _the_words_of(clean);
    !words.is_empty() && words.iter().all(|w| ASK_ONLY_WORDS.contains(w))
}

/// Python: bot/compose/not_knowing.py:484 `_an_ask_with_no_subject(clean, memory=None)`
///
/// The usual first moves for the kind of help asked for, or "" when the ask is not one.
pub fn _an_ask_with_no_subject(clean: &str, memory: &Memory) -> String {
    let line = text::squash(clean);
    if line.is_empty() || !_says_nothing_of_its_own(&line) {
        return String::new();
    }
    let pool: &Vec<String> = if WHERE_TO_START_ASKS.search(&line).is_some() {
        &WHERE_TO_START_LINES
    } else if WHAT_YOU_NEED_ASKS.search(&line).is_some() {
        &WHAT_YOU_NEED_LINES
    } else if ROUTE_IN_ASKS.search(&line).is_some() {
        &ROUTE_IN_LINES
    } else if GET_READY_ASKS.search(&line).is_some() {
        &GET_READY_LINES
    } else {
        return String::new();
    };
    _choose(pool, memory, clean)
}

/// Python: bot/compose/not_knowing.py:516 `_the_whole_exchange(line, memory)`
///
/// The line and what was said before it, so a follow-up keeps the subject of the turn before.
pub fn _the_whole_exchange(line: &str, memory: &Memory) -> String {
    let turns = &memory.turns;
    let mut said = vec![line.to_string()];
    said.extend(turns[turns.len().saturating_sub(3)..].iter().map(|t| t.user.clone()));
    said.join(" ")
}

/// Python: bot/compose/not_knowing.py:529 `_the_routes_hold_this(line)`
///
/// True when `routes.py` names a page or a rule for this line, so this file must stand down.
pub fn _the_routes_hold_this(line: &str) -> bool {
    crate::compose::com4::attempt(|| crate::compose::routes::options(line).is_some_and(|o| !o.is_empty())).unwrap_or(false)
}

/// Python: bot/compose/not_knowing.py:541 `_the_ok_lead(line)`
///
/// The lead for a yes-or-no ask: about food and bodies only where one of them is in the line.
pub fn _the_ok_lead(line: &str) -> Vec<String> {
    // `route is IS_IT_OK_PLAIN_ROUTES`: the plain route is the one with neither a body nor food.
    if A_BODY_IN_IT.search(line).is_none() && A_FOOD_IN_IT.search(line).is_none() {
        IS_IT_OK_PLAIN_LEADS.clone()
    } else {
        IS_IT_OK_LEADS.clone()
    }
}

/// Python: bot/compose/not_knowing.py:547 `_the_ok_route(line)`
///
/// The route for a yes-or-no ask: a body first, then the kitchen, then the general one.
pub fn _the_ok_route(line: &str) -> Vec<String> {
    if A_BODY_IN_IT.search(line).is_some() {
        return IS_IT_OK_ROUTES.clone();
    }
    if A_FOOD_IN_IT.search(line).is_some() {
        return IS_IT_OK_FOOD_ROUTES.clone();
    }
    IS_IT_OK_PLAIN_ROUTES.clone()
}

/// Python: bot/compose/not_knowing.py:557 `_the_rules_route(line)`
///
/// The route for a rule: the document one only where the line names a document.
pub fn _the_rules_route(line: &str) -> Vec<String> {
    if DOCUMENT_NAMED.search(line).is_some() { RULES_ROUTES.clone() } else { RULES_PLAIN_ROUTES.clone() }
}

/// Python: bot/compose/not_knowing.py:562 `_the_lead_and_the_route(clean, memory=None)`
///
/// What can be said about this kind of question, and where the rest lives, or "".
pub fn _the_lead_and_the_route(clean: &str, memory: &Memory) -> String {
    let line = text::squash(clean);
    if line.is_empty() || ASKS_TO_DECIDE.search(&line).is_some() {
        return String::new();
    }
    // Whatever the routes hold is left to them.
    if _the_routes_hold_this(&line) {
        return String::new();
    }
    let has = |rx: &Re| rx.search(&line).is_some();
    let both = |lead: &[String], route: &[String]| format!("{} {}", _choose(lead, memory, clean), _choose(route, memory, clean));
    if has(&NEWS_RESULT) {
        return both(&NEWS_LEADS, &NEWS_ROUTES);
    }
    if has(&WHAT_TO_TAKE) {
        return both(&WHAT_TO_TAKE_LEADS, &WHAT_TO_TAKE_ROUTES);
    }
    if has(&HOW_TO_TEACH) {
        return both(&HOW_TO_TEACH_LEADS, &HOW_TO_TEACH_ROUTES);
    }
    if has(&LEAVING_SOMEONE) {
        return both(&LEAVING_SOMEONE_LEADS, &LEAVING_SOMEONE_ROUTES);
    }
    if has(&IS_IT_OK) {
        return both(&_the_ok_lead(&line), &_the_ok_route(&line));
    }
    if has(&FIRST_ATTEMPT_ASKS) {
        return _choose(&FIRST_ATTEMPT_LINES, memory, clean);
    }
    if has(&WATCH_OUT) {
        if A_BODY_IN_IT.search(&_the_whole_exchange(&line, memory)).is_some() {
            return both(&WATCH_OUT_BODY_LEADS, &WATCH_OUT_BODY_ROUTES);
        }
        return both(&WATCH_OUT_LEADS, &WATCH_OUT_ROUTES);
    }
    if has(&A_USUAL_AMOUNT) {
        return both(&USUAL_AMOUNT_LEADS, &USUAL_AMOUNT_ROUTES);
    }
    if has(&SOMEONE_ELSES_OWN) {
        return both(&SOMEONE_ELSES_LEADS, &SOMEONE_ELSES_ROUTES);
    }
    // The kind that is actually being asked about wins, and a machine is the last of them.
    if has(&LIVE_TODAY) && has(&LIVE_ASK) {
        return both(&LIVE_TODAY_LEADS, &LIVE_TODAY_ROUTES);
    }
    if has(&HOW_MUCH_TO_PUT_IN) {
        return both(&PUT_IN_LEADS, &PUT_IN_ROUTES);
    }
    if has(&A_PRICE_THEY_WILL_PAY) {
        return both(&A_PRICE_LEADS, &A_PRICE_ROUTES);
    }
    if has(&GET_READY_FOR) {
        return both(&GET_READY_LEADS, &GET_READY_ROUTES);
    }
    if has(&RULES_FOR_THEIR_CASE) {
        return both(&RULES_LEADS, &_the_rules_route(&line));
    }
    if has(&A_MACHINE_OF_THEIRS) {
        return both(&A_MACHINE_LEADS, &A_MACHINE_ROUTES);
    }
    // No kind recognised: the pool's own line stands.
    String::new()
}

/// Python: bot/compose/not_knowing.py:654 `_asks_about_this_chat(clean)`
///
/// True when the line asks how long this chat has gone on, or whether there is a limit.
pub fn _asks_about_this_chat(clean: &str) -> bool {
    let line = text::squash(clean);
    !line.is_empty() && ABOUT_THIS_CHAT.match_(&line).is_some()
}
