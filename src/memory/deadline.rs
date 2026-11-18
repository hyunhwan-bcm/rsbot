//! Port of `bot/memory/deadline.py`: a line that is only a deadline ("by december") is a
//! deadline on the plan the chat is already on ("we want to save 3000 pounds by december").

use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understanding::t;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.deadline.";

static DEADLINE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DEADLINE"));
static WHEN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHEN"));
static MONEY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MONEY"));
static FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FILLER"));

/// `the_deadline(text)`: the date part of a line that is only a deadline ("by december" ->
/// "december"), or "".
pub fn the_deadline(text_: &str) -> String {
    let Some(found) = DEADLINE.match_(text_) else { return String::new() };
    let when = text::squash(&found.name_str("when").to_lowercase());
    if when.is_empty() || WHEN.match_(&when).is_none() {
        return String::new();
    }
    when
}

/// `_a_plan(said)`: the line before is a plan the date can be put on.
pub fn _a_plan(said: &str) -> bool {
    let said = text::squash(&said.to_lowercase());
    if said.is_empty() || said.ends_with('?') {
        return false;
    }
    if crate::memory::topics::words(&said).iter().all(|w| FILLER.contains(w)) {
        return false;
    }
    MONEY.is_match(&said)
}

/// `follow_up(text, memory)`: the plan this deadline belongs to, or "" when the line is not that
/// turn.
pub fn follow_up(text_: &str, memory: &Memory) -> String {
    let when = the_deadline(text_);
    if when.is_empty() {
        return String::new();
    }
    let start = memory.turns.len().saturating_sub(3);
    for turn in memory.turns[start..].iter().rev() {
        let said = &turn.user;
        if _a_plan(said) {
            let plan = text::squash(text::rstrip_chars(text::strip(said), ".!,"));
            return format!("{plan} by {when}");
        }
    }
    String::new()
}
