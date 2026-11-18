//! Small Python idioms shared by the MEM7 ports (35 files of `bot/memory/`). Not a port of any
//! PyBot file: the helpers here stand for built-in behaviour (`str.format`, `int()`, `str()`,
//! slices counted in characters) that those files lean on.
#![allow(dead_code, unused_macros, unused_imports, clippy::all)]

use crate::data::Value;
use crate::py::text as pt;
use crate::state::Memory;

/// `bot.memory.<module>.<NAME>` compiled regexes as `LazyLock<&'static Re>` statics.
macro_rules! res {
    ($module:literal; $($n:ident),* $(,)?) => { $(
        static $n: std::sync::LazyLock<&'static crate::py::re::Re> = std::sync::LazyLock::new(||
            crate::data::re(concat!("bot.memory.", $module, ".", stringify!($n))));
    )* };
}
/// `bot.memory.<module>.<NAME>` sets of strings (or a dict's keys), for membership tests.
macro_rules! sets {
    ($module:literal; $($n:ident),* $(,)?) => { $(
        static $n: std::sync::LazyLock<std::collections::HashSet<String>> = std::sync::LazyLock::new(||
            crate::memory::mem7::key_set(crate::data::get(concat!("bot.memory.", $module, ".", stringify!($n)))));
    )* };
}
/// `bot.memory.<module>.<NAME>` tuples / lists of strings, in order.
macro_rules! lists {
    ($module:literal; $($n:ident),* $(,)?) => { $(
        static $n: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
            crate::data::strs(concat!("bot.memory.", $module, ".", stringify!($n))));
    )* };
}
/// `bot.memory.<module>.<NAME>` strings.
macro_rules! texts {
    ($module:literal; $($n:ident),* $(,)?) => { $(
        static $n: std::sync::LazyLock<String> = std::sync::LazyLock::new(||
            crate::data::string(concat!("bot.memory.", $module, ".", stringify!($n))));
    )* };
}
/// `bot.memory.<module>.<NAME>` as the exported value (a dict, a tuple of tuples...).
macro_rules! values {
    ($module:literal; $($n:ident),* $(,)?) => { $(
        static $n: std::sync::LazyLock<&'static crate::data::Value> = std::sync::LazyLock::new(||
            crate::data::get(concat!("bot.memory.", $module, ".", stringify!($n))));
    )* };
}
/// A regex written inline in a function body (`re.search(r"...", text)`), compiled once.
macro_rules! inline_re {
    ($($n:ident = $pattern:expr, $flags:expr;)*) => { $(
        static $n: std::sync::LazyLock<crate::py::re::Re> =
            std::sync::LazyLock::new(|| crate::py::re::Re::new($pattern, $flags));
    )* };
}
pub(crate) use {inline_re, lists, res, sets, texts, values};

/// The strings of a set / tuple / list, or the keys of a dict.
pub fn key_set(value: &Value) -> std::collections::HashSet<String> {
    match value {
        Value::Dict(pairs) => pairs.iter().filter_map(|(key, _)| key.as_str().map(str::to_string)).collect(),
        other => other.str_set(),
    }
}

/// `template.format(name=value, ...)` for `{name}` fields (and `{{` / `}}`), read in one pass so
/// a value is never itself read as a field. A field with no value is Python's KeyError.
pub fn fmt(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 64);
    let mut chars = template.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        match c {
            '{' if template[at + 1..].starts_with('{') => {
                chars.next();
                out.push('{');
            }
            '}' if template[at + 1..].starts_with('}') => {
                chars.next();
                out.push('}');
            }
            '{' => {
                let close = template[at..].find('}').unwrap_or_else(|| panic!("ValueError: expected '}}' in {template:?}")) + at;
                let name = &template[at + 1..close];
                match fields.iter().find(|(key, _)| *key == name) {
                    Some((_, value)) => out.push_str(value),
                    None => panic!("KeyError: '{name}'"),
                }
                while chars.peek().is_some_and(|(next, _)| *next <= close) {
                    chars.next();
                }
            }
            other => out.push(other),
        }
    }
    out
}

/// `len(memory.get("turns") or [])`.
pub fn turn_count(memory: &Memory) -> i64 {
    memory.turns.len() as i64
}

/// `str(value or "")`: "" for anything falsy, the string itself, or Python's `str()` of it.
pub fn text_of(value: Option<&Value>) -> String {
    match value {
        Some(v) if v.truthy() => crate::memory::facts::py_str(v),
        _ => String::new(),
    }
}

/// `str(value)`.
pub fn py_str(value: &Value) -> String {
    crate::memory::facts::py_str(value)
}

/// `d.get(key)` on a value that may not be a dict (None for anything else).
pub fn get<'a>(value: &'a Value, key: &str) -> &'a Value {
    static NONE: Value = Value::None;
    value.get(key).unwrap_or(&NONE)
}

/// `d.get(key)` as a str ("" when it is missing or not a str).
pub fn get_str<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).map_or("", Value::str)
}

/// `memory.get(key)`, with None for a missing key.
pub fn mem<'a>(memory: &'a Memory, key: &str) -> &'a Value {
    static NONE: Value = Value::None;
    memory.get(key).unwrap_or(&NONE)
}

/// `int(value)` for the values memory holds. A str that is not a number is Python's ValueError.
pub fn int(value: &Value) -> i64 {
    match value {
        Value::Int(i) => *i,
        Value::Bool(b) => *b as i64,
        Value::Float(f) => f.trunc() as i64,
        Value::Str(s) => int_str(s).unwrap_or_else(|| panic!("ValueError: invalid literal for int() with base 10: {s:?}")),
        other => panic!("TypeError: int() argument must be a string or a number, not {other:?}"),
    }
}

/// `int(text)`: optional whitespace, sign and underscores between digits; None for ValueError.
pub fn int_str(text: &str) -> Option<i64> {
    let trimmed = pt::strip(text);
    let (sign, digits) = match trimmed.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };
    if digits.is_empty() || digits.starts_with('_') || digits.ends_with('_') || digits.contains("__") {
        return None;
    }
    let mut total: i64 = 0;
    for c in digits.chars() {
        if c == '_' {
            continue;
        }
        total = total.checked_mul(10)?.checked_add(c.to_digit(10)? as i64)?;
    }
    Some(sign * total)
}

/// `int(value or 0)`.
pub fn int_or_0(value: Option<&Value>) -> i64 {
    match value {
        Some(v) if v.truthy() => int(v),
        _ => 0,
    }
}

/// The byte offset `count` characters before `position` (Python's `max(0, position - count)`).
pub fn chars_back(text: &str, position: usize, count: usize) -> usize {
    text[..position].char_indices().rev().take(count).last().map_or(position, |(at, _)| at)
}

/// The byte offset `count` characters after `position` (Python's `position + count`, clamped).
pub fn chars_on(text: &str, position: usize, count: usize) -> usize {
    text[position..].char_indices().nth(count).map_or(text.len(), |(at, _)| position + at)
}

/// `text[:count]` counted in characters.
pub fn first_chars(text: &str, count: usize) -> &str {
    &text[..chars_on(text, 0, count)]
}

/// `text[-count:]` counted in characters.
pub fn last_chars(text: &str, count: usize) -> &str {
    if count == 0 {
        return text; // `text[-0:]` is the whole string
    }
    &text[chars_back(text, text.len(), count)..]
}

/// `text[count:]` counted in characters.
pub fn from_char(text: &str, count: usize) -> &str {
    &text[chars_on(text, 0, count)..]
}

/// `s[:1].upper() + s[1:]`.
pub fn up1(text: &str) -> String {
    pt::upper_first(text)
}

/// `s[:1].lower() + s[1:]`.
pub fn low1(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().chain(chars).collect(),
    }
}

/// `seq[-count:]` for a slice.
pub fn tail<T>(items: &[T], count: usize) -> &[T] {
    if count == 0 {
        return items; // `seq[-0:]` is the whole sequence
    }
    &items[items.len().saturating_sub(count)..]
}

/// The user's previous line (`last_user(memory) or ""`).
pub fn last_user(memory: &Memory) -> String {
    text_of(Some(&crate::memory::last_user(memory, 1)))
}

/// The bot's previous line (`last_bot(memory) or ""`).
pub fn last_bot(memory: &Memory) -> String {
    crate::memory::last_bot(memory)
}

/// A `{"key": value, ...}` dict from pairs, in order.
pub fn dict(pairs: Vec<(&str, Value)>) -> Value {
    let mut out = Value::dict();
    for (key, value) in pairs {
        out.set_key(key, value);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_idioms() {
        assert_eq!(fmt("{a} and {b}{{x}}", &[("a", "1"), ("b", "{a}")]), "1 and {a}{x}");
        assert_eq!(int_str(" 12 "), Some(12));
        assert_eq!(int_str("-1_0"), Some(-10));
        assert_eq!(int_str("1.5"), None);
        assert_eq!(first_chars("héllo", 2), "hé");
        assert_eq!(last_chars("héllo", 4), "éllo");
        assert_eq!(last_chars("ab", 5), "ab");
        assert_eq!(chars_back("héllo", 3, 1), 1);
        assert_eq!(low1("Élan"), "élan");
    }
}

/// `bot.memory.user(memory)`: the user's facts (a dict), created on first use. A copy of the
/// three-line PyBot helper, so the MEM7 files do not wait on it (its stub in `memory/mod.rs` is
/// typed for another shape).
pub fn user(memory: &mut Memory) -> &mut Value {
    if !memory.get("user").is_some_and(Value::is_dict) {
        memory.set("user", Value::dict());
    }
    memory.get_mut("user").expect("just set")
}

/// `bot.memory.others(memory)`: the facts about other people, a list, created on first use. A
/// copy of the three-line PyBot helper (its stub in `memory/mod.rs` returns `Vec<String>`, which
/// cannot carry the dicts the list holds).
pub fn others(memory: &mut Memory) -> Vec<Value> {
    match memory.get("others") {
        Some(Value::List(seen)) => seen.to_vec(),
        _ => {
            memory.set("others", Value::list());
            Vec::new()
        }
    }
}

/// `entry.get(key)` on an item of a list memory holds: Python's AttributeError when the item is
/// not a dict.
pub fn item_get<'a>(entry: &'a Value, key: &str) -> &'a Value {
    if !entry.is_dict() {
        panic!("AttributeError: {entry:?} has no attribute 'get'");
    }
    get(entry, key)
}

/// Python's `str.islower()`: at least one cased character, and none upper-case.
pub fn is_lower(text: &str) -> bool {
    let mut cased = false;
    for c in text.chars() {
        if c.is_uppercase() || (c.is_alphabetic() && !c.is_lowercase() && c.to_lowercase().next() != Some(c)) {
            return false;
        }
        if c.is_lowercase() {
            cased = true;
        }
    }
    cased
}

/// `for x in value` for the values memory holds: a list's items, a dict's keys, a str's
/// characters; nothing for None. Anything else is Python's TypeError.
pub fn iterate(value: &Value) -> Vec<Value> {
    match value {
        Value::None => Vec::new(),
        Value::List(items) | Value::Tuple(items) | Value::Set(items) => items.to_vec(),
        Value::Dict(pairs) => pairs.iter().map(|(key, _)| key.clone()).collect(),
        Value::Str(text) => text.chars().map(|c| Value::from(c.to_string())).collect(),
        other => panic!("TypeError: {other:?} is not iterable"),
    }
}

/// `x or ()` iterated: a falsy value is nothing.
pub fn iterate_or_empty(value: Option<&Value>) -> Vec<Value> {
    match value {
        Some(v) if v.truthy() => iterate(v),
        _ => Vec::new(),
    }
}

/// `isinstance(x, int)` (a bool is an int in Python) and its value.
pub fn as_int(value: Option<&Value>) -> Option<i64> {
    value.and_then(Value::as_int)
}

/// `str(one.get(key) or "").lower()`.
pub fn lower_of(entry: &Value, key: &str) -> String {
    pt::lower(&text_of(entry.get(key)))
}
