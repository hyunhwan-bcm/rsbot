//! Helpers `money.rs` needs from code other workstreams own, until their ports are merged.
//! Each is marked DUP / DUP-D2 / NEEDS with the Python function it stands for.

use crate::data;
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
// The clock

/// A calendar date (proleptic Gregorian), enough for `datetime.date` arithmetic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Date {
    pub year: i64,
    pub month: u32,
    pub day: u32,
}

impl Date {
    /// Days since 1970-01-01.
    fn ordinal(&self) -> i64 {
        // Howard Hinnant's days_from_civil.
        let y = if self.month <= 2 { self.year - 1 } else { self.year };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = self.month as i64;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + self.day as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    }
    /// `(self - other).days`.
    pub fn days_since(&self, other: &Date) -> i64 {
        self.ordinal() - other.ordinal()
    }
    /// `date.weekday()`: Monday is 0.
    pub fn weekday(&self) -> i64 {
        (self.ordinal() + 3).rem_euclid(7)
    }
}

// NEEDS(clock: datetime.date.today) — PyBot's runs (and every oracle) pin the date to
// 2026-11-17; RSBOT_TODAY=YYYY-MM-DD overrides it.
/// `datetime.date.today()`.
pub fn today() -> Date {
    if let Ok(s) = std::env::var("RSBOT_TODAY") {
        let p: Vec<&str> = s.split('-').collect();
        if p.len() == 3 {
            if let (Ok(y), Ok(m), Ok(d)) = (p[0].parse(), p[1].parse(), p[2].parse()) {
                return Date { year: y, month: m, day: d };
            }
        }
    }
    Date { year: 2026, month: 11, day: 17 }
}

// ---------------------------------------------------------------------------------------------
// bot/memory/facts.py

// NEEDS(bot.memory.facts::extract) — the facts reader is a 10k-line module owned elsewhere;
// `money._a_first_meeting` only reads `extract(text).get("name")`. Until the port is merged
// this reads no name, so the saving answer is never held back for a welcome.
/// `extract(text).get("name")`.
pub fn extract_name(_text: &str) -> Option<String> {
    None
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/prices.py

static _JUDGING: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.prices._JUDGING"));

// was DUP-D2(bot.knowledge.prices::a_price_is_judged): now the real port
pub use crate::knowledge::prices::a_price_is_judged;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/cooking.py and bot/knowledge/units.py

fn str_int_pairs(name: &str) -> Vec<(String, i64)> {
    data::get(name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap())).collect()
}

static _CROWD_NUMBERS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| str_int_pairs("bot.knowledge.cooking._CROWD_NUMBERS"));
static _CROWD_FOR: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.cooking._CROWD_FOR"));
static _CROWD_WITHIN: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.cooking._CROWD_WITHIN"));
static _CROWD_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.cooking._CROWD_PHRASES"));
static _NOT_A_TABLE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.cooking._NOT_A_TABLE"));
static _TABLE_GROUPS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.units._TABLE_GROUPS"));
static _GROUP_VALUE: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| str_int_pairs("bot.knowledge.units._GROUP_VALUE"));


// was DUP-D2(bot.knowledge.cooking::_people_after): now the real port
pub use crate::knowledge::cooking::_people_after;

// DUP(bot.knowledge.units::people_in_groups)
/// How many people the groups named in the line add up to, or None when it names none.
pub fn people_in_groups(text: &str) -> Option<i64> {
    let mut total = 0;
    for raw in _TABLE_GROUPS.findall_str(&text.to_lowercase()) {
        let count = if text::is_digit(&raw) {
            raw.parse().ok()
        } else {
            _GROUP_VALUE.iter().find(|(k, _)| *k == raw).map(|(_, v)| *v)
        };
        if let Some(c) = count.filter(|c| *c != 0) {
            total += c;
        }
    }
    if total != 0 { Some(total) } else { None }
}

// was DUP-D2(bot.knowledge.cooking::_crowd): now the real port
pub use crate::knowledge::cooking::_crowd;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/geography.py

/// Every country phrase with its bounded pattern, and the alias table.
static COUNTRY_PHRASES: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    data::strs("bot.knowledge.geography._COUNTRY_PHRASES")
        .into_iter()
        .map(|p| {
            let re = Re::new(&format!("(?<![a-z]){}(?![a-z])", pyre::escape(&p)), 0);
            (p, re)
        })
        .collect()
});
static COUNTRY_ALIASES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.geography._COUNTRY_ALIASES")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), v.str().to_string()))
        .collect()
});

// DUP(bot.knowledge.geography::_countries_said)
/// The countries a line names, in the order it says them, as keys into COUNTRIES.
pub fn _countries_said(clean: &str) -> Vec<String> {
    let mut found: Vec<(usize, String)> = Vec::new();
    let mut taken: Vec<(usize, usize)> = Vec::new();
    for (phrase, re) in COUNTRY_PHRASES.iter() {
        for m in re.finditer(clean) {
            let (start, end) = (m.start(), m.end());
            if taken.iter().any(|&(before, after)| before < end && start < after) {
                continue;
            }
            taken.push((start, end));
            let name = COUNTRY_ALIASES.iter().find(|(k, _)| k == phrase).map(|(_, v)| v.clone()).unwrap_or_else(|| phrase.clone());
            found.push((start, name));
        }
    }
    found.sort();
    let mut said: Vec<String> = Vec::new();
    for (_, name) in found {
        if !said.contains(&name) {
            said.push(name);
        }
    }
    said
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dates() {
        let d = Date { year: 2026, month: 11, day: 17 };
        assert_eq!(d.weekday(), 1); // a Tuesday
        assert_eq!(Date { year: 2026, month: 12, day: 1 }.days_since(&d), 14);
        assert_eq!(Date { year: 2027, month: 1, day: 1 }.days_since(&d), 45);
    }
}
