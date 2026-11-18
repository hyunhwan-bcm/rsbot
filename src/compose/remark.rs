//! Port of `bot/compose/remark.py`: the reply to an ordinary statement that says what the person
//! means - a plan, a report of something that happened, a regret, a fact about one of their
//! people or a thing they have, something arranged, an interest.
#![allow(non_snake_case, clippy::all)]

use crate::compose::fallback_chain::{caught, py_format};
use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.remark.", stringify!($name))));)*
    };
}
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.compose.remark.", stringify!($name))));)*
    };
}
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.remark.", stringify!($name))));)*
    };
}
macro_rules! strings {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<String> =
            LazyLock::new(|| data::string(concat!("bot.compose.remark.", stringify!($name))));)*
    };
}

regexes!(
    BOOKED, DEPENDANT_WANT, ASKED_OF_THE_PERSON, WANTS_TO_BE, A_LOSS_IN_THE_LINE, A_CONDOLENCE, SETBACK, RISE, MISSED,
    OWN_SLIP, DEADLINE_LATER, PRESENT_SETBACK, BURGLED, BURGLED_HOME, BURGLED_SOMEONE_ELSE, BURGLED_NOT_THEIRS,
    BARE_SETBACK, NEVER_CAME_SUBJECT, DONE_TO_ME, SMASHED, SMASHED_OF_A_THING, A_GROUP_OF_PEOPLE, A_PERSON_OF_THEIRS,
    A_PERSON_LOST, CRACKED_A_JOKE, A_THING_CARRIED_OFF, A_NIGHT_OUT, STARTED_VERDICT, STARTED_GONE_BADLY, AGAIN,
    STARTED, START_AHEAD, START_PAST, START_PRESENT, START_CLOCK, TAKEN_FROM_THEM, NOT_A_LOSS, TOOK_IT_IN, LOSS_FOLLOW,
    STARTED_WELL, FIRST_STRETCH, REGRET, FIRST_PERSON, PRACTICAL_ASK, POLITE, FUTURE_REGRET, SHOULDNT_HAVE, PUT_DOWN,
    _OWN_SHOULD, _HARM_AT_PERSON, WISH_I, THEIR_NAME_AGE, THEIR_NAME, THEIR_AGE, YEAR_UNIT, THEIR_PLACE, THEIR_JOB,
    THEIR_ELDER_AGE, THEIR_CALLED, THEIR_NAME_SO, THEIR_LIKING, THEIR_DISLIKE, THEIR_HATE, THEIR_MOOD_WITH_YOU,
    GOING_ON, A_HELD_NAME_AND_AN_AGE, A_MILESTONE_TAIL, OWN_HAVE, OWN_ELECTRIC, OWN_NOT_THE_THING, OWN_TWO_THINGS,
    OWN_FAULT, OWN_TIME, OWN_TAIL, MARRIAGE_IN_TROUBLE, BAD_WITH_THEM, QUIET_BETWEEN_THEM, ROWS_BETWEEN_THEM,
    RELATION_GONE_WRONG, NOT_A_RELATION_TROUBLE, ARRANGED, DOUBLE_BOOKED, WENT_WRONG, TURNED, QUESTION_FIRST,
    THE_PERSON_WITH_IT, HAD_IT_BOOKED,
);
word_sets!(
    OTHER_PEOPLE, STEP_NOUNS, BIGGER_PLAN, ARRANGEMENT, PLANNED_VERBS, NOT_PLAIN, MARKED_DATE, ASKS, A_WEAPON,
    WEAPON_WORDS, PRICE, NOT_A_BREAK_IN, REPORT_BODY, REPORT_FEELING, NO_REGRET, BABY_WORDS, NOT_A_NAME_AFTER_CALLED,
    NOT_A_THING, NOT_A_TASTE_AS_A_THING, SHE_PEOPLE, HE_PEOPLE, A_CHILD_NAMED, OWN_SPACES, OWN_KIT, OWN_KIT_NAMES,
    THOUGHT_OBJECT, NOT_AN_INTEREST,
);
pools!(
    A_THING_TAKEN, PLAN, GOING_TO, COUNTERFACTUAL, CRISIS, DOUBT, BARRIER, PLAN_REPLIES_PLAIN, PLAN_REPLIES,
    NOT_A_WEAPON, DEPENDANT_WANT_REPLIES, LOSS_FOLLOW_REPLIES, SETBACK_REPLIES, AGAIN_REPLIES, STARTED_REPLIES,
    STARTED_AHEAD_REPLIES, BURGLED_REPLIES, BURGLED_FOR_SOMEONE_REPLIES, SLIP_REPLIES, STARTED_FINE_REPLIES,
    STARTED_BADLY_REPLIES, REGRET_LINES, REGRET_LINES_PLAIN, THEIR_JOB_LINES, THEIR_ELDER_LINES, THE_THING_IS_A_CLAUSE,
    THEIR_DISLIKE_LINES, THEIR_HATE_LINES, THEY_PERSON, SHE_PERSON, HE_PERSON, THEIR_MOOD_WITH_YOU_LINES,
    GOING_ON_LINES, THEIR_NAME_LINES, THEIR_NAME_GROWN_LINES, THEIR_NAME_AGE_LINES, THEIR_NAMES_LINES,
    THEIR_BABY_LINES, THEIR_SMALL_LINES, THEIR_CHILD_LINES, THEIR_TEEN_LINES, THEIR_GROWN_LINES, THEIR_PLACE_LINES,
    THEIR_LIKING_LINES, AGE_PUT_RIGHT, AGE_FIRST_TOLD, AGE_ALREADY_HAD, OWN_ASKS, OWN_KIT_ASKS, OWN_ARTICLE,
    THE_QUIET_LINES, THE_ROWS_LINES, THE_RELATION_LINES, BOOKED_REPLIES, INTEREST_DOING, INTEREST_STARTED,
    INTEREST_DOING_PLAIN, INTEREST_STARTED_PLAIN, INTEREST_BACK, INTEREST_BACK_PLAIN,
);
strings!(OWN_SPACES_PLAIN, OWN_KIT_PLAIN, OWN_ELECTRIC_CAR, _RELATION_PERSON);

static AGE_WORDS: LazyLock<HashMap<String, i64>> = LazyLock::new(|| {
    data::get("bot.compose.remark.AGE_WORDS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().expect("an int"))).collect()
});
static PLACE_CODES: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.compose.remark.PLACE_CODES"));
static OWN_FACTS: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.compose.remark.OWN_FACTS"));
static ALREADY_DATED: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.compose.remark.ALREADY_DATED"));
static TASTE_TOO_LONG: LazyLock<usize> =
    LazyLock::new(|| data::get("bot.compose.remark.TASTE_TOO_LONG").as_int().expect("an int") as usize);
/// `facts.JOB_WORDS`, `facts.FUTURE`, `facts.DAYS | facts.MONTHS`.
static JOB_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.JOB_WORDS"));
static FUTURE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.facts.FUTURE"));
static DAYS_AND_MONTHS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let mut set = data::set("bot.memory.facts.DAYS");
    set.extend(data::set("bot.memory.facts.MONTHS"));
    set
});

static BURGLED_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:burgled|burglary)\b", crate::py::re::I));
static BROKEN_INTO_US: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(?:i|we|they)\s+(?:was|were|got|have\s+been|had\s+been)\s+(?:broken\s+into|broken\s+in)\b", 0));
static ASKS_THE_BOT: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?:do|does|did|would|have|are|is|was|were|can|could)\s+(?:you|u)\b", 0));
static NO_REGRETS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:no|dont|don't|never|not)\s+regrets?\b", 0));
static REGRETS_NOTHING: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bregrets?\s+(?:nothing|none)\b", 0));
static A_LEADING_ARTICLE: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:the|a|an|his|her|their|its) ", 0));
static MY_OR_OUR: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:my|our)\b", 0));
static A_JOB_SPLIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"[\s'-]+", 0));
static MONTHS_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(r"months?|mths?|\bmth\b", 0));
static THEIR_RELATION: LazyLock<Re> = LazyLock::new(|| Re::new(&format!(r"\b(?:my|our)\s+({})\b", *_RELATION_PERSON), 0));

/// `compose.pick(options, memory, seed_text)`, through the one wrapper this file has.
fn pick(options: &[String], memory: &Memory, clean: &str) -> String {
    crate::compose::pick(options, &crate::compose::fallback::memory_value(memory), clean)
}

/// `set(words) & names` is not empty.
fn any_in(words: &[String], names: &HashSet<String>) -> bool {
    words.iter().any(|w| names.contains(w))
}

/// `s[:1].upper() + s[1:]`.
fn first_upper(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => text::upper(&c.to_string()) + chars.as_str(),
        None => String::new(),
    }
}

/// `int(said) if said.isdigit() else AGE_WORDS[said]`.
fn age_of(said: &str) -> i64 {
    if text::is_digit(said) {
        return digits_value(said);
    }
    *AGE_WORDS.get(said).unwrap_or_else(|| panic!("KeyError: {said:?}"))
}

/// `int(s)` for a run of Unicode decimal digits: each digit's value is its place in its run of
/// ten, found by walking back over `\d`.
fn digits_value(s: &str) -> i64 {
    static A_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d", 0));
    let is_digit = |c: char| A_DIGIT.fullmatch(&c.to_string()).is_some();
    let mut n: i64 = 0;
    for c in s.chars() {
        let d = match c.to_digit(10) {
            Some(d) => d,
            None => {
                let mut start = c as u32;
                while start > 0 && char::from_u32(start - 1).is_some_and(is_digit) {
                    start -= 1;
                }
                (c as u32 - start) % 10
            }
        };
        n = n.saturating_mul(10).saturating_add(d as i64);
    }
    n
}

/// Python: bot/compose/remark.py:411 `_a_good_night_out(clean)`
/// True when "smashed" means drunk, so the setback reading stands down.
pub fn _a_good_night_out(clean: &str) -> bool {
    if !SMASHED.is_match(clean) {
        return false;
    }
    if !SMASHED_OF_A_THING.is_match(clean) {
        return true;
    }
    if A_GROUP_OF_PEOPLE.is_match(clean) {
        return A_NIGHT_OUT.is_match(clean);
    }
    false
}

/// Python: bot/compose/remark.py:427 `done_to_me(clean)`
/// True when the line reports something done to a thing of theirs, for the pools that cheer.
pub fn done_to_me(clean: &str) -> bool {
    if !DONE_TO_ME.is_match(clean) || _a_good_night_out(clean) || CRACKED_A_JOKE.is_match(clean) {
        return false;
    }
    if A_PERSON_OF_THEIRS.is_match(clean) && !A_PERSON_LOST.is_match(clean) && !A_THING_CARRIED_OFF.is_match(clean) {
        return false;
    }
    true
}

/// Python: bot/compose/remark.py:544 `_a_thing_taken(clean)`
/// True for a report of something of theirs taken by somebody else.
pub fn _a_thing_taken(clean: &str) -> bool {
    let text_ = text::squash(&text::lower(clean));
    if text_.is_empty() || !TAKEN_FROM_THEM.is_match(&text_) {
        return false;
    }
    !NOT_A_LOSS.is_match(&text_) && !TOOK_IT_IN.is_match(&text_)
}

/// Python: bot/compose/remark.py:574 `_the_loss_was_asked(memory)`
/// True when the bot's last reply was the loss pool asking its question.
pub fn _the_loss_was_asked(memory: &Memory) -> bool {
    let last = memory.turns.last().map(|t| text::strip(&t.bot).to_string()).unwrap_or_default();
    A_THING_TAKEN.contains(&last)
}

/// Python: bot/compose/remark.py:581 `_the_answers_take_it(clean)`
/// Whether the knowledge module answers this line itself (asked on an empty memory); a module
/// that fails answers nothing.
pub fn _the_answers_take_it(clean: &str) -> bool {
    caught(|| crate::modules::answers::respond(clean, &mut Memory::new()).is_some()).unwrap_or(false)
}

/// Python: bot/compose/remark.py:672 `_missed_a_person(clean)`
/// True when "missed" is about a person: "i missed my dad" is not a bus that went without me.
pub fn _missed_a_person(clean: &str) -> bool {
    MISSED.finditer(clean).iter().any(|found| found.get(1).is_some_and(|w| OTHER_PEOPLE.contains(w)))
}

/// Python: bot/compose/remark.py:680 `reads_as_report(clean, words)`
/// 'burgled', 'started', 'slip', 'setback', 'again' or None: the shape of the thing they told us.
pub fn reads_as_report(clean: &str, words: &[String]) -> Option<String> {
    let shape = |s: &str| Some(s.to_string());
    if clean.is_empty() || clean.contains('?') {
        return None;
    }
    if any_in(words, &REPORT_FEELING) || any_in(words, &REPORT_BODY) {
        return None;
    }
    if _missed_a_person(clean) {
        return None;
    }
    if _reads_as_a_break_in(clean, words) {
        return shape("burgled");
    }
    if STARTED.is_match(clean) {
        return shape("started");
    }
    if OWN_SLIP.is_match(clean) {
        return shape("slip");
    }
    // A missed filing is the person's own slip even with a word between "missed" and the deadline.
    if MISSED.is_match(clean) && DEADLINE_LATER.is_match(clean) {
        return shape("slip");
    }
    if BARE_SETBACK.is_match(clean) && NEVER_CAME_SUBJECT.is_match(clean) {
        // "you never showed up" is somebody's hurt, not a nuisance.
        return if AGAIN.is_match(clean) { shape("again") } else { None };
    }
    if SETBACK.is_match(clean) || BARE_SETBACK.is_match(clean) || done_to_me(clean) || (RISE.is_match(clean) && any_in(words, &PRICE)) {
        return if AGAIN.is_match(clean) { shape("again") } else { shape("setback") };
    }
    if AGAIN.is_match(clean) && PRESENT_SETBACK.is_match(clean) {
        return shape("again");
    }
    None
}

/// Python: bot/compose/remark.py:712 `_still_to_come(clean)`
/// A start the person is telling the bot about before it has happened.
pub fn _still_to_come(clean: &str) -> bool {
    if START_PAST.is_match(clean) || START_CLOCK.is_match(clean) {
        return false;
    }
    START_AHEAD.is_match(clean) || START_PRESENT.is_match(clean)
}

/// Python: bot/compose/remark.py:733 `_started_badly(clean)`
/// True when a start is said with something wrong in it and nothing after to say it came right.
pub fn _started_badly(clean: &str) -> bool {
    let Some(bad) = STARTED_GONE_BADLY.search(clean) else { return false };
    STARTED_WELL.search_at(clean, bad.end()).is_none()
}

/// Python: bot/compose/remark.py:745 `_reads_as_a_break_in(clean, words)`
/// True when the line is a home being entered, rather than a thing to repair.
pub fn _reads_as_a_break_in(clean: &str, words: &[String]) -> bool {
    if !BURGLED.is_match(clean) {
        return false;
    }
    if BURGLED_NOT_THEIRS.is_match(clean) && !BURGLED_SOMEONE_ELSE.is_match(clean) {
        return false;
    }
    if BURGLED_WORD.is_match(clean) {
        return true;
    }
    if any_in(words, &NOT_A_BREAK_IN) {
        return false;
    }
    BURGLED_HOME.is_match(clean) || BROKEN_INTO_US.is_match(clean)
}

/// Python: bot/compose/remark.py:765 `break_in_reply_for(clean, words, memory)`
/// The reply to a home being broken into, or None when the line is not one.
pub fn break_in_reply_for(clean: &str, words: &[String], memory: &Memory) -> Option<String> {
    if !_reads_as_a_break_in(clean, words) {
        return None;
    }
    let pool = if BURGLED_SOMEONE_ELSE.is_match(clean) { &*BURGLED_FOR_SOMEONE_REPLIES } else { &*BURGLED_REPLIES };
    Some(pick(pool, memory, clean))
}

/// Python: bot/compose/remark.py:779 `report_reply_for(clean, words, memory)`
/// A reply to a report of something that happened, or None to leave it to the caller.
pub fn report_reply_for(clean: &str, words: &[String], memory: &Memory) -> Option<String> {
    if any_in(words, &NOT_PLAIN) || _has_phrase(clean, &CRISIS) {
        return None;
    }
    if _has_phrase(clean, &COUNTERFACTUAL) || any_in(words, &MARKED_DATE) {
        return None;
    }
    if any_in(words, &ASKS) || _has_phrase(clean, &DOUBT) {
        return None;
    }
    let shape = reads_as_report(clean, words);
    if shape.as_deref() == Some("burgled") {
        // A home entered is not a thing to repair, so this comes before the trouble reading.
        return break_in_reply_for(clean, words, memory);
    }
    if _a_thing_taken(clean) && !_the_answers_take_it(clean) {
        // The commonest thefts have no shape the report reader knows.
        return Some(pick(&A_THING_TAKEN, memory, clean));
    }
    if _the_loss_was_asked(memory) && !any_in(words, &ASKS) && LOSS_FOLLOW.is_match(clean) {
        return Some(pick(&LOSS_FOLLOW_REPLIES, memory, clean));
    }
    let shape = shape?;
    if shape != "started" && crate::compose::trouble::reads_as_trouble(clean, Some(words), clean).is_some_and(|s| !s.is_empty()) {
        // A thing of theirs that has gone wrong is trouble.py's turn.
        return None;
    }
    if shape == "started" && _started_badly(clean) {
        if FIRST_STRETCH.is_match(clean) {
            // "my first day went badly" is about the day: the setback reader answers it.
            return None;
        }
        return Some(pick(&STARTED_BADLY_REPLIES, memory, clean));
    }
    if shape == "started" && STARTED_VERDICT.is_match(clean) {
        return Some(pick(&STARTED_FINE_REPLIES, memory, clean));
    }
    if shape == "started" && _still_to_come(clean) {
        // "i start my new job monday" is news about what is coming.
        return Some(pick(&STARTED_AHEAD_REPLIES, memory, clean));
    }
    let pool: &Vec<String> = match shape.as_str() {
        "setback" => &SETBACK_REPLIES,
        "again" => &AGAIN_REPLIES,
        "started" => &STARTED_REPLIES,
        "slip" => &SLIP_REPLIES,
        other => panic!("KeyError: {other:?}"),
    };
    let mut pool = pool.clone();
    if _asked_last(memory) && shape != "started" {
        // The bot has just asked something; a report is not an interview, so answer it plainly.
        pool = without_questions(&pool);
    }
    Some(pick(&pool, memory, clean))
}

/// `tuple(o for o in pool if not o.endswith("?")) or pool`.
fn without_questions(pool: &[String]) -> Vec<String> {
    let kept: Vec<String> = pool.iter().filter(|o| !o.ends_with('?')).cloned().collect();
    if kept.is_empty() { pool.to_vec() } else { kept }
}

/// Python: bot/compose/remark.py:942 `reads_as_regret(clean, words=())`
/// True when the line is a regret the person is telling the bot about.
pub fn reads_as_regret(clean: &str, words: &[String]) -> bool {
    if clean.is_empty() || clean.contains('?') {
        return false; // a question is not a regret
    }
    if ASKS_THE_BOT.match_(clean).is_some() {
        return false; // a regret the person is asking the bot about is not theirs to be met
    }
    if any_in(words, &NO_REGRET) || any_in(words, &NOT_PLAIN) {
        return false;
    }
    if _has_phrase(clean, &CRISIS) || _has_phrase(clean, &COUNTERFACTUAL) {
        return false;
    }
    if !(FIRST_PERSON.is_match(clean) || WISH_I.is_match(clean) || (_OWN_SHOULD.is_match(clean) && SHOULDNT_HAVE.is_match(clean))) {
        return false; // somebody else's regret is not the person's to work through
    }
    if POLITE.is_match(clean) {
        return false; // "i regret to say i cant come" is a polite apology
    }
    if PRACTICAL_ASK.is_match(clean) {
        // "can i return a car i regret buying" asks what can be done, and the entries answer it.
        return false;
    }
    if FUTURE_REGRET.is_match(clean) {
        return false; // "im going to regret this" is a thing not done yet
    }
    if PUT_DOWN.is_match(clean) {
        return false; // a regret about a pet put to sleep is the loss band's
    }
    if _HARM_AT_PERSON.is_match(clean) {
        return false; // a regret that names the person it was done to has its own card
    }
    if NO_REGRETS.is_match(clean) {
        return false; // "no regrets" is the opposite, and "i dont regret it" is a defence
    }
    if REGRETS_NOTHING.is_match(clean) || clean.contains("regret a thing") {
        return false; // "i regret nothing" is not a regret
    }
    REGRET.is_match(clean) || SHOULDNT_HAVE.is_match(clean)
}

/// Python: bot/compose/remark.py:976 `regret_reply_for(clean, words, memory, repaired="")`
/// A reply to a regret, or None to leave the line to the caller. The typed line is asked first,
/// the repaired one (`plain`) only when the typed one is not a regret.
pub fn regret_reply_for(clean: &str, words: &[String], memory: &Memory, repaired: &str) -> Option<String> {
    let mut said = clean;
    let mut words: Vec<String> = if words.is_empty() { text::split(clean) } else { words.to_vec() };
    if !reads_as_regret(clean, &words) {
        if !(!repaired.is_empty() && repaired != clean && reads_as_regret(repaired, &text::split(repaired))) {
            return None;
        }
        said = repaired;
        words = text::split(repaired);
    }
    if any_in(&words, &ASKS) || _has_phrase(said, &DOUBT) {
        return None;
    }
    let pool = if _asked_last(memory) { &*REGRET_LINES_PLAIN } else { &*REGRET_LINES };
    Some(pick(pool, memory, said))
}

/// Python: bot/compose/remark.py:1194 `_a_taste_they_refuse(clean, memory)`
/// A reply to a line that is only what one of their people will not have, or None.
pub fn _a_taste_they_refuse(clean: &str, memory: &Memory) -> Option<String> {
    if clean.contains('?') {
        return None;
    }
    for (pattern, pool) in [(&*THEIR_DISLIKE, &*THEIR_DISLIKE_LINES), (&*THEIR_HATE, &*THEIR_HATE_LINES)] {
        let Some(found) = pattern.match_(clean) else { continue };
        let thing = text::squash(&text::lower(found.name_str("thing")));
        // The article is not part of the thing for the purpose of asking whether there is one.
        let bare = A_LEADING_ARTICLE.subn("", &thing, 1);
        // A clause after the thing means the thing is not the whole of it.
        let padded = format!(" {thing} ");
        if THE_THING_IS_A_CLAUSE.iter().any(|hang| padded.contains(hang.as_str())) {
            continue;
        }
        let bare_words = text::split(&bare);
        if bare.is_empty() || bare_words.len() > *TASTE_TOO_LONG {
            continue;
        }
        if NOT_A_THING.contains(&bare) || any_in(&bare_words, &NOT_A_TASTE_AS_A_THING) {
            continue;
        }
        if AGE_WORDS.contains_key(&bare_words[0]) || text::is_digit(&bare) {
            continue;
        }
        // The thing is said back in the person's own words, so their "my" turns into "your".
        let thing = MY_OR_OUR.sub("your", &thing);
        let person = text::strip(found.name_str("person")).to_string();
        let lines: Vec<String> = pool
            .iter()
            .map(|line| fill_person(&line.replace("{Thing}", &first_upper(&thing)).replace("{thing}", &thing), &person))
            .collect();
        return Some(pick(&lines, memory, clean));
    }
    None
}

/// Python: bot/compose/remark.py:1251 `person_pronouns(person_word)`
/// The pronoun, the object and the verb forms for the person word in the line.
pub fn person_pronouns(person_word: &str) -> [String; 5] {
    let word = text::lower(text::strip(person_word));
    let set: &Vec<String> = if SHE_PEOPLE.contains(&word) {
        &SHE_PERSON
    } else if HE_PEOPLE.contains(&word) {
        &HE_PERSON
    } else {
        &THEY_PERSON
    };
    [set[0].clone(), set[1].clone(), set[2].clone(), set[3].clone(), set[4].clone()]
}

/// Python: bot/compose/remark.py:1261 `fill_person(line, person_word)`
/// The pool line with the person's own pronouns and the verb that agrees with them.
pub fn fill_person(line: &str, person_word: &str) -> String {
    let [they, them, their, do_, are] = person_pronouns(person_word);
    line.replace("{they}", &they)
        .replace("{them}", &them)
        .replace("{their}", &their)
        .replace("{do}", &do_)
        .replace("{are}", &are)
        .replace("{They}", &text::capitalize(&they))
        .replace("{Them}", &text::capitalize(&them))
        .replace("{Their}", &text::capitalize(&their))
        .replace("{Do}", &text::capitalize(&do_))
        .replace("{Are}", &text::capitalize(&are))
}

/// `int(value or 0)` for a number kept in memory.
fn py_int(value: Option<&Value>) -> i64 {
    match value {
        Some(Value::Int(n)) => *n,
        Some(Value::Bool(b)) => *b as i64,
        Some(Value::Float(x)) => x.trunc() as i64,
        Some(Value::Str(s)) if !s.is_empty() => text::strip(s).parse().unwrap_or_else(|_| panic!("ValueError: invalid literal for int(): {s:?}")),
        _ => 0,
    }
}

/// Python: bot/compose/remark.py:1337 `their_mood_going_on_reply_for(clean, memory)`
/// "it happens most weeks" after a mood with them, or None to leave the line to the caller.
pub fn their_mood_going_on_reply_for(clean: &str, memory: &Memory) -> Option<String> {
    if clean.is_empty() || clean.contains('?') {
        return None;
    }
    let kept = memory.get("mood_with_you").filter(|k| k.is_dict())?;
    // The note is for the turn right after the mood, or the one after that.
    let gap = memory.turns.len() as i64 - py_int(kept.get("at"));
    if !(0..=2).contains(&gap) {
        return None;
    }
    if !GOING_ON.is_match(clean) {
        return None;
    }
    if _has_phrase(clean, &CRISIS) {
        return None;
    }
    let person_word = match kept.get("person") {
        Some(Value::Str(s)) => s.to_string(),
        _ => String::new(),
    };
    let person = _pronoun_person(&person_word);
    let lines: Vec<String> = GOING_ON_LINES.iter().map(|line| fill_person(line, &person)).collect();
    Some(pick(&lines, memory, clean))
}

/// Python: bot/compose/remark.py:1359 `_pronoun_person(person)`
/// The relation whose pronouns are the ones in the line: "he snapped" is a husband here.
pub fn _pronoun_person(person: &str) -> String {
    match person {
        "he" => "husband".to_string(),
        "she" => "wife".to_string(),
        "they" => String::new(),
        other => other.to_string(),
    }
}

/// Python: bot/compose/remark.py:1369 `their_mood_with_you_reply_for(clean, words, memory)`
/// A reply to "somebody was off with me", or None to leave the line to whoever owns it. The
/// person is kept in memory for one turn, so the length of it is answered on the line after.
pub fn their_mood_with_you_reply_for(clean: &str, _words: &[String], memory: &mut Memory) -> Option<String> {
    if clean.is_empty() || clean.contains('?') {
        return None;
    }
    if !THEIR_MOOD_WITH_YOU.is_match(clean) {
        return None;
    }
    let said = text::squash(clean);
    if _has_phrase(&said, &CRISIS) || _has_phrase(&said, &NOT_PLAIN.iter().cloned().collect::<Vec<_>>()) {
        return None;
    }
    const PEOPLE: [&str; 35] = [
        "mum", "mom", "mam", "mother", "dad", "father", "husband", "wife", "partner", "boyfriend", "girlfriend", "sister",
        "brother", "son", "daughter", "friend", "mate", "boss", "manager", "colleague", "flatmate", "housemate",
        "neighbour", "neighbor", "nan", "nana", "gran", "granny", "grandma", "grandad", "grandpa", "aunt", "uncle",
        "cousin", "he",
    ];
    let mut person = String::new();
    for word in PEOPLE.iter().chain(["she", "they"].iter()) {
        if Re::new(&format!(r"\b{word}\b"), 0).is_match(&said) {
            person = word.to_string();
            break;
        }
    }
    // "he snapped at me" names the person by pronoun, read as the relation whose pronouns they are.
    let person = _pronoun_person(&person);
    let mut note = Value::dict();
    note.set_key("person", person.as_str());
    note.set_key("at", memory.turns.len() as i64);
    memory.set("mood_with_you", note);
    let lines: Vec<String> = THEIR_MOOD_WITH_YOU_LINES.iter().map(|line| fill_person(line, &person)).collect();
    Some(pick(&lines, memory, clean))
}

/// Python: bot/compose/remark.py:1499 `name_a_place(said)`
/// The place as a person would write it, for a pool line that opens with it.
pub fn name_a_place(said: &str) -> String {
    let mut place = text::squash(said);
    let mut lead = "";
    if text::lower(&place).starts_with("the ") && text::len(&place) > 4 {
        lead = "The ";
        place = text::slice(&place, 4, None);
    }
    let words: Vec<String> = text::split_ref(&place)
        .iter()
        .map(|w| PLACE_CODES.get(&text::lower(w)).cloned().unwrap_or_else(|| text::title(w)))
        .collect();
    format!("{lead}{}", words.join(" "))
}

/// Python: bot/compose/remark.py:1508 `fill_age(line, n, months=False)`
/// The pool line with the age filled in: a one-year-old is "1 year old", not "1 years".
pub fn fill_age(line: &str, n: i64, months: bool) -> String {
    let s = if n != 1 { "s" } else { "" };
    let years = format!("{n} year{s} old");
    let count = if months { format!("{n} month{s}") } else { years.clone() };
    line.replace("{y}", &years).replace("{m}", &count).replace("{n}", &n.to_string())
}

/// Python: bot/compose/remark.py:1515 `_age_pool(n, months=False)`
/// The pool for a child of this age: a baby is a different conversation from a grown one.
pub fn _age_pool(n: i64, months: bool) -> &'static Vec<String> {
    if months && n <= 30 {
        return &THEIR_BABY_LINES;
    }
    if n < 5 {
        return &THEIR_SMALL_LINES;
    }
    if n < 13 {
        return &THEIR_CHILD_LINES;
    }
    if n < 18 {
        return &THEIR_TEEN_LINES;
    }
    &THEIR_GROWN_LINES
}

/// Python: bot/compose/remark.py:1528 `_the_job_in(phrase)`
/// The job the phrase names, written the way a person would, or None if it names no job.
pub fn _the_job_in(phrase: &str) -> Option<String> {
    let words: Vec<String> = A_JOB_SPLIT.split(text::strip(phrase)).into_iter().filter(|w| !w.is_empty()).collect();
    if words.is_empty() || words.len() > 2 {
        return None;
    }
    if !any_in(&words, &JOB_WORDS) {
        return None;
    }
    Some(text::title(&words.join(" ")))
}

/// Python: bot/compose/remark.py:1543 `_pick_filled(lines, memory, clean, fill)`
/// One of the pool, chosen after the line is filled in, so the picker sees the repeat.
pub fn _pick_filled(lines: &[String], memory: &Memory, clean: &str, fill: impl Fn(&str) -> String) -> String {
    let filled: Vec<String> = lines.iter().map(|line| fill(line)).collect();
    pick(&filled, memory, clean)
}

/// `str(entry.get("name") or "").strip().lower()`.
fn entry_name(entry: &Value) -> String {
    match entry.get("name") {
        Some(Value::Str(s)) => text::lower(text::strip(s)),
        _ => String::new(),
    }
}

/// Python: bot/compose/remark.py:1590 `_the_people_in(memory, name)`
/// Every entry the store holds for that name, the people and the animals.
pub fn _the_people_in(memory: &Memory, name: &str) -> Vec<Value> {
    let mut found = Vec::new();
    if let Some(others) = memory.get("others") {
        found.extend(others.items().iter().filter(|e| e.is_dict() && entry_name(e) == name).cloned());
    }
    if let Some(pets) = memory.get("user").filter(|u| u.is_dict()).and_then(|u| u.get("pets")) {
        found.extend(pets.items().iter().filter(|e| e.is_dict() && entry_name(e) == name).cloned());
    }
    found
}

/// Python: bot/compose/remark.py:1604 `_the_age_i_had(name, memory)`
/// The age an earlier turn gave that name, or None: the store overwrites it in place.
pub fn _the_age_i_had(name: &str, memory: &Memory) -> Option<i64> {
    let mut old = None;
    let Some(log) = memory.get("facts_log") else { return None };
    for fact in log.items() {
        let Some(value) = fact.get("value").filter(|v| v.is_dict()) else { continue };
        if entry_name(value) != name {
            continue;
        }
        // `isinstance(age, int) and age`: a bool is an int in Python.
        match value.get("age") {
            Some(Value::Int(n)) if *n != 0 => old = Some(*n),
            Some(Value::Bool(true)) => old = Some(1),
            _ => {}
        }
    }
    old
}

/// Python: bot/compose/remark.py:1619 `_an_age_put_right(clean, memory)`
/// A known child or animal, named alone with a new age, said back with the age it had.
pub fn _an_age_put_right(clean: &str, memory: &Memory) -> Option<String> {
    if A_MILESTONE_TAIL.is_match(clean) {
        return None;
    }
    let found_match = A_HELD_NAME_AND_AN_AGE.match_(clean)?;
    let name = found_match.name_str("name");
    let found = _the_people_in(memory, name);
    let first = found.first()?;
    let n = age_of(found_match.name_str("n"));
    if n == 0 || n > 120 {
        return None;
    }
    let held = match first.get("name") {
        Some(Value::Str(s)) if !s.is_empty() => s.to_string(),
        _ => name.to_string(),
    };
    let spelled = text::title(text::strip(&held));
    let n_said = n.to_string();
    match _the_age_i_had(name, memory) {
        None => Some(py_format(&pick(&AGE_FIRST_TOLD, memory, clean), &[("name", &spelled), ("n", &n_said)])),
        Some(old) if old == n => Some(py_format(&pick(&AGE_ALREADY_HAD, memory, clean), &[("name", &spelled), ("n", &n_said)])),
        Some(old) => Some(py_format(&pick(&AGE_PUT_RIGHT, memory, clean), &[("name", &spelled), ("n", &n_said), ("old", &old.to_string())])),
    }
}

/// Python: bot/compose/remark.py:1643 `their_people_reply_for(clean, words, memory)`
/// A reply to a plain fact about one of their people, or None to leave it to the caller.
pub fn their_people_reply_for(clean: &str, words: &[String], memory: &Memory) -> Option<String> {
    if clean.is_empty() || clean.contains('?') {
        return None;
    }
    // A fact about one of their people is as often said after a leader as on its own.
    let led = crate::understanding::normalize::without_leaders(clean);
    let clean = if led.is_empty() { clean.to_string() } else { led };
    let clean = clean.as_str();
    if any_in(words, &NOT_PLAIN) || _has_phrase(clean, &CRISIS) {
        return None;
    }
    if let Some(refused) = _a_taste_they_refuse(clean, memory).filter(|s| !s.is_empty()) {
        return Some(refused);
    }
    if let Some(put_right) = _an_age_put_right(clean, memory).filter(|s| !s.is_empty()) {
        return Some(put_right);
    }
    if let Some(found) = THEIR_ELDER_AGE.match_(clean) {
        let n = age_of(found.name_str("n"));
        let person = found.name_str("person");
        return Some(_pick_filled(&THEIR_ELDER_LINES, memory, clean, |line| fill_person(&fill_age(line, n, false), person)));
    }
    if let Some(found) = THEIR_JOB.match_(clean) {
        if let Some(job) = _the_job_in(found.name_str("phrase")) {
            let person = found.name_str("person");
            return Some(_pick_filled(&THEIR_JOB_LINES, memory, clean, |line| fill_person(&line.replace("{Job}", &job), person)));
        }
    }
    if let Some(found) = THEIR_PLACE.match_(clean) {
        let place = name_a_place(found.name_str("place"));
        let person = found.name_str("person");
        return Some(_pick_filled(&THEIR_PLACE_LINES, memory, clean, |line| fill_person(&line.replace("{place}", &place), person)));
    }
    if let Some(found) = THEIR_NAME_AGE.match_(clean) {
        let name = text::title(text::strip(found.name_str("name")));
        let n = digits_value(found.name_str("age")).to_string();
        let person = found.name_str("person");
        return Some(_pick_filled(&THEIR_NAME_AGE_LINES, memory, clean, |line| {
            fill_person(&line.replace("{name}", &name).replace("{n}", &n), person)
        }));
    }
    if let Some(found) = THEIR_AGE.match_(clean) {
        let mut months = MONTHS_SAID.is_match(found.as_str());
        let said = found.name_str("n");
        let n = age_of(said);
        let person = found.name_str("person");
        if !months && text::is_digit(said) && BABY_WORDS.contains(person) && !YEAR_UNIT.is_match(found.as_str()) {
            // Bare "my baby is 3" is three months; a typed digit only, and a unit of its own wins.
            if n > 24 {
                return None;
            }
            months = true;
        }
        return Some(_pick_filled(_age_pool(n, months), memory, clean, |line| fill_person(&fill_age(line, n, months), person)));
    }
    if let Some(found) = THEIR_NAME.match_(clean) {
        let name = text::title(text::strip(found.name_str("name")));
        let person_word = found.name_str("person");
        let person = text::lower(text::strip(person_word));
        let pool = if A_CHILD_NAMED.contains(&person) { &*THEIR_NAME_LINES } else { &*THEIR_NAME_GROWN_LINES };
        return Some(_pick_filled(pool, memory, clean, |line| fill_person(&line.replace("{name}", &name), person_word)));
    }
    // The same two facts with the be-verb left out, read last.
    if let Some(found) = THEIR_NAME_SO.match_(clean) {
        let name = text::strip(found.name_str("name")).to_string();
        if crate::memory::facts::a_name_could_be(&text::lower(&name)) {
            let line = pick(&THEIR_NAME_AGE_LINES, memory, clean)
                .replace("{name}", &text::title(&name))
                .replace("{n}", &digits_value(found.name_str("age")).to_string());
            return Some(fill_person(&line, found.name_str("person")));
        }
    }
    if let Some(found) = THEIR_CALLED.match_(clean) {
        let names: Vec<String> = found.name_str("names").split(" and ").map(|w| text::strip(w).to_string()).collect();
        let all_names = names.iter().all(|w| {
            let low = text::lower(w);
            crate::memory::facts::a_name_could_be(&low) && !NOT_A_NAME_AFTER_CALLED.contains(&low) && !text::is_digit(w)
        });
        if all_names {
            let said = names.iter().map(|w| text::title(w)).collect::<Vec<_>>().join(" and ");
            if names.len() == 2 {
                return Some(pick(&THEIR_NAMES_LINES, memory, clean).replace("{names}", &said));
            }
            if let Some(age) = found.name("age").filter(|a| !a.is_empty()) {
                let line = pick(&THEIR_NAME_AGE_LINES, memory, clean);
                return Some(
                    fill_person(&line, found.name_str("person")).replace("{name}", &said).replace("{n}", &digits_value(age).to_string()),
                );
            }
            let line = pick(&THEIR_NAME_LINES, memory, clean).replace("{name}", &said);
            return Some(fill_person(&line, found.name_str("person")));
        }
    }
    if let Some(found) = THEIR_LIKING.match_(clean) {
        let thing = text::strip(found.name_str("thing")).to_string();
        let first_word = text::split_ref(&thing).first().copied().unwrap_or_else(|| panic!("IndexError: list index out of range"));
        if NOT_A_THING.contains(first_word) {
            return None;
        }
        let padded = format!(" {thing} ");
        if THE_THING_IS_A_CLAUSE.iter().any(|w| padded.contains(w.as_str())) {
            return None;
        }
        let said = first_upper(&thing);
        let person = found.name_str("person");
        return Some(_pick_filled(&THEIR_LIKING_LINES, memory, clean, |line| fill_person(&line.replace("{thing}", &said), person)));
    }
    None
}

/// Python: bot/compose/remark.py:1919 `_the_thing_they_have(clean)`
/// The thing the line says they have, and which class it is in ("space" or "kit"), or None.
pub fn _the_thing_they_have(clean: &str) -> Option<(String, String)> {
    let said = text::squash(text::rstrip_chars(text::strip(&text::lower(clean)), ".!,"));
    if said.is_empty() || said.contains('?') {
        return None;
    }
    let found = OWN_HAVE.match_(&said)?;
    let mut thing = text::strip(found.name("one").filter(|s| !s.is_empty()).or(found.name("two")).unwrap_or("")).to_string();
    for article in OWN_ARTICLE.iter() {
        if let Some(rest) = thing.strip_prefix(article.as_str()) {
            thing = rest.to_string();
            break;
        }
    }
    let thing = text::strip(&OWN_TAIL.sub("", &thing)).to_string();
    let words = text::split_ref(&thing);
    if words.is_empty() || words.len() > 4 {
        return None;
    }
    if OWN_NOT_THE_THING.is_match(&thing) || OWN_FAULT.is_match(&thing) || OWN_TIME.is_match(&thing) {
        return None;
    }
    // Two things in one line are two facts.
    if OWN_TWO_THINGS.is_match(&thing) {
        return None;
    }
    // The class comes from the tail of the phrase, not from any word in it.
    let key = _the_class_word(&thing);
    if key.is_empty() {
        return None;
    }
    let kind = if OWN_SPACES.contains(&key) { "space" } else { "kit" };
    Some((thing, kind.to_string()))
}

/// Python: bot/compose/remark.py:1950 `_the_class_word(thing)`
/// The longest ending of the phrase that names a space or a piece of kit, or "".
pub fn _the_class_word(thing: &str) -> String {
    let words = text::split_ref(thing);
    for size in [3, 2, 1] {
        if words.len() >= size {
            let tail = words[words.len() - size..].join(" ");
            if OWN_SPACES.contains(&tail) || OWN_KIT.contains(&tail) || OWN_KIT_NAMES.contains(&tail) || OWN_FACTS.contains_key(&tail) {
                return tail;
            }
        }
    }
    String::new()
}

/// Python: bot/compose/remark.py:1966 `_article(thing)`
/// A or An, by the sound the first word makes: "an allotment", "a spare room".
pub fn _article(thing: &str) -> String {
    let word = text::lower(text::strip(thing));
    if word.is_empty() {
        return "A".to_string();
    }
    let vowel = word.chars().next().is_some_and(|c| "aeiou".contains(c));
    if vowel && !["one", "uni", "use", "euro"].iter().any(|p| word.starts_with(p)) {
        return "An".to_string();
    }
    "A".to_string()
}

/// Python: bot/compose/remark.py:1976 `_the_fact_about(thing, kind)`
/// The one true thing about this kind of thing, filled in the way the person said it.
pub fn _the_fact_about(thing: &str, kind: &str) -> String {
    let article = _article(thing);
    let key = _the_class_word(thing);
    let mut fact = OWN_FACTS.get(&key).cloned().unwrap_or_default();
    // The kind of car the person named decides which fact is true of it.
    if key == "car" && OWN_ELECTRIC.is_match(thing) {
        fact = OWN_ELECTRIC_CAR.clone();
    }
    if fact.is_empty() {
        fact = if kind == "space" { OWN_SPACES_PLAIN.clone() } else { OWN_KIT_PLAIN.clone() };
    }
    fact.replace("{A}", &article).replace("{thing}", thing)
}

/// Python: bot/compose/remark.py:1996 `_capped(line)`
pub fn _capped(line: &str) -> String {
    first_upper(line)
}

/// Python: bot/compose/remark.py:2000 `a_thing_of_theirs_reply_for(clean, words, memory)`
/// A reply to a plain fact about a thing they have, or None to leave it to the caller.
pub fn a_thing_of_theirs_reply_for(clean: &str, words: &[String], memory: &Memory) -> Option<String> {
    if clean.is_empty() || clean.contains('?') {
        return None;
    }
    if any_in(words, &NOT_PLAIN) || _has_phrase(clean, &CRISIS) {
        return None;
    }
    let (thing, kind) = _the_thing_they_have(clean)?;
    let fact = _capped(&_the_fact_about(&thing, &kind));
    let asks = if kind == "space" { &*OWN_ASKS } else { &*OWN_KIT_ASKS };
    let lines: Vec<String> = asks.iter().map(|ask| format!("{fact}. {ask}")).collect();
    Some(_pick_filled(&lines, memory, clean, |line| line.to_string()))
}

/// Python: bot/compose/remark.py:2085 `the_two_of_them(clean, memory=None)`
/// A reply to a relationship said to be in trouble, or None to leave it to the caller.
pub fn the_two_of_them(clean: &str, memory: &Memory) -> Option<String> {
    if clean.is_empty() || clean.contains('?') {
        return None;
    }
    if NOT_A_RELATION_TROUBLE.is_match(clean) {
        return None;
    }
    let pool: &Vec<String> = if QUIET_BETWEEN_THEM.is_match(clean) {
        &THE_QUIET_LINES
    } else if ROWS_BETWEEN_THEM.is_match(clean) {
        &THE_ROWS_LINES
    } else if MARRIAGE_IN_TROUBLE.is_match(clean) || BAD_WITH_THEM.is_match(clean) || RELATION_GONE_WRONG.is_match(clean) {
        &THE_RELATION_LINES
    } else {
        return None;
    };
    if pool.is_empty() {
        return None;
    }
    let found = THEIR_RELATION.search(clean);
    let (who, pair) = match &found {
        Some(f) => (format!("your {}", f.str(1)), format!("you and your {}", f.str(1))),
        None => ("the two of you".to_string(), "the two of you".to_string()),
    };
    Some(_pick_filled(pool, memory, clean, |line| line.replace("{who}", &who).replace("{pair}", &pair)))
}

/// Python: bot/compose/remark.py:2114 `_has_phrase(clean, phrases)`
pub fn _has_phrase(clean: &str, phrases: &[String]) -> bool {
    let padded = format!(" {clean} ");
    phrases.iter().any(|phrase| padded.contains(&format!(" {phrase} ")))
}

/// Python: bot/compose/remark.py:2118 `_planned_word(word)`
/// True when a word is one of PLANNED_VERBS or an ordinary inflection of one.
pub fn _planned_word(word: &str) -> bool {
    if PLANNED_VERBS.contains(word) {
        return true;
    }
    PLANNED_VERBS.iter().any(|verb| {
        let chop = &verb[..verb.len() - verb.chars().last().map_or(0, char::len_utf8)];
        let last = verb.chars().last().map(String::from).unwrap_or_default();
        [
            format!("{verb}ing"),
            format!("{chop}ing"),
            format!("{verb}s"),
            format!("{verb}d"),
            format!("{verb}{last}ing"),
            format!("{chop}ed"),
        ]
        .iter()
        .any(|form| form == word)
    })
}

/// Python: bot/compose/remark.py:2144 `_going_to_plan(clean)`
pub fn _going_to_plan(clean: &str) -> bool {
    for opener in GOING_TO.iter() {
        let Some(at) = clean.find(opener.as_str()) else { continue };
        let rest = text::split_ref(&clean[at + opener.len()..]);
        if rest.len() > 1 && ["work", "working", "works"].contains(&rest[0]) {
            // the work itself is the plan ("going to work on my cv"), the place is not
            return rest[1] == "on";
        }
        if rest.first().is_some_and(|w| _planned_word(w)) {
            return true;
        }
        // "thinking about going back to college" is a plan whose verb is one step further out
        if rest.len() > 1 && (rest[0] == "going" || rest[0] == "getting") && rest[1] == "back" {
            return true;
        }
        // "thinking of / about" takes any object, so the object must not be a person or the past.
        let stripped = text::strip(opener);
        if (stripped == "thinking of" || stripped == "thinking about") && !rest.iter().take(3).any(|w| THOUGHT_OBJECT.contains(*w)) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/remark.py:2166 `reads_as_plan(clean)`
/// True when the line says what the person means to do, over time.
pub fn reads_as_plan(clean: &str) -> bool {
    let words = text::split(clean);
    if any_in(&words, &ARRANGEMENT) {
        return _has_phrase(clean, &PLAN); // a visit is an arrangement, not a project
    }
    if _has_phrase(clean, &PLAN) || _going_to_plan(clean) {
        return true;
    }
    BOOKED.is_match(clean) && any_in(&words, &STEP_NOUNS) && any_in(&words, &BIGGER_PLAN)
}

/// Python: bot/compose/remark.py:2176 `_asked_last(memory)`
/// True when the bot's last reply ended on a question.
pub fn _asked_last(memory: &Memory) -> bool {
    memory.turns.last().is_some_and(|t| text::strip(&t.bot).ends_with('?'))
}

/// Python: bot/compose/remark.py:2183 `a_want_said_back(said_back, clean)`
/// True when the store's answer to this line only filed the want, so the reader can take it.
pub fn a_want_said_back(said_back: Option<&str>, clean: &str) -> bool {
    if A_LOSS_IN_THE_LINE.is_match(clean) || A_CONDOLENCE.is_match(said_back.unwrap_or("")) {
        return false;
    }
    crate::memory::repeat::only_said_it_back(clean, said_back)
}

/// Python: bot/compose/remark.py:2199 `dependant_want_reply_for(clean, words, memory)`
/// A reply to a want somebody the person looks after has, or None to leave it to the caller.
pub fn dependant_want_reply_for(clean: &str, words: &[String], memory: &Memory) -> Option<String> {
    if clean.is_empty() || !DEPENDANT_WANT.is_match(clean) {
        return None;
    }
    if any_in(words, &NOT_PLAIN) || _has_phrase(clean, &CRISIS) {
        return None;
    }
    if ASKED_OF_THE_PERSON.is_match(clean) || WANTS_TO_BE.is_match(clean) {
        return None;
    }
    if any_in(words, &A_WEAPON) {
        return None;
    }
    if any_in(words, &WEAPON_WORDS) && !_has_phrase(clean, &NOT_A_WEAPON) {
        return None;
    }
    if any_in(words, &ASKS) || _has_phrase(clean, &DOUBT) {
        return None;
    }
    if _has_phrase(clean, &COUNTERFACTUAL) {
        return None;
    }
    let mut options = DEPENDANT_WANT_REPLIES.clone();
    if _asked_last(memory) {
        // Two questions in a row is an interrogation: say something they can use instead.
        options = without_questions(&options);
    }
    Some(pick(&options, memory, clean))
}

/// Python: bot/compose/remark.py:2229 `reply_for(clean, words, memory, topics=())`
/// A short reply that fits the plan in a statement, or None to leave it to the caller.
pub fn reply_for(clean: &str, words: &[String], memory: &Memory, _topics: &[String]) -> Option<String> {
    if clean.is_empty() || any_in(words, &NOT_PLAIN) || _has_phrase(clean, &CRISIS) {
        return None;
    }
    if _has_phrase(clean, &COUNTERFACTUAL) || any_in(words, &MARKED_DATE) {
        return None;
    }
    if any_in(words, &ASKS) || _has_phrase(clean, &DOUBT) {
        return None;
    }
    if !reads_as_plan(clean) {
        return None;
    }
    let mut options = if _has_phrase(clean, &BARRIER) { PLAN_REPLIES_PLAIN.clone() } else { PLAN_REPLIES.clone() };
    if _asked_last(memory) {
        options = without_questions(&options);
    }
    Some(pick(&options, memory, clean))
}

/// Python: bot/compose/remark.py:2334 `reads_as_arranged(clean)`
/// True when the line says the person has booked or is booking something of their own.
pub fn reads_as_arranged(clean: &str) -> bool {
    if clean.is_empty() || clean.contains('?') {
        return false; // a question about booking is a question, not an arrangement
    }
    if DOUBLE_BOOKED.is_match(clean) {
        return false;
    }
    if WENT_WRONG.is_match(clean) {
        return false; // something about the booking has gone wrong
    }
    let booked = ARRANGED.search(clean);
    if let Some(b) = &booked {
        if TURNED.is_match(&clean[b.start()..]) {
            return false; // the line takes the news back after the booking
        }
    }
    if HAD_IT_BOOKED.is_match(clean) {
        return false; // the booking is behind them
    }
    if let Some(b) = &booked {
        if QUESTION_FIRST.match_(clean).is_some() && THE_PERSON_WITH_IT.match_(&clean[b.start()..]).is_none() {
            return false; // a question about a booking, typed without its question mark
        }
    }
    booked.is_some()
}

/// Python: bot/compose/remark.py:2352 `_says_when_it_is(clean)`
/// True when the line already says when the thing it names is.
pub fn _says_when_it_is(clean: &str) -> bool {
    if !clean.is_empty() && FUTURE.is_match(clean) {
        return true;
    }
    let low = text::lower(clean).replace(',', " ").replace('.', " ");
    text::split_ref(&low).iter().any(|w| DAYS_AND_MONTHS.contains(*w))
}

/// Python: bot/compose/remark.py:2364 `arranged_reply_for(clean, words, memory)`
/// A reply to a thing the person has arranged, or None to leave it to the caller.
pub fn arranged_reply_for(clean: &str, words: &[String], memory: &Memory) -> Option<String> {
    if any_in(words, &NOT_PLAIN) || _has_phrase(clean, &CRISIS) {
        return None;
    }
    if _has_phrase(clean, &COUNTERFACTUAL) || any_in(words, &MARKED_DATE) {
        return None;
    }
    if any_in(words, &ASKS) || _has_phrase(clean, &DOUBT) {
        return None;
    }
    if !reads_as_arranged(clean) {
        return None;
    }
    let mut options = BOOKED_REPLIES.clone();
    if _says_when_it_is(clean) {
        // The booking carries its own date, so the one reply that asks for one is swapped.
        options = options.iter().map(|o| ALREADY_DATED.get(o).cloned().unwrap_or_else(|| o.clone())).collect();
    }
    if _asked_last(memory) {
        options = without_questions(&options);
    }
    Some(pick(&options, memory, clean))
}

/// Python: bot/compose/remark.py:2448 `_named(template, thing)`
/// The template with the activity in it, capitalised when it opens the sentence.
pub fn _named(template: &str, thing: &str) -> String {
    template.replace("{thing}", thing).replace("{Thing}", &first_upper(thing))
}

/// Python: bot/compose/remark.py:2454 `interest_reply_for(interest, memory)`
/// A reply to an activity the person is into, or None to leave it to the caller. `interest` is
/// `understood["taking_up"]` as a value.
pub fn interest_reply_for(interest: &Value, memory: &Memory) -> Option<String> {
    if !interest.is_dict() {
        return None;
    }
    let thing = match interest.get("text") {
        Some(Value::Str(s)) => text::strip(s).to_string(),
        _ => String::new(),
    };
    if thing.is_empty() || text::split_ref(&thing).iter().any(|w| NOT_AN_INTEREST.contains(*w)) {
        return None;
    }
    let started = interest.get("shape").and_then(Value::as_str) == Some("started");
    let back = interest.get("back").is_some_and(Value::truthy);
    let asked_last = _asked_last(memory);
    let options: &Vec<String> = if back {
        if asked_last { &INTEREST_BACK_PLAIN } else { &INTEREST_BACK }
    } else if asked_last {
        if started { &INTEREST_STARTED_PLAIN } else { &INTEREST_DOING_PLAIN }
    } else if started {
        &INTEREST_STARTED
    } else {
        &INTEREST_DOING
    };
    let lines: Vec<String> = options.iter().map(|t| _named(t, &thing)).collect();
    Some(pick(&lines, memory, &thing))
}
