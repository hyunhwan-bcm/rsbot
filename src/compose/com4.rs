//! Small helpers shared by the compose files of the COM4 workstream: bundle statics by Python
//! name, and the Python idioms those files repeat.
#![allow(dead_code, unused_macros, unused_imports)]

use crate::data::Value;
use crate::py::text;
use crate::state::Memory;

/// `static NAME: &Re` for module-level compiled regexes of one Python module.
macro_rules! bundle_res {
    ($module:literal; $($name:ident),* $(,)?) => {
        $(pub static $name: std::sync::LazyLock<&'static crate::py::re::Re> =
            std::sync::LazyLock::new(|| crate::data::re(concat!($module, ".", stringify!($name))));)*
    };
}
/// `static NAME: HashSet<String>` for module-level (frozen)sets of strings.
macro_rules! bundle_sets {
    ($module:literal; $($name:ident),* $(,)?) => {
        $(pub static $name: std::sync::LazyLock<std::collections::HashSet<String>> =
            std::sync::LazyLock::new(|| crate::data::set(concat!($module, ".", stringify!($name))));)*
    };
}
/// `static NAME: Vec<String>` for module-level tuples / lists of strings, in order.
macro_rules! bundle_strs {
    ($module:literal; $($name:ident),* $(,)?) => {
        $(pub static $name: std::sync::LazyLock<Vec<String>> =
            std::sync::LazyLock::new(|| crate::data::strs(concat!($module, ".", stringify!($name))));)*
    };
}
/// `static NAME: String` for module-level strings.
macro_rules! bundle_str {
    ($module:literal; $($name:ident),* $(,)?) => {
        $(pub static $name: std::sync::LazyLock<String> =
            std::sync::LazyLock::new(|| crate::data::string(concat!($module, ".", stringify!($name))));)*
    };
}
/// `static NAME: &Value` for any other module-level value.
macro_rules! bundle_vals {
    ($module:literal; $($name:ident),* $(,)?) => {
        $(pub static $name: std::sync::LazyLock<&'static crate::data::Value> =
            std::sync::LazyLock::new(|| crate::data::get(concat!($module, ".", stringify!($name))));)*
    };
}
/// `static NAME: Vec<&Re>` for module-level tuples of compiled regexes.
macro_rules! bundle_re_lists {
    ($module:literal; $($name:ident),* $(,)?) => {
        $(pub static $name: std::sync::LazyLock<Vec<&'static crate::py::re::Re>> = std::sync::LazyLock::new(|| {
            crate::data::get(concat!($module, ".", stringify!($name)))
                .items()
                .iter()
                .map(|v| v.as_re().expect(concat!(stringify!($name), " holds compiled patterns")))
                .collect()
        });)*
    };
}
/// A string pattern (a module-level str) compiled once, as `re.search(PATTERN, ...)` does.
macro_rules! bundle_str_res {
    ($module:literal; $($name:ident),* $(,)?) => {
        $(pub static $name: std::sync::LazyLock<crate::py::re::Re> =
            std::sync::LazyLock::new(|| crate::py::re::Re::new(crate::data::get(concat!($module, ".", stringify!($name))).str(), 0));)*
    };
}
/// A regex written inline in a function body.
macro_rules! inline_re {
    ($name:ident, $pat:expr) => {
        static $name: std::sync::LazyLock<crate::py::re::Re> = std::sync::LazyLock::new(|| crate::py::re::Re::new($pat, 0));
    };
    ($name:ident, $pat:expr, $flags:expr) => {
        static $name: std::sync::LazyLock<crate::py::re::Re> = std::sync::LazyLock::new(|| crate::py::re::Re::new($pat, $flags));
    };
}
pub(crate) use {bundle_re_lists, bundle_res, bundle_sets, bundle_str, bundle_str_res, bundle_strs, bundle_vals, inline_re};

/// `" ".join(str(text or "").lower().split())`.
pub fn flat(text: &str) -> String {
    text::squash(&text.to_lowercase())
}

/// `seq[-n:]` with Python's rules (`n == 0` is the whole list, a negative `n` skips from the front).
pub fn tail<T>(seq: &[T], n: i64) -> &[T] {
    let len = seq.len() as i64;
    let start = if n == 0 {
        0
    } else if n > 0 {
        (len - n).max(0)
    } else {
        (-n).min(len)
    };
    &seq[start as usize..]
}

/// `seq[max(0, i - window):i]`.
pub fn before<T>(seq: &[T], i: usize, window: i64) -> &[T] {
    let lo = (i as i64 - window).max(0).min(i as i64) as usize;
    &seq[lo..i]
}

/// The memory as the Python dict it is, for a callee whose contract takes a `Value`.
pub fn memory_value(memory: &Memory) -> Value {
    Value::from_plain(&serde_json::Value::Object(memory.to_plain()))
}

/// `(turn.get("bot") or "")` of the last turn, or "" when there is none.
pub fn last_bot(memory: &Memory) -> &str {
    memory.turns.last().map(|t| t.bot.as_str()).unwrap_or("")
}

/// `(turn.get("user") or "")` of the last turn, or "" when there is none.
pub fn last_user(memory: &Memory) -> &str {
    memory.turns.last().map(|t| t.user.as_str()).unwrap_or("")
}

/// `sum(ord(c) for c in text)`.
pub fn ord_sum(text: &str) -> i128 {
    crate::py::random::ord_sum(text)
}

/// `[s.to_string() ...]` for a pool written as literals.
pub fn owned(options: &[&str]) -> Vec<String> {
    options.iter().map(|s| s.to_string()).collect()
}

/// `catch_unwind` for a Python `try: ... except Exception:` block: a real failure gives None
/// (the caller's fallback), while a call into a function nobody has ported yet still propagates,
/// so it is counted as unported rather than read as a Python exception.
pub fn attempt<T>(f: impl FnOnce() -> T) -> Option<T> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(v) => Some(v),
        Err(e) => {
            let msg = e.downcast_ref::<String>().map(String::as_str).or_else(|| e.downcast_ref::<&str>().copied()).unwrap_or("");
            if msg.starts_with("UNPORTED") {
                std::panic::resume_unwind(e)
            }
            None
        }
    }
}

pub(crate) use crate::compose::fallback::format_named;

static WS_RUN: std::sync::LazyLock<crate::py::re::Re> = std::sync::LazyLock::new(|| crate::py::re::Re::new(r"\s+", 0));

/// `re.sub(r"\s+", " ", text).strip()`.
pub fn ws_one(text: &str) -> String {
    text::strip(&WS_RUN.sub(" ", text)).to_string()
}

/// `re.sub(r"\s+", " ", text)` (no strip).
pub fn ws_sub(text: &str) -> String {
    WS_RUN.sub(" ", text)
}

/// `text.replace("’", "'")`.
pub fn straight(text: &str) -> String {
    text.replace('\u{2019}', "'")
}

/// `(memory.get("understood") or {}).get("clean") or ""`.
pub fn understood_clean(memory: &Memory) -> &str {
    memory.understood.as_ref().map(|u| u.clean.as_str()).unwrap_or("")
}

/// `any(pattern.search(text) for pattern in patterns)`.
pub fn any_search(patterns: &[&crate::py::re::Re], text: &str) -> bool {
    patterns.iter().any(|p| p.search(text).is_some())
}
