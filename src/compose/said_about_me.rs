//! Port of `bot/compose/said_about_me.py`: someone's judgement of the person, reported ("my mum
//! says im not trying hard enough"), answered as what it is like to be spoken to that way.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_strs};
use crate::compose::pick;
use crate::py::text;
use crate::state::Memory;

bundle_res!("bot.compose.said_about_me"; REPORTED_LABEL, REPORTED_ACT, _DANGER, _PARTNER, _GRANDPARENT, _PARENT,
    _SIBLING, _WORK, _SCHOOL, _REPORTING, _A_HABIT, _SELF_ADDRESSED);
bundle_strs!("bot.compose.said_about_me"; _PARENT_LINES, _GRANDPARENT_LINES, _SIBLING_LINES, _WORK_LINES,
    _SCHOOL_LINES, _ACCUSED_LINES, _GENERIC_LINES, _HABIT_LINES);

/// Python: bot/compose/said_about_me.py:189 `reads_as_somebody_elses_words(text)`
///
/// Whether the person is quoting what somebody close calls them, in their own words.
pub fn reads_as_somebody_elses_words(text: &str) -> bool {
    let said = text::squash(text);
    !said.is_empty() && _REPORTING.search(&said).is_some()
}

/// Python: bot/compose/said_about_me.py:195 `_kind(text)`
///
/// Which of the two the line is: "label" for a name they were called, "act" for a charge about
/// what they do, and "" for neither.
pub fn _kind(text: &str) -> String {
    let said = text::squash(text);
    if said.is_empty() || said.contains('?') {
        return String::new();
    }
    if _DANGER.search(&said).is_some() || _PARTNER.search(&said).is_some() {
        return String::new();
    }
    if REPORTED_LABEL.search(&said).is_some() {
        return "label".to_string();
    }
    if REPORTED_ACT.search(&said).is_some() {
        return "act".to_string();
    }
    String::new()
}

/// Python: bot/compose/said_about_me.py:210 `reads_as_reported_judgement(text)`
///
/// Whether the line is somebody close handing the person a verdict about themselves.
pub fn reads_as_reported_judgement(text: &str) -> bool {
    !_kind(text).is_empty()
}

/// Python: bot/compose/said_about_me.py:233 `_user_lines(memory, how_many=3)`
pub fn _user_lines(memory: &Memory, how_many: i64) -> Vec<String> {
    if memory.turns.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<String> = Vec::new();
    for turn in com4::tail(&memory.turns, how_many * 2) {
        let line = turn.user.clone();
        // The store records the turn it is given and the store's copy of it, so the same line
        // can arrive twice in a row; the thread is read from the lines, not from the records.
        if out.last() != Some(&line) {
            out.push(line);
        }
    }
    com4::tail(&out, how_many).to_vec()
}

/// Python: bot/compose/said_about_me.py:249 `_on_the_thread(memory)`
///
/// Whether the person has just been talking about being spoken to that way. The judgement
/// itself, or the remark coming round again, is the same thread, so a third turn stays on it.
pub fn _on_the_thread(memory: &Memory) -> bool {
    let lines = _user_lines(memory, 3);
    let Some(last) = lines.last() else { return false };
    if reads_as_reported_judgement(last) {
        return true;
    }
    lines.len() > 1 && _A_HABIT.search(last).is_some() && reads_as_reported_judgement(&lines[lines.len() - 2])
}

/// Python: bot/compose/said_about_me.py:264 `answer_followup(text, memory=None)`
///
/// A reply when the judgement just described is said to happen again and again, or "".
pub fn answer_followup(text: &str, memory: &Memory) -> String {
    let said = text::squash(text);
    if said.is_empty() || said.contains('?') || text::split_ref(&said).len() > 12 {
        return String::new();
    }
    if _DANGER.search(&said).is_some() || _A_HABIT.search(&said).is_none() {
        return String::new();
    }
    if !_on_the_thread(memory) {
        return String::new();
    }
    pick(&_HABIT_LINES, memory, &said)
}

/// Python: bot/compose/said_about_me.py:286 `speaks_to_a_self_judgement(reply)`
///
/// Whether the reply answers the person as if they had judged themselves.
pub fn speaks_to_a_self_judgement(reply: &str) -> bool {
    let said = text::squash(reply);
    !said.is_empty() && _SELF_ADDRESSED.search(&said).is_some()
}

/// Python: bot/compose/said_about_me.py:292 `answer_for(text, memory=None)`
///
/// A reply to a reported judgement, or "" when the line is not one.
pub fn answer_for(text: &str, memory: &Memory) -> String {
    if !reads_as_reported_judgement(text) {
        return String::new();
    }
    let said = text::squash(text);
    if _kind(&said) == "act" {
        return pick(&_ACCUSED_LINES, memory, &said);
    }
    let pool: &Vec<String> = if _SCHOOL.search(&said).is_some() {
        &_SCHOOL_LINES
    } else if _WORK.search(&said).is_some() {
        &_WORK_LINES
    } else if _SIBLING.search(&said).is_some() {
        &_SIBLING_LINES
    } else if _GRANDPARENT.search(&said).is_some() {
        &_GRANDPARENT_LINES
    } else if _PARENT.search(&said).is_some() {
        &_PARENT_LINES
    } else {
        &_GENERIC_LINES
    };
    pick(pool, memory, &said)
}
