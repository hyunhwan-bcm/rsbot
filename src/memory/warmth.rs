//! Port of `bot/memory/warmth.py`: warmth said to the bot ("i love you", "youre on form"), and
//! being told it missed ("thats not what i asked", "you said that already") - each answered with
//! the next try of that kind, or with the honest "this is the start" on a first line.

use super::mem8_py::{pick, rx, strs, val};
use crate::state::Memory;
use crate::py::text;

const M: &str = "bot.memory.warmth.";

rx!(
    _LOVE, _LIKE, _FAVOURITE, _APPRECIATE, _ON_FORM, _GOT_IT_WRONG, _THE_WRONG_THING, _NOT_WHAT_I_MEANT, _MIXED_UP,
    _THE_OTHER_ONE, _NO_NOT_THAT, _BARE_WRONG, _MACHINE, _NO_HELP, _WRONG, _INVENTED, _OFF_THE_ASK, _NO_ANSWER,
    _REPEATED, _TOLD_BEFORE, _MUDDLED, _BACK_TO_SOMETHING, _YOUR_NOT
);
strs!(
    PRAISED_REPLIES, MACHINE_REPLIES, WARM_REPLIES, FIRST_LINE_REPLIES, FIRST_LINE_READING, MISSED_REPLIES,
    MISSED_ASK_REPLIES, REPEATED_REPLIES, TOLD_BEFORE_REPLIES, MUDDLED_REPLIES, WRONG_REPLIES
);
val!(_TYPED_SHORT);

/// Python: bot/memory/warmth.py:265 `_nothing_said_before(memory)`
///
/// True on the first line of a conversation: there is nothing earlier to answer about.
pub fn _nothing_said_before(memory: &Memory) -> bool {
    !memory.turns.iter().any(|turn| !text::strip(&turn.user).is_empty())
}

/// Python: bot/memory/warmth.py:273 `_a_complaint_reply(pool, memory, text, first=None)`
///
/// A reply to a complaint about the conversation, and a first-line one when there is none yet.
pub fn _a_complaint_reply(pool: &[String], memory: &Memory, text: &str, first: Option<&[String]>) -> String {
    if !_nothing_said_before(memory) {
        return pick(pool, memory, text);
    }
    let first = match first {
        Some(f) if !f.is_empty() => f,
        _ => &FIRST_LINE_REPLIES,
    };
    pick(first, memory, text)
}

/// Python: bot/memory/warmth.py:489 `_nothing_said_yet(memory)`
///
/// True on the first line of a chat, where there is no earlier turn to answer about.
pub fn _nothing_said_yet(memory: &Memory) -> bool {
    !super::history::an_earlier_turn(memory, "")
}

/// Python: bot/memory/warmth.py:507 `reply(text, memory)`
///
/// A reply to warmth said to the bot, or to being told it missed - else None.
pub fn reply(text: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&text.to_lowercase().replace('\u{2019}', "'"));
    let clean = text::split_ref(&clean)
        .iter()
        .map(|word| _TYPED_SHORT.get(word).map(|v| v.str().to_string()).unwrap_or_else(|| word.to_string()))
        .collect::<Vec<_>>()
        .join(" ");
    let clean = _YOUR_NOT.sub(r"youre \1", &clean);
    let clean = text::strip_chars(&clean, " \t").to_string();
    if clean.is_empty() || text::split_ref(&clean).len() > 8 {
        return None;
    }
    let m = |rx: &crate::py::re::Re| rx.match_(&clean).is_some();
    if m(&_LOVE) || m(&_LIKE) || m(&_FAVOURITE) || m(&_APPRECIATE) {
        return Some(pick(&WARM_REPLIES, memory, text));
    }
    if m(&_ON_FORM) {
        return Some(pick(&PRAISED_REPLIES, memory, text));
    }
    // The corrections below are about an answer the bot has already given, so there has to be a
    // line before them.
    let nothing_before_it = _nothing_said_before(memory);
    if !nothing_before_it
        && (m(&_GOT_IT_WRONG)
            || m(&_THE_WRONG_THING)
            || m(&_NOT_WHAT_I_MEANT)
            || m(&_MIXED_UP)
            || m(&_THE_OTHER_ONE)
            || m(&_NO_NOT_THAT)
            || m(&_BARE_WRONG))
    {
        return Some(pick(&WRONG_REPLIES, memory, text));
    }
    if m(&_MACHINE) {
        return Some(pick(&MACHINE_REPLIES, memory, text));
    }
    // A line about the conversation, on the first line of one.
    let first = _nothing_said_yet(memory);
    if first
        && (m(&_BACK_TO_SOMETHING)
            || m(&_TOLD_BEFORE)
            || m(&_REPEATED)
            || m(&_OFF_THE_ASK)
            || m(&_NO_ANSWER)
            || m(&_MUDDLED)
            || m(&_WRONG)
            || m(&_INVENTED)
            || m(&_NO_HELP))
    {
        return Some(pick(&FIRST_LINE_REPLIES, memory, text));
    }
    if m(&_WRONG) || m(&_INVENTED) {
        return Some(pick(&WRONG_REPLIES, memory, text));
    }
    if m(&_NO_HELP) {
        return Some(_a_complaint_reply(&MISSED_REPLIES, memory, text, None));
    }
    if m(&_REPEATED) {
        return Some(_a_complaint_reply(&REPEATED_REPLIES, memory, text, None));
    }
    if m(&_TOLD_BEFORE) {
        return Some(_a_complaint_reply(&TOLD_BEFORE_REPLIES, memory, text, None));
    }
    if m(&_OFF_THE_ASK) || m(&_NO_ANSWER) {
        return Some(_a_complaint_reply(&MISSED_ASK_REPLIES, memory, text, None));
    }
    if m(&_MUDDLED) {
        return Some(_a_complaint_reply(&MUDDLED_REPLIES, memory, text, Some(&FIRST_LINE_READING)));
    }
    None
}
