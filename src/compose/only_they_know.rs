//! Port of `bot/compose/only_they_know.py`: a judgement put to the bot ("was i too harsh"), a
//! fact about one of their own people only they can know ("does my mum like golf"), or how one
//! of them feels about them - answered as that, in place of a hedge.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_strs, bundle_vals, inline_re};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;

bundle_res!("bot.compose.only_they_know"; JUDGEMENT_WORDS, PERMISSION_ASKED, ASKS_MY_VIEW, A_VERDICT_ASKED_FOR, A_MILD_GRADE,
    SOMEONE_ELSE_JUDGED, _THE_TWO_OF_US, _THING_OF_THEIRS, _ABOUT_THE_ASKER, AM_I_TOO_OLD, A_THING_IT_IS_TOO_OLD_FOR,
    AGE_IS_THE_RULE, A_NUMBER_IN_THE_LINE, A_GIFT_OR_TASTE_ASKED, A_ROW_IN_THE_LINE, NOT_A_FACT_ASK, A_BODY_OR_ILLNESS,
    A_FACT_ABOUT_THEIR_PERSON, A_WH_ABOUT_THEIR_PERSON, A_STATE_ABOUT_THEIR_PERSON, A_PLACE_ABOUT_THEIR_PERSON,
    A_FIGURE_ABOUT_THEIR_PERSON, THEIR_PERSON_WORD, THEIR_PERSON_THROUGH_ANOTHER, ABOUT_ME_ASKED, THEIR_FEELING_TOWARD_ME,
    WHY_TOWARD_ME, WHY_WITHDRAWN, SOMEBODY_ELSES_PERSON, AM_I_TOO_MUCH, A_SUSPICION, NOT_A_FEELING_ASK);
bundle_strs!("bot.compose.only_they_know"; TOO_OLD_FOR_IT, JUDGE_SOMEONE_ELSE, JUDGE_SOMEONE_AT_AN_AGE, JUDGE_THEIR_TASTE,
    JUDGE_MYSELF, JUDGE_MYSELF_NO_ROW, JUDGE_THE_TWO_OF_US, THEIR_PERSON_WH_LINES, THEIR_PERSON_FIGURE_LINES,
    THEIR_PERSON_LINES, THEIR_FEELINGS_LINES, THEY_FEEL_ABOUT_ME, WHY_THEY_DID_THAT, A_SUSPICION_LINES);
bundle_vals!("bot.compose.only_they_know"; _PERSON_SUBJECT, _PERSON_OBJECT, _PERSON_FORMS);
inline_re!(SHOULD, r"\bshould\b");
inline_re!(WHAT_I_DID, r"\bwhat\s+i\s+(?:said|did|wrote|sent|texted|told)\b");
inline_re!(A_PRONOUN, r"\b(?:he|she|they|him|her)\b");
inline_re!(A_WORD, r"\b[a-z']+\b");

/// `line.replace("{who}", ...).replace("{them}", ...).replace("{poss}", ...)` over a pool.
fn fill(pool: &[String], who: &str, them: &str, poss: &str) -> Vec<String> {
    pool.iter().map(|l| l.replace("{who}", who).replace("{them}", them).replace("{poss}", poss)).collect()
}

/// Python: bot/compose/only_they_know.py:207 `_a_judgement_i_am_asked_for(clean, words)`
///
/// The pool for a person asking the bot's judgement, or None when it is another question.
pub fn _a_judgement_i_am_asked_for(clean: &str, words: &[String]) -> Option<Vec<String>> {
    let _ = words;
    let has = |rx: &Re| rx.search(clean).is_some();
    let too_old = AM_I_TOO_OLD.match_(clean).is_some();
    if !(has(&ASKS_MY_VIEW) || A_VERDICT_ASKED_FOR.match_(clean).is_some() || has(&A_MILD_GRADE) || too_old) {
        return None;
    }
    // "should" asks what to do, and a permission whether a thing is allowed.
    if has(&SHOULD) || has(&PERMISSION_ASKED) {
        return None;
    }
    // The judgement word makes an ask a verdict rather than a wish to talk about somebody.
    if !(has(&JUDGEMENT_WORDS) || too_old) {
        return None;
    }
    if too_old && !has(&SOMEONE_ELSE_JUDGED) {
        // Only the ask that names what it is too old for comes here.
        if has(&A_THING_IT_IS_TOO_OLD_FOR) && !has(&AGE_IS_THE_RULE) {
            return Some(TOO_OLD_FOR_IT.clone());
        }
        return None;
    }
    // Their own conduct with somebody named in the line.
    if has(&WHAT_I_DID) {
        return Some(JUDGE_MYSELF.clone());
    }
    if has(&A_GIFT_OR_TASTE_ASKED) && has(&SOMEONE_ELSE_JUDGED) {
        let (subject, them, possessive) = _said_person(clean);
        return Some(fill(&JUDGE_THEIR_TASTE, &subject, &them, &possessive));
    }
    if has(&_THE_TWO_OF_US) && !has(&SOMEONE_ELSE_JUDGED) {
        return Some(JUDGE_THE_TWO_OF_US.clone());
    }
    if has(&SOMEONE_ELSE_JUDGED) {
        return Some(if has(&A_NUMBER_IN_THE_LINE) { JUDGE_SOMEONE_AT_AN_AGE.clone() } else { JUDGE_SOMEONE_ELSE.clone() });
    }
    if has(&_THING_OF_THEIRS) && !has(&_ABOUT_THE_ASKER) {
        return None;
    }
    // A row in the line makes "what did you actually say" the right question.
    Some(if has(&A_ROW_IN_THE_LINE) { JUDGE_MYSELF.clone() } else { JUDGE_MYSELF_NO_ROW.clone() })
}

/// Python: bot/compose/only_they_know.py:470 `_said_person(clean)`
///
/// The three forms of the person the line is about: subject, object, possessive.
pub fn _said_person(clean: &str) -> (String, String, String) {
    if let Some(through) = THEIR_PERSON_THROUGH_ANOTHER.search(clean) {
        // "his wife" reads right in every slot.
        let who = through.as_str().to_string();
        return (who.clone(), who.clone(), format!("{who}'s"));
    }
    if let Some(named) = THEIR_PERSON_WORD.search(clean) {
        let who = format!("your {}", named.str(1));
        return (who.clone(), who.clone(), format!("{who}'s"));
    }
    let said = _feeling_subject(clean);
    match _PERSON_FORMS.get(&said) {
        Some(forms) => {
            let f = forms.strs();
            (f[0].clone(), f[1].clone(), f[2].clone())
        }
        None => ("they".into(), "them".into(), "their".into()),
    }
}

/// Python: bot/compose/only_they_know.py:487 `_a_feeling_toward_me(clean)`
///
/// The pool for how one of their people feels about them, or why they did that; or None.
pub fn _a_feeling_toward_me(clean: &str) -> Option<Vec<String>> {
    let has = |rx: &Re| rx.search(clean).is_some();
    if has(&NOT_A_FEELING_ASK) || has(&SOMEBODY_ELSES_PERSON) {
        return None;
    }
    let withdrawn = has(&WHY_WITHDRAWN) && (has(&THEIR_PERSON_WORD) || has(&A_PRONOUN));
    let (subject, them, possessive) = _said_person(clean);
    if has(&A_SUSPICION) {
        return Some(A_SUSPICION_LINES.clone());
    }
    if has(&WHY_TOWARD_ME) || withdrawn {
        return Some(fill(&WHY_THEY_DID_THAT, &subject, &them, &possessive));
    }
    if has(&AM_I_TOO_MUCH) || has(&THEIR_FEELING_TOWARD_ME) {
        return Some(fill(&THEY_FEEL_ABOUT_ME, &subject, &them, &possessive));
    }
    None
}

/// Python: bot/compose/only_they_know.py:507 `_feeling_subject(clean)`
pub fn _feeling_subject(clean: &str) -> String {
    A_WORD.findall_str(clean).into_iter().find(|w| _PERSON_FORMS.get(w).is_some()).unwrap_or_else(|| "they".to_string())
}

/// Python: bot/compose/only_they_know.py:514 `_a_fact_about_their_person(clean)`
///
/// The pool for a question only the asker can answer about their own people, or None.
pub fn _a_fact_about_their_person(clean: &str) -> Option<Vec<String>> {
    // Where somebody is comes first and needs no body fence.
    let place = A_PLACE_ABOUT_THEIR_PERSON.match_(clean);
    let fact = A_FACT_ABOUT_THEIR_PERSON.match_(clean);
    let state = A_STATE_ABOUT_THEIR_PERSON.match_(clean);
    let figure = A_FIGURE_ABOUT_THEIR_PERSON.match_(clean);
    let asked = place
        .clone()
        .or_else(|| fact.clone())
        .or_else(|| A_WH_ABOUT_THEIR_PERSON.match_(clean))
        .or_else(|| state.clone())
        .or_else(|| figure.clone())?;
    // A body, a dose, an illness, a death, a pet or an ask for advice has its own lane.
    if place.is_none() && (NOT_A_FACT_ASK.search(clean).is_some() || A_BODY_OR_ILLNESS.search(clean).is_some()) {
        return None;
    }
    let wh = !(fact.is_some() || state.is_some());
    let (_who_said, _them_said, possessive) = _said_person(clean);
    // "your mum" reads right where "mum" does not; the bare pronoun lines fall back to "them".
    let said = asked.name_str("who").to_string();
    let named = THEIR_PERSON_WORD.search(clean);
    let (object_word, found) = match named {
        Some(n) => (format!("your {}", n.str(1)), true),
        None => match THEIR_PERSON_THROUGH_ANOTHER.search(clean) {
            Some(t) => (t.as_str().to_string(), true),
            None => (_PERSON_OBJECT.get(&said).map(|v| v.str().to_string()).unwrap_or_else(|| "them".into()), false),
        },
    };
    let subject = if found {
        object_word.clone()
    } else {
        _PERSON_SUBJECT.get(&said).map(|v| v.str().to_string()).unwrap_or_else(|| said.clone())
    };
    if ABOUT_ME_ASKED.search(clean).is_some() {
        return Some(THEIR_FEELINGS_LINES.iter().map(|l| l.replace("{subject}", &subject).replace("{object}", &object_word)).collect());
    }
    let pool: &Vec<String> = if figure.is_some() {
        &THEIR_PERSON_FIGURE_LINES
    } else if wh {
        &THEIR_PERSON_WH_LINES
    } else {
        &THEIR_PERSON_LINES
    };
    Some(
        pool.iter()
            .map(|l| {
                l.replace("{she}", &subject).replace("{who}", &object_word).replace("{them}", &object_word).replace("{poss}", &possessive)
            })
            .collect(),
    )
}

/// Python: bot/compose/only_they_know.py:555 `answer_for(text, memory)`
///
/// A reply for a judgement put to the bot, or a fact about one of their own people; or None. The
/// reading is made on the clean line and on the repaired one.
pub fn answer_for(text_: &str, memory: &Memory) -> Option<String> {
    let understood = memory.understood.as_ref();
    let clean = match understood.map(|u| u.clean.as_str()).filter(|c| !c.is_empty()) {
        Some(c) => c.to_string(),
        None => text::squash(&text_.to_lowercase()),
    };
    let words: Vec<String> = match understood.map(|u| &u.words).filter(|w| !w.is_empty()) {
        Some(w) => w.clone(),
        None => text::split(&clean.replace('\'', "")),
    };
    let repaired = words.join(" ");
    for line in [&clean, &repaired] {
        if line.is_empty() {
            continue;
        }
        let pool = _a_judgement_i_am_asked_for(line, &words)
            .filter(|p| !p.is_empty())
            .or_else(|| _a_fact_about_their_person(line).filter(|p| !p.is_empty()))
            .or_else(|| _a_feeling_toward_me(line).filter(|p| !p.is_empty()));
        if let Some(pool) = pool {
            return crate::compose::fallback_chain::_choose(&pool, memory, &clean, None);
        }
    }
    None
}
