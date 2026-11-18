//! Port of `bot/compose/finding_out.py`: the answer the bot does not hold, and the one place that
//! has it - a trade to hire, a thing to buy, a route, a figure, a record, a tide table.
#![allow(non_snake_case, clippy::all)]

use crate::compose::fallback_chain::py_format;
use crate::data;
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.finding_out.", stringify!($name))));)*
    };
}
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.compose.finding_out.", stringify!($name))));)*
    };
}
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.finding_out.", stringify!($name))));)*
    };
}

regexes!(
    NOT_A_PLACE, NOT_A_LOOKUP, ROUTE_ASKS, TAKE_TO_ASKS, OWN_PROFESSIONAL, IN_A_PLACE, NEAR_ME, WORTH_VISITING,
    WHO_FACT, FIGURE_ASKS, SETTING_ASKS, METHOD_ASKS, MATERIAL_ASK, BUY_ASK, BUY_THING, BUY_THING_STOPS, SERVICE_ASK,
    SERVICE_THING, OPEN_CHEMIST_HOURS, RECORD_ASKS, RECORD_ASKED, TIDE_ASK, TIDE_WHY, FILLING_IT_IN, SAVING_FAILED,
    LOST_THE_WORK, ON_A_SCREEN, THE_WORK_ITSELF, BACK_TO_START, THE_BARE_FORMS,
);
word_sets!(OWN_PERSON, BUY_TAIL, BUY_NOT_THINGS, SERVICE_NOT_A_THING, SERVICE_NOT_HEALTH, SERVICE_STOPS);
pools!(
    ABOUT_THE_BOT, CONTACT_ASKS, WHERE_ASKS, PLACE_NOUNS, PLACE_POOL, TRAVEL_WORDS, GET_TO_ASKS, NOT_A_JOURNEY,
    GOES_TO, PLACE_ROUTE_POOL, TRADES, HIRING, NOT_A_TRADE, NOT_A_HIRE, QUOTING_TRADES, TRADE_POOL,
    PROFESSIONAL_POOL, PICKED_IN_A_PLACE, PICK_ASKS, NOT_A_PICK, PICK_POOL, PICK_ANYWHERE_POOL, WHO_POOL,
    POLICY_WORDS, FIGURE_POOL, MATERIALS, HOW_MANY_POOL, BUY_TAIL_PHRASES, BUY_NOT, BUY_NOT_PEOPLE,
    BUY_NOT_THE_THING, SERVICE_POOL, BUY_POOL, CHEMISTS, OPEN_CHEMIST_POOL, RECORD_POOL, TRAVEL_TOPICS, TIDE_POOL,
    NOT_A_WRECKED_FORM, WRECKED_POOL,
);

/// `SERVICE_WORK`: verb -> the work, in dict order.
static SERVICE_WORK: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.compose.finding_out.SERVICE_WORK").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
});
/// `SERVICE_PHRASES`: (phrase, work).
static SERVICE_PHRASES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.compose.finding_out.SERVICE_PHRASES")
        .items()
        .iter()
        .map(|row| (row.items()[0].str().to_string(), row.items()[1].str().to_string()))
        .collect()
});
/// `SERVICE_NOUNS`: (noun, (work, thing)).
static SERVICE_NOUNS: LazyLock<Vec<(String, String, String)>> = LazyLock::new(|| {
    data::get("bot.compose.finding_out.SERVICE_NOUNS")
        .items()
        .iter()
        .map(|row| {
            let pair = row.items()[1].items();
            (row.items()[0].str().to_string(), pair[0].str().to_string(), pair[1].str().to_string())
        })
        .collect()
});
/// `SERVICE_JOB_NAMES`: (thing, work) -> the job's name.
static SERVICE_JOB_NAMES: LazyLock<HashMap<(String, String), String>> = LazyLock::new(|| {
    data::get("bot.compose.finding_out.SERVICE_JOB_NAMES")
        .pairs()
        .iter()
        .map(|(k, v)| ((k.items()[0].str().to_string(), k.items()[1].str().to_string()), v.str().to_string()))
        .collect()
});
/// `READERS`, by the Python ids of the route functions, in order.
static READERS: LazyLock<Vec<String>> = LazyLock::new(|| {
    data::get("bot.compose.finding_out.READERS").items().iter().map(|f| f.as_fn().expect("a reader").name().to_string()).collect()
});

static A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9']+", 0));
static HOW_MANY_OR_MUCH: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<![a-z])how (?:many|much)(?![a-z])", 0));
static FROM_TO: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<![a-z])from\b[^.!?]{1,40}\bto(?![a-z])", 0));
static HOW_DO_I: LazyLock<Re> = LazyLock::new(|| Re::new(r"how (?:do|can|should|would) (?:i|you|we)", 0));
static A_BUYING_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<![a-z])(?:what|anything|something|buy|get|recommend)(?![a-z])", 0));
static AN_OPENING_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<![a-z])(?:open|closes|close|shut|late|tonight)(?![a-z])", 0));
static WHO_IS_IT: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"(?<![a-z])(?:who|whos|who's)\s+(?:is|was|were|are)\s+(?:it|that|this|he|she|they|them|you|thou)\b", 0)
});
static HOW_BIG_IS_IT: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"(?<![a-z])how (?:old|tall|big|heavy|deep|far) (?:is|are) (?:he|she|they|it|you|we)(?![a-z])", 0)
});
static A_BEST_OF: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"(?<![a-z])(?:the best|my best|your best|best way|worst way|best time)(?![a-z])", 0));
static A_LETTER: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]", 0));

/// `r"(?<![a-z0-9])" + re.escape(phrase) + r"(?![a-z0-9])"` (and the `[a-z]` form), compiled once.
fn whole(phrase: &str, digits: bool) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<(String, bool), &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    cache
        .entry((phrase.to_string(), digits))
        .or_insert_with(|| {
            let edge = if digits { "a-z0-9" } else { "a-z" };
            Box::leak(Box::new(Re::new(&format!("(?<![{edge}]){}(?![{edge}])", pyre::escape(phrase)), 0)))
        })
}

/// The phrase as whole words with any run of space between them, compiled once.
fn whole_spaced(parts: &[&str]) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<String, &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let key = parts.join(" ");
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    cache.entry(key).or_insert_with(|| {
        let body: Vec<String> = parts.iter().map(|p| pyre::escape(p)).collect();
        Box::leak(Box::new(Re::new(&format!(r"(?<![a-z0-9]){}(?![a-z0-9])", body.join(r"\s+")), 0)))
    })
}

/// Python: bot/compose/finding_out.py:34 `_has(line, phrases)`
/// True when one of the phrases is in the line as a whole word or phrase.
pub fn _has(line: &str, phrases: &[String]) -> bool {
    phrases.iter().any(|phrase| whole(phrase, true).is_match(line))
}

/// Python: bot/compose/finding_out.py:46 `_words(line)` (a set in Python)
pub fn _words(line: &str) -> Vec<String> {
    A_WORD.findall_str(line)
}

fn word_set(line: &str) -> HashSet<String> {
    _words(line).into_iter().collect()
}

/// Python: bot/compose/finding_out.py:50 `_names_one_of(line, phrases)`
/// True when the line names one of the things, as a whole word or a whole phrase.
pub fn _names_one_of(line: &str, phrases: &[String]) -> bool {
    let words = word_set(line);
    for phrase in phrases {
        let parts = text::split_ref(phrase);
        if parts.len() == 1 {
            if words.contains(parts[0]) {
                return true;
            }
        } else if whole_spaced(&parts).is_match(line) {
            return true;
        }
    }
    false
}

/// What the readers use of the understanding engine's reading.
#[derive(Clone, Debug, Default)]
pub struct Ctx {
    pub places: HashSet<String>,
    pub travel: bool,
    pub pointed: String,
    pub settled: bool,
}

/// Python: bot/compose/finding_out.py:491 `_the_trade(line)`
/// The trade named in the line, or "" when it names none.
pub fn _the_trade(line: &str) -> String {
    TRADES.iter().find(|trade| whole(trade, false).is_match(line)).cloned().unwrap_or_default()
}

/// Python: bot/compose/finding_out.py:499 `_the_thing(line)`
/// The thing being bought, with its article, or "" when the line does not yield one.
pub fn _the_thing(line: &str) -> String {
    let Some(found) = BUY_THING.search(line) else { return String::new() };
    let mut text_ = format!(" {}", text::strip(found.str(1)));
    for phrase in BUY_TAIL_PHRASES.iter() {
        if text_.ends_with(phrase.as_str()) {
            text_.truncate(text_.len() - phrase.len());
        }
    }
    if let Some(cut) = BUY_THING_STOPS.search(&text_) {
        let at = cut.start();
        text_.truncate(at);
    }
    let mut words: Vec<&str> = text::split_ref(&text_).into_iter().filter(|w| !BUY_TAIL.contains(*w)).collect();
    while words.last().is_some_and(|w| ["a", "an", "the", "some", "any", "my", "our", "of", "for"].contains(w)) {
        words.pop();
    }
    let thing = text::strip(&words.iter().take(4).copied().collect::<Vec<_>>().join(" ")).to_string();
    if thing.is_empty() || ["a", "an", "the", "some", "any", "it", "one", "them", "me"].contains(&thing.as_str()) {
        return String::new();
    }
    thing
}

/// `tuple(line.format(name=value) for line in pool)`.
fn filled(pool: &[String], name: &str, value: &str) -> Vec<String> {
    pool.iter().map(|line| py_format(line, &[(name, value)])).collect()
}

/// Python: bot/compose/finding_out.py:520 `_trade_route(line, ctx)`
pub fn _trade_route(line: &str, _ctx: &Ctx) -> Option<Vec<String>> {
    let trade = _the_trade(line);
    if trade.is_empty() || _has(line, &NOT_A_TRADE) || _has(line, &NOT_A_HIRE) {
        return None;
    }
    if OWN_PROFESSIONAL.is_match(line) {
        return None;
    }
    if !_has(line, &HIRING) {
        return None;
    }
    let pool = if QUOTING_TRADES.contains(&trade) { &*TRADE_POOL } else { &*PROFESSIONAL_POOL };
    Some(filled(pool, "trade", &trade))
}

/// Python: bot/compose/finding_out.py:532 `_buy_route(line, ctx)`
pub fn _buy_route(line: &str, _ctx: &Ctx) -> Option<Vec<String>> {
    if !BUY_ASK.is_match(line) || _has(line, &BUY_NOT) {
        return None;
    }
    let thing = _the_thing(line);
    let words = text::split_ref(&thing);
    if thing.is_empty() || words.last().is_some_and(|w| BUY_NOT_THINGS.contains(*w)) {
        return None;
    }
    // "where can i buy my prescription" is their own thing, which a shop does not sell.
    if ["my", "our"].contains(&text::strip(words[0])) {
        return None;
    }
    let padded = format!(" {thing} ");
    if _has(&padded, &BUY_NOT_THE_THING) || _has(&padded, &BUY_NOT_PEOPLE) {
        return None;
    }
    Some(filled(&BUY_POOL, "thing", &thing))
}

/// Python: bot/compose/finding_out.py:547 `_the_service(line, pointed="")`
/// (the work, the thing) for a line that puts a thing in for work, or None. `pointed` is what a
/// pronoun in the line points at, from the understanding engine.
pub fn _the_service(line: &str, pointed: &str) -> Option<(String, String)> {
    if !SERVICE_ASK.is_match(line) {
        return None;
    }
    let mut work = String::new();
    for (phrase, word) in SERVICE_PHRASES.iter() {
        if whole(phrase, false).is_match(line) {
            work = word.clone();
            break;
        }
    }
    if work.is_empty() {
        let mut by_length: Vec<&(String, String)> = SERVICE_WORK.iter().collect();
        by_length.sort_by_key(|(verb, _)| std::cmp::Reverse(text::len(verb)));
        for (verb, word) in by_length {
            if whole(verb, false).is_match(line) {
                work = word.clone();
                break;
            }
        }
    }
    if work.is_empty() {
        for (noun, word, thing) in SERVICE_NOUNS.iter() {
            if whole(noun, false).is_match(line) {
                return Some((word.clone(), thing.clone()));
            }
        }
        return None;
    }
    let found = SERVICE_THING.search(line)?;
    let mut words: Vec<String> = Vec::new();
    for word in text::split_ref(found.str(1)) {
        if ["a", "an", "the", "my", "our", "some", "of", "for", "to", "in", "on"].contains(&word) {
            continue;
        }
        if BUY_TAIL.contains(word) || SERVICE_WORK.iter().any(|(verb, _)| verb == word) || SERVICE_STOPS.contains(word) {
            break;
        }
        words.push(word.to_string());
    }
    if words.first().is_some_and(|w| SERVICE_NOT_A_THING.contains(w)) {
        // "can i get them taken in": the pronoun is the thing, and the reading says what it is.
        let low = text::lower(pointed);
        words = text::split_ref(&low).into_iter().filter(|w| A_LETTER.is_match(w)).map(str::to_string).collect();
        if words.first().is_none_or(|w| SERVICE_NOT_A_THING.contains(w)) {
            return None;
        }
    }
    if words.is_empty() {
        return None;
    }
    if words.iter().any(|w| SERVICE_NOT_HEALTH.contains(w)) {
        return None;
    }
    Some((work, words.iter().take(3).cloned().collect::<Vec<_>>().join(" ")))
}

/// Python: bot/compose/finding_out.py:593 `_service_route(line, ctx)`
pub fn _service_route(line: &str, ctx: &Ctx) -> Option<Vec<String>> {
    let (work, thing) = _the_service(line, &ctx.pointed)?;
    let job = SERVICE_JOB_NAMES
        .get(&(thing.clone(), work.clone()))
        .filter(|j| !j.is_empty())
        .cloned()
        .unwrap_or_else(|| text::strip(&format!("{thing} {work}")).to_string());
    Some(filled(&SERVICE_POOL, "job", &job))
}

/// Python: bot/compose/finding_out.py:602 `_about_a_place(line, ctx)`
/// True when the line is about somewhere out in the world, not "in food" or "in a new job".
pub fn _about_a_place(line: &str, ctx: &Ctx) -> bool {
    if !ctx.places.is_empty() || ctx.travel {
        return true;
    }
    if WORTH_VISITING.is_match(line) {
        return true;
    }
    NEAR_ME.is_match(line)
}

/// Python: bot/compose/finding_out.py:618 `_the_picked_thing(line)`
/// The phrase the person picked out of the place: "beaches" from "the best beach in cornwall".
pub fn _the_picked_thing(line: &str) -> String {
    let mut found = String::new();
    for phrase in PICKED_IN_A_PLACE.iter() {
        if text::len(phrase) <= text::len(&found) {
            continue;
        }
        if _names_one_of(line, std::slice::from_ref(phrase)) {
            found = phrase.clone();
        }
    }
    if found == "thing to do" || found == "things to do" {
        return "things to do".to_string();
    }
    found
}

/// Python: bot/compose/finding_out.py:636 `_pick_route(line, ctx)`
/// Something to do, eat or stay in somewhere: the picks are the reviews' job, not the bot's.
pub fn _pick_route(line: &str, ctx: &Ctx) -> Option<Vec<String>> {
    if !_has(line, &PICK_ASKS) && !WORTH_VISITING.is_match(line) {
        return None;
    }
    if _has(line, &NOT_A_PICK) || !_about_a_place(line, ctx) {
        return None;
    }
    if WORTH_VISITING.is_match(line) && IN_A_PLACE.is_match(line) {
        let thing = _the_picked_thing(line);
        if thing.is_empty() {
            // "is cornwall worth visiting" names the place and no thing in it.
            return Some(PICK_ANYWHERE_POOL.clone());
        }
        return Some(filled(&PICK_POOL, "thing", &thing));
    }
    let where_ = IN_A_PLACE.search(line)?;
    // The thing is the phrase before the place: "the best food in town" picks food.
    let thing = _the_picked_thing(&line[..where_.start()]);
    if thing.is_empty() {
        return None;
    }
    Some(filled(&PICK_POOL, "thing", &thing))
}

/// Python: bot/compose/finding_out.py:661 `_how_many_route(line, ctx)`
pub fn _how_many_route(line: &str, _ctx: &Ctx) -> Option<Vec<String>> {
    if !(MATERIAL_ASK.is_match(line) && _names_one_of(line, &MATERIALS)) {
        return None;
    }
    if !HOW_MANY_OR_MUCH.is_match(line) {
        return None;
    }
    Some(HOW_MANY_POOL.clone())
}

/// Python: bot/compose/finding_out.py:669 `_place_route(line, ctx)`
pub fn _place_route(line: &str, _ctx: &Ctx) -> Option<Vec<String>> {
    if NOT_A_PLACE.is_match(line) || NOT_A_LOOKUP.is_match(line) {
        return None;
    }
    if _has(line, &CONTACT_ASKS) {
        // A postcode or a phone number is a lookup wherever the line puts it.
        return Some(PLACE_POOL.clone());
    }
    if !_has(line, &WHERE_ASKS) {
        return None;
    }
    if !_names_one_of(line, &PLACE_NOUNS) {
        return None;
    }
    Some(PLACE_POOL.clone())
}

/// Python: bot/compose/finding_out.py:683 `_route_route(line, ctx)`
/// How to get somewhere, and how far it is: the journey is the map app's to measure.
pub fn _route_route(line: &str, ctx: &Ctx) -> Option<Vec<String>> {
    if _has(line, &NOT_A_JOURNEY) {
        return None;
    }
    let words = word_set(line);
    let travel = TRAVEL_WORDS.iter().any(|w| words.contains(w));
    let places = !ctx.places.is_empty();
    if ROUTE_ASKS.is_match(line) {
        if !(travel || places) {
            return None;
        }
        // "how long does it take to" is a journey only when it goes somewhere.
        if TAKE_TO_ASKS.is_match(line) && !(places || _names_one_of(line, &GOES_TO)) {
            return None;
        }
        return Some(PLACE_ROUTE_POOL.clone());
    }
    if _has(line, &GET_TO_ASKS) && (travel || places) {
        return Some(PLACE_ROUTE_POOL.clone());
    }
    if FROM_TO.is_match(line) && travel && HOW_DO_I.is_match(line) {
        return Some(PLACE_ROUTE_POOL.clone());
    }
    None
}

/// Python: bot/compose/finding_out.py:729 `_open_chemist_route(line, ctx)`
/// A chemist that has to be open now, or near, or on a day the usual one shuts.
pub fn _open_chemist_route(line: &str, _ctx: &Ctx) -> Option<Vec<String>> {
    if !_has(line, &CHEMISTS) || !OPEN_CHEMIST_HOURS.is_match(line) {
        return None;
    }
    // "what can i get from the chemist tonight" asks what to buy, not where is open.
    if A_BUYING_WORD.is_match(line) && !AN_OPENING_WORD.is_match(line) {
        return None;
    }
    Some(OPEN_CHEMIST_POOL.clone())
}

/// Python: bot/compose/finding_out.py:741 `_who_route(line, ctx)`
pub fn _who_route(line: &str, _ctx: &Ctx) -> Option<Vec<String>> {
    if !WHO_FACT.is_match(line) || _has(line, &ABOUT_THE_BOT) {
        return None;
    }
    if word_set(line).iter().any(|w| OWN_PERSON.contains(w)) {
        return None;
    }
    // "who is it" and "who is he" point back at the chat, not at a fact.
    if WHO_IS_IT.is_match(line) {
        return None;
    }
    Some(WHO_POOL.clone())
}

/// Python: bot/compose/finding_out.py:755 `_tide_route(line, ctx)`
/// When the sea turns, which is a table for one beach and one day: the route names the table.
pub fn _tide_route(line: &str, _ctx: &Ctx) -> Option<Vec<String>> {
    if !TIDE_ASK.is_match(line) {
        return None;
    }
    if TIDE_WHY.is_match(line) {
        // Why there are tides is a fact the bot does hold.
        return None;
    }
    Some(TIDE_POOL.clone())
}

/// Python: bot/compose/finding_out.py:770 `_figure_route(line, ctx)`
pub fn _figure_route(line: &str, _ctx: &Ctx) -> Option<Vec<String>> {
    if !FIGURE_ASKS.is_match(line) || SETTING_ASKS.is_match(line) {
        return None;
    }
    if METHOD_ASKS.is_match(line) || _has(&format!(" {line} "), &POLICY_WORDS) {
        return None;
    }
    if word_set(line).iter().any(|w| OWN_PERSON.contains(w)) {
        return None;
    }
    if HOW_BIG_IS_IT.is_match(line) {
        return None;
    }
    Some(FIGURE_POOL.clone())
}

/// Python: bot/compose/finding_out.py:808 `_record_route(line, ctx)`
/// A record the bot does not hold: the page that keeps the list, not today's news.
pub fn _record_route(line: &str, ctx: &Ctx) -> Option<Vec<String>> {
    if !ctx.settled {
        return None;
    }
    if !RECORD_ASKS.is_match(line) || !RECORD_ASKED.is_match(line) {
        return None;
    }
    if word_set(line).iter().any(|w| OWN_PERSON.contains(w)) || SETTING_ASKS.is_match(line) || METHOD_ASKS.is_match(line) {
        return None;
    }
    if A_BEST_OF.is_match(line) {
        return None;
    }
    let padded = format!(" {line} ");
    let sizes: Vec<String> =
        ["size", "sizes", "pack", "packs", "box", "boxes", "bottle", "bottles", "tin", "tins", "bag", "bags"].iter().map(|s| s.to_string()).collect();
    if _has(&padded, &sizes) {
        return None;
    }
    let numbers: Vec<String> = ["number", "numbers", "word", "words", "letter", "letters", "digit", "digits", "prime", "sum", "total"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    if _has(&padded, &numbers) {
        return None;
    }
    Some(RECORD_POOL.clone())
}

/// Python: bot/compose/finding_out.py:839 `_the_reading(understood)`
/// The parts of the reading the readers use, from what the understanding engine says.
pub fn _the_reading(understood: Option<&Understood>) -> Ctx {
    let Some(understood) = understood else { return Ctx::default() };
    let places: HashSet<String> =
        understood.entities.iter().filter(|e| e.kind == "place" && !e.text.is_empty()).map(|e| e.text.clone()).collect();
    let r = &understood.reference;
    let mut pointed = String::new();
    for value in [&r.it, &r.that, &r.they] {
        if let Some(value) = value.as_deref().filter(|v| !text::strip(v).is_empty()) {
            pointed = text::strip(value).to_string();
            break;
        }
    }
    let mut topics: HashSet<&str> = understood.topics.iter().map(String::as_str).collect();
    if let Some(topic) = understood.topic.as_deref().filter(|t| !t.is_empty()) {
        topics.insert(topic);
    }
    let travel = TRAVEL_TOPICS.iter().any(|t| topics.contains(t.as_str()));
    // A record is only asked about a thing that is settled, not one that changes over time.
    let settled = understood.over_time == Some(false);
    Ctx { places, travel, pointed, settled }
}

/// Python: bot/compose/finding_out.py:964 `the_wrecked_form_reply(line)`
/// The pool for a form or a page that threw the person's work away, or None.
pub fn the_wrecked_form_reply(line: &str) -> Option<Vec<String>> {
    if line.is_empty() || _has(line, &NOT_A_WRECKED_FORM) {
        return None;
    }
    if !FILLING_IT_IN.is_match(line) || !LOST_THE_WORK.is_match(line) {
        return None;
    }
    // A bare "form" or "application" could be on paper: it needs a screen, a save, a restart or
    // the work itself in the line.
    if THE_BARE_FORMS.is_match(line)
        && !(ON_A_SCREEN.is_match(line) || SAVING_FAILED.is_match(line) || BACK_TO_START.is_match(line) || THE_WORK_ITSELF.is_match(line))
    {
        return None;
    }
    Some(WRECKED_POOL.clone())
}

/// Run one of `READERS` by its Python id.
fn run_reader(id: &str, line: &str, ctx: &Ctx) -> Option<Vec<String>> {
    let name = id.split(':').nth(1).unwrap_or("");
    match name {
        "_trade_route" => _trade_route(line, ctx),
        "_service_route" => _service_route(line, ctx),
        "_buy_route" => _buy_route(line, ctx),
        "_pick_route" => _pick_route(line, ctx),
        "_how_many_route" => _how_many_route(line, ctx),
        "_place_route" => _place_route(line, ctx),
        "_route_route" => _route_route(line, ctx),
        "_who_route" => _who_route(line, ctx),
        "_tide_route" => _tide_route(line, ctx),
        "_figure_route" => _figure_route(line, ctx),
        "_record_route" => _record_route(line, ctx),
        "_open_chemist_route" => _open_chemist_route(line, ctx),
        other => panic!("finding_out: no reader {other:?}"),
    }
}

/// Python: bot/compose/finding_out.py:997 `for_the_ask(clean, repaired="", understood=None)`
/// The pool of honest routes for this ask, or None when none of them beats the hedge. The typed
/// line is read first, and the repaired one only catches what a slip hid.
pub fn for_the_ask(clean: &str, repaired: &str, understood: &Understood) -> Option<Vec<String>> {
    for_the_ask_with(clean, repaired, Some(understood))
}

/// `for_the_ask` with `understood` None or given.
pub fn for_the_ask_with(clean: &str, repaired: &str, understood: Option<&Understood>) -> Option<Vec<String>> {
    if clean.is_empty() {
        return None;
    }
    let ctx = _the_reading(understood);
    for line in [clean, repaired] {
        if text::strip(line).is_empty() {
            continue;
        }
        for reader in READERS.iter() {
            if let Some(found) = run_reader(reader, line, &ctx).filter(|p| !p.is_empty()) {
                return Some(found);
            }
        }
    }
    None
}
