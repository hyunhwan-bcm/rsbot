//! Port of `bot/knowledge/lookup.py`: the shared matcher that finds the knowledge entry a line
//! asks for (`find`), the `Entry` every domain builds its knowledge from (shared with main as
//! `knowledge::entry`), and the guards and replies lookup.py defines for the domains to use.
//!
//! The module is split by what the Python file does, in its order:
//! - `guards`: the small readers and the guard / reply functions entries name (L72-L1700);
//! - `forms`: how a key word is typed (`_forms`) and where a phrase sits in a line;
//! - `score`: whether an entry's key is said (`score`, `_rank`, the frame and report readers);
//! - `find`: `find`, `_the_best_entry`, `_best` and the other readings it falls back on.
//!
//! Python's `seen` memo is [`Seen`] (shared as Python's dict is); a line's word forms are a
//! [`FormLine`], whose `id` stands in for Python's `forms is` identity.

use crate::data;
use crate::py::re::Re;
use crate::py::text;
use crate::understood::Understood;
use std::sync::{Arc, LazyLock, RwLock};

/// A fast hasher for the matcher's sets of short words (FxHash, as rustc uses): the sets are
/// Python's sets, and only membership is ever asked of them, so the hasher changes nothing but
/// speed.
#[derive(Default, Clone, Copy)]
pub struct Fx(u64);

impl std::hash::Hasher for Fx {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        const K: u64 = 0x51_7c_c1_b7_27_22_0a_95;
        let mut h = self.0;
        let mut chunks = bytes.chunks_exact(8);
        for c in &mut chunks {
            h = (h.rotate_left(5) ^ u64::from_le_bytes(c.try_into().unwrap())).wrapping_mul(K);
        }
        for &b in chunks.remainder() {
            h = (h.rotate_left(5) ^ b as u64).wrapping_mul(K);
        }
        self.0 = h;
    }
    #[inline]
    fn write_u8(&mut self, b: u8) {
        self.0 = (self.0.rotate_left(5) ^ b as u64).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }
    #[inline]
    fn write_usize(&mut self, n: usize) {
        self.0 = (self.0.rotate_left(5) ^ n as u64).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

pub type FxBuild = std::hash::BuildHasherDefault<Fx>;
/// `set` / `frozenset` in the matcher.
pub type HashSet<T> = std::collections::HashSet<T, FxBuild>;
/// `dict` in the matcher.
pub type HashMap<K, V> = std::collections::HashMap<K, V, FxBuild>;

pub use crate::knowledge::entry::{Entry, KeyPart, Reply};

mod find;
mod forms;
mod guards;
mod score;

pub use find::*;
pub use forms::*;
pub use guards::*;
pub use score::*;

pub(crate) const M: &str = "bot.knowledge.lookup.";

fn v(name: &str) -> &'static data::Value {
    data::get(&format!("{M}{name}"))
}

macro_rules! sets {
    ($($n:ident),* $(,)?) => { $(pub(crate) static $n: LazyLock<HashSet<String>> = LazyLock::new(|| v(stringify!($n)).str_set().into_iter().collect());)* };
}
macro_rules! lists {
    ($($n:ident),* $(,)?) => { $(pub(crate) static $n: LazyLock<Vec<String>> = LazyLock::new(|| v(stringify!($n)).strs());)* };
}
macro_rules! res {
    ($($n:ident),* $(,)?) => { $(pub(crate) static $n: LazyLock<&'static Re> = LazyLock::new(|| v(stringify!($n)).as_re().unwrap_or_else(|| panic!("{} is not a regex", stringify!($n))));)* };
}
macro_rules! ints {
    ($t:ty; $($n:ident),* $(,)?) => { $(pub(crate) static $n: LazyLock<$t> = LazyLock::new(|| v(stringify!($n)).as_int().unwrap_or_else(|| panic!("{} is not an int", stringify!($n))) as $t);)* };
}
macro_rules! strings {
    ($($n:ident),* $(,)?) => { $(pub(crate) static $n: LazyLock<String> = LazyLock::new(|| v(stringify!($n)).str().to_string());)* };
}

sets!(
    COMMON_WORDS, GRAMMAR_ONLY, ASKING_MODALS, ASKING_SUBJECTS, _NOT_A_SUBJECT, _NOT_A_THING,
    ASK_FRAME_IDS, AS_A_KIND, GONE_INTENSIFIER, GONE_STATE, SMELL_VERBS, _BAD_IS_A_STATE, OFF_SAID_OF_FOOD,
    A_GOOD_SMELL, BEFORE_OFF, _CHILD_WORDS, _CHILD_PHRASES, A_DEVICE_THAT_SLEEPS, _A_PERSON_WHO_SLEEPS,
    _CONTRACTION, NOT_THE_STEM, NOT_A_WORD_WITH_AN_S, NOT_THE_STEM_WITH_E, NOT_A_BASE, NOT_THE_DOUBLED_STEM,
    _FUNCTION_WORDS, DEAD_AS_VERY_AFTER, COOKING_AS_A_NAME, CHILD_PLACE_WORDS, CHILD_SIZE_WORDS,
    CHILD_AFTER_A_SIZE, EVERYDAY_CHILD, NOT_A_CHILD_SAID, _THE_PRONOUNS, _COMMON, _STEMS, PLURAL_ONLY_PHRASES,
    _ARTICLE_WORDS, _ARTICLE_FORMS, _SAID_BETWEEN, WANT_FILLER, ASK_PEOPLE, DENY_SAID_OF, DENIERS,
    STATES_AFTER, DENY_BETWEEN, ASK_FRAME, WHEN_WORDS, A_LENGTH_WORD, A_STAND_IN, A_SAYS_NOTHING, FRAME_ENDS,
    FRAME_PRONOUNS, ASK_PHRASE_WORDS, REPORT_LEADS, REPORT_PERSON, REPORT_FILLER, HAPPENING_AGAIN,
    EXPLAIN_LEADS, EXPLAIN_FILLER, THEIR_OWN_HANDS_THINGS, _KEEPING_VERB, _RESULT_VERB, _SPOILING, _ING_NOUNS,
    _KEEPING_KEY, _LIFE_KEY, _RECURRING, _RECURRING_ANYWAY, _RECURRING_WITH, _SUBJECT_POINTS,
    NOT_THE_ADJECTIVE, _A_PERSON, _NOT_A_THING_HAD, _FILLER, FEELING_ABOUT, FEELING_ENDS,
    FEELING_NAMES_NOTHING, FEELING_DOMAINS, FEELING_FAULT_WORDS, FEELING_SPEAKER, FEARED_THINGS,
    _KEPT_SOMEWHERE, _POINTS_AT_IT, _THE_DEGREE_WORDS, _THE_HEAT_OF_THE_DAY, _THE_WH_WORDS, _ASK_FRAME,
    _PLACES_PEOPLE_LIVE_IN, _COLD_WORDS, _BETWEEN_A_PLACE_AND_ITS_COLD, _HOW_A_PERSON_IS_COLD,
    _SAYS_OF_THEMSELVES, _DEFINITION_TAIL, _DEFINITION_POINTERS, _DEFINITION_TROUBLE, _DEFINITION_EMPTY,
    _DEFINING_VERBS, _BE_VERBS, _DEFINITION_ARTICLES, _BEFORE_THE_THING, NAME_ALONE_DOMAINS, NOT_A_NAME_ALONE,
    A_STATE_AS_A_NAME_END, A_STATE_IN_THE_PHRASE, POINTS_BACK, FOLLOW_ON_WORDS, ASK_FRAME_WORDS,
    SUBSTITUTE_WORDS, RETURNED_AFTER, BODY_WORDS, GENERAL_BODY, _CUT_BESIDE, _STILL_GOOD_ASHERS,
    _KEEPING_WORDS, COLOUR_WORDS, ASKING_FIRST, COLOUR_ASKED_WORDS, WEARING, AFTER_A_COLOUR_STANDS,
    WORN_AFTER_A_COLOUR, _BARE_IN_A_KEY, WEIGHT_UNITS, PERSON_PRONOUNS,
);

// Tuples: kept in order where the code walks them; membership tests use the same Vec.
lists!(
    HOW_OFTEN_WORDS, SAID_LEADS, ENDINGS, APPLIANCE_NAMES, PEOPLE_MALE, PEOPLE_FEMALE,
    WHERE_PHRASES, PRICE_COMPARED, WHERE_PART_WORDS, STOPPED_WORKING, FOOD_NAMED, OFF_STATE_ASKS,
    KINDS_WORDS, KINDS_FOLLOW_UP_WORDS, WANTING, WANT_KNOWING, ASK_PERSON_PRONOUNS, ASK_PERSON_POSSESSIVES,
    FRAME_WORDS, KEPT_FOODS, KEEPING_WORDS, CONTAINERS, FEELING_ABOUT_A_THING, PLAN_WORDS, _KEEPING_ANSWER,
    A_DRESS_CODE_AFTER, SPEAKER_ONLY, HANDLING_WORDS, GONE_OFF_SIGNS, _BRIDGES,
);

res!(
    WORD, _SAVING_VERB, _SAVING_FOR_A_TRIP, _AFFORD_THE_TRIP, SANDING_JOB, BACK_OF_THE_HOUSE, THE_BACK_ITSELF,
    HAS_GONE, _BORROWING_IN_THE_LINE, _OFF_A_PERSON, _FLAT_YOU_LIVE_IN, _THE_HEAT_OF_AN_ARGUMENT,
    _ORDER_OF_STEPS, _A_BAKE_OR_A_PAN, _CAME_OUT_FLAT, _THE_GROUND_UNDERFOOT, _YEAR_OLD_FIGURE,
    _A_THING_SAID_IS, _A_THING_JUST_BEFORE, _A_THING_POINTED_AT, _PEOPLE_AGED, _A_YEARS_OLD, _KEEPING_AN_EYE,
    SHORT_OF_SLEEP, NOT_TIRED_AFTER_ALL, KINDS_ASKS_ME, KINDS_ASKS_WITH_A_VERB, KINDS_SAYS_HOW_IT_IS,
    KINDS_NAMED_A_USE, _ABOUT_THE_CHAT, _THROAT, _BRIDGE, _PUNCTUATION, CARER_FRAME, AN_ILLNESS,
    CHILD_BY_PLACE, A_CHILD_WORD_BY_ITSELF, A_CHILD_GETTING_OLDER, A_TEEN, TWO_OF_US, ASKS_FOR_THE_HANDS,
    A_HAND_OR_EYE_DOES_IT, ASKS_WHAT_IT_SAYS, OWN_THING_ON_THEIR_SIDE, GOES_IN, COUNTED_INTO_A_THING,
    HOW_MANY_DAYS, KEEPS_OR_STORES, _WHERE_I_PUT, _WHERE_TO_KEEP, _HOLDS_MONEY, _THE_THING_JUDGED,
    _A_PLACE_ON_THEM, _NOT_GONE, _SUBJECT_LEAD, _POSSESSIVE, _DEFINITION_ASK, _A_KEEP_AND_A_LENGTH,
    A_COLOUR_USED, A_COLOUR_ASKED, OCCASION, _WEAR_ASK, MISSING_WORDS,
);

ints!(i64; MIN_SCORE, WORD_WEIGHT, PHRASE_WEIGHT, BARE_WEIGHT, PLAIN_MARGIN, _MIN_BARE_WORDS);
ints!(usize; MIN_FORM, MIN_TYPO_LENGTH, _A_BREATH, CONTEXT_WORDS, SPECIFIC, COMMON_WORD, ASK_NAME_CAP, ASK_PLAIN_CAP, SPECIFIC_PARTS, _SUBJECT_AT_MOST);

strings!(_SLEEPY_MARKS, _MARKS, KINDS_SHAPE);

/// `_WRAPPED`: the frames a subject is wrapped in ("what should i do about <subject>").
pub(crate) static _WRAPPED: LazyLock<Vec<&'static Re>> = LazyLock::new(|| v("_WRAPPED").items().iter().map(|r| r.as_re().unwrap()).collect());
/// `THE_FAULT`: (word, the fault's name) pairs.
pub(crate) static THE_FAULT: LazyLock<Vec<(String, String)>> = LazyLock::new(|| pairs("THE_FAULT"));
/// `SLIPS`: {word: the ids of the entries whose key misspells it on purpose}.
pub(crate) static SLIPS: LazyLock<HashMap<String, HashSet<String>>> =
    LazyLock::new(|| v("SLIPS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_set().into_iter().collect())).collect());
pub(crate) static MODAL_PHRASES: LazyLock<HashMap<String, Vec<String>>> =
    LazyLock::new(|| v("MODAL_PHRASES").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect());
pub(crate) static IRREGULAR_PLURALS: LazyLock<HashMap<String, String>> = LazyLock::new(|| v("IRREGULAR_PLURALS").str_map().into_iter().collect());
/// `THE_SAME_OCCASION`: {family: the words of it}, in the dict's order.
pub(crate) static THE_SAME_OCCASION: LazyLock<Vec<(String, HashSet<String>)>> =
    LazyLock::new(|| v("THE_SAME_OCCASION").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_set().into_iter().collect())).collect());
pub(crate) static HOUSEHOLD_WORDS: LazyLock<HashMap<String, String>> = LazyLock::new(|| v("HOUSEHOLD_WORDS").str_map().into_iter().collect());
pub(crate) static PART_INSIDE: LazyLock<HashMap<String, String>> = LazyLock::new(|| v("PART_INSIDE").str_map().into_iter().collect());
/// `_OF_THE_RULE_OUT`: the words a ruled-out phrase is built from (keys only matter).
pub(crate) static _OF_THE_RULE_OUT: LazyLock<HashSet<String>> =
    LazyLock::new(|| v("_OF_THE_RULE_OUT").pairs().iter().filter(|(_, v)| !v.is_none()).map(|(k, _)| k.str().to_string()).collect());

pub(crate) static US_SPELLINGS: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.understanding.lexicon.US_SPELLINGS").into_iter().collect());
pub(crate) static LEX_CHAT: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.understanding.lexicon.CHAT").into_iter().collect());
pub(crate) static LEX_EXPAND: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.understanding.lexicon.EXPAND").into_iter().collect());
pub(crate) static HANDOFF_LEADS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.understanding.lexicon.HANDOFF_LEADS"));
pub(crate) static BODY_PARTS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.understanding.lexicon.BODY_PARTS").into_iter().collect());

fn pairs(name: &str) -> Vec<(String, String)> {
    v(name)
        .items()
        .iter()
        .map(|p| {
            let it = p.strs();
            (it[0].clone(), it[1].clone())
        })
        .collect()
}

/// The alternatives of a key part (`part if isinstance(part, tuple) else (part,)`).
pub fn alternatives(part: &KeyPart) -> &[String] {
    match part {
        KeyPart::Word(s) => std::slice::from_ref(s),
        KeyPart::Any(v) => v,
    }
}

/// Every entry in the knowledge base (`bot.knowledge.entries()`).
pub fn all_entries() -> &'static [Entry] {
    crate::knowledge::entries()
}

/// Every entry as references, for the functions that take Python's `entries` list.
pub fn all_entry_refs() -> &'static [&'static Entry] {
    static REFS: LazyLock<Vec<&'static Entry>> = LazyLock::new(|| all_entries().iter().collect());
    &REFS
}

/// `str.strip(chars)` over Python's characters.
pub(crate) fn strip_chars<'a>(s: &'a str, chars: &str) -> &'a str {
    s.trim_matches(|c| chars.contains(c))
}

/// `"".join(words)` of a reading's words, the way the Python code reads `understood.get(key)`
/// for the keys a reading holds as text.
pub(crate) fn get_text<'a>(u: &'a Understood, key: &str) -> &'a str {
    match key {
        "clean" => &u.clean,
        "text" => &u.text,
        "plain" => &u.plain,
        "norm_text" => &u.norm_text,
        _ => u.extra.get(key).and_then(|v| v.as_str()).unwrap_or(""),
    }
}

/// `a or b or ''` over the text keys of a reading.
pub(crate) fn first_text<'a>(u: &'a Understood, keys: &[&str]) -> &'a str {
    for k in keys {
        let t = get_text(u, k);
        if !t.is_empty() {
            return t;
        }
    }
    ""
}
