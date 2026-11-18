//! Port of `bot/knowledge/money.py`: saving, borrowing, bills, housing and tax, in everyday plain
//! advice.
//!
//! The entries (and every module-level table, regex and reply text) are data from the bundle;
//! this file is the logic: the entries' guards and replies, and the arithmetic they share
//! (take-home pay, stamp duty bands, months to a goal, months to clear a card, ...).

use crate::data::{self, FnRef, Value};
use crate::knowledge::d2_shared as sh;
use crate::knowledge::d2dup_money as dup;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
// Module data, by Python name

macro_rules! re_g {
    ($($n:ident),* $(,)?) => {$(
        static $n: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.knowledge.money.", stringify!($n))));
    )*};
}
macro_rules! strs_g {
    ($($n:ident),* $(,)?) => {$(
        static $n: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.knowledge.money.", stringify!($n))));
    )*};
}
macro_rules! str_g {
    ($($n:ident),* $(,)?) => {$(
        static $n: LazyLock<String> =
            LazyLock::new(|| data::string(concat!("bot.knowledge.money.", stringify!($n))));
    )*};
}
/// A regex written inline in a function: compiled once, where it is used.
macro_rules! rx {
    ($p:expr) => {
        rx!($p, 0)
    };
    ($p:expr, $f:expr) => {{
        static R: LazyLock<Re> = LazyLock::new(|| Re::new($p, $f));
        &*R
    }};
}

fn int_g(name: &str) -> i64 {
    data::get(&format!("bot.knowledge.money.{name}")).as_int().unwrap_or_else(|| panic!("{name} is not an int"))
}

re_g!(
    _WHAT_IT_IS, _WHICH_OR_HOW_MUCH, _SOMETHING_TO_CARRY, _ASKS_WHAT_THE_WORD_MEANS, _A_JUDGEMENT_WORD,
    _A_TABLE_WORD, _HOME_IN_THE_LINE, _SALARY_NUMBER, _MY_PAY, _ASKS_THE_PRICE, _A_THE_HOUSEHOLD,
    _A_CAR_OR_A_DEAL, _A_PAYMENT_GONE_WRONG, _A_FIGURE_FOR_A_PLAN, _A_PLANNING_TO_MOVE, _OWING,
    _AGE_FRAME, _MONEY_AFTER, _AGE_AFTER, CONSOLIDATE, HOW_LONG, INCOME_BEFORE, INCOME_AFTER,
    INCOME_ON_K, PRICE_NEAR,
);
strs_g!(_A_CHILDS_POT, _PENSION_SAFE, GOING_OUT, GOING_IN);
str_g!(_THE_SURVEY_ITSELF, _SURVEY_ADVICE, _MONEY, BETWEEN_THE_SUM_AND_THE_RATE);

static _PERSONAL_ALLOWANCE: LazyLock<f64> = LazyLock::new(|| int_g("_PERSONAL_ALLOWANCE") as f64);
static _NI_MAIN_TO: LazyLock<f64> = LazyLock::new(|| int_g("_NI_MAIN_TO") as f64);
static _TAX_TOP: LazyLock<f64> =
    LazyLock::new(|| data::get("bot.knowledge.money._TAX_TOP").as_f64().expect("_TAX_TOP"));
/// (width, rate) per band.
static _TAX_BANDS: LazyLock<Vec<(f64, f64)>> = LazyLock::new(|| {
    data::get("bot.knowledge.money._TAX_BANDS")
        .items()
        .iter()
        .map(|b| (b.items()[0].as_f64().unwrap(), b.items()[1].as_f64().unwrap()))
        .collect()
});

/// `re.compile(_MONEY)`: a sum, with "k" / "grand" / "thousand" after it.
static MONEY_RE: LazyLock<Re> = LazyLock::new(|| Re::new(&_MONEY, 0));
/// A sum, the words that may sit between, and the period that makes it a rate (group 3).
static A_RATE: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        &format!("{}{}{}", *_MONEY, *BETWEEN_THE_SUM_AND_THE_RATE, r"\s*(?:a|per|each|every)\s*(month|week|fortnight|year)"),
        0,
    )
});
static RATE_WORDS: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        &format!("{}{}", *BETWEEN_THE_SUM_AND_THE_RATE, r"\s*(?:a|per|each|every|/)\s*(?:month|week|fortnight|year|day)\b"),
        pyre::I,
    )
});

// ---------------------------------------------------------------------------------------------
// Small shared readers

/// `str(clean or text or "")`.
fn clean_or_text(u: &Understood) -> &str {
    sh::or(&u.clean, &u.text)
}

/// `str(clean or text or "").lower()`.
fn said_lower(u: &Understood) -> String {
    clean_or_text(u).to_lowercase()
}

/// `" ".join(str(one) for one in words).lower()`.
fn words_lower(u: &Understood) -> String {
    u.words.join(" ").to_lowercase()
}

/// `text[at:at + n]` with `at` a byte offset and `n` characters (Python's slice after a match).
fn chars_after(text: &str, at: usize, n: usize) -> &str {
    let rest = &text[at..];
    let end = rest.char_indices().nth(n).map(|(b, _)| b).unwrap_or(rest.len());
    &rest[..end]
}

/// `text[max(0, at - n):at]` with `at` a byte offset and `n` characters.
fn chars_before(text: &str, at: usize, n: usize) -> &str {
    let head = &text[..at];
    let count = head.chars().count();
    if count <= n {
        return head;
    }
    let start = head.char_indices().nth(count - n).map(|(b, _)| b).unwrap_or(0);
    &head[start..]
}

/// `format(n, ",")` for an int.
fn comma(n: i64) -> String {
    text::thousands(n)
}

/// `"{:,.0f}".format(x)`.
fn comma_f0(x: f64) -> String {
    let s = format!("{:.0}", x);
    let (sign, digits) = match s.strip_prefix('-') {
        Some(d) => ("-", d),
        None => ("", s.as_str()),
    };
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    format!("{sign}{out}")
}

/// Python `int(x)` on a float (towards zero).
fn int(x: f64) -> i64 {
    x.trunc() as i64
}

/// Python `float("1,234.5".replace(",", ""))` on a matched figure.
fn figure(raw: &str) -> f64 {
    raw.replace(',', "").parse().unwrap_or(0.0)
}

// DUP(bot.memory.history::the_lines_before)
/// What they said in the last `look_back` turns, oldest first, empties dropped.
fn the_lines_before(memory: &Memory, look_back: usize) -> Vec<String> {
    let turns = &memory.turns;
    let from = turns.len().saturating_sub(look_back);
    turns[from..].iter().map(|t| text::strip(&t.user).to_string()).filter(|l| !l.is_empty()).collect()
}

/// The user lines of the turns, newest first (`reversed(memory.get("turns") or [])`).
fn user_lines_newest_first(memory: &Memory) -> impl Iterator<Item = &str> {
    memory.turns.iter().rev().map(|t| t.user.as_str())
}

// ---------------------------------------------------------------------------------------------
// The survey, take-home pay, food bills

/// The levels when the line asks what or which or how much, the advice when it asks whether.
pub fn _the_survey(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = text::squash(&understood.clean.to_lowercase());
    if _WHAT_IT_IS.is_match(&clean) || _WHICH_OR_HOW_MUCH.is_match(&clean) {
        return Some(_THE_SURVEY_ITSELF.clone());
    }
    Some(_SURVEY_ADVICE.clone())
}

/// Keep the pay rows off the allowance question, which wears the same words.
pub fn _not_a_customs_line(understood: &Understood) -> bool {
    let text = understood.clean.as_str();
    if _SOMETHING_TO_CARRY.is_match(text) {
        return false;
    }
    if rx!(r"\bpayslips?\b|\bpay\s+slips?\b|\bnet\s+pay\b", pyre::I).is_match(text) {
        return true;
    }
    _ASKS_WHAT_THE_WORD_MEANS.is_match(text)
}

/// True when the line is someone introducing themselves, so the welcome is the reply.
pub fn _a_first_meeting(text: &str, understood: &Understood) -> bool {
    if understood.kind == "question" {
        return false;
    }
    dup::extract_name(text).is_some_and(|n| !n.is_empty())
}

/// True when the line asks whether splitting a table's bill is rude, and names no home.
pub fn _a_manners_question_not_a_home_split(understood: &Understood) -> bool {
    let mut text = words_lower(understood);
    if text.is_empty() {
        text = said_lower(understood);
    }
    if _HOME_IN_THE_LINE.is_match(&text) {
        return false;
    }
    _A_JUDGEMENT_WORD.is_match(&text) && _A_TABLE_WORD.is_match(&text)
}

/// The salary a line names, in pounds, or None when it names none.
pub fn _salary_in(text: &str) -> Option<i64> {
    let mut best: Option<f64> = None;
    for m in _SALARY_NUMBER.finditer(text) {
        let raw = m.str(1).replace(',', "");
        let Ok(mut value) = raw.parse::<f64>() else { continue };
        if m.get(2).is_some() {
            value *= 1000.0;
        } else if value < 3000.0 {
            continue;
        }
        if value >= 1000.0 && best.is_none_or(|b| value > b) {
            best = Some(value);
        }
    }
    match best {
        Some(b) if b != 0.0 => Some(int(b)),
        _ => None,
    }
}

/// (income tax, National Insurance) on a salary for one person, England and Wales.
pub fn _income_tax_and_ni(income: f64) -> (f64, f64) {
    let mut allowance = *_PERSONAL_ALLOWANCE;
    if income > 100000.0 {
        allowance = f64::max(0.0, allowance - (income - 100000.0) / 2.0);
    }
    let taxable = f64::max(0.0, income - allowance);
    let (mut tax, mut left) = (0.0, taxable);
    for &(width, rate) in _TAX_BANDS.iter() {
        let part = f64::min(left, width);
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
    if income > *_PERSONAL_ALLOWANCE {
        ni += (f64::min(income, *_NI_MAIN_TO) - *_PERSONAL_ALLOWANCE) * 0.08;
    }
    if income > *_NI_MAIN_TO {
        ni += (income - *_NI_MAIN_TO) * 0.02;
    }
    (tax, ni)
}

/// The salary an earlier line gave as their own pay, or 0.
pub fn _a_salary_they_said_before(memory: &Memory) -> i64 {
    for line in the_lines_before(memory, 8).iter().rev() {
        let low = line.to_lowercase();
        if !_MY_PAY.is_match(&low) {
            continue;
        }
        if !_SALARY_NUMBER.finditer(&low).iter().any(|m| _is_income(&low, m)) {
            continue;
        }
        if let Some(income) = _salary_in(&low) {
            if income != 0 {
                return income;
            }
        }
    }
    0
}

/// What a salary leaves in the bank, band by band.
pub fn _take_home(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = said_lower(understood);
    let mut income = _salary_in(&text).unwrap_or(0);
    if income == 0 {
        income = _a_salary_they_said_before(memory);
    }
    if income == 0 {
        return Some("Income tax starts above £12,570 a year: 20 per cent of it up to £50,270, then 40 per cent, and National Insurance is 8 per cent above £12,570. Tell me the salary and I'll work out the take-home.".to_string());
    }
    let (tax, ni) = _income_tax_and_ni(income as f64);
    let take = income as f64 - tax - ni;
    let bands = if take > 0.0 && income <= 50270 {
        "That is the first £12,570 tax free, 20 per cent of the next £37,700"
    } else {
        "That is the first £12,570 tax free, 20 per cent up to £50,270 and 40 per cent above it"
    };
    Some(format!(
        "On £{} a year you pay about £{} income tax and £{} National Insurance, so you take home roughly £{} a month. {}, and National Insurance at 8 per cent on pay above £12,570.",
        comma(income),
        comma_f0(tax),
        comma_f0(ni),
        comma_f0(take / 12.0),
        bands
    ))
}

/// A week named in the line, in the shapes people name it.
pub fn _a_week_asked(text: &str) -> bool {
    rx!(r"\b(?:per|a|each|every)\s+week\b|\bweekly\b|\bin a week\b|\ba week\b|\bweek for food\b").is_match(text)
}

/// A month named in the line, in the shapes people name it.
pub fn _a_month_asked(text: &str) -> bool {
    rx!(r"\b(?:per|a|each|every)\s+month\b|\bmonthly\b|\ba month\b|\bin a month\b").is_match(text)
}

/// A cost question gets the going rate first; a saving question gets the plan.
pub fn _food_bill(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = said_lower(understood);
    let s = |x: &str| Some(x.to_string());
    if _ASKS_THE_PRICE.is_match(&text) && _a_month_asked(&text) {
        if _A_THE_HOUSEHOLD.is_match(&text) {
            return s("A couple or a small family spends about 300 to 400 pounds a month on food shopping, and about 450 to 550 with lunches and takeaways in it. Frozen veg, tinned pulses and own-brand staples are where the month comes down.");
        }
        return s("A single adult spends about 130 to 195 pounds a month on food cooked at home, so 30 to 45 a week, and about 215 to 300 a month once lunches and a takeaway are in it.");
    }
    if rx!(r"\b(?:per|a|each|every)\s+day\b|\bdaily\b").is_match(&text) {
        return s("Budget about 10 to 15 pounds a day per person if you cook for yourself and 25 to 35 eating out or taking away, so a week self-catering lands near 70 to 100 pounds a head.");
    }
    if _a_week_asked(&text) && !rx!(r"\bhow do i\b|\bhow can i\b|\bhow to\b").is_match(&text) {
        if _A_THE_HOUSEHOLD.is_match(&text) {
            return s("A couple or a family of four spends about 70 to 80 pounds a week on food shopping, from about 55 for two cooking from scratch, and about 100 to 130 once lunches and a takeaway are in it. A list once a week is what keeps it there.");
        }
        return s("A single adult spends about 30 to 45 pounds a week on food cooked at home, and about 50 to 70 with lunches and a takeaway. A list once a week and two meat-free dinners is the quickest way to land at the bottom of that.");
    }
    if _ASKS_THE_PRICE.is_match(&text) {
        if _A_THE_HOUSEHOLD.is_match(&text) {
            return s("A couple or a family of four spends about 70 to 80 pounds a week on food shopping, from about 55 for two cooking from scratch, and about 100 to 130 once lunches and a takeaway are in it. A list once a week is what keeps it there.");
        }
        return s("A single adult spends about 30 to 45 pounds a week on food cooked at home, so 130 to 195 pounds a month. Add about 20 a week if lunches and a takeaway are in it, and a written list once a week is what keeps the shop at the bottom end.");
    }
    s("Plan the week around what's already in the cupboard, shop with a list once a week, and buy own-brand staples (pasta, rice, tins, oats, frozen veg) where the difference is invisible. Cutting meat to two or three dinners a week is the biggest single saving for most households.")
}

/// The habit answer has to lead with the thing the person named.
pub fn _habit_plan(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = said_lower(understood);
    let night = "Keep one planned night a week for a takeaway and put the money aside for it, so the rest of the week has nothing to decide.";
    let flask = "Take a flask and a packed lunch in your bag the night before, so the shop stop or the lunch queue has nothing to decide.";
    let rule = "Put a 24-hour rule on the apps: take the saved card off, so ordering takes effort.";
    let week = "A normal week measured first is what makes the total real.";
    if rx!(r"\bcoffee|\bcoffees\b|\blunch(es)?\b|\bbrew\b|\bsandwich").is_match(&text) {
        return Some([flask, rule, week].join(" "));
    }
    let tail = format!(
        "For the shop coffee, a flask and a packed lunch in your bag the night before does the same job, and {}{}",
        week[..1].to_lowercase(),
        &week[1..]
    );
    Some([night, rule, &tail].join(" "))
}

/// A "how much can i put in" question gets the cap first; a "what is it" gets the wrapper.
pub fn _isa(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = said_lower(understood);
    let s = |x: &str| Some(x.to_string());
    if rx!(r"\bhow many isas\b|\bmore than one isa\b|\btwo isas\b|\bseveral isas\b|\banother isa\b|\ba second isa\b|\bopen a second\b|\bcash isa and\b|\bisa and a\b|\bboth\b|\bsame year\b").is_match(&text) {
        return s("As many as you like: there is no limit on the number of ISAs you hold, only on what goes into them - £20,000 a year across all of them together in 2025-26. In one tax year you can pay into one cash ISA and one stocks and shares ISA (plus a lifetime ISA), and money you move from one provider to another does not use any allowance at all. So a second account is fine; the shared £20,000 is the number to watch.");
    }
    if rx!(r"\b(?:allowance|limit|how much|put in|pay in|pay into|a year|per year|reset)\b").is_match(&text) {
        return s("You can pay in up to £20,000 a year across your ISAs in 2025-26, and the allowance resets each April - check gov.uk for the current figure before you rely on it. Inside the ISA the interest and growth are not taxed, so a cash ISA only beats an ordinary savings account if it pays a better rate.");
    }
    if rx!(r"\blifetime isa\b|\blisa\b").is_match(&text) {
        return s("A Lifetime ISA pays a 25% bonus on what you put in - up to £1,000 a year on £4,000 - and the interest and growth are not taxed. It is only worth it if the money is for a first home under £450,000, or kept until you are 60; take it out before then for anything else and you lose the bonus and pay a 25% charge, so money you might need sooner belongs in a plain ISA or a savings account.");
    }
    if rx!(r"\bopen\b|\bget (?:an|a) isa\b|\bhow do i start\b|\bset up\b").is_match(&text)
        && !rx!(r"\bshould\b|\bworth\b|\bbetter\b|\bor\b").is_match(&text)
    {
        return s("Apply online with a bank, building society or investment platform: you need your national insurance number, your date of birth and a debit card, and it takes about ten minutes with the account open the same day. Pick cash if you want it safe and might need it, stocks and shares if it is for five years or more. To move an ISA you already have, ask the new provider for a transfer form - never withdraw it and pay it back in, because that uses up the year's allowance.");
    }
    if rx!(r"\b(?:what|whats|how does|explain|mean|explanation)\b").is_match(&text)
        && !rx!(r"\b(?:worth|should|better|or)\b").is_match(&text)
    {
        return s("An ISA is a savings or investment account where the interest and growth are not taxed. A cash ISA is only worth it if it pays more than an ordinary savings account, and money you won't touch for five years or more usually grows more in a stocks and shares ISA; a lifetime ISA adds 25% (up to £1,000 a year) towards a first home or retirement.");
    }
    s("A cash ISA is only worth it if it pays more than an ordinary savings account, and for small amounts the tax-free part barely matters. Money you won't touch for five years or more usually grows more in a stocks and shares ISA, and a Lifetime ISA adds 25% (up to £1,000 a year) towards a first home or retirement.")
}

/// The money question behind "i spend too much".
pub fn _over_spending(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = said_lower(understood);
    if rx!(r"\bcoffee|\bcoffees\b|\blunch(es)?\b|\btakeaway|\btakeaways\b|\beating out\b|\bdeliveroo\b|\bjust eat\b|\bfast food\b").is_match(&text) {
        return _habit_plan(memory, understood);
    }
    Some("Write down every purchase for a month, or let your banking app sort it into categories - the surprises are nearly always takeaways, subscriptions and small card taps. Then decide the figure before the month starts and move it out on payday, because a number set in advance beats willpower every evening.".to_string())
}

/// Who a late or cancelled journey is claimed from: the mode decides it.
pub fn _late_leg(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = said_lower(understood);
    let s = |x: &str| Some(x.to_string());
    if rx!(r"\b(?:ferry|ferries|sea\s+crossing|ferry\s+company|stena|brittany\s+ferries)\b").is_match(&text) {
        return s("A ferry to or from an EU port owes you 25% of the fare if you arrive more than an hour late and 50% if it is more than two hours, and a cancelled crossing is your choice of a refund or a rerouting. Ask the operator in writing within three months and keep the booking and the boarding pass; if they will not pay, their complaints procedure comes first, then the port authority or a consumer body.");
    }
    if rx!(r"\b(?:bus|buses|coach|coaches|megabus|national\s+express|stagecoach|arriva|first\s+bus|tram|trams)\b").is_match(&text) {
        return s("Claim it from the bus or coach operator, not a train scheme: they run their own promise, so send the claim on their website or by post with the ticket, the route and the time. If a bus did not turn up at all, ask for that fare back and complain to the local transport authority if they say no; a coach cancelled or over three hours late is a full refund or a replacement journey, and a missed connection on one ticket is theirs to sort out.");
    }
    if rx!(r"\b(refund|refunds|money back|unused|didn'?t use|didnt use|not use|never used|can'?t go|cant go)\b").is_match(&text)
        && !rx!(r"\b(delay|delayed|delays|late|later|missed|cancel+ed|cancel+ation|strike|compensation)\b").is_match(&text)
    {
        return s("Two different claims, so ask for the right one: a ticket you did not use can be refunded before travel on most fares, less an admin fee of up to about £10, though advance tickets are often non-refundable and only changeable. If the train itself was late or cancelled, that is Delay Repay instead - 15 minutes late on most operators, claimed on their site within 28 days with the ticket and the time you arrived.");
    }
    s("Claim Delay Repay from the train company, not the ticket seller: most pay out for 15 minutes or more late, and the claim goes in within 28 days on the operator's website with a photo of the ticket. A season ticket is worked out pro rata, so keep a note of each delay. If the train was cancelled and you did not travel, you are entitled to a full refund, and you can claim a refund of the unused part of a flexible ticket at the ticket office.")
}

/// What a car costs to run, with the charging figures when the car is electric.
pub fn _car_running_cost(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = understood.clean.as_str();
    if ["electric", " ev", "ev ", "charged", "charging"].iter().any(|w| text.contains(w)) {
        return Some("Charged at home an electric car is about 3-4p a mile, so £300-£500 a year for 8,000 miles, and public rapid chargers cost three or four times that. Insurance and tyres are like any other car, servicing is cheaper with no oil or exhaust, and road tax is £195 now that the exemption has ended.".to_string());
    }
    Some("For a small used car, budget about £2,000-£3,000 a year all in: insurance around £500, fuel £80-£120 a month, road tax from £195, and £400-£600 for MOT, servicing and tyres. Get an insurance quote before you buy the car, because that one number can decide the whole budget.".to_string())
}

/// False when the line asks about paying monthly with no vehicle and no deal in it.
pub fn _a_car_or_a_deal(understood: &Understood) -> bool {
    _A_CAR_OR_A_DEAL.is_match(clean_or_text(understood))
}

/// True unless the line is working out what it can pay, which is a plan and not arrears.
pub fn _a_figure_for_a_plan_is_not_a_debt(understood: &Understood) -> bool {
    let mut said = words_lower(understood);
    if said.is_empty() {
        said = said_lower(understood);
    }
    if said.is_empty() {
        return true;
    }
    if _A_PAYMENT_GONE_WRONG.is_match(&said) {
        return true;
    }
    !(_A_FIGURE_FOR_A_PLAN.is_match(&said) || _A_PLANNING_TO_MOVE.is_match(&said))
}

/// The other half of the same read: the rent question, when nothing has gone wrong yet.
pub fn _a_plan_and_no_trouble(understood: &Understood) -> bool {
    !_a_figure_for_a_plan_is_not_a_debt(understood)
}

/// Paying into a pension that is not your own: an adult's, or a child's junior SIPP.
pub fn _pension_for_someone(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let mut text = understood.clean.to_lowercase();
    if text.is_empty() {
        text = words_lower(understood);
    }
    if sh::any_in(&text, &_A_CHILDS_POT) {
        return Some("A junior SIPP is the one for a child: you open it in their name, 2,880 pounds a year from you becomes 3,600 with the 20% top-up, that is the most anyone can put in for them in a year, and the money stays locked until they are 57 - so it is a long cheap start rather than a savings account. Grandparents can pay in too, and the child owns it from eighteen.".to_string());
    }
    Some("Yes. Anyone can pay into someone else's pension - a husband or wife, a partner, a parent - and the tax relief goes to the person whose pension it is, so 2,880 pounds from you becomes 3,600 in their pot, which is the most that can go in without their earnings. A basic-rate taxpayer has nothing more to claim; a higher earner claims the rest back through self assessment. If they are under 18 the pot is a junior SIPP, locked until they are 57.".to_string())
}

/// A pension question is one of three: is the money safe, when can I get at it, or should I pay
/// into one at all.
pub fn _pension(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let joined = understood.words.join(" ");
    let text = sh::or3(&understood.clean, &understood.text, &joined).to_lowercase();
    if sh::any_in(&text, &_PENSION_SAFE) {
        return Some("In a defined-benefit (final salary) scheme the promise is the employer's, and the Pension Protection Fund steps in for most of it if the company fails. Money in a defined-contribution pot - the usual workplace kind - is yours and held apart from the company, so the firm going bust does not touch it; what it is worth moves with the funds you chose, and cash in a fund is not the same as cash in the bank. Check the yearly statement for the scheme's name and value, and a free Pension Wise appointment before you move anything.".to_string());
    }
    if rx!(r"\b(?:take|taken|taking|access|cash in|withdraw|withdrawn|draw|early|lump sum|25%|quarter|before i retire|when can i|how early)\b").is_match(&text) {
        return Some("Private pension money can usually be taken from 55, rising to 57 for most people from April 2028, and your scheme itself may set a later age. You get a quarter of it tax free and the rest is taxed as income as you draw it, so take the smallest amount you need rather than the lot: what is left has to last longer than you think, and it is usually better to live on savings first if you can.".to_string());
    }
    Some("A workplace pension is free money: you put in 5% and your employer at least 3%, with tax relief on top. Don't opt out to add a few pounds to your pay packet now, and once a year check which fund your pot is invested in - the default is often too cautious if retirement is decades away.".to_string())
}

/// True when a recent line said the person asking is the one who owes the money.
pub fn _they_owe(memory: &Memory) -> bool {
    let mut seen = 0;
    for said in user_lines_newest_first(memory) {
        if said.is_empty() {
            continue;
        }
        seen += 1;
        if _OWING.is_match(said) {
            return true;
        }
        if seen >= 3 {
            break;
        }
    }
    false
}

// ---------------------------------------------------------------------------------------------
// Sums, rates and goals

/// True when the figure at `m` is a person's age rather than a sum of money.
pub fn _an_age(text: &str, m: &Match) -> bool {
    let after = chars_after(text, m.end(), 12);
    if _MONEY_AFTER.match_(after).is_some() {
        return false;
    }
    _AGE_FRAME.is_match(&text[..m.start()]) || _AGE_AFTER.match_(after).is_some()
}

/// Every sum the line names, with an age left out: (value, match) pairs.
pub fn _sums_in(text: &str) -> Vec<(f64, Match<'_>)> {
    let mut out = Vec::new();
    for m in MONEY_RE.finditer(text) {
        if _an_age(text, &m) {
            continue;
        }
        let mut value = figure(m.str(1));
        if m.get(2).is_some() {
            value *= 1000.0;
        }
        out.push((value, m));
    }
    out
}

/// The clause the figure sits in, so its direction can be read on its own (byte offsets).
pub fn _clause_about(text: &str, start: usize, end: usize) -> String {
    let left: i64 = [".", ";", ",", "?"]
        .iter()
        .map(|mark| text[..start].rfind(mark).map(|i| i as i64).unwrap_or(-1))
        .max()
        .unwrap();
    let right: Vec<usize> = [".", ";", ",", "?"].iter().filter_map(|mark| text[end..].find(mark).map(|i| i + end)).collect();
    let stop = right.iter().min().copied().unwrap_or(text.len());
    let from = (left + 1) as usize;
    let clause = if from < stop { text[from..stop].to_lowercase() } else { String::new() };
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

/// The rate a matched figure is said in (group 3 of `A_RATE`), per month.
fn per_month_factor(unit: &str) -> f64 {
    match unit {
        "month" => 1.0,
        "week" => 52.0 / 12.0,
        "fortnight" => 26.0 / 12.0,
        "year" => 1.0 / 12.0,
        _ => panic!("KeyError: {unit}"),
    }
}

/// What the line puts aside each month, in pounds: "200 a month", "£50 a week" -> £217.
pub fn _per_month(text: &str) -> i64 {
    let Some(m) = A_RATE.search(text) else { return 0 };
    let clause = _clause_about(text, m.start(), m.end());
    if sh::any_in(&clause, &GOING_OUT) && !sh::any_in(&clause, &GOING_IN) {
        return 0;
    }
    text::round_half_even(_said_a_month(&m) * per_month_factor(m.str(3)))
}

/// The sum the user named, in pounds, before it is turned into a month.
pub fn _said_a_month(m: &Match) -> f64 {
    let amount = figure(m.str(1));
    if m.get(2).is_some() { amount * 1000.0 } else { amount }
}

/// `str(x or "")` for an int.
fn int_or_blank(x: i64) -> String {
    if x == 0 { String::new() } else { x.to_string() }
}

/// The biggest sum the line names that is not the rate the line puts aside.
pub fn _target(text: &str) -> i64 {
    let monthly = int_or_blank(_per_month(text));
    let (mut best, mut biggest) = (0i64, 0f64);
    for (amount, m) in _sums_in(text) {
        if _rate_words(chars_after(text, m.end(), 24)) {
            continue;
        }
        if int(amount).to_string() != monthly && amount > biggest {
            biggest = amount;
            best = int(amount);
        }
    }
    best
}

/// True when the words after a figure make it a rate: "a month", "spare a week".
pub fn _rate_words(after: &str) -> bool {
    RATE_WORDS.match_(after).is_some()
}

/// Months to reach `target` at `amount` a month, rounded up.
pub fn _months(amount: i64, target: i64) -> i64 {
    if amount != 0 { -((-target).div_euclid(amount)) } else { 0 }
}

/// £200 a month, or "£217 a month (£50 a week)" when the line named another unit.
pub fn _a_month_in_words(text: &str, amount: i64) -> String {
    let m = A_RATE.search(text);
    let monthly = format!("£{} a month", comma(amount));
    let Some(m) = m else { return monthly };
    if m.str(3) == "month" {
        return monthly;
    }
    let unit = match m.str(3) {
        "fortnight" => "a fortnight",
        "year" => "a year",
        _ => "a week",
    };
    let said = format!("£{} {}", comma(int(_said_a_month(&m))), unit);
    format!("{monthly} ({said})")
}

/// A number of months as people say it: 1 month, 18 months, 2 years and 3 months.
pub fn _how_long(months: i64) -> String {
    let (years, rest) = (months.div_euclid(12), months.rem_euclid(12));
    if months <= 1 {
        return "a single month at that rate".to_string();
    }
    if months < 24 {
        return format!("{months} months");
    }
    if rest == 1 {
        return format!("{months} months, about {years} years and a month");
    }
    if rest != 0 {
        return format!("{months} months, about {years} years and {rest} months");
    }
    format!("{months} months, {years} years")
}

/// "Should I take a loan out to pay this off" and "is overpaying worth it" are two questions.
pub fn _pay_off(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = clean_or_text(understood);
    let s = |x: &str| Some(x.to_string());
    if CONSOLIDATE.is_match(line) {
        return s("Only if the new rate is clearly lower and the card stops being used: a consolidation loan on a card you keep spending on just moves the debt and adds interest. Check the total repayable rather than the monthly payment, and ask for the settlement figure on what you owe now, since paying early can carry up to 58 days' extra interest or an early repayment charge.");
    }
    if rx!(r"\bmortgages?\b|\bremortgag", pyre::I).is_match(line) {
        return s("Overpaying a mortgage almost always saves money, because the interest you save is your mortgage rate for every year left on the term, and no savings account pays that after tax. Check the early repayment charge first: most fixes let you overpay 10% of the balance a year without one, and above that the fee can cost more than the interest you save. Ask the lender to take the overpayment off the term rather than the monthly payment unless the lower payment is what you need, and keep three to six months of bills in savings before you start, since an overpayment cannot be taken back.");
    }
    if rx!(r"\bloans?\b|\bfinance agreement\b|\bcar finance\b", pyre::I).is_match(line) {
        return s("Check the settlement figure before you overpay a loan: many carry an early repayment charge, often 58 days' extra interest or a percentage of what is left, and if that is bigger than the interest you would save, keep the money in savings instead. Where there is no charge, overpaying a loan at 8% or more beats any savings account, so it is the better use of spare money after a small buffer.");
    }
    s("Compare the interest you'd save with what the savings earn: a credit card at 20% or more beats any savings account, so clear that first, and check the early repayment charge before overpaying a loan or mortgage (most mortgages allow 10% of the balance a year without one). Keep a small buffer in savings while you do it, so a surprise bill doesn't put you back on the card.")
}

/// Months to clear a balance at a fixed payment, on a stated APR, by the annuity formula.
pub fn _card_months(balance: f64, payment: f64, apr: f64) -> i64 {
    let rate = apr / 100.0 / 12.0;
    if payment <= balance * rate {
        return 0;
    }
    int(-(1.0 - rate * balance / payment).ln() / (1.0 + rate).ln()) + 1
}

/// "How long will it take to pay off my card" wants a date, not the minimum-payment talk.
pub fn _card_payoff(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = clean_or_text(understood);
    let balance = _target(text);
    let payment = _per_month(text);
    let how_long = HOW_LONG.is_match(text);
    if how_long && balance != 0 && payment != 0 {
        let months = _card_months(balance as f64, payment as f64, 20.0);
        if months != 0 {
            return Some(format!(
                "£{} at 20% APR on £{} a month is {}, and about £{} in total, mostly interest. Every extra £50 a month comes off the end of that, so a pay rise or a balance transfer is worth more here than almost anywhere else.",
                comma(balance),
                comma(payment),
                _how_long(months),
                comma(payment * months)
            ));
        }
    }
    if how_long && balance != 0 {
        let floor = i64::max(25, text::round_half_even(balance as f64 * 0.025));
        let (slow, quick) = (_card_months(balance as f64, floor as f64, 20.0), _card_months(balance as f64, 100.0, 20.0));
        if slow != 0 && quick != 0 {
            return Some(format!(
                "£{} at 20% APR on the minimum, about £{} a month, takes {} and costs about £{}; at £100 a month it is {}. Paying the minimum keeps the debt alive, so fix a standing order for a figure that actually falls.",
                comma(balance),
                comma(floor),
                _how_long(slow),
                comma(floor * slow),
                _how_long(quick)
            ));
        }
    }
    if how_long {
        return Some("At 20% APR a card barely moves on the minimum, which is 1-3% of the balance: £1,000 on £25 a month takes five and a half years and costs £1,675, and £50 a month clears it in just over two. Tell me the balance, the APR and what you can pay, and I'll give you the month it ends.".to_string());
    }
    Some("Pay more than the minimum: it's set at 1-3% of the balance and mostly covers interest, so the debt barely moves. List the cards by APR, throw spare money at the dearest one first, and ask about a 0% balance transfer, which buys you a year or more to clear the rest.".to_string())
}

/// The first sum in the line followed by one of these units: ("a year", "per year").
pub fn _sum_and_unit(text: &str, units: &[&str]) -> i64 {
    static SUM_AND_TAIL: LazyLock<Re> = LazyLock::new(|| Re::new(&format!("{}{}", *_MONEY, r"\s*([^.;!?]{0,18})"), 0));
    for m in SUM_AND_TAIL.finditer(text) {
        let tail = format!(" {}", m.str(3).to_lowercase());
        if units.iter().any(|u| tail.contains(u)) {
            let amount = figure(m.str(1));
            return int(if m.get(2).is_some() { amount * 1000.0 } else { amount });
        }
    }
    0
}

/// The sum a match names, in whole pounds.
pub fn _sum_amount(m: &Match) -> i64 {
    let amount = figure(m.str(1));
    int(if m.get(2).is_some() { amount * 1000.0 } else { amount })
}

/// True when the line gives this sum as pay rather than as the price of anything.
pub fn _is_income(text: &str, m: &Match) -> bool {
    let before = chars_before(text, m.start(), 30);
    let after = chars_after(text, m.end(), 16);
    if INCOME_BEFORE.is_match(before) || INCOME_AFTER.is_match(after) {
        return true;
    }
    if !(m.get(2).is_some_and(|g| !g.is_empty()) && INCOME_ON_K.is_match(before)) {
        return false;
    }
    let pieces = rx!(r"[\n.;,]|\band\b|\bbut\b").split(before);
    let clause = pieces.last().map(String::as_str).unwrap_or("");
    !PRICE_NEAR.is_match(clause)
}

/// The biggest sum the line gives as pay, or 0.
pub fn _income_target(text: &str) -> f64 {
    let mut best = 0.0;
    for (amount, m) in _sums_in(text) {
        if _is_income(text, &m) {
            best = f64::max(best, amount);
        }
    }
    best
}

/// The biggest sum the line gives as a price: not a rate, and not pay.
pub fn _price_target(text: &str) -> f64 {
    // Python compares str(amount) (a float: "300000.0") with str(_per_month(text)) (an int, or
    // ""), which never agree; the comparison is kept for the record.
    let monthly = int_or_blank(_per_month(text));
    let mut best = 0.0;
    for (amount, m) in _sums_in(text) {
        if _is_income(text, &m) {
            continue;
        }
        if _rate_words(chars_after(text, m.end(), 24)) {
            continue;
        }
        if text::float_repr(amount) != monthly && amount > best {
            best = amount;
        }
    }
    best
}

/// "How big a deposit on a 300,000 house" wants 10% of that price, not of 200,000.
pub fn _house_deposit(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = clean_or_text(understood);
    let price = _price_target(text);
    if price >= 20000.0 {
        return Some(format!(
            "Aim for 10% of the price: on £{} that is £{}, and 5% is £{}, which costs you noticeably more in interest over the term. A Lifetime ISA adds 25% on up to £4,000 a year towards a first home, and a broker can tell you what you'd actually be offered before you start looking.",
            _comma(price),
            _comma(int(price * 0.10) as f64),
            _comma(int(price * 0.05) as f64)
        ));
    }
    if price >= 1000.0 {
        return None;
    }
    let pay = _income_target(text);
    if pay != 0.0 {
        let (low, high) = (int(pay * 4.0), int(pay * 4.5));
        return Some(format!(
            "That is a salary rather than a price, so the deposit depends on what you buy: a lender will usually offer around four to four and a half times your pay, so £{} is roughly a £{} to £{} mortgage, and 10% of that is a £{} to £{} deposit. Aim for 10% rather than 5% if you can - it gets you noticeably better rates - and a Lifetime ISA adds 25% on up to £4,000 a year towards a first home.",
            _comma(pay),
            _comma(low as f64),
            _comma(high as f64),
            _comma(int(low as f64 * 0.10) as f64),
            _comma(int(high as f64 * 0.10) as f64)
        ));
    }
    Some("Aim for 10% of the price: £20,000 on a £200,000 home, and it gets you noticeably better rates than 5%. A Lifetime ISA adds 25% on up to £4,000 a year towards a first home, and a broker can tell you what you'd actually be offered before you start looking.".to_string())
}

/// `"{:,}".format(int(amount))`.
pub fn _comma(amount: f64) -> String {
    comma(int(amount))
}

// ---------------------------------------------------------------------------------------------
// Stamp duty, LBTT and LTT

/// (low, top or None, rate) per band.
type Bands = Vec<(f64, Option<f64>, f64)>;

fn bands(name: &str) -> Bands {
    data::get(&format!("bot.knowledge.money.{name}"))
        .items()
        .iter()
        .map(|b| {
            let x = b.items();
            (x[0].as_f64().unwrap(), x[1].as_f64(), x[2].as_f64().unwrap())
        })
        .collect()
}

static _SDLT: LazyLock<Bands> = LazyLock::new(|| bands("_SDLT"));
static _LBTT: LazyLock<Bands> = LazyLock::new(|| bands("_LBTT"));
static _LTT: LazyLock<Bands> = LazyLock::new(|| bands("_LTT"));
static _SLBTT_FIRST_NIL: LazyLock<f64> = LazyLock::new(|| int_g("_SLBTT_FIRST_NIL") as f64);

/// The tax on a price, band by band: each rate applies only to the part inside its band.
pub fn _duty(value: f64, bands: &Bands) -> f64 {
    let mut total = 0.0;
    for &(low, top, rate) in bands {
        let top = top.unwrap_or(value);
        total += f64::max(0.0, f64::min(value, top) - low) * rate;
    }
    total
}

/// "2 million" and "1.5m" are prices too, and _target only reads digits.
pub fn _price_named(text: &str) -> i64 {
    let mut price = _target(text);
    for m in rx!(r"\b(\d+(?:\.\d+)?)\s*(million|m)\b").finditer(text) {
        let amount = int(figure(m.str(1)) * 1000000.0);
        if amount > price {
            price = amount;
        }
    }
    price
}

/// A price in the line is the price: the tax on a 250,000 home is nothing, not a deposit.
pub fn _stamp_duty(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = clean_or_text(understood);
    let price = _price_named(text) as f64;
    let first = rx!("first[- ]time|first home|first house|first flat|first property|first place|our first").is_match(text);
    let second = rx!("second home|buy[- ]to[- ]let|buy to let|second property|additional home|additional dwelling|investment property|second house|rental property|landlord").is_match(text);
    if rx!("scotland|scottish|lbtt|land and buildings|edinburgh|glasgow|aberdeen|dundee|inverness").is_match(text) {
        return Some(_scottish_duty(price, first, second));
    }
    if rx!(r"wales|welsh|\bltt\b|land transaction tax|cardiff|swansea|newport").is_match(text) {
        return Some(_welsh_duty(price, first, second));
    }
    Some(_english_duty(price, first, second))
}

/// A tax figure: "Nothing to pay" rather than "0", and thousands separated.
pub fn _duty_said(amount: f64) -> String {
    if amount <= 0.0 {
        "Nothing to pay".to_string()
    } else {
        format!("£{}", _comma(text::round_half_even(amount) as f64))
    }
}

/// SDLT, England and Northern Ireland, from April 2025.
pub fn _english_duty(price: f64, first: bool, second: bool) -> String {
    let first_time = |value: f64| -> f64 {
        if value > 500000.0 {
            return _duty(value, &_SDLT);
        }
        f64::max(0.0, value - 300000.0) * 0.05
    };
    let priced = if price >= 20000.0 { format!("£{}", _comma(price)) } else { String::new() };
    if price >= 20000.0 && second {
        let extra = price * 0.05;
        let normal = _duty(price, &_SDLT);
        let made = if normal <= 0.0 {
            "nothing to pay in the normal bands, so the whole bill is the 5% surcharge on the price".to_string()
        } else {
            format!(
                "{} of it is the normal bands and {} is the 5% surcharge, which is 5 percentage points on every band from the first pound",
                _duty_said(normal),
                _duty_said(extra)
            )
        };
        return format!(
            "{} on a {} second home: {}. It is usually paid within 14 days of completion and reclaimed if you sell the old home within three years. Check GOV.UK before you exchange, because the bands move with budgets.",
            _duty_said(normal + extra),
            priced,
            made
        );
    }
    if price >= 20000.0 && first {
        if price > 500000.0 {
            return format!(
                "{} on a {} first home: the first-time relief stops above £500,000, so this pays the mover bill - nothing to £250,000, 5% to £925,000, 10% to £1.5m and 12% above that. A second home or a buy-to-let adds 5% to every band. Check GOV.UK before you exchange, because the bands move with budgets.",
                _duty_said(_duty(price, &_SDLT)),
                priced
            );
        }
        return format!(
            "{} on a {} first home: a first-time buyer pays nothing up to £300,000 and 5% on the part from £300,000 to £500,000, so the bill is the part above £300,000 at 5%. A home mover would pay {} on the same price, and above £500,000 the relief goes and this becomes the mover bill.",
            _duty_said(first_time(price)),
            priced,
            _duty_said(_duty(price, &_SDLT))
        );
    }
    if price >= 20000.0 {
        return format!(
            "{} on a {} home: nothing up to £250,000, 5% on £250,000 to £925,000, 10% to £1.5m and 12% above that, so the bill is only what is above those bands. A first-time buyer pays nothing up to £300,000 instead, and a second home or a buy-to-let adds 5% to every band, which would be {} here. Check GOV.UK before you exchange, because the bands move with budgets.",
            _duty_said(_duty(price, &_SDLT)),
            priced,
            _duty_said(_duty(price, &_SDLT) + price * 0.05)
        );
    }
    if second {
        return "Add 5% to every band, from the first pound: on a £250,000 buy-to-let that is £12,500 instead of nothing, and on a £400,000 one £27,500 instead of £7,500. It is paid within 14 days of completion and reclaimed if you sell the old home within three years.".to_string();
    }
    "Nothing up to £250,000 for a home mover, or up to £300,000 for a first-time buyer: 5% from £250,000 to £925,000, 10% to £1.5m and 12% above that, and a second home or a buy-to-let adds 5 percentage points to every band. In Scotland it is LBTT and in Wales it is LTT, with their own bands, and the bill is paid within 14 days of completion, so check GOV.UK for your price before you exchange.".to_string()
}

/// LBTT, Scotland: nil to 145,000, and the first-time relief lifts that to 175,000.
pub fn _scottish_duty(price: f64, first: bool, second: bool) -> String {
    let priced = if price >= 20000.0 { format!("£{}", _comma(price)) } else { String::new() };
    if price >= 20000.0 && second {
        let extra = price * 0.08;
        let normal = _duty(price, &_LBTT);
        let made = if normal <= 0.0 {
            "nothing to pay in the LBTT bands, so the whole bill is the 8% Additional Dwelling Supplement".to_string()
        } else {
            format!(
                "{} of it is the LBTT bands and {} is the 8% Additional Dwelling Supplement",
                _duty_said(normal),
                _duty_said(extra)
            )
        };
        return format!(
            "{} on a {} second home in Scotland: {} on the whole price. The supplement is paid within 30 days of settlement and claimed back if you sell the old home within 36 months, so check Revenue Scotland's calculator before you bid.",
            _duty_said(normal + extra),
            priced,
            made
        );
    }
    if price >= 20000.0 && first {
        if price > *_SLBTT_FIRST_NIL {
            return format!(
                "{} on a {} first home in Scotland: the first-time relief stops above £175,000, so this pays the full LBTT bill - nothing to £145,000, 2% to £250,000, 5% to £325,000, 10% to £750,000 and 12% above that. The relief is worth £600 at most. An additional home adds 8% on the whole price.",
                _duty_said(_duty(price, &_LBTT)),
                priced
            );
        }
        return format!(
            "Nothing, on a {} first home in Scotland: first-time buyers pay no LBTT up to £175,000 instead of £145,000. The relief is worth £600 at most and stops once the price is over £175,000, so a £200,000 first home pays the ordinary 2% above £145,000, which is £1,100.",
            priced
        );
    }
    if price >= 20000.0 {
        return format!(
            "{} on a {} home in Scotland under LBTT: nothing up to £145,000, 2% to £250,000, 5% to £325,000, 10% to £750,000 and 12% above that. A first-time buyer pays nothing up to £175,000 instead, which is worth £600 at most, and an additional home adds 8% on the whole price, which would be {} here. Revenue Scotland has the calculator to check it against.",
            _duty_said(_duty(price, &_LBTT)),
            priced,
            _duty_said(_duty(price, &_LBTT) + price * 0.08)
        );
    }
    "In Scotland it is LBTT, not stamp duty: nothing up to £145,000, 2% to £250,000, 5% to £325,000, 10% to £750,000 and 12% above that. A first-time buyer's nil band is £175,000, worth £600 at most, an additional home adds 8% on the whole price, and the bill is due within 30 days of settlement, so put the price in and check Revenue Scotland's calculator.".to_string()
}

/// LTT, Wales: nil to 225,000, and no first-time buyer relief at all.
pub fn _welsh_duty(price: f64, _first: bool, second: bool) -> String {
    let priced = if price >= 20000.0 { format!("£{}", _comma(price)) } else { String::new() };
    if price >= 20000.0 && second {
        let extra = price * 0.05;
        let normal = _duty(price, &_LTT);
        let made = if normal <= 0.0 {
            "nothing to pay in the LTT bands, so the whole bill is the 5% higher rate".to_string()
        } else {
            format!("{} of it is the LTT bands and {} is the 5% higher rate", _duty_said(normal), _duty_said(extra))
        };
        return format!(
            "{} on a {} second home in Wales: {} on the whole price. It is due within 30 days of completion and reclaimed if you sell the old home within three years, so check the Welsh Revenue Authority calculator before you exchange.",
            _duty_said(normal + extra),
            priced,
            made
        );
    }
    if price >= 20000.0 {
        return format!(
            "{} on a {} home in Wales under LTT: nothing up to £225,000, 6% from there to £400,000, 7.5% to £750,000, 10% to £1.5m and 12% above that. There is no first-time buyer relief in Wales, and an additional home pays 5% on the whole price, which would be {} here. The Welsh Revenue Authority has the calculator to check it against.",
            _duty_said(_duty(price, &_LTT)),
            priced,
            _duty_said(_duty(price, &_LTT) + price * 0.05)
        );
    }
    if second {
        return "In Wales the surcharge is 5% on the whole price: a £250,000 second home pays £14,000 instead of £1,500. LTT itself is nothing up to £225,000, 6% to £400,000, 7.5% to £750,000, 10% to £1.5m and 12% above, and there is no first-time buyer relief in Wales.".to_string();
    }
    "In Wales it is LTT, not stamp duty, and it is nothing up to £225,000: 6% from there to £400,000, 7.5% to £750,000, 10% to £1.5m and 12% above that. There is no first-time buyer relief, an additional home pays 5% on the whole price, and the bill is due within 30 days of completion, so check the Welsh Revenue Authority calculator with your price.".to_string()
}

/// "On 32k a year, is 1,500 a month too much" wants the two limits, not what a mortgage is.
pub fn _mortgage_afford(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = clean_or_text(understood);
    let income = _sum_and_unit(text, &["a year", "per year", "a yr", "salary", "annum", "earn"]);
    let payment = _sum_and_unit(text, &["a month", "per month", "monthly", "pcm", "a month"]);
    if income == 0 && payment == 0 && text.contains("afford") {
        return Some("Lenders usually cap borrowing at about 4.5 times income, so two salaries of £25,000 buy roughly £225,000 and a single £30,000 about £135,000 - the deposit sits on top of that (5% at least, 10% for a decent rate). Two salaries add up, but childcare, car finance and card limits all come off the total, so run your own figures through a bank or broker calculator before you view anything.".to_string());
    }
    if income != 0 && payment == 0 {
        let wanted = _target(text);
        if wanted != 0 && wanted != income {
            let cap = int(income as f64 * 4.5);
            if wanted > cap {
                return Some(format!(
                    "On £{} a year lenders cap borrowing at about 4.5 times income, so £{} - a £{} mortgage is above that on one salary. A joint application, a bigger deposit or a much longer term is what closes the gap, and a fee-free broker will tell you what lenders would actually offer.",
                    comma(income),
                    comma(cap),
                    comma(wanted)
                ));
            }
            // Python formats this one with `%`, and its "(5% at least, 10% for ..." reads as two
            // more conversions than there are values: TypeError, so the entry declines.
            panic!("TypeError: not enough arguments for format string");
        }
        return Some("Lenders usually cap borrowing at about 4.5 times income, so two salaries of £25,000 buy roughly £225,000, and a single £30,000 about £135,000 - the deposit sits on top of that (5% at least, 10% for a decent rate). Two salaries add up, but childcare, car finance and card limits all come off the total, so run your own figures through a bank or broker calculator before you view anything.".to_string());
    }
    let p = payment as f64;
    if payment != 0 && income == 0 {
        let loan = int(p * (1.0 - (1.0 + 0.05 / 12.0f64).powf(-300.0)) / (0.05 / 12.0));
        return Some(format!(
            "£{} a month is roughly a £{} mortgage at 5% over 25 years. The usual test is keeping the payment under about a third of your take-home pay, so £{} a month wants a net income near £{}, and the deposit (5% at least, 10% for a decent rate) sits on top of the loan.",
            comma(payment),
            comma(loan),
            comma(payment),
            comma(payment * 3)
        ));
    }
    if income == 0 || payment == 0 {
        return None;
    }
    let cap = int(income as f64 * 4.5);
    let support = int(p * (1.0 - (1.0 + 0.05 / 12.0f64).powf(-360.0)) / (0.05 / 12.0));
    let limit = i64::min(cap, support);
    let monthly = int(limit as f64 * (0.045 / 12.0) / (1.0 - (1.0 + 0.045 / 12.0f64).powf(-360.0)));
    let first = if support < cap {
        format!(
            "£{} a month supports a loan of about £{} at 5% over 30 years, and the payment is the limit here rather than the salary: lenders cap borrowing near 4.5 times income, which on £{} a year is £{}.",
            comma(payment),
            comma(support),
            comma(income),
            comma(cap)
        )
    } else {
        format!(
            "On £{} a year lenders cap borrowing at about 4.5 times income, so £{} - your £{} a month would support about £{}, so the salary is the limit, not the payment.",
            comma(income),
            comma(cap),
            comma(payment),
            comma(support)
        )
    };
    Some(format!(
        "{} Borrowing £{} at 4.5% over 30 years costs about £{} a month, so keep the difference for fees, insurance and the months when something breaks.",
        first,
        comma(limit),
        comma(monthly)
    ))
}

// ---------------------------------------------------------------------------------------------
// Time to a goal, putting by, the share of pay

re_g!(ASKS_A_LENGTH, AIM_FOR, _DURATION, _MONEY_SUBJECT);
strs_g!(_MONTHS);

static SMALLEST_GOAL: LazyLock<i64> = LazyLock::new(|| int_g("SMALLEST_GOAL"));
static SMALL_GOAL_MONTHS: LazyLock<Vec<i64>> = LazyLock::new(|| {
    data::get("bot.knowledge.money.SMALL_GOAL_MONTHS").items().iter().map(|v| v.as_int().unwrap()).collect()
});
static RATES_PEOPLE_SAVE_AT: LazyLock<Vec<i64>> = LazyLock::new(|| {
    data::get("bot.knowledge.money.RATES_PEOPLE_SAVE_AT").items().iter().map(|v| v.as_int().unwrap()).collect()
});

/// A `{key: int}` dict global, by its str keys.
fn str_int_dict(name: &str) -> Vec<(String, i64)> {
    data::get(&format!("bot.knowledge.money.{name}"))
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), v.as_int().unwrap()))
        .collect()
}

/// A `{int: str}` dict global.
fn int_str_dict(name: &str) -> Vec<(i64, String)> {
    data::get(&format!("bot.knowledge.money.{name}"))
        .pairs()
        .iter()
        .map(|(k, v)| (k.as_int().unwrap(), v.str().to_string()))
        .collect()
}

fn lookup<'a, K: PartialEq + ?Sized, V>(table: &'a [(impl std::borrow::Borrow<K>, V)], key: &K) -> Option<&'a V> {
    table.iter().find(|(k, _)| k.borrow() == key).map(|(_, v)| v)
}

static _DURATION_WORDS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| str_int_dict("_DURATION_WORDS"));
static _IN_A_YEAR: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| str_int_dict("_IN_A_YEAR"));
static MONTHS_IN_WORDS: LazyLock<Vec<(i64, String)>> = LazyLock::new(|| int_str_dict("MONTHS_IN_WORDS"));

/// Python's `a // b` on ints.
fn floordiv(a: i64, b: i64) -> i64 {
    let q = a / b;
    if a % b != 0 && ((a < 0) != (b < 0)) { q - 1 } else { q }
}

/// The two ways to save a sum below the smallest rate on the list, in months.
pub fn _small_goal_rates(named: i64) -> Vec<(i64, i64)> {
    let mut found = Vec::new();
    for &months in SMALL_GOAL_MONTHS.iter() {
        let rate = named as f64 / months as f64;
        if rate != rate.trunc() || rate < *SMALLEST_GOAL as f64 {
            continue;
        }
        if months > 1 && int(rate) % 5 != 0 {
            continue;
        }
        found.push((int(rate), months));
    }
    found.truncate(2);
    found
}

/// The texts of the answers this chat has given from entries (`memory["entry_answers"]`).
fn entry_answer_texts(memory: &Memory) -> Vec<String> {
    let Some(records) = memory.extra.get("entry_answers") else { return Vec::new() };
    records
        .items()
        .iter()
        .map(|record| match record {
            Value::Dict(_) => match record.get("text") {
                Some(Value::Str(s)) => s.to_string(),
                Some(v) if v.truthy() => py_str(v),
                _ => String::new(),
            },
            // `record.get` on anything but a dict is an AttributeError in Python.
            _ => panic!("AttributeError: entry answer is not a dict"),
        })
        .collect()
}

/// Python `str(value)` for the plain values a memory record holds.
fn py_str(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        Value::Bool(b) => if *b { "True".into() } else { "False".into() },
        Value::None => "None".into(),
        other => format!("{other:?}"),
    }
}

/// The sums each month this chat has already shown, read from the answers it gave.
pub fn _rates_already_given(memory: &Memory) -> Vec<i64> {
    let mut given = Vec::new();
    for text in entry_answer_texts(memory) {
        for &rate in RATES_PEOPLE_SAVE_AT.iter() {
            if text.contains(&format!("£{} a month", comma(rate))) && !given.contains(&rate) {
                given.push(rate);
            }
        }
    }
    given
}

/// (what is put by, its unit, how many, unit) for "50 a week for a year", or None.
pub fn _rate_for_a_time(text: &str) -> Option<(f64, String, i64, String)> {
    static RATE: LazyLock<Re> =
        LazyLock::new(|| Re::new(&format!("{}{}", *_MONEY, r"\s*(?:a|per|each|every)\s*(month|week|fortnight|year)"), 0));
    let rate = RATE.search(text)?;
    let clause = _clause_about(text, rate.start(), rate.end());
    if sh::any_in(&clause, &GOING_OUT) && !sh::any_in(&clause, &GOING_IN) {
        return None;
    }
    let length = _DURATION.search(text)?;
    let count = match lookup(&_DURATION_WORDS, length.str(1).to_lowercase().as_str()) {
        Some(c) => *c,
        None => int(figure(length.str(1))),
    };
    let unit = length.str(2).to_lowercase().trim_end_matches('s').to_string();
    if count <= 0 || count > 60 {
        return None;
    }
    Some((_said_a_month(&rate), rate.str(3).to_string(), count, unit))
}

/// The total a named rate reaches in a named time, or None when the line names no both.
pub fn _rate_over_time(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text = clean_or_text(understood);
    let (amount, rate_unit, count, unit) = _rate_for_a_time(text)?;
    let per_year = |u: &str| *lookup(&_IN_A_YEAR, u).unwrap_or_else(|| panic!("KeyError: {u}")) as f64;
    let total = amount * per_year(&rate_unit) * count as f64 / per_year(&unit);
    if total < amount {
        return None;
    }
    let total = text::round_half_even(total);
    let said = match rate_unit.as_str() {
        "week" => "a week",
        "fortnight" => "a fortnight",
        "month" => "a month",
        "year" => "a year",
        other => panic!("KeyError: {other}"),
    };
    let length = length_in_words(count, &unit);
    let working = _working_out(amount, &rate_unit, count, &unit, total);
    Some(format!(
        "£{} {} for {} is £{}.{} Set the standing order for the day after payday and the money leaves before you can spend it.",
        comma(int(amount)),
        said,
        length,
        comma(total),
        if working.is_empty() { String::new() } else { format!(" {working}") }
    ))
}

/// "a year", "3 months".
pub fn length_in_words(count: i64, unit: &str) -> String {
    if count == 1 {
        return format!("a {unit}");
    }
    format!("{count} {unit}s")
}

/// The multiplication said plainly, in the unit the person used, when it is not the same.
pub fn _working_out(amount: f64, rate_unit: &str, count: i64, _unit: &str, total: i64) -> String {
    let in_rate_unit = if amount != 0.0 { total as f64 / amount } else { 0.0 };
    let rounded = text::round_half_even(in_rate_unit);
    if (in_rate_unit - rounded as f64).abs() < 0.01 && rounded != count {
        return format!("That is {} {}s at £{} each.", rounded, rate_unit, comma(int(amount)));
    }
    String::new()
}

/// How long a goal takes at what the line puts aside: the sum people actually want.
pub fn _time_to_save(memory: &mut Memory, understood: &Understood) -> Option<String> {
    if let Some(over_time) = _rate_over_time(memory, understood) {
        if !over_time.is_empty() {
            return Some(over_time);
        }
    }
    let text = understood.clean.as_str();
    let said = understood.text.as_str();
    let (amount, mut target) = (_per_month(text), _target(text));
    if amount <= 0 {
        let mut named = if target != 0 && _target(said) != 0 { target } else { 0 };
        if named == 0 && ASKS_A_LENGTH.is_match(&text.to_lowercase()) {
            named = _goal_from_user(memory, 0.0);
        }
        if named > 0 {
            let skip = if _target(said) != 0 { Vec::new() } else { _rates_already_given(memory) };
            let rates: Vec<i64> =
                RATES_PEOPLE_SAVE_AT.iter().copied().filter(|r| *r < named && !skip.contains(r)).take(3).collect();
            if rates.is_empty() && *SMALLEST_GOAL < named && skip.is_empty() {
                let small = _small_goal_rates(named);
                if !small.is_empty() {
                    let parts: Vec<String> = small
                        .iter()
                        .map(|&(rate, months)| {
                            let words = lookup(&MONTHS_IN_WORDS, &months).cloned().unwrap_or_else(|| format!("{months} months"));
                            format!("{} at £{} a month", words, comma(rate))
                        })
                        .collect();
                    return Some(format!(
                        "For £{}: {}. Pick the rate your pay can carry and set the standing order for the day after payday, so you never have to decide again.",
                        comma(named),
                        if parts.len() > 1 { parts.join(" or ") } else { parts[0].clone() }
                    ));
                }
            }
            if !rates.is_empty() {
                let parts: Vec<String> = rates
                    .iter()
                    .map(|&rate| format!("£{} a month is {}", comma(rate), _how_long(_months(rate, named))))
                    .collect();
                return Some(format!(
                    "For £{}: {}. Pick the rate your pay can carry and set the standing order for the day after payday, so you never have to decide again.",
                    comma(named),
                    parts.join(", ")
                ));
            }
        }
        return None;
    }
    if target == 0 {
        for number in &understood.numbers {
            if let Some(value) = &number.value {
                if value.as_f64() > amount as f64 {
                    target = int(value.as_f64());
                    break;
                }
            }
        }
    }
    if target <= 0 {
        target = _goal_from_user(memory, amount as f64);
    }
    if target <= 0 {
        target = _goal_already_named(memory);
    }
    if target <= 0 {
        return Some(format!(
            "{} is £{} in a year and £{} in five. Say what you are saving for and the total, and I will work out exactly how long it takes.",
            _a_month_in_words(text, amount),
            comma(amount * 12),
            comma(amount * 60)
        ));
    }
    let months = _months(amount, target);
    let after = _how_long(months);
    Some(format!(
        "£{} at {} is {}. A standing order the day after payday does it without you deciding again, and any interest or pay rise only shortens it.",
        comma(target),
        _a_month_in_words(text, amount),
        after
    ))
}

/// The biggest sum the person themselves named lately, if it is a goal rather than a rate.
pub fn _goal_from_user(memory: &Memory, amount: f64) -> i64 {
    let mut seen = 0;
    for said in user_lines_newest_first(memory) {
        if said.is_empty() {
            continue;
        }
        seen += 1;
        let mut best = 0i64;
        for (value, m) in _sums_in(said) {
            if _rate_words(chars_after(said, m.end(), 24)) {
                continue;
            }
            if value > amount && value > best as f64 {
                best = int(value);
            }
        }
        if best != 0 {
            return best;
        }
        if seen >= 4 {
            break;
        }
    }
    0
}

/// The month the person named lately as a deadline: "I want 500 saved by December".
pub fn _named_month(memory: &Memory) -> String {
    let mut seen = 0;
    for said in user_lines_newest_first(memory) {
        let said = said.to_lowercase();
        if said.is_empty() {
            continue;
        }
        seen += 1;
        let found = _month_in(&said);
        if !found.is_empty() {
            return found;
        }
        if seen >= 4 {
            break;
        }
    }
    String::new()
}

/// The deadline month this line names, or Christmas when it says Christmas.
pub fn _month_in(text: &str) -> String {
    static MONTH_RES: LazyLock<Vec<(String, Re)>> =
        LazyLock::new(|| _MONTHS.iter().map(|m| (m.clone(), Re::new(&format!(r"\b{m}\b"), 0))).collect());
    let low = format!(" {} ", text.to_lowercase());
    for (name, re) in MONTH_RES.iter() {
        if re.is_match(&low) {
            return name.clone();
        }
    }
    if low.contains("christmas") || low.contains("xmas") {
        return "december".to_string();
    }
    String::new()
}

/// A goal named in the line gets its own arithmetic, not an example with someone else's sum.
pub fn _save_goal(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = clean_or_text(understood);
    if _a_first_meeting(text, understood) {
        return None;
    }
    if _target(text) != 0
        || _per_month(text) != 0
        || _goal_from_user(memory, 0.0) != 0
        || _goal_already_named(memory) != 0
    {
        if let Some(worked) = _time_to_save(memory, understood) {
            if !worked.is_empty() {
                return Some(worked);
            }
        }
    }
    Some("Work backwards from the date you need it: divide the total by the weeks left to get the weekly figure, then set a standing order for it the day after payday into an account you don't carry a card for. Any bonus, refund or bill that goes down follows it in.".to_string())
}

/// How much to set aside each week for a target that was already named.
pub fn _put_by(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = clean_or_text(understood);
    if let Some(over_time) = _rate_over_time(memory, understood) {
        if !over_time.is_empty() {
            return Some(over_time);
        }
    }
    let mut goal = _target(text);
    if goal == 0 {
        goal = _goal_from_user(memory, 0.0);
    }
    if goal == 0 {
        goal = _goal_already_named(memory);
    }
    let mut month = _month_in(text);
    if month.is_empty() {
        month = _named_month(memory);
    }
    let mut weeks = 0;
    if !month.is_empty() {
        let today = dup::today();
        let month_no = _MONTHS.iter().position(|m| *m == month).unwrap() as u32 + 1;
        let year = today.year + if month_no <= today.month { 1 } else { 0 };
        let days = dup::Date { year, month: month_no, day: 1 }.days_since(&today);
        weeks = -floordiv(-days, 7);
    }
    if goal != 0 && weeks >= 2 {
        let a_week = -floordiv(-goal, weeks);
        return Some(format!(
            "£{} by {} is about £{} a week over the {} weeks left — a standing order for that on payday moves it without you deciding again, and any bonus, refund or bill that goes down only shortens it.",
            comma(goal),
            text::capitalize(&month),
            comma(a_week),
            comma(weeks)
        ));
    }
    if goal != 0 {
        if let Some(worked) = _time_to_save(memory, understood) {
            if !worked.is_empty() {
                return Some(worked);
            }
        }
    }
    if _target(text) != 0 || _goal_from_user(memory, 0.0) != 0 || _goal_already_named(memory) != 0 {
        return None;
    }
    if !_MONEY_SUBJECT.is_match(text) {
        return None;
    }
    Some(_share_of_pay(text))
}

/// "How much should i put by each week" names no total, so the answer is the share.
pub fn _share_of_pay(text: &str) -> String {
    if rx!(r"\b(?:a|per|each|every)\s*week\b|\bweekly\b").is_match(text) {
        return "A tenth to a fifth of what you take home is the usual amount to put by: on £2,000 a month that is £50 to £90 a week, and a standing order for the day after payday moves it before you can spend it. If that is too much this month, take the fixed amount you would not miss and raise it by £5 whenever a bill drops.".to_string();
    }
    "A tenth to a fifth of what you take home is the usual amount to put by: £200 to £400 a month on £2,000, in a separate account with a standing order for the day after payday so it leaves before you can spend it. If that is too much this month, take the fixed amount you would not miss and raise it by £5 whenever a bill drops.".to_string()
}

// ---------------------------------------------------------------------------------------------
// Habits, money to last, gifts, food costs

re_g!(
    _A_MONTH_NAMED, _A_SUM_NAMED, _A_THING_TO_COVER, _A_BIRTHDAY, _A_CHILD, _A_THIN_WEEK, _A_RECEIPT_QUESTION,
    _A_HEADCOUNT, _A_FOOD_WORD, _ASKS_HOW_LONG, _NO_MONEY, _THE_FIGURE, _A_STATED_BUDGET, _A_SPEND_ON_A_THING,
    _ASKS_FOR_SOMETHING_TO_DO, _AN_OCCASION_OR_A_TREAT, _A_TRIP, _A_MEAL, _SAYS_NOTHING_AT_ALL,
    _ASKS_WHAT_TO_COOK, _A_GIFT_OR_A_PARTY,
);
str_g!(_A_BIRTHDAY_WITH_LITTLE, _GIFT_BUDGET, _CHEAP_DINNERS, _CHEAP_WEEK, _CHEAP_FOOD);

/// `_NUMBER_WORDS` as the module ends up with it (the later {"2": "two", ...} table).
static _NUMBER_WORDS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.money._NUMBER_WORDS")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), v.str().to_string()))
        .collect()
});
static _COUNT_WORDS: LazyLock<Vec<(i64, String)>> = LazyLock::new(|| int_str_dict("_COUNT_WORDS"));
static _HEAD_WORDS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| str_int_dict("_HEAD_WORDS"));
static _WEEKDAYS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| str_int_dict("_WEEKDAYS"));
/// (phrase, figure) pairs.
static _WORD_FIGURES: LazyLock<Vec<(String, f64)>> = LazyLock::new(|| {
    data::get("bot.knowledge.money._WORD_FIGURES")
        .items()
        .iter()
        .map(|p| (p.items()[0].str().to_string(), p.items()[1].as_f64().unwrap()))
        .collect()
});
/// (phrase, days) pairs.
static _SPANS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| {
    data::get("bot.knowledge.money._SPANS")
        .items()
        .iter()
        .map(|p| (p.items()[0].str().to_string(), p.items()[1].as_int().unwrap()))
        .collect()
});
/// (phrases, days or None) pairs: a phrase is one string or a tuple of them.
static _DEADLINES: LazyLock<Vec<(Vec<String>, Option<i64>)>> = LazyLock::new(|| {
    data::get("bot.knowledge.money._DEADLINES")
        .items()
        .iter()
        .map(|p| {
            let phrase = &p.items()[0];
            let phrases = match phrase {
                Value::Str(s) => vec![s.to_string()],
                other => other.strs(),
            };
            (phrases, p.items()[1].as_int())
        })
        .collect()
});
/// (word, going rate) pairs.
static _GOING_RATE: LazyLock<Vec<(String, f64)>> = LazyLock::new(|| {
    data::get("bot.knowledge.money._GOING_RATE")
        .items()
        .iter()
        .map(|p| (p.items()[0].str().to_string(), p.items()[1].as_f64().unwrap()))
        .collect()
});

/// What `_number` gives back: an int, or (from the word table the module ends up with) a str.
#[derive(Clone, Debug, PartialEq)]
pub enum PyNumber {
    Int(i64),
    Str(String),
}

/// `_number(word)`: a digit string is its int, otherwise the word table's value, or 0.
pub fn _number(word: &str) -> PyNumber {
    let word = word.to_lowercase();
    if text::is_digit(&word) {
        return PyNumber::Int(word.parse().unwrap_or(0));
    }
    match lookup(&_NUMBER_WORDS, word.as_str()) {
        Some(v) => PyNumber::Str(v.clone()),
        None => PyNumber::Int(0),
    }
}

/// Python `round(x, 2)`.
fn round2(x: f64) -> f64 {
    format!("{:.2}", x).parse().unwrap()
}

/// £3.50 rather than £3.5, and £4 rather than £4.00.
pub fn _pounds(amount: f64) -> String {
    if round2(amount) != int(amount) as f64 { format!("£{:.2}", amount) } else { format!("£{}", int(amount)) }
}

/// True unless the line names a month and no figure, and says nothing it has to cover.
pub fn _a_month_and_no_sum(understood: &Understood) -> bool {
    let line = understood.clean.as_str();
    if !_A_MONTH_NAMED.is_match(line) || _A_SUM_NAMED.is_match(line) {
        return true;
    }
    _A_THING_TO_COVER.is_match(line)
}

/// How much a present should cost, or how to mark a child's birthday in a thin week.
pub fn _a_gift_budget(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = said_lower(understood);
    if _A_BIRTHDAY.is_match(&text) && _A_CHILD.is_match(&text) && _A_THIN_WEEK.is_match(&text) {
        return Some(_A_BIRTHDAY_WITH_LITTLE.clone());
    }
    Some(_GIFT_BUDGET.clone())
}

/// True when the line asks about a receipt (the shop's paper, not the money).
pub fn _a_receipt_question(understood: &Understood) -> bool {
    let said = format!("{} {}", understood.text, understood.clean);
    _A_RECEIPT_QUESTION.is_match(&said.to_lowercase())
}

/// How many people the line says the money has to feed, or 0.
pub fn _a_headcount_in(text: &str) -> i64 {
    let Some(found) = _A_HEADCOUNT.search(text) else { return 0 };
    let said = found.groups().into_iter().flatten().find(|g| !g.is_empty()).unwrap_or("");
    if text::is_digit(said) {
        return said.parse().unwrap_or(0);
    }
    lookup(&_HEAD_WORDS, said).copied().unwrap_or(0)
}

/// A sum that has to be spent on food: the days it has to last, and how.
pub fn _money_to_last(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = said_lower(understood);
    let t = text.as_str();
    if _ASKS_THE_PRICE.is_match(t)
        && _A_FOOD_WORD.is_match(t)
        && !_ASKS_HOW_LONG.is_match(t)
        && !_NO_MONEY.is_match(t)
        && !_THE_FIGURE.is_match(t)
        && !_WORD_FIGURES.iter().any(|(phrase, _)| t.contains(phrase.as_str()))
    {
        return None;
    }
    if _A_STATED_BUDGET.is_match(t) && !_ASKS_HOW_LONG.is_match(t) {
        return None;
    }
    if _A_SPEND_ON_A_THING.is_match(t) && !_A_FOOD_WORD.is_match(t) {
        return None;
    }
    if _ASKS_FOR_SOMETHING_TO_DO.is_match(t) && !_A_FOOD_WORD.is_match(t) {
        return None;
    }
    if _A_BIRTHDAY.is_match(t) && _A_THIN_WEEK.is_match(t) && !_A_FOOD_WORD.is_match(t) {
        return Some(_A_BIRTHDAY_WITH_LITTLE.clone());
    }
    if _AN_OCCASION_OR_A_TREAT.is_match(t) && !_A_FOOD_WORD.is_match(t) {
        return None;
    }
    let mut amount = 0.0;
    if let Some(m) = _THE_FIGURE.finditer(t).into_iter().next() {
        let raw = if m.get(1).is_some_and(|g| !g.is_empty()) { m.str(1) } else { m.str(2) };
        amount = raw.replace(',', "").parse::<f64>().unwrap_or_else(|_| panic!("ValueError: float({raw:?})"));
    }
    if amount == 0.0 {
        for (phrase, figure) in _WORD_FIGURES.iter() {
            if t.contains(phrase.as_str()) {
                amount = *figure;
                break;
            }
        }
    }
    if amount == 0.0 && _A_TRIP.is_match(t) && !_A_FOOD_WORD.is_match(t) {
        return None;
    }
    let mut days = 0i64;
    let mut named = String::new();
    let mut assumed = "";
    for (phrase, span) in _SPANS.iter() {
        if t.contains(phrase.as_str()) {
            days = *span;
            named = phrase.clone();
            break;
        }
    }
    if days == 0 {
        for (phrases, span) in _DEADLINES.iter() {
            let found = phrases.iter().find(|w| t.contains(w.as_str())).cloned().unwrap_or_default();
            if !found.is_empty() {
                if let Some(span) = span.filter(|s| *s != 0) {
                    days = span;
                    named = found;
                } else if let Some(day) = lookup(&_WEEKDAYS, found.as_str()) {
                    let today = dup::today();
                    let d = (day - today.weekday()).rem_euclid(7);
                    days = if d == 0 { 7 } else { d };
                    named = found;
                } else {
                    days = 14;
                    assumed = "payday";
                }
                break;
            }
        }
    }
    let heads = _a_headcount_in(t);
    if amount == 0.0 && _NO_MONEY.is_match(t) {
        return Some(format!("Use a food bank this week rather than going without: the Trussell Trust site finds the nearest one and a voucher comes from a GP, a school or Citizens Advice, and it costs you nothing. Have a look in the cupboard before you go - rice, pasta, oats and tinned beans still make a dinner: {}.", *_CHEAP_DINNERS));
    }
    let have = amount != 0.0;
    if have && days == 1 {
        return Some(format!(
            "{} for a day is not much of a shop: one big dinner cooked in double does two meals, and the cheapest of them are {}. Buy from that list and nothing else and the day comes in under the figure.",
            _pounds(amount),
            *_CHEAP_DINNERS
        ));
    }
    if have && days != 0 && assumed == "payday" {
        return Some(format!(
            "If payday is a fortnight off, {} is {} a day. {}",
            _pounds(amount),
            _pounds(round2(amount / days as f64)),
            *_CHEAP_WEEK
        ));
    }
    if have && days != 0 && !named.is_empty() {
        return Some(format!(
            "{} over {} days is {} a day. {}",
            _pounds(amount),
            days,
            _pounds(round2(amount / days as f64)),
            *_CHEAP_WEEK
        ));
    }
    if have && days != 0 {
        return Some(format!(
            "{} for {} days is {} a day. {}",
            _pounds(amount),
            days,
            _pounds(round2(amount / days as f64)),
            *_CHEAP_WEEK
        ));
    }
    if have && heads != 0 {
        return Some(format!(
            "{} for {} is {} a head, which is one of these each: {}. If it has to stretch to a week as well, that is {} a day, and the staples are what make that work: rice, pasta, lentils, eggs, oats, potatoes, frozen veg.",
            _pounds(amount),
            heads,
            _pounds(round2(amount / heads as f64)),
            *_CHEAP_DINNERS,
            _pounds(round2(amount / 7.0))
        ));
    }
    if have && _A_MEAL.is_match(t) {
        return Some(format!(
            "{} is one dinner for two with change, or two for one: {}. It will not cover a week - tell me how long it has to last and I will work the days out.",
            _pounds(amount),
            *_CHEAP_DINNERS
        ));
    }
    if have {
        return Some(format!(
            "If that has to last a week, {} is {} a day. {}",
            _pounds(amount),
            _pounds(round2(amount / 7.0)),
            *_CHEAP_WEEK
        ));
    }
    if days != 0 {
        return Some(format!("Short until then is the cheap week, and it needs no more than the staples: buy the staples first - rice, pasta, tinned tomatoes, lentils, eggs, oats, potatoes, frozen veg - and build the days from them: {}.", *_CHEAP_DINNERS));
    }
    let plan = format!("A cheap week is the staples first - rice, pasta, tinned tomatoes, lentils, eggs, oats, potatoes, frozen veg - and the dinners built on them: {}. Cook double and the week costs less per meal than the ready meals do.", *_CHEAP_DINNERS);
    if !_SAYS_NOTHING_AT_ALL.is_match(t) {
        return Some(plan);
    }
    if _ASKS_WHAT_TO_COOK.is_match(t) {
        return Some(format!(
            "Cooking with nothing in is the staples and the dinners built on them. {} If there is nothing coming in at all, a food bank is there for this week and a voucher comes from a GP, a school or Citizens Advice.",
            *_CHEAP_WEEK
        ));
    }
    Some(format!("Use a food bank this week rather than going without: the Trussell Trust site finds the nearest one, and a voucher comes from a GP, a school or Citizens Advice. About a pound a head still covers a dinner: {}", *_CHEAP_DINNERS))
}

/// A price as Python holds it in `_habit_cost`: an int from the line, or a float going rate.
#[derive(Clone, Copy)]
enum Price {
    Int(i64),
    Float(f64),
}

impl Price {
    fn f(self) -> f64 {
        match self {
            Price::Int(i) => i as f64,
            Price::Float(f) => f,
        }
    }
    fn times(self, n: i64) -> Price {
        match self {
            Price::Int(i) => Price::Int(i * n),
            Price::Float(f) => Price::Float(f * n as f64),
        }
    }
    /// `_comma(x)`: the int part with thousands separators.
    fn comma(self) -> String {
        match self {
            Price::Int(i) => comma(i),
            Price::Float(f) => _comma(f),
        }
    }
}

/// A small repeated purchase turned into a year's money.
pub fn _habit_cost(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    static TIMES_A_WEEK: LazyLock<Re> = LazyLock::new(|| {
        let words: Vec<&str> = _NUMBER_WORDS.iter().map(|(k, _)| k.as_str()).collect();
        Re::new(
            &format!(
                r"\b(\d+|{})\s+(?:days|times|mornings|lunches|coffees|takeaways|pints)\s+(?:a|per|each)\s+week",
                words.join("|")
            ),
            0,
        )
    });
    let text = said_lower(understood);
    let t = text.as_str();
    let smoking = ["smok", "cigarette", "fag", "vape", "vaping"].iter().any(|w| t.contains(w));
    let mut price = _sum_and_unit(t, &["a day", "per day", "each day", "daily"]);
    let mut a_day = price != 0;
    let mut monthly = 0;
    if price == 0 {
        monthly = _sum_and_unit(t, &["a month", "per month", "monthly"]);
    }
    if price == 0 {
        price = _sum_and_unit(t, &["a week", "per week", "each week", "weekly"]);
    }
    let mut times = PyNumber::Int(0);
    if let Some(m) = TIMES_A_WEEK.search(t) {
        times = _number(m.str(1));
    } else if rx!(r"\bonce\s+(?:a|per|each)\s+week\b").is_match(t) {
        times = PyNumber::Int(1);
    } else if rx!(r"\btwice\s+(?:a|per|each)\s+week\b").is_match(t) {
        times = PyNumber::Int(2);
    }
    if price == 0 && smoking {
        return Some("A pack of 20 is about £15 now, so ten a day is roughly £2,700 a year and a pack a day about £5,500. Stopping is the single biggest bill most smokers can cut, and a free stop-smoking service through your GP doubles your odds.".to_string());
    }
    if price == 0 && rx!(r"\b(?:every|each|a|per)\s+day\b|\bdaily\b").is_match(t) {
        a_day = true;
    }
    let price = if price != 0 {
        Price::Int(price)
    } else {
        let going = _GOING_RATE.iter().find(|(word, _)| t.contains(word.as_str())).map(|(_, g)| *g);
        match going {
            Some(g) if g != 0.0 => Price::Float(g),
            _ => return None,
        }
    };
    if monthly != 0 {
        let year = monthly * 12;
        return Some(format!(
            "£{} a month is £{} a year. Halving it is £{} back over the year.",
            comma(monthly),
            comma(year),
            _comma(year as f64 / 2.0)
        ));
    }
    if a_day {
        let (year, week) = (price.times(365), price.times(7));
        let line = format!("{} a day is about £{} a year", _pounds(price.f()), year.comma());
        return Some(format!(
            "{}, about £{} a week. Cutting two of them a week is £{} back over the year.",
            line,
            week.comma(),
            price.times(2).times(52).comma()
        ));
    }
    let times = match times {
        PyNumber::Int(0) => 0,
        PyNumber::Int(n) => n,
        PyNumber::Str(s) if s.is_empty() => 0,
        // `price * times` with a str: a TypeError, or a repeated str that `int()` cannot read.
        PyNumber::Str(_) => panic!("TypeError: price * str"),
    };
    if times != 0 {
        let year = price.times(times).times(52);
        let when = match times {
            1 => "once a week".to_string(),
            2 => "twice a week".to_string(),
            n => format!("{} times a week", lookup(&_COUNT_WORDS, &n).cloned().unwrap_or_else(|| n.to_string())),
        };
        let line = format!("{} {} is about £{} a year", _pounds(price.f()), when, year.comma());
        if times == 1 {
            return Some(format!("{}, so skipping it every other week is £{} back over the year.", line, price.times(26).comma()));
        }
        if times == 2 {
            return Some(format!(
                "{}. Swapping one a week for something from home is £{} back over the year.",
                line,
                price.times(52).comma()
            ));
        }
        return Some(format!("{}. Cutting two of them a week is £{} back over the year.", line, price.times(2).times(52).comma()));
    }
    let year = price.times(52);
    Some(format!(
        "{} a week is about £{} a year, £{} a month. Skip it every other week and it is £{} back over the year.",
        _pounds(price.f()),
        year.comma(),
        _comma(year.f() / 12.0),
        price.times(26).comma()
    ))
}

/// One dish's cost row: (name, whole?, figure, makes, note).
pub type DishCost = (String, bool, f64, String, String);

/// The costed dishes, each name with its word-bounded pattern.
static DISH_COSTS: LazyLock<(Vec<(Re, String, f64, String)>, Vec<(Re, String, f64, String, String)>)> = LazyLock::new(|| {
    let bounded = |name: &str| Re::new(&format!(r"\b{}\b", pyre::escape(name)), 0);
    let portion = data::get("bot.knowledge.money._DISH_COST_PORTION")
        .items()
        .iter()
        .flat_map(|row| {
            let x = row.items();
            let (figure, note) = (x[1].as_f64().unwrap(), x[2].str().to_string());
            x[0].strs().into_iter().map(move |name| (bounded(&name), name, figure, note.clone()))
        })
        .collect();
    let whole = data::get("bot.knowledge.money._DISH_COST_WHOLE")
        .items()
        .iter()
        .flat_map(|row| {
            let x = row.items();
            let (figure, makes, note) = (x[1].as_f64().unwrap(), x[2].str().to_string(), x[3].str().to_string());
            x[0].strs().into_iter().map(move |name| (bounded(&name), name, figure, makes.clone(), note.clone()))
        })
        .collect();
    (portion, whole)
});

/// The costed dish the line names, the longest name winning: (name, whole, figure, makes, note).
pub fn _the_dish_named(text: &str) -> Option<DishCost> {
    let text = text.to_lowercase();
    let mut best: Option<DishCost> = None;
    let longer = |best: &Option<DishCost>, name: &str| best.as_ref().is_none_or(|b| name.chars().count() > b.0.chars().count());
    for (re, name, figure, note) in DISH_COSTS.0.iter() {
        if re.is_match(&text) && longer(&best, name) {
            best = Some((name.clone(), false, *figure, String::new(), note.clone()));
        }
    }
    for (re, name, figure, makes, note) in DISH_COSTS.1.iter() {
        if re.is_match(&text) && longer(&best, name) {
            best = Some((name.clone(), true, *figure, makes.clone(), note.clone()));
        }
    }
    best
}

/// 80p under a pound, else `_pounds`.
pub fn _money(amount: f64) -> String {
    if round2(amount) < 1.0 {
        return format!("{}p", text::round_half_even(amount * 100.0));
    }
    _pounds(amount)
}

/// What a dish costs to cook at home, for the table the line names.
pub fn _dish_to_cook_cost(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = clean_or_text(understood);
    if dup::a_price_is_judged(text) {
        return None;
    }
    let (_name, whole, figure, makes, note) = _the_dish_named(text)?;
    let caveat = "Shop prices move by the week, so that is the shape of it rather than a quote.";
    if whole {
        return Some(format!("Ingredients for {} come to about {}, and {}. {}", makes, _money(figure), note, caveat));
    }
    let mut people = dup::_crowd(text);
    if people == 0 {
        people = 4;
    }
    if people < 2 {
        people = 4;
    }
    // round(figure, 1), then round(per_head * people * 2, 0) / 2.0
    let per_head: f64 = format!("{:.1}", figure).parse().unwrap();
    let total = text::round_half_even(per_head * people as f64 * 2.0) as f64 / 2.0;
    if people == 4 {
        return Some(format!("About {} for four, so {} a head: {}. {}", _money(total), _money(per_head), note, caveat));
    }
    let count = lookup(&_COUNT_WORDS, &people).cloned().unwrap_or_else(|| people.to_string());
    Some(format!("About {} a head, so {} for {}: {}. {}", _money(per_head), _money(total), count, note, caveat))
}

/// The goal an earlier answer of this chat worked out, read back from what it said.
pub fn _goal_already_named(memory: &Memory) -> i64 {
    let mut best = 0;
    for text in entry_answer_texts(memory) {
        let sums: Vec<String> =
            if text.contains("takes:") { rx!(r"£([\d,]+)").findall_str(&text) } else { AIM_FOR.findall_str(&text) };
        for found in sums {
            let Ok(value) = found.replace(',', "").parse::<i64>() else { continue };
            best = i64::max(best, value);
        }
    }
    best
}

/// A rent deposit: how much, and where it has to be kept.
pub fn _rent_deposit(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let back = "Ask for it back in writing with photos and the inventory attached; if you disagree, use the scheme's free dispute service instead of arguing with the agent.";
    let amount = "Five weeks' rent is the legal cap in England, so on £1,200 a month expect about £1,380. ";
    if understood.asks.iter().any(|a| a == "how_much") || understood.clean.contains("how many") {
        return Some(format!("{amount}It has to sit in a government-backed protection scheme within 30 days, and the landlord must tell you which one. {back}"));
    }
    Some(format!("Your deposit must be in a government-backed protection scheme within 30 days, and the landlord has to tell you which one. {back}"))
}

/// False when the line is about a gift or a party rather than a household's money.
pub fn _not_a_gift(understood: &Understood) -> bool {
    !_A_GIFT_OR_A_PARTY.is_match(&said_lower(understood))
}

/// Retirement: the state pension record, the ages, or stopping early.
pub fn _retirement(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let mut text = understood.clean.to_lowercase();
    if text.is_empty() {
        text = understood.words.join(" ");
    }
    let t = text.as_str();
    let has = |w: &str| t.contains(w);
    let target = "A rough target is 25 times what you want to spend each year on top of the state pension: £10,000 a year beyond it wants a pot of about £250,000. ";
    if has("state pension")
        && (has("how much")
            || has("forecast")
            || has("check")
            || has("record")
            || has("how many years")
            || has("years do i need")
            || has("qualifying")
            || has("voluntary")
            || has("buy extra")
            || has("fill a gap")
            || has("gap in my record"))
    {
        return Some(format!(
            "The full new state pension is about £11,500 a year, £230 a week, if you have 35 years of contributions; each year short pays about a thirtieth less, and you need at least 10 years to get anything. Check your own record and forecast at gov.uk/check-state-pension. {}",
            target.replace("on top of the state pension", "beyond the state pension")
        ));
    }
    if (has("when") || has("what age") || has("how old") || has("pension age") || has("take my pension") || has("draw") || has("claim"))
        && !has("how much")
        && !has("enough")
    {
        return Some(format!("Private and workplace pension money opens from 55, and 57 from April 2028; the state pension starts at 66 now and 67 by 2028. There is no fixed retirement age in law - your employer cannot push you out at 65 unless the job itself has one written in - so the dates that matter are the ones on your own forecast at gov.uk/check-state-pension. {target}"));
    }
    let early = has("early") || has("before i") || has("as soon as");
    let ages: Vec<i64> = rx!(r"\b(\d\d)\b").findall_str(t).iter().filter_map(|a| a.parse().ok()).collect();
    if early || ages.iter().any(|&age| age < 66) {
        let lead = if understood.asks.iter().any(|a| a == "yesno") { "Yes, if you can bridge the gap. " } else { "" };
        return Some(format!("{lead}Stopping before the state pension is mostly a bridging problem: the years between the date you want and the date each pot opens have to be paid for out of something else. Work out that gap first - private money from 55 (57 from April 2028), the state pension from 66 or 67 - then cover it with savings, an ISA or part-time work rather than the pension. {target}The earlier the date, the more years the pot has to carry, so try the sums at two or three ages and see which one the money actually allows."));
    }
    Some(format!("{target}The full new state pension is around £11,500 a year from 66 or 67, and you can take private pension money from 55 (57 from 2028), so check your forecast on the government's pension service before you decide anything."))
}

/// Drawdown, an annuity, or the tax-free quarter of a pot.
pub fn _pension_choice(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let mut text = understood.clean.to_lowercase();
    if text.is_empty() {
        text = understood.words.join(" ");
    }
    if text.contains("drawdown") {
        return Some("Drawdown keeps the pot invested and pays you an income you choose, so it can run out if the money does badly or you take too much. Take no more than about 4% a year to begin with, keep three to five years of spending in cash so you are not selling investments in a bad year, and remember the pot stays yours to pass on.".to_string());
    }
    if text.contains("annuit") {
        return Some("An annuity swaps your pot for a guaranteed income for life: you give the insurer the money and it pays you a set amount whatever happens to markets or how long you live. Shop around with an adviser or a broker, because the rate varies a lot between insurers, and tell them about any health problems - they can raise the income. It is the safe option rather than the generous one; most people now mix an annuity for the bills with drawdown for the rest.".to_string());
    }
    if !["annuit", "drawdown", "lump", "25%", "tax free", "tax-free", "cash in", "release", "pension pot", "pot"]
        .iter()
        .any(|w| text.contains(w))
    {
        return None;
    }
    Some("You can usually take 25% of a defined contribution pot tax-free, up to about £268,000 in total, and the rest is taxed as income when you take it. So taking a big lump sum in one year can push you into a higher tax band: spread it over several years, and leave the rest invested rather than cashing in the whole pot. Defined benefit or final salary pensions are different - leave those alone unless an adviser tells you otherwise, because transferring out is rarely a good deal and needs regulated advice over £30,000.".to_string())
}

/// How to split a pay packet.
pub fn _budget_split(_memory: &mut Memory, _understood: &Understood) -> Option<String> {
    Some("Split what actually lands in your account: fixed bills and food first, then move a set amount to savings the day you're paid, and treat what's left as the spending money. The 50/30/20 rule (half needs, a third wants, a fifth saved) is a starting point, not a law; paying yourself first is the part that works.".to_string())
}

// ---------------------------------------------------------------------------------------------
// Debts, missed payments, money trouble

re_g!(_A_DEBT, _DEBT_DUE, _A_SAVING_RATE, _A_RATE_FROM_INCOME, _CHARGED_A_THING, _PAYMENT_SAID, _SERVICING_A_DEBT);
strs_g!(_GAP_SAYINGS, _ASKING_FOR_AN_OVERDRAFT, _PAYMENT_ALREADY_MISSED, SPENT_UP);
str_g!(MISSED_MORTGAGE);

/// A debt in the line and a date it is due.
#[allow(non_snake_case)]
pub fn _A_DEBT_IS_DUE(said: &str) -> bool {
    _A_DEBT.is_match(said) && _DEBT_DUE.is_match(said)
}

/// False when the line says it cannot save and the only rate in it is a share of income.
pub fn _the_rate_row_has_a_rate(understood: &Understood) -> bool {
    let said = format!(" {} ", text::squash(&said_lower(understood)));
    if !sh::any_in(&said, &_GAP_SAYINGS) {
        return true;
    }
    for m in _A_SAVING_RATE.finditer(&said) {
        let start = m.start_of(1).unwrap_or_else(|| panic!("TypeError: group 1 did not take part"));
        let before = chars_before(&said, start, 40);
        if !_A_RATE_FROM_INCOME.is_match(before) {
            return true;
        }
    }
    false
}

/// Nothing saved yet: the first £500 and a standing order.
pub fn _savings_gap(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = words_lower(understood);
    let mut age = String::new();
    for word in text::split_ref(&said) {
        if text::is_digit(word) {
            if let Ok(n) = word.parse::<i64>() {
                if (16..=80).contains(&n) {
                    age = format!(" at {word}");
                    break;
                }
            }
        }
    }
    Some(format!("Nothing put by{age} is fixable in an afternoon, and the first £500 does more for you than any rate: open an easy-access savings account if you have not got one (ten minutes online), then set a standing order for payday of £10 or £25 - an amount you will not miss - and leave it alone. Build to £1,000, which covers most surprises, then to a month of rent, bills and food. What is left each month after the bills go out?"))
}

/// True unless the line names a thing charged and no count of payments.
pub fn _a_payment_count(understood: &Understood) -> bool {
    let said = words_lower(understood);
    if !_CHARGED_A_THING.is_match(&said) {
        return true;
    }
    _PAYMENT_SAID.is_match(&said)
}

/// A purse in trouble, unless the line is asking for an overdraft.
pub fn _a_purse_in_trouble(understood: &Understood) -> bool {
    let said = words_lower(understood);
    if sh::any_in(&said, &_ASKING_FOR_AN_OVERDRAFT) {
        return false;
    }
    _a_debt_in_trouble(understood)
}

/// True unless the line names a debt that is simply being paid as agreed.
pub fn _a_debt_in_trouble(understood: &Understood) -> bool {
    let said = words_lower(understood);
    if !_A_DEBT.is_match(&said) || _A_DEBT_IS_DUE(&said) {
        return true;
    }
    !_SERVICING_A_DEBT.is_match(&said)
}

/// A missed payment: insurance, a mortgage, a refund of the fee, or the plain card.
pub fn _missed_payment(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = words_lower(understood);
    let has = |w: &str| said.contains(w);
    let s = |x: &str| Some(x.to_string());
    if has("insurance") || has("policy") {
        return s("Ring the insurer today and pay it: cover is cancelled after a missed direct debit once they have chased it, and driving uninsured is an offence with a police fixed penalty on top of the premium going up. Ask to move the payment date to just after payday, and check the letters - a cancelled policy on purpose has to be declared at the next renewal. Is it car insurance or the house?");
    }
    if has("mortgage")
        && !["application", "apply", "applying", "get a mortgage", "getting a mortgage", "lender", "afford"].iter().any(|w| has(w))
    {
        return Some(MISSED_MORTGAGE.clone());
    }
    if has("mortgage") {
        return s("One late payment is usually not fatal for a mortgage: lenders read the last 12 months hardest and a single mark with everything else on time is explainable, especially once it is over a year old. What hurts is a run of missed payments, a default or a debt arrangement. Get the free reports from Experian, Equifax and TransUnion first and check nothing is wrong on them, then ask a broker which lenders take a single missed payment - several do. How recent was it, and was it a card or a loan?");
    }
    if has("refund") || has("take it back") || has("get it back") {
        return s("Ring and ask for it back, once: say it is the first one, that the payment has since gone through, and ask them to refund the charge as a goodwill gesture - most card issuers do it once a year without much argument. Do it in the app's chat or by email so there is a record, and check the next statement to see it land. If they say no, ask what their complaint process is; a written complaint gets a different answer from a call. Was it a card?");
    }
    if sh::any_in(&said, &_PAYMENT_ALREADY_MISSED) {
        return s("Three things follow a missed payment: a late fee, interest from the date you should have paid, and a mark on your credit file that stays six years - the mark is the expensive part. Pay what you can today, then ring them and ask them to note the reason, because a first miss with a reason is treated far more kindly than a run of them, and the fee is often refunded once you are back on track. Set a direct debit for at least the minimum so the next one cannot happen. Is it a card or a loan, and how far behind is it?");
    }
    s("You get a late fee (£12 on most cards, £25 or so on a loan or mortgage), interest from the due date, and a late payment recorded on your credit file, which then shows for six years - the mark matters more than the fee. Ring them before the date rather than after and ask to move the date or split the payment: an arrangement agreed before the due date is usually not reported as missed. Cancelling the direct debit is the worst version, because you lose the payment and pay the bank's charge too. When is it due, and how much are you short?")
}

/// Money trouble: a payment due, a spent-up week, or the general squeeze.
pub fn _money_trouble(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = words_lower(understood);
    if _A_DEBT_IS_DUE(&said) {
        return Some("Ring the card company before the payment date, not after: say you cannot make this month and ask for a payment plan or an interest freeze - they would rather have something than a missed payment, and it is the missed payment that goes on your file. Pay what you can, even a token amount, and ask them to confirm the arrangement in writing. Free debt advice from StepChange or Citizens Advice can do the call with you. Which card is it, and when is the payment due?".to_string());
    }
    if sh::any_in(&said, &SPENT_UP) {
        return Some("Spent up before the money comes in is a cash-flow week, not a crisis: write down what has to be paid before the next payment lands, then ring the one bill that will move a date, and plan the meals from what is already in the cupboard. What is left, and how long till you are paid?".to_string());
    }
    Some("Two calls free up the most cash fastest: a cheaper energy and mobile tariff, and the subscriptions and standing orders you have forgotten - usually £20 to £50 a month between them. If a bill is what has done it, the council will spread council tax over twelve months instead of ten. Say which bit is tightest and I'll go at that one.".to_string())
}

// ---------------------------------------------------------------------------------------------
// Making a sum last until a day

re_g!(_DAY_NAMED, _A_STATED_SIZE, _TWO_OF_US, _A_HOUSEHOLD, _A_WEEK_DAY_NAMED);
static _A_COUNT: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| str_int_dict("_A_COUNT"));

/// How long the money has to last: ("until Friday", 3), or ("", 0) when the line names no day.
pub fn _till_when(text: &str) -> (String, i64) {
    let Some(m) = _DAY_NAMED.search(text) else { return (String::new(), 0) };
    let said = m.str(1).to_lowercase();
    let today = dup::today();
    let wd = today.weekday();
    if ["today", "tonight", "tomorrow"].contains(&said.as_str()) {
        return (format!("until {said}"), 1);
    }
    if let Some(day) = lookup(&_WEEKDAYS, said.as_str()) {
        let days = (day - wd).rem_euclid(7);
        return (format!("until {}", text::capitalize(&said)), if days == 0 { 7 } else { days });
    }
    if ["the weekend", "the week", "this week", "the end of the week", "end of the week", "next week"].contains(&said.as_str()) {
        let d = (6 - wd).rem_euclid(7);
        return ("until the end of the week".to_string(), if d == 0 { 7 } else { d });
    }
    if ["a week", "a fortnight", "the fortnight", "two weeks"].contains(&said.as_str()) {
        return if said == "a week" { ("for a week".to_string(), 7) } else { ("for a fortnight".to_string(), 14) };
    }
    if said == "next friday" {
        let d = (4 - wd).rem_euclid(7);
        return ("until Friday".to_string(), if d == 0 { 7 } else { d });
    }
    let next_first = dup::Date {
        year: today.year + if today.month == 12 { 1 } else { 0 },
        month: today.month % 12 + 1,
        day: 1,
    };
    let days = i64::max(next_first.days_since(&today) - 1, 1);
    if said.contains("month") {
        return ("until the end of the month".to_string(), days);
    }
    ("until payday".to_string(), days)
}

/// 40p, £3, £2.50.
pub fn _a_pound(amount: f64) -> String {
    if amount < 1.0 {
        return format!("{}p", i64::max(text::round_half_even(amount * 100.0), 1));
    }
    let rounded = text::round_half_even(amount);
    if (amount - rounded as f64).abs() < 0.05 {
        return format!("£{rounded}");
    }
    format!("£{:.2}", amount)
}

/// How many people the money feeds: a stated size, two of us, a household (0), or one.
pub fn _the_mouths(text: &str) -> i64 {
    if let Some(stated) = _A_STATED_SIZE.search(text) {
        let size = stated.str(1).to_lowercase();
        let counted = lookup(&_A_COUNT, size.as_str()).copied().unwrap_or(0);
        if counted != 0 {
            return counted;
        }
        return if text::is_digit(&size) { size.parse().unwrap_or(3) } else { 3 };
    }
    if _TWO_OF_US.is_match(text) {
        return 2;
    }
    if _A_HOUSEHOLD.is_match(text) {
        return 0;
    }
    1
}

/// A count in words where the table has it ("2" -> "two"), else the digits.
pub fn _the_number(count: i64) -> String {
    let key = count.to_string();
    lookup(&_NUMBER_WORDS, key.as_str()).cloned().unwrap_or(key)
}

/// A sum to make last until a day: so much a day, and what that buys.
pub fn _make_it_last(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = clean_or_text(understood);
    let mut amount = 0.0f64;
    for (value, _m) in _sums_in(text) {
        amount = f64::max(amount, value);
    }
    let (label, days) = _till_when(text);
    if amount != 0.0 && days != 0 && _A_WEEK_DAY_NAMED.is_match(text) {
        return None;
    }
    if amount == 0.0 || days == 0 {
        return None;
    }
    let a_day = amount / days as f64;
    let food_only = rx!(r"\b(?:food|eat|eating|feed|meals?|groceries|shopping)\b", pyre::I).is_match(text);
    let span = if days >= 5 { "week" } else { "stretch" };
    if food_only {
        let mouths = _the_mouths(text);
        if mouths == 1 {
            return Some(format!(
                "{} {} is {} a day on food, which feeds one person properly if you cook. {} Do one shop for the whole {} rather than topping up, buy the own-brand version of everything on the list, and plan the meals round what is reduced at the end of the day.",
                _a_pound(amount), label, _a_pound(a_day), *_CHEAP_WEEK, span
            ));
        }
        if mouths == 0 {
            return Some(format!(
                "{} {} is {} a day on food, which covers the basics for a household if you cook. {} Do one shop for the whole {} rather than topping up, buy the own-brand version of everything on the list, and plan the meals round what is reduced at the end of the day.{}",
                _a_pound(amount),
                label,
                _a_pound(a_day),
                *_CHEAP_WEEK,
                span,
                if a_day < 3.0 { " That is tight for more than one, so ring Citizens Advice or the council's welfare line this week rather than on the last day: a food bank referral, a crisis grant or a payment break on a bill can all land in time." } else { "" }
            ));
        }
        let each = a_day / mouths as f64;
        let tight = if each < 3.0 {
            format!(" That is tight for {}, so ring Citizens Advice or the council's welfare line this week rather than on the last day: a food bank referral, a crisis grant or a payment break on a bill can all land in time.", _the_number(mouths))
        } else {
            String::new()
        };
        return Some(format!(
            "{} {} is {} a day on food, which is {} a head a day for {}. {} Do one shop for the whole {} rather than topping up, buy the own-brand version of everything on the list, and plan the meals round what is reduced at the end of the day.{}",
            _a_pound(amount),
            label,
            _a_pound(a_day),
            _a_pound(each),
            _the_number(mouths),
            *_CHEAP_WEEK,
            span,
            tight
        ));
    }
    if a_day < 2.0 {
        return Some(format!(
            "That is {} a day {}. Buy a day at a time and pay cash: a loaf, a bag of rice, a bag of lentils, eggs and a bag of frozen veg is what that buys, and the bus or the phone top-up comes last. Under {} a day is the point to ring Citizens Advice or the council's welfare line today rather than on the last day - a food bank referral, a crisis grant or a payment break on a bill can all land before then.",
            _a_pound(a_day),
            label,
            _a_pound(3.0)
        ));
    }
    if a_day < 3.0 {
        return Some(format!(
            "That is {} a day {}. Buy a day at a time and pay cash: {}, and the bus or the phone top-up comes last. Under {} a day is the point to ring Citizens Advice or the council's welfare line today rather than on the last day - a food bank referral, a crisis grant or a payment break on a bill can all land before then.",
            _a_pound(a_day),
            label,
            *_CHEAP_FOOD,
            _a_pound(3.0)
        ));
    }
    if a_day < 15.0 {
        return Some(format!(
            "That is {} a day {}. Buy the staples first - rice, pasta, eggs, tinned tomatoes, beans, frozen veg, oats, bananas, which feeds one person on about £2 to £3 a day - and do one shop for the whole stretch rather than topping up, because the top-ups are what eat a small sum. Keep the rest for the bus, the phone and anything that comes up, and pay by cash or a debit card so a small slip cannot take the week.",
            _a_pound(a_day),
            label
        ));
    }
    Some(format!(
        "That is {} a day {}, so food is comfortably covered: do one shop for the whole stretch, cook from the cheap staples rather than buying lunch out, and you will finish it with money left. Put whatever is spare at the end into the next stretch rather than spending it, because the week after a long one is the one that bites.",
        _a_pound(a_day),
        label
    ))
}

// ---------------------------------------------------------------------------------------------
// Pensions, council tax, splitting costs, credit, debts, borrowing

strs_g!(_PENSION_MONEY_MOVED);

/// "Text or words", lower-cased: `(str(text or "") or " ".join(words)).lower()`.
fn text_or_words_lower(u: &Understood) -> String {
    if !u.text.is_empty() { u.text.to_lowercase() } else { words_lower(u) }
}

/// A pension scam: money already moved, or the shape of one to avoid.
pub fn _pension_scam(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = text_or_words_lower(understood);
    if sh::any_in(&line, &_PENSION_MONEY_MOVED) {
        return Some("Ring the bank's fraud line now, on the number on the back of the card or from the bank's own site, and say what was taken and when: a payment still in transit can often be recalled and the account frozen. Then report it to Action Fraud on 0300 123 2040 and to the Pensions Advisory Service, and ring the provider whose name was used, because they can flag the transfer at their end too. Do all of that today rather than after the weekend.".to_string());
    }
    Some("Nobody legitimate cold-calls, emails or knocks about your pension: a call, a free review, a guaranteed better return or a hurry to move the pot is the shape of a scam. Check the firm on the FCA register, check the scheme with the Pension Regulator, and book a free Pension Wise appointment if you are 50 or over, before you sign anything. Ring your provider on the number on your own paperwork, never the one the caller gives you, and never move money to a scheme you did not find yourself.".to_string())
}

/// Council tax: the cost, instalments, refunds, the name on the bill, who counts, discounts.
pub fn _council_tax(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = text_or_words_lower(understood);
    let any = |ws: &[&str]| ws.iter().any(|w| line.contains(w));
    let s = |x: &str| Some(x.to_string());
    if any(&["how much", "how much is", "average", "typical", "cost of", "how expensive"]) {
        return s("It depends on your band and your council: a band D home in England averages about 2,200 pounds a year, band A about 1,450, and the bill is split over 10 months unless you ask for 12. Your council's own site lists every band for your postcode, and a band A to C property pays far less than the average suggests.");
    }
    if any(&["12 months", "twelve months", "instalments", "installments", "spread", "monthly", "over the year"]) {
        return s("Yes - councils bill over 10 months by default, and almost all will spread it over 12 if you ask, which lowers the monthly amount. Ring or use the council's online account before the first payment date, and if money is tight ask about the council tax reduction scheme at the same time.");
    }
    if any(&["refund", "refunds", "money back", "rebate", "credit balance", "in credit", "overpaid"]) {
        return s("An account in credit is refunded by the council, but only if you ask: ring the council tax line, say the date you moved out or the discount that was backdated, and give the bank details the direct debit came from - most councils pay within a fortnight once the account is closed. Keep the closing bill. If the money has not come after a month, put the request in writing, then complain to the council and after eight weeks to the Local Government Ombudsman, which is free.");
    }
    if any(&["name", "address", "in my name", "moving in", "moved in"]) {
        return s("Use the council's own online form to change the name or address on the bill - or ring them if you cannot find it - and keep the reference number it gives you. Do it the week you move: the single person's 25 per cent discount only starts from the date the council knows you are on your own, and a bill in the wrong name is how people end up with a missed payment they never saw.");
    }
    if any(&[
        "per person", "each person", "per household", "each household", "how many people", "who has to pay", "who pays", "counted",
        "counts as", "does my partner", "does my flatmate", "count my partner", "counts my partner", "my partner count",
        "count my flatmate", "two of us", "both of us", "in the house",
    ]) {
        return s("On the home, not on the people: one bill per household, so two adults in a flat pay one bill between them, split however you like, not one each. Living alone gets 25 per cent off, and some adults are not counted at all - full-time students, apprentices under 25, a live-in carer, and someone with a severe mental impairment - so a two-adult home where only one counts still gets the single person's discount. Tell the council who lives there when you move in, because the discounts only start from the date they know.");
    }
    if any(&["student", "students", "in uni", "at uni", "university"]) {
        return s("Full-time students do not pay council tax, so a household where everyone is a full-time student is exempt - tell the council and keep the certificate from your university. One part-time student or one working person on the bill and it is a discount instead of an exemption. It starts from the date the council knows, so report it the week you move rather than at the end of the year.");
    }
    if any(&["live alone", "living alone", "on my own", "by myself", "single person", "single occupant", "money off", "discount", "reduction"]) {
        return s("Living alone gets 25 per cent off the bill, and it is not automatic: ring the council or use its online account and ask for the single person discount, and it runs from the date they are told, not the date you moved in. If every other adult has moved out and one is left, the same 25 per cent applies. If your income is low, ask about the council's council tax reduction scheme at the same time - that is separate and can be much bigger.");
    }
    if any(&["empty", "nobody lives", "no one lives", "vacant", "unoccupied"]) {
        return s("An empty home loses its discounts and can be charged a premium: in England a property empty for two years or more can carry a 100 per cent premium, and some councils apply one sooner. Tell the council the date it became empty - a short spell while it is being sold or done up is usually exempt - and ask what their rule is, because it differs by council.");
    }
    if any(&["too high", "so high", "too much", "expensive", "high", "higher", "huge", "massive", "sky high"]) {
        return s("Check the band first, then the discounts, because a council tax bill is only 'too high' in one of those two ways. Compare your band with identical houses on the council's valuation list, which is free to challenge through the Valuation Office and is backdated if they agree, then confirm every discount you are owed: 25 per cent for one adult, an exemption for a full-time student household, and - the biggest one, and separate from the 25 per cent - the council's own council tax reduction scheme if the income is low. Ring the council tax line and ask them to go through the bill with you; they do it all day.");
    }
    s("The two ways a council tax bill is wrong are the band and the discounts. Check the band against similar houses on your council's valuation list and challenge it with the Valuation Office if two identical houses differ, which is free. Then check the discounts: 25 per cent for one adult, none for a full-time student household, and a reduction scheme if the income is low. Ring the council tax line and ask them to go through the bill with you; they do it all day.")
}

re_g!(
    _INCOME_COMPARISON, _FOOD, _HOME, _SHARED_FUEL, _SHARED_RIDE, _SHARED_STAY, _SHARED_TRIP, _SHARED_GIFT,
    _ASKING_IF_EXPECTED, _SPLIT_ASK,
);
str_g!(SPLIT_BILL_REPLY);

/// `" ".join(str(reading.get(key) or "") for key in ("clean", "text")).lower()`.
fn clean_and_text_lower(u: &Understood) -> String {
    format!("{} {}", u.clean, u.text).to_lowercase()
}

/// Splitting a bill: declines for a couple comparing incomes, else the table rule.
pub fn _split_bill(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    // The joined pair is never empty (it holds the space), so the words are never read.
    let said = clean_and_text_lower(understood);
    if _INCOME_COMPARISON.is_match(&said) && !_FOOD.is_match(&said) {
        return Some(String::new());
    }
    Some(SPLIT_BILL_REPLY.clone())
}

/// Splitting costs: the table, or the home when a turn of the chat names one.
pub fn _split_costs(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = if understood.text.is_empty() { " ".to_string() } else { understood.text.to_lowercase() };
    let table = "Agree the rule before the bill arrives: split food evenly and let whoever drank pay for drinks, or use an app that itemises. Say the number out loud before you order the pudding, because the argument afterwards is almost never about the money.";
    if _FOOD.is_match(&line) {
        return Some(table.to_string());
    }
    let mut said = vec![line];
    for turn in &memory.turns {
        said.push(turn.user.clone());
    }
    if _HOME.is_match(&said.join(" ").to_lowercase()) {
        return Some("Split it by room if the rooms differ, or by income if one of you earns much more, and agree the rule before the first bill rather than after it. Pay the house bills from one shared account that each of you tops up on payday, with the amounts written down, so nobody fronts the lot and nobody has to ask twice.".to_string());
    }
    Some(table.to_string())
}

/// Sharing a cost: fuel, a fare, a place to stay, a trip, a whip round, or the general rule.
pub fn _share_a_cost(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = clean_and_text_lower(understood);
    let s = |x: &str| Some(x.to_string());
    if _SHARED_FUEL.is_match(&line) {
        return s("Split fuel by the miles each person actually drove, not by who was in the car: note the odometer when you set off and when you get back, keep the receipt for the tank, and divide it by the miles rather than by the passengers. Agree before you set off whether the owner's wear is part of it - tyres and servicing are already in the driver's own costs, and a passenger is paying for the journey. One person pays at the pump, the rest send their share the same day, because a tank remembered a fortnight later is always argued about.");
    }
    if _SHARED_RIDE.is_match(&line) {
        return s("Split a fare by the journey each person actually made, not by head: whoever was picked up first and dropped last pays most, and a taxi app already shows the breakdown, so screenshot it into the group chat the same night. Say who is paying the driver before anyone gets in, and let one person book and collect, so there is a single number to divide rather than three arguments about it.");
    }
    if _SHARED_STAY.is_match(&line) {
        return s("Split a shared place by the room and the nights, not by head: a couple in the double pays more than the person on the sofa bed, and anyone staying three nights out of seven pays for three. Work the total out per room per night, round it to something easy, and take the deposits from everyone before the booking is made so the person whose card it is on is never the one chasing.");
    }
    if _SHARED_TRIP.is_match(&line) {
        return s("Book the big things - the flights, the house, the car - to one card and split those equally, or by room if the rooms differ, then have everyone put the same amount into a kitty for meals, taxis and tickets. Photograph every receipt and count the kitty on the last night rather than after you get home, and refund what is left to the person who paid it in, because a group holiday is a lot of small shared costs rather than one bill.");
    }
    if _SHARED_GIFT.is_match(&line) {
        return s("Fix the amount before anybody is asked, and ask for it once: a whip round works when the target is a number - 'twenty pounds each, I'll get the vouchers' - and fails when it is a collection with no total. Buy from one person's card, keep the receipt and the list of who has paid in the group chat, and cover any shortfall yourself rather than going round a second time.");
    }
    s("Say the number before the thing is booked or ordered, then split by what each person actually had rather than by head, and keep the reckoning in one place - a note in the group chat or a splitting app - so nobody has to trust their memory. Whoever pays up front collects, and it is fair to ask the same day rather than a fortnight later.")
}

/// True when the line asks how to share a cost, not whether one is expected of them.
pub fn _shares_a_cost(understood: &Understood) -> bool {
    let said = clean_and_text_lower(understood);
    if _ASKING_IF_EXPECTED.is_match(&said) {
        return false;
    }
    _SPLIT_ASK.is_match(&said)
}

str_g!(CREDIT_SCORE_GOOD, CREDIT_SCORE_IS, CREDIT_SCORE_UP, CREDIT_SCORE_MISSED, CREDIT_SCORE_DROP, CREDIT_SCORE_STEPS);

/// A credit score question: what it is, what is good, why it moved, or how to lift it.
pub fn _credit_score(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let said = text::squash(&reading.clean.to_lowercase());
    let any = |ws: &[&str]| ws.iter().any(|w| said.contains(w));
    if reading.asks.iter().any(|a| a == "definition") && !rx!(r"\b(?:my|our)\b").is_match(&said) {
        if any(&["good", "great", "decent", "healthy", "normal", "average", "ok"]) {
            return Some(CREDIT_SCORE_GOOD.clone());
        }
        return Some(CREDIT_SCORE_IS.clone());
    }
    if any(&["went up", "gone up", "goes up", "go up", "going up", "rose", "risen", "rises", "increased", "improved", "jumped up", "climbed"]) {
        return Some(CREDIT_SCORE_UP.clone());
    }
    if any(&[
        "missing a payment", "miss a payment", "missed a payment", "missed payment", "late payment", "paid late", "pay late",
        "payment late", "miss a direct debit", "missed a direct debit",
    ]) {
        return Some(CREDIT_SCORE_MISSED.clone());
    }
    if any(&[
        "dropped", "drop", "drops", "gone down", "went down", "fallen", "fell", "lower", "decreased", "why has", "why did",
        "what happened", "changed", "change",
    ]) {
        return Some(CREDIT_SCORE_DROP.clone());
    }
    Some(CREDIT_SCORE_STEPS.clone())
}

/// (name, pattern) per formal debt plan.
static _DEBT_PLAN_ASKS: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    data::get("bot.knowledge.money._DEBT_PLAN_ASKS")
        .items()
        .iter()
        .map(|p| (p.items()[0].str().to_string(), p.items()[1].as_re().expect("pattern")))
        .collect()
});

/// The formal debt plan the line names, or the comparison of them all.
pub fn _debt_plans(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let plans = data::get("bot.knowledge.money._DEBT_PLANS");
    let clean = format!(" {} ", said_lower(understood));
    let named: Vec<&String> = _DEBT_PLAN_ASKS.iter().filter(|(_, re)| re.is_match(&clean)).map(|(n, _)| n).collect();
    let key = if named.len() == 1 { named[0].as_str() } else { "compare" };
    Some(plans.get(key).unwrap_or_else(|| panic!("KeyError: {key}")).str().to_string())
}

str_g!(_WINDFALL_PLAN, _BORROW_ANSWER, _FIRST_HOME_STEPS, _FIRST_HOME_STEPS_SAVING);
strs_g!(_MONEY_IN_A_BORROW_LINE, _HOUSE_JOB_WORDS, _HOUSE_BUYING_WORDS, _STILL_SAVING, _ALREADY_BUYING);

/// What to do with a windfall, unless it is a small sum or a monthly figure.
pub fn _windfall_plan(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = said_lower(understood);
    let size = _target(&text);
    if size != 0 && size < int_g("_SMALL_SUM") {
        return None;
    }
    if size == 0 && _per_month(&text) != 0 {
        return None;
    }
    Some(_WINDFALL_PLAN.clone())
}

/// How to borrow, when the line is about money (a money word or a figure).
pub fn _borrow_how(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = said_lower(understood);
    let words = rx!("[a-z£]+").findall_str(&said);
    if words.iter().any(|w| _MONEY_IN_A_BORROW_LINE.contains(w)) {
        return Some(_BORROW_ANSWER.clone());
    }
    if rx!(r"\d{2,}").is_match(&said) {
        return Some(_BORROW_ANSWER.clone());
    }
    None
}

/// True unless the line is about a job on the house rather than buying one.
pub fn _not_a_job_on_the_house(understood: &Understood) -> bool {
    let text = said_lower(understood);
    let words: std::collections::HashSet<String> = rx!("[a-z0-9']+").findall_str(&text).into_iter().collect();
    if !_HOUSE_JOB_WORDS.iter().any(|w| words.contains(w)) {
        return true;
    }
    _HOUSE_BUYING_WORDS.iter().any(|w| words.contains(w))
        || _HOUSE_BUYING_WORDS.iter().filter(|p| p.contains(' ')).any(|p| text.contains(p.as_str()))
}

/// First home steps: still saving, or already buying.
pub fn _first_home_steps(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.as_str();
    if sh::any_in(clean, &_STILL_SAVING) {
        return Some(_FIRST_HOME_STEPS_SAVING.clone());
    }
    if sh::any_in(clean, &_ALREADY_BUYING) {
        return Some(_FIRST_HOME_STEPS.clone());
    }
    Some(_FIRST_HOME_STEPS_SAVING.clone())
}

strs_g!(_GIFT_FROM_WORK, _GIFT_ARRIVES);

/// The tax on a gift of money (or the prize-scam answer when the "gift" asks for money first).
pub fn _the_tax_on_a_gift(memory: &mut Memory, reading: &Understood) -> Option<String> {
    if _a_claim_that_asks_for_money_first(reading) {
        return _prize_answer(memory, reading);
    }
    let clean = format!(" {} ", text::squash(&reading.clean.to_lowercase()));
    let tail = " The one tax a gift can touch is inheritance tax, and that is about the giver's estate rather than about you: a gift made more than seven years before they die is usually outside it, £3,000 a year is free, £250 per person per year in small gifts is free, and a wedding gift of up to £5,000 from a parent is free. Gifts out of spare income, not savings, are free however big they are. Keep a note of the date and the amount, because the estate has to prove the seven years if it ever comes up, and if the giver may need care in the next few years, that question is looked at separately from tax.";
    if sh::any_in(&clean, &_GIFT_FROM_WORK) {
        return Some(format!("Nothing if it comes from a person, but a present from an employer is the one exception: a trivial benefit of up to £50 a year is tax free, while a voucher, gift card or hamper from work can be taxed as a benefit in kind, so ask payroll. Money given to you by anyone else is not income, so there is nothing to pay and nothing to declare.{tail}"));
    }
    if sh::any_in(&clean, &_GIFT_ARRIVES) {
        return Some(format!("Nothing. Money somebody gives you is not income, so there is no tax to pay on it and nothing to declare to HMRC.{tail}"));
    }
    Some(format!("Nothing at the time: the UK has no gift tax, so giving money away costs neither of you anything now, and the person who gets it declares nothing.{tail}"))
}

/// False when "pip" is a pet's name rather than the benefit.
pub fn _pip_the_benefit(understood: &Understood) -> bool {
    let text = sh::or(&understood.text, &understood.clean).to_lowercase();
    if rx!(r"\b(?:cat|cats|kitten|kittens|dog|dogs|puppy|puppies|hamster|rabbit|bunny|bird|birds|parrot|budgie|guinea|ferret|chick|chicks|hen|hens|pet|pets)\b").is_match(&text) {
        return false;
    }
    !rx!(r"\b(?:called|named|name is|nickname)\b[^.?!]{0,12}\bpip\b").is_match(&text)
}

static _DEBT_OWING: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.money._DEBT_OWING"));
static _DEBT_OWED: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.money._DEBT_OWED"));
static _DEBT_FIRST: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.money._DEBT_FIRST"));
static _DEBT_THIRD: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.money._DEBT_THIRD"));

/// Which way a debt runs: "owed" (money is owed to the speaker), "owing" (they owe it), or None.
pub fn a_debt_that_runs_one_way(understood: &Understood) -> Option<&'static str> {
    let has = |set: &[String], w: &str| set.iter().any(|x| x == w);
    for clause in &understood.clauses {
        for (i, word) in clause.iter().enumerate() {
            if !has(&_DEBT_OWING, word) && !has(&_DEBT_OWED, word) {
                continue;
            }
            let from = i.saturating_sub(6);
            let Some(subject) = clause[from..i].iter().rev().find(|w| has(&_DEBT_FIRST, w) || has(&_DEBT_THIRD, w)) else {
                continue;
            };
            let first = has(&_DEBT_FIRST, subject);
            if has(&_DEBT_OWED, word) {
                return Some(if first { "owed" } else { "owing" });
            }
            return Some(if first { "owing" } else { "owed" });
        }
    }
    None
}

str_g!(_THE_CHASE_SCRIPT, _THE_DEBTOR_SCRIPT, _WHAT_INHERITANCE_TAX_IS);

/// Chasing money they are owed, or paying back money they owe.
pub fn _the_debt_answer(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    if a_debt_that_runs_one_way(understood) == Some("owed") {
        return Some(_THE_CHASE_SCRIPT.clone());
    }
    Some(_THE_DEBTOR_SCRIPT.clone())
}

/// What inheritance tax is (or the prize-scam answer when a "legacy" asks for money first).
pub fn _the_inheritance_tax(memory: &mut Memory, reading: &Understood) -> Option<String> {
    if _a_claim_that_asks_for_money_first(reading) {
        return _prize_answer(memory, reading);
    }
    Some(_WHAT_INHERITANCE_TAX_IS.clone())
}

re_g!(_THE_ESTATE_BEING_WOUND_UP);

/// A win or a legacy that asks for money first, unless a real estate is being wound up.
pub fn _a_claim_that_asks_for_money_first(understood: &Understood) -> bool {
    let text = said_lower(understood);
    if text.is_empty() || _THE_ESTATE_BEING_WOUND_UP.is_match(&text) {
        return false;
    }
    _a_prize_and_a_bill(understood)
}

/// True when the line names no country.
pub fn _no_place_named(understood: &Understood) -> bool {
    let clean = text::squash(&understood.clean.to_lowercase());
    dup::_countries_said(&clean).is_empty()
}

re_g!(_CASH_REFUSED, _CASH_ON_HAND, A_SUPPLY_STOPPED, A_SUPPLIED_THING);

/// Cash or card: a shop refusing one, cash to keep at home, or which to use when.
pub fn _cash_or_card(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let clean = text::squash(&reading.clean.to_lowercase());
    if _CASH_REFUSED.is_match(&clean) {
        return Some("A shop can set its own payment terms, so it can say card only or cash only as long as it says so before you buy - 'legal tender' only means a court has to accept it for a debt, which is why a bus or a corner shop can turn down a £50 note. What it cannot do is take your money and then refuse the only way you can pay, or refuse you for a protected reason. If it is a service you have no alternative to, such as a council or an NHS car park, complain in writing and quote the terms they published.".to_string());
    }
    if _CASH_ON_HAND.is_match(&clean) {
        return Some("Keep about £50 to £100 at home in small notes, and a few pound coins: enough for a power cut, a taxi or a market stall on a day the card machines are down, and not so much that losing it hurts. Keep it somewhere dry and not in the obvious drawer, and top it up when you spend it rather than all at once.".to_string());
    }
    Some("Both, for different jobs. A card - credit for anything over £100 - is the one with protection: Section 75 makes the card company jointly liable when something never arrives or is not what was sold, and a debit card gives you a chargeback instead. Cash is the one with a limit: an envelope of notes for the week is spending you can see run out, which is why it works for people who overspend on the card, and small shops, markets and tips still run on it. So pay for anything you might have to claim on by card, and use cash for the everyday spending you want to cap.".to_string())
}

/// True when the line is about a supplied bill (or a supply stopped), not a person.
pub fn _a_bill_and_not_a_person(understood: &Understood) -> bool {
    let mut said = words_lower(understood);
    if said.is_empty() {
        said = said_lower(understood);
    }
    if A_SUPPLY_STOPPED.is_match(&said) {
        return true;
    }
    A_SUPPLIED_THING.is_match(&said)
}

str_g!(_ENERGY_VERDICT, _ENERGY_EXPLANATION);
strs_g!(_ENERGY_WANTS_THE_VERDICT, _THE_TAX_OFFICE, _NOT_THE_TAX_OFFICES_REFUND);

/// The verdict on fixing a tariff when the line asks for one, else what the cap is.
pub fn _energy_tariff(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    if reading.asks.iter().any(|a| _ENERGY_WANTS_THE_VERDICT.contains(a)) {
        return Some(_ENERGY_VERDICT.clone());
    }
    Some(_ENERGY_EXPLANATION.clone())
}

/// True unless the refund in the line is a shop's rather than the tax office's.
pub fn _a_refund_the_tax_office_sent(reading: &Understood) -> bool {
    let words: Vec<String> = reading.words.iter().map(|w| w.to_lowercase()).collect();
    if reading.text.to_lowercase().contains("inland revenue") {
        return true;
    }
    if words.iter().any(|w| _THE_TAX_OFFICE.contains(w)) {
        return true;
    }
    !words.iter().any(|w| _NOT_THE_TAX_OFFICES_REFUND.contains(w))
}

// ---------------------------------------------------------------------------------------------
// Care costs, scams and prizes, transfers, the care assessment

str_g!(_CARE_COSTS_HELP, _CARE_COSTS_MEANS, _NOT_MEANS_TESTED);
strs_g!(_CARE_HELP_WORDS);

/// What help there is with care costs, or how the means test works.
pub fn _care_costs_answer(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let said = said_lower(reading);
    if sh::any_in(&said, &_CARE_HELP_WORDS) {
        return Some(_CARE_COSTS_HELP.clone());
    }
    Some(_CARE_COSTS_MEANS.clone())
}

/// After a scam: where to report it, or how to get the money back.
pub fn _scam_steps(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = said_lower(understood);
    let any = |ws: &[&str]| ws.iter().any(|w| text.contains(w));
    let money = any(&["money back", "get it back", "refund", "recover", "reclaim", "lost", "stolen", "took", "taken"]);
    if !money && any(&["report", "reporting", "action fraud", "police", "tell someone", "where do i"]) {
        return Some("Report it to Action Fraud on 0300 123 2040 or at actionfraud.police.uk - the police do not take scam reports directly - and tell your bank even so, because a payment that has not left can still be stopped. Keep every message, phone number and payment reference: the report needs the sender's details, the date and the amount. If it came through a text or an app, forward it to 7726 and block the number.".to_string());
    }
    Some("Ring your bank the same day: a transfer can sometimes be stopped before it leaves, and the reimbursement rules for authorised push-payment scams mean most banks have to refund you. Then report it to Action Fraud on 0300 123 2040, and keep every message, phone number and payment reference. Change the password on whatever account it came through. Never pay anyone who rings offering to get the money back: that is the same scam running again.".to_string())
}

/// A prize that asks for money first is the scam.
pub fn _prize_answer(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = said_lower(understood);
    if [
        "never entered", "did not enter", "didnt enter", "didn't enter", "never applied", "never signed up", "never signed up for",
        "dont remember entering", "do not remember entering", "never put my name",
    ]
    .iter()
    .any(|w| text.contains(w))
    {
        return Some("You cannot win a competition you never entered, so that is a scam: send nothing, ring nobody, and do not confirm your details to sort out the winnings. Real lotteries, draws and prize sites publish winners themselves and never ask a winner for a fee, a delivery charge or bank details first. Keep the message and report it to Action Fraud on 0300 123 2040, and forward a suspicious text to 7726.".to_string());
    }
    Some("Nothing genuine asks you for money first: a fee, a 'release charge', a tax payment or your bank details before a prize arrives is the whole scam. Send nothing, do not ring the number in the letter or text, and check the organisation through a number you find yourself rather than the one they gave you. A real draw publishes its winners and pays by the method you entered with. Keep everything and report it to Action Fraud on 0300 123 2040, and forward a suspicious text to 7726.".to_string())
}

re_g!(
    _ABOUT_THE_WIN, _A_RELATIVE_TOLD_ME, _TAX_ASKED, _A_MESSAGE_THAT_CLAIMS_A_WIN, _A_MESSAGE_THAT_CLAIMS_MONEY,
    _NEVER_ENTERED, _A_WIN_THEN_A_MESSAGE, _ASKS_FOR_SOMETHING_BACK, _BANK_DETAILS_ASKED, _A_WIN_OR_AN_INHERITANCE,
    _A_WIN_NAMED, _MONEY_LEFT_NAMED, _MONEY_OUT, _ASKS_ABOUT_THE_ASSESSMENT,
);

/// True when the line is a claimed win or legacy that wants money (or details) first.
pub fn _a_prize_and_a_bill(understood: &Understood) -> bool {
    let text = said_lower(understood);
    let t = text.as_str();
    if t.is_empty() {
        return true;
    }
    if _ABOUT_THE_WIN.is_match(t) || _A_RELATIVE_TOLD_ME.is_match(t) {
        return false;
    }
    if _TAX_ASKED.is_match(t) {
        return false;
    }
    if _A_MESSAGE_THAT_CLAIMS_A_WIN.is_match(t) || _A_MESSAGE_THAT_CLAIMS_MONEY.is_match(t) {
        return true;
    }
    if _NEVER_ENTERED.is_match(t) {
        return true;
    }
    if _A_WIN_THEN_A_MESSAGE.is_match(t) {
        return _ASKS_FOR_SOMETHING_BACK.is_match(t);
    }
    if _BANK_DETAILS_ASKED.is_match(t) && _A_WIN_OR_AN_INHERITANCE.is_match(t) {
        return true;
    }
    if _A_WIN_NAMED.is_match(t) || _MONEY_LEFT_NAMED.is_match(t) {
        return _MONEY_OUT.is_match(t);
    }
    false
}

/// How long a bank transfer takes: CHAPS, international, or Faster Payments.
pub fn _transfer_time(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = said_lower(understood);
    if text.contains("chaps") {
        return Some("CHAPS is the same-day one, for house deposits and car money: sent by about 3pm on a working day it lands that day, and after that it is the next working day. It costs £20 to £30 a transfer, the bank may ring you to confirm it, and it is the wrong scheme to use for a few hundred pounds - Faster Payments is free and arrives in seconds.".to_string());
    }
    if ["international", "overseas", "abroad", "another country", "europe", "usa", "america", "australia", "india", "spain"]
        .iter()
        .any(|w| text.contains(w))
    {
        return Some("An international transfer takes one to three working days, and it can sit in a check for longer if the bank wants more detail. Send it before about 4pm on a working day, because after that it is really being sent tomorrow, and expect a fee plus a poor exchange rate. A UK transfer is much faster: Faster Payments lands in seconds, even if the first payment to a new payee can be held a few hours.".to_string());
    }
    Some("A UK transfer over Faster Payments normally lands in seconds and is almost always there the same day, though a bank can hold the first payment to a new payee for a few hours while it checks. CHAPS is for big same-day payments and costs a fee. An international transfer takes one to three working days, and anything sent after about 4pm is really being sent tomorrow.".to_string())
}

/// The "not means-tested" sentence when the line asks about the assessment, else "".
pub fn _the_assessment_sentence(understood: &Understood) -> String {
    let said = text::squash(&said_lower(understood));
    if _ASKS_ABOUT_THE_ASSESSMENT.is_match(&said) { _NOT_MEANS_TESTED.clone() } else { String::new() }
}

// ---------------------------------------------------------------------------------------------
// The entries' guards and replies, by Python id

/// `lambda reading: not owns_the_deed(reading) and not _a_manners_question_not_a_home_split(reading)`
/// (line 5563): a bill kept back is the repair, and a table's manners question is the judgement row's.
fn _lambda_5563(u: &Understood) -> bool {
    !sh::owns_the_deed_reading(u) && !_a_manners_question_not_a_home_split(u)
}

/// `lambda u: not _a_receipt_question(u)` (line 7482).
fn _lambda_7482(u: &Understood) -> bool {
    !_a_receipt_question(u)
}

/// `lambda reading: not _a_manners_question_not_a_home_split(reading)` (line 8529).
fn _lambda_8529(u: &Understood) -> bool {
    !_a_manners_question_not_a_home_split(u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.money:<lambda>:5563", |_: &FnRef, u: &Understood| _lambda_5563(u)),
    ("bot.knowledge.money:<lambda>:7482", |_: &FnRef, u: &Understood| _lambda_7482(u)),
    ("bot.knowledge.money:<lambda>:8529", |_: &FnRef, u: &Understood| _lambda_8529(u)),
    ("bot.knowledge.money:_a_bill_and_not_a_person:3836", |_: &FnRef, u: &Understood| _a_bill_and_not_a_person(u)),
    ("bot.knowledge.money:_a_car_or_a_deal:521", |_: &FnRef, u: &Understood| _a_car_or_a_deal(u)),
    ("bot.knowledge.money:_a_figure_for_a_plan_is_not_a_debt:560", |_: &FnRef, u: &Understood| _a_figure_for_a_plan_is_not_a_debt(u)),
    ("bot.knowledge.money:_a_month_and_no_sum:1737", |_: &FnRef, u: &Understood| _a_month_and_no_sum(u)),
    ("bot.knowledge.money:_a_payment_count:2640", |_: &FnRef, u: &Understood| _a_payment_count(u)),
    ("bot.knowledge.money:_a_plan_and_no_trouble:579", |_: &FnRef, u: &Understood| _a_plan_and_no_trouble(u)),
    ("bot.knowledge.money:_a_prize_and_a_bill:10748", |_: &FnRef, u: &Understood| _a_prize_and_a_bill(u)),
    ("bot.knowledge.money:_a_purse_in_trouble:2677", |_: &FnRef, u: &Understood| _a_purse_in_trouble(u)),
    ("bot.knowledge.money:_a_refund_the_tax_office_sent:3903", |_: &FnRef, u: &Understood| _a_refund_the_tax_office_sent(u)),
    ("bot.knowledge.money:_no_place_named:3758", |_: &FnRef, u: &Understood| _no_place_named(u)),
    ("bot.knowledge.money:_not_a_customs_line:88", |_: &FnRef, u: &Understood| _not_a_customs_line(u)),
    ("bot.knowledge.money:_not_a_gift:2349", |_: &FnRef, u: &Understood| _not_a_gift(u)),
    ("bot.knowledge.money:_not_a_job_on_the_house:3552", |_: &FnRef, u: &Understood| _not_a_job_on_the_house(u)),
    ("bot.knowledge.money:_pip_the_benefit:3615", |_: &FnRef, u: &Understood| _pip_the_benefit(u)),
    ("bot.knowledge.money:_shares_a_cost:3299", |_: &FnRef, u: &Understood| _shares_a_cost(u)),
    ("bot.knowledge.money:_the_rate_row_has_a_rate:2572", |_: &FnRef, u: &Understood| _the_rate_row_has_a_rate(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.money:_a_gift_budget:1840", |_, m, u| _a_gift_budget(m, u)),
    ("bot.knowledge.money:_borrow_how:3485", |_, m, u| _borrow_how(m, u)),
    ("bot.knowledge.money:_budget_split:2440", |_, m, u| _budget_split(m, u)),
    ("bot.knowledge.money:_car_running_cost:493", |_, m, u| _car_running_cost(m, u)),
    ("bot.knowledge.money:_card_payoff:901", |_, m, u| _card_payoff(m, u)),
    ("bot.knowledge.money:_care_costs_answer:10365", |_, m, u| _care_costs_answer(m, u)),
    ("bot.knowledge.money:_cash_or_card:3769", |_, m, u| _cash_or_card(m, u)),
    ("bot.knowledge.money:_council_tax:3029", |_, m, u| _council_tax(m, u)),
    ("bot.knowledge.money:_credit_score:3325", |_, m, u| _credit_score(m, u)),
    ("bot.knowledge.money:_debt_plans:3415", |_, m, u| _debt_plans(m, u)),
    ("bot.knowledge.money:_dish_to_cook_cost:2272", |_, m, u| _dish_to_cook_cost(m, u)),
    ("bot.knowledge.money:_energy_tariff:3883", |_, m, u| _energy_tariff(m, u)),
    ("bot.knowledge.money:_first_home_steps:3562", |_, m, u| _first_home_steps(m, u)),
    ("bot.knowledge.money:_food_bill:304", |_, m, u| _food_bill(m, u)),
    ("bot.knowledge.money:_habit_cost:2130", |_, m, u| _habit_cost(m, u)),
    ("bot.knowledge.money:_habit_plan:351", |_, m, u| _habit_plan(m, u)),
    ("bot.knowledge.money:_house_deposit:1015", |_, m, u| _house_deposit(m, u)),
    ("bot.knowledge.money:_isa:370", |_, m, u| _isa(m, u)),
    ("bot.knowledge.money:_late_leg:442", |_, m, u| _late_leg(m, u)),
    ("bot.knowledge.money:_make_it_last:2906", |_, m, u| _make_it_last(m, u)),
    ("bot.knowledge.money:_missed_payment:2705", |_, m, u| _missed_payment(m, u)),
    ("bot.knowledge.money:_money_to_last:1999", |_, m, u| _money_to_last(m, u)),
    ("bot.knowledge.money:_money_trouble:2769", |_, m, u| _money_trouble(m, u)),
    ("bot.knowledge.money:_mortgage_afford:1264", |_, m, u| _mortgage_afford(m, u)),
    ("bot.knowledge.money:_over_spending:425", |_, m, u| _over_spending(m, u)),
    ("bot.knowledge.money:_pay_off:854", |_, m, u| _pay_off(m, u)),
    ("bot.knowledge.money:_pension:617", |_, m, u| _pension(m, u)),
    ("bot.knowledge.money:_pension_choice:2405", |_, m, u| _pension_choice(m, u)),
    ("bot.knowledge.money:_pension_for_someone:591", |_, m, u| _pension_for_someone(m, u)),
    ("bot.knowledge.money:_pension_scam:3008", |_, m, u| _pension_scam(m, u)),
    ("bot.knowledge.money:_prize_answer:10632", |_, m, u| _prize_answer(m, u)),
    ("bot.knowledge.money:_put_by:1629", |_, m, u| _put_by(m, u)),
    ("bot.knowledge.money:_rent_deposit:2327", |_, m, u| _rent_deposit(m, u)),
    ("bot.knowledge.money:_retirement:2355", |_, m, u| _retirement(m, u)),
    ("bot.knowledge.money:_save_goal:1595", |_, m, u| _save_goal(m, u)),
    ("bot.knowledge.money:_savings_gap:2601", |_, m, u| _savings_gap(m, u)),
    ("bot.knowledge.money:_scam_steps:10572", |_, m, u| _scam_steps(m, u)),
    ("bot.knowledge.money:_share_a_cost:3253", |_, m, u| _share_a_cost(m, u)),
    ("bot.knowledge.money:_split_bill:3147", |_, m, u| _split_bill(m, u)),
    ("bot.knowledge.money:_stamp_duty:1091", |_, m, u| _stamp_duty(m, u)),
    ("bot.knowledge.money:_take_home:249", |_, m, u| _take_home(m, u)),
    ("bot.knowledge.money:_the_debt_answer:3715", |_, m, u| _the_debt_answer(m, u)),
    ("bot.knowledge.money:_the_inheritance_tax:3722", |_, m, u| _the_inheritance_tax(m, u)),
    ("bot.knowledge.money:_the_survey:55", |_, m, u| _the_survey(m, u)),
    ("bot.knowledge.money:_the_tax_on_a_gift:3588", |_, m, u| _the_tax_on_a_gift(m, u)),
    ("bot.knowledge.money:_time_to_save:1453", |_, m, u| _time_to_save(m, u)),
    ("bot.knowledge.money:_transfer_time:10826", |_, m, u| _transfer_time(m, u)),
    ("bot.knowledge.money:_windfall_plan:3447", |_, m, u| _windfall_plan(m, u)),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Turn;

    fn turns(lines: &[&str]) -> Memory {
        let mut m = Memory::new();
        m.turns = lines.iter().map(|l| Turn { user: l.to_string(), bot: String::new(), module: None }).collect();
        m
    }

    fn reading(text: &str) -> Understood {
        Understood { text: text.to_string(), clean: text.to_lowercase(), ..Default::default() }
    }

    // Expected values are PyBot's own, for the helpers no registered entry reaches.
    #[test]
    fn helpers_match_pybot() {
        assert!(_they_owe(&turns(&["hi", "I owe my sister 200 pounds"])));
        assert!(!_they_owe(&turns(&["I owe my sister 200", "a", "b", "c"])));
        assert!(!_they_owe(&Memory::new()));
        let home = turns(&["my flatmate and i rent a flat"]);
        for (text, with_home, alone, assessment) in [
            ("How do we split the bill at dinner", "Agree the rule", "Agree the rule", ""),
            ("How do we split the bills", "Split it by room", "Agree the rule", ""),
            ("Is care means tested", "Split it by room", "Agree the rule", "It is not means-tested, so sav"),
            ("what does it cost", "Split it by room", "Agree the rule", ""),
        ] {
            let u = reading(text);
            assert!(_split_costs(&mut home.clone(), &u).unwrap().starts_with(with_home), "{text}");
            assert!(_split_costs(&mut Memory::new(), &u).unwrap().starts_with(alone), "{text}");
            assert!(_the_assessment_sentence(&u).starts_with(assessment), "{text}");
        }
        let m = MONEY_RE.search("it was 3k").unwrap();
        assert_eq!(_sum_amount(&m), 3000);
    }
}
