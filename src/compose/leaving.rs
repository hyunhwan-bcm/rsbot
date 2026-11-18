//! Port of `bot/compose/leaving.py`: the person saying goodbye, with or without a reason
//! around it, and the goodbye (or the wish back) that answers it.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_sets, bundle_strs, inline_re};
use crate::data::Value;
use crate::py::text;
use crate::state::Memory;
use std::sync::Arc;

bundle_res!("bot.compose.leaving"; _A_SIGN_OFF, _A_LEAVE_TIME, _LEAVE, _A_DAY_AHEAD, _WISH, _OPENS_QUESTION, _ASKS,
    _ASKS_ANYWHERE, _ASKS_OPENER, _KEEPS, _SPLIT, _NOT_A_BEDTIME, _NITE, _END_SIGN_OFF, _UNWELL, _HURTING, _BODY,
    _CONTRAST, _COMES_BACK, _MIND, _OVERWHELM, _FEELS_ROTTEN, _SIGN_OFF, _AWAY_OUT, _AWAY_PLAN, _BED_LEAVE, _NEXT_DAY,
    _AWAY, _NIGHT, _WELL, _WISH_THING, _UNWELL_FEELING, _OWN_STATE, _ANOTHER);
bundle_sets!("bot.compose.leaving"; _WISH_ALLOWED, _ALLOWED, _THANKS_WORDS, _RUN_UP, _SIGN_OFF_WORDS, _SIGN_OFF_VOCATIVE,
    _END_SIGN_OFF_KEEP, _LEAVE_SHORTHAND);
bundle_strs!("bot.compose.leaving"; POOL, NIGHT_GOODBYE_POOL, REASON_POOL, WISH_POOL, WELL_POOL, CRISIS_POOL, KINDER_POOL,
    NIGHT_POOL, OWN_NEWS_POOL);
inline_re!(A_WORD, r"[a-z']+");

fn words_of(s: &str) -> Vec<String> {
    A_WORD.findall_str(s)
}

/// Python: bot/compose/leaving.py:338 `_clauses(clean)`
pub fn _clauses(clean: &str) -> Vec<String> {
    _SPLIT.split(clean).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect()
}

/// Python: bot/compose/leaving.py:375 `_is_sign_off(clause)`
///
/// True when a clause is only a sign-off word or two, with who it is said to.
pub fn _is_sign_off(clause: &str) -> bool {
    let words: Vec<String> = words_of(clause).into_iter().filter(|w| !_SIGN_OFF_VOCATIVE.contains(w)).collect();
    !words.is_empty() && words.iter().all(|w| _SIGN_OFF_WORDS.contains(w))
}

/// Python: bot/compose/leaving.py:399 `_ends_in_a_sign_off(clause)`
///
/// True when the clause finishes on a sign-off word and nothing of its own before it.
pub fn _ends_in_a_sign_off(clause: &str) -> bool {
    let Some(m) = _END_SIGN_OFF.search(clause) else { return false };
    let head = text::strip(&clause[..m.start()]);
    words_of(head).iter().all(|w| _END_SIGN_OFF_KEEP.contains(w))
}

/// Python: bot/compose/leaving.py:409 `_is_thanks_clause(clause)`
pub fn _is_thanks_clause(clause: &str) -> bool {
    let words = words_of(clause);
    !words.is_empty() && words.iter().all(|w| _THANKS_WORDS.contains(w))
}

/// Python: bot/compose/leaving.py:442 `_reads_as_a_body_complaint(clause)`
pub fn _reads_as_a_body_complaint(clause: &str) -> bool {
    _BODY.search(clause).is_some() && _HURTING.search(clause).is_some()
}

/// Python: bot/compose/leaving.py:447 `_reads_as_unwell(clause)`
pub fn _reads_as_unwell(clause: &str) -> bool {
    _UNWELL.search(clause).is_some() || _reads_as_a_body_complaint(clause)
}

/// Python: bot/compose/leaving.py:493 `_a_trouble_behind_a_contrast(clean)`
///
/// True when a contrast sets the person's own trouble against the leaving ("im going to bed but
/// i cant switch off").
pub fn _a_trouble_behind_a_contrast(clean: &str) -> bool {
    if _CONTRAST.search(clean).is_none() {
        return false;
    }
    // `_CONTRAST.split(clean, 1)[1]`: what follows the first contrast (the pattern has no
    // groups, so the second piece is the rest of the line).
    let first = _CONTRAST.search(clean).expect("searched above");
    let tail = text::squash(&clean[first.end()..]);
    if tail.is_empty() {
        return false;
    }
    if _is_thanks_clause(&tail) || _is_sign_off(&tail) || _is_wish_clause(&tail) {
        return false;
    }
    if _names_their_own_trouble(&tail) {
        return true;
    }
    _COMES_BACK.search(&tail).is_some() && _MIND.search(&tail).is_some()
}

/// Python: bot/compose/leaving.py:515 `_names_their_own_trouble(text)`
pub fn _names_their_own_trouble(text_: &str) -> bool {
    _OVERWHELM.search(text_).is_some() || _FEELS_ROTTEN.search(text_).is_some()
}

/// Python: bot/compose/leaving.py:542 `reads_as_a_sign_off(reply)`
///
/// True when a reply lets the person go rather than answering the reason they gave.
pub fn reads_as_a_sign_off(reply: &str) -> bool {
    _SIGN_OFF.search(&text::squash(&reply.to_lowercase())).is_some()
}

/// Python: bot/compose/leaving.py:552 `_is_wish_clause(clause)`
///
/// True when this clause is a goodbye said as a wish and nothing else.
pub fn _is_wish_clause(clause: &str) -> bool {
    let Some(m) = _WISH.search(clause) else { return false };
    words_of(&clause[m.end()..]).iter().all(|w| _WISH_ALLOWED.contains(w))
}

/// Python: bot/compose/leaving.py:572 `_is_leave_clause(clause)`
///
/// True when this clause is a leave and nothing else.
pub fn _is_leave_clause(clause: &str) -> bool {
    if _is_wish_clause(clause) || _ends_in_a_sign_off(clause) {
        return true;
    }
    if _LEAVE.search(clause).is_none() {
        return false;
    }
    // Going away somewhere later is a plan, not a goodbye.
    if _AWAY_OUT.search(clause).is_some() && _AWAY_PLAN.search(clause).is_some() {
        return false;
    }
    // A night named for another day is somebody arranging one, not tonight's goodbye.
    if _BED_LEAVE.search(clause).is_some() && _NEXT_DAY.search(clause).is_some() {
        return false;
    }
    let rest = _LEAVE.sub(" ", clause);
    // Who the goodbye is said to is part of saying it.
    let down = |w: &String| _ALLOWED.contains(w) || _SIGN_OFF_WORDS.contains(w) || _LEAVE_SHORTHAND.contains(w) || _SIGN_OFF_VOCATIVE.contains(w);
    if words_of(&rest).iter().all(down) {
        return true;
    }
    // A time is part of a sign-off, not of every leave.
    if _A_SIGN_OFF.search(clause).is_none() {
        return false;
    }
    let trimmed = _A_LEAVE_TIME.sub(" ", &rest);
    trimmed != rest && words_of(&trimmed).iter().all(down)
}

/// Python: bot/compose/leaving.py:606 `reads_as_leaving(clean, words=None)`
///
/// True when the person is saying goodbye, with or without a reason around it.
pub fn reads_as_leaving(clean: &str, words: Option<&[String]>) -> bool {
    _reads_as_leaving(clean, words, true, true)
}

/// Python: bot/compose/leaving.py:617 `reads_as_a_leave_with_a_body_in_it(clean, words=None)`
///
/// True when a goodbye stops being one only because the reason names a body in trouble.
pub fn reads_as_a_leave_with_a_body_in_it(clean: &str, words: &[String]) -> bool {
    !reads_as_leaving(clean, Some(words)) && _reads_as_leaving(clean, Some(words), false, true)
}

/// Python: bot/compose/leaving.py:629 `reads_as_a_leave_with_a_trouble_in_it(clean, words=None)`
///
/// True when a goodbye stops being one only because a contrast sets a trouble against it.
pub fn reads_as_a_leave_with_a_trouble_in_it(clean: &str, words: &[String]) -> bool {
    !reads_as_leaving(clean, Some(words)) && _reads_as_leaving(clean, Some(words), true, false)
}

/// Python: bot/compose/leaving.py:643 `_reads_as_leaving(clean, words=None, unwell_blocks=True, trouble_blocks=True)`
pub fn _reads_as_leaving(clean: &str, words: Option<&[String]>, unwell_blocks: bool, trouble_blocks: bool) -> bool {
    let _ = words;
    let clean = text::squash(&clean.to_lowercase());
    if clean.is_empty() || clean.contains('?') {
        return false;
    }
    if _OPENS_QUESTION.match_(&clean).is_some() || _KEEPS.search(&clean).is_some() {
        return false;
    }
    if trouble_blocks && _a_trouble_behind_a_contrast(&clean) {
        return false;
    }
    let clauses = _clauses(&clean);
    if clauses.is_empty() {
        return false;
    }
    if clauses.len() == 1 {
        return _is_leave_clause(&clauses[0]);
    }
    if clauses.iter().any(|c| _asks_something(c)) {
        // A leave with a question in it is not a leave: the question is the turn.
        return false;
    }
    let (first, last) = (&clauses[0], &clauses[clauses.len() - 1]);
    if _is_leave_clause(first) {
        let rest = clauses[1..].join(" ");
        // The tail is the reason: a statement, not somebody being unwell, not their own trouble.
        return _ASKS.search(&rest).is_none()
            && !(unwell_blocks && _reads_as_unwell(&rest))
            && !(trouble_blocks && _names_their_own_trouble(&rest));
    }
    if _is_leave_clause(last) {
        return true;
    }
    // The leave can also be in the middle, behind the front of a goodbye.
    for index in 1..clauses.len() - 1 {
        if !_is_leave_clause(&clauses[index]) {
            continue;
        }
        let before = clauses[..index].join(" ");
        let after = clauses[index + 1..].join(" ");
        if _ASKS.search(&after).is_some() || (unwell_blocks && _reads_as_unwell(&after)) {
            continue;
        }
        if !before.is_empty() && !(_is_filler(&before) || _is_thanks_clause(&before)) {
            continue;
        }
        return true;
    }
    false
}

/// Python: bot/compose/leaving.py:719 `_asks_something(clause)`
///
/// True when a clause is itself a question, with or without the question mark.
pub fn _asks_something(clause: &str) -> bool {
    let clause = text::squash(&clause.to_lowercase());
    if clause.is_empty() {
        return false;
    }
    if clause.contains('?') {
        return true;
    }
    _ASKS_OPENER.match_(&clause).is_some() || _ASKS_ANYWHERE.search(&clause).is_some()
}

/// Python: bot/compose/leaving.py:734 `_is_filler(clause)`
///
/// True when a clause is only the run-up to a goodbye: "ok", "right then", "ok thanks".
pub fn _is_filler(clause: &str) -> bool {
    let words = words_of(clause);
    !words.is_empty() && words.iter().all(|w| _RUN_UP.contains(w))
}

/// Python: bot/compose/leaving.py:814 `_thread_is_an_emergency(memory)`
///
/// True when the last thing said was an emergency reply (a case the safety module recorded).
pub fn _thread_is_an_emergency(memory: &Memory) -> bool {
    if !memory.get("safety_case").is_some_and(Value::truthy) {
        return false;
    }
    memory.turns.last().is_some_and(|t| t.module.as_deref() == Some("safety"))
}

/// Python: bot/compose/leaving.py:838 `_thread_is_unwell(memory)`
///
/// True when the last thing the person said was that they (or their animal) are unwell.
pub fn _thread_is_unwell(memory: &Memory) -> bool {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(2)..]
        .iter()
        .rev()
        .any(|t| _reads_as_unwell(&t.user) || _UNWELL_FEELING.search(&t.user.to_lowercase()).is_some())
}

/// Python: bot/compose/leaving.py:853 `_wish_pool(clean, memory=None)`
///
/// The wish-back lines for a goodbye said as a wish.
pub fn _wish_pool(clean: &str, memory: &Memory) -> &'static Vec<String> {
    let line = text::squash(&clean.to_lowercase());
    if _thread_is_an_emergency(memory) {
        return &CRISIS_POOL;
    }
    if _thread_is_unwell(memory) {
        return &KINDER_POOL;
    }
    if _WELL.search(&line).is_some() {
        return &WELL_POOL;
    }
    if _NIGHT.search(&line).is_some() {
        return &NIGHT_POOL;
    }
    &WISH_POOL
}

/// Python: bot/compose/leaving.py:867 `pool(clean, words=None, unwell_blocks=True, memory=None, trouble_blocks=True)`
///
/// The goodbye lines for this leave, or None when the line is not one.
pub fn pool(clean: &str, words: &[String], unwell_blocks: bool, memory: &Memory, trouble_blocks: bool) -> Option<Vec<String>> {
    if !_reads_as_leaving(clean, Some(words), unwell_blocks, trouble_blocks) {
        return None;
    }
    let clauses = _clauses(&text::squash(&clean.to_lowercase()));
    let mut wishes: Vec<&String> = clauses.iter().filter(|c| _is_wish_clause(c)).collect();
    if !wishes.is_empty() && _WISH_THING.search(&wishes.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" ")).is_none() {
        wishes.clear();
    }
    let leaving_too = clauses.iter().any(|c| _is_leave_clause(c) && !_is_wish_clause(c));
    // A wish with news in the same line is not the wish coming back.
    let own_news: Vec<&String> = clauses
        .iter()
        .filter(|c| !_is_wish_clause(c) && !_is_leave_clause(c) && !_is_filler(c) && !_is_thanks_clause(c) && !_is_sign_off(c))
        .collect();
    let opens_with_a_wish =
        !clauses.is_empty() && !wishes.is_empty() && _is_wish_clause(&clauses[0]) && _WISH_THING.search(&clauses[0]).is_some();
    let going_away = own_news.iter().any(|c| _AWAY.search(c).is_some());
    if !wishes.is_empty() && !leaving_too && !going_away {
        if !own_news.is_empty() && opens_with_a_wish {
            // The wish opens the line and the news follows it: the news is asked about.
            return Some(OWN_NEWS_POOL.clone());
        }
        if !own_news.is_empty() {
            // The wish is on the end of news that has an answer of its own elsewhere.
            return None;
        }
        // The whole line is a wish, so the wish comes back.
        let joined = wishes.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" ");
        return Some(_wish_pool(&joined, memory).clone());
    }
    // A reason in the line is answered before the leave is; a thank-you is not a reason.
    for (index, clause) in clauses.iter().enumerate() {
        if !_is_leave_clause(clause) {
            continue;
        }
        // The leave found is the one the reply answers.
        let leave_pool: &Vec<String> = if _is_a_bedtime(clause) { &NIGHT_GOODBYE_POOL } else { &POOL };
        let rest = &clauses[index + 1..];
        // Only a thanks, a sign-off, a wish or the leave said a second time are set aside;
        // anything else after the leave is the reason.
        if !rest.is_empty()
            && rest.iter().any(|part| !(_is_thanks_clause(part) || _is_sign_off(part) || _is_wish_clause(part) || _is_leave_clause(part)))
        {
            // "see you at 5 tomorrow, i have a meeting": the meeting is the plan for that day.
            if _A_DAY_AHEAD.search(clause).is_some() {
                return Some(leave_pool.clone());
            }
            if _a_state_not_an_errand(rest) {
                return Some(leave_pool.clone());
            }
            return Some(REASON_POOL.clone());
        }
        return Some(leave_pool.clone());
    }
    Some(if _is_a_bedtime(&clauses.join(" ")) { NIGHT_GOODBYE_POOL.clone() } else { POOL.clone() })
}

/// Python: bot/compose/leaving.py:960 `_a_state_not_an_errand(clauses)`
///
/// True when the reasons after the leave are the person's own state or their own day.
pub fn _a_state_not_an_errand<S: AsRef<str>>(clauses: &[S]) -> bool {
    let said: Vec<&str> = clauses
        .iter()
        .map(|c| c.as_ref())
        .filter(|c| !(_is_thanks_clause(c) || _is_sign_off(c) || _is_wish_clause(c) || _is_leave_clause(c)))
        .collect();
    if said.is_empty() {
        return false;
    }
    said.iter().all(|c| _OWN_STATE.search(c).is_some() && _ANOTHER.search(c).is_none())
}

/// Python: bot/compose/leaving.py:980 `_is_a_bedtime(clause)`
///
/// True when the leave itself is a bedtime: the bed or the night is what they are going to.
pub fn _is_a_bedtime(clause: &str) -> bool {
    if clause.is_empty() || _NOT_A_BEDTIME.search(clause).is_some() {
        return false;
    }
    _NITE.search(clause).is_some()
}

/// Python: bot/compose/leaving.py:987 `lines()`
///
/// Every goodbye this module can say, for the arbiter's "is this the sign-off" test (a frozenset
/// in Python; here a set value holding each line once).
pub fn lines() -> Value {
    let mut out: Vec<Value> = Vec::new();
    for pool in [&*POOL, &*NIGHT_GOODBYE_POOL, &*REASON_POOL, &*WISH_POOL, &*WELL_POOL, &*KINDER_POOL, &*NIGHT_POOL, &*CRISIS_POOL, &*OWN_NEWS_POOL] {
        for line in pool {
            let v = Value::from(line.as_str());
            if !out.contains(&v) {
                out.push(v);
            }
        }
    }
    Value::Set(Arc::new(out))
}
