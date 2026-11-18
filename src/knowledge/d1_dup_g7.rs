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

/// DUP(bot.knowledge.cooking::_same): the word is the wanted one or starts with it.
pub fn _same(word: &str, wanted: &str) -> bool {
    !word.is_empty() && !wanted.is_empty() && (word == wanted || word.starts_with(wanted))
}

/// DUP(bot.knowledge.cooking::_roast_joint): the oven time for the joint the line names (both
/// when it names none); None on a keeping or going-off line.
pub fn _roast_joint(_memory: &Memory, understood: &Understood) -> Option<String> {
    let line = text::squash(&understood.text).to_lowercase();
    if GOING_OFF_ASK.iter().any(|p| line.contains(p.as_str())) {
        return None;
    }
    if KEEPING_VERB.iter().any(|p| line.contains(p.as_str())) && !IN_THE_OVEN.iter().any(|w| line.contains(w.as_str())) {
        return None;
    }
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let names = |wanted: &[&str]| words.iter().any(|w| wanted.iter().any(|x| _same(w, x)));
    if names(&["pork", "gammon", "ham"]) {
        return Some(JOINT_PORK.clone());
    }
    if names(&["beef", "lamb", "mutton"]) {
        return Some(JOINT_BEEF.clone());
    }
    Some(format!("{} {}", *JOINT_BEEF, *JOINT_PORK))
}

static STARTING_OUT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.repeat.STARTING_OUT"));
static STARTING_INTO: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.repeat.STARTING_INTO"));
static STARTING_AT: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.repeat.STARTING_AT"));
static STARTING_BACK: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.repeat.STARTING_BACK"));

/// DUP(bot.memory.repeat::_starting_out_names_a_thing): "starting out <thing>" names a thing
/// (not "starting out again", "starting out slowly").
pub fn _starting_out_names_a_thing(line: &str) -> bool {
    let Some(found) = STARTING_OUT.search(line) else { return false };
    let (first, second) = (found.str(1), found.str(2));
    if STARTING_INTO.contains(first) || STARTING_AT.contains(first) {
        return !second.is_empty() && !STARTING_BACK.contains(second);
    }
    !STARTING_BACK.contains(first)
}

static KEEPING_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.KEEPING_WORDS"));
static KEPT_FOODS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.KEPT_FOODS"));
static CONTAINERS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.CONTAINERS"));

/// DUP(bot.knowledge.lookup::a_food_is_being_kept): the line is about storing a food (a keeping
/// word and a kept food both named).
pub fn a_food_is_being_kept(understood: &Understood) -> bool {
    let text = format!(" {} ", crate::knowledge::d1_shared::or(&understood.norm_text, &understood.clean).to_lowercase());
    if !KEEPING_WORDS.iter().any(|w| text.contains(w.as_str())) {
        return false;
    }
    let said: HashSet<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    for food in KEPT_FOODS.iter() {
        if food.contains(' ') {
            if text.contains(food.as_str()) {
                return true;
            }
        } else if said.contains(food) || text.contains(&format!(" {food} ")) {
            return true;
        }
    }
    false
}

/// DUP(bot.knowledge.lookup::only_a_container): every one of `names` the line holds is a
/// container (a bag, a tub), and it holds at least one.
pub fn only_a_container<S: AsRef<str>>(understood: &Understood, names: &[S]) -> bool {
    let text = format!(" {} ", understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" "));
    let found: Vec<&str> = names.iter().map(|n| n.as_ref()).filter(|one| text.contains(&format!(" {one} "))).collect();
    !found.is_empty() && found.iter().all(|one| CONTAINERS.contains(*one))
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

/// DUP(bot.knowledge.money::_salary_in): the largest yearly salary the text names (a figure of
/// 3,000 or more, or one with "k"/"grand"), or None.
pub fn _salary_in(text: &str) -> Option<i64> {
    let mut best: Option<f64> = None;
    for m in _SALARY_NUMBER.finditer(text) {
        let raw = m.str(1).replace(',', "");
        let Ok(mut value) = raw.parse::<f64>() else { continue };
        if m.get(2).is_some_and(|g| !g.is_empty()) {
            value *= 1000.0;
        } else if value < 3000.0 {
            continue;
        }
        if value >= 1000.0 && best.is_none_or(|b| value > b) {
            best = Some(value);
        }
    }
    best.filter(|b| *b != 0.0).map(|b| b as i64)
}

/// DUP(bot.knowledge.money::_income_tax_and_ni): UK income tax and employee National Insurance
/// on a yearly income, as (tax, ni).
pub fn _income_tax_and_ni(income: f64) -> (f64, f64) {
    let pa = *_PERSONAL_ALLOWANCE;
    let mut allowance = pa;
    if income > 100000.0 {
        allowance = (allowance - (income - 100000.0) / 2.0).max(0.0);
    }
    let taxable = (income - allowance).max(0.0);
    let (mut tax, mut left) = (0.0, taxable);
    for (width, rate) in _TAX_BANDS.iter() {
        let part = left.min(*width);
        tax += part * rate;
        left -= part;
        if left <= 0.0 {
            break;
        }
    }
    if left > 0.0 {
        tax += left * *_TAX_TOP;
    }
    let mut ni = 0.0;
    if income > pa {
        ni += (income.min(*_NI_MAIN_TO) - pa) * 0.08;
    }
    if income > *_NI_MAIN_TO {
        ni += (income - *_NI_MAIN_TO) * 0.02;
    }
    (tax, ni)
}

/// DUP(bot.knowledge.money::_clause_about): the clause of `text` around [start, end) (byte
/// offsets; the marks are ASCII, so this is Python's code-point slicing), lower-cased; when it
/// joins halves with "and"/"but"/"so"/"then", the first half that holds a sum of money.
pub fn _clause_about(text: &str, start: usize, end: usize) -> String {
    let marks = ['.', ';', ',', '?'];
    let left = marks.iter().filter_map(|m| text[..start].rfind(*m)).max().map(|i| i + 1).unwrap_or(0);
    let right = marks.iter().filter_map(|m| text.get(end..).and_then(|t| t.find(*m)).map(|i| i + end)).min();
    let stop = right.unwrap_or(text.len());
    let clause = if left <= stop { text[left..stop].to_lowercase() } else { String::new() };
    for joiner in [" and ", " but ", " so ", " then "] {
        if clause.contains(joiner) {
            for piece in clause.split(joiner) {
                if MONEY_RE.is_match(piece) {
                    return piece.to_string();
                }
            }
        }
    }
    clause
}

/// DUP(bot.knowledge.money::_said_a_month): the figure a money match names (times 1,000 for
/// "k"/"grand").
pub fn _said_a_month(m: &crate::py::re::Match) -> f64 {
    let amount: f64 = m.str(1).replace(',', "").parse().unwrap_or(0.0);
    if m.get(2).is_some_and(|g| !g.is_empty()) { amount * 1000.0 } else { amount }
}

/// DUP(bot.knowledge.money::_per_month): a sum "a month/week/fortnight/year" in the text, as a
/// monthly figure (0 when there is none, or the clause is a cost going out).
pub fn _per_month(text: &str) -> i64 {
    let Some(m) = PER_MONTH_RE.search(text) else { return 0 };
    let clause = _clause_about(text, m.start(), m.end());
    if GOING_OUT.iter().any(|w| clause.contains(w.as_str())) && !GOING_IN.iter().any(|w| clause.contains(w.as_str())) {
        return 0;
    }
    let factor = match m.str(3) {
        "month" => 1.0,
        "week" => 52.0 / 12.0,
        "fortnight" => 26.0 / 12.0,
        _ => 1.0 / 12.0,
    };
    text::round_half_even(_said_a_month(&m) * factor)
}

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

/// DUP(bot.knowledge.lookup::the_ask_is_the_order_of_steps): the line asks which step comes
/// first.
pub fn the_ask_is_the_order_of_steps(understood: &Understood) -> bool {
    let joined;
    let mut said = crate::knowledge::d1_shared::text_or_clean(understood);
    if said.is_empty() {
        joined = understood.words.join(" ");
        said = &joined;
    }
    !said.is_empty() && _ORDER_OF_STEPS.is_match(said)
}

/// DUP(bot.knowledge.lookup::not_the_ask_about_the_order).
pub fn not_the_ask_about_the_order(understood: &Understood) -> bool {
    !the_ask_is_the_order_of_steps(understood)
}

static FREEZE_DISH_TIMES: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.cooking_entries.FREEZE_DISH_TIMES")
        .items()
        .iter()
        .map(|r| (r.items()[0].strs(), r.items()[1].str().to_string()))
        .collect()
});
static FREEZE_COOKED_STEP: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.cooking_entries.FREEZE_COOKED_STEP"));

/// DUP(bot.knowledge.cooking_entries::freeze_cooked_answer): how long the cooked dish the line
/// names keeps frozen (`_freeze_cooked`).
pub fn freeze_cooked_answer(_memory: &Memory, understood: &Understood) -> String {
    let said = format!(" {} ", understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" "));
    for (names, keeps) in FREEZE_DISH_TIMES.iter() {
        if names.iter().any(|n| said.contains(&format!(" {n} ")) || said.contains(&format!(" {n}s "))) {
            return pct(
                "Yes - it keeps %s in the freezer, so cool it within an hour, then pack it flat in bags or tubs so it freezes fast. %s",
                &[keeps, &FREEZE_COOKED_STEP],
            );
        }
    }
    pct(
        "Yes - cooked dishes keep about three months frozen, and a month for mash, so cool it within an hour, then pack it flat in bags or tubs so it freezes fast. %s",
        &[&FREEZE_COOKED_STEP],
    )
}

/// DUP(bot.knowledge.lookup::said_in_this_line): the line as typed this turn (the answers
/// module's copy, else the reading's own line without the carried-over earlier line).
pub fn said_in_this_line(understood: &Understood, memory: Option<&Memory>) -> String {
    if let Some(typed) = memory.and_then(|m| m.asked_now.as_deref()) {
        if !text::strip(typed).is_empty() {
            return text::strip(typed).to_string();
        }
    }
    // `understood["said_now"]` is not a field the reading carries here.
    if understood.setup.as_deref().is_some_and(|s| !s.is_empty()) && !understood.focus.is_empty() {
        return text::strip(&understood.focus.join(" ")).to_string();
    }
    // LEANS_ON: Python removes the earlier reading's text when the line leans on one.
    text::strip(crate::knowledge::d1_shared::clean_or_text(understood)).to_string()
}
