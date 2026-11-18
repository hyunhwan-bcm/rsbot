//! Port of `bot/compose/a_wait_about_a_body.py`: the wait before an answer about a body (a scan,
//! a biopsy, "they think it might be cancer"), the news itself, and the good turn when the
//! serious thing is ruled out.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_strs, inline_re};
use crate::compose::pick;
use crate::py::re::Match;
use crate::state::Memory;

bundle_res!("bot.compose.a_wait_about_a_body"; OWN_SERIOUS_CONDITION, A_FINDING, A_BENIGN_FINDING, ITS_THE_SPEAKERS,
    NOT_ABOUT_THEIR_BODY, A_SCARE, A_GOOD_TURN, NOT_LANDED, A_WORRY, A_MEDICAL_TEST, NOT_A_MEDICAL_TEST, AN_ANIMAL,
    A_QUESTION_ABOUT_IT, _A_RELATIVE, RULED_OUT, NEGATED_CONDITION, RULED_OUT_TAIL);
bundle_strs!("bot.compose.a_wait_about_a_body"; THE_WAIT, THE_WAIT_AFTER_A_TEST, THEIR_WAIT, THE_NEWS,
    THE_ILLNESS_GOING, THE_THING_RULED_OUT);
inline_re!(OPENS_AN_ASK, r"\s*(?:is|are|was|were|did|does|do|has|have|had|can|could|would|will|should|any)\b");
inline_re!(OPENS_A_QUESTION,
    r"\s*(?:is|are|was|were|will|would|could|can|has|had|have|did|does|do|any\s+chance|do\s+you\s+think)\b");

/// Python: bot/compose/a_wait_about_a_body.py:197 `_the_relative(text)`
pub fn _the_relative(text: &str) -> Option<Match<'_>> {
    _A_RELATIVE.search(text)
}

/// Python: bot/compose/a_wait_about_a_body.py:201 `_the_wait(text)`
///
/// The thing the line is waiting on: a condition, a finding, or a test with an answer coming.
pub fn _the_wait(text: &str) -> Option<Match<'_>> {
    if let Some(found) = OWN_SERIOUS_CONDITION.search(text).or_else(|| A_FINDING.search(text)) {
        return Some(found);
    }
    if NOT_A_MEDICAL_TEST.search(text).is_some() {
        return None;
    }
    A_MEDICAL_TEST.search(text)
}

/// Python: bot/compose/a_wait_about_a_body.py:211 `tells_of_a_condition_of_their_own(text)`
///
/// Whether the line tells the bot a serious condition about the person typing it.
pub fn tells_of_a_condition_of_their_own(text: &str) -> bool {
    if text.is_empty() || NOT_ABOUT_THEIR_BODY.search(text).is_some() {
        return false;
    }
    if AN_ANIMAL.search(text).is_some() || A_QUESTION_ABOUT_IT.search(text).is_some() {
        return false;
    }
    let condition = match OWN_SERIOUS_CONDITION.search(text) {
        Some(c) => c,
        None => match A_FINDING.search(text) {
            Some(c) if A_BENIGN_FINDING.search(text).is_none() => c,
            _ => return false,
        },
    };
    let Some(owner) = ITS_THE_SPEAKERS.search(text) else { return false };
    if A_SCARE.search(text).is_some() || A_GOOD_TURN.search(text).is_some() {
        return false;
    }
    if let Some(landed) = NOT_LANDED.search(text) {
        if landed.start() < condition.start() {
            return false;
        }
    }
    if let Some(mine) = _the_relative(text) {
        if mine.start() < owner.start() {
            return false;
        }
    }
    true
}

/// The readings every wait shares: about a body, not an animal or a question, no scare or
/// good turn, a worry and a thing waited on.
fn a_wait_at_all(text: &str) -> bool {
    if text.is_empty() || NOT_ABOUT_THEIR_BODY.search(text).is_some() {
        return false;
    }
    if AN_ANIMAL.search(text).is_some() || A_QUESTION_ABOUT_IT.search(text).is_some() {
        return false;
    }
    if A_SCARE.search(text).is_some() || A_GOOD_TURN.search(text).is_some() {
        return false;
    }
    A_WORRY.search(text).is_some() && _the_wait(text).is_some()
}

/// Python: bot/compose/a_wait_about_a_body.py:236 `reads_as_the_wait_of_their_own(text)`
///
/// Whether the line is the wait before an answer about the person's own body.
pub fn reads_as_the_wait_of_their_own(text: &str) -> bool {
    if !a_wait_at_all(text) {
        return false;
    }
    if _the_relative(text).is_some() {
        return tells_of_a_condition_of_their_own(text);
    }
    !tells_of_a_condition_of_their_own(text)
}

/// Python: bot/compose/a_wait_about_a_body.py:251 `reads_as_the_wait_of_somebody_elses(text)`
///
/// The same wait, about a relative: "my mum might have cancer", "my dads scan is on monday".
pub fn reads_as_the_wait_of_somebody_elses(text: &str) -> bool {
    a_wait_at_all(text) && _the_relative(text).is_some()
}

/// Python: bot/compose/a_wait_about_a_body.py:264 `reads_as_a_thing_ruled_out(text)`
///
/// Whether the line says the serious thing was ruled out - the good direction, not a wait.
pub fn reads_as_a_thing_ruled_out(text: &str) -> bool {
    let said = com4::flat(text);
    if said.is_empty() || said.contains('?') {
        return false;
    }
    // A question is not the news, whichever way round it is typed: "is it not cancer".
    if A_QUESTION_ABOUT_IT.search(&said).is_some() || OPENS_AN_ASK.match_(&said).is_some() {
        return false;
    }
    for m in RULED_OUT.finditer(&said) {
        if RULED_OUT_TAIL.match_(&said[m.end()..]).is_some() {
            return true;
        }
    }
    for m in NEGATED_CONDITION.finditer(&said) {
        if RULED_OUT_TAIL.match_(&said[m.end()..]).is_some() {
            return true;
        }
    }
    false
}

/// Python: bot/compose/a_wait_about_a_body.py:284 `reads_as_the_news_of_their_own(text)`
///
/// Whether the line tells the news itself about the person's own body: "i have cancer".
pub fn reads_as_the_news_of_their_own(text: &str) -> bool {
    tells_of_a_condition_of_their_own(text) && A_WORRY.search(text).is_none()
}

/// Python: bot/compose/a_wait_about_a_body.py:291 `reads_as_the_illness_going(text)`
///
/// Whether the line says the condition itself is over: "the cancer is gone", "my tumour shrank".
pub fn reads_as_the_illness_going(text: &str) -> bool {
    if text.is_empty() || NOT_ABOUT_THEIR_BODY.search(text).is_some() {
        return false;
    }
    if OWN_SERIOUS_CONDITION.search(text).is_none() {
        return false;
    }
    let Some(turn) = A_GOOD_TURN.search(text) else { return false };
    let before = &text[..turn.start()];
    if text.contains('?') || NOT_LANDED.search(if before.is_empty() { " " } else { before }).is_some() {
        return false;
    }
    if OPENS_A_QUESTION.match_(text).is_some() {
        return false;
    }
    // The good word has to be the end of the news: "the cancer is gone but they want another
    // scan in a year" is relief and a worry.
    RULED_OUT_TAIL.match_(&text[turn.end()..]).is_some()
}

/// Python: bot/compose/a_wait_about_a_body.py:310 `pool_for(text)`
///
/// The pool this line belongs to, or None when it is not one of these five readings.
pub fn pool_for(text: &str) -> Option<&'static Vec<String>> {
    if reads_as_a_thing_ruled_out(text) {
        return Some(&THE_THING_RULED_OUT);
    }
    if reads_as_the_illness_going(text) {
        return Some(&THE_ILLNESS_GOING);
    }
    if reads_as_the_news_of_their_own(text) {
        return Some(&THE_NEWS);
    }
    if reads_as_the_wait_of_somebody_elses(text) {
        return Some(&THEIR_WAIT);
    }
    if reads_as_the_wait_of_their_own(text) {
        return Some(&THE_WAIT);
    }
    None
}

/// Python: bot/compose/a_wait_about_a_body.py:325 `names_a_condition(text)`
///
/// Whether the line names a condition or a finding, not just a test that is booked.
pub fn names_a_condition(text: &str) -> bool {
    let said = crate::py::text::squash(&text.to_lowercase());
    OWN_SERIOUS_CONDITION.search(&said).is_some() || A_FINDING.search(&said).is_some()
}

/// Python: bot/compose/a_wait_about_a_body.py:331 `reply_for(text, memory=None)`
///
/// The line for the wait or the ruled-out thing, or None when this file has nothing to say.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    let mut pool = pool_for(text).filter(|p| !p.is_empty())?;
    if std::ptr::eq(pool, &*THE_WAIT) && !names_a_condition(text) {
        pool = &THE_WAIT_AFTER_A_TEST;
    }
    Some(pick(pool, memory, text))
}
