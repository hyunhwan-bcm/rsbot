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
    crate::compose::pick(&owned, &memory_value(memory), seed_text)
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
    if crate::memory::history::asked(text_, memory.as_deref()).val().truthy() {
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
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or(fallback)
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
        let found = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| crate::memory::facts::extract(&said, false).val())) {
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

/// Python: bot/memory/store_replies.py:4256 `_swapped_lines(memory, fixed)` (never called on the trace corpus)
pub fn _swapped_lines(memory: &Memory, fixed: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_swapped_lines")
}

/// Python: bot/memory/store_replies.py:4308 `_the_time_that_moved(told, fixed, value)` (never called on the trace corpus)
pub fn _the_time_that_moved(told: &Value, fixed: &Value, value: &Value) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_the_time_that_moved")
}

/// Python: bot/memory/store_replies.py:4341 `_a_rate_changed(text, memory)` (never called on the trace corpus)
pub fn _a_rate_changed(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_a_rate_changed")
}

/// Python: bot/memory/store_replies.py:4348 `_answer_the_new_one(memory, fixed)` (never called on the trace corpus)
pub fn _answer_the_new_one(memory: &mut Memory, fixed: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_answer_the_new_one")
}

/// Python: bot/memory/store_replies.py:4386 `_name_fixed_this_turn(text, memory)`
pub fn _name_fixed_this_turn(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_name_fixed_this_turn")
}

/// Python: bot/memory/store_replies.py:4410 `_a_number_word(number)` (never called on the trace corpus)
pub fn _a_number_word(number: i64) -> String {
    crate::unported!("bot.memory.store_replies:_a_number_word")
}

/// Python: bot/memory/store_replies.py:4429 `_fact_moved_this_turn(text, memory)`
pub fn _fact_moved_this_turn(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_fact_moved_this_turn")
}

/// Python: bot/memory/store_replies.py:4453 `_fact_fixed_this_turn(text, memory)`
pub fn _fact_fixed_this_turn(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_fact_fixed_this_turn")
}

/// Python: bot/memory/store_replies.py:4479 `_respond(text, memory)`
pub fn _respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_respond")
}

/// Python: bot/memory/store_replies.py:5073 `_a_place_new_to_the_chat(memory, low)` (never called on the trace corpus)
pub fn _a_place_new_to_the_chat(memory: &Memory, low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_place_new_to_the_chat")
}

/// Python: bot/memory/store_replies.py:5095 `_about_what_they_said(text, memory)`
pub fn _about_what_they_said(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_about_what_they_said")
}

/// Python: bot/memory/store_replies.py:5154 `_the_job_was_just_said(memory)` (never called on the trace corpus)
pub fn _the_job_was_just_said(memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_the_job_was_just_said")
}

/// Python: bot/memory/store_replies.py:5185 `_nothing_but_the_job(said, job, memory)` (never called on the trace corpus)
pub fn _nothing_but_the_job(said: &str, job: &Value, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_nothing_but_the_job")
}

/// Python: bot/memory/store_replies.py:5204 `_the_allergy_is_what_is_asked(text, memory)` (never called on the trace corpus)
pub fn _the_allergy_is_what_is_asked(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_the_allergy_is_what_is_asked")
}

/// Python: bot/memory/store_replies.py:5229 `_general_recall(text, memory)`
pub fn _general_recall(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_general_recall")
}

/// Python: bot/memory/store_replies.py:5312 `_pet_names(family)` (never called on the trace corpus)
pub fn _pet_names(family: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_pet_names")
}

/// Python: bot/memory/store_replies.py:5326 `_points_at_something_else(low)` (never called on the trace corpus)
pub fn _points_at_something_else(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_points_at_something_else")
}

/// Python: bot/memory/store_replies.py:5340 `_the_pet_in_play(low, memory)` (never called on the trace corpus)
pub fn _the_pet_in_play(low: &str, memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_the_pet_in_play")
}

/// Python: bot/memory/store_replies.py:5355 `_a_pet_of_theirs_is_named(low, family, memory)` (never called on the trace corpus)
pub fn _a_pet_of_theirs_is_named(low: &str, family: &Value, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_a_pet_of_theirs_is_named")
}

/// Python: bot/memory/store_replies.py:5365 `_a_name_for_the_pet(text, memory, low)`
pub fn _a_name_for_the_pet(text: &str, memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_a_name_for_the_pet")
}

/// Python: bot/memory/store_replies.py:5387 `_count_of_a_kind(pets)` (never called on the trace corpus)
pub fn _count_of_a_kind(pets: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_count_of_a_kind")
}

/// Python: bot/memory/store_replies.py:5400 `_return_reply(text, memory)`
pub fn _return_reply(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.store_replies:_return_reply")
}

/// Python: bot/memory/store_replies.py:5416 `_same_case(thing, text)` (never called on the trace corpus)
pub fn _same_case(thing: &str, text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_same_case")
}

/// Python: bot/memory/store_replies.py:5423 `_name_reply(memory, low='')` (never called on the trace corpus)
pub fn _name_reply(memory: &mut Memory, low: &str) -> String {
    crate::unported!("bot.memory.store_replies:_name_reply")
}

/// Python: bot/memory/store_replies.py:5439 `_the_day_of(plan)` (never called on the trace corpus)
pub fn _the_day_of(plan: &str) -> String {
    crate::unported!("bot.memory.store_replies:_the_day_of")
}

/// Python: bot/memory/store_replies.py:5455 `_at_the_clock(plan)` (never called on the trace corpus)
pub fn _at_the_clock(plan: &str) -> String {
    crate::unported!("bot.memory.store_replies:_at_the_clock")
}

/// Python: bot/memory/store_replies.py:5478 `_the_day_before(plan)` (never called on the trace corpus)
pub fn _the_day_before(plan: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_day_before")
}

/// Python: bot/memory/store_replies.py:5494 `_the_prep_day(memory, low)`
pub fn _the_prep_day(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_the_prep_day")
}

/// Python: bot/memory/store_replies.py:5526 `_a_plan_answers(memory, low)` (never called on the trace corpus)
pub fn _a_plan_answers(memory: &mut Memory, low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_plan_answers")
}

/// Python: bot/memory/store_replies.py:5545 `_the_date_behind_that(memory, low, plans)`
pub fn _the_date_behind_that(memory: &Memory, low: &str, plans: &[String]) {
    crate::unported!("bot.memory.store_replies:_the_date_behind_that")
}

/// Python: bot/memory/store_replies.py:5568 `_the_whole_word_in(text, word)` (never called on the trace corpus)
pub fn _the_whole_word_in(text: &str, word: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_whole_word_in")
}

/// Python: bot/memory/store_replies.py:5582 `_third_person(word)` (never called on the trace corpus)
pub fn _third_person(word: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_third_person")
}

/// Python: bot/memory/store_replies.py:5596 `_the_line_that_said(memory, plan)` (never called on the trace corpus)
pub fn _the_line_that_said(memory: &Memory, plan: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_line_that_said")
}

/// Python: bot/memory/store_replies.py:5610 `_the_verb_from_the_line(plan, line)` (never called on the trace corpus)
pub fn _the_verb_from_the_line(plan: &str, line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_verb_from_the_line")
}

/// Python: bot/memory/store_replies.py:5647 `_the_verb_they_asked(plan, low, line='')` (never called on the trace corpus)
pub fn _the_verb_they_asked(plan: &str, low: &str, line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_verb_they_asked")
}

/// Python: bot/memory/store_replies.py:5702 `_plans_reply(memory, low)`
pub fn _plans_reply(memory: &mut Memory, low: &str) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_plans_reply")
}

/// Python: bot/memory/store_replies.py:5871 `_their_place(low)`
pub fn _their_place(low: &str) {
    crate::unported!("bot.memory.store_replies:_their_place")
}

/// Python: bot/memory/store_replies.py:5881 `_naming_words(text)` (never called on the trace corpus)
pub fn _naming_words(text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_naming_words")
}

/// Python: bot/memory/store_replies.py:5886 `_the_day_is_the_whole_date(plan)` (never called on the trace corpus)
pub fn _the_day_is_the_whole_date(plan: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_the_day_is_the_whole_date")
}

/// Python: bot/memory/store_replies.py:5898 `_the_plan_was_just_said_back(memory, plan)` (never called on the trace corpus)
pub fn _the_plan_was_just_said_back(memory: &Memory, plan: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_the_plan_was_just_said_back")
}

/// Python: bot/memory/store_replies.py:5912 `_the_answer_about_it_is_on_the_screen(memory, plan, text)` (never called on the trace corpus)
pub fn _the_answer_about_it_is_on_the_screen(memory: &Memory, plan: &str, text: &str) -> Option<bool> {
    crate::unported!("bot.memory.store_replies:_the_answer_about_it_is_on_the_screen")
}

/// Python: bot/memory/store_replies.py:5966 `_a_time_for_the_job_itself(low)` (never called on the trace corpus)
pub fn _a_time_for_the_job_itself(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_time_for_the_job_itself")
}

/// Python: bot/memory/store_replies.py:5977 `_thing_is_a_quality(low)` (never called on the trace corpus)
pub fn _thing_is_a_quality(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_thing_is_a_quality")
}

/// Python: bot/memory/store_replies.py:5992 `_names_something_of_its_own(low, start)` (never called on the trace corpus)
pub fn _names_something_of_its_own(low: &str, start: i64) -> bool {
    crate::unported!("bot.memory.store_replies:_names_something_of_its_own")
}

/// Python: bot/memory/store_replies.py:6001 `_when_thing(low, wide=False)`
pub fn _when_thing(low: &str, wide: bool) -> String {
    crate::unported!("bot.memory.store_replies:_when_thing")
}

/// Python: bot/memory/store_replies.py:6034 `_the_one_asked(matching, rest)`
pub fn _the_one_asked(matching: &[String], rest: &str) -> String {
    crate::unported!("bot.memory.store_replies:_the_one_asked")
}

/// Python: bot/memory/store_replies.py:6050 `_ask_verb(low)`
pub fn _ask_verb(low: &str) -> String {
    crate::unported!("bot.memory.store_replies:_ask_verb")
}

/// Python: bot/memory/store_replies.py:6055 `_asks_the_end(low)`
pub fn _asks_the_end(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_asks_the_end")
}

/// Python: bot/memory/store_replies.py:6060 `_asks_the_start(low)`
pub fn _asks_the_start(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_asks_the_start")
}

/// Python: bot/memory/store_replies.py:6065 `_says_the_end(plan)` (never called on the trace corpus)
pub fn _says_the_end(plan: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_says_the_end")
}

/// Python: bot/memory/store_replies.py:6070 `_answers_the_end_asked(plan, low)` (never called on the trace corpus)
pub fn _answers_the_end_asked(plan: &str, low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_answers_the_end_asked")
}

/// Python: bot/memory/store_replies.py:6082 `_names_the_thing(plan, thing, memory=None)` (never called on the trace corpus)
pub fn _names_the_thing(plan: &str, thing: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_names_the_thing")
}

/// Python: bot/memory/store_replies.py:6099 `_says_the_verb(plan, verb)` (never called on the trace corpus)
pub fn _says_the_verb(plan: &str, verb: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_says_the_verb")
}

/// Python: bot/memory/store_replies.py:6125 `_job_reply(memory, where=False)` (never called on the trace corpus)
pub fn _job_reply(memory: &mut Memory, r#where: bool) -> String {
    crate::unported!("bot.memory.store_replies:_job_reply")
}

/// Python: bot/memory/store_replies.py:6147 `_yours(value)` (never called on the trace corpus)
pub fn _yours(value: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_yours")
}

/// Python: bot/memory/store_replies.py:6161 `_a_circumstance(job)` (never called on the trace corpus)
pub fn _a_circumstance(job: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_a_circumstance")
}

/// Python: bot/memory/store_replies.py:6177 `_job_said(known)` (never called on the trace corpus)
pub fn _job_said(known: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_job_said")
}

/// Python: bot/memory/store_replies.py:6217 `_place_phrase(place)` (never called on the trace corpus)
pub fn _place_phrase(place: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_place_phrase")
}

/// Python: bot/memory/store_replies.py:6232 `_job_phrase(known)` (never called on the trace corpus)
pub fn _job_phrase(known: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_job_phrase")
}

/// Python: bot/memory/store_replies.py:6246 `_places_reply(memory, low)` (never called on the trace corpus)
pub fn _places_reply(memory: &mut Memory, low: &str) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_places_reply")
}

/// Python: bot/memory/store_replies.py:6321 `_grown_up_reply(memory)` (never called on the trace corpus)
pub fn _grown_up_reply(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_grown_up_reply")
}

/// Python: bot/memory/store_replies.py:6334 `_from_reply(memory)` (never called on the trace corpus)
pub fn _from_reply(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_from_reply")
}

/// Python: bot/memory/store_replies.py:6351 `_is_are(who)` (never called on the trace corpus)
pub fn _is_are(who: &str) -> String {
    crate::unported!("bot.memory.store_replies:_is_are")
}

/// Python: bot/memory/store_replies.py:6354 `_eats(who)` (never called on the trace corpus)
pub fn _eats(who: &str) -> String {
    crate::unported!("bot.memory.store_replies:_eats")
}

/// Python: bot/memory/store_replies.py:6357 `_does_not(who)` (never called on the trace corpus)
pub fn _does_not(who: &str) -> String {
    crate::unported!("bot.memory.store_replies:_does_not")
}

/// Python: bot/memory/store_replies.py:6360 `_they(who)` (never called on the trace corpus)
pub fn _they(who: &str) -> String {
    crate::unported!("bot.memory.store_replies:_they")
}

/// Python: bot/memory/store_replies.py:6369 `_a_diet_predicate(who, diet)` (never called on the trace corpus)
pub fn _a_diet_predicate(who: &str, diet: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_a_diet_predicate")
}

/// Python: bot/memory/store_replies.py:6385 `_their_pronoun(who)` (never called on the trace corpus)
pub fn _their_pronoun(who: &str) -> String {
    crate::unported!("bot.memory.store_replies:_their_pronoun")
}

/// Python: bot/memory/store_replies.py:6400 `_no_diet(who)` (never called on the trace corpus)
pub fn _no_diet(who: &str) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_no_diet")
}

/// Python: bot/memory/store_replies.py:6414 `_a_condition_rules_out(condition)` (never called on the trace corpus)
pub fn _a_condition_rules_out(condition: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_a_condition_rules_out")
}

/// Python: bot/memory/store_replies.py:6423 `_both_food_facts(who, allergies, diet)` (never called on the trace corpus)
pub fn _both_food_facts(who: &str, allergies: &Value, diet: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_both_food_facts")
}

/// Python: bot/memory/store_replies.py:6441 `_allergy_is_their_food(who, allergies, only=True, low='')` (never called on the trace corpus)
pub fn _allergy_is_their_food(who: &str, allergies: &Value, only: &Value, low: &str) -> String {
    crate::unported!("bot.memory.store_replies:_allergy_is_their_food")
}

/// Python: bot/memory/store_replies.py:6467 `_what_the_food_has(low, ruled)` (never called on the trace corpus)
pub fn _what_the_food_has(low: &str, ruled: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_what_the_food_has")
}

/// Python: bot/memory/store_replies.py:6491 `_diet_is_their_food(who, diet, low='', plain=False)` (never called on the trace corpus)
pub fn _diet_is_their_food(who: &str, diet: &Value, low: &str, plain: &Value) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_diet_is_their_food")
}

/// Python: bot/memory/store_replies.py:6536 `_last_answer(memory)`
pub fn _last_answer(memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_last_answer")
}

/// Python: bot/memory/store_replies.py:6547 `_one_half_of_the_dish(food, taste)` (never called on the trace corpus)
pub fn _one_half_of_the_dish(food: &Value, taste: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_one_half_of_the_dish")
}

/// Python: bot/memory/store_replies.py:6567 `_dislikes_here(text)`
pub fn _dislikes_here(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.store_replies:_dislikes_here")
}

/// Python: bot/memory/store_replies.py:6581 `_a_dislike_told(text, memory)`
pub fn _a_dislike_told(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.store_replies:_a_dislike_told")
}

/// Python: bot/memory/store_replies.py:6641 `_the_take_back(memory, key)`
pub fn _the_take_back(memory: &Memory, key: &str) -> Vec<String> {
    crate::unported!("bot.memory.store_replies:_the_take_back")
}

/// Python: bot/memory/store_replies.py:6662 `_a_take_back_said_back(line, gone)` (never called on the trace corpus)
pub fn _a_take_back_said_back(line: &str, gone: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_a_take_back_said_back")
}

/// Python: bot/memory/store_replies.py:6687 `_a_negation_said_back(gone, line='')` (never called on the trace corpus)
pub fn _a_negation_said_back(gone: &Value, line: &str) -> String {
    crate::unported!("bot.memory.store_replies:_a_negation_said_back")
}

/// Python: bot/memory/store_replies.py:6701 `_says_the_fact(line, key)` (never called on the trace corpus)
pub fn _says_the_fact(line: &str, key: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_says_the_fact")
}

/// Python: bot/memory/store_replies.py:6736 `_the_line_about(memory, key, own=None)`
pub fn _the_line_about(memory: &Memory, key: &str, own: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_the_line_about")
}

/// Python: bot/memory/store_replies.py:6771 `_said_of_someone_else(line, wanted)` (never called on the trace corpus)
pub fn _said_of_someone_else(line: &str, wanted: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_said_of_someone_else")
}

/// Python: bot/memory/store_replies.py:6786 `_deny(memory, key, denial, own=None)`
pub fn _deny(memory: &Memory, key: &str, denial: &str, own: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_deny")
}

/// Python: bot/memory/store_replies.py:6793 `_their_own_diet(line)` (never called on the trace corpus)
pub fn _their_own_diet(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_their_own_diet")
}

/// Python: bot/memory/store_replies.py:6817 `_diet_words(value)` (never called on the trace corpus)
pub fn _diet_words(value: &str) -> String {
    crate::unported!("bot.memory.store_replies:_diet_words")
}

/// Python: bot/memory/store_replies.py:6823 `_a_diet_put_back(shown)` (never called on the trace corpus)
pub fn _a_diet_put_back(shown: &str) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_a_diet_put_back")
}

/// Python: bot/memory/store_replies.py:6847 `_diet_reply(memory, low='')`
pub fn _diet_reply(memory: &mut Memory, low: &str) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_diet_reply")
}

/// Python: bot/memory/store_replies.py:6970 `_about_people(low)` (never called on the trace corpus)
pub fn _about_people(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_about_people")
}

/// Python: bot/memory/store_replies.py:6982 `_age_compare(memory, low)`
pub fn _age_compare(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_age_compare")
}

/// Python: bot/memory/store_replies.py:7091 `_the_day_a_plan_names(plan)` (never called on the trace corpus)
pub fn _the_day_a_plan_names(plan: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_day_a_plan_names")
}

/// Python: bot/memory/store_replies.py:7128 `_the_plans_in_date_order(memory)` (never called on the trace corpus)
pub fn _the_plans_in_date_order(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_the_plans_in_date_order")
}

/// Python: bot/memory/store_replies.py:7175 `_which_comes_first(memory, low)`
pub fn _which_comes_first(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_which_comes_first")
}

/// Python: bot/memory/store_replies.py:7201 `_the_pronoun_is_one_of_the_two(by_who, low)` (never called on the trace corpus)
pub fn _the_pronoun_is_one_of_the_two(by_who: &Value, low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_the_pronoun_is_one_of_the_two")
}

/// Python: bot/memory/store_replies.py:7217 `_pointed_at(aged, low)` (never called on the trace corpus)
pub fn _pointed_at(aged: &Value, low: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_pointed_at")
}

/// Python: bot/memory/store_replies.py:7249 `_a_food(value)` (never called on the trace corpus)
pub fn _a_food(value: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_a_food")
}

/// Python: bot/memory/store_replies.py:7262 `_diet_food(diet)` (never called on the trace corpus)
pub fn _diet_food(diet: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_diet_food")
}

/// Python: bot/memory/store_replies.py:7274 `_said_a_limit(memory)` (never called on the trace corpus)
pub fn _said_a_limit(memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_said_a_limit")
}

/// Python: bot/memory/store_replies.py:7284 `_said_list(items)` (never called on the trace corpus)
pub fn _said_list(items: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_said_list")
}

/// Python: bot/memory/store_replies.py:7299 `_limit_reply(memory, low='')` (never called on the trace corpus)
pub fn _limit_reply(memory: &mut Memory, low: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_limit_reply")
}

/// Python: bot/memory/store_replies.py:7386 `_the_answer_the_ask_gets(text, memory)`
pub fn _the_answer_the_ask_gets(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_the_answer_the_ask_gets")
}

/// Python: bot/memory/store_replies.py:7407 `_the_ask_beats_the_receipt(text, memory)`
pub fn _the_ask_beats_the_receipt(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_the_ask_beats_the_receipt")
}

/// Python: bot/memory/store_replies.py:7421 `_a_diet_told(text, memory, low)`
pub fn _a_diet_told(text: &str, memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_a_diet_told")
}

/// Python: bot/memory/store_replies.py:7451 `_object_pronoun(who, known=None)` (never called on the trace corpus)
pub fn _object_pronoun(who: &str, known: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_object_pronoun")
}

/// Python: bot/memory/store_replies.py:7456 `_who_the_pronoun_means(low, memory, pronoun)` (never called on the trace corpus)
pub fn _who_the_pronoun_means(low: &str, memory: &mut Memory, pronoun: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_who_the_pronoun_means")
}

/// Python: bot/memory/store_replies.py:7498 `_a_relations_name_reply(memory, low, text='')`
pub fn _a_relations_name_reply(memory: &mut Memory, low: &str, text: &str) {
    crate::unported!("bot.memory.store_replies:_a_relations_name_reply")
}

/// Python: bot/memory/store_replies.py:7613 `_the_group(who)` (never called on the trace corpus)
pub fn _the_group(who: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_group")
}

/// Python: bot/memory/store_replies.py:7621 `_in_group(who, group)` (never called on the trace corpus)
pub fn _in_group(who: &str, group: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_in_group")
}

/// Python: bot/memory/store_replies.py:7627 `_the_people(entries)` (never called on the trace corpus)
pub fn _the_people(entries: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_the_people")
}

/// Python: bot/memory/store_replies.py:7648 `_which_one_of_them(memory, who)` (never called on the trace corpus)
pub fn _which_one_of_them(memory: &mut Memory, who: &str) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_which_one_of_them")
}

/// Python: bot/memory/store_replies.py:7670 `_what_the_group_is_called(group, memory)` (never called on the trace corpus)
pub fn _what_the_group_is_called(group: &Value, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_what_the_group_is_called")
}

/// Python: bot/memory/store_replies.py:7711 `_everyone_they_mean(memory)` (never called on the trace corpus)
pub fn _everyone_they_mean(memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_everyone_they_mean")
}

/// Python: bot/memory/store_replies.py:7723 `_who_is_the_name_reply(memory, low)`
pub fn _who_is_the_name_reply(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_who_is_the_name_reply")
}

/// Python: bot/memory/store_replies.py:7814 `_kids_reply(memory, low)`
pub fn _kids_reply(memory: &mut Memory, low: &str) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_kids_reply")
}

/// Python: bot/memory/store_replies.py:7859 `_and_list(words)` (never called on the trace corpus)
pub fn _and_list(words: &[String]) -> String {
    crate::unported!("bot.memory.store_replies:_and_list")
}

/// Python: bot/memory/store_replies.py:7866 `_lines_a_person_moved(memory)` (never called on the trace corpus)
pub fn _lines_a_person_moved(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_lines_a_person_moved")
}

/// Python: bot/memory/store_replies.py:7897 `_only_a_pointer(flat)` (never called on the trace corpus)
pub fn _only_a_pointer(flat: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_only_a_pointer")
}

/// Python: bot/memory/store_replies.py:7916 `_a_known_person(memory, who)` (never called on the trace corpus)
pub fn _a_known_person(memory: &mut Memory, who: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_a_known_person")
}

/// Python: bot/memory/store_replies.py:7990 `_only_the_name(line, named, who)` (never called on the trace corpus)
pub fn _only_the_name(line: &str, named: &Value, who: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_only_the_name")
}

/// Python: bot/memory/store_replies.py:8009 `_a_name_in_a_pronoun_line(line)` (never called on the trace corpus)
pub fn _a_name_in_a_pronoun_line(line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_a_name_in_a_pronoun_line")
}

/// Python: bot/memory/store_replies.py:8027 `_named_back(who, line)` (never called on the trace corpus)
pub fn _named_back(who: &str, line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_named_back")
}

/// Python: bot/memory/store_replies.py:8056 `_a_relation_in(line)` (never called on the trace corpus)
pub fn _a_relation_in(line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_a_relation_in")
}

/// Python: bot/memory/store_replies.py:8076 `_person_clauses(who, entry)` (never called on the trace corpus)
pub fn _person_clauses(who: &str, entry: &'static Entry) -> Value {
    crate::unported!("bot.memory.store_replies:_person_clauses")
}

/// Python: bot/memory/store_replies.py:8103 `_date_written_as_a_date(value)` (never called on the trace corpus)
pub fn _date_written_as_a_date(value: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_date_written_as_a_date")
}

/// Python: bot/memory/store_replies.py:8115 `_pronoun_person(text, memory)` (never called on the trace corpus)
pub fn _pronoun_person(text: &str, memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_pronoun_person")
}

/// Python: bot/memory/store_replies.py:8134 `_a_line_about(memory, who)` (never called on the trace corpus)
pub fn _a_line_about(memory: &Memory, who: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_a_line_about")
}

/// Python: bot/memory/store_replies.py:8173 `_a_that_fits(name)` (never called on the trace corpus)
pub fn _a_that_fits(name: &str) -> String {
    crate::unported!("bot.memory.store_replies:_a_that_fits")
}

/// Python: bot/memory/store_replies.py:8184 `_a_thing_i_own_reply(memory, low)`
pub fn _a_thing_i_own_reply(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_a_thing_i_own_reply")
}

/// Python: bot/memory/store_replies.py:8228 `_about_a_person_reply(memory, low, text='')`
pub fn _about_a_person_reply(memory: &mut Memory, low: &str, text: &str) {
    crate::unported!("bot.memory.store_replies:_about_a_person_reply")
}

/// Python: bot/memory/store_replies.py:8347 `_about_the_children_reply(memory, low)` (never called on the trace corpus)
pub fn _about_the_children_reply(memory: &mut Memory, low: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_about_the_children_reply")
}

/// Python: bot/memory/store_replies.py:8430 `_a_list_of_clauses(clauses)` (never called on the trace corpus)
pub fn _a_list_of_clauses(clauses: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_a_list_of_clauses")
}

/// Python: bot/memory/store_replies.py:8437 `_the_person_called(memory, line)`
pub fn _the_person_called(memory: &mut Memory, line: &str) -> String {
    crate::unported!("bot.memory.store_replies:_the_person_called")
}

/// Python: bot/memory/store_replies.py:8454 `_covers(said, clauses)` (never called on the trace corpus)
pub fn _covers(said: &str, clauses: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_covers")
}

/// Python: bot/memory/store_replies.py:8463 `_tokens(text)` (never called on the trace corpus)
pub fn _tokens(text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_tokens")
}

/// Python: bot/memory/store_replies.py:8473 `_said_everything(answer, memory)` (never called on the trace corpus)
pub fn _said_everything(answer: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_said_everything")
}

/// Python: bot/memory/store_replies.py:8490 `_a_pregnancy_of_their_own(text, memory, low)`
pub fn _a_pregnancy_of_their_own(text: &str, memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_a_pregnancy_of_their_own")
}

/// Python: bot/memory/store_replies.py:8515 `_a_state_told(text, memory, low)`
pub fn _a_state_told(text: &str, memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_a_state_told")
}

/// Python: bot/memory/store_replies.py:8560 `_a_state_that_was(text, memory, low)`
pub fn _a_state_that_was(text: &str, memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_a_state_that_was")
}

/// Python: bot/memory/store_replies.py:8583 `_pet_part_reply(memory, low)`
pub fn _pet_part_reply(memory: &mut Memory, low: &str) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_pet_part_reply")
}

/// Python: bot/memory/store_replies.py:8593 `_pet_breed_reply(memory, low)`
pub fn _pet_breed_reply(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_pet_breed_reply")
}

/// Python: bot/memory/store_replies.py:8637 `_a_number(value)` (never called on the trace corpus)
pub fn _a_number(value: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_a_number")
}

/// Python: bot/memory/store_replies.py:8644 `_a_pronoun_age(memory, low)` (never called on the trace corpus)
pub fn _a_pronoun_age(memory: &mut Memory, low: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_a_pronoun_age")
}

/// Python: bot/memory/store_replies.py:8710 `_a_pronoun_word(text)` (never called on the trace corpus)
pub fn _a_pronoun_word(text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_a_pronoun_word")
}

/// Python: bot/memory/store_replies.py:8720 `_someone_else_named(low, memory)`
pub fn _someone_else_named(low: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.memory.store_replies:_someone_else_named")
}

/// Python: bot/memory/store_replies.py:8776 `_the_same_fact_about_someone(text, memory)`
pub fn _the_same_fact_about_someone(text: &str, memory: &mut Memory) -> Option<(String, Value)> {
    crate::unported!("bot.memory.store_replies:_the_same_fact_about_someone")
}

/// Python: bot/memory/store_replies.py:8839 `_the_fact_said_against(kind, value, rest)` (never called on the trace corpus)
pub fn _the_fact_said_against(kind: &str, value: &Value, rest: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_the_fact_said_against")
}

/// Python: bot/memory/store_replies.py:8857 `_the_person_last_given_a_fact(memory)` (never called on the trace corpus)
pub fn _the_person_last_given_a_fact(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_the_person_last_given_a_fact")
}

/// Python: bot/memory/store_replies.py:8867 `_take_the_fact_back(memory, who, kind)` (never called on the trace corpus)
pub fn _take_the_fact_back(memory: &mut Memory, who: &str, kind: &str) {
    crate::unported!("bot.memory.store_replies:_take_the_fact_back")
}

/// Python: bot/memory/store_replies.py:8880 `_ive_taken_it_back(who, kind, value)` (never called on the trace corpus)
pub fn _ive_taken_it_back(who: &str, kind: &str, value: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_ive_taken_it_back")
}

/// Python: bot/memory/store_replies.py:8898 `_the_fact_on_record(memory)` (never called on the trace corpus)
pub fn _the_fact_on_record(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_the_fact_on_record")
}

/// Python: bot/memory/store_replies.py:8912 `_both_of_you(who)` (never called on the trace corpus)
pub fn _both_of_you(who: &str) -> String {
    crate::unported!("bot.memory.store_replies:_both_of_you")
}

/// Python: bot/memory/store_replies.py:8916 `_this_too(who, kind, value)` (never called on the trace corpus)
pub fn _this_too(who: &str, kind: &str, value: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_this_too")
}

/// Python: bot/memory/store_replies.py:8927 `_isnt_this_too(who, kind, value)` (never called on the trace corpus)
pub fn _isnt_this_too(who: &str, kind: &str, value: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_isnt_this_too")
}

/// Python: bot/memory/store_replies.py:8941 `_the_person_still_in_hand(low, memory)`
pub fn _the_person_still_in_hand(low: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.memory.store_replies:_the_person_still_in_hand")
}

/// Python: bot/memory/store_replies.py:8954 `_a_food_word(low, entry)` (never called on the trace corpus)
pub fn _a_food_word(low: &str, entry: &'static Entry) -> bool {
    crate::unported!("bot.memory.store_replies:_a_food_word")
}

/// Python: bot/memory/store_replies.py:8969 `_other_person_reply(memory, low)`
pub fn _other_person_reply(memory: &mut Memory, low: &str) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_other_person_reply")
}

/// Python: bot/memory/store_replies.py:9118 `_asks_for_a_limit_not_advice(low)`
pub fn _asks_for_a_limit_not_advice(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_asks_for_a_limit_not_advice")
}

/// Python: bot/memory/store_replies.py:9139 `_the_pronoun_in(low)`
pub fn _the_pronoun_in(low: &str) -> String {
    crate::unported!("bot.memory.store_replies:_the_pronoun_in")
}

/// Python: bot/memory/store_replies.py:9153 `_nobody_else_kept(memory)`
pub fn _nobody_else_kept(memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_nobody_else_kept")
}

/// Python: bot/memory/store_replies.py:9171 `_a_bare_limit_ask(low, memory)`
pub fn _a_bare_limit_ask(low: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_a_bare_limit_ask")
}

/// Python: bot/memory/store_replies.py:9191 `_nothing_kept_about_them(low)` (never called on the trace corpus)
pub fn _nothing_kept_about_them(low: &str) -> String {
    crate::unported!("bot.memory.store_replies:_nothing_kept_about_them")
}

/// Python: bot/memory/store_replies.py:9209 `_did_you_mean_the_person(memory, low)`
pub fn _did_you_mean_the_person(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_did_you_mean_the_person")
}

/// Python: bot/memory/store_replies.py:9251 `_condition_is_their_food(who, condition, low='')` (never called on the trace corpus)
pub fn _condition_is_their_food(who: &str, condition: &Value, low: &str) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_condition_is_their_food")
}

/// Python: bot/memory/store_replies.py:9281 `_diet_sentence(who, diet)` (never called on the trace corpus)
pub fn _diet_sentence(who: &str, diet: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_diet_sentence")
}

/// Python: bot/memory/store_replies.py:9310 `car_line(car)` (never called on the trace corpus)
pub fn car_line(car: &Value) -> String {
    crate::unported!("bot.memory.store_replies:car_line")
}

/// Python: bot/memory/store_replies.py:9333 `_birthday_in_words(memory)` (never called on the trace corpus)
pub fn _birthday_in_words(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_birthday_in_words")
}

/// Python: bot/memory/store_replies.py:9356 `_birthday_reply(memory)` (never called on the trace corpus)
pub fn _birthday_reply(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_birthday_reply")
}

/// Python: bot/memory/store_replies.py:9386 `_the_same_relative(one, other)` (never called on the trace corpus)
pub fn _the_same_relative(one: &str, other: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_the_same_relative")
}

/// Python: bot/memory/store_replies.py:9396 `_a_persons_birthday_reply(memory, low)`
pub fn _a_persons_birthday_reply(memory: &mut Memory, low: &str) {
    crate::unported!("bot.memory.store_replies:_a_persons_birthday_reply")
}

/// Python: bot/memory/store_replies.py:9459 `_weight_reply(memory)` (never called on the trace corpus)
pub fn _weight_reply(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_weight_reply")
}

/// Python: bot/memory/store_replies.py:9498 `_height_reply(memory)` (never called on the trace corpus)
pub fn _height_reply(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_height_reply")
}

/// Python: bot/memory/store_replies.py:9514 `_fact_reply(memory, low, key=None)`
pub fn _fact_reply(memory: &mut Memory, low: &str, key: Option<&str>) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_fact_reply")
}

/// Python: bot/memory/store_replies.py:9684 `_was_called(named)` (never called on the trace corpus)
pub fn _was_called(named: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_was_called")
}

/// Python: bot/memory/store_replies.py:9692 `_a_pet_that_is_gone(gone)` (never called on the trace corpus)
pub fn _a_pet_that_is_gone(gone: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_a_pet_that_is_gone")
}

/// Python: bot/memory/store_replies.py:9715 `_put_together(parts)` (never called on the trace corpus)
pub fn _put_together(parts: &[String]) -> Value {
    crate::unported!("bot.memory.store_replies:_put_together")
}

/// Python: bot/memory/store_replies.py:9724 `_pets_together(pets)` (never called on the trace corpus)
pub fn _pets_together(pets: &Value) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_pets_together")
}

/// Python: bot/memory/store_replies.py:9755 `_a_pet(pet)` (never called on the trace corpus)
pub fn _a_pet(pet: &Value) -> String {
    crate::unported!("bot.memory.store_replies:_a_pet")
}

/// Python: bot/memory/store_replies.py:9783 `_a_why_line_they_have_answered(text, memory)`
pub fn _a_why_line_they_have_answered(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_a_why_line_they_have_answered")
}

/// Python: bot/memory/store_replies.py:9788 `_holds_the_subject(memory, text)`
pub fn _holds_the_subject(memory: &mut Memory, text: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_holds_the_subject")
}

/// Python: bot/memory/store_replies.py:9804 `_asks_for_a_fact(low, text='', memory=None)`
pub fn _asks_for_a_fact(low: &str, text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_asks_for_a_fact")
}

/// Python: bot/memory/store_replies.py:9829 `_vary(options, memory)`
pub fn _vary(options: &[String], memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_vary")
}

/// Python: bot/memory/store_replies.py:9837 `_turn_lines(memory)` (never called on the trace corpus)
pub fn _turn_lines(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_turn_lines")
}

/// Python: bot/memory/store_replies.py:9866 `asks_again(text, memory=None)`
pub fn asks_again(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:asks_again")
}

/// Python: bot/memory/store_replies.py:9877 `_repeat_reply(memory, text='')` (never called on the trace corpus)
pub fn _repeat_reply(memory: &Memory, text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_repeat_reply")
}

/// Python: bot/memory/store_replies.py:9946 `_a_whole_answer_to_hand_back(memory, text='')` (never called on the trace corpus)
pub fn _a_whole_answer_to_hand_back(memory: &Memory, text: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_whole_answer_to_hand_back")
}

/// Python: bot/memory/store_replies.py:9962 `_carries_something(turn)` (never called on the trace corpus)
pub fn _carries_something(turn: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_carries_something")
}

/// Python: bot/memory/store_replies.py:9985 `_only_a_nod(line)` (never called on the trace corpus)
pub fn _only_a_nod(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_only_a_nod")
}

/// Python: bot/memory/store_replies.py:9992 `_hearing_again(said, line)` (never called on the trace corpus)
pub fn _hearing_again(said: &str, line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_hearing_again")
}

/// Python: bot/memory/store_replies.py:10032 `_just_a_hello(line)` (never called on the trace corpus)
pub fn _just_a_hello(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_just_a_hello")
}

/// Python: bot/memory/store_replies.py:10045 `_worth_reporting(line)` (never called on the trace corpus)
pub fn _worth_reporting(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_worth_reporting")
}

/// Python: bot/memory/store_replies.py:10056 `_first_reply(memory)` (never called on the trace corpus)
pub fn _first_reply(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_first_reply")
}

/// Python: bot/memory/store_replies.py:10075 `_what_i_said_about(memory, thing)` (never called on the trace corpus)
pub fn _what_i_said_about(memory: &Memory, thing: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_what_i_said_about")
}

/// Python: bot/memory/store_replies.py:10126 `_adds_something(said, asked)` (never called on the trace corpus)
pub fn _adds_something(said: &str, asked: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_adds_something")
}

/// Python: bot/memory/store_replies.py:10137 `_first_bot_reply(memory)` (never called on the trace corpus)
pub fn _first_bot_reply(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_first_bot_reply")
}

/// Python: bot/memory/store_replies.py:10146 `_asked_and_answered(memory)` (never called on the trace corpus)
pub fn _asked_and_answered(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_asked_and_answered")
}

/// Python: bot/memory/store_replies.py:10162 `_said_reply(memory)` (never called on the trace corpus)
pub fn _said_reply(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_said_reply")
}

/// Python: bot/memory/store_replies.py:10242 `_corrects(line)` (never called on the trace corpus)
pub fn _corrects(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_corrects")
}

/// Python: bot/memory/store_replies.py:10246 `_last_real(memory)` (never called on the trace corpus)
pub fn _last_real(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_last_real")
}

/// Python: bot/memory/store_replies.py:10265 `_the_fact_is_kept(key, memory)` (never called on the trace corpus)
pub fn _the_fact_is_kept(key: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_the_fact_is_kept")
}

/// Python: bot/memory/store_replies.py:10270 `_a_denial(reply)` (never called on the trace corpus)
pub fn _a_denial(reply: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_denial")
}

/// Python: bot/memory/store_replies.py:10274 `_names_its_thing(low)`
pub fn _names_its_thing(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_names_its_thing")
}

/// Python: bot/memory/store_replies.py:10290 `_say_back(line, memory=None)` (never called on the trace corpus)
pub fn _say_back(line: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_say_back")
}

/// Python: bot/memory/store_replies.py:10306 `_recall_reply(memory, low, text='', spoken=False)` (never called on the trace corpus)
pub fn _recall_reply(memory: &mut Memory, low: &str, text: &str, spoken: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_recall_reply")
}

/// Python: bot/memory/store_replies.py:10487 `_tagged(line)` (never called on the trace corpus)
pub fn _tagged(line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_tagged")
}

/// Python: bot/memory/store_replies.py:10492 `_the_same_person(who)` (never called on the trace corpus)
pub fn _the_same_person(who: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_same_person")
}

/// Python: bot/memory/store_replies.py:10513 `_the_name_pattern(who)` (never called on the trace corpus)
pub fn _the_name_pattern(who: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_name_pattern")
}

/// Python: bot/memory/store_replies.py:10535 `_the_word_for_them(who)` (never called on the trace corpus)
pub fn _the_word_for_them(who: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_word_for_them")
}

/// Python: bot/memory/store_replies.py:10574 `_a_stay_in_a_place(text)` (never called on the trace corpus)
pub fn _a_stay_in_a_place(text: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_stay_in_a_place")
}

/// Python: bot/memory/store_replies.py:10598 `_kind(thing)` (never called on the trace corpus)
pub fn _kind(thing: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_kind")
}

/// Python: bot/memory/store_replies.py:10605 `_says_where_they_live(text)` (never called on the trace corpus)
pub fn _says_where_they_live(text: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_says_where_they_live")
}

/// Python: bot/memory/store_replies.py:10620 `_a_place_they_go(text)` (never called on the trace corpus)
pub fn _a_place_they_go(text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_a_place_they_go")
}

/// Python: bot/memory/store_replies.py:10634 `_about_kind(thing, text, memory=None, phrase=False)` (never called on the trace corpus)
pub fn _about_kind(thing: &str, text: &str, memory: &mut Memory, phrase: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_about_kind")
}

/// Python: bot/memory/store_replies.py:10666 `_seen(line)` (never called on the trace corpus)
pub fn _seen(line: &str) -> HashSet<String> {
    crate::unported!("bot.memory.store_replies:_seen")
}

/// Python: bot/memory/store_replies.py:10681 `_alike(word, seen)` (never called on the trace corpus)
pub fn _alike(word: &str, seen: &HashSet<String>) -> bool {
    crate::unported!("bot.memory.store_replies:_alike")
}

/// Python: bot/memory/store_replies.py:10704 `_recall_quote(line, memory=None, asked=False)` (never called on the trace corpus)
pub fn _recall_quote(line: &str, memory: Option<&Memory>, asked: bool) -> String {
    crate::unported!("bot.memory.store_replies:_recall_quote")
}

/// Python: bot/memory/store_replies.py:10739 `_a_figure_put_right(line, memory)` (never called on the trace corpus)
pub fn _a_figure_put_right(line: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_a_figure_put_right")
}

/// Python: bot/memory/store_replies.py:10778 `_a_narrowing_put_after(line, memory)` (never called on the trace corpus)
pub fn _a_narrowing_put_after(line: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_a_narrowing_put_after")
}

/// Python: bot/memory/store_replies.py:10795 `_put_right_after_it(line, memory)` (never called on the trace corpus)
pub fn _put_right_after_it(line: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_put_right_after_it")
}

/// Python: bot/memory/store_replies.py:10824 `_age_reply(memory)` (never called on the trace corpus)
pub fn _age_reply(memory: &mut Memory) -> String {
    crate::unported!("bot.memory.store_replies:_age_reply")
}

/// Python: bot/memory/store_replies.py:10833 `_content_words(line)` (never called on the trace corpus)
pub fn _content_words(line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_content_words")
}

/// Python: bot/memory/store_replies.py:10837 `_the_line_before_was_a_plan(memory)` (never called on the trace corpus)
pub fn _the_line_before_was_a_plan(memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_the_line_before_was_a_plan")
}

/// Python: bot/memory/store_replies.py:10854 `_plan_was_corrected(memory)` (never called on the trace corpus)
pub fn _plan_was_corrected(memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_plan_was_corrected")
}

/// Python: bot/memory/store_replies.py:10871 `_about_the_same(line, line_words)` (never called on the trace corpus)
pub fn _about_the_same(line: &str, line_words: &[String]) -> bool {
    crate::unported!("bot.memory.store_replies:_about_the_same")
}

/// Python: bot/memory/store_replies.py:10875 `_activity_line(memory)` (never called on the trace corpus)
pub fn _activity_line(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_activity_line")
}

/// Python: bot/memory/store_replies.py:10906 `_the_making_line_they_said(memory, back=8)` (never called on the trace corpus)
pub fn _the_making_line_they_said(memory: &Memory, back: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_the_making_line_they_said")
}

/// Python: bot/memory/store_replies.py:10924 `_recap(line)` (never called on the trace corpus)
pub fn _recap(line: &str) -> String {
    crate::unported!("bot.memory.store_replies:_recap")
}

/// Python: bot/memory/store_replies.py:10940 `_tidy_words(text)` (never called on the trace corpus)
pub fn _tidy_words(text: &str) -> String {
    crate::unported!("bot.memory.store_replies:_tidy_words")
}

/// Python: bot/memory/store_replies.py:10961 `_a_thing(name)` (never called on the trace corpus)
pub fn _a_thing(name: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_thing")
}

/// Python: bot/memory/store_replies.py:10983 `_says_the_thing(said, thing)` (never called on the trace corpus)
pub fn _says_the_thing(said: &str, thing: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_says_the_thing")
}

/// Python: bot/memory/store_replies.py:10993 `_a_description(thing)` (never called on the trace corpus)
pub fn _a_description(thing: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_description")
}

/// Python: bot/memory/store_replies.py:11005 `_the_fixed_fact(memory)` (never called on the trace corpus)
pub fn _the_fixed_fact(memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_the_fixed_fact")
}

/// Python: bot/memory/store_replies.py:11028 `_the_line_named(memory, low)` (never called on the trace corpus)
pub fn _the_line_named(memory: &Memory, low: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_line_named")
}

/// Python: bot/memory/store_replies.py:11051 `_topic_reply(memory, low, again=False)` (never called on the trace corpus)
pub fn _topic_reply(memory: &mut Memory, low: &str, again: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_topic_reply")
}

/// Python: bot/memory/store_replies.py:11157 `_the_last_ask_about_the_bot(memory)` (never called on the trace corpus)
pub fn _the_last_ask_about_the_bot(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_the_last_ask_about_the_bot")
}

/// Python: bot/memory/store_replies.py:11173 `_asked_reply(memory)` (never called on the trace corpus)
pub fn _asked_reply(memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_asked_reply")
}

/// Python: bot/memory/store_replies.py:11198 `_last_told(memory)` (never called on the trace corpus)
pub fn _last_told(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_last_told")
}

/// Python: bot/memory/store_replies.py:11226 `_as_repaired(word)` (never called on the trace corpus)
pub fn _as_repaired(word: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_as_repaired")
}

/// Python: bot/memory/store_replies.py:11230 `_with_the_name(said, raw)` (never called on the trace corpus)
pub fn _with_the_name(said: &str, raw: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_with_the_name")
}

/// Python: bot/memory/store_replies.py:11276 `_reads_as_a_correction(line)` (never called on the trace corpus)
pub fn _reads_as_a_correction(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_reads_as_a_correction")
}

/// Python: bot/memory/store_replies.py:11286 `_lines_that_moved(memory)` (never called on the trace corpus)
pub fn _lines_that_moved(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_lines_that_moved")
}

/// Python: bot/memory/store_replies.py:11325 `_the_summary_already_says(memory, flat)` (never called on the trace corpus)
pub fn _the_summary_already_says(memory: &mut Memory, flat: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_the_summary_already_says")
}

/// Python: bot/memory/store_replies.py:11343 `_safety_lines(memory)` (never called on the trace corpus)
pub fn _safety_lines(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_safety_lines")
}

/// Python: bot/memory/store_replies.py:11371 `_shape(line)` (never called on the trace corpus)
pub fn _shape(line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_shape")
}

/// Python: bot/memory/store_replies.py:11383 `_out_of_date_lines(memory)` (never called on the trace corpus)
pub fn _out_of_date_lines(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_out_of_date_lines")
}

/// Python: bot/memory/store_replies.py:11427 `_root(word)` (never called on the trace corpus)
pub fn _root(word: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_root")
}

/// Python: bot/memory/store_replies.py:11440 `_one_word(first, second)` (never called on the trace corpus)
pub fn _one_word(first: &Value, second: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_one_word")
}

/// Python: bot/memory/store_replies.py:11475 `_judging_themselves(line)` (never called on the trace corpus)
pub fn _judging_themselves(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_judging_themselves")
}

/// Python: bot/memory/store_replies.py:11480 `_recent_told(memory, limit=3)` (never called on the trace corpus)
pub fn _recent_told(memory: &mut Memory, limit: i64) -> Value {
    crate::unported!("bot.memory.store_replies:_recent_told")
}

/// Python: bot/memory/store_replies.py:11567 `_all_about_the_bot(memory)` (never called on the trace corpus)
pub fn _all_about_the_bot(memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_all_about_the_bot")
}

/// Python: bot/memory/store_replies.py:11582 `_a_figure_the_store_could_not_keep(line, memory)` (never called on the trace corpus)
pub fn _a_figure_the_store_could_not_keep(line: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_a_figure_the_store_could_not_keep")
}

/// Python: bot/memory/store_replies.py:11597 `_the_thing_the_chat_is_on(memory)` (never called on the trace corpus)
pub fn _the_thing_the_chat_is_on(memory: &Memory) -> String {
    crate::unported!("bot.memory.store_replies:_the_thing_the_chat_is_on")
}

/// Python: bot/memory/store_replies.py:11618 `_a_fact_of_its_own(clause)` (never called on the trace corpus)
pub fn _a_fact_of_its_own(clause: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_fact_of_its_own")
}

/// Python: bot/memory/store_replies.py:11623 `_told_phrase(text)` (never called on the trace corpus)
pub fn _told_phrase(text: &str) -> String {
    crate::unported!("bot.memory.store_replies:_told_phrase")
}

/// Python: bot/memory/store_replies.py:11661 `_expanded(text)` (never called on the trace corpus)
pub fn _expanded(text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_expanded")
}

/// Python: bot/memory/store_replies.py:11678 `_a_judgement_was_said(memory)` (never called on the trace corpus)
pub fn _a_judgement_was_said(memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_a_judgement_was_said")
}

/// Python: bot/memory/store_replies.py:11696 `_plain_words(text)` (never called on the trace corpus)
pub fn _plain_words(text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_plain_words")
}

/// Python: bot/memory/store_replies.py:11700 `_says_nothing_new(phrase, told_words, body_words)` (never called on the trace corpus)
pub fn _says_nothing_new(phrase: &str, told_words: &[String], body_words: &[String]) -> bool {
    crate::unported!("bot.memory.store_replies:_says_nothing_new")
}

/// Python: bot/memory/store_replies.py:11713 `_summary_words(text)` (never called on the trace corpus)
pub fn _summary_words(text: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_summary_words")
}

/// Python: bot/memory/store_replies.py:11730 `_already_said(words, said)` (never called on the trace corpus)
pub fn _already_said(words: &[String], said: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_already_said")
}

/// Python: bot/memory/store_replies.py:11748 `_taste_reply(memory, low)` (never called on the trace corpus)
pub fn _taste_reply(memory: &mut Memory, low: &str) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_taste_reply")
}

/// Python: bot/memory/store_replies.py:11781 `_tastes_said(memory)` (never called on the trace corpus)
pub fn _tastes_said(memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_tastes_said")
}

/// Python: bot/memory/store_replies.py:11813 `_without_the_age(line)` (never called on the trace corpus)
pub fn _without_the_age(line: &str) -> String {
    crate::unported!("bot.memory.store_replies:_without_the_age")
}

/// Python: bot/memory/store_replies.py:11818 `_the_words_of_a_fact(value)` (never called on the trace corpus)
pub fn _the_words_of_a_fact(value: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_the_words_of_a_fact")
}

/// Python: bot/memory/store_replies.py:11833 `_a_fact_the_recap_does_not_say(memory, phrase, body_words)` (never called on the trace corpus)
pub fn _a_fact_the_recap_does_not_say(memory: &mut Memory, phrase: &str, body_words: &[String]) -> bool {
    crate::unported!("bot.memory.store_replies:_a_fact_the_recap_does_not_say")
}

/// Python: bot/memory/store_replies.py:11855 `_kids_told(memory)` (never called on the trace corpus)
pub fn _kids_told(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_kids_told")
}

/// Python: bot/memory/store_replies.py:11871 `_a_child_is_described(memory)` (never called on the trace corpus)
pub fn _a_child_is_described(memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_a_child_is_described")
}

/// Python: bot/memory/store_replies.py:11880 `_a_few_children(count)` (never called on the trace corpus)
pub fn _a_few_children(count: i64) -> String {
    crate::unported!("bot.memory.store_replies:_a_few_children")
}

/// Python: bot/memory/store_replies.py:11909 `_summed_up(low)`
pub fn _summed_up(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_summed_up")
}

/// Python: bot/memory/store_replies.py:11913 `_summary_reply(memory)` (never called on the trace corpus)
pub fn _summary_reply(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_summary_reply")
}

/// Python: bot/memory/store_replies.py:12170 `_quote(text, limit=90)` (never called on the trace corpus)
pub fn _quote(text: &str, limit: i64) -> String {
    crate::unported!("bot.memory.store_replies:_quote")
}

/// Python: bot/memory/store_replies.py:12192 `_heavy(line)` (never called on the trace corpus)
pub fn _heavy(line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_heavy")
}

/// Python: bot/memory/store_replies.py:12196 `_topics_reply(memory)` (never called on the trace corpus)
pub fn _topics_reply(memory: &Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_topics_reply")
}

/// Python: bot/memory/store_replies.py:12216 `_first_sentence(text, limit=70)` (never called on the trace corpus)
pub fn _first_sentence(text: &str, limit: i64) -> String {
    crate::unported!("bot.memory.store_replies:_first_sentence")
}

/// Python: bot/memory/store_replies.py:12295 `_sentences(answer)` (never called on the trace corpus)
pub fn _sentences(answer: &str) -> Vec<String> {
    crate::unported!("bot.memory.store_replies:_sentences")
}

/// Python: bot/memory/store_replies.py:12299 `_halves(sentence)` (never called on the trace corpus)
pub fn _halves(sentence: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_halves")
}

/// Python: bot/memory/store_replies.py:12308 `_says_something(part)` (never called on the trace corpus)
pub fn _says_something(part: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_says_something")
}

/// Python: bot/memory/store_replies.py:12320 `_seconds(said)` (never called on the trace corpus)
pub fn _seconds(said: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_seconds")
}

/// Python: bot/memory/store_replies.py:12345 `_names_minutes(said)` (never called on the trace corpus)
pub fn _names_minutes(said: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_names_minutes")
}

/// Python: bot/memory/store_replies.py:12350 `_longest(said, bare=False)` (never called on the trace corpus)
pub fn _longest(said: &str, bare: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_longest")
}

/// Python: bot/memory/store_replies.py:12381 `_a_remark_about_a_length(answer)` (never called on the trace corpus)
pub fn _a_remark_about_a_length(answer: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_a_remark_about_a_length")
}

/// Python: bot/memory/store_replies.py:12397 `_moves(answer)` (never called on the trace corpus)
pub fn _moves(answer: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_moves")
}

/// Python: bot/memory/store_replies.py:12408 `_up_to_the_move(answer, part)` (never called on the trace corpus)
pub fn _up_to_the_move(answer: &str, part: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_up_to_the_move")
}

/// Python: bot/memory/store_replies.py:12430 `_the_move_that_fits(moves, room)` (never called on the trace corpus)
pub fn _the_move_that_fits(moves: &Value, room: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_the_move_that_fits")
}

/// Python: bot/memory/store_replies.py:12456 `_left_out(moves, room)` (never called on the trace corpus)
pub fn _left_out(moves: &Value, room: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_left_out")
}

/// Python: bot/memory/store_replies.py:12460 `_the_lead(answer)` (never called on the trace corpus)
pub fn _the_lead(answer: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_lead")
}

/// Python: bot/memory/store_replies.py:12478 `_a_rate_of_the_same_length(answer, room)` (never called on the trace corpus)
pub fn _a_rate_of_the_same_length(answer: &str, room: &Value) -> bool {
    crate::unported!("bot.memory.store_replies:_a_rate_of_the_same_length")
}

/// Python: bot/memory/store_replies.py:12489 `_the_budget_on_the_answer(text, memory)`
pub fn _the_budget_on_the_answer(text: &str, memory: &Memory) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_the_budget_on_the_answer")
}

/// Python: bot/memory/store_replies.py:12611 `_theirs(length, mid=False)` (never called on the trace corpus)
pub fn _theirs(length: &Value, mid: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_theirs")
}

/// Python: bot/memory/store_replies.py:12620 `_tidy(part)` (never called on the trace corpus)
pub fn _tidy(part: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_tidy")
}

/// Python: bot/memory/store_replies.py:12633 `_clauses(reply)` (never called on the trace corpus)
pub fn _clauses(reply: &str) -> Vec<String> {
    crate::unported!("bot.memory.store_replies:_clauses")
}

/// Python: bot/memory/store_replies.py:12638 `_the_limit(reply)` (never called on the trace corpus)
pub fn _the_limit(reply: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_limit")
}

/// Python: bot/memory/store_replies.py:12648 `_the_condition(reply)` (never called on the trace corpus)
pub fn _the_condition(reply: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_condition")
}

/// Python: bot/memory/store_replies.py:12662 `_recent_answers(memory, back=3)` (never called on the trace corpus)
pub fn _recent_answers(memory: &Memory, back: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_recent_answers")
}

/// Python: bot/memory/store_replies.py:12683 `_shares_a_word(low, said)` (never called on the trace corpus)
pub fn _shares_a_word(low: &str, said: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_shares_a_word")
}

/// Python: bot/memory/store_replies.py:12694 `_the_call_from_the_advice(text, memory)`
pub fn _the_call_from_the_advice(text: &str, memory: &Memory) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_the_call_from_the_advice")
}

/// Python: bot/memory/store_replies.py:12743 `_trim(text, limit=90)` (never called on the trace corpus)
pub fn _trim(text: &str, limit: i64) -> Value {
    crate::unported!("bot.memory.store_replies:_trim")
}

/// Python: bot/memory/store_replies.py:12751 `_food_facts(memory)`
pub fn _food_facts(memory: &mut Memory) -> Vec<String> {
    crate::unported!("bot.memory.store_replies:_food_facts")
}

/// Python: bot/memory/store_replies.py:12773 `_the_food_a_condition_rules_out(condition)` (never called on the trace corpus)
pub fn _the_food_a_condition_rules_out(condition: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_the_food_a_condition_rules_out")
}

/// Python: bot/memory/store_replies.py:12788 `_what_a_dinner_has_to_leave_out(memory)` (never called on the trace corpus)
pub fn _what_a_dinner_has_to_leave_out(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.store_replies:_what_a_dinner_has_to_leave_out")
}

/// Python: bot/memory/store_replies.py:12820 `_all_of_them(count)` (never called on the trace corpus)
pub fn _all_of_them(count: i64) -> String {
    crate::unported!("bot.memory.store_replies:_all_of_them")
}

/// Python: bot/memory/store_replies.py:12834 `_say_a_list(items)` (never called on the trace corpus)
pub fn _say_a_list(items: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_say_a_list")
}

/// Python: bot/memory/store_replies.py:12943 `_reads_as_the_invitation(text, memory)`
pub fn _reads_as_the_invitation(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_reads_as_the_invitation")
}

/// Python: bot/memory/store_replies.py:12958 `_the_module_behind(memory, said)` (never called on the trace corpus)
pub fn _the_module_behind(memory: &Memory, said: &str) -> String {
    crate::unported!("bot.memory.store_replies:_the_module_behind")
}

/// Python: bot/memory/store_replies.py:12966 `_the_line_before_the_check(memory, line)` (never called on the trace corpus)
pub fn _the_line_before_the_check(memory: &Memory, line: &str) -> Value {
    crate::unported!("bot.memory.store_replies:_the_line_before_the_check")
}

/// Python: bot/memory/store_replies.py:12980 `_a_check_choice(options, memory, salt)` (never called on the trace corpus)
pub fn _a_check_choice(options: &Value, memory: &Memory, salt: &Value) -> Value {
    crate::unported!("bot.memory.store_replies:_a_check_choice")
}

/// Python: bot/memory/store_replies.py:12987 `_a_check_came_before(memory, line)` (never called on the trace corpus)
pub fn _a_check_came_before(memory: &Memory, line: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_check_came_before")
}

/// Python: bot/memory/store_replies.py:13007 `_the_followed_check(text, memory)`
pub fn _the_followed_check(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_the_followed_check")
}

/// Python: bot/memory/store_replies.py:13085 `_plain_line(text)`
pub fn _plain_line(text: &str) -> String {
    crate::unported!("bot.memory.store_replies:_plain_line")
}

/// Python: bot/memory/store_replies.py:13115 `_a_subject_to_leave_alone(low)`
pub fn _a_subject_to_leave_alone(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_subject_to_leave_alone")
}

/// Python: bot/memory/store_replies.py:13140 `_closes_the_subject(text)`
pub fn _closes_the_subject(text: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_closes_the_subject")
}

/// Python: bot/memory/store_replies.py:13161 `_closes_by_a_frame(low)`
pub fn _closes_by_a_frame(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_closes_by_a_frame")
}

/// Python: bot/memory/store_replies.py:13170 `_a_plain_frame_closes(low)` (never called on the trace corpus)
pub fn _a_plain_frame_closes(low: &str) -> bool {
    crate::unported!("bot.memory.store_replies:_a_plain_frame_closes")
}

/// Python: bot/memory/store_replies.py:13182 `_held_thread(memory)` (never called on the trace corpus)
pub fn _held_thread(memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_held_thread")
}

/// Python: bot/memory/store_replies.py:13193 `_gentle_subject(memory)` (never called on the trace corpus)
pub fn _gentle_subject(memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_gentle_subject")
}

/// Python: bot/memory/store_replies.py:13205 `_can_close(text, memory)`
pub fn _can_close(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.store_replies:_can_close")
}

/// Python: bot/memory/store_replies.py:13226 `_the_close(text, memory)`
pub fn _the_close(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.memory.store_replies:_the_close")
}

