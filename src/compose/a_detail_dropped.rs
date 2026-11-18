//! Port of `bot/compose/a_detail_dropped.py`: a receipt ("Got it. Is there anything you'd like
//! a hand with?") replaced by one short question about the thing the person just said.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_strs, bundle_vals};
use crate::compose::pick;
use crate::py::text;
use crate::state::Memory;

bundle_res!("bot.compose.a_detail_dropped"; A_RECEIPT, QUESTION, A_SMALL_LINE, A_FIGURE_ALONE, A_FIGURE_THEIRS,
    A_BODY_PART, A_BODY_QUALIFIER, A_THING_FAILING, A_THING_FAILING_WITH_ITS_OWNER, A_PERSON_OR_A_BOND,
    A_DAY_WITH_SOMETHING_ON, A_DAY_NAMED, A_DECISION_STUCK, A_LOAD, A_HEDGED_READ, A_WORK_STUCK, A_OWED,
    A_PAYMENT_NAMED, A_QUOTED_PRICE, A_PERSON_DID, A_COMPARISON, A_DAY_SIGHED, A_PAIR_NAMED);
bundle_strs!("bot.compose.a_detail_dropped"; THE_OWED, THE_QUOTED_PRICE, THE_PERSON_DID, THE_COMPARISON,
    THE_DAY_SIGHED, THE_LOAD, THE_HEDGED_READ, THE_HEDGED_READ_OF_A_PERSON, THE_WORK_STUCK, THE_FIGURE_ALONE,
    THE_FIGURE_THEIRS_DEFAULT, THE_BODY, THE_THING_FAILING, THE_DAY_WITH_SOMETHING_ON, THE_DECISION_PAIR,
    THE_DECISION_STUCK);
bundle_vals!("bot.compose.a_detail_dropped"; THE_FIGURE_THEIRS);

/// Python: bot/compose/a_detail_dropped.py:302 `reads_as_a_receipt(reply)`
///
/// True only for the composer's own acknowledgements.
pub fn reads_as_a_receipt(reply: &str) -> bool {
    !reply.is_empty() && A_RECEIPT.search(reply).is_some()
}

/// Python: bot/compose/a_detail_dropped.py:307 `turned(respond)`
///
/// `respond`, with a receipt replaced by a question about the thing the person just said.
/// (Python returns a closure over `respond`; so does this.)
pub fn turned<F>(respond: F) -> impl Fn(&str, &mut Memory) -> Option<String>
where
    F: Fn(&str, &mut Memory) -> Option<String>,
{
    move |text: &str, memory: &mut Memory| {
        let said = respond(text, memory);
        answer_for(text, memory, said.as_deref().unwrap_or("")).filter(|s| !s.is_empty()).or(said)
    }
}

/// Python: bot/compose/a_detail_dropped.py:315 `_money_kind(clean)`
pub fn _money_kind(clean: &str) -> Option<String> {
    let m = A_FIGURE_THEIRS.search(clean)?;
    let noun = m.str(1);
    if matches!(noun, "pay" | "wage" | "wages" | "income" | "salary") {
        return Some("pay".to_string());
    }
    Some(if THE_FIGURE_THEIRS.get(noun).is_some() { noun.to_string() } else { "default".to_string() })
}

/// Python: bot/compose/a_detail_dropped.py:325 `answer_for(text, memory=None, reply=None)`
///
/// One short question about the thing they just said, or None. Called with the reply the
/// composer chose: the whole point is that no reader wanted the turn.
pub fn answer_for(text: &str, memory: &Memory, reply: &str) -> Option<String> {
    if !reads_as_a_receipt(reply) {
        return None;
    }
    let understood_clean = memory.understood.as_ref().map(|u| u.clean.as_str()).unwrap_or("");
    let clean = if !understood_clean.is_empty() { understood_clean } else { text }.to_lowercase();
    let clean = text::squash(&clean.replace('\u{2019}', "'"));
    let word_count = text::split_ref(&clean).len();
    if clean.is_empty() || word_count > 18 {
        return None;
    }
    if QUESTION.match_(&clean).is_some() || text::strip(text).ends_with('?') {
        return None;
    }
    if A_SMALL_LINE.match_(&clean).is_some() {
        return None;
    }
    if A_OWED.search(&clean).is_some() {
        return Some(pick(&THE_OWED, memory, &clean));
    }
    let mut kind = _money_kind(&clean);
    if kind.is_none() && A_PAYMENT_NAMED.search(&clean).is_some() {
        let head = clean.split(" is ").next().unwrap_or("");
        kind = _money_kind(&format!("{clean} my {head}"));
        kind = Some(kind.unwrap_or_else(|| "default".to_string()));
    }
    if let Some(kind) = kind {
        let pool = match THE_FIGURE_THEIRS.get(&kind) {
            Some(v) if v.truthy() => v.strs(),
            _ => THE_FIGURE_THEIRS_DEFAULT.clone(),
        };
        return Some(pick(&pool, memory, &clean));
    }
    if A_FIGURE_ALONE.match_(&clean).is_some() {
        return Some(pick(&THE_FIGURE_ALONE, memory, &clean));
    }
    if A_WORK_STUCK.search(&clean).is_some()
        && A_PERSON_OR_A_BOND.search(&clean.replace("job", "").replace("work", "")).is_none()
    {
        return Some(pick(&THE_WORK_STUCK, memory, &clean));
    }
    if A_QUOTED_PRICE.search(&clean).is_some() {
        return Some(pick(&THE_QUOTED_PRICE, memory, &clean));
    }
    if A_COMPARISON.search(&clean).is_some() {
        return Some(pick(&THE_COMPARISON, memory, &clean));
    }
    if A_PERSON_DID.search(&clean).is_some() && A_PERSON_OR_A_BOND.search(&clean).is_none() {
        return Some(pick(&THE_PERSON_DID, memory, &clean));
    }
    if A_DAY_SIGHED.search(&clean).is_some() {
        return Some(pick(&THE_DAY_SIGHED, memory, &clean));
    }
    if A_LOAD.search(&clean).is_some() {
        return Some(pick(&THE_LOAD, memory, &clean));
    }
    if A_HEDGED_READ.search(&clean).is_some() {
        let pool: &Vec<String> =
            if A_PERSON_OR_A_BOND.search(&clean).is_some() { &THE_HEDGED_READ_OF_A_PERSON } else { &THE_HEDGED_READ };
        return Some(pick(pool, memory, &clean));
    }
    if A_DECISION_STUCK.search(&clean).is_some() {
        // The pair is asked about only where the line named one.
        let pool: Vec<String> = if A_PAIR_NAMED.search(&clean).is_some() {
            THE_DECISION_PAIR.iter().chain(THE_DECISION_STUCK.iter()).cloned().collect()
        } else {
            THE_DECISION_STUCK.clone()
        };
        return Some(pick(&pool, memory, &clean));
    }
    if A_BODY_PART.search(&clean).is_some() && A_BODY_QUALIFIER.search(&clean).is_some() {
        return Some(pick(&THE_BODY, memory, &clean));
    }
    if A_THING_FAILING.search(&clean).is_some()
        && A_PERSON_OR_A_BOND.search(&clean).is_none()
        && (A_THING_FAILING_WITH_ITS_OWNER.search(&clean).is_some() || word_count <= 8)
    {
        return Some(pick(&THE_THING_FAILING, memory, &clean));
    }
    if A_DAY_WITH_SOMETHING_ON.search(&clean).is_some() && A_DAY_NAMED.search(&clean).is_some() {
        return Some(pick(&THE_DAY_WITH_SOMETHING_ON, memory, &clean));
    }
    None
}
