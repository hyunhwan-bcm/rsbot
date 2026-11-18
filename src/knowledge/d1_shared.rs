//! Helpers the D1 knowledge modules share: a reading's fields by their Python key, the
//! `x or y` string idioms the guards are written in, cached dynamic regexes, and minimal
//! ports (marked `DUP`) of helpers that live in modules other workstreams own.
//!
//! A `DUP(<python module>::<name>)` function is a private port of the original helper;
//! the integrator replaces it with the owning workstream's version.

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

/// `str(understood.get(key) or "")` for the reading's string fields ("" for None or a key the
/// reading does not have).
pub fn field<'a>(u: &'a Understood, key: &str) -> std::borrow::Cow<'a, str> {
    use std::borrow::Cow;
    fn o(x: &Option<String>) -> &str {
        x.as_deref().unwrap_or("")
    }
    Cow::Borrowed(match key {
        "text" => &u.text,
        "clean" => &u.clean,
        "plain" => &u.plain,
        "norm_text" => &u.norm_text,
        "kind" => &u.kind,
        "intent" => &u.intent,
        "animal_in_hand" => &u.animal_in_hand,
        "topic" => o(&u.topic),
        "subject" => o(&u.subject),
        "body" => o(&u.body),
        "own_output" => return Cow::Owned(u.own_output.filter(|n| *n != 0).map(|n| n.to_string()).unwrap_or_default()),
        "setting" => o(&u.setting),
        "shift" => o(&u.shift),
        "emotion" => o(&u.emotion),
        "ask_subject" => o(&u.ask_subject),
        "leans_on" => return Cow::Owned(u.leans_on.as_ref().map(|l| l.py_repr()).unwrap_or_default()),
        "subject_from" => o(&u.subject_from),
        "subject_unclear" => o(&u.subject_unclear),
        "shortage" => o(&u.shortage),
        "state_owner" => o(&u.state_owner),
        "stand_in" => return Cow::Owned(u.stand_in.as_ref().map(|l| l.py_repr()).unwrap_or_default()),
        "warmth" => o(&u.warmth),
        "wants" => o(&u.wants),
        _ => "",
    })
}

/// `field` for the reading's plain string fields (text, clean, plain, ...).
pub fn str_field<'a>(u: &'a Understood, key: &str) -> &'a str {
    match key {
        "text" => &u.text,
        "clean" => &u.clean,
        "plain" => &u.plain,
        "norm_text" => &u.norm_text,
        "kind" => &u.kind,
        "intent" => &u.intent,
        _ => "",
    }
}

/// Python's `a or b` on strings.
pub fn or<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.is_empty() { b } else { a }
}

/// `str(understood.get("clean") or understood.get("text") or "")`.
pub fn clean_or_text(u: &Understood) -> &str {
    or(&u.clean, &u.text)
}

/// `str(understood.get("text") or understood.get("clean") or "")`.
pub fn text_or_clean(u: &Understood) -> &str {
    or(&u.text, &u.clean)
}

/// `" ".join(str(w) for w in understood.get("words") or ())`.
pub fn words_line(u: &Understood) -> String {
    u.words.join(" ")
}

/// `s.lower()`.
pub fn lower(s: &str) -> String {
    s.to_lowercase()
}

/// `" ".join(s.lower().split())`.
pub fn squash_lower(s: &str) -> String {
    text::squash(&s.to_lowercase())
}

/// `" " + s + " "`.
pub fn padded(s: &str) -> String {
    format!(" {s} ")
}

/// `any(word in text for word in words)`.
pub fn any_in<S: AsRef<str>>(text: &str, words: &[S]) -> bool {
    words.iter().any(|w| text.contains(w.as_ref()))
}

/// `any(" " + word + " " in text for word in words)`.
pub fn any_word_in<S: AsRef<str>>(text: &str, words: &[S]) -> bool {
    words.iter().any(|w| text.contains(&format!(" {} ", w.as_ref())))
}

/// Python's `string.punctuation`.
pub const PUNCTUATION: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

/// `word.strip(string.punctuation)`.
pub fn strip_punct(word: &str) -> &str {
    text::strip_chars(word, PUNCTUATION)
}

/// `re.escape(word)`.
pub fn escape(word: &str) -> String {
    crate::py::re::escape(word)
}

/// A regex built from a pattern made at call time (`re.search(r"\b" + word + r"\b", text)`),
/// compiled once per distinct pattern as Python's own `re` cache would.
pub fn re_cached(pattern: &str, flags: u32) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<(String, u32), &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let key = (pattern.to_string(), flags);
    if let Some(r) = CACHE.lock().unwrap().get(&key) {
        return r;
    }
    let r: &'static Re = Box::leak(Box::new(Re::new(pattern, flags)));
    CACHE.lock().unwrap().insert(key, r);
    r
}

/// `re.search(pattern, text, flags)` for a pattern made at call time.
pub fn re_search(pattern: &str, text: &str, flags: u32) -> bool {
    re_cached(pattern, flags).is_match(text)
}

/// A string global as a set, for `in` tests.
pub fn set_of(name: &str) -> HashSet<String> {
    data::set(name)
}

/// `bool(guard(reading))` for a guard the bundle holds as a function value (a closure's cell).
pub fn call_guard(f: &Value, reading: &Understood) -> bool {
    match f.as_fn() {
        Some(f) => crate::knowledge::registry::guard(f, reading),
        None => false,
    }
}

// ---------------------------------------------------------------------------------------------
// DUP ports of helpers owned by other workstreams (full original logic).

/// DUP(bot.memory::last_user): the person's previous line; "" on a single-turn reply.
pub fn last_user(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.user.clone()).unwrap_or_default()
}

// was DUP(bot.knowledge.lookup::the_line): now the real port
pub use crate::knowledge::lookup::the_line;

// was DUP(bot.knowledge.lookup::a_sanding_job): now the real port
pub use crate::knowledge::lookup::a_sanding_job;

// was DUP(bot.knowledge.lookup::said_text): now the real port
pub use crate::knowledge::lookup::said_text;

// was DUP(bot.knowledge.lookup::not_the_heat_of_an_argument): now the real port
pub use crate::knowledge::lookup::not_the_heat_of_an_argument;

// was DUP(bot.knowledge.health::_the_line): now the real port
pub use crate::knowledge::health::_the_line as health_the_line;

// ---------------------------------------------------------------------------------------------
// Entry callables other than guard/reply. The registry runs guards and replies; an entry's
// `shape` and `lead` are functions too (`Entry.shape`: given the line's words as one string,
// true when it holds the row's figure; `Entry.lead`: the sentence the row owes the line, "" for
// none). A module that has any exports `SHAPES` / `LEADS` tables keyed by id, like `GUARDS`.

/// `shape(text) -> bool`.
pub type ShapeFn = fn(&crate::data::FnRef, &str) -> bool;
/// `lead(reading) -> str`.
pub type LeadFn = fn(&crate::data::FnRef, &Understood) -> String;
