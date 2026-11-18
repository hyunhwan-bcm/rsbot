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

// DUP(bot.knowledge.geography::_bare_place)
/// The place words after a direction, without the article in front of them.
pub fn _bare_place(phrase: &str) -> String {
    let mut phrase = crate::py::text::strip_chars(crate::py::text::strip(phrase), ".,!?;:").to_string();
    loop {
        let shorter = _ARTICLE.subn("", &phrase, 1);
        if shorter == phrase {
            return phrase;
        }
        phrase = shorter;
    }
}

// DUP(bot.knowledge.geography::directed_place)
/// The place a line carries something towards, in its own words, or None.
pub fn directed_place(line: &str) -> Option<String> {
    for m in _DIRECTION.finditer(line) {
        let place = _bare_place(m.str(1));
        if !place.is_empty() && !_NOT_A_PLACE.contains(&place) {
            return Some(place);
        }
    }
    None
}

// DUP(bot.knowledge.geography::_is_everyday)
/// True when the place words name a room, a building or a do rather than a country.
pub fn _is_everyday(place: &str) -> bool {
    let place = _bare_place(place);
    _EVERYDAY_PLACE.iter().any(|name| {
        place == *name || place.starts_with(&format!("{name} ")) || place.ends_with(&format!(" {name}"))
    })
}

// DUP(bot.knowledge.geography::crosses_a_border)
/// True when the line carries something across a border rather than to a do.
pub fn crosses_a_border(line: &str) -> bool {
    let line = line.to_lowercase();
    if _CROSSING_WORD.is_match(&line) {
        return true;
    }
    match directed_place(&line) {
        Some(place) => !_is_everyday(&place),
        None => false,
    }
}

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

// NEEDS(bot.compose.loss::dying_in_the_chat)
/// True when a line earlier in this chat said that someone of ours is dying. Exact with no
/// earlier turns (False); with turns it needs `loss.is_dying_now`, which is not ported here, so
/// it says False (the most neutral reading: the reply goes ahead).
pub fn dying_in_the_chat(memory: &Memory) -> bool {
    let _ = memory.turns.len();
    false
}

// NEEDS(bot.compose.loss::reads_as_loss)
/// True when the line is about a death or a loss of a person or a pet. Not ported here (a large
/// reader in compose.loss): False, so a reply goes ahead.
pub fn reads_as_loss(text: &str) -> bool {
    let _ = text;
    false
}

// ---------------------------------------------------------------------------------------------
// DUP(bot.knowledge.dishes::cook_ask)

globals!("bot.knowledge.dishes", strs { _COOK_PHRASES, FOOD_WORDS });

// DUP(bot.knowledge.dishes::cook_ask)
/// True when the line asks what to make or eat with a food it names.
pub fn cook_ask(understood: &Understood) -> bool {
    let text = if understood.norm_text.is_empty() { &understood.clean } else { &understood.norm_text }.to_lowercase();
    if !_COOK_PHRASES.iter().any(|w| text.contains(w.as_str())) {
        return false;
    }
    FOOD_WORDS.iter().any(|f| text.contains(f.as_str()))
}

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

// DUP(bot.memory::last_user)
/// The user's last line before this turn ("" when there is none).
pub fn last_user(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.user.clone()).unwrap_or_default()
}

/// TYPE(leans_on): on a follow-up Python's `reading["leans_on"]` is a dict ("text", "subject",
/// "topic", "words", ...); the shared typed reading holds `leans_on: Option<String>`, which
/// Python's `isinstance(leans, dict)` tests would turn away. Until the shared type carries the
/// dict, every field of it reads as absent here; the logic that reads it is ported in full.
pub fn leans_on_field<'a>(understood: &'a Understood, key: &str) -> Option<&'a str> {
    let l = understood.leans_on.as_ref()?;
    match key {
        "text" => Some(l.text.as_str()),
        "subject" => l.subject.as_deref(),
        "subject_type" => l.subject_type.as_deref(),
        "topic" => l.topic.as_deref(),
        _ => None,
    }
}

/// `leans_on["words"]` (see `leans_on_field`): absent until the shared type carries the dict.
pub fn leans_on_words(understood: &Understood) -> Option<Vec<String>> {
    understood.leans_on.as_ref().map(|l| l.words.clone())
}
