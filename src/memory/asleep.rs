//! Port of `bot/memory/asleep.py`: the night the person described - two ends of it, and the
//! hours between them.
//!
//! "i go to bed at about 11" / "and wake at 6" is one night, and the answer is the length
//! between them read against the seven to nine hours most adults need. The rule is narrow: a
//! line with both ends answers on its own; a wake time alone has to carry on the night the turn
//! before described; nobody else's night is read; a wake time is a morning one; and a pair that
//! gives under two hours or over fourteen is not a night's sleep.
#![allow(non_snake_case, clippy::all)]

use crate::data;
use crate::memory::mem7::{self, lists, res};
use crate::py::re::{self, Match, Re};
use crate::py::text as pt;
use crate::state::Memory;
use std::sync::LazyLock;

res!("asleep"; SOMEONE_ELSE, THEY_NOT_ME, NIGHT_WAKING, CARRIES_ON, HAS_A_SUBJECT, I_WAKE, BED_MARKER, WAKE_MARKER, ASKS_A_BEDTIME);
lists!("asleep"; SHORT, FINE, LONG);

/// `re.search(_ATOM, window, re.I)`.
static ATOM: LazyLock<Re> = LazyLock::new(|| Re::new(&data::string("bot.memory.asleep._ATOM"), re::I));
static WORD_HOURS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| {
    data::get("bot.memory.asleep._WORD_HOURS")
        .pairs()
        .iter()
        .map(|(word, hour)| (word.str().to_string(), hour.as_int().unwrap_or(0)))
        .collect()
});

fn word_hour(word: &str) -> Option<i64> {
    let word = pt::lower(word);
    WORD_HOURS.iter().find(|(key, _)| *key == word).map(|(_, hour)| *hour)
}

/// Python: bot/memory/asleep.py:92 `_hour_from(atom)`
///
/// The hour an atom names (a whole hour, or a half past as a fraction), or None when the atom
/// cannot be read on its own.
pub fn _hour_from(atom: Option<&Match>) -> Option<f64> {
    let atom = atom?;
    let said = pt::lower(pt::strip(atom.name_str("atom")));
    if said.starts_with("midnight") {
        return Some(0.0);
    }
    if said.starts_with("noon") {
        return Some(12.0);
    }
    if let Some(half) = atom.name("half").filter(|half| !half.is_empty()) {
        let hour = word_hour(half)? as f64;
        return Some(if (12.0..=23.0).contains(&hour) { hour + 0.5 } else { hour + 12.5 });
    }
    if let Some(word) = atom.name("word").filter(|word| !word.is_empty()) {
        // A word hour is the time only when the line says it as one: "bed at eleven".
        return word_hour(word).map(|hour| hour as f64);
    }
    let hour = mem7::int_str(atom.name_str("clock")).expect("ValueError: the clock group is digits") as f64;
    let when = pt::lower(atom.name_str("when")).replace('.', "");
    if when.starts_with('p') {
        return Some(if hour < 12.0 { hour + 12.0 } else { hour });
    }
    if when.starts_with('a') {
        return Some(if hour == 12.0 { 0.0 } else { hour });
    }
    Some(hour)
}

/// Python: bot/memory/asleep.py:119 `_end(marker, text, after_bed, limit=26)`
///
/// The time said just after the word that names one end of the night, or None.
pub fn _end(marker: &Re, text: &str, after_bed: bool, limit: i64) -> Option<f64> {
    for found in marker.finditer(text) {
        let window = &text[found.end()..mem7::chars_on(text, found.end(), limit.max(0) as usize)];
        let Some(atom) = ATOM.search(window) else {
            continue;
        };
        if after_bed && WAKE_MARKER.is_match(&window[..atom.start()]) {
            // The hour after the word "bed" is the time they get *up* when the line says so
            // ("go to bed to wake at 6"): one end of the night would become both ends.
            continue;
        }
        let Some(hour) = _hour_from(Some(&atom)) else {
            continue;
        };
        let has = |name: &str| atom.name(name).is_some_and(|group| !group.is_empty());
        if has("when") {
            return Some(hour);
        }
        if has("clock") {
            let minutes = atom.name("minutes").filter(|m| !m.is_empty()).map_or(0.0, |m| mem7::int_str(m).unwrap_or(0) as f64) / 60.0;
            if after_bed {
                if hour == 12.0 {
                    return Some(minutes);
                }
                return if (6.0..=11.0).contains(&hour) { Some(hour + 12.0 + minutes) } else { None };
            }
            return if (4.0..=11.0).contains(&hour) { Some(hour + minutes) } else { None };
        }
        // A word hour or a half hour is a bedtime when it is one and a morning when it is one.
        if after_bed {
            return Some(if (6.0..=11.0).contains(&hour) { hour + 12.0 } else { hour });
        }
        return if (4.0..=11.0).contains(&hour) { Some(hour) } else { None };
    }
    None
}

/// Python: bot/memory/asleep.py:152 `_ends(line, wake_only=False)`
///
/// The bedtime and the wake time a line names, as (bed, wake), either one possibly None.
pub fn _ends(line: &str, wake_only: bool) -> (Option<f64>, Option<f64>) {
    let text = format!(" {} ", pt::lower(line));
    let bed = if wake_only { None } else { _end(&BED_MARKER, &text, true, 26) };
    let wake = _end(&WAKE_MARKER, &text, false, 26);
    (bed, wake)
}

/// Python: bot/memory/asleep.py:160 `_last_night(memory)`
///
/// The bedtime and the wake time the chat has given, newest line first.
pub fn _last_night(memory: &Memory) -> (Option<f64>, Option<f64>) {
    let (mut bed, mut wake) = (None, None);
    for turn in mem7::tail(&memory.turns, 6).iter().rev() {
        let said = turn.user.as_str();
        if pt::strip(said).is_empty() {
            continue;
        }
        let low = pt::lower(said);
        if SOMEONE_ELSE.is_match(&low) || THEY_NOT_ME.is_match(&low) {
            return (None, None);
        }
        let (one_bed, one_wake) = _ends(said, false);
        if bed.is_none() && one_bed.is_some() {
            bed = one_bed;
        }
        if wake.is_none() && one_wake.is_some() {
            wake = one_wake;
        }
        if bed.is_some() && wake.is_some() {
            break;
        }
    }
    (bed, wake)
}

/// Python: bot/memory/asleep.py:183 `_a_length(hours)`
///
/// A length of a night the way a person says it: seven hours, seven and a half hours.
pub fn _a_length(hours: f64) -> String {
    let whole = hours.trunc() as i64;
    let half = pt::round_half_even((hours - whole as f64) * 60.0);
    if half == 30 {
        return format!("{} and a half hours", _a_word(whole));
    }
    if half != 0 {
        return format!("{} hours and {} minutes", _a_word(whole), half);
    }
    format!("{} hours", _a_word(whole))
}

/// Python: bot/memory/asleep.py:194 `_a_word(number)`
pub fn _a_word(number: i64) -> String {
    let words = ["two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve", "thirteen", "fourteen"];
    match number {
        2..=14 => words[(number - 2) as usize].to_string(),
        _ => number.to_string(),
    }
}

/// Python: bot/memory/asleep.py:220 `_seen(memory, reply)`
///
/// The reply, remembered, so the next turn can add to it rather than say it again.
pub fn _seen(memory: &mut Memory, reply: &str) -> String {
    memory.set("asleep_said", mem7::first_chars(reply, 120));
    reply.to_string()
}

/// Python: bot/memory/asleep.py:227 `answer_for(text, memory=None)`
///
/// The hours between the two ends of the night they described, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn answer_for(text: &str, memory: &mut Memory) -> Option<String> {
    let said = pt::squash(text);
    if said.is_empty() || mem7::word_count(&said) > 14 {
        return None;
    }
    let low = pt::lower(&said);
    if ASKS_A_BEDTIME.is_match(&low) {
        return None;
    }
    let (mut bed, wake) = _ends(&said, false);
    if bed.is_none() && wake.is_some() {
        // One end given: only the other end of the night the turn before described, and only
        // when the line is that night continued rather than a sentence about waking up.
        if SOMEONE_ELSE.is_match(&low) || THEY_NOT_ME.is_match(&low) || NIGHT_WAKING.is_match(&low) {
            return None;
        }
        let mine_and_short = I_WAKE.match_(&low).is_some() && mem7::word_count(&said) <= 6;
        let carries_on = CARRIES_ON.match_(&low).is_some() || !HAS_A_SUBJECT.is_match(&low) || mine_and_short;
        if !carries_on {
            return None;
        }
        bed = _last_night(memory).0;
    }
    let (Some(bed), Some(wake)) = (bed, wake) else {
        return None;
    };
    if SOMEONE_ELSE.is_match(&low) {
        return None;
    }
    let hours = (wake - bed).rem_euclid(24.0);
    if hours < 2.0 || hours > 14.0 {
        return None;
    }
    let shapes = if hours < 7.0 {
        &*SHORT
    } else if hours <= 9.0 {
        &*FINE
    } else {
        &*LONG
    };
    let reply = mem7::fmt(&mem7::pick(shapes, memory, &said), &[("length", &_a_length(hours))]);
    Some(_seen(memory, &pt::upper_first(&reply)))
}
