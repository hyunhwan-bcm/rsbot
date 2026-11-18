//! Port of `bot/compose/plans.py`: an ask about their own plan ("who should i go with", "what
//! time do i need to be there") answered with the one rule worth knowing and one short question.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_sets, bundle_vals};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

bundle_res!("bot.compose.plans"; TRIP_TOLD, WORK_TOLD, HEALTH_TOLD, MOVE_TOLD, EVENT_TOLD, PLACE_AND_MONTH, A_PLACE_NAMED,
    WHO_WITH_ME, WHAT_TIME, TIME_DOING, WHAT_TO_TAKE, WHAT_FIRST, WHAT_MISSED, ABOUT_THEMSELVES);
bundle_sets!("bot.compose.plans"; PLAN_NOT_A_PLACE);
bundle_vals!("bot.compose.plans"; PLAN_KINDS);

static KINDS: LazyLock<Vec<(&'static str, &'static Re)>> = LazyLock::new(|| {
    PLAN_KINDS.items().iter().map(|row| (row.items()[0].str(), row.items()[1].as_re().expect("a pattern"))).collect()
});

/// Python: bot/compose/plans.py:87 `_the_plan_told(memory, clean='')`
///
/// (kind, place, word) of a plan the chat has named, or None. Only the last few lines count.
pub fn _the_plan_told(memory: &Memory, clean: &str) -> Option<(String, String, String)> {
    let turns = &memory.turns;
    let mine = text::squash(&clean.to_lowercase());
    let mut behind = 0;
    for turn in turns[turns.len().saturating_sub(6)..].iter().rev() {
        let said = text::squash(&turn.user.to_lowercase());
        if said.is_empty() || said == mine {
            continue;
        }
        behind += 1;
        if behind > 3 {
            break;
        }
        for (kind, pattern) in KINDS.iter() {
            let Some(found) = pattern.search(&said) else { continue };
            let mut place = String::new();
            if *kind == "trip" {
                for named in A_PLACE_NAMED.finditer(&said) {
                    let word = named.str(1);
                    if !PLAN_NOT_A_PLACE.contains(word) {
                        place = word.to_string();
                        break;
                    }
                }
            }
            let word = if found.group_count() > 0 { found.str(1).to_string() } else { String::new() };
            return Some((kind.to_string(), place, word));
        }
        if let Some(named) = PLACE_AND_MONTH.search(&said) {
            if !PLAN_NOT_A_PLACE.contains(named.str(1)) {
                return Some(("trip".to_string(), named.str(1).to_string(), String::new()));
            }
        }
    }
    None
}

/// Python: bot/compose/plans.py:122 `_the_ask(clean)`
///
/// Which question about the plan the line is, or None.
pub fn _the_ask(clean: &str) -> Option<&'static str> {
    if WHAT_MISSED.search(clean).is_some() {
        return Some("missed");
    }
    if WHO_WITH_ME.search(clean).is_some() {
        return Some("who");
    }
    if WHAT_TIME.search(clean).is_some() && TIME_DOING.search(clean).is_some() {
        return Some("time");
    }
    if WHAT_TO_TAKE.search(clean).is_some() {
        return Some("take");
    }
    if WHAT_FIRST.search(clean).is_some() {
        return Some("first");
    }
    None
}

/// Python: bot/compose/plans.py:137 `answer_for(text, memory)`
///
/// The line for an ask about their own plan, or None.
pub fn answer_for(text_: &str, memory: &Memory) -> Option<String> {
    let said = text::squash(&text_.to_lowercase());
    if said.is_empty() || ABOUT_THEMSELVES.search(&said).is_none() {
        return None;
    }
    let ask = _the_ask(&said)?;
    let (kind, place, word) = _the_plan_told(memory, &said)?;
    _the_line(ask, &kind, &place, &word)
}

/// Python: bot/compose/plans.py:158 `_the_line(ask, kind, place, word='')`
///
/// One short question, and the rule worth knowing first where there is one.
pub fn _the_line(ask: &str, kind: &str, place: &str, word: &str) -> Option<String> {
    let place_cap = text::capitalize(place);
    let line = match ask {
        "who" => {
            // A friend does not know either, so the line leads with where it is written down.
            if kind == "health" {
                "Nobody has to come with you unless you are having something that needs a lift home - then ask. Is anyone going with you?".to_string()
            } else if kind == "event" {
                "Check who is named on the invitation or ticket. Who are you going with?".to_string()
            } else if !place.is_empty() {
                format!("The booking lists everyone who is travelling to {place_cap} on it. Who are you going with?")
            } else {
                "The booking lists everyone who is travelling on it. Who are you going with?".to_string()
            }
        }
        "time" => {
            let where_ = if kind == "health" {
                if !word.is_empty() && !matches!(word, "check-up" | "scan" | "clinic") {
                    format!(" at the {word}")
                } else {
                    " at the surgery".to_string()
                }
            } else if kind == "work" {
                " at work".to_string()
            } else if !place.is_empty() {
                format!(" in {place_cap}")
            } else {
                " there".to_string()
            };
            format!("What time do you need to be{where_}? Work back from the journey, and add half an hour for the queue.")
        }
        "take" => {
            if kind == "move" {
                "Pack one box of the things you need on day one and keep it with you - what's top of that list?".to_string()
            } else if !place.is_empty() {
                format!("The forecast for {place_cap} decides most of the bag, so check it the day before. Will you be on your feet most of the day?")
            } else {
                "It depends on what you will be doing, more than on the weather: on your feet all day, or sitting down?".to_string()
            }
        }
        "first" => match kind {
            "move" => "Meter readings and a change of address are the two that cost money if they slip. Are those done?".to_string(),
            "health" => "Write down what you want to ask while you're there - it's easy to forget on the day. What's the main thing?".to_string(),
            "work" => "Sort the paperwork they asked for and the route there the night before. What time are you due in?".to_string(),
            _ => "Sort the one thing that has a date on it first. What's already booked?".to_string(),
        },
        "missed" => {
            if kind == "work" {
                "Ask one person what changed while you were off - that's the whole catch-up. Who would you ask?".to_string()
            } else {
                "Most of what comes in while you're away sorts itself out. What's the one thing with a deadline this week?".to_string()
            }
        }
        _ => return None,
    };
    Some(line)
}
