//! Port of `bot/knowledge/units.py`: the one table of units, every measure people ask about,
//! converted by meaning.
//!
//! Shared knowledge, not a domain: it declares no entries. `maths::solve` asks `convert` first.
//! A line is answered only when it names a measure and the unit wanted, in the same family;
//! anything else is None. The tables, word lists and module-level regexes are the bundle's;
//! regexes Python builds inside a function are built here from the same pattern text.
//!
//! Numbers: Python keeps ints and floats apart when it prints them (`f"{grams} ml"`), so a table
//! figure is a [`Num`]. Offsets are bytes here and code points in Python: every place Python
//! does arithmetic on an index (`clean[start - 24:start]`) goes through `back` / `ahead`.
//!
//! Where Python raises and nothing in this file catches it (`_counted_as("six")`, a mixed span
//! of "forty five minutes"), the port panics with Python's error, which a caller that guards
//! against exceptions treats the same way.
#![allow(non_snake_case)]

use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::Memory;
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

// ---------------------------------------------------------------------------------------------
// Reading the module's data

const P: &str = "bot.knowledge.units.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{P}{name}"))
}

macro_rules! res {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<&'static Re> = LazyLock::new(|| g(stringify!($n)).as_re().unwrap_or_else(|| panic!("{} is not a regex", stringify!($n))));)* };
}
macro_rules! strs {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<Vec<String>> = LazyLock::new(|| g(stringify!($n)).strs());)* };
}
macro_rules! sets {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<HashSet<String>> = LazyLock::new(|| g(stringify!($n)).str_set());)* };
}
macro_rules! maps {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<HashMap<String, String>> = LazyLock::new(|| g(stringify!($n)).str_map());)* };
}
/// `{str: number}` and `{int: str}` dicts, in the dict's order.
macro_rules! num_maps {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<Vec<(String, f64)>> = LazyLock::new(|| g(stringify!($n)).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_f64().unwrap_or(0.0))).collect());)* };
}
macro_rules! int_keyed {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<HashMap<i64, String>> = LazyLock::new(|| g(stringify!($n)).pairs().iter().filter_map(|(k, v)| Some((k.as_int()?, v.str().to_string()))).collect());)* };
}
/// A pattern written inside a Python function, compiled once.
macro_rules! lazy_re {
    ($name:ident, $pat:expr) => {
        static $name: LazyLock<Re> = LazyLock::new(|| Re::new($pat, 0));
    };
    ($name:ident, $pat:expr, $flags:expr) => {
        static $name: LazyLock<Re> = LazyLock::new(|| Re::new($pat, $flags));
    };
}

/// A regex built from a table word at run time (`re.search(rf"\b{re.escape(name)}\b", ...)`),
/// compiled once per distinct pattern.
fn rx(pattern: String) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<String, &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(r) = cache.get(&pattern) {
        return r;
    }
    let r: &'static Re = Box::leak(Box::new(Re::new(&pattern, 0)));
    cache.insert(pattern, r);
    r
}

fn num_of(map: &[(String, f64)], key: &str) -> Option<f64> {
    map.iter().find(|(k, _)| k == key).map(|(_, v)| *v)
}

/// A table figure: Python prints `400` and `2.6` differently, and `0.5` is not `1`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Num {
    I(i64),
    F(f64),
}

impl Num {
    fn of(v: &Value) -> Num {
        match v {
            Value::Float(f) => Num::F(*f),
            other => Num::I(other.as_int().unwrap_or(0)),
        }
    }
    pub fn f(self) -> f64 {
        match self {
            Num::I(i) => i as f64,
            Num::F(f) => f,
        }
    }
    /// `str(n)`.
    fn said(self) -> String {
        match self {
            Num::I(i) => i.to_string(),
            Num::F(f) => text::float_repr(f),
        }
    }
    /// `n * k` for an int `k`.
    fn times(self, k: i64) -> Num {
        match self {
            Num::I(i) => Num::I(i * k),
            Num::F(f) => Num::F(f * k as f64),
        }
    }
    fn truthy(self) -> bool {
        self.f() != 0.0
    }
}

/// `(name, grams)` rows.
fn weight_rows(name: &str) -> Vec<(String, Num)> {
    g(name).items().iter().map(|row| (row.items()[0].str().to_string(), Num::of(&row.items()[1]))).collect()
}

/// `UNITS`: {family: [(unit, words, factor)]}, in the dict's order.
struct Unit {
    name: String,
    words: Vec<String>,
    factor: f64,
}
static UNITS: LazyLock<Vec<(String, Vec<Unit>)>> = LazyLock::new(|| {
    g("UNITS")
        .pairs()
        .iter()
        .map(|(family, items)| {
            let units = items
                .items()
                .iter()
                .map(|u| Unit { name: u.items()[0].str().to_string(), words: u.items()[1].strs(), factor: u.items()[2].as_f64().unwrap_or(0.0) })
                .collect();
            (family.str().to_string(), units)
        })
        .collect()
});
fn family_units(family: &str) -> &'static [Unit] {
    UNITS.iter().find(|(f, _)| f == family).map(|(_, u)| u.as_slice()).unwrap_or_else(|| panic!("KeyError: {family:?}"))
}

maps!(SYSTEM, SAID, SAID_ONE, AS_KITCHEN_SAYS, COUNTED_PACKS, LAND_PLAIN, _TIME_ARTICLE, _TIME_UNIT);
num_maps!(_SHARE_OF_CUP, _SAID_WHOLE, PACK_DRAINED, _VESSEL_ML, _PEOPLE_VALUE, _GROUP_VALUE, _SHARE_VALUE, _NUMERAL, _DENOMINATOR, _TIME_SECONDS);
maps!(_TIME_PLURAL);
int_keyed!(_SAID_PEOPLE, _PEOPLE_WORD_TO_TWENTY, _SAID_COUNT);
strs!(_WHOLE_PIECE, _NO_PLURAL, PACK_IN_ML, _BIRDS);
sets!(MADE_NOT_MEASURED, _NOT_A_TABLE_BEFORE, _A_MEASURE_SAID_ANOTHER_WAY, _NOT_A_FOOD, _BARE_JOIN, _CARRIED_FILLER);

static GRAMS_PER_CUP: LazyLock<Vec<(String, Num)>> = LazyLock::new(|| weight_rows("GRAMS_PER_CUP"));
static GRAMS_PER_TSP: LazyLock<Vec<(String, Num)>> = LazyLock::new(|| weight_rows("GRAMS_PER_TSP"));
static PIECE_GRAMS: LazyLock<Vec<(String, Num)>> = LazyLock::new(|| weight_rows("PIECE_GRAMS"));
static OBJECT_GRAMS: LazyLock<Vec<(String, Num)>> = LazyLock::new(|| weight_rows("OBJECT_GRAMS"));
static PACK_GRAMS: LazyLock<Vec<(String, Num)>> = LazyLock::new(|| weight_rows("PACK_GRAMS"));
static LIQUID_GRAMS_LITRE: LazyLock<Vec<(String, Num)>> = LazyLock::new(|| weight_rows("LIQUID_GRAMS_LITRE"));
static LOOSE: LazyLock<Vec<(String, String)>> =
    LazyLock::new(|| g("LOOSE").items().iter().map(|r| (r.items()[0].str().to_string(), r.items()[1].str().to_string())).collect());
static _COUNT_CONTENT: LazyLock<HashMap<String, (String, String)>> = LazyLock::new(|| {
    g("_COUNT_CONTENT").pairs().iter().map(|(k, v)| (k.str().to_string(), (v.items()[0].str().to_string(), v.items()[1].str().to_string()))).collect()
});
/// `GAS_OVEN`: (mark, °C, °F).
static GAS_OVEN: LazyLock<Vec<(i64, i64, i64)>> = LazyLock::new(|| {
    g("GAS_OVEN").items().iter().map(|r| (r.items()[0].as_int().unwrap(), r.items()[1].as_int().unwrap(), r.items()[2].as_int().unwrap())).collect()
});
static FAN_COOLER: LazyLock<i64> = LazyLock::new(|| g("FAN_COOLER").as_int().unwrap());
static BIRD_GRAMS: LazyLock<i64> = LazyLock::new(|| g("BIRD_GRAMS").as_int().unwrap());
static _TIN_FOR: LazyLock<i64> = LazyLock::new(|| g("_TIN_FOR").as_int().unwrap());
/// `_NAMES`: {word people type: (family, unit)}.
static _NAMES: LazyLock<HashMap<String, (String, String)>> = LazyLock::new(|| {
    g("_NAMES").pairs().iter().map(|(k, v)| (k.str().to_string(), (v.items()[0].str().to_string(), v.items()[1].str().to_string()))).collect()
});
/// `_FRACTION`: {(numerator, denominator): value}.
static _FRACTION: LazyLock<Vec<((String, String), f64)>> = LazyLock::new(|| {
    g("_FRACTION").pairs().iter().map(|(k, v)| ((k.items()[0].str().to_string(), k.items()[1].str().to_string()), v.as_f64().unwrap())).collect()
});
static _WORD_FRACTION: LazyLock<Vec<(&'static Re, String)>> =
    LazyLock::new(|| g("_WORD_FRACTION").items().iter().map(|r| (r.items()[0].as_re().unwrap(), r.items()[1].str().to_string())).collect());
/// `_SPLIT`: {unit: (smaller unit, how many of it)}.
static _SPLIT: LazyLock<HashMap<String, (String, f64)>> = LazyLock::new(|| {
    g("_SPLIT").pairs().iter().map(|(k, v)| (k.str().to_string(), (v.items()[0].str().to_string(), v.items()[1].as_f64().unwrap()))).collect()
});
/// `_FRACTIONS`: {cups: how a cook says it}, in the dict's order.
static _FRACTIONS: LazyLock<Vec<(f64, String)>> =
    LazyLock::new(|| g("_FRACTIONS").pairs().iter().map(|(k, v)| (k.as_f64().unwrap(), v.str().to_string())).collect());
static _AREA_PICTURES: LazyLock<Vec<(String, String, f64)>> = LazyLock::new(|| {
    g("_AREA_PICTURES").items().iter().map(|r| (r.items()[0].str().to_string(), r.items()[1].str().to_string(), r.items()[2].as_f64().unwrap())).collect()
});
static _WORD_SHARE: LazyLock<Vec<(String, f64)>> =
    LazyLock::new(|| g("_WORD_SHARE").items().iter().map(|r| (r.items()[0].str().to_string(), r.items()[1].as_f64().unwrap())).collect());
/// `_TIME_SMALLER`: {unit: (how many, the smaller unit's plural)}.
static _TIME_SMALLER: LazyLock<HashMap<String, (f64, String)>> = LazyLock::new(|| {
    g("_TIME_SMALLER").pairs().iter().map(|(k, v)| (k.str().to_string(), (v.items()[0].as_f64().unwrap(), v.items()[1].str().to_string()))).collect()
});

/// One `PER_PERSON` row: (names, amount, kind, noun).
struct Portion {
    names: Vec<String>,
    amount: Num,
    kind: String,
    noun: String,
}
fn portion_rows(name: &str) -> Vec<Portion> {
    g(name)
        .items()
        .iter()
        .map(|r| {
            let r = r.items();
            Portion { names: r[0].strs(), amount: Num::of(&r[1]), kind: r[2].str().to_string(), noun: r[3].str().to_string() }
        })
        .collect()
}
static PER_PERSON: LazyLock<Vec<Portion>> = LazyLock::new(|| portion_rows("PER_PERSON"));
static PER_PERSON_WHEN_COUNTED: LazyLock<Vec<Portion>> = LazyLock::new(|| portion_rows("PER_PERSON_WHEN_COUNTED"));

/// One `DISHES_FOR_A_TABLE` row: (names, main, side or None, tomatoes, spice).
struct Dish {
    names: Vec<String>,
    main: Vec<String>,
    side: Option<Vec<String>>,
    tomatoes: bool,
    spice: bool,
}
static DISHES_FOR_A_TABLE: LazyLock<Vec<Dish>> = LazyLock::new(|| {
    g("DISHES_FOR_A_TABLE")
        .items()
        .iter()
        .map(|r| {
            let r = r.items();
            Dish { names: r[0].strs(), main: r[1].strs(), side: if r[2].is_none() { None } else { Some(r[2].strs()) }, tomatoes: r[3].truthy(), spice: r[4].truthy() }
        })
        .collect()
});

/// One `WHOLE_THING_SERVES` row.
struct Whole {
    names: Vec<String>,
    size: String,
    feeds: i64,
    stretch: Option<i64>,
    more: String,
    grams_each: i64,
    allowance: String,
    keeps: String,
}
static WHOLE_THING_SERVES: LazyLock<Vec<Whole>> = LazyLock::new(|| {
    g("WHOLE_THING_SERVES")
        .items()
        .iter()
        .map(|r| {
            let r = r.items();
            Whole {
                names: r[0].strs(),
                size: r[1].str().to_string(),
                feeds: r[2].as_int().unwrap(),
                stretch: r[3].as_int(),
                more: r[4].str().to_string(),
                grams_each: r[5].as_int().unwrap(),
                allowance: r[6].str().to_string(),
                keeps: r[7].str().to_string(),
            }
        })
        .collect()
});

res!(
    _MEASURE_AFTER_PIECE, _WHOLE_THING, _UNCOUNTED, _COUNT, _COMPONENT, _WEIGH, _COMPONENT_MEASURED, _DRY_IN_A_MEASURE,
    _COUNTED_THING, _UNIT_WORD, _NUMBER, _TARGET, _SAY_K, _JOINED, _SLASH, _AND_A_HALF, _UNIT_AND_A_HALF, _MIXED, _C_ASKS,
    _F_ASKS, _BARE_TEMPERATURE, _PACK_WORD, _PACK_ASK, _COUNT_ASK, _NOT_A_COUNT, _LOOSE_ASK, _LOOSE_NOT, _IN_A_VESSEL,
    _PER_CUP, _CHILLI_THE_VEGETABLE, _A_DISH_PART, _A_TIME_IS_THE_ASK, _CONTAINER, _AN_ASK, CORRECTING, _PEOPLE_WORD,
    _BARE_FOR_COUNT, _PEOPLE_COUNT, _PORTION_AVOID, _BAKING_PORTION, _PORTION_COUNT, _PORTION_WORD, _TABLE_GROUPS,
    _A_FIGURE, _A_CLOCK, _LAND_ASK, _A_SIZE_JUDGEMENT, _ASKS_FEET, _SAYS_OWN_HEIGHT, _HEIGHT_METRES, _HEIGHT_CM,
    _HEIGHT_FEET, _HEIGHT_FEET_WORD, _NOT_A_HEIGHT, _ASKS_ELSEWHERE, _A_MEASURE_QUESTION, _A_COMPLAINT, _ASKS_HOW_MANY,
    _POUNDS_IS_MONEY, _HOW_TO, _ASKING_AFTER_MEASURE, _SHARE, _SHARE_FRACTION, _SHARE_ASK, _NOT_ABOUT_A_MEASURE,
    _A_TIME_SHARE, _A_MIXED_SPAN, _A_FRACTION_OF_A_MIXED, _A_HALF_SPAN, _A_FRACTION_OF, _A_FRACTION_AS, _A_SPAN_ASK,
    _A_FRACTION_ASK, _A_TARGET_UNIT, _A_UNIT_ASKED, _A_WHOLE_UNIT, _SHARE_THAT, _SHARE_LEAD, _WEIGHT_SAID, _PORTIONS_ASK,
    _A_CUT_NOT_A_BIRD, _FEEDS_ASK, _NOT_A_TABLE_QUESTION, _KEEPING_HENS, _WHOLE_WEIGHT, _COUNT_OF_BIRDS, _TRAILING_JOIN,
    _CARRIED_LEAD, _ASKED_MEASURE, _CARRIED_PEOPLE, _CORRECTION, _NEW_WEIGHT_THEN, _CARRIED_NOW, _SPECIFIC,
);

// ---------------------------------------------------------------------------------------------
// Python's number formatting and index arithmetic

/// `f"{x:g}"`: six significant digits, trailing zeros dropped, an exponent outside 1e-4..1e6.
fn fmt_g(x: f64) -> String {
    if x == 0.0 {
        return if x.is_sign_negative() { "-0".into() } else { "0".into() };
    }
    if !x.is_finite() {
        return text::float_repr(x);
    }
    let sci = format!("{x:.5e}");
    let (mantissa, exp) = sci.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    let trim = |s: &str| -> String { if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s.to_string() } };
    if !(-4..6).contains(&exp) {
        return format!("{}e{}{:02}", trim(mantissa), if exp < 0 { '-' } else { '+' }, exp.abs());
    }
    trim(&format!("{:.*}", (5 - exp) as usize, x))
}

/// `f"{x:,.0f}"`.
fn fmt_commas(x: f64) -> String {
    let s = format!("{x:.0}");
    let (sign, digits) = match s.strip_prefix('-') {
        Some(d) => ("-", d),
        None => ("", s.as_str()),
    };
    let mut out = String::from(sign);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// `text.rstrip("0").rstrip(".")`.
fn strip_zeros(s: &str) -> &str {
    s.trim_end_matches('0').trim_end_matches('.')
}

/// `round(x)`: to the nearest whole number, a half going to the even one.
fn round_py(x: f64) -> i64 {
    text::round_half_even(x)
}

/// `round(x, n)`: the float nearest the correctly rounded decimal.
fn round_to(x: f64, n: usize) -> f64 {
    format!("{x:.n$}").parse().unwrap()
}

/// `s[max(0, at - n):at]`: the `n` characters before a byte offset.
fn back(s: &str, at: usize, n: usize) -> &str {
    let head = &s[..at];
    let start = head.char_indices().rev().nth(n.saturating_sub(1)).map(|(b, _)| b).unwrap_or(0);
    if n == 0 { "" } else { &head[start..] }
}

/// `s[at:at + n]`: the `n` characters after a byte offset.
fn ahead(s: &str, at: usize, n: usize) -> &str {
    let tail = &s[at..];
    &tail[..text::byte_at(tail, n)]
}

/// `s[0].upper() + s[1:]`.
fn up(s: &str) -> String {
    text::upper_first(s)
}

/// `(memory or {}).get("turns")[-n:]`, the users' lines, oldest first.
fn last_turns(memory: Option<&Memory>, n: usize) -> Vec<&str> {
    let Some(memory) = memory else { return Vec::new() };
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(n)..].iter().map(|t| t.user.as_str()).collect()
}

/// `bool(memory)`: a dict with anything in it.
fn has_memory(memory: Option<&Memory>) -> bool {
    memory.is_some_and(|m| {
        m.understood.is_some() || m.intent_wants.is_some() || m.extra_had_intent || m.asked_now.is_some() || !m.turns.is_empty() || !m.extra.is_empty()
    })
}

// ---------------------------------------------------------------------------------------------
// A measure that is the whole of a thing

/// True when the component word is all the measure holds: "a teaspoon of sugar".
pub fn _the_component_is_the_whole_measure(clean: &str) -> bool {
    let Some(found) = _COMPONENT_MEASURED.search(clean) else { return false };
    let again = found.name_str("again");
    if again.is_empty() {
        return true;
    }
    let component = found.name_str("component");
    again == component || again == format!("{component}s")
}

/// The same question written the way the weight tables read it, or None.
pub fn _the_thing_measured_by_itself(clean: &str) -> Option<String> {
    let (found, food) = match _DRY_IN_A_MEASURE.search(clean) {
        Some(found) => {
            let food = found.name_str("food");
            (found, food)
        }
        None => {
            let held = _COMPONENT_MEASURED.search(clean)?;
            if !held.name_str("measure").starts_with("cup") {
                return None;
            }
            if !_the_component_is_the_whole_measure(clean) {
                return None;
            }
            let food = held.name_str("component");
            (held, food)
        }
    };
    let again = found.name_str("again");
    if !again.is_empty() {
        let last = text::split_ref(food).last().copied().unwrap_or("");
        if again != food && again != format!("{food}s") && again != last {
            return None;
        }
    }
    Some(format!("how many grams in a {} of {food}", found.name_str("measure")))
}

lazy_re!(TOO_HEAVY, r"\btoo\s+(?:heavy|much)\b");
lazy_re!(WEIGHT_ASK, r"\bhow\s+(?:much|heavy|many|big)\b|\bwhat(?:'s| is)?\s+(?:the\s+)?weight\b");
lazy_re!(
    NOT_A_WEIGHT_ASK,
    r"\b(?:cost|costs|price|prices|worth|sell|selling|buy|buying|allowance|allowed|limit|limits|maximum|max|can)\b"
);

/// What a whole thing weighs: "how much does a suitcase weigh", "how heavy is a laptop".
pub fn _object_weight(clean: &str) -> Option<String> {
    if !_WEIGH.is_match(clean) || TOO_HEAVY.is_match(clean) {
        return None;
    }
    if !WEIGHT_ASK.is_match(clean) {
        return None; // "my suitcase is too heavy" is a complaint, not a question about one
    }
    if NOT_A_WEIGHT_ASK.is_match(clean) {
        return None;
    }
    let mut best: Option<(&str, Num)> = None;
    for (name, grams) in OBJECT_GRAMS.iter() {
        for words in [name.clone(), _many(name)] {
            let pattern = format!(r"(?<![\w.]){}(?![\w])(?=\s*(?:$|[,?]|weigh|weighs|heavy|about|is\b))", pyre::escape(&words));
            let Some(m) = rx(pattern).search(clean) else { continue };
            if !_COUNTED_THING.is_match(&clean[..m.start()]) {
                continue;
            }
            if best.is_none_or(|b| text::len(name) > text::len(b.0)) {
                best = Some((name, *grams));
            }
        }
    }
    let (name, grams) = best?;
    let article = if name.starts_with(['a', 'e', 'i', 'o', 'u']) { "An" } else { "A" };
    Some(format!("{article} {name} is about {}.", _kilos(grams.f())))
}

/// The words for more than one: "bikes", "washing machines".
pub fn _many(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    if name.ends_with('y') && chars.len() > 1 && !"aeiou".contains(chars[chars.len() - 2]) {
        let stem: String = chars[..chars.len() - 1].iter().collect();
        return format!("{stem}ies");
    }
    if ["o", "ch", "sh", "s", "x"].iter().any(|e| name.ends_with(e)) {
        return format!("{name}es");
    }
    format!("{name}s")
}

/// A weight the way someone says a big thing: 12 kg, 1.5 kg, 200 g.
pub fn _kilos(grams: f64) -> String {
    if grams >= 1000.0 {
        return format!("{} kg", _fmt(grams / 1000.0));
    }
    format!("{} g", _fmt(grams))
}

/// {word people type: (family, unit)} in the order Python's dict holds them, and one regex of
/// every word, longest first. The bundle holds what this gave at import (`_NAMES`,
/// `_UNIT_WORD`), which is what every reader here uses.
pub fn _build() -> (Vec<(String, (String, String))>, Re) {
    let mut names: Vec<(String, (String, String))> = Vec::new();
    fn put(names: &mut Vec<(String, (String, String))>, word: String, to: (String, String), overwrite: bool) {
        match names.iter_mut().find(|(w, _)| *w == word) {
            Some(slot) => {
                if overwrite {
                    slot.1 = to;
                }
            }
            None => names.push((word, to)),
        }
    }
    for (family, items) in UNITS.iter() {
        for unit in items {
            for word in &unit.words {
                put(&mut names, word.clone(), (family.clone(), unit.name.clone()), true);
                if text::len(word) <= 4 && text::is_alpha(word) && !word.ends_with('s') && !_NO_PLURAL.contains(word) {
                    put(&mut names, format!("{word}s"), (family.clone(), unit.name.clone()), false);
                }
            }
        }
    }
    let mut sorted: Vec<&String> = names.iter().map(|(w, _)| w).collect();
    sorted.sort_by(|a, b| text::len(b).cmp(&text::len(a)));
    let pattern = sorted.iter().map(|w| pyre::escape(w)).collect::<Vec<_>>().join("|");
    let re = Re::new(&format!(r"(?<![\w.])(?:{pattern})(?![\w])"), 0);
    (names, re)
}

/// How much of the family's base unit one of this unit is, or None.
pub fn _factor(family: &str, unit: &str) -> Option<f64> {
    family_units(family).iter().find(|u| u.name == unit).map(|u| u.factor)
}

/// The words people type for a unit.
pub fn _words(unit: &str) -> Vec<String> {
    for (_, items) in UNITS.iter() {
        for u in items {
            if u.name == unit {
                return u.words.clone();
            }
        }
    }
    Vec::new()
}

/// A weight the way a cook says it: whole grams once there are ten or more.
pub fn _grams(value: f64) -> String {
    let rounded = value + 1e-9; // 312.5 g rounds up to 313, the way a cook would write it
    if value.abs() >= 10.0 { fmt_commas(rounded) } else { strip_zeros(&format!("{value:.1}")).to_string() }
}

/// A number the way someone would say it: no long tails of decimals.
pub fn _fmt(value: f64) -> String {
    let size = value.abs();
    let said = if size >= 1000.0 {
        fmt_commas(value)
    } else if size >= 100.0 {
        format!("{value:.0}")
    } else if size >= 1.0 {
        format!("{value:.1}")
    } else {
        format!("{value:.2}")
    };
    if said.contains('.') { strip_zeros(&said).to_string() } else { said }
}

/// "a", "an", "one" are 1; anything else is the number written (commas dropped), or None.
pub fn _value(raw: &str) -> Option<f64> {
    if matches!(raw, "a" | "an" | "one") {
        return Some(1.0);
    }
    py_float(&raw.replace(',', ""))
}

/// `float(text)` for the digit strings this file reads; None where Python raises ValueError.
fn py_float(s: &str) -> Option<f64> {
    let s = text::strip(s);
    if s.is_empty() || !s.chars().all(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E')) {
        return None;
    }
    s.parse().ok()
}

/// A unit the line names: where it is, its family and its name.
#[derive(Clone, Debug)]
pub struct Mention {
    pub start: usize,
    pub end: usize,
    pub family: String,
    pub unit: String,
}

/// Every unit the line names, in the order typed.
pub fn _mentions(clean: &str) -> Vec<Mention> {
    let mut found = Vec::new();
    for m in _UNIT_WORD.finditer(clean) {
        let (family, unit) = &_NAMES[m.as_str()];
        // "in" is the preposition far more often than it is inches, so a bare one only counts
        // as a unit when it follows a number: "20 in to cm" is inches, "my oil in miles" is not.
        if m.as_str() == "in" && _number_before(clean, m.start()).is_none() {
            continue;
        }
        found.push(Mention { start: m.start(), end: m.end(), family: family.clone(), unit: unit.clone() });
    }
    found
}

/// The number just before a unit ("20 miles", "a pound"), or None.
pub fn _number_before(clean: &str, start: usize) -> Option<f64> {
    let window = back(clean, start, 24);
    let mut found = None;
    for m in _NUMBER.finditer(window) {
        if text::strip(&window[m.end()..]).is_empty() {
            found = _value(m.str(1));
        }
    }
    found
}

/// The unit as it is said: singular only for one. `one` None is Python's default (`value == 1`).
pub fn _say(value: f64, unit: &str, one: Option<bool>) -> String {
    let said = |unit: &str| SAID.get(unit).cloned().unwrap_or_else(|| unit.to_string());
    if one.unwrap_or(value == 1.0) {
        return SAID_ONE.get(unit).cloned().unwrap_or_else(|| said(unit));
    }
    said(unit)
}

/// A decimal count of weeks or days as whole units and the rest ("12 weeks and 6 days").
pub fn _split_time(result: f64, to_unit: &str) -> Option<String> {
    let (small_unit, per) = _SPLIT.get(to_unit)?;
    if (result - round_py(result) as f64).abs() < 1e-9 {
        return None;
    }
    let whole = result.trunc() as i64;
    let rest = round_py((result - whole as f64) * per);
    if rest == 0 || rest as f64 >= *per {
        return None; // rounding can push the rest up to the next whole unit
    }
    Some(format!("{whole} {} and {rest} {}", _say(whole as f64, to_unit, None), _say(rest as f64, small_unit, None)))
}

/// The converted measure as a sentence.
pub fn _sentence(value: f64, family: &str, from_unit: &str, to_unit: &str, in_phrase: bool) -> Option<String> {
    let factor = _factor(family, from_unit).filter(|f| *f != 0.0)?;
    let other = _factor(family, to_unit).filter(|f| *f != 0.0)?;
    let result = value * factor / other;
    let about = if SYSTEM.get(from_unit) == SYSTEM.get(to_unit) { "" } else { "about " };
    let left = if value == 1.0 { format!("one {}", _say(1.0, from_unit, None)) } else { _amount_phrase(value, from_unit) };
    let split = if family == "time" { _split_time(result, to_unit) } else { None };
    if let Some(split) = split {
        return Some(format!("{} is {about}{split}.", up(&left)));
    }
    let said = _cups_phrase(result, to_unit, family == "mass" && to_unit == "cup");
    let amount = if said != format!("{} {}", _fmt(result), _say(result, to_unit, Some(false))) {
        format!("{about}{said}")
    } else {
        format!("{about}{} {}", _fmt(result), _say(result, to_unit, Some(_fmt(result) == "1")))
    };
    if in_phrase && value == 1.0 {
        // "There is about 1 cup in one mug", not "There are".
        let verb = if (result - 1.0).abs() < 0.06 { "is" } else { "are" };
        return Some(format!("There {verb} {amount} in {left}."));
    }
    Some(format!("{} is {amount}.", up(&left)))
}

lazy_re!(
    C_TO_F,
    r"(-?\d+(?:\.\d+)?)\s*(?:degrees?\s*|°\s*)?(?:c|celsius|centigrade)\b.*?\b(?:in|to|into|as)\s*(?:degrees?\s*)?(?:f|fahrenheit)\b"
);
lazy_re!(
    F_TO_C,
    r"(-?\d+(?:\.\d+)?)\s*(?:degrees?\s*|°\s*)?(?:f|fahrenheit)\b.*?\b(?:in|to|into|as)\s*(?:degrees?\s*)?(?:c|celsius|centigrade)\b"
);

/// "180 c to f": a temperature converted, an oven heat said the way recipes say it.
pub fn _temperature(clean: &str) -> Option<String> {
    for (pattern, wanted) in [(&*C_TO_F, "F"), (&*F_TO_C, "C")] {
        let Some(m) = pattern.search(clean) else { continue };
        let value: f64 = m.str(1).parse().expect("float()");
        let converted = if wanted == "F" { value * 9.0 / 5.0 + 32.0 } else { (value - 32.0) * 5.0 / 9.0 };
        let source = if wanted == "F" { "°C" } else { "°F" };
        if wanted == "C" {
            // An oven heat is the one Fahrenheit figure people convert: 350°F is 177°C by
            // arithmetic, but every recipe calls it 180°C at gas mark 4.
            let (mark, celsius, fahrenheit) = min_by_key(&GAS_OVEN, |row| (row.2 as f64 - value).abs());
            if (fahrenheit as f64 - value).abs() <= 12.0 {
                return Some(format!(
                    "{}{source} is about {}°C - gas mark {mark}, which most recipes call {}°C.",
                    _fmt(value),
                    _fmt(converted),
                    _fmt(celsius as f64)
                ));
            }
        }
        return Some(format!("{}{source} is {}°{wanted}.", _fmt(value), _fmt(converted)));
    }
    None
}

/// `min(rows, key=key)`: the first row with the smallest key.
fn min_by_key(rows: &[(i64, i64, i64)], key: impl Fn(&(i64, i64, i64)) -> f64) -> (i64, i64, i64) {
    let mut best = rows[0];
    for row in &rows[1..] {
        if key(row) < key(&best) {
            best = *row;
        }
    }
    best
}

/// "how do i convert celsius to fahrenheit": the formula, since no number is given.
pub fn _temperature_rule(clean: &str) -> Option<String> {
    if _NUMBER.is_match(clean) {
        return None;
    }
    if !(_HOW_TO.is_match(clean) || _BARE_TEMPERATURE.fullmatch(clean).is_some()) {
        return None;
    }
    let celsius = _C_ASKS.search(clean)?;
    let fahrenheit = _F_ASKS.search(clean)?;
    if celsius.start() < fahrenheit.start() {
        return Some(
            "Multiply the Celsius by 9, divide by 5 and add 32: 20°C is 68°F. The quick way in your head is to double it and add 30, which is close enough for the weather."
                .into(),
        );
    }
    Some("Take away 32, then multiply by 5 and divide by 9: 68°F is 20°C. The quick way in your head is to take away 30 and halve it.".into())
}

/// The gas mark nearest a temperature in °C, the way a recipe would round it.
pub fn _mark_for(celsius: f64) -> (i64, i64, i64) {
    min_by_key(&GAS_OVEN, |row| (row.1 as f64 - celsius).abs())
}

lazy_re!(GAS_MARK, r"gas\s*mark\s*(\d)");
lazy_re!(FAN, r"\bfan\b");
lazy_re!(GAS, r"\bgas\b");
lazy_re!(OVEN_NUMBER, r"(-?\d{2,3})\s*(?:degrees?\s*|°\s*)?(c|celsius|centigrade|fan|f|fahrenheit)\b");
lazy_re!(OVEN_TIME, r"\b(how long|minutes?|hours?|mins)\b");
lazy_re!(OVEN_BARE, r"(?<![\d.])(\d{2,3})(?![\d.])");
lazy_re!(OVEN_ASKING, r"\b(what|which|how|convert|equivalent|set|change|turn|use)\b");
lazy_re!(OVEN_F, r"\d\s*(?:degrees?\s*|°\s*)?(f|fahrenheit)\b");

/// Gas marks and fan ovens, so one table answers every way an oven heat is asked for.
pub fn _oven(clean: &str) -> Option<String> {
    let mark_said = GAS_MARK.search(clean);
    let fan_asked = FAN.is_match(clean);
    let gas_asked = GAS.is_match(clean);
    let number = OVEN_NUMBER.search(clean);
    let cooler = *FAN_COOLER as f64;
    if let Some(mark_said) = mark_said {
        let mark: i64 = mark_said.str(1).parse().expect("int()");
        let row = GAS_OVEN.iter().find(|row| row.0 == mark)?;
        return Some(format!("Gas mark {mark} is {}°C ({}°F), or {}°C in a fan oven.", row.1, row.2, row.1 - *FAN_COOLER));
    }
    if number.is_none() && fan_asked && !OVEN_TIME.is_match(clean) {
        // "what is 180 in a fan oven" names no unit at all: a bare oven heat with the fan in the
        // line is still the same question, as long as the line is asking to convert one.
        if let Some(bare) = OVEN_BARE.search(clean) {
            let celsius: f64 = bare.str(1).parse().expect("float()");
            if OVEN_ASKING.is_match(clean) && (90.0..=280.0).contains(&celsius) {
                return Some(format!("{}°C is {}°C in a fan oven, gas mark {}.", _fmt(celsius), _fmt(celsius - cooler), _mark_for(celsius).0));
            }
        }
    }
    let number = number?;
    let mut celsius: f64 = number.str(1).parse().expect("float()");
    let fahrenheit = matches!(number.str(2), "f" | "fahrenheit");
    if fahrenheit && (fan_asked || gas_asked) {
        // "350f in a fan oven": convert first, then say both settings.
        celsius = (celsius - 32.0) * 5.0 / 9.0;
        let mut said = format!("{}°F is {}°C", number.str(1), _fmt(celsius));
        if fan_asked {
            said += &format!(", or {}°C in a fan oven", _fmt(celsius - cooler));
        }
        said += &format!(", gas mark {}", _mark_for(celsius).0);
        return Some(said + ".");
    }
    if number.str(2) == "fan" && gas_asked {
        // "what gas mark is 160 fan": the figure is the fan dial, so the plain oven is hotter.
        let plain = celsius + cooler;
        return Some(format!("{}°C on a fan setting is {}°C in a plain oven, gas mark {}.", _fmt(celsius), _fmt(plain), _mark_for(plain).0));
    }
    if gas_asked {
        if OVEN_F.is_match(clean) {
            celsius = (celsius - 32.0) * 5.0 / 9.0;
            return Some(format!("{}°F is gas mark {}, and {}°C in a plain oven.", number.str(1), _mark_for(celsius).0, _fmt(celsius)));
        }
        return Some(format!("{}°C is gas mark {}, and {}°C in a fan oven.", _fmt(celsius), _mark_for(celsius).0, _fmt(celsius - cooler)));
    }
    if number.str(2) == "fan" {
        let plain = celsius + cooler;
        return Some(format!("{}°C on a fan setting is {}°C in a plain oven, gas mark {}.", _fmt(celsius), _fmt(plain), _mark_for(plain).0));
    }
    if fan_asked {
        return Some(format!("{}°C is {}°C in a fan oven, gas mark {}.", _fmt(celsius), _fmt(celsius - cooler), _mark_for(celsius).0));
    }
    None
}

lazy_re!(STICKS, r"\bsticks?\b");
lazy_re!(BUTTER_UNIT, r"\b(grams?|g|oz|ounces?|cups?|tablespoons?|tbsp|teaspoons?)\b");
lazy_re!(BUTTER_ASK, r"\b(?:how (?:much|many)|what is|whats|what's|weigh\w*)\b");
lazy_re!(STICK_COUNT, r"(half an?|a|an|one|\d+(?:\.\d+)?)\s+sticks?\b");
lazy_re!(POUNDS, r"\bpounds?\b");
lazy_re!(CUPS, r"\bcups?\b");
lazy_re!(GRAMS_WORD, r"\bgrams?\b|\bg\b");

/// A stick of butter: a weight cooks name instead of giving grams.
pub fn _butter(clean: &str) -> Option<String> {
    if !clean.contains("butter") || !STICKS.is_match(clean) {
        return None;
    }
    if !BUTTER_UNIT.is_match(clean) && !BUTTER_ASK.is_match(clean) {
        return None;
    }
    let count = STICK_COUNT.search(clean);
    if POUNDS.is_match(clean) && !CUPS.is_match(clean) {
        return Some("There are four sticks of butter in a pound, 450 g, so a stick is a quarter of one.".into());
    }
    if clean.contains("block") {
        return Some("A stick is 113 g, so a 250 g block is about two and a quarter sticks; butter sold in sticks is usually four to a 450 g block.".into());
    }
    let said = count.as_ref().map(|c| c.str(1)).unwrap_or("a");
    let amount = if said.starts_with("half") { 0.5 } else { _value(said)? };
    if CUPS.is_match(clean) && !GRAMS_WORD.is_match(clean) {
        return Some("One stick of butter is half a cup, which is 8 tablespoons or about 113 g.".into());
    }
    if amount == 1.0 {
        return Some("A stick of butter is about 113 g, which is half a cup or 8 tablespoons.".into());
    }
    if amount == 0.5 {
        return Some("Half a stick of butter is about 57 g, which is 4 tablespoons.".into());
    }
    Some(format!("{} sticks of butter is about {} g.", _fmt(amount), _fmt(amount * 113.4)))
}

lazy_re!(A_PRICE, r"\b(?:cost|costs|costing|price|prices|cheap|cheaper|expensive)\b");

/// A tin, block or carton: the amount people buy instead of weighing it out.
pub fn _pack(clean: &str) -> Option<String> {
    if !_PACK_WORD.is_match(clean) || !_PACK_ASK.is_match(clean) {
        return None;
    }
    if A_PRICE.is_match(clean) {
        return None; // a price question keeps its price
    }
    let mut best: Option<(&str, Num)> = None;
    for (name, grams) in PACK_GRAMS.iter() {
        if !rx(format!(r"\b{}\b", pyre::escape(name))).is_match(clean) {
            continue;
        }
        if name == "cream" && clean.contains("ice cream") {
            continue;
        }
        if best.is_none_or(|b| text::len(name) > text::len(b.0)) {
            best = Some((name, *grams));
        }
    }
    let (name, grams) = best?;
    let mut container = _PACK_WORD.search(clean).unwrap().as_str();
    if let Some(one) = container.strip_suffix('s') {
        container = one;
    }
    let amount = if PACK_IN_ML.iter().any(|n| n == name) { format!("{} ml", grams.said()) } else { _kilos(grams.f()) };
    let mut reply = format!("A standard {container} of {name} is {amount}");
    if let Some(drained) = num_of(&PACK_DRAINED, name).filter(|d| *d != 0.0) {
        reply += &format!(", about {} g once drained", drained as i64);
    }
    Some(reply + ".")
}

/// The one of a food the weight tables are keyed by: "potatoes" -> "potato".
pub fn _singular(name: &str) -> String {
    for (ending, swap) in [("ies", "y"), ("oes", "o"), ("ches", "ch"), ("shes", "sh"), ("ses", "s"), ("es", ""), ("s", "")] {
        if name.ends_with(ending) && name.len() > ending.len() {
            let gone = format!("{}{swap}", &name[..name.len() - ending.len()]);
            if PIECE_GRAMS.iter().any(|(key, _)| gone == *key || gone == _plural_of(key)) {
                return gone;
            }
        }
    }
    let bare = name.trim_end_matches('s');
    if bare.is_empty() { name.to_string() } else { bare.to_string() }
}

/// The row for this food in a weight table, singular or plural, or None.
fn _table_grams(name: &str, table: &'static [(String, Num)]) -> Option<(&'static str, Num)> {
    let mut best: Option<(&'static str, Num)> = None;
    for (key, grams) in table {
        if (name == key || name == _plural_of(key)) && best.is_none_or(|b| text::len(key) > text::len(b.0)) {
            best = Some((key, *grams));
        }
    }
    best
}

/// How many pieces the packet holds, from the packet's weight and one piece's weight.
pub fn _pack_count(clean: &str) -> Option<String> {
    let found = _COUNT_ASK.search(clean)?;
    let piece = text::squash(found.name_str("piece"));
    let box_ = found.name_str("box");
    if _NOT_A_COUNT.match_(&piece).is_some() {
        return None;
    }
    let counted = |key: &str| COUNTED_PACKS.get(key).filter(|v| !v.is_empty()).cloned();
    if let Some(reply) = counted(&piece).or_else(|| counted(piece.trim_end_matches('s'))) {
        return Some(reply);
    }
    let words = text::split(found.name_str("of"));
    for size in (1..=words.len()).rev() {
        let said = words[..size].join(" ");
        if let Some(reply) = counted(&said).or_else(|| counted(&_singular(&said))) {
            return Some(reply);
        }
    }
    let content_row = _COUNT_CONTENT.get(&piece);
    let (content, name) = content_row.cloned().unwrap_or_else(|| (piece.clone(), piece.clone()));
    let singular = if content_row.is_some() { name.clone() } else { _singular(&name) };
    let mut packed = _table_grams(&content, &PACK_GRAMS);
    let mut one = _table_grams(&singular, &PIECE_GRAMS).or_else(|| _table_grams(&name, &PIECE_GRAMS));
    if (packed.is_none() || one.is_none()) && !words.is_empty() {
        for size in (1..=words.len()).rev() {
            let other = words[..size].join(" ");
            if let Some(row) = _table_grams(&_singular(&other), &PIECE_GRAMS) {
                packed = _table_grams(&_singular(&other), &PACK_GRAMS).or(packed);
                one = Some(row);
                break;
            }
        }
    }
    let (packed, one) = (packed?, one?);
    if !one.1.truthy() {
        return None;
    }
    let count = round_py(packed.1.f() / one.1.f());
    if count < 2 {
        return None;
    }
    let said = _plural_of(one.0);
    Some(format!(
        "About {count} {said} in a {box_}: a {box_} is about {} and one {} about {}g, so the count moves with the size of both.",
        _kilos(packed.1.f()),
        one.0,
        _fmt(one.1.f())
    ))
}

/// "milk in a glass": the pour a vessel holds.
pub fn _in_a_vessel(clean: &str) -> Option<String> {
    let found = _IN_A_VESSEL.search(clean)?;
    let (liquid, vessel) = (found.name_str("liquid"), found.name_str("vessel"));
    let size = found.name_str("size");
    if vessel == "glass" {
        let phrase = format!("glass of {liquid}");
        if let Some((_, reply)) = LOOSE.iter().find(|(p, _)| *p == phrase) {
            return Some(reply.clone());
        }
    }
    let mut amount = num_of(&_VESSEL_ML, vessel).unwrap_or_else(|| panic!("KeyError: {vessel:?}")) as i64;
    if matches!(size, "big" | "large" | "tall") {
        amount = 250;
    }
    let grams = if liquid == "water" && vessel == "glass" { ", which is 200 g" } else { "" };
    Some(format!("A {vessel} of {liquid} is about {amount} ml{grams}."))
}

/// A knob, a squeeze, a shot or a glass: the amount the word stands for.
pub fn _loose(clean: &str) -> Option<String> {
    if !_LOOSE_ASK.is_match(clean) || _LOOSE_NOT.is_match(clean) {
        return None;
    }
    if let Some(in_vessel) = _in_a_vessel(clean).filter(|r| !r.is_empty()) {
        return Some(in_vessel);
    }
    let mut best: Option<&(String, String)> = None;
    for row in LOOSE.iter() {
        if rx(format!(r"\b{}\b", pyre::escape(&row.0))).is_match(clean) && best.is_none_or(|b| text::len(&row.0) > text::len(&b.0)) {
            best = Some(row);
        }
    }
    best.map(|row| row.1.clone())
}

lazy_re!(NOT_A_LIQUID_WEIGHT, r"\b(?:cost|costs|price|prices|worth|sell|buy|buying|calories|protein)\b");
lazy_re!(MUGS, r"\bmugs?\b");

/// What a volume of a named liquid weighs: "a pint of milk", "a litre of water".
pub fn _liquid_weight(clean: &str) -> Option<String> {
    if _PER_CUP.is_match(clean) {
        return None;
    }
    if !_asks_for_weight(clean) {
        return None;
    }
    if NOT_A_LIQUID_WEIGHT.is_match(clean) {
        return None;
    }
    let liquid = LIQUID_GRAMS_LITRE.iter().find(|(name, _)| rx(format!(r"\b{name}\b")).is_match(clean))?;
    let mut measured: Option<(f64, String)> = None;
    for mention in _mentions(clean) {
        if mention.family != "volume" || matches!(mention.unit.as_str(), "cup" | "tbsp" | "tsp") {
            continue; // a cup of a thing is the cup table's question, and it knows the food
        }
        let Some(value) = _number_before(clean, mention.start) else { continue };
        let millilitres = value * _factor("volume", &mention.unit).expect("TypeError: float * None");
        let said = if value == 1.0 {
            format!("a {}", SAID_ONE.get(&mention.unit).unwrap_or(&mention.unit))
        } else {
            _amount_phrase(value, &mention.unit)
        };
        measured = Some((millilitres, said));
        break;
    }
    let (millilitres, said) = match measured {
        Some(found) => found,
        None => {
            if !MUGS.is_match(clean) {
                return None;
            }
            (250.0, "a mug".to_string()) // a mug is 250 ml, and people do ask this
        }
    };
    let grams = millilitres * liquid.1.f() / 1000.0;
    Some(format!("{} of {} weighs {}.", up(&said), liquid.0, _mass_said(grams)))
}

/// A weight said the way someone would: 1 kg, 1.03 kg, 585 g.
pub fn _mass_said(grams: f64) -> String {
    if grams < 1000.0 {
        return format!("{} g", _grams(grams));
    }
    let kilos = grams / 1000.0;
    let said = if kilos < 2.0 { format!("{kilos:.2}") } else { fmt_commas_1(kilos) };
    format!("{} kg", strip_zeros(&said))
}

/// `f"{x:,.1f}"`.
fn fmt_commas_1(x: f64) -> String {
    let s = format!("{x:.1}");
    let (whole, frac) = s.split_once('.').unwrap();
    let (sign, digits) = match whole.strip_prefix('-') {
        Some(d) => ("-", d),
        None => ("", whole),
    };
    let mut out = String::from(sign);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    format!("{out}.{frac}")
}

// ---------------------------------------------------------------------------------------------
// Portions: how much of a food a table of people needs

/// How much of a named dish to cook for the number of people the line names.
pub fn _a_dish_for_a_table(clean: &str) -> Option<String> {
    if clean.is_empty() {
        return None;
    }
    if _A_TIME_IS_THE_ASK.is_match(clean) {
        return None;
    }
    let people = _people_asked(clean).filter(|p| *p != 0)?;
    if !(2..=60).contains(&people) {
        return None;
    }
    let said = text::squash(&text::lower(clean));
    let mut named: Option<(&str, &Dish)> = None;
    for dish in DISHES_FOR_A_TABLE.iter() {
        for name in &dish.names {
            let Some(found) = rx(format!(r"(?<![\w.]){}(?![\w])", pyre::escape(name))).search(&said) else { continue };
            if _A_DISH_PART.match_(&said[found.end()..]).is_some() {
                continue; // a spoonful of an ingredient, not the dish
            }
            if named.is_none_or(|n| text::len(name) > text::len(n.0)) {
                named = Some((name, dish));
            }
        }
    }
    let (name, dish) = named?;
    if matches!(name, "chilli" | "chili") && _CHILLI_THE_VEGETABLE.is_match(&said) {
        return None;
    }
    let mut found: Vec<(Num, &str)> = Vec::new();
    for part in [Some(&dish.main), dish.side.as_ref()].into_iter().flatten() {
        if part.is_empty() {
            continue;
        }
        if let Some(allowance) = _an_allowance(part) {
            found.push(allowance);
        }
    }
    if found.is_empty() {
        return None;
    }
    let each: Vec<String> = found.iter().map(|(amount, noun)| format!("{} g of {noun}", fmt_g(amount.f()))).collect();
    let total: Vec<String> = found.iter().map(|(amount, noun)| format!("{} of {noun}", _a_weight(amount.f() * people as f64))).collect();
    let (per_head, all_of_it) = (each.join(" and "), total.join(" and "));
    let mut reply = format!("For {people}, {name} is about {per_head} each: {all_of_it} in all.");
    if dish.tomatoes {
        let tins = round_py(people as f64 / *_TIN_FOR as f64).max(1);
        let tin = if tins == 1 { "tin" } else { "tins" };
        reply += &format!(" One 400 g tin of tomatoes does four, so {tins} {tin}.");
    }
    if dish.spice {
        reply += " Use a little more spice than the recipe says - a big batch tastes milder.";
    }
    Some(reply)
}

/// `PER_PERSON + PER_PERSON_WHEN_COUNTED`.
fn every_portion() -> impl Iterator<Item = &'static Portion> {
    PER_PERSON.iter().chain(PER_PERSON_WHEN_COUNTED.iter())
}

/// The per-person allowance for one part of a dish, from the one table of allowances.
fn _an_allowance(names: &[String]) -> Option<(Num, &'static str)> {
    for row in every_portion() {
        if row.kind != "g" || !row.amount.truthy() {
            continue;
        }
        if names.iter().any(|name| row.names.contains(name)) {
            return Some((row.amount, &row.noun));
        }
    }
    None
}

/// A weight the way a cook says it: kilos past a thousand, with halves kept.
pub fn _a_weight(grams: f64) -> String {
    if grams >= 1000.0 {
        let kilos = round_py(grams / 100.0) as f64 / 10.0;
        return format!("{} kg", fmt_g(kilos));
    }
    format!("{} g", fmt_g(grams))
}

/// How many people the groups named in the line add up to, or None when it names none.
///
/// Every group counts: "4 adults and 3 kids" is seven, and a line with one group ("for 4
/// people", "6 kids") is that group.
pub fn people_in_groups(text_: &str) -> Option<i64> {
    let mut total = 0;
    for raw in _TABLE_GROUPS.findall_str(&text::lower(text_)) {
        let count = if text::is_digit(&raw) { raw.parse::<i64>().ok() } else { num_of(&_GROUP_VALUE, &raw).map(|v| v as i64) };
        if let Some(count) = count.filter(|c| *c != 0) {
            total += count;
        }
    }
    if total == 0 { None } else { Some(total) }
}

lazy_re!(BETWEEN_US, r"\bbetween\s+us\b");

/// A count word or figure the people regexes caught: `int(raw) if raw.isdigit() else table`.
fn people_value(raw: &str) -> Option<i64> {
    if text::is_digit(raw) { raw.parse().ok() } else { num_of(&_PEOPLE_VALUE, raw).map(|v| v as i64) }
}

/// The first group of a match that took part with something in it.
fn first_group<'t>(m: &Match<'t>) -> Option<&'t str> {
    m.groups().into_iter().flatten().find(|part| !part.is_empty())
}

/// How many people the line cooks for, or None when it only says "a person".
pub fn _people_asked(clean: &str) -> Option<i64> {
    if BETWEEN_US.is_match(clean) {
        return Some(2); // "how much chicken between us" is dinner for two
    }
    let grouped = people_in_groups(clean);
    let Some(found) = _PEOPLE_COUNT.search(clean) else { return grouped };
    let raw = first_group(&found).unwrap_or("");
    let said = people_value(raw);
    if let Some(grouped) = grouped {
        if said.is_none_or(|s| s == 0 || grouped > s) {
            return Some(grouped);
        }
    }
    said
}

/// A headcount from the last turn or two: "we are 6 on saturday", then "how much mince".
///
/// `statements_only` leaves out a count that came inside an earlier portion question. A
/// correction of the count is read as the count even with no word for the people in it, and a
/// count said as a statement about the table stands for a few turns when the line asks how much
/// (`asked_for_an_amount`).
pub fn _people_before(memory: Option<&Memory>, statements_only: bool, asked_for_an_amount: bool) -> Option<i64> {
    let memory = memory?;
    let lines = last_turns(Some(memory), 2);
    let earlier = last_turns(Some(memory), 6);
    let said: Vec<String> = lines
        .iter()
        .map(|line| text::squash(&text::lower(line)))
        .filter(|line| !line.is_empty() && !(statements_only && _AN_ASK.is_match(line)))
        .collect();
    for line in said.iter().rev() {
        if !(_PEOPLE_WORD.is_match(line) || _PEOPLE_COUNT.is_match(line)) && !CORRECTING.is_match(line) {
            continue;
        }
        if !_a_table_without_people(line) {
            continue;
        }
        let mut found = _people_asked(line).filter(|f| *f != 0);
        if found.is_none() && CORRECTING.is_match(line) {
            found = _a_corrected_figure(line).filter(|f| *f != 0);
        }
        if found.is_some() {
            return found;
        }
    }
    if statements_only || !asked_for_an_amount {
        return None;
    }
    let mut older: Vec<String> = earlier.iter().map(|line| text::squash(&text::lower(line))).collect();
    older.truncate(older.len().saturating_sub(2));
    _a_count_that_still_stands(&older)
}

/// The newest headcount in the turns before the last two, while it still holds.
pub fn _a_count_that_still_stands(older: &[String]) -> Option<i64> {
    for line in older.iter().rev() {
        if _AN_ASK.is_match(line) {
            continue;
        }
        if _PEOPLE_WORD.is_match(line) || _PEOPLE_COUNT.is_match(line) {
            return _people_asked(line).filter(|f| *f != 0);
        }
        if CORRECTING.is_match(line) {
            let found = _people_asked(line).filter(|f| *f != 0).or_else(|| _a_corrected_figure(line)).filter(|f| *f != 0);
            if found.is_some() {
                return found;
            }
        }
    }
    None
}

/// A bare figure in a line that corrects something: "so its eleven now".
pub fn _a_corrected_figure(text_: &str) -> Option<i64> {
    if _A_CLOCK.is_match(text_) {
        return None;
    }
    let found = _A_FIGURE.search(text_)?;
    people_value(found.str(1))
}

/// True when a line with no word for people still says how many it cooks for.
pub fn _a_table_without_people(text_: &str) -> bool {
    if _PEOPLE_WORD.is_match(text_) || CORRECTING.is_match(text_) {
        return true;
    }
    let Some(found) = _BARE_FOR_COUNT.search(text_) else { return false };
    let before = text::split_ref(&text_[..found.start()]);
    let Some(last) = before.last() else { return true }; // "im cooking for 8" names no dish, but it is the plan
    !_NOT_A_TABLE_BEFORE.contains(&text::lower(text::strip_chars(last, ".,!?;:")))
}

lazy_re!(PORTION_ASK, r"\bhow (?:much|many|big|large)\b|\bwhat size\b|\bi (?:need|want)\b|\bi(?:'?m| am) (?:cooking|making)\b");
lazy_re!(A_TIME_UNIT, r"\b(?:minutes?|mins?|hours?|hrs?|seconds?)\b");
lazy_re!(I_NEED_A, r"\bi (?:need|want)\s+(?:a|an|one|some)\b");
lazy_re!(EGG, r"\begg");
lazy_re!(HOW_MUCH_OR_MANY, r"\bhow (?:much|many)\b");
lazy_re!(I_AM_MAKING, r"\bi (?:need|want)\s+to\b|\bi(?:'?m| am) (?:cooking|making|baking|roasting|prepping)\b");
lazy_re!(HOW_BIG_ASK, r"\bhow (?:much|many|big|large)\b|\bwhat size\b");
lazy_re!(HOW_BIG, r"\bhow (?:big|large)\b|\bwhat size\b");

/// How much of a food a person gets, when the line asks to cook for a number of people.
///
/// The reply gives the portion for one, and the total only for a headcount the person actually
/// said: a number they never gave is the one thing a portion must not do.
pub fn _per_person(clean: &str, memory: Option<&Memory>) -> Option<String> {
    if !PORTION_ASK.is_match(clean) {
        return None;
    }
    if A_TIME_UNIT.is_match(clean) {
        return None;
    }
    if I_NEED_A.is_match(clean) {
        return None;
    }
    if _COMPONENT.is_match(clean) || _PORTION_AVOID.is_match(clean) {
        return None;
    }
    if _BAKING_PORTION.is_match(clean) && EGG.is_match(clean) {
        return None;
    }
    let mut people = _people_asked(clean);
    let stated;
    if people.is_none() {
        people = _people_before(memory, false, HOW_MUCH_OR_MANY.is_match(clean));
        stated = people.is_some() && _people_before(memory, true, false) == people;
    } else {
        stated = true;
    }
    let people_word = _PEOPLE_WORD.is_match(clean);
    let portion_word = _PORTION_WORD.is_match(clean);
    if people.is_none() && !people_word && !portion_word {
        for phrase in _WHOLE_PIECE.iter() {
            if rx(format!(r"\b{}\b", pyre::escape(phrase))).is_match(clean) {
                return None;
            }
        }
    }
    if !people_word && people.is_none() && !portion_word {
        return None;
    }
    if _PORTION_COUNT.is_match(clean) && people.is_none() && !people_word {
        return None;
    }
    let mut best: Option<(&str, &Portion)> = None;
    let counted: &[Portion] = if stated && !_CONTAINER.is_match(clean) { &PER_PERSON_WHEN_COUNTED } else { &[] };
    for row in PER_PERSON.iter().chain(counted.iter()) {
        for name in &row.names {
            let pattern = format!(r"(?<![\w.]){}(?![\w])(?!\s+(?:dressing|sauce|dip|oil|mayo|mayonnaise|vinaigrette|juice))", pyre::escape(name));
            if rx(pattern).is_match(clean) && best.is_none_or(|b| text::len(name) > text::len(b.0)) {
                best = Some((name, row));
            }
        }
    }
    let Some((name, row)) = best else {
        if let Some(people) = people {
            if stated && portion_word {
                return Some(format!(
                    "A main is about 300 g a head, a plate of pasta or rice 75 to 100 g dry and a buffet six to eight pieces a guest, so for {} plan on {} of the main dish. Tell me what you are cooking and I will do the sum for the shopping.",
                    _said_people(people),
                    _mass_of((300 * people) as f64)
                ));
            }
        }
        return None;
    };
    let (mut amount, kind, mut noun) = (row.amount, row.kind.as_str(), row.noun.as_str());
    if MADE_NOT_MEASURED.contains(noun) && I_AM_MAKING.is_match(clean) && !HOW_BIG_ASK.is_match(clean) {
        return None;
    }
    if _BIRDS.iter().any(|b| b == name) && HOW_BIG.is_match(clean) {
        amount = Num::I(*BIRD_GRAMS);
        noun = name;
    }
    // `if people:` - a count of nought is no count
    let people = people.filter(|p| *p != 0);
    if kind == "each" {
        let bunch = if noun.ends_with('s') || noun.contains(' ') { noun.to_string() } else { format!("{noun}s") };
        let said = if amount.f() > 1.0 { format!("{} {bunch}", amount.said()) } else { format!("1 {noun}") };
        if let Some(people) = people {
            let total = if amount.f() > 1.0 {
                format!("{} {bunch}", fmt_g(amount.f() * people as f64))
            } else {
                format!("{} {bunch}", _SAID_PEOPLE.get(&people).cloned().unwrap_or_else(|| people.to_string()))
            };
            return Some(format!("About {said} each, so {total} for {}.", _said_people(people)));
        }
        if !people_word {
            return Some(format!("A portion is about {said}."));
        }
        return Some(format!("About {said} each."));
    }
    if kind == "bottle" {
        let per = if amount.f() == 0.5 { "half a bottle".to_string() } else { format!("{} bottles", fmt_g(amount.f())) };
        let Some(people) = people else { return Some(format!("Allow about {per} of {noun} a guest.")) };
        let bottles = (amount.f() * people as f64).ceil() as i64;
        return Some(format!(
            "Allow about {per} of {noun} a guest, so {bottles} {} for {}.",
            if bottles == 1 { "bottle" } else { "bottles" },
            _said_people(people)
        ));
    }
    if kind == "ml" {
        if let Some(people) = people {
            return Some(format!("About {} ml of {noun} each, so {} for {}.", amount.said(), _litres_of(amount.times(people)), _said_people(people)));
        }
        if !people_word {
            return Some(format!("A portion of {noun} is about {} ml.", amount.said()));
        }
        return Some(format!("About {} ml of {noun} each.", amount.said()));
    }
    if let Some(people) = people {
        return Some(format!("About {} g of {noun} each, so {} for {}.", _grams(amount.f()), _mass_of(amount.f() * people as f64), _said_people(people)));
    }
    if !people_word {
        return Some(format!("A portion of {noun} is about {} g.", _grams(amount.f())));
    }
    Some(format!("About {} g of {noun} each.", _grams(amount.f())))
}

/// A headcount the way it is said: "for one", "for eight", never "for 1".
pub fn _said_people(people: i64) -> String {
    _SAID_PEOPLE.get(&people).cloned().unwrap_or_else(|| people.to_string())
}

/// A scaled-up portion the way a shopper reads it: 1 kg, not 1,000 g.
pub fn _mass_of(grams: f64) -> String {
    if grams >= 1000.0 {
        return format!("{} kg", strip_zeros(&format!("{:.1}", grams / 1000.0)));
    }
    format!("{} g", _grams(grams))
}

/// A scaled-up pour the way a jug reads it: 2.4 litres, not 2,400 ml.
pub fn _litres_of(ml: Num) -> String {
    if ml.f() >= 1000.0 {
        return format!("{} litres", strip_zeros(&format!("{:.1}", ml.f() / 1000.0)));
    }
    format!("{} ml", ml.said())
}

// ---------------------------------------------------------------------------------------------
// Land and heights

/// A count of a picture, said the way a person says it: "two", "two and a half".
pub fn _count_said(ratio: f64) -> String {
    let whole = ratio.trunc() as i64;
    let part = ratio - whole as f64;
    if part < 0.25 {
        return whole.to_string();
    }
    if part < 0.75 {
        return if whole != 0 { format!("{whole} and a half") } else { "half".to_string() };
    }
    (whole + 1).to_string()
}

/// The nearest thing a person has stood on, said as a count: "about two tennis courts".
pub fn _area_pictures(square_metres: f64) -> String {
    let last_size = _AREA_PICTURES[_AREA_PICTURES.len() - 1].2;
    let mut picked = &_AREA_PICTURES[0];
    let mut ratio = 0.0;
    for row in _AREA_PICTURES.iter() {
        picked = row;
        ratio = square_metres / row.2;
        if ratio < 5.0 || row.2 == last_size {
            break;
        }
    }
    let (name, plural) = (&picked.0, &picked.1);
    if ratio < 0.4 {
        return format!("less than half a {name}");
    }
    if ratio < 0.75 {
        return format!("about two thirds of a {name}");
    }
    if ratio < 1.15 {
        return format!("about a {name}");
    }
    if ratio < 1.85 {
        return format!("a bit more than a {name}");
    }
    format!("about {} {plural}", _count_said(ratio))
}

lazy_re!(SHARE_FIGURE, r"(\d+(?:\.\d+)?)\s*(?:of\s+)?(?:an?\s+)?$");

/// The count or share of a land measure said before it: 0.15, half, a quarter, 2.
pub fn _share_before(clean: &str, start: usize) -> Option<f64> {
    let head = &clean[..start];
    if let Some(found) = SHARE_FIGURE.search(head) {
        return Some(found.str(1).parse().expect("float()"));
    }
    for (word, value) in _WORD_SHARE.iter() {
        if rx(format!(r"\b{}\s*(?:of\s+)?(?:an?\s+)?$", pyre::escape(word))).is_match(head) {
            return Some(*value);
        }
    }
    None
}

/// A number of acres asked *whether it is big*: the size, then the nearest picture of it.
pub fn _land_in_pictures(value: f64, unit: &str, clean: &str) -> Option<String> {
    let size = family_units("area").iter().find(|u| u.name == unit).map(|u| u.factor).unwrap_or_else(|| panic!("KeyError: {unit:?}"));
    let square_metres = value * size;
    if square_metres < 1.0 {
        return None;
    }
    let said = if (value - 1.0).abs() < 1e-9 {
        format!("One {}", SAID_ONE[unit])
    } else if (value - round_py(value) as f64).abs() < 1e-9 {
        format!("{} {}", round_py(value), SAID[unit])
    } else {
        format!("{} {}", _fmt(value), SAID[unit])
    };
    let mut body = format!("{said} is about {} square metres - {}", _fmt(round_py(square_metres) as f64), _area_pictures(square_metres));
    if square_metres >= 400.0 && clean.contains("garden") {
        body += ", which is a generous garden for a house";
    }
    if (value - 1.0).abs() < 1e-9 {
        return Some(body + "."); // one acre is the definition itself: the tail would repeat it
    }
    Some(format!("{body}. {}", LAND_PLAIN[unit]))
}

lazy_re!(ENDS_IN_A_DIGIT, r"\d\s*$");
lazy_re!(ENDS_IN_A_COUNT, r"\b(?:two|three|four|five|ten|twenty)\s+$");

/// "how big is an acre": one land measure named, with no number and no second unit.
pub fn _land_size(clean: &str, mentions: &[Mention]) -> Option<String> {
    let [mention] = mentions else { return None };
    let (start, end, unit) = (mention.start, mention.end, mention.unit.as_str());
    if mention.family != "area" || !LAND_PLAIN.contains_key(unit) {
        return None;
    }
    if _A_SIZE_JUDGEMENT.is_match(clean) {
        if let Some(counted) = _share_before(clean, start) {
            if let Some(pictured) = _land_in_pictures(counted, unit, clean).filter(|p| !p.is_empty()) {
                return Some(pictured);
            }
        }
    }
    let before = back(clean, start, 14);
    if !_LAND_ASK.is_match(clean) {
        return None;
    }
    if ENDS_IN_A_DIGIT.is_match(before) || ENDS_IN_A_COUNT.is_match(before) {
        return None;
    }
    if _UNIT_WORD.is_match(&clean[end..]) || _TARGET.is_match(text::strip(&clean[..start])) {
        return None;
    }
    Some(LAND_PLAIN[unit].clone())
}

lazy_re!(FEET_AND_INCHES, "(\\d)\\s*(?:'|\u{2019}|ft|foot|feet)\\s*(\\d{1,2})\\s*(?:\"|''|in|inch|inches)?");
lazy_re!(FEET_THEN_INCHES, r"(\d)\s*(?:ft|foot|feet)\s+(\d{1,2})\b");

/// A height typed the way people type it: 5'6, 5ft 6, or 1.8m asked for in feet and inches.
pub fn _height(clean: &str, mentions: &[Mention]) -> Option<String> {
    if !mentions.iter().any(|m| m.unit == "cm" || m.unit == "m") {
        return None;
    }
    let Some(m) = FEET_AND_INCHES.search(clean).or_else(|| FEET_THEN_INCHES.search(clean)) else {
        return _metres_in_feet(clean);
    };
    let (feet, inches): (i64, i64) = (m.str(1).parse().expect("int()"), m.str(2).parse().expect("int()"));
    if !(2..=8).contains(&feet) || inches > 11 {
        return None;
    }
    let total = feet * 12 + inches;
    Some(format!("{feet} feet {inches} is about {} cm, or {:.2} m.", round_py(total as f64 * 2.54), total as f64 * 0.0254))
}

lazy_re!(TALL, r"\btall\b", pyre::I);

/// True when the number is how far or how long, not how tall.
pub fn _is_a_distance(clean: &str) -> bool {
    _NOT_A_HEIGHT.is_match(clean) && !TALL.is_match(clean)
}

/// True when a stated height comes with a question about something that is not a measure.
pub fn _asks_something_else(clean: &str) -> bool {
    if _A_COMPLAINT.is_match(clean) {
        return true;
    }
    for m in _ASKS_HOW_MANY.finditer(clean) {
        let thing = text::lower(m.name_str("thing"));
        if _UNIT_WORD.fullmatch(&thing).is_none() && !_A_MEASURE_SAID_ANOTHER_WAY.contains(&thing) {
            return true;
        }
    }
    for m in _ASKS_ELSEWHERE.finditer(clean) {
        let tail = text::split_ref(&clean[m.end()..]).into_iter().take(5).collect::<Vec<_>>().join(" ");
        if !_A_MEASURE_QUESTION.is_match(&tail) {
            return true;
        }
    }
    false
}

/// A height in metres as feet and inches: "1.75 m is about 5 foot 9 (5.7 feet)."
fn metres_said(metres: f64, said: &str) -> String {
    let total = metres / 0.0254;
    let mut feet = (total / 12.0).floor() as i64;
    let mut inches = round_py(total - (feet * 12) as f64);
    if inches == 12 {
        feet += 1;
        inches = 0;
    }
    format!("{said} is about {feet} foot {inches} ({:.1} feet).", total / 12.0)
}

/// The other number for a height the person states about themselves, or None.
pub fn _own_height(clean: &str) -> Option<String> {
    if !_SAYS_OWN_HEIGHT.is_match(clean) {
        return None;
    }
    if _is_a_distance(clean) {
        return None;
    }
    if _asks_something_else(clean) {
        return None;
    }
    let mut metres: Option<(f64, String)> = None;
    if let Some(found) = _HEIGHT_METRES.search(clean) {
        metres = Some((found.str(1).parse().expect("float()"), format!("{} m", found.str(1))));
    }
    if metres.is_none() {
        if let Some(found) = _HEIGHT_CM.search(clean) {
            metres = Some((found.str(1).parse::<f64>().expect("float()") / 100.0, format!("{} cm", found.str(1))));
        }
    }
    if let Some((metres, said)) = metres {
        if !(1.2..=2.2).contains(&metres) {
            return None;
        }
        return Some(metres_said(metres, &said));
    }
    let m = _HEIGHT_FEET.search(clean).or_else(|| _HEIGHT_FEET_WORD.search(clean))?;
    let (feet, inches): (i64, i64) = (m.str(1).parse().expect("int()"), m.str(2).parse().expect("int()"));
    if inches > 11 {
        return None;
    }
    let cm = round_py((feet * 12 + inches) as f64 * 2.54);
    Some(format!("{feet} foot {inches} is about {cm} cm, or {:.2} m.", cm as f64 / 100.0))
}

lazy_re!(METRES_SAID, r"(?<![\w.])(\d(?:\.\d{1,2})?)\s*(?:m|metres?|meters?)(?!\w)");
lazy_re!(CM_SAID, r"(?<![\w.])(\d{2,3}(?:\.\d)?)\s*(?:cm|centimetres?|centimeters?)(?!\w)");

/// A height given in metres or centimetres, asked for in feet and inches.
pub fn _metres_in_feet(clean: &str) -> Option<String> {
    if !_ASKS_FEET.is_match(clean) {
        return None;
    }
    let (metres, said) = if let Some(found) = METRES_SAID.search(clean) {
        (found.str(1).parse::<f64>().expect("float()"), format!("{} m", found.str(1)))
    } else {
        let found = CM_SAID.search(clean)?;
        (found.str(1).parse::<f64>().expect("float()") / 100.0, format!("{} cm", found.str(1)))
    };
    if !(1.2..=2.2).contains(&metres) {
        return None;
    }
    Some(metres_said(metres, &said))
}

// ---------------------------------------------------------------------------------------------
// Cups, spoons and handfuls of a named food

lazy_re!(A_WEIGHT_UNIT, r"\b(grams?|g|oz|ounces?|kg)\b");

/// Grams in a cup, mug, tablespoon or teaspoon of a dry thing, asked about by name.
pub fn _cup_of_something(clean: &str) -> Option<String> {
    if _PER_CUP.is_match(clean) {
        return None; // "grams of coffee per cup" is a dose, and the coffee entry knows it
    }
    let (measure, share) = if MUGS.is_match(clean) {
        ("mug", 240.0 / 250.0)
    } else if _names_unit(clean, "tbsp") {
        ("tablespoon", 16.0)
    } else if _names_unit(clean, "tsp") {
        ("teaspoon", 48.0)
    } else if _names_unit(clean, "dessertspoon") {
        ("dessertspoon", 24.0)
    } else {
        ("cup", 1.0)
    };
    if matches!(measure, "tablespoon" | "teaspoon" | "dessertspoon") {
        // `max(spoon, key=len)`: the first of the longest names
        let mut spoon: Option<&(String, Num)> = None;
        for row in GRAMS_PER_TSP.iter() {
            if rx(format!(r"\b{}\b", pyre::escape(&row.0))).is_match(clean) && spoon.is_none_or(|s| text::len(&row.0) > text::len(&s.0)) {
                spoon = Some(row);
            }
        }
        if let Some((food, grams)) = spoon {
            let amount = grams.f() * (240.0 / share) / 5.0;
            return Some(format!("A {measure} of {food} is about {} g.", _fmt(amount)));
        }
    }
    if let Some((food, grams)) = _food_named(clean) {
        if _asks_for_weight(clean) {
            let amount = if share == 1.0 { grams.f() } else { round_py(grams.f() / share).max(1) as f64 };
            let said = AS_KITCHEN_SAYS.get(food).map(String::as_str).unwrap_or(food);
            return Some(format!("A {measure} of {said} is about {} g.", _grams(amount)));
        }
    }
    if _WEIGH.is_match(clean) || A_WEIGHT_UNIT.is_match(clean) {
        return Some(format!(
            "It depends what is in a {measure}: flour is about 120 g, sugar about 200 g, butter 227 g. By volume a cup is 240 ml, which is 8 fluid ounces."
        ));
    }
    None
}

lazy_re!(HANDFULS, r"\bhandfuls?\b");
lazy_re!(HOW_MUCH, r"\bhow much\b");
lazy_re!(A_PRICE_WORD, r"\b(?:cost|costs|costing|price|prices|priced|cheap|cheaper|expensive)\b");
lazy_re!(HANDFUL_OF, r"\bhandfuls?\s+(?:of\s+)?(?!of\b|it\b|them\b|these\b|those\b|that\b)([a-z]+(?:\s+[a-z]+)?)");

/// A handful, and how many grams of the food in it: a quarter of a cup of whatever it is.
pub fn _handful(clean: &str) -> Option<String> {
    if !HANDFULS.is_match(clean) {
        return None;
    }
    if !(_asks_for_weight(clean) || HOW_MUCH.is_match(clean)) {
        return None;
    }
    if A_PRICE_WORD.is_match(clean) {
        return None;
    }
    if let Some((name, grams_per_cup)) = _food_named(clean) {
        let said = AS_KITCHEN_SAYS.get(name).map(String::as_str).unwrap_or(name);
        return Some(format!(
            "A handful of {said} is about {} g, since a handful is roughly a quarter of a cup.",
            _grams(round_py(grams_per_cup.f() / 4.0) as f64)
        ));
    }
    let mut stuff = String::new();
    if let Some(said_food) = HANDFUL_OF.search(clean) {
        stuff = text::split_ref(said_food.str(1)).into_iter().filter(|w| !_NOT_A_FOOD.contains(*w)).collect::<Vec<_>>().join(" ");
    }
    if !stuff.is_empty() {
        return Some(format!(
            "A handful of {stuff} is about a quarter of a cup, but the weight depends on what is in it: 35 g of nuts against 8 g of spinach."
        ));
    }
    Some("A handful is about a quarter of a cup, which comes to roughly 35 g of nuts, 30 g of raisins or 25 g of crisps.".into())
}

lazy_re!(A_WEIGHT_WORD, r"\b(?:grams?|g|kg|kilograms?|oz|ounces?|pounds?|lbs?)\b");

/// True when the line wants a weight: "how many grams", "what does a cup weigh", "in oz".
/// A figure in pounds that the line is judging the price of is money, not a weight.
pub fn _asks_for_weight(clean: &str) -> bool {
    if _POUNDS_IS_MONEY.is_match(clean) {
        return false;
    }
    if _WEIGH.is_match(clean) {
        return true;
    }
    A_WEIGHT_WORD.is_match(clean)
}

/// True when the line names this unit, in any of its words.
pub fn _names_unit(clean: &str, unit: &str) -> bool {
    UNITS.iter().any(|(_, items)| {
        items.iter().any(|u| u.name == unit && u.words.iter().any(|word| rx(format!(r"\b{}\b", pyre::escape(word))).is_match(clean)))
    })
}

/// A recipe's fractions as decimals, so "3/4 cup" reads as three quarters of a cup.
pub fn _frac(clean: &str) -> String {
    let whole_said = |count: &str| -> f64 {
        // `_SAID_WHOLE.get(count) or int(count)` for a figure, the table for a word
        match num_of(&_SAID_WHOLE, count).filter(|v| *v != 0.0) {
            Some(v) => v,
            None if text::is_digit(count) => count.parse::<f64>().expect("int()"),
            None => panic!("KeyError: {count:?}"),
        }
    };
    let fraction = |a: &str, b: &str| _FRACTION.iter().find(|((x, y), _)| x == a && y == b).map(|(_, v)| *v).filter(|v| *v != 0.0);
    let clean = _UNIT_AND_A_HALF.sub_fn(clean, |m| {
        let count = m.name("count").filter(|c| !c.is_empty()).unwrap_or("one");
        format!("{} {}", fmt_g(whole_said(count) + 0.5), m.name_str("unit"))
    });
    let clean = _MIXED.sub_fn(&clean, |m| match fraction(m.str(2), m.str(3)) {
        Some(value) => fmt_g(m.str(1).parse::<f64>().expect("int()") + value),
        None => m.as_str().to_string(),
    });
    let mut clean = _AND_A_HALF.sub_fn(&clean, |m| fmt_g(whole_said(m.str(1)) + 0.5));
    for (word_fraction, value) in _WORD_FRACTION.iter() {
        clean = word_fraction.sub(value, &clean);
    }
    _SLASH.sub_fn(&clean, |m| match fraction(m.str(1), m.str(2)) {
        Some(value) => strip_zeros(&format!("{value:.4}")).to_string(),
        None => m.as_str().to_string(),
    })
}

/// The rule for a conversion asked as a rule: no numbers, so the factor and one example.
pub fn _rule_sentence(family: &str, from_unit: &str, to_unit: &str) -> Option<String> {
    let source = _factor(family, from_unit).filter(|f| *f != 0.0)?;
    let target = _factor(family, to_unit).filter(|f| *f != 0.0)?;
    if (source - target).abs() < 1e-9 {
        return None;
    }
    let factor = source / target;
    let said_from = SAID.get(from_unit).map(String::as_str).unwrap_or(from_unit);
    let said_to = SAID.get(to_unit).map(String::as_str).unwrap_or(to_unit);
    let rule = if factor >= 1.0 { format!("multiply the {said_from} by {}", _fmt(factor)) } else { format!("divide the {said_from} by {}", _fmt(1.0 / factor)) };
    let mut amount = 1.0;
    for candidate in [1.0, 10.0, 100.0, 1000.0, 10000.0] {
        amount = candidate;
        if candidate * factor >= 1.0 {
            break;
        }
    }
    let example = format!(
        "{} {} is {} {}",
        _fmt(amount),
        _say(amount, from_unit, Some(amount == 1.0)),
        _fmt(amount * factor),
        _say(amount * factor, to_unit, None)
    );
    Some(format!("To go from {said_from} to {said_to}, {rule}: {example}."))
}

lazy_re!(A_FIGURE_IN_THE_LINE, r"\d+(?:\.\d+)?");
lazy_re!(LETTERS, r"[a-z]+");

/// True when the line is nothing but two units and the word between them: "km to miles".
pub fn _is_bare_pair(clean: &str, mentions: &[Mention]) -> bool {
    if mentions.len() < 2 {
        return false;
    }
    let mut rest = clean.to_string();
    for mention in mentions.iter().rev() {
        rest = format!("{} {}", &rest[..mention.start], &rest[mention.end..]);
    }
    let rest = A_FIGURE_IN_THE_LINE.sub(" ", &rest);
    let words = LETTERS.findall_str(&rest);
    !words.is_empty() && words.iter().all(|word| _BARE_JOIN.contains(word))
}

/// Both sides of a conversion, with a bare ounce read in the other side's family.
pub fn _the_ounce_pair(family: &str, unit: &str, other_family: &str, other_unit: &str) -> (String, String, String, String) {
    let s = |a: &str, b: &str, c: &str, d: &str| (a.to_string(), b.to_string(), c.to_string(), d.to_string());
    if unit == "ounce" && other_family == "volume" {
        return s("volume", "fl oz", other_family, other_unit);
    }
    if other_unit == "ounce" && family == "volume" {
        return s(family, unit, "volume", "fl oz");
    }
    if unit == "fl oz" && other_family == "mass" {
        return s("mass", "ounce", other_family, other_unit);
    }
    if other_unit == "fl oz" && family == "mass" {
        return s(family, unit, "mass", "ounce");
    }
    s(family, unit, other_family, other_unit)
}

/// "how do i convert km to miles" names the two units and no number: give the rule.
pub fn _how_to_convert(clean: &str, mentions: &[Mention]) -> Option<String> {
    if !_HOW_TO.is_match(clean) && !_is_bare_pair(clean, mentions) {
        return None;
    }
    if _NUMBER.is_match(clean) || mentions.iter().any(|m| _number_before(clean, m.start).is_some()) {
        return None;
    }
    for (position, mention) in mentions.iter().enumerate() {
        // Python rebinds `family` and `unit` to the ounce reading inside the inner loop, and
        // the rebinding carries on to the next pair.
        let (mut family, mut unit) = (mention.family.clone(), mention.unit.clone());
        for other in &mentions[position + 1..] {
            let (f, u, other_family, other_unit) = _the_ounce_pair(&family, &unit, &other.family, &other.unit);
            family = f;
            unit = u;
            if other_family != family || other_unit == unit {
                continue;
            }
            return _rule_sentence(&family, &unit, &other_unit);
        }
    }
    None
}

/// True when the measure is what the line asks about, rather than the object of a plan.
pub fn _tail_is_about_the_measure(tail: &str) -> bool {
    !_ASKING_AFTER_MEASURE.is_match(&text::lower(tail))
}

// ---------------------------------------------------------------------------------------------
// A fraction of a cup or a spoon

lazy_re!(A_SLASH, r"(\d)\s*/\s*(\d)");

/// The number a fraction word is, or None when it is not a fraction this table knows.
pub fn _share_value(word: &str) -> Option<f64> {
    if let Some(value) = num_of(&_SHARE_VALUE, word) {
        return Some(value);
    }
    let slash = A_SLASH.fullmatch(word)?;
    let (top, bottom): (f64, f64) = (slash.str(1).parse().unwrap(), slash.str(2).parse().unwrap());
    if bottom == 0.0 {
        return None;
    }
    Some(top / bottom)
}

/// The name a cook would give this fraction, for the lines that write it in digits.
pub fn _share_name(value: f64) -> Option<&'static str> {
    for (known, name) in [
        (0.25, "a quarter"),
        (0.5, "half"),
        (0.75, "three quarters"),
        (1.0 / 3.0, "a third"),
        (2.0 / 3.0, "two thirds"),
        (0.2, "a fifth"),
        (1.0 / 6.0, "a sixth"),
        (1.0 / 8.0, "an eighth"),
        (0.125, "an eighth"),
    ] {
        if (value - known).abs() < 0.001 {
            return Some(name);
        }
    }
    None
}

/// The spoons a fraction of a unit is: whole ones, then the teaspoons left over.
pub fn _share_spoons(value: f64, per: f64) -> String {
    let spoons = value * per;
    let mut whole = spoons.trunc() as i64;
    let mut rest = round_py((spoons - whole as f64) * 3.0);
    if rest >= 3 {
        whole += 1;
        rest = 0;
    }
    if whole == 0 {
        return if rest == 1 { "a teaspoon".to_string() } else { format!("{rest} teaspoons") };
    }
    let said = if whole == 1 { format!("{whole} tablespoon") } else { format!("{whole} tablespoons") };
    if rest == 0 {
        return said;
    }
    said + &if rest == 1 { " and a teaspoon".to_string() } else { format!(" and {rest} teaspoons") }
}

lazy_re!(ENDS_IN_OF, r"\bof\s*$");
lazy_re!(A_CUP_OR_SPOON, r"\b(?:cups?|tablespoons?|tbsp|teaspoons?|tsp)\b");

/// A fraction of a cup, a tablespoon or a teaspoon, answered in the smaller units.
pub fn _share_of_a_measure(clean: &str, tail: bool) -> Option<String> {
    if !_SHARE_ASK.is_match(clean) || _NOT_ABOUT_A_MEASURE.is_match(clean) {
        return None;
    }
    let m = _SHARE.search(clean)?;
    let word = text::squash(m.name_str("word"));
    let mut value = _share_value(&word).filter(|v| *v != 0.0)?;
    let mut said = _share_name(value).map(str::to_string).unwrap_or_else(|| word.clone());
    let head = &clean[..m.start()];
    let before = if ENDS_IN_OF.is_match(head) { _SHARE_FRACTION.findall_str(head) } else { Vec::new() };
    if let Some(last) = before.last() {
        let outer_word = text::squash(last);
        if let Some(outer) = _share_value(&outer_word).filter(|v| *v != 0.0) {
            value *= outer;
            said = format!("{} of {said}", _share_name(outer).unwrap_or(&outer_word));
        }
    }
    let unit = m.name_str("unit");
    let rest = format!("{} {}", head, &clean[m.end()..]);
    if A_CUP_OR_SPOON.is_match(&rest) {
        return None;
    }
    if unit.starts_with("cup") {
        let millilitres = round_py(value * 240.0);
        let lead = if said == "half" { "Half a cup".to_string() } else { format!("{} of a cup", up(&said)) };
        let advice = if tail { ": spoons are the easy way to cut a cup down." } else { "." };
        return Some(format!("{lead} is {}, about {millilitres} ml{advice}", _share_spoons(value, 16.0)));
    }
    let said = up(&said);
    if unit.starts_with("tablespoon") || unit.starts_with("tbsp") {
        let millilitres = round_py(value * 15.0);
        return Some(format!("{said} of a tablespoon is {}, about {millilitres} ml.", _tsp_phrase(value * 3.0)));
    }
    let millilitres = round_py(value * 5.0);
    Some(format!("{said} of a teaspoon is about {millilitres} ml."))
}

/// A count of teaspoons, rounded the way a cook says it.
pub fn _tsp_phrase(count: f64) -> String {
    if (count - round_py(count) as f64).abs() < 0.05 {
        let whole = round_py(count);
        return if whole != 1 { format!("{whole} teaspoons") } else { "a teaspoon".to_string() };
    }
    format!("about {} teaspoons", text::float_repr(round_to(count, 1)))
}

// ---------------------------------------------------------------------------------------------
// A span of time and a fraction of one

lazy_re!(A_SLASH_OF_ANY, r"(\d+)\s*/\s*(\d+)");
lazy_re!(A_DECIMAL_SHARE, r"\d*\.\d+");

/// What a fraction word is worth: "a quarter" 0.25, "three quarters" 0.75, ".75" 0.75.
pub fn _a_fraction_value(word: &str) -> Option<f64> {
    let word = text::lower(&text::squash(&word.replace('-', " ")));
    let below_one = |value: f64| if value < 1.0 { Some(value) } else { None };
    if let Some(slash) = A_SLASH_OF_ANY.fullmatch(&word.replace(' ', "")) {
        let (top, bottom): (f64, f64) = (slash.str(1).parse().expect("int()"), slash.str(2).parse().expect("int()"));
        if bottom != 0.0 {
            return below_one(top / bottom);
        }
    }
    if A_DECIMAL_SHARE.fullmatch(&word).is_some() {
        return below_one(word.parse().expect("float()"));
    }
    let parts = text::split_ref(&word);
    if parts == ["half"] {
        return Some(0.5);
    }
    if parts.len() == 2 {
        let mut numeral = num_of(&_NUMERAL, parts[0]);
        if numeral.is_none() && text::is_digit(parts[0]) {
            numeral = parts[0].parse().ok();
        }
        let denominator = num_of(&_DENOMINATOR, parts[1]);
        if let (Some(numeral), Some(denominator)) = (numeral.filter(|n| *n != 0.0), denominator.filter(|d| *d != 0.0)) {
            return below_one(numeral / denominator);
        }
    }
    None
}

/// The name a person gives a fraction: 1/3 is "a third", 3/4 "three quarters".
pub fn _a_fraction_name(value: f64) -> Option<&'static str> {
    if let Some(name) = _share_name(value) {
        return Some(name);
    }
    for (numerator, denominator, said) in [
        (1.0, 7.0, "a seventh"),
        (1.0, 9.0, "a ninth"),
        (2.0, 7.0, "two sevenths"),
        (1.0, 10.0, "a tenth"),
        (1.0, 12.0, "a twelfth"),
        (1.0, 24.0, "a twenty-fourth"),
        (5.0, 6.0, "five sixths"),
        (5.0, 8.0, "five eighths"),
        (3.0, 8.0, "three eighths"),
        (7.0, 8.0, "seven eighths"),
        (1.0, 20.0, "a twentieth"),
    ] {
        if (value - numerator / denominator).abs() < 0.001 {
            return Some(said);
        }
    }
    None
}

/// 15 minutes, one minute, 7 and a half minutes.
pub fn _a_span_amount(amount: f64, name: &str) -> String {
    if (amount - round_py(amount) as f64).abs() < 0.0001 {
        let whole = round_py(amount);
        return if whole != 1 { format!("{} {name}", text::thousands(whole)) } else { format!("one {}", text::slice(name, 0, Some(-1))) };
    }
    if ((amount - amount.floor()) - 0.5).abs() < 0.0001 {
        return format!("{} and a half {name}", amount.trunc() as i64);
    }
    format!("{} {name}", fmt_g(amount))
}

/// Two, three, four: the way a span of time is said out loud.
pub fn _a_number_word(number: f64) -> String {
    const WORDS: [&str; 12] = ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve"];
    let n = number.trunc() as i64;
    if (1..=12).contains(&n) { WORDS[(n - 1) as usize].to_string() } else { n.to_string() }
}

/// A quarter of an hour, half an hour, an hour and a half, two and a half hours.
pub fn _a_share_of_a_unit(value: f64, unit: &str) -> String {
    let article = _TIME_ARTICLE.get(unit).unwrap_or_else(|| panic!("KeyError: {unit:?}"));
    let plural = format!("{}s", text::split_ref(article).last().unwrap());
    if (value - 0.5).abs() < 0.0001 {
        return format!("Half {article}");
    }
    if value > 1.0 - 0.0001 {
        let whole = value.trunc() as i64;
        let rest = round_to(value - whole as f64, 4);
        if rest.abs() < 0.0001 {
            if whole == 1 {
                return up(article);
            }
            return format!("{} {plural}", _a_number_word(whole as f64));
        }
        if (rest - 0.5).abs() < 0.0001 {
            if whole == 1 {
                return format!("{} and a half", up(article));
            }
            return format!("{} and a half {plural}", _a_number_word(whole as f64));
        }
        if whole == 1 {
            let rest_name = _a_fraction_name(rest).map(str::to_string).unwrap_or_else(|| fmt_g(rest));
            return format!("{} and {rest_name}", up(article));
        }
        return format!("{} {plural} and {}", _a_number_word(whole as f64), text::lower(&_a_share_of_a_unit(rest, unit)));
    }
    let name = _a_fraction_name(value).map(str::to_string).unwrap_or_else(|| fmt_g(value));
    if let Some(rest) = name.strip_prefix("a ") {
        return format!("A {rest} of {article}");
    }
    if let Some(rest) = name.strip_prefix("an ") {
        return format!("An {rest} of {article}");
    }
    format!("{} of {article}", up(&name))
}

lazy_re!(A_PLAIN_FIGURE, r"\d+(?:\.\d+)?");

/// The fraction one span is of another: 20 minutes is a third of an hour.
pub fn _a_fraction_of_a_time_unit(clean: &str) -> Option<String> {
    for pattern in [&*_A_FRACTION_OF, &*_A_FRACTION_AS] {
        let Some(m) = pattern.search(clean) else { continue };
        let unit = m.name_str("unit");
        let small = _TIME_UNIT.get(m.name_str("small")).map(String::as_str);
        let amount = m.name_str("amount");
        let mut count = num_of(&_NUMERAL, amount);
        if count.is_none() && A_PLAIN_FIGURE.fullmatch(amount).is_some() {
            count = amount.parse().ok();
        }
        let per_small = match (unit, small) {
            ("hour", Some("minute")) => Some(60.0),
            ("day", Some("hour")) => Some(24.0),
            ("week", Some("day")) => Some(7.0),
            ("minute", Some("second")) => Some(60.0),
            _ => None,
        };
        let (Some(count), Some(per_small)) = (count.filter(|c| *c != 0.0), per_small) else { continue };
        let plural = match small.unwrap() {
            "minute" => "minutes",
            "hour" => "hours",
            "second" => "seconds",
            _ => "days",
        };
        let said = _a_span_amount(count, plural);
        let article = &_TIME_ARTICLE[unit];
        return Some(match _a_fraction_name(count / per_small) {
            Some(name) => format!("{} is {name} of {article}.", up(&said)),
            None => format!("{} is about {}% of {article}.", up(&said), round_py(100.0 * count / per_small)),
        });
    }
    None
}

fn seconds_in(unit: &str) -> f64 {
    num_of(&_TIME_SECONDS, unit).unwrap_or_else(|| panic!("KeyError: {unit:?}"))
}

/// The span in the unit the line asked for, or in the next unit down.
pub fn _the_span_said(span_seconds: f64, unit: &str, clean: &str) -> String {
    let (factor, name) = _TIME_SMALLER.get(unit).unwrap_or_else(|| panic!("KeyError: {unit:?}"));
    if let Some(asked) = _A_UNIT_ASKED.search(clean) {
        if let Some(wanted) = _TIME_UNIT.get(asked.str(1)).filter(|w| !w.is_empty()) {
            if seconds_in(wanted) < seconds_in(unit) {
                return _a_span_amount(span_seconds / seconds_in(wanted), &_TIME_PLURAL[wanted.as_str()]);
            }
        }
    }
    _a_span_amount(span_seconds / (seconds_in(unit) / factor), name)
}

/// The whole hours, the leftover as a share of one, and the unit, of a mixed span. `has_rest`
/// says whether the pattern has a `rest` group (`_A_MIXED_SPAN` has, `_A_HALF_SPAN` has not).
fn _a_mixed_parts(m: &Match, has_rest: bool) -> (String, f64, f64) {
    let unit = _TIME_UNIT[m.name_str("unit")].clone();
    let whole = num_of(&_NUMERAL, m.name_str("whole")).unwrap_or_else(|| m.name_str("whole").parse().expect("float()"));
    if !has_rest || matches!(text::strip(m.name_str("rest")), "a half" | "half") {
        return (unit, whole, 0.5);
    }
    let n = m.name_str("n");
    let said_n = num_of(&_NUMERAL, n).unwrap_or_else(|| py_float(n).unwrap_or_else(|| panic!("ValueError: could not convert string to float: {n:?}")));
    (unit, whole, said_n / 60.0)
}

lazy_re!(FRACTIONS_WORD, r"\bfractions?\b");
lazy_re!(AND_A_HALF_NEXT, r"\s*and\s+a\s+half\b");

/// The span of time the line asks for, in the smaller unit, or the fraction of the unit.
pub fn _a_span_of_time(clean: &str) -> Option<String> {
    if clean.is_empty() || text::len(clean) > 200 {
        return None;
    }
    let asks = _A_SPAN_ASK.is_match(clean);
    let target = _A_TARGET_UNIT.is_match(clean);
    if !asks && !target && !_A_FRACTION_ASK.is_match(clean) && !_A_FRACTION_AS.is_match(clean) {
        return None;
    }
    if FRACTIONS_WORD.is_match(clean) {
        if let Some(said) = _a_fraction_of_a_time_unit(clean).filter(|s| !s.is_empty()) {
            return Some(said);
        }
    }
    if let Some(share) = _A_FRACTION_OF_A_MIXED.search(clean) {
        let span = share.name_str("span");
        let late = match _A_MIXED_SPAN.search(span) {
            Some(m) => Some((m, true)),
            None => _A_HALF_SPAN.search(span).map(|m| (m, false)),
        };
        let value = _a_fraction_value(share.name_str("word")).filter(|v| *v != 0.0);
        if let (Some((late, has_rest)), Some(value)) = (late, value) {
            let (late_unit, late_whole, late_extra) = _a_mixed_parts(&late, has_rest);
            let part = (late_whole + late_extra) * value;
            return Some(format!("{} is {}.", _a_share_of_a_unit(part, &late_unit), _the_span_said(part * seconds_in(&late_unit), &late_unit, clean)));
        }
    }
    let mixed = match _A_MIXED_SPAN.search(clean) {
        Some(m) => Some((m, true)),
        None => _A_HALF_SPAN.search(clean).map(|m| (m, false)),
    };
    if let Some((m, has_rest)) = mixed {
        let (unit, whole, extra) = _a_mixed_parts(&m, has_rest);
        let said = if (extra - 0.5).abs() < 0.0001 {
            _a_share_of_a_unit(whole + 0.5, &unit)
        } else {
            let article = &_TIME_ARTICLE[unit.as_str()];
            let plural = format!("{}s", text::split_ref(article).last().unwrap());
            let head = if whole == 1.0 { up(article) } else { format!("{} {plural}", _a_number_word(whole)) };
            format!("{head} and {}", _a_span_amount(extra * 60.0, "minutes"))
        };
        return Some(format!("{} is {}.", up(&said), _the_span_said((whole + extra) * seconds_in(&unit), &unit, clean)));
    }
    if target {
        return None; // a conversion to a named unit, which the table answers in its own words
    }
    if !asks {
        return None;
    }
    if let Some(whole) = _A_WHOLE_UNIT.match_(clean) {
        let unit = &_TIME_UNIT[whole.name_str("unit")];
        return Some(format!("{} is {}.", _a_share_of_a_unit(1.0, unit), _the_span_said(seconds_in(unit), unit, clean)));
    }
    let m = _A_TIME_SHARE.search(clean)?;
    let unit = &_TIME_UNIT[m.name_str("unit")];
    let mut value = _a_fraction_value(m.name_str("word")).filter(|v| *v != 0.0)?;
    if AND_A_HALF_NEXT.match_(ahead(clean, m.end(), 14)).is_some() {
        value += 0.5; // "half of an hour and a half" is the half of ninety minutes
    }
    Some(format!("{} is {}.", _a_share_of_a_unit(value, unit), _the_span_said(value * seconds_in(unit), unit, clean)))
}

/// A fraction of the measure the last turn was about: "and half of that".
pub fn _share_carried(clean: &str, memory: Option<&Memory>) -> Option<String> {
    const EDGE: &str = " ?.!,andso";
    let m = _SHARE_THAT.search(clean)?;
    let rest = format!("{} {}", &clean[..m.start()], &clean[m.end()..]);
    let mut rest = text::strip_chars(&rest, EDGE).to_string();
    while _SHARE_LEAD.match_(&rest).is_some() {
        rest = text::strip_chars(&_SHARE_LEAD.sub("", &rest), EDGE).to_string();
    }
    if !rest.is_empty() {
        return None; // a question with a subject of its own is not this follow-up
    }
    for user in last_turns(memory, 4).into_iter().rev() {
        let line = text::lower(text::strip(user));
        let Some(share) = _SHARE.search(&line) else {
            if !line.is_empty() {
                return None;
            }
            continue;
        };
        let said = text::squash(m.name_str("word"));
        return _share_of_a_measure(&format!("what is {said} of {}", &line[share.start()..]), false);
    }
    None
}

/// A converted measure as a sentence, or None when the line is not a conversion.
pub fn convert(clean: &str, memory: Option<&Memory>) -> Option<String> {
    if let Some(reply) = _answer(clean, memory).filter(|r| !r.is_empty()) {
        return Some(reply);
    }
    if has_memory(memory) {
        let said = |r: Option<String>| r.filter(|r| !r.is_empty());
        return said(_share_carried(clean, memory))
            .or_else(|| said(_carried_weight(clean, memory)))
            .or_else(|| said(_carried_measure(clean, memory)))
            .or_else(|| _carried_people(clean, memory));
    }
    None
}

// ---------------------------------------------------------------------------------------------
// How many a weight or a whole thing feeds

/// How many portions a counted number of pieces makes: "6 eggs", "12 sausages".
pub fn _count_in_a_line(clean: &str) -> Option<String> {
    if !_PORTIONS_ASK.is_match(clean) {
        return None;
    }
    let mut best: Option<(&str, String, &Portion)> = None;
    for row in PER_PERSON.iter() {
        if row.kind != "each" || !row.amount.truthy() {
            continue;
        }
        for name in &row.names {
            let found = rx(format!(r"\b(\d+)\s+(?:[a-z]+\s+)?{}(?![\w])", pyre::escape(name))).search(clean);
            if let Some(found) = found {
                if best.as_ref().is_none_or(|b| text::len(name) > text::len(b.0)) {
                    best = Some((name, found.str(1).to_string(), row));
                }
            }
        }
    }
    let (_name, count, row) = best?;
    // a figure too long for an i64 is more than 200 either way
    let count: i64 = count.parse().unwrap_or(i64::MAX);
    if count <= 0 || count > 200 {
        return None;
    }
    let (amount, noun) = (row.amount.f(), row.noun.as_str());
    let portions = count as f64 / amount;
    let halves = round_py(portions * 2.0) as f64 / 2.0;
    let piece = if noun.ends_with('s') || noun.contains(' ') { noun.to_string() } else { format!("{noun}s") };
    let each = if amount == 1.0 { "one".to_string() } else { fmt_g(amount) };
    if halves <= 1.0 {
        return Some(format!("{count} {piece} is about one portion: {each} a person is the usual allowance."));
    }
    Some(format!("{count} {piece} is about {} portions: {each} a person is the usual allowance.", fmt_g(halves)))
}

lazy_re!(A_BARE_WEIGHT, r"\b(half\s+)?(?:a|an|one)?\s*(kg|kilo|kilos|kilogram|kilograms|gram|grams|gramme|grammes)\b");

/// How many people a stated weight of a known food feeds.
pub fn _portions_in_a_weight(clean: &str) -> Option<String> {
    if clean.is_empty() {
        return None;
    }
    let m = _WEIGHT_SAID.search(clean);
    if m.is_none() {
        if let Some(counted) = _count_in_a_line(clean).filter(|c| !c.is_empty()) {
            return Some(counted);
        }
    }
    let (mut grams, unit) = match &m {
        Some(m) => (m.str(1).parse::<f64>().expect("float()"), text::lower(m.str(2))),
        None => {
            let bare = A_BARE_WEIGHT.search(clean)?;
            (if bare.get(1).is_some_and(|h| !h.is_empty()) { 0.5 } else { 1.0 }, text::lower(bare.str(2)))
        }
    };
    let in_kilos = unit.starts_with('k');
    if in_kilos {
        grams *= 1000.0;
    }
    if !(20.0..=20000.0).contains(&grams) {
        return None;
    }
    if !_PORTIONS_ASK.is_match(clean) {
        return None;
    }
    if _PORTION_WORD.is_match(clean) && _PEOPLE_WORD.is_match(clean) {
        return None;
    }
    let mut best: Option<(&str, &Portion)> = None;
    for row in PER_PERSON.iter() {
        if row.kind != "g" || !row.amount.truthy() {
            continue;
        }
        for name in &row.names {
            if rx(format!(r"(?<![\w.]){}(?![\w])", pyre::escape(name))).is_match(clean) && best.is_none_or(|b| text::len(name) > text::len(b.0)) {
                best = Some((name, row));
            }
        }
    }
    let (_name, row) = best?;
    let (amount, noun) = (row.amount.f(), row.noun.as_str());
    let weight = if in_kilos && grams >= 1000.0 { format!("{} kg", fmt_g(grams / 1000.0)) } else { format!("{} g", fmt_g(grams)) };
    let portions = grams / amount;
    if portions < 0.75 {
        return Some(format!("{weight} of {noun} is not a full portion on its own: allow about {} g a person.", fmt_g(amount)));
    }
    let halves = round_py(portions * 2.0) as f64 / 2.0;
    let said = fmt_g(halves).replace(".5", " and a half");
    if halves <= 1.0 {
        return Some(format!("{weight} of {noun} is about one portion: the usual allowance is {} g a person.", fmt_g(amount)));
    }
    Some(format!("{weight} of {noun} is about {said} portions: the usual allowance is {} g a person, so it feeds {said}.", fmt_g(amount)))
}

/// The number of people as words, as far as the table's own words go.
pub fn _a_table_word(count: i64) -> String {
    _SAID_PEOPLE.get(&count).filter(|w| !w.is_empty()).or_else(|| _PEOPLE_WORD_TO_TWENTY.get(&count).filter(|w| !w.is_empty())).cloned().unwrap_or_else(|| count.to_string())
}

/// `_SAID_PEOPLE.get(n, str(n))`.
fn said_or_figure(n: i64) -> String {
    _SAID_PEOPLE.get(&n).cloned().unwrap_or_else(|| n.to_string())
}

/// How many whole birds a table needs, or whether the number in the line covers it.
pub fn _birds_for_a_table(clean: &str) -> Option<String> {
    if clean.is_empty() || text::len(clean) > 200 {
        return None;
    }
    let found = _COUNT_OF_BIRDS.search(clean)?;
    if _NOT_A_TABLE_QUESTION.is_match(clean) || _KEEPING_HENS.is_match(clean) {
        return None;
    }
    let wanted = _people_asked(clean)?;
    let (word, name) = (found.str(1), found.str(2));
    let row = WHOLE_THING_SERVES.iter().find(|row| row.names.iter().any(|n| n == name))?;
    let meat = &row.names[0]; // the singular, for "10kg of chicken"
    let one = row.grams_each * row.feeds; // what one of them weighs, from the people it feeds
    let said = _a_table_word(wanted);
    let needs = -((-wanted * row.grams_each).div_euclid(one)); // whole birds, rounded up
    let size = &row.size;
    if word == "how many" {
        let kilos = fmt_g((wanted * row.grams_each) as f64 / 1000.0);
        if needs == 1 {
            return Some(format!("For {said} people you want about {kilos}kg of {meat} on the bone, and one bird of {size} is enough."));
        }
        return Some(format!("For {said} people you want about {kilos}kg of {meat} on the bone: {} birds of {size} each.", _a_table_word(needs)));
    }
    let have = _people_in_a_count(word).filter(|h| *h != 0)?;
    if have < 2 {
        return None;
    }
    let feeds_had = (have * one).div_euclid(row.grams_each);
    let many = text::capitalize(&_a_table_word(have));
    let had = if have == 1 { "is" } else { "are" };
    let kilos = fmt_g((have * one) as f64 / 1000.0);
    if feeds_had == wanted {
        return Some(format!("{many} {name} {had} about {kilos}kg on the bone, which is exactly enough for {said}."));
    }
    if feeds_had > wanted {
        return Some(format!("{many} {name} {had} about {kilos}kg on the bone, enough for {} - so yes for {said}.", said_or_figure(feeds_had)));
    }
    Some(format!("{many} {name} feed about {}, so {said} wants {} birds.", said_or_figure(feeds_had), said_or_figure(needs)))
}

/// The number a word or a figure says, or None.
pub fn _people_in_a_count(word: &str) -> Option<i64> {
    if word.is_empty() {
        return None;
    }
    people_value(word)
}

lazy_re!(
    FEEDS_A_COUNT,
    r"\b(?:feed|feeds|serve|serves|enough for)\s+(\d{1,2}|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve)\b"
);

/// How many people one whole bird, leg or joint feeds.
pub fn _a_whole_thing_feeds(clean: &str, _memory: Option<&Memory>) -> Option<String> {
    if clean.is_empty() || text::len(clean) > 200 {
        return None;
    }
    if !_FEEDS_ASK.is_match(clean) {
        return None;
    }
    if _NOT_A_TABLE_QUESTION.is_match(clean) || _KEEPING_HENS.is_match(clean) {
        return None;
    }
    if _A_CUT_NOT_A_BIRD.is_match(clean) {
        return None;
    }
    let mut best: Option<(&str, &Whole)> = None;
    for row in WHOLE_THING_SERVES.iter() {
        for name in &row.names {
            let named = pyre::escape(name);
            let pattern = format!(
                r"\b(?:one|1|a|an|the|single)\s+(?:whole\s+)?(?:\d+(?:\.\d+)?\s*kg\s+)?{named}\b|\bwhole\s+(?:\d+(?:\.\d+)?\s*kg\s+)?{named}\b"
            );
            if rx(pattern).is_match(clean) && best.is_none_or(|b| text::len(name) > text::len(b.0)) {
                best = Some((name, row));
            }
        }
    }
    let (name, row) = best?;
    let (mut size, mut feeds, mut stretch) = (row.size.clone(), row.feeds, row.stretch);
    let (more, grams_each, allowance, keeps) = (&row.more, row.grams_each, &row.allowance, &row.keeps);
    if let Some(weighed) = _WHOLE_WEIGHT.search(clean) {
        let kilos: f64 = weighed.str(1).parse().expect("float()");
        let grams = kilos * 1000.0;
        feeds = round_py(grams / grams_each as f64).max(1);
        size = format!("{}kg", fmt_g(kilos));
        stretch = if stretch.is_some_and(|s| s != 0) { Some(round_py(feeds as f64 * 1.5)) } else { None };
    }
    let stretch = stretch.filter(|s| *s != 0);
    let smaller = said_or_figure(feeds);
    let mut wanted = _people_asked(clean);
    if wanted.is_none() {
        wanted = FEEDS_A_COUNT.search(clean).and_then(|found| _people_in_a_count(found.str(1)));
    }
    let Some(wanted) = wanted else {
        let stretched = match stretch {
            Some(stretch) => format!(", {} at a stretch with plenty of sides", said_or_figure(stretch)),
            None => String::new(),
        };
        return Some(format!("One {name}, {size}, feeds {smaller} people{stretched}{keeps}. A bigger table is {more}."));
    };
    let said = said_or_figure(wanted);
    if wanted <= feeds {
        return Some(format!(
            "One {name}, {size}, feeds {smaller} people as a roast - so one is enough for {said}. Allow {allowance} if you are buying another."
        ));
    }
    if let Some(stretch) = stretch.filter(|s| wanted <= *s) {
        let reached = said_or_figure(stretch);
        return Some(format!(
            "One {name}, {size}, feeds {smaller} comfortably and {reached} at a stretch with plenty of sides - so one is enough for {said} if you lay on the sides. Otherwise {more}."
        ));
    }
    Some(format!("One {name}, {size}, feeds {smaller}, so for {said} you want {more}. That is the {allowance} figure."))
}

// ---------------------------------------------------------------------------------------------
// The table's own answer

lazy_re!(HOW_MANY_AT_THE_END, r"how (?:many|much)\s+$");
lazy_re!(WANTED, "how (?:many|much)\\s+(?P<want>[a-z/'\u{2019} ]+?)\\s+(?:are there\\s+)?(?:in|to|into|is|as)\\s+(?P<rest>.+)$");
lazy_re!(A_CUP_OR_A_SPOON, r"\b(cups?|mugs?|tablespoons?|tbsp|teaspoons?|tsp|dessertspoons?|dsp)\b");

/// The table's own answer to a line that carries its whole question.
pub fn _answer(clean: &str, memory: Option<&Memory>) -> Option<String> {
    if clean.is_empty() || text::len(clean) > 200 {
        return None;
    }
    let said = |r: Option<String>| r.filter(|r| !r.is_empty());
    if let Some(whole) = said(_a_whole_thing_feeds(clean, memory)).or_else(|| said(_birds_for_a_table(clean))) {
        return Some(whole);
    }
    if let Some(measured) = said(_the_thing_measured_by_itself(clean)) {
        return _answer(&measured, memory);
    }
    if _COMPONENT.is_match(clean) && !_the_component_is_the_whole_measure(clean) {
        return None; // what is in a cup of tea is a nutrition question, not a measure one
    }
    if let Some(share) = said(_share_of_a_measure(clean, true)) {
        return Some(share);
    }
    if let Some(span) = said(_a_span_of_time(clean)) {
        return Some(span);
    }
    if let Some(portions) = said(_portions_in_a_weight(clean)) {
        return Some(portions);
    }
    if let Some(temperature) = said(_oven(clean)).or_else(|| said(_temperature(clean))).or_else(|| said(_temperature_rule(clean))) {
        return Some(temperature);
    }
    if let Some(butter) = said(_butter(clean)) {
        return Some(butter);
    }
    if _PEOPLE_COUNT.is_match(clean) || _PEOPLE_WORD.is_match(clean) {
        if let Some(dish) = said(_a_dish_for_a_table(clean)) {
            return Some(dish);
        }
        if let Some(portion) = said(_per_person(clean, memory)) {
            return Some(portion);
        }
    }
    if let Some(pack) = said(_pack(clean)) {
        return Some(pack);
    }
    if let Some(counted) = said(_pack_count(clean)) {
        return Some(counted);
    }
    if let Some(loose) = said(_loose(clean)) {
        return Some(loose);
    }
    if let Some(weight) = said(_object_weight(clean)) {
        return Some(weight);
    }
    if let Some(piece) = said(_piece_weight(clean)) {
        return Some(piece);
    }
    if let Some(liquid) = said(_liquid_weight(clean)) {
        return Some(liquid);
    }
    if let Some(portion) = said(_per_person(clean, memory)) {
        return Some(portion);
    }
    let clean = _SAY_K.sub(r"\1 km", clean);
    let clean = _frac(&clean);
    let clean = _JOINED.sub(" ", &clean);
    let clean = clean.as_str();
    if let Some(handful) = said(_handful(clean)) {
        return Some(handful);
    }
    if let Some(own) = said(_own_height(clean)) {
        return Some(own);
    }
    let mentions = _mentions(clean);
    if mentions.is_empty() {
        return None;
    }
    if let Some(rule) = said(_how_to_convert(clean, &mentions)) {
        return Some(rule);
    }
    if let Some(height) = said(_height(clean, &mentions)) {
        return Some(height);
    }
    if let Some(land) = said(_land_size(clean, &mentions)) {
        return Some(land);
    }

    let mut measured: Vec<Measured> = Vec::new();
    for mention in &mentions {
        if let Some(value) = _number_before(clean, mention.start) {
            measured.push(Measured { value, family: mention.family.clone(), unit: mention.unit.clone(), end: mention.end });
        }
    }

    if let Some(weighed) = said(_named_ingredient(clean, &mentions, &measured)) {
        return Some(weighed);
    }

    for one in &measured {
        // Python rebinds `family` and `unit` to the ounce reading as it walks the units after
        // this one, and the rebinding stays for the rest of them.
        let (mut family, mut unit) = (one.family.clone(), one.unit.clone());
        let rest = &clean[one.end..];
        for m in _UNIT_WORD.finditer(rest) {
            let word = m.as_str();
            let before = &rest[..m.start()];
            if word == "in" || !(_TARGET.is_match(before) || HOW_MANY_AT_THE_END.is_match(before)) {
                continue;
            }
            let (named_family, named_unit) = &_NAMES[word];
            let (f, u, to_family, to_unit) = _the_ounce_pair(&family, &unit, named_family, named_unit);
            family = f;
            unit = u;
            if to_family != family || to_unit == unit {
                continue;
            }
            if _UNIT_WORD.is_match(&rest[m.end()..]) {
                break;
            }
            return _sentence(one.value, &family, &unit, &to_unit, false);
        }
    }

    if let Some(wanted_match) = WANTED.search(clean) {
        let mut want = _named_unit(wanted_match.name_str("want"));
        let rest_at = wanted_match.name_span("rest").map(|s| s.0).unwrap_or(0);
        let mut source = measured.iter().find(|one| one.end > rest_at).cloned();
        if source.as_ref().is_some_and(|s| !_tail_is_about_the_measure(&clean[s.end..])) {
            source = None;
        }
        if let (Some(w), Some(s)) = (&want, &source) {
            let (want_family, want_unit, source_family, source_unit) = _the_ounce_pair(&w.0, &w.1, &s.family, &s.unit);
            source = Some(Measured { value: s.value, family: source_family, unit: source_unit, end: s.end });
            want = Some((want_family, want_unit));
        }
        if let (Some(w), Some(s)) = (&want, &source) {
            if w.0 == s.family && w.1 != s.unit {
                if s.family == "mass" && matches!(s.unit.as_str(), "cup" | "tbsp" | "tsp") {
                    return _cup_of_something(clean);
                }
                return _sentence(s.value, &s.family, &s.unit, &w.1, true);
            }
            if matches!(w.1.as_str(), "cup" | "tbsp" | "tsp" | "dessertspoon") && s.family == "volume" {
                return _cup_of_something(clean);
            }
        }
    }
    if A_CUP_OR_A_SPOON.is_match(clean) {
        return _cup_of_something(clean);
    }
    None
}

/// A unit the line names with a number in front of it: (value, family, unit, end).
#[derive(Clone, Debug)]
pub struct Measured {
    pub value: f64,
    pub family: String,
    pub unit: String,
    pub end: usize,
}

// ---------------------------------------------------------------------------------------------
// A follow-up that carries the last measure, portion or headcount

type UnitWord = (String, (String, String));

/// Every unit word a line names, in the order written.
pub fn _unit_words(text_: &str) -> Vec<UnitWord> {
    _UNIT_WORD.finditer(text_).iter().map(|m| (m.as_str().to_string(), _NAMES[m.as_str()].clone())).collect()
}

lazy_re!(A_DIGIT, r"\d");

/// The unit and food a bare measure phrase names, or None when it says anything more.
///
/// A follow-up is the unit, the food and the grammar between them ("and in a tablespoon",
/// "what about a cup of sugar"). A line with a verb or a subject of its own is a new question.
pub fn _bare_measure(text_: &str) -> Option<(Vec<UnitWord>, Option<(&'static str, Num)>)> {
    let lowered = text::lower(text::strip(text_));
    let line = _CARRIED_LEAD.sub("", &lowered);
    let line = text::strip_chars(&line, " ?.!,");
    if line.is_empty() || A_DIGIT.is_match(line) {
        return None;
    }
    let said = _unit_words(line);
    if said.is_empty() {
        return None;
    }
    let mut spare: HashSet<String> = text::split(&line.replace(',', " ")).into_iter().collect();
    spare.retain(|word| !_CARRIED_FILLER.contains(word));
    for (word, _) in &said {
        for part in text::split_ref(word) {
            spare.remove(part);
        }
    }
    let food = _food_named(line);
    if let Some((name, _)) = food {
        for part in text::split_ref(name) {
            spare.remove(part);
        }
    }
    if !spare.is_empty() {
        return None;
    }
    Some((said, food))
}

/// The measure question the chat is still on, from the turns the memory keeps.
///
/// A bare measure phrase is walked past to reach the question it belongs to; a turn that is
/// neither ends the chain.
pub fn _previous_measure(memory: Option<&Memory>) -> Option<String> {
    for user in last_turns(memory, 4).into_iter().rev() {
        let line = text::lower(text::strip(user));
        if line.is_empty() {
            return None;
        }
        if _answer(&line, None).is_some_and(|r| !r.is_empty()) {
            return Some(line);
        }
        _bare_measure(&line)?;
    }
    None
}

/// The last portion question in the chat, from the turns the memory keeps.
pub fn _previous_portion(memory: Option<&Memory>) -> Option<String> {
    for user in last_turns(memory, 3).into_iter().rev() {
        let line = text::lower(text::strip(user));
        if line.is_empty() {
            return None;
        }
        if _PEOPLE_COUNT.is_match(&line) && _per_person(&line, None).is_some_and(|r| !r.is_empty()) {
            return Some(line);
        }
        _bare_measure(&line)?; // the chat has moved on to something that is not a portion
    }
    None
}

/// The portion question the chat is still on, asked for a different number of people.
pub fn _carried_people(clean: &str, memory: Option<&Memory>) -> Option<String> {
    const EDGE: &str = " ?.!,";
    let mut line = text::lower(text::strip(clean));
    if _CARRIED_NOW.match_(text::strip_chars(&line, EDGE)).is_some() {
        return _portion_after_correction(memory);
    }
    for _ in 0..3 {
        let stripped = _TRAILING_JOIN.sub("", &_CARRIED_LEAD.sub("", &line));
        let stripped = text::strip_chars(&stripped, EDGE).to_string();
        if stripped == text::strip_chars(&line, EDGE) {
            break;
        }
        line = stripped;
    }
    let line = text::strip_chars(&line, EDGE);
    let m = _CARRIED_PEOPLE.match_(line)?;
    let previous = _previous_portion(memory).filter(|p| !p.is_empty())?;
    let found = _PEOPLE_COUNT.search(&previous)?;
    let old = first_group(&found)?;
    let rebuilt = format!("{}{}{}", &previous[..found.start()], found.as_str().replacen(old, m.name_str("count"), 1), &previous[found.end()..]);
    _answer(&rebuilt, None)
}

lazy_re!(A_WEIGHT_FIGURE, r"\b(\d+(?:\.\d+)?)\s*(kg|kilo|kilos|kilogram|kilograms|g|gram|grams|gramme|grammes)\b");

/// The portion question the chat is still on, asked with a different weight.
pub fn _carried_weight(clean: &str, memory: Option<&Memory>) -> Option<String> {
    let squashed = text::squash(&text::lower(clean));
    let line = text::strip_chars(&squashed, " ?.!,");
    let m = _NEW_WEIGHT_THEN.match_(line)?;
    let previous = _previous_portion(memory).filter(|p| !p.is_empty())?;
    let said_amount = m.name_str("amount");
    let grams = match A_WEIGHT_FIGURE.search(said_amount) {
        Some(found) => {
            let grams: f64 = found.str(1).parse().expect("float()");
            if found.str(2).starts_with('k') { grams * 1000.0 } else { grams }
        }
        None => {
            if said_amount.starts_with("half") { 500.0 } else { 1000.0 }
        }
    };
    let mut best: Option<(&str, &Portion)> = None;
    for row in every_portion() {
        if row.kind != "g" || !row.amount.truthy() {
            continue;
        }
        for name in &row.names {
            if rx(format!(r"(?<![\w.]){}(?![\w])", pyre::escape(name))).is_match(&previous) && best.is_none_or(|b| text::len(name) > text::len(b.0)) {
                best = Some((name, row));
            }
        }
    }
    let (_name, row) = best?;
    let (amount, noun) = (fmt_g(row.amount.f()), row.noun.as_str());
    let weight = if grams < 1000.0 || !said_amount.contains("kg") { format!("{} g", fmt_g(grams)) } else { format!("{} kg", fmt_g(grams / 1000.0)) };
    let portions = grams / row.amount.f();
    let people = _people_asked(&previous).filter(|p| *p != 0);
    let halves = round_py(portions * 2.0) as f64 / 2.0;
    let said = fmt_g(halves).replace(".5", " and a half");
    if let Some(people) = people {
        if (halves - people as f64).abs() < 0.75 {
            return Some(format!("{weight} of {noun} is about right for {people}: the usual allowance is {amount} g a person."));
        }
        let side = if halves > people as f64 { "a little more" } else { "a little less" };
        return Some(format!(
            "{weight} of {noun} is about {said} portions, {side} than the {people} you asked about: the usual allowance is {amount} g a person."
        ));
    }
    if halves <= 1.0 {
        return Some(format!("{weight} of {noun} is about one portion: the usual allowance is {amount} g a person."));
    }
    Some(format!("{weight} of {noun} is about {said} portions: the usual allowance is {amount} g a person, so it feeds {said}."))
}

/// True when the line names a food the portion table knows, so it is a portion question.
pub fn _names_a_portion_food(line: &str) -> bool {
    every_portion().any(|row| row.names.iter().any(|name| rx(format!(r"(?<![\w.]){}(?![\w])", pyre::escape(name))).is_match(line)))
}

/// The portion question the chat was on, asked again for the count the person corrected.
///
/// "how much mince do i need" answers for six, then "oh sorry i meant 8 people not 6" is the
/// new headcount, and "so how much now" is the first question with the second number.
pub fn _portion_after_correction(memory: Option<&Memory>) -> Option<String> {
    let mut portion: Option<String> = None;
    let mut count: Option<i64> = None;
    for user in last_turns(memory, 4).into_iter().rev() {
        let line = text::squash(&text::lower(user));
        if line.is_empty() {
            break;
        }
        if _AN_ASK.is_match(&line) && _names_a_portion_food(&line) {
            portion = Some(line);
            break;
        }
        if _CORRECTION.is_match(&line) {
            if let Some(found) = _people_asked(&line) {
                count = Some(found);
                continue;
            }
        }
        break;
    }
    let (portion, count) = (portion?, count?);
    let Some(found) = _PEOPLE_COUNT.search(&portion) else {
        return _answer(&format!("{portion} for {count} people"), None);
    };
    let old = first_group(&found)?;
    let rebuilt = format!("{}{}{}", &portion[..found.start()], found.as_str().replacen(old, &count.to_string(), 1), &portion[found.end()..]);
    _answer(&rebuilt, None)
}

/// The measure question the turn before asked, asked again with this line's unit.
pub fn _carried_measure(clean: &str, memory: Option<&Memory>) -> Option<String> {
    let (said, mut food) = _bare_measure(clean)?;
    let previous = _previous_measure(memory).filter(|p| !p.is_empty())?;
    let want = _ASKED_MEASURE.search(&previous)?;
    let want_family = _named_unit(want.name_str("want"));
    let (unit_word, unit) = said.last().unwrap();
    let previous_units = _unit_words(&previous);
    let mut source: Option<&UnitWord> = None;
    for one in &previous_units {
        if want_family.as_ref().is_none_or(|w| one.1.1 != w.1) {
            source = Some(one);
        }
    }
    if food.is_none() {
        food = _food_named(&previous);
    }
    let of_food = food.map(|f| format!(" of {}", f.0)).unwrap_or_default();
    let rebuilt = match (&want_family, source) {
        (Some(w), Some(source)) if unit.1 == w.1 => format!("how many {unit_word} in a {}{of_food}", source.0),
        _ => format!("how many {} in a {unit_word}{of_food}", want.name_str("want")),
    };
    _answer(&rebuilt, None)
}

// ---------------------------------------------------------------------------------------------
// Amounts as a cook says them

/// The article a word takes: "an hour", "a minute". Half an hour, not half a hour.
pub fn _a_or_an(word: &str) -> String {
    let first = word.chars().next().map(|c| text::lower(&c.to_string()));
    if first.is_some_and(|c| "aeiou".contains(c.as_str())) || text::lower(word).starts_with("hour") {
        return format!("an {word}");
    }
    format!("a {word}")
}

/// An amount as a cook says it: half a cup, three quarters of a cup, 200 grams.
pub fn _amount_phrase(value: f64, unit: &str) -> String {
    if num_of(&_SHARE_OF_CUP, unit).is_some() {
        let said = _cups_phrase(value, unit, false);
        if said != format!("{} {}", _fmt(value), _say(value, unit, Some(false))) {
            return said;
        }
    }
    if let Some(one) = SAID_ONE.get(unit) {
        let word = if value == 0.25 {
            Some("a quarter of")
        } else if value == 0.5 {
            Some("half")
        } else if value == 0.75 {
            Some("three quarters of")
        } else {
            None
        };
        if let Some(word) = word {
            return format!("{word} {}", _a_or_an(one));
        }
    }
    format!("{} {}", _fmt(value), _say(value, unit, None))
}

/// A cup amount as a cook says it: half a cup, not 0.5 cups.
///
/// Under half a cup and off the fractions, a cook weighing a food reaches for a spoon
/// (`spoons`); a plain pair of units stays in cups.
pub fn _cups_phrase(amount: f64, unit: &str, spoons: bool) -> String {
    let mut amount = amount;
    if unit == "cup" {
        for (value, said) in _FRACTIONS.iter() {
            if (amount - value).abs() < 0.06 {
                return said.clone();
            }
        }
        if (amount - 1.0).abs() < 0.06 {
            return "1 cup".to_string();
        }
        if spoons && 0.0 < amount && amount < 0.5 {
            let spoons = amount * 16.0;
            if spoons < 2.0 {
                return format!("{} teaspoons", _spoons(spoons * 3.0));
            }
            if spoons < 10.0 {
                return format!("{} tablespoons", _spoons(spoons));
            }
        }
        if amount >= 0.1 {
            amount = round_to(amount, 1); // "about 0.6 cups", not 0.59
        }
    }
    format!("{} {}", _fmt(amount), _say(amount, unit, Some(false)))
}

/// A number of spoons as a recipe says it: 4, 6, or 1 1/2 when half a spoon matters.
pub fn _spoons(count: f64) -> String {
    let nearest = round_py(count * 2.0) as f64 / 2.0;
    if nearest == nearest.trunc() {
        return (nearest as i64).to_string();
    }
    format!("{} 1/2", nearest.trunc() as i64)
}

/// True when the reply says more than how many of one unit make another: a named food or an
/// oven has no hand-written fact, and the table is the only place that knows the answer.
pub fn answers_more_than_a_pair(clean: &str) -> bool {
    if _SPECIFIC.is_match(clean) {
        return true;
    }
    _food_named(clean).is_some()
}

/// The food this line names, and what a cup of it weighs, the longest name winning.
pub fn _food_named(clean: &str) -> Option<(&'static str, Num)> {
    let mut best: Option<(&'static str, Num)> = None;
    for (name, grams) in GRAMS_PER_CUP.iter() {
        if rx(format!(r"\b{}\b", pyre::escape(name))).is_match(clean) && best.is_none_or(|b| text::len(name) > text::len(b.0)) {
            best = Some((name, *grams));
        }
    }
    best
}

/// A cook's weight and cups of the same ingredient, converted either way.
pub fn _named_ingredient(clean: &str, mentions: &[Mention], measured: &[Measured]) -> Option<String> {
    let (food_name, grams_per_cup) = _food_named(clean)?;
    let said_food = AS_KITCHEN_SAYS.get(food_name).map(String::as_str).unwrap_or(food_name);
    let share_of = |unit: &str| num_of(&_SHARE_OF_CUP, unit);
    let wanted: Vec<&str> = mentions.iter().map(|m| m.unit.as_str()).filter(|u| share_of(u).is_some()).collect();
    if wanted.is_empty() {
        return None;
    }
    for one in measured {
        if let Some(share) = share_of(&one.unit) {
            let grams = one.value * grams_per_cup.f() * share;
            let source = up(&format!("{} of {said_food}", _amount_phrase(one.value, &one.unit)));
            let Some(target) = mentions.iter().find(|m| m.family == "mass").map(|m| m.unit.as_str()) else { continue };
            if share_of(target).is_some() {
                continue;
            }
            let amount = grams / _factor("mass", target).expect("TypeError: float / None");
            let said = if target == "g" { _grams(amount) } else { _fmt(amount) };
            return Some(format!("{source} is about {said} {}.", SAID[target]));
        }
        if one.family == "mass" {
            let grams = one.value * _factor("mass", &one.unit).expect("TypeError: float * None");
            let target = wanted[0];
            let amount = grams / (grams_per_cup.f() * share_of(target).unwrap());
            let source = up(&format!("{} of {said_food}", _amount_phrase(one.value, &one.unit)));
            return Some(format!("{source} is about {}.", _cups_phrase(amount, target, true)));
        }
    }
    None
}

/// The words a cook counts more than one of: "onions", "cloves of garlic".
pub fn _plural_of(name: &str) -> String {
    let (head, _, tail) = text::partition(name, " ");
    let chars: Vec<char> = head.chars().collect();
    let head = if head.ends_with('y') && chars.len() > 1 && !"aeiou".contains(chars[chars.len() - 2]) {
        format!("{}ies", chars[..chars.len() - 1].iter().collect::<String>())
    } else if ["o", "ch", "sh", "s", "x"].iter().any(|e| head.ends_with(e)) {
        format!("{head}es")
    } else {
        format!("{head}s")
    };
    if tail.is_empty() { head } else { format!("{head} {tail}") }
}

/// The piece with its article: "an egg", "a banana".
pub fn _a(name: &str) -> String {
    format!("{}{name}", if name.starts_with(['a', 'e', 'i', 'o', 'u']) { "an " } else { "a " })
}

lazy_re!(WHOLE_AND_A_HALF, r"(one|two|three|four) and a half");

/// How many the line counts: "an" is one, "half a" is a half, "a quarter of" a quarter.
/// A count word past "five" is not in Python's table (`_SAID_WHOLE`), which raises KeyError.
pub fn _counted_as(word: &str) -> f64 {
    let word = text::squash(&word.replace('-', " "));
    let whole = |w: &str| num_of(&_SAID_WHOLE, w).unwrap_or_else(|| panic!("KeyError: {w:?}"));
    match word.as_str() {
        "half" | "a half" => return 0.5,
        "quarter" | "a quarter" => return 0.25,
        "three quarters" => return 0.75,
        "a third" => return 1.0 / 3.0,
        "two thirds" => return 2.0 / 3.0,
        _ => {}
    }
    if let Some(m) = WHOLE_AND_A_HALF.fullmatch(&word) {
        return whole(m.str(1)) + 0.5;
    }
    if A_PLAIN_FIGURE.fullmatch(&word).is_some() {
        return word.parse().expect("float()");
    }
    whole(&word)
}

/// The piece this line counts, and what one of it weighs: (name, grams, count) or None.
///
/// Only a counted piece is a piece ("a pepper", "2 onions"); a tin, a jar or a loaf is not the
/// thing loose, so those lines are left alone.
pub fn _piece_named(clean: &str) -> Option<(&'static str, Num, f64)> {
    if _UNCOUNTED.is_match(&_WHOLE_THING.sub("", clean)) {
        return None;
    }
    if _MEASURE_AFTER_PIECE.is_match(clean) {
        return None;
    }
    let mut best: Option<(&'static str, Num, f64)> = None;
    for (name, grams) in PIECE_GRAMS.iter() {
        for words in [name.clone(), _plural_of(name)] {
            let pattern = format!(
                r"(?<![\w.]){}(?![\w])(?=\s*(?:$|[,?]|of\b|weigh|weighs|weighed|about|in grams?|in g\b|to grams|is\b))",
                pyre::escape(&words)
            );
            let Some(m) = rx(pattern).search(clean) else { continue };
            let Some(counted) = _COUNT.search(&clean[..m.start()]) else { continue };
            if best.is_none_or(|b| text::len(name) > text::len(b.0)) {
                best = Some((name, *grams, _counted_as(counted.str(1))));
            }
        }
    }
    best
}

/// What one of a food weighs, when the line counts pieces instead of naming a measure.
pub fn _piece_weight(clean: &str) -> Option<String> {
    if !_asks_for_weight(clean) {
        return None;
    }
    let (name, grams, count) = _piece_named(clean)?;
    if _mentions(clean).iter().any(|m| _number_before(clean, m.start).is_some()) {
        return None;
    }
    let amount = _mass_of(grams.f() * count);
    if count == 1.0 {
        return Some(format!("{} is about {amount}.", text::capitalize(&_a(name))));
    }
    let (said, plural) = _count_phrase(count);
    if !plural {
        return Some(format!("{said} {} is about {amount}.", _a(name)));
    }
    Some(format!("{said} {} weigh about {amount}.", _plural_of(name)))
}

/// How the amount is said, and whether the food is plural: ("Half", False), ("Two", True).
pub fn _count_phrase(count: f64) -> (String, bool) {
    for (share, said) in [(0.25, "A quarter of"), (1.0 / 3.0, "A third of"), (0.5, "Half"), (2.0 / 3.0, "Two thirds of"), (0.75, "Three quarters of")] {
        if (count - share).abs() < 0.01 {
            return (said.to_string(), false);
        }
    }
    let whole = count.trunc() as i64;
    if count != count.trunc() {
        let word = if whole == 1 { "One".to_string() } else { _SAID_COUNT.get(&whole).cloned().unwrap_or_else(|| _fmt(whole as f64)) };
        return (format!("{word} and a half"), true);
    }
    if let Some(said) = _SAID_COUNT.get(&whole) {
        return (said.clone(), true);
    }
    (_fmt(count), true)
}

/// The unit a short phrase names ("grams", "cups of flour"), or None: the longest unit word.
pub fn _named_unit(text_: &str) -> Option<(String, String)> {
    let mut best: Option<(&(String, String), usize)> = None;
    for m in _UNIT_WORD.finditer(text_) {
        let length = text::len(m.as_str());
        if best.is_none_or(|b| length > b.1) {
            best = Some((&_NAMES[m.as_str()], length));
        }
    }
    best.map(|b| b.0.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `_build()` gives what Python's import left in `_NAMES` and `_UNIT_WORD`.
    #[test]
    fn the_unit_table_builds_as_python_built_it() {
        data::load();
        let (names, re) = _build();
        assert_eq!(re.pattern(), _UNIT_WORD.pattern());
        let exported: Vec<(String, (String, String))> = g("_NAMES")
            .pairs()
            .iter()
            .map(|(k, v)| (k.str().to_string(), (v.items()[0].str().to_string(), v.items()[1].str().to_string())))
            .collect();
        assert_eq!(names, exported);
    }

    #[test]
    fn python_number_formats() {
        assert_eq!(fmt_g(0.5), "0.5");
        assert_eq!(fmt_g(1200000.0), "1.2e+06");
        assert_eq!(fmt_g(150.0), "150");
        assert_eq!(fmt_g(1.0 / 3.0), "0.333333");
        assert_eq!(fmt_g(0.00001), "1e-05");
        assert_eq!(fmt_commas(1234567.5), "1,234,568");
        assert_eq!(fmt_commas_1(12345.67), "12,345.7");
        assert_eq!(back("abcdef", 4, 2), "cd");
        assert_eq!(back("ab", 2, 24), "ab");
        assert_eq!(ahead("a°cdef", 1, 2), "°c");
    }
}
