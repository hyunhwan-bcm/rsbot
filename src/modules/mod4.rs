//! Shared Python-behaviour helpers for the small modules of `bot/modules` (the MOD4 files):
//! `str()`/truthiness on `Value`s, `try/except`, the adapters for calls into files whose
//! contracts are still being typed, and the store's own tiny readers (`user`, `others`).
//! Not a port of a PyBot file.
#![allow(dead_code, clippy::all)]

use crate::data::Value;
use crate::py::text;
use crate::state::{Memory, Turn};
use std::sync::Arc;

// ---------------------------------------------------------------------------------------------
// Python behaviour helpers.

pub(crate) static NONE: Value = Value::None;

/// Python `str(v)`.
pub(crate) fn py_str(v: &Value) -> String {
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
pub(crate) fn py_repr(v: &Value) -> String {
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
        Value::Dict(d) => {
            format!("{{{}}}", d.iter().map(|(k, v)| format!("{}: {}", py_repr(k), py_repr(v))).collect::<Vec<_>>().join(", "))
        }
        _ => py_str(v),
    }
}

/// Python `str(v or "")`: "" for a falsy value, else `str(v)`.
pub(crate) fn s_or(v: Option<&Value>) -> String {
    match v {
        Some(v) if v.truthy() => py_str(v),
        _ => String::new(),
    }
}

/// Python `str(d.get(key) or "")` on a dict value (anything else reads as `{}`).
pub(crate) fn sget(d: &Value, key: &str) -> String {
    s_or(d.get(key))
}

/// `d.get(key)` as a value (None when missing or when `d` is no dict).
pub(crate) fn vget<'a>(d: &'a Value, key: &str) -> &'a Value {
    d.get(key).unwrap_or(&NONE)
}

/// `d.get(key)` is truthy.
pub(crate) fn has(d: &Value, key: &str) -> bool {
    d.get(key).map_or(false, Value::truthy)
}

/// `key in d` for a dict value.
pub(crate) fn has_key(d: &Value, key: &str) -> bool {
    d.get(key).is_some()
}

/// `memory.get(key)` as a value (None when missing).
pub(crate) fn mget<'a>(memory: &'a Memory, key: &str) -> &'a Value {
    memory.get(key).unwrap_or(&NONE)
}

/// `(memory.get("user") or {})`: the user's facts as stored (never created here).
pub(crate) fn user_ro(memory: &Memory) -> &Value {
    mget(memory, "user")
}

/// `bot.memory.user(memory)`: the user's facts, created on first use.
pub(crate) fn user(memory: &mut Memory) -> &mut Value {
    if !mget(memory, "user").is_dict() {
        memory.set("user", Value::dict());
    }
    memory.get_mut("user").unwrap()
}

/// `bot.memory.others(memory)`: facts about other people, created on first use.
pub(crate) fn others(memory: &mut Memory) -> &mut Value {
    if !mget(memory, "others").is_list() {
        memory.set("others", Value::list());
    }
    memory.get_mut("others").unwrap()
}

/// `bot.memory.log(memory)`: every fact the user stated, created on first use.
pub(crate) fn log(memory: &mut Memory) -> &mut Value {
    if !mget(memory, "facts_log").is_list() {
        memory.set("facts_log", Value::list());
    }
    memory.get_mut("facts_log").unwrap()
}

/// `bot.memory.recent(memory, n)`: the last n turns, oldest first.
pub(crate) fn recent(memory: &Memory, n: usize) -> &[Turn] {
    let turns = &memory.turns;
    if n == 0 {
        // `turns[-0:]` is the whole list.
        return turns;
    }
    &turns[turns.len().saturating_sub(n)..]
}

/// `bot.memory.last_user(memory)`: the last thing the user said before this turn, or "".
pub(crate) fn last_user(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.user.clone()).unwrap_or_default()
}

/// `bot.memory.last_bot(memory)`: the bot's last reply, or "".
pub(crate) fn last_bot(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.bot.clone()).unwrap_or_default()
}

/// `bot.memory.last_module(memory)`: the module that gave the last reply.
pub(crate) fn last_module(memory: &Memory) -> Option<String> {
    memory.turns.last().and_then(|t| t.module.clone())
}

/// `bot.memory.name(memory)`: the user's name, or "".
pub(crate) fn name(memory: &mut Memory) -> String {
    user(memory).get("name").and_then(Value::as_str).unwrap_or("").to_string()
}

/// `" ".join(str(text or "").lower().split())`.
pub(crate) fn fold(text_: &str) -> String {
    text::squash(&text_.to_lowercase())
}

/// A str list as a Python list value.
pub(crate) fn str_list<S: AsRef<str>>(items: &[S]) -> Value {
    Value::List(Arc::new(items.iter().map(|s| Value::from(s.as_ref())).collect()))
}

/// A list value from values.
pub(crate) fn list_of(items: Vec<Value>) -> Value {
    Value::List(Arc::new(items))
}

/// `[str(x) for x in v]` for a list/tuple value.
pub(crate) fn py_strs(v: &Value) -> Vec<String> {
    v.items().iter().map(py_str).collect()
}

/// Python `list(v)`: the items of a container, the characters of a str, the keys of a dict.
pub(crate) fn py_list(v: &Value) -> Vec<Value> {
    match v {
        Value::Str(s) => s.chars().map(|c| Value::from(c.to_string())).collect(),
        Value::Dict(d) => d.iter().map(|(k, _)| k.clone()).collect(),
        Value::None => Vec::new(),
        v => v.items().to_vec(),
    }
}

/// A dict value from (key, value) pairs, in order.
pub(crate) fn dict_of(pairs: Vec<(&str, Value)>) -> Value {
    let mut d = Value::dict();
    for (k, v) in pairs {
        d.set_key(k, v);
    }
    d
}

/// `isinstance(v, int)` (a bool is an int in Python).
pub(crate) fn is_int(v: &Value) -> bool {
    matches!(v, Value::Int(_) | Value::Bool(_))
}

/// The int a value holds (a bool is 0/1).
pub(crate) fn int_of(v: &Value) -> Option<i64> {
    match v {
        Value::Int(i) => Some(*i),
        Value::Bool(b) => Some(*b as i64),
        _ => None,
    }
}

/// Python `int(s)` for a str of decimal digits (surrounding space allowed, an optional sign).
pub(crate) fn py_int(s: &str) -> Option<i64> {
    let t = text::strip(s);
    let (neg, digits) = match t.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    if digits.is_empty() {
        return None;
    }
    let mut n: i64 = 0;
    for c in digits.chars() {
        if c == '_' {
            continue;
        }
        let d = c.to_digit(10)?;
        n = n.checked_mul(10)?.checked_add(d as i64)?;
    }
    Some(if neg { -n } else { n })
}

/// Python `float(s)` for the plain decimal figures the store reads.
pub(crate) fn py_float(s: &str) -> Option<f64> {
    text::strip(s).replace('_', "").parse::<f64>().ok()
}

/// `s[a:b]` in characters.
pub(crate) fn sl(s: &str, a: isize, b: Option<isize>) -> String {
    text::slice(s, a, b)
}

/// `s[:1].upper() + s[1:]`.
pub(crate) fn cap1(s: &str) -> String {
    text::upper_first(s)
}

/// `s[:1].lower() + s[1:]`.
pub(crate) fn low1(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().chain(c).collect(),
    }
}

/// Python `template.format(...)` for templates with plain `{name}` fields and `{}` positions.
pub(crate) fn fmt_named(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut chars = template.chars().peekable();
    let mut position = 0usize;
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
                if name.is_empty() {
                    match fields.get(position) {
                        Some((_, v)) => out.push_str(v),
                        None => panic!("IndexError: str.format position {position}"),
                    }
                    position += 1;
                } else {
                    match fields.iter().find(|(k, _)| *k == name) {
                        Some((_, v)) => out.push_str(v),
                        None => panic!("KeyError: {name:?} in str.format"),
                    }
                }
            }
            c => out.push(c),
        }
    }
    out
}

/// Python `try:` around a call: `Err(())` where Python's `except Exception` would run. A call into
/// a function not yet ported (`UNPORTED ...`) is not a Python exception, so it is not caught here:
/// it goes on up, and the checkers count it as a gap instead of a quietly wrong answer.
pub(crate) fn py_try<T>(f: impl FnOnce() -> T) -> Result<T, ()> {
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

/// The value a call into another file gives back, whatever Rust type its contract has: a
/// contract generated from a trace that only saw `None` returns `()`, and such a function's
/// real value arrives here once its owner types it properly.
pub(crate) trait PyRet {
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
impl PyRet for &mut Value {
    fn val(self) -> Value {
        self.clone()
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
impl PyRet for usize {
    fn val(self) -> Value {
        Value::Int(self as i64)
    }
}
impl PyRet for f64 {
    fn val(self) -> Value {
        Value::Float(self)
    }
}
impl<T: PyRet> PyRet for Option<T> {
    fn val(self) -> Value {
        self.map(PyRet::val).unwrap_or(Value::None)
    }
}
impl<T: PyRet> PyRet for Vec<T> {
    fn val(self) -> Value {
        Value::List(Arc::new(self.into_iter().map(PyRet::val).collect()))
    }
}
impl<A: PyRet, B: PyRet> PyRet for (A, B) {
    fn val(self) -> Value {
        Value::Tuple(Arc::new(vec![self.0.val(), self.1.val()]))
    }
}
impl<A: PyRet, B: PyRet, C: PyRet> PyRet for (A, B, C) {
    fn val(self) -> Value {
        Value::Tuple(Arc::new(vec![self.0.val(), self.1.val(), self.2.val()]))
    }
}

/// A call's value as Python's `x or ""` string: "" for a falsy value.
pub(crate) fn ret_str(v: impl PyRet) -> String {
    let v = v.val();
    s_or(Some(&v))
}

/// A call's value as a truthy string or None (`x or None` for a str result).
pub(crate) fn ret_some(v: impl PyRet) -> Option<String> {
    let v = v.val();
    if v.truthy() {
        Some(py_str(&v))
    } else {
        None
    }
}

/// A call's value as `Option<String>`: None for Python `None`, else `str(v)`.
pub(crate) fn ret_opt(v: impl PyRet) -> Option<String> {
    match v.val() {
        Value::None => None,
        v => Some(py_str(&v)),
    }
}

/// A call's value, truth-tested.
pub(crate) fn ret_true(v: impl PyRet) -> bool {
    v.val().truthy()
}

/// `x if x else None` for a reply string: Python's readers hand back "" or None for "nothing".
pub(crate) fn nz(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

// ---------------------------------------------------------------------------------------------
// ext: the calls into files other workstreams own, each through one wrapper.

/// The memory as a Python-like dict value (for the contract functions that take one).
pub(crate) fn memory_value(memory: &Memory) -> Value {
    Value::from_plain(&serde_json::Value::Object(memory.to_plain()))
}

/// `pick(options, memory, seed_text)` from `bot.compose`.
pub(crate) fn pick<S: AsRef<str>>(options: &[S], memory: &Memory, seed_text: &str) -> String {
    let owned: Vec<String> = options.iter().map(|o| o.as_ref().to_string()).collect();
    crate::compose::pick(&owned, &memory_value(memory), seed_text)
}


/// `x or None` for a hook's reply: a truthy value as its text.
pub(crate) fn or_none(v: impl PyRet) -> Option<String> {
    ret_some(v)
}

/// A hook's reply returned as it is: None for Python `None`, else the text (an empty one kept).
pub(crate) fn as_reply(v: impl PyRet) -> Option<String> {
    ret_opt(v)
}

/// `try: return f() / except Exception: return fallback`.
pub(crate) fn or_on_raise<T>(f: impl FnOnce() -> T, fallback: T) -> T {
    py_try(f).unwrap_or(fallback)
}
