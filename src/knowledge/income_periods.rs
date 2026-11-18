//! Port of PyBot's `bot/knowledge/income_periods.py`: whether a sum is enough to live on, and a
//! sum in one period said in another (a week's pay as a year's).

use crate::data::{self, FnRef};
use crate::knowledge::d1_dup_g7 as dup;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text::{round_half_even, thousands};
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("bot.knowledge.income_periods.{name}"))
}
fn rx(name: &str) -> &'static Re {
    data::re(&format!("bot.knowledge.income_periods.{name}"))
}
fn table(name: &str) -> Vec<(String, crate::data::Value)> {
    data::get(&format!("bot.knowledge.income_periods.{name}")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.clone())).collect()
}
static _LIVING: LazyLock<Vec<String>> = LazyLock::new(|| strs("_LIVING"));
static _COPING: LazyLock<Vec<String>> = LazyLock::new(|| strs("_COPING"));
static _ASKS_WHAT_IS_NEEDED: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ASKS_WHAT_IS_NEEDED"));
static _EARN_ENOUGH: LazyLock<Vec<String>> = LazyLock::new(|| strs("_EARN_ENOUGH"));
static _ASKS_FOR_THE_FIGURE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ASKS_FOR_THE_FIGURE"));
static _LONDON: LazyLock<&'static Re> = LazyLock::new(|| rx("_LONDON"));
static _A_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_FIGURE"));
static _A_PERIOD_IN_A_SUM: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_PERIOD_IN_A_SUM"));
static _ANOTHER_PERIOD: LazyLock<&'static Re> = LazyLock::new(|| rx("_ANOTHER_PERIOD"));
static _PERIODS_IN_A_YEAR: LazyLock<Vec<(String, crate::data::Value)>> = LazyLock::new(|| table("_PERIODS_IN_A_YEAR"));
static _EVERYDAY_PERIOD: LazyLock<Vec<(String, crate::data::Value)>> = LazyLock::new(|| table("_EVERYDAY_PERIOD"));
static _SAID_AGAIN: LazyLock<Vec<(String, crate::data::Value)>> = LazyLock::new(|| table("_SAID_AGAIN"));
static EARN_RE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(salary|wage)\b.{0,20}\b(need|needed|require[sd]?)\b", 0));
static A_YEAR: LazyLock<Re> = LazyLock::new(|| Re::new(r"a year|per year|each year|yearly|annual", 0));

fn lookup<'a>(t: &'a [(String, crate::data::Value)], key: &str) -> Option<&'a crate::data::Value> {
    t.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}
fn everyday(word: &str) -> String {
    lookup(&_EVERYDAY_PERIOD, word).map(|v| v.str().to_string()).unwrap_or_else(|| word.to_string())
}

/// "cost of living" read as living, so it is not taken for a bill.
pub fn _living_not_a_bill(text: &str) -> String {
    let mut text = text.to_string();
    for name in ["cost of living", "costs of living", "living costs", "living cost"] {
        text = text.replace(name, "living");
    }
    text
}

/// The yearly salary before tax that leaves `net_a_month` a month, to the next £100.
pub fn _the_gross_for(net_a_month: i64) -> i64 {
    let mut gross = net_a_month * 12;
    for _ in 0..40 {
        let (tax, ni) = dup::_income_tax_and_ni(gross as f64);
        if (gross as f64 - tax - ni) / 12.0 >= net_a_month as f64 {
            return gross.div_euclid(100) * 100 + if gross.rem_euclid(100) != 0 { 100 } else { 0 };
        }
        gross += 100;
    }
    gross
}

/// What one adult's month costs, in London or outside it.
pub fn _a_months_costs(london: bool) -> &'static str {
    if london {
        return "In London the same month is roughly £2,000, nearly all of it rent: a one-bed is £1,300 to £1,900 there before the bills.";
    }
    "Outside London one adult needs roughly £1,200 a month: rent £600 to £900, council tax about £180, energy about £145, food £150 to £200, and about £100 for broadband, phone and travel."
}

/// Whether the line asks if a sum is enough to live on for a month, or what one needs to earn.
pub fn _a_month_of_life(understood: &Understood) -> bool {
    let text = sh::padded(&sh::clean_or_text(understood).to_lowercase());
    if _A_FIGURE.is_match(&text) {
        if sh::any_in(&text, &_LIVING) {
            return true;
        }
        let topic = sh::field(understood, "topic");
        return sh::any_in(&text, &_COPING) && (topic.is_empty() || topic == "money");
    }
    if sh::any_in(&text, &_ASKS_WHAT_IS_NEEDED) || _earn_asked(&text) {
        return sh::any_in(&text, &_LIVING) || sh::any_in(&text, &_COPING);
    }
    false
}

/// Whether the text asks what salary one needs to earn.
pub fn _earn_asked(text: &str) -> bool {
    sh::any_in(text, &_EARN_ENOUGH) || EARN_RE.is_match(text)
}

/// Whether a sum is enough to live on: the take-home on a named salary, or the monthly sum
/// named, against what one adult needs (more in London).
pub fn _enough_to_live_on(_memory: &Memory, understood: &Understood) -> String {
    let text = sh::padded(&sh::clean_or_text(understood).to_lowercase());
    let london = _LONDON.is_match(&text);
    let (mut monthly, mut lead) = (0i64, String::new());
    if A_YEAR.is_match(&text) {
        if let Some(salary) = dup::_salary_in(&text).filter(|s| *s >= 5000) {
            let (tax, ni) = dup::_income_tax_and_ni(salary as f64);
            monthly = round_half_even((salary as f64 - tax - ni) / 12.0);
            lead = format!("On £{} a year you take home about £{} a month. ", thousands(salary), thousands(monthly));
        }
    }
    if monthly == 0 {
        monthly = dup::_per_month(&_living_not_a_bill(&text));
    }
    let (tight, enough) = if london { (1750, 2100) } else { (1050, 1300) };
    let place = if london { "in London" } else { "outside London" };
    if _earn_asked(&text) {
        return format!(
            "You need roughly £{} a year before tax to live on your own {} - about £{} a month after tax. {} Both move, so check this year's band and the price cap.",
            thousands(_the_gross_for(enough)),
            place,
            thousands(enough),
            _a_months_costs(london)
        );
    }
    if monthly == 0 {
        return format!("{} Tell me the figure you have in mind and I will say whether it covers the month.", _a_months_costs(london));
    }
    let verdict = if monthly >= enough {
        format!("£{} a month is enough {}, with room to save.", thousands(monthly), place)
    } else if monthly >= tight {
        format!(
            "£{} a month is enough {} only with a cheap rent - there is little left at the end of the month.",
            thousands(monthly),
            place
        )
    } else {
        format!("£{} a month leaves you about £{} short of what one adult needs {}.", thousands(monthly), thousands(enough - monthly), place)
    };
    format!("{lead}{verdict} {} Both move, so check this year's band and the price cap.", _a_months_costs(london))
}

/// A sum with its period and another period the text asks for: (amount, own, wanted).
pub fn _per_period(text: &str) -> Option<(f64, String, String)> {
    let m = _A_PERIOD_IN_A_SUM.search(text)?;
    let mut amount: f64 = m.str(1).replace(',', "").parse().ok()?;
    if m.get(2).is_some_and(|g| !g.is_empty()) {
        amount *= 1000.0;
    }
    let own = everyday(&m.str(3).to_lowercase());
    for other in _ANOTHER_PERIOD.finditer(text) {
        if other.start() < m.end() && other.end() > m.start() {
            continue;
        }
        let wanted = everyday(&sh::or(other.str(1), other.str(2)).to_lowercase());
        if ["week", "fortnight", "month", "year"].contains(&wanted.as_str()) && wanted != own {
            return Some((amount, own, wanted));
        }
    }
    None
}

/// Whether the line names a sum in one period and asks for another.
pub fn _two_periods_named(understood: &Understood) -> bool {
    _per_period(&sh::clean_or_text(understood).to_lowercase()).is_some()
}

/// The sum said in the period asked for; on pay, what it leaves after tax and NI.
pub fn _a_sum_in_another_period(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text = sh::clean_or_text(understood).to_lowercase();
    if ["keep", "records", "receipts", "payslips", "statements", "tax return", "how long", "how many years"]
        .iter()
        .any(|w| text.contains(w))
    {
        return None;
    }
    let (amount, own, wanted) = _per_period(&text)?;
    if !sh::any_in(&text, &_ASKS_FOR_THE_FIGURE) {
        return None;
    }
    let per_year = |k: &str| lookup(&_PERIODS_IN_A_YEAR, k).and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
    let said_again = |k: &str| lookup(&_SAID_AGAIN, k).map(|v| v.str().to_string()).unwrap_or_default();
    let annual = amount * per_year(&own);
    let value = annual / per_year(&wanted);
    let mut said = format!(
        "£{} {} is £{} {}.",
        thousands(round_half_even(amount)),
        said_again(&own),
        thousands(round_half_even(value)),
        said_again(&wanted)
    );
    let clause = dup::_clause_about(&text, 0, text.len());
    if ["earn", "earns", "salary", "wage", "wages", "paid", "pay", "job"].iter().any(|w| clause.contains(w)) {
        let (tax, ni) = dup::_income_tax_and_ni(round_half_even(annual) as f64);
        let left = annual - tax - ni;
        said.push_str(&format!(
            " After tax and National Insurance that is about £{} a month, or £{} a year.",
            thousands(round_half_even(left / 12.0)),
            thousands(round_half_even(left))
        ));
    }
    Some(said)
}

fn g_month(_f: &FnRef, u: &Understood) -> bool {
    _a_month_of_life(u)
}
fn g_two(_f: &FnRef, u: &Understood) -> bool {
    _two_periods_named(u)
}
fn r_enough(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_enough_to_live_on(m, u))
}
fn r_another(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _a_sum_in_another_period(m, u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.income_periods:_a_month_of_life:105", g_month),
    ("bot.knowledge.income_periods:_two_periods_named:248", g_two),
];
pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.income_periods:_enough_to_live_on:138", r_enough),
    ("bot.knowledge.income_periods:_a_sum_in_another_period:259", r_another),
];
