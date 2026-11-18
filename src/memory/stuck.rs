//! Port of `bot/memory/stuck.py`: "i dont know what to do", "where do i start" - a stuck line,
//! answered with one first step read from the kind of news that came before it.

use super::mem8_py::{rx, strs, strset};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.stuck.";

rx!(
    _STATE, STUCK, AN_ASK_AT_THE_END, REACTION_FRAME, NAMES_A_THING, _DEATH, _ENDING, _JOB, _MONEY, _SKINT, _ILLNESS,
    _FEELING, _FLAT, _PLAN, _MAKING, _SELF, _ANIMAL, EMERGENCY, FIRST_SENTENCE, A_CONDITION_IN_FRONT, _START_ASK
);
strs!(DEATH, ENDING, JOB, MONEY, ILLNESS, FEELING, PLAN, SELF, HARD, URGENT_ALREADY_GIVEN, FIRST_LINE);
strset!(DOING);
static STOPWORDS: LazyLock<HashSet<String>> = LazyLock::new(|| crate::data::set("bot.memory.topics.STOPWORDS"));
static A_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d", 0));
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

const EDGE: &str = " ,;:.!?-\u{2014}";

/// Python: bot/memory/stuck.py:77 `_a_reaction_to_what_was_said(lead)`
///
/// Whether the clause before the ask is a reaction to the last thing said ("that sounds hard").
pub fn _a_reaction_to_what_was_said(lead: &str) -> bool {
    let squashed = text::squash(lead);
    let said = text::strip_chars(&squashed, EDGE);
    if said.is_empty() {
        return false;
    }
    let Some(found) = REACTION_FRAME.match_(said) else { return false };
    !NAMES_A_THING.is_match(found.name_str("about"))
}

/// Python: bot/memory/stuck.py:88 `the_ask_in(low)`
///
/// The stuck ask the line is or ends on, or "".
pub fn the_ask_in(low: &str) -> String {
    let said = text::squash(low);
    if STUCK.match_(&said).is_some() {
        return said;
    }
    let Some(found) = AN_ASK_AT_THE_END.search(&said) else { return String::new() };
    let lead = text::strip_chars(&said[..found.start()], EDGE);
    if !lead.is_empty() && !_a_reaction_to_what_was_said(lead) {
        return String::new();
    }
    text::strip_chars(found.as_str(), EDGE).to_string()
}

/// Python: bot/memory/stuck.py:340 `_opens_with_an_emergency(reply)`
///
/// Whether the reply's first sentence sends them to be seen, with no condition in front.
pub fn _opens_with_an_emergency(reply: &str) -> bool {
    let said = text::squash(reply);
    let Some(found) = EMERGENCY.search(&said) else { return false };
    if let Some(end) = FIRST_SENTENCE.search(&said) {
        if found.start() >= end.end() {
            return false;
        }
    }
    !A_CONDITION_IN_FRONT.is_match(&said[..found.start()])
}

/// Python: bot/memory/stuck.py:352 `the_advice_was_to_get_seen(memory)`
///
/// Whether the bot's last reply opened by sending them to be seen.
pub fn the_advice_was_to_get_seen(memory: &Memory) -> bool {
    match memory.turns.iter().rev().find(|t| !text::strip(&t.bot).is_empty()) {
        Some(turn) => _opens_with_an_emergency(&turn.bot),
        None => false,
    }
}

/// Python: bot/memory/stuck.py:371 `pool(text, memory)`
///
/// The replies for a stuck line, read from the news that came before it, or None.
pub fn pool(text: &str, memory: &Memory) -> Option<Vec<String>> {
    let low = text::squash(&text.to_lowercase().replace('\u{2019}', "'"));
    let ask = the_ask_in(&low);
    if ask.is_empty() {
        return None;
    }
    let before = _before(memory);
    if before.is_empty() {
        if _START_ASK.match_(&ask).is_some() {
            return Some(_not_said_again(&HARD, memory));
        }
        if _STATE.match_(&low).is_some() {
            // "im stuck" as the first thing said: the state has its own answers.
            return None;
        }
        return Some(_not_said_again(&FIRST_LINE, memory));
    }
    if the_advice_was_to_get_seen(memory) {
        return Some(_not_said_again(&URGENT_ALREADY_GIVEN, memory));
    }
    // POOLS, in Python's order.
    let pools: [(&Re, &[String], &str); 9] = [
        (*_DEATH, &DEATH, "death"),
        (*_ENDING, &ENDING, ""),
        (*_JOB, &JOB, ""),
        (*_MONEY, &MONEY, ""),
        (*_SKINT, &MONEY, ""),
        (*_ILLNESS, &ILLNESS, ""),
        (*_FEELING, &FEELING, ""),
        (*_FLAT, &FEELING, ""),
        (*_PLAN, &PLAN, "plan"),
    ];
    for (pattern, replies, which) in pools {
        if which == "plan" && _MAKING.is_match(&before) {
            continue;
        }
        if which == "plan" && _SELF.is_match(&before) {
            if _the_answer_gave_a_step(memory) {
                // The wish was already answered with something to do.
                continue;
            }
            return Some(_not_said_again(&SELF, memory));
        }
        if pattern.is_match(&before) {
            if which == "death" && _ANIMAL.is_match(&before) {
                // A pet's death: the pets card answers it better.
                continue;
            }
            return Some(_not_said_again(replies, memory));
        }
    }
    if crate::compose::badnews::reads_as_bad_news(&before) {
        return Some(_not_said_again(&HARD, memory));
    }
    None
}

/// Python: bot/memory/stuck.py:436 `_the_answer_gave_a_step(memory)`
///
/// Whether the bot's last reply named something to do.
pub fn _the_answer_gave_a_step(memory: &Memory) -> bool {
    let said = match memory.turns.iter().rev().find(|t| !text::strip(&t.bot).is_empty()) {
        Some(turn) => text::squash(&turn.bot.to_lowercase()),
        None => return false,
    };
    if A_DIGIT.is_match(&said) || said.contains(':') {
        return true;
    }
    LETTERS.findall_str(&said).iter().take(6).any(|w| DOING.contains(w))
}

/// Python: bot/memory/stuck.py:451 `_not_said_again(replies, memory)`
///
/// The replies with the ones that repeat what the bot just said moved to the back.
pub fn _not_said_again(replies: &[String], memory: &Memory) -> Vec<String> {
    let turns = &memory.turns;
    let said = turns[turns.len().saturating_sub(4)..].iter().map(|t| t.bot.to_lowercase()).collect::<Vec<_>>().join(" ");
    if said.is_empty() {
        return replies.to_vec();
    }
    let written = |reply: &String| -> bool {
        let words: Vec<String> = LETTERS
            .findall_str(&reply.to_lowercase())
            .into_iter()
            .filter(|w| text::len(w) > 3 && !STOPWORDS.contains(w))
            .collect();
        let shared = words.iter().filter(|w| said.contains(w.trim_end_matches('s'))).count();
        if words.is_empty() {
            return false;
        }
        shared >= 3 && shared as f64 / words.len() as f64 >= 0.35
    };
    let mut out = replies.to_vec();
    out.sort_by_key(|r| written(r));
    out
}

/// Python: bot/memory/stuck.py:495 `_before(memory)`
///
/// The line before this one, flattened, or "".
pub fn _before(memory: &Memory) -> String {
    match memory.turns.last() {
        Some(last) => text::squash(&last.user.to_lowercase().replace('\u{2019}', "'")),
        None => String::new(),
    }
}
