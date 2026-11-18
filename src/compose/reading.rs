//! Port of `bot/compose/reading.py`: a number offered into a thread the bot has just answered - a
//! gauge reading against the answer's range, or a doubt ("is that a lot", "is 2400 too much")
//! judged against the range the answer gave.
#![allow(non_snake_case, clippy::all)]

use crate::compose::fallback_chain::{caught, py_format, PyRet};
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.reading.", stringify!($name))));)*
    };
}
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.compose.reading.", stringify!($name))));)*
    };
}
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.reading.", stringify!($name))));)*
    };
}
macro_rules! string_maps {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashMap<String, String>> =
            LazyLock::new(|| data::str_map(concat!("bot.compose.reading.", stringify!($name))));)*
    };
}

regexes!(
    NUMBER, UNIT, RANGE, QUESTION, WORD_RANGE, AN_AGE, TAG_ASK, FOR_THE_THING, FOR_THE_ARTICLE, POINTING_ONE, FIGURE,
    PER_UNIT, DOUBT_UNIT, MONEY_RANGE, AN_AGE_AFTER, _SAME_JOB, _A_RATE, _BUYS, _ASIDE_ENDS, A_WORD_FIGURE,
    A_CLOCK_TIME_BEFORE, A_CLOCK_TIME_AFTER, A_LONE_PRICE, A_RATE_OF_THEIRS, THEIR_FIGURE, THEIR_NUMBER, A_MONEY_PHRASE,
    A_PRICE_VERB, A_MONEY_WORD, BARE_CEILING, ALREADY_JUDGED,
);
word_sets!(
    MEASURED, CARRIED, MONEY_DOUBTS, QUANTITY_WORDS, TIME_DOUBT, DOUBT_WORDS, BARE_DOUBTS, POINTING_AT_SOMETHING,
    DOUBT_UNITS, PLACES_A_FIGURE, A_GAUGE_UNIT, MONEY_WORDS, TOO_MUCH_WORDS, _SAYS_NOTHING_ABOUT_THE_TIER,
    TOO_LITTLE_WORDS, SAID_LOW, SAID_HIGH, _POINTS_AT_IT, A_BILL, A_PERIOD, A_STRETCH, A_DISTANCE, A_CEILING,
);
pools!(
    TAILS, ABOUT_RIGHT, NOT_TOO_LITTLE, NOT_TOO_MUCH, ON_THE_HIGH_SIDE, NOT_TOO_LONG, NOT_TOO_LONG_INSIDE,
    NOT_TOO_SHORT, ON_THE_LONG_SIDE, ON_THE_SHORT_SIDE, ABOVE_THE_USUAL, ENOUGH, ON_THE_LOW_SIDE, THE_LOW_SIDE_FOUND,
    UNDER_THE_USUAL, MORE_THAN_ENOUGH, THE_USUAL_NO, ASKS_FOR_A_PRICE, NO_PRICE_SAID, THE_PRICE_IT_GAVE,
    A_RANGE_THAT_IS_NOT_A_PRICE, THE_RANGE_ITSELF, BELOW_THE_RANGE, ABOVE_THE_RANGE, RIGHT_FOR_THE_THING,
    HIGH_FOR_THE_THING, LOW_FOR_THE_THING, ENOUGH_FOR_THE_THING, SHORT_FOR_THE_THING,
    THE_LOW_SIDE_FOUND_FOR_THE_THING, A_PRICE_NEXT_TO, A_JOURNEY_IS_TIME, A_STRETCH_OF_TIME, THE_REST_IS_UNSURE,
    INSIDE,
);
string_maps!(PLURALS, WORD_ALIAS, UNIT_ALIAS, UNIT_WORDS);

fn named(name: &str) -> &'static Value {
    data::get(&format!("bot.compose.reading.{name}"))
}
fn number(v: &Value) -> f64 {
    v.as_f64().or_else(|| v.as_int().map(|n| n as f64)).expect("a number")
}
static NUMBER_WORDS: LazyLock<HashMap<String, f64>> =
    LazyLock::new(|| named("NUMBER_WORDS").pairs().iter().map(|(k, v)| (k.str().to_string(), number(v))).collect());
static DAYS_IN: LazyLock<HashMap<String, f64>> =
    LazyLock::new(|| named("DAYS_IN").pairs().iter().map(|(k, v)| (k.str().to_string(), number(v))).collect());
static WORD_FIGURE_COUNTS: LazyLock<HashMap<String, f64>> =
    LazyLock::new(|| named("WORD_FIGURE_COUNTS").pairs().iter().map(|(k, v)| (k.str().to_string(), number(v))).collect());
static SHORT: LazyLock<usize> = LazyLock::new(|| named("SHORT").as_int().expect("an int") as usize);
static YEAR: LazyLock<f64> = LazyLock::new(|| number(named("YEAR")));
static NEXT_YEAR: LazyLock<f64> = LazyLock::new(|| number(named("NEXT_YEAR")));
static LOOK_BACK: LazyLock<usize> = LazyLock::new(|| named("LOOK_BACK").as_int().expect("an int") as usize);
static SAME_LEAGUE: LazyLock<f64> = LazyLock::new(|| number(named("SAME_LEAGUE")));

static A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
/// `r"\b" + DOUBT_UNIT.pattern`, compiled with no flags as Python does.
static A_UNIT_FROM_A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(&format!(r"\b{}", DOUBT_UNIT.pattern()), 0));
static A_GREETING_FIRST: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"^(?:(?:good )?(?:hi|hiya|hey|hello|morning|afternoon|evening|yo)\b[ ,!.]*)+", 0));
static A_LEADER_FIRST: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:(?:and|so|but|ok|okay|alright|well|hmm|umm|er)[ ,]+)+", 0));
static AN_IMPRESSION: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"(?P<figure>[£$€]?\s?\d[\d,]*(?:\.\d+)?)\s+(?:seems|seemed|feels|felt|looks|sounds)\s+(?P<rest>\S.*)$", 0)
});
static TURNED_ROUND: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"(?P<figure>[£$€]?\s?\d[\d,]*(?:\.\d+)?)\s*(?:pounds?|quid|euros?|",
            r"dollars?)?\s*(?:for\s+(?!a\s+(?:day|week|month|year|hour)\b)",
            r"(?P<thing>[a-z][\w' ]{0,40}?))?\s*[-,]?\s*(?:is|was|would)\s+",
            r"(?:that|this|it)\s+(?P<word>a lot|too much|expensive|reasonable|ok|okay|",
            r"alright|fine|fair|about right|steep|dear|pricey|cheap)\b"
        ),
        0,
    )
});
static AN_OPINION: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"(?P<figure>[£$€]?\s?\d[\d,]*(?:\.\d+)?)(?P<between>[^0-9]{0,24}?)",
            r"\b(?:seems|seemed|feels|felt|looks|sounds)\s+(?P<word>",
            r"high|low|steep|expensive|cheap|dear|pricey|a lot|too much|too high|",
            r"too low|about right|about average|a rip off|loads)\b"
        ),
        0,
    )
});
static A_CURRENCY: LazyLock<Re> = LazyLock::new(|| Re::new(r"[£$€]", 0));
static A_VERB_FIRST: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"(?:is|was|were|are|would|will|does|do|did|should|isn'?t|isnt|aren'?t|arent|",
            r"that's|that is|thats|it's|it is|its|this is|am i being|am i|are they|",
            r"are we being|have i been|did i)\b"
        ),
        0,
    )
});
static A_POINTER_FIRST: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?:that|this|it|there|they|those)\b", 0));
static A_POINTER_LEAD: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:that|this|it|there|they|those)\b", 0));
static A_HEDGE_LEAD: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:be|being|really|actually|roughly)\s+", 0));
static A_CLAUSE_BREAK: LazyLock<Re> = LazyLock::new(|| Re::new(r"[.;:!?]| - |\band\b", 0));
static YEARS_OLD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:years?|yrs?|months?)\s+old\b", 0));

/// The memory `compose.pick` is handed: `{"turns": [{key: text}]}`.
fn one_turn(key: &str, text_: &str) -> crate::state::Memory {
    // The key that is not given reads as "" in every place `pick` looks, as None does in Python.
    let mut turn = crate::state::Turn::default();
    match key {
        "user" => turn.user = text_.to_string(),
        "bot" => turn.bot = text_.to_string(),
        other => panic!("one_turn: no turn key {other:?}"),
    }
    crate::state::Memory { turns: vec![turn], ..crate::state::Memory::new() }
}

/// `_verdict.pick(options, {"turns": [{key: text}]}, seed)` (verdict's `pick` is compose's).
fn pick_with(options: &[String], key: &str, turn_text: &str, seed: &str) -> String {
    crate::compose::pick(options, &one_turn(key, turn_text), seed)
}

/// `float(s.replace(",", ""))`.
fn to_float(s: &str) -> f64 {
    s.replace(',', "").parse().unwrap_or_else(|_| panic!("ValueError: could not convert string to float: {s:?}"))
}

/// Python's `"%g" % value`.
fn py_g(x: f64) -> String {
    if x.is_nan() {
        return "nan".to_string();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf" } else { "-inf" }.to_string();
    }
    if x == 0.0 {
        return if x.is_sign_negative() { "-0" } else { "0" }.to_string();
    }
    let sci = format!("{x:.5e}");
    let (mantissa, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    let trim = |s: &str| -> String {
        if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s.to_string() }
    };
    if exp < -4 || exp >= 6 {
        let sign = if exp < 0 { '-' } else { '+' };
        return format!("{}e{sign}{:02}", trim(mantissa), exp.abs());
    }
    let decimals = (5 - exp).max(0) as usize;
    trim(&format!("{x:.decimals$}"))
}

/// One range an answer states.
#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    pub low: f64,
    pub high: f64,
    pub mark: String,
    pub money: bool,
    pub unit: String,
    /// Where it starts and ends in the answer, in characters.
    pub at: Option<usize>,
    pub end: Option<usize>,
    pub text: String,
}

/// A doubt about a figure: the doubt word, the figure, and what it is in or for.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Doubt {
    pub word: String,
    pub value: Option<f64>,
    pub money: bool,
    pub unit: String,
    pub subject: String,
    pub rate: bool,
}

/// A figure in one of the person's lines.
#[derive(Clone, Debug, PartialEq)]
pub struct Figure {
    pub value: f64,
    pub money: bool,
    pub unit: String,
}

impl Doubt {
    /// `dict(doubt, **figure)`.
    fn with(&self, figure: &Figure) -> Doubt {
        Doubt { value: Some(figure.value), money: figure.money, unit: figure.unit.clone(), ..self.clone() }
    }
}

/// Python: bot/compose/reading.py:59 `_number(clean)`
/// (value, unit) for a line that is nothing but a reading, or None.
pub fn _number(clean: &str) -> Option<(f64, String)> {
    if QUESTION.is_match(clean) {
        return None;
    }
    let found = NUMBER.findall_str(clean);
    if found.len() != 1 {
        return None;
    }
    let words = A_WORD.findall_str(clean);
    if words.is_empty() || words.iter().any(|w| !CARRIED.contains(w)) {
        return None;
    }
    let unit = UNIT.search(clean).map(|u| u.str(1).to_string()).unwrap_or_default();
    Some((to_float(&found[0]), unit))
}

/// Python: bot/compose/reading.py:76 `_range(said)`
/// (low, high, unit) for the first measured range in the answer, or None.
pub fn _range(said: &str) -> Option<(f64, f64, String)> {
    for found in RANGE.finditer(said) {
        let unit = text::lower(found.str(3));
        if MEASURED.contains(&unit) {
            let (low, high) = (to_float(found.str(1)), to_float(found.str(2)));
            if low == high {
                continue;
            }
            return Some(if low < high { (low, high, unit) } else { (high, low, unit) });
        }
    }
    None
}

/// Python: bot/compose/reading.py:88 `_say(value)`
/// A number as a person writes it: 0.5 stays 0.5, 1.0 is 1.
pub fn _say(value: f64) -> String {
    py_g(value)
}

/// Python: bot/compose/reading.py:93 `_unit(unit)`
/// The unit as it is written: "1 to 1.5 bar", "0 to 5C".
pub fn _unit(unit: &str) -> String {
    match unit {
        "" => String::new(),
        "c" | "celsius" | "centigrade" => "C".to_string(),
        "f" => "F".to_string(),
        "deg" | "degree" | "degrees" => " degrees".to_string(),
        other => format!(" {other}"),
    }
}

/// Python: bot/compose/reading.py:128 `_word_number(word)`
/// A count the answer wrote in words, or None.
pub fn _word_number(word: &str) -> Option<f64> {
    NUMBER_WORDS.get(&text::lower(word)).copied()
}

/// Python: bot/compose/reading.py:142 `_plural(unit, high)`
/// The unit as a range of that many of it is written.
pub fn _plural(unit: &str, high: Option<f64>) -> String {
    match high {
        Some(h) if h > 1.0 => PLURALS.get(unit).cloned().unwrap_or_else(|| unit.to_string()),
        _ => unit.to_string(),
    }
}

/// Python: bot/compose/reading.py:160 `_usual_unit(text, before=None)`
/// The one measurement the answer keeps using, for a range it wrote without one; only a unit
/// used before the range (`before`, in characters) counts.
pub fn _usual_unit(text_: &str, before: Option<usize>) -> String {
    let low = text::lower(text_);
    let mut counts: Vec<(String, usize)> = Vec::new();
    for found in A_UNIT_FROM_A_WORD.finditer(&low) {
        if before.is_some_and(|b| text::char_index(&low, found.start()) > b) {
            continue;
        }
        let raw = found.str(1);
        let unit = UNIT_ALIAS.get(raw).cloned().unwrap_or_else(|| raw.to_string());
        if !unit.is_empty() && !MONEY_WORDS.contains(&unit) {
            match counts.iter_mut().find(|(u, _)| *u == unit) {
                Some((_, n)) => *n += 1,
                None => counts.push((unit, 1)),
            }
        }
    }
    // `max(counts, key=counts.get)`: the first of the most used.
    let mut best: Option<&(String, usize)> = None;
    for entry in &counts {
        if best.is_none_or(|b| entry.1 > b.1) {
            best = Some(entry);
        }
    }
    best.map(|(u, _)| u.clone()).unwrap_or_default()
}

/// Python: bot/compose/reading.py:182 `_add_measured(spans, low, high, unit, usual, at=None, matched="")`
/// Add one measured span, taking the answer's own unit when the range did not name one.
pub fn _add_measured(spans: &mut Vec<Span>, low: Option<f64>, high: Option<f64>, unit: &str, usual: &str, at: Option<usize>, matched: &str) {
    let (Some(low), Some(high)) = (low, high) else { return };
    let unit = if DOUBT_UNITS.contains(unit) {
        unit.to_string()
    } else if DOUBT_UNITS.contains(usual) {
        usual.to_string()
    } else {
        String::new()
    };
    if unit.is_empty() || low == high {
        return;
    }
    let (low, high) = (low.min(high), low.max(high));
    let text_ = format!("{} to {}{}", _say(low), _say(high), _unit(&_plural(&unit, Some(high))));
    let end = at.map(|a| if matched.is_empty() { a } else { a + text::len(matched) });
    spans.push(Span { low, high, mark: String::new(), money: false, unit, at, end, text: text_ });
}

/// Python: bot/compose/reading.py:201 `_same_measure(left, right)`
/// True when two unit words measure the same thing (a bar is a bar, a degree is a degree).
pub fn _same_measure(left: &str, right: &str) -> bool {
    let groups: [&[&str]; 3] = [&["c", "celsius", "centigrade", "deg", "degree", "degrees"], &["f", "fahrenheit"], &["deg", "degree", "degrees"]];
    if groups.iter().any(|g| g.contains(&left) && g.contains(&right)) {
        return true;
    }
    text::slice(left, 0, Some(3)) == text::slice(right, 0, Some(3))
}

/// Python: bot/compose/reading.py:212 `_where(value, low, high, unit)`
/// Which side of the range the reading is on.
pub fn _where(value: f64, low: f64, high: f64, _unit: &str) -> String {
    if value < low {
        "below"
    } else if value > high {
        "above"
    } else {
        "inside"
    }
    .to_string()
}

/// Python: bot/compose/reading.py:221 `_step(said)`
/// The first thing the answer told them to do, in the answer's own words; a missing step is not
/// a reason to drop the reading.
pub fn _step(said: &str) -> Option<String> {
    caught(|| crate::compose::verdict::_step(said)).and_then(|v| v.as_str().map(str::to_string))
}

/// Python: bot/compose/reading.py:235 `_said(memory, clean)`
/// The answer this reading is about: the entry answer the line points at, or the last one.
pub fn _said(memory: &Memory, clean: &str) -> String {
    if let Some(text_) = crate::memory::topics::entry_answer(memory, clean).filter(|t| !t.is_empty()) {
        return text_;
    }
    let Some(last) = memory.turns.last() else { return String::new() };
    if last.module.as_deref() != Some("answers") {
        return String::new();
    }
    if A_WORD.findall_str(clean).len() > *SHORT {
        return String::new();
    }
    let said = last.bot.clone();
    let kept = match memory.get("entry_answer").and_then(|r| r.get("text")) {
        Some(Value::Str(s)) => s.to_string(),
        _ => String::new(),
    };
    if !kept.is_empty() {
        let head = text::split_ref(&text::lower(&kept)).into_iter().take(6).collect::<Vec<_>>().join(" ");
        if !text::squash(&text::lower(&said)).contains(&head) {
            return String::new();
        }
    }
    said
}

/// Python: bot/compose/reading.py:445 `_the_figure_in_the_doubt(rest)`
/// (value, money, unit, rest, rate) for a figure and its unit at the front of a doubt, if any.
pub fn _the_figure_in_the_doubt(rest: &str) -> (Option<f64>, bool, String, String, bool) {
    let Some(found) = FIGURE.match_(rest) else { return (None, false, String::new(), rest.to_string(), false) };
    let value = to_float(found.name_str("number"));
    let mut rest = text::strip(&rest[found.end()..]).to_string();
    let mut unit = String::new();
    // "is 45 an hour normal": the unit said with an article makes the figure a rate.
    let mut rate = false;
    if let Some(per) = PER_UNIT.match_(&rest) {
        rate = true;
        rest = text::strip(&rest[per.end()..]).to_string();
    }
    if let Some(measured) = DOUBT_UNIT.match_(&rest) {
        let raw = text::lower(measured.str(1));
        unit = UNIT_ALIAS.get(&raw).cloned().unwrap_or(raw);
        rest = text::strip(&rest[measured.end()..]).to_string();
    }
    let mut money = found.name("mark").is_some_and(|m| !m.is_empty());
    if MONEY_WORDS.contains(&unit) {
        // "is 80 pounds a lot" is a price
        money = true;
        unit.clear();
        rate = false;
    }
    (Some(value), money, unit, rest, rate)
}

/// `value is not None and YEAR <= value <= NEXT_YEAR and value % 100`: a year, not a figure.
fn a_year(value: Option<f64>) -> bool {
    value.is_some_and(|v| *YEAR <= v && v <= *NEXT_YEAR && v.rem_euclid(100.0) != 0.0)
}

/// Python: bot/compose/reading.py:472 `_doubt(text)`
/// The doubt in a line that only asks whether a figure is a lot or enough, else None.
pub fn _doubt(text_: &str) -> Option<Doubt> {
    let squashed = text::squash(&text::lower(text_));
    let mut line = text::strip_chars(&squashed, " ?!.,;").to_string();
    // A greeting or a leader in front of the doubt is how people open a message.
    line = A_GREETING_FIRST.subn("", &line, 1);
    line = A_LEADER_FIRST.subn("", &line, 1);
    line = text::strip_chars(&TAG_ASK.sub("", &line), " ?!.,;").to_string();
    // "60 seems high": the person's own figure with their doubt on it, read as the same question.
    if let Some(found) = AN_IMPRESSION.match_(&line) {
        line = format!("is {} {}", found.name_str("figure").replace(' ', ""), text::strip(found.name_str("rest")));
    }
    // The figure first and the doubt after it: rebuilt as one question.
    if let Some(turned) = TURNED_ROUND.match_(&line) {
        let thing = text::strip(turned.name_str("thing")).to_string();
        let tail = if thing.is_empty() { String::new() } else { format!(" for {thing}") };
        line = format!("is {} {}{tail}", turned.name_str("figure").replace(' ', ""), turned.name_str("word"));
    }
    // The same opinion with the figure further along the line: read here and returned.
    if let Some(opinion) = AN_OPINION.search(&line) {
        let raw_word = opinion.name_str("word");
        let word = WORD_ALIAS.get(raw_word).cloned().unwrap_or_else(|| raw_word.to_string());
        let figure = opinion.name_str("figure");
        let value = to_float(text::strip(&figure.replace(',', "").replace(['£', '$', '€'], "")));
        let money = A_CURRENCY.is_match(figure);
        if DOUBT_WORDS.contains(&word) && !(a_year(Some(value)) && !money) {
            return Some(Doubt { word, value: Some(value), money, ..Doubt::default() });
        }
    }
    // "thats expensive": the copula said as a statement is the same question.
    let Some(verb) = A_VERB_FIRST.match_(&line) else {
        // The doubt said with no verb at all: only the words that carry a doubt on their own.
        let bare = WORD_ALIAS.get(&line).cloned().unwrap_or_else(|| line.clone());
        if BARE_DOUBTS.contains(&bare) {
            return Some(Doubt { word: bare, ..Doubt::default() });
        }
        return None;
    };
    let verb_said = verb.as_str().to_string();
    let mut rest = text::strip(&line[verb.end()..]).to_string();
    let stated = [
        "that's", "that is", "thats", "it's", "it is", "its", "this is", "am i being", "am i", "are they", "are we being",
        "have i been",
    ]
    .contains(&verb_said.as_str());
    let (value, money, unit, rate);
    if stated || A_POINTER_FIRST.match_(&rest).is_some() {
        rest = text::strip(&A_POINTER_LEAD.subn("", &rest, 1)).to_string();
        if stated {
            let r = _the_figure_in_the_doubt(&rest);
            (value, money, unit, rest, rate) = (r.0, r.1, r.2, r.3, r.4);
        } else {
            (value, money, unit, rate) = (None, false, String::new(), false);
        }
    } else {
        let r = _the_figure_in_the_doubt(&rest);
        (value, money, unit, rest, rate) = (r.0, r.1, r.2, r.3, r.4);
    }
    rest = text::strip(&A_HEDGE_LEAD.subn("", &rest, 1)).to_string();
    let mut changed = true;
    while changed && !rest.is_empty() {
        changed = false;
        for tail in TAILS.iter() {
            if rest.ends_with(&format!(" {tail}")) {
                rest = text::strip(&rest[..rest.len() - tail.len()]).to_string();
                changed = true;
            }
        }
    }
    // The thing the figure is for comes off the end before the doubt word is read.
    let mut subject = String::new();
    if let Some(for_thing) = FOR_THE_THING.search(&rest) {
        if value.is_some() {
            let thing = text::squash(&text::lower(text::strip(for_thing.name_str("thing"))));
            let short_thing = FOR_THE_ARTICLE.subn("", &thing, 1);
            if !POINTING_AT_SOMETHING.contains(&short_thing) && !POINTING_ONE.is_match(&short_thing) {
                subject = thing;
            }
            let at = for_thing.start();
            rest = text::strip(&rest[..at]).to_string();
        }
    }
    let rest = WORD_ALIAS.get(&rest).cloned().unwrap_or(rest);
    if !DOUBT_WORDS.contains(&rest) {
        return None;
    }
    if a_year(value) && !money && unit.is_empty() {
        return None; // a year is not a figure anybody asks to have judged
    }
    Some(Doubt { word: rest, value, money, unit, subject, rate })
}

/// Python: bot/compose/reading.py:580 `_doubt_ranges(said)`
/// (money, measured) spans the answer states, each with its two ends and its own words.
pub fn _doubt_ranges(said: &str) -> (Vec<Span>, Vec<Span>) {
    let mut money = Vec::new();
    let mut measured = Vec::new();
    for found in MONEY_RANGE.finditer(said) {
        let present = |n: &str| found.name(n).is_some_and(|m| !m.is_empty());
        if !(present("mark") || present("mark2") || present("word")) {
            continue; // a range with no money on it is a measure or a stretch of time
        }
        let (low, high) = (to_float(found.name_str("low")), to_float(found.name_str("high")));
        if low == high {
            continue;
        }
        let (low, high) = (low.min(high), low.max(high));
        let mark = found.name("mark").filter(|m| !m.is_empty()).or(found.name("mark2").filter(|m| !m.is_empty())).unwrap_or("£").to_string();
        let text_ = format!("{} to {}", _money(low, &mark), _money(high, &mark));
        money.push(Span { low, high, mark, money: true, unit: String::new(), at: Some(text::char_index(said, found.start())), end: None, text: text_ });
    }
    for found in RANGE.finditer(said) {
        let unit = text::lower(found.str(3));
        if !DOUBT_UNITS.contains(&unit) {
            continue;
        }
        let (low, high) = (to_float(found.str(1)), to_float(found.str(2)));
        if low == high {
            continue;
        }
        let (low, high) = (low.min(high), low.max(high));
        let text_ = format!("{} to {}{}", _say(low), _say(high), _unit(&unit));
        measured.push(Span {
            low,
            high,
            mark: String::new(),
            money: false,
            unit,
            at: Some(text::char_index(said, found.start())),
            end: Some(text::char_index(said, found.end())),
            text: text_,
        });
    }
    for found in WORD_RANGE.finditer(said) {
        let at = text::char_index(said, found.start());
        _add_measured(
            &mut measured,
            _word_number(found.str(1)),
            _word_number(found.str(2)),
            &text::lower(found.str(3)),
            &_usual_unit(said, Some(at)),
            Some(at),
            found.as_str(),
        );
    }
    measured.retain(|span| !_an_age(said, span));
    measured.sort_by_key(|span| span.at.unwrap_or(0));
    (money, measured)
}

/// Python: bot/compose/reading.py:630 `_an_age(text, span)`
/// True when the range is somebody's age ("one to three year old"), not a stretch of time.
pub fn _an_age(text_: &str, span: &Span) -> bool {
    let Some(end) = span.end else { return false };
    let after = text::slice(text_, end as isize, Some(end as isize + 24));
    AN_AGE_AFTER.match_(&after).is_some()
}

/// Python: bot/compose/reading.py:638 `_money(value, mark="£")`
/// A price as a person writes it: 2400 is £2,400.
pub fn _money(value: f64, mark: &str) -> String {
    let body = if value.fract() == 0.0 {
        text::thousands(value.trunc() as i64)
    } else {
        let fixed = format!("{value:.2}");
        let (int_part, frac) = fixed.split_once('.').unwrap_or((&fixed, "00"));
        let negative = int_part.starts_with('-');
        let digits: i64 = int_part.trim_start_matches('-').parse().unwrap_or(0);
        format!("{}{}.{frac}", if negative { "-" } else { "" }, text::thousands(digits))
    };
    format!("{mark}{body}")
}

/// Python: bot/compose/reading.py:762 `_they_named_the_tier(qualifier, mine)`
/// Whether the person's own words name what a later range is for ("in London", "a whole roof").
pub fn _they_named_the_tier(qualifier: &str, mine: &str) -> bool {
    if mine.is_empty() {
        return false;
    }
    let spoken: HashSet<String> = A_WORD.findall_str(&text::lower(mine)).into_iter().collect();
    for word in A_WORD.findall_str(&text::lower(qualifier)) {
        if text::len(&word) < 3 || text::is_digit(&word) || _SAYS_NOTHING_ABOUT_THE_TIER.contains(&word) {
            continue;
        }
        if spoken.contains(&word) || spoken.contains(word.trim_end_matches('s')) {
            return true;
        }
    }
    false
}

/// `text[a:b]` in characters.
fn chars(text_: &str, from: usize, to: usize) -> String {
    text::slice(text_, from as isize, Some(to as isize))
}

/// Python: bot/compose/reading.py:775 `_the_range_for(value, spans, said, mine="")`
/// The range the figure is about among several, or None to keep the answer's leading one.
pub fn _the_range_for(value: Option<f64>, spans: &[Span], said: &str, mine: &str) -> Option<Vec<Span>> {
    let value = value?;
    if spans.len() < 2 {
        return None;
    }
    if spans[0].low <= value && value <= spans[0].high {
        return None; // inside the range the answer leads with: that is the usual being asked about
    }
    let at = |s: &Span| s.at.unwrap_or_else(|| panic!("KeyError: 'at'"));
    let total = text::len(said);
    let mut apart: Vec<(&Span, String)> = Vec::new();
    for place in 1..spans.len() {
        let span = &spans[place];
        let prev = &spans[place - 1];
        let before = chars(said, at(prev) + text::len(&prev.text), at(span));
        let end = at(span) + text::len(&span.text);
        let next = if place + 1 < spans.len() { at(&spans[place + 1]) } else { total };
        let after_all = chars(said, end, next);
        // Only the words attached to the range itself.
        let after = match A_CLAUSE_BREAK.search(&after_all) {
            Some(m) => after_all[..m.start()].to_string(),
            None => after_all.clone(),
        };
        if !(_SAME_JOB.is_match(&before) || _SAME_JOB.is_match(&after)) {
            apart.push((span, format!("{before} {after}")));
        }
    }
    for (span, qualifier) in apart {
        if !(span.low <= value && value <= span.high) {
            continue;
        }
        let lead_tail = chars(said, at(&spans[0]) + text::len(&spans[0].text), total);
        if span.high > spans[0].high && !_they_named_the_tier(&qualifier, mine) && _A_RATE.match_(&lead_tail).is_none() {
            continue; // a dearer variant of the same thing, and they have not said which one
        }
        return Some(vec![span.clone()]);
    }
    None
}

/// Python: bot/compose/reading.py:822 `_the_person_words(memory, line="")`
/// The line and the last few things the person said, as one string.
pub fn _the_person_words(memory: &Memory, line: &str) -> String {
    let turns = &memory.turns;
    let mut words = vec![line.to_string()];
    words.extend(turns[turns.len().saturating_sub(3)..].iter().map(|t| t.user.clone()));
    text::lower(&words.join(" "))
}

/// Python: bot/compose/reading.py:836 `_the_span(doubt, said, mine="")`
/// The span the figure is judged against, or None when the answer states none it can be.
pub fn _the_span(doubt: &Doubt, said: &str, mine: &str) -> Option<Span> {
    let (money, measured) = _doubt_ranges(said);
    let spans: Vec<Span> = if doubt.money {
        _the_range_for(doubt.value, &money, said, mine).unwrap_or_else(|| money.clone())
    } else if !doubt.unit.is_empty() {
        // A figure named in a unit is only judged against a range in that same measurement.
        let mut spans: Vec<Span> = measured.iter().filter(|s| _same_measure(&doubt.unit, &s.unit)).cloned().collect();
        if doubt.rate {
            // "is 45 an hour normal" is about the hourly price the answer gave.
            let priced: Vec<Span> = money
                .iter()
                .filter(|s| {
                    Re::new(&format!(r"{}\s*(?:an?|per|each)?\s*{}s?\b", pyre::escape(&s.text), pyre::escape(&doubt.unit)), pyre::I)
                        .is_match(said)
                })
                .cloned()
                .collect();
            if !priced.is_empty() {
                spans = priced;
            }
        }
        spans
    } else {
        let measured: Vec<Span> = measured.into_iter().filter(|s| !A_GAUGE_UNIT.contains(&s.unit)).collect();
        // A bare figure: the answer decides what it is a figure of.
        let mut both: Vec<Span> = money.iter().chain(measured.iter()).cloned().collect();
        both.sort_by_key(|s| s.at.unwrap_or_else(|| panic!("KeyError: 'at'")));
        let inside: Vec<&Span> = both.iter().filter(|s| doubt.value.is_some_and(|v| s.low <= v && v <= s.high)).collect();
        let spans = if !inside.is_empty() && !money.is_empty() && !measured.is_empty() {
            vec![inside[0].clone()]
        } else {
            _the_range_for(doubt.value, &both, said, mine)
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| if !money.is_empty() { money.clone() } else { measured.clone() })
        };
        if money.is_empty() {
            if let (Some(v), Some(first)) = (doubt.value, spans.first()) {
                if v > first.high * 10.0 || v * 10.0 < first.low {
                    // A bare figure only means a measure when it could be one.
                    return None;
                }
            }
        }
        spans
    };
    if spans.is_empty() {
        return None;
    }
    // Several sums for different parts of the same thing are no yardstick for a figure far below.
    let distinct: HashSet<(u64, u64)> = spans.iter().map(|s| (s.low.to_bits(), s.high.to_bits())).collect();
    if let Some(v) = doubt.value {
        if distinct.len() > 1 && v * 2.0 < spans[0].low && !spans.iter().any(|s| s.low <= v && v <= s.high) {
            return None;
        }
        // A figure past every range takes the range it is nearest to.
        if spans.len() > 1 {
            let top = spans.iter().map(|s| s.high).fold(f64::NEG_INFINITY, f64::max);
            let bottom = spans.iter().map(|s| s.low).fold(f64::INFINITY, f64::min);
            if v > top || v < bottom {
                let gap = |s: &Span| (s.low - v).max(v - s.high).max(0.0);
                let mut best = &spans[0];
                for s in &spans[1..] {
                    if gap(s) < gap(best) {
                        best = s;
                    }
                }
                return Some(best.clone());
            }
        }
    }
    // The first range of that kind is the one the answer leads with.
    Some(spans[0].clone())
}

/// Python: bot/compose/reading.py:921 `_the_side_asked(word)`
/// Which side of the range the doubt's own word asks about, or None when it names neither.
pub fn _the_side_asked(word: &str) -> Option<String> {
    let word = text::lower(text::strip(word));
    if TOO_MUCH_WORDS.contains(&word) {
        return Some("high".to_string());
    }
    if TOO_LITTLE_WORDS.contains(&word) {
        return Some("low".to_string());
    }
    None
}

/// Python: bot/compose/reading.py:931 `_the_side_decides(options, doubt, value, low, high, low_side, found_side)`
/// The frames the doubt's own word asks for, on the side the figure is on.
pub fn _the_side_decides<'a>(
    options: &'a [String],
    doubt: &Doubt,
    value: f64,
    low: f64,
    high: f64,
    low_side: &'a [String],
    found_side: &'a [String],
) -> &'a [String] {
    if doubt.word == "enough" || TIME_DOUBT.contains(&doubt.word) {
        return options;
    }
    let asked = _the_side_asked(&doubt.word);
    if low <= value && value <= high {
        return if asked.as_deref() == Some("low") { &NOT_TOO_LITTLE } else { options };
    }
    if value < low {
        return if asked.as_deref() == Some("low") { low_side } else { found_side };
    }
    options
}

/// Python: bot/compose/reading.py:951 `_judged(doubt, said, mine="")`
/// The verdict on the figure in the answer's own range, or None when nothing can be judged.
pub fn _judged(doubt: &Doubt, said: &str, mine: &str) -> Option<String> {
    let value = doubt.value?;
    let span = _the_span(doubt, said, mine)?;
    let (low, high, span_text) = (span.low, span.high, span.text.clone());
    let word = doubt.word.as_str();
    let options: &Vec<String> = if TIME_DOUBT.contains(word) {
        let long = word == "too long";
        if low <= value && value <= high {
            if long { &NOT_TOO_LONG_INSIDE } else { &NOT_TOO_SHORT }
        } else if value > high {
            if long { &ON_THE_LONG_SIDE } else { &NOT_TOO_SHORT }
        } else if long {
            &NOT_TOO_LONG
        } else {
            &ON_THE_SHORT_SIDE
        }
    } else if low <= value && value <= high {
        if word == "enough" {
            &ENOUGH
        } else if TOO_MUCH_WORDS.contains(word) {
            &NOT_TOO_MUCH
        } else {
            &ABOUT_RIGHT
        }
    } else if value > high {
        if word == "enough" {
            &MORE_THAN_ENOUGH
        } else if TOO_MUCH_WORDS.contains(word) {
            &ON_THE_HIGH_SIDE
        } else {
            &ABOVE_THE_USUAL
        }
    } else if word == "enough" {
        &UNDER_THE_USUAL
    } else {
        &ON_THE_LOW_SIDE
    };
    let options = _the_side_decides(options, doubt, value, low, high, &ON_THE_LOW_SIDE, &THE_LOW_SIDE_FOUND);
    let line = pick_with(options, "bot", &span_text, &span_text);
    let mark = if span.mark.is_empty() { "£" } else { span.mark.as_str() };
    let mut line = py_format(&line, &[("span", &span_text), ("value", &_money(value, mark))]);
    // What the answer said that money buys, when the verdict ends on the range itself.
    let aside = _the_band_aside(said, &span_text);
    if !aside.is_empty() {
        line = _with_the_aside(&line, &span_text, &aside);
    }
    Some(line)
}

/// Python: bot/compose/reading.py:1008 `_the_band_aside(said, span_text)`
/// What the answer says the range buys, or "" - the clause after the range, when it has one.
pub fn _the_band_aside(said: &str, span_text: &str) -> String {
    let said = text::squash(said);
    if said.is_empty() || span_text.is_empty() {
        return String::new();
    }
    let Some(found) = said.find(span_text) else { return String::new() };
    let after = &said[found + span_text.len()..];
    let clause = match _ASIDE_ENDS.search(after) {
        Some(end) => text::strip(&after[..end.start()]).to_string(),
        None => text::strip(after).to_string(),
    };
    if clause.is_empty() || text::len(&clause) > 70 {
        return String::new();
    }
    if _BUYS.match_(&clause).is_none() {
        return String::new();
    }
    text::rstrip_chars(&clause, " .,;:").to_string()
}

/// Python: bot/compose/reading.py:1026 `_with_the_aside(line, span_text, aside)`
/// The verdict with the answer's own clause on the end, when the verdict ends on the range.
pub fn _with_the_aside(line: &str, span_text: &str, aside: &str) -> String {
    let body = text::rstrip(line);
    if !body.ends_with(&format!("{span_text}.")) {
        return line.to_string();
    }
    format!("{}, which {aside}.", &body[..body.len() - 1])
}

/// Python: bot/compose/reading.py:1057 `_in_the_unit(value, unit, wanted)`
/// `value` counted in `wanted`, when both units count time, or None.
pub fn _in_the_unit(value: f64, unit: &str, wanted: &str) -> Option<f64> {
    let mine = DAYS_IN.get(text::lower(unit).trim_end_matches('s')).copied()?;
    let theirs = DAYS_IN.get(text::lower(wanted).trim_end_matches('s')).copied().filter(|t| *t != 0.0)?;
    Some(value * mine / theirs)
}

/// Python: bot/compose/reading.py:1066 `_a_stretch_of_time(unit)`
/// True when a span's unit counts time: days, weeks and months can be too long, bars cannot.
pub fn _a_stretch_of_time(unit: &str) -> bool {
    DAYS_IN.contains_key(text::lower(unit).trim_end_matches('s'))
}

/// Python: bot/compose/reading.py:1082 `_the_usual_is_the_answer(doubt, said)`
/// The verdict on the answer's own usual stretch of time, for "is that too long" with no figure.
pub fn _the_usual_is_the_answer(doubt: &Doubt, said: &str) -> Option<String> {
    if !TIME_DOUBT.contains(&doubt.word) {
        return None;
    }
    let span = _the_span(doubt, said, "")?;
    if span.money || !_a_stretch_of_time(&span.unit) {
        // "is that too long" about a price is not a question at all.
        return None;
    }
    let lines: Vec<String> = THE_USUAL_NO.iter().map(|l| py_format(l, &[("span", &span.text)])).collect();
    Some(pick_with(&lines, "bot", said, &span.text))
}

/// Python: bot/compose/reading.py:1123 `_a_word_figure(said)`
/// {value, money, unit} for a figure the person wrote in words (the last one), or None.
pub fn _a_word_figure(said: &str) -> Option<Figure> {
    let found = A_WORD_FIGURE.finditer(said).pop()?;
    let word = text::lower(found.str(1));
    let value = _word_number(&word).or_else(|| WORD_FIGURE_COUNTS.get(&word).copied())?;
    let raw = text::lower(found.str(2));
    let unit = UNIT_ALIAS.get(&raw).cloned().unwrap_or(raw);
    Some(Figure { value, money: false, unit })
}

/// Python: bot/compose/reading.py:1144 `_the_last_figure_in(said)`
/// The last figure in one of their lines, in digits or in words, or None.
pub fn _the_last_figure_in(said: &str) -> Option<Figure> {
    if let Some(last) = FIGURE.finditer(said).last() {
        if let Some(figure) = _the_figure_in(said, last) {
            return Some(figure);
        }
    }
    _a_word_figure(said)
}

/// Python: bot/compose/reading.py:1155 `_in_the_same_league(value, said)`
/// Whether a range in this answer could be about a figure this size.
pub fn _in_the_same_league(value: Option<f64>, said: &str) -> bool {
    let Some(value) = value else { return true };
    let (money, measured) = _doubt_ranges(said);
    let spans = if money.is_empty() { measured } else { money };
    if spans.is_empty() {
        return true;
    }
    spans.iter().any(|s| s.low <= value * *SAME_LEAGUE && value <= s.high * *SAME_LEAGUE)
}

/// Python: bot/compose/reading.py:1167 `_answer_before(memory, value=None)`
/// (the knowledge answer, its index) the doubt is about, or ("", -1) when there is none.
pub fn _answer_before(memory: &Memory, value: Option<f64>) -> (String, i64) {
    let turns = &memory.turns;
    if turns.is_empty() {
        return (String::new(), -1);
    }
    let n = turns.len();
    let mut elsewhere = (String::new(), -1i64);
    for back in 1..=(*LOOK_BACK).min(n) {
        let turn = &turns[n - back];
        if turn.module.as_deref() != Some("answers") {
            continue;
        }
        let said = &turn.bot;
        let has_range = (!said.is_empty() && RANGE.finditer(said).iter().any(|f| !f.str(3).is_empty()))
            || MONEY_RANGE.is_match(said)
            || WORD_RANGE.is_match(said);
        if has_range {
            if value.is_none() || _in_the_same_league(value, said) {
                return (said.clone(), (n - back) as i64);
            }
            if elsewhere.1 < 0 {
                elsewhere = (said.clone(), (n - back) as i64);
            }
        }
    }
    if elsewhere.1 >= 0 || value.is_some() {
        return elsewhere;
    }
    // No answer gave a range: one that named a single price is still the answer a doubt is about.
    for back in 1..=(*LOOK_BACK).min(n) {
        let turn = &turns[n - back];
        if turn.module.as_deref() != Some("answers") {
            continue;
        }
        if !_a_lone_price(&turn.bot).is_empty() {
            return (turn.bot.clone(), (n - back) as i64);
        }
    }
    elsewhere
}

/// Python: bot/compose/reading.py:1205 `_figure_since(memory, after, narrow=False)`
/// The last figure the person gave around the answer (its own turn, the turns after it, and one
/// turn before it), or None.
pub fn _figure_since(memory: &Memory, after: i64, narrow: bool) -> Option<Figure> {
    let turns = &memory.turns;
    let n = turns.len() as i64;
    if !narrow && 0 <= after && after < n {
        let said_before = &turns[after as usize].user;
        if !said_before.is_empty() {
            if let Some(figure) = _the_last_figure_in(said_before) {
                return Some(figure);
            }
        }
    }
    // `turns[after + 1:]` with Python's slice rules for a negative start.
    let from = after + 1;
    let start = if from >= 0 { from.min(n) } else { (n + from).max(0) } as usize;
    for turn in turns[start..].iter().rev() {
        if let Some(figure) = _the_last_figure_in(&turn.user) {
            return Some(figure);
        }
    }
    // One turn further back: the quote is often given before the question the bot answered.
    if !narrow && after >= 1 && after - 1 < n {
        if let Some(figure) = _the_last_figure_in(&turns[(after - 1) as usize].user) {
            return Some(figure);
        }
    }
    None
}

/// Python: bot/compose/reading.py:1248 `_a_clock_time(said, found, value)`
/// Whether this number of the line names an hour of the day rather than a count of anything.
pub fn _a_clock_time(said: &str, found: &pyre::Match, value: f64) -> bool {
    if A_CLOCK_TIME_AFTER.match_(&said[found.end()..]).is_some() {
        return true;
    }
    0.0 < value && value <= 24.0 && A_CLOCK_TIME_BEFORE.is_match(&said[..found.start()])
}

/// Python: bot/compose/reading.py:1259 `_the_figure_in(said, found)`
/// {value, money, unit} for one number in the person's line, or None when it is not a figure.
pub fn _the_figure_in(said: &str, found: &pyre::Match) -> Option<Figure> {
    let value = to_float(found.name_str("number"));
    let money = found.name("mark").is_some_and(|m| !m.is_empty());
    let unit = UNIT.match_(text::strip(&said[found.end()..])).map(|m| text::lower(m.str(1))).unwrap_or_default();
    if a_year(Some(value)) && !money && unit.is_empty() {
        return None; // a year, not a sum (a round 2000 is a price)
    }
    if AN_AGE.match_(&text::lower(text::strip(said))).is_some() {
        return None;
    }
    if unit.is_empty() && !money && _a_clock_time(said, found, value) {
        return None;
    }
    Some(Figure { value, money, unit })
}

/// Python: bot/compose/reading.py:1336 `_a_lone_price(said)`
/// The text of a single price in the answer, when the answer gave one and no range.
pub fn _a_lone_price(said: &str) -> String {
    let (money, _measured) = _doubt_ranges(said);
    if !money.is_empty() {
        return String::new();
    }
    for found in A_LONE_PRICE.finditer(said) {
        let amount = found.name("amount").filter(|a| !a.is_empty()).or(found.name("amount2")).unwrap_or("");
        if amount.is_empty() {
            continue;
        }
        let Ok(value) = amount.replace(',', "").parse::<f64>() else { continue };
        if value <= 0.0 {
            continue;
        }
        let word = text::lower(found.name_str("word"));
        let mark = found.name("mark").is_some_and(|m| !m.is_empty());
        if !(mark || !word.is_empty()) && *YEAR <= value && value <= *NEXT_YEAR {
            continue;
        }
        return text::strip(found.as_str()).to_string();
    }
    String::new()
}

/// Python: bot/compose/reading.py:1414 `asks_for_the_figure(doubt, said)`
/// The reply that asks for the figure a doubt is about, or None when it is not that doubt.
pub fn asks_for_the_figure(doubt: &Doubt, said: &str) -> Option<String> {
    if !QUANTITY_WORDS.contains(&doubt.word) {
        // "is that ok" asks whether the advice stands, which is the composer's verdict.
        return None;
    }
    let (money, measured) = _doubt_ranges(said);
    let fill = |lines: &[String], fields: &[(&str, &str)]| -> Vec<String> { lines.iter().map(|l| py_format(l, fields)).collect() };
    let spans = if MONEY_DOUBTS.contains(&doubt.word) {
        // A price ask is judged by a price, never by a lifespan or a distance.
        if money.is_empty() {
            if let Some(span) = measured.iter().find(|s| UNIT_WORDS.get(&s.unit).is_some_and(|u| !u.is_empty())) {
                // The answer gave months, years or hours: say which kind of number it was.
                let unit = UNIT_WORDS[&span.unit].clone();
                return Some(pick_with(&fill(&A_RANGE_THAT_IS_NOT_A_PRICE, &[("span", &span.text), ("unit", &unit)]), "bot", said, said));
            }
            let single = _a_lone_price(said);
            if !single.is_empty() {
                return Some(pick_with(&fill(&THE_PRICE_IT_GAVE, &[("span", &single)]), "bot", said, said));
            }
            if !measured.is_empty() && (SAID_LOW.contains(&doubt.word) || SAID_HIGH.contains(&doubt.word)) {
                // A doubt with a side, over a range this file has no word for.
                let span = &measured[0].text;
                let options = if SAID_LOW.contains(&doubt.word) { &*BELOW_THE_RANGE } else { &*ABOVE_THE_RANGE };
                return Some(pick_with(&fill(options, &[("span", span)]), "bot", said, said));
            }
            if !measured.is_empty() {
                // A range of something that is not money and no side named: left to the modules.
                return None;
            }
            return Some(pick_with(&NO_PRICE_SAID, "bot", said, said));
        }
        money
    } else if !money.is_empty() {
        money
    } else {
        measured
    };
    let span = spans.first()?;
    let options: &Vec<String> = if span.money {
        &ASKS_FOR_A_PRICE
    } else if SAID_LOW.contains(&doubt.word) {
        &BELOW_THE_RANGE
    } else if SAID_HIGH.contains(&doubt.word) {
        &ABOVE_THE_RANGE
    } else {
        &THE_RANGE_ITSELF
    };
    Some(pick_with(&fill(options, &[("span", &span.text)]), "bot", said, said))
}

/// Python: bot/compose/reading.py:1474 `_doubt_and_answer(clean, memory)`
/// (the doubt, the answer it is about, that answer's turn) or None.
pub fn _doubt_and_answer(clean: &str, memory: &Memory) -> Option<(Doubt, String, i64)> {
    let doubt = _doubt(clean)?;
    let (said, at) = _answer_before(memory, None);
    if said.is_empty() {
        return None;
    }
    if !doubt.subject.is_empty() && !_the_answer_is_about(&said, &doubt.subject) {
        // A line that names the thing the figure is for is judged on that thing's own range.
        return None;
    }
    Some((doubt, said, at))
}

/// Python: bot/compose/reading.py:1496 `_the_answer_is_about(said, subject)`
/// Whether the answer's opening is about the thing the figure in the line is for.
pub fn _the_answer_is_about(said: &str, subject: &str) -> bool {
    let words: Vec<String> = A_WORD.findall_str(&text::lower(&_the_opening_sentence(said))).into_iter().filter(|w| text::len(w) >= 3).collect();
    for word in A_WORD.findall_str(&text::lower(subject)) {
        if text::len(&word) < 3 {
            continue;
        }
        if words.iter().any(|there| word.contains(there.as_str()) || there.contains(word.as_str())) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/reading.py:1513 `_only_says_that(doubt)`
/// True for a doubt word that says "that" and no more ("normal", "ok", "reasonable", "fine").
pub fn _only_says_that(doubt: &Doubt) -> bool {
    !QUANTITY_WORDS.contains(&doubt.word)
}

/// `str(reply)` of an entry's reply when it is a tuple of lines: Python's repr of a tuple.
fn tuple_repr(lines: &[String]) -> String {
    let parts: Vec<String> = lines.iter().map(|l| crate::understood::py_str_repr(l)).collect();
    if parts.len() == 1 { format!("({},)", parts[0]) } else { format!("({})", parts.join(", ")) }
}

/// Python: bot/compose/reading.py:1528 `_the_usuals(subject)`
/// The price ranges the knowledge base gives for this thing, or () when it gives none.
pub fn _the_usuals(subject: &str) -> Vec<Span> {
    let short = FOR_THE_ARTICLE.subn("", &text::squash(subject), 1);
    if short.is_empty() || text::split_ref(&short).len() > 4 {
        return Vec::new();
    }
    for question in [format!("how much does a {short} cost"), format!("how much does {short} cost"), format!("how much is a {short}")] {
        let reading = Understood { clean: question.clone(), words: text::split(&question), ..Default::default() };
        // A lookup that raises answers nothing.
        let Some(found) = caught(|| crate::knowledge::lookup::find(&reading, None, None, false)) else { continue };
        let Some(entry) = found else { continue };
        let reply = match &entry.reply {
            crate::knowledge::entry::Reply::Text(s) => s.clone(),
            crate::knowledge::entry::Reply::Choices(lines) => tuple_repr(lines),
            crate::knowledge::entry::Reply::Fn(f) => {
                // A row that needs a conversation answers nothing.
                match caught(|| crate::knowledge::registry::reply(f, &mut Memory::new(), &reading)) {
                    Some(r) => r,
                    None => continue,
                }
            }
        };
        let (money, _) = _doubt_ranges(&_the_opening_sentence(&reply));
        if !money.is_empty() {
            return money;
        }
    }
    Vec::new()
}

/// Python: bot/compose/reading.py:1564 `_the_opening_sentence(reply)`
/// The first sentence of an answer, which is where a price answer states its price.
pub fn _the_opening_sentence(reply: &str) -> String {
    let mut text_ = text::squash(reply);
    for mark in [". ", "! ", "? "] {
        let cut = text::find(&text_, mark);
        if cut > 0 {
            text_ = text::slice(&text_, 0, Some(cut));
        }
    }
    text_
}

/// Python: bot/compose/reading.py:1611 `_judged_on_its_subject(doubt)`
/// The verdict on a figure the person brought with them, or None when nothing can judge it.
pub fn _judged_on_its_subject(doubt: Option<&Doubt>) -> Option<String> {
    let doubt = doubt?;
    let value = doubt.value?;
    if doubt.subject.is_empty() || !doubt.unit.is_empty() {
        return None; // a measurement is not a price, and every range here is one
    }
    let spans = _the_usuals(&doubt.subject);
    let span = spans.first()?;
    let (low, high) = (span.low, span.high);
    let enough = doubt.word == "enough";
    let options: &Vec<String> = if low <= value && value <= high {
        if enough { &ENOUGH_FOR_THE_THING } else { &RIGHT_FOR_THE_THING }
    } else if value > high {
        if enough { &ENOUGH_FOR_THE_THING } else { &HIGH_FOR_THE_THING }
    } else if enough {
        &SHORT_FOR_THE_THING
    } else {
        &LOW_FOR_THE_THING
    };
    let options = _the_side_decides(options, doubt, value, low, high, &LOW_FOR_THE_THING, &THE_LOW_SIDE_FOUND_FOR_THE_THING);
    let lines: Vec<String> = options.iter().map(|l| py_format(l, &[("thing", &doubt.subject), ("span", &span.text)])).collect();
    Some(pick_with(&lines, "bot", &span.text, &span.text))
}

/// Python: bot/compose/reading.py:1645 `_the_thing_the_chat_is_on(memory)`
/// The thing the conversation is on, as "a chicken", or "" when nothing says.
pub fn _the_thing_the_chat_is_on(memory: &Memory) -> String {
    let Some(understood) = memory.understood.as_ref() else { return String::new() };
    let r = &understood.reference;
    for value in [&r.it, &r.that, &r.they] {
        let thing = text::strip(value.as_deref().unwrap_or("")).to_string();
        if !thing.is_empty() && !_POINTS_AT_IT.contains(&text::lower(&thing)) {
            return if FOR_THE_ARTICLE.match_(&thing).is_some() { thing } else { format!("a {thing}") };
        }
    }
    String::new()
}

/// Python: bot/compose/reading.py:1659 `_the_doubt_with_its_thing(clean, memory)`
/// The doubt about the figure, about the thing the chat is on when the line names none.
pub fn _the_doubt_with_its_thing(clean: &str, memory: &Memory) -> Option<Doubt> {
    let doubt = _doubt(clean)?;
    if doubt.value.is_none() || !doubt.money {
        return Some(doubt);
    }
    let subject = text::strip(&doubt.subject).to_string();
    if !subject.is_empty() && !_POINTS_AT_IT.contains(&text::lower(&subject)) {
        return Some(doubt);
    }
    let thing = _the_thing_the_chat_is_on(memory);
    if !thing.is_empty() {
        return Some(Doubt { subject: thing, ..doubt });
    }
    Some(doubt)
}

/// Python: bot/compose/reading.py:1678 `_its_own_subject_judges_it(clean, memory)`
/// True when the line names both a figure and the thing it is for, and the base has a range,
/// with no range in the answer before it and no card that answers the line itself.
pub fn _its_own_subject_judges_it(clean: &str, memory: &Memory) -> bool {
    let Some(doubt) = _the_doubt_with_its_thing(clean, memory) else { return false };
    if doubt.subject.is_empty() || doubt.value.is_none() {
        return false;
    }
    let (said, _) = _answer_before(memory, None);
    if !said.is_empty() && _the_answer_is_about(&said, &doubt.subject) {
        return false; // that answer is the range, and the doubt about it is read above
    }
    if _a_card_answers(clean) {
        return false;
    }
    _judged_on_its_subject(Some(&doubt)).is_some()
}

/// Python: bot/compose/reading.py:1699 `_a_card_answers(text)`
/// Whether the knowledge base answers this line on its own words.
pub fn _a_card_answers(text_: &str) -> bool {
    let reading = Understood { clean: text_.to_string(), words: text::split(text_), ..Default::default() };
    caught(|| crate::knowledge::lookup::find(&reading, None, None, false).is_some()).unwrap_or(false)
}

/// Python: bot/compose/reading.py:1711 `judges_a_figure(clean, memory)`
/// True when the line is a doubt this module answers, with an answer it can be judged by.
pub fn judges_a_figure(clean: &str, memory: &Memory) -> bool {
    judges_the_figure_on_the_table(clean, memory) || _its_own_subject_judges_it(clean, memory)
}

/// Python: bot/compose/reading.py:1726 `judges_the_figure_on_the_table(clean, memory)`
/// True when the line is a doubt about the figure the answer on the table already named.
pub fn judges_the_figure_on_the_table(clean: &str, memory: &Memory) -> bool {
    _the_verdict_for_the_table(clean, memory).is_some()
}

/// Python: bot/compose/reading.py:1758 `_a_rate_of_theirs(memory)`
/// Whether the person's own figure was given as a frequency ("3 times a week").
pub fn _a_rate_of_theirs(memory: &Memory) -> bool {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(*LOOK_BACK)..].iter().rev().any(|t| A_RATE_OF_THEIRS.is_match(&t.user))
}

/// Python: bot/compose/reading.py:1770 `_the_verdict_for_the_table(clean, memory)`
/// The verdict on the figure the answer on the table named, or None.
pub fn _the_verdict_for_the_table(clean: &str, memory: &Memory) -> Option<String> {
    let (mut doubt, said, at) = _doubt_and_answer(clean, memory)?;
    if doubt.value.is_none() && PLACES_A_FIGURE.contains(&doubt.word) {
        // The figure may be in the line the answer answers: read it only where the store read a
        // measurement and the answer states a range in the same unit.
        let held = _figure_since(memory, at, _only_says_that(&doubt));
        if held.is_some_and(|h| !h.unit.is_empty()) {
            if let Some(figured) = _figure_of_the_answer(memory, at) {
                doubt = doubt.with(&figured);
            }
        }
    }
    if doubt.value.is_none() {
        let Some(before) = _figure_since(memory, at, _only_says_that(&doubt)) else {
            if MONEY_DOUBTS.contains(&doubt.word) {
                // A price doubt with no figure of theirs behind it is about the answer's own price.
                if let Some(asked) = asks_for_the_figure(&doubt, &said).filter(|a| !a.is_empty()) {
                    return Some(asked);
                }
            }
            // A time doubt with no figure behind it is a question about the answer's own stretch.
            return _the_usual_is_the_answer(&doubt, &said);
        };
        if !before.unit.is_empty() && TIME_DOUBT.contains(&doubt.word) {
            // A figure in days against an answer in weeks is brought into the answer's unit.
            let span = _the_span(&doubt, &said, "");
            let wanted = span.as_ref().map(|s| s.unit.clone()).unwrap_or_default();
            if let Some(converted) = _in_the_unit(before.value, &before.unit, &wanted) {
                let unit = span.map(|s| s.unit).unwrap_or_default();
                return _judged(&Doubt { value: Some(converted), money: false, unit, rate: false, ..doubt.clone() }, &said, "");
            }
        }
        if !before.unit.is_empty() {
            return None;
        }
        doubt = doubt.with(&before);
    }
    let (said, _at) = _the_answer_about_this_figure(&mut doubt, memory, at)?;
    if doubt.unit.is_empty() && _a_rate_of_theirs(memory) {
        if let Some(span) = _the_span(&doubt, &said, "") {
            if _a_stretch_of_time(&span.unit) {
                return None; // a frequency of theirs is not a stretch of time to hold against a range
            }
        }
    }
    let mine = _judged(&doubt, &said, &_the_person_words(memory, clean))?;
    // The memory store's own sentence when the figure is outside the range it read.
    Some(_the_price_verdict(clean, memory).filter(|v| !v.is_empty()).unwrap_or(mine))
}

/// Python: bot/compose/reading.py:1833 `_the_price_verdict(clean, memory)`
/// The memory store's judgement of the figure on the table, when it is the better reply (the
/// figure outside the range); a store that raises has said nothing.
pub fn _the_price_verdict(clean: &str, memory: &Memory) -> Option<String> {
    let said = caught(|| crate::memory::price::judged(clean, memory).into_text())??;
    if said.is_empty() || !_the_store_found_it_outside(clean, memory) {
        return None;
    }
    Some(said)
}

/// The turns as the plain list of dicts Python's memory holds.
fn turns_value(memory: &Memory) -> Value {
    crate::compose::fallback::memory_value(memory).get("turns").cloned().unwrap_or(Value::None)
}

/// Python: bot/compose/reading.py:1859 `_the_store_found_it_outside(clean, memory, a_price)`
/// Whether the figure the store judged is outside the range it judged it against; yes whenever
/// anything about it cannot be read.
pub fn _the_store_found_it_outside(clean: &str, memory: &Memory) -> bool {
    let read = caught(|| -> bool {
        let meant = crate::memory::price::_the_price_they_mean(clean, memory);
        let mine = match meant.items().first() {
            Some(v) if v.truthy() => v.as_f64().or_else(|| v.as_int().map(|n| n as f64)).unwrap_or_else(|| panic!("TypeError: not a number")),
            _ => return true,
        };
        let answers = crate::memory::price::_answers_in(&turns_value(memory));
        let period = crate::memory::price::_the_period_they_said(clean, memory);
        let aim = crate::memory::price::_the_range_that_is_about_it(&answers, mine, &period).into_value();
        if aim.is_none() {
            return true;
        }
        let ends = aim.items();
        let num = |v: &Value| v.as_f64().or_else(|| v.as_int().map(|n| n as f64)).unwrap_or_else(|| panic!("TypeError: not a number"));
        !(num(&ends[0]) <= mine && mine <= num(&ends[1]))
    });
    read.unwrap_or(true)
}

/// Python: bot/compose/reading.py:1880 `reply_to_a_doubt(clean, memory)`
/// The reply to a line that only asks whether a figure is a lot, enough or about right.
pub fn reply_to_a_doubt(clean: &str, memory: &Memory) -> Option<String> {
    if let Some(verdict) = _the_verdict_for_the_table(clean, memory).filter(|v| !v.is_empty()) {
        return Some(verdict);
    }
    let Some((doubt, said, at)) = _doubt_and_answer(clean, memory) else {
        if !_its_own_subject_judges_it(clean, memory) {
            return None;
        }
        return _judged_on_its_subject(_the_doubt_with_its_thing(clean, memory).as_ref());
    };
    if doubt.value.is_some() {
        return None; // a figure with no range to hold it: the modules that know better answer
    }
    match _figure_since(memory, at, _only_says_that(&doubt)) {
        // A doubt with no figure at all is asked for in one; a measured figure is the store's.
        None => asks_for_the_figure(&doubt, &said),
        Some(_) => None,
    }
}

/// Python: bot/compose/reading.py:1901 `_the_answer_about_this_figure(doubt, memory, at)`
/// (the answer, its index) the figure is about, or None when no answer can hold it. The figure
/// found again around an earlier answer is written into `doubt`, as Python's `doubt.update`.
pub fn _the_answer_about_this_figure(doubt: &mut Doubt, memory: &Memory, at: i64) -> Option<(String, i64)> {
    let value = doubt.value?;
    let (said, found_at) = _answer_before(memory, Some(value));
    if said.is_empty() {
        return None;
    }
    if found_at == at {
        return Some((said, at));
    }
    if let Some(again) = _figure_since(memory, found_at, _only_says_that(doubt)) {
        *doubt = doubt.with(&again);
    }
    Some((said, found_at))
}

/// Python: bot/compose/reading.py:1965 `_figure_words(number, unit, mark="")`
/// The figure as it is said back: 45 with "month" is "45 a month", 6 with "miles" is "6 miles".
pub fn _figure_words(number: &str, unit: &str, mark: &str) -> String {
    let text_ = format!("{mark}{number}");
    if unit.is_empty() {
        return text_;
    }
    if A_PERIOD.contains(unit) && ["month", "week", "year", "day"].contains(&unit) {
        return format!("{text_} a {unit}");
    }
    text_ + &_unit(unit)
}

/// Python: bot/compose/reading.py:2007 `_their_figure(memory, back=1)`
/// The last figure the person gave in their own words, as (number, unit, text, money), or None
/// (as a value: the tuple, or None).
pub fn _their_figure(memory: &Memory, back: &Value) -> Value {
    let back = back.as_int().unwrap_or(1);
    let mut seen = 0;
    for turn in memory.turns.iter().rev() {
        let said = turn.user.as_str();
        if said.is_empty() {
            continue;
        }
        seen += 1;
        if seen > back {
            break;
        }
        let low = text::lower(said);
        let mut found = THEIR_FIGURE.finditer(said).pop();
        if found.is_none() {
            // A figure the line explains rather than a unit: "the quote came to 900".
            let words: HashSet<String> = A_WORD.findall_str(&low).into_iter().collect();
            if !words.iter().any(|w| A_BILL.contains(w)) && !A_MONEY_PHRASE.is_match(said) && !A_CURRENCY.is_match(said) && !A_PRICE_VERB.is_match(&low) {
                continue;
            }
            found = THEIR_NUMBER.finditer(said).pop();
            if found.is_none() {
                continue;
            }
        }
        let found = found.expect("a figure");
        let value = to_float(found.name_str("number"));
        let unit = text::lower(found.name("unit").unwrap_or(""));
        let mark = found.name("mark").unwrap_or("");
        if *YEAR <= value && value <= *NEXT_YEAR && mark.is_empty() && unit != "miles" && unit != "km" {
            continue; // a year is not a figure anybody asks to have judged
        }
        if AN_AGE.match_(&text::lower(text::strip(said))).is_some() || YEARS_OLD.is_match(&low) {
            continue; // an age is not a figure anybody asks to have judged
        }
        let words: HashSet<String> = A_WORD.findall_str(&low).into_iter().collect();
        let money = !mark.is_empty()
            || MONEY_WORDS.contains(&unit)
            || words.iter().any(|w| A_BILL.contains(w))
            || A_MONEY_PHRASE.is_match(said)
            || (unit.is_empty() && A_PRICE_VERB.is_match(&low));
        let text_ = _figure_words(found.name_str("number"), &unit, mark);
        return Value::Tuple(Arc::new(vec![Value::Float(value), Value::from(unit.as_str()), Value::from(text_.as_str()), Value::Bool(money)]));
    }
    Value::None
}

/// Python: bot/compose/reading.py:2091 `_already_judged(memory)`
/// Whether the bot's own last line already put the figure against a threshold.
pub fn _already_judged(memory: &Memory) -> bool {
    match memory.turns.iter().rev().find(|t| !t.bot.is_empty()) {
        Some(turn) => ALREADY_JUDGED.is_match(&turn.bot),
        None => false,
    }
}

/// Python: bot/compose/reading.py:2106 `_a_bare_doubt(clean, memory)`
/// (value, unit, text, money) for a bare doubt with the person's own figure and no range.
pub fn _a_bare_doubt(clean: &str, memory: &Memory) -> Option<(f64, String, String, bool)> {
    let doubt = match _doubt(clean) {
        Some(d) => d,
        None => {
            // "is that too far", "is that too long": the words `_doubt` does not carry.
            let said_line = text::rstrip_chars(&text::lower(text::strip(clean)), "?!. ").to_string();
            let found = BARE_CEILING.match_(&said_line)?;
            Doubt { word: found.name_str("word").to_string(), ..Doubt::default() }
        }
    };
    if !A_CEILING.contains(&doubt.word) {
        return None;
    }
    if A_MONEY_WORD.is_match(clean) {
        // "is that a lot of money" names the money, and the composer's own price check reads it.
        return None;
    }
    let (said, _at) = _answer_before(memory, None);
    if !said.is_empty() {
        return None; // there is a range behind it, and `reply_to_a_doubt` judges against that
    }
    if doubt.value.is_some() {
        return None; // the figure is in the line, so the reading that judges a named figure has it
    }
    if _already_judged(memory) {
        return None; // the answer in hand already put this figure against a threshold
    }
    let figure = _their_figure(memory, &Value::Int(1));
    let parts = figure.items();
    if parts.len() != 4 {
        return None;
    }
    Some((number(&parts[0]), parts[1].str().to_string(), parts[2].str().to_string(), parts[3].truthy()))
}

/// Python: bot/compose/reading.py:2141 `reply_to_a_bare_doubt(clean, memory)`
/// The honest reply to a doubt with nothing behind it to judge the figure against.
pub fn reply_to_a_bare_doubt(clean: &str, memory: &Memory) -> Option<String> {
    let (_value, unit, text_, money) = _a_bare_doubt(clean, memory)?;
    let options: &Vec<String> = if money {
        &A_PRICE_NEXT_TO
    } else if A_DISTANCE.contains(&unit) {
        &A_JOURNEY_IS_TIME
    } else if A_STRETCH.contains(&unit) {
        &A_STRETCH_OF_TIME
    } else {
        &THE_REST_IS_UNSURE
    };
    let lines: Vec<String> = options.iter().map(|l| py_format(l, &[("said", &text_)])).collect();
    Some(pick_with(&lines, "t", &text_, &text_))
}

/// Python: bot/compose/reading.py:2159 `reply_for(clean, memory)`
/// The reading in the line, checked against the range in the answer just given.
pub fn reply_for(clean: &str, memory: &Memory) -> Option<String> {
    if AN_AGE.match_(text::strip(clean)).is_some() {
        // "im 52" after an answer about a boiler gauge is the person's age, not a reading.
        return None;
    }
    if let Some(judged) = reply_to_a_doubt(clean, memory).filter(|j| !j.is_empty()) {
        return Some(judged);
    }
    // A doubt with a figure and nothing behind it to judge it against.
    if let Some(bare) = reply_to_a_bare_doubt(clean, memory).filter(|b| !b.is_empty()) {
        return Some(bare);
    }
    let (value, unit) = _number(clean)?;
    let said = _said(memory, clean);
    if said.is_empty() {
        return None;
    }
    let (low, high, says_unit) = _range(&said)?;
    // The person's unit and the answer's have to be the same measurement.
    if !unit.is_empty() && !says_unit.is_empty() && !_same_measure(&unit, &says_unit) {
        return None;
    }
    let span_text = format!("{} to {}{}", _say(low), _say(high), _unit(&says_unit));
    let where_ = _where(value, low, high, &says_unit);
    if where_ == "inside" {
        return Some(pick_inside(value, &span_text, &said));
    }
    let step = _step(&said);
    if where_ == "below" {
        let line = format!("{} is under the {span_text} it should be", _say(value));
        if let Some(step) = step.filter(|s| !s.is_empty()) {
            return Some(format!("{line}: {}.", text::rstrip_chars(&step, ".")));
        }
        return Some(format!("{line} - that is the thing to put right."));
    }
    // Over is not under: nothing is repeated here beyond the range itself.
    Some(format!("{} is over the {span_text} it should be, which is the thing to put right.", _say(value)))
}

/// Python: bot/compose/reading.py:2222 `pick_inside(value, span_text, said)`
pub fn pick_inside(value: f64, span_text: &str, said: &str) -> String {
    let options: Vec<String> = INSIDE.iter().map(|l| py_format(l, &[("value", &_say(value)), ("span", span_text)])).collect();
    pick_with(&options, "bot", said, said)
}

/// Python: bot/compose/reading.py:2227 `_figure_of_the_answer(memory, at)`
/// The figure in the line the answer was given to, when the answer can hold it.
pub fn _figure_of_the_answer(memory: &Memory, at: i64) -> Option<Figure> {
    if at < 0 || at as usize >= memory.turns.len() {
        return None;
    }
    let turn = &memory.turns[at as usize];
    let said = &turn.user;
    let last = FIGURE.finditer(said).pop()?;
    let figure = _the_figure_in(said, &last)?;
    let (money, measured) = _doubt_ranges(&turn.bot);
    if !figure.unit.is_empty() {
        if !measured.iter().any(|s| s.unit == figure.unit) {
            return None;
        }
    } else if figure.money {
        if money.is_empty() {
            return None;
        }
    } else if !measured.is_empty() || money.is_empty() {
        return None;
    }
    Some(figure)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_g_as_python_writes_it() {
        assert_eq!(py_g(1.0), "1");
        assert_eq!(py_g(0.5), "0.5");
        assert_eq!(py_g(2400.0), "2400");
        assert_eq!(py_g(1_000_000.0), "1e+06");
        assert_eq!(py_g(0.0001), "0.0001");
        assert_eq!(py_g(0.00001), "1e-05");
        assert_eq!(py_g(123456.7), "123457");
    }
}
