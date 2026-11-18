//! Port of `bot/memory/facts.py`: facts about the user, pulled out of what they say.
//!
//! `extract(text)` returns only the facts a line clearly states, e.g.
//! {"name": "Jess", "job": "nurse", "city": "Leeds", "allergies": ["nuts"]}. Callers merge the
//! result into memory["user"]. Nothing here replies; it is data extraction only.
//!
//! Module data (word lists, regexes, tables) comes from the exported bundle by Python name.
//! Positions inside this file are byte offsets into the text (Python's are code points); where
//! Python does arithmetic on a position in characters, the port counts characters too.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::py::re::{Match, Re};
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// A set (or a dict's keys) from the bundle, under `bot.memory.facts.<NAME>` unless the name
/// has a dot in it.
fn bundle_set(name: &str) -> HashSet<String> {
    let full = if name.contains('.') { name.to_string() } else { format!("bot.memory.facts.{name}") };
    let v = data::get(&full);
    match v {
        Value::Dict(d) => d.iter().filter_map(|(k, _)| k.as_str().map(str::to_string)).collect(),
        _ => v.str_set(),
    }
}

fn bundle_strs(name: &str) -> Vec<String> {
    let full = if name.contains('.') { name.to_string() } else { format!("bot.memory.facts.{name}") };
    data::strs(&full)
}

fn bundle_map(name: &str) -> HashMap<String, String> {
    let full = if name.contains('.') { name.to_string() } else { format!("bot.memory.facts.{name}") };
    data::str_map(&full)
}

/// `bot.memory.facts.<NAME>` sets, as `LazyLock<HashSet<String>>` statics.
macro_rules! fsets {
    ($($n:ident),* $(,)?) => { $(
        pub static $n: LazyLock<HashSet<String>> = LazyLock::new(|| bundle_set(stringify!($n)));
    )* };
}
/// `bot.memory.facts.<NAME>` compiled regexes.
macro_rules! fres {
    ($($n:ident),* $(,)?) => { $(
        pub static $n: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.memory.facts.", stringify!($n))));
    )* };
}
/// `bot.memory.facts.<NAME>` tuples of strings, in order.
macro_rules! fstrs {
    ($($n:ident),* $(,)?) => { $(
        pub static $n: LazyLock<Vec<String>> = LazyLock::new(|| bundle_strs(stringify!($n)));
    )* };
}
/// `bot.memory.facts.<NAME>` {str: str} dicts.
macro_rules! fmaps {
    ($($n:ident),* $(,)?) => { $(
        pub static $n: LazyLock<HashMap<String, String>> = LazyLock::new(|| bundle_map(stringify!($n)));
    )* };
}

/// An inline `re.compile(pattern, flags)` (or a `re.search(pattern, ...)` in a function body).
macro_rules! inline_re {
    ($name:ident, $pat:expr, $flags:expr) => {
        static $name: LazyLock<Re> = LazyLock::new(|| Re::new($pat, $flags));
    };
}

const I: u32 = crate::py::re::I;

// ---------------------------------------------------------------------------------------------
// Small Python idioms

/// `" ".join(str(text or "").split())`.
fn squash(text: &str) -> String {
    pt::squash(text)
}

/// `" ".join(str(text or "").lower().split())`.
fn low_squash(text: &str) -> String {
    pt::squash(&pt::lower(text))
}

/// `text.lower()`.
fn lower(text: &str) -> String {
    pt::lower(text)
}

/// The byte offset `n` characters before byte `pos` (Python's `max(0, start - n)`).
fn back_chars(text: &str, pos: usize, n: usize) -> usize {
    let head = &text[..pos];
    match head.char_indices().rev().nth(n.saturating_sub(1)) {
        Some((b, _)) if n > 0 => b,
        _ if n == 0 => pos,
        _ => 0,
    }
}

/// The byte offset `n` characters after byte `pos`, clamped to the end (`text[pos:pos + n]`).
fn fwd_chars(text: &str, pos: usize, n: usize) -> usize {
    let pos = pos.min(text.len());
    match text[pos..].char_indices().nth(n) {
        Some((b, _)) => pos + b,
        None => text.len(),
    }
}

/// A dict value from (key, value) pairs, in order.
fn dict(pairs: Vec<(&str, Value)>) -> Value {
    Value::Dict(Arc::new(pairs.into_iter().map(|(k, v)| (Value::from(k), v)).collect()))
}

/// A list of strings as a Value.
fn str_list<S: AsRef<str>>(items: &[S]) -> Value {
    Value::List(Arc::new(items.iter().map(|s| Value::from(s.as_ref())).collect()))
}

/// `word[:1].upper() + word[1:]`.
fn up1(word: &str) -> String {
    pt::upper_first(word)
}

/// `any(char.isupper() for char in s)`.
fn any_upper(s: &str) -> bool {
    s.chars().any(char::is_uppercase)
}

/// The value of a dict key, or None.
fn vget<'a>(v: &'a Value, key: &str) -> &'a Value {
    static NONE: Value = Value::None;
    v.get(key).unwrap_or(&NONE)
}

// ---------------------------------------------------------------------------------------------
// names

fsets!(
    NOT_NAMES, NAMES_THAT_LOOK_LIKE_STATES, NOT_NAMES_EXTRA, NOT_A_NAME_FEELING, NOT_A_NAME_LAUGH,
    CONDITIONS_OF_THE_MIND, NOT_A_NAME_ILLNESS, NOT_A_NAME_TRAIT, NOT_A_NAME_DESCRIBING,
    NOT_A_NAME_CONDITION, NOT_A_NAME_STATE, NAME_FILLER, NAME_FOLLOWERS, NAME_PLACE_WORDS,
    PLACE_JOIN, NAME_SAYS_ABOUT_ME, GO_BY_WAYS, MONTHS, DAYS, COUNTRIES, NOT_POSITIONS, NOT_PLACES,
    NOT_A_FACT, ASK_WORDS, AFTER_THEN, HOME_STOP, GROWN_NOT_PLACE, GROWN_ARTICLE, PLACE_MIDDLE,
    PLACE_NAME_FIRST, COUNTRY_ARTICLE, PLACE_VERBS, NOT_AN_ALLERGEN, DRINK, MEAT, ANIMALS,
    DIET_FREE_WORDS, ONE_MORE_FOODS, DIET_WORDS, SOBER_WORDS, DIET_WHAT, EATING_VERB,
    DIET_STOP_A_WAY_OF_EATING, NO_EAT_QUALIFIER, NOT_A_NAME, NOT_THEIR_DETERMINER,
    NOT_THE_VERB_NAME, NOT_A_PRONOUN_NAME, NAME_AFTER_TITLE, NOT_A_NAME_MIDDLE, PET_ANIMALS,
    NOT_A_PET_NAME, A_COMMENT_ON_THE_NAME, NOT_A_NAME_THING, JOB_WORDS, NUMBER_WORDS,
    PLURAL_PRONOUN,
);
fstrs!(NAME_FILLER_PHRASES, NAME_ASIDES, DIET_WHAT_TWO);
fmaps!(COUNTRY_CAPS, DIET_NAMES, ALLERGEN_PLURALS, QUIT_AS_DOING, ANIMAL_KINDS);
fres!(
    NAME_ANSWER_HEDGES, GOES_BY_NAME, CLAUSE_SPLIT, PLACE_AFTER, PLACE_LOWER, HOME_DWELL,
    HOME_PLACE, FROM_PLACE, FROM_MORE, FROM_GROWN, GROWN_NEAR, PLACE_LOWER_NAMED, HOME_LOOSE,
    SHORT_LIST, DENIAL_WORDS, DENIAL_OF, DIET_OFF, CLAUSE_EDGE, NOT_HAVE, NOT_HAVE_LONGER,
    HAVE_NONE, A_COMPOUND_AFTER, WITHOUT_ANIMAL, NOT_JUST, TAKEN_BACK_BEFORE, BARE_NO,
    NO_THEN_A_THING, GIVEN_AWAY, NO_PETS, DIED, ANIMAL_DEATH, LOSS_ALL, PET_PRONOUN, HEDGE,
    ALLERGY, CONTRAST, CANNOT_EAT, X_ALLERGY, VEGETARIAN, NO_MEAT, NO_DRINK, GAVE_UP_DRINK,
    GAVE_UP_DRINK_THING, NOT_DRINKING, NO_DRINK_THING, OTHER_DIET, THEIR_BACK, A_CORRECTION,
    A_NEGATOR_IN_BETWEEN, ONE_MORE_HEAD, ONE_MORE_TAIL, NOT_THEIRS, A_WISH, NOT_THEIRS_AFTER,
    THEIRS_WORD, COMING_HOME, ON_ITS_WAY_BETWEEN, CONDITION, DIET_NAMED, DIET_UNDERWAY,
    DIET_BARE, DIET_DOING, DIET_QUIT, DIET_BACK, PETS, PET_COUNT, NAME_WORD, ANOTHER_ANIMAL,
    ALSO_HAVE, PET_TITLE, PET_VERB_NAME, PET_PRONOUN_NAME, PET_NAME, PET_IS_NAME,
    PET_POSSESSIVE_NAME, PET_APPOSITIVE, PET_BEFORE_FACT, PET_NAME_ONLY, PET_NAME_ONLY_LOWER,
    PET_NAME_ASKED, USER_NAME_ASKED,
);

/// `ANNOUNCED_NAME_PATTERNS` and `GUESSED_NAME_PATTERNS` (tuples of compiled patterns).
pub static ANNOUNCED_NAME_PATTERNS: LazyLock<Vec<&'static Re>> = LazyLock::new(|| {
    data::get("bot.memory.facts.ANNOUNCED_NAME_PATTERNS").items().iter().map(|v| v.as_re().unwrap()).collect()
});
pub static GUESSED_NAME_PATTERNS: LazyLock<Vec<&'static Re>> = LazyLock::new(|| {
    data::get("bot.memory.facts.GUESSED_NAME_PATTERNS").items().iter().map(|v| v.as_re().unwrap()).collect()
});

/// `PET_COUNT_WORDS`: the count each word gives.
pub static PET_COUNT_WORDS: LazyLock<HashMap<String, i64>> = LazyLock::new(|| {
    data::get("bot.memory.facts.PET_COUNT_WORDS")
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_int()?)))
        .collect()
});

pub const EVERYTHING: &str = "*";

/// Python: bot/memory/facts.py:168 `_health_words()`: the words the reading engine already uses
/// for an illness, as whole words.
pub fn _health_words() -> Value {
    let strong = data::get("bot.understanding.topics.STRONG");
    let phrases = data::get("bot.understanding.topics.PHRASES");
    let mut words: HashSet<String> = pt::split(strong.get("health").map(Value::str).unwrap_or(""))
        .into_iter()
        .collect();
    if let Some(p) = phrases.get("health") {
        for phrase in p.items() {
            words.extend(pt::split(phrase.str()));
        }
    }
    words.extend(bundle_set("bot.understanding.topics.CONDITIONS"));
    words.extend(CONDITIONS_OF_THE_MIND.iter().cloned());
    let mut sorted: Vec<String> = words.into_iter().collect();
    sorted.sort();
    Value::Set(Arc::new(sorted.into_iter().map(Value::from).collect()))
}

/// Python: bot/memory/facts.py:266 `without_the_hedges(text)`: a name answer with its hedges
/// taken off: "its sam by the way" -> "its sam".
pub fn without_the_hedges(text: &str) -> String {
    let low = low_squash(text);
    let cut = NAME_ANSWER_HEDGES.sub(" ", &low);
    pt::strip_chars(&squash(&cut), " ,.:;!?").to_string()
}

inline_re!(SPACE_OR_COMMA_RUN, r"([\s,]+)", 0);

/// Python: bot/memory/facts.py:356 `_fact_value(word)`: the thing a pattern caught, cut short
/// where the question starts: "" when there is none.
pub fn _fact_value(word: &str) -> String {
    let parts = SPACE_OR_COMMA_RUN.split(pt::strip(word));
    let words: Vec<&String> = parts.iter().step_by(2).collect();
    let piece = |i: usize| -> String {
        let mut s = parts[2 * i].clone();
        if 2 * i + 1 < parts.len() {
            s.push_str(&parts[2 * i + 1]);
        }
        s
    };
    let mut kept: Vec<String> = Vec::new();
    for (i, value) in words.iter().enumerate() {
        let vlow = lower(value);
        if AFTER_THEN.contains(&vlow) && i > 0 && lower(words[i - 1]) == "then" {
            kept.push(piece(i));
            continue;
        }
        if ASK_WORDS.contains(&vlow) {
            if vlow == "then" && i + 1 < words.len() {
                kept.push(piece(i));
                continue;
            }
            break;
        }
        kept.push(piece(i));
    }
    let joined = kept.concat();
    let mut out = pt::strip_chars(&joined, " ,;:").to_string();
    if lower(&out).ends_with(" then") {
        out = pt::rstrip_chars(&pt::slice(&out, 0, Some(-5)), " ,;:").to_string();
    }
    out
}

/// Python: bot/memory/facts.py:502 `grown_up_in_a_place(phrase)`: true when what follows "grew
/// up in" names a place, so "care" and "a council estate" do not.
pub fn grown_up_in_a_place(phrase: &str) -> bool {
    _grown_up_in_a_place(phrase, "")
}

/// Python: bot/memory/facts.py:511 `_grown_up_in_a_place(phrase, text)`.
pub fn _grown_up_in_a_place(phrase: &str, text: &str) -> bool {
    let _ = text;
    let words = pt::split(phrase);
    if words.is_empty() {
        return false;
    }
    let low: Vec<String> = words.iter().map(|w| pt::strip_chars(&lower(w), ".,!?;:'\"").to_string()).collect();
    if low.iter().any(|w| GROWN_NOT_PLACE.contains(w)) {
        return false;
    }
    if let Some(near) = GROWN_NEAR.search(phrase) {
        if !_place_name(near.str(1)).is_empty() {
            return true;
        }
    }
    if GROWN_ARTICLE.contains(&low[0]) {
        return false;
    }
    if low[0] == "the" {
        return low.len() > 1 && COUNTRY_ARTICLE.contains(&low[1]);
    }
    true
}

static LEXICON: LazyLock<HashSet<String>> = LazyLock::new(|| bundle_set("bot.understanding.normalize.LEXICON"));

/// Python: bot/memory/facts.py:552 `_an_everyday_word(word)`: whether the word is one of the
/// language's own, so it is not part of a place's name.
pub fn _an_everyday_word(word: &str) -> bool {
    let low = lower(word);
    if low.is_empty() {
        return true;
    }
    if LEXICON.contains(&low) {
        return true;
    }
    for tail in ["es", "s"] {
        if low.ends_with(tail) && pt::len(&low) - tail.len() > 2 && LEXICON.contains(&low[..low.len() - tail.len()]) {
            return true;
        }
    }
    false
}

inline_re!(SPACE_COMMA_SPLIT, r"[ ,]+", 0);

/// Python: bot/memory/facts.py:849 `_the_drink_they_do_not_take(text)`: what the line says they
/// do not drink, as the word a diet says "no" to, or "".
pub fn _the_drink_they_do_not_take(text: &str) -> String {
    let low = squash(text);
    if low.is_empty() {
        return String::new();
    }
    if NOT_DRINKING.is_match(&low) || GAVE_UP_DRINK.is_match(&low) || NO_DRINK.is_match(&low) {
        return "alcohol".to_string();
    }
    for pattern in [&*NO_DRINK_THING, &*GAVE_UP_DRINK_THING] {
        if let Some(drank) = pattern.search(&low) {
            let thing = _finish(drank.name_str("thing"));
            let words: Vec<String> =
                SPACE_COMMA_SPLIT.split(&lower(&thing)).into_iter().filter(|w| w != "or" && w != "and").collect();
            if !words.is_empty() && words.iter().all(|w| DRINK.contains(w)) {
                return thing;
            }
        }
    }
    String::new()
}

/// `CLAUSE_EDGE.split(text[max(0, start - 60):start])[-1]`: the clause the position ends.
fn clause_before(text: &str, start: usize) -> String {
    let from = back_chars(text, start, 60);
    CLAUSE_EDGE.split(&text[from..start]).pop().unwrap_or_default()
}

/// Python: bot/memory/facts.py:1016 `_coming_home(text, start)`: true when the animal in the
/// line is one they are getting, not one they have. (`start` is a byte offset.)
pub fn _coming_home(text: &str, start: usize) -> bool {
    let window = clause_before(text, start);
    for cue in COMING_HOME.finditer(&window) {
        let between = &window[cue.end()..];
        if ON_ITS_WAY_BETWEEN.match_(between).is_none() {
            continue;
        }
        return true;
    }
    false
}

/// Python: bot/memory/facts.py:1027 `_only_wanted(text, start, end)`: true when the animal is one
/// they want, dislike or had once, not one they have. (Byte offsets.)
pub fn _only_wanted(text: &str, start: usize, end: usize) -> bool {
    let window = clause_before(text, start);
    for cue in NOT_THEIRS.finditer(&window) {
        if THEIRS_WORD.is_match(&window[cue.end()..]) {
            continue;
        }
        return true;
    }
    let end = end.min(text.len());
    NOT_THEIRS_AFTER.match_(&text[end..fwd_chars(text, end, 30)]).is_some()
}

/// Python: bot/memory/facts.py:1045 `_not_theirs(animal, text, start, end)`: the record for an
/// animal the person does not have, marked, and a wish marked as a wish.
pub fn _not_theirs(animal: &str, text: &str, start: usize, end: usize) -> Value {
    let _ = end;
    let mut record = dict(vec![("animal", animal.into()), ("name", Value::None), ("not_theirs", true.into())]);
    if _a_wish_for(text, start) {
        record.set_key("wanted", true);
    }
    record
}

/// Python: bot/memory/facts.py:1053 `_a_wish_for(text, start)`: true when the animal is one they
/// would like rather than one taken away.
pub fn _a_wish_for(text: &str, start: usize) -> bool {
    let window = clause_before(text, start);
    if THEIRS_WORD.is_match(&window) {
        return false;
    }
    A_WISH.is_match(&window)
}

/// Python: bot/memory/facts.py:1122 `_the_food_named(raw)`: the food a phrase about eating names,
/// or "" when the words are not something to eat.
pub fn _the_food_named(raw: &str) -> String {
    let words = pt::split(&low_squash(raw));
    if words.is_empty() {
        return String::new();
    }
    let two = words.iter().take(2).cloned().collect::<Vec<_>>().join(" ");
    if !DIET_WHAT.contains(&words[0]) && !DIET_WHAT_TWO.contains(&two) {
        return String::new();
    }
    let mut kept: Vec<&str> = Vec::new();
    for word in &words {
        if DIET_STOP_A_WAY_OF_EATING.contains(word) {
            break;
        }
        kept.push(word);
    }
    kept.join(" ")
}

/// Python: bot/memory/facts.py:1167 `_a_way_of_eating_said_with_a_verb(text)`: the way of eating
/// a line says with a verb, as they said it, or "".
pub fn _a_way_of_eating_said_with_a_verb(text: &str) -> String {
    for m in DIET_DOING.finditer(text) {
        let food = _the_food_named(m.name_str("what"));
        if food.is_empty() {
            continue;
        }
        return lower(m.name_str("doing")) + " " + &food;
    }
    for m in DIET_QUIT.finditer(text) {
        let what = pt::split(m.name_str("what"));
        let mut food = _the_food_named(&what.join(" "));
        if food.is_empty() && EATING_VERB.contains(&what[0]) {
            food = _the_food_named(&what[1..].join(" "));
        }
        if food.is_empty() {
            continue;
        }
        let doing = lower(m.name_str("doing"));
        let said = QUIT_AS_DOING.get(&doing).cloned().unwrap_or(doing);
        return said + " " + &food;
    }
    String::new()
}

/// Python: bot/memory/facts.py:1300 `_with_title(match)` for a match given as a Value: a dict
/// holding the match's "title" and "name" groups (`match.groupdict()`).
pub fn _with_title(r#match: &Value) -> String {
    let title = r#match.get("title").and_then(Value::as_str).filter(|s| !s.is_empty());
    let name = r#match.get("name").map(Value::str).unwrap_or("");
    with_title(title, name)
}

/// `_with_title(match)` on a regex match with "title" and "name" groups.
pub fn _with_title_m(m: &Match) -> String {
    with_title(m.name("title").filter(|s| !s.is_empty()), m.name_str("name"))
}

/// The name a "called"/"named" match gives, with its title kept: "Mr Whiskers", "Lady Grey".
pub fn with_title(title: Option<&str>, name: &str) -> String {
    let Some(title) = title else {
        return name.to_string();
    };
    let nlow = lower(name);
    if NOT_A_NAME.contains(&nlow) && !NAME_AFTER_TITLE.contains(&nlow) {
        return pt::rstrip_chars(title, ".").to_string();
    }
    let both = format!("{title} {name}");
    if any_upper(&both) {
        both
    } else {
        pt::split(&both).iter().map(|w| up1(w)).collect::<Vec<_>>().join(" ")
    }
}

/// Python: bot/memory/facts.py:1319 `_name_at(match)`: where a "called" match names the animal
/// (a byte offset): the title, when there is one, comes first.
pub fn _name_at(m: &Match) -> usize {
    if m.name("title").is_some_and(|t| !t.is_empty()) {
        m.name_span("title").unwrap().0
    } else {
        m.name_span("name").map(|s| s.0).unwrap_or(0)
    }
}

inline_re!(LETTER_RUNS, r"[a-z]+", 0);

/// Python: bot/memory/facts.py:1380 `another_animal(text, animal)`: a second, different kind of
/// animal in the same line, or "".
pub fn another_animal(text: &str, animal: &str) -> String {
    let kind = ANIMAL_KINDS.get(&lower(pt::strip(animal))).cloned().unwrap_or_default();
    if kind.is_empty() {
        return String::new();
    }
    for word in LETTER_RUNS.findall_str(&lower(text)) {
        if let Some(other) = ANIMAL_KINDS.get(&word) {
            if !other.is_empty() && *other != kind {
                return other.clone();
            }
        }
    }
    String::new()
}

/// Python: bot/memory/facts.py:1429 `_a_name_for_the_pet(word, strict_ly=True)`: the word a pet's
/// name was given as, or None when the word only comments on the name.
pub fn _a_name_for_the_pet(word: &str, strict_ly: bool) -> Option<String> {
    let low = lower(word);
    if word.is_empty() || _not_a_name(word, strict_ly) || A_COMMENT_ON_THE_NAME.contains(&low) {
        return None;
    }
    Some(word.to_string())
}

static TRAVEL_CITIES: LazyLock<HashSet<String>> = LazyLock::new(|| bundle_set("bot.knowledge.travel.CITIES"));
static COSTS_JOBS: LazyLock<HashSet<String>> = LazyLock::new(|| bundle_set("bot.knowledge.costs.JOBS"));

/// Python: bot/memory/facts.py:1475 `_names_something_else(low)`: whether a word on its own
/// names a thing rather than a person.
pub fn _names_something_else(low: &str) -> bool {
    if low.is_empty() {
        return true;
    }
    if DAYS.contains(low)
        || MONTHS.contains(low)
        || COUNTRIES.contains(low)
        || NOT_PLACES.contains(low)
        || NUMBER_WORDS.contains(low)
        || JOB_WORDS.contains(low)
    {
        return true;
    }
    TRAVEL_CITIES.contains(low) || COSTS_JOBS.contains(low)
}

// ---------------------------------------------------------------------------------------------
// a name said on its own, budgets, ages, heights, birthdays, weights, relations

fres!(
    NAME_ANSWER_ANNOUNCED, NAME_ANSWER_ANNOUNCED_LAST, NAMES_OFFERED, NAME_PICK_ASK, BUDGET,
    SPEND_BUDGET, INTENDED_SPEND, THE_THING_MAY_COST, ALL_I_CAN_DO, SPEND_FOR, ONLY_HAVE,
    SPEND_BUDGET_NOW, STRETCH_BUDGET, INCOME_PERIOD, INCOME_QUALIFIER, INCOME_RATE, INCOME,
    INCOME_BARE, BARE_BUDGET, CEILING_BUDGET, BUDGET_THOUGHT, BUDGET_ASKED, VAGUE_BUDGET, MONEY,
    AGE, HEIGHT_FEET, HEIGHT_BARE, HEIGHT_CM, AGE_IN_WORDS, BEFORE_A_FIGURE, AFTER_A_FIGURE,
    NOT_YEARS_AFTER, BIRTHDAY, A_DATE_CLAUSE_ENDS, WEIGH, WEIGH_IM, WEIGH_MY, MY_AGE, PEOPLE_WORD,
    IN_LAW_WORD, IN_LAW_AFTER, RELATION_NAME, RELATION_HAVE_NAME, RELATION_POSSESSIVE_NAME,
    RELATION_NAMED_AGE, RELATION_POSSESSIVE_AGE, RELATION_AGE, THE_PLACE_AGE, BABY_AGE,
    MONTH_OLD_BABY, RELATION_BARE_NAME, RELATION_PAIR_NAMES, GROUP_NAMED_NAMES, GROUP_NAMED_AGES,
    RELATION_WHOS_AGE, CHAINED_RELATION_AGE, CHAINED_PERSON, HOW_THEY_FEEL, PRONOUN_AGE,
);
fsets!(COMPOUND_AFTER, COMPOUND_THAT_IS_THEIRS, TITLE_WORDS, _A_WORD_FOR_A_PERSON);
fstrs!(_STEP_OF, _GRAND_OF, _BUILT_KIN, RELATIONS, ROLE_PEOPLE, KIN, ALONE_PERSON, PERSON_MODIFIER, KIN_WORDS);
fmaps!(INCH_WORDS, BIRTHDAY_PRONOUN, WEIGH_UNITS, SPOKEN_RELATION, TITLE_GENDER);

/// `INCOME_A_YEAR`: a tuple of compiled patterns.
pub static INCOME_A_YEAR: LazyLock<Vec<&'static Re>> = LazyLock::new(|| {
    data::get("bot.memory.facts.INCOME_A_YEAR").items().iter().map(|v| v.as_re().unwrap()).collect()
});

/// `_NUMBER_IN_WORDS`: the number each word for one says.
pub static _NUMBER_IN_WORDS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| {
    data::get("bot.memory.facts._NUMBER_IN_WORDS")
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_int()?)))
        .collect()
});

fn number_in_words(word: &str) -> i64 {
    _NUMBER_IN_WORDS.iter().find(|(k, _)| k == word).map(|(_, v)| *v).unwrap_or(0)
}

/// Python: bot/memory/facts.py:1498 `reads_as_a_name(word)`: whether a word on its own, in
/// answer to a name question, is what a person is called. Returns the name in its spelled
/// form, or None.
pub fn reads_as_a_name(word: &str) -> Option<String> {
    if bare_name(word).is_none() {
        return None;
    }
    let low = _finish(word);
    if low.is_empty() || NOT_A_NAME_THING.contains(&low) || NOT_A_NAME_TRAIT.contains(&low) || _names_something_else(&low) {
        return None;
    }
    let name = _name(&format!("im {low}"));
    if let Some(name) = name {
        if !name.is_empty() && _finish(&name) == low {
            return Some(name);
        }
    }
    None
}

/// Python: bot/memory/facts.py:1724 `vague_budget(text)`: the limit a line states with no figure
/// in it, as the store says it back, or None.
pub fn vague_budget(text: &str) -> Option<String> {
    let m = VAGUE_BUDGET.match_(text)?;
    let said = pt::strip_chars(&low_squash(m.as_str()), " .,;:!").to_string();
    if said.contains("cheap") && said.starts_with("on ") {
        return Some("on the cheap".into());
    }
    if said.contains("cheap") || said.contains("reasonable") {
        return Some("cheap".into());
    }
    if said.starts_with("budget") {
        return Some(said);
    }
    Some("not too expensive".into())
}

inline_re!(DASH_OR_SPACE_RUN, r"[-\s]+", 0);

/// Python: bot/memory/facts.py:1809 `_the_numbers_in_words(text)`: the line with an age said in
/// words written as figures, so the age readers can read it.
pub fn _the_numbers_in_words(text: &str) -> String {
    if text.is_empty() {
        return text.to_string();
    }
    let tl = lower(text);
    if !_NUMBER_IN_WORDS.iter().any(|(w, _)| tl.contains(w.as_str())) {
        return text.to_string();
    }
    AGE_IN_WORDS.sub_fn(text, |m| {
        if !BEFORE_A_FIGURE.is_match(&text[..m.start()]) {
            return m.as_str().to_string();
        }
        let after = &text[m.end()..];
        if NOT_YEARS_AFTER.match_(after).is_some() || AFTER_A_FIGURE.match_(after).is_none() {
            return m.as_str().to_string();
        }
        let total: i64 = DASH_OR_SPACE_RUN.split(&lower(m.name_str("words"))).iter().map(|p| number_in_words(p)).sum();
        if total != 0 {
            total.to_string()
        } else {
            m.as_str().to_string()
        }
    })
}

/// Python's `int(s)` for a string of digits (None when it is not one).
fn py_int(s: &str) -> Option<i64> {
    let t = pt::strip(s);
    if t.is_empty() {
        return None;
    }
    let mut n: i64 = 0;
    for c in t.chars() {
        let d = c.to_digit(10)? as i64;
        n = n.checked_mul(10)?.checked_add(d)?;
    }
    Some(n)
}

/// Python: bot/memory/facts.py:1827 `_how_tall(text)`: the height a line states as the words to
/// say it back in, or "".
pub fn _how_tall(text: &str) -> String {
    if let Some(m) = HEIGHT_CM.search(text) {
        return format!("{}cm", m.name_str("cm"));
    }
    let Some(m) = HEIGHT_FEET.search(text).or_else(|| HEIGHT_BARE.match_(text)) else {
        return String::new();
    };
    let raw = m.name_str("inches");
    let inches = INCH_WORDS.get(&lower(raw)).cloned().unwrap_or_else(|| raw.to_string());
    if !inches.is_empty() && !pt::is_digit(&inches) {
        return String::new();
    }
    let n = if inches.is_empty() { None } else { py_int(&inches) };
    if let Some(n) = n {
        if n >= 12 {
            return String::new();
        }
    }
    let mut out = format!("{} foot", m.name_str("feet"));
    if let Some(n) = n {
        out.push_str(&format!(" {n}"));
    }
    out
}

/// Python: bot/memory/facts.py:1868 `_the_date_clause(date)`: the date as far as the clause it
/// is said in goes: "may and my sons is in july" -> "may".
pub fn _the_date_clause(date: &str) -> String {
    A_DATE_CLAUSE_ENDS.split(date).into_iter().next().unwrap_or_default()
}

inline_re!(IN_OR_ON_FIRST, r"^(?:in|on)\s+", 0);
inline_re!(NOT_ALNUM_RUN, r"[^a-z0-9]+", 0);
inline_re!(A_DIGIT, r"\d", 0);
inline_re!(NOT_A_DATE_WORD, r"\b(?:to be|going|quiet|coming|soon|weekend|week|tomorrow|today)\b", 0);

/// Python: bot/memory/facts.py:1873 `_a_birthday_date(after)`: the date words of a birthday
/// clause, or "" when they name no date at all.
pub fn _a_birthday_date(after: &str) -> String {
    let date = _the_date_clause(&low_squash(after));
    let date = pt::rstrip_chars(pt::strip(&date), " .,;:").to_string();
    let date = IN_OR_ON_FIRST.sub("", &date);
    let looks_like_a_date =
        NOT_ALNUM_RUN.split(&date).iter().any(|w| MONTHS.contains(w)) || A_DIGIT.is_match(&date);
    if date.is_empty() || !looks_like_a_date {
        return String::new();
    }
    if NOT_A_DATE_WORD.is_match(&date) {
        return String::new();
    }
    date
}

/// Python: bot/memory/facts.py:1898 `_weight_unit(word)`: the unit a weight was said in, however
/// they pluralised it, or None.
pub fn _weight_unit(word: &str) -> Option<String> {
    let word = lower(word);
    let mut unit = WEIGH_UNITS.get(&word).cloned();
    if unit.is_none() && word.ends_with('s') {
        unit = WEIGH_UNITS.get(&word[..word.len() - 1]).cloned();
    }
    unit
}

/// Python: bot/memory/facts.py:2041 `an_in_law(word)`: whether a person's word names an in-law.
pub fn an_in_law(word: &str) -> bool {
    IN_LAW_WORD.match_(pt::strip(word)).is_some()
}

inline_re!(POSSESSIVE_END, "(?:'s|\u{2019}s|s)$", 0);

/// Python: bot/memory/facts.py:2206 `a_chained_person(text)`: "partner's mum" for "my partners
/// mum", or "" when the line names no pair of people.
pub fn a_chained_person(text: &str) -> String {
    let Some(found) = CHAINED_PERSON.search(text) else {
        return String::new();
    };
    let first = POSSESSIVE_END.sub("", &lower(found.name_str("first")));
    format!("{first}'s {}", lower(found.name_str("second")))
}

/// Python: bot/memory/facts.py:2256 `a_stated_number(word)`: the number a person wrote as a word
/// or a digit, or None when it says no number at all.
pub fn a_stated_number(word: &str) -> Option<i64> {
    let said = lower(pt::strip(word));
    if said.is_empty() {
        return None;
    }
    if pt::is_digit(&said) {
        return py_int(&said);
    }
    let total: i64 = DASH_OR_SPACE_RUN.split(&said).iter().map(|p| number_in_words(p)).sum();
    if total != 0 {
        Some(total)
    } else {
        None
    }
}

inline_re!(SPACED_DASH, r"[ \t]*-[ \t]*", 0);
inline_re!(IN_LAWS_END, r"\bin laws$", 0);

/// Python: bot/memory/facts.py:2267 `a_person_word(word)`: a person's word said plainly:
/// "brother in law" and "brother-in-law" are one person.
pub fn a_person_word(word: &str) -> String {
    let plain = squash(&SPACED_DASH.sub(" ", &lower(pt::strip(word))));
    IN_LAWS_END.sub("in law", &plain)
}

/// Python: bot/memory/facts.py:2277 `_the_person(found, who, name="")`: the entry for one person
/// (its index in `found`), when the word for the relation can be more than one of them.
pub fn _the_person(found: &[Value], who: &str, name: &str) -> Option<usize> {
    let who = a_person_word(who);
    for (i, one) in found.iter().enumerate() {
        if one.get("who").and_then(Value::as_str) != Some(who.as_str()) {
            continue;
        }
        if !name.is_empty() {
            if let Some(n) = one.get("name") {
                if n.truthy() && lower(&value_str(n)) != lower(name) {
                    continue;
                }
            }
        }
        return Some(i);
    }
    None
}

/// Python's `str(value)` for the values a fact holds (strings, numbers, None).
fn value_str(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => pt::float_repr(*f),
        Value::Bool(b) => if *b { "True".into() } else { "False".into() },
        Value::None => "None".into(),
        _ => String::new(),
    }
}

// ---------------------------------------------------------------------------------------------
// relations: names and ages of the people in the person's life

fres!(OTHER_BIRTHDAY, AGE_TAIL, CLAUSE_BREAK, PRONOUN_SUBJECT, RELATION_SAID, NAME_THEN_THE_VERB, NAME_AND_A_FIGURE);
fsets!(DIET_GROUP, SHE, HE, _SHE_SAID, _HE_SAID, NOT_A_PERSON_WORD, A_PRONOUN_NOT_A_NAME, NOT_A_NAME_FIGURE);
fstrs!(OTHER_NOUNS);

/// `taste.read_other(text)` as the list of dicts Python returns.
///
/// CONTRACT NOTE: the skeleton gives `taste::read_other` the return type `Vec<String>`, but
/// PyBot returns a list of dicts (`[{"who"/"pronoun", "thing", "how", "eaten"}]`), which a list
/// of strings cannot carry. This adapter compiles against either shape, so the call keeps
/// working when the owner gives it the type that holds the dicts.
fn taste_read_other(text: &str) -> Vec<Value> {
    trait IntoTastes {
        fn into_tastes(self) -> Vec<Value>;
    }
    impl IntoTastes for Vec<String> {
        fn into_tastes(self) -> Vec<Value> {
            self.into_iter().map(Value::from).collect()
        }
    }
    impl IntoTastes for Vec<Value> {
        fn into_tastes(self) -> Vec<Value> {
            self
        }
    }
    impl IntoTastes for Value {
        fn into_tastes(self) -> Vec<Value> {
            self.items().to_vec()
        }
    }
    crate::memory::taste::read_other(text).into_tastes()
}

inline_re!(SENTENCE_END, r"[.!?]", 0);
inline_re!(COMMA_OR_AND, r"\s*(?:,|and)\s*", 0);
inline_re!(NAME_THEN_AGE, r"([A-Za-z][A-Za-z'-]{1,19})\s+(\d{1,3})", 0);

/// `int(text)` for a regex group of digits.
fn int_of(s: &str) -> i64 {
    py_int(s).unwrap_or(0)
}

/// Python: bot/memory/facts.py:2295 `_relation_facts(text)`: a relation's own name and age:
/// [{"who": "mum", "name": "Janet", "age": 72}].
pub fn _relation_facts(text: &str) -> Vec<Value> {
    let mut found: Vec<Value> = Vec::new();
    let text = stated_not_asked(text);
    if text.is_empty() {
        return found;
    }
    let text = text.as_str();

    // `entry(who, name="")`: the index of this person's entry, made when there is none.
    fn entry(found: &mut Vec<Value>, who: &str, name: &str) -> usize {
        if let Some(i) = _the_person(found, who, name) {
            return i;
        }
        found.push(dict(vec![("who", a_person_word(who).into())]));
        found.len() - 1
    }

    for m in RELATION_POSSESSIVE_AGE.finditer(text) {
        let i = entry(&mut found, &lower(m.name_str("who")), "");
        found[i].set_key("age", int_of(m.name_str("age")));
    }
    for pattern in [&*RELATION_NAME, &*RELATION_POSSESSIVE_NAME, &*RELATION_NAMED_AGE, &*RELATION_HAVE_NAME] {
        for m in pattern.finditer(text) {
            let raw = m.name_str("name");
            let name = _finish(raw);
            if name.is_empty() || _not_a_name(raw, false) {
                continue;
            }
            if !_name_after_a_title(raw) {
                continue;
            }
            if _A_WORD_FOR_A_PERSON.contains(&lower(raw)) {
                continue;
            }
            let i = entry(&mut found, &lower(m.name_str("who")), &tidy(&name));
            found[i].setdefault("name", _tidy_a_titled_name(&name));
        }
    }
    for m in RELATION_BARE_NAME.finditer(text) {
        let name = _bare_name(m.name_str("name"), false);
        if !name.is_empty() {
            let i = entry(&mut found, &lower(m.name_str("who")), &name);
            found[i].setdefault("name", name.as_str());
        }
    }
    for m in RELATION_PAIR_NAMES.finditer(text) {
        let names = [_bare_name(m.name_str("name1"), true), _bare_name(m.name_str("name2"), true)];
        if names.iter().all(|n| !n.is_empty()) {
            let who = format!("{} and {}", lower(m.name_str("who1")), lower(m.name_str("who2")));
            found.push(dict(vec![("who", who.into()), ("names", str_list(&names))]));
        }
    }
    for m in GROUP_NAMED_NAMES.finditer(text) {
        let said: Vec<String> = COMMA_OR_AND.split(m.name_str("rest")).into_iter().filter(|w| !w.is_empty()).collect();
        let names: Vec<String> = said.iter().map(|w| _bare_name(w, true)).collect();
        if names.len() < 2 || !names.iter().all(|n| !n.is_empty()) {
            continue;
        }
        let mut who = lower(m.name_str("who"));
        if m.name("who2").is_some_and(|w| !w.is_empty()) {
            who = format!("{who} and {}", lower(m.name_str("who2")));
        }
        found.push(dict(vec![("who", who.into()), ("names", str_list(&names))]));
    }
    for m in GROUP_NAMED_AGES.finditer(text) {
        let pairs: Vec<(String, String)> = NAME_THEN_AGE
            .finditer(m.name_str("rest"))
            .iter()
            .map(|p| (p.str(1).to_string(), p.str(2).to_string()))
            .collect();
        if pairs.len() < 2 {
            continue;
        }
        let named: Vec<(String, String)> = pairs.iter().map(|(said, age)| (_bare_name(said, true), age.clone())).collect();
        if !named.iter().all(|(name, _)| !name.is_empty()) {
            continue;
        }
        for (name, age) in named {
            let made = dict(vec![
                ("who", lower(m.name_str("who")).into()),
                ("name", name.into()),
                ("age", int_of(&age).into()),
            ]);
            if !found.contains(&made) {
                found.push(made);
            }
        }
    }
    for m in BABY_AGE.finditer(text) {
        let said = m
            .name("who")
            .filter(|s| !s.is_empty())
            .or_else(|| m.name("little").filter(|s| !s.is_empty()))
            .unwrap_or("baby");
        let i = entry(&mut found, &lower(said), "");
        if lower(m.name_str("unit")).starts_with("month") {
            found[i].setdefault("months", int_of(m.name_str("age")));
        } else {
            found[i].setdefault("age", int_of(m.name_str("age")));
        }
    }
    for m in MONTH_OLD_BABY.finditer(text) {
        let i = entry(&mut found, "baby", "");
        found[i].setdefault("months", int_of(m.name_str("age")));
    }
    for (k, pattern) in [&*RELATION_AGE, &*THE_PLACE_AGE, &*RELATION_NAMED_AGE, &*RELATION_WHOS_AGE].into_iter().enumerate() {
        let clause = k == 3;
        for m in pattern.finditer(text) {
            let rest = &text[m.end()..];
            let months = _month_age_after(text, m.end());
            if months {
                let i = entry(&mut found, &lower(m.name_str("who")), "");
                found[i].setdefault("months", int_of(m.name_str("age")));
                continue;
            }
            if !(clause || AGE_TAIL.match_(rest).is_some()) {
                continue;
            }
            let named = m.name("name").filter(|s| !s.is_empty());
            let i = match named {
                Some(n) => entry(&mut found, &lower(m.name_str("who")), &tidy(n)),
                None => entry(&mut found, &lower(m.name_str("who")), ""),
            };
            found[i].setdefault("age", int_of(m.name_str("age")));
        }
    }
    for tasted in taste_read_other(text) {
        let thing = vget(&tasted, "thing").clone();
        if !thing.truthy() {
            continue;
        }
        let key = if vget(&tasted, "how").str() == "yes" { "likes" } else { "dislikes" };
        if vget(&tasted, "eaten").truthy() && pt::split(&value_str(&thing)).iter().any(|w| DIET_GROUP.contains(w)) {
            continue;
        }
        let who = vget(&tasted, "who");
        let i = if who.truthy() {
            entry(&mut found, &value_str(who), "")
        } else {
            let pron = vget(&tasted, "pronoun");
            match found.iter().position(|made| vget(made, "pronoun") == pron && !vget(made, "who").truthy()) {
                Some(i) => i,
                None => {
                    let p = if pron.truthy() { pron.clone() } else { Value::from("") };
                    found.push(dict(vec![("pronoun", p)]));
                    found.len() - 1
                }
            }
        };
        let held = found[i].setdefault(key, Value::list());
        if !held.items().contains(&thing) {
            held.push(thing);
        }
    }
    for m in OTHER_BIRTHDAY.finditer(text) {
        let date = _a_birthday_date(m.name_str("date"));
        if date.is_empty() {
            continue;
        }
        let said_who = lower(m.name_str("who"));
        let said_pronoun = lower(m.name_str("pronoun"));
        let i = if !said_who.is_empty() {
            entry(&mut found, &said_who, "")
        } else {
            let pronoun_word = BIRTHDAY_PRONOUN.get(&said_pronoun).cloned().unwrap_or(said_pronoun);
            let person = _the_person_the_pronoun_is(text, m.end(), &pronoun_word);
            if !person.is_empty() {
                entry(&mut found, &person, "")
            } else {
                match found
                    .iter()
                    .position(|made| vget(made, "pronoun").as_str() == Some(pronoun_word.as_str()) && !vget(made, "who").truthy())
                {
                    Some(i) => i,
                    None => {
                        found.push(dict(vec![("pronoun", pronoun_word.as_str().into())]));
                        found.len() - 1
                    }
                }
            }
        };
        found[i].setdefault("birthday", date.as_str());
    }
    let mut pronoun = PRONOUN_AGE.search(text);
    if let Some(p) = &pronoun {
        if a_stated_number(p.name_str("age")).is_none() {
            pronoun = None;
        }
    }
    if let Some(p) = &pronoun {
        if p.name("come").is_some_and(|c| !c.is_empty()) {
            let same_breath = SENTENCE_END.split(&text[p.end()..]).into_iter().next().unwrap_or_default();
            if HOW_THEY_FEEL.is_match(&same_breath) {
                pronoun = None;
            }
        }
    }
    if let Some(p) = pronoun.as_ref().filter(|p| _month_age_after(text, p.end())) {
        let age = a_stated_number(p.name_str("age"));
        let fits: &HashSet<String> = if lower(p.name_str("who")) == "she" { &SHE } else { &HE };
        let mut placed = false;
        for known in found.iter_mut() {
            if known.get("age").is_none()
                && known.get("months").is_none()
                && known.get("who").and_then(Value::as_str).is_some_and(|w| fits.contains(w))
            {
                known.set_key("months", age);
                placed = true;
                break;
            }
        }
        if !placed && !found.iter().any(|made| made.get("months").is_some()) {
            found.push(dict(vec![("months", age.into()), ("pronoun", lower(p.name_str("who")).into())]));
        }
    } else if let Some(p) = pronoun.as_ref().filter(|p| AGE_TAIL.match_(&text[p.end()..]).is_some()) {
        let word = lower(p.name_str("who"));
        let empty = HashSet::new();
        let fits: &HashSet<String> = if word == "she" {
            &SHE
        } else if word == "he" {
            &HE
        } else {
            &empty
        };
        let age = a_stated_number(p.name_str("age"));
        for known in found.iter_mut() {
            if known.get("age").is_none() && known.get("who").and_then(Value::as_str).is_some_and(|w| fits.contains(w)) {
                known.set_key("age", age);
                break;
            }
        }
        if !found.iter().any(|made| made.get("age").is_some()) {
            let person = _the_person_the_pronoun_is(text, p.start(), &word);
            if !person.is_empty() {
                found.push(dict(vec![("who", person.into()), ("age", age.into())]));
            } else {
                found.push(dict(vec![("age", age.into()), ("pronoun", word.into())]));
            }
        }
    }
    found.into_iter().filter(|known| known.len() > 1 || known.get("who").is_none()).collect()
}

/// Python: bot/memory/facts.py:2541 `_kin_pronoun(word)`: the relation inside a step- or
/// grand-word, so the pronoun it takes can be read from it.
pub fn _kin_pronoun(word: &str) -> String {
    for prefix in ["step", "grand"] {
        if let Some(rest) = word.strip_prefix(prefix) {
            if _SHE_SAID.contains(rest) || _HE_SAID.contains(rest) {
                return rest.to_string();
            }
        }
    }
    word.to_string()
}

/// `sorted(_SHE_SAID | _HE_SAID, key=len, reverse=True)`. Python's set order breaks the ties
/// between words of one length; which of two same-length words is tried first cannot change
/// the answer unless one is the start or end of the other, which equal lengths rule out.
static GENDERED_LONGEST_FIRST: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut all: Vec<String> = _SHE_SAID.union(&_HE_SAID).cloned().collect();
    all.sort();
    all.sort_by(|a, b| pt::len(b).cmp(&pt::len(a)));
    all
});

/// Python: bot/memory/facts.py:2562 `_sided(word)`: the pronoun a word for a person names from
/// inside it: "she", "he", or "" for neither.
pub fn _sided(word: &str) -> String {
    let inside = _kin_pronoun(word);
    for gendered in GENDERED_LONGEST_FIRST.iter() {
        if inside.starts_with(gendered.as_str()) || inside.ends_with(gendered.as_str()) {
            return if _SHE_SAID.contains(gendered) { "she".into() } else { "he".into() };
        }
    }
    String::new()
}

fres!(A_LIMIT, COMMITS_TO, MONEY_LIMIT);

/// Python: bot/memory/facts.py:2625 `_own_body_is_the_trouble(clean)`: true when the line is the
/// speaker's own body complaining, read by the pool that owns it (`bot/compose/badnews.py`).
pub fn _own_body_is_the_trouble(clean: &str) -> bool {
    crate::compose::badnews::reads_like_a_symptom(clean)
}

inline_re!(
    A_MOVING_LIMIT,
    r"\b(?:walk|stand|sit|climb|climbing|manage|stairs?|steps?|hills?|walking|standing)\b",
    0
);

/// Python: bot/memory/facts.py:2642 `a_limit_on_the_plan(text)`: the limit a line puts on a
/// plan - "move", "money" or "other" - or None when the line puts no limit on anything.
pub fn a_limit_on_the_plan(text: &str) -> Option<String> {
    let clean = low_squash(text);
    if clean.is_empty() || clean.contains('?') {
        return None;
    }
    let m = A_LIMIT.search(&clean)?;
    if _own_body_is_the_trouble(&clean) {
        return None;
    }
    let hit = m.as_str();
    if m.name("move").is_some_and(|s| !s.is_empty()) || A_MOVING_LIMIT.is_match(hit) {
        return Some("move".into());
    }
    Some(if MONEY_LIMIT.is_match(hit) { "money".into() } else { "other".into() })
}

fres!(PAST_DEED);
fsets!(PAST_VERBS, PAST_ARRANGED, PAST_STATES, INTENTION_AFTER);

/// Python: bot/memory/facts.py:2780 `a_deed(clause)`: whether the clause reports something the
/// person has already done.
pub fn a_deed(clause: &str) -> bool {
    let Some(m) = PAST_DEED.match_(clause) else {
        return false;
    };
    let word = lower(m.str(1));
    if PAST_ARRANGED.contains(&word) {
        return false;
    }
    if PAST_VERBS.contains(&word) {
        return true;
    }
    if word.ends_with("ed") && pt::len(&word) >= 5 && !PAST_STATES.contains(&word) {
        return !INTENTION_AFTER.contains(&lower(m.str(2)));
    }
    false
}

fres!(CLOCK_VALUE, CLOCK_IN_PLAN, DATE_VALUE);

/// Python: bot/memory/facts.py:2940 `a_clock_time(phrase)`: the clock time a phrase is, said the
/// way the person said it ("2pm", "noon"), or "".
pub fn a_clock_time(phrase: &str) -> String {
    match CLOCK_VALUE.match_(&squash(phrase)) {
        Some(m) => m.str(1).to_string(),
        None => String::new(),
    }
}

/// Python: bot/memory/facts.py:2952 `the_clock_in(text)`: the clock a plan says, as the match to
/// replace, or None.
pub fn the_clock_in(text: &str) -> Option<Match<'_>> {
    CLOCK_IN_PLAN.search(text)
}

inline_re!(A_BARE_DATE_NUMBER, r"(?:the\s+)?\d{1,2}(?:st|nd|rd|th)?", I);

/// Python: bot/memory/facts.py:2957 `a_moved_time(phrase)`: true when a phrase is a time a plan
/// can be moved to: a day, a month, a date or a clock.
pub fn a_moved_time(phrase: &str) -> bool {
    let words: Vec<String> = NOT_ALNUM_RUN.split(&lower(phrase)).into_iter().filter(|w| !w.is_empty()).collect();
    if words.is_empty() {
        return false;
    }
    if words.iter().any(|w| MONTHS.contains(w) || DAYS.contains(w)) {
        return true;
    }
    if !a_clock_time(phrase).is_empty() {
        return true;
    }
    DATE_VALUE.fullmatch(pt::strip(phrase)).is_some() || A_BARE_DATE_NUMBER.fullmatch(pt::strip(phrase)).is_some()
}

/// Python: bot/memory/facts.py:10360 `_union(one, other)`.
pub fn _union(one: &[Value], other: &[Value]) -> Vec<Value> {
    let mut out: Vec<Value> = one.to_vec();
    for value in other {
        if !out.contains(value) {
            out.push(value.clone());
        }
    }
    out
}

/// The start of the clause that ends at `pos`: after the last `CLAUSE_BREAK` before it
/// (`CLAUSE_BREAK.finditer(text, 0, pos)`).
fn clause_start(text: &str, pos: usize) -> usize {
    let mut start = 0;
    for m in CLAUSE_BREAK.finditer(&text[..pos]) {
        start = m.end();
    }
    start
}

/// Python: bot/memory/facts.py:10368 `_the_pronoun(text, pos)`: which person a clause that names
/// no relation is about: "she", "he", or "" for none. (`pos` is a byte offset.)
pub fn _the_pronoun(text: &str, pos: usize) -> String {
    let start = clause_start(text, pos);
    let Some(found) = PRONOUN_SUBJECT.search_at(&text[..pos], start) else {
        return String::new();
    };
    let word = lower(found.name_str("pron"));
    match word.as_str() {
        "her" | "shes" => "she".into(),
        "him" | "hes" => "he".into(),
        "them" | "theyre" => "they".into(),
        _ => word,
    }
}

/// Python: bot/memory/facts.py:10388 `_the_person_the_pronoun_is(text, pos, pronoun)`: the
/// relation the line named before a bare pronoun, when it names exactly one; "" otherwise.
pub fn _the_person_the_pronoun_is(text: &str, pos: usize, pronoun: &str) -> String {
    let fits: &HashSet<String> = match pronoun {
        "she" => &SHE,
        "he" => &HE,
        _ => return String::new(),
    };
    let mut who = String::new();
    for m in RELATION_SAID.finditer(&text[..pos.min(text.len())]) {
        let word = a_person_word(&lower(&squash(&format!("{}{}", m.name_str("who"), m.name_str("inlaw")))));
        if !who.is_empty() && word != who {
            return String::new();
        }
        who = word;
    }
    if who.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = DASH_OR_SPACE_RUN.split(&who).into_iter().filter(|p| !p.is_empty()).collect();
    if fits.contains(&who) || parts.iter().any(|p| fits.contains(p)) {
        who
    } else {
        String::new()
    }
}

/// Python: bot/memory/facts.py:10428 `_a_name_opening_the_clause(clause)`: the name a clause opens
/// with, or "" - "tom" in "tom is allergic to nuts".
pub fn _a_name_opening_the_clause(clause: &str) -> String {
    let Some(found) = NAME_THEN_THE_VERB.match_(clause) else {
        return String::new();
    };
    let name = found.name_str("name");
    let low = _a_person_word(name);
    if NOT_A_PERSON_WORD.contains(&low) || RELATIONS.contains(&low) || DIET_GROUP.contains(&low) || A_PRONOUN_NOT_A_NAME.contains(&low) {
        return String::new();
    }
    if !(_may_be_a_name(name) && _name_after_a_title(name)) {
        return String::new();
    }
    name.to_string()
}

static OTHER_NOUN_THEN_A_NAME: LazyLock<Re> = LazyLock::new(|| {
    let mut nouns: Vec<String> = OTHER_NOUNS.clone();
    nouns.sort_by(|a, b| pt::len(b).cmp(&pt::len(a)));
    Re::new(
        &format!(r"\b(?:{})(?:\s+in\s+laws?)?s?\s+(?P<name>[A-Za-z][A-Za-z'\-]{{1,19}})\s*$", nouns.join("|")),
        I,
    )
});

/// Python: bot/memory/facts.py:10443 `_the_name_in_front(text, pos)`: the name of the person the
/// clause ending at pos names, or "".
pub fn _the_name_in_front(text: &str, pos: usize) -> String {
    let start = clause_start(text, pos);
    let clause = &text[start..pos];
    let (who, name) = _the_person_without_a_relation(clause);
    if !name.is_empty() && who.is_empty() {
        return name;
    }
    if who.is_empty() {
        let opening = _a_name_opening_the_clause(clause);
        if !opening.is_empty() {
            return opening;
        }
    }
    if !who.is_empty() {
        if let Some(found) = OTHER_NOUN_THEN_A_NAME.search(clause) {
            let name = found.name_str("name");
            let low = pt::strip_chars(&lower(name), "'").to_string();
            if !NOT_A_PERSON_WORD.contains(&low) && !RELATIONS.contains(&low) && _may_be_a_name(name) && _name_after_a_title(name) {
                return name.to_string();
            }
        }
    }
    String::new()
}

/// Python: bot/memory/facts.py:10475 `_a_person(text, pos, who)`: the person a fact about
/// somebody else belongs to: {"who": "mum"}, {"name": "Tom"} or {"pronoun": "she"}; {} for none.
pub fn _a_person(text: &str, pos: usize, who: &str) -> Value {
    if !who.is_empty() {
        return dict(vec![("who", who.into())]);
    }
    let named = _the_name_in_front(text, pos);
    if !named.is_empty() {
        return dict(vec![("name", named.into())]);
    }
    let pronoun = _the_pronoun(text, pos);
    if pronoun.is_empty() {
        return Value::dict();
    }
    let person = _the_person_the_pronoun_is(text, pos, &pronoun);
    if !person.is_empty() {
        return dict(vec![("who", person.into())]);
    }
    dict(vec![("pronoun", pronoun.into())])
}

/// `str(value or "")`.
fn str_or_empty(v: Option<&Value>) -> String {
    match v {
        Some(v) if v.truthy() => value_str(v),
        _ => String::new(),
    }
}

/// Python: bot/memory/facts.py:10501 `_same_person(seen, entry)`: whether two readings of one
/// line are about the same person.
pub fn _same_person(seen: &Value, entry: &Value) -> bool {
    let who = str_or_empty(seen.get("who"));
    let other = str_or_empty(entry.get("who"));
    if !who.is_empty() || !other.is_empty() {
        let mine = lower(&str_or_empty(seen.get("name")));
        let theirs = lower(&str_or_empty(entry.get("name")));
        return who == other && (mine.is_empty() || theirs.is_empty() || mine == theirs);
    }
    let sp = vget(seen, "pronoun");
    sp.truthy() && sp == vget(entry, "pronoun")
}

/// `_tidy(word)` as a string: stripped, first letter upper-cased.
pub fn tidy(word: &str) -> String {
    let word = pt::strip_chars(pt::strip(word), ".,!?;:'\"");
    up1(word)
}

/// Python: bot/memory/facts.py:4277 `_tidy(word)` (the contract's Value: always a str).
pub fn _tidy(word: &str) -> Value {
    Value::from(tidy(word))
}

/// Python: bot/memory/facts.py:4282 `_tidy_a_titled_name(word)`: a name that starts with a title
/// is written the way a name is: "mr smith" -> "Mr Smith".
pub fn _tidy_a_titled_name(word: &str) -> String {
    let words = pt::split(word);
    if words.len() > 1 && TITLE_WORDS.contains(&lower(pt::strip_chars(&words[0], "."))) {
        return words.iter().map(|w| tidy(w)).collect::<Vec<_>>().join(" ");
    }
    tidy(word)
}

// ---------------------------------------------------------------------------------------------
// corrections: the shapes a value is put right in

fres!(
    CORRECTION, CORRECTION_PAIR, CORRECTION_MAKE, CORRECTION_SAID, CORRECTION_MOVED,
    CORRECTION_NUMBER_PAIR, CHANGED_TO, CHANGED_BARE, CHANGED_NOW, CORRECTION_A_DATE,
    A_DAY_TAKEN_BACK, A_DAY_I_MEANT, A_TIME_INSTEAD, MOVED_BY_ITS_OWN_VERB, TRIP_DATE_MOVED,
    CORRECTION_NOT, RETRACT, CORRECTION_NOT_TO, CORRECTION_NOT_PAIR, DENIAL_OF_A_VALUE,
    NAME_TAKEN_BACK, A_MARKER_ON_THE_END, CORRECTION_BARE, CORRECTION_HEDGED, CORRECTION_CLOCK,
    CLOCK_PREFERRED, AMOUNT_CORRECTION, AMOUNT_CORRECTION_BARE, CORRECTION_MARK, COPULA_SPLIT,
    CORRECTION_DATE, CORRECTION_DATE_GOING, CORRECTION_GOING_MOVED, NO_THEN_A_THING_NAMED,
    JOB_AFTER, JOB_BECOME, JOB_WAS, JOB_WORK, JOB_APPOSITIVE, POSSESSION, JOB_GOT_AT,
    JOB_GOT_POSSESSIVE, JOB_AS_A, JOB_OWN_VOICE, JOB_MINE,
);
fsets!(
    NOT_A_DAY, A_NAME_IS_TAKEN_BACK, AMOUNT_WISH, CORRECTION_BARE_NO, CLAUSE_OPENERS, NOT_A_VALUE,
    POINTING_AT, ARTICLES, CORRECTION_TAIL_WORDS, WAS_NOT_A_JOB, JOB_FRAME,
    CORRECTION_OPENER_WORDS, JOB_GOT_FRAME, JOB_GOT_NOT_A_PLACE, JOB_GOT_AN_EMPLOYER,
);
fstrs!(CORRECT_FROM_MARK);
fmaps!(CORRECTION_SHAPE);

/// Python: bot/memory/facts.py:2977 `_a_day_taken_back(text)`: the day the line puts right, as a
/// correction: "sorry saturday i got the day wrong".
pub fn _a_day_taken_back(text: &str) -> Option<Match<'_>> {
    A_DAY_TAKEN_BACK.search(text).or_else(|| A_DAY_I_MEANT.search(text))
}

/// Python: bot/memory/facts.py:2986 `_a_moved_time(text)`: the moved-time correction in the line,
/// when what it moves to is a time.
pub fn _a_moved_time(text: &str) -> Option<Match<'_>> {
    CORRECTION_MOVED.finditer(text).into_iter().find(|moved| a_moved_time(moved.str(1)))
}

/// Python: bot/memory/facts.py:3206 `a_name_is_taken_back(text)`: true when the line denies a
/// name rather than giving one: "no hes not called max".
pub fn a_name_is_taken_back(text: &str) -> bool {
    let Some(denial) = DENIAL_OF_A_VALUE.match_(pt::strip(text)) else {
        return false;
    };
    denial.name("value").is_some_and(|v| !v.is_empty())
        && A_NAME_IS_TAKEN_BACK.contains(&lower(pt::strip(denial.name_str("says"))))
        && !denial.name("wrong").is_some_and(|w| !w.is_empty())
}

/// Python: bot/memory/facts.py:3247 `_the_same_name(left, right)`: whether a word names the same
/// person as a name on record: "kay" is "Kay Smith".
pub fn _the_same_name(left: &str, right: &str) -> bool {
    let said = squash(&lower(left).replace('.', " "));
    let known = squash(&lower(right).replace('.', " "));
    if said.is_empty() || known.is_empty() {
        return false;
    }
    said == known || pt::split(&known).contains(&said) || pt::split(&said).contains(&known)
}

/// Python: bot/memory/facts.py:3256 `the_name_they_take_back(text, stored="")`: the name on record
/// a line takes back, "" when it denies the name without saying which, or None when the line
/// does not take the name back at all.
pub fn the_name_they_take_back(text: &str, stored: &str) -> Option<String> {
    let known = pt::strip(stored).to_string();
    if known.is_empty() {
        return None;
    }
    let said = squash(text);
    let without = crate::memory::filler::without_filler(&said);
    let denial = NAME_TAKEN_BACK.match_(&without)?;
    let mut named = String::new();
    for group in ["said", "said2", "said3", "said4"] {
        if named.is_empty() {
            named = pt::strip(denial.name_str(group)).to_string();
        }
    }
    if !named.is_empty() && !_the_same_name(&named, &known) {
        return None;
    }
    Some(known)
}

/// Python: bot/memory/facts.py:3340 `_the_marker_in_front(text, question=False)`: the line with a
/// correction word moved from its end to its front, or None.
pub fn _the_marker_in_front(text: &str, question: bool) -> Option<String> {
    let said = pt::strip(text);
    let end = A_MARKER_ON_THE_END.search(said)?;
    let head = pt::strip(&said[..end.start()]).to_string();
    if head.is_empty() || pt::split(&head).len() > 8 {
        return None;
    }
    if _extract(&head, question).truthy() {
        return None;
    }
    Some(format!("actually {head}"))
}

/// Python: bot/memory/facts.py:3587 `_a_tail_hangs_off(pattern, then="")`: `pattern` with a run of
/// the words that name nothing allowed after its value.
pub fn _a_tail_hangs_off(pattern: &str, then: &str) -> String {
    let trimmed = pattern.trim_end_matches(pt::is_space);
    if !trimmed.ends_with('$') {
        return pattern.to_string();
    }
    format!("{}{}{}", &trimmed[..trimmed.len() - 1], then, data::string("bot.memory.facts.CORRECTION_TAIL"))
}

inline_re!(WORD_PARTS, r"[a-z0-9']+", 0);

/// Python: bot/memory/facts.py:3615 `a_value(phrase)`: true when a phrase reads as a value (a
/// name, a place, a number), not as a clause.
pub fn a_value(phrase: &str) -> bool {
    let parts = WORD_PARTS.findall_str(&lower(phrase));
    if parts.is_empty() {
        return false;
    }
    if parts.len() > 3 && DATE_VALUE.fullmatch(pt::strip(phrase)).is_none() {
        return false;
    }
    if CLAUSE_OPENERS.contains(&parts[0]) || NOT_A_VALUE.contains(&parts[0]) {
        return false;
    }
    if parts[1..].iter().any(|w| POINTING_AT.contains(w)) {
        return false;
    }
    parts
        .iter()
        .filter(|w| !ARTICLES.contains(*w))
        .any(|w| !NOT_A_VALUE.contains(w) && !CLAUSE_OPENERS.contains(w))
}

inline_re!(LEADING_ARTICLE, r"^(?:a|an|the)\s+", I);

/// Python: bot/memory/facts.py:3635 `reread(value, key)`: the facts a corrected value states when
/// it is read in the shape that key arrives in.
pub fn reread(value: &Value, key: &str) -> Value {
    let shape = CORRECTION_SHAPE.get(key);
    let value = LEADING_ARTICLE.sub("", pt::strip(&str_or_empty(Some(value))));
    let Some(shape) = shape else {
        return Value::dict();
    };
    if shape.is_empty() || value.is_empty() {
        return Value::dict();
    }
    let mut found = extract(&shape.replacen("{}", &value, 1), false);
    found.del_key("correction");
    found
}

/// Python: bot/memory/facts.py:3735 `a_place_that_employs(phrase)`: whether the words after
/// "at"/"with" name somewhere a person works.
pub fn a_place_that_employs(phrase: &str) -> bool {
    let words = pt::split(pt::strip_chars(phrase, " .!?,"));
    if words.is_empty() {
        return false;
    }
    let said = words.join(" ");
    if JOB_GOT_POSSESSIVE.match_(&said).is_some() {
        return false;
    }
    let first = lower(&words[0]);
    if JOB_GOT_NOT_A_PLACE.contains(&first) && !JOB_GOT_AN_EMPLOYER.contains(&first) {
        return false;
    }
    true
}

// ---------------------------------------------------------------------------------------------
// jobs, judgements, take-backs, the tone and the moment at the end of a value

fres!(
    JOB_WANTED, SELF_AS_A_ROLE, SELF_JUDGEMENT_SAID, SELF_JUDGEMENT_ABOUT_A_NOUN, TAKE_BACK_SPLIT,
    CHANGED_WORK, WORK_NOW, INFINITIVE, DAY_OF_A_MONTH,
);
fsets!(
    JUDGEMENT_NOUNS, JOB_STOP, SELF_JUDGEMENT, ROLE_YOU_CAN_FAIL, SELF_JUDGEMENT_ROLE_ONLY,
    JUDGEMENT_TAIL, QUALIFYING, JOB_DANGLING, WORK_DETAIL, WORK_DETAIL_FILLER, NOT_AN_END,
    MAY_AS_A_MONTH,
);
fstrs!(TAKE_BACK, JOB_TAIL, JOB_TAIL_PLACE, THE_MOMENT, TONE_TAIL);

/// Python: bot/memory/facts.py:3866 `is_a_judgement(text)`: whether the line is the speaker
/// judging themselves, not telling a fact about the world.
pub fn is_a_judgement(text: &str) -> bool {
    let low = squash(&lower(text).replace('\u{2019}', "'").replace('\'', ""));
    if let Some(about) = SELF_JUDGEMENT_ABOUT_A_NOUN.search(&low) {
        let noun = about.name_str("next");
        if JUDGEMENT_TAIL.contains(noun) {
            let judged = about.name_str("judged");
            return SELF_JUDGEMENT.contains(judged) || ROLE_YOU_CAN_FAIL.contains(judged);
        }
        return ROLE_YOU_CAN_FAIL.contains(noun) || SELF_JUDGEMENT.contains(noun) || JUDGEMENT_NOUNS.contains(noun);
    }
    SELF_JUDGEMENT_SAID.is_match(&low)
}

/// `after[0].lower().strip(".,;:!?") in QUALIFYING` for the words after a take-back word.
fn qualifies(after: &str) -> Option<bool> {
    let words = pt::split_ref(after);
    let first = words.first()?;
    Some(QUALIFYING.contains(pt::strip_chars(&lower(first), ".,;:!?")))
}

/// Python: bot/memory/facts.py:3902 `take_back_cut(phrase)`: the phrase up to the words that take
/// it back, with a qualifier's own words left in (a str).
pub fn take_back_cut(phrase: &str) -> Value {
    Value::from(take_back_cut_str(phrase))
}

/// `take_back_cut(phrase)` as a string.
pub fn take_back_cut_str(phrase: &str) -> String {
    for m in TAKE_BACK_SPLIT.finditer(phrase) {
        if qualifies(&phrase[m.end()..]) == Some(true) {
            continue;
        }
        return phrase[..m.start()].to_string();
    }
    phrase.to_string()
}

/// Python: bot/memory/facts.py:3918 `qualified_by(phrase)`: the qualifier said after a value
/// ("not far from the station"), or "" (a str).
pub fn qualified_by(phrase: &str) -> Value {
    for m in TAKE_BACK_SPLIT.finditer(phrase) {
        if qualifies(&phrase[m.end()..]) != Some(true) {
            continue;
        }
        let said = pt::strip_chars(&phrase[m.start()..], " ,.;:-'");
        if !said.is_empty() {
            return Value::from(said);
        }
    }
    Value::from("")
}

/// Python: bot/memory/facts.py:3933 `_the_role(words)`: the role in the first words of a phrase,
/// with a joining word left off its end.
pub fn _the_role(words: &[String]) -> String {
    let mut words: Vec<&String> = words.iter().take(3).collect();
    while let Some(last) = words.last() {
        if JOB_DANGLING.contains(pt::strip_chars(&lower(last), ".,!?")) {
            words.pop();
        } else {
            break;
        }
    }
    words.iter().map(|w| lower(w)).collect::<Vec<_>>().join(" ")
}

/// Python: bot/memory/facts.py:3983 `is_a_place_of_work(frame)`: whether the frame says where
/// they work rather than what they do.
pub fn is_a_place_of_work(frame: &Value) -> bool {
    let f = str_or_empty(Some(frame));
    let preposition = lower(pt::strip(&f).split(' ').next().unwrap_or(""));
    matches!(preposition.as_str(), "in" | "at" | "on")
}

inline_re!(LETTERS_AND_APOSTROPHES, r"[a-z']+", 0);

/// Python: bot/memory/facts.py:3995 `is_a_work_detail(value, frame=None)`: whether what a line
/// says about their work is only when or where it is, not what it is.
pub fn is_a_work_detail(value: &Value, frame: &Value) -> bool {
    if is_a_place_of_work(frame) {
        return true;
    }
    let words = LETTERS_AND_APOSTROPHES.findall_str(&lower(&str_or_empty(Some(value))));
    let meaningful: Vec<&String> = words.iter().filter(|w| !WORK_DETAIL_FILLER.contains(*w)).collect();
    if meaningful.is_empty() {
        return !words.is_empty();
    }
    meaningful.iter().all(|w| WORK_DETAIL.contains(*w) || *w == "job" || *w == "jobs")
}

/// Python: bot/memory/facts.py:4013 `work_weight(value, frame=None)`: how much a line about their
/// work says: 3 for the role, 2 for a place, 1 for a shift.
pub fn work_weight(value: &Value, frame: &Value) -> i64 {
    if !is_a_work_detail(value, frame) {
        return 3;
    }
    if is_a_place_of_work(frame) {
        2
    } else {
        1
    }
}

/// Python: bot/memory/facts.py:4025 `_finish(word)`: stripped of space and punctuation, lower case.
pub fn _finish(word: &str) -> String {
    lower(pt::strip_chars(pt::strip(word), ".,!?;:'\""))
}

/// Python: bot/memory/facts.py:4039 `without_the_moment(words)`: the same words with the moment
/// they were said in taken off the end.
pub fn without_the_moment(words: &[String]) -> Vec<String> {
    let mut words: Vec<String> = words.iter().filter(|w| !pt::strip(w).is_empty()).cloned().collect();
    while words.len() > 1 {
        let mut cut = 0;
        for span in [3usize, 2, 1] {
            if words.len() >= span && THE_MOMENT.contains(&_finish(&words[words.len() - span..].join(" "))) {
                cut = span;
                break;
            }
        }
        if cut == 0 {
            break;
        }
        words.truncate(words.len() - cut);
    }
    if words.len() == 1 && THE_MOMENT.contains(&_finish(&words[0])) {
        return Vec::new();
    }
    words
}

/// Python: bot/memory/facts.py:4092 `_can_end(words)`: whether the last word of a sentence can be
/// the end of it ("may" is the month or the modal).
pub fn _can_end(words: &[String]) -> bool {
    let Some(last) = words.last() else {
        return false;
    };
    let last = pt::strip_chars(&lower(last), ".,!?;:'\"").to_string();
    if !NOT_AN_END.contains(&last) {
        return true;
    }
    if last == "may" && words.len() > 1 {
        let before = pt::strip_chars(&lower(&words[words.len() - 2]), ".,!?;:'\"").to_string();
        return MAY_AS_A_MONTH.contains(&before) || DAY_OF_A_MONTH.match_(&before).is_some();
    }
    false
}

/// Python: bot/memory/facts.py:4105 `no_tone(text)`: the same words with the tone words people
/// leave at the end taken off.
pub fn no_tone(text: &str) -> String {
    let mut out = pt::strip(text).to_string();
    for _ in 0..3 {
        let low_full = lower(&out);
        let low = pt::rstrip_chars(&low_full, " .,!?;:");
        let cut = TONE_TAIL.iter().find(|phrase| low.ends_with(&format!(" {phrase}")) || low == phrase.as_str());
        let Some(cut) = cut else {
            break;
        };
        let keep = pt::len(low) as isize - pt::len(cut) as isize;
        let left = pt::rstrip_chars(&pt::slice(&out, 0, Some(keep)), " ,;:-.)").to_string();
        let words = pt::split(&left);
        if cut.starts_with("to ") && INFINITIVE.is_match(&words.join(" ")) {
            break;
        }
        if !words.is_empty() && !_can_end(&words) {
            break;
        }
        out = left;
    }
    if pt::strip_chars(&out, " ,;:-.").is_empty() {
        String::new()
    } else {
        out
    }
}

// ---------------------------------------------------------------------------------------------
// a date put right, a plan's place moved, and the name reader

fres!(ORDINAL_DAY, MOVED_PLACE_TO, MOVED_PLACE_PASSIVE, VERB_ISH, SAYS_A_NAME);
fsets!(
    A_DATE_WORD, NOT_A_PLACE_NAME, A_LEAD_IDIOM, TAIL_DETERMINERS, SAYS_SOMETHING_VERBS,
    SAYS_SOMETHING_ELSE, _SQUEEZED_INTROS, LOOKS_WORDS,
);
fstrs!(A_LEAD_PREPOSITION, INTRO_WORDS, NAME_FRAMES);

/// `NAME_TAIL_MIN`: an ending, and the length a word needs before the ending counts.
pub static NAME_TAIL_MIN: LazyLock<Vec<(String, usize)>> = LazyLock::new(|| {
    data::get("bot.memory.facts.NAME_TAIL_MIN")
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_int()? as usize)))
        .collect()
});
/// `STATE_DEGREES`: (ending, the letter it took off).
pub static STATE_DEGREES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.memory.facts.STATE_DEGREES")
        .items()
        .iter()
        .map(|t| (t.items()[0].str().to_string(), t.items()[1].str().to_string()))
        .collect()
});
/// `COMPARED`: (ending, shortest word).
pub static COMPARED: LazyLock<Vec<(String, usize)>> = LazyLock::new(|| {
    data::get("bot.memory.facts.COMPARED")
        .items()
        .iter()
        .map(|t| (t.items()[0].str().to_string(), t.items()[1].as_int().unwrap_or(0) as usize))
        .collect()
});

static LEX_FEELINGS: LazyLock<HashSet<String>> = LazyLock::new(|| bundle_set("bot.understanding.lexicon.FEELINGS"));
static LEX_A_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| bundle_set("bot.understanding.lexicon.A_STATE"));
static LEX_JOINED_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| bundle_set("bot.understanding.lexicon.JOINED_WORDS"));

/// Python: bot/memory/facts.py:4127 `_date_change(text)`: the date a plan is moved to, or None:
/// "we moved it to july", "it's july now".
pub fn _date_change(text: &str) -> Option<String> {
    if text.contains('?') {
        return None;
    }
    for pattern in [&*CHANGED_TO, &*CHANGED_BARE, &*CHANGED_NOW, &*CORRECTION_A_DATE, &*A_TIME_INSTEAD, &*MOVED_BY_ITS_OWN_VERB] {
        let Some(found) = pattern.search(text) else {
            continue;
        };
        for group in found.groups().into_iter().flatten() {
            if !group.is_empty() && a_date(group) {
                return Some(tidy(group));
            }
        }
    }
    None
}

/// Python: bot/memory/facts.py:4175 `a_moved_place(text)`: the place a line moves a plan to:
/// "no we changed it to devon" -> "Devon", else "".
pub fn a_moved_place(text: &str) -> String {
    if text.is_empty() || text.contains('?') {
        return String::new();
    }
    for pattern in [&*MOVED_PLACE_TO, &*MOVED_PLACE_PASSIVE] {
        for found in pattern.finditer(text) {
            let (places, _home, _said) = _places(&format!("we are going to {}", found.str(1)));
            for place in places {
                if a_date(&place) || a_moved_time(&place) {
                    continue;
                }
                if NOT_A_PLACE_NAME.contains(&lower(&place)) {
                    continue;
                }
                return place;
            }
        }
    }
    String::new()
}

/// Python: bot/memory/facts.py:4199 `a_date_put_right(text)`: true when the line is only a date
/// put right, with no subject of its own.
pub fn a_date_put_right(text: &str) -> bool {
    if text.contains('?') {
        return false;
    }
    CORRECTION_A_DATE.is_match(text) || A_DAY_TAKEN_BACK.is_match(text) || A_DAY_I_MEANT.is_match(text) || A_TIME_INSTEAD.is_match(text)
}

/// Python: bot/memory/facts.py:4213 `the_date_put_right(text)`: the date the line puts right:
/// "sorry, it's the 4th" -> "the 4th", else "".
pub fn the_date_put_right(text: &str) -> String {
    if text.contains('?') {
        return String::new();
    }
    for pattern in [&*CORRECTION_A_DATE, &*A_DAY_TAKEN_BACK, &*A_DAY_I_MEANT, &*A_TIME_INSTEAD] {
        if let Some(found) = pattern.search(text) {
            for group in found.groups().into_iter().flatten() {
                if !group.is_empty() && a_date(group) {
                    return pt::strip_chars(group, " .,").to_string();
                }
            }
        }
    }
    String::new()
}

/// Python: bot/memory/facts.py:4227 `a_trip_date_put_right(text)`: the date a trip is moved to by
/// a line that says how they travel, or "".
pub fn a_trip_date_put_right(text: &str) -> String {
    if text.contains('?') {
        return String::new();
    }
    let Some(found) = TRIP_DATE_MOVED.search(text) else {
        return String::new();
    };
    for group in found.groups().into_iter().flatten() {
        if !group.is_empty() && a_date(group) {
            return tidy(group);
        }
    }
    String::new()
}

inline_re!(SPACE_RUN, r"\s+", 0);

/// Python: bot/memory/facts.py:4247 `_a_date(word)` (the contract's Value: a bool).
pub fn _a_date(word: &str) -> Value {
    Value::Bool(a_date(word))
}

/// `_a_date(word)`: a month or a day: the only values a plan carries, so the only ones a change
/// can name.
pub fn a_date(word: &str) -> bool {
    let phrase = pt::strip(word);
    if phrase.is_empty() {
        return false;
    }
    let mut seen = false;
    let mut a_month = false;
    let mut a_plain_number = false;
    for piece in SPACE_RUN.split(&lower(phrase)) {
        if piece == "of" || piece == "the" {
            continue;
        }
        if ORDINAL_DAY.fullmatch(&piece).is_some() || A_DATE_WORD.contains(&piece) {
            seen = true;
            a_month = a_month || MONTHS.contains(&piece);
            continue;
        }
        if pt::is_digit(&piece) && pt::len(&piece) <= 2 && !a_plain_number {
            a_plain_number = true;
            continue;
        }
        return false;
    }
    seen || (a_plain_number && a_month)
}

inline_re!(A_LETTER, r"[a-z]", I);

/// Python: bot/memory/facts.py:4306 `_the_value_alone(phrase)`: `phrase` without the word of
/// apology, the shrug, or the preposition around a value.
pub fn _the_value_alone(phrase: &str) -> String {
    let mut words = pt::split(phrase);
    while words.len() > 1 && CORRECTION_TAIL_WORDS.contains(&lower(pt::strip_chars(words.last().unwrap(), ".,!?;:'\""))) {
        words.pop();
    }
    if words.len() > 2 {
        let last2: Vec<String> = words[words.len() - 2..].iter().map(|w| lower(w)).collect();
        if last2 == ["i", "mean"] || last2 == ["i", "meant"] {
            words.truncate(words.len() - 2);
        }
    }
    if words.len() > 1
        && A_LEAD_PREPOSITION.contains(&lower(&words[0]))
        && A_LEAD_IDIOM.contains(&lower(pt::strip_chars(&words[1], ".,!?;:'\"")))
    {
        return String::new();
    }
    if words.len() > 1
        && A_LEAD_PREPOSITION.contains(&lower(&words[0]))
        && A_LETTER.is_match(&words[1])
        && !(lower(&words[0]) == "for" && lower(&words[1]).ends_with("ing"))
    {
        words.remove(0);
    }
    words.join(" ")
}

/// Python: bot/memory/facts.py:4333 `_tidy_apart(word)`: the old value said beside the new one,
/// an article in front of it kept lower case.
pub fn _tidy_apart(word: &str) -> String {
    let word = pt::strip_chars(pt::strip(word), ".,!?;:'\"");
    let (first, _, rest) = pt::partition(word, " ");
    if ARTICLES.contains(&lower(first)) {
        let joined = if rest.is_empty() { lower(first) } else { format!("{} {}", lower(first), rest) };
        return pt::strip(&joined).to_string();
    }
    up1(word)
}

/// Python: bot/memory/facts.py:4352 `_lowercase_name_fits(text, match)`: a name written without a
/// capital only passes if its clause reads as an introduction. `start1` is `match.start(1)`.
pub fn _lowercase_name_fits(text: &str, start1: usize) -> bool {
    let mut head = &text[..start1];
    for mark in ['.', '!', '?', ';'] {
        if let Some(at) = head.rfind(mark) {
            head = &head[at + 1..];
        }
    }
    let mut head = lower(head).replace('\u{2019}', "'");
    for phrase in NAME_FILLER_PHRASES.iter() {
        head = head.replace(phrase.as_str(), " ");
    }
    let words: Vec<String> = pt::split(&head).iter().map(|w| pt::strip_chars(w, ",.!?;:'\"()").to_string()).collect();
    let left: Vec<&String> = words.iter().filter(|w| !w.is_empty() && !_filler_or_intro(w)).collect();
    let joined = left.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" ");
    NAME_FRAMES.contains(&joined)
}

/// Python: bot/memory/facts.py:4373 `_filler_or_intro(word)`: whether a word in front of a name is
/// filler or the greeting that opens a line.
pub fn _filler_or_intro(word: &str) -> bool {
    if NAME_FILLER.contains(word) || INTRO_WORDS.iter().any(|w| w == word) {
        return true;
    }
    let settled = _squeezed(word);
    if settled == word {
        return false;
    }
    NAME_FILLER.contains(&settled) || INTRO_WORDS.contains(&settled) || _SQUEEZED_INTROS.contains(&_squeezed(word))
}

/// Python: bot/memory/facts.py:4391 `_squeezed(word)`: every run of one letter cut to one
/// (`re.sub(r"(.)\1+", r"\1", word)`; `.` does not match a newline).
pub fn _squeezed(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    let mut prev: Option<char> = None;
    for c in word.chars() {
        if prev == Some(c) && c != '\n' {
            continue;
        }
        out.push(c);
        prev = Some(c);
    }
    out
}

/// Python: bot/memory/facts.py:4410 `_tail_has_a_verb(words)`: whether a clause says something,
/// rather than being the object of the word before it.
pub fn _tail_has_a_verb(words: &[String]) -> bool {
    for word in words {
        let low = lower(word);
        if SAYS_SOMETHING_VERBS.contains(&low) || low.ends_with("'s") || low.ends_with("\u{2019}s") {
            return true;
        }
        if SAYS_SOMETHING_ELSE.contains(&low) {
            return true;
        }
        if pt::len(&low) > 4 && low.ends_with("ed") {
            return true;
        }
    }
    false
}

inline_re!(SENTENCE_OR_COMMA, r"[.!?;,]", 0);
inline_re!(WORDS_AND_FIGURES, r"[A-Za-z0-9']+", 0);

/// Python: bot/memory/facts.py:4452 `_followers_fit(text, end, name_word=None)`: whether the line
/// goes on with the rest of an introduction after a name typed without a capital. (`end` is a
/// byte offset.)
pub fn _followers_fit(text: &str, end: usize, name_word: &str) -> bool {
    let tail = SENTENCE_OR_COMMA.split(&text[end..]).into_iter().next().unwrap_or_default();
    let tail = _without_asides(&tail);
    let found = WORDS_AND_FIGURES.findall_str(&tail);
    let mut at = 0;
    while at < found.len() {
        let word = &found[at];
        let low = lower(word);
        at += 1;
        if pt::is_digit(word) || word.chars().next().is_some_and(char::is_uppercase) {
            continue;
        }
        if NAME_SAYS_ABOUT_ME.contains(&low) {
            if !name_word.is_empty()
                && VERB_ISH.match_(name_word).is_some()
                && TAIL_DETERMINERS.contains(&low)
                && !_tail_has_a_verb(&found[at..])
            {
                return false;
            }
            return true;
        }
        if NAME_FOLLOWERS.contains(&low) {
            continue;
        }
        if NAME_PLACE_WORDS.contains(&low) {
            if at < found.len() && PLACE_JOIN.contains(&lower(&found[at])) {
                at += 1;
            }
            if at >= found.len() {
                return false;
            }
            at += 1;
            continue;
        }
        return false;
    }
    true
}

/// Python: bot/memory/facts.py:4495 `_without_asides(tail)`: the tail of an introduction with a
/// trailing hedge taken off.
pub fn _without_asides(tail: &str) -> String {
    let mut tail = tail.to_string();
    let mut low = lower(&tail).replace('\u{2019}', "'").replace('\'', "");
    let mut changed = true;
    while changed && !pt::strip(&low).is_empty() {
        changed = false;
        for aside in NAME_ASIDES.iter() {
            let aside = aside.replace('\'', "");
            let stripped = pt::rstrip_chars(&low, " ,.!?").to_string();
            if stripped.ends_with(&aside) {
                let cut = pt::rstrip_chars(&stripped[..stripped.len() - aside.len()], " ,").to_string();
                low = cut.clone();
                tail = cut;
                changed = true;
                break;
            }
        }
    }
    tail
}

/// Python: bot/memory/facts.py:4521 `_a_state_word(low)`: whether the word on its own is one of the
/// states a person describes themselves with.
pub fn _a_state_word(low: &str) -> bool {
    NOT_NAMES.contains(low)
        || NOT_NAMES_EXTRA.contains(low)
        || NOT_A_NAME_FEELING.contains(low)
        || NOT_A_NAME_ILLNESS.contains(low)
        || DIET_WORDS.contains(low)
        || SOBER_WORDS.contains(low)
}

/// The last `n` characters off a string (`s[:-n]`).
fn drop_last_chars(s: &str, n: usize) -> String {
    let k = pt::len(s);
    pt::slice(s, 0, Some(k.saturating_sub(n) as isize))
}

/// The `i`th character from the end (`s[-i]`, 1-based).
fn char_from_end(s: &str, i: usize) -> Option<char> {
    s.chars().rev().nth(i - 1)
}

/// Python: bot/memory/facts.py:4536 `_a_degree_of_a_state(low)`.
pub fn _a_degree_of_a_state(low: &str) -> bool {
    for (ending, restore) in STATE_DEGREES.iter() {
        if !low.ends_with(ending.as_str()) || pt::len(low) <= pt::len(ending) + 2 {
            continue;
        }
        let stem = format!("{}{}", drop_last_chars(low, pt::len(ending)), restore);
        if _a_state_word(&stem) {
            return true;
        }
        if pt::len(&stem) >= 3 && char_from_end(&stem, 1) == char_from_end(&stem, 2) && _a_state_word(&drop_last_chars(&stem, 1)) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/facts.py:4549 `_a_letter_too_many(low)`.
pub fn _a_letter_too_many(low: &str) -> bool {
    if pt::len(low) < 5 || char_from_end(low, 1) != char_from_end(low, 2) {
        return false;
    }
    _a_state_word(&drop_last_chars(low, 1))
}

/// Python: bot/memory/facts.py:4555 `_sounds_like_a_state(low)`: whether a lower-case word after
/// "im" describes the person rather than naming them.
pub fn _sounds_like_a_state(low: &str) -> bool {
    if NAMES_THAT_LOOK_LIKE_STATES.contains(low) {
        return false;
    }
    if _a_degree_of_a_state(low) || _a_letter_too_many(low) {
        return true;
    }
    for (tail, shortest) in NAME_TAIL_MIN.iter() {
        if pt::len(low) >= *shortest && low.ends_with(tail.as_str()) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/facts.py:4571 `_a_word_that_is_a_state(word)`.
pub fn _a_word_that_is_a_state(word: &str) -> bool {
    pt::len(word) >= 3
        && (NOT_A_NAME_STATE.contains(word) || LEX_FEELINGS.contains(word) || LEX_A_STATE.contains(word) || NOT_NAMES_EXTRA.contains(word))
}

/// Python: bot/memory/facts.py:4587 `_a_word_not_a_name(low)`: whether a word after "is" says how
/// somebody is rather than what they are called.
pub fn _a_word_not_a_name(low: &str) -> bool {
    LOOKS_WORDS.contains(low) || NUMBER_WORDS.contains(low) || _sounds_like_a_state(low) || _a_word_that_is_a_state(low)
}

/// Python: bot/memory/facts.py:4608 `_the_state_compared(word)`: the state a comparison is written
/// in, or None: "happier" is happy, "sadder" is sad.
pub fn _the_state_compared(word: &str) -> Option<String> {
    for (ending, shortest) in COMPARED.iter() {
        if pt::len(word) < *shortest || !word.ends_with(ending.as_str()) {
            continue;
        }
        let stem = drop_last_chars(word, pt::len(ending));
        if stem.is_empty() {
            continue;
        }
        let mut states: Vec<Option<String>> =
            vec![Some(stem.clone()), _with_its_y_back(&stem), Some(format!("{stem}y")), Some(format!("{stem}e"))];
        states.extend(_a_letter_off_each_run(&stem).into_iter().map(Some));
        for state in states.into_iter().flatten() {
            if !state.is_empty() && _a_word_that_is_a_state(&state) {
                return Some(state);
            }
        }
    }
    None
}

/// Python: bot/memory/facts.py:4631 `_with_its_y_back(stem)`: the y a comparison turned into an i.
pub fn _with_its_y_back(stem: &str) -> Option<String> {
    if stem.ends_with('i') {
        return Some(format!("{}y", &stem[..stem.len() - 1]));
    }
    None
}

/// Python: bot/memory/facts.py:4643 `_a_letter_off_each_run(word)`: the word with one letter taken
/// out of each run of the same letter: "stuckk" gives "stuck".
pub fn _a_letter_off_each_run(word: &str) -> Vec<String> {
    let chars: Vec<char> = word.chars().collect();
    let mut out = Vec::new();
    let mut at = 0;
    while at < chars.len() {
        let mut end = at;
        while end + 1 < chars.len() && chars[end + 1] == chars[at] {
            end += 1;
        }
        if end > at {
            let mut s: String = chars[..end].iter().collect();
            s.extend(chars[end + 1..].iter());
            out.push(s);
        }
        at = end + 1;
    }
    out
}

/// Python: bot/memory/facts.py:4665 `_a_state_written_as_a_comparison_or_a_double(low)`.
pub fn _a_state_written_as_a_comparison_or_a_double(low: &str) -> bool {
    use crate::understanding::normalize;
    if normalize::reads_as_a_feeling(low) || _the_state_compared(low).is_some() {
        return true;
    }
    if normalize::the_feeling_two_letters_are_gone_from(low).is_some_and(|f| !f.is_empty()) {
        return true;
    }
    for cut in _a_letter_off_each_run(low) {
        if _a_word_that_is_a_state(&cut) || _the_state_compared(&cut).is_some() {
            return true;
        }
    }
    false
}

/// Python: bot/memory/facts.py:4690 `_name(text)`: the name the person gives for themselves, or
/// None.
pub fn _name(text: &str) -> Option<String> {
    let mut went_by = _name_from(&GOES_BY_NAME, text, true);
    if went_by.is_none() && GOES_BY_NAME.is_match(text) {
        for clause in CLAUSE_SPLIT.split(text) {
            if !GOES_BY_NAME.is_match(&clause) {
                continue;
            }
            went_by = _name_from(&GOES_BY_NAME, &clause, true);
            if went_by.is_some() {
                break;
            }
        }
    }
    for pattern in ANNOUNCED_NAME_PATTERNS.iter() {
        if let Some(name) = _name_from(pattern, text, false) {
            if let Some(went) = &went_by {
                if lower(went) != lower(&name) {
                    return went_by;
                }
            }
            return Some(name);
        }
    }
    for pattern in GUESSED_NAME_PATTERNS.iter() {
        if let Some(name) = _name_from(pattern, text, true) {
            return Some(name);
        }
    }
    None
}

/// Python: bot/memory/facts.py:4735 `_a_judgement_on_themselves(low)`.
pub fn _a_judgement_on_themselves(low: &str) -> bool {
    if SELF_JUDGEMENT.contains(low) {
        return true;
    }
    for ending in ["ish", "y"] {
        if !low.ends_with(ending) || pt::len(low) <= ending.len() + 2 {
            continue;
        }
        let stem = &low[..low.len() - ending.len()];
        if SELF_JUDGEMENT.contains(stem) {
            return true;
        }
        if char_from_end(stem, 1) == char_from_end(stem, 2) && SELF_JUDGEMENT.contains(&drop_last_chars(stem, 1)) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/facts.py:4749 `_name_from(pattern, text, guessed)`: the first word this
/// pattern offers that could be a name, or None.
pub fn _name_from(pattern: &Re, text: &str, guessed: bool) -> Option<String> {
    use crate::understanding::{lexicon, normalize};
    let says_a_name = || SAYS_A_NAME.is_match(text);
    for m in pattern.finditer(text) {
        let word = m.str(1);
        let low = _finish(word);
        if NOT_NAMES.contains(&low)
            || NOT_NAMES_EXTRA.contains(&low)
            || GO_BY_WAYS.contains(&low)
            || DIET_WORDS.contains(&low)
            || SOBER_WORDS.contains(&low)
            || LEX_JOINED_WORDS.contains(&low)
            || pt::len(&low) < 2
        {
            continue;
        }
        let capital = word.chars().next().is_some_and(char::is_uppercase);
        if !capital
            && (NOT_A_NAME_STATE.contains(&low)
                || _a_judgement_on_themselves(&low)
                || NOT_A_NAME_LAUGH.contains(&low)
                || lexicon::is_a_laugh(&low))
        {
            continue;
        }
        let spelled = normalize::spelled(&low);
        let looks_like_a_state = NAMES_THAT_LOOK_LIKE_STATES.contains(&low);
        if !looks_like_a_state
            && spelled
                .iter()
                .any(|one| *one != low && (NOT_NAMES.contains(one) || NOT_NAMES_EXTRA.contains(one) || NOT_A_NAME_STATE.contains(one)))
        {
            continue;
        }
        if NUMBER_WORDS.contains(&low) {
            continue;
        }
        let repaired = normalize::repaired_tokens(&low);
        if !looks_like_a_state
            && repaired.iter().any(|one| {
                *one != low
                    && pt::len(one) >= pt::len(&low)
                    && (NOT_NAMES.contains(one) || NOT_NAMES_EXTRA.contains(one) || NOT_A_NAME_STATE.contains(one))
            })
        {
            continue;
        }
        if guessed && !looks_like_a_state && _describes_them(&low) {
            continue;
        }
        if guessed
            && !looks_like_a_state
            && repaired.iter().any(|one| *one != low && pt::len(one) >= pt::len(&low) && _sounds_like_a_state(one))
        {
            continue;
        }
        if _a_state_written_as_a_comparison_or_a_double(&low) && !says_a_name() {
            continue;
        }
        if normalize::reads_as_a_sound(&low) && !says_a_name() {
            continue;
        }
        if !capital {
            if _sounds_like_a_state(&low) && !says_a_name() {
                continue;
            }
            let start1 = m.span(1).map(|s| s.0).unwrap_or(0);
            let end1 = m.span(1).map(|s| s.1).unwrap_or(0);
            if guessed && (!_lowercase_name_fits(text, start1) || !_followers_fit(text, end1, word)) {
                continue;
            }
        }
        return Some(up1_first_char(word));
    }
    None
}

/// `word[0].upper() + word[1:]`.
fn up1_first_char(word: &str) -> String {
    up1(word)
}

// ---------------------------------------------------------------------------------------------
// the job, the car, and places

fsets!(
    JUDGEMENT_AFTER, STATE_WORDS, PREGNANCY_STATES, TAKES_ONE_BACK, NOT_A_CAR, MEASURED,
    ADJECTIVE_NAMING, NOT_A_PLACE_WORD, DETERMINERS,
);
fstrs!(ROLE_ENDING);
fres!(I_DRIVE, CAR_ASKED, CAR_NAMED, CAR_IN_THE_PAST, FROM_A_MOVE, A_MOVE_FROM_THEN_TO, A_HOME_BEFORE, FUTURE);

/// Python: bot/memory/facts.py:4838 `_first_tail(phrase, tails=JOB_TAIL)`: the earliest place the
/// job phrase stops: its position (a byte offset) and the word it stops on.
pub fn _first_tail(phrase: &str, tails: &[String]) -> Option<(usize, String)> {
    let mut found: Option<(usize, String)> = None;
    for tail in tails {
        if let Some(at) = phrase.find(tail.as_str()) {
            if found.as_ref().map_or(true, |(f, _)| at < *f) {
                found = Some((at, tail.clone()));
            }
        }
    }
    found
}

/// Python: bot/memory/facts.py:4853 `_job_as_a(text)`: the "job as a ..." lines that name the
/// speaker's own job, as their matches.
pub fn _job_as_a(text: &str) -> Vec<Match<'_>> {
    let mut found = Vec::new();
    for m in JOB_AS_A.finditer(text) {
        let clause = CLAUSE_EDGE.split(&text[..m.start()]).pop().unwrap_or_default();
        if JOB_WANTED.is_match(&clause) || !_people(&clause).is_empty() {
            continue;
        }
        if !(JOB_OWN_VOICE.is_match(&clause) || JOB_MINE.is_match(&clause)) {
            continue;
        }
        found.push(m);
    }
    found
}

/// Python: bot/memory/facts.py:4880 `_is_a_judgement_phrase(words)`: whether "im a <judgement
/// noun> ..." judges the person rather than naming what they do.
pub fn _is_a_judgement_phrase(words: &[String]) -> bool {
    if words.is_empty() || !JUDGEMENT_NOUNS.contains(&lower(&words[0])) {
        return false;
    }
    words.len() == 1 || JUDGEMENT_AFTER.contains(&lower(&words[1]))
}

/// Python: bot/memory/facts.py:4895 `_only_discourse_before(before)`: true when the words in front
/// of a comma name nobody, only the way a correction opens.
pub fn _only_discourse_before(before: &str) -> bool {
    LETTERS_AND_APOSTROPHES.findall_str(&lower(before)).iter().all(|w| CORRECTION_OPENER_WORDS.contains(w))
}

/// Python: bot/memory/facts.py:4901 `_the_correction_named_a_thing(text, match)`: true when the
/// noun after a correction opens the line and names a thing, not a person.
pub fn _the_correction_named_a_thing(text: &str, m: &Match) -> bool {
    if !_only_discourse_before(&text[..m.start()]) {
        return false;
    }
    let groups = m.groups();
    let g = if groups.len() > 2 { groups[groups.len() - 1] } else { groups[0] };
    let phrase = lower(pt::strip(g.unwrap_or("")));
    let words = pt::split(&phrase);
    !words.is_empty() && !JOB_WORDS.contains(&words[0])
}

/// Python: bot/memory/facts.py:4923 `_was_names_a_role(phrase, article=None)`: whether a phrase
/// said after "i was" names what they did, not how they were.
pub fn _was_names_a_role(phrase: &str, article: bool) -> bool {
    let mut low: Vec<String> = pt::split(phrase)
        .iter()
        .filter(|w| !pt::strip(w).is_empty())
        .map(|w| pt::strip_chars(&lower(w), ".,!?;:").to_string())
        .collect();
    while !low.is_empty() && ["both", "all", "just", "only", "still", "now", "then", "also"].contains(&low[0].as_str()) {
        low.remove(0);
    }
    low.truncate(3);
    if article && !low.is_empty() {
        return true;
    }
    if low.is_empty() {
        return false;
    }
    if low.iter().any(|w| pt::len(w) > 4 && (w.ends_with("ing") || w.ends_with("ed"))) {
        return false;
    }
    let head = low.last().unwrap().clone();
    if STATE_WORDS.contains(&head) || PAST_STATES.contains(&head) || PREGNANCY_STATES.contains(&head) {
        return false;
    }
    let single: Vec<String> = low
        .iter()
        .map(|w| if w.ends_with('s') && !JOB_WORDS.contains(w) { w[..w.len() - 1].to_string() } else { w.clone() })
        .collect();
    if low.iter().chain(single.iter()).any(|w| JOB_WORDS.contains(w)) {
        return true;
    }
    pt::len(&head) > 4 && ROLE_ENDING.iter().any(|e| head.ends_with(e.as_str()))
}

/// Which job pattern a match came from (`match.re is ...`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum JobShape {
    After,
    Become,
    Work,
    Was,
    AsA,
    GotAt,
    Appositive,
}

inline_re!(A_CAPITALISED_PLACE, r"([A-Z][a-z]+(?: [A-Z][a-z]+)?)", 0);
inline_re!(I_AM_BEFORE, r"\bi(?:'m|m| am)\b", I);
inline_re!(A_FIGURE_NEXT, r"\s*\d", 0);
inline_re!(AS_AN_ARTICLE, r"\bas\s+(an?)\b", I);

/// Python: bot/memory/facts.py:4954 `_job_and_city(text, name=None, question=False)`: the job,
/// the city said with it, and the frame the job was said in.
pub fn _job_and_city(text: &str, name: &str, question: bool) -> (Option<String>, Option<String>, String) {
    let mut job: Option<String> = None;
    let mut city: Option<String> = None;
    let mut frame = String::new();
    let mut text = text;
    if !name.is_empty() {
        let tl = lower(text);
        let nl = lower(name);
        if let Some(at) = tl.find(&nl) {
            // `text[at + len(name):]` in characters: the lowered text can differ in length from
            // the text only for characters no name holds, so the character count is taken from
            // the lowered text and applied to the original.
            let chars_at = pt::char_index(&tl, at) + pt::len(name);
            text = &text[pt::byte_at(text, chars_at)..];
        }
    }
    let mut matches: Vec<(Match, JobShape)> = Vec::new();
    for (shape, pattern) in [(JobShape::After, &*JOB_AFTER), (JobShape::Become, &*JOB_BECOME), (JobShape::Work, &*JOB_WORK), (JobShape::Was, &*JOB_WAS)] {
        let found = pattern.finditer(text);
        if !found.is_empty() {
            matches = found.into_iter().map(|m| (m, shape)).collect();
            break;
        }
    }
    if matches.is_empty() {
        matches = _job_as_a(text).into_iter().map(|m| (m, JobShape::AsA)).collect();
    }
    if matches.is_empty() {
        matches = JOB_GOT_AT.finditer(text).into_iter().map(|m| (m, JobShape::GotAt)).collect();
    }
    if matches.is_empty() && !POSSESSION.is_match(text) {
        matches = JOB_APPOSITIVE
            .finditer(text)
            .into_iter()
            .filter(|m| !_the_correction_named_a_thing(text, m))
            .map(|m| (m, JobShape::Appositive))
            .collect();
        if question {
            matches.retain(|(m, _)| I_AM_BEFORE.is_match(&text[..m.start()]));
        }
    }
    if matches.is_empty() {
        return (job, city, frame);
    }
    for (m, shape) in &matches {
        let groups = m.groups();
        let was_working = groups.len() > 2;
        let g0 = groups[0].unwrap_or("");
        let mut phrase = pt::strip(if was_working { groups[groups.len() - 1].unwrap_or("") } else { g0 }).to_string();
        let place_job = (was_working && ["in", "at", "for", "on"].contains(&lower(g0).as_str()))
            || *shape == JobShape::AsA
            || *shape == JobShape::GotAt;
        let mut cut = _first_tail(&phrase, &JOB_TAIL);
        if place_job && cut.is_none() {
            cut = _first_tail(&phrase, &JOB_TAIL_PLACE);
        }
        if let Some((at, tail)) = cut {
            let rest = phrase[at + tail.len()..].to_string();
            let t = pt::strip(&tail);
            if (t == "in" || t == "at") && city.is_none() {
                if let Some(place) = A_CAPITALISED_PLACE.match_(pt::strip(&rest)) {
                    let p = place.str(1);
                    if !NOT_PLACES.contains(&lower(p.split_whitespace().next().unwrap_or(""))) {
                        city = Some(p.to_string());
                    }
                }
            }
            phrase.truncate(at);
        }
        if *shape == JobShape::GotAt {
            if lower(g0) == "in" && pt::strip(groups.get(1).copied().flatten().unwrap_or("")).is_empty() {
                continue;
            }
            if !a_place_that_employs(&phrase) {
                continue;
            }
        }
        let words = without_the_moment(&pt::split(&phrase));
        if *shape == JobShape::Was && !words.is_empty() && WAS_NOT_A_JOB.contains(&lower(&words[0])) {
            continue;
        }
        if *shape == JobShape::Was {
            let article = groups.len() > 1 && groups[1].is_some_and(|a| !a.is_empty());
            if !_was_names_a_role(&phrase, article) {
                continue;
            }
        }
        if words.is_empty() {
            continue;
        }
        let w0 = lower(&words[0]);
        if JOB_STOP.contains(&w0) || NOT_NAMES.contains(&w0) || NOT_NAMES_EXTRA.contains(&w0) || _is_a_judgement_phrase(&words) {
            continue;
        }
        if !name.is_empty() && w0 == lower(name) {
            continue;
        }
        if words[0].chars().next().is_some_and(char::is_uppercase) && words.len() == 1 {
            continue;
        }
        let last = m.lastindex().unwrap_or(1);
        let after = m.end_of(last).unwrap_or(m.end());
        if A_FIGURE_NEXT.match_(&text[after..]).is_some() {
            continue;
        }
        if ["bit", "little", "lot", "couple", "few"].contains(&w0.as_str()) {
            continue;
        }
        if ["my", "our", "his", "her", "their", "your"].contains(&w0.as_str()) {
            continue;
        }
        if SELF_JUDGEMENT.contains(&w0) {
            continue;
        }
        let role = _the_role(&words);
        if *shape == JobShape::AsA && !was_working {
            let article = AS_AN_ARTICLE.search(m.as_str());
            frame = format!("as {}", article.map(|a| lower(a.str(1))).unwrap_or_else(|| "a".into()));
        } else if *shape == JobShape::Was {
            frame = "was".into();
        } else if was_working && pt::strip(g0).is_empty() {
            frame = "work".into();
        } else if was_working && (JOB_FRAME.contains(&lower(g0)) || JOB_GOT_FRAME.contains(&lower(g0))) {
            frame = groups[..2]
                .iter()
                .filter_map(|p| p.filter(|p| !pt::strip(p).is_empty()).map(|p| pt::strip(p).to_string()))
                .collect::<Vec<_>>()
                .join(" ");
            if *shape == JobShape::GotAt && pt::split(&frame).len() == 1 && JOB_GOT_AN_EMPLOYER.contains(&lower(&role)) {
                frame.push_str(" a");
            }
        }
        job = Some(role);
        break;
    }
    (job, city, frame)
}

/// Python: bot/memory/facts.py:5072 `place_value(phrase)`: the place a corrected value names, or
/// "" when the value is not a place at all (a str).
pub fn place_value(phrase: &str) -> Value {
    let words = pt::split(phrase);
    if words.len() > 1 && !PLACE_MIDDLE.contains(&lower(&words[0])) {
        return Value::from("");
    }
    Value::from(_place_name(phrase))
}

/// `sorted(set(cars.MODELS), key=len, reverse=True)`.
static _CAR_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut names: Vec<String> = bundle_set("bot.knowledge.cars.MODELS").into_iter().collect();
    names.sort();
    names.sort_by(|a, b| pt::len(b).cmp(&pt::len(a)));
    names
});

/// Python: bot/memory/facts.py:5127 `_the_car_name_in(phrase)`: the car name the phrase starts
/// with, or "" when the words name no car.
pub fn _the_car_name_in(phrase: &str) -> String {
    let flat = squash(&lower(&phrase.replace('\'', "")));
    for name in _CAR_NAMES.iter() {
        if flat == *name || flat.starts_with(&format!("{name} ")) {
            return name.clone();
        }
    }
    String::new()
}

/// Python: bot/memory/facts.py:5148 `_car_named_in_passing(text)`: the car a line names without
/// the verb, or None.
pub fn _car_named_in_passing(text: &str) -> Option<String> {
    let past = CAR_IN_THE_PAST.is_match(text);
    for m in CAR_NAMED.finditer(text) {
        let mine = m.name("mine").filter(|s| !s.is_empty());
        if mine.is_some() && past {
            continue;
        }
        let what = m.name("what").filter(|s| !s.is_empty()).or(mine).unwrap_or("");
        let name = _the_car_name_in(what);
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

/// Python: bot/memory/facts.py:5161 `_car(text)`: the car a line says they drive, or None when the
/// verb is not about a car.
pub fn _car(text: &str) -> Option<String> {
    if CAR_ASKED.is_match(text) {
        return None;
    }
    for m in I_DRIVE.finditer(text) {
        let mut phrase = _finish(m.name_str("what"));
        let article = lower(pt::strip(m.name_str("art")));
        for tail in TONE_TAIL.iter() {
            if phrase.ends_with(&format!(" {tail}")) {
                phrase = pt::strip(&drop_last_chars(&phrase, pt::len(tail))).to_string();
            }
        }
        let mut words = pt::split(&phrase);
        while !words.is_empty() && pt::is_digit(&words[0]) {
            words.remove(0);
        }
        let phrase = words.join(" ");
        if words.is_empty() {
            continue;
        }
        if words.iter().any(|w| NOT_A_CAR.contains(w)) || MEASURED.contains(words.last().unwrap()) {
            continue;
        }
        if article.is_empty() && words.last().unwrap().ends_with('s') {
            continue;
        }
        return Some(phrase);
    }
    _car_named_in_passing(text)
}

/// Python: bot/memory/facts.py:5192 `_reads_as_a_plural(name_word, first)`: whether the word after
/// a naming word is the plural of something, so not part of a name.
pub fn _reads_as_a_plural(name_word: &str, first: &str) -> bool {
    let word = lower(name_word);
    ADJECTIVE_NAMING.contains(first) && word.ends_with('s') && !word.ends_with("ss")
}

/// Python: bot/memory/facts.py:5213 `_place_name(phrase)`: the name a place phrase really names,
/// in title case, or "".
pub fn _place_name(phrase: &str) -> String {
    let spaced = phrase.replace('-', " - ");
    let mut words: Vec<String> = pt::split(&spaced);
    let mut the = "";
    if words.len() > 1 && lower(&words[0]) == "the" && COUNTRY_ARTICLE.contains(&lower(&words[1])) {
        the = "the ";
        words.remove(0);
    }
    let mut kept: Vec<String> = Vec::new();
    for (i, word) in words.iter().enumerate() {
        if !the.is_empty() {
            kept.push(word.clone());
            break;
        }
        let low = lower(word);
        if i == 0 && words.len() > 1 && PLACE_MIDDLE.contains(&low) {
            if DETERMINERS.contains(&lower(&words[1])) {
                return String::new();
            }
            if _an_everyday_word(&words[1]) || _reads_as_a_plural(&words[1], &low) {
                return String::new();
            }
            kept.push(word.clone());
            continue;
        }
        if HOME_STOP.contains(&low)
            || NOT_PLACES.contains(&low)
            || PLACE_VERBS.contains(&low)
            || low.ends_with("ly")
            || TAKES_ONE_BACK.contains(&low)
            || NOT_A_PLACE_WORD.contains(&low)
            || word == "-"
        {
            break;
        }
        kept.push(word.clone());
    }
    if kept.is_empty() {
        return String::new();
    }
    if kept.len() == 1 && PLACE_MIDDLE.contains(&lower(&kept[0])) {
        return String::new();
    }
    let name = kept.join(" ");
    if kept.len() == 1 {
        if let Some(caps) = COUNTRY_CAPS.get(&lower(&name)) {
            return format!("{the}{caps}");
        }
    }
    let titled: Vec<String> = pt::split(&name).iter().map(|w| if pt::is_lower(w) { up1(w) } else { w.clone() }).collect();
    format!("{the}{}", titled.join(" "))
}

inline_re!(BORN_WORD, r"\bborn\b", I);
inline_re!(BORN_IN, r"\bborn\s+in\b", I);

/// `max(pairs)[1]` for (position, name) pairs.
fn latest(pairs: &[(usize, String)]) -> Option<String> {
    pairs.iter().max().map(|(_, n)| n.clone())
}

/// Python: bot/memory/facts.py:5257 `_born_place(text)`: the place they were born, when the line
/// says it, or None.
pub fn _born_place(text: &str) -> Option<String> {
    let mut born: Vec<(usize, String)> = Vec::new();
    for pattern in [&*FROM_PLACE, &*FROM_GROWN] {
        for m in pattern.finditer(text) {
            if !BORN_WORD.is_match(m.as_str()) {
                continue;
            }
            let name = _place_name(m.str(1));
            if !name.is_empty() {
                born.push((m.start(), name));
            }
        }
    }
    latest(&born)
}

inline_re!(CLAUSE_PIECES, r"[.;!?,]|\band\b|\bbut\b|\bso\b", 0);
inline_re!(THE_PERSON_FIRST, r"\s*(?:i|im|i'm|ive|i've|id|we|were|we're|weve|we've|we'd)\b", I);

/// Python: bot/memory/facts.py:5295 `_the_person_moved(before)`: true when the clause in front of
/// a move verb has the person themselves as its subject.
pub fn _the_person_moved(before: &str) -> bool {
    let clause = CLAUSE_PIECES.split(before).pop().unwrap_or_default();
    THE_PERSON_FIRST.match_(&clause).is_some()
}

/// Python: bot/memory/facts.py:5302 `_from_place(text)`: the place they say they are from, when
/// the line says where that is, or None.
pub fn _from_place(text: &str) -> Option<String> {
    let mut born: Vec<(usize, String)> = Vec::new();
    let mut said: Vec<(usize, String)> = Vec::new();
    for (k, pattern) in [&*FROM_PLACE, &*FROM_MORE, &*FROM_GROWN, &*FROM_A_MOVE].into_iter().enumerate() {
        for m in pattern.finditer(text) {
            if k == 3 && !_the_person_moved(&text[..m.start()]) {
                continue;
            }
            let phrase = m.str(1);
            if k == 2 && !_grown_up_in_a_place(phrase, text) {
                continue;
            }
            let name = _place_name(phrase);
            if name.is_empty() {
                continue;
            }
            if BORN_IN.is_match(m.as_str()) {
                born.push((m.start(), name));
            } else {
                said.push((m.start(), name));
            }
        }
    }
    if !said.is_empty() {
        return latest(&said);
    }
    latest(&born)
}

inline_re!(CAPITALISED_WORDS, r"[A-Z][a-z]+", 0);

/// Python: bot/memory/facts.py:5345 `_places(text)`: places the line named, the one they live in
/// if it says where that is, and the words the home was said with.
pub fn _places(text: &str) -> (Vec<String>, Option<String>, String) {
    let mut places: Vec<String> = Vec::new();
    let mut home: Option<String> = None;
    let mut home_said = String::new();
    let has = |places: &Vec<String>, word: &str| places.iter().any(|p| lower(p) == lower(word));
    for pattern in [&*HOME_DWELL, &*HOME_PLACE] {
        for m in pattern.finditer(text) {
            let word = _place_name(m.name_str("place"));
            if word.is_empty() {
                continue;
            }
            if !has(&places, &word) {
                places.push(word.clone());
            }
            let ahead = m.name("planned").is_some_and(|p| !p.is_empty()) && FUTURE.is_match(text);
            if home.is_none() && !m.name("stay").is_some_and(|s| !s.is_empty()) && !ahead {
                home = Some(word.clone());
                let place_at = m.name_span("place").map(|s| s.0).unwrap_or(0);
                if let Some(said_with) = A_HOME_BEFORE.search(&text[..place_at]) {
                    home_said = format!("{} {}", squash(said_with.as_str()), word);
                }
            }
            break;
        }
        if home.as_ref().is_some_and(|h| !h.is_empty()) {
            break;
        }
    }
    if home.is_none() {
        for m in A_MOVE_FROM_THEN_TO.finditer(text) {
            if !_the_person_moved(&text[..m.start()]) {
                continue;
            }
            let word = _place_name(m.name_str("place"));
            if word.is_empty() {
                continue;
            }
            if !has(&places, &word) {
                places.push(word.clone());
            }
            if !FUTURE.is_match(text) {
                home = Some(word);
            }
            break;
        }
    }
    for m in PLACE_AFTER.finditer(text) {
        let mut word = m.str(1).to_string();
        let first = lower(pt::split_ref(&word).first().copied().unwrap_or(""));
        if NOT_PLACES.contains(&first) || NOT_PLACES.contains(&lower(&word)) {
            continue;
        }
        if PLACE_MIDDLE.contains(&first) {
            let from = m.span(1).map(|s| s.0).unwrap_or(0);
            let pair = pt::split_ref(&text[from..]).into_iter().take(2).collect::<Vec<_>>().join(" ");
            word = _place_name(&pair);
            if word.is_empty() {
                continue;
            }
        }
        if !places.contains(&word) {
            places.push(word);
        }
    }
    for m in PLACE_LOWER_NAMED.finditer(text) {
        if _an_everyday_word(m.str(2)) {
            continue;
        }
        let word = _place_name(&format!("{} {}", m.str(1), m.str(2)));
        if !word.is_empty() && !has(&places, &word) {
            places.push(word);
        }
    }
    for m in PLACE_LOWER.finditer(text) {
        let word = _finish(m.str(1));
        let low = lower(&word);
        if word.is_empty() || NOT_PLACES.contains(&low) || PLACE_VERBS.contains(&low) {
            continue;
        }
        if PLACE_MIDDLE.contains(&low) {
            continue;
        }
        if word.chars().next().is_some_and(char::is_uppercase) && places.contains(&word) {
            continue;
        }
        if places.iter().any(|p| lower(p).starts_with(&format!("{low} "))) {
            continue;
        }
        if !has(&places, &word) {
            places.push(up1(&word));
        }
    }
    if places.is_empty() && SHORT_LIST.match_(pt::strip(text)).is_some() {
        for word in CAPITALISED_WORDS.findall_str(text) {
            if !NOT_PLACES.contains(&lower(&word)) && !places.contains(&word) {
                places.push(word);
            }
        }
    }
    (places, home, home_said)
}

// ---------------------------------------------------------------------------------------------
// who a fact is about, and the allergy items

fres!(QUESTION_OPEN, CLAUSE_JOIN, SUBJECT, A_ME_WITH_SOMEONE, BARE_RELATION, QUESTION_START);
fsets!(
    _QUANTITY_WORDS, ALLERGY_STOP, BETWEEN_PERSON_AND_FACT, NOT_A_BARE_PERSON_AHEAD, FILLER_TAIL,
    NOT_A_FOOD, A_GENERIC_FOOD, NOT_A_FOOD_IN_IT, ONE_MORE_DETERMINERS, NOT_AN_ADDITION,
);
fstrs!(FILLER_TAIL_PHRASES);

/// Python: bot/memory/facts.py:5476 `stated_not_asked(text)`: the part of a yes/no question that
/// states something, or "" when it only asks.
pub fn stated_not_asked(text: &str) -> String {
    let said = squash(pt::strip(text));
    if said.is_empty() || QUESTION_OPEN.match_(&said).is_none() {
        return said;
    }
    let kept: Vec<String> = CLAUSE_JOIN
        .split(&said)
        .iter()
        .map(|p| pt::strip_chars(p, " ,.;:!?").to_string())
        .filter(|p| !p.is_empty() && QUESTION_OPEN.match_(p).is_none())
        .collect();
    kept.join(", ")
}

/// Python: bot/memory/facts.py:5524 `_the_person_is_in_it(clause)`: whether the clause says the
/// thing is about the person as well as somebody else.
pub fn _the_person_is_in_it(clause: &str) -> bool {
    A_ME_WITH_SOMEONE.is_match(clause)
}

/// The largest char boundary at or before `pos` (a caller's offset, clamped).
fn boundary_at(text: &str, pos: usize) -> usize {
    let mut p = pos.min(text.len());
    while !text.is_char_boundary(p) {
        p -= 1;
    }
    p
}

/// Python: bot/memory/facts.py:5529 `subject(text, pos)`: who the clause that ends at pos is
/// about, as [kind, who]. `pos` is a byte offset into `text` (a Rust match position).
pub fn subject(text: &str, pos: i64) -> Vec<String> {
    let (kind, who) = subject_at(text, boundary_at(text, pos.max(0) as usize));
    vec![kind, who]
}

/// `subject(text, pos)` as (kind, who), with `pos` a byte offset.
pub fn subject_at(text: &str, pos: usize) -> (String, String) {
    let start = clause_start(text, pos);
    let clause = &text[start..pos];
    let mut kind = String::new();
    let mut who = String::new();
    for m in SUBJECT.finditer(clause) {
        if m.name("rel").is_some_and(|r| !r.is_empty()) {
            kind = "other".into();
            who = lower(&squash(&format!("{}{}", m.name_str("rel"), m.name_str("inlaw"))));
            if let Some(full) = SPOKEN_RELATION.get(&who) {
                who = full.clone();
            }
        } else if m.name("pron").is_some_and(|p| !p.is_empty()) || m.name("her").is_some_and(|p| !p.is_empty()) {
            kind = "other".into();
            who = String::new();
        } else {
            kind = "user".into();
            who = String::new();
        }
    }
    if kind == "other" && !who.is_empty() && _the_person_is_in_it(clause) {
        kind = "user".into();
    }
    if kind == "other" && !who.is_empty() {
        let pair = a_chained_person(clause);
        if !pair.is_empty() && pair.split("'s ").next() == Some(who.as_str()) {
            who = pair;
        }
    }
    if kind.is_empty() {
        let (bare_who, bare_name) = _the_person_without_a_relation(clause);
        if !bare_who.is_empty() || !bare_name.is_empty() {
            kind = "other".into();
            who = bare_who;
            let pair = a_chained_person(clause);
            if !pair.is_empty() && pair.split("'s ").next() == Some(who.as_str()) {
                who = pair;
            }
        }
    }
    (kind, who)
}

/// Python: bot/memory/facts.py:5628 `_a_person_word(word)`: a word the way a person says it, so
/// "i'm", "im" and "I’m" are one word.
pub fn _a_person_word(word: &str) -> String {
    lower(pt::strip_chars(pt::strip(word), ".,!?;:\"'")).replace('\u{2019}', "'").replace('\'', "")
}

/// Python: bot/memory/facts.py:5633 `_the_person_without_a_relation(clause)`: (who, name) for a
/// person the clause names without a "my": ("mum", ""), ("", "Tom").
pub fn _the_person_without_a_relation(clause: &str) -> (String, String) {
    let mut words: Vec<String> = pt::split(clause)
        .iter()
        .map(|w| pt::strip_chars(pt::strip_chars(w, " ,.;:!?"), "'").to_string())
        .filter(|w| !w.is_empty())
        .collect();
    while !words.is_empty()
        && ["and", "but", "so", "then", "also", "oh", "ok", "okay", "well", "now", "btw"].contains(&lower(&words[0]).as_str())
    {
        words.remove(0);
    }
    while !words.is_empty() && BETWEEN_PERSON_AND_FACT.contains(&lower(words.last().unwrap())) {
        words.pop();
    }
    if words.is_empty() || words.len() > 3 {
        return (String::new(), String::new());
    }
    if words[..words.len() - 1].iter().any(|w| NOT_A_BARE_PERSON_AHEAD.contains(&lower(w)))
        || NOT_A_BARE_PERSON_AHEAD.contains(&lower(words.last().unwrap()))
    {
        return (String::new(), String::new());
    }
    let said = words.join(" ");
    if let Some(found) = BARE_RELATION.search(&said) {
        let mut who = squash(&lower(&format!("{}{}", found.name_str("rel"), found.name_str("inlaw"))));
        if who.ends_with('s') && RELATIONS.contains(&who[..who.len() - 1].to_string()) {
            who.pop();
        }
        let who = SPOKEN_RELATION.get(&who).cloned().unwrap_or(who);
        return (who, String::new());
    }
    if words.len() == 1 {
        let word = &words[0];
        let pw = _a_person_word(word);
        if NOT_A_PERSON_WORD.contains(&pw) || DIET_GROUP.contains(&pw) || RELATIONS.contains(&pw) || A_PRONOUN_NOT_A_NAME.contains(&pw) {
            return (String::new(), String::new());
        }
        if _may_be_a_name(word) && _name_after_a_title(word) {
            return (String::new(), word.clone());
        }
    }
    (String::new(), String::new())
}

const END_PUNCT: &str = ".,!?;:'\"";

/// Python: bot/memory/facts.py:5683 `_trim_filler(text)`.
pub fn _trim_filler(text: &str) -> String {
    let mut words = pt::split(text);
    loop {
        while !words.is_empty() && FILLER_TAIL.contains(&lower(pt::strip_chars(words.last().unwrap(), END_PUNCT))) {
            words.pop();
        }
        let from = words.len().saturating_sub(3);
        let tail = lower(pt::strip_chars(&words[from..].join(" "), END_PUNCT));
        let Some(cut) = FILLER_TAIL_PHRASES.iter().find(|p| tail.ends_with(p.as_str())) else {
            break;
        };
        let joined = words.join(" ");
        let stripped = pt::strip_chars(&joined, END_PUNCT);
        words = pt::split(&drop_last_chars(stripped, pt::len(cut)));
    }
    words.join(" ")
}

/// Python: bot/memory/facts.py:5717 `_trim_time_tail(item)`: the food off the front of a time:
/// "chilli after 6" is chilli, "dairy before work" dairy.
pub fn _trim_time_tail(item: &str) -> String {
    let mut words = pt::split(item);
    while words.len() > 1 {
        let last = pt::strip_chars(words.last().unwrap(), END_PUNCT).to_string();
        if NOT_A_FOOD_IN_IT.contains(&last) || pt::is_digit(&last) || ["the", "a", "my", "our"].contains(&last.as_str()) {
            words.pop();
        } else {
            break;
        }
    }
    words.join(" ")
}

/// Python: bot/memory/facts.py:5739 `_a_modifier_before_a_generic_noun(words)`: whether the thing
/// said is a word that names it plus a generic noun: "spicy food".
pub fn _a_modifier_before_a_generic_noun(words: &[String]) -> bool {
    if words.len() < 2 {
        return false;
    }
    if !A_GENERIC_FOOD.contains(&lower(pt::strip_chars(words.last().unwrap(), END_PUNCT))) {
        return false;
    }
    !words[..words.len() - 1].iter().any(|w| ALLERGY_STOP.contains(w))
}

/// Python: bot/memory/facts.py:5748 `_only_stop_words(item)`: whether every word of a piece is one
/// the allergy reader knows is not an allergen.
pub fn _only_stop_words(item: &str) -> bool {
    let words = pt::split(item);
    !words.is_empty() && words.iter().all(|w| NOT_A_FOOD.contains(w) || ALLERGY_STOP.contains(w))
}

inline_re!(NOT_OR_A_JOIN, r"\bnot\b|\b(?:but|so|because|which)\b", 0);
inline_re!(COMMA_OR_AND_SPACED, r",| and ", 0);

/// Python: bot/memory/facts.py:5754 `_allergy_items(chunk)`: the things in "nuts and shellfish,
/// not peanuts": ["nuts", "shellfish"].
pub fn _allergy_items(chunk: &str) -> Vec<String> {
    let mut items: Vec<String> = Vec::new();
    let chunk = NOT_OR_A_JOIN.split(chunk).into_iter().next().unwrap_or_default();
    for piece in COMMA_OR_AND_SPACED.split(&chunk) {
        let item = _trim_time_tail(&_trim_filler(&_finish(&piece)));
        if QUESTION_START.match_(&lower(&item)).is_some() {
            break;
        }
        if item.is_empty() || pt::len(&item) < 2 {
            continue;
        }
        let words = pt::split(&item);
        if words.iter().any(|w| ALLERGY_STOP.contains(w)) && !_a_modifier_before_a_generic_noun(&words) {
            continue;
        }
        if ["it", "them", "that", "this"].contains(&item.as_str()) || items.contains(&item) {
            continue;
        }
        if words.iter().any(|w| NOT_A_FOOD_IN_IT.contains(w)) && !words.iter().any(|w| ONE_MORE_FOODS.contains(w)) {
            continue;
        }
        if _only_stop_words(&item) {
            continue;
        }
        items.push(item);
    }
    items
}

// ---------------------------------------------------------------------------------------------
// one more thing, shared limits, take-backs and losses of animals, allergies, names in passing

fres!(
    I_SHARE_IT, I_SHARE_IT_SHORT, SOMEONE_SHARES_IT_VERB_FIRST, SOMEONE_SHARES_IT,
    SOMEONE_SHARES_IT_PRONOUN, ONE_TAKEN, DIY_OF_ALL, ONE_OF_THEM, ALL_OF_THEM, AGE_MONTHS_TAIL,
    AGE_TO_COME, OTHER_CALLED, OTHER_NAME_IS, OTHER_NAMED_BARE, OTHER_NAMED_BEFORE_VERB,
    OTHER_NAMED_CAPITAL, OTHER_NAME_POSSESSIVE, OTHER_NAMED_MIDLINE,
);
fsets!(ANY_SUBJECT, NOT_A_NAME_MIDLINE, NOT_A_NAME_THERE, NAMES_THAT_END_LIKE_A_VERB);
fstrs!(NOT_NAME_ENDING);

pub const LAST_PERSON: &str = "@last";

static VERB_FORMS: LazyLock<HashSet<String>> = LazyLock::new(|| bundle_set("bot.understanding.kinds.VERB_FORMS"));

inline_re!(COMMA_AND_OR, r",| and | or ", 0);

/// Python: bot/memory/facts.py:5796 `one_more(text)`: the one more thing a line names and nothing
/// else: "and milk too" -> ["milk"].
pub fn one_more(text: &str) -> Vec<String> {
    let mut said = low_squash(text);
    if said.is_empty() {
        return Vec::new();
    }
    let typed = said.clone();
    for _ in 0..6 {
        let before = said.clone();
        said = pt::strip_chars(&ONE_MORE_HEAD.sub("", &said), " ,").to_string();
        said = pt::strip_chars(&ONE_MORE_TAIL.sub("", &said), " ,.!?").to_string();
        if said == before {
            break;
        }
    }
    if said.is_empty() || said == typed {
        return Vec::new();
    }
    let mut items: Vec<String> = Vec::new();
    for part in COMMA_AND_OR.split(&said) {
        let mut words: Vec<String> = pt::split(&part)
            .iter()
            .filter(|w| !pt::strip_chars(w, "'\u{2019}").is_empty())
            .map(|w| pt::strip_chars(w, "'\u{2019}").to_string())
            .collect();
        if !words.is_empty() && NOT_AN_ADDITION.contains(&lower(&words[0])) {
            continue;
        }
        while !words.is_empty() && ONE_MORE_DETERMINERS.contains(&words[0]) {
            words.remove(0);
        }
        if words.is_empty() || words.len() > 2 {
            continue;
        }
        let last_word = words.last().unwrap().clone();
        if !ONE_MORE_FOODS.contains(&last_word) {
            continue;
        }
        let head = words[..words.len() - 1].join(" ");
        let last = ALLERGEN_PLURALS.get(&last_word).cloned().unwrap_or(last_word);
        let item = pt::strip(&format!("{head} {last}")).to_string();
        if !items.contains(&item) {
            items.push(item);
        }
    }
    items
}

/// Python: bot/memory/facts.py:5899 `the_limit_they_share(text)`: "im too", "my son is too": the
/// limit just said is theirs as well, and who "theirs" is ("i", a relation, or "@last").
pub fn the_limit_they_share(text: &str) -> String {
    let said = pt::strip_chars(&low_squash(text), " !.,").to_string();
    if said.is_empty() || pt::split(&said).len() > 6 {
        return String::new();
    }
    if I_SHARE_IT.match_(&said).is_some() || I_SHARE_IT_SHORT.match_(&said).is_some() {
        return "i".into();
    }
    if let Some(found) = SOMEONE_SHARES_IT.match_(&said).or_else(|| SOMEONE_SHARES_IT_VERB_FIRST.match_(&said)) {
        let who = pt::strip(found.name_str("who")).to_string();
        return if who == "own" || who == "self" { String::new() } else { who };
    }
    if SOMEONE_SHARES_IT_PRONOUN.match_(&said).is_some() {
        return LAST_PERSON.into();
    }
    String::new()
}

/// Python: bot/memory/facts.py:5928 `denied_before(text, start)`: whether the words at `start`
/// are said not to be true: "im not allergic to nuts". (`start` is a byte offset.)
pub fn denied_before(text: &str, start: usize) -> bool {
    let from = back_chars(text, start, 40);
    DENIAL_WORDS.is_match(&text[from..start])
}

/// Python: bot/memory/facts.py:5933 `_clause(text, start)`: the words before the fact at `start`,
/// from the nearest clause edge.
pub fn _clause(text: &str, start: usize) -> String {
    let head = &text[..start];
    let mut cut = 0;
    for edge in CLAUSE_EDGE.finditer(head) {
        cut = edge.end();
    }
    head[cut..].to_string()
}

/// Python: bot/memory/facts.py:5942 `_the_clause_before(text, start, edges=4)`: the clause before
/// the fact at `start`, with the clause it continues put back.
pub fn _the_clause_before(text: &str, start: usize, edges: usize) -> String {
    let head = &text[..start];
    let mut parts: Vec<&str> = Vec::new();
    let mut last = 0;
    for edge in CLAUSE_EDGE.finditer(head) {
        parts.push(&head[last..edge.end()]);
        last = edge.end();
    }
    parts.push(&head[last..]);
    let mut said = parts[parts.len() - 1].to_string();
    let mut walked = 0;
    while walked + 1 < parts.len() && walked < edges && !_opens_a_subject(parts[parts.len() - 1 - walked]) {
        walked += 1;
        said = format!("{}{}", parts[parts.len() - 1 - walked], said);
    }
    said
}

inline_re!(NOT_A_WORD_LETTER, r"[^a-z']", 0);

/// Python: bot/memory/facts.py:5969 `_opens_a_subject(clause)`: whether this clause starts by
/// naming who it is about.
pub fn _opens_a_subject(clause: &str) -> bool {
    let words = pt::split(&clause.replace(',', " "));
    let Some(first) = words.first() else {
        return false;
    };
    ANY_SUBJECT.contains(&NOT_A_WORD_LETTER.sub("", &lower(first)))
}

/// Python: bot/memory/facts.py:5977 `_around(text, start)`: the whole clause the words at `start`
/// sit in, from one edge to the next.
pub fn _around(text: &str, start: usize) -> String {
    let mut left = 0;
    for edge in CLAUSE_EDGE.finditer(&text[..start]) {
        left = edge.end();
    }
    let right = CLAUSE_EDGE.search_at(text, start).map(|e| e.start()).unwrap_or(text.len());
    if left >= right {
        return String::new();
    }
    text[left..right].to_string()
}

/// `str((pet or {}).get(key) or "").strip()` for a pet record.
fn pet_field(pet: &Value, key: &str) -> String {
    pt::strip(&str_or_empty(pet.get(key))).to_string()
}

/// `dict.fromkeys(items)` order: first sighting kept.
fn dedupe(items: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for i in items {
        if !out.contains(&i) {
            out.push(i);
        }
    }
    out
}

/// A whole-word search for `word` (`re.search(r"\b" + re.escape(word) + r"\b", text, flags)`).
fn word_re(word: &str, flags: u32) -> Re {
    Re::new(&format!(r"\b{}\b", crate::py::re::escape(word)), flags)
}

/// Python: bot/memory/facts.py:5989 `_own_named(text, start, own)`: the animals the store holds
/// whose name the line has said before `start`.
pub fn _own_named(text: &str, start: usize, own: &Value) -> Vec<String> {
    let mut found = Vec::new();
    let lowered = lower(text);
    let head = pt::slice(&lowered, 0, Some(pt::char_index(text, start) as isize));
    for pet in own.items() {
        let name = lower(&pet_field(pet, "name"));
        let animal = lower(&pet_field(pet, "animal"));
        if name.is_empty() || animal.is_empty() || pt::len(&name) < 3 {
            continue;
        }
        if word_re(&name, 0).is_match(&head) {
            found.push(animal);
        }
    }
    dedupe(found)
}

/// Python: bot/memory/facts.py:6009 `_own_in_denial(text, own)`: the store's pets the line says are
/// gone by name, when it named no animal kind (the records themselves).
pub fn _own_in_denial(text: &str, own: &Value) -> Vec<Value> {
    let mut gone: Vec<Value> = Vec::new();
    for pet in own.items() {
        let name = pet_field(pet, "name");
        let animal = lower(&pet_field(pet, "animal"));
        if name.is_empty() || animal.is_empty() || pt::len(&name) < 3 {
            continue;
        }
        for m in word_re(&name, I).finditer(text) {
            if not_had(text, m.start()) {
                if !gone.contains(pet) {
                    gone.push(pet.clone());
                }
                break;
            }
        }
    }
    gone
}

/// `int(pet.get("count") or 1)`.
fn count_of(pet: &Value) -> i64 {
    match pet.get("count") {
        Some(v) if v.truthy() => match v {
            Value::Int(i) => *i,
            Value::Bool(b) => *b as i64,
            Value::Float(f) => f.trunc() as i64,
            Value::Str(s) => py_int(s).unwrap_or(1),
            _ => 1,
        },
        _ => 1,
    }
}

/// Python: bot/memory/facts.py:6042 `given_away_one(text, own=None)`.
///
/// CONTRACT NOTE: Python returns the pet *record* to take one from (`[dict]`); the skeleton gave
/// this function `Vec<String>`, which cannot hold one. This keeps the contract and returns each
/// record as its JSON text; `given_away_one_records` returns the records themselves.
pub fn given_away_one(text: &str, own: &Value) -> Vec<String> {
    given_away_one_records(text, own).iter().map(|r| r.to_plain().to_string()).collect()
}

/// `given_away_one(text, own)` as Python returns it: the one animal record a line gives away
/// without saying which, out of a set on the store.
pub fn given_away_one_records(text: &str, own: &Value) -> Vec<Value> {
    if !ONE_OF_THEM.is_match(text) || !ONE_TAKEN.is_match(text) {
        return Vec::new();
    }
    let mut kinds: Vec<(String, Value)> = Vec::new();
    for pet in own.items() {
        if !pet.is_dict() {
            continue;
        }
        let animal = lower(&pet_field(pet, "animal"));
        let count = match pet.get("count") {
            Some(v) if v.truthy() => v.as_f64().unwrap_or(1.0),
            _ => 1.0,
        };
        if animal.is_empty() || count < 2.0 {
            continue;
        }
        if !kinds.iter().any(|(k, _)| *k == animal) {
            kinds.push((animal, pet.clone()));
        }
    }
    if kinds.len() != 1 {
        return Vec::new();
    }
    vec![kinds[0].1.clone()]
}

/// Python: bot/memory/facts.py:6066 `_pointed_at(text, start, plural)`: the animals a pronoun at
/// `start` can be pointing at: the ones the line named first.
pub fn _pointed_at(text: &str, start: usize, plural: bool) -> Vec<String> {
    let named = dedupe(PETS.finditer(&text[..start]).iter().map(|m| lower(m.str(1))).collect());
    if named.is_empty() {
        return named;
    }
    if plural {
        named
    } else {
        vec![named.last().unwrap().clone()]
    }
}

/// Python: bot/memory/facts.py:6081 `not_had(text, start)`: true when the line says the pet at
/// `start` is not, or no longer, theirs.
pub fn not_had(text: &str, start: usize) -> bool {
    let clause = _clause(text, start);
    if HEDGE.is_match(&clause) {
        return false;
    }
    if NOT_HAVE_LONGER.is_match(&clause)
        || _have_none(&format!("{clause} {}", &text[start..]))
        || WITHOUT_ANIMAL.is_match(&clause)
        || NOT_JUST.is_match(&clause)
        || taken_back_before(text, start)
    {
        return true;
    }
    GIVEN_AWAY.finditer(text).iter().any(|m| m.start() <= start && start <= m.end())
}

/// Python: bot/memory/facts.py:6100 `_have_none(clause)`: whether the clause says the animal is not
/// theirs, read without a bare verb deciding it.
pub fn _have_none(clause: &str) -> bool {
    for m in HAVE_NONE.finditer(clause) {
        if A_COMPOUND_AFTER.match_(&clause[m.end()..]).is_some() {
            continue;
        }
        return true;
    }
    false
}

/// `pattern.match(text, pos)`: a match that starts exactly at `pos`.
fn match_at<'t>(pattern: &'t Re, text: &'t str, pos: usize) -> Option<Match<'t>> {
    pattern.search_at(text, pos).filter(|m| m.start() == pos)
}

/// Python: bot/memory/facts.py:6115 `taken_back_before(text, start)`: true when the words in front
/// of the animal take it back, read with the whole line.
pub fn taken_back_before(text: &str, start: usize) -> bool {
    let clause = _clause(text, start);
    if HEDGE.is_match(&clause) {
        return false;
    }
    if TAKEN_BACK_BEFORE.is_match(&clause) {
        return true;
    }
    let rest = match match_at(&PETS, text, start) {
        Some(animal) => &text[animal.end()..],
        None => &text[start..],
    };
    if NO_THEN_A_THING.match_(&clause).is_some() {
        return false;
    }
    BARE_NO.match_(&clause).is_some() && pt::strip_chars(rest, " .,!?;:").is_empty()
}

inline_re!(ALLERGIC_TO_FIRST, r"^allergic to\s+", I);

/// Push `items` onto the list at `key` of a dict value, each one only once.
fn extend_unique(list: &mut Value, items: &[String]) {
    for item in items {
        if !list.contains_str(item) {
            list.push(item.as_str());
        }
    }
}

/// Python: bot/memory/facts.py:6139 `denials(text, own=None)`: the facts a line says the person
/// does *not* have, in the shape extract() keeps them.
pub fn denials(text: &str, own: &Value) -> Value {
    let said = text;
    let mut denied = Value::dict();
    for m in ALLERGY.finditer(said) {
        if !denied_before(said, m.start()) {
            continue;
        }
        let (kind, who) = subject_at(said, m.start());
        if kind != "other" {
            continue;
        }
        let person = _a_person(said, m.end(), &who);
        let items: Vec<String> =
            _allergy_items(m.str(1)).into_iter().map(|i| ALLERGEN_PLURALS.get(&i).cloned().unwrap_or(i)).collect();
        if !person.truthy() || items.is_empty() {
            continue;
        }
        let theirs = denied.setdefault("theirs", Value::list());
        let at = theirs.items().iter().position(|seen| _same_person(seen, &person));
        let at = match at {
            Some(i) => i,
            None => {
                let mut entry = person.clone();
                entry.set_key("allergies", Value::list());
                theirs.push(entry);
                theirs.len() - 1
            }
        };
        let entry = &mut theirs.items_mut().unwrap()[at];
        extend_unique(entry.setdefault("allergies", Value::list()), &items);
    }
    for m in CONTRAST.finditer(said) {
        for item in _allergy_items(m.name_str("old")) {
            let name = ALLERGEN_PLURALS.get(&item).cloned().unwrap_or(item);
            let list = denied.setdefault("allergies", Value::list());
            if !list.contains_str(&name) {
                list.push(name);
            }
        }
    }
    for m in DENIAL_OF.finditer(said) {
        if m.name("food").is_some_and(|f| !f.is_empty()) {
            let items = _allergy_items(&ALLERGIC_TO_FIRST.sub("", m.name_str("food")));
            if !items.is_empty() {
                extend_unique(denied.setdefault("allergies", Value::list()), &items);
            }
            continue;
        }
        if m.name("condition").is_some_and(|c| !c.is_empty()) {
            denied.set_key("condition", lower(m.name_str("condition")));
            continue;
        }
        if m.name("diet").is_some_and(|d| !d.is_empty()) {
            let diet = lower(m.name_str("diet"));
            denied.set_key("diet", DIET_NAMES.get(&diet).cloned().unwrap_or(diet));
        }
    }
    for m in DIET_OFF.finditer(said) {
        let plan = ["plan", "off", "gone", "quit"].iter().find_map(|g| m.name(g).filter(|s| !s.is_empty())).unwrap_or("");
        if plan.is_empty() {
            continue;
        }
        let plan = pt::split(&lower(plan).replace('-', " ")).join("-");
        let value = match DIET_NAMES.get(&plan) {
            Some(v) => v.clone(),
            None if plan.starts_with("diet") => "on a diet".into(),
            None => plan,
        };
        denied.set_key("diet", value);
    }
    for m in DIET_BACK.finditer(said) {
        let back = m.name("back").filter(|s| !s.is_empty()).or_else(|| m.name("eat").filter(|s| !s.is_empty())).unwrap_or("");
        let food = _the_food_named(back);
        if !food.is_empty() {
            denied.set_key("diet", food);
        }
    }
    if NO_PETS.is_match(said) {
        denied.set_key("pets", str_list(&[EVERYTHING]));
    }
    let mut animals: Vec<Value> = Vec::new();
    let others = _someone_elses(said);
    for m in PETS.finditer(said) {
        if !not_had(said, m.start()) {
            continue;
        }
        if others.iter().any(|(start, stop)| *start <= m.start() && m.start() < *stop) {
            continue;
        }
        let animal = Value::from(lower(m.str(1)));
        if !animals.contains(&animal) {
            animals.push(animal);
        }
    }
    if animals.is_empty() {
        if let Some(m) = PET_PRONOUN.finditer(said).into_iter().find(|m| not_had(said, m.start())) {
            let mut pointed = _the_one_they_have(said, &m, own);
            if pointed.is_empty() && !_names_a_relative(said) {
                pointed = _only_pet(own);
            }
            for animal in pointed {
                let animal = Value::from(animal);
                if !animals.contains(&animal) {
                    animals.push(animal);
                }
            }
        }
    }
    if animals.is_empty() {
        animals = _own_in_denial(said, own);
    }
    let everything = str_list(&[EVERYTHING]);
    if !animals.is_empty() && denied.get("pets") != Some(&everything) {
        let had: Vec<Value> = denied.get("pets").map(|p| p.items().to_vec()).unwrap_or_default();
        let merged = _merge_pets(&had, &_one_of_several(&animals, said, own));
        denied.set_key("pets", Value::from(merged));
    }
    denied
}

/// Python: bot/memory/facts.py:6233 `their_back(text)`.
///
/// CONTRACT NOTE: Python returns records (`[{"who", "pronoun", "kind", "food"}]`); the skeleton
/// gave this function `Vec<String>`, which cannot hold them. This keeps the contract and returns
/// each record as its JSON text; `their_back_records` returns the records themselves.
pub fn their_back(text: &str) -> Vec<String> {
    their_back_records(text).iter().map(|r| r.to_plain().to_string()).collect()
}

/// `their_back(text)` as Python returns it: a line that puts back what somebody else does not eat
/// or does not like, as [{"who", "pronoun", "kind", "food"}].
pub fn their_back_records(text: &str) -> Vec<Value> {
    let marked = A_CORRECTION.is_match(text);
    let mut found = Vec::new();
    for m in THEIR_BACK.finditer(text) {
        let emph = m.name("emph").filter(|s| !s.is_empty());
        if !marked && emph.is_none() {
            continue;
        }
        if A_NEGATOR_IN_BETWEEN.is_match(m.name_str("gap")) {
            continue;
        }
        let verb = lower(emph.or_else(|| m.name("verb").filter(|s| !s.is_empty())).unwrap_or(""));
        if verb.is_empty() {
            continue;
        }
        let kind = if verb.starts_with("eat") || ["does", "do", "did"].contains(&verb.as_str()) { "diet" } else { "taste" };
        let food = low_squash(m.name_str("food"));
        if food.is_empty() {
            continue;
        }
        let subj = low_squash(m.name_str("subj"));
        let named = ["my ", "his ", "her ", "their ", "our "].iter().any(|p| subj.starts_with(p));
        let who = if named { pt::split(&subj).last().cloned().unwrap_or_default() } else { String::new() };
        let pronoun = if named { String::new() } else { subj.clone() };
        found.push(dict(vec![("who", who.into()), ("pronoun", pronoun.into()), ("kind", kind.into()), ("food", food.into())]));
    }
    found
}

inline_re!(THEM_THOSE_THESE, r"\b(?:them|those|these)\b", I);

/// Python: bot/memory/facts.py:6271 `_one_of_several(animals, said, own)`: a kind word takes one
/// animal when the store holds several: "we gave the cat away". (Items are animal words or records.)
pub fn _one_of_several(animals: &[Value], said: &str, own: &Value) -> Vec<Value> {
    let one_of = ONE_OF_THEM.is_match(said);
    let whole = (ALL_OF_THEM.is_match(said) || THEM_THOSE_THESE.is_match(said)) && !one_of;
    let mut out = Vec::new();
    for animal in animals {
        let Some(word) = animal.as_str() else {
            out.push(animal.clone());
            continue;
        };
        let same: Vec<&Value> =
            own.items().iter().filter(|pet| pet.is_dict() && lower(&pet_field(pet, "animal")) == word).collect();
        let held: i64 = same.iter().map(|pet| count_of(pet)).sum();
        let plural = whole
            || (Re::new(&format!(r"\b{}s\b", crate::py::re::escape(word)), I).is_match(said) && !one_of);
        if held > 1 && !plural {
            out.push((*same.last().unwrap()).clone());
        } else {
            out.push(animal.clone());
        }
    }
    out
}

/// Python's `str(item)` for a pet take-back item (a word, or a record).
fn item_str(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        other => value_str(other),
    }
}

/// Python: bot/memory/facts.py:6303 `_merge_pets(had, more)`: the take-backs of one line, without
/// saying the same one twice.
pub fn _merge_pets(had: &[Value], more: &[Value]) -> Vec<Value> {
    let mut out: Vec<Value> = had.to_vec();
    let mut words: HashSet<String> = out.iter().filter(|i| !i.is_dict()).map(|i| lower(pt::strip(&item_str(i)))).collect();
    let mut records: Vec<Value> = out.iter().filter(|i| i.is_dict()).cloned().collect();
    for item in more {
        if item.is_dict() {
            if !records.contains(item) {
                records.push(item.clone());
            }
        } else {
            let w = lower(pt::strip(&item_str(item)));
            if !words.contains(&w) {
                words.insert(w);
                out.push(item.clone());
            }
        }
    }
    let plain: Vec<Value> = out.into_iter().filter(|i| !i.is_dict()).collect();
    if plain.is_empty() {
        records
    } else {
        plain.into_iter().chain(records).collect()
    }
}

/// Python: bot/memory/facts.py:6323 `_the_one_they_have(said, match, own)`: the animal a bare
/// "him"/"her" points at: one the line already named, or one the store holds.
pub fn _the_one_they_have(said: &str, m: &Match, own: &Value) -> Vec<String> {
    let pointed = _pointed_at(said, m.start(), PLURAL_PRONOUN.contains(&lower(m.name_str("pron"))));
    if !pointed.is_empty() {
        return pointed;
    }
    _own_named(said, m.start(), own)
}

/// Python: bot/memory/facts.py:6334 `_only_pet(own)`: the one animal the store holds, when it holds
/// exactly one.
pub fn _only_pet(own: &Value) -> Vec<String> {
    let held = dedupe(own.items().iter().map(|pet| lower(&pet_field(pet, "animal"))).collect());
    let held: Vec<String> = held.into_iter().filter(|a| !a.is_empty()).collect();
    if held.len() == 1 {
        held
    } else {
        Vec::new()
    }
}

static RELATION_WORD_RES: LazyLock<Vec<Re>> =
    LazyLock::new(|| RELATIONS.iter().map(|w| Re::new(&format!(r"\b{w}\b"), I)).collect());

/// Python: bot/memory/facts.py:6341 `_names_a_relative(said)`.
pub fn _names_a_relative(said: &str) -> bool {
    RELATION_WORD_RES.iter().any(|r| r.is_match(said))
}

/// Python: bot/memory/facts.py:6345 `_may_be_an_animal(said, match, own=None, said_before=None)`:
/// whether a bare "he"/"she" is the animal they keep rather than a person.
pub fn _may_be_an_animal(said: &str, m: &Match, own: &Value, said_before: &str) -> bool {
    if ["it", "them", "those", "they"].contains(&lower(m.name_str("pron")).as_str()) {
        return true;
    }
    if _names_a_relative(said) || PEOPLE_WORD.is_match(said) {
        return false;
    }
    if ANIMAL_DEATH.is_match(&_around(said, m.start())) {
        return true;
    }
    if said_before.is_empty() || PEOPLE_WORD.is_match(said_before) || !PETS.is_match(said_before) {
        return false;
    }
    !_only_pet(own).is_empty()
}

/// Python: bot/memory/facts.py:6368 `losses(text, own=None, said_before=None)`: the animals a line
/// says have died, in the shape extract() keeps pets.
pub fn losses(text: &str, own: &Value, said_before: &str) -> Vec<String> {
    let said = text;
    if !DIED.is_match(said) {
        return Vec::new();
    }
    let mut animals: Vec<String> = Vec::new();
    let others = _someone_elses(said);
    for m in PETS.finditer(said) {
        if !DIED.is_match(&_around(said, m.start())) {
            continue;
        }
        if others.iter().any(|(start, stop)| *start <= m.start() && m.start() < *stop) {
            continue;
        }
        if _not_their_animal(said, m.start(), m.end()) {
            continue;
        }
        let animal = lower(m.str(1));
        if !animals.contains(&animal) {
            animals.push(animal);
        }
    }
    if animals.is_empty() {
        if let Some(m) = PET_PRONOUN.finditer(said).into_iter().find(|m| DIED.is_match(&_around(said, m.start()))) {
            let mut pointed = _the_one_they_have(said, &m, own);
            if pointed.is_empty() && _may_be_an_animal(said, &m, own, said_before) {
                pointed = _only_pet(own);
            }
            for animal in pointed {
                if !animals.contains(&animal) {
                    animals.push(animal);
                }
            }
        }
    }
    if animals.is_empty() && own.truthy() {
        let said_low = lower(said);
        for pet in own.items() {
            let name = lower(&pet_field(pet, "name"));
            let animal = lower(&pet_field(pet, "animal"));
            if animal.is_empty() {
                continue;
            }
            for m in DIED.finditer(said) {
                if HEDGE.is_match(&_clause(said, m.start())) {
                    continue;
                }
                if !name.is_empty() && pt::len(&name) >= 3 && word_re(&name, 0).is_match(&said_low) {
                    if !animals.contains(&animal) {
                        animals.push(animal.clone());
                    }
                }
            }
        }
    }
    animals
}

/// Python: bot/memory/facts.py:6422 `_allergies(text)`: (the user's allergies, the others':
/// [{"who", "allergies"}]).
pub fn _allergies(text: &str) -> (Vec<String>, Vec<Value>) {
    let mut mine: Vec<String> = Vec::new();
    let mut others: Vec<Value> = Vec::new();
    for (k, pattern) in [&*ALLERGY, &*CANNOT_EAT, &*X_ALLERGY].into_iter().enumerate() {
        for m in pattern.finditer(text) {
            if k != 1 && denied_before(text, m.start()) {
                continue;
            }
            if k == 2 && NOT_AN_ALLERGEN.contains(&lower(m.str(1))) {
                continue;
            }
            let mut items = _allergy_items(m.str(1));
            if items.is_empty() {
                continue;
            }
            if k == 2 {
                items = items.into_iter().map(|i| ALLERGEN_PLURALS.get(&i).cloned().unwrap_or(i)).collect();
            }
            let (kind, who) = subject_at(text, m.start());
            if kind != "other" {
                for item in items {
                    if !mine.contains(&item) {
                        mine.push(item);
                    }
                }
                continue;
            }
            let person = _a_person(text, m.end(), &who);
            if !person.truthy() {
                continue;
            }
            let at = match others.iter().position(|seen| _same_person(seen, &person)) {
                Some(i) => i,
                None => {
                    let mut entry = person.clone();
                    entry.set_key("allergies", Value::list());
                    others.push(entry);
                    others.len() - 1
                }
            };
            extend_unique(others[at].setdefault("allergies", Value::list()), &items);
        }
    }
    (mine, others)
}

/// Python: bot/memory/facts.py:6477 `_month_age_after(text, end)`: true when the words after `end`
/// are an age in months and then the end of the fact. (`end` is a byte offset.)
pub fn _month_age_after(text: &str, end: usize) -> bool {
    let Some(m) = AGE_MONTHS_TAIL.match_(&text[end..]) else {
        return false;
    };
    AGE_TAIL.match_(&text[end + m.end()..]).is_some()
}

/// Python: bot/memory/facts.py:6542 `_the_word_then_a_verb(name, following)`: whether the word
/// after a name is a verb, so the word before it is the person's name.
pub fn _the_word_then_a_verb(name: &str, following: &str) -> bool {
    let name = lower(name);
    let following = lower(following);
    if name.is_empty() || following.is_empty() || VERB_FORMS.contains(&name) {
        return false;
    }
    if VERB_FORMS.contains(&following) {
        return true;
    }
    pt::len(&following) > 3 && following.ends_with("ed")
}

/// Python: bot/memory/facts.py:6605 `_may_be_a_name(word)`: whether a word standing where a name
/// stands can be one.
pub fn _may_be_a_name(word: &str) -> bool {
    let low = lower(word);
    _looks_like_a_name(word) && !NOT_A_NAME_MIDLINE.contains(&low) && !NOT_A_NAME_THERE.contains(&low)
}

static TITLED_NAME: LazyLock<Re> = LazyLock::new(|| {
    Re::new(&format!(r"(?:{})\s+[A-Za-z][A-Za-z'\-]{{1,19}}", data::string("bot.memory.facts.TITLE")), I)
});

/// Python: bot/memory/facts.py:6621 `a_titled_name(word)`: whether a word is a title with a name
/// after it: "mr patel", "dr. kaur", "mrs smith".
pub fn a_titled_name(word: &str) -> bool {
    TITLED_NAME.fullmatch(pt::strip(word)).is_some()
}

/// Python: bot/memory/facts.py:6627 `_looks_like_a_name(word)`: whether a word after a relation can
/// be their name: not a relation, not a grammar word.
pub fn _looks_like_a_name(word: &str) -> bool {
    let low = lower(word);
    if a_titled_name(word) {
        return true;
    }
    if TITLE_WORDS.contains(pt::strip_chars(&low, ".")) {
        return false;
    }
    if NAME_WORD.match_(word).is_none() || NOT_A_NAME.contains(&low) || RELATIONS.contains(&low) {
        return false;
    }
    if is_a_contraction(&low) {
        return false;
    }
    if word.chars().next().is_some_and(char::is_uppercase) || NAMES_THAT_END_LIKE_A_VERB.contains(&low) {
        return true;
    }
    !NOT_NAME_ENDING.iter().any(|e| low.ends_with(e.as_str()))
}

/// Python: bot/memory/facts.py:6643 `_name_after_a_title(name)`: whether a name that starts with a
/// title is a name at all.
pub fn _name_after_a_title(name: &str) -> bool {
    let words = pt::split(name);
    if words.is_empty() {
        return false;
    }
    let title = lower(pt::strip_chars(&words[0], "."));
    if !TITLE_WORDS.contains(&title) {
        return true;
    }
    if words.len() == 1 {
        return false;
    }
    let rest = lower(&words[1..].join(" "));
    if NAME_AFTER_TITLE.contains(&rest) {
        return true;
    }
    if NOT_A_NAME.contains(&rest) || NOT_NAMES.contains(&rest) || NOT_NAMES_EXTRA.contains(&rest) || _A_WORD_FOR_A_PERSON.contains(&rest) {
        return false;
    }
    !_not_a_name(&rest, false)
}

/// Python: bot/memory/facts.py:6668 `_the_name_without_a_possessive(text)`: {"tom's": "Tom",
/// "sarahs": "Sarah"} for the possessive runs in a line, in order.
pub fn _the_name_without_a_possessive(text: &str) -> Vec<(String, String)> {
    let mut stems: Vec<(String, String)> = Vec::new();
    for m in OTHER_NAME_POSSESSIVE.finditer(text) {
        let start = m.name_span("name").map(|s| s.0).unwrap_or(m.end());
        let raw = lower(&text[start..m.end()].replace('\u{2019}', "'"));
        let stem = m.name_str("name").to_string();
        if !raw.is_empty() && !stem.is_empty() {
            match stems.iter_mut().find(|(k, _)| *k == raw) {
                Some(slot) => slot.1 = stem,
                None => stems.push((raw, stem)),
            }
        }
    }
    stems
}

// ---------------------------------------------------------------------------------------------
// relations' names, children and their ages, states of other people, names offered and picked

fres!(
    KID_COUNT, KID_COUNT_AND, FIGURE_PAIR, FIGURE_WORD, KID_AGES, KID_AGES_AFTER_A_COMMA,
    ONES_BY_ONE, KID_ELSEWHERE, KID_ASKING, KID_NONE, KID_NONE_NOT_AFTER_ALL, COUNT_OF_THEM,
    COUNT_OF_THEM_ELSEWHERE, KID_AGES_ARE, KID_GROUP_AGES_ARE, KID_COUNT_AGES, KID_ADULTS_AGES,
    KID_AGES_NOT, KID_NAME_AGES, KID_NAME_AGE, FIGURE_AFTER, OTHER_STATE, OTHER_LIMIT, OWN_STATE,
    YEAR_OLD, OWN_AGE_BEFORE_IT, YEAR_OLD_ANIMAL, THING_AGED, A_PERSON_AGED, NAMED_THING_AGED,
    THING_AGED_QUALIFIED, AFTER_YEAR_OLD, LIST_BEFORE_IT, A_LIST_OF_AGES, GROUP_NAMED,
    THING_NAMED_BACK, PEOPLE_NAMED_ANYWHERE, _A_THING_SAID_IS, _A_THING_POINTED_AT, BARE_NAME,
);
fsets!(AGES_ONLY_WORD, NOT_A_NAME_HEDGE, PERSON_ELSEWHERE, PEOPLE_AGED, AFTER_YEAR_OLD_OKAY, NOT_A_BARE_NAME);
fstrs!(CHILDREN_WORDS, KIDS, KID_GROUP, KID_ONE_BY_ONE);
fmaps!(FIGURE_WORDS);

/// `KID_NUMBERS`: the number a count word says.
pub static KID_NUMBERS: LazyLock<HashMap<String, i64>> = LazyLock::new(|| {
    data::get("bot.memory.facts.KID_NUMBERS")
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_int()?)))
        .collect()
});

fn bundle_int(name: &str) -> i64 {
    data::get(&format!("bot.memory.facts.{name}")).as_int().unwrap_or(0)
}

static BREED_RE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.understanding.breeds.BREED_RE"));

/// The names in `found` for one person word (`[entry["who"] for entry in found]`).
fn has_who(found: &[Value], who: &str) -> bool {
    found.iter().any(|e| vget(e, "who").as_str() == Some(who))
}

/// "my sons" is the son: the plural is kept as the relation when the singular is one.
fn singular_relation(who: String) -> String {
    if who.ends_with('s') && RELATIONS.contains(&who[..who.len() - 1].to_string()) {
        who[..who.len() - 1].to_string()
    } else {
        who
    }
}

/// Python: bot/memory/facts.py:6684 `_a_relations_name(text)`: [{"who": "sister", "name": "meg"}]
/// for a name the line gives to someone in the family.
pub fn _a_relations_name(text: &str) -> Vec<Value> {
    let mut found: Vec<Value> = Vec::new();
    let text = stated_not_asked(text);
    if text.is_empty() {
        return found;
    }
    let text = text.as_str();
    let without_possessive = _the_name_without_a_possessive(text);
    let unpossessed = |name: &str| -> String {
        let key = lower(name);
        without_possessive.iter().find(|(k, _)| *k == key).map(|(_, v)| v.clone()).unwrap_or_else(|| name.to_string())
    };
    for pattern in [&*OTHER_CALLED, &*OTHER_NAME_IS, &*OTHER_NAMED_BARE] {
        for m in pattern.finditer(text) {
            let who = singular_relation(a_person_word(m.name_str("who")));
            let name = m.name_str("name");
            if !_looks_like_a_name(name) || !_name_after_a_title(name) {
                continue;
            }
            if has_who(&found, &who) {
                continue;
            }
            found.push(dict(vec![("who", who.into()), ("name", name.into())]));
        }
    }
    for m in OTHER_NAMED_BEFORE_VERB.finditer(text) {
        let who = singular_relation(a_person_word(m.name_str("who")));
        let name = unpossessed(m.name_str("name"));
        if !_the_word_then_a_verb(&name, m.name_str("verb")) {
            continue;
        }
        if !_may_be_a_name(&name) {
            continue;
        }
        if has_who(&found, &who) {
            continue;
        }
        found.push(dict(vec![("who", who.into()), ("name", name.into())]));
    }
    for pattern in [&*OTHER_NAMED_CAPITAL, &*OTHER_NAMED_MIDLINE] {
        for m in pattern.finditer(text) {
            let who = singular_relation(a_person_word(m.name_str("who")));
            let name = unpossessed(m.name_str("name"));
            if !_may_be_a_name(&name) {
                continue;
            }
            if has_who(&found, &who) {
                continue;
            }
            found.push(dict(vec![("who", who.into()), ("name", name.into())]));
        }
    }
    found
}

/// Python: bot/memory/facts.py:6775 `figures_as_digits(text)`: the line with every figure it
/// spells as a word written as a digit instead.
pub fn figures_as_digits(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let text = FIGURE_PAIR.sub_fn(text, |m| {
        let n = FIGURE_WORDS.get(&lower(m.str(1))).and_then(|s| py_int(s)).unwrap_or(0);
        (20 + n).to_string()
    });
    FIGURE_WORD.sub_fn(&text, |m| FIGURE_WORDS.get(&lower(m.as_str())).cloned().unwrap_or_default())
}

inline_re!(ONE_TO_TWO_DIGITS, r"\d{1,2}", 0);
inline_re!(ONE_TO_THREE_DIGITS, r"\d{1,3}", 0);

/// Python: bot/memory/facts.py:6796 `kid_ages(text)`.
///
/// CONTRACT NOTE: Python returns the ages as ints; the skeleton gave this `Vec<String>`. This
/// keeps the contract (the ages written as figures); `kid_ages_n` returns the numbers.
pub fn kid_ages(text: &str) -> Vec<String> {
    kid_ages_n(text).iter().map(|a| a.to_string()).collect()
}

/// `kid_ages(text)`: the ages the line gives the children, oldest reading only, or [].
pub fn kid_ages_n(text: &str) -> Vec<i64> {
    let digits = figures_as_digits(text);
    let found = KID_AGES.search(&digits).or_else(|| KID_AGES_AFTER_A_COMMA.search(&digits));
    if let Some(m) = found {
        return ONE_TO_TWO_DIGITS
            .findall_str(m.name_str("ages"))
            .iter()
            .filter_map(|n| py_int(n))
            .filter(|a| (0..=110).contains(a))
            .collect();
    }
    if let Some(counted) = _children_by_year_old(&digits) {
        return _ages_held(&counted);
    }
    let others = _other_ages(text);
    let grouped: Vec<&Value> = others.iter().filter(|e| KID_GROUP.contains(&lower(&str_or_empty(e.get("who"))))).collect();
    if !grouped.is_empty() && !grouped.iter().any(|e| vget(e, "name").truthy()) {
        let mut held: Vec<i64> = Vec::new();
        for entry in &grouped {
            for age in _ages_held(entry) {
                if !held.contains(&age) {
                    held.push(age);
                }
            }
        }
        if !held.is_empty() {
            return held;
        }
    }
    let each: Vec<&Value> = others.iter().filter(|e| KID_ONE_BY_ONE.contains(&lower(&str_or_empty(e.get("who"))))).collect();
    if each.len() >= 2 && !each.iter().any(|e| vget(e, "name").truthy()) {
        let mut held: Vec<i64> = Vec::new();
        for entry in &each {
            for age in _ages_held(entry) {
                if !held.contains(&age) {
                    held.push(age);
                }
            }
        }
        if !held.is_empty() {
            return held;
        }
    }
    Vec::new()
}

/// Python: bot/memory/facts.py:6849 `the_ones_are_given_one_at_a_time(text)`: whether the line
/// hands the ages out one by one ("ones 7 and ones 4", "the older one is 7").
pub fn the_ones_are_given_one_at_a_time(text: &str) -> bool {
    ONES_BY_ONE.is_match(&squash(&lower(text).replace('\'', "")))
}

/// Python: bot/memory/facts.py:6856 `ages_said_one_by_one(text, count)`.
///
/// CONTRACT NOTE: Python returns the ages as ints; the skeleton gave this `Vec<String>`. This
/// keeps the contract (the ages written as figures); `ages_said_one_by_one_n` returns the numbers.
pub fn ages_said_one_by_one(text: &str, count: i64) -> Vec<String> {
    ages_said_one_by_one_n(text, count).iter().map(|a| a.to_string()).collect()
}

/// `ages_said_one_by_one(text, count)`: the ages a line gives a group the store counts, or [].
pub fn ages_said_one_by_one_n(text: &str, count: i64) -> Vec<i64> {
    if count < 2 {
        return Vec::new();
    }
    let flat = squash(&lower(text).replace('\'', ""));
    if flat.is_empty() || pt::split(&flat).len() > 12 {
        return Vec::new();
    }
    let flat = figures_as_digits(&ONES_BY_ONE.sub(" & ", &flat));
    let mut ages = Vec::new();
    for word in NOT_ALNUM_RUN.split(&flat) {
        if word.is_empty() {
            continue;
        }
        if pt::is_digit(&word) {
            let age = py_int(&word).unwrap_or(-1);
            if !(0 < age && age <= 120) {
                return Vec::new();
            }
            ages.push(age);
        } else if !AGES_ONLY_WORD.contains(&word) {
            return Vec::new();
        }
    }
    if ages.len() as i64 == count {
        ages
    } else {
        Vec::new()
    }
}

/// Python: bot/memory/facts.py:6903 `kid_count(text)`: "i have two kids" -> 2, "i dont have any
/// kids" -> 0: the number a line states, or None.
pub fn kid_count(text: &str) -> Option<i64> {
    if KID_ASKING.match_(text).is_some() {
        return None;
    }
    if let Some(none) = KID_NONE.search(text) {
        if !KID_NONE_NOT_AFTER_ALL.is_match(&text[none.end()..]) {
            return Some(0);
        }
    }
    let Some(m) = KID_COUNT.search(text).or_else(|| KID_COUNT_AND.search(text)) else {
        let ages = kid_ages_n(text);
        if !ages.is_empty() {
            return Some(ages.len() as i64);
        }
        if let Some(counted) = _children_by_year_old(&figures_as_digits(text)) {
            return Some(_ages_held(&counted).len() as i64);
        }
        let others = _other_ages(text);
        let grouped: Vec<&Value> = others.iter().filter(|e| KID_GROUP.contains(&lower(&str_or_empty(e.get("who"))))).collect();
        if !grouped.is_empty() && !grouped.iter().any(|e| vget(e, "name").truthy()) {
            let mut held: Vec<i64> = Vec::new();
            for entry in &grouped {
                for age in _ages_held(entry) {
                    if !held.contains(&age) {
                        held.push(age);
                    }
                }
            }
            if !held.is_empty() {
                return Some(held.len() as i64);
            }
        }
        let each: Vec<&Value> = others
            .iter()
            .filter(|e| {
                KID_ONE_BY_ONE.contains(&lower(&str_or_empty(e.get("who"))))
                    && (vget(e, "age").truthy() || !_ages_held(e).is_empty())
            })
            .collect();
        if each.len() >= 2 && !each.iter().any(|e| vget(e, "name").truthy()) {
            return Some(each.len() as i64);
        }
        return None;
    };
    if KID_ELSEWHERE.is_match(&text[m.end()..]) {
        return None;
    }
    let said = lower(m.name("n").filter(|s| !s.is_empty()).or_else(|| m.name("m").filter(|s| !s.is_empty())).unwrap_or(""));
    if said.starts_with("a couple") {
        return Some(2);
    }
    if pt::is_digit(&said) {
        return py_int(&said);
    }
    KID_NUMBERS.get(&said).copied()
}

/// Python: bot/memory/facts.py:6973 `count_of_the_people(text, memory)`: "there are three of them"
/// -> 3, while the group in hand is the children; None otherwise.
pub fn count_of_the_people(text: &str, memory: &Memory) -> Option<i64> {
    if KID_ASKING.match_(text).is_some() {
        return None;
    }
    let m = COUNT_OF_THEM.search(text)?;
    let said = lower(m.name_str("n"));
    let number = if pt::is_digit(&said) { py_int(&said) } else { KID_NUMBERS.get(&said).copied() };
    let number = number.filter(|n| *n != 0)?;
    if !(1 < number && number <= 12) {
        return None;
    }
    if COUNT_OF_THEM_ELSEWHERE.match_(&text[m.end()..]).is_some() {
        return None;
    }
    if !_the_group_in_hand_is_the_children(memory) {
        return None;
    }
    Some(number)
}

inline_re!(NOT_A_WORD_RUN, r"[^a-z']+", 0);

/// Python: bot/memory/facts.py:6998 `_the_group_in_hand_is_the_children(memory)`: whether the
/// children are what the chat was last on, so a bare count is theirs.
pub fn _the_group_in_hand_is_the_children(memory: &Memory) -> bool {
    let said = memory.turns.last().map(|t| low_squash(&t.user)).unwrap_or_default();
    let words: HashSet<String> = NOT_A_WORD_RUN.split(&said).into_iter().collect();
    if CHILDREN_WORDS.iter().any(|w| words.contains(w)) {
        return true;
    }
    if let Some(user) = memory.get("user") {
        if user.is_dict() && vget(user, "kids").truthy() {
            let animals = ["cats", "cat", "dogs", "dog", "pets", "pet", "fish", "birds"];
            return !animals.iter().any(|a| words.contains(*a));
        }
    }
    false
}

/// Python: bot/memory/facts.py:7095 `_kids_named_with_ages(text)`: [{"who", "name", "age"}] for the
/// children given with a name and an age, or [].
pub fn _kids_named_with_ages(text: &str) -> Vec<Value> {
    let mut found = Vec::new();
    for m in KID_NAME_AGES.finditer(text) {
        let rest = m.name_str("rest");
        let rest_at = m.name_span("rest").map(|s| s.0).unwrap_or(m.start());
        if KID_AGES_NOT.is_match(&text[..rest_at]) {
            continue;
        }
        let pairs = KID_NAME_AGE.finditer(rest);
        if pairs.is_empty() {
            continue;
        }
        if ONE_TO_THREE_DIGITS.findall_str(rest).len() != pairs.len() || FIGURE_AFTER.is_match(&text[m.end()..]) {
            continue;
        }
        for pair in &pairs {
            let name = pair.name_str("name");
            let age = int_of(pair.name_str("age"));
            let low = lower(name);
            if NOT_A_NAME.contains(&low)
                || KIDS.contains(&low)
                || NOT_A_NAME_HEDGE.contains(&low)
                || NOT_A_NAME_MIDDLE.contains(&low)
                || NOT_A_NAME_THING.contains(&low)
                || NOT_A_NAME_STATE.contains(&low)
                || MONTHS.contains(&low)
                || DAYS.contains(&low)
                || !(0..=120).contains(&age)
            {
                continue;
            }
            if age <= 2 && !name.chars().next().is_some_and(char::is_alphabetic) {
                continue;
            }
            found.push(dict(vec![("who", lower(m.name_str("who")).into()), ("name", name.into()), ("age", age.into())]));
        }
    }
    found
}

/// Python: bot/memory/facts.py:7127 `_the_same_person(one, other)`: whether two readings of the
/// people in a line are about the same person.
pub fn _the_same_person(one: &Value, other: &Value) -> bool {
    if str_or_empty(one.get("who")) != str_or_empty(other.get("who")) {
        return false;
    }
    lower(&str_or_empty(one.get("name"))) == lower(&str_or_empty(other.get("name")))
}

inline_re!(OWN_STATE_HEDGED, r"\b(?:was|were|thought|maybe|might|may|hope|hoping|could)\b", 0);

/// Python: bot/memory/facts.py:7184 `_own_states(text)`: {"pregnant": True, "weeks": 12} for the
/// person themselves, or {}.
pub fn _own_states(text: &str) -> Value {
    let digits = figures_as_digits(text);
    for m in OWN_STATE.finditer(&digits) {
        let said = lower(m.as_str());
        if OWN_STATE_HEDGED.is_match(&said) {
            continue;
        }
        let mut found = dict(vec![("pregnant", true.into())]);
        if let Some(weeks) = m.name("weeks") {
            let w = int_of(weeks);
            if 1 <= w && w <= bundle_int("OWN_PREGNANCY_WEEKS_MAX") {
                found.set_key("weeks", w);
            }
        }
        return found;
    }
    Value::dict()
}

/// Python: bot/memory/facts.py:7206 `_other_states(text)`: [{"who", "pregnant"|"breastfeeding":
/// True}] and [{"who", "drinks": False}] for these people.
pub fn _other_states(text: &str) -> Vec<Value> {
    let mut found = Vec::new();
    for m in OTHER_STATE.finditer(text) {
        let tense = lower(m.name_str("tense"));
        if tense == "was" || tense == "were" {
            continue;
        }
        let who = squash(&lower(m.name_str("who")));
        let state = lower(m.name_str("state"));
        if PREGNANCY_STATES.contains(&state) {
            let mut entry = dict(vec![("who", who.into()), ("pregnant", true.into())]);
            if let Some(weeks) = m.name("weeks").filter(|w| !w.is_empty()) {
                let w = int_of(weeks);
                if (1..=45).contains(&w) {
                    entry.set_key("weeks", w);
                }
            }
            found.push(entry);
        } else {
            found.push(dict(vec![("who", who.into()), ("breastfeeding", true.into())]));
        }
    }
    for m in OTHER_LIMIT.finditer(text) {
        let who = squash(&lower(m.name_str("who")));
        found.push(dict(vec![("who", who.into()), ("drinks", false.into())]));
    }
    found
}

inline_re!(LOWER_WORDS, r"[a-z']+", 0);

/// The last word of a noun group, lowered (`noun.split()[-1].lower()`).
fn last_word_lower(noun: &str) -> String {
    lower(pt::split_ref(noun).last().copied().unwrap_or(""))
}

/// Python: bot/memory/facts.py:7332 `_the_years_are_a_things(text)`: whether every age in years on
/// the line belongs to a thing rather than to a child.
pub fn _the_years_are_a_things(text: &str) -> bool {
    let figures = YEAR_OLD.finditer(text);
    if figures.is_empty() {
        return false;
    }
    for m in &figures {
        let before = &text[back_chars(text, m.start(), 80)..m.start()];
        let words = LOWER_WORDS.findall_str(&lower(before));
        if words.iter().any(|w| PEOPLE_AGED.contains(w)) {
            return false;
        }
        if let Some(named) = _A_THING_SAID_IS.search(before) {
            if !PEOPLE_AGED.contains(&last_word_lower(named.name_str("noun"))) {
                continue;
            }
        }
        if _A_THING_POINTED_AT.is_match(before) {
            continue;
        }
        return false;
    }
    true
}

static MY_A_PERSON: LazyLock<Re> =
    LazyLock::new(|| Re::new(&format!(r"\bmy\s+{}\b", data::string("bot.memory.facts._A_PERSON")), I));

/// Python: bot/memory/facts.py:7356 `_children_by_year_old(text)`: the children a line counts in
/// years: {"who": "kids", "ages": [...]}, or None.
pub fn _children_by_year_old(text: &str) -> Option<Value> {
    if !YEAR_OLD.is_match(text) {
        return None;
    }
    if _the_years_are_a_things(text) {
        return None;
    }
    if !GROUP_NAMED.is_match(text) && MY_A_PERSON.is_match(text) {
        return None;
    }
    let group_named = GROUP_NAMED.is_match(text);
    let figures = YEAR_OLD.finditer(text);
    if group_named && figures.len() > 1 {
        return None;
    }
    let mut ages: Vec<i64> = Vec::new();
    for m in &figures {
        let age = int_of(m.name_str("age"));
        if !(0..=120).contains(&age) {
            continue;
        }
        if OWN_AGE_BEFORE_IT.is_match(&text[..m.start()]) {
            continue;
        }
        if !group_named && A_PERSON_AGED.is_match(&text[back_chars(text, m.start(), 40)..m.start()]) {
            continue;
        }
        if YEAR_OLD_ANIMAL.is_match(&text[back_chars(text, m.start(), 24)..m.start()]) {
            continue;
        }
        let before = &text[back_chars(text, m.start(), 40)..m.start()];
        if BREED_RE.is_match(before) {
            continue;
        }
        if !PEOPLE_NAMED_ANYWHERE.is_match(text) {
            let window = &text[back_chars(text, m.start(), bundle_int("SUBJECT_REACH") as usize)..m.start()];
            let nearest = THING_NAMED_BACK.finditer(window).into_iter().last();
            if let Some(nearest) = &nearest {
                let noun = pt::split(nearest.name_str("noun"));
                if noun.is_empty() || !PEOPLE_AGED.contains(&lower(noun.last().unwrap())) {
                    continue;
                }
            }
        }
        if group_named && LIST_BEFORE_IT.is_match(before) {
            continue;
        }
        if THING_AGED.is_match(before) {
            continue;
        }
        if let Some(qualified) = THING_AGED_QUALIFIED.search(before) {
            let thing = lower(qualified.name_str("noun"));
            if !PEOPLE_AGED.contains(&thing) && !PERSON_ELSEWHERE.contains(&thing) {
                continue;
            }
        }
        if let Some(named) = NAMED_THING_AGED.search(before) {
            if !PEOPLE_AGED.contains(&last_word_lower(named.name_str("noun"))) {
                continue;
            }
        }
        if let Some(after) = AFTER_YEAR_OLD.match_(&text[m.end()..]) {
            if !group_named && !AFTER_YEAR_OLD_OKAY.contains(&lower(after.name_str("word"))) {
                continue;
            }
        }
        if age >= bundle_int("CHILD_AGE_UNDER") && !group_named {
            continue;
        }
        ages.push(age);
    }
    if ages.is_empty() {
        return None;
    }
    Some(if ages.len() > 1 {
        dict(vec![("who", "kids".into()), ("ages", Value::from(ages.into_iter().map(Value::from).collect::<Vec<_>>()))])
    } else {
        dict(vec![("who", "kids".into()), ("age", ages[0].into())])
    })
}

/// Python: bot/memory/facts.py:7436 `_ages_held(entry)`: every age an entry holds, the list and the
/// lone one together, in the order given.
pub fn _ages_held(entry: &Value) -> Vec<i64> {
    let mut held: Vec<i64> = vget(entry, "ages").items().iter().filter_map(Value::as_int).collect();
    if let Some(age) = vget(entry, "age").as_int() {
        if !held.contains(&age) {
            held.push(age);
        }
    }
    held
}

fn ints(values: &[i64]) -> Value {
    Value::from(values.iter().map(|v| Value::from(*v)).collect::<Vec<_>>())
}

/// Python: bot/memory/facts.py:7444 `_keep_the_ages(found, entry)`: add a reading about a group of
/// children, or fold it into the group already read.
pub fn _keep_the_ages(found: &mut Vec<Value>, entry: &Value) {
    let who = vget(entry, "who").clone();
    let Some(at) = found.iter().position(|one| *vget(one, "who") == who) else {
        found.push(entry.clone());
        return;
    };
    let said = _ages_held(entry);
    let held = _ages_held(&found[at]);
    if !held.is_empty() && !said.is_empty() && held.len() >= said.len() && said.iter().all(|a| held.contains(a)) {
        return;
    }
    let known = &mut found[at];
    known.del_key("age");
    let ages: Vec<i64> = if said.len() > 1 {
        said
    } else {
        let mut all = held.clone();
        all.extend(said.into_iter().filter(|a| !held.contains(a)));
        all
    };
    known.set_key("ages", ints(&ages));
    if ages.len() == 1 {
        known.del_key("ages");
        known.set_key("age", ages[0]);
    }
}

struct OtherAgePatterns {
    pattern: Re,
    pair: Re,
    named: Re,
}

static OTHER_AGE_PATTERNS: LazyLock<OtherAgePatterns> = LazyLock::new(|| {
    let mut rel: Vec<String> = RELATIONS.clone();
    rel.sort_by(|a, b| pt::len(b).cmp(&pt::len(a)));
    let relations = rel.join("|");
    let who_one = format!(r"(?:{relations})(?:\s+in\s+laws?)?");
    let group_two = "grandparents|grandchildren";
    OtherAgePatterns {
        pattern: Re::new(
            &format!(
                r"\bmy\s+(?:(?P<many>(?:{who_one}|{group_two})s?)\s+are|(?P<one>{who_one})\s*(?:'s|is|was))\s+(?P<age>\d{{1,3}})\b"
            ),
            I,
        ),
        pair: Re::new(
            &format!(
                r"\bmy\s+(?P<a>{who_one})\s+and\s+(?:my\s+)?(?P<b>{who_one})s?\s+(?:are|is)\s+(?P<ages>\d{{1,3}}(?:\s*(?:,|and)\s*\d{{1,3}})+)"
            ),
            I,
        ),
        named: Re::new(
            &format!(r"\bmy\s+(?P<who>{relations})\s+(?P<name>[A-Za-z][a-z'-]{{1,19}})\s*(?:'s|is|was)\s+(?P<age>\d{{1,3}})\b"),
            I,
        ),
    }
});

/// Python: bot/memory/facts.py:7466 `_other_ages(text)`: [{"who", "age"}] for the people the line
/// gives an age for, or [].
pub fn _other_ages(text: &str) -> Vec<Value> {
    let text = figures_as_digits(text);
    let mut found: Vec<Value> = Vec::new();
    let text = stated_not_asked(&text);
    if text.is_empty() {
        return found;
    }
    let text = text.as_str();
    let p = &*OTHER_AGE_PATTERNS;
    for m in p.pair.finditer(text) {
        let ages: Vec<i64> = ONE_TO_THREE_DIGITS.findall_str(m.name_str("ages")).iter().map(|n| int_of(n)).collect();
        if !ages.iter().all(|a| (0..=120).contains(a)) || ages.len() < 2 {
            continue;
        }
        for (who, age) in [lower(m.name_str("a")), lower(m.name_str("b"))].into_iter().zip(ages) {
            match found.iter().position(|one| vget(one, "who").as_str() == Some(who.as_str())) {
                None => found.push(dict(vec![("who", who.into()), ("age", age.into())])),
                Some(i) => found[i].set_key("age", age),
            }
        }
    }
    for kid_pattern in [&*KID_GROUP_AGES_ARE, &*KID_COUNT_AGES, &*KID_ADULTS_AGES] {
        for m in kid_pattern.finditer(text) {
            let ages: Vec<i64> = ONE_TO_THREE_DIGITS.findall_str(m.name_str("ages")).iter().map(|n| int_of(n)).collect();
            if ages.is_empty() || !ages.iter().all(|a| (0..=120).contains(a)) {
                continue;
            }
            let ages_at = m.name_span("ages").map(|s| s.0).unwrap_or(m.start());
            if KID_AGES_NOT.is_match(&text[..ages_at]) {
                continue;
            }
            if AGE_TAIL.match_(&text[m.end()..]).is_none() {
                continue;
            }
            let mut who = lower(m.name_str("who"));
            if who.is_empty() {
                who = found
                    .iter()
                    .map(|one| str_or_empty(one.get("who")))
                    .find(|w| KID_GROUP.contains(&lower(w)))
                    .unwrap_or_else(|| "kids".into());
            }
            let entry = if ages.len() > 1 {
                dict(vec![("who", who.into()), ("ages", ints(&ages))])
            } else {
                dict(vec![("who", who.into()), ("age", ages[0].into())])
            };
            _keep_the_ages(&mut found, &entry);
        }
    }
    for m in CHAINED_RELATION_AGE.finditer(text) {
        let age = int_of(m.name_str("age"));
        let age_end = m.name_span("age").map(|s| s.1).unwrap_or(m.end());
        if !(0..=120).contains(&age) || _month_age_after(text, age_end) {
            continue;
        }
        let first = POSSESSIVE_END.sub("", &lower(m.name_str("first")));
        let mut entry = dict(vec![("who", format!("{first}'s {}", lower(m.name_str("second"))).into()), ("age", age.into())]);
        if let Some(name) = m.name("name").filter(|n| !n.is_empty()) {
            if !_A_WORD_FOR_A_PERSON.contains(&lower(name)) {
                entry.set_key("name", name);
            }
        }
        found.push(entry);
    }
    for m in RELATION_WHOS_AGE.finditer(text) {
        let age = int_of(m.name_str("age"));
        if !(0..=120).contains(&age) || _month_age_after(text, m.end()) {
            continue;
        }
        found.push(dict(vec![("who", lower(m.name_str("who")).into()), ("age", age.into())]));
    }
    for m in AGE_TO_COME.finditer(text) {
        let (kind, mut who) = subject_at(text, m.end());
        if kind != "other" || (!who.is_empty() && !RELATIONS.contains(&who)) {
            continue;
        }
        let age = int_of(m.str(1));
        if !(0..=120).contains(&age) {
            continue;
        }
        let turns = low_squash(m.str(2));
        if who.is_empty() {
            who = _relation_before(text, m.start());
        }
        if !who.is_empty() {
            if let Some(i) = found.iter().position(|one| vget(one, "who").as_str() == Some(who.as_str())) {
                if !vget(&found[i], "turns").truthy() {
                    found[i].set_key("turns", turns.as_str());
                    if *vget(&found[i], "age") != Value::from(age) {
                        found[i].set_key("to_come", age);
                    }
                    continue;
                }
            }
            found.push(dict(vec![("who", who.into()), ("age", age.into()), ("turns", turns.into())]));
        } else {
            let pronoun = _the_pronoun(text, m.end());
            if !pronoun.is_empty() {
                let person = _the_person_the_pronoun_is(text, m.start(), &pronoun);
                if !person.is_empty() {
                    found.push(dict(vec![("who", person.into()), ("age", age.into()), ("turns", turns.into())]));
                } else {
                    found.push(dict(vec![("pronoun", pronoun.into()), ("age", age.into()), ("turns", turns.into())]));
                }
            }
        }
    }
    for m in p.named.finditer(text) {
        let name = m.name_str("name");
        if !_may_be_a_name(name) {
            continue;
        }
        if AGE_TAIL.match_(&text[m.end()..]).is_none() {
            continue;
        }
        let age = int_of(m.name_str("age"));
        if !(0..=120).contains(&age) {
            continue;
        }
        let who = singular_relation(lower(m.name_str("who")));
        let kept = tidy(name);
        match _the_person(&found, &who, &kept) {
            None => found.push(dict(vec![("who", who.into()), ("name", kept.into()), ("age", age.into())])),
            Some(i) => {
                found[i].setdefault("name", kept.as_str());
                found[i].set_key("age", age);
            }
        }
    }
    for m in p.pattern.finditer(text) {
        let plural = m.name("many").filter(|s| !s.is_empty());
        if let Some(plural) = plural {
            let age_at = m.name_span("age").map(|s| s.0).unwrap_or(m.start());
            if let Some(listed) = match_at(&A_LIST_OF_AGES, text, age_at) {
                let ages: Vec<i64> = ONE_TO_THREE_DIGITS.findall_str(listed.name_str("ages")).iter().map(|n| int_of(n)).collect();
                if ages.len() > 1 && ages.iter().all(|a| (0..=120).contains(a)) && AGE_TAIL.match_(&text[listed.end()..]).is_some() {
                    _keep_the_ages(&mut found, &dict(vec![("who", squash(&lower(plural)).into()), ("ages", ints(&ages))]));
                    continue;
                }
            }
        }
        let rest = &text[m.end()..];
        if AGE_TAIL.match_(rest).is_none() {
            continue;
        }
        let age = int_of(m.name_str("age"));
        if !(0..=120).contains(&age) {
            continue;
        }
        let who = plural.or_else(|| m.name("one").filter(|s| !s.is_empty())).unwrap_or("");
        let who = IN_LAWS_END.sub("in law", &squash(&lower(who)));
        let who = singular_relation(who);
        match _the_person(&found, &who, "") {
            None => found.push(dict(vec![("who", who.into()), ("age", age.into())])),
            Some(i) => {
                let entry = &mut found[i];
                if vget(entry, "turns").truthy() && !vget(entry, "age").is_none() && *vget(entry, "age") != Value::from(age) {
                    let previous = vget(entry, "age").clone();
                    entry.set_key("to_come", previous);
                }
                entry.set_key("age", age);
            }
        }
    }
    if let Some(counted) = _children_by_year_old(text) {
        _keep_the_ages(&mut found, &counted);
    }
    for entry in _kids_named_with_ages(text) {
        if !found.iter().any(|one| *vget(one, "who") == *vget(&entry, "who") && *vget(one, "name") == *vget(&entry, "name")) {
            found.push(entry);
        }
    }
    found
}

static RELATION_BEFORE: LazyLock<Re> = LazyLock::new(|| Re::new(&format!(r"\bmy\s+({})\b", RELATIONS.join("|")), I));

/// Python: bot/memory/facts.py:7659 `_relation_before(text, pos)`: the relation the line named most
/// recently before `pos`, or "" for none. (`pos` is a byte offset.)
pub fn _relation_before(text: &str, pos: usize) -> String {
    let mut who = String::new();
    for m in RELATION_BEFORE.finditer(&text[..pos]) {
        who = lower(m.str(1));
    }
    who
}

/// Python: bot/memory/facts.py:7667 `pet_appositive(text, start=0)`: the name given for the animal
/// at `start` as an aside ("a rescue dog, biscuit"), or None. (`start` is a byte offset.)
pub fn pet_appositive(text: &str, start: i64) -> Option<String> {
    let start = boundary_at(text, start.max(0) as usize);
    let later: Vec<usize> = PETS.finditer(text).iter().map(|m| m.start()).filter(|p| *p >= start).collect();
    let limit = if later.len() > 1 { later[1] } else { text.len() };
    let span = &text[start..limit.max(start)];
    let m = PET_APPOSITIVE.search(span)?;
    if !_names_it_aside(&m, span) {
        return None;
    }
    Some(m.str(1).to_string())
}

inline_re!(NAME_IN_A_LIST, r"[A-Za-z][a-z'-]{1,20}", 0);

/// Python: bot/memory/facts.py:7687 `names_offered(memory)`: the names the bot itself put in front
/// of them on the turn before, and what they are for: {"animal", "names"}, or None.
pub fn names_offered(memory: &Memory) -> Option<Value> {
    let last = memory.turns.last()?;
    let m = NAMES_OFFERED.search(&last.bot)?;
    let names: Vec<String> = NAME_IN_A_LIST.findall_str(m.name_str("list")).into_iter().filter(|w| lower(w) != "or" && lower(w) != "and").collect();
    if names.len() < 2 {
        return None;
    }
    Some(dict(vec![("animal", lower(m.name_str("animal")).into()), ("names", str_list(&names))]))
}

/// Python: bot/memory/facts.py:7707 `a_name_picked(text, memory)`: the name given as the answer to
/// the bot's own question or list, or None.
pub fn a_name_picked(text: &str, memory: &Memory) -> Option<String> {
    let word = bare_name(text)?;
    let last = memory.turns.last()?;
    let asked = last.bot.as_str();
    let asked_for_one = pt::rstrip_chars(pt::rstrip(asked), "\"").ends_with('?') && PET_NAME_ASKED.is_match(asked);
    let theirs = NAME_PICK_ASK.is_match(&last.user);
    if !(asked_for_one || theirs || names_offered(memory).is_some()) {
        return None;
    }
    Some(word)
}

inline_re!(A_VOWEL, r"[aeiouy]", 0);
inline_re!(ONLY_DIGITS, r"\d+", 0);

/// Python: bot/memory/facts.py:7736 `bare_name(text)`: the one word a line is, when the line is
/// nothing but that word: "biscuit", "milo"; None otherwise.
pub fn bare_name(text: &str) -> Option<String> {
    let m = BARE_NAME.match_(pt::strip(text))?;
    let word = m.str(1).to_string();
    let low = lower(&word);
    if is_addressing(&word) {
        return None;
    }
    if NOT_A_BARE_NAME.contains(&low) || NOT_A_PET_NAME.contains(&low) || PET_ANIMALS.contains(&low) {
        return None;
    }
    if !A_VOWEL.is_match(&low) || ONLY_DIGITS.fullmatch(&low).is_some() {
        return None;
    }
    Some(word)
}

/// Python: bot/memory/facts.py:8916 `is_addressing(word)`: whether a word is how someone addresses
/// a person ("mate", "love"), not a pet's name.
pub fn is_addressing(word: &str) -> bool {
    ADDRESSING.contains(&lower(word))
}

fsets!(ADDRESSING);

/// Python: bot/memory/facts.py:8921 `_names_it_aside(match, text)`: whether the word after the
/// animal in commas names it.
pub fn _names_it_aside(m: &Match, text: &str) -> bool {
    let word = m.str(1);
    if !_not_a_name(word, true) {
        return true;
    }
    word.chars().next().is_some_and(char::is_uppercase) && A_LETTER_ANY.is_match(&text[m.end()..])
}

inline_re!(A_LETTER_ANY, r"[A-Za-z]", 0);

// ---------------------------------------------------------------------------------------------
// foods ruled out, other people's diets, conditions and jobs, pet names in passing

fres!(OTHER_NO_DRINK, OTHER_CONDITION, OTHER_JOB, OTHER_JOB_AS, OTHER_JOB_NAMED, NOT_DIET, AFTER_A_PET);
fsets!(DIET_STOP, PET_COMPOUND);
fmaps!(CONDITION_AS_A_WORD);

inline_re!(NO_A_FOOD, r"^\s*no\s+([a-z][a-z ]{1,20}?)\s*$", I);

/// The items of a value that is a list or a tuple, or the value itself as one item.
fn as_items(v: Option<&Value>) -> Vec<Value> {
    match v {
        Some(v) if v.truthy() => match v {
            Value::List(_) | Value::Tuple(_) => v.items().to_vec(),
            other => vec![other.clone()],
        },
        _ => Vec::new(),
    }
}

/// Python: bot/memory/facts.py:7772 `foods_ruled_out(found)`: every food a line rules out, for the
/// person or for somebody they named, or [].
pub fn foods_ruled_out(found: &Value) -> Vec<String> {
    if !found.is_dict() {
        return Vec::new();
    }
    let mut foods: Vec<String> = Vec::new();
    if let Some(m) = NO_A_FOOD.match_(&str_or_empty(found.get("diet"))) {
        foods.push(m.str(1).to_string());
    }
    for food in as_items(found.get("allergies")) {
        let s = pt::strip(&value_str(&food)).to_string();
        if !s.is_empty() {
            foods.push(s);
        }
    }
    for entry in found.get("others").map(|o| o.items().to_vec()).unwrap_or_default() {
        if !entry.is_dict() {
            continue;
        }
        if let Some(m) = NO_A_FOOD.match_(&str_or_empty(entry.get("diet"))) {
            foods.push(m.str(1).to_string());
        }
        for food in as_items(entry.get("allergies")) {
            let s = pt::strip(&value_str(&food)).to_string();
            if !s.is_empty() {
                foods.push(s);
            }
        }
    }
    foods
}

/// The entry in `found` for this person, made (a copy of `person`) when there is none.
fn entry_for(found: &mut Vec<Value>, person: &Value) -> usize {
    match found.iter().position(|seen| _same_person(seen, person)) {
        Some(i) => i,
        None => {
            found.push(person.clone());
            found.len() - 1
        }
    }
}

/// Python: bot/memory/facts.py:7825 `_other_no_drink(text)`: who else in the line does not drink,
/// as the person the store keeps: [{...}], or [].
pub fn _other_no_drink(text: &str) -> Vec<Value> {
    let mut found = Vec::new();
    let text = stated_not_asked(text);
    if text.is_empty() {
        return found;
    }
    let text = text.as_str();
    for m in OTHER_NO_DRINK.finditer(text) {
        if NOT_DIET.is_match(m.as_str()) {
            continue;
        }
        let subj_at = m.name_span("subj").map(|s| s.0).unwrap_or(m.start());
        let (mut kind, mut who) = subject_at(text, subj_at);
        if kind.is_empty() {
            (kind, who) = _somebody(m.name_str("subj"));
        }
        if kind != "other" || ANIMALS.contains(&who) || who == "i" || who == "we" {
            continue;
        }
        let person = _a_person(text, m.end(), &who);
        if !person.truthy() {
            continue;
        }
        let i = entry_for(&mut found, &person);
        found[i].setdefault("diet", "no alcohol");
    }
    found
}

inline_re!(SOMEBODYS, r"(?:my|his|her|their|our)\s+(\w+)", I);

/// Python: bot/memory/facts.py:7852 `_somebody(subject_text)`: who a subject phrase names:
/// ("other", "mum") for "my mum", ("other", "") for "shes".
pub fn _somebody(subject_text: &str) -> (String, String) {
    if let Some(named) = SOMEBODYS.match_(subject_text) {
        let word = lower(named.str(1));
        let who = SPOKEN_RELATION.get(&word).cloned().unwrap_or(word);
        return ("other".into(), who);
    }
    ("other".into(), String::new())
}

/// Python: bot/memory/facts.py:7861 `_other_diets(text)`: what someone else in the line eats:
/// [{"who": "mum", "diet": "no meat"}].
pub fn _other_diets(text: &str) -> Vec<Value> {
    let mut found: Vec<Value> = Vec::new();
    let text = stated_not_asked(text);
    if text.is_empty() {
        return found;
    }
    let text = text.as_str();
    let claimed: Vec<(usize, usize)> = [&*ALLERGY, &*CANNOT_EAT, &*X_ALLERGY]
        .iter()
        .flat_map(|p| p.finditer(text).iter().map(|m| (m.start(), m.end())).collect::<Vec<_>>())
        .collect();
    for m in OTHER_DIET.finditer(text) {
        if claimed.iter().any(|(start, end)| *start < m.end() && m.start() < *end) {
            continue;
        }
        let word = m.name("word").filter(|s| !s.is_empty());
        let food = m.name("food").filter(|s| !s.is_empty());
        let free = m.name("free").filter(|s| !s.is_empty());
        let at = if word.is_some() {
            m.name_span("word").unwrap().0
        } else if free.is_some() {
            m.name_span("free").unwrap().0
        } else {
            m.name_span("food").map(|s| s.0).unwrap_or(m.start())
        };
        let (mut kind, mut who) = subject_at(text, at);
        if kind.is_empty() {
            (kind, who) = _somebody(m.name_str("subj"));
        }
        if kind != "other" || ANIMALS.contains(&who) {
            continue;
        }
        if NOT_DIET.is_match(m.as_str()) {
            continue;
        }
        let diet = if let Some(said) = free.or(word) {
            pt::split(&lower(said).replace('-', " ")).join("-")
        } else {
            let food = _trim_filler(&_finish(food.unwrap_or("")));
            let words = pt::split(&food);
            if food.is_empty() || words.iter().any(|w| DIET_STOP.contains(w)) {
                continue;
            }
            if !words.iter().any(|w| DIET_GROUP.contains(w)) {
                continue;
            }
            format!("no {food}")
        };
        let person = _a_person(text, m.end(), &who);
        if !person.truthy() {
            continue;
        }
        let i = entry_for(&mut found, &person);
        found[i].setdefault("diet", diet.as_str());
    }
    for entry in _other_no_drink(text) {
        match found.iter().position(|seen| _same_person(seen, &entry)) {
            None => found.push(entry),
            Some(i) => {
                let diet = vget(&entry, "diet").clone();
                found[i].setdefault("diet", diet);
            }
        }
    }
    found
}

/// Python: bot/memory/facts.py:7956 `_other_conditions(text)`: what somebody else has:
/// [{"who": "mum", "condition": "diabetic"}].
pub fn _other_conditions(text: &str) -> Vec<Value> {
    let mut found = Vec::new();
    let text = stated_not_asked(text);
    if text.is_empty() {
        return found;
    }
    let text = text.as_str();
    for m in OTHER_CONDITION.finditer(text) {
        if NOT_DIET.is_match(m.as_str()) {
            continue;
        }
        let cond_at = m.name_span("cond").map(|s| s.0).unwrap_or(m.start());
        let (mut kind, mut who) = subject_at(text, cond_at);
        if kind.is_empty() {
            (kind, who) = _somebody(m.name_str("subj"));
        }
        if kind != "other" || ANIMALS.contains(&who) {
            continue;
        }
        let cond = squash(&lower(m.name_str("cond")));
        let cond = CONDITION_AS_A_WORD.get(&cond).cloned().unwrap_or(cond);
        let person = _a_person(text, m.end(), &who);
        if !person.truthy() {
            continue;
        }
        let i = entry_for(&mut found, &person);
        found[i].setdefault("condition", cond.as_str());
    }
    found
}

inline_re!(A_DIGIT_FIRST, r"\d", 0);

/// Python: bot/memory/facts.py:8022 `_the_job_phrase(phrase, place_job=False)`: a job as it is
/// said back: "nurse", "car mechanic", "primary school teacher".
pub fn _the_job_phrase(phrase: &str, place_job: bool) -> String {
    let mut phrase = phrase;
    let cut = _first_tail(phrase, if place_job { &JOB_TAIL_PLACE } else { &JOB_TAIL });
    if let Some((at, _)) = cut {
        phrase = &phrase[..at];
    }
    let words = without_the_moment(&pt::split(phrase));
    if words.is_empty() {
        return String::new();
    }
    let first = lower(&words[0]);
    if JOB_STOP.contains(&first)
        || NOT_NAMES.contains(&first)
        || NOT_NAMES_EXTRA.contains(&first)
        || SELF_JUDGEMENT.contains(&first)
        || _is_a_judgement_phrase(&words)
    {
        return String::new();
    }
    if A_DIGIT_FIRST.match_(&first).is_some() {
        return String::new();
    }
    _the_role(&words)
}

/// Python: bot/memory/facts.py:8043 `_other_jobs(text)`: what somebody else does:
/// [{"who": "sister", "job": "nurse"}].
pub fn _other_jobs(text: &str) -> Vec<Value> {
    let mut found: Vec<Value> = Vec::new();
    for (pattern, place_job) in [(&*OTHER_JOB_AS, true), (&*OTHER_JOB, false), (&*OTHER_JOB_NAMED, false)] {
        for m in pattern.finditer(text) {
            let at = m.name_span("job").map(|s| s.0).unwrap_or(m.start());
            let (kind, who) = subject_at(text, at);
            if kind != "other" {
                continue;
            }
            if JOB_WANTED.is_match(&CLAUSE_EDGE.split(&text[..at]).pop().unwrap_or_default()) {
                continue;
            }
            let job = _the_job_phrase(m.name_str("job"), place_job);
            if job.is_empty() || SOBER_WORDS.contains(&job) {
                continue;
            }
            let mut person = _a_person(text, at, &who);
            if !person.truthy() {
                continue;
            }
            let said = pt::strip(m.name_str("name")).to_string();
            if !said.is_empty() && !_not_a_name(&said, false) && _name_after_a_title(&said) && !_A_WORD_FOR_A_PERSON.contains(&lower(&said)) {
                person.set_key("name", tidy(&said));
            }
            let i = entry_for(&mut found, &person);
            found[i].setdefault("job", job.as_str());
        }
    }
    found
}

/// Python: bot/memory/facts.py:8084 `_names_before_a_fact(text)`: the names a line puts between the
/// animal and what it says about it, as (position, name).
pub fn _names_before_a_fact(text: &str) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    for m in PET_BEFORE_FACT.finditer(text) {
        let name = m.name_str("name");
        if _not_a_name(name, false) || NOT_A_NAME_MIDDLE.contains(&lower(name)) {
            continue;
        }
        let (name_start, name_end) = m.name_span("name").unwrap();
        if PET_COMPOUND.contains(&lower(name)) && AFTER_A_PET.match_(&text[name_end..]).is_none() {
            continue;
        }
        found.push((name_start, name.to_string()));
    }
    found
}

/// Python: bot/memory/facts.py:8103 `pet_name_only(text, lower=False)`: the name a line gives to a
/// pet it does not name again: "and her name is Biscuit"; None otherwise.
pub fn pet_name_only(text: &str, lower_ok: bool) -> Option<String> {
    for m in PET_NAME_ONLY.finditer(text) {
        let name = m.get(1).filter(|s| !s.is_empty()).or_else(|| m.get(2).filter(|s| !s.is_empty())).or_else(|| m.get(3).filter(|s| !s.is_empty()));
        if let Some(name) = name.and_then(|n| _a_name_for_the_pet(n, true)) {
            return Some(name);
        }
    }
    if lower_ok {
        for m in PET_NAME_ONLY_LOWER.finditer(text) {
            if let Some(name) = _a_name_for_the_pet(m.str(1), false) {
                return Some(name);
            }
        }
    }
    None
}

/// Python: bot/memory/facts.py:8128 `_count_before(text, start)`: "two cats", "2 dogs", "a couple
/// of rabbits": how many, or 0 when the line does not say. (`start` is a byte offset.)
pub fn _count_before(text: &str, start: usize) -> i64 {
    let Some(m) = PET_COUNT.search(&text[..start]) else {
        return 0;
    };
    let word = lower(m.str(1));
    if pt::is_digit(&word) {
        return py_int(&word).unwrap_or(0);
    }
    PET_COUNT_WORDS.get(&word).copied().unwrap_or(0)
}

/// Python: bot/memory/facts.py:8144 `pet_counts(text)`: the animals a corrected line counts: "3
/// cats", "two dogs" -> [{"animal", "count"}].
pub fn pet_counts(text: &str) -> Value {
    let found = _pets(&format!("i have {text}"), false);
    let out: Vec<Value> = found
        .iter()
        .filter(|pet| vget(pet, "count").truthy())
        .map(|pet| dict(vec![("animal", vget(pet, "animal").clone()), ("count", vget(pet, "count").clone())]))
        .collect();
    Value::from(out)
}

// ---------------------------------------------------------------------------------------------
// pets: names after a count, whose animal it is, the people a line names, owned things, _pets

fres!(
    OTHER_PET, OWNER_OF, LIVES_ELSEWHERE, OWNER_OF_PLAIN, OWN_ANIMAL, OURS_ANOTHER, KIN_OWNER,
    FOOD_BEFORE, FOOD_AFTER, NAME_BEFORE, THEIR_ANIMAL, SOMEONE_ELSE_HAS, A_RELATION_OWNED,
    OWNED_VALUE, OPENS_AN_ASK, TELLS_ABOUT, PET_AGE_TAIL, PET_AGE, PET_AGE_BARE,
);
fsets!(OWNED, OWNED_STOP, OWNED_NOT_A_MODEL);
fmaps!(OWNED_KIND, GROWN_UP);

/// `OWNED_MAKES` / `OWNED_MODELS`: word -> (the kind of thing, or None; the spelling).
fn owned_table(name: &str) -> HashMap<String, (Option<String>, String)> {
    data::get(&format!("bot.memory.facts.{name}"))
        .pairs()
        .iter()
        .filter_map(|(k, v)| {
            let items = v.items();
            Some((k.as_str()?.to_string(), (items.first()?.as_str().map(str::to_string), items.get(1)?.str().to_string())))
        })
        .collect()
}
static OWNED_MAKES: LazyLock<HashMap<String, (Option<String>, String)>> = LazyLock::new(|| owned_table("OWNED_MAKES"));
static OWNED_MODELS: LazyLock<HashMap<String, (Option<String>, String)>> = LazyLock::new(|| owned_table("OWNED_MODELS"));
static BREED_KIND: LazyLock<HashMap<String, String>> = LazyLock::new(|| bundle_map("bot.understanding.breeds.BREED_KIND"));
static NOT_A_CREATURE_NEXT: LazyLock<HashSet<String>> = LazyLock::new(|| bundle_set("bot.understanding.breeds.NOT_A_CREATURE_NEXT"));

inline_re!(CALLED_FIRST, r"\s*(?:(?:are|is|were|was)\s+)?(?:called|named)\s+", I);
inline_re!(COMMA_FIRST, r"\s*[,:]\s*", 0);
inline_re!(NAME_LIST_SPLIT, r"\s*(?:,|/|&|\band\b)\s*", 0);

/// Python: bot/memory/facts.py:8155 `_names_after(rest, count, most=None)`: the names a line lists
/// for the animals it has just counted.
pub fn _names_after(rest: &str, count: i64, most: Option<i64>) -> Vec<String> {
    let most_v = most.unwrap_or(0);
    if count < 2 && most_v == 0 {
        return Vec::new();
    }
    let target = if count >= 2 { count } else { most_v };
    let rest = if let Some(m) = CALLED_FIRST.match_(rest) {
        &rest[m.end()..]
    } else if let Some(m) = COMMA_FIRST.match_(rest) {
        &rest[m.end()..]
    } else {
        return Vec::new();
    };
    let mut names: Vec<String> = Vec::new();
    for piece in NAME_LIST_SPLIT.split(rest) {
        let piece = pt::strip_chars(pt::strip(&piece), ".!?").to_string();
        if piece.is_empty() || pt::split(&piece).len() != 1 {
            break;
        }
        if NAME_WORD.match_(&piece).is_none() || NOT_A_NAME.contains(&lower(&piece)) {
            break;
        }
        names.push(piece);
        if names.len() as i64 == target {
            break;
        }
    }
    if count < 2 {
        return if names.len() >= 2 { names } else { Vec::new() };
    }
    if names.len() as i64 == count {
        names
    } else {
        Vec::new()
    }
}

/// Python: bot/memory/facts.py:8260 `_not_their_animal(text, start, end)`: true when the animal at
/// this position belongs to somebody else, or to nobody. (Byte offsets.)
pub fn _not_their_animal(text: &str, start: usize, end: usize) -> bool {
    let before = &text[back_chars(text, start, 40)..start];
    if KIN_OWNER.is_match(before) {
        return true;
    }
    if OWN_ANIMAL.is_match(before) {
        return false;
    }
    if OURS_ANOTHER.is_match(before) {
        return false;
    }
    OWNER_OF.is_match(before) || OWNER_OF_PLAIN.is_match(before) || LIVES_ELSEWHERE.match_(&text[end..fwd_chars(text, end, 40)]).is_some()
}

inline_re!(A_SENTENCE_STOP, r"[.!?;]", 0);

/// Python: bot/memory/facts.py:8275 `_someone_elses(text)`: the parts of a line where the animal
/// belongs to another person, as byte spans.
pub fn _someone_elses(text: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    for m in OTHER_PET.finditer(text) {
        let end = A_SENTENCE_STOP.search(&text[m.end()..]);
        spans.push((m.end(), end.map(|e| m.end() + e.start()).unwrap_or(text.len())));
    }
    spans
}

static RELATION_WORD_PLURAL_RES: LazyLock<Vec<(String, Re)>> =
    LazyLock::new(|| RELATIONS.iter().map(|w| (w.clone(), Re::new(&format!(r"\b{w}s?\b"), I))).collect());
inline_re!(IN_LAW_AFTER_SPACE, r"\s+in\s+laws?\b", I);

/// Python: bot/memory/facts.py:8329 `_people(text)`: the people this line is about. "my parents
/// evening" is an evening, not the people in it.
pub fn _people(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (relation, re) in RELATION_WORD_PLURAL_RES.iter() {
        for m in re.finditer(text) {
            let mut after: &str = &text[m.end()..];
            let said_possessive = after.starts_with("'s") || after.starts_with("s'");
            if said_possessive {
                // `after.split("'", 1)[1][1:]`
                let rest = &after[after.find('\'').unwrap() + 1..];
                after = match rest.chars().next() {
                    Some(c) => &rest[c.len_utf8()..],
                    None => rest,
                };
            }
            if IN_LAW_AFTER.match_(after).is_some() {
                found.push(format!("{relation} in law"));
                break;
            }
            let words: Vec<String> =
                pt::split(after).iter().map(|w| pt::strip_chars(w, ".,!?;:'").to_string()).filter(|w| !w.is_empty()).collect();
            let next_word = words.first().map(|w| lower(w)).unwrap_or_default();
            if COMPOUND_AFTER.contains(&next_word) {
                if !((lower(m.as_str()).ends_with('s') || said_possessive) && COMPOUND_THAT_IS_THEIRS.contains(&next_word)) {
                    continue;
                }
            }
            let before = _the_clause_before(text, m.start(), 4);
            if SOMEONE_ELSE_HAS.is_match(&before) || A_RELATION_OWNED.is_match(&before) {
                continue;
            }
            if SELF_AS_A_ROLE.is_match(&text[..m.start()]) && is_a_judgement(text) {
                continue;
            }
            match IN_LAW_AFTER_SPACE.match_(after) {
                Some(inlaw) => found.push(format!("{relation} {}", squash(inlaw.as_str()))),
                None => found.push(relation.clone()),
            }
            break;
        }
    }
    found
}

/// Python: bot/memory/facts.py:8495 `_a_question(text)`: whether the line asks rather than tells.
pub fn _a_question(text: &str) -> bool {
    let low = pt::strip(text);
    if OPENS_AN_ASK.match_(low).is_some() {
        return true;
    }
    low.ends_with('?') && TELLS_ABOUT.match_(low).is_none()
}

/// Python: bot/memory/facts.py:8503 `_owns(text)`: what the person says they have, by the name of
/// the thing: {"car": "Ford Fiesta"}.
pub fn _owns(text: &str) -> Value {
    let mut found = Value::dict();
    if text.is_empty() || _a_question(text) {
        return found;
    }
    for m in OWNED_VALUE.finditer(text) {
        let g = |n: &str| m.name(n).filter(|s| !s.is_empty());
        let mut thing = lower(g("thing").or_else(|| g("thing2")).unwrap_or(""));
        let said = g("value").or_else(|| g("value2")).or_else(|| g("value3")).or_else(|| g("value4")).unwrap_or("");
        if thing.is_empty() {
            if let Some(v4) = g("value4") {
                thing = _the_thing_of(v4);
            }
        }
        if thing.is_empty() {
            thing = match lower(g("verb").unwrap_or("")).as_str() {
                "drive" => "car".into(),
                "ride" => "bike".into(),
                _ => String::new(),
            };
        }
        let thing = thing.replace('-', " ");
        if !OWNED.contains(&thing) || found.get(&thing).is_some() {
            continue;
        }
        let value = _the_model(&Value::from(said), &thing);
        if !value.is_empty() {
            found.set_key(thing.as_str(), value);
        }
    }
    found
}

inline_re!(MODEL_WORDS, r"[A-Za-z0-9][A-Za-z0-9'+-]*", 0);

/// Python: bot/memory/facts.py:8534 `_the_thing_of(value)`: the thing a make belongs to.
pub fn _the_thing_of(value: &str) -> String {
    let words = MODEL_WORDS.findall_str(value);
    let family = words.first().and_then(|w| OWNED_MAKES.get(&lower(w))).and_then(|(k, _)| k.clone());
    match family.as_deref() {
        Some(k @ ("car" | "phone" | "bike" | "boiler")) => k.to_string(),
        _ => String::new(),
    }
}

/// Python: bot/memory/facts.py:8541 `_the_model(value, thing="")`: the make and model out of what
/// followed the article, or "" when it is not one.
pub fn _the_model(value: &Value, thing: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    for word in MODEL_WORDS.findall_str(&str_or_empty(Some(value))) {
        if OWNED_STOP.contains(&lower(&word)) || words.len() == 4 {
            break;
        }
        words.push(word);
    }
    let words = without_the_moment(&words);
    if words.is_empty() {
        return String::new();
    }
    let first = lower(&words[0]);
    if OWNED_NOT_A_MODEL.contains(&first) {
        return String::new();
    }
    let kind = OWNED_KIND.get(&lower(thing)).cloned().unwrap_or_default();
    if let Some((make_kind, _)) = OWNED_MAKES.get(&first) {
        if !kind.is_empty() && make_kind.as_ref().is_some_and(|k| !k.is_empty() && *k != kind) {
            return String::new();
        }
        return _spell_the_model(&words);
    }
    match OWNED_MODELS.get(&first) {
        Some((Some(model_kind), _)) if !kind.is_empty() && *model_kind == kind => _spell_the_model(&words),
        _ => String::new(),
    }
}

/// Python: bot/memory/facts.py:8574 `_spell_the_model(words)`: the value written the way a name
/// is: "ford fiesta" -> "Ford Fiesta", "bmw" -> "BMW".
pub fn _spell_the_model(words: &[String]) -> String {
    words
        .iter()
        .map(|w| {
            let l = lower(w);
            match OWNED_MAKES.get(&l).or_else(|| OWNED_MODELS.get(&l)) {
                Some((_, spelled)) => spelled.clone(),
                None => up1(w),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Python: bot/memory/facts.py:8583 `_a_name_given_by_the_verb(word)`: true when the word after "we
/// named the cat ..." is the name and not the rest of a sentence.
pub fn _a_name_given_by_the_verb(word: &str) -> bool {
    let word = pt::strip(word);
    let low = lower(word);
    if NOT_A_NAME.contains(&low) || NOT_THE_VERB_NAME.contains(&low) || NOT_A_PRONOUN_NAME.contains(&low) {
        return false;
    }
    NAME_WORD.match_(word).is_some()
}

/// Python: bot/memory/facts.py:8603 `_the_same_animal(one, other)`: whether two names for a kind of
/// animal are the same animal.
pub fn _the_same_animal(one: &str, other: &str) -> bool {
    GROWN_UP.get(one).map(String::as_str).unwrap_or(one) == GROWN_UP.get(other).map(String::as_str).unwrap_or(other)
}

/// Python: bot/memory/facts.py:8608 `_with_the_breed(pet, breed)`: the record, with the breed the
/// line named kept beside the kind of animal it is.
pub fn _with_the_breed(mut pet: Value, breed: &str) -> Value {
    if !breed.is_empty() {
        pet.set_key("breed", breed);
    }
    pet
}

inline_re!(WORD_AFTER_A_BREED, r"[\s-]*([a-z]+)", 0);
inline_re!(WORD_AFTER_AN_ANIMAL, r"\s*([a-z]+)(.{0,40})", crate::py::re::S);

/// Python: bot/memory/facts.py:8619 `_pets(text, question=False)`: the animals a line introduces,
/// with the names it gives them.
pub fn _pets(text: &str, question: bool) -> Vec<Value> {
    let theirs = _someone_elses(text);
    let in_theirs = |p: usize| theirs.iter().any(|(start, stop)| *start <= p && p < *stop);
    let mut names: Vec<(usize, String)> = PET_NAME.finditer(text).iter().map(|m| (_name_at(m), _with_title_m(m))).collect();
    names.extend(
        PET_VERB_NAME
            .finditer(text)
            .iter()
            .filter(|m| _a_name_given_by_the_verb(m.name_str("name")) && !NOT_THEIR_DETERMINER.contains(&lower(m.name_str("whose"))))
            .map(|m| (_name_at(m), _with_title_m(m))),
    );
    names.extend(
        PET_PRONOUN_NAME
            .finditer(text)
            .iter()
            .filter(|m| _a_name_given_by_the_verb(m.str(1)))
            .map(|m| (m.span(1).unwrap().0, m.str(1).to_string())),
    );
    let before_a_fact: Vec<(usize, String)> =
        _names_before_a_fact(text).into_iter().filter(|(_, n)| !names.iter().any(|(_, k)| k == n)).collect();
    names.extend(before_a_fact);
    let is_names: Vec<(usize, String)> = PET_IS_NAME
        .finditer(text)
        .iter()
        .filter(|m| !names.iter().any(|(_, k)| *k == _with_title_m(m)))
        .map(|m| (_name_at(m), _with_title_m(m)))
        .collect();
    names.extend(is_names);
    let possessive: Vec<(usize, String)> = PET_POSSESSIVE_NAME
        .finditer(text)
        .iter()
        .filter(|m| _a_name_for_the_pet(m.name_str("name"), false).is_some() && !names.iter().any(|(_, k)| *k == _with_title_m(m)))
        .map(|m| (m.name_span("name").unwrap().0, _with_title_m(m)))
        .collect();
    names.extend(possessive);
    names.extend(
        PET_APPOSITIVE.finditer(text).iter().filter(|m| _names_it_aside(m, text)).map(|m| (m.span(1).unwrap().0, m.str(1).to_string())),
    );
    let mut names: Vec<(usize, String)> = names.into_iter().filter(|(_, n)| !NOT_A_NAME.contains(&lower(n))).collect();
    names.sort();
    if question {
        names.retain(|(_, n)| n.chars().next().is_some_and(char::is_uppercase));
    }
    let named_at: HashSet<usize> = names.iter().map(|(p, _)| *p).collect();
    // [start, end, animal, breed]
    let mut animals: Vec<(usize, usize, String, String)> = PETS
        .finditer(text)
        .iter()
        .filter(|m| !in_theirs(m.start()) && !named_at.contains(&m.start()))
        .map(|m| (m.start(), m.end(), lower(m.str(1)), String::new()))
        .collect();
    let mut one_of_the_group: HashSet<usize> = HashSet::new();
    for m in BREED_RE.finditer(text) {
        if in_theirs(m.start()) {
            continue;
        }
        if named_at.contains(&m.start()) {
            continue;
        }
        let breed = crate::understanding::breeds::the_breed_named(m.as_str());
        let kind = BREED_KIND.get(&breed).cloned().unwrap_or_default();
        if kind.is_empty() {
            continue;
        }
        if !crate::understanding::breeds::a_frame_before(text, m.start()) {
            continue;
        }
        if let Some(after) = WORD_AFTER_A_BREED.match_(&text[m.end()..fwd_chars(text, m.end(), 24)]) {
            if NOT_A_CREATURE_NEXT.contains(&lower(after.str(1))) {
                continue;
            }
        }
        let at_chars = pt::char_index(text, m.start()) as i64;
        let distance = |p: usize| (pt::char_index(text, p) as i64 - at_chars).abs();
        let beside: Vec<usize> =
            (0..animals.len()).filter(|&i| _the_same_animal(&animals[i].2, &kind) && distance(animals[i].0) <= 40).collect();
        if !beside.is_empty() {
            let mut closest = beside[0];
            for &i in &beside[1..] {
                if distance(animals[i].0) < distance(animals[closest].0) {
                    closest = i;
                }
            }
            if animals[closest].3.is_empty() {
                animals[closest].3 = breed.clone();
                if _count_before(text, animals[closest].0) > 1 {
                    one_of_the_group.insert(animals[closest].0);
                }
            }
            continue;
        }
        animals.push((m.start(), m.end(), kind, breed));
    }
    animals.sort();
    let mut pets: Vec<Value> = Vec::new();
    let one_animal = animals.len() == 1;
    for index in 0..animals.len() {
        let (start, end, ref animal, ref breed) = animals[index];
        let limit = if index + 1 < animals.len() { animals[index + 1].0 } else { text.len() };
        let before = &text[back_chars(text, start, 60)..start];
        let second = ANOTHER_ANIMAL.is_match(before) || ALSO_HAVE.is_match(before);
        if FOOD_BEFORE.is_match(&text[back_chars(text, start, 30)..start]) {
            continue;
        }
        if crate::memory::taste::said_about(text, start as i64) {
            continue;
        }
        if FOOD_AFTER.match_(&text[end..fwd_chars(text, end, 30)]).is_some() {
            continue;
        }
        let mut compound_they_have = false;
        if let Some(after) = WORD_AFTER_AN_ANIMAL.match_(&text[end..]) {
            if PET_COMPOUND.contains(&lower(after.str(1))) {
                let theirs_word = THEIR_ANIMAL.is_match(&text[back_chars(text, start, 12)..start]);
                let named = theirs_word && AFTER_A_PET.match_(after.str(2)).is_some();
                if !named {
                    if !theirs_word {
                        continue;
                    }
                    compound_they_have = true;
                }
            }
        }
        if NAME_BEFORE.is_match(&text[back_chars(text, start, 12)..start]) {
            continue;
        }
        if _not_their_animal(text, start, end) {
            pets.push(_with_the_breed(_not_theirs(animal, text, start, end), breed));
            continue;
        }
        let clause = clause_before(text, start);
        if _only_wanted(text, start, end)
            || NOT_HAVE.is_match(&clause)
            || WITHOUT_ANIMAL.is_match(&clause)
            || NOT_JUST.is_match(&clause)
            || taken_back_before(text, start)
        {
            pets.push(_with_the_breed(_not_theirs(animal, text, start, end), breed));
            continue;
        }
        let coming = _coming_home(text, start);
        let count = _count_before(text, start);
        let plural = count < 2 && lower(&text[start..end]).ends_with('s');
        let mut listed = _names_after(&text[end..limit], count, if plural { Some(bundle_int("MOST_NAMES")) } else { None });
        if compound_they_have {
            listed.clear();
        }
        if question {
            listed.retain(|n| n.chars().next().is_some_and(char::is_uppercase));
        }
        if !listed.is_empty() {
            for name in &listed {
                let mut pet = dict(vec![("animal", animal.as_str().into()), ("name", name.as_str().into())]);
                if coming {
                    pet.set_key("coming", true);
                }
                pets.push(_with_the_breed(pet, breed));
            }
            continue;
        }
        let mut name: Option<String> = None;
        if let Some(i) = names.iter().position(|(p, _)| end <= *p && *p < limit) {
            name = Some(names.remove(i).1);
        }
        if question && name.is_none() && PET_APPOSITIVE.search(&text[start..limit]).is_none() {
            continue;
        }
        let mut pet = if name.is_none() && count > 0 {
            dict(vec![("animal", animal.as_str().into()), ("name", Value::None), ("count", count.into())])
        } else {
            dict(vec![("animal", animal.as_str().into()), ("name", Value::from(name.clone()))])
        };
        if coming {
            pet.set_key("coming", true);
        }
        let mut age = _pet_age(&text[end..limit]);
        if age.is_none() && one_animal && !vget(&pet, "count").truthy() {
            age = _pet_age_tail(&text[end..limit]);
        }
        if let Some((a, unit)) = age {
            pet.set_key("age", a);
            if !unit.is_empty() {
                pet.set_key("unit", unit);
            }
        }
        if second {
            pet.set_key("another", true);
        }
        if !breed.is_empty() && one_of_the_group.contains(&start) {
            pet.set_key("breed_of_one", true);
        }
        pets.push(_with_the_breed(pet, breed));
    }
    pets
}

/// Python: bot/memory/facts.py:8846 `_pet_age_tail(after)`: the age said after the animal's own
/// description, or None: "hes a labrador, 9".
pub fn _pet_age_tail(after: &str) -> Option<(i64, String)> {
    let m = PET_AGE_TAIL.match_(after)?;
    let age = int_of(m.name_str("age"));
    if !(0 < age && age <= 40) {
        return None;
    }
    let unit = lower(m.name_str("unit"));
    let unit = if unit.starts_with("month") {
        "months"
    } else if unit.starts_with("week") {
        "weeks"
    } else {
        ""
    };
    Some((age, unit.to_string()))
}

/// Python: bot/memory/facts.py:8884 `_pet_age(rest)`: how old the animal is, when the words right
/// after it say so, or None.
pub fn _pet_age(rest: &str) -> Option<(i64, String)> {
    let m = PET_AGE.match_(rest).or_else(|| PET_AGE_BARE.match_(rest))?;
    let age = int_of(m.name_str("age"));
    if !(0 < age && age <= 30) {
        return None;
    }
    let unit = lower(m.name_str("unit"));
    let unit = if unit.starts_with("month") {
        "months"
    } else if unit.starts_with("week") {
        "weeks"
    } else {
        ""
    };
    Some((age, unit.to_string()))
}

// ---------------------------------------------------------------------------------------------
// names and plans: describing words, bare names, plan text, appointments, dated plans,
// continued parts, the bot's name, extract(), and where it hurts

fres!(
    PLAN, INFINITIVE_FRAME, PERSON_OPEN, _FAREWELL, _GOT_A, _GOT_TO, OWN, SECOND_PERSON, NEGATED,
    ASKED_BEFORE, ASKING, PAST, PLAN_CLAUSE_BREAK, PLAN_CLAUSE_OPENER, COPULA, OFF_ABSENCE,
    OFF_AHEAD, THING_COPULA, AHEAD_OF_THE_THING, APPOINTMENT_CLOCK, APPOINTMENT_DAY,
    APPOINTMENT_PAST, APPOINTMENT_OWN_SUBJECT, TAKE_BACK_WORDS, A_DAY_SAID_FIRST,
    _A_COUNT_NOT_A_CLOCK, OBLIGATION, RECURRENCE, A_CLOCKED_EVENT, CLAUSE_SUBJECT, WHEN_OR_WHERE,
    A_SHARED_VERB, CORRECTION_LEAD_RE, OPENER_PUNCTUATION, THE_THING_HAS_A_DAY, SUBJECT_JOIN,
    SELF_INTRO, CONTINUATION_SPLIT, WISHING, SAME_PERSON_CLAUSE, SAME_PERSON_SPLIT, BOT_NAME,
    BOT_NAME_ASKED, BOT_NAME_TAG, BOT_NAME_ASK, CORRECTION_OPENER, REPORTED, BODY_PART, BODY_HURTS,
    BODY_LATERAL, BODY_PLACE, SOMEONE_ELSE, A_PART_HANDED_OVER, MY_OWN_PART, THE_PART_HURTS,
    THE_PERSON_SAYS, A_PLACE_ON_IT,
);
fsets!(
    DESCRIBING_WORDS, ED_NAMES, NOT_A_NAME_WORD, COMMON_WORDS, NOT_A_NAME_EITHER, PARTICLES,
    PREPOSITIONS, POINTING, STATE_FILLER, STATE_PARTICLE,
    STATE_MORE, THING_TODAY, APPOINTMENT_NOT_A_THING, APPOINTMENT_PAST_VERB, APPOINTMENT_A_VERB,
    APPOINTMENT_A_DATE_VERB, APPOINTMENT_NOT_A_VERB, APPOINTMENT_CORRECTION, DAY_REMARK_WORDS,
    DAY_REMARK_GLUE, OTHER_START, NOT_A_PART, PERSON_SUBJECT, CONTINUATION_VERB,
    CONTINUATION_KEYS, BOT_NAME_TITLE, BOT_NAME_STOP, THEIR_NAME, THEIR_PLAN,
);
fstrs!(DESCRIBING_ENDS, NOT_A_NAME_ENDINGS, OPENER_WORDS, BODY_PARTS);

static LEX_COMMON_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| bundle_set("bot.understanding.lexicon.COMMON_NAMES"));
static WORD_NUMBERS: LazyLock<HashSet<String>> = LazyLock::new(|| bundle_set("bot.memory.facts.WORD_NUMBERS"));

/// `SAYS_WHAT_THEY_ARE`: the leads (word tuples) that say a line is about them.
static SAYS_WHAT_THEY_ARE: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| {
    data::get("bot.memory.facts.SAYS_WHAT_THEY_ARE").items().iter().map(|t| t.strs()).collect()
});
/// `SUBJECT_OPENERS`: (shape, frame) pairs.
static SUBJECT_OPENERS: LazyLock<Vec<(&'static Re, String)>> = LazyLock::new(|| {
    data::get("bot.memory.facts.SUBJECT_OPENERS")
        .items()
        .iter()
        .map(|t| (t.items()[0].as_re().unwrap(), t.items()[1].str().to_string()))
        .collect()
});

/// Python: bot/memory/facts.py:8967 `_describes_them(word)`: whether a word says how somebody is
/// rather than what they are called.
pub fn _describes_them(word: &str) -> bool {
    let low = lower(word);
    if low.is_empty() {
        return false;
    }
    if STATE_WORDS.contains(&low) || DESCRIBING_WORDS.contains(&low) || LEX_A_STATE.contains(&low) {
        return true;
    }
    if DIET_WORDS.contains(&low) || DIET_NAMES.contains_key(&low) || DIET_FREE_WORDS.contains(&low) || SOBER_WORDS.contains(&low) {
        return true;
    }
    if ED_NAMES.contains(&low) {
        return false;
    }
    if low.ends_with("ed") && pt::len(&low) > 3 {
        return true;
    }
    DESCRIBING_ENDS.iter().any(|end| low.ends_with(end.as_str()) && pt::len(&low) > pt::len(end) + 2)
}

inline_re!(A_NAME_SHAPE, r"[a-z][a-z'-]{1,19}", 0);

/// Python: bot/memory/facts.py:8991 `_bare_name(word, pair=False)`: the name a line gives without
/// the naming verb, or "" when the word is something else.
pub fn _bare_name(word: &str, pair: bool) -> String {
    let said = pt::strip_chars(pt::strip(word), ".,!?;:'\"").to_string();
    let low = lower(&said);
    if A_NAME_SHAPE.fullmatch(&low).is_none() {
        return String::new();
    }
    if _not_a_name(&said, false) {
        return String::new();
    }
    if WORD_NUMBERS.contains(&low) {
        return String::new();
    }
    if NOT_A_NAME_WORD.contains(&low) {
        return String::new();
    }
    if is_a_contraction(&low) {
        return String::new();
    }
    if LEX_COMMON_NAMES.contains(&low) || NAMES_THAT_END_LIKE_A_VERB.contains(&low) {
        return tidy(&_finish(&said));
    }
    let capital = said.chars().next().is_some_and(char::is_uppercase);
    if !capital && _describes_them(&low) {
        return String::new();
    }
    if !pair && LEXICON.contains(&low) && !capital {
        return String::new();
    }
    tidy(&_finish(&said))
}

/// Python: bot/memory/facts.py:9047 `a_name_could_be(word)`: whether a single word could be
/// somebody's name, with nothing else in the line to say so.
pub fn a_name_could_be(word: &str) -> bool {
    let low = lower(pt::strip(word));
    if A_NAME_SHAPE.fullmatch(&low).is_none() {
        return false;
    }
    if MONTHS.contains(&low) || DAYS.contains(&low) {
        return false;
    }
    if LEX_COMMON_NAMES.contains(&low) {
        return true;
    }
    if NOT_A_PET_NAME.contains(&low) || NOT_NAMES.contains(&low) || NOT_NAMES_EXTRA.contains(&low) {
        return false;
    }
    if STATE_WORDS.contains(&low) || DESCRIBING_WORDS.contains(&low) {
        return false;
    }
    if !COMMON_WORDS.contains(&low) && NOT_A_NAME_ENDINGS.iter().any(|e| low.ends_with(e.as_str())) {
        return false;
    }
    !COMMON_WORDS.contains(&low)
}

inline_re!(NEGATED_VERB, "^(?P<stem>[a-z]{2,})n['\u{2019}]?t$", 0);

/// Python: bot/memory/facts.py:9080 `is_a_contraction(word)`: whether a word is a verb with its
/// "not" written on: "isnt", "arent", "didnt".
pub fn is_a_contraction(word: &str) -> bool {
    let low = lower(pt::strip_chars(pt::strip(word), ".,!?;:'\"")).replace('\u{2019}', "'");
    let Some(m) = NEGATED_VERB.match_(&low) else {
        return false;
    };
    let stem = m.name_str("stem");
    LEXICON.contains(stem) || LEXICON.contains(&format!("{stem}e"))
}

/// Python: bot/memory/facts.py:9095 `_not_a_name(word, strict_ly=True)`: whether a word after the
/// comma is one of the things people say about a pet, not a name.
pub fn _not_a_name(word: &str, strict_ly: bool) -> bool {
    let low = lower(word);
    if is_addressing(word) || NOT_A_PET_NAME.contains(&low) || NOT_NAMES.contains(&low) || NOT_NAMES_EXTRA.contains(&low) {
        return true;
    }
    if NOT_A_NAME_EITHER.contains(&low) {
        return true;
    }
    if PET_ANIMALS.contains(&low) || !A_VOWEL.is_match(&low) {
        return true;
    }
    if strict_ly && low.ends_with("ly") && pt::len(&low) > 4 {
        return true;
    }
    if _a_word_not_a_name(&low) {
        return true;
    }
    ONLY_DIGITS.fullmatch(&low).is_some()
}

/// Python: bot/memory/facts.py:9119 `_object_words(rest)`: the words of a plan's object, once the
/// preposition and the article are out of the way.
pub fn _object_words(rest: &str) -> Vec<String> {
    let words: Vec<String> = LOWER_WORDS.findall_str(&lower(rest)).into_iter().filter(|w| !ARTICLES.contains(w)).collect();
    if words.is_empty() {
        return words;
    }
    words.into_iter().filter(|w| !PREPOSITIONS.contains(w)).collect()
}

inline_re!(NOT_A_LOWER_LETTER, r"[^a-z]", 0);

/// Python: bot/memory/facts.py:9125 `_object_plan(rest)`: whether the rest of a plan is one the
/// verb has to carry.
pub fn _object_plan(rest: &str) -> bool {
    let Some(first) = pt::split_ref(rest).first().copied() else {
        return false;
    };
    PREPOSITIONS.contains(&NOT_A_LOWER_LETTER.sub("", &lower(first)))
}

/// Python: bot/memory/facts.py:9138 `_plan_verb(match, rest)`: the verb the plan pattern stopped
/// after, when the plan is that verb and its object. `said` is `match.group(0)`.
pub fn _plan_verb(said: &str, rest: &str) -> String {
    let verb = pt::strip_chars(&PERSON_OPEN.subn("", said, 1), " .").to_string();
    let words = pt::split(&verb);
    if words.is_empty() || !lower(&words[0]).ends_with("ing") {
        return String::new();
    }
    if _object_plan(rest) || PARTICLES.contains(&lower(words.last().unwrap())) {
        return up1(&verb);
    }
    String::new()
}

/// Python: bot/memory/facts.py:9153 `_a_plan_has_a_thing(rest)`: whether the object of the verb
/// names something, rather than pointing at it.
pub fn _a_plan_has_a_thing(rest: &str) -> bool {
    if !_object_plan(rest) {
        return true;
    }
    let words = _object_words(rest);
    !words.is_empty() && !POINTING.contains(&words[0])
}

/// Python: bot/memory/facts.py:9175 `_asks_about_the_plan(text, at)`: whether the plan frame at
/// `at` (a byte offset) sits in a question rather than in a statement.
pub fn _asks_about_the_plan(text: &str, at: usize) -> bool {
    let before = &text[..at];
    if ASKED_BEFORE.is_match(before) {
        return true;
    }
    let clause = PLAN_CLAUSE_BREAK.split(before).pop().unwrap_or_default();
    let clause = pt::strip(&PLAN_CLAUSE_OPENER.sub("", pt::strip(&clause))).to_string();
    !clause.is_empty() && ASKING.match_(&clause).is_some()
}

inline_re!(PLAN_TEXT_END, r"(?<=[.!?])\s|\s+(?:but|and then|so)\s+", 0);
inline_re!(
    PLAN_TEXT_ASK,
    r",\s+(?:(?:how|what|which|where|when|who|why|should|can|could|would|do|does|is|are|will|any|and)\b)",
    0
);

/// `re.split(pattern, text, 1)[0]`: the text up to the first match.
fn before_first(pattern: &Re, text: &str) -> String {
    match pattern.search(text) {
        Some(m) => text[..m.start()].to_string(),
        None => text.to_string(),
    }
}

/// Python: bot/memory/facts.py:9191 `_plan_text(text, start)`: the plan itself: from the verb the
/// pattern stopped after, to the end of its clause. (`start` is a byte offset.)
pub fn _plan_text(text: &str, start: usize) -> String {
    let rest = pt::strip_chars(&text[start..], " ,;:-").to_string();
    let rest = pt::strip_chars(&before_first(&PLAN_TEXT_END, &rest), " ,;:-").to_string();
    let rest = pt::strip_chars(&before_first(&PLAN_TEXT_ASK, &rest), " ,;:-").to_string();
    if rest.is_empty() {
        return String::new();
    }
    up1(&rest)
}

inline_re!(NOT_A_WORD_CHAR_OR_SPACE, r"[^a-z' ]", 0);

/// The words of a predicate after the date and the filler come out.
fn predicate_words(rest: &str) -> Vec<String> {
    let rest = format!(" {} ", lower(rest));
    let rest = NOT_A_WORD_CHAR_OR_SPACE.sub(" ", &FUTURE.sub(" ", &rest));
    pt::split(&rest)
        .into_iter()
        .filter(|w| !STATE_FILLER.contains(w))
        .filter(|w| !STATE_PARTICLE.contains(w))
        .collect()
}

/// Python: bot/memory/facts.py:9248 `_how_they_are(clause)`: true when the clause only says how
/// the person is, so a date on it is not a plan.
pub fn _how_they_are(clause: &str) -> bool {
    let Some(m) = COPULA.match_(clause) else {
        return false;
    };
    if OFF_ABSENCE.is_match(m.name_str("rest")) && OFF_AHEAD.is_match(clause) {
        return false;
    }
    let words = predicate_words(m.name_str("rest"));
    !words.is_empty() && words.iter().all(|w| STATE_WORDS.contains(w))
}

/// Python: bot/memory/facts.py:9295 `_how_their_things_are(clause)`: true when the clause only says
/// how a thing of theirs is, so a date on it is not a plan.
pub fn _how_their_things_are(clause: &str) -> bool {
    let Some(m) = THING_COPULA.match_(clause) else {
        return false;
    };
    let raw = lower(m.name_str("rest"));
    if AHEAD_OF_THE_THING.is_match(&raw) {
        return false;
    }
    let words = predicate_words(&raw);
    !words.is_empty() && words.iter().all(|w| STATE_WORDS.contains(w) || THING_TODAY.contains(w))
}

/// Python: bot/memory/facts.py:9400 `_the_day_described(thing)`: true when the thing before the day
/// is how that day is, not something they have on it.
pub fn _the_day_described(thing: &str) -> bool {
    pt::split(&lower(thing))
        .iter()
        .map(|w| pt::strip_chars(w, ",.!;:'\u{2019}").to_string())
        .filter(|w| !w.is_empty() && !DAY_REMARK_WORDS.contains(w) && !DAY_REMARK_GLUE.contains(w))
        .next()
        .is_none()
}

inline_re!(ON_IN_AT_FIRST, r"^(?:on|in|at)\s+", 0);
inline_re!(A_DETERMINER_FIRST, r"^(?:the|my|our|a|an)\s+", 0);

/// Python: bot/memory/facts.py:9435 `_the_thing_said_after_the_day(low, day)`: the thing named after
/// a day said at the front of the line, "" when the rest is not a thing.
pub fn _the_thing_said_after_the_day(low: &str, day: &Match) -> String {
    if pt::char_index(low, day.start()) > 4 {
        return String::new();
    }
    let rest = pt::strip_chars(&low[day.end()..], " ,;:-").to_string();
    let rest = ON_IN_AT_FIRST.sub("", &rest);
    let mut kept: Vec<String> = Vec::new();
    for word in pt::split(&rest) {
        let bare = pt::strip_chars(&word, ",.!;:").to_string();
        if APPOINTMENT_A_VERB.contains(&bare) || APPOINTMENT_A_DATE_VERB.contains(&bare) {
            break;
        }
        if pt::len(&bare) > 3 && bare.ends_with("ed") && !APPOINTMENT_NOT_A_VERB.contains(&bare) {
            break;
        }
        kept.push(word);
    }
    let thing = pt::strip_chars(&kept.join(" "), " ,;:-").to_string();
    A_DETERMINER_FIRST.sub("", &thing)
}

inline_re!(FOR_FIRST, r"for\s", I);
static DAY_ALONE: LazyLock<Re> =
    LazyLock::new(|| Re::new(&format!(r"^(?:on\s+)?{}$", APPOINTMENT_DAY.pattern()), I));
inline_re!(THE_MY_OUR_FIRST, r"^(?:the|my|our)\s+", 0);
inline_re!(IS_AT_ON_FOR_LAST, r"\s+(?:is|at|on|for)$", 0);
inline_re!(THREE_LETTERS, r"[a-z]{3}", 0);
static A_DIGIT_OR_MONTH: LazyLock<Re> =
    LazyLock::new(|| Re::new(&format!(r"\d|{}", data::string("bot.memory.facts.MONTH_NAMES")), I));
inline_re!(A_DAY_WITH_ITS_WORD, r"^(?:on|this|next|today|tomorrow|tonight)\b", I);

/// Python: bot/memory/facts.py:9459 `_appointment(text)`: the appointment in a line that names a
/// thing and a time and nothing else, or "".
pub fn _appointment(text: &str) -> String {
    let said = pt::strip_chars(&squash(text), " .,!;:-").to_string();
    let words = pt::split(&said);
    if words.is_empty() || said.contains('?') || words.len() > 12 {
        return String::new();
    }
    let low = lower(&said);
    if APPOINTMENT_OWN_SUBJECT.match_(&low).is_some() {
        return String::new();
    }
    if ASKING.match_(&low).is_some() || APPOINTMENT_PAST.is_match(&low) || NEGATED.is_match(&low) {
        return String::new();
    }
    if SECOND_PERSON.is_match(&low) {
        return String::new();
    }
    let mut clock = APPOINTMENT_CLOCK.search(&low);
    let day = APPOINTMENT_DAY.search(&low);
    if let Some(c) = &clock {
        if FOR_FIRST.match_(c.as_str()).is_some() && _A_COUNT_NOT_A_CLOCK.is_match(&low[..c.start()]) {
            clock = None;
        }
    }
    if clock.is_none() && day.is_none() {
        return String::new();
    }
    if let Some(c) = &clock {
        let after = pt::strip_chars(&low[c.end()..], " ,.;:-");
        if !after.is_empty() && DAY_ALONE.match_(after).is_none() {
            return String::new();
        }
    }
    let cut = [clock.as_ref().map(|m| m.start()), day.as_ref().map(|m| m.start())].into_iter().flatten().min().unwrap();
    let thing = pt::strip_chars(&low[..cut], " ,;:-").to_string();
    let thing = THE_MY_OUR_FIRST.sub("", &thing);
    let mut thing = pt::strip(&IS_AT_ON_FOR_LAST.sub("", &thing)).to_string();
    if thing.is_empty() {
        if let Some(d) = &day {
            if cut == d.start() {
                thing = _the_thing_said_after_the_day(&low, d);
            }
        }
    }
    let twords = pt::split(&thing);
    if thing.is_empty() || twords.len() > 4 {
        return String::new();
    }
    if _the_day_described(&thing) {
        return String::new();
    }
    if TAKE_BACK_WORDS.is_match(&thing) {
        return String::new();
    }
    if APPOINTMENT_NOT_A_THING.contains(&twords[0]) {
        return String::new();
    }
    if APPOINTMENT_CORRECTION.contains(pt::strip_chars(&twords[0], ",.!")) {
        return String::new();
    }
    if !THREE_LETTERS.is_match(&thing) {
        return String::new();
    }
    for word in &twords {
        if APPOINTMENT_PAST_VERB.contains(word) || APPOINTMENT_A_VERB.contains(word) {
            return String::new();
        }
        if pt::len(word) > 3 && word.ends_with("ed") && !APPOINTMENT_NOT_A_VERB.contains(word) {
            return String::new();
        }
    }
    if let Some(d) = &day {
        if A_DIGIT_OR_MONTH.is_match(d.as_str()) && APPOINTMENT_A_DATE_VERB.contains(pt::strip_chars(twords.last().unwrap(), ",.!")) {
            return String::new();
        }
    }
    let mut when: Vec<String> = Vec::new();
    if let Some(c) = &clock {
        when.push(format!("at {}", squash(c.str(1))));
    }
    if let Some(d) = &day {
        let mut day_text = squash(d.as_str());
        if A_DAY_WITH_ITS_WORD.match_(&day_text).is_none() {
            day_text = format!("on {day_text}");
        }
        when.push(day_text);
    }
    format!("{thing} {}", when.join(" "))
}

/// Python: bot/memory/facts.py:9560 `_coming_up(clause)`: true when the clause says something that
/// is still to come.
pub fn _coming_up(clause: &str) -> bool {
    if RECURRENCE.is_match(clause) {
        return false;
    }
    if FUTURE.is_match(clause) {
        return true;
    }
    if OBLIGATION.is_match(clause) {
        return APPOINTMENT_CLOCK.is_match(clause) || APPOINTMENT_DAY.is_match(clause);
    }
    A_CLOCKED_EVENT.is_match(clause) && APPOINTMENT_CLOCK.is_match(clause)
}

/// Python: bot/memory/facts.py:9576 `_dated_plan(text, deed=None)`: the plan in a line that puts
/// something in the future, however it is worded; a deed it passes on the way goes in `deed`.
pub fn _dated_plan(text: &str, deed: Option<&mut Vec<String>>) -> String {
    if !_coming_up(text) {
        return String::new();
    }
    _dated_plans_upto(text, deed, true).into_iter().next().unwrap_or_default()
}

inline_re!(OPENER_WORD_SPLIT, r"[^a-z']+", 0);

/// Python: bot/memory/facts.py:9620 `_the_opener_end(said)`: where the correction opener in front
/// of `said` ends (a byte offset), or 0 when it does not open with one.
pub fn _the_opener_end(said: &str) -> usize {
    let Some(lead) = CORRECTION_LEAD_RE.match_(said) else {
        return 0;
    };
    let front = lead.as_str();
    if OPENER_PUNCTUATION.is_match(front) {
        return lead.end();
    }
    let words: Vec<String> = OPENER_WORD_SPLIT
        .split(&lower(front))
        .into_iter()
        .filter(|w| !w.is_empty() && !OPENER_WORDS.contains(w))
        .collect();
    if words.len() >= 2 {
        lead.end()
    } else {
        0
    }
}

/// Python: bot/memory/facts.py:9634 `without_the_correction_opener(text)`: the line after the
/// opener that says the last thing is being said again, or "" when the line does not open with
/// one.
pub fn without_the_correction_opener(text: &str) -> String {
    let said = low_squash(text);
    let end = _the_opener_end(&said);
    if end == 0 {
        return String::new();
    }
    pt::strip_chars(&said[end..], " ,.;:\t\u{2013}\u{2014}").to_string()
}

/// Python: bot/memory/facts.py:9658 `is_a_correction_opener(text)`: true when the words are nothing
/// but the opener that announces a correction.
pub fn is_a_correction_opener(text: &str) -> bool {
    let said = low_squash(text);
    if said.is_empty() {
        return false;
    }
    let padded = format!("{said} ");
    CORRECTION_LEAD_RE.match_(&padded).is_some_and(|lead| lead.end() > said.len())
}

/// Python: bot/memory/facts.py:9675 `_said_after_the_subject(clause)`: what the clause says about
/// its subject, before the time or place it is nailed to.
pub fn _said_after_the_subject(clause: &str) -> String {
    let rest = match CLAUSE_SUBJECT.match_(clause) {
        Some(m) => &clause[m.end()..],
        None => clause,
    };
    let cut = match WHEN_OR_WHERE.search(rest) {
        Some(w) => &rest[..w.start()],
        None => rest,
    };
    pt::strip_chars(cut, " ,").to_string()
}

/// Python: bot/memory/facts.py:9684 `_the_verb_for(body, subject)`: the lent verb agreeing with the
/// person it now belongs to.
pub fn _the_verb_for(body: &str, subject: &str) -> String {
    let first = pt::split_ref(body).first().map(|w| lower(w)).unwrap_or_default();
    let person = lower(pt::strip(subject));
    let plural = ["we", "they", "you"].contains(&person.as_str()) || person.contains(" and ");
    if !plural {
        if first == "am" {
            return format!("is{}", pt::slice(body, 2, None));
        }
        if first == "are" || first == "were" {
            let verb = if first == "are" { "is" } else { "was" };
            return format!("{verb}{}", pt::slice(body, pt::len(&first) as isize, None));
        }
    }
    if person == "i" && first == "is" {
        return format!("am{}", pt::slice(body, 2, None));
    }
    body.to_string()
}

/// Python: bot/memory/facts.py:9700 `_with_the_verb_lent(clause, earlier)`: the clause with the verb
/// of the clause before it put back.
pub fn _with_the_verb_lent(clause: &str, earlier: &str) -> String {
    if clause.is_empty() || earlier.is_empty() || !_said_after_the_subject(clause).is_empty() {
        return clause.to_string();
    }
    let body = _said_after_the_subject(earlier);
    if A_SHARED_VERB.match_(&body).is_none() {
        return clause.to_string();
    }
    let m = CLAUSE_SUBJECT.match_(clause);
    let subject = m.as_ref().map(|m| m.as_str()).unwrap_or("");
    let body = _the_verb_for(&body, subject);
    let rest = match &m {
        Some(m) => pt::lstrip(&clause[m.end()..]),
        None => clause,
    };
    if rest.is_empty() {
        return clause.to_string();
    }
    let head = match &m {
        Some(m) => pt::rstrip(&clause[..m.end()]),
        None => "",
    };
    pt::strip(&format!("{head} {body} {rest}")).to_string()
}

inline_re!(DATED_PLAN_SPLIT, r"(?<=[.!?])\s+|,|;|\s+(?:but|and|so|then)\s+", 0);
inline_re!(OPENING_FILLERS, r"^(?:ok(?:ay)?[, ]+|and |so |but |also |well |oh )+", I);
inline_re!(A_PERSON_WORD_IN, r"\b(?:my|our|i|we|i'm|im|me)\b", I);
inline_re!(I_MY_WE_OUR, r"\b(?:i|my|we|our)\b", I);

/// Python: bot/memory/facts.py:9718 `_dated_plans(text, deed=None)`: every plan a line carries, in
/// the order it says them (Python's generator, consumed whole).
pub fn _dated_plans(text: &str, deed: Option<&mut Vec<String>>) -> Vec<String> {
    _dated_plans_upto(text, deed, false)
}

/// The generator `_dated_plans`, stopped after its first plan when `first_only` (as a caller that
/// returns on the first one stops it, so no deed after that plan is filed).
fn _dated_plans_upto(text: &str, mut deed: Option<&mut Vec<String>>, first_only: bool) -> Vec<String> {
    let mut out = Vec::new();
    if !FUTURE.is_match(text) && !_coming_up(text) {
        return out;
    }
    let mut at = 0usize;
    let mut earlier = String::new();
    for piece in DATED_PLAN_SPLIT.split(text) {
        let start = if piece.is_empty() { None } else { text[at.min(text.len())..].find(piece.as_str()).map(|i| i + at) };
        let ahead = match start {
            Some(s) => &text[..s],
            None => "",
        };
        if let Some(s) = start {
            at = s + piece.len();
        }
        let mut clause = pt::strip_chars(&squash(&piece), " ,.;:-!").to_string();
        if clause.is_empty() {
            continue;
        }
        let opened = pt::strip(&CORRECTION_LEAD_RE.subn("", &clause, 1)).to_string();
        if !opened.is_empty() && OWN.match_(&opened).is_some() {
            clause = opened;
        }
        let joined = _with_the_person_joined_to_the_subject(ahead, &clause);
        if !(OWN.match_(&joined).is_some() || OWN.match_(&clause).is_some() || THE_THING_HAS_A_DAY.match_(&clause).is_some())
            || joined.contains('?')
        {
            continue;
        }
        let clause = _with_the_verb_lent(&joined, &earlier);
        if !_coming_up(&clause) || PAST.is_match(&clause) {
            continue;
        }
        if ASKING.match_(&OPENING_FILLERS.sub("", &clause)).is_some() {
            continue;
        }
        if SECOND_PERSON.is_match(&clause) {
            continue;
        }
        if NEGATED.is_match(&clause) {
            continue;
        }
        if _how_they_are(&clause) || _how_their_things_are(&clause) {
            continue;
        }
        if a_deed(&clause) {
            if let Some(d) = deed.as_deref_mut() {
                d.push(clause.clone());
            }
            continue;
        }
        let words = pt::split(&clause);
        if words.len() < 3 || words.len() > 18 {
            continue;
        }
        if !(A_PERSON_WORD_IN.is_match(&clause) || THE_THING_HAS_A_DAY.match_(&clause).is_some()) {
            continue;
        }
        if I_MY_WE_OUR.findall(&clause).len() > 3 {
            continue;
        }
        earlier = clause.clone();
        out.push(clause);
        if first_only {
            break;
        }
    }
    out
}

/// Python: bot/memory/facts.py:9815 `_with_the_person_joined_to_the_subject(ahead, clause)`: the
/// clause with the person it was split from, when the split went through a subject.
pub fn _with_the_person_joined_to_the_subject(ahead: &str, clause: &str) -> String {
    let person = squash(ahead);
    if person.is_empty() || SUBJECT_JOIN.match_(&person).is_none() {
        return clause.to_string();
    }
    format!("{person} {clause}")
}

/// Python: bot/memory/facts.py:9859 `_continued_parts(text)`: the later clauses of a line about them
/// that leave their subject out, read as their own.
pub fn _continued_parts(text: &str) -> Vec<Value> {
    let mut clauses: Vec<&str> = Vec::new();
    let mut start = 0;
    for m in CONTINUATION_SPLIT.finditer(text) {
        clauses.push(&text[start..m.start()]);
        start = m.end();
    }
    clauses.push(&text[start..]);
    let words = pt::split(&clauses[0].replace(',', " "));
    let mut subject = if !words.is_empty() && PERSON_SUBJECT.contains(&lower(&words[0])) { "person" } else { "" };
    if subject == "person" && WISHING.is_match(&words.join(" ")) {
        return Vec::new();
    }
    let mut parts = Vec::new();
    for clause in &clauses[1..] {
        let clause = pt::strip_chars(clause, " .!?,");
        if clause.is_empty() {
            continue;
        }
        let first = pt::strip_chars(&lower(pt::split_ref(clause)[0]), ",").to_string();
        if ANY_SUBJECT.contains(&first) {
            subject = if PERSON_SUBJECT.contains(&first) { "person" } else { "other" };
            continue;
        }
        if subject != "person" || !CONTINUATION_VERB.contains(&first) {
            continue;
        }
        for lead in ["i ", "i'm "] {
            let mut read = _extract(&format!("{lead}{clause}"), false);
            read.del_key("name");
            let kept: Vec<(Value, Value)> =
                read.pairs().iter().filter(|(k, _)| CONTINUATION_KEYS.contains(k.str())).cloned().collect();
            if !kept.is_empty() {
                parts.push(Value::Dict(Arc::new(kept)));
                break;
            }
        }
    }
    parts
}

static RELATION_NAMED_RES: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    RELATIONS.iter().map(|r| (r.clone(), Re::new(&format!(r"\b(?:my|our)\s+{r}(?:'s|s)?\b"), I))).collect()
});

/// Python: bot/memory/facts.py:9922 `_the_relation_named(clause)`: the relation a clause names with
/// "my"/"our", or None.
pub fn _the_relation_named(clause: &str) -> Option<String> {
    RELATION_NAMED_RES.iter().find(|(_, re)| re.is_match(clause)).map(|(r, _)| r.clone())
}

/// Python: bot/memory/facts.py:9930 `_a_clause_about_the_same_person(text)`: the later clauses of a
/// line that say more about the person the clause before named.
pub fn _a_clause_about_the_same_person(text: &str) -> Vec<Value> {
    let clauses: Vec<String> = SAME_PERSON_SPLIT.split(text).into_iter().filter(|p| !pt::strip(p).is_empty()).collect();
    if clauses.len() < 2 {
        return Vec::new();
    }
    let mut named = _the_relation_named(&clauses[0]);
    if named.is_none() && !clauses[1..].iter().any(|c| _the_relation_named(c).is_some()) {
        return Vec::new();
    }
    let mut parts = Vec::new();
    for clause in &clauses[1..] {
        let Some(found) = SAME_PERSON_CLAUSE.match_(pt::strip(clause)) else {
            named = _the_relation_named(clause).or(named);
            continue;
        };
        let Some(who) = named.clone() else {
            continue;
        };
        let read = _extract(&format!("my {who} {}", pt::strip(found.name_str("rest"))), false);
        let mut kept: Vec<Value> = Vec::new();
        for entry in vget(&read, "others").items() {
            if !(entry.is_dict() && vget(entry, "who").truthy() && lower(&value_str(vget(entry, "who"))) == lower(&who)) {
                continue;
            }
            let entry: Vec<(Value, Value)> = entry.pairs().iter().filter(|(k, _)| k.str() != "name").cloned().collect();
            if entry.iter().any(|(k, v)| k.str() != "who" && v.truthy()) {
                kept.push(Value::Dict(Arc::new(entry)));
            }
        }
        if !kept.is_empty() {
            parts.push(dict(vec![("others", Value::from(kept))]));
        }
    }
    parts
}

inline_re!(SELF_INTRO_PIECES, r",|\band\b|;", 0);

/// Python: bot/memory/facts.py:9974 `_self_intro_parts(text)`: the list parts of a
/// self-introduction, each read as a line of its own.
pub fn _self_intro_parts(text: &str) -> Vec<Value> {
    let Some(m) = SELF_INTRO.match_(pt::strip(text)) else {
        return Vec::new();
    };
    let mut parts = Vec::new();
    for piece in SELF_INTRO_PIECES.split(m.name_str("rest")) {
        let piece = pt::strip_chars(&piece, " .!?").to_string();
        let words = pt::split(&piece);
        if words.is_empty() {
            continue;
        }
        let first = lower(&words[0]);
        if OTHER_START.contains(&first) || NOT_A_PART.contains(&first) {
            continue;
        }
        let leads: &[&str] = if PERSON_SUBJECT.contains(&first) { &[""] } else { &["i'm ", "i "] };
        for lead in leads {
            let read = _extract(&format!("{lead}{piece}"), false);
            if read.truthy() {
                parts.push(read);
                break;
            }
        }
    }
    parts
}

/// Python: bot/memory/facts.py:9997 `_same_thing(one, other)`: the same fact under a plural: "pets"
/// and "pet", "cats" and "cat".
pub fn _same_thing(one: &str, other: &str) -> bool {
    let one = pt::strip(&lower(&tidy(one))).to_string();
    let other = pt::strip(&lower(&tidy(other))).to_string();
    if one.is_empty() || other.is_empty() {
        return false;
    }
    one == other || pt::rstrip_chars(&one, "s") == pt::rstrip_chars(&other, "s")
}

/// Python: bot/memory/facts.py:10006 `_is_only_the_take_back(value, gone)`: "no pets" is not a
/// correction to something called "Pets".
pub fn _is_only_the_take_back(value: &Value, gone: &Value) -> bool {
    if !gone.truthy() {
        return false;
    }
    let value = value_str(value);
    for (fact, said) in gone.pairs() {
        if _same_thing(&value, &value_str(fact)) {
            return true;
        }
        let items: Vec<Value> = if said.is_list() && matches!(said, Value::List(_)) { said.items().to_vec() } else { vec![said.clone()] };
        for item in items {
            let item = if item.is_dict() {
                let animal = vget(&item, "animal");
                let name = vget(&item, "name");
                if animal.truthy() {
                    animal.clone()
                } else if name.truthy() {
                    name.clone()
                } else {
                    Value::from("")
                }
            } else {
                item
            };
            let s = value_str(&item);
            if s == EVERYTHING {
                return true;
            }
            if _same_thing(&value, &s) {
                return true;
            }
        }
    }
    false
}

inline_re!(A_NAME_AFTER_A_TITLE, r"\s+([A-Za-z][a-z'-]{1,19})\b", 0);

/// Python: bot/memory/facts.py:10069 `bot_name(text)`: the name they have just given the bot, or
/// None.
pub fn bot_name(text: &str) -> Option<String> {
    let m = BOT_NAME.search(text)?;
    if BOT_NAME_ASKED.is_match(&text[..m.start()]) || BOT_NAME_TAG.is_match(text) {
        return None;
    }
    let mut word = pt::strip(m.get(1).filter(|s| !s.is_empty()).or_else(|| m.get(2).filter(|s| !s.is_empty())).unwrap_or("")).to_string();
    if word.is_empty() || BOT_NAME_STOP.contains(&lower(&word)) || NOT_A_NAME.contains(&lower(&word)) {
        return None;
    }
    if BOT_NAME_TITLE.contains(&lower(&word)) {
        if let Some(more) = A_NAME_AFTER_A_TITLE.match_(&text[m.end()..]) {
            word = format!("{word} {}", more.str(1));
        }
    }
    if !pt::split(&word).iter().all(|part| NAME_WORD.match_(part).is_some()) {
        return None;
    }
    Some(if pt::is_lower(&word) { up1(&word) } else { word })
}

/// Python: bot/memory/facts.py:10114 `a_correction_without_a_subject(text, before)`: the fact a
/// subject-less correction states, read with the turn before's subject put back.
pub fn a_correction_without_a_subject(text: &str, before: &str) -> Value {
    let low = lower(pt::strip(text));
    if low.is_empty() || !TAKE_BACK.iter().any(|w| low.contains(w.as_str())) {
        return Value::dict();
    }
    let Some(opener) = CORRECTION_OPENER.match_(&low) else {
        return Value::dict();
    };
    let rest = pt::strip(&low[opener.end()..]).to_string();
    if rest.is_empty() || !TAKE_BACK.iter().any(|w| rest.contains(w.as_str())) {
        return Value::dict();
    }
    let before = lower(pt::strip(before));
    for (shape, frame) in SUBJECT_OPENERS.iter() {
        if shape.is_match(&before) {
            let found = extract(&format!("{frame} {rest}"), false);
            if found.truthy() {
                return found;
            }
        }
    }
    Value::dict()
}

/// Python: bot/memory/facts.py:10149 `_reported_clause(text)`: the thing a line reports having
/// said, when the line is only the reporting of it; None otherwise.
pub fn _reported_clause(text: &str) -> Option<String> {
    let text = pt::strip(text);
    if text.is_empty() || text.contains('?') {
        return None;
    }
    if CORRECTION_SAID.is_match(text) || CORRECTION.is_match(text) || CORRECTION_PAIR.is_match(text) || CORRECTION_MAKE.is_match(text) {
        return None;
    }
    let m = REPORTED.match_(text)?;
    let rest = pt::strip_chars(m.name_str("rest"), " ,.;:-").to_string();
    if rest.is_empty() {
        None
    } else {
        Some(rest)
    }
}

/// Python: bot/memory/facts.py:10164 `_paid_by_a_period(text, end)`: whether a period follows the
/// figure, with the household allowed in the gap. (`end` is a byte offset.)
pub fn _paid_by_a_period(text: &str, end: usize) -> bool {
    let qualifier = INCOME_QUALIFIER.match_(&text[end..]);
    let at = end + qualifier.map(|q| q.end()).unwrap_or(0);
    match_at(&INCOME_PERIOD, text, at).is_some()
}

inline_re!(A_FIGURE_IN_THOUSANDS, r"\d[\d,.]*\s?(?:k|grand|thousand)\b", I);
inline_re!(NOT_A_DIGIT, r"[^\d]", 0);

/// Python: bot/memory/facts.py:10175 `_a_years_pay(figure)`: whether a figure with no period is a
/// year's wage rather than a count.
pub fn _a_years_pay(figure: &str) -> bool {
    if A_FIGURE_IN_THOUSANDS.is_match(figure) {
        return true;
    }
    pt::len(&NOT_A_DIGIT.sub("", figure)) >= 4
}

/// Python: bot/memory/facts.py:10188 `extract(text, question=False)`: facts this line states about
/// the user (only the keys it states).
pub fn extract(text: &str, question: bool) -> Value {
    let mut text = text.to_string();
    if let Some(reported) = _reported_clause(&text) {
        if _extract(&reported, question).truthy() {
            text = reported;
        }
    }
    let text = text.as_str();
    let mut found = _extract(text, question);
    if let Some(given) = bot_name(text) {
        found.set_key("bot_name", given);
    }
    if a_limit_on_the_plan(text).is_some() && !COMMITS_TO.is_match(text) {
        found.del_key("plan");
        found.del_key("plans");
        if let Some(correction) = found.get("correction") {
            if correction.is_dict() && !vget(correction, "from").truthy() {
                found.del_key("correction");
            }
        }
    }
    let mut parts = _self_intro_parts(text);
    parts.extend(_continued_parts(text));
    parts.extend(_a_clause_about_the_same_person(text));
    for part in parts {
        for (key, value) in part.pairs() {
            let key = key.str();
            if key == "name" {
                continue;
            }
            if key == "places" || key == "others" {
                let have: Vec<Value> = found.get(key).map(|v| v.items().to_vec()).unwrap_or_default();
                let mut merged = _union(&have, value.items());
                if key == "others" {
                    merged = _one_entry_per_person(&merged);
                }
                found.set_key(key, Value::from(merged));
            } else if found.get(key).is_none() {
                found.set_key(key, value.clone());
            }
        }
    }
    if vget(&found, "others").truthy() {
        let mut others: Vec<Value> = vget(&found, "others").items().to_vec();
        for entry in others.iter_mut() {
            if entry.is_dict() {
                if let Some(who) = entry.get("who").and_then(Value::as_str).map(str::to_string) {
                    if let Some(full) = SPOKEN_RELATION.get(&who) {
                        entry.set_key("who", full.as_str());
                    }
                }
            }
        }
        found.set_key("others", Value::from(_one_entry_per_person(&others)));
    }
    if !vget(&found, "correction").truthy() && found.get("drop").map_or(true, Value::is_none) {
        if let Some(marked) = _the_marker_in_front(text, question) {
            let again = _extract(&marked, question);
            for key in ["correction", "drop"] {
                if let Some(v) = again.get(key) {
                    if !v.is_none() {
                        found.set_key(key, v.clone());
                    }
                }
            }
        }
    }
    found
}

/// Python: bot/memory/facts.py:10263 `about_themselves(said)`.
///
/// CONTRACT NOTE: Python returns the facts (a dict) or None; the skeleton gave this
/// `Option<Vec<String>>`. This keeps the contract and returns the keys of the facts;
/// `about_themselves_facts` returns the facts themselves.
pub fn about_themselves(said: &str) -> Option<Vec<String>> {
    about_themselves_facts(said).map(|kept| kept.pairs().iter().map(|(k, _)| k.str().to_string()).collect())
}

/// `about_themselves(said)`: the facts a line says about the person themselves, or None when it
/// says none about them.
pub fn about_themselves_facts(said: &str) -> Option<Value> {
    let words: Vec<String> = pt::split(said).iter().map(|w| lower(w)).collect();
    if !SAYS_WHAT_THEY_ARE.iter().any(|lead| words.len() >= lead.len() && words[..lead.len()] == lead[..]) {
        return None;
    }
    let kept = extract(said, false);
    let keys: Vec<&str> = kept.pairs().iter().map(|(k, _)| k.str()).collect();
    if !kept.truthy() || keys.iter().all(|k| THEIR_NAME.contains(*k)) || keys.iter().all(|k| THEIR_PLAN.contains(*k)) {
        return None;
    }
    Some(kept)
}

/// `str(value or "")[:1]`'s case.
fn first_is_lower(s: &str) -> bool {
    s.chars().next().is_some_and(char::is_lowercase)
}
fn first_is_upper(s: &str) -> bool {
    s.chars().next().is_some_and(char::is_uppercase)
}

/// Python: bot/memory/facts.py:10281 `_one_entry_per_person(entries)`: one entry per person: a line
/// that says the same thing twice is still one person.
pub fn _one_entry_per_person(entries: &[Value]) -> Vec<Value> {
    let mut out: Vec<Value> = Vec::new();
    for entry in entries {
        if !entry.is_dict() {
            continue;
        }
        let who = lower(&str_or_empty(entry.get("who")));
        let mut kept: Option<usize> = None;
        if !who.is_empty() && !vget(entry, "pronoun").truthy() {
            for (i, one) in out.iter().enumerate() {
                if lower(&str_or_empty(one.get("who"))) != who || vget(one, "pronoun").truthy() {
                    continue;
                }
                let mine = lower(&str_or_empty(one.get("name")));
                let theirs = lower(&str_or_empty(entry.get("name")));
                if !mine.is_empty() && !theirs.is_empty() && mine != theirs {
                    continue;
                }
                kept = Some(i);
                break;
            }
        }
        let Some(i) = kept else {
            out.push(entry.clone());
            continue;
        };
        for (key, value) in entry.pairs() {
            let key = key.str();
            let one = &mut out[i];
            if key == "who" || key == "pronoun" {
                continue;
            }
            if key == "name" {
                let mine = str_or_empty(one.get("name"));
                if mine.is_empty() || (first_is_lower(&mine) && first_is_upper(&value_str(value))) {
                    one.set_key("name", value.clone());
                }
                continue;
            }
            if key == "age" {
                if *vget(one, "to_come") == *value {
                    continue;
                }
                if vget(one, "turns").truthy() && !vget(one, "age").truthy() {
                    one.set_key("age", value.clone());
                    continue;
                }
                if let (Some(have), Some(said)) = (vget(one, "age").as_int(), value.as_int()) {
                    if have != said {
                        if !vget(one, "turns").truthy() {
                            one.set_key("age", value.clone());
                        }
                        continue;
                    }
                }
                one.set_key("age", value.clone());
                continue;
            }
            if key == "turns" || key == "to_come" {
                if !vget(one, "turns").truthy() {
                    one.set_key("turns", value.clone());
                    if key == "to_come" && *vget(one, "age") != *value {
                        one.set_key("to_come", value.clone());
                    }
                }
                continue;
            }
            if !value.is_list() {
                one.setdefault(key, value.clone());
                continue;
            }
            let have = one.setdefault(key, Value::list());
            if matches!(have, Value::List(_)) {
                for item in value.items() {
                    if !have.items().contains(item) {
                        have.push(item.clone());
                    }
                }
            }
        }
    }
    out
}

/// Python: bot/memory/facts.py:10543 `_a_name_and_a_figure(text)`: {"name": "Ada", "age": 8} for
/// "ada is 8", "no sam is 5", "ada's 8 now" - or None.
pub fn _a_name_and_a_figure(text: &str) -> Option<Value> {
    let said = squash(text);
    let m = NAME_AND_A_FIGURE.match_(&said)?;
    let name = m.name_str("name");
    let low = lower(name);
    if NOT_A_NAME_FIGURE.contains(&low)
        || NOT_A_NAME.contains(&low)
        || NOT_A_NAME_THING.contains(&low)
        || NOT_A_NAME_STATE.contains(&low)
        || MONTHS.contains(&low)
        || DAYS.contains(&low)
    {
        return None;
    }
    let age = int_of(m.name_str("age"));
    if !(0 < age && age <= 120) {
        return None;
    }
    Some(dict(vec![("name", up1(name).into()), ("age", age.into())]))
}

inline_re!(BODY_CLAUSE_SPLIT, r"[,;.!?]|\s+(?:and|but|so|then)\s+", 0);
inline_re!(A_SELF_WORD, r"\b(?:my|our|i|me|mine|myself)\b", 0);

/// Python: bot/memory/facts.py:10631 `_tied_to_them(part, low)`: whether the line ties the part (a
/// byte span) to the person typing it, or only talks about parts.
pub fn _tied_to_them(part: (usize, usize), low: &str) -> bool {
    let before = &low[..part.0];
    let after = &low[part.1..];
    if MY_OWN_PART.is_match(before) {
        return true;
    }
    if THE_PART_HURTS.match_(after).is_some() {
        return true;
    }
    let clause = BODY_CLAUSE_SPLIT.split(before).pop().unwrap_or_default();
    if THE_PERSON_SAYS.match_(pt::strip(&clause)).is_some() && BODY_HURTS.is_match(low) {
        return true;
    }
    A_PLACE_ON_IT.match_(low).is_some() && BODY_PLACE.is_match(low)
}

/// Python: bot/memory/facts.py:10656 `_theirs(part, low)`: whether the part at this place (a byte
/// span) in the line is the speaker's own body.
pub fn _theirs(part: (usize, usize), low: &str) -> bool {
    let part_at = part.0;
    let mut owner: Option<(usize, usize)> = None;
    for found in SOMEONE_ELSE.finditer(low) {
        if found.start() < part_at {
            owner = Some((found.start(), found.end()));
        }
    }
    if let Some((_, owner_end)) = owner {
        let between = if owner_end <= part_at { &low[owner_end..part_at] } else { "" };
        if !A_SELF_WORD.is_match(between) {
            return false;
        }
    }
    let next_char = low[part_at..].chars().next().map(|c| part_at + c.len_utf8()).unwrap_or(part_at);
    for other in A_PART_HANDED_OVER.finditer(low) {
        if other.end() <= next_char && other.start() < part_at {
            return false;
        }
    }
    _tied_to_them(part, low)
}

inline_re!(
    A_BODY_WORD_BEFORE_A_THING_TAIL,
    r"\s+(?:door|room|garden|yard|seat|window|wheel|teacher|line|table|of the house)\b",
    0
);

/// Python: bot/memory/facts.py:10679 `the_part_it_hurts(text)`: the part of them that is sore, and
/// where on it, or None.
pub fn the_part_it_hurts(text: &str) -> Option<Value> {
    let low = low_squash(text);
    let mut part: Option<Match> = None;
    for found in BODY_PART.finditer(&low) {
        if _theirs((found.start(), found.end()), &low) {
            part = Some(found);
            break;
        }
    }
    let part = part?;
    let lateral = BODY_LATERAL.search(&low);
    let place = BODY_PLACE.search(&low);
    if !(BODY_HURTS.is_match(&low) || lateral.is_some() || place.is_some()) {
        return None;
    }
    let word = crate::py::re::escape(part.str(1));
    if Re::new(&format!(r"\b(?:no|not|never)\b[^.]{{0,20}}{word}\b"), 0).is_match(&low) {
        return None;
    }
    if lateral.is_none() && place.is_none() && Re::new(&format!(r"\b{word}{}", A_BODY_WORD_BEFORE_A_THING_TAIL.pattern()), 0).is_match(&low) {
        return None;
    }
    let mut held = dict(vec![("part", lower(part.str(1)).into()), ("said", squash(text).into())]);
    if let Some(l) = &lateral {
        held.set_key("side", lower(l.str(1)));
    }
    if let Some(p) = &place {
        held.set_key("place", lower(p.str(1)));
    }
    Some(held)
}

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























































































































































































































































    /// Python: bot/memory/facts.py:10720 `_extract(text, question=False)`
    pub fn _extract(text: &str, question: bool) -> Value {
        crate::unported!("bot.memory.facts:_extract")
    }

}

