//! Port of `bot/compose/a_place_nobody_can_see.py`: a verdict asked on a thing the bot cannot
//! see ("is benidorm nice for kids"), answered with what settles it and where to look, in place
//! of the search hedge.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_sets, bundle_str, bundle_strs, format_named};
use crate::compose::pick;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;

bundle_strs!("bot.compose.a_place_nobody_can_see"; HEDGE_OPENERS, _STOP, TRIP, WITH_CHILDREN, SAFE_, SAFE_OBJECT,
    SCHOOL, FOOD, STAY, CLASS, WALK, SIGHT, FAMILY, HOME, OTHER, SCREEN, JOB);
bundle_str!("bot.compose.a_place_nobody_can_see"; _TRIP_CRITERIA, _TRIP_WHERE, _CHILD_CRITERIA, _CHILD_WHERE,
    _SAFE_CRITERIA, _SAFE_WHERE, _SAFE_OBJECT_CRITERIA, _SAFE_OBJECT_WHERE, _SCHOOL_CRITERIA, _SCHOOL_WHERE,
    _FOOD_CRITERIA, _FOOD_WHERE, _STAY_CRITERIA, _STAY_WHERE, _CLASS_CRITERIA, _CLASS_WHERE, _WALK_CRITERIA,
    _WALK_WHERE, _HOME_CRITERIA, _HOME_WHERE, _SCREEN_CRITERIA, _SCREEN_WHERE, _JOB_CRITERIA, _JOB_WHERE,
    _SIGHT_CRITERIA, _SIGHT_WHERE, _FAMILY_CRITERIA, _FAMILY_WHERE);
bundle_res!("bot.compose.a_place_nobody_can_see"; VERDICT_ASK, NOT_A_JUDGEMENT, _THING, _ABOUT_THE_BOT, _PLACE, _SAFE,
    _FEELING, _SCREEN, _JOB, _SAFE_OBJECT, _SCHOOL, _STAY, _FOOD, _CLASS, _WITH_CHILDREN, _WALK, _RAISING, _SIGHT,
    _TRIP, _NAMED, _HOME, _PAST_ASK);
bundle_sets!("bot.compose.a_place_nobody_can_see"; _BARE_NOUN, _NOT_A_THING, _TRAVEL_TOPICS, _NOT_A_PLACE_TOPICS);

/// Python: bot/compose/a_place_nobody_can_see.py:311 `_the_thing(said)`
///
/// The thing the line names, capitalised and ready to say back, or "" when it names none.
pub fn _the_thing(said: &str) -> String {
    let Some(found) = _THING.search(said) else { return String::new() };
    let mut words: Vec<&str> = text::split_ref(found.name_str("thing"));
    while words.last().is_some_and(|w| _STOP.iter().any(|s| s == w)) {
        words.pop();
    }
    while words.first().is_some_and(|w| matches!(*w, "it" | "this" | "that" | "there" | "they" | "these" | "those")) {
        words.remove(0);
    }
    // "is my street safe" said back "My street - can't tell you..." as if the bot owned the
    // street: a thing the person owns is said back as "That street".
    let mut owning = false;
    while words.first().is_some_and(|w| matches!(*w, "my" | "our")) {
        words.remove(0);
        owning = true;
    }
    if owning && !words.is_empty() {
        return text::upper_first(&format!("that {}", words.join(" ")));
    }
    // "is it a nice place" leaves "a" behind once "it" is dropped: an article with nothing
    // after it names nothing.
    while words.len() > 1
        && matches!(*words.last().unwrap(), "a" | "an" | "the" | "one" | "career" | "idea" | "option" | "choice")
    {
        words.pop();
    }
    if words.len() == 1 && matches!(words[0], "a" | "an" | "the" | "one") {
        return String::new();
    }
    if words.is_empty() || _NOT_A_THING.contains(*words.last().unwrap()) {
        return String::new();
    }
    let mut thing = words.join(" ");
    if words.len() == 1 && _BARE_NOUN.contains(words[0]) {
        thing = format!("that {}", words[0]);
    }
    text::upper_first(&thing)
}

/// What `_pool` gives back: the pool with its criteria and place, or Python's
/// `(None, None, None)`.
pub type Pool = Option<(&'static Vec<String>, &'static str, &'static str)>;

/// Python: bot/compose/a_place_nobody_can_see.py:344 `_pool(said, named=False, other_subject=False, travel=False)`
///
/// The pool for the kind of thing being judged, with the criteria and place that go with it.
/// None when the line names a kind nothing here can speak to, so the hedge it had is kept.
/// Order is the reading: the most specific kind wins, and a class in the line ("for kids",
/// "with a toddler") beats the verb it is asked with.
pub fn _pool(said: &str, named: bool, other_subject: bool, travel: bool) -> Pool {
    let has = |rx: &crate::py::re::Re| rx.search(said).is_some();
    if has(&_SAFE) && has(&_SAFE_OBJECT) && !has(&_HOME) {
        return Some((&SAFE_OBJECT, &_SAFE_OBJECT_CRITERIA, &_SAFE_OBJECT_WHERE));
    }
    if has(&_SCHOOL) {
        return Some((&SCHOOL, &_SCHOOL_CRITERIA, &_SCHOOL_WHERE));
    }
    if has(&_STAY) {
        return Some((&STAY, &_STAY_CRITERIA, &_STAY_WHERE));
    }
    if has(&_FOOD) {
        return Some((&FOOD, &_FOOD_CRITERIA, &_FOOD_WHERE));
    }
    if has(&_CLASS) {
        return Some((&CLASS, &_CLASS_CRITERIA, &_CLASS_WHERE));
    }
    // A sight comes before the child: "is the museum any good for kids" is a walk and a crowd,
    // not a flight and a beach.
    if has(&_SIGHT) {
        return Some((&SIGHT, &_SIGHT_CRITERIA, &_SIGHT_WHERE));
    }
    if has(&_RAISING) && has(&_WITH_CHILDREN) {
        return Some((&FAMILY, &_FAMILY_CRITERIA, &_FAMILY_WHERE));
    }
    // A child in the line makes it a trip with children only when the line has somewhere to go
    // in it: a place word, a trip word, or a place named by name. "is rugby safe for kids" is a
    // question about rugby.
    let somewhere = has(&_PLACE) || has(&_TRIP) || named || travel;
    if has(&_WITH_CHILDREN)
        && (somewhere || !has(&_SAFE))
        && !(other_subject && !somewhere)
        && !(has(&_SAFE) && !_the_thing(said).is_empty() && !somewhere)
        && !(has(&_HOME) && !has(&_TRIP))
    {
        return Some((&WITH_CHILDREN, &_CHILD_CRITERIA, &_CHILD_WHERE));
    }
    // Safety is about a place. A job, a marriage, a file and an account are not streets.
    if has(&_SAFE) && !has(&_PLACE) {
        return None;
    }
    if has(&_SAFE) {
        return Some((&SAFE_, &_SAFE_CRITERIA, &_SAFE_WHERE));
    }
    if has(&_WALK) {
        return Some((&WALK, &_WALK_CRITERIA, &_WALK_WHERE));
    }
    if has(&_TRIP) {
        return Some((&TRIP, &_TRIP_CRITERIA, &_TRIP_WHERE));
    }
    if has(&_HOME) {
        return Some((&HOME, &_HOME_CRITERIA, &_HOME_WHERE));
    }
    if has(&_SCREEN) {
        return Some((&SCREEN, &_SCREEN_CRITERIA, &_SCREEN_WHERE));
    }
    if has(&_JOB) {
        return Some((&JOB, &_JOB_CRITERIA, &_JOB_WHERE));
    }
    Some((&OTHER, "", ""))
}

/// Python: bot/compose/a_place_nobody_can_see.py:415 `reads_as_a_verdict_ask(text)`
///
/// True when the line asks the bot to judge a thing it cannot see.
pub fn reads_as_a_verdict_ask(text: &str) -> bool {
    let said = com4::ws_one(&com4::straight(&text.to_lowercase()));
    if said.is_empty() || VERDICT_ASK.search(&said).is_none() {
        return false;
    }
    if _ABOUT_THE_BOT.search(&said).is_some() {
        return false;
    }
    // A feeling answered as a decision to weigh up has no warmth in it at all.
    if _FEELING.search(&said).is_some() {
        return false;
    }
    // "was it a nice holiday" is past tense about a thing nobody has named: criteria for a trip
    // not yet booked is the wrong shape.
    if _PAST_ASK.search(&said).is_some()
        && matches!(_the_thing(&said).to_lowercase().as_str(), "" | "it" | "that" | "this" | "they")
    {
        return false;
    }
    NOT_A_JUDGEMENT.search(&said).is_none()
}

/// Python: bot/compose/a_place_nobody_can_see.py:434 `reads_as_a_hedge(reply)`
///
/// True when the reply is the search hedge this file replaces.
pub fn reads_as_a_hedge(reply: &str) -> bool {
    let said = com4::straight(&text::strip(reply).to_lowercase());
    HEDGE_OPENERS.iter().any(|opener| said.starts_with(opener.as_str()))
}

/// Python: bot/compose/a_place_nobody_can_see.py:440 `_the_topics(memory)`
///
/// The topics the understanding engine read the line as being about.
pub fn _the_topics(memory: &Memory) -> HashSet<String> {
    let Some(understood) = memory.understood.as_ref() else { return HashSet::new() };
    let mut topics: HashSet<String> = understood.topics.iter().cloned().collect();
    if let Some(topic) = understood.topic.as_ref().filter(|t| !t.is_empty()) {
        topics.insert(topic.clone());
    }
    topics
}

/// Python: bot/compose/a_place_nobody_can_see.py:453 `pool_for(text, memory=None)`
///
/// The pool of replies for a judgement ask on a thing the bot cannot see, or None. The caller
/// picks from the pool with the memory in hand.
pub fn pool_for(text: &str, memory: &Memory) -> Option<Vec<String>> {
    let raw = com4::ws_one(&com4::straight(text));
    let said = raw.to_lowercase();
    if !reads_as_a_verdict_ask(&said) {
        return None;
    }
    // The capitals are read off the line as it was typed: a place named by name has no place word.
    let topics = _the_topics(memory);
    let (pool, criteria, where_) = _pool(
        &said,
        _NAMED.search(&raw).is_some(),
        topics.iter().any(|t| _NOT_A_PLACE_TOPICS.contains(t)),
        topics.iter().any(|t| _TRAVEL_TOPICS.contains(t)),
    )?;
    let is_other = std::ptr::eq(pool, &*OTHER);
    if is_other && _the_thing(&said).is_empty() {
        // "is that ok", "is it worth it": a bare doubt with nothing named keeps the hedge.
        return None;
    }
    if is_other {
        return Some(pool.clone());
    }
    let thing = _the_thing(&said);
    let lead = if thing.is_empty() { String::new() } else { format!("{thing} \u{2014} ") };
    let mut said_lines = Vec::with_capacity(pool.len());
    for line in pool {
        let line = format_named(line, &[("lead", &lead), ("criteria", criteria), ("where", where_)]);
        said_lines.push(if lead.is_empty() { text::upper_first(&line) } else { line });
    }
    Some(said_lines)
}

/// Python: bot/compose/a_place_nobody_can_see.py:485 `answer_for(text, memory=None, reply='')`
///
/// A verdict ask the bot can help with, said honestly, or None. Only the search hedge is
/// replaced, so any entry, any module and the composer's own answer all keep the turn.
pub fn answer_for(text: &str, memory: &Memory, reply: &str) -> Option<String> {
    if !reads_as_a_hedge(reply) {
        return None;
    }
    let pool = pool_for(text, memory)?;
    if pool.is_empty() {
        return None;
    }
    Some(pick(&pool, memory, text))
}
