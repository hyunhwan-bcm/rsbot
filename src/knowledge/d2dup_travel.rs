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

// was DUP(bot.memory.when::the_plan): now the real port
pub use crate::memory::when::the_plan;

// ---------------------------------------------------------------------------------------------
// bot/memory/topics.py

// DUP(bot.memory.topics::LIVES_THERE)
pub static LIVES_THERE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.topics.LIVES_THERE"));

// was DUP(bot.memory.topics::going_there): now the real port
pub use crate::memory::topics::going_there;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/geography.py (D1)


// was DUP(bot.knowledge.geography::_bare_place): now the real port
pub use crate::knowledge::geography::_bare_place;

// was DUP(bot.knowledge.geography::directed_place): now the real port
pub use crate::knowledge::geography::directed_place;

// was DUP(bot.knowledge.geography::_is_everyday): now the real port
pub use crate::knowledge::geography::_is_everyday;

// DUP(bot.knowledge.geography::_is_a_place)
fn _is_a_place(line: &str) -> Option<String> {
    let place = directed_place(line)?;
    if _is_everyday(&place) {
        return None;
    }
    Some(place)
}

// was DUP(bot.knowledge.geography::crosses_a_border): now the real port
pub use crate::knowledge::geography::crosses_a_border;

// was DUP(bot.knowledge.geography::destination_place): now the real port
pub use crate::knowledge::geography::destination_place;

// was DUP(bot.knowledge.geography::carries_it_out_of_britain): now the real port
pub use crate::knowledge::geography::carries_it_out_of_britain;

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

/// `datetime.now(timezone.utc)`, from the shared clock (`py::clock`), to the minute.
pub fn now_utc() -> Moment {
    use chrono::{Datelike, Timelike};
    let now = crate::py::clock::now_utc();
    Moment { year: now.year() as i64, month: now.month() as i64, day: now.day() as i64, hour: now.hour() as i64, minute: now.minute() as i64 }
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
