//! Port of `bot/memory/trips.py`: a trip the person is going on, said plainly, is answered about
//! the trip - what the time of year means for booking it, what changes with kids in the car - and
//! the short line that only adds a detail to a live trip ("with two kids", "for a week", "we're
//! driving") is answered about the trip too. Every other line is left alone.
//!
//! Module data (patterns, word lists, reply pools, the knowledge base's travel tables) comes from
//! the exported bundle. A trip is PyBot's dict {place, month, when, who, how_long, by, ...}: here
//! the `Trip` struct, and a `Value` dict where the contract hands it to another file.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::facts::py_str;
use crate::py::re::{self as pyre, Re};
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

const I: u32 = pyre::I;

fn g(name: &str) -> &'static Value {
    data::get(&format!("bot.memory.trips.{name}"))
}
fn re(name: &str) -> &'static Re {
    g(name).as_re().unwrap_or_else(|| panic!("bot.memory.trips.{name} is not a regex"))
}

macro_rules! patterns {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> = LazyLock::new(|| re(stringify!($name)));)*
    };
}
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> = LazyLock::new(|| g(stringify!($name)).strs());)*
    };
}
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> = LazyLock::new(|| g(stringify!($name)).str_set());)*
    };
}
patterns!(
    GOING, GOING_BARE, TO_PLACE, IN_PLACE, BOOKED_PLACE, NOT_A_TRIP, A_PERSON_IS_NOT_A_PLACE,
    EVERYDAY_TRIP, ON_A, WITH_WHO, FOR_HOW_LONG, JUST_HOW_LONG, DURATION, PACKING, PACKING_SAID,
    ONE_THING_NEEDED, PACKING_FOR, A_STAY_LENGTH, A_MEAL, A_NEW_BABY, THE_FLIGHT, HOW_LONG_ASK,
    THE_MODE, THE_DRIVE, THE_JOURNEY_ASK, THE_LENGTH_ASK, JOURNEY_MODE, BY, OUTING, JUST_A_DETAIL,
    WORD, BOOKED, BOOKED_DONE, BOOKED_IN_PLACE, BOOKED_IN_WHEN, A_POSSESSION_NOT_A_PLACE,
    A_CARING_PLACE_HEAD, A_WHEN_PHRASE, BARE_DAY, BARE_WHO, SOMEWHERE_TO_EAT, SOMEWHERE_TO_STAY,
    WEEKEND_AWAY, A_BABY, JUST_A_MONTH,
);
pools!(
    PACKING_NOTES, PACKING_NO_MONTH, PACKING_NO_PLACE, PACKING_NO_PLACE_NO_MONTH, VISIT_PACKING,
    VISIT_PACKING_FOR_A_MEAL, VISIT_PACKING_FOR_A_BABY, VISIT_HOW_LONG, VISIT_A_WEEK,
    VISIT_A_FEW_DAYS, VISIT_A_WEEKEND, THE_LENGTH_NOT_SAID, THE_LENGTH_NOT_SAID_NAMED,
    THE_NIGHTS_FIRST, PEAK, BUSY, QUIET, PEAK_NOTES, PEAK_BOOKED_NOTES, BUSY_NOTES, QUIET_NOTES,
    SEASON_NOTES, WINTER_SUN, KIDS_NOTES, KIDS_NUMBERED, WEEK_NOTES, WEEK_NOTES_LAND, DAY_NOTES,
    WEEKEND_NOTES, LONG_NOTES, ASK_WHEN, ASK_WHEN_NO_QUESTION, SEA_NOTES, LAND_NOTES, INDOOR_NOTES,
    SEA, TICKETED, OPEN_AIR, ASK_WHAT, ASK_VISIT, WARM, EAT_WITHOUT_A_CARD, STAY_WITHOUT_A_CARD,
    EAT_WITHOUT_A_TOWN, EAT_ADVICE, STAY_ADVICE, STAY_WITHOUT_A_TOWN,
);
word_sets!(
    NOT_A_PLACE, _NOT_A_THING_OF_THEIRS, TIME_WORD, NO_THING_NAMED, OUT_OF_SEASON,
    AN_EVENT_NOT_A_PLACE, SEASON, SEA_PLACES, PLURAL_RELATIONS, BOOKED_THING, STAY_THING,
    STOP_AFTER, SMALL, _PLURAL_PEOPLE, MONTHS, DAYS,
);
static PACKING_WITH_KIDS: LazyLock<String> = LazyLock::new(|| g("PACKING_WITH_KIDS").str().to_string());

static RELATIONS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.RELATIONS"));
static IN_A_CARING_PLACE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.not_a_trip.IN_A_CARING_PLACE"));

/// A table of the knowledge base's travel data, keyed by its (lowercase) names.
fn travel(name: &str) -> HashMap<String, String> {
    data::try_get(&format!("bot.knowledge.travel.{name}"))
        .map(|table| table.pairs().iter().map(|(k, v)| (py_str(k), py_str(v))).collect())
        .unwrap_or_default()
}
static CITIES: LazyLock<HashMap<String, String>> = LazyLock::new(|| travel("CITIES"));
static BEST_TIME: LazyLock<HashMap<String, String>> = LazyLock::new(|| travel("BEST_TIME"));
static CITY_BEST_TIME: LazyLock<HashMap<String, String>> = LazyLock::new(|| travel("CITY_BEST_TIME"));
static CITY_COUNTRY: LazyLock<HashMap<String, String>> = LazyLock::new(|| travel("CITY_COUNTRY"));
static CITY_AREA: LazyLock<HashMap<String, String>> = LazyLock::new(|| travel("CITY_AREA"));
static MONTH_PICKS: LazyLock<HashMap<String, String>> = LazyLock::new(|| travel("_MONTH_PICKS"));

// The patterns PyBot writes inline.
static WENT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bwent\b|\bhave been\b|\bwas in\b|\blast year\b", 0));
static A_QUESTION_WORD: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(?:when|where|how|what|why|should|shall|can|could|would|will|worth)\b", 0));
static A_STATEMENT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:we're|were|we are|i'm|im|i am)\b", 0));
static BOOK_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bbook", I));
static AN_ARTICLE: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:the|a|an)\s+", 0));
static SENTENCES: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s+", 0));
static NIGHT_OR_DAY: LazyLock<Re> = LazyLock::new(|| Re::new(r"night|\bday", 0));
static WEEK_OR_LONGER: LazyLock<Re> = LazyLock::new(|| Re::new(r"week|fortnight|month", 0));
static CHILDREN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bkids?\b|\bchild|\bchildren\b|\bbaby\b|\btoddler", 0));
static A_FEW: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:two|three|four|five|six|\d+)\b", 0));
static MORE_THAN_ONE: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|fortnight|\d", 0));
static A_WEEK_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bweek", 0));
static A_MONTH_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bmonth", 0));
static A_DAY_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bday", 0));
static A_LENGTH_FRAME: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"^(?:its|it's|thats|that's|were|we're|we are|there|were there|its for|its about|we're there)\s+", 0)
});

/// A trip, as PyBot's dict. `visit` is Some only for a visit (the dict then carries visit, meal
/// and baby); `plan` is Some once `_the_trip_so_far` has said which plan it came from.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Trip {
    pub place: String,
    pub month: String,
    pub when: String,
    pub who: String,
    pub how_long: String,
    pub by: String,
    pub visit: Option<(bool, bool, bool)>,
    pub plan: Option<String>,
}

impl Trip {
    fn is_a_visit(&self) -> bool {
        self.visit.is_some_and(|v| v.0)
    }
    fn meal(&self) -> bool {
        self.visit.is_some_and(|v| v.1)
    }
    fn baby(&self) -> bool {
        self.visit.is_some_and(|v| v.2)
    }
    /// The dict PyBot passes around.
    pub fn to_value(&self) -> Value {
        let s = |v: &str| Value::from(v);
        let mut pairs = vec![
            (s("place"), s(&self.place)),
            (s("month"), s(&self.month)),
            (s("when"), s(&self.when)),
            (s("who"), s(&self.who)),
            (s("how_long"), s(&self.how_long)),
            (s("by"), s(&self.by)),
        ];
        if let Some((visit, meal, baby)) = self.visit {
            pairs.push((s("visit"), Value::Bool(visit)));
            pairs.push((s("meal"), Value::Bool(meal)));
            pairs.push((s("baby"), Value::Bool(baby)));
        }
        if let Some(plan) = &self.plan {
            pairs.push((s("plan"), s(plan)));
        }
        Value::Dict(Arc::new(pairs))
    }
}

/// `bot.memory.user(memory)`: the user's facts, created on first use (a copy of the dict).
fn user(memory: &mut Memory) -> Value {
    if !memory.get("user").is_some_and(Value::is_dict) {
        memory.set("user", Value::dict());
    }
    memory.get("user").cloned().unwrap_or(Value::None)
}

/// `known.get(key) or []` as its items.
fn list_of(known: &Value, key: &str) -> Vec<Value> {
    known.get(key).filter(|v| v.truthy()).map(|v| v.items().to_vec()).unwrap_or_default()
}

/// `s[:1].upper() + s[1:]`.
fn up1(s: &str) -> String {
    pt::upper_first(s)
}

/// `str.format` for the `{name}` fields these pools use.
fn fmt(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 32);
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find('}') {
            Some(close) => {
                let name = &after[..close];
                match fields.iter().find(|(k, _)| *k == name) {
                    Some((_, v)) => out.push_str(v),
                    None => panic!("KeyError: '{name}'"),
                }
                rest = &after[close + 1..];
            }
            None => {
                out.push_str(&rest[open..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

/// Python: bot/memory/trips.py:189 `_one_thing_they_need(low)`
///
/// The thing a "will i need ..." asks about, or "" when it names nothing in particular.
pub fn _one_thing_they_need(low: &str) -> String {
    let Some(found) = ONE_THING_NEEDED.match_(low) else {
        return String::new();
    };
    let thing = pt::lower(found.name_str("thing"));
    if NO_THING_NAMED.contains(&thing) {
        return String::new();
    }
    thing
}

/// Python: bot/memory/trips.py:541 `_start(said)`
///
/// A reply that begins with a place whose "the" was lowered mid-sentence still opens a line.
pub fn _start(said: &str) -> String {
    up1(said)
}

/// Python: bot/memory/trips.py:547 `_low(text)`
pub fn _low(text: &str) -> String {
    pt::squash(&pt::lower(text))
}

/// Python: bot/memory/trips.py:590 `_a_relation_not_a_place(place)`
///
/// True when the place the line names is somebody of theirs, not somewhere.
pub fn _a_relation_not_a_place(place: &str) -> bool {
    let Some(found) = A_POSSESSION_NOT_A_PLACE.match_(pt::strip(place)) else {
        return false;
    };
    let word = pt::lower(found.str(1));
    RELATIONS.contains(&word) || RELATIONS.contains(pt::rstrip_chars(&word, "s")) || PLURAL_RELATIONS.contains(&word)
}

/// Python: bot/memory/trips.py:612 `_a_person_visited(line)`
///
/// The person a line is going to see, with the words that belong to them, or "".
pub fn _a_person_visited(line: &str) -> String {
    if IN_A_CARING_PLACE.is_match(line) {
        return String::new();
    }
    let who = crate::memory::not_a_trip::the_visit(line);
    if !who.is_empty() {
        return who;
    }
    let phrase = _places(line, true);
    if !phrase.is_empty() && _a_relation_not_a_place(&phrase) { phrase } else { String::new() }
}

/// Python: bot/memory/trips.py:631 `_places(line, a_visit=False)`
///
/// The place a line says it is going to, in the person's own casing, or "".
pub fn _places(line: &str, a_visit: bool) -> String {
    if let Some(booked) = BOOKED.search(line) {
        if _the_thing_booked(&_low(&line[booked.end()..])) {
            // What people book is often a thing rather than somewhere to go.
            return String::new();
        }
    }
    for pattern in [&*TO_PLACE, &*BOOKED_IN_PLACE, &*BOOKED_IN_WHEN, &*BOOKED_PLACE, &*IN_PLACE] {
        let Some(found) = pattern.search(line) else {
            continue;
        };
        let words = pt::split(found.str(1));
        // Keep the place short: a word that says when, who or how long starts the rest.
        let mut kept: Vec<&str> = Vec::new();
        for word in &words {
            let bare = pt::lower(pt::strip_chars(word, ",.;:!?"));
            let a_when = MONTHS.contains(&bare) || DAYS.contains(&bare) || SEASON.contains(&bare) || STOP_AFTER.contains(&bare);
            if kept.is_empty() && matches!(bare.as_str(), "the" | "a" | "an") {
                kept.push(word);
                continue;
            }
            if kept.is_empty() && (NOT_A_PLACE.contains(&bare) || a_when) {
                break;
            }
            if !kept.is_empty() && (a_when || matches!(bare.as_str(), "with" | "and" | "for" | "next" | "this")) {
                break;
            }
            kept.push(word);
        }
        let place = pt::strip_chars(&kept.join(" "), " ,.;:!?").to_string();
        let low = pt::lower(&place);
        let head: Vec<&str> = pt::split_ref(&low).into_iter().filter(|w| !matches!(*w, "the" | "a" | "an")).collect();
        if head.first().is_some_and(|first| TIME_WORD.contains(*first)) || (!head.is_empty() && DURATION.match_(&low).is_some()) {
            continue; // the words say when
        }
        if NOT_A_TRIP.match_(&low).is_some() {
            continue;
        }
        if crate::memory::not_a_trip::not_a_destination(&words, &place) {
            continue;
        }
        if place.is_empty()
            || NOT_A_PLACE.contains(&low)
            || MONTHS.contains(&low)
            || DAYS.contains(&low)
            || pt::split_ref(&low).iter().any(|w| SEASON.contains(*w))
        {
            continue;
        }
        if A_CARING_PLACE_HEAD.match_(&low).is_some() {
            continue;
        }
        let bare_place = AN_ARTICLE.sub("", &low);
        if NOT_A_PLACE.contains(&bare_place) || MONTHS.contains(&bare_place) || DAYS.contains(&bare_place) {
            continue;
        }
        // The person themselves is not the destination.
        if !a_visit && _a_relation_not_a_place(&low) {
            continue;
        }
        if matches!(low.as_str(), "the" | "a" | "an") || pt::len(&low) < 3 {
            continue;
        }
        if BOOK_SAID.is_match(line) && (_the_thing_booked(&low) || _a_thing_booked_for_when(&low)) {
            continue;
        }
        return place;
    }
    String::new()
}

/// The words of a phrase without its articles and possessives.
fn without_determiners(low: &str) -> Vec<&str> {
    pt::split_ref(low).into_iter().filter(|w| !matches!(*w, "a" | "an" | "the" | "my" | "our" | "his" | "her")).collect()
}

/// Python: bot/memory/trips.py:717 `_the_thing_booked(low)`
///
/// Whether the words name the thing that was booked rather than somewhere to go.
pub fn _the_thing_booked(low: &str) -> bool {
    without_determiners(low).first().is_some_and(|first| BOOKED_THING.contains(*first))
}

/// Python: bot/memory/trips.py:732 `_a_thing_booked_for_when(low)`
///
/// Whether the words name the thing booked, where a date and not a place follows it.
pub fn _a_thing_booked_for_when(low: &str) -> bool {
    without_determiners(low).first().is_some_and(|first| STAY_THING.contains(*first))
}

/// Python: bot/memory/trips.py:743 `_month(line)`
pub fn _month(line: &str) -> String {
    WORD.findall_str(&_low(line)).into_iter().find(|word| MONTHS.contains(word)).unwrap_or_default()
}

/// Python: bot/memory/trips.py:764 `_when_phrase(line)`
///
/// The date the line gives without naming a month ("next month"), or "".
pub fn _when_phrase(line: &str) -> String {
    if let Some(found) = A_WHEN_PHRASE.search(line) {
        return pt::squash(&pt::lower(found.as_str()));
    }
    BARE_DAY.search(line).map(|day| pt::lower(day.str(1))).unwrap_or_default()
}

/// Python: bot/memory/trips.py:789 `_who(line)`
pub fn _who(line: &str) -> String {
    if let Some(found) = WITH_WHO.search(line) {
        return pt::strip_chars(found.str(1), " ,.;:!?").to_string();
    }
    BARE_WHO.match_(line).map(|bare| pt::strip_chars(bare.str(1), " ,.;:!?").to_string()).unwrap_or_default()
}

/// Python: bot/memory/trips.py:797 `_how_long(line)`
pub fn _how_long(line: &str) -> String {
    if let Some(found) = FOR_HOW_LONG.search(line) {
        return pt::strip_chars(found.str(1), " ,.;:!?").to_string();
    }
    match JUST_HOW_LONG.match_(pt::strip_chars(line, " .,;:!?")) {
        Some(bare) => {
            let said = bare.get(1).filter(|s| !s.is_empty()).unwrap_or_else(|| bare.str(2));
            pt::strip(said).to_string()
        }
        None => String::new(),
    }
}

/// Python: bot/memory/trips.py:807 `_how_getting_there(line)`
pub fn _how_getting_there(line: &str) -> String {
    BY.search(line).map(|found| pt::lower(found.str(1))).unwrap_or_default()
}

/// Python: bot/memory/trips.py:812 `_by_the_sea(place)`
///
/// Whether a week there is a week of beach days: a coast, or a place famous for one.
pub fn _by_the_sea(place: &str) -> bool {
    pt::split(&_low(place).replace('-', " "))
        .iter()
        .any(|w| SEA_PLACES.contains(w) || matches!(w.as_str(), "beach" | "beaches" | "coast" | "seaside" | "sea" | "shore"))
}

/// `_low(place).strip()` without a leading "the ".
fn the_key(place: &str) -> String {
    let key = pt::strip(&_low(place)).to_string();
    key.strip_prefix("the ").map(str::to_string).unwrap_or(key)
}

/// Python: bot/memory/trips.py:822 `_a_carded_place(place)`
///
/// Whether the knowledge base has a card of its own for this place.
pub fn _a_carded_place(place: &str) -> bool {
    let key = the_key(place);
    [&*CITIES, &*BEST_TIME].iter().any(|table| table.keys().any(|other| pt::lower(other) == key))
}

/// Python: bot/memory/trips.py:846 `a_moved_destination(line, memory)`
///
/// The trip the conversation is on, with its place put right, or None.
pub fn a_moved_destination(line: &str, memory: &mut Memory) -> Option<Trip> {
    if line.is_empty() || line.contains('?') {
        return None;
    }
    let place = crate::memory::facts::a_moved_place(line);
    if place.is_empty() {
        return None;
    }
    if _the_last_line_named_an_animal(memory) {
        // "my dog is called rex" then "i changed it to max" is the dog renamed.
        return None;
    }
    let Some(live) = the_trip_so_far(memory) else {
        // A trip rebooked before it was ever said as a trip: the place is all the line gives.
        return Some(Trip { place, ..Trip::default() });
    };
    if _low(&_cap(&live.place)) == _low(&place) {
        return None; // already the place they said, so nothing has moved
    }
    let old = live.place.clone();
    _keep_the_new_place(memory, &place, &old, line);
    Some(Trip { place, ..live })
}

/// Python: bot/memory/trips.py:875 `_the_last_line_named_an_animal(memory)`
///
/// True when the line before this one gave an animal a name or asked about one.
pub fn _the_last_line_named_an_animal(memory: &Memory) -> bool {
    let Some(said) = memory.turns.iter().rev().map(|t| &t.user).find(|u| !pt::strip(u).is_empty()) else {
        return false;
    };
    let found = crate::memory::facts::extract(said, false);
    found.get("pets").is_some_and(Value::truthy) || found.get("pets_gone").is_some_and(Value::truthy)
}

/// Python: bot/memory/trips.py:894 `_keep_the_new_place(memory, place, old='', said='')`
///
/// The store keeps the place the trip is going to now, in place of the one it left.
pub fn _keep_the_new_place(memory: &mut Memory, place: &str, old: &str, said: &str) {
    let mut known = user(memory);
    let old_place = pt::strip(old).to_string();
    let places = _the_place_put_right(known.get("places"), &old_place, place);
    let plans = _the_place_put_right(known.get("plans"), &old_place, place);
    let plan = plans.iter().find(|entry| _low(entry).contains(&_low(place))).cloned().unwrap_or_else(|| place.to_string());
    known.set_key("places", places);
    known.set_key("plans", plans);
    known.set_key("plan", plan);
    memory.set("user", known);
    if !matches!(memory.get("facts_log"), Some(Value::List(_))) {
        memory.set("facts_log", Value::list());
    }
    let turn = memory.turns.len() as i64;
    for key in ["places", "plan"] {
        let entry = Value::Dict(Arc::new(vec![
            (Value::from("key"), Value::from(key)),
            (Value::from("value"), Value::from(place)),
            (Value::from("turn"), Value::Int(turn)),
            (Value::from("text"), Value::from(said)),
        ]));
        memory.get_mut("facts_log").unwrap().push(entry);
    }
}

/// Python: bot/memory/trips.py:921 `_the_place_put_right(entries, old, place)`
///
/// The chat's own list with the destination put right, and every other entry left alone.
pub fn _the_place_put_right(entries: Option<&Value>, old: &str, place: &str) -> Vec<String> {
    let mut kept: Vec<String> = Vec::new();
    let mut put_right = false;
    let entries = match entries {
        Some(Value::List(items)) => items.to_vec(),
        _ => Vec::new(),
    };
    for entry in entries {
        let mut text = pt::strip(&if entry.truthy() { py_str(&entry) } else { String::new() }).to_string();
        if text.is_empty() {
            continue;
        }
        if !old.is_empty() && _low(&text).contains(&_low(old)) {
            text = Re::new(&pyre::escape(old), I).sub_with(&text, 1, |_| place.to_string());
            put_right = true;
        }
        if !kept.contains(&text) {
            kept.push(text);
        }
    }
    if !put_right && !kept.iter().any(|k| k == place) {
        kept.push(place.to_string());
    }
    kept
}

/// Python: bot/memory/trips.py:945 `a_trip(text, memory, cards=True)`
///
/// The trip this line states, or None. A trip is a going to a place that is not one of the
/// everyday ones, in a line that is not a question and not a trip already taken.
pub fn a_trip(text: &str, _memory: &Memory, cards: bool) -> Option<Trip> {
    let line = pt::strip(text);
    if line.is_empty() || line.ends_with('?') {
        return None;
    }
    if let Some(kind) = crate::memory::ack::reads_as_a_trip(line).filter(|k| !k.is_empty()) {
        if kind != "trip" {
            // An occasion is read and answered by bot/memory/ack.py.
            return None;
        }
    }
    let low = _low(line);
    if EVERYDAY_TRIP.is_match(&low) || ON_A.is_match(&low) {
        return None;
    }
    if WENT.is_match(&low) {
        return None; // a trip taken, not one coming up
    }
    if A_QUESTION_WORD.is_match(&low) && !A_STATEMENT.is_match(&low) {
        return None; // a question about a trip, not a statement of one
    }
    let visit = _a_person_visited(line);
    if !visit.is_empty() {
        return Some(Trip {
            place: visit,
            month: _month(&low),
            when: _when_phrase(&low),
            who: _who(&low),
            how_long: _how_long(&low),
            by: String::new(),
            visit: Some((true, A_MEAL.is_match(&low), A_NEW_BABY.is_match(&low))),
            plan: None,
        });
    }
    if !(GOING.is_match(&low) || GOING_BARE.is_match(&low)) {
        return None;
    }
    let place = _places(line, false);
    if place.is_empty() {
        return None;
    }
    if cards && _a_carded_place(&place) && !(BOOKED_DONE.is_match(&low) && (!_month(&low).is_empty() || !_when_phrase(&low).is_empty())) {
        // The card says more about the place; a dated booking stays with the store.
        return None;
    }
    if crate::memory::facts::extract(line, false).get("name").is_some_and(Value::truthy) {
        // The name is the one the conversation needs first.
        return None;
    }
    Some(Trip {
        place,
        month: _month(&low),
        when: _when_phrase(&low),
        who: _who(&low),
        how_long: _how_long(&low),
        by: _how_getting_there(&low),
        visit: None,
        plan: None,
    })
}

/// Python: bot/memory/trips.py:999 `the_trip_they_booked(text)`
///
/// The [place, when] a line says it has booked, or ["", ""].
pub fn the_trip_they_booked(text: &str) -> Vec<String> {
    let nothing = vec![String::new(), String::new()];
    if !BOOKED.is_match(text) {
        return nothing;
    }
    let place = _places(text, false);
    if place.is_empty() {
        return nothing;
    }
    let month = _month(text);
    let when = if month.is_empty() { _when_phrase(text) } else { month };
    vec![place, when]
}

/// `_the_trip_so_far` as a `Trip`.
fn the_trip_so_far(memory: &mut Memory) -> Option<Trip> {
    let known = user(memory);
    let mut plans: Vec<String> =
        list_of(&known, "plans").iter().filter_map(|p| p.as_str().filter(|p| !pt::strip(p).is_empty()).map(str::to_string)).collect();
    if let Some(Value::Str(latest)) = known.get("plan") {
        if !pt::strip(latest).is_empty() {
            plans.push(latest.to_string());
        }
    }
    let recent = plans[plans.len().saturating_sub(3)..].to_vec();
    for plan in recent.iter().rev() {
        let line = crate::memory::plan_line(memory, plan);
        let said = if line.is_empty() { plan.clone() } else { line };
        if let Some(mut trip) = a_trip(&said, memory, false) {
            trip.plan = Some(plan.clone());
            return Some(trip);
        }
    }
    // The last few lines they typed are read the same way.
    let lines: Vec<String> = memory.turns[memory.turns.len().saturating_sub(4)..].iter().map(|t| t.user.clone()).collect();
    for said in lines.iter().rev() {
        if let Some(mut trip) = a_trip(said, memory, false) {
            trip.plan = Some(String::new());
            return Some(trip);
        }
    }
    None
}

/// Python: bot/memory/trips.py:1016 `_the_trip_so_far(memory)`
///
/// The trip the conversation is on, as PyBot's dict, or None.
pub fn _the_trip_so_far(memory: &mut Memory) -> Value {
    the_trip_so_far(memory).map(|trip| trip.to_value()).unwrap_or(Value::None)
}

/// Python: bot/memory/trips.py:1046 `_asked_last(memory)`
///
/// Whether the bot's last reply ended on a question, so this one must not ask another.
pub fn _asked_last(memory: &Memory) -> bool {
    pt::strip(&crate::memory::last_bot(memory)).ends_with('?')
}

/// Python: bot/memory/trips.py:1051 `_told(text, memory)`
///
/// The replies already given this conversation. (A set in PyBot; kept here in the order they
/// were said, which only matters to `_pick`'s look inside them - see there.)
pub fn _told(_text: &str, memory: &Memory) -> Vec<String> {
    let mut said: Vec<String> = Vec::new();
    for turn in &memory.turns {
        let low = _low(&turn.bot);
        if !said.contains(&low) {
            said.push(low);
        }
    }
    said
}

/// Python: bot/memory/trips.py:1057 `_pick(options, memory, text)`
///
/// One of options, never one the bot has just said. Same line and same turn, same pick.
///
/// PyBot looks for a long option inside `" ".join(said)` where `said` is a set: a phrase that
/// only exists across the boundary of two replies depends on CPython's string-hash order, and
/// the replies are joined here in the order they were said.
pub fn _pick(options: &[String], memory: &Memory, text: &str) -> String {
    let said = _told(text, memory);
    let said_inside = said.join(" ");
    let mut fresh: Vec<&String> =
        options.iter().filter(|o| !said.contains(&_low(o)) && !(pt::len(o) > 15 && said_inside.contains(&_low(o)))).collect();
    if fresh.is_empty() {
        fresh = options.iter().collect();
    }
    let turns = &memory.turns;
    let ords = |s: &str| s.chars().map(|c| c as i128).sum::<i128>();
    let mut seed: i128 = 0;
    for turn in &turns[turns.len().saturating_sub(4)..] {
        seed = seed * 31 + ords(&turn.user);
    }
    seed = seed * 7919 + ords(text) + turns.len() as i128;
    fresh[seed.rem_euclid(fresh.len() as i128) as usize].clone()
}

/// Python: bot/memory/trips.py:1075 `_join(parts)`
///
/// Sentences joined as sentences: each one opens with a capital.
pub fn _join(parts: &[String]) -> String {
    parts.iter().map(|part| pt::strip(part)).filter(|part| !part.is_empty()).map(up1).collect::<Vec<_>>().join(" ")
}

/// Python: bot/memory/trips.py:1089 `_with_them(named, who, memory, line)`
///
/// Somebody coming along who is not a child: the trip said with them, and the useful ask.
pub fn _with_them(named: &str, who: &str, memory: &Memory, line: &str) -> String {
    let ask = _pick(&ASK_WHAT, memory, line);
    format!("{} with {}. {}", _mid(named), pt::lower(&_yours(who)), up1(&ask))
}

/// Python: bot/memory/trips.py:1096 `_yours(who)`
///
/// The people the person is going with, said back as theirs: "with my mum" -> "with your mum".
pub fn _yours(who: &str) -> String {
    pt::split(who)
        .into_iter()
        .map(|word| match pt::lower(&word).as_str() {
            "my" | "our" => "your".to_string(),
            "ours" | "mine" => "yours".to_string(),
            _ => word,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Python: bot/memory/trips.py:1114 `_mid(named)`
///
/// A place as it reads inside a sentence: "The Isle of Wight" -> "the Isle of Wight".
pub fn _mid(named: &str) -> String {
    match named.strip_prefix("The ") {
        Some(rest) => format!("the {rest}"),
        None => named.to_string(),
    }
}

/// Python: bot/memory/trips.py:1120 `_cap(text)`
///
/// The place as it is written: "the isle of wight" -> "The Isle of Wight".
pub fn _cap(text: &str) -> String {
    let text = pt::strip(text);
    pt::split_ref(text)
        .iter()
        .enumerate()
        .map(|(position, word)| {
            if position > 0 && SMALL.contains(&pt::lower(word)) {
                pt::lower(word)
            } else if word.chars().next().is_some_and(char::is_alphabetic) {
                up1(word)
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Python: bot/memory/trips.py:1137 `_seasons_we_know(place)`
///
/// The knowledge base's own note on when to go there, or "".
pub fn _seasons_we_know(place: &str) -> String {
    let key = the_key(place);
    if key.is_empty() {
        return String::new();
    }
    if let Some(note) = CITY_BEST_TIME.get(&key).filter(|n| !n.is_empty()) {
        return note.clone();
    }
    let country = CITY_COUNTRY.get(&key).cloned().unwrap_or_else(|| key.clone());
    BEST_TIME.get(&country).filter(|n| !n.is_empty()).or_else(|| BEST_TIME.get(&key).filter(|n| !n.is_empty())).cloned().unwrap_or_default()
}

/// Python: bot/memory/trips.py:1161 `_their_month(note, month)`
///
/// The sentence of that note which names their month, or "".
pub fn _their_month(note: &str, month: &str) -> String {
    let names_it = Re::new(&format!(r"\b{month}\b"), 0);
    for sentence in SENTENCES.split(note) {
        if names_it.is_match(&pt::lower(&sentence)) {
            return up1(&_mid(pt::strip(&sentence)));
        }
    }
    String::new()
}

/// Python: bot/memory/trips.py:1170 `_in_its_season(place, month)`
///
/// Whether the knowledge base's own month picks name this place as a place to go then.
pub fn _in_its_season(place: &str, month: &str) -> bool {
    let row = _low(MONTH_PICKS.get(month).map(String::as_str).unwrap_or(""));
    if row.is_empty() {
        return false;
    }
    let key = the_key(place);
    if pt::len(&key) < 4 {
        return false;
    }
    Re::new(&format!(r"\b{}\b", pyre::escape(&key)), 0).is_match(&row)
}

/// Python: bot/memory/trips.py:1193 `_a_winter_place(place)`
///
/// Whether this is somewhere the winter is the season, not the off-season.
pub fn _a_winter_place(place: &str) -> bool {
    let key = the_key(place);
    WINTER_SUN.iter().any(|word| Re::new(&format!(r"\b{}\b", pyre::escape(word)), 0).is_match(&key))
}

/// Each line of a pool with `{month}` filled in.
fn with_the_month(pool: &[String], month: &str) -> Vec<String> {
    pool.iter().map(|n| fmt(n, &[("month", &_cap(month))])).collect()
}

/// Python: bot/memory/trips.py:1201 `_when_note(month, memory, text, booked=False, place='')`
///
/// The one useful sentence a friend adds about a trip in that month: the season itself.
pub fn _when_note(month: &str, memory: &Memory, text: &str, booked: bool, place: &str) -> String {
    let known = _seasons_we_know(place);
    if !known.is_empty() {
        return _their_month(&known, month);
    }
    let quiet = QUIET.iter().any(|m| m == month);
    if quiet && (_a_winter_place(place) || _in_its_season(place, month)) {
        return _pick(&with_the_month(&SEASON_NOTES, month), memory, text);
    }
    let notes: &[String] = if PEAK.iter().any(|m| m == month) {
        if booked { &PEAK_BOOKED_NOTES } else { &PEAK_NOTES }
    } else if BUSY.iter().any(|m| m == month) {
        &BUSY_NOTES
    } else if quiet {
        &QUIET_NOTES
    } else {
        return String::new();
    };
    _pick(&with_the_month(notes, month), memory, text)
}

/// Python: bot/memory/trips.py:1219 `_the_visit_length(how_long, memory, text)`
///
/// The sentence a visit gets for how long it lasts, or "" when the line gave no length.
pub fn _the_visit_length(how_long: &str, memory: &Memory, text: &str) -> String {
    if how_long.is_empty() {
        return String::new();
    }
    let low = _low(how_long);
    if low.contains("weekend") {
        return _pick(&VISIT_A_WEEKEND, memory, text);
    }
    if NIGHT_OR_DAY.is_match(&low) {
        return _pick(&VISIT_A_FEW_DAYS, memory, text);
    }
    if WEEK_OR_LONGER.is_match(&low) {
        return _pick(&VISIT_A_WEEK, memory, text);
    }
    String::new()
}

/// Python: bot/memory/trips.py:1233 `_who_note(who, how_long, memory, text, place=None)`
///
/// The sentence about who is going and for how long, if the line said either.
pub fn _who_note(who: &str, how_long: &str, memory: &Memory, text: &str, place: Option<&str>) -> String {
    let mut notes: Vec<String> = Vec::new();
    if !who.is_empty() && CHILDREN.is_match(who) {
        if A_FEW.is_match(who) {
            // The number they gave is the number said back.
            notes.extend(KIDS_NUMBERED.iter().map(|n| fmt(n, &[("who", who)])));
        } else {
            notes.extend(KIDS_NOTES.iter().cloned());
        }
    }
    if !how_long.is_empty() && how_long.contains("weekend") {
        notes.extend(WEEKEND_NOTES.iter().cloned());
    } else if !how_long.is_empty() && NIGHT_OR_DAY.is_match(how_long) {
        notes.extend(DAY_NOTES.iter().cloned());
    } else if !how_long.is_empty() && WEEK_OR_LONGER.is_match(how_long) {
        if MORE_THAN_ONE.is_match(how_long) {
            notes.extend(LONG_NOTES.iter().cloned());
        } else if place.map_or(true, _by_the_sea) {
            notes.extend(WEEK_NOTES.iter().cloned());
        } else {
            notes.extend(WEEK_NOTES_LAND.iter().cloned());
        }
    }
    if notes.is_empty() {
        return String::new();
    }
    _pick(&notes, memory, text)
}

/// Python: bot/memory/trips.py:1260 `_a_note(month, who, how_long, memory, text, booked=False, place='')`
///
/// Both notes, when the month and the people are both worth a sentence.
pub fn _a_note(month: &str, who: &str, how_long: &str, memory: &Memory, text: &str, booked: bool, place: &str) -> String {
    let parts: Vec<String> = [_when_note(month, memory, text, booked, place), _who_note(who, how_long, memory, text, Some(place))]
        .into_iter()
        .filter(|n| !n.is_empty())
        .collect();
    _join(&parts)
}

/// Python: bot/memory/trips.py:1288 `_a_known_city(place)`
///
/// Whether the knowledge base has a card for this place, so its own answer keeps the turn.
pub fn _a_known_city(place: &str) -> bool {
    let place = pt::strip_chars(&_low(place), " .,;:!?").to_string();
    if place.is_empty() {
        return false;
    }
    CITIES.contains_key(&place) || pt::split_ref(&place).first().is_some_and(|first| CITIES.contains_key(*first))
}

/// Python: bot/memory/trips.py:1347 `_the_areas_of(place)`
///
/// The parts of the town the knowledge base says to stay in, or "" when it says none.
pub fn _the_areas_of(place: &str) -> String {
    CITY_AREA.get(pt::strip_chars(&_low(place), " .,;:!?")).cloned().unwrap_or_default()
}

/// Python: bot/memory/trips.py:1356 `_somewhere_for(line, memory)`
///
/// The reply to "and somewhere to eat" / "somewhere to stay", or None for anything else.
pub fn _somewhere_for(line: &str, memory: &mut Memory) -> Option<String> {
    let eat = SOMEWHERE_TO_EAT.is_match(line);
    let stay = SOMEWHERE_TO_STAY.is_match(line);
    if !(eat || stay) {
        return None;
    }
    let trip = the_trip_so_far(memory).unwrap_or_default();
    let mut place = _cap(&_as_a_visit(&trip.place));
    if place.is_empty() {
        // The place is in the line: understanding leaves it in ref.there for the follow-up.
        if let Some(there) = memory.understood.as_ref().and_then(|u| u.reference.there.clone()).filter(|t| !t.is_empty()) {
            place = _cap(&there);
        }
    }
    if place.is_empty() {
        let known = user(memory);
        if let Some(found) = list_of(&known, "places").iter().rev().find(|p| p.truthy()) {
            place = _cap(&py_str(found));
        }
    }
    if place.is_empty() {
        for turn in memory.turns.iter().rev() {
            let said = _places(&turn.user, false);
            if !said.is_empty() {
                place = _cap(&said);
                break;
            }
        }
    }
    // A town the knowledge base has a card for keeps the turn.
    if !place.is_empty() && _a_known_city(&place) && (eat || !_the_areas_of(&place).is_empty()) {
        return None;
    }
    if !place.is_empty() {
        let pool = if eat { &*EAT_WITHOUT_A_CARD } else { &*STAY_WITHOUT_A_CARD };
        let filled: Vec<String> = pool.iter().map(|l| fmt(l, &[("place", &_mid(&place))])).collect();
        return Some(_pick(&filled, memory, line));
    }
    if _asked_last(memory) {
        return Some(_pick(if eat { &EAT_ADVICE } else { &STAY_ADVICE }, memory, line));
    }
    Some(_pick(if eat { &EAT_WITHOUT_A_TOWN } else { &STAY_WITHOUT_A_TOWN }, memory, line))
}

/// Python: bot/memory/trips.py:1417 `_the_kind_of_trip(how_long)`
///
/// The trip said as a kind rather than a place: "a weekend away", "a week away".
pub fn _the_kind_of_trip(how_long: &str) -> String {
    let said = _low(how_long);
    let kind = if said.contains("weekend") {
        "a weekend away"
    } else if said.contains("fortnight") {
        "a fortnight away"
    } else if A_WEEK_SAID.is_match(&said) {
        "a week away"
    } else if A_MONTH_SAID.is_match(&said) {
        "a month away"
    } else if A_DAY_SAID.is_match(&said) {
        "a few days away"
    } else {
        "a weekend away"
    };
    kind.to_string()
}

/// Python: bot/memory/trips.py:1433 `_the_warm_week_is_out_of_season(month)`
pub fn _the_warm_week_is_out_of_season(month: &str) -> bool {
    OUT_OF_SEASON.contains(pt::strip(&_low(month)))
}

/// Python: bot/memory/trips.py:1438 `_an_event_not_a_place(place)`
///
/// True when the "place" the trip reading found is a thing they are going to, not a town.
pub fn _an_event_not_a_place(place: &str) -> bool {
    let bare = pt::strip(&_low(place)).to_string();
    let bare = ["a ", "an ", "the ", "my ", "our "].iter().find_map(|w| bare.strip_prefix(w)).unwrap_or(&bare).to_string();
    AN_EVENT_NOT_A_PLACE.contains(&bare)
}

/// `a_packing_question_about_the_trip` as a `Trip`.
fn packing_question_about_the_trip(text: &str, memory: &mut Memory) -> Option<Trip> {
    let line = _low(text);
    let found = if line.is_empty() { None } else { PACKING.search(&line) }?;
    if A_BABY.is_match(&line) {
        return None;
    }
    let recent = &memory.turns[memory.turns.len().saturating_sub(6)..];
    if recent.iter().any(|t| A_BABY.is_match(&_low(&t.user))) {
        return None;
    }
    let mut live = the_trip_so_far(memory);
    if live.is_none() {
        let lines: Vec<String> = memory.turns[memory.turns.len().saturating_sub(4)..].iter().map(|t| _low(&t.user)).collect();
        for said in lines.iter().rev() {
            if WEEKEND_AWAY.is_match(said) && !pt::strip(said).ends_with('?') {
                // No place named yet, so the trip is the kind of trip and its month.
                let how_long = _how_long(said);
                live = Some(Trip {
                    month: _month(said),
                    who: _who(said),
                    how_long: if how_long.is_empty() { "a weekend".to_string() } else { how_long },
                    plan: Some(String::new()),
                    ..Trip::default()
                });
                break;
            }
        }
    }
    let live = live?;
    let tail = pt::strip_chars(found.str(1), " ?.!,").to_string();
    let place = _low(&live.place);
    if _an_event_not_a_place(&place) {
        return None;
    }
    if !tail.is_empty() && !(PACKING_FOR.match_(&tail).is_some() || (!place.is_empty() && tail == format!("for {place}"))) {
        // Something else to pack for, which has a card of its own.
        return None;
    }
    Some(live)
}

/// Python: bot/memory/trips.py:1448 `a_packing_question_about_the_trip(text, memory)`
///
/// The trip a packing question belongs to (PyBot's dict), or None.
pub fn a_packing_question_about_the_trip(text: &str, memory: &mut Memory) -> Value {
    packing_question_about_the_trip(text, memory).map(|t| t.to_value()).unwrap_or(Value::None)
}

/// Python: bot/memory/trips.py:1496 `the_bag_beats_the_list(text, memory)`
///
/// True when the bag for the place and the month is the better answer than the list.
pub fn the_bag_beats_the_list(text: &str, memory: &mut Memory) -> bool {
    packing_question_about_the_trip(text, memory).is_some_and(|live| _the_warm_week_is_out_of_season(&live.month))
}

/// Python: bot/memory/trips.py:1508 `reply_for(text, memory, cards=True)`
///
/// The reply to a trip said out loud, or a detail added to one, or None.
pub fn reply_for(text: &str, memory: &mut Memory, cards: bool) -> Option<String> {
    let line = pt::squash(text);
    if line.is_empty() {
        return None;
    }
    if let Some(moved) = a_moved_destination(&line, memory) {
        // A destination put right is the trip already on the table with its place changed.
        return Some(_start(&a_new_trip(&line, &moved, memory)));
    }
    if let Some(trip) = a_trip(&line, memory, cards) {
        return Some(_start(&a_new_trip(&line, &trip, memory)));
    }
    if A_STAY_LENGTH.is_match(&line) {
        if let Some(live) = the_trip_so_far(memory) {
            if live.is_a_visit() {
                return Some(_start(&_pick(&VISIT_HOW_LONG, memory, &line)));
            }
        }
    }
    if let Some(somewhere) = _somewhere_for(&line, memory).filter(|s| !s.is_empty()) {
        return Some(_start(&somewhere));
    }
    // A question about the journey or the packing is still a detail of the trip.
    let about_the_trip = PACKING.is_match(&line)
        || PACKING_SAID.match_(&line).is_some()
        || THE_DRIVE.is_match(&line)
        || THE_FLIGHT.is_match(&line)
        || THE_JOURNEY_ASK.is_match(&line);
    if (JUST_A_DETAIL.match_(&line).is_some() || _only_a_length(&line) || PACKING_SAID.match_(&line).is_some() || THE_JOURNEY_ASK.is_match(&line))
        && (!line.ends_with('?') || about_the_trip)
    {
        let live = match the_trip_so_far(memory) {
            Some(live) => Some(live),
            None => packing_question_about_the_trip(&line, memory),
        };
        if let Some(live) = live {
            if let Some(detail) = a_detail(&line, &live, memory).filter(|d| !d.is_empty()) {
                return Some(_start(&detail));
            }
        }
    }
    if THE_LENGTH_ASK.is_match(&line)
        && !(THE_FLIGHT.is_match(&line) || THE_DRIVE.is_match(&line) || THE_JOURNEY_ASK.is_match(&line) || THE_MODE.is_match(&line))
    {
        if let Some(live) = the_trip_so_far(memory) {
            if live.how_long.is_empty() && _a_length_said(memory).is_none() {
                // The trip is on the table but its length is not.
                return Some(_start(&the_length_asked(&live, memory, &line)));
            }
        }
    }
    let month = _the_month_answered(&line, memory);
    if !month.is_empty() {
        if let Some(live) = the_trip_so_far(memory) {
            let live = Trip { month, ..live };
            return Some(_start(&a_new_trip(&line, &live, memory)));
        }
    }
    None
}

/// Python: bot/memory/trips.py:1578 `_only_a_length(line)`
///
/// Whether the line is the length of the stay and nothing else: "for ten days", "two weeks".
pub fn _only_a_length(line: &str) -> bool {
    let bare = pt::strip_chars(&_low(line), " .,;:!?").to_string();
    let bare = A_LENGTH_FRAME.sub("", &bare);
    let bare = pt::strip(&bare);
    FOR_HOW_LONG.fullmatch(bare).is_some() || JUST_HOW_LONG.match_(bare).is_some()
}

/// Python: bot/memory/trips.py:1594 `_the_month_answered(line, memory)`
///
/// The month in a line that is only a month, when the bot's last line asked a trip's when.
pub fn _the_month_answered(line: &str, memory: &Memory) -> String {
    let asked = _low(&crate::memory::last_bot(memory));
    if !asked.contains("when") {
        return String::new();
    }
    let Some(found) = JUST_A_MONTH.match_(line) else {
        return String::new();
    };
    let word = pt::lower(found.str(1));
    if MONTHS.contains(&word) { word } else { String::new() }
}

/// Python: bot/memory/trips.py:1612 `_as_a_visit(place)`
///
/// A relative or friend someone is visiting, said back as a place they belong to: "your
/// sister's", "your parents' house".
pub fn _as_a_visit(place: &str) -> String {
    let Some(found) = A_PERSON_IS_NOT_A_PLACE.match_(place) else {
        return place.to_string();
    };
    let person = found.name_str("person");
    let low = pt::lower(person);
    let own = found.name_str("poss").replace('\u{2019}', "'");
    let owner = if !own.is_empty() {
        format!("{person}{own}")
    } else if _PLURAL_PEOPLE.contains(&low) {
        format!("{person}{}", if low.ends_with('s') { "'" } else { "'s" })
    } else {
        format!("{person}'s")
    };
    let rest = pt::strip_chars(&place[found.end()..], " .,;:!?").to_string();
    let first = pt::split_ref(&rest).first().map(|w| pt::lower(w)).unwrap_or_default();
    if !rest.is_empty() && !_NOT_A_THING_OF_THEIRS.contains(&first) {
        return format!("your {owner} {rest}");
    }
    format!("your {owner}")
}

/// `_pick(...)` with its first letter capitalised, as the reply sentences use it.
fn picked_up(options: &[String], memory: &Memory, line: &str) -> String {
    up1(&_pick(options, memory, line))
}

/// `_a_new_trip` for a `Trip`.
fn a_new_trip(line: &str, trip: &Trip, memory: &Memory) -> String {
    let visit = _as_a_visit(&trip.place);
    let (place, who) = (_cap(&visit), trip.who.as_str());
    let month = trip.month.as_str();
    if let Some(outing) = OUTING.match_(&_low(&trip.place)) {
        // A day at the beach or the park is a day out, not a place with a name.
        let word = pt::lower(outing.str(1));
        let sea = SEA.contains(&word);
        let said = format!(
            "A day {} the {word}, {}. ",
            if OPEN_AIR.contains(&word) { "in" } else { "at" },
            _pick(&WARM, memory, line)
        );
        let who_note = if !who.is_empty() && sea { _who_note(who, &trip.how_long, memory, line, None) } else { String::new() };
        if !who_note.is_empty() {
            return said + &_join(&[who_note]);
        }
        let notes: &[String] = if sea {
            &SEA_NOTES
        } else if TICKETED.contains(&word) {
            &INDOOR_NOTES
        } else {
            &LAND_NOTES
        };
        return said + &_join(&[_pick(notes, memory, line)]);
    }
    // A day with no month behind it is said back the way they said it.
    let day = if DAYS.contains(&trip.when) && trip.when != "weekend" { trip.when.as_str() } else { "" };
    let mut named = place.clone();
    if !month.is_empty() {
        named += &format!(" in {}", _cap(month));
    } else if !day.is_empty() {
        named += &format!(" on {}", _cap(day));
    }
    if !who.is_empty() {
        named += &format!(" with {}", _yours(who));
    }
    let booked = BOOK_SAID.is_match(line);
    let said = if booked { format!("{named} is booked, then. ") } else { format!("{named}, {}. ", _pick(&WARM, memory, line)) };
    if visit != trip.place {
        // A visit is not a holiday: the length is still worth a sentence, and when there is none
        // the visit is the thing to ask about.
        let parts: Vec<String> = [_who_note(who, "", memory, line, None), _the_visit_length(&trip.how_long, memory, line)]
            .into_iter()
            .filter(|p| !p.is_empty())
            .collect();
        let note = _join(&parts);
        if !note.is_empty() {
            return _join(&[said, note]);
        }
        return _join(&[said, picked_up(&ASK_VISIT, memory, line)]);
    }
    let note = _a_note(month, who, &trip.how_long, memory, line, booked, &trip.place);
    if !note.is_empty() {
        return said + &note;
    }
    if booked {
        return said + "Get the big days out booked now too, and you can take the rest as it comes.";
    }
    if month.is_empty() && !day.is_empty() {
        return pt::strip(&said).to_string();
    }
    if month.is_empty() && !trip.when.is_empty() {
        return said + &picked_up(&ASK_WHAT, memory, line);
    }
    if month.is_empty() {
        // The one question a friend asks - unless the bot has just asked one.
        let pool = if _asked_last(memory) { &*ASK_WHEN_NO_QUESTION } else { &*ASK_WHEN };
        return fmt(&_pick(pool, memory, line), &[("place", &place)]);
    }
    said + &picked_up(&ASK_WHAT, memory, line)
}

/// Read a trip dict PyBot hands in.
fn trip_of(value: &Value) -> Trip {
    let s = |key: &str| value.get(key).filter(|v| v.truthy()).map(py_str).unwrap_or_default();
    let b = |key: &str| value.get(key).is_some_and(Value::truthy);
    Trip {
        place: s("place"),
        month: s("month"),
        when: s("when"),
        who: s("who"),
        how_long: s("how_long"),
        by: s("by"),
        visit: if value.get("visit").is_some() { Some((b("visit"), b("meal"), b("baby"))) } else { None },
        plan: value.get("plan").map(py_str),
    }
}

/// Python: bot/memory/trips.py:1640 `_a_new_trip(line, trip, memory)`
pub fn _a_new_trip(line: &str, trip: &Value, memory: &Memory) -> String {
    a_new_trip(line, &trip_of(trip), memory)
}

/// Python: bot/memory/trips.py:1703 `_a_length_said(memory)`
///
/// The length the chat has already given for the trip, in their own words, or None.
pub fn _a_length_said(memory: &Memory) -> Option<String> {
    for turn in memory.turns.iter().rev() {
        let text = &turn.user;
        if text.is_empty() || pt::rstrip(text).ends_with('?') {
            continue;
        }
        if let Some(found) = FOR_HOW_LONG.search(text) {
            return Some(found.str(1).to_string());
        }
    }
    None
}

/// `_the_length_asked` for a `Trip`.
fn the_length_asked(trip: &Trip, memory: &Memory, line: &str) -> String {
    let place = _cap(&_as_a_visit(&trip.place));
    let month = &trip.month;
    let options: Vec<String> = if !place.is_empty() {
        let named = _mid(&format!("{place}{}", if month.is_empty() { String::new() } else { format!(" in {}", _cap(month)) }));
        THE_LENGTH_NOT_SAID_NAMED.iter().map(|n| fmt(n, &[("named", &named)])).collect()
    } else {
        THE_LENGTH_NOT_SAID.clone()
    };
    _join(&[_pick(&options, memory, line), _pick(&THE_NIGHTS_FIRST, memory, line)])
}

/// Python: bot/memory/trips.py:1724 `_the_length_asked(trip, memory, line)`
///
/// The length of the stay asked while the chat has not said one: the question back.
pub fn _the_length_asked(trip: &Value, memory: &Memory, line: &str) -> String {
    the_length_asked(&trip_of(trip), memory, line)
}

/// `_a_detail` for a `Trip`.
fn a_detail(line: &str, trip: &Trip, memory: &Memory) -> Option<String> {
    let low = _low(line);
    let (place, month) = (_cap(&_as_a_visit(&trip.place)), trip.month.as_str());
    let named = format!("{place}{}", if month.is_empty() { String::new() } else { format!(" in {}", _cap(month)) });
    let who = _who(&low);
    let how_long = _how_long(&low);
    // The mode is a note for the line that adds it to a trip already on the table.
    let mut by = if !low.contains(&_low(&place)) { _how_getting_there(&low) } else { String::new() };
    let mut told: Vec<String> = Vec::new();
    let named = _mid(&named);
    let kids = CHILDREN.is_match(&who);
    let packing = PACKING.match_(&low).or_else(|| PACKING_SAID.match_(&low));
    if let Some(packing) = packing {
        let tail = pt::strip_chars(packing.str(1), " ?.!,").to_string();
        let said_as_a_need = PACKING_SAID.match_(&low).is_some();
        if place.is_empty() {
            // A trip with no place yet: the list is about the kind of trip and the month.
            let kind = _the_kind_of_trip(&trip.how_long);
            let pool: Vec<String> = if !month.is_empty() {
                PACKING_NO_PLACE.iter().map(|n| fmt(n, &[("kind", &kind), ("month", &_cap(month))])).collect()
            } else {
                PACKING_NO_PLACE_NO_MONTH.iter().map(|n| fmt(n, &[("kind", &kind)])).collect()
            };
            return Some(_join(&[_pick(&pool, memory, line)]));
        }
        if trip.is_a_visit() {
            // A visit to family is not a holiday with a packing list.
            if said_as_a_need && !_one_thing_they_need(&low).is_empty() {
                return None;
            }
            if trip.baby() {
                return Some(_pick(&VISIT_PACKING_FOR_A_BABY, memory, line));
            }
            if trip.meal() {
                return Some(_pick(&VISIT_PACKING_FOR_A_MEAL, memory, line));
            }
            let pool: Vec<String> = VISIT_PACKING.iter().map(|n| fmt(n, &[("place", &_mid(&place))])).collect();
            return Some(_pick(&pool, memory, line));
        }
        if !tail.is_empty() && !said_as_a_need && !(PACKING_FOR.match_(&tail).is_some() || _low(&tail) == format!("for {}", _low(&place))) {
            return None; // something else to pack for
        }
        if _an_event_not_a_place(&place) {
            return None;
        }
        let mut notes: Vec<String> = if !month.is_empty() {
            PACKING_NOTES.iter().map(|n| fmt(n, &[("place", &_mid(&place)), ("month", &_cap(month))])).collect()
        } else {
            PACKING_NO_MONTH.iter().map(|n| fmt(n, &[("place", &_mid(&place))])).collect()
        };
        if kids {
            notes.push(PACKING_WITH_KIDS.clone());
        }
        return Some(_join(&[_pick(&notes, memory, line)]));
    }
    if (THE_FLIGHT.is_match(&low) || THE_DRIVE.is_match(&low)) && HOW_LONG_ASK.is_match(&low) && _a_carded_place(&place) {
        // The card knows the hours.
        return None;
    }
    if by.is_empty() && !THE_MODE.is_match(&low) {
        if THE_FLIGHT.is_match(&low) {
            by = "flying".into();
        } else if THE_DRIVE.is_match(&low) {
            by = "driving".into();
        } else if THE_JOURNEY_ASK.is_match(&low) {
            by = if JOURNEY_MODE.is_match(&low) { "driving".into() } else { "either".into() };
        }
    }
    if by == "either" || (by.is_empty() && THE_MODE.is_match(&low)) {
        told.push(format!(
            "To {named}: the car or the train if it is a few hours, a flight if it is not - the train is the one you can leave late on."
        ));
    } else if by == "driving" || by == "by car" {
        told.push(format!("Driving to {named}, so leave early and take the break halfway - the last hour is the one that drags."));
    } else if by == "flying" || by == "by plane" {
        told.push(format!("Flying to {named} - take the early flight, because the day you lose waiting around is the first day of the trip."));
    } else if matches!(by.as_str(), "getting the train" | "on the train" | "by train" | "getting a train") {
        told.push(format!("Taking the train to {named} is the easy way in - book it now, the cheap seats go first."));
    } else if matches!(by.as_str(), "taking the ferry" | "getting a ferry" | "sailing" | "by ferry") {
        told.push(format!("On the ferry to {named}, so book it now - the summer crossings fill up."));
    }
    if !told.is_empty() {
        if !who.is_empty() {
            let note = _who_note(&who, &how_long, memory, line, Some(&place));
            told.push(if note.is_empty() { _with_them(&named, &who, memory, line) } else { note });
        }
        let told: Vec<String> = told.into_iter().filter(|t| !t.is_empty()).collect();
        return Some(_join(&told));
    }
    if !who.is_empty() && kids {
        return Some(_join(&[_who_note(&who, &how_long, memory, line, Some(&place))]));
    }
    if !who.is_empty() {
        return Some(_with_them(&named, &who, memory, line));
    }
    let note = _who_note(&who, &how_long, memory, line, Some(&place));
    if !note.is_empty() {
        return Some(_join(&[note]));
    }
    None
}

/// Python: bot/memory/trips.py:1735 `_a_detail(line, trip, memory)`
pub fn _a_detail(line: &str, trip: &Value, memory: &Memory) -> Option<String> {
    a_detail(line, &trip_of(trip), memory)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Values from CPython.
    #[test]
    fn places_as_python_says_them() {
        assert_eq!(["the isle of wight", "new york", "3 hills"].map(_cap), ["The Isle of Wight", "New York", "3 Hills"]);
        assert_eq!(
            ["my sisters", "my parents house", "my mum's", "my brothers tonight"].map(_as_a_visit),
            ["your sister's", "your parents' house", "your mum's", "your brother's"]
        );
        assert_eq!(_yours("my mum and our kids"), "your mum and your kids");
    }
}
