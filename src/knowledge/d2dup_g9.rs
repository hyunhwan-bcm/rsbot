//! Private ports of helpers the g9 modules (babyfood, opinions, family_more, keeping,
//! cooking_gaps, fruit, costs, devices) need from code other workstreams own. Each is marked
//! `// DUP(<python module>::<name>)` (another workstream) or `// DUP-D2(...)` (another D2 group),
//! so the integrator can swap in the owner's port.

use crate::data;
use crate::py::re::{escape, Re};
use crate::understood::Understood;
use std::sync::LazyLock;



/// `lookup.the_appliance_swallows_the_name(understood, names)` (the real port).
pub fn the_appliance_swallows_the_name<S: AsRef<str>>(understood: &Understood, names: &[S]) -> bool {
    let names: Vec<String> = names.iter().map(|n| n.as_ref().to_string()).collect();
    crate::knowledge::lookup::the_appliance_swallows_the_name(understood, &names)
}

// ---------------------------------------------------------------------------------------------
// bot/memory

/// `memory.recent(memory, n)` (the real port): the last n turns.
pub fn recent(memory: &crate::state::Memory, n: usize) -> &[crate::state::Turn] {
    crate::memory::recent(Some(memory), n as i64)
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/family.py


// was DUP(bot.knowledge.family::_the_child_pronouns): now the real port
pub use crate::knowledge::family::_the_child_pronouns;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/units.py




pub use crate::knowledge::units::people_in_groups;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/cooking.py (owned by the cooking groups)


// was DUP-D2(bot.knowledge.cooking::_people_after): now the real port
pub use crate::knowledge::cooking::_people_after;

// was DUP-D2(bot.knowledge.cooking::_crowd): now the real port
pub use crate::knowledge::cooking::_crowd;


// was DUP-D2(bot.knowledge.cooking::_trays): now the real port
pub use crate::knowledge::cooking::_trays;

// was DUP-D2(bot.knowledge.cooking::_crowd_meal): now the real port
pub use crate::knowledge::cooking::_crowd_meal;

// was DUP-D2(bot.knowledge.cooking::_occasion): now the real port
pub use crate::knowledge::cooking::_occasion;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/dishes.py


// was DUP(bot.knowledge.dishes::_same_word): now the real port
pub use crate::knowledge::dishes::_same_word;

// was DUP(bot.knowledge.dishes::_ruled_out_near): now the real port
pub use crate::knowledge::dishes::_ruled_out_near;

// was DUP(bot.knowledge.dishes::dishes_ruled_out): now the real port
pub use crate::knowledge::dishes::dishes_ruled_out;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/entertainment.py (owned by the entertainment group)


// was DUP-D2(bot.knowledge.entertainment::_a_reservation): now the real port
pub use crate::knowledge::entertainment::_a_reservation;

// was DUP-D2(bot.knowledge.entertainment::the_book_is_a_reservation): now the real port
pub use crate::knowledge::entertainment::the_book_is_a_reservation;

// ---------------------------------------------------------------------------------------------
// Python's datetime.date, for the replies that name a date.

/// A calendar date (proleptic Gregorian), as Python's `datetime.date`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: i64,
    pub month: u32,
    pub day: u32,
}

impl Date {
    pub fn new(year: i64, month: u32, day: u32) -> Date {
        Date { year, month, day }
    }

    /// Days since 1970-01-01.
    pub fn ordinal(&self) -> i64 {
        let y = if self.month <= 2 { self.year - 1 } else { self.year };
        let era = (if y >= 0 { y } else { y - 399 }) / 400;
        let yoe = y - era * 400;
        let m = self.month as i64;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + self.day as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    }

    /// `(a - b).days`.
    pub fn days_since(&self, other: &Date) -> i64 {
        self.ordinal() - other.ordinal()
    }

    /// Monday is 0, as `date.weekday()`.
    pub fn weekday(&self) -> u32 {
        // 1970-01-01 was a Thursday (3)
        ((self.ordinal() + 3).rem_euclid(7)) as u32
    }

    /// `strftime("%A")`.
    pub fn weekday_name(&self) -> &'static str {
        ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"][self.weekday() as usize]
    }

    /// `strftime("%B")`.
    pub fn month_name(&self) -> &'static str {
        [
            "January", "February", "March", "April", "May", "June", "July", "August", "September", "October",
            "November", "December",
        ][(self.month - 1) as usize]
    }
}

/// `datetime.date.today()`, from the shared clock (`py::clock`).
pub fn today() -> Date {
    use chrono::Datelike;
    let d = crate::py::clock::today();
    Date::new(d.year() as i64, d.month(), d.day())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dates_match_python() {
        let cases = [
            ((2026, 11, 17), "Tuesday", 20774),
            ((2026, 12, 19), "Saturday", 20806),
            ((2027, 7, 21), "Wednesday", 21020),
            ((2000, 2, 29), "Tuesday", 11016),
            ((1969, 12, 31), "Wednesday", -1),
        ];
        for ((y, m, d), name, days) in cases {
            let date = Date::new(y, m, d);
            assert_eq!(date.weekday_name(), name);
            assert_eq!(date.ordinal(), days);
        }
    }
}
