//! Port of `bot/compose/the_rule_for_it.py`: a line about to be shrugged at that is one of a
//! few kinds of ask (is it still good, how much do I need, what does it cost, how long does it
//! take) gets the rule the answer is worked out by.

use crate::data;
use crate::py::re::Re;
use crate::py::text;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.the_rule_for_it.", stringify!($name))));)*
    };
}
regexes!(
    PRICE_ASK, PRICE_VERDICT, A_DEAR_AS_A_NAME, A_FARE_OR_A_TICKET, A_CAB_OR_A_LIFT, A_AN_OCCASION,
    NOT_ABOUT_A_PRICE, AMOUNT_ASK, FEEDING, MATERIAL, MONEY_ON_A_JOB, GONE_OFF_ASK, STILL_ASK,
    KEEP_ASK, OFF_ASK, NOT_A_KEEPING_QUESTION, KEEPING_HAS_NO_DATE, ABOUT_A_PERSON_OR_AN_ANIMAL,
    FOOD_OR_PACK, TIME_ASK, TIME_NOT_THIS, A_COOKING_TIME, A_FOOD,
);
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.the_rule_for_it.", stringify!($name))));)*
    };
}
pools!(
    PRICE_RULE, FARE_PRICE_RULE, CAB_FARE_RULE, PRICE_VERDICT_RULE, OCCASION_PRICE_RULE,
    AMOUNT_FEEDING_RULE, AMOUNT_COVER_RULE, STILL_GOOD_RULE, TIME_RULE, COOKING_TIME_RULE,
);

/// Python: bot/compose/the_rule_for_it.py:337 `the_rule_for_it(clean, repaired=None)`
///
/// The pool for the kind of ask this line is, or None. Read on the typed line, then the repaired
/// one (`repaired` None is "" here; Python skips both alike).
pub fn the_rule_for_it(clean: &str, repaired: &str) -> Option<Vec<String>> {
    for line in [clean, repaired] {
        let said = text::squash(&text::lower(line));
        if said.is_empty() {
            continue;
        }
        let said = said.as_str();
        if NOT_A_KEEPING_QUESTION.is_match(said) {
            return None;
        }
        if ABOUT_A_PERSON_OR_AN_ANIMAL.is_match(said) {
            return None;
        }
        if !KEEPING_HAS_NO_DATE.is_match(said) {
            if KEEP_ASK.is_match(said) || GONE_OFF_ASK.is_match(said) {
                return Some(STILL_GOOD_RULE.clone());
            }
            if STILL_ASK.is_match(said) && FOOD_OR_PACK.is_match(said) {
                return Some(STILL_GOOD_RULE.clone());
            }
            if OFF_ASK.is_match(said) && FOOD_OR_PACK.is_match(said) {
                return Some(STILL_GOOD_RULE.clone());
            }
        }
        if AMOUNT_ASK.is_match(said) && !MONEY_ON_A_JOB.is_match(said) {
            if MATERIAL.is_match(said) && !FEEDING.is_match(said) {
                return Some(AMOUNT_COVER_RULE.clone());
            }
            if FEEDING.is_match(said) {
                return Some(AMOUNT_FEEDING_RULE.clone());
            }
        }
        if PRICE_VERDICT.is_match(said) && !NOT_ABOUT_A_PRICE.is_match(said) && !A_DEAR_AS_A_NAME.is_match(said) {
            if A_CAB_OR_A_LIFT.is_match(said) {
                return Some(CAB_FARE_RULE.clone()); // the meter, not the day
            }
            if A_AN_OCCASION.is_match(said) {
                return Some(OCCASION_PRICE_RULE.clone()); // per head, by the day
            }
            if A_FARE_OR_A_TICKET.is_match(said) {
                return Some(FARE_PRICE_RULE.clone());
            }
            return Some(PRICE_VERDICT_RULE.clone()); // a verdict names no job
        }
        if PRICE_ASK.is_match(said) && !NOT_ABOUT_A_PRICE.is_match(said) {
            if A_CAB_OR_A_LIFT.is_match(said) {
                return Some(CAB_FARE_RULE.clone());
            }
            if A_AN_OCCASION.is_match(said) {
                return Some(OCCASION_PRICE_RULE.clone());
            }
            return Some(PRICE_RULE.clone());
        }
        if TIME_ASK.is_match(said) && !TIME_NOT_THIS.is_match(said) {
            if A_COOKING_TIME.is_match(said) && A_FOOD.is_match(said) {
                return Some(COOKING_TIME_RULE.clone()); // a joint and an oven
            }
            return Some(TIME_RULE.clone());
        }
    }
    None
}
