//! Private ports of helpers the g9 modules (babyfood, opinions, family_more, keeping,
//! cooking_gaps, fruit, costs, devices) need from code other workstreams own. Each is marked
//! `// DUP(<python module>::<name>)` (another workstream) or `// DUP-D2(...)` (another D2 group),
//! so the integrator can swap in the owner's port.

use crate::data;
use crate::py::re::{escape, Re};
use crate::understood::Understood;
use std::sync::LazyLock;

/// Python `str(list_of_str)`, or "" for an empty list (`str(x or "")`).
fn list_or_empty(items: &[String]) -> String {
    if items.is_empty() { String::new() } else { crate::knowledge::d2_shared::py_repr_list(items) }
}

static APPLIANCE_NAMES: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    data::strs("bot.knowledge.lookup.APPLIANCE_NAMES")
        .into_iter()
        .map(|a| {
            let rx = Re::new(&format!("(?<![a-z0-9]){}(?![a-z0-9])", escape(&a)), 0);
            (a, rx)
        })
        .collect()
});

// DUP(bot.knowledge.lookup::the_appliance_swallows_the_name)
/// True when a named appliance contains one of `names` in its own name ("heat pump" inside
/// "heat pump dryer"), so the appliance's own row answers the line.
pub fn the_appliance_swallows_the_name<S: AsRef<str>>(understood: &Understood, names: &[S]) -> bool {
    let parts = [understood.clean.clone(), understood.text.clone(), list_or_empty(&understood.norm)];
    let said = format!(" {} ", parts.join(" ").to_lowercase());
    let names: Vec<String> = names
        .iter()
        .map(|n| n.as_ref())
        .filter(|n| !crate::py::text::strip(n).is_empty())
        .map(|n| n.to_lowercase())
        .collect();
    if names.is_empty() {
        return false;
    }
    for (appliance, rx) in APPLIANCE_NAMES.iter() {
        if !rx.is_match(&said) {
            continue;
        }
        if names.iter().any(|n| appliance.contains(n.as_str()) && n.chars().count() < appliance.chars().count()) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------------------------
// bot/memory

// DUP(bot.memory::recent)
/// The last n turns, oldest first.
pub fn recent(memory: &crate::state::Memory, n: usize) -> &[crate::state::Turn] {
    let turns = &memory.turns;
    &turns[turns.len().saturating_sub(n)..]
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/family.py

static A_BOY: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.family._A_BOY"));
static A_GIRL: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.family._A_GIRL"));

// was DUP(bot.knowledge.family::_the_child_pronouns): now the real port
pub use crate::knowledge::family::_the_child_pronouns;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/units.py

static TABLE_GROUPS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.units._TABLE_GROUPS"));
static GROUP_VALUE: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| int_dict("bot.knowledge.units._GROUP_VALUE"));

fn int_dict(name: &str) -> Vec<(String, i64)> {
    data::get(name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0))).collect()
}

fn lookup_int(table: &[(String, i64)], key: &str) -> Option<i64> {
    table.iter().find(|(k, _)| k == key).map(|(_, v)| *v)
}

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

static INGREDIENT_AFTER: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.dishes._INGREDIENT_AFTER"));
static RULE_OUT_FILLER: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.dishes._RULE_OUT_FILLER"));
static RULE_OUT: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.dishes._RULE_OUT"));
/// `dishes.PLANS`: each plan key with the token lists of its names.
static PLAN_NAMES: LazyLock<Vec<(String, Vec<Vec<String>>)>> = LazyLock::new(|| {
    data::get("bot.knowledge.dishes.PLANS")
        .pairs()
        .iter()
        .map(|(k, plan)| {
            let mut names = Vec::new();
            if let Some(groups) = plan.get("names") {
                for group in groups.items() {
                    match group {
                        crate::data::Value::Tuple(_) => {
                            for name in group.items() {
                                names.push(crate::knowledge::d2_shared::tokens(name.str()));
                            }
                        }
                        other => names.push(crate::knowledge::d2_shared::tokens(other.str())),
                    }
                }
            }
            (k.str().to_string(), names)
        })
        .collect()
});

// was DUP(bot.knowledge.dishes::_same_word): now the real port
pub use crate::knowledge::dishes::_same_word;

// was DUP(bot.knowledge.dishes::_ruled_out_near): now the real port
pub use crate::knowledge::dishes::_ruled_out_near;

// DUP(bot.knowledge.dishes::dishes_ruled_out)
/// The plan keys a text names only to rule them out ("isnt a roast").
pub fn dishes_ruled_out(text: &str) -> std::collections::HashSet<String> {
    let have = crate::knowledge::d2_shared::tokens(text);
    let mut out = std::collections::HashSet::new();
    for (key, names) in PLAN_NAMES.iter() {
        for wanted in names {
            if wanted.is_empty() || wanted.len() > have.len() {
                continue;
            }
            for i in 0..(have.len() - wanted.len() + 1) {
                if (0..wanted.len()).all(|j| _same_word(&have[i + j], &wanted[j])) && _ruled_out_near(&have, i, wanted.len()) {
                    out.insert(key.clone());
                    break;
                }
            }
        }
    }
    out
}

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
