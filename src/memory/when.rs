//! Port of `bot/memory/when.py`: a "when ..." follow-up, answered from what the person already
//! told us - the date of a thing they said, how long until it, whether it has happened, and how
//! long they have been at something.
//!
//! Module data (patterns, word lists, reply pools) comes from the exported bundle. Dates are
//! `chrono::NaiveDate`, read from `py::clock::today()`. Positions PyBot does arithmetic on (the
//! start of each date in a line) are kept as Python's code-point indices.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::facts::{a_date_put_right, a_moved_time, py_str, the_date_put_right};
use crate::memory::topics::{content, forms};
use crate::py::re::{self as pyre, Re};
use crate::py::text as pt;
use crate::state::{Memory, Turn};
use chrono::{Datelike, Days, NaiveDate};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

const I: u32 = pyre::I;

fn g(name: &str) -> &'static Value {
    data::get(&format!("bot.memory.when.{name}"))
}
fn re(name: &str) -> &'static Re {
    g(name).as_re().unwrap_or_else(|| panic!("bot.memory.when.{name} is not a regex"))
}

macro_rules! patterns {
    ($($name:ident),* $(,)?) => {
        $(pub static $name: LazyLock<&'static Re> = LazyLock::new(|| re(stringify!($name)));)*
    };
}
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(pub static $name: LazyLock<HashSet<String>> = LazyLock::new(|| g(stringify!($name)).str_set());)*
    };
}
patterns!(
    WHEN_ASK, BARE_CALENDAR_ASK, WHEN_SAID, WORD, CALENDAR_NOW, SHORT_DATE, BARE_DAY, PAST_WHEN,
    TRIP_LINE, TRIP_BOOKED, BOOKING, TRIP_BOOKED_ONLY_WITH_A_BOOKING, NOT_A_DATE_TO_GIVE,
    THEIR_OWN_DEATH, NOT_A_DATE_ASK, EVENT_WORD, WANT_LINE, PLAN_VERB_ASK, ASK_OPENER,
    THE_MEAN_NOTE, JOINED_TO_A_DATE, MONTH_AFTER_THE_DAY, A_THING_NAMED, ASKS_WHERE, ASKS_WHEN,
    ASKS_THE_MONTH, ASKS_THE_DAY, GOES_TO, PLACE_PREP, TRAILING_JOINER, SAYS_IT_IS_COMING,
    GOING_ON, WAS_GOING_ON, ALREADY_DONE_ASK, SAYS_IT_AGAIN, HOW_OFTEN, AN_ASK_IN_THE_PAST,
    AN_INTERVAL_AHEAD, UNTIL_A_LEARNING, UNTIL_ASK, AGO_ASK, SAID_IN_THE_PAST, NOT_IN_THE_PAST,
    UNTIL_COUNTDOWN, BARE_UNIT_ASK, A_LENGTH_IN_THE_LINE, A_COUNT_TAIL, DAY_NUMBER, _DAY_NAMED,
    _MOVED, _PLAN_VERB, _POINTS_AT_IT, HOW_LONG_ASK, NOT_A_LENGTH_ASK, AGO_LENGTH, LAST_LENGTH,
    FOR_LENGTH, WEEKDAY_SAID, A_YEAR_SAID, BEGAN, DAY_PUT_RIGHT, A_DAY_FORM,
);
word_sets!(
    ASK_THROWS, WHAT_DAY_SAYS, TRIP_WORDS, POINTS_BACK, NOT_A_THING, NO_PLACE, A_VERB,
    UNTIL_NAMES_THE_EVENT, A_TRIP_SAID, A_GOING, NOT_A_PLACE, UNTIL_THROWS, A_DATE_ITSELF, FILLER,
    ASKED_ABOUT_THE_THING,
);

static BACK_TURNS: LazyLock<usize> = LazyLock::new(|| g("BACK_TURNS").as_int().unwrap_or(3) as usize);
static NO_DATE_YET: LazyLock<Vec<String>> = LazyLock::new(|| g("NO_DATE_YET").strs());
static LEAD: LazyLock<Vec<String>> = LazyLock::new(|| g("LEAD").strs());
static NOT_A_PLURAL: LazyLock<Vec<String>> = LazyLock::new(|| g("NOT_A_PLURAL").strs());
static WEEKDAYS: LazyLock<Vec<String>> = LazyLock::new(|| g("WEEKDAYS").strs());
static EVENT_NAME: LazyLock<Vec<(String, Vec<String>)>> =
    LazyLock::new(|| g("EVENT_NAME").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect());
static SAME_THING: LazyLock<HashMap<String, HashSet<String>>> =
    LazyLock::new(|| g("SAME_THING").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_set())).collect());
fn str_int_map(name: &str) -> HashMap<String, i64> {
    g(name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0))).collect()
}
fn int_str_map(name: &str) -> HashMap<i64, String> {
    g(name).pairs().iter().map(|(k, v)| (k.as_int().unwrap_or(0), v.str().to_string())).collect()
}
static MONTH_NUMBER: LazyLock<HashMap<String, i64>> = LazyLock::new(|| str_int_map("MONTH_NUMBER"));
static MONTH_NAME: LazyLock<HashMap<i64, String>> = LazyLock::new(|| int_str_map("MONTH_NAME"));
static WEEKDAY_NUMBER: LazyLock<HashMap<String, i64>> = LazyLock::new(|| str_int_map("WEEKDAY_NUMBER"));
static WEEKDAY_NAME: LazyLock<HashMap<i64, String>> = LazyLock::new(|| int_str_map("WEEKDAY_NAME"));
static NUMBER_WORDS: LazyLock<HashMap<String, i64>> = LazyLock::new(|| str_int_map("NUMBER_WORDS"));
static A_UNIT_OF_TIME: LazyLock<HashMap<String, i64>> = LazyLock::new(|| str_int_map("A_UNIT_OF_TIME"));
static SPELLED_COUNT: LazyLock<HashMap<i64, String>> = LazyLock::new(|| int_str_map("SPELLED_COUNT"));

// `bot.memory.facts`.
static FUTURE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.facts.FUTURE"));
static MONTHS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.MONTHS"));
static DAYS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.DAYS"));
static DETERMINERS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.DETERMINERS"));

// The patterns PyBot writes inline.
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static A_DAY_OF_THE_MONTH: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b\d{1,2}(?:st|nd|rd|th)\b", 0));
static A_RUN_OF_LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[A-Za-z]+", 0));
static SPACE_OR_COMMA: LazyLock<Re> = LazyLock::new(|| Re::new(r"[\s,]", 0));
static ENDS_ON_A_DAY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d{1,2}(?:st|nd|rd|th)$", 0));
static A_DAY_SAID_ALONE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d{1,2}(?:st|nd|rd|th)?", 0));
static TWO_DIGITS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d{1,2}", 0));
static A_LETTER: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]", I));
static NOT_A_PLACE_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:is|are|was|were|am|and|with|for|about)\b", 0));
static A_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d", 0));
static AGO_OR_SINCE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:ago|since)\b", I));
static DAYS_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bdays?\b", 0));
static WEEKS_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bweeks?\b", 0));
static MONTHS_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bmonths?\b", 0));
static AHEAD_OF_TIME_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:on|till|until)\s+ahead of time\b", 0));
static NOT_ALNUM: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9]+", 0));
static FOUR_DIGITS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d{4}", 0));
static NOT_A_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\D", 0));

/// One date in a line: the date as it is said back, and where it starts (a code-point index).
pub type Span = (String, usize);
/// A line that carries dates, with them.
pub type DatedLine = (String, Vec<Span>);

/// `" ".join(text.split())`.
fn sq(text: &str) -> String {
    pt::squash(text)
}

/// A Python index (code points) for a byte offset into `text`.
fn ci(text: &str, byte: usize) -> usize {
    pt::char_index(text, byte)
}

/// The value of one Unicode decimal digit (`int()` reads every Nd digit).
fn decimal_value(c: char) -> Option<u32> {
    if c.is_ascii_digit() {
        return c.to_digit(10);
    }
    if !crate::understanding::pyuni::ch_decimal(c) {
        return None;
    }
    let mut back = 0u32;
    let mut cp = c as u32;
    while cp > 0 {
        match char::from_u32(cp - 1) {
            Some(p) if crate::understanding::pyuni::ch_decimal(p) => {
                back += 1;
                cp -= 1;
            }
            _ => break,
        }
    }
    Some(back % 10)
}

/// `int(s)` for a string of digits (what `\d` matched).
fn int_of_digits(s: &str) -> i64 {
    s.chars().fold(0i64, |n, c| n.saturating_mul(10).saturating_add(decimal_value(c).unwrap_or(0) as i64))
}

/// `str.isdigit()` for the strings `int()` can read.
fn all_digits(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| decimal_value(c).is_some())
}

/// `s[:1].upper() + s[1:]`.
fn up1(s: &str) -> String {
    pt::upper_first(s)
}

/// `s[:1].lower() + s[1:]`.
fn low1(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().chain(c).collect(),
    }
}

/// `round(x)` as Python does it (half to even), as an int.
fn py_round(x: f64) -> i64 {
    pt::round_half_even(x)
}

/// The user's facts as `(memory.get("user") or {})`: None for "no facts"; a value that is not a
/// dict but is truthy has no `.get`, which is an AttributeError in Python.
fn user_facts(memory: &Memory) -> Option<&Value> {
    match memory.get("user") {
        Some(user) if user.truthy() => {
            if !user.is_dict() {
                panic!("AttributeError: 'user' has no attribute 'get'");
            }
            Some(user)
        }
        _ => None,
    }
}

/// The plans the store keeps as strings: `[p for p in (known.get("plans") or []) if
/// isinstance(p, str)]`.
fn plans_of(known: Option<&Value>) -> Vec<String> {
    let Some(plans) = known.and_then(|k| k.get("plans")).filter(|v| v.truthy()) else {
        return Vec::new();
    };
    match plans {
        Value::Dict(d) => d.iter().filter_map(|(k, _)| k.as_str().map(str::to_string)).collect(),
        Value::Str(s) => s.chars().map(|c| c.to_string()).collect(),
        other => other.items().iter().filter_map(|p| p.as_str().map(str::to_string)).collect(),
    }
}

/// The lines the person typed, squashed, oldest first.
fn typed_lines(memory: &Memory) -> Vec<String> {
    memory.turns.iter().map(|turn| sq(&turn.user)).collect()
}

/// Python: bot/memory/when.py:171 `_is_a_trip_line(said)`
///
/// True when this line is about a journey, whether said by its verb or by what was booked.
pub fn _is_a_trip_line(said: &str) -> bool {
    if TRIP_LINE.is_match(said) || TRIP_BOOKED.is_match(said) {
        return true;
    }
    TRIP_BOOKED_ONLY_WITH_A_BOOKING.is_match(said) && BOOKING.is_match(said)
}

/// Python: bot/memory/when.py:200 `_names_that_event(word, other)`
///
/// True when the line states, with `other`, the event the ask named with `word`.
pub fn _names_that_event(word: &str, other: &str) -> bool {
    EVENT_NAME.iter().any(|(name, says)| _same(word, name) && says.iter().any(|one| _same(one, other)))
}

/// Python: bot/memory/when.py:275 `asks_what_today_is(text)`
///
/// "What day is it", "what's the date", "what date is it today": the calendar now.
pub fn asks_what_today_is(text: &str) -> bool {
    let line = sq(text);
    CALENDAR_NOW.match_(pt::rstrip_chars(&line, "?!. ")).is_some()
}

/// Python: bot/memory/when.py:288 `_more_than_one_day(memory)`
///
/// True when the chat holds two different days, and neither of them is a day put right.
pub fn _more_than_one_day(memory: &Memory) -> bool {
    let dated = _dated(memory);
    if dated.iter().any(|(said, _)| a_date_put_right(said)) {
        return false;
    }
    let days: HashSet<&str> = dated.iter().filter_map(|(_, spans)| spans.last().map(|s| s.0.as_str())).collect();
    days.len() > 1
}

/// `spans[-1][0]`.
fn last_date(spans: &[Span]) -> String {
    match spans.last() {
        Some(span) => span.0.clone(),
        None => panic!("IndexError: list index out of range"),
    }
}

/// Python: bot/memory/when.py:301 `when_answer(text, memory)`
///
/// The date of the thing the line asks about, or None when no earlier line says it.
pub fn when_answer(text: &str, memory: &Memory) -> Option<String> {
    let line = text;
    if CALENDAR_NOW.match_(line).is_some() {
        return None; // "what day is it" asks about today, and today is not one of their lines
    }
    if !WHEN_ASK.is_match(line) || WHEN_SAID.is_match(line) {
        return None;
    }
    if BARE_CALENDAR_ASK.match_(&sq(line)).is_some() {
        return None; // "what day is it" asks the calendar
    }
    let words: Vec<String> =
        content(line).into_iter().filter(|word| !ASK_THROWS.contains(word) && !WHAT_DAY_SAYS.contains(word)).collect();
    if NOT_A_DATE_TO_GIVE.is_match(line) {
        return None; // a date asked for over a death is not the thing they want said back
    }
    if !words.is_empty() && _points_back(line) {
        // "when is the best time to prep for it" names the asking and nothing of the thing, so
        // no word has to match: the newest dated line is the thing.
        let dated = _dated(memory);
        if let Some(first) = dated.first() {
            if a_date_put_right(&first.0) && _said_in_the_last_few_turns(memory, &first.0) {
                // A day put right under a line that gave the month is both halves of one date.
                let joined = if dated.len() > 1 { _a_day_and_a_month(&last_date(&first.1), &last_date(&dated[1].1)) } else { String::new() };
                if !joined.is_empty() {
                    return Some(_sentence(&joined));
                }
            }
        }
        if let Some((said, spans)) = dated.first() {
            if _said_in_the_last_few_turns(memory, said) {
                return Some(_sentence(&last_date(spans)));
            }
        }
        return None;
    }
    let mut asked: Vec<String> = words.iter().filter(|word| !TRIP_WORDS.contains(*word)).cloned().collect();
    let mut trip = asked != words;
    if asked.is_empty() {
        // Every word in the line is the asking; a word the ask shares with a dated line in the
        // chat is that plan, so it is kept as the binding when there is one.
        let said_before: HashSet<String> = _dated(memory).iter().flat_map(|(said, _)| content(said)).collect();
        let anchors: Vec<String> = content(line).into_iter().filter(|word| said_before.contains(word)).collect();
        if !anchors.is_empty() {
            asked = anchors;
            trip = false;
        } else if _more_than_one_day(memory) {
            // Nothing to bind the ask to a line and two different days in the chat: the newest
            // one is a guess, and no binding means no date given.
            return None;
        }
    }
    // A date put right sits directly above the line it corrects, so it is held for that line only.
    let mut put_right = String::new();
    for (said, spans) in _dated(memory) {
        if NOT_A_DATE_TO_GIVE.is_match(&said) {
            continue; // a date in the line about a death is not an answer to give back
        }
        // The put-right line is read before the trip filter, because it never says the thing it
        // is about. Only the newest of a run of corrections is kept.
        if a_date_put_right(&said) {
            if put_right.is_empty() {
                put_right = last_date(&spans);
            }
            continue;
        }
        if trip && !_is_a_trip_line(&said) {
            continue;
        }
        if let Some(when) = _time_for(&asked, &said, &spans).filter(|when| !when.is_empty()) {
            let joined = _a_day_and_a_month(&put_right, &when);
            let said_date = if !joined.is_empty() {
                joined
            } else if !put_right.is_empty() {
                put_right.clone()
            } else {
                when
            };
            // A date that has happened is said back as one.
            if _has_happened(&said) && !AN_ASK_IN_THE_PAST.is_match(line) {
                return Some(_was_sentence(&said_date));
            }
            return Some(_sentence(&said_date));
        }
        // A date said with no subject of its own is about the thing named a line or two above.
        if !asked.is_empty() && !_names_a_thing(&said, &spans) {
            let above = _the_thing_before_the_date(memory, &said, &asked);
            if !above.is_empty() {
                let last = last_date(&spans);
                let joined = _a_day_and_a_month(&put_right, &last);
                let date = if !joined.is_empty() {
                    joined
                } else if !put_right.is_empty() {
                    put_right.clone()
                } else {
                    last
                };
                return Some(_sentence(&date));
            }
        }
        put_right = String::new();
    }
    _no_date_yet(&asked, line, memory)
}

/// Python: bot/memory/when.py:395 `_is_an_ask(line)`
///
/// True when the line is a question, not a statement of what they are doing.
pub fn _is_an_ask(line: &str) -> bool {
    let text = pt::strip(line);
    if text.is_empty() {
        return false;
    }
    text.ends_with('?') || ASK_OPENER.is_match(text)
}

/// Python: bot/memory/when.py:403 `_backs_off(line)`
///
/// True when the ask ends on a word that points at something already said, once the words
/// that ask where in the line the answer is ("due", "exactly", "again") are taken off.
pub fn _backs_off(line: &str) -> bool {
    let mut tokens = LETTERS.findall_str(&pt::lower(line));
    while tokens.last().is_some_and(|last| ASK_THROWS.contains(last)) {
        tokens.pop();
    }
    tokens.last().is_some_and(|last| POINTS_BACK.contains(last))
}

/// Python: bot/memory/when.py:415 `_no_date_yet(asked, line, memory)`
///
/// The reply for a thing they have told us about but never dated, or None.
pub fn _no_date_yet(asked: &[String], line: &str, memory: &Memory) -> Option<String> {
    if NOT_A_DATE_ASK.is_match(line) {
        return None;
    }
    let turns = typed_lines(memory);
    let known = user_facts(memory);
    let mut plans = plans_of(known);
    if let Some(Value::Str(latest)) = known.and_then(|k| k.get("plan")) {
        plans.push(latest.to_string());
    }
    let said_lines: Vec<&String> = turns.iter().filter(|said| !said.is_empty()).collect();
    let recent = &said_lines[said_lines.len().saturating_sub(*BACK_TURNS)..];
    let lines: Vec<String> = recent.iter().map(|s| s.to_string()).chain(plans.iter().rev().cloned()).collect();
    for said in &lines {
        if said.is_empty() || NOT_A_DATE_TO_GIVE.is_match(said) {
            continue;
        }
        // A question they asked is not a plan they told us.
        if _is_an_ask(said) {
            continue;
        }
        if FUTURE.is_match(said) || SHORT_DATE.is_match(said) {
            return None; // that line is dated; the date was not read, so leave the turn alone
        }
        if !asked.is_empty() {
            let theirs: Vec<String> = content(said).into_iter().filter(|word| !ASK_THROWS.contains(word)).collect();
            if !asked.iter().any(|word| theirs.iter().any(|other| _same(word, other))) {
                continue;
            }
        } else {
            // The verb on its own ("when do i start") names nothing that can be matched, so the
            // line has to be an event they put in their diary.
            if !(_backs_off(line) || PLAN_VERB_ASK.is_match(line)) {
                return None;
            }
            if !EVENT_WORD.is_match(said) || WANT_LINE.is_match(said) {
                continue;
            }
        }
        return Some(NO_DATE_YET[memory.turns.len() % NO_DATE_YET.len()].clone());
    }
    None
}

/// Python: bot/memory/when.py:457 `_a_day_and_a_month(day, month)`
///
/// The two halves of one date joined: "on the 4th" put right under "in june" is "on the 4th of
/// june". "" unless one half is a day and the other a month.
pub fn _a_day_and_a_month(day: &str, month: &str) -> String {
    let Some(the_day) = A_DAY_OF_THE_MONTH.search(day) else {
        return String::new();
    };
    let a_month = A_RUN_OF_LETTERS.findall_str(month).into_iter().find(|word| MONTHS.contains(&pt::lower(word)));
    match a_month {
        Some(a_month) => format!("on the {} of {}", the_day.as_str(), pt::lower(&a_month)),
        None => String::new(),
    }
}

/// Python: bot/memory/when.py:473 `_points_back(line)`
///
/// True when the ask ends on a word that points at something already said.
pub fn _points_back(line: &str) -> bool {
    LETTERS.findall_str(&pt::lower(line)).last().is_some_and(|last| POINTS_BACK.contains(last))
}

/// Python: bot/memory/when.py:483 `the_plan(memory)`
///
/// The newest plan the store keeps, in the person's own words, or None.
pub fn the_plan(memory: &Memory) -> Option<String> {
    let known = match memory.get("user") {
        Some(user) if user.truthy() => user,
        _ => return None, // `{}` holds no plan
    };
    if !known.is_dict() {
        return None;
    }
    if let Some(Value::Str(plan)) = known.get("plan") {
        if !pt::strip(plan).is_empty() {
            return Some(sq(plan));
        }
    }
    let plans: Vec<String> = plans_of(Some(known)).into_iter().filter(|p| !pt::strip(p).is_empty()).collect();
    plans.last().map(|p| sq(p))
}

/// Python: bot/memory/when.py:501 `_said_in_the_last_few_turns(memory, said)`
///
/// True when that line is still one of the last few things the person typed.
pub fn _said_in_the_last_few_turns(memory: &Memory, said: &str) -> bool {
    let turns = typed_lines(memory);
    let key = sq(said);
    turns[turns.len().saturating_sub(*BACK_TURNS)..].contains(&key)
}

/// Python: bot/memory/when.py:509 `_names_a_thing(text, spans)`
///
/// True when the line names something other than the date it carries.
pub fn _names_a_thing(text: &str, spans: &[Span]) -> bool {
    let mut said = text.to_string();
    let mut order: Vec<&Span> = spans.iter().collect();
    order.sort_by_key(|span| std::cmp::Reverse(span.1)); // stable, as `sorted(key=-start)`
    for (when, _start) in order {
        said = said.replacen(when.as_str(), " ", 1);
    }
    content(&said).iter().any(|word| !NOT_A_THING.contains(word))
}

/// Python: bot/memory/when.py:533 `_the_thing_before_the_date(memory, said, asked)`
///
/// The earlier line the ask is about, when the date was said with no subject of its own, or "".
pub fn _the_thing_before_the_date(memory: &Memory, said: &str, asked: &[String]) -> String {
    let turns = typed_lines(memory);
    let key = sq(said);
    let cut = match turns.iter().rposition(|line| *line == key) {
        Some(at) => at,
        None => turns.len(),
    };
    let from = cut.saturating_sub(*BACK_TURNS * 2);
    for line in turns[from..cut].iter().rev() {
        if line.is_empty() || *line == key {
            continue;
        }
        let spans: Vec<String> = FUTURE
            .finditer(line)
            .iter()
            .map(|found| found.as_str().to_string())
            .chain(SHORT_DATE.finditer(line).iter().map(|found| found.as_str().to_string()))
            .collect();
        if a_date_put_right(line) {
            continue;
        }
        let as_spans: Vec<Span> = spans.iter().map(|span| (span.clone(), 0)).collect();
        if !_names_a_thing(line, &as_spans) {
            continue;
        }
        if !spans.is_empty() {
            // A dated thing of its own in between: the bridge only carries a date over the lines
            // that name nothing.
            break;
        }
        if asked.iter().any(|word| WORD.findall_str(line).iter().any(|found| _same(word, found))) {
            return line.clone();
        }
    }
    String::new()
}

/// Python: bot/memory/when.py:580 `_the_line_put_right(memory)`
///
/// The topic's line with the value they meant in place of the date they took back, and the date
/// it replaced; ("", "") when there is none.
pub fn _the_line_put_right(memory: &Memory) -> (String, String) {
    let nothing = (String::new(), String::new());
    let text = match memory.get("topic") {
        Some(record) if record.is_dict() => sq(&record.get("text").filter(|v| v.truthy()).map(py_str).unwrap_or_default()),
        _ => String::new(),
    };
    let Some(note) = THE_MEAN_NOTE.search(&text) else {
        return nothing;
    };
    let meant = pt::strip_chars(&sq(note.str(1)), " .,").to_string();
    let line = pt::strip_chars(&text[..note.start()], " ,;").to_string();
    if line.is_empty() || meant.is_empty() {
        return nothing;
    }
    if !a_moved_time(&meant) {
        // A value put right that is no kind of time says nothing about a date.
        return nothing;
    }
    let mut spans = FUTURE.finditer(&line);
    spans.extend(SHORT_DATE.finditer(&line));
    if spans.is_empty() {
        return nothing;
    }
    // The last date on the line is the one the value they meant goes in place of; of two that
    // start together, the longer one (the first of the longest, as `max` keeps).
    let mut last = &spans[0];
    for found in &spans[1..] {
        if (found.start(), found.end()) > (last.start(), last.end()) {
            last = found;
        }
    }
    let joined = JOINED_TO_A_DATE.match_(last.as_str()).map(|m| m.as_str().to_string()).unwrap_or_default();
    let swapped = format!("{}{}{}{}", &line[..last.start()], joined, meant, &line[last.end()..]);
    // The date they took back is the date itself, with no joining word on it.
    let old = JOINED_TO_A_DATE.subn("", last.as_str(), 1);
    (sq(&swapped), _plain(&old))
}

/// Python: bot/memory/when.py:618 `_dated(memory)`
///
/// Every earlier line that carries a date, with its dates: newest first.
pub fn _dated(memory: &Memory) -> Vec<DatedLine> {
    let mut out: Vec<DatedLine> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let known = user_facts(memory);
    let mut plans = plans_of(known);
    if let Some(Value::Str(latest)) = known.and_then(|k| k.get("plan")) {
        if !plans.iter().any(|p| **p == **latest) {
            plans.push(latest.to_string());
        }
    }
    let typed: Vec<String> = memory.turns.iter().rev().map(|turn| turn.user.clone()).collect();
    let mut lines = typed.clone();
    // A date the person put right is the date: the corrected line stands where the newest one
    // does, and the date it replaced is dropped from the lines that still say it.
    let (mut fixed_sentence, mut replaced) = _put_right(memory);
    if fixed_sentence.is_empty() {
        (fixed_sentence, replaced) = _the_line_put_right(memory);
    }
    if !fixed_sentence.is_empty() {
        lines.insert(0, fixed_sentence);
    }
    let theirs: HashSet<String> = typed.iter().map(|line| pt::lower(&sq(line))).collect();
    for said in lines.iter().chain(plans.iter().rev()) {
        let said = sq(said);
        let key = pt::lower(&said);
        if said.is_empty() || seen.contains(&key) {
            continue;
        }
        let mut spans: Vec<Span> = Vec::new();
        let mut places: HashSet<usize> = HashSet::new();
        for found in FUTURE.finditer(&said) {
            let at = ci(&said, found.start());
            spans.push((_lead(&_with_the_month_after(&said, found.as_str(), found.end())), at));
            places.insert(at);
        }
        for pattern in [&*SHORT_DATE, &*PAST_WHEN] {
            for found in pattern.finditer(&said) {
                let at = ci(&said, found.start());
                if places.contains(&at) {
                    continue;
                }
                spans.push((_lead(found.as_str()), at));
                places.insert(at);
            }
        }
        for found in BARE_DAY.finditer(&said) {
            // The day is the date, said back with the "on" a date takes; a day the "on" reader
            // already has is the same date and is not counted twice.
            let (start, _) = found.span(1).unwrap_or((found.start(), found.start()));
            let at = ci(&said, start);
            if places.contains(&at) {
                continue;
            }
            spans.push((_lead(&format!("on {}", found.str(1))), at));
            places.insert(at);
        }
        if spans.is_empty() && a_date_put_right(&said) {
            // "sorry, it's the 4th" carries a date and nothing else: it is the date put right.
            spans.push((_a_date_said(&the_date_put_right(&said)), 0));
        }
        spans.sort_by_key(|span| span.1);
        // Only a line they typed loses the date they took back.
        if !replaced.is_empty() && theirs.contains(&key) {
            spans.retain(|span| !pt::split(&_plain(&span.0)).contains(&replaced));
        }
        if spans.is_empty() {
            continue;
        }
        seen.insert(key);
        out.push((said, spans));
    }
    out
}

/// Python: bot/memory/when.py:696 `_lead(found)`
///
/// A date given with the word that joins it to the thing it belongs to, said back with the
/// preposition a date takes: "is tuesday" is "on tuesday".
pub fn _lead(found: &str) -> String {
    let text = pt::strip_chars(&sq(found), " ,;.").to_string();
    let words = pt::split(&text);
    if words.len() > 1 && LEAD.contains(&pt::lower(&words[0])) {
        let rest = words[1..].join(" ");
        if pt::starts_with_any(&pt::lower(&rest), &["in ", "on ", "at ", "by ", "next ", "this ", "last "]) {
            return rest;
        }
        let first = pt::lower(&SPACE_OR_COMMA.split(&rest)[0]);
        if MONTHS.contains(&first) || matches!(first.as_str(), "early" | "mid" | "late") {
            return format!("in {rest}");
        }
        return format!("on {rest}");
    }
    text
}

/// Python: bot/memory/when.py:717 `_time_for(asked, said, spans)`
///
/// The date in that line that belongs to the thing asked about, or None.
pub fn _time_for(asked: &[String], said: &str, spans: &[Span]) -> Option<String> {
    if asked.is_empty() {
        // "when is it?" names nothing of its own, so it means the last date in the line.
        return Some(last_date(spans));
    }
    let mut hits: Vec<i64> = Vec::new();
    let mut seen: HashSet<&str> = HashSet::new();
    for found in WORD.finditer(said) {
        for word in asked {
            if !seen.contains(word.as_str()) && (_same(word, found.as_str()) || _names_that_event(word, found.as_str())) {
                seen.insert(word.as_str());
                hits.push(ci(said, found.start()) as i64);
            }
        }
    }
    // The line has to be about what the ask names, not just share one word with it.
    if seen.len() < asked.len() && (asked.len() < 3 || seen.len() < asked.len() - 1) {
        return None;
    }
    let mut best: Option<((i64, i64), &String)> = None;
    for (when, start) in spans {
        let start = *start as i64;
        // The date that follows the thing it belongs to, and the nearest one.
        let after: Vec<i64> = hits.iter().filter(|hit| start >= **hit).map(|hit| start - hit).collect();
        let score = if let Some(min) = after.iter().min() {
            (*min, 0)
        } else {
            match hits.iter().map(|hit| (start - hit).abs()).min() {
                Some(min) => (min, 1),
                None => panic!("ValueError: min() arg is an empty sequence"),
            }
        };
        if best.as_ref().map_or(true, |(held, _)| score < *held) {
            best = Some((score, when));
        }
    }
    match best {
        Some((_, when)) => Some(when.clone()),
        None => panic!("TypeError: 'NoneType' object is not subscriptable"),
    }
}

/// Python: bot/memory/when.py:758 `_with_the_month_after(said, found)`
///
/// The date text, plus the month said right after a bare day number. (`found` is the match:
/// its text and where it ends in `said`, a byte offset.)
pub fn _with_the_month_after(said: &str, found: &str, found_end: usize) -> String {
    if !ENDS_ON_A_DAY.is_match(pt::strip(found)) {
        return found.to_string();
    }
    match MONTH_AFTER_THE_DAY.match_(&said[found_end..]) {
        Some(after) => format!("{found}{}", after.as_str()),
        None => found.to_string(),
    }
}

/// Python: bot/memory/when.py:769 `_names_a_thing_of_its_own(line, said)`
///
/// Whether a countdown ask names a thing the dated line is not about.
pub fn _names_a_thing_of_its_own(line: &str, said: &str) -> bool {
    let Some(found) = UNTIL_COUNTDOWN.search(line) else {
        return false;
    };
    let body = &line[found.end()..];
    if _POINTS_AT_IT.is_match(body) {
        return false; // "till it goes off": the thing is the one just said
    }
    let dated: HashSet<String> = WORD.findall_str(&pt::lower(said)).into_iter().collect();
    if UNTIL_NAMES_THE_EVENT.contains(&pt::lower(found.str(1))) {
        let words = WORD.findall_str(&pt::lower(body));
        for (index, word) in words.iter().enumerate() {
            if UNTIL_THROWS.contains(word) || ASK_THROWS.contains(word) {
                continue;
            }
            if dated.contains(word) || dated.iter().any(|held| _same(word, held)) {
                continue;
            }
            let behind = if index > 0 { words[index - 1].as_str() } else { "" };
            if DETERMINERS.contains(behind) && !_a_doing_word(&words[index + 1..]) {
                // A definite thing with no verb of its own is the event the chat already holds.
                continue;
            }
            return true; // "till christmas", "until the boiler is fixed"
        }
    }
    for found in A_THING_NAMED.finditer(body) {
        let thing = pt::lower(found.str(1));
        if dated.contains(&thing) || dated.iter().any(|word| _same(&thing, word)) {
            continue;
        }
        if UNTIL_THROWS.contains(&thing) {
            continue;
        }
        return true;
    }
    false
}

/// Python: bot/memory/when.py:816 `_a_doing_word(words)`
///
/// Whether any of the words says what is done, rather than naming the thing.
pub fn _a_doing_word(words: &[String]) -> bool {
    words
        .iter()
        .filter(|word| !matches!(word.as_str(), "is" | "are" | "was" | "were" | "be" | "been"))
        .any(|word| word.ends_with("ed") || word.ends_with("ing") || (pt::len(word) > 3 && word.ends_with('s')))
}

/// Python: bot/memory/when.py:827 `_plain(when)`
///
/// The date with the word that joins it stripped: "in july" -> "july".
pub fn _plain(when: &str) -> String {
    let words = pt::split(&pt::lower(when));
    let start = words.iter().position(|word| !LEAD.contains(word)).unwrap_or(words.len());
    words[start..].join(" ")
}

/// Python: bot/memory/when.py:835 `_put_right(memory)`
///
/// The date a correction settled on (the sentence it was put right in), and the old one it
/// replaced; ("", "") when nothing dated was corrected.
pub fn _put_right(memory: &Memory) -> (String, String) {
    let nothing = (String::new(), String::new());
    let Some(fixed) = memory.get("correction").filter(|fixed| fixed.is_dict()) else {
        return nothing;
    };
    let text_of = |key: &str| fixed.get(key).filter(|v| v.truthy()).map(py_str).unwrap_or_default();
    let sentence = sq(&text_of("sentence"));
    let old = _plain(&text_of("from"));
    if sentence.is_empty() || old.is_empty() {
        return nothing;
    }
    if !FUTURE.is_match(&sentence) && !SHORT_DATE.is_match(&sentence) {
        return nothing;
    }
    (sentence, old)
}

/// Python: bot/memory/when.py:869 `_a_date_said(value)`
///
/// A bare date said back the way a date is said: "june" -> "in june", "the 4th" -> "on the 4th",
/// "4th may" -> "on the 4th of may".
pub fn _a_date_said(value: &str) -> String {
    let phrase = pt::strip_chars(&sq(value), " .,!?").to_string();
    let words = pt::split(&pt::lower(&phrase));
    if words.is_empty() {
        return String::new();
    }
    if words.len() >= 2
        && A_DAY_SAID_ALONE.fullmatch(&words[0]).is_some()
        && (MONTHS.contains(&words[1]) || (words.len() > 2 && words[1] == "of"))
    {
        let rest = if MONTHS.contains(&words[1]) { words[1..].join(" ") } else { words[2..].join(" ") };
        return format!("on the {} of {rest}", words[0]);
    }
    if MONTHS.contains(&words[0]) || matches!(words[0].as_str(), "early" | "mid" | "late") {
        return format!("in {phrase}");
    }
    format!("on {phrase}")
}

/// Python: bot/memory/when.py:890 `_same(word, other)`
///
/// True when the two words are the same word, with or without a verb ending on it.
pub fn _same(word: &str, other: &str) -> bool {
    let (word, other) = (pt::lower(word), pt::lower(other));
    if word == other || !forms(&word).is_disjoint(&forms(&other)) {
        return true;
    }
    if _root(&word) == _root(&other) {
        return true;
    }
    if _a_plural_stem(&word) == other || word == _a_plural_stem(&other) {
        return true;
    }
    SAME_THING.get(&word).is_some_and(|same| same.contains(&other))
        || SAME_THING.get(&other).is_some_and(|same| same.contains(&word))
}

/// Python: bot/memory/when.py:902 `_root(word)`
pub fn _root(word: &str) -> String {
    for ending in ["ing", "ed"] {
        if word.ends_with(ending) && pt::len(word) as i64 - ending.len() as i64 >= 2 {
            return word[..word.len() - ending.len()].to_string();
        }
    }
    word.to_string()
}

/// Python: bot/memory/when.py:916 `_a_plural_stem(word)`
///
/// The word without a plural "s", or "" when there is none to take off.
pub fn _a_plural_stem(word: &str) -> String {
    if pt::len(word) > 3 && word.ends_with('s') && !pt::ends_with_any(word, &NOT_A_PLURAL) {
        return word[..word.len() - 1].to_string();
    }
    String::new()
}

/// `is_a_place(place)` for the function value the caller passed: a Python callable, carried
/// as its id ("module:qualname:line") and dispatched to its Rust port by name.
fn call_is_a_place(is_a_place: &Value, place: &str) -> bool {
    let Some(f) = is_a_place.as_fn() else {
        panic!("TypeError: is_a_place is not callable");
    };
    let id = f.id.as_str();
    let name = if id.matches(':').count() >= 2 { f.name() } else { id };
    match name {
        "bot.memory.places:_a_place" => crate::memory::places::_a_place(&Value::from(place)).is_some_and(|p| !p.is_empty()),
        other => panic!("bot.memory.when: no port for the callable {other}"),
    }
}

/// Python: bot/memory/when.py:960 `the_part_asked(plan, low, line=None, is_a_place=None)`
///
/// The part of a plan an ask names on its own, as a whole sentence, or "". `line` is the
/// sentence the plan was said in ("" for None); `is_a_place` is the caller's own reading of a
/// place, a function value (`Value::None` for None).
pub fn the_part_asked(plan: &str, low: &str, line: &str, is_a_place: &Value) -> String {
    let text = sq(plan);
    let ask = low;
    if text.is_empty() || ask.is_empty() {
        return String::new();
    }
    let spans = _the_dates_in(&text);
    if ASKS_WHERE.is_match(ask) {
        let place = _the_place(&text, line, is_a_place);
        if place.is_empty() {
            return String::new();
        }
        return _a_name(&place) + ".";
    }
    if ASKS_WHEN.is_match(ask) {
        let dates: Vec<String> = spans.into_iter().map(|(_, _, said)| said).collect();
        let when = _the_date_asked(ask, &dates);
        return if when.is_empty() { String::new() } else { _sentence(&when) };
    }
    String::new()
}

/// What is left of a phrase once its dates and the joining words they leave are taken out.
fn without_the_date_and_joiner(text: &str) -> String {
    let place = _without_the_date(text);
    let place = TRAILING_JOINER.sub("", &place);
    pt::strip_chars(&place, " .,;:!?-").to_string()
}

/// Python: bot/memory/when.py:991 `_the_place(plan, line, is_a_place=None)`
///
/// The place a plan carries, said the way the person said it, or "" when it has none.
pub fn _the_place(plan: &str, line: &str, is_a_place: &Value) -> String {
    let said = sq(line);
    if !said.is_empty() {
        if let Some(found) = GOES_TO.search(&said) {
            let place = without_the_date_and_joiner(found.str(1));
            if _names_a_place(&place) {
                return place;
            }
        }
    }
    let mut place = without_the_date_and_joiner(plan);
    if let Some(cut) = PLACE_PREP.search(&place) {
        place = pt::strip_chars(cut.str(1), " .,;:!?-").to_string();
    }
    if !_names_a_place(&place) {
        return String::new();
    }
    if !is_a_place.is_none() && !call_is_a_place(is_a_place, &place) {
        return String::new();
    }
    place
}

/// Python: bot/memory/when.py:1020 `_a_name(place)`
///
/// A place said as a name: "new york" -> "New York", "the gym" -> "The gym".
pub fn _a_name(place: &str) -> String {
    const SMALL: [&str; 10] = ["the", "a", "an", "of", "and", "in", "on", "at", "for", "to"];
    let words = pt::split_ref(place);
    let title = words.len() > 1 && !words.iter().any(|word| SMALL.contains(&pt::lower(word).as_str()));
    words
        .iter()
        .enumerate()
        .map(|(turn, word)| {
            if word.chars().next().is_some_and(char::is_alphabetic) && (turn == 0 || title) {
                up1(word)
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Python: bot/memory/when.py:1038 `_without_the_date(text)`
///
/// A phrase with every date taken out of it: "portugal in august" -> "portugal".
pub fn _without_the_date(text: &str) -> String {
    let mut out = text.to_string();
    for (start, end, _) in _the_dates_in(text).into_iter().rev() {
        out = format!("{} {}", &out[..start], &out[end..]);
    }
    sq(&out)
}

/// Python: bot/memory/when.py:1046 `_names_a_place(place)`
///
/// True when a phrase left over from a plan is a place and not the remains of a sentence.
pub fn _names_a_place(place: &str) -> bool {
    if place.is_empty() || !A_LETTER.is_match(place) {
        return false;
    }
    let plain = pt::strip_chars(&pt::lower(place), " .,;:!?-").to_string();
    if NO_PLACE.contains(&plain) {
        return false;
    }
    let words = LETTERS.findall_str(&plain);
    if words.is_empty() || words.iter().any(|word| A_VERB.contains(word)) {
        return false;
    }
    !NOT_A_PLACE_WORD.is_match(&plain)
}

/// Python: bot/memory/when.py:1059 `_the_dates_in(text)`
///
/// Every date in a piece of a line, in the order it is said, with where it sits (byte offsets
/// into `text`, which is all they are used for).
pub fn _the_dates_in(text: &str) -> Vec<(usize, usize, String)> {
    let mut out: Vec<(usize, usize, String)> = Vec::new();
    for pattern in [&*FUTURE, &*SHORT_DATE, &*PAST_WHEN] {
        for found in pattern.finditer(text) {
            if out.iter().any(|(start, end, _)| found.start() < *end && *start < found.end()) {
                continue;
            }
            out.push((found.start(), found.end(), found.as_str().to_string()));
        }
    }
    out.sort();
    out
}

/// Python: bot/memory/when.py:1071 `_the_date_asked(ask, dates)`
///
/// The date the ask names the kind of: the month it asks for, the day it asks for.
pub fn _the_date_asked(ask: &str, dates: &[String]) -> String {
    for when in dates.iter().rev() {
        let plain = pt::lower(&_plain(when));
        let words: HashSet<String> = pt::split(&plain).into_iter().collect();
        if ASKS_THE_MONTH.is_match(ask) && words.is_disjoint(&MONTHS) {
            continue;
        }
        if ASKS_THE_DAY.is_match(ask) && !(!words.is_disjoint(&DAYS) || A_DIGIT.is_match(&plain)) {
            continue;
        }
        return when.clone();
    }
    String::new()
}

/// "That's"/"that was" and the date said after it, the way the plain answer says a month or a
/// day: "in June." for a month or a weekday, "on the 12th." otherwise.
fn the_date_after(when: &str, lead: &str) -> String {
    let plain = pt::split(&_plain(when));
    if plain.first().is_some_and(|first| MONTHS.contains(first) || DAYS.contains(first)) {
        return format!("{lead} in {}", _sentence(when));
    }
    format!("{lead} {}", low1(&_sentence(when)))
}

/// Python: bot/memory/when.py:1116 `already_answer(text, memory)`
///
/// Whether the thing the person asks about has happened, from the line that dated it.
pub fn already_answer(text: &str, memory: &Memory) -> Option<String> {
    let line = text;
    if !ALREADY_DONE_ASK.is_match(line) || WHEN_ASK.is_match(line) {
        return None;
    }
    // A recall or a frequency ask is not a question about whether the thing happened.
    if SAYS_IT_AGAIN.is_match(line) || HOW_OFTEN.is_match(line) {
        return None;
    }
    if NOT_A_DATE_TO_GIVE.is_match(line) {
        return None;
    }
    let words: Vec<String> = content(line).into_iter().filter(|word| !ASK_THROWS.contains(word)).collect();
    let asked: Vec<String> = words.into_iter().filter(|word| !TRIP_WORDS.contains(word)).collect();
    for (said, spans) in _dated(memory).into_iter().take(2) {
        if NOT_A_DATE_TO_GIVE.is_match(&said) || !_said_in_the_last_few_turns(memory, &said) {
            continue;
        }
        let found = if asked.is_empty() { String::new() } else { _time_for(&asked, &said, &spans).unwrap_or_default() };
        let when = if !found.is_empty() { found } else { spans.last().map(|s| s.0.clone()).unwrap_or_default() };
        if when.is_empty() {
            continue;
        }
        // Both sides are answered.
        if _has_happened(&said) {
            return Some(the_date_after(&when, "Yes - that was"));
        }
        let plain = pt::split(&_plain(&when));
        if plain.first().is_some_and(|first| MONTHS.contains(first) || DAYS.contains(first)) {
            return Some(format!("No - that's in {}", _sentence(&when)));
        }
        return Some(format!("No - that's {}", low1(&_sentence(&when))));
    }
    None
}

/// Python: bot/memory/when.py:1157 `_has_happened(said)`
///
/// Whether the line that gave the date says the thing is behind the person, not ahead.
pub fn _has_happened(said: &str) -> bool {
    if SAYS_IT_IS_COMING.is_match(said) {
        return false;
    }
    if GOING_ON.is_match(said) && !WAS_GOING_ON.is_match(said) {
        return false;
    }
    _said_in_the_past(said)
}

/// Python: bot/memory/when.py:1167 `_was_sentence(when)`
///
/// A date the line says has already happened: "in june" -> "That was in June."
pub fn _was_sentence(when: &str) -> String {
    the_date_after(when, "That was")
}

/// Python: bot/memory/when.py:1176 `_sentence(when)`
///
/// A date said on its own: "in march" -> "In March."
pub fn _sentence(when: &str) -> String {
    let out: Vec<String> = pt::split(when)
        .into_iter()
        .map(|word| {
            let low = pt::lower(pt::strip_chars(&word, ",;:"));
            if MONTHS.contains(&low) || DAYS.contains(&low) { up1(&word) } else { word }
        })
        .collect();
    let said = pt::rstrip_chars(&out.join(" "), " .").to_string();
    up1(&said) + "."
}

/// Python: bot/memory/when.py:1221 `_the_interval_it_means(line, memory)`
///
/// The length said for the thing the ask names, or "" when no recent line gives one.
pub fn _the_interval_it_means(line: &str, memory: &Memory) -> String {
    let turns = &memory.turns[memory.turns.len().saturating_sub(*BACK_TURNS)..];
    let said_lines: Vec<String> = turns.iter().map(|turn| _said_before_now(memory, turn)).filter(|said| !said.is_empty()).collect();
    if said_lines.is_empty() {
        return String::new();
    }
    let asked: Vec<String> =
        content(line).into_iter().filter(|word| !ASK_THROWS.contains(word) && !UNTIL_THROWS.contains(word)).collect();
    let named: Vec<String> = asked.iter().filter(|word| !TRIP_WORDS.contains(*word)).cloned().collect();
    let points_back = _points_back(line) || asked.is_empty();
    for said in said_lines.iter().rev() {
        let Some(found) = AN_INTERVAL_AHEAD.search(said) else {
            continue;
        };
        if pt::lower(found.name_str("unit")).ends_with("day") || pt::lower(found.name_str("unit")).ends_with("days") {
            if AGO_OR_SINCE.is_match(said) {
                continue; // "in 3 days" after a thing that happened is not ahead of anybody
            }
        }
        if !points_back && !named.is_empty() {
            let line_words = content(said);
            if !named.iter().any(|word| line_words.iter().any(|other| _same(word, other))) {
                continue; // the length is for another thing than the one asked about
            }
        }
        let length = pt::split(found.as_str())[1..].join(" ");
        return format!("You said {length}, so a little less than that by now.");
    }
    String::new()
}

/// Python: bot/memory/when.py:1253 `_said_before_now(memory, turn)`
///
/// The line the person typed on that turn.
pub fn _said_before_now(_memory: &Memory, turn: &Turn) -> String {
    sq(&turn.user)
}

/// Python: bot/memory/when.py:1285 `_said_in_the_past(line)`
///
/// Whether the line the date came from says the thing has already happened.
pub fn _said_in_the_past(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }
    SAID_IN_THE_PAST.is_match(line) && !NOT_IN_THE_PAST.is_match(line)
}

/// Python: bot/memory/when.py:1338 `_a_trip_is_named(dated)`
///
/// Whether the line says a trip or a journey to a place of its own.
pub fn _a_trip_is_named(dated: &str) -> bool {
    let words = LETTERS.findall_str(&sq(&pt::lower(dated)));
    if words.iter().any(|word| A_TRIP_SAID.contains(word)) {
        return true;
    }
    for (place, word) in words.iter().enumerate() {
        if A_GOING.contains(word) && place + 2 < words.len() && words[place + 1] == "to" && !NOT_A_PLACE.contains(&words[place + 2]) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/when.py:1350 `_a_bare_unit_ask_wants_a_length(line, dated)`
///
/// Whether a bare unit ask is about a length, not a countdown.
pub fn _a_bare_unit_ask_wants_a_length(line: &str, dated: &str) -> bool {
    if A_COUNT_TAIL.is_match(line) {
        return false;
    }
    if A_LENGTH_IN_THE_LINE.is_match(dated) {
        return true;
    }
    if !DAYS_WORD.is_match(line) {
        return false; // a stay is counted in days; weeks and months asked bare are a countdown
    }
    if let Some((_, kind)) = _the_day(dated, None) {
        if kind != "month" {
            // A day of its own is a date to count to, whatever the trip.
            return false;
        }
    }
    _a_trip_is_named(dated)
}

/// Python: bot/memory/when.py:1372 `until_ask_matches(line, again=None)`
///
/// Whether the line asks how long until something, a bare unit ask counting as one. (`again`
/// is whether BARE_UNIT_ASK matched, when the caller has read it: `Value::None` to read it here.)
pub fn until_ask_matches(line: &str, again: &Value) -> bool {
    let again = match again {
        Value::None => BARE_UNIT_ASK.match_(line).is_some(),
        other => other.truthy(),
    };
    UNTIL_ASK.is_match(line) || again
}

/// Python: bot/memory/when.py:1393 `_today()`
///
/// The day these replies count from.
pub fn _today() -> NaiveDate {
    crate::py::clock::today()
}

/// Python: bot/memory/when.py:1398 `until_answer(text, memory)`
///
/// How long until the thing the line asks about, or None when no line dates it.
pub fn until_answer(text: &str, memory: &Memory) -> Option<String> {
    let line = text;
    let again = BARE_UNIT_ASK.match_(line).is_some();
    if !until_ask_matches(line, &Value::Bool(again)) || WHEN_SAID.is_match(line) {
        return None;
    }
    if NOT_A_DATE_TO_GIVE.is_match(line) || THEIR_OWN_DEATH.is_match(line) {
        return None; // counting down to a death is not what the person asked for
    }
    if UNTIL_A_LEARNING.is_match(line) {
        return None; // how long till they know is not how long till the date they gave
    }
    let (when, dated) = _the_date_it_means(if again { "how long until that" } else { line }, memory);
    let Some(when) = when.filter(|when| !when.is_empty()) else {
        // No line gave a date, but one may have said how far off the thing was ("in 6 weeks").
        return Some(_the_interval_it_means(line, memory)).filter(|said| !said.is_empty());
    };
    if again && _a_bare_unit_ask_wants_a_length(line, &dated) {
        // The length of the stay is the answer, and the countdown is a question they did not ask.
        return None;
    }
    if AN_INTERVAL_AHEAD.is_match(&when) {
        // The line said how far off the thing is, not the day it falls on.
        return Some(_the_interval_it_means(line, memory)).filter(|said| !said.is_empty());
    }
    // A day on its own is the date the month in play completes.
    let mut month = String::new();
    if _a_day_but_no_month(&when) {
        month = _the_month_in_play(&when, memory);
    }
    let (day, kind) = _the_day(&if month.is_empty() { when.clone() } else { format!("{when} {month}") }, None)?;
    if _said_in_the_past(&dated) {
        // A thing that has happened: asked how long ago, the reply counts back; asked how long
        // till it, the line stands down.
        if !AGO_ASK.is_match(line) {
            return None;
        }
        return _how_long_ago(day, &kind, line);
    }
    if AGO_ASK.is_match(line) {
        // "how long ago was that" about a date that has not come yet: nothing to count back.
        return None;
    }
    _how_long(day, &kind, line)
}

/// Python: bot/memory/when.py:1452 `_the_date_it_means(line, memory)`
///
/// The date words the ask is about and the line they came from, or (None, "").
pub fn _the_date_it_means(line: &str, memory: &Memory) -> (Option<String>, String) {
    let words: Vec<String> =
        content(line).into_iter().filter(|word| !ASK_THROWS.contains(word) && !UNTIL_THROWS.contains(word)).collect();
    if words.is_empty() || _points_back(line) {
        // "how long till that": the newest dated line is the one, as long as it is recent.
        return _the_newest_date(memory);
    }
    let asked: Vec<String> = words.iter().filter(|word| !TRIP_WORDS.contains(*word)).cloned().collect();
    let trip = asked != words;
    for (said, spans) in _dated(memory) {
        if NOT_A_DATE_TO_GIVE.is_match(&said) || THEIR_OWN_DEATH.is_match(&said) {
            continue; // a death, or a date they set for their own, is not counted down to
        }
        if trip && !_is_a_trip_line(&said) {
            continue;
        }
        if let Some(when) = _time_for(&asked, &said, &spans).filter(|when| !when.is_empty()) {
            return (Some(when), said);
        }
    }
    if UNTIL_COUNTDOWN.is_match(line) && !NOT_A_DATE_ASK.is_match(line) {
        // The ask is a countdown and the date just given is what it counts to.
        let (newest, said) = _the_newest_date(memory);
        if newest.as_deref().is_some_and(|n| !n.is_empty()) && _names_a_thing_of_its_own(line, &said) {
            return (None, String::new());
        }
        return (newest, said);
    }
    (None, String::new())
}

/// Python: bot/memory/when.py:1483 `_the_newest_date(memory)`
///
/// The date on the newest line they dated and that line, or (None, "") when it has gone cold.
pub fn _the_newest_date(memory: &Memory) -> (Option<String>, String) {
    let (fixed, _) = _put_right(memory);
    let fixed = sq(&fixed);
    if let Some((said, spans)) = _dated(memory)
        .into_iter()
        .find(|(said, _)| !(NOT_A_DATE_TO_GIVE.is_match(said) || THEIR_OWN_DEATH.is_match(said)))
    {
        if _said_in_the_last_few_turns(memory, &said) || (!fixed.is_empty() && said == fixed) {
            return (Some(last_date(&spans)), said);
        }
    }
    (None, String::new())
}

/// `_plain(when).replace(",", " ").split()`.
fn date_words(when: &str) -> Vec<String> {
    pt::split(&_plain(when).replace(',', " "))
}

/// Python: bot/memory/when.py:1501 `_a_day_but_no_month(when)`
///
/// Whether the date says which day of the month and not which month.
pub fn _a_day_but_no_month(when: &str) -> bool {
    let words = date_words(when);
    if words.is_empty() {
        return false;
    }
    if words.iter().any(|word| MONTH_NUMBER.contains_key(word) || WEEKDAY_NUMBER.contains_key(word)) {
        return false;
    }
    if words.iter().any(|word| matches!(word.as_str(), "today" | "tomorrow" | "tonight" | "weekend" | "month" | "week" | "year")) {
        return false;
    }
    words.iter().any(|word| A_DAY_SAID_ALONE.fullmatch(word).is_some())
}

/// Python: bot/memory/when.py:1518 `_the_month_in_play(when, memory)`
///
/// The month a recent line named, for a day given on its own ("on the 12th"), or "".
pub fn _the_month_in_play(when: &str, memory: &Memory) -> String {
    for (said, spans) in _dated(memory) {
        if !_said_in_the_last_few_turns(memory, &said) {
            break;
        }
        for span in &spans {
            let words = date_words(&span.0);
            if let Some(named) = words.iter().filter(|word| MONTH_NUMBER.contains_key(*word)).last() {
                // The span the ask is about is skipped: it is the bare day itself.
                if pt::strip(&_plain(&span.0)) != pt::strip(&_plain(when)) {
                    return named.clone();
                }
            }
        }
    }
    String::new()
}

/// Python: bot/memory/when.py:1561 `_a_day_number(word)`
pub fn _a_day_number(word: &str) -> bool {
    DAY_NUMBER.match_(word).is_some()
}

/// `date.replace(day=n)`, None where Python raises ValueError.
fn with_day(date: NaiveDate, day: i64) -> Option<NaiveDate> {
    u32::try_from(day).ok().and_then(|day| date.with_day(day))
}

/// `datetime.date(year, month, day)`, None where Python raises ValueError.
fn ymd(year: i64, month: i64, day: i64) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(i32::try_from(year).ok()?, u32::try_from(month).ok()?, u32::try_from(day).ok()?)
}

/// `date + timedelta(days=n)` / `date - timedelta(days=n)`.
fn add_days(date: NaiveDate, n: i64) -> NaiveDate {
    if n >= 0 { date + Days::new(n as u64) } else { date - Days::new((-n) as u64) }
}

/// `date.weekday()`.
fn weekday_of(date: NaiveDate) -> i64 {
    date.weekday().num_days_from_monday() as i64
}

/// Python: bot/memory/when.py:1565 `_the_day(when, today=None)`
///
/// The calendar day a date said the short way falls on, and how it was said ("month", "date",
/// "end", "week"), or None.
pub fn _the_day(when: &str, today: Option<NaiveDate>) -> Option<(NaiveDate, String)> {
    let today = today.unwrap_or_else(_today);
    let words = date_words(when);
    let (mut month, mut day, mut weekday): (Option<i64>, Option<i64>, Option<i64>) = (None, None, None);
    let (mut early, mut has_end, mut has_next, mut has_this, mut has_month, mut has_week) = (false, false, false, false, false, false);
    // Words before the first day, month or weekday are skipped; a word after it ends the read.
    let mut dated = false;
    // A day number on its own is a date only when there is nothing in front of it to step over,
    // or the slice names a month or a weekday.
    let anchored = words.iter().any(|word| MONTH_NUMBER.contains_key(word) || WEEKDAY_NUMBER.contains_key(word));
    for (index, word) in words.iter().enumerate() {
        if !dated {
            let mut started = A_DATE_ITSELF.contains(word);
            if !started && _a_day_number(word) {
                started = anchored || words[..index].iter().all(|before| FILLER.contains(before));
            }
            if !started {
                continue;
            }
        }
        dated = true;
        if let Some(number) = MONTH_NUMBER.get(word) {
            month = Some(*number);
        } else if let Some(number) = WEEKDAY_NUMBER.get(word) {
            weekday = Some(*number);
        } else if matches!(word.as_str(), "early" | "mid" | "late") {
            early = true;
            day = Some(match word.as_str() {
                "early" => 1,
                "mid" => 15,
                _ => 25,
            });
        } else if matches!(word.as_str(), "end" | "ends" | "ending") {
            has_end = true;
        } else if word == "next" {
            has_next = true;
        } else if word == "this" {
            has_this = true;
        } else if word == "month" {
            has_month = true;
        } else if word == "week" {
            has_week = true;
        } else if A_DAY_SAID_ALONE.fullmatch(word).is_some() {
            day = Some(int_of_digits(TWO_DIGITS.match_(word).map(|m| m.as_str()).unwrap_or("0")));
        } else if matches!(
            word.as_str(),
            "the" | "of" | "in" | "on" | "at" | "by" | "start" | "beginning" | "weekend" | "a" | "an" | "for" | "from" | "to" | "until" | "till" | "around" | "about"
        ) {
            continue;
        } else {
            return None;
        }
    }
    let first_of_this_month = with_day(today, 1).unwrap();
    if has_month && has_next {
        return Some((_add_months(first_of_this_month, 1), "month".into()));
    }
    if has_month && has_this {
        return Some((first_of_this_month, "month".into()));
    }
    if has_end && day.is_none() {
        // "the end of may" is the last day of the month; a day number with a month wins.
        if let Some(month) = month {
            return Some((_last_day(_a_year_for(today, month, Some(1))), "end".into()));
        }
        if has_next {
            return Some((_last_day(_add_months(first_of_this_month, 1)), "end".into()));
        }
        return Some((_last_day(first_of_this_month), "end".into()));
    }
    if has_week && has_next {
        return Some((add_days(today, 7), "week".into()));
    }
    if let Some(month) = month {
        let when_day = _a_year_for(today, month, day);
        let kind = if day.is_none() || early { "month" } else { "date" };
        return Some((when_day, kind.into()));
    }
    if let Some(weekday) = weekday {
        let ahead = (weekday - weekday_of(today)).rem_euclid(7);
        let ahead = if ahead == 0 { 7 } else { ahead };
        return Some((add_days(today, ahead), "date".into()));
    }
    if let Some(day) = day {
        let (upcoming, kind) = _this_month_or_next(today, day);
        return Some((upcoming, if early { "month".into() } else { kind }));
    }
    None
}

/// Python: bot/memory/when.py:1649 `_this_month_or_next(today, day)`
///
/// That day number in this month, or next month when this month's has gone.
pub fn _this_month_or_next(today: NaiveDate, day: i64) -> (NaiveDate, String) {
    match with_day(today, day) {
        None => (_a_year_for(today, today.month() as i64 + 1, Some(day)), "date".into()),
        Some(found) if found >= today => (found, "date".into()),
        Some(_) => (_a_year_for(today, today.month() as i64 + 1, Some(day)), "date".into()),
    }
}

/// Python: bot/memory/when.py:1660 `_a_year_for(today, month, day)`
///
/// That month and day in the year it next comes round.
pub fn _a_year_for(today: NaiveDate, month: i64, day: Option<i64>) -> NaiveDate {
    let mut today = today;
    let mut month = month;
    while month > 12 {
        month -= 12;
        today = match today.with_year(today.year() + 1) {
            Some(next) => next,
            None => panic!("ValueError: day is out of range for month"),
        };
    }
    let day = match day {
        Some(d) if d != 0 => d,
        _ => 1,
    };
    for year in [today.year() as i64, today.year() as i64 + 1] {
        let Some(found) = ymd(year, month, day.min(_days_in(year, month))) else {
            continue;
        };
        if found >= today {
            return found;
        }
    }
    match ymd(today.year() as i64 + 1, month, 1) {
        Some(found) => found,
        None => panic!("ValueError: month must be in 1..12"),
    }
}

/// Python: bot/memory/when.py:1676 `_days_in(year, month)`
pub fn _days_in(year: i64, month: i64) -> i64 {
    if month == 12 {
        return 31;
    }
    match (ymd(year, month + 1, 1), ymd(year, month, 1)) {
        (Some(next), Some(this)) => (next - this).num_days(),
        _ => panic!("ValueError: month must be in 1..12"),
    }
}

/// Python: bot/memory/when.py:1682 `_last_day(first_of_month)`
pub fn _last_day(first_of_month: NaiveDate) -> NaiveDate {
    let days = _days_in(first_of_month.year() as i64, first_of_month.month() as i64);
    with_day(first_of_month, days).unwrap()
}

/// Python: bot/memory/when.py:1686 `_add_months(day, months)`
pub fn _add_months(day: NaiveDate, months: i64) -> NaiveDate {
    let mut month = day.month() as i64 + months;
    let mut year = day.year() as i64;
    while month > 12 {
        month -= 12;
        year += 1;
    }
    match ymd(year, month, 1) {
        Some(found) => found,
        None => panic!("ValueError: month must be in 1..12"),
    }
}

/// The unit an ask counts in: the one it names, else the one that suits the count.
fn the_unit_asked(asked: &str, days: i64) -> (&'static str, &'static str) {
    if WEEKS_WORD.is_match(asked) {
        ("week", "week")
    } else if DAYS_WORD.is_match(asked) {
        ("day", "day")
    } else if MONTHS_WORD.is_match(asked) {
        ("month", "month")
    } else if days <= 14 {
        ("", "day")
    } else if days <= 84 {
        ("", "week")
    } else {
        ("", "month")
    }
}

/// Python: bot/memory/when.py:1695 `_how_long(day, kind, line)`
///
/// The reply: how many days or weeks off it is, and the day itself when they said one.
pub fn _how_long(day: NaiveDate, kind: &str, line: &str) -> Option<String> {
    let today = _today();
    let days = (day - today).num_days();
    if days < 0 {
        return None;
    }
    if days == 0 {
        return Some("That's today.".to_string());
    }
    if days == 1 {
        return Some("That's tomorrow.".to_string());
    }
    let asked = pt::lower(line);
    let (named, mut unit) = the_unit_asked(&asked, days);
    let mut head = String::new();
    if unit == "day" {
        head = format!("That's {days} days away");
    } else if unit == "week" {
        let weeks = py_round(days as f64 / 7.0);
        if weeks <= 1 {
            head = "That's just over a week away".to_string();
        } else if named == "week" || weeks <= 30 {
            // The unit they asked in is the unit they get.
            head = format!("That's about {weeks} weeks away");
        } else {
            // Past half a year nobody counts in weeks.
            unit = "month";
        }
    }
    if unit == "month" {
        let months = py_round(days as f64 / 30.44);
        head = if months <= 1 { "That's about a month away".to_string() } else { format!("That's about {months} months away") };
    }
    let said = _the_day_said(day, kind);
    Some(if said.is_empty() { format!("{head}.") } else { format!("{head} - {said}.") })
}

/// Python: bot/memory/when.py:1740 `_how_long_ago(day, kind, line)`
///
/// The reply to a count-back: how long ago the day they gave was, and the day itself.
pub fn _how_long_ago(day: NaiveDate, kind: &str, line: &str) -> Option<String> {
    let today = _today();
    let past = _a_day_in_the_past(day, today)?;
    let days = (today - past).num_days();
    if days <= 0 {
        return None;
    }
    let asked = pt::lower(line);
    let (_, unit) = the_unit_asked(&asked, days);
    let head = if unit == "day" {
        format!("That was {days} days ago")
    } else if unit == "week" {
        let weeks = py_round(days as f64 / 7.0);
        if weeks <= 1 { "That was just over a week ago".to_string() } else { format!("That was about {weeks} weeks ago") }
    } else {
        let months = py_round(days as f64 / 30.44);
        if months <= 1 { "That was about a month ago".to_string() } else { format!("That was about {months} months ago") }
    };
    // "sometime in June" is how a month to come is said; a month that has gone is "in June".
    let said = match kind {
        "month" => format!("in {}", MONTH_NAME[&(past.month() as i64)]),
        "end" => format!("at the end of {}", MONTH_NAME[&(past.month() as i64)]),
        _ => _the_day_said(past, kind),
    };
    Some(if said.is_empty() { format!("{head}.") } else { format!("{head} - {said}.") })
}

/// Python: bot/memory/when.py:1786 `_a_day_in_the_past(day, today)`
///
/// The same month and day as `day`, on the side of today that has already happened.
pub fn _a_day_in_the_past(day: NaiveDate, today: NaiveDate) -> Option<NaiveDate> {
    let (month, number) = (day.month() as i64, day.day() as i64);
    for year in [day.year() as i64 - 1, day.year() as i64 - 2, day.year() as i64 - 3] {
        let Some(found) = ymd(year, month, number.min(_days_in(year, month))) else {
            continue;
        };
        if found <= today {
            return Some(found);
        }
    }
    None
}

/// Python: bot/memory/when.py:1799 `_the_day_said(day, kind)`
///
/// The day itself, said as much as the person said it.
pub fn _the_day_said(day: NaiveDate, kind: &str) -> String {
    let month = &MONTH_NAME[&(day.month() as i64)];
    match kind {
        "month" => format!("sometime in {month}"),
        "end" => format!("the end of {month}"),
        "week" => format!("around {}", WEEKDAY_NAME[&weekday_of(day)]),
        _ => format!("{} {} {month}", WEEKDAY_NAME[&weekday_of(day)], day.day()),
    }
}

/// Python: bot/memory/when.py:1831 `a_plan_moved_to_a_day(text)`
///
/// {"day": "friday night", "line": "can i make it ahead of time"} for a plan moved to another
/// day, or None.
pub fn a_plan_moved_to_a_day(text: &str) -> Option<Value> {
    let said = pt::strip_chars(&sq(&pt::lower(text)), " .!?").to_string();
    if said.is_empty() || pt::split_ref(&said).len() > 14 {
        return None;
    }
    let day = _DAY_NAMED.search(&said)?;
    let moved_to_the_day = Re::new(&format!(r"\b(?:on|till|until)\s+{}", pyre::escape(day.as_str())), 0);
    if !_MOVED.is_match(&said) && !moved_to_the_day.is_match(&said) {
        return None;
    }
    if !_PLAN_VERB.is_match(&said) || !_POINTS_AT_IT.is_match(&said) {
        return None;
    }
    let line = _DAY_NAMED.subn("ahead of time", &said, 1);
    let line = _MOVED.sub("", &line);
    let line = AHEAD_OF_TIME_SAID.sub("ahead of time", &line);
    let line = sq(&line);
    if !line.contains("ahead of time") {
        return None;
    }
    Some(Value::Dict(Arc::new(vec![
        (Value::from("day"), Value::from(sq(day.as_str()))),
        (Value::from("line"), Value::from(line)),
    ])))
}

/// Python: bot/memory/when.py:1958 `how_long_answer(text, memory)`
///
/// The length the chat says it has been, or None.
pub fn how_long_answer(text: &str, memory: &Memory) -> Option<String> {
    let line = sq(text);
    if !HOW_LONG_ASK.is_match(&line) || NOT_A_LENGTH_ASK.is_match(&line) {
        return None;
    }
    if CALENDAR_NOW.match_(&line).is_some() || WHEN_SAID.is_match(&line) {
        return None;
    }
    if let Some(put_right) = _a_day_put_right(memory).filter(|said| !said.is_empty()) {
        return Some(put_right);
    }
    let words: Vec<String> = content(&line).into_iter().filter(|word| !ASK_THROWS.contains(word)).collect();
    let named: Vec<String> = words.into_iter().filter(|word| !ASKED_ABOUT_THE_THING.contains(word)).collect();
    for (said, _said_words) in _the_lines_that_began(memory) {
        if !named.is_empty() && !_names_it(&named, &said) {
            continue;
        }
        if let Some(answer) = _the_length_from(&said).filter(|said| !said.is_empty()) {
            return Some(answer);
        }
    }
    None
}

/// Python: bot/memory/when.py:1997 `_a_day_put_right(memory)`
///
/// The length from a day the person has just put right, or None.
pub fn _a_day_put_right(memory: &Memory) -> Option<String> {
    let turns = &memory.turns;
    if turns.len() < 2 {
        return None;
    }
    let said = sq(&turns[turns.len() - 1].user);
    let earlier = sq(&turns[turns.len() - 2].user);
    if said.is_empty() || earlier.is_empty() || !BEGAN.is_match(&earlier) {
        return None;
    }
    let found = DAY_PUT_RIGHT.search(&said)?;
    let day = found.name_str("day");
    _the_length_from(&if !A_DAY_FORM.is_match(day) { format!("it began in {day}") } else { format!("it began {day}") })
}

/// Python: bot/memory/when.py:2022 `_the_lines_that_began(memory)`
///
/// The lines of theirs that say when something began, newest first, with their words (at most
/// four).
pub fn _the_lines_that_began(memory: &Memory) -> Vec<(String, Vec<String>)> {
    let mut found = Vec::new();
    for turn in memory.turns.iter().rev() {
        let said = sq(&turn.user);
        if said.is_empty() || _is_an_ask(&said) {
            continue;
        }
        if !BEGAN.is_match(&said) {
            continue;
        }
        let words = content(&said);
        found.push((said, words));
        if found.len() >= 4 {
            break;
        }
    }
    found
}

/// Python: bot/memory/when.py:2048 `_names_it(named, said)`
///
/// True when the ask's own words are in the line the day came from.
pub fn _names_it(named: &[String], said: &str) -> bool {
    let said_words = content(said);
    named.iter().all(|word| said_words.iter().any(|other| _same(word, other)))
}

/// Python: bot/memory/when.py:2062 `_the_length_from(said)`
///
/// The length from today back to the day this line names, said as a person says it, or None.
pub fn _the_length_from(said: &str) -> Option<String> {
    let today = _today();
    if let Some(ago) = AGO_LENGTH.search(said) {
        let count = _a_count(ago.name_str("n"));
        let unit = pt::rstrip_chars(&pt::lower(ago.name_str("unit")), "s").to_string();
        if count == 0 {
            return None;
        }
        if unit == "couple" {
            return _about("a couple of weeks", "");
        }
        let days = count * A_UNIT_OF_TIME.get(&unit).copied().unwrap_or(0);
        let words = pt::split(ago.as_str());
        let said_length = words[..words.len() - 1].join(" ");
        let when = if unit == "year" { (today.year() as i64 - count).to_string() } else { String::new() };
        return if days != 0 { _about(&said_length, &when) } else { None };
    }
    if let Some(last) = LAST_LENGTH.search(said) {
        match pt::lower(last.name_str("unit")).as_str() {
            "week" => return _about("a week", ""),
            "fortnight" => return _about("a fortnight", ""),
            "month" => return _about("a month", &_the_month_back(1)),
            "year" => return _about("a year", &(today.year() - 1).to_string()),
            _ => {}
        }
    }
    if let Some(held) = FOR_LENGTH.search(said) {
        if !AGO_LENGTH.is_match(said) {
            let count = _a_count(held.name_str("n"));
            let unit = pt::rstrip_chars(&pt::lower(held.name_str("unit")), "s").to_string();
            if count != 0 && A_UNIT_OF_TIME.contains_key(&unit) {
                return _about(&format!("{} {unit}{}", _a_count_said(count), if count == 1 { "" } else { "s" }), "");
            }
        }
    }
    // PyBot unpacks `_dated(...)`'s (line, dates) pairs as (span, _at), so the "span" read here
    // is the whole line the date was found in.
    for (span, _at) in _dated(&memory_of_one(said)) {
        let Some(day) = _the_day_a_span_names(&span) else {
            continue;
        };
        return _about(&_a_length_in_words((today - day).num_days()), &_a_day_said(day));
    }
    if let Some(weekday) = WEEKDAY_SAID.search(said) {
        if let Some(day) = _the_day_a_weekday_names(weekday.name_str("day")) {
            let when = if day.day() == 1 { _the_month_name(day.month() as i64) } else { pt::capitalize(weekday.name_str("day")) };
            return _about(&_a_length_in_words((today - day).num_days()), &when);
        }
    }
    // A year with nothing else said is enough to count from.
    if let Some(year) = A_YEAR_SAID.search(said) {
        if let Some(day) = _the_day_a_span_names(year.as_str()) {
            return _about(&_a_length_in_words((today - day).num_days()), year.as_str());
        }
    }
    None
}

/// Python: bot/memory/when.py:2117 `memory_of_one(said)`
///
/// One line of theirs, in the shape the date readers take.
pub fn memory_of_one(said: &str) -> Memory {
    let mut memory = Memory::new();
    memory.turns.push(Turn { user: said.to_string(), bot: String::new(), module: None });
    memory
}

/// Python: bot/memory/when.py:2122 `_a_count(said)`
///
/// The number a length is said with, or 0.
pub fn _a_count(said: &str) -> i64 {
    let word = pt::lower(pt::strip(said));
    if all_digits(&word) {
        return int_of_digits(&word);
    }
    NUMBER_WORDS.get(&word).copied().unwrap_or(0)
}

/// Python: bot/memory/when.py:2130 `_the_day_a_span_names(span)`
///
/// The day a date in a line names, choosing the one behind us, or None.
pub fn _the_day_a_span_names(span: &str) -> Option<NaiveDate> {
    let words: Vec<String> = NOT_ALNUM.split(&pt::lower(span)).into_iter().filter(|word| !word.is_empty()).collect();
    let (mut month, mut day, mut year) = (0i64, 0i64, 0i64);
    for word in &words {
        if MONTHS.contains(word) {
            month = _the_month_number(word);
        } else if FOUR_DIGITS.fullmatch(word).is_some() {
            year = int_of_digits(word);
        } else if A_DAY_SAID_ALONE.fullmatch(word).is_some() {
            day = int_of_digits(&NOT_A_DIGIT.sub("", word));
        }
    }
    let today = _today();
    if year != 0 {
        return match ymd(year, if month != 0 { month } else { 1 }, if day != 0 { day } else { 1 }) {
            Some(found) => Some(found),
            None => panic!("ValueError: day is out of range for month"),
        };
    }
    if month == 0 {
        return None;
    }
    for back in 0..3 {
        let found = ymd(today.year() as i64 - back, month, if day != 0 { day } else { 1 })?;
        if found <= today {
            return Some(found);
        }
    }
    None
}

/// Python: bot/memory/when.py:2164 `_the_day_a_weekday_names(word)`
///
/// The most recent day with that name - today when today is that day.
pub fn _the_day_a_weekday_names(word: &str) -> Option<NaiveDate> {
    let index = WEEKDAYS.iter().position(|day| *day == pt::lower(word))? as i64;
    let today = _today();
    let back = (weekday_of(today) - index).rem_euclid(7);
    Some(add_days(today, -back))
}

/// Python: bot/memory/when.py:2173 `_the_month_number(word)`
///
/// The number of a month, 1 to 12 (0 for a word that is not a month's full name).
pub fn _the_month_number(word: &str) -> i64 {
    const ORDER: [&str; 12] =
        ["january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november", "december"];
    ORDER.iter().position(|month| *month == pt::lower(word)).map_or(0, |at| at as i64 + 1)
}

/// Python: bot/memory/when.py:2183 `_the_month_back(months)`
///
/// The name of the month this many months ago.
pub fn _the_month_back(months: i64) -> String {
    let today = _today();
    let mut month = today.month() as i64;
    for _ in 0..months.max(1) {
        month -= 1;
        if month < 1 {
            month = 12;
        }
    }
    _the_month_name(month)
}

/// Python: bot/memory/when.py:2194 `_the_month_name(number)`
pub fn _the_month_name(number: i64) -> String {
    const ORDER: [&str; 12] =
        ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
    if (1..=12).contains(&number) { ORDER[number as usize - 1].to_string() } else { String::new() }
}

/// Python: bot/memory/when.py:2200 `_a_day_said(day)`
///
/// A date said the way a person says one in a sentence - "January", "the 4th of May".
pub fn _a_day_said(day: NaiveDate) -> String {
    if day.day() == 1 {
        return _the_month_name(day.month() as i64);
    }
    format!("the {}{} of {}", day.day(), _an_ordinal_end(day.day() as i64), _the_month_name(day.month() as i64))
}

/// Python: bot/memory/when.py:2207 `_an_ordinal_end(number)`
pub fn _an_ordinal_end(number: i64) -> &'static str {
    if (10..=20).contains(&number.rem_euclid(100)) {
        return "th";
    }
    match number.rem_euclid(10) {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    }
}

/// Python: bot/memory/when.py:2220 `_a_count_said(number)`
///
/// A small count as a word, a larger one as a figure.
pub fn _a_count_said(number: i64) -> String {
    SPELLED_COUNT.get(&number).cloned().unwrap_or_else(|| number.to_string())
}

/// Python: bot/memory/when.py:2226 `_a_length_in_words(days)`
///
/// A number of days said the way a person says a length: "about five months", "two years".
pub fn _a_length_in_words(days: i64) -> String {
    let days = days.max(0);
    if days < 1 {
        return "a day".to_string();
    }
    if days < 12 {
        return format!("{} {}", _a_count_said(days), if days == 1 { "day" } else { "days" });
    }
    if days < 60 {
        return format!("{} weeks", _a_count_said(py_round(days as f64 / 7.0)));
    }
    if days < 730 {
        return format!("{} months", _a_count_said(py_round(days as f64 / 30.44)));
    }
    let years = days as f64 / 365.25;
    let halves = (years * 2.0).trunc() / 2.0;
    if halves == halves.trunc() {
        return format!("{} {}", _a_count_said(halves as i64), if halves == 1.0 { "year" } else { "years" });
    }
    format!("{} and a half years", _a_count_said(halves.trunc() as i64))
}

/// Python: bot/memory/when.py:2244 `_about(length, when)`
///
/// The reply: the length, and the day it is counted from when the day adds something.
pub fn _about(length: &str, when: &str) -> Option<String> {
    let length = sq(length);
    if length.is_empty() {
        return None;
    }
    if !when.is_empty() {
        return Some(format!("{} - since {when}.", _tidy(&length)));
    }
    Some(format!("{}.", _tidy(&length)))
}

/// Python: bot/memory/when.py:2254 `_tidy(said)`
pub fn _tidy(said: &str) -> String {
    up1(pt::strip(said))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn calendar_arithmetic_matches_python() {
        // Values from CPython with today = 2026-11-17.
        let today = day(2026, 11, 17);
        assert_eq!(_a_year_for(today, 3, None), day(2027, 3, 1));
        assert_eq!(_a_year_for(today, 13, Some(31)), day(2027, 1, 31));
        assert_eq!(_this_month_or_next(today, 31), (day(2026, 12, 31), "date".to_string()));
        assert_eq!(_this_month_or_next(today, 3), (day(2026, 12, 3), "date".to_string()));
        assert_eq!(_days_in(2028, 2), 29);
        assert_eq!(_last_day(day(2026, 2, 1)), day(2026, 2, 28));
        assert_eq!(_add_months(day(2026, 12, 1), 1), day(2027, 1, 1));
        assert_eq!(_a_day_in_the_past(day(2027, 6, 1), today), Some(day(2026, 6, 1)));
        assert_eq!(_the_day_said(day(2026, 11, 22), "date"), "Sunday 22 November");
        assert_eq!(_an_ordinal_end(12), "th");
        assert_eq!(_an_ordinal_end(22), "nd");
        assert_eq!(_a_length_in_words(800), "two years");
        assert_eq!(_a_length_in_words(1000), "two and a half years");
        assert_eq!(_a_length_in_words(45), "six weeks");
        assert_eq!(_a_length_in_words(1), "a day");
    }

    #[test]
    fn readers_without_memory() {
        assert_eq!(_lead("is september"), "in september");
        assert_eq!(_lead("is tuesday"), "on tuesday");
        assert_eq!(_lead("for next week"), "next week");
        assert_eq!(_sentence("on the 4th of june"), "On the 4th of June.");
        assert_eq!(_was_sentence("in june"), "That was in June.");
        assert_eq!(_a_date_said("4th may"), "on the 4th of may");
        assert_eq!(_a_date_said("june"), "in june");
        assert_eq!(_a_day_and_a_month("on the 4th", "in june"), "on the 4th of june");
        assert_eq!(_plain("in july"), "in july");
        assert_eq!(_plain("is july"), "july");
        assert_eq!(_a_name("new york"), "New York");
        assert_eq!(_a_name("the gym"), "The gym");
        assert_eq!(_a_plural_stem("results"), "result");
        assert_eq!(_a_plural_stem("news"), "");
        assert!(_backs_off("when is it due"));
    }
}
