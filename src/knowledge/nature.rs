//! Port of PyBot's `bot/knowledge/nature.py`: the moon, tides, daylight, autumn leaves, and
//! whether a tomato is a fruit.

use crate::data::{self, FnRef};
use crate::knowledge::d1_dup_g7 as dup;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

fn key(name: &str) -> String {
    format!("bot.knowledge.nature.{name}")
}
static SYNODIC_MONTH: LazyLock<f64> = LazyLock::new(|| data::get(&key("SYNODIC_MONTH")).as_f64().unwrap_or(29.530588853));
static BOTANICAL_FRUITS: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map(&key("BOTANICAL_FRUITS")));
static PLANT_PARTS: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map(&key("PLANT_PARTS")));
static AGGREGATE_BERRIES: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&key("AGGREGATE_BERRIES")));
static KITCHEN_FRUITS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&key("KITCHEN_FRUITS")));
static _SETTING_CROPS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&key("_SETTING_CROPS")));
static _PLANT_TROUBLE: LazyLock<&'static Re> = LazyLock::new(|| data::re(&key("_PLANT_TROUBLE")));
static THE_HARVEST_ASK: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&key("THE_HARVEST_ASK")));
static _PLANTS_IN_HAND: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&key("_PLANTS_IN_HAND")));
static _DAYLIGHT_SHORT_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&key("_DAYLIGHT_SHORT_WORDS")));
static _DAYLIGHT_LONG_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&key("_DAYLIGHT_LONG_WORDS")));
static _DAYLIGHT_WINTER: LazyLock<String> = LazyLock::new(|| data::string(&key("_DAYLIGHT_WINTER")));
static _DAYLIGHT_SUMMER: LazyLock<String> = LazyLock::new(|| data::string(&key("_DAYLIGHT_SUMMER")));
static _DAYLIGHT_SCOTLAND: LazyLock<String> = LazyLock::new(|| data::string(&key("_DAYLIGHT_SCOTLAND")));
/// `_DAYLIGHT_BY_MONTH`: (month, hours, minutes).
static _DAYLIGHT_BY_MONTH: LazyLock<Vec<(String, i64, i64)>> = LazyLock::new(|| {
    data::get(&key("_DAYLIGHT_BY_MONTH"))
        .items()
        .iter()
        .map(|r| {
            let r = r.items();
            (r[0].str().to_string(), r[1].as_int().unwrap_or(0), r[2].as_int().unwrap_or(0))
        })
        .collect()
});
/// `PRODUCE`, each name with its plural pattern (`\bstem(?:y|ies)\b`, ...).
static PRODUCE: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    data::strs(&key("PRODUCE"))
        .into_iter()
        .map(|name| {
            let (stem, plural) = if let Some(s) = name.strip_suffix('y') {
                (s.to_string(), "(?:y|ies)")
            } else if let Some(s) = name.strip_suffix('o') {
                (s.to_string(), "(?:o|os|oes)")
            } else if ["ch", "sh", "s", "x"].iter().any(|e| name.ends_with(e)) {
                (name.clone(), "(?:es)?")
            } else {
                (name.clone(), "s?")
            };
            let re = Re::new(&format!(r"\b{}{}\b", sh::escape(&stem), plural), 0);
            (name, re)
        })
        .collect()
});
static WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static NOT_PLAIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' ]+", 0));

/// When the next full moon is, from the clock (to the second).
pub fn _next_full_moon(_memory: &Memory, _understood: &Understood) -> String {
    let new_moon = dup::days_from_civil(2000, 1, 6) * 86400 + 18 * 3600 + 14 * 60;
    let (today, seconds) = dup::now_local();
    let now = today * 86400 + seconds;
    let cycles = (now - new_moon) as f64 / 86400.0 / *SYNODIC_MONTH;
    let full = (cycles - 0.5) as i64 + 1;
    let micros = ((full as f64 + 0.5) * *SYNODIC_MONTH * 86_400_000_000.0).round() as i64;
    let when = new_moon * 1_000_000 + micros;
    let when_day = when.div_euclid(86_400_000_000);
    let days = when_day - today;
    let (_, wm, wd) = dup::civil_from_days(when_day);
    let when_text = format!("{} {} {}", dup::weekday_name(when_day), wd, dup::month_name(wm));
    let lead = match days {
        0 => "Tonight is a full moon".to_string(),
        1 => format!("The next full moon is tomorrow, {when_text}"),
        _ => format!("The next full moon is on {when_text}, in {days} days"),
    };
    lead + ". Full moons come round every 29.5 days, so the one after is about four weeks later; the moon looks biggest when it is low in the sky, which is an illusion rather than the moon being closer."
}

/// False when the line is about a fruiting crop in trouble (flowers dropping, not setting), not
/// what a fruit is.
pub fn _not_the_definition(understood: &Understood) -> bool {
    let line = sh::text_or_clean(understood).to_lowercase();
    let crop = WORD.finditer(&line).iter().any(|m| _SETTING_CROPS.contains(m.as_str()));
    !(crop && _PLANT_TROUBLE.is_match(&line))
}

/// Whether the line asks about a crop fruiting (harvest), not botany.
pub fn _fruiting_of_a_crop(line: &str) -> bool {
    THE_HARVEST_ASK.iter().any(|w| line.contains(w.as_str()))
}

/// Whether the produce the line names is a fruit, a berry or a vegetable, botanically.
pub fn _fruit_or_berry(_memory: &Memory, understood: &Understood) -> Option<String> {
    let line = sh::text_or_clean(understood).to_lowercase();
    if _fruiting_of_a_crop(&line) {
        return None;
    }
    let Some(word) = PRODUCE.iter().find(|(_, re)| re.is_match(&line)).map(|(n, _)| n.as_str()) else {
        return Some("Botanically a fruit is the part that grows from the flower and holds the seeds, so tomatoes, cucumbers and peppers are fruits. A berry is a fruit from one ovary with seeds in the flesh, which makes bananas berries but strawberries not.".into());
    };
    let art = if word.chars().next().is_some_and(|c| "aeiou".contains(c)) { "an" } else { "a" };
    let asks_berry = line.contains("berr");
    let asks_vegetable = line.contains("vegetable") && !asks_berry;
    if let Some(part) = PLANT_PARTS.get(word) {
        if asks_vegetable {
            return Some(format!("Yes - in the kitchen {art} {word} is a vegetable, but botanically it is {part}, not a fruit: the fruit is the part that holds the seeds."));
        }
        return Some(format!("No - {art} {word} is {part}, not a fruit. The fruit is the part that holds the seeds, so {art} {word} is a vegetable in the kitchen and {part} in the ground."));
    }
    if asks_berry {
        if AGGREGATE_BERRIES.contains(word) {
            return Some(format!("No - {art} {word} is not a true berry: its seeds are on the outside and each one is its own tiny fruit, so botanists call it an aggregate. A banana or a tomato really is a berry, which is where the rule surprises people."));
        }
        if word == "cherry" {
            return Some("No - a cherry is a drupe: one seed inside a hard stone, not a berry. A banana or a tomato really is a berry, which is where the rule surprises people.".into());
        }
        return Some(format!("Yes - {art} {word} is a berry, botanically: it grows from one ovary and has its seeds inside the flesh. A strawberry is not, because its seeds sit on the outside."));
    }
    if let Some(kind) = BOTANICAL_FRUITS.get(word) {
        return Some(format!("Botanically {art} {word} is a {kind}. Cooks call it a vegetable because it is savoury and goes in mains, but that is a kitchen word, not a plant one."));
    }
    if KITCHEN_FRUITS.contains(word) {
        return Some(format!("Botanically {art} {word} is a fruit, and in the kitchen too: it grows from the flower and holds its seeds. A vegetable is a different part of the plant - a root, a leaf or a stem."));
    }
    Some(format!("Botanically {art} {word} is a fruit, because it grows from the flower and holds its seeds. A vegetable is the other parts of the plant, such as roots, leaves and stems."))
}

/// What to do with a day out when it rains (more for a wet week).
pub fn _the_wet_day(_memory: &Memory, reading: &Understood) -> String {
    let line = sh::clean_or_text(reading).to_lowercase();
    let mut said = String::from("Rain rearranges a day out rather than cancelling it: keep the outdoor parts loose - a walk, a beach, a garden - and put one indoor thing into each day, a museum, a pool, a long lunch or a castle you can go round under cover. Book only what you can move and decide on the morning, when you can see the sky.");
    if ["all week", "every day", "a lot", "pours", "turns", "weather is bad"].iter().any(|w| line.contains(w)) {
        said.push_str(" If it is set in for the week, pick one proper indoor day rather than a wet walk every day, and keep the evenings for the things that need no weather at all.");
    }
    said
}

/// The person's last few lines (before this one), plain lower-case, padded with spaces.
pub fn _the_older_turns(memory: &Memory, keep: usize) -> String {
    let start = memory.turns.len().saturating_sub(keep);
    let said = memory.turns[start..].iter().map(|t| t.user.as_str()).collect::<Vec<_>>().join(" ");
    format!(" {} ", NOT_PLAIN.sub(" ", &said.to_lowercase()))
}

/// Whether a plant the person grows was the subject of the chat so far.
pub fn _a_plant_they_own_is_the_subject(memory: &Memory) -> bool {
    let said = _the_older_turns(memory, 3);
    _PLANTS_IN_HAND.iter().any(|w| said.contains(&format!(" {w} ")) || said.contains(&format!(" {w}s ")))
}

/// Why leaves change colour in autumn (not when the person's own plant is the subject).
pub fn _the_autumn_leaves(memory: &Memory, _reading: &Understood) -> Option<String> {
    if _a_plant_they_own_is_the_subject(memory) {
        return None;
    }
    Some("Shorter days tell trees to stop making chlorophyll, the green stuff, and as it fades the yellow and orange pigments underneath show through; some trees also make red pigments. The tree then seals off each leaf and lets it drop, so it loses less water over winter.".into())
}

/// The tide card for the line: which way it is going, crossing sand, swimming, or the times.
pub fn _the_tide(_memory: &Memory, reading: &Understood) -> String {
    let line = sh::clean_or_text(reading).to_lowercase();
    let has = |ws: &[&str]| ws.iter().any(|w| line.contains(w));
    if has(&["in or out", "coming in", "going out", "tide doing", "rising", "falling", "dropping", "state of the tide"]) {
        return "Watch it rather than guess: on a flooding tide the water climbs a few inches every ten minutes and the weed floats past in one direction; on the ebb the wet sand shows above the water line and the rocks come out. The nearest port's table tells you the turn of it, and the turn is what the times are.".into();
    }
    if has(&["causeway", "cross the sand", "crossing the sand", "walk across", "mudflat", "quicksand", "tidal road"]) {
        return "Do not cross on a guess, and never on another port's table: the bay's own crossing times are published by the coastguard or the local guides, and they are the only ones that count. Tell somebody where you are going and when you will be back, and never park below the high-water mark - cars are lost that way every year.".into();
    }
    if has(&["swim", "swimming", "paddle", "paddling", "in the water"]) {
        return "Swim near the turn of the tide rather than on a big spring ebb, keep to a lifeguarded beach, and stay off flat sand where the flood gets behind you: the water comes back faster than it left. The nearest port's table gives you the turn, and aim to be out of it well before.".into();
    }
    "A tide time is local, so the nearest port's table is the only one that counts - the harbour, the coastguard, or the tide page for that port - and high water comes about 50 minutes later each day, so yesterday's table is no use. The sea slacks for a few minutes either side of the turn, and on flat sand the flood comes in faster than it looks: be off the beach an hour before the table says.".into()
}

/// The month the text names, with its daylight (May only as "in/of/during May" or at the end).
pub fn _month_daylight(said: &str) -> Option<(String, i64, i64)> {
    let said = format!(" {} ", text::strip(said));
    for (name, hours, minutes) in _DAYLIGHT_BY_MONTH.iter() {
        if name == "may" {
            if said.contains(" in may ") || said.contains(" of may ") || said.contains(" during may ") || text::rstrip(&said).ends_with(" may") {
                return Some((name.clone(), *hours, *minutes));
            }
            continue;
        }
        if said.contains(&format!(" {name} ")) {
            return Some((name.clone(), *hours, *minutes));
        }
    }
    None
}

/// The daylight card: winter, summer (Scotland's longer), or the month named.
pub fn _daylight(_memory: &Memory, reading: &Understood) -> String {
    let said = text::squash(sh::clean_or_text(reading)).to_lowercase();
    if sh::any_in(&said, &_DAYLIGHT_SHORT_WORDS) {
        return _DAYLIGHT_WINTER.clone();
    }
    if sh::any_in(&said, &_DAYLIGHT_LONG_WORDS) {
        if said.contains("scotland") || said.contains("scottish") || said.contains("shetland") {
            return format!("{}{}", *_DAYLIGHT_SUMMER, *_DAYLIGHT_SCOTLAND);
        }
        return _DAYLIGHT_SUMMER.clone();
    }
    if !said.contains(" december ") {
        if let Some((name, hours, minutes)) = _month_daylight(&said) {
            return format!("In the UK, about the middle of {name} gives roughly {hours} hours {minutes} minutes of daylight, and it is still changing fastest around the equinoxes - a week either side of one moves the sunset by several minutes.");
        }
    }
    _DAYLIGHT_WINTER.clone()
}

fn g_not_def(_f: &FnRef, u: &Understood) -> bool {
    _not_the_definition(u)
}
fn r_moon(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_next_full_moon(m, u))
}
fn r_fruit(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _fruit_or_berry(m, u)
}
fn r_wet(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_the_wet_day(m, u))
}
fn r_leaves(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _the_autumn_leaves(m, u)
}
fn r_tide(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_the_tide(m, u))
}
fn r_daylight(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_daylight(m, u))
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.nature:_not_the_definition:128", g_not_def)];
pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.nature:_next_full_moon:42", r_moon),
    ("bot.knowledge.nature:_fruit_or_berry:153", r_fruit),
    ("bot.knowledge.nature:_the_wet_day:209", r_wet),
    ("bot.knowledge.nature:_the_autumn_leaves:258", r_leaves),
    ("bot.knowledge.nature:_the_tide:269", r_tide),
    ("bot.knowledge.nature:_daylight:358", r_daylight),
];
