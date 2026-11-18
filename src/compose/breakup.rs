//! Port of `bot/compose/breakup.py`: a break-up is not a broken bone, so the question after it
//! ("is she going to be ok") is not about a ward. Only a reply already chosen from the pools that
//! name a patient is replaced.

use crate::data;
use crate::py::random::{ord_sum, Random};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

static THE_ILL_POOLS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.breakup.THE_ILL_POOLS"));
static A_BREAK_UP: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.breakup.A_BREAK_UP"));
static A_HARD_TIME: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.fallback.A_HARD_TIME"));
static A_MEDICAL_THREAD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.fallback.A_MEDICAL_THREAD"));
static AN_ILLNESS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.fallback.AN_ILLNESS"));
static UNKNOWN_HOW_IT_GOES_FOR_A_HARD_TIME: LazyLock<Vec<String>> =
    LazyLock::new(|| data::strs("bot.compose.fallback.UNKNOWN_HOW_IT_GOES_FOR_A_HARD_TIME"));

/// Python: bot/compose/breakup.py:38 `_thread_said(memory, line)` (never called on the trace corpus)
///
/// The last few turns of the talk, the bot's own included, as one lower-case string.
pub fn _thread_said(memory: &Memory, line: &str) -> String {
    let turns = &memory.turns[memory.turns.len().saturating_sub(3)..];
    let mut said: Vec<&str> = turns.iter().map(|turn| turn.user.as_str()).collect();
    said.extend(turns.last().map(|turn| turn.bot.as_str()));
    said.push(line);
    text::lower(&said.join(" "))
}

/// Python: bot/compose/breakup.py:47 `answer_for(text, reply, memory)`
///
/// The hard-patch answer where the ill-somebody pool was reached by a break-up, else None. Only a
/// reply from one of those pools is replaced, and only when taking the break-up words out of the
/// thread leaves nothing medical behind.
pub fn answer_for(text: &str, reply: &str, memory: &Memory) -> Option<String> {
    if !THE_ILL_POOLS.iter().any(|line| line == reply) {
        return None;
    }
    let said = _thread_said(memory, text);
    if !A_BREAK_UP.is_match(&said) {
        return None;
    }
    if !A_HARD_TIME.is_match(&said) {
        return None; // a break-up on its own is not this reading's line
    }
    let without_the_split = A_BREAK_UP.sub(" ", &said);
    if A_MEDICAL_THREAD.is_match(&without_the_split) {
        return None;
    }
    if AN_ILLNESS.is_match(&without_the_split) {
        return None;
    }
    let pool = &*UNKNOWN_HOW_IT_GOES_FOR_A_HARD_TIME;
    let turns = &memory.turns;
    let last = turns.last().map(|turn| turn.bot.as_str()).unwrap_or("");
    let mut fresh: Vec<&String> = pool.iter().filter(|line| line.as_str() != last).collect();
    if fresh.is_empty() {
        fresh = pool.iter().collect();
    }
    // Seeded, not `random.choice`: same turns and same line, same sentence.
    let seed = turns.len() as i128 * 7919 + ord_sum(&text::lower(text));
    Some((*Random::new(seed).choice(&fresh)).clone())
}
