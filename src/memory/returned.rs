//! Port of `bot/memory/returned.py`: "its back", "its dropped again", "and if it does not get
//! better" after an answer - the sentence of that answer that says what to do when the trouble
//! recurs, handed back.

use super::mem8_py::{rx, strs};
use crate::py::text;
use crate::state::{Memory, Turn};

const M: &str = "bot.memory.returned.";

rx!(
    RETURNING, AGAIN, RECURRING, SELF_EVIDENT, ASKING, NEXT_STEP, SENTENCE, CONDITIONAL_ASK, LEADING_CONDITION,
    RECURRENCE, REMIND_ME, AN_ASK
);
strs!(NOT_AN_ANSWER, CONTINUES);

/// Python: bot/memory/returned.py:91 `reads_as_returning(text)`
///
/// True when the line reports that the thing the answer was about has come back.
pub fn reads_as_returning(text: &str) -> bool {
    let line = text::squash(&text.to_lowercase());
    if line.is_empty() || text::split_ref(&line).len() > 16 {
        return false;
    }
    if ASKING.match_(&line).is_some() {
        return false;
    }
    if RETURNING.is_match(&line) {
        return true;
    }
    AGAIN.is_match(&line) && (RECURRING.is_match(&line) || SELF_EVIDENT.is_match(&line))
}

/// Python: bot/memory/returned.py:103 `_the_turn_before(memory)`
pub fn _the_turn_before(memory: &Memory) -> Option<&Turn> {
    memory.turns.last()
}

/// Python: bot/memory/returned.py:108 `_the_answer_before(memory, keep=4)`
///
/// The last turn whose reply holds a next step, or None; an emergency answer stops the walk.
pub fn _the_answer_before(memory: &Memory, keep: i64) -> Option<&Turn> {
    let turns = &memory.turns;
    // `turns[-keep:]` (keep is 4 everywhere it is called; 0 would be the whole list).
    let from = if keep > 0 { turns.len().saturating_sub(keep as usize) } else { 0 };
    for turn in turns[from..].iter().rev() {
        let module = turn.module.clone().unwrap_or_default();
        if NOT_AN_ANSWER.contains(&module) {
            return None;
        }
        if the_next_step(&turn.bot).is_some() {
            return Some(turn);
        }
    }
    None
}

/// Python: bot/memory/returned.py:141 `the_next_step(reply)`
///
/// The sentence of an answer that says what to do if the trouble recurs, or None, with the
/// conditional taken off its front where what is left still stands as a sentence.
pub fn the_next_step(reply: &str) -> Option<String> {
    for sentence in SENTENCE.split(text::strip(reply)) {
        let sentence = text::strip(&sentence).to_string();
        if sentence.is_empty() || !NEXT_STEP.is_match(&sentence) {
            continue;
        }
        let bare = LEADING_CONDITION.sub("", &sentence);
        if !bare.is_empty() && bare != sentence && text::split_ref(&bare).len() >= 4 {
            let first = text::split_ref(&bare)[0].to_lowercase();
            if !CONTINUES.iter().any(|c| c == text::strip_chars(&first, ",.;:")) {
                return Some(text::upper_first(&bare));
            }
        }
        if text::starts_with_any(&sentence.to_lowercase(), &["if ", "when ", "once "]) {
            // The conditional will not come off, so the sentence is half of one.
            continue;
        }
        return Some(sentence);
    }
    None
}

/// Python: bot/memory/returned.py:185 `says_it_happened_again(text)`
///
/// True when the line reports the thing happening once more, so a repeat frame is wrong.
pub fn says_it_happened_again(text: &str) -> bool {
    let said = text::squash(&text.to_lowercase());
    if said.is_empty() || REMIND_ME.is_match(&said) || AN_ASK.match_(&said).is_some() {
        return false;
    }
    if reads_as_returning(&said) {
        return true;
    }
    RECURRENCE.is_match(&said)
}

/// Python: bot/memory/returned.py:201 `reads_as_a_conditional(text)`
///
/// True when the line asks what happens if the trouble is still there.
pub fn reads_as_a_conditional(text: &str) -> bool {
    CONDITIONAL_ASK.match_(&text::squash(&text.to_lowercase())).is_some()
}

/// Python: bot/memory/returned.py:211 `pool_for(text, memory)`
///
/// The next step the answer already names, or None when this line is not that follow-up.
pub fn pool_for(text: &str, memory: &Memory) -> Option<String> {
    let conditional = reads_as_a_conditional(text);
    if !reads_as_returning(text) && !conditional {
        return None;
    }
    let before = _the_answer_before(memory, 4)?;
    let said = before.bot.clone();
    let step = the_next_step(&said)?;
    if text::squash(&step.to_lowercase()) == text::squash(&said.to_lowercase()) {
        return None; // the answer was that one sentence, and it is already on the screen
    }
    if conditional {
        // The branch is only worth handing back when there was an answer around it.
        let around = text::squash(&said.replace(&step, " "));
        if text::split_ref(&around).len() <= 8 {
            return None;
        }
    }
    Some(step)
}

/// Python: bot/memory/returned.py:240 `the_whole_of_it(text, memory)`
///
/// The answer the step was cut from, for the store's record of what has been said.
pub fn the_whole_of_it(text: &str, memory: &Memory) -> Option<String> {
    if !reads_as_returning(text) {
        return None;
    }
    let said = _the_answer_before(memory, 4).map(|t| t.bot.clone()).unwrap_or_default();
    if the_next_step(&said).is_some() { Some(said) } else { None }
}
