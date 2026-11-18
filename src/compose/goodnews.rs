//! Port of `bot/compose/goodnews.py`: good news said as an event, not as a feeling - a pass, a
//! win, a baby, a thing that has arrived, an occasion coming up, something that got better.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use chrono::Datelike;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.goodnews.", stringify!($name))));)*
    };
}
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.compose.goodnews.", stringify!($name))));)*
    };
}
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.goodnews.", stringify!($name))));)*
    };
}
/// Tuples of compiled patterns, in order.
macro_rules! pattern_lists {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<&'static Re>> = LazyLock::new(|| {
            data::get(concat!("bot.compose.goodnews.", stringify!($name)))
                .items().iter().map(|v| v.as_re().expect("a pattern")).collect()
        });)*
    };
}

regexes!(
    _I_PRAISED, HAVE_GOT, NEWS_OF_IT, MOVED_IN, FAULT, A_POINTER_IN_FRONT, ORDINAL_DAY, NOT_AN_OCCASION, SOMEONE_LEFT,
    LET_DOWN, RELATION_BEFORE_DAY, NOT_NEWS, NOT_QUITE, A_TURN_IN_THE_TALE, A_SETBACK_AFTER_THE_WIN, A_GOOD_ENDING, BAD,
    ABOUT_THEM, CONDITIONAL, NEWS_ASK, NEWS_WORD, WORLD_NEWS, _STANDIN, HANDED_OVER, NEWS_IDIOM, WHAT_HAPPENED_ASK,
    _AHEAD, _COUNTED_UP, _A_PERSON, _AN_AGE_SAID, _SOON_ITSELF, _ALREADY, _DREAD, _NOT_HAPPENING, _BARELY_ASKED, _MONEY,
    _CLAUSE_BREAK, _AHEAD_SPAN, _DONE_ALREADY, _ENTERED_IT, _A_MESSAGE_THAT_CLAIMS_A_WIN, _A_WIN_THEN_A_MESSAGE,
    _ASKS_FOR_SOMETHING_BACK, _NEVER_ENTERED, _ABOUT_THE_WIN, _A_RELATIVE_TOLD_ME, A_YEAR_AFTER, JUST_HAPPENED,
    SAYS_IT_IN_THE_PRESENT, _A_CHEER, _AN_ATTEMPT, _A_SCORE, _OR_A_SCORE, _A_CLOCK, _A_WEIGHT, _A_FIGURE_IN_THE_LINE,
    _ASKS_OR_THANKS, NOBODY_SAID_ANYTHING, CLAUSE, NEGATED, DENIED, ASK_OPEN, BARE_GONE, BODY_GAVE_WAY, SETTLED_TAIL,
    CLEAR_TAIL, NOT_CLEAR,
);
word_sets!(
    ANIMAL_THING, HOME_WORK, HOME_THING, OBJECT_THING, NOT_AN_ARRIVAL, THING_STOPS, FAMILY, PASS_PARTICLE, PASS_TAKEN,
    NEWS_FRAME, NEWS_IDIOM_TAIL, _AROUND_THE_FIGURES, SPEAKER, TAKES_A_CLAUSE, SILENCE_WORDS, CLAUSE_STOPS, WORRY_WORDS,
);
pools!(NEW_ANIMAL, NEW_HOME, NEW_PLACE, NEW_THING, YEARS_BACK_LINES, _MORE_OF_THE_NEWS_MARKS);
pattern_lists!(BABY, PASSED, WIN, GOOD, ARRIVED, RECOVERY, _COMING_UP, SETTLED, CLEAR_RESULT);

/// `OCCASION`: (name, pattern) in order.
static OCCASION: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    data::get("bot.compose.goodnews.OCCASION")
        .items()
        .iter()
        .map(|row| (row.items()[0].str().to_string(), row.items()[1].as_re().expect("a pattern")))
        .collect()
});

static A_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d", 0));
static AN_OWN_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:i|we|my|our|im|i'm|we're|were)\b|\bthe\s+baby\b", 0));
static GONE_AGAIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bgone\s+again\b", crate::py::re::I));
static A_PUNCTUATION_MARK: LazyLock<Re> = LazyLock::new(|| Re::new(r"[,;:!?]", 0));
static A_CLAUSE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"[,.;!?]", 0));
static A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// `compose.doubted(clean)` with its default window.
fn doubted(clean: &str) -> bool {
    crate::compose::doubted(clean, 4)
}

/// `" ".join(str(x).lower().split())`.
fn flat(s: &str) -> String {
    text::squash(&text::lower(s))
}

/// `w.strip(".,!?;:\"'")` for each word.
fn bare_words(s: &str) -> Vec<String> {
    text::split_ref(s).iter().map(|w| text::strip_chars(w, ".,!?;:\"'").to_string()).collect()
}

fn any_matches(patterns: &[&Re], text_: &str) -> bool {
    patterns.iter().any(|p| p.is_match(text_))
}

/// Python: bot/compose/goodnews.py:395 `_bare(word)`
/// The word without a plural ending, so "two puppies" names a puppy.
pub fn _bare(word: &str) -> String {
    let n = text::len(word);
    if n > 3 && word.ends_with("ies") {
        return format!("{}y", &word[..word.len() - 3]);
    }
    if n > 3 && word.ends_with("es") && word[..word.len() - 2].ends_with(['s', 'x', 'z']) {
        return word[..word.len() - 2].to_string();
    }
    if n > 3 && word.ends_with('s') && !word.ends_with("ss") {
        return word[..word.len() - 1].to_string();
    }
    word.to_string()
}

/// Python: bot/compose/goodnews.py:411 `_thing_words(match)`
/// The thing that arrived, cut at the first word that is not part of its name (`thing` is the
/// match's "thing" group).
pub fn _thing_words(thing: &str) -> String {
    let said = A_POINTER_IN_FRONT.sub("", thing);
    let mut kept: Vec<String> = Vec::new();
    for word in text::split_ref(&said) {
        let bare = text::strip_chars(word, ".,!?;:\"'");
        if bare.is_empty() || THING_STOPS.contains(bare) || NOT_AN_ARRIVAL.contains(bare) || kept.len() == 3 {
            break;
        }
        kept.push(bare.to_string());
    }
    kept.join(" ")
}

/// `{"kind": kind, "thing": thing}`.
fn arrival(kind: &str, thing: &str) -> Value {
    let mut d = Value::dict();
    d.set_key("kind", kind);
    d.set_key("thing", thing);
    d
}

/// Python: bot/compose/goodnews.py:423 `new_thing(clean)`
/// The kind and name of a thing that has just arrived, or None: `{"kind": "animal"|"home"|
/// "place"|"thing", "thing": "rescue greyhound"}`.
pub fn new_thing(clean: &str) -> Option<Value> {
    if clean.is_empty() {
        return None;
    }
    if NOT_NEWS.is_match(clean) || BAD.is_match(clean) || FAULT.is_match(clean) || doubted(clean) {
        return None;
    }
    let mut said = clean.to_string();
    if HAVE_GOT.is_match(&said) && NEWS_OF_IT.is_match(&said) {
        // "i have got a new car" is "i got a new car" said the British way.
        said = HAVE_GOT.sub(r"\g<who> got", &said);
    }
    if MOVED_IN.is_match(&said) {
        return Some(arrival("place", ""));
    }
    for pattern in ARRIVED.iter() {
        let Some(found) = pattern.search(&said) else { continue };
        let thing = _thing_words(found.name_str("thing"));
        if thing.is_empty() || ["new", "another", "first", "second"].contains(&thing.as_str()) {
            continue;
        }
        // Both the word as said and its bare form.
        let mut words: HashSet<String> = text::split(&thing).into_iter().collect();
        let bares: Vec<String> = words.iter().map(|w| _bare(w)).collect();
        words.extend(bares);
        if words.iter().any(|w| NOT_AN_ARRIVAL.contains(w)) {
            continue;
        }
        // Only an animal, a job on the house, or an object the person now has can be asked
        // "what's it like".
        let has = |set: &HashSet<String>| words.iter().any(|w| set.contains(w));
        if !(has(&ANIMAL_THING) || has(&HOME_THING) || has(&OBJECT_THING)) {
            continue;
        }
        if has(&ANIMAL_THING) {
            return Some(arrival("animal", &thing));
        }
        if has(&HOME_WORK) {
            return Some(arrival("home", &thing));
        }
        return Some(arrival("thing", &thing));
    }
    None
}

/// Python: bot/compose/goodnews.py:470 `new_thing_replies(arrived)`
/// The lines an arrival is answered with, the thing's name said back once.
pub fn new_thing_replies(arrived: &Value) -> Vec<String> {
    let kind = arrived.get("kind").and_then(Value::as_str).unwrap_or("");
    let thing = arrived.get("thing").and_then(Value::as_str).unwrap_or("");
    if kind == "place" {
        return NEW_PLACE.clone();
    }
    let pool: &Vec<String> = match kind {
        "animal" => &NEW_ANIMAL,
        "home" => &NEW_HOME,
        _ => &NEW_THING,
    };
    pool.iter().map(|line| crate::compose::fallback_chain::py_format(line, &[("thing", thing)])).collect()
}

/// Python: bot/compose/goodnews.py:572 `_the_person_word(word)`
/// The relation word inside a possessive - "mums", "wifes", "son's" - or "".
pub fn _the_person_word(word: &str) -> String {
    let mut word = text::lower(text::strip_chars(text::strip(word), "'"));
    if word.ends_with('s') {
        word.pop();
    }
    word
}

/// Python: bot/compose/goodnews.py:580 `someone_elses_occasion(clean)`
/// True when the day named belongs to somebody other than the person writing.
pub fn someone_elses_occasion(clean: &str) -> bool {
    let text_ = flat(clean);
    for found in RELATION_BEFORE_DAY.finditer(&text_) {
        if text::split_ref(found.name_str("whose")).iter().any(|w| FAMILY.contains(&_the_person_word(w))) {
            return true;
        }
    }
    // "my mums 70th is in march" names the day with the number.
    ORDINAL_DAY.finditer(&text_).iter().any(|found| FAMILY.contains(&_the_person_word(found.name_str("whose"))))
}

/// Python: bot/compose/goodnews.py:599 `reads_as_an_occasion(clean)`
/// The occasion a line says is coming up - "birthday", "anniversary", "wedding" - or None.
pub fn reads_as_an_occasion(clean: &str) -> Option<String> {
    let text_ = flat(clean);
    if text_.is_empty() || text_.contains('?') {
        return None;
    }
    if NOT_AN_OCCASION.is_match(&text_) || LET_DOWN.is_match(&text_) || SOMEONE_LEFT.is_match(&text_) || doubted(&text_) || BAD.is_match(&text_) {
        return None;
    }
    for (name, pattern) in OCCASION.iter() {
        if pattern.is_match(&text_) {
            return Some(name.clone());
        }
    }
    if ORDINAL_DAY.is_match(&text_) {
        return Some("birthday".to_string());
    }
    None
}

/// Python: bot/compose/goodnews.py:740 `_a_hand_over(words)`
/// True when a "passed" in these words hands something over rather than reports a result.
pub fn _a_hand_over(words: &[String]) -> bool {
    for (i, word) in words.iter().enumerate() {
        if word != "passed" && word != "pass" {
            continue;
        }
        let after = &words[i + 1..(i + 4).min(words.len())];
        if after.is_empty() {
            continue;
        }
        if PASS_PARTICLE.contains(&after[0]) {
            return true;
        }
        let thing = if ["the", "a", "an"].contains(&after[0].as_str()) && after.len() > 1 { &after[1] } else { &after[0] };
        if PASS_TAKEN.contains(thing) {
            return true;
        }
        if after[0] == "it" && after.len() > 1 && PASS_PARTICLE.contains(&after[1]) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/goodnews.py:758 `reads_as_a_pass(clean)`
/// Whether the line reports a pass: "i passed my driving test", "i passed".
pub fn reads_as_a_pass(clean: &str) -> bool {
    if clean.is_empty() {
        return false;
    }
    if NOT_NEWS.is_match(clean) || NOT_QUITE.is_match(clean) || doubted(clean) {
        return false;
    }
    if reads_as_a_win_nobody_noticed(clean) {
        return false; // a pass with a silence round it is answered as both, not as a pass
    }
    if _a_hand_over(&bare_words(clean)) {
        return false;
    }
    any_matches(&PASSED, clean)
}

/// Python: bot/compose/goodnews.py:849 `_an_idiom_standing_alone(low, end)`
/// True when nothing after the idiom names a thing of its own (`end` a byte offset).
pub fn _an_idiom_standing_alone(low: &str, end: usize) -> bool {
    bare_words(&low[end..]).iter().all(|w| w.is_empty() || NEWS_IDIOM_TAIL.contains(w))
}

/// Python: bot/compose/goodnews.py:866 `asks_for_news(clean)`
/// Whether the line asks for the news rather than telling it.
pub fn asks_for_news(clean: &str) -> bool {
    if clean.is_empty() {
        return false;
    }
    let low = text::lower(text::rstrip_chars(text::strip(clean), "?!."));
    if let Some(idiom) = NEWS_IDIOM.match_(&low) {
        if _an_idiom_standing_alone(&low, idiom.end()) {
            // The ask in the words people use for it, with nothing named after it.
            return !HANDED_OVER.is_match(&low);
        }
    }
    if WHAT_HAPPENED_ASK.match_(&low).is_some() {
        return true;
    }
    if !(NEWS_ASK.match_(&low).is_some() && NEWS_WORD.is_match(&low)) {
        return false;
    }
    if low.contains("news") {
        if WORLD_NEWS.is_match(&low) {
            return false;
        }
    } else if !_STANDIN.is_match(&low) {
        return false;
    }
    // "got some good news for you" is a person handing the news over, not asking for any.
    if HANDED_OVER.is_match(&low) {
        return false;
    }
    let words: Vec<String> = bare_words(&low).into_iter().filter(|w| !w.is_empty()).collect();
    if words.iter().all(|w| NEWS_FRAME.contains(w)) {
        return true;
    }
    // A topic after the ask is still an ask for the news about it; "tell me something good about
    // london" is a question about London.
    !["tell", "give", "share", "spill"].contains(&words[0].as_str())
}

/// Python: bot/compose/goodnews.py:1008 `_a_money_clause(clean)`
/// True when a clause of the line counts something in money rather than in years.
pub fn _a_money_clause(clean: &str) -> bool {
    _CLAUSE_BREAK.split(clean).iter().any(|clause| _MONEY.is_match(clause) && A_DIGIT.is_match(clause))
}

/// Python: bot/compose/goodnews.py:1039 `ahead_of_now(text)`
/// Whether the line puts its thing ahead of now, by a date or by a span of time.
pub fn ahead_of_now(text_: &str) -> bool {
    let clean = flat(text_);
    if clean.is_empty() || clean.contains('?') {
        return false;
    }
    // A move already made is behind the person: the tense decides it, not the day.
    if _DONE_ALREADY.is_match(&clean) {
        return false;
    }
    _AHEAD.is_match(&clean) || _AHEAD_SPAN.is_match(&clean)
}

/// `pattern is _COUNTED_UP` for a pattern of `_COMING_UP`.
fn is_counted_up(pattern: &Re) -> bool {
    pattern.pattern() == _COUNTED_UP.pattern() && pattern.flags() == _COUNTED_UP.flags()
}

/// Python: bot/compose/goodnews.py:1055 `reads_as_news_to_come(clean)`
/// Whether the line puts a happy event ahead of now: a date and a thing still to happen.
pub fn reads_as_news_to_come(clean: &str) -> bool {
    if clean.is_empty() || clean.contains('?') {
        return false;
    }
    if BAD.is_match(clean)
        || NOT_NEWS.is_match(clean)
        || NOT_QUITE.is_match(clean)
        || doubted(clean)
        || crate::compose::badnews::reads_as_someones_death(clean)
    {
        return false;
    }
    if _ALREADY.is_match(clean) || _DREAD.is_match(clean) {
        return false;
    }
    if _NOT_HAPPENING.is_match(clean) || _BARELY_ASKED.is_match(clean) {
        return false;
    }
    if !_AHEAD.is_match(clean) && !_SOON_ITSELF.is_match(clean) {
        return false;
    }
    if !AN_OWN_WORD.is_match(clean) {
        return false;
    }
    let hits: Vec<&Re> = _COMING_UP.iter().copied().filter(|p| p.is_match(clean)).collect();
    if hits.is_empty() {
        return false;
    }
    // A sum is not an age, and a rota or a timetable is not an age either.
    if hits.iter().all(|p| is_counted_up(p)) {
        if _a_money_clause(clean) || !_the_number_counts_a_life(clean, &hits) {
            return false;
        }
    }
    true
}

/// Python: bot/compose/goodnews.py:1083 `_the_number_counts_a_life(clean, hits)`
/// True when the counted number is somebody's age, not a rota or a timetable.
pub fn _the_number_counts_a_life(clean: &str, _hits: &[&Re]) -> bool {
    if _AN_AGE_SAID.is_match(clean) {
        return true;
    }
    _CLAUSE_BREAK.split(clean).iter().any(|clause| _COUNTED_UP.is_match(clause) && _A_PERSON.is_match(clause))
}

/// Python: bot/compose/goodnews.py:1149 `reads_as_a_prize_in_a_message(clean)`
/// True when the win in the line came to the person in a message they did not ask for.
pub fn reads_as_a_prize_in_a_message(clean: &str) -> bool {
    if clean.is_empty() || _ENTERED_IT.is_match(clean) {
        return false;
    }
    if _ABOUT_THE_WIN.is_match(clean) || _A_RELATIVE_TOLD_ME.is_match(clean) {
        return false;
    }
    if _NEVER_ENTERED.is_match(clean) {
        return true; // "i won a competition i never entered" is the scam whatever the order
    }
    if _A_MESSAGE_THAT_CLAIMS_A_WIN.is_match(clean) {
        return true;
    }
    _A_WIN_THEN_A_MESSAGE.is_match(clean) && _ASKS_FOR_SOMETHING_BACK.is_match(clean)
}

/// Python: bot/compose/goodnews.py:1177 `a_year_behind(text, today=None)`
/// Whether the line puts the thing in a year that has gone: "answered in 1998".
pub fn a_year_behind(text_: &str, today: Option<chrono::NaiveDate>) -> bool {
    let year = today.unwrap_or_else(crate::py::clock::today).year() as i64;
    A_YEAR_AFTER.finditer(text_).iter().any(|found| found.str(1).parse::<i64>().is_ok_and(|y| y < year))
}

/// Python: bot/compose/goodnews.py:1209 `reply_for_a_year_ago(text, reply, memory=None)`
/// The line for a win the person puts in a year that has gone, or None for another turn.
pub fn reply_for_a_year_ago(text_: &str, reply: &str, memory: &Memory) -> Option<String> {
    if !(reads_as_a_congratulation(reply) || the_cheer_was_this_pool(memory)) {
        return None;
    }
    reply_for_a_win_from_a_past_year(text_).as_str().map(str::to_string)
}

/// Python: bot/compose/goodnews.py:1216 `reply_for_a_win_from_a_past_year(text)`
/// The line for a win the person puts in a year that has gone, or None (as a value).
pub fn reply_for_a_win_from_a_past_year(text_: &str) -> Value {
    if !a_year_behind(text_, None) {
        return Value::None;
    }
    if JUST_HAPPENED.is_match(&text::lower(text_)) || SAYS_IT_IN_THE_PRESENT.is_match(text_) {
        return Value::None;
    }
    let seed = crate::py::random::ord_sum(&text::lower(text::strip(text_)));
    let at = seed.rem_euclid(YEARS_BACK_LINES.len() as i128) as usize;
    Value::from(YEARS_BACK_LINES[at].as_str())
}

/// The good-news note `chat._say` leaves: (kind is "good_news", the turn it was made on).
fn good_news_note(memory: &Memory) -> Option<Option<&Value>> {
    let note = memory.get("said_kind").filter(|n| n.is_dict())?;
    if note.get("kind").and_then(Value::as_str) != Some("good_news") {
        return None;
    }
    Some(note.get("turn"))
}

/// `note.get("turn") == n`, Python's equality (an int or a float equal to it, never a bool's 1).
fn turn_is(turn: Option<&Value>, n: i64) -> bool {
    match turn {
        Some(Value::Int(t)) => *t == n,
        Some(Value::Float(t)) => *t == n as f64,
        Some(Value::Bool(b)) => (*b as i64) == n,
        _ => false,
    }
}

/// Python: bot/compose/goodnews.py:1231 `the_cheer_was_this_pool(memory)`
/// Whether the reply the person just got came from the good-news pool, whatever it said.
pub fn the_cheer_was_this_pool(memory: &Memory) -> bool {
    match good_news_note(memory) {
        Some(turn) => turn_is(turn, memory.turns.len() as i64),
        None => false,
    }
}

/// Python: bot/compose/goodnews.py:1256 `reads_as_a_congratulation(reply)`
/// Whether the composer's line is a cheer for something that has just happened.
pub fn reads_as_a_congratulation(reply: &str) -> bool {
    _A_CHEER.match_(&text::squash(reply)).is_some()
}

/// Python: bot/compose/goodnews.py:1309 `_only_the_arrivals_figures(said)`
/// Whether every word of the line is the sex, a figure, or the words that join them.
pub fn _only_the_arrivals_figures(said: &str) -> bool {
    let low = text::lower(said).replace('\u{2019}', "'");
    for word in text::split_ref(&low) {
        let bare = text::strip_chars(word, ".,!;:\"'");
        if bare.is_empty() || _AROUND_THE_FIGURES.contains(bare) {
            continue;
        }
        if _A_FIGURE_IN_THE_LINE.fullmatch(bare).is_some() {
            continue;
        }
        return false;
    }
    true
}

/// Python: bot/compose/goodnews.py:1358 `reads_as_the_detail_line(reply)`
/// Whether a reply is this module's own wording for a detail added to the news.
pub fn reads_as_the_detail_line(reply: &str) -> bool {
    let said = text::squash(&text::lower(reply).replace('\u{2019}', "'"));
    _MORE_OF_THE_NEWS_MARKS.iter().any(|mark| said.contains(mark.as_str()))
}

/// Python: bot/compose/goodnews.py:1364 `the_reply_just_cheered(memory)`
/// Whether the reply the person just got came from the good-news pool, read the way the next
/// turn finds the note: one back from the turn just finished, and from the chat module.
pub fn the_reply_just_cheered(memory: &Memory) -> bool {
    let Some(turn) = good_news_note(memory) else { return false };
    let Some(last) = memory.turns.last() else { return false };
    if last.module.as_deref().unwrap_or("") != "chat" {
        return false;
    }
    turn_is(turn, memory.turns.len() as i64 - 1)
}

/// Python: bot/compose/goodnews.py:1387 `a_detail_of_the_news(clean, memory=None)`
/// The shape of a detail added to good news - "attempt", "score", "arrival" - or "".
pub fn a_detail_of_the_news(clean: &str, memory: &Memory) -> String {
    a_detail_of_the_news_in(clean, Some(memory))
}

/// `a_detail_of_the_news` with `memory` None or given.
pub fn a_detail_of_the_news_in(clean: &str, memory: Option<&Memory>) -> String {
    let said = text::squash(clean);
    if said.is_empty() || text::split_ref(&said).len() > 6 {
        return String::new();
    }
    // The arrival's figures are the one shape read without the cheer.
    let empty = Memory::new();
    if _A_WEIGHT.is_match(&said) && !said.contains('?') && _only_the_arrivals_figures(&said) && a_birth_just_announced(memory.unwrap_or(&empty)) {
        return "arrival".to_string();
    }
    if let Some(memory) = memory {
        if !(the_cheer_was_this_pool(memory) || the_reply_just_cheered(memory)) {
            return String::new();
        }
    }
    if _AN_ATTEMPT.is_match(&said) {
        return "attempt".to_string();
    }
    if _OR_A_SCORE.match_(&said).is_some() && _A_SCORE.is_match(&said) && _A_CLOCK.match_(&said).is_none() {
        return "score".to_string();
    }
    String::new()
}

/// Python: bot/compose/goodnews.py:1413 `a_birth_just_announced(memory)`
/// Whether the person announced a birth in one of the two lines before this one.
pub fn a_birth_just_announced(memory: &Memory) -> bool {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(2)..].iter().any(|t| !t.user.is_empty() && any_matches(&BABY, &t.user))
}

/// Python: bot/compose/goodnews.py:1427 `reads_as_a_detail_of_the_news(clean, memory=None)`
/// Whether the line adds a detail to news the bot has just cheered, with nothing asked in it.
pub fn reads_as_a_detail_of_the_news(clean: &str, memory: &Memory) -> bool {
    if _ASKS_OR_THANKS.is_match(&text::squash(clean)) {
        return false;
    }
    !a_detail_of_the_news(clean, memory).is_empty()
}

/// Python: bot/compose/goodnews.py:1434 `reads_as_good_news(clean)`
/// Whether a cleaned, lowercased line reports a happy event as a plain phrase.
pub fn reads_as_good_news(clean: &str) -> bool {
    if reads_as_news_to_come(clean) {
        return false;
    }
    if clean.is_empty() {
        return false;
    }
    if BAD.is_match(clean)
        || NOT_NEWS.is_match(clean)
        || NOT_QUITE.is_match(clean)
        || doubted(clean)
        || crate::compose::badnews::reads_as_someones_death(clean)
    {
        return false;
    }
    if asks_for_news(clean) {
        return false;
    }
    if reads_as_a_win_nobody_noticed(clean) {
        return false; // news with nobody's word on it is answered as both halves
    }
    if reads_as_a_prize_in_a_message(clean) {
        return false; // a prize that arrived in a text is a scam, not a celebration
    }
    if A_TURN_IN_THE_TALE.is_match(clean) || A_SETBACK_AFTER_THE_WIN.is_match(clean) {
        return false; // the clause after the "and" takes the win back
    }
    if _a_hand_over(&bare_words(clean)) {
        return false;
    }
    if _I_PRAISED.is_match(clean) && !(any_matches(&BABY, clean) || any_matches(&PASSED, clean) || any_matches(&WIN, clean)) {
        // "i said well done to my son": the praise is the person's own, going out.
        return false;
    }
    any_matches(&GOOD, clean)
}

/// Python: bot/compose/goodnews.py:1500 `_the_speaker_is_the_quiet_one(text, match)`
/// Whether the words in front of the match name the person talking as the one who said nothing
/// (`start` is the match's start, a byte offset).
pub fn _the_speaker_is_the_quiet_one(text_: &str, start: usize) -> bool {
    let clauses = CLAUSE.split(&text_[..start]);
    let last = clauses.last().map(String::as_str).unwrap_or("");
    let words = text::split_ref(last);
    words[words.len().saturating_sub(4)..].iter().any(|w| SPEAKER.contains(text::strip_chars(w, ".,!?;:\"'")))
}

/// Python: bot/compose/goodnews.py:1523 `_the_silence_is_the_whole_of_it(text, match)`
/// Whether nothing but the silence follows the verb in a silence match.
pub fn _the_silence_is_the_whole_of_it(text_: &str, found: &crate::py::re::Match) -> bool {
    if found.name("object").is_some() {
        return true; // "my mum said nothing about it"
    }
    let verb = text::split_ref(found.name_str("verb")).last().copied().unwrap_or("").to_string();
    if TAKES_A_CLAUSE.contains(&verb) {
        return true;
    }
    let after = &text_[found.end()..];
    let rest = match A_CLAUSE_END.search(after) {
        Some(m) => &after[..m.start()],
        None => after,
    };
    let mut kept: Vec<&str> = Vec::new();
    for word in text::split_ref(rest) {
        let bare = text::strip_chars(word, ".,!?;:\"'");
        if CLAUSE_STOPS.contains(bare) {
            break;
        }
        kept.push(bare);
    }
    kept.iter().all(|b| SILENCE_WORDS.contains(*b))
}

/// Python: bot/compose/goodnews.py:1545 `reads_as_a_win_a_cost_came_with(clean)`
/// Whether a line reports a win and the cost of it in the same breath.
pub fn reads_as_a_win_a_cost_came_with(clean: &str) -> bool {
    let text_ = flat(clean);
    if text_.is_empty() || text_.contains('?') {
        return false;
    }
    if NOT_NEWS.is_match(&text_) || NOT_QUITE.is_match(&text_) || doubted(&text_) {
        return false;
    }
    if !any_matches(&GOOD, &text_) {
        return false;
    }
    if !(A_TURN_IN_THE_TALE.is_match(&text_) || A_SETBACK_AFTER_THE_WIN.is_match(&text_)) {
        return false;
    }
    // "we came into some money and now we can fix the roof": the other half of the win.
    !A_GOOD_ENDING.is_match(&text_)
}

/// Python: bot/compose/goodnews.py:1567 `reads_as_a_win_nobody_noticed(clean)`
/// Whether a line reports a win and a silence around it, the silence somebody else's.
pub fn reads_as_a_win_nobody_noticed(clean: &str) -> bool {
    let text_ = flat(clean);
    if text_.is_empty() || text_.contains('?') {
        return false;
    }
    if NOT_NEWS.is_match(&text_) || NOT_QUITE.is_match(&text_) || doubted(&text_) {
        return false;
    }
    if !any_matches(&GOOD, &text_) {
        return false;
    }
    for found in NOBODY_SAID_ANYTHING.finditer(&text_) {
        if _the_speaker_is_the_quiet_one(&text_, found.start()) {
            continue;
        }
        if _the_silence_is_the_whole_of_it(&text_, &found) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/goodnews.py:1605 `_denied(text, start, end=None)`
/// Whether a negator sits in the phrase or just before it (offsets are bytes; the 26 is
/// characters, as in Python).
pub fn _denied(text_: &str, start: usize, end: Option<usize>) -> bool {
    let from_char = text::char_index(text_, start).saturating_sub(26);
    let from = text::byte_at(text_, from_char);
    let to = end.unwrap_or(start).max(from);
    DENIED.is_match(&text_[from..to])
}

/// Python: bot/compose/goodnews.py:1613 `reads_as_recovery(clean)`
/// Whether a cleaned, lowercased line reports that something has got better.
pub fn reads_as_recovery(clean: &str) -> bool {
    let text_ = flat(clean);
    if text_.is_empty() || text_.contains('?') {
        return false;
    }
    if !ABOUT_THEM.is_match(&text_) {
        return false;
    }
    if BAD.is_match(&text_) || NOT_NEWS.is_match(&text_) {
        return false;
    }
    if CONDITIONAL.is_match(&text_) || NEGATED.is_match(&text_) {
        return false;
    }
    any_matches(&RECOVERY, &text_)
}

/// Python: bot/compose/goodnews.py:1696 `worry_on_the_table(memory, look_back=4)`
/// True when the last few things the person said named a body, an illness or a worry.
pub fn worry_on_the_table(memory: &Memory, look_back: &Value) -> bool {
    let look_back = look_back.as_int().unwrap_or(4);
    let mut seen = 0;
    for turn in memory.turns.iter().rev() {
        let said = flat(&turn.user);
        if said.is_empty() {
            continue;
        }
        if A_WORD.findall_str(&said).iter().any(|w| WORRY_WORDS.contains(w)) {
            return true;
        }
        seen += 1;
        if seen >= look_back {
            break;
        }
    }
    false
}

/// Python: bot/compose/goodnews.py:1730 `the_tail_adds_nothing(text, end)`
/// Whether what follows a settled phrase is only the words that add nothing (`end` in bytes).
pub fn the_tail_adds_nothing(text_: &str, end: usize) -> bool {
    let tail = A_PUNCTUATION_MARK.sub(" ", &text_[end..]);
    let tail = text::strip(&tail);
    tail.is_empty() || SETTLED_TAIL.match_(&format!(" {tail}")).is_some()
}

/// Python: bot/compose/goodnews.py:1736 `_a_bare_gone_is_not_the_news(text, match)`
/// Whether a match that turns on "gone"/"went" is still the news, once the line is read.
pub fn _a_bare_gone_is_not_the_news(text_: &str, found: &crate::py::re::Match) -> bool {
    if !BARE_GONE.is_match(found.as_str()) {
        return true;
    }
    if GONE_AGAIN.is_match(text_) {
        return false;
    }
    if BODY_GAVE_WAY.is_match(text_) {
        return false;
    }
    // "its gone" is the news only when nothing follows it.
    the_tail_adds_nothing(text_, found.end())
}

/// Python: bot/compose/goodnews.py:1766 `the_news_is_the_whole_of_it(clean)`
/// Whether the recovery or settled phrase in the line closes the clause.
pub fn the_news_is_the_whole_of_it(clean: &str) -> bool {
    let text_ = flat(clean);
    let mut end = 0;
    for pattern in RECOVERY.iter().chain(SETTLED.iter()) {
        for found in pattern.finditer(&text_) {
            end = end.max(found.end());
        }
    }
    end > 0 && SETTLED_TAIL.match_(&text_[end..]).is_some()
}

/// Python: bot/compose/goodnews.py:1837 `reads_as_a_clear_result(clean)`
/// Whether a cleaned, lowercased line says a test or a treatment came back well, about anyone.
pub fn reads_as_a_clear_result(clean: &str) -> bool {
    let text_ = flat(clean);
    if text_.is_empty() || text_.contains('?') {
        return false;
    }
    if ASK_OPEN.is_match(&text_) || NOT_CLEAR.is_match(&text_) || NEGATED.is_match(&text_) {
        return false;
    }
    if CONDITIONAL.is_match(&text_) || NOT_NEWS.is_match(&text_) || NOT_QUITE.is_match(&text_) {
        return false;
    }
    CLEAR_RESULT.iter().any(|pattern| pattern.finditer(&text_).iter().any(|found| CLEAR_TAIL.match_(&text_[found.end()..]).is_some()))
}

/// Python: bot/compose/goodnews.py:1858 `reads_as_settled(clean)`
/// Whether a line says the thing they were worried about has passed, about anyone.
pub fn reads_as_settled(clean: &str) -> bool {
    let text_ = flat(clean);
    if text_.is_empty() || text_.contains('?') {
        return false;
    }
    if ASK_OPEN.is_match(&text_) || CONDITIONAL.is_match(&text_) {
        return false;
    }
    SETTLED.iter().any(|pattern| {
        pattern
            .search(&text_)
            .is_some_and(|found| !_denied(&text_, found.start(), Some(found.end())) && _a_bare_gone_is_not_the_news(&text_, &found))
    })
}
