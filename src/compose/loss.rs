//! Port of `bot/compose/loss.py`: a death or a loss is never answered as small talk.
//!
//! The tables, word lists, reply pools and compiled patterns are read from the exported bundle
//! by their Python names; the functions follow the Python file in its order.
#![allow(non_snake_case, non_upper_case_globals, clippy::all)]

use crate::compose::fallback_chain::{caught, py_format};
use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// Compiled patterns of this module's namespace, by their Python names.
macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.loss.", stringify!($name))));)*
    };
}
/// String sets (frozensets) of this module's namespace, for membership tests.
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.compose.loss.", stringify!($name))));)*
    };
}
/// String tuples (reply pools, phrase lists) of this module's namespace, in order.
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.loss.", stringify!($name))));)*
    };
}

regexes!(
    ROLE_PHRASE, THING_ROLE, PLAIN, A_PRICE, PROCEEDINGS_ASK, THIS_ONE, A_DEATH_HAPPENED, CORRECTS,
    WAKE, BURIED, DEATH_NOUN, OWN_DEATH, DEATH_CUE, DEATH_COMPOUND, FICTION, OWN_WORRY, PET_DEATH,
    THING_DEATH, DEATH_OF_OWNER, DIED, DYING, PUT_DOWN, MISADVENTURE, SURVIVED, OWNED, VERB_OBJECT,
    KNOCKED_BY, DEATH_SAID, LOSS_OF, LOSING, NEWS_OF, LOSS_GRIEF_CUE, LOST_IN_A_PLACE, PREY,
    HUNTING, LOST_THE, LOST_THE_FOUND, STILL_CRY, OTHER_WEEPS, THEIR_TEARS_IN_A_CLAUSE,
    DEATH_AS_A_THING, NOT_A_DEATH, TOOK, KILLED, KILLED_IN, GAME_TITLES, WOULD_HAVE_BEEN,
    GOING_TO_DIE, CONDITIONAL_TAIL, A_RELATION_IN_A_HARD_TIME, SOMEONE_ELSES, HIS_HERS,
    SOMEONE_ELSES_OWN, A_STRANGERS_ANIMAL, CARD, EULOGY_WORD, EULOGY_ASK, CARD_TO_WRITE,
    WRITE_WORD, WEAR, ASK_WORDS, QUESTION, GRIEF_ASK_FORM, PRACTICAL, GRIEF_CUE, AT_THE_END,
    DYING_NOW, ANTICIPATED, DYING_ALREADY, _WORKING_WITH_THE_DYING, _SAID_DYING,
    THE_GIVE_UP_NAMES, SAID_IT, NOT_A_CARE_GIVER, FUNERAL_COST_ELSE, A_PART_OF_THE_DAY,
    SAY_THE_WORDS, _DEATH_IN_LINE, FUNERAL_COST, _THE_ESTATE, FUNERAL_HELP, HELP_ASK, QUIETER_ASK,
    FUNERAL_COST_COMPARE, _BURIAL_SIDE, _CREMATION_SIDE, _PRONOUN_MONEY_ASK, _BILL_SUBJECT,
    _BILL_MONEY, ACKNOWLEDGE, OWN_STATE, OWN_PET, CLAUSE, PUTTING_REGRETTED, NOT_YET, ESTIMATE,
    NOTHING_MORE_TO_DO, TREATMENT_STOPPED, MEDICATION_STOPPED, ON_THE_LAST_DRUG, SELF_ON_THE_DRUG,
    CARE_WHO, DAYS_LEFT, NOT_THE_END, FINAL_DAYS, COULD_BE_DAYS, DAYS_WAIT_FOR, LAST_STAGES,
    NOT_GETTING_BETTER, GIVEN_UP_ON, PREPARE, PREPARE_OTHERWISE, OFF_LIFE_SUPPORT,
    OFF_SUPPORT_WELL, TOLD_US, NOT_WORKING, SERIOUS_CARE, HOSPICE_CARE, HOSPICE_JOB,
    PALLIATIVE_ITSELF, DYING_OTHER, ASK_BARE, PRACTICAL_ASK, ADMINISTRATION, DEFINES, THE_DEATH,
    OPEN_ASK, GET_THROUGH, GET_THROUGH_TO, GET_THROUGH_DAY, GET_THROUGH_OBJECT, BODY_FEELING,
    WHEN_ASK, ASKS_ABOUT_THE_FEELING, SYMPATHY, CARD_WORD, ASK_FORM, ASK_WE, ASK_TAIL,
    PET_ARRANGEMENT, PET, PET_HAS_HAD_ENOUGH, SPEAKER_HAS_HAD_ENOUGH, SPEAKER_TOO, WISH_FRAME,
    PET_AT_THE_END, HELLO_ON_ITS_OWN, AN_ENDING, A_FEELING_BETTER_ASK, A_SPAN, A_SPAN_NO_NUMBER,
    A_SPAN_LEAD, A_SPAN_TAIL, A_SPAN_ELSEWHERE, MISS_VERB, MISS_NOUN, MISS_NOT_BETWEEN, MISS_TAIL,
    MISS_PRONOUN, MISS_NOT, ASK_AT_START, FEELING_ASK, GRIEF_POINTS_BACK, REFERS_TO_THEM, IDEA_ASK,
    ABSENCE, REASSURING, PEACE_IS_MINE, WOULD_HAVE, OCCASION, WITHOUT, TRADITION, STILL_KEEP,
    DAY_THING, DAY_QUESTION, FUNERAL, FUNERALS, TELL_CHILD, YEAR_OLD, ASKS_ABOUT_THE_DAY,
    TIME_OF_DAY, THEMSELVES, ASK_NORMAL, NORMAL_TO_FEEL, QUANTITY, STILL_ABOUT_THEM,
    ABOUT_THEM_OPENS, MISSING_SAID_AGAIN, _CONTRACTED, DYING_TO, LOST_THING_TO_FIND,
    CANT_FIND_THING, LEFT_THING, REMEMBER_NOT_WHERE, KEEP_LOSING_THING, KEPT_GOING_MISSING,
    GONE_THING, WHERE_IS_THING, WHERE_I_PUT_IT, WHERE_IS_THING_BARE, LOCKED_OUT, NOT_A_DOOR,
    CAR_LOCK_OUT, FOUND_WHAT_WAS_LOST, SHED, FEARED_NOT_YET, PET_DIED, IT_DIED, CHILDS_OWN_LOSS,
    DEATH_WORD, SWAP_ASK, ASKS, A_SWAP_INHERITS, _HELPERS, KEEPING_JUDGED, ENDING_ASKED,
    SAY_GOODBYE, SAY_TO_WHO, RITUAL, RITUAL_PERSON, PAPER_ASK, GIFT_THING_ASK, IN_CARE,
    NOT_A_BEDSIDE,
);

word_sets!(
    PEOPLE, TWO_WAY_ROLES, THEIRS, THINGS, PET_WORDS, HUMAN_WORDS, DEATHABLE, MACHINE_ANIMALS,
    CLOSE, PLACES, UNNAMED, MEDIA, LOST_SELF, STORY_MEDIUM, STORY_INCIDENT, PASSED_ON_SUBJECT,
    PASSED_ON_OBJECT, MONTHS, PASSED_ALONE_TAIL, PASSED_ALONE_HANDED, PASSED_ALONE_NOUN,
    PASSED_A_RESULT, PASSED_DATE, PASSED_TIME, OBJECTS, DEAD_IDIOM, DEAD_PREDICATE,
    PEOPLE_AFTER_A_GIVE_UP, _NOT_A_BILL_THREAD, GET_THROUGH_FILLER, GET_THROUGH_THE_LOSS, FILLERS,
    MISS_ANIMALS, MISS_ANIMAL_NOUNS, FEELING_STATES, FEELING_ONLY_WORDS, FEELING_PERSON_WORDS,
    GRIEF_SELF, FAMILY, SELF_NOUNS, WHEN_WORDS, THREAD_TOPICS, GRIEF_FEELINGS, PRONOUN, STATE_TAIL,
    BODY_STATE, MEASURE_TOPICS, BODY_ANSWER, _PEOPLE_PRONOUNS, _NOT_A_CONTRACTION, _CAN_BE_DYING,
    NOT_MISLAID, NOT_A_THING_NEXT, VISIT_TARGETS,
);

pools!(
    BURIED_OBJECT, BURIED_ADJECTIVE, LOSS_CONTEXT, BEDTIME, LAUGHING, THEIR_HARD_TIME_REPLIES,
    SOMEONE_DYING, THE_DAYS_BEFORE, FIRST_DAYS_PERSON, FIRST_DAYS_PET, DYING_REPLIES,
    PUT_DOWN_REGRET_REPLIES, LOSS_REPLIES, DYING_SAY_REPLIES, DYING_GRIEF_REPLIES,
    GETTING_THROUGH_REPLIES, GETTING_THROUGH_DYING_REPLIES, REMEMBER_REPLIES, MISLAID_REPLIES,
    SAY_REPLIES, NAMED_LOSS_SAY_REPLIES, CARD_REPLIES, GRIEF_REPLIES, PET_GRIEF_REPLIES,
    FUNERAL_REPLIES, FUNERAL_COST_COMPARE_REPLIES, FUNERAL_HELP_REPLIES, FUNERAL_WHEN_REPLIES,
    SPEAKER_TOO_REPLIES, HELLO_AFTER, TIME_SINCE_REPLIES, ENDING_SINCE_REPLIES,
    ENDING_FEELINGS_REPLIES, MISS_REPLIES, SELF_OPENERS, GRIEF_REFERRING, GRIEF_OBJECTS,
    AFTER_REPLIES, PASSIVE_AUX, REMAINS, BURIABLE, RELATIVE_PET_REPLIES, STRANGER_ANIMAL_REPLIES,
    KEEPING_GOING_REPLIES, IS_IT_TIME_REPLIES, SAFETY_MODULES, FALLBACK_MODULES,
);

static PASSED_ON: LazyLock<String> = LazyLock::new(|| data::string("bot.compose.loss.PASSED_ON"));
static PASSED_ALONE: LazyLock<String> = LazyLock::new(|| data::string("bot.compose.loss.PASSED_ALONE"));

fn int_named(name: &str) -> i64 {
    data::get(name).as_int().unwrap_or_else(|| panic!("{name} is not an int"))
}
static LOSS_STILL_THE_THREAD: LazyLock<i64> = LazyLock::new(|| int_named("bot.compose.loss.LOSS_STILL_THE_THREAD"));
static SWAP_MOST_WORDS: LazyLock<i64> = LazyLock::new(|| int_named("bot.compose.loss.SWAP_MOST_WORDS"));
static THREAD_TURNS: LazyLock<i64> = LazyLock::new(|| int_named("bot.compose.loss.THREAD_TURNS"));
static ENDING_TURNS: LazyLock<i64> = LazyLock::new(|| int_named("bot.compose.loss.ENDING_TURNS"));

/// Pairs of words (`PEOPLE_PHRASES`, `TWO_WORD_PETS`).
fn word_pairs(name: &str) -> Vec<(String, String)> {
    data::get(name)
        .items()
        .iter()
        .map(|pair| (pair.items()[0].str().to_string(), pair.items()[1].str().to_string()))
        .collect()
}
static PEOPLE_PHRASES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| word_pairs("bot.compose.loss.PEOPLE_PHRASES"));
static TWO_WORD_PETS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| word_pairs("bot.compose.loss.TWO_WORD_PETS"));

/// `LOST_FORMS`: the patterns that read "lost", each with the group that holds what was lost.
static LOST_FORMS: LazyLock<Vec<(&'static Re, usize)>> = LazyLock::new(|| {
    data::get("bot.compose.loss.LOST_FORMS")
        .items()
        .iter()
        .map(|pair| {
            let pattern = pair.items()[0].as_re().expect("LOST_FORMS: a pattern");
            (pattern, pair.items()[1].as_int().expect("LOST_FORMS: a group") as usize)
        })
        .collect()
});

/// `HABIT_PATTERNS`: each habit's pattern with the pool that answers it.
static HABIT_PATTERNS: LazyLock<Vec<(&'static Re, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.loss.HABIT_PATTERNS")
        .items()
        .iter()
        .map(|pair| (pair.items()[0].as_re().expect("HABIT_PATTERNS: a pattern"), pair.items()[1].strs()))
        .collect()
});

/// `ADMINISTRATION_KEYS`: (key, pattern) rows, the pattern compiled as `re.search` would.
static ADMINISTRATION_KEYS: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    data::get("bot.compose.loss.ADMINISTRATION_KEYS")
        .items()
        .iter()
        .map(|pair| (pair.items()[0].str().to_string(), Re::new(pair.items()[1].str(), 0)))
        .collect()
});

static FEELING_WORDS: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.compose.loss.FEELING_WORDS"));
static FEELING_REPLIES: LazyLock<HashMap<String, Vec<String>>> =
    LazyLock::new(|| data::get("bot.compose.loss.FEELING_REPLIES").str_list_map());
static PLURAL_PAIRS: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.compose.loss.PLURAL_PAIRS"));

/// `POOLS`: every line of every pool that keeps the thread open (`any(last in pool for pool in POOLS)`).
static POOL_LINES: LazyLock<HashSet<String>> = LazyLock::new(|| {
    data::get("bot.compose.loss.POOLS").items().iter().flat_map(|pool| pool.strs()).collect()
});

// Patterns the Python writes inline in a function.
static TELL_OR_EXPLAIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:tell|explain)\w*\b", 0));
static NOT_A_CHILD_TOLD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:about|of|people|everyone|office|work|boss)\b", 0));
static MISSED_NAME: LazyLock<Re> = LazyLock::new(|| Re::new(r"([A-Za-z]+)([.!?]?)(.*)$", 0));
static AN_ASK_INSIDE: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"\b(?:should|shall|do|does|did|can|could|would|will|is|are|was|were|",
            r"why|how|what|when|where|who)\s+(?:i|we|you|he|she|they|it)\b"
        ),
        0,
    )
});
static LOST_IT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\blost\s+it\b", 0));
static LOST_THE_THING: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:lost|losing|lose)\s+the\s+([a-z]+)", 0));
static WHO_WHERE_WHICH: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s*(?:who|who's|whos|whose|where|which)\b", 0));

// ---------------------------------------------------------------------------------------------
// Small Python idioms used all through the file.
// ---------------------------------------------------------------------------------------------

/// `set(words) & names` is not empty.
fn any_in(words: &[String], names: &HashSet<String>) -> bool {
    words.iter().any(|word| names.contains(word))
}

/// `word in words` for a list of words.
fn has(words: &[String], word: &str) -> bool {
    words.iter().any(|w| w == word)
}

/// `any(phrase in text for phrase in phrases)`.
fn any_phrase_in(text_: &str, phrases: &[String]) -> bool {
    phrases.iter().any(|phrase| text_.contains(phrase.as_str()))
}

/// `str(line or "").lower().replace("’", "'")`.
fn lowered(line: &str) -> String {
    text::lower(line).replace('\u{2019}', "'")
}

/// `seq[-n:]` for a count that Python negates to slice from the end.
fn last_n<T>(seq: &[T], n: i64) -> &[T] {
    let len = seq.len() as i64;
    let start = if n > 0 { (len - n).max(0) } else { (-n).min(len) };
    &seq[start as usize..]
}

/// `line.startswith(("what", "how", "should", "can", "do", "does", "is", "why", "when"))`.
fn opens_with_an_ask(line: &str) -> bool {
    ["what", "how", "should", "can", "do", "does", "is", "why", "when"].iter().any(|w| line.starts_with(w))
}

/// `str(value or "")` for a value kept in memory.
fn py_str(value: Option<&Value>) -> String {
    match value {
        Some(Value::Str(s)) => s.to_string(),
        Some(Value::Int(n)) if *n != 0 => n.to_string(),
        Some(Value::Float(x)) if *x != 0.0 => text::float_repr(*x),
        Some(Value::Bool(true)) => "True".to_string(),
        _ => String::new(),
    }
}

/// The first character is a digit (`word[:1].isdigit()`).
fn opens_with_a_digit(word: &str) -> bool {
    word.chars().next().is_some_and(|c| text::is_digit(&c.to_string()))
}

// ---------------------------------------------------------------------------------------------

/// Python: bot/compose/loss.py:150 `_only_a_new_subject(line)`
/// True when the line only puts a different thing in place of the earlier one.
pub fn _only_a_new_subject(line: &str) -> bool {
    let stripped = text::strip(line);
    let Some(found) = CORRECTS.match_(stripped) else { return false };
    let rest = text::strip_chars(found.name_str("rest"), " .!?,");
    !rest.is_empty() && text::split_ref(rest).len() <= 3 && !DIED.is_match(rest)
}

/// Python: bot/compose/loss.py:159 `_only_the_proceedings(clean, words)`
/// True when the line only asks about funerals as a thing, so the knowledge base answers it.
pub fn _only_the_proceedings(clean: &str, words: &[String]) -> bool {
    if A_DEATH_HAPPENED.is_match(clean) || _has_person(words) {
        return false;
    }
    if _unnamed_person(words, clean) || THIS_ONE.is_match(clean) {
        return false;
    }
    if ASK_WORDS.is_match(clean) || CARD_WORD.is_match(clean) {
        // "what do i write in a condolence card" asks for words, and the card pool has them.
        return false;
    }
    A_PRICE.is_match(clean) || PROCEEDINGS_ASK.is_match(clean)
}

/// Python: bot/compose/loss.py:505 `_killed_in_a_story(clean)`
/// True when the thing "killed in/on" points at is a story, not an incident.
pub fn _killed_in_a_story(clean: &str) -> bool {
    if GAME_TITLES.is_match(clean) {
        return true;
    }
    let Some(found) = KILLED_IN.search(clean) else { return false };
    for word in text::split_ref(&clean[found.end()..]).into_iter().take(4) {
        if STORY_INCIDENT.contains(word) {
            return false;
        }
        if STORY_MEDIUM.contains(word) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/loss.py:619 `_dead_is_not_a_death(words, index)`
/// True when the "dead" at `index` is not a death: it describes the word after it ("dead
/// birds") or the two words are an idiom ("dead easy", "the battery is dead").
pub fn _dead_is_not_a_death(words: &[String], index: usize) -> bool {
    if index + 1 >= words.len() {
        return false;
    }
    let following = &words[index + 1];
    DEAD_IDIOM.contains(following) || !DEAD_PREDICATE.contains(following)
}

/// Python: bot/compose/loss.py:905 `is_dying_now(text)`
/// True when someone of ours is dying but not dead: "my dad is dying", "she's terminal".
///
/// Kept apart from `reads_as_loss` on purpose - the reading is the reader's, and this only
/// decides which wording fits. A pet dying counts: the dog in front of them has not gone yet.
pub fn is_dying_now(text_: &str) -> bool {
    let said = _spoken(text_);
    let clean = _tight(&said);
    let words = words_of(&said);
    if !(DYING_NOW.is_match(&clean) || _the_end_is_near(&clean, Some(&words)) || AT_THE_END.is_match(&clean)) {
        return false;
    }
    if DYING_ALREADY.is_match(&clean) {
        return false;
    }
    if _WORKING_WITH_THE_DYING.is_match(&clean) && !_SAID_DYING.is_match(&clean) {
        return false; // "my mum is a palliative care nurse" is a job, not a father in the next room
    }
    if _the_end_is_near(&clean, Some(&words)) && !DYING_NOW.is_match(&clean) {
        // The give-up names the doctor or the hospital rather than the person, and the person is
        // the one the chat is about: "the consultant said its a matter of time" is somebody dying.
        return true;
    }
    if _the_family_is_told(&clean) {
        // The strongest of those sentences carry no person at all: "theyve said its palliative
        // now", "the oncologist said the treatment isnt working anymore".
        return true;
    }
    words.iter().any(|w| HUMAN_WORDS.contains(w) || PET_WORDS.contains(w) || _PEOPLE_PRONOUNS.contains(w))
        || _a_role_phrase(&clean)
}

/// Python: bot/compose/loss.py:949 `dying_in_the_chat(memory, turns=6)`
/// True when a line earlier in this chat said that someone of ours is dying.
pub fn dying_in_the_chat(memory: &Memory, turns: i64) -> bool {
    last_n(&memory.turns, turns).iter().any(|said| is_dying_now(&said.user))
}

/// Python: bot/compose/loss.py:966 `_a_pet_of_ours(line, words=None)`
/// True when the one dying is an animal, so the vet has the timeline and not a hospice.
pub fn _a_pet_of_ours(line: &str, words: Option<&[String]>) -> bool {
    let own;
    let words = match words {
        Some(words) => words,
        None => {
            own = words_of(line);
            &own
        }
    };
    any_in(words, &PET_WORDS) && !any_in(words, &HUMAN_WORDS)
}

/// Python: bot/compose/loss.py:974 `_the_end_is_near(tight, words=None)`
/// True when the line says the treatment has run out or the days are counted.
///
/// These are the sentences somebody's dying arrives in, and none of them has a death word in it:
/// "the doctor said theres nothing more they can do", "theyve stopped mums chemo", "my dad has
/// weeks left". The who is what keeps the reading honest: a give-up is about somebody dying when
/// a doctor, a vet or a hospital says it, while the council, the mechanic and the bank say the
/// same sentence about a noise, a car and a loan and each has an entry that answers it.
pub fn _the_end_is_near(tight: &str, words: Option<&[String]>) -> bool {
    let own;
    let words = match words {
        Some(words) => words,
        None => {
            own = words_of(tight);
            &own
        }
    };
    let who = CARE_WHO.is_match(tight);
    if who
        && (NOTHING_MORE_TO_DO.is_match(tight)
            || TREATMENT_STOPPED.is_match(tight)
            || (NOT_WORKING.is_match(tight) && SERIOUS_CARE.is_match(tight)))
    {
        // The treatment that stopped working is the treatment that ran out, but only when a
        // specialist said it: "my acne treatment isnt working anymore" is a chemist's line. And
        // the give-up that names something is only about somebody when what it names is one:
        // "nothing more they can do about my knee" is an orthopaedic appointment.
        if !NOTHING_MORE_TO_DO.is_match(tight) || _the_give_up_is_about_somebody(tight, Some(words), false) {
            return true;
        }
    }
    if _the_give_up_without_a_care_word(tight, who) {
        // "they said there is nothing more they can do for her" - the shape and the person, said
        // by somebody, with no care word to carry it.
        return true;
    }
    if MEDICATION_STOPPED.is_match(tight) && SERIOUS_CARE.is_match(tight) {
        // The pills only count next to the illness.
        return true;
    }
    if says_the_last_drug(tight) {
        // The last drug needs no other word: "they put my dad on morphine" is the news on its own.
        return true;
    }
    let somebody =
        _has_person(words) || _has_pet(words) || has(words, "she") || has(words, "he") || any_in(words, &HUMAN_WORDS);
    if FINAL_DAYS.is_match(tight) {
        // "shes in her final weeks" needs no other word to place it: the possessive already says
        // whose weeks they are, and "the final weeks of the season" has none.
        return true;
    }
    if !somebody && !who {
        // Nothing says whose days these are. The patterns below that name nobody need the
        // doctor's word to carry them; the ones that need somebody are read as a death only when
        // it is there.
        return _the_family_is_told(tight);
    }
    if DAYS_LEFT.is_match(tight) && !NOT_THE_END.is_match(tight) {
        // Nobody named, so only a doctor or a hospital saying it makes the days somebody's; a
        // pronoun is not enough, but "she has days left" is the person the chat has been about.
        return true;
    }
    if COULD_BE_DAYS.is_match(tight) && !NOT_THE_END.is_match(tight) {
        // The same count in other words, and this one is only read with somebody named. A pronoun
        // is not that somebody, and the days may be a wait for something (see DAYS_WAIT_FOR).
        return !DAYS_WAIT_FOR.is_match(tight) && (who || _has_person(words) || _has_pet(words));
    }
    if LAST_STAGES.is_match(tight) {
        return true;
    }
    if NOT_GETTING_BETTER.is_match(tight) {
        // "my nan isnt going to get better" - said of a person it is the death, said of a back it
        // is the back, and `_has_person`/`HUMAN_WORDS` is what tells them apart.
        return true;
    }
    if GIVEN_UP_ON.is_match(tight) && who {
        return true;
    }
    if PREPARE.is_match(tight) && who && !PREPARE_OTHERWISE.is_match(tight) {
        return true;
    }
    if OFF_LIFE_SUPPORT.is_match(tight) && !OFF_SUPPORT_WELL.is_match(tight) {
        return true;
    }
    if HOSPICE_CARE.is_match(tight) && !HOSPICE_JOB.is_match(tight) {
        return true;
    }
    if PALLIATIVE_ITSELF.is_match(tight) && !asks_anything(tight, None) && !_WORKING_WITH_THE_DYING.is_match(tight) {
        // "my mum is a palliative care nurse" is a job with the same word in it.
        return true;
    }
    false
}

/// Python: bot/compose/loss.py:1057 `_the_family_is_told(tight)`
/// True for the give-up that names nobody but carries the specialist's word.
pub fn _the_family_is_told(tight: &str) -> bool {
    let serious = SERIOUS_CARE.is_match(tight);
    if serious && (NOT_WORKING.is_match(tight) || PREPARE.is_match(tight)) {
        return !PREPARE_OTHERWISE.is_match(tight);
    }
    if PALLIATIVE_ITSELF.is_match(tight) && !asks_anything(tight, None) && !_WORKING_WITH_THE_DYING.is_match(tight) {
        // "theyve said its palliative now" is the sentence, "what is palliative care" is the
        // question about it, and the second one is somebody reading up, not somebody losing.
        return true;
    }
    if COULD_BE_DAYS.is_match(tight) && TOLD_US.is_match(tight) {
        // "weve been told it could be days" is a family in a relatives' room, and "they said it
        // could be days for the results" is the same words about a wait.
        return !DAYS_WAIT_FOR.is_match(tight);
    }
    if OFF_LIFE_SUPPORT.is_match(tight) && !OFF_SUPPORT_WELL.is_match(tight) {
        return true;
    }
    if HOSPICE_CARE.is_match(tight) && !HOSPICE_JOB.is_match(tight) {
        return true;
    }
    false
}

/// Python: bot/compose/loss.py:1085 `_the_vet_said_it(tight, words)`
/// True when a vet is the one who said it, so the animal's own pool answers.
pub fn _the_vet_said_it(tight: &str, words: &[String]) -> bool {
    let said = text::split_ref(tight);
    (said.contains(&"vet") || said.contains(&"vets")) && !any_in(words, &HUMAN_WORDS)
}

/// Python: bot/compose/loss.py:1105 `_the_give_up_is_about_somebody(tight, words, third_person_only=False)`
/// True when the doctor's giving up is about somebody, else False.
///
/// Nothing before the verb separates "nothing more they can do" from "nothing more they can do
/// about my knee", so what the giving up names decides it: no object at all is the whole of
/// somebody's care; an object that is a person or their illness is the same news; an object that
/// is a body part or a thing is not. Only the text after the verb is read. `third_person_only` is
/// for the reading that has no care word at all to carry it: there, "nothing more they can do
/// for me" is the speaker talking about themselves, and only somebody else counts.
pub fn _the_give_up_is_about_somebody(tight: &str, _words: Option<&[String]>, third_person_only: bool) -> bool {
    let Some(named) = THE_GIVE_UP_NAMES.search(tight) else { return !third_person_only };
    let subject = named.str(1);
    if SERIOUS_CARE.is_match(subject) {
        return true;
    }
    // The last word is the thing the giving up is really about: "my mums knee" hangs the body part
    // off a person, and the appointment is the knee's.
    let named_words = words_of(subject);
    let Some(last) = named_words.last() else { return !third_person_only };
    if third_person_only && ["me", "us", "i", "mine", "ours"].contains(&last.as_str()) {
        return false;
    }
    PEOPLE_AFTER_A_GIVE_UP.contains(last) || _has_person(std::slice::from_ref(last))
}

/// Python: bot/compose/loss.py:1154 `_the_give_up_without_a_care_word(tight, who)`
/// True for the give-up that names a person but no doctor, else False.
pub fn _the_give_up_without_a_care_word(tight: &str, who: bool) -> bool {
    if who || !NOTHING_MORE_TO_DO.is_match(tight) || !SAID_IT.is_match(tight) {
        return false;
    }
    if NOT_A_CARE_GIVER.is_match(tight) {
        return false;
    }
    _the_give_up_is_about_somebody(tight, None, true)
}

/// Python: bot/compose/loss.py:1493 `_the_bill_was_the_last_turn(memory)`
/// True when the last answer was about the arrangements and what they cost.
pub fn _the_bill_was_the_last_turn(memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    if last.module.as_ref().is_some_and(|module| _NOT_A_BILL_THREAD.contains(module)) {
        return false;
    }
    let said = text::lower(&last.bot);
    _BILL_SUBJECT.is_match(&said) && _BILL_MONEY.is_match(&said)
}

/// `int(value or 0)` for a number kept in memory.
fn py_int(value: Option<&Value>) -> i64 {
    match value {
        Some(Value::Int(n)) => *n,
        Some(Value::Bool(b)) => *b as i64,
        Some(Value::Float(x)) => x.trunc() as i64,
        Some(Value::Str(s)) if !s.is_empty() => {
            text::strip(s).parse().unwrap_or_else(|_| panic!("ValueError: invalid literal for int(): {s:?}"))
        }
        _ => 0,
    }
}

/// Python: bot/compose/loss.py:1511 `_loss_is_the_thread(memory)`
/// True when a death has been talked about in the last few turns of this conversation.
pub fn _loss_is_the_thread(memory: &Memory) -> bool {
    let Some(record) = memory.get("loss").filter(|record| record.is_dict()) else { return false };
    memory.turns.len() as i64 - py_int(record.get("turn")) <= *LOSS_STILL_THE_THREAD
}

/// Python: bot/compose/loss.py:1520 `arrangement_cost_pool(line, tight=None, memory=None)`
/// The cost of the arrangements, asked on its own: the pool to answer with, or None.
///
/// Read before `is_practical_ask`, which stands the composer aside for every other practical
/// question inside a bereavement: a price question with no death in it is not one of those. A
/// comparison of the two ("is cremation cheaper than burial") has its own pool, because the
/// answer is which of them costs less rather than where the money goes.
pub fn arrangement_cost_pool(line: &str, tight: Option<&str>, memory: &Memory) -> Option<Vec<String>> {
    let own;
    let tight = match tight {
        Some(tight) => tight,
        None => {
            own = _tight(line);
            &own
        }
    };
    if FUNERAL_COST_ELSE.is_match(tight) || _DEATH_IN_LINE.is_match(tight) {
        return None;
    }
    if _PRONOUN_MONEY_ASK.is_match(tight) && _the_bill_was_the_last_turn(memory) {
        // "and can i get help paying for it" after an answer about the bill: the "it" is the
        // arrangements, and the help with them is what is being asked for. This is read before
        // the death-thread stand-aside below, because the grief module leaves the practical
        // follow-up to the composer and the search shrug is what answered it.
        if FUNERAL_COST_COMPARE.is_match(tight) && !HELP_ASK.is_match(tight) {
            return Some(FUNERAL_COST_COMPARE_REPLIES.clone());
        }
        return Some(FUNERAL_HELP_REPLIES.clone());
    }
    if _loss_is_the_thread(memory) {
        // A death is already in this conversation and the grief module is still on it, so the
        // price question is answered there. This reading is for the price asked cold.
        return None;
    }
    if !(FUNERAL_COST.is_match(tight) || FUNERAL_HELP.is_match(tight)) {
        return None;
    }
    // "cremation or burial, which is cheaper" is a question without a question mark or a question
    // word at the front, so the comparison itself is read as the ask. A remark about what was
    // already paid is not a question and gets whatever the pools above have.
    if !(asks_anything(line, Some(tight))
        || QUIETER_ASK.is_match(tight)
        || FUNERAL_COST_COMPARE.is_match(tight)
        || HELP_ASK.is_match(tight))
    {
        return None;
    }
    if FUNERAL_HELP.is_match(tight) && !_THE_ESTATE.is_match(tight) {
        // "does the council pay for funerals", "i cant afford a funeral": the money is the
        // question, and where it goes is not the answer to it.
        return Some(FUNERAL_HELP_REPLIES.clone());
    }
    if FUNERAL_COST_COMPARE.is_match(tight) && _BURIAL_SIDE.is_match(tight) && _CREMATION_SIDE.is_match(tight) {
        return Some(FUNERAL_COST_COMPARE_REPLIES.clone());
    }
    // A plain "how much does a funeral cost" is left to the knowledge entry that carries the
    // figures; nothing here answers it, so the line falls through to whatever it had.
    None
}

/// Python: bot/compose/loss.py:1703 `says_the_last_drug(tight)`
/// True when the line says somebody has been started on the drug that ends the treatment. The
/// person's own painkiller is not that news, so the first person switches it off.
pub fn says_the_last_drug(tight: &str) -> bool {
    if !ON_THE_LAST_DRUG.is_match(tight) {
        return false;
    }
    !SELF_ON_THE_DRUG.is_match(tight)
}

/// Python: bot/compose/loss.py:1842 `asks_about_the_remains(line, tight=None)`
/// True when the pets entry is the answer to what the line asks: it keeps the turn only for a
/// line that asks about the body. (`tight` is a Python value: None, or the tight line.)
pub fn asks_about_the_remains(line: &str, tight: &Value) -> bool {
    let own;
    let tight = match tight.as_str() {
        Some(tight) => tight,
        None => {
            own = _tight(line);
            &own
        }
    };
    if !PET_ARRANGEMENT.is_match(tight) {
        return false;
    }
    // The question can be in the second half of the line, so the household shapes and the
    // comma-then-question shape count beside the composer's own ask reading. `ASK_TAIL` reads the
    // line as typed: `_tight` is for the word lists, and it drops the comma that makes "my cat
    // died, is cremation expensive" a question at all.
    asks_anything(line, Some(tight)) || ASK_WE.is_match(tight) || ASK_TAIL.is_match(line) || ASK_BARE.is_match(tight)
}

/// Python: bot/compose/loss.py:1869 `a_pet_died(line)`
/// True when the death in the line is a pet's, so the pets entry is the answer to it. Read
/// clause by clause; a death still to come is not a death (`NOT_YET`).
pub fn a_pet_died(line: &str) -> bool {
    let text_ = lowered(line);
    if NOT_YET.is_match(&text_) && !PUTTING_REGRETTED.is_match(&text_) {
        return false;
    }
    for clause in CLAUSE.split(&text_) {
        if !OWN_PET.is_match(&clause) {
            continue; // "my mates dog", "her cat": someone else's pet, so it is not their loss
        }
        let words = _plain(&clause);
        if !words.is_empty() && reads_as_loss(&clause, Some(&words), false) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/loss.py:1919 `asks_the_administration(line)`
/// True when the line asks about the paperwork or the money a death leaves behind.
pub fn asks_the_administration(line: &str) -> bool {
    ADMINISTRATION.is_match(&text::squash(&lowered(line)))
}

/// Python: bot/compose/loss.py:1956 `administration_keys(text)`
/// The pieces of paperwork a line or a reply names, as plain keys (a frozenset in Python).
pub fn administration_keys(text_: &str) -> Vec<String> {
    let low = text::squash(&lowered(text_));
    ADMINISTRATION_KEYS.iter().filter(|(_, pattern)| pattern.is_match(&low)).map(|(key, _)| key.clone()).collect()
}

/// Python: bot/compose/loss.py:1975 `is_about_the_paperwork(reply, line)`
/// Whether the reply answers the paperwork the line named, not merely the right shape: it names
/// the same piece of paperwork, it says whose paperwork it is, and it is not a definition.
pub fn is_about_the_paperwork(reply: &str, line: &str) -> bool {
    let of_the_reply = administration_keys(reply);
    if !administration_keys(line).iter().any(|key| of_the_reply.contains(key)) {
        return false;
    }
    if DEFINES.match_(reply).is_some() || !THE_DEATH.is_match(reply) {
        return false;
    }
    true
}

/// Python: bot/compose/loss.py:2070 `_a_thing_of_its_own_to_get_through(line)`
/// True when the "get through" ask is about something that has an answer of its own.
pub fn _a_thing_of_its_own_to_get_through(line: &str) -> bool {
    let Some(found) = GET_THROUGH_OBJECT.search(line) else { return false };
    for word in words_of(found.str(1)) {
        if GET_THROUGH_FILLER.contains(&word) || GET_THROUGH_THE_LOSS.contains(&word) {
            continue;
        }
        return true;
    }
    false
}

/// Python: bot/compose/loss.py:2087 `asks_for_a_decision(line)`
/// True when the line asks what to do now, not how the person feels.
pub fn asks_for_a_decision(line: &str) -> bool {
    PRACTICAL_ASK.is_match(&text::squash(&lowered(line)))
}

/// Python: bot/compose/loss.py:2115 `feeling_about_the_body(line)`
/// True when a feeling word in the line is about the body, not about the person.
pub fn feeling_about_the_body(line: &str) -> bool {
    BODY_FEELING.is_match(&text::squash(&text::lower(line)))
}

/// Python: bot/compose/loss.py:2120 `names_a_feeling(line)`
/// True when the line talks about the person's own feelings, so sympathy beats the card.
pub fn names_a_feeling(line: &str) -> bool {
    let text_ = text::squash(&text::lower(line));
    // The absence shape is how people say it in the days after: "i keep expecting him at the
    // door", "the house feels so quiet". ABSENCE already reads those for the composer.
    OWN_STATE.is_match(&text_) || ABSENCE.is_match(&text_)
}

/// Python: bot/compose/loss.py:2134 `acknowledges(reply)`
/// True when a reply says the sad thing in its own words, so an entry that does can speak.
pub fn acknowledges(reply: &str) -> bool {
    ACKNOWLEDGE.is_match(&text::squash(&text::lower(reply)))
}

/// Python: bot/compose/loss.py:2158 `_when_asked(tight)`
/// True for a line asking when the funeral is, rather than whether to go.
pub fn _when_asked(tight: &str) -> bool {
    WHEN_ASK.is_match(tight)
}

/// Python: bot/compose/loss.py:2163 `a_part_of_the_day_asked(tight)`
/// True when the line asks about a part of the service, not whether to be at it. A line asking
/// what to *say* stays here: the words for a funeral are this file's.
pub fn a_part_of_the_day_asked(tight: &str) -> bool {
    if !A_PART_OF_THE_DAY.is_match(tight) {
        return false;
    }
    !SAY_THE_WORDS.is_match(tight)
}

/// Python: bot/compose/loss.py:2192 `_asks_about_the_feeling(tight)`
/// Whether the question is about the feeling rather than about the death or the day.
pub fn _asks_about_the_feeling(tight: &str) -> bool {
    ASKS_ABOUT_THE_FEELING.is_match(tight)
}

/// Python: bot/compose/loss.py:2326 `_feeling(tight)`
/// The feeling the line names, if it names one of ours, else "".
pub fn _feeling(tight: &str) -> String {
    for word in text::split_ref(tight) {
        if let Some(feeling) = FEELING_WORDS.get(word) {
            return feeling.clone();
        }
    }
    String::new()
}

/// Python: bot/compose/loss.py:2451 `_how_long_since(tight, words)`
/// True when the line says only how long it has been, inside a bereavement chat. The span has to
/// be there, the line has to be short, and it must not name anything else with a length in it.
pub fn _how_long_since(tight: &str, _words: &[String]) -> bool {
    if tight.is_empty() || text::split_ref(tight).len() > 9 {
        return false;
    }
    if ASK_FORM.is_match(tight) || tight.contains('?') {
        return false;
    }
    if A_SPAN_ELSEWHERE.is_match(tight) {
        return false;
    }
    // "its been a long time" is the gap said without a number in it. It is the same shape and
    // the same pool, and no other reader knows what to do with it either.
    let Some(span) = A_SPAN.search(tight).or_else(|| A_SPAN_NO_NUMBER.search(tight)) else { return false };
    let joined = format!("{} {}", &tight[..span.start()], &tight[span.end()..]);
    let rest = text::strip(&joined);
    // The span is the gap however many words it takes to write, so it is counted as one: what is
    // left has to be short, or the line is about something longer than the gap.
    let left = text::split_ref(rest).len();
    if left + 2 > 6 {
        return false;
    }
    A_SPAN_LEAD.is_match(rest) || A_SPAN_TAIL.is_match(rest) || left <= 2
}

/// Python: bot/compose/loss.py:2551 `_missing_someone(text)`
/// True when the line says they miss a person or a pet, without saying that anyone died.
pub fn _missing_someone(text_: &str) -> bool {
    if !MISS_VERB.is_match(text_) {
        return false;
    }
    if ASK_AT_START.match_(text_).is_some() || text_.contains('?') {
        return false;
    }
    let mut noun = MISS_NOUN.search(text_);
    if noun.as_ref().is_some_and(|found| MISS_NOT_BETWEEN.is_match(found.as_str())) {
        noun = None;
    }
    let pronoun = MISS_PRONOUN.search(text_);
    if noun.is_none() && pronoun.is_none() {
        return false;
    }
    if MISS_NOT.is_match(text_) {
        return false;
    }
    if let Some(found) = &noun {
        let after = &text_[found.end()..];
        if MISS_TAIL.match_(after).is_none() {
            // "i miss my mum's cooking", "i miss my dog walker": what is missed is the thing after.
            let rest = text::strip(after);
            let last = text::split_ref(found.as_str()).last().map(|word| text::lower(word)).unwrap_or_default();
            let animal = MISS_ANIMALS.contains(&last);
            let named = MISSED_NAME.match_(rest);
            let still_the_animal = animal
                && named.as_ref().is_some_and(|named| {
                    !MISS_ANIMAL_NOUNS.contains(&text::lower(named.str(1))) && MISS_TAIL.match_(named.str(3)).is_some()
                });
            if !still_the_animal {
                // "i miss my old dog rex", "i miss my cat milly so much": the name after the animal
                // is still the animal that is missed.
                return false;
            }
        }
    }
    true
}

/// Python: bot/compose/loss.py:2602 `_a_question_about_the_feeling(line, words)`
/// True when the line asks about a feeling itself and names nothing else to be about. (`words`
/// is a set in Python; the slice is read as one.)
pub fn _a_question_about_the_feeling(line: &str, words: &[String]) -> bool {
    if !FEELING_ASK.is_match(line) {
        return false;
    }
    if !any_in(words, &FEELING_STATES) {
        return false;
    }
    words
        .iter()
        .all(|w| FEELING_STATES.contains(w) || FEELING_ONLY_WORDS.contains(w) || FEELING_PERSON_WORDS.contains(w))
}

/// Python: bot/compose/loss.py:2804 `_an_ask_in_the_line(line)`
/// True when the line puts a question, whatever its punctuation.
pub fn _an_ask_in_the_line(line: &str) -> bool {
    if ASK_AT_START.match_(line).is_some() {
        return true;
    }
    AN_ASK_INSIDE.is_match(line)
}

/// Python: bot/compose/loss.py:2839 `_still_here(line, words)`
/// The pool for the habit a line reports, or None. It is asked only once a death is read in the
/// line, because the habit alone is not a death.
pub fn _still_here(line: &str, words: &[String]) -> Option<Vec<String>> {
    if line.contains('?') || QUESTION.is_match(line) {
        return None;
    }
    // A question with a "?" is gone before this reading sees the line (the caller hands it the
    // words), so "should i still set a place for him" has to be caught by its shape.
    if _an_ask_in_the_line(line) {
        return None;
    }
    if MISS_NOT.is_match(line) || _a_thing_of_theirs(words) {
        return None;
    }
    if _only_the_proceedings(&words.join(" "), words) {
        return None;
    }
    HABIT_PATTERNS.iter().find(|(pattern, _)| pattern.is_match(line)).map(|(_, pool)| pool.clone())
}

/// Python: bot/compose/loss.py:2866 `_a_habit_in(line)`
/// True when the line reports one of the habits the pools above answer, whatever the pool.
pub fn _a_habit_in(line: &str) -> bool {
    HABIT_PATTERNS.iter().any(|(pattern, _)| pattern.is_match(line))
}

/// Python: bot/compose/loss.py:3077 `_telling_a_child(tight)`
/// True when the line asks how to explain a death to a child of the user's.
pub fn _telling_a_child(tight: &str) -> bool {
    let Some(found) = TELL_CHILD.search(tight) else {
        // "how do i tell my 5 year old" carries no child word from the list at all
        return YEAR_OLD.is_match(tight) && TELL_OR_EXPLAIN.is_match(tight);
    };
    !NOT_A_CHILD_TOLD.is_match(found.str(1))
}

/// `marks_a_day` / `asks_what_to_do` read a line or its words: (raw, words).
fn raw_and_words(text_: &str) -> (String, Vec<String>) {
    let raw = _plain(text_).join(" ");
    let words = text::split(&raw);
    (raw, words)
}

/// Python: bot/compose/loss.py:3097 `marks_a_day(text, known=False)`
/// True when the line is about a day that matters and someone who is not there for it.
///
/// Strong evidence, enough on its own: the age someone would have been, and an occasion with the
/// missing person in it. Weak evidence, only enough once the chat is already about a loss
/// (`known`): the tradition and the question about keeping it going.
pub fn marks_a_day(text_: &str, known: bool) -> bool {
    let (raw, words) = raw_and_words(text_);
    let tight = words.iter().filter(|word| !FILLERS.contains(*word)).cloned().collect::<Vec<_>>().join(" ");
    let subject = _has_person(&words) || any_in(&words, &PRONOUN);
    if WOULD_HAVE.is_match(&tight) {
        return subject;
    }
    // The tradition needs the day in it too, or "we always have pasta on fridays, what should i
    // cook tonight" inside a bereavement chat becomes a plan for the anniversary. A named day or
    // a named person, not the bare pronoun.
    let day_named = DAY_THING.is_match(&raw) || OCCASION.is_match(&tight) || _has_person(&words);
    if known
        && ((TRADITION.is_match(&tight) && day_named)
            || (STILL_KEEP.is_match(&raw) && DAY_THING.is_match(&raw))
            || (DAY_QUESTION.is_match(&tight) && day_named))
    {
        // the day is already the subject of the chat, so a question about keeping it going is
        // the same question: "should i still do a cake" after "mum would have turned 80".
        return true;
    }
    if !OCCASION.is_match(&tight) {
        return false;
    }
    WITHOUT.is_match(&tight)
}

/// Python: bot/compose/loss.py:3150 `asks_what_to_do(text)`
/// True when the line about the day also asks what to do about it.
pub fn asks_what_to_do(text_: &str) -> bool {
    let (raw, _words) = raw_and_words(text_);
    if text_.contains('?') {
        return true;
    }
    ASKS_ABOUT_THE_DAY.is_match(&raw)
}

/// Python: bot/compose/loss.py:3167 `_plain(text)`
/// The line as words: lower case, apostrophes and punctuation flattened.
pub fn _plain(text_: &str) -> Vec<String> {
    let low = lowered(text_);
    text::split_ref(&low).into_iter().map(|word| text::strip_chars(word, ".,!?;:\"'()").to_string()).collect()
}

/// Python: bot/compose/loss.py:3173 `_about_them(line)`
/// True when the line is about the person talking, not about the pub or the boss.
pub fn _about_them(line: &str) -> bool {
    let text_ = text::strip(line);
    if SELF_OPENERS.iter().any(|opener| text_.starts_with(opener.as_str())) {
        return true;
    }
    let words = text::split_ref(text_);
    words.len() > 1 && words[0] == "my" && SELF_NOUNS.contains(text::strip_chars(words[1], ",.!?"))
}

/// Python: bot/compose/loss.py:3182 `_about_family(words)`
/// True when the line names a family member as its subject: "my mum isnt coping".
pub fn _about_family(words: &[String]) -> bool {
    words.len() > 1
        && ["my", "his", "her", "their", "our"].contains(&words[0].as_str())
        && words.iter().skip(1).take(2).any(|word| FAMILY.contains(text::strip_chars(word, ",.!?")))
}

/// Python: bot/compose/loss.py:3188 `_own_state(words)`
/// The person's own state: "can't sleep", "so tired", "the mornings are hard" - but not "i lost
/// my keys this morning".
pub fn _own_state(words: &[String]) -> bool {
    words
        .iter()
        .enumerate()
        .any(|(index, word)| GRIEF_SELF.contains(word) && !(index > 0 && WHEN_WORDS.contains(&words[index - 1])))
}

/// Python: bot/compose/loss.py:3216 `_state_describes_them(words)`
/// True when a state word in the line describes the person, not a thing they named.
pub fn _state_describes_them(words: &[String]) -> bool {
    words.iter().enumerate().any(|(index, word)| {
        GRIEF_SELF.contains(word) && (index + 1 >= words.len() || STATE_TAIL.contains(&words[index + 1]))
    })
}

/// Python: bot/compose/loss.py:3253 `_the_body_is_theirs(tight, words)`
/// True when the line asks what the loss is doing to the person's own body or appetite.
pub fn _the_body_is_theirs(tight: &str, words: &[String]) -> bool {
    if QUANTITY.is_match(tight) {
        return false;
    }
    if NORMAL_TO_FEEL.match_(tight).is_some() {
        return true;
    }
    if !(THEMSELVES.is_match(tight) || ASK_NORMAL.match_(tight).is_some()) {
        return false;
    }
    words.iter().enumerate().any(|(index, word)| {
        BODY_STATE.contains(word) && (index + 1 >= words.len() || STATE_TAIL.contains(&words[index + 1]))
    })
}

/// Python: bot/compose/loss.py:3281 `_a_body_with_its_own_answer(tight, words)`
/// True when the line reports the person's own body in a way that has a practical answer.
pub fn _a_body_with_its_own_answer(tight: &str, words: &[String]) -> bool {
    if QUANTITY.is_match(tight) {
        return false;
    }
    if !THEMSELVES.is_match(tight) {
        return false;
    }
    any_in(words, &BODY_ANSWER)
}

/// Python: bot/compose/loss.py:3295 `_names_a_subject(topics)`
/// True when the reader gave the line a subject of its own to be answered: `_own_subject` with
/// the measure topics taken out.
pub fn _names_a_subject(topics: &[String]) -> bool {
    topics.iter().any(|topic| !THREAD_TOPICS.contains(topic) && !MEASURE_TOPICS.contains(topic))
}

/// Python: bot/compose/loss.py:3306 `_own_subject(topics)`
/// True when the reader gave the line a subject that has an answer of its own.
pub fn _own_subject(topics: &[String]) -> bool {
    topics.iter().any(|topic| !THREAD_TOPICS.contains(topic))
}

/// Python: bot/compose/loss.py:3338 `_hands_over_a_detail(tight)`
/// Whether the line adds something new about the person rather than the missing said again.
pub fn _hands_over_a_detail(tight: &str) -> bool {
    ABOUT_THEM_OPENS.match_(tight).is_some() && !MISSING_SAID_AGAIN.is_match(tight)
}

/// Python: bot/compose/loss.py:3351 `_thread_pool(memory)`
/// The pool behind the bot's last line, when that line was the what-to-say advice (or the
/// company for a missing): the pool object itself, so a caller can ask which one it `is`.
pub fn _thread_pool(memory: &Memory) -> Option<&'static Vec<String>> {
    let last = &memory.turns.last()?.bot;
    if last.is_empty() {
        return None;
    }
    if SAY_REPLIES.contains(last) {
        return Some(&SAY_REPLIES);
    }
    if MISS_REPLIES.contains(last) {
        // "i miss my mum" was met with company, so the next line ("she died two years ago",
        // "she taught me to bake") is the same person, not a fresh chat.
        return Some(&MISS_REPLIES);
    }
    None
}

/// `_thread_pool(memory) is pool`.
fn thread_pool_is(memory: &Memory, pool: &'static Vec<String>) -> bool {
    _thread_pool(memory).is_some_and(|found| std::ptr::eq(found, pool))
}

/// Python: bot/compose/loss.py:3395 `_spoken(text)`
/// The line with the contractions spelled out, so "my mum's dying" matches "my mum is dying".
pub fn _spoken(text_: &str) -> String {
    _CONTRACTED.sub_fn(text_, |found| {
        let base = text::lower(found.str(1));
        if _CAN_BE_DYING.contains(&base) && !_NOT_A_CONTRACTION.contains(&format!("{base}s")) {
            format!("{} is ", found.str(1))
        } else {
            found.as_str().to_string()
        }
    })
}

/// Python: bot/compose/loss.py:3411 `_tight(text)`
/// The line without filler words, for phrase matching: "what do I even say" is "what do I say".
pub fn _tight(text_: &str) -> String {
    _plain(text_).into_iter().filter(|word| !FILLERS.contains(word)).collect::<Vec<_>>().join(" ")
}

/// Python: bot/compose/loss.py:3417 `_their_thing_after(clean, start)`
/// True when the word after the person is one of their things rather than a place or a time.
/// (`start` is a byte offset into `clean`, as a match gives it.)
pub fn _their_thing_after(clean: &str, start: usize) -> bool {
    text::split_ref(&clean[start..]).first().is_some_and(|word| OBJECTS.contains(*word) || THEIRS.contains(*word))
}

/// Python: bot/compose/loss.py:3427 `_another_person(words, clean)`
/// True when the line names a person who is not the role word a thing has borrowed.
pub fn _another_person(words: &[String], clean: &str) -> bool {
    if words.iter().any(|word| HUMAN_WORDS.contains(word) && !TWO_WAY_ROLES.contains(word)) {
        return true;
    }
    ROLE_PHRASE.is_match(clean)
}

/// Python: bot/compose/loss.py:3439 `_a_thing_of_theirs(words)`
/// True when a person word is only the qualifier of one of their things: "my carer badge", "my
/// nurse notes". Read as a person, a badge that has stopped got a condolence.
pub fn _a_thing_of_theirs(words: &[String]) -> bool {
    for (index, word) in words.iter().enumerate() {
        if index + 1 >= words.len() || !(OBJECTS.contains(&words[index + 1]) || THEIRS.contains(&words[index + 1])) {
            continue;
        }
        let stem = if let Some(stem) = word.strip_suffix("'s") {
            stem
        } else if let Some(stem) = word.strip_suffix('s') {
            stem
        } else {
            word.as_str()
        };
        if !PEOPLE.contains(word) && !PEOPLE.contains(stem) {
            continue;
        }
        let others = words.iter().enumerate().any(|(place, other)| {
            place != index
                && (PEOPLE.contains(other)
                    || (text::len(other) > 2 && other.strip_suffix('s').is_some_and(|stem| PEOPLE.contains(stem))))
        });
        if !others {
            return true;
        }
    }
    false
}

/// Python: bot/compose/loss.py:3461 `_a_role_phrase(clean)`
/// True when the line names a person a death can be said about with two words.
pub fn _a_role_phrase(clean: &str) -> bool {
    ROLE_PHRASE.is_match(clean)
}

/// Python: bot/compose/loss.py:3471 `_has_person(words)`
/// True when a person or a pet is named, in any form people type it: a possessive and a plural
/// are how the line says whose day it is.
pub fn _has_person(words: &[String]) -> bool {
    for word in words {
        if PEOPLE.contains(word) {
            return true;
        }
        if word.strip_suffix("'s").is_some_and(|stem| PEOPLE.contains(stem)) {
            return true;
        }
        if word.strip_suffix('s').is_some_and(|stem| PEOPLE.contains(stem)) {
            return true;
        }
    }
    for (first, second) in PEOPLE_PHRASES.iter() {
        if words.windows(2).any(|pair| &pair[0] == first && &pair[1] == second) {
            return true;
        }
    }
    // A role said in two ordinary words ("my home help", "my night carer") names a person the
    // token list cannot hold, because each word on its own is something else.
    _a_role_phrase(&words.join(" "))
}

/// Python: bot/compose/loss.py:3494 `_object_died(words)`
/// True when the thing before "died"/"dead" is an object, not a person.
pub fn _object_died(words: &[String]) -> bool {
    for (index, word) in words.iter().enumerate() {
        if index > 0 && ["died", "dies", "dead", "dying"].contains(&word.as_str()) {
            if OBJECTS.contains(&words[index - 1]) {
                return true;
            }
            if index > 1
                && OBJECTS.contains(&words[index - 2])
                && ["is", "was", "has", "had"].contains(&words[index - 1].as_str())
            {
                return true;
            }
        }
    }
    false
}

/// Python: bot/compose/loss.py:3589 `_where_i_put_it(said)`
/// True when the line asks where a thing was put, people and errands aside.
pub fn _where_i_put_it(said: &str) -> bool {
    for found in WHERE_I_PUT_IT.finditer(said) {
        let rest = text::split_ref(&said[found.end()..]);
        if !rest.is_empty() && NOT_MISLAID.contains(rest[0]) {
            if rest.len() == 1 || NOT_A_THING_NEXT.contains(rest[1]) {
                continue;
            }
        }
        return true;
    }
    false
}

/// Python: bot/compose/loss.py:3623 `locked_out(text)`
/// True when they are shut out of their home or car: "im locked out, i lost my keys". A digital
/// account ("locked out of my email") and the cat being shut out are not this.
pub fn locked_out(text_: &str) -> bool {
    let said = words_of(text_).join(" ");
    if !LOCKED_OUT.is_match(&said) {
        return false;
    }
    !NOT_A_DOOR.is_match(&said)
}

/// Python: bot/compose/loss.py:3636 `locked_out_of_the_car(text)`
/// True when the thing they are shut out of is the car, which has its own way in.
pub fn locked_out_of_the_car(text_: &str) -> bool {
    CAR_LOCK_OUT.is_match(&words_of(text_).join(" "))
}

/// Python: bot/compose/loss.py:3641 `_readable(text)`
/// The line with the engine's repairs in it: "wheres my keys" reads as "where is my keys". The
/// repairs are taken from the understanding engine rather than listed again here.
pub fn _readable(text_: &str) -> String {
    let Some(read) = caught(|| crate::understanding::normalize::normalise(text_)) else {
        return text_.to_string();
    };
    let joined = read.norm.join(" ");
    if joined.is_empty() { text_.to_string() } else { joined }
}

/// Python: bot/compose/loss.py:3656 `lost_thing_to_find(text)`
/// True when the line is something mislaid and findable: "i lost my keys", "ive lost my bag".
pub fn lost_thing_to_find(text_: &str) -> bool {
    let said = words_of(&_readable(text_)).join(" ");
    LOST_THING_TO_FIND.is_match(&said)
        || CANT_FIND_THING.is_match(&said)
        || WHERE_IS_THING.is_match(&said)
        || _where_i_put_it(&said)
        || WHERE_IS_THING_BARE.is_match(&said)
        || LEFT_THING.is_match(&said)
        || GONE_THING.is_match(&said)
        || KEEP_LOSING_THING.is_match(&said)
        || REMEMBER_NOT_WHERE.is_match(&said)
}

/// Python: bot/compose/loss.py:3677 `keeps_losing_thing(text)`
/// True when the same thing goes missing again and again: "i keep losing my keys".
pub fn keeps_losing_thing(text_: &str) -> bool {
    let said = words_of(text_).join(" ");
    KEEP_LOSING_THING.is_match(&said) || KEPT_GOING_MISSING.is_match(&said)
}

/// Python: bot/compose/loss.py:3686 `found_what_was_lost(text)`
/// True when the line says a thing that was being looked for has turned up.
pub fn found_what_was_lost(text_: &str) -> bool {
    FOUND_WHAT_WAS_LOST.is_match(&words_of(text_).join(" "))
}

/// Python: bot/compose/loss.py:3695 `lost_in_a_place(text)`
/// True when a person was lost in an everyday place: "i lost my son at the park". It reads the
/// person the line lost, then looks for a place after it.
pub fn lost_in_a_place(text_: &str) -> bool {
    let clean = words_of(text_).join(" ");
    for (pattern, group) in LOST_FORMS.iter() {
        let Some(found) = pattern.search(&clean) else { continue };
        if !DEATHABLE.contains(found.str(*group)) {
            continue;
        }
        return _lost_in_a_place_after(&clean, found.end());
    }
    false
}

/// Python: bot/compose/loss.py:3718 `_shed_after(clean, start)`
/// True when what follows the relation is the weight rather than the person (`SHED.match(clean,
/// start)`: a match that begins at `start`).
pub fn _shed_after(clean: &str, start: usize) -> bool {
    SHED.search_at(clean, start).is_some_and(|found| found.start() == start)
}

/// Python: bot/compose/loss.py:3735 `_feared_not_done(clean, start)`
/// True when the verb a match begins at sits behind a frame that keeps it in the future. The
/// frame ends on the verb itself, so a match that starts before the frame's end is the verb the
/// frame is about.
pub fn _feared_not_done(clean: &str, start: usize) -> bool {
    let start_chars = text::char_index(clean, start) as i64;
    for frame in FEARED_NOT_YET.finditer(clean) {
        let over = text::char_index(clean, frame.end()) as i64 - start_chars;
        if frame.start() <= start && (4..=6).contains(&over) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/loss.py:3748 `_lost_in_a_place_after(clean, start)`
/// True when an everyday place follows the person at *start*.
pub fn _lost_in_a_place_after(clean: &str, start: usize) -> bool {
    LOST_IN_A_PLACE.search_at(clean, start).is_some_and(|place| PLACES.contains(place.str(1)))
}

/// What can die in a line: `DEATHABLE`, less the animals that are also machines when the line is
/// about a machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanDie {
    a_machine_line: bool,
}

impl CanDie {
    pub fn contains(&self, word: &str) -> bool {
        DEATHABLE.contains(word) && !(self.a_machine_line && MACHINE_ANIMALS.contains(word))
    }
}

/// Python: bot/compose/loss.py:3758 `_can_die(clean)`
/// What can die in this line. A mouse in a line about a computer cannot: "i lost my mouse" is a
/// bad afternoon with a mouse mat, not a bereavement. "my pet mouse died" keeps the animal reading.
pub fn _can_die(clean: &str) -> CanDie {
    CanDie { a_machine_line: crate::understanding::topics::is_a_machine_word(clean) }
}

/// Python: bot/compose/loss.py:3768 `lost_a_thing(text)`
/// True when the line says something was lost and it is not a person or a pet.
pub fn lost_a_thing(text_: &str) -> bool {
    let words = words_of(text_);
    let clean = words.join(" ");
    let deathable = _can_die(&clean);
    if LOST_IT.is_match(&clean) {
        return true; // "i lost it with the kids": a bad moment, not a death
    }
    for (pattern, group) in LOST_FORMS.iter() {
        let Some(found) = pattern.search(&clean) else { continue };
        let thing = found.str(*group);
        if deathable.contains(thing) || LOST_SELF.contains(thing) {
            return false;
        }
        return true;
    }
    LOST_THE_THING.is_match(&clean)
}

/// Python: bot/compose/loss.py:3794 `words_of(text)`
/// The line's words, lower case and without punctuation: the reading both functions use.
pub fn words_of(text_: &str) -> Vec<String> {
    _plain(text_)
}

/// Python: bot/compose/loss.py:3806 `_brought_in_prey(clean, words)`
/// True when a living pet has brought something dead in, which is not a loss.
pub fn _brought_in_prey(clean: &str, words: &[String]) -> bool {
    if !(_has_pet(words) && PREY.is_match(clean) && HUNTING.is_match(clean)) {
        return false;
    }
    !PET_DIED.is_match(&PREY.sub(" ", clean))
}

/// Python: bot/compose/loss.py:3820 `_a_pronoun_is_what_died(words, clean)`
/// True when "it" is what died and the line names a thing that could be the "it".
pub fn _a_pronoun_is_what_died(words: &[String], clean: &str) -> bool {
    if !IT_DIED.is_match(clean) {
        return false;
    }
    any_in(words, &THINGS)
}

/// Python: bot/compose/loss.py:3832 `a_thing_stopped(text, words=None)`
/// True when the death word in the line is about a thing, not a person or a pet. Only the death
/// words are read here, so a line that says a person died stays a death however much of the rest
/// of it is about a thing. (The caller hands over the words; `a_thing_stopped_in` reads them
/// from the text, as Python does for `words=None`.)
pub fn a_thing_stopped(_text: &str, words: &[String]) -> bool {
    if words.is_empty() {
        return false;
    }
    if _object_died(words) {
        // "my laptop is dead", "the battery dies by lunchtime": the word before the death word
        // is the thing that stopped.
        return true;
    }
    words.iter().enumerate().any(|(place, word)| word == "dead" && _dead_is_not_a_death(words, place))
}

/// `a_thing_stopped(text)` with no words given.
pub fn a_thing_stopped_in(text_: &str) -> bool {
    a_thing_stopped(text_, &words_of(text_))
}

/// Python: bot/compose/loss.py:3855 `reads_as_loss(text, words=None, a_pet=False)`
/// True when the line is about a death or a loss of a person or a pet.
///
/// `a_pet` says the line names an animal the store already holds ("rex died"): a name is not in
/// any of the word lists, so without it a death said by name alone is read as small talk.
pub fn reads_as_loss(text_: &str, words: Option<&[String]>, a_pet: bool) -> bool {
    let own;
    let words = match words {
        Some(words) => words,
        None => {
            own = words_of(text_);
            &own
        }
    };
    if words.is_empty() {
        return false;
    }
    let clean = words.join(" ");
    let clean = clean.as_str();
    let deathable = _can_die(clean);
    if any_phrase_in(clean, &LAUGHING) {
        return false;
    }
    if lost_in_a_place(clean) {
        return false;
    }
    if WOULD_HAVE_BEEN.is_match(clean) && (_has_person(words) || _unnamed_person(words, clean)) {
        // "my mum would have been 70 this week": the day is the loss.
        return true;
    }
    if _brought_in_prey(clean, words) {
        // "my cat keeps bringing dead birds in": the cat is fine and the bird is its dinner.
        return false;
    }
    if _a_thing_of_theirs(words) {
        // "my carer badge died", "my nurse notes went missing": the badge and the notes are the
        // subject and the person is whose they are, so the line is about the thing. Read before
        // the death words, because a thing of theirs said with "died" is a thing that has stopped.
        return false;
    }
    if _a_pronoun_is_what_died(words, clean) {
        // "i bought a phone for my daughter and it died": a person is in the line, but the subject
        // of "died" is "it", and the thing a phone is beats the person whose phone it is. Read
        // before the person test, because a parent in the sentence is what made this a condolence.
        return false;
    }
    let mut person = _has_person(words) || _unnamed_person(words, clean);
    if _the_end_is_near(clean, Some(words)) {
        // "the doctor said theres nothing more they can do", "theyve stopped mums chemo", "my dad
        // has weeks left": the treatment has run out, which is how most people say somebody is
        // dying now, and the grief starts here rather than at the funeral. Asked before the
        // proceedings, because those lines say "can" and have no person in them, which is the
        // shape of "how do i arrange a funeral".
        return true;
    }
    if _only_the_proceedings(clean, words) {
        // "funeral costs": a question about the proceedings is the knowledge base's, not a
        // condolence.
        return false;
    }
    if person && THING_ROLE.is_match(clean) && !_another_person(words, clean) {
        // "my vacuum cleaner died": the appliance has stopped, and sympathy is the wrong answer.
        person = false;
    }
    if PLAIN.is_match(clean) {
        return true;
    }
    if _passed_alone(clean, person) || _passed_on(clean, person) || _death_noun(clean, words, person) {
        return true;
    }
    if WAKE.is_match(clean) {
        return true;
    }
    if _buried_person(clean, words, person) {
        return true;
    }
    if SYMPATHY.is_match(clean) && CARD_WORD.is_match(clean) {
        return true;
    }
    if TOOK.is_match(clean) {
        return true;
    }
    // A killing is a death when a person is in the line, and only then.
    if KILLED.is_match(clean) && !_killed_in_a_story(clean) && (person || any_in(words, &PASSED_ON_SUBJECT)) {
        return true;
    }
    if _put_down(clean, words) {
        return true;
    }
    if person && STILL_CRY.is_match(clean) && !OTHER_WEEPS.is_match(clean) && !THEIR_TEARS_IN_A_CLAUSE.is_match(clean)
    {
        // "is it normal to still cry about my mum three years later" names no death but is grief.
        // A line where it is someone else crying ("my son keeps crying") is about them.
        return true;
    }
    if DEATH_AS_A_THING.is_match(clean) && !NOT_A_DEATH.is_match(clean) {
        // "a death is never good news, is it" is about death itself, not about the rate of it.
        // The same three readings that keep a false death out of the noun form keep it out here:
        // the speaker's own death, a thing that died and a death on a screen.
        if OWN_WORRY.is_match(clean) || THING_DEATH.is_match(clean) || PET_DEATH.is_match(clean) {
            return false;
        }
        if FICTION.is_match(clean) && !person {
            return false;
        }
        if DEATH_COMPOUND.is_match(clean) {
            // "his death doula was lovely": the word after "death" is the thing being named.
            return false;
        }
        if let Some(owner) = DEATH_OF_OWNER.search(clean) {
            let last = text::split_ref(owner.str(1)).last().copied().unwrap_or("");
            if THINGS.contains(last) && !person {
                // "the death of our old boiler": the last word of the owner phrase is the thing.
                return false;
            }
        }
        return true;
    }
    if let Some(lost_the) = LOST_THE.search(clean) {
        let lost = lost_the.str(1);
        if deathable.contains(lost) && !PET_WORDS.contains(lost) && !LOST_THE_FOUND.is_match(clean) {
            // "we lost the baby at twelve weeks" is a bereavement; "we lost the baby in the
            // supermarket" is a minute of panic, and "we lost the dog" could be either.
            return true;
        }
    }
    for (pattern, group) in LOST_FORMS.iter() {
        let Some(found) = pattern.search(clean) else { continue };
        if deathable.contains(found.str(*group)) {
            // "lost her job" is a job; "lost my dad" is a dad; "we lost nan" is nan.
            if _their_thing_after(clean, found.end()) {
                continue; // "i lost my carer badge": the badge went missing, not the carer
            }
            // "losing my son at the park" is ten bad minutes and "how do i lose the baby
            // weight" is the pounds, in the present tense as well as the past.
            if _lost_in_a_place_after(clean, found.end())
                || _shed_after(clean, found.end())
                || _feared_not_done(clean, found.start())
            {
                continue;
            }
            return true;
        }
    }
    for (pattern, is_loss_of) in [(&*LOSS_OF, true), (&*NEWS_OF, false)] {
        let Some(found) = pattern.search(clean) else { continue };
        if deathable.contains(found.str(1)) {
            // "the loss of my mum" is the loss; "the loss of my appetite" names nothing that can
            // die, and "the news of my dad hit me hard" is the death in the hit, not in the noun.
            if is_loss_of && _lost_in_a_place_after(clean, found.end()) {
                continue;
            }
            return true;
        }
    }
    if let Some(losing) = LOSING.search(clean) {
        let lost = losing.str(1);
        if deathable.contains(lost)
            && (CLOSE.contains(lost) || LOSS_GRIEF_CUE.is_match(clean))
            && !_shed_after(clean, losing.end())
            && !_feared_not_done(clean, losing.start())
            && !_lost_in_a_place_after(clean, losing.end())
        {
            // "losing my wife has hit me hard" is a bereavement; "losing my son at the park" is ten
            // bad minutes and "im losing my friend" has no grief cue at all.
            return true;
        }
    }
    if marks_a_day(clean, false) {
        // "my mum would have been 70 this week", "the first christmas without my dad": the death
        // is in the tense and in the missing person, not in a death word.
        return true;
    }
    if CONDITIONAL_TAIL.is_match(clean) && !any_in(words, &HUMAN_WORDS) {
        // A conditional is a chore, not a death: "our fish are going to die if we dont clean the
        // tank" is about the tank, and "the plants will die unless i water them" is about water.
        return false;
    }
    // "she's dying" names no noun: the person was named a turn earlier, and the pronoun stands
    // for them. Only she and he and they: "it is dying" is a plant.
    let spoken = _spoken(clean);
    let a_pronoun = any_in(&words_of(&spoken), &_PEOPLE_PRONOUNS);
    if ANTICIPATED.is_match(&spoken)
        && (person || a_pronoun)
        && !_object_died(words)
        && !(_WORKING_WITH_THE_DYING.is_match(clean) && !_SAID_DYING.is_match(clean))
    {
        // "my wife is on palliative care", "my nan is in a hospice", "she has weeks to live":
        // the death is ahead and the grief is already here. Read as a loss so the composer
        // answers it, and answered by the DYING_REPLIES pool because nobody has died yet.
        return true;
    }
    if !person && a_pronoun && DYING_NOW.is_match(&spoken) && !_object_died(words) && !DYING_TO.is_match(clean) {
        // "she's dying", "he is terminal": the line names nobody, and the turn before named who.
        // Only where no noun carries the person, so a line that says who is read as it was.
        return true;
    }
    if GOING_TO_DIE.is_match(clean) && (person || a_pronoun) && !_object_died(words) {
        // "my nan is going to die" and "my mum is going to die soon" are the loss now, not later:
        // the future tense is how people say it while it is happening. A feeling that follows
        // ("going to die of embarrassment") is the idiom, and LAUGHING has already returned.
        return true;
    }
    if a_pet && (DYING.is_match(clean) || DIED.is_match(clean)) && !_object_died(words) && !NOT_A_DEATH.is_match(clean)
    {
        return true;
    }
    if _accident_death(clean) {
        // "my cat was run over last night": no death word in the line, and the cat card that
        // knows about cats answers a cat that is still in the room.
        return true;
    }
    if (DYING.is_match(clean) || DIED.is_match(clean)) && person && !_object_died(words) {
        for (index, word) in words.iter().enumerate() {
            if word == "dead" && _dead_is_not_a_death(words, index) {
                // "dead birds": this "dead" describes the word after it, so it is not a death.
                // "my cat died yesterday and he used to bring dead birds in" still is one.
                continue;
            }
            if ["died", "dies", "dead", "dying"].contains(&word.as_str()) {
                return true;
            }
        }
    }
    false
}

/// Python: bot/compose/loss.py:4027 `_accident_death(clean)`
/// True when the line says someone of ours died in an accident, without the word "died". The
/// passive is what makes it a death rather than a mishap of the speaker's doing, the loss has to
/// be of ours, and a line that says the person is fine is a fright, not a bereavement.
pub fn _accident_death(clean: &str) -> bool {
    if !MISADVENTURE.is_match(clean) || SURVIVED.is_match(clean) {
        return false;
    }
    if NOT_A_DEATH.is_match(clean) {
        return false; // "death row", "the death of the high street": the word is not a loss
    }
    if VERB_OBJECT.is_match(clean) {
        return false; // "my daughter drowned her phone", not the daughter
    }
    let Some(victim) = _owned_deathable(clean) else { return false };
    if victim.is_empty() {
        return false;
    }
    if !PET_WORDS.contains(&victim) && KNOCKED_BY.is_match(clean) && !DEATH_SAID.is_match(clean) {
        return false; // "my dad got knocked over by a bike" is a broken arm until it says otherwise
    }
    true
}

/// Python: bot/compose/loss.py:4050 `_owned_deathable(clean)`
/// The person or pet an accident happened to, or None. Every word after the owner is tried, so
/// the adjective in "our old dog" does not stand in for the one that can die.
pub fn _owned_deathable(clean: &str) -> Option<String> {
    for owner in OWNED.finditer(clean) {
        for word in text::split_ref(owner.str(1)) {
            if DEATHABLE.contains(word) {
                return Some(word.to_string());
            }
        }
    }
    None
}

/// Python: bot/compose/loss.py:4063 `_passed_on(clean, person)`
/// True when "passed on" is a death, not a declined offer. A person or pet has to be the
/// subject, and the word after "on" must not be an object.
pub fn _passed_on(clean: &str, person: bool) -> bool {
    if !clean.contains(PASSED_ON.as_str()) {
        return false;
    }
    let (before, _, after) = text::partition(clean, &PASSED_ON);
    let before_words = text::split_ref(before);
    let near = &before_words[before_words.len().saturating_sub(3)..];
    if !person && !near.iter().any(|word| PASSED_ON_SUBJECT.contains(*word)) {
        return false;
    }
    let rest = text::split_ref(after);
    if rest.is_empty() {
        return true;
    }
    if !PASSED_ON_OBJECT.contains(rest[0]) {
        return true;
    }
    if rest[0] == "the"
        && rest.len() > 1
        && (MONTHS.contains(rest[1]) || opens_with_a_digit(rest[1]) || ["1st", "2nd", "3rd"].contains(&rest[1]))
    {
        return true;
    }
    false
}

/// Python: bot/compose/loss.py:4086 `_passed_alone(clean, person)`
/// True when "passed" on its own is a death, not a test, a deadline or a plate. A person or pet
/// has to be the subject, and what follows has to be a date, a time or nothing.
pub fn _passed_alone(clean: &str, person: bool) -> bool {
    let words = text::split_ref(clean);
    let passed = PASSED_ALONE.as_str();
    if !words.contains(&passed) {
        return false;
    }
    for (i, word) in words.iter().enumerate() {
        if *word != passed {
            continue;
        }
        if !person && !words[i.saturating_sub(4)..i].iter().any(|w| PASSED_ON_SUBJECT.contains(*w)) {
            continue;
        }
        if words[..i].iter().any(|w| PASSED_A_RESULT.contains(*w)) {
            // "my mum got her results and she passed": something to pass was named, so the word
            // is the achievement and not the death.
            continue;
        }
        let rest = &words[i + 1..];
        if rest.is_empty() {
            return true;
        }
        let head = rest[0];
        let nxt = if rest.len() > 1 { rest[1] } else { "" };
        if PASSED_ALONE_TAIL.contains(head) {
            continue;
        }
        if ["in", "on", "at", "last", "this", "that", "back"].contains(&head) {
            if head == "at" {
                if PASSED_TIME.contains(nxt) {
                    return true;
                }
                continue;
            }
            if PASSED_DATE.contains(nxt) || PASSED_TIME.contains(nxt) || opens_with_a_digit(nxt) {
                return true;
            }
            continue;
        }
        if head == "it" && ["was", "is", "has", "had"].contains(&nxt) {
            // "my nan passed, it was expected" - the next clause, not an object.
            return true;
        }
        if PASSED_ON_OBJECT.contains(head) || PASSED_ALONE_HANDED.contains(head) || PASSED_ALONE_NOUN.contains(head) {
            continue;
        }
        return true;
    }
    false
}

/// Python: bot/compose/loss.py:4129 `_death_noun(clean, words, person)`
/// True when the word "death" in this line is someone's death, not a word doing another job. An
/// owner makes it a particular death, so it counts on its own; a bare article needs a
/// bereavement cue as well. Fiction, the speaker's own death and object deaths are read out.
pub fn _death_noun(clean: &str, _words: &[String], person: bool) -> bool {
    if !DEATH_NOUN.is_match(clean) || NOT_A_DEATH.is_match(clean) {
        return false;
    }
    if THING_DEATH.is_match(clean) || PET_DEATH.is_match(clean) {
        return false;
    }
    if DEATH_COMPOUND.is_match(clean) {
        return false;
    }
    if FICTION.is_match(clean) && !person {
        return false;
    }
    if OWN_WORRY.is_match(clean) {
        return false;
    }
    if person {
        return true;
    }
    if let Some(owned) = OWN_DEATH.search(clean) {
        // A match of OWN_DEATH begins on a word, so the split has a first word.
        let owner = text::split_ref(owned.as_str()).first().copied().expect("IndexError: list index out of range");
        if ["my", "his", "her", "their", "your", "our"].contains(&owner) {
            return true;
        }
        // "dads death" is a death; "hamlets death" is a play, so the owner has to be a person.
        let stem = owner.trim_end_matches('s').trim_end_matches('\'');
        if PEOPLE.contains(stem) || PEOPLE.contains(owner) {
            return true;
        }
    }
    DEATH_CUE.is_match(clean)
}

/// Python: bot/compose/loss.py:4162 `_unnamed_person(words, clean)`
/// True only when "someone"/"people" is a real death: the line asks what to say about it. A show
/// or a film dying is never anyone's loss.
pub fn _unnamed_person(words: &[String], clean: &str) -> bool {
    if !any_in(words, &UNNAMED) {
        return false;
    }
    if any_in(words, &MEDIA) {
        return false;
    }
    ASK_WORDS.is_match(clean)
}

/// Python: bot/compose/loss.py:4194 `_buried_person(clean, words, person=False)`
/// True for "we buried my dad", false for "i buried the fence posts" and "the dog buried a
/// bone": after the verb, the thing being buried has to be a person or a pet. A passive "be
/// buried" is read from the subject in front of it.
pub fn _buried_person(clean: &str, words: &[String], person: bool) -> bool {
    if !BURIED.is_match(clean) {
        return false;
    }
    for (index, word) in words.iter().enumerate() {
        if word != "buried" && word != "bury" {
            continue;
        }
        if index > 0 && PASSIVE_AUX.contains(&words[index - 1]) {
            // "my dad was buried last year" is a funeral; "the fence post should be buried two
            // foot deep" is not. A person, a pet or a grave object settles it one way, a thing
            // that gets buried the other, and anything else keeps the reading the verb had before.
            if person || _has_pet(words) || words.iter().any(|w| REMAINS.contains(w)) {
                return true;
            }
            return !words.iter().any(|w| BURIABLE.contains(w));
        }
        let mut rest = &words[index + 1..(index + 5).min(words.len())];
        while !rest.is_empty() && BURIED_ADJECTIVE.contains(&rest[0]) {
            rest = &rest[1..];
        }
        if !rest.is_empty() && ["him", "her", "them"].contains(&rest[0].as_str()) {
            return true;
        }
        if !rest.is_empty() && BURIED_OBJECT.contains(&rest[0]) {
            rest = &rest[1..];
            while !rest.is_empty() && BURIED_ADJECTIVE.contains(&rest[0]) {
                rest = &rest[1..];
            }
            if rest.is_empty() {
                return false;
            }
            return PEOPLE.contains(&rest[0]);
        }
        return true;
    }
    false
}

/// Python: bot/compose/loss.py:4229 `_put_down(clean, words)`
/// True when "put down" / "put to sleep" is about a pet that died, not a nap or a phone.
pub fn _put_down(clean: &str, words: &[String]) -> bool {
    if !PUT_DOWN.is_match(clean) {
        return false;
    }
    if any_phrase_in(clean, &BEDTIME) && !any_phrase_in(clean, &LOSS_CONTEXT) {
        // "how do i put my cat to sleep at night" is about a bedtime routine, not a death;
        // a loss still counts when it says when ("last night", "yesterday") or how ("vet").
        return false;
    }
    if _has_pet(words) {
        return true;
    }
    any_phrase_in(clean, &LOSS_CONTEXT)
}

/// Python: bot/compose/loss.py:4243 `_names_a_stored_pet(text, memory)`
/// Whether the line names a pet the store holds: "rex died" is about a pet the bot knows.
pub fn _names_a_stored_pet(text_: &str, memory: &Memory) -> bool {
    let Some(known) = memory.get("user").filter(|known| known.truthy()) else { return false };
    let said = words_of(text_);
    // A pet the observer has just taken off the store is still the animal the line is about.
    for key in ["pets", "pets_gone"] {
        let Some(pets) = known.get(key) else { continue };
        for pet in pets.items() {
            if !pet.is_dict() {
                continue;
            }
            let name = text::lower(text::strip(&py_str(pet.get("name"))));
            if !name.is_empty() && text::len(&name) > 2 && said.contains(&name) {
                return true;
            }
        }
    }
    false
}

/// Python: bot/compose/loss.py:4258 `_has_pet(words)`
pub fn _has_pet(words: &[String]) -> bool {
    any_in(words, &PET_WORDS)
}

/// Python: bot/compose/loss.py:4262 `_a_person_dying(tight)`
/// True when a death that is still ahead is somebody's: "my dad is dying", "shes dying". The
/// person is read from the same words the rest of the loss code uses (`_has_person`).
pub fn _a_person_dying(tight: &str) -> bool {
    let words: Vec<String> =
        text::split_ref(tight).into_iter().map(|word| text::strip_chars(word, ",.?!").to_string()).collect();
    if words.iter().any(|word| ["he", "she", "they", "him", "her", "them"].contains(&word.as_str())) {
        return true;
    }
    _has_person(&words)
}

/// Python: bot/compose/loss.py:4277 `_a_hard_time_that_is_theirs(line)`
/// True when the line says a person close to the user is having the hard time: the relation has
/// to be the subject, the state has to be theirs, and a death has to be in the line.
pub fn _a_hard_time_that_is_theirs(line: &str) -> bool {
    let text_ = lowered(line);
    if !A_RELATION_IN_A_HARD_TIME.is_match(&text_) {
        return false;
    }
    if !DEATH_WORD.is_match(&text_) {
        return false;
    }
    the_animal(&text_).is_empty()
}

/// Python: bot/compose/loss.py:4307 `whose(clean)`
/// 'someone' when the loss belongs to someone else, else 'own'.
pub fn whose(clean: &str) -> String {
    if CHILDS_OWN_LOSS.is_match(clean) && !A_STRANGERS_ANIMAL.is_match(clean) {
        return "own".to_string();
    }
    if SOMEONE_ELSES.is_match(clean)
        || HIS_HERS.is_match(clean)
        || SOMEONE_ELSES_OWN.is_match(clean)
        || A_STRANGERS_ANIMAL.is_match(clean)
    {
        return "someone".to_string();
    }
    "own".to_string()
}

/// Python: bot/compose/loss.py:4364 `one_of(animal)`
/// The animal in the singular, because "A cats is family" is not a sentence.
pub fn one_of(animal: &str) -> String {
    PLURAL_PAIRS.get(animal).cloned().unwrap_or_else(|| animal.to_string())
}

/// Python: bot/compose/loss.py:4377 `the_animal(line)`
/// The animal the line names ("dog", "guinea pig"), or "" when it names none.
pub fn the_animal(line: &str) -> String {
    let text_ = lowered(line);
    let words = words_of(&text_);
    let mut named: Vec<&String> = words.iter().filter(|word| PET_WORDS.contains(*word)).collect();
    if named.is_empty() {
        return String::new();
    }
    let before_words;
    if let Some(death) = DEATH_WORD.search(&text_) {
        before_words = words_of(&text_[..death.start()]);
        let before: Vec<&String> = before_words.iter().filter(|word| PET_WORDS.contains(*word)).collect();
        if !before.is_empty() {
            named = before;
        }
    }
    for (first, second) in TWO_WORD_PETS.iter() {
        if words.contains(first) && words.contains(second) {
            return format!("{first} {second}");
        }
    }
    named[named.len() - 1].clone()
}

/// Python: bot/compose/loss.py:4394 `named_animal(replies, line)`
/// The pool with the animal the line named in it: "my mum's dog died" hears about the dog. "pet"
/// is the word when the line names no animal at all.
pub fn named_animal(replies: &[String], line: &str) -> Vec<String> {
    let mut animal = the_animal(line);
    if animal.is_empty() {
        animal = "pet".to_string();
    }
    let one = one_of(&animal);
    replies.iter().map(|reply| py_format(reply, &[("animal", &animal), ("one", &one)])).collect()
}

/// Python: bot/compose/loss.py:4404 `a_relative_pet(line)`
/// True when the animal that died was someone else's in the family: "my mum's cat died".
pub fn a_relative_pet(line: &str) -> bool {
    let low = lowered(line);
    SOMEONE_ELSES_OWN.search(&low).is_some_and(|found| found.name("who").is_some_and(|who| PET_WORDS.contains(who)))
}

/// Python: bot/compose/loss.py:4410 `a_strangers_animal(line)`
/// True when the animal that died belongs to nobody in the house: "the cat next door".
pub fn a_strangers_animal(line: &str) -> bool {
    A_STRANGERS_ANIMAL.is_match(&lowered(line))
}

/// Python: bot/compose/loss.py:4438 `_a_swap_of_the_question_before(text, memory)`
/// True when the line only swaps the thing in the question before it, and that question asked
/// something this file's funeral card does not answer. The card keeps the line when the question
/// before was about going, about what to say, or when nothing was asked before at all.
pub fn _a_swap_of_the_question_before(text_: &str, memory: &Memory) -> bool {
    let tight_line = _tight(text_);
    if SWAP_ASK.match_(&tight_line).is_none() {
        return false;
    }
    let plain = _plain(text_);
    if plain.len() as i64 > *SWAP_MOST_WORDS {
        return false;
    }
    let mut previous = "";
    for turn in memory.turns.iter().take(8).rev() {
        let line = turn.user.as_str();
        if text::strip(line).is_empty() || _plain(line) == plain {
            continue;
        }
        // A correction ("i meant a funeral") is not the question the swap inherits: the question
        // is the one before it, which the correction was rewriting.
        if ASKS.is_match(&_plain(line).join(" ")) {
            previous = line;
            break;
        }
    }
    if previous.is_empty() || _plain(previous).is_empty() {
        return false;
    }
    A_SWAP_INHERITS.is_match(&_tight(previous))
}

/// Python: bot/compose/loss.py:4467 `_still_ahead(memory)`
/// The pool for a death that has not come yet, or None when it has. A chat about a pet the vet
/// has given weeks to, or a parent in the last stages, is at the days before.
pub fn _still_ahead(memory: &Memory) -> Option<Vec<String>> {
    let record = memory.get("loss").filter(|record| record.is_dict());
    if let Some(record) = record {
        if record.get("ahead").is_some_and(Value::truthy) {
            // The store was told which week this is: "the doctor said theres nothing more they
            // can do" is the days before the death, so the turn after it is not the register office.
            let pool = if record.get("pet").is_some_and(Value::truthy) { &*THE_DAYS_BEFORE } else { &*SOMEONE_DYING };
            return Some(_less_what_was_said(pool, &memory.turns));
        }
    }
    let turns = &memory.turns;
    for turn in last_n(turns, 4).iter().rev() {
        let line = turn.user.as_str();
        let tight_line = _tight(line);
        if line.is_empty() || QUESTION.is_match(line) || !NOT_YET.is_match(&tight_line) {
            continue;
        }
        // The estimate has to be about somebody: "i have three weeks left until my holiday" is a
        // countdown to a holiday, not a death.
        if _has_pet(&_plain(line)) {
            return Some(_less_what_was_said(&THE_DAYS_BEFORE, turns));
        }
        if _a_person_dying(&tight_line) {
            return Some(_less_what_was_said(&SOMEONE_DYING, turns));
        }
    }
    let last = turns.last().map(|turn| turn.bot.as_str()).unwrap_or("");
    if !last.is_empty() && THE_DAYS_BEFORE.iter().any(|line| line == last) {
        return Some(_less_what_was_said(&THE_DAYS_BEFORE, turns));
    }
    if !last.is_empty() && SOMEONE_DYING.iter().any(|line| line == last) {
        return Some(_less_what_was_said(&SOMEONE_DYING, turns));
    }
    None
}

/// Python: bot/compose/loss.py:4507 `_less_what_was_said(pool, turns)`
/// The pool with the advice this chat has already been given taken out: a line the chat has
/// already had - and a line that sends them to the same people - is not offered again.
pub fn _less_what_was_said(pool: &[String], turns: &[Turn]) -> Vec<String> {
    let said: Vec<&str> = turns.iter().map(|turn| text::strip(&turn.bot)).filter(|line| !line.is_empty()).collect();
    if said.is_empty() {
        return pool.to_vec();
    }
    let helpers_said = said.iter().any(|one| _HELPERS.is_match(one));
    let kept: Vec<String> = pool
        .iter()
        .filter(|line| !said.contains(&text::strip(line)) && !(_HELPERS.is_match(line) && helpers_said))
        .cloned()
        .collect();
    if kept.is_empty() { pool.to_vec() } else { kept }
}

/// Python: bot/compose/loss.py:4591 `_an_animal_in_the_chat(memory, turns=6)`
/// True when a line earlier in this chat named an animal of ours. Read off the turns, never
/// assumed.
pub fn _an_animal_in_the_chat(memory: &Memory, turns: i64) -> bool {
    last_n(&memory.turns, turns).iter().any(|turn| PET.is_match(&_tight(&turn.user)))
}

/// Python: bot/compose/loss.py:4608 `_keeping_going(tight, words, memory)`
/// The pool for whether to keep an animal going, or None when the line is not that question.
/// Kept to animals; when the line names no animal, the chat has to have one.
pub fn _keeping_going(tight: &str, words: &[String], memory: &Memory) -> Option<Vec<String>> {
    let judged = KEEPING_JUDGED.is_match(tight);
    if !judged && !ENDING_ASKED.is_match(tight) {
        return None;
    }
    let pool = if judged { &*KEEPING_GOING_REPLIES } else { &*IS_IT_TIME_REPLIES };
    if _has_pet(words) {
        return Some(pool.clone());
    }
    if _has_person(words) {
        return None; // "is it cruel to keep my mum going" is a different question, with its own pools
    }
    if _an_animal_in_the_chat(memory, 6) {
        return Some(pool.clone());
    }
    None
}

/// Python: bot/compose/loss.py:4629 `_first_days(memory)`
/// The pool for the days just after a death: the animal's, or a person's.
pub fn _first_days(memory: &Memory) -> Vec<String> {
    let pet = memory.get("loss").filter(|record| record.is_dict()).and_then(|record| record.get("pet"));
    if pet.is_some_and(Value::truthy) { FIRST_DAYS_PET.clone() } else { FIRST_DAYS_PERSON.clone() }
}

/// Python: bot/compose/loss.py:4652 `_the_visit_ask(clean)`
/// True when the line asks what to say to the person who is dying, not what to tell others.
pub fn _the_visit_ask(clean: &str) -> bool {
    if !SAY_GOODBYE.is_match(clean) {
        return false;
    }
    SAY_TO_WHO.finditer(clean).iter().all(|found| VISIT_TARGETS.contains(found.str(1)))
}

/// Python: bot/compose/loss.py:4662 `_the_animal_of(text, memory)`
/// The animal this turn is about: the one the line names, else the one the loss was about.
pub fn _the_animal_of(text_: &str, memory: &Memory) -> String {
    let animal = the_animal(text_);
    if !animal.is_empty() {
        return animal;
    }
    match memory.get("loss").filter(|record| record.is_dict()) {
        Some(record) => the_animal(&py_str(record.get("who"))),
        None => String::new(),
    }
}

/// Python: bot/compose/loss.py:4673 `_grief_replies(line, memory)`
/// The pool for "does it get easier": a pet's death is answered with the animal in it. Only the
/// person's own animal counts.
pub fn _grief_replies(line: &str, memory: &Memory) -> Vec<String> {
    if whose(line) == "someone" {
        return GRIEF_REPLIES.clone();
    }
    let mut animal = _the_animal_of(line, memory);
    if animal.is_empty() {
        return GRIEF_REPLIES.clone();
    }
    if let Some(singular) = animal.strip_suffix('s') {
        // "my dogs died": the pool says "losing a dog", not "losing a dogs".
        if PET_WORDS.contains(singular) || singular.ends_with(" pig") {
            animal = singular.to_string();
        }
    }
    let article = if animal.chars().next().is_none_or(|c| "aeiou".contains(c)) { "an" } else { "a" };
    PET_GRIEF_REPLIES.iter().map(|reply| py_format(reply, &[("animal", &animal), ("article", article)])).collect()
}

/// Python: bot/compose/loss.py:4693 `_a_colour_is_worn(line)`
/// True when the line asks about a colour of clothes (the knowledge base's own reading); a
/// reading that fails leaves the loss where it was.
pub fn _a_colour_is_worn(line: &str) -> bool {
    caught(|| crate::knowledge::lookup::a_colour_is_worn_text(line)).unwrap_or(false)
}

/// Python: bot/compose/loss.py:4702 `reply_for(text, memory, clean=None, topics=None, mood=None)`
/// The pool the line asks for, or None when it is not about a death.
///
/// The caller picks from the pool, so the wording still varies and never repeats. A follow-up
/// to the bot's own sympathy ("what do I even say to him", "i still can't sleep") is read the
/// same way, because the family of a death keeps asking the same few things.
///
/// `clean` is "" for Python's None (an empty `clean` is read the same way), and `topics` is
/// empty for None (every reading of it treats the two alike).
pub fn reply_for(text_: &str, memory: &Memory, clean: &str, topics: &[String], mood: Option<&str>) -> Option<Vec<String>> {
    let words = words_of(text_);
    let line_owned = if !clean.is_empty() { clean.to_string() } else { words.join(" ") };
    let line = line_owned.as_str();
    let tight_owned = _tight(line);
    let tight = tight_owned.as_str();
    if _a_colour_is_worn(line) && !A_DEATH_HAPPENED.is_match(tight) {
        // Clothes are not the loss. "is white ok at a funeral" is the question "what do I wear to
        // the funeral" with the colour where the verb would go. The reading is the knowledge
        // base's own, so this file keeps no second list of colours. A line that says a death
        // happened keeps the loss even with a colour in it.
        return None;
    }
    if crate::memory::dream::SETTLED_IT.is_match(line) && crate::memory::dream::told_in_the_turn_before(memory) {
        // The line that settles a dream the bot was just told ("hes fine its just a dream") names
        // no frame of its own, and it was read as a bereavement. The dream thread keeps it.
        return None;
    }
    if crate::memory::dream::a_dream_is_not_happening(line) && !crate::memory::dream::a_death_came_first(line) {
        // A death in a dream is not a death. A death in the waking life with a dream beside it
        // ("my dad died and i dreamt about him") is still this module's.
        return None;
    }
    if _only_a_new_subject(line) {
        return None; // the thing the line corrects, not a loss it reports
    }
    if DYING_OTHER.is_match(tight) {
        return None; // "my mum is dying for a coffee", "my mums dying her hair"
    }
    if SPEAKER_TOO.is_match(tight) && !WISH_FRAME.is_match(tight) {
        // Read the other half as it would be read alone: the same clause said of the speaker is
        // taken off first, so "my dog is at the end and so am i" is read as "my dog is at the end"
        // and "the paint is fading and so am i" is read as "the paint is fading" - which is not
        // somebody's week, and stays ordinary.
        let subbed = SPEAKER_TOO.sub(" ", tight);
        let rest = text::strip(&subbed);
        let animal = PET.is_match(rest) && _a_pet_of_ours(rest, Some(&words));
        if is_dying_now(rest)
            || _the_end_is_near(rest, Some(&words))
            || (animal && (AT_THE_END.is_match(rest) || PET_AT_THE_END.is_match(rest) || PET_HAS_HAD_ENOUGH.is_match(rest)))
        {
            // The ending is said of the other one and then of the speaker, and the speaker is the
            // one nobody was answering.
            return Some(SPEAKER_TOO_REPLIES.clone());
        }
    }
    if PET_HAS_HAD_ENOUGH.is_match(tight)
        && PET.is_match(tight)
        && _a_pet_of_ours(line, Some(&words))
        && !SPEAKER_HAS_HAD_ENOUGH.is_match(tight)
    {
        // "my dog wants to die", "my cat has no quality of life": the animal has had enough and
        // the person is facing the days before it, which is the one pool written for that day.
        return Some(THE_DAYS_BEFORE.clone());
    }
    if PET_AT_THE_END.is_match(tight)
        && PET.is_match(tight)
        && _a_pet_of_ours(line, Some(&words))
        && !SPEAKER_HAS_HAD_ENOUGH.is_match(tight)
    {
        // "my dog is at the end", "my cat is ready to go": the same week as "my dog is dying",
        // said the way people say it. Nothing here has died, so every condolence pool is about
        // the wrong day.
        return Some(THE_DAYS_BEFORE.clone());
    }
    if AT_THE_END.is_match(tight) && !SPEAKER_HAS_HAD_ENOUGH.is_match(tight) {
        // The ending said another way, for whichever of ours it is: "my dad is at the end", "my nan
        // is on her last legs". `is_dying_now` carries the reading, so the follow-up after one of
        // these lines ("what do i say to him") knows the week too.
        if is_dying_now(tight) {
            if PET.is_match(tight) && _a_pet_of_ours(line, Some(&words)) {
                return Some(THE_DAYS_BEFORE.clone());
            }
            if line.contains('?') || ASK_WORDS.is_match(tight) || opens_with_an_ask(line) {
                return Some(DYING_SAY_REPLIES.clone());
            }
            return Some(DYING_REPLIES.clone());
        }
    }
    if _the_visit_ask(tight) && dying_in_the_chat(memory, 6) {
        // "how do i say goodbye to her" two turns after "my carer is dying" is about the visit,
        // not the announcement. Read before the follow-up guard below, which wants a pronoun the
        // line may not have ("how do i say goodbye").
        return Some(DYING_SAY_REPLIES.clone());
    }
    let asking = line.contains('?') || ASK_FORM.is_match(tight) || ASK_WORDS.is_match(tight) || opens_with_an_ask(line);
    if EULOGY_WORD.is_match(tight) && EULOGY_ASK.is_match(tight) {
        // a eulogy to write is a writing job: the wording entry has the structure and the length
        return None;
    }
    if WEAR.is_match(tight) {
        // "what do i wear to the funeral" is a question about clothes, and the clothing entry
        // answers it better than the funeral pool can.
        return None;
    }
    if _a_swap_of_the_question_before(text_, memory) {
        // "what about the wake" a turn after "what do i wear to the funeral" is the wear question
        // with a new thing in it, so the entry for that thing answers it. This file's funeral
        // card answers whether to turn up, which is a different question.
        return None;
    }
    if _a_gift_for_someone_in_care(tight) {
        // "my mum is in a hospice, what do i buy her" is a present question asked by somebody whose
        // mother is dying, and the sympathy pool has no present in it. Only the ask, never the
        // statement: "my nan is in a hospice" is the condolence and nothing else.
        return None;
    }
    if _telling_a_child(tight) {
        // how to explain a death to a child is the family entries' question, not sympathy for
        // the adult the line is written by (see TELL_CHILD)
        return None;
    }
    let cost_pool = arrangement_cost_pool(line, Some(tight), memory)
        .filter(|pool| !pool.is_empty())
        .or_else(|| arrangement_cost_pool(text_, Some(tight), memory))
        .filter(|pool| !pool.is_empty());
    if let Some(cost_pool) = cost_pool {
        // "is cremation cheaper than burial": a price question about the arrangements with no
        // death in the line.
        return Some(cost_pool);
    }
    if is_practical_ask(line, Some(tight)) || is_practical_ask(text_, Some(tight)) {
        // A practical question in the middle of a bereavement is not a request for comfort:
        // what a funeral costs, how long probate takes, what to cook for the wake, how to ask
        // for bereavement leave. The answer modules and the honest hedges do better, so the
        // composer stands aside.
        return None;
    }
    if (!QUESTION.is_match(line)
        && !ASK_WORDS.is_match(tight)
        && !NOT_THE_END.is_match(tight)
        && (ESTIMATE.is_match(tight) || _the_end_is_near(tight, Some(&words))))
        && (_has_pet(&words) || _a_person_dying(tight) || CARE_WHO.is_match(tight))
    {
        // The vet's estimate instead of the word itself: "the vet says my cat has a few weeks
        // left", "they gave my dad two months". Nothing has died yet, so the day it is at is the
        // days before it. Only the estimate, never the plain "is dying": that one is
        // `DYING_REPLIES`, below.
        let pet = _has_pet(&words) || _the_vet_said_it(tight, &words);
        return Some(if pet { THE_DAYS_BEFORE.clone() } else { SOMEONE_DYING.clone() });
    }
    if OPEN_ASK.match_(tight).is_some() {
        // "what do i do now" while a death is still ahead is that question, whether or not the
        // store has a record of the loss yet, so the thread has to be read off the turns.
        if let Some(ahead) = _still_ahead(memory) {
            return Some(ahead);
        }
    }
    if lost_in_a_place(text_) {
        // A person lost in a shop or a park is a panic, not a bereavement, and the answer is
        // what to do in the next five minutes.
        return Some(MISLAID_REPLIES.clone());
    }
    if (GET_THROUGH.is_match(tight) || GET_THROUGH_DAY.is_match(tight)) && !GET_THROUGH_TO.is_match(tight) {
        // It is a loss line only when the chat already is one, because the line names no death.
        // "how do i get through the week" on its own is somebody with too much on and is left to
        // the entries.
        let ahead = is_dying_now(tight)
            || dying_in_the_chat(memory, 6)
            || _still_ahead(memory).is_some_and(|pool| !pool.is_empty());
        if _a_thing_of_its_own_to_get_through(tight) {
            // A practical thing got through is that thing's question, and it stays one inside a
            // loss conversation: the rota, the exam, the paperwork all have an entry.
            return None;
        }
        if ahead || continues(memory) || reads_as_loss(text_, Some(&words), _names_a_stored_pet(text_, memory)) {
            if GET_THROUGH_DAY.is_match(tight) {
                // A day with a shape to it is got through by marking it: the ritual pool.
                return Some(REMEMBER_REPLIES.clone());
            }
            if ahead {
                // The days before: the nursing and the grief are the same days.
                return Some(GETTING_THROUGH_DYING_REPLIES.clone());
            }
            return Some(GETTING_THROUGH_REPLIES.clone());
        }
    }
    if !continues(memory) && _an_ending_in_the_chat(memory) {
        // "my girlfriend broke up with me" then "its been 3 weeks" or "will it ever get better":
        // a follow-up in a chat that is a loss without a death in it. `continues` is False
        // because an ending's own answer is an entry's card, not a pool line, and a bereavement
        // chat keeps `TIME_SINCE_REPLIES` below for itself.
        if _how_long_since(tight, &text::split(tight)) && !_names_a_subject(topics) {
            return Some(ENDING_SINCE_REPLIES.clone());
        }
        if A_FEELING_BETTER_ASK.is_match(tight)
            && topics.iter().all(|topic| THREAD_TOPICS.contains(topic) || MEASURE_TOPICS.contains(topic) || topic == "health")
        {
            // "how long will it take to feel better" is read as a health line by the reader, and
            // the health topics are this file's own, so health is allowed here where the span
            // above does not need it.
            return Some(ENDING_FEELINGS_REPLIES.clone());
        }
    }
    if let Some(keeping) = _keeping_going(tight, &words, memory).filter(|pool| !pool.is_empty()) {
        // "is it cruel to keep him going" after "my dog is 12 and going blind": nothing has died,
        // so the condolence pools are about the wrong day. The decision itself is what was asked.
        return Some(keeping);
    }
    if reads_as_loss(text_, Some(&words), _names_a_stored_pet(text_, memory)) {
        if CARD.is_match(tight)
            && (ASK_WORDS.is_match(tight) || asking || (CARD_TO_WRITE.is_match(tight) && WRITE_WORD.is_match(tight)))
        {
            return Some(CARD_REPLIES.clone());
        }
        let ahead_now = is_dying_now(line)
            || _the_end_is_near(tight, Some(&words))
            || (dying_in_the_chat(memory, 6) && (TREATMENT_STOPPED.is_match(tight) || NOTHING_MORE_TO_DO.is_match(tight)));
        if ahead_now && !(FUNERAL.is_match(tight) || WAKE.is_match(tight)) {
            // Still here, so the plain sympathy is about the wrong day (see DYING_NOW). It comes
            // before the "what do i say to them" pool because what to say to someone who is
            // dying is not what to say to someone who has lost them.
            if _a_pet_of_ours(line, Some(&words)) || _the_vet_said_it(tight, &words) {
                return Some(THE_DAYS_BEFORE.clone());
            }
            if ASK_WORDS.is_match(tight) {
                return Some(DYING_SAY_REPLIES.clone());
            }
            if QUESTION.is_match(line) {
                return Some(DYING_GRIEF_REPLIES.clone());
            }
            return Some(DYING_REPLIES.clone());
        }
        if ASK_WORDS.is_match(tight) {
            return Some(SAY_REPLIES.clone());
        }
        if NOT_YET.is_match(tight) && !PUTTING_REGRETTED.is_match(tight) && !QUESTION.is_match(line) {
            // A death that is still ahead is not a condolence for a death that has happened: what
            // helps is the days before it, whether it is a pet or a person. A putting the person
            // says they regret is not still ahead (`PUTTING_REGRETTED`).
            if PET.is_match(tight) {
                return Some(THE_DAYS_BEFORE.clone());
            }
            if _a_person_dying(tight) {
                return Some(SOMEONE_DYING.clone());
            }
        }
        if (FUNERAL.is_match(tight) || WAKE.is_match(tight))
            && (asking || whose(line) == "someone")
            && _feeling(tight).is_empty()
        {
            // "i'm going to his wake tomorrow" wants the same answer as "should i go to the
            // funeral": the user is not the bereaved one, they are the one turning up. A line
            // that names a feeling of theirs is asking about the feeling, not about the attendance.
            if _when_asked(tight) {
                return Some(FUNERAL_WHEN_REPLIES.clone());
            }
            return Some(FUNERAL_REPLIES.clone());
        }
        if marks_a_day(tight, false) && asks_what_to_do(tight) {
            return Some(REMEMBER_REPLIES.clone());
        }
        let feeling = _feeling(tight);
        if !feeling.is_empty() && _asks_about_the_feeling(tight) {
            // A question about the feeling itself is answered by what the feeling is, and the
            // line for that is here.
            return Some(feeling_replies(&feeling));
        }
        if QUESTION.is_match(line) {
            return Some(_grief_replies(line, memory));
        }
        if !feeling.is_empty() {
            return Some(feeling_replies(&feeling));
        }
        if whose(line) == "someone" {
            // Not the user's loss, so the answer is what to say to whoever had it - or, for an
            // animal that is nobody's in the house, a warm line about the animal itself.
            if a_relative_pet(line) {
                return Some(named_animal(&RELATIVE_PET_REPLIES, line));
            }
            if a_strangers_animal(line) {
                return Some(named_animal(&STRANGER_ANIMAL_REPLIES, line));
            }
            if any_in(&words_of(line), &CLOSE) {
                // The line names whose relation died, so the answer names them too. One line of
                // the pool is for a loss the line never names, which is the weaker answer once
                // the person is in the line; the other two carry the sentence to say.
                return Some(NAMED_LOSS_SAY_REPLIES.clone());
            }
            return Some(SAY_REPLIES.clone());
        }
        if PUTTING_REGRETTED.is_match(tight) {
            // "i regret putting my dog down" is grief and a decision in one line. Meet it, then
            // offer the one practical thing that is still theirs to sort out.
            return Some(PUT_DOWN_REGRET_REPLIES.clone());
        }
        // `words or tight.split()`: the words of the line, or of the tight line when there are none.
        let habit_words = if words.is_empty() { text::split(tight) } else { words.clone() };
        if let Some(habit) = _still_here(line, &habit_words).filter(|pool| !pool.is_empty()) {
            // "my dog died yesterday and i keep expecting to see him by the door": the death is in
            // the line and so is the habit, and the habit is the half they are telling us about.
            return Some(habit);
        }
        if _a_hard_time_that_is_theirs(tight) && whose(line) == "own" {
            // The last place the line can land is the plain sympathy pool, and that is the pool
            // meant for the person typing. "my mum is lonely since dad died" is not their
            // bereavement: the death is the reason for her loneliness.
            return Some(THEIR_HARD_TIME_REPLIES.clone());
        }
        return Some(LOSS_REPLIES.clone());
    }
    if !continues(memory) {
        if _missing_someone(tight) {
            // "i miss my mum" with no death mentioned and nothing to continue: the feeling is
            // there to be met, not a problem to be worked out, and not news to be sorry about.
            return Some(MISS_REPLIES.clone());
        }
        return None;
    }
    if dying_in_the_chat(memory, 6) && (QUESTION.is_match(line) || !_feeling(tight).is_empty()) {
        // Somebody at home is dying and the question is now about the person asking: "is it
        // normal to cry all the time" is not answered by the pool for after a death.
        return Some(DYING_GRIEF_REPLIES.clone());
    }
    if dying_in_the_chat(memory, 6) && _the_visit_ask(tight) {
        // "how do i say goodbye to her" two turns after "my carer is dying" is about the visit,
        // not about telling anyone the news.
        return Some(DYING_SAY_REPLIES.clone());
    }
    if thread_pool_is(memory, &MISS_REPLIES)
        && !GRIEF_ASK_FORM.is_match(line)
        && !REFERS_TO_THEM.is_match(tight)
        && !_DEATH_IN_LINE.is_match(tight)
        && _a_body_with_its_own_answer(tight, &text::split(tight))
    {
        // The thread was opened by "i miss my mum", so the missing is already said: what the body
        // is doing ("i cant sleep", "i keep waking at 3", "i have no appetite") is a practical
        // ask, and the answer modules have the answer for it. Asked as a grief question, or after
        // a death said outright, the same words stay here.
        return None;
    }
    if HELLO_ON_ITS_OWN.match_(tight).is_some() {
        // A hello two lines after the sympathy is a hello, not a new subject.
        return Some(HELLO_AFTER.clone());
    }
    if OPEN_ASK.match_(tight).is_some() {
        // "what do i do now" is the question the whole chat is about, and the loss has the answer
        // for the stage it is at.
        return Some(_still_ahead(memory).filter(|pool| !pool.is_empty()).unwrap_or_else(|| _first_days(memory)));
    }
    if marks_a_day(line, true) && asks_what_to_do(line) {
        // "we always did a lunch for her birthday, i don't know what to do instead", a turn after
        // the sympathy: the day is the question, and the tradition is enough here because the
        // chat is already about the loss.
        return Some(REMEMBER_REPLIES.clone());
    }
    // `_thread_pool(memory) in (MISS_REPLIES, AFTER_REPLIES, THE_HABIT_REPLIES)`: the thread pool
    // is the say pool or the miss pool, and of those only the miss pool is in the tuple.
    if !asking
        && _a_habit_in(&format!("{tight} {line}"))
        && (continues(memory) || _thread_pool(memory).is_some_and(|pool| thread_pool_in_habit_set(pool)))
    {
        if let Some(habit) = _still_here(line, &text::split(tight)).filter(|pool| !pool.is_empty()) {
            // "i keep expecting him to ring", a turn after "i missed my dad": the habit is the one
            // specific thing in the line, and the general condolence said back to it tells a
            // person their line was not heard.
            return Some(habit);
        }
    }
    if thread_pool_is(memory, &MISS_REPLIES)
        && !asking
        && STILL_ABOUT_THEM.is_match(tight)
        && !_a_question_about_the_feeling(line, &text::split(tight))
        && !_hands_over_a_detail(tight)
    {
        // "i miss my mum" then "she died two years ago" or "she taught me to bake": the person is
        // still the subject, so the company pool rather than "Noted!". A line that keeps the
        // person in it is the same thread; a question about anything else is that question.
        return Some(AFTER_REPLIES.clone());
    }
    if RITUAL.is_match(tight) && RITUAL_PERSON.is_match(tight) {
        return Some(AFTER_REPLIES.clone());
    }
    if lost_a_thing(line) {
        // "i lost my keys" a turn after sympathy is still a lost bunch of keys.
        return None;
    }
    if CARD.is_match(tight) && (ASK_WORDS.is_match(tight) || (CARD_TO_WRITE.is_match(tight) && WRITE_WORD.is_match(tight)))
    {
        return Some(CARD_REPLIES.clone());
    }
    if ASK_WORDS.is_match(tight) {
        return Some(SAY_REPLIES.clone());
    }
    if (FUNERAL.is_match(tight) || WAKE.is_match(tight)) && asking && _feeling(tight).is_empty() {
        if _when_asked(tight) {
            return Some(FUNERAL_WHEN_REPLIES.clone()); // when and where, whether it is the service or the wake
        }
        return Some(FUNERAL_REPLIES.clone());
    }
    if GRIEF_POINTS_BACK.match_(tight).is_some() {
        // "does that stop", "will this always hurt": a question that points back at the feeling
        // and names nothing else is about the loss, whatever topic word the reader thought it
        // heard in it. A pet's death gets the animal in the answer here too.
        return Some(_grief_replies(line, memory));
    }
    let words_now = text::split(tight);
    if QUESTION.is_match(line)
        && !(_names_a_subject(topics) && !_state_describes_them(&words_now) && !_the_body_is_theirs(tight, &words_now))
    {
        // The grief question shapes ("how long does it take", "why do i", "is it normal") are
        // about the loss only when nothing else in the line is being asked about. The state word
        // still wins when it describes the person - that is `_state_describes_them`.
        return Some(_grief_replies(line, memory));
    }
    if any_in(&words_now, &GRIEF_FEELINGS) {
        // "i miss her", "the funeral is on friday": about the loss wherever they sit in the line.
        if _a_question_about_the_feeling(line, &words_now) {
            // "how do i stop feeling guilty", "i feel sad": the feeling is the whole subject and
            // the chat band answers it, so the condolence stands aside.
            return None;
        }
        return Some(AFTER_REPLIES.clone());
    }
    if _about_family(&words_now) && any_in(&words_now, &GRIEF_SELF) {
        // "my mum isnt coping", "my grandad is struggling": the death is still the subject.
        return Some(AFTER_REPLIES.clone());
    }
    if REFERS_TO_THEM.is_match(tight) && _about_them(tight) && mood != Some("positive") {
        // "i cant stop thinking about him", "i have to sort out her house and i cant face it":
        // still the person who died, still the grief.
        if IDEA_ASK.is_match(tight) || (asking && _own_subject(topics)) {
            return None;
        }
        if thread_pool_is(memory, &SAY_REPLIES) {
            // "what do i say to him" then "its a workmate, i barely know him" is still the same
            // question about what to say, not the person's own bereavement.
            return Some(SAY_REPLIES.clone());
        }
        return Some(AFTER_REPLIES.clone());
    }
    if ABSENCE.is_match(line) && !asking {
        // "i still set the table for two", "the house feels empty": the line says the absence, so
        // a topic word in it does not turn it into a question about that.
        return Some(AFTER_REPLIES.clone());
    }
    if REASSURING.is_match(tight) && !asking && !_own_subject(topics) && !PEACE_IS_MINE.is_match(tight) {
        // "it was the right thing to do", "she had a good innings": the death was handled right
        // and the line is still about it.
        return Some(AFTER_REPLIES.clone());
    }
    if _how_long_since(tight, &words_now) && !_names_a_subject(topics) {
        // "its been a year", "six months now": the gap is the whole of the line, and it is about
        // the person who died. `_names_a_subject` rather than `_own_subject` because "a year and
        // a half" is read as maths and numbers, and half a year said about the gap is still only
        // the gap.
        return Some(TIME_SINCE_REPLIES.clone());
    }
    if _own_subject(topics) && !_state_describes_them(&words_now) {
        return None;
    }
    if (_about_them(tight) || TIME_OF_DAY.is_match(tight)) && _own_state(&words_now) {
        // "i still can't sleep", "some days i can't get out of bed", "the mornings are hard".
        return Some(AFTER_REPLIES.clone());
    }
    if any_phrase_in(tight, &GRIEF_REFERRING) || any_phrase_in(tight, &GRIEF_OBJECTS) {
        // "does it get better", "the house feels so quiet without her".
        return Some(AFTER_REPLIES.clone());
    }
    if mood == Some("negative") {
        return Some(AFTER_REPLIES.clone());
    }
    if ABSENCE.is_match(line) {
        // "i keep expecting to see her" is the same death, said in the way people actually say
        // it in the second week: no grief word in the line at all.
        return Some(AFTER_REPLIES.clone());
    }
    if !topics.is_empty() && topics.iter().all(|topic| ["feelings", "health", "sleep"].contains(&topic.as_str())) {
        return Some(_grief_replies(line, memory));
    }
    if _missing_someone(tight) {
        return Some(MISS_REPLIES.clone());
    }
    None
}

/// `FEELING_REPLIES[feeling]`.
fn feeling_replies(feeling: &str) -> Vec<String> {
    FEELING_REPLIES.get(feeling).cloned().unwrap_or_else(|| panic!("KeyError: {feeling:?}"))
}

/// `pool in (MISS_REPLIES, AFTER_REPLIES, THE_HABIT_REPLIES)` for a pool `_thread_pool` gave.
fn thread_pool_in_habit_set(pool: &'static Vec<String>) -> bool {
    static THE_HABIT_REPLIES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.loss.THE_HABIT_REPLIES"));
    *pool == *MISS_REPLIES || *pool == *AFTER_REPLIES || *pool == *THE_HABIT_REPLIES
}

/// Python: bot/compose/loss.py:5156 `asks_anything(line, tight=None)`
/// True when the line puts a question, however it is written. A bare "?" counts, and so does a
/// wh-word or a modal at the front without one.
pub fn asks_anything(line: &str, tight: Option<&str>) -> bool {
    let own;
    let tight = match tight {
        Some(tight) => tight,
        None => {
            own = _tight(line);
            &own
        }
    };
    line.contains('?') || ASK_FORM.is_match(tight) || opens_with_an_ask(line) || line.starts_with("where")
}

/// Python: bot/compose/loss.py:5208 `_a_gift_for_someone_in_care(tight)`
/// True for a gift asked for a person in care: the present entry answers it, not sympathy.
pub fn _a_gift_for_someone_in_care(tight: &str) -> bool {
    GIFT_THING_ASK.is_match(tight) && IN_CARE.is_match(tight) && !NOT_A_BEDSIDE.is_match(tight)
}

/// Python: bot/compose/loss.py:5214 `is_practical_ask(line, tight=None)`
/// True for a practical question inside a bereavement: what a funeral costs, probate. The
/// composer does not answer these, and the person is still bereaved, so this is also what
/// `continues` reads to keep the thread open past the answer modules' reply.
pub fn is_practical_ask(line: &str, tight: Option<&str>) -> bool {
    let own;
    let tight = match tight {
        Some(tight) => tight,
        None => {
            own = _tight(line);
            &own
        }
    };
    // A line opening with an interrogative word is a question even when it carries no question
    // mark and no ASK_FORM: "who pays for a funeral", "where is the funeral", "which funeral
    // director". Read here, where a practical question after a death is what stands the composer
    // down, rather than across the whole file.
    let asking = asks_anything(line, Some(tight)) || WHO_WHERE_WHICH.match_(tight).is_some();
    // A question that arrives as the second half of a line - "my cat died, should we bury her" -
    // is only read as an ask by the pet card, which is what the shape was added for.
    let opened_late = ASK_WE.is_match(tight) || ASK_TAIL.is_match(line);
    if (FUNERAL.is_match(tight) || FUNERALS.is_match(tight) || WAKE.is_match(tight)) && a_part_of_the_day_asked(tight) {
        // A part of the day - the music, the readings, the flowers, how long the service runs,
        // when the day is, whether the children come - is a practical question about the
        // arrangements, and each has a knowledge row.
        return true;
    }
    if (asking || opened_late) && PET_ARRANGEMENT.is_match(tight) && PET.is_match(tight) {
        // "my cat died, should we bury her in the garden": the pets entry already answers what
        // to do with the body, so it is the card that answers this, not a condolence. A person's
        // body is not read here: there is no card for that yet.
        return true;
    }
    if opened_late && PAPER_ASK.is_match(tight) && !GRIEF_CUE.is_match(tight) {
        // The paperwork asked after the death in the same line is still the paperwork: "my dad
        // died, how long does probate take". Only the papers, and not every practical word.
        return true;
    }
    asking && PRACTICAL.is_match(tight) && !GRIEF_CUE.is_match(tight) && !ASK_WORDS.is_match(tight)
}

/// Python: bot/compose/loss.py:5301 `_an_ending_in_the_chat(memory)`
/// True when the chat holds a break-up, so the gap and the how-long ask are about that. It reads
/// back past this file's own answers and stops at the first turn that is something else.
pub fn _an_ending_in_the_chat(memory: &Memory) -> bool {
    let mut turns = &memory.turns[..];
    let mut stepped = 0;
    while let Some(last) = turns.last() {
        if stepped >= *ENDING_TURNS {
            break;
        }
        if AN_ENDING.is_match(&last.user) {
            return true;
        }
        if !["loss", "grief"].contains(&last.module.as_deref().unwrap_or("")) {
            return false;
        }
        turns = &turns[..turns.len() - 1];
        stepped += 1;
    }
    false
}

/// Python: bot/compose/loss.py:5322 `continues(memory)`
/// True when the bot's last line was sympathy or what to say, so this is still that chat.
///
/// A safety answer does not end the thread: whatever was last said about the death still stands.
/// When the death itself arrived inside the emergency, the sympathy was never said, so the line
/// that was answered as an emergency is read instead. A practical question inside a bereavement
/// is answered by the answer modules, and the answer must not close the thread: the thread reads
/// past up to THREAD_TURNS of those turns, and it closes on any other kind of line.
pub fn continues(memory: &Memory) -> bool {
    let mut turns = &memory.turns[..];
    if turns.is_empty() {
        return false;
    }
    let mut looked_past = false;
    let mut stepped = 0;
    while let Some(last) = turns.last() {
        if stepped >= *THREAD_TURNS {
            break;
        }
        if SAFETY_MODULES.iter().any(|module| module == last.module.as_deref().unwrap_or("")) {
            turns = &turns[..turns.len() - 1];
            looked_past = true;
            continue;
        }
        if is_practical_ask(&last.user, None) {
            turns = &turns[..turns.len() - 1];
            looked_past = true;
            stepped += 1;
            continue;
        }
        break;
    }
    let Some(last) = turns.last() else {
        if !looked_past {
            return false;
        }
        return memory.turns.iter().any(|turn| reads_as_loss(&turn.user, None, false));
    };
    if !last.bot.is_empty() && POOL_LINES.contains(&last.bot) {
        return true;
    }
    // A death answered by another module ("my cat was put down last week" -> the pet entry) is a
    // bereavement turn too, as long as it was not the fallback that answered.
    let module = last.module.as_deref().unwrap_or("");
    !module.is_empty()
        && !FALLBACK_MODULES.iter().any(|m| m == module)
        && !SAFETY_MODULES.iter().any(|m| m == module)
        && reads_as_loss(&last.user, None, false)
}
