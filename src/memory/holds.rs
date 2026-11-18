//! Port of `bot/memory/holds.py`: the earlier turns, one check for "was this said before the
//! line I am answering?".
//!
//! The turns here are the turns that came before the line being answered: the runtime appends
//! the current turn after the modules have replied.

use crate::py::re::{self, Re, I};
use crate::py::text;
use crate::state::{Memory, Turn};

/// Python: bot/memory/holds.py:32 `turns(memory)`
///
/// The turns before this one, oldest first (every one a dict in Python; a `Turn` here).
pub fn turns(memory: &Memory) -> &[Turn] {
    &memory.turns
}

/// Python: bot/memory/holds.py:40 `there_is_one(memory)`
///
/// Whether the chat has an earlier turn at all.
pub fn there_is_one(memory: &Memory) -> bool {
    !turns(memory).is_empty()
}

/// Python: bot/memory/holds.py:49 `lines(memory)`
///
/// What the person said, before this line, oldest first, each one tidied to one line.
pub fn lines(memory: &Memory) -> Vec<String> {
    turns(memory).iter().map(|turn| text::squash(&turn.user)).filter(|line| !line.is_empty()).collect()
}

/// Python: bot/memory/holds.py:59 `the_bot_lines(memory)`
///
/// What the bot said before this line, oldest first.
pub fn the_bot_lines(memory: &Memory) -> Vec<String> {
    turns(memory).iter().map(|turn| text::squash(&turn.bot)).filter(|line| !line.is_empty()).collect()
}

/// Python: bot/memory/holds.py:69 `last_reply(memory)`
///
/// The bot's last line, or "" at the start of a chat.
pub fn last_reply(memory: &Memory) -> String {
    the_bot_lines(memory).pop().unwrap_or_default()
}

/// Python: bot/memory/holds.py:75 `last_module(memory)`
///
/// The module that answered the last line, or "" at the start of a chat.
pub fn last_module(memory: &Memory) -> String {
    turns(memory).last().and_then(|t| t.module.clone()).unwrap_or_default()
}

/// Python: bot/memory/holds.py:81 `recent(memory, n=3)`
///
/// The last `n` turns as text, the person's line and the bot's answer, in order.
pub fn recent(memory: &Memory, n: i64) -> String {
    let held = turns(memory);
    // Python's `held[-n:]`: n == 0 is the whole list, a negative n drops from the front.
    let from = if n > 0 {
        held.len().saturating_sub(n as usize)
    } else if n == 0 {
        0
    } else {
        (n.unsigned_abs() as usize).min(held.len())
    };
    let mut said = Vec::new();
    for turn in &held[from..] {
        for part in [&turn.user, &turn.bot] {
            let part = text::squash(part);
            if !part.is_empty() {
                said.push(part);
            }
        }
    }
    said.join(" ")
}

/// Python: bot/memory/holds.py:92 `says(memory, word)`
///
/// Whether an earlier line of theirs names this word, as a word.
pub fn says(memory: &Memory, word: &str) -> bool {
    let word = text::strip(word).to_lowercase();
    if word.is_empty() {
        return false;
    }
    let pattern = Re::new(&format!(r"\b{}\b", re::escape(&word)), I);
    lines(memory).iter().any(|line| pattern.is_match(line))
}

/// Python: bot/memory/holds.py:101 `the_line(memory, pattern)`
///
/// The first earlier line of theirs that matches `pattern`, or "" when none does.
pub fn the_line(memory: &Memory, pattern: &Re) -> String {
    lines(memory).into_iter().find(|line| pattern.is_match(line)).unwrap_or_default()
}

/// Python: bot/memory/holds.py:109 `the_last_line(memory, pattern)`
///
/// The most recent earlier line of theirs that matches `pattern`, or "".
pub fn the_last_line(memory: &Memory, pattern: &Re) -> String {
    lines(memory).into_iter().rev().find(|line| pattern.is_match(line)).unwrap_or_default()
}
