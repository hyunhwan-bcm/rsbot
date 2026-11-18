//! Port of `bot/memory/store_replies.py`: the reply readers the memory store answers from
//! (moved out of `bot/modules/memory_store.py` whole, in their original order).
//!
//! Module-level data (the regexes, word sets and pools, including the names this module imports
//! from `store_lines`, `relations`, `memory_tables` and `corrected`) is read from the exported
//! bundle under `bot.memory.store_replies.<NAME>`, never retyped.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, unused_macros, clippy::all)]

use crate::data::{self, Value};
use crate::knowledge::entry::Entry;
use crate::py::re::Re;
use crate::py::text;
use crate::runtime::Candidate;
use crate::state::{Memory, Turn};
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.memory.store_replies.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{M}{name}"))
}

/// A compiled module-level regex, by Python name.
macro_rules! rx {
    ($($name:ident),* $(,)?) => {$(
        static $name: LazyLock<&'static Re> = LazyLock::new(|| g(stringify!($name)).as_re().expect(stringify!($name)));
    )*};
}
/// A module-level collection of strings, for membership tests.
macro_rules! strset {
    ($($name:ident),* $(,)?) => {$(
        static $name: LazyLock<HashSet<String>> = LazyLock::new(|| g(stringify!($name)).str_set());
    )*};
}
/// A module-level tuple/list of strings, in order.
macro_rules! strs {
    ($($name:ident),* $(,)?) => {$(
        static $name: LazyLock<Vec<String>> = LazyLock::new(|| g(stringify!($name)).strs());
    )*};
}
/// Any module-level value, as exported.
macro_rules! val {
    ($($name:ident),* $(,)?) => {$(
        static $name: LazyLock<&'static Value> = LazyLock::new(|| g(stringify!($name)));
    )*};
}
/// A module-level string.
macro_rules! sval {
    ($($name:ident),* $(,)?) => {$(
        static $name: LazyLock<&'static str> = LazyLock::new(|| g(stringify!($name)).str());
    )*};
}

// ---------------------------------------------------------------------------------------------
// Python behaviour helpers.

/// Python `str(v)`.
fn py_str(v: &Value) -> String {
    match v {
        Value::None => "None".to_string(),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        Value::Str(s) => s.to_string(),
        _ => py_repr(v),
    }
}

/// Python `repr(v)`.
fn py_repr(v: &Value) -> String {
    match v {
        Value::Str(s) => crate::understood::py_str_repr(s),
        Value::List(items) => format!("[{}]", items.iter().map(py_repr).collect::<Vec<_>>().join(", ")),
        Value::Tuple(items) => {
            if items.len() == 1 {
                format!("({},)", py_repr(&items[0]))
            } else {
                format!("({})", items.iter().map(py_repr).collect::<Vec<_>>().join(", "))
            }
        }
        Value::Set(items) => {
            if items.is_empty() {
                "set()".to_string()
            } else {
                format!("{{{}}}", items.iter().map(py_repr).collect::<Vec<_>>().join(", "))
            }
        }
        Value::Dict(d) => format!(
            "{{{}}}",
            d.iter().map(|(k, v)| format!("{}: {}", py_repr(k), py_repr(v))).collect::<Vec<_>>().join(", ")
        ),
        _ => py_str(v),
    }
}

/// Python `str(v or "")`: "" for a falsy value, else `str(v)`.
fn s_or(v: Option<&Value>) -> String {
    match v {
        Some(v) if v.truthy() => py_str(v),
        _ => String::new(),
    }
}

/// Python `str(d.get(key) or "")` on a dict value (anything else reads as `{}`).
fn sget(d: &Value, key: &str) -> String {
    s_or(d.get(key))
}

/// `memory.get(key)` as a value (None when missing).
fn mget<'a>(memory: &'a Memory, key: &str) -> &'a Value {
    static NONE: Value = Value::None;
    memory.get(key).unwrap_or(&NONE)
}

/// `memory.get("user") or {}`: the user's facts as stored (never created here).
fn user_facts(memory: &Memory) -> &Value {
    mget(memory, "user")
}

/// `" ".join(str(text or "").lower().split())`.
fn fold(text_: &str) -> String {
    text::squash(&text_.to_lowercase())
}

/// The memory as a Python-like dict value (for the contract functions that take one).
fn memory_value(memory: &Memory) -> Value {
    Value::from_plain(&serde_json::Value::Object(memory.to_plain()))
}

/// `pick(options, memory, seed_text)` from `bot.compose`.
fn pick<S: AsRef<str>>(options: &[S], memory: &Memory, seed_text: &str) -> String {
    let owned: Vec<String> = options.iter().map(|o| o.as_ref().to_string()).collect();
    crate::compose::pick(&owned, memory, seed_text)
}

/// A str list as a Python list value.
fn str_list<S: AsRef<str>>(items: &[S]) -> Value {
    Value::List(std::sync::Arc::new(items.iter().map(|s| Value::from(s.as_ref())).collect()))
}

/// `[str(x) for x in v]` for a list/tuple value (non-str items in Python's `str()`).
fn py_strs(v: &Value) -> Vec<String> {
    v.items().iter().map(py_str).collect()
}

/// The last turn's user line (`str((turns[-1] or {}).get("user") or "")`), "" without one.
fn last_turn_user(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.user.clone()).unwrap_or_default()
}

/// The last turn's bot line, "" without one.
fn last_turn_bot(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.bot.clone()).unwrap_or_default()
}

/// The value a call into another file gives back, whatever Rust type its contract has: a
/// contract generated from a trace that only saw `None` returns `()`, and such a function's
/// real value arrives here once its owner types it properly.
trait PyRet {
    fn val(self) -> Value;
}
impl PyRet for () {
    fn val(self) -> Value {
        Value::None
    }
}
impl PyRet for Value {
    fn val(self) -> Value {
        self
    }
}
impl PyRet for &Value {
    fn val(self) -> Value {
        self.clone()
    }
}
impl PyRet for Option<Value> {
    fn val(self) -> Value {
        self.unwrap_or(Value::None)
    }
}
impl PyRet for Option<&Value> {
    fn val(self) -> Value {
        self.cloned().unwrap_or(Value::None)
    }
}
impl PyRet for String {
    fn val(self) -> Value {
        Value::from(self)
    }
}
impl PyRet for &str {
    fn val(self) -> Value {
        Value::from(self)
    }
}
impl PyRet for Option<String> {
    fn val(self) -> Value {
        self.map(Value::from).unwrap_or(Value::None)
    }
}
impl PyRet for Option<&str> {
    fn val(self) -> Value {
        self.map(Value::from).unwrap_or(Value::None)
    }
}
impl PyRet for bool {
    fn val(self) -> Value {
        Value::Bool(self)
    }
}
impl PyRet for i64 {
    fn val(self) -> Value {
        Value::Int(self)
    }
}
impl PyRet for Option<i64> {
    fn val(self) -> Value {
        self.map(Value::Int).unwrap_or(Value::None)
    }
}
impl PyRet for Vec<String> {
    fn val(self) -> Value {
        Value::from(self)
    }
}
impl PyRet for Option<Vec<String>> {
    fn val(self) -> Value {
        self.map(Value::from).unwrap_or(Value::None)
    }
}

/// A call's value as Python's `x or ""` string: "" for a falsy value.
fn ret_str(v: impl PyRet) -> String {
    let v = v.val();
    s_or(Some(&v))
}

/// A call's value as `Option<String>`: None for Python `None`, else `str(v)`.
fn ret_opt(v: impl PyRet) -> Option<String> {
    match v.val() {
        Value::None => None,
        v => Some(py_str(&v)),
    }
}

/// Python `"%g" % x`.
fn fmt_g(x: f64) -> String {
    if x == 0.0 {
        return if x.is_sign_negative() { "-0".into() } else { "0".into() };
    }
    if !x.is_finite() {
        return text::float_repr(x);
    }
    let exp = format!("{:.5e}", x);
    let (mant, e) = exp.split_once('e').unwrap();
    let e: i32 = e.parse().unwrap();
    if e < -4 || e >= 6 {
        let mant = mant.trim_end_matches('0').trim_end_matches('.');
        format!("{}e{}{:02}", mant, if e < 0 { '-' } else { '+' }, e.abs())
    } else {
        let prec = (5 - e).max(0) as usize;
        let s = format!("{:.*}", prec, x);
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s
        }
    }
}

/// Python `template.format(**fields)` for templates with plain `{name}` fields.
fn fmt_named(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                out.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                out.push('}');
            }
            '{' => {
                let mut name = String::new();
                for c2 in chars.by_ref() {
                    if c2 == '}' {
                        break;
                    }
                    name.push(c2);
                }
                match fields.iter().find(|(k, _)| *k == name) {
                    Some((_, v)) => out.push_str(v),
                    None => panic!("KeyError: {name:?} in str.format"),
                }
            }
            c => out.push(c),
        }
    }
    out
}

/// Python `list(v)`: the items of a container, the characters of a str, the keys of a dict.
fn py_list(v: &Value) -> Vec<Value> {
    match v {
        Value::Str(s) => s.chars().map(|c| Value::from(c.to_string())).collect(),
        Value::Dict(d) => d.iter().map(|(k, _)| k.clone()).collect(),
        Value::None => Vec::new(),
        v => v.items().to_vec(),
    }
}

/// The dicts in a list value (`[x for x in v if isinstance(x, dict)]`).
fn dicts(v: &Value) -> Vec<Value> {
    v.items().iter().filter(|x| x.is_dict()).cloned().collect()
}

/// A dict value from (key, value) pairs, in order.
fn dict_of(pairs: Vec<(&str, Value)>) -> Value {
    let mut d = Value::dict();
    for (k, v) in pairs {
        d.set_key(k, v);
    }
    d
}

/// `m.group(i)` as an f-string shows it: "None" for a group that did not take part.
fn gn<'t>(m: &crate::py::re::Match<'t>, i: usize) -> &'t str {
    m.get(i).unwrap_or("None")
}

// ---------------------------------------------------------------------------------------------
// Small shared readers: the store's own lists, read the way `bot.memory.user()` / `others()`
// read them (created on first use, as Python's do).

/// `bot.memory.user(memory)`: the user's facts, created on first use (a copy of the dict).
fn user(memory: &mut Memory) -> Value {
    if !mget(memory, "user").is_dict() {
        memory.set("user", Value::dict());
    }
    mget(memory, "user").clone()
}

/// `bot.memory.others(memory)`: facts about other people, created on first use (a copy).
fn others(memory: &mut Memory) -> Value {
    if !matches!(mget(memory, "others"), Value::List(_)) {
        memory.set("others", Value::list());
    }
    mget(memory, "others").clone()
}

/// `isinstance(v, int)` (a bool is an int in Python).
fn is_int(v: Option<&Value>) -> bool {
    matches!(v, Some(Value::Int(_)) | Some(Value::Bool(_)))
}

/// The value of one Unicode decimal digit (`int()` reads every Nd digit).
fn decimal_value(c: char) -> Option<u32> {
    if c.is_ascii_digit() {
        return c.to_digit(10);
    }
    if !crate::understanding::pyuni::ch_decimal(c) {
        return None;
    }
    // Nd digits come in contiguous runs of ten from zero, so the run walked back gives the value.
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

/// Python `int(s)` for a str: None where Python raises ValueError.
fn py_int(s: &str) -> Option<i64> {
    let t = text::strip(s);
    let (neg, body) = match t.chars().next() {
        Some('-') => (true, &t[1..]),
        Some('+') => (false, &t[1..]),
        _ => (false, t),
    };
    if body.is_empty() || body.starts_with('_') || body.ends_with('_') || body.contains("__") {
        return None;
    }
    let mut n: i64 = 0;
    for c in body.chars() {
        if c == '_' {
            continue;
        }
        let d = decimal_value(c)? as i64;
        n = n.checked_mul(10)?.checked_add(d)?;
    }
    Some(if neg { -n } else { n })
}

/// Python `float(s)` for a str: None where Python raises ValueError.
fn py_float(s: &str) -> Option<f64> {
    let t = text::strip(s);
    let ascii: String = t.chars().map(|c| decimal_value(c).map(|d| char::from_digit(d, 10).unwrap()).unwrap_or(c)).collect();
    if ascii.contains('_') {
        return None;
    }
    match ascii.to_lowercase().as_str() {
        "inf" | "+inf" | "infinity" | "+infinity" => return Some(f64::INFINITY),
        "-inf" | "-infinity" => return Some(f64::NEG_INFINITY),
        "nan" | "+nan" | "-nan" => return Some(f64::NAN),
        _ => {}
    }
    ascii.parse::<f64>().ok()
}

/// Python's float `a // b` (CPython's `float_floor_div`, which is not always `floor(a / b)`).
fn py_floordiv(vx: f64, wx: f64) -> f64 {
    let mut m = vx % wx;
    let mut div = (vx - m) / wx;
    if m != 0.0 {
        if (wx < 0.0) != (m < 0.0) {
            m += wx;
            div -= 1.0;
        }
    }
    let _ = m;
    if div != 0.0 {
        let mut floordiv = div.floor();
        if div - floordiv > 0.5 {
            floordiv += 1.0;
        }
        floordiv
    } else {
        0.0f64.copysign(vx / wx)
    }
}

/// "" (Python's falsy reply) as None, so a reply is `Some` only when it says something.
fn nz(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

strset!(STOPWORDS);
rx!(COUNT_ANIMAL, COUNT_BARE);

/// Python: bot/memory/store_replies.py:374 `_a_count_ask(low)`
///
/// The animal a "how many" ask counts, or "" when the line is not that ask: "how many cats",
/// "how many cats do i have", "how many have i got". None when the line is not a count ask.
pub fn _a_count_ask(low: &str) -> Option<String> {
    if let Some(m) = COUNT_ANIMAL.match_(low) {
        let word = text::rstrip_chars(&m.str(1).to_lowercase(), "s").to_string();
        return Some(if matches!(word.as_str(), "pet" | "puppie" | "kitten" | "bunnie" | "horsie") {
            "pet".to_string()
        } else {
            word
        });
    }
    if COUNT_BARE.match_(low).is_some() {
        return Some("any".to_string());
    }
    None
}

rx!(
    ASKED_ALL, MORE_OF_IT, FIRST_ASK_BARE, FIRST_ASK, REPEAT_TAIL, REPEAT_ASK, LAST_LINE_ASK, WHOLE_ASK,
    REPEAT_START, REMINDER_MID, REMINDER_HEAD, REMINDER_TAIL, ABOUT_HEAD, ACTIVITY_ASK, PLAN_WORDS, UP_TO_ASK,
    NAMED_ACTIVITY, ACTIVITY_WORDS, ACTIVITY_LOOSE, ASK_LINE, DETAIL_START, CHAT_TAIL, WHEN_INVERTED,
    WHEN_INVERTED_SUBJECT,
);
strset!(REPEAT_WORDS);

/// Python: bot/memory/store_replies.py:595 `_bare(folded)`
///
/// The ask inside the reminder wrapper: "remind me what I'm allergic to" -> "what i am allergic
/// to", "and my name again?" -> "my name".
pub fn _bare(folded: &str) -> String {
    let asked = REMINDER_MID.search(folded).is_some();
    let bare = REMINDER_MID.sub(" ", folded);
    let bare = ABOUT_HEAD.sub("", &bare);
    let bare = REMINDER_HEAD.sub("", &bare);
    let bare = REMINDER_TAIL.sub("", &bare);
    let bare = CHAT_TAIL.sub("", &bare);
    let bare = text::strip_chars(&bare, " ,.;:!?").to_string();
    let mut bare = text::squash(&bare);
    if let Some(flipped) = WHEN_INVERTED.match_(&bare) {
        // "when my interview is" -> "when is my interview": one order for every when-ask reader
        bare = format!("when {} {} {}", gn(&flipped, 3), gn(&flipped, 1), gn(&flipped, 2));
    } else if asked {
        if let Some(subject) = WHEN_INVERTED_SUBJECT.match_(&bare) {
            // "when i am going to rome" -> "when am i going to rome"
            let rest = text::split(subject.str(2));
            let first = rest.first().expect("IndexError: list index out of range").clone();
            let said = format!("when {} {} {}", first, gn(&subject, 1), rest[1..].join(" "));
            bare = text::strip(&said).to_string();
        }
    }
    bare
}

strs!(LIMITS);

/// Python: bot/memory/store_replies.py:617 `_the_limits_on(entry)`
///
/// The limits a read entry holds, as a dict, or {} for an entry that limits nothing.
pub fn _the_limits_on(entry: &Value) -> Value {
    if !entry.is_dict() {
        return Value::dict();
    }
    let empty = [Value::None, Value::from(""), Value::list(), Value::dict()];
    let mut out = Value::dict();
    for key in LIMITS.iter() {
        let v = entry.get(key).cloned().unwrap_or(Value::None);
        if !empty.iter().any(|e| *e == v) {
            out.set_key(key.as_str(), entry.get(key).cloned().unwrap_or(Value::None));
        }
    }
    out
}

static FACTS_RELATIONS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.facts.RELATIONS"));
static FACTS_ANIMALS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.ANIMALS"));
static LAST_PERSON: LazyLock<String> = LazyLock::new(|| data::string("bot.memory.facts.LAST_PERSON"));

/// Python: bot/memory/store_replies.py:623 `_the_limit_they_share(text, memory)`
///
/// "im too" after "my son is allergic to nuts": the limit is the person talking's as well. The
/// limit is read out of the turn before and filed on the other person. Returns the reply, or
/// None. (Contract change: the generated stub returned `()`.)
pub fn _the_limit_they_share(text_: &str, memory: &mut Memory) -> Option<String> {
    let mut who = ret_str(crate::memory::facts::the_limit_they_share(text_));
    if who.is_empty() {
        return None;
    }
    let before = last_turn_user(memory);
    if before.is_empty() {
        return None;
    }
    let said = crate::memory::facts::extract(&before, false).val();
    if !said.is_dict() {
        return None;
    }
    let about_them: Vec<(String, Value)> = said
        .get("others")
        .map(|o| o.items().to_vec())
        .unwrap_or_default()
        .iter()
        .filter(|e| e.is_dict())
        .map(|e| (sget(e, "who"), _the_limits_on(e)))
        .filter(|(relation, limits)| !relation.is_empty() && limits.truthy())
        .collect();
    let about_me = _the_limits_on(&said);
    if who == "i" {
        let Some((_relation, limits)) = about_them.last().cloned() else {
            return None;
        };
        for (key, value) in limits.pairs().to_vec() {
            crate::memory::remember(memory, key.str(), &value, text_);
        }
        return _said_of_a_shared_limit(&limits, "", memory);
    }
    if who == *LAST_PERSON {
        // "she is too", "he as well": the last person the chat named, out of the line before the
        // share and then the store's own person.
        let named: Vec<String> =
            said.get("people").map(|p| p.items().iter().filter_map(|w| w.as_str().map(str::to_string)).collect()).unwrap_or_default();
        who = named.last().cloned().unwrap_or_default();
        if who.is_empty() {
            let person = mget(memory, "person");
            who = if person.is_dict() { sget(person, "who") } else { String::new() };
        }
        if who.is_empty() {
            return None;
        }
    }
    if !about_me.truthy() {
        return None;
    }
    let mut entry = about_me.clone();
    entry.set_key("who", who.as_str());
    crate::memory::remember_other(memory, &entry, text_);
    _said_of_a_shared_limit(&about_me, &who, memory)
}

/// Python: bot/memory/store_replies.py:671 `_is_or_are(words)`
///
/// "dairy is off your plate", but "nuts are off your plate" and "nuts and dairy are too".
pub fn _is_or_are(words: &[String]) -> String {
    if words.iter().filter(|w| !text::strip(w).is_empty()).count() > 1 {
        return "are".to_string(); // two things said together are plural however each is spelt
    }
    let last = match words.last() {
        Some(w) => text::split(w).last().cloned().expect("IndexError: list index out of range"),
        None => String::new(),
    };
    if last.ends_with('s') && !last.ends_with("ss") { "are" } else { "is" }.to_string()
}

/// Python: bot/memory/store_replies.py:678 `_the_things(words)`
///
/// The things named, as a person says them: "nuts", "nuts and dairy", "nuts, dairy and soya".
pub fn _the_things(words: &[String]) -> String {
    let said: Vec<String> = words.iter().map(|w| text::strip(w).to_string()).filter(|w| !w.is_empty()).collect();
    match said.len() {
        0 => String::new(),
        1 => said[0].clone(),
        n => said[..n - 1].join(", ") + " and " + &said[n - 1],
    }
}

/// Python: bot/memory/store_replies.py:687 `_said_of_a_shared_limit(limits, who, memory)`
///
/// The reply to a line that shares the limit: what it is, and whose it is now.
pub fn _said_of_a_shared_limit(limits: &Value, who: &str, memory: &Memory) -> Option<String> {
    let things: Vec<String> = py_list(limits.get("allergies").filter(|v| v.truthy()).unwrap_or(&Value::None)).iter().map(py_str).collect();
    if !things.is_empty() {
        let named = _the_things(&things);
        if !who.is_empty() {
            return Some(pick(
                &[
                    "Then ".to_string() + &named + " " + &_is_or_are(&things) + " off " + &_their_own(who)
                        + "'s plate too \u{2014} I'll keep both of you in mind.",
                    "Right, ".to_string() + &named + " out for " + &_their_own(who) + " as well, then.",
                    "So ".to_string() + &named + " " + &_is_or_are(&things) + " off the table for " + &_their_own(who)
                        + " too. Both of you, then.",
                ],
                memory,
                "",
            ));
        }
        return Some(pick(
            &[
                "Then ".to_string() + &named + " " + &_is_or_are(&things) + " off your plate too \u{2014} I'll answer for both of you.",
                "Right, ".to_string() + &named + " out for you as well, then.",
                "So ".to_string() + &named + " " + &_is_or_are(&things) + " off the table for you too.",
            ],
            memory,
            "",
        ));
    }
    let diet = limits.get("diet").cloned().unwrap_or(Value::None);
    if diet.truthy() {
        let diet = py_str(&diet);
        if !who.is_empty() {
            return Some("Then ".to_string() + &diet + " for " + &_their_own(who) + " as well \u{2014} I'll plan around the two of you.");
        }
        return Some(pick(
            &[
                "Then ".to_string() + &diet + " for you as well \u{2014} I'll plan around the two of you.",
                "Same for you, then: ".to_string() + &diet + ". I'll keep that straight.",
            ],
            memory,
            "",
        ));
    }
    let condition = limits.get("condition").cloned().unwrap_or(Value::None);
    if condition.truthy() {
        let items: Vec<String> = match &condition {
            Value::List(_) | Value::Tuple(_) => condition.items().iter().map(py_str).collect(),
            other => vec![py_str(other)],
        };
        let named = _the_things(&items);
        if !who.is_empty() {
            // Said the way it is said: the condition is what the person is, so it goes after them
            // with "is", not held off with "has the same thing".
            return Some(pick(
                &[
                    "Then ".to_string() + &_their_own(who) + " " + &_is_or_are(&[named.clone()]) + " " + &named
                        + " too \u{2014} I'll keep it in mind for both of you.",
                    "Right, ".to_string() + &named + " for " + &_their_own(who) + " as well, then. Both of you.",
                    "So ".to_string() + &_their_own(who) + " has that too. I'll keep the two of you in mind.",
                ],
                memory,
                "",
            ));
        }
        return Some(pick(
            &[
                "Then you have the same thing, ".to_string() + &named + " \u{2014} I'll keep that in mind.",
                "Same condition for you too, then: ".to_string() + &named + ".",
            ],
            memory,
            "",
        ));
    }
    None
}

/// Python: bot/memory/store_replies.py:728 `_their_own(who)`
///
/// "your son", "your daughter": the reply names the person the limit is now on.
pub fn _their_own(who: &str) -> String {
    let word = text::strip(who).to_lowercase();
    if !word.is_empty() && word != "i" && word != "me" {
        "your ".to_string() + &word
    } else {
        "your".to_string()
    }
}

rx!(MEATLESS_DIET);

/// Python: bot/memory/store_replies.py:739 `_the_back_this_turn(text, memory)`
///
/// The line that says what the bot has just stopped believing about somebody else, only on the
/// turn they lifted it. (Contract change: the generated stub returned `()`.)
pub fn _the_back_this_turn(text_: &str, memory: &Memory) -> Option<String> {
    let done = mget(memory, "reversal");
    if !done.is_dict() || done.get("said").map(|s| *s != Value::from(fold(text_))).unwrap_or(true) {
        return None;
    }
    let who = sget(done, "who");
    let food = text::strip(&sget(done, "food")).to_string();
    if food.is_empty() {
        return None;
    }
    let theirs = if FACTS_RELATIONS.contains(&who) { "your ".to_string() + &who } else { "them".to_string() };
    if done.get("kind").map(|k| *k == Value::from("taste")).unwrap_or(false) {
        // A taste is not a diet: "nothing off her plate" would read as food she cannot have.
        return Some(pick(
            &[
                "Right \u{2014} ".to_string() + &food + " back on the yes list for " + &theirs + ", then.",
                "Got it \u{2014} ".to_string() + &theirs + " likes " + &food + ", then.",
                "Noted \u{2014} ".to_string() + &food + " for " + &theirs + " from now on, then.",
            ],
            memory,
            "",
        ));
    }
    Some(pick(
        &[
            "Right \u{2014} ".to_string() + &food + " back on the plate for " + &theirs + ", then.",
            "Got it \u{2014} ".to_string() + &food + " is back on the menu for " + &theirs + ", then.",
            "Fair enough \u{2014} nothing off ".to_string() + &theirs + "'s plate, then.",
        ],
        memory,
        "",
    ))
}

/// Python: bot/memory/store_replies.py:764 `_still_holding(memory, animals)`
///
/// True when a pet of that kind is still on the store, so the whole kind is not gone.
pub fn _still_holding(memory: &Memory, animals: &[String]) -> bool {
    let held: HashSet<String> = user_facts(memory)
        .get("pets")
        .map(|p| p.items().to_vec())
        .unwrap_or_default()
        .iter()
        .filter(|p| p.is_dict())
        .map(|p| text::strip(&sget(p, "animal")).to_lowercase())
        .collect();
    animals.iter().any(|a| held.contains(&a.to_lowercase()))
}

// `_kind(item)` at store_replies.py:770 is redefined by `_kind(thing)` further down the module
// (line 10598), so every call reaches the later one: it is ported there.

/// `SIZE_UNITS`: a unit word -> the unit it is written as.
static SIZE_UNITS: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("SIZE_UNITS").str_map());
rx!(A_SIZE, A_TIME);
strset!(SIZE_LINE_FILLER);

/// Python: bot/memory/store_replies.py:798 `_a_size(text)`
///
/// The size a line names as one string ("2kg"), or None when it names none.
pub fn _a_size(text_: &str) -> Option<String> {
    let found = A_SIZE.search(text_)?;
    Some(found.str(1).to_string() + SIZE_UNITS.get(&found.str(2).to_lowercase()).expect("KeyError"))
}

/// Python: bot/memory/store_replies.py:805 `_only_names_the_size(text)`
///
/// True when the line is the size and nothing else, so what it says is the answer itself.
pub fn _only_names_the_size(text_: &str) -> bool {
    let flat = fold(text_);
    let Some(found) = A_SIZE.search(&flat) else { return false };
    if text::split(&flat).len() > 8 {
        return false;
    }
    let rest = text::split(&(flat[..found.start()].to_string() + " " + &flat[found.end()..]));
    rest.iter().all(|w| SIZE_LINE_FILLER.contains(text::strip_chars(w, ",.!?")))
}

static AND_SO_LEAD: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"^(?:(?:and|so|then|which means|that means)\b[\s,]*)+", crate::py::re::I));

/// Python: bot/memory/store_replies.py:814 `_the_clause_that_works_it_out(answer, size)`
///
/// The clause of `answer` that names this size with a time in it, or None.
pub fn _the_clause_that_works_it_out(answer: &str, size: &str) -> Option<String> {
    for clause in answer.split(|c| c == ',' || c == ';' || c == '.') {
        let flat = text::squash(clause);
        if flat.is_empty() || !A_TIME.is_match(&flat) || _a_size(&flat).as_deref() != Some(size) {
            continue;
        }
        let flat = AND_SO_LEAD.sub("", &flat);
        if !flat.is_empty() {
            return Some(text::upper_first(&flat) + ".");
        }
    }
    None
}

rx!(FEEDING_ASK, ANSWER_PORTION, SO_A_SIZE, MADE_ONLY, FOR_A_COUNT);
static HEAD_COUNTS: LazyLock<HashMap<String, i64>> =
    LazyLock::new(|| g("HEAD_COUNTS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap())).collect());
static HEAD_WORDS: LazyLock<HashMap<String, i64>> =
    LazyLock::new(|| g("HEAD_WORDS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap())).collect());
static SIZE_GRAMS: LazyLock<HashMap<String, f64>> =
    LazyLock::new(|| g("SIZE_GRAMS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_f64().unwrap())).collect());
strset!(BIRDS);

/// `HEAD_COUNTS.get(word) or HEAD_WORDS.get(word)`, then `int(word)`; None where Python returns.
fn _a_head_count(word: &str) -> Option<i64> {
    let count = HEAD_COUNTS.get(word).copied().filter(|c| *c != 0).or_else(|| HEAD_WORDS.get(word).copied().filter(|c| *c != 0));
    match count {
        Some(c) => Some(c),
        None => py_int(word),
    }
}

/// Python: bot/memory/store_replies.py:852 `_the_people_that_size_feeds(text, memory)`
///
/// What the amount in hand feeds, against the portion the knowledge keeps for that food, or
/// None. (Contract change: the generated stub returned `()`.)
pub fn _the_people_that_size_feeds(text_: &str, memory: &Memory) -> Option<String> {
    let said = fold(text_);
    let found = FEEDING_ASK.search(&said)?;
    let count = _a_head_count(&found.str(1).to_lowercase())?;
    if !(2..=40).contains(&count) {
        return None;
    }
    let turns = &memory.turns;
    let held = said.clone()
        + " "
        + &turns[turns.len().saturating_sub(3)..].iter().map(|t| t.user.to_lowercase()).collect::<Vec<_>>().join(" ");
    let answers = turns[turns.len().saturating_sub(2)..].iter().map(|t| t.bot.to_lowercase()).collect::<Vec<_>>().join(" ");
    let portion = _the_portion_for_the_food(&held, &answers)?;
    let size = _a_size(text_).or_else(|| _the_size_on_the_screen(turns))?;
    _the_portion_reply(&size, &portion, count)
}

/// Python: bot/memory/store_replies.py:897 `_the_amount_they_made(text, memory)`
///
/// A line saying they only made this much, read against the total an answer worked out, or
/// None. (Contract change: the generated stub returned `()`.)
pub fn _the_amount_they_made(text_: &str, memory: &Memory) -> Option<String> {
    let said = fold(text_);
    if !MADE_ONLY.is_match(&said) {
        return None;
    }
    let size = _a_size(text_)?;
    let turns = &memory.turns;
    let answer = turns[turns.len().saturating_sub(2)..].iter().map(|t| t.bot.to_lowercase()).collect::<Vec<_>>().join(" ");
    let last_bot = turns.last().map(|t| t.bot.to_lowercase())?;
    let counted = FOR_A_COUNT.search(&last_bot)?;
    let word = counted.str(1);
    let count = _a_head_count(word)?;
    if !(2..=40).contains(&count) {
        return None;
    }
    let held = said.clone()
        + " "
        + &turns[turns.len().saturating_sub(3)..].iter().map(|t| t.user.to_lowercase()).collect::<Vec<_>>().join(" ");
    // No portion for any food in the chat: the weight could be anything, so the line is left to
    // whoever else reads it.
    let portion = _the_portion_for_the_food(&held, &answer)?;
    _the_portion_reply(&size, &portion, count)
}

/// A portion the knowledge keeps for a food: (amount per person, unit, noun, on the bone).
#[derive(Clone, Debug, PartialEq)]
pub struct Portion {
    pub per: f64,
    pub unit: String,
    pub noun: String,
    pub bone: bool,
}

/// Python: bot/memory/store_replies.py:938 `_the_portion_reply(size, portion, count)`
///
/// One size read against one portion, or None when it does not feed anyone.
pub fn _the_portion_reply(size: &str, portion: &Portion, count: i64) -> Option<String> {
    let Portion { per, unit, noun, bone } = portion;
    if unit != "g" {
        return None; // pieces and millilitres are not a weight the size can be read against
    }
    let grams = _size_in_grams(size).filter(|g| *g != 0.0)?;
    let feeds = py_floordiv(grams, *per) as i64;
    if feeds < 1 {
        return None;
    }
    let line = format!(
        "{size} of {noun}{bone}is about {per}g a person, so it feeds {feeds}",
        size = size,
        noun = if noun.is_empty() { String::new() } else { noun.clone() + " " },
        bone = if *bone { "on the bone " } else { "" },
        per = fmt_g(*per),
        feeds = _a_number_word(feeds)
    );
    if feeds >= count {
        if feeds > count {
            return Some(line + &format!(" \u{2014} enough for {}, with a second helping in it.", _a_number_word(count)));
        }
        return Some(line + "."); // the number of people it feeds is the answer, so not said twice
    }
    let wanted = count as f64 * per;
    let wants = if wanted >= 1000.0 { fmt_g(wanted / 1000.0) + "kg" } else { fmt_g(wanted) + "g" };
    Some(line + &format!(", and {} wants about {}.", _a_number_word(count), wants))
}

static PER_PERSON_ALL: LazyLock<Vec<(Vec<String>, f64, String, String)>> = LazyLock::new(|| {
    let mut rows = Vec::new();
    for name in ["bot.knowledge.units.PER_PERSON", "bot.knowledge.units.PER_PERSON_WHEN_COUNTED"] {
        for row in data::get(name).items() {
            let r = row.items();
            rows.push((r[0].strs(), r[1].as_f64().unwrap(), r[2].str().to_string(), r[3].str().to_string()));
        }
    }
    rows
});
static BIRD_GRAMS: LazyLock<f64> = LazyLock::new(|| data::get("bot.knowledge.units.BIRD_GRAMS").as_f64().unwrap());

/// `re.search(r"\b" + re.escape(name) + r"\b", text)`.
fn says_word(name: &str, text_: &str) -> bool {
    Re::new(&format!(r"\b{}\b", crate::py::re::escape(name)), 0).is_match(text_)
}

/// Python: bot/memory/store_replies.py:962 `_the_portion_for_the_food(held, answers)`
///
/// The portion the knowledge keeps for the food this chat is about, or None; the most specific
/// name wins.
pub fn _the_portion_for_the_food(held: &str, answers: &str) -> Option<Portion> {
    let mut best: Option<(usize, Portion)> = None;
    for (names, amount, unit, noun) in PER_PERSON_ALL.iter() {
        for name in names {
            if !says_word(name, held) {
                continue;
            }
            let score = text::len(name) + if says_word(name, answers) { 2 } else { 0 };
            if let Some((b, _)) = &best {
                if score <= *b {
                    continue;
                }
            }
            if BIRDS.contains(name) {
                best = Some((score, Portion { per: *BIRD_GRAMS, unit: "g".into(), noun: name.clone(), bone: true }));
            } else {
                best = Some((score, Portion { per: *amount, unit: unit.clone(), noun: noun.clone(), bone: false }));
            }
        }
    }
    let (_, portion) = best?;
    if portion.unit != "g" {
        return Some(portion);
    }
    // The answer's own portion is the one on the screen, so it is read first.
    if let Some(stated) = ANSWER_PORTION.search(answers) {
        return Some(Portion {
            per: py_float(stated.str(1)).expect("ValueError"),
            unit: "g".into(),
            noun: portion.noun,
            bone: answers.contains("on the bone"),
        });
    }
    Some(portion)
}

/// Python: bot/memory/store_replies.py:994 `_the_size_on_the_screen(turns)`
///
/// The size the chat has in hand: the last one they named, or the one the answer worked out.
pub fn _the_size_on_the_screen(turns: &[Turn]) -> Option<String> {
    for turn in turns.iter().rev().take(3) {
        if let Some(size) = _a_size(&turn.user) {
            return Some(size);
        }
    }
    for turn in turns.iter().rev().take(2) {
        if let Some(worked) = SO_A_SIZE.search(&turn.bot) {
            return Some(worked.str(1).to_string() + SIZE_UNITS.get(&worked.str(2).to_lowercase()).expect("KeyError"));
        }
    }
    None
}

/// Python: bot/memory/store_replies.py:1012 `_size_in_grams(size)`
///
/// The weight a size names, in grams, or None when it is not a weight.
pub fn _size_in_grams(size: &str) -> Option<f64> {
    let found = A_SIZE.match_(size)?;
    let grams = *SIZE_GRAMS.get(&found.str(2).to_lowercase())?;
    if grams == 0.0 {
        return None;
    }
    Some(py_float(found.str(1))? * grams)
}

/// Python: bot/memory/store_replies.py:1025 `_the_figure_for_the_size_they_named(text, memory)`
///
/// The figure the answer on the screen worked out for the size this line names, or None.
/// (Contract change: the generated stub returned `()`.)
pub fn _the_figure_for_the_size_they_named(text_: &str, memory: &Memory) -> Option<String> {
    let size = _a_size(text_)?;
    if !_only_names_the_size(text_) {
        return None;
    }
    let turns = &memory.turns;
    if turns.is_empty() {
        return None;
    }
    if turns.iter().any(|t| _a_size(&t.user).as_deref() == Some(size.as_str())) {
        return None;
    }
    // The size the answer worked its own arithmetic out for is already in front of them.
    let last_bot = &turns[turns.len() - 1].bot;
    if let Some(worked) = SO_A_SIZE.search(last_bot) {
        if worked.str(1).to_string() + SIZE_UNITS.get(&worked.str(2).to_lowercase()).expect("KeyError") == size {
            return None;
        }
    }
    _the_clause_that_works_it_out(last_bot, &size)
}

static CORRECTED_SAY: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("CORRECTED_SAY").str_map());
static DENIAL_SAY: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("DENIAL_SAY").str_map());
static RETRACT_SAY: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("RETRACT_SAY").str_map());

/// `v if isinstance(v, list) else [v]`, each item in Python's `str()`.
fn listed_strs(v: &Value) -> Vec<String> {
    match v {
        Value::List(items) => items.iter().map(py_str).collect(),
        other => vec![py_str(other)],
    }
}

/// Python: bot/memory/store_replies.py:1059 `_denial_this_turn(text, memory)`
///
/// The short line that says what the bot has just stopped believing, on the turn they said it.
/// (Contract change: the generated stub returned `()`.)
pub fn _denial_this_turn(text_: &str, memory: &Memory) -> Option<String> {
    let gone = mget(memory, "denial").clone();
    if !gone.is_dict() || gone.get("said").map(|s| *s != Value::from(fold(text_))).unwrap_or(true) {
        return None;
    }
    let truthy = |k: &str| gone.get(k).map(Value::truthy).unwrap_or(false);
    let others_keys = ["allergies", "condition", "diet"];
    if truthy("who") {
        // Somebody else's take-back is theirs, not the speaker's.
        let items: Vec<String> =
            gone.get("allergies").filter(|a| a.truthy()).map(|a| py_list(a).iter().map(py_str).filter(|s| !s.is_empty()).collect()).unwrap_or_default();
        if items.is_empty() {
            return None;
        }
        let who = py_str(gone.get("who").unwrap());
        let theirs = if FACTS_RELATIONS.contains(&who) { "your ".to_string() + &who } else { "them".to_string() };
        // The line lifts a restriction, so it says the food is fine now.
        let things = if items.len() == 1 {
            items[0].clone()
        } else {
            items[..items.len() - 1].join(", ") + " and " + &items[items.len() - 1]
        };
        return Some(pick(
            &[
                "Right \u{2014} ".to_string() + &things + " back on the table for " + &theirs + ", then.",
                "Good \u{2014} no allergy to ".to_string() + &things + " for " + &theirs + ", then.",
                "Noted \u{2014} ".to_string() + &things + " back on for " + &theirs + ".",
            ],
            memory,
            "",
        ));
    }
    if truthy("pets_died") && !others_keys.iter().any(|k| truthy(k)) {
        return None; // a death is not a take-back to say back to them
    }
    let kept = crate::memory::facts::extract(text_, false).val();
    if truthy("pets_given") && !others_keys.iter().chain(["pets"].iter()).any(|k| truthy(k)) {
        return None; // one of a set, so "no cat, then" would say they have none left
    }
    for key in ["allergies", "condition", "diet", "pets"] {
        let value = gone.get(key).cloned().unwrap_or(Value::None);
        if !value.truthy() {
            continue;
        }
        let shown: String;
        if key == "pets" {
            let given_line = ret_str(crate::memory::ack::pet_given_away(text_, memory));
            if !given_line.is_empty() {
                // The animal was given to a new home, so the line they get is the warm one and not
                // the record being crossed out.
                return Some(given_line);
            }
            // A pet is a record, so the line names the animal rather than the record.
            let mut animals: Vec<String> = py_list(&value)
                .iter()
                .filter(|p| p.is_dict())
                .map(|p| {
                    let a = sget(p, "animal");
                    if a.is_empty() { "pet".to_string() } else { a }
                })
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
            animals.sort();
            if !truthy("pets_all") && !animals.is_empty() && _still_holding(memory, &animals) {
                continue; // one of several: "no cat, then" would say they have none left
            }
            shown = if truthy("pets_all") {
                "pets".to_string()
            } else {
                let joined = animals.join(" or ");
                if joined.is_empty() { "pets".to_string() } else { joined }
            };
        } else {
            shown = listed_strs(&value).join(", ");
        }
        let fresh = kept.get(key).cloned().unwrap_or(Value::None);
        if fresh.truthy() && CORRECTED_SAY.contains_key(key) {
            let now = listed_strs(&fresh).join(", ");
            return Some(fmt_named(&CORRECTED_SAY[key], &[("thing", &now), ("Thing", &text::upper_first(&now)), ("old", &shown)]));
        }
        if key == "diet" && !MEATLESS_DIET.is_match(&shown) {
            // A diet that is itself a negation is put back positively.
            if let Some(put_back) = _a_diet_put_back(&shown) {
                return Some(pick(
                    &[
                        "Right \u{2014} ".to_string() + &put_back + " again, then.",
                        "Got it \u{2014} ".to_string() + &put_back + " again, then.",
                        "Fair enough \u{2014} ".to_string() + &put_back + " again, then.",
                    ],
                    memory,
                    "",
                ));
            }
            let label = if text::strip(&shown).to_lowercase() == "on a diet" { "dieting".to_string() } else { _diet_words(&shown) };
            return Some(pick(
                &[
                    "Right \u{2014} no more ".to_string() + &label + ", then.",
                    "Right, ".to_string() + &label + " is off, then.",
                    "Fair enough \u{2014} no more ".to_string() + &label + ", then.",
                ],
                memory,
                "",
            ));
        }
        let shape = DENIAL_SAY.get(key).cloned().unwrap_or_else(|| "{Thing} \u{2014} scratch that, then.".to_string());
        return Some(fmt_named(&shape, &[("thing", &shown), ("Thing", &text::upper_first(&shown))]));
    }
    None
}

/// Python: bot/memory/store_replies.py:1134 `_retraction_this_turn(text, memory)`
///
/// The short line that says the last thing they said is off the record, on the turn they said
/// it. (Contract change: the generated stub returned `()`.)
pub fn _retraction_this_turn(text_: &str, memory: &Memory) -> Option<String> {
    let gone = mget(memory, "retraction");
    if !gone.is_dict() || gone.get("said").map(|s| *s != Value::from(fold(text_))).unwrap_or(true) {
        return None;
    }
    let keys: Vec<Value> = gone.get("keys").filter(|k| k.truthy()).map(py_list).unwrap_or_default();
    let shown: Vec<String> = gone.get("shown").filter(|k| k.truthy()).map(py_list).unwrap_or_default().iter().filter(|i| i.truthy()).map(py_str).collect();
    let has = |k: &str| keys.iter().any(|x| *x == Value::from(k));
    if has("pets") && !shown.is_empty() {
        let thing = shown[..shown.len().min(2)].join(" and ");
        return Some(pick(
            &[
                "Right \u{2014} ".to_string() + &thing + " is off the record, then.",
                "Taken back \u{2014} no ".to_string() + &thing + " on my list any more.",
            ],
            memory,
            "",
        ));
    }
    if has("plans") && !keys.iter().any(|k| *k != Value::from("plans")) {
        return Some(pick(
            &[
                "Right \u{2014} that plan is off the record, then.",
                "Taken back: I\u{2019}ve nothing about that trip from you now.",
                "Scratch that, then \u{2014} the plan is out.",
            ],
            memory,
            "",
        ));
    }
    let phrase = keys.first().and_then(|k| k.as_str()).and_then(|k| RETRACT_SAY.get(k)).cloned();
    if let Some(phrase) = phrase.filter(|p| !p.is_empty()) {
        return Some(pick(
            &[
                "Right \u{2014} ".to_string() + &phrase + " is off the record, then.",
                "Dropped. ".to_string() + &text::upper_first(&phrase) + " is gone.",
                "Taken back \u{2014} I\u{2019}ve nothing from you on record about it now.".to_string(),
            ],
            memory,
            "",
        ));
    }
    Some(pick(
        &["Right, that\u{2019}s off the record then.", "Taken back \u{2014} I\u{2019}ve nothing on record about it now."],
        memory,
        "",
    ))
}

rx!(THEIR_NAMES, ASKS_IT_BACK);

/// The keys `_told_readers()` hands out, in Python's order.
pub const TOLD_READER_KEYS: [&str; 11] =
    ["allergies", "budget", "diet", "age", "name", "job", "plan", "places", "pets", "family", "kids"];

/// Python: bot/memory/store_replies.py:1166 `_told_readers()`
///
/// The store's own answer for each kind of fact, for `told` (bot/memory/told.py). Python hands
/// out lambdas; a Rust `Value` cannot hold a closure, so each key holds a function reference
/// `bot.memory.store_replies:_told_readers:<key>` and [`run_told_reader`] runs it.
pub fn _told_readers() -> Value {
    let mut d = Value::dict();
    for key in TOLD_READER_KEYS {
        d.set_key(
            key,
            Value::Fn(std::sync::Arc::new(data::FnRef {
                id: format!("bot.memory.store_replies:_told_readers:{key}"),
                cells: Vec::new(),
                defaults: Vec::new(),
            })),
        );
    }
    d
}

/// Run the reader `_told_readers()[key]`: `reader(memory)` for most keys, `reader(memory,
/// animal)` for "pets", `reader(word)` for "family" (`arg` is the animal or the word). The value
/// is the reader's own (a str, or None where the Python reader returns None).
pub fn run_told_reader(key: &str, memory: &mut Memory, arg: &str) -> Value {
    match key {
        "allergies" => _fact_reply(memory, "allergic to", None).val(),
        "budget" => _fact_reply(memory, "budget", None).val(),
        "diet" => _diet_reply(memory, "").val(),
        "age" => Value::from(_age_reply(memory)),
        "name" => Value::from(_name_reply(memory, "my name")),
        "job" => Value::from(_job_reply(memory, false)),
        // The plan read out whole: the told reader asks about the plan, not for a part of it.
        "plan" => _plans_reply(memory, "what am i planning").val(),
        "places" => _places_reply(memory, "where do i live").val(),
        "pets" => Value::from(_told_pet_reply(memory, arg).unwrap_or_default()),
        // A puppy is a dog and a kitten is a cat, and `told` needs the store's own reading.
        "family" => Value::from(crate::memory::relations::_animal_family(arg)),
        "kids" => Value::from(_kids_reply(memory, "how many kids do i have").unwrap_or_default()),
        _ => panic!("KeyError: {key:?} is not a told reader"),
    }
}

rx!(OWN_ANIMAL_LINE, NOT_THEIR_ANIMALS);
static AN_ANIMAL_LINE: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\b(?:dogs?|cats?|kittens?|pupp\w*|animals?|pets?|rabbits?|hamsters?|fish|horses?)\b", 0)
});

/// Python: bot/memory/store_replies.py:1202 `_the_line_before_is_their_own_animal(memory, back=3)`
///
/// True when the last line that named an animal named one of the person's own.
pub fn _the_line_before_is_their_own_animal(memory: &Memory, back: usize) -> bool {
    let turns = &memory.turns;
    let from = if back == 0 { 0 } else { turns.len().saturating_sub(back) };
    for turn in turns[from..].iter().rev() {
        let said = text::strip(&turn.user).to_lowercase();
        if said.is_empty() {
            continue;
        }
        if NOT_THEIR_ANIMALS.is_match(&said) {
            return false;
        }
        if OWN_ANIMAL_LINE.is_match(&said) {
            return true;
        }
        if AN_ANIMAL_LINE.is_match(&said) {
            // An animal line all the same, and not one of theirs: the walk stops here.
            return false;
        }
    }
    false
}

/// Python: bot/memory/store_replies.py:1224 `_the_names_again(folded, low, memory)`
///
/// The names of their animals, asked back with the pronoun instead of the animal ("what are
/// they called again?"); None for Python's "".
pub fn _the_names_again(folded: &str, low: &str, memory: &mut Memory) -> Option<String> {
    if text::strip(low).is_empty() {
        return None;
    }
    // "again" is filler the ask is bare of, so the pointer back is read on the line as typed.
    let said = if folded.is_empty() { low } else { folded };
    if !ASKS_IT_BACK.is_match(said) || !THEIR_NAMES.is_match(low) {
        return None;
    }
    let pets = dicts(user(memory).get("pets").unwrap_or(&Value::None));
    if pets.is_empty() {
        // No animal of theirs is on record, so this pronoun points at something out in the world.
        return None;
    }
    if !_the_line_before_is_their_own_animal(memory, 3) {
        // The pronoun is pointing at animals that are not theirs.
        return None;
    }
    _fact_reply(memory, "what are my pets called", Some("pet"))
}

/// Python: bot/memory/store_replies.py:1255 `_the_one_animal_with_a_name(memory)`
///
/// What the one animal they named is called, or None ("") when the chat holds none or several.
pub fn _the_one_animal_with_a_name(memory: &mut Memory) -> Option<String> {
    let pets: Vec<Value> = dicts(user(memory).get("pets").unwrap_or(&Value::None))
        .into_iter()
        .filter(|p| p.get("name").map(Value::truthy).unwrap_or(false))
        .collect();
    if pets.len() != 1 {
        return None;
    }
    let mut animal = sget(&pets[0], "animal");
    if animal.is_empty() {
        animal = "pet".into();
    }
    _fact_reply(memory, &("what is my ".to_string() + &animal + " called"), None)
}

/// Python: bot/memory/store_replies.py:1269 `_told_pet_reply(memory, animal)`
///
/// What the store says about a kind of pet, for "have i told you about the cat".
pub fn _told_pet_reply(memory: &mut Memory, animal: &str) -> Option<String> {
    let pets: Vec<Value> = dicts(user(memory).get("pets").unwrap_or(&Value::None))
        .into_iter()
        .filter(|p| crate::memory::relations::_animal_family(&sget(p, "animal")) == animal)
        .collect();
    if pets.is_empty() {
        return None;
    }
    if pets.iter().any(|p| p.get("name").map(Value::truthy).unwrap_or(false)) {
        return _fact_reply(memory, &("what is my ".to_string() + animal + " called"), None);
    }
    _fact_reply(memory, &("how many ".to_string() + animal + "s do i have"), None)
}

rx!(NO_RECORD);
strset!(ASK_AND_PERSON, GENERIC_WORDS);
static LETTER_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// Python: bot/memory/store_replies.py:1289 `_says_more_than_the_ask(low)`
///
/// True when the line carries a word the ask about that fact does not need.
pub fn _says_more_than_the_ask(low: &str) -> bool {
    for word in LETTER_WORDS.findall_str(&low.to_lowercase()) {
        if text::len(&word) < 3 || ASK_AND_PERSON.contains(&word) || GENERIC_WORDS.contains(&word) {
            continue;
        }
        if ASK_AND_PERSON.contains(text::rstrip_chars(&word, "s")) {
            continue;
        }
        return true;
    }
    false
}

rx!(HOW_FAR_ALONG, OWN_LENGTH_ASK);

/// The entries of `others(memory)` with an int `weeks`.
fn _holding_weeks(memory: &mut Memory) -> Vec<Value> {
    others(memory).items().iter().filter(|e| e.is_dict() && is_int(e.get("weeks"))).cloned().collect()
}

/// The entries the ask names by their relation (`re.search(r"\b" + re.escape(who) + r"\b", said)`).
fn _named_in(holding: &[Value], said: &str) -> Vec<Value> {
    holding.iter().filter(|e| says_word(&sget(e, "who"), said)).cloned().collect()
}

/// Python: bot/memory/store_replies.py:1311 `_how_far_along(memory, low)`
///
/// The weeks a pregnancy the chat was told has run, said as a length: "Eight weeks.".
/// (Contract change: the generated stub returned `()`.)
pub fn _how_far_along(memory: &mut Memory, low: &str) -> Option<String> {
    let said = low;
    if !HOW_FAR_ALONG.is_match(said) {
        return None;
    }
    let mut holding = _holding_weeks(memory);
    // A relation the ask names is the one it is about, own pregnancy or not.
    let named = _named_in(&holding, said);
    if named.len() == 1 {
        holding = named;
    } else if OWN_LENGTH_ASK.is_match(said) {
        // The person's own pregnancy is kept on them, not in `others`.
        let own = user(memory);
        if is_int(own.get("weeks")) {
            return Some(text::capitalize(&_a_number_word(own.get("weeks").unwrap().as_int().unwrap())) + " weeks.");
        }
        if own.get("pregnant").map(Value::truthy).unwrap_or(false) {
            // The pregnancy is on file and the length never was.
            return Some(
                "You didn't say how many weeks \u{2014} tell me and I'll keep it, so the meal plans fit where you are."
                    .to_string(),
            );
        }
    }
    if holding.is_empty() {
        return None;
    }
    if holding.len() > 1 {
        // Two pregnancies on file: the pronoun, and without it the one told last.
        let who = crate::memory::relations::_person_from_pronoun(said, memory, &[]);
        if !who.is_empty() {
            let chosen: Vec<Value> = holding.iter().filter(|e| sget(e, "who").to_lowercase() == who).cloned().collect();
            holding = if chosen.is_empty() { holding[holding.len() - 1..].to_vec() } else { chosen };
        }
    }
    let weeks = holding[holding.len() - 1].get("weeks").unwrap().as_int().unwrap();
    Some(text::capitalize(&_a_number_word(weeks)) + " weeks.")
}

/// Python: bot/memory/store_replies.py:1350 `_a_length_i_have(memory, low)`
///
/// True when a length ask is about a pregnancy this chat already holds the weeks for.
pub fn _a_length_i_have(memory: &mut Memory, low: &str) -> bool {
    let said = low;
    if !HOW_FAR_ALONG.is_match(said) {
        return false;
    }
    let holding = _holding_weeks(memory);
    let named = _named_in(&holding, said);
    if named.len() == 1 {
        return true;
    }
    if OWN_LENGTH_ASK.is_match(said) {
        let own = user(memory);
        return is_int(own.get("weeks")) || own.get("pregnant").map(Value::truthy).unwrap_or(false);
    }
    !holding.is_empty()
}

/// Python: bot/memory/store_replies.py:1370 `_a_limit_kept_for_them(low, memory)`
///
/// True when the person the line names has a diet or a condition on record.
pub fn _a_limit_kept_for_them(low: &str, memory: &mut Memory) -> bool {
    let who = _someone_else_named(low, memory);
    for known in others(memory).items() {
        if known.get("who").cloned().unwrap_or(Value::None) != Value::from(who.as_str()) {
            continue;
        }
        return known.get("condition").map(Value::truthy).unwrap_or(false) || known.get("diet").map(Value::truthy).unwrap_or(false);
    }
    false
}

rx!(PET_NAME_AGAIN, PET_NAME_SETTLED, NAME_PICKED_AGAIN);
/// `r"\b(?:" + "|".join(extract.RELATIONS) + r")\b"`.
static A_RELATION_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(&format!(r"\b(?:{})\b", FACTS_RELATIONS.join("|")), 0));

/// `" ".join(str(text or "").lower().replace("’", "'").split())`.
fn fold_quote(text_: &str) -> String {
    text::squash(&text_.to_lowercase().replace('\u{2019}', "'"))
}

/// The pets the store holds with a name, in order.
fn named_pets(memory: &mut Memory) -> Vec<Value> {
    user(memory)
        .get("pets")
        .map(|p| p.items().to_vec())
        .unwrap_or_default()
        .into_iter()
        .filter(|p| p.is_dict() && p.get("name").map(Value::truthy).unwrap_or(false))
        .collect()
}

/// Python: bot/memory/store_replies.py:1403 `_the_pick_asked_again(text, memory)`
///
/// The name the store holds, when the ask is the choosing said with "pick" or "settle".
pub fn _the_pick_asked_again(text_: &str, memory: &mut Memory) -> Option<String> {
    let low = fold_quote(text_);
    if low.is_empty() || !NAME_PICKED_AGAIN.is_match(&low) {
        return None;
    }
    if A_RELATION_WORD.is_match(&low) {
        return None;
    }
    let named = named_pets(memory);
    let pet = named.last()?.clone();
    let name = sget(&pet, "name");
    let mut animal = sget(&pet, "animal");
    if animal.is_empty() {
        animal = "pet".into();
    }
    if !low.contains("name") {
        // Without the word "name" in it, a bare "what did i pick" is only about the pet while the
        // chat is still on the pet.
        let subject = text::split(&crate::memory::topics::subject(memory).to_lowercase());
        if subject.is_empty() || (!subject.contains(&animal.to_lowercase()) && !subject.contains(&name.to_lowercase())) {
            return None;
        }
        let turns = &memory.turns;
        let recent: Vec<String> = turns[turns.len().saturating_sub(3)..].iter().map(|t| t.user.clone()).collect();
        let said: Vec<String> = recent.iter().map(|line| ret_str(crate::memory::facts::bare_name(line)).to_lowercase()).collect();
        if name.is_empty() || !said.contains(&name.to_lowercase()) {
            return None;
        }
    }
    let called = crate::memory::relations::_called(&name);
    Some(_vary(
        &[
            called.clone() + " \u{2014} that is the one you picked for your " + &animal + ".",
            "You went for ".to_string() + &called + ". I have it written down.",
            called.clone() + ". That is your " + &animal + "'s name.",
        ],
        memory,
    ))
}

/// Python: bot/memory/store_replies.py:1445 `_what_the_pet_is_called(text, memory)`
///
/// The name the store holds for the animal the line points at: "Milo — you settled on that."
pub fn _what_the_pet_is_called(text_: &str, memory: &mut Memory) -> Option<String> {
    let low = fold_quote(text_);
    if low.is_empty() || !(PET_NAME_AGAIN.is_match(&low) || PET_NAME_SETTLED.is_match(&low)) {
        return None;
    }
    if A_RELATION_WORD.is_match(&low) {
        return None;
    }
    let named = named_pets(memory);
    if named.is_empty() {
        // The animal is in play and has no name yet: the honest line is about the name.
        if PET_NAME_SETTLED.is_match(&low) {
            return None; // no name yet, so the naming question still wants names
        }
        let animal = _the_animal_of_the_pick(memory);
        if !animal.truthy() || animal == Value::from("pet") {
            return None;
        }
        return Some("You haven't told me a name for your ".to_string() + &py_str(&animal) + " yet.");
    }
    let pet = named.last().unwrap();
    let name = crate::memory::relations::_called(&py_str(pet.get("name").unwrap()));
    let mut animal = sget(pet, "animal");
    if animal.is_empty() {
        animal = "pet".into();
    }
    Some(_vary(
        &[
            name.clone() + " \u{2014} that is what you settled on for your " + &animal + ".",
            "Your ".to_string() + &animal + " is called " + &name + ".",
            name.clone() + ". I have that one written down.",
        ],
        memory,
    ))
}

/// Python: bot/memory/store_replies.py:1475 `_the_name_they_picked(text, memory)`
///
/// The name the person settled on, said back: "Pepper it is. I'll remember that for your cat."
pub fn _the_name_they_picked(text_: &str, memory: &mut Memory) -> Option<String> {
    let mut said = ret_str(crate::memory::facts::pet_name_only(text_, false));
    if said.is_empty() && _offered_here(memory).truthy() {
        // Only the list this module put in front of them is this module's turn.
        said = _name_they_answered(text_, memory).unwrap_or_default();
    }
    if said.is_empty() {
        return None;
    }
    let pets = user(memory).get("pets").map(|p| p.items().to_vec()).unwrap_or_default();
    let pet = pets.iter().rev().find(|one| one.is_dict() && sget(one, "name").to_lowercase() == said.to_lowercase())?.clone();
    let mut kept = sget(&pet, "name");
    if kept.is_empty() {
        kept = said.clone();
    }
    let name = crate::memory::relations::_called(&kept);
    let mut animal = sget(&pet, "animal");
    if animal.is_empty() {
        animal = "pet".into();
    }
    Some(_vary(
        &[
            name.clone() + " it is. I'll remember that for your " + &animal + ".",
            name.clone() + " \u{2014} good choice, and easy to shout across a park.",
            "Right, ".to_string() + &name + ". That is the one from here on.",
        ],
        memory,
    ))
}

/// Python: bot/memory/store_replies.py:1508 `_offered_here(memory)`
///
/// The names the bot itself put in front of them on the turn before, and what they are for:
/// {"animal", "names"}, or None.
pub fn _offered_here(memory: &Memory) -> Value {
    let offered = crate::memory::facts::names_offered(memory).val();
    if !offered.truthy() {
        return Value::None;
    }
    let animal = offered.get("animal").cloned().expect("KeyError: 'animal'");
    let names = offered.get("names").cloned().expect("KeyError: 'names'");
    dict_of(vec![("animal", Value::from(crate::memory::relations::_animal_family(&py_str(&animal)))), ("names", names)])
}

/// Python: bot/memory/store_replies.py:1518 `_name_they_answered(text, memory)`
///
/// The name chosen in answer to the bot's own question or list (`facts.a_name_picked`).
/// (Contract change: the generated stub returned `()`.)
pub fn _name_they_answered(text_: &str, memory: &Memory) -> Option<String> {
    ret_opt(crate::memory::facts::a_name_picked(text_, memory))
}

/// Python: bot/memory/store_replies.py:1530 `_the_animal_of_the_pick(memory)`
///
/// Which animal a name just picked is for: the one the list was about, or the one in play.
pub fn _the_animal_of_the_pick(memory: &mut Memory) -> Value {
    let offered = _offered_here(memory);
    if offered.truthy() && offered.get("animal").map(Value::truthy).unwrap_or(false) {
        return offered.get("animal").unwrap().clone();
    }
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(3)..].iter().rev() {
        let line = turn.user.to_lowercase();
        for word in LETTER_WORDS.findall_str(&line) {
            if FACTS_ANIMALS.contains(&word) {
                return Value::from(crate::memory::relations::_animal_family(&word));
            }
        }
    }
    let pets = user(memory).get("pets").map(|p| p.items().to_vec()).unwrap_or_default();
    for pet in &pets {
        if pet.is_dict() && pet.get("animal").map(Value::truthy).unwrap_or(false) {
            return Value::from(crate::memory::relations::_animal_family(&py_str(pet.get("animal").unwrap())));
        }
    }
    Value::None
}

strset!(PLAN_LOW_START);

/// Python: bot/memory/store_replies.py:1552 `_correction_this_turn(text, memory)`
///
/// The short note that says which value the bot goes with, on the turn of the correction.
pub fn _correction_this_turn(text_: &str, memory: &mut Memory) -> Option<String> {
    let fixed = mget(memory, "correction").clone();
    if !fixed.is_dict() {
        return None;
    }
    if fixed.get("said").map(|s| *s != Value::from(fold(text_))).unwrap_or(true) {
        return None;
    }
    nz(_correction_said(&fixed, memory))
}

static PET_TAIL_MANY: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.ack.PET_TAIL_MANY"));

/// Python: bot/memory/store_replies.py:1561 `_the_second_pet_said(fixed, memory)`
///
/// A line that adds a second animal of a kind is answered as a pet of theirs, not echoed.
pub fn _the_second_pet_said(fixed: &Value, memory: &mut Memory) -> String {
    let animal = text::strip(&sget(fixed, "animal")).to_lowercase();
    let pets = dicts(user(memory).get("pets").unwrap_or(&Value::None));
    let named: Vec<String> = pets
        .iter()
        .filter(|p| text::strip(&sget(p, "animal")).to_lowercase() == animal && p.get("name").map(Value::truthy).unwrap_or(false))
        .map(|p| sget(p, "name"))
        .collect();
    if named.len() < 2 {
        return String::new();
    }
    // A name is written the way a name is written, whatever case it was filed in.
    let named: Vec<String> = named.iter().map(|n| text::upper_first(n)).collect();
    // "Two cats, Milo and Max": the count the store holds, said in words.
    let many = crate::memory::corrected::_many(named.len() as i64, &Value::from(animal.as_str()));
    let thing = text::upper_first(&many) + ", " + &crate::memory::relations::_a_list_of_names(&named);
    fmt_named(&pick(&PET_TAIL_MANY, memory, &sget(fixed, "said")), &[("Thing", &thing)])
}

static PET_RENAMED_SAY: LazyLock<Vec<String>> = LazyLock::new(|| g("PET_RENAMED_SAY").strs());
static CORRECTION_SAY_PAIR: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("CORRECTION_SAY_PAIR").str_map());
static CORRECTION_SAY_DROP: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("CORRECTION_SAY_DROP").str_map());
static CORRECTION_SAY: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("CORRECTION_SAY").str_map());
sval!(BARE_ECHO_ONLY);

/// `s[:1].lower() + s[1:]`.
fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().chain(c).collect(),
    }
}

/// `d.get(key)` looked up by a value that may not be a str (None when it is not one).
fn str_key<'a>(map: &'a HashMap<String, String>, key: Option<&Value>) -> Option<&'a String> {
    key.and_then(|k| k.as_str()).and_then(|k| map.get(k))
}

/// Python: bot/memory/store_replies.py:1584 `_correction_said(fixed, memory=None)`
///
/// The short line that says which value the bot is going with now.
pub fn _correction_said(fixed: &Value, memory: &mut Memory) -> String {
    if !fixed.is_dict() {
        return String::new();
    }
    let key = fixed.get("key").cloned().unwrap_or(Value::None);
    if key == Value::from("pets_more") {
        return _the_second_pet_said(fixed, memory);
    }
    // A fact about someone else is worded by the reading that kept it, so its note is the whole
    // line rather than a value in a shape.
    if fixed.get("note").map(Value::truthy).unwrap_or(false) {
        return py_str(fixed.get("note").unwrap());
    }
    let raw = fixed.get("value").cloned().unwrap_or(Value::None);
    if !raw.truthy() {
        return String::new();
    }
    let mut value = py_str(&raw);
    if key == Value::from("job") {
        // A job is said in the words the person used for it.
        let known = user(memory);
        let note = _job_note(&value, &known);
        if !note.is_empty() {
            return note;
        }
    }
    let in_a_sentence = ["budget", "plan", "birthday"].iter().any(|k| key == Value::from(*k))
        && fixed.get("sentence").map(Value::truthy).unwrap_or(false);
    if in_a_sentence {
        value = text::squash(&py_str(fixed.get("sentence").unwrap()));
        // The sentence is sometimes the person's own repair rather than the reading's: the frame
        // comes off before the sentence is said back.
        let unframed = ret_str(crate::memory::ack::without_the_frame(&value));
        if !unframed.is_empty() && unframed != value {
            value = unframed;
            if key == Value::from("plan") {
                let bare = text::strip(&sget(fixed, "value")).to_string();
                // A sentence that was only the frame and the new value says the value on its own.
                if !bare.is_empty() {
                    let said = text::squash(&value).to_lowercase();
                    let b = bare.to_lowercase();
                    if said == b || said == "on ".to_string() + &b || said == "in ".to_string() + &b || said == "at ".to_string() + &b {
                        value = bare;
                    }
                }
            }
        }
        if key == Value::from("plan") {
            // The rest of the plan is said back in the second person, so the note is too.
            value = crate::memory::you(&value);
            value = lower_first(&value);
        }
        // The sentence arrives as the person typed it, full stop and all, and the note adds its own.
        value = text::rstrip_chars(&value, " .!?").to_string();
    }
    let mut shown = text::upper_first(&value);
    let wrong = text::strip(&sget(fixed, "wrong")).to_string();
    if !wrong.is_empty() {
        if key == Value::from("pet") {
            // A name put right is answered warmly and in different words from turn to turn.
            return fmt_named(&pick(&PET_RENAMED_SAY, memory, &sget(fixed, "said")), &[("v", &shown), ("w", &wrong)]);
        }
        // A correction that names both values says both back: "8 kilos, not 5 — got it."
        let shape = str_key(&CORRECTION_SAY_PAIR, Some(&key)).cloned().unwrap_or_else(|| "{v}, not {w} \u{2014} got it.".to_string());
        return fmt_named(&shape, &[("v", &shown), ("w", &wrong)]);
    }
    if fixed.get("dropped").map(Value::truthy).unwrap_or(false) {
        let shape = str_key(&CORRECTION_SAY_DROP, Some(&key)).cloned().unwrap_or_else(|| "Not {v} \u{2014} got it.".to_string());
        return fmt_named(&shape, &[("v", &lower_first(&value))]);
    }
    if in_a_sentence {
        // A plan is said inside a sentence, so the article or the "my" in front of it is not
        // capitalised the way a name in that place would be.
        let (first, _, rest) = text::partition(&shown, " ");
        if PLAN_LOW_START.contains(&first.to_lowercase()) {
            shown = first.to_lowercase() + &(if rest.is_empty() { String::new() } else { " ".to_string() + rest });
        }
    }
    if key.is_none() {
        // No fact was corrected, so there is nothing to keep: the line is said back and dropped.
        return fmt_named(&BARE_ECHO_ONLY, &[("v", &shown)]);
    }
    let shape = str_key(&CORRECTION_SAY, Some(&key)).cloned().unwrap_or_else(|| "{v} \u{2014} got it, I'll go with {v}.".to_string());
    if shape.contains("{a}") {
        let mut animal = sget(fixed, "animal");
        if animal.is_empty() {
            animal = "pet".into();
        }
        return fmt_named(&shape, &[("v", &shown), ("a", &animal)]);
    }
    fmt_named(&shape, &[("v", &shown)])
}

/// Python: bot/memory/store_replies.py:1665 `_job_note(value, known)`
///
/// The corrected job said back in the frame the person used, or "" when there is no frame.
pub fn _job_note(value: &str, known: &Value) -> String {
    let frame = text::strip(&sget(known, "job_frame")).to_string();
    if frame == "work" {
        // The word they used for it was the job itself, so the note says the job.
        return "Got it \u{2014} your job is ".to_string() + &crate::memory::relations::_a(value) + ".";
    }
    let head = frame.split(' ').next().unwrap_or("");
    if !["at", "in", "for", "as"].contains(&head) {
        return String::new();
    }
    let said = _job_said(&dict_of(vec![("job", Value::from(value)), ("job_frame", Value::from(frame.as_str()))]));
    if said.is_empty() {
        return String::new();
    }
    "Got it \u{2014} ".to_string() + &lower_first(&said) + "."
}

rx!(NOT_THE_SAME);

/// Python: bot/memory/store_replies.py:1690 `_the_limit_the_last_line_gave(memory)`
///
/// The limit the line before stated, on the person that line was about, or None.
pub fn _the_limit_the_last_line_gave(memory: &mut Memory) -> Value {
    let said = fold(&last_turn_user(memory));
    if said.is_empty() {
        return Value::None;
    }
    let log = mget(memory, "facts_log").clone();
    for entry in log.items().iter().rev() {
        if !entry.is_dict() {
            continue;
        }
        if fold(&sget(entry, "text")) != said {
            continue;
        }
        let value = entry.get("value").cloned().unwrap_or(Value::None);
        let key = entry.get("key").cloned().unwrap_or(Value::None);
        if key == Value::from("allergies") || key == Value::from("diet") {
            return user(memory);
        }
        if (key == Value::from("other") || key == Value::from("others"))
            && value.is_dict()
            && (value.get("allergies").map(Value::truthy).unwrap_or(false) || value.get("diet").map(Value::truthy).unwrap_or(false))
        {
            return value;
        }
    }
    Value::None
}

static A_WORD_FOR_A_PERSON: LazyLock<&'static Value> = LazyLock::new(|| data::get("bot.memory.facts._A_WORD_FOR_A_PERSON"));

/// Python: bot/memory/store_replies.py:1712 `_not_the_same_reply(memory, low)`
///
/// The limit refused for somebody, acknowledged, and nothing filed.
pub fn _not_the_same_reply(memory: &mut Memory, low: &str) -> Option<String> {
    let flat = text::squash(low);
    let m = NOT_THE_SAME.match_(&flat)?;
    let who = crate::memory::relations::_canon_relation(&crate::memory::facts::a_person_word(m.name_str("who"))).val();
    let who = who.as_str().map(str::to_string)?;
    if !A_WORD_FOR_A_PERSON.contains_str(&who) {
        return None;
    }
    if !_the_limit_the_last_line_gave(memory).truthy() {
        return None;
    }
    let said = "your ".to_string() + &who;
    Some(_vary(
        &[
            "Got it \u{2014} ".to_string() + &said + " isn't.",
            "Good to know \u{2014} ".to_string() + &said + " isn't, then.",
            "Right, so it's not ".to_string() + &said + " \u{2014} noted.",
        ],
        memory,
    ))
}

rx!(OTHER_ASK, OTHER_FACT, WHO_IS_NAME_ASK, THEM, OTHER_NAME_ASK, PRONOUN_NAME_ASK);
strset!(NAME_ASK_TAIL);
sval!(IN_LAW_AFTER_TEXT);

/// Python: bot/memory/store_replies.py:1792 `_with_an_in_law(who, low)`
///
/// The person an ask names with the "in law" it was said with: "brother in law".
pub fn _with_an_in_law(who: &str, low: &str) -> String {
    if who.is_empty() {
        return who.to_string();
    }
    if crate::memory::facts::an_in_law(who) {
        return crate::memory::facts::a_person_word(who); // "brother-in-law" is filed as "brother in law"
    }
    let pattern = format!(r"\b{}s?{}", crate::py::re::escape(who), *IN_LAW_AFTER_TEXT);
    if Re::new(&pattern, crate::py::re::I).is_match(low) {
        return who.to_string() + " in law";
    }
    who.to_string()
}

rx!(KIDS_ASK, KIDS_SOMEONE_ELSE, KIDS_A_DECISION);

/// Python: bot/memory/store_replies.py:1815 `_kids_ask(low)`
///
/// Whether the line asks the person's own count of children.
pub fn _kids_ask(low: &str) -> bool {
    let Some(m) = KIDS_ASK.search(low) else { return false };
    if KIDS_SOMEONE_ELSE.match_(&low[m.end()..]).is_some() {
        return false;
    }
    !KIDS_A_DECISION.is_match(low)
}

rx!(
    ABOUT_A_NAME, CHILDREN_ABOUT, ABOUT_PERSON_MORE, EAT_ASK, FOOD_ASK, DIET_ASK, DIET_HOW_TO, DIET_FACT_ASK, ASK_WORD,
    HELP_ASK, CANT_HAVE, CAN_HAVE, CAN_THEY_HAVE, CAN_THEY_HAVE_A_FOOD, STATED_THEN_ASKED, ASKS_WITH_WH, ASK_START,
);
/// `FOOD_HAS`: a limit -> the foods that hold it.
static FOOD_HAS: LazyLock<Vec<(String, HashSet<String>)>> =
    LazyLock::new(|| g("FOOD_HAS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_set())).collect());

/// Python: bot/memory/store_replies.py:1910 `_the_ask_after_a_statement(low, memory=None)`
///
/// The question clause at the end of a line that states something first, or the line itself.
pub fn _the_ask_after_a_statement(low: &str, memory: &Memory) -> String {
    _the_ask_after_a_statement_m(low, Some(memory))
}

/// `_the_ask_after_a_statement` with Python's `memory=None` expressible.
fn _the_ask_after_a_statement_m(low: &str, memory: Option<&Memory>) -> String {
    let text_ = low.to_string();
    if let Some(read) = memory.and_then(|m| m.understood.as_ref()) {
        // The understanding engine has already cut the line into its clauses, and its focus is
        // the one the person is asking about.
        let clauses = &read.clauses;
        let focus = &read.focus;
        if clauses.len() > 1 && !focus.is_empty() {
            let asked = focus.iter().filter(|w| !text::strip(w).is_empty()).cloned().collect::<Vec<_>>().join(" ");
            let asked = text::strip(&asked).to_string();
            let first = clauses[0].join(" ");
            // Only a fact said before the question is context for it.
            if !asked.is_empty() && asked != text::strip(&text_) && asked != text::strip(&first) {
                return asked;
            }
        }
    }
    let Some(m) = STATED_THEN_ASKED.search(&text_) else { return text_ };
    let head = &text_[..m.start()];
    if ASKS_WITH_WH.is_match(head) {
        return text_.clone();
    }
    let asked = text::strip(&text_[m.end()..]).to_string();
    if asked.is_empty() {
        text_
    } else {
        asked
    }
}

/// Python: bot/memory/store_replies.py:1940 `_asks_about_someone(low, text, memory=None)`
///
/// Whether this line asks about a fact the store keeps for someone else.
pub fn _asks_about_someone(low: &str, text_: &str, memory: &Memory) -> bool {
    _asks_about_someone_m(low, text_, Some(memory))
}

/// `_asks_about_someone` with Python's `memory=None` expressible.
fn _asks_about_someone_m(low: &str, text_: &str, memory: Option<&Memory>) -> bool {
    let whole = low;
    if !_asks_the_line(whole, text_) {
        return false;
    }
    let ask = _the_ask_after_a_statement_m(whole, memory);
    if text::strip(&ask) == text::strip(whole) {
        return true;
    }
    _asks_the_line(&ask, text_)
}

/// Python: bot/memory/store_replies.py:1966 `_asks_the_line(low, text)`
///
/// Whether this line, read whole, asks about a fact the store keeps for someone else.
pub fn _asks_the_line(low: &str, text_: &str) -> bool {
    if OTHER_ASK.is_match(low) {
        return true;
    }
    let ends_with_a_question = text::strip(text_).ends_with('?');
    if _asks_for_a_limit_not_advice(low) && (ASK_WORD.is_match(low) || ASK_START.is_match(low) || ends_with_a_question) {
        // "does she need to avoid anything" names the limit and asks for it.
        return true;
    }
    if DIET_ASK.is_match(low) && !DIET_HOW_TO.is_match(low) {
        return false; // "what's my diet" is about them, whatever is kept for someone else
    }
    if !(EAT_ASK.is_match(low) || CANT_HAVE.is_match(low) || CAN_HAVE.is_match(low) || CAN_THEY_HAVE.is_match(low) || OTHER_FACT.is_match(low)) {
        return false;
    }
    ASK_WORD.is_match(low) || ASK_START.is_match(low) || CAN_THEY_HAVE.is_match(low) || ends_with_a_question
}

rx!(THE_CHAT_ASK, MISSED_CATCH_UP);

/// Python: bot/memory/store_replies.py:2010 `_about_the_bot_here(memory)`
///
/// True when the understanding engine read this line as a question about the bot itself.
pub fn _about_the_bot_here(memory: &Memory) -> bool {
    let Some(reading) = memory.understood.as_ref() else { return false };
    if crate::understanding::kinds::the_manners_question(&reading.norm) {
        // "What are you up to", "watcha doing" and "wuu2": how the bot is filling its time.
        return true;
    }
    if !reading.about_bot {
        return false;
    }
    reading.about_bot_kind.as_deref() != Some("memory")
}

rx!(PLACES_ASK, TOPIC_ASK, ASK_WITHOUT_MY, PLAN_MINE, WHAT_I_SAID, SAID_ASK, MEMORY_POINTER, THING_AGAIN);
sval!(NOT_ON_A_MEDIUM, NOT_THE_PLAN);

/// The `checks` patterns `_about_this_chat` reads (built from the same pieces as Python's tuple).
static CHAT_CHECKS: LazyLock<Vec<Re>> = LazyLock::new(|| {
    let pats: Vec<String> = vec![
        r"\b(?:what|which)\b.*\bmy name\b|\bwho am i\b|\b(?:know|remember) my name\b|\bwhat did (?:i|u|you) (?:say|tell me) my name was\b|^my name$|\bwhat my name (?:is|was)\b".to_string(),
        r"\bwhere do (?:i|we) (?:live|work|come from)\b".to_string(),
        r"\bwhat did i say my job\b|\bwhat(?:'s| is| was) my job\b|\bwhere do (?:i|we) work\b|\bwhat did i say i (?:do|did)\b|\bwhat do i work as\b|\bwhat do i do for (?:a )?living\b".to_string(),
        PLACES_ASK.pattern().to_string() + r"|\bwhere do (?:i|we) work\b",
        concat!(
            r"\bwhat(?:'s| is| was) my (?:dog|cat|pet)(?:'?s)?\b|",
            r"\bwhat did i say my (?:dog|cat|pet)(?:'?s)?\b|",
            r"\bremember my (?:dog|cat|pet)(?:'?s)?\b|\bwhat do i call my (?:dog|cat|pet)(?:'?s)?\b|",
            r"\bwhat did i call my (?:dog|cat|pet)(?:'?s)?\b|",
            r"\bwhat(?:'s| is| was) my (?:dog|cat|pet)(?:'?s)? called\b|",
            r"\bwhat are my (?:pets|dogs|cats) called\b|\bwhat are my (?:pets|dogs|cats)\b|",
            r"\bwhat did i say my (?:pets|dogs|cats) (?:were|are) called\b|",
            r"\bwhat my (?:dog|cat|pet)(?:'?s)?\b|^my (?:dog|cat|pet)(?:'?s)?$"
        )
        .to_string(),
        concat!(
            r"\bwhat (?:am|was) i allergic to\b|\bwhat i (?:am|was) allergic to\b|",
            r"\bwhat did i say i (?:am|was) allergic to\b|",
            r"\bwhat(?:'s| is| was)? my budget\b|\bmy budget again\b|\bwhat did i say my budget was\b|",
            r"^my budget$"
        )
        .to_string(),
        concat!(
            r"\bmy diet\b|\bwhat do i not eat\b|",
            r"\bwhat (?:do|did) i eat\b(?=\s*[?.!,]|\s*$)|",
            r"\bwhat am i (?:eating|not eating)\b"
        )
        .to_string(),
        THE_CHAT_ASK.pattern().to_string(),
        concat!(
            r"\bwhat did i (?:just )?(?:ask|want to know)\b",
            r"(?![^.?!]*\babout\b(?!\s+(?:me|it|that|this|them|us)\b))|",
            r"\bwhat did i (?:just )?say\b(?!\s+(?:about|to|when|in|on|wrong)\b)|"
        )
        .to_string()
            + ACTIVITY_ASK.pattern()
            + "|"
            + PLAN_MINE.pattern(),
        concat!(
            r"\bwhat have i told you\b|\bwhat do you (?:know|remember) about me\b|",
            r"\bwhat did i (?:tell|say)(?: you)? about me\b|\bwhat do you know about me\b|",
            r"^what i (?:told|said)(?: you)?$|^what have i (?:told|said)(?: you)?$"
        )
        .to_string(),
        r"\bremember what we talked about\b|\bdo you remember (?:what|our|yesterday|me)\b".to_string(),
        WHAT_I_SAID.pattern().to_string()
            + concat!(
                r"|\bwhat was it i said\b|",
                r"\bwhat (?:was|were) (?:i|we) (?:telling you|mentioning|on about)\b|",
                r"\bwhat have i been (?:saying|telling you)\b|",
                r"\b(?:what|anything) (?:i|we) (?:said|told you|mentioned|asked)",
                r"|\bwhere (?:we|it|this|the (?:chat|conversation)) (?:all )?(?:started|began)\b",
                r"|\bwhich (?:question|thing) (?:did i|i) (?:ask|asked|say|said|start with|started with)\b",
                r"(?=\s*(?:\?|$|again\b|please\b|about\b|i\b|my\b|the\b|that\b|",
                r"to (?:me|you)\b|before\b|earlier\b))|",
                r"\b(?:do|can|could|did) (?:you|u) remember what i\b"
            ),
        concat!(
            r"^what i (?:do|did)$|\b(?:remind me|tell me|can you (?:remind me|tell me)) what i (?:do|did)\b|",
            r"\bwhat did i say my age\b|\bhow old am i\b|\bwhat(?:'s| is| was)? my age\b|",
            r"\bmy age again\b|\bhow old did i say\b|",
            r"\bhow old i (?:am|was)\b"
        )
        .to_string(),
        concat!(
            r"\bwhat (?:am i|are we) (?:planning|up to)\b|\bwhat (?:am i|are we) doing\b(?!\s+wrong)\b|",
            r"\bwhat(?:'s| is|s)?\s+(?:coming up|on for (?:me|us))\b|",
            r"\bwhat (?:have|has) (?:i|we) got (?:coming up|on|planned)?\b|",
            r"\bwhat do (?:i|we) have (?:coming up|on|planned)\b|",
            r"\banything (?:planned|coming up)\b|\banything on\b"
        )
        .to_string()
            + *NOT_ON_A_MEDIUM
            + concat!(
                r"|\bwhat am i looking forward to\b|",
                r"\bwhat (?:was|were) (?:i|we) (?:planning|saving|saving up)\b|",
                r"\bwhat (?:am i|are we|is) (?:saving|saving up)\b|",
                r"\bwhat i (?:was|am) (?:planning|saving)\b|",
                r"\bwhat(?:'s| is| was) my plan\b"
            )
            + *NOT_THE_PLAN
            + r"|"
            + r"\bwhat was (?:the|my) plan\b"
            + *NOT_THE_PLAN
            + concat!(
                r"|",
                r"\bwhat was i going to do\b(?:\s*then)?\s*[?.!,]*$|",
                r"\bwhere (?:are|were|was|am) (?:we|i) (?:going|off|headed|heading|away)\b|",
                r"\bwhen (?:are|were|was|am) (?:we|i) (?:off|going|away|moving|flying|travelling)\b|",
                r"\b(?:which|what) (?:month|day|week|date) (?:are|am|is) (?:we|i)\b|",
                r"\bremind me where\b"
            ),
    ];
    pats.iter().map(|p| Re::new(p, 0)).collect()
});
static MY_FACT_ALONE: LazyLock<Re> = LazyLock::new(|| Re::new(r"^my (?:job|work|name|budget)$", 0));
static WHAT_DO_I_DO: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bwhat do i do\b", 0));
static SPENT_PRESENT_ASK: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.spent.PRESENT_ASK"));

/// Python: bot/memory/store_replies.py:2033 `_about_this_chat(text, memory=None)`
///
/// Lines that ask about the conversation itself, not about the world.
pub fn _about_this_chat(text_: &str, memory: &mut Memory) -> bool {
    _about_this_chat_m(text_, Some(memory))
}

/// `_about_this_chat` with Python's `memory=None` expressible (most of the readers below ask it of
/// a line alone). Where Python reads the missing memory as "no turns", an empty memory stands in.
pub fn _about_this_chat_m(text_: &str, mut memory: Option<&mut Memory>) -> bool {
    let empty = Memory::new();
    if let Some(m) = memory.as_deref() {
        if _about_the_bot_here(m) {
            return false;
        }
    }
    if crate::memory::history::asked(text_, None).val().truthy() {
        return true;
    }
    // A recall ask with nothing named in it is the whole picture asked for in other words.
    if crate::memory::recall::ask(text_, false).as_deref() == Some("me") {
        return true;
    }
    // "what was wrong with my boiler again" is a reminder about the boiler, and the store owns it.
    if crate::memory::reminded::about_a_thing_they_said(text_, memory.as_deref().unwrap_or(&empty)) {
        return true;
    }
    let folded = crate::memory::relations::_folded(text_, memory.as_deref().unwrap_or(&empty));
    let low = MORE_OF_IT.sub(r"\1", &_bare(&folded));
    // "how many cats", "how many have i got": a count of their own animals is a fact they told
    // the store, not a question about the world.
    if _a_count_ask(&low).is_some() {
        return true;
    }
    // The length of a pregnancy the chat was told is the same kind of ask.
    if let Some(m) = memory.as_deref_mut() {
        if _a_length_i_have(m, &low) {
            return true;
        }
    }
    if ASKED_ALL.is_match(&low) {
        return true;
    }
    if CHAT_CHECKS.iter().any(|p| p.is_match(&low)) {
        return true;
    }
    if TOPIC_ASK.is_match(&low) || ASK_WITHOUT_MY.is_match(&low) {
        return true;
    }
    // "summarise our chat", "what did we talk about": a recap ask is about the conversation.
    if _summary_ask(&low) {
        return true;
    }
    // "what's the noise again?" names the thing instead of asking "what did I say about it".
    let mr: &Memory = memory.as_deref().unwrap_or(&empty);
    if let Some(named) = THING_AGAIN.match_(&low) {
        if _said_word(named.str(1), mr) {
            return true;
        }
    }
    // "and which knee was it": a detail they gave, asked for by the thing it is about.
    if _which_thing_was_it(&low, mr).is_some() {
        return true;
    }
    if _an_attribute_they_gave(&low) {
        return true;
    }
    // "what subject did i say i was revising": only when a line answers it does the gate hold.
    if !_an_open_attribute_ask(&low, mr).is_empty() {
        return true;
    }
    // "and my job?" after "what's my name?" is one more fact recalled, not a new subject.
    if MY_FACT_ALONE.is_match(&low) && _recall_in_progress_m(memory.as_deref()) {
        return true;
    }
    // Reads of the chat itself keep the words that ask for them: "say that again" needs its
    // "again", so these are matched before the reminder wrapper comes off.
    if FIRST_ASK.is_match(&low) || SAID_ASK.is_match(&low) || _repeat_ask_m(text_, &folded, None) {
        return true;
    }
    // A money ask in the present tense names a thing the chat may already hold a bill for.
    if let Some(m) = memory.as_deref() {
        if SPENT_PRESENT_ASK.is_match(text_) && crate::memory::spent::answer(text_, m).val().truthy() {
            return true;
        }
    }
    // "what do I do?" is only about their job when the line points back at what they told us or
    // when the recall is already going.
    if !WHAT_DO_I_DO.is_match(&low) || text::split(&low).len() > 7 {
        return false;
    }
    if MEMORY_POINTER.is_match(&folded) || _recall_in_progress_m(memory.as_deref()) {
        return true;
    }
    // "i work in a warehouse" then "what do i do" is the job they just told us.
    match memory {
        Some(m) => _job_just_said(m, &low),
        // `user(None)` in Python: the read raises.
        None => panic!("AttributeError: 'NoneType' object has no attribute 'get'"),
    }
}

strset!(ASK_TAIL_FILLER);
rx!(ASK_ABOUT_JOB);
static WHAT_DO_I_DO_TAIL: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bwhat do i do\b(.*)$", 0));

/// Python: bot/memory/store_replies.py:2223 `_ask_about_the_job(ask)`
///
/// Whether a "what do i do" ask has nothing after it, or only words about the work itself.
pub fn _ask_about_the_job(ask: &str) -> bool {
    let flat = fold(ask);
    let Some(found) = WHAT_DO_I_DO_TAIL.search(&flat) else { return true };
    let tail = LETTER_WORDS.findall_str(found.str(1)).into_iter().filter(|w| !ASK_TAIL_FILLER.contains(w)).collect::<Vec<_>>().join(" ");
    tail.is_empty() || ASK_ABOUT_JOB.match_(&tail).is_some()
}

/// Python: bot/memory/store_replies.py:2234 `_job_just_said(memory, ask="")`
///
/// True when the last line the person wrote is the one that gave us their job.
pub fn _job_just_said(memory: &mut Memory, ask: &str) -> bool {
    let me = user(memory);
    let job = me.get("job").cloned().unwrap_or(Value::None);
    let line = last_turn_user(memory);
    if !job.truthy() || line.is_empty() {
        return false;
    }
    let job = py_str(&job);
    // The job can be a phrase or a name, so any of its words will do; the name is checked first.
    let said = fold(&job);
    // Where they work is the job's too.
    let place = fold(&sget(&me, "workplace"));
    if !(fold(&line).contains(&said) || _job_words_seen(&job, &line)) {
        if !(!place.is_empty() && _the_place_said(&place, &line)) {
            // A line about the work itself keeps the conversation on it.
            return _a_work_detail(&line) && _ask_about_the_job(ask);
        }
    }
    if !_only_the_job(&line, &job, &place) {
        return false;
    }
    // The ask has to be about the job too: "what do i do about my back" is a back question.
    _ask_about_the_job(ask)
}

rx!(SELF_DO_AGAIN);

/// Python: bot/memory/store_replies.py:2266 `_doing_again(text, memory)`
///
/// True when "what do i do again" is about the task on the table, not the person's job.
pub fn _doing_again(text_: &str, memory: &mut Memory) -> bool {
    let folded = crate::memory::relations::_folded(text_, memory);
    if !SELF_DO_AGAIN.is_match(&folded) {
        return false;
    }
    if _recall_in_progress(memory) {
        return false;
    }
    // The line the person just wrote may be the one that gave the job.
    let job = user(memory).get("job").cloned().unwrap_or(Value::None);
    let line = last_turn_user(memory);
    if job.truthy() {
        let job = py_str(&job);
        if line.to_lowercase().contains(&job.to_lowercase()) || _job_words_seen(&job, &line) {
            return false;
        }
    }
    !_job_just_said(memory, "what do i do again")
}

strset!(WORK_WORDS, NOT_A_DETAIL, JOB_ONLY, PLACE_LITTLE);

/// `{word.strip(",.!?;:'\"()") for word in text.split()}` without the empty word.
fn stripped_words(text_: &str) -> HashSet<String> {
    text::split(text_).iter().map(|w| text::strip_chars(w, ",.!?;:'\"()").to_string()).filter(|w| !w.is_empty()).collect()
}

/// Python: bot/memory/store_replies.py:2306 `_a_work_detail(line)`
///
/// Whether the line is a detail of the work alone, with nothing else that wants an answer.
pub fn _a_work_detail(line: &str) -> bool {
    let low = fold(line);
    if !text::starts_with_any(&low, &["i ", "im ", "i'm ", "we ", "we're "]) {
        return false;
    }
    let words = stripped_words(&low);
    if words.len() > 10 || low.contains('?') {
        return false;
    }
    if words.iter().any(|w| NOT_A_DETAIL.contains(w)) {
        return false;
    }
    if !words.iter().any(|w| WORK_WORDS.contains(w)) {
        return false;
    }
    // Every other word says something about the work or nothing at all.
    words.iter().all(|w| WORK_WORDS.contains(w) || JOB_ONLY.contains(w) || crate::understanding::pyuni::is_digit(w))
}

/// Python: bot/memory/store_replies.py:2328 `_job_words_seen(job, line)`
///
/// Whether the line uses one of the job's words, however it ends the word.
pub fn _job_words_seen(job: &str, line: &str) -> bool {
    use crate::memory::topics::{content, forms};
    let mut mine: HashSet<String> = content(job).into_iter().collect();
    for word in text::split(job) {
        mine.extend(forms(&word));
    }
    let seen: HashSet<String> = content(line).into_iter().collect();
    seen.iter().any(|w| mine.contains(w)) || seen.iter().any(|w| forms(w).iter().any(|f| mine.contains(f)))
}

/// Python: bot/memory/store_replies.py:2352 `_the_place_said(place, line)`
///
/// Whether this line is the one that told us where they work.
pub fn _the_place_said(place: &str, line: &str) -> bool {
    use crate::memory::topics::content;
    let mut words: HashSet<String> = content(place).into_iter().collect();
    if words.is_empty() {
        // Every content word of the place is short enough that `content` drops it, so the
        // written words have to do, less the little ones.
        words = text::split(&place.to_lowercase()).iter().map(|w| text::strip_chars(w, ",.!?;:'\"()").to_string()).collect();
        words.retain(|w| !PLACE_LITTLE.contains(w));
    }
    if words.is_empty() {
        return false;
    }
    let said: HashSet<String> = text::split(&line.to_lowercase()).iter().map(|w| text::strip_chars(w, ",.!?;:'\"()").to_string()).collect();
    let content_line: HashSet<String> = content(line).into_iter().collect();
    words.iter().all(|w| content_line.contains(w) || said.contains(w))
}

static TAKE_BACK_SPLIT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.facts.TAKE_BACK_SPLIT"));

/// Python: bot/memory/store_replies.py:2373 `_only_the_job(line, job, place="")`
///
/// Whether the line is the job and nothing else.
pub fn _only_the_job(line: &str, job: &str, place: &str) -> bool {
    use crate::memory::topics::{content, forms};
    // A job said with the one it replaces in the same breath is still a line about the job, so
    // the part that takes the old one back is cut off first.
    let line = match TAKE_BACK_SPLIT.search(line) {
        Some(m) => &line[..m.start()],
        None => line,
    };
    let left: HashSet<String> = text::split(&fold(line)).iter().map(|w| text::strip_chars(w, ",.!?;:'\"()").to_string()).filter(|w| !w.is_empty()).collect();
    let mut mine: HashSet<String> = HashSet::new();
    for word in text::split(&job.to_lowercase()) {
        mine.extend(forms(&word).iter().map(|f| f.to_lowercase()));
        mine.insert(word);
    }
    mine.extend(content(job).iter().map(|w| w.to_lowercase()));
    for word in text::split(&place.to_lowercase()) {
        mine.extend(forms(&word).iter().map(|f| f.to_lowercase()));
        mine.insert(word);
    }
    mine.extend(content(place).iter().map(|w| w.to_lowercase()));
    left.iter().all(|w| JOB_ONLY.contains(w) || mine.contains(w))
}

rx!(TALKED_ABOUT, SUMMARY_WORD, CHAT_WORD, SUMMARY_ASK_FILLER);

/// Python: bot/memory/store_replies.py:2414 `_summary_ask(low)`
///
/// True for a line that asks for a summary or a recap of this conversation.
pub fn _summary_ask(low: &str) -> bool {
    if TALKED_ABOUT.is_match(low) {
        return true;
    }
    if !SUMMARY_WORD.is_match(low) {
        return false;
    }
    if CHAT_WORD.is_match(low) {
        return true;
    }
    text::split(&SUMMARY_ASK_FILLER.sub(" ", &SUMMARY_WORD.sub(" ", low))).is_empty()
}

rx!(WHICH_WAS, ATTRIBUTE_SAID, ATTRIBUTE_KIND, A_WHEN, A_CLOCK_ASK);
strset!(NOT_A_THING_NAMED, NOT_A_KIND);
static SAID_VERB_TAIL: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:say|said|tell you|told you|mention|mentioned)\b(.*)$", 0));

/// Python: bot/memory/store_replies.py:2448 `_an_attribute_they_gave(low)`
///
/// True for "what day did i say it was due": the ask with the attribute in front of it.
pub fn _an_attribute_they_gave(low: &str) -> bool {
    if ATTRIBUTE_SAID.match_(low).is_none() {
        return false;
    }
    match SAID_VERB_TAIL.search(low) {
        Some(after) => !crate::memory::topics::content(after.str(1)).is_empty(),
        None => false,
    }
}

/// Python: bot/memory/store_replies.py:2471 `_an_open_attribute_ask(low, memory)`
///
/// The line that answers "what subject did i say i was revising", or "" when none does.
pub fn _an_open_attribute_ask(low: &str, memory: &Memory) -> String {
    use crate::memory::topics::content;
    let Some(part) = ATTRIBUTE_KIND.match_(low) else { return String::new() };
    let kind = text::squash(part.str(1));
    let kind_words = text::split(&kind);
    if kind.is_empty() || NOT_A_KIND.contains(&kind_words[0]) || kind_words.len() > 3 {
        return String::new();
    }
    let after: Vec<String> = match SAID_VERB_TAIL.search(low) {
        Some(m) => content(m.str(1)),
        None => Vec::new(),
    };
    let base = if after.is_empty() { content(&kind) } else { after };
    let words: HashSet<String> = base.into_iter().filter(|w| text::len(w) > 2).collect();
    if words.is_empty() {
        return String::new();
    }
    let (mut best, mut score) = (String::new(), 0usize);
    for turn in &memory.turns {
        let line = &turn.user;
        if line.is_empty() || crate::memory::corrected::_asks(line) {
            continue;
        }
        let seen = _seen(line);
        let found = words.iter().filter(|w| _alike(w, &seen)).count();
        // The later line wins a tie: it is the one with the name in it, which is what was asked.
        if found > 0 && found >= score {
            best = line.clone();
            score = found;
        }
    }
    if best.is_empty() {
        String::new()
    } else {
        _recall_quote(&best, Some(memory), false)
    }
}

/// Python: bot/memory/store_replies.py:2502 `_which_thing_was_it(low, memory)`
///
/// The thing in "and which knee was it", when they named it themselves earlier in the chat.
pub fn _which_thing_was_it(low: &str, memory: &Memory) -> Option<String> {
    let part = WHICH_WAS.match_(low)?;
    let thing = text::squash(part.str(1));
    if NOT_A_THING_NAMED.contains(&thing) || crate::memory::topics::content(&thing).is_empty() {
        return None;
    }
    if !_said_word(&thing, memory) {
        return None;
    }
    Some(thing)
}

/// Python: bot/memory/store_replies.py:2535 `_without_the_when(line)`
///
/// The line with the phrases that say when taken out, so "all the time" is not "the time".
pub fn _without_the_when(line: &str) -> String {
    A_WHEN.sub(" ", line)
}

/// Python: bot/memory/store_replies.py:2553 `_a_clock_ask(line)`
///
/// True for a line that asks what the clock says.
pub fn _a_clock_ask(line: &str) -> bool {
    A_CLOCK_ASK.match_(&text::strip(line).to_lowercase()).is_some()
}

/// `recent(memory, n)`: the last n turns, oldest first (`turns[-n:]`, the whole list for 0).
fn recent_turns(memory: &Memory, n: usize) -> &[Turn] {
    let turns = &memory.turns;
    if n == 0 {
        return turns;
    }
    &turns[turns.len().saturating_sub(n)..]
}

/// `last_module(memory)`.
fn last_module(memory: &Memory) -> Option<String> {
    memory.turns.last().and_then(|t| t.module.clone())
}

/// Python: bot/memory/store_replies.py:2558 `_said_word(word, memory)`
///
/// Whether the person named this word, or one of its forms, in one of their last lines.
pub fn _said_word(word: &str, memory: &Memory) -> bool {
    use crate::memory::topics::{content, forms};
    let mine = forms(word);
    for turn in recent_turns(memory, 8) {
        let said = &turn.user;
        if text::strip(said).is_empty() || crate::memory::corrected::_asks(said) {
            continue;
        }
        let line = _without_the_when(said);
        if text::strip(&line).is_empty() {
            continue;
        }
        if content(&line).iter().any(|seen| forms(seen).iter().any(|f| mine.contains(f))) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/store_replies.py:2578 `_recall_in_progress(memory)`
///
/// True when the last turn was the bot answering a question about the conversation.
pub fn _recall_in_progress(memory: &Memory) -> bool {
    _recall_in_progress_m(Some(memory))
}

/// `_recall_in_progress` with Python's `memory=None` (False).
fn _recall_in_progress_m(memory: Option<&Memory>) -> bool {
    let Some(memory) = memory else { return false };
    let before = fold(&last_turn_user(memory));
    // A question about how the chat went tells us nothing about the person's job.
    if !before.is_empty() && (FIRST_ASK.is_match(&before) || SAID_ASK.is_match(&before) || _repeat_ask_m(&before, &before, None)) {
        return false;
    }
    if last_module(memory).as_deref() == Some("memory_store") {
        return true;
    }
    !before.is_empty() && _about_this_chat_m(&before, None)
}

rx!(ELIDED_ASK, ELIDED_AGAIN);
strset!(ELIDED_WORDS);
/// `ELIDED_NAMES`: (slot, the words that name it).
static ELIDED_NAMES: LazyLock<Vec<(String, &'static Re)>> =
    LazyLock::new(|| g("ELIDED_NAMES").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].as_re().unwrap())).collect());

/// Python `int(v)` for a value: None where Python raises.
fn py_int_value(v: &Value) -> Option<i64> {
    match v {
        Value::Int(i) => Some(*i),
        Value::Bool(b) => Some(*b as i64),
        Value::Float(f) if f.is_finite() => Some(f.trunc() as i64),
        Value::Str(s) => py_int(s),
        _ => None,
    }
}

/// Python: bot/memory/store_replies.py:2590 `_elided_ask(text, memory, low="")`
///
/// (which fact an ask that leaves the fact out is about, whether that was a guess).
pub fn _elided_ask(text_: &str, memory: &Memory, low: &str) -> (String, bool) {
    let source = if low.is_empty() { text_ } else { low };
    let line = text::strip_chars(&fold(source), " .,!?;:-").to_string();
    if line.is_empty() || text::split(&line).iter().any(|w| !ELIDED_WORDS.contains(w)) {
        return (String::new(), false);
    }
    let raw = fold(text_);
    // The raw line is tried as well as the folded one - it is where "again" still is.
    if !(ELIDED_ASK.is_match(&line) || ELIDED_ASK.is_match(&raw) || (ELIDED_AGAIN.is_match(&line) && raw.contains("again"))) {
        return (String::new(), false);
    }
    let turns = &memory.turns;
    let named = _named_slots(&line);
    // The fact they just told me wins; "call" fits a pet as well as a name.
    let fits: HashSet<String> = if named.contains("name") || named.contains("pets") {
        let mut f = named.clone();
        f.insert("name".into());
        f.insert("pets".into());
        f
    } else {
        named.clone()
    };
    let stated = _stated_slot(memory, turns, 1);
    if !stated.is_empty() && (named.is_empty() || fits.contains(&stated)) {
        return (stated, true);
    }
    // One line back: a question in between still leaves them asking about the fact they told me.
    let stated = _stated_slot(memory, turns, 2);
    if !stated.is_empty() && (named.is_empty() || fits.contains(&stated)) {
        return (stated, true);
    }
    let asked = mget(memory, "asked");
    if asked.is_dict() && asked.get("slot").map(Value::truthy).unwrap_or(false) {
        let turn = asked.get("turn").filter(|t| t.truthy()).cloned().unwrap_or(Value::Int(0));
        let asked_turn = py_int_value(&turn);
        let slot = py_str(asked.get("slot").unwrap());
        if let Some(asked_turn) = asked_turn {
            if turns.len() as i64 - asked_turn <= 2 && (named.is_empty() || fits.contains(&slot)) {
                return (slot, false);
            }
        }
    }
    (String::new(), false)
}

/// Python: bot/memory/store_replies.py:2638 `_named_slots(line)`
///
/// The facts an elided ask names in its own words (empty when it names none).
pub fn _named_slots(line: &str) -> HashSet<String> {
    ELIDED_NAMES.iter().filter(|(_, p)| p.is_match(line)).map(|(s, _)| s.clone()).collect()
}

/// Python: bot/memory/store_replies.py:2642 `_stated_slot(memory, turns, back=1)`
///
/// The one fact the last thing the person said gave ("" when it gave none or several).
pub fn _stated_slot(memory: &Memory, turns: &[Turn], back: i64) -> String {
    let take = back.max(1) as usize;
    let window: Vec<&Turn> = turns[turns.len().saturating_sub(take)..].iter().rev().collect();
    for (offset, turn) in window.iter().enumerate() {
        let said = &turn.user;
        if text::strip(said).is_empty() {
            continue;
        }
        // One line back is for a question in between, never for any line.
        if offset == 0 && back > 1 && !crate::memory::corrected::_asks(said) {
            return String::new();
        }
        let found = crate::memory::facts::extract(said, false).val();
        // `job` is left out on purpose: the job frame cannot yet say "work in a warehouse".
        let kinds: Vec<&str> = [("name", "name"), ("city", "city"), ("age", "age"), ("budget", "budget"), ("allergies", "allergy"), ("pets", "pets")]
            .iter()
            .filter(|(key, _)| found.get(key).map(Value::truthy).unwrap_or(false))
            .map(|(_, slot)| *slot)
            .collect();
        if kinds.len() > 1 {
            return String::new();
        }
        if let Some(k) = kinds.first() {
            return k.to_string();
        }
    }
    String::new()
}

/// Python: bot/memory/store_replies.py:2677 `_remember_asked(memory, slot)`
///
/// Keep the fact this answer was about, so the next elided ask is about the same one.
pub fn _remember_asked(memory: &mut Memory, slot: &str) {
    if slot.is_empty() {
        return;
    }
    let turn = memory.turns.len() as i64;
    memory.set("asked", dict_of(vec![("slot", Value::from(slot)), ("turn", Value::Int(turn))]));
}

/// Python: bot/memory/store_replies.py:9845 `_repeat_ask(text, low, memory=None)`
///
/// True when the line asks to hear the bot's last reply again, not about its content.
pub fn _repeat_ask(text_: &str, low: &str, memory: &Memory) -> bool {
    _repeat_ask_m(text_, low, Some(memory))
}

/// `_repeat_ask` with Python's `memory=None` expressible (a whole-answer ask is then a repeat).
pub fn _repeat_ask_m(text_: &str, low: &str, memory: Option<&Memory>) -> bool {
    if WHOLE_ASK.match_(text::strip(low)).is_some() {
        // A whole-answer ask is the store's only when there is a whole answer behind it.
        return match memory {
            None => true,
            Some(m) => _a_whole_answer_to_hand_back(m, text_),
        };
    }
    if !REPEAT_ASK.is_match(low) || REPEAT_START.match_(low).is_none() {
        return false;
    }
    // The words are read off the folded line, not the raw one.
    crate::memory::topics::keywords(&REPEAT_TAIL.sub(" ", low)).iter().all(|w| REPEAT_WORDS.contains(w))
}

strs!(ADVICE_MODULES);

/// A reader's value as a reply: `Some` only when it is a non-empty str (`if x: return x`).
fn some_str(v: impl PyRet) -> Option<String> {
    match v.val() {
        Value::Str(s) if !s.is_empty() => Some(s.to_string()),
        Value::None => None,
        v if v.truthy() => Some(py_str(&v)),
        _ => None,
    }
}

/// `x.strip(chars)` then `x[:1].upper() + x[1:]`-style helpers read better spelled out; this is
/// Python's `str.strip(chars)` returning an owned string.
fn strip_owned(s: &str, chars: &str) -> String {
    text::strip_chars(s, chars).to_string()
}

/// Python: bot/memory/store_replies.py:2683 `_the_case_they_just_gave(text, memory)`
///
/// The consequence of the last answer the short line just chose, said as the reply.
pub fn _the_case_they_just_gave(text_: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(text_);
    if clean.is_empty() || clean.contains('?') {
        return None;
    }
    if crate::memory::corrected::_asks(&clean) {
        // A line that opens with an ask word is a question without the question mark.
        return None;
    }
    match last_module(memory) {
        Some(m) if ADVICE_MODULES.contains(&m) => {}
        _ => return None,
    }
    let said = text::strip(&last_turn_bot(memory)).to_string();
    if said.is_empty() {
        return None;
    }
    let consequence = ret_str(crate::memory::condition::chosen(&clean, &said));
    if consequence.is_empty() {
        if !crate::memory::condition::refused(&clean, &said) {
            return None;
        }
        // The answer tested the number this line gives and their number fails the test: hand the
        // fact back and stop.
        let gave = strip_owned(&ret_str(crate::memory::ack::said_back(&clean, memory)), " .");
        return if gave.is_empty() { None } else { Some(text::upper_first(&gave) + ".") };
    }
    let case = ret_str(crate::memory::ack::said_back(&clean, memory));
    if case.is_empty() {
        return None;
    }
    let body = strip_owned(&consequence, " ,;:-\u{2014}.");
    if body.is_empty() {
        return None;
    }
    let body = text::upper_first(&body);
    // Their case first, then the choice the answer offered for it.
    let line = format!("{}. {}, then.", text::capitalize(text::strip_chars(&case, " .")), body);
    if crate::memory::relations::_same_as_last(&line, memory) {
        return None;
    }
    Some(line)
}

rx!(COUNT_IN_AN_ANSWER);
static SAID_ANOTHER_SLOT: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:Your|My) (name|job|budget) is (.+)$", 0));
static SAID_ANOTHER_YOURE: LazyLock<Re> = LazyLock::new(|| Re::new(r"^You told me you(?:'re| are) (.+)$", 0));
static SAID_ANOTHER_GOT: LazyLock<Re> = LazyLock::new(|| Re::new(r"^You've got (.+)$", 0));
static SAID_ANOTHER_CALLED: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bcalled (.+)$", 0));
static SAID_ANOTHER_LIVE: LazyLock<Re> = LazyLock::new(|| Re::new(r"^You (?:live|'re|are)(?: in| from)? (.+)$", 0));

/// Python: bot/memory/store_replies.py:2733 `_said_another_way(reply)`
///
/// The same fact in a second wording, or None when there is no plain one.
pub fn _said_another_way(reply: &str) -> Option<String> {
    let text_ = text::strip(reply);
    if let Some(m) = SAID_ANOTHER_SLOT.match_(text_) {
        let slot = m.str(1);
        let value = text::rstrip_chars(m.str(2), ".");
        match slot {
            "budget" => return Some("You said ".to_string() + value + "."),
            "name" => return Some("I have you as ".to_string() + value + "."),
            "job" => return Some("You told me it is ".to_string() + value + "."),
            _ => {}
        }
    }
    if let Some(m) = SAID_ANOTHER_YOURE.match_(text_) {
        return Some("You said you're ".to_string() + text::rstrip_chars(m.str(1), ".") + ".");
    }
    if let Some(m) = SAID_ANOTHER_GOT.match_(text_) {
        let animals = text::rstrip_chars(m.str(1), ".");
        if animals.matches("called").count() > 1 {
            return None; // more than one name: no plain short way to say it
        }
        if let Some(named) = SAID_ANOTHER_CALLED.search(animals) {
            // "what's my dog called?" then "and what did I call him?" is the same name twice.
            return Some("Still called ".to_string() + named.str(1) + ".");
        }
        if !COUNT_IN_AN_ANSWER.is_match(animals) {
            // One animal has no plain second wording, so the plain one stands.
            return None;
        }
        return Some("Still ".to_string() + animals + ".");
    }
    if let Some(m) = SAID_ANOTHER_LIVE.match_(text_) {
        return Some("You said ".to_string() + text::rstrip_chars(m.str(1), ".") + ".");
    }
    None
}

/// Python: bot/memory/store_replies.py:2767 `_slot_words(slot, memory=None)`
///
/// The plain ask for a fact slot, so one set of patterns answers every ask.
pub fn _slot_words(slot: &str, memory: &mut Memory) -> String {
    if slot == "pets" {
        let pets = dicts(user(memory).get("pets").unwrap_or(&Value::None));
        let animals: Vec<String> = pets.iter().map(|p| sget(p, "animal")).collect();
        if let Some(last) = animals.last() {
            if animals.iter().filter(|a| *a == last).count() == 1 {
                return "what is my ".to_string() + last + " called";
            }
        }
    }
    match slot {
        "name" => "what is my name",
        "age" => "how old am i",
        "job" => "what is my job",
        "city" => "which city do i live in",
        "budget" => "what is my budget",
        "allergy" => "what am i allergic to",
        "pets" => "what are my pets called",
        _ => "what have i told you",
    }
    .to_string()
}

static SLOT_OF: LazyLock<Vec<(&'static str, Re)>> = LazyLock::new(|| {
    [
        ("allergy", r"\ballerg"),
        ("car", r"\bwhat (?:do|did) i drive\b|\bcar\b"),
        ("budget", r"\bbudget"),
        ("pets", r"\b(?:dog|cat|pet)s?\b"),
        ("job", r"\bmy (?:job|work)\b|\bwhat do i do\b|\bwhere do i work\b"),
        ("city", r"\b(?:city|place|country)\b|\bwhere do i (?:live|come from)\b"),
        ("name", r"\bmy name\b|\bwho am i\b"),
    ]
    .into_iter()
    .map(|(s, p)| (s, Re::new(p, 0)))
    .collect()
});

/// Python: bot/memory/store_replies.py:2783 `_slot_of(low)`
///
/// The fact a recall reply was about, from the plain ask that was answered.
pub fn _slot_of(low: &str) -> String {
    SLOT_OF.iter().find(|(_, p)| p.is_match(low)).map(|(s, _)| s.to_string()).unwrap_or_default()
}

rx!(BARE_MINE);
static SLOT_NOUNS: LazyLock<HashMap<String, HashSet<String>>> =
    LazyLock::new(|| g("SLOT_NOUNS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_set())).collect());
strset!(SLOT_FILLER);
static BARE_SLOTS: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    g("BARE_SLOTS").items().iter().map(|p| (p.items()[0].str().to_string(), Re::new(p.items()[1].str(), 0))).collect()
});
static SLOT_KEY: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| g("SLOT_KEY").str_list_map());

/// Python: bot/memory/store_replies.py:2814 `_slot_named_in(low)`
///
/// The fact a line that only names it asks for, from the fact's own words.
pub fn _slot_named_in(low: &str) -> String {
    BARE_SLOTS.iter().find(|(_, p)| p.is_match(low)).map(|(s, _)| s.clone()).unwrap_or_default()
}

/// Python: bot/memory/store_replies.py:2821 `_slot_held(slot, memory)`
///
/// Whether the store has a fact of theirs for this slot.
pub fn _slot_held(slot: &str, memory: &mut Memory) -> bool {
    let Some(keys) = SLOT_KEY.get(slot).filter(|k| !k.is_empty()) else { return false };
    let known = user(memory);
    keys.iter().any(|k| known.get(k).map(Value::truthy).unwrap_or(false))
}

/// Python: bot/memory/store_replies.py:2829 `_bare_ask_naming_a_fact(low, memory)`
///
/// The slot a line that only names one of their facts is asking for, or "".
pub fn _bare_ask_naming_a_fact(low: &str, memory: &mut Memory) -> String {
    use crate::memory::topics::forms;
    let stripped = text::strip_chars(low, " .!?;,");
    let Some(found) = BARE_MINE.match_(stripped) else { return String::new() };
    let words: HashSet<String> = text::split(found.str(1)).into_iter().collect();
    let slot = _slot_named_in(&("my ".to_string() + found.str(1)));
    if slot.is_empty() || !_slot_held(&slot, memory) {
        return String::new();
    }
    let empty = HashSet::new();
    let nouns = SLOT_NOUNS.get(&slot).unwrap_or(&empty);
    if words.iter().any(|w| !nouns.contains(w) && !SLOT_FILLER.contains(w)) {
        return String::new(); // the line says something about the fact, so it is not only naming it
    }
    if slot == "pets" {
        // "my cat?" is about a cat, so a store that only holds a dog has nothing to say to it.
        let named: Vec<&String> =
            words.iter().filter(|w| !SLOT_FILLER.contains(*w) && !["pet", "pets", "animal", "animals"].contains(&w.as_str())).collect();
        let mut held: HashSet<String> = HashSet::new();
        for pet in user(memory).get("pets").map(|p| p.items().to_vec()).unwrap_or_default() {
            if pet.is_dict() {
                held.extend(forms(&sget(&pet, "animal")));
            }
        }
        if !named.is_empty() && !named.iter().any(|one| forms(one).iter().any(|f| held.contains(f))) {
            return String::new();
        }
    }
    slot
}

rx!(SELF_WORD, PLACE_ADVICE, PLACE_RECALLED, GROWN_UP_ASK, FEELING_ALONE, COMING_UP, A_DAY, TREATMENT, A_PERSON_IN_TROUBLE, WHO_IS_IN_TROUBLE);
strset!(SHE, HE);

/// Python `try: ... except Exception: fallback` around a call into another file: a panic there
/// (Python's exception) gives the fallback.
fn or_on_raise<T>(f: impl FnOnce() -> T, fallback: T) -> T {
    py_try(f).unwrap_or(fallback)
}

/// Python `try:` around a call: `Err(())` where Python's `except Exception` would run. A call into
/// a function not yet ported (`UNPORTED ...`) is not a Python exception, so it is not caught here:
/// it goes on up, and the checkers count it as a gap instead of a quietly wrong answer.
fn py_try<T>(f: impl FnOnce() -> T) -> Result<T, ()> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(v) => Ok(v),
        Err(e) => {
            let msg = e.downcast_ref::<String>().map(String::as_str).or_else(|| e.downcast_ref::<&str>().copied()).unwrap_or("");
            if msg.starts_with("UNPORTED") {
                std::panic::resume_unwind(e);
            }
            Err(())
        }
    }
}

/// Python: bot/memory/store_replies.py:2902 `_is_a_question(previous)`
pub fn _is_a_question(previous: &str) -> bool {
    if text::rstrip(previous).ends_with('?') {
        return true;
    }
    // An unreadable line is not this rule's business.
    or_on_raise(|| crate::understanding::understand(previous, None).kind == "question", true)
}

/// Python: bot/memory/store_replies.py:2925 `_again(who)`
///
/// The word for the person in the rest of the reply: "her", "him" or "them".
pub fn _again(who: &str) -> String {
    let word = who.to_lowercase();
    if SHE.contains(&word) {
        return "her".into();
    }
    if HE.contains(&word) {
        return "him".into();
    }
    "them".into()
}

/// Python: bot/memory/store_replies.py:2934 `_the_thing_before_the_feelings(memory)`
///
/// The last line they said that was not itself a bare feeling.
pub fn _the_thing_before_the_feelings(memory: &Memory) -> String {
    let mut seen = 0;
    for turn in recent_turns(memory, 6).iter().rev() {
        let line = text::strip(&turn.user).to_string();
        if line.is_empty() {
            continue;
        }
        if FEELING_ALONE.match_(text::rstrip_chars(&line.to_lowercase(), "!.,")).is_some() {
            continue;
        }
        seen += 1;
        if seen > 1 {
            // Only the nearest real thing they said is the subject.
            break;
        }
        return line;
    }
    String::new()
}

static SELF_IN_THE_LINE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:i|we|my|our|me)\b|\bim\b|\bive\b", 0));
static FOR_A_PRONOUN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:for|about)\s+(him|her|them)\b", crate::py::re::I));
rx!(OPENS_ON_THE_SENTENCE);

/// The bot's lines of the last `n` turns, joined.
fn recent_bot_lines(memory: &Memory, n: usize) -> String {
    recent_turns(memory, n).iter().map(|t| t.bot.clone()).collect::<Vec<_>>().join(" ")
}

/// Python: bot/memory/store_replies.py:2959 `_a_feeling_about_what_they_said(memory, low)`
///
/// The thing they just told us, said back, when the whole line is how they feel about it.
/// (Contract change: the generated stub returned `()`.)
pub fn _a_feeling_about_what_they_said(memory: &Memory, low: &str) -> Option<String> {
    let flat = text::rstrip_chars(text::strip(low), "!.,");
    let found = FEELING_ALONE.match_(flat)?;
    let previous = _the_thing_before_the_feelings(memory);
    if previous.is_empty() || text::split(&previous).len() < 4 {
        return None;
    }
    if _is_a_question(&previous) {
        return None;
    }
    if TREATMENT.is_match(&previous) {
        return None;
    }
    if !SELF_IN_THE_LINE.is_match(&previous.to_lowercase()) {
        return None;
    }
    let trouble = A_PERSON_IN_TROUBLE.is_match(&previous);
    if !(COMING_UP.is_match(&previous) || trouble) {
        return None;
    }
    if trouble {
        // The person is named in the reply itself, so the news is not read back on top of them.
        if let Some(who) = WHO_IS_IN_TROUBLE.search(&previous) {
            // A body already said stays out.
            let recent = recent_bot_lines(memory, 5);
            // The person's own word for them wins over what the kin word usually means.
            let said_for = FOR_A_PRONOUN.search(low);
            let mut again = _again(who.str(1));
            if let Some(said_for) = said_for {
                let theirs = said_for.str(1).to_lowercase();
                // ... but only where the kin word cannot tell.
                if again == "them" || theirs == again {
                    again = theirs;
                }
            }
            let whose = "your ".to_string() + &who.str(1).to_lowercase();
            let said_bodies: Vec<String> = [
                "That is a lot to be carrying, and being frightened for {who} does not make you any less help to {again}.",
                "It is all right to be frightened for {who}. You do not have to be the calm one all the time.",
                "Scared for {who} is one of the heaviest things to carry, and you are allowed to say so.",
                "Being frightened for {who} is the honest reaction, and saying it out loud once usually takes some of the weight off.",
            ]
            .iter()
            .map(|body| fmt_named(body, &[("who", &whose), ("again", &again)]))
            .collect();
            let fresh: Vec<String> = said_bodies.iter().filter(|b| !recent.contains(b.as_str())).cloned().collect();
            let fresh = if fresh.is_empty() { said_bodies } else { fresh };
            // A seed off the person as well as the turn.
            let seed = memory.turns.len() as i128 + crate::py::random::ord_sum(who.str(1));
            return Some(crate::py::random::Random::new(seed).choice(&fresh).clone());
        }
    }
    let said = crate::memory::thread::restate(&previous);
    if said.is_empty() || text::split(&said).len() < 3 {
        return None;
    }
    let said = text::upper_first(&said);
    let feeling = found.str(1).to_lowercase();
    let bodies: Vec<&str> = if feeling == "worried" || feeling == "anxious" {
        vec![
            "That is a fair thing to have on your mind, and most of it is the not-knowing.",
            "Worrying about it makes sense. Is there one thing you could check that would settle it?",
            "That kind of worry is tiring to carry, and saying the worst version out loud usually shrinks it.",
            "Worry about something coming up is the mind rehearsing it. Is there one thing you could check that would settle it?",
        ]
    } else if A_DAY.is_match(&previous) {
        vec![
            "Nerves before something like that are normal, and it is usually worse in the days before than on the day.",
            "Of course you are - most of it is the not-knowing. What is the first thing that happens?",
            "That is a fair thing to be nervous about. Tell me the part of it that keeps coming back.",
            "That is a nervous-making thing to have coming, and it says nothing bad about how ready you are.",
            "Nerves are usually worse in the days before than on the day. What is the first thing that happens?",
        ]
    } else {
        vec![
            "That is a fair thing to be nervous about, and it is all right to say so.",
            "Nerves about it make sense, and naming the worst version out loud usually shrinks it.",
            "That is a lot to be carrying around. Is there one part of it you could start on?",
            "Saying the thing you dread out loud is usually the worst part of it, and the rest is smaller than it looks.",
            "Most of the nerves is the not-knowing. What is the part that keeps coming back?",
        ]
    };
    let bodies: Vec<String> = bodies.into_iter().map(str::to_string).collect();
    // A body that was already said stays out even when the sentence in front of it was not the same.
    let recent = recent_bot_lines(memory, 5);
    let unsaid = |pool: &[String]| -> Vec<String> {
        pool.iter().filter(|b| !recent.contains(&text::slice(b, 0, Some(40)))).cloned().collect()
    };
    let first = unsaid(&bodies);
    let body = _vary(if first.is_empty() { &bodies } else { &first }, memory);
    if recent_turns(memory, 2).iter().any(|t| _said_back_already(&t.bot, &said)) {
        // The sentence is left off because the bot has just said the news, so any body that
        // opens by pointing at a sentence which is not there has nothing to point at.
        let standing: Vec<String> = bodies.iter().filter(|b| OPENS_ON_THE_SENTENCE.match_(b).is_none()).cloned().collect();
        let pool = unsaid(&standing);
        let pool = if !pool.is_empty() {
            pool
        } else if !standing.is_empty() {
            standing
        } else {
            bodies.clone()
        };
        return Some(_vary(&pool, memory));
    }
    Some(said + ". " + &body)
}

/// Python: bot/memory/store_replies.py:3080 `_said_back_already(reply, said)`
///
/// True when a bot line already carried this sentence's own subjects.
pub fn _said_back_already(reply: &str, said: &str) -> bool {
    use crate::memory::topics::keywords;
    let words: HashSet<String> = keywords(said).into_iter().filter(|w| text::len(w) > 3).collect();
    if words.is_empty() {
        return false;
    }
    let theirs: HashSet<String> = keywords(reply).into_iter().filter(|w| text::len(w) > 3).collect();
    let shared = words.iter().filter(|w| theirs.contains(*w)).count();
    shared as i64 >= 2.max(words.len() as i64 - 1)
}

strset!(ANSWER_MODULES);
static BOT_NAME_ASK: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.facts.BOT_NAME_ASK"));

/// Python: bot/memory/store_replies.py:3094 `_the_bot_name(memory, low)`
///
/// The name they gave the bot, said back when they ask for it, or None.
pub fn _the_bot_name(memory: &mut Memory, low: &str) -> Option<String> {
    let known = text::strip(&sget(&user(memory), "bot_name")).to_string();
    if known.is_empty() || !BOT_NAME_ASK.is_match(low) {
        return None;
    }
    Some(pick(
        &[
            "You call me ".to_string() + &known + ".",
            known.clone() + " \u{2014} you named me that.",
            "I'm ".to_string() + &known + ", since you named me.",
        ],
        memory,
        "",
    ))
}

rx!(ELIDED_NOT_SURE);

/// `_said_another_way(reply) or reply`.
fn another_way_or(reply: String) -> String {
    _said_another_way(&reply).filter(|s| !s.is_empty()).unwrap_or(reply)
}

/// Python: bot/memory/store_replies.py:3108 `_the_store_reply(text, memory)`
///
/// A recall ask answered about the fact it means, even with the fact left out. Python's value
/// is kept exactly: `None`, or a str, which can be "" (the memory store treats the two apart).
/// (Contract change: the generated stub returned `()`.)
pub fn _the_store_reply(text_: &str, memory: &mut Memory) -> Option<String> {
    if crate::memory::when::asks_what_today_is(text_) {
        // "what day is it" asks the calendar, and the store has no part of it.
        return None;
    }
    if let Some(behind) = _behind_a_remembering_frame(text_).filter(|b| !b.is_empty()) {
        // A remembering frame is not the line: the store answers the thing behind it.
        if let Some(r) = _remembers_fact(text_, memory).filter(|r| !r.is_empty()) {
            return Some(r);
        }
        if let Some(r) = _the_store_reply(&behind, memory).filter(|r| !r.is_empty()) {
            return Some(r);
        }
    }
    if let Some(r) = _the_followed_check(text_, memory).filter(|r| !r.is_empty()) {
        // "did you get that?" is a question about the conversation, not about the world.
        return Some(r);
    }
    if let Some(r) = _the_close(text_, memory).filter(|r| !r.is_empty()) {
        // "i just want to forget it" is the person closing the subject.
        return Some(r);
    }
    if let Some(r) = _the_limit_they_share(text_, memory).filter(|r| !r.is_empty()) {
        // The limit the turn before stated is this person's as well.
        return Some(r);
    }
    // "i had an interview today" / "how did it go": the answer is to ask them.
    if let Some(r) = some_str(crate::memory::thread::went(text_, memory)) {
        return Some(r);
    }
    if let Some(r) = some_str(crate::memory::times::answer(text_, memory)) {
        // "what time do i get up" is about the time they told us, not about sleep in general.
        return Some(r);
    }
    let took = memory.remove("pet_age").unwrap_or(Value::None);
    if took.is_dict() && is_int(took.get("age")) {
        // The bot asked how old their animal was and this line was the answer: say it back.
        return Some(
            "Noted: ".to_string()
                + &py_str(&crate::memory::relations::_pet_named(&took).val())
                + " is "
                + &crate::memory::relations::_a_pet_age(&took)
                + ".",
        );
    }
    if let Some(r) = _the_call_from_the_advice(text_, memory).filter(|r| !r.is_empty()) {
        // "should i keep running?" after an answer that said how to go on with a change.
        return Some(r);
    }
    if let Some(given) = some_str(crate::memory::facts::bot_name(text_)) {
        // A name for the bot is theirs to give, and the store keeps it.
        return Some(pick(
            &[
                given.clone() + " it is \u{2014} I'll answer to that.",
                given.clone() + " \u{2014} noted, I'll answer to it.",
                "I'll be ".to_string() + &given + " then.",
            ],
            memory,
            "",
        ));
    }
    if let Some(r) = some_str(crate::memory::choice::the_decision(text_, memory)) {
        // "what did i decide to get": the thing is in the line where they changed their mind.
        return Some(r);
    }
    if let Some(r) = _the_budget_on_the_answer(text_, memory).filter(|r| !r.is_empty()) {
        // "and i only get 20 minutes" after a lunch answer is the room that answer has.
        return Some(r);
    }
    if let Some(r) = _the_amount_they_made(text_, memory).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    if let Some(r) = _the_people_that_size_feeds(text_, memory).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    if let Some(r) = _the_figure_for_the_size_they_named(text_, memory).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    let folded = crate::memory::relations::_folded(text_, memory);
    let low = _bare(&folded);
    let undeclared = _the_person_still_in_hand(&low, memory);
    if !undeclared.is_empty() {
        // "can he have cheese" then "what about milk": one more food asked about the same person.
        if let Some(r) = _other_person_reply(memory, &low).filter(|r| !r.is_empty()) {
            return Some(r);
        }
    }
    if let Some(r) = _the_bot_name(memory, &low) {
        return Some(r);
    }
    if let Some(r) = _the_names_again(&folded, &low, memory).filter(|r| !r.is_empty()) {
        // "what are they called again?" names the pets the store is holding.
        return Some(r);
    }
    if let Some(r) = some_str(crate::memory::counts::answer(&low, memory)) {
        // "how many brothers do i have" a turn after "i have three brothers".
        return Some(r);
    }
    if let Some((said, note)) = _the_same_fact_about_someone(text_, memory) {
        // "and my wife is too" after "im allergic to nuts": the fact comes with them.
        if note.truthy() {
            crate::memory::remember_other(memory, &note, "");
        }
        return Some(said);
    }
    // "im scared" right after they told us something coming up is about that thing.
    if let Some(r) = _a_feeling_about_what_they_said(memory, &low).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    if let Some(r) = _the_fact_just_asked(memory, &low).filter(|r| !r.is_empty()) {
        // "do you remember it?" right after a recall answer is a question about that fact.
        return Some(r);
    }
    let (slot, guessed) = _elided_ask(text_, memory, &low);
    if !slot.is_empty() {
        let words = _slot_words(&slot, memory);
        let reply = _respond(&words, memory);
        let truthy = reply.as_deref().map(|r| !r.is_empty()).unwrap_or(false);
        if !truthy || (guessed && ELIDED_NOT_SURE.is_match(reply.as_deref().unwrap_or(""))) {
            // The fact the guess picked has nothing to say: fall through to the echo.
            let reply = _respond(text_, memory);
            if reply.as_deref().map(|r| !r.is_empty()).unwrap_or(false) {
                _remember_asked(memory, &_slot_of(&low));
            }
            return reply;
        }
        let reply = reply.unwrap();
        if crate::memory::relations::_same_as_last(&reply, memory) {
            // the same sentence twice in a row reads like a machine: say the fact another way.
            return Some(another_way_or(reply));
        }
        return Some(reply);
    }
    if let Some(r) = _the_case_they_just_gave(text_, memory).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    let named = _bare_ask_naming_a_fact(&low, memory);
    if !named.is_empty() {
        // "and my job?" names the fact rather than asking for it.
        let words = _slot_words(&named, memory);
        if let Some(reply) = _respond(&words, memory).filter(|r| !r.is_empty()) {
            if !ELIDED_NOT_SURE.is_match(&reply) {
                _remember_asked(memory, &named);
                if crate::memory::relations::_same_as_last(&reply, memory) {
                    return Some(another_way_or(reply));
                }
                return Some(reply);
            }
        }
    }
    // "and which knee was it": read before the store's own reply.
    if let Some(r) = some_str(crate::memory::whathurts::the_part_asked(memory, &low)) {
        return Some(r);
    }
    let falsy = |r: &Option<String>| r.as_deref().map(|s| s.is_empty()).unwrap_or(true);
    let mut reply = _respond(text_, memory);
    if falsy(&reply) {
        // "no i meant the front room" names no fact to move.
        reply = ret_opt(crate::memory::repair::the_thing_they_meant(text_, memory));
    }
    if falsy(&reply) {
        // "but i work long hours" after a question about getting a cat names nothing of its own.
        reply = ret_opt(crate::memory::carry::the_subject_in_the_condition(text_, memory));
    }
    if falsy(&reply) {
        reply = ret_opt(crate::memory::carry::the_verdict_in_the_chat(text_, memory));
    }
    if falsy(&reply) {
        // "hows about something cold" after a dessert was offered names no dish.
        reply = ret_opt(crate::memory::carry::the_quality_in_the_chat(text_, memory));
    }
    if falsy(&reply) {
        // "and no dairy" after a chicken dish takes something out of an answer that never named it.
        reply = ret_opt(crate::memory::carry::the_limit_in_the_chat(text_, memory));
    }
    if !falsy(&reply) {
        _remember_asked(memory, &_slot_of(&low));
        let r = reply.clone().unwrap();
        if crate::memory::relations::_same_as_last(&r, memory) {
            // The same sentence twice in a row reads like a machine, whatever the ask was.
            return Some(another_way_or(r));
        }
    }
    if falsy(&reply) {
        // A part of an animal the chat has described is a fact it holds.
        reply = _pet_part_reply(memory, &low);
    }
    if falsy(&reply) {
        // "and which knee was it": the store holds the part and the side it was said with.
        reply = ret_opt(crate::memory::whathurts::the_part_asked(memory, &low));
    }
    if falsy(&reply) {
        // The rate of their own doing, asked back late.
        reply = ret_opt(crate::memory::rates::answer(text_, memory));
    }
    if falsy(&reply) {
        reply = _detail_they_left_out(memory, &low);
    }
    if falsy(&reply) {
        // "im knackered" ... "why" ... "the baby": the thing they named is the reason.
        reply = ret_opt(crate::memory::cause::answer_for(text_, memory));
    }
    if falsy(&reply) {
        reply = _the_part_it_never_said(memory, &low);
    }
    reply
}

/// `DETAIL_ASKS`: (the ask, the kind of detail it wants), in order.
static DETAIL_ASKS: LazyLock<Vec<(&'static Re, String)>> =
    LazyLock::new(|| g("DETAIL_ASKS").items().iter().map(|p| (p.items()[0].as_re().unwrap(), p.items()[1].str().to_string())).collect());
rx!(DETAIL_LEAN, BARE_MEASURE, MONEY_SAID, ASK_OPENER, HOW_LONG_IT_TAKES, WANTING, PLACE_NARROWER);
static LOOK_BACK_ANSWER: LazyLock<usize> = LazyLock::new(|| g("LOOK_BACK_ANSWER").as_int().unwrap() as usize);
strset!(DETAIL_VERBS, SHAPE_WORDS);
/// `DETAIL_KNOWN`: the kind of detail -> what a line that already gave it says.
static DETAIL_KNOWN: LazyLock<HashMap<String, &'static Re>> =
    LazyLock::new(|| g("DETAIL_KNOWN").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_re().unwrap())).collect());
static TURNED: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("TURNED").str_map());

/// Python: bot/memory/store_replies.py:3376 `_the_answer_already_gives(detail, memory)`
///
/// True when the newest answer on the screen states the detail this line asks about.
pub fn _the_answer_already_gives(detail: &str, memory: &Memory) -> bool {
    if detail != "money" {
        return false;
    }
    for turn in recent_turns(memory, *LOOK_BACK_ANSWER).iter().rev() {
        match &turn.module {
            Some(m) if ANSWER_MODULES.contains(m) => {}
            _ => continue,
        }
        let reply = text::squash(&turn.bot);
        if reply.is_empty() || text::rstrip(&reply).ends_with('?') {
            continue;
        }
        return MONEY_SAID.is_match(&reply);
    }
    false
}

static TURNED_SHORT: LazyLock<Vec<(Re, &'static str)>> = LazyLock::new(|| {
    [("what is the", "what's the"), ("what is it", "what is it"), ("it is", "it's"), ("you are", "you're"), ("where is the", "where's the")]
        .into_iter()
        .map(|(long_form, short)| (Re::new(&format!(r"\b{long_form}\b"), 0), short))
        .collect()
});

/// Python: bot/memory/store_replies.py:3449 `_turned(question)`
///
/// The question said back to them in their own person ("my" -> "your").
pub fn _turned(question: &str) -> String {
    let words = text::split(text::rstrip_chars(question, " ?!."));
    let mut said = words.iter().map(|w| TURNED.get(&w.to_lowercase()).cloned().unwrap_or_else(|| w.clone())).collect::<Vec<_>>().join(" ");
    // The understanding writes "what is the pay" out in full; said back to them it reads stiff.
    for (pattern, short) in TURNED_SHORT.iter() {
        said = pattern.sub(short, &said);
    }
    said
}

static CHOICE_ASK: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.answers.CHOICE_ASK"));

/// Python: bot/memory/store_replies.py:3459 `_a_choice_from_what_was_said(line, memory)`
///
/// "which one?" after a recommendation, when the last answer has a pick in it.
pub fn _a_choice_from_what_was_said(line: &str, memory: &Memory) -> bool {
    let clean = fold(line);
    if CHOICE_ASK.match_(&clean).is_none() {
        return false;
    }
    // A pick that cannot be made declines nothing.
    or_on_raise(|| crate::modules::answers::_the_one_to_pick(&clean, &memory_value(memory)).val().truthy(), false)
}

static ASKS_A_DETAIL: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:how|whats?|where|when|which|whos?)\b", 0));
static THE_PAY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bthe (?:pay|price|salary)\b", 0));
static THEIR_OWN_NEWS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:i|we|my|our|me)\b|\bive\b|\bim\b", 0));
static A_COST: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bcost\b", 0));
static EVENT_WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.when.EVENT_WORD"));

/// Python: bot/memory/store_replies.py:3475 `_detail_they_left_out(memory, low)`
///
/// A warm ask back when the question is about a detail of what they just told us.
pub fn _detail_they_left_out(memory: &mut Memory, low: &str) -> Option<String> {
    let line = text::strip(low).to_string();
    if line.is_empty() || text::split(&line).len() > 9 {
        return None;
    }
    if _a_clock_ask(&line) && !EVENT_WORD.is_match(&last_turn_user(memory)) {
        // "what time is it" is a question about the clock, not a detail of their news.
        return None;
    }
    if _about_the_bot_here(memory) {
        // "where are you" asks the bot, not for a detail of the person's own news.
        return None;
    }
    if _a_choice_from_what_was_said(&line, memory) {
        return None;
    }
    if !ASKS_A_DETAIL.is_match(&line) {
        return None;
    }
    let detail = DETAIL_ASKS.iter().find(|(p, _)| p.is_match(&line)).map(|(_, k)| k.clone())?;
    // Nothing of its own: every other word is a function word or one of the light verbs.
    let spare = LETTER_WORDS.findall_str(&line).into_iter().any(|w| {
        !DETAIL_VERBS.contains(&w) && !SHAPE_WORDS.contains(&w) && !["how", "what", "where", "when", "which", "why", "s"].contains(&w.as_str())
    });
    if spare {
        return None;
    }
    // A bare "how much", "how many", "how long" or "how old" is a question about the world.
    if !DETAIL_LEAN.is_match(&line) && !THE_PAY.is_match(&line) {
        if ["money", "count", "time"].contains(&detail.as_str()) || BARE_MEASURE.is_match(&line) {
            return None;
        }
    }
    // The thing on the table has stopped working, so a measure ask about it points at that thing.
    if or_on_raise(|| crate::memory::named::a_faulty_thing(memory), false) {
        return None;
    }
    let previous = text::strip(&last_turn_user(memory)).to_string();
    if previous.is_empty() || previous.ends_with('?') || text::split(&previous).len() < 3 {
        return None;
    }
    // The line before was a question, typed without its mark as often as with it.
    if ASK_OPENER.match_(&previous.to_lowercase()).is_some() {
        return None;
    }
    // Their own news, not a question to us.
    if !THEIR_OWN_NEWS.is_match(&previous.to_lowercase()) {
        return None;
    }
    // ... and something that happened or is the case, not a want or a plan.
    if WANTING.is_match(&previous.to_lowercase()) {
        return None;
    }
    // A destination does not answer an ask about a narrower place.
    let narrower = detail == "place" && PLACE_NARROWER.is_match(&line);
    if let Some(known) = DETAIL_KNOWN.get(&detail) {
        if !narrower && known.is_match(&previous) {
            return None;
        }
    }
    if detail == "time" && HOW_LONG_IT_TAKES.is_match(&line) {
        return None;
    }
    if _the_answer_already_gives(&detail, memory) {
        return None;
    }
    if detail == "money" && A_COST.is_match(&line) {
        return Some("You didn't say what it cost - how much was it?".to_string());
    }
    let ask = _turned(&line);
    if ask.is_empty() {
        return None;
    }
    Some("You haven't said - ".to_string() + &ask + "?")
}

rx!(PART_ASK, PART_MINE, PART_POINTS, RECORD_PART, PART_ADVICE, PART_HELP, JOB_PART);
static PART_OF: LazyLock<Vec<(String, &'static Re)>> =
    LazyLock::new(|| g("PART_OF").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].as_re().unwrap())).collect());
strs!(PART_KEYS, KEPT_THINGS);
static PART_HELD: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("PART_HELD").str_map());
static A_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d", 0));

/// Python: bot/memory/store_replies.py:3600 `_a_held_part(memory, line)`
///
/// True when a fact the store keeps is one of the things the line names.
pub fn _a_held_part(memory: &mut Memory, line: &str) -> bool {
    let known = user(memory);
    PART_OF.iter().any(|(key, p)| p.is_match(line) && known.get(key).map(Value::truthy).unwrap_or(false))
}

/// Python: bot/memory/store_replies.py:3608 `_a_part_of_the_job_never_said(memory, low)`
///
/// The ask back for a part of the job the person never described, or None.
pub fn _a_part_of_the_job_never_said(memory: &mut Memory, low: &str) -> Option<String> {
    let line = text::strip(low).to_string();
    if line.is_empty() || text::split(&line).len() > 8 {
        return None;
    }
    if !user(memory).get("job").map(Value::truthy).unwrap_or(false) {
        return None;
    }
    if PART_ADVICE.is_match(&line) || PART_HELP.is_match(&line) || A_DIGIT.is_match(&line) {
        return None;
    }
    if !JOB_PART.is_match(&line) {
        return None;
    }
    let ask = _turned(&line);
    if ask.is_empty() {
        return None;
    }
    Some("You haven't said - ".to_string() + &ask + "?")
}

/// Python: bot/memory/store_replies.py:3624 `_holds_nothing(known)`
pub fn _holds_nothing(known: &Value) -> bool {
    !PART_KEYS.iter().any(|k| known.get(k).map(Value::truthy).unwrap_or(false))
}

/// Python: bot/memory/store_replies.py:3627 `_a_thing_it_keeps(memory, mine)`
///
/// The rows of the store a possessive names, or None when the chat has been told nothing.
pub fn _a_thing_it_keeps(memory: &mut Memory, mine: &str) -> Option<String> {
    let known = user(memory);
    if _holds_nothing(&known) {
        return None;
    }
    let key = PART_HELD.get(&mine.to_lowercase())?.clone();
    let has = |k: &str| known.get(k).map(Value::truthy).unwrap_or(false);
    let held = match key.as_str() {
        "people" => has("people") || has("others") || has("kids"),
        "city" => has("city") || has("home") || has("places"),
        k => has(k),
    };
    if held {
        Some(key)
    } else {
        None
    }
}

static I_OR_IM: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:i|im)\b", 0));

/// Python: bot/memory/store_replies.py:3643 `_the_part_it_never_said(memory, low)`
///
/// The honest ask back for a part of a kept fact they never gave, or None.
pub fn _the_part_it_never_said(memory: &mut Memory, low: &str) -> Option<String> {
    let line = text::strip(low).to_string();
    if line.is_empty() || text::split(&line).len() > 8 {
        return None;
    }
    if PART_ADVICE.is_match(&line) || PART_HELP.is_match(&line) || A_DIGIT.is_match(&line) {
        return None;
    }
    if !PART_ASK.is_match(&line) || !RECORD_PART.is_match(&line) {
        return None;
    }
    if let Some(mine) = PART_MINE.search(&line) {
        if _a_thing_it_keeps(memory, mine.str(1)).is_none() {
            return None;
        }
    } else if !(PART_POINTS.is_match(&line) || (I_OR_IM.is_match(&line) && _a_held_part(memory, &line))) {
        return None;
    }
    // A pointing word is about what was just said, so there has to be a fact line behind it.
    if !_a_kept_thing_behind(memory) {
        return None;
    }
    let ask = _turned(&line);
    if ask.is_empty() {
        return None;
    }
    Some("You haven't said - ".to_string() + &ask + "?")
}

static ABOUT_THEMSELVES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:i|we|my|our|im|ive)\b", 0));

/// Python: bot/memory/store_replies.py:3675 `_a_kept_thing_behind(memory)`
///
/// True when the nearest thing they said about themselves is a fact the store keeps.
pub fn _a_kept_thing_behind(memory: &Memory) -> bool {
    for turn in recent_turns(memory, 5).iter().rev() {
        let said = text::strip(&turn.user).to_string();
        if said.is_empty() || said.ends_with('?') {
            continue;
        }
        if !ABOUT_THEMSELVES.is_match(&said.to_lowercase()) {
            continue;
        }
        // A line that cannot be read keeps the old answer.
        let found = match py_try(|| crate::memory::facts::extract(&said, false).val()) {
            Ok(v) => v,
            Err(_) => return false,
        };
        if !found.is_dict() {
            return false;
        }
        if KEPT_THINGS.iter().any(|k| found.get(k).map(Value::truthy).unwrap_or(false)) {
            return true;
        }
    }
    false
}

strset!(DRINK_DIET);
static SPACES_COMMAS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[ ,]+", 0));

/// Python: bot/memory/store_replies.py:3704 `_diet_how(diet)`
///
/// The verb a diet is said with: what you drink is not what you eat.
pub fn _diet_how(diet: &str) -> String {
    let rest = text::slice(diet, 3, None);
    let words: HashSet<String> = SPACES_COMMAS.split(&text::strip(&rest).to_lowercase()).into_iter().collect();
    if words.iter().any(|w| DRINK_DIET.contains(w)) { "drink" } else { "eat" }.to_string()
}

rx!(REMEMBER_IT);

/// Python: bot/memory/store_replies.py:3714 `_the_fact_just_asked(memory, low)`
///
/// The fact the store answered on the turn before, said again for "do you remember it?".
pub fn _the_fact_just_asked(memory: &mut Memory, low: &str) -> Option<String> {
    if REMEMBER_IT.match_(text::strip(low)).is_none() {
        return None;
    }
    let asked = mget(memory, "asked").clone();
    if !asked.is_dict() || !asked.get("slot").map(Value::truthy).unwrap_or(false) {
        return None;
    }
    let turn = asked.get("turn").filter(|t| t.truthy()).cloned().unwrap_or(Value::Int(0));
    let asked_turn = py_int_value(&turn)?;
    let turns = memory.turns.len() as i64;
    if turns == 0 || turns - asked_turn > 2 {
        return None;
    }
    if last_module(memory).as_deref() != Some("memory_store") {
        return None; // a question in between ("whats my name" / "nice one" / "remember it?")
    }
    let words = _slot_words(&py_str(asked.get("slot").unwrap()), memory);
    let reply = _respond(&words, memory).filter(|r| !r.is_empty())?;
    if ELIDED_NOT_SURE.is_match(&reply) {
        return None;
    }
    Some(_yes_then(&reply))
}

static YOU_OPENS: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:You|Your|It)\b", 0));

/// Python: bot/memory/store_replies.py:3741 `_yes_then(reply)`
///
/// The fact with a yes in front of it, "you" still lower case after the dash.
pub fn _yes_then(reply: &str) -> String {
    let mut text_ = text::strip(reply).to_string();
    if YOU_OPENS.match_(&text_).is_some() {
        text_ = lower_first(&text_);
    }
    "Yes \u{2014} ".to_string() + &text_
}

/// `CONFIRM_FRAMES`: (the yes/no frame, the statement's lead).
static CONFIRM_FRAMES: LazyLock<Vec<(&'static Re, String)>> =
    LazyLock::new(|| g("CONFIRM_FRAMES").items().iter().map(|p| (p.items()[0].as_re().unwrap(), p.items()[1].str().to_string())).collect());
rx!(YES_NO_ASK, YES_NO_FRAME, TELLING_AFTER_ASK);

/// Python: bot/memory/store_replies.py:3767 `_after_the_yes_no_frame(text)`
///
/// What the line says after a yes/no question about someone else, or "".
pub fn _after_the_yes_no_frame(text_: &str) -> String {
    let line = text::squash(text_);
    if let Some(m) = YES_NO_FRAME.match_(&line) {
        return text::strip(&line[m.end()..]).to_string();
    }
    // Most people type no mark at all: the ask ends where the telling begins.
    let Some(opened) = YES_NO_ASK.match_(&line) else { return String::new() };
    let after_who = &line[opened.end()..];
    let Some(telling) = TELLING_AFTER_ASK.search(after_who) else { return String::new() };
    text::strip(&after_who[telling.start()..]).to_string()
}

static STATEMENT_LEAD: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:sorry|hey|hi|ok|okay|so|but|and|well|quick one)[, ]+", 0));
static IS_MY_THING: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:is|was) my\s+(\S+)\s+(.+)$", 0));
static ONE_WORD_NAME: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:called |named )?\S+$", 0));

/// Python: bot/memory/store_replies.py:3784 `_statement(text)`
///
/// The plain statement a yes/no question about the person makes: "am i vegetarian?" is "i am
/// vegetarian". None when the line is not that shape. (Contract change: the generated stub
/// returned `()`.)
pub fn _statement(text_: &str) -> Option<String> {
    let low = text::strip_chars(&fold(text_), " ?!.").to_string();
    let low = STATEMENT_LEAD.sub("", &low);
    for (pattern, lead) in CONFIRM_FRAMES.iter() {
        if let Some(m) = pattern.match_(&low) {
            return Some(lead.clone() + m.str(1));
        }
    }
    // "is my dog called biscuit" -> "my dog is called biscuit". Only a short answer is read this way.
    if let Some(m) = IS_MY_THING.match_(&low) {
        if ONE_WORD_NAME.match_(m.str(2)).is_some() || m.str(2).starts_with("called ") {
            return Some("my ".to_string() + m.str(1) + " is " + m.str(2));
        }
    }
    None
}

/// Python: bot/memory/store_replies.py:3804 `_same(left, right)`
pub fn _same(left: &Value, right: &Value) -> bool {
    fold(&py_str(left)) == fold(&py_str(right))
}

/// Python: bot/memory/store_replies.py:3807 `_clause(key, value, known=None)`
///
/// The fact said the way a friend would say it back: "you're vegetarian".
pub fn _clause(key: &str, value: &Value, known: &Value) -> String {
    match key {
        "name" | "age" => "you're ".to_string() + &py_str(value),
        "job" => {
            // A job told with a place preposition is a place, so the frame stored with it says it back.
            let frame = text::strip(&sget(known, "job_frame")).to_string();
            if !frame.is_empty() && _same(value, known.get("job").unwrap_or(&Value::None)) {
                let said = _job_said(&dict_of(vec![("job", Value::from(py_str(value))), ("job_frame", Value::from(frame.as_str()))]));
                return lower_first(&said);
            }
            "you're ".to_string() + &crate::memory::relations::_a(&py_str(value))
        }
        "city" => "you live in ".to_string() + &py_str(value),
        "car" => "you drive ".to_string() + &py_str(value),
        "from" => "you're from ".to_string() + &py_str(value),
        "allergies" => "you're allergic to ".to_string() + &py_list(value).iter().map(py_str).collect::<Vec<_>>().join(", "),
        "budget" => "your budget is ".to_string() + &py_str(value),
        "diet" => {
            let diet = py_str(value);
            if diet.starts_with("no ") {
                "you don't ".to_string() + &_diet_how(&diet) + " " + &text::slice(&diet, 3, None)
            } else {
                "you're ".to_string() + &diet
            }
        }
        "pets" => {
            let pets: Vec<Value> = match value {
                Value::List(items) => items.to_vec(),
                other => vec![other.clone()],
            };
            "you've got ".to_string() + &pets.iter().map(_a_pet).collect::<Vec<_>>().join(" and ")
        }
        _ => String::new(),
    }
}

/// Python: bot/memory/store_replies.py:3841 `_their_allergy(who, theirs)`
///
/// Someone else's allergy, said back with whose it is: a no about the person asked about.
pub fn _their_allergy(who: &str, theirs: &[Value]) -> String {
    "You haven't told me one \u{2014} the allergy to ".to_string()
        + &theirs.iter().map(py_str).collect::<Vec<_>>().join(", ")
        + " is your "
        + who
        + "'s, not yours."
}

rx!(DIET_ANY);
static NOT_A_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^0-9]", 0));

/// Python: bot/memory/store_replies.py:3848 `_said_matches(key, said, known)`
///
/// Whether what the question states is what they told you. None when it is not about a fact
/// the store would keep.
pub fn _said_matches(key: &str, said: &Value, known: &Value) -> Option<bool> {
    match key {
        "name" => Some(if known.truthy() { _same(said, known) } else { false }),
        "age" => Some(!known.is_none() && py_str(said) == py_str(known)),
        "diet" if DIET_ANY.match_(text::strip(&py_str(said))).is_some() => {
            // They asked about a diet in general, not about a diet they named.
            None
        }
        "job" | "city" | "from" | "diet" => Some(if known.truthy() { _same(said, known) } else { false }),
        "budget" => {
            let mine = NOT_A_DIGIT.sub("", &py_str(said));
            let theirs = NOT_A_DIGIT.sub("", &s_or(Some(known)));
            Some(!mine.is_empty() && mine == theirs)
        }
        "allergies" => {
            let stored: HashSet<String> = py_list(if known.truthy() { known } else { &Value::None }).iter().map(|a| py_str(a).to_lowercase()).collect();
            Some(said.truthy() && py_list(said).iter().all(|a| stored.contains(&py_str(a).to_lowercase())))
        }
        "pets" => {
            let family = |v: Option<&Value>| crate::memory::relations::_animal_family(&s_or(v));
            for pet in py_list(if known.truthy() { known } else { &Value::None }) {
                if family(pet.get("animal")) != family(said.get("animal")) {
                    continue;
                }
                if !said.get("name").map(Value::truthy).unwrap_or(false)
                    || _same(pet.get("name").unwrap_or(&Value::None), said.get("name").unwrap_or(&Value::None))
                {
                    return Some(true);
                }
            }
            Some(false)
        }
        _ => None,
    }
}

static NAMED_BY: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("NAMED_BY").str_map());

/// Python: bot/memory/store_replies.py:3877 `_names(key, text)`
pub fn _names(key: &str, text_: &str) -> bool {
    match NAMED_BY.get(key) {
        Some(p) if !p.is_empty() => Re::new(p, 0).is_match(text_),
        _ => false,
    }
}

rx!(DUTY_ASK, REMEMBERING_FRAME, RESTATED, CONFIRM_TAG, SAYS_SOMETHING);

/// Whether a remembering frame asks something: a question mark, an auxiliary, the bare verb,
/// or "remember"/"recall" as the verb.
fn _frame_asks(frame: &crate::py::re::Match, question: bool) -> bool {
    let verb = frame.name("verb").unwrap_or("");
    question
        || frame.name("aux").map(|s| !s.is_empty()).unwrap_or(false)
        || frame.name("bare").map(|s| !s.is_empty()).unwrap_or(false)
        || verb.starts_with("remember")
        || verb.starts_with("recall")
}

/// Python: bot/memory/store_replies.py:3907 `_restated(text, typed=None)`
///
/// The statement behind a remembering frame: "do you remember that i live in leeds" -> "i live
/// in leeds". None when the line is not that shape.
pub fn _restated(text_: &str, typed: Option<&str>) -> Option<String> {
    let low = text::strip(&fold(text_)).to_string();
    let tagged = CONFIRM_TAG.is_match(&low);
    let low = text::strip_chars(&CONFIRM_TAG.sub("", &low), " ?!.,").to_string();
    let frame = REMEMBERING_FRAME.match_(&low)?;
    let question = typed.unwrap_or(text_).contains('?');
    // "you know im veggie" without a question mark is the person telling you again.
    if !_frame_asks(&frame, question || tagged) {
        return None;
    }
    let rest = text::strip_chars(&low[frame.end()..], " ?!.,").to_string();
    if rest.is_empty() || RESTATED.match_(&rest).is_none() {
        return None;
    }
    Some(rest)
}

/// Python: bot/memory/store_replies.py:3930 `_behind_a_remembering_frame(text)`
///
/// The line a remembering frame is around, when it is a line the store can read.
/// (Contract change: the generated stub returned `()`.)
pub fn _behind_a_remembering_frame(text_: &str) -> Option<String> {
    let low = text::strip_chars(&fold(text_), " ?!.,").to_string();
    let frame = REMEMBERING_FRAME.match_(&low)?;
    if !_frame_asks(&frame, text_.contains('?')) {
        return None;
    }
    let rest = text::strip_chars(&low[frame.end()..], " ?!.,").to_string();
    if rest.is_empty() || rest == low {
        return None;
    }
    if !SAYS_SOMETHING.is_match(&rest) {
        // "do you remember my mums age" is a question about a fact, not the fact.
        return None;
    }
    if crate::memory::facts::extract(&rest, false).val().truthy() {
        return Some(rest);
    }
    None
}

strs!(RESTATED_KEYS);

/// Python: bot/memory/store_replies.py:3966 `_remembers_fact(text, memory)`
///
/// Yes or no to "do you remember that i ...", from the fact they told the store.
pub fn _remembers_fact(text_: &str, memory: &mut Memory) -> Option<String> {
    let folded = crate::memory::relations::_folded(text_, memory);
    let said = _restated(&folded, Some(text_))?;
    let found = crate::memory::facts::extract(&said, false).val();
    let known = user(memory);
    let (mut yes, mut no): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    for key in RESTATED_KEYS.iter() {
        let mut value = found.get(key).cloned().unwrap_or(Value::None);
        if !value.truthy() {
            continue;
        }
        let stored = known.get(key).cloned().unwrap_or(Value::None);
        if !stored.truthy() {
            continue;
        }
        if key == "pets" {
            let pets: Vec<Value> = py_list(&value)
                .into_iter()
                .filter(|p| p.is_dict() && (p.get("name").map(Value::truthy).unwrap_or(false) || p.get("animal").map(Value::truthy).unwrap_or(false)))
                .collect();
            let Some(first) = pets.into_iter().next() else { continue };
            value = first;
        }
        let matches = _said_matches(key, &value, &stored);
        let said_back = _clause(key, &stored, &known);
        if said_back.is_empty() {
            continue;
        }
        match matches {
            Some(true) => yes.push(said_back),
            Some(false) => no.push(said_back),
            None => {}
        }
    }
    if let Some(fact) = yes.first() {
        return Some(pick(
            &[
                "Yes \u{2014} ".to_string() + fact + ".",
                text::upper_first(fact) + " \u{2014} I have that.",
                "That's right \u{2014} ".to_string() + fact + ".",
            ],
            memory,
            "",
        ));
    }
    if let Some(fact) = no.first() {
        return Some(pick(&["You told me ".to_string() + fact + ".", "Not quite \u{2014} ".to_string() + fact + "."], memory, ""));
    }
    // A frame around a fact the store has just been told about someone else: the fact it now
    // holds is said back.
    _someone_said_back(&said, memory)
}

/// Python: bot/memory/store_replies.py:4022 `_someone_said_back(said, memory)`
///
/// The fact about someone else that the framed statement just gave, said back.
pub fn _someone_said_back(said: &str, memory: &mut Memory) -> Option<String> {
    let words = fold(said);
    if words.is_empty() {
        return None;
    }
    let found = crate::memory::facts::extract(said, false).val();
    let said_about = dicts(found.get("others").unwrap_or(&Value::None));
    let empty = [Value::None, Value::from(""), Value::list(), Value::dict()];
    for entry in others(memory).items() {
        if !entry.is_dict() {
            continue;
        }
        let who = text::strip(&sget(entry, "who")).to_lowercase();
        if who.is_empty() || !says_word(&who, &words) {
            continue;
        }
        if entry.get("name").is_some() {
            let called = crate::memory::relations::_called(text::strip(&sget(entry, "name")));
            if !called.is_empty() {
                return Some("Your ".to_string() + &who + " is called " + &called + ".");
            }
            continue;
        }
        // A figure said about someone in the frame is said back the same way.
        let age = entry.get("age").cloned().unwrap_or(Value::None);
        if empty.iter().any(|e| *e == age) {
            continue;
        }
        let given: Vec<String> =
            said_about.iter().filter(|o| sget(o, "who").to_lowercase() == who).map(|o| py_str(o.get("age").unwrap_or(&Value::None))).collect();
        if given.contains(&py_str(&age)) {
            return Some("Your ".to_string() + &who + " is " + &py_str(&age) + ".");
        }
    }
    None
}

/// `FOOD_CATEGORIES`: (category, the words for it).
static FOOD_CATEGORIES: LazyLock<Vec<(String, Vec<String>)>> =
    LazyLock::new(|| g("FOOD_CATEGORIES").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].strs())).collect());
strset!(NOT_THE_FOOD);
static A_DIET_RULES_OUT: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| g("A_DIET_RULES_OUT").str_list_map());
static A_CONDITION_RULES_OUT: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| g("A_CONDITION_RULES_OUT").str_list_map());
rx!(EATING_ASK, LEAD_IN);

/// Python: bot/memory/store_replies.py:4108 `_the_food_named(rest)`
///
/// The category a bare food phrase names, or "" when it names none of them.
pub fn _the_food_named(rest: &str) -> String {
    let said: Vec<String> = LETTER_WORDS.findall_str(&rest.to_lowercase()).into_iter().filter(|w| !NOT_THE_FOOD.contains(w)).collect();
    if said.is_empty() {
        return String::new();
    }
    let unplural = |w: &str| text::rstrip_chars(w, "s").to_string();
    for (category, words) in FOOD_CATEGORIES.iter() {
        if said.iter().all(|word| words.iter().any(|one| word == one || *word == unplural(one) || unplural(word) == unplural(one))) {
            return category.clone();
        }
    }
    String::new()
}

/// Python: bot/memory/store_replies.py:4121 `_the_rule_that_rules_it_out(known, category)`
///
/// The rule they gave the store that takes this food off the table, said back, or "".
pub fn _the_rule_that_rules_it_out(known: &Value, category: &str) -> String {
    let diet = fold(&sget(known, "diet"));
    let mut taken: Vec<String> = A_DIET_RULES_OUT.get(&diet).cloned().unwrap_or_default();
    if known.get("no_meat").map(Value::truthy).unwrap_or(false) && !taken.iter().any(|t| t == "meat") {
        taken.push("meat".into());
    }
    if taken.iter().any(|t| t == category) {
        if !diet.is_empty() {
            return "No \u{2014} ".to_string() + &_clause("diet", known.get("diet").unwrap_or(&Value::None), &Value::None) + ".";
        }
        return "No \u{2014} you don't eat meat.".to_string();
    }
    let unplural = |w: &str| text::rstrip_chars(w, "s").to_string();
    let words: Vec<String> = FOOD_CATEGORIES.iter().find(|(c, _)| c == category).map(|(_, w)| w.clone()).expect("KeyError");
    for allergy in py_list(known.get("allergies").filter(|v| v.truthy()).unwrap_or(&Value::None)) {
        let said = py_str(&allergy).to_lowercase();
        if words.iter().any(|one| said == *one || unplural(&said) == unplural(one)) {
            return "No \u{2014} ".to_string() + &_clause("allergies", known.get("allergies").unwrap_or(&Value::None), &Value::None) + ".";
        }
    }
    for condition in py_list(known.get("condition").filter(|v| v.truthy()).unwrap_or(&Value::None)) {
        let word = fold(&py_str(&condition));
        if A_CONDITION_RULES_OUT.get(&word).map(|c| c.iter().any(|x| x == category)).unwrap_or(false) {
            return "No \u{2014} you've told me you're ".to_string() + &word + ", so " + category + " is out.";
        }
    }
    String::new()
}

/// Python: bot/memory/store_replies.py:4145 `_a_rule_on_eating(text, memory)`
///
/// A yes/no about eating or drinking a food, answered from the rule the store keeps.
pub fn _a_rule_on_eating(text_: &str, memory: &mut Memory) -> Option<String> {
    let line = fold(text_);
    let led = LEAD_IN.sub("", &line);
    let m = EATING_ASK.match_(&led)?;
    let category = _the_food_named(text::strip_chars(m.str(1), " ?!."));
    if category.is_empty() {
        return None;
    }
    let known = user(memory);
    nz(_the_rule_that_rules_it_out(&known, &category))
}

static NOT_IN_THE_ASK: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:not|n't)\b", 0));
static AN_OR: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bor\b", 0));

/// Python: bot/memory/store_replies.py:6521 `_other_fact(memory, key)`
///
/// Someone else's fact the user mentioned: (who, values), or ("", []) when there is none.
pub fn _other_fact(memory: &mut Memory, key: &str) -> (String, Vec<Value>) {
    for entry in others(memory).items() {
        if let Some(Value::List(values)) = entry.get(key) {
            if !values.is_empty() {
                return (s_or(entry.get("who")), values.to_vec());
            }
        }
    }
    (String::new(), Vec::new())
}

/// Python: bot/memory/store_replies.py:4170 `_confirmation(text, memory)`
///
/// Answer a yes/no question about something they told you, from what they told you.
pub fn _confirmation(text_: &str, memory: &mut Memory) -> Option<String> {
    let statement = _statement(text_)?;
    if statement.is_empty() || DUTY_ASK.is_match(&statement) {
        return None;
    }
    // "am i not allergic to nuts?" asks for the opposite yes or no.
    let flat = fold(text_);
    let inverted = NOT_IN_THE_ASK.is_match(&flat);
    let found = crate::memory::facts::extract(&statement, false).val();
    if !found.truthy() {
        return None;
    }
    let known = user(memory);
    let (mut yes, mut no): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    let mut elsewhere: Vec<(String, String, Vec<Value>)> = Vec::new();
    for (k, value) in found.pairs().to_vec() {
        let key = py_str(&k);
        if !["name", "age", "job", "city", "from", "diet", "budget", "allergies", "pets"].contains(&key.as_str()) {
            continue;
        }
        let stored = known.get(&key).cloned().unwrap_or(Value::None);
        let mut value = value;
        // A pet is read as a list even for one animal, and only a named one is an assertion.
        if key == "pets" {
            let pets: Vec<Value> = py_list(&value)
                .into_iter()
                .filter(|p| p.get("name").map(Value::truthy).unwrap_or(false) || p.get("animal").map(Value::truthy).unwrap_or(false))
                .collect();
            let Some(first) = pets.into_iter().next() else { continue };
            value = first;
        }
        // Only a fact the store actually holds is confirmed or denied.
        if !stored.truthy() {
            // A fact they gave about someone else, asked about themselves, is a no that says whose.
            let (theirs_who, theirs) = _other_fact(memory, &key);
            if !theirs_who.is_empty() && !theirs.is_empty() {
                elsewhere.push((key.clone(), theirs_who, theirs));
            }
            continue;
        }
        let Some(matches) = _said_matches(&key, &value, &stored) else { continue };
        if matches {
            yes.push(_clause(&key, if stored.truthy() { &stored } else { &value }, &known));
        } else if _names(&key, &flat) {
            let said_back = _clause(&key, if stored.truthy() { &stored } else { &value }, &known);
            if !said_back.is_empty() {
                no.push(said_back);
            }
        }
    }
    if inverted {
        std::mem::swap(&mut yes, &mut no);
    }
    if yes.is_empty() && no.is_empty() {
        for (key, who, theirs) in &elsewhere {
            if key == "allergies" {
                return Some(_their_allergy(who, theirs));
            }
        }
        return None;
    }
    if !no.is_empty() && yes.is_empty() {
        // "do i live in york or leeds?" is an either/or, so the answer is just the fact.
        if AN_OR.is_match(&flat) {
            return Some(text::upper_first(&no[0]) + ".");
        }
        return Some(pick(&["No \u{2014} ".to_string() + &no[0] + ".", "Not quite \u{2014} ".to_string() + &no[0] + "."], memory, ""));
    }
    let said = if yes.is_empty() { String::new() } else { yes[..yes.len().min(2)].join("; ") };
    if !no.is_empty() && !yes.is_empty() {
        return Some(pick(
            &[
                "Yes on one thing, no on another: ".to_string() + &said + ", but " + &no[0] + ".",
                "Partly \u{2014} ".to_string() + &said + ", though " + &no[0] + ".",
            ],
            memory,
            "",
        ));
    }
    Some(pick(
        &[
            "Yes \u{2014} ".to_string() + &said + ".",
            "That's right \u{2014} ".to_string() + &said + ".",
            "Yes, you told me \u{2014} ".to_string() + &said + ".",
        ],
        memory,
        "",
    ))
}

strset!(SWAPPABLE, MONTHS);
static FACTS_DAYS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.DAYS"));
static SPACES_OR_COMMAS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[\s,]+", 0));

/// Python: bot/memory/store_replies.py:4248 `_old_values(memory, told)`
///
/// The words of the line before the correction that a correction could have swapped out.
pub fn _old_values(memory: &Memory, told: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let Some(Value::List(entries)) = memory.get("facts_log") else { return out };
    for entry in entries.iter().rev() {
        if !entry.is_dict() {
            continue;
        }
        if fold(&sget(entry, "text")) != told.to_lowercase() {
            continue;
        }
        match entry.get("key").and_then(|k| k.as_str()) {
            Some(k) if SWAPPABLE.contains(k) => {}
            _ => continue,
        }
        for word in SPACES_OR_COMMAS.split(&sget(entry, "value")) {
            let low = text::strip_chars(&word.to_lowercase(), "'-.!?").to_string();
            if text::len(&low) > 2
                && !MONTHS.contains(&low)
                && !FACTS_DAYS.contains(&low)
                && !STOPWORDS.contains(&low)
                && !out.contains(&low)
                && Re::new(&format!(r"\b{}\b", crate::py::re::escape(&low)), crate::py::re::I).is_match(told)
            {
                out.push(low);
            }
        }
    }
    out
}

rx!(PUTTING_BY);
static A_FIGURE_WORD: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\d[\d,]*(?:\.\d+)?\s*(?:k|grand|thousand|pounds?|quid)?", crate::py::re::I));

/// Python: bot/memory/store_replies.py:4278 `_a_figure(word)`
///
/// True when the value is a number rather than a word of the sentence ("100", "2k", "1,500").
pub fn _a_figure(word: &Value) -> bool {
    A_FIGURE_WORD.fullmatch(text::strip(&s_or(Some(word)))).is_some()
}

/// `re.sub(r"\b" + re.escape(old) + r"\b", new, told, count=1, flags=re.I)`.
fn swap_once(old: &str, new: &str, told: &str) -> String {
    Re::new(&format!(r"\b{}\b", crate::py::re::escape(old)), crate::py::re::I).subn(new, told, 1)
}

/// `re.search(r"\b" + re.escape(word) + r"\b", text, re.I)`.
fn says_word_i(word: &str, text_: &str) -> bool {
    Re::new(&format!(r"\b{}\b", crate::py::re::escape(word)), crate::py::re::I).is_match(text_)
}

/// Python: bot/memory/store_replies.py:4256 `_swapped_lines(memory, fixed)`
///
/// The lines the person would have said if they had said it right the first time.
pub fn _swapped_lines(memory: &Memory, fixed: &Value) -> Vec<String> {
    if !fixed.is_dict() {
        return Vec::new();
    }
    if fixed.get("dropped").map(Value::truthy).unwrap_or(false) {
        // Nothing took the value's place, so there is no line they "would have said".
        return Vec::new();
    }
    let value = text::strip(&sget(fixed, "value")).to_string();
    let said: Vec<String> = recent_turns(memory, 3).iter().map(|t| t.user.clone()).collect();
    // The correction line itself is the newest turn on some paths and not yet recorded on others.
    let mine = fold(&sget(fixed, "said"));
    let said: Vec<String> = said.into_iter().filter(|x| fold(x) != mine).collect();
    let told = text::squash(said.last().map(String::as_str).unwrap_or(""));
    if value.is_empty() || told.is_empty() {
        return Vec::new();
    }
    if let Some(moved) = _the_time_that_moved(&told, fixed, &value).filter(|m| !m.is_empty()) {
        // The time is the one value in the sentence that cannot be added beside the old one.
        return vec![moved];
    }
    let mut olds = _old_values(memory, &told);
    let from = fixed.get("from").cloned().unwrap_or(Value::None);
    if _a_figure(&from) && _a_figure(&Value::from(value.as_str())) {
        // The thing corrected is the figure in the line and the new thing is a figure too.
        olds.retain(|old| _a_figure(&Value::from(old.as_str())));
    }
    let mut out: Vec<String> = olds.iter().map(|old| swap_once(old, &value, &told)).collect();
    // A figure of the line before, changed rather than a word of it (bot/memory/resize.py).
    if let Some(resized) = some_str(crate::memory::resize::rewritten(&sget(fixed, "said"), &memory_value(memory))) {
        out.push(resized);
    }
    let mut seen: Vec<String> = Vec::new();
    for line in out {
        if !seen.contains(&line) {
            seen.push(line);
        }
    }
    let mut out = seen;
    if _a_figure(&from) && A_DIGIT.is_match(&value) {
        // A line that still carries the figure the person replaced has not been corrected at all.
        let gone = Re::new(&format!(r"\b{}\b", crate::py::re::escape(&py_str(&from).replace(',', ""))), 0);
        out.retain(|line| !gone.is_match(&line.replace(',', "")));
    }
    out
}

rx!(RELATIVE_MONTH);

/// Python: bot/memory/store_replies.py:4308 `_the_time_that_moved(told, fixed, value)`
///
/// The line they would have said, when what they corrected is the month or the day in it.
pub fn _the_time_that_moved(told: &str, fixed: &Value, value: &str) -> Option<String> {
    use crate::memory::corrected::{_date_said, _date_word, _date_word_case};
    let wrong = text::strip(&sget(fixed, "from")).to_string();
    let relative = if wrong.is_empty() { false } else { RELATIVE_MONTH.fullmatch(&wrong).is_some() };
    if relative {
        // The date taken back was said in relative words, so the month they name now goes in its place.
        let said = _date_said(&sget(fixed, "value").to_lowercase());
        let parts = said.items();
        let day = parts.first().cloned().unwrap_or(Value::None);
        let month = parts.get(1).cloned().unwrap_or(Value::None);
        if !month.truthy() || !says_word_i(&wrong, told) {
            return None;
        }
        let said = py_str(&_date_word_case(&month).val());
        let said = if day.truthy() { format!("on the {} of {}", py_str(&day), said) } else { "in ".to_string() + &said };
        return Some(swap_once(&wrong, &said, told));
    }
    let was = s_or(Some(&_date_word(&wrong).val()));
    let now = s_or(Some(&_date_word(value).val()));
    if was.is_empty() || now.is_empty() || was.to_lowercase() == now.to_lowercase() {
        return None;
    }
    // Only within one family: a month moves for a month and a day for a day.
    if MONTHS.contains(&was.to_lowercase()) != MONTHS.contains(&now.to_lowercase()) {
        return None;
    }
    if !says_word_i(&wrong, told) {
        return None;
    }
    Some(swap_once(&wrong, &now, told))
}

/// Python: bot/memory/store_replies.py:4341 `_a_rate_changed(text, memory)`
///
/// True when the figure this line changed is money put by each month, not a word of a fact.
pub fn _a_rate_changed(text_: &str, memory: &Memory) -> bool {
    let Some(rewritten) = some_str(crate::memory::resize::rewritten(text_, &memory_value(memory))) else { return false };
    if crate::memory::resize::rate(&rewritten).val().is_none() {
        return false;
    }
    PUTTING_BY.is_match(&rewritten)
}

/// Python: bot/memory/store_replies.py:4348 `_answer_the_new_one(memory, fixed)`
///
/// The answer about the thing a correction swapped in, or None when there is nothing to say.
pub fn _answer_the_new_one(memory: &mut Memory, fixed: &Value) -> Option<String> {
    if fixed.is_dict() && fixed.get("quiet").map(Value::truthy).unwrap_or(false) {
        // The note already says the thing in the sentence it was said in.
        return None;
    }
    let lines = _swapped_lines(memory, fixed);
    if lines.is_empty() {
        return None;
    }
    let before = memory.understood.clone();
    let mut answer = None;
    for line in lines.iter().take(4) {
        let reading = crate::understanding::understand(line, Some(memory));
        memory.understood = Some(reading);
        let reply = crate::modules::answers::respond(line, memory);
        let Some(reply) = reply.filter(|r| !text::strip(r).is_empty()) else { continue };
        if crate::memory::again::unframed(&reply).map(|u| !u.is_empty()).unwrap_or(false) {
            // The only answer the new thing has is the paragraph already on the screen.
            answer = None;
            break;
        }
        answer = Some(reply);
        break;
    }
    memory.understood = before;
    answer
}

/// Python: bot/memory/store_replies.py:4386 `_name_fixed_this_turn(text, memory)`
///
/// The line that says a person's name has been moved: "Dana, not Dave - got it."
pub fn _name_fixed_this_turn(text_: &str, memory: &mut Memory) -> Option<String> {
    let fixed = mget(memory, "name_fixed").clone();
    if !fixed.is_dict() || fixed.get("said").map(|s| *s != Value::from(fold(text_))).unwrap_or(true) {
        return None;
    }
    memory.remove("name_fixed");
    let name = sget(&fixed, "name");
    let before = sget(&fixed, "from");
    if name.is_empty() {
        return None;
    }
    if !before.is_empty() && before.to_lowercase() != name.to_lowercase() {
        return Some(format!("{}, not {} \u{2014} got it.", text::upper_first(&name), text::upper_first(&before)));
    }
    Some(format!("{} \u{2014} got it.", text::upper_first(&name)))
}

strs!(_NUMBER_WORDS, _TENS_WORDS);

/// Python: bot/memory/store_replies.py:4410 `_a_number_word(number)`
///
/// A number said in words: 8 -> "eight", 71 -> "seventy-one", 105 -> "one hundred and five".
pub fn _a_number_word(number: i64) -> String {
    if number < 0 {
        return number.to_string();
    }
    let n = number as usize;
    if n < _NUMBER_WORDS.len() {
        return _NUMBER_WORDS[n].clone();
    }
    if n < 100 {
        let (tens, ones) = (n / 10, n % 10);
        return _TENS_WORDS[tens].clone() + &(if ones != 0 { "-".to_string() + &_NUMBER_WORDS[ones] } else { String::new() });
    }
    if n < 1000 {
        let (hundreds, rest) = (n / 100, n % 100);
        let said = _NUMBER_WORDS[hundreds].clone() + " hundred";
        return if rest == 0 { said } else { said + " and " + &_a_number_word(rest as i64) };
    }
    number.to_string()
}

/// `_a_number_word(number)` for any value: Python tries `int(number)` and says `str(number)` when
/// that raises.
fn _a_number_word_v(number: &Value) -> String {
    match py_int_value(number) {
        Some(n) => _a_number_word(n),
        None => py_str(number),
    }
}

/// Python: bot/memory/store_replies.py:8637 `_a_number(value)`
///
/// A figure the fact reader gave as text, as a number: "12" and 12 are the same age.
pub fn _a_number(value: &Value) -> Value {
    match py_int(&py_str(value)) {
        Some(n) => Value::Int(n),
        None => value.clone(),
    }
}

static CORRECTED_WHICH_PERSON: LazyLock<Vec<String>> = LazyLock::new(|| g("CORRECTED_WHICH_PERSON").strs());
static CORRECTED_AGAIN: LazyLock<Vec<String>> = LazyLock::new(|| g("CORRECTED_AGAIN").strs());
static CORRECTED_AGAIN_WHO: LazyLock<Vec<String>> = LazyLock::new(|| g("CORRECTED_AGAIN_WHO").strs());

/// Python: bot/memory/store_replies.py:4430 `_fact_moved_this_turn(text, memory)`
///
/// The correction that moves a fact to another person: "Your daughter, not your son - got it.".
pub fn _fact_moved_this_turn(text_: &str, memory: &mut Memory) -> Option<String> {
    let moved = mget(memory, "fact_moved").clone();
    if !moved.is_dict() || moved.get("said").map(|s| *s != Value::from(fold(text_))).unwrap_or(true) {
        return None;
    }
    memory.remove("fact_moved");
    let who = text::strip(&sget(&moved, "who")).to_string();
    let was_who = text::strip(&sget(&moved, "was_who")).to_string();
    let value = moved.get("value").cloned().unwrap_or(Value::None);
    let said_value = if moved.get("kind").map(|k| *k == Value::from("age")).unwrap_or(false) {
        let number = _a_number(&value);
        if !number.truthy() {
            return None;
        }
        _a_number_word_v(&number)
    } else if let Value::Str(s) = &value {
        s.to_string()
    } else {
        return None;
    };
    if who.is_empty() || was_who.is_empty() || said_value.is_empty() {
        return None;
    }
    Some(fmt_named(
        &pick(&CORRECTED_WHICH_PERSON, memory, text_),
        &[("Value", &text::upper_first(&said_value)), ("value", &said_value), ("who", &who), ("was_who", &was_who)],
    ))
}

/// Python: bot/memory/store_replies.py:4454 `_fact_fixed_this_turn(text, memory)`
///
/// The line that says a fact about someone has been put right: "Eight, not seven - got it.".
pub fn _fact_fixed_this_turn(text_: &str, memory: &mut Memory) -> Option<String> {
    let fixed = mget(memory, "fact_fixed").clone();
    if !fixed.is_dict() || fixed.get("said").map(|s| *s != Value::from(fold(text_))).unwrap_or(true) {
        return None;
    }
    memory.remove("fact_fixed");
    let who = text::strip(&sget(&fixed, "who")).to_string();
    let was = fixed.get("was").cloned().unwrap_or(Value::None);
    let value = fixed.get("value").cloned().unwrap_or(Value::None);
    // An age arrives as a figure in one shape and as a word in the other.
    let (new_number, old_number) = (_a_number(&value), _a_number(&was));
    if fixed.get("kind").map(|k| *k == Value::from("age")).unwrap_or(false) && new_number.truthy() && old_number.truthy() {
        let value_said = _a_number_word_v(&new_number);
        let was_said = _a_number_word_v(&old_number);
        return Some(fmt_named(
            &pick(&CORRECTED_AGAIN, memory, text_),
            &[
                ("Value", &text::upper_first(&value_said)),
                ("was", &was_said),
                ("were", &text::upper_first(&was_said)),
                ("who", &who),
                ("value", &value_said),
            ],
        ));
    }
    if let (Value::Str(v), Value::Str(_)) = (&value, &was) {
        let who_said = if who.is_empty() { "you".to_string() } else { who.clone() };
        return Some(fmt_named(
            &pick(&CORRECTED_AGAIN_WHO, memory, text_),
            &[("Value", &text::upper_first(v)), ("who", &who_said), ("value", v)],
        ));
    }
    None
}

strs!(NOTHING_STARTED);
rx!(
    WHERE_PLAN, SAID_AS_A_SENTENCE, WHEN_ASKS, WHEN_THING, TIME_ASKS, TIME_THING, TIME_IT, DAY_THING, DAY_IT, DAY_ASKS, WHEN_PLAN,
    WHEN_THING_VERB, ASKS_FOR_TASTES, TASTE_ASK, AGE_ASK, WHERE_WORK, SAID_ABOUT, ASKING_AGAIN, PLAN_ASKS,
);
static A_PRONOUN_PERSON: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:she|her|he|him)\b", 0));
static MY_NAME_ASK: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        r"\b(?:what|which)\b.*\bmy name\b|\bwho am i\b|^my name$|\bwhat my name\b|\b(?:know|remember) my name\b|\bwhat did (?:i|u|you) (?:say|tell me) my name was\b",
        0,
    )
});
static MY_JOB_ASK: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\bmy job\b|\bmy work\b|\bwhat do i do\b|\bwhere do i work\b|\bwhat do i work as\b|\bwhat i (?:do|did)\b|\bfor a living\b", 0)
});
static REMEMBER_WHAT_I_DO: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:remember|recall|know)\b[^.]{0,18}\bwhat i (?:do|did)\b\W*$", 0));
static WHAT_I_TOLD_ALONE: LazyLock<Re> = LazyLock::new(|| Re::new(r"^what i (?:told|said)(?: you)?$|^what have i (?:told|said)(?: you)?$", 0));
static ABOUT_ME: LazyLock<Re> = LazyLock::new(|| Re::new(r"\babout me\b|\bof me\b", 0));
static WHAT_I_SAID_MORE: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"\bwhat was it i said\b|^what i (?:said|told you|mentioned)\b|",
            r"^what we (?:said|told you|mentioned)\s+(?:about|on|regarding)\b|",
            r"^anything i (?:said|told you|mentioned)\b|",
            r"\bi (?:cant|cannot|can not|dont|do not) (?:really |quite )?remember ",
            r"what i (?:said|told you|mentioned)\b|",
            r"\b(?:do|can|could|did) (?:you|u) remember what i\b"
        ),
        0,
    )
});
static MY_AGE_ASK: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\bmy age\b|\bhow old am i\b|\bhow old did i say\b|\bhow old (?:i am|i was|we are|i(?:'m| am) gon(?:na|e) to be)\b", 0)
});
static HER_OR_HIM: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:she|her)\b|\b(?:he|him)\b", 0));
static SHE_OR_HER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:she|her)\b", 0));
static ALL_ABOUT_ME: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\bwhat have i told you\b|\bwhat do you (?:know|remember) about me\b|\babout me\b|^what i (?:told|said)(?: you)?$|^what have i (?:told|said)(?: you)?$", 0)
});

/// The `plain` reader `_respond` hands `told` (Python's `lambda asked: _respond(asked, memory)`):
/// a function reference to `_respond`, called as `store_replies::_respond(asked, memory)`.
fn _plain_reader() -> Value {
    Value::Fn(std::sync::Arc::new(data::FnRef { id: "bot.memory.store_replies:_respond".into(), cells: Vec::new(), defaults: Vec::new() }))
}

/// The `claims` reader `_respond` hands `told` (Python's `lambda: bool(_about_this_chat(text,
/// memory))`): a function reference to `_about_this_chat` with the line in its "text" cell.
fn _claims_reader(text_: &str) -> Value {
    Value::Fn(std::sync::Arc::new(data::FnRef {
        id: "bot.memory.store_replies:_about_this_chat".into(),
        cells: vec![("text".into(), Value::from(text_))],
        defaults: Vec::new(),
    }))
}

/// `general or _about_what_they_said(text, memory)`.
fn general_or_about(general: Option<String>, text_: &str, memory: &mut Memory) -> Option<String> {
    match general {
        Some(g) if !g.is_empty() => Some(g),
        _ => _about_what_they_said(text_, memory),
    }
}

/// `a or b` on two reader values (the first when it says something, else the second as it is).
fn or_else_reply(a: Option<String>, b: impl FnOnce() -> Option<String>) -> Option<String> {
    match a {
        Some(s) if !s.is_empty() => Some(s),
        _ => b(),
    }
}

/// Python: bot/memory/store_replies.py:4479 `_respond(text, memory)`
///
/// The store's own answer to the line, read by every recall reader in turn. Python's value is
/// kept exactly: `None`, or a str that may be "".
pub fn _respond(text_: &str, memory: &mut Memory) -> Option<String> {
    // "whats our plan", "what have we decided": the ask is for everything the chat holds.
    let low_here = fold(text_);
    if _summed_up(&low_here) {
        // A chat that holds nothing of theirs has nothing to sum up.
        let any_said = user(memory).pairs().iter().any(|(_, v)| !text::strip(&s_or(Some(v))).is_empty());
        if !any_said {
            return Some(_vary(&NOTHING_STARTED, memory));
        }
        return _summary_reply(memory);
    }
    if let Some(r) = _name_fixed_this_turn(text_, memory) {
        return Some(r);
    }
    if let Some(r) = _fact_fixed_this_turn(text_, memory).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    if let Some(r) = _fact_moved_this_turn(text_, memory).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    if let Some(corrected) = _correction_this_turn(text_, memory).filter(|r| !r.is_empty()) {
        let fixed = mget(memory, "correction").clone();
        let more = _answer_the_new_one(memory, &fixed);
        if let Some(more) = more.filter(|m| !text::strip(m).is_empty()) {
            if _a_rate_changed(text_, memory) {
                // The figure the person changed is what they put by each month.
                return Some(text::strip(&more).to_string());
            }
            return Some(text::rstrip(&corrected).to_string() + " " + text::strip(&more));
        }
        return Some(corrected);
    }
    if let Some(r) = _denial_this_turn(text_, memory).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    if let Some(r) = _the_back_this_turn(text_, memory).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    if let Some(r) = _retraction_this_turn(text_, memory).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    // "my sister lives in perth" then "no, york": the value alone corrects the line before it.
    if let Some(r) = some_str(crate::memory::places::_a_town_moved(memory, text_)) {
        return Some(r);
    }
    // A plan with something still missing is the ask they did not make.
    if let Some(r) = some_str(crate::memory::pending::answer(text_, memory)) {
        return Some(r);
    }
    // The ask for their own family name, first name or both.
    if let Some(r) = some_str(crate::memory::surname::reply_for(text_, memory)) {
        return Some(r);
    }
    if let Some(r) = _the_name_they_picked(text_, memory) {
        return Some(r);
    }
    if let Some(r) = _what_the_pet_is_called(text_, memory) {
        return Some(r);
    }
    if let Some(r) = _the_pick_asked_again(text_, memory) {
        return Some(r);
    }
    let folded = crate::memory::relations::_folded(text_, memory);
    let low = MORE_OF_IT.sub(r"\1", &_bare(&folded));
    if let Some(r) = _a_diet_told(text_, memory, &low).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    // A pregnancy, or a limit on drink, is a fact about the person they live with.
    if let Some(r) = _a_pregnancy_of_their_own(text_, memory, &low).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    if let Some(r) = _a_state_told(text_, memory, &low).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    // The same state, said in the past: "my wife was pregnant" is not news now.
    if let Some(r) = _a_state_that_was(text_, memory, &low).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    if let Some(r) = _a_dislike_told(text_, memory).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    // "what should I call it?" a turn after "we're getting a kitten" asks the bot for a name.
    if let Some(r) = _a_name_for_the_pet(text_, memory, &low).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    // "who's older, me or my wife": two ages the store was given, put side by side.
    if let Some(r) = _age_compare(memory, &low).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    // "how far along is she" a turn after "my wife is 8 weeks pregnant".
    if let Some(r) = _how_far_along(memory, &low).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    // "which one is first": two plans with a date each, put in the order the calendar puts them.
    if let Some(r) = _which_comes_first(memory, &low).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    // "what did i decide to get": the choice a chat that weighed two things made.
    if let Some(r) = some_str(crate::memory::decided::answer(text_, memory)) {
        return Some(r);
    }
    // "who is coming with me": the companion they told the chat about.
    if let Some(r) = some_str(crate::memory::company::answer(text_, memory)) {
        return Some(r);
    }
    // "what's my mum's name", "how old is my son": a fact kept about someone they mentioned.
    if let Some(r) = some_str(crate::memory::relations::_relation_reply(memory, &low)) {
        return Some(r);
    }
    // "have i told you about my sister", "remember when i said my knee hurt": a question about
    // the chat, read by `told` with the store's own readers.
    let facts = _told_readers();
    if let Some(r) = some_str(crate::memory::told::callback(text_, memory, &facts, &_plain_reader(), &_claims_reader(text_))) {
        return Some(r);
    }
    if let Some(r) = some_str(crate::memory::told::answer(text_, memory, &facts, &_plain_reader(), &_claims_reader(text_))) {
        return Some(r);
    }
    if WHERE_PLAN.is_match(&low) {
        // "where are we going again": the plan carries the place.
        if let Some(r) = _plans_reply(memory, &low).filter(|r| !r.is_empty()) {
            return Some(r);
        }
        if let Some(Value::Str(gone)) = user(memory).get("plan_dropped") {
            if !gone.is_empty() {
                // They took it back a turn or two ago, so the honest answer names that.
                if SAID_AS_A_SENTENCE.match_(gone).is_some() {
                    return Some(pick(
                        &[
                            "That plan is off, then \u{2014} where are you going?",
                            "Right, that one is off the record \u{2014} where are you headed?",
                            "Scratch that trip, then \u{2014} where is it now?",
                        ],
                        memory,
                        "",
                    ));
                }
                let shown = text::upper_first(gone);
                return Some(pick(
                    &[
                        shown.clone() + " is off, then \u{2014} where are you going?",
                        "Not ".to_string() + &shown + ", then. Where are you headed?",
                        "Right, ".to_string() + &shown + " is dropped \u{2014} where is it now?",
                    ],
                    memory,
                    "",
                ));
            }
        }
        return None;
    }
    // "when is my mums birthday": the date is kept on the person.
    if let Some(r) = _a_persons_birthday_reply(memory, &low).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    // "how many weeks till that": the count from today to a date they said.
    if let Some(r) = some_str(crate::memory::when::until_answer(text_, memory)) {
        return Some(r);
    }
    // "where did i say we were going": an ask about what they said, answered with the line.
    if let Some(r) = some_str(crate::compose::own_facts::said_back(text_, memory)) {
        return Some(r);
    }
    // "did i already go": which side of today the filed date is on.
    if let Some(r) = some_str(crate::memory::when::already_answer(text_, memory)) {
        return Some(r);
    }
    let said_when = some_str(crate::memory::when::when_answer(text_, memory));
    // "when's my birthday" is the one when ask the plan reader gets wrong.
    let a_birthday_ask = crate::memory::recall::ask(text_, false).as_deref() == Some("birthday")
        || crate::memory::recall::ask(&low, false).as_deref() == Some("birthday");
    let when_shaped = WHEN_ASKS.is_match(&low)
        || WHEN_THING.is_match(&low)
        || TIME_ASKS.is_match(&low)
        || TIME_THING.is_match(&low)
        || TIME_IT.is_match(&low)
        || DAY_THING.is_match(&low)
        || DAY_IT.is_match(&low)
        || DAY_ASKS.is_match(&low)
        || WHEN_PLAN.is_match(&low)
        || WHEN_THING_VERB.is_match(&low);
    if when_shaped && !a_birthday_ask {
        // "when do I start?" is about the plan that names that verb, when there is one.
        if let Some(r) = _the_prep_day(memory, &low).filter(|r| !r.is_empty()) {
            return Some(r);
        }
        if let Some(r) = _plans_reply(memory, &low).filter(|r| !r.is_empty()) {
            return Some(r);
        }
        if let Some(r) = said_when {
            return Some(r);
        }
        if WHEN_ASKS.is_match(&low) || TIME_ASKS.is_match(&low) || WHEN_THING_VERB.is_match(&low) || DAY_ASKS.is_match(&low) {
            // The paragraph the person has just been given may answer it itself.
            if let Some(part) = some_str(crate::memory::plans::follow_up(text_, memory)) {
                return Some(part);
            }
            return None;
        }
    } else if said_when.is_some() && !a_birthday_ask {
        return said_when;
    }
    // A plan announced is the question its plan implies; the announcement is not this module's turn.
    if some_str(crate::memory::plans::answer(text_, memory)).is_some() {
        return None;
    }
    // The turn after that paragraph is a question about one part of the plan.
    if let Some(part) = some_str(crate::memory::plans::follow_up(text_, memory)) {
        return Some(part);
    }
    if folded.is_empty() {
        return None;
    }
    let asked = ret_opt(crate::memory::history::asked(text_, Some(&*memory)));
    match asked.as_deref() {
        Some("first") => return _first_reply(memory),
        Some("first_bot") => return _first_bot_reply(memory),
        Some("said_about") => {
            let thing = crate::memory::history::the_thing_named(text_);
            if let Some(said) = _what_i_said_about(memory, &thing).filter(|s| !s.is_empty()) {
                return Some(said);
            }
        }
        Some("topics") => {
            // A recap ask is answered with the thing we were on.
            if _summary_ask(&low) || TALKED_ABOUT.is_match(&low) {
                return _topic_reply(memory, &low, false);
            }
            return _topics_reply(memory);
        }
        _ => {}
    }
    if let Some(r) = some_str(crate::memory::choice::reply_for(text_, memory)) {
        return Some(r);
    }
    // "is that enough" is about the amount they just gave.
    if let Some(r) = some_str(crate::memory::enough::enough_reply(text_, memory)) {
        return Some(r);
    }
    // They turn the offer down ("nah not that one"): another of the things the bot named.
    if let Some(r) = some_str(crate::memory::reject::reply_for(text_, memory)) {
        return Some(r);
    }
    // "what else can i try?" after an answer asks for the other half of that answer.
    if let Some(r) = some_str(crate::memory::another::other_option(text_, memory)) {
        return Some(r);
    }
    // Going back to something said earlier.
    if let Some(r) = _return_reply(text_, memory).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    // "do you remember that i live in leeds".
    if let Some(r) = _remembers_fact(text_, memory).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    // "do i eat meat" after "im vegetarian": the rule they gave the store.
    if let Some(r) = _a_rule_on_eating(text_, memory).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    // "am i vegetarian?", "is my name sam?".
    if let Some(r) = _confirmation(text_, memory).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    // "do i like fish?" is a question about a taste they have already told the bot.
    if ASKS_FOR_TASTES.is_match(&low) {
        if let Some(r) = _tastes_said(memory).filter(|r| !r.is_empty()) {
            return Some(r);
        }
    }
    if TASTE_ASK.is_match(&low) {
        if let Some(r) = _taste_reply(memory, &low).filter(|r| !r.is_empty()) {
            return Some(r);
        }
    }
    // "how much have i spent": the sum of the sums they told you.
    let spend = some_str(crate::memory::spent::answer(text_, memory)).or_else(|| some_str(crate::memory::spent::answer(&folded, memory)));
    if let Some(r) = spend {
        return Some(r);
    }
    let first_general = _general_recall(text_, memory);
    let general = or_else_reply(first_general, || _general_recall(&folded, memory));
    // "what does my mum eat": a question about someone the user told us about.
    if _asks_about_someone(&low, text_, memory) {
        let mut told: Option<String> = None;
        if _other_person_reply(memory, &low).map(|r| !r.is_empty()).unwrap_or(false) {
            let ask = _the_ask_after_a_statement(&low, memory);
            told = _other_person_reply(memory, &ask);
        }
        // A miss is not a denial when the line brings a fact with it.
        let telling = _after_the_yes_no_frame(text_);
        if let Some(told) = told.filter(|t| !t.is_empty()) {
            if !(NO_RECORD.is_match(&told) && !telling.is_empty() && _says_more_than_the_ask(&telling)) {
                return Some(told);
            }
        }
    }
    // The line points at someone the chat never named: the age is read back off the line that gave it.
    let mut who_asked = crate::memory::relations::_named_person(&low, Some(&mut *memory));
    if who_asked.is_empty() && A_PRONOUN_PERSON.is_match(&low) {
        who_asked = crate::memory::relations::_person_from_pronoun(&low, memory, &[]);
    }
    if who_asked.is_empty() && AGE_ASK.is_match(&low) {
        if let Some(r) = _a_pronoun_age(memory, &low).filter(|r| !r.is_empty()) {
            return Some(r);
        }
    }
    // "how old is my son": the number was given with the person.
    if let Some(r) = some_str(crate::memory::relations::_ages_reply(memory, &low, &folded)) {
        return Some(r);
    }
    // What is in their own kitchen is not a fact about the world and not one they have told us.
    if let Some(place) = _their_place(&low).filter(|p| !p.is_empty()) {
        let said = pick(
            &[
                "I can\u{2019}t see inside your ".to_string() + &place + ", so tell me what\u{2019}s in there and I\u{2019}ll tell you what it will make",
                "I\u{2019}ve no way of knowing what\u{2019}s in your ".to_string() + &place + ": say what\u{2019}s in it and I\u{2019}ll find you something to cook",
                "Nothing in your ".to_string() + &place + " I can see from here. Tell me what\u{2019}s in it and I\u{2019}ll work something out",
            ],
            memory,
            "",
        );
        return Some(text::rstrip_chars(&said, ".").to_string() + ".");
    }
    // "whats my sisters name" is the same kind of ask as "how old is my son".
    if let Some(r) = _kids_reply(memory, &low).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    if let Some(r) = _a_relations_name_reply(memory, &low, text_).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    // "what car do i drive": the make they gave with the thing.
    if let Some(r) = _a_thing_i_own_reply(memory, &low).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    // "what do you know about my mum": the whole person the chat has been told about.
    if let Some(r) = _about_a_person_reply(memory, &low, text_).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    // "whos meg": the name is the subject and the relation is the answer.
    if let Some(r) = _who_is_the_name_reply(memory, &low).filter(|r| !r.is_empty()) {
        return Some(r);
    }
    // "and where does she live": answered from the chat.
    if !crate::memory::places::_asked_person(memory, &low).is_empty() && !(PLACE_ADVICE.is_match(&low) && !PLACE_RECALLED.is_match(&low)) {
        return _places_reply(memory, &low);
    }
    // "my son isnt" after a limit was given is that limit refused for him.
    if let Some(r) = _not_the_same_reply(memory, &low) {
        return Some(r);
    }
    if !_about_this_chat(text_, memory) && !_a_why_line_they_have_answered(text_, memory) {
        // A question about them that the chat-shaped readers do not claim is answered from their line.
        return general_or_about(general, text_, memory);
    }
    if MY_NAME_ASK.is_match(&low) {
        return Some(_name_reply(memory, &low));
    }
    if MY_JOB_ASK.is_match(&low) || (WHERE_WORK.is_match(&low) && !_doing_again(text_, memory)) {
        // "what was i saying about my job" asks for the line, not for the role.
        if _doing_again(text_, memory) || (SAID_ABOUT.is_match(&low) && _names_its_thing(&low)) {
            let asked = _topic_reply(memory, &low, false);
            if let Some(asked) = asked.filter(|a| !a.is_empty()) {
                if !_a_denial(&asked) {
                    return Some(asked);
                }
            }
        }
        // "and what do i do there": a part of the job they never described.
        if let Some(part) = _a_part_of_the_job_never_said(memory, &low) {
            return Some(part);
        }
        let r#where = WHERE_WORK.is_match(&low);
        return Some(_job_reply(memory, r#where));
    }
    // "do you remember what I do?" asks for the job in the store's own words.
    if REMEMBER_WHAT_I_DO.is_match(&low) {
        return Some(_job_reply(memory, false));
    }
    if PLACES_ASK.is_match(&low) && SELF_WORD.is_match(&low) && !(PLACE_ADVICE.is_match(&low) && !PLACE_RECALLED.is_match(&low)) {
        return _places_reply(memory, &low);
    }
    // "what cant i have" names the limit itself.
    if crate::memory::recall::ask(text_, false).as_deref() == Some("limit") {
        if let Some(limit) = _limit_reply(memory, &low).filter(|l| !l.is_empty()) {
            return Some(limit);
        }
    }
    // A how-to in front of "my diet" is advice, not a recall.
    if _asks_for_a_fact(&low, text_, memory) || (DIET_ASK.is_match(&low) && !DIET_HOW_TO.is_match(&low)) || _a_count_ask(&low).is_some() {
        return _fact_reply(memory, &low, None);
    }
    if PLAN_MINE.is_match(&low) {
        return _plans_reply(memory, &low);
    }
    if FIRST_ASK.is_match(&low) {
        return _first_reply(memory);
    }
    if _repeat_ask_m(text_, &folded, None) {
        return _repeat_reply(memory, text_);
    }
    if SAID_ASK.is_match(&low) {
        return _said_reply(memory);
    }
    // "remind me what i told you" with nothing named is the whole chat, not one line of it.
    if WHAT_I_TOLD_ALONE.is_match(&low) {
        return _summary_reply(memory);
    }
    // "what did i tell you about me" asks for the whole picture.
    if ABOUT_ME.is_match(&low) {
        return _summary_reply(memory);
    }
    // The ask names the kind of detail, and the line their own words find is the answer.
    let kind_said = _an_open_attribute_ask(&low, memory);
    if !kind_said.is_empty() {
        return Some(kind_said);
    }
    if WHAT_I_SAID.is_match(&low)
        || WHAT_I_SAID_MORE.is_match(&low)
        || THING_AGAIN.match_(&low).is_some()
        || some_str(crate::memory::reminded::the_thing(text_, memory)).is_some()
        || _which_thing_was_it(&low, memory).is_some()
        || _an_attribute_they_gave(&low)
    {
        if let Some(said) = _recall_reply(memory, &low, text_, false).filter(|s| !s.is_empty()) {
            return Some(said);
        }
        // A place in the ask that the chat has never been on is what the ask is about.
        let mine = text::squash(text_).to_lowercase();
        let earlier: Vec<String> = recent_turns(memory, 4).iter().map(|t| t.user.clone()).filter(|line| text::squash(line).to_lowercase() != mine).collect();
        if !earlier.is_empty() && _a_place_new_to_the_chat(memory, &low) {
            return general_or_about(general, text_, memory);
        }
    }
    if MY_AGE_ASK.is_match(&low) {
        // "how old did i say she was" asks about her, and the asker's own age is not the answer.
        if HER_OR_HIM.is_match(&low) && crate::memory::relations::_person_from_pronoun(&low, memory, &[]).is_empty() {
            return Some(if SHE_OR_HER.is_match(&low) {
                "You haven't told me how old she is.".to_string()
            } else {
                "You haven't told me how old he is.".to_string()
            });
        }
        return Some(_age_reply(memory));
    }
    // A part of an animal they have already described, asked about with a pronoun.
    if let Some(pet) = _pet_part_reply(memory, &low).filter(|p| !p.is_empty()) {
        return Some(pet);
    }
    if ASKED_ALL.is_match(&low) {
        return Some(_asked_reply(memory));
    }
    if !_about_the_bot_here(memory) && (THE_CHAT_ASK.is_match(&low) || TOPIC_ASK.is_match(&low) || ACTIVITY_ASK.is_match(&low) || TALKED_ABOUT.is_match(&low)) {
        // "what am i planning" is a question about the plans we hold when we hold one.
        if ACTIVITY_ASK.is_match(&low)
            && (PLAN_WORDS.is_match(&low) || _plan_was_corrected(memory) || (UP_TO_ASK.is_match(&low) && _the_line_before_was_a_plan(memory)))
        {
            if let Some(asked) = _plans_reply(memory, &low).filter(|a| !a.is_empty()) {
                return Some(asked);
            }
        } else {
            let again = ASKING_AGAIN.is_match(&folded);
            if let Some(asked) = _topic_reply(memory, &low, again).filter(|a| !a.is_empty()) {
                return Some(asked);
            }
        }
    }
    if _summary_ask(&low) {
        // "summarise our chat" asks about the conversation the same way "recap" does.
        return _topic_reply(memory, &low, false);
    }
    if ALL_ABOUT_ME.is_match(&low) {
        return _summary_reply(memory);
    }
    if PLAN_ASKS.is_match(&low) {
        if let Some(asked) = _plans_reply(memory, &low).filter(|a| !a.is_empty()) {
            return Some(asked);
        }
    }
    if low.contains("remember") && low.contains("yesterday") {
        return _topic_reply(memory, &low, false);
    }
    general_or_about(general, text_, memory)
}

strset!(PLACES);
static FACTS_COUNTRIES: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.COUNTRIES"));
static PLACE_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z][a-z'\-]+", 0));

/// Python: bot/memory/store_replies.py:5073 `_a_place_new_to_the_chat(memory, low)`
///
/// True when the line names a place nothing said so far named.
pub fn _a_place_new_to_the_chat(memory: &mut Memory, low: &str) -> bool {
    let named: HashSet<String> = PLACE_WORDS.findall_str(&text::squash(low)).into_iter().map(|w| w.to_lowercase()).collect();
    let places: Vec<&String> = named.iter().filter(|w| PLACES.contains(*w) || FACTS_COUNTRIES.contains(*w)).collect();
    if places.is_empty() {
        return false;
    }
    let said = recent_turns(memory, 12).iter().map(|t| fold(&t.user)).collect::<Vec<_>>().join(" ");
    !places.iter().any(|p| said.contains(p.as_str()))
}

rx!(OWN_ASK);
strset!(ASK_FILLER, NO_THING);
static THREE_LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b[a-z]{3}\b", 0));

/// Python: bot/memory/store_replies.py:5095 `_about_what_they_said(text, memory)`
///
/// A yes-or-no ask the store has no fact for, answered from their own earlier line.
pub fn _about_what_they_said(text_: &str, memory: &mut Memory) -> Option<String> {
    use crate::memory::topics::stem;
    let low = text::strip_chars(&fold(text_), " ?!.,").to_string();
    if OWN_ASK.match_(&low).is_none() || text::split(&low).len() > 9 {
        return None;
    }
    let stems: HashSet<String> = LETTER_WORDS.findall_str(&low).iter().map(|w| stem(w)).collect();
    let mut words: HashSet<String> =
        stems.into_iter().filter(|w| text::len(w) > 3 && !ASK_FILLER.contains(w) && !NO_THING.contains(w)).collect();
    // A thing named in three letters is still the thing: "car", "job", "dog", "gas".
    words.extend(THREE_LETTERS.findall_str(&low).into_iter().filter(|w| !NO_THING.contains(w) && !ASK_FILLER.contains(w)));
    if words.is_empty() {
        return None;
    }
    let said_before: Vec<String> = memory.turns.iter().map(|t| t.user.clone()).collect();
    let newest = text::strip_chars(&fold(said_before.last().map(String::as_str).unwrap_or("")), " ?!.,").to_string();
    for line in said_before.iter().rev() {
        let said: HashSet<String> = LETTER_WORDS.findall_str(&line.to_lowercase()).iter().map(|w| stem(w)).collect();
        if !words.iter().all(|w| said.contains(w)) {
            continue;
        }
        // The newest line about the thing is the one read; an older one goes back to them as what
        // they said earlier, with the door open.
        let phrase = _told_phrase(line);
        if !phrase.is_empty() {
            let phrase = phrase.replace("can not", "cannot");
            let flat = text::strip_chars(&fold(line), " ?!.,").to_string();
            if flat != newest {
                return Some(pick(
                    &[
                        "Earlier you said ".to_string() + &phrase + " - still the case?",
                        "You said ".to_string() + &phrase + ", earlier in this chat - still the case?",
                    ],
                    memory,
                    "",
                ));
            }
            return Some(pick(&["You told me ".to_string() + &phrase + ".", "You said ".to_string() + &phrase + "."], memory, ""));
        }
        return None;
    }
    None
}

rx!(AVOID_ADVICE, BARE_WHAT_I_DO, AVOID_REST, EATING, ALLERGY_SAID);
strset!(JOB_LINE_WORDS);

/// Python: bot/memory/store_replies.py:5175 `_the_job_was_just_said(memory)`
///
/// True when the line before the ask is where the job was given, and is nothing but the job.
pub fn _the_job_was_just_said(memory: &mut Memory) -> bool {
    let job = text::strip(&sget(&user(memory), "job")).to_lowercase();
    if job.is_empty() {
        return false;
    }
    let turns = memory.turns.clone();
    for turn in turns.iter().rev() {
        let said = text::strip(&turn.user).to_string();
        if said.is_empty() {
            continue;
        }
        if BARE_WHAT_I_DO.match_(&_bare(&said)).is_some() {
            continue; // the ask again: it answers the line before it too
        }
        return said.to_lowercase().contains(&job) && _nothing_but_the_job(&said, &job, memory);
    }
    false
}

/// Python: bot/memory/store_replies.py:5207 `_nothing_but_the_job(said, job, memory)`
///
/// True when the job is all the line said, once the words around it are set aside.
pub fn _nothing_but_the_job(said: &str, job: &str, memory: &mut Memory) -> bool {
    let mut rest = said.to_lowercase().replace(job, " ");
    let frame = text::strip(&sget(&user(memory), "job_frame")).to_lowercase();
    if !frame.is_empty() {
        rest = rest.replace(&frame, " ");
    }
    !LETTER_WORDS.findall_str(&rest).iter().any(|w| text::len(w) > 2 && !JOB_LINE_WORDS.contains(w))
}

/// Python: bot/memory/store_replies.py:5229 `_the_allergy_is_what_is_asked(text, memory)`
///
/// True when an ask for advice about what to avoid is a question about the allergy.
pub fn _the_allergy_is_what_is_asked(text_: &str, memory: &Memory) -> bool {
    let tail = match AVOID_REST.search(text_) {
        Some(rest) => text::squash(rest.str(1)),
        None => String::new(),
    };
    if !tail.is_empty() && !EATING.is_match(&tail) {
        return false;
    }
    let said = text::squash(text_);
    let before = last_turn_user(memory);
    if ALLERGY_SAID.is_match(&said) || ALLERGY_SAID.is_match(&before) {
        return true;
    }
    before.is_empty()
}

/// Python: bot/memory/store_replies.py:5229 `_general_recall(text, memory)`
///
/// A recall ask in any words, answered with the reply the store already gives for that fact.
/// Anything else is a real request: None.
pub fn _general_recall(text_: &str, memory: &mut Memory) -> Option<String> {
    let low = text::squash(&text_.to_lowercase());
    // "what should i avoid": the allergy they told you, when that is what they are asking about.
    if AVOID_ADVICE.is_match(text_) && user(memory).get("allergies").map(Value::truthy).unwrap_or(false) && _the_allergy_is_what_is_asked(text_, memory) {
        let limit = _limit_reply(memory, &low);
        return or_else_reply(limit, || _fact_reply(memory, &low, Some("allergy")));
    }
    // "i work as a nurse" then "what do i do": the job they have just told you.
    if BARE_WHAT_I_DO.match_(&low).is_some() && user(memory).get("job").map(Value::truthy).unwrap_or(false) && _the_job_was_just_said(memory) {
        return Some(_job_reply(memory, false));
    }
    // "how much have i spent": the sum is kept from the lines they typed.
    if let Some(r) = some_str(crate::memory::spent::answer(text_, memory)) {
        return Some(r);
    }
    // "how much have i got left": the same sums, against the money they said they had.
    if let Some(r) = some_str(crate::memory::spent::what_is_left(text_, memory)) {
        return Some(r);
    }
    let key = crate::memory::recall::ask(text_, false).unwrap_or_default();
    if key.is_empty() {
        return None;
    }
    if SAID_ABOUT.is_match(&low) && _names_its_thing(&low) && !_the_fact_is_kept(&key, memory) {
        // "what was i saying about my job" asks for the line they said, not for the job as a fact.
        return None;
    }
    match key.as_str() {
        "name" => Some(_name_reply(memory, &low)),
        "job" => {
            // A part of the job they never described: the job said back answers nothing.
            if let Some(part) = _a_part_of_the_job_never_said(memory, &low) {
                return Some(part);
            }
            let r#where = WHERE_WORK.is_match(&low);
            Some(_job_reply(memory, r#where))
        }
        "city" => _places_reply(memory, &low),
        "birthday" | "weight" | "height" => _fact_reply(memory, &low, Some(&key)),
        "diet" => _diet_reply(memory, &low),
        "allergy" | "budget" | "pet" | "car" => _fact_reply(memory, &low, Some(&key)),
        "limit" => _limit_reply(memory, &low),
        "me" => _summary_reply(memory),
        _ => None,
    }
}

rx!(NAME_ASK, POINTING);
strs!(CAT_NAMES, DOG_NAMES, SMALL_NAMES, NAME_ADVICE, RESUME);
strset!(POINTING_ANY);

/// Python: bot/memory/store_replies.py:5311 `_pet_names(family)`
pub fn _pet_names(family: &str) -> &'static [String] {
    match family {
        "cat" => &CAT_NAMES,
        "rabbit" | "hamster" | "fish" | "parrot" => &SMALL_NAMES,
        _ => &DOG_NAMES,
    }
}

/// Python: bot/memory/store_replies.py:5325 `_points_at_something_else(low)`
///
/// True when the line points at a thing that is not an animal ("this song", "my new business").
pub fn _points_at_something_else(low: &str) -> bool {
    POINTING.findall_str(low).iter().any(|w| !FACTS_ANIMALS.contains(w) && !POINTING_ANY.contains(w))
}

/// Python: bot/memory/store_replies.py:5339 `_the_pet_in_play(low, memory)`
///
/// The animal the naming question is about: the one in the line, or the one just mentioned.
pub fn _the_pet_in_play(low: &str, memory: &mut Memory) -> Option<String> {
    use crate::memory::relations::_animal_family;
    if let Some(w) = LETTER_WORDS.findall_str(low).into_iter().find(|w| FACTS_ANIMALS.contains(w)) {
        return Some(_animal_family(&w));
    }
    for turn in recent_turns(memory, 3).iter().rev() {
        let line = turn.user.to_lowercase();
        if let Some(w) = LETTER_WORDS.findall_str(&line).into_iter().find(|w| FACTS_ANIMALS.contains(w)) {
            return Some(_animal_family(&w));
        }
    }
    let pets = user(memory).get("pets").map(py_list).unwrap_or_default();
    if let Some(first) = pets.first() {
        if first.is_dict() && first.get("animal").map(Value::truthy).unwrap_or(false) {
            return Some(_animal_family(&py_str(first.get("animal").unwrap())));
        }
    }
    None
}

static POINTS_AT_THE_PET: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:it|him|her|them|this one|that one)\b", 0));

/// Python: bot/memory/store_replies.py:5355 `_a_pet_of_theirs_is_named(low, family, memory)`
///
/// True when the animal the naming ask points at already carries a name in the store.
pub fn _a_pet_of_theirs_is_named(low: &str, family: &str, memory: &mut Memory) -> bool {
    use crate::memory::relations::_animal_family;
    if !POINTS_AT_THE_PET.is_match(low) {
        return false;
    }
    let pets = user(memory).get("pets").map(py_list).unwrap_or_default();
    pets.iter().any(|pet| {
        pet.is_dict() && pet.get("name").map(Value::truthy).unwrap_or(false) && _animal_family(&s_or(pet.get("animal"))) == _animal_family(family)
    })
}

/// Python: bot/memory/store_replies.py:5365 `_a_name_for_the_pet(text, memory, low)`
///
/// A short list of names for the pet in play, or None when the line is not about naming one.
pub fn _a_name_for_the_pet(text_: &str, memory: &mut Memory, low: &str) -> Option<String> {
    if !NAME_ASK.is_match(low) {
        return None;
    }
    let family = _the_pet_in_play(low, memory)?;
    if family.is_empty() || family == "pet" {
        return None;
    }
    if _points_at_something_else(low) {
        return None;
    }
    if _a_pet_of_theirs_is_named(low, &family, memory) {
        // The pet in play has a name, and the store answers with it.
        return None;
    }
    let pool: Vec<String> = _pet_names(&family).to_vec();
    // A different three each time, so asking twice does not get the same list back.
    let seed = memory.turns.len() as i128 * 7919 + crate::py::random::ord_sum(&text_.to_lowercase());
    let shuffled = crate::py::random::Random::new(seed).sample(&pool, 3);
    let advice = &NAME_ADVICE[(seed.rem_euclid(NAME_ADVICE.len() as i128)) as usize];
    let animal = crate::memory::relations::_a(&family);
    Some("For ".to_string() + &animal + ": " + &shuffled[..shuffled.len() - 1].join(", ") + " or " + &shuffled[shuffled.len() - 1] + ". " + advice)
}

/// Python `int(v or default)` for a count in a record (raises on a bad value, as Python does).
fn count_of(v: Option<&Value>, default: i64) -> i64 {
    match v {
        Some(v) if v.truthy() => py_int_value(v).expect("ValueError: invalid literal for int()"),
        _ => default,
    }
}

/// Python: bot/memory/store_replies.py:5387 `_count_of_a_kind(pets)`
///
/// How many animals are in this group, when one of them says how many there are.
pub fn _count_of_a_kind(pets: &[Value]) -> i64 {
    let said: Vec<i64> = pets.iter().map(|p| count_of(p.get("count"), 0)).filter(|c| *c > 1).collect();
    if let Some(most) = said.iter().max() {
        if *most >= pets.len() as i64 {
            return *most;
        }
    }
    pets.iter().map(|p| count_of(p.get("count"), 1)).sum()
}

/// Python: bot/memory/store_replies.py:5400 `_return_reply(text, memory)`
///
/// Greet a return to an earlier topic: "ok back to the boiler", "as I was saying, Rome".
pub fn _return_reply(text_: &str, memory: &mut Memory) -> Option<String> {
    let record = crate::memory::topics::resumed(memory).val();
    if !record.truthy() || record.get("asks").map(Value::truthy).unwrap_or(false) {
        return None;
    }
    let thing = text::strip(&s_or(record.get("asked"))).to_string();
    if thing.is_empty() || text::split(&thing).len() > 4 {
        return None;
    }
    let said = _same_case(&thing, text_);
    let lines: Vec<String> = RESUME.iter().map(|line| fmt_named(line, &[("thing", &said)])).collect();
    Some(pick(&lines, memory, ""))
}

/// Python: bot/memory/store_replies.py:5414 `_same_case(thing, text)`
///
/// Keep a name a name: someone typing "as I was saying, Rome" gets "Back to Rome".
pub fn _same_case(thing: &str, text_: &str) -> String {
    for word in text::split(text_) {
        let first_upper = word.chars().next().map(|c| crate::understanding::pyuni::ch_upper(c)).unwrap_or(false);
        if text::strip_chars(&word, " ,.!?").to_lowercase() == thing.to_lowercase() && first_upper {
            return text::upper_first(thing);
        }
    }
    thing.to_string()
}

static PLAN_DOING: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bplan(?:ning|s)?\b|\bdoing\b|\bup to\b", 0));

/// Python: bot/memory/store_replies.py:5423 `_name_reply(memory, low="")`
pub fn _name_reply(memory: &mut Memory, low: &str) -> String {
    let who = ret_str(crate::memory::name(memory));
    let plan = s_or(user(memory).get("plan"));
    if !who.is_empty() && !plan.is_empty() && PLAN_DOING.is_match(low) {
        let rest = text::slice(&plan, 1, None);
        let rest_upper = rest.chars().next().map(|c| crate::understanding::pyuni::ch_upper(c)).unwrap_or(false);
        let clause = if !rest_upper { lower_first(&plan) } else { plan.clone() };
        return "You're ".to_string() + &who + ", and you're planning " + text::rstrip_chars(&clause, ".") + ".";
    }
    if who.is_empty() {
        // The store says which fact it asked for, so the answer is read as that fact.
        memory.set("asked_for", "name");
        return pick(&["You haven't told me your name yet.", "I don't know it yet \u{2014} you haven't said."], memory, "");
    }
    pick(&["You're ".to_string() + &who + ".", who.clone() + " \u{2014} you told me.", "Your name's ".to_string() + &who + "."], memory, "")
}

rx!(WEEKDAY_WORD, DAY_OF_MONTH, AT_TIME);

/// Python: bot/memory/store_replies.py:5443 `_the_day_of(plan)`
///
/// The day the plan falls on, as somebody would answer "what day?": "on friday" -> "On Friday".
pub fn _the_day_of(plan: &str) -> String {
    let said = plan.to_lowercase();
    if let Some(found) = WEEKDAY_WORD.search(&said) {
        return "On ".to_string() + &text::capitalize(found.as_str()) + ".";
    }
    if let Some(number) = DAY_OF_MONTH.search(&said) {
        return "On the ".to_string() + number.str(1) + ".";
    }
    String::new()
}

/// Python: bot/memory/store_replies.py:5459 `_at_the_clock(plan)`
///
/// The hour the plan says, as somebody would answer "what time?": "at 6" -> "At 6".
pub fn _at_the_clock(plan: &str) -> String {
    let Some(found) = AT_TIME.search(plan) else { return String::new() };
    let said = text::squash(found.as_str());
    if said.to_lowercase().starts_with("at ") {
        "At ".to_string() + &text::slice(&said, 3, None) + "."
    } else {
        String::new()
    }
}

rx!(POINTS_BACK_AT_IT, SETTING_OUT, STUDY_PLAN, MONTH_WORD, DATE_PHRASE);
strs!(WEEK);
static TAIL_WINDOW: LazyLock<i64> = LazyLock::new(|| g("TAIL_WINDOW").as_int().unwrap());

/// Python: bot/memory/store_replies.py:5481 `_the_day_before(plan)`
///
/// The weekday the plan is on and the one before it, or None when it names no weekday.
pub fn _the_day_before(plan: &str) -> Option<(String, String)> {
    let low = fold(plan);
    let found: Vec<&String> = WEEK.iter().filter(|day| Re::new(&format!(r"\b{day}\b"), 0).is_match(&low)).collect();
    if found.len() != 1 {
        return None;
    }
    if MONTH_WORD.is_match(&low) || DATE_PHRASE.is_match(&low) {
        return None;
    }
    let day = found[0].clone();
    let at = WEEK.iter().position(|d| *d == day).unwrap();
    let before = WEEK[(at + WEEK.len() - 1) % WEEK.len()].clone();
    Some((day, before))
}

/// The plans the store keeps as strs, with the latest plan last when it is not among them.
fn _plans_with_the_latest(known: &Value) -> Vec<String> {
    let mut plans: Vec<String> =
        known.get("plans").map(py_list).unwrap_or_default().iter().filter_map(|p| p.as_str()).filter(|p| !text::strip(p).is_empty()).map(str::to_string).collect();
    if let Some(Value::Str(latest)) = known.get("plan") {
        if !text::strip(latest).is_empty() && !plans.iter().any(|p| **p == **latest) {
            plans.push(latest.to_string());
        }
    }
    plans
}

/// Python: bot/memory/store_replies.py:5494 `_the_prep_day(memory, low)`
///
/// The plan and the day the work for it goes on, or None when the store cannot say.
/// (Contract change: the generated stub returned `()`.)
pub fn _the_prep_day(memory: &mut Memory, low: &str) -> Option<String> {
    if !POINTS_BACK_AT_IT.is_match(low) || !_a_plan_answers(memory, low) {
        return None;
    }
    let back = POINTS_BACK_AT_IT.search(low).unwrap();
    let at = text::char_index(low, back.start()) as isize;
    let window = text::slice(low, (at - *TAIL_WINDOW as isize).max(0), Some(at));
    if SETTING_OUT.is_match(&window) {
        return None; // "when is the best time to leave for it" is about setting out
    }
    let known = user(memory);
    let plans = _plans_with_the_latest(&known);
    for plan in plans.iter().rev() {
        if STUDY_PLAN.is_match(plan) {
            continue; // studying for an exam has an answer of its own
        }
        let Some((day, before)) = _the_day_before(plan) else { continue };
        let line = crate::memory::plan_line(memory, plan);
        let said = text::rstrip_chars(&crate::memory::plan_sentence(plan, &line), ".").to_string();
        return Some(said + ", so " + &text::title(&before) + " is the day for the prep \u{2014} keep " + &day + " itself for the last bits.");
    }
    None
}

strs!(NO_PLANS_REPLY);

/// Python: bot/memory/store_replies.py:5528 `_a_plan_answers(memory, low)`
///
/// True when a when or time ask that names nothing of its own is answered by a plan.
pub fn _a_plan_answers(memory: &mut Memory, low: &str) -> bool {
    for pattern in [&*WHEN_ASKS, &*WHEN_THING, &*TIME_ASKS, &*TIME_THING] {
        let Some(found) = pattern.search(low) else { continue };
        if _names_something_of_its_own(low, text::char_index(low, found.end()) as i64) {
            return false;
        }
        let reply = _plans_reply(memory, low);
        return match reply {
            Some(r) if !r.is_empty() => !NO_PLANS_REPLY.contains(&r),
            _ => false,
        };
    }
    false
}

rx!(THAT_DATE, CALENDAR_SAID);

/// Python: bot/memory/store_replies.py:5546 `_the_date_behind_that(memory, low, plans)`
///
/// "And what date is that" after the calendar has been read out is a date, not the plan.
pub fn _the_date_behind_that(memory: &mut Memory, low: &str, plans: &[String]) -> Option<String> {
    if !THAT_DATE.is_match(low) || !CALENDAR_SAID.is_match(&last_turn_bot(memory)) {
        return None;
    }
    for plan in plans.iter().rev() {
        if let Some(day) = WEEKDAY_WORD.search(&plan.to_lowercase()) {
            let ask = "what date is ".to_string() + day.as_str();
            if let Some(worked) = crate::knowledge::maths::solve(&ask, &Understood::default(), memory).filter(|w| !w.is_empty()) {
                return Some(worked);
            }
        }
    }
    None
}

static _A_DATE_AT_THE_END: LazyLock<&'static Re> = LazyLock::new(|| g("_A_DATE_AT_THE_END").as_re().unwrap());

/// Python: bot/memory/store_replies.py:5569 `_the_whole_word_in(text, word)`
///
/// Where the word sits in the line as a word of its own (a character index), or -1.
pub fn _the_whole_word_in(text_: &str, word: &str) -> i64 {
    if word.is_empty() {
        return -1;
    }
    match Re::new(&format!(r"(?<![\w']){}(?![\w'])", crate::py::re::escape(word)), 0).search(text_) {
        Some(found) => text::char_index(text_, found.start()) as i64,
        None => -1,
    }
}

/// Python: bot/memory/store_replies.py:5585 `_third_person(word)`
///
/// The form a verb takes after "he": "start" -> "starts", "go" -> "goes", "try" -> "tries".
pub fn _third_person(word: &str) -> String {
    let word = text::strip_chars(word, ".,!?;:").to_string();
    if word.is_empty() || !crate::understanding::pyuni::is_alpha(&word) {
        return word;
    }
    if text::ends_with_any(&word, &["s", "x", "z", "ch", "sh"]) {
        return word + "es";
    }
    let chars: Vec<char> = word.chars().collect();
    let before_last = if chars.len() > 1 { Some(chars[chars.len() - 2]) } else { None };
    let a_consonant = before_last.map(|c| !"aeiou".contains(c)).unwrap_or(false);
    if word.ends_with('o') && chars.len() > 1 && a_consonant {
        // "go" is "goes" and "do" is "does", not "gos" and "dos".
        return word + "es";
    }
    if word.ends_with('y') && chars.len() > 1 && a_consonant {
        return chars[..chars.len() - 1].iter().collect::<String>() + "ies";
    }
    word + "s"
}

/// Python: bot/memory/store_replies.py:5601 `_the_line_that_said(memory, plan)`
///
/// The recent line that carries the plan, for a plan the log kept as one of several.
pub fn _the_line_that_said(memory: &Memory, plan: &str) -> Option<String> {
    let wanted = fold(plan);
    if wanted.is_empty() {
        return None;
    }
    recent_turns(memory, 6).iter().rev().map(|t| t.user.clone()).find(|line| fold(line).contains(&wanted))
}

static AND_OR_COMMA: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+and\s+|,|;", 0));

/// Python: bot/memory/store_replies.py:5614 `_the_verb_from_the_line(plan, line)`
///
/// The words the other half of the line carries, for a half that was left without a verb.
pub fn _the_verb_from_the_line(plan: &str, line: &str) -> String {
    let wanted = fold(plan);
    let text_ = text::squash(line);
    if wanted.is_empty() || text_.is_empty() {
        return String::new();
    }
    for piece in AND_OR_COMMA.split(&text_) {
        let clause = text::strip_chars(&text::squash(&piece), " ,.;:-!").to_string();
        if clause.is_empty() || fold(&clause) == wanted {
            continue;
        }
        let Some(m) = _A_DATE_AT_THE_END.match_(&clause) else { continue };
        // The other half is "my niece turns 5": the determiner and the person, then the words that
        // are left are what the half this ask names was never given ("turns 5").
        let before = text::split(m.name_str("subject"));
        if before.len() < 3 || !["my", "our", "the", "his", "her", "their"].contains(&before[0].to_lowercase().as_str()) {
            continue;
        }
        let body = text::strip_chars(&before[2..].join(" "), " ,").to_string();
        let words = text::split(&body);
        if words.is_empty() || words.len() > 4 || !crate::understanding::pyuni::is_alpha(&words[0]) {
            continue;
        }
        // A small count in the other half is said in words when the half is said whole.
        let words: Vec<String> = words
            .iter()
            .map(|w| {
                if crate::understanding::pyuni::is_digit(w) && text::len(w) < 3 {
                    _a_number_word_v(&Value::from(w.as_str()))
                } else {
                    w.clone()
                }
            })
            .collect();
        let body = words.join(" ");
        // a half it cannot stand on its own is not a verb: "is" alone says nothing about the day
        if ["is", "are", "was", "were", "am"].contains(&words[0].to_lowercase().as_str()) {
            continue;
        }
        return body;
    }
    String::new()
}

/// Python: bot/memory/store_replies.py:5647 `_the_verb_they_asked(plan, low, line="")`
///
/// The plan with the verb the ask named put back into it.
pub fn _the_verb_they_asked(plan: &str, low: &str, line: &str) -> String {
    let text_ = text::squash(plan);
    let Some(m) = _A_DATE_AT_THE_END.match_(&text_) else { return plan.to_string() };
    let date = m.name_str("date").to_string();
    let mut subject = m.name_str("subject").to_string();
    // "my daughters" is the thing, and "is" is the verb with its predicate left out.
    let parts = text::split(&subject);
    let last = parts.last().expect("IndexError: list index out of range");
    if ["is", "are", "was", "were"].contains(&last.as_str()) {
        subject = parts[..parts.len() - 1].join(" ");
    }
    let asked = text::strip_chars(&text::squash(low), "?.! ").to_string();
    let at = _the_whole_word_in(&asked, &subject);
    if at < 0 {
        return plan.to_string();
    }
    let rest = text::strip(&text::slice(&asked, (at as usize + text::len(&subject)) as isize, None)).to_string();
    if rest.is_empty() || ["again", "now", "this year", "next year"].contains(&rest.as_str()) {
        // The ask names nothing the half left out, so the line it came from does.
        let borrowed = _the_verb_from_the_line(plan, line);
        if !borrowed.is_empty() {
            return subject + " " + &borrowed + " " + &date;
        }
        return plan.to_string();
    }
    let mut words = text::split(&rest);
    if words.len() > 4 {
        return plan.to_string();
    }
    let first = words[0].to_lowercase();
    // A word that is not a doing word is the predicate left out of the plan, said as it is.
    if ["birthday", "bday", "dob", "date of birth", "name"].contains(&first.as_str()) {
        return subject + " " + &rest + " is " + &date;
    }
    if first.ends_with("ing") && text::len(&words[0]) > 4 {
        // "when is my nephew turning 5" is the doing word as it is, with the "is" it needs.
        return subject + " is " + &words.join(" ") + " " + &date;
    }
    if ["is", "are", "was", "were", "will", "would", "can", "could", "has", "had", "does", "did"].contains(&first.as_str()) {
        return subject + " " + &words.join(" ") + " " + &date;
    }
    if first.ends_with('s') && !first.ends_with("ss") {
        // A word already said in the plural after the thing is more of the thing, not the verb.
        return plan.to_string();
    }
    words[0] = _third_person(&words[0]);
    subject + " " + &words.join(" ") + " " + &date
}

/// A function reference in a value, for a contract that takes a Python callable as `&Value`.
fn fn_ref(id: &str) -> Value {
    Value::Fn(std::sync::Arc::new(data::FnRef { id: id.to_string(), cells: Vec::new(), defaults: Vec::new() }))
}

rx!(SAYS_A_TRIP);

/// Python: bot/memory/store_replies.py:5702 `_plans_reply(memory, low)`
///
/// What is coming up, in the words they said it in: the one plan, or the newest of several.
/// (Contract change: the generated stub returned `()`.)
pub fn _plans_reply(memory: &mut Memory, low: &str) -> Option<String> {
    use crate::memory::{plan_line, plan_sentence};
    let known = user(memory);
    let plans = _plans_with_the_latest(&known);
    if let Some(worked) = _the_date_behind_that(memory, low, &plans) {
        return Some(worked);
    }
    if WHERE_PLAN.is_match(low) || WHEN_PLAN.is_match(low) {
        // "where are we going again" and "which month are we going" are the trip said back; an
        // ask that names one part gets that part.
        if let Some(last) = plans.last() {
            let said = plan_line(memory, last);
            let part = crate::memory::when::the_part_asked(last, low, &said, &fn_ref("bot.memory.places:_a_place"));
            if !part.is_empty() && crate::memory::topics::content(&part) != crate::memory::topics::content(&last_turn_bot(memory)) {
                return Some(part);
            }
            return Some(plan_sentence(last, &said) + ".");
        }
        return None;
    }
    if let Some(clock) = TIME_ASKS.search(low) {
        // "what time am I meeting sam?" is the plan that names that verb, answered with its hour.
        let matching: Vec<String> = plans.iter().filter(|p| _says_the_verb(p, clock.str(1))).cloned().collect();
        let plan = _the_one_asked(&matching, &low[clock.end()..]);
        if !plan.is_empty() {
            let at = _at_the_clock(&plan);
            if !at.is_empty() {
                return Some(at);
            }
            let line = plan_line(memory, &plan);
            return Some(plan_sentence(&plan, &line) + ".");
        }
        return None;
    }
    if let Some(hour) = TIME_THING.search(low) {
        let matching: Vec<String> = plans.iter().filter(|p| _names_the_thing_m(p, hour.str(1), None)).cloned().collect();
        let plan = _the_one_asked(&matching, &low[hour.end()..]);
        if !plan.is_empty() {
            let at = _at_the_clock(&plan);
            if !at.is_empty() {
                return Some(at);
            }
            return Some(plan_sentence(&plan, "") + ".");
        }
        return None;
    }
    if TIME_IT.is_match(low) {
        // "what time was that again" points at the plan on the table, and the hour it names is the answer.
        if let Some(last) = plans.last() {
            return Some(_at_the_clock(last));
        }
        return None;
    }
    if let Some(day_asked) = DAY_ASKS.search(low) {
        // "what day do i start": the day is the whole answer ("On Tuesday.").
        let matching: Vec<String> = plans.iter().filter(|p| _says_the_verb(p, day_asked.str(1))).cloned().collect();
        let plan = _the_one_asked(&matching, &low[day_asked.end()..]);
        if !plan.is_empty() {
            let day = _the_day_of(&plan);
            if !day.is_empty() {
                return Some(day);
            }
            let line = plan_line(memory, &plan);
            return Some(plan_sentence(&plan, &line) + ".");
        }
        return None;
    }
    if let Some(day) = DAY_THING.search(low) {
        let matching: Vec<String> = plans.iter().filter(|p| _names_the_thing_m(p, day.str(1), None)).cloned().collect();
        let plan = _the_one_asked(&matching, &low[day.end()..]);
        if !plan.is_empty() {
            let said = _the_day_of(&plan);
            if !said.is_empty() {
                return Some(said);
            }
            let line = plan_line(memory, &plan);
            return Some(plan_sentence(&plan, &line) + ".");
        }
        return None;
    }
    if DAY_IT.is_match(low) {
        // "what day is it on" is the plan's day and not today's.
        if let Some(last) = plans.last() {
            return Some(_the_day_of(last));
        }
        return None;
    }
    // The plan said back with the verb the ask named put into it.
    let said_with_the_verb = |memory: &mut Memory, plan: &str| -> String {
        let line = plan_line(memory, plan);
        let source = if line.is_empty() { _the_line_that_said(memory, plan).unwrap_or_default() } else { line };
        let line2 = plan_line(memory, plan);
        plan_sentence(&_the_verb_they_asked(plan, low, &source), &line2) + "."
    };
    if let Some(when) = WHEN_ASKS.search(low) {
        // The plan that names the verb they asked about answers "when do I start?".
        let matching: Vec<String> = plans.iter().filter(|p| _says_the_verb(p, when.str(1))).cloned().collect();
        let plan = _the_one_asked(&matching, &low[when.end()..]);
        if !plan.is_empty() {
            let day = _the_day_of(&plan);
            if !day.is_empty() && _the_day_is_the_whole_date(&plan) && _the_plan_was_just_said_back(memory, &plan) {
                // The plan read back is that same sentence twice, a line apart: the day is the answer.
                return Some(day);
            }
            return Some(said_with_the_verb(memory, &plan));
        }
        // A half of a line with two things in it keeps no verb, and the ask names the verb it left out.
        let halves: Vec<String> = plans.iter().filter(|p| _the_verb_they_asked(p, low, "") != **p).cloned().collect();
        let plan = if halves.is_empty() { String::new() } else { _the_one_asked(&halves, "") };
        if !plan.is_empty() {
            return Some(said_with_the_verb(memory, &plan));
        }
        // "when do i go on holiday?": a plan that is a trip answers it.
        if SAYS_A_TRIP.is_match(low) {
            for plan in plans.iter().rev() {
                if _about_kind("trip", plan, Some(&mut *memory), true) {
                    let line = plan_line(memory, plan);
                    return Some(plan_sentence(plan, &line) + ".");
                }
            }
        }
        return None;
    }
    let thing = _when_thing(low, plans.is_empty());
    if !thing.is_empty() {
        // The plan that names the thing they asked about answers "when is the wedding?".
        let mut asking: Vec<String> = Vec::new();
        for p in &plans {
            if _names_the_thing_m(p, &thing, Some(&mut *memory)) {
                asking.push(p.clone());
            }
        }
        if _asks_the_end(low) || _asks_the_start(low) {
            // "when does my job end" is not answered by the day it starts.
            asking.retain(|p| _answers_the_end_asked(p, low));
        }
        let plan = _the_one_asked(&asking, "");
        if !plan.is_empty() {
            let day = _the_day_of(&plan);
            if !day.is_empty() && _the_day_is_the_whole_date(&plan) && _the_plan_was_just_said_back(memory, &plan) {
                return Some(day);
            }
            return Some(said_with_the_verb(memory, &plan));
        }
        // The thing is named but nothing they told us is about it: not a plans answer at all.
        return None;
    }
    if WHEN_THING.is_match(low) && BANK_HOLIDAY.is_match(low) {
        // A when ask about a bank holiday is a question about the calendar.
        return None;
    }
    if _thing_is_a_quality(low) {
        return None;
    }
    if plans.is_empty() {
        if _a_time_for_the_job_itself(low) {
            // "when is the best time to do it" is about the job in front of them.
            return None;
        }
        return Some(pick(&NO_PLANS_REPLY, memory, ""));
    }
    if _the_answer_about_it_is_on_the_screen(memory, plans.last().unwrap(), low) {
        // The answer this plan's own line was given is what the person is asking for again.
        return None;
    }
    let mut said: Vec<String> = Vec::new();
    for plan in &plans[plans.len().saturating_sub(2)..] {
        let line = plan_line(memory, plan);
        said.push(plan_sentence(plan, &line));
    }
    said.reverse();
    if said.len() == 1 {
        return ret_opt(crate::compose::recall::with_a_tail(&(said[0].clone() + "."), memory));
    }
    Some(said[0].clone() + ". And " + &lower_first(&said[1]) + ".")
}

static BANK_HOLIDAY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:bank|public)\s+holiday", 0));
strset!(THEIR_PLACES);
rx!(IN_A_PLACE, IN_MY_PLACE, FOR_A_MEAL);

/// Python: bot/memory/store_replies.py:5870 `_their_place(low)`
///
/// The place they asked about, when the line asks what is in one of their own food places.
pub fn _their_place(low: &str) -> Option<String> {
    for pattern in [&*IN_A_PLACE, &*IN_MY_PLACE] {
        if let Some(found) = pattern.search(low) {
            if THEIR_PLACES.contains(found.str(1)) {
                return Some(found.str(1).to_string());
            }
        }
    }
    if FOR_A_MEAL.is_match(low) {
        return Some("fridge".into()); // what to have for dinner is answered from what is in it
    }
    None
}

static TOPICS_STOPWORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.topics.STOPWORDS"));
static SHORT_TURN: LazyLock<usize> = LazyLock::new(|| data::get("bot.memory.topics.SHORT_TURN").as_int().unwrap() as usize);
static FACTS_MONTHS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.facts.MONTHS"));

/// Python: bot/memory/store_replies.py:5881 `_naming_words(text)`
///
/// The words of a line that name something, in one spelling both sides compare them in.
pub fn _naming_words(text_: &str) -> Vec<String> {
    crate::memory::topics::words(text_).into_iter().filter(|w| !TOPICS_STOPWORDS.contains(w) && text::len(w) > 2).collect()
}

/// Python: bot/memory/store_replies.py:5886 `_the_day_is_the_whole_date(plan)`
///
/// True when the day is all the plan says about when, so the day is the whole answer.
pub fn _the_day_is_the_whole_date(plan: &str) -> bool {
    let low = plan.to_lowercase();
    if !_at_the_clock(plan).is_empty() {
        return false;
    }
    !FACTS_MONTHS.iter().any(|name| Re::new(&format!(r"\b{name}\b"), 0).is_match(&low))
}

/// Python: bot/memory/store_replies.py:5898 `_the_plan_was_just_said_back(memory, plan)`
///
/// True when the reply above is this plan in the store's own words.
pub fn _the_plan_was_just_said_back(memory: &Memory, plan: &str) -> bool {
    let mine: HashSet<String> = LETTER_WORDS.findall_str(&plan.to_lowercase()).into_iter().collect();
    if mine.len() < 3 {
        return false;
    }
    let said: HashSet<String> = LETTER_WORDS.findall_str(&last_turn_bot(memory).to_lowercase()).into_iter().collect();
    let shared = mine.iter().filter(|w| said.contains(*w)).count();
    shared >= 3.max((mine.len() * 7 + 9) / 10)
}

static HEDGE: LazyLock<&'static Re> = LazyLock::new(|| g("_HEDGE").as_re().unwrap());

/// Python: bot/memory/store_replies.py:5912 `_the_answer_about_it_is_on_the_screen(memory, plan, text)`
///
/// True when the last line on the screen is the answer the plan's own line was given.
pub fn _the_answer_about_it_is_on_the_screen(memory: &Memory, plan: &str, text_: &str) -> bool {
    use crate::memory::topics::{keywords, only_acknowledges};
    let mine: HashSet<String> = text::split(&plan.to_lowercase())
        .into_iter()
        .filter(|w| crate::understanding::pyuni::is_alpha(w) && text::len(w) > 2 && !TOPICS_STOPWORDS.contains(w))
        .collect();
    let turns = &memory.turns;
    if mine.is_empty() || turns.is_empty() {
        return false;
    }
    let asked: HashSet<String> = _naming_words(text_).into_iter().collect();
    if asked.is_empty() {
        return false; // a bare "what am i up to" asks for the plan itself
    }
    let mut screen: HashSet<String> = HashSet::new();
    for turn in turns.iter().rev() {
        let on_screen = text::strip(&turn.bot);
        if on_screen.is_empty() {
            continue;
        }
        if only_acknowledges(on_screen) {
            continue; // a courtesy between the two asks is not the answer leaving the screen
        }
        screen = _naming_words(on_screen).into_iter().collect();
        break;
    }
    if screen.is_empty() {
        return false;
    }
    for turn in turns.iter().rev() {
        let kw: HashSet<String> = keywords(&turn.user).into_iter().collect();
        let named = mine.iter().filter(|w| kw.contains(*w)).count();
        if !(named >= 2 && named * 2 >= mine.len()) {
            continue;
        }
        let answer = text::strip(&turn.bot).to_string();
        if answer.is_empty() {
            continue; // the turn in hand, before anything has been said back
        }
        if HEDGE.is_match(&answer) || answer.ends_with('?') || text::split(&answer).len() <= *SHORT_TURN {
            return false;
        }
        let in_answer: HashSet<String> = _naming_words(&answer).into_iter().collect();
        if !asked.iter().any(|w| in_answer.contains(w)) {
            return false; // the answer says nothing about what this ask names
        }
        return !screen.is_empty() && screen.iter().any(|w| in_answer.contains(w));
    }
    false
}

strset!(WHEN_QUALITY, WHEN_VERB, POINTS_BACK, WHEN_VAGUE, WHEN_END_ASK, WHEN_START_ASK, WHEN_END_WORD);
static A_WORD_WITH_DASHES: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z][a-z'-]*", 0));

/// Python: bot/memory/store_replies.py:5966 `_a_time_for_the_job_itself(low)`
///
/// Whether the when ask is about the best time for a job rather than about their calendar.
pub fn _a_time_for_the_job_itself(low: &str) -> bool {
    for pattern in [&*WHEN_THING, &*TIME_THING] {
        let Some(found) = pattern.search(low) else { continue };
        let said = A_WORD_WITH_DASHES.findall_str(found.str(1));
        if !said.is_empty() && said.iter().all(|w| WHEN_QUALITY.contains(w)) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/store_replies.py:5977 `_thing_is_a_quality(low)`
///
/// True when a when ask names a quality of the world and nothing of its own.
pub fn _thing_is_a_quality(low: &str) -> bool {
    let Some(found) = WHEN_THING.search(low) else { return false };
    let said = A_WORD_WITH_DASHES.findall_str(found.str(1));
    if said.is_empty() || said.iter().any(|w| !WHEN_QUALITY.contains(w)) {
        return false;
    }
    _names_something_of_its_own(low, text::char_index(low, found.end()) as i64)
}

/// Python: bot/memory/store_replies.py:5991 `_names_something_of_its_own(low, start)`
///
/// Whether the rest of the line (from character `start`) names a thing, rather than a verb and
/// a word pointing back.
pub fn _names_something_of_its_own(low: &str, start: i64) -> bool {
    let rest = text::slice(low, start as isize, None);
    for word in A_WORD_WITH_DASHES.findall_str(&rest) {
        if STOPWORDS.contains(&word)
            || WHEN_VERB.contains(&word)
            || POINTS_BACK.contains(&word)
            || WHEN_QUALITY.contains(&word)
            || WHEN_VAGUE.contains(&word)
            || text::len(&word) < 3
        {
            continue;
        }
        return true;
    }
    false
}

/// Python: bot/memory/store_replies.py:6001 `_when_thing(low, wide=False)`
///
/// The thing a when ask names, as a content word: "when is the wedding" -> "wedding".
pub fn _when_thing(low: &str, wide: bool) -> String {
    use crate::memory::topics::keywords;
    let found = WHEN_THING.search(low).or_else(|| WHEN_THING_VERB.search(low));
    let Some(found) = found else { return String::new() };
    let mut said = keywords(found.str(1));
    if wide && !said.is_empty() && said.iter().all(|w| WHEN_VAGUE.contains(w)) {
        for word in A_WORD_WITH_DASHES.findall_str(&low[found.end()..]).into_iter().take(4) {
            let words = keywords(&word);
            if words.is_empty() {
                continue;
            }
            if WHEN_VAGUE.contains(&words[0]) {
                return String::new(); // "the next time we meet" is still their own diary
            }
            said = words;
            break;
        }
    }
    said.into_iter().find(|w| !WHEN_VAGUE.contains(w)).unwrap_or_default()
}

/// Python: bot/memory/store_replies.py:6034 `_the_one_asked(matching, rest)`
///
/// Which of the plans that fit the ask is the one they mean, or "" when none fits.
pub fn _the_one_asked(matching: &[String], rest: &str) -> String {
    let asked = if rest.is_empty() { Vec::new() } else { crate::memory::topics::content(rest) };
    if !asked.is_empty() {
        for plan in matching.iter().rev() {
            if asked.iter().any(|w| _names_the_thing_m(plan, w, None)) {
                return plan.clone();
            }
        }
    }
    matching.last().cloned().unwrap_or_default()
}

/// Python: bot/memory/store_replies.py:6050 `_ask_verb(low)`
///
/// The verb the ask puts after the thing: "when does my job end" -> "end".
pub fn _ask_verb(low: &str) -> String {
    WHEN_THING_VERB.search(low).map(|m| m.str(2).to_string()).unwrap_or_default()
}

/// Python: bot/memory/store_replies.py:6055 `_asks_the_end(low)`
pub fn _asks_the_end(low: &str) -> bool {
    let verb = _ask_verb(low);
    !verb.is_empty() && WHEN_END_ASK.contains(&crate::memory::topics::stem(&verb))
}

/// Python: bot/memory/store_replies.py:6060 `_asks_the_start(low)`
pub fn _asks_the_start(low: &str) -> bool {
    let verb = _ask_verb(low);
    !verb.is_empty() && WHEN_START_ASK.contains(&crate::memory::topics::stem(&verb))
}

/// Python: bot/memory/store_replies.py:6065 `_says_the_end(plan)`
///
/// Whether the plan's own words name its end: "i finish my course in june" does.
pub fn _says_the_end(plan: &str) -> bool {
    text::split(plan).iter().any(|w| {
        let word = text::strip_chars(w, ".,!?;:");
        WHEN_END_WORD.contains(&word.to_lowercase()) || WHEN_END_WORD.contains(&crate::memory::topics::stem(word))
    })
}

/// Python: bot/memory/store_replies.py:6070 `_answers_the_end_asked(plan, low)`
pub fn _answers_the_end_asked(plan: &str, low: &str) -> bool {
    let verb = _ask_verb(low);
    if !verb.is_empty() && _says_the_verb(plan, &verb) {
        return true;
    }
    if _asks_the_end(low) {
        return _says_the_end(plan);
    }
    !_says_the_end(plan)
}

/// Python: bot/memory/store_replies.py:6082 `_names_the_thing(plan, thing, memory=None)`
///
/// True when the plan is about that thing: "wedding" is in "my sister's wedding is next june".
pub fn _names_the_thing(plan: &str, thing: &str, memory: &mut Memory) -> bool {
    _names_the_thing_m(plan, thing, Some(memory))
}

/// `_names_the_thing` with Python's `memory=None` expressible.
pub fn _names_the_thing_m(plan: &str, thing: &str, memory: Option<&mut Memory>) -> bool {
    use crate::memory::topics::{content, forms};
    let mut keys = vec![thing.to_string()];
    keys.extend(content(thing));
    for word in content(plan) {
        let word_forms = forms(&word);
        for key in &keys {
            if *key == word || word_forms.contains(key) || forms(key).contains(&word) {
                return true;
            }
        }
    }
    _about_kind(thing, plan, memory, true)
}

static VERB_ALSO: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| g("VERB_ALSO").str_list_map());

/// Python: bot/memory/store_replies.py:6099 `_says_the_verb(plan, verb)`
///
/// True when the plan names that verb: "start" is in "i start my new job on monday".
pub fn _says_the_verb(plan: &str, verb: &str) -> bool {
    let verb = verb.to_lowercase();
    let mut forms: HashSet<String> = [verb.clone(), verb.clone() + "s", verb.clone() + "ing", verb.clone() + "ed"].into_iter().collect();
    let stem_of = |v: &str| -> String {
        let mut c = v.chars();
        c.next_back();
        c.as_str().to_string()
    };
    if verb.ends_with('e') {
        forms.insert(stem_of(&verb) + "ing");
        forms.insert(verb.clone() + "d");
    }
    if verb.ends_with('y') {
        forms.insert(stem_of(&verb) + "ied");
    }
    if text::ends_with_any(&verb, &["s", "x", "z", "ch", "sh"]) {
        forms.insert(verb.clone() + "es");
    }
    for other in VERB_ALSO.get(&verb).cloned().unwrap_or_default() {
        forms.extend([other.clone(), other.clone() + "s", other.clone() + "ing", other.clone() + "ed"]);
        if other.ends_with('e') {
            forms.insert(stem_of(&other) + "ing");
            forms.insert(other.clone() + "d");
        }
    }
    text::split(plan).iter().any(|w| forms.contains(&text::strip_chars(w, ".,!?;:").to_lowercase()))
}

/// Python: bot/memory/store_replies.py:6125 `_job_reply(memory, where=False)`
pub fn _job_reply(memory: &mut Memory, r#where: bool) -> String {
    let known = user(memory);
    let job = known.get("job").cloned().unwrap_or(Value::None);
    let city = known.get("city").cloned().unwrap_or(Value::None);
    let place = text::strip(&sget(&known, "workplace")).to_string();
    if r#where && !place.is_empty() {
        // The place is exactly what was asked for, so it is the answer, with the role left out.
        return "You work ".to_string() + &_place_phrase(&place) + ".";
    }
    if job.truthy() && city.truthy() {
        let said = _job_said(&known);
        let city = py_str(&city);
        return pick(
            &[said.clone() + " in " + &city + ".", "You're in ".to_string() + &city + " \u{2014} " + &lower_first(&said) + "."],
            memory,
            "",
        );
    }
    if job.truthy() {
        return _job_said(&known) + ".";
    }
    if city.truthy() && r#where {
        // Only a where-ask can be answered with the town.
        return "You said you live in ".to_string() + &py_str(&city) + ".";
    }
    let denial = pick(&["You haven't told me what you do yet.", "I don't know your job \u{2014} you haven't said."], memory, "");
    _deny(memory, "job", &denial, None)
}

/// Python: bot/memory/store_replies.py:6147 `_yours(value)`
///
/// The value said back with their own words turned round: "for myself" -> "for yourself".
pub fn _yours(value: &str) -> String {
    text::split(value)
        .iter()
        .map(|word| {
            let low = text::strip_chars(&word.to_lowercase(), ".,!?").to_string();
            match low.as_str() {
                "myself" => "yourself".to_string(),
                "my" => "your".to_string(),
                "me" => "you".to_string(),
                "i" => "you".to_string(),
                "myself." => "yourself.".to_string(),
                "our" => "your".to_string(),
                "us" => "you".to_string(),
                _ => word.clone(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

strset!(WORK_CIRCUMSTANCE);
static WORDS_AND_DIGITS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9']+", 0));

/// Python: bot/memory/store_replies.py:6161 `_a_circumstance(job)`
///
/// Whether the word after "I work" is how they work, not what they do.
pub fn _a_circumstance(job: &str) -> bool {
    let words = WORDS_AND_DIGITS.findall_str(&job.to_lowercase());
    let Some(last) = words.last() else { return true };
    if words.iter().any(|w| WORK_CIRCUMSTANCE.contains(w)) {
        return true;
    }
    last.ends_with('s') && !last.ends_with("ss")
}

/// Python: bot/memory/store_replies.py:6177 `_job_said(known)`
///
/// The job said back the way they said it: "You work in a warehouse", "You're a nurse".
pub fn _job_said(known: &Value) -> String {
    use crate::memory::relations::_a;
    let job = _yours(text::strip(&sget(known, "job")));
    let frame = text::strip(&sget(known, "job_frame")).to_string();
    let preposition = frame.split(' ').next().unwrap_or("").to_string();
    if frame == "was" {
        // The job was given in the past, so it is said back in the past.
        if _a_circumstance(&job) {
            return "You did ".to_string() + &job;
        }
        return "You were ".to_string() + &_a(&job);
    }
    if frame == "work" {
        // "i work nights": the verb is the frame; a role said with the same frame is "you're a chef".
        if _a_circumstance(&job) {
            return "You work ".to_string() + &job;
        }
        return "You're ".to_string() + &_a(&job);
    }
    if ["in", "for", "on", "with"].contains(&preposition.as_str()) || preposition == "as" {
        // The preposition they said is the one said back.
        return "You work ".to_string() + &frame + " " + &job;
    }
    if preposition == "at" {
        // "at" alone is followed by a name, which keeps its capital.
        if frame == "at" {
            return "You work at ".to_string() + &text::upper_first(&job);
        }
        return "You work ".to_string() + &frame + " " + &job;
    }
    "You told me you're ".to_string() + &_a(&job)
}

/// Python: bot/memory/store_replies.py:6217 `_place_phrase(place)`
///
/// Where they work, as the line said it: "in a school", "at Tesco".
pub fn _place_phrase(place: &str) -> String {
    let place = text::strip(place).to_string();
    let (head, _, rest) = text::partition(&place, " ");
    if head.to_lowercase() == "at" && !rest.is_empty() && rest.to_lowercase() == rest {
        let first = text::split(rest).first().map(|w| w.to_lowercase()).expect("IndexError: list index out of range");
        if !["the", "a", "an"].contains(&first.as_str()) {
            return "at ".to_string() + &text::upper_first(rest);
        }
    }
    place
}

/// Python: bot/memory/store_replies.py:6232 `_job_phrase(known)`
///
/// The job as a clause inside another sentence: "you work nights", "you're a nurse".
pub fn _job_phrase(known: &Value) -> String {
    let said = _job_said(known);
    let low = lower_first(&said);
    match low.strip_prefix("you told me ") {
        Some(rest) => rest.to_string(),
        None => low,
    }
}

static BORN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bborn\b", 0));
static HOMETOWN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhometown\b", 0));
static GOING_AWAY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bgoing\b|\btrip\b|\bholiday\b|\btravel", 0));
static FROM_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bfrom\b", 0));
static COME_FROM: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bfrom\b|\bcome from\b", 0));
static WHERE_DO_I_LIVE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bwhere (?:do|does) (?:i|we) live\b|\bwhere am i (?:living|based)\b", 0));

/// A str value that says something (`isinstance(v, str) and v`).
fn a_str(v: Option<&Value>) -> Option<String> {
    match v {
        Some(Value::Str(s)) if !s.is_empty() => Some(s.to_string()),
        _ => None,
    }
}

/// Python: bot/memory/store_replies.py:6246 `_places_reply(memory, low)`
pub fn _places_reply(memory: &mut Memory, low: &str) -> Option<String> {
    let known = user(memory);
    // "where does my sister live" is about her, not about the user.
    let who = crate::memory::places::_asked_person(memory, low);
    if !who.is_empty() {
        if let Some(hers) = crate::memory::places::_other_place(memory, &who).filter(|h| !h.is_empty()) {
            return Some(hers);
        }
        return Some("You haven't said where your ".to_string() + &who + if who.to_lowercase().ends_with('s') { " live." } else { " lives." });
    }
    let city = known.get("city").cloned().unwrap_or(Value::None);
    if BORN.is_match(low) {
        // A birth is not where they are from.
        if let Some(born) = a_str(known.get("born")) {
            return Some(pick(&["You were born in ".to_string() + &born + ".", born.clone() + " \u{2014} you were born there."], memory, ""));
        }
        if let Some(origin) = a_str(known.get("from")) {
            return Some(pick(
                &[
                    "You haven't told me where you were born \u{2014} you're from ".to_string() + &origin + ".",
                    "I don't know where you were born. You told me you're from ".to_string() + &origin + ".",
                ],
                memory,
                "",
            ));
        }
        return Some(pick(
            &["You haven't told me where you were born.", "I don't know that one \u{2014} you haven't said where you were born."],
            memory,
            "",
        ));
    }
    if HOMETOWN.is_match(low) {
        return Some(_from_reply(memory));
    }
    if GROWN_UP_ASK.is_match(low) {
        // "where did i grow up" is said back as growing up.
        return Some(_grown_up_reply(memory));
    }
    let going = GOING_AWAY.is_match(low);
    // "where am I from" asks where they grew up, not where they live now.
    if let Some(c) = a_str(Some(&city)) {
        if known.get("home").map(Value::truthy).unwrap_or(false) && !going && !FROM_WORD.is_match(low) {
            let said_with = known.get("home").unwrap();
            return Some("You live in ".to_string() + &(if let Value::Str(s) = said_with { s.to_string() } else { c }) + ".");
        }
    }
    if COME_FROM.is_match(low) {
        return Some(_from_reply(memory));
    }
    let mut places: Vec<String> = known.get("places").map(py_list).unwrap_or_default().iter().filter_map(|p| p.as_str().map(str::to_string)).collect();
    if going {
        // "where am I going" is about the trip, so the place they live is not the answer.
        let city_s = if let Value::Str(c) = &city { Some(c.to_string()) } else { None };
        let away: Vec<String> = places.iter().filter(|p| city_s.as_ref().map(|c| c != *p).unwrap_or(false)).cloned().collect();
        if !away.is_empty() {
            places = away;
        }
    }
    if low.contains("city") || low.contains("town") {
        let cities: Vec<String> = places.iter().filter(|p| !FACTS_COUNTRIES.contains(&p.to_lowercase())).cloned().collect();
        if !cities.is_empty() {
            places = cities;
        }
    }
    if places.is_empty() {
        return Some(_deny(memory, "place", "You haven't mentioned a place yet.", None));
    }
    if WHERE_DO_I_LIVE.is_match(low) {
        // They asked where they live and never said: the place they named is not an answer.
        if let Some(origin) = a_str(known.get("from")) {
            return Some(pick(
                &[
                    "You haven't told me where you live \u{2014} I know you're from ".to_string() + &origin + ".",
                    "I don't know where you live. You're from ".to_string() + &origin + ", that I do know.",
                ],
                memory,
                "",
            ));
        }
        return Some(if places.len() == 1 {
            "You haven't told me where you live. You mentioned ".to_string() + &places[0] + "."
        } else {
            "You haven't told me where you live. You mentioned ".to_string() + &places[0] + ", then " + &places[1..].join(", ") + "."
        });
    }
    let first = places[0].clone();
    let rest: Vec<String> = places[1..].iter().filter(|p| **p != first).cloned().collect();
    if !rest.is_empty() {
        return Some("You said ".to_string() + &first + " first, then " + &rest.join(", ") + ".");
    }
    Some("You mentioned ".to_string() + &first + ".")
}

/// Python: bot/memory/store_replies.py:6321 `_grown_up_reply(memory)`
///
/// Where they grew up, said as growing up rather than as where they are from.
pub fn _grown_up_reply(memory: &mut Memory) -> String {
    let known = user(memory);
    if let Some(origin) = a_str(known.get("from")) {
        return pick(&["You grew up in ".to_string() + &origin + ".", origin.clone() + " \u{2014} that's where you grew up."], memory, "");
    }
    if let Some(city) = a_str(known.get("city")) {
        return pick(
            &[
                "You haven't said where you grew up \u{2014} I know you live in ".to_string() + &city + ".",
                "Not sure where you grew up. You mentioned ".to_string() + &city + " though.",
            ],
            memory,
            "",
        );
    }
    pick(&["You haven't said where you grew up yet.", "I don't know that one \u{2014} you haven't told me where you grew up."], memory, "")
}

/// Python: bot/memory/store_replies.py:6334 `_from_reply(memory)`
///
/// Where they are from, which is a different question from where they live now.
pub fn _from_reply(memory: &mut Memory) -> String {
    let known = user(memory);
    let origin = known.get("from").cloned().unwrap_or(Value::None);
    let city = known.get("city").cloned().unwrap_or(Value::None);
    if origin.truthy() && city.truthy() && py_str(&origin).to_lowercase() != py_str(&city).to_lowercase() {
        let (o, c) = (py_str(&origin), py_str(&city));
        return pick(
            &[
                "You're from ".to_string() + &o + " originally, and you live in " + &c + " now.",
                "You said you're from ".to_string() + &o + ", and you're in " + &c + " now.",
            ],
            memory,
            "",
        );
    }
    if origin.truthy() {
        let o = py_str(&origin);
        return pick(&["You're from ".to_string() + &o + ".", o.clone() + " \u{2014} you told me you're from there."], memory, "");
    }
    if city.truthy() {
        let c = py_str(&city);
        return pick(
            &[
                "You haven't said where you're from \u{2014} I know you live in ".to_string() + &c + ".",
                "Not sure where you're from. You mentioned ".to_string() + &c + " though.",
            ],
            memory,
            "",
        );
    }
    pick(&["You haven't said where you're from yet.", "I don't know that one \u{2014} you haven't said."], memory, "")
}

/// Python: bot/memory/store_replies.py:6351 `_is_are(who)`
pub fn _is_are(who: &str) -> String {
    if crate::memory::relations::_plural_person(who) { " are " } else { " is " }.to_string()
}

/// Python: bot/memory/store_replies.py:6354 `_eats(who)`
pub fn _eats(who: &str) -> String {
    if crate::memory::relations::_plural_person(who) { " eat" } else { " eats" }.to_string()
}

/// Python: bot/memory/store_replies.py:6357 `_does_not(who)`
pub fn _does_not(who: &str) -> String {
    if crate::memory::relations::_plural_person(who) { " don't" } else { " doesn't" }.to_string()
}

static FACTS_SHE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.SHE"));
static FACTS_HE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.HE"));

/// Python: bot/memory/store_replies.py:6360 `_they(who)`
///
/// The word for the person at the front of a sentence of their own: "she", "he", "they".
pub fn _they(who: &str) -> String {
    let low = text::strip(&who.to_lowercase()).to_string();
    if FACTS_SHE.contains(&low) {
        return "she".into();
    }
    if FACTS_HE.contains(&low) {
        return "he".into();
    }
    "they".into()
}

/// Python: bot/memory/store_replies.py:6369 `_a_diet_predicate(who, diet)`
///
/// A person's diet as the words after their name: "eats fish", "doesn't eat meat", "is vegan".
pub fn _a_diet_predicate(who: &str, diet: &str) -> String {
    let said = text::strip(diet).to_string();
    if let Some(rest) = said.strip_prefix("eats ") {
        return text::strip(&(_eats(who) + " " + rest)).to_string();
    }
    if said.starts_with("no ") {
        // What they drink is not what they eat.
        return text::strip(&(_does_not(who) + " " + &_diet_how(&said) + " " + &text::slice(&said, 3, None))).to_string();
    }
    text::strip(&(_is_are(who) + &said)).to_string()
}

/// Python: bot/memory/store_replies.py:6385 `_their_pronoun(who)`
///
/// The word for the person in the second half of a sentence: "for her", "for them".
pub fn _their_pronoun(who: &str) -> String {
    let low = text::strip(&who.to_lowercase()).to_string();
    let (hers, his) = (FACTS_SHE.contains(&low), FACTS_HE.contains(&low));
    if hers && !his {
        return "her".into();
    }
    if his && !hers {
        return "him".into();
    }
    "them".into()
}

/// Python: bot/memory/store_replies.py:6400 `_no_diet(who)`
///
/// An ask about how someone else eats when nothing food-shaped is kept for them.
pub fn _no_diet(who: &str) -> Option<String> {
    if text::strip(who).to_lowercase() == "baby" {
        return None;
    }
    let whose = if who.is_empty() { "they".to_string() } else { "your ".to_string() + who };
    Some("You haven't told me what ".to_string() + &whose + &_eats(who) + " \u{2014} tell me and I'll plan the meals round it.")
}

static CONDITION_FOOD: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.memory.ack.CONDITION_FOOD"));

/// Python: bot/memory/store_replies.py:6414 `_a_condition_rules_out(condition)`
///
/// Whether a condition rules a food out, by the bot's own table: coeliac is gluten.
pub fn _a_condition_rules_out(condition: &Value) -> bool {
    let kept: Vec<Value> = match condition {
        Value::List(items) | Value::Tuple(items) => items.to_vec(),
        other => vec![other.clone()],
    };
    kept.iter().any(|w| CONDITION_FOOD.get(&text::strip(&py_str(w)).to_lowercase()).map(|f| !f.is_empty()).unwrap_or(false))
}

/// Python: bot/memory/store_replies.py:6423 `_both_food_facts(who, allergies, diet)`
///
/// An allergy and a diet kept for the same person, said in one answer.
pub fn _both_food_facts(who: &str, allergies: &Value, diet: &Value) -> String {
    let said = _allergy_is_their_food(who, allergies, false, "");
    let them = text::upper_first(&_they(who));
    let mut rest = _diet_sentence(who, diet);
    let first_lead = if who.is_empty() { "They".to_string() } else { "Your ".to_string() + who };
    for lead in [first_lead.as_str(), "They"] {
        if rest.starts_with(lead) {
            rest = them.clone() + &rest[lead.len()..];
            break;
        }
    }
    said + " " + &rest
}

/// Python: bot/memory/store_replies.py:6441 `_allergy_is_their_food(who, allergies, only=True, low="")`
///
/// What a person can eat, from the only food fact kept for them: the allergy.
pub fn _allergy_is_their_food(who: &str, allergies: &Value, only: bool, low: &str) -> String {
    let owner = if who.is_empty() { "They".to_string() } else { "Your ".to_string() + who };
    let said: Vec<String> = py_list(allergies).iter().map(py_str).filter(|a| !text::strip(a).is_empty()).collect();
    for one in &said {
        let (named, thing) = if low.is_empty() { (String::new(), String::new()) } else { _what_the_food_has(low, one) };
        if !named.is_empty() {
            // The food in their hand is the answer.
            return text::upper_first(&named) + " has " + &thing + " in it, so it's off the table for " + &_their_pronoun(who) + ".";
        }
    }
    let ruled = if said.len() == 1 {
        said[0].clone()
    } else {
        "any of ".to_string() + &said.join(", ")
    };
    let tail = if only { " \u{2014} that's all I have on food for ".to_string() + &_their_pronoun(who) } else { String::new() };
    owner + &_is_are(who) + "allergic to " + &said.join(", ") + ", so nothing with " + &ruled + " in it" + &tail + "."
}

static FOOD_PHRASES: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| g("FOOD_PHRASES").str_list_map());
/// `FOOD_HAS` as lists, in the exported (sorted) order of each set: `sorted(..., key=len)` keeps
/// Python's set order among words of one length, which depends on the hash seed.
static FOOD_HAS_LISTS: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| g("FOOD_HAS").str_list_map());
static RULED_SPLIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s*(?:,|\bor\b|\band\b)\s*", 0));

/// Python: bot/memory/store_replies.py:6467 `_what_the_food_has(low, ruled)`
///
/// The food the line names, when it is a thing the person cannot have: ("cheese", "dairy").
pub fn _what_the_food_has(low: &str, ruled: &str) -> (String, String) {
    for thing in RULED_SPLIT.split(ruled) {
        let thing = text::strip(&thing).to_lowercase();
        if thing.is_empty() {
            continue;
        }
        for phrase in FOOD_PHRASES.get(&thing).cloned().unwrap_or_default() {
            if Re::new(&format!(r"\b{}s?\b", crate::py::re::escape(&phrase)), 0).is_match(low) {
                return (phrase, thing);
            }
        }
        // The longest food word wins, so "peanut butter" is not answered as "peanut".
        let mut words = FOOD_HAS_LISTS.get(&thing).cloned().unwrap_or_default();
        words.sort_by(|a, b| text::len(b).cmp(&text::len(a)));
        for word in words {
            if word == thing {
                continue;
            }
            if Re::new(&format!(r"\b{}s?\b", crate::py::re::escape(&word)), 0).is_match(low) {
                return (word, thing);
            }
        }
    }
    (String::new(), String::new())
}

/// `DIET_RULES_OUT`: a diet -> (what it rules out, the tail said after it).
static DIET_RULES_OUT: LazyLock<HashMap<String, (String, String)>> = LazyLock::new(|| {
    g("DIET_RULES_OUT").pairs().iter().map(|(k, v)| (k.str().to_string(), (v.items()[0].str().to_string(), v.items()[1].str().to_string()))).collect()
});
static A_LIST_OF_THINGS: LazyLock<Re> = LazyLock::new(|| Re::new(r",|\bor\b|\band\b", 0));

/// Python: bot/memory/store_replies.py:6491 `_diet_is_their_food(who, diet, low="", plain=False)`
///
/// What a person cannot have, from the diet kept for them, or None when the diet says it.
pub fn _diet_is_their_food(who: &str, diet: &str, low: &str, plain: bool) -> Option<String> {
    let said = fold(low);
    let diet_low = fold(diet);
    let owner = if who.is_empty() { "They".to_string() } else { "Your ".to_string() + who };
    if diet_low.starts_with("no ") {
        if plain {
            return None;
        }
        // The food said as missing is the answer.
        return Some(owner + " can't have " + &text::slice(&diet_low, 3, None) + " \u{2014} that's all I have on food for " + &_their_pronoun(who) + ".");
    }
    let (rule, tail) = DIET_RULES_OUT.get(&diet_low)?.clone();
    if plain && A_LIST_OF_THINGS.is_match(&rule) {
        return None;
    }
    let (named, thing) = if said.is_empty() { (String::new(), String::new()) } else { _what_the_food_has(&said, &rule) };
    if !named.is_empty() {
        // The food in their hand is the answer.
        return Some(text::upper_first(&named) + " has " + &thing + " in it, and " + &owner.to_lowercase() + &_is_are(who) + &diet_low + ".");
    }
    Some(owner + &_is_are(who) + &diet_low + ", so nothing with " + &rule + " in it" + &tail + " \u{2014} that's all I have on food for " + &_their_pronoun(who) + ".")
}

rx!(DISLIKE_ASK);

/// Python: bot/memory/store_replies.py:6534 `_last_answer(memory)`
///
/// The bot's last knowledge answer, or "" when the previous turn was something else.
pub fn _last_answer(memory: &Memory) -> String {
    match memory.turns.last() {
        Some(last) if last.module.as_deref() == Some("answers") => last.bot.clone(),
        _ => String::new(),
    }
}

static A_DISH_OF_TWO: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?P<one>.+?)\s+(?P<how>and|with|&)\s+(?P<two>.+)$", 0));

/// Python: bot/memory/store_replies.py:6547 `_one_half_of_the_dish(food, taste)`
///
/// True when the food the guest reader found is one half of the dish the line names.
pub fn _one_half_of_the_dish(food: &str, taste: &Value) -> bool {
    if !taste.is_dict() {
        return false;
    }
    let thing = text::strip(&sget(taste, "thing")).to_lowercase();
    let food = text::strip(food).to_lowercase();
    if thing.is_empty() || food.is_empty() || thing == food {
        return false;
    }
    let Some(joined) = A_DISH_OF_TWO.match_(&thing) else { return false };
    says_word(&food, &(joined.name_str("one").to_string() + " " + joined.name_str("two")))
}

/// `guest.found(text)`: [(food, where)]. The contract in `memory::guest` types it `Vec<String>`,
/// which cannot hold the positions; this reads whichever shape it returns.
trait FoundFoods {
    fn food_pairs(self) -> Vec<(String, i64)>;
}
impl FoundFoods for Vec<String> {
    fn food_pairs(self) -> Vec<(String, i64)> {
        if self.is_empty() {
            return Vec::new();
        }
        panic!("contract: bot.memory.guest:found returns (food, at) pairs, typed Vec<String>")
    }
}
impl FoundFoods for Vec<(String, i64)> {
    fn food_pairs(self) -> Vec<(String, i64)> {
        self
    }
}
impl FoundFoods for Value {
    fn food_pairs(self) -> Vec<(String, i64)> {
        self.items().iter().map(|p| (py_str(&p.items()[0]), p.items()[1].as_int().unwrap_or(0))).collect()
    }
}

/// Python: bot/memory/store_replies.py:6567 `_dislikes_here(text)`
///
/// [(food, kind)] in the line, a taste nobody else is named for read as their own.
pub fn _dislikes_here(text_: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (food, at) in crate::memory::guest::found(text_).food_pairs() {
        let said = crate::memory::facts::subject(text_, at);
        let kind = said.first().cloned().unwrap_or_default();
        out.push((food, if kind.is_empty() { "user".to_string() } else { kind }));
    }
    out
}

/// Python: bot/memory/store_replies.py:6581 `_a_dislike_told(text, memory)`
///
/// The answer just given with the food the guest hates taken out, or None.
pub fn _a_dislike_told(text_: &str, memory: &mut Memory) -> Option<String> {
    if DISLIKE_ASK.is_match(text_) {
        return None;
    }
    if _about_this_chat(text_, memory) {
        return None;
    }
    let tastes = crate::memory::taste::read_all(text_).val();
    if tastes.items().len() > 1 {
        // A line of more than one taste is the taste module's.
        return None;
    }
    let found = _dislikes_here(text_);
    if found.iter().any(|(food, _)| tastes.items().iter().any(|t| _one_half_of_the_dish(food, t))) {
        // A dish said with both its halves is one thing, not its first word.
        return None;
    }
    let mut foods: Vec<String> = found.iter().filter(|(_, k)| k == "user" || k == "other").map(|(f, _)| f.clone()).collect();
    let mut alone: Vec<String> = Vec::new();
    if foods.is_empty() && !_last_answer(memory).is_empty() {
        // A bare "no mushrooms" answers the suggestion the bot has just made.
        alone = crate::memory::guest::turned_down_alone(text_);
        foods = alone.clone();
    }
    if foods.is_empty() {
        return None;
    }
    let previous = _last_answer(memory);
    if !previous.is_empty() {
        if let Some(revised) = crate::memory::guest::revise(&previous, &str_list(&foods)).filter(|r| !r.is_empty()) {
            return Some(revised);
        }
        // The answer on the table does not name the food, so this is a steer on that answer.
        if or_on_raise(|| crate::compose::ruledout::takes_as_it_comes(text_, memory).unwrap_or(false), false) {
            return None;
        }
    }
    let a_pair = foods.len() > 1;
    let mut own: Vec<Value> =
        found.iter().filter(|(_, k)| k == "user").map(|(f, _)| crate::memory::guest::plain(f, a_pair).val()).collect();
    if own.is_empty() {
        own = alone.iter().map(|f| crate::memory::guest::plain(f, false).val()).collect();
    }
    let own: Vec<Value> = own.into_iter().filter(Value::truthy).collect();
    if !own.is_empty() {
        return crate::memory::guest::left_out(&Value::List(std::sync::Arc::new(own)));
    }
    None
}

rx!(STOPPED_SAID, NEGATED_SAID, ANY_MORE);
static STOPPED_LEAD: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:i|we)(?:'ve|ve|\s+have|\s+had)?\s+", crate::py::re::I));

/// Python: bot/memory/store_replies.py:6640 `_the_take_back(memory, key)`
///
/// The last line that took the fact back, and the value it took back, or ("", "").
pub fn _the_take_back(memory: &Memory, key: &str) -> (String, String) {
    for turn in memory.turns.iter().rev() {
        let line = &turn.user;
        if text::strip(line).is_empty() {
            continue;
        }
        let denied = crate::memory::facts::denials(line, &Value::None).val();
        let mut gone = denied.get(key).cloned().unwrap_or(Value::None);
        if gone.truthy() {
            // Said back as the fact it took, not as the words the take-back used.
            let record = mget(memory, "denial");
            if text::squash(line) == sget(record, "said") {
                let dropped = record.get(key).cloned().unwrap_or(Value::None);
                match &dropped {
                    Value::List(items) if !items.is_empty() => gone = items[0].clone(),
                    d if d.truthy() => gone = dropped.clone(),
                    _ => {}
                }
            }
            return (text::squash(line), py_str(&gone));
        }
    }
    (String::new(), String::new())
}

strset!(PLAN_DIETS, WAY_OF_EATING);

/// Python: bot/memory/store_replies.py:6662 `_a_take_back_said_back(line, gone)`
///
/// Their own take-back, said back as a statement: "You told me you stopped doing keto."
pub fn _a_take_back_said_back(line: &str, gone: &str) -> String {
    let said = text::squash(line);
    if STOPPED_SAID.match_(&said).is_some() {
        let rest = STOPPED_LEAD.sub("", &said);
        return "You told me you ".to_string() + &text::squash(&rest) + ".";
    }
    if ["diet", "dieting", "on a diet"].contains(&text::strip(gone).to_lowercase().as_str()) {
        return "You told me you're not on a diet any more.".to_string();
    }
    if NEGATED_SAID.is_match(&said) {
        // "im not vegetarian" is a fact about now, not a take-back of one they had.
        return _a_negation_said_back(gone, &said);
    }
    if WAY_OF_EATING.contains(gone) {
        return "You told me you stopped being ".to_string() + &_diet_words(gone) + ".";
    }
    if PLAN_DIETS.contains(gone) {
        return "You told me you stopped doing ".to_string() + &_diet_words(gone) + ".";
    }
    "You told me you stopped ".to_string() + &_diet_words(gone) + "."
}

/// Python: bot/memory/store_replies.py:6688 `_a_negation_said_back(gone, line="")`
///
/// "You told me you're not vegetarian", for a diet said as a negation rather than left.
pub fn _a_negation_said_back(gone: &str, line: &str) -> String {
    let value = text::strip(gone).to_lowercase();
    let tail = if ANY_MORE.is_match(line) { " any more" } else { "" };
    if value.is_empty() {
        return "You told me you're not on a diet".to_string() + tail + ".";
    }
    if value.starts_with("no ") {
        return "You told me you don't ".to_string() + &_diet_how(&value) + " " + &text::slice(&value, 3, None) + tail + ".";
    }
    if WAY_OF_EATING.contains(&value) {
        return "You told me you're not ".to_string() + &_diet_words(&value) + tail + ".";
    }
    if PLAN_DIETS.contains(&value) {
        return "You told me you're not doing ".to_string() + &_diet_words(&value) + tail + ".";
    }
    "You told me you're not ".to_string() + &_diet_words(&value) + tail + "."
}

rx!(A_CLAIM_OF_TELLING, ASKING_ABOUT_IT, PLANNING_IT);

/// Python: bot/memory/store_replies.py:6703 `_says_the_fact(line, key)`
///
/// Whether a past line tells the thing a denial would deny, rather than asking about it.
pub fn _says_the_fact(line: &str, key: &str) -> bool {
    if crate::memory::facts::extract(line, false).val().get(key).map(Value::truthy).unwrap_or(false) {
        return true;
    }
    let low = fold(line);
    if ASKING_ABOUT_IT.is_match(&low) || PLANNING_IT.is_match(&low) {
        return false;
    }
    if A_CLAIM_OF_TELLING.match_(&low).is_some() {
        // "i told you about the damp" carries the word and says nothing about the damp.
        return false;
    }
    true
}

static DENIAL_WORDS: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| g("DENIAL_WORDS").str_list_map());
static DENIAL_STEMS: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| g("DENIAL_STEMS").str_list_map());

/// Python: bot/memory/store_replies.py:6740 `_the_line_about(memory, key, own=None)`
///
/// The last line the person sent that says the thing a denial would deny, or "". `own`, when
/// given, says whether a line that carries the words carries the fact itself.
pub fn _the_line_about(memory: &Memory, key: &str, own: Option<fn(&str) -> bool>) -> String {
    let words = DENIAL_WORDS.get(key).cloned().unwrap_or_default();
    let stems = DENIAL_STEMS.get(key).cloned().unwrap_or_default();
    if words.is_empty() && stems.is_empty() {
        return String::new();
    }
    let mut parts: Vec<String> = words.iter().map(|w| format!(r"\b{}\b", crate::py::re::escape(w))).collect();
    parts.extend(stems.iter().map(|s| format!(r"\b{}", crate::py::re::escape(s))));
    let wanted = Re::new(&parts.join("|"), 0);
    for turn in memory.turns.iter().rev() {
        let line = &turn.user;
        let found = LETTER_WORDS.findall_str(&line.to_lowercase()).join(" ");
        if found.is_empty() {
            continue;
        }
        if !wanted.is_match(&found) {
            continue;
        }
        if crate::memory::facts::denials(line, &Value::None).val().contains_str(key) {
            continue; // a line that takes the fact back is not the fact said
        }
        // A line that asks about the thing, or plans around it, is not a line that tells it.
        if !_says_the_fact(line, key) {
            continue;
        }
        if _said_of_someone_else(line, &wanted) {
            continue;
        }
        if let Some(own) = own {
            if !own(line) {
                continue;
            }
        }
        return text::squash(line);
    }
    String::new()
}

/// Python: bot/memory/store_replies.py:6774 `_said_of_someone_else(line, wanted)`
///
/// Whether every place the line has the fact's word it is said of another person.
pub fn _said_of_someone_else(line: &str, wanted: &Re) -> bool {
    let mut kinds: Vec<String> = Vec::new();
    for m in wanted.finditer(line) {
        let at = text::char_index(line, m.start()) as i64;
        // A ValueError or IndexError from the subject reader claims nothing.
        let said = py_try(|| crate::memory::facts::subject(line, at));
        match said {
            Ok(v) if !v.is_empty() => kinds.push(v[0].clone()),
            _ => return false,
        }
    }
    !kinds.is_empty() && kinds.iter().all(|k| k == "other")
}

/// Python: bot/memory/store_replies.py:6786 `_deny(memory, key, denial, own=None)`
///
/// The denial, unless they did say it: then their own line is the honest answer.
pub fn _deny(memory: &Memory, key: &str, denial: &str, own: Option<fn(&str) -> bool>) -> String {
    let said = _the_line_about(memory, key, own);
    if !said.is_empty() {
        return "You said: \"".to_string() + &said + "\"";
    }
    denial.to_string()
}

rx!(DIET_NOT_THEIRS, DIET_NEGATED, MEAL_WANTED);
static WE_EAT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:i|we)\b[^.?!]*\b(?:eat|eats|eating|drink|drinks|diet)\w*\b", crate::py::re::I));

/// Python: bot/memory/store_replies.py:6793 `_their_own_diet(line)`
///
/// Whether the line says the diet as the person's own, now.
pub fn _their_own_diet(line: &str) -> bool {
    if crate::memory::facts::extract(line, false).val().get("diet").map(Value::truthy).unwrap_or(false)
        || crate::memory::facts::denials(line, &Value::None).val().get("diet").map(Value::truthy).unwrap_or(false)
    {
        return true;
    }
    if DIET_NOT_THEIRS.is_match(line) {
        return false;
    }
    // "i mostly eat low fat stuff" is how they eat now.
    WE_EAT.is_match(line)
}

/// Python: bot/memory/store_replies.py:6817 `_diet_words(value)`
///
/// A diet as a person would say it: "slimming world", not the store's "slimming-world".
pub fn _diet_words(value: &str) -> String {
    value.replace('-', " ")
}

/// Python: bot/memory/store_replies.py:6823 `_a_diet_put_back(shown)`
///
/// The thing they are back on, when the diet held was a negation, else None.
pub fn _a_diet_put_back(shown: &str) -> Option<String> {
    let flat = text::squash(shown);
    let bare = DIET_NEGATED.match_(&flat)?;
    let thing = text::strip(bare.str(1)).to_string();
    let words = text::split(&thing);
    if words.is_empty() || words.len() > 4 || ["more", "longer", "any", "the"].contains(&words[0].to_lowercase().as_str()) {
        return None;
    }
    Some(thing)
}

/// `user(memory)[key] = value`: a write into the user's facts, as Python's shared dict takes it.
fn user_set(memory: &mut Memory, key: &str, value: Value) {
    user(memory);
    memory.get_mut("user").unwrap().set_key(key, value);
}

/// The reading's `kind`, as `(memory.get("understood") or {}).get("kind")` gives it.
fn reading_kind(memory: &Memory) -> Option<String> {
    memory.understood.as_ref().map(|u| u.kind.clone())
}

/// Python: bot/memory/store_replies.py:6847 `_diet_reply(memory, low="")`
///
/// How they eat, said back when they ask: "You told me you're vegetarian."
pub fn _diet_reply(memory: &mut Memory, low: &str) -> Option<String> {
    // A line that reports a change to the diet, or asks for one, is not a request for the fact.
    let said = ret_str(crate::memory::change::reported(low, "diet"));
    if !said.is_empty() {
        let was = user(memory).get("diet").cloned().unwrap_or(Value::None);
        if said == "changed" && was.truthy() {
            // The reply says what it is doing with the old value, so the store has to do it.
            crate::memory::forget(memory, "diet", &Value::from(vec![was.clone()]), "");
            user_set(memory, "diet_changed", was.clone());
        }
        return ret_opt(crate::memory::change::reply(&said, "diet", &s_or(Some(&was))));
    }
    let diet = user(memory).get("diet").cloned().unwrap_or(Value::None);
    if !diet.truthy() {
        if !low.is_empty() && !DIET_FACT_ASK.is_match(low) && !_food_facts(memory).is_empty() {
            // A meal ask after they have told the store what they cannot eat is a question about the food.
            let (left_out, said_so) = _what_a_dinner_has_to_leave_out(memory);
            if !left_out.is_empty() {
                return Some(
                    "Nothing with ".to_string()
                        + &_say_a_list(&left_out)
                        + " in it \u{2014} "
                        + &said_so.join(" and ")
                        + ". Tell me what is in the fridge and I'll pick a dinner around it.",
                );
            }
        }
        let kept = _food_facts(memory);
        if !kept.is_empty() {
            // The store holds what they cannot eat, so a denial would be false.
            return Some("You told me you're ".to_string() + &_say_a_list(&kept) + ".");
        }
        if DIET_FACT_ASK.is_match(low) {
            let kept = _food_facts(memory);
            if !kept.is_empty() {
                return Some("You told me you're ".to_string() + &_say_a_list(&kept) + ".");
            }
        }
        let (who, theirs) = _other_fact(memory, "diet");
        if !theirs.is_empty() && !who.is_empty() {
            return Some("You haven't told me about yours \u{2014} the diet you mentioned is your ".to_string() + &who + "'s.");
        }
        let changed_from = user(memory).get("diet_changed").cloned().unwrap_or(Value::None);
        if changed_from.truthy() && DIET_FACT_ASK.is_match(low) {
            // The change was said and the value it named was dropped.
            let was = text::strip(&py_str(&changed_from)).to_string();
            let was = if was.to_lowercase().starts_with("no ") {
                "not eating ".to_string() + &text::slice(&was, 3, None)
            } else {
                "being ".to_string() + &_diet_words(&was)
            };
            return Some("You told me your diet had changed from ".to_string() + &was + ". What is it now?");
        }
        let (take_back, gone) = _the_take_back(memory, "diet");
        if !gone.is_empty() {
            // They took a diet back: their own take-back said back.
            return Some(_a_take_back_said_back(&take_back, &gone));
        }
        return Some(_deny(memory, "diet", "You haven't told me about your diet.", Some(_their_own_diet)));
    }
    let diet = py_str(&diet);
    if MEAL_WANTED.is_match(low) && !DIET_FACT_ASK.is_match(low) {
        // A dinner is the kitchen's to answer.
        let (left_out, said_so) = _what_a_dinner_has_to_leave_out(memory);
        if !left_out.is_empty() {
            return Some(
                "Nothing with ".to_string()
                    + &_say_a_list(&left_out)
                    + " in it \u{2014} "
                    + &said_so.join(" and ")
                    + ". Tell me what is in the fridge and I'll pick a dinner around it.",
            );
        }
        return None;
    }
    if diet.starts_with("no ") {
        return Some("You told me you don't ".to_string() + &_diet_how(&diet) + " " + &text::slice(&diet, 3, None) + ".");
    }
    if WAY_OF_EATING.contains(&diet) {
        return Some("You told me you're ".to_string() + &_diet_words(&diet) + ".");
    }
    if PLAN_DIETS.contains(&diet) {
        // A plan is something they are doing, not something they are.
        return Some("You told me you're doing ".to_string() + &_diet_words(&diet) + ".");
    }
    Some("You told me you're ".to_string() + &diet + ".")
}

rx!(AGE_COMPARE, BARE_AGE_WHO, THE_AGE_GAP, WHICH_ONE_AGE_WHO, PEOPLE_WORDS, SOMEONE_ELSE);
static THAN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bthan\b", 0));

/// Python: bot/memory/store_replies.py:6967 `_about_people(low)`
///
/// Whether a comparison with nobody named is about people at all.
pub fn _about_people(low: &str) -> bool {
    if !(BARE_AGE_WHO.match_(low).is_some() || PEOPLE_WORDS.is_match(low)) {
        return false;
    }
    if THAN.is_match(low) {
        return SOMEONE_ELSE.is_match(low);
    }
    true
}

static FACTS_KIN: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.facts.KIN"));
static MY_KIN: LazyLock<Re> = LazyLock::new(|| Re::new(&format!(r"\b(?:my|our)\s+({})(?:'s|s)?\b", FACTS_KIN.join("|")), 0));
static MY_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bmy\s+([a-z']+)", 0));
static ME_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:me|myself|mine|i|im)\b", 0));
static ME_ONLY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:me|myself|mine)\b", 0));
static YOUNGER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:younger|youngest)\b", 0));
static HOW_MUCH_OLDER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhow (?:much|many years)\s+(?:older|younger)\b", 0));
static THE_EST: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(youngest|eldest|oldest)\b", 0));

/// Python: bot/memory/store_replies.py:6982 `_age_compare(memory, low)`
///
/// Two ages the store holds, compared out loud: "You are - 34 to your wife's 32."
pub fn _age_compare(memory: &mut Memory, low: &str) -> Option<String> {
    if !AGE_COMPARE.is_match(low) {
        return None;
    }
    match reading_kind(memory).as_deref() {
        None | Some("question") => {}
        // "my wife is older than me" states the comparison instead of asking it.
        _ => return None,
    }
    let aged: Vec<Value> = others(memory)
        .items()
        .iter()
        .filter(|e| e.get("age").map(|a| !a.is_none()).unwrap_or(false) && e.get("who").map(Value::truthy).unwrap_or(false))
        .cloned()
        .collect();
    let mut by_who: Vec<(String, Value)> = Vec::new();
    for entry in &aged {
        let key = s_or(entry.get("who")).to_lowercase();
        if !by_who.iter().any(|(k, _)| *k == key) {
            by_who.push((key, entry.clone()));
        }
    }
    let find = |w: &str| by_who.iter().find(|(k, _)| k == w).map(|(_, v)| v.clone());
    // The people the line names decide the comparison.
    let words: Vec<String> = MY_KIN.findall_str(low).into_iter().map(|w| w.to_lowercase()).collect();
    if words.iter().any(|w| find(w).is_none()) {
        return None;
    }
    // "is my car older than my wife's" compares two cars.
    if MY_WORD.findall_str(low).iter().any(|w| !FACTS_KIN.contains(w)) {
        return None;
    }
    let mut named: Vec<Value> = Vec::new();
    for word in &words {
        let entry = find(word).unwrap();
        if !named.contains(&entry) {
            named.push(entry);
        }
    }
    let mine = user(memory).get("age").cloned().unwrap_or(Value::None);
    let me = if !mine.is_none() { Some(dict_of(vec![("who", Value::from("")), ("age", mine.clone())])) } else { None };
    let sides: Vec<Value> = if named.len() >= 2 {
        named[..2].to_vec()
    } else if !named.is_empty() && me.is_some() {
        vec![me.clone().unwrap(), named[0].clone()]
    } else if named.is_empty() && me.is_some() && !aged.is_empty() && _about_people(low) {
        let pointed = _pointed_at(&aged, low)?;
        vec![me.clone().unwrap(), pointed]
    } else if named.is_empty()
        && me.is_none()
        && by_who.len() == 2
        && !ME_WORDS.is_match(low)
        && (BARE_AGE_WHO.match_(low).is_some() || WHICH_ONE_AGE_WHO.match_(low).is_some())
    {
        // "who is older" names nobody: the comparison is the two people the store holds.
        by_who.iter().map(|(_, v)| v.clone()).collect()
    } else if named.is_empty() && me.is_none() && by_who.len() == 2 && THE_AGE_GAP.is_match(low) && !ME_ONLY.is_match(low) && _the_pronoun_is_one_of_the_two(&by_who, low) {
        // "how much older is he" asks for the gap between the two people the store holds.
        by_who.iter().map(|(_, v)| v.clone()).collect()
    } else {
        return None;
    };
    let mut ages: Vec<i64> = Vec::new();
    for s in &sides {
        ages.push(py_int_value(s.get("age").unwrap_or(&Value::None))?);
    }
    let (first, second) = (&sides[0], &sides[1]);
    let who_of = |v: &Value| py_str(v.get("who").unwrap_or(&Value::None));
    if ages[0] == ages[1] {
        if !first.get("who").map(Value::truthy).unwrap_or(false) {
            return Some(format!("Neither of you - you're both {}.", ages[0]));
        }
        return Some(format!("Neither of them - your {} and your {} are both {}.", who_of(first), who_of(second), ages[0]));
    }
    let younger = YOUNGER.is_match(low);
    let (winner, other) = if (ages[0] > ages[1]) != younger { (0, 1) } else { (1, 0) };
    let (win, lose) = (&sides[winner], &sides[other]);
    let gap = (ages[0] - ages[1]).abs();
    let years = if gap == 1 { "year" } else { "years" };
    if HOW_MUCH_OLDER.is_match(low) {
        if !first.get("who").map(Value::truthy).unwrap_or(false) {
            return Some(format!("{gap} {years}: you're {} and your {} is {}.", ages[0], who_of(second), ages[1]));
        }
        return Some(format!("{gap} {years}: your {} is {} and your {} is {}.", who_of(first), ages[0], who_of(second), ages[1]));
    }
    let theirs = if !lose.get("who").map(Value::truthy).unwrap_or(false) { "your".to_string() } else { "your ".to_string() + &who_of(lose) + "'s" };
    let word = match THE_EST.search(low) {
        Some(said) => "the ".to_string() + said.str(1),
        None => if ages[winner] > ages[other] { "older" } else { "younger" }.to_string(),
    };
    if !win.get("who").map(Value::truthy).unwrap_or(false) {
        return Some(format!("You're {word} - {} to {theirs} {}.", ages[winner], ages[other]));
    }
    Some(format!("Your {}'s {word} - {} to {theirs} {}.", who_of(win), ages[winner], ages[other]))
}

rx!(WHICH_FIRST_ASK, MONTH_SAID);
strset!(WHICH_FIRST_WORDS);
strs!(MONTH_ORDER);

/// Python: bot/memory/store_replies.py:7088 `_the_day_a_plan_names(plan)`
///
/// How many days away the day a plan names is, or None when it names no day.
pub fn _the_day_a_plan_names(plan: &str) -> Option<i64> {
    use chrono::Datelike;
    let low = fold(plan);
    let today = crate::py::clock::today();
    if let Some(found) = MONTH_SAID.search(&low) {
        let short = MONTH_WORD.search(found.as_str());
        let name: Vec<&String> = match &short {
            Some(s) => {
                let head = text::slice(s.as_str(), 0, Some(3));
                MONTH_ORDER.iter().filter(|m| m.starts_with(&head)).collect()
            }
            None => Vec::new(),
        };
        let name = (*name.first()?).clone();
        let index = MONTH_ORDER.iter().position(|m| *m == name).unwrap() as u32 + 1;
        let mut day = 1u32;
        let escaped = crate::py::re::escape(&name);
        let near = Re::new(&format!(r"\b(\d{{1,2}})(?:st|nd|rd|th)?\b[^.?!]{{0,12}}{escaped}"), 0)
            .search(&low)
            .map(|m| m.str(1).to_string())
            .or_else(|| Re::new(&format!(r"{escaped}\b[^.?!]{{0,12}}?\b(\d{{1,2}})(?:st|nd|rd|th)?\b"), 0).search(&low).map(|m| m.str(1).to_string()));
        if let Some(n) = near.and_then(|n| py_int(&n)) {
            if (1..=31).contains(&n) {
                day = n as u32;
            }
        }
        for year in [today.year(), today.year() + 1] {
            let Some(when) = chrono::NaiveDate::from_ymd_opt(year, index, day) else { continue };
            if when >= today {
                return Some((when - today).num_days());
            }
        }
        return None;
    }
    if let Some(day) = WEEKDAY_WORD.search(&low) {
        if !DATE_PHRASE.is_match(&low) {
            let at = WEEK.iter().position(|d| d == day.as_str()).expect("ValueError: not in list") as i64;
            return Some((at - today.weekday().num_days_from_monday() as i64).rem_euclid(7));
        }
    }
    None
}

/// Python: bot/memory/store_replies.py:7130 `_the_plans_in_date_order(memory)`
///
/// The things the store holds a day for, earliest first, each said the way it was kept.
pub fn _the_plans_in_date_order(memory: &mut Memory) -> Vec<(i64, String)> {
    let known = user(memory);
    let mut held = _plans_with_the_latest(&known);
    for entry in others(memory).items() {
        let who = text::strip(&sget(entry, "who")).to_string();
        let day = text::strip(&sget(entry, "birthday")).to_string();
        if who.is_empty() || day.is_empty() {
            continue;
        }
        // A birthday that is also a plan is in the store twice, and the plan is the way they said it.
        let whose = Re::new(&format!(r"\b{}s?\b", crate::py::re::escape(&who)), 0);
        if held.iter().any(|p| p.to_lowercase().contains("birthday") && whose.is_match(&p.to_lowercase())) {
            continue;
        }
        // "3 may" takes "on", a month on its own takes "in".
        let link = if A_DIGIT.is_match(&day) { "on " } else { "in " };
        held.push("my ".to_string() + &who + "s birthday is " + link + &day);
    }
    let mut dated: Vec<(i64, String)> = Vec::new();
    let mut seen: HashSet<(i64, String)> = HashSet::new();
    for plan in &held {
        let Some(days) = _the_day_a_plan_names(plan) else { continue };
        let line = crate::memory::plan_line(memory, plan);
        let said = text::rstrip_chars(&crate::memory::plan_sentence(plan, &line), " .!?").to_string();
        if said.is_empty() {
            continue;
        }
        // The same day said twice is one day in the order.
        let key = (days, said.to_lowercase());
        if seen.contains(&key) {
            continue;
        }
        seen.insert(key);
        dated.push((days, said));
    }
    dated.sort_by_key(|pair| pair.0);
    dated
}

/// Python: bot/memory/store_replies.py:7175 `_which_comes_first(memory, low)`
///
/// Two dated plans in the order the calendar puts them: "May comes first, then June".
pub fn _which_comes_first(memory: &mut Memory, low: &str) -> Option<String> {
    let found = WHICH_FIRST_ASK.search(low)?;
    match reading_kind(memory).as_deref() {
        None | Some("question") => {}
        _ => return None,
    }
    let rest = low[..found.start()].to_string() + " " + &low[found.end()..];
    if LETTER_WORDS.findall_str(&rest).iter().any(|w| !WHICH_FIRST_WORDS.contains(w)) {
        return None;
    }
    let dated = _the_plans_in_date_order(memory);
    if dated.len() < 2 {
        return None;
    }
    let first = dated[0].1.clone();
    let later: Vec<String> = dated[1..dated.len().min(4)].iter().map(|(_, s)| lower_first(s)).collect();
    if dated[1].0 == dated[0].0 {
        return Some("Both land on the same day: ".to_string() + &lower_first(&first) + " and " + &later[0] + ".");
    }
    Some(first + ", so that one is first \u{2014} then " + &later.join(", then ") + ".")
}

static SHE_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:she|her|hers)\b", 0));
static HE_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:he|him|his)\b", 0));

/// Python: bot/memory/store_replies.py:7206 `_the_pronoun_is_one_of_the_two(by_who, low)`
///
/// Whether a pronoun in the line fits one of the two people the store holds an age for.
pub fn _the_pronoun_is_one_of_the_two(by_who: &[(String, Value)], low: &str) -> bool {
    let fits: &HashSet<String> = if SHE_WORDS.is_match(low) {
        &FACTS_SHE
    } else if HE_WORDS.is_match(low) {
        &FACTS_HE
    } else {
        return true;
    };
    by_who.iter().any(|(_, e)| fits.contains(&s_or(e.get("who")).to_lowercase()))
}

/// Python: bot/memory/store_replies.py:7223 `_pointed_at(aged, low)`
///
/// The person an age comparison points at with a pronoun, or the last one heard of.
pub fn _pointed_at(aged: &[Value], low: &str) -> Option<Value> {
    let fits: Option<&HashSet<String>> = if SHE_WORDS.is_match(low) {
        Some(&FACTS_SHE)
    } else if HE_WORDS.is_match(low) {
        Some(&FACTS_HE)
    } else {
        None
    };
    if let Some(fits) = fits {
        return aged.iter().rev().find(|e| fits.contains(&s_or(e.get("who")).to_lowercase())).cloned();
    }
    Some(aged.last().expect("IndexError: list index out of range").clone())
}

rx!(LIMIT_LIKE);
strset!(MORE_FOODS);
static COOKING_FAMILIES: LazyLock<Vec<(String, Vec<String>)>> =
    LazyLock::new(|| data::get("bot.knowledge.cooking.FAMILIES").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect());
static COOKING_CHEESES: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.cooking.CHEESES"));
static COOKING_FOOD: LazyLock<Re> = LazyLock::new(|| Re::new(&format!(r"\b{}\b", data::string("bot.knowledge.cooking.FOOD")), 0));

/// Python: bot/memory/store_replies.py:7251 `_a_food(value)`
///
/// Whether the thing they can't have is food: "nuts" is, "penicillin" is not.
pub fn _a_food(value: &str) -> bool {
    let word = text::strip(value).to_lowercase();
    if word.is_empty() {
        return false;
    }
    if MORE_FOODS.contains(&word) {
        return true;
    }
    if COOKING_FAMILIES.iter().any(|(_, family)| family.contains(&word)) {
        return true;
    }
    COOKING_CHEESES.contains(&word) || COOKING_FOOD.is_match(&word)
}

static DIET_FOOD: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| g("DIET_FOOD").str_list_map());

/// Python: bot/memory/store_replies.py:7265 `_diet_food(diet)`
///
/// The food a diet rules out, or () for a diet the table does not know.
pub fn _diet_food(diet: &str) -> Vec<String> {
    let low = text::strip(diet).to_lowercase();
    if let Some(food) = DIET_FOOD.get(&low) {
        return food.clone();
    }
    if low.starts_with("no ") {
        // "no meat" is the limit said as the food already.
        let rest = text::strip(&text::slice(&low, 3, None)).to_string();
        return if rest.is_empty() { Vec::new() } else { vec![rest] };
    }
    if low.ends_with(" free") {
        let rest = text::strip(&text::slice(&low, 0, Some(-5))).to_string();
        return if rest.is_empty() { Vec::new() } else { vec![rest] };
    }
    Vec::new()
}

/// Python: bot/memory/store_replies.py:7277 `_said_a_limit(memory)`
pub fn _said_a_limit(memory: &mut Memory) -> bool {
    let known = user(memory);
    if ["allergies", "condition", "diet"].iter().any(|k| known.get(k).map(Value::truthy).unwrap_or(false)) {
        return true;
    }
    recent_turns(memory, 12).iter().any(|t| !t.user.is_empty() && LIMIT_LIKE.is_match(&t.user))
}

/// Python: bot/memory/store_replies.py:7287 `_said_list(items)`
///
/// Two things are joined with "and"; three or more take commas and a final "and".
pub fn _said_list(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        2 => items[0].clone() + " and " + &items[1],
        n => items[..n - 1].join(", ") + " and " + &items[n - 1],
    }
}

strset!(FOOD_CONDITION);

/// A str or a list of them, as the store keeps a fact that can be either.
fn str_or_list(v: Option<&Value>) -> Option<Vec<Value>> {
    match v {
        Some(Value::Str(s)) => Some(vec![Value::Str(s.clone())]),
        Some(Value::List(items)) | Some(Value::Tuple(items)) => Some(items.to_vec()),
        _ => None,
    }
}

/// Python: bot/memory/store_replies.py:12751 `_food_facts(memory)`
///
/// The food restrictions the store holds, in the words they can be said back in.
pub fn _food_facts(memory: &mut Memory) -> Vec<String> {
    let mine = user(memory);
    let mut kept = Vec::new();
    if let Some(allergies) = str_or_list(mine.get("allergies")) {
        for item in allergies {
            let item = text::strip(&py_str(&item)).to_string();
            if !item.is_empty() {
                kept.push("allergic to ".to_string() + &item);
            }
        }
    }
    if let Some(condition) = str_or_list(mine.get("condition")) {
        for item in condition {
            let item = text::strip(&py_str(&item)).to_string();
            if !item.is_empty() && FOOD_CONDITION.contains(&item.to_lowercase()) {
                kept.push(item);
            }
        }
    }
    kept
}

static CONDITION_BANS: LazyLock<HashMap<String, HashSet<String>>> = LazyLock::new(|| {
    data::get("bot.knowledge.cooking.CONDITION_BANS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_set())).collect()
});

/// Python: bot/memory/store_replies.py:12773 `_the_food_a_condition_rules_out(condition)`
///
/// The one food word a condition rules out ("coeliac" -> "gluten"), or "".
pub fn _the_food_a_condition_rules_out(condition: &str) -> String {
    let Some(wanted) = CONDITION_BANS.get(text::strip(&condition.to_lowercase())).filter(|w| !w.is_empty()) else { return String::new() };
    for (name, family) in COOKING_FAMILIES.iter() {
        let family: HashSet<String> = family.iter().cloned().collect();
        if family == *wanted {
            return name.clone();
        }
    }
    String::new()
}

/// Python: bot/memory/store_replies.py:12788 `_what_a_dinner_has_to_leave_out(memory)`
///
/// The foods a dinner has to leave out, and the fact that rules each one out.
pub fn _what_a_dinner_has_to_leave_out(memory: &mut Memory) -> (Vec<String>, Vec<String>) {
    let mine = user(memory);
    let (mut left_out, mut said_so): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    if let Some(allergies) = str_or_list(mine.get("allergies")) {
        let kept: Vec<String> = allergies.iter().map(|n| text::strip(&py_str(n)).to_string()).filter(|n| !n.is_empty()).collect();
        if !kept.is_empty() {
            said_so.push("you're allergic to ".to_string() + &_all_of_them(kept.len() as i64));
            left_out.extend(kept);
        }
    }
    if let Some(condition) = str_or_list(mine.get("condition")) {
        for item in condition {
            let named = text::strip(&py_str(&item)).to_string();
            if named.is_empty() || !FOOD_CONDITION.contains(&named.to_lowercase()) {
                continue;
            }
            let food = _the_food_a_condition_rules_out(&named);
            said_so.push("you're ".to_string() + &named);
            if !food.is_empty() && !left_out.contains(&food) {
                left_out.push(food);
            }
        }
    }
    (left_out, said_so)
}

/// Python: bot/memory/store_replies.py:12820 `_all_of_them(count)`
///
/// How a person says the whole list back: one thing, two things, or three and more.
pub fn _all_of_them(count: i64) -> String {
    match count {
        1 => "them",
        2 => "both",
        _ => "all of them",
    }
    .to_string()
}

/// Python: bot/memory/store_replies.py:12834 `_say_a_list(items)`
///
/// One thing named, or several as people say them: "a, b and c".
pub fn _say_a_list(items: &[String]) -> String {
    match items.len() {
        1 => items[0].clone(),
        2 => items[0].clone() + " and " + &items[1],
        n => {
            let last = items.last().expect("IndexError: list index out of range");
            items[..n - 1].join(", ") + " and " + last
        }
    }
}

/// Python: bot/memory/store_replies.py:7299 `_limit_reply(memory, low="")`
///
/// What they can't have, said back: "what can't i eat", "what should i avoid".
pub fn _limit_reply(memory: &mut Memory, low: &str) -> Option<String> {
    let known = user(memory);
    let allergies: Vec<String> =
        known.get("allergies").filter(|v| v.truthy()).map(py_list).unwrap_or_default().iter().map(py_str).filter(|a| !text::strip(a).is_empty()).collect();
    let conditions: Vec<String> = match known.get("condition").filter(|v| v.truthy()) {
        Some(Value::Str(s)) => vec![s.to_string()],
        Some(v) => py_list(v).iter().map(py_str).collect(),
        None => Vec::new(),
    }
    .into_iter()
    .filter(|c| !text::strip(c).is_empty())
    .collect();
    let diet = text::strip(&sget(&known, "diet")).to_string();
    // Only a food can be said back as one of the things they can't have.
    let foods: Vec<String> = allergies.iter().filter(|a| _a_food(a)).cloned().collect();
    if foods.is_empty() && conditions.is_empty() && diet.is_empty() {
        // Nothing is stored, so there is no limit to say back.
        if !FOOD_ASK.is_match(low) {
            return None;
        }
        // Unless they did tell us something a moment ago.
        if _said_a_limit(memory) {
            return None;
        }
        return Some("You haven't told me anything you can't have. Say what to keep out of your food and I'll stick to it.".to_string());
    }
    // Every limit the store keeps is about food, so it is said as the food it rules out.
    let mut clauses: Vec<String> = Vec::new();
    if !foods.is_empty() {
        let it = if foods.len() > 1 || foods[0].ends_with('s') { "them" } else { "it" };
        clauses.push("nothing with ".to_string() + &_said_list(&foods) + " in it \u{2014} you're allergic to " + it);
    }
    if !conditions.is_empty() {
        // A condition that rules a food out is said as the food.
        let ruled: Vec<Option<String>> = conditions.iter().map(|c| CONDITION_FOOD.get(&c.to_lowercase()).cloned()).collect();
        if !ruled.is_empty() && ruled.iter().all(|r| r.as_deref().map(|s| !s.is_empty()).unwrap_or(false)) {
            let ruled: Vec<String> = ruled.into_iter().map(Option::unwrap).collect();
            clauses.push("nothing with ".to_string() + &_said_list(&ruled) + " in it \u{2014} you're " + &_said_list(&conditions));
        } else {
            clauses.push("you're ".to_string() + &_said_list(&conditions));
        }
    }
    if !diet.is_empty() {
        let ruled = _diet_food(&diet);
        // "no meat" is already the food, so that shape keeps the wording it had.
        if !ruled.is_empty() && ruled.iter().all(|r| !r.is_empty()) && !diet.to_lowercase().starts_with("no ") {
            clauses.push("nothing with ".to_string() + &_said_list(&ruled) + " in it \u{2014} you're " + &diet);
        } else if diet.starts_with("no ") {
            clauses.push("you don't ".to_string() + &_diet_how(&diet) + " " + &text::slice(&diet, 3, None));
        } else {
            clauses.push("you're ".to_string() + &diet);
        }
    }
    let mut said = clauses[0].clone();
    if clauses.len() == 2 {
        said += &(", and ".to_string() + &clauses[1]);
    } else if clauses.len() > 2 {
        said += &(", ".to_string() + &clauses[1..clauses.len() - 1].join(", ") + ", and " + &clauses[clauses.len() - 1]);
    }
    let said = text::upper_first(&said) + ".";
    if crate::memory::relations::_same_as_last(&said, memory) {
        // The same sentence twice in a row reads like a machine: the limit is said afresh.
        let rest = lower_first(&said);
        return Some(_vary(
            &["Still this: ".to_string() + &rest, "That hasn't changed: ".to_string() + &rest, "Same answer: ".to_string() + &rest],
            memory,
        ));
    }
    Some(said)
}

rx!(AN_ASK_AT_THE_END, HEDGED);

/// Python: bot/memory/store_replies.py:7386 `_the_answer_the_ask_gets(text, memory)`
///
/// What the cards say to this line as a question, or "" when they have nothing to say.
pub fn _the_answer_the_ask_gets(text_: &str, memory: &Memory) -> String {
    let mut fresh = Memory::new();
    fresh.set("aside", true);
    for key in ["user", "others", "facts_log"] {
        if let Some(v) = memory.get(key) {
            fresh.set(key, v.clone());
        }
    }
    // A store that cannot ask is a store that says the fact.
    let reply = or_on_raise(|| crate::modules::answers::respond(text_, &mut fresh).unwrap_or_default(), String::new());
    let said = text::squash(&reply);
    if said.is_empty() || HEDGED.is_match(&said) {
        return String::new();
    }
    said
}

/// Python: bot/memory/store_replies.py:7407 `_the_ask_beats_the_receipt(text, memory)`
///
/// True when a line that ends on a question is a question the cards can answer.
pub fn _the_ask_beats_the_receipt(text_: &str, memory: &Memory) -> bool {
    if text_.contains('?') {
        return false;
    }
    if !AN_ASK_AT_THE_END.is_match(&text::squash(text_)) {
        return false;
    }
    !_the_answer_the_ask_gets(text_, memory).is_empty()
}

/// Python: bot/memory/store_replies.py:7421 `_a_diet_told(text, memory, low)`
///
/// "she doesnt eat meat" said about someone: "Your mum doesn't eat meat."
pub fn _a_diet_told(text_: &str, memory: &mut Memory, low: &str) -> Option<String> {
    if text_.contains('?') {
        return None;
    }
    if _the_ask_beats_the_receipt(text_, memory) {
        // The line ends on a question the cards can answer, so the fact is only what it is about.
        return None;
    }
    if _about_this_chat(text_, memory) || OTHER_ASK.is_match(low) || REMINDER_MID.is_match(low) {
        return None;
    }
    let found = crate::memory::facts::extract(text_, false).val();
    if ["diet", "no_meat", "allergies"].iter().any(|k| found.get(k).map(Value::truthy).unwrap_or(false)) {
        return None;
    }
    let named: Vec<String> = found.get("people").map(py_list).unwrap_or_default().iter().filter_map(|w| w.as_str().map(str::to_string)).collect();
    for entry in found.get("others").map(py_list).unwrap_or_default() {
        let diet = entry.get("diet").cloned().unwrap_or(Value::None);
        if !diet.truthy() {
            continue;
        }
        let who = match entry.get("who").filter(|w| w.truthy()) {
            Some(w) => py_str(w),
            None => crate::memory::relations::_person_from_pronoun(text_, memory, &named),
        };
        return Some(_diet_sentence(&who, &diet));
    }
    None
}

/// Python: bot/memory/store_replies.py:7451 `_object_pronoun(who, known=None)`
///
/// The pronoun a person takes as an object: "her", "him" or "them".
pub fn _object_pronoun(who: &str, known: &Value) -> String {
    match ret_str(crate::memory::relations::_pronoun_for(who, known)).as_str() {
        "she" => "her".into(),
        "he" => "him".into(),
        _ => "them".into(),
    }
}

static PRONOUN_FITS: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| g("PRONOUN_FITS").str_list_map());
static TITLE_GENDER: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.memory.facts.TITLE_GENDER"));

/// Python: bot/memory/store_replies.py:7456 `_who_the_pronoun_means(low, memory, pronoun)`
///
/// The last person the chat named whose pronoun fits, for "whats his name".
pub fn _who_the_pronoun_means(low: &str, memory: &mut Memory, pronoun: &str) -> String {
    let fits = PRONOUN_FITS.get(&pronoun.to_lowercase()).cloned().unwrap_or_default();
    let mut allowed: HashSet<String> = HashSet::new();
    for key in &fits {
        allowed.extend(if key == "he" { FACTS_HE.iter().cloned() } else { FACTS_SHE.iter().cloned() }.collect::<Vec<_>>());
    }
    if allowed.is_empty() {
        return String::new();
    }
    let mut people: Vec<String> = user(memory).get("people").map(py_list).unwrap_or_default().iter().filter_map(|w| w.as_str().map(str::to_string)).collect();
    people.extend(others(memory).items().iter().filter(|e| e.get("who").map(Value::truthy).unwrap_or(false)).map(|e| py_str(e.get("who").unwrap())));
    if let Some(who) = people.iter().rev().find(|w| allowed.contains(*w)) {
        return who.clone();
    }
    // A role can be held by anybody: the title the chat was given says it, and with one person on
    // the table the pronoun means them whatever their word says.
    let record = crate::memory::topics::about(memory);
    if !record.truthy() {
        return String::new();
    }
    let who = sget(&record, "who");
    let mut titled = String::new();
    for entry in others(memory).items() {
        if sget(entry, "who") != who {
            continue;
        }
        let name = sget(entry, "name");
        let title = text::strip_chars(name.split(' ').next().unwrap_or(""), ".").to_lowercase();
        if let Some(t) = TITLE_GENDER.get(&title) {
            titled = t.clone();
        }
    }
    if !titled.is_empty() {
        // The title says it plainly, so it settles the pronoun both ways.
        return if fits.contains(&titled) { who } else { String::new() };
    }
    let distinct: HashSet<&String> = people.iter().filter(|p| !p.is_empty()).collect();
    if !who.is_empty() && !FACTS_SHE.contains(&who) && !FACTS_HE.contains(&who) && distinct.len() == 1 {
        return who;
    }
    String::new()
}

static KIN_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.KIN_WORDS"));
static ALONE_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.ALONE_PERSON"));
strset!(A_ONE_PERSON_WORD, PLURAL_PEOPLE);

/// The named group of `m` that matched first, in the order given ("" when none did).
fn first_group(m: &crate::py::re::Match, names: &[&str]) -> String {
    names.iter().find_map(|g| m.name(g).filter(|s| !s.is_empty())).unwrap_or("").to_string()
}

/// Python: bot/memory/store_replies.py:7498 `_a_relations_name_reply(memory, low, text="")`
///
/// What someone in the family is called: "Your sister is called Meg."
pub fn _a_relations_name_reply(memory: &mut Memory, low: &str, text_: &str) -> Option<String> {
    use crate::memory::relations::{_a_list_of_names, _called, _canon_relation, _relations_of};
    let who: String;
    if let Some(m) = OTHER_NAME_ASK.search(low) {
        let mut w = first_group(&m, &["who1", "who2", "who3", "who4", "who5", "who6", "who7", "who8", "who9", "who10", "who11"]).to_lowercase();
        // "my babys name": the "s" on the end is the possessive.
        if w.ends_with('s') {
            let bare = text::slice(&w, 0, Some(-1));
            if KIN_WORDS.contains(&bare) || ALONE_PERSON.contains(&bare) {
                w = bare;
            }
        }
        let w = _with_an_in_law(&w, low);
        let group = _the_group(&w);
        if !group.is_empty() {
            if A_ONE_PERSON_WORD.contains(&w) {
                // "what is my child called" names one person with a word that covers several.
                if let Some(narrowed) = _which_one_of_them(memory, &w) {
                    return Some(narrowed);
                }
            }
            // "what are my children called": every name the store holds for that group.
            if let Some(together) = _what_the_group_is_called(&group, memory) {
                return Some(together);
            }
            // A line that named the group is one the store failed to file.
            if crate::memory::relations::_the_group_was_said(&w, memory) {
                return None;
            }
            let word = py_str(&crate::memory::relations::_the_group_word(&w).val());
            return Some(_vary(
                &[
                    "You haven't told me what your ".to_string() + &word + " are called.",
                    "I don't have any names for your ".to_string() + &word + " yet.",
                    "You've not said what your ".to_string() + &word + " are called.",
                ],
                memory,
            ));
        }
        if !KIN_WORDS.contains(&w) && !ALONE_PERSON.contains(&w) && !crate::memory::facts::an_in_law(&w) {
            return None;
        }
        // A wide kin word is only a person the chat knows of once it has heard of them.
        if !FACTS_RELATIONS.contains(&w) && !crate::memory::facts::an_in_law(&w) && !crate::memory::relations::_a_person_in_the_store(memory, &w) {
            return None;
        }
        who = w;
    } else {
        let point = PRONOUN_NAME_ASK.search(low)?;
        let pronoun = first_group(&point, &["pos", "sub", "obj", "pos2", "sub2", "obj2", "pos3", "pos4"]);
        if ["their", "they", "them"].contains(&pronoun.as_str()) {
            // "what are their names" about the children the chat has just named.
            if let Some(together) = _everyone_they_mean(memory) {
                return Some(together);
            }
        }
        let w = _who_the_pronoun_means(low, memory, &pronoun).to_lowercase();
        if w.is_empty() {
            // The pronoun points at the one animal the chat has named.
            if let Some(named_pet) = _the_one_animal_with_a_name(memory) {
                return Some(named_pet);
            }
            return None;
        }
        who = w;
    }
    let told = others(memory);
    let told = told.items();
    let mine: Vec<&Value> = told.iter().filter(|e| sget(e, "who").to_lowercase() == who).collect();
    let named: Vec<&&Value> = mine.iter().filter(|e| e.get("name").map(Value::truthy).unwrap_or(false)).collect();
    if let Some(last) = named.last() {
        let name = called_of(last.get("name"));
        return Some(_vary(
            &[
                "Your ".to_string() + &who + " is called " + &name + ".",
                name.clone() + " \u{2014} your " + &who + ", you told me.",
                "You said your ".to_string() + &who + " is called " + &name + ".",
            ],
            memory,
        ));
    }
    // A pair of names said for two people does not say which name is whose.
    for entry in told.iter() {
        let entry_who = sget(entry, "who");
        let pair: Vec<String> = entry_who.to_lowercase().split(" and ").map(|w| text::strip(w).to_string()).collect();
        let pair_names: Vec<String> = entry.get("names").filter(|v| v.truthy()).map(py_list).unwrap_or_default().iter().map(py_str).collect();
        let canon = _canon_relation(&who).val();
        let relations = _relations_of(&py_str(entry.get("who").unwrap_or(&Value::None))).val();
        let canon_in = relations.items().iter().any(|r| *r == canon);
        if ((pair.len() > 1 && pair.contains(&who)) || canon_in) && pair_names.len() > 1 {
            return Some("You told me your ".to_string() + &py_str(entry.get("who").unwrap()) + " are " + &_a_list_of_names(&pair_names) + " - which one is your " + &who + "?");
        }
    }
    let in_law_of_the_same = told.iter().any(|e| {
        let w = s_or(e.get("who"));
        crate::memory::facts::an_in_law(&w) && w.to_lowercase().starts_with(&(who.clone() + " in law"))
    });
    let people: Vec<String> = user(memory).get("people").map(py_list).unwrap_or_default().iter().map(|p| py_str(p).to_lowercase()).collect();
    if !mine.is_empty() || in_law_of_the_same || people.contains(&who) {
        // The person is known to the chat even with no name filed for them.
        return Some("You haven't told me your ".to_string() + &who + "'s name.");
    }
    None
}

static PERSON_GROUP: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| g("PERSON_GROUP").str_list_map());
static CHILD_OF: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("CHILD_OF").str_map());
strs!(CHILD_RELATIONS);

/// Python: bot/memory/store_replies.py:7613 `_the_group(who)`
///
/// The people a group word names, or nothing: "children", "childrens", "kids", "boys".
pub fn _the_group(who: &str) -> Vec<String> {
    let word = py_str(&crate::memory::relations::_the_group_word(who).val());
    PERSON_GROUP.get(&word).cloned().unwrap_or_default()
}

/// Python: bot/memory/store_replies.py:7621 `_in_group(who, group)`
///
/// Whether what the store filed a person under is in a group of relations.
pub fn _in_group(who: &str, group: &[String]) -> bool {
    let who = who.to_lowercase();
    let has = |w: &str| group.iter().any(|g| g == w);
    has(&who) || has(CHILD_OF.get(&who).map(String::as_str).unwrap_or("")) || (who.ends_with('s') && has(&text::slice(&who, 0, Some(-1))))
}

/// Python: bot/memory/store_replies.py:7627 `_the_people(entries)`
///
/// The people as one phrase: "daughter and son", "sons", "children".
pub fn _the_people(entries: &[Value]) -> String {
    let mut words: Vec<String> = Vec::new();
    let mut counts: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        let who = sget(entry, "who").to_lowercase();
        *counts.entry(who.clone()).or_insert(0) += 1;
        if !words.contains(&who) {
            words.push(who);
        }
    }
    let spoken: Vec<String> = words.into_iter().map(|w| if counts[&w] > 1 && !w.ends_with('s') { w + "s" } else { w }).collect();
    _and_list(&spoken)
}

/// Python: bot/memory/store_replies.py:7648 `_which_one_of_them(memory, who)`
///
/// The several people a one-person word covers, when the store cannot say which is meant.
pub fn _which_one_of_them(memory: &mut Memory, who: &str) -> Option<String> {
    use crate::memory::relations::{_a_list_of_names, _called, _in_the_group, _relations_of};
    let group = _relations_of(who).val();
    let mine: Vec<Value> = others(memory).items().iter().filter(|e| e.is_dict() && _in_the_group(&s_or(e.get("who")), &group)).cloned().collect();
    let mut said: Vec<String> = Vec::new();
    for entry in &mine {
        let names: Vec<String> = entry.get("names").filter(|v| v.truthy()).map(py_list).unwrap_or_default().iter().map(py_str).filter(|n| !text::strip(n).is_empty()).collect();
        if names.len() > 1 {
            said.extend(names.iter().map(|n| _called(n)));
        } else if entry.get("name").map(Value::truthy).unwrap_or(false) {
            said.push(_called(&py_str(entry.get("name").unwrap())));
        }
    }
    if said.len() < 2 {
        return None;
    }
    Some("You told me your ".to_string() + &_the_people(&mine) + " are " + &_a_list_of_names(&said) + " - which one do you mean?")
}

/// Python: bot/memory/store_replies.py:7670 `_what_the_group_is_called(group, memory)`
///
/// The names of the people a group word covers: "Emma and Jack - your daughter and son."
pub fn _what_the_group_is_called(group: &[String], memory: &mut Memory) -> Option<String> {
    use crate::memory::relations::_called;
    let mine: Vec<Value> = others(memory).items().iter().filter(|e| e.is_dict() && _in_group(&s_or(e.get("who")), group)).cloned().collect();
    if mine.is_empty() {
        return None;
    }
    let has_names = |e: &Value| e.get("name").map(Value::truthy).unwrap_or(false) || e.get("names").map(Value::truthy).unwrap_or(false);
    let named: Vec<Value> = mine.iter().filter(|e| has_names(e)).cloned().collect();
    let unnamed: Vec<Value> = mine.iter().filter(|e| !has_names(e)).cloned().collect();
    if named.is_empty() {
        // "my kids are 4, 7 and 9" is several people filed as one entry.
        let many = mine.len() > 1 || mine.iter().any(|e| PLURAL_PEOPLE.contains(&sget(e, "who").to_lowercase()));
        return Some(
            "You've told me about your ".to_string() + &_the_people(&mine) + ", not what " + if many { "they are" } else { "he or she is" } + " called.",
        );
    }
    let mut said: Vec<String> = Vec::new();
    let mut paired = false;
    for entry in &named {
        let names: Vec<String> = entry.get("names").filter(|v| v.truthy()).map(py_list).unwrap_or_default().iter().map(py_str).filter(|n| !text::strip(n).is_empty()).collect();
        if names.len() > 1 {
            said.extend(names.iter().map(|n| _called(n)));
            paired = true;
        } else {
            let name = match entry.get("name").filter(|v| v.truthy()) {
                Some(n) => py_str(n),
                None => names.first().cloned().expect("IndexError: list index out of range"),
            };
            said.push(_called(&name));
        }
    }
    let names = _and_list(&said);
    if paired && unnamed.is_empty() {
        // One fact about two people: both names are said back.
        return Some("You told me your ".to_string() + &_the_people(&mine) + " are called " + &names + ".");
    }
    if unnamed.is_empty() {
        if said.len() > 1 {
            return Some(names + " \u{2014} your " + &_the_people(&mine) + ".");
        }
        return Some(names + ", your " + &_the_people(&mine) + ".");
    }
    Some(names + ", and your " + &_the_people(&unnamed) + " you haven't named.")
}

/// Python: bot/memory/store_replies.py:7711 `_everyone_they_mean(memory)`
///
/// "What are their names" after two children were named, and nothing else was: them.
pub fn _everyone_they_mean(memory: &mut Memory) -> Option<String> {
    let told: Vec<Value> = others(memory).items().iter().filter(|e| e.is_dict()).cloned().collect();
    if told.len() < 2 || !told.iter().all(|e| _in_group(&s_or(e.get("who")), &CHILD_RELATIONS)) {
        return None;
    }
    _what_the_group_is_called(&CHILD_RELATIONS, memory)
}

/// `CONCEPTS`: a word -> the everyday thing it names, in Python's order.
static CONCEPTS: LazyLock<Vec<(String, String)>> =
    LazyLock::new(|| g("CONCEPTS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect());

/// `CONCEPTS.get(word)`.
fn concept(word: &str) -> Option<&'static str> {
    CONCEPTS.iter().find(|(k, _)| k == word).map(|(_, v)| v.as_str())
}

/// `CONCEPTS.get(word, word)`.
fn concept_or(word: &str) -> String {
    concept(word).map(str::to_string).unwrap_or_else(|| word.to_string())
}

/// Python: bot/memory/store_replies.py:7723 `_who_is_the_name_reply(memory, low)`
///
/// The person a name they gave belongs to: "Meg is your sister."
pub fn _who_is_the_name_reply(memory: &mut Memory, low: &str) -> Option<String> {
    use crate::memory::relations::{_a_list_of_names, _relations_of, _tidy_name};
    let m = WHO_IS_NAME_ASK.search(low)?;
    let part = |g: &str| m.name(g).filter(|s| !s.is_empty()).map(str::to_string);
    let said = text::strip(&part("name1").or_else(|| part("name2")).or_else(|| part("name3")).or_else(|| part("name4")).unwrap_or_default()).to_lowercase();
    let mut asked_who = text::strip(&part("who3").or_else(|| part("who4")).unwrap_or_default()).to_lowercase();
    if asked_who.ends_with('s') && FACTS_RELATIONS.contains(&text::slice(&asked_who, 0, Some(-1))) {
        asked_who = text::slice(&asked_who, 0, Some(-1));
    }
    if said.is_empty() {
        return None;
    }
    if part("name1").is_some() || part("name2").is_some() {
        // Only the name on its own asks about the chat.
        let group = if part("name1").is_some() { "name1" } else { "name2" };
        let end = m.name_span(group).unwrap().1;
        let tail = text::strip_chars(&low[end..], " \t?.,!;:'\"").to_string();
        if !tail.is_empty() && !text::split(&tail).iter().all(|w| NAME_ASK_TAIL.contains(w)) {
            return None;
        }
    }
    let told = others(memory);
    for entry in told.items().iter().rev() {
        if !entry.is_dict() {
            continue;
        }
        let name = text::strip(&sget(entry, "name")).to_string();
        let who = text::strip(&sget(entry, "who")).to_lowercase();
        if name.is_empty() || who.is_empty() || name.to_lowercase() != said {
            continue;
        }
        let proper = py_str(&_tidy_name(&Value::from(name.as_str())).val());
        if !asked_who.is_empty() {
            if concept_or(&asked_who) == concept_or(&who) {
                return Some(_vary(
                    &["Yes - ".to_string() + &proper + " is your " + &who + ".", "That's right: ".to_string() + &proper + " is your " + &who + "."],
                    memory,
                ));
            }
            return Some("No - ".to_string() + &proper + " is your " + &who + ".");
        }
        return Some(_vary(
            &[
                proper.clone() + " is your " + &who + ".",
                proper.clone() + " \u{2014} your " + &who + ", you told me.",
                "That's ".to_string() + &proper + ", your " + &who + ".",
            ],
            memory,
        ));
    }
    for entry in told.items().iter().rev() {
        // A group given whole files both names on the one entry.
        if !entry.is_dict() {
            continue;
        }
        let names: Vec<String> =
            entry.get("names").filter(|v| v.truthy()).map(py_list).unwrap_or_default().iter().map(|n| text::strip(&py_str(n)).to_string()).filter(|n| !n.is_empty()).collect();
        let who = text::strip(&sget(entry, "who")).to_lowercase();
        if names.len() < 2 || who.is_empty() || !names.iter().any(|n| n.to_lowercase() == said) {
            continue;
        }
        let one = names.iter().find(|n| n.to_lowercase() == said).unwrap();
        let proper = py_str(&_tidy_name(&Value::from(one.as_str())).val());
        if !asked_who.is_empty() {
            let wanted = Value::from(concept_or(&asked_who));
            if _relations_of(&who).val().items().iter().any(|r| *r == wanted) {
                return Some(_vary(
                    &[
                        "Yes - ".to_string() + &proper + " is one of your " + &who + ".",
                        "That's right: ".to_string() + &proper + " is among your " + &who + ".",
                    ],
                    memory,
                ));
            }
            return Some(proper + " is one of your " + &who + ", not your " + &asked_who + ".");
        }
        let listed = _a_list_of_names(&names);
        return Some(_vary(
            &[
                proper.clone() + " is one of your " + &who + " - you told me your " + &who + " are " + &listed + ".",
                "One of your ".to_string() + &who + ": you said your " + &who + " are " + &listed + ".",
            ],
            memory,
        ));
    }
    // The same thing for an animal: "whos tom" after "i have a cat called tom".
    for pet in user(memory).get("pets").map(py_list).unwrap_or_default() {
        if !pet.is_dict() {
            continue;
        }
        let name = text::strip(&sget(&pet, "name")).to_string();
        if name.is_empty() || name.to_lowercase() != said {
            continue;
        }
        let proper = py_str(&_tidy_name(&Value::from(name.as_str())).val());
        let mut animal = sget(&pet, "animal");
        if animal.is_empty() {
            animal = "pet".into();
        }
        let animal = text::strip(&animal).to_lowercase();
        if !asked_who.is_empty() {
            if concept_or(&asked_who) == concept_or(&animal) {
                return Some(_vary(
                    &["Yes - ".to_string() + &proper + " is your " + &animal + ".", "That's right - ".to_string() + &proper + " is your " + &animal + "."],
                    memory,
                ));
            }
            return Some(proper + " is your " + &animal + ", not your " + &asked_who + ".");
        }
        return Some(_vary(&[proper.clone() + " is your " + &animal + ".", "That's ".to_string() + &proper + ", your " + &animal + "."], memory));
    }
    None
}

static NUMBER_WORDS: LazyLock<HashMap<i64, String>> =
    LazyLock::new(|| g("NUMBER_WORDS").pairs().iter().map(|(k, v)| (k.as_int().unwrap(), v.str().to_string())).collect());

/// Python: bot/memory/store_replies.py:7814 `_kids_reply(memory, low)`
///
/// How many children they have, said back: "You've got two kids."
pub fn _kids_reply(memory: &mut Memory, low: &str) -> Option<String> {
    if !_kids_ask(low) {
        return None;
    }
    let held = user(memory).get("kids").cloned().unwrap_or(Value::None);
    if held == Value::Int(0) && matches!(held, Value::Int(_)) {
        // A person who said they have none has answered the question.
        return Some(_vary(
            &["You told me you don't have any kids.", "None \u{2014} you said you don't have any.", "You said you haven't got any kids."]
                .map(str::to_string),
            memory,
        ));
    }
    if held.is_none() {
        // A child named without an age is in `people` and not in `others`.
        let mut named: Vec<Value> =
            others(memory).items().iter().filter(|e| e.is_dict() && CHILD_RELATIONS.contains(&sget(e, "who").to_lowercase())).cloned().collect();
        if named.is_empty() {
            named = user(memory)
                .get("people")
                .map(py_list)
                .unwrap_or_default()
                .iter()
                .filter(|w| CHILD_RELATIONS.contains(&py_str(w).to_lowercase()))
                .map(|w| dict_of(vec![("who", w.clone())]))
                .collect();
        }
        if let Some(first) = named.first() {
            let mut who = sget(first, "who");
            if who.is_empty() {
                who = "child".into();
            }
            let who = who.to_lowercase();
            return Some(_vary(
                &[
                    "You've mentioned your ".to_string() + &who + " \u{2014} how many have you got altogether?",
                    "I know about your ".to_string() + &who + ", but not a number. How many have you got in all?",
                ],
                memory,
            ));
        }
        // Their own children: the honest answer is that they have not said.
        return Some(_vary(
            &[
                "You haven't told me how many kids you have yet.",
                "I don't have a number from you \u{2014} how many have you got?",
                "You haven't said how many children you've got.",
            ]
            .map(str::to_string),
            memory,
        ));
    }
    let count = py_int_value(&held).expect("ValueError: invalid literal for int()");
    if count == 1 {
        return Some(_vary(&["You've got one kid.", "One kid, as you told me.", "You said you've got one kid."].map(str::to_string), memory));
    }
    let word = NUMBER_WORDS.get(&count).cloned().unwrap_or_else(|| count.to_string());
    let kids = if count == 2 { "children" } else { "kids" };
    Some(_vary(
        &[
            "You've got ".to_string() + &word + " " + kids + ".",
            text::capitalize(&word) + " " + kids + " \u{2014} that's what you told me.",
            "You said you've got ".to_string() + &word + " " + kids + ".",
        ],
        memory,
    ))
}

/// Python: bot/memory/store_replies.py:7859 `_and_list(words)`
///
/// The words as one phrase: "nuts", "mum and dad", "Meg, Dan and Sam".
pub fn _and_list(words: &[String]) -> String {
    let words: Vec<&String> = words.iter().filter(|w| !w.is_empty()).collect();
    if words.len() < 2 {
        return words.iter().map(|w| w.as_str()).collect();
    }
    words[..words.len() - 1].iter().map(|w| w.as_str()).collect::<Vec<_>>().join(", ") + " and " + words[words.len() - 1]
}

/// Python: bot/memory/store_replies.py:7866 `_lines_a_person_moved(memory)`
///
/// The lines about someone whose fact a later line gave again with another value.
pub fn _lines_a_person_moved(memory: &Memory) -> HashSet<String> {
    let mut last: HashMap<(String, String), (String, String)> = HashMap::new();
    let mut moved: HashSet<String> = HashSet::new();
    for entry in mget(memory, "facts_log").items() {
        if !entry.is_dict() {
            continue;
        }
        let value = entry.get("value").cloned().unwrap_or(Value::None);
        if !value.is_dict() {
            continue;
        }
        let who = sget(&value, "who").to_lowercase();
        if who.is_empty() {
            continue;
        }
        let text_ = fold(&sget(entry, "text"));
        if text_.is_empty() {
            continue;
        }
        for (key, held) in value.pairs() {
            if key.as_str() == Some("who") || matches!(held, Value::List(_) | Value::Tuple(_) | Value::Dict(_)) {
                continue;
            }
            let slot = (who.clone(), py_str(key));
            let said = py_str(held).to_lowercase();
            if let Some(older) = last.get(&slot) {
                if older.0 != said {
                    moved.insert(older.1.clone());
                }
            }
            last.insert(slot, (said, text_.clone()));
        }
    }
    moved
}

rx!(SAME_FACT_ABOUT, SAME_FACT_AGAINST, SAME_FACT_AGAINST_PRONOUN);
strset!(NEGATIVE_TAIL);

/// Python: bot/memory/store_replies.py:7893 `_only_a_pointer(flat)`
///
/// True for a line that only points at a fact: "she isnt", "my wife is too".
pub fn _only_a_pointer(flat: &str) -> bool {
    if SAME_FACT_ABOUT.match_(flat).is_some() {
        return true;
    }
    for shape in [&*SAME_FACT_AGAINST, &*SAME_FACT_AGAINST_PRONOUN] {
        let Some(m) = shape.match_(flat) else { continue };
        let rest = text::strip_chars(m.name("rest").unwrap_or(""), " ?!.").to_string();
        if rest.is_empty() || NEGATIVE_TAIL.contains(&text::split(&rest)[0]) {
            return true;
        }
    }
    false
}

rx!(ABOUT_PERSON_ASK);
static CARRIED_ON: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"^\s*(?:and\s+|also\s+)?(?:she|he|they|her|him|them|his|hers|their)(?:'s|'re|s|re)?\b", crate::py::re::I));

/// Python: bot/memory/store_replies.py:7916 `_a_known_person(memory, who)`
///
/// What the chat holds about one person: the facts filed against them and the lines said.
/// Python merges list facts with `entry[key] += [...]` on the list it took from the first
/// record, which extends that record's list in `memory["others"]` too; that write is kept.
pub fn _a_known_person(memory: &mut Memory, who: &str) -> (Value, Vec<String>) {
    let family = _the_same_person(who);
    let mut entry = Value::dict();
    // The record each list in `entry` is (in Python, the same object as) the list of.
    let mut owner: HashMap<String, usize> = HashMap::new();
    let told = others(memory);
    for (index, known) in told.items().iter().enumerate() {
        if !family.contains(&s_or(known.get("who")).to_lowercase()) {
            continue;
        }
        // The facts for one person arrive in several entries, so they are read together.
        for (key, value) in known.pairs() {
            let key = py_str(key);
            if key == "who" {
                continue;
            }
            if let (Value::List(items), Some(Value::List(_))) = (value, entry.get(&key)) {
                let mut merged = entry.get(&key).unwrap().clone();
                let fresh: Vec<Value> = items.iter().filter(|i| !merged.items().contains(i)).cloned().collect();
                for item in fresh {
                    merged.push(item);
                }
                entry.set_key(key.as_str(), merged.clone());
                if let Some(&at) = owner.get(&key) {
                    if let Some(Value::List(list)) = memory.get_mut("others") {
                        if let Some(record) = std::sync::Arc::make_mut(list).get_mut(at) {
                            record.set_key(key.as_str(), merged);
                        }
                    }
                }
            } else {
                entry.set_key(key.as_str(), value.clone());
                if matches!(value, Value::List(_)) {
                    owner.insert(key.clone(), index);
                } else {
                    owner.remove(&key);
                }
            }
        }
    }
    // "my mum" is the usual way of naming her; a line that carries on in a pronoun is about the
    // person named just before it.
    let pattern = _the_name_pattern(who);
    let mut lines: Vec<String> = Vec::new();
    // A line whose fact a later line gave again with a different value is not something to say back.
    let moved = _lines_a_person_moved(memory);
    let mut current = String::new();
    let turns = memory.turns.clone();
    for turn in &turns {
        let line = &turn.user;
        if line.is_empty() {
            continue;
        }
        let named_here = _a_relation_in(line);
        if !named_here.is_empty() {
            current = named_here;
        }
        if crate::memory::corrected::_asks(line) || _about_this_chat_m(line, None) || ABOUT_PERSON_ASK.is_match(&line.to_lowercase()) {
            continue;
        }
        if (pattern.is_match(line) && !A_PROGRAMME_IS_ON.is_match(line)) || (family.contains(&current) && CARRIED_ON.match_(line).is_some()) {
            let flat = text::strip_chars(&fold(line), " ?!.").to_string();
            if moved.contains(&flat) {
                continue;
            }
            if _only_a_pointer(&flat) {
                // "she isnt" is the fact taken back, not something to say back about the person.
                continue;
            }
            lines.push(line.clone());
            // "his names tom" after "my husband" is his name.
            let given = _a_name_in_a_pronoun_line(line);
            if !given.is_empty() && !entry.get("name").map(Value::truthy).unwrap_or(false) {
                entry.set_key("name", given.as_str());
                lines.clear();
            }
        }
    }
    // A line that gives the name already filed for them says nothing the store cannot say better.
    if let Some(name) = entry.get("name").filter(|n| n.truthy()).cloned() {
        lines.retain(|one| !_only_the_name(one, &name, who));
    }
    (entry, lines)
}

static WORDS_LOWER_DIGITS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9]+", 0));

/// Python: bot/memory/store_replies.py:7991 `_only_the_name(line, named, who)`
///
/// True when a line says nothing but the name the store already files for the person.
pub fn _only_the_name(line: &str, named: &Value, who: &str) -> bool {
    let person: HashSet<String> = _the_same_person(who).into_iter().collect();
    let mut drop: HashSet<String> =
        ["my", "our", "your", "the", "is", "was", "name", "names", "called", "and", "she", "he", "her", "his", "they", "their", "them", "also", "s"]
            .iter()
            .map(|w| w.to_string())
            .collect();
    drop.extend(person.iter().cloned());
    let words: HashSet<String> = WORDS_LOWER_DIGITS
        .findall_str(&line.to_lowercase())
        .into_iter()
        .map(|w| {
            let bare = text::slice(&w, 0, Some(-1));
            if w.ends_with('s') && (drop.contains(&bare) || person.contains(&bare)) {
                bare
            } else {
                w
            }
        })
        .collect();
    let left: HashSet<&String> = words.iter().filter(|w| !drop.contains(*w)).collect();
    let name = s_or(Some(named)).to_lowercase();
    left.len() == 1 && left.contains(&name)
}

static A_NAME_FOR_THEM: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"^\s*(?:and\s+|also\s+)?(?:his|her|their)\s+names?\s*(?:'s|is\s+|are\s+)?(?P<name>[a-z]{2,})\b", crate::py::re::I));
strset!(NOT_A_NAME);

/// Python: bot/memory/store_replies.py:8012 `_a_name_in_a_pronoun_line(line)`
///
/// The name a line gives for the person it only calls "his"/"her", or "".
pub fn _a_name_in_a_pronoun_line(line: &str) -> String {
    let Some(m) = A_NAME_FOR_THEM.match_(line) else { return String::new() };
    let name = m.name_str("name").to_lowercase();
    if NOT_A_NAME.contains(&name) {
        return String::new(); // "his name is not important": no name there
    }
    name
}

static PRONOUN_LEAD: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:and\s+)?(?:she|he|they)(?:'s|'re|s|re)?\b", 0));
static POSSESSIVE_LEAD: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:and\s+)?(?:his|her|their)\b", 0));

/// Python: bot/memory/store_replies.py:8031 `_named_back(who, line)`
///
/// "she doesnt eat meat" is said back as "your sister doesnt eat meat", verb and all. (Python
/// passes `re.I` where `re.sub` takes its count, so the swap is case-sensitive and counts 2.)
pub fn _named_back(who: &str, line: &str) -> String {
    let your_who = Re::new(&format!(r"\byour\s+{}\b", crate::py::re::escape(who)), crate::py::re::I);
    if your_who.is_match(line) {
        return line.to_string();
    }
    let line = PRONOUN_LEAD.sub_with(line, crate::py::re::I as usize, |m| {
        let word = m.as_str().to_lowercase();
        let head = "your ".to_string() + who;
        if word.ends_with("re") {
            head + " are"
        } else if word.ends_with('s') && !word.ends_with("ss") {
            head + " is"
        } else {
            head
        }
    });
    POSSESSIVE_LEAD.sub_with(&line, crate::py::re::I as usize, |_| {
        "your ".to_string() + who + if crate::memory::relations::_plural_person(who) { "'" } else { "'s" }
    })
}

static COMPOUND_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.COMPOUND_AFTER"));
static RELATION_HITS: LazyLock<Vec<(String, Re)>> =
    LazyLock::new(|| FACTS_RELATIONS.iter().map(|r| (r.clone(), Re::new(&format!(r"\b{r}s?\b"), 0))).collect());

/// Python: bot/memory/store_replies.py:8061 `_a_relation_in(line)`
///
/// The relation a line is about, read loosely: a noun right after it is a thing of theirs.
pub fn _a_relation_in(line: &str) -> String {
    let low = line.to_lowercase();
    for (relation, pattern) in RELATION_HITS.iter() {
        for hit in pattern.finditer(&low) {
            let mut rest = low[hit.end()..].to_string();
            if rest.starts_with("'s") || rest.starts_with("s'") {
                let after = rest.splitn(2, '\'').nth(1).unwrap_or("").to_string();
                rest = text::slice(&after, 1, None);
            }
            let words = text::split(&rest);
            if let Some(first) = words.first() {
                if COMPOUND_AFTER.contains(text::strip_chars(first, ".,!?;:")) {
                    continue;
                }
            }
            return relation.clone();
        }
    }
    String::new()
}

/// Python: bot/memory/store_replies.py:8081 `_person_clauses(who, entry)`
///
/// The predicates about one person, subject left off: "is 32", "doesn't eat meat".
pub fn _person_clauses(who: &str, entry: &Value) -> Vec<String> {
    let be = if crate::memory::relations::_plural_person(who) { "are" } else { "is" };
    let mut clauses = Vec::new();
    if let Some(Value::Int(age)) = entry.get("age") {
        clauses.push(format!("{be} {age}"));
    }
    let allergies: Vec<String> = entry.get("allergies").filter(|v| v.truthy()).map(py_list).unwrap_or_default().iter().filter(|a| a.truthy()).map(py_str).collect();
    if !allergies.is_empty() {
        clauses.push(format!("{be} allergic to {}", _and_list(&allergies)));
    }
    let condition = match entry.get("condition") {
        Some(Value::List(items)) => items.first().map(py_str).unwrap_or_default(),
        Some(v) => s_or(Some(v)),
        None => String::new(),
    };
    if !text::strip(&condition).is_empty() {
        clauses.push(format!("{be} {}", text::strip(&condition)));
    }
    let diet = text::strip(&sget(entry, "diet")).to_string();
    if !diet.is_empty() {
        clauses.push(_a_diet_predicate(who, &diet));
    }
    let dislikes: Vec<String> = entry.get("dislikes").filter(|v| v.truthy()).map(py_list).unwrap_or_default().iter().filter(|a| a.truthy()).map(py_str).collect();
    if !dislikes.is_empty() {
        clauses.push(text::strip(&(_does_not(who) + " like " + &_and_list(&dislikes))).to_string());
    }
    clauses
}

static ORDINAL_DATE: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(\d{1,2}(?:st|nd|rd|th))\s+([A-Z][a-z]+)$", 0));

/// Python: bot/memory/store_replies.py:8109 `_date_written_as_a_date(value)`
///
/// A month inside a date is written with a capital: "4 may" is said "4 May".
pub fn _date_written_as_a_date(value: &Value) -> String {
    let mut said = s_or(Some(value));
    for month in FACTS_MONTHS.iter() {
        said = Re::new(&format!(r"\b{month}\b"), crate::py::re::I).sub(&text::upper_first(month), &said);
    }
    // "4th may" is said "the 4th of May", never "the 4th May".
    if let Some(ordinal) = ORDINAL_DATE.match_(&said) {
        said = format!("the {} of {}", ordinal.str(1), ordinal.str(2));
    }
    said
}

static ANY_PRONOUN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:her|him|them|she|he|they)\b", 0));
static HER_SHE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:her|she)\b", 0));
static HIM_HE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:him|he)\b", 0));

/// Python: bot/memory/store_replies.py:8123 `_pronoun_person(text, memory)`
///
/// Who "her"/"him"/"them" is in this line: the person last named, when the word fits them.
pub fn _pronoun_person(text_: &str, memory: &mut Memory) -> String {
    let low = text_.to_lowercase();
    if !ANY_PRONOUN.is_match(&low) {
        return String::new();
    }
    let who = crate::memory::relations::_person_from_pronoun(text_, memory, &[]);
    if who.is_empty() || crate::memory::relations::_plural_person(&who) {
        return who;
    }
    if HER_SHE.is_match(&low) && !FACTS_SHE.contains(&who) {
        return String::new();
    }
    if HIM_HE.is_match(&low) && !FACTS_HE.contains(&who) {
        return String::new();
    }
    who
}

/// Python: bot/memory/store_replies.py:8143 `_a_line_about(memory, who)`
///
/// The newest line of theirs that names this person, or None.
pub fn _a_line_about(memory: &Memory, who: &str) -> Option<String> {
    let pattern = _the_word_for_them(who);
    for turn in memory.turns.iter().rev() {
        let line = text::strip(&turn.user).to_string();
        // An ask is not something they told me.
        if line.is_empty() || line.contains('?') || crate::memory::corrected::_asks(&line) || ABOUT_PERSON_ASK.is_match(&line.to_lowercase()) {
            continue;
        }
        if _only_a_pointer(text::strip_chars(&fold(&line), " ?!.")) {
            continue;
        }
        if pattern.is_match(&line) {
            return Some(line);
        }
    }
    None
}

rx!(OWNED_ASK);

/// Python: bot/memory/store_replies.py:8172 `_a_that_fits(name)`
///
/// The article a name takes: "an iPhone 13", "a Pixel", "a BMW", "an Audi".
pub fn _a_that_fits(name: &str) -> String {
    let first = text::split(name).first().cloned().unwrap_or_default();
    if text::len(&first) > 1 && text::is_upper(&first) {
        return "a".into();
    }
    let head = text::slice(&first, 0, Some(1)).to_lowercase();
    // `"" in "aeiou"` is True in Python.
    if "aeiou".contains(head.as_str()) { "an" } else { "a" }.to_string()
}

static FACTS_OWNED: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.OWNED"));

/// Python: bot/memory/store_replies.py:8184 `_a_thing_i_own_reply(memory, low)`
///
/// The make of a thing they said was theirs: "your car is a Ford Fiesta".
pub fn _a_thing_i_own_reply(memory: &mut Memory, low: &str) -> Option<String> {
    let owned = user(memory).get("owns").cloned().unwrap_or(Value::None);
    let held = if owned.is_dict() && owned.truthy() { owned } else { Value::dict() };
    for m in OWNED_ASK.finditer(low) {
        let thing = m.groups().into_iter().flatten().find(|w| !w.is_empty()).unwrap_or("").replace('-', " ");
        if thing.is_empty() {
            continue;
        }
        let mut name = held.get(&thing).filter(|v| v.truthy()).or_else(|| held.get(text::rstrip_chars(&thing, "s"))).cloned().unwrap_or(Value::None);
        if thing == "car" && !name.truthy() {
            // The fact reader holds the car, and this reader asks first.
            name = user(memory).get("car").filter(|v| v.truthy()).cloned().unwrap_or(Value::from(""));
        }
        if thing == "car" && name.truthy() {
            // One fact, one shape: a model is capitalised however the car was filed.
            return Some(car_line(&name));
        }
        let said: Option<(&str, &str)> = match thing.as_str() {
            "bike" => Some(("You ride ", "Your bike is ")),
            "motorbike" => Some(("You ride ", "Your motorbike is ")),
            _ => None,
        };
        if name.truthy() {
            let name = py_str(&name);
            let article = _a_that_fits(&name);
            if let Some((ride, is)) = said {
                return Some(pick(&[ride.to_string() + &article + " " + &name + ".", is.to_string() + &article + " " + &name + "."], memory, ""));
            }
            if thing == "phone" {
                return Some(pick(
                    &["Your phone is ".to_string() + &article + " " + &name + ".", "You've got ".to_string() + &article + " " + &name + "."],
                    memory,
                    "",
                ));
            }
            return Some(pick(
                &[
                    "Your ".to_string() + &thing + " is " + &article + " " + &name + ".",
                    "You told me your ".to_string() + &thing + " is " + &article + " " + &name + ".",
                ],
                memory,
                "",
            ));
        }
        if FACTS_OWNED.contains(&thing) {
            // They asked about a thing nobody has mentioned the make of.
            return Some(
                pick(&["You haven't told me the make of your ".to_string() + &thing, "You've not said what your ".to_string() + &thing + " is"], memory, "") + ".",
            );
        }
    }
    None
}

/// Python: bot/memory/store_replies.py:10492 `_the_same_person(who)`
///
/// Every word the chat may use for the person an ask names, its own word first.
pub fn _the_same_person(who: &str) -> Vec<String> {
    let who = text::strip(who).to_lowercase();
    if who.is_empty() {
        return Vec::new();
    }
    let mut words = vec![who.clone()];
    if let Some(c) = concept(&who) {
        words.extend(CONCEPTS.iter().filter(|(w, its)| its == c && *w != who).map(|(w, _)| w.clone()));
    }
    words
}

rx!(A_PROGRAMME_IS_ON, PERSON_STANDS);
sval!(POSSESSIVE);

/// Python: bot/memory/store_replies.py:10513 `_the_name_pattern(who)`
///
/// A line that names this person, in any of the words the chat uses for them.
pub fn _the_name_pattern(who: &str) -> Re {
    let called = format!("(?:{})", _the_same_person(who).iter().map(|w| crate::py::re::escape(w)).collect::<Vec<_>>().join("|"));
    let not_a_relation = r"(?!\s*-?\s*in\s*-?\s*law)";
    Re::new(
        &format!(
            r"\b(?:my|our)\s+{called}{not_a_relation}{possessive}\b|(?:^\s*|[,;]\s*|\b(?:and|but|also|then)\s+){called}{not_a_relation}{possessive}\b",
            possessive = *POSSESSIVE
        ),
        crate::py::re::I,
    )
}

/// Python: bot/memory/store_replies.py:10535 `_the_word_for_them(who)`
///
/// Every word for this person standing where the person stands, anywhere in the line.
pub fn _the_word_for_them(who: &str) -> Re {
    let words = _the_same_person(who).iter().map(|w| crate::py::re::escape(w)).collect::<Vec<_>>().join("|");
    Re::new(&format!(r"\b(?:{words}){}\s*{}", *POSSESSIVE, PERSON_STANDS.pattern()), crate::py::re::I)
}

/// A str argument for a contract parameter whatever Rust type it was given: `relations::_one_age`
/// types its `turns` (the age a person turns, a str in Python) as `&[Turn]`.
trait FromPyStr<'a> {
    fn from_py(s: &'a str) -> Self;
}
impl<'a> FromPyStr<'a> for &'a str {
    fn from_py(s: &'a str) -> Self {
        s
    }
}
impl<'a> FromPyStr<'a> for &'a [Turn] {
    fn from_py(s: &'a str) -> Self {
        if s.is_empty() {
            return &[];
        }
        panic!("contract: bot.memory.relations:_one_age takes `turns` as a str, typed &[Turn]")
    }
}

rx!(TELLING_ALREADY);

/// Python: bot/memory/store_replies.py:8228 `_about_a_person_reply(memory, low, text="")`
///
/// Everything held about someone they mentioned: "what do you know about my sister?".
pub fn _about_a_person_reply(memory: &mut Memory, low: &str, text_: &str) -> Option<String> {
    use crate::memory::relations::_called;
    // "what do you know about ada" is the same ask when Ada is a name the chat was given.
    let called = _the_person_called(memory, low);
    let asked = ABOUT_PERSON_ASK.is_match(low)
        || crate::memory::relations::_continues_an_about_ask(memory, low)
        || (!called.is_empty() && ABOUT_A_NAME.is_match(low));
    if !asked {
        return None;
    }
    let mut who = crate::memory::relations::_named_person(low, Some(&mut *memory));
    if who.is_empty() {
        who = called;
    }
    if who.is_empty() {
        // "what do you know about her" asks about the person the chat was just on.
        who = _pronoun_person(if text_.is_empty() { low } else { text_ }, memory);
    }
    if who.is_empty() {
        // "what do you know about my kids" names no one relation: the children as a group.
        if let Some(children) = _about_the_children_reply(memory, low).filter(|c| !c.is_empty()) {
            return Some(children);
        }
        return None;
    }
    let (entry, lines) = _a_known_person(memory, &who);
    let named = match entry.get("name").filter(|n| n.truthy()) {
        Some(n) => called_of(Some(n)),
        None => String::new(),
    };
    let clauses = _person_clauses(&who, &entry);
    let birthday = text::strip(&sget(&entry, "birthday")).to_string();
    let more = ABOUT_PERSON_MORE.is_match(text_);
    if !birthday.is_empty() && clauses.is_empty() && lines.is_empty() {
        // A date filed against a person is a fact about them like an age is.
        let date = _date_written_as_a_date(&Value::from(birthday.as_str()));
        return Some(_vary(
            &["Your ".to_string() + &who + "'s birthday is " + &date + ".", "You told me your ".to_string() + &who + "'s birthday is " + &date + "."],
            memory,
        ));
    }
    if clauses.is_empty() && !named.is_empty() && lines.is_empty() {
        // The only thing the chat holds about them is the name they gave for them.
        return Some(_vary(&["Your ".to_string() + &who + " is called " + &named + ".", "Your ".to_string() + &who + " is " + &named + "."], memory));
    }
    if clauses.is_empty() && lines.is_empty() {
        // A line the chat has about them is something to say, even when no fact was filed from it.
        if let Some(named_line) = _a_line_about(memory, &who) {
            let back = text::rstrip_chars(text::strip(&_named_back(&who, &crate::memory::you(&named_line))), ".").to_string();
            return Some(if !more { "You told me ".to_string() + &back + "." } else { "That's all I have: ".to_string() + &back + "." });
        }
        // The person is known to the chat only by name, or not at all.
        if more {
            return Some("You haven't told me anything about your ".to_string() + &who + " yet.");
        }
        return Some(_vary(
            &[
                "You haven't told me anything about your ".to_string() + &who + " yet.",
                "Nothing on your ".to_string() + &who + " so far \u{2014} tell me and I'll keep it.",
            ],
            memory,
        ));
    }
    let subject = "Your ".to_string() + &who + &(if !named.is_empty() && !clauses.is_empty() { " ".to_string() + &named } else { String::new() });
    let mut facts = String::new();
    if !clauses.is_empty() {
        if clauses.len() == 1 {
            facts = subject.clone() + " " + &clauses[0];
        } else {
            let rest: Vec<String> = clauses[1..]
                .iter()
                .map(|c| if clauses[0].starts_with("is ") && c.starts_with("is ") { text::slice(c, 3, None) } else { c.clone() })
                .collect();
            facts = subject.clone() + " " + &clauses[0] + " and " + &_and_list(&rest);
        }
    }
    let mut said = String::new();
    if let Some(last) = lines.last() {
        let line = text::rstrip_chars(text::strip(&crate::memory::you(last)), ".").to_string();
        // "she doesnt eat meat" is about the person they named a line earlier, so the reply names them.
        let mut line = _named_back(&who, &line);
        if !named.is_empty() {
            // The name they gave is written the way a name is written.
            line = Re::new(&format!(r"\b{}\b", crate::py::re::escape(&named)), crate::py::re::I).sub(&named, &line);
        }
        said = line;
    }
    // One line already carrying the facts is the whole answer.
    let answer = if !said.is_empty() && (facts.is_empty() || _covers(&said, &clauses)) {
        if TELLING_ALREADY.match_(&said).is_some() {
            text::upper_first(&said) + "."
        } else {
            _vary(&["You told me ".to_string() + &said, "You said ".to_string() + &said], memory) + "."
        }
    } else if !facts.is_empty() && !said.is_empty() && _tokens(&said).is_subset(&_tokens(&facts)) {
        facts.clone() + "."
    } else if !facts.is_empty() && !said.is_empty() {
        facts.clone() + ". You also said " + &said + "."
    } else {
        facts.clone() + "."
    };
    let everything = Re::new(
        &format!(r"\b(?:everything you've told me|all I've got) (?:about|on) your {}", crate::py::re::escape(&who)),
        crate::py::re::I,
    );
    if more && (_said_everything(&answer, memory) || everything.is_match(&last_turn_bot(memory))) {
        return Some(_vary(
            &[
                "That's everything you've told me about your ".to_string() + &who + ".",
                "That's still all I've got on your ".to_string() + &who + ".",
            ],
            memory,
        ));
    }
    if _said_everything(&answer, memory) {
        // The same ask twice: the second time it is shorter, and still true.
        let source = if !facts.is_empty() { &facts } else if !said.is_empty() { &said } else { &answer };
        let body = text::rstrip_chars(text::strip(source), ".").to_string();
        let _ = body.chars().next().expect("IndexError: string index out of range");
        return Some("Still: ".to_string() + &lower_first(&body) + ".");
    }
    Some(answer)
}

/// Python: bot/memory/store_replies.py:8347 `_about_the_children_reply(memory, low)`
///
/// What the chat holds about the children as a group: "tell me about my kids".
pub fn _about_the_children_reply(memory: &mut Memory, low: &str) -> Option<String> {
    use crate::memory::relations::{_a_list_of_names, _called, _in_the_group};
    if !CHILDREN_ABOUT.is_match(low) {
        return None;
    }
    let group = str_list(&CHILD_RELATIONS);
    let kept: Vec<Value> = others(memory).items().iter().filter(|e| e.is_dict() && _in_the_group(&s_or(e.get("who")), &group)).cloned().collect();
    let mut clauses: Vec<String> = Vec::new();
    for entry in kept.iter().take(4) {
        let name = match entry.get("name").filter(|n| n.truthy()) {
            Some(n) => _called(&py_str(n)),
            None => String::new(),
        };
        let names: Vec<String> = entry.get("names").filter(|v| v.truthy()).map(py_list).unwrap_or_default().iter().map(py_str).filter(|n| !text::strip(n).is_empty()).collect();
        let who_or = |default: &str| {
            let w = sget(entry, "who");
            if w.is_empty() {
                default.to_string()
            } else {
                w
            }
        };
        if names.len() > 1 {
            // "my kids are called tom and ada" is one fact about several children.
            let called: Vec<String> = names.iter().map(|n| _called(n)).collect();
            clauses.push("your ".to_string() + &who_or("children") + " are called " + &_a_list_of_names(&called));
        } else if entry.get("ages").map(Value::truthy).unwrap_or(false) {
            // A group given its ages as a list is one fact about the children.
            let ages: Vec<String> = py_list(entry.get("ages").unwrap()).iter().filter(|a| is_int(Some(*a))).map(py_str).collect();
            let who = who_or("children");
            let said = if ages.len() > 1 {
                ages[..ages.len() - 1].join(", ") + " and " + &ages[ages.len() - 1]
            } else {
                ages.first().cloned().expect("IndexError: list index out of range")
            };
            clauses.push("your ".to_string() + &who + &_is_are(&who) + &said);
        } else if entry.get("months").map(Value::truthy).unwrap_or(false) && !entry.get("age").map(Value::truthy).unwrap_or(false) {
            // A baby's age was given in months, and is said back in months.
            let lead = if name.is_empty() { "your ".to_string() + &who_or("child") } else { name.clone() };
            clauses.push(lead + " is " + &py_str(entry.get("months").unwrap()) + " months old");
        } else if entry.get("age").map(Value::truthy).unwrap_or(false) || entry.get("turns").map(Value::truthy).unwrap_or(false) {
            let turns = sget(entry, "turns");
            let said = crate::memory::relations::_one_age(
                &who_or("child"),
                entry.get("age").unwrap_or(&Value::None),
                FromPyStr::from_py(turns.as_str()),
                entry.get("to_come").unwrap_or(&Value::None),
                &Value::from(name.as_str()),
            );
            clauses.push(text::rstrip_chars(text::strip(&said), ".").to_string());
        } else if !name.is_empty() {
            clauses.push("your ".to_string() + &who_or("child") + " is called " + &name);
        }
    }
    // A clause that begins "Your son" is folded into the sentence and takes a small letter.
    let mut clauses: Vec<String> = clauses.into_iter().map(|c| if c.starts_with("Your ") { lower_first(&c) } else { c }).collect();
    if clauses.is_empty() {
        // A name given for the group in words the name readers did not file is still in the chat.
        let typed = {
            let kids = crate::memory::relations::_the_group_names_typed("kids", memory).val();
            if kids.truthy() {
                kids
            } else {
                crate::memory::relations::_the_group_names_typed("children", memory).val()
            }
        };
        if typed.truthy() {
            let called: Vec<String> = py_list(&typed).iter().map(|n| _called(&py_str(n))).collect();
            clauses = vec!["your kids are called ".to_string() + &_a_list_of_names(&called)];
        }
    }
    if clauses.is_empty() {
        let counted = user_facts(memory).get("kids").cloned().unwrap_or(Value::None);
        if let Value::Int(n) = counted {
            if n != 0 {
                let word = _a_number_word(n);
                return Some(_vary(
                    &[
                        "You've told me you have ".to_string() + &word + " " + if n == 1 { "kid" } else { "kids" } + ", and nothing more about them yet.",
                        "Just the count: ".to_string() + &word + " kids, with no name or age yet.",
                    ],
                    memory,
                ));
            }
        }
        if let Value::Bool(true) = counted {
            let word = _a_number_word(1);
            return Some(_vary(
                &[
                    "You've told me you have ".to_string() + &word + " kid, and nothing more about them yet.",
                    "Just the count: ".to_string() + &word + " kids, with no name or age yet.",
                ],
                memory,
            ));
        }
        return None;
    }
    // One child is "him" or "her".
    let one = if kept.len() == 1 { sget(&kept[0], "who") } else { String::new() };
    let about = if one.is_empty() { "them".to_string() } else { _object_pronoun(&one, &Value::None) };
    if clauses.len() == 1 {
        let first = "You've told me ".to_string() + &clauses[0] + ".";
        let second = text::upper_first(&clauses[0]) + " - that's all I've got on " + &about + ".";
        return Some(_vary(&[first, second], memory));
    }
    let facts = _a_list_of_clauses(&clauses);
    let answer = _vary(
        &[facts.clone() + " - that's what I've got on " + &about + ".", "That's everything I have on ".to_string() + &about + ": " + &facts + "."],
        memory,
    );
    Some(text::upper_first(&answer))
}

/// Python: bot/memory/store_replies.py:8448 `_a_list_of_clauses(clauses)`
///
/// "Tom is 8", "Ada is 5" reads "Tom is 8 and Ada is 5", and three read with a comma.
pub fn _a_list_of_clauses(clauses: &[String]) -> String {
    let said: Vec<&String> = clauses.iter().filter(|c| !c.is_empty()).collect();
    if said.len() < 3 {
        return said.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" and ");
    }
    said[..said.len() - 1].iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ") + " and " + said[said.len() - 1]
}

static ABOUT_A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\babout\s+([a-z][a-z'-]{1,20})\b", crate::py::re::I));

/// Python: bot/memory/store_replies.py:8455 `_the_person_called(memory, line)`
///
/// The person whose name the line asks about, or "".
pub fn _the_person_called(memory: &mut Memory, line: &str) -> String {
    for m in ABOUT_A_WORD.finditer(line) {
        let word = m.str(1).to_lowercase();
        if FACTS_RELATIONS.contains(&word) || ALONE_PERSON.contains(&word) {
            continue;
        }
        for entry in others(memory).items() {
            let name = text::strip(&sget(entry, "name")).to_lowercase();
            if !name.is_empty() && name == word {
                return sget(entry, "who");
            }
        }
    }
    String::new()
}

/// Python: bot/memory/store_replies.py:8472 `_covers(said, clauses)`
///
/// True when one line already says the facts: a number counts as a word to be found.
pub fn _covers(said: &str, clauses: &[String]) -> bool {
    let seen = _tokens(said);
    clauses.iter().all(|c| _tokens(c).is_subset(&seen))
}

static DIGITS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d+", 0));

/// Python: bot/memory/store_replies.py:8481 `_tokens(text)`
///
/// The words of a line to look for, with its numbers kept - content() drops them.
pub fn _tokens(text_: &str) -> HashSet<String> {
    let mut words: HashSet<String> = crate::memory::topics::content(text_).into_iter().collect();
    words.extend(DIGITS.findall_str(text_));
    words
        .into_iter()
        .map(|w| {
            let bare = text::slice(&w, 0, Some(-1));
            if w.ends_with('s') && FACTS_RELATIONS.contains(&bare) {
                bare
            } else {
                w
            }
        })
        .collect()
}

/// Python: bot/memory/store_replies.py:8491 `_said_everything(answer, memory)`
///
/// True when a reply of this chat already carried every word of this one.
pub fn _said_everything(answer: &str, memory: &Memory) -> bool {
    let mine = _tokens(answer);
    if mine.is_empty() {
        return false;
    }
    memory.turns.iter().any(|t| !t.bot.is_empty() && mine.is_subset(&_tokens(&t.bot)))
}

strs!(PREGNANT_OWN_SAY, PREGNANT_SAY, PREGNANT_WEEKS_SAY, LIMIT_SAY, FEEDING_SAY, STATE_WAS_SAY);

/// Python: bot/memory/store_replies.py:8490 `_a_pregnancy_of_their_own(text, memory, low)`
///
/// The person's own pregnancy said back, with the length that dates it.
pub fn _a_pregnancy_of_their_own(text_: &str, memory: &mut Memory, low: &str) -> Option<String> {
    if text_.contains('?') {
        return None;
    }
    if _about_this_chat(text_, memory) || OTHER_ASK.is_match(low) || REMINDER_MID.is_match(low) {
        return None;
    }
    let found = crate::memory::facts::extract(text_, false).val();
    if !found.get("pregnant").map(Value::truthy).unwrap_or(false) {
        return None;
    }
    if is_int(found.get("weeks")) {
        let weeks = _a_number_word(py_int_value(found.get("weeks").unwrap()).unwrap());
        let lines: Vec<String> = PREGNANT_OWN_SAY.iter().map(|l| fmt_named(l, &[("weeks", &weeks)])).collect();
        return Some(_vary(&lines, memory));
    }
    // A length nobody said is not one to invent.
    Some(_vary(
        &[
            "Good to know \u{2014} I'll keep the drink out of anything I suggest, and the weeks are worth telling me if you want a meal plan that fits them.",
            "Noted, and congratulations. Nothing alcoholic in anything I suggest; say if you want a hand with what to cook.",
        ]
        .map(str::to_string),
        memory,
    ))
}

/// Python: bot/memory/store_replies.py:8515 `_a_state_told(text, memory, low)`
///
/// A state of another person, said back: "Noted - your wife is pregnant...".
pub fn _a_state_told(text_: &str, memory: &mut Memory, low: &str) -> Option<String> {
    if text_.contains('?') {
        return None;
    }
    if _the_ask_beats_the_receipt(text_, memory) {
        // The line ends on a question the cards can answer.
        return None;
    }
    if _about_this_chat(text_, memory) || OTHER_ASK.is_match(low) || REMINDER_MID.is_match(low) {
        return None;
    }
    let found = crate::memory::facts::extract(text_, false).val();
    if ["diet", "allergies", "pregnant"].iter().any(|k| found.get(k).map(Value::truthy).unwrap_or(false)) {
        return None; // a line about the person typing is theirs
    }
    let named: Vec<String> = found.get("people").map(py_list).unwrap_or_default().iter().filter_map(|w| w.as_str().map(str::to_string)).collect();
    for entry in found.get("others").map(py_list).unwrap_or_default() {
        let who = match entry.get("who").filter(|w| w.truthy()) {
            Some(w) => py_str(w),
            None => crate::memory::relations::_person_from_pronoun(text_, memory, &named),
        };
        if who.is_empty() {
            continue;
        }
        let pregnant = entry.get("pregnant").map(Value::truthy).unwrap_or(false);
        if pregnant && is_int(entry.get("weeks")) {
            // The length is what makes the fact worth keeping.
            let weeks = _a_number_word(py_int_value(entry.get("weeks").unwrap()).unwrap());
            let lines: Vec<String> = PREGNANT_WEEKS_SAY.iter().map(|l| fmt_named(l, &[("who", &who), ("weeks", &weeks)])).collect();
            return Some(_vary(&lines, memory));
        }
        let pool: Option<&Vec<String>> = if pregnant {
            Some(&PREGNANT_SAY)
        } else if entry.get("breastfeeding").map(Value::truthy).unwrap_or(false) {
            Some(&FEEDING_SAY)
        } else if entry.get("drinks") == Some(&Value::Bool(false)) {
            Some(&LIMIT_SAY)
        } else {
            None
        };
        if let Some(pool) = pool {
            let lines: Vec<String> = pool.iter().map(|l| fmt_named(l, &[("who", &who)])).collect();
            return Some(_vary(&lines, memory));
        }
    }
    None
}

rx!(WAS_A_STATE);

/// Python: bot/memory/store_replies.py:8560 `_a_state_that_was(text, memory, low)`
///
/// A pregnancy or a feeding that is over, acknowledged as the past thing it is.
/// (Contract change: the generated stub returned `()`.)
pub fn _a_state_that_was(text_: &str, memory: &mut Memory, low: &str) -> Option<String> {
    if text_.contains('?') {
        return None;
    }
    if _about_this_chat(text_, memory) {
        return None;
    }
    if !WAS_A_STATE.is_match(&text::squash(text_)) {
        return None;
    }
    Some(_vary(&STATE_WAS_SAY, memory))
}

rx!(PET_PART_ASK, BREED_WORD, PET_ASK);
static AGE_OR_HOW_OLD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bage\b|\bhow old\b", 0));

/// Python: bot/memory/store_replies.py:8584 `_pet_part_reply(memory, low)`
///
/// An age or a breed asked about an animal they have described, or None.
pub fn _pet_part_reply(memory: &mut Memory, low: &str) -> Option<String> {
    if !PET_PART_ASK.is_match(low) {
        return None;
    }
    if AGE_OR_HOW_OLD.is_match(low) {
        if let Some(said) = some_str(crate::memory::relations::_pet_age_reply(memory, low)) {
            return Some(said);
        }
    }
    _pet_breed_reply(memory, low)
}

static A_PRONOUN_OR_POSSESSIVE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:she|he|it|they|his|her|its|their)\b", 0));

/// Python: bot/memory/store_replies.py:8594 `_pet_breed_reply(memory, low)`
///
/// The breed the chat was told, for "what breed is he", or None.
pub fn _pet_breed_reply(memory: &mut Memory, low: &str) -> Option<String> {
    use crate::memory::relations::_animal_family;
    if !BREED_WORD.is_match(low) {
        return None;
    }
    let pets: Vec<Value> = user(memory)
        .get("pets")
        .map(py_list)
        .unwrap_or_default()
        .into_iter()
        .filter(|p| p.is_dict() && !text::strip(&sget(p, "breed")).is_empty() && !p.get("not_theirs").map(Value::truthy).unwrap_or(false))
        .collect();
    if pets.is_empty() {
        return None;
    }
    let mut asked: Vec<Value> = Vec::new();
    let mut family = String::new();
    if let Some(animal) = PET_ASK.search(low) {
        family = _animal_family(animal.str(1));
    }
    if !family.is_empty() {
        asked = pets.iter().filter(|p| _animal_family(&s_or(p.get("animal"))) == family).cloned().collect();
    }
    if asked.is_empty() {
        for pet in &pets {
            let name = text::strip(&sget(pet, "name")).to_string();
            if text::len(&name) > 2 && says_word(&name.to_lowercase(), low) {
                asked.push(pet.clone());
            }
        }
    }
    if asked.is_empty() && A_PRONOUN_OR_POSSESSIVE.is_match(low) {
        asked = vec![pets[pets.len() - 1].clone()];
    }
    let pet = asked.last()?.clone();
    let breed = text::strip(&sget(&pet, "breed")).to_string();
    let name = text::strip(&sget(&pet, "name")).to_string();
    if name.is_empty() {
        // The breed is the only word the chat has for it.
        return Some(_vary(
            &["A ".to_string() + &breed + " - that is what you said.", "You said a ".to_string() + &breed + ".", "A ".to_string() + &breed + "."],
            memory,
        ));
    }
    let name = py_str(&crate::memory::relations::_tidy_name(&Value::from(name.as_str())).val());
    Some(_vary(
        &[name.clone() + " is a " + &breed + ".", "You told me ".to_string() + &name + " is a " + &breed + ".", "A ".to_string() + &breed + ", that one."],
        memory,
    ))
}

static A_PRONOUN_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(she|he|they)(?:'s|s|'re|re)?\b", 0));
rx!(AN_ANIMAL, A_SMALL_UNIT);

/// Python: bot/memory/store_replies.py:8644 `_a_pronoun_age(memory, low)`
///
/// The age of a person the chat only ever pointed at, or None.
pub fn _a_pronoun_age(memory: &mut Memory, low: &str) -> Option<String> {
    let found = A_PRONOUN_SAID.search(low)?;
    let pronoun = found.str(1).to_string();
    let lines: Vec<String> = recent_turns(memory, 5).iter().map(|t| t.user.clone()).collect();
    for i in (0..lines.len()).rev() {
        let said = &lines[i];
        if text::strip(said).is_empty() {
            continue;
        }
        // A kitten, a puppy or a budgie is not a person.
        let lowered = said.to_lowercase();
        let small = A_SMALL_UNIT.search(&lowered);
        let before = if i > 0 { lines[i - 1].as_str() } else { "" };
        if AN_ANIMAL.is_match(said) || (small.is_some() && (AN_ANIMAL.is_match(before) || mget(memory, "pets").truthy())) {
            continue;
        }
        // A baby's age is said in weeks or months, and the unit is said as it was said.
        if let Some(small) = &small {
            if _a_pronoun_word(said) == pronoun {
                let was = if pronoun == "they" { "are" } else { "is" };
                return Some(text::capitalize(&format!("{} {} {} {} old.", pronoun, was, small.str(1), small.str(2))));
            }
        }
        for entry in crate::memory::facts::extract(said, false).val().get("others").map(py_list).unwrap_or_default() {
            if !entry.is_dict() || entry.get("who").map(Value::truthy).unwrap_or(false) {
                continue;
            }
            // The reader keeps the pronoun on most shapes and not on others.
            let mut theirs = sget(&entry, "pronoun");
            if theirs.is_empty() {
                theirs = _a_pronoun_word(said);
            }
            if theirs != pronoun {
                continue;
            }
            let mut age = entry.get("age").cloned().unwrap_or(Value::None);
            if let Value::Str(s) = &age {
                let t = text::strip(s);
                if crate::understanding::pyuni::is_digit(t) {
                    age = Value::Int(py_int(t).expect("ValueError"));
                }
            }
            if !is_int(Some(&age)) {
                continue;
            }
            // A pronoun is never a name: the age is said with the animal in it when the store
            // holds this same age for the one animal the chat has been told about.
            if let Some(named) = crate::memory::relations::the_pronoun_said_with_a_name(memory, &age).filter(|n| !n.is_empty()) {
                return Some(named);
            }
            let turns = sget(&entry, "turns");
            return Some(crate::memory::relations::_one_age(&pronoun, &age, FromPyStr::from_py(turns.as_str()), &Value::None, &Value::from("")));
        }
    }
    None
}

/// Python: bot/memory/store_replies.py:8708 `_a_pronoun_word(text)`
///
/// The first pronoun in the line that points at a person: "she", "he" or "they".
pub fn _a_pronoun_word(text_: &str) -> String {
    A_PRONOUN_SAID.search(&text_.to_lowercase()).map(|m| m.str(1).to_string()).unwrap_or_default()
}

/// Python: bot/memory/store_replies.py:8720 `_someone_else_named(low, memory)`
///
/// Who else the line is about, as the store files them; "" when the line names nobody.
pub fn _someone_else_named(low: &str, memory: &mut Memory) -> String {
    let mut names: Vec<String> = others(memory).items().iter().map(|k| text::strip(&s_or(k.get("who"))).to_string()).filter(|n| !n.is_empty()).collect();
    names.extend(FACTS_RELATIONS.iter().cloned());
    // Longest first, then where the name is in the line, then the word itself.
    let flat = low.replace('\'', "").replace('\u{2019}', "");
    let unique: HashSet<String> = names.into_iter().collect();
    let mut found: Vec<(i64, i64, String)> = Vec::new();
    for name in unique {
        let bare = name.replace('\'', "").replace('\u{2019}', "");
        if let Some(m) = Re::new(&format!(r"\b{}s?\b", crate::py::re::escape(&bare)), 0).search(&flat) {
            found.push((-(text::len(&name) as i64), -(text::char_index(&flat, m.start()) as i64), name));
        }
    }
    if let Some(best) = found.into_iter().min() {
        return best.2;
    }
    if THEM.is_match(low) {
        return crate::memory::relations::_person_from_pronoun(low, memory, &[]);
    }
    String::new()
}

/// Python: bot/memory/store_replies.py:8775 `_the_same_fact_about_someone(text, memory)`
///
/// The fact standing about the person typing, given to someone they name: (reply, note).
/// (Contract change: the generated stub returned `()`.)
pub fn _the_same_fact_about_someone(text_: &str, memory: &mut Memory) -> Option<(String, Value)> {
    let low = text::strip_chars(&fold(text_), " ?!.").to_string();
    if low.is_empty() || text::split(&low).len() > 7 {
        return None;
    }
    let mut m = SAME_FACT_AGAINST.match_(&low);
    let mut against = m.is_some();
    let mut by_pronoun = false;
    if m.is_none() {
        m = SAME_FACT_AGAINST_PRONOUN.match_(&low);
        against = m.is_some();
        by_pronoun = against;
    }
    if m.is_none() {
        m = SAME_FACT_ABOUT.match_(&low);
    }
    let m = m?;
    let group = |g: &str| m.name(g).unwrap_or("").to_string();
    if by_pronoun {
        // "she isnt" is about the person the last note was written on.
        let (who, kind, value) = _the_person_last_given_a_fact(memory)?;
        if !_the_fact_said_against(&kind, &value, &group("rest")) {
            return None;
        }
        _take_the_fact_back(memory, &who, &kind);
        return Some((_ive_taken_it_back(&who, &kind, &value), Value::None));
    }
    let mut who = text::strip(&if group("who").is_empty() { group("more") } else { group("who") }).to_string();
    if !PLURAL_PEOPLE.contains(&who) && who.ends_with('s') && FACTS_RELATIONS.contains(&text::slice(&who, 0, Some(-1))) {
        who = text::slice(&who, 0, Some(-1));
    }
    if who.is_empty() {
        return None;
    }
    let same = text::strip(&group("same")).to_lowercase();
    if !against && same.is_empty() && !low.starts_with("same for") {
        // "my wife" on its own is not the allergy said again.
        return None;
    }
    let (kind, value) = _the_fact_on_record(memory)?;
    let given = crate::memory::facts::extract(text_, false).val();
    if SAME_FACT_THEIR_OWN.iter().any(|k| given.get(k).map(Value::truthy).unwrap_or(false)) {
        return None; // the line states a fact of its own
    }
    for known in others(memory).items() {
        if known.get("who") == Some(&Value::from(who.as_str())) && SAME_FACT_THEIR_OWN.iter().any(|k| known.get(k).map(Value::truthy).unwrap_or(false)) {
            return None; // the store already has this person's own facts
        }
    }
    if against {
        if group("same").to_lowercase() == "neither" {
            return None;
        }
        if !_the_fact_said_against(&kind, &value, &group("rest")) {
            return None;
        }
        return Some((_isnt_this_too(&who, &kind, &value), Value::None));
    }
    let kept = match &value {
        Value::List(items) => Value::List(items.clone()),
        other => other.clone(),
    };
    Some((_this_too(&who, &kind, &value), dict_of(vec![(kind.as_str(), kept), ("who", Value::from(who.as_str()))])))
}

strs!(SAME_FACT_THEIR_OWN);

/// Python: bot/memory/store_replies.py:8839 `_the_fact_said_against(kind, value, rest)`
///
/// True when what follows "my wife isnt" is the fact itself, or nothing at all.
pub fn _the_fact_said_against(kind: &str, value: &Value, rest: &str) -> bool {
    let words = text::split(text::strip_chars(rest, " ?!."));
    if words.is_empty() {
        return true;
    }
    if NEGATIVE_TAIL.contains(&words[0].to_lowercase()) {
        return words.iter().all(|w| NEGATIVE_TAIL.contains(&w.to_lowercase()));
    }
    let first = words[0].to_lowercase();
    if kind == "allergies" {
        return first == "allergic";
    }
    let head = match value {
        Value::List(items) => py_str(items.first().expect("IndexError: list index out of range")),
        other => py_str(other),
    }
    .to_lowercase();
    !head.is_empty() && (first == head || head.starts_with(&first))
}

/// Python: bot/memory/store_replies.py:8857 `_the_person_last_given_a_fact(memory)`
///
/// The person a "too" last gave a fact to, with that fact: (who, key, value) or None.
pub fn _the_person_last_given_a_fact(memory: &mut Memory) -> Option<(String, String, Value)> {
    for known in others(memory).items().iter().rev() {
        if !known.is_dict() || !known.get("who").map(Value::truthy).unwrap_or(false) {
            continue;
        }
        for key in ["allergies", "diet", "condition"] {
            if known.get(key).map(Value::truthy).unwrap_or(false) {
                return Some((py_str(known.get("who").unwrap()), key.to_string(), known.get(key).unwrap().clone()));
            }
        }
    }
    None
}

/// Python: bot/memory/store_replies.py:8867 `_take_the_fact_back(memory, who, kind)`
///
/// Take the fact off the person it was given to: "she isnt" after "and my wife is too".
pub fn _take_the_fact_back(memory: &mut Memory, who: &str, kind: &str) {
    let told = others(memory);
    let Some(at) = told.items().iter().rposition(|k| k.is_dict() && k.get("who") == Some(&Value::from(who))) else { return };
    let Some(Value::List(list)) = memory.get_mut("others") else { return };
    let list = std::sync::Arc::make_mut(list);
    list[at].del_key(kind);
    let known = list[at].clone();
    let left = known.pairs().iter().any(|(k, _)| !matches!(k.as_str(), Some("who") | Some("name") | Some("turns")));
    if !left {
        // Nothing is left on the person, so the entry goes (`list.remove`: the first equal one).
        if let Some(first) = list.iter().position(|k| *k == known) {
            list.remove(first);
        }
    }
}

static _SHE: LazyLock<HashSet<String>> = LazyLock::new(|| g("_SHE").str_set());
static _HE: LazyLock<HashSet<String>> = LazyLock::new(|| g("_HE").str_set());

/// The fact a too/isn't line says back: "allergic to nuts", or the condition or diet itself.
fn _the_fact_said(kind: &str, value: &Value) -> String {
    if kind == "allergies" {
        return "allergic to ".to_string() + &_and_list(&py_list(value).iter().map(py_str).collect::<Vec<_>>());
    }
    match value {
        Value::List(items) => py_str(items.first().expect("IndexError: list index out of range")),
        other => py_str(other),
    }
}

/// Python: bot/memory/store_replies.py:8880 `_ive_taken_it_back(who, kind, value)`
pub fn _ive_taken_it_back(who: &str, kind: &str, value: &Value) -> String {
    let said = _the_fact_said(kind, value);
    let plural = crate::memory::relations::_plural_person(who);
    let negative = if plural { " aren't " } else { " isn't " };
    let them = if plural {
        "them"
    } else if _SHE.contains(who) {
        "her"
    } else if _HE.contains(who) {
        "him"
    } else {
        "them"
    };
    "Your ".to_string() + who + negative + &said + ", then - I've taken that back off " + them + "."
}

/// Python: bot/memory/store_replies.py:8900 `_the_fact_on_record(memory)`
///
/// The one fact about the person typing that a "too" can carry: (key, value) or None.
pub fn _the_fact_on_record(memory: &mut Memory) -> Option<(String, Value)> {
    let them = user(memory);
    for key in ["allergies", "diet", "condition"] {
        if let Some(value) = them.get(key).filter(|v| v.truthy()) {
            return Some((key.to_string(), value.clone()));
        }
    }
    None
}

/// Python: bot/memory/store_replies.py:8914 `_both_of_you(who)`
pub fn _both_of_you(who: &str) -> String {
    if crate::memory::relations::_plural_person(who) { "all of you" } else { "both of you" }.to_string()
}

/// Python: bot/memory/store_replies.py:8918 `_this_too(who, kind, value)`
pub fn _this_too(who: &str, kind: &str, value: &Value) -> String {
    if kind == "allergies" {
        return "Your ".to_string()
            + who
            + &_is_are(who)
            + "allergic to "
            + &_and_list(&py_list(value).iter().map(py_str).collect::<Vec<_>>())
            + " too, then - I'll keep it out for "
            + &_both_of_you(who)
            + ".";
    }
    if kind == "condition" {
        let said = match value {
            Value::List(items) => py_str(items.first().expect("IndexError: list index out of range")),
            other => py_str(other),
        };
        return "Your ".to_string() + who + &_is_are(who) + &said + " too, then - I'll keep it in mind for " + &_both_of_you(who) + ".";
    }
    "Your ".to_string() + who + &_is_are(who) + &py_str(value) + " too, then - I'll plan around " + &_both_of_you(who) + "."
}

/// Python: bot/memory/store_replies.py:8930 `_isnt_this_too(who, kind, value)`
pub fn _isnt_this_too(who: &str, kind: &str, value: &Value) -> String {
    let said = _the_fact_said(kind, value);
    let negative = if crate::memory::relations::_plural_person(who) { " aren't " } else { " isn't " };
    "Your ".to_string() + who + negative + &said + ", then - good to know. It's just you I'll plan for."
}

rx!(WHAT_ABOUT_A_FOOD);

/// Python: bot/memory/store_replies.py:8941 `_the_person_still_in_hand(low, memory)`
///
/// The person with a limit the chat was last told about, on a food ask that names nobody.
pub fn _the_person_still_in_hand(low: &str, memory: &mut Memory) -> String {
    if WHAT_ABOUT_A_FOOD.match_(low).is_none() {
        return String::new();
    }
    let person = mget(memory, "person").clone();
    let who = if person.is_dict() { text::strip(&sget(&person, "who")).to_string() } else { String::new() };
    if who.is_empty() {
        return String::new();
    }
    for known in others(memory).items() {
        if known.get("who") == Some(&Value::from(who.as_str())) && _a_food_word(low, known) {
            return who;
        }
    }
    String::new()
}

static CONDITION_RULES_OUT: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("CONDITION_RULES_OUT").str_map());

/// Python: bot/memory/store_replies.py:8953 `_a_food_word(low, entry)`
///
/// True when the line names a food the limit kept for that person rules out.
pub fn _a_food_word(low: &str, entry: &Value) -> bool {
    let mut ruled: Vec<String> = Vec::new();
    let condition = sget(entry, "condition").to_lowercase();
    if let Some(thing) = CONDITION_RULES_OUT.get(&condition) {
        ruled.push(thing.clone());
    }
    let diet = sget(entry, "diet").to_lowercase();
    if let Some((rule, _)) = DIET_RULES_OUT.get(&diet) {
        ruled.push(rule.clone());
    }
    ruled.iter().any(|thing| !_what_the_food_has(low, thing).0.is_empty())
}

/// Python: bot/memory/store_replies.py:8969 `_other_person_reply(memory, low)`
///
/// What the user told us about someone else, said back: "Your mum doesn't eat meat."
/// (Contract change: the generated stub returned `()`.)
pub fn _other_person_reply(memory: &mut Memory, low: &str) -> Option<String> {
    let undeclared = _the_person_still_in_hand(low, memory);
    let bare_limit_ask = _a_bare_limit_ask(low, memory);
    if !(!undeclared.is_empty()
        || OTHER_FACT.is_match(low)
        || EAT_ASK.is_match(low)
        || CANT_HAVE.is_match(low)
        || CAN_HAVE.is_match(low)
        || CAN_THEY_HAVE.is_match(low)
        || bare_limit_ask)
    {
        return None;
    }
    if CAN_THEY_HAVE_A_FOOD.is_match(low) && !_a_limit_kept_for_them(low, memory) && !bare_limit_ask {
        // Which foods have nuts in them is the cooking entries' reading, not the store's.
        return None;
    }
    let mut who = _someone_else_named(low, memory);
    if who.is_empty() {
        who = undeclared.clone();
    }
    let entry = others(memory).items().iter().find(|k| k.get("who") == Some(&Value::from(who.as_str()))).cloned();
    let Some(entry) = entry else {
        if let Some(offered) = _did_you_mean_the_person(memory, low) {
            return Some(offered);
        }
        // An honest answer about the person, not "you haven't mentioned an allergy to me".
        if _a_bare_limit_ask(low, memory) {
            // Before HELP_ASK, because "what can she not eat" has "can" in it.
            return Some(_nothing_kept_about_them(low));
        }
        if HELP_ASK.is_match(low) {
            return None;
        }
        if who.is_empty() {
            // A bare "she"/"he"/"they" with nobody in the chat behind it.
            if _nobody_else_kept(memory) && !_the_pronoun_in(low).is_empty() && _asks_for_a_limit_not_advice(low) {
                return Some(_nothing_kept_about_them(low));
            }
            return None;
        }
        if OTHER_FACT.is_match(low) {
            return Some("You haven't told me about an allergy of your ".to_string() + &who + ".");
        }
        return _no_diet(&who);
    };
    if OTHER_FACT.is_match(low) {
        let allergies = entry.get("allergies").filter(|v| v.truthy()).cloned().unwrap_or(Value::list());
        let owner = if who.is_empty() { "They".to_string() } else { "Your ".to_string() + &who };
        if !allergies.truthy() {
            return Some("You haven't told me about an allergy of ".to_string() + &(if who.is_empty() { "theirs".to_string() } else { "your ".to_string() + &who }) + ".");
        }
        return Some(owner + &_is_are(&who) + "allergic to " + &py_list(&allergies).iter().map(py_str).collect::<Vec<_>>().join(", ") + ".");
    }
    let diet = entry.get("diet").cloned().unwrap_or(Value::None);
    let allergies = entry.get("allergies").filter(|v| v.truthy()).cloned().unwrap_or(Value::list());
    let condition = entry.get("condition").cloned().unwrap_or(Value::None);
    let food_ask = |low: &str| CANT_HAVE.is_match(low) || CAN_HAVE.is_match(low) || EAT_ASK.is_match(low) || CAN_THEY_HAVE.is_match(low);
    if condition.truthy() && (!undeclared.is_empty() || food_ask(low)) {
        // The condition kept for him is a food fact too.
        if let Some(said) = _condition_is_their_food(&who, &condition, low).filter(|s| !s.is_empty()) {
            if diet.truthy() && !allergies.truthy() {
                return Some(said + " " + &_diet_sentence(&who, &diet));
            }
            return Some(said);
        }
    }
    if diet.truthy() && (!undeclared.is_empty() || food_ask(low)) {
        // "what cant she have" wants the food the diet rules out, not the diet's name again.
        let ruled = _diet_is_their_food(&who, &py_str(&diet), low, EAT_ASK.is_match(low));
        if let Some(ruled) = ruled.filter(|r| !r.is_empty()) {
            if !allergies.truthy() {
                return Some(ruled);
            }
        }
    }
    if diet.truthy() && allergies.truthy() {
        // An allergy kept beside a diet belongs in the answer too.
        return Some(_both_food_facts(&who, &allergies, &diet));
    }
    if !diet.truthy() {
        if allergies.truthy() {
            // The allergy is a food fact, so "what can she eat" is answered from it.
            return Some(_allergy_is_their_food(&who, &allergies, true, low));
        }
        let condition = entry.get("condition").filter(|v| v.truthy()).cloned().unwrap_or(Value::list());
        if condition.truthy() && _a_condition_rules_out(&condition) {
            // A condition that rules a food out is left to the modules that answer meals.
            return None;
        }
        return _no_diet(&who);
    }
    Some(_diet_sentence(&who, &diet))
}

rx!(THE_PRONOUN, A_LIMIT_NAMED, ADVICE_NOT_A_FACT, A_HAVE_QUESTION, STATES_A_LIMIT, A_ASK_FRAME, A_MEAL_WORD);
static NOT_EAT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bnot\s+eat\b", 0));

/// Python: bot/memory/store_replies.py:9116 `_asks_for_a_limit_not_advice(low)`
///
/// True for a line that asks for a kept fact about somebody's allergies or diet.
pub fn _asks_for_a_limit_not_advice(low: &str) -> bool {
    if ADVICE_NOT_A_FACT.is_match(low) {
        return false;
    }
    if STATES_A_LIMIT.is_match(low) && !A_HAVE_QUESTION.is_match(low) && !A_ASK_FRAME.is_match(low) {
        return false;
    }
    if NOT_EAT.is_match(low) {
        // "what can she not eat" asks for the limit from the other side.
        return true;
    }
    if CANT_HAVE.is_match(low) {
        return true;
    }
    A_LIMIT_NAMED.is_match(low) || ASKING_AGAIN.is_match(low)
}

/// Python: bot/memory/store_replies.py:9139 `_the_pronoun_in(low)`
///
/// The pronoun the line named its person with: "she", "he", "they", or "" for none.
pub fn _the_pronoun_in(low: &str) -> String {
    let found: Vec<String> = THE_PRONOUN.finditer(low).iter().map(|m| m.str(1).to_lowercase()).collect();
    for word in &found {
        if ["she", "her", "hers"].contains(&word.as_str()) {
            return "she".into();
        }
        if ["he", "him", "his"].contains(&word.as_str()) {
            return "he".into();
        }
    }
    if found.iter().any(|w| ["they", "them", "their"].contains(&w.as_str())) {
        return "they".into();
    }
    String::new()
}

/// Python: bot/memory/store_replies.py:9153 `_nobody_else_kept(memory)`
///
/// True when the store holds no fact about anybody else (read, never created).
pub fn _nobody_else_kept(memory: &Memory) -> bool {
    !matches!(memory.get("others"), Some(Value::List(items)) if !items.is_empty())
}

/// Python: bot/memory/store_replies.py:9171 `_a_bare_limit_ask(low, memory)`
///
/// A limit asked for about somebody named only by a pronoun, with nobody in the chat behind it.
pub fn _a_bare_limit_ask(low: &str, memory: &Memory) -> bool {
    // A line that also says what it is for is asking for a meal.
    if A_MEAL_WORD.is_match(low) {
        return false;
    }
    _nobody_else_kept(memory) && !_the_pronoun_in(low).is_empty() && _asks_for_a_limit_not_advice(low)
}

/// Python: bot/memory/store_replies.py:9190 `_nothing_kept_about_them(low)`
///
/// What the store says when a limit is asked about a person it was never told about.
pub fn _nothing_kept_about_them(low: &str) -> String {
    let pronoun = _the_pronoun_in(low);
    if OTHER_FACT.is_match(low) {
        let theirs = match pronoun.as_str() {
            "she" => "hers",
            "he" => "his",
            _ => "theirs",
        };
        return "You haven't told me about an allergy of ".to_string() + theirs + " yet \u{2014} what is it? I'll keep it out of anything I suggest.";
    }
    if pronoun == "she" || pronoun == "he" {
        return "You haven't told me how ".to_string() + &pronoun + " eats yet \u{2014} what does " + &pronoun + " avoid? I'll plan the meals round it.";
    }
    "You haven't told me how they eat yet \u{2014} what do they avoid? I'll plan the meals round it.".to_string()
}

static IS_AT_THE_FRONT: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(\s+)is\b", 0));
static DOES_NOT_AT_THE_FRONT: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(\s+)does not\b", 0));

/// Python: bot/memory/store_replies.py:9207 `_did_you_mean_the_person(memory, low)`
///
/// The one person the store holds, offered back when a pronoun fits nobody.
pub fn _did_you_mean_the_person(memory: &mut Memory, low: &str) -> Option<String> {
    let held: Vec<Value> = others(memory).items().iter().filter(|e| e.is_dict() && e.get("who").map(Value::truthy).unwrap_or(false)).cloned().collect();
    if held.len() != 1 {
        return None;
    }
    // The offer answers a pronoun that fits nobody, so the line has to have one.
    if !THEM.is_match(low) {
        return None;
    }
    let one = &held[0];
    let who = py_str(one.get("who").unwrap());
    let has = |k: &str| one.get(k).map(Value::truthy).unwrap_or(false);
    let said = if OTHER_FACT.is_match(low) && has("allergies") {
        "Your ".to_string() + &who + &_is_are(&who) + "allergic to " + &py_list(one.get("allergies").unwrap()).iter().map(py_str).collect::<Vec<_>>().join(", ") + "."
    } else if EAT_ASK.is_match(low) && has("diet") {
        _diet_sentence(&who, one.get("diet").unwrap())
    } else if EAT_ASK.is_match(low) && has("allergies") {
        _allergy_is_their_food(&who, one.get("allergies").unwrap(), true, "")
    } else {
        String::new()
    };
    if said.is_empty() {
        return None;
    }
    let gender = s_or(Some(&crate::memory::relations::_gender_of(&who).val()));
    let pronoun = match gender.as_str() {
        "she" => "She",
        "he" => "He",
        _ => "They",
    };
    let mut tail = text::slice(&said, text::len(&("Your ".to_string() + &who)) as isize, None);
    if pronoun == "They" {
        // "Your partner is allergic to shellfish" said again with a pronoun takes "are".
        tail = IS_AT_THE_FRONT.sub(r"\1are", &tail);
        tail = DOES_NOT_AT_THE_FRONT.sub(r"\1do not", &tail);
    }
    let ask_gender = crate::memory::relations::_ask_gender(low);
    if !ask_gender.is_empty() && !gender.is_empty() && gender != ask_gender {
        // The ask says "he" about a person the store knows as a she: the mismatch is said out loud.
        let him = if gender == "she" { "her" } else { "him" };
        return Some("I've only got your ".to_string() + &who + " down, and " + &pronoun.to_lowercase() + &tail + " Did you mean " + him + "?");
    }
    Some("Did you mean your ".to_string() + &who + "? " + pronoun + &tail)
}

static CONDITION_KEEPS_DOWN: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("CONDITION_KEEPS_DOWN").str_map());

/// Python: bot/memory/store_replies.py:9257 `_condition_is_their_food(who, condition, low="")`
///
/// What the condition kept for a person means for their plate, or None when it means nothing.
pub fn _condition_is_their_food(who: &str, condition: &Value, low: &str) -> Option<String> {
    let said = fold(low);
    let low = fold(&s_or(Some(condition)));
    let owner = if who.is_empty() { "They".to_string() } else { "Your ".to_string() + who };
    let tail = " \u{2014} that's all I have on food for ".to_string() + &_their_pronoun(who) + ".";
    let ruled = CONDITION_RULES_OUT.get(&low).cloned();
    let (named, thing) = match &ruled {
        Some(r) if !r.is_empty() && !said.is_empty() => _what_the_food_has(&said, r),
        _ => (String::new(), String::new()),
    };
    if !named.is_empty() {
        // "can he have cheese" is answered about the cheese.
        return Some(text::upper_first(&named) + " has " + &thing + " in it, and " + &owner.to_lowercase() + &_is_are(who) + &low + ".");
    }
    if let Some(r) = ruled.filter(|r| !r.is_empty()) {
        return Some(owner + &_is_are(who) + &low + ", so nothing with " + &r + " in it" + &tail);
    }
    if let Some(watched) = CONDITION_KEEPS_DOWN.get(&low).filter(|w| !w.is_empty()) {
        return Some(owner + &_is_are(who) + &low + ", so keep the " + watched + " down" + &tail);
    }
    if !low.is_empty() {
        // A condition with no food of its own is still said back.
        return Some(owner + &_is_are(who) + &low + &tail);
    }
    None
}

/// Python: bot/memory/store_replies.py:9281 `_diet_sentence(who, diet)`
///
/// A diet said as a sentence: "Your mum doesn't eat meat.", "They're vegetarian.".
pub fn _diet_sentence(who: &str, diet: &Value) -> String {
    let diet = py_str(diet);
    if diet.starts_with("eats ") || diet.starts_with("no ") {
        // "eats fish" is a sentence of its own: "Your son is eats fish." is not one.
        if !who.is_empty() {
            return "Your ".to_string() + who + " " + &_a_diet_predicate(who, &diet) + ".";
        }
        if diet.starts_with("eats ") {
            return "They eat ".to_string() + &text::slice(&diet, 5, None) + ".";
        }
        return "They don't ".to_string() + &_diet_how(&diet) + " " + &text::slice(&diet, 3, None) + ".";
    }
    if !who.is_empty() {
        return "Your ".to_string() + who + &_is_are(who) + &diet + ".";
    }
    "They're ".to_string() + &diet + "."
}

rx!(A_BIRTHDAY_PLAN, A_RELATIONS_BIRTHDAY, WHOSE_BIRTHDAY, ALLERGY_IN_THE_WORLD, A_HOW_TO_ASK, A_RECALL_WORD);
strset!(NO_CAP, CAR_DESCRIBED);
static CAR_CAPS: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("CAR_CAPS").str_map());

/// Python: bot/memory/store_replies.py:9310 `car_line(car)`
///
/// The car said back, in one shape wherever it is asked for.
pub fn car_line(car: &Value) -> String {
    use crate::memory::relations::_a;
    let said = text::squash(&s_or(Some(car)));
    if said.is_empty() {
        return String::new();
    }
    let words = text::split(&said);
    if words.iter().any(|w| CAR_DESCRIBED.contains(&w.to_lowercase())) {
        return "You drive ".to_string() + &_a(&said) + ".";
    }
    if words.len() == 1 && NO_CAP.contains(&said.to_lowercase()) {
        return "You drive ".to_string() + &_a(&said) + ".";
    }
    let shown = words
        .iter()
        .map(|w| match CAR_CAPS.get(&w.to_lowercase()) {
            Some(c) => c.clone(),
            None => {
                if text::is_lower(w) {
                    text::upper_first(w)
                } else {
                    w.clone()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    "You drive ".to_string() + &_a(&shown) + "."
}

static FACTS_BIRTHDAY: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.facts.BIRTHDAY"));
static IN_OR_ON: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:in|on)\s+", 0));

/// Python: bot/memory/store_replies.py:9333 `_birthday_in_words(memory)`
///
/// A birthday they gave in words rather than as a date: "my birthday is tomorrow".
pub fn _birthday_in_words(memory: &mut Memory) -> String {
    let plans = user(memory).get("plans").map(py_list).unwrap_or_default();
    for plan in plans.iter().rev() {
        let said = py_str(plan);
        let found = FACTS_BIRTHDAY.search(&said).or_else(|| A_BIRTHDAY_PLAN.search(&said));
        let Some(m) = found else { continue };
        let subject = crate::memory::facts::subject(&said, text::char_index(&said, m.start()) as i64);
        if subject.first().map(String::as_str) == Some("other") {
            continue;
        }
        if A_RELATIONS_BIRTHDAY.is_match(&said) {
            // "my sons birthday is tomorrow": the relation in front of it says whose date it is.
            continue;
        }
        let words = text::rstrip_chars(&fold(m.str(1)), " .,;:").to_string();
        let words = IN_OR_ON.sub("", &words);
        if !words.is_empty() {
            return words;
        }
    }
    String::new()
}

static A_DIGIT_START: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?:the\s+)?\d", 0));

/// Whether a reply is one of the bot's lines in this chat (`plain in {turn.get("bot") ...}`).
fn said_before(memory: &Memory, plain: &str) -> bool {
    memory.turns.iter().any(|t| t.bot == plain)
}

/// Python: bot/memory/store_replies.py:9356 `_birthday_reply(memory)`
///
/// The birthday they gave, in the words they would use for it.
pub fn _birthday_reply(memory: &mut Memory) -> String {
    let known = user(memory);
    let date = known.get("birthday").cloned().unwrap_or(Value::None);
    let words = if date.truthy() { s_or(Some(&crate::memory::corrected::_birthday_words(&date).val())) } else { _birthday_in_words(memory) };
    if words.is_empty() {
        return "You haven't told me your birthday yet.".to_string();
    }
    // A birthday is a date, and a date is answered the way every other date in the store is.
    let plain = if words.starts_with("in ") {
        text::upper_first(&words) + "."
    } else if A_DIGIT_START.match_(&words).is_some() {
        "On ".to_string() + &words + "."
    } else {
        "Your birthday is ".to_string() + &words + "."
    };
    if said_before(memory, &plain) {
        return _vary(
            &["It's ".to_string() + &words + " \u{2014} that's what you told me.", "You said ".to_string() + &words + ".", "Still ".to_string() + &words + "."],
            memory,
        );
    }
    plain
}

/// Python: bot/memory/store_replies.py:9389 `_the_same_relative(one, other)`
///
/// Whether two words are the same person's relation: "mum" and "mums" are one mum.
pub fn _the_same_relative(one: &str, other: &str) -> bool {
    let one = one.to_lowercase();
    let other = other.to_lowercase();
    if one.is_empty() || other.is_empty() {
        return false;
    }
    if one == other {
        return true;
    }
    (other.ends_with('s') && one == text::slice(&other, 0, Some(-1))) || (one.ends_with('s') && text::slice(&one, 0, Some(-1)) == other)
}

/// Python: bot/memory/store_replies.py:9396 `_a_persons_birthday_reply(memory, low)`
///
/// The date the store holds for the person the ask names, said back, or None.
pub fn _a_persons_birthday_reply(memory: &mut Memory, low: &str) -> Option<String> {
    let asked = WHOSE_BIRTHDAY.search(low)?;
    let mut who = crate::memory::facts::a_person_word(asked.name("who").unwrap_or(""));
    let pronoun = asked.name("pronoun").unwrap_or("").to_lowercase();
    let mut named = asked.name("name").unwrap_or("").to_lowercase();
    if ["birthday", "bday", "dob", "date"].contains(&who.as_str()) {
        return None;
    }
    let told = others(memory);
    let mut known: Option<Value> = None;
    if !who.is_empty() {
        known = told.items().iter().find(|e| _the_same_relative(&s_or(e.get("who")), &who)).cloned();
        if let Some(k) = &known {
            let w = s_or(k.get("who"));
            if !w.is_empty() {
                who = w;
            }
        }
    } else if !named.is_empty() {
        known = told
            .items()
            .iter()
            .find(|e| {
                let name = e.get("name").filter(|n| n.truthy()).cloned().unwrap_or(Value::from(""));
                crate::memory::relations::_same_name(&name, &named)
            })
            .cloned();
    } else if !pronoun.is_empty() {
        // "when is he/she ...": the last person the chat was about whose pronoun fits.
        let fits: &HashSet<String> = if pronoun == "her" { &FACTS_SHE } else { &FACTS_HE };
        known = told.items().iter().rev().find(|e| fits.contains(&s_or(e.get("who")).to_lowercase())).cloned();
        if let Some(k) = &known {
            who = s_or(k.get("who"));
            named = s_or(k.get("name"));
        } else {
            let word = if pronoun == "her" { "she" } else { "he" };
            known = told.items().iter().find(|e| sget(e, "pronoun") == word && e.get("birthday").map(Value::truthy).unwrap_or(false)).cloned();
        }
    }
    let known = known?;
    let birthday = known.get("birthday").filter(|b| b.truthy())?.clone();
    let words = s_or(Some(&crate::memory::corrected::_birthday_words(&birthday).val()));
    let mut said_name = s_or(known.get("name"));
    if said_name.is_empty() {
        said_name = named.clone();
    }
    if !said_name.is_empty() && who.is_empty() {
        // The date is said with the name as a name reads.
        let said_name = text::upper_first(&said_name);
        return Some(_vary(
            &[said_name.clone() + "'s birthday is " + &words + ".", "That's ".to_string() + &words + " for " + &said_name + "."],
            memory,
        ));
    }
    if who.is_empty() {
        return None;
    }
    // A word for more than one person takes the apostrophe after it.
    let mark = if crate::memory::relations::_plural_person(&who) { "'" } else { "'s" };
    Some(_vary(
        &[
            "Your ".to_string() + &who + mark + " birthday is " + &words + ".",
            "That's ".to_string() + &words + " for your " + &who + ".",
            "Your ".to_string() + &who + mark + " birthday: " + &words + ".",
        ],
        memory,
    ))
}

/// Python: bot/memory/store_replies.py:9462 `_weight_reply(memory)`
///
/// What they weigh, in the unit they gave it in.
pub fn _weight_reply(memory: &mut Memory) -> String {
    let known = user(memory);
    let weight = known.get("weight").cloned().unwrap_or(Value::None);
    if !weight.truthy() {
        return "You haven't told me what you weigh.".to_string();
    }
    let weight = py_str(&weight);
    let plain = "You weigh ".to_string() + &weight + ".";
    if said_before(memory, &plain) {
        return _vary(
            &["Still ".to_string() + &weight + ".", "You said ".to_string() + &weight + ".", "It's ".to_string() + &weight + " \u{2014} that's what you told me."],
            memory,
        );
    }
    plain
}

/// Python: bot/memory/store_replies.py:9500 `_height_reply(memory)`
///
/// How tall they are, in the unit they gave it in.
pub fn _height_reply(memory: &mut Memory) -> String {
    let known = user(memory);
    let height = text::strip(&sget(&known, "height")).to_string();
    if height.is_empty() {
        return "You haven't told me how tall you are.".to_string();
    }
    let plain = "You're ".to_string() + &height + ".";
    if said_before(memory, &plain) {
        return _vary(
            &["Still ".to_string() + &height + ".", "You said ".to_string() + &height + ".", "It's ".to_string() + &height + " \u{2014} that's what you told me."],
            memory,
        );
    }
    plain
}

/// `relations._called(name)` on any value: a name that is not a str is "" (as Python's is).
fn called_of(v: Option<&Value>) -> String {
    match v {
        Some(Value::Str(s)) => crate::memory::relations::_called(s),
        _ => String::new(),
    }
}

static DRIVE_ASK: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.recall.DRIVE_ASK"));
static CALLED_OR_NAME: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:called|name)\b", 0));
static HOW_MANY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhow many\b", 0));
static WAS_WERE_DID: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:was|were|did)\b", 0));
static CALL_NAME_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:call|called|name|named)\b", 0));
/// The animal an ask names, and the words for it (`_fact_reply`'s own table).
static ANIMAL_ASKED: LazyLock<Vec<(&'static str, Re)>> = LazyLock::new(|| {
    [
        ("dog", "dog puppy"),
        ("cat", "cat kitten"),
        ("rabbit", "rabbit bunny"),
        ("hamster", "hamster"),
        ("fish", "fish goldfish"),
        ("horse", "horse pony"),
        ("parrot", "parrot bird"),
    ]
    .into_iter()
    .map(|(animal, words)| (animal, Re::new(&format!(r"\b(?:{})s?\b", words.replace(' ', "|")), 0)))
    .collect()
});

/// `d.get(key, default)`: the value when the key is there (whatever it is), else the default.
fn get_or(d: &Value, key: &str, default: &str) -> String {
    match d.get(key) {
        Some(v) => py_str(v),
        None => default.to_string(),
    }
}

/// `pet.get("count") if isinstance(pet.get("count"), int) else 1`.
fn int_count(pet: &Value, default: i64) -> i64 {
    match pet.get("count") {
        Some(Value::Int(i)) => *i,
        Some(Value::Bool(b)) => *b as i64,
        _ => default,
    }
}

/// Python: bot/memory/store_replies.py:9514 `_fact_reply(memory, low, key=None)`
///
/// The reply for one of their facts. `key` is the fact already read from the ask.
pub fn _fact_reply(memory: &mut Memory, low: &str, key: Option<&str>) -> Option<String> {
    use crate::memory::relations::{_a, _animal_family};
    // A how-to in front of the fact word is advice, not a recall of it.
    if !low.is_empty() && A_HOW_TO_ASK.is_match(low) && !A_RECALL_WORD.is_match(low) {
        return None;
    }
    let known = user(memory);
    if key == Some("allergy") || low.contains("allerg") {
        if ALLERGY_IN_THE_WORLD.is_match(low) {
            return None;
        }
        let allergies = known.get("allergies").filter(|v| v.truthy()).cloned().unwrap_or(Value::list());
        if allergies.truthy() {
            let said = py_list(&allergies).iter().map(py_str).collect::<Vec<_>>().join(", ");
            let plain = "You told me you're allergic to ".to_string() + &said + ".";
            // Asked twice in a row, the phrasing rotates.
            if said_before(memory, &plain) {
                return Some(_vary(
                    &["It's ".to_string() + &said + " \u{2014} that's what you said.", "Still ".to_string() + &said + ".", "Your allergy is ".to_string() + &said + "."],
                    memory,
                ));
            }
            return Some(plain);
        }
        if known.get("allergies").is_some() {
            // The list is still there and empty: they named one and then took it back.
            let fixed = mget(memory, "correction").clone();
            let gone = text::strip(&sget(&fixed, "value")).to_lowercase();
            if !gone.is_empty() && fixed.get("dropped").map(Value::truthy).unwrap_or(false) {
                return Some("Nothing on the list now \u{2014} you took the ".to_string() + &gone + " one back.");
            }
            return Some("Nothing on the list now.".to_string());
        }
        let (who, theirs) = _other_fact(memory, "allergies");
        if !theirs.is_empty() && !who.is_empty() {
            return Some(_their_allergy(&who, &theirs));
        }
        if !theirs.is_empty() {
            return Some(
                "You haven't mentioned one for yourself \u{2014} the allergy to ".to_string()
                    + &theirs.iter().map(py_str).collect::<Vec<_>>().join(", ")
                    + " you mentioned was someone else's.",
            );
        }
        return Some(_deny(memory, "allergy", "You haven't mentioned an allergy to me.", None));
    }
    if key == Some("budget") || low.contains("budget") {
        let budget = known.get("budget").cloned().unwrap_or(Value::None);
        if !budget.truthy() {
            return Some(_deny(memory, "budget", "You haven't told me a budget.", None));
        }
        let budget = py_str(&budget);
        // The first answer is the plain sentence; asked again, a different phrasing.
        let plain = "Your budget is ".to_string() + &budget + ".";
        if said_before(memory, &plain) {
            return Some(_vary(
                &["It's ".to_string() + &budget + " \u{2014} that's what you told me.", "You said ".to_string() + &budget + ".", "Still ".to_string() + &budget + "."],
                memory,
            ));
        }
        return Some(plain);
    }
    match key {
        Some("birthday") => return Some(_birthday_reply(memory)),
        Some("weight") => return Some(_weight_reply(memory)),
        Some("height") => return Some(_height_reply(memory)),
        _ => {}
    }
    if key == Some("diet") || (DIET_ASK.is_match(low) && !DIET_HOW_TO.is_match(low)) {
        return _diet_reply(memory, low);
    }
    if key == Some("car") || DRIVE_ASK.is_match(low) {
        if let Some(car) = known.get("car").filter(|c| c.truthy()) {
            return Some(car_line(car));
        }
        return Some(pick(&["You haven't told me what you drive.", "I don't know your car \u{2014} you haven't said."], memory, ""));
    }
    let mut pets: Vec<Value> = known.get("pets").filter(|v| v.truthy()).map(py_list).unwrap_or_default();
    let mut gone: Vec<Value> = known.get("pets_gone").filter(|v| v.truthy()).map(py_list).unwrap_or_default().into_iter().filter(|p| p.is_dict()).collect();
    let mut asked: Option<&str> = None;
    // "what is my dog called?" is about the dog, not the cat: read the animal they named.
    for (animal, pattern) in ANIMAL_ASKED.iter() {
        if pattern.is_match(low) {
            asked = Some(animal);
            pets.retain(|p| _animal_family(&s_or(p.get("animal"))) == *animal);
            gone.retain(|p| _animal_family(&s_or(p.get("animal"))) == *animal);
            break;
        }
    }
    let coming_only: Vec<&Value> = pets.iter().filter(|p| p.is_dict() && p.get("coming").map(Value::truthy).unwrap_or(false)).collect();
    if !coming_only.is_empty() && coming_only.len() == pets.len() && !CALLED_OR_NAME.is_match(low) {
        // An animal that is on its way: the sentence says it is coming rather than here.
        let pieces: Vec<String> = coming_only
            .iter()
            .map(|pet| {
                let count = int_count(pet, 1);
                if count > 1 {
                    let animal = pet.get("animal").filter(|a| a.truthy()).cloned().unwrap_or(Value::from("pet"));
                    crate::memory::corrected::_many(count, &animal)
                } else {
                    _a_pet(pet)
                }
            })
            .collect();
        let due = _put_together(&pieces);
        if !due.is_empty() {
            return Some("You're getting ".to_string() + &due + ".");
        }
    }
    let counted = _a_count_ask(low);
    if !pets.is_empty() {
        if let Some(counted) = &counted {
            // "how many cats" asks for the number, not for the list.
            let kinds: HashSet<String> = pets.iter().map(|p| _animal_family(&s_or(p.get("animal")))).collect();
            let total = _count_of_a_kind(&pets);
            if kinds.len() == 1 && (counted == "any" || counted == "pet" || kinds.contains(counted)) {
                let animal = _animal_family(&s_or(pets[0].get("animal")));
                if total == 1 {
                    // One animal, named: the number and the name both survive.
                    let was = called_of(pets[0].get("name"));
                    if !was.is_empty() {
                        return Some("You've got one ".to_string() + &animal + " \u{2014} " + &was + ".");
                    }
                    return Some("You've got ".to_string() + &_a(&animal) + ".");
                }
                return Some("You've got ".to_string() + &_a_pet(&dict_of(vec![("animal", Value::from(animal.as_str())), ("count", Value::Int(total))])) + ".");
            }
        }
    }
    if !pets.is_empty() {
        if CALLED_OR_NAME.is_match(low) && !pets.iter().any(|p| p.get("name").map(Value::truthy).unwrap_or(false)) {
            return Some("You haven't told me a name for your ".to_string() + &get_or(&pets[0], "animal", "pet") + " yet.");
        }
        if let Some(together) = _pets_together(&pets).filter(|t| !t.is_empty()) {
            return Some("You've got ".to_string() + &together + ".");
        }
        // "how many dogs do i have" asks for the number, so the number is the answer.
        let counted = HOW_MANY.is_match(low);
        if counted && pets.len() == 1 {
            let pet = &pets[0];
            let animal = pet.get("animal").filter(|a| a.truthy()).cloned().unwrap_or(Value::from("pet"));
            let number = int_count(pet, 1).max(1);
            let many = if number > 1 { crate::memory::corrected::_many(number, &animal) } else { "one ".to_string() + &py_str(&animal) };
            let was = called_of(pet.get("name"));
            return Some("You've got ".to_string() + &many + &(if was.is_empty() { ".".to_string() } else { " \u{2014} ".to_string() + &was + "." }));
        }
        if counted && pets.len() > 1 && pets.iter().map(|p| _animal_family(&s_or(p.get("animal")))).collect::<HashSet<_>>().len() == 1 {
            let top = pets.iter().filter_map(|p| match p.get("count") {
                Some(Value::Int(i)) => Some(*i),
                Some(Value::Bool(b)) => Some(*b as i64),
                _ => None,
            }).max().unwrap_or(0);
            if top >= pets.len() as i64 {
                // The count is the whole set.
                let animal = pets[0].get("animal").filter(|a| a.truthy()).cloned().unwrap_or(Value::from("pet"));
                return Some("You've got ".to_string() + &crate::memory::corrected::_many(top, &animal) + ".");
            }
        }
        let parts: Vec<String> = pets
            .iter()
            .map(|p| {
                let count = match p.get("count") {
                    Some(Value::Int(i)) => Some(*i),
                    Some(Value::Bool(b)) => Some(*b as i64),
                    _ => None,
                };
                match count {
                    Some(c) if counted && c > 1 => {
                        let animal = p.get("animal").filter(|a| a.truthy()).cloned().unwrap_or(Value::from("pet"));
                        crate::memory::corrected::_many(c, &animal)
                    }
                    _ => _a_pet(p),
                }
            })
            .collect();
        // An animal on its way is not one in the house.
        let coming = |p: &Value| p.get("coming").map(Value::truthy).unwrap_or(false);
        let here: Vec<String> = pets.iter().zip(parts.iter()).filter(|(p, _)| !coming(p)).map(|(_, s)| s.clone()).collect();
        let due: Vec<String> = pets.iter().zip(parts.iter()).filter(|(p, _)| coming(p)).map(|(_, s)| s.clone()).collect();
        let mut said: Vec<String> = Vec::new();
        if !here.is_empty() {
            said.push("You've got ".to_string() + &_put_together(&here) + ".");
        }
        if !due.is_empty() {
            said.push("You're getting ".to_string() + &_put_together(&due) + ".");
        }
        if !said.is_empty() {
            return Some(said.join(" "));
        }
    }
    if !gone.is_empty() {
        // "what was my cat called?" asks after a death: the name is the thing they want back.
        let was_called = WAS_WERE_DID.is_match(low) && CALL_NAME_WORDS.is_match(low);
        let named: Vec<Value> = gone.iter().filter(|p| p.get("name").map(Value::truthy).unwrap_or(false)).cloned().collect();
        if was_called && !named.is_empty() {
            return Some(_was_called(&named));
        }
        // A pet they told me about and then lost is not a pet they never mentioned.
        return Some(_a_pet_that_is_gone(&gone));
    }
    if let Some(asked) = asked {
        // They do have pets, just not that one.
        let theirs: Vec<String> = known
            .get("pets")
            .filter(|v| v.truthy())
            .map(py_list)
            .unwrap_or_default()
            .iter()
            .map(|p| {
                _a(&get_or(p, "animal", "pet"))
                    + &(if p.get("name").map(Value::truthy).unwrap_or(false) { " called ".to_string() + &called_of(p.get("name")) } else { String::new() })
            })
            .collect();
        if !theirs.is_empty() {
            return Some("I don't have ".to_string() + &_a(asked) + " from you \u{2014} " + &theirs.join(" and ") + " is what I have.");
        }
    }
    Some("You haven't told me about a pet yet.".to_string())
}

/// Python: bot/memory/store_replies.py:9671 `_was_called(named)`
///
/// The name of a pet that is gone: "Your cat was called Milly.".
pub fn _was_called(named: &[Value]) -> String {
    let animal_of = |p: &Value| {
        let a = sget(p, "animal");
        if a.is_empty() {
            "pet".to_string()
        } else {
            a
        }
    };
    if named.len() == 1 {
        return "Your ".to_string() + &animal_of(&named[0]) + " was called " + &called_of(named[0].get("name")) + ".";
    }
    let parts: Vec<String> = named.iter().map(|p| animal_of(p) + " " + &called_of(p.get("name"))).collect();
    let last = parts.last().expect("IndexError: list index out of range");
    "You called them ".to_string() + &parts[..parts.len() - 1].join(", ") + " and " + last + "."
}

/// Python: bot/memory/store_replies.py:9679 `_a_pet_that_is_gone(gone)`
///
/// A pet they no longer have, said back the way they told me: died, or gone elsewhere.
pub fn _a_pet_that_is_gone(gone: &[Value]) -> String {
    let one = |pet: &Value| {
        let mut animal = sget(pet, "animal");
        if animal.is_empty() {
            animal = "pet".into();
        }
        let name = called_of(pet.get("name"));
        if name.is_empty() {
            animal
        } else {
            animal + " " + &name
        }
    };
    let shown: Vec<String> = gone.iter().map(one).collect();
    let which = match shown.len() {
        n if n > 2 => "any pets".to_string(),
        2 => "your ".to_string() + &shown[0] + " or your " + &shown[1],
        _ => "your ".to_string() + shown.first().expect("IndexError: list index out of range"),
    };
    let died = |p: &Value| p.get("gone") == Some(&Value::from("died"));
    if gone.iter().all(died) {
        return "I'm sorry \u{2014} you told me ".to_string() + &which + " died.";
    }
    if !gone.iter().any(died) {
        return "You told me you don't have ".to_string() + &which + " any more.";
    }
    let told_died: Vec<String> = gone.iter().enumerate().filter(|(_, p)| died(p)).map(|(i, _)| "your ".to_string() + &shown[i]).collect();
    "I'm sorry \u{2014} you told me ".to_string() + &(if told_died.len() > 1 { told_died.join(" or ") } else { told_died[0].clone() }) + " died."
}

/// Python: bot/memory/store_replies.py:9701 `_put_together(parts)`
///
/// The pieces of a pet sentence as one phrase: "a dog", "a dog and a cat", "a, b and c".
pub fn _put_together(parts: &[String]) -> String {
    let parts: Vec<&String> = parts.iter().filter(|p| !p.is_empty()).collect();
    match parts.len() {
        0 => String::new(),
        1 => parts[0].clone(),
        n => parts[..n - 1].iter().map(|p| p.as_str()).collect::<Vec<_>>().join(", ") + " and " + parts[n - 1],
    }
}

/// The plural a pet word takes: "fish", "sheep" and "deer" keep their own.
fn pet_plural(animal: &str) -> String {
    if text::ends_with_any(animal, &["fish", "sheep", "deer"]) {
        animal.to_string()
    } else {
        animal.to_string() + "s"
    }
}

/// Python: bot/memory/store_replies.py:9710 `_pets_together(pets)`
///
/// Two or more of the same animal, each named: "two cats called Tom and Jerry", or None.
pub fn _pets_together(pets: &[Value]) -> Option<String> {
    let animals: Vec<String> = pets
        .iter()
        .map(|p| {
            let a = p.get("animal").filter(|a| a.truthy()).cloned().unwrap_or(Value::from("pet"));
            py_str(&a)
        })
        .collect();
    if pets.len() < 2 || animals.iter().collect::<HashSet<_>>().len() != 1 {
        return None;
    }
    if !pets.iter().all(|p| !p.get("count").map(Value::truthy).unwrap_or(false)) {
        return None;
    }
    let named: Vec<&Value> = pets.iter().filter(|p| p.get("name").map(Value::truthy).unwrap_or(false)).collect();
    let nameless = pets.len() - named.len();
    if named.is_empty() {
        return None;
    }
    let many = pet_plural(&animals[0]);
    let count = pets.len() as i64;
    let number = NUMBER_WORDS.get(&count).cloned().unwrap_or_else(|| count.to_string());
    let mut names: Vec<String> = named.iter().map(|p| called_of(p.get("name"))).collect();
    if nameless == 1 {
        // A second animal they have not named yet: the whole set said once, with the gap beside it.
        names.push("one you haven't named".to_string());
        return Some(number + " " + &many + " \u{2014} " + &_put_together(&names));
    }
    if nameless > 1 {
        return None;
    }
    let last = names.last().unwrap().clone();
    Some(number + " " + &many + " called " + &names[..names.len() - 1].join(", ") + " and " + &last)
}

/// Python: bot/memory/store_replies.py:9755 `_a_pet(pet)`
///
/// "a cat", "a cat called Milly", "two cats": a number they gave is said back as a number.
pub fn _a_pet(pet: &Value) -> String {
    use crate::memory::relations::_a;
    // The breed they named comes first.
    let animal = {
        let b = pet.get("breed").filter(|v| v.truthy()).or_else(|| pet.get("animal").filter(|v| v.truthy()));
        match b {
            Some(v) => py_str(v),
            None => "pet".to_string(),
        }
    };
    let count = pet.get("count").filter(|v| v.truthy()).cloned().unwrap_or(Value::Int(1));
    if pet.get("name").map(Value::truthy).unwrap_or(false) {
        let many = py_int_value(&count).map(|c| c > 1).unwrap_or(false);
        if many {
            // Two cats with one name between them: the number they gave stays in the reply.
            let n = py_int_value(&count).unwrap();
            let number = NUMBER_WORDS.get(&n).cloned().unwrap_or_else(|| py_str(&count));
            return number + " " + &pet_plural(&animal) + ", one called " + &called_of(pet.get("name"));
        }
        return _a(&animal) + " called " + &called_of(pet.get("name"));
    }
    let n = py_int_value(&count).expect("ValueError: invalid literal for int()");
    if n > 1 {
        let number = NUMBER_WORDS.get(&n).cloned().unwrap_or_else(|| py_str(&count));
        return number + " " + &pet_plural(&animal);
    }
    _a(&animal)
}

rx!(ADVICE_ASK);
static WHY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bwhy\b", 0));
static MY_PET_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bmy (?:dog|cat|pet)s?\b", 0));
static MY_FACT_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bmy (?:dog|cat|pet)s?\b|\ballerg\w*\b|\bmy budget\b", 0));
static WHAT_DID_I: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bwhat (?:did|have) i\b", 0));
static SUBJECT_KEYS: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("_SUBJECT_KEYS").str_map());

/// Python: bot/memory/store_replies.py:9782 `_a_why_line_they_have_answered(text, memory)`
///
/// A why-line the store can answer, because it holds the subject the line names.
pub fn _a_why_line_they_have_answered(text_: &str, memory: &mut Memory) -> bool {
    let low = fold(text_);
    WHY.is_match(&low) && _holds_the_subject(memory, text_)
}

/// Python: bot/memory/store_replies.py:9787 `_holds_the_subject(memory, text)`
///
/// Whether the store has the thing a why-line names, so it can say what they said.
pub fn _holds_the_subject(memory: &mut Memory, text_: &str) -> bool {
    let Some(key) = crate::memory::recall::ask(text_, true).filter(|k| !k.is_empty()) else { return false };
    let known = user(memory);
    let slot = SUBJECT_KEYS.get(&key).cloned().unwrap_or_else(|| key.clone());
    let held = known.get(&slot).map(Value::truthy).unwrap_or(false);
    if key == "limit" {
        return held || known.get("diet").map(Value::truthy).unwrap_or(false) || known.get("limits").map(Value::truthy).unwrap_or(false);
    }
    held
}

/// Python: bot/memory/store_replies.py:9804 `_asks_for_a_fact(low, text="", memory=None)`
///
/// True when the line asks for one of their own facts, in the words they use for it.
pub fn _asks_for_a_fact(low: &str, text_: &str, memory: &mut Memory) -> bool {
    if MY_PET_WORD.is_match(low) && ADVICE_ASK.is_match(low) {
        return false;
    }
    // A why-ask is a read of the store only when it holds the subject.
    if WHY.is_match(low) {
        return _holds_the_subject(memory, text_);
    }
    if MY_FACT_WORD.is_match(low) {
        return true;
    }
    if WHAT_DID_I.is_match(low) {
        return false;
    }
    ASK_WITHOUT_MY.is_match(low)
}

/// Python: bot/memory/store_replies.py:9829 `_vary(options, memory)`
///
/// pick(), but over the whole reply: a lead-in and a body put together can repeat a line the bot
/// said, even when the pieces on their own do not.
pub fn _vary(options: &[String], memory: &Memory) -> String {
    let turns = &memory.turns;
    let recent: Vec<&str> = turns[turns.len().saturating_sub(5)..].iter().map(|t| t.bot.as_str()).collect();
    let fresh: Vec<String> = options.iter().filter(|o| !recent.contains(&o.as_str())).cloned().collect();
    let fresh = if fresh.is_empty() { options.to_vec() } else { fresh };
    crate::py::random::Random::new(turns.len() as i128).choice(&fresh).clone()
}

/// Python: bot/memory/store_replies.py:9837 `_turn_lines(memory)`
///
/// The user and bot lines of the earlier turns, oldest first.
pub fn _turn_lines(memory: &Memory) -> Vec<(String, String)> {
    memory.turns.iter().map(|t| (t.user.clone(), t.bot.clone())).collect()
}

rx!(FIGURE_REPEAT);

/// Python: bot/memory/store_replies.py:9866 `asks_again(text, memory=None)`
///
/// True when the whole line asks to hear the bot's last reply again.
pub fn asks_again(text_: &str, memory: &Memory) -> bool {
    asks_again_m(text_, Some(memory))
}

/// `asks_again` with Python's `memory=None` expressible.
pub fn asks_again_m(text_: &str, memory: Option<&Memory>) -> bool {
    let low = text::strip(text_).to_lowercase();
    !low.is_empty() && _repeat_ask_m(&low, &low, memory)
}

static A_HAND_BACK_LEAD: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:sure|again|of course)\s*[—-]\s*|^here it is again:\s*", crate::py::re::I));

/// The bot lines a repeat reads, a take-back's acknowledgement passed over.
fn _said_lines(memory: &Memory) -> Vec<Turn> {
    let mut said: Vec<Turn> = memory.turns.iter().filter(|t| !text::strip(&t.bot).is_empty()).cloned().collect();
    if let Some(last) = said.last() {
        if crate::memory::named::takes_back(&last.user) {
            let answered: Vec<Turn> = said[..said.len() - 1].iter().filter(|t| !crate::memory::named::takes_back(&t.user)).cloned().collect();
            if !answered.is_empty() {
                said = answered;
            }
        }
    }
    said
}

/// Python: bot/memory/store_replies.py:9877 `_repeat_reply(memory, text="")`
///
/// The bot's last answer said again, for "say that again".
pub fn _repeat_reply(memory: &mut Memory, text_: &str) -> Option<String> {
    // A line that takes something back answered nothing, so the answer to repeat is the last one
    // that answered something.
    let said = _said_lines(memory);
    let whole = WHOLE_ASK.match_(&text::strip(text_).to_lowercase()).is_some();
    if said.is_empty() {
        return Some(_vary(
            &["I haven't said anything yet \u{2014} you go first.", "Nothing from me so far. What would you like to talk about?"].map(str::to_string),
            memory,
        ));
    }
    let line = memory.understood.as_ref().map(|u| u.text.clone()).unwrap_or_default();
    let about = _hearing_again(&said, &line);
    let answer_module = |t: &Turn| t.module.as_deref().map(|m| ANSWER_MODULES.contains(m)).unwrap_or(false);
    if whole && about.as_ref().map(|t| !answer_module(t)).unwrap_or(true) {
        // A request for the whole of it with no answer behind it: the store says nothing at all.
        return None;
    }
    let Some(about) = about else {
        // Nothing behind the ask: no answer and nothing that carried content.
        return Some(_vary(
            &[
                "Nothing from me on that yet. What were you asking about?",
                "I haven't given you an answer on that yet. What is it you want to know?",
                "Nothing of mine to give back there yet - what was the question?",
            ]
            .map(str::to_string),
            memory,
        ));
    };
    if FIGURE_REPEAT.is_match(text_) && !A_DIGIT.is_match(&about.bot) && !answer_module(&about) {
        // The ask names a length and the line it would hand back has no length in it.
        return Some(_vary(
            &["I haven't said a length yet - what are you timing?", "Nothing with a time in it from me so far. What are you looking at?"].map(str::to_string),
            memory,
        ));
    }
    let last = about.bot.clone();
    // A frame this codebase puts on a hand-back is taken off first, so asking twice does not stack.
    let base = crate::memory::again::unframed(&last).filter(|u| !u.is_empty()).unwrap_or(last);
    let mut base = A_HAND_BACK_LEAD.sub("", &base);
    if FIGURE_REPEAT.is_match(text_) {
        // The ask names the figure, so the figure is the answer.
        if let Some(figure) = crate::memory::narrow::narrow(&base, text_, "").filter(|f| !text::strip(f).is_empty()) {
            base = text::strip(&figure).to_string();
        }
    }
    if !answer_module(&about) {
        // Not an answer to give again, so nothing is promised: the line is said as it was.
        return Some(base);
    }
    if WHOLE_ASK.match_(&text::strip(text_).to_lowercase()).is_some() {
        // The person asked for the whole of it, so the lead-in says that is what this is.
        return Some(_vary(
            &["All of it: ".to_string() + &base, "Here it is in full: ".to_string() + &base, "The whole of it, then: ".to_string() + &base],
            memory,
        ));
    }
    Some(_vary(&["Sure \u{2014} ".to_string() + &base, "Again \u{2014} ".to_string() + &base, "Here it is again: ".to_string() + &base, base.clone()], memory))
}

/// Python: bot/memory/store_replies.py:9946 `_a_whole_answer_to_hand_back(memory, text="")`
///
/// True when the chat's last answer is one this store can hand over whole.
pub fn _a_whole_answer_to_hand_back(memory: &Memory, text_: &str) -> bool {
    let said: Vec<Turn> = memory.turns.iter().filter(|t| !text::strip(&t.bot).is_empty()).cloned().collect();
    if said.is_empty() {
        return false;
    }
    let said = _said_lines(memory);
    match _hearing_again(&said, text_) {
        Some(about) => about.module.as_deref().map(|m| ANSWER_MODULES.contains(m)).unwrap_or(false),
        None => false,
    }
}

/// Python: bot/memory/store_replies.py:9961 `_carries_something(turn)`
///
/// True when a bot turn has something in it to hear again.
pub fn _carries_something(turn: &Turn) -> bool {
    let reply = text::strip(&turn.bot);
    if reply.is_empty() {
        return false;
    }
    if _about_this_chat_m(&turn.user, None) {
        // The bot's own paperwork is not something the person asked to hear again.
        return false;
    }
    turn.module.as_deref().map(|m| ANSWER_MODULES.contains(m)).unwrap_or(false) || text::split(reply).len() >= 8
}

strset!(NOD_WORDS, HELLO_FILLER, GREET, ACKS);

/// Python: bot/memory/store_replies.py:9982 `_only_a_nod(line)`
///
/// True for a line that says nothing but thank you or so: "thanks", "got it", "ok cheers".
pub fn _only_a_nod(line: &str) -> bool {
    let found = LETTER_WORDS.findall_str(&line.to_lowercase());
    if found.is_empty() || found.len() > 4 {
        return false;
    }
    found.iter().all(|w| NOD_WORDS.contains(w))
}

/// Python: bot/memory/store_replies.py:9989 `_hearing_again(said, line)`
///
/// The bot turn a repeat ask is about: the last line, or the last one with content in it.
pub fn _hearing_again(said: &[Turn], line: &str) -> Option<Turn> {
    if LAST_LINE_ASK.is_match(line) {
        return Some(said.last().expect("IndexError: list index out of range").clone());
    }
    let mut said: Vec<Turn> = said.to_vec();
    let live: Vec<Turn> = said.iter().filter(|t| !_only_a_nod(&t.user)).cloned().collect();
    if !live.is_empty() && live.len() < said.len() {
        said = live;
    }
    let answered: Vec<Turn> = said.iter().filter(|t| !_about_this_chat_m(&t.user, None)).cloned().collect();
    if !answered.is_empty() && answered.len() < said.len() {
        // A turn that was itself a question about the chat is the bot's paperwork.
        said = answered;
    }
    let last = said.last().expect("IndexError: list index out of range");
    if !crate::memory::thread::thin(&last.user) {
        // The bot was answering a line of its own, so its last reply is the thing to hear again.
        return Some(last.clone());
    }
    // After a "thanks" or an "ok" the reply is a nod: the answer is the one the nod followed.
    if let Some(t) = said.iter().rev().find(|t| t.module.as_deref().map(|m| ANSWER_MODULES.contains(m)).unwrap_or(false)) {
        return Some(t.clone());
    }
    if let Some(t) = said.iter().rev().find(|t| _carries_something(t)) {
        return Some(t.clone());
    }
    None
}

/// Python: bot/memory/store_replies.py:10039 `_just_a_hello(line)`
///
/// True for a line that is only a greeting ("hi", "good morning", "hiya there").
pub fn _just_a_hello(line: &str) -> bool {
    if line.contains('?') {
        return false;
    }
    let found = LETTER_WORDS.findall_str(&line.to_lowercase());
    !found.is_empty() && found.iter().any(|w| GREET.contains(w)) && found.iter().all(|w| GREET.contains(w) || ACKS.contains(w) || HELLO_FILLER.contains(w))
}

/// Python: bot/memory/store_replies.py:10052 `_worth_reporting(line)`
///
/// True for a line that said something a chat can be said to have opened with.
pub fn _worth_reporting(line: &str) -> bool {
    !line.is_empty() && !_just_a_hello(line) && !crate::memory::history::not_a_subject(line)
}

/// Python: bot/memory/store_replies.py:10056 `_first_reply(memory)`
///
/// The first line the person sent, the way a friend would recall how the chat began.
pub fn _first_reply(memory: &mut Memory) -> Option<String> {
    let lines = _turn_lines(memory);
    if lines.is_empty() || lines[0].0.is_empty() {
        return Some(_vary(&["You haven't said anything yet, so this is the start.", "We're just beginning \u{2014} you go first."].map(str::to_string), memory));
    }
    let said: Vec<String> = lines.iter().map(|(u, _)| u.clone()).filter(|l| _worth_reporting(l)).collect();
    if said.is_empty() {
        return Some(_vary(
            &[
                "We've only just started \u{2014} nothing of yours to look back on yet.",
                "Only hellos so far: your first proper line is still to come.",
                "Nothing of yours to report back yet \u{2014} you've only said hello.",
            ]
            .map(str::to_string),
            memory,
        ));
    }
    let first = _trim(&said[0], 90);
    let mut options = vec![
        "You started with: \"".to_string() + &first + "\"",
        "Your first line was: \"".to_string() + &first + "\"",
        "That's how we opened: \"".to_string() + &first + "\"",
    ];
    if said.len() <= 1 {
        options.push("This chat opened with: \"".to_string() + &first + "\" \u{2014} not much else yet.");
    }
    Some(_vary(&options, memory))
}

static SENTENCE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s+", 0));

/// Python: bot/memory/store_replies.py:10075 `_what_i_said_about(memory, thing)`
///
/// The bot's own answer about the thing they name, in a sentence, or None.
pub fn _what_i_said_about(memory: &mut Memory, thing: &str) -> Option<String> {
    use crate::memory::topics::keywords;
    let words: HashSet<String> = keywords(thing).into_iter().filter(|w| !STOPWORDS.contains(w)).collect();
    if words.is_empty() {
        return None;
    }
    let turns = memory.turns.clone();
    for turn in turns.iter().rev() {
        if ["safety", "urgent", "grief", "loss"].contains(&turn.module.as_deref().unwrap_or("")) {
            return None; // an emergency answer is not read out again as a quotation
        }
        let said = text::strip(&turn.bot).to_string();
        let asked = &turn.user;
        if said.is_empty() {
            continue;
        }
        // The answer matches when it *opens* on the thing.
        let parts = SENTENCE_END.split(&said);
        let opening: HashSet<String> = keywords(&parts[0]).into_iter().collect();
        let in_answer = words.iter().all(|w| opening.contains(w));
        let in_question = {
            let q: HashSet<String> = keywords(asked).into_iter().collect();
            words.iter().all(|w| q.contains(w))
        };
        if !_carries_something(turn) || !_adds_something(&said, asked) {
            continue; // a nod, or the line said back, is not an answer about the thing
        }
        if !(in_answer || in_question) {
            continue;
        }
        let mut sentence = text::strip(&parts[0]).to_string();
        // An opening that says little on its own takes the sentence after it along.
        let own: HashSet<String> = keywords(&sentence).into_iter().filter(|w| !NOD_WORDS.contains(w)).collect();
        if own.len() <= 2 && parts.len() > 1 {
            sentence = sentence + " " + text::strip(&parts[1]);
        }
        if sentence.is_empty() || text::split(&sentence).len() > 45 {
            sentence = text::rstrip_chars(&text::split(&sentence).into_iter().take(45).collect::<Vec<_>>().join(" "), ",;:").to_string() + "...";
        }
        let name = text::squash(thing);
        return Some(pick(
            &["On ".to_string() + &name + ": " + &sentence, "I said: ".to_string() + &sentence, "What I said about ".to_string() + &name + ": " + &sentence],
            memory,
            "",
        ));
    }
    if words.len() <= 4 {
        // Nothing in the chat was about it: saying so, and naming it, is the answer.
        let name = text::squash(thing);
        return Some(pick(
            &[
                "I don't think we've been over ".to_string() + &name + " yet - what do you want to know?",
                "We haven't got to ".to_string() + &name + " yet. Ask me and I'll say what I know.",
                text::upper_first(&name) + " hasn't come up yet - ask away.",
            ],
            memory,
            "",
        ));
    }
    None
}

/// Python: bot/memory/store_replies.py:10126 `_adds_something(said, asked)`
///
/// True when a bot line says more than the line it answers, so it can be quoted as an answer.
pub fn _adds_something(said: &str, asked: &str) -> bool {
    use crate::memory::topics::keywords;
    let asked_words: HashSet<String> = keywords(asked).into_iter().collect();
    keywords(said).into_iter().any(|w| !NOD_WORDS.contains(&w) && !asked_words.contains(&w))
}

/// Python: bot/memory/store_replies.py:10137 `_first_bot_reply(memory)`
///
/// The first thing the bot said, for "what was the first thing you said?".
pub fn _first_bot_reply(memory: &mut Memory) -> Option<String> {
    let first = _first_sentence(&crate::memory::history::first_bot_line(memory), 70);
    if first.is_empty() {
        return Some("I haven't said anything yet \u{2014} you go first.".to_string());
    }
    Some(_vary(
        &["I opened with: \"".to_string() + &first + "\"", "My first line was: \"".to_string() + &first + "\"", "I started with: \"".to_string() + &first + "\""],
        memory,
    ))
}

/// Python: bot/memory/store_replies.py:10146 `_asked_and_answered(memory)`
///
/// The last line they said that was about something, and the bot's own reply to it.
pub fn _asked_and_answered(memory: &Memory) -> (String, String) {
    for seen in recent_turns(memory, 8).iter().rev() {
        if !seen.user.is_empty() && !_about_this_chat_m(&seen.user, None) {
            return (seen.user.clone(), text::strip(&seen.bot).to_string());
        }
    }
    (String::new(), String::new())
}

rx!(THEIR_OWN_ASK);
static A_SHORT_ANSWER: LazyLock<usize> = LazyLock::new(|| g("A_SHORT_ANSWER").as_int().unwrap() as usize);

/// Python: bot/memory/store_replies.py:10162 `_said_reply(memory)`
///
/// The person's own last line, for "what did I just say?".
pub fn _said_reply(memory: &mut Memory) -> Option<String> {
    let mut last = last_turn_user(memory);
    if last.is_empty() {
        return Some("That was your first line \u{2014} there's nothing before it.".to_string());
    }
    // A recall said after a recall is about the thing the chat is on.
    let mut real = _last_real(memory);
    if !real.is_empty() && _just_a_hello(&real) {
        // A greeting is not a thing they told us about themselves.
        real = String::new();
    }
    if !real.is_empty() {
        last = real.clone();
    } else if crate::memory::corrected::_asks(&last) {
        // A question they asked is not something they told us either.
        let (mut asked, mut mine) = _asked_and_answered(memory);
        if asked.is_empty() {
            for seen in recent_turns(memory, 2).iter().rev() {
                let line = &seen.user;
                if !line.is_empty() && !SAID_ASK.is_match(&line.to_lowercase()) && !FIRST_ASK.is_match(&line.to_lowercase()) {
                    asked = line.clone();
                    mine = text::strip(&seen.bot).to_string();
                    break;
                }
            }
        }
        let again = _about_this_chat_m(&last, None);
        // A question is worth handing back when it is all the chat holds.
        let nothing_yet = real.is_empty() && (crate::memory::history::not_a_subject(&asked) || THEIR_OWN_ASK.is_match(&asked));
        if !mine.is_empty() && (nothing_yet || (!_just_a_hello(&asked) && (again || text::split(&mine).len() > *A_SHORT_ANSWER))) {
            let asked = text::rstrip_chars(&_trim(&asked, 90), " .!?").to_string();
            let first = text::strip(&SENTENCE_END.split(&mine)[0]).to_string();
            return Some(_vary(
                &[
                    "You asked: \"".to_string() + &asked + "\" \u{2014} I said: " + &first,
                    "That one was your question: \"".to_string() + &asked + "\". My answer: " + &first,
                    "You asked \"".to_string() + &asked + "\", and what I said was: " + &first,
                ],
                memory,
            ));
        }
        return None;
    } else {
        // Nothing they told us yet, so say that rather than quote a greeting back at them.
        return _summary_reply(memory);
    }
    let fixed = mget(memory, "correction").clone();
    let fixed_words = {
        let s = fixed.get("sentence").filter(|v| v.truthy()).or_else(|| fixed.get("value").filter(|v| v.truthy()));
        text::squash(&s.map(py_str).unwrap_or_default())
    };
    if fixed.is_dict() && !fixed_words.is_empty() && fixed.get("said") == Some(&Value::from(fold(&last))) {
        // Their last line was a correction of the one before it: what they said is the line with
        // the correction in it.
        let said = text::rstrip_chars(&fixed_words, " .!?").to_string();
        return Some(_vary(
            &[
                "You said ".to_string() + &said + " \u{2014} you put me right on it.",
                "You said ".to_string() + &said + ", and that is the version I am going with.",
                "\"".to_string() + &said + "\" \u{2014} your first line, put right a moment later.",
            ],
            memory,
        ));
    }
    let trimmed = _trim(&last, 90);
    Some(_vary(
        &["You said: \"".to_string() + &trimmed + "\"", "Your last line was: \"".to_string() + &trimmed + "\"", "That was: \"".to_string() + &trimmed + "\""],
        memory,
    ))
}

rx!(ASKS_IN_LINE, CORRECTS, DENIAL);

/// Python: bot/memory/store_replies.py:10242 `_corrects(line)`
///
/// "sorry, Naples not Rome" replaces an earlier line instead of saying something new.
pub fn _corrects(line: &str) -> bool {
    CORRECTS.is_match(&fold(line))
}

/// Python: bot/memory/store_replies.py:10246 `_last_real(memory)`
///
/// The last thing the user said that was about something, not about this chat.
pub fn _last_real(memory: &Memory) -> String {
    for seen in recent_turns(memory, 8).iter().rev() {
        let line = &seen.user;
        if !line.is_empty() && !_about_this_chat_m(line, None) && !crate::memory::corrected::_asks(line) {
            return line.clone();
        }
    }
    String::new()
}

static KEY_SLOT: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("KEY_SLOT").str_map());

/// Python: bot/memory/store_replies.py:10265 `_the_fact_is_kept(key, memory)`
///
/// True when the store holds the fact a recall key names.
pub fn _the_fact_is_kept(key: &str, memory: &mut Memory) -> bool {
    match KEY_SLOT.get(key).filter(|s| !s.is_empty()) {
        Some(slot) => user(memory).get(slot).map(Value::truthy).unwrap_or(false),
        None => false,
    }
}

/// Python: bot/memory/store_replies.py:10270 `_a_denial(reply)`
///
/// True when the reply is the store saying it has nothing about the thing.
pub fn _a_denial(reply: &str) -> bool {
    !reply.is_empty() && DENIAL.is_match(reply)
}

static ABOUT_THE_REST: LazyLock<Re> = LazyLock::new(|| Re::new(r"\babout\b\s+(.+)$", 0));
static RECALL_TAIL_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:again|before|earlier|please|so far|up to now|until now)\b", 0));

/// Python: bot/memory/store_replies.py:10274 `_names_its_thing(low)`
///
/// True when a "what was i saying about ..." line names the thing it asks about.
pub fn _names_its_thing(low: &str) -> bool {
    let Some(after) = ABOUT_THE_REST.search(low) else { return false };
    let thing = text::strip_chars(&RECALL_TAIL_WORDS.sub("", after.str(1)), " ?!.,").to_string();
    if crate::memory::topics::content(&thing).is_empty() || ["it", "that", "this", "them", "that one"].contains(&thing.as_str()) {
        return false;
    }
    true
}

/// Python: bot/memory/store_replies.py:10290 `_say_back(line, memory=None)`
///
/// Their own line said again as a sentence about them, with no quotation marks round it.
pub fn _say_back(line: &str, memory: &Memory) -> String {
    let restated = crate::memory::thread::restate(line);
    let said = if restated.is_empty() { _recap(line) } else { restated };
    let said = text::rstrip_chars(text::strip(&said), ".").to_string();
    if said.is_empty() {
        return _recall_quote(line, Some(memory), false);
    }
    let said = lower_first(&said);
    _vary(&["You said ".to_string() + &said + ". Want to carry on?", "You said ".to_string() + &said + "."], memory)
}

strset!(REFLEXIVE);

static ABOUT_TAIL: LazyLock<Re> = LazyLock::new(|| Re::new(r"\babout\b(.*)$", 0));
static ABOUT_TAIL_I: LazyLock<Re> = LazyLock::new(|| Re::new(r"\babout\b(.*)$", crate::py::re::I));
static SAID_TAIL: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:say|said|tell you|told you|mention)\b(.*)$", 0));
static TO_SOMEONE: LazyLock<Re> = LazyLock::new(|| Re::new(r"to\s+(?!me\b|you\b)", 0));
static MY_AT_THE_FRONT: LazyLock<Re> = LazyLock::new(|| Re::new(r"^my\b", 0));
static RECALL_LIVING: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.recall.LIVING"));
strset!(JUDGING, A_ROLE);
rx!(PRONOUN_ASK);

/// Python: bot/memory/store_replies.py:10306 `_recall_reply(memory, low, text="", spoken=False)`
///
/// The line they are asking about: "what did i say about the cake?", "and about it?".
pub fn _recall_reply(memory: &mut Memory, low: &str, text_: &str, spoken: bool) -> Option<String> {
    use crate::memory::topics::content;
    // "what did i say was hurting" asks about a complaint the store holds.
    if let Some(hurt) = some_str(crate::memory::whathurts::what_is_hurting_reply(memory, low)) {
        return Some(hurt);
    }
    let about = ABOUT_TAIL.search(low);
    let part = if about.is_some() { None } else { _which_thing_was_it(low, memory) };
    let mut thing = if let Some(a) = &about {
        a.str(1).to_string()
    } else if let Some(p) = &part {
        // "and which knee was it" names the thing the detail is about.
        p.clone()
    } else {
        SAID_TAIL.search(low).map(|m| m.str(1).to_string()).unwrap_or_default()
    };
    // The named-back form is said in their words, not in the normalised line.
    let shown = match ABOUT_TAIL_I.search(text_) {
        Some(mine) => text::strip_chars(mine.str(1), " ?!.,").to_string(),
        None => text::strip_chars(&thing, " ?!.,").to_string(),
    };
    thing = text::strip_chars(&RECALL_TAIL_WORDS.sub("", &thing), " ?!.,").to_string();
    if REFLEXIVE.contains(&thing.to_lowercase()) {
        // "what did i tell you about myself" is the whole picture, not a thing they said.
        return None;
    }
    if content(&thing).is_empty() {
        // "what's the noise again?" names the thing instead of saying "about it".
        if let Some(named) = THING_AGAIN.match_(low) {
            thing = named.str(1).to_string();
        } else {
            let source = if text_.is_empty() { low } else { text_ };
            if let Some(reminded_thing) = some_str(crate::memory::reminded::the_thing(source, memory)) {
                thing = reminded_thing;
            }
        }
    }
    // "what did i say to her?" is what they said to someone else, not to the bot.
    if TO_SOMEONE.match_(&thing).is_some() {
        return None;
    }
    // "for a living" is the job said as a phrase, not a thing a line is about.
    if RECALL_LIVING.is_match(&thing) {
        return None;
    }
    let words = content(&thing);
    if words.is_empty() || ["it", "that", "this", "them", "that one"].contains(&thing.as_str()) {
        let real = _last_real(memory);
        if !real.is_empty() {
            return Some(_recall_quote(&real, Some(memory), false));
        }
        if let Some(record) = crate::memory::topics::current(memory).cloned() {
            let said = record.get("text").filter(|t| t.truthy()).map(py_str);
            if let Some(said) = said {
                if !_about_this_chat_m(&said, None) {
                    return Some(_recall_quote(&said, Some(memory), crate::memory::corrected::_asks(&said)));
                }
            }
        }
        return Some(pick(&["You haven't told me anything about that yet.", "Nothing comes to mind \u{2014} you haven't said."], memory, ""));
    }
    // A line about the telling is not the thing told; it is kept apart rather than dropped.
    let (mut hits, mut asked, mut fixed, mut claims): (Vec<String>, Vec<String>, Vec<String>, Vec<String>) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let lines: Vec<String> = memory.turns.iter().map(|t| t.user.clone()).collect();
    for line in &lines {
        if line.is_empty() || _about_this_chat_m(line, None) {
            continue;
        }
        let seen = _seen(line);
        let question = crate::memory::corrected::_asks(line);
        // A word of the ask finds the line by the kind it names as well as by the word.
        let mut all = true;
        for word in &words {
            if _alike(word, &seen) {
                continue;
            }
            if !question && _about_kind(word, line, Some(&mut *memory), false) {
                continue;
            }
            all = false;
            break;
        }
        if all {
            // "sorry, Naples not Rome" names Rome in order to replace it.
            if crate::memory::corrected::_asks(line) {
                asked.push(line.clone());
            } else if _corrects(line) {
                fixed.push(line.clone());
            } else if A_CLAIM_OF_TELLING.match_(&fold(line)).is_some() {
                claims.push(line.clone());
            } else {
                hits.push(line.clone());
            }
        }
    }
    if hits.is_empty() && !claims.is_empty() {
        hits = claims;
    }
    if !hits.is_empty() {
        // A line that asks about the thing does not tell us the thing.
        let stated: Vec<String> = hits.iter().filter(|l| !ASKS_IN_LINE.is_match(l)).cloned().collect();
        if !stated.is_empty() {
            hits = stated;
        }
        let last = hits.last().unwrap().clone();
        if !spoken {
            let put_right = _put_right_after_it(&last, memory);
            if !put_right.is_empty() {
                return Some(put_right);
            }
            let moved = _a_figure_put_right(&last, memory);
            if !moved.is_empty() {
                return Some(moved);
            }
            let narrowed = _a_narrowing_put_after(&last, memory);
            if !narrowed.is_empty() {
                return Some(narrowed);
            }
        }
        return Some(if spoken { _say_back(&last, memory) } else { _recall_quote(&last, Some(memory), false) });
    }
    // The only line about it was a question, which they still said.
    if let Some(a) = asked.last() {
        return Some(_recall_quote(a, Some(memory), true));
    }
    if let Some(f) = fixed.last() {
        return Some(_recall_quote(f, Some(memory), false));
    }
    // A recall ask carries its own judgement: the judging words are dropped when nothing is found.
    let naming: Vec<String> = words.iter().filter(|w| !JUDGING.contains(*w) && !A_ROLE.contains(*w)).cloned().collect();
    if !naming.is_empty() && naming != words {
        let (mut naming_hits, mut naming_asked): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
        for turn in recent_turns(memory, 8) {
            let line = &turn.user;
            if line.is_empty() || _about_this_chat_m(line, None) {
                continue;
            }
            let seen = _seen(line);
            if naming.iter().all(|w| _alike(w, &seen)) {
                if crate::memory::corrected::_asks(line) {
                    naming_asked.push(line.clone());
                } else {
                    naming_hits.push(line.clone());
                }
            }
        }
        if let Some(h) = naming_hits.last() {
            return Some(_recall_quote(h, Some(memory), false));
        }
        if let Some(a) = naming_asked.last() {
            return Some(_recall_quote(a, Some(memory), true));
        }
    }
    // The only word left is the ask's own judgement and the thing is a pronoun.
    if naming.is_empty() && PRONOUN_ASK.is_match(&thing) {
        let real = _last_real(memory);
        // A greeting is not a thing that could be wrong.
        if !real.is_empty() && content(&real).len() > 1 {
            return Some(_recall_quote(&real, Some(memory), false));
        }
    }
    if about.is_none() {
        return None;
    }
    // The thing is named back in the second person.
    let said = text::strip(&MY_AT_THE_FRONT.sub("your", &shown)).to_string();
    Some(pick(
        &["You haven't said anything to me about ".to_string() + &said + " yet.", "You haven't mentioned ".to_string() + &said + " \u{2014} say it and I'll keep it."],
        memory,
        "",
    ))
}

/// `TAGS`: (the ask's own word for a thing, what a line that stands for it says).
static TAGS: LazyLock<Vec<(String, &'static Re)>> =
    LazyLock::new(|| g("TAGS").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].as_re().unwrap())).collect());

/// Python: bot/memory/store_replies.py:10484 `_tagged(line)`
///
/// The words a line stands for without saying them: the ask's own word for that thing.
pub fn _tagged(line: &str) -> HashSet<String> {
    TAGS.iter().filter(|(_, p)| p.is_match(line)).map(|(c, _)| c.clone()).collect()
}

rx!(SAYS_A_STAY, SAYS_A_JOB, SAYS_FOOD);
strset!(NOT_A_PLACE_IN);
static FACTS_NOT_PLACES: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.NOT_PLACES"));
static FACTS_PLACE_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.PLACE_VERBS"));
static PLACE_LOWER: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.facts.PLACE_LOWER"));
/// `KINDS`: (kind, the words that name it), in order.
static KINDS: LazyLock<Vec<(String, HashSet<String>)>> =
    LazyLock::new(|| g("KINDS").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].str_set())).collect());
static SAYS_THE_KIND: LazyLock<HashMap<String, &'static Re>> =
    LazyLock::new(|| g("SAYS_THE_KIND").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_re().unwrap())).collect());

/// Python: bot/memory/store_replies.py:10575 `_a_stay_in_a_place(text)`
///
/// True when the line says a stretch of time somewhere they are going.
pub fn _a_stay_in_a_place(text_: &str) -> bool {
    SAYS_A_STAY.finditer(text_).iter().any(|found| {
        let word = found.name_str("place").to_lowercase();
        !FACTS_NOT_PLACES.contains(&word) && !NOT_A_PLACE_IN.contains(&word)
    })
}

/// Python: bot/memory/store_replies.py:10598 `_kind(thing)`
///
/// The kind of thing an ask names, or "" when the word is a thing of its own. (The earlier
/// `_kind(item)` at line 770 is shadowed by this one in the module.)
pub fn _kind(thing: &str) -> Value {
    Value::from(_kind_s(thing))
}

/// `_kind` as the str it is.
fn _kind_s(thing: &str) -> String {
    KINDS.iter().find(|(_, words)| words.contains(thing)).map(|(k, _)| k.clone()).unwrap_or_default()
}

/// Python: bot/memory/store_replies.py:10605 `_says_where_they_live(text)`
///
/// True when the line says where they live, in whatever words: "we moved to york last year".
pub fn _says_where_they_live(text_: &str) -> bool {
    if text::strip(text_).is_empty() {
        return false;
    }
    // A line this cannot read names no home.
    match py_try(|| crate::memory::facts::extract(text_, false).val()) {
        Ok(found) => found.get("home").map(Value::truthy).unwrap_or(false),
        Err(_) => false,
    }
}

/// Python: bot/memory/store_replies.py:10620 `_a_place_they_go(text)`
///
/// The place a line says they are going to, or "": "im going to lisbon in may".
pub fn _a_place_they_go(text_: &str) -> String {
    let Some(found) = PLACE_LOWER.search(text_) else { return String::new() };
    let word = found.str(1).to_lowercase();
    if FACTS_NOT_PLACES.contains(&word) || FACTS_PLACE_VERBS.contains(&word) {
        return String::new();
    }
    word
}

/// Python: bot/memory/store_replies.py:10634 `_about_kind(thing, text, memory=None, phrase=False)`
///
/// True when `text` is about the kind of thing the ask names ("trip" -> "Lisbon in may").
pub fn _about_kind(thing: &str, text_: &str, memory: Option<&mut Memory>, phrase: bool) -> bool {
    let kind = _kind_s(thing);
    if kind.is_empty() || text::strip(text_).is_empty() {
        return false;
    }
    let said = SAYS_THE_KIND.get(&kind);
    if kind == "job" {
        return said.map(|p| p.is_match(text_)).expect("'NoneType' object has no attribute 'search'");
    }
    // Where they live is decided by the store's own reading of a home.
    if kind == "home" {
        return _says_where_they_live(text_);
    }
    // A trip is the one kind allowed to answer a place.
    if kind == "trip" && (SAYS_A_TRIP.is_match(text_) || _a_stay_in_a_place(text_) || !_a_place_they_go(text_).is_empty()) {
        return true;
    }
    if let Some(p) = said {
        if p.is_match(text_) {
            return true;
        }
    }
    if !phrase {
        return false;
    }
    let known: Vec<String> = match memory {
        Some(m) => user(m).get("places").map(py_list).unwrap_or_default().iter().filter_map(|p| p.as_str().map(str::to_string)).filter(|p| !p.is_empty()).collect(),
        None => Vec::new(),
    };
    let low = text_.to_lowercase();
    known.iter().any(|p| low.contains(&p.to_lowercase()))
}

/// Python: bot/memory/store_replies.py:10666 `_seen(line)`
///
/// The content words of a line, read the way the ask about it is read.
pub fn _seen(line: &str) -> HashSet<String> {
    use crate::memory::topics::content;
    let mut words: HashSet<String> = content(line).into_iter().collect();
    let plain = crate::understanding::normalize::normalise(line).norm_text;
    if !plain.is_empty() && plain != line {
        words.extend(content(&plain));
    }
    words.extend(_tagged(line));
    words
}

/// Python: bot/memory/store_replies.py:10681 `_alike(word, seen)`
///
/// One word family is enough: "my allergy" finds "i'm allergic to nuts".
pub fn _alike(word: &str, seen: &HashSet<String>) -> bool {
    use crate::memory::topics::forms;
    let want = concept(word);
    let mine = forms(word);
    for other in seen {
        if word == other {
            return true;
        }
        if forms(other).iter().any(|f| mine.contains(f)) {
            return true;
        }
        if want.is_some() && want == concept(other) {
            return true;
        }
        let (lw, lo) = (text::len(word), text::len(other));
        if lw >= 4 && lo >= 4 && (word.starts_with(other.as_str()) || other.starts_with(word)) {
            return true;
        }
        if lw >= 5 && lo >= 5 && text::slice(word, 0, Some(5)) == text::slice(other, 0, Some(5)) {
            return true;
        }
    }
    false
}

strs!(QUOTE_LEADS, ASKED_LEADS);
static WHAT_DID_I_I: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bwhat (?:did|have) i\b", crate::py::re::I));

/// Python: bot/memory/store_replies.py:10704 `_recall_quote(line, memory=None, asked=False)`
///
/// "You said: \"...\"": their own words, trimmed, so the answer is the thing itself.
pub fn _recall_quote(line: &str, memory: Option<&Memory>, asked: bool) -> String {
    let said = "\"".to_string() + &_trim(line, 120) + "\"";
    if asked {
        let n = memory.map(|m| recent_turns(m, 8).len()).unwrap_or(0);
        return ASKED_LEADS[n % ASKED_LEADS.len()].clone() + &said;
    }
    let Some(memory) = memory else { return QUOTE_LEADS[0].clone() + &said };
    // The same question twice should not read as the same sentence twice.
    let asked = recent_turns(memory, 8).iter().filter(|t| WHAT_DID_I_I.is_match(&t.user)).count();
    QUOTE_LEADS[asked % QUOTE_LEADS.len()].clone() + &said
}

rx!(A_BARE_PUT_RIGHT, A_FIGURE);

/// Python: bot/memory/store_replies.py:10740 `_a_figure_put_right(line, memory)`
///
/// The quote and the figure the person put right after it, or "".
pub fn _a_figure_put_right(line: &str, memory: &Memory) -> String {
    let said = text::squash(line);
    let figures = A_FIGURE.findall_str(&said);
    if figures.len() != 1 {
        return String::new();
    }
    let lines: Vec<String> = memory.turns.iter().map(|t| text::squash(&t.user)).collect();
    let Some(at) = lines.iter().rposition(|l| *l == said) else { return String::new() };
    // Only the line right after the quote.
    for after in lines.iter().skip(at + 1).take(1) {
        let Some(found) = A_BARE_PUT_RIGHT.match_(text::strip(after)) else { continue };
        let value = text::rstrip_chars(text::strip(found.name_str("value")), ",").to_string();
        if value.is_empty() || value == figures[0] {
            continue;
        }
        return text::rstrip(&_recall_quote(line, Some(memory), false)).to_string() + " \u{2014} then you put it right: " + &value + ".";
    }
    String::new()
}

/// Python: bot/memory/store_replies.py:10781 `_a_narrowing_put_after(line, memory)`
///
/// The quoted line and the narrowing the person put after it, or "".
pub fn _a_narrowing_put_after(line: &str, memory: &Memory) -> String {
    let pair = crate::memory::repeat::a_narrowing_and_its_line(line, memory).val();
    if !pair.truthy() {
        return String::new();
    }
    let parts = pair.items();
    let (before, after) = (py_str(&parts[0]), py_str(&parts[1]));
    text::rstrip(&_recall_quote(&before, Some(memory), false)).to_string() + " \u{2014} then \"" + &_trim(&after, 80) + "\"."
}

/// Python: bot/memory/store_replies.py:10797 `_put_right_after_it(line, memory)`
///
/// The quoted line and the correction that followed it, or "" when there is none.
pub fn _put_right_after_it(line: &str, memory: &Memory) -> String {
    let fixed = mget(memory, "correction");
    if !fixed.is_dict() {
        return String::new();
    }
    let said_back = fold(&sget(fixed, "said"));
    let sentence = text::rstrip_chars(&text::squash(&sget(fixed, "sentence")), " .!?").to_string();
    if said_back.is_empty() || sentence.is_empty() {
        return String::new();
    }
    let lines: Vec<String> = memory.turns.iter().map(|t| fold(&t.user)).collect();
    let quoted = fold(line);
    let (Some(q), Some(s)) = (lines.iter().position(|l| *l == quoted), lines.iter().position(|l| *l == said_back)) else { return String::new() };
    if q >= s {
        return String::new(); // the line quoted is the correction itself, or came after it
    }
    let mine = _content_words(line);
    if !_content_words(&sentence).iter().any(|w| mine.contains(w)) {
        return String::new(); // the correction is about something else in the chat
    }
    let tail = lower_first(&sentence);
    text::rstrip(&_recall_quote(line, Some(memory), false)).to_string() + " \u{2014} then you put it right: " + &tail + "."
}

/// Python: bot/memory/store_replies.py:10824 `_age_reply(memory)`
pub fn _age_reply(memory: &mut Memory) -> String {
    let age = user(memory).get("age").cloned().unwrap_or(Value::None);
    // A number outside the range a person can be is no answer at all.
    let in_range = |v: &Value| -> bool {
        match v.as_f64() {
            Some(a) => (1.0..=120.0).contains(&a),
            None => panic!("TypeError: '<=' not supported between instances of 'int' and '{}'", if v.is_str() { "str" } else { "object" }),
        }
    };
    if !age.truthy() || !in_range(&age) {
        return pick(&["You haven't told me your age yet.", "I don't know it \u{2014} you haven't said."], memory, "");
    }
    let said = py_str(&age);
    pick(&["You told me you're ".to_string() + &said + ".", said.clone() + " \u{2014} you said."], memory, "")
}

/// Python: bot/memory/store_replies.py:10833 `_content_words(line)`
pub fn _content_words(line: &str) -> HashSet<String> {
    LETTER_WORDS.findall_str(&line.to_lowercase()).into_iter().filter(|w| text::len(w) > 3 && !STOPWORDS.contains(w)).collect()
}

/// Python: bot/memory/store_replies.py:10837 `_the_line_before_was_a_plan(memory)`
///
/// True when the last thing they said was a plan of their own.
pub fn _the_line_before_was_a_plan(memory: &mut Memory) -> bool {
    let Some(Value::Str(plan)) = user(memory).get("plan").cloned() else { return false };
    if text::strip(&plan).is_empty() {
        return false;
    }
    let told = crate::memory::plan_line(memory, &plan);
    if told.is_empty() {
        return false;
    }
    fold(&told) == fold(&last_turn_user(memory))
}

/// Python: bot/memory/store_replies.py:10854 `_plan_was_corrected(memory)`
///
/// True when the last thing they put right was a value inside the plan itself.
pub fn _plan_was_corrected(memory: &mut Memory) -> bool {
    let changed = mget(memory, "correction").clone();
    if !changed.is_dict() {
        return false;
    }
    if changed.get("key") == Some(&Value::from("plan")) || changed.get("key") == Some(&Value::from("plans")) {
        return true;
    }
    let value = fold(&sget(&changed, "value"));
    let plan = fold(&sget(&user(memory), "plan"));
    !value.is_empty() && !plan.is_empty() && plan.contains(&value)
}

/// Python: bot/memory/store_replies.py:10871 `_about_the_same(line, line_words)`
///
/// A line that carries on about the subject (a detail) and not a statement of its own.
pub fn _about_the_same(line: &str, line_words: &HashSet<String>) -> bool {
    DETAIL_START.is_match(line) || _content_words(line).iter().any(|w| line_words.contains(w))
}

/// Python: bot/memory/store_replies.py:10875 `_activity_line(memory)`
///
/// The last line in which they said what they were making, cooking or planning (a str; the
/// contract types it `Value`).
pub fn _activity_line(memory: &Memory) -> Value {
    Value::from(_activity_line_str(memory))
}

/// `_activity_line` as the str it is.
fn _activity_line_str(memory: &Memory) -> String {
    let lines: Vec<String> = recent_turns(memory, 4).iter().map(|t| text::squash(&t.user)).collect();
    let changed = mget(memory, "correction");
    let fixed = if changed.is_dict() { text::squash(&sget(changed, "said")) } else { String::new() };
    for wanted in [&*ACTIVITY_WORDS, &*ACTIVITY_LOOSE] {
        for index in (0..lines.len()).rev() {
            let line = &lines[index];
            if line.is_empty() || ASK_LINE.is_match(line) || ACTIVITY_ASK.is_match(line) {
                continue;
            }
            if !wanted.is_match(line) {
                continue;
            }
            let words = _content_words(line);
            let newer: Vec<&String> = lines[index + 1..].iter().filter(|l| !l.is_empty()).collect();
            if newer.iter().any(|later| !fixed.is_empty() && **later == fixed) {
                // A correction is the newest word on the thing they were on.
                return String::new();
            }
            if newer.iter().all(|later| ASK_LINE.is_match(later) || ACTIVITY_ASK.is_match(later) || _about_the_same(later, &words)) {
                return line.clone();
            }
            return String::new();
        }
    }
    String::new()
}

/// Python: bot/memory/store_replies.py:10906 `_the_making_line_they_said(memory, back=8)`
///
/// The last line of theirs that says what they are making, without the trail check.
pub fn _the_making_line_they_said(memory: &Memory, back: usize) -> String {
    for turn in recent_turns(memory, back).iter().rev() {
        let line = text::squash(&turn.user);
        if line.is_empty() || ASK_LINE.is_match(&line) || ACTIVITY_ASK.is_match(&line) {
            continue;
        }
        if ACTIVITY_WORDS.is_match(&line) || ACTIVITY_LOOSE.is_match(&line) {
            return line;
        }
    }
    String::new()
}

static RECAP_PLANNED: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:im|i'm|i am|ive|i've|were|we're|we are)\s+(\S.*)$", crate::py::re::I));
static RECAP_WANTED: LazyLock<Re> = LazyLock::new(|| Re::new(r"^i\s+(?:want|need|plan)\s+to\s+(\S.*)$", crate::py::re::I));

/// Python: bot/memory/store_replies.py:10924 `_recap(line)`
///
/// Their own line said back as a sentence about them: "im making a cake" -> "You're making a
/// cake for your son."
pub fn _recap(line: &str) -> String {
    let flat = text::strip_chars(&text::squash(line), " .").to_string();
    let body = if let Some(planned) = RECAP_PLANNED.match_(&flat) {
        "you're ".to_string() + planned.str(1)
    } else {
        match RECAP_WANTED.match_(&flat) {
            Some(wanted) => "you want to ".to_string() + wanted.str(1),
            None => return String::new(),
        }
    };
    let body = _tidy_words(&crate::memory::you(&body));
    text::upper_first(&body)
}

static LOWER_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]+", 0));

/// Python: bot/memory/store_replies.py:10941 `_tidy_words(text)`
///
/// A country or a month keeps its capital: "going to portugal in june" -> "Portugal in June".
pub fn _tidy_words(text_: &str) -> String {
    const ORDINARY: [&str; 11] = ["a", "an", "the", "my", "your", "our", "his", "her", "their", "this", "that"];
    // A name that is also an everyday word needs the sentence to say it is a place.
    const BOTH: [&str; 6] = ["turkey", "china", "bath", "hull", "york", "goa"];
    const TRAVEL: [&str; 8] = ["to", "in", "from", "at", "near", "visit", "visiting", "into"];
    LOWER_WORDS.sub_fn(text_, |m| {
        let word = m.as_str();
        if !FACTS_COUNTRIES.contains(word) && !FACTS_MONTHS.iter().any(|w| w == word) && !PLACES.contains(word) {
            return word.to_string();
        }
        let before = text::split(&text_[..m.start()]);
        let last = before.last().map(|w| w.to_lowercase());
        if let Some(l) = &last {
            if ORDINARY.contains(&l.as_str()) {
                return word.to_string();
            }
        }
        if BOTH.contains(&word) && !last.as_deref().map(|l| TRAVEL.contains(&l)).unwrap_or(false) {
            return word.to_string();
        }
        text::upper_first(word)
    })
}

static THREAD_QUALITY: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.thread.QUALITY"));
static THREAD_NON_NOUN: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.thread.NON_NOUN"));

/// Python: bot/memory/store_replies.py:10967 `_a_thing(name)`
///
/// Whether the thing we were on is a phrase a person would say, and not a verb.
pub fn _a_thing(name: &str) -> bool {
    let words = text::split(name);
    let Some(last) = words.last() else { return false };
    let head = text::strip_chars(&last.to_lowercase(), ".,;:!?").to_string();
    if head.is_empty() {
        return false;
    }
    if text::ends_with_any(&head, &["ing", "ed", "ish"]) {
        return false;
    }
    // "and again" left a subject of "again": a filler word is not a thing to be on either.
    !THREAD_QUALITY.contains(&head) && !THREAD_NON_NOUN.contains(&head)
}

strset!(THING_POINTER);

/// Python: bot/memory/store_replies.py:10990 `_says_the_thing(said, thing)`
///
/// Whether the line about to be quoted already names the thing the recap would name.
pub fn _says_the_thing(said: &str, thing: &str) -> bool {
    let named: HashSet<String> = LETTER_WORDS.findall_str(&thing.to_lowercase()).into_iter().filter(|w| !THING_POINTER.contains(w)).collect();
    let words: HashSet<String> = LETTER_WORDS.findall_str(&said.to_lowercase()).into_iter().collect();
    !named.is_empty() && named.iter().any(|w| words.contains(w))
}

/// Python: bot/memory/store_replies.py:11001 `_a_description(thing)`
///
/// Whether the thing is a description the store gave an article to, not a name.
pub fn _a_description(thing: &str) -> bool {
    let words = text::split(text::strip(thing));
    if words.len() < 2 {
        return false;
    }
    ["the", "a", "an"].contains(&words[0].to_lowercase().as_str())
}

/// Python: bot/memory/store_replies.py:11014 `_the_fixed_fact(memory)`
///
/// The fact a correction just fixed, as the store keeps it, or None.
pub fn _the_fixed_fact(memory: &mut Memory) -> Option<String> {
    let fixed = mget(memory, "correction").clone();
    if !fixed.is_dict() {
        return None;
    }
    let key = text::strip(&sget(&fixed, "key")).to_string();
    let value = text::strip(&sget(&fixed, "value")).to_string();
    let said = text::strip(&sget(&fixed, "said")).to_string();
    if key.is_empty() || value.is_empty() {
        return None;
    }
    if !said.is_empty() && text::strip(&last_turn_user(memory)).to_lowercase() != said.to_lowercase() {
        return None; // an older correction: the line after it is the one we were on
    }
    let ask = text::squash(&_slot_words(&key, memory));
    let sentence = _fact_reply(memory, &ask, Some(&key)).unwrap_or_default();
    let sentence = text::strip(&sentence).to_string();
    if !(sentence.starts_with("Your ") || sentence.starts_with("You ")) {
        return None;
    }
    Some(lower_first(text::rstrip_chars(&sentence, ".")))
}

static ABOUT_THE_THING: LazyLock<Re> = LazyLock::new(|| Re::new(r"\babout\s+(.*)$", 0));

/// Python: bot/memory/store_replies.py:11034 `_the_line_named(memory, low)`
///
/// The earlier line of theirs that the thing asked about is in, or "" when none is.
pub fn _the_line_named(memory: &Memory, low: &str) -> String {
    use crate::memory::topics::content;
    let Some(about) = ABOUT_THE_THING.search(low) else { return String::new() };
    let words = content(about.str(1));
    if words.is_empty() {
        return String::new();
    }
    for seen in recent_turns(memory, 8).iter().rev() {
        let line = &seen.user;
        if line.is_empty() || _about_this_chat_m(line, None) {
            continue;
        }
        let have: HashSet<String> = content(line).into_iter().collect();
        if !have.is_empty() && words.iter().all(|w| _alike(w, &have)) {
            return line.clone();
        }
    }
    String::new()
}

/// The `skip` reader the thread functions take (Python passes `_about_this_chat` itself): a
/// function reference to `_about_this_chat`, called with the line alone, which is
/// `store_replies::_about_this_chat_m(line, None)` in Rust.
pub fn skip_about_this_chat() -> Value {
    fn_ref("bot.memory.store_replies:_about_this_chat")
}

/// Python: bot/memory/store_replies.py:11051 `_topic_reply(memory, low, again=False)`
///
/// Where we were, said back in a sentence: the thing, and what was asked about it.
pub fn _topic_reply(memory: &mut Memory, low: &str, again: bool) -> Option<String> {
    use crate::memory::thread;
    let skip = skip_about_this_chat();
    let mut record_text: Option<String> = crate::memory::topics::current(memory).cloned().and_then(|r| if r.truthy() { Some(s_or(r.get("text"))) } else { None });
    let record_truthy = crate::memory::topics::current(memory).map(Value::truthy).unwrap_or(false);
    let real = _last_real(memory);
    if !real.is_empty() {
        record_text = Some(real.clone());
    }
    let has_record = !real.is_empty() || record_truthy;
    // The thing the line names decides which line is said back.
    let on_that_line = _the_line_named(memory, low);
    let mut said_again: Option<String> = _the_fixed_fact(memory).filter(|s| !s.is_empty());
    if said_again.is_none() && !on_that_line.is_empty() {
        said_again = nz(thread::restate(&on_that_line));
    }
    if said_again.is_none() {
        said_again = ret_opt(thread::last_said(memory, &skip)).filter(|s| !s.is_empty());
    }
    if let Some(s) = &said_again {
        if text::split(s).len() < 2 {
            said_again = None; // "and again" is not a thing the chat was on
        }
    }
    if low.contains("yesterday") {
        let said = match &said_again {
            Some(s) => " In this one, you last said ".to_string() + s + ".",
            None => String::new(),
        };
        return Some("I don't keep yesterday's chat, sorry \u{2014} each conversation starts fresh for me.".to_string() + &said);
    }
    if _names_its_thing(low) {
        // The thing is named, so the line that names the thing is the answer.
        if let Some(by_name) = _recall_reply(memory, low, "", true).filter(|b| !b.is_empty()) {
            if !_a_denial(&by_name) {
                return Some(by_name);
            }
        }
        // A miss is never a denial: it falls through to the answers this line got before.
    }
    let named = NAMED_ACTIVITY.is_match(low);
    let mut carried = if ACTIVITY_ASK.is_match(low) { _activity_line_str(memory) } else { String::new() };
    if carried.is_empty() && named && again {
        carried = _the_making_line_they_said(memory, 8);
    }
    if carried.is_empty() && named {
        // "what was i cooking?" after a chat about a dishwasher asks for the cooking line.
        return None;
    }
    if !carried.is_empty() {
        let recap = _recap(&carried);
        let said_plan = if recap.is_empty() { "You said: \"".to_string() + &_trim(&carried, 90) + "\"" } else { recap };
        let said_plan = text::rstrip_chars(&said_plan, ".").to_string() + ".";
        return Some(_vary(
            &[said_plan.clone() + " Want to carry on with it?", said_plan.clone() + " Where did you get to with it?", said_plan.clone()],
            memory,
        ));
    }
    let mut thing = ret_opt(thread::about(memory, &skip));
    if !_a_thing(thing.as_deref().unwrap_or("")) {
        thing = None;
    }
    let ask = if !on_that_line.is_empty() {
        String::new()
    } else {
        let not_before = thread::topic_index(memory, &skip).val();
        ret_str(thread::last_ask(memory, &skip, &not_before))
    };
    let said_text = said_again.clone().unwrap_or_default();
    if let Some(thing) = thing.as_deref().filter(|t| !t.is_empty()) {
        if !ask.is_empty() && !(_a_description(thing) && (_says_the_thing(&said_text, thing) || _says_the_thing(&ask, thing))) {
            return Some(_vary(
                &[
                    "We were on ".to_string() + thing + " \u{2014} you asked " + &ask + ". Want to carry on?",
                    "Still on ".to_string() + thing + ": you asked " + &ask + ".",
                    "The last thing we got into was ".to_string() + thing + ", and you asked " + &ask + ".",
                ],
                memory,
            ));
        }
    }
    if !ask.is_empty() {
        if let Some(s) = &said_again {
            return Some(_vary(
                &[
                    "You said ".to_string() + s + ". You asked " + &ask + ". Want to carry on?",
                    text::upper_first(s) + " - and you asked " + &ask + ". Want to carry on?",
                ],
                memory,
            ));
        }
        return Some(_vary(&["You asked ".to_string() + &ask + ". Want to carry on?", "You were asking ".to_string() + &ask + "."], memory));
    }
    if let Some(s) = &said_again {
        return Some(_vary(
            &["You said ".to_string() + s + ". Want to carry on?", "You last said ".to_string() + s + ".", "You'd told me ".to_string() + s + "."],
            memory,
        ));
    }
    if let Some(thing) = thing.as_deref().filter(|t| !t.is_empty()) {
        return Some(_vary(&["We were on ".to_string() + thing + ".", "Still ".to_string() + thing + "."], memory));
    }
    if has_record {
        let record_text = record_text.unwrap_or_default();
        if !thread::thin(&record_text) && thread::carries_subject(&record_text) {
            let plain = thread::restate(&record_text);
            if !plain.is_empty() && _a_thing(&plain) {
                return Some("We were on ".to_string() + &plain + ".");
            }
        }
    }
    if let Some(asked_of_me) = _the_last_ask_about_the_bot(memory) {
        return Some(_vary(
            &[
                "You asked ".to_string() + &asked_of_me + " - that is all we have been into so far. What else is on your mind?",
                "So far you have asked ".to_string() + &asked_of_me + ". Tell me something of your own and I will keep it.",
                "You asked ".to_string() + &asked_of_me + ". That is about all so far - what else are you wondering about?",
            ],
            memory,
        ));
    }
    // Nothing but "hi", "ok", "lol" so far: there is no subject yet.
    Some(_vary(&NOTHING_STARTED, memory))
}

/// Python: bot/memory/store_replies.py:11156 `_the_last_ask_about_the_bot(memory)`
///
/// The last thing they asked about the bot, as a sentence, when that is all they have said.
pub fn _the_last_ask_about_the_bot(memory: &Memory) -> Option<String> {
    let lines = crate::memory::thread::user_lines(memory).val();
    for pair in lines.items().iter().rev() {
        let parts = pair.items();
        let (said, reading) = (parts.first().cloned().unwrap_or(Value::None), parts.get(1).cloned().unwrap_or(Value::None));
        if !(reading.is_dict() && reading.get("about_bot").map(Value::truthy).unwrap_or(false)) {
            return None;
        }
        let asked = crate::memory::thread::restate(&py_str(&said));
        if !asked.is_empty() {
            return Some(asked);
        }
    }
    None
}

/// Python: bot/memory/store_replies.py:11173 `_asked_reply(memory)`
///
/// 'What did I just ask you?' — their last question, said again in their words.
pub fn _asked_reply(memory: &mut Memory) -> String {
    let skip = skip_about_this_chat();
    let ask = ret_str(crate::memory::thread::last_ask(memory, &skip, &Value::None));
    if !ask.is_empty() {
        return "You asked ".to_string() + &ask + ".";
    }
    let said = ret_str(crate::memory::thread::last_said(memory, &skip));
    if !said.is_empty() {
        return "You didn't ask me anything \u{2014} you said ".to_string() + &said + ".";
    }
    // A chat whose only line so far is a question about the bot has no subject of theirs.
    if let Some(asked_of_me) = _the_last_ask_about_the_bot(memory) {
        return "You asked ".to_string() + &asked_of_me + ".";
    }
    "You haven't asked me anything yet.".to_string()
}

strset!(SMALL_TALK_ONLY, SUMMARY_FILLER);

/// `last_user(memory, n)` for n > 1: the last n user lines, oldest first.
fn last_users(memory: &Memory, n: usize) -> Vec<String> {
    recent_turns(memory, n).iter().map(|t| t.user.clone()).collect()
}

/// Python: bot/memory/store_replies.py:11198 `_last_told(memory)`
///
/// What they last told you, in their own words, or "" when the line was a question or small talk.
pub fn _last_told(memory: &Memory) -> String {
    let record = crate::memory::topics::current(memory).cloned().unwrap_or(Value::None);
    let mut candidates = vec![sget(&record, "text")];
    candidates.extend(last_users(memory, 3).into_iter().rev());
    for text_ in candidates {
        if crate::memory::recall::ask(&text_, false).as_deref() == Some("me") {
            // A question about what the bot holds is not something they told it.
            continue;
        }
        let phrase = _told_phrase(&text_);
        if !phrase.is_empty() {
            return phrase;
        }
    }
    String::new()
}

rx!(NAME_SAID);

/// Python: bot/memory/store_replies.py:11226 `_as_repaired(word)`
///
/// The word as the reader repairs it: the repair is right for a typo and wrong for a name.
pub fn _as_repaired(word: &str) -> String {
    crate::understanding::normalize::fix_typo(&word.to_lowercase()).filter(|w| !w.is_empty()).unwrap_or_else(|| word.to_string())
}

/// Python: bot/memory/store_replies.py:11230 `_with_the_name(said, raw)`
///
/// The same line with a name put back as they spelt it.
pub fn _with_the_name(said: &str, raw: &str) -> String {
    let mut names: Vec<String> = Vec::new();
    for m in NAME_SAID.finditer(raw) {
        let name = text::strip_chars(m.str(1), "'\u{2019}-").to_string();
        if NOT_A_NAME.contains(&name.to_lowercase()) || text::len(&name) < 2 || names.contains(&name) {
            continue;
        }
        names.push(name);
    }
    if names.is_empty() || said.is_empty() {
        return said.to_string();
    }
    let mut words = text::split(said);
    for name in &names {
        let spelled = text::upper_first(name);
        let wanted = [name.to_lowercase(), _as_repaired(name)];
        for word in words.iter_mut() {
            let core = text::strip_chars(word, ".,!?;:").to_string();
            if wanted.contains(&core.to_lowercase()) {
                *word = word.replace(&core, &spelled);
                break;
            }
        }
    }
    words.join(" ")
}

strset!(SAY_BACK_VAGUE_TAIL, SAY_BACK_VAGUE);
rx!(SAY_BACK_CORRECTION, DENIAL_LINE, A_NEW_PERSON);
static A_NEGATION_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:isnt|isn't|wasnt|wasn't|not)\b", 0));

/// Python: bot/memory/store_replies.py:11276 `_reads_as_a_correction(line)`
///
/// True when the line takes something back rather than stating something new.
pub fn _reads_as_a_correction(line: &str) -> bool {
    let text_ = fold(line);
    if text_.is_empty() {
        return false;
    }
    let a_correction = || crate::memory::facts::extract(line, false).val().get("correction").map(Value::truthy).unwrap_or(false);
    if DENIAL_LINE.is_match(&text_) && A_NEW_PERSON.is_match(&text_) && !A_NEGATION_WORD.is_match(&text_) {
        return a_correction();
    }
    DENIAL_LINE.is_match(&text_) || a_correction()
}

/// Python: bot/memory/store_replies.py:11286 `_lines_that_moved(memory)`
///
/// The lines whose fact a later line put again, with a different value.
pub fn _lines_that_moved(memory: &Memory) -> HashSet<String> {
    let entries: Vec<Value> = mget(memory, "facts_log").items().iter().filter(|e| e.is_dict() && e.get("key").map(Value::truthy).unwrap_or(false)).cloned().collect();
    let framed: Vec<(String, Value)> = entries
        .iter()
        .filter_map(|e| {
            let key = py_str(e.get("key").unwrap());
            key.strip_suffix("_frame").map(|k| (k.to_string(), e.get("turn").cloned().unwrap_or(Value::None)))
        })
        .collect();
    let mut moved: HashSet<String> = HashSet::new();
    let mut last: HashMap<String, String> = HashMap::new();
    for entry in &entries {
        let key = py_str(entry.get("key").unwrap());
        // A list is a thing that grows: a later line adds one, it does not replace the old one.
        if key.ends_with("_frame") || matches!(entry.get("value"), Some(Value::List(_)) | Some(Value::Tuple(_)) | Some(Value::Dict(_))) {
            continue;
        }
        let text_ = fold(&sget(entry, "text"));
        if let Some(older) = last.get(&key) {
            if !older.is_empty() && *older != text_ {
                let turn = entry.get("turn").cloned().unwrap_or(Value::None);
                if _reads_as_a_correction(&sget(entry, "text")) || !framed.iter().any(|(k, t)| *k == key && *t == turn) {
                    moved.insert(older.clone());
                }
            }
        }
        last.insert(key, text_);
    }
    moved
}

strset!(SUMMARY_KEYS);

/// Python: bot/memory/store_replies.py:11325 `_the_summary_already_says(memory, flat)`
///
/// True when every fact the line gave is one the summary says, at the value it holds.
pub fn _the_summary_already_says(memory: &mut Memory, flat: &str) -> bool {
    let known = user(memory);
    let mut keys: HashSet<String> = HashSet::new();
    for entry in mget(memory, "facts_log").items() {
        if !entry.is_dict() || !entry.get("key").map(Value::truthy).unwrap_or(false) {
            continue;
        }
        if fold(&sget(entry, "text")) != flat {
            continue;
        }
        let key = py_str(entry.get("key").unwrap());
        let then = entry.get("value").cloned().unwrap_or(Value::None);
        let now = known.get(&key).cloned().unwrap_or(Value::None);
        let container = |v: &Value| matches!(v, Value::List(_) | Value::Tuple(_) | Value::Dict(_));
        if container(&now) || container(&then) {
            continue;
        }
        if then.is_none() || now.is_none() || py_str(&then).to_lowercase() != py_str(&now).to_lowercase() {
            return false;
        }
        keys.insert(key);
    }
    !keys.is_empty() && keys.iter().all(|k| SUMMARY_KEYS.contains(k))
}

/// Python: bot/memory/store_replies.py:11343 `_safety_lines(memory)`
///
/// The lines the bot answered with care: a crisis.
pub fn _safety_lines(memory: &Memory) -> HashSet<String> {
    memory.turns.iter().filter(|t| t.module.as_deref() == Some("safety")).map(|t| fold(&t.user)).collect()
}

rx!(CHANGE_WORD, NEGATIVE_LINE, AN_ATTITUDE);
static FIRST_PERSON_LEAD: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:i|ive|i've|im|i'm|we|weve) ", 0));

/// Python: bot/memory/store_replies.py:11371 `_shape(line)`
///
/// The frame a line about the person is written in: "im", "i have", "i dont".
pub fn _shape(line: &str) -> String {
    let words = text::split(line);
    if words.is_empty() {
        return String::new();
    }
    if ["im", "i'm", "ive", "i've", "iam"].contains(&words[0].as_str()) {
        return words[0].clone();
    }
    if words.len() > 1 && words[0] == "i" && ["am", "have", "got", "dont", "do", "cant", "cannot", "was", "were", "live", "work"].contains(&words[1].as_str()) {
        return words[..2].join(" ");
    }
    words[0].clone()
}

/// Python: bot/memory/store_replies.py:11383 `_out_of_date_lines(memory)`
///
/// The lines a later line changed, without the store having a key to record it under.
pub fn _out_of_date_lines(memory: &Memory) -> HashSet<String> {
    let lines: Vec<String> = last_users(memory, 8).iter().map(|l| fold(l)).collect();
    let logged: HashSet<String> = mget(memory, "facts_log").items().iter().filter(|e| e.is_dict()).map(|e| fold(&sget(e, "text"))).collect();
    let mut out: HashSet<String> = HashSet::new();
    for (index, line) in lines.iter().enumerate() {
        if !CHANGE_WORD.is_match(line) {
            continue;
        }
        let changed: HashSet<String> =
            WORDS_AND_DIGITS.findall_str(line).into_iter().filter(|w| text::len(w) > 2 && !STOPWORDS.contains(w) && CHANGE_WORD.fullmatch(w).is_none()).collect();
        for older in &lines[..index] {
            if logged.contains(older) {
                continue;
            }
            let knew: HashSet<String> = WORDS_AND_DIGITS.findall_str(older).into_iter().filter(|w| text::len(w) > 2 && !STOPWORDS.contains(w)).collect();
            if changed.iter().any(|w| knew.contains(w)) || _shape(older) == _shape(line) {
                out.insert(older.clone());
            }
        }
    }
    // A negative line and a later line where they do the thing.
    for (index, older) in lines.iter().enumerate() {
        if !NEGATIVE_LINE.is_match(older) || AN_ATTITUDE.is_match(older) {
            continue;
        }
        let knew: Vec<String> = LETTER_WORDS.findall_str(older).into_iter().filter(|w| text::len(w) > 2 && !STOPWORDS.contains(w)).map(|w| _root(&w)).collect();
        for later in &lines[index + 1..] {
            if FIRST_PERSON_LEAD.match_(later).is_none() || NEGATIVE_LINE.is_match(later) || later.ends_with('?') {
                continue;
            }
            let there: Vec<String> = LETTER_WORDS.findall_str(later).into_iter().filter(|w| text::len(w) > 2 && !STOPWORDS.contains(w)).map(|w| _root(&w)).collect();
            if knew.iter().any(|a| there.iter().any(|b| _one_word(a, b))) {
                out.insert(older.clone());
                break;
            }
        }
    }
    out
}

/// Python: bot/memory/store_replies.py:11427 `_root(word)`
///
/// The same word with its ending taken off: drive and driving are one thing.
pub fn _root(word: &str) -> String {
    let mut word: Vec<char> = text::strip_chars(&word.to_lowercase(), "'").chars().collect();
    for suffix in ["ing", "ers", "er", "ed", "es", "s", "ly"] {
        let s: Vec<char> = suffix.chars().collect();
        if word.ends_with(&s) && word.len() - s.len() >= 3 {
            word.truncate(word.len() - s.len());
            break;
        }
    }
    let n = word.len();
    if n > 3 && word[n - 1] == word[n - 2] && !"aeiou".contains(word[n - 1]) {
        word.truncate(n - 1); // swimming, stopped: the doubling is the ending, not the word
    }
    if word.len() > 3 && word.last() == Some(&'e') {
        word.pop();
    }
    word.into_iter().collect()
}

/// Python: bot/memory/store_replies.py:11440 `_one_word(first, second)`
///
/// Whether two words are the same thing said with a different ending: drive, driving.
pub fn _one_word(first: &str, second: &str) -> bool {
    first == second || (_root(first) == _root(second) && text::len(&_root(first)) >= 3)
}

rx!(SELF_JUDGEMENT, SELF_WORST);

/// Python: bot/memory/store_replies.py:11475 `_judging_themselves(line)`
///
/// Whether the line is the person rating themselves rather than telling the bot a fact.
pub fn _judging_themselves(line: &str) -> bool {
    let said = fold(line);
    !said.is_empty() && (SELF_JUDGEMENT.match_(&said).is_some() || SELF_WORST.match_(&said).is_some())
}

/// Python: bot/memory/store_replies.py:11480 `_recent_told(memory, limit=3)`
///
/// The last few things they told you, newest first, in their own words: (phrase, is the line
/// the chat is on).
pub fn _recent_told(memory: &mut Memory, limit: usize) -> Vec<(String, bool)> {
    let mut out: Vec<(String, bool)> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    // A chat the bot has answered with care gets no inventory.
    if crate::memory::topics::in_a_safety_thread(memory) {
        return out;
    }
    let moved = _lines_that_moved(memory);
    let care = _safety_lines(memory);
    let record = crate::memory::topics::current(memory).cloned().unwrap_or(Value::None);
    let mut lines: Vec<String> = last_users(memory, 8).into_iter().rev().collect();
    if lines.is_empty() {
        lines = vec![sget(&record, "text")];
    }
    let newest = fold(&lines[0]);
    let out_of_date = _out_of_date_lines(memory);
    for (index, line) in lines.iter().enumerate() {
        let flat = fold(line);
        // A line whose fact the store now holds a different value for is not said.
        if moved.contains(&flat) || out_of_date.contains(&flat) {
            continue;
        }
        // A line the bot answered with care is not a fact about the person.
        if care.contains(&flat) || _judging_themselves(&flat) {
            continue;
        }
        // A correction takes back what came before it, so the walk stops there.
        if _reads_as_a_correction(line) {
            if !_a_figure_the_store_could_not_keep(line, memory) {
                break;
            }
            continue;
        }
        // Half of the question they just asked is not something they told us.
        let next = lines.get(index + 1).map(String::as_str).unwrap_or("");
        if crate::memory::ack::_the_rest_of_a_question(line, next) {
            continue;
        }
        if A_CLAIM_OF_TELLING.match_(&flat).is_some() && !crate::memory::facts::extract(line, false).val().truthy() {
            // "i told you already" is about the telling, not a fact about the person.
            continue;
        }
        if crate::memory::recall::ask(&flat, false).as_deref() == Some("me") {
            continue;
        }
        if _the_summary_already_says(memory, &flat) {
            continue;
        }
        let mut phrase = text::squash(&_told_phrase(&crate::memory::facts::no_tone(line)));
        loop {
            let words = text::split(&phrase);
            match words.last() {
                Some(last) if SAY_BACK_VAGUE_TAIL.contains(&last.to_lowercase()) => phrase = words[..words.len() - 1].join(" "),
                _ => break,
            }
        }
        // "can not swim" reads like a translation.
        let phrase = phrase.replace("can not", "cannot");
        let words = _summary_words(&phrase);
        if words.is_empty() || words.iter().all(|w| SAY_BACK_VAGUE.contains(w)) || SAY_BACK_CORRECTION.is_match(&phrase) {
            continue;
        }
        let key = phrase.to_lowercase();
        if seen.contains(&key) {
            continue;
        }
        seen.insert(key);
        out.push((phrase, flat == newest));
        if out.len() >= limit {
            break;
        }
    }
    out
}

strset!(AGREEMENT_ONLY);

/// Python: bot/memory/store_replies.py:11567 `_all_about_the_bot(memory)`
///
/// Whether every line they typed was a question about the bot itself.
pub fn _all_about_the_bot(memory: &Memory) -> bool {
    let lines: Vec<String> = last_users(memory, 8).iter().map(|l| text::strip(l).to_string()).filter(|l| !l.is_empty()).collect();
    if lines.is_empty() {
        return false;
    }
    lines.iter().all(|line| crate::compose::fallback::about_the_bot(&fold(line)).val().truthy())
}

rx!(ANY_NUMBER);

/// Python: bot/memory/store_replies.py:11582 `_a_figure_the_store_could_not_keep(line, memory)`
///
/// True when the line is only a new figure for something, and no fact moved for it.
pub fn _a_figure_the_store_could_not_keep(line: &str, memory: &Memory) -> bool {
    if mget(memory, "correction").get("key").map(Value::truthy).unwrap_or(false) {
        return false;
    }
    let found = crate::memory::facts::extract(line, false).val().get("correction").filter(|c| c.truthy()).cloned().unwrap_or(Value::dict());
    let value = text::squash(&sget(&found, "to"));
    !value.is_empty() && ANY_NUMBER.fullmatch(&value).is_some()
}

static THEIRS_IN_THE_LINE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:i|im|i'm|my|we|our|me|us|mine)\b", 0));
static TO_THE_BOT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:you|your|youre|you're|u)\b", 0));

/// Python: bot/memory/store_replies.py:11597 `_the_thing_the_chat_is_on(memory)`
///
/// The phrase to name in place of facts, or "" when the chat holds nothing of the person's.
pub fn _the_thing_the_chat_is_on(memory: &Memory) -> String {
    if _all_about_the_bot(memory) {
        return String::new();
    }
    let record = crate::memory::topics::current(memory).cloned().unwrap_or(Value::None);
    let text_ = text::squash(&sget(&record, "text")).to_lowercase();
    if text_.is_empty() {
        return String::new();
    }
    let theirs = THEIRS_IN_THE_LINE.is_match(&text_);
    // A question put to the bot is not a subject of theirs either.
    let to_the_bot = TO_THE_BOT.is_match(&text_);
    if !(theirs || (crate::memory::corrected::_asks(&text_) && !to_the_bot)) {
        return String::new();
    }
    text::squash(&ret_str(crate::memory::thread::about(memory, &Value::None)))
}

/// `{word.strip(chars) for word in text.lower().split()} - SUMMARY_FILLER`.
fn clause_words(clause: &str) -> HashSet<String> {
    text::split(&clause.to_lowercase()).iter().map(|w| text::strip_chars(w, ".,!?;:'\"").to_string()).filter(|w| !SUMMARY_FILLER.contains(w)).collect()
}

/// Python: bot/memory/store_replies.py:11618 `_a_fact_of_its_own(clause)`
///
/// Whether a clause beside a judgement says something, rather than agreeing with the verdict.
pub fn _a_fact_of_its_own(clause: &str) -> bool {
    let words = clause_words(clause);
    !words.is_empty() && !words.iter().all(|w| AGREEMENT_ONLY.contains(w))
}

static JUDGEMENT_SPLIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s*(?:[,;.]|\bbut\b|\band\b)\s*", 0));

/// Python: bot/memory/store_replies.py:11623 `_told_phrase(text)`
///
/// One line said back inside a sentence, or "" when it is not something they told you.
pub fn _told_phrase(text_: &str) -> String {
    if text_.is_empty() {
        return String::new();
    }
    let reading = crate::understanding::understand(text_, Some(&Memory::new()));
    // A question or an ask is not something they told you; a feeling is how they are.
    if ["question", "greeting", "thanks"].contains(&reading.kind.as_str())
        || ["question", "request", "greeting", "thanks"].contains(&reading.intent.as_str())
        || reading.intent == "feeling"
        || !reading.asks.is_empty()
    {
        return String::new();
    }
    if reading.about_bot {
        return String::new();
    }
    let mut text_ = text_.to_string();
    if crate::memory::facts::is_a_judgement(&text_) {
        // A judgement about themselves is how they feel; the clause beside it that is not one is said back.
        let rest: Vec<String> = JUDGEMENT_SPLIT.split(&text_).into_iter().filter(|p| !text::strip(p).is_empty()).map(|p| text::strip(&p).to_string()).collect();
        text_ = rest.into_iter().find(|p| !crate::memory::facts::is_a_judgement(p) && _a_fact_of_its_own(p)).unwrap_or_default();
        if text_.is_empty() {
            return String::new();
        }
    }
    let said = _with_the_name(&crate::memory::thread::restate(&crate::memory::filler::without_filler(&text_)), &text_);
    let words: Vec<String> = text::split(&said).into_iter().filter(|w| !text::strip_chars(w, ".,!?;:").is_empty()).collect();
    let body: HashSet<String> =
        words.iter().map(|w| text::strip_chars(w, ".,!?;:").to_lowercase()).filter(|w| !SUMMARY_FILLER.contains(w)).collect();
    if body.is_empty() || body.iter().all(|w| SMALL_TALK_ONLY.contains(w)) || words.len() > 14 {
        return String::new();
    }
    words.join(" ")
}

static EXPANSIONS: LazyLock<Vec<(Re, &'static str)>> = LazyLock::new(|| {
    [
        (r"n't\b", " not"),
        (r"'re\b", " are"),
        (r"'ve\b", " have"),
        (r"'ll\b", " will"),
        (r"'d\b", " would"),
        (r"\bi'm\b", "i am"),
        (r"\bim\b", "i am"),
        (r"\bive\b", "i have"),
        (r"\byoure\b", "you are"),
        (r"\bdont\b", "do not"),
        (r"\bdoesnt\b", "does not"),
        (r"\bim\b", "i am"),
    ]
    .into_iter()
    .map(|(p, w)| (Re::new(p, 0), w))
    .collect()
});

/// Python: bot/memory/store_replies.py:11660 `_expanded(text)`
///
/// A line with its contractions written out: "you're veg" and "you are veg" are the same line.
pub fn _expanded(text_: &str) -> String {
    let mut out = text_.to_lowercase().replace('\u{2019}', "'");
    for (pattern, word) in EXPANSIONS.iter() {
        out = pattern.sub(word, &out);
    }
    out
}

static NUMBER_AS_A_WORD: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("NUMBER_AS_A_WORD").str_map());

/// Python: bot/memory/store_replies.py:11677 `_a_judgement_was_said(memory)`
///
/// Whether a line they said in this chat judged themselves rather than stating a fact.
pub fn _a_judgement_was_said(memory: &Memory) -> bool {
    memory.turns.iter().any(|t| crate::memory::facts::is_a_judgement(&t.user))
}

strset!(FACT_FRAME_WORDS);

/// Python: bot/memory/store_replies.py:11695 `_plain_words(text)`
///
/// The words of a line as they are, without the length rule the summary meaning applies.
pub fn _plain_words(text_: &str) -> HashSet<String> {
    WORDS_AND_DIGITS.findall_str(&text_.to_lowercase().replace('-', " ")).into_iter().collect()
}

/// Python: bot/memory/store_replies.py:11699 `_says_nothing_new(phrase, told_words, body_words)`
///
/// Whether the line is the fact above, in the person's own words, and nothing more.
pub fn _says_nothing_new(phrase: &str, told_words: &HashSet<String>, body_words: &str) -> bool {
    let body = _summary_words(body_words);
    if told_words.is_subset(&body) {
        return true;
    }
    // The short words a summary drops are sometimes the whole of the thing shared.
    let rest: HashSet<String> = _plain_words(phrase).into_iter().filter(|w| !FACT_FRAME_WORDS.contains(w)).collect();
    if rest.is_empty() {
        return false;
    }
    rest.is_subset(&_plain_words(body_words))
}

static DIET_NAMES: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.memory.facts.DIET_NAMES"));

/// Python: bot/memory/store_replies.py:11713 `_summary_words(text)`
///
/// The words that carry the meaning of a line, for telling a repeat from a new thing.
pub fn _summary_words(text_: &str) -> HashSet<String> {
    use crate::memory::topics::stem;
    let mut words = HashSet::new();
    for word in text::split(&_expanded(text_).replace(';', " ").replace(',', " ")) {
        let mut word = stem(text::strip_chars(&word, ".,!?;:'"));
        if let Some(w) = NUMBER_AS_A_WORD.get(&word) {
            word = w.clone();
        }
        // A figure is a fact in its own right, so it is kept whatever its length.
        let digit = crate::understanding::pyuni::is_digit(&word);
        if !digit {
            // The slang has to fold the same way the store keeps it.
            let plain = DIET_NAMES.get(&word).cloned().unwrap_or_else(|| word.clone());
            word = stem(&plain);
        }
        let digit = crate::understanding::pyuni::is_digit(&word);
        if !word.is_empty() && (digit || text::len(&word) > 2) && !SUMMARY_FILLER.contains(&word) && !SMALL_TALK_ONLY.contains(&word) {
            words.insert(word);
        }
    }
    words
}

/// Python: bot/memory/store_replies.py:11730 `_already_said(words, said)`
///
/// Whether the facts above already say this: "allergy" and "allergic" are the same word here.
pub fn _already_said(words: &HashSet<String>, said: &str) -> bool {
    let mut said = _summary_words(said);
    if said.is_empty() {
        return false; // nothing above says anything, so this is new
    }
    if words.is_empty() {
        return true;
    }
    for word in words {
        let same: Vec<String> = said
            .iter()
            .filter(|other| *other == word || (text::len(word) >= 4 && text::len(other) >= 4 && text::slice(other, 0, Some(4)) == text::slice(word, 0, Some(4))))
            .cloned()
            .collect();
        for s in same {
            said.remove(&s);
        }
    }
    said.len() < 1.max(words.len() / 2)
}

static _TASTE_LEFT: LazyLock<&'static Re> = LazyLock::new(|| g("_TASTE_LEFT").as_re().unwrap());
static STILL_AGAIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:again|still|actually|really|now|any ?more)\b", 0));

/// Python: bot/memory/store_replies.py:11748 `_taste_reply(memory, low)`
///
/// The taste they have already said, said back as the answer to "do i like fish?".
pub fn _taste_reply(memory: &mut Memory, low: &str) -> Option<String> {
    use crate::memory::topics::{content, stem};
    // `_TASTE_LEFT.split(low, maxsplit=1)[-1]`: the words after the ask.
    let asked = match _TASTE_LEFT.search(low) {
        Some(m) => low[m.end()..].to_string(),
        None => String::new(),
    };
    let asked = text::strip_chars(&STILL_AGAIN.sub("", &asked), " ?!.,").to_string();
    if asked.is_empty() {
        return None;
    }
    let wanted: Vec<String> = content(&asked).iter().map(|w| stem(w)).collect();
    if wanted.is_empty() {
        return None;
    }
    let tastes = user(memory).get("tastes").filter(|v| v.truthy()).map(py_list).unwrap_or_default();
    for taste in tastes.iter().rev() {
        let thing = sget(taste, "thing").to_lowercase();
        let bare = if thing.starts_with("your ") { text::slice(&thing, 5, None) } else { thing.clone() };
        let said: Vec<String> = content(&bare).iter().map(|w| stem(w)).collect();
        if said.is_empty() {
            continue;
        }
        if wanted.iter().filter(|w| said.contains(w)).count() < 1.max(wanted.len().saturating_sub(1)) {
            continue;
        }
        let how = taste.get("how").filter(|h| h.truthy()).map(py_str).unwrap_or_else(|| "yes".to_string());
        // Said back with the possessive it was kept in.
        let named = if text::strip(&text::slice(&thing, 5, None)).is_empty() || !thing.to_lowercase().starts_with("your ") { bare.clone() } else { thing.clone() };
        if how == "no" {
            return Some("You don't, you told me you're not keen on ".to_string() + &named + ".");
        }
        if how == "unsure" {
            return Some("You weren't sure about ".to_string() + &named + ", that's all you've said.");
        }
        return Some("You do, you told me you like ".to_string() + &named + ".");
    }
    None
}

/// Python: bot/memory/store_replies.py:11781 `_tastes_said(memory)`
///
/// "what do i like?" as the short list of tastes, not the whole summary of what is known.
pub fn _tastes_said(memory: &mut Memory) -> Option<String> {
    let (mut liked, mut other): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    for taste in user(memory).get("tastes").filter(|v| v.truthy()).map(py_list).unwrap_or_default() {
        let thing = text::strip(&sget(&taste, "thing")).to_string();
        if thing.is_empty() {
            continue;
        }
        let how = taste.get("how").filter(|h| h.truthy()).map(py_str).unwrap_or_else(|| "yes".to_string());
        if how == "no" {
            other.push("You're not keen on ".to_string() + &thing + ".");
        } else if how == "unsure" {
            other.push("You're not sure about ".to_string() + &thing + ".");
        } else {
            liked.push(thing);
        }
    }
    let mut said: Vec<String> = Vec::new();
    if !liked.is_empty() {
        let joined = if liked.len() == 1 { liked[0].clone() } else { liked[..liked.len() - 1].join(", ") + " and " + &liked[liked.len() - 1] };
        said.push("You like ".to_string() + &joined + ".");
    }
    said.extend(other);
    if said.is_empty() {
        None
    } else {
        Some(said.join(" "))
    }
}

rx!(AGE_SAID_IN_WORDS);
static AND_ALSO_PLUS: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:and|also|plus)\s+", crate::py::re::I));

/// Python: bot/memory/store_replies.py:11813 `_without_the_age(line)`
///
/// The line with the age taken out of it: "you're 41 and you're training" -> "you're training".
pub fn _without_the_age(line: &str) -> String {
    let without = text::strip_chars(&AGE_SAID_IN_WORDS.subn("", line, 1), " .,;:!?-").to_string();
    text::strip(&AND_ALSO_PLUS.sub("", &without)).to_string()
}

/// Python: bot/memory/store_replies.py:11818 `_the_words_of_a_fact(value)`
///
/// The words a stored fact is made of, whatever shape the store keeps it in.
pub fn _the_words_of_a_fact(value: &Value) -> String {
    match value {
        Value::Bool(_) => String::new(),
        Value::Int(_) | Value::Float(_) => py_str(value),
        Value::Str(s) => s.to_string(),
        Value::Dict(d) => d.iter().map(|(_, v)| _the_words_of_a_fact(v)).collect::<Vec<_>>().join(" "),
        Value::List(items) | Value::Tuple(items) | Value::Set(items) => items.iter().map(_the_words_of_a_fact).collect::<Vec<_>>().join(" "),
        _ => String::new(),
    }
}

/// Python: bot/memory/store_replies.py:11833 `_a_fact_the_recap_does_not_say(memory, phrase, body_words)`
///
/// True when this line filed a fact the recap above does not say.
pub fn _a_fact_the_recap_does_not_say(memory: &mut Memory, phrase: &str, body_words: &str) -> bool {
    let said = _summary_words(body_words);
    let entries = mget(memory, "facts_log").clone();
    for entry in entries.items() {
        if !entry.is_dict() || !entry.get("key").map(Value::truthy).unwrap_or(false) {
            continue;
        }
        let text_ = fold(&sget(entry, "text"));
        if text::squash(&_told_phrase(&crate::memory::facts::no_tone(&text_))) != phrase {
            continue;
        }
        let key = py_str(entry.get("key").unwrap());
        let words = _summary_words(&_the_words_of_a_fact(user(memory).get(&key).unwrap_or(&Value::None)));
        if words.iter().any(|w| !said.contains(w)) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/store_replies.py:11857 `_kids_told(memory)`
///
/// How many children the person said they have, or 0.
pub fn _kids_told(memory: &mut Memory) -> i64 {
    match user(memory).get("kids") {
        Some(Value::Bool(_)) => 0,
        Some(Value::Int(n)) => (*n).max(0),
        Some(Value::Str(s)) if crate::understanding::pyuni::is_digit(text::strip(s)) => py_int(text::strip(s)).expect("ValueError"),
        _ => 0,
    }
}

strset!(CHILD);

/// Python: bot/memory/store_replies.py:11873 `_a_child_is_described(memory)`
///
/// True when the store can say a child of theirs by name or age, which says the same thing.
pub fn _a_child_is_described(memory: &mut Memory) -> bool {
    others(memory).items().iter().any(|entry| {
        let who = text::rstrip_chars(&sget(entry, "who"), "s").to_string();
        !who.is_empty()
            && CHILD.contains(&who)
            && ["name", "age", "ages"].iter().any(|k| entry.get(k).map(Value::truthy).unwrap_or(false))
    })
}

static KID_NUMBERS_SAID: LazyLock<HashMap<i64, String>> = LazyLock::new(|| {
    let mut out = HashMap::new();
    for (word, number) in data::get("bot.memory.facts.KID_NUMBERS").pairs() {
        out.insert(number.as_int().unwrap(), word.str().to_string());
    }
    out
});

/// Python: bot/memory/store_replies.py:11882 `_a_few_children(count)`
///
/// Two kids, one child, four children - the count the way a person says it.
pub fn _a_few_children(count: i64) -> String {
    if count == 1 {
        return "one child".to_string();
    }
    KID_NUMBERS_SAID.get(&count).cloned().unwrap_or_else(|| count.to_string()) + " kids"
}

rx!(SUMMED_UP);

/// Python: bot/memory/store_replies.py:11909 `_summed_up(low)`
///
/// True for a line that asks for the whole chat summed up, rather than for one line of it.
pub fn _summed_up(low: &str) -> bool {
    SUMMED_UP.match_(text::strip(low)).is_some()
}

/// Python: bot/memory/store_replies.py:11913 `_summary_reply(memory)`
///
/// Everything the store holds about them, said in one sentence: "what do you know about me".
pub fn _summary_reply(memory: &mut Memory) -> Option<String> {
    use crate::memory::relations::_called;
    let known = user(memory);
    let has = |k: &str| known.get(k).map(Value::truthy).unwrap_or(false);
    let mut parts: Vec<String> = Vec::new();
    if has("name") {
        parts.push("you're ".to_string() + &py_str(known.get("name").unwrap()));
    }
    // Their age is a fact they gave, so it is in the summary.
    if has("age") {
        let age = py_str(known.get("age").unwrap());
        if !parts.is_empty() && parts[0].starts_with("you're ") && !parts[0].contains(',') {
            parts[0] += &(", ".to_string() + &age);
        } else {
            parts.push("you're ".to_string() + &age);
        }
    }
    if has("job") {
        // The summary says the job the way the job reply does, from its frame.
        let mut said = _job_phrase(&known);
        if has("city") {
            said += &(" in ".to_string() + &py_str(known.get("city").unwrap()));
        }
        parts.push(said);
    } else if has("city") {
        parts.push("you live in ".to_string() + &py_str(known.get("city").unwrap()));
    }
    let shift = text::strip(&sget(&known, "shift")).to_string();
    if !shift.is_empty() {
        parts.push("you work ".to_string() + &shift);
    }
    let place = text::strip(&sget(&known, "workplace")).to_string();
    if !place.is_empty() {
        // Where they work is kept beside the role.
        parts.push("you work ".to_string() + &_place_phrase(&place));
    }
    if let Some(origin) = known.get("from").filter(|o| o.truthy()) {
        if py_str(origin).to_lowercase() != sget(&known, "city").to_lowercase() {
            parts.push("you're from ".to_string() + &py_str(origin));
        }
    }
    let mut said_things: HashSet<String> = HashSet::new();
    for pet in known.get("pets").filter(|v| v.truthy()).map(py_list).unwrap_or_default() {
        parts.push("you've got ".to_string() + &_a_pet(&pet));
        for k in ["animal", "name"] {
            said_things.insert(text::rstrip_chars(&sget(&pet, k).to_lowercase(), "s").to_string());
        }
    }
    // A taste is a thing they told the bot about themselves.
    for taste in known.get("tastes").filter(|v| v.truthy()).map(py_list).unwrap_or_default() {
        let thing = text::strip(&sget(&taste, "thing")).to_string();
        if thing.is_empty() {
            continue;
        }
        // A taste about a thing the summary has already named is said once, in the fuller line.
        let lowered = thing.to_lowercase();
        let bare = if lowered.starts_with("your ") { text::slice(&lowered, 5, None) } else { lowered };
        if said_things.contains(&bare) || said_things.contains(text::rstrip_chars(&bare, "s")) {
            continue;
        }
        let how = taste.get("how").filter(|h| h.truthy()).map(py_str).unwrap_or_else(|| "yes".to_string());
        parts.push(match how.as_str() {
            "no" => "you're not keen on ".to_string() + &thing,
            "unsure" => "you're not sure about ".to_string() + &thing,
            _ => "you like ".to_string() + &thing,
        });
    }
    if has("allergies") {
        parts.push("you're allergic to ".to_string() + &py_list(known.get("allergies").unwrap()).iter().map(py_str).collect::<Vec<_>>().join(", "));
    }
    let mut named: HashSet<String> = HashSet::new();
    for entry in others(memory).items() {
        let who_v = entry.get("who").cloned().unwrap_or(Value::None);
        let who = py_str(&who_v);
        let has_who = who_v.truthy();
        let p_allergic = entry.get("allergies").filter(|v| v.truthy()).map(py_list).unwrap_or_default();
        if has_who && !p_allergic.is_empty() {
            parts.push("your ".to_string() + &who + &_is_are(&who) + "allergic to " + &p_allergic.iter().map(py_str).collect::<Vec<_>>().join(", "));
            named.insert(who.clone());
        }
        // A person they named, or told the age of, is a fact of theirs like the allergy is.
        let p_name = s_or(entry.get("name"));
        let p_age = s_or(entry.get("age"));
        let p_ages: Vec<String> = entry.get("ages").filter(|v| v.truthy()).map(py_list).unwrap_or_default().iter().filter(|a| is_int(Some(*a))).map(py_str).collect();
        let p_diet = s_or(entry.get("diet"));
        let a_child = CHILD.contains(text::rstrip_chars(&who, "s"));
        if has_who && !p_name.is_empty() && a_child && !p_age.is_empty() {
            // A child given with a name and an age is said as the child.
            parts.push(_called(&p_name) + " is " + &p_age);
            named.insert(who.clone());
        } else if has_who && !p_name.is_empty() {
            let mut said = "your ".to_string() + &who + " is called " + &_called(&p_name);
            if !p_age.is_empty() {
                said += &(" and is ".to_string() + &p_age);
            }
            parts.push(said);
            named.insert(who.clone());
        } else if has_who && !p_ages.is_empty() {
            let said_ages = if p_ages.len() > 1 {
                p_ages[..p_ages.len() - 1].join(", ") + " and " + &p_ages[p_ages.len() - 1]
            } else {
                p_ages[0].clone()
            };
            parts.push("your ".to_string() + &who + &_is_are(&who) + &said_ages);
            named.insert(who.clone());
        } else if has_who && !p_age.is_empty() && a_child {
            // One child, given as an age rather than a count.
            parts.push("you've got a ".to_string() + &p_age + " year old");
            named.insert(who.clone());
        } else if has_who && !p_age.is_empty() {
            parts.push("your ".to_string() + &who + " is " + &p_age);
            named.insert(who.clone());
        }
        if has_who && !p_diet.is_empty() {
            parts.push("your ".to_string() + &who + " " + &_a_diet_predicate(&who, &p_diet));
            named.insert(who.clone());
        }
    }
    // A count of children is a fact they gave.
    let told_kids = _kids_told(memory);
    if told_kids != 0 && !_a_child_is_described(memory) {
        let count = _a_few_children(told_kids);
        match parts.last_mut() {
            Some(last) if last.starts_with("you've got ") => *last += &(" and ".to_string() + &count),
            _ => parts.push("you've got ".to_string() + &count),
        }
    }
    if has("diet") {
        let diet = py_str(known.get("diet").unwrap());
        parts.push(if diet.starts_with("no ") {
            "you don't ".to_string() + &_diet_how(&diet) + " " + &text::slice(&diet, 3, None)
        } else {
            "you're ".to_string() + &diet
        });
    }
    if has("condition") {
        parts.push("you're ".to_string() + &py_list(known.get("condition").unwrap()).iter().map(py_str).collect::<Vec<_>>().join(", "));
    }
    if has("budget") {
        parts.push("your budget is ".to_string() + &py_str(known.get("budget").unwrap()));
    }
    let mut plans: Vec<Value> = known.get("plans").filter(|v| v.truthy()).map(py_list).unwrap_or_default();
    // A correction puts the plan itself under `user["plan"]`.
    if let Some(plan) = known.get("plan").filter(|p| p.truthy()) {
        if !plans.contains(plan) {
            plans.push(plan.clone());
        }
    }
    let last_plan = plans.last().map(py_str).unwrap_or_default();
    // A plan often names what another part would say, so it is said once, in the fuller line.
    let told = _last_told(memory);
    let in_tastes = known.get("tastes").filter(|v| v.truthy()).map(py_list).unwrap_or_default().iter().map(|t| sget(t, "thing").to_lowercase()).collect::<Vec<_>>().join(" ");
    let people: Vec<String> = known
        .get("people")
        .filter(|v| v.truthy())
        .map(py_list)
        .unwrap_or_default()
        .iter()
        .map(py_str)
        .filter(|p| {
            let low = p.to_lowercase();
            !named.contains(p) && !last_plan.to_lowercase().contains(&low) && !told.to_lowercase().contains(&low) && !in_tastes.contains(&low)
        })
        .collect();
    if !people.is_empty() {
        parts.push("you mentioned your ".to_string() + &people[..people.len().min(3)].join(", "));
    }
    if let Some(last) = plans.last() {
        // The plan is said in the shape it was given.
        let last = py_str(last);
        let line = crate::memory::plan_line(memory, &last);
        let mut plan = lower_first(&crate::memory::plan_sentence(&last, &line));
        if has("age") && AGE_SAID_IN_WORDS.is_match(&plan) {
            // The age is already said first in the summary: it is said once.
            plan = _without_the_age(&plan);
        }
        if !plan.is_empty() && !parts.join(" ").to_lowercase().contains(&plan.to_lowercase()) {
            parts.push(plan);
        }
    }
    // The chat itself, said back in their words, last and only when a fact above does not say it.
    let subject = crate::memory::topics::subject(memory).to_lowercase();
    let head = text::split(&subject).last().cloned().unwrap_or_default();
    let facts_body = parts.join("; ");
    let mut said_back = 0;
    for (phrase, from_current) in _recent_told(memory, 4) {
        let body_words = parts.join("; ");
        // Every word of the line is already in the facts above.
        let told_words = _summary_words(&phrase);
        let mut covered = !told_words.is_empty() && _says_nothing_new(&phrase, &told_words, &body_words);
        if !covered && told.starts_with("you work ") && body_words.contains("you work ") {
            // Where they work is already in the summary, said from the store.
            covered = true;
        }
        // "you grew up in Bolton" adds nothing to "you're from Bolton".
        let mut anchor_word = String::new();
        for word in text::split(&phrase) {
            if text::len(&word) > 3 && !SAY_BACK_VAGUE.contains(&word.to_lowercase()) {
                anchor_word = word.to_lowercase();
            }
        }
        // The topic guard gives way to a fact the recap does not say.
        let on_topic = from_current && !head.is_empty() && facts_body.to_lowercase().contains(&head) && !_a_fact_the_recap_does_not_say(memory, &phrase, &body_words);
        let anchored = !anchor_word.is_empty() && {
            let body = _summary_words(&body_words);
            _summary_words(&anchor_word).iter().any(|w| body.contains(w))
        };
        if covered || _already_said(&told_words, &body_words) || anchored || on_topic {
            continue;
        }
        // One "you said" and then the facts on their own.
        parts.push((if said_back == 0 { "you said " } else { "" }).to_string() + &phrase);
        said_back += 1;
        if said_back >= 2 {
            break;
        }
    }
    if !people.is_empty() && said_back != 0 {
        // Once a line says what happened to the person, the fuller line is the one to keep.
        let said_lines = parts.iter().filter(|p| !p.starts_with("you mentioned your ")).cloned().collect::<Vec<_>>().join(" ").to_lowercase();
        parts.retain(|p| !(p.starts_with("you mentioned your ") && text::split(p)[3..].iter().any(|w| said_lines.contains(w.as_str()))));
    }
    if parts.is_empty() {
        if _a_judgement_was_said(memory) {
            // A judgement about themselves is never filed as a fact.
            return Some(pick(
                &[
                    "I've got that you're being hard on yourself, and no facts about you yet. What should I know?",
                    "I have that you're being hard on yourself - and nothing about your life yet that I can say back.",
                    "You've told me how you're feeling about yourself, and nothing about your life yet. What should I know?",
                ],
                memory,
                "",
            ));
        }
        // Nothing to quote is not the same as having heard nothing.
        let about = _the_thing_the_chat_is_on(memory);
        if !about.is_empty() {
            return Some(pick(
                &[
                    "Nothing about you yet \u{2014} so far it's been ".to_string() + &about + ". What should I know?",
                    "I've got no facts about you yet; we've been on ".to_string() + &about + ". What should I know?",
                    "No notes about you so far \u{2014} you've had me on ".to_string() + &about + ". What should I know?",
                ],
                memory,
                "",
            ));
        }
        return Some("Not much yet \u{2014} you haven't told me about yourself. What should I know?".to_string());
    }
    let body = parts.join("; ");
    // The same summary twice in a row reads like a machine: the lead-in moves instead.
    Some(pick(
        &["So far I know ".to_string() + &body + ".", "Here's what you've told me: ".to_string() + &body + ".", "You've told me ".to_string() + &body + "."],
        memory,
        "",
    ))
}

/// Python: bot/memory/store_replies.py:12162 `_quote(text, limit=90)`
///
/// The user's own words, without the full stop before the closing quote.
pub fn _quote(text_: &str, limit: i64) -> String {
    let mut said = text::squash(text_);
    if text::len(&said) as i64 > limit {
        let mut cut = text::slice(&said, 0, Some((limit - 1) as isize));
        if !cut.ends_with(' ') && cut.contains(' ') {
            cut = cut[..cut.rfind(' ').unwrap()].to_string();
        }
        said = text::rstrip_chars(&cut, " ,;:-").to_string() + "\u{2026}";
    }
    "\"".to_string() + text::rstrip_chars(&said, " .!?") + "\""
}

rx!(HEAVY_TOPIC);

/// Python: bot/memory/store_replies.py:12185 `_heavy(line)`
///
/// True when the line is about a death, a crisis or someone being hurt.
pub fn _heavy(line: &str) -> bool {
    HEAVY_TOPIC.is_match(&text::squash(line))
}

/// Python: bot/memory/store_replies.py:12189 `_topics_reply(memory)`
///
/// The subjects the conversation has been on, most recent last.
pub fn _topics_reply(memory: &mut Memory) -> Option<String> {
    let lines: Vec<String> = py_list(&crate::memory::history::topic_lines(memory, 3).val()).iter().map(py_str).collect();
    let light: Vec<&String> = lines.iter().filter(|l| !_heavy(l)).collect();
    if lines.is_empty() {
        return Some(pick(&NOTHING_STARTED, memory, ""));
    }
    if light.is_empty() {
        return Some(pick(
            &[
                "We've been on something heavy, and I won't read it back to you. It is still here if you want to talk about it.",
                "What we've been on is heavy, so I'd rather not put it in quotation marks. Say more about it whenever you want.",
            ],
            memory,
            "",
        ));
    }
    let said = light.iter().map(|l| _quote(l, 60)).collect::<Vec<_>>().join("; ");
    if light.len() < lines.len() {
        return Some(pick(
            &[
                "We've been on ".to_string() + &said + ", and on something heavy I won't read back.",
                "Mostly ".to_string() + &said + ". There was something heavier too, and I have not forgotten it.",
            ],
            memory,
            "",
        ));
    }
    Some(pick(&["We've been on ".to_string() + &said + ".", "Mostly ".to_string() + &said + ".", "Covered so far: ".to_string() + &said + "."], memory, ""))
}

/// Python: bot/memory/store_replies.py:12216 `_first_sentence(text, limit=70)`
///
/// The first sentence of `text`, cut to `limit`: what the bot opened with, not the whole paragraph.
pub fn _first_sentence(text_: &str, limit: i64) -> String {
    let said = text::squash(text_);
    let mut end = text::len(&said) as isize;
    for mark in [". ", "? ", "! "] {
        let at = text::find(&said, mark);
        if 0 < at && at < end {
            end = at + 1;
        }
    }
    let mut said = text::strip(&text::slice(&said, 0, Some(end))).to_string();
    if text::len(&said) as i64 > limit {
        let mut cut = text::slice(&said, 0, Some((limit - 1) as isize));
        if cut.contains(' ') {
            cut = cut[..cut.rfind(' ').unwrap()].to_string();
        }
        said = text::rstrip_chars(&cut, " ,;:-").to_string() + "\u{2026}";
    }
    text::rstrip_chars(&said, " .!?").to_string()
}

rx!(CALL_ASK, CARRY_ON, STOP_IT, STOP_CONDITION, CLAUSE_SPLIT, SEE_SOMEONE, HALF_AN_HOUR, TIME_LENGTH, LEAD_LABEL, SENTENCE_SPLIT, SPLIT_MOVES, BUDGET, A_HORIZON);
strset!(LENGTH_FILLER);

/// Python: bot/memory/store_replies.py:12295 `_sentences(answer)`
///
/// The sentences of an answer, in the order given.
pub fn _sentences(answer: &str) -> Vec<String> {
    SENTENCE_SPLIT.split(&text::squash(answer)).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect()
}

/// Python: bot/memory/store_replies.py:12299 `_halves(sentence)`
///
/// A sentence cut where a semicolon, a dash or an "or" joins two moves (a list of str; the
/// contract types it `Value`).
pub fn _halves(sentence: &str) -> Value {
    str_list(&_halves_v(sentence))
}

/// `_halves` as the strs it is.
fn _halves_v(sentence: &str) -> Vec<String> {
    let mut parts = Vec::new();
    for part in SPLIT_MOVES.split(sentence) {
        let part = LEAD_LABEL.sub("", text::strip_chars(&part, " .;,"));
        if !part.is_empty() {
            parts.push(part);
        }
    }
    parts
}

/// Python: bot/memory/store_replies.py:12308 `_says_something(part)`
///
/// True when a part of an answer says more than how long it takes.
pub fn _says_something(part: &str) -> bool {
    let left = TIME_LENGTH.sub(" ", &text::squash(part));
    let left = HALF_AN_HOUR.sub(" ", &left);
    LETTER_WORDS.findall_str(&left.to_lowercase()).iter().filter(|w| !LENGTH_FILLER.contains(*w) && text::len(w) > 1).count() >= 2
}

static WORD_NUMBERS: LazyLock<HashMap<String, i64>> =
    LazyLock::new(|| g("WORD_NUMBERS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap())).collect());
static UNIT_SECONDS: LazyLock<HashMap<String, i64>> =
    LazyLock::new(|| g("UNIT_SECONDS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap())).collect());

/// Python: bot/memory/store_replies.py:12320 `_seconds(said)`
///
/// A length of time in seconds ("20 minutes", "half an hour", "a couple of days"), or None.
pub fn _seconds(said: &str) -> Option<i64> {
    let text_ = text::squash(said);
    if text_.is_empty() {
        return None;
    }
    let half = HALF_AN_HOUR.search(&text_);
    let found = TIME_LENGTH.search(&text_);
    if let Some(h) = &half {
        if found.as_ref().map(|f| h.start() < f.start()).unwrap_or(true) {
            return Some(1800);
        }
    }
    let found = found?;
    let count = found.name_str("count").to_lowercase();
    let number: Option<i64> = if count.starts_with("a couple") {
        Some(2)
    } else if count.starts_with("a few") {
        Some(3)
    } else if crate::understanding::pyuni::is_digit(&count) {
        py_int(&count)
    } else {
        WORD_NUMBERS.get(&count).copied()
    };
    let unit = text::rstrip_chars(&found.name_str("unit").to_lowercase(), "s").to_string();
    let number = number.filter(|n| *n != 0)?;
    let per = UNIT_SECONDS.get(&unit)?;
    Some(number * per)
}

/// Python: bot/memory/store_replies.py:12343 `_names_minutes(said)`
///
/// True when the wording names minutes rather than hours: "45 minutes", "twenty-minute".
pub fn _names_minutes(said: &str) -> bool {
    TIME_LENGTH.finditer(&text::squash(said)).iter().any(|m| m.name_str("unit").to_lowercase().starts_with("min"))
}

static IN_A_NUMBER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bin\s+(\d{1,2})\b", crate::py::re::I));

/// Python: bot/memory/store_replies.py:12348 `_longest(said, bare=False)`
///
/// The longest length of time named in a sentence, in seconds, or None (an int or None; the
/// contract types it `Value`).
pub fn _longest(said: &str, bare: &Value) -> Value {
    _longest_n(said, bare.truthy()).val()
}

/// `_longest` as the number it is.
fn _longest_n(said: &str, bare: bool) -> Option<i64> {
    let mut text_ = text::squash(said);
    if text_.is_empty() {
        return None;
    }
    if let Some(half) = HALF_AN_HOUR.search(&text_) {
        text_ = text_[..half.start()].to_string() + " 30 minutes " + &text_[half.end()..];
    }
    let found: Vec<String> = TIME_LENGTH.finditer(&text_).iter().map(|m| m.as_str().to_string()).collect();
    let mut lengths: Vec<i64> = found.iter().filter_map(|p| _seconds(p)).filter(|n| *n != 0).collect();
    if bare || lengths.iter().any(|s| *s <= 3600) {
        // A bare number after "in" is a length too, beside a figure that does name minutes.
        for m in IN_A_NUMBER.finditer(&text_) {
            let minutes = py_int(m.str(1)).expect("ValueError");
            if 0 < minutes && minutes <= 90 {
                lengths.push(minutes * 60);
            }
        }
    }
    lengths.into_iter().max()
}

static A_LENGTH_AS_THE_SUBJECT: LazyLock<&'static Re> = LazyLock::new(|| g("A_LENGTH_AS_THE_SUBJECT").as_re().unwrap());

/// Python: bot/memory/store_replies.py:12383 `_a_remark_about_a_length(answer)`
///
/// Whether the reply only remarks on a length, giving none to a thing it names.
pub fn _a_remark_about_a_length(answer: &str) -> bool {
    let mut remarked = false;
    for sentence in _sentences(answer) {
        if _longest_n(&sentence, false).is_none() {
            continue;
        }
        if !A_LENGTH_AS_THE_SUBJECT.is_match(&text::squash(&sentence)) {
            return false;
        }
        remarked = true;
    }
    remarked
}

/// One move of an answer: the move, its length in seconds, and its sentence number.
pub type Move = (String, Option<i64>, usize);

/// Python: bot/memory/store_replies.py:12398 `_moves(answer)`
///
/// The moves of an answer in the order given: the move, its length, and its sentence number.
pub fn _moves(answer: &str) -> Vec<Move> {
    let names = _names_minutes(answer);
    let mut out = Vec::new();
    for (at, sentence) in _sentences(answer).iter().enumerate() {
        for part in _halves_v(sentence) {
            if !_says_something(&part) {
                continue;
            }
            let took = _longest_n(&part, names);
            out.push((part, took, at));
        }
    }
    out
}

/// Python: bot/memory/store_replies.py:12409 `_up_to_the_move(answer, part)`
///
/// The answer from its first sentence to the one `part` was cut from, and no further.
pub fn _up_to_the_move(answer: &str, part: &str) -> Option<String> {
    let want = text::squash(part);
    let sentences = _sentences(answer);
    for (at, sentence) in sentences.iter().enumerate() {
        for half in _halves_v(sentence) {
            if text::squash(&half) == want {
                if at == 0 {
                    // The move is in the answer's first sentence: it needs no lead-in.
                    return Some(want);
                }
                return Some(text::strip(&sentences[..at + 1].join(" ")).to_string());
            }
        }
    }
    None
}

/// Python: bot/memory/store_replies.py:12432 `_the_move_that_fits(moves, room)`
///
/// The move whose own length is the largest that fits, and the long move it is a step of.
pub fn _the_move_that_fits(moves: &[Move], room: i64) -> (Option<String>, Option<i64>) {
    let (mut best, mut best_sentence, mut best_seconds): (Option<String>, Option<usize>, i64) = (None, None, -1);
    let (mut over, mut over_sentence): (Option<i64>, Option<usize>) = (None, None);
    for (part, took, sentence) in moves {
        let Some(took) = took else { continue };
        if *took > room {
            if over.is_none() {
                over = Some(*took);
                over_sentence = Some(*sentence);
            }
            continue;
        }
        if *took > best_seconds {
            best = Some(part.clone());
            best_sentence = Some(*sentence);
            best_seconds = *took;
        }
    }
    let step = if best.is_some() && over.is_some() && over_sentence < best_sentence { over } else { None };
    (best, step)
}

/// Python: bot/memory/store_replies.py:12458 `_left_out(moves, room)`
///
/// True when the answer has a move of its own that takes longer than the room.
pub fn _left_out(moves: &[Move], room: i64) -> bool {
    moves.iter().any(|(_, took, _)| took.map(|t| t > room).unwrap_or(false))
}

/// Python: bot/memory/store_replies.py:12462 `_the_lead(answer)`
///
/// The first thing an answer says, for a budget too small for any move of it.
pub fn _the_lead(answer: &str) -> Option<String> {
    for sentence in _sentences(answer) {
        let part = LEAD_LABEL.sub("", text::strip_chars(&sentence, " .;,"));
        if _says_something(&part) {
            return Some(part);
        }
    }
    None
}

rx!(A_RATE_OF, ANY_FIGURE);

/// Python: bot/memory/store_replies.py:12479 `_a_rate_of_the_same_length(answer, room)`
///
/// True when the answer names this same length as a regular allowance ("20 minutes a day").
pub fn _a_rate_of_the_same_length(answer: &str, room: i64) -> bool {
    A_RATE_OF.finditer(answer).iter().any(|m| _seconds(&(m.name_str("length").to_string() + " " + m.name_str("unit"))) == Some(room))
}

strs!(BUDGET_FITS, BUDGET_MORE, BUDGET_TIGHT, CALL_LEADS, RESTRICT_OPENS);
static OF_FIRST: LazyLock<Re> = LazyLock::new(|| Re::new(r"^of\b", crate::py::re::I));

/// A budget shape with the length and the move filled in.
fn budget_said(shape: &[String], memory: &Memory, budget: &str, part: &str) -> String {
    fmt_named(&pick(shape, memory, ""), &[("budget", budget), ("part", part)])
}

/// Python: bot/memory/store_replies.py:12488 `_the_budget_on_the_answer(text, memory)`
///
/// The answer above, cut to the part of it that fits the time they say they have, or None.
/// (Contract change: the generated stub returned `()`.)
pub fn _the_budget_on_the_answer(text_: &str, memory: &Memory) -> Option<String> {
    let said = text::squash(text_);
    if said.is_empty() || said.contains('?') || text::split(&said).len() > 11 {
        return None;
    }
    let found = BUDGET.match_(&said)?;
    let tail = text::strip_chars(&said[found.end()..], " .!,;").to_string();
    if text::split(&tail).len() > 4 {
        // "i only get 20 minutes to get to the station" is a question about the journey.
        return None;
    }
    if OF_FIRST.match_(&tail).is_some() {
        // "i have 2 hours of meetings today": the length belongs to the thing after it.
        return None;
    }
    let length = found.name_str("length").to_string();
    let room = _seconds(&length)?;
    let mut longer: Option<String> = None;
    for answer in _recent_answers(memory, 3) {
        if _a_rate_of_the_same_length(&answer, room) {
            // The answer already talks about this length as a regular allowance.
            return None;
        }
        if room < 86400 && A_HORIZON.is_match(&answer) {
            // A day's regime read to a short budget: the answer stands as it was given.
            if let Some(lead) = _the_lead(&answer) {
                return Some(budget_said(&BUDGET_TIGHT, memory, &_theirs(&length, true), &_tidy(&lead)));
            }
            continue;
        }
        if _a_remark_about_a_length(&answer) {
            // A remark about a length is not an answer with a length in it.
            continue;
        }
        if _longest_n(&answer, false).is_none() {
            // A turn with no length in it is not the answer this budget is about.
            continue;
        }
        let moves = _moves(&answer);
        let (part, over) = _the_move_that_fits(&moves, room);
        if over.is_some() {
            // The answer's long move is behind the one that fits, so the fitting move is a step of it.
            if let Some(lead) = _the_lead(&answer) {
                return Some(budget_said(&BUDGET_TIGHT, memory, &_theirs(&length, true), &_tidy(&lead)));
            }
        }
        if let Some(mut part) = part.filter(|p| !p.is_empty()) {
            // Which shape it takes is decided by whether the budget leaves anything out.
            let shape: &[String] = if _left_out(&moves, room) {
                if let Some(shorter) = _up_to_the_move(&answer, &part) {
                    if !shorter.is_empty() && text::split(&shorter).len() <= 90 && _says_something(&shorter) {
                        // A move is handed back with everything the answer said before it.
                        part = shorter;
                    }
                }
                &BUDGET_FITS
            } else {
                // A budget bigger than every move of the answer is not a limit on anything.
                let mut kept: Option<String> = None;
                if ANY_FIGURE.findall(&answer).len() == 1 {
                    if let Some(shorter) = _up_to_the_move(&answer, &part) {
                        if !shorter.is_empty() && text::split(&shorter).len() <= 45 && _says_something(&shorter) {
                            kept = Some(shorter);
                        }
                    }
                }
                if let Some(kept) = kept {
                    part = kept;
                } else if let Some(lead) = _the_lead(&answer) {
                    if text::split(&lead).len() <= 90 && _says_something(&lead) {
                        part = lead;
                    }
                }
                &BUDGET_MORE
            };
            return Some(budget_said(shape, memory, &_theirs(&length, false), &_tidy(&part)));
        }
        if longer.is_none() {
            longer = Some(answer.clone());
        }
    }
    let longer = longer?;
    let part = _the_lead(&longer).filter(|p| !p.is_empty())?;
    Some(budget_said(&BUDGET_TIGHT, memory, &_theirs(&length, true), &_tidy(&part)))
}

/// Python: bot/memory/store_replies.py:12613 `_theirs(length, mid=False)`
///
/// The length as they said it, with a capital and no full stop: "20 minutes".
pub fn _theirs(length: &str, mid: bool) -> String {
    let text_ = text::strip_chars(&text::squash(length), " .;,").to_string();
    if text_.is_empty() {
        return text_;
    }
    let second_upper = text_.chars().nth(1).map(|c| crate::understanding::pyuni::ch_upper(c)).unwrap_or(false);
    if mid && !second_upper {
        return lower_first(&text_);
    }
    text::upper_first(&text_)
}

/// Python: bot/memory/store_replies.py:12622 `_tidy(part)`
///
/// A move of an answer as the first thing a reply says: capital, no trailing full stop.
pub fn _tidy(part: &str) -> String {
    let text_ = text::strip_chars(&text::squash(part), " .;,?!").to_string();
    if text_.is_empty() {
        return text_;
    }
    text::upper_first(&text_)
}

/// Python: bot/memory/store_replies.py:12635 `_clauses(reply)`
///
/// The clauses of an answer, longest-lived punctuation split, in the order given.
pub fn _clauses(reply: &str) -> Vec<String> {
    CLAUSE_SPLIT.split(&text::squash(reply)).iter().map(|p| text::strip_chars(p, " .;").to_string()).filter(|p| !p.is_empty()).collect()
}

/// Python: bot/memory/store_replies.py:12640 `_the_limit(reply)`
///
/// The answer's own restriction - "cut the distance and the speed for a week or two".
pub fn _the_limit(reply: &str) -> Option<String> {
    for part in _clauses(reply) {
        let low = part.to_lowercase();
        if text::split(&part).len() > 24 {
            continue;
        }
        if RESTRICT_OPENS.iter().any(|w| low == *w || low.starts_with(&(w.clone() + " "))) {
            return Some(part);
        }
    }
    None
}

static MEANS_STOP: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:means stop|stop driving|do not drive|don't drive)\b", 0));

/// Python: bot/memory/store_replies.py:12650 `_the_condition(reply)`
///
/// The answer's own stop condition - "see a physio if the pain is sharp" - or None.
pub fn _the_condition(reply: &str) -> Option<String> {
    let clauses = _clauses(reply);
    for part in &clauses {
        let low = part.to_lowercase();
        if text::split(part).len() > 26 {
            continue;
        }
        if STOP_CONDITION.match_(&low).is_some() && low.contains(" if ") {
            return Some(part.clone());
        }
    }
    clauses.into_iter().find(|p| MEANS_STOP.is_match(&p.to_lowercase()))
}

/// Python: bot/memory/store_replies.py:12664 `_recent_answers(memory, back=3)`
///
/// The answers the chat has had, newest first, as far back as "the advice above" reaches.
pub fn _recent_answers(memory: &Memory, back: usize) -> Vec<String> {
    let turns = &memory.turns;
    let mut said: Vec<String> = Vec::new();
    for turn in turns[turns.len().saturating_sub(back + 1)..].iter().rev() {
        let reply = text::strip(&turn.bot);
        if !reply.is_empty() {
            said.push(reply.to_string());
        }
        if said.len() >= back {
            break;
        }
    }
    said
}

/// Python: bot/memory/store_replies.py:12683 `_shares_a_word(low, said)`
///
/// True when the line names something the answer it would quote also talks about.
pub fn _shares_a_word(low: &str, said: &str) -> bool {
    use crate::memory::topics::{keywords, stem};
    let words = |t: &str| -> HashSet<String> { keywords(&t.to_lowercase()).iter().map(|w| stem(w)).collect() };
    let mine = words(low);
    words(said).iter().any(|w| mine.contains(w))
}

static KEEP_OR_GO_ON: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:keep|keep going|carry on|continue|still|go on)\b", 0));

/// Python: bot/memory/store_replies.py:12694 `_the_call_from_the_advice(text, memory)`
///
/// The decision a follow-up asks, answered with the limit the last answer already gave.
/// (Contract change: the generated stub returned `()`.)
pub fn _the_call_from_the_advice(text_: &str, memory: &Memory) -> Option<String> {
    let low = _bare(&crate::memory::relations::_folded(text_, memory));
    if low.is_empty() || text::split(&low).len() > 9 || (" ".to_string() + &low + " ").contains(" you ") {
        return None;
    }
    if CALL_ASK.match_(&low).is_none() || !CARRY_ON.is_match(&low) {
        return None;
    }
    if SEE_SOMEONE.is_match(&low) {
        // "should i see someone about it or wait" is answered by the entry that gave the advice.
        return None;
    }
    // The answer that had the limit in it: the newest one about the same thing as the line.
    let (mut limit, mut condition): (Option<String>, Option<String>) = (None, None);
    for said in _recent_answers(memory, 3) {
        if !_shares_a_word(&low, &said) {
            continue;
        }
        let (found, if_half) = (_the_limit(&said), _the_condition(&said));
        if found.is_some() && if_half.is_some() {
            limit = found;
            condition = if_half;
            break;
        }
    }
    let (Some(limit), Some(condition)) = (limit.filter(|l| !l.is_empty()), condition.filter(|c| !c.is_empty())) else { return None };
    let rest_or_go = KEEP_OR_GO_ON.is_match(&low);
    if STOP_IT.is_match(&low) && rest_or_go {
        // "should i rest it or keep going" is a choice the entry answers better.
        return None;
    }
    if STOP_IT.is_match(&low) {
        // "should i stop running?": the answer is the change rather than stopping.
        return Some("Not all the way: ".to_string() + &lower_first(&limit) + ". And " + &lower_first(&condition) + ".");
    }
    Some(fmt_named(&pick(&CALL_LEADS, memory, ""), &[("limit", &lower_first(&limit)), ("condition", &lower_first(&condition))]))
}

/// Python: bot/memory/store_replies.py:12743 `_trim(text, limit=90)`
pub fn _trim(text_: &str, limit: i64) -> String {
    let said = text::squash(text_);
    if text::len(&said) as i64 <= limit {
        return said;
    }
    text::rstrip(&text::slice(&said, 0, Some((limit - 1) as isize))).to_string() + "\u{2026}"
}

rx!(CLOSE_SUBJECT, CHANGE_SUBJECT, CARRIES_ON_AFTER, ASKS_WHETHER_TO, ASKS_FOR_HELP, HOLDS_FOR_NOW, CHECK_FOLLOWED, BARE_ACKNOWLEDGEMENT, SAID_TO_THE_BOT, A_MOOD_LINE, DECLINE_A_SUBJECT);
strset!(CLOSE_FILLER, CLOSE_FILLER_ALWAYS, SAFETY_MODULES, NOT_A_SUBJECT);
strs!(HELD_THREAD, GRIEF_WORDS, CHECK_PLAIN, CHECK_A_FEELING, CHECK_A_QUESTION, CHECK_NOTHING_SAID, CLOSE_PLAIN, CLOSE_HEAVY, CHANGE_PLAIN, CHANGE_HEAVY);
sval!(INTENSIFIER);

/// Python: bot/memory/store_replies.py:12965 `_reads_as_the_invitation(text, memory)`
///
/// True where the composer reads the line as asking whether the bot saw the thing named.
pub fn _reads_as_the_invitation(text_: &str, memory: &Memory) -> bool {
    // A reading that fails leaves the check in service.
    or_on_raise(|| crate::compose::fallback::reads_as_if_i_saw(text_, memory), false)
}

/// Python: bot/memory/store_replies.py:12980 `_the_module_behind(memory, said)`
///
/// The module that answered this line, or "" when the turns do not say.
pub fn _the_module_behind(memory: &Memory, said: &str) -> String {
    let low = _plain_line(said);
    for turn in memory.turns.iter().rev() {
        if _plain_line(&turn.user) == low {
            return turn.module.clone().unwrap_or_default();
        }
    }
    String::new()
}

/// Python: bot/memory/store_replies.py:12988 `_the_line_before_the_check(memory, line)`
///
/// The last line they said that was not itself a check, or "".
pub fn _the_line_before_the_check(memory: &Memory, line: &str) -> String {
    for turn in recent_turns(memory, 6).iter().rev() {
        let said = text::strip(&turn.user).to_string();
        if said.is_empty() {
            continue;
        }
        if CHECK_FOLLOWED.match_(&_plain_line(&said)).is_some() {
            continue;
        }
        if !line.is_empty() && _plain_line(&said) == line {
            continue;
        }
        return said;
    }
    String::new()
}

/// Python: bot/memory/store_replies.py:13002 `_a_check_choice(options, memory, salt)`
///
/// Pick from a pool without saying the same line twice in a row.
pub fn _a_check_choice(options: &[String], memory: &Memory, salt: usize) -> String {
    let turns = &memory.turns;
    let recent = recent_bot_lines(memory, 4);
    let fresh: Vec<&String> = options.iter().filter(|o| !recent.contains(&text::slice(o, 0, Some(30)))).collect();
    let fresh: Vec<&String> = if fresh.is_empty() { options.iter().collect() } else { fresh };
    fresh[(turns.len() + salt) % fresh.len()].clone()
}

/// Python: bot/memory/store_replies.py:13009 `_a_check_came_before(memory, line)`
///
/// True when the last thing they said was itself a check ("did you get that" twice).
pub fn _a_check_came_before(memory: &Memory, line: &str) -> bool {
    let turns = recent_turns(memory, 6);
    let mut said_before: Vec<String> = turns.iter().map(|t| _plain_line(&t.user)).collect();
    if !said_before.is_empty() && !line.is_empty() && said_before.last().map(String::as_str) == Some(line) && text::strip(&turns[turns.len() - 1].bot).is_empty() {
        // The turn being answered is already recorded, with no reply yet.
        said_before.pop();
    }
    for said in said_before.iter().rev() {
        if said.is_empty() {
            continue;
        }
        return CHECK_FOLLOWED.match_(said).is_some();
    }
    false
}

/// Python: bot/memory/store_replies.py:13028 `_the_followed_check(text, memory)`
///
/// The answer to "did you get that?": the thing they said, said back, or an honest no.
pub fn _the_followed_check(text_: &str, memory: &mut Memory) -> Option<String> {
    let as_said = crate::memory::relations::_folded(text_, memory);
    let low = _plain_line(&as_said);
    if _reads_as_the_invitation(&low, memory) {
        // "did you see it" after they named the match is a conversation about the match.
        return None;
    }
    if CHECK_FOLLOWED.match_(&low).is_none() {
        return None;
    }
    if BARE_ACKNOWLEDGEMENT.match_(&low).is_some() && !text_.contains('?') {
        // "got it" is a nod: answering it by reading their own line back turns it into a receipt.
        return None;
    }
    let previous = _the_line_before_the_check(memory, &low);
    if !previous.is_empty() && SAFETY_MODULES.contains(&_the_module_behind(memory, &previous)) {
        // An emergency is not read back.
        return None;
    }
    // The seed carries the turn count as well as the line.
    let salt = text::len(&low) + 3 * memory.turns.len();
    if !previous.is_empty() {
        let lowered = previous.to_lowercase();
        if FEELING_ALONE.match_(text::rstrip_chars(&lowered, "!.,")).is_some()
            || A_MOOD_LINE.is_match(&lowered)
            || A_PERSON_IN_TROUBLE.is_match(&previous)
            || TREATMENT.is_match(&previous)
            || GRIEF_WORDS.iter().any(|w| lowered.contains(w.as_str()))
        {
            return Some(_a_check_choice(&CHECK_A_FEELING, memory, salt));
        }
        if _is_a_question(&previous) {
            return Some(_a_check_choice(&CHECK_A_QUESTION, memory, salt));
        }
        if SAID_TO_THE_BOT.is_match(&_plain_line(&previous)) {
            // The line they want read back was addressed to the bot.
            return Some(_a_check_choice(&CHECK_A_FEELING, memory, salt));
        }
        let said = crate::memory::thread::restate(&previous);
        if !said.is_empty() && text::split(&said).len() >= 3 {
            // A check is answered with the thing they said, unless this is the second check in a row.
            if !_a_check_came_before(memory, &low) {
                return Some(fmt_named(&_a_check_choice(&CHECK_PLAIN, memory, salt), &[("said", &said)]));
            }
            return Some(_a_check_choice(&CHECK_A_FEELING, memory, salt));
        }
        // They did say something, it is just not a thing to say back.
        return Some(_a_check_choice(&CHECK_A_FEELING, memory, salt));
    }
    let record = crate::memory::topics::current(memory).cloned().unwrap_or(Value::None);
    let kept = if record.is_dict() { sget(&record, "text") } else { String::new() };
    if !kept.is_empty() && !_is_a_question(&kept) {
        // With nothing to read back, the subject the chat is on is still true.
        let said = crate::memory::thread::restate(&kept);
        if !said.is_empty() && text::split(&said).len() >= 2 {
            return Some(fmt_named(&_a_check_choice(&CHECK_PLAIN, memory, salt), &[("said", &said)]));
        }
    }
    Some(_a_check_choice(&CHECK_NOTHING_SAID, memory, salt))
}

static NOT_PLAIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' ]+", 0));

/// Python: bot/memory/store_replies.py:13084 `_plain_line(text)`
///
/// The line in plain lower-case words, with the punctuation taken off.
pub fn _plain_line(text_: &str) -> String {
    let low = fold(text_);
    text::squash(&NOT_PLAIN.sub(" ", &low))
}

/// Python: bot/memory/store_replies.py:13114 `_a_subject_to_leave_alone(low)`
///
/// True when the line is a closing that names its subject rather than saying "it".
pub fn _a_subject_to_leave_alone(low: &str) -> bool {
    for found in DECLINE_A_SUBJECT.finditer(low) {
        let subject = [found.name("subject"), found.name("rather"), found.name("let")].into_iter().flatten().find(|s| !s.is_empty()).unwrap_or("");
        let words = text::split(text::strip(subject));
        if words.is_empty() || words.len() > 4 {
            continue;
        }
        if words.len() > 1 && ["it", "that", "this"].contains(&words[0].as_str()) {
            // The pronoun frame with a word after it is the frames' own to decide.
            continue;
        }
        if words.iter().any(|w| NOT_A_SUBJECT.contains(w)) {
            continue;
        }
        // What is left in front of the frame is filler or nothing.
        if !text::split(&low[..found.start()]).iter().all(|w| CLOSE_FILLER.contains(w)) {
            continue;
        }
        return true;
    }
    false
}

static ASKS_A_WH: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?:what|how|why|when|where|who|which)\b", 0));

/// Python: bot/memory/store_replies.py:13140 `_closes_the_subject(text)`
///
/// True when the whole line asks to leave the thing in the air alone.
pub fn _closes_the_subject(text_: &str) -> bool {
    let low = _plain_line(text_);
    if low.is_empty() || text::split(&low).len() > 12 {
        return false;
    }
    if ASKS_A_WH.match_(&low).is_some() {
        // "how do i forget it" is an ask about the thing, not a request to leave it.
        return false;
    }
    if ASKS_WHETHER_TO.is_match(&low) || ASKS_FOR_HELP.is_match(&low) {
        return false;
    }
    if CARRIES_ON_AFTER.is_match(&HOLDS_FOR_NOW.sub(" ", &low)) {
        // "drop it in the pan" is about the thing, not about leaving it alone.
        return false;
    }
    if _closes_by_a_frame(&low) {
        return true;
    }
    _a_subject_to_leave_alone(&low)
}

/// Python: bot/memory/store_replies.py:13161 `_closes_by_a_frame(low)`
///
/// True when one of the frames above covers the line and nothing else in it carries on.
pub fn _closes_by_a_frame(low: &str) -> bool {
    for pattern in [&*CLOSE_SUBJECT, &*CHANGE_SUBJECT] {
        for found in pattern.finditer(low) {
            let rest = low[..found.start()].to_string() + " " + &low[found.end()..];
            if text::split(&rest).iter().all(|w| CLOSE_FILLER.contains(w)) {
                return true;
            }
        }
    }
    false
}

static INTENSIFIER_RE: LazyLock<Re> = LazyLock::new(|| Re::new(&INTENSIFIER, 0));

/// Python: bot/memory/store_replies.py:13170 `_a_plain_frame_closes(low)`
///
/// True when the frames read the line with no help from this reader's newer allowances.
pub fn _a_plain_frame_closes(low: &str) -> bool {
    for pattern in [&*CLOSE_SUBJECT, &*CHANGE_SUBJECT] {
        for found in pattern.finditer(low) {
            if INTENSIFIER_RE.is_match(found.as_str()) {
                continue;
            }
            let rest = low[..found.start()].to_string() + " " + &low[found.end()..];
            if text::split(&rest).iter().all(|w| CLOSE_FILLER_ALWAYS.contains(w)) {
                return true;
            }
        }
    }
    false
}

/// Python: bot/memory/store_replies.py:13182 `_held_thread(memory)`
///
/// True when a safety answer, rather than a topic, is the thing the chat is carrying.
pub fn _held_thread(memory: &Memory) -> bool {
    for turn in recent_turns(memory, 3) {
        if turn.module.as_deref() == Some("safety") {
            return true;
        }
        let said = turn.bot.to_lowercase();
        if HELD_THREAD.iter().any(|w| said.contains(w.as_str())) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/store_replies.py:13193 `_gentle_subject(memory)`
///
/// True when what was closed is heavy: a loss, an illness, a stay in hospital.
pub fn _gentle_subject(memory: &Memory) -> bool {
    for turn in recent_turns(memory, 4) {
        if matches!(turn.module.as_deref(), Some("loss") | Some("grief")) {
            return true;
        }
        let bot = turn.bot.to_lowercase();
        if GRIEF_WORDS.iter().any(|w| bot.contains(w.as_str())) {
            return true;
        }
        let user_line = turn.user.to_lowercase();
        if GRIEF_WORDS.iter().any(|w| user_line.contains(w.as_str())) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/store_replies.py:13205 `_can_close(text, memory)`
///
/// Whether the store is the one to answer this closing line.
pub fn _can_close(text_: &str, memory: &Memory) -> bool {
    if !_closes_the_subject(text_) {
        return false;
    }
    let current = crate::memory::topics::current(memory).map(Value::truthy).unwrap_or(false);
    if !current && recent_turns(memory, 2).is_empty() {
        return false;
    }
    if _held_thread(memory) {
        return false;
    }
    if _gentle_subject(memory) && !_a_plain_frame_closes(&_plain_line(text_)) {
        // A loss or a stay in hospital is answered better by the grief reader.
        return false;
    }
    true
}

/// Python: bot/memory/store_replies.py:13226 `_the_close(text, memory)`
///
/// The answer to a person who has asked to leave the subject: a short yes, and no advice.
pub fn _the_close(text_: &str, memory: &mut Memory) -> Option<String> {
    if !_can_close(text_, memory) {
        return None;
    }
    let low = _plain_line(text_);
    let gentle = _gentle_subject(memory);
    if CHANGE_SUBJECT.is_match(&low) {
        return Some(pick(if gentle { &CHANGE_HEAVY } else { &CHANGE_PLAIN }, memory, &low));
    }
    Some(pick(if gentle { &CLOSE_HEAVY } else { &CLOSE_PLAIN }, memory, &low))
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Python's `"%g" % x` and float `//`, checked against CPython's own output.
    #[test]
    fn python_number_formats() {
        assert_eq!(fmt_g(500.0), "500");
        assert_eq!(fmt_g(0.5), "0.5");
        assert_eq!(fmt_g(1e-5), "1e-05");
        assert_eq!(fmt_g(1234567.0), "1.23457e+06");
        assert_eq!(fmt_g(453.59), "453.59");
        assert_eq!(py_floordiv(1.0, 0.1), 9.0);
        assert_eq!(py_floordiv(7.5, 2.0), 3.0);
        assert_eq!(py_floordiv(2000.0, 500.0), 4.0);
    }

    /// Python `int()` on str: whitespace, signs, underscores and any Unicode decimal digit.
    #[test]
    fn python_int_parsing() {
        assert_eq!(py_int(" 42 "), Some(42));
        assert_eq!(py_int("-7"), Some(-7));
        assert_eq!(py_int("1_000"), Some(1000));
        assert_eq!(py_int("_1"), None);
        assert_eq!(py_int("\u{0663}"), Some(3));
        assert_eq!(py_int("4.5"), None);
        assert_eq!(py_int(""), None);
    }

    /// `str.format` with named fields and doubled braces.
    #[test]
    fn python_str_format() {
        assert_eq!(fmt_named("{Thing}, not {old} {{x}}", &[("Thing", "Nuts"), ("old", "eggs")]), "Nuts, not eggs {x}");
    }
}
