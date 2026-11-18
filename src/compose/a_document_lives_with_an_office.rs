//! Port of `bot/compose/a_document_lives_with_an_office.py`: "where do i get a copy of the
//! deeds", "we lost the log book" - a record that one office holds gets that office named.

use crate::data::{self, Value};
use crate::memory::narrow::memory_value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

const M: &str = "bot.compose.a_document_lives_with_an_office.";
macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.a_document_lives_with_an_office.", stringify!($name))));)*
    };
}
regexes!(AN_ASK, A_LOST, A_PERSON_LOST, A_MAP_LINE);
static THE_OFFICE_NAMES: LazyLock<&'static Value> = LazyLock::new(|| data::get(&format!("{M}THE_OFFICE_NAMES")));
static THE_OFFICE: LazyLock<&'static Value> = LazyLock::new(|| data::get(&format!("{M}THE_OFFICE")));
static THE_KINDS: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    data::get(&format!("{M}THE_KINDS"))
        .items()
        .iter()
        .map(|row| {
            let row: &'static [Value] = row.items();
            (row[0].str().to_string(), row[1].as_re().unwrap())
        })
        .collect()
});

/// `from . import pick`: its contract takes the memory as a `Value`.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, &memory_value(memory), seed_text)
}

/// Python: bot/compose/a_document_lives_with_an_office.py:179 `a_map_app_line(reply)`
///
/// True when the reply is the map-app reader's line about a thing to buy.
pub fn a_map_app_line(reply: &str) -> bool {
    !reply.is_empty() && A_MAP_LINE.match_(text::strip(reply)).is_some()
}

/// Python: bot/compose/a_document_lives_with_an_office.py:184 `_the_kind(clean)`
///
/// The family of record the line names, or "" when it names none.
pub fn _the_kind(clean: &str) -> String {
    THE_KINDS.iter().find(|(_, pattern)| pattern.is_match(clean)).map(|(name, _)| name.clone()).unwrap_or_default()
}

/// Python: bot/compose/a_document_lives_with_an_office.py:192 `_names_the_office(reply, kind)` (never called on the trace corpus)
///
/// True when the reply already says who holds it.
pub fn _names_the_office(reply: &str, kind: &str) -> bool {
    if reply.is_empty() {
        return false;
    }
    THE_OFFICE_NAMES.get(kind).and_then(Value::as_re).is_some_and(|pattern| pattern.is_match(reply))
}

/// Python: bot/compose/a_document_lives_with_an_office.py:198 `a_lost_record(text, reply=None)`
///
/// True when the line is a record somebody has lost and the reply has not named its holder.
pub fn a_lost_record(text: &str, reply: &str) -> bool {
    let clean = text::squash(&text::lower(text));
    if clean.is_empty() || !A_LOST.is_match(&clean) || A_PERSON_LOST.is_match(&clean) {
        return false;
    }
    let kind = _the_kind(&clean);
    if kind.is_empty() {
        return false;
    }
    !_names_the_office(reply, &kind)
}

/// Python: bot/compose/a_document_lives_with_an_office.py:215 `answer_for(text, memory=None, reply=None)`
///
/// A reply naming the office that holds the record, or None.
pub fn answer_for(text: &str, memory: &Memory, reply: &str) -> Option<String> {
    let clean = text::squash(&text::lower(text));
    if clean.is_empty() {
        return None;
    }
    let asked = AN_ASK.is_match(&clean);
    let lost = A_LOST.is_match(&clean);
    if !asked && !lost {
        return None;
    }
    let kind = _the_kind(&clean);
    if kind.is_empty() {
        return None;
    }
    if lost && A_PERSON_LOST.is_match(&clean) {
        return None;
    }
    if lost && !asked && _names_the_office(reply, &kind) {
        return None;
    }
    let pool = THE_OFFICE.get(&kind).expect("KeyError").strs();
    Some(pick(&pool, memory, &clean))
}
