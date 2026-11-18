//! Helpers `travel.rs` needs from code other workstreams own: the plan the store keeps
//! (`bot.memory.when`), the trip/lives-there reading of a line (`bot.memory.topics`), the border
//! reading of a line (`bot.knowledge.geography`, D1) and the pinned clock. Each is marked so the
//! integrator can swap in the owner's port.

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

/// `memory.get("user")` when it is a dict.
pub fn user_dict(memory: &Memory) -> Option<&Value> {
    match memory.extra.get("user") {
        Some(v @ Value::Dict(_)) => Some(v),
        _ => None,
    }
}

/// The text a byte offset starts, `n` characters long (Python's `s[i:i + n]` for a match end).
pub fn char_window(s: &str, byte: usize, n: usize) -> &str {
    let rest = &s[byte..];
    let end = rest.char_indices().nth(n).map(|(b, _)| b).unwrap_or(rest.len());
    &rest[..end]
}

// ---------------------------------------------------------------------------------------------
// bot/memory/when.py

// DUP(bot.memory.when::the_plan)
/// The newest plan the store keeps, in the person's own words, or None.
pub fn the_plan(memory: &Memory) -> Option<String> {
    let known = user_dict(memory)?;
    if let Some(Value::Str(plan)) = known.get("plan") {
        if !text::strip(plan).is_empty() {
            return Some(text::squash(plan));
        }
    }
    let plans: Vec<&str> = known
        .get("plans")
        .map(|p| p.items().iter().filter_map(|x| x.as_str()).filter(|s| !text::strip(s).is_empty()).collect())
        .unwrap_or_default();
    plans.last().map(|p| text::squash(p))
}

// ---------------------------------------------------------------------------------------------
// bot/memory/topics.py

// DUP(bot.memory.topics::LIVES_THERE)
pub static LIVES_THERE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.topics.LIVES_THERE"));
static TRIP_FRAME: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.topics.TRIP_FRAME"));

// DUP(bot.memory.topics::going_there)
/// Whether the topic says the person is going to the place, not living in it.
pub fn going_there(place: &str, topic: &str) -> bool {
    let said = topic.to_lowercase();
    for m in LIVES_THERE.finditer(&said) {
        if char_window(&said, m.end(), 40).contains(place) {
            return false;
        }
    }
    TRIP_FRAME.is_match(&said)
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/geography.py (D1)

static G_CROSSING_WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.geography._CROSSING_WORD"));
static G_DIRECTION: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.geography._DIRECTION"));
static G_DESTINATION: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.geography._DESTINATION"));
static G_ARTICLE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.geography._ARTICLE"));
static G_NOT_A_PLACE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.geography._NOT_A_PLACE"));
static G_EVERYDAY_PLACE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.geography._EVERYDAY_PLACE"));
static G_BRITAIN: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.geography._BRITAIN"));

// DUP(bot.knowledge.geography::_bare_place)
/// The place words after a direction, without the article in front of them.
pub fn _bare_place(phrase: &str) -> String {
    let mut phrase = text::strip_chars(text::strip(phrase), ".,!?;:").to_string();
    loop {
        let shorter = G_ARTICLE.subn("", &phrase, 1);
        if shorter == phrase {
            return phrase;
        }
        phrase = shorter;
    }
}

// DUP(bot.knowledge.geography::directed_place)
/// The place a line carries something towards, in its own words, or None.
pub fn directed_place(line: &str) -> Option<String> {
    for m in G_DIRECTION.finditer(line) {
        let place = _bare_place(m.str(1));
        if !place.is_empty() && !G_NOT_A_PLACE.iter().any(|w| *w == place) {
            return Some(place);
        }
    }
    None
}

// DUP(bot.knowledge.geography::_is_everyday)
/// True when the place words name a room, a building or a do rather than a country.
pub fn _is_everyday(place: &str) -> bool {
    let place = _bare_place(place);
    G_EVERYDAY_PLACE.iter().any(|name| {
        place == *name || place.starts_with(&format!("{name} ")) || place.ends_with(&format!(" {name}"))
    })
}

// DUP(bot.knowledge.geography::_is_a_place)
fn _is_a_place(line: &str) -> Option<String> {
    let place = directed_place(line)?;
    if _is_everyday(&place) {
        return None;
    }
    Some(place)
}

// DUP(bot.knowledge.geography::crosses_a_border)
/// True when the line carries something across a border rather than to a do.
pub fn crosses_a_border(line: &str) -> bool {
    let line = line.to_lowercase();
    if G_CROSSING_WORD.is_match(&line) {
        return true;
    }
    _is_a_place(&line).is_some()
}

// DUP(bot.knowledge.geography::destination_place)
/// The place a line travels into or to, in its own words, or None.
pub fn destination_place(line: &str) -> Option<String> {
    let low = line.to_lowercase();
    for m in G_DESTINATION.finditer(&low) {
        let place = _bare_place(m.str(1));
        if !place.is_empty() && !G_NOT_A_PLACE.iter().any(|w| *w == place) {
            return Some(place);
        }
    }
    None
}

// DUP(bot.knowledge.geography::carries_it_out_of_britain)
/// True when the place travelled to is named and is not Britain.
pub fn carries_it_out_of_britain(line: &str) -> bool {
    let Some(place) = destination_place(line) else { return false };
    if _is_everyday(&place) {
        return false;
    }
    !G_BRITAIN.iter().any(|b| *b == place)
}

// ---------------------------------------------------------------------------------------------
// The clock

/// A UTC moment, to the minute.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Moment {
    pub year: i64,
    pub month: i64,
    pub day: i64,
    pub hour: i64,
    pub minute: i64,
}

// NEEDS(runtime clock: datetime.datetime.now(datetime.timezone.utc))
/// `datetime.now(timezone.utc)`: pinned to 2026-11-17 12:00 UTC, as in every oracle run.
pub fn now_utc() -> Moment {
    Moment { year: 2026, month: 11, day: 17, hour: 12, minute: 0 }
}

/// Days since 1970-01-01 of a civil date (proleptic Gregorian).
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// The civil date of a day count since 1970-01-01.
pub fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Python's `date.weekday()`: Monday is 0.
pub fn weekday(days: i64) -> i64 {
    // 1970-01-01 was a Thursday (3).
    (days + 3).rem_euclid(7)
}

impl Moment {
    /// The moment `minutes` later (or earlier).
    pub fn plus_minutes(self, minutes: i64) -> Moment {
        let total = days_from_civil(self.year, self.month, self.day) * 1440 + self.hour * 60 + self.minute + minutes;
        let days = total.div_euclid(1440);
        let rest = total.rem_euclid(1440);
        let (year, month, day) = civil_from_days(days);
        Moment { year, month, day, hour: rest / 60, minute: rest % 60 }
    }
}
