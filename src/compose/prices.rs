//! Port of `bot/compose/prices.py`: an ask for a price (or for a price to be judged, or whether a
//! budget is enough) gets the honest note about where a figure comes from.

use crate::data;
use crate::py::re::{escape, Re};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.prices.", stringify!($name))));)*
    };
}
regexes!(
    PRICE_ASK, PRICE_ASK_HEADLESS, PRICE_JUDGEMENT, A_FAIRNESS_JUDGEMENT, MONEY_JUDGEMENT,
    A_MONEY_CUE, FOOD_SAID_BEFORE, A_PRICE_WORD, NOT_A_PRICE, BALANCE_ASK, BALANCE_CARRIER,
    A_BUDGET_ASK, WORTH_AN_AMOUNT, WORTH_IT, A_BARE_BUDGET_JUDGEMENT, A_MONEY_NEARBY,
);
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.prices.", stringify!($name))));)*
    };
}
pools!(PRICE_OF_A_JOB, A_JOB_JUDGED, PRICE_OF_A_THING, A_BUDGET_IS_THEIR_OWN_SUM);
/// A frozenset: the order only decides which word is found first, never whether one is.
static JOB_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut v: Vec<String> = data::set("bot.compose.prices.JOB_WORDS").into_iter().collect();
    v.sort();
    v
});
/// `re`'s own cache, for the patterns `_bounded` builds.
static BOUNDED: LazyLock<Mutex<HashMap<String, Arc<Re>>>> = LazyLock::new(Default::default);

/// Python: bot/compose/prices.py:186 `_a_balance_ask(clean)`
///
/// True when the line asks what is already on a card or in an account, not what one costs.
pub fn _a_balance_ask(clean: &str) -> bool {
    BALANCE_ASK.is_match(clean) && BALANCE_CARRIER.is_match(clean)
}

/// Python: bot/compose/prices.py:259 `_bounded(phrase)`: the phrase as whole words (only this
/// file calls it).
pub fn _bounded(phrase: &str) -> Arc<Re> {
    let mut cache = BOUNDED.lock().unwrap();
    cache
        .entry(phrase.to_string())
        .or_insert_with(|| Arc::new(Re::new(&format!("(?<![a-z0-9]){}(?![a-z0-9])", escape(phrase)), 0)))
        .clone()
}

/// Python: bot/compose/prices.py:263 `_has(clean, phrases)`
pub fn _has(clean: &str, phrases: &[String]) -> bool {
    phrases.iter().any(|phrase| _bounded(phrase).is_match(clean))
}

/// Python: bot/compose/prices.py:267 `the_price_reply(clean, phrases, before, options)`
///
/// The honest note for a price, or None when the line is not asking for one. `before` is the
/// person's previous line (always a str), `options` the live row's own pool.
pub fn the_price_reply(clean: &str, phrases: &[String], before: &str, options: &[String]) -> Option<Vec<String>> {
    if NOT_A_PRICE.is_match(clean) || _a_balance_ask(clean) {
        return None;
    }
    if !before.is_empty() && FOOD_SAID_BEFORE.is_match(before) && !A_PRICE_WORD.is_match(clean) {
        return None;
    }
    if A_BUDGET_ASK.is_match(clean) || (A_BARE_BUDGET_JUDGEMENT.is_match(clean) && A_MONEY_NEARBY.is_match(before)) {
        return Some(A_BUDGET_IS_THEIR_OWN_SUM.clone());
    }
    let judged = MONEY_JUDGEMENT.is_match(clean) || PRICE_JUDGEMENT.is_match(clean) || A_FAIRNESS_JUDGEMENT.is_match(clean);
    let worth = WORTH_AN_AMOUNT.is_match(clean)
        || (WORTH_IT.is_match(clean) && (A_MONEY_NEARBY.is_match(clean) || A_MONEY_NEARBY.is_match(before)));
    let asked = _has(clean, phrases)
        || PRICE_ASK.is_match(clean)
        || worth
        || PRICE_ASK_HEADLESS.match_(clean).is_some()
        || (judged && (A_MONEY_CUE.is_match(clean) || A_MONEY_CUE.is_match(before) || MONEY_JUDGEMENT.is_match(clean)));
    if !asked {
        return None;
    }
    if _has(clean, &JOB_WORDS) || (judged && _has(before, &JOB_WORDS)) {
        return Some(if judged { A_JOB_JUDGED.clone() } else { PRICE_OF_A_JOB.clone() });
    }
    if judged || worth {
        return Some(PRICE_OF_A_THING.clone());
    }
    Some(options.to_vec())
}
