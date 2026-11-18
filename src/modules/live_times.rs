//! Port of `bot/modules/live_times.py`: a bus or train time is live, and the bot says it cannot
//! see it from here and where the next one is shown (the operator's app, the board at the stop).
#![allow(non_snake_case, clippy::all)]

use super::chat::{dyn_re, py_format};
use crate::data::{self, Value};
use crate::py::re::{Re, I};
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

macro_rules! res {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.live_times.", stringify!($n))));)* };
}
macro_rules! vals {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<&'static Value> = LazyLock::new(|| data::get(concat!("bot.modules.live_times.", stringify!($n))));)* };
}

res!(
    VEHICLE, TIME, STATUS, PLATFORM, PLACE_OPENING, MONEY, ADVICE, DELAYED, BOARD_ASK, FLIGHT_LIVE,
    ASK, WHEN_ASK, NEWS, SERVICE, CLOCK,
);
vals!(VEHICLE_STEM, BUS, THE_BOARD_THAT_KEEPS_THEM, RAIL, FLIGHT, FERRY);

static DEST_CUT: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\b(?:cancelled|canceled|delayed|running|tomorrow|today|tonight|please|leave|leaves|go|goes|departs?|arrives?)\b", 0)
});
static DANGLING: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?:\s*\b(?:of|the|and|a|an|to|from|in|on|at|for|by)\b)+$", I));

/// Python: bot/modules/live_times.py:91 `_service_named(clean)`
pub fn _service_named(clean: &str) -> Option<String> {
    let m = SERVICE.search(clean)?;
    let words: Vec<&str> = [m.name("clock"), m.name("qual"), m.name("vehicle")].into_iter().flatten().filter(|p| !p.is_empty()).collect();
    let mut name = words.join(" ");
    if let Some(dest) = m.name("dest").filter(|d| !d.is_empty()) {
        let cut = match DEST_CUT.search(dest) {
            Some(c) => &dest[..c.start()],
            None => dest,
        };
        let dest = DANGLING.sub("", text::strip(cut));
        let dest = text::strip(&dest);
        if !dest.is_empty() {
            name = format!("{name} {} {dest}", m.name_str("via"));
        }
    }
    Some(name)
}

static NOT_THEIR_OWN: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\b(?:next|last|first|night|early|late|platform|station|stop|running|on\s+time|delayed|cancelled|canceled|replacement)\b", 0)
});

/// Python: bot/modules/live_times.py:127 `_their_own_time(clean, memory)`
pub fn _their_own_time(clean: &str, memory: &Memory) -> bool {
    if NOT_THEIR_OWN.search(clean).is_some() {
        return false;
    }
    let Some(found) = VEHICLE.search(clean) else { return false };
    let word = found.as_str().to_lowercase();
    let stem = VEHICLE_STEM.get(&word).map(|v| v.str().to_string()).unwrap_or(word);
    let re = dyn_re(&format!(r"\b{}s?\b", crate::py::re::escape(&stem)), 0);
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(3)..].iter().rev() {
        let said = turn.user.to_lowercase();
        if re.search(&said).is_some() && CLOCK.search(&said).is_some() {
            return true;
        }
    }
    false
}

/// Python: bot/modules/live_times.py:147 `_clean(text)`
pub fn _clean(text_: &str) -> String {
    text::squash(&text_.to_lowercase())
}

static FLIGHT_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:flight|flights|plane)\b", 0));
static AIRPORT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bairport\b", 0));
static NEXT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bnext\b", 0));
static WHAT_TIME: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bwhat\s+time\b", 0));
static REPLACEMENT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:replacement|engineering\s+works?)\b", 0));
static FLIGHT_OR_AIRPORT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:flight|flights|plane|airport)\b", 0));
static PLURAL_SERVICE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:trains|trams|coaches)\b", 0));

/// Python: bot/modules/live_times.py:151 `respond(text, memory)` - a runtime hook.
pub fn respond(text_: &str, memory: &mut Memory) -> Option<String> {
    reply(text_, memory)
}

fn reply(text_: &str, memory: &Memory) -> Option<String> {
    let clean = _clean(text_);
    if clean.is_empty() || VEHICLE.search(&clean).is_none() || TIME.search(&clean).is_none() {
        return None;
    }
    if MONEY.search(&clean).is_some()
        || NEWS.search(&clean).is_some()
        || PLACE_OPENING.search(&clean).is_some()
        || ADVICE.search(&clean).is_some()
        || _their_own_time(&clean, memory)
    {
        return None;
    }
    if DELAYED.search(&clean).is_some() && BOARD_ASK.search(&clean).is_none() {
        return None;
    }
    let flight = FLIGHT_WORD.search(&clean).is_some() || (AIRPORT.search(&clean).is_some() && VEHICLE.search(&clean).is_none());
    if flight && FLIGHT_LIVE.search(&clean).is_none() {
        return None;
    }
    if ASK.search(&clean).is_none() && !text::strip(text_).ends_with('?') {
        return None;
    }
    if NEXT.search(&clean).is_none()
        && STATUS.search(&clean).is_none()
        && PLATFORM.search(&clean).is_none()
        && WHAT_TIME.search(&clean).is_none()
        && WHEN_ASK.search(&clean).is_none()
    {
        return None;
    }
    let service = _service_named(&clean);
    let lead = match service.as_deref().filter(|s| !s.is_empty()) {
        Some(s) => format!("For the {s}: "),
        None => String::new(),
    };
    let vehicle = SERVICE.search(&clean).map(|m| m.name_str("vehicle").to_lowercase()).unwrap_or_default();
    if REPLACEMENT.search(&clean).is_some() {
        return Some(
            "A replacement is the operator's own announcement: their app or site says where it leaves from and when, and the station's board carries it on the day."
                .into(),
        );
    }
    if vehicle == "flight" || (vehicle.is_empty() && FLIGHT_OR_AIRPORT.search(&clean).is_some()) {
        return Some(lead + &_the_wording(memory, &FLIGHT));
    }
    if vehicle == "ferry" {
        return Some(lead + &_the_wording(memory, &FERRY));
    }
    if ["train", "tube", "tram", "coach"].contains(&vehicle.as_str()) || PLURAL_SERVICE.search(&clean).is_some() {
        let mut named = VEHICLE_STEM.get(&vehicle).map(|v| v.str().to_string()).unwrap_or(vehicle.clone());
        if named.is_empty() {
            let mut keys: Vec<&str> = VEHICLE_STEM.pairs().iter().filter_map(|(k, _)| k.as_str()).collect();
            keys.sort();
            let found_plural = dyn_re(&format!(r"\b(?:{})\b", keys.join("|")), 0);
            named = match found_plural.search(&clean) {
                Some(f) => VEHICLE_STEM.get(f.as_str()).map(|v| v.str().to_string()).unwrap_or_else(|| f.as_str().to_string()),
                None => "train".into(),
            };
        }
        let place = THE_BOARD_THAT_KEEPS_THEM.get(&named).map(|v| v.str().to_string()).unwrap_or_else(|| "the board at the station".into());
        return Some(lead + &py_format(&_the_wording(memory, &RAIL), &[("vehicle", &named), ("where", &place)]));
    }
    Some(lead + &_the_wording(memory, &BUS))
}

/// Python: bot/modules/live_times.py:250 `_the_wording(memory, wordings)`
pub fn _the_wording(memory: &Memory, wordings: &Value) -> String {
    let items = wordings.items();
    items[memory.turns.len() % items.len()].str().to_string()
}

/// Python: bot/modules/live_times.py:258 `TAKES_AS_IT_COMES(text, memory=None)`
pub fn TAKES_AS_IT_COMES(text_: &str, memory: &Memory) -> bool {
    reply(text_, memory).is_some()
}
