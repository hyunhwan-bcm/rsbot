//! Port of `bot/memory/doubt.py`: a line that takes issue with the answer just given is
//! answered about that answer.
//!
//! Two things are told apart: a line that says the answer is *wrong* ("that cant be right") and
//! one that says it is *unclear* ("i dont follow"). Only a doubt that is what the line says, bar
//! a little chatter after it, is read here.

use super::mem8_py::{inline_re, int, pick, py_str, rx, strs, strset, PyVal};
use super::meant;
use crate::py::re::I;
use crate::py::text;
use crate::state::Memory;

const M: &str = "bot.memory.doubt.";

rx!(DOUBTS_THE_ANSWER, SAYS_THE_ANSWER_IS_WRONG);
strset!(DOUBT_TAIL);
strs!(UNCLEAR_ANSWER, DOUBTING, _MEANT_LEADS);
int!(SHORTEST_ANSWER, MOST_WORDS_IN_A_DOUBT);
inline_re!(SENSE, r"\bsense\b", I);

/// Python: bot/memory/doubt.py:97 `_the_answer_just_given(memory)`
///
/// The bot's own last line, or "" when there is none in this conversation yet.
pub fn _the_answer_just_given(memory: &Memory) -> String {
    memory.turns.last().map(|last| last.bot.clone()).unwrap_or_default()
}

/// `" ".join(str(text or "").lower().replace("’", "'").split())`.
fn flat(text: &str) -> String {
    text::squash(&text.to_lowercase().replace('\u{2019}', "'"))
}

/// Python: bot/memory/doubt.py:108 `the_doubt(text, memory)`
///
/// Whether the line doubts the bot's own last answer: "wrong", "unclear", or None.
pub fn the_doubt(text: &str, memory: &Memory) -> Option<&'static str> {
    let said = _the_answer_just_given(memory);
    if (text::len(text::strip(&said)) as i64) < *SHORTEST_ANSWER {
        return None;
    }
    let line = flat(text);
    let words = text::split_ref(&line);
    if words.is_empty() || words.len() as i64 > *MOST_WORDS_IN_A_DOUBT {
        // A long line that carries a doubt is a new line about something of theirs, and the
        // things it names are the answer.
        return None;
    }
    let mut hit = false;
    for found in DOUBTS_THE_ANSWER.finditer(&line) {
        // The doubt has to be what the line is saying, not a clause of something else. Only the
        // end of the line, bar chatter, counts.
        let tail = text::strip_chars(&line[found.end()..], " .,!?;:-");
        if text::split_ref(tail).iter().all(|word| DOUBT_TAIL.contains(*word)) {
            hit = true;
        }
    }
    if !hit {
        return None;
    }
    if SAYS_THE_ANSWER_IS_WRONG.is_match(&line) && !SENSE.is_match(&line) {
        return Some("wrong");
    }
    Some("unclear")
}

/// Python: bot/memory/doubt.py:138 `reply_for(text, memory)`
///
/// The reply to a line that doubts the bot's own answer, or None for every other line.
pub fn reply_for(text: &str, memory: &mut Memory) -> Option<String> {
    let kind = the_doubt(text, memory)?;
    let line = flat(text);
    if kind == "wrong" {
        return Some(pick(&DOUBTING, memory, &line));
    }
    // Where the bot has a real answer to hand back, the reader that gives it plainer words keeps
    // the turn (`bot/memory/meant.py`). A pool line here is only for the doubt the bot has no
    // plainer answer for.
    let plainer = meant::answer_for(&line, memory).val();
    if plainer.truthy() && text::starts_with_any(&py_str(&plainer).to_lowercase(), &_MEANT_LEADS) {
        return None;
    }
    Some(pick(&UNCLEAR_ANSWER, memory, &line))
}
