//! Port of `bot/knowledge/sleep.py`: sleep (a bedtime, caffeine and sleep, a machine left on).

use crate::data::FnRef;
use crate::knowledge::d2dup_g10::{globals, re_cached};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

globals!("bot.knowledge.sleep",
    re { _BED_CLOCK, _BED_AT_HOUR },
    value { _BED_WORD_HOUR, _HOUR_WORDS },
    strs { _WIDE_OPEN, SLEEPLESS, MACHINES, SWITCHED_ON },
    string { _CUTTING_DOWN, _CUP_GOES });

static HALF: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhalf\s+(\d{1,2})\b", 0));
static MIDNIGHT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bmidnight\b", 0));
static LATE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(late|later|midnight)\b", 0));
static SMALL_HOURS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:one|two|three|eleven|twelve)\b", 0));
static CUTTING_DOWN: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        r"\bheadache|headaches|withdrawal|withdrawals|shaky|irritable|stopped|stopping|quit|quitting|without|cut(ting)? (down|out|back)\b",
        0,
    )
});

/// An hour of the day as it is said: "midnight", "noon", "11pm", "9am".
pub fn _clock_said(hour: i64) -> String {
    if hour == 0 {
        return "midnight".into();
    }
    if hour == 12 {
        return "noon".into();
    }
    if hour > 12 {
        return format!("{}pm", hour - 12);
    }
    format!("{hour}am")
}

/// An evening hour: 12 is midnight, 6 to 11 are pm.
fn evening(hour: i64) -> i64 {
    if hour == 12 {
        0
    } else if (6..=11).contains(&hour) {
        hour + 12
    } else {
        hour
    }
}

/// The bedtime arithmetic, said back with the number it gives.
pub fn _bedtime_said(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = _line(understood);
    let mut hour: Option<i64> = None;
    if let Some(found) = _BED_CLOCK.search(&line) {
        let mut h: i64 = found.str(1).parse().unwrap_or(0);
        let when = found.str(2).to_lowercase();
        if when.starts_with('p') && h < 12 {
            h += 12;
        } else if when.starts_with('a') && h == 12 {
            h = 0;
        }
        hour = Some(h);
    }
    if hour.is_none() {
        if let Some(half) = HALF.search(&line) {
            let h: i64 = half.str(1).parse().unwrap_or(0);
            hour = Some(if h <= 11 { h + 12 } else { h });
        }
    }
    if hour.is_none() && MIDNIGHT.is_match(&line) {
        hour = Some(0);
    }
    if hour.is_none() {
        if let Some(found) = _BED_AT_HOUR.search(&line) {
            hour = Some(evening(found.str(1).parse().unwrap_or(0)));
        }
    }
    if hour.is_none() {
        for (word, number) in _BED_WORD_HOUR.pairs() {
            if re_cached(&format!(r"\b{}\b", word.str()), 0).is_match(&line) {
                hour = Some(evening(number.as_int().unwrap_or(0)));
                break;
            }
        }
    }
    let Some(hour) = hour else {
        if _WIDE_OPEN.iter().any(|w| line.contains(w.as_str())) {
            return Some("A bedtime that moves every night is harder to sleep on than a late one that holds: fix the wake-up time first and keep it seven days a week, get daylight before noon, and let the bedtime find its own level. Move it 15 minutes earlier every few nights rather than all at once.".into());
        }
        return Some("Late enough that it is the hours rather than the sleep itself: pick a wake-up time you can keep seven days a week and the bedtime follows within a week or two, because the morning light is what sets the clock. No caffeine after two.".into());
    };
    let hours = (7 - hour).rem_euclid(24);
    let said = _clock_said(hour);
    let count = _HOUR_WORDS
        .pairs()
        .iter()
        .find(|(k, _)| k.as_int() == Some(hours))
        .map(|(_, v)| v.str().to_string())
        .unwrap_or_else(|| hours.to_string());
    if hours >= 7 {
        return Some(format!(
            "{} is a good bedtime - {count} hours before a seven o'clock start - and keeping it the same seven days a week matters more than the exact hour: dark, cool room, phone outside the bedroom.",
            text::capitalize(&said)
        ));
    }
    Some(format!(
        "Bed at {said} is {count} hours if you are up at seven, and most adults need seven to nine. Don't force an early night: get up at the same time every day, daylight before noon, no caffeine after two, and the bedtime follows the wake-up time within a week or two."
    ))
}

/// True only when the line names a bedtime: a clock, or the word late for one.
pub fn _the_clock_i_go_to_bed(understood: &Understood) -> bool {
    let line = _line(understood);
    if _WIDE_OPEN.iter().any(|w| line.contains(w.as_str())) {
        return true;
    }
    if _BED_CLOCK.is_match(&line) || _BED_AT_HOUR.is_match(&line) {
        return true;
    }
    if LATE.is_match(&line) {
        return true;
    }
    SMALL_HOURS.is_match(&line)
}

/// The repaired line, lower case, with a space at each end, for a whole-word search.
pub fn _line(understood: &Understood) -> String {
    let text = if understood.plain.is_empty() { &understood.clean } else { &understood.plain };
    format!(" {} ", text.to_lowercase())
}

/// How long a cup stays in you, or what cutting down does.
pub fn _caffeine_timeline(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    if CUTTING_DOWN.is_match(&_line(understood)) {
        return Some(_CUTTING_DOWN.clone());
    }
    Some(_CUP_GOES.clone())
}

/// False when the line names a machine in the house, says it is switched on, and is not
/// somebody saying they themselves cannot sleep.
pub fn _a_machine_that_is_on_is_not_a_sleeper(understood: &Understood) -> bool {
    let text = format!(" {} ", understood.text.to_lowercase());
    if SLEEPLESS.iter().any(|p| text.contains(p.as_str())) {
        return true;
    }
    if !MACHINES.iter().any(|n| text.contains(&format!(" {n} ")) || text.contains(&format!(" {n}s "))) {
        return true;
    }
    !SWITCHED_ON.iter().any(|p| text.contains(p.as_str()))
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.sleep:_the_clock_i_go_to_bed:141", |_: &FnRef, u: &Understood| _the_clock_i_go_to_bed(u)),
    ("bot.knowledge.sleep:_a_machine_that_is_on_is_not_a_sleeper:201", |_: &FnRef, u: &Understood| {
        _a_machine_that_is_on_is_not_a_sleeper(u)
    }),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.sleep:_bedtime_said:82", |_: &FnRef, m: &mut Memory, u: &Understood| _bedtime_said(m, u)),
    ("bot.knowledge.sleep:_caffeine_timeline:170", |_: &FnRef, m: &mut Memory, u: &Understood| _caffeine_timeline(m, u)),
];
