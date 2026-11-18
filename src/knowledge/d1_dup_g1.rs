//! DUP ports (group 1 of workstream D1): private ports of helpers from modules other
//! workstreams own, each marked `DUP(<python module>::<name>)` so the integrator can replace it.
//! Also two Python number-formatting helpers the home rows print figures with.

use crate::data::{self, Value};
use crate::knowledge::d1_shared as sh;
use crate::py::random::{ord_sum, Random};
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
// Python number formatting.

/// Python's `round(x, ndigits)` for a float: the correctly rounded decimal (ties to even on the
/// exact binary value), which is what Rust's fixed-precision formatting gives too.
pub fn py_round(x: f64, ndigits: usize) -> f64 {
    format!("{:.*}", ndigits, x).parse().unwrap_or(x)
}

/// Python's `"%g" % x` / `f"{x:g}"`: six significant digits, trailing zeros dropped, exponent
/// form below 1e-4 and from 1e6.
pub fn py_g(x: f64) -> String {
    if x.is_nan() {
        return "nan".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf".into() } else { "-inf".into() };
    }
    if x == 0.0 {
        return if x.is_sign_negative() { "-0".into() } else { "0".into() };
    }
    let sci = format!("{:.5e}", x);
    let (mant, exp) = sci.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    fn trim(s: &str) -> String {
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s.to_string()
        }
    }
    if exp < -4 || exp >= 6 {
        format!("{}e{}{:02}", trim(mant), if exp < 0 { '-' } else { '+' }, exp.abs())
    } else {
        let decimals = (5 - exp).max(0) as usize;
        trim(&format!("{:.*}", decimals, x))
    }
}

// ---------------------------------------------------------------------------------------------
// bot.compose / bot.memory.history: one line of a pool.

static _PERSON_WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose._PERSON_WORD"));
static OPENS_ON_A_FRAGMENT_QUESTION: LazyLock<&'static Re> =
    LazyLock::new(|| data::re("bot.compose.OPENS_ON_A_FRAGMENT_QUESTION"));
static _NOT_A_FRAGMENT: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose._NOT_A_FRAGMENT"));
static CLAIMS_A_PAST: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.history.CLAIMS_A_PAST"));
static GREETS_A_RETURN: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.history.GREETS_A_RETURN"));
static A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// DUP(bot.compose::said_again): the sentence with whoever it is about blanked, so two sayings
/// of it compare equal.
pub fn said_again(text: &str) -> String {
    _PERSON_WORD.sub("@", &sh::squash_lower(text))
}

/// DUP(bot.compose::_asks_about_a_thing_nobody_said): a pool line that opens on a fragment
/// question about a thing neither side has named.
pub fn _asks_about_a_thing_nobody_said(option: &str, last: Option<&str>, said: &str) -> bool {
    if let Some(last) = last {
        if !last.is_empty() && crate::py::text::strip(last).ends_with('?') {
            return false;
        }
    }
    let Some(m) = OPENS_ON_A_FRAGMENT_QUESTION.match_(option) else { return false };
    let fragment = m.str(1).to_lowercase();
    let words: HashSet<String> = A_WORD
        .findall_str(&fragment)
        .into_iter()
        .filter(|w| w.chars().count() > 2)
        .collect();
    if words.is_empty() || words.iter().any(|w| _NOT_A_FRAGMENT.contains(w)) {
        return false;
    }
    let heard: HashSet<String> =
        A_WORD.findall_str(&format!("{} {}", last.unwrap_or(""), said).to_lowercase()).into_iter().collect();
    !words.iter().any(|w| heard.contains(w))
}

/// DUP(bot.memory.history::_earlier_turns): every turn of theirs before this line (`text`).
fn _earlier_turns<'m>(memory: &'m Memory, text: &str) -> Vec<&'m crate::state::Turn> {
    let mine = sh::squash_lower(text);
    memory
        .turns
        .iter()
        .filter(|row| {
            let line = crate::py::text::squash(&row.user);
            !line.is_empty() && line.to_lowercase() != mine
        })
        .collect()
}

// was DUP(bot.memory.history::an_earlier_turn): now the real port
pub use crate::memory::history::an_earlier_turn;

/// `memory.history.a_past_that_is_not_there(reply, memory)` (the real port, `said` left out).
pub fn a_past_that_is_not_there(reply: &str, memory: &Memory) -> bool {
    crate::memory::history::a_past_that_is_not_there(reply, Some(memory), None)
}

/// DUP(bot.compose::pick): one of `options`, never the reply just given, preferring one the bot
/// has not said lately; seeded by the turn count (and the line, when given).
pub fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    let turns = &memory.turns;
    let last: Option<&str> = turns.last().map(|t| t.bot.as_str());
    let mut options: Vec<String> = options.to_vec();
    let standing: Vec<String> =
        options.iter().filter(|o| !_asks_about_a_thing_nobody_said(o, last, seed_text)).cloned().collect();
    if !standing.is_empty() && standing.len() < options.len() {
        options = standing;
    }
    if !an_earlier_turn(memory, "") {
        let backed: Vec<String> = options.iter().filter(|o| !a_past_that_is_not_there(o, memory)).cloned().collect();
        if !backed.is_empty() {
            options = backed;
        }
    }
    let mut fresh: Vec<String> = options.iter().filter(|o| Some(o.as_str()) != last).cloned().collect();
    if fresh.is_empty() {
        fresh = options.clone();
    }
    // said: the first position each (blanked) bot line was said at
    let mut said: Vec<(String, i64)> = Vec::new();
    for (position, turn) in turns.iter().enumerate() {
        let key = said_again(&turn.bot);
        if !said.iter().any(|(k, _)| *k == key) {
            said.push((key, position as i64));
        }
    }
    let said_at = |o: &str| -> Option<i64> {
        let key = said_again(o);
        said.iter().find(|(k, _)| *k == key).map(|(_, p)| *p)
    };
    let unsaid: Vec<String> = fresh.iter().filter(|o| said_at(o).is_none()).cloned().collect();
    if !unsaid.is_empty() {
        fresh = unsaid;
    } else {
        let oldest = fresh.iter().map(|o| said_at(o).unwrap_or(-1)).min().unwrap_or(-1);
        fresh = fresh.into_iter().filter(|o| said_at(o).unwrap_or(-1) == oldest).collect();
    }
    let seed: i128 = if !seed_text.is_empty() {
        turns.len() as i128 * 7919 + ord_sum(seed_text)
    } else {
        turns.len() as i128
    };
    Random::new(seed).choice(&fresh).clone()
}

/// `pick` over string literals.
pub fn pick_strs(options: &[&str], memory: &Memory) -> String {
    let owned: Vec<String> = options.iter().map(|s| s.to_string()).collect();
    pick(&owned, memory, "")
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.lookup

static HAS_GONE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup.HAS_GONE"));
static GONE_INTENSIFIER: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.GONE_INTENSIFIER"));
static GONE_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.GONE_STATE"));
static WHERE_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.WHERE_PHRASES"));

// was DUP(bot.knowledge.lookup::the_thing_has_gone): now the real port
pub use crate::knowledge::lookup::the_thing_has_gone;

// was DUP(bot.knowledge.lookup::asks_where): now the real port
pub use crate::knowledge::lookup::asks_where;

// ---------------------------------------------------------------------------------------------
// bot.understanding.topics

static MACHINE_BIN_WORDS: LazyLock<HashSet<String>> =
    LazyLock::new(|| data::set("bot.understanding.topics.MACHINE_BIN_WORDS"));

// was DUP(bot.understanding.topics::the_bin_is_on_a_machine): now the real port
pub use crate::understanding::topics::the_bin_is_on_a_machine;

// ---------------------------------------------------------------------------------------------
// bot.knowledge.body

static _PERSON_OWNER: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.body._PERSON_OWNER"));
static _DETERMINERS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.body._DETERMINERS"));
static _POSSESSIVES: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.body._POSSESSIVES"));

// ---------------------------------------------------------------------------------------------
// bot.knowledge.dishes

static _COOK_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.dishes._COOK_PHRASES"));

// ---------------------------------------------------------------------------------------------
// bot.knowledge.travel

static AMBIGUOUS_PLACES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.travel.AMBIGUOUS_PLACES"));
static _PLACE_SHAPE: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.travel._PLACE_SHAPE"));

// was DUP(bot.knowledge.travel::_worth_a_visit): now the real port
pub use crate::knowledge::travel::_worth_a_visit;

// was DUP(bot.knowledge.travel::_names_place): now the real port
pub use crate::knowledge::travel::_names_place;

/// DUP(bot.knowledge.travel::_find_in): the key of `table` the line names, longest first.
pub fn _find_in(text: &str, table: &[String]) -> Option<String> {
    let mut names: Vec<&String> = table.iter().collect();
    names.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()));
    for name in names {
        let pattern = format!(r"(?<![a-z]){}(?![a-z])", sh::escape(name));
        if sh::re_search(&pattern, text, 0) && _names_place(text, name) {
            return Some(name.clone());
        }
    }
    None
}

/// The keys of `bot.knowledge.travel.CITY_DAYS`, in dict order.
pub static CITY_DAYS: LazyLock<Vec<String>> = LazyLock::new(|| {
    data::get("bot.knowledge.travel.CITY_DAYS").pairs().iter().filter_map(|(k, _)| k.as_str().map(str::to_string)).collect()
});

/// `_find_in(text, CITY_DAYS)` with the keys sorted and their patterns compiled once.
pub fn find_city(text: &str) -> Option<String> {
    static SORTED: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
        let mut names: Vec<&String> = CITY_DAYS.iter().collect();
        names.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()));
        names
            .into_iter()
            .map(|n| (n.clone(), Re::new(&format!(r"(?<![a-z]){}(?![a-z])", sh::escape(n)), 0)))
            .collect()
    });
    SORTED.iter().find(|(name, re)| re.is_match(text) && _names_place(text, name)).map(|(name, _)| name.clone())
}

// ---------------------------------------------------------------------------------------------
// bot.memory.topics

/// `memory.topics.context(memory, n)` (the real port).
pub fn topics_context(memory: &Memory, n: usize) -> Vec<String> {
    crate::memory::topics::context(memory, n as i64)
}

/// Python's float floor division `a // b` (CPython's `float_floor_div`: fmod first, so
/// `10 // 0.1` is 99.0, not 100.0).
pub fn py_floordiv(a: f64, b: f64) -> f64 {
    let mut m = a % b; // C fmod
    let mut div = (a - m) / b;
    if m != 0.0 && ((b < 0.0) != (m < 0.0)) {
        m += b;
        div -= 1.0;
    }
    let _ = m;
    if div != 0.0 {
        let mut floordiv = div.floor();
        if div - floordiv > 0.5 {
            floordiv += 1.0;
        }
        floordiv
    } else {
        (0.0f64).copysign(a / b)
    }
}

/// `int(-(-x // 1))`: the whole number at or above x, as Python works it out.
pub fn py_ceil(x: f64) -> i64 {
    -(py_floordiv(-x, 1.0)) as i64
}
