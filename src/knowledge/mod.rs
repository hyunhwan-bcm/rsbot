//! Port of PyBot's `bot/knowledge` package (`bot/knowledge/__init__.py` is this file). Each Python
//! file `bot/knowledge/<name>.py` is `<name>.rs` here.
//!
//! The knowledge base is data: every domain module's `ENTRIES` (exported in the bundle as
//! `bot.knowledge.<domain>.ENTRIES`), gathered here in Python's order by [`entries`].

pub mod lookup;
pub mod registry;

use crate::data::{self, FnRef, Value};
use crate::knowledge::lookup::{Entry, Guard};
use std::sync::{Arc, LazyLock};

/// `domains()`: the knowledge domain names, in the order `pkgutil.iter_modules` found them
/// (the bundle keeps `_cache["domains"]`). Python maps name -> module; the port only needs names.
pub fn domains() -> &'static [String] {
    static DOMAINS: LazyLock<Vec<String>> = LazyLock::new(|| {
        let cache = data::get("bot.knowledge._cache");
        cache.get("domains").map(|d| d.pairs().iter().filter_map(|(k, _)| k.as_str().map(str::to_string)).collect()).unwrap_or_default()
    });
    &DOMAINS
}

/// `_with_one_more_test(guard, extra)`: a row's guard with one more condition folded in.
pub fn _with_one_more_test(guard: Option<Guard>, extra: Arc<FnRef>) -> Guard {
    match guard {
        None => Guard::Fn(extra),
        Some(g) => Guard::And(Box::new(g), extra),
    }
}

/// `entries()`: every Entry in every domain, in a stable order (domains sorted by name, each
/// domain's ENTRIES in order), each carrying its domain name, with the domain's `STANDS_ASIDE`
/// and `GUARD_FOLDS` applied as Python does.
pub fn entries() -> &'static [Entry] {
    static ALL: LazyLock<Vec<Entry>> = LazyLock::new(|| {
        let mut names: Vec<&String> = domains().iter().collect();
        names.sort();
        let mut found = Vec::new();
        for name in names {
            let prefix = format!("bot.knowledge.{name}.");
            let stands_aside = data::try_get(&format!("{prefix}STANDS_ASIDE")).and_then(Value::as_fn).map(|f| Arc::new(f.clone()));
            let folds = data::try_get(&format!("{prefix}GUARD_FOLDS"));
            let Some(list) = data::try_get(&format!("{prefix}ENTRIES")) else { continue };
            for item in list.items() {
                let Some(mut entry) = Entry::from_value(item) else { continue };
                entry.domain = name.clone();
                if let (Some(aside), None) = (&stands_aside, &entry.guard) {
                    entry.guard = Some(Guard::Fn(aside.clone()));
                }
                if let Some(extra) = folds.and_then(|f| f.get(&entry.id)).and_then(Value::as_fn) {
                    entry.guard = Some(_with_one_more_test(entry.guard.take(), Arc::new(extra.clone())));
                }
                found.push(entry);
            }
        }
        found
    });
    &ALL
}
