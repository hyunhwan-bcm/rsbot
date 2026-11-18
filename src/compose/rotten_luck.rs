//! Port of `bot/compose/rotten_luck.py`: a rotten thing (damaged, taken, refused, a fall, a near
//! miss, a wait that went on, a meal gone without) answered as rotten where the composer was
//! about to congratulate it or only receive it.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_re_lists, bundle_res, bundle_strs, format_named, inline_re};
use crate::compose::pick;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;

bundle_re_lists!("bot.compose.rotten_luck"; NOT_GOOD);
bundle_res!("bot.compose.rotten_luck"; NOT_GOOD_DESPITE, GLAD_OPENER, BLANK_OR_THIN, GLAD_ANYWHERE, _A_SAID_NO, _A_REFUSED, _MINE,
    _A_NO_STILL_BEING_WEIGHED, _A_REFUSAL_IN_THE_PASSIVE, RESOLVED, _SOMEONE_ELSE, _FALL, NEAR_MISS, FRIGHT, NOT_A_FRIGHT,
    _A_PLAN_THAT_FELL_THROUGH, _A_BARE_NEAR_MISS, _A_BODY_CAN_BE_HURT, _A_THING_WITH_NO_BODY, _FROM_SOMEONE, _A_JOB_NEEDED,
    _NOT_A_TRADE, _A_PERSON_NEEDING, _A_JOB_THAT_IS_SORTED, _PILE_WORDS, _CHORE_THINGS, _TRIED_AND_IT_DID_NOT, _STILL_NO,
    _A_WAIT, _A_LONG_WAIT, _A_REMARK, _A_DISPUTED_CHARGE, _A_MEAL_MISSED, _A_MEAL_ON_PURPOSE, _SOMEBODY_ELSES_MEAL,
    _MINE_OR_OURS, _AN_ASK_OPENING, _SHAKE_IS_THEIRS);
bundle_strs!("bot.compose.rotten_luck"; A_NO_ALREADY, A_NO_FROM, DAMAGED_OR_TAKEN, A_BLOW_FROM_SOMEONE, A_FALL, A_FALL_ELSE,
    A_FRIGHT, A_NEAR_MISS, A_FRIGHT_ELSE, A_NEAR_MISS_ELSE, A_THING_THAT_DID_NOT_WORK, A_WAIT_THAT_WENT_ON,
    A_REMARK_THAT_STUNG, A_BILL_THAT_SAYS_OTHERWISE, A_MEAL_THEY_WENT_WITHOUT, A_CHORE_WAITING, A_CHORE_PILE,
    A_JOB_SOMEBODY_ELSE_DOES, A_LACK_THAT_IS_STILL_GOING);
inline_re!(OPENS_ON_ME, r"^(?:i|we)\b");
inline_re!(SOMEBODY_IN_IT, r"\b(?:she|he|they|my|our|the)\b");
inline_re!(WHO_REFUSED, r"\b((?:my|our|the)\s+[a-z]+|mum|dad)\b");

/// Python: bot/compose/rotten_luck.py:421 `_a_pile_of_chores(clean)`
///
/// Whether the line named more than one job, or said the whole of it is a state.
pub fn _a_pile_of_chores(clean: &str) -> bool {
    if _PILE_WORDS.search(clean).is_some() {
        return true;
    }
    _CHORE_THINGS.findall_str(clean).into_iter().collect::<HashSet<_>>().len() > 1
}

/// Python: bot/compose/rotten_luck.py:527 `reads_as_stuck_with_it(clean)`
///
/// True when the line reports an attempt that failed, a wait, a remark that stung or a bill.
pub fn reads_as_stuck_with_it(clean: &str) -> bool {
    if clean.is_empty() || clean.contains('?') || NOT_GOOD_DESPITE.search(clean).is_some() {
        return false;
    }
    if _AN_ASK_OPENING.match_(clean).is_some() {
        return false;
    }
    if _A_DISPUTED_CHARGE.search(clean).is_some() || _A_REMARK.search(clean).is_some() || _TRIED_AND_IT_DID_NOT.search(clean).is_some() {
        return true;
    }
    _A_WAIT.search(clean).is_some() && _A_LONG_WAIT.search(clean).is_some()
}

/// Python: bot/compose/rotten_luck.py:546 `reads_as_not_good(clean)`
///
/// True when the line is a thing of theirs that has been damaged, taken, refused or fallen.
pub fn reads_as_not_good(clean: &str) -> bool {
    if clean.is_empty() || NOT_GOOD_DESPITE.search(clean).is_some() {
        return false;
    }
    reads_the_shake(clean) || NOT_GOOD.iter().any(|p| p.search(clean).is_some())
}

/// Python: bot/compose/rotten_luck.py:555 `reads_the_shake(clean)`
///
/// True when the line is a near miss or a fright - nothing landed, they are still shaken.
pub fn reads_the_shake(clean: &str) -> bool {
    if clean.is_empty() || NOT_A_FRIGHT.search(clean).is_some() || _A_PLAN_THAT_FELL_THROUGH.search(clean).is_some() {
        return false;
    }
    if _A_BARE_NEAR_MISS.search(clean).is_some()
        && _A_THING_WITH_NO_BODY.search(clean).is_some()
        && _A_BODY_CAN_BE_HURT.search(clean).is_none()
    {
        return false;
    }
    NEAR_MISS.search(clean).is_some() || FRIGHT.search(clean).is_some()
}

/// Python: bot/compose/rotten_luck.py:574 `reads_the_shake_of_someone_else(clean)`
pub fn reads_the_shake_of_someone_else(clean: &str) -> bool {
    reads_the_shake(clean) && (_SOMEONE_ELSE.search(clean).is_some() || _SHAKE_IS_THEIRS.match_(clean).is_some())
}

/// Python: bot/compose/rotten_luck.py:581 `reads_as_glad(reply)`
///
/// True when the reply congratulates the news, or is a receipt of it with nothing after.
pub fn reads_as_glad(reply: &str) -> bool {
    if reply.is_empty() {
        return false;
    }
    let line = text::strip(reply).to_lowercase();
    GLAD_OPENER.match_(&line).is_some() || GLAD_ANYWHERE.search(&line).is_some() || BLANK_OR_THIN.match_(&line).is_some()
}

/// Python: bot/compose/rotten_luck.py:590 `reads_as_a_meal_missed(clean)`
///
/// True when the line says they went without a meal of their own.
pub fn reads_as_a_meal_missed(clean: &str) -> bool {
    if clean.is_empty() || clean.contains('?') || _A_MEAL_ON_PURPOSE.search(clean).is_some() || _A_MEAL_MISSED.search(clean).is_none() {
        return false;
    }
    _MINE_OR_OURS.search(clean).is_some() || _SOMEBODY_ELSES_MEAL.search(clean).is_none()
}

/// Python: bot/compose/rotten_luck.py:607 `reads_as_a_no_they_had(clean)`
///
/// True when the line reports somebody refusing the person something.
pub fn reads_as_a_no_they_had(clean: &str) -> bool {
    if clean.is_empty() || clean.contains('?') || text::split_ref(clean).len() > 14 {
        return false;
    }
    if _A_NO_STILL_BEING_WEIGHED.search(clean).is_some() {
        return false;
    }
    if OPENS_ON_ME.match_(clean).is_some() && SOMEBODY_IN_IT.search(clean).is_none() {
        return false;
    }
    _A_SAID_NO.search(clean).is_some() || _A_REFUSED.search(clean).is_some()
}

/// Python: bot/compose/rotten_luck.py:626 `_who_said_no(clean, memory)`
///
/// Who refused, said the way the person would hear it back: "your boss", "the council".
pub fn _who_said_no(clean: &str, memory: &Memory) -> String {
    if _A_REFUSAL_IN_THE_PASSIVE.search(clean).is_some() {
        // In "my claim was refused" the "my" is the thing refused, not the person who refused it.
        return String::new();
    }
    let mut who = WHO_REFUSED.search(clean).map(|m| m.str(1).to_string()).unwrap_or_default();
    if who.is_empty() {
        if let Some(lean) = memory.understood.as_ref().and_then(|u| u.leans_on.as_ref()) {
            let subject = text::strip(lean.subject.as_deref().unwrap_or("")).to_lowercase();
            if !subject.is_empty() && lean.subject_type.as_deref() == Some("person") {
                who = subject;
            }
        }
    }
    if who.is_empty() {
        return String::new();
    }
    text::squash(&_MINE.subn("your", &who, 1))
}

fn has(rx: &Re, s: &str) -> bool {
    rx.search(s).is_some()
}

/// Python: bot/compose/rotten_luck.py:645 `answer_for(text, memory=None, reply=None)`
///
/// A rotten thing answered as rotten, or None. Called with the reply the composer chose.
pub fn answer_for(text_: &str, memory: &Memory, reply: &str) -> Option<String> {
    if !reads_as_glad(reply) {
        return None;
    }
    let understood_clean = memory.understood.as_ref().map(|u| u.clean.as_str()).unwrap_or("");
    let clean = text::squash(&if understood_clean.is_empty() { text_ } else { understood_clean }.to_lowercase());
    if reads_the_shake(&clean) {
        // The shake is asked about before the resolved check.
        let mine: &Vec<String> = if reads_the_shake_of_someone_else(&clean) {
            if has(&NEAR_MISS, &clean) { &A_NEAR_MISS_ELSE } else { &A_FRIGHT_ELSE }
        } else if has(&NEAR_MISS, &clean) {
            &A_NEAR_MISS
        } else {
            &A_FRIGHT
        };
        return Some(pick(mine, memory, &clean));
    }
    if has(&RESOLVED, &clean) {
        return None;
    }
    if reads_as_a_meal_missed(&clean) {
        return Some(pick(&A_MEAL_THEY_WENT_WITHOUT, memory, &clean));
    }
    if has(&_A_JOB_NEEDED, &clean) && !has(&_A_PERSON_NEEDING, &clean) && !has(&_A_JOB_THAT_IS_SORTED, &clean) {
        if has(&_NOT_A_TRADE, &clean) {
            // Work of their own with no trade behind it: one job or a pile of them.
            let pool: &Vec<String> = if _a_pile_of_chores(&clean) { &A_CHORE_PILE } else { &A_CHORE_WAITING };
            return Some(pick(pool, memory, &clean));
        }
        // A job that needs doing: what is written down before anybody starts sets the bill.
        return Some(pick(&A_JOB_SOMEBODY_ELSE_DOES, memory, &clean));
    }
    if has(&_STILL_NO, &clean) && !has(&_A_WAIT, &clean) {
        // A lack that is still going on, and not an attempt at a fix.
        return Some(pick(&A_LACK_THAT_IS_STILL_GOING, memory, &clean));
    }
    if reads_as_stuck_with_it(&clean) {
        let pool: &Vec<String> = if has(&_A_DISPUTED_CHARGE, &clean) {
            &A_BILL_THAT_SAYS_OTHERWISE
        } else if has(&_A_REMARK, &clean) {
            &A_REMARK_THAT_STUNG
        } else if has(&_A_WAIT, &clean) && has(&_A_LONG_WAIT, &clean) {
            &A_WAIT_THAT_WENT_ON
        } else {
            &A_THING_THAT_DID_NOT_WORK
        };
        return Some(pick(pool, memory, &clean));
    }
    if !reads_as_not_good(&clean) {
        // A refusal is asked last of all the readings here.
        if !reads_as_a_no_they_had(&clean) {
            return None;
        }
        let who = _who_said_no(&clean, memory);
        let pool: &Vec<String> = if who.is_empty() { &A_NO_ALREADY } else { &A_NO_FROM };
        let line = pick(pool, memory, &format!("{clean}|{who}"));
        return Some(if who.is_empty() { line } else { format_named(&line, &[("who", &who)]) });
    }
    let pool: &Vec<String> = if has(&_FALL, &clean) {
        if has(&_SOMEONE_ELSE, &clean) { &A_FALL_ELSE } else { &A_FALL }
    } else if has(&_FROM_SOMEONE, &clean) {
        &A_BLOW_FROM_SOMEONE
    } else {
        &DAMAGED_OR_TAKEN
    };
    Some(pick(pool, memory, &clean))
}
