//! Port of PyBot's `bot/knowledge/geography.py`: the everyday geography facts (records, peaks,
//! continents, borders, the EU, the parts of the UK, river lengths) and the reading of whether a
//! line carries something across a border.
#![allow(non_snake_case)] // closures are named `<factory>__<inner>`

use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::ReplyFn;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

const M: &str = "bot.knowledge.geography.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{M}{name}"))
}
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}

macro_rules! strs {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> = LazyLock::new(|| g(stringify!($name)).strs());)*
    };
}
macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> = LazyLock::new(|| rx(stringify!($name)));)*
    };
}

strs!(
    _PLACES,
    _BRITAIN,
    _EVERYDAY_PLACE,
    _NOT_A_PLACE,
    _RECORD_SMALLEST,
    _RECORD_POPULOUS,
    _LEAST,
    TAKES_THE,
    EU_MEMBERS,
    _THE_FOUR,
    _COUNTRY_PHRASES,
);
regexes!(
    _CROSSING_WORD,
    _DIRECTION,
    _ARTICLE,
    _DESTINATION,
    _ORDINAL,
    _RECORD_LIST,
    _NATION_NAMED,
    _RIVER_LENGTH_ASK,
);

static _RIVER_NAME_NEEDS_RIVER: LazyLock<std::collections::HashSet<String>> =
    LazyLock::new(|| g("_RIVER_NAME_NEEDS_RIVER").str_set());

/// `(phrase, value)` pairs of a `{str: str}` dict global, in dict order.
fn str_pairs(name: &str) -> Vec<(String, String)> {
    g(name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
}

static HIGHEST_IN: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("HIGHEST_IN"));
static _CONTINENT_COUNTS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    g("_CONTINENT_COUNTS").items().iter().map(|r| (r.items()[0].str().to_string(), r.items()[1].str().to_string())).collect()
});
static _CONTINENT_WORDS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    g("_CONTINENT_WORDS").items().iter().map(|r| (r.items()[0].str().to_string(), r.items()[1].str().to_string())).collect()
});
static _CONTINENTS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("_CONTINENTS"));
static _CONTINENT_NOTES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("_CONTINENT_NOTES"));
static _CONTINENT_NOTES_BY_COUNTRY: LazyLock<Vec<(String, String)>> =
    LazyLock::new(|| str_pairs("_CONTINENT_NOTES_BY_COUNTRY"));
static _COUNTRY_ALIASES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("_COUNTRY_ALIASES"));
static _NOT_A_COUNTRY: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("_NOT_A_COUNTRY"));
static _NOT_IN_EU: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("_NOT_IN_EU"));
static _NATION_SAID: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("_NATION_SAID"));
static _RIVER_LENGTHS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    g("_RIVER_LENGTHS").items().iter().map(|r| (r.items()[0].str().to_string(), r.items()[1].str().to_string())).collect()
});

/// One `COUNTRIES` row: the continent, the land neighbours, the note.
pub struct Country {
    pub continent: String,
    pub borders: Vec<String>,
    pub note: String,
}

/// `COUNTRIES`, in dict order.
pub static COUNTRIES: LazyLock<Vec<(String, Country)>> = LazyLock::new(|| {
    g("COUNTRIES")
        .pairs()
        .iter()
        .map(|(k, v)| {
            let r = v.items();
            (
                k.str().to_string(),
                Country { continent: r[0].str().to_string(), borders: r[1].strs(), note: r[2].str().to_string() },
            )
        })
        .collect()
});

fn country(name: &str) -> Option<&'static Country> {
    COUNTRIES.iter().find(|(k, _)| k == name).map(|(_, c)| c)
}

fn lookup<'a>(pairs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

/// `re.search(r"\b" + re.escape(word) + r"\b", text)`.
fn word_in(word: &str, text: &str) -> bool {
    sh::re_search(&format!(r"\b{}\b", sh::escape(word)), text, 0)
}

/// `re.search(r"(?<![a-z])" + re.escape(phrase) + r"(?![a-z])", text)`.
fn phrase_in(phrase: &str, text: &str) -> bool {
    sh::re_search(&format!(r"(?<![a-z]){}(?![a-z])", sh::escape(phrase)), text, 0)
}

/// `_world(reply)`'s closure: the world record, unless the line asks about a named region.
pub fn _world__answer(f: &FnRef, _memory: &Memory, understood: &Understood) -> Option<String> {
    let reply = f.cell("reply").map(|v| v.str().to_string());
    let clean = understood.clean.as_str();
    if let Some(named) = f.cell("place").and_then(Value::as_re).and_then(|p| p.search(clean)) {
        let word = named.str(1);
        let whole = f.cell("whole").map(Value::strs).unwrap_or_default();
        let unit = f.cell("unit").map(Value::strs).unwrap_or_default();
        if !whole.iter().any(|w| w == word) && !unit.iter().any(|w| w == word) {
            return None;
        }
    }
    reply
}

/// The same height in feet, so an answer works in either unit.
pub fn _feet(metres: i64) -> i64 {
    text::round_half_even(metres as f64 / 0.3048)
}

/// The line's clean words lower-cased and padded with a space each side.
pub fn _said(understood: &Understood) -> String {
    format!(" {} ", understood.clean.to_lowercase())
}

/// The peak row the line names (display name, metres, note), the longest name first.
pub fn _peak_named(understood: &Understood) -> Option<(String, i64, String)> {
    let text = _said(understood);
    let (mut found, mut best): (Option<&Value>, &str) = (None, "");
    for (name, row) in g("PEAKS").pairs() {
        let name = name.str();
        if text::len(name) > text::len(best) && word_in(name, &text) {
            found = Some(row);
            best = name;
        }
    }
    let r = found?.items();
    Some((r[0].str().to_string(), r[1].as_int().unwrap_or(0), r[2].str().to_string()))
}

/// The height of the peak the line names.
pub fn _peak(_memory: &Memory, understood: &Understood) -> Option<String> {
    let (name, metres, note) = _peak_named(understood)?;
    Some(format!(
        "{} is {} metres ({} feet) high: {}",
        name,
        text::thousands(metres),
        text::thousands(_feet(metres)),
        format!("{}.", note.trim_end_matches('.'))
    ))
}

/// The highest point of the place the line names.
pub fn _highest_in_place(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text = _said(understood);
    let mut places: Vec<&(String, String)> = HIGHEST_IN.iter().collect();
    places.sort_by(|a, b| text::len(&b.0).cmp(&text::len(&a.0)));
    places.into_iter().find(|(place, _)| word_in(place, &text)).map(|(_, said)| format!("{said}."))
}

/// The continent's own count of countries, unless the line asks about a subset of them.
pub fn _continent_count(_memory: &Memory, understood: &Understood) -> Option<String> {
    static TRAILING_HAVE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+(?:have|has)\s*[?.!]*$", 0));
    static SUBSET: LazyLock<Re> = LazyLock::new(|| {
        Re::new(r"\b(?:have|has|with|without|allow|allows|legal|banned?|bans|speak|speaks|use|uses|need|needs)\b", 0)
    });
    let text = understood.text.to_lowercase();
    let trimmed = TRAILING_HAVE.sub("", &text);
    if SUBSET.is_match(&trimmed) {
        return None;
    }
    _CONTINENT_COUNTS.iter().find(|(name, _)| word_in(name, &text)).map(|(_, count)| count.clone())
}

/// The place words after a direction, without the article in front of them.
pub fn _bare_place(phrase: &str) -> String {
    let mut phrase = text::strip_chars(text::strip(phrase), ".,!?;:").to_string();
    loop {
        let shorter = _ARTICLE.subn("", &phrase, 1);
        if shorter == phrase {
            return phrase;
        }
        phrase = shorter;
    }
}

/// The place a line carries something towards, in its own words, or None.
pub fn directed_place(line: &str) -> Option<String> {
    for m in _DIRECTION.finditer(line) {
        let place = _bare_place(m.str(1));
        if !place.is_empty() && !_NOT_A_PLACE.iter().any(|w| *w == place) {
            return Some(place);
        }
    }
    None
}

/// True when the place words name a room, a building or a do rather than a country.
pub fn _is_everyday(place: &str) -> bool {
    let place = _bare_place(place);
    _EVERYDAY_PLACE
        .iter()
        .any(|name| place == *name || place.starts_with(&format!("{name} ")) || place.ends_with(&format!(" {name}")))
}

/// The place the line travels towards, if it is one and not an everyday room or do.
pub fn _is_a_place(line: &str) -> Option<String> {
    let place = directed_place(line)?;
    if _is_everyday(&place) {
        return None;
    }
    Some(place)
}

/// True when the line carries something across a border rather than to a do.
pub fn crosses_a_border(line: &str) -> bool {
    let line = line.to_lowercase();
    if _CROSSING_WORD.is_match(&line) {
        return true;
    }
    _is_a_place(&line).is_some()
}

/// The place a line travels into or to, in its own words, or None.
pub fn destination_place(line: &str) -> Option<String> {
    let line = line.to_lowercase();
    for m in _DESTINATION.finditer(&line) {
        let place = _bare_place(m.str(1));
        if !place.is_empty() && !_NOT_A_PLACE.iter().any(|w| *w == place) {
            return Some(place);
        }
    }
    None
}

/// True when the place travelled to is named and is not Britain.
pub fn carries_it_out_of_britain(line: &str) -> bool {
    let Some(place) = destination_place(line) else { return false };
    if _is_everyday(&place) {
        return false;
    }
    !_BRITAIN.iter().any(|w| *w == place)
}

/// Which place an ordinal names: 2 for "second" and "2nd", or None.
pub fn _ordinal_position(clean: &str) -> Option<usize> {
    g("_ORDINAL_POS")
        .pairs()
        .iter()
        .find(|(word, _)| word_in(word.str(), clean))
        .and_then(|(_, pos)| pos.as_int())
        .map(|p| p as usize)
}

/// Whether any of the words is in the line as a whole word.
pub fn _has<S: AsRef<str>>(clean: &str, words: &[S]) -> bool {
    words.iter().any(|w| word_in(w.as_ref(), clean))
}

/// The region a line names, adjectives included, or None.
pub fn _region_name(clean: &str) -> Option<String> {
    for (word, region) in _CONTINENT_WORDS.iter() {
        if word_in(word, clean) {
            return Some(region.clone());
        }
    }
    _PLACES.iter().find(|name| word_in(name, clean)).cloned()
}

/// The row of a region table (`_REGION_COUNTRY` by default) for the region the line names.
pub fn _region_row(clean: &str, table: Option<&'static Value>) -> Option<&'static Value> {
    let table = table.unwrap_or_else(|| g("_REGION_COUNTRY"));
    if let Some(name) = _region_name(clean) {
        if let Some(row) = table.get(&name) {
            return Some(row);
        }
    }
    let mut keys: Vec<&str> = table.pairs().iter().map(|(k, _)| k.str()).filter(|k| !_PLACES.iter().any(|p| p == k)).collect();
    keys.sort_by(|a, b| text::len(b).cmp(&text::len(a)));
    keys.into_iter().find(|key| word_in(key, clean)).and_then(|key| table.get(key))
}

/// A region row's string value (None for a missing key or a value that is not a string).
fn row_str(row: &Value, key: &str) -> Option<String> {
    row.get(key).and_then(Value::as_str).map(str::to_string)
}

/// `country_extreme(kind, world_reply)`'s closure: the region's own largest or smallest country,
/// or the world record on an unqualified line.
pub fn country_extreme__answer(f: &FnRef, _memory: &Memory, understood: &Understood) -> Option<String> {
    let kind = f.cell("kind").map(|v| v.str()).unwrap_or("");
    let world_reply = f.cell("world_reply").and_then(Value::as_str).map(str::to_string);
    let clean = understood.clean.to_lowercase();
    let Some(row) = _region_row(&clean, None) else { return world_reply };
    let look = if _has(&clean, &_RECORD_SMALLEST) { "small" } else { kind };
    let ranked = row_str(row, if look == "small" { "bottom" } else { "top" });
    let position = _ordinal_position(&clean);
    let people = row.get("people").map(Value::strs).unwrap_or_default();
    if let Some(position) = position {
        if _has(&clean, &_RECORD_POPULOUS) && people.len() >= position {
            let where_ = text::title(&_region_name(&clean).unwrap_or_default());
            let pos_word = g("_POS_WORD").pairs().iter().find(|(k, _)| k.as_int() == Some(position as i64))?.1.str();
            // Python indexes people[2]; a shorter row raises, and the reply declines
            let third = people.get(2)?;
            return Some(format!(
                "{} has the {} most people in {}: {} is first, then {} and {}.",
                people[position - 1], pos_word, where_, people[0], people[1], third
            ));
        }
    }
    let ranked = ranked.filter(|s| !s.is_empty());
    if _ORDINAL.is_match(&clean) {
        return ranked.or(world_reply);
    }
    if _RECORD_LIST.is_match(&clean) {
        return ranked.or(world_reply);
    }
    row_str(row, look).filter(|s| !s.is_empty()).or(world_reply)
}

/// Africa's biggest country, Europe's smallest, Asia's most people: the record inside the
/// continent the line names (None for the plain biggest and smallest, which travel answers).
pub fn _continent_record(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text = sh::clean_or_text(understood).to_lowercase();
    let row = _region_row(&text, None)?;
    let has = |words: &[String]| _has(&text, words);
    if has(&_LEAST) || (has(&_RECORD_SMALLEST) && has(&_RECORD_POPULOUS)) {
        return row_str(row, "least");
    }
    if has(&_RECORD_POPULOUS) {
        return row_str(row, "populous");
    }
    None
}

/// `_continent_record`'s inner `has(words)`: any of the words in the line as a whole word.
pub fn _continent_record__has(text: &str, words: &[String]) -> bool {
    _has(text, words)
}

/// `region_extreme(kind, world_reply)`'s closure: the region's own river, mountain or desert;
/// the world answer (through `_world`) only on an unqualified line.
pub fn region_extreme__answer(f: &FnRef, memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    match _region_row(&clean, Some(g("_REGION_EXTREME"))) {
        None => {
            let world = f.cell("world").and_then(Value::as_fn)?;
            _world__answer(world, memory, understood)
        }
        // A known region with no such fact (Europe's desert) passes rather than borrows the world's.
        Some(row) => row_str(row, f.cell("kind").map(|v| v.str()).unwrap_or("")),
    }
}

/// A country key as it is written in a sentence: Czech Republic, the US, Ivory Coast.
pub fn _display_country(name: &str) -> String {
    match name {
        "united states" => return "the US".to_string(),
        "united kingdom" => return "the UK".to_string(),
        "united arab emirates" => return "the UAE".to_string(),
        _ => {}
    }
    if let Some(shown) = lookup(&_NOT_A_COUNTRY, name) {
        return shown.to_string();
    }
    let mut words: Vec<String> = Vec::new();
    for word in text::split_ref(name) {
        if ["and", "of", "the"].contains(&word) && !words.is_empty() {
            words.push(word.to_string());
        } else if word.contains('-') {
            words.push(word.split('-').map(text::capitalize).collect::<Vec<_>>().join("-"));
        } else {
            words.push(text::capitalize(word));
        }
    }
    words.join(" ")
}

/// The name as it reads in a sentence: "France", "the Netherlands", "the UK".
pub fn _the_country(name: &str) -> String {
    let shown = _display_country(name);
    if shown.starts_with("the ") || !TAKES_THE.iter().any(|w| w == name) {
        return shown;
    }
    format!("the {shown}")
}

/// Whether the line is a question at all, which is what these facts answer.
pub fn _asked(understood: &Understood) -> bool {
    understood.kind == "question"
}

/// The reply with its first letter a capital, for the names that start with "the".
pub fn _sentence(answer: &str) -> String {
    text::upper_first(answer)
}

/// One compiled pattern per country phrase, longest phrase first.
static COUNTRY_PATTERNS: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    _COUNTRY_PHRASES
        .iter()
        .map(|p| (p.clone(), Re::new(&format!(r"(?<![a-z]){}(?![a-z])", sh::escape(p)), 0)))
        .collect()
});

/// The countries a line names, in the order it says them, as keys into COUNTRIES.
pub fn _countries_said(clean: &str) -> Vec<String> {
    let mut found: Vec<(usize, String)> = Vec::new();
    let mut taken: Vec<(usize, usize)> = Vec::new();
    for (phrase, pattern) in COUNTRY_PATTERNS.iter() {
        for m in pattern.finditer(clean) {
            let (start, end) = (m.start(), m.end());
            if taken.iter().any(|&(before, after)| before < end && start < after) {
                continue;
            }
            taken.push((start, end));
            found.push((start, lookup(&_COUNTRY_ALIASES, phrase).unwrap_or(phrase).to_string()));
        }
    }
    found.sort();
    let mut said: Vec<String> = Vec::new();
    for (_, name) in found {
        if !said.contains(&name) {
            said.push(name);
        }
    }
    said
}

/// A list said the way a person says it: one, two and three.
pub fn _join_names<S: AsRef<str>>(names: &[S]) -> String {
    let names: Vec<String> = names.iter().map(|n| _the_country(n.as_ref())).collect();
    if names.len() == 1 {
        return names[0].clone();
    }
    format!("{} and {}", names[..names.len() - 1].join(", "), names[names.len() - 1])
}

/// `lambda row: len(row[0])`: the sort key that puts the longest continent phrase first.
pub fn _continent_said__lambda_1237(row: &(String, String)) -> usize {
    text::len(&row.0)
}

/// The continent a line names, as the name this file uses, or None.
pub fn _continent_said(clean: &str) -> Option<String> {
    let mut rows: Vec<&(String, String)> = _CONTINENTS.iter().collect();
    rows.sort_by_key(|row| std::cmp::Reverse(_continent_said__lambda_1237(row)));
    rows.into_iter().find(|(phrase, _)| phrase_in(phrase, clean)).map(|(_, name)| name.clone())
}

/// Which countries touch the one the line names, and whether two of them touch each other.
pub fn _borders(_memory: &Memory, understood: &Understood) -> Option<String> {
    static HOW_MANY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhow many\b", 0));
    if !_asked(understood) {
        return None;
    }
    let clean = understood.clean.to_lowercase();
    let said = _countries_said(&clean);
    let first = said.first()?;
    let row = country(first)?;
    let (next_to, note) = (&row.borders, &row.note);
    if said.len() > 1 {
        let other = &said[1];
        let theirs = country(other).map(|c| c.borders.as_slice()).unwrap_or(&[]);
        if next_to.contains(other) || theirs.contains(first) {
            return Some(_sentence(&format!(
                "Yes - {} and {} share a border.",
                _the_country(first),
                _display_country(other)
            )));
        }
        return Some(_sentence(&format!(
            "No - {} and {} do not share a land border.",
            _the_country(first),
            _display_country(other)
        )));
    }
    let note = if note.is_empty() { String::new() } else { format!(" {note}") };
    if next_to.is_empty() {
        return Some(_sentence(&format!("{} has no land neighbours.{}", _the_country(first), note)));
    }
    if HOW_MANY.is_match(&clean) {
        return Some(_sentence(&format!(
            "{} has {} land neighbours: {}.",
            _the_country(first),
            next_to.len(),
            _join_names(next_to)
        )));
    }
    let answer = format!("{} borders {}.", _the_country(first), _join_names(next_to));
    Some(_sentence(&format!("{answer}{note}")))
}

/// The continent a named country is in, and whether it is in the one the line names.
pub fn _continent(_memory: &Memory, understood: &Understood) -> Option<String> {
    if !_asked(understood) {
        return None;
    }
    let clean = understood.clean.to_lowercase();
    let said = _countries_said(&clean);
    let first = said.first()?;
    let row = country(first)?;
    if row.continent.is_empty() {
        return None;
    }
    let where_ = &row.continent;
    let mut answer = format!("{} is in {}.", _the_country(first), where_);
    let asked = _continent_said(&clean);
    if let Some(asked) = asked {
        if where_.to_lowercase().contains(&asked.to_lowercase()) && asked != *where_ {
            answer = format!("Yes - {answer}");
        } else if asked != *where_ {
            answer = format!("No - {answer}");
        }
    }
    let note = lookup(&_CONTINENT_NOTES_BY_COUNTRY, first).unwrap_or("");
    let note = if note.is_empty() { String::new() } else { format!(" {note}") };
    Some(_sentence(&format!("{answer}{note}")))
}

/// The countries usually counted in a continent, from the same table the borders come from.
pub fn _continent_countries(_memory: &Memory, understood: &Understood) -> Option<String> {
    if !_asked(understood) {
        return None;
    }
    let clean = understood.clean.to_lowercase();
    let region = _continent_said(&clean)?;
    let members: Vec<&str> = COUNTRIES.iter().filter(|(_, c)| c.continent == region).map(|(k, _)| k.as_str()).collect();
    if members.is_empty() {
        return None;
    }
    Some(format!(
        "The countries usually counted in {}: {}. {}",
        region,
        _join_names(&members),
        lookup(&_CONTINENT_NOTES, &region).unwrap_or("")
    ))
}

/// Whether a named country is in the EU, or which countries are.
pub fn _in_eu(_memory: &Memory, understood: &Understood) -> Option<String> {
    static LEAVE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(leave|leaves|leaving|left|quit|quitting|brexit)\b", 0));
    if !_asked(understood) {
        return None;
    }
    let clean = understood.clean.to_lowercase();
    let said = _countries_said(&clean);
    let Some(country) = said.first() else {
        return Some(
            "Twenty-seven countries are in the EU: Austria, Belgium, Bulgaria, Croatia, Cyprus, Czech Republic, \
             Denmark, Estonia, Finland, France, Germany, Greece, Hungary, Ireland, Italy, Latvia, Lithuania, \
             Luxembourg, Malta, the Netherlands, Poland, Portugal, Romania, Slovakia, Slovenia, Spain and Sweden."
                .to_string(),
        );
    };
    let member = EU_MEMBERS.iter().any(|w| w == country);
    if LEAVE.is_match(&clean) {
        if country == "united kingdom" {
            return Some(
                "Yes - the UK left the EU on 31 January 2020, after the Brexit vote in 2016. It has been outside \
                 the single market and customs union since then."
                    .to_string(),
            );
        }
        if member {
            return Some(_sentence(&format!(
                "No - {} is still in the EU; it is one of the 27 members.",
                _the_country(country)
            )));
        }
        return Some(_sentence(&format!(
            "No - {} was never in the EU, so there was nothing to leave.",
            _the_country(country)
        )));
    }
    if member {
        return Some(_sentence(&format!("Yes - {} is one of the 27 countries in the EU.", _the_country(country))));
    }
    if let Some(reason) = lookup(&_NOT_IN_EU, country).filter(|r| !r.is_empty()) {
        return Some(_sentence(&format!("No - {} is not in the EU: {}", _the_country(country), reason)));
    }
    Some(_sentence(&format!("No - {} is not in the EU.", _the_country(country))))
}

/// The one explanation every comparison of these names comes to.
pub fn _the_nations_explained() -> String {
    "Great Britain is the island: England, Scotland and Wales. The UK is the country, which is that island plus \
     Northern Ireland. So England is in Britain and in the UK, Northern Ireland is in the UK but not in Britain, \
     and the British Isles is only geography - both islands, including the Republic of Ireland, which is a \
     separate country."
        .to_string()
}

fn nation_said(name: &str) -> &'static str {
    lookup(&_NATION_SAID, name).expect("_NATION_SAID")
}

/// The answer for "is X part of Y" when both are named, or None when the UK is the answer.
pub fn _the_pair_a_part_of_names(clean: &str) -> Option<String> {
    static PART_OF: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bpart of\b", 0));
    let split = PART_OF.search(clean)?;
    let before = _NATION_NAMED.findall_str(&clean[..split.start()]);
    let after = _NATION_NAMED.findall_str(&clean[split.end()..]);
    let (one, two) = (before.last()?.as_str(), after.first()?.as_str());
    if one == two {
        return None;
    }
    if two == "uk" || two == "united kingdom" {
        return None;
    }
    let four = |n: &str| _THE_FOUR.iter().any(|w| w == n);
    if two == "europe" {
        if four(one) || ["britain", "great britain", "uk", "united kingdom"].contains(&one) {
            return Some(format!(
                "Yes - {} is in Europe. Every part of the UK is: the UK left the European Union in 2020, but it \
                 never left the continent.",
                nation_said(one)
            ));
        }
        return Some(
            "Yes - Ireland is in Europe, and in the European Union. It is a separate country from the UK: only \
             Northern Ireland is part of the UK."
                .to_string(),
        );
    }
    if four(one) && four(two) {
        return Some(format!(
            "No - {} is not part of {}. They are both part of the UK, which is England, Scotland, Wales and \
             Northern Ireland.",
            nation_said(one),
            nation_said(two)
        ));
    }
    if two == "britain" || two == "great britain" {
        if one == "northern ireland" || one == "ireland" {
            return Some(format!(
                "No - {} is not in Great Britain: Great Britain is England, Scotland and Wales.{}",
                nation_said(one),
                if one == "northern ireland" {
                    " Northern Ireland is in the UK all the same, on the island of Ireland."
                } else {
                    " The Republic of Ireland is a separate country."
                }
            ));
        }
        return Some(format!(
            "Yes - {} is part of Great Britain, the island that also has {} on it. Great Britain and Northern \
             Ireland together are the UK.",
            nation_said(one),
            if one == "scotland" {
                "England and Wales"
            } else if one == "england" {
                "Scotland and Wales"
            } else {
                "England and Scotland"
            }
        ));
    }
    if two == "ireland" && four(one) {
        return Some(format!(
            "No - {} is not part of Ireland. Ireland is a separate country; {}",
            nation_said(one),
            if one == "northern ireland" {
                "Northern Ireland is the part of the UK on the same island."
            } else {
                "it shares its island with Northern Ireland, which is part of the UK."
            }
        ));
    }
    None
}

/// `_uk_part`'s inner `says(phrase)`: the phrase in the line, not inside a longer word.
pub fn _uk_part__says(clean: &str, phrase: &str) -> bool {
    phrase_in(phrase, clean)
}

/// Whether a part of the UK is in the UK, and whether Ireland is one of them.
pub fn _uk_part(_memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    let says = |phrase: &str| _uk_part__says(&clean, phrase);
    let comparison = says("british isles") || says("difference") || says("same") || says("versus") || says(" vs ");
    if !_asked(understood) && !comparison {
        return None;
    }
    if comparison {
        return Some(_the_nations_explained());
    }
    if let Some(pair) = _the_pair_a_part_of_names(&clean) {
        return Some(pair);
    }
    let part = if says("northern ireland") {
        "Northern Ireland".to_string()
    } else if says("ireland") {
        return Some(
            "No - Ireland is not part of the UK. The Republic of Ireland is a separate country; Northern Ireland \
             is the part of the UK on the same island. Great Britain is England, Scotland and Wales."
                .to_string(),
        );
    } else {
        ["england", "scotland", "wales"].iter().find(|n| says(n)).map(|n| text::title(n))?
    };
    if says("a country") || says("independent country") || says("sovereign") {
        if part == "Northern Ireland" {
            return Some(
                "Northern Ireland is called a province or a region rather than a country, but it is part of the \
                 UK all the same: the UK is England, Scotland, Wales and Northern Ireland."
                    .to_string(),
            );
        }
        return Some(format!(
            "Yes - {part} is a country, one of the four in the UK. It is not a sovereign state on its own: the UK \
             is the country that sits at the UN and signs treaties."
        ));
    }
    if says("independent") || says("separate country") {
        return Some(format!(
            "No - {part} is not a separate country: it is one of the four parts of the UK, along with England, \
             Scotland, Wales and Northern Ireland. The UK is the country."
        ));
    }
    Some(format!(
        "Yes - {part} is part of the UK. The UK is England, Scotland, Wales and Northern Ireland; Great Britain \
         is the island with the first three on it."
    ))
}

/// The length of the river the line names, or None when it names none of them.
pub fn _river_length(_memory: &Memory, reading: &Understood) -> Option<String> {
    static RIVER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\briver\b", 0));
    let clean = sh::text_or_clean(reading).to_lowercase();
    if !_RIVER_LENGTH_ASK.is_match(&clean) {
        return None;
    }
    for (name, said) in _RIVER_LENGTHS.iter() {
        if !word_in(name, &clean) {
            continue;
        }
        // "amazon" is a shop, "po" a post office and "darling" somebody's name
        if _RIVER_NAME_NEEDS_RIVER.contains(name) && !RIVER.is_match(&clean) {
            continue;
        }
        return Some(said.clone());
    }
    None
}

// --- registry wrappers ------------------------------------------------------------------------

fn r_world(f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _world__answer(f, m, u)
}
fn r_country_extreme(f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    country_extreme__answer(f, m, u)
}
fn r_region_extreme(f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    region_extreme__answer(f, m, u)
}
fn r_borders(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _borders(m, u)
}
fn r_continent(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _continent(m, u)
}
fn r_continent_count(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _continent_count(m, u)
}
fn r_continent_countries(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _continent_countries(m, u)
}
fn r_continent_record(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _continent_record(m, u)
}
fn r_highest_in_place(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _highest_in_place(m, u)
}
fn r_in_eu(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _in_eu(m, u)
}
fn r_peak(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _peak(m, u)
}
fn r_river_length(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _river_length(m, u)
}
fn r_uk_part(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _uk_part(m, u)
}

pub const GUARDS: &[(&str, crate::knowledge::registry::GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.geography:_borders:1244", r_borders),
    ("bot.knowledge.geography:_continent:1274", r_continent),
    ("bot.knowledge.geography:_continent_count:183", r_continent_count),
    ("bot.knowledge.geography:_continent_countries:1301", r_continent_countries),
    ("bot.knowledge.geography:_continent_record:623", r_continent_record),
    ("bot.knowledge.geography:_highest_in_place:153", r_highest_in_place),
    ("bot.knowledge.geography:_in_eu:1346", r_in_eu),
    ("bot.knowledge.geography:_peak:144", r_peak),
    ("bot.knowledge.geography:_river_length:1599", r_river_length),
    ("bot.knowledge.geography:_uk_part:1462", r_uk_part),
    ("bot.knowledge.geography:_world.<locals>.answer:117", r_world),
    ("bot.knowledge.geography:country_extreme.<locals>.answer:557", r_country_extreme),
    ("bot.knowledge.geography:region_extreme.<locals>.answer:780", r_region_extreme),
];


// Stubs appended by harness/integrate_skeleton.py: the Python functions this file does not
// define yet, with signatures from the generated contract. Port them in place (move a
// function up into the file when you port it).
pub use self::skeleton_stubs::*;
#[allow(unused_imports, unused_variables, non_snake_case, dead_code, clippy::all)]
mod skeleton_stubs {
    use super::*;
    use crate::data::{self, Value};
    use crate::knowledge::entry::Entry;
    use crate::runtime::Candidate;
    use crate::state::{Memory, Turn};
    use crate::understood::Understood;

    /// Python: bot/knowledge/geography.py:106 `_world(reply)` (never called on the trace corpus)
    pub fn _world(reply: &str) -> Value {
        crate::unported!("bot.knowledge.geography:_world")
    }

    /// Python: bot/knowledge/geography.py:551 `country_extreme(kind, world_reply)` (never called on the trace corpus)
    pub fn country_extreme(kind: &str, world_reply: &str) -> Value {
        crate::unported!("bot.knowledge.geography:country_extreme")
    }

    /// Python: bot/knowledge/geography.py:772 `region_extreme(kind, world_reply)` (never called on the trace corpus)
    pub fn region_extreme(kind: &str, world_reply: &str) -> Value {
        crate::unported!("bot.knowledge.geography:region_extreme")
    }

}

