//! DUP ports (group 7 of workstream D1): private ports of helpers from modules other
//! workstreams own, each marked `DUP(<python module>::<name>)` so the integrator can replace it.

use crate::data;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

static GOING_OFF_ASK: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.cooking.GOING_OFF_ASK"));
static KEEPING_VERB: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.cooking.KEEPING_VERB"));
static IN_THE_OVEN: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.cooking.IN_THE_OVEN"));
static JOINT_BEEF: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.cooking.JOINT_BEEF"));
static JOINT_PORK: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.cooking.JOINT_PORK"));

// was DUP(bot.knowledge.cooking::_same): now the real port
pub use crate::knowledge::cooking::_same;

/// `cooking._roast_joint(memory, understood)` (the real port; it writes nothing to memory).
pub fn _roast_joint(memory: &Memory, understood: &Understood) -> Option<String> {
    crate::knowledge::cooking::_roast_joint(&mut memory.clone(), understood)
}

static STARTING_OUT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.repeat.STARTING_OUT"));
static STARTING_INTO: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.repeat.STARTING_INTO"));
static STARTING_AT: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.repeat.STARTING_AT"));
static STARTING_BACK: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.repeat.STARTING_BACK"));

// was DUP(bot.memory.repeat::_starting_out_names_a_thing): now the real port
pub use crate::memory::repeat::_starting_out_names_a_thing;

static KEEPING_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.KEEPING_WORDS"));
static KEPT_FOODS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.KEPT_FOODS"));
static CONTAINERS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.CONTAINERS"));

// was DUP(bot.knowledge.lookup::a_food_is_being_kept): now the real port
pub use crate::knowledge::lookup::a_food_is_being_kept;

/// `lookup.only_a_container(understood, names)` (the real port).
pub fn only_a_container<S: AsRef<str>>(understood: &Understood, names: &[S]) -> bool {
    let names: Vec<String> = names.iter().map(|n| n.as_ref().to_string()).collect();
    crate::knowledge::lookup::only_a_container(understood, &names)
}

/// Python's `fmt % args` for `%s` (and `%%`) placeholders, in order.
pub fn pct(fmt: &str, args: &[&str]) -> String {
    let mut out = String::with_capacity(fmt.len() + 32);
    let mut args = args.iter();
    let mut chars = fmt.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            match chars.peek() {
                Some('s') => {
                    chars.next();
                    out.push_str(args.next().copied().unwrap_or(""));
                    continue;
                }
                Some('%') => {
                    chars.next();
                    out.push('%');
                    continue;
                }
                _ => {}
            }
        }
        out.push(c);
    }
    out
}

/// Python's `str.isdigit()` for one character: decimal digits and the superscript digits
/// (not fractions or roman numerals, which Python counts as numeric but not digits).
pub fn is_digit_char(c: char) -> bool {
    c.is_ascii_digit()
        || (c.is_numeric() && !('\u{00BC}'..='\u{00BE}').contains(&c) && !('\u{2150}'..='\u{218F}').contains(&c))
}

// ---- bot.knowledge.money ------------------------------------------------------------------

fn money_f(name: &str) -> f64 {
    data::get(&format!("bot.knowledge.money.{name}")).as_f64().unwrap_or(0.0)
}
/// `money._MONEY`: a sum of money as a pattern string (group 1 the figure, 2 "k"/"grand").
pub static MONEY_PATTERN: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.money._MONEY"));
static MONEY_RE: LazyLock<Re> = LazyLock::new(|| Re::new(&MONEY_PATTERN, 0));
static _SALARY_NUMBER: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.money._SALARY_NUMBER"));
static _PERSONAL_ALLOWANCE: LazyLock<f64> = LazyLock::new(|| money_f("_PERSONAL_ALLOWANCE"));
static _TAX_TOP: LazyLock<f64> = LazyLock::new(|| money_f("_TAX_TOP"));
static _NI_MAIN_TO: LazyLock<f64> = LazyLock::new(|| money_f("_NI_MAIN_TO"));
static _TAX_BANDS: LazyLock<Vec<(f64, f64)>> = LazyLock::new(|| {
    data::get("bot.knowledge.money._TAX_BANDS")
        .items()
        .iter()
        .map(|b| (b.items()[0].as_f64().unwrap_or(0.0), b.items()[1].as_f64().unwrap_or(0.0)))
        .collect()
});
static PER_MONTH_RE: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        &format!(
            r"{}{}\s*(?:a|per|each|every)\s*(month|week|fortnight|year)",
            *MONEY_PATTERN,
            data::string("bot.knowledge.money.BETWEEN_THE_SUM_AND_THE_RATE")
        ),
        0,
    )
});
static GOING_OUT: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.money.GOING_OUT"));
static GOING_IN: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.money.GOING_IN"));

// was DUP(bot.knowledge.money::_salary_in): now the real port
pub use crate::knowledge::money::_salary_in;

// was DUP(bot.knowledge.money::_income_tax_and_ni): now the real port
pub use crate::knowledge::money::_income_tax_and_ni;

// was DUP(bot.knowledge.money::_clause_about): now the real port
pub use crate::knowledge::money::_clause_about;

// was DUP(bot.knowledge.money::_said_a_month): now the real port
pub use crate::knowledge::money::_said_a_month;

// was DUP(bot.knowledge.money::_per_month): now the real port
pub use crate::knowledge::money::_per_month;

// ---- the pinned clock and civil dates ---------------------------------------------------------
// Read from the shared clock (`py::clock`), which the checkers pin to the oracle's day.

/// `date.today()` as (year, month, day).
pub fn today() -> (i64, i64, i64) {
    use chrono::Datelike;
    let d = crate::py::clock::today();
    (d.year() as i64, d.month() as i64, d.day() as i64)
}

/// `datetime.now()` (naive local time) as (days since 1970-01-01, seconds into the day). On a
/// pinned day it is noon, as the harness pins it (`run_python.pin_clock`).
pub fn now_local() -> (i64, i64) {
    use chrono::{Datelike, Timelike};
    if std::env::var_os("RSBOT_TODAY").is_some() {
        let (y, m, d) = today();
        return (days_from_civil(y, m, d), 12 * 3600);
    }
    let now = crate::py::clock::now_utc().with_timezone(&chrono::Local).naive_local();
    (days_from_civil(now.year() as i64, now.month() as i64, now.day() as i64), now.num_seconds_from_midnight() as i64)
}

/// Days since 1970-01-01 of a proleptic Gregorian date (Howard Hinnant's algorithm).
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// The date (year, month, day) of a day count since 1970-01-01.
pub fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { yoe + era * 400 + 1 } else { yoe + era * 400 }, m, d)
}

/// `strftime("%A")` of a day count since 1970-01-01.
pub fn weekday_name(days: i64) -> &'static str {
    ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"][(days + 3).rem_euclid(7) as usize]
}

/// `strftime("%B")` of a month number.
pub fn month_name(m: i64) -> &'static str {
    ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"]
        [(m - 1).clamp(0, 11) as usize]
}

/// Whether a (year, month, day) is a real date (`datetime.date(y, m, d)` does not raise).
pub fn valid_date(y: i64, m: i64, d: i64) -> bool {
    if !(1..=9999).contains(&y) || !(1..=12).contains(&m) || d < 1 {
        return false;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let len = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][(m - 1) as usize];
    d <= len
}

static _ORDER_OF_STEPS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup._ORDER_OF_STEPS"));

// was DUP(bot.knowledge.lookup::the_ask_is_the_order_of_steps): now the real port
pub use crate::knowledge::lookup::the_ask_is_the_order_of_steps;

// was DUP(bot.knowledge.lookup::not_the_ask_about_the_order): now the real port
pub use crate::knowledge::lookup::not_the_ask_about_the_order;

static FREEZE_DISH_TIMES: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.cooking_entries.FREEZE_DISH_TIMES")
        .items()
        .iter()
        .map(|r| (r.items()[0].strs(), r.items()[1].str().to_string()))
        .collect()
});
static FREEZE_COOKED_STEP: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.cooking_entries.FREEZE_COOKED_STEP"));

/// `cooking_entries.freeze_cooked_answer(memory, understood)` (the real port).
pub fn freeze_cooked_answer(memory: &Memory, understood: &Understood) -> String {
    crate::knowledge::cooking_entries::freeze_cooked_answer(&mut memory.clone(), understood).unwrap_or_default()
}

// was DUP(bot.knowledge.lookup::said_in_this_line): now the real port
pub use crate::knowledge::lookup::said_in_this_line;
