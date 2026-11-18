//! Port of `bot/compose/doubt.py`: a bare doubt ("is that bad", "should i be worried") on top
//! of an answer that told the person to get it seen, where somebody qualified already sent them.

use crate::data;
use crate::memory::narrow::memory_value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.doubt.", stringify!($name))));)*
    };
}
regexes!(BARE_DOUBT, QUANTITY_ASK, CARE_NAMED, SOMEBODY_TOLD_THEM, TOLD);
static CARE_LINES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.doubt.CARE_LINES"));
static NOT_READ_BACK: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.doubt.NOT_READ_BACK"));
/// `re.sub(r"[^a-z0-9 ]+", " ", ...)` and `re.sub(r"\s+", " ", ...)`, written inline.
static NOT_A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9 ]+", 0));
static SPACES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+", 0));

/// `from . import pick`: its contract takes the memory as a `Value`.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, &memory_value(memory), seed_text)
}

/// Python: bot/compose/doubt.py:66 `_last_bot(memory)`
///
/// The bot's last line before this turn, as (text, module). Only this file calls it.
pub fn _last_bot(memory: &Memory) -> (String, String) {
    for turn in memory.turns.iter().rev() {
        if !turn.bot.is_empty() {
            return (turn.bot.clone(), turn.module.clone().unwrap_or_default());
        }
    }
    (String::new(), String::new())
}

/// Python: bot/compose/doubt.py:80 `reads_as_a_doubt(clean)`
///
/// True when the whole line is a doubt about the answer above and nothing else.
pub fn reads_as_a_doubt(clean: &str) -> bool {
    let flat = NOT_A_WORD.sub(" ", &text::lower(clean));
    let flat = SPACES.sub(" ", &flat);
    let flat = text::strip(&flat);
    !flat.is_empty() && BARE_DOUBT.match_(flat).is_some()
}

/// Python: bot/compose/doubt.py:87 `_the_line_behind(memory)`
///
/// The person's own line that the answer above was answering, lowercased.
pub fn _the_line_behind(memory: &Memory) -> String {
    for turn in memory.turns.iter().rev() {
        if !turn.bot.is_empty() {
            return text::lower(&turn.user);
        }
    }
    String::new()
}

/// Python: bot/compose/doubt.py:100 `reply_for(clean, memory)`
///
/// The answer to a doubt on top of a call to get it seen, or None.
pub fn reply_for(clean: &str, memory: &Memory) -> Option<String> {
    if !reads_as_a_doubt(clean) {
        return None;
    }
    if QUANTITY_ASK.is_match(&text::lower(clean)) {
        return None;
    }
    let (said, module) = _last_bot(memory);
    if said.is_empty() || NOT_READ_BACK.contains(&module) {
        return None;
    }
    if !CARE_NAMED.is_match(&said) {
        return None;
    }
    // Somebody qualified already made the referral: the doubt is "was that fussing".
    let behind = _the_line_behind(memory);
    if !(SOMEBODY_TOLD_THEM.is_match(&behind) && TOLD.is_match(&behind)) {
        return None;
    }
    // Picked, not drawn: the same conversation has to get the same sentence.
    Some(pick(&CARE_LINES, memory, clean))
}

