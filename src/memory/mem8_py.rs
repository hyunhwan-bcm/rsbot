//! Python behaviour shared by the MEM8 workstream's small readers (`a_year_of_birth` ...
//! `workplace`): `str()`, `int()`, the memory as a dict value, and `bot.compose.pick` on a
//! typed memory. Not a port of any one Python file.
#![allow(dead_code, unused_macros, unused_imports)]

use crate::data::Value;
use crate::py::text;
use crate::state::Memory;

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
        Value::Dict(d) => format!(
            "{{{}}}",
            d.iter().map(|(k, v)| format!("{}: {}", py_repr(k), py_repr(v))).collect::<Vec<_>>().join(", ")
        ),
        _ => py_str(v),
    }
}

/// Python `str(v or "")`: "" for a missing or falsy value, else `str(v)`.
pub(crate) fn s_or(v: Option<&Value>) -> String {
    match v {
        Some(v) if v.truthy() => py_str(v),
        _ => String::new(),
    }
}

/// Python `int(v)`: None where Python raises TypeError or ValueError.
pub(crate) fn py_int(v: &Value) -> Option<i64> {
    match v {
        Value::Int(i) => Some(*i),
        Value::Bool(b) => Some(*b as i64),
        Value::Float(f) if f.is_finite() => Some(f.trunc() as i64),
        Value::Str(s) => py_int_str(s),
        _ => None,
    }
}

/// Python `int(s)` for a str: surrounding whitespace, a sign, digits with single underscores
/// between them. None where Python raises ValueError.
pub(crate) fn py_int_str(s: &str) -> Option<i64> {
    let t = text::strip(s);
    let (neg, digits) = match t.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    if digits.is_empty() || digits.starts_with('_') || digits.ends_with('_') || digits.contains("__") {
        return None;
    }
    let mut n: i64 = 0;
    for c in digits.chars() {
        if c == '_' {
            continue;
        }
        let d = decimal_value(c)?;
        n = n.checked_mul(10)?.checked_add(d)?;
    }
    Some(if neg { -n } else { n })
}

/// The value of a Unicode decimal digit (category Nd), as `int()` reads it. Every run of Nd
/// characters is whole tens starting at a zero, so the value is the offset in its run.
pub(crate) fn decimal_value(c: char) -> Option<i64> {
    use crate::understanding::pyuni::ch_decimal;
    if c.is_ascii() {
        return c.to_digit(10).map(|d| d as i64);
    }
    if !ch_decimal(c) {
        return None;
    }
    let mut start = c as u32;
    while char::from_u32(start - 1).is_some_and(ch_decimal) {
        start -= 1;
    }
    Some(((c as u32 - start) % 10) as i64)
}

/// `memory.get(key)` as a value (None when missing).
pub(crate) fn mget<'a>(memory: &'a Memory, key: &str) -> &'a Value {
    static NONE: Value = Value::None;
    memory.get(key).unwrap_or(&NONE)
}

/// The memory as a Python-like dict value (for the contract functions that take one).
pub(crate) fn memory_value(memory: &Memory) -> Value {
    Value::from_plain(&serde_json::Value::Object(memory.to_plain()))
}

/// `pick(options, memory, seed_text)` from `bot.compose`.
pub(crate) fn pick<S: AsRef<str>>(options: &[S], memory: &Memory, seed_text: &str) -> String {
    let owned: Vec<String> = options.iter().map(|o| o.as_ref().to_string()).collect();
    crate::compose::pick(&owned, &memory_value(memory), seed_text)
}

/// `len(s.split())`.
pub(crate) fn word_count(s: &str) -> usize {
    text::split_ref(s).len()
}

/// A str list as a Python list value.
pub(crate) fn str_list<S: AsRef<str>>(items: &[S]) -> Value {
    Value::List(std::sync::Arc::new(items.iter().map(|s| Value::from(s.as_ref())).collect()))
}

/// A compiled module-level regex, by Python name (the calling file's `M` is its module prefix).
macro_rules! rx {
    ($($name:ident),* $(,)?) => {$(
        static $name: std::sync::LazyLock<&'static crate::py::re::Re> =
            std::sync::LazyLock::new(|| crate::data::re(&format!("{}{}", M, stringify!($name))));
    )*};
}
/// A module-level tuple/list of strings, in order.
macro_rules! strs {
    ($($name:ident),* $(,)?) => {$(
        static $name: std::sync::LazyLock<Vec<String>> =
            std::sync::LazyLock::new(|| crate::data::strs(&format!("{}{}", M, stringify!($name))));
    )*};
}
/// A module-level collection of strings, for membership tests.
macro_rules! strset {
    ($($name:ident),* $(,)?) => {$(
        static $name: std::sync::LazyLock<std::collections::HashSet<String>> =
            std::sync::LazyLock::new(|| crate::data::set(&format!("{}{}", M, stringify!($name))));
    )*};
}
/// Any module-level value, as exported.
macro_rules! val {
    ($($name:ident),* $(,)?) => {$(
        static $name: std::sync::LazyLock<&'static crate::data::Value> =
            std::sync::LazyLock::new(|| crate::data::get(&format!("{}{}", M, stringify!($name))));
    )*};
}
/// A regex written inline in a Python function: `inline_re!(NAME, r"pattern", flags)`.
macro_rules! inline_re {
    ($name:ident, $pattern:expr, $flags:expr) => {
        static $name: std::sync::LazyLock<crate::py::re::Re> =
            std::sync::LazyLock::new(|| crate::py::re::Re::new($pattern, $flags));
    };
}
/// A module-level int.
macro_rules! int {
    ($($name:ident),* $(,)?) => {$(
        static $name: std::sync::LazyLock<i64> = std::sync::LazyLock::new(|| {
            crate::data::get(&format!("{}{}", M, stringify!($name))).as_int().expect(stringify!($name))
        });
    )*};
}
pub(crate) use {inline_re, int, rx, strs, strset, val};

/// The value a call into another file gives back, whatever Rust type its contract has: a
/// contract generated from a trace that only saw `None` returns `()`, and such a function's
/// real value arrives here unchanged once its owner types it properly.
pub(crate) trait PyVal {
    fn val(self) -> Value;
}
impl PyVal for () {
    fn val(self) -> Value {
        Value::None
    }
}
impl PyVal for Value {
    fn val(self) -> Value {
        self
    }
}
impl PyVal for &Value {
    fn val(self) -> Value {
        self.clone()
    }
}
impl PyVal for bool {
    fn val(self) -> Value {
        Value::Bool(self)
    }
}
impl PyVal for i64 {
    fn val(self) -> Value {
        Value::Int(self)
    }
}
impl PyVal for String {
    fn val(self) -> Value {
        Value::from(self)
    }
}
impl PyVal for &str {
    fn val(self) -> Value {
        Value::from(self)
    }
}
impl PyVal for Vec<String> {
    fn val(self) -> Value {
        Value::from(self)
    }
}
impl PyVal for Vec<Value> {
    fn val(self) -> Value {
        Value::from(self)
    }
}
impl<T: PyVal> PyVal for Option<T> {
    fn val(self) -> Value {
        self.map(PyVal::val).unwrap_or(Value::None)
    }
}
impl<A: PyVal, B: PyVal> PyVal for (A, B) {
    fn val(self) -> Value {
        Value::Tuple(std::sync::Arc::new(vec![self.0.val(), self.1.val()]))
    }
}

/// `s[:1].lower() + s[1:]`.
pub(crate) fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().chain(c).collect(),
    }
}

/// The last turn's bot line (`turns[-1].get("bot") or ""`), "" without a turn.
pub(crate) fn last_bot(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.bot.clone()).unwrap_or_default()
}

/// The last turn's user line, "" without a turn.
pub(crate) fn last_user(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.user.clone()).unwrap_or_default()
}
