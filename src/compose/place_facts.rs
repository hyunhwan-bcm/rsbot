//! Port of `bot/compose/place_facts.py`: a fact that belongs to one place ("do they allow
//! dogs", "is there parking", "is the market on on sunday", "is it free") gets the honest line
//! about where that place keeps it.

use crate::data;
use crate::py::re::{escape, Re};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.place_facts.", stringify!($name))));)*
    };
}
regexes!(
    PLACE_ASKS, PLACE_ASKS_OF_ITS_OWN, PLACE_PERMISSION, PLACE_LETS_ANIMALS, BARE_PLACE_SUBJECT,
    A_BRING_ASK, A_DESTINATION, A_HOME_OF_THEIR_OWN, PLACE_IS_THERE, A_DAY, ON_A_DAY, A_FREE_ASK,
    A_FIXTURE, A_RUNS_TO_A_CALENDAR, A_STANDING_DAY,
);
macro_rules! phrases {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.place_facts.", stringify!($name))));)*
    };
}
phrases!(
    PLACE_OF_ITS_OWN, PLACE_WITH_A_VENUE, PLACE_ASKS_WITH_NO_SUBJECT, PLACE_ANIMALS, PLACE_VENUES,
    PLACE_LINES, A_CLOCK_ASK, A_CROWD_ASK, A_SWITCHED_ON_THING, PLACE_EVENTS, PLACE_ON_A_DAY_LINES,
    PLACE_FREE_LINES,
);
/// A frozenset: `_has` reads it in its iteration order, which only decides which phrase is
/// found first, never whether one is.
static PLACE_FACILITIES: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut v: Vec<String> = data::set("bot.compose.place_facts.PLACE_FACILITIES").into_iter().collect();
    v.sort();
    v
});
/// `re`'s own cache, for the patterns `_bounded` builds.
static BOUNDED: LazyLock<Mutex<HashMap<String, Arc<Re>>>> = LazyLock::new(Default::default);

/// Python: bot/compose/place_facts.py:154 `_bounded(phrase)`: the phrase as whole words (only
/// this file calls it).
pub fn _bounded(phrase: &str) -> Arc<Re> {
    let mut cache = BOUNDED.lock().unwrap();
    cache
        .entry(phrase.to_string())
        .or_insert_with(|| Arc::new(Re::new(&format!("(?<![a-z0-9]){}(?![a-z0-9])", escape(phrase)), 0)))
        .clone()
}

/// Python: bot/compose/place_facts.py:158 `_has(clean, phrases)`
///
/// True when one of these phrases is in the line as whole words.
pub fn _has(clean: &str, phrases: &[String]) -> bool {
    phrases.iter().any(|phrase| _bounded(phrase).is_match(clean))
}

/// Python: bot/compose/place_facts.py:166 `_a_place_they_are_bringing_somebody_to(clean)`
///
/// True when the line asks to bring an animal or a child to a place it names.
pub fn _a_place_they_are_bringing_somebody_to(clean: &str) -> bool {
    if !A_BRING_ASK.is_match(clean) {
        return false;
    }
    if A_DESTINATION.is_match(clean) && !_has(clean, &PLACE_VENUES) {
        return false; // "can i take the kids to school" is the school run, not a venue's rule
    }
    true
}

/// Python: bot/compose/place_facts.py:303 `the_line(clean)`
///
/// The honest line for a fact that belongs to one place, or None.
pub fn the_line(clean: &str) -> Option<Vec<String>> {
    let venue = || _has(clean, &PLACE_VENUES);
    if A_HOME_OF_THEIR_OWN.is_match(clean) && !venue() {
        return None;
    }
    if PLACE_IS_THERE.is_match(clean) && _has(clean, &PLACE_FACILITIES) {
        return None;
    }
    if A_FIXTURE.is_match(clean) && !A_RUNS_TO_A_CALENDAR.is_match(clean) {
        return None;
    }
    let of_its_own = PLACE_ASKS_OF_ITS_OWN.is_match(clean);
    let bringing = _a_place_they_are_bringing_somebody_to(clean);
    let lets_animals = PLACE_LETS_ANIMALS.is_match(clean) && _has(clean, &PLACE_ANIMALS);
    if !(of_its_own || bringing || lets_animals || PLACE_ASKS.is_match(clean)) {
        return _a_day_ask(clean);
    }
    let a_place_fact = of_its_own
        || bringing
        || lets_animals
        || _has(clean, &PLACE_OF_ITS_OWN)
        || (venue() && _has(clean, &PLACE_WITH_A_VENUE))
        || (BARE_PLACE_SUBJECT.is_match(clean) && _has(clean, &PLACE_ASKS_WITH_NO_SUBJECT))
        || (PLACE_IS_THERE.is_match(clean)
            && _has(clean, &PLACE_ASKS_WITH_NO_SUBJECT)
            && !(A_HOME_OF_THEIR_OWN.is_match(clean) && !venue()))
        || (BARE_PLACE_SUBJECT.is_match(clean) && _has(clean, &PLACE_ANIMALS) && PLACE_PERMISSION.is_match(clean));
    if !a_place_fact {
        return _a_day_ask(clean);
    }
    Some(PLACE_LINES.clone())
}

/// Python: bot/compose/place_facts.py:330 `_a_day_ask(clean)`
///
/// The lines for a place or an event asked about on a named day (or whether it is free), or None.
pub fn _a_day_ask(clean: &str) -> Option<Vec<String>> {
    if _has(clean, &A_CLOCK_ASK) {
        return None;
    }
    if A_FREE_ASK.is_match(clean) && (_has(clean, &PLACE_VENUES) || _has(clean, &PLACE_EVENTS) || A_DAY.is_match(clean)) {
        if !_has(clean, &A_SWITCHED_ON_THING) {
            return Some(PLACE_FREE_LINES.clone());
        }
    }
    if _has(clean, &A_CROWD_ASK) {
        return None;
    }
    if A_STANDING_DAY.is_match(clean) {
        return None;
    }
    if !(ON_A_DAY.is_match(clean) && A_DAY.is_match(clean)) {
        return None;
    }
    if _has(clean, &A_SWITCHED_ON_THING) {
        return None;
    }
    if !(_has(clean, &PLACE_VENUES) || _has(clean, &PLACE_EVENTS)) {
        return None;
    }
    Some(PLACE_ON_A_DAY_LINES.clone())
}
