//! Port of `bot/compose/plan_ask.py`: "so whats the plan" when the thread already holds a day
//! out, answered with the order of the day and the person's own settled lines said back.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_sets};
use crate::py::text;
use crate::state::Memory;

bundle_res!("bot.compose.plan_ask"; PLAN_ASK, PLAN_PLACE, PLAN_MEAL, PLAN_BED, PLAN_BOOKED, PLAN_TOGETHER, PLAN_DOORS,
    PLAN_QUESTION);
bundle_sets!("bot.compose.plan_ask"; PLAN_EMPTY);

/// The plan the thread holds: what it is for, what is settled, the order, what to book, and
/// whether the booking is done.
pub type Plan = (String, Vec<String>, String, String, bool);

/// Python: bot/compose/plan_ask.py:59 `_a_statement(line)`
///
/// The line, if it says how the thing will be, with nothing of the plan machinery in it.
pub fn _a_statement(line: &str) -> String {
    let squashed = text::squash(&line.to_lowercase());
    let said = text::strip_chars(&squashed, " .!,");
    if said.is_empty() || said.contains('?') || PLAN_QUESTION.match_(said).is_some() {
        return String::new();
    }
    let words = text::split_ref(said);
    if words.len() < 2 || words.len() > 14 || words.iter().all(|w| PLAN_EMPTY.contains(*w)) {
        return String::new();
    }
    said.to_string()
}

/// Python: bot/compose/plan_ask.py:71 `_the_plan_the_thread_holds(memory, ask='')`
///
/// The thing the plan is for, and what the person has settled about it; None for Python's
/// `(None, [])`. It has to be a day out: two of its parts named, one of them a place or a bed.
pub fn _the_plan_the_thread_holds(memory: &Memory, ask: &str) -> Option<Plan> {
    let turns = &memory.turns;
    // The ask itself is skipped by its own words rather than by its position.
    let ask = text::squash(&ask.to_lowercase());
    let said: Vec<String> = turns
        .iter()
        .filter(|t| text::squash(&t.user.to_lowercase()) != ask)
        .map(|t| _a_statement(&t.user))
        .filter(|s| !s.is_empty())
        .collect();
    if turns.len() < 3 || said.len() < 2 {
        return None;
    }
    let thread = said.join(" ");
    let has = |rx: &crate::py::re::Re| rx.search(&thread).is_some();
    let parts = [&*PLAN_PLACE, &*PLAN_MEAL, &*PLAN_BED, &*PLAN_TOGETHER].iter().filter(|rx| has(rx)).count();
    if parts < 2 || !(has(&PLAN_PLACE) || has(&PLAN_BED)) {
        return None;
    }
    let thing = said[0].clone();
    let (meal, bed, booked) = (has(&PLAN_MEAL), has(&PLAN_BED), has(&PLAN_BOOKED));
    let mut settled: Vec<String> = Vec::new();
    for one in said[1..].iter().rev() {
        if settled.contains(one) {
            continue;
        }
        settled.push(one.clone());
        if settled.len() == 2 {
            break;
        }
    }
    settled.reverse();
    // The order is only as long as the day is.
    let mut order = "the hour that cannot move".to_string();
    if meal {
        order.push_str(", then the meal");
    }
    if bed {
        order.push_str(", then the bed");
    }
    let book = match (meal, bed) {
        (true, true) => "the table and the room",
        (true, false) => "the table",
        (false, true) => "the room",
        _ => "the two bookings",
    };
    Some((thing, settled, order, book.to_string(), booked))
}

/// Python: bot/compose/plan_ask.py:132 `_quoted_back(settled)`
///
/// The lines said back in the person's own words, joined the way a person says a list.
pub fn _quoted_back<S: AsRef<str>>(settled: &[S]) -> String {
    let quoted: Vec<String> = settled.iter().map(|s| format!("\"{}\"", s.as_ref())).collect();
    match quoted.len() {
        1 => quoted[0].clone(),
        2 => format!("{} and {}", quoted[0], quoted[1]),
        n => format!("{} and {}", quoted[..n - 1].join(", "), quoted[n - 1]),
    }
}

/// Python: bot/compose/plan_ask.py:142 `plan_ask_pool(clean, memory=None)`
///
/// The pool for "what's the plan" when the thread already holds one, else None.
pub fn plan_ask_pool(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let said = text::squash(&clean.to_lowercase());
    PLAN_ASK.search(&said)?;
    let (thing, settled, order, book, booked) = _the_plan_the_thread_holds(memory, &said)?;
    if settled.is_empty() {
        return None;
    }
    let mut named: Vec<String> = vec![thing.clone()];
    named.extend(settled.iter().cloned());
    let said_back = _quoted_back(&named);
    // The one thing the bot cannot know is the hour, so it names where it is written.
    let left = if PLAN_DOORS.search(&format!("{} {thing}", settled.join(" "))).is_some() {
        "the doors time - the venue's own page has that"
    } else {
        "the hour you have to be there - the booking page has that"
    };
    let (tail, close) = if booked {
        (
            "The one thing that runs out is done already, so the hours are all that is left.".to_string(),
            "Fix the hours round it and the day runs itself.".to_string(),
        )
    } else {
        (format!("Book {book} tonight: the rest is only times."), format!("Get {book} settled and the day runs itself."))
    };
    Some(vec![
        format!("You have told me {said_back}, so the order is {order}. {tail}"),
        format!("I have it all: {said_back}. The order is {order}, and the one thing left to find is {left}."),
        format!(
            "Nothing here needs looking up - you have told me {said_back}, and that is a day out with the order already in it: {order}. {close}"
        ),
    ])
}
