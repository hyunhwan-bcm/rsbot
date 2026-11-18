//! Port of `bot/knowledge/costs.py`: what everyday jobs and things cost, and whether a sum a
//! month is too much. The job tables and entries are data from the bundle.
//!
//! Build-time only (their results are the bundle's ENTRIES): `_repair_phrases`, `time_entries`.

use crate::data;
use crate::data::{FnRef, Value};
use crate::knowledge::d2dup_g9::the_appliance_swallows_the_name;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{escape, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashMap;
use std::sync::LazyLock;

fn re(name: &str) -> &'static Re {
    data::re(&format!("bot.knowledge.costs.{name}"))
}

static _HOW_TO: LazyLock<&'static Re> = LazyLock::new(|| re("_HOW_TO"));
static _STEPS_ASK: LazyLock<&'static Re> = LazyLock::new(|| re("_STEPS_ASK"));
static _PRICE_ASK: LazyLock<&'static Re> = LazyLock::new(|| re("_PRICE_ASK"));
static _PRICE_SHAPE: LazyLock<&'static Re> = LazyLock::new(|| re("_PRICE_SHAPE"));
static _AMOUNT_SHAPE: LazyLock<&'static Re> = LazyLock::new(|| re("_AMOUNT_SHAPE"));
static _SUM: LazyLock<&'static Re> = LazyLock::new(|| re("_SUM"));
static _TOO_MUCH: LazyLock<&'static Re> = LazyLock::new(|| re("_TOO_MUCH"));
static _WORTH_IT: LazyLock<&'static Re> = LazyLock::new(|| re("_WORTH_IT"));
static _ABOUT_RIGHT: LazyLock<&'static Re> = LazyLock::new(|| re("_ABOUT_RIGHT"));
static _FIXED_FEE: LazyLock<&'static Re> = LazyLock::new(|| re("_FIXED_FEE"));
static _NOT_POUNDS: LazyLock<&'static Re> = LazyLock::new(|| re("_NOT_POUNDS"));
static _SHORT_PERIOD: LazyLock<&'static Re> = LazyLock::new(|| re("_SHORT_PERIOD"));
static _ONE_OCCASION: LazyLock<&'static Re> = LazyLock::new(|| re("_ONE_OCCASION"));
static _NOT_A_BILL: LazyLock<&'static Re> = LazyLock::new(|| re("_NOT_A_BILL"));
static _USES_IT: LazyLock<&'static Re> = LazyLock::new(|| re("_USES_IT"));
static _NOT_GOING: LazyLock<&'static Re> = LazyLock::new(|| re("_NOT_GOING"));
static DIGITS: LazyLock<Re> = LazyLock::new(|| Re::new(r"(\d+)", 0));

static _JUST_A_PAYMENT: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.costs._JUST_A_PAYMENT"));
static _COUNT: LazyLock<HashMap<String, i64>> = LazyLock::new(|| {
    data::get("bot.knowledge.costs._COUNT")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0)))
        .collect()
});
/// `_PERIODS`: (pattern, how many of that period make a year).
static _PERIODS: LazyLock<Vec<(&'static Re, f64)>> = LazyLock::new(|| {
    data::get("bot.knowledge.costs._PERIODS")
        .items()
        .iter()
        .map(|row| (row.items()[0].as_re().unwrap(), row.items()[1].as_f64().unwrap()))
        .collect()
});
static SUM_A_MONTH_SAMPLE: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.costs.SUM_A_MONTH_SAMPLE"));

/// One row of `SUBSCRIPTIONS`.
struct Subscription {
    names: Vec<(String, Re)>,
    thing: String,
    span: (f64, f64),
    visit: Option<(f64, f64)>,
    uses: String,
    advice: String,
}

fn pair(v: Option<&Value>) -> Option<(f64, f64)> {
    let v = v?;
    let items = v.items();
    Some((items.first()?.as_f64()?, items.get(1)?.as_f64()?))
}

static SUBSCRIPTIONS: LazyLock<Vec<Subscription>> = LazyLock::new(|| {
    data::get("bot.knowledge.costs.SUBSCRIPTIONS")
        .pairs()
        .iter()
        .map(|(_, d)| Subscription {
            names: d
                .get("names")
                .map(|n| n.strs())
                .unwrap_or_default()
                .into_iter()
                .map(|n| {
                    let rx = Re::new(&format!(r"\b{}\b", escape(&n)), 0);
                    (n, rx)
                })
                .collect(),
            thing: d.get("thing").map(|x| x.str().to_string()).unwrap_or_default(),
            span: pair(d.get("span")).unwrap_or((0.0, 0.0)),
            visit: pair(d.get("visit")).filter(|_| d.get("visit").is_some_and(Value::truthy)),
            uses: d.get("uses").map(|x| x.str().to_string()).unwrap_or_default(),
            advice: d.get("advice").map(|x| x.str().to_string()).unwrap_or_default(),
        })
        .collect()
});

/// False when the line asks for the steps themselves ("how to unblock a drain").
pub fn _not_a_steps_ask(understood: &Understood) -> bool {
    let text = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    !_HOW_TO.is_match(&text.to_lowercase())
}

/// True when the line asks how, or what is needed, and never asks what it costs.
pub fn _a_steps_ask_with_no_price(understood: &Understood) -> bool {
    let text = if understood.clean.is_empty() { &understood.text } else { &understood.clean }.to_lowercase();
    _STEPS_ASK.is_match(&text) && !_PRICE_ASK.is_match(&text)
}

/// The verdict for "is it worth servicing the boiler", the price otherwise.
pub fn _boiler_service_answer(_memory: &mut Memory, reading: &Understood) -> String {
    let asks = &reading.asks;
    if !asks.iter().any(|a| a == "how_often") && asks.iter().any(|a| a == "yesno" || a == "recommend") {
        return "Yes, once a year: a boiler service is usually £60 to £120 depending on where you \
                are, it keeps the warranty valid, catches a failing part before it strands you in \
                winter, and landlords are legally required to have one. Book it in late summer, \
                when engineers are quiet and the price is lower."
            .to_string();
    }
    "A boiler service is usually £60 to £120 once a year, and a gas safety check on top is \
     £50 to £90 if you rent the place out. It is a check, not a repair: the engineer \
     cleans and tests the boiler and tells you what is wearing out, so it is the cheap way \
     to avoid a winter breakdown. Ask whether the price includes the gas safety certificate \
     if you need one."
        .to_string()
}

/// The guard every job's price row carries (the lambda in the ENTRIES comprehension): not a
/// steps ask, not a fenced how-to with no price in it, and not an appliance that holds the name.
pub fn _job_price_guard(understood: &Understood, names: &[String], fence: bool) -> bool {
    _not_a_steps_ask(understood)
        && !(fence && _a_steps_ask_with_no_price(understood))
        && !the_appliance_swallows_the_name(understood, names)
}

/// True when the line asks what a grocery costs, and not how much of it to use.
pub fn _a_price_for_the_shop(understood: &Understood) -> bool {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean }.to_lowercase();
    if line.is_empty() || _AMOUNT_SHAPE.is_match(&line) {
        return false;
    }
    _PRICE_SHAPE.is_match(&line)
}

/// How often they say they go, as visits a month: 0 says nothing about how often, -1 says it
/// without a period ("i go twice").
pub fn _visits_a_month(text: &str) -> f64 {
    let Some(m) = _USES_IT.search(text) else { return 0.0 };
    let said = m.str(1).to_lowercase();
    let count = match _COUNT.get(&said) {
        Some(c) => *c,
        None => DIGITS.match_(&said).and_then(|d| d.str(1).parse::<i64>().ok()).unwrap_or(0),
    };
    if count == 0 {
        return 0.0;
    }
    let period = m.str(2).to_lowercase();
    if period.is_empty() {
        return -1.0;
    }
    match period.as_str() {
        "month" => count as f64,
        "day" => (count * 30) as f64,
        "fortnight" => (count * 2) as f64,
        _ => count as f64 * 52.0 / 12.0,
    }
}

/// The first sum in the line and what it comes to per month, or (0, 0).
pub fn _sum_and_per_month(text: &str) -> (f64, f64) {
    for m in _SUM.finditer(text) {
        let Ok(mut amount) = m.str(1).replace(',', "").parse::<f64>() else { continue };
        if m.get(2).is_some_and(|k| !k.is_empty()) {
            amount *= 1000.0;
        }
        if amount < 1.0 {
            continue;
        }
        let end = text::char_index(text, m.end()) as isize;
        let after = text::slice(text, end, Some(end + 22));
        for (pattern, a_year) in _PERIODS.iter() {
            if pattern.is_match(&after) {
                return (amount, amount * a_year / 12.0);
            }
        }
    }
    (0.0, 0.0)
}

/// "£1,234" for a figure, rounded as Python's round().
pub fn _pounds(figure: f64) -> String {
    format!("£{}", text::thousands(text::round_half_even(figure)))
}

/// Yes or no in the words the line asked with: too much, worth it, or about right.
pub fn _verdict(monthly: f64, low: f64, high: f64, text: &str, thing: &str) -> String {
    if _WORTH_IT.is_match(text) {
        if monthly < (low + high) / 2.0 {
            return format!("Yes, that is good value for {thing}");
        }
        if monthly <= high {
            return format!("Yes, that is about the going rate for {thing}");
        }
        return format!("No, that is dear for {thing}");
    }
    if _ABOUT_RIGHT.is_match(text) {
        if monthly <= high {
            return format!("Yes, that is about right for {thing}");
        }
        return format!("No, it is above the usual range for {thing}");
    }
    if monthly < low {
        return format!("No, that is below the usual range for {thing}");
    }
    if monthly <= high {
        return format!("No, that is normal for {thing}");
    }
    if monthly <= high * 1.6 {
        return format!("No, though it is at the top of the range for {thing}");
    }
    format!("Yes, that is well above the range for {thing}")
}

/// The verdict on a sum per period, from the thing the line names, or None.
pub fn _sum_a_month_reply(understood: &Understood) -> Option<String> {
    let text: &str = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    if !(_TOO_MUCH.is_match(text) || _WORTH_IT.is_match(text) || _ABOUT_RIGHT.is_match(text)) {
        return None;
    }
    if _FIXED_FEE.is_match(text) || _NOT_A_BILL.is_match(text) || _NOT_POUNDS.is_match(text) {
        return None;
    }
    let uses = _visits_a_month(text);
    let (_amount, monthly) = _sum_and_per_month(text);
    if monthly == 0.0 {
        return None;
    }
    let joined = understood.words.join(" ").to_lowercase();
    let r#where = if joined.is_empty() { text.to_lowercase() } else { joined };
    let mut named: Vec<(i64, usize, &Subscription)> = Vec::new();
    for data in SUBSCRIPTIONS.iter() {
        let matched: Vec<&String> = data.names.iter().filter(|(_, rx)| rx.is_match(&r#where)).map(|(n, _)| n).collect();
        let thing_named: Vec<&&String> = matched.iter().filter(|n| !_JUST_A_PAYMENT.contains(n)).collect();
        if !thing_named.is_empty() {
            named.push((1, thing_named.iter().map(|n| n.chars().count()).max().unwrap(), data));
        } else if !matched.is_empty() {
            named.push((0, matched.iter().map(|n| n.chars().count()).max().unwrap(), data));
        }
    }
    named.sort_by_key(|row| (-row.0, -(row.1 as i64)));
    for (_, _, data) in named {
        let (low, high) = data.span;
        if _NOT_GOING.is_match(text) {
            if let Some((visit_low, visit_high)) = data.visit {
                return Some(format!(
                    "Yes, that is too much for something you do not use: {} a month is {} a year \
                     for nothing. Cancel it in writing and keep the date you sent it, because most \
                     gyms want a month's notice, and pay as you go when you do go - about {} to \
                     {} a visit.",
                    _pounds(monthly),
                    _pounds(monthly * 12.0),
                    _pounds(visit_low),
                    _pounds(visit_high)
                ));
            }
            return None;
        }
        if uses >= 1.0 {
            if let Some((visit_low, visit_high)) = data.visit {
                let per_visit = monthly / uses;
                let verdict = if per_visit <= visit_high {
                    "No, that works out at a fair price a visit"
                } else if per_visit <= visit_high * 1.6 {
                    "No, though it is dear per visit at the rate you go"
                } else {
                    "Yes, that is too much for how often you go"
                };
                let count = text::round_half_even(uses);
                return Some(format!(
                    "{}. {} visit{} a month at {} a month is {} a visit, and paying as you go is \
                     usually {} to {} a visit. {}",
                    verdict,
                    count,
                    if count == 1 { "" } else { "s" },
                    _pounds(monthly),
                    _pounds(per_visit),
                    _pounds(visit_low),
                    _pounds(visit_high),
                    data.advice
                ));
            }
        }
        if uses != 0.0 {
            return None;
        }
        if data.visit.is_some() && _SHORT_PERIOD.is_match(text) && _ONE_OCCASION.is_match(text) {
            return None;
        }
        let uses_said = data
            .uses
            .replace("{quarter}", &_pounds(monthly / 4.0))
            .replace("{eighth}", &_pounds(monthly / 8.0))
            .replace("{day}", &_pounds(monthly / 22.0))
            .replace("{week}", &_pounds(monthly * 12.0 / 52.0));
        return Some(format!(
            "{}. {} a month is {} a year, and {} is usually {} to {} a month. {} {}",
            _verdict(monthly, low, high, text, &data.thing),
            _pounds(monthly),
            _pounds(monthly * 12.0),
            data.thing,
            _pounds(low),
            _pounds(high),
            uses_said,
            data.advice
        ));
    }
    None
}

/// A sum per period with an opinion of it: the verdict first, then what makes it one; read
/// with no sum in the line, the row's own sample line.
pub fn _a_sum_a_month(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    _sum_a_month_reply(understood).or_else(|| {
        let sample = Understood { clean: SUM_A_MONTH_SAMPLE.clone(), ..Default::default() };
        _sum_a_month_reply(&sample)
    })
}

fn job_price_guard(f: &FnRef, u: &Understood) -> bool {
    let names = f.defaults.first().map(|v| v.strs()).unwrap_or_default();
    let fence = f.defaults.get(1).is_some_and(Value::truthy);
    _job_price_guard(u, &names, fence)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.costs:<lambda>:1137", job_price_guard),
    ("bot.knowledge.costs:<lambda>:1929", |_: &FnRef, u: &Understood| _sum_a_month_reply(u).is_some_and(|s| !s.is_empty())),
    ("bot.knowledge.costs:_a_price_for_the_shop:1202", |_: &FnRef, u: &Understood| _a_price_for_the_shop(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.costs:_a_sum_a_month:1895", |_: &FnRef, m: &mut Memory, u: &Understood| _a_sum_a_month(m, u)),
    ("bot.knowledge.costs:_boiler_service_answer:172", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_boiler_service_answer(m, u))
    }),
];


// ---------------------------------------------------------------------------------------------
// The row factories (`knowledge::built`).

use crate::knowledge::built;
use crate::knowledge::entry::{Entry, Reply};

/// `_repair_phrases(name)`: the ways a repair of this thing is asked for.
pub fn _repair_phrases(name: &str) -> Vec<String> {
    let article = if name.starts_with(['a', 'e', 'i', 'o', 'u']) { "an" } else { "a" };
    let mut out: Vec<String> = ["fix", "repair"].iter().map(|verb| format!("{verb} {article} {name}")).collect();
    out.extend([
        format!("fix the {name}"),
        format!("repair the {name}"),
        format!("fix my {name}"),
        format!("repair my {name}"),
        format!("repair on the {name}"),
        format!("fix on the {name}"),
        format!("repair on a {name}"),
        format!("{name} repair"),
        format!("{name} repairs"),
        format!("{name} fix"),
        format!("get the {name} fixed"),
        format!("get my {name} fixed"),
        format!("get {article} {name} fixed"),
        format!("get the {name} repaired"),
        format!("get my {name} repaired"),
        format!("have the {name} fixed"),
        format!("have my {name} fixed"),
        format!("have {article} {name} fixed"),
        format!("have the {name} repaired"),
        format!("have my {name} repaired"),
        format!("have {article} {name} repaired"),
        format!("{name} fixed"),
        format!("{name} repaired"),
        format!("{name} looked at"),
        format!("look at the {name}"),
        format!("look at my {name}"),
    ]);
    out
}

/// `time_entries(*keys)`: the how-long rows for the jobs named, from `JOB_TIMES`.
pub fn time_entries(keys: &[&str]) -> Vec<Entry> {
    let g = |n: &str| data::get(&format!("bot.knowledge.costs.{n}"));
    keys.iter()
        .map(|key| {
            let job = g("JOB_TIMES").get(key).unwrap_or_else(|| panic!("KeyError: {key:?}"));
            let mut names = job.get("names").unwrap_or_else(|| panic!("KeyError: 'names'")).strs();
            match job.get("bare") {
                Some(Value::Str(s)) => names.push(s.to_string()),
                Some(other) => names.extend(other.strs()),
                None => {}
            }
            let bias = job.get("bias").and_then(Value::as_int).unwrap_or(0).max(3);
            built::entry(
                &format!("time-{key}"),
                vec![built::part(&names), built::part(&g("_TIME").strs())],
                Reply::Text(job.get("reply").map(Value::str).unwrap_or_else(|| panic!("KeyError: 'reply'")).to_string()),
                &[g("_A_WAIT").clone(), g("_A_CAREER").clone(), g("A_AGE_OWNED").clone(), g("_DIY").clone(), job.get("avoid").cloned().unwrap_or(Value::None)],
                bias,
                vec![],
                None,
            )
        })
        .collect()
}

#[cfg(test)]
mod factory_tests {
    #[test]
    fn time_rows_rebuild() {
        crate::data::load();
        let rows = super::time_entries(&["tyres", "car-service"]);
        crate::knowledge::built::assert_rows_match(&rows, "bot.knowledge.cars.ENTRIES");
        crate::knowledge::built::assert_rows_match(&super::time_entries(&["house-survey"]), "bot.knowledge.job_times.ENTRIES");
    }
}
