//! Port of `bot/knowledge/kit_lists.py`: what to take for one occasion (a day at the beach, ...).

use crate::data;
use crate::data::FnRef;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

static _NEEDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.kit_lists._NEEDS"));
static _A_TRIP_OF_SOME_LENGTH: LazyLock<&'static Re> =
    LazyLock::new(|| data::re("bot.knowledge.kit_lists._A_TRIP_OF_SOME_LENGTH"));

/// True when the line is about a stay of some days rather than one occasion.
pub fn _a_trip_of_some_length(understood: &Understood) -> bool {
    let mut said = if understood.text.is_empty() { understood.clean.clone() } else { understood.text.clone() };
    if said.is_empty() {
        said = understood.words.join(" ");
    }
    _A_TRIP_OF_SOME_LENGTH.is_match(&said)
}

/// True when the line is asking what to have with them, for one occasion and not a trip.
pub fn _asks_for_the_kit(understood: &Understood) -> bool {
    _NEEDS.iter().any(|w| understood.words.contains(w)) && !_a_trip_of_some_length(understood)
}

/// True when a turn or two back named a stay of some days rather than the day itself.
pub fn _a_trip_said_before(memory: &Memory) -> bool {
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(4);
    let earlier: Vec<&str> = turns[start..].iter().map(|t| t.user.as_str()).collect();
    _A_TRIP_OF_SOME_LENGTH.is_match(&earlier.join(" "))
}

/// The kit for a day at the beach, or nothing when the conversation is a trip away.
pub fn _the_day_at_the_beach(memory: &mut Memory, _understood: &Understood) -> Option<String> {
    if _a_trip_said_before(memory) {
        return None;
    }
    Some(
        "Factor 30 or higher sun cream, a bottle of water each, a hat, and a windbreak or \
umbrella for shade, plus a towel each and one spare for the car. Take cash for \
parking and a bag for your rubbish, because bins on the sand are thin on the \
ground."
            .to_string(),
    )
}

pub const GUARDS: &[(&str, GuardFn)] =
    &[("bot.knowledge.kit_lists:_asks_for_the_kit:45", |_: &FnRef, u: &Understood| _asks_for_the_kit(u))];

pub const REPLIES: &[(&str, ReplyFn)] = &[(
    "bot.knowledge.kit_lists:_the_day_at_the_beach:65",
    |_: &FnRef, m: &mut Memory, u: &Understood| _the_day_at_the_beach(m, u),
)];
