//! Port of PyBot's `bot/understanding` package (`bot/understanding/__init__.py` is this file).
//! Each Python file `bot/understanding/<name>.py` is `<name>.rs` here.
//!
//! `understand(text, memory)` is the reading every module looks at first.

use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

pub mod attempts;
pub mod body;
pub mod breeds;
pub mod car_glass;
pub mod cars;
pub mod confusion;
pub mod kinds;
pub mod lexicon;
pub mod normalize;
pub mod pyuni;
pub mod references;
pub mod senses;
pub mod topics;

/// A regex built from a pattern made at run time (`re.search(r"\b" + re.escape(cue) ...)`),
/// compiled once per distinct pattern and kept for the life of the process (the patterns come
/// from PyBot's own tables, so there are finitely many).
pub fn rx(pattern: &str) -> &'static Re {
    rx_flags(pattern, 0)
}

/// [`rx`] with Python flag bits (`re::I`, ...).
pub fn rx_flags(pattern: &str, flags: u32) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<(String, u32), &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let key = (pattern.to_string(), flags);
    if let Some(r) = CACHE.lock().unwrap().get(&key) {
        return r;
    }
    let r: &'static Re = Box::leak(Box::new(Re::new(pattern, flags)));
    CACHE.lock().unwrap().insert(key, r);
    r
}

// ---- bot/understanding/__init__.py ----

const M: &str = "bot.understanding.";

mod tables {
    #![allow(non_upper_case_globals, dead_code)]
    use super::{t, M};
    use crate::py::re::Re;
    use indexmap::IndexMap;
    use std::collections::HashSet;
    use std::sync::LazyLock;
    pub(super) static ABOUT_THE_BOT: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "ABOUT_THE_BOT"));
    pub(super) static AFTER_BOT_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AFTER_BOT_FILLER"));
    pub(super) static ALL_ON_THEM: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ALL_ON_THEM"));
    pub(super) static ANY_PRONOUN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ANY_PRONOUN"));
    pub(super) static ASKING_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASKING_WORDS"));
    pub(super) static ASKS_WHY_IT_IS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKS_WHY_IT_IS"));
    pub(super) static ASK_FREE_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_FREE_FILLER"));
    pub(super) static ASK_FREE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_FREE_WORDS"));
    pub(super) static ASK_PERMISSION: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "ASK_PERMISSION"));  // t, 23 items
    pub(super) static AT_IT_STILL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AT_IT_STILL"));
    pub(super) static A_CLAUSE_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_CLAUSE_VERB"));
    pub(super) static A_CONDITION_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_CONDITION_AFTER"));
    pub(super) static A_DESCRIBING_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_DESCRIBING_SUBJECT"));
    pub(super) static A_FAULT_TO_LEND: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_FAULT_TO_LEND"));
    pub(super) static A_STAND_IN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_STAND_IN"));
    pub(super) static A_VALUE_IN_FRONT_OF_NOT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_VALUE_IN_FRONT_OF_NOT"));
    pub(super) static BOT_ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BOT_ASK_WORDS"));
    pub(super) static BOT_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BOT_NOUNS"));
    pub(super) static BOT_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BOT_STATE"));
    pub(super) static BROKEN_OF_A_THING: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "BROKEN_OF_A_THING"));  // t, 2 items
    pub(super) static CARRY_PASS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CARRY_PASS"));
    pub(super) const CARRY_TOPIC: i64 = 2;
    pub(super) const CARRY_WORDS: i64 = 8;
    pub(super) static CHAT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CHAT_WORDS"));
    pub(super) static CONSTRAINT_FACT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONSTRAINT_FACT"));
    pub(super) static COPULA_AUX: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COPULA_AUX"));
    pub(super) static CORRECTION_HEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CORRECTION_HEAD"));
    pub(super) static CORRECTION_MARKER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CORRECTION_MARKER"));
    pub(super) static CORRECTION_OPENER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CORRECTION_OPENER"));
    pub(super) static DAY_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DAY_VERBS"));
    pub(super) static DAY_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DAY_WORDS"));
    pub(super) static DETERMINER_TAIL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DETERMINER_TAIL"));
    pub(super) static DID_IT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DID_IT"));
    pub(super) static DOES_ALL_THEY_CAN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DOES_ALL_THEY_CAN"));
    pub(super) static DOES_IT_ALL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DOES_IT_ALL"));
    pub(super) static DOES_IT_RIGHT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DOES_IT_RIGHT"));
    pub(super) static EDGE_PUNCTUATION: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "EDGE_PUNCTUATION"));
    pub(super) static EVERYTHING_AT_ONCE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "EVERYTHING_AT_ONCE"));
    pub(super) static FIRST_PERSON_ANYWHERE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FIRST_PERSON_ANYWHERE"));
    pub(super) static FOLLOW_OPENER: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| t::tuples(M, "FOLLOW_OPENER"));
    pub(super) static GOOD_TIME_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GOOD_TIME_WORDS"));
    pub(super) static GREETING_PHRASE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GREETING_PHRASE"));
    pub(super) static HELPS_ME_NOT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HELPS_ME_NOT"));
    pub(super) const HOME_LOOKBACK: i64 = 8;
    pub(super) static HOME_SCOPE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HOME_SCOPE"));
    pub(super) static HYPOTHETICAL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HYPOTHETICAL"));
    pub(super) static INTRODUCING_A_STATE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "INTRODUCING_A_STATE"));
    pub(super) static LEANS_ON_THEM: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LEANS_ON_THEM"));
    pub(super) const LOOKBACK: i64 = 4;
    pub(super) static MANNER_TALK: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MANNER_TALK"));
    pub(super) const MAX_KEYWORDS: i64 = 14;
    pub(super) static MEANING_TAIL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MEANING_TAIL"));
    pub(super) static MEANING_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MEANING_WORDS"));
    pub(super) static MODAL_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MODAL_WORDS"));
    pub(super) static MOMENT_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MOMENT_FILLER"));
    pub(super) static MOMENT_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MOMENT_VERBS"));
    pub(super) static MOMENT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MOMENT_WORDS"));
    pub(super) static MORE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MORE_WORDS"));
    pub(super) static NOTHING_NAMED_ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOTHING_NAMED_ASK_WORDS"));
    pub(super) static NOTHING_NAMED_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOTHING_NAMED_FILLER"));
    pub(super) static NOTHING_NAMED_VIEW_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOTHING_NAMED_VIEW_WORDS"));
    pub(super) static NOT_A_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_SUBJECT"));
    pub(super) static NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_THING"));
    pub(super) static NOT_MANNER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_MANNER"));
    pub(super) static NO_ONE_HELPS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NO_ONE_HELPS"));
    pub(super) static NO_TIME_OFF: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NO_TIME_OFF"));
    pub(super) static OTHER_PRONOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OTHER_PRONOUNS"));
    pub(super) static OWN_QUESTION_TOPIC: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "OWN_QUESTION_TOPIC"));
    pub(super) static OWN_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OWN_VERB"));
    pub(super) static PERMISSION_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PERMISSION_FILLER"));
    pub(super) static PERSON_HEADS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PERSON_HEADS"));
    pub(super) static PERSON_STOP: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PERSON_STOP"));
    pub(super) static POSSESSIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "POSSESSIVE"));
    pub(super) static PRAISED: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PRAISED"));
    pub(super) static PRESENCE_ASKER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PRESENCE_ASKER"));
    pub(super) static PRESENCE_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PRESENCE_FILLER"));
    pub(super) static PRESENCE_GREETING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PRESENCE_GREETING"));
    pub(super) const PRESENCE_MAX: i64 = 5;
    pub(super) static PRESENCE_NOT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PRESENCE_NOT"));
    pub(super) static PRESENCE_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PRESENCE_SUBJECT"));
    pub(super) static PRESENCE_WORD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PRESENCE_WORD"));
    pub(super) static PROPERTY_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PROPERTY_ASK"));
    pub(super) static PROPERTY_SUBJECTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PROPERTY_SUBJECTS"));
    pub(super) static REACHING_KEYS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "REACHING_KEYS"));  // t, 6 items
    pub(super) static RUNS_MYSELF_RAGGED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RUNS_MYSELF_RAGGED"));
    pub(super) static RUNS_THE_HOUSE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RUNS_THE_HOUSE"));
    pub(super) static SECOND_JOIN_HEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SECOND_JOIN_HEAD"));
    pub(super) static SECOND_JOIN_TAIL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SECOND_JOIN_TAIL"));
    pub(super) static SECOND_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SECOND_PERSON"));
    pub(super) static SHIFTS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "SHIFTS"));  // t, 18 items
    pub(super) static SHIFT_ACTUALLY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SHIFT_ACTUALLY"));
    pub(super) static SPEAKER_SUBJECTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SPEAKER_SUBJECTS"));
    pub(super) static STAND_IN_NOT_THINGS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STAND_IN_NOT_THINGS"));
    pub(super) static STAND_IN_SELECTORS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STAND_IN_SELECTORS"));
    pub(super) static STAND_IN_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STAND_IN_WORDS"));
    pub(super) static STOPWORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STOPWORDS"));
    pub(super) static SUBJECT_TYPES: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "SUBJECT_TYPES"));  // t, 5 items
    pub(super) static TALK_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TALK_ASK"));
    pub(super) static TALK_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TALK_FILLER"));
    pub(super) static TALK_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TALK_VERBS"));
    pub(super) static THANKED_FOR: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THANKED_FOR"));
    pub(super) static THE_ONE_WHO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "THE_ONE_WHO"));
    pub(super) static THIRD_PERSON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "THIRD_PERSON"));
    pub(super) static WHO_ENDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WHO_ENDS"));
    pub(super) static WORK_SCOPE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WORK_SCOPE"));
    pub(super) static WORN_OUT_OF_A_PERSON: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "WORN_OUT_OF_A_PERSON"));
    pub(super) static _A_CLAUSE_WITH_NO_SUBJECT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_CLAUSE_WITH_NO_SUBJECT"));
    pub(super) static _A_THING_THAT_CHANGES: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_THING_THAT_CHANGES"));
    pub(super) static _NOT_ABOUT_THE_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_NOT_ABOUT_THE_THING"));
    pub(super) static _SEARCH_FRAME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_SEARCH_FRAME"));
    /// `SAME_KIND_WORDS`: (kind, words) pairs.
    pub(super) static SAME_KIND_WORDS: LazyLock<Vec<(String, HashSet<String>)>> =
        LazyLock::new(|| t::value(M, "SAME_KIND_WORDS").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].str_set())).collect());
    pub(super) static COUNT_UNITS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.normalize.", "COUNT_UNITS"));
    pub(super) static KINDS_ADDRESS_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.kinds.", "ADDRESS_WORDS"));
    pub(super) static KINDS_MEANT_APPLIED_TO: LazyLock<&'static Re> = LazyLock::new(|| t::re("bot.understanding.kinds.", "MEANT_APPLIED_TO"));
    pub(super) static KINDS_MOOD_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.kinds.", "MOOD_NOUNS"));
    pub(super) static KINDS_NEEDS_ANSWER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.kinds.", "NEEDS_ANSWER"));
    pub(super) static KINDS_OTHER_PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.kinds.", "OTHER_PEOPLE"));
    pub(super) static KINDS_PERSON_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.kinds.", "PERSON_NOUNS"));
    pub(super) static KINDS_PRAISE_SAID: LazyLock<&'static Re> = LazyLock::new(|| t::re("bot.understanding.kinds.", "PRAISE_SAID"));
    pub(super) static KINDS_PRAISING_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.kinds.", "PRAISING_VERB"));
    pub(super) static KINDS_SIGNING_OFF: LazyLock<&'static Re> = LazyLock::new(|| t::re("bot.understanding.kinds.", "SIGNING_OFF"));
    pub(super) static KINDS_STILL_WRONG: LazyLock<&'static Re> = LazyLock::new(|| t::re("bot.understanding.kinds.", "STILL_WRONG"));
    const LX: &str = "bot.understanding.lexicon.";
    pub(super) static LX_A_DESCRIBING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "A_DESCRIBING"));
    pub(super) static LX_A_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "A_STATE"));
    pub(super) static LX_COMMON_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "COMMON_NAMES"));
    pub(super) static LX_CONFUSIONS: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(LX, "CONFUSIONS"));
    pub(super) static LX_CONFUSIONS_BEFORE_A_QUESTION: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(LX, "CONFUSIONS_BEFORE_A_QUESTION"));
    fn pairs(name: &str) -> IndexMap<String, (String, HashSet<String>)> {
        t::value(LX, name).pairs().iter().map(|(k, v)| (k.str().to_string(), (v.items()[0].str().to_string(), v.items()[1].str_set()))).collect()
    }
    pub(super) static LX_CONFUSIONS_AFTER: LazyLock<IndexMap<String, (String, HashSet<String>)>> = LazyLock::new(|| pairs("CONFUSIONS_AFTER"));
    pub(super) static LX_CONFUSIONS_BEFORE: LazyLock<IndexMap<String, (String, HashSet<String>)>> = LazyLock::new(|| pairs("CONFUSIONS_BEFORE"));
    pub(super) static LX_CONFUSIONS_IN_A_FEELING: LazyLock<IndexMap<String, (String, HashSet<String>)>> = LazyLock::new(|| pairs("CONFUSIONS_IN_A_FEELING"));
    pub(super) static LX_FEELING_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "FEELING_FILLER"));
    pub(super) static LX_FEELING_LEAD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "FEELING_LEAD"));
    pub(super) static LX_QUESTION_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "QUESTION_WORDS"));
    pub(super) static LX_TRAITS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "TRAITS"));
    pub(super) static TOPICS_COMMON_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.topics.", "COMMON_WORDS"));
}
use tables::*;

use crate::data::Value;
use crate::py::text;
use crate::understanding::kinds::Read;
use crate::understanding::normalize::{background_positions, normalise, Line};
use crate::understood::{Entity, FixItem, Fixes, LeansOn, StandIn};
use std::collections::{BTreeSet, HashSet};

fn sv(norm: &[String]) -> HashSet<&str> {
    norm.iter().map(String::as_str).collect()
}

/// `memory.get("topic")`: the topic record a module keeps (a dict, or a topic name).
fn topic_record(memory: Option<&Memory>) -> Option<&Value> {
    memory.and_then(|m| m.extra.get("topic"))
}

/// A memory with nothing in it (`not memory`).
fn memory_is_empty(memory: &Memory) -> bool {
    memory.understood.is_none() && memory.intent_wants.is_none() && memory.asked_now.is_none() && memory.turns.is_empty() && memory.extra.is_empty()
}

/// `_the_second_thing(text)`: the thing a short second line swaps in ("and the brakes?").
fn _the_second_thing(text_: &str) -> String {
    let line = text::strip(text_);
    if line.is_empty() {
        return String::new();
    }
    let mut correction = CORRECTION_HEAD.match_(line);
    if correction.is_some() && !_says_which_value_is_meant(line) {
        correction = None;
    }
    let head_end = match &correction {
        Some(c) => Some(c.end()),
        None => SECOND_JOIN_HEAD.match_(line).map(|m| m.end()),
    };
    let rest = match head_end {
        Some(e) => &line[e..],
        None => line,
    };
    let tail = SECOND_JOIN_TAIL.search(rest);
    let core = text::strip(match &tail {
        Some(t_) => &rest[..t_.start()],
        None => rest,
    });
    if core.is_empty() {
        return String::new();
    }
    let words = |c: &str| -> Vec<String> { rx("[A-Za-z']+").findall_str(c).into_iter().map(|w| w.to_lowercase()).collect() };
    if correction.is_some() {
        let w = words(core);
        if w.is_empty() || w.len() > 5 || w.iter().any(|x| OWN_VERB.contains(x)) {
            return String::new();
        }
        return core.to_string();
    }
    if head_end.is_none() && tail.is_none() {
        return String::new();
    }
    let w = words(core);
    if w.is_empty() || w.len() > 5 || w.iter().any(|x| OWN_VERB.contains(x)) {
        return String::new();
    }
    core.to_string()
}

/// `_says_which_value_is_meant(line)`.
fn _says_which_value_is_meant(line: &str) -> bool {
    if CORRECTION_MARKER.match_(line).is_some() {
        return true;
    }
    let Some(rest) = CORRECTION_HEAD.match_(line) else { return false };
    A_VALUE_IN_FRONT_OF_NOT.is_match(&line[rest.end()..])
}

/// `_the_fault_to_lend(previous)`.
fn _the_fault_to_lend(previous: &str) -> String {
    A_FAULT_TO_LEND.search(previous).map(|m| m.str(1).to_string()).unwrap_or_default()
}

/// What `_a_stand_in(text)` finds: "one"/"ones" standing in for a noun.
struct Stand {
    word: String,
    at: usize,
    end: usize,
}

/// `_a_stand_in(text)`.
fn _a_stand_in(text_: &str) -> Option<Stand> {
    for m in A_STAND_IN.finditer(text_) {
        if let Some(lead) = m.get(2) {
            let low = lead.to_lowercase();
            if !STAND_IN_WORDS.contains(&low) || STAND_IN_SELECTORS.contains(&low) {
                continue;
            }
        }
        let (at, end) = m.span(3).unwrap();
        return Some(Stand { word: m.str(3).to_lowercase(), at, end });
    }
    None
}

/// `_plural_of(phrase)`.
fn _plural_of(phrase: &str) -> String {
    let (head, _, last) = text::rpartition(phrase, " ");
    let word = if last.is_empty() { phrase } else { last };
    let many = _a_plural_word(word);
    if head.is_empty() { many } else { format!("{head} {many}") }
}

/// `_a_plural_word(word)`.
fn _a_plural_word(word: &str) -> String {
    let lower = word.to_lowercase();
    if lower.ends_with("ss") {
        return format!("{word}es");
    }
    if lower.ends_with('s') {
        return word.to_string();
    }
    if ["ch", "sh", "x", "z"].iter().any(|e| lower.ends_with(e)) {
        return format!("{word}es");
    }
    let l: Vec<char> = lower.chars().collect();
    if lower.ends_with('y') && l.len() > 1 && !"aeiou".contains(l[l.len() - 2]) {
        return format!("{}ies", text::slice(word, 0, Some(-1)));
    }
    format!("{word}s")
}

/// `_singular_of(phrase)`.
fn _singular_of(phrase: &str) -> String {
    let (head, _, last) = text::rpartition(phrase, " ");
    let word = if last.is_empty() { phrase } else { last };
    let one = _a_singular_word(word);
    if head.is_empty() { one } else { format!("{head} {one}") }
}

/// `_a_singular_word(word)`.
fn _a_singular_word(word: &str) -> String {
    let lower = word.to_lowercase();
    if lower.ends_with("ies") && text::len(&lower) > 3 {
        return format!("{}y", text::slice(word, 0, Some(-3)));
    }
    if ["ses", "xes", "zes", "ches", "shes"].iter().any(|e| lower.ends_with(e)) {
        return text::slice(word, 0, Some(-2));
    }
    if lower.ends_with('s') && !lower.ends_with("ss") {
        return text::slice(word, 0, Some(-1));
    }
    word.to_string()
}

/// `_the_noun_to_lend(memory, plural=False)`: the thing an earlier line named.
fn _the_noun_to_lend(memory: Option<&Memory>, plural: bool) -> String {
    for earlier in _user_lines(memory, 3).iter().rev() {
        let line = normalise(earlier);
        let entities = topics::detect_entities(&line, None);
        let (name, kind) = references::_target(&line, &entities, None);
        let Some(name) = name else { continue };
        if kind.as_deref() != Some("thing") || text::len(&name) < 3 {
            continue;
        }
        if STAND_IN_NOT_THINGS.contains(&name.to_lowercase()) {
            continue;
        }
        if plural && !text::split_ref(&name.to_lowercase()).last().is_some_and(|w| w.ends_with('s')) {
            continue;
        }
        return name;
    }
    String::new()
}

/// `_with_the_noun_lent(text, memory)`: the line with the stand-in replaced by the noun.
fn _with_the_noun_lent(text_: &str, memory: Option<&Memory>) -> (String, String) {
    let Some(stand) = _a_stand_in(text_) else { return (String::new(), String::new()) };
    let noun = _the_noun_to_lend(memory, stand.word == "ones");
    if noun.is_empty() {
        return (String::new(), String::new());
    }
    let noun = if stand.word == "ones" { _plural_of(&noun) } else { _singular_of(&noun) };
    (format!("{}{}{}", &text_[..stand.at], noun, &text_[stand.end..]), noun)
}

/// `_a_describing_word_is_the_subject(read)`: the line's subject is only a describing word.
fn _a_describing_word_is_the_subject(read: &Understood) -> bool {
    if read.topic_from.iter().any(|(w, weight)| *weight >= 2 && A_DESCRIBING_SUBJECT.contains(&w.to_lowercase())) {
        return true;
    }
    let things: Vec<String> = read.entities.iter().filter(|e| e.kind == "thing").map(|e| e.text.to_lowercase()).collect();
    !things.is_empty() && things.iter().all(|t_| A_DESCRIBING_SUBJECT.contains(t_))
}

/// `_stand_in_here(text)`.
fn _stand_in_here(text_: &str) -> Option<StandIn> {
    _a_stand_in(text_).map(|s| StandIn { word: s.word, noun: None })
}

/// `_with_the_fault_lent(text, memory)`: "and the brakes?" after "the clutch is slipping".
fn _with_the_fault_lent(text_: &str, memory: Option<&Memory>) -> String {
    let core = _the_second_thing(text_);
    if core.is_empty() {
        return String::new();
    }
    let mut fault = String::new();
    for earlier in _user_lines(memory, 3).iter().rev() {
        let found = _the_fault_to_lend(earlier);
        if !found.is_empty() {
            fault = found;
            break;
        }
        if _the_second_thing(earlier).is_empty() {
            break;
        }
    }
    if fault.is_empty() || A_FAULT_TO_LEND.is_match(&core) {
        return String::new();
    }
    format!("{core} {fault}")
}

/// `_user_lines(memory, limit)`: the last `limit` lines the person typed, oldest first.
fn _user_lines(memory: Option<&Memory>, limit: usize) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(m) = memory {
        for turn in m.turns.iter().rev() {
            if !text::strip(&turn.user).is_empty() {
                lines.push(turn.user.clone());
                if lines.len() >= limit {
                    break;
                }
            }
        }
    }
    lines.reverse();
    lines
}

/// `_pointable(entities)`: the entities a pronoun can point at.
fn _pointable(entities: Vec<Entity>) -> Vec<Entity> {
    entities
        .into_iter()
        .filter(|e| {
            let word = e.text.to_lowercase();
            matches!(e.kind.as_str(), "thing" | "place" | "person" | "title") || (e.kind == "name" && !word.contains('\'') && !TOPICS_COMMON_WORDS.contains(&word))
        })
        .collect()
}

/// `_pointed_at(history)`: the latest earlier line that names something to point at.
fn _pointed_at(history: &[String]) -> Option<(Line, Vec<Entity>)> {
    for t_ in history.iter().rev() {
        let read = normalise(t_);
        let entities = _pointable(topics::detect_entities(&read, None));
        if !entities.is_empty() {
            return Some((read, entities));
        }
    }
    None
}

/// `_the_recorded_subject(memory)`: what the kept topic record names.
fn _the_recorded_subject(memory: Option<&Memory>) -> Option<(Line, Vec<Entity>)> {
    let record = topic_record(memory)?;
    if !matches!(record, Value::Dict(_)) {
        return None;
    }
    for key in ["of", "text"] {
        let Some(line) = record.get(key).and_then(Value::as_str) else { continue };
        if text::strip(line).is_empty() {
            continue;
        }
        let read = normalise(line);
        let entities = _pointable(topics::detect_entities(&read, None));
        if !entities.is_empty() {
            return Some((read, entities));
        }
    }
    None
}

/// `_recent_topic(lines)`: the strongest topic of the recent lines (the later on a tie).
fn _recent_topic(lines: &[String]) -> Option<String> {
    let mut best: Option<((i64, i64), String)> = None;
    for (index, t_) in lines.iter().enumerate() {
        let read = normalise(t_);
        let entities = topics::detect_entities(&read, None);
        let (topic, score) = topics::detect_topic(&read, None, Some(&entities), None);
        if let Some(topic) = topic.filter(|t_| !t_.is_empty()) {
            if score >= CARRY_TOPIC {
                let key = (score, -(index as i64));
                if best.as_ref().is_none_or(|b| key > b.0) {
                    best = Some((key, topic));
                }
            }
        }
    }
    best.map(|b| b.1)
}

/// `_changes_subject(line)`: "anyway", "different question".
fn _changes_subject(line: &Line) -> bool {
    let padded = format!(" {} ", line.norm_text);
    if SHIFTS.iter().any(|cue| padded.contains(cue.as_str())) {
        return true;
    }
    SHIFT_ACTUALLY.is_match(&line.norm_text)
}

/// `_names_a_bot_noun(words)`.
fn _names_a_bot_noun(words: &[&str]) -> bool {
    words.iter().any(|w| BOT_NOUNS.contains(*w))
}

/// `_a_stop_in_a_chat_about_something(norm, memory)`: "stop" in a chat about a person or thing
/// asks about stopping that, not the bot.
fn _a_stop_in_a_chat_about_something(norm: &[String], memory: Option<&Memory>) -> bool {
    let said = norm.join(" ");
    if PERSON_STOP.fullmatch(text::strip(&said)).is_none() {
        return false;
    }
    let Some(memory) = memory.filter(|m| !memory_is_empty(m)) else { return false };
    if let Some(last) = memory.turns.last() {
        if last.module.as_deref() == Some("keeping") {
            return false;
        }
        if TALK_ASK.is_match(&last.user) || PERSON_STOP.match_(text::strip(&last.user)).is_some() {
            return false;
        }
    }
    let words: HashSet<String> = match topic_record(Some(memory)) {
        Some(record @ Value::Dict(_)) => record.get("words").map(|w| w.items().iter().map(|x| x.str().to_string()).collect()).unwrap_or_default(),
        _ => HashSet::new(),
    };
    !words.is_empty() && !words.iter().any(|w| CHAT_WORDS.contains(w))
}

/// `_about_the_bot(line, read, entities=())`: the question is about the bot itself.
fn _about_the_bot(line: &Line, read: &Read, entities: &[Entity]) -> bool {
    if read.intent == "about_bot" {
        return true;
    }
    if read.kind != "question" || !entities.is_empty() {
        return false;
    }
    let words: Vec<&str> = line.words.iter().map(String::as_str).filter(|w| *w != "|").collect();
    if words.len() < 2 {
        return false;
    }
    if words[0] == "how" && words.len() > 3 && SECOND_PERSON.contains(words[2]) && DAY_VERBS.contains(words[1]) {
        return words[3..].iter().any(|w| DAY_WORDS.contains(*w));
    }
    if SECOND_PERSON.contains(words[1]) {
        if COPULA_AUX.contains(words[0]) {
            let rest: Vec<&&str> = words[2..].iter().filter(|w| !AFTER_BOT_FILLER.contains(**w)).collect();
            return rest.is_empty() || BOT_STATE.contains(*rest[0]) || _names_a_bot_noun(&words);
        }
        return MODAL_WORDS.contains(words[0]) && words.len() > 2 && words.len() <= 4 && BOT_STATE.contains(words[2]);
    }
    if !(ASKING_WORDS.contains(words[0]) && words.iter().take(4).any(|w| SECOND_PERSON.contains(*w))) {
        return false;
    }
    if words[0] == "who" {
        return SECOND_PERSON.contains(*words.last().unwrap()) || WHO_ENDS.contains(*words.last().unwrap());
    }
    let bot_at = words.iter().position(|w| SECOND_PERSON.contains(*w)).unwrap();
    if words[1..bot_at].iter().any(|w| HYPOTHETICAL.contains(*w)) {
        return false;
    }
    let rest: Vec<&&str> = words[bot_at + 1..].iter().filter(|w| !AFTER_BOT_FILLER.contains(**w)).collect();
    if !rest.is_empty() && MEANING_WORDS.contains(*rest[0]) && rest[1..].iter().all(|w| MEANING_TAIL.contains(**w)) {
        return true;
    }
    if words.len() > 5 {
        return false;
    }
    rest.is_empty() || BOT_STATE.contains(*rest[0]) || _names_a_bot_noun(&words)
}

/// `_the_speaker_is_the_subject(between)`.
fn _the_speaker_is_the_subject(between: &[String]) -> bool {
    let rest: Vec<&String> = between.iter().filter(|w| pyuni::is_alnum(w) && !NOT_A_SUBJECT.contains(*w)).collect();
    rest.len() == 1 && SPEAKER_SUBJECTS.contains(rest[0])
}

/// `_a_word_that_depends_on_the_owner(plain, entities, state_owner=None)`: "knackered" said of
/// a thing is broken, of a person worn out.
fn _a_word_that_depends_on_the_owner(plain: &str, entities: &[Entity]) -> String {
    if plain.is_empty() {
        return plain.to_string();
    }
    let edge = *EDGE_PUNCTUATION;
    let mut things: HashSet<String> = HashSet::new();
    let mut people: HashSet<String> = HashSet::new();
    for e in entities {
        let words = text::split(&e.text).into_iter().map(|w| w.to_lowercase());
        if e.kind == "thing" {
            things.extend(words);
        } else if e.kind == "person" {
            people.extend(words);
        }
    }
    let words = text::split(plain);
    let mut out: Vec<String> = Vec::new();
    for (at, word) in words.iter().enumerate() {
        let bare = text::strip_chars(word, edge).to_lowercase();
        if !BROKEN_OF_A_THING.contains(&bare) {
            out.push(word.clone());
            continue;
        }
        let mut between: Vec<String> = Vec::new();
        let mut whose: Option<&str> = None;
        for back in (0..at).rev() {
            let earlier = &words[back];
            if earlier.chars().any(|c| ",;:?!.".contains(c)) {
                break;
            }
            let low = text::strip_chars(earlier, edge).to_lowercase();
            if things.contains(&low) {
                whose = Some("thing");
                break;
            }
            if people.contains(&low) {
                whose = Some("person");
                break;
            }
            between.push(low);
        }
        if between.iter().any(|w| OTHER_PRONOUNS.contains(w)) {
            whose = None;
        } else if whose.is_none() && _the_speaker_is_the_subject(&between) {
            whose = Some("person");
        }
        let after: Vec<String> = words[at + 1..(at + 3).min(words.len())].iter().map(|w| text::strip_chars(w, edge).to_lowercase()).collect();
        let core = text::strip_chars(word, edge);
        if whose.is_none() || after.iter().any(|w| A_CONDITION_AFTER.contains(w)) {
            out.push(word.clone());
        } else if between.iter().any(|w| A_CLAUSE_VERB.contains(w)) {
            out.push(word.clone());
        } else if whose == Some("thing") {
            out.push(word.replace(core, "broken"));
        } else {
            let worn = WORN_OUT_OF_A_PERSON.get(&bare).unwrap_or_else(|| panic!("KeyError: {bare}"));
            out.push(word.replace(core, worn));
        }
    }
    out.join(" ")
}

/// `_opens_the_conversation(previous)`.
fn _opens_the_conversation(previous: Option<&Line>) -> bool {
    match previous {
        None => true,
        Some(p) => p.norm.iter().filter(|w| !w.is_empty() && *w != "|").count() <= 3,
    }
}

/// `asks_with_nothing_named(norm)`: "what do you think?" with nothing to think about.
pub fn asks_with_nothing_named(norm: &[String]) -> bool {
    let words: Vec<&String> = norm.iter().filter(|w| !w.is_empty() && *w != "|").collect();
    if !words.iter().any(|w| BOT_ASK_WORDS.contains(*w)) || !words.iter().any(|w| NOTHING_NAMED_VIEW_WORDS.contains(*w)) {
        return false;
    }
    let rest = words.iter().any(|w| !NOTHING_NAMED_ASK_WORDS.contains(*w) && !BOT_ASK_WORDS.contains(*w) && !NOTHING_NAMED_VIEW_WORDS.contains(*w) && !NOTHING_NAMED_FILLER.contains(*w));
    if rest {
        return false;
    }
    !(matches!(words[0].as_str(), "can" | "could") && words.len() > 1 && BOT_ASK_WORDS.contains(words[1]))
}

/// `_presence(line)`: "you there?", "anyone around".
fn _presence(line: &Line) -> bool {
    let mut words: Vec<&String> = line.norm.iter().filter(|w| !w.is_empty() && *w != "|").collect();
    if words.first().is_some_and(|w| PRESENCE_GREETING.contains(*w)) {
        words.remove(0);
    }
    if words.is_empty() || words.len() as i64 > PRESENCE_MAX {
        return false;
    }
    if !words.iter().any(|w| PRESENCE_WORD.contains(*w)) {
        return false;
    }
    let asked = words.iter().any(|w| PRESENCE_SUBJECT.contains(*w) || PRESENCE_ASKER.contains(*w));
    let dropped_subject = words.len() <= 2;
    if !asked && !dropped_subject {
        return false;
    }
    words.iter().all(|w| PRESENCE_WORD.contains(*w) || PRESENCE_SUBJECT.contains(*w) || PRESENCE_ASKER.contains(*w) || PRESENCE_FILLER.contains(*w))
}

/// `_a_manner(norm, at)`: "talk quickly".
fn _a_manner(norm: &[String], at: usize) -> bool {
    let word = &norm[at];
    if text::len(word) <= 4 || !word.ends_with("ly") || NOT_MANNER.contains(word) {
        return false;
    }
    if MANNER_TALK.contains(word) {
        return true;
    }
    let mut back = at as isize - 1;
    if back >= 0 && MORE_WORDS.contains(&norm[back as usize]) {
        back -= 1;
    }
    back >= 0 && TALK_VERBS.contains(&norm[back as usize])
}

/// `_not_manner(norm)`: the words that are not how to talk.
fn _not_manner(norm: &[String]) -> HashSet<String> {
    let mut left = HashSet::new();
    for (at, word) in norm.iter().enumerate() {
        if _a_manner(norm, at) {
            continue;
        }
        if MORE_WORDS.contains(word) && at + 1 < norm.len() && _a_manner(norm, at + 1) {
            continue;
        }
        left.insert(word.clone());
    }
    left
}

/// `a_load_carried(text)`: "i do everything around the house and nobody helps".
pub fn a_load_carried(text_: &str) -> bool {
    let said = text::squash(&text_.to_lowercase().replace('\u{2019}', "'"));
    if said.is_empty() {
        return false;
    }
    let said = text::strip_chars(&said, "'\" ");
    for piece in rx(r"[.!?,;:]+|\bbut\b|\bthough\b").split(said) {
        let piece = text::strip(&piece);
        if piece.is_empty() || ASKS_WHY_IT_IS.match_(piece).is_some() || DOES_ALL_THEY_CAN.is_match(piece) {
            continue;
        }
        if WORK_SCOPE.is_match(piece) && !HOME_SCOPE.is_match(piece) {
            continue;
        }
        if DOES_IT_RIGHT.is_match(piece) {
            continue;
        }
        if RUNS_MYSELF_RAGGED.is_match(piece) || RUNS_THE_HOUSE.is_match(piece) || HELPS_ME_NOT.is_match(piece) {
            return true;
        }
        if LEANS_ON_THEM.is_match(piece) || DOES_IT_ALL.is_match(piece) || NO_TIME_OFF.is_match(piece) || ALL_ON_THEM.is_match(piece) {
            return true;
        }
        if let Some(unhelped) = NO_ONE_HELPS.search(piece) {
            let head = &piece[..unhelped.start()];
            if FIRST_PERSON_ANYWHERE.is_match(head) && !THIRD_PERSON.is_match(head) {
                return true;
            }
        }
        if THE_ONE_WHO.is_match(piece) && EVERYTHING_AT_ONCE.is_match(piece) {
            return true;
        }
    }
    false
}

/// `_asks_for_a_moment(line)`: "can i ask you something", "got a minute?".
fn _asks_for_a_moment(line: &Line) -> bool {
    let mut norm: Vec<String> = line.norm.iter().filter(|w| !w.is_empty() && *w != "|").cloned().collect();
    if norm.is_empty() || norm.len() > 9 {
        return false;
    }
    let bare = norm.join(" ");
    for phrase in ASK_PERMISSION.iter() {
        if bare.contains(phrase.as_str()) {
            let mut rest = text::split(&bare.replace(phrase.as_str(), " "));
            let last = text::split_ref(phrase).last().unwrap().to_string();
            if TALK_VERBS.contains(&last) {
                rest.insert(0, last);
            }
            if _not_manner(&rest).iter().all(|w| MOMENT_FILLER.contains(w) || MOMENT_WORDS.contains(w) || MOMENT_VERBS.contains(w) || PERMISSION_FILLER.contains(w)) {
                return true;
            }
            continue;
        }
    }
    while norm.first().is_some_and(|w| PRESENCE_GREETING.contains(w)) {
        norm.remove(0);
    }
    if norm.len() > 2 && GREETING_PHRASE.contains(&norm[..2].join(" ")) {
        norm.drain(..2);
    }
    let words = sv(&norm);
    if words.is_empty() {
        return false;
    }
    let any = |set: &HashSet<String>| words.iter().any(|w| set.contains(*w));
    if any(&ASK_FREE_WORDS) && (any(&PRESENCE_SUBJECT) || norm.len() <= 2) {
        return _not_manner(&norm).iter().all(|w| ASK_FREE_FILLER.contains(w) || ASK_FREE_WORDS.contains(w));
    }
    if norm.join(" ").contains("the time") && !(words.iter().any(|w| *w != "time" && MOMENT_WORDS.contains(*w)) || any(&TALK_VERBS)) {
        return false;
    }
    if any(&MOMENT_WORDS) && any(&MOMENT_VERBS) {
        return _not_manner(&norm).iter().all(|w| MOMENT_FILLER.contains(w) || MOMENT_WORDS.contains(w) || MOMENT_VERBS.contains(w) || matches!(w.as_str(), "you" | "u" | "ya"));
    }
    if (words.contains("time") || words.contains("moment")) && words.iter().all(|w| TALK_VERBS.contains(*w) || GOOD_TIME_WORDS.contains(*w)) {
        return true;
    }
    if norm.len() <= 6 && any(&TALK_VERBS) {
        return _not_manner(&norm).iter().all(|w| TALK_FILLER.contains(w) || TALK_VERBS.contains(w));
    }
    false
}

/// `_keywords(norm)`: the content words of the line.
fn _keywords(norm: &[String]) -> Vec<String> {
    norm.iter().filter(|w| !STOPWORDS.contains(*w) && (text::len(w) > 1 || pyuni::is_digit(w))).take(MAX_KEYWORDS as usize).cloned().collect()
}

/// `_asks_a_property_and_nothing_else(text)`: "how big is it" asks only a property.
fn _asks_a_property_and_nothing_else(text_: &str) -> bool {
    let low = text::strip(text_).to_lowercase();
    if PROPERTY_ASK.match_(&low).is_none() {
        return false;
    }
    let found = _keywords(&text::split(&low));
    !found.is_empty() && found.iter().all(|w| PROPERTY_SUBJECTS.contains(w))
}

/// `_the_chat_subject(chat)`: the subject the kept topic record names `(subject, type, words)`.
fn _the_chat_subject(chat: Option<&Value>) -> (Option<String>, Option<String>, Vec<String>) {
    let record = match chat {
        Some(v) if v.truthy() => match v {
            Value::Dict(_) => Some(v),
            _ => panic!("AttributeError: '{v:?}' object has no attribute 'get'"),
        },
        _ => None,
    };
    let get = |k: &str| record.and_then(|r| r.get(k));
    let said = get("text").filter(|v| v.truthy()).map(value_str).unwrap_or_default();
    let said = text::strip(&said).to_string();
    if said.is_empty() {
        return (None, None, vec![]);
    }
    let named: Vec<Entity> = topics::detect_entities(&normalise(&said), None).into_iter().filter(|e| SUBJECT_TYPES.contains(&e.kind)).collect();
    if named.is_empty() {
        return (None, None, vec![]);
    }
    let plain: Vec<&Entity> = named.iter().filter(|e| e.kind != "name" && e.kind != "title").collect();
    let chosen = if plain.is_empty() { named.last().unwrap() } else { *plain.last().unwrap() };
    let words: Vec<String> = get("words").filter(|v| v.truthy()).map(|w| w.items().iter().map(|x| value_str(x).to_lowercase()).collect()).unwrap_or_default();
    (Some(chosen.text.to_lowercase()), Some(chosen.kind.clone()), words)
}

/// Python's `str(value)` for the values a topic record holds.
fn value_str(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        Value::Bool(b) => if *b { "True".into() } else { "False".into() },
        Value::None => "None".into(),
        other => format!("{other:?}"),
    }
}

/// `_the_meaning_is_applied_to_what_was_said(line, leans_on, memory)`: "what do you mean for my
/// son" only counts as asking what was meant when the earlier talk named him.
fn _the_meaning_is_applied_to_what_was_said(line: &Line, leans_on: Option<&LeansOn>, memory: Option<&Memory>) -> bool {
    let said_text = if line.text.is_empty() { &line.clean } else { &line.text };
    let Some(applied) = KINDS_MEANT_APPLIED_TO.search(said_text) else { return true };
    let thing = applied.str(1).to_lowercase();
    let mut said: Vec<String> = Vec::new();
    if let Some(l) = leans_on {
        said.push(l.text.clone());
        said.push(l.subject.clone().unwrap_or_default());
        said.extend(l.words.iter().cloned());
    }
    if let Some(last) = memory.and_then(|m| m.turns.last()) {
        said.push(last.bot.clone());
    }
    let mut words: HashSet<String> = HashSet::new();
    for blob in said {
        words.extend(rx("[a-z]+").findall_str(&blob.to_lowercase()));
    }
    words.iter().filter(|w| text::len(w) > 2).any(|w| w.starts_with(&thing) || thing.starts_with(w.as_str()))
}

/// `_leans_on(...)`: the earlier line this one carries on from, or None.
#[allow(clippy::too_many_arguments)]
fn _leans_on(
    line: &Line,
    read: &Read,
    reference: &crate::understood::Ref,
    carried_text: Option<&str>,
    carried: Option<&Line>,
    carried_is_previous: bool,
    previous: Option<&Line>,
    own_topic: Option<&str>,
    previous_topic: Option<&str>,
    previous_entities: &[Entity],
    carried_is_older: bool,
    chat: Option<&Value>,
) -> Option<LeansOn> {
    let carried_text = carried_text.filter(|c| !c.is_empty())?;
    if CARRY_PASS.contains(&read.intent) {
        return None;
    }
    if !reference.followup && own_topic.is_some_and(|t_| !t_.is_empty()) {
        return None;
    }
    if _changes_subject(line) {
        return None;
    }
    let carried = carried.unwrap();
    let mut words = breeds::the_animal_a_breed_is(&_keywords(&carried.norm));
    if !carried_is_previous && !carried_is_older {
        for word in _keywords(&previous.unwrap().norm) {
            if !words.contains(&word) {
                words.push(word);
            }
        }
    }
    let (chat_subject, chat_type, chat_words) = _the_chat_subject(chat);
    for word in &chat_words {
        if !words.contains(word) {
            words.push(word.clone());
        }
    }
    words.truncate(CARRY_WORDS as usize);
    if words.is_empty() {
        return None;
    }
    let asked_only = _asks_a_property_and_nothing_else(carried_text);
    if asked_only {
        words.retain(|w| !PROPERTY_SUBJECTS.contains(w));
    }
    let named: Vec<&Entity> = previous_entities.iter().filter(|e| SUBJECT_TYPES.contains(&e.kind) && !(asked_only && PROPERTY_SUBJECTS.contains(&e.text.to_lowercase()))).collect();
    let plain: Vec<&&Entity> = named.iter().filter(|e| e.kind != "name" && e.kind != "title").collect();
    let chosen: Option<&Entity> = if plain.is_empty() { named.last().copied() } else { plain.last().map(|e| **e) };
    let mut subject = chosen.map(|e| e.text.to_lowercase());
    let mut subject_type = chosen.map(|e| e.kind.clone());
    if let Some(cs) = chat_subject.filter(|c| !c.is_empty()) {
        if subject.as_deref() != Some(cs.as_str()) && !subject.as_ref().is_some_and(|s| chat_words.contains(s)) {
            subject = Some(cs);
            subject_type = chat_type;
        }
    }
    Some(LeansOn { text: carried_text.to_string(), words, subject, subject_type, topic: previous_topic.map(str::to_string) })
}

/// `_about_a_person_or_a_trouble(after)`.
fn _about_a_person_or_a_trouble(after: &[String]) -> bool {
    if after.iter().any(|w| KINDS_NEEDS_ANSWER.contains(w) || AT_IT_STILL.contains(w)) {
        return true;
    }
    if KINDS_STILL_WRONG.is_match(&after.join(" ")) {
        return true;
    }
    if PERSON_HEADS.contains(&after[0]) {
        return true;
    }
    POSSESSIVE.contains(&after[0]) && after.len() > 1 && (KINDS_OTHER_PEOPLE.contains(&after[1]) || KINDS_PERSON_NOUNS.contains(&after[1]) || KINDS_MOOD_NOUNS.contains(&after[1]))
}

/// `_about_something_of_its_own(after)`: the words after a thank-you are a new subject.
fn _about_something_of_its_own(after: &[String]) -> bool {
    if after.len() < 2 || KINDS_ADDRESS_WORDS.contains(&after[0]) {
        return false;
    }
    let joined = after.join(" ");
    if after[0] == "i" && DID_IT.contains(&after[1]) && !after.iter().any(|w| KINDS_NEEDS_ANSWER.contains(w)) && !KINDS_STILL_WRONG.is_match(&joined) {
        return false;
    }
    if after.iter().any(|w| PRAISED.contains(w)) {
        return false;
    }
    if kinds::_a_step_done(after) && !kinds::_the_step_went_wrong(after) {
        return false;
    }
    if after.iter().any(|w| KINDS_PRAISING_VERB.contains(w)) || KINDS_PRAISE_SAID.is_match(&joined) {
        return false;
    }
    if kinds::_a_fall_that_is_not_one(after) {
        return false;
    }
    if after.iter().any(|w| w == "temperature") || KINDS_SIGNING_OFF.is_match(&joined) {
        return false;
    }
    if !_about_a_person_or_a_trouble(after) {
        return false;
    }
    let rest = normalise(&joined);
    let entities = topics::detect_entities(&rest, None);
    let (topic, score) = topics::detect_topic(&rest, None, Some(&entities), Some(false));
    topic.is_some() && score >= CARRY_TOPIC + 1
}

/// `_the_new_thing(words)`: the thing a follow-up swaps in ("and for brown rice" -> brown rice).
fn _the_new_thing(words: &[String]) -> Option<Vec<String>> {
    let mut phrase: &[String] = words;
    let mut dropped = true;
    while dropped && !phrase.is_empty() {
        dropped = false;
        for opener in FOLLOW_OPENER.iter() {
            if phrase.len() >= opener.len() && phrase[..opener.len()] == opener[..] {
                phrase = &phrase[opener.len()..];
                dropped = true;
                break;
            }
        }
    }
    while phrase.first().is_some_and(|w| CORRECTION_OPENER.contains(w)) {
        phrase = &phrase[1..];
    }
    if phrase.is_empty() || phrase.len() > 5 {
        return None;
    }
    if phrase.iter().any(|w| NOT_A_THING.contains(w)) {
        return None;
    }
    if !phrase.iter().any(|w| text::len(w) > 2 || LX_COMMON_NAMES.contains(w)) {
        return None;
    }
    Some(phrase.to_vec())
}

/// `_the_same_kind(a, b)`: the kind two words share (a person, a number, ...).
fn _the_same_kind(a: &str, b: &str) -> Option<String> {
    for (kind, words) in SAME_KIND_WORDS.iter() {
        if words.contains(a) && words.contains(b) {
            return Some(kind.clone());
        }
    }
    if a.chars().any(pyuni::ch_digit) && b.chars().any(pyuni::ch_digit) {
        return Some("number".to_string());
    }
    None
}

/// `_where_the_swap_goes(before, phrase)`: `(start, end, kind)` in the earlier line.
fn _where_the_swap_goes(before: &[String], phrase: &[String]) -> Option<(usize, usize, String)> {
    if before.is_empty() || phrase.is_empty() {
        return None;
    }
    for word in phrase {
        for (at, earlier) in before.iter().enumerate() {
            let Some(kind) = _the_same_kind(word, earlier) else { continue };
            let mut end = at + 1;
            if earlier.chars().any(pyuni::ch_digit) {
                while end < before.len() && (COUNT_UNITS.contains(&before[end]) || matches!(before[end].as_str(), "a" | "an" | "each" | "per" | "every")) {
                    end += 1;
                }
            }
            return Some((at, end, kind));
        }
    }
    None
}

/// `_the_swap_is_said(before, phrase, place)`: the earlier line with the new thing in place.
fn _the_swap_is_said(before: &[String], phrase: &[String], place: &(usize, usize, String)) -> Option<String> {
    if before.is_empty() || phrase.is_empty() {
        return None;
    }
    let (start, end, _) = place;
    let mut said: &[String] = phrase;
    if *start > 0 && DETERMINER_TAIL.contains(&before[start - 1]) && DETERMINER_TAIL.contains(&said[0]) {
        said = &said[1..];
    }
    if said.is_empty() {
        return None;
    }
    let mut out: Vec<String> = before[..*start].to_vec();
    out.extend(said.iter().cloned());
    out.extend(before[*end..].iter().cloned());
    Some(out.join(" "))
}

/// `_the_line_before(read, memory)`.
fn _the_line_before(read: &Understood, memory: Option<&Memory>) -> Option<String> {
    if let Some(m) = memory {
        for turn in m.turns.iter().rev() {
            if !text::strip(&turn.user).is_empty() {
                return Some(turn.user.clone());
            }
        }
    }
    read.leans_on.as_ref().map(|l| l.text.clone())
}

/// `_asked_again(read, memory=None)`: `(the earlier line with the new thing swapped in, kind)`.
fn _asked_again(read: &Understood, memory: Option<&Memory>) -> Option<(String, String)> {
    read.leans_on.as_ref()?;
    let before = _the_line_before(read, memory).filter(|b| !text::strip(b).is_empty())?;
    let phrase = _the_new_thing(&read.norm)?;
    let earlier = normalise(&before);
    let at = _where_the_swap_goes(&earlier.norm, &phrase)?;
    let said = _the_swap_is_said(&earlier.norm, &phrase, &at)?;
    if said.is_empty() || said == earlier.norm.join(" ") {
        return None;
    }
    Some((said, at.2))
}

/// `_the_spans_of(clauses)`.
fn _the_spans_of(clauses: &[Vec<String>]) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut at = 0;
    for clause in clauses {
        spans.push((at, at + clause.len()));
        at += clause.len();
    }
    spans
}

/// `_the_thing_the_ask_names(clauses, focus, entities)`: `(ask_subject, setting, subject_from)`.
fn _the_thing_the_ask_names(clauses: &[Vec<String>], focus: &[String], entities: &[Entity]) -> (Option<String>, Option<String>, Option<String>) {
    let named: Vec<&Entity> = entities.iter().filter(|e| SUBJECT_TYPES.contains(&e.kind)).collect();
    if named.is_empty() {
        return (None, None, None);
    }
    let mut ask_span: Option<(usize, usize)> = None;
    for (clause, span) in clauses.iter().zip(_the_spans_of(clauses)) {
        if clause[..] == focus[..] {
            ask_span = Some(span);
        }
    }
    let Some((start, end)) = ask_span else { return (Some(named[0].text.to_lowercase()), None, Some("ask".to_string())) };
    let within = |e: &&Entity| (start as i64) <= e.at && e.at < end as i64;
    let inside = named.iter().find(|e| within(e));
    let outside = named.iter().find(|e| !within(e));
    (inside.map(|e| e.text.to_lowercase()), outside.map(|e| e.text.to_lowercase()), Some(if inside.is_some() { "ask" } else { "setting" }.to_string()))
}

/// `understand(text, memory=None)`: the reading of one line, read against the conversation so
/// far. With `RSBOT_ORACLE_UNDERSTAND` set, PyBot's own recorded reading is returned for a text
/// it has seen (for checking modules before this engine is complete).
pub fn understand(text_: &str, memory: Option<&Memory>) -> Understood {
    if crate::oracle::use_oracle_understand() {
        if let Some(u) = crate::oracle::UNDERSTAND_CALLS.get(text_) {
            return u.clone();
        }
    }
    let mut read = _read_line(text_, memory);
    let stated = _an_introduction_that_says_how_they_are(text_);
    if !stated.is_empty() {
        let mut alt = _read_line(&stated, memory);
        alt.text = read.text;
        return alt;
    }
    let (errand, at_the_head) = _the_ask_inside_an_errand(text_);
    if !errand.is_empty() {
        let mut alt = _read_line(&errand, memory);
        let alt_clean = if alt.clean.is_empty() { errand.as_str() } else { alt.clean.as_str() };
        if _reaches(&alt) && (!_reaches(&read) || (at_the_head && _A_THING_THAT_CHANGES.is_match(alt_clean))) {
            alt.text = read.text;
            alt.fixes = read.fixes;
            alt.errand = Some(errand);
            return alt;
        }
        if at_the_head && !_reaches(&read) {
            read.errand = Some(errand);
            return read;
        }
    }
    let lent = _with_the_fault_lent(text_, memory);
    if !lent.is_empty() {
        let mut alt = _read_line(&lent, memory);
        if _reaches(&alt) {
            alt.text = read.text;
            return alt;
        }
    }
    if let Some(stand) = _a_stand_in(text_) {
        if _a_describing_word_is_the_subject(&read) {
            let (lent_noun, noun) = _with_the_noun_lent(text_, memory);
            if !lent_noun.is_empty() {
                let mut alt = _read_line(&lent_noun, memory);
                if _reaches(&alt) {
                    alt.text = read.text;
                    alt.stand_in = Some(StandIn { word: stand.word, noun: Some(noun) });
                    return alt;
                }
            }
        }
    }
    let asked = _asked_again(&read, memory).and_then(|(said, kind)| if read.kind == "question" || kind != "number" { Some(said) } else { None });
    if let Some(asked) = asked.filter(|a| !a.is_empty()) {
        let mut alt = _read_line(&asked, memory);
        if !alt.clean.is_empty() {
            alt.text = read.text;
            alt.fixes = read.fixes;
            alt.reference = read.reference;
            alt.leans_on = read.leans_on;
            alt.asked_again = Some(asked);
            return alt;
        }
    }
    if _reaches(&read) {
        return read;
    }
    let (swapped, pairs) = _confused_text(text_);
    if swapped.is_empty() {
        return read;
    }
    let mut alt = _read_line(&swapped, memory);
    if !_reaches(&alt) {
        return read;
    }
    alt.text = read.text;
    let mut items: Vec<FixItem> = match std::mem::take(&mut alt.fixes) {
        Fixes::Map(m) => m.into_keys().map(FixItem::Key).collect(),
        Fixes::List(v) => v,
    };
    items.extend(pairs.into_iter().map(|(a, b)| FixItem::Pair(a, b)));
    alt.fixes = Fixes::List(items);
    alt
}

/// `line_as_read(understood)`: the line as the engine read it.
pub fn line_as_read(understood: &Understood) -> String {
    if !text::strip(&understood.plain).is_empty() {
        return understood.plain.clone();
    }
    if !understood.clean.is_empty() { understood.clean.clone() } else { understood.text.clone() }
}

/// `_the_ask_inside_an_errand(text)`: "can you look up how long ..." -> the ask, and whether it
/// opened the line.
fn _the_ask_inside_an_errand(text_: &str) -> (String, bool) {
    let Some(m) = _SEARCH_FRAME.match_(text_) else { return (String::new(), false) };
    let rest = _NOT_ABOUT_THE_THING.sub("", &text_[m.end()..]);
    let rest = text::strip_chars(&rest, " ,.?!").to_string();
    if text::split_ref(&rest).len() < 2 {
        return (String::new(), false);
    }
    if _A_CLAUSE_WITH_NO_SUBJECT.match_(&rest).is_some() {
        return (String::new(), false);
    }
    let polite = m.name("polite").filter(|p| !p.is_empty()).or(m.name("polite2")).unwrap_or("");
    let at_the_head = text::strip(polite).is_empty();
    (rest, at_the_head)
}

/// `_reaches(reading)`: the reading found something to answer from (a topic, a feeling, ...).
fn _reaches(reading: &Understood) -> bool {
    REACHING_KEYS.iter().any(|key| match key.as_str() {
        "topic" => reading.topic.as_ref().is_some_and(|t_| !t_.is_empty()),
        "emotion" => reading.emotion.as_ref().is_some_and(|t_| !t_.is_empty()),
        "body" => reading.body.as_ref().is_some_and(|t_| !t_.is_empty()),
        "numbers" => !reading.numbers.is_empty(),
        "summons" => reading.summons.is_some(),
        "presence" => reading.presence,
        other => serde_json::to_value(reading).ok().and_then(|v| v.get(other).cloned()).is_some_and(|v| match v {
            serde_json::Value::Null => false,
            serde_json::Value::Bool(b) => b,
            serde_json::Value::Number(n) => n.as_f64() != Some(0.0),
            serde_json::Value::String(s) => !s.is_empty(),
            serde_json::Value::Array(a) => !a.is_empty(),
            serde_json::Value::Object(o) => !o.is_empty(),
        }),
    })
}

/// `_an_introduction_that_says_how_they_are(text)`: "hi, i'm shattered" -> "im shattered".
fn _an_introduction_that_says_how_they_are(text_: &str) -> String {
    let Some(m) = INTRODUCING_A_STATE.match_(text::strip(text_)) else { return String::new() };
    let word = m.str(1).to_lowercase();
    if LX_A_DESCRIBING.contains(&word) || LX_TRAITS.contains(&word) || LX_A_STATE.contains(&word) {
        return format!("im {word}");
    }
    String::new()
}

/// `_said_of_themselves(words, place)`: "i was board", "im so board".
fn _said_of_themselves(words: &[(String, usize, usize)], place: usize) -> bool {
    let mut back = place as isize - 1;
    while back >= 0 && LX_FEELING_FILLER.contains(&words[back as usize].0) {
        back -= 1;
    }
    if back < 0 {
        return false;
    }
    let lead = words[back as usize].0.as_str();
    if LX_FEELING_LEAD.contains(lead) {
        return true;
    }
    matches!(lead, "was" | "wasnt" | "wasn't" | "is" | "isnt" | "isn't" | "get" | "got") && back > 0 && matches!(words[back as usize - 1].0.as_str(), "i" | "im" | "i'm" | "iam")
}

/// `_confused_text(text)`: words confused for another ("loose", "quite") swapped for the one
/// meant, with the pairs swapped.
fn _confused_text(text_: &str) -> (String, Vec<(String, String)>) {
    if text::strip(text_).is_empty() {
        return (String::new(), vec![]);
    }
    let words: Vec<(String, usize, usize)> = rx("[A-Za-z']+").finditer(text_).iter().map(|m| (m.as_str().to_lowercase(), m.start(), m.end())).collect();
    if words.is_empty() {
        return (String::new(), vec![]);
    }
    let mut pieces = String::new();
    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut cut = 0;
    for (place, (word, start, end)) in words.iter().enumerate() {
        let before = if place > 0 { words[place - 1].0.as_str() } else { "" };
        let after = words.get(place + 1).map(|w| w.0.as_str()).unwrap_or("");
        let mut meant: Option<String> = LX_CONFUSIONS.get(word).cloned().filter(|m| !m.is_empty());
        if meant.is_none() {
            match LX_CONFUSIONS_AFTER.get(word) {
                Some((m, befores)) if befores.contains(before) => meant = Some(m.clone()),
                _ => match LX_CONFUSIONS_BEFORE.get(word) {
                    Some((m, afters)) if afters.contains(after) => meant = Some(m.clone()),
                    _ => {
                        if after.is_empty() {
                            meant = None;
                        }
                    }
                },
            }
        }
        if meant.is_none() {
            if let Some((m, afters)) = LX_CONFUSIONS_IN_A_FEELING.get(word) {
                if (after.is_empty() || afters.contains(after)) && _said_of_themselves(&words, place) {
                    meant = Some(m.clone());
                }
            }
        }
        if meant.is_none() && LX_QUESTION_WORDS.contains(after) {
            meant = LX_CONFUSIONS_BEFORE_A_QUESTION.get(word).cloned();
        }
        let Some(meant) = meant.filter(|m| !m.is_empty() && m != word) else { continue };
        pieces.push_str(&text_[cut..*start]);
        pieces.push_str(&meant);
        cut = *end;
        pairs.push((word.clone(), meant));
    }
    if pairs.is_empty() {
        return (String::new(), vec![]);
    }
    pieces.push_str(&text_[cut..]);
    (pieces, pairs)
}

/// `_a_constraint_on_the_thing(line)`.
fn _a_constraint_on_the_thing(line: Option<&Line>) -> bool {
    line.is_some_and(|l| CONSTRAINT_FACT.match_(&l.norm.join(" ")).is_some())
}

/// `_read_line(text, memory=None)`: one reading of the text.
fn _read_line(text_: &str, memory: Option<&Memory>) -> Understood {
    use crate::understanding::normalize::{_the_part_that_hurts, _the_sugar_that_is_blood, without_the_figure_of_speech};
    let mut line = normalise(&_the_sugar_that_is_blood(&_the_part_that_hurts(&without_the_figure_of_speech(text_))));
    let mut read = kinds::classify(&line);
    if read.kind == "thanks" {
        let after = kinds::the_line_after_a_courtesy(&line.norm);
        if !after.is_empty() && !THANKED_FOR.contains(&after[0]) && _about_something_of_its_own(&after) {
            line = normalise(&after.join(" "));
            read = kinds::classify(&line);
        }
    }
    let history = _user_lines(memory, LOOKBACK as usize);
    let previous_text: Option<String> = history.last().cloned();
    let previous: Option<Line> = previous_text.as_deref().map(normalise);
    let mut previous_entities: Vec<Entity> = previous.as_ref().map(|p| topics::detect_entities(p, None)).unwrap_or_default();
    let mut previous_topic: Option<String> = None;
    if let Some(p) = &previous {
        previous_topic = topics::detect_topic(p, None, Some(&previous_entities), None).0;
    }
    let mut carried_text = previous_text.clone();
    let mut carried: Option<Line> = None; // None: the carried line is `previous`
    let mut carried_is_older = false;
    let constraint_before = _a_constraint_on_the_thing(previous.as_ref());
    if let Some(p) = &previous {
        if previous_entities.is_empty() || constraint_before {
            let own_subject = if constraint_before { None } else { topics::subject_topic(p, Some(&previous_entities)) };
            let mut earlier = _pointed_at(&history[..history.len() - 1]);
            if earlier.is_none() && line.norm.iter().any(|w| ANY_PRONOUN.contains(w)) {
                earlier = _the_recorded_subject(memory);
                carried_is_older = earlier.is_some();
            }
            if let Some((e_line, e_ents)) = &earlier {
                if own_subject.is_some() && !constraint_before {
                    let earlier_subject = topics::subject_topic(e_line, Some(e_ents));
                    if earlier_subject.is_some() && earlier_subject != own_subject {
                        earlier = None;
                        carried_is_older = false;
                    }
                }
            }
            if let Some((e_line, e_ents)) = earlier {
                previous_entities = e_ents;
                carried_text = Some(e_line.text.clone());
                if previous_topic.is_none() {
                    previous_topic = topics::detect_topic(&e_line, None, Some(&previous_entities), None).0;
                }
                carried = Some(e_line);
            }
        }
    }
    let carried_is_previous = carried.is_none();
    let carried_line: Option<&Line> = if carried_is_previous { previous.as_ref() } else { carried.as_ref() };
    let reference = references::resolve(&line, &read, previous.as_ref(), &previous_entities, previous_topic.as_deref());
    let about = kinds::after_any_link(&line.norm);
    let mut topic_line_owned: Option<Line> = None;
    let entities: Vec<Entity>;
    if !about.is_empty() {
        let about_line = normalise(&about.join(" "));
        let after_entities = topics::detect_entities(&about_line, None);
        if topics::subject_topic(&normalise(&about.join(" ")), Some(&after_entities)).is_some() {
            entities = after_entities;
            topic_line_owned = Some(about_line);
        } else {
            entities = topics::detect_entities(&line, None);
        }
    } else {
        entities = topics::detect_entities(&line, None);
    }
    let topic_line: &Line = topic_line_owned.as_ref().unwrap_or(&line);
    let continues = reference.followup || topics::subject_topic(topic_line, Some(&entities)).is_none();
    let mut inherited: Option<String> = None;
    if continues && !_changes_subject(&line) {
        match topic_record(memory) {
            Some(Value::Str(claimed)) if !claimed.is_empty() => inherited = Some(claimed.to_string()),
            _ => inherited = _recent_topic(&history).or_else(|| previous_topic.clone()),
        }
    }
    if read.kind == "statement" && read.asks.is_empty() && entities.iter().any(|e| e.kind == "place") && kinds::noun_phrase_request(&line) {
        read.intent = "request".to_string();
        read.asks = vec!["recommend".to_string()];
        read.replies_to_bot = false;
    }
    let asks_something = read.kind == "question" || read.intent == "request" || read.intent == "advice";
    let (own_topic, own_score) = topics::detect_topic(topic_line, None, Some(&entities), Some(asks_something));
    let (mut topic, score) = topics::detect_topic(topic_line, inherited.as_deref(), Some(&entities), Some(asks_something));
    let mut ranked: Vec<String> = topics::rank_topics(topic_line, None, Some(&entities), Some(asks_something)).into_iter().map(|(n, _)| n).collect();
    if _a_stop_in_a_chat_about_something(&line.norm, memory) {
        if read.intent == "about_bot" {
            read.intent = "question".to_string();
        }
        read.about_bot = false;
        read.about_bot_kind = None;
    }
    let promote = |ranked: &mut Vec<String>, name: &str| {
        let mut v = vec![name.to_string()];
        v.extend(ranked.iter().filter(|n| *n != name).cloned());
        *ranked = v;
    };
    if topic.is_none() && !kinds::opening_line(&line.norm) && _about_the_bot(&line, &read, &entities) {
        topic = Some(ABOUT_THE_BOT.to_string());
        promote(&mut ranked, &ABOUT_THE_BOT);
    } else if topic.is_none() && (read.intent == "greeting" || read.intent == "smalltalk") {
        topic = Some("small-talk".to_string());
    } else if topic.is_none() && read.state_owner.as_deref() == Some("self") && read.emotion.is_some() && entities.is_empty() && read.intent != "question" && read.intent != "request" {
        topic = Some("feelings".to_string());
        promote(&mut ranked, "feelings");
    }
    if read.kind == "question" && topic.as_deref() == Some("small-talk") && ranked.iter().any(|n| n == *OWN_QUESTION_TOPIC) {
        topic = Some(OWN_QUESTION_TOPIC.to_string());
        promote(&mut ranked, &OWN_QUESTION_TOPIC);
    }
    let subject = entities.iter().find(|e| SUBJECT_TYPES.contains(&e.kind)).map(|e| e.text.to_lowercase());
    let (ask_subject, setting, subject_from) = _the_thing_the_ask_names(&line.clauses, &line.focus, &entities);
    let topic_from = topics::the_words_that_said(&line, topic.as_deref(), Some(&entities));
    let leans_on = _leans_on(
        &line,
        &read,
        &reference,
        carried_text.as_deref(),
        carried_line,
        carried_is_previous,
        previous.as_ref(),
        own_topic.as_deref(),
        previous_topic.as_deref(),
        &previous_entities,
        carried_is_older,
        topic_record(memory),
    );
    let mut intent = read.intent.clone();
    let wanting = matches!(intent.as_str(), "question" | "request" | "news" | "statement" | "ack");
    if ranked.is_empty() && wanting && kinds::opening_line(&line.norm) {
        intent = "opening".to_string();
    } else if entities.is_empty()
        && leans_on.is_none()
        && matches!(intent.as_str(), "question" | "request")
        && _opens_the_conversation(previous.as_ref())
        && !(previous_topic.is_some() || !previous_entities.is_empty())
        && asks_with_nothing_named(&line.norm)
    {
        intent = "opening".to_string();
    } else if wanting && kinds::talk_frame(&line.norm) {
        intent = "opening".to_string();
    }
    let mut presence = _presence(&line) || _asks_for_a_moment(&line);
    if presence && PRESENCE_NOT.contains(&read.intent) {
        presence = line.norm.iter().any(|w| PRESENCE_SUBJECT.contains(w) || PRESENCE_ASKER.contains(w)) || _asks_for_a_moment(&line);
    }
    if presence {
        intent = "presence".to_string();
    }
    let plain = _a_word_that_depends_on_the_owner(&line.plain, &entities);
    let words = breeds::the_animal_a_breed_is(&line.words);
    let last_read = memory.and_then(|m| m.understood.as_ref());
    let mut animal_in_hand = references::the_animal_in_hand(&reference, &words);
    if animal_in_hand.is_empty() {
        animal_in_hand = references::the_animal_the_chat_is_on(&words, last_read);
    }
    let (over_time, over_time_what) = kinds::over_time(&line, &read, None, false).unwrap_or((None, None));
    let home_lines = _user_lines(memory, HOME_LOOKBACK as usize);
    let car_lines = _user_lines(memory, cars::lookback());
    let topic_is_a_brush = topic_from.iter().all(|(_, w)| *w < 2);
    let asks_what_was_meant = read.asks_what_was_meant && _the_meaning_is_applied_to_what_was_said(&line, leans_on.as_ref(), memory);
    let mut out = Understood {
        text: text_.to_string(),
        clean: line.clean.clone(),
        words,
        plain,
        animal_in_hand,
        change_of_subject: line.change_of_subject,
        dropped_subject: line.dropped_subject,
        sense: senses::senses(&line),
        over_time,
        over_time_what,
        kept: line.kept.clone(),
        kind: if presence { "question".to_string() } else { read.kind.clone() },
        asks_how_i_am: kinds::asks_how_i_am(&line.norm) && (intent == "smalltalk" || intent == "greeting"),
        presence,
        asks: read.asks.clone(),
        tag: read.tag,
        taking_up: kinds::taking_up(&line, Some(&read)),
        shift: kinds::shift_worked(&line.clean),
        care_home: kinds::a_care_home(&line.clean).or_else(|| kinds::a_home_carried_forward(&line.clean, &home_lines)),
        setup: kinds::the_setup(&line),
        help_ask: read.help_ask,
        asks_a_verdict: read.asks_a_verdict,
        emotion: read.emotion.clone(),
        state_owner: read.state_owner.clone(),
        shortage: read.shortage.clone(),
        polarity: read.polarity,
        warmth: topics::the_warmth_said(&line.norm),
        body: body::read(&line.norm_text),
        own_output: topics::own_output_word(&line.norm).map(|p| p as i64),
        short: read.short,
        first_person: read.first_person,
        about_bot: read.about_bot,
        about_bot_kind: read.about_bot_kind.clone(),
        counts_the_bots_own_day: read.counts_the_bots_own_day,
        replies_to_bot: read.replies_to_bot,
        agrees_to_try: read.agrees_to_try && intent == "ack",
        wants_a_quiz: read.wants_a_quiz,
        agreement: read.agreement.clone(),
        tight_on_money: topics::tight_on_money(&line.norm, Some(&read.kind), &line.text),
        amount_left: topics::purse_till_at(&line.norm, &line.text).is_some(),
        routine: kinds::is_a_routine(&line, Some(&read)),
        a_load_carried: a_load_carried(&line.clean),
        car_fault: cars::a_broken_car_is_the_subject(&line.clean, &car_lines),
        summons: read.summons.clone(),
        asks_back: read.asks_back,
        stand_in: _stand_in_here(text_),
        excludes: read.excludes.clone(),
        constraint: read.constraint,
        confusion: confusion::confusion_of(&line),
        suggestion: read.suggestion.clone(),
        ruled_out: read.ruled_out.clone(),
        norm: line.norm.clone(),
        norm_text: line.norm_text.clone(),
        fixes: Fixes::Map(line.fixes.clone()),
        clauses: line.clauses.clone(),
        focus: line.focus.clone(),
        asking: if line.asking.is_empty() { line.focus.clone() } else { line.asking.clone() },
        signals: line.signals.clone(),
        topic_score: score,
        topics: ranked,
        topic_own_score: own_score,
        unsure: topics::unsure_words(&line.norm, own_topic.as_deref().filter(|t_| !t_.is_empty()).or(topic.as_deref())),
        topic_own: own_topic,
        topic,
        topic_from,
        topic_is_a_brush,
        person_names: topics::person_names(&line),
        background: background_positions(&line.norm).into_iter().map(|k| line.norm[k].clone()).collect::<Vec<_>>(),
        intensifier: line.intensifier.clone(),
        intensifier_left: line.intensifier_left.clone(),
        subject,
        subject_unclear: topics::unclear_subject(&line.norm, inherited.as_deref(), previous.as_ref()),
        ask_subject,
        setting,
        subject_from,
        numbers: topics::detect_numbers(&line),
        tried: attempts::things_tried(&line.text, Some(&line.norm)),
        keywords: _keywords(&line.norm),
        followup: reference.followup,
        someone_new: references::person_is_new(&line, &previous_entities),
        asks_what_was_meant,
        leans_on,
        entities,
        reference,
        intent,
        ..Default::default()
    };
    if let Some((ot, what)) = kinds::over_time(&line, &out, last_read, true) {
        out.over_time = ot;
        out.over_time_what = what;
    }
    out
}

#[allow(dead_code)]
fn _unused(_: &BTreeSet<usize>) {}
/// Typed readers for PyBot's module-level tables in the data bundle, by module and Python name:
/// `t::set("bot.understanding.normalize.", "LEADERS")`.
pub mod t {
    use crate::data::{self, Value};
    use crate::py::re::Re;
    use indexmap::IndexMap;
    use std::collections::HashSet;

    fn get(m: &str, name: &str) -> &'static Value {
        data::get(&format!("{m}{name}"))
    }
    pub fn set(m: &str, name: &str) -> HashSet<String> {
        get(m, name).str_set()
    }
    pub fn strs(m: &str, name: &str) -> Vec<String> {
        get(m, name).strs()
    }
    pub fn string(m: &str, name: &str) -> &'static str {
        get(m, name).str()
    }
    pub fn int(m: &str, name: &str) -> i64 {
        get(m, name).as_int().unwrap_or_else(|| panic!("{m}{name} is not an int"))
    }
    pub fn re(m: &str, name: &str) -> &'static Re {
        get(m, name).as_re().unwrap_or_else(|| panic!("{m}{name} is not a regex"))
    }
    pub fn res(m: &str, name: &str) -> Vec<&'static Re> {
        get(m, name).items().iter().map(|v| v.as_re().unwrap()).collect()
    }
    pub fn tuples(m: &str, name: &str) -> Vec<Vec<String>> {
        get(m, name).items().iter().map(Value::strs).collect()
    }
    pub fn re_str_pairs(m: &str, name: &str) -> Vec<(&'static Re, String)> {
        get(m, name).items().iter().map(|p| (p.items()[0].as_re().unwrap(), p.items()[1].str().to_string())).collect()
    }
    pub fn str_re_pairs(m: &str, name: &str) -> Vec<(String, &'static Re)> {
        get(m, name).items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].as_re().unwrap())).collect()
    }
    pub fn map(m: &str, name: &str) -> IndexMap<String, String> {
        get(m, name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
    }
    pub fn map_int(m: &str, name: &str) -> IndexMap<String, i64> {
        get(m, name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0))).collect()
    }
    pub fn map_strs(m: &str, name: &str) -> IndexMap<String, Vec<String>> {
        get(m, name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
    }
    pub fn map_set(m: &str, name: &str) -> IndexMap<String, HashSet<String>> {
        get(m, name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_set())).collect()
    }
    pub fn map_re(m: &str, name: &str) -> IndexMap<String, &'static Re> {
        get(m, name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_re().unwrap())).collect()
    }
    pub fn value(m: &str, name: &str) -> &'static Value {
        get(m, name)
    }
}

/// `pattern.match(text, pos)`: a match anchored at `pos` (a byte offset), with the text before
/// `pos` still seen by lookbehind and `\b`, as Python's `match` with a start position.
pub fn match_at<'t>(re: &Re, text: &'t str, pos: usize) -> Option<crate::py::re::Match<'t>> {
    let p = re.pattern();
    let mut lead_end = 0;
    let b = p.as_bytes();
    while b.len() > lead_end + 3 && b[lead_end] == b'(' && b[lead_end + 1] == b'?' {
        let mut j = lead_end + 2;
        while j < b.len() && b[j].is_ascii_alphabetic() {
            j += 1;
        }
        if j < b.len() && b[j] == b')' && j > lead_end + 2 {
            lead_end = j + 1;
        } else {
            break;
        }
    }
    let sep = if re.flags() & crate::py::re::X != 0 { "\n" } else { "" };
    let wrapped = format!("{}\\G(?:{}{})", &p[..lead_end], &p[lead_end..], sep);
    rx_flags(&wrapped, re.flags()).search_at(text, pos)
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

    /// Python: bot/understanding/__init__.py:425 `_turns(memory)`
    pub fn _turns(memory: &Memory) -> Vec<String> {
        crate::unported!("bot.understanding:_turns")
    }

}

