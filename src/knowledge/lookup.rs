//! Port of `bot/knowledge/lookup.py`: the shared matcher that finds the knowledge entry a line
//! asks for (`find`), and the `Entry` every domain builds its knowledge from.

use crate::data::{FnRef, Value};
use crate::understood::Understood;
use std::sync::Arc;

/// One part of an entry's `key` (or `ask`): a phrase, or a tuple of alternative phrases.
#[derive(Clone, Debug, PartialEq)]
pub enum KeyPart {
    One(String),
    Any(Vec<String>),
}

impl KeyPart {
    fn from_value(v: &Value) -> Option<KeyPart> {
        match v {
            Value::Str(s) => Some(KeyPart::One(s.to_string())),
            Value::Tuple(_) | Value::List(_) | Value::Set(_) => Some(KeyPart::Any(v.strs())),
            _ => None,
        }
    }
    /// The alternatives (`part if isinstance(part, tuple) else (part,)`).
    pub fn alternatives(&self) -> &[String] {
        match self {
            KeyPart::One(s) => std::slice::from_ref(s),
            KeyPart::Any(v) => v,
        }
    }
    pub fn is_group(&self) -> bool {
        matches!(self, KeyPart::Any(_))
    }
}

/// An entry's guard: a Python function (run through `knowledge::registry`), or a guard with one
/// more condition folded in by `bot/knowledge/__init__.py::_with_one_more_test`.
#[derive(Clone, Debug)]
pub enum Guard {
    Fn(Arc<FnRef>),
    And(Box<Guard>, Arc<FnRef>),
}

impl Guard {
    /// `bool(guard(reading))`; a guard that raises fails (as `guard_passes` treats it).
    pub fn passes(&self, reading: &Understood) -> bool {
        match self {
            Guard::Fn(f) => crate::knowledge::registry::guard(f, reading),
            Guard::And(g, extra) => g.passes(reading) && crate::knowledge::registry::guard(extra, reading),
        }
    }
}

/// `bot.knowledge.lookup.Entry`: one piece of knowledge, keyed by meaning.
#[derive(Clone, Debug)]
pub struct Entry {
    pub id: String,
    pub key: Vec<KeyPart>,
    /// A string, a function (`Value::Fn`), or a tuple/list of strings.
    pub reply: Value,
    pub avoid: Vec<String>,
    pub bias: i64,
    pub ask: Vec<KeyPart>,
    pub slips: Vec<String>,
    pub domain: String,
    pub fault: bool,
    pub report: bool,
    pub guard: Option<Guard>,
    pub shape: Vec<Arc<FnRef>>,
    pub explain: bool,
    /// A string or a tuple of strings.
    pub lead: Value,
    pub outcomes: Value,
}

impl Entry {
    /// An Entry from its exported attributes (`{"obj": "Entry", "attrs": {...}}`).
    pub fn from_value(v: &Value) -> Option<Entry> {
        if v.class() != Some("Entry") {
            return None;
        }
        let a = |name: &str| v.attr(name).cloned().unwrap_or(Value::None);
        let parts = |name: &str| a(name).items().iter().filter_map(KeyPart::from_value).collect::<Vec<_>>();
        Some(Entry {
            id: a("id").str().to_string(),
            key: parts("key"),
            reply: a("reply"),
            avoid: a("avoid").strs(),
            bias: a("bias").as_int().unwrap_or(0),
            ask: parts("ask"),
            slips: a("slips").strs(),
            domain: a("domain").str().to_string(),
            fault: a("fault").truthy(),
            report: a("report").truthy(),
            guard: a("guard").as_fn().map(|f| Guard::Fn(Arc::new(f.clone()))),
            shape: a("shape").items().iter().filter_map(|x| x.as_fn().map(|f| Arc::new(f.clone()))).collect(),
            explain: a("explain").truthy(),
            lead: a("lead"),
            outcomes: a("outcomes"),
        })
    }
}

/// Every entry in the knowledge base (`bot.knowledge.entries()`).
pub fn all_entries() -> &'static [Entry] {
    crate::knowledge::entries()
}

/// `guard_passes(item, reading)`.
pub fn guard_passes(item: &Entry, reading: &Understood) -> bool {
    match &item.guard {
        None => true,
        Some(g) => g.passes(reading),
    }
}

/// `tokens(text)`: the lower-cased words of a text.
pub fn tokens(text: &str) -> Vec<String> {
    let _ = text;
    todo!("lookup::tokens")
}

/// `find(understood, entries=None, reject=None, use_context=False)`: the entry the line asks
/// for, or None. `entries` None means every entry; `reject` turns an entry down.
pub fn find(understood: &Understood, entries: Option<&[&'static Entry]>, reject: Option<&dyn Fn(&Entry) -> bool>, use_context: bool) -> Option<&'static Entry> {
    let _ = (understood, entries, reject, use_context);
    todo!("lookup::find")
}

/// What `_vocabulary(entries)` works out: each domain's key words with how many entries use
/// them, the same over every domain, and every form of those words.
#[derive(Clone, Debug, Default)]
pub struct Vocabulary {
    pub by_domain: std::collections::HashMap<String, std::collections::HashMap<String, usize>>,
    pub every: std::collections::HashMap<String, usize>,
    pub known: std::collections::HashSet<String>,
}

/// `_vocabulary(entries)`: `(by_domain, every, known)`.
pub fn _vocabulary(entries: &[&'static Entry]) -> Arc<Vocabulary> {
    let _ = entries;
    todo!("lookup::_vocabulary")
}
