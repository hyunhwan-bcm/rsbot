//! Port of `bot/memory/the_verdict.py`: "the plumber said it would be 350", "the consultant said
//! to come back in six weeks" - what somebody's verdict leaves the person to do, by who said it.

use super::mem8_py::{mget, rx, strs, strset};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.the_verdict.";

strset!(OTHERS, CARE, TRADE, OFFICE);
rx!(
    SAID, SELF_SAID, YOU_SAID, EMERGENCY, ASKED, A_TIME, A_PLAN, A_TREATMENT, A_CHANGE, ALL_CLEAR, DONE, PRAISE,
    GOOD_NEWS, MY_MAN, A_SPAN, A_WINDOW, PRONOUN, ABOUT_SOMEONE_ELSE, CLEARED, A_GOOD_SAID
);
strs!(
    TRADE_REPLIES, CARE_REPLIES, CARE_PLAN_REPLIES, CARE_TREATMENT_REPLIES, OFFICE_REPLIES, WAITING_REPLIES,
    SPAN_REPLIES, CARER_REPLIES, CARER_ON_THE_DAY_REPLIES
);
static WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9'&]+", 0));

/// One of the three party pools of `POOLS` (Python compares them by identity).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Party {
    Care,
    Trade,
    Office,
}

impl Party {
    pub fn replies(self) -> &'static [String] {
        match self {
            Party::Care => &CARE_REPLIES,
            Party::Trade => &TRADE_REPLIES,
            Party::Office => &OFFICE_REPLIES,
        }
    }
}

/// Python: bot/memory/the_verdict.py:282 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    WORD.findall_str(&text.to_lowercase())
}

/// Python: bot/memory/the_verdict.py:286 `_the_subject(clean)`
///
/// The words before the saying verb: who spoke.
pub fn _the_subject(clean: &str) -> Vec<String> {
    let head = SAID.split(clean).swap_remove(0);
    _words(&head)
}

/// Python: bot/memory/the_verdict.py:300 `_the_pool(words)`
///
/// The pool for the first party the words name (care, then trade, then office), or None.
pub fn _the_pool(words: &[String]) -> Option<Party> {
    let named: HashSet<&str> = words.iter().map(String::as_str).collect();
    for (parties, party) in [(&*CARE, Party::Care), (&*TRADE, Party::Trade), (&*OFFICE, Party::Office)] {
        if named.iter().any(|w| parties.contains(*w)) {
            return Some(party);
        }
    }
    None
}

/// Python: bot/memory/the_verdict.py:315 `_last_party(game)`
///
/// The trade named in one of the person's last four lines, for a verdict given by pronoun.
pub fn _last_party(game: &Memory) -> Option<Party> {
    let turns = &game.turns;
    let said: Vec<&str> =
        turns[turns.len().saturating_sub(4)..].iter().map(|t| t.user.as_str()).filter(|u| !u.is_empty()).collect();
    for line in said.iter().rev() {
        if _the_pool(&_words(line)) == Some(Party::Trade) {
            return Some(Party::Trade);
        }
    }
    None
}

/// Python: bot/memory/the_verdict.py:345 `_recent_bot_lines(memory)`
pub fn _recent_bot_lines(memory: &Memory) -> Vec<String> {
    let turns = &memory.turns;
    let mut said: Vec<String> = turns[turns.len().saturating_sub(4)..]
        .iter()
        .filter(|t| !t.bot.is_empty())
        .map(|t| text::squash(&t.bot.to_lowercase()))
        .collect();
    let bot = mget(memory, "bot");
    if bot.truthy() {
        said.push(text::squash(&super::mem8_py::py_str(bot).to_lowercase()));
    }
    said.into_iter().filter(|l| !l.is_empty()).collect()
}

/// Python: bot/memory/the_verdict.py:357 `_choose(pool, memory)`
///
/// The first line of the pool the bot has not said lately, or the first line.
pub fn _choose(pool: &[String], memory: &Memory) -> String {
    let recent = _recent_bot_lines(memory);
    for line in pool {
        let head = text::slice(&text::squash(&line.to_lowercase()), 0, Some(40));
        if !recent.iter().any(|said| said.contains(&head)) {
            return line.clone();
        }
    }
    pool[0].clone()
}

/// Python: bot/memory/the_verdict.py:432 `answer_for(text, memory)`
///
/// The reply to a reported verdict, or None.
pub fn answer_for(text: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&text.to_lowercase());
    if clean.is_empty() || !SAID.is_match(&clean) {
        return None;
    }
    if EMERGENCY.is_match(&clean) || ASKED.is_match(&clean) || SELF_SAID.is_match(&clean) || YOU_SAID.is_match(&clean) {
        return None;
    }
    let timed = A_TIME.is_match(&clean);
    let planned = A_PLAN.is_match(&clean);
    if (DONE.is_match(&clean) || ALL_CLEAR.is_match(&clean)) && !timed {
        return None;
    }
    if PRAISE.is_match(&clean) && !(timed || planned) {
        return None;
    }
    if MY_MAN.is_match(&clean) || GOOD_NEWS.is_match(&clean) {
        return None;
    }
    // A verdict that clears the thing leaves nothing to decide, unless a wait is still in it.
    if CLEARED.is_match(&clean) && !(timed || planned) {
        return None;
    }
    let words = _words(&clean);
    let pool = match _the_pool(&_the_subject(&clean)) {
        Some(p) => p,
        None => {
            if !PRONOUN.is_match(&clean) {
                return None;
            }
            _last_party(memory)?
        }
    };
    if (pool == Party::Care || pool == Party::Office) && words.iter().any(|w| OTHERS.contains(w)) {
        // A verdict about somebody in their care is the carer's to follow up.
        if ABOUT_SOMEONE_ELSE.is_match(&clean) && pool == Party::Office && !A_GOOD_SAID.is_match(&clean) {
            if timed {
                return Some(_choose(&CARER_ON_THE_DAY_REPLIES, memory));
            }
            return Some(_choose(&CARER_REPLIES, memory));
        }
        return None;
    }
    // The care pool is about the wait, so the wait has to be in the line.
    if pool == Party::Care {
        if timed {
            return Some(_choose(&CARE_REPLIES, memory));
        }
        if planned {
            return Some(_choose(&CARE_PLAN_REPLIES, memory));
        }
        if A_TREATMENT.is_match(&clean) || A_CHANGE.is_match(&clean) {
            return Some(_choose(&CARE_TREATMENT_REPLIES, memory));
        }
        return None;
    }
    if A_SPAN.is_match(&clean) && !A_WINDOW.is_match(&clean) {
        return Some(_choose(&SPAN_REPLIES, memory));
    }
    if timed {
        return Some(_choose(&WAITING_REPLIES, memory));
    }
    Some(_choose(pool.replies(), memory))
}
