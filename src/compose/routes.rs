//! Port of `bot/compose/routes.py`: where the answer to a rules, terms, cover or local-service
//! question lives (the policy, the operator's page, the council), said instead of a hedge.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_strs, inline_re};
use crate::py::re::Re;

bundle_res!("bot.compose.routes"; INSURANCE_WORD, TERMS_THING, PRICE_THING, INCLUDED_ASK, COVERED_PHRASE, TERMS_ASK,
    WORKING_OUT, ALLOWED_ASK, TAKE_ON, TAKE_TO, NEEDED_THING, NEEDED_ASK, AGREEMENT_WORD, NOT_AN_AGREEMENT, LEAVING_ASK,
    GET_OUT_OF, SHIFT_WORD, LEAVE_EARLY, CANCEL_TERMS, HARM_NOUN, TERM_SHAPE, PRICE_ASK, TERM_WORD, COUNT_AS,
    ALLOWANCE_WORD, NAMED_OBJECT, VENUE_WORDS, CONTRACT_WORDS, QUESTION, WARD, VISITOR, SOMEONES_PERSON, LAW_NAMED,
    CARRYING, TO_INSIDE, PERSON_ALLOWED, COURT_ARM, LOCAL_WORDS, SERVICE_WORDS, NOISE, COMPLAINT, NOT_A_SERVICE,
    ASKING_SHAPE, BODY_STATE, COVER_IT, COVERED_IN, TICKET_WORD, A_PLACE, A_FACILITY, A_ROOM_FACILITY, A_HAS_ASK,
    NOT_A_HAS, THE_OLD_ONE, MOVING_THE_OLD, NOT_THE_OLD_ONE);
bundle_strs!("bot.compose.routes"; ISSUER, LOCAL, TERMS, INCLUDED, LOCAL_SERVICE, OPERATOR, INSURER, ALLOWED, VENUE, CONTRACT,
    A_PLACE_HAS, THE_OLD_ONE_POOL);
inline_re!(OPENS_IN_THE_PAST, r"\s*(?:was|were)\b");
inline_re!(COVERED, r"\bcover(?:ed|s)?\b");
inline_re!(EARLY, r"\bearly\b");

/// Python: bot/compose/routes.py:547 `options(clean)`
///
/// A pool of "here is where to find out" replies for this line, or None. `clean` is the
/// matchable line: lower case, no apostrophes, single spaces.
pub fn options(clean: &str) -> Option<Vec<String>> {
    if clean.is_empty() {
        return None;
    }
    let has = |rx: &Re| rx.search(clean).is_some();
    if has(&COVER_IT) || has(&COVERED_IN) {
        return None; // the lid on a pan; a thing covered in something
    }
    if has(&PERSON_ALLOWED) || has(&WORKING_OUT) || has(&BODY_STATE) {
        return None;
    }
    if has(&VISITOR) && (has(&WARD) || has(&SOMEONES_PERSON)) {
        return None; // the ward answers this one, not a page
    }
    let asking = has(&QUESTION);
    if OPENS_IN_THE_PAST.match_(clean).is_some() {
        return None; // what was allowed at something that has already happened
    }
    if asking && has(&THE_OLD_ONE) && has(&MOVING_THE_OLD) && !has(&NOT_THE_OLD_ONE) {
        return Some(THE_OLD_ONE_POOL.clone()); // the fitter and the council
    }
    if asking && (has(&A_PLACE) || has(&A_ROOM_FACILITY)) && has(&A_FACILITY) && has(&A_HAS_ASK) && !has(&NOT_A_HAS) {
        return Some(A_PLACE_HAS.clone()); // what the building has is on the page you book on
    }
    if asking && has(&CANCEL_TERMS) {
        return Some(TERMS.clone()); // the fee for calling it off is on the page it was sold on
    }
    if asking && has(&HARM_NOUN) && has(&COVERED) {
        return Some(INSURER.clone()); // the perils are named, so a policy owns the answer
    }
    if asking && has(&TERM_SHAPE) {
        if has(&PRICE_ASK) && !has(&TERM_WORD) {
            return None; // a quote for this person, not a rule printed on a page
        }
        if has(&INSURANCE_WORD) || has(&HARM_NOUN) {
            return Some(INSURER.clone());
        }
        if has(&TICKET_WORD) {
            return Some(OPERATOR.clone());
        }
        if has(&CONTRACT_WORDS) {
            return Some(CONTRACT.clone());
        }
        return Some(TERMS.clone());
    }
    let terms = has(&TERMS_THING) || has(&COVERED_PHRASE);
    if asking && !terms && has(&INCLUDED_ASK) && has(&PRICE_THING) {
        return Some(INCLUDED.clone()); // what the price covers is on the page it was sold on
    }
    if has(&COURT_ARM) {
        return None; // a court or a notice is another reader's arm
    }
    if asking && terms && has(&TERMS_ASK) {
        if has(&INSURANCE_WORD) {
            return Some(INSURER.clone());
        }
        if has(&TICKET_WORD) {
            return Some(OPERATOR.clone());
        }
        return Some(TERMS.clone());
    }
    // "can i leave my gym early": "early" on its own is a question about the terms.
    let early_exit = has(&EARLY) && has(&AGREEMENT_WORD);
    let mut leaving = has(&LEAVING_ASK) || early_exit || (has(&GET_OUT_OF) && has(&AGREEMENT_WORD));
    if leaving && has(&LEAVE_EARLY) && has(&SHIFT_WORD) {
        leaving = false; // the rota, not the agreement
    }
    if leaving && !has(&NOT_AN_AGREEMENT) {
        return Some(CONTRACT.clone()); // the notice and the fee are in the terms
    }
    if has(&COUNT_AS) && has(&ALLOWANCE_WORD) {
        return Some(VENUE.clone());
    }
    if asking && has(&NEEDED_THING) && has(&NEEDED_ASK) {
        return Some(ISSUER.clone());
    }
    let taking = !has(&NAMED_OBJECT) && (has(&TAKE_ON) || (has(&TAKE_TO) && has(&VENUE_WORDS)));
    if has(&ALLOWED_ASK) || taking {
        if has(&LOCAL_WORDS) {
            return Some(LOCAL.clone()); // the council writes this one
        }
        if has(&LAW_NAMED) && !(has(&CARRYING) && has(&TO_INSIDE)) {
            return None; // the law decides this, not whoever runs the place
        }
        if has(&VENUE_WORDS) {
            return Some(VENUE.clone());
        }
        if has(&CONTRACT_WORDS) {
            return Some(CONTRACT.clone());
        }
        return Some(ALLOWED.clone());
    }
    // Every way of asking about a local service.
    if has(&ASKING_SHAPE) && !has(&NOT_A_SERVICE) && (has(&SERVICE_WORDS) || (has(&NOISE) && has(&COMPLAINT))) {
        return Some(LOCAL_SERVICE.clone());
    }
    None
}
