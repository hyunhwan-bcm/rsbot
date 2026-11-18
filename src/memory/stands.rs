//! Port of `bot/memory/stands.py`: a yes/no about a figure the bot itself gave - "so is eight
//! weeks still right" - and "does that change the time?" after a swap.
//!
//! The figure is the answer's, so the reply is the figure with the reason the chat already gave
//! for it ("Still right: Three runs a week gets most people to 5k in about eight weeks."). Narrow:
//! the line asks, carries one figure the chat gave, and names nothing else.
#![allow(non_snake_case, clippy::all)]

use crate::data::Value;
use crate::memory::mem7::{self, inline_re, lists, res, sets, values};
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::HashSet;

res!("stands"; FIGURE, QUESTION_OPENING, ASKING, SENTENCE, CHANGE_ASK, ATTRIBUTE_WORDS, A_SETTING, FIGURED, A_METHOD_CHANGE, A_CHANGE, A_SIZE_CHANGE);
sets!("stands"; UNITS, NOTHING_NAMED, NOT_THE_SUBJECT);
lists!("stands"; STANDS, CONFIRMS);
values!("stands"; NUMBERS, MOST_WORDS, LOOK_BACK, SHORTEST_REASON, MOST_REASON_WORDS, UNIT_FAMILY, CHANGE_REPLIES, CHANGE_REPLIES_MOVED);
inline_re! {
    NOT_A_DIGIT = r"\D", 0;
    SUBJECT_WORDS = r"[a-z]{4,}", 0;
}

fn int_of(v: &Value) -> usize {
    v.as_int().expect("an int global") as usize
}

/// Python: bot/memory/stands.py:108 `_number(word)`
pub fn _number(word: &str) -> Option<f64> {
    let word = pt::lower(pt::strip_chars(word, ".,"));
    if word.is_empty() {
        return None;
    }
    if pt::is_digit(&word.replace(',', "").replace('.', "")) {
        let plain = word.replace(',', "");
        return Some(mem7::py_float(&plain).unwrap_or_else(|| panic!("ValueError: could not convert string to float: {plain:?}")));
    }
    NUMBERS.get(&word).and_then(Value::as_int).map(|n| n as f64)
}

/// A figure in a line: (value, unit or "", where it starts, where it ends), byte offsets.
pub type Figure = (f64, String, usize, usize);

/// Python: bot/memory/stands.py:117 `_figures(text)`
pub fn _figures(text: &str) -> Vec<Figure> {
    let mut found = Vec::new();
    for m in FIGURE.finditer(text) {
        let Some(value) = _number(m.str(1)) else {
            continue;
        };
        let unit = pt::rstrip_chars(&pt::lower(m.str(2)), "s").to_string();
        found.push((value, unit, m.start(), m.end()));
    }
    found
}

/// Python: bot/memory/stands.py:129 `_names_the_figure(text, value, unit)`
///
/// True when this text gives the same figure, with the same unit where the line names one.
pub fn _names_the_figure(text: &str, value: f64, unit: &str) -> bool {
    _figures(text).iter().any(|(other, other_unit, _, _)| *other == value && (unit.is_empty() || unit == other_unit))
}

/// Python: bot/memory/stands.py:145 `_the_line_asks_about_a_figure_and_elsewhere_nothing(text)`
///
/// The (value, unit, phrase) the line asks about, or None when it names something of its own.
pub fn _the_line_asks_about_a_figure_and_elsewhere_nothing(text: &str) -> Option<(f64, String, String)> {
    let line = pt::squash(&pt::lower(text).replace('\u{2019}', "'"));
    if line.is_empty() || mem7::word_count(&line) > int_of(&MOST_WORDS) {
        return None;
    }
    if !(QUESTION_OPENING.is_match(&line) || ASKING.is_match(&line)) || !ASKING.is_match(&line) {
        return None;
    }
    let figures = _figures(&line);
    if figures.len() != 1 {
        return None; // none, or a comparison
    }
    let (value, unit, start, end) = figures.into_iter().next().expect("one");
    let rest = format!("{} {}", &line[..start], &line[end..]);
    for word in pt::split_ref(&rest) {
        let word = pt::strip_chars(word, ".,!?;:");
        if NOTHING_NAMED.contains(word) || UNITS.contains(pt::rstrip_chars(word, "s")) || matches!(word, "a" | "an" | "per" | "each" | "every") {
            continue;
        }
        if NOTHING_NAMED.contains(pt::rstrip_chars(word, "'s")) {
            continue;
        }
        return None;
    }
    Some((value, unit, pt::strip(&line[start..end]).to_string()))
}

/// Python: bot/memory/stands.py:172 `_the_chat_gave_it(memory, value, unit)`
///
/// True when one of the bot's own recent answers gives this figure with this unit.
pub fn _the_chat_gave_it(memory: &Memory, value: f64, unit: &str) -> bool {
    mem7::tail(&memory.turns, int_of(&LOOK_BACK)).iter().any(|turn| _names_the_figure(&turn.bot, value, unit))
}

/// Python: bot/memory/stands.py:183 `_the_reason_behind(memory, value, unit)`
///
/// The bot's own sentence that gives this figure and says what it rests on, or "".
pub fn _the_reason_behind(memory: &Memory, value: f64, unit: &str) -> String {
    for turn in mem7::tail(&memory.turns, int_of(&LOOK_BACK)).iter().rev() {
        let said = &turn.bot;
        if said.is_empty() || !_names_the_figure(said, value, unit) {
            continue;
        }
        for sentence in SENTENCE.split(said) {
            let sentence = pt::strip(&sentence);
            let words = mem7::word_count(sentence);
            if !(int_of(&SHORTEST_REASON) <= words && words <= int_of(&MOST_REASON_WORDS)) {
                continue;
            }
            if _names_the_figure(sentence, value, unit) {
                return sentence.to_string();
            }
        }
    }
    String::new()
}

/// Python: bot/memory/stands.py:208 `the_figure_asked_about(text, memory)`
///
/// (value, unit, phrase, reason) when the line asks whether a figure of the chat stands.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the quadruple or None.
pub fn the_figure_asked_about(text: &str, memory: &Memory) -> Option<(f64, String, String, String)> {
    let (value, unit, phrase) = _the_line_asks_about_a_figure_and_elsewhere_nothing(text)?;
    if !_the_chat_gave_it(memory, value, &unit) {
        return None; // the figure is not the chat's
    }
    let reason = _the_reason_behind(memory, value, &unit);
    Some((value, unit, phrase, reason))
}

/// Python: bot/memory/stands.py:300 `_in_minutes(number, unit)`
///
/// A time figure as a number of minutes, so "1 hour 40 minutes" beats "20 minutes".
pub fn _in_minutes(number: &str, unit: &str) -> f64 {
    let Some(value) = _number(number) else {
        return 0.0;
    };
    match pt::rstrip_chars(&pt::lower(unit), "s") {
        "hour" | "hr" => value * 60.0,
        "day" => value * 1440.0,
        "week" => value * 10080.0,
        "month" => value * 43200.0,
        _ => value,
    }
}

/// A figure of an answer: (value, unit, as said).
pub type Candidate = (String, String, String);

/// Python: bot/memory/stands.py:317 `_the_best(candidates, family)`
///
/// The figure to hand back: the whole time rather than a rate, and the first temperature.
pub fn _the_best(candidates: &[Candidate], family: &str) -> String {
    let Some(first) = candidates.first() else {
        return String::new();
    };
    if family == "temperature" {
        return first.2.clone();
    }
    // `max(...)`: the first of the largest.
    let key = |c: &Candidate| _in_minutes(pt::split_ref(&c.0).first().copied().unwrap_or(""), &c.1);
    let mut best = first;
    for candidate in &candidates[1..] {
        if key(candidate) > key(best) {
            best = candidate;
        }
    }
    best.2.clone()
}

/// Python: bot/memory/stands.py:330 `_figures_in_answer(said, family)`
///
/// The figures one answer gives for this attribute, as [(value, unit, as said)].
pub fn _figures_in_answer(said: &str, family: &str) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    for m in FIGURED.finditer(said) {
        if let Some(hours) = m.get(1) {
            let whole = format!("{} {} {} {}", hours, pt::lower(m.str(2)), m.str(3), pt::lower(m.get(4).unwrap_or("minutes")));
            let minutes = _in_minutes(hours, "hour") + _in_minutes(m.str(3), "minute");
            if family != "time" {
                continue;
            }
            candidates.push((pt::float_repr(minutes), "minute".to_string(), whole));
            continue;
        }
        let unit_said = pt::lower(m.str(6));
        let unit = pt::rstrip_chars(&unit_said, "s").to_string();
        if UNIT_FAMILY.get(&unit).and_then(Value::as_str) != Some(family) {
            continue;
        }
        let figure = if unit == "c" || unit == "\u{00b0}c" { format!("{}C", m.str(5)) } else { format!("{} {}", m.str(5), unit_said) };
        candidates.push((m.str(5).to_string(), unit_said, figure));
    }
    candidates
}

/// Python: bot/memory/stands.py:353 `_the_same_figure(one, other)`
///
/// True when two figures are the same one: "12 to 15 minutes" and "12-15 minutes".
pub fn _the_same_figure(one: &str, other: &str) -> bool {
    NOT_A_DIGIT.sub("", one) == NOT_A_DIGIT.sub("", other)
}

/// Python: bot/memory/stands.py:367 `_subjects(said)`
pub fn _subjects(said: &str) -> HashSet<String> {
    SUBJECT_WORDS.findall_str(&pt::lower(said)).into_iter().filter(|w| !NOT_THE_SUBJECT.contains(w)).collect()
}

/// Python: bot/memory/stands.py:372 `_about_the_same_thing(one, other)`
pub fn _about_the_same_thing(one: &str, other: &str) -> bool {
    !_subjects(one).is_disjoint(&_subjects(other))
}

/// Python: bot/memory/stands.py:386 `_a_figure_for(memory, family)`
///
/// (figure, moved) - the figure to hand back, and whether the swap moved it.
pub fn _a_figure_for(memory: &Memory, family: &str) -> (String, bool) {
    let answers: Vec<&str> = mem7::tail(&memory.turns, 2).iter().map(|turn| turn.bot.as_str()).collect();
    let Some(first) = answers.first() else {
        return (String::new(), false);
    };
    let plan = _the_best(&_figures_in_answer(first, family), family);
    let swapped = answers.get(1).map_or(String::new(), |second| _the_best(&_figures_in_answer(second, family), family));
    if swapped.is_empty() {
        return (plan, false);
    }
    if plan.is_empty() {
        return (swapped, true);
    }
    if _the_same_figure(&swapped, &plan) {
        return (plan, false);
    }
    if _about_the_same_thing(answers[0], answers[1]) {
        return (swapped, true);
    }
    let said = memory.turns.last().map_or("", |turn| turn.user.as_str());
    if A_METHOD_CHANGE.is_match(said) {
        return (String::new(), false);
    }
    (plan, false)
}

/// The person's last three lines.
fn last_lines(memory: &Memory) -> Vec<&str> {
    mem7::tail(&memory.turns, 3).iter().map(|turn| turn.user.as_str()).collect()
}

/// Python: bot/memory/stands.py:438 `_a_change_was_made(memory)`
///
/// True when one of the last few lines of theirs swaps something for something else.
pub fn _a_change_was_made(memory: &Memory) -> bool {
    last_lines(memory).iter().any(|line| A_CHANGE.is_match(line))
}

/// Python: bot/memory/stands.py:463 `_a_change_of_size(memory)`
///
/// True when the change the person just made was a size or a quantity.
pub fn _a_change_of_size(memory: &Memory) -> bool {
    last_lines(memory).iter().any(|line| A_SIZE_CHANGE.is_match(line))
}

/// Python: bot/memory/stands.py:472 `the_change_asked_about(text, memory)`
///
/// (attribute, figure, moved) when the line asks whether a change moves a figure, else None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the triple or None.
pub fn the_change_asked_about(text: &str, memory: &Memory) -> Option<(String, String, bool)> {
    let line = pt::squash(&pt::lower(text).replace('\u{2019}', "'"));
    if line.is_empty() || mem7::word_count(&line) > int_of(&MOST_WORDS) + 3 {
        return None;
    }
    if !CHANGE_ASK.is_match(&line) || A_SETTING.is_match(&line) {
        return None;
    }
    if _a_change_of_size(memory) || !_a_change_was_made(memory) {
        return None;
    }
    let named = ATTRIBUTE_WORDS.search(&line)?;
    let (start, end) = named.span(1).expect("the attribute group");
    let family = match pt::lower(named.str(1)).as_str() {
        "time" | "cooking time" | "oven time" | "long" => "time",
        "temperature" | "heat" => "temperature",
        _ => return None,
    };
    // Every other word has to name nothing.
    let rest = format!("{} {}", &line[..start], &line[end..]);
    for word in pt::split_ref(&rest) {
        let word = pt::strip_chars(word, ".,!?;:");
        if word.is_empty() || NOTHING_NAMED.contains(word) || matches!(word, "that" | "this" | "it" | "which") {
            continue;
        }
        if matches!(
            word,
            "change" | "changed" | "changes" | "affect" | "affects" | "alter" | "alters" | "impact" | "matter" | "difference" | "make" | "makes" | "made" | "a" | "an" | "any" | "to" | "for" | "the" | "on" | "of" | "in" | "with" | "much" | "long" | "time" | "cooking" | "oven"
        ) {
            continue;
        }
        return None;
    }
    let (figure, moved) = _a_figure_for(memory, family);
    if figure.is_empty() {
        return None;
    }
    Some((family.to_string(), figure, moved))
}

/// Python: bot/memory/stands.py:515 `change_reply(text, memory)`
///
/// The reply to "does that change the time?" - the chat's own figure, handed back.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn change_reply(text: &str, memory: &Memory) -> Option<String> {
    let (attribute, figure, moved) = the_change_asked_about(text, memory)?;
    let line = pt::squash(&pt::lower(text));
    let pools = if moved { &*CHANGE_REPLIES_MOVED } else { &*CHANGE_REPLIES };
    let pool = pools.get(&attribute).filter(|p| p.truthy()).or_else(|| pools.get("time")).expect("KeyError: 'time'").strs();
    Some(mem7::fmt(&mem7::pick(&pool, memory, &line), &[("figure", &figure)]))
}

/// Python: bot/memory/stands.py:527 `reply_for(text, memory)`
///
/// The reply to a yes/no about a figure the bot gave, or None for every other line.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    let (_, _, phrase, reason) = the_figure_asked_about(text, memory)?;
    let line = pt::squash(&pt::lower(text));
    if reason.is_empty() {
        return Some(mem7::fmt(&mem7::pick(&CONFIRMS, memory, &line), &[("figure", &phrase)]));
    }
    Some(mem7::fmt(&mem7::pick(&STANDS, memory, &line), &[("reason", &reason)]))
}
