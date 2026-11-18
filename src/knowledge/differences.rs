//! Port of `bot/knowledge/differences.py`: "what is the difference between X and Y". The pairs
//! and their entries are data (built at import time by `_pair` / `_slug`, exported in the
//! bundle); the runtime logic is the cream guard.

use crate::data::FnRef;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::understood::Understood;

/// The line says cream somewhere: "single or double cream" is the same question as the pair.
pub fn _with_cream(understood: &Understood) -> bool {
    understood.clean.to_lowercase().contains("cream")
}

pub const GUARDS: &[(&str, GuardFn)] =
    &[("bot.knowledge.differences:_with_cream:274", |_: &FnRef, u: &Understood| _with_cream(u))];

pub const REPLIES: &[(&str, ReplyFn)] = &[];


// ---------------------------------------------------------------------------------------------
// The row factory (`knowledge::built`).

use crate::data::{self, Value};
use crate::knowledge::built;
use crate::knowledge::entry::{Entry, KeyPart, Reply};

/// A row id from the two names, so a duplicate pair shows up as a duplicate id.
pub fn _slug(name: &str) -> String {
    static NOT_AN_ID: std::sync::LazyLock<crate::py::re::Re> = std::sync::LazyLock::new(|| crate::py::re::Re::new(r"[^a-z0-9]+", 0));
    crate::py::text::strip_chars(&NOT_AN_ID.sub("-", &crate::py::text::lower(name)), "-").to_string()
}

/// `_pair(row)`: one pair - the two names as the two key groups, and only a comparing line
/// reaches it. `row` is a `_PAIRS` row: (first names, second names, what to say).
pub fn _pair(row: &Value) -> Entry {
    let row = row.items();
    let (first, second, said) = (row[0].strs(), row[1].strs(), row[2].str());
    built::entry(
        &format!("difference-{}-and-{}", _slug(&first[0]), _slug(&second[0])),
        vec![built::part(&first), built::part(&second)],
        Reply::Text(said.to_string()),
        &[],
        2,
        data::get("bot.knowledge.differences._COMPARING").strs().into_iter().map(KeyPart::Word).collect(),
        None,
    )
}

#[cfg(test)]
mod factory_tests {
    use super::*;

    #[test]
    fn pairs_rebuild() {
        data::load();
        let exported = built::exported("bot.knowledge.differences.ENTRIES");
        let pairs = data::get("bot.knowledge.differences._PAIRS").items();
        assert!(!pairs.is_empty());
        for (row, theirs) in pairs.iter().zip(exported.iter()) {
            assert_eq!(built::same_entry(&_pair(row), theirs), None);
        }
    }
}
