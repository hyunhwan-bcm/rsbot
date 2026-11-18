//! Port of `bot/compose/griefplea.py`: a plea with nothing in it ("any ideas", "im lost") inside
//! a bereavement is answered with a keeping-going line rather than the nod or the search hedge.

use crate::compose::{fallback, loss};
use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

static A_PLEA_WITH_NOTHING_IN_IT: LazyLock<&'static Re> =
    LazyLock::new(|| data::re("bot.compose.griefplea.A_PLEA_WITH_NOTHING_IN_IT"));
static NOTHING_OF_ITS_OWN: LazyLock<&'static Value> = LazyLock::new(|| data::get("bot.compose.griefplea.NOTHING_OF_ITS_OWN"));
static GETTING_THROUGH_REPLIES: LazyLock<Vec<String>> =
    LazyLock::new(|| data::strs("bot.compose.loss.GETTING_THROUGH_REPLIES"));

/// `fallback.pick(options, memory, seed_text)` (it is `bot.compose.pick`): its contract takes the
/// memory as a `Value`.
fn pick(options: &[String], memory: &Value, seed_text: &str) -> String {
    // Python hands pick a dict that is not the chat's memory (a reading or a one-turn dict);
    // pick reads it as one, so it is read here as a Memory built from that dict.
    let as_memory = match memory.to_plain() {
        serde_json::Value::Object(m) => Memory::from_plain(&m),
        _ => Memory::new(),
    };
    crate::compose::pick(options, &as_memory, seed_text)
}

/// `memory.get("understood") or {}` as the dict Python hands on.
fn understood_or_empty(memory: &Memory) -> Value {
    match &memory.understood {
        Some(u) => Value::from_plain(&serde_json::to_value(u).unwrap()),
        None => Value::dict(),
    }
}

/// Python: bot/compose/griefplea.py:59 `_said(text)`
pub fn _said(text: &str) -> String {
    text::squash(&text::lower(text))
}

/// Python: bot/compose/griefplea.py:63 `reply_for(text, reply, memory)`
///
/// A keeping-going line for a plea inside a bereavement, or None.
pub fn reply_for(text: &str, reply: &str, memory: &Memory) -> Option<String> {
    let said = _said(text);
    if said.is_empty() || A_PLEA_WITH_NOTHING_IN_IT.match_(&said).is_none() {
        return None;
    }
    if text::strip(reply).is_empty() {
        return None;
    }
    if !fallback::_in_a_pool(reply, &NOTHING_OF_ITS_OWN) {
        return None;
    }
    if !loss::continues(memory) {
        // The chat is not a bereavement: the plea is an ordinary bare ask.
        return None;
    }
    let pool = &*GETTING_THROUGH_REPLIES;
    // None of the pool said twice; only when every line has been used does it come round again.
    let already: HashSet<&str> = memory.turns.iter().map(|turn| turn.bot.as_str()).collect();
    let mut fresh: Vec<String> = pool.iter().filter(|line| !already.contains(line.as_str())).cloned().collect();
    if fresh.is_empty() {
        fresh = pool.clone();
    }
    Some(pick(&fresh, &understood_or_empty(memory), &said))
}
