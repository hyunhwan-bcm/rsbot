//! Port of `bot/knowledge/plans.py`: parties, weddings, free time, moves, decluttering, baby
//! gear and room plans. The entries and their word tables are data in the bundle; these are the
//! guards, replies and the readers they share.

use crate::data;
use crate::data::{FnRef, Value};
use crate::knowledge::d2_shared::{amends_to, line_of, owns_the_deed_reading, pick_in, said_again};
use crate::knowledge::d2dup_g8::{a_dish_is_already_the_plan, cached_re, has_bounded_word, has_fenced_word};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{escape, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

macro_rules! g {
    ($t:ty, $name:ident, $f:path) => {
        static $name: LazyLock<$t> = LazyLock::new(|| $f(concat!("bot.knowledge.plans.", stringify!($name))));
    };
}

g!(&'static Re, AT_HOME, data::re);
g!(HashSet<String>, A_DAY_CONDITION, data::set);
g!(String, DAY_OFF_PLAN, data::string);
g!(String, DAY_PLAN, data::string);
g!(String, EVENING_PLAN, data::string);
g!(Vec<String>, FREE_TIME, data::strs);
g!(Vec<String>, FREE_TIME_ASK, data::strs);
g!(&'static Re, FREE_TIME_CUE, data::re);
g!(HashSet<String>, FREE_TIME_TIME, data::set);
g!(HashSet<String>, FREE_TIME_WHO, data::set);
g!(HashSet<String>, FREE_TIME_WORDS, data::set);
g!(String, FREE_WEEKEND, data::string);
g!(HashSet<String>, HOLIDAY_PREFIX, data::set);
g!(String, MEDICINE_NEEDS_A_REASON, data::string);
g!(Vec<String>, MONEY_WORDS, data::strs);
g!(String, NEWBORN_WEEK, data::string);
g!(String, NO_MONEY, data::string);
g!(String, SUMMER_WEEK, data::string);
g!(Vec<String>, TRY_ONCE, data::strs);
g!(String, WEEKEND_PLAN, data::string);
g!(Vec<String>, WEEK_OFF, data::strs);
g!(String, WEEK_PLAN, data::string);
g!(String, WEEK_PLAN_SKINT, data::string);
g!(String, WINTER_WEEK, data::string);
g!(String, _ADULT_PARTY, data::string);
g!(&'static Re, _AGE_AT_PARTY, data::re);
g!(HashSet<String>, _AMBIGUOUS_CITY_WORDS, data::set);
g!(&'static Re, _AN_IDEA_ASKED_FOR, data::re);
g!(Vec<String>, _ASKS, data::strs);
g!(Vec<String>, _ASKS_ABOUT_IT, data::strs);
g!(&'static Re, _ASKS_A_CLOCK, data::re);
g!(&'static Re, _ASKS_THE_WORLD, data::re);
g!(HashSet<String>, _ASK_WORDS, data::set);
g!(Vec<String>, _AT_HOME, data::strs);
g!(Vec<String>, _AT_THE_PLACE_WORDS, data::strs);
g!(&'static Re, _A_CHILD_AT_A_MILESTONE, data::re);
g!(&'static Re, _A_DATE_ASK, data::re);
g!(&'static Re, _A_DO, data::re);
g!(&'static Re, _A_MILESTONE_AGE, data::re);
g!(&'static Re, _A_MILESTONE_PERSON, data::re);
g!(&'static Re, _A_MILESTONE_SOON, data::re);
g!(&'static Re, _A_MOVE_ELSEWHERE, data::re);
g!(&'static Re, _A_MOVE_TO_PLAN, data::re);
g!(Vec<String>, _A_PLANNED_ROOM, data::strs);
g!(&'static Re, _A_PRESENT_ASKED, data::re);
g!(Vec<String>, _A_THING_THE_PERSON_HAS, data::strs);
g!(Vec<String>, _BABY_GEAR_FAULT, data::strs);
g!(String, _BABY_GEAR_MEND, data::string);
g!(&'static Re, _BARE_RID, data::re);
g!(String, _BATHROOM_PLAN, data::string);
g!(HashSet<String>, _BLOCK_QUALIFIERS, data::set);
g!(Vec<String>, _BRING_IT, data::strs);
g!(Vec<String>, _CHILD_AT_PARTY, data::strs);
g!(&'static Re, _COLD, data::re);
g!(&'static Re, _COLD_WEATHER, data::re);
g!(HashSet<String>, _COUNT_WORDS, data::set);
g!(Vec<String>, _DECLUTTER_ROOM, data::strs);
g!(Vec<String>, _DECLUTTER_STUFF, data::strs);
g!(Vec<String>, _EVENT_AWAY, data::strs);
g!(Vec<String>, _EVENT_EVENING, data::strs);
g!(&'static Re, _FOR_A_RAINY_DAY, data::re);
g!(Vec<String>, _FOR_THE_PLACE_WORDS, data::strs);
g!(HashSet<String>, _FREE_BLOCK_WORDS, data::set);
g!(Vec<String>, _GIFT_ASK, data::strs);
g!(&'static Re, _HOME_IN_THE_LINE, data::re);
g!(&'static Re, _IDEAS_ASK, data::re);
g!(&'static Re, _IDEAS_FOR, data::re);
g!(HashSet<String>, _IDEA_FILLER, data::set);
g!(&'static Re, _IDEA_NONE, data::re);
g!(Vec<String>, _JOB_NAMED, data::strs);
g!(HashSet<String>, _JOB_WORDS, data::set);
g!(String, _KIDS_PARTY, data::string);
g!(String, _KITCHEN_PLAN, data::string);
g!(Vec<String>, _MEDIUM_NAMED, data::strs);
g!(Vec<String>, _MONEY_ONLY, data::strs);
g!(&'static Re, _MONEY_WITH_IT, data::re);
g!(Vec<String>, _MONTHS, data::strs);
g!(&'static Re, _MUSING, data::re);
g!(Vec<String>, _NOTE_ASK, data::strs);
g!(HashSet<String>, _NOT_THE_THING, data::set);
g!(HashSet<String>, _NO_SUBJECT, data::set);
g!(Vec<String>, _OWN_ACTIVITY, data::strs);
g!(&'static Re, _OWN_COUNT, data::re);
g!(&'static Re, _PURPOSE_AFTER_RUN, data::re);
g!(Vec<String>, _REQUESTING_TIME, data::strs);
g!(&'static Re, _SHORT_RUN, data::re);
g!(Vec<String>, _SWALLOWED, data::strs);
g!(Vec<String>, _SWALLOWED_THING, data::strs);
g!(Vec<String>, _THE_EVENING, data::strs);
g!(Vec<String>, _THE_OWNER, data::strs);
g!(Vec<String>, _THE_WORLD, data::strs);
g!(HashSet<String>, _TIME_WORDS, data::set);
g!(Vec<String>, _TODAY_WORDS, data::strs);
g!(&'static Re, _TOO_MUCH, data::re);
g!(Vec<String>, _TRIP_PHRASES, data::strs);
g!(Vec<String>, _UNWELL, data::strs);
g!(String, _UTILITY_PLAN, data::string);
g!(&'static Re, _WEDDING_COUNT, data::re);
g!(String, _WEDDING_GUESTS, data::string);
g!(String, _WEDDING_LEAD_TIME, data::string);
g!(Vec<String>, _WEDDING_MONEY, data::strs);
g!(Vec<String>, _WEDDING_PAPERS, data::strs);
g!(Vec<String>, _WEDDING_WHEN, data::strs);
g!(Vec<String>, _WEEKEND_WORDS, data::strs);
g!(&'static Re, _WHAT_HAPPENED, data::re);
g!(Vec<String>, _WHEN_AWAY, data::strs);
g!(Vec<String>, _WHERE_GO_PHRASES, data::strs);
g!(Vec<String>, _WORK_BLOCK, data::strs);

/// `VAGUE_TOPICS`: the topics a line with no subject of its own is read as (None among them).
static VAGUE_TOPICS: LazyLock<Vec<Option<String>>> = LazyLock::new(|| {
    data::get("bot.knowledge.plans.VAGUE_TOPICS").items().iter().map(|v| v.as_str().map(str::to_string)).collect()
});

/// `_BABY_GEAR`: (keys, name, sentence) per piece of gear, in table order.
static _BABY_GEAR: LazyLock<Vec<(Vec<String>, String, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.plans._BABY_GEAR")
        .items()
        .iter()
        .map(|row| (row.items()[0].strs(), row.items()[1].str().to_string(), row.items()[2].str().to_string()))
        .collect()
});

/// `RECIPES` (cooking's table, read here): (aliases, keys, body, avoid).
static RECIPES: LazyLock<Vec<(Vec<String>, String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.knowledge.plans.RECIPES")
        .items()
        .iter()
        .map(|row| (row.items()[0].strs(), row.items()[2].str().to_string(), row.items()[3].strs()))
        .collect()
});

/// `travel.CITIES`, the cities a guide answers for, in table order.
static CITIES: LazyLock<Vec<String>> = LazyLock::new(|| {
    data::get("bot.knowledge.travel.CITIES").pairs().iter().map(|(k, _)| k.str().to_string()).collect()
});

static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static NOT_A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9']+", 0));

fn any_in(text: &str, phrases: &[String]) -> bool {
    phrases.iter().any(|p| text.contains(p.as_str()))
}

fn any_of(text: &str, phrases: &[&str]) -> bool {
    phrases.iter().any(|p| text.contains(p))
}

fn or<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.is_empty() { b } else { a }
}

fn letters(text: &str) -> Vec<String> {
    LETTERS.findall_str(text)
}

/// The line as typed, or its words when the reading has no text.
fn typed_line(understood: &Understood) -> String {
    if !understood.text.is_empty() {
        return understood.text.clone();
    }
    understood.words.join(" ")
}

/// False when the line counts what the person has rather than asking how many to invite.
pub fn _not_a_count_of_their_own(understood: &Understood) -> bool {
    !_OWN_COUNT.is_match(&format!(" {} ", typed_line(understood).to_lowercase()))
}

/// False when the line asks for the date of a birthday rather than for a party.
pub fn _not_a_date_question(understood: &Understood) -> bool {
    let line = format!(" {} ", typed_line(understood).to_lowercase());
    !(_A_DATE_ASK.is_match(&line) && !_A_DO.is_match(&line))
}

/// False when the line asks what happened rather than for something to do.
pub fn _asks_for_ideas(understood: &Understood) -> bool {
    !_WHAT_HAPPENED.is_match(&format!(" {} ", typed_line(understood).to_lowercase()))
}

/// The free weekend fits the line and the line asks for ideas.
pub fn _the_free_weekend_fits_and_is_asked(understood: &Understood) -> bool {
    _free_weekend_fits(understood) && _asks_for_ideas(understood)
}

/// False when the line reports something the person did to somebody: that is a repair.
pub fn _not_a_deed_of_their_own(reading: &Understood) -> bool {
    !(owns_the_deed_reading(reading) || amends_to(line_of(reading).unwrap_or("")))
}

/// True when the line names a room or a pile to clear, or asks the job with nothing named.
pub fn _declutter_fits(understood: &Understood) -> bool {
    let line = or(&understood.clean, &understood.text);
    let clean = format!(" {} ", text::squash(line));
    if _BARE_RID.is_match(&clean) {
        return true;
    }
    if _JOB_NAMED.iter().any(|job| clean.contains(&format!(" {job} "))) {
        return true;
    }
    if _TOO_MUCH.is_match(&clean) {
        return true;
    }
    let names_room = _DECLUTTER_ROOM.iter().any(|room| clean.contains(&format!(" {room} ")));
    let names_pile = _DECLUTTER_STUFF.iter().any(|pile| clean.contains(&format!(" {pile} ")));
    names_room || names_pile
}

/// One thing worth trying once, or None when the line is about a subject of its own.
pub fn _try_once(memory: &mut Memory, understood: &Understood) -> Option<String> {
    if understood.subject.as_deref().is_some_and(|s| !s.is_empty()) {
        return None;
    }
    if !VAGUE_TOPICS.contains(&understood.topic) {
        return None;
    }
    Some(pick_in(&TRY_ONCE, memory, &_words_of(understood)))
}

/// Another thing to try, when the last reply was one - None for every other line.
pub fn _try_once_more(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let last = memory.turns.last().map(|t| t.bot.clone()).unwrap_or_default();
    let pool: HashSet<String> = TRY_ONCE.iter().map(|one| said_again(one)).collect();
    if !pool.contains(&said_again(&last)) {
        return None;
    }
    if understood.subject.as_deref().is_some_and(|s| !s.is_empty()) {
        return None;
    }
    Some(pick_in(&TRY_ONCE, memory, &_words_of(understood)))
}

/// A wedding: when to book, the paperwork, the guest count, or the order to plan it in.
pub fn _wedding_plan(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = format!(" {} ", or(&understood.clean, &understood.text).to_lowercase());
    if any_in(&line, &_WEDDING_WHEN) {
        return Some(_WEDDING_LEAD_TIME.clone());
    }
    if any_in(&line, &_WEDDING_PAPERS) && !any_in(&line, &_WEDDING_MONEY) {
        return Some("Give notice first: you each do it at your local register office at least 29 days before the wedding (in Scotland a marriage notice to the registrar), with a passport or birth certificate and proof of address. Book the venue and the registrar's slot, and find two witnesses. The 29 days is a legal minimum, not a booking window, so do not leave it late.".into());
    }
    if _WEDDING_COUNT.is_match(&line) {
        return Some(_WEDDING_GUESTS.clone());
    }
    Some("Guest list first, because it sets the cost of everything else: the venue, the food and the drink are about 60% of a wedding, and halving the guest list roughly halves the bill. Book the venue and the registrar next (a year ahead for a Saturday), then the photographer, and keep 10% back for what you forgot.".into())
}

/// True when the line is asking what is going on in the world rather than filling a day.
pub fn _asks_about_the_world(said: &str) -> bool {
    _holds(said, &_THE_WORLD) && _ASKS_THE_WORLD.is_match(said)
}

/// The line as one lowered string, from the clean text as well as the words.
pub fn _said_words(understood: &Understood) -> String {
    let mut parts: Vec<&str> = vec![&understood.clean, &understood.text];
    parts.extend(understood.words.iter().map(String::as_str));
    parts.into_iter().filter(|p| !p.is_empty()).collect::<Vec<_>>().join(" ").to_lowercase()
}

/// This line alone, without the turn before that a short follow-up arrives with.
pub fn _said_now(understood: &Understood) -> String {
    match understood.clauses.iter().filter(|c| !c.is_empty()).last() {
        Some(clause) => clause.join(" ").to_lowercase(),
        None => _said_words(understood),
    }
}

/// True when the line asks for the plan of a move of house, spelled without the word house.
pub fn _a_household_move(understood: &Understood) -> bool {
    let said = _said_words(understood);
    _A_MOVE_TO_PLAN.is_match(&said) && !_A_MOVE_ELSEWHERE.is_match(&said)
}

/// True when the line names the home it is leaving, or asks for a move's plan without one.
pub fn _a_household_move_or_a_named_move(understood: &Understood) -> bool {
    if _HOME_IN_THE_LINE.is_match(&_said_words(understood)) {
        return true;
    }
    _a_household_move(understood)
}

/// The free-weekend plan, or None when the line is asking about something you swallow.
pub fn _a_free_weekend(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = _said_words(understood);
    if _a_medicine_line(&said) {
        return Some(MEDICINE_NEEDS_A_REASON.clone());
    }
    if _holds(&said, &_TODAY_WORDS) && !_holds(&said, &_WEEKEND_WORDS) {
        if _holds(&said, &["tonight".to_string(), "this evening".to_string()]) {
            return Some(EVENING_PLAN.clone());
        }
        return Some(DAY_OFF_PLAN.clone());
    }
    Some(FREE_WEEKEND.clone())
}

/// True when one of `words` stands in `said` as a whole word or phrase.
pub fn _holds(said: &str, words: &[String]) -> bool {
    words.iter().any(|w| has_bounded_word(said, w))
}

/// True when the line is about something you swallow, not about a free day.
pub fn _a_medicine_line(said: &str) -> bool {
    if !_holds(said, &_SWALLOWED) {
        return false;
    }
    if _AN_IDEA_ASKED_FOR.is_match(said) && !_holds(said, &_SWALLOWED_THING) {
        return false;
    }
    true
}

/// Whether the line asks what to do with time it has, rather than about a thing it names.
pub fn _a_stretch_of_time_to_fill(understood: &Understood) -> bool {
    let mut named: Vec<String> = Vec::new();
    if let Some(subject) = understood.subject.as_deref().filter(|s| !s.is_empty()) {
        named.push(subject.to_lowercase());
    }
    for found in &understood.entities {
        if !found.text.is_empty() {
            named.push(found.text.to_lowercase());
        }
    }
    let line_said = understood.clean.to_lowercase();
    let named_line = named.join(" ").to_lowercase();
    if FREE_TIME_CUE.is_match(&line_said)
        && letters(&named_line).iter().all(|w| {
            FREE_TIME_WORDS.contains(w) || FREE_TIME_WHO.contains(w) || A_DAY_CONDITION.contains(w) || FREE_TIME_TIME.contains(w)
        })
    {
        return true; // "nothing planned this weekend and it is cold" is the weekend
    }
    let mut words_named = letters(&AT_HOME.sub(" ", &named_line));
    if words_named.iter().any(|w| FREE_TIME_TIME.contains(w)) {
        words_named.retain(|w| !HOLIDAY_PREFIX.contains(w));
    }
    let covered = _covered_words(&line_said, &FREE_TIME);
    let line_words: HashSet<String> = letters(&line_said).into_iter().collect();
    let block = !covered.is_empty() || line_words.iter().any(|w| FREE_TIME_TIME.contains(w));
    let here_now = _the_place_the_block_is_at(&line_said, block);
    for word in &words_named {
        if FREE_TIME_WORDS.contains(word) || FREE_TIME_WHO.contains(word) {
            continue;
        }
        if covered.contains_key(word) || here_now.contains(word) {
            continue;
        }
        return false; // the line names something of its own to be about
    }
    if line_words.iter().any(|w| FREE_TIME_TIME.contains(w)) {
        return true;
    }
    if AT_HOME.is_match(&line_said) {
        return true;
    }
    if !covered.is_empty() {
        return true;
    }
    // `understood.get("topic_own", understood.get("topic"))`: a reading always has the key
    match understood.topic_own.as_deref() {
        None | Some("") => true,
        Some(topic) => topic == "small-talk",
    }
}

static BRING_TO: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(take|taking|took|bring|bringing|give|giving|make)\b[^.]*\bto\b", 0));

/// False when the line is not a question about the weekend at all.
pub fn _free_weekend_fits(understood: &Understood) -> bool {
    if _about_a_thing_the_person_has(understood) {
        return false;
    }
    let said = _said_words(understood);
    if _ASKS_A_CLOCK.is_match(&said) {
        return false;
    }
    if _asks_about_the_world(&said) {
        return false;
    }
    // TYPE: Python reads `leans_on` as a dict (`subject`, `topic`) on a follow-up turn; the
    // typed reading does not carry that yet, so the line leans on nothing here.
    let follows = false;
    if _holds(&said, &_BRING_IT) && BRING_TO.is_match(&said) {
        return false;
    }
    if _holds(&said, &_SWALLOWED) && !_holds(&_said_now(understood), &_SWALLOWED) {
        return _holds(&_said_now(understood), &_OWN_ACTIVITY);
    }
    if !_a_medicine_line(&said) {
        // `follows and leans_on["topic"] == "health" and ...` cannot hold without a leans_on
        return true;
    }
    !follows
}

/// False when "for a rainy day" is the weather rather than the savings idiom.
pub fn _a_rainy_day_is_money(reading: &Understood) -> bool {
    let said = _said_words(reading);
    if !_FOR_A_RAINY_DAY.is_match(&said) {
        return true;
    }
    _MONEY_WITH_IT.is_match(&said)
}

/// True when the line names a city a guide answers for.
pub fn _a_city_is_named(line: &str) -> bool {
    let padded = format!(" {line} ");
    for city in CITIES.iter() {
        if _AMBIGUOUS_CITY_WORDS.contains(city) {
            continue;
        }
        if cached_re(&format!("(?<![a-z]){}(?![a-z])", escape(city)), 0).is_match(&padded) {
            return true;
        }
    }
    false
}

/// True when the line asks for ideas about a named thing rather than about a free block.
pub fn _ideas_about_a_thing(understood: &Understood) -> bool {
    let clean = understood.clean.to_lowercase();
    if clean.is_empty() {
        return false;
    }
    if _IDEA_NONE.is_match(&clean) {
        return false;
    }
    if _IDEAS_ASK.is_match(&clean) {
        return true;
    }
    let Some(found) = _IDEAS_FOR.search(&clean) else { return false };
    let rest = found.str(1);
    let mut words: Vec<String> = letters(rest).into_iter().filter(|w| !_IDEA_FILLER.contains(w)).collect();
    if words.iter().any(|w| w == "away" || w == "somewhere" || w == "abroad") {
        return true;
    }
    if words.is_empty() {
        return false;
    }
    let phrases: Vec<String> = FREE_TIME.iter().chain(WEEK_OFF.iter()).cloned().collect();
    let covered = _covered_words(rest, &phrases);
    let here = _the_place_the_block_is_at(&clean, true);
    words.retain(|w| !covered.contains_key(w) && !here.contains(w));
    let Some(first) = words.first() else { return false };
    !_TIME_WORDS.contains(first) && !_NO_SUBJECT.contains(first)
}

/// True when the line asks what to do about an ache or a complaint of its own.
pub fn _about_a_thing_the_person_has(reading: &Understood) -> bool {
    if !reading.subject.as_deref().is_some_and(|s| !s.is_empty()) {
        return false;
    }
    let topic_own = reading.topic_own.as_deref();
    if !topic_own.is_some_and(|t| _A_THING_THE_PERSON_HAS.iter().any(|x| x == t)) {
        return false;
    }
    _holds(&_said_words(reading), &_ASKS_ABOUT_IT)
}

/// False when the line asks what to do about a thing of its own rather than about a free day.
pub fn _a_day_out_is_the_question(reading: &Understood) -> bool {
    if _ideas_about_a_thing(reading) {
        return false;
    }
    !_about_a_thing_the_person_has(reading)
}

/// The counted run of days in the line, unless the line says what the time is for (Python
/// returns the match; only whether there is one is ever read).
pub fn _short_run(line: &str) -> bool {
    let Some(found) = _SHORT_RUN.search(line) else { return false };
    !_PURPOSE_AFTER_RUN.is_match(&line[found.end()..])
}

/// The words of the room or corner the free block is spent in, if the line names one.
pub fn _the_place_the_block_is_at(said: &str, block: bool) -> HashSet<String> {
    let words = letters(&said.to_lowercase());
    let preps: HashSet<&str> = _AT_THE_PLACE_WORDS
        .iter()
        .map(String::as_str)
        .chain(if block { _FOR_THE_PLACE_WORDS.iter().map(String::as_str).collect::<Vec<_>>() } else { Vec::new() })
        .collect();
    let mut here = HashSet::new();
    for (i, word) in words.iter().enumerate() {
        if !preps.contains(word.as_str()) {
            continue;
        }
        let mut j = i + 1;
        if j < words.len() && _THE_OWNER.contains(&words[j]) {
            j += 1;
        }
        let pair = words[j.min(words.len())..(j + 2).min(words.len())].join(" ");
        if _AT_HOME.contains(&pair) {
            here.extend(pair.split(' ').map(str::to_string));
        } else if j < words.len() && _AT_HOME.contains(&words[j]) {
            here.insert(words[j].clone());
        }
    }
    here
}

/// Whether the line's subject is the free block itself rather than a thing of its own.
pub fn _names_the_free_block(named: &str) -> bool {
    let lowered = named.to_lowercase();
    let words: Vec<String> = NOT_A_WORD.split(&lowered).into_iter().filter(|w| !w.is_empty()).collect();
    if words.is_empty() {
        return true;
    }
    words.iter().all(|w| {
        _FREE_BLOCK_WORDS.contains(w) || _ASK_WORDS.contains(w) || _BLOCK_QUALIFIERS.contains(w) || _COUNT_WORDS.contains(w)
    })
}

/// The words of the phrases of `phrases` this line holds, counted once each.
pub fn _covered_words(text: &str, phrases: &[String]) -> HashMap<String, i64> {
    let mut covered: HashMap<String, i64> = HashMap::new();
    let mut left = format!(" {text} ");
    let mut ordered: Vec<&String> = phrases.iter().collect();
    ordered.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()));
    for phrase in ordered {
        let pattern = cached_re(&format!("(?<![a-z0-9]){}(?![a-z0-9])", escape(phrase)), 0);
        while pattern.is_match(&left) {
            left = pattern.subn(" ", &left, 1);
            for word in text::split_ref(phrase) {
                *covered.entry(word.to_string()).or_insert(0) += 1;
            }
        }
    }
    covered
}

/// Whether the line names a thing the plan is not about.
pub fn _names_a_thing_of_its_own(understood: &Understood) -> bool {
    let words: HashSet<String> = understood.keywords.iter().map(|w| w.to_lowercase().replace('\'', "")).collect();
    if words.iter().any(|w| _JOB_WORDS.contains(w)) {
        return false;
    }
    let said = format!(" {} ", understood.clean.to_lowercase());
    if _MUSING.is_match(&said) {
        return false;
    }
    let block_phrases: Vec<String> = FREE_TIME.iter().chain(WEEK_OFF.iter()).chain(_WORK_BLOCK.iter()).cloned().collect();
    let holds_block =
        !_covered_words(&said, &block_phrases).is_empty() || letters(&said).iter().any(|w| FREE_TIME_TIME.contains(w));
    let here = _the_place_the_block_is_at(&said, holds_block);
    if let Some(shown) = understood.subject.as_deref().filter(|s| !s.is_empty()) {
        let lowered = shown.to_lowercase();
        let left = NOT_A_WORD
            .split(&lowered)
            .into_iter()
            .filter(|w| !w.is_empty() && !here.contains(w))
            .collect::<Vec<_>>()
            .join(" ");
        if !left.is_empty() && !_names_the_free_block(&left) {
            return true;
        }
    }
    let all_phrases: Vec<String> = FREE_TIME
        .iter()
        .chain(WEEK_OFF.iter())
        .chain(FREE_TIME_ASK.iter())
        .chain(_ASKS.iter())
        .chain(_WORK_BLOCK.iter())
        .cloned()
        .collect();
    let mut covered = _covered_words(&said, &all_phrases);
    for word in &understood.keywords {
        let low = word.to_lowercase().replace('\'', "");
        if _FREE_BLOCK_WORDS.contains(&low) || _ASK_WORDS.contains(&low) || _BLOCK_QUALIFIERS.contains(&low) || _COUNT_WORDS.contains(&low)
        {
            continue;
        }
        if text::is_digit(&low) {
            continue;
        }
        if here.contains(&low) {
            continue;
        }
        if let Some(n) = covered.get_mut(&low).filter(|n| **n != 0) {
            *n -= 1;
            continue;
        }
        if _said_as_the_thing_it_does_not_want(&understood.clean, &low) {
            continue;
        }
        return true;
    }
    false
}

/// Whether this word comes into the line as the thing the person wants to get away from.
pub fn _said_as_the_thing_it_does_not_want(line: &str, word: &str) -> bool {
    let lowered = line.to_lowercase();
    let pattern = cached_re(&format!("(?<![a-z0-9]){}(?![a-z0-9])", escape(&word.to_lowercase())), 0);
    let Some(found) = pattern.search(&lowered) else { return false };
    // Python slices the line as typed at the match's character index, then lowers it
    let at = text::char_index(&lowered, found.start());
    let before_text: String = line.chars().take(at).collect::<String>().to_lowercase();
    let before = text::split_ref(&before_text);
    let last_two = &before[before.len().saturating_sub(2)..];
    !last_two.is_empty() && last_two.iter().any(|w| _NOT_THE_THING.contains(*w))
}

/// The plan for a free block of time, or None when the line is really about something else.
pub fn _free_time(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" ");
    let clean = understood.clean.to_lowercase();
    let line = format!("{said} {clean}");
    if a_dish_is_already_the_plan(&clean) {
        return None;
    }
    if _UNWELL.iter().any(|w| has_fenced_word(&line, w)) {
        return None;
    }
    if _COLD.is_match(&line) && !_COLD_WEATHER.is_match(&line) {
        return None;
    }
    if any_in(&line, &_WHERE_GO_PHRASES)
        && (any_in(&line, &MONEY_WORDS) || any_in(&line, &_MONEY_ONLY) || any_in(&line, &_WHEN_AWAY))
    {
        return None;
    }
    if any_in(&line, &_TRIP_PHRASES) {
        return None;
    }
    if any_in(&line, &_MONTHS) {
        return None;
    }
    if _a_city_is_named(&line) {
        return None;
    }
    if _dish_for_tonight(memory, understood).is_some() {
        return None;
    }
    if any_in(&line, &_NOTE_ASK) {
        return None;
    }
    if any_in(&line, &_EVENT_AWAY) {
        return None;
    }
    if any_in(&line, &_EVENT_EVENING) {
        return None;
    }
    if any_in(&line, &_MEDIUM_NAMED) {
        return None;
    }
    if any_of(&line, &[
        "newborn", "new baby", "new-born", "just had a baby", "had a baby", "baby is due", "baby due", "paternity",
        "maternity leave", "on maternity",
    ]) {
        return Some(NEWBORN_WEEK.clone());
    }
    if any_in(&line, &_REQUESTING_TIME) {
        return None;
    }
    if any_in(&line, &_WORK_BLOCK) && !_short_run(&line) && !any_in(&line, &_ASKS) {
        return None;
    }
    if _ideas_about_a_thing(understood) {
        return None;
    }
    if _about_a_thing_the_person_has(understood) {
        return None;
    }
    if _names_a_thing_of_its_own(understood) {
        return None;
    }
    if any_in(&line, &_GIFT_ASK) {
        return None;
    }
    // `plan is WEEK_PLAN` in Python: whether the week's plan was the one picked
    let mut week = false;
    let mut plan = if any_of(&line, &["this afternoon", "free afternoon", "afternoon off"]) {
        DAY_PLAN.clone()
    } else if any_of(&line, &["day off", "days off", "free day"]) {
        DAY_OFF_PLAN.clone()
    } else if any_in(&line, &_THE_EVENING) {
        EVENING_PLAN.clone()
    } else if (_short_run(&line) || any_of(&line, &["few days", "couple of days"]))
        && !any_of(&line, &["week off", "a week off", "weeks off", "whole week", "week free"])
    {
        DAY_PLAN.clone()
    } else if any_in(&line, &WEEK_OFF) {
        week = true;
        WEEK_PLAN.clone()
    } else {
        WEEKEND_PLAN.clone()
    };
    if week && (any_in(&line, &MONEY_WORDS) || any_in(&line, &_MONEY_ONLY)) {
        plan = WEEK_PLAN_SKINT.clone();
        week = false;
    }
    if week {
        if any_of(&line, &["summer", "hot", "heat", "sunny", "sunshine"]) {
            plan = format!("{plan} {}", *SUMMER_WEEK);
        } else if any_of(&line, &["winter", "autumn", "frost", "snow", "dark"]) {
            plan = format!("{plan} {}", *WINTER_WEEK);
        }
    }
    if any_in(&line, &MONEY_WORDS) {
        plan = format!("{plan} {}", *NO_MONEY);
    }
    Some(plan)
}

/// The pieces of gear the line names, in the order they are in the table.
pub fn _gear_named(words: &str) -> Vec<String> {
    _BABY_GEAR
        .iter()
        .filter(|(keys, _, _)| keys.iter().any(|key| _has_word(words, key)))
        .map(|(_, name, _)| name.clone())
        .collect()
}

/// `word` stands in `words` with no letter or digit either side.
pub fn _has_word(words: &str, word: &str) -> bool {
    has_fenced_word(words, word)
}

/// The one piece the line is about, answered first, or the whole first shop.
pub fn _a_piece_of_baby_gear(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = _words_of(understood);
    let mut named = _gear_named(&words);
    let fault = _BABY_GEAR_FAULT.iter().any(|w| _has_word(&words, w));
    if fault && named.is_empty() {
        // "why do the wheels keep squeaking" after a pram: the piece is carried, not said
        let topic = match memory.extra.get("topic") {
            Some(Value::Dict(_)) => memory.extra["topic"].get("text").map(|v| v.str().to_lowercase()).unwrap_or_default(),
            Some(v) if v.truthy() => panic!("memory[\"topic\"] has no .get"),
            _ => String::new(),
        };
        named = _gear_named(&topic);
    }
    if fault && !named.is_empty() {
        return Some(_BABY_GEAR_MEND.clone());
    }
    if named.len() >= 2
        && (words.contains("which")
            || words.contains("better")
            || format!(" {words} ").contains(" or ")
            || words.contains("versus")
            || words.contains("vs"))
    {
        let said: HashMap<&str, &str> = _BABY_GEAR.iter().map(|(_, name, sentence)| (name.as_str(), sentence.as_str())).collect();
        let pair = named[..2].iter().map(|n| said[n.as_str()]).collect::<Vec<_>>().join(" ");
        return Some(text::strip(&format!("Whichever you use most is the one to spend on. {pair}")).to_string());
    }
    if named.len() == 1 {
        for (_, name, sentence) in _BABY_GEAR.iter() {
            if *name == named[0] {
                return Some(sentence.clone());
            }
        }
    }
    Some("Buy second-hand for all of it except the mattress: a cot (mattress 120 by 60cm) or cot bed (140 by 70cm) with a new firm mattress that fits with no gaps, a car seat that has not been in a crash and whose history you know - i-Size (R129) is the newer standard, and how it fits your car matters more than the brand - and a pram that folds small enough for your car and lies flat for a newborn. Prices: a new pram \u{a3}150 to \u{a3}400, second-hand \u{a3}60 to \u{a3}150, a cot \u{a3}100 to \u{a3}250 new. You need little else at the start (six babygros in two sizes, nappies, muslins); a monitor is a comfort rather than a must, and a high chair can wait until six months.".into())
}

/// The line, and the words it was read as, in one lowered string.
pub fn _words_of(understood: &Understood) -> String {
    _said_words(understood)
}

/// The recipe for the dish the line names for an evening, or None.
pub fn _dish_for_tonight(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = _words_of(understood);
    let mut best: Option<(usize, &str)> = None;
    for (aliases, body, avoid) in RECIPES.iter() {
        let hits: Vec<&String> = aliases.iter().filter(|a| has_bounded_word(&said, a)).collect();
        if hits.is_empty() {
            continue;
        }
        if avoid.iter().any(|w| has_bounded_word(&said, w)) {
            continue;
        }
        let length = hits.iter().map(|a| a.chars().count()).max().unwrap_or(0);
        // `max(found, key=...)` keeps the first of the longest
        if best.is_none_or(|(l, _)| length > l) {
            best = Some((length, body.as_str()));
        }
    }
    best.map(|(_, body)| body.to_string())
}

/// True when the line is somebody's birthday, said as an age and a day.
pub fn _a_milestone_said_as_an_age(understood: &Understood) -> bool {
    let mut parts = vec![understood.clean.clone()];
    parts.extend(understood.words.iter().cloned());
    let line = parts.join(" ").to_lowercase();
    if !_A_MILESTONE_PERSON.is_match(&line) {
        return false;
    }
    let Some(age) = _A_MILESTONE_AGE.search(&line) else { return false };
    let years: i64 = age.str(1).parse().unwrap_or(0);
    if years < 18 && !_A_CHILD_AT_A_MILESTONE.is_match(&line) {
        return false;
    }
    if _A_PRESENT_ASKED.is_match(&line) {
        return false;
    }
    _A_MILESTONE_SOON.is_match(&line)
}

/// The grown-up party plan, or the children's one when the line names a child.
pub fn _party_plan(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let line = format!("{} {}", reading.clean.to_lowercase(), reading.words.join(" "));
    let ages: Vec<i64> = _AGE_AT_PARTY
        .findall_str(&line)
        .iter()
        .filter_map(|f| f.parse::<i64>().ok())
        .filter(|n| (1..=99).contains(n))
        .collect();
    if let Some(youngest) = ages.iter().min() {
        return Some(if *youngest < 13 { _KIDS_PARTY.clone() } else { _ADULT_PARTY.clone() });
    }
    // the word goes into the pattern as it is, unescaped, as in Python
    if _CHILD_AT_PARTY.iter().any(|w| cached_re(&format!("\\b{w}\\b"), 0).is_match(&line)) {
        return Some(_KIDS_PARTY.clone());
    }
    Some(_ADULT_PARTY.clone())
}

/// Whether the line is asking for a thing to give rather than for the do itself.
pub fn _not_a_gift_ask(understood: &Understood) -> bool {
    let line = format!(
        " {} {} ",
        understood.clean.to_lowercase(),
        understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" ")
    );
    !any_in(&line, &_GIFT_ASK)
}

/// Both of the party row's stand-downs, so the row keeps neither a present nor a date.
pub fn _not_a_gift_or_a_date(understood: &Understood) -> bool {
    _not_a_gift_ask(understood) && _not_a_date_question(understood)
}

static A_PLANNED_ROOM: LazyLock<Re> = LazyLock::new(|| {
    let words: Vec<String> = _A_PLANNED_ROOM.iter().map(|w| escape(w)).collect();
    Re::new(&format!("(?<![a-z0-9])(?:{})(?![a-z0-9])", words.join("|")), 0)
});

/// The room the line names, planned in its own terms, or None for a line that names none.
pub fn _a_room_plan(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = or(&understood.clean, &understood.text).to_lowercase();
    if !A_PLANNED_ROOM.is_match(&said) {
        return None;
    }
    if any_of(&said, &["utility", "laundry", "boot room"]) {
        return Some(_UTILITY_PLAN.clone());
    }
    if any_of(&said, &["bath", "bathroom", "shower", "toilet", "loo", "ensuite", "wet room"]) {
        return Some(_BATHROOM_PLAN.clone());
    }
    Some(_KITCHEN_PLAN.clone())
}

/// `guard=lambda u: _not_a_gift_or_a_date(u) and _not_a_deed_of_their_own(u)` (line 2220).
pub fn _a_party_to_plan(understood: &Understood) -> bool {
    _not_a_gift_or_a_date(understood) && _not_a_deed_of_their_own(understood)
}

/// `guard=lambda reading: _the_free_weekend_fits_and_is_asked(r) and _a_stretch_of_time_to_fill(r)` (line 2752).
pub fn _a_free_weekend_to_fill(reading: &Understood) -> bool {
    _the_free_weekend_fits_and_is_asked(reading) && _a_stretch_of_time_to_fill(reading)
}

/// `guard=lambda reading: _a_day_out_is_the_question(r) and _a_stretch_of_time_to_fill(r)` (line 3335).
pub fn _a_day_out_to_fill(reading: &Understood) -> bool {
    _a_day_out_is_the_question(reading) && _a_stretch_of_time_to_fill(reading)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.plans:<lambda>:2220", |_: &FnRef, u: &Understood| _a_party_to_plan(u)),
    ("bot.knowledge.plans:<lambda>:2752", |_: &FnRef, u: &Understood| _a_free_weekend_to_fill(u)),
    ("bot.knowledge.plans:<lambda>:3335", |_: &FnRef, u: &Understood| _a_day_out_to_fill(u)),
    ("bot.knowledge.plans:_a_household_move_or_a_named_move:649", |_: &FnRef, u: &Understood| {
        _a_household_move_or_a_named_move(u)
    }),
    ("bot.knowledge.plans:_a_milestone_said_as_an_age:1769", |_: &FnRef, u: &Understood| _a_milestone_said_as_an_age(u)),
    ("bot.knowledge.plans:_a_rainy_day_is_money:885", |_: &FnRef, u: &Understood| _a_rainy_day_is_money(u)),
    ("bot.knowledge.plans:_a_stretch_of_time_to_fill:763", |_: &FnRef, u: &Understood| _a_stretch_of_time_to_fill(u)),
    ("bot.knowledge.plans:_declutter_fits:185", |_: &FnRef, u: &Understood| _declutter_fits(u)),
    ("bot.knowledge.plans:_not_a_count_of_their_own:51", |_: &FnRef, u: &Understood| _not_a_count_of_their_own(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.plans:_a_free_weekend:669", |_: &FnRef, m: &mut Memory, u: &Understood| _a_free_weekend(m, u)),
    ("bot.knowledge.plans:_a_piece_of_baby_gear:1626", |_: &FnRef, m: &mut Memory, u: &Understood| {
        _a_piece_of_baby_gear(m, u)
    }),
    ("bot.knowledge.plans:_a_room_plan:1873", |_: &FnRef, m: &mut Memory, u: &Understood| _a_room_plan(m, u)),
    ("bot.knowledge.plans:_dish_for_tonight:1668", |_: &FnRef, m: &mut Memory, u: &Understood| _dish_for_tonight(m, u)),
    ("bot.knowledge.plans:_free_time:1448", |_: &FnRef, m: &mut Memory, u: &Understood| _free_time(m, u)),
    ("bot.knowledge.plans:_party_plan:1794", |_: &FnRef, m: &mut Memory, u: &Understood| _party_plan(m, u)),
    ("bot.knowledge.plans:_try_once:312", |_: &FnRef, m: &mut Memory, u: &Understood| _try_once(m, u)),
    ("bot.knowledge.plans:_try_once_more:327", |_: &FnRef, m: &mut Memory, u: &Understood| _try_once_more(m, u)),
    ("bot.knowledge.plans:_wedding_plan:345", |_: &FnRef, m: &mut Memory, u: &Understood| _wedding_plan(m, u)),
];
