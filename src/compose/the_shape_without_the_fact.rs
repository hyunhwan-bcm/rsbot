//! Port of `bot/compose/the_shape_without_the_fact.py`: when the bot holds no fact for the line,
//! what can still be said about the shape of answer it wants (a pick, an amount, a time, a yes or
//! no), in place of a placeholder.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_sets, bundle_str, bundle_strs, bundle_vals, inline_re};
use crate::compose::fallback;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

bundle_str!("bot.compose.the_shape_without_the_fact"; PICK, OPTIONS, AMOUNT, TIME, STEPS, YESNO, VAGUE, RECALL, PAPERWORK, REASON,
    ADVICE);
bundle_strs!("bot.compose.the_shape_without_the_fact"; PICK_LINES_NAMED);
bundle_vals!("bot.compose.the_shape_without_the_fact"; _LINES, _A_HEADS);
bundle_res!("bot.compose.the_shape_without_the_fact"; _A_VAGUE, _WHAT_TO_DO_WITH, _A_WHAT_WAS_IT, _A_THEY_SAID_SO, _A_FREQUENCY,
    _AN_ASK_AT_THE_FRONT, _WHICH_I_NEED, _HEAVY, _TWO_THINGS, _HOW_MANY, _A_TIME_WORD, _A_FACT_ABOUT_TIME, _A_STORED_COUNT,
    _A_FIT_STATE, _A_POINTS_AT_A_THING, _A_VERDICT_WORD, _A_DECISION, _A_WHOSE_THING, _A_DATE_ON_IT, _THEIR_THINGS,
    _AN_ENOUGH_ASK, _A_PICK_ASK, _A_STRAIGHT_YESNO, _A_WORK_OUT, _ON_PAPER, _BARE_ASK, _LEANS_ON_THE_THREAD);
bundle_sets!("bot.compose.the_shape_without_the_fact"; _NOT_AN_OPTION, _NOT_THE_THING);
inline_re!(A_DIGIT, r"\d");

static NEUTRAL: LazyLock<&'static Value> = LazyLock::new(|| data::get("bot.compose.fallback.NEUTRAL"));
static UNKNOWN: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.fallback.UNKNOWN"));

/// `understood.get("clean") or understood.get("text") or ""`.
fn the_line(understood: &Understood) -> &str {
    if understood.clean.is_empty() { &understood.text } else { &understood.clean }
}

/// The last `n` turns, each as its user and bot lines.
fn last_turns(memory: &Memory, n: usize) -> &[crate::state::Turn] {
    &memory.turns[memory.turns.len().saturating_sub(n)..]
}

/// `_LINES.get(want)`.
fn lines_for(want: &str) -> Option<Vec<String>> {
    _LINES.get(want).map(Value::strs)
}

/// Python: bot/compose/the_shape_without_the_fact.py:252 `_the_two_things_are_on_the_table(understood, memory)`
///
/// True when the person or the bot has already said what the choice is between.
pub fn _the_two_things_are_on_the_table(understood: &Understood, memory: &Memory) -> bool {
    let mut said = vec![the_line(understood).to_string()];
    // Four turns back, not one.
    for turn in last_turns(memory, 4) {
        said.push(turn.user.clone());
        said.push(turn.bot.clone());
    }
    for t in &said {
        for m in _TWO_THINGS.finditer(&t.to_lowercase()) {
            if !_NOT_AN_OPTION.contains(m.str(1)) && !_NOT_AN_OPTION.contains(m.str(2)) {
                return true;
            }
        }
    }
    false
}

/// Python: bot/compose/the_shape_without_the_fact.py:277 `_the_figures_are_in_the_thread(memory)`
///
/// The person has already given the numbers, so a reply asking for them is a daft reply.
pub fn _the_figures_are_in_the_thread(memory: &Memory) -> bool {
    last_turns(memory, 2).iter().any(|t| A_DIGIT.search(&t.user).is_some() || A_DIGIT.search(&t.bot).is_some())
}

/// Python: bot/compose/the_shape_without_the_fact.py:287 `_the_bot_just_said_a_lot(memory)`
///
/// The previous answer gave the person something to work with, not just a line back.
pub fn _the_bot_just_said_a_lot(memory: &Memory) -> bool {
    last_turns(memory, 2).iter().rev().any(|t| text::split_ref(&t.bot).len() >= 25)
}

/// Python: bot/compose/the_shape_without_the_fact.py:297 `_a_count_of_the_thing_just_named(clean, memory)`
pub fn _a_count_of_the_thing_just_named(clean: &str, memory: &Memory) -> bool {
    let lowered = clean.to_lowercase();
    let Some(m) = _HOW_MANY.match_(text::strip(&lowered)) else { return false };
    let word = m.str(1);
    if _NOT_THE_THING.contains(word) {
        return false;
    }
    let turns = last_turns(memory, 4);
    if turns.is_empty() {
        return false;
    }
    let rx = Re::new(&format!(r"(?<![a-z]){}[a-z]{{0,2}}(?![a-z])", pyre::escape(word.trim_end_matches('s'))), 0);
    turns.iter().rev().any(|t| rx.search(&t.bot.to_lowercase()).is_some() || rx.search(&t.user.to_lowercase()).is_some())
}

/// Python: bot/compose/the_shape_without_the_fact.py:439 `_on_paper(line, memory)`
///
/// True when a paper named in the line or the last few turns holds the date or the figure.
pub fn _on_paper(line: &str, memory: &Memory) -> bool {
    let mut said = vec![line.to_string()];
    for turn in last_turns(memory, 4) {
        said.push(turn.user.clone());
        said.push(turn.bot.clone());
    }
    said.iter().any(|t| !t.is_empty() && _ON_PAPER.search(&t.to_lowercase()).is_some())
}

/// Python: bot/compose/the_shape_without_the_fact.py:453 `_the_thing_is_theirs(memory)`
///
/// True when the thread has a thing of their own in it that a date could belong to.
pub fn _the_thing_is_theirs(memory: &Memory) -> bool {
    last_turns(memory, 3).iter().any(|t| {
        let said = t.user.to_lowercase();
        _A_WHOSE_THING.search(&said).is_some() && _THEIR_THINGS.search(&said).is_some()
    })
}

/// Python: bot/compose/the_shape_without_the_fact.py:471 `_the_thread_is_heavy(memory)`
pub fn _the_thread_is_heavy(memory: &Memory) -> bool {
    last_turns(memory, 3).iter().any(|t| _HEAVY.search(&t.user.to_lowercase()).is_some())
}

/// Python: bot/compose/the_shape_without_the_fact.py:480 `the_shape_without_the_fact(understood, memory=None)`
///
/// The pool for the shape of answer this line wants, or None when it wants none of these.
pub fn the_shape_without_the_fact(understood: &Understood, memory: &Memory) -> Option<Vec<String>> {
    let mut want: Option<String> = understood.wants.clone();
    let line = the_line(understood).to_lowercase();
    let stripped = text::strip(&line);
    let is = |w: &Option<String>, s: &str| w.as_deref() == Some(s);
    let has = |rx: &Re| rx.search(&line).is_some();
    // The two readings this file corrects are corrected before the shape is looked up.
    if is(&want, &STEPS) && _A_PICK_ASK.match_(stripped).is_some() {
        want = Some(PICK.clone());
    }
    // A figure worked out is asked for before the decision words are read.
    if is(&want, &STEPS) && has(&_A_WORK_OUT) {
        want = Some(AMOUNT.clone());
    } else if is(&want, &STEPS) && _A_STRAIGHT_YESNO.match_(stripped).is_some() {
        want = Some(YESNO.clone());
    }
    if is(&want, &AMOUNT) && has(&_A_TIME_WORD) {
        want = Some(TIME.clone());
    }
    // A date on something of theirs is on the thing itself.
    if has(&_A_DATE_ON_IT) && (has(&_A_WHOSE_THING) || _the_thing_is_theirs(memory)) {
        want = Some(PAPERWORK.clone());
    }
    let amount_or_time = |w: &Option<String>| is(w, &AMOUNT) || is(w, &TIME);
    let a_pick = |w: &Option<String>| is(w, &PICK) || is(w, &OPTIONS);
    // A frequency is not a job's size; a remark with no ask in it is not a choice.
    if amount_or_time(&want) && has(&_A_FREQUENCY) {
        return None;
    }
    if a_pick(&want) && _AN_ASK_AT_THE_FRONT.match_(stripped).is_none() && !has(&_TWO_THINGS) {
        return None;
    } else if amount_or_time(&want) && has(&_A_FACT_ABOUT_TIME) {
        // A date or a lead time is a fact somebody else holds.
        return None;
    }
    if is(&want, &AMOUNT) && has(&_A_STORED_COUNT) {
        return None;
    }
    // A callback is a question about this chat: only on a chat with nothing in it yet.
    let has_turns = !memory.turns.is_empty();
    if has(&_A_WHAT_WAS_IT) && has(&_A_THEY_SAID_SO) && !has_turns {
        want = Some(RECALL.clone());
    } else if has(&_A_WHAT_WAS_IT) && has_turns {
        // In a thread the store is the reader for a callback.
        return None;
    }
    // A line that carries no subject of its own gets the one question that can give it one.
    if _A_VAGUE.match_(stripped).is_some() {
        want = Some(VAGUE.clone());
    }
    // "Which trade do i need" is a lookup, not a choice; "what do i do with the rest" a next step.
    if a_pick(&want) && (has(&_WHICH_I_NEED) || has(&_WHAT_TO_DO_WITH)) {
        return None;
    }
    let want = want?;
    lines_for(&want)?;
    if _the_thread_is_heavy(memory) {
        return None;
    }
    if has_turns {
        // A yes/no that leans on the thread is still the same decision.
        if want != *YESNO && _LEANS_ON_THE_THREAD.match_(stripped).is_some() {
            return None;
        }
        if (want == *PICK || want == *STEPS || want == *OPTIONS) && _BARE_ASK.match_(stripped).is_some() && _the_bot_just_said_a_lot(memory) {
            return None;
        }
    }
    if want == *YESNO && has(&_AN_ENOUGH_ASK) {
        return lines_for(&AMOUNT);
    }
    if want == *YESNO && has(&_A_FIT_STATE) && has(&_A_POINTS_AT_A_THING) {
        return None;
    }
    if want == *YESNO && _LEANS_ON_THE_THREAD.match_(stripped).is_some() && !has(&_A_VERDICT_WORD) {
        return None;
    }
    if want == *YESNO && !has(&_A_DECISION) && !has_turns {
        return None;
    }
    if (want == *AMOUNT || want == *TIME) && (_on_paper(&line, memory) || _the_figures_are_in_the_thread(memory)) {
        return None;
    }
    let mut pool = lines_for(&want);
    if want == *PICK && _the_two_things_are_on_the_table(understood, memory) {
        pool = Some(PICK_LINES_NAMED.clone());
    }
    if _a_count_of_the_thing_just_named(the_line(understood), memory) {
        return None;
    }
    pool
}

/// Python: bot/compose/the_shape_without_the_fact.py:589 `_a_placeholder(reply, fallback)`
///
/// True when the line says nothing about this turn: a plain hedge or a bare receipt. (Python
/// passes the fallback module in; here it is `crate::compose::fallback` itself.)
pub fn _a_placeholder(reply: &str) -> bool {
    let said = text::squash(&reply.to_lowercase());
    if said.is_empty() {
        return false;
    }
    let pools = Value::Tuple(std::sync::Arc::new(vec![(*NEUTRAL).clone()]));
    if fallback::_in_a_pool(reply, &pools) {
        return true;
    }
    let heads = _A_HEADS.as_int().expect("an int") as isize;
    let mine = text::slice(&said, 0, Some(heads));
    UNKNOWN.iter().any(|line| text::slice(&line.to_lowercase(), 0, Some(heads)) == mine)
}

/// Python: bot/compose/the_shape_without_the_fact.py:599 `_a_thread_with_a_person_in_trouble(fallback, memory)`
///
/// A worry about somebody's health, a death, a crisis, or a line about the body. A reader that
/// cannot be asked does not fence.
pub fn _a_thread_with_a_person_in_trouble(memory: &Memory) -> bool {
    let readers: [fn(&Memory) -> bool; 4] = [
        fallback::_in_a_care_thread,
        fallback::_in_a_loss_thread,
        fallback::_in_a_crisis_thread,
        |m| fallback::_thread_is_about_the_body(m, 4),
    ];
    readers.iter().any(|reader| crate::compose::com4::attempt(|| reader(memory)).unwrap_or(false))
}

/// Python: bot/compose/the_shape_without_the_fact.py:615 `answer_for(text, memory, reply)`
///
/// The pool to say in place of a placeholder reply, or None to leave the reply alone.
pub fn answer_for(text_: &str, memory: &Memory, reply: &str) -> Option<String> {
    let understood = memory.understood.as_ref()?;
    if !_a_placeholder(reply) {
        return None;
    }
    let pool = the_shape_without_the_fact(understood, memory)?;
    if _a_thread_with_a_person_in_trouble(memory) {
        return None;
    }
    let clean = if understood.clean.is_empty() { text::squash(&text_.to_lowercase()) } else { understood.clean.clone() };
    crate::compose::fallback_chain::_choose(&pool, memory, &clean, None)
}

#[allow(dead_code)]
fn _shapes_named() -> [&'static str; 2] {
    [REASON.as_str(), ADVICE.as_str()]
}
