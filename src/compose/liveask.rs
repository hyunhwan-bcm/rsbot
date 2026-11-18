//! Port of `bot/compose/liveask.py`: a decision about live information ("should i go if it
//! rains") gets the place to check and the weighing, in place of the composer's live note.

use crate::compose::fallback_chain::py_format;
use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashMap;
use std::sync::LazyLock;

static LIVE_DECIDE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.liveask.LIVE_DECIDE"));
/// kind -> (check, word)
static LIVE_CHECK: LazyLock<HashMap<String, (String, String)>> = LazyLock::new(|| {
    data::get("bot.compose.liveask.LIVE_CHECK")
        .pairs()
        .iter()
        .map(|(k, v)| {
            let row = v.strs();
            (k.str().to_string(), (row[0].clone(), row[1].clone()))
        })
        .collect()
});
static NOTE_KIND: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.compose.liveask.NOTE_KIND"));
static LIVE_DECIDE_ASKS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.liveask.LIVE_DECIDE_ASKS"));
static NO_MARK: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.liveask.NO_MARK"));

/// `from . import pick`: its contract takes the memory as a `Value`.
fn pick(options: &[String], memory: &Value, seed_text: &str) -> String {
    crate::compose::pick(options, memory, seed_text)
}

/// Python: bot/compose/liveask.py:58 `answer_for(text, memory=None, reply="")`
///
/// The check and the weighing for a decision about live information, or None. Only one of the
/// composer's live notes (`reply`) is replaced.
pub fn answer_for(text: &str, memory: &Memory, reply: &str) -> Option<String> {
    let kind = NOTE_KIND.get(text::strip(reply)).filter(|k| !k.is_empty())?;
    let (check, word) = LIVE_CHECK.get(kind).cloned().unwrap_or_default();
    if check.is_empty() {
        return None;
    }
    let said = NO_MARK.sub("", &text::lower(text));
    if !LIVE_DECIDE_ASKS.is_match(&said) {
        return None;
    }
    let options: Vec<String> =
        LIVE_DECIDE.iter().map(|line| py_format(line, &[("check", &check), ("kind", &word)])).collect();
    // `read = memory.get("understood")`: None there makes `pick` raise (None has no `.get`).
    let read = match &memory.understood {
        Some(u) => Value::from_plain(&serde_json::to_value(u).unwrap()),
        None => panic!("AttributeError: 'NoneType' object has no attribute 'get'"),
    };
    Some(pick(&options, &read, &said))
}
