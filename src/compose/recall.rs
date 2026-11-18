//! Port of `bot/compose/recall.py`: one plan said back by the store, with a short forward-looking
//! question after it ("How are you feeling about it?").

use crate::data;
use crate::memory::narrow::memory_value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

static TAILS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.recall.TAILS"));
static LONG_PLAN: LazyLock<usize> = LazyLock::new(|| data::get("bot.compose.recall.LONG_PLAN").as_int().unwrap() as usize);
static PLAN_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.recall.PLAN_WORDS"));
static GRIEF_PLAN: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.recall.GRIEF_PLAN"));

/// `from . import pick`: its contract takes the memory as a `Value`.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, &memory_value(memory), seed_text)
}

/// Python: bot/compose/recall.py:47 `_is_a_death(said)`
///
/// True when the plan itself is a death, a funeral, or the arrangements around one.
pub fn _is_a_death(said: &str) -> bool {
    GRIEF_PLAN.is_match(said)
}

/// Python: bot/compose/recall.py:57 `_said_on_the_last_turn(said, memory)`
///
/// True when the bot's own last line was this plan (with or without its tail).
pub fn _said_on_the_last_turn(said: &str, memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    let bot = text::strip(&last.bot);
    !bot.is_empty() && (bot == said || bot.starts_with(&format!("{said} ")))
}

/// Python: bot/compose/recall.py:71 `_names_a_thing(said)`
///
/// True when the plan says what it is about, so "it" in the tail has something to hold.
pub fn _names_a_thing(said: &str) -> bool {
    text::split_ref(text::strip_chars(said, " ."))
        .iter()
        .skip(1)
        .any(|word| text::is_alpha(word) && !PLAN_WORDS.contains(&text::lower(word)))
}

/// Python: bot/compose/recall.py:77 `with_a_tail(said, memory)`
///
/// The one plan said back, and a short question after it (always a str).
pub fn with_a_tail(said: &str, memory: &Memory) -> String {
    let said = text::strip(said);
    if said.is_empty() || text::len(said) > *LONG_PLAN || text::count(said, ".") > 1 {
        return said.to_string();
    }
    if !_names_a_thing(said) || _is_a_death(said) {
        return said.to_string();
    }
    if _said_on_the_last_turn(said, memory) {
        return said.to_string();
    }
    format!("{said} {}", pick(&TAILS, memory, said))
}
