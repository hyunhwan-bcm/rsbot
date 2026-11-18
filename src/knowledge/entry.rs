//! The knowledge base's rows: PyBot's `Entry` (bot/knowledge/lookup.py) and the canonical list
//! `bot.knowledge.entries()` returns, loaded from the data bundle.
//!
//! `entries()` is the matcher's view: every domain's ENTRIES in domain order, each with its
//! `domain` set and the domain's STANDS_ASIDE / GUARD_FOLDS tests already folded into `guard`
//! (exported after Python's `entries()` ran, as `bot.knowledge.__entries__`). Python compares
//! entries by identity; here an entry is `&'static Entry` from that one list, so pointer
//! equality (`std::ptr::eq`) or `index` is identity, and `id` is unique too.

use crate::data::{self, FnRef, Value};
use std::collections::HashMap;
use std::sync::LazyLock;

/// One key position: a word, or a tuple of words any of which fills the position.
#[derive(Clone, Debug, PartialEq)]
pub enum KeyPart {
    Word(String),
    Any(Vec<String>),
}

impl KeyPart {
    /// The words this position accepts.
    pub fn words(&self) -> Vec<&str> {
        match self {
            KeyPart::Word(w) => vec![w.as_str()],
            KeyPart::Any(v) => v.iter().map(String::as_str).collect(),
        }
    }
}

/// What the entry says: a fixed reply, a function of (memory, reading), or a pool to pick from.
#[derive(Clone, Debug)]
pub enum Reply {
    Text(String),
    Fn(FnRef),
    /// A tuple / list of lines; `compose.pick` chooses one.
    Choices(Vec<String>),
}

/// `lead`: a sentence put in front when the line wants a shape (`""` = none). In Python it is a
/// str, or a tuple (shape name or shape function, sentence).
#[derive(Clone, Debug)]
pub enum Lead {
    Text(String),
    Shaped(Vec<Value>),
}

/// An `Entry`. Field names and meanings are Python's (see lookup.py's class docstrings).
#[derive(Clone, Debug)]
pub struct Entry {
    /// Position in `entries()`: identity, as Python's `is`.
    pub index: usize,
    pub id: String,
    pub key: Vec<KeyPart>,
    pub reply: Reply,
    pub avoid: Vec<String>,
    pub bias: i64,
    pub ask: Vec<KeyPart>,
    pub slips: Vec<String>,
    pub domain: String,
    pub fault: bool,
    pub report: bool,
    pub guard: Option<FnRef>,
    /// Figure-shape readers (functions), each worth one key word.
    pub shape: Vec<FnRef>,
    pub explain: bool,
    pub lead: Lead,
    /// (shape asked, sentence owed).
    pub outcomes: Vec<(String, String)>,
    /// The exported object, for anything not modelled above.
    pub raw: Value,
}

fn key_parts(v: &Value) -> Vec<KeyPart> {
    v.items()
        .iter()
        .map(|x| match x {
            Value::Str(s) => KeyPart::Word(s.to_string()),
            other => KeyPart::Any(other.strs()),
        })
        .collect()
}

impl Entry {
    /// An entry from its exported object.
    pub fn from_value(index: usize, v: &Value) -> Entry {
        let a = |name: &str| v.attr(name).cloned().unwrap_or(Value::None);
        let reply = match a("reply") {
            Value::Str(s) => Reply::Text(s.to_string()),
            Value::Fn(f) => Reply::Fn((*f).clone()),
            other => Reply::Choices(other.strs()),
        };
        let lead = match a("lead") {
            Value::Str(s) => Lead::Text(s.to_string()),
            other => Lead::Shaped(other.items().to_vec()),
        };
        Entry {
            index,
            id: a("id").str().to_string(),
            key: key_parts(&a("key")),
            reply,
            avoid: a("avoid").strs(),
            bias: a("bias").as_int().unwrap_or(0),
            ask: key_parts(&a("ask")),
            slips: a("slips").strs(),
            domain: a("domain").str().to_string(),
            fault: a("fault").truthy(),
            report: a("report").truthy(),
            guard: a("guard").as_fn().cloned(),
            shape: a("shape").items().iter().filter_map(|x| x.as_fn().cloned()).collect(),
            explain: a("explain").truthy(),
            lead,
            outcomes: a("outcomes")
                .items()
                .iter()
                .map(|t| {
                    let it = t.items();
                    (it.first().map(|x| x.str().to_string()).unwrap_or_default(), it.get(1).map(|x| x.str().to_string()).unwrap_or_default())
                })
                .collect(),
            raw: v.clone(),
        }
    }
}

static ENTRIES: LazyLock<Vec<Entry>> = LazyLock::new(|| {
    data::get("bot.knowledge.__entries__").items().iter().enumerate().map(|(i, v)| Entry::from_value(i, v)).collect()
});

static BY_ID: LazyLock<HashMap<&'static str, &'static Entry>> =
    LazyLock::new(|| ENTRIES.iter().map(|e| (e.id.as_str(), e)).collect());

/// `bot.knowledge.entries()`: every entry, in the matcher's order.
pub fn entries() -> &'static [Entry] {
    &ENTRIES
}

/// The entry with this id (ids are unique across the knowledge base).
pub fn by_id(id: &str) -> Option<&'static Entry> {
    BY_ID.get(id).copied()
}

/// The canonical entry for an exported Entry object found in some other table (by id).
pub fn canonical(v: &Value) -> Option<&'static Entry> {
    by_id(v.attr("id")?.str())
}
