//! Port of `bot/compose/is_it_worth_it.py`: "is it worth it" where the composer hedged,
//! answered with the number to weigh and where it lives (a part off a shelf, a trade's quote, a
//! used price, a ticket).
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_sets, bundle_strs};
use crate::compose::{fallback, pick};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;

bundle_res!("bot.compose.is_it_worth_it"; FIX_OR_REPLACE, A_TRADE, A_SUGGESTIONS, A_VISIT, PLACE_WORD, A_PURCHASE,
    A_SMALL_THING, A_NAMED_THING, A_REPLACE_WORD, A_APPLIANCE, A_MONEY, A_RESALE_THING, A_MINE, A_USED_MARK, A_SALE_ASK,
    A_NEW_PRICE, AN_APPEAL, THEIR_OWN_LIFE, A_LIVING_THING, STAND_DOWN, A_TRIP, A_GOING, A_TROUBLE_THREAD, A_BODY_THREAD,
    A_DEAL_THREAD, A_STUDY, A_PAID_PERSON, NOT_AN_ASK, A_REPORT, A_FAULT_ALREADY_SAID, LAST_ASK);
bundle_strs!("bot.compose.is_it_worth_it"; A_FIX_REPLY, A_SMALL_THING_REPLY, A_DAY_OUT_REPLY, A_TRADE_REPLY, A_VISIT_REPLY,
    A_PURCHASE_REPLY, A_VALUATION_REPLY, AN_APPEAL_REPLY);
bundle_sets!("bot.compose.is_it_worth_it"; GOING_ANYWAY);

/// The person's own lines of the last `back` turns, joined and lowered.
fn users(memory: &Memory, back: usize) -> String {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(back)..].iter().map(|t| t.user.as_str()).collect::<Vec<_>>().join(" ").to_lowercase()
}

/// Python: bot/compose/is_it_worth_it.py:285 `_a_destination(said)`
///
/// A place they say they are going to, and not one of the errands.
pub fn _a_destination(said: &str) -> String {
    A_GOING
        .findall_str(said)
        .into_iter()
        .find(|w| !GOING_ANYWAY.contains(w) && text::len(w) > 2)
        .unwrap_or_default()
}

/// Python: bot/compose/is_it_worth_it.py:293 `_a_trip_in_the_thread(memory)`
pub fn _a_trip_in_the_thread(memory: &Memory) -> bool {
    A_TRIP.search(&users(memory, 4)).is_some()
}

/// Python: bot/compose/is_it_worth_it.py:299 `_the_thread(memory, back=2)`
///
/// The last turn or two of the person's own lines - the subject this ask is about.
pub fn _the_thread(memory: &Memory, back: i64) -> String {
    let turns = &memory.turns;
    crate::compose::com4::tail(turns, back).iter().map(|t| t.user.as_str()).collect::<Vec<_>>().join(" ").to_lowercase()
}

/// Python: bot/compose/is_it_worth_it.py:310 `_is_it_worth_it(text, memory, reply)`
///
/// The number to weigh for a "worth it" the composer hedged on, or None.
pub fn _is_it_worth_it(text_: &str, memory: &Memory, reply: &str) -> Option<String> {
    if reply.is_empty() || !fallback::is_a_hedge(reply) {
        return None;
    }
    let said = text::squash(&text_.to_lowercase());
    let has = |rx: &Re| rx.search(&said).is_some();
    // A sale or an offer names a figure and asks whether it is right, without ever saying "worth".
    if said.is_empty() || !has(&A_SALE_ASK) || text::split_ref(&said).len() > 12 {
        return None;
    }
    if has(&STAND_DOWN) {
        return None;
    }
    let thread = _the_thread(memory, 2);
    let with_thread = format!("{said} {thread}");
    // A thing with a used market, and a figure against it, is a valuation before the life guard.
    if has(&A_MONEY)
        && !has(&A_NEW_PRICE)
        && A_STUDY.search(&with_thread).is_none()
        && (has(&A_RESALE_THING) || A_RESALE_THING.search(&thread).is_some())
    {
        return Some(pick(&A_VALUATION_REPLY, memory, &said));
    }
    if has(&THEIR_OWN_LIFE) {
        return None;
    }
    // A course, a degree, a qualification: a price against a life, not against a market.
    if A_STUDY.search(&with_thread).is_some() && !has(&A_RESALE_THING) {
        return None;
    }
    if has(&A_LIVING_THING) || has(&A_PAID_PERSON) || NOT_AN_ASK.match_(&said).is_some() || has(&A_REPORT) {
        return None;
    }
    let around = thread.clone();
    if A_TROUBLE_THREAD.search(&around).is_some() || A_BODY_THREAD.search(&around).is_some() || A_DEAL_THREAD.search(&around).is_some() {
        return None;
    }
    if fallback::in_a_safety_thread(memory) {
        return None;
    }
    // Money already named against a thing that has a used market is a valuation.
    if has(&A_MONEY) && !has(&A_NEW_PRICE) && A_STUDY.search(&with_thread).is_none() && (has(&A_MINE) || has(&A_USED_MARK)) {
        return Some(pick(&A_VALUATION_REPLY, memory, &said));
    }
    if has(&A_TRADE) {
        return Some(pick(&A_TRADE_REPLY, memory, &said));
    }
    // The thing named decides where the number lives; the thread is only read when the line
    // says "it".
    let reads_thread = !has(&A_NAMED_THING);
    let small = has(&A_SMALL_THING) || (reads_thread && A_SMALL_THING.search(&around).is_some());
    let appliance = has(&A_APPLIANCE) || (reads_thread && A_APPLIANCE.search(&around).is_some());
    if (small || appliance) && (has(&FIX_OR_REPLACE) || has(&A_PURCHASE) || has(&A_REPLACE_WORD)) {
        return Some(pick(if small { &A_SMALL_THING_REPLY } else { &A_FIX_REPLY }, memory, &said));
    }
    if has(&FIX_OR_REPLACE) {
        return Some(pick(&A_FIX_REPLY, memory, &said));
    }
    if has(&A_TRADE) {
        return Some(pick(&A_TRADE_REPLY, memory, &said));
    }
    if has(&AN_APPEAL) {
        return Some(pick(&AN_APPEAL_REPLY, memory, &said));
    }
    if has(&A_SUGGESTIONS) {
        return Some(pick(&A_DAY_OUT_REPLY, memory, &said));
    }
    // A place the line does not name but the thread does is a ticket, not a shelf price.
    let a_place = has(&A_VISIT)
        || has(&PLACE_WORD)
        || _a_trip_in_the_thread(memory)
        || (reads_thread && (PLACE_WORD.search(&around).is_some() || !_a_destination(&around).is_empty()));
    if a_place {
        return Some(pick(&A_VISIT_REPLY, memory, &said));
    }
    if has(&A_PURCHASE) {
        return Some(pick(&A_PURCHASE_REPLY, memory, &said));
    }
    None
}

/// Python: bot/compose/is_it_worth_it.py:394 `_with_what_they_said(reply, memory, said)`
///
/// The same reply without its closing question when the person has just answered it.
pub fn _with_what_they_said(reply: Option<String>, memory: &Memory, said: &str) -> Option<String> {
    let reply = reply?;
    if reply.is_empty() {
        return Some(reply);
    }
    let before = users(memory, 2);
    if A_FAULT_ALREADY_SAID.search(&format!("{said} {before}")).is_some() {
        let stripped = LAST_ASK.sub("", &reply);
        if !stripped.is_empty() && stripped != reply && stripped.ends_with('.') {
            return Some(stripped);
        }
    }
    Some(reply)
}

/// Python: bot/compose/is_it_worth_it.py:407 `is_it_worth_it(text, memory, reply)`
///
/// The pool's reply, minus a question the person has already answered this conversation.
pub fn is_it_worth_it(text_: &str, memory: &Memory, reply: &str) -> Option<String> {
    let said = text::squash(&text_.to_lowercase());
    _with_what_they_said(_is_it_worth_it(text_, memory, reply), memory, &said)
}
