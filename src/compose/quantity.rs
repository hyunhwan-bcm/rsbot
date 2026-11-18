//! Port of `bot/compose/quantity.py`: the reply to a line that is nothing but a number - a sum, a
//! length of time, a distance, a size, a span or a weight - and the figure read against the rule
//! the last answer named.
#![allow(non_snake_case, clippy::all)]

use crate::compose::fallback_chain::py_format;
use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.quantity.", stringify!($name))));)*
    };
}
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.compose.quantity.", stringify!($name))));)*
    };
}
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.quantity.", stringify!($name))));)*
    };
}

regexes!(
    TRIP, DOUBT_ON_THE_FIGURE, SAVINGS_AFTER_A_RATE, INCOME, WEIGHT_ON_THE_TABLE, WEIGHT_GOING_ON, SUBJECT,
    SUM_IN_A_WORD, ON_THE_TABLE_DISTANCE, ON_THE_TABLE_MONEY, A_BUDGET_THEY_SET, SPAN_FRAME, A_QUESTION_ABOUT_A_LENGTH,
    A_BUDGET_QUESTION, A_BOUND, THEY_HAVE_THE_TIME, A_REMAINDER, THEY_HAVE_IT, A_CHANGE_SAID, I_WEIGH, SOMETHING_WEIGHED,
    A_COUNT_ASKED_FOR, A_WAY_OF_PAYING, THEIR_THING, A_RANGE_WITH_A_UNIT, A_SINGLE_RULE, A_FIGURE_ALONE, ABOUT_A_FIGURE,
    AN_AGE, NOT_A_UNIT_ITSELF,
);
word_sets!(LEAD, POINTING_BACK, A_PRONOUN, HEDGES, TAIL, BOUNDED_BY, NOT_A_THING);
pools!(
    CURRENCY, SAVINGS, MONEY_MONTH, MONEY_WEEK, MONEY_YEAR, MONEY_DAY, INCOME_MONTH, INCOME_WEEK, INCOME_YEAR,
    INCOME_PLAIN, MONEY_PLAIN, TIME_BUDGET, TIME_TRIP, TIME_A_DAY, TIME_LONG, TIME_WEEK, AREA, DISTANCE_NEAR,
    DISTANCE_MID, DISTANCE_FAR, DISTANCE_DAY_OUT, DISTANCE_FAR_TRIP, SPAN_DAYS, SPAN_WEEKS, SPAN_MONTHS, WEIGHT_LOSS,
    WEIGHT_PLAIN, WEIGHT_OF_A_THING, WEIGHT_THEIR_OWN, A_LIMIT_ON_A_LENGTH, A_WHAT_IS_LEFT, FIGURE_BELOW,
    FIGURE_INSIDE, FIGURE_ABOVE, TAKE,
);

fn named(name: &str) -> &'static Value {
    data::get(&format!("bot.compose.quantity.{name}"))
}
fn number(v: &Value) -> f64 {
    v.as_f64().or_else(|| v.as_int().map(|n| n as f64)).expect("a number")
}
/// `UNITS`: unit -> (family, factor).
static UNITS: LazyLock<HashMap<String, (String, f64)>> = LazyLock::new(|| {
    named("UNITS").pairs().iter().map(|(k, v)| (k.str().to_string(), (v.items()[0].str().to_string(), number(&v.items()[1])))).collect()
});
/// `NUMBERS`, in dict order (the size reading sorts the keys).
static NUMBERS: LazyLock<Vec<(String, i64)>> =
    LazyLock::new(|| named("NUMBERS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().expect("an int"))).collect());
/// `TENS`, in dict order.
static TENS: LazyLock<Vec<(String, i64)>> =
    LazyLock::new(|| named("TENS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().expect("an int"))).collect());
/// `PERIODS`: (said, name) in order.
static PERIODS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    named("PERIODS").items().iter().map(|row| (row.items()[0].str().to_string(), row.items()[1].str().to_string())).collect()
});
static MONEY_NOUN: LazyLock<HashMap<String, i64>> =
    LazyLock::new(|| named("MONEY_NOUN").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().expect("an int"))).collect());
static THE_HOUSEHOLD: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| named("THE_HOUSEHOLD").items().iter().map(Value::strs).collect());
static UNIT_SAID: LazyLock<HashMap<String, String>> = LazyLock::new(|| named("UNIT_SAID").str_map());
static PERIOD_NAMED: LazyLock<HashMap<String, String>> = LazyLock::new(|| named("PERIOD_NAMED").str_map());
static UNIT_IS: LazyLock<Vec<(&'static Re, String)>> = LazyLock::new(|| {
    named("UNIT_IS").items().iter().map(|row| (row.items()[0].as_re().expect("a pattern"), row.items()[1].str().to_string())).collect()
});
static STEPS: LazyLock<HashMap<String, HashMap<String, String>>> =
    LazyLock::new(|| named("STEPS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_map())).collect());
static A_STEP_EITHER_WAY: LazyLock<HashMap<String, String>> = LazyLock::new(|| named("A_STEP_EITHER_WAY").str_map());
static A_CHANGE_IS_THIS_SMALL: LazyLock<f64> = LazyLock::new(|| number(named("A_CHANGE_IS_THIS_SMALL")));
static A_STRETCH_THAT_IS_NOT_SHORT: LazyLock<f64> = LazyLock::new(|| number(named("A_STRETCH_THAT_IS_NOT_SHORT")));
static TEN_TIMES: LazyLock<f64> = LazyLock::new(|| number(named("TEN_TIMES")));
/// `understanding.topics.THINGS`.
static THINGS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.understanding.topics.THINGS"));
/// `prices.PRICE_JUDGEMENT`.
static PRICE_JUDGEMENT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.prices.PRICE_JUDGEMENT"));

static A_PLAIN_FIGURE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d+(?:\.\d+)?", 0));
static A_STUCK_UNIT: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"(?<=\d)(kilometres|kilometers|kilograms|kilogram|miles|mile|stones|kilos|kilo|stone|kms|km|mi|kgs|kg|lbs|lb)\b", crate::py::re::I)
});
static A_LETTER_RUN: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]+", 0));
/// The size reading's second measurement: `r"\bby\s+(\d+(?:\.\d+)?|" + "|".join(sorted(NUMBERS)) + r")\b"`.
static BY_A_SECOND: LazyLock<Re> = LazyLock::new(|| {
    let mut words: Vec<&str> = NUMBERS.iter().map(|(w, _)| w.as_str()).collect();
    words.sort();
    Re::new(&format!(r"\bby\s+(\d+(?:\.\d+)?|{})\b", words.join("|")), 0)
});

/// `compose.pick(options, memory, seed_text)`, through the one wrapper this file has.
fn pick(options: &[String], memory: &Memory, clean: &str) -> String {
    crate::compose::pick(options, &crate::compose::fallback::memory_value(memory), clean)
}

/// `s[:1].upper() + s[1:]`.
fn first_upper(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => text::upper(&c.to_string()) + chars.as_str(),
        None => String::new(),
    }
}

/// `" ".join(str(x).lower().split())`.
fn flat(s: &str) -> String {
    text::squash(&text::lower(s))
}

/// Python's `round(x)` on a float: half to even, as an int.
fn round_int(x: f64) -> i64 {
    text::round_half_even(x)
}

/// Python's `round(x, 1)`: the float correctly rounded to one decimal place.
fn round1(x: f64) -> f64 {
    format!("{x:.1}").parse().unwrap_or(x)
}

/// `str.isdigit()` of the first character.
fn digit_first(s: &str) -> bool {
    s.chars().next().is_some_and(|c| text::is_digit(&c.to_string()))
}

/// `word.lower().strip(".,!?:;")`.
fn plain_word(word: &str) -> String {
    text::strip_chars(&text::lower(word), ".,!?:;").to_string()
}

/// Python: bot/compose/quantity.py:370 `_number_at(words, at)`
/// (value, how many words it took) for the number starting at `words[at]`, or (None, 0).
pub fn _number_at(words: &[String], at: usize) -> (Option<f64>, usize) {
    let Some(first) = _amount(&words[at]) else { return (None, 0) };
    if at + 1 < words.len() {
        let nxt = text::lower(&words[at + 1]).replace('-', "");
        if nxt == "hundred" && first < 100.0 {
            return (Some(first * 100.0), 2);
        }
        // Only a units word after a tens word: "twenty five", not "forty a month".
        let low = text::lower(&words[at]);
        if TENS.iter().any(|(t, _)| *t == low) && ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine"].contains(&nxt.as_str()) {
            let units = NUMBERS.iter().find(|(w, _)| *w == nxt).map(|(_, n)| *n).unwrap_or_else(|| panic!("KeyError: {nxt:?}"));
            return (Some(first + units as f64), 2);
        }
    }
    (Some(first), 1)
}

/// Python: bot/compose/quantity.py:391 `_amount(word)`
/// The number a word gives, or None: "45", "£250", "twenty", "forty", "twentyfive".
pub fn _amount(word: &str) -> Option<f64> {
    let mut plain = word.replace(',', "");
    for mark in CURRENCY.iter() {
        plain = plain.replace(mark.as_str(), "");
    }
    let plain = plain.replace('-', "");
    if A_PLAIN_FIGURE.fullmatch(&plain).is_some() {
        return Some(decimal_float(&plain));
    }
    if let Some((_, n)) = NUMBERS.iter().find(|(w, _)| *w == plain) {
        return Some(*n as f64);
    }
    for (ten, base) in TENS.iter() {
        if let Some(rest) = plain.strip_prefix(ten.as_str()) {
            if let Some((_, n)) = NUMBERS.iter().find(|(w, _)| w == rest) {
                return Some((base + n) as f64);
            }
        }
    }
    None
}

/// `float(s)` for what `\d+(?:\.\d+)?` matched: Python's `\d` is any Unicode decimal digit, and
/// `float()` reads each by its value. A decimal digit's value is its place in its run of ten
/// (the runs start at zero), so the run's start is found by walking back over `\d`.
fn decimal_float(s: &str) -> f64 {
    static A_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d", 0));
    let is_digit = |c: char| A_DIGIT.fullmatch(&c.to_string()).is_some();
    let mut out = String::new();
    for c in s.chars() {
        if c == '.' || c.is_ascii_digit() {
            out.push(c);
            continue;
        }
        let mut start = c as u32;
        while start > 0 && char::from_u32(start - 1).is_some_and(is_digit) {
            start -= 1;
        }
        out.push(char::from_digit((c as u32 - start) % 10, 10).unwrap_or('0'));
    }
    out.parse().expect("ValueError: could not convert string to float")
}

/// Python: bot/compose/quantity.py:407 `_fmt(value)`
/// A number as a person writes it: whole numbers whole, halves with one decimal place.
pub fn _fmt(value: f64) -> String {
    if value.fract() == 0.0 && value.is_finite() {
        return text::thousands(round_int(value));
    }
    let one = format!("{value:.1}");
    let trimmed = one.trim_end_matches('0');
    trimmed.trim_end_matches('.').to_string()
}

/// Python: bot/compose/quantity.py:414 `_minutes(total)`
/// A length in minutes said the way people say it: "1 hour 20", "6 hours 40".
pub fn _minutes(total: f64) -> String {
    let total = round_int(total / 5.0) * 5; // people say "2 hours 10", not "2 hours 9"
    if total < 60 {
        return format!("{total} minutes");
    }
    let (hours, minutes) = (total.div_euclid(60), total.rem_euclid(60));
    let s = if hours == 1 { "" } else { "s" };
    if minutes == 0 {
        return format!("{hours} hour{s}");
    }
    format!("{hours} hour{s} {minutes}")
}

/// Python: bot/compose/quantity.py:438 `_the_sum_inside_the_word(text)`
/// ("a tenner", "... 10 ...") when the line names its sum with a word, else ("", text).
pub fn _the_sum_inside_the_word(text_: &str) -> (String, String) {
    let Some(found) = SUM_IN_A_WORD.search(text_) else { return (String::new(), text_.to_string()) };
    let word = text::lower(found.str(1));
    let sum = MONEY_NOUN.get(&word).copied().unwrap_or_else(|| panic!("KeyError: {word:?}"));
    (format!("a {word}"), format!("{}{sum}{}", &text_[..found.start()], &text_[found.end()..]))
}

/// Python: bot/compose/quantity.py:468 `_the_money_words(text)`
pub fn _the_money_words(text_: &str) -> bool {
    ON_THE_TABLE_MONEY.is_match(&text::lower(text_))
}

/// Python: bot/compose/quantity.py:472 `_about_a_distance(clean, memory)`
/// True when the thing just said, or the last turn or two, is a distance and not a sum.
pub fn _about_a_distance(clean: &str, memory: &Memory) -> bool {
    let mut elsewhere = vec![flat(clean)];
    let turns = &memory.turns;
    for turn in &turns[turns.len().saturating_sub(2)..] {
        elsewhere.push(turn.user.clone());
        elsewhere.push(turn.bot.clone());
    }
    if elsewhere.iter().any(|t| _the_money_words(t)) {
        return false;
    }
    elsewhere.iter().any(|t| ON_THE_TABLE_DISTANCE.is_match(&text::lower(t)))
}

/// What `_read` makes of a bare number line.
#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    pub value: f64,
    pub family: String,
    pub unit: String,
    pub period: Option<String>,
    pub phrase: String,
    pub tail: String,
}

/// Python: bot/compose/quantity.py:487 `_read(clean, memory=None)`
/// (value, family, unit, period, phrase, tail) for a bare number line, or six Nones. `phrase`
/// is the number as the person said it, and `tail` is what the words after it said it is about.
pub fn _read(clean: &str, memory: &Memory) -> Option<Reading> {
    let stripped = text::strip_chars(text::strip(clean), ".!,");
    // "3km", "5miles": the unit stuck to the figure is split off.
    let text_ = A_STUCK_UNIT.sub(r" \1", stripped);
    let named_thing = SUBJECT.is_match(&text_);
    // "my budget is about 800 pounds" sets a limit for the rest of the conversation.
    let a_budget = A_BUDGET_THEY_SET.is_match(&text_);
    let text_ = SUBJECT.subn("", &text_, 1);
    // "a tenner", "a fiver": the number is inside the noun.
    let (sum_word, text_) = _the_sum_inside_the_word(&text_);
    let words = text::split(&text_);
    let mut at = 0;
    while at < words.len() && _amount(&words[at]).is_none() {
        if !LEAD.contains(&text::lower(&words[at])) {
            return None;
        }
        at += 1;
    }
    if at >= words.len() {
        return None;
    }
    let head = words[at].clone();
    let head_low = text::lower(&head);
    let (amount, took) = _number_at(&words, at);
    let mut amount = amount.expect("a number at the head");
    let mut body: Vec<String> = words[at + took..].to_vec();
    // "half an hour", "half a mile": the half is the number and the hour is the unit.
    if head_low == "half" && body.first().is_some_and(|b| ["a", "an"].contains(&text::lower(b).as_str())) {
        body.remove(0);
    }
    let unit = match body.first().map(|b| text::lower(b)) {
        Some(u) if UNITS.contains_key(&u) => u,
        _ => String::new(),
    };
    if !unit.is_empty() {
        body.remove(0);
    }
    // "an hour and a half each way": the half is part of the number.
    let half_more = body.iter().take(3).map(|w| text::lower(w)).collect::<Vec<_>>() == ["and", "a", "half"];
    if half_more {
        amount *= 1.5;
        body.drain(..3);
    }
    let mut body = _without_the_household(&_without_a_remark(&body));
    let tail = text::lower(&body.join(" "));
    let mut period: Option<String> = None;
    for (said, name) in PERIODS.iter() {
        if tail == *said || tail.starts_with(said.as_str()) {
            period = Some(name.clone());
            let n = text::split_ref(said).len().min(body.len());
            body.drain(..n);
            break;
        }
    }
    // "its 45 a month, is that a lot": the judgement on the figure is the question about it.
    if period.is_some() && DOUBT_ON_THE_FIGURE.match_(text::strip_chars(text::strip(&body.join(" ")), " .,!?")).is_some() {
        body.clear();
    }
    for word in &body {
        let plain = text::strip_chars(&text::lower(word), ".,!?:").to_string();
        if TAIL.contains(&plain) || UNITS.contains_key(&plain) || _amount(word).is_some() {
            continue;
        }
        if _a_length_they_have_bounded(clean, &unit, &body, period.as_deref()) {
            // "i have 20 minutes before i have to leave": the rest says what the stretch fits inside.
            break;
        }
        return None;
    }
    let mut phrase = if digit_first(&head) || CURRENCY.iter().any(|c| head.starts_with(c.as_str())) { head.clone() } else { _fmt(amount) };
    if unit == "k" || unit == "grand" {
        phrase = format!("{phrase} {unit}");
    }
    let vowel_or_h = |u: &str| u.chars().next().is_some_and(|c| "aeiouh".contains(c));
    if head_low == "half" && !unit.is_empty() {
        phrase = format!("{} {unit}", if vowel_or_h(&unit) { "half an" } else { "half a" });
    }
    if (head_low == "a" || head_low == "an") && !unit.is_empty() {
        phrase = format!("{} {unit}", if vowel_or_h(&unit) { "an" } else { "a" });
    }
    if half_more {
        phrase = format!("{phrase} and a half");
    }
    if !unit.is_empty() {
        let (family, factor) = UNITS[&unit].clone();
        let body_tail = text::lower(&body.join(" "));
        if unit == "k" && period.is_none() && _about_a_distance(clean, memory) {
            // "a k" with a distance on the table is a kilometre, not a thousand pounds.
            let phrase = match phrase.strip_suffix(" k") {
                Some(p) => text::strip(p).to_string(),
                None => phrase,
            };
            return Some(Reading { value: amount * UNITS["km"].1, family: "distance".into(), unit: "km".into(), period: None, phrase, tail: body_tail });
        }
        if family == "money"
            && ["pound", "pounds", "quid", "euro", "euros", "dollar", "dollars"].contains(&unit.as_str())
            && TRIP.is_match(&format!("{body_tail} {tail}"))
        {
            return None;
        }
        if family == "money" && a_budget {
            return None;
        }
        if family == "span" && named_thing {
            // "my kitten is 4 months": a period said about a named thing is that thing's age.
            return None;
        }
        if family == "weight" && named_thing {
            // "the parcel is 2 kilos": the line says what weighs that.
            return None;
        }
        return Some(Reading { value: amount * factor, family, unit, period, phrase, tail: body_tail });
    }
    if _a_take_without_home(&words[..at]) && sum_word.is_empty() {
        // "i take 5 a day" is a count of tablets, not a wage.
        return None;
    }
    if a_budget && sum_word.is_empty() {
        return None;
    }
    let rest = text::lower(&body.join(" "));
    if let Some(period) = period {
        if sum_word.is_empty() && _a_count_the_question_asked_for(clean, memory) {
            return None;
        }
        let phrase = if sum_word.is_empty() { phrase } else { sum_word };
        return Some(Reading { value: amount, family: "money".into(), unit: "money".into(), period: Some(period), phrase, tail: rest });
    }
    // A number with no unit and no period is only money when it carries a currency mark.
    if !sum_word.is_empty() {
        return Some(Reading { value: amount, family: "money".into(), unit: "money".into(), period: None, phrase: sum_word, tail: rest });
    }
    if CURRENCY.iter().any(|c| head.starts_with(c.as_str())) {
        return Some(Reading { value: amount, family: "money".into(), unit: "money".into(), period: None, phrase, tail: String::new() });
    }
    None
}

/// Python: bot/compose/quantity.py:614 `_q(phrase, unit)`
/// The number with its unit, unless the unit is the period the sentence already carries.
pub fn _q(phrase: &str, unit: &str) -> String {
    if ["money", "k", "grand"].contains(&unit) || phrase.contains(unit) {
        return phrase.to_string();
    }
    format!("{phrase} {unit}")
}

/// Python: bot/compose/quantity.py:694 `_a_length_they_have_bounded(clean, unit, body, period)`
/// Whether a length the person has is being bounded or explained, not made into a subject.
pub fn _a_length_they_have_bounded(clean: &str, unit: &str, body: &[String], period: Option<&str>) -> bool {
    let family = UNITS.get(unit).map(|(f, _)| f.as_str());
    if family != Some("time") || period.is_some_and(|p| !p.is_empty()) {
        return false;
    }
    if !THEY_HAVE_THE_TIME.is_match(&flat(clean)) {
        return false;
    }
    let Some(first) = body.first() else { return false };
    BOUNDED_BY.contains(&text::strip_chars(&text::lower(first), ".,!?:").to_string())
}

/// Python: bot/compose/quantity.py:714 `_a_bound_not_a_budget(clean)`
/// Whether the length is a limit the person put on it rather than time they have to spend.
pub fn _a_bound_not_a_budget(clean: &str) -> bool {
    let said = flat(clean);
    A_BOUND.is_match(&said) && !THEY_HAVE_IT.is_match(&said)
}

/// Python: bot/compose/quantity.py:720 `_the_turn_above(memory)`
pub fn _the_turn_above(memory: &Memory) -> Option<&Turn> {
    memory.turns.last()
}

/// Python: bot/compose/quantity.py:739 `_the_change_is_the_line(clean)`
/// Whether this line itself says the figure is weight coming off or going on.
pub fn _the_change_is_the_line(clean: &str) -> bool {
    A_CHANGE_SAID.is_match(&flat(clean))
}

/// Python: bot/compose/quantity.py:744 `_a_weight_they_mean_themselves(memory, clean="")`
/// Whether this line, or a turn or two back, says the figure is about a body.
pub fn _a_weight_they_mean_themselves(memory: &Memory, clean: &str) -> bool {
    if I_WEIGH.is_match(&flat(clean)) {
        return true;
    }
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(2)..].iter().any(|turn| {
        [&turn.user, &turn.bot].iter().any(|said| {
            let said = flat(said);
            WEIGHT_ON_THE_TABLE.is_match(&said) || WEIGHT_GOING_ON.is_match(&said)
        })
    })
}

/// Python: bot/compose/quantity.py:764 `_a_thing_was_named(memory, clean="")`
/// Whether this line or the turn above named a thing this figure could be the weight of.
pub fn _a_thing_was_named(memory: &Memory, clean: &str) -> bool {
    if SOMETHING_WEIGHED.is_match(&flat(clean)) {
        return true;
    }
    let Some(above) = _the_turn_above(memory).map(|t| t.user.as_str()).filter(|u| !u.is_empty()) else { return false };
    A_LETTER_RUN.findall_str(&text::lower(above)).iter().any(|w| THINGS.contains(w))
}

/// Python: bot/compose/quantity.py:785 `_a_span_they_are_in(clean, memory)`
/// Whether the line, or the turn above it, says this stretch has already been lived.
pub fn _a_span_they_are_in(clean: &str, memory: &Memory) -> bool {
    let mut said = vec![clean.to_string()];
    if let Some(above) = _the_turn_above(memory).map(|t| t.user.clone()).filter(|u| !u.is_empty()) {
        said.push(above);
    }
    said.iter().any(|one| SPAN_FRAME.is_match(&flat(one)))
}

/// Python: bot/compose/quantity.py:804 `_a_count_the_question_asked_for(clean, memory)`
/// Whether the turn above asked "how many <thing>" and nothing on the table is money.
pub fn _a_count_the_question_asked_for(clean: &str, memory: &Memory) -> bool {
    let (user, bot) = memory.turns.last().map(|t| (t.user.as_str(), t.bot.as_str())).unwrap_or(("", ""));
    let said = [flat(user), flat(bot), flat(clean)];
    if !said[..2].iter().any(|one| A_COUNT_ASKED_FOR.is_match(one)) {
        return false;
    }
    !said.iter().any(|one| A_WAY_OF_PAYING.is_match(one))
}

/// Python: bot/compose/quantity.py:821 `_a_fact_about_a_length(memory)`
/// Whether the turn above asked how long something lasts, is, or takes.
pub fn _a_fact_about_a_length(memory: &Memory) -> bool {
    let above = flat(_the_turn_above(memory).map(|t| t.user.as_str()).unwrap_or(""));
    !above.is_empty() && A_QUESTION_ABOUT_A_LENGTH.is_match(&above) && !A_BUDGET_QUESTION.is_match(&above)
}

/// Python: bot/compose/quantity.py:828 `reply_for(clean, memory)`
/// A reply to a line that is nothing but a number, or None to leave it to the caller.
pub fn reply_for(clean: &str, memory: &Memory) -> Option<String> {
    let r = _read(clean, memory)?;
    let period = r.period.as_deref();
    if r.family == "span" && _the_turn_above(memory).is_some() && !_a_span_they_are_in(clean, memory) {
        return None;
    }
    if r.family == "time" && period.is_none() && r.value <= 180.0 && _a_fact_about_a_length(memory) {
        return None;
    }
    let q = _q(&r.phrase, &r.unit);
    if (r.family == "time" || r.family == "span") && period.is_none() && A_REMAINDER.is_match(&flat(clean)) {
        let left = py_format(&pick(&A_WHAT_IS_LEFT, memory, clean), &[("q", &q)]);
        return Some(first_upper(&left));
    }
    if r.family == "time" && period.is_none() && _a_bound_not_a_budget(clean) {
        // The length is a limit they set, not time they have: answer the limit itself.
        let limit = py_format(&pick(&A_LIMIT_ON_A_LENGTH, memory, clean), &[("q", &q)]);
        return Some(first_upper(&limit));
    }
    let reply = _say(&r.family, &q, r.value, period, &r.tail, memory, clean, &r.unit)?;
    // A sentence does not start in lower case.
    if reply.is_empty() { None } else { Some(first_upper(&reply)) }
}

/// Python: bot/compose/quantity.py:866 `the_thing_they_pay_for(clean)`
/// What the rate in the line is for, or None when the line only gives the number.
pub fn the_thing_they_pay_for(clean: &str) -> Option<String> {
    THEIR_THING.findall_str(&flat(clean)).into_iter().find(|w| !NOT_A_THING.contains(w))
}

/// Python: bot/compose/quantity.py:882 `the_rate_they_asked_about(clean, memory=None)`
/// The arithmetic on a rate the line itself carries and judges, or None.
pub fn the_rate_they_asked_about(clean: &str, memory: &Memory) -> Option<String> {
    let r = _read(clean, memory)?;
    let period = r.period.as_deref().filter(|p| !p.is_empty())?;
    if r.family != "money" {
        return None;
    }
    let said = flat(clean);
    if !PRICE_JUDGEMENT.is_match(&said) {
        return None;
    }
    let q = _q(&r.phrase, &r.unit);
    if SAVINGS_AFTER_A_RATE.is_match(&said) && ["month", "week", "year"].contains(&period) {
        let (times, said_period) = match period {
            "month" => (12.0, "a month"),
            "week" => (52.0, "a week"),
            _ => (1.0, "a year"),
        };
        let year = r.value * times;
        let reply = py_format(&pick(&SAVINGS, memory, clean), &[("q", &q), ("period", said_period), ("year", &_fmt(year))]);
        return Some(first_upper(&reply));
    }
    let reply = _say(&r.family, &q, r.value, Some(period), &r.tail, memory, clean, &r.unit)?;
    if reply.is_empty() { None } else { Some(first_upper(&reply)) }
}

/// Python: bot/compose/quantity.py:908 `_without_a_pointing_tail(words)`
/// The words with a trailing preposition and pronoun taken off ("a day for it").
pub fn _without_a_pointing_tail(words: &[String]) -> Vec<String> {
    if words.len() >= 2 {
        let n = words.len();
        if POINTING_BACK.contains(&plain_word(&words[n - 2])) && A_PRONOUN.contains(&plain_word(&words[n - 1])) {
            return words[..n - 2].to_vec();
        }
    }
    words.to_vec()
}

/// Python: bot/compose/quantity.py:921 `_without_a_hedge(words)`
/// The words with the speaker's own doubt taken off the end ("... a day, i think").
pub fn _without_a_hedge(words: &[String]) -> Vec<String> {
    if words.len() >= 2 {
        let n = words.len();
        if ["i", "id", "i'd"].contains(&plain_word(&words[n - 2]).as_str()) && HEDGES.contains(&plain_word(&words[n - 1])) {
            return words[..n - 2].to_vec();
        }
    }
    words.to_vec()
}

/// Python: bot/compose/quantity.py:945 `_without_the_household(words)`
/// The words with the household taken off, if that is what they are.
pub fn _without_the_household(words: &[String]) -> Vec<String> {
    let plain: Vec<String> = words.iter().map(|w| plain_word(w)).collect();
    for phrase in THE_HOUSEHOLD.iter() {
        let n = phrase.len();
        if plain.len() >= n && plain[..n] == phrase[..] {
            return words[n..].to_vec();
        }
        if plain.len() > n && plain[plain.len() - n..] == phrase[..] {
            return words[..words.len() - n].to_vec();
        }
    }
    words.to_vec()
}

/// Python: bot/compose/quantity.py:963 `_a_take_without_home(lead)`
/// Whether the line led with "take" and never said "home": a count, not a wage.
pub fn _a_take_without_home(lead: &[String]) -> bool {
    let said: Vec<String> = lead.iter().map(|w| plain_word(w)).collect();
    said.iter().any(|w| TAKE.contains(w)) && !said.iter().any(|w| w == "home")
}

/// Python: bot/compose/quantity.py:969 `_without_a_remark(words)`
/// A number with the person's own tail on it: "a day for it, i think".
pub fn _without_a_remark(words: &[String]) -> Vec<String> {
    let mut words = words.to_vec();
    for _ in 0..3 {
        let trimmed = _without_a_pointing_tail(&_without_a_hedge(&words));
        if trimmed == words {
            break;
        }
        words = trimmed;
    }
    words
}

/// Python: bot/compose/quantity.py:979 `_in_a_conversation(memory)`
/// Whether the bot has already been told what the number is about.
pub fn _in_a_conversation(memory: &Memory) -> bool {
    memory.turns.iter().any(|t| !t.user.is_empty())
}

/// Python: bot/compose/quantity.py:992 `_not_short(pool, value)`
/// The pool with the lines that tell the person the time is short taken out, if it is not.
pub fn _not_short(pool: &[String], value: f64) -> Vec<String> {
    if value < *A_STRETCH_THAT_IS_NOT_SHORT {
        return pool.to_vec();
    }
    let kept: Vec<String> = pool.iter().filter(|l| !l.contains("is tight") && !l.contains("faster than it looks")).cloned().collect();
    if kept.is_empty() { pool.to_vec() } else { kept }
}

/// Python: bot/compose/quantity.py:1001 `_without_a_question(pool)`
/// The lines that do not ask, or the pool when every line asks.
pub fn _without_a_question(pool: &[String]) -> Vec<String> {
    let kept: Vec<String> = pool.iter().filter(|l| !l.contains('?')).cloned().collect();
    if kept.is_empty() { pool.to_vec() } else { kept }
}

/// Python: bot/compose/quantity.py:1008 `_say(family, q, value, period, tail, memory, clean, unit=None)`
/// The reply for one family of number, or None.
pub fn _say(family: &str, q: &str, value: f64, period: Option<&str>, tail: &str, memory: &Memory, clean: &str, unit: &str) -> Option<String> {
    let say = |pool: &[String], fields: &[(&str, &str)]| -> Option<String> { Some(py_format(&pick(pool, memory, clean), fields)) };
    let period = period.unwrap_or("");
    match family {
        "money" => {
            if INCOME.is_match(&text::lower(clean)) {
                return match period {
                    "month" => say(&INCOME_MONTH, &[("q", q), ("year", &_fmt(value * 12.0))]),
                    "week" | "fortnight" => {
                        let weeks = if period == "week" { value } else { value / 2.0 };
                        say(&INCOME_WEEK, &[("q", q), ("month", &_fmt(round_int(weeks * 52.0 / 12.0) as f64)), ("year", &_fmt(weeks * 52.0))])
                    }
                    "year" => say(&INCOME_YEAR, &[("q", q), ("month", &_fmt(round_int(value / 12.0) as f64)), ("week", &_fmt(round_int(value / 52.0) as f64))]),
                    _ => say(&INCOME_PLAIN, &[("q", q)]),
                };
            }
            match period {
                "month" => say(&MONEY_MONTH, &[("q", q), ("year", &_fmt(value * 12.0))]),
                "week" => say(&MONEY_WEEK, &[("q", q), ("month", &_fmt(round_int(value * 52.0 / 12.0) as f64)), ("year", &_fmt(value * 52.0))]),
                "year" => say(&MONEY_YEAR, &[("q", q), ("month", &_fmt(round_int(value / 12.0) as f64)), ("week", &_fmt(round_int(value / 52.0) as f64))]),
                "day" | "fortnight" => {
                    let per_day = if period == "fortnight" { value / 14.0 } else { value };
                    say(&MONEY_DAY, &[("q", q), ("month", &_fmt(round_int(per_day * 30.0) as f64)), ("year", &_fmt(round_int(per_day * 365.0) as f64))])
                }
                _ => say(&MONEY_PLAIN, &[("q", q)]),
            }
        }
        "time" => {
            if period == "day" && !TRIP.is_match(tail) {
                // Every day is seven days: the week is the figure the reply quotes.
                return say(&TIME_A_DAY, &[("q", q), ("week", &_minutes(value * 7.0)), ("month", &_minutes(value * 30.0))]);
            }
            if TRIP.is_match(tail) {
                return say(&TIME_TRIP, &[("q", q), ("day", &_minutes(value * 2.0)), ("week", &_minutes(value * 10.0))]);
            }
            if period == "week" {
                return say(&TIME_WEEK, &[("q", q), ("day", &_minutes(value / 7.0))]);
            }
            if value > 180.0 {
                return say(&TIME_LONG, &[("q", q)]);
            }
            let pool = if !_in_a_conversation(memory) { TIME_BUDGET.clone() } else { _without_a_question(&TIME_BUDGET) };
            say(&_not_short(&pool, value), &[("q", q)])
        }
        "distance" => {
            if value <= 0.0 {
                return None;
            }
            let walk = round_int(value * 20.0).to_string(); // three miles an hour
            let bike = round_int(value * 5.0).to_string(); // twelve miles an hour
            let drive = round_int(if value > 10.0 { value * 1.33 } else { value * 2.4 }) as f64;
            if value <= 3.0 {
                return say(&DISTANCE_NEAR, &[("q", q), ("walk", &walk), ("bike", &bike)]);
            }
            if value <= 10.0 {
                return say(&DISTANCE_MID, &[("q", q), ("bike", &bike), ("drive", &_minutes(drive))]);
            }
            let pool = if TRIP.is_match(tail) {
                &*DISTANCE_FAR_TRIP
            } else if value > 60.0 {
                &*DISTANCE_DAY_OUT
            } else {
                &*DISTANCE_FAR
            };
            say(pool, &[("q", q), ("drive", &_minutes(drive))])
        }
        "size" => {
            // "2 metres by 3": the pair is an area, and an area is what paint is sold against.
            let other = BY_A_SECOND.search(tail)?;
            let second = _amount(other.str(1)).filter(|s| *s > 0.0)?;
            say(&AREA, &[("q", q), ("b", &_fmt(second)), ("area", &_fmt(value * second))])
        }
        "span" => {
            if value <= 3.0 {
                return say(&SPAN_DAYS, &[("q", q)]);
            }
            if value <= 21.0 {
                return say(&SPAN_WEEKS, &[("q", q)]);
            }
            if value <= 400.0 {
                return say(&SPAN_MONTHS, &[("q", q)]);
            }
            None
        }
        "weight" => {
            if value <= 0.2 {
                return None;
            }
            let kilos_value = round1(value);
            let kilos = format!("{} kilo{}", _fmt(kilos_value), if kilos_value == 1.0 { "" } else { "s" });
            let pounds = value / 0.4536;
            let said_kilos = ["kilo", "kilos", "kilogram", "kilograms", "kg", "kgs"].contains(&unit);
            let other = if said_kilos {
                let p = round1(pounds);
                format!("{} pound{}", _fmt(p), if p == 1.0 { "" } else { "s" })
            } else {
                kilos
            };
            let slow = round_int(pounds / 2.0).max(1).to_string();
            let fast_n = round_int(pounds).max(1);
            let fast = fast_n.to_string();
            // A pound or two a week is the pace worth naming only when it is more than a week or two.
            let body = _a_weight_they_mean_themselves(memory, clean);
            let pool = if body && fast_n > 2 && (value <= *A_CHANGE_IS_THIS_SMALL || _the_change_is_the_line(clean)) {
                &*WEIGHT_LOSS
            } else if I_WEIGH.is_match(&flat(clean)) {
                &*WEIGHT_THEIR_OWN
            } else if _a_thing_was_named(memory, clean) {
                &*WEIGHT_OF_A_THING
            } else {
                &*WEIGHT_PLAIN
            };
            say(pool, &[("q", q), ("other", &other), ("slow", &slow), ("fast", &fast)])
        }
        _ => None,
    }
}

/// Python: bot/compose/quantity.py:1223 `_the_unit_of_one(unit)`
/// The unit as it is written after a figure of one: "1 glass", not "1 glasses".
pub fn _the_unit_of_one(unit: &str) -> String {
    if let Some(stem) = unit.strip_suffix("sses") {
        return format!("{stem}ss"); // glasses -> glass, classes -> class
    }
    if unit.ends_with("ies") && text::len(unit) > 4 {
        return format!("{}y", &unit[..unit.len() - 3]);
    }
    if unit.ends_with('s') && !unit.ends_with("ss") && text::len(unit) > 2 {
        return unit[..unit.len() - 1].to_string();
    }
    unit.to_string()
}

/// The rule the answer named: (low, high, unit, the rule as said).
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub low: f64,
    pub high: f64,
    pub unit: String,
    pub said: String,
}

/// Python: bot/compose/quantity.py:1235 `_the_answer_range(above)`
/// (low, high, unit, said) for the rule with a unit in the answer, or None. The rule that comes
/// first in the answer is the one the figure is about.
pub fn _the_answer_range(above: &str) -> Option<Rule> {
    let mut rules: Vec<(usize, f64, f64, String, Option<String>)> = Vec::new();
    for found in A_RANGE_WITH_A_UNIT.finditer(above) {
        let low = _amount(&text::lower(found.name_str("lo")));
        let high = _amount(&text::lower(found.name_str("hi")));
        let (Some(mut low), Some(mut high)) = (low.filter(|v| *v != 0.0), high.filter(|v| *v != 0.0)) else { continue };
        if low <= 0.0 || high <= 0.0 {
            continue;
        }
        if high < low {
            std::mem::swap(&mut low, &mut high);
        }
        rules.push((found.start(), low, high, text::lower(found.name_str("unit")), found.name("period").map(str::to_string)));
    }
    for found in A_SINGLE_RULE.finditer(above) {
        let Some(value) = _amount(&text::lower(found.name_str("num"))).filter(|v| *v > 0.0) else { continue };
        rules.push((found.start(), value, value, text::lower(found.name_str("unit")), found.name("period").map(str::to_string)));
    }
    // `min(rules, key=start)`: the first of the earliest.
    let (_, low, high, unit, period) = rules.into_iter().reduce(|a, b| if b.0 < a.0 { b } else { a })?;
    let said = _the_rule_said(low, if low == high { None } else { Some(high) }, &unit, period.as_deref());
    Some(Rule { low, high, unit, said })
}

/// Python: bot/compose/quantity.py:1267 `_the_rule_said(low, high, unit, period)`
/// The rule as the reply says it back: "6 to 8 glasses a day", "150 minutes a week".
pub fn _the_rule_said(low: f64, high: Option<f64>, unit: &str, period: Option<&str>) -> String {
    let figure = match high {
        Some(high) => format!("{} to {}", _fmt(low), _fmt(high)),
        None => _fmt(low),
    };
    let unit_low = text::lower(unit);
    let said = format!("{figure} {}", UNIT_SAID.get(&unit_low).cloned().unwrap_or_else(|| unit_low.clone()));
    match PERIOD_NAMED.get(&text::lower(period.unwrap_or(""))).filter(|w| !w.is_empty()) {
        Some(when) => format!("{said} {when}"),
        None => said,
    }
}

/// Python: bot/compose/quantity.py:1275 `_the_family(unit)`
pub fn _the_family(unit: &str) -> String {
    UNIT_IS.iter().find(|(pattern, _)| pattern.is_match(unit)).map(|(_, f)| f.clone()).unwrap_or_default()
}

/// Python: bot/compose/quantity.py:1282 `the_figure_against_the_answer(clean, memory)`
/// The figure the person gives, read in the unit of the rule the answer just named, or None.
pub fn the_figure_against_the_answer(clean: &str, memory: &Memory) -> Option<String> {
    let line = flat(clean);
    if line.is_empty() || NOT_A_UNIT_ITSELF.is_match(&line) || AN_AGE.is_match(&line) {
        return None;
    }
    if !ABOUT_A_FIGURE.is_match(&line) {
        return None;
    }
    let alone = A_FIGURE_ALONE.match_(&line)?;
    let figure = _amount(alone.name_str("num")).filter(|f| *f > 0.0 && *f <= 500.0)?;
    if memory.understood.as_ref().is_some_and(|u| u.asks.iter().any(|a| a == "maths")) {
        return None;
    }
    let above = text::squash(&crate::memory::last_bot(memory));
    if above.is_empty() {
        return None;
    }
    let rule = _the_answer_range(&above)?;
    let family = _the_family(&rule.unit);
    let said_unit = UNIT_SAID.get(&rule.unit).cloned().unwrap_or_else(|| rule.unit.clone());
    let their_unit = text::lower(alone.name_str("what"));
    if !their_unit.is_empty() && (_the_family(&their_unit) != family || family.is_empty()) {
        return None; // they named something else, and another entry reads that
    }
    if figure * *TEN_TIMES < rule.low || figure > rule.high * *TEN_TIMES {
        // Not the answer's unit at all.
        return None;
    }
    let figure_said = _fmt(figure);
    let said_unit = if figure == 1.0 { _the_unit_of_one(&said_unit) } else { said_unit };
    let steps = STEPS.get(&family).unwrap_or(&A_STEP_EITHER_WAY);
    let step_of = |side: &str| steps.get(side).cloned().unwrap_or_else(|| panic!("KeyError: {side:?}"));
    let reply = if figure < rule.low {
        let step = step_of("below");
        py_format(&pick(&FIGURE_BELOW, memory, clean), &[("figure", &figure_said), ("unit", &said_unit), ("range", &rule.said), ("step", &step)])
    } else if figure > rule.high {
        let step = step_of("above");
        py_format(&pick(&FIGURE_ABOVE, memory, clean), &[("figure", &figure_said), ("unit", &said_unit), ("range", &rule.said), ("step", &step)])
    } else {
        py_format(&pick(&FIGURE_INSIDE, memory, clean), &[("figure", &figure_said), ("unit", &said_unit), ("range", &rule.said)])
    };
    Some(first_upper(&reply))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_said_the_python_way() {
        assert_eq!(_fmt(1234.0), "1,234");
        assert_eq!(_fmt(2.5), "2.5");
        assert_eq!(_fmt(2.25), "2.2");
        assert_eq!(_minutes(80.0), "1 hour 20");
        assert_eq!(_minutes(120.0), "2 hours");
        assert_eq!(_minutes(22.0), "20 minutes");
        assert_eq!(_the_unit_of_one("glasses"), "glass");
        assert_eq!(_the_unit_of_one("calories"), "calory");
    }
}
