//! Port of `bot/knowledge/travel.py`: capitals, city guides, weather by month, jet lag, packing
//! and the airport.
//!
//! City knowledge is a table keyed by the city name (from the bundle), so "what shouldn't I miss
//! in Lisbon" and "lisbon in may, three days" reach the same facts. The tables are [`Table`]s:
//! Python's dicts, with their keys longest-first for [`_find_in`]. The entries themselves are
//! data; this file holds every function they and the rest of the bot call.

use crate::data::{self, FnRef, Value};
use crate::knowledge::d2_shared::{self, a_weight, not_the_flat_of_something_made, not_the_flat_you_live_in, tokens};
use crate::knowledge::d2dup_travel::{
    LIVES_THERE, Moment, carries_it_out_of_britain, char_window, crosses_a_border, days_from_civil, going_there, now_utc,
    the_plan, user_dict, weekday,
};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use crate::understood::Understood;
use std::cell::Cell;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};

// ---------------------------------------------------------------------------------------------
// Data access and Python shims

fn g(name: &str) -> &'static Value {
    data::get(&format!("bot.knowledge.travel.{name}"))
}
fn gre(name: &str) -> &'static Re {
    g(name).as_re().unwrap_or_else(|| panic!("travel.{name} is not a regex"))
}
fn gstr(name: &str) -> String {
    g(name).str().to_string()
}
fn gstrs(name: &str) -> Vec<String> {
    g(name).strs()
}
fn gset(name: &str) -> HashSet<String> {
    g(name).str_set()
}
fn num(v: &Value) -> f64 {
    v.as_f64().unwrap_or(0.0)
}
fn int(v: &Value) -> i64 {
    match v {
        Value::Int(i) => *i,
        Value::Float(f) => *f as i64,
        Value::Bool(b) => *b as i64,
        _ => 0,
    }
}

/// Python `str(value)` for the plain values the tables hold.
fn py_str(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        Value::Bool(b) => if *b { "True".into() } else { "False".into() },
        Value::None => "None".into(),
        _ => String::new(),
    }
}

/// `str.title()`.
fn title(s: &str) -> String {
    text::title(s)
}

/// `s[0].upper() + s[1:]`.
fn upper_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => panic!("IndexError: string index out of range"),
    }
}

/// `seq[i]` with Python's negative indices (IndexError is a panic).
fn py_idx<T>(seq: &[T], i: i64) -> &T {
    let n = seq.len() as i64;
    let at = if i < 0 { n + i } else { i };
    if at < 0 || at >= n {
        panic!("IndexError: index out of range");
    }
    &seq[at as usize]
}

/// Python's `s[a:b]` on character indices, from byte offsets of the same string.
fn bslice(s: &str, a: usize, b: usize) -> &str {
    if a >= b { "" } else { &s[a..b] }
}

/// The last `n` characters (`s[-n:]`).
fn last_chars(s: &str, n: usize) -> &str {
    let count = s.chars().count();
    if count <= n {
        return s;
    }
    &s[text::byte_at(s, count - n)..]
}

/// The first `n` characters (`s[:n]`).
fn first_chars(s: &str, n: usize) -> &str {
    &s[..text::byte_at(s, n)]
}

/// The `n` characters before a byte offset (`s[max(0, i - n):i]`).
fn chars_before(s: &str, byte: usize, n: usize) -> &str {
    last_chars(&s[..byte], n)
}

/// A regex built from a pattern made at run time (Python's `re.search(pattern, text)` and its
/// cache), compiled once per pattern.
fn rx(pattern: &str, flags: u32) -> Arc<Re> {
    static CACHE: LazyLock<Mutex<HashMap<(String, u32), Arc<Re>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let mut cache = CACHE.lock().unwrap();
    cache.entry((pattern.to_string(), flags)).or_insert_with(|| Arc::new(Re::new(pattern, flags))).clone()
}

/// An inline pattern, compiled once.
macro_rules! re {
    ($pat:expr) => {{
        static R: LazyLock<Re> = LazyLock::new(|| Re::new($pat, 0));
        &*R
    }};
    ($pat:expr, $flags:expr) => {{
        static R: LazyLock<Re> = LazyLock::new(|| Re::new($pat, $flags));
        &*R
    }};
}

/// Python's `\w` for one character.
fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Where `re.search(r"\b" + re.escape(name) + r"\b", text)` matches: the byte span of the first
/// occurrence with a word boundary at both ends.
fn word_find(text: &str, name: &str) -> Option<(usize, usize)> {
    let boundary = |pos: usize| -> bool {
        let before = text[..pos].chars().next_back().is_some_and(is_word);
        let after = text[pos..].chars().next().is_some_and(is_word);
        before != after
    };
    let mut start = 0;
    while start <= text.len() {
        let i = start + text[start..].find(name)?;
        let end = i + name.len();
        if boundary(i) && boundary(end) {
            return Some((i, end));
        }
        match text[i..].chars().next() {
            Some(c) => start = i + c.len_utf8(),
            None => return None,
        }
    }
    None
}

fn has_word(text: &str, name: &str) -> bool {
    word_find(text, name).is_some()
}

/// Where `re.search(r"(?<![a-z])" + re.escape(name) + r"(?![a-z])", text)` matches (byte start).
fn bounded_find(text: &str, name: &str) -> Option<usize> {
    let mut start = 0;
    while start <= text.len() {
        let i = start + text[start..].find(name)?;
        let end = i + name.len();
        let before = text[..i].chars().next_back().is_some_and(|c| c.is_ascii_lowercase());
        let after = text[end..].chars().next().is_some_and(|c| c.is_ascii_lowercase());
        if !before && !after {
            return Some(i);
        }
        match text[i..].chars().next() {
            Some(c) => start = i + c.len_utf8(),
            None => return None,
        }
    }
    None
}

fn bounded_in(text: &str, name: &str) -> bool {
    bounded_find(text, name).is_some()
}

/// The user's lines of the last `n` turns (`turns[-n:]`), oldest first.
fn last_turns(memory: &Memory, n: usize) -> &[Turn] {
    let turns = &memory.turns;
    &turns[turns.len().saturating_sub(n)..]
}

/// `str(user.get(key))` items of the user's dict: a string, or the strings of a list.
fn user_value<'m>(memory: &'m Memory, key: &str) -> Option<&'m Value> {
    user_dict(memory)?.get(key)
}

// ---------------------------------------------------------------------------------------------
// Tables

/// A Python dict of the bot's place tables: the values by key, the keys in dict order, and the
/// keys longest-first (`sorted(table, key=len, reverse=True)`), which is the order every place
/// search reads them in.
pub struct Table<V> {
    pub name: &'static str,
    keys: Vec<String>,
    longest: Vec<String>,
    map: HashMap<String, V>,
}

impl<V> Table<V> {
    fn new(name: &'static str, pairs: Vec<(String, V)>) -> Table<V> {
        let keys: Vec<String> = pairs.iter().map(|(k, _)| k.clone()).collect();
        let mut longest = keys.clone();
        longest.sort_by_key(|k| std::cmp::Reverse(k.chars().count()));
        Table { name, keys, longest, map: pairs.into_iter().collect() }
    }
    pub fn get(&self, key: &str) -> Option<&V> {
        self.map.get(key)
    }
    pub fn contains(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }
    /// The keys in dict order.
    pub fn keys(&self) -> &[String] {
        &self.keys
    }
    /// The keys longest first, ties in dict order.
    pub fn longest_first(&self) -> &[String] {
        &self.longest
    }
    /// `table[key]`: a missing key is Python's KeyError.
    pub fn at(&self, key: &str) -> &V {
        self.map.get(key).unwrap_or_else(|| panic!("KeyError: {key}"))
    }
}

fn table_of<V>(name: &'static str, value: impl Fn(&Value) -> V) -> Table<V> {
    let pairs = g(name).pairs().iter().map(|(k, v)| (py_str(k), value(v))).collect();
    Table::new(name, pairs)
}
fn str_table(name: &'static str) -> Table<String> {
    table_of(name, |v| v.str().to_string())
}
fn keys_table(name: &'static str) -> Table<()> {
    Table::new(name, gstrs(name).into_iter().map(|k| (k, ())).collect())
}

pub static AIRPORT_NAME: LazyLock<Table<String>> = LazyLock::new(|| str_table("AIRPORT_NAME"));
pub static AIRPORT_TRANSFER: LazyLock<Table<String>> = LazyLock::new(|| str_table("AIRPORT_TRANSFER"));
pub static AIR_HOURS_CITY: LazyLock<Table<String>> = LazyLock::new(|| str_table("AIR_HOURS_CITY"));
pub static BEST_TIME: LazyLock<Table<String>> = LazyLock::new(|| str_table("BEST_TIME"));
pub static CAPITALS: LazyLock<Table<String>> = LazyLock::new(|| str_table("CAPITALS"));
pub static CHEAP_EATS: LazyLock<Table<String>> = LazyLock::new(|| str_table("CHEAP_EATS"));
pub static CITIES: LazyLock<Table<String>> = LazyLock::new(|| str_table("CITIES"));
pub static CITY_AREA: LazyLock<Table<String>> = LazyLock::new(|| str_table("CITY_AREA"));
pub static CITY_CLIMATE: LazyLock<Table<String>> = LazyLock::new(|| str_table("CITY_CLIMATE"));
pub static CITY_COUNTRY: LazyLock<Table<String>> = LazyLock::new(|| str_table("CITY_COUNTRY"));
pub static CITY_DAYS: LazyLock<Table<i64>> = LazyLock::new(|| table_of("CITY_DAYS", int));
pub static CITY_TELLS: LazyLock<Table<String>> = LazyLock::new(|| str_table("CITY_TELLS"));
pub static CITY_TRANSIT: LazyLock<Table<String>> = LazyLock::new(|| str_table("CITY_TRANSIT"));
pub static COUNTRY_ALIASES: LazyLock<Table<String>> = LazyLock::new(|| str_table("COUNTRY_ALIASES"));
pub static DAILY_BUDGET: LazyLock<Table<(Value, Value, String)>> = LazyLock::new(|| {
    table_of("DAILY_BUDGET", |v| {
        let t = v.items();
        (t[0].clone(), t[1].clone(), t[2].str().to_string())
    })
});
pub static DAY_TRIPS: LazyLock<Table<String>> = LazyLock::new(|| str_table("DAY_TRIPS"));
pub static DAY_TRIP_PLACES: LazyLock<Table<String>> = LazyLock::new(|| str_table("DAY_TRIP_PLACES"));
pub static FAMILY_VERDICT: LazyLock<Table<String>> = LazyLock::new(|| str_table("FAMILY_VERDICT"));
pub static FLIGHT_HOURS: LazyLock<Table<f64>> = LazyLock::new(|| table_of("FLIGHT_HOURS", num));
pub static FOOD_ADJECTIVES: LazyLock<Table<String>> = LazyLock::new(|| str_table("FOOD_ADJECTIVES"));
pub static FOOD_IN: LazyLock<Table<String>> = LazyLock::new(|| str_table("FOOD_IN"));
pub static GET_TO: LazyLock<Table<String>> = LazyLock::new(|| str_table("GET_TO"));
pub static MONTHS: LazyLock<Table<i64>> = LazyLock::new(|| table_of("MONTHS", int));
pub static MULTI_ZONES: LazyLock<Table<String>> = LazyLock::new(|| str_table("MULTI_ZONES"));
pub static PLUG_TYPES: LazyLock<Table<(String, String, String)>> = LazyLock::new(|| {
    table_of("PLUG_TYPES", |v| {
        let t = v.items();
        (t[0].str().to_string(), t[1].str().to_string(), t[2].str().to_string())
    })
});
pub static REGION_BEACHES: LazyLock<Table<String>> = LazyLock::new(|| str_table("REGION_BEACHES"));
pub static REGION_DAYS: LazyLock<Table<String>> = LazyLock::new(|| str_table("REGION_DAYS"));
pub static REGION_NAMES: LazyLock<Table<String>> = LazyLock::new(|| str_table("REGION_NAMES"));
pub static REGION_PLACES: LazyLock<Table<String>> = LazyLock::new(|| str_table("REGION_PLACES"));
pub static SITE_ALIASES: LazyLock<Table<String>> = LazyLock::new(|| str_table("SITE_ALIASES"));
pub static SITE_NOTES: LazyLock<Table<String>> = LazyLock::new(|| str_table("SITE_NOTES"));
pub static TIME_ZONES: LazyLock<Table<(f64, f64)>> = LazyLock::new(|| {
    table_of("TIME_ZONES", |v| {
        let t = v.items();
        (num(&t[0]), num(&t[1]))
    })
});
pub static TRANSIT_CARDS: LazyLock<Table<String>> = LazyLock::new(|| str_table("TRANSIT_CARDS"));
pub static WATER: LazyLock<Table<bool>> = LazyLock::new(|| table_of("WATER", |v| v.truthy()));
pub static WHEN_TO_GO: LazyLock<Table<String>> = LazyLock::new(|| str_table("WHEN_TO_GO"));
pub static WHERE_TO_GO: LazyLock<Table<(String, String)>> = LazyLock::new(|| {
    table_of("WHERE_TO_GO", |v| {
        let t = v.items();
        (t[0].str().to_string(), t[1].str().to_string())
    })
});
pub static CARD_WORDS: LazyLock<Table<()>> = LazyLock::new(|| keys_table("CARD_WORDS"));
pub static _AT_HOME: LazyLock<Table<String>> = LazyLock::new(|| str_table("_AT_HOME"));
pub static _PLACE: LazyLock<Table<(f64, f64, String)>> = LazyLock::new(|| {
    table_of("_PLACE", |v| {
        let t = v.items();
        (num(&t[0]), num(&t[1]), t[2].str().to_string())
    })
});

static CLIMATES: LazyLock<HashMap<String, HashMap<i64, (Value, Value)>>> = LazyLock::new(|| {
    g("CLIMATES")
        .pairs()
        .iter()
        .map(|(k, v)| {
            let months = v.pairs().iter().map(|(m, lh)| (int(m), (lh.items()[0].clone(), lh.items()[1].clone()))).collect();
            (py_str(k), months)
        })
        .collect()
});
static SEASON_MONTHS: LazyLock<HashMap<String, BTreeSet<i64>>> = LazyLock::new(|| {
    g("SEASON_MONTHS").pairs().iter().map(|(k, v)| (py_str(k), v.items().iter().map(int).collect())).collect()
});
static ROUTE_BETWEEN: LazyLock<Vec<((String, String), String)>> = LazyLock::new(|| {
    g("ROUTE_BETWEEN")
        .pairs()
        .iter()
        .map(|(k, v)| ((k.items()[0].str().to_string(), k.items()[1].str().to_string()), v.str().to_string()))
        .collect()
});
static TRIP_UNITS: LazyLock<Vec<(String, i64)>> =
    LazyLock::new(|| g("TRIP_UNITS").items().iter().map(|t| (t.items()[0].str().to_string(), int(&t.items()[1]))).collect());
/// `sorted(TRIP_UNITS, key=lambda item: -len(item[0]))`.
static TRIP_UNITS_LONGEST: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| {
    let mut v = TRIP_UNITS.clone();
    v.sort_by_key(|(p, _)| std::cmp::Reverse(p.chars().count()));
    v
});
/// `sorted(MONTHS.items(), key=lambda item: -len(item[0]))`.
static MONTHS_LONGEST: LazyLock<Vec<(String, i64)>> =
    LazyLock::new(|| MONTHS.longest_first().iter().map(|k| (k.clone(), *MONTHS.at(k))).collect());

fn words_then_reply(name: &str) -> Vec<(Vec<String>, String)> {
    g(name).items().iter().map(|t| (t.items()[0].strs(), t.items()[1].str().to_string())).collect()
}
static LONDON_AIRPORTS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| words_then_reply("LONDON_AIRPORTS"));
static HOLIDAY_WANTS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| words_then_reply("HOLIDAY_WANTS"));
static _TRAFFIC_SITUATIONS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| words_then_reply("_TRAFFIC_SITUATIONS"));
static DETAIL_PARTS: LazyLock<Vec<(Vec<String>, Vec<String>)>> =
    LazyLock::new(|| g("DETAIL_PARTS").items().iter().map(|t| (t.items()[0].strs(), t.items()[1].strs())).collect());
static _TRIP_LENGTH: LazyLock<Vec<(&'static Re, String, i64)>> = LazyLock::new(|| {
    g("_TRIP_LENGTH")
        .items()
        .iter()
        .map(|t| (t.items()[0].as_re().unwrap(), t.items()[1].str().to_string(), int(&t.items()[2])))
        .collect()
});
static _CABIN_RULES: LazyLock<Vec<(Re, String)>> = LazyLock::new(|| {
    g("_CABIN_RULES").items().iter().map(|t| (Re::new(t.items()[0].str(), 0), t.items()[1].str().to_string())).collect()
});
static _DAYS_WORD: LazyLock<HashMap<i64, String>> =
    LazyLock::new(|| g("_DAYS_WORD").pairs().iter().map(|(k, v)| (int(k), v.str().to_string())).collect());
static _NUMBER_WORDS: LazyLock<Table<i64>> = LazyLock::new(|| table_of("_NUMBER_WORDS", int));
static _SMALL_COUNT: LazyLock<Table<i64>> = LazyLock::new(|| table_of("_SMALL_COUNT", int));
static _ROAD_SPEED: LazyLock<Table<f64>> = LazyLock::new(|| table_of("_ROAD_SPEED", num));
static _NETWORK_FIX: LazyLock<Table<String>> = LazyLock::new(|| str_table("_NETWORK_FIX"));
static _NETWORK_NAME: LazyLock<Table<String>> = LazyLock::new(|| str_table("_NETWORK_NAME"));
static _MONTH_PICKS: LazyLock<Table<String>> = LazyLock::new(|| str_table("_MONTH_PICKS"));
static _MONTH_SEASON: LazyLock<Table<String>> = LazyLock::new(|| str_table("_MONTH_SEASON"));
static _MONTH_SUN: LazyLock<Table<String>> = LazyLock::new(|| str_table("_MONTH_SUN"));
static _MONTH_VERDICT: LazyLock<Table<String>> = LazyLock::new(|| str_table("_MONTH_VERDICT"));
static _WEEKS_AT_HOME: LazyLock<Table<String>> = LazyLock::new(|| str_table("_WEEKS_AT_HOME"));
static _BY_KIND: LazyLock<Table<String>> = LazyLock::new(|| str_table("_BY_KIND"));

macro_rules! data_strs {
    ($($name:ident),* $(,)?) => { $( static $name: LazyLock<Vec<String>> = LazyLock::new(|| gstrs(stringify!($name))); )* };
}
macro_rules! data_sets {
    ($($name:ident),* $(,)?) => { $( static $name: LazyLock<HashSet<String>> = LazyLock::new(|| gset(stringify!($name))); )* };
}
macro_rules! data_res {
    ($($name:ident),* $(,)?) => { $( static $name: LazyLock<&'static Re> = LazyLock::new(|| gre(stringify!($name))); )* };
}
macro_rules! data_text {
    ($($name:ident),* $(,)?) => { $( static $name: LazyLock<String> = LazyLock::new(|| gstr(stringify!($name))); )* };
}

data_strs!(
    AMBIGUOUS_PLACES, ANSWERS, FOLLOW_MARKS, SITE_WORDS, _AIRPORT_CHEAPEST, _AIRPORT_FASTEST, _AIRPORT_HOTEL,
    _AIRPORT_PORT, _AIRPORT_TRAIN, _ARRIVAL_ABOUT, _ARRIVAL_FARE_ASK, _ARRIVAL_LATE, _ARRIVAL_PICK, _A_LENGTH_ONLY,
    _A_MONTH_FAULT_ASK, _A_PLACE_OPENS, _A_PRICE, _A_TIME_VERDICT_ASK, _A_VAGUE_ASK, _CABIN_LIST, _CABIN_YES,
    _EARLY_FLIGHT, _FARES_ASK, _FARES_GOING_SOMEWHERE, _FAR_ASKED, _FOOD_NOT_A_TRIP, _FOOD_TO_CARRY, _LONG_MONTH,
    _MONTH_SUN_ASK, _NUMBER_WORD, _TAXI_TROUBLE, _TRANSIT_NOT, _TRANSIT_WORDS, _TRIP_IN_LINE, _UK_NATIONS,
    _WHERE_IN_THE_LINE,
);
data_sets!(
    ALSO_A_THING, AMBIGUOUS_MONTHS, ASKING_ONLY, FLIGHT_THREAD, MORE_PLACES, NOT_THE_ASK_WORDS, SEEING, _AN_OPEN_ASK,
    _A_VERDICT_FILLER, _EATING_WORDS, _NOT_A_PLACE, _NOT_A_TRIP_EVENT, _NOT_WHERE_ASKS, _PLACE_ENDS, _PLACE_FILLER,
    _SOMEONE_ELSE, _TRIP_SHAPE,
);
data_res!(
    AIRPORT_WORD, ASKS_A_LENGTH, A_DEADLINE_NEED, A_FACT_OF_THE_COUNTRY, A_JOURNEY_OR_A_BAG, BOOK_ASK,
    BOOK_NOT_THE_TRIP, CAPITAL_WORD, COUNTS_AS_A_THING, COUNTS_AS_A_TRIP, GOING_TO_A_PLACE, GROUND_TRANSPORT,
    SIGHTSEEING_ASK, STAYS_THERE, TRIP_WORDS, WHICH_FIRST, WHICH_ONE_FIRST, _ABOUT_A_TRIP, _ABOUT_THE_JOURNEY,
    _ABOUT_THE_PASSPORT, _AN_EATING, _A_BEACH, _A_CAR_NIGHT, _A_CLAUSE_VERB, _A_CUSTOMS_WORD, _A_FLIGHT_WORD,
    _A_GOODS_WORD, _A_JOURNEY_SAID, _A_MONTH_OF_A_TRIP, _A_PAY_ASK_AT_HOME, _A_PLANE, _A_QUESTION_OPENS, _A_REFUSAL,
    _A_SHIFT_WORK, _A_STAY_ASK, _A_STAY_PLACE_ASK, _A_TIME_LEFT, _A_TREATMENT_AT_HOME, _A_TRIP_NAMED,
    _A_VALIDITY_WORD, _A_WALK_IS_SAID, _A_WAY_ON_THE_GROUND, _CLOCK_AFTER, _CLOCK_MERIDIEM, _COACH_ASK, _DAY_SAID,
    _DRIVE_SAID, _EAT_ASK, _FERRY_ASK, _FLIGHT_LENGTH_ASK, _FLIGHT_SAID, _GUIDE_ASK, _HILL, _HOW_OFTEN, _IN_A_CITY,
    _LEAVE_SAID, _LENGTH_MINS, _LENGTH_SAID, _MONTH_IN_TEXT, _MONTH_NOT_A_PLACE_RE, _MONTH_RANGE, _MONTH_WORSE,
    _NO_TRIP_WANTED, _SAYS_THEIR_OWN_PLAN, _SEASON_IN_TEXT, _SEAT_ASK, _SOMEONE_WANTS, _STARTS_AT,
    _THE_TRIP_IN_THE_LINE, _TICKET_CHANGE, _TICKET_LOST, _TICKET_PRINT, _TIME_OF_DAY, _TIME_TO_DO_SOMETHING,
    _TRAINS_FOR, _TRIP_GO, _TRIP_WORD, _UNDER_SIX, _US_AS_A_PLACE, _VALUE_EATING, _WHEN_IT_IS, _WORTH_ASK,
);
data_text!(
    CHEAPER_TRIP, FLIGHT_LIQUIDS, GENERAL_CARD, TAP_WATER_GENERAL, TAP_WATER_HOME, TAP_WATER_HOME_BROWN,
    TAP_WATER_HOME_TASTE, TIP_ELSEWHERE, TIP_EUROPE, TIP_JAPAN, TIP_TAXI, TIP_US, TOWN_STAY, _AIRPORT_CHEAPEST_REPLY,
    _AIRPORT_EARLY_REPLY, _AIRPORT_FASTEST_REPLY, _AIRPORT_HOTEL_REPLY, _AIRPORT_PORT_REPLY, _AIRPORT_SLEEP_REPLY,
    _AIRPORT_TRAIN_REPLY, _AIRPORT_WAY_IN, _ARRIVAL_ABOUT_NOTE, _ARRIVAL_FARE, _ARRIVAL_LATE_NOTE,
    _ARRIVAL_PICK_NOTE, _ARRIVAL_TOWN, _CABIN_NOT_ALLOWED, _CARRY_MONEY, _CARRY_SHORT, _EUROPE_PLACE,
    _IN_THE_HOLD, _IN_THE_HOLD_ITSELF, _JAM_REPLY, _KEEP_SAFE, _NO_THE_WINDOW, _PLACE_SHAPE, _STAG_DO,
    _TAXI_WARNING, _TO_THE_AIRPORT, _US_PLACE, _WARM_WEEK, _YES_THE_WINDOW,
);

static _EUROPE_PLACE_RE: LazyLock<Re> = LazyLock::new(|| Re::new(&_EUROPE_PLACE, 0));
static _US_PLACE_RE: LazyLock<Re> = LazyLock::new(|| Re::new(&_US_PLACE, 0));
static _MONTH_LOOKS_BACK: LazyLock<usize> = LazyLock::new(|| int(g("_MONTH_LOOKS_BACK")) as usize);
static _NEAR_ENOUGH: LazyLock<i64> = LazyLock::new(|| int(g("_NEAR_ENOUGH")));
static _ROAD_FACTOR: LazyLock<f64> = LazyLock::new(|| num(g("_ROAD_FACTOR")));
static _ROAD_ALLOWANCE: LazyLock<f64> = LazyLock::new(|| num(g("_ROAD_ALLOWANCE")));

thread_local! {
    /// `_MONTH_ASKED`: whether the line that reached `_month_destination` was a question. The
    /// module global it sets and `_a_month_for_a_place` reads.
    static _MONTH_ASKED: Cell<bool> = Cell::new(g("_MONTH_ASKED").truthy());
}

/// `clean = understood.get("clean", "")` and its kin read the reading's clean line.
fn clean_of(u: &Understood) -> &str {
    &u.clean
}

/// `understood.get("clean") or understood.get("text") or ""`.
fn clean_or_text(u: &Understood) -> &str {
    d2_shared::or(&u.clean, &u.text)
}

// ---------------------------------------------------------------------------------------------
// Tickets, bookings and the flight thread

/// What to do with the ticket in hand, read off the line.
pub fn _a_ticket_already_bought(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = format!("{} {}", understood.clean, understood.text).to_lowercase();
    if _TICKET_LOST.is_match(&said) {
        return Some(concat!(
            "Tell the train company as soon as you know: with an e-ticket you can show the ",
            "confirmation again, and most operators will let you travel on the booking ",
            "reference and the card you paid with. A paper ticket is rarely replaced, so ",
            "photograph it when you buy it, and if it is lost at the station go to the ticket ",
            "office before boarding rather than after."
        ).into());
    }
    if _TICKET_CHANGE.is_match(&said) {
        return Some(concat!(
            "It depends which ticket it is: an advance ticket can usually be changed up to ",
            "the departure for a set fee (about \u{a3}10 on most operators) plus any difference in ",
            "fare, and an off-peak or anytime ticket needs no change at all, because it is ",
            "already good for any train in its band. If the train is cancelled, your ticket ",
            "is valid on the next one, and on a later one if you decide not to wait."
        ).into());
    }
    if _TICKET_PRINT.is_match(&said) {
        return Some(concat!(
            "An e-ticket on your phone is enough at almost every gateline now: have it open ",
            "and the brightness up before you reach the barrier, and screenshot it in case ",
            "signal drops. Print it only if the operator asks you to or you are crossing a ",
            "border, where a paper copy is easier to show."
        ).into());
    }
    Some(concat!(
        "Check the ticket's own restriction: an advance ticket is only good for the train it ",
        "names, while an off-peak or anytime ticket is valid on any train in its band, so a ",
        "different time is usually fine on those. If you are on the wrong train with an ",
        "advance, speak to the guard before the ticket check rather than after it."
    ).into())
}

/// How far ahead the ferry, the train or the coach is booked, in that order.
pub fn _book_a_journey(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = format!("{} {}", understood.clean, understood.text).to_lowercase();
    if _FERRY_ASK.is_match(&said) {
        return Some(concat!(
            "Book the ferry as soon as the dates are fixed: foot passengers can usually buy ",
            "on the day, but a car needs booking ahead and a summer weekend sailing sells ",
            "out. Book with the operator itself rather than a resale site, and take the ",
            "flexible ticket if the price is close, because a missed sailing with a car is ",
            "the expensive way to learn. Turn up an hour before with a car and 30 minutes on ",
            "foot, with the booking reference to hand."
        ).into());
    }
    if _COACH_ASK.is_match(&said) {
        return Some(concat!(
            "Book the coach two or three weeks ahead: the cheapest seats are the first to go ",
            "and a seat bought on the day can cost two or three times as much. Pick the seat ",
            "when the site offers it, be at the stop 15 minutes early because coaches leave ",
            "on time, and take the luggage rules seriously - one case in the hold is usual ",
            "and anything extra is paid for at the stop."
        ).into());
    }
    if _SEAT_ASK.is_match(&said) && !_FERRY_ASK.is_match(&said) {
        return Some(concat!(
            "Reserve the seat when you book the ticket: on most trains a reservation is free ",
            "with an advance ticket and a few pounds on a walk-up fare, and it is worth having ",
            "on any long trip. Friday evening and Sunday afternoon trains are the busiest, so ",
            "a booked seat there is the difference between sitting and standing."
        ).into());
    }
    Some(concat!(
        "Book the train as soon as the date is fixed: advance singles tied to one train are ",
        "usually about half the walk-up fare, and they go on sale around 12 weeks ahead. A ",
        "railcard pays for itself in one or two long journeys, and check the price of two ",
        "singles as well as a return - two advances are often cheaper than one return."
    ).into())
}

/// False when the line says the booking has already happened.
pub fn _the_booking_is_behind_them(understood: &Understood) -> bool {
    let low = if !understood.words.is_empty() {
        format!(" {} ", understood.words.join(" ").to_lowercase())
    } else {
        format!(" {} ", understood.clean.to_lowercase())
    };
    !low.contains(" booked ")
}

/// True unless "do it myself" is about a job rather than a trip.
pub fn _a_trip_to_arrange(understood: &Understood) -> bool {
    let text = if understood.clean.is_empty() { understood.words.join(" ") } else { understood.clean.clone() };
    _THE_TRIP_IN_THE_LINE.is_match(&text.to_lowercase())
}

/// Whether the last exchange was about a flight, so a short follow-up belongs to it.
pub fn _in_a_flight_thread(memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    let said = format!("{} {}", last.user, last.bot);
    tokens(&said).iter().any(|w| FLIGHT_THREAD.contains(w))
}

/// "Is it safe to travel to X" - the Foreign Office page, named, for the country asked about.
pub fn _travel_advice(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let place = _place_in(memory, understood, &WATER)?;
    Some(format!(
        concat!(
            "For {} the Foreign Office travel advice is the page to read before you ",
            "book: it says which areas to avoid and what has changed lately, and it is the page ",
            "your insurer reads too. Insurance can be void in an area it advises against. Take the ",
            "embassy number with you."
        ),
        title(&place)
    ))
}

static IN_THE_US: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bin (the )?us\b", 0));

/// The country the line names decides the answer; a general tip line when it names none.
pub fn _tipping(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    if ["taxi", "cab", "uber", "driver", "minicab"].iter().any(|w| clean.contains(w)) {
        return Some(TIP_TAXI.clone());
    }
    if ["japan", "japanese", "tokyo", "kyoto"].iter().any(|w| clean.contains(w)) {
        return Some(format!("{} In the US it is the opposite, 18 to 20%.", *TIP_JAPAN));
    }
    if _US_PLACE_RE.is_match(clean) || IN_THE_US.is_match(clean) {
        return Some(format!("{} Most of Europe includes service instead.", *TIP_US));
    }
    if _EUROPE_PLACE_RE.is_match(clean) {
        return Some(format!("{} The US is the big exception.", *TIP_EUROPE));
    }
    const HOTEL_OR_TOUR: &[&str] = &[
        "hotel", "housekeeping", "porter", "concierge", "bellboy", "maid", "room service", "tour guide", "tour",
        "excursion", "walking tour", "coach driver", "mover", "movers", "removal men", "man and van",
    ];
    if HOTEL_OR_TOUR.iter().any(|w| clean.contains(w)) {
        if let Some(said) = crate::knowledge::lifeadmin::_tipping(memory, understood).filter(|s| !s.is_empty()) {
            return Some(said);
        }
    }
    Some(TIP_ELSEWHERE.clone())
}

/// The airport the line names gets its own route; otherwise the closest-airport answer.
pub fn _london_airport(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    if GROUND_TRANSPORT.is_match(clean) && !AIRPORT_WORD.is_match(clean) {
        return None;
    }
    if A_DEADLINE_NEED.is_match(clean) {
        return None;
    }
    for (names, answer) in LONDON_AIRPORTS.iter() {
        if names.iter().any(|n| clean.contains(n.as_str())) {
            return Some(format!(
                "{answer} A black cab is the expensive option: the train beats it into the centre every time."
            ));
        }
    }
    Some(concat!(
        "London City is the closest airport to the centre: about 25 minutes to Bank on the ",
        "DLR. Of the big four, Heathrow is the quickest into town - the Elizabeth line takes ",
        "30 minutes to Paddington for about 13 pounds."
    ).into())
}

/// `_flight_followup(answer)`'s reply: the answer only inside a flight thread, else pass.
pub fn _flight_followup_reply(answer: &str, memory: &Memory) -> Option<String> {
    if _in_a_flight_thread(memory) { Some(answer.to_string()) } else { None }
}

// ---------------------------------------------------------------------------------------------
// Countries: where to go, and the country that was the question

/// Whether the place word in this line is preceded by a word that makes it a thing.
pub fn _a_thing_not_a_place(clean: &str, name: &str) -> bool {
    if !ALSO_A_THING.contains(&text::strip(name).to_lowercase()) {
        return false;
    }
    let low = format!(" {} ", text::squash(&clean.to_lowercase()));
    let pattern = rx(&format!(r"\b{}\b", pyre::escape(&name.to_lowercase())), 0);
    let mut found_a_thing = false;
    for m in pattern.finditer(&low) {
        if COUNTS_AS_A_THING.is_match(&low[..m.start()]) {
            if COUNTS_AS_A_TRIP.match_(&low[m.end()..]).is_some() {
                return false;
            }
            found_a_thing = true;
        } else {
            return false;
        }
    }
    found_a_thing
}

/// Whether the place named here is the subject of the question before it.
pub fn _the_country_was_the_question(memory: &Memory, name: &str) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    let said = &last.user;
    if text::strip(said).is_empty() {
        return false;
    }
    if TRIP_WORDS.is_match(said) {
        return false;
    }
    if !A_FACT_OF_THE_COUNTRY.is_match(said) {
        return false;
    }
    let before = _find_in(said, &WHERE_TO_GO).or_else(|| _find_in(said, &COUNTRY_ALIASES));
    match before {
        None => false,
        Some(b) if b == name => false,
        Some(_) => true,
    }
}

/// "Where should i go in spain": the picks, and the reason for each one.
pub fn _where_to_go(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    let name = _find_in(clean, &WHERE_TO_GO).or_else(|| _find_in(clean, &COUNTRY_ALIASES))?;
    if _a_thing_not_a_place(clean, name) {
        return None;
    }
    if _us_is_a_pronoun(clean, Some(name)) {
        return None;
    }
    if !TRIP_WORDS.is_match(clean) && _the_country_was_the_question(memory, name) {
        return None;
    }
    let country = COUNTRY_ALIASES.get(name).map(String::as_str).unwrap_or(name);
    let (label, body) = WHERE_TO_GO.at(country);
    Some(format!("{label}: {body}"))
}

/// Whether the "us" this line found is the country or the people in it.
pub fn _us_is_a_pronoun(clean: &str, name: Option<&str>) -> bool {
    if text::strip(name.unwrap_or("")).to_lowercase() != "us" {
        return false;
    }
    !_US_AS_A_PLACE.is_match(clean)
}

/// "We are going to greece in july": the country's picks, since nowhere in it is named.
pub fn _country_trip(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    let name = _find_in(clean, &WHERE_TO_GO).or_else(|| _find_in(clean, &COUNTRY_ALIASES));
    if _us_is_a_pronoun(clean, name) {
        return None;
    }
    // `WHERE_TO_GO[COUNTRY_ALIASES.get(name, name)]` with no name is a KeyError.
    let name = name.unwrap_or_else(|| panic!("KeyError: None"));
    let (label, body) = WHERE_TO_GO.at(COUNTRY_ALIASES.get(name).map(String::as_str).unwrap_or(name));
    Some(format!("{label}: {body}"))
}

// ---------------------------------------------------------------------------------------------
// Finding a place in a line

/// "Is bath worth visiting": the verdict question with the place inside it.
pub fn _worth_a_visit(text: &str, name: &str) -> bool {
    let name = pyre::escape(name);
    let pattern = format!(
        r"(?<![a-z]){name}\s+(?:is\s+)?worth|\bis\s+{name}\s+worth|\bworth\s+(?:visiting|seeing|going to)\s+{name}\b"
    );
    rx(&pattern, 0).is_match(text)
}

/// Whether a name that is also an everyday word ("bath", "split") is said as the place.
pub fn _names_place(text: &str, name: &str) -> bool {
    if !AMBIGUOUS_PLACES.iter().any(|p| p == name) {
        return true;
    }
    let shape = _PLACE_SHAPE.replacen("%s", &pyre::escape(name), 1);
    if rx(&shape, 0).is_match(text) {
        return true;
    }
    _worth_a_visit(text, name)
}

/// The longest key of the table that the text names as a place, or None.
pub fn _find_in<'t, V>(text: &str, table: &'t Table<V>) -> Option<&'t str> {
    table.longest_first().iter().find(|name| bounded_in(text, name) && _names_place(text, name)).map(String::as_str)
}

/// "What is the capital of X": the capital, with the usual wrong guess put right.
pub fn _capital(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = clean_of(understood);
    if !CAPITAL_WORD.is_match(said) {
        return None;
    }
    let country = _find_in(said, &CAPITALS)?;
    Some(CAPITALS.at(country).clone())
}

/// The table's place named in the line, or in the user's last two lines.
pub fn _place_in<V>(memory: &Memory, understood: &Understood, table: &Table<V>) -> Option<String> {
    let clean = clean_of(understood);
    if let Some(name) = _find_in(clean, table) {
        if table.name == "WHERE_TO_GO" && _a_thing_not_a_place(clean, name) {
            return None;
        }
        return Some(name.to_string());
    }
    for turn in last_turns(memory, 2).iter().rev() {
        if let Some(name) = _find_in(&turn.user.to_lowercase(), table) {
            return Some(name.to_string());
        }
    }
    None
}

// ---------------------------------------------------------------------------------------------
// When: months and seasons said in a line

/// The months (as numbers) and seasons a text names. `loose` also takes a bare month word.
pub fn _time_terms(text_in: &str, loose: bool) -> (BTreeSet<i64>, BTreeSet<String>) {
    if text::strip(text_in).is_empty() {
        return (BTreeSet::new(), BTreeSet::new());
    }
    let low = text::squash(&text_in.to_lowercase());
    let words: HashSet<String> = tokens(&low).into_iter().collect();
    let mut months: BTreeSet<i64> = _MONTH_IN_TEXT.finditer(&low).iter().map(|m| *MONTHS.at(m.str(1))).collect();
    if loose {
        for word in MONTHS.keys() {
            if words.contains(word) && !AMBIGUOUS_MONTHS.contains(word) {
                months.insert(*MONTHS.at(word));
            }
        }
    }
    for word in MONTHS.keys() {
        if word.chars().count() > 3 && has_word(text_in, &text::capitalize(word)) {
            months.insert(*MONTHS.at(word));
        }
    }
    let mut seasons: BTreeSet<String> = _SEASON_IN_TEXT.finditer(&low).iter().map(|m| m.str(1).to_string()).collect();
    if low.contains("high summer") {
        seasons.insert("high summer".into());
    }
    for m in _MONTH_RANGE.finditer(&low) {
        months.extend(_between(*MONTHS.at(m.str(1)), *MONTHS.at(m.str(2))));
    }
    (months, seasons)
}

/// The months from one to the other, going round the year.
pub fn _between(first: i64, last: i64) -> BTreeSet<i64> {
    let mut months = BTreeSet::new();
    let mut number = first;
    for _ in 0..12 {
        months.insert(number);
        if number == last {
            break;
        }
        number = number.rem_euclid(12) + 1;
    }
    months
}

/// When the person is going: the months and seasons of this line, the plan, and the last turns.
pub fn _when_said(memory: &Memory, understood: &Understood) -> (BTreeSet<i64>, BTreeSet<String>) {
    let (mut months, mut seasons) = _time_terms(clean_of(understood), true);
    if let Some(planned) = the_plan(memory) {
        let (planned_months, planned_seasons) = _time_terms(&planned, true);
        if !planned_months.is_empty() || !planned_seasons.is_empty() {
            return (&months | &planned_months, &seasons | &planned_seasons);
        }
    }
    for turn in last_turns(memory, 2).iter().rev() {
        let (more_months, more_seasons) = _time_terms(&turn.user, true);
        months.extend(more_months);
        seasons.extend(more_seasons);
    }
    (months, seasons)
}

/// Whether a sentence's own season (if any) is the time they said (if any).
pub fn _season_fits(sentence: &str, when: &(BTreeSet<i64>, BTreeSet<String>)) -> bool {
    let (stated_months, stated_seasons) = when;
    if stated_months.is_empty() && stated_seasons.is_empty() {
        return true;
    }
    let (months, seasons) = _time_terms(sentence, false);
    if months.is_empty() && seasons.is_empty() {
        return true;
    }
    if !months.is_disjoint(stated_months) {
        return true;
    }
    for season in &seasons {
        let in_season = SEASON_MONTHS.get(season).unwrap_or_else(|| panic!("KeyError: {season}"));
        if !in_season.is_disjoint(stated_months) {
            return true;
        }
        if stated_months.is_empty() && stated_seasons.contains(season) {
            return true;
        }
    }
    false
}

/// The text without its last sentence when that sentence is about another time of year.
pub fn _without_the_season(text_in: &str, when: &(BTreeSet<i64>, BTreeSet<String>)) -> String {
    let sentences = re!(r"(?<=[.!?]) ").split(text::strip(text_in));
    if sentences.len() < 2 || _season_fits(sentences.last().unwrap(), when) {
        return text_in.to_string();
    }
    sentences[..sentences.len() - 1].join(" ")
}

/// The table's place named in the plan the store keeps (or the plans before it).
pub fn _place_in_the_plan<V>(memory: &Memory, table: &Table<V>) -> Option<String> {
    let user = user_dict(memory)?;
    if let Some(current) = the_plan(memory) {
        if !text::strip(&current).is_empty() {
            if let Some(name) = _find_in(&current.to_lowercase(), table) {
                return Some(name.to_string());
            }
            if _find_in(&current.to_lowercase(), &CITIES).is_some() {
                return None;
            }
        }
    }
    let plans: Vec<Value> = match user.get("plans") {
        Some(Value::List(p)) => p.to_vec(),
        _ => vec![user.get("plan").cloned().unwrap_or(Value::None)],
    };
    for plan in plans.iter().rev() {
        let Some(plan) = plan.as_str() else { continue };
        if text::strip(plan).is_empty() {
            continue;
        }
        if let Some(name) = _find_in(&plan.to_lowercase(), table) {
            return Some(name.to_string());
        }
    }
    None
}

/// Cheap eats for the city named, or the three habits that make any city cheap to eat in.
pub fn _cheap_eats(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_or_text(understood).to_lowercase();
    let mut city = CHEAP_EATS.keys().iter().find(|one| clean.contains(one.as_str())).cloned();
    if city.is_none() {
        city = _place_in_the_plan(memory, &CHEAP_EATS);
    }
    if let Some(city) = city {
        return Some(CHEAP_EATS.at(&city).clone());
    }
    if clean.contains("eat there") {
        return None;
    }
    Some(concat!(
        "Three habits make any city cheap to eat in: take the fixed lunch rather than ",
        "dinner - menu del d\u{ed}a, plat du jour, prato do dia, Mittagsmen\u{fc} - which is the same ",
        "kitchen at half the price; eat where people work and shop, at a market hall, a ",
        "bakery counter or a busy bar, rather than where they sightsee; and stand up to eat, ",
        "because pizza by the slice, a kebab, a bowl of noodles at a counter or a picnic ",
        "from a supermarket costs a fraction of a table with a view. Coffee and breakfast at ",
        "a bar, and tap water in a bottle you refill, save the rest."
    ).into())
}

fn any_time_terms(text_in: &str) -> bool {
    let (m, s) = _time_terms(text_in, true);
    !m.is_empty() || !s.is_empty()
}

/// The city guide: what to see in the city the line names.
pub fn _guide(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    if _WORTH_ASK.is_match(clean) && any_time_terms(clean) {
        return None;
    }
    if BOOK_ASK.is_match(clean) {
        return None;
    }
    if _EAT_ASK.is_match(clean) && _find_in(clean, &CHEAP_EATS).is_some() {
        return None;
    }
    let Some(city) = _find_in(clean, &CITIES) else {
        if SIGHTSEEING_ASK.is_match(clean) {
            return _guide_next(memory, understood);
        }
        return None;
    };
    let body = _without_the_season(CITIES.at(city), &_when_said(memory, understood));
    if _already_guided(memory, city) && !understood.asks.is_empty() {
        let said = d2_shared::or3(&understood.plain, &understood.text, clean).to_lowercase();
        if _said_before_in_the_chat(memory, clean, Some(understood)) {
            return Some(format!("In {}: {body}", title(city)));
        }
        if _find_in(&said, &CITIES) == Some(city) && _asked_for_the_guide_before(memory, city, clean) {
            return Some(format!("In {}: {body}", title(city)));
        }
        return _about_city(city, memory, clean, Some(understood));
    }
    Some(format!("In {}: {body}", title(city)))
}

/// The words of a line that carry its ask: no filler, no city names.
pub fn _the_ask_words(text_in: &str) -> BTreeSet<String> {
    const PUNCT: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
    let mut words = BTreeSet::new();
    let low = text_in.to_lowercase().replace('?', " ").replace(',', " ");
    for word in text::split_ref(&low) {
        let word = text::strip_chars(word, PUNCT);
        if word.chars().count() < 3 || !text::is_alpha(word) {
            continue;
        }
        if NOT_THE_ASK_WORDS.contains(word) || CITIES.contains(word) {
            continue;
        }
        words.insert(word.to_string());
    }
    words
}

/// Whether the same ask (in its own words) was already put in this chat.
pub fn _said_before_in_the_chat(memory: &Memory, clean: &str, understood: Option<&Understood>) -> bool {
    let line = match understood {
        Some(u) => d2_shared::or3(&u.text, &u.plain, clean),
        None => clean,
    };
    let plan_words = _the_plan_words(memory);
    let mine: BTreeSet<String> = _the_ask_words(line).difference(&plan_words).cloned().collect();
    if mine.is_empty() {
        return false;
    }
    for turn in &memory.turns {
        if turn.bot.is_empty() {
            continue;
        }
        let theirs: BTreeSet<String> = _the_ask_words(&turn.user).difference(&plan_words).cloned().collect();
        if theirs == mine {
            return true;
        }
    }
    false
}

/// The ask words of the plans the store keeps.
pub fn _the_plan_words(memory: &Memory) -> BTreeSet<String> {
    let Some(user) = user_dict(memory) else { return BTreeSet::new() };
    let mut plans: Vec<String> = Vec::new();
    if let Some(p) = user.get("plan") {
        if p.truthy() {
            plans.push(py_str(p));
        }
    }
    if let Some(ps) = user.get("plans") {
        for p in ps.items() {
            if p.truthy() {
                plans.push(py_str(p));
            }
        }
    }
    let mut words = BTreeSet::new();
    for plan in &plans {
        words.extend(_the_ask_words(plan));
    }
    words
}

/// This line and the user's last two, newest first.
pub fn _the_lines_said(memory: &Memory, understood: &Understood) -> Vec<String> {
    let mut said = vec![understood.clean.clone()];
    for turn in last_turns(memory, 2).iter().rev() {
        said.push(turn.user.clone());
    }
    said
}

/// The lines (lowered) that name the place.
pub fn _lines_about(place: &str, said: &[String]) -> Vec<String> {
    said.iter().filter(|l| !l.is_empty()).map(|l| l.to_lowercase()).filter(|l| l.contains(place)).collect()
}

/// Whether the chat says the person is going to the place, rather than living in it.
pub fn _a_trip_to(place: &str, memory: &Memory, understood: &Understood) -> bool {
    let said = _the_lines_said(memory, understood);
    let low = _lines_about(place, &said);
    if low.iter().any(|line| _lives_in(place, line)) {
        return false;
    }
    if low.iter().any(|line| STAYS_THERE.is_match(line)) {
        return true;
    }
    if said.iter().filter(|l| !l.is_empty() && l.to_lowercase().contains(place)).any(|line| going_there(place, line)) {
        return true;
    }
    if ASKS_A_LENGTH.is_match(&understood.clean) {
        let since: Vec<String> = last_turns(memory, 3).iter().rev().map(|t| t.user.to_lowercase()).collect();
        if since.iter().any(|line| line.contains(place))
            && !since
                .iter()
                .any(|line| LIVES_THERE.finditer(line).iter().any(|f| char_window(line, f.end(), 40).contains(place)))
        {
            return true;
        }
    }
    false
}

/// Whether the line says somebody lives in (or works in, grew up in) the place.
pub fn _lives_in(place: &str, line: &str) -> bool {
    let low = line.to_lowercase();
    LIVES_THERE.finditer(&low).iter().any(|m| char_window(&low, m.end(), 40).contains(place))
}

static STATED_STAY: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        r"\b(?:\d+|one|two|three|four|five|six|seven|eight|nine|ten|a few|a couple of|several)\s+(?:full\s+)?(?:days|nights|day|night)\b",
        0,
    )
});
static ASKING_STAY: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(?:enough|too long|too short|too much|worth it|worth|do i need|should i|should we|do we need)\b", 0));

/// How many days a city wants, when the line asks how long to stay.
pub fn _stay_length(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    if A_JOURNEY_OR_A_BAG.is_match(clean) {
        return None;
    }
    let stated = STATED_STAY.is_match(clean);
    let asking = ASKING_STAY.is_match(clean);
    if stated && !asking {
        return None;
    }
    if !asking && !re!(r"\b(?:how long|how many days|how many nights|how much time)\b").is_match(clean) {
        return None;
    }
    let Some(place) = _place_in(memory, understood, &CITY_DAYS) else {
        return _country_stay_length(memory, understood);
    };
    if !_a_trip_to(&place, memory, understood) {
        let pattern = format!(r"\b(?:in|at|to see|to visit|around|for)\s+{}", pyre::escape(&place));
        if !(_find_in(clean, &CITY_DAYS).is_some() && rx(&pattern, 0).is_match(clean)) {
            return None;
        }
        let said = _the_lines_said(memory, understood);
        if _lines_about(&place, &said).iter().any(|line| _lives_in(&place, line)) {
            return None;
        }
    }
    Some(_stay_days(&place))
}

/// The cities of the stay-length table that a country's card names, in the order it names them.
pub fn _the_places_the_country_card_names(country: &str) -> Vec<String> {
    let body = WHERE_TO_GO.get(country).map(|(_, b)| b.to_lowercase()).unwrap_or_default();
    let mut found: Vec<(usize, String)> = Vec::new();
    for name in CITY_DAYS.keys() {
        if let Some(at) = bounded_find(&body, name) {
            found.push((text::char_index(&body, at), name.clone()));
        }
    }
    found.sort();
    found.into_iter().map(|(_, n)| n).collect()
}

const NUMBER_TEN: [&str; 10] = ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten"];

/// How long to give a country: its first two cities' days, and the journey between them.
pub fn _country_stay_length(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    if A_JOURNEY_OR_A_BAG.is_match(clean) {
        return None;
    }
    let country = _place_in(memory, understood, &WHERE_TO_GO)?;
    if _place_in(memory, understood, &CITY_DAYS).is_some() {
        return None;
    }
    if re!(r"\b(?:there|that city|this city|the city)\b", pyre::I).is_match(clean) {
        return None;
    }
    let typed = understood.text.to_lowercase();
    if !typed.is_empty() && !typed.contains(country.as_str()) {
        return None;
    }
    if !(_a_trip_to(&country, memory, understood)
        || (ASKS_A_LENGTH.is_match(clean) && clean.to_lowercase().contains(country.as_str())))
    {
        return None;
    }
    if LIVES_THERE.finditer(clean).iter().any(|f| char_window(clean, f.end(), 40).contains(country.as_str())) {
        return None;
    }
    let places = _the_places_the_country_card_names(&country);
    if places.is_empty() {
        return None;
    }
    let label = &WHERE_TO_GO.at(&country).0;
    if places.len() == 1 {
        return Some(_stay_days(&places[0]));
    }
    let (first, second) = (&places[0], &places[1]);
    let both = CITY_DAYS.at(first) + CITY_DAYS.at(second);
    let mut out = format!(
        "{label}: {} {}, {} {}, so {} days in all with the journey between them",
        title(first),
        _full_days(*CITY_DAYS.at(first)),
        title(second),
        py_idx(&NUMBER_TEN, CITY_DAYS.at(second) - 1),
        py_idx(&NUMBER_TEN, both - 1),
    );
    if places.len() > 2 && CITY_DAYS.contains(&places[2]) {
        out += &format!(", and a day or two more for {}", title(&places[2]));
    }
    Some(out + ".")
}

/// "three full days", in words.
pub fn _full_days(count: i64) -> String {
    if count == 1 {
        return "one full day".into();
    }
    if count < NUMBER_TEN.len() as i64 {
        return format!("{} full days", py_idx(&NUMBER_TEN, count - 1));
    }
    format!("{count} full days")
}

/// How long the city wants, not counting the travel days.
pub fn _stay_days(place: &str) -> String {
    let days = *CITY_DAYS.at(place);
    let length = if days == 1 { "One full day".to_string() } else { format!("{} full days", title(py_idx(&NUMBER_TEN, days - 1))) };
    let mut out = format!(
        "{length} in {}, not counting the day you arrive or the day you fly home: that covers the main sights at a normal pace, with time to sit down for lunch",
        title(place)
    );
    if DAY_TRIPS.contains(place) {
        out += ", and a day more if you want one of the day trips";
    }
    out + "."
}

/// "How long should we stay there": the city in the chat, when the line names none itself.
pub fn _stay_there(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    if _find_in(clean, &CITY_DAYS).is_some() {
        return None;
    }
    let place = _place_in(memory, understood, &CITY_DAYS)?;
    if !_a_trip_to(&place, memory, understood) {
        return None;
    }
    Some(_stay_days(&place))
}

// ---------------------------------------------------------------------------------------------
// Plugs, packing, water

/// The plug the country named takes, and whether a UK plug needs an adapter there.
pub fn _adapter(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let place = _place_in(memory, understood, &PLUG_TYPES)?;
    let (letter, shape, note) = PLUG_TYPES.at(&place);
    if letter == "G" {
        return Some(format!(
            concat!(
                "{} uses the same three square pins as the UK, so what you bring ",
                "from home fits. Take an adapter only for anything with a European plug on it, ",
                "and check that the voltage is the same, which it is in most of these places."
            ),
            title(&place)
        ));
    }
    let mut out = format!("In {} the sockets are type {letter}, {shape}, so a UK plug needs an adapter", title(&place));
    if !note.is_empty() {
        out += ". ";
        out += note;
    }
    Some(
        text::rstrip_chars(&out, ".").to_string()
            + concat!(
                ". Phone and laptop chargers handle 100 to 240 volts; hairdryers and straighteners ",
                "often do not, so look at the label before you pack them."
            ),
    )
}

/// What to pack for a city break, and the winter version.
pub fn _city_pack(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    if !re!(r"\b(?:cold|winter|snow|snowing|freezing|december|january|february|november)\b").is_match(clean) {
        return Some(concat!(
            "Take what you can walk in all day and layer: one pair of comfortable shoes, a ",
            "warm mid-layer for the evening, a light waterproof you can fold into a bag, and ",
            "a small day bag. One evening outfit is enough, plus a charger and your tickets."
        ).into());
    }
    Some(concat!(
        "Pack for 2 to 8C, wind and rain: a waterproof coat with a hood, warm layers you can ",
        "take off indoors, and shoes you can walk a hill in. It gets dark around 4pm in ",
        "December, so a small torch and plans for the afternoon indoors help."
    ).into())
}

/// Water and food for a day on the hill, or the kit to carry.
pub fn _hike_day(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    if re!(r"\b(?:every day|everyday|daily|most days|each day)\b").is_match(clean) && !_HILL.is_match(clean) {
        return Some(String::new());
    }
    let water = concat!(
        "half a litre an hour of walking is the working rule, so two litres for a half day ",
        "out and three or four for a full one, more in heat or on a climb, and carry it since ",
        "a stream is not safe to drink but a tap or a cafe will fill you up"
    );
    let food = concat!(
        "about 60g of carbohydrate an hour - sandwiches, oatcakes, nuts, dried fruit, a ",
        "flapjack - eaten little and often rather than as one big lunch, with something salty ",
        "for what you sweat out"
    );
    if !re!(r"\b(?:how much|how many|how long)\b").is_match(clean)
        && re!(r"\b(?:pack|packing|packed|bring|take|carry|wear|kit|bag|rucksack|backpack)\b").is_match(clean)
        && re!(r"\b(?:what|which|anything|need|should)\b").is_match(clean)
    {
        let kit = concat!(
            "Layers, a waterproof, and the water and food: base layer, fleece, a shell you ",
            "can get into in the wind, spare socks; half a litre an hour, so two litres for a ",
            "half day; 60g of carbohydrate an hour. Add a map or a charged phone with the ",
            "route, a torch, hat and gloves, a blister kit, and a foil blanket."
        );
        if re!(r"\b(?:pack|packing|packed|bring|carry|wear|kit|bag|rucksack|backpack)\b").is_match(clean) {
            return Some(format!("{kit} Leave the jeans and the cotton at home."));
        }
    }
    if re!(r"\b(?:eat|eating|ate|food|snack|snacks|lunch|fuel|energy|hungry|carb|carbs|carbohydrate)\b").is_match(clean)
        && !re!(r"\b(?:water|drink|drinking|liquid|litre|litres)\b").is_match(clean)
    {
        return Some(format!("{}. For the drink, {water}.", upper_first(food)));
    }
    Some(format!("{}. For food, aim at {food}.", upper_first(water)))
}

/// Whether the tap water is safe in the place named, or at home.
pub fn _tap_water(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    if re!(concat!(
        r"\b(?:liquids|bottle|bottles|hand luggage|cabin bag|security|plane|flight|",
        r"fountain|how much|more water|enough water|glasses|dehydrat|hot water|",
        r"boil|watering|water the|water my|water it|water them|bill|leak|pressure|",
        r"heater|plant|plants|houseplant|houseplants|leaves|leaf)\b"
    ))
    .is_match(clean)
    {
        return None;
    }
    if !re!(r"\b(?:tap|safe|drink|drinking|ice|okay|ok|fine|stomach|ill)\b").is_match(clean) {
        return None;
    }
    let mut place = _place_in(memory, understood, &WATER);
    let mut city = None;
    if place.is_none() {
        city = _place_in(memory, understood, &CITY_COUNTRY);
        let country = city.as_deref().and_then(|c| CITY_COUNTRY.get(c)).cloned().unwrap_or_default();
        if WATER.contains(&country) {
            place = Some(country);
        }
    }
    let Some(place) = place else {
        if re!(r"\b(?:abroad|holiday|holidays|trip|travelling|traveling|travel)\b").is_match(clean) {
            return Some(TAP_WATER_GENERAL.clone());
        }
        if !re!(r"\b(?:tap|taps|mains|bottled|safe|water company|water supply)\b").is_match(clean) {
            return None;
        }
        if re!(r"\b(?:brown|yellow|orange|rust|rusty|gritty)\b").is_match(clean) {
            return Some(TAP_WATER_HOME_BROWN.clone());
        }
        if re!(r"\b(?:smell|smells|smelled|taste|tastes|tasted|chlorine|metallic|lead|discoloured|discolored)\b").is_match(clean) {
            return Some(TAP_WATER_HOME_TASTE.clone());
        }
        return Some(TAP_WATER_HOME.clone());
    };
    let label = title(city.as_deref().unwrap_or(&place));
    if *WATER.at(&place) {
        return Some(format!(
            concat!(
                "Tap water in {label} is safe to drink: it is treated, the restaurants serve it, ",
                "and a refillable bottle saves money all week. Bottled is only worth it if the ",
                "plumbing in an old building makes the taste odd."
            ),
            label = label
        ));
    }
    Some(format!(
        concat!(
            "In {label}, drink bottled or filtered water, and skip the ice from street ",
            "stalls: the local water upsets visitors far more than locals. Use bottled for ",
            "brushing your teeth, and eat fruit you peel yourself for the first few days."
        ),
        label = label
    ))
}

// ---------------------------------------------------------------------------------------------
// Clocks: the times a line says, and when to be at the airport

/// A time said in a line: the hour (24h when am/pm was said), the minute, the am/pm, whether it
/// is when they leave or when they fly, and the separator it was written with.
#[derive(Clone, Debug, PartialEq)]
pub struct Clock {
    pub hour: i64,
    pub minute: i64,
    pub meridiem: String,
    pub role: String,
    pub sep: String,
}

static A_LENGTH_AFTER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s*(?:hours?|minutes?|mins?|days?|weeks?|nights?)\b", 0));

/// Every clock time the text says, each read as a leaving time or a flight time.
pub fn _clock_pairs(text_in: &str) -> Vec<Clock> {
    let mut found = Vec::new();
    let mut seen: HashSet<(i64, i64, String)> = HashSet::new();
    for pattern in [*_CLOCK_AFTER, *_CLOCK_MERIDIEM] {
        for m in pattern.finditer(text_in) {
            let Ok(hour) = m.str(1).parse::<i64>() else { continue };
            let minute: i64 = m.get(2).map(|s| s.parse().unwrap_or(0)).unwrap_or(0);
            let meridiem = m.str(3).to_lowercase();
            if hour > 23 || minute > 59 || (hour > 12 && meridiem.is_empty() && m.get(2).is_none()) {
                continue;
            }
            if A_LENGTH_AFTER.match_(&text_in[m.end()..]).is_some() {
                continue;
            }
            let key = (hour, minute, meridiem.clone());
            if seen.contains(&key) {
                continue;
            }
            seen.insert(key);
            let head = chars_before(text_in, m.start(), 34);
            let tail = char_window(text_in, m.end(), 26);
            let near_head = last_chars(head, 16);
            let role = if _LEAVE_SAID.is_match(near_head) && !re!(r"\b(?:miss|make|catch)\b").is_match(near_head) {
                "leave"
            } else if _FLIGHT_SAID.is_match(near_head) || _FLIGHT_SAID.is_match(first_chars(tail, 16)) {
                "flight"
            } else if _nearest(&_LEAVE_SAID, head, tail) >= 0 {
                "leave"
            } else {
                "flight"
            };
            let mut hour = hour;
            if !meridiem.is_empty() {
                hour = hour % 12 + if meridiem == "pm" { 12 } else { 0 };
            }
            let sep = if m.as_str().contains(':') { ":" } else { "." };
            found.push(Clock { hour, minute, meridiem, role: role.into(), sep: sep.into() });
        }
    }
    found
}

/// How near the pattern is said to a time: characters back into the head or on into the tail.
pub fn _nearest(pattern: &Re, head: &str, tail: &str) -> i64 {
    let mut best: i64 = -1;
    let head_len = text::len(head) as i64;
    for m in pattern.finditer(head) {
        let distance = head_len - text::char_index(head, m.start()) as i64;
        if best < 0 || distance < best {
            best = distance;
        }
    }
    for m in pattern.finditer(tail) {
        let distance = text::char_index(tail, m.start()) as i64 + 1;
        if best < 0 || distance < best {
            best = distance;
        }
    }
    best
}

static HOUR_AND_A_HALF: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:an?\s+hour|hour)\s+and\s+a\s+half\b", 0));
static HALF_AN_HOUR: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhalf\s+an?\s+hour\b", 0));
static AND_MINUTES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:and\s*)?(\d{1,2})\s*(?:minutes?|mins?)\b", 0));

/// A length of time the text says, in minutes.
pub fn _length_said(text_in: &str) -> Option<i64> {
    if HOUR_AND_A_HALF.is_match(text_in) {
        return Some(90);
    }
    if HALF_AN_HOUR.is_match(text_in) {
        return Some(30);
    }
    if let Some(hours) = _LENGTH_SAID.search(text_in) {
        let word = hours.str(1);
        let count = if word == "a" || word == "an" {
            1
        } else if text::is_digit(word) {
            word.parse::<i64>().unwrap_or(0)
        } else {
            _SMALL_COUNT.get(word).copied().unwrap_or(0)
        };
        if count != 0 {
            let after = char_window(text_in, hours.end(), 20);
            let more = AND_MINUTES.search(after).map(|m| m.str(1).parse::<i64>().unwrap_or(0)).unwrap_or(0);
            return Some(count * 60 + more);
        }
    }
    if let Some(minutes) = _LENGTH_MINS.search(text_in) {
        return minutes.str(1).parse::<i64>().ok();
    }
    None
}

/// "an hour and a half", "three hours", "45 minutes".
pub fn _long_said(minutes: i64) -> String {
    if minutes < 60 {
        return format!("{minutes} minutes");
    }
    let (hours, extra) = (minutes.div_euclid(60), minutes.rem_euclid(60));
    let base = if hours == 1 {
        "an hour".to_string()
    } else if hours <= 12 {
        format!("{} hours", py_idx(&_NUMBER_WORD, hours - 1))
    } else {
        format!("{hours} hours")
    };
    if extra == 0 {
        return base;
    }
    if extra == 30 {
        return if hours == 1 { "an hour and a half".into() } else { format!("{} and a half hours", py_idx(&_NUMBER_WORD, hours - 1)) };
    }
    if extra == 15 {
        return if hours == 1 {
            "an hour and a quarter".into()
        } else {
            format!("{} and a quarter hours", py_idx(&_NUMBER_WORD, hours - 1))
        };
    }
    format!("{base} and {extra} minutes")
}

/// The hour and minute a number of minutes after (or before) a clock.
pub fn _at_a_time(clock: &Clock, minutes: i64) -> (i64, i64) {
    let total = (clock.hour * 60 + clock.minute + minutes).rem_euclid(24 * 60);
    (total.div_euclid(60), total.rem_euclid(60))
}

/// "a 6am flight", "an 8.30 flight".
pub fn _a_flight(clock_said: &str, noun: &str) -> String {
    let digits: String = clock_said.chars().take_while(|c| c.is_ascii_digit()).collect();
    let hour: i64 = if digits.is_empty() { 0 } else { digits.parse().unwrap_or(0) };
    let article = if [8, 11, 18].contains(&hour) { "an" } else { "a" };
    format!("{article} {clock_said}") + &if noun.is_empty() { String::new() } else { format!(" {noun}") }
}

/// A time written the way the person wrote theirs: "6am", "18:30" or "6.30".
pub fn _a_clock(hour: i64, minute: i64, clock: &Clock) -> String {
    if !clock.meridiem.is_empty() {
        return _clock(hour, minute);
    }
    if clock.sep == ":" {
        return format!("{:02}:{:02}", hour.rem_euclid(24), minute);
    }
    let tail = if minute != 0 { format!(".{minute:02}") } else { String::new() };
    let h = hour.rem_euclid(24);
    format!("{}{tail}", if h == 0 { 12 } else { h })
}

static LONG_HAUL: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"\b(?:long haul|long-haul|transatlantic|usa|america|the states|",
            r"canada|australia|new zealand|japan|china|asia|dubai|south africa|",
            r"long flight)\b"
        ),
        0,
    )
});
static CHECKED_BAG: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"\b(?:checked bag|checked luggage|bag to check|suitcase|hold bag|",
            r"check a bag|checked in bag|check my bag|check a suitcase|bag drop|",
            r"hold luggage|check luggage)\b"
        ),
        0,
    )
});
static HAND_ONLY: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\b(?:hand luggage|cabin bag|carry on|carry-on|no bag|no luggage|hand baggage)\b", 0)
});

/// When to be at the airport, worked out from the times and the bags the line says.
pub fn _airport_early(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    let text_l = clean.to_lowercase();
    let text_s = text_l.as_str();
    let long_haul = LONG_HAUL.is_match(text_s);
    let bag = CHECKED_BAG.is_match(text_s);
    let hand = HAND_ONLY.is_match(text_s);
    let (lead, lead_said): (i64, &str) = if long_haul {
        (180, "three hours before departure")
    } else if hand && !bag {
        (90, "ninety minutes before departure")
    } else {
        (120, "two hours before departure")
    };
    let (closes, shuts) = if hand && !bag {
        (
            concat!(
                "The gate closes 20 to 30 minutes before the flight, so being through security ",
                "45 minutes before it is the deadline that matters."
            ),
            "The gate shuts 20 to 30 minutes before the flight.",
        )
    } else {
        (
            concat!(
                "The bag drop closes 45 to 60 minutes before the flight and the gate 20 to 30, ",
                "so the bag drop is the real deadline, not the departure time."
            ),
            "Bag drop shuts 45 to 60 minutes before the flight and the gate 20 to 30.",
        )
    };
    let clocks = _clock_pairs(text_s);
    let leaving = clocks.iter().find(|c| c.role == "leave").cloned();
    let flying = clocks.iter().find(|c| c.role != "leave").cloned();
    let drive = if _DRIVE_SAID.is_match(text_s) { _length_said(text_s) } else { None };
    let says_before = re!(r"(?:before|ahead of)\s+(?:my|the|our|a|departure|boarding)\b").is_match(text_s);
    let planned = if drive.is_some() || !says_before { None } else { _length_said(text_s) };
    if re!(r"\b(?:land|lands|landing|arrive at the other|get in at)\b").is_match(text_s)
        && !re!(r"\b(?:airport|before|leave|early)\b").is_match(text_s)
    {
        return Some(String::new());
    }
    if _FLIGHT_LENGTH_ASK.is_match(text_s) {
        return Some(_flight_length(memory, understood));
    }
    if re!(r"\b(?:connecting|connection|connections|layover|stopover|transit|change planes|changing planes)\b").is_match(text_s) {
        if let Some(planned) = planned {
            let said = _long_said(planned);
            if planned >= 90 {
                return Some(format!(
                    concat!(
                        "{} at Heathrow is comfortable if both flights ",
                        "are on one ticket: the bag goes through and the walk and security again ",
                        "are 30 to 60 minutes. The shortest connection airlines sell there is 60 ",
                        "to 90 minutes."
                    ),
                    upper_first(&said)
                ));
            }
            return Some(format!(
                concat!(
                    "{} is under what I would want between flights at ",
                    "Heathrow: 60 to 90 minutes is the usual minimum on one ticket, and that ",
                    "leaves nothing for a late first leg. If the first flight is late the airline ",
                    "has to move you on, but only when both legs are on one booking."
                ),
                upper_first(&said)
            ));
        }
        return Some(concat!(
            "Allow 60 to 90 minutes at Heathrow between flights on one ticket, and two ",
            "hours if they are separate bookings or you have to collect a bag and check it ",
            "again. A ",
            "late first leg is what breaks a connection, so one ticket is worth paying a ",
            "little more for."
        ).into());
    }
    if re!(r"\b(?:cancel|cancelled|canceled|cancelling|refund|refunds|compensation|delay|delayed|voucher)\b").is_match(text_s) {
        return Some(String::new());
    }
    if re!(r"\b(?:book|books|booking|booked|cheap|cheaper|cheapest|price|prices|fare|fares|deal|deals|cost)\b").is_match(text_s) {
        return Some(String::new());
    }
    if re!(r"\b(?:food|foods|snack|snacks|eat|eating|meal|meals|sandwich|sandwiches|lunch|picnic|drink|drinks)\b").is_match(text_s)
        && re!(r"\b(?:take|bring|carry|pack|eat)\b").is_match(text_s)
    {
        return Some(String::new());
    }
    if re!(r"\b(?:cruise|ferry|ferries|boat|ship|train|trains|rail|eurostar|coach|bus|hotel|hostel|campsite|airport transfer|transfer)\b")
        .is_match(text_s)
    {
        return Some(String::new());
    }
    if clocks.is_empty()
        && re!(r"\b(?:miss|missed|missing|misses)\b.*\b(?:flight|plane)\b").is_match(text_s)
        && re!(r"\b(?:what happens|what do i do|missed|too late|no.?show|rebook|refund|claim|compensation|rebooked)\b").is_match(text_s)
    {
        return Some(String::new());
    }
    if re!(r"\b(?:gate|gates|boarding)\b").is_match(text_s) && !re!(r"\bboarding (?:pass|passes|card)\b").is_match(text_s) {
        if let Some(flying) = &flying {
            let (h, m) = _at_a_time(flying, -25);
            let shut = _a_clock(h, m, flying);
            let (h, m) = _at_a_time(flying, -40);
            let board = _a_clock(h, m, flying);
            return Some(format!(
                concat!(
                    "For {} the gate shuts about {} and boarding starts around {}: be in the ",
                    "departure lounge before that, which means through security an hour ahead."
                ),
                _a_flight(&_a_clock(flying.hour, flying.minute, flying), "flight"),
                shut,
                board
            ));
        }
        return Some(concat!(
            "Boarding starts about 40 minutes before departure and the gate shuts 20 to 30 ",
            "minutes before it, so be through security and in the departure lounge an hour ",
            "ahead. The gate's clock is the one that costs you the seat if you miss it."
        ).into());
    }
    if re!(r"\bcheck(?:ing)?[ -]?in\b|\bcheckin\b").is_match(text_s)
        && !re!(r"\b(?:what time should i leave|leave home)\b").is_match(text_s)
    {
        if re!(r"\bhow long\b").is_match(text_s) && re!(r"\b(?:take|takes|queue|wait)\b").is_match(text_s) {
            return Some(concat!(
                "Ten to twenty minutes at the desk with your bag ready and the documents out, ",
                "and more like half an hour at six in the morning. The desk shuts 45 to 60 ",
                "minutes before departure, so the queue is not what makes you late."
            ).into());
        }
        return Some(concat!(
            "Check in online as soon as it opens - 24 to 48 hours before the flight at most ",
            "airlines, and the pass appears in their app - and then the airport is only bag ",
            "drop and security. The desk shuts 45 to 60 minutes before departure, and 90 ",
            "minutes on some long-haul flights."
        ).into());
    }
    if let Some(planned) = planned {
        let said = _long_said(planned);
        if planned >= lead + 45 {
            return Some(format!(
                "That is plenty: {lead_said} is the safe rule, so {said} leaves room for the bag queue and a coffee. {shuts}"
            ));
        }
        if planned >= lead {
            let that = if planned == lead { "that".to_string() } else { said.clone() };
            return Some(format!(
                "That is about right: {lead_said} is the rule, and {that} works as long as the bag queue is short. {shuts}"
            ));
        }
        if planned >= lead - 30 {
            return Some(format!(
                "That is tight: {said} is under the {} I would allow, so a queue at the bags or at security is what makes you late. {shuts}",
                _long_said(lead)
            ));
        }
        return Some(format!(
            "Too tight unless you have no bag to drop and a boarding pass already: {said} leaves nothing for the queue at security, and {lead_said} is the safe rule. {shuts}"
        ));
    }
    if let (Some(leaving0), Some(flying)) = (&leaving, &flying) {
        let mut leaving = leaving0.clone();
        let road = drive.unwrap_or(30);
        let mut leave_at = leaving.hour * 60 + leaving.minute;
        let fly_at = flying.hour * 60 + flying.minute;
        if leaving.meridiem.is_empty() && !flying.meridiem.is_empty() {
            while fly_at - leave_at - road > 12 * 60 {
                leave_at += 12 * 60;
            }
            let hour = leave_at.div_euclid(60).rem_euclid(24);
            leaving.hour = hour;
            leaving.meridiem = if hour < 12 { "am".into() } else { "pm".into() };
        }
        let mut gap = fly_at - leave_at - road;
        if gap < -720 {
            gap += 24 * 60;
        }
        let said_flight = _a_clock(flying.hour, flying.minute, flying);
        let (h, m) = _at_a_time(&leaving, road);
        let there = _a_clock(h, m, &leaving);
        let sides = if drive.is_some() { format!("with {} on the road", _long_said(road)) } else { "with a half-hour drive".into() };
        let leaving_said = _a_clock(leaving.hour, leaving.minute, &leaving);
        if gap >= lead + 60 {
            let (h, m) = _at_a_time(flying, -(lead + road));
            let earlier = _a_clock(h, m, flying);
            return Some(format!(
                "You are early: leaving at {leaving_said} {sides} puts you at the airport for {there}, {} before {}, so {earlier} would still be there {} ahead. {shuts}",
                _long_said(gap),
                _a_flight(&said_flight, "flight"),
                _long_said(lead)
            ));
        }
        if gap >= lead {
            return Some(format!(
                "That works: leaving at {leaving_said} {sides} gets you there for {there}, {} before {}. {shuts}",
                _long_said(gap),
                _a_flight(&said_flight, "flight")
            ));
        }
        if gap >= lead - 30 {
            return Some(format!(
                "Cutting it fine: {there} is only {} before {} and I would allow {}. Leave half an hour earlier if you have a bag to drop. {shuts}",
                _long_said(gap),
                _a_flight(&said_flight, "flight"),
                _long_said(lead)
            ));
        }
        let (h, m) = _at_a_time(flying, -(lead + road));
        return Some(format!(
            "That is too late for {}: {there} leaves {} at the airport, and bag drop shuts 45 to 60 minutes before departure. Leave by {} at the latest.",
            _a_flight(&said_flight, "flight"),
            _long_said(gap),
            _a_clock(h, m, flying)
        ));
    }
    if let Some(leaving) = &leaving {
        let leaving_said = _a_clock(leaving.hour, leaving.minute, leaving);
        let head = if let Some(drive) = drive {
            let (h, m) = _at_a_time(leaving, drive);
            let there = _a_clock(h, m, leaving);
            format!("Leaving at {leaving_said} with {} on the road puts you at the airport for {there}, ", _long_said(drive))
        } else {
            let (h, m) = _at_a_time(leaving, 30);
            let there = _a_clock(h, m, leaving);
            format!("Leaving at {leaving_said} puts you at the airport for {there} if the drive is about half an hour, ")
        };
        let (h, m) = _at_a_time(leaving, drive.unwrap_or(30) + lead);
        let suits = _a_clock(h, m, leaving);
        return Some(format!(
            "{head}which is {} before {} - about right. A longer drive needs a later flight. {shuts}",
            _long_said(lead),
            _a_flight(&suits, "flight")
        ));
    }
    if let Some(flying) = &flying {
        let said = _a_clock(flying.hour, flying.minute, flying);
        let (h, m) = _at_a_time(flying, -lead);
        let there = _a_clock(h, m, flying);
        if let Some(drive) = drive {
            let (h, m) = _at_a_time(flying, -(lead + drive));
            let home = _a_clock(h, m, flying);
            return Some(format!(
                "For {}, be at the airport by {there}: {lead_said} is the safe rule, and with {} on the road that means leaving home around {home}. {shuts}",
                _a_flight(&said, "flight"),
                _long_said(drive)
            ));
        }
        let (h, m) = _at_a_time(flying, -(lead + 30));
        let home = _a_clock(h, m, flying);
        return Some(format!(
            "For {}, be at the airport by {there}: {lead_said} is the safe rule. Work back from that for the drive - half an hour on the road means leaving home around {home}. {shuts}",
            _a_flight(&said, "flight")
        ));
    }
    if re!(concat!(
        r"\b(?:leave home|leave the house|leave my house|leaving home|set off|",
        r"set out|get going|need to leave|should i leave|when do i leave|",
        r"leave for the airport|leaving for the airport|head off)\b"
    ))
    .is_match(text_s)
    {
        return Some(format!(
            "{} at the airport, and the drive comes on top: half an hour on the road means leaving home {} before the flight. {shuts}",
            upper_first(lead_said),
            _long_said(lead + 30)
        ));
    }
    Some(format!(
        "{} is the safe rule. {closes} Add half an hour at a big airport in the early morning or on a Friday.",
        upper_first(lead_said)
    ))
}

// ---------------------------------------------------------------------------------------------
// How long the flight is, and the flight they told me about

fn place_the_flight_starts_at(named: &str, clean: &str, turns: &[Turn]) -> bool {
    let mut texts = vec![clean.to_string()];
    let start = turns.len().saturating_sub(3);
    for turn in &turns[start..] {
        texts.push(turn.user.to_lowercase());
    }
    let pattern = rx(&format!(r"(.{{0,24}})\b{}", pyre::escape(named)), 0);
    for t in &texts {
        if let Some(m) = pattern.search(t) {
            if _STARTS_AT.is_match(m.str(1)) {
                return true;
            }
        }
    }
    false
}

/// Whether the place is where the flight leaves from ("from manchester"), not where it goes.
pub fn _a_place_the_flight_starts_at(named: &str, clean: &str, memory: &Memory) -> bool {
    place_the_flight_starts_at(named, clean, &memory.turns)
}

/// The flight-hours place said after the one the flight starts at.
pub fn _a_place_said_after(named: &str, clean: &str) -> Option<&'static str> {
    let rest = match clean.split_once(named) {
        Some((_, after)) => after,
        None => clean,
    };
    _find_in(rest, &AIR_HOURS_CITY)
}

/// The place the user's last lines fly to.
pub fn _the_destination_in_the_turns(memory: &Memory) -> Option<String> {
    for turn in last_turns(memory, 3).iter().rev() {
        let found = _find_in(&turn.user.to_lowercase(), &AIR_HOURS_CITY);
        if let Some(found) = found {
            if !place_the_flight_starts_at(found, "", std::slice::from_ref(turn)) {
                return Some(found.to_string());
            }
        }
    }
    None
}

/// How long the flight is to the place named (from the UK).
pub fn _flight_length(memory: &Memory, understood: &Understood) -> String {
    let clean = clean_of(understood);
    let mut named = _place_in(memory, understood, &AIR_HOURS_CITY);
    if let Some(n) = named.clone() {
        if _a_place_the_flight_starts_at(&n, clean, memory) {
            named = _a_place_said_after(&n, clean)
                .map(str::to_string)
                .or_else(|| _place_in_the_plan(memory, &AIR_HOURS_CITY))
                .or_else(|| _the_destination_in_the_turns(memory));
        }
    }
    let country = named.as_deref().and_then(|n| AIR_HOURS_CITY.get(n)).cloned().unwrap_or_default();
    if let Some(hours) = FLIGHT_HOURS.get(&country) {
        return _flight_hours_said(&title(named.as_deref().unwrap_or("")), *hours);
    }
    let city = _place_in(memory, understood, &CITY_COUNTRY).or_else(|| _place_in(memory, understood, &CITIES));
    let country = city.as_deref().and_then(|c| CITY_COUNTRY.get(c)).cloned().unwrap_or_default();
    if let Some(hours) = FLIGHT_HOURS.get(&country) {
        return _flight_hours_said(&title(city.as_deref().unwrap_or("")), *hours);
    }
    if let Some(place) = _place_in(memory, understood, &FLIGHT_HOURS) {
        return _flight_hours_said(&title(&place), *FLIGHT_HOURS.at(&place));
    }
    concat!(
        "The airline's timetable has the exact time, and the booking shows it to the minute. ",
        "As a rule the air time is two to four hours across Europe, seven to eight to the ",
        "east coast of America, and eleven to thirteen to Asia."
    )
    .into()
}

/// "About two and a half hours in the air from the UK to Rome".
pub fn _flight_hours_said(name: &str, hours: f64) -> String {
    let in_the_air = _long_said((hours * 60.0) as i64);
    if hours >= 20.0 {
        return format!(
            concat!(
                "To {} it is {} in the air from the UK even non-stop, and a day of ",
                "it with a stop: book a night flight if you can sleep on a plane, and the return ",
                "leg is longer still."
            ),
            name, in_the_air
        );
    }
    format!(
        concat!(
            "About {} in the air from the UK to {}, plus the airport at both ends, ",
            "so half a day door to door. The booking shows the exact time."
        ),
        in_the_air, name
    )
}

/// "What time is my flight": the time they told me, worked back to when to leave.
pub fn _my_flight_time(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    if re!(r"\b(?:land|lands|landing)\b").is_match(clean) {
        return Some(concat!(
            "The landing time is on your ticket and in the airline's app, and it is the local ",
            "time where you land rather than the time back home. It moves with the flight, so ",
            "the live arrival time in the app is the one to trust on the day."
        ).into());
    }
    if re!(r"\b(?:gate|terminal)\b").is_match(clean) {
        return Some(concat!(
            "The gate and terminal are on your boarding pass and in the airline's app, and ",
            "they can change on the day: the app and the departure boards after security are ",
            "the ",
            "ones to trust, and the gate is worth checking once more when you sit down."
        ).into());
    }
    if let Some((clock, day)) = _flight_told(memory) {
        let said = _a_clock(clock.hour, clock.minute, &clock);
        let (h, m) = _at_a_time(&clock, -120);
        let there = _a_clock(h, m, &clock);
        let (h, m) = _at_a_time(&clock, -150);
        return Some(format!(
            concat!(
                "You told me your flight is at {}{}, so be at the airport by {}: two ",
                "hours before departure is the safe rule. The drive comes on top, so half an ",
                "hour on the road means leaving home at ",
                "{}."
            ),
            said,
            day,
            there,
            _a_clock(h, m, &clock)
        ));
    }
    Some(concat!(
        "I can't see your booking, so I can't look it up: the time is on your ticket and in ",
        "the airline's app, in the local time of the airport you fly from. Tell me the time ",
        "and I'll work back to when you need to leave."
    ).into())
}

fn flight_clock_in(said: &str) -> Option<(Clock, String)> {
    if !_FLIGHT_SAID.is_match(said) {
        return None;
    }
    let clock = _clock_pairs(said).into_iter().find(|c| c.role == "flight")?;
    let day = _DAY_SAID.search(said).map(|d| format!(" on {}", title(d.str(1)))).unwrap_or_default();
    Some((clock, day))
}

/// The flight time the person told me (in the plan, or in their last lines), with its day.
pub fn _flight_told(memory: &Memory) -> Option<(Clock, String)> {
    if let Some(held) = _the_plan_told(memory) {
        return Some(held);
    }
    for turn in last_turns(memory, 3).iter().rev() {
        if let Some(found) = flight_clock_in(&turn.user.to_lowercase()) {
            return Some(found);
        }
    }
    None
}

/// The flight time in the plans the store keeps.
pub fn _the_plan_told(memory: &Memory) -> Option<(Clock, String)> {
    let user = user_dict(memory)?;
    let plans: Vec<Value> = match user.get("plans") {
        Some(Value::List(p)) => p.to_vec(),
        _ => vec![user.get("plan").cloned().unwrap_or(Value::None)],
    };
    for plan in plans.iter().rev() {
        let Some(plan) = plan.as_str() else { continue };
        if text::strip(plan).is_empty() {
            continue;
        }
        if let Some(found) = flight_clock_in(&plan.to_lowercase()) {
            return Some(found);
        }
    }
    None
}

/// "6am", "2.30pm".
pub fn _clock(hour: i64, minute: i64) -> String {
    let hanging = if hour.rem_euclid(24) < 12 { "am" } else { "pm" };
    let twelve = match hour.rem_euclid(12) {
        0 => 12,
        h => h,
    };
    let tail = if minute != 0 { format!(".{minute:02}") } else { String::new() };
    format!("{twelve}{tail}{hanging}")
}

/// Whether the UK is on summer time on the day (today by default).
pub fn _uk_summer(day: Option<(i64, i64, i64)>) -> bool {
    let (year, month, dom) = day.unwrap_or_else(|| {
        let now = now_utc();
        (now.year, now.month, now.day)
    });
    if (4..=9).contains(&month) {
        return true;
    }
    if month == 3 || month == 10 {
        let last = days_from_civil(year, month, 28) + 4;
        let last_sunday = last - (weekday(last) + 1);
        let today = days_from_civil(year, month, dom);
        return if month == 3 { today >= last_sunday } else { today < last_sunday };
    }
    false
}

fn uk_now() -> Moment {
    now_utc().plus_minutes(if _uk_summer(None) { 60 } else { 0 })
}

/// The clock there (hours ahead of the UK) and here, now.
pub fn _now_there(hours: f64) -> (String, String) {
    let here = uk_now();
    let there = here.plus_minutes((hours * 60.0).round() as i64);
    (_clock(there.hour, there.minute), _clock(here.hour, here.minute))
}

/// "What time is it": the UK clock, when the line names nowhere else.
pub fn _the_clock_here(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    if !_WHEN_IT_IS.is_match(clean) {
        return None;
    }
    if _HOW_OFTEN.is_match(clean) {
        return None;
    }
    if _TIME_TO_DO_SOMETHING.is_match(clean) {
        return None;
    }
    if _TIME_OF_DAY.is_match(clean) {
        return None;
    }
    if _place_in(memory, understood, &TIME_ZONES).is_some() || _find_in(clean, &MULTI_ZONES).is_some() {
        return None;
    }
    let here = uk_now();
    Some(format!(
        "It is {} here, on the UK clock - if you are somewhere else your own clock is the one to trust.",
        _clock(here.hour, here.minute)
    ))
}

/// Whether the line's asks leave room for a "where" answer.
pub fn _asks_where(understood: &Understood) -> bool {
    !understood.asks.iter().any(|a| _NOT_WHERE_ASKS.contains(a))
}

/// Which area of the city to stay in.
pub fn _where_to_stay(memory: &mut Memory, understood: &Understood) -> Option<String> {
    if !_asks_where(understood) {
        return None;
    }
    let place = _place_in(memory, understood, &CITY_AREA)?;
    Some(format!("Stay in {}.", CITY_AREA.at(&place)))
}

/// Whether the line asks where to stay or sleep.
pub fn _a_night_near_the_airport(understood: &Understood) -> bool {
    _A_STAY_ASK.is_match(clean_of(understood))
}

/// Where to stay in a town the city table does not hold.
pub fn _a_town_stay(memory: &mut Memory, understood: &Understood) -> Option<String> {
    if _place_in(memory, understood, &CITY_AREA).is_some() {
        return None;
    }
    Some(TOWN_STAY.clone())
}

/// Whether the line asks where to stay in a place the city table does not hold.
pub fn _a_town_stay_line(understood: &Understood) -> bool {
    if understood.asks.iter().any(|a| _NOT_WHERE_ASKS.contains(a)) {
        return false;
    }
    let clean = clean_of(understood);
    if _A_FLIGHT_WORD.is_match(clean) {
        return false;
    }
    if _A_CAR_NIGHT.is_match(clean) {
        return false;
    }
    if _A_SHIFT_WORK.is_match(clean) {
        return false;
    }
    if !_A_STAY_PLACE_ASK.is_match(clean) {
        return false;
    }
    _find_in(clean, &CITY_AREA).is_none()
}

const HOUR_WORDS: [&str; 14] = [
    "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve", "thirteen", "fourteen",
];

/// "four hours", "five and a half hours".
fn zone_size(hours: f64) -> String {
    let whole = hours.abs() as i64;
    if hours.abs() != whole as f64 {
        return format!("{} and a half hours", py_idx(&HOUR_WORDS, whole - 1));
    }
    if whole == 1 { format!("{} hour", py_idx(&HOUR_WORDS, whole - 1)) } else { format!("{} hours", py_idx(&HOUR_WORDS, whole - 1)) }
}

static GIVEN_TIME: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(\d{1,2})(?:[:.](\d{2}))?\s*(am|pm)\b", 0));

/// The time in the place named, and how far ahead or behind the UK it is.
pub fn _time_zone(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    if re!(concat!(
        r"\b(fly|flight|flying|flights|drive|driving|walk|walking|train|bus|queue|",
        r"check in|checking|passport|shops|dinner|kick off|kickoff|lunch|sunrise|",
        r"sunset|tablets|bed|check out|checkout)\b"
    ))
    .is_match(clean)
    {
        return None;
    }
    let Some(place) = _place_in(memory, understood, &TIME_ZONES) else {
        let wide = _find_in(clean, &MULTI_ZONES)?;
        return Some(format!("{} has more than one clock: {}.", title(wide), MULTI_ZONES.at(wide)));
    };
    let (winter, summer) = *TIME_ZONES.at(&place);
    let name = title(&place);
    let gap = if _uk_summer(None) { summer } else { winter };
    if let Some(given) = GIVEN_TIME.search(clean) {
        if re!(r"\bhere\b").is_match(clean) {
            let hour = given.str(1).parse::<i64>().unwrap_or(0) % 12 + if given.str(3) == "pm" { 12 } else { 0 };
            let minute = given.get(2).map(|m| m.parse::<i64>().unwrap_or(0)).unwrap_or(0);
            let there_given = _clock(hour + gap.trunc() as i64, minute + if gap.rem_euclid(1.0) != 0.0 { 30 } else { 0 });
            let colon = given.get(2).map(|m| format!(":{m}")).unwrap_or_default();
            return Some(format!("{}{colon}{} here is {there_given} in {name}.", given.str(1), given.str(3)));
        }
    }
    let (there, here) = _now_there(gap);
    if winter == 0.0 && summer == 0.0 {
        if ["uk", "the uk", "britain", "england", "scotland", "wales"].contains(&place.as_str()) {
            return Some(format!("It is {here} here, and the same clock in the UK."));
        }
        return Some(format!("{name} runs on the same clock as the UK, so it is {there} there now and {here} here."));
    }
    let asked_how_far = re!(r"hours|difference|ahead|behind|zone").is_match(clean);
    if winter == summer {
        if asked_how_far {
            return Some(format!(
                "{name} is {} {} the UK all year, so it is {there} there now and {here} here.",
                zone_size(winter),
                if winter > 0.0 { "ahead of" } else { "behind" }
            ));
        }
        return Some(format!("It is {there} in {name} now, and {here} here."));
    }
    if asked_how_far {
        return Some(format!(
            "{name} is {} ahead of the UK in winter and {} in summer, because its clocks do not move when ours do: it is {there} there now and {here} here.",
            zone_size(winter),
            zone_size(summer)
        ));
    }
    let way = if winter > 0.0 { "ahead of" } else { "behind" };
    Some(format!(
        "It is {there} in {name} now, and {here} here: {name} is {} {way} us at the moment.",
        if _uk_summer(None) { zone_size(summer) } else { zone_size(winter) }
    ))
}

/// The month the plan names, in full.
pub fn _plan_month(memory: &Memory) -> Option<String> {
    let plan = user_value(memory, "plan").filter(|v| v.truthy()).map(py_str).unwrap_or_default().to_lowercase();
    for name in MONTHS.longest_first() {
        if has_word(&plan, name) {
            return Some(py_idx(&_LONG_MONTH, MONTHS.at(name) - 1).clone());
        }
    }
    None
}

/// How far ahead to book, and by when for the month in the plan.
pub fn _when_to_book(memory: &Memory) -> String {
    let window = concat!(
        "six to eight weeks ahead for short-haul and two to four months for long-haul, ",
        "with Tuesday and Wednesday usually the cheapest days to fly"
    );
    let Some(month) = _plan_month(memory) else {
        return format!("Book {window}.");
    };
    let earlier = py_idx(&_LONG_MONTH, (MONTHS.at(&month) - 3).rem_euclid(12));
    format!("For a trip in {}, book by mid-{}: {window}.", title(&month), title(earlier))
}

/// The best time to go to the city or country named.
pub fn _best_time(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    if _WORTH_ASK.is_match(clean) && !any_time_terms(clean) {
        return None;
    }
    if BOOK_ASK.is_match(clean) && !BOOK_NOT_THE_TRIP.is_match(clean) {
        return Some(_when_to_book(memory));
    }
    if let Some(city) = _place_in(memory, understood, &WHEN_TO_GO) {
        return Some(format!("For {}: {}", title(&city), WHEN_TO_GO.at(&city)));
    }
    let place = _place_in(memory, understood, &BEST_TIME)?;
    Some(format!("For {}: {}", title(&place), BEST_TIME.at(&place)))
}

/// "In Cornwall", "On the Isle of Wight".
pub fn _region_lead(region: &str) -> String {
    if let Some(named) = REGION_NAMES.get(region) {
        return if named.starts_with("on") { upper_first(named) } else { format!("In {named}") };
    }
    format!("In {}", title(region))
}

/// What to do with days in a UK region the line names.
pub fn _region_days(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    if MONTHS.keys().iter().any(|name| has_word(&clean, name)) {
        return None;
    }
    let named = _find_in(&clean, &REGION_PLACES)?;
    let region = REGION_PLACES.at(named);
    if _A_BEACH.is_match(&clean) {
        return REGION_BEACHES.get(region).cloned();
    }
    Some(format!("{}: {}", _region_lead(region), REGION_DAYS.at(region)))
}

/// Arriving at a station: the fare, which station, getting about, arriving late.
pub fn _arriving(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    if d2_shared::any_in(&clean, &_FARES_ASK) && !d2_shared::any_in(&clean, &_FARES_GOING_SOMEWHERE) {
        return None;
    }
    if clean.contains("airport") || clean.contains("flight") || clean.contains("fly") {
        return None;
    }
    if d2_shared::any_in(&clean, &_ARRIVAL_FARE_ASK) {
        return Some(_ARRIVAL_FARE.clone());
    }
    if d2_shared::any_in(&clean, &_ARRIVAL_PICK) {
        return Some(_ARRIVAL_PICK_NOTE.clone());
    }
    if d2_shared::any_in(&clean, &_ARRIVAL_ABOUT) {
        return Some(_ARRIVAL_ABOUT_NOTE.clone());
    }
    if d2_shared::any_in(&clean, &_ARRIVAL_LATE) {
        return Some(_ARRIVAL_LATE_NOTE.clone());
    }
    Some(_ARRIVAL_TOWN.clone())
}

/// The way in from an airport the line does not name.
pub fn _airport_without_a_city(understood: &Understood) -> Option<String> {
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let text_j = words.join(" ");
    let has = |options: &[String]| options.iter().any(|o| words.contains(o) || text_j.contains(o.as_str()));
    if (text_j.contains("to the airport") || text_j.contains("to the terminal")) && !text_j.contains("from the airport") {
        return None;
    }
    if has(&_AIRPORT_PORT) {
        return Some(_AIRPORT_PORT_REPLY.clone());
    }
    if has(&_AIRPORT_HOTEL) {
        return Some(_AIRPORT_HOTEL_REPLY.clone());
    }
    if has(&_AIRPORT_CHEAPEST) {
        return Some(_AIRPORT_CHEAPEST_REPLY.clone());
    }
    if has(&_AIRPORT_TRAIN) {
        return Some(_AIRPORT_TRAIN_REPLY.clone());
    }
    if has(&_AIRPORT_FASTEST) {
        return Some(_AIRPORT_FASTEST_REPLY.clone());
    }
    Some(_AIRPORT_WAY_IN.clone())
}

/// From the airport into the city: the airport or city named, else the general way in.
pub fn _airport(memory: &mut Memory, understood: &Understood, general: bool) -> Option<String> {
    let clean = clean_of(understood);
    let mut place = _place_in(memory, understood, &AIRPORT_TRANSFER);
    if place.is_none() && clean.contains("airport") {
        for name in AIRPORT_TRANSFER.longest_first() {
            if clean.contains(name.replace(" airport", "").as_str()) {
                place = Some(name.clone());
                break;
            }
        }
    }
    let Some(place) = place else {
        return if general { _airport_without_a_city(understood) } else { None };
    };
    Some(format!("From {}: {}", title(&place), AIRPORT_TRANSFER.at(&place)))
}

/// A taxi from a foreign airport: the scam warning, or the transfer for the airport named.
pub fn _taxi_abroad(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    if d2_shared::any_in(clean, &_TAXI_TROUBLE) {
        return Some(_TAXI_WARNING.clone());
    }
    Some(_airport(memory, understood, false).filter(|s| !s.is_empty()).unwrap_or_else(|| _TAXI_WARNING.clone()))
}

/// The airport the city flies into.
pub fn _city_airport(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let city = _place_in(memory, understood, &AIRPORT_NAME)?;
    Some(AIRPORT_NAME.at(&city).clone())
}

/// The places of the route table the line names, longest name first.
pub fn _two_places(clean: &str) -> Vec<String> {
    // `sorted({name for pair in ROUTE_BETWEEN for name in pair}, key=len, reverse=True)`: Python
    // breaks length ties in set order (per-process hash seed); here they are in sorted order.
    let mut names: Vec<String> =
        ROUTE_BETWEEN.iter().flat_map(|((a, b), _)| [a.clone(), b.clone()]).collect::<BTreeSet<_>>().into_iter().collect();
    names.sort_by_key(|n| std::cmp::Reverse(n.chars().count()));
    names.into_iter().filter(|n| has_word(clean, n)).collect()
}

/// The way between two places the route table holds.
pub fn _route_between(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let names = _two_places(clean_of(understood));
    for first in &names {
        for second in &names {
            if first == second {
                continue;
            }
            for (a, b) in [(first, second), (second, first)] {
                if let Some((_, reply)) = ROUTE_BETWEEN.iter().find(|((x, y), _)| x == a && y == b) {
                    return Some(reply.clone());
                }
            }
        }
    }
    None
}

/// How to get to a place the get-to table holds.
pub fn _get_to(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    GET_TO.longest_first().iter().find(|p| has_word(clean, p)).map(|p| GET_TO.at(p).clone())
}

/// The note for a site the line names (Pompeii, the Vatican museums...).
pub fn _site_note(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    for site in SITE_WORDS.iter() {
        if has_word(clean, site) {
            let key = SITE_ALIASES.get(site).map(String::as_str).unwrap_or(site);
            return Some(SITE_NOTES.at(key).clone());
        }
    }
    None
}

/// The day trips from the city named.
pub fn _day_trip(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let city = _place_in(memory, understood, &DAY_TRIPS)?;
    Some(format!("Day trips from {}: {}", title(&city), DAY_TRIPS.at(&city)))
}

// ---------------------------------------------------------------------------------------------
// Day trips

/// The clauses of a day-trip list: split at commas (outside brackets) and at "and" before a
/// day-trip place.
pub fn _clauses(line: &str) -> Vec<String> {
    let chars: Vec<char> = format!(" {line} ").chars().collect();
    let sub = |a: usize, b: usize| -> String { chars[a.min(chars.len())..b.min(chars.len())].iter().collect() };
    let mut pieces: Vec<String> = Vec::new();
    let mut piece = String::new();
    let mut depth: i64 = 0;
    let mut at = 0;
    while at < chars.len() {
        let c = chars[at];
        depth += (c == '(') as i64 - (c == ')') as i64;
        if c == ',' && depth <= 0 {
            pieces.push(std::mem::take(&mut piece));
            at += 1;
            continue;
        }
        let join = sub(at, at + 5).to_lowercase();
        if join == " and " && depth <= 0 {
            let after = text::strip_chars(&sub(at + 5, at + 30), " ,").to_lowercase();
            let after = text::split_ref(&after).into_iter().take(2).collect::<Vec<_>>().join(" ");
            if DAY_TRIP_PLACES.keys().iter().any(|name| after == *name || after.starts_with(&format!("{name} "))) {
                pieces.push(std::mem::take(&mut piece));
                at += 5;
                continue;
            }
        }
        piece.push(c);
        at += 1;
    }
    pieces.push(piece);
    let mut tidy = Vec::new();
    for piece in pieces {
        let piece = re!(r"^\s*(?:and|plus)\s+").sub("", &piece);
        let piece = text::strip_chars(&piece, " ,");
        if !piece.is_empty() {
            tidy.push(piece.to_string());
        }
    }
    tidy
}

/// The clause of a day-trip line that names the place, with the clauses that share its journey.
pub fn _day_trip_clause(line: &str, place: &str) -> String {
    let pieces = _clauses(line);
    for (place_at, piece) in pieces.iter().enumerate() {
        if !bounded_in(&piece.to_lowercase(), place) {
            continue;
        }
        let mut said = piece.clone();
        for more in &pieces[place_at + 1..] {
            if said.to_lowercase().contains(" for ") || said.contains('(') {
                break;
            }
            said += " and ";
            said += more;
        }
        return said;
    }
    String::new()
}

/// A day-trip place named in the user's last two lines.
pub fn _place_named_in_the_chat(memory: &Memory) -> Option<String> {
    for turn in last_turns(memory, 2).iter().rev() {
        if let Some(place) = _find_in(&turn.user.to_lowercase(), &DAY_TRIP_PLACES) {
            return Some(place.to_string());
        }
    }
    None
}

static WAY_THERE: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\bhow (?:do|can|would|should) (?:i|we|you) get\b|\bhow to get\b|\bhow far\b|\bhow long does it take\b", 0)
});

/// A day-trip place: the trip from its city, or how to get there.
pub fn _day_trip_place(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    let way = WAY_THERE.is_match(clean);
    let mut place = _find_in(clean, &DAY_TRIP_PLACES).map(str::to_string);
    if place.is_none() && way {
        place = _place_named_in_the_chat(memory);
    }
    let place = place?;
    let city = DAY_TRIP_PLACES.at(&place).clone();
    let in_hand = _place_in(memory, understood, &CITIES).or_else(|| _place_in_the_plan(memory, &CITIES));
    if let Some(in_hand) = in_hand {
        if in_hand != city {
            return None;
        }
    }
    let trips = DAY_TRIPS.at(&city);
    let (first, _, rest) = text::partition(trips, ". ");
    let clause = _day_trip_clause(first, &place);
    if clause.is_empty() {
        return None;
    }
    let how_long = re!(r"\(([^)]*)\)").search(&clause);
    if way && how_long.is_none() {
        return Some(_without_the_season(&format!("Day trips from {}: {}", title(&city), trips), &_when_said(memory, understood)));
    }
    let mut said = if way {
        let inner = how_long.unwrap().str(1).to_string();
        let journey = text::strip(inner.split(',').next().unwrap_or("")).to_string();
        if journey.to_lowercase().contains(" from ") {
            format!("{}: {journey}.", title(&place))
        } else {
            format!("{}: {journey} from {}.", title(&place), title(&city))
        }
    } else {
        format!("Day trips from {}: {clause}.", title(&city))
    };
    if !rest.is_empty() && !way && !re!(r"\b(bus|coach|car|drive|ferry)\b").is_match(&clause) {
        said += " ";
        said += rest;
    }
    Some(_without_the_season(&said, &_when_said(memory, understood)))
}

// ---------------------------------------------------------------------------------------------
// Getting about, and what the guide has already said

/// How to get about the city named: its card, its tell-tale words, or the general card rule.
pub fn _transit(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    if d2_shared::any_in(clean, &_TRANSIT_NOT) {
        return None;
    }
    if !d2_shared::any_in(clean, &_TRANSIT_WORDS) && _find_in(clean, &TRANSIT_CARDS).is_none() {
        return None;
    }
    let mut city = _place_in(memory, understood, &CITY_TRANSIT);
    if city.is_none() {
        city = _place_in_the_plan(memory, &CITY_TRANSIT);
    }
    if city.is_none() {
        city = _find_in(clean, &TRANSIT_CARDS).map(|c| TRANSIT_CARDS.at(c).clone());
    }
    if city.is_none() {
        city = _find_in(clean, &CITY_TELLS).map(|t| CITY_TELLS.at(t).clone());
    }
    match city {
        Some(city) if CITY_TRANSIT.contains(&city) => Some(format!("In {}: {}", title(&city), CITY_TRANSIT.at(&city))),
        _ => {
            if _find_in(clean, &CARD_WORDS).is_some() {
                return Some(GENERAL_CARD.clone());
            }
            None
        }
    }
}

/// The first 40 characters of the city's guide, lowered: how a reply is known to be the guide.
fn guide_head(city: &str) -> String {
    first_chars(CITIES.get(city).map(String::as_str).unwrap_or(""), 40).to_lowercase()
}

/// Whether the guide for the city was already given in this chat.
pub fn _already_guided(memory: &Memory, city: &str) -> bool {
    let guide = guide_head(city);
    memory.turns.iter().any(|t| !guide.is_empty() && t.bot.to_lowercase().contains(&guide))
}

/// Whether the person asked for this city's guide before and was given it.
pub fn _asked_for_the_guide_before(memory: &Memory, city: &str, _clean: &str) -> bool {
    let body = guide_head(city);
    for turn in &memory.turns {
        let said = turn.user.to_lowercase();
        if said.is_empty() {
            continue;
        }
        if _find_in(&said, &CITIES) != Some(city) {
            continue;
        }
        if !body.is_empty() && turn.bot.to_lowercase().contains(&body) {
            return true;
        }
    }
    false
}

/// Whether the guide was given in answer to a question about the city.
pub fn _guided_off_a_question(memory: &Memory, city: &str) -> bool {
    let body = guide_head(city);
    for turn in &memory.turns {
        let low = turn.user.to_lowercase();
        let said = text::strip(&low);
        if said.is_empty() || _find_in(said, &CITIES) != Some(city) {
            continue;
        }
        if !(_A_QUESTION_OPENS.match_(said).is_some() || said.contains('?')) {
            continue;
        }
        if !body.is_empty() && turn.bot.to_lowercase().contains(&body) {
            return true;
        }
    }
    false
}

/// The sentence of the city's guide about the part asked for (food, museums...).
pub fn _detail_line(city: &str, clean: &str) -> Option<String> {
    let words: HashSet<&str> = text::split_ref(clean).into_iter().collect();
    if words.iter().any(|w| _EATING_WORDS.contains(*w)) && CHEAP_EATS.contains(city) {
        return Some(CHEAP_EATS.at(city).clone());
    }
    for (asked, said) in DETAIL_PARTS.iter() {
        if !words.iter().any(|w| asked.iter().any(|a| a == w) && !ASKING_ONLY.contains(*w)) {
            continue;
        }
        for sentence in CITIES.at(city).split(". ") {
            let lowered = sentence.to_lowercase();
            if text::split_ref(&lowered).iter().any(|w| said.iter().any(|s| s == w)) {
                return Some(format!("In {}: {}.", title(city), text::rstrip_chars(sentence, ".")));
            }
        }
    }
    None
}

/// Whether the bot's last reply was a city card ("In Lisbon: ...").
pub fn a_city_card_is_in_front(card: &str) -> bool {
    let Some(found) = _IN_A_CITY.match_(card) else { return false };
    CITIES.contains(&text::strip(found.str(1)).to_lowercase())
}

/// Whether the line asks what to see first.
pub fn asks_which_one_first(text_in: &str) -> bool {
    WHICH_ONE_FIRST.is_match(text_in)
}

/// The first thing on the city's guide, and the one after it.
pub fn _the_first_thing(city: &str) -> Option<String> {
    let sights = CITIES.get(city).map(String::as_str).unwrap_or("");
    let head = text::rstrip_chars(sights.split(". ").next().unwrap_or(""), ".");
    let items: Vec<String> = head
        .split(',')
        .map(text::strip)
        .filter(|p| !p.is_empty())
        .map(|p| if p.to_lowercase().starts_with("and ") { text::slice(p, 4, None) } else { p.to_string() })
        .collect();
    if items.is_empty() {
        return None;
    }
    let rest = if items.len() > 1 { format!(", then {}", items[1]) } else { String::new() };
    let first = &items[0];
    let first_upper = first.chars().next().is_some_and(|c| c.is_uppercase());
    if first_upper || text::starts_with_any(&first.to_lowercase(), &_A_PLACE_OPENS) {
        return Some(format!("Start with {first}{rest}."));
    }
    let mut c = first.chars();
    let head_char: String = c.next().map(|f| f.to_uppercase().collect()).unwrap_or_default();
    Some(format!("{head_char}{}{rest}.", c.as_str()))
}

/// More about a city the guide has been given for: what first, a detail, or the next card.
pub fn _about_city(city: &str, memory: &Memory, clean: &str, understood: Option<&Understood>) -> Option<String> {
    if WHICH_FIRST.is_match(clean) {
        if let Some(first) = _the_first_thing(city) {
            return Some(first);
        }
    }
    if let Some(detail) = _detail_line(city, clean) {
        return Some(detail);
    }
    if _already_guided(memory, city) {
        if let Some(u) = understood {
            if u.asks.is_empty() && _A_QUESTION_OPENS.match_(clean).is_none() {
                return Some(format!("In {}: {}", title(city), _without_the_season(CITIES.at(city), &_when_said(memory, u))));
            }
        }
        return _the_next_card_about(city, memory, clean);
    }
    Some(format!("In {}: {}", title(city), CITIES.at(city)))
}

/// Whether the line asks for more places (further afield, a day out).
pub fn _wants_more_places(clean: &str) -> bool {
    if text::split_ref(clean).iter().any(|w| MORE_PLACES.contains(*w)) {
        return true;
    }
    re!(r"\bday\s+(?:trip|trips|out)\b").is_match(clean)
}

static DAYS_OR_NIGHTS: LazyLock<Re> =
    LazyLock::new(|| Re::new(&format!(r"\b(\d{{1,2}}|{})\s+(?:days|nights)\b", _NUMBER_WORD.join("|")), 0));

/// Whether the stay they told me is long for the city (twice what it wants, or more).
pub fn _the_stay_is_long(memory: &Memory, city: &str) -> bool {
    let fills = CITY_DAYS.get(city).copied().unwrap_or(3);
    for turn in &memory.turns {
        let said = turn.user.to_lowercase();
        let mut days = TRIP_UNITS_LONGEST.iter().find(|(p, _)| said.contains(p.as_str())).map(|(_, c)| *c);
        if days.is_none() {
            if let Some(more) = DAYS_OR_NIGHTS.search(&said) {
                let word = more.str(1);
                days = Some(if text::is_digit(word) {
                    word.parse().unwrap_or(0)
                } else {
                    _NUMBER_WORD.iter().position(|w| w == word).map(|i| i as i64 + 1).unwrap_or(0)
                });
            }
        }
        if let Some(d) = days {
            if d != 0 && d >= std::cmp::max(2, 2 * fills) {
                return true;
            }
        }
    }
    false
}

/// Whether a reply starting like this text was already given in this chat.
pub fn _said_in_this_chat(memory: &Memory, text_in: &str) -> bool {
    let head = first_chars(text_in, 40).to_lowercase();
    memory.turns.iter().any(|t| !head.is_empty() && t.bot.to_lowercase().contains(&head))
}

/// The next card about the city that this chat has not had yet.
pub fn _the_next_card_about(city: &str, memory: &Memory, clean: &str) -> Option<String> {
    let place = CITY_COUNTRY.get(city).map(String::as_str).unwrap_or(city);
    let mut cards: Vec<Option<String>> = Vec::new();
    let areas = CITY_AREA.get(city).map(|a| format!("In {}: by area, {a}.", title(city)));
    let day_trips = DAY_TRIPS.get(city).map(|d| format!("Day trips from {}: {d}", title(city)));
    if _wants_more_places(clean) || _the_stay_is_long(memory, city) {
        cards.extend([day_trips, areas]);
    } else if text::split_ref(clean).iter().any(|w| SEEING.contains(*w)) {
        cards.extend([areas, day_trips]);
    } else {
        cards.extend([day_trips, areas]);
    }
    if let Some(t) = CITY_TRANSIT.get(city) {
        cards.push(Some(format!("In {}: {t}", title(city))));
    }
    if let Some(w) = WHEN_TO_GO.get(city) {
        cards.push(Some(format!("For {}: {w}", title(city))));
    } else if let Some(b) = BEST_TIME.get(place) {
        cards.push(Some(format!("For {}: {b}", title(city))));
    }
    // Python asks `city in CLIMATES` (a table keyed by climate, not city), then prints
    // `CLIMATES[CITY_CLIMATE[city]]`, a dict of month -> (low, high).
    if CITY_CLIMATE.contains(city) && CLIMATES.contains_key(city) {
        let climate = g("CLIMATES").get(CITY_CLIMATE.at(city)).unwrap_or_else(|| panic!("KeyError"));
        cards.push(Some(format!("For {}: {}", title(city), py_dict_repr(climate))));
    }
    if let Some((low, high, note)) = DAILY_BUDGET.get(city) {
        cards.push(Some(format!(
            "{} is about \u{a3}{} to \u{a3}{} a day per person on the ground, room aside. {}.",
            title(city),
            py_str(low),
            py_str(high),
            upper_first(note)
        )));
    }
    cards.into_iter().flatten().find(|card| !_said_in_this_chat(memory, card))
}

/// Python's `str()` of a climate dict: `{1: (8, 15), 2: (9, 16), ...}`.
fn py_dict_repr(v: &Value) -> String {
    fn repr(v: &Value) -> String {
        match v {
            Value::Str(s) => d2_shared::py_repr_str(s),
            Value::Tuple(xs) if xs.len() == 1 => format!("({},)", repr(&xs[0])),
            Value::Tuple(xs) => format!("({})", xs.iter().map(repr).collect::<Vec<_>>().join(", ")),
            Value::List(xs) => format!("[{}]", xs.iter().map(repr).collect::<Vec<_>>().join(", ")),
            Value::Dict(d) => format!("{{{}}}", d.iter().map(|(k, v)| format!("{}: {}", repr(k), repr(v))).collect::<Vec<_>>().join(", ")),
            other => py_str(other),
        }
    }
    repr(v)
}

/// "What else is there" after a city guide: the city in the chat, and more about it.
pub fn _guide_followup(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    if _find_in(clean, &CITIES).is_some() {
        return None;
    }
    if !d2_shared::any_in(clean, &FOLLOW_MARKS) {
        return None;
    }
    let mut city = _place_in(memory, understood, &CITIES);
    if city.is_none() && SIGHTSEEING_ASK.is_match(clean) {
        city = _the_place_held(memory);
    }
    let city = city?;
    if _already_guided(memory, &city) && SIGHTSEEING_ASK.is_match(clean) && _guided_off_a_question(memory, &city) {
        return Some(format!("In {}: {}", title(&city), CITIES.at(&city)));
    }
    _about_city(&city, memory, clean, Some(understood))
}

/// The newest place the store holds, when it is a city with a guide.
pub fn _the_place_held(memory: &Memory) -> Option<String> {
    if let Some(Value::List(held)) = user_value(memory, "places") {
        if let Some(last) = held.last() {
            let newest = text::squash(&if last.truthy() { py_str(last) } else { String::new() }.to_lowercase());
            if CITIES.contains(&newest) {
                return Some(newest);
            }
        }
    }
    _city_they_are_going_to(memory)
}

/// A short follow-up to the last city card: more about that city.
pub fn _guide_next(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    if text::split_ref(clean).iter().filter(|w| text::is_alpha(w)).count() > 8 {
        return None;
    }
    if _find_in(clean, &CITIES).is_some() {
        return None;
    }
    let last = memory.turns.last()?;
    let said = last.bot.to_lowercase();
    for city in CITIES.keys() {
        if said.contains(&format!("in {city}:")) {
            return _about_city(city, memory, clean, Some(understood));
        }
    }
    let mut city = _city_they_are_going_to(memory);
    if city.is_none() && SIGHTSEEING_ASK.is_match(clean) {
        city = _the_place_held(memory);
    }
    let city = city?;
    if !_already_guided(memory, &city) {
        return Some(format!("In {}: {}", title(&city), CITIES.at(&city)));
    }
    _about_city(&city, memory, clean, Some(understood))
}

/// The city the user's last two lines say they are going to.
pub fn _city_they_are_going_to(memory: &Memory) -> Option<String> {
    for turn in last_turns(memory, 2).iter().rev() {
        let said = turn.user.to_lowercase();
        if let Some(city) = _find_in(&said, &CITIES) {
            if GOING_TO_A_PLACE.is_match(&said) {
                return Some(city.to_string());
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------------------------
// Money, weather and the odds and ends of a trip

/// Money abroad: keeping it safe, how much a day, or how to carry it.
pub fn _money_abroad(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    if re!(concat!(
        r"\b(?:safe|safely|safety|stolen|theft|pickpocket|pickpockets|money belt|",
        r"robbed|lose|lost|keep (?:my )?(?:money|cash|card))\b"
    ))
    .is_match(&clean)
    {
        return Some(_KEEP_SAFE.clone());
    }
    if re!(r"\bhow much\b").is_match(&clean) {
        if let Some(rate) = _trip_budget(memory, understood) {
            if !rate.is_empty() {
                return Some(format!("{rate} {}", *_CARRY_SHORT));
            }
        }
    }
    Some(_CARRY_MONEY.clone())
}

static COST_WORD: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"\b(?:how much|cost|costs|costing|expensive|pricey|afford|budget|",
            r"spend|spending|will that|does that|go far|saved|money|cheap|",
            r"inexpensive|affordable)\b"
        ),
        0,
    )
});
static NOT_THE_GROUND: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"\b(?:flight|flights|fly|flying|hotel|hotels|accommodation|apartment|",
            r"villa|hostel|ticket|tickets|deal|deals|package|insurance|car hire)\b"
        ),
        0,
    )
});
static QUALITY_ENOUGH: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"\b(?:warm|cold|hot|good|bad|big|small|deep|strong|quick|fast|early|",
            r"late|cheap|safe|close|near|light|heavy|bright|tall|old|fit|well|",
            r"soon|long|clear|loud|quiet|tidy)\s+enough\b"
        ),
        0,
    )
});

/// What a day on the ground costs in the place named, and whether their sum covers the days.
pub fn _trip_budget(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    let cost_word = COST_WORD.is_match(clean);
    if NOT_THE_GROUND.is_match(clean) {
        return None;
    }
    let quality = QUALITY_ENOUGH.is_match(clean);
    let amount_said = re!("[\u{a3}$\u{20ac}]|\\b\\d{2,}\\b").is_match(clean);
    if !cost_word && !amount_said && (quality || !re!(r"\benough\b").is_match(clean)) {
        return None;
    }
    if !cost_word && !amount_said && re!(r"\b(?:days|nights|how long|too long|too short)\b").is_match(clean) {
        return None;
    }
    let place = _place_in(memory, understood, &DAILY_BUDGET)?;
    let days = TRIP_UNITS_LONGEST.iter().find(|(p, _)| clean.contains(p.as_str())).map(|(_, c)| *c);
    let (low, high, note) = DAILY_BUDGET.at(&place);
    let Some(days) = days else {
        return Some(format!(
            "{} is about \u{a3}{} to \u{a3}{} a day per person on the ground, room aside. {}.",
            title(&place),
            py_str(low),
            py_str(high),
            upper_first(note)
        ));
    };
    let (low_total, high_total) = (int(low) * days, int(high) * days);
    let amount = re!(r"\d[\d,]*").findall_str(clean).into_iter().find_map(|m| {
        let n: i128 = m.replace(',', "").parse().unwrap_or(i128::MAX);
        (50..=20000).contains(&n).then_some(n as i64)
    });
    let Some(amount) = amount else {
        return Some(format!(
            "{days} days in {} usually costs about \u{a3}{low_total} to \u{a3}{high_total} per person on the ground, room aside. {}.",
            title(&place),
            upper_first(note)
        ));
    };
    let verdict = if amount >= high_total {
        "That is comfortable"
    } else if amount >= low_total {
        "That works if you are careful"
    } else {
        "That is tight"
    };
    Some(format!(
        "{verdict}: {days} days in {} usually costs about \u{a3}{low_total} to \u{a3}{high_total} per person on the ground, room aside, and you have \u{a3}{amount}. {}.",
        title(&place),
        upper_first(note)
    ))
}

/// The month the plan or the user's last lines name.
pub fn _month_in_the_chat(memory: &Memory) -> Option<i64> {
    let mut texts: Vec<String> = Vec::new();
    if user_dict(memory).is_some() {
        texts.push(user_value(memory, "the_plan").filter(|v| v.truthy()).map(py_str).unwrap_or_default());
    }
    for turn in last_turns(memory, 4).iter().rev() {
        texts.push(turn.user.clone());
    }
    for t in texts {
        let t = t.to_lowercase();
        if let Some((_, number)) = MONTHS_LONGEST.iter().find(|(name, _)| t.contains(name.as_str())) {
            if *number != 0 {
                return Some(*number);
            }
        }
    }
    None
}

/// The usual temperatures in the city in the month asked about.
pub fn _weather(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    let city = _find_in(clean, &CITY_CLIMATE)
        .map(str::to_string)
        .or_else(|| _place_in(memory, understood, &CITY_CLIMATE))
        .or_else(|| _place_in_the_plan(memory, &CITY_CLIMATE));
    let mut month = MONTHS_LONGEST.iter().find(|(name, _)| clean.contains(name.as_str())).map(|(_, n)| *n);
    if month.is_none() {
        month = _month_in_the_chat(memory);
    }
    let (Some(city), Some(month)) = (city, month) else { return None };
    if month == 0 {
        return None;
    }
    let (low, high) = CLIMATES
        .get(CITY_CLIMATE.at(&city))
        .and_then(|m| m.get(&month))
        .unwrap_or_else(|| panic!("KeyError: {month}"));
    let name = MONTHS_LONGEST
        .iter()
        .find(|(word, number)| *number == month && word.chars().count() > 3)
        .map(|(w, _)| w.clone())
        .unwrap_or_else(|| "May".into());
    Some(format!(
        concat!(
            "{} in {} is usually about {} to {}C, and I can't see a ",
            "live forecast from here. Take a layer either way: evenings are cooler than the ",
            "afternoon, and a light rain jacket covers most days."
        ),
        title(&name),
        title(&city),
        py_str(low),
        py_str(high)
    ))
}

/// Cheap train fares: the railcard, or booking early and off-peak.
pub fn _cheap_trains(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = clean_of(understood);
    if re!(r"\brail ?cards?\b").is_match(t) {
        return Some(concat!(
            "A railcard costs about \u{a3}30 and takes a third off most fares, so it pays for itself ",
            "in two or three trips. Get the one for your age or your area (a Two Together card ",
            "covers two named people travelling together) and load it as a digital card in the ",
            "train company app."
        ).into());
    }
    Some(concat!(
        "Book as early as you can: advance singles usually go on sale about twelve weeks ahead ",
        "and are the cheapest fares. Travelling off-peak (in the UK, after 9:30am) avoids the ",
        "peak price, a railcard pays for itself in a couple of trips, and a split-ticket site ",
        "can cut the fare further by buying the same journey as two shorter tickets."
    ).into())
}

/// A suitcase: what to do when it was damaged, and how to keep it from damage.
pub fn _suitcase_protect(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = understood.clean.to_lowercase();
    const BROKEN: &[&str] = &["broke", "broken", "cracked", "damaged", "snapped", "torn", "ripped", "came back", "came off", "fell off"];
    const KEEPING: &[&str] = &["how do i keep", "keep it from", "from being", "stop", "prevent", "protect", "avoid", "not get", "without"];
    if BROKEN.iter().any(|w| t.contains(w)) && !KEEPING.iter().any(|w| t.contains(w)) {
        return Some(concat!(
            "If the airline did it, report it at the baggage desk before you leave the airport ",
            "and photograph the damage there: the report number is what a claim needs, and ",
            "under the Montreal rules the airline pays for repair or replacement. If it broke ",
            "at home, a luggage repair shop fits new wheels, straps and zip pulls for far less ",
            "than a new case."
        ).into());
    }
    Some(concat!(
        "Do not fill it until it splits: a zip bursts from the inside, so leave a hand's width ",
        "of give and put the heavy things at the wheel end. A luggage strap takes the strain ",
        "off the zip, a hard shell or a padded cover protects what is fragile, and anything ",
        "breakable goes in the middle wrapped in clothes. Photograph it open and closed before ",
        "check-in, because that is what an airline asks for when you claim for the damage."
    ).into())
}

/// Whether paid extras on a flight are worth it: legroom, priority, a chosen seat.
pub fn _flight_extras(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = understood.clean.to_lowercase();
    if t.contains("legroom") || t.contains("leg room") || t.contains("exit row") {
        return Some(concat!(
            "Extra legroom earns its money on a flight over about four hours, or if you are ",
            "tall; on a two-hour hop it buys very little. An exit row has the most space of all, ",
            "but the seat next to you may not recline, the bags have to go in the overhead ",
            "locker, and you have to be able and willing to open the door and help - so read the ",
            "airline's rules before you pay."
        ).into());
    }
    if t.contains("priority") || t.contains("speedy") {
        return Some(concat!(
            "Priority boarding is worth paying for when you are travelling with only a cabin ",
            "bag on a full flight: it gets you on early enough to find space overhead, and a bag ",
            "that gets gate-checked is one you wait for at the carousel at the other end. With a ",
            "checked bag in the hold there is nothing to win, so keep the money."
        ).into());
    }
    Some(concat!(
        "Choosing your seat is worth paying for when you are travelling with somebody and want ",
        "to sit together, or you want a window to sleep against; most airlines seat a child next ",
        "to a parent for free, so a family does not need to buy it. On a short flight with an ",
        "aisle seat left over anyway, save the money."
    ).into())
}

/// A lost passport: abroad now, or one lost a while ago.
pub fn _passport_lost(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = clean_of(understood);
    if ["last year", "years ago", "a while ago", "long ago"].iter().any(|p| t.contains(p)) || re!(r"\b(?:19|20)\d{2}\b").is_match(t) {
        return Some(concat!(
            "A passport reported lost stays cancelled even if it turns up, so you can't fly on ",
            "it. Apply for a new one now and ask for an urgent appointment if the trip is ",
            "close; take your old passport number and a police report if you have one."
        ).into());
    }
    Some(concat!(
        "Report it to the local police and get a written report, then contact your embassy or ",
        "consulate for an emergency travel document. Take the police report, a passport photo ",
        "and any copy of the old passport. In most of Europe this takes a day or two; tell your ",
        "airline and travel insurer as well, and keep the report for the insurance claim."
    ).into())
}

/// The cabin rule for the thing the line names, if it names one.
pub fn _cabin_named(text_in: &str) -> Option<String> {
    _CABIN_RULES.iter().find(|(p, _)| p.is_match(text_in)).map(|(_, s)| s.clone())
}

/// False when the line is about packing something the cabin rules do not name.
pub fn _cabin_guard(understood: &Understood) -> bool {
    let t = understood.clean.to_lowercase();
    if re!(r"\b(?:pack|packs|packing|packed)\b").is_match(&t) && _cabin_named(&t).is_none() {
        return false;
    }
    true
}

/// False when "how much will i take home" is about a wage, not a customs allowance.
pub fn _not_a_wage(understood: &Understood) -> bool {
    let t = clean_of(understood);
    if !_A_PAY_ASK_AT_HOME.is_match(t) {
        return true;
    }
    _A_GOODS_WORD.is_match(t) || _A_CUSTOMS_WORD.is_match(t)
}

/// What can go in the cabin bag (or the hold).
pub fn _cabin_rules(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = understood.clean.to_lowercase();
    if re!(r"\bhow (?:much|many)\b.{0,20}\b(?:liquid|liquids|water|drink|drinks|ml)\b").is_match(&t) {
        return Some(FLIGHT_LIQUIDS.clone());
    }
    if re!(r"\bhow (?:much|many|big|large|heavy)\b.{0,30}\b(?:luggage|baggage|bags?|cabin bag|kilos?|kg|weight)\b").is_match(&t) {
        return Some(String::new());
    }
    if re!(r"\b(?:take (?:it |them |my \w+ )?out|out at security|out of my bag|out of the bag)\b").is_match(&t) {
        return Some(String::new());
    }
    let named = _cabin_named(&t);
    if let Some(n) = &named {
        if n.starts_with("Power banks") || n.starts_with("In the cabin bag only 100ml") {
            return named;
        }
    }
    let hold = re!(r"\b(?:suitcase|hold|checked bag|checked in luggage|checked luggage|holdall)\b").is_match(&t);
    if hold && named.is_none() {
        return Some(_IN_THE_HOLD_ITSELF.clone());
    }
    if hold {
        return Some(_IN_THE_HOLD.clone());
    }
    if named.is_some() {
        return named;
    }
    let banned = re!(r"\bnot allow|banned|prohibit|forbidden|cannot take|can.t take|not take|what can.t i|what can i not|what am i not")
        .is_match(&t);
    if banned {
        return Some(_CABIN_NOT_ALLOWED.replace("{yes}", &_CABIN_LIST[0]));
    }
    if re!(r"\b(?:cabin bag|cabin bags|hand luggage|hand baggage|carry ?on|carry ?ons|luggage|baggage|backpack|suitcase|bag)\b").is_match(&t) {
        return Some(String::new());
    }
    Some(_CABIN_NOT_ALLOWED.replace("{yes}", &_CABIN_YES[0]))
}

// ---------------------------------------------------------------------------------------------
// Months: the verdict on a month for a place

static EDGE: LazyLock<Re> = LazyLock::new(|| Re::new(r"[,;:!?](?:\s|$)|\.(?:\s|$)", 0));

/// Whether the clause around the month says it is the person's own plan ("we're going in june").
pub fn _a_report_of_their_own(clean: &str, month: &str) -> bool {
    let Some((at_start, at_end)) = word_find(clean, month) else { return false };
    let mut start = 0;
    for edge in EDGE.finditer(clean) {
        if edge.end() <= at_start {
            start = edge.end();
        } else if edge.start() >= at_end {
            return _SAYS_THEIR_OWN_PLAN.is_match(bslice(clean, start, edge.start()));
        }
    }
    _SAYS_THEIR_OWN_PLAN.is_match(&clean[start..])
}

/// Every key of the table the text names as a place, longest first.
pub fn _named_place<'t, V>(text_in: &str, table: &'t Table<V>) -> Vec<&'t str> {
    table.longest_first().iter().filter(|n| has_word(text_in, n) && _names_place(text_in, n)).map(String::as_str).collect()
}

/// The verdict on the month for the place, from the place's own window.
pub fn _a_month_for_a_place(name: &str, window: &str, month: &str) -> String {
    let asked = _MONTH_ASKED.with(Cell::get);
    let label = if name.chars().next().is_some_and(|c| c.is_uppercase()) { name.to_string() } else { text::capitalize(name) };
    let clauses: Vec<String> =
        re!(r"(?<=[.;])\s+").split(window).into_iter().filter(|p| !text::strip(p).is_empty()).collect();
    let short = first_chars(month, 3);
    let month_here = rx(&format!(r"\b(?:{}|{})\b", pyre::escape(month), pyre::escape(short)), pyre::I);
    let verdict = if asked { ("Yes, ", "No, ") } else { ("", "") };
    let month_cap = text::capitalize(month);
    if !clauses.is_empty() && month_here.is_match(&clauses[0]) {
        return format!("{}{month_cap} is in the good window for {label}: {window}", verdict.0);
    }
    if clauses.iter().skip(1).any(|part| month_here.is_match(part) && _MONTH_WORSE.is_match(part)) {
        let head = re!(r"[;.:]").split(window).into_iter().next().unwrap_or_default();
        if _A_CLAUSE_VERB.is_match(&head) {
            return format!("{}{month_cap} is not the best month for {label}. {window}", verdict.1);
        }
        return format!("{}{month_cap} is not the best month for {label}. The window is {window}", verdict.1);
    }
    format!("{label}: {window}")
}

/// The table's place the store keeps (plan, plans, places, city).
pub fn _the_place_in_the_chat<V>(memory: &Memory, table: &Table<V>) -> Option<String> {
    let mut said: Vec<String> = Vec::new();
    if let Some(user) = user_dict(memory) {
        for key in ["plan", "plans", "places", "city"] {
            match user.get(key) {
                Some(Value::Str(s)) => said.push(s.to_string()),
                Some(Value::List(xs)) | Some(Value::Tuple(xs)) => {
                    said.extend(xs.iter().filter(|x| x.truthy()).map(py_str));
                }
                _ => {}
            }
        }
    }
    let joined = said.join(" ").to_lowercase();
    if joined.is_empty() {
        return None;
    }
    _find_in(&joined, table).map(str::to_string)
}

/// A week at home (a UK coast or country place) in the month.
pub fn _a_week_at_home(name: &str, month: &str, kind: &str) -> String {
    let label = text::split_ref(name).iter().map(|w| text::capitalize(w)).collect::<Vec<_>>().join(" ");
    let season = _MONTH_SEASON.get(month).map(String::as_str).unwrap_or("shoulder");
    let mut said = vec![format!("{label} in {}: {}", text::capitalize(month), _WEEKS_AT_HOME.at(season))];
    if kind == "coast" {
        said.push(_BY_KIND.at("coast").clone());
        if month == "september" {
            said.push("The sea is at its warmest of the year in September.".into());
        }
    } else {
        said.push(_BY_KIND.at("country").clone());
    }
    said.join(" ")
}

/// Where to go in the month the line names, or the verdict on that month for a place.
pub fn _month_destination(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_of(understood);
    let topics: HashSet<&str> = understood.topics.iter().map(String::as_str).filter(|t| *t != "small-talk").collect();
    let words: Vec<&str> = text::split_ref(clean);
    let asks_a_place = words.iter().any(|w| _AN_OPEN_ASK.contains(*w));
    let ask_clause = understood.focus.join(" ");
    let asks_the_month = !ask_clause.is_empty() && MONTHS.keys().iter().any(|n| has_word(&ask_clause, n));
    if !topics.is_empty()
        && !topics.iter().any(|t| ANSWERS.iter().any(|a| a == t))
        && !asks_a_place
        && !asks_the_month
        && !_A_TRIP_NAMED.is_match(clean)
    {
        return None;
    }
    _MONTH_ASKED.with(|a| a.set(understood.kind == "question" || clean.contains('?')));
    let mut month: Option<&str> = None;
    let mut found: Vec<(usize, usize, &str)> = Vec::new();
    for name in MONTHS.longest_first() {
        if let Some((s, e)) = word_find(clean, name) {
            found.push((s, e, name));
            if month.is_none() {
                month = Some(name);
            }
        }
    }
    let month = month?;
    if _MONTH_NOT_A_PLACE_RE.is_match(clean) {
        return None;
    }
    found.sort();
    if found.len() > 1 {
        let between = bslice(clean, found[0].1, found[1].0);
        if re!(r"\bor\b|\bvs\b|versus|/").is_match(between) {
            return None;
        }
    }
    for table in [&*WHEN_TO_GO, &*BEST_TIME] {
        if let Some(name) = _named_place(clean, table).first() {
            return Some(_a_month_for_a_place(name, table.at(name), month));
        }
    }
    let open_ask = words.iter().any(|w| _AN_OPEN_ASK.contains(*w));
    if !open_ask {
        for table in [&*BEST_TIME, &*WHEN_TO_GO] {
            if let Some(in_hand) = _the_place_in_the_chat(memory, table) {
                return Some(_a_month_for_a_place(&in_hand, table.at(&in_hand), month));
            }
        }
    }
    if let Some(name) = _named_place(clean, &_AT_HOME).first() {
        return Some(_a_week_at_home(name, month, _AT_HOME.at(name)));
    }
    for table in [&*CITY_COUNTRY, &*AIR_HOURS_CITY] {
        for name in _named_place(clean, table) {
            if _UK_NATIONS.iter().any(|n| n == table.at(name)) {
                return Some(_a_week_at_home(name, month, "country"));
            }
        }
    }
    if !open_ask {
        if let Some(in_hand) = _the_place_in_the_chat(memory, &_AT_HOME) {
            return Some(_a_week_at_home(&in_hand, month, _AT_HOME.at(&in_hand)));
        }
        for table in [&*CITY_COUNTRY, &*AIR_HOURS_CITY] {
            if let Some(in_hand) = _the_place_in_the_chat(memory, table) {
                if _UK_NATIONS.iter().any(|n| n == table.at(&in_hand)) {
                    return Some(_a_week_at_home(&in_hand, month, "country"));
                }
            }
        }
    }
    if !_named_place(clean, &WHERE_TO_GO).is_empty()
        || !_named_place(clean, &COUNTRY_ALIASES).is_empty()
        || !_named_place(clean, &CITIES).is_empty()
        || !_named_place(clean, &CAPITALS).is_empty()
    {
        return None;
    }
    if _a_report_of_their_own(clean, month) && !d2_shared::any_in(clean, &_WHERE_IN_THE_LINE) {
        if !_ABOUT_A_TRIP.is_match(clean) {
            return None;
        }
        return Some(format!(
            "Where are you going in {}? Tell me the place and I'll say what the weather does there and what a day costs.",
            text::capitalize(month)
        ));
    }
    if clean.contains("ski") || clean.contains("snow") {
        return Some(format!(
            "In {}: the Alps for snow, with a resort at 1,500m or above for the best chance of it, and book the transfer with the room.",
            text::capitalize(month)
        ));
    }
    if let Some(sun) = _MONTH_SUN.get(month) {
        if _MONTH_SUN_ASK.iter().any(|w| words.contains(&w.as_str())) {
            return Some(format!("{}: {sun}.", text::capitalize(month)));
        }
    }
    let picks = _MONTH_PICKS.at(month);
    Some(format!("{}: {picks}.", text::capitalize(month)))
}

/// The month names, longest first.
pub fn _month_names() -> Vec<String> {
    MONTHS.longest_first().to_vec()
}

/// The month a text names: in time ("in may"), or a month word that is not also a verb.
pub fn _the_month_named_in(text_in: &str) -> Option<String> {
    let low = text::squash(&text_in.to_lowercase());
    if low.is_empty() {
        return None;
    }
    let words: HashSet<&str> = text::split_ref(&low).into_iter().collect();
    if let Some(m) = _MONTH_IN_TEXT.search(&low) {
        return Some(m.str(1).to_string());
    }
    for name in _month_names() {
        if AMBIGUOUS_MONTHS.contains(&name) {
            continue;
        }
        if words.contains(name.as_str()) {
            return Some(name);
        }
    }
    None
}

/// The month of a trip in the user's last few lines.
pub fn _the_month_in_the_chat(memory: &Memory) -> Option<String> {
    let said: Vec<String> =
        last_turns(memory, *_MONTH_LOOKS_BACK).iter().map(|t| t.user.to_lowercase()).filter(|t| !t.is_empty()).collect();
    for t in said.iter().rev() {
        if !_A_MONTH_OF_A_TRIP.is_match(t) {
            continue;
        }
        if let Some(month) = _the_month_named_in(t) {
            return Some(month);
        }
    }
    None
}

/// The verdict on a month for the place in the chat, or for the month itself.
pub fn _the_verdict_on(month: &str, memory: &Memory, clean: &str) -> String {
    for table in [&*BEST_TIME, &*WHEN_TO_GO] {
        if let Some(in_hand) = _the_place_in_the_chat(memory, table) {
            return _a_month_for_a_place(&in_hand, table.at(&in_hand), month);
        }
    }
    if let Some(in_hand) = _the_place_in_the_chat(memory, &_AT_HOME) {
        return _a_week_at_home(&in_hand, month, _AT_HOME.at(&in_hand));
    }
    let words: Vec<&str> = text::split_ref(clean);
    if _MONTH_SUN_ASK.iter().any(|w| words.contains(&w.as_str())) {
        return format!("{}: {}.", text::capitalize(month), _MONTH_SUN.at(month));
    }
    if d2_shared::any_in(&clean.to_lowercase(), &_A_MONTH_FAULT_ASK) {
        return format!("{} is {}.", text::capitalize(month), _MONTH_VERDICT.at(month));
    }
    format!("Yes, for the place it suits. {} is {}.", text::capitalize(month), _MONTH_VERDICT.at(month))
}

/// "Is that a good time?" with no month of its own: the verdict on the month in the chat.
pub fn _a_month_verdict_pointed_at(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = &understood.clean;
    let low = clean.to_lowercase();
    if MONTHS.keys().iter().any(|n| has_word(&low, n)) {
        return None;
    }
    if text::split_ref(clean)
        .iter()
        .any(|w| w.chars().count() > 2 && !_A_VERDICT_FILLER.contains(text::strip_chars(w, ".,!?'")))
    {
        return None;
    }
    let month = _the_month_in_the_chat(memory)?;
    Some(_the_verdict_on(&month, memory, clean))
}

// ---------------------------------------------------------------------------------------------
// Planning a trip

/// The shape of a trip of the length the line says, and the place's window if it names one.
pub fn _trip_plan(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = clean_or_text(understood).to_lowercase();
    let city = _find_in(&t, &CITY_DAYS);
    let length = _TRIP_LENGTH.iter().find(|(p, _, _)| p.is_match(&t)).map(|(_, name, _)| name.as_str());
    let place = _find_in(&t, &BEST_TIME);
    if length.is_some() && place.is_none() && !_TRIP_WORD.is_match(&t) {
        return None;
    }
    if (t.contains("holiday") || t.contains("holidays")) && length.is_none() && place.is_none() {
        return None;
    }
    let shape = match length {
        Some("three weeks") => concat!(
            "Three weeks is slow travel: pick a region and give each of two or three places a ",
            "week, so the trip has a rhythm instead of a list."
        ),
        Some("two weeks") => concat!(
            "Two or three bases for two weeks, with a travel day between them. More than that ",
            "and you spend the trip moving."
        ),
        Some("a weekend") => "A weekend is one neighbourhood walked properly, not two cities.",
        Some("a week") => concat!(
            "One base and two or three day trips beats three cities in a week, because the day ",
            "you move is a day you lose."
        ),
        _ => concat!(
            "Work out how many nights you have first, then give each place at least three of ",
            "them: a base with day trips beats a list of cities, because every move costs you ",
            "half a day."
        ),
    };
    let rest = concat!(
        "Book the first two nights and the last night only, keep one day free in the middle, ",
        "and sleep near the airport on the way home."
    );
    if let Some(city) = city {
        let days = CITY_DAYS.at(city);
        let word = _DAYS_WORD.get(days).unwrap_or_else(|| panic!("KeyError: {days}"));
        return Some(format!("{} wants about {word} full days. {rest}", text::capitalize(city)));
    }
    if let Some(place) = place {
        let window = text::rstrip_chars(BEST_TIME.at(place).split(". ").next().unwrap_or(""), ".");
        return Some(format!("{shape} For {} the window is {window}. {rest}", text::capitalize(place)));
    }
    Some(format!("{shape} {rest}"))
}

/// How to make a holiday cheaper, when the line is about a trip.
pub fn _holiday_costs(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_or_text(understood).to_lowercase();
    if !_TRIP_IN_LINE.iter().any(|w| has_word(&clean, w)) {
        return None;
    }
    Some(CHEAPER_TRIP.clone())
}

/// False when a vague "ideas for a few days away" names neither a price nor more than a length.
pub fn _a_shortlist_question(understood: &Understood) -> bool {
    let clean = clean_or_text(understood).to_lowercase();
    if _A_PRICE.iter().any(|w| has_word(&clean, w)) {
        return true;
    }
    if !_A_VAGUE_ASK.iter().any(|w| has_word(&clean, w)) {
        return true;
    }
    !_A_LENGTH_ONLY.iter().any(|w| has_word(&clean, w))
}

/// Where to go when no place is named: by what they want, or the two things that decide it.
pub fn _where_no_place(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_or_text(understood).to_lowercase();
    for (words, reply) in HOLIDAY_WANTS.iter() {
        if words.iter().any(|w| has_word(&clean, w)) {
            return Some(reply.clone());
        }
    }
    Some(concat!(
        "Two things decide it: how long you have, and whether you want sun or a city. With a ",
        "week and a wish for sun, the Canaries, Greece or southern Spain; with a long ",
        "weekend, Lisbon, Porto, Prague or Edinburgh; with children, a short flight to ",
        "Menorca or the Algarve. Tell me the month and I will be more use than that."
    ).into())
}

/// The place a trip line goes to, in its own words ("" when none).
pub fn _trip_place(clean: &str) -> String {
    let low = clean.to_lowercase();
    let Some(m) = _TRIP_GO.search(&low) else { return String::new() };
    let tail = m.str(1);
    for place in AIR_HOURS_CITY.longest_first() {
        if has_word(tail, place) {
            return place.clone();
        }
    }
    let words: Vec<String> = re!(r"[a-z][a-z-]*").findall_str(tail).into_iter().filter(|w| !_PLACE_FILLER.contains(w)).collect();
    if words.is_empty() || _NOT_A_PLACE.contains(&words[0]) || _TRIP_SHAPE.contains(&words[0]) {
        return String::new();
    }
    let mut name = vec![words[0].clone()];
    for word in words.iter().skip(1).take(2) {
        if _PLACE_ENDS.contains(word) || _NOT_A_PLACE.contains(word) {
            break;
        }
        name.push(word.clone());
    }
    name.join(" ")
}

fn names_a_known_place(clean: &str) -> bool {
    WHERE_TO_GO.keys().iter().chain(COUNTRY_ALIASES.keys()).chain(CITIES.keys()).any(|w| has_word(clean, w))
}

/// Whether the line wants a trip of the person's own (a stag do, a weekend away somewhere).
pub fn _a_trip_is_wanted(understood: &Understood) -> bool {
    let clean = clean_or_text(understood).to_lowercase();
    let words: HashSet<String> = re!(r"[a-z']+").findall_str(&clean).into_iter().collect();
    if _A_REFUSAL.is_match(&clean) {
        return false;
    }
    if re!(r"\bstag\b|\bhen (?:do|weekend|night)\b").is_match(&clean) {
        return true;
    }
    if words.iter().any(|w| _SOMEONE_ELSE.contains(w)) || _SOMEONE_WANTS.is_match(&clean) {
        return false;
    }
    if _NO_TRIP_WANTED.is_match(&clean) {
        return false;
    }
    if words.iter().any(|w| _NOT_A_PLACE.contains(w) || _NOT_A_TRIP_EVENT.contains(w)) {
        return false;
    }
    let place = _trip_place(&clean);
    if !place.is_empty() && !names_a_known_place(&clean) {
        return true;
    }
    if words.iter().any(|w| _TRIP_SHAPE.contains(w)) {
        return !re!(r"\b(?:go|going|get|getting)\s+(?:home|back|in|out)\b").is_match(&clean);
    }
    false
}

/// A trip wanted: the stag-do card, a few days in the place they name, or where to go.
pub fn _want_a_trip(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_or_text(understood).to_lowercase();
    if _A_REFUSAL.is_match(&clean) {
        return None;
    }
    if re!(r"\bstag\b|\bhen (?:do|weekend|night)\b").is_match(&clean) {
        return Some(_STAG_DO.clone());
    }
    if names_a_known_place(&clean) {
        return None;
    }
    let place = _trip_place(&clean);
    if !place.is_empty() {
        return Some(format!(
            concat!(
                "{} for a few days: fix the dates and pick one base, then book the flight ",
                "and the room together six to eight weeks ahead, and agree the one thing you ",
                "all want to do before anything else. The rest is food and wandering, and for ",
                "a group that is the part that goes wrong - one person books, everyone pays ",
                "into one pot, and keep the time on the ground to two or three nights."
            ),
            title(&place)
        ));
    }
    _where_no_place(memory, understood)
}

/// False when the line is about bringing or making food rather than a trip.
pub fn _not_a_food_to_carry(understood: &Understood) -> bool {
    let clean = clean_or_text(understood).to_lowercase();
    let words: HashSet<String> = re!(r"[a-z']+").findall_str(&clean).into_iter().collect();
    if !_FOOD_TO_CARRY.iter().any(|w| words.contains(w)) {
        return true;
    }
    const CARRY: &[&str] = &["bring", "brings", "bringing", "take", "takes", "taking", "carry", "food", "eat", "make", "makes", "cook", "bake"];
    !CARRY.iter().any(|w| words.contains(*w))
}

/// What to pack for a warm week, with the month's own note.
pub fn _warm_week_pack(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_or_text(understood);
    let (months, seasons) = _time_terms(clean, true);
    let base = "one long-sleeved layer and a light waterproof, plus";
    if [6, 7, 8].iter().any(|m| months.contains(m)) || seasons.contains("summer") || seasons.contains("high summer") {
        return Some(format!(
            "For a warm summer week: {}",
            _WARM_WEEK.replace(
                base,
                concat!(
                    "one long-sleeved layer for air conditioning and a light waterproof for a ",
                    "thunderstorm, plus"
                )
            )
        ));
    }
    if [5, 9, 10].iter().any(|m| months.contains(m)) {
        return Some(format!(
            "For a warm week out of high summer: {}",
            _WARM_WEEK.replace(
                base,
                concat!(
                    "one long-sleeved layer and a light waterproof, as May and October evenings turn ",
                    "cool even in Spain or Italy, plus"
                )
            )
        ));
    }
    Some(format!("For a warm week: {}", *_WARM_WEEK))
}

/// Keeping drinks cold: fast, or for the day.
pub fn _cold_drinks(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = clean_or_text(understood).to_lowercase();
    if ["quick", "quickly", "fast", "in a hurry", "right now", "asap", "hurry", "minutes"].iter().any(|w| said.contains(w)) {
        return Some(concat!(
            "Fastest is a bucket or a big bowl of cold water with ice in it and a good ",
            "handful of salt: salt makes the water colder than ice alone, and a bottle ",
            "or a glass drops to fridge-cold in about 15 minutes. No ice? Wet a tea ",
            "towel, wring it out, wrap the bottle and stand it in the freezer for 10 to ",
            "15 minutes, or stand the glass in iced water and stir."
        ).into());
    }
    Some(concat!(
        "Chill it in the fridge first, then a cool bag or box with a frozen ice block or ",
        "a bag of ice at the bottom and the drinks on top - ice above them, not under, ",
        "since cold sinks. Fill bottles right to the top and keep the bag shut and out of ",
        "the sun; a wrapped bottle holds the cold far longer than a bare one. For a ",
        "party, chill half the bottles and freeze the other half: the frozen ones keep ",
        "the box cold and thaw by the time you need them."
    ).into())
}

/// Whether the window's first clause was already said by the bot in the last five turns.
pub fn _the_window_is_read(memory: &Memory, window: &str) -> bool {
    let said = text::strip(window).to_lowercase();
    let pieces = re!(r",? when |,? and (?:the|it) |\. ").split(&said);
    let first = text::strip_chars(pieces.first().map(String::as_str).unwrap_or(""), " .;,").to_string();
    if text::split_ref(&first).len() < 4 {
        return false;
    }
    last_turns(memory, 5).iter().any(|t| t.bot.to_lowercase().contains(&first))
}

/// Whether the same verdict was given in the last five turns.
pub fn _the_same_verdict_was_given(memory: &Memory, tail: &str) -> bool {
    let said_tail = text::strip(tail).to_lowercase();
    if said_tail.is_empty() {
        return false;
    }
    last_turns(memory, 5).iter().any(|t| t.bot.to_lowercase().contains(&said_tail))
}

/// The month verdict, said once: a window already given is not read out again.
pub fn _a_month_verdict_once(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let here = memory.understood.as_ref().map(|u| u.clean.clone()).unwrap_or_default();
    if !here.is_empty() && _the_month_of_text(&here).is_none() && d2_shared::any_in(&here, &_A_TIME_VERDICT_ASK) {
        if let Some(month) = _the_month_of(understood) {
            return Some(_the_verdict_on(&month, memory, &here));
        }
    }
    let said = _month_destination(memory, understood)?;
    let yes_window = _YES_THE_WINDOW.as_str();
    let no_window = _NO_THE_WINDOW.as_str();
    if said.starts_with("Yes, ") && said.contains(yes_window) {
        let (head, _, window) = text::partition(&said, ": ");
        if !_the_window_is_read(memory, window) {
            return Some(said.clone());
        }
        let label = head.split_once(yes_window).map(|(_, l)| l).unwrap_or_else(|| panic!("IndexError"));
        let tail = format!("{yes_window}{label}");
        return Some(format!("{head}{}", if _the_same_verdict_was_given(memory, &tail) { " too." } else { "." }));
    }
    if said.starts_with("No, ") && said.contains(no_window) {
        let marker = [". The window is ", ". "].into_iter().find(|m| said.contains(m)).unwrap_or("");
        let (head, rest) = if marker.is_empty() {
            (said.as_str(), "")
        } else {
            let (h, _, r) = text::partition(&said, marker);
            (h, r)
        };
        if rest.is_empty() || !_the_window_is_read(memory, rest) {
            return Some(said.clone());
        }
        let label = head.split_once(no_window).map(|(_, l)| l).unwrap_or_else(|| panic!("IndexError"));
        let tail = format!("{no_window}{label}");
        return Some(format!("{head}{}", if _the_same_verdict_was_given(memory, &tail) { " either." } else { "." }));
    }
    let (label, _, window) = text::partition(&said, ": ");
    if window.is_empty() || !_the_window_is_read(memory, window) {
        return Some(said.clone());
    }
    let month = _the_month_of(understood);
    let pieces = re!(r",? when |\. ").split(window);
    let first0 = pieces.first().cloned().unwrap_or_default();
    let first = text::strip_chars(first0.split(':').next().unwrap_or(""), " .;,").to_string();
    let first_low = first.to_lowercase();
    let names_a_month = MONTHS.keys().iter().any(|n| has_word(&first_low, n));
    if let Some(month) = month {
        if names_a_month {
            let outside = format!("is not in the window for {label}");
            if _the_same_verdict_was_given(memory, &outside) {
                return Some(format!("No, {} {outside} either.", text::capitalize(&month)));
            }
            return Some(format!("No, {} {outside}: {first}.", text::capitalize(&month)));
        }
    }
    Some(said.clone())
}

fn _the_month_of_text(clean: &str) -> Option<String> {
    let low = clean.to_lowercase();
    MONTHS.longest_first().iter().find(|n| has_word(&low, n)).cloned()
}

/// The month the line names (the longest month word first).
pub fn _the_month_of(understood: &Understood) -> Option<String> {
    _the_month_of_text(clean_or_text(understood))
}

// ---------------------------------------------------------------------------------------------
// Guards on journeys

/// Whether the way-there line is about eating on the way (or value eating).
pub fn _a_way_there_needs_no_eating(understood: &Understood) -> bool {
    let clean = understood.clean.to_lowercase();
    _AN_EATING.is_match(&clean) || _VALUE_EATING.is_match(&clean)
}

/// False when the line gives a weight (a kitchen question, not a luggage one).
pub fn _a_weight_is_the_kitchen(understood: &Understood) -> bool {
    !a_weight(understood)
}

/// A plane-or-ground comparison needs a way on the ground in the line too.
pub fn _a_comparison_needs_two_ways(understood: &Understood) -> bool {
    let clean = understood.clean.to_lowercase();
    if !_A_PLANE.is_match(&clean) {
        return true;
    }
    _A_WAY_ON_THE_GROUND.is_match(&clean)
}

/// "Is X safe": the ordinary kit that keeps you out of trouble in the city named (or in the chat).
pub fn _a_place_it_might_not_be_safe(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    let mut city = _find_in(&clean, &CITIES).map(str::to_string);
    if city.is_none() {
        for turn in last_turns(memory, 3).iter().rev() {
            city = _find_in(&turn.user.to_lowercase(), &CITIES)
                .or_else(|| _find_in(&turn.bot.to_lowercase(), &CITIES))
                .map(str::to_string);
            if city.is_some() {
                break;
            }
        }
    }
    let city = city?;
    Some(format!(
        concat!(
            "Nobody can promise {} is safe, and anyone selling that is selling ",
            "something. What keeps you out of trouble there is the ordinary kit: busy, lit ",
            "streets, more so after dark, the metro or a licensed taxi rather than a shortcut, ",
            "the phone out of sight and the bag in front of you, and the two or three streets ",
            "the hotel desk tells you to avoid - they know this week's answer better than any ",
            "guide. Keep the hotel's address on you and the local emergency number in your ",
            "phone."
        ),
        title(&city)
    ))
}

/// A journey, not a treatment at home ("treating worms before a holiday" is a journey).
pub fn _a_journey_not_a_treatment(understood: &Understood) -> bool {
    let t = clean_or_text(understood).to_lowercase();
    if _A_JOURNEY_SAID.is_match(&t) {
        return true;
    }
    !_A_TREATMENT_AT_HOME.is_match(&t)
}

/// A journey, not training for something ("trains to run a marathon").
pub fn _a_journey_not_training(understood: &Understood) -> bool {
    !_TRAINS_FOR.is_match(&understood.clean)
}

/// A coach that is a vehicle, not a teacher or a sofa.
pub fn _a_coach_is_a_journey(understood: &Understood) -> bool {
    !matches!(understood.sense.get("coach").map(String::as_str), Some("teacher") | Some("furniture"))
}

/// A journey in the line: neither training nor a coach that is a person or furniture.
pub fn _the_journey_in_the_line(understood: &Understood) -> bool {
    _a_journey_not_training(understood) && _a_coach_is_a_journey(understood)
}

/// Driving in traffic: the situation the line names, or the jam answer.
pub fn _drive_in_traffic(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let mut parts = vec![understood.clean.to_lowercase()];
    parts.extend(understood.words.iter().map(|w| w.to_lowercase()));
    let t = parts.join(" ");
    for (words, reply) in _TRAFFIC_SITUATIONS.iter() {
        if d2_shared::any_in(&t, words) {
            return Some(reply.clone());
        }
    }
    Some(_JAM_REPLY.clone())
}

/// Whether the line's food is a dish being cooked, not the food of a place.
pub fn _not_a_plate(clean: &str) -> bool {
    _FOOD_NOT_A_TRIP.iter().any(|w| bounded_in(clean, w))
}

/// What to eat in the country (or the city's country) the line or the trip names.
pub fn _food_there(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = clean_or_text(understood).to_lowercase();
    if _not_a_plate(&clean) {
        return None;
    }
    if _find_in(&clean, &CHEAP_EATS).is_some() {
        return None;
    }
    let mut place = _find_in(&clean, &FOOD_IN).map(str::to_string).unwrap_or_default();
    if place.is_empty() {
        place = _find_in(&clean, &FOOD_ADJECTIVES).and_then(|a| FOOD_ADJECTIVES.get(a)).cloned().unwrap_or_default();
    }
    if place.is_empty() {
        let city = _find_in(&clean, &CITY_COUNTRY);
        place = city.and_then(|c| CITY_COUNTRY.get(c)).cloned().unwrap_or_default();
    }
    if place.is_empty() {
        let city = _place_in(memory, understood, &CITY_COUNTRY);
        place = city.as_deref().and_then(|c| CITY_COUNTRY.get(c)).cloned().unwrap_or_default();
        if place.is_empty() {
            place = _place_in(memory, understood, &FOOD_IN).unwrap_or_default();
        }
        if !place.is_empty() && !_a_trip_to(&place, memory, understood) {
            return None;
        }
    }
    if !FOOD_IN.contains(&place) {
        return None;
    }
    let pattern = format!(r"(?<![a-z])(?:from|born in|raised in|grew up in)\s+(?:the\s+)?{}", pyre::escape(&place));
    if rx(&pattern, 0).is_match(&clean) {
        return None;
    }
    let name = if place == "usa" { "the USA".to_string() } else { title(&place) };
    Some(format!("In {name}: {}", FOOD_IN.at(&place)))
}

/// The paperwork for any trip abroad, with the plug and the water for the country named.
pub fn _trip_essentials(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let mut out = concat!(
        "Sort the paperwork first: anywhere abroad, that is a passport with six months left on ",
        "it when you come back, travel insurance taken out the day you book rather than the day ",
        "you fly, and your booking, policy and passport photo page saved on your phone so they ",
        "work with no signal."
    )
    .to_string();
    if let Some(place) = _place_in(memory, understood, &PLUG_TYPES) {
        let (letter, _shape, _note) = PLUG_TYPES.at(&place);
        let mut bits: Vec<String> = Vec::new();
        if letter != "G" {
            bits.push(format!("{} uses type {letter} sockets, so British plugs need an adapter", title(&place)));
        }
        if let Some(safe) = WATER.get(&place) {
            if *safe {
                bits.push("the tap water is fine to drink".into());
            } else {
                bits.push("the tap water is not safe to drink, so bottled or filtered".into());
            }
        }
        if !bits.is_empty() {
            out += &format!(" {}.", bits.join("; "));
        }
    }
    Some(out)
}

/// False when "flat" is the flat they live in or a thing made flat, unless it is a walk.
pub fn _not_the_flat_the_ground(understood: &Understood) -> bool {
    let mut said = d2_shared::or(&understood.text, &understood.clean).to_string();
    if said.is_empty() {
        said = understood.words.join(" ");
    }
    if !not_the_flat_of_something_made(understood) {
        return false;
    }
    if _A_WALK_IS_SAID.is_match(&said) {
        return true;
    }
    not_the_flat_you_live_in(understood)
}

/// Whether the reading places the line's subject in a care home being visited.
// TYPE(Understood.care_home is Option<String>; Python reads a dict with a "shape" key, so this
// is never true until the field carries the dict).
pub fn _a_visit_to_a_home(_understood: &Understood) -> bool {
    false
}

/// The time left on a passport, said near the passport (or the journey): (count, unit).
pub fn _the_time_left(said: &str) -> Option<(String, String)> {
    let words = text::split_ref(said);
    let mut marks: Vec<i64> = words.iter().enumerate().filter(|(_, w)| _ABOUT_THE_PASSPORT.is_match(w)).map(|(i, _)| i as i64).collect();
    if marks.is_empty() {
        marks = words.iter().enumerate().filter(|(_, w)| _ABOUT_THE_JOURNEY.is_match(w)).map(|(i, _)| i as i64).collect();
    }
    if marks.is_empty() {
        return None;
    }
    let mut best: Option<(i64, String, String)> = None;
    for m in _A_TIME_LEFT.finditer(said) {
        let first = text::split_ref(&said[..m.start()]).len() as i64;
        let last = first + text::split_ref(m.as_str()).len() as i64 - 1;
        let away = marks.iter().flat_map(|mark| [(mark - first).abs(), (mark - last).abs()]).min().unwrap_or(i64::MAX);
        if away > *_NEAR_ENOUGH {
            continue;
        }
        if best.as_ref().is_none_or(|b| away < b.0) {
            best = Some((away, m.str(1).to_string(), m.str(2).to_lowercase()));
        }
    }
    best.map(|(_, token, unit)| (token, unit))
}

/// "No - that is too little time left" when the passport has under six months on it.
pub fn _the_passport_verdict(understood: &Understood) -> String {
    let raw = d2_shared::or(&understood.text, &understood.clean);
    let said = text::squash(&raw.to_lowercase().replace('\u{2019}', "'"));
    if !_A_VALIDITY_WORD.is_match(&said) {
        return String::new();
    }
    let too_little = "No - that is too little time left";
    if _UNDER_SIX.is_match(&said) {
        return too_little.into();
    }
    let Some((token, unit)) = _the_time_left(&said) else { return String::new() };
    let count = if text::is_digit(&token) { token.parse::<i64>().unwrap_or(99) } else { _NUMBER_WORDS.get(&token).copied().unwrap_or(99) };
    if unit.starts_with("day") {
        return too_little.into();
    }
    if unit.starts_with("week") && count < 26 {
        return too_little.into();
    }
    if unit.starts_with("month") && count < 6 {
        return too_little.into();
    }
    String::new()
}

/// How to read a bus or train timetable.
pub fn _read_timetable(memory: &mut Memory, _understood: &Understood) -> Option<String> {
    let options = [
        concat!(
            "Read a timetable by column, not by line: each column is one journey, so run your finger ",
            "down it to see the time at your stop and the one after. Times after noon are often in ",
            "24-hour form (17:40 is 5:40 in the evening), the small letters beside a time are ",
            "explained in the key at the bottom - S for schooldays, NS for not Sundays - and Sunday ",
            "and bank holiday services have their own table with the last bus much earlier. The ",
            "operator's app or the display at the stop is the only thing that knows about today's ",
            "delays."
        ),
        concat!(
            "Find your stop's row first and read across it: the minutes are printed under the hour, ",
            "so a column is one bus and a row is one stop. The letters next to a time are notes - ",
            "schooldays only, not Saturdays, bank holidays - and there is usually a separate table ",
            "for Sunday. Be at the stop five minutes early, because a bus that comes every half hour ",
            "is the one you cannot afford to miss, and check live times on the operator's app before ",
            "you set off: the printed table is the plan, not what is happening."
        ),
    ];
    Some(d2_shared::pick_in(&options, memory, "").to_string())
}

/// False when the line asks what to do or see there (the guide's question, not the verdict's).
pub fn _a_verdict_not_a_guide(understood: &Understood) -> bool {
    let t = format!(" {} ", text::squash(&understood.text.to_lowercase()));
    !_GUIDE_ASK.is_match(&t)
}

/// Whether the city named is good with children.
pub fn _the_family_verdict(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let city = _place_in(memory, understood, &FAMILY_VERDICT)?;
    Some(FAMILY_VERDICT.at(&city).clone())
}

/// Whether the line carries something across a border into Britain.
pub fn _comes_across_a_border(understood: &Understood) -> bool {
    let line = clean_or_text(understood).to_lowercase();
    crosses_a_border(&line) && !carries_it_out_of_britain(&line)
}

/// Getting to the airport: for an early flight, or in general.
pub fn _to_the_airport(understood: &Understood) -> String {
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let t = words.join(" ");
    if _EARLY_FLIGHT.iter().any(|k| words.contains(k) || t.contains(k.as_str())) {
        return _AIRPORT_EARLY_REPLY.clone();
    }
    _TO_THE_AIRPORT.clone()
}

// ---------------------------------------------------------------------------------------------
// How far it is from one place to another

/// The road network a place is on ("gb", "ie", "eu"...), or the place itself.
pub fn _network(place: &str) -> String {
    if let Some(fix) = _NETWORK_FIX.get(place) {
        return fix.clone();
    }
    let code = &_PLACE.at(place).2;
    if code == "other" {
        return place.to_string();
    }
    _NETWORK_NAME.get(code).cloned().unwrap_or_else(|| code.clone())
}

/// The places of the distance table the text names, in the order it names them.
pub fn _places_named(said: &str) -> Vec<String> {
    let chars: Vec<char> = said.chars().collect();
    let mut found: Vec<(usize, usize, String)> = Vec::new();
    for name in _PLACE.longest_first() {
        let name_chars: Vec<char> = name.chars().collect();
        let n = name_chars.len();
        if n == 0 || n > chars.len() {
            continue;
        }
        for at in 0..=chars.len() - n {
            if chars[at..at + n] != name_chars[..] {
                continue;
            }
            let before_ok = at == 0 || !chars[at - 1].is_alphanumeric();
            let after_ok = at + n >= chars.len() || !chars[at + n].is_alphanumeric();
            let overlaps = found.iter().any(|(lo, hi, _)| *lo < at + n && at < *hi);
            if before_ok && after_ok && !overlaps {
                found.push((at, at + n, name.clone()));
            }
        }
    }
    found.sort();
    found.into_iter().map(|(_, _, n)| n).collect()
}

/// The great-circle distance between two places of the table, in km.
pub fn _km(first: &str, second: &str) -> f64 {
    let (lat1, lon1, _) = _PLACE.at(first);
    let (lat2, lon2, _) = _PLACE.at(second);
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let h = (dlat / 2.0).sin().powi(2) + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    2.0 * 6371.0 * h.sqrt().asin()
}

/// Python's `round(x)` as a float, half to even.
fn py_round(x: f64) -> f64 {
    text::round_half_even(x) as f64
}

/// The number to the nearest five.
pub fn _nearest_five(number: f64) -> i64 {
    (py_round(number / 5.0) * 5.0) as i64
}

/// Miles for km: to the nearest five from twenty up.
pub fn _miles(km: f64) -> i64 {
    let miles = km * 0.62137;
    if miles >= 20.0 { _nearest_five(miles) } else { py_round(miles) as i64 }
}

/// The driving time for a road distance on a network.
pub fn _drive_time(km: f64, group: &str) -> String {
    _duration(km / _ROAD_SPEED.get(group).copied().unwrap_or(80.0))
}

/// "2 hours 35 minutes", to the nearest five minutes.
pub fn _duration(hours: f64) -> String {
    let mut whole = hours as i64;
    let mut minutes = (py_round((hours - whole as f64) * 60.0 / 5.0) * 5.0) as i64;
    if minutes == 60 {
        whole += 1;
        minutes = 0;
    }
    if whole == 0 {
        return format!("{minutes} minutes");
    }
    let unit = if whole == 1 { "hour" } else { "hours" };
    if minutes == 0 {
        return format!("{whole} {unit}");
    }
    format!("{whole} {unit} {minutes} minutes")
}

/// How far one place is from another: the straight line, the road, and the sensible way.
pub fn _distance_between(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = clean_or_text(understood).to_lowercase();
    if !d2_shared::any_in(&said, &_FAR_ASKED) {
        return None;
    }
    let names: Vec<String> = _places_named(&said).into_iter().filter(|n| n != "belfast city").collect();
    if names.len() < 2 {
        return None;
    }
    let (first, second) = (&names[0], &names[1]);
    if first == second {
        return None;
    }
    let km = _km(first, second);
    let miles = _miles(km);
    let said_km = _nearest_five(km);
    let (net_one, net_two) = (_network(first), _network(second));
    let there = format!("{} to {}", title(first), title(second));
    let road = _nearest_five(km * *_ROAD_FACTOR + *_ROAD_ALLOWANCE);
    if said_km < 40 {
        let mut how = "a taxi or a bus".to_string();
        if said_km <= 10 {
            how = format!("a taxi, a bus or about {} on a bike", _duration(km / 16.0));
        }
        return Some(format!(
            "{there} is only about {said_km} km ({miles} miles) apart - closer than most people think, so it is {how} rather than a journey."
        ));
    }
    if net_one == net_two && _ROAD_SPEED.contains(&net_one) && km <= 1200.0 {
        return Some(format!(
            concat!(
                "{} is about {} km ({} miles) in a straight line and roughly {} km ({} miles) by ",
                "road, so allow about {} of driving without stops. That is the motorway figure; ",
                "town traffic or a slow A-road at either end adds to it."
            ),
            there,
            said_km,
            miles,
            road,
            _miles(road as f64),
            _drive_time(road as f64, &net_one)
        ));
    }
    if km <= 1200.0 && net_one != net_two {
        let crossing: BTreeSet<&str> = [net_one.as_str(), net_two.as_str()].into_iter().collect();
        let air = _duration(km / 700.0 + 0.8);
        if crossing == BTreeSet::from(["gb", "ie"]) {
            return Some(format!(
                concat!(
                    "{} is about {} km ({} miles) in a straight line, with the Irish Sea in ",
                    "between: a flight is about {} once the airports are counted, and the ferry ",
                    "runs to a timetable rather than a distance."
                ),
                there, said_km, miles, air
            ));
        }
        if crossing == BTreeSet::from(["gb", "eu"]) {
            return Some(format!(
                concat!(
                    "{} is about {} km ({} miles) in a straight line, with the Channel in ",
                    "between: the tunnel does take cars, but the Eurostar or a flight of about ",
                    "{} beats driving it once the check-in and the parking are counted."
                ),
                there, said_km, miles, air
            ));
        }
        return Some(format!(
            concat!(
                "{} is about {} km ({} miles) in a straight line, with water in between: a ",
                "flight is about {} once the airports are counted, and the ferry runs to a ",
                "timetable rather than a distance."
            ),
            there, said_km, miles, air
        ));
    }
    if km <= 1200.0 {
        return Some(format!(
            concat!(
                "{} is about {} km ({} miles) in a straight line. The road route is longer than ",
                "that - allow roughly {} of driving, or take the train, which is usually quicker ",
                "between two cities that close."
            ),
            there,
            said_km,
            miles,
            _drive_time(road as f64, "eu")
        ));
    }
    if km < 3500.0 {
        return Some(format!(
            concat!(
                "{} is about {} km ({} miles) in a straight line, so roughly {} in the air once ",
                "the climb and the landing are counted - too far to drive in a day."
            ),
            there,
            said_km,
            miles,
            _duration(km / 800.0 + 1.0)
        ));
    }
    Some(format!(
        "{there} is about {said_km} km ({miles} miles) in a straight line, which is a long-haul flight of {} hours or more, and nobody drives it.",
        format_0f(km / 850.0 + 2.0)
    ))
}

/// Python's `"%.0f" % x`: round half to even, no decimals.
fn format_0f(x: f64) -> String {
    format!("{:.0}", x)
}

// ---------------------------------------------------------------------------------------------
// The registry: every travel function an entry uses as its guard or reply

macro_rules! guard {
    ($id:literal, $f:path) => {
        ($id, (|_: &FnRef, u: &Understood| $f(u)) as GuardFn)
    };
}
macro_rules! reply {
    ($id:literal, $f:path) => {
        ($id, (|_: &FnRef, m: &mut Memory, u: &Understood| $f(m, u)) as ReplyFn)
    };
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.travel:<lambda>:10387", |_: &FnRef, u: &Understood| !_a_visit_to_a_home(u)),
    guard!("bot.knowledge.travel:_a_comparison_needs_two_ways:7366", _a_comparison_needs_two_ways),
    guard!("bot.knowledge.travel:_a_journey_not_a_treatment:7427", _a_journey_not_a_treatment),
    guard!("bot.knowledge.travel:_a_night_near_the_airport:4267", _a_night_near_the_airport),
    guard!("bot.knowledge.travel:_a_shortlist_question:6906", _a_shortlist_question),
    guard!("bot.knowledge.travel:_a_town_stay_line:4290", _a_town_stay_line),
    guard!("bot.knowledge.travel:_a_trip_is_wanted:7093", _a_trip_is_wanted),
    guard!("bot.knowledge.travel:_a_trip_to_arrange:155", _a_trip_to_arrange),
    guard!("bot.knowledge.travel:_a_verdict_not_a_guide:11972", _a_verdict_not_a_guide),
    guard!("bot.knowledge.travel:_a_way_there_needs_no_eating:7329", _a_way_there_needs_no_eating),
    guard!("bot.knowledge.travel:_a_weight_is_the_kitchen:7345", _a_weight_is_the_kitchen),
    guard!("bot.knowledge.travel:_cabin_guard:5886", _cabin_guard),
    guard!("bot.knowledge.travel:_comes_across_a_border:12071", _comes_across_a_border),
    guard!("bot.knowledge.travel:_not_a_food_to_carry:7161", _not_a_food_to_carry),
    guard!("bot.knowledge.travel:_not_the_flat_the_ground:7799", _not_the_flat_the_ground),
    guard!("bot.knowledge.travel:_the_booking_is_behind_them:126", _the_booking_is_behind_them),
    guard!("bot.knowledge.travel:_the_journey_in_the_line:7453", _the_journey_in_the_line),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.travel:<lambda>:12209", |_: &FnRef, _: &mut Memory, u: &Understood| Some(_to_the_airport(u))),
    ("bot.knowledge.travel:<lambda>:12223", |_: &FnRef, _: &mut Memory, _: &Understood| Some(_AIRPORT_SLEEP_REPLY.clone())),
    reply!("bot.knowledge.travel:_a_month_verdict_once:7255", _a_month_verdict_once),
    reply!("bot.knowledge.travel:_a_month_verdict_pointed_at:6599", _a_month_verdict_pointed_at),
    reply!("bot.knowledge.travel:_a_place_it_might_not_be_safe:7372", _a_place_it_might_not_be_safe),
    reply!("bot.knowledge.travel:_a_ticket_already_bought:57", _a_ticket_already_bought),
    reply!("bot.knowledge.travel:_a_town_stay:4278", _a_town_stay),
    reply!("bot.knowledge.travel:_adapter:3258", _adapter),
    ("bot.knowledge.travel:_airport:4804", |_: &FnRef, m: &mut Memory, u: &Understood| _airport(m, u, true)),
    reply!("bot.knowledge.travel:_airport_early:3531", _airport_early),
    reply!("bot.knowledge.travel:_arriving:4681", _arriving),
    reply!("bot.knowledge.travel:_best_time:4408", _best_time),
    reply!("bot.knowledge.travel:_book_a_journey:98", _book_a_journey),
    reply!("bot.knowledge.travel:_cabin_rules:5950", _cabin_rules),
    reply!("bot.knowledge.travel:_capital:2646", _capital),
    reply!("bot.knowledge.travel:_cheap_eats:2885", _cheap_eats),
    reply!("bot.knowledge.travel:_cheap_trains:5722", _cheap_trains),
    reply!("bot.knowledge.travel:_city_airport:4849", _city_airport),
    reply!("bot.knowledge.travel:_city_pack:3282", _city_pack),
    reply!("bot.knowledge.travel:_cold_drinks:7201", _cold_drinks),
    reply!("bot.knowledge.travel:_country_stay_length:3168", _country_stay_length),
    reply!("bot.knowledge.travel:_country_trip:1092", _country_trip),
    reply!("bot.knowledge.travel:_day_trip:4895", _day_trip),
    reply!("bot.knowledge.travel:_day_trip_place:4984", _day_trip_place),
    reply!("bot.knowledge.travel:_distance_between:12466", _distance_between),
    reply!("bot.knowledge.travel:_drive_in_traffic:7503", _drive_in_traffic),
    reply!("bot.knowledge.travel:_flight_extras:5758", _flight_extras),
    ("bot.knowledge.travel:_flight_followup.<locals>.reply:311", |f: &FnRef, m: &mut Memory, _: &Understood| {
        _flight_followup_reply(f.cell("answer").map(Value::str).unwrap_or(""), m)
    }),
    reply!("bot.knowledge.travel:_food_there:7712", _food_there),
    reply!("bot.knowledge.travel:_get_to:4875", _get_to),
    reply!("bot.knowledge.travel:_guide:2907", _guide),
    reply!("bot.knowledge.travel:_guide_followup:5458", _guide_followup),
    reply!("bot.knowledge.travel:_guide_next:5511", _guide_next),
    reply!("bot.knowledge.travel:_hike_day:3295", _hike_day),
    reply!("bot.knowledge.travel:_holiday_costs:6876", _holiday_costs),
    reply!("bot.knowledge.travel:_london_airport:293", _london_airport),
    reply!("bot.knowledge.travel:_money_abroad:5600", _money_abroad),
    reply!("bot.knowledge.travel:_my_flight_time:4017", _my_flight_time),
    reply!("bot.knowledge.travel:_passport_lost:5778", _passport_lost),
    reply!("bot.knowledge.travel:_read_timetable:11537", _read_timetable),
    reply!("bot.knowledge.travel:_region_days:4576", _region_days),
    reply!("bot.knowledge.travel:_route_between:4862", _route_between),
    reply!("bot.knowledge.travel:_site_note:4887", _site_note),
    reply!("bot.knowledge.travel:_stay_length:3111", _stay_length),
    reply!("bot.knowledge.travel:_stay_there:3247", _stay_there),
    reply!("bot.knowledge.travel:_suitcase_protect:5736", _suitcase_protect),
    reply!("bot.knowledge.travel:_tap_water:3334", _tap_water),
    reply!("bot.knowledge.travel:_taxi_abroad:4833", _taxi_abroad),
    reply!("bot.knowledge.travel:_the_clock_here:4165", _the_clock_here),
    reply!("bot.knowledge.travel:_the_family_verdict:11978", _the_family_verdict),
    reply!("bot.knowledge.travel:_time_zone:4317", _time_zone),
    reply!("bot.knowledge.travel:_tipping:231", _tipping),
    reply!("bot.knowledge.travel:_transit:5126", _transit),
    reply!("bot.knowledge.travel:_travel_advice:207", _travel_advice),
    reply!("bot.knowledge.travel:_trip_budget:5618", _trip_budget),
    reply!("bot.knowledge.travel:_trip_essentials:7767", _trip_essentials),
    reply!("bot.knowledge.travel:_trip_plan:6657", _trip_plan),
    reply!("bot.knowledge.travel:_want_a_trip:7123", _want_a_trip),
    reply!("bot.knowledge.travel:_warm_week_pack:7181", _warm_week_pack),
    reply!("bot.knowledge.travel:_weather:5699", _weather),
    reply!("bot.knowledge.travel:_where_no_place:6921", _where_no_place),
    reply!("bot.knowledge.travel:_where_to_go:1062", _where_to_go),
    reply!("bot.knowledge.travel:_where_to_stay:4200", _where_to_stay),
];


// Stubs appended by harness/integrate_skeleton.py: the Python functions this file does not
// define yet, with signatures from the generated contract. Port them in place (move a
// function up into the file when you port it).
pub use self::skeleton_stubs::*;
#[allow(unused_imports, unused_variables, non_snake_case, dead_code, clippy::all)]
mod skeleton_stubs {
    use super::*;
    use crate::data::{self, Value};
    use crate::knowledge::entry::Entry;
    use crate::runtime::Candidate;
    use crate::state::{Memory, Turn};
    use crate::understood::Understood;

    /// Python: bot/knowledge/travel.py:309 `_flight_followup(answer)` (never called on the trace corpus)
    pub fn _flight_followup(answer: &str) -> Value {
        crate::unported!("bot.knowledge.travel:_flight_followup")
    }

}

