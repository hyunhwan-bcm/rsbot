//! Port of `bot/compose/a_thing_wants_doing.py`: a thing handed over in a state that wants
//! doing ("the garden is overgrown"), answered by asking how bad it is.

use crate::data;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.compose.a_thing_wants_doing.";
fn set_(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}

static BODY_PARTS: LazyLock<HashSet<String>> = LazyLock::new(|| set_("BODY_PARTS"));
static A_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| set_("A_STATE"));
static A_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| set_("A_PERSON"));
static A_FEELING: LazyLock<HashSet<String>> = LazyLock::new(|| set_("A_FEELING"));
static THE_THING_IS: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}THE_THING_IS")));
static A_THING_WANTS_DOING: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}A_THING_WANTS_DOING")));

/// Python: bot/compose/a_thing_wants_doing.py:48 `reads(text, memory=None)`
///
/// The thing the line hands over, when it is in a state that wants doing, else None.
pub fn reads(text: &str, memory: &Memory) -> Option<String> {
    let flat = text::squash(&text::lower(text));
    let words = text::split_ref(&flat);
    if flat.is_empty() || words.len() > 8 {
        return None;
    }
    if words.iter().any(|word| A_FEELING.contains(*word)) {
        return None;
    }
    let found = THE_THING_IS.match_(&flat)?;
    let (state, thing) = (found.name_str("state"), found.name_str("thing"));
    if !A_STATE.contains(state) {
        return None;
    }
    if text::split_ref(thing).iter().any(|word| A_PERSON.contains(*word) || BODY_PARTS.contains(*word)) {
        return None;
    }
    Some(thing.to_string())
}

/// Python: bot/compose/a_thing_wants_doing.py:66 `pool(text, memory=None)`
pub fn pool(text: &str, memory: &Memory) -> Option<Vec<String>> {
    reads(text, memory).filter(|thing| !thing.is_empty())?;
    Some(A_THING_WANTS_DOING.clone())
}
