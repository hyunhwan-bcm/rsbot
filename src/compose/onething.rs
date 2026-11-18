//! Port of `bot/compose/onething.py`: one thing that is true about what an account names (water
//! in the house, short pay, an early shift, a visit on a day), or the subject alone.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_sets, bundle_strs, bundle_vals, format_named, inline_re};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

bundle_res!("bot.compose.onething"; A_QUESTION_OPENER, A_QUESTION_ANYWHERE, A_WATER_TROUBLE, A_PAY_OWN, A_PAY_SOMEBODY_ELSE,
    A_SHORT_PAY, A_SHIFT, AN_EARLY_HOUR, A_STANDING, A_BABY, AN_AGE_UNDER_A_YEAR, A_FEED, A_FEED_FIGHT, A_COMING_VISIT,
    A_VISIT_DAY, A_PAST_VISIT, A_COMING_THAT_IS_NOT_A_VISIT, A_MEDS, A_MEDS_CHANGE, A_MEDS_MINE, A_MEDS_SYMPTOM);
bundle_sets!("bot.compose.onething"; WATER_THINGS, POSTURAL, VISITORS, A_CARE_VISITOR, A_TRADE_VISITOR, A_PERSON_IN_THE_VISIT,
    A_SUBJECT_FILLER, A_SUBJECT_BESIDE);
bundle_strs!("bot.compose.onething"; WATER_IN_THE_HOUSE, SHORT_PAY, EARLY_START, A_STANDING_UP, UNDER_A_YEAR, A_FEED_FIGHT_REPLY,
    A_VISIT_CARE, A_VISIT_TRADE, A_VISIT_PLACE, A_MEDS_CHANGE_REPLY);
bundle_vals!("bot.compose.onething"; A_SUBJECT_FAMILY);
inline_re!(A_WORD, r"[a-z']+");

static SUBJECT_FAMILIES: LazyLock<Vec<(&'static Re, Vec<String>)>> = LazyLock::new(|| {
    A_SUBJECT_FAMILY.items().iter().map(|row| (row.items()[0].as_re().expect("a pattern"), row.items()[1].strs())).collect()
});

fn words_of(text_: &str) -> HashSet<String> {
    A_WORD.findall_str(text_).into_iter().collect()
}

/// Python: bot/compose/onething.py:260 `_a_visit(text)`
///
/// True when the line says somebody is turning up on a day.
pub fn _a_visit(text_: &str) -> bool {
    let words = words_of(text_);
    if !words.iter().any(|w| VISITORS.contains(w)) || !words.iter().any(|w| A_PERSON_IN_THE_VISIT.contains(w)) {
        return false;
    }
    if A_VISIT_DAY.search(text_).is_none() || A_PAST_VISIT.search(text_).is_some() {
        return false;
    }
    if A_COMING_THAT_IS_NOT_A_VISIT.search(text_).is_some() {
        return false;
    }
    A_COMING_VISIT.search(text_).is_some()
}

/// Python: bot/compose/onething.py:275 `_the_visit_family(text)`
///
/// The pool for who is coming: a person's care, a thing in the house, or the place itself.
pub fn _the_visit_family(text_: &str) -> &'static Vec<String> {
    let words = words_of(text_);
    if words.iter().any(|w| A_CARE_VISITOR.contains(w)) {
        return &A_VISIT_CARE;
    }
    if words.iter().any(|w| A_TRADE_VISITOR.contains(w)) {
        return &A_VISIT_TRADE;
    }
    &A_VISIT_PLACE
}

/// Python: bot/compose/onething.py:284 `_has(text, words)`
pub fn _has<S: AsRef<str>>(text_: &str, words: &[S]) -> bool {
    let found = words_of(text_);
    words.iter().any(|w| found.contains(w.as_ref()))
}

fn has_set(text_: &str, set: &HashSet<String>) -> bool {
    words_of(text_).iter().any(|w| set.contains(w))
}

/// Python: bot/compose/onething.py:535 `_a_subject_alone(text)`
///
/// The family for a line that is nothing but the name of a subject, else None.
pub fn _a_subject_alone(text_: &str) -> Option<Vec<String>> {
    let mut stripped = text_.to_string();
    let mut family: Option<&Vec<String>> = None;
    for (pattern, lines) in SUBJECT_FAMILIES.iter() {
        if pattern.search(&stripped).is_none() {
            continue;
        }
        family = family.or(Some(lines));
        stripped = pattern.sub(" ", &stripped);
    }
    let family = family?;
    if A_WORD.findall_str(&stripped).iter().any(|w| !A_SUBJECT_FILLER.contains(w) && !A_SUBJECT_BESIDE.contains(w)) {
        return None;
    }
    Some(family.clone())
}

/// Python: bot/compose/onething.py:558 `_a_statement(clean)`
///
/// The line when it is an account and not a question, else None.
pub fn _a_statement(clean: &str) -> Option<String> {
    let t = text::squash(&clean.to_lowercase());
    if t.is_empty() || t.contains('?') || A_QUESTION_OPENER.match_(&t).is_some() {
        return None;
    }
    if A_QUESTION_ANYWHERE.search(&t).is_some() {
        return None;
    }
    // A state of the person's own said that short is still a line.
    if text::split_ref(&t).len() < 3 && !(A_MEDS.search(&t).is_some() && A_MEDS_CHANGE.search(&t).is_some()) {
        return None;
    }
    Some(t)
}

/// Python: bot/compose/onething.py:573 `pool(clean, understood=None, memory=None)`
///
/// One thing that is true about what the line names, or None when it names nothing.
pub fn pool(clean: &str, understood: &Understood, memory: &Memory) -> Option<Vec<String>> {
    let _ = (understood, memory);
    // A subject and nothing else is read before the three-word floor below.
    let whole = text::squash(&clean.to_lowercase());
    if !whole.is_empty() && !whole.contains('?') && A_QUESTION_OPENER.match_(&whole).is_none() && A_QUESTION_ANYWHERE.search(&whole).is_none() {
        if let Some(family) = _a_subject_alone(&whole).filter(|f| !f.is_empty()) {
            return Some(family);
        }
    }
    let t = _a_statement(clean)?;
    let has = |rx: &Re| rx.search(&t).is_some();
    if (has(&A_STANDING) && has_set(&t, &POSTURAL)) || (_has(&t, &["vision", "eyes", "sight"]) && has(&A_STANDING)) {
        return Some(A_STANDING_UP.clone());
    }
    if has_set(&t, &WATER_THINGS) && has(&A_WATER_TROUBLE) {
        return Some(WATER_IN_THE_HOUSE.clone());
    }
    if has(&A_PAY_OWN) && has(&A_SHORT_PAY) && !has(&A_PAY_SOMEBODY_ELSE) {
        return Some(SHORT_PAY.clone());
    }
    if has(&A_BABY) && has(&A_FEED) && has(&A_FEED_FIGHT) {
        return Some(A_FEED_FIGHT_REPLY.clone());
    }
    if has(&A_BABY) && has(&AN_AGE_UNDER_A_YEAR) {
        return Some(UNDER_A_YEAR.clone());
    }
    if has(&A_SHIFT) && has(&AN_EARLY_HOUR) {
        return Some(EARLY_START.clone());
    }
    if has(&A_MEDS) && has(&A_MEDS_CHANGE) && A_MEDS_MINE.match_(&t).is_some() && !has(&A_MEDS_SYMPTOM) {
        return Some(A_MEDS_CHANGE_REPLY.clone());
    }
    if _a_visit(&t) {
        let day = A_VISIT_DAY.search(&t).expect("a day, read above");
        let word = day.str(1).to_lowercase();
        let cap = text::upper_first(&word);
        return Some(_the_visit_family(&t).iter().map(|line| format_named(line, &[("day", &word), ("Day", &cap)])).collect());
    }
    None
}
