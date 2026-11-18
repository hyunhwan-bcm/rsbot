//! Group g10's helpers: a macro for module globals read from the bundle, and private ports of
//! helpers owned by other workstreams (each marked DUP / DUP-D2 / NEEDS).

/// Module globals from the bundle, by Python name, in `LazyLock` statics:
/// `globals!("bot.knowledge.icing", re { _FROSTING_UP }, string { ICING_TEXT })`.
macro_rules! globals {
    ($prefix:literal $(, $kind:ident { $($name:ident),* $(,)? })* $(,)?) => {
        $( $( $crate::knowledge::d2dup_g10::globals!(@one $prefix, $kind, $name); )* )*
    };
    (@one $prefix:literal, re, $name:ident) => {
        static $name: std::sync::LazyLock<&'static $crate::py::re::Re> =
            std::sync::LazyLock::new(|| $crate::data::re(concat!($prefix, ".", stringify!($name))));
    };
    (@one $prefix:literal, string, $name:ident) => {
        static $name: std::sync::LazyLock<String> =
            std::sync::LazyLock::new(|| $crate::data::string(concat!($prefix, ".", stringify!($name))));
    };
    (@one $prefix:literal, strs, $name:ident) => {
        static $name: std::sync::LazyLock<Vec<String>> =
            std::sync::LazyLock::new(|| $crate::data::strs(concat!($prefix, ".", stringify!($name))));
    };
    (@one $prefix:literal, set, $name:ident) => {
        static $name: std::sync::LazyLock<std::collections::HashSet<String>> =
            std::sync::LazyLock::new(|| $crate::data::set(concat!($prefix, ".", stringify!($name))));
    };
    (@one $prefix:literal, value, $name:ident) => {
        static $name: std::sync::LazyLock<&'static $crate::data::Value> =
            std::sync::LazyLock::new(|| $crate::data::get(concat!($prefix, ".", stringify!($name))));
    };
}
pub(crate) use globals;

use crate::state::Memory;
use crate::understood::Understood;

/// `str(memory.get(key) or "")` for a memory key that holds a string.
pub fn memory_str(memory: &Memory, key: &str) -> String {
    match memory.extra.get(key) {
        Some(crate::data::Value::Str(s)) => s.to_string(),
        Some(crate::data::Value::Int(i)) if *i != 0 => i.to_string(),
        _ => String::new(),
    }
}

/// `str(reading.get("clean") or reading.get("text") or "")`.
pub fn clean_or_text(understood: &Understood) -> &str {
    if understood.clean.is_empty() { &understood.text } else { &understood.clean }
}

// ---------------------------------------------------------------------------------------------
// DUP(bot.knowledge.geography::crosses_a_border) and the helpers it reads

globals!("bot.knowledge.geography", re { _CROSSING_WORD, _DIRECTION, _ARTICLE }, strs { _EVERYDAY_PLACE, _NOT_A_PLACE });

// was DUP(bot.knowledge.geography::_bare_place): now the real port
pub use crate::knowledge::geography::_bare_place;

// was DUP(bot.knowledge.geography::directed_place): now the real port
pub use crate::knowledge::geography::directed_place;

// was DUP(bot.knowledge.geography::_is_everyday): now the real port
pub use crate::knowledge::geography::_is_everyday;

// was DUP(bot.knowledge.geography::crosses_a_border): now the real port
pub use crate::knowledge::geography::crosses_a_border;

// ---------------------------------------------------------------------------------------------
// Python's `re` module cache: `re.search(pattern, text)` with a pattern string built at run time.

static RE_CACHE: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<(String, u32), &'static crate::py::re::Re>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

/// The compiled regex for a pattern string (compiled once, like Python's `re` cache).
pub fn re_cached(pattern: &str, flags: u32) -> &'static crate::py::re::Re {
    let mut cache = RE_CACHE.lock().unwrap();
    if let Some(r) = cache.get(&(pattern.to_string(), flags)) {
        return r;
    }
    let r: &'static crate::py::re::Re = Box::leak(Box::new(crate::py::re::Re::new(pattern, flags)));
    cache.insert((pattern.to_string(), flags), r);
    r
}

// ---------------------------------------------------------------------------------------------
// bot/compose/loss.py (owned elsewhere; a large reader of death and dying)

/// `compose.loss.dying_in_the_chat(memory)` (the real port, `turns=6`).
pub fn dying_in_the_chat(memory: &Memory) -> bool {
    crate::compose::loss::dying_in_the_chat(memory, 6)
}

/// `compose.loss.reads_as_loss(text)` (the real port, `words=None, a_pet=False`).
pub fn reads_as_loss(text: &str) -> bool {
    crate::compose::loss::reads_as_loss(text, None, false)
}

// ---------------------------------------------------------------------------------------------
// DUP(bot.knowledge.dishes::cook_ask)

globals!("bot.knowledge.dishes", strs { _COOK_PHRASES, FOOD_WORDS });

// was DUP(bot.knowledge.dishes::cook_ask): now the real port
pub use crate::knowledge::dishes::cook_ask;

/// Python's `template % {"key": value, ...}` for `%(key)s` fields (and `%%`).
pub fn percent_named(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 32);
    let mut rest = template;
    while let Some(at) = rest.find('%') {
        out.push_str(&rest[..at]);
        let tail = &rest[at + 1..];
        if let Some(after) = tail.strip_prefix('%') {
            out.push('%');
            rest = after;
            continue;
        }
        if let Some(inner) = tail.strip_prefix('(') {
            let close = inner.find(')').expect("ValueError: incomplete format key");
            let key = &inner[..close];
            let spec = &inner[close + 1..];
            let value = values.iter().find(|(k, _)| *k == key).map(|(_, v)| *v).unwrap_or_else(|| panic!("KeyError: {key}"));
            assert!(spec.starts_with('s'), "unsupported format spec");
            out.push_str(value);
            rest = &spec[1..];
            continue;
        }
        panic!("TypeError: format requires a mapping");
    }
    out.push_str(rest);
    out
}

/// `memory.last_user(memory)` (the real port): the user's last line before this turn, or "".
pub fn last_user(memory: &Memory) -> String {
    crate::memory::last_user(memory, 1).str().to_string()
}

