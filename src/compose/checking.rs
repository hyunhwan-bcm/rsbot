//! Port of `bot/compose/checking.py`: the things the bot cannot see - a streaming catalogue, the
//! weather, a timetable, a shop's stock, a council's rules - and the one place that does see them.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.checking.", stringify!($name))));)*
    };
}
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.compose.checking.", stringify!($name))));)*
    };
}
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.checking.", stringify!($name))));)*
    };
}

regexes!(
    NOT_AVAILABILITY, GET_FROM_TO, WANTS_A_TIME, NOT_A_DESTINATION, NOT_A_JOURNEY, SHOP_ASK, A_PLACE, WANTS_A_PICK,
    A_MEAL_TIME, A_PART_OF_DAY, WANTS_A_PLAN, AN_ON_ASK, WANTS_A_DAY_OUT, DAY_OUT_NOT, ASKS_WHAT_TO_DO,
    A_THING_IN_HAND, A_PLACE_THERE, A_BARE_DAY_OUT, A_TRIP_BEFORE, NOT_A_DAY_OUT_PLACE, ABROAD, MOVING, GOING_AWAY,
    A_THING_NOT_A_PLACE, NOT_THE_PAPERWORK, MOVING_HOUSE, COUNCIL_WORD, PARKING_RULE, WANTS_THE_RULES, _THE_ENDS,
    _THE_ENDS_REVERSED, _A_QUALIFIER_AFTER_THE_PLACE, A_SKY_ASK, A_SETTING, ONLY_A_SUBJECT, HAS_A_VERB,
    A_QUESTION_OF_ITS_OWN, WANTS_A_CHOICE, NOT_A_CHOICE, WANTS_A_SHORTLIST, NOT_A_SHORTLIST, GET_A_FORM, SEND_A_FORM,
    NOT_A_FORM, WANTS_A_CHECK, CHECK_OF_SOMETHING_ELSE, A_CHECK_SENTENCE, NOT_A_CHECK_SENTENCE, A_CONDITION,
);
word_sets!(COUNTRIES, HOME_NATIONS, MOVE_HOME_NAMES, _NOT_THE_PLACE, _A_NAME_PART, POINTING_WORDS);
pools!(
    NOT_HOME_COUNTRIES, MEAL_TIME_POOL, PART_OF_DAY_POOL, THINGS_TO_DO_POOL, SERVICES_POOL, JOURNEY_POOL,
    ROUTE_ENDS_POOL, ROUTE_POOL, PICK_POOL, SHOP_POOL, ABROAD_POOL, WEATHER_POOL, COUNCIL_POOL, CHOICE_POOL,
    SHORTLIST_POOL, FORM_FROM_POOL, FORM_TO_POOL, CHECKS_POOL,
);

fn named(name: &str) -> &'static Value {
    data::get(&format!("bot.compose.checking.{name}"))
}
/// (word, shown) rows: `SERVICES`, `SHOPS`.
fn word_rows(name: &str) -> Vec<(String, String)> {
    named(name).items().iter().map(|row| (row.items()[0].str().to_string(), row.items()[1].str().to_string())).collect()
}
static SERVICES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| word_rows("SERVICES"));
static SHOPS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| word_rows("SHOPS"));
/// `TRAVEL`: word -> (single, plural), in dict order.
static TRAVEL: LazyLock<Vec<(String, String, String)>> = LazyLock::new(|| {
    named("TRAVEL")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), v.items()[0].str().to_string(), v.items()[1].str().to_string()))
        .collect()
});
/// `PICKS`: (words, thing).
static PICKS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    named("PICKS").items().iter().map(|row| (row.items()[0].strs(), row.items()[1].str().to_string())).collect()
});
static PICK_NOT: LazyLock<HashMap<String, &'static Re>> = LazyLock::new(|| {
    named("PICK_NOT").pairs().iter().filter_map(|(k, v)| v.as_re().map(|re| (k.str().to_string(), re))).collect()
});
static SUBJECT_KINDS: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    named("SUBJECT_KINDS").items().iter().map(|row| (row.items()[0].str().to_string(), row.items()[1].as_re().expect("a pattern"))).collect()
});
static _SUBJECT_LABEL: LazyLock<HashMap<String, String>> = LazyLock::new(|| named("_SUBJECT_LABEL").str_map());
static _SUBJECT_MODE: LazyLock<HashMap<String, String>> = LazyLock::new(|| named("_SUBJECT_MODE").str_map());
static A_FEW_WORDS: LazyLock<usize> = LazyLock::new(|| named("A_FEW_WORDS").as_int().expect("an int") as usize);

/// `_A_NAMED_COUNTRY`: every country the knowledge base knows, and its aliases, but the moves at
/// home, longest first.
static _A_NAMED_COUNTRY: LazyLock<Re> = LazyLock::new(|| {
    let mut names: Vec<String> = data::get("bot.knowledge.geography.COUNTRIES").pairs().iter().map(|(k, _)| k.str().to_string()).collect();
    for (k, _) in data::get("bot.knowledge.geography._COUNTRY_ALIASES").pairs() {
        if !names.iter().any(|n| n == k.str()) {
            names.push(k.str().to_string());
        }
    }
    names.retain(|n| !MOVE_HOME_NAMES.contains(n));
    names.sort_by_key(|n| std::cmp::Reverse(text::len(n)));
    let alternation: Vec<String> = names.iter().map(|n| pyre::escape(n)).collect();
    Re::new(&format!(r"\b(?:{})\b", alternation.join("|")), pyre::I)
});

static SENTENCE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s+", 0));
static AN_ASK_WORD: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(?:is|are|was|were|isnt|arent|any|anything|do|does|did|can|could|where|how)\b", pyre::I));
static WHAT_ABOUT: LazyLock<Re> = LazyLock::new(|| Re::new(r"what about", pyre::I));
static THERE_HERE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:there|here|nearby|local)\b", 0));
static A_PLACE_PREPOSITION: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(?:in|near|around|round|at|by|beside|on|from|to)\s+(?:the\s+)?$", 0));
static STRONG_ABROAD: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\b(?:abroad|overseas|emigrat\w*|expat\w*|visa|residency|passport for|work permit|right to (?:live|work)|country)\b", 0)
});
static ABROAD_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:abroad|overseas|country)\b", 0));

/// `re.compile(pattern, flags)` for a pattern built from a word at run time, compiled once.
fn built(pattern: String, flags: u32) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<(String, u32), &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    cache.entry((pattern.clone(), flags)).or_insert_with(|| Box::leak(Box::new(Re::new(&pattern, flags))))
}

/// `re.search(r"\b" + re.escape(word) + r"\b", text)`.
fn says_word(word: &str, text_: &str) -> bool {
    built(format!(r"\b{}\b", pyre::escape(word)), 0).is_match(text_)
}

/// `compose.pick(options, read, said)`: `pick` is handed the reading (or `{}`) as its memory.
fn pick(options: &[String], read: Option<&Understood>, said: &str) -> String {
    let read_value = match read {
        Some(read) => Value::from_plain(&serde_json::to_value(read).unwrap_or(serde_json::Value::Null)),
        None => Value::dict(),
    };
    crate::compose::pick(options, &read_value, said)
}

/// Python: bot/compose/checking.py:76 `_a_named_country_in(text)`
/// True when the line names a country other than this one.
pub fn _a_named_country_in(text_: &str) -> bool {
    _A_NAMED_COUNTRY.is_match(text_)
}

/// Python: bot/compose/checking.py:309 `_a_time_of_day(said)`
/// The reply for a plan asked about a meal or a part of the day, or None.
pub fn _a_time_of_day(said: &str) -> Option<String> {
    if said.is_empty() {
        return None;
    }
    let meal = A_MEAL_TIME.search(said);
    let part = A_PART_OF_DAY.search(said);
    if !(meal.is_some() || part.is_some()) || !WANTS_A_PLAN.is_match(said) {
        return None;
    }
    if NOT_A_DAY_OUT_PLACE.is_match(said) {
        return None;
    }
    let mut rest = said.to_string();
    for found in [&meal, &part].into_iter().flatten() {
        rest = rest.replace(found.as_str(), " ");
    }
    if A_PLACE.is_match(&rest) || A_PLACE_THERE.is_match(&rest) {
        return None;
    }
    // A meal time is the meal pool only when the ask was about the time.
    let meal_pool = meal.is_some() && !AN_ON_ASK.is_match(said);
    Some(pick(if meal_pool { &MEAL_TIME_POOL } else { &PART_OF_DAY_POOL }, None, said))
}

/// Python: bot/compose/checking.py:543 `_the_route_ends(said)`
/// The two places in a route ask, as the person wrote them, or None.
pub fn _the_route_ends(said: &str) -> Option<(String, String)> {
    for pattern in [&*_THE_ENDS, &*_THE_ENDS_REVERSED] {
        let Some(found) = pattern.search(said) else { continue };
        let end = _a_place_name(found.name_str("end"));
        let start = _a_place_name(found.name_str("start"));
        if !end.is_empty() && !start.is_empty() && text::lower(&end) != text::lower(&start) {
            return Some((start, end));
        }
    }
    None
}

/// Python: bot/compose/checking.py:575 `_without_a_qualifier(run)`
/// The place alone, with anything said after it about how or when cut off.
pub fn _without_a_qualifier(run: &str) -> String {
    let text_ = text::strip_chars(run, " .?,'");
    for found in _A_QUALIFIER_AFTER_THE_PLACE.finditer(text_) {
        if ["in", "on", "upon"].contains(&text::lower(found.name_str("word")).as_str()) {
            let rest = text::split_ref(&text_[found.end()..]);
            if rest.first().is_some_and(|w| _A_NAME_PART.contains(&text::lower(text::strip_chars(w, ".,")))) {
                continue;
            }
        }
        return text_[..found.start()].to_string();
    }
    text_.to_string()
}

/// Python: bot/compose/checking.py:586 `_a_place_name(run)`
pub fn _a_place_name(run: &str) -> String {
    let bare = _without_a_qualifier(run);
    let mut words: Vec<&str> = text::split_ref(&bare);
    while words.last().is_some_and(|w| _NOT_THE_PLACE.contains(*w)) {
        words.pop();
    }
    while words.first().is_some_and(|w| ["the", "a", "an", "to", "from", "get", "my", "our"].contains(w)) {
        words.remove(0);
    }
    // "here" and "there" are ends the bot cannot see.
    if words.is_empty() || words.len() > 4 || words.iter().any(|w| *w == "here" || *w == "there") {
        return String::new();
    }
    words.join(" ")
}

/// Python: bot/compose/checking.py:722 `_said(text)`
pub fn _said(text_: &str) -> String {
    text::lower(text_)
}

/// Python: bot/compose/checking.py:726 `_the_line_before(memory)`
/// What the person said in the last few turns, or "" on the first turn.
pub fn _the_line_before(memory: &Memory) -> String {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(3)..].iter().map(|t| text::lower(&t.user)).collect::<Vec<_>>().join(" ")
}

/// Python: bot/compose/checking.py:742 `_the_answers_before(memory, count=4)`
/// The bot's own last few answers, newest first, or [] on the first turn.
pub fn _the_answers_before(memory: &Memory, count: i64) -> Vec<String> {
    let turns = &memory.turns;
    let len = turns.len() as i64;
    let start = if count > 0 { (len - count).max(0) } else { (-count).min(len) } as usize;
    turns[start..].iter().rev().map(|t| text::strip(&t.bot).to_string()).filter(|l| !l.is_empty()).collect()
}

/// Python: bot/compose/checking.py:756 `_the_checks_in(answer)`
/// The sentence of an answer that names the conditions, or "": an "if" sentence first, wherever
/// it comes, else the first sentence that names checks.
pub fn _the_checks_in(answer: &str) -> String {
    let mut found: Vec<String> = Vec::new();
    for sentence in SENTENCE_END.split(answer) {
        let sentence = text::strip(text::strip_chars(text::strip(&sentence), "\"'\u{201c}\u{201d}")).to_string();
        if sentence.is_empty() || text::len(&sentence) > 320 {
            continue;
        }
        if NOT_A_CHECK_SENTENCE.is_match(&sentence) {
            continue;
        }
        if A_CHECK_SENTENCE.is_match(&sentence) {
            found.push(sentence);
        }
    }
    if let Some(sentence) = found.iter().find(|s| A_CONDITION.is_match(s)) {
        return sentence.clone();
    }
    found.into_iter().next().unwrap_or_default()
}

/// Python: bot/compose/checking.py:783 `_about_a_check(said, understood, memory)`
/// The checks the bot has named, handed back, or None. Nothing is added to the list.
pub fn _about_a_check(said: &str, understood: Option<&Understood>, memory: &Memory) -> Option<String> {
    if !WANTS_A_CHECK.is_match(said) || CHECK_OF_SOMETHING_ELSE.is_match(said) {
        return None;
    }
    for answer in _the_answers_before(memory, 4) {
        let mut checks = _the_checks_in(&answer);
        if checks.is_empty() {
            continue;
        }
        if !checks.ends_with(['.', '!', '?']) {
            checks.push('.');
        }
        return Some(pick(&CHECKS_POOL, understood, said).replace("{checks}", &checks));
    }
    None
}

/// Python: bot/compose/checking.py:804 `_named_service(said)`
/// The service a line names, as (the word in the line, the name on screen), or ("", ""). The
/// longest name wins.
pub fn _named_service(said: &str) -> (String, String) {
    let mut best: Option<&(String, String)> = None;
    for row in SERVICES.iter() {
        if text::len(&row.0) > best.map_or(0, |b| text::len(&b.0)) && says_word(&row.0, said) {
            best = Some(row);
        }
    }
    best.cloned().unwrap_or_default()
}

/// Python: bot/compose/checking.py:817 `_about_a_service(said, understood)`
pub fn _about_a_service(said: &str, understood: Option<&Understood>) -> Option<String> {
    let (word, service) = _named_service(said);
    if word.is_empty() || NOT_AVAILABILITY.is_match(said) {
        return None;
    }
    // The service has to be named in a frame that makes it a platform, and the line has to ask
    // something: "who is the prime minister" names a prime and "why is the sky blue" a sky.
    let w = pyre::escape(&word);
    let platform = built(
        format!(
            r"\b(?:on|from|at|via|in)\s+(?:the\s+)?{w}\b|\b{w}(?:'s|s')\b|\b{w}\s+(?:has|have|got|shows|show|streams|stream|adds)\b|\b(?:does|do|did|is|are|has|have)\s+(?:the\s+)?{w}\b"
        ),
        pyre::I,
    );
    if !platform.is_match(said) {
        return None;
    }
    if !AN_ASK_WORD.is_match(said) {
        return None;
    }
    Some(pick(&SERVICES_POOL, understood, said).replace("{service}", &service))
}

/// Python: bot/compose/checking.py:838 `_about_a_shop(said)`
/// The shop a line asks about, in the form the reply uses, or "". The name has to be in a frame
/// that makes it a shop.
pub fn _about_a_shop(said: &str) -> String {
    let mut best: Option<&(String, String)> = None;
    for row in SHOPS.iter() {
        let w = pyre::escape(&row.0);
        let frame = built(format!(r"(?:\b(?:at|in|from|to|does|do|did|is|are|asda's|with)\s+(?:the\s+)?{w}\b)|(?:\b{w}(?:'s|s')\b)"), 0);
        if frame.is_match(said) && text::len(&row.0) > best.map_or(0, |b| text::len(&b.0)) {
            best = Some(row);
        }
    }
    match best {
        Some((_, shown)) if SHOP_ASK.is_match(said) => shown.clone(),
        _ => String::new(),
    }
}

/// Python: bot/compose/checking.py:857 `_the_mode_named(mode)`
/// The service noun as the pools need it: a modifier before "times", so always singular.
pub fn _the_mode_named(mode: &str) -> String {
    let low = text::lower(mode);
    let single = TRAVEL.iter().find(|(word, _, _)| *word == low).map(|(_, single, _)| single.clone()).unwrap_or_default();
    if single.is_empty() { mode.to_string() } else { single }
}

/// Python: bot/compose/checking.py:870 `_about_a_journey(said)`
/// "route" for a from-to ask, the mode for a timetable ask, or None.
pub fn _about_a_journey(said: &str) -> Option<String> {
    if GET_FROM_TO.is_match(said) && !NOT_A_DESTINATION.is_match(said) && !TRAVEL.iter().any(|(word, _, _)| says_word(word, said)) {
        return Some("route".to_string());
    }
    for (word, single, _plural) in TRAVEL.iter() {
        // The mode is a noun only when something in front of it makes it one.
        let noun = built(
            format!(
                r"\b(?:the|a|an|my|your|our|his|her|their|this|that|these|those|last|next|first|any|every|no|by|take the|catch the|get the)\s+{}\b",
                pyre::escape(word)
            ),
            0,
        );
        if !noun.is_match(said) {
            continue;
        }
        if NOT_A_JOURNEY.is_match(said) || !WANTS_A_TIME.is_match(said) {
            return None;
        }
        // The word that goes in front of "times" is the singular, whatever the person typed.
        let mode = if word == "tube" || word == "metro" { "Tube".to_string() } else { single.clone() };
        return Some(mode);
    }
    None
}

/// Python: bot/compose/checking.py:963 `_a_country_not_home(text)`
/// The first country in the line that is not this one, or "".
pub fn _a_country_not_home(text_: &str) -> String {
    NOT_HOME_COUNTRIES.iter().find(|name| says_word(name, text_)).cloned().unwrap_or_default()
}

/// Python: bot/compose/checking.py:975 `_about_the_subject_named(said, understood, before="")`
/// The limit and the place that knows, for a follow-up that names only a subject.
pub fn _about_the_subject_named(said: &str, understood: Option<&Understood>, before: &str) -> Option<String> {
    let lead = ONLY_A_SUBJECT.match_(said)?;
    let rest = text::strip_chars(&said[lead.end()..], " ?.!,").to_string();
    if rest.is_empty() || text::split_ref(&rest).len() > *A_FEW_WORDS || A_QUESTION_OF_ITS_OWN.is_match(said) {
        return None;
    }
    let opener = text::strip(&said[lead.start()..lead.end()]);
    if (opener.starts_with("and") || opener.starts_with("but") || opener.starts_with("also")) && !WHAT_ABOUT.is_match(&said[..lead.end()]) {
        // "and the food" is this shape; "and the food was cold" is a report about dinner.
        if HAS_A_VERB.is_match(said) {
            return None;
        }
    }
    let words_in_line: Vec<String> = text::split_ref(&rest).iter().map(|w| text::strip_chars(w, "?,.!'").to_string()).collect();
    for (kind, words) in SUBJECT_KINDS.iter() {
        let Some(named) = words.search(&rest) else { continue };
        let mut left = words_in_line.clone();
        for word in text::split_ref(&text::lower(named.as_str())) {
            if let Some(at) = left.iter().position(|w| w == word) {
                left.remove(at);
            }
        }
        if left.iter().any(|w| !w.is_empty() && !POINTING_WORDS.contains(w)) {
            // Something else is in the line, so the subject is not the whole of it.
            continue;
        }
        if kind == "council" {
            // A council is the wrong body in another country.
            if ABROAD.is_match(before) || !_a_country_not_home(before).is_empty() || !_a_country_not_home(said).is_empty() {
                return None;
            }
            return Some(pick(&COUNCIL_POOL, understood, said));
        }
        if kind == "journey" {
            let word = text::lower(named.as_str());
            let mode = _SUBJECT_MODE.get(&word).cloned().unwrap_or(word);
            return Some(pick(&JOURNEY_POOL, understood, said).replace("{mode}", &_the_mode_named(&mode)));
        }
        let thing = _SUBJECT_LABEL.get(&text::lower(named.as_str())).filter(|t| !t.is_empty())?;
        // A pick is a pick somewhere: "what about the food" on its own is left to the floor.
        if !(A_PLACE.is_match(&rest) || THERE_HERE.is_match(&rest) || A_PLACE.is_match(before) || THERE_HERE.is_match(before)) {
            return None;
        }
        return Some(pick(&PICK_POOL, understood, said).replace("{thing}", thing));
    }
    None
}

/// Python: bot/compose/checking.py:1032 `_about_a_pick(said, before="")`
/// The kind of place a line asks to be told about, or None: an ask word just before the word, and
/// the word not the object of a preposition of place.
pub fn _about_a_pick(said: &str, before: &str) -> Option<String> {
    if !A_PLACE.is_match(said) && !A_PLACE.is_match(before) {
        return None;
    }
    for (words, thing) in PICKS.iter() {
        for word in words {
            for found in built(format!(r"\b{}\b", pyre::escape(word)), 0).finditer(said) {
                if PICK_NOT.get(&words[0]).is_some_and(|blocked| blocked.is_match(said)) {
                    break;
                }
                let start = found.start();
                let from = text::char_index(said, start).saturating_sub(26);
                let lead = &said[text::byte_at(said, from)..start];
                if A_PLACE_PREPOSITION.is_match(lead) {
                    continue;
                }
                if !WANTS_A_PICK.is_match(lead) {
                    continue;
                }
                return Some(thing.clone());
            }
        }
    }
    None
}

/// `str(value)` for a value kept in the user store.
fn value_str(value: &Value) -> String {
    match value {
        Value::Str(s) => s.to_string(),
        Value::None => "None".to_string(),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        Value::Int(n) => n.to_string(),
        Value::Float(x) => text::float_repr(*x),
        other => format!("{other:?}"),
    }
}

/// Python: bot/compose/checking.py:1058 `_the_home_town(memory)`
/// The town the person's own words already put them in, lowercased, or (). A country is not a
/// town (bar the four nations), and a move filed as a plan has not happened yet.
pub fn _the_home_town(memory: &Memory) -> Vec<String> {
    let Some(user) = memory.get("user").filter(|u| u.is_dict()) else { return Vec::new() };
    if !user.get("home").is_some_and(Value::truthy) {
        return Vec::new();
    }
    let plans: Vec<String> = match user.get("plans") {
        Some(Value::Str(s)) => vec![s.to_string()],
        Some(v @ (Value::List(_) | Value::Tuple(_))) => v.items().iter().map(value_str).collect(),
        _ => Vec::new(),
    };
    if plans.iter().any(|plan| GOING_AWAY.is_match(plan) || MOVING.is_match(plan)) {
        return Vec::new();
    }
    let mut found = Vec::new();
    for key in ["city", "town", "village", "hometown", "home_city"] {
        if let Some(Value::Str(value)) = user.get(key) {
            found.push(text::lower(text::strip(value)));
        }
    }
    found.into_iter().filter(|w| !w.is_empty() && (HOME_NATIONS.contains(w) || !COUNTRIES.contains(w))).collect()
}

/// Python: bot/compose/checking.py:1096 `_the_lines_of_this_question(said, memory)`
/// The last few turns and this one, oldest first, as the person said them.
pub fn _the_lines_of_this_question(said: &str, memory: &Memory) -> Vec<String> {
    let turns = &memory.turns;
    let mut lines: Vec<String> = turns[turns.len().saturating_sub(3)..].iter().map(|t| text::lower(&t.user)).collect();
    lines.push(said.to_string());
    lines
}

/// Python: bot/compose/checking.py:1110 `_the_move_was_to_their_own_town(said, memory)`
/// True when the newest thing said about moving is the move that brought them where they are.
pub fn _the_move_was_to_their_own_town(said: &str, memory: &Memory) -> bool {
    let places = _the_home_town(memory);
    if places.is_empty() {
        return false;
    }
    for line in _the_lines_of_this_question(said, memory).iter().rev() {
        if let Some(found) = MOVING.search(line) {
            let destination = text::slice(&line[found.end()..], 0, Some(30));
            return places
                .iter()
                .any(|place| built(format!(r"(?:^|[^a-z]){}(?:[^a-z]|$)", pyre::escape(place)), 0).is_match(&destination));
        }
        if GOING_AWAY.is_match(line) {
            return false;
        }
    }
    false
}

/// Python: bot/compose/checking.py:1140 `_a_day_out(said, before="")`
/// True when the line asks what to do or see in the place the thread is already about.
pub fn _a_day_out(said: &str, before: &str) -> bool {
    if said.is_empty() || !WANTS_A_DAY_OUT.is_match(said) || DAY_OUT_NOT.is_match(said) {
        return false;
    }
    if ASKS_WHAT_TO_DO.is_match(said) && A_THING_IN_HAND.is_match(said) {
        return false;
    }
    if A_PLACE.is_match(said) && !NOT_A_DAY_OUT_PLACE.is_match(said) {
        return true;
    }
    if !(A_PLACE_THERE.is_match(said) || A_BARE_DAY_OUT.is_match(said)) {
        return false;
    }
    // no name on this line: "there" and a bare "what should we see" need the trip behind them
    A_TRIP_BEFORE.is_match(before) && !NOT_A_DAY_OUT_PLACE.is_match(before)
}

/// Python: bot/compose/checking.py:1162 `_about_the_rules(said, understood, before="", memory=None)`
/// The paperwork for a move abroad, or the council's rules, or None.
pub fn _about_the_rules(said: &str, understood: Option<&Understood>, before: &str, memory: &Memory) -> Option<String> {
    if !WANTS_THE_RULES.is_match(said) {
        return None;
    }
    let both = format!("{said} {before}");
    let strong = STRONG_ABROAD.is_match(said);
    if ABROAD.is_match(&both)
        && (strong || _a_named_country_in(&both))
        && !_the_move_was_to_their_own_town(said, memory)
        && !NOT_THE_PAPERWORK.is_match(said)
    {
        // The thing being moved is not a phone, an account or a house.
        if (strong || !A_THING_NOT_A_PLACE.is_match(said)) && !(MOVING_HOUSE.is_match(&both) && !ABROAD_WORD.is_match(&both)) {
            return Some(pick(&ABROAD_POOL, understood, said));
        }
    }
    // The council has to be named on this line.
    if COUNCIL_WORD.is_match(said) || PARKING_RULE.is_match(said) {
        return Some(pick(&COUNCIL_POOL, understood, said));
    }
    None
}

/// Python: bot/compose/checking.py:1234 `_a_form(said, understood)`
/// Where a form comes from, or where it goes back to, or None.
pub fn _a_form(said: &str, understood: Option<&Understood>) -> Option<String> {
    if NOT_A_FORM.is_match(said) {
        return None;
    }
    if SEND_A_FORM.is_match(said) {
        return Some(pick(&FORM_TO_POOL, understood, said));
    }
    if GET_A_FORM.is_match(said) {
        return Some(pick(&FORM_FROM_POOL, understood, said));
    }
    None
}

/// Python: bot/compose/checking.py:1245 `_a_choice(said, understood)`
/// The deciding factor for a choice between two things, or None.
pub fn _a_choice(said: &str, understood: Option<&Understood>) -> Option<String> {
    if !WANTS_A_CHOICE.is_match(said) || NOT_A_CHOICE.is_match(said) {
        return None;
    }
    Some(pick(&CHOICE_POOL, understood, said))
}

/// Python: bot/compose/checking.py:1257 `_a_shortlist(said, understood)`
/// What a shortlist has to come from, or None: the bot cannot draw one out of nothing.
pub fn _a_shortlist(said: &str, understood: Option<&Understood>) -> Option<String> {
    if !WANTS_A_SHORTLIST.is_match(said) || NOT_A_SHORTLIST.is_match(said) {
        return None;
    }
    Some(pick(&SHORTLIST_POOL, understood, said))
}

/// Python: bot/compose/checking.py:1264 `_about_the_sky(said, understood)`
/// The weather a line asks about, as a reply naming where it is read, or None.
pub fn _about_the_sky(said: &str, understood: Option<&Understood>) -> Option<String> {
    if !A_SKY_ASK.is_match(said) {
        return None;
    }
    if A_SETTING.is_match(said) {
        return None;
    }
    Some(pick(&WEATHER_POOL, understood, said))
}

/// Python: bot/compose/checking.py:1340 `the_checks_already_named(text, memory=None)`
/// The checks the bot named earlier, handed back, or None.
pub fn the_checks_already_named(text_: &str, memory: &Memory) -> Option<String> {
    let said = _said(text_);
    if text::strip(&said).is_empty() {
        return None;
    }
    _about_a_check(&said, memory.understood.as_ref(), memory)
}

/// `str.replace` of a pick with a non-empty answer, or None (Python's `x or None` chain).
fn some(s: Option<String>) -> Option<String> {
    s.filter(|s| !s.is_empty())
}

/// Python: bot/compose/checking.py:1360 `answer_for(text, memory=None)`
/// The honest limit and the place that knows, or None when the line is not one of these.
pub fn answer_for(text_: &str, memory: &Memory) -> Option<String> {
    let said = _said(text_);
    if text::strip(&said).is_empty() {
        return None;
    }
    let read = memory.understood.as_ref();
    if let Some(service) = some(_about_a_service(&said, read)) {
        return Some(service);
    }
    if let Some(sky) = some(_about_the_sky(&said, read)) {
        return Some(sky);
    }
    let mode = _about_a_journey(&said);
    if mode.as_deref() == Some("route") {
        if let Some((start, end)) = _the_route_ends(&said) {
            return Some(pick(&ROUTE_ENDS_POOL, read, &said).replace("{start}", &start).replace("{end}", &end));
        }
        return Some(pick(&ROUTE_POOL, read, &said));
    }
    if let Some(mode) = mode.filter(|m| !m.is_empty()) {
        return Some(pick(&JOURNEY_POOL, read, &said).replace("{mode}", &_the_mode_named(&mode)));
    }
    let shop = _about_a_shop(&said);
    if !shop.is_empty() {
        return Some(pick(&SHOP_POOL, read, &said).replace("{shop}", &shop));
    }
    let before = _the_line_before(memory);
    // A time, not a place: asked here so the day-out reading below cannot take it.
    if let Some(time_of_day) = some(_a_time_of_day(&said)) {
        return Some(time_of_day);
    }
    if _a_day_out(&said, &before) {
        return Some(pick(&THINGS_TO_DO_POOL, read, &said));
    }
    if let Some(thing) = some(_about_a_pick(&said, &before)) {
        return Some(pick(&PICK_POOL, read, &said).replace("{thing}", &thing));
    }
    if let Some(subject) = some(_about_the_subject_named(&said, read, &before)) {
        return Some(subject);
    }
    if let Some(checks) = some(_about_a_check(&said, read, memory)) {
        return Some(checks);
    }
    if let Some(rules) = some(_about_the_rules(&said, read, &before, memory)) {
        return Some(rules);
    }
    // Lowest of all, because each of these is read for the shape of the question.
    some(_a_choice(&said, read)).or_else(|| some(_a_shortlist(&said, read))).or_else(|| some(_a_form(&said, read)))
}
