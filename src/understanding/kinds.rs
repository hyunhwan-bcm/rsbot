//! Port of `bot/understanding/kinds.py`: what kind of line it is (a question, a greeting, news,
//! a feeling), what it asks for, what feeling it carries and whose state it is.

#![allow(non_upper_case_globals)]

use crate::py::re::{escape, Re};
use crate::py::text;
use crate::understanding::normalize::{background_clause, background_positions, opens_without_subject, strip_leaders, Line};
use crate::understanding::{pyuni, rx, t};
use crate::understood::{CareHome, Suggestion, Summons, TakingUp, Understood};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.understanding.kinds.";

/// What `classify(line)` returns (Python's dict, same keys). `_read_line` changes `intent`,
/// `asks`, `replies_to_bot`, `about_bot` and `about_bot_kind` afterwards.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Read {
    pub kind: String,
    pub intent: String,
    pub asks: Vec<String>,
    pub tag: bool,
    pub emotion: Option<String>,
    pub state_owner: Option<String>,
    pub shortage: Option<String>,
    pub polarity: i64,
    pub short: bool,
    pub first_person: bool,
    pub asks_opinion: bool,
    pub asks_recommendation: bool,
    pub asks_steps: bool,
    pub help_ask: bool,
    pub asks_a_verdict: bool,
    pub about_bot: bool,
    pub about_bot_kind: Option<String>,
    pub counts_the_bots_own_day: bool,
    pub replies_to_bot: bool,
    pub agrees_to_try: bool,
    pub wants_a_quiz: bool,
    pub agreement: Option<String>,
    pub summons: Option<Summons>,
    pub asks_back: bool,
    pub asks_what_was_meant: bool,
    pub excludes: Vec<String>,
    pub ruled_out: Vec<String>,
    pub constraint: bool,
    pub suggestion: Option<Suggestion>,
}

/// The keys of a reading that `over_time`, `taking_up` and `is_a_routine` read with
/// `read.get(...)`: they are handed either `classify`'s dict or the finished reading.
pub trait ReadView {
    fn kind(&self) -> Option<&str>;
    fn intent(&self) -> Option<&str>;
    fn asks(&self) -> &[String];
    /// `read.get("followup")` (classify's dict has no such key: false).
    fn followup(&self) -> bool;
    /// `read.get("text")` (None for classify's dict).
    fn text(&self) -> Option<&str>;
    /// `read.get("clean")` (None for classify's dict).
    fn clean(&self) -> Option<&str>;
}

impl ReadView for Read {
    fn kind(&self) -> Option<&str> {
        Some(&self.kind)
    }
    fn intent(&self) -> Option<&str> {
        Some(&self.intent)
    }
    fn asks(&self) -> &[String] {
        &self.asks
    }
    fn followup(&self) -> bool {
        false
    }
    fn text(&self) -> Option<&str> {
        None
    }
    fn clean(&self) -> Option<&str> {
        None
    }
}

impl ReadView for Understood {
    fn kind(&self) -> Option<&str> {
        Some(&self.kind)
    }
    fn intent(&self) -> Option<&str> {
        Some(&self.intent)
    }
    fn asks(&self) -> &[String] {
        &self.asks
    }
    fn followup(&self) -> bool {
        self.followup
    }
    fn text(&self) -> Option<&str> {
        Some(&self.text)
    }
    fn clean(&self) -> Option<&str> {
        Some(&self.clean)
    }
}

// ---- PyBot's tables (bot.understanding.kinds.*) ----
static ABILITY_AT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ABILITY_AT"));
static ABOUT_AT_A_TIME: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ABOUT_AT_A_TIME"));
static ABOUT_BOT: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "ABOUT_BOT"));  // t, 14 items
static ABOUT_BOT_END: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "ABOUT_BOT_END"));  // t, 3 items
static ACK: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ACK"));
static ACTS_ON_THE_SPEAKER: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| t::tuples(M, "ACTS_ON_THE_SPEAKER"));
static ACT_ON_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ACT_ON_FILLER"));
static ACT_ON_LEAD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ACT_ON_LEAD"));
static ADDRESSED: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ADDRESSED"));
static ADDRESSED_LINES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ADDRESSED_LINES"));
static AFFIRM: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AFFIRM"));
static AFTER_COMPARATIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AFTER_COMPARATIVE"));
static AFTER_HELP_WAS_GOOD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AFTER_HELP_WAS_GOOD"));
static AFTER_HELP_WAS_GOOD_SAID_OUT: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "AFTER_HELP_WAS_GOOD_SAID_OUT"));  // t, 6 items
static AGREEMENT_IDIOMS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "AGREEMENT_IDIOMS"));  // t, 10 items
static AGREE_AFTER_NOT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AGREE_AFTER_NOT"));
static AGREE_TAIL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AGREE_TAIL"));
static ALREADY_ASKED: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ALREADY_ASKED"));
static ALWAYS_SELF_ONLY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ALWAYS_SELF_ONLY"));
static ANSWERING_A_QUOTED_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ANSWERING_A_QUOTED_ASK"));
static ANY_ALREADY_ASKS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ANY_ALREADY_ASKS"));
static ANY_ANSWER_LINKS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ANY_ANSWER_LINKS"));
static ANY_ANSWER_WANTED: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ANY_ANSWER_WANTED"));
static ASKING_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASKING_SUBJECT"));
static ASKING_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASKING_VERBS"));
static ASKS_ABOUT_A_PERSON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKS_ABOUT_A_PERSON"));
static ASKS_IF_ANYTHING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASKS_IF_ANYTHING"));
static ASKS_IT_BACK: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "ASKS_IT_BACK"));  // t, 12 items
static ASK_ABOUT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_ABOUT"));
static ASK_AUX: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_AUX"));
static ASK_FOLLOW: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_FOLLOW"));
static ASK_FOR_A_NAME: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_FOR_A_NAME"));
static ASK_FRAMES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_FRAMES"));
static ASK_MODALS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_MODALS"));
static ASK_TARGET: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_TARGET"));
static ASK_THE_ROOM: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_THE_ROOM"));
static ASK_THE_ROOM_ANY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_THE_ROOM_ANY"));
static ASK_THE_ROOM_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_THE_ROOM_VERB"));
static ASK_THE_ROOM_WANTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_THE_ROOM_WANTS"));
static ASK_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_VERBS"));
static ASSISTANT_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASSISTANT_NAMES"));
static A_BODY_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_BODY_SUBJECT"));
static A_BROKEN_HEART: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_BROKEN_HEART"));
static A_COMPARISON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_COMPARISON"));
static A_DATE_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_DATE_FIGURE"));
static A_FRAME_IN_FRONT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_FRAME_IN_FRONT"));
static A_HAVING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_HAVING"));
static A_HOW_MANY_OF_THE_BOTS_OWN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_HOW_MANY_OF_THE_BOTS_OWN"));
static A_LIMIT_ABOUT_PEOPLE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_LIMIT_ABOUT_PEOPLE"));
static A_MONEY_SUBJECT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_MONEY_SUBJECT"));
static A_MONTH_PERIOD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_MONTH_PERIOD"));
static A_NOT_A_SUGGESTION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_NOT_A_SUGGESTION"));
static A_PERIOD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_PERIOD"));
static A_PERSON_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_PERSON_AFTER"));
static A_PERSON_AFTER_A_POSSESSIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_PERSON_AFTER_A_POSSESSIVE"));
static A_PERSON_ELSE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_PERSON_ELSE"));
static A_PERSON_IS_THE_SUBJECT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_PERSON_IS_THE_SUBJECT"));
static A_PERSON_NAMED: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_PERSON_NAMED"));
static A_POSSESSED_HEART: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_POSSESSED_HEART"));
static A_REPORTED_VERB: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_REPORTED_VERB"));
static A_SAID_BY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_SAID_BY"));
static A_SUGGESTED_CLAUSE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_SUGGESTED_CLAUSE"));
static A_THING_NOT_A_CLAIM: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_THING_NOT_A_CLAIM"));
static A_TOLD_TO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_TOLD_TO"));
static A_WHAT_ITS_LIKE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_WHAT_ITS_LIKE"));
static A_YOUR_KIN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_YOUR_KIN"));
static A_YOUR_OWN_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_YOUR_OWN_THING"));
static BACK_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BACK_AFTER"));
static BACK_ASKING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BACK_ASKING"));
static BACK_AT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BACK_AT"));
static BACK_INTO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BACK_INTO"));
static BACK_JOIN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BACK_JOIN"));
static BACK_NOT_A_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BACK_NOT_A_SUBJECT"));
static BARE_DEFINITION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BARE_DEFINITION"));
static BEEN_SAID: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BEEN_SAID"));
static BEEN_WHEN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BEEN_WHEN"));
static BEFORE_COURTESY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BEFORE_COURTESY"));
static BEFORE_HELP_WAS_GOOD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BEFORE_HELP_WAS_GOOD"));
static BEFORE_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BEFORE_STATE"));
static BETTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BETTER"));
static BETTER_OFF: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BETTER_OFF"));
static BETWEEN_STATE_AND_NOUN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BETWEEN_STATE_AND_NOUN"));
static BE_FORMS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BE_FORMS"));
static BODY_STATE_CUES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BODY_STATE_CUES"));
static BODY_STATE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BODY_STATE_WORDS"));
static BODY_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BODY_SUBJECT"));
static BOT_CHATS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BOT_CHATS"));
static BOT_COUNTED: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BOT_COUNTED"));
static BOT_DAY_SHAPES: LazyLock<Vec<&'static Re>> = LazyLock::new(|| t::res(M, "BOT_DAY_SHAPES"));
static BOT_DOERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BOT_DOERS"));
static BOT_DOINGS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BOT_DOINGS"));
static BOT_HAVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BOT_HAVE"));
static BOT_LIFE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BOT_LIFE"));
static BOT_LIFE_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BOT_LIFE_ASK"));
static BOT_LIFE_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BOT_LIFE_FILLER"));
static BOUT: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "BOUT"));
static BROKEN_HEARTED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BROKEN_HEARTED"));
static BROKE_A_HEART: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BROKE_A_HEART"));
static CAPABILITY: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "CAPABILITY"));  // t, 10 items
static CAPABILITY_CARRIER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CAPABILITY_CARRIER"));
static CAPABILITY_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CAPABILITY_FILLER"));
static CARE_HOME_BRING_ASKS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CARE_HOME_BRING_ASKS"));
static CARE_HOME_BRING_WHO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CARE_HOME_BRING_WHO"));
static CARE_HOME_BRING_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CARE_HOME_BRING_WORD"));
static CARE_HOME_GUILT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CARE_HOME_GUILT"));
static CARE_HOME_KIN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CARE_HOME_KIN"));
static CARE_HOME_MOVED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CARE_HOME_MOVED"));
static CARE_HOME_NOT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CARE_HOME_NOT"));
static CARE_HOME_PLACE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CARE_HOME_PLACE"));
static CARE_HOME_VISIT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CARE_HOME_VISIT"));
static CHILD_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CHILD_NOUNS"));
static CLAIM_STEMS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CLAIM_STEMS"));
static CLAUSE_HELPERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CLAUSE_HELPERS"));
static CLAUSE_OPENERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CLAUSE_OPENERS"));
static CLAUSE_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CLAUSE_VERBS"));
static COMMIT_ALONE: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "COMMIT_ALONE"));  // t, 11 items
static COMMIT_ALONE_TAIL: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "COMMIT_ALONE_TAIL"));  // t, 2 items
static COMMIT_NO_HARM: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "COMMIT_NO_HARM"));  // t, 8 items
static COMMIT_OPEN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COMMIT_OPEN"));
static COMMIT_STOP: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COMMIT_STOP"));
static COMMIT_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COMMIT_SUBJECT"));
static COMMIT_TRY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COMMIT_TRY"));
static COMPARATIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COMPARATIVE"));
static COMPARE: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "COMPARE"));  // t, 5 items
static COMPARISON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "COMPARISON"));
static CONFUSED_SELF_ONLY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CONFUSED_SELF_ONLY"));
static CONJUNCTIONS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CONJUNCTIONS"));
static COPULA: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COPULA"));
static COPULAS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COPULAS"));
static COPULA_BEFORE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COPULA_BEFORE"));
static COPULA_STATES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COPULA_STATES"));
static CORRECTION: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "CORRECTION"));  // t, 12 items
static COUNT_FILLERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COUNT_FILLERS"));
static COUNT_OPENERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COUNT_OPENERS"));
static COURTESY_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COURTESY_THING"));
static COURTESY_WORD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COURTESY_WORD"));
static CUE_FOLLOW: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CUE_FOLLOW"));
static DEFINITION: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "DEFINITION"));  // t, 7 items
static DEGREE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DEGREE"));
static DENY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DENY"));
static DENY_BEFORE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DENY_BEFORE"));
static DESCRIBES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DESCRIBES"));
static DETERMINERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DETERMINERS"));
static DOING_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DOING_FILLER"));
static EPISODE_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "EPISODE_NOUNS"));
static EVENT_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "EVENT_NOUNS"));
static EVENT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "EVENT_WORDS"));
static EXPERIENCE_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "EXPERIENCE_ASK"));
static EXPERIENCE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "EXPERIENCE_WORDS"));
static FALL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FALL"));
static FAREWELLS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "FAREWELLS"));  // t, 30 items
static FAREWELL_TAIL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FAREWELL_TAIL"));
static FEELING_COPULA: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FEELING_COPULA"));
static FEELING_CUES_BEFORE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FEELING_CUES_BEFORE"));
static FEELING_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FEELING_FILLER"));
static FEELING_NOT_MOOD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FEELING_NOT_MOOD"));
static FEELING_SOUND_FRAME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FEELING_SOUND_FRAME"));
static FELL_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FELL_WORDS"));
static FILLER_AFTER: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "FILLER_AFTER"));  // t, 6 items
static FILLER_BEFORE: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "FILLER_BEFORE"));  // t, 7 items
static FIRST_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FIRST_PERSON"));
static FOUND: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FOUND"));
static FRAME_NOT_A_SETUP: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FRAME_NOT_A_SETUP"));
static FREE_OR_BUSY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FREE_OR_BUSY"));
static GENDER_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GENDER_WORDS"));
static GETTING_WORSE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "GETTING_WORSE"));
static GIVEN_ADVICE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GIVEN_ADVICE"));
static GOING_ON_WHEN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GOING_ON_WHEN"));
static GOOD_EMOTION_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GOOD_EMOTION_NAMES"));
static GOOD_WORD_BY_COMPARISON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "GOOD_WORD_BY_COMPARISON"));
static GOOD_WORD_COMPARED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "GOOD_WORD_COMPARED"));
static GOOD_WORD_MISSING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "GOOD_WORD_MISSING"));
static GRATITUDE_ASK_AGAIN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GRATITUDE_ASK_AGAIN"));
static GRATITUDE_JOINERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GRATITUDE_JOINERS"));
static GRATITUDE_OBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GRATITUDE_OBJECT"));
static GREETINGS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "GREETINGS"));  // t, 14 items
static GREETING_ASK_ABOUT_SOMETHING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GREETING_ASK_ABOUT_SOMETHING"));
static GREETING_ASK_SECOND_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GREETING_ASK_SECOND_PERSON"));
static GREETING_ASK_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GREETING_ASK_STATE"));
static GREETING_ASK_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GREETING_ASK_SUBJECT"));
static GREETING_ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GREETING_ASK_WORDS"));
static GREETING_TAIL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GREETING_TAIL"));
static GREETING_WHAT_SAYING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GREETING_WHAT_SAYING"));
static HAND_OVER_LEADS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HAND_OVER_LEADS"));
static HAND_OVER_MINE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HAND_OVER_MINE"));
static HAND_OVER_MODALS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HAND_OVER_MODALS"));
static HAND_OVER_SUBJECTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HAND_OVER_SUBJECTS"));
static HAND_OVER_TO: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HAND_OVER_TO"));
static HAND_OVER_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HAND_OVER_VERBS"));
static HAVE_A_BROKEN_HEART: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HAVE_A_BROKEN_HEART"));
static HEART_IN_A_STATE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HEART_IN_A_STATE"));
static HEDGED_ALONE: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "HEDGED_ALONE"));  // t, 16 items
static HEDGED_COMMIT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HEDGED_COMMIT"));
static HEDGED_DOUBT: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "HEDGED_DOUBT"));  // t, 8 items
static HEDGED_LOW: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HEDGED_LOW"));
static HEDGED_OK: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HEDGED_OK"));
static HEDGED_WRAPPER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HEDGED_WRAPPER"));
static HEDGE_LEAD_ONE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HEDGE_LEAD_ONE"));
static HEDGE_LEAD_TWO: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "HEDGE_LEAD_TWO"));  // t, 4 items
static HELP_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HELP_ASK"));
static HELP_ASK_NOT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HELP_ASK_NOT"));
static HELP_ASK_OWN_SUBJECT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HELP_ASK_OWN_SUBJECT"));
static HELP_ASK_SUBJECT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HELP_ASK_SUBJECT"));
static HELP_ASK_TAIL_CLAUSE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HELP_ASK_TAIL_CLAUSE"));
static HELP_WAS_GOOD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HELP_WAS_GOOD"));
static IDIOM_STATES: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "IDIOM_STATES"));
static ING_TAKES_NO_OBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ING_TAKES_NO_OBJECT"));
static ING_THINGS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ING_THINGS"));
static INTEREST_HEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "INTEREST_HEAD"));
static INTEREST_NOT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "INTEREST_NOT"));
static INTEREST_TAIL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "INTEREST_TAIL"));
static INTRODUCES: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| t::tuples(M, "INTRODUCES"));
static ITSELF_LIFE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ITSELF_LIFE"));
static ITSELF_LONG: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ITSELF_LONG"));
static ITSELF_WHEN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ITSELF_WHEN"));
static ITSELF_WHEN_TAIL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ITSELF_WHEN_TAIL"));
static JOIN_OPENERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "JOIN_OPENERS"));
static KEEPING_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "KEEPING_ASK"));
static KEEPS_BUSY_SHAPE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "KEEPS_BUSY_SHAPE"));
static LABEL_ON_A_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LABEL_ON_A_PERSON"));
static LAUGH: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LAUGH"));
static LAUGHED_OFF_AFTER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LAUGHED_OFF_AFTER"));
static LAUGH_AT_THE_END: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LAUGH_AT_THE_END"));
static LEARNED_ON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LEARNED_ON"));
static LEARNING_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LEARNING_WORDS"));
static LETTER_ASKED_FOR: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LETTER_ASKED_FOR"));
static LETTER_ITSELF: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LETTER_ITSELF"));
static LETTER_RECEIVED: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LETTER_RECEIVED"));
static LETTER_SOMEONE_ELSES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LETTER_SOMEONE_ELSES"));
static LIMIT_A_FACT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LIMIT_A_FACT"));
static LIMIT_NOT_A_START: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LIMIT_NOT_A_START"));
static LIMIT_NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LIMIT_NOT_A_THING"));
static LIMIT_REFUSAL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LIMIT_REFUSAL"));
const LIMIT_ROOM: i64 = 3;
static LIMIT_SHAPES: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| t::str_re_pairs(M, "LIMIT_SHAPES"));
static LIMIT_SOFT_AFTER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LIMIT_SOFT_AFTER"));
static LIMIT_SOFT_BEFORE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LIMIT_SOFT_BEFORE"));
static MAKER_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MAKER_WORDS"));
static MANIPULATE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MANIPULATE"));
static MANNERS_OPENERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MANNERS_OPENERS"));
static MANNERS_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MANNERS_WORDS"));
static MATH_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MATH_WORDS"));
static ME_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "ME_ASKS"));  // t, 23 items
static ME_LEAD: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "ME_LEAD"));  // t, 9 items
static ME_TAIL: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "ME_TAIL"));  // t, 17 items
static MOOD_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MOOD_NOUNS"));
static MOOD_TAIL: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "MOOD_TAIL"));  // t, 11 items
static MOOD_TAIL_TAKEN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MOOD_TAIL_TAKEN"));
static MOOD_TAIL_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MOOD_TAIL_WORDS"));
static MOTION_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MOTION_WORDS"));
static NAMING_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NAMING_WORDS"));
static NEEDS_ANSWER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NEEDS_ANSWER"));
static NEGATIONS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NEGATIONS"));
static NEGATION_SKIP: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NEGATION_SKIP"));
static NEGATIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NEGATIVE"));
static NEGATORS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NEGATORS"));
static NEWS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "NEWS"));  // t, 18 items
static NEW_ASK_AHEAD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NEW_ASK_AHEAD"));
static NEW_ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NEW_ASK_WORDS"));
static NEW_FRAME: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NEW_FRAME"));
static NOT_AFTER_THE_BOT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_AFTER_THE_BOT"));
static NOT_AUX: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_AUX"));
static NOT_A_DEFINITION_HEAD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_DEFINITION_HEAD"));
static NOT_A_MOOD_THERE: LazyLock<IndexMap<String, &'static Re>> = LazyLock::new(|| t::map_re(M, "NOT_A_MOOD_THERE"));
static NOT_A_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_SUBJECT"));
static NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_THING"));
static NOT_BEFORE_THE_BOT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_BEFORE_THE_BOT"));
static NOT_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_FILLER"));
static NOT_THEIR_OWN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_THEIR_OWN"));
static NOT_THE_SPEAKERS_DAY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_THE_SPEAKERS_DAY"));
const NOUN_REQUEST_MAX: i64 = 8;
static NOUN_REQUEST_SELF: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOUN_REQUEST_SELF"));
static NOUN_STATES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOUN_STATES"));
static NO_BETTER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NO_BETTER"));
static NO_HARM_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NO_HARM_SUBJECT"));
static NUMBER_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NUMBER_WORDS"));
static OBJECT_MARKERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OBJECT_MARKERS"));
static OK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OK_WORDS"));
static OPENING_FRAME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "OPENING_FRAME"));
const OPENING_MAX: i64 = 9;
static OPENING_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OPENING_WORDS"));
static OPENS_A_FRAME: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OPENS_A_FRAME"));
static OPINION: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "OPINION"));  // t, 10 items
static OPPOSITE_PAIRS: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| t::tuples(M, "OPPOSITE_PAIRS"));
static OTHER_PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OTHER_PEOPLE"));
static OWNS_A_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OWNS_A_PERSON"));
static OWN_PERIOD_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OWN_PERIOD_FILLER"));
static OWN_PERIOD_HEADS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OWN_PERIOD_HEADS"));
static OWN_PERIOD_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OWN_PERIOD_NOUNS"));
const OWN_PERIOD_REACH: i64 = 6;
static OWN_PERIOD_STOPS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OWN_PERIOD_STOPS"));
static OWN_RESERVES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OWN_RESERVES"));
static PARTICLE_STATES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PARTICLE_STATES"));
static PARTS_OF_ME: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PARTS_OF_ME"));
static PAST_ASK: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PAST_ASK"));
static PAST_AUX: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PAST_AUX"));
static PAST_ENDS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PAST_ENDS"));
static PERIOD_CUES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PERIOD_CUES"));
static PERSON_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PERSON_NOUNS"));
static PERSON_PREPOSITIONS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PERSON_PREPOSITIONS"));
static PERSON_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PERSON_SUBJECT"));
static PHRASE_OPENERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PHRASE_OPENERS"));
static PICKING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PICKING"));
static PICKING_QUALITY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PICKING_QUALITY"));
static PLACE_WORD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLACE_WORD"));
static POINTED_AT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "POINTED_AT"));
static POLITE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "POLITE"));
static POSITIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "POSITIVE"));
static POSSESSIVE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "POSSESSIVE_WORDS"));
static PRAISE_SAID: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PRAISE_SAID"));
static PRAISING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PRAISING"));
static PRAISING_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PRAISING_VERB"));
static PRODUCT_ASK_TAIL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PRODUCT_ASK_TAIL"));
const PRODUCT_ASK_WORDS: i64 = 7;
static PRONOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PRONOUNS"));
static PRONOUN_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PRONOUN_SUBJECT"));
static QUESTION_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "QUESTION_NOUNS"));
static QUESTION_STARTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "QUESTION_STARTS"));
static QUESTION_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "QUESTION_WORD"));
static QUESTION_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "QUESTION_WORDS"));
static RECOMMEND: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RECOMMEND"));
static RECOMMEND_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "RECOMMEND_PHRASES"));  // t, 13 items
static REFERRING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "REFERRING"));
static REFUSED_PAIR: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "REFUSED_PAIR"));
static RELATIVE_SUBJECTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RELATIVE_SUBJECTS"));
static ROBOT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ROBOT_WORDS"));
static ROUTINE_IN_THE_PAST: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ROUTINE_IN_THE_PAST"));
static ROUTINE_IS_A_FAULT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ROUTINE_IS_A_FAULT"));
static ROUTINE_SHAPE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ROUTINE_SHAPE"));
static RULED_OUT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RULED_OUT"));
static RULED_OUT_CORRECTION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RULED_OUT_CORRECTION"));
static RULED_OUT_ENDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RULED_OUT_ENDS"));
static RULED_OUT_NO_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RULED_OUT_NO_THING"));
static SAID_AS_A_NEGATION: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SAID_AS_A_NEGATION"));
static SAYS_ABOUT_YOU: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SAYS_ABOUT_YOU"));
static SAYS_HOW_THEY_ARE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SAYS_HOW_THEY_ARE"));
static SAYS_WHAT_ARE_YOU: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SAYS_WHAT_ARE_YOU"));
static SAYS_WHAT_IT_IS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SAYS_WHAT_IT_IS"));
static SCALE_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SCALE_ASK"));
static SELF_ASK: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SELF_ASK"));
static SELF_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SELF_FILLER"));
static SELF_ONLY_CUES: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "SELF_ONLY_CUES"));  // t, 26 items
static SELF_ONLY_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SELF_ONLY_STATE"));
static SELF_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SELF_WORDS"));
static SETUP_CONNECTIVES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SETUP_CONNECTIVES"));
const SETUP_MOST_WORDS: i64 = 45;
static SETUP_NOTHING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SETUP_NOTHING"));
static SHIFT_EARLY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SHIFT_EARLY"));
static SHIFT_LATE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SHIFT_LATE"));
static SHIFT_NIGHT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SHIFT_NIGHT"));
static SHORT_REPLIES: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "SHORT_REPLIES"));  // t, 78 items
static SIGNING_OFF: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SIGNING_OFF"));
static SLIPPED_FROM: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SLIPPED_FROM"));
static SMALL_TALK_LEFTOVER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SMALL_TALK_LEFTOVER"));
static SOUND_FOLLOWS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SOUND_FOLLOWS"));
static SPEAKER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SPEAKER"));
static SPEAKER_TAIL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SPEAKER_TAIL"));
static SPENDS_ITS_TIME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SPENDS_ITS_TIME"));
static SPENT_TIME: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SPENT_TIME"));
static STARTED_OUT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "STARTED_OUT"));
static STATE_AS_FEELING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STATE_AS_FEELING"));
static STATE_JOINERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STATE_JOINERS"));
static STATE_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STATE_NOUNS"));
static STATE_TAIL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "STATE_TAIL"));
static STATE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STATE_WORDS"));
static STATING_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STATING_VERB"));
static STEPS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "STEPS"));  // t, 8 items
static STEPS_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "STEPS_WORD"));
static STEP_ANY_OBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STEP_ANY_OBJECT"));
static STEP_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STEP_VERBS"));
static STEP_WRONG: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STEP_WRONG"));
static STILL_COMING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STILL_COMING"));
static STILL_WRONG: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "STILL_WRONG"));
static STOMACH_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STOMACH_WORDS"));
static STOP_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "STOP_ASK"));
static STOP_ASK_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STOP_ASK_FILLER"));
static SUBJECT_NEGATIONS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUBJECT_NEGATIONS"));
static SUBJECT_OF_ITS_OWN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUBJECT_OF_ITS_OWN"));
static SUMMONS_CALL_DONE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUMMONS_CALL_DONE"));
static SUMMONS_NOT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUMMONS_NOT"));
static SUMMONS_RANG: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "SUMMONS_RANG"));  // t, 8 items
static SUMMONS_SAID: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUMMONS_SAID"));
static SUMMONS_SEE: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "SUMMONS_SEE"));  // t, 15 items
static SUMMONS_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUMMONS_VERB"));
static SUMMONS_WANTS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "SUMMONS_WANTS"));  // t, 10 items
static SUMMONS_WHO: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUMMONS_WHO"));
static SUMMONS_WHO_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "SUMMONS_WHO_PHRASES"));  // t, 13 items
static SUMMONS_WRITTEN: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "SUMMONS_WRITTEN"));  // t, 8 items
static SUPPLY_PREPOSITIONS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUPPLY_PREPOSITIONS"));
static SUPPLY_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUPPLY_WORDS"));
static TAG_CHECKABLE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TAG_CHECKABLE"));
static TAG_ENDINGS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TAG_ENDINGS"));
static TAG_PLAN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TAG_PLAN"));
static TAG_RIGHT_NOT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TAG_RIGHT_NOT"));
static TAG_TIME_NUMBER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TAG_TIME_NUMBER"));
static TAIL_ASKS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TAIL_ASKS"));
static TAIL_ASK_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TAIL_ASK_FILLER"));
static TAIL_STOPS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TAIL_STOPS"));
static TALK_HEAD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TALK_HEAD"));
static TALK_OPENING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TALK_OPENING"));
static TALK_TAIL: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| t::tuples(M, "TALK_TAIL"));
static TASTE_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TASTE_VERBS"));
static TERMS_OF_ADDRESS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TERMS_OF_ADDRESS"));
static THANKS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "THANKS"));  // t, 10 items
static THANKS_ABOUT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THANKS_ABOUT"));
static THANKS_ASK_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THANKS_ASK_NOUNS"));
static THANKS_ASK_PREPS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THANKS_ASK_PREPS"));
static THANKS_ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THANKS_ASK_WORDS"));
static THANKS_NOISE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THANKS_NOISE"));
static THEIR_PROBLEM: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THEIR_PROBLEM"));
static THING_OR_PERSON_CUES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THING_OR_PERSON_CUES"));
static TIME_AFTER_A_PARTICLE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TIME_AFTER_A_PARTICLE"));
static TIME_FILLERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TIME_FILLERS"));
static TIME_MARKERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TIME_MARKERS"));
static TIPS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TIPS"));
static TIPS_ABOUT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TIPS_ABOUT"));
static TIPS_AMOUNT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TIPS_AMOUNT"));
static TIPS_FRAME: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TIPS_FRAME"));
static TIPS_SUBJECT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TIPS_SUBJECT"));
static TIPS_WANTED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TIPS_WANTED"));
static TIP_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TIP_WORDS"));
static TO_A_VERB: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TO_A_VERB"));
static TURNED_ROUND: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TURNED_ROUND"));
static UP_TO_LEADS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "UP_TO_LEADS"));
static VERBISH: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "VERBISH"));
static VERB_FORMS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "VERB_FORMS"));
static VERB_ONLY_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "VERB_ONLY_STATE"));
static VERDICT_ADJECTIVES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "VERDICT_ADJECTIVES"));
static VERDICT_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "VERDICT_ASK"));
static VERDICT_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "VERDICT_FILLER"));
static VERDICT_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "VERDICT_NOUNS"));
static WHEN_IN_THE_ASK: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WHEN_IN_THE_ASK"));
static WHERE_BOT_TAIL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WHERE_BOT_TAIL"));
static WHILE_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHILE_ASK"));
static WHO_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WHO_A_THING"));
static WHO_TAIL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WHO_TAIL"));
static WHO_TALKING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WHO_TALKING"));
static WH_CONTINUES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WH_CONTINUES"));
static WORK_ON_ME: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "WORK_ON_ME"));
static YESNO_STARTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "YESNO_STARTS"));
static YOU_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "YOU_WORDS"));
static _ASKS_AFTER_A_COMMA: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_ASKS_AFTER_A_COMMA"));
static _ASKS_AFTER_A_COMMA_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_ASKS_AFTER_A_COMMA_WORD"));
static _ASKS_MID_LINE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_ASKS_MID_LINE"));
static _ASKS_WITH_AN_AUXILIARY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_ASKS_WITH_AN_AUXILIARY"));
static _ASKS_WITH_A_QUESTION_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_ASKS_WITH_A_QUESTION_WORD"));
static _A_AN_AMOUNT_OF_A_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_AN_AMOUNT_OF_A_THING"));
static _A_AN_OCCASION_OF_THEIR_OWN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_AN_OCCASION_OF_THEIR_OWN"));
static _A_ARITHMETIC: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_ARITHMETIC"));
static _A_A_CONVERSION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_A_CONVERSION"));
static _A_A_DIFFERENCE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_A_DIFFERENCE"));
static _A_A_PLAN_ABOUT_THE_WEATHER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_A_PLAN_ABOUT_THE_WEATHER"));
static _A_A_TRAFFIC_LIGHT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_A_TRAFFIC_LIGHT"));
const _A_BARE_ASK: i64 = 4;
static _A_BARE_LIVE: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| t::str_re_pairs(M, "_A_BARE_LIVE"));
const _A_BARE_LIVE_WORDS: i64 = 9;
static _A_BARE_RECORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_BARE_RECORD"));
static _A_BARE_TAIL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_BARE_TAIL"));
static _A_DEFINITION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_DEFINITION"));
static _A_DETERMINER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_DETERMINER"));
static _A_DID_YOU_SEE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_DID_YOU_SEE"));
const _A_FOLLOW_UP_WORDS: i64 = 7;
static _A_FRESHENING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_FRESHENING"));
static _A_HAS_A_NUMBER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_HAS_A_NUMBER"));
static _A_HISTORY_ITSELF: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_HISTORY_ITSELF"));
static _A_HOW_A_THING_MEASURES: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_HOW_A_THING_MEASURES"));
static _A_HOW_IT_WORKS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_HOW_IT_WORKS"));
static _A_HOW_MUCH_A_FEELING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_HOW_MUCH_A_FEELING"));
static _A_HOW_MUCH_ENOUGH: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_HOW_MUCH_ENOUGH"));
static _A_HOW_MUCH_IS_IN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_HOW_MUCH_IS_IN"));
static _A_HOW_MUCH_IT_HOLDS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_HOW_MUCH_IT_HOLDS"));
static _A_HOW_MUCH_I_SPENT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_HOW_MUCH_I_SPENT"));
static _A_HOW_MUCH_OF_A_UNIT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_HOW_MUCH_OF_A_UNIT"));
static _A_HOW_OLD_IS_THE_WORLD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_HOW_OLD_IS_THE_WORLD"));
static _A_MY_OWN_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_MY_OWN_FIGURE"));
static _A_MY_OWN_PLAN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_MY_OWN_PLAN"));
static _A_PAST_YEAR: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_PAST_YEAR"));
static _A_REPORTS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_REPORTS"));
static _A_REPORTS_THE_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_REPORTS_THE_ASK"));
static _A_SAYS_NOW: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_SAYS_NOW"));
static _A_THE_CLOCK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_THE_CLOCK"));
static _A_WEATHER_HERE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_WEATHER_HERE"));
static _A_WORD_MEANS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_WORD_MEANS"));
static _I_SAID_IT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_I_SAID_IT"));
static _MONEY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_MONEY"));
static _ROUTINE_SUBJECT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_ROUTINE_SUBJECT"));
static _SAID_ON_ITS_OWN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_SAID_ON_ITS_OWN"));
static _SEALED_ASKS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_SEALED_ASKS"));
static _SETTLED: LazyLock<Vec<&'static Re>> = LazyLock::new(|| t::res(M, "_SETTLED"));
static _THANKS_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_THANKS_WORDS"));
static _WISH_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_WISH_WORDS"));

/// `EMOTION_WORDS`: feeling name -> its cue words, in Python's dict order.
pub(crate) static EMOTION_WORDS: LazyLock<Vec<(String, Vec<String>)>> =
    LazyLock::new(|| t::value(M, "EMOTION_WORDS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect());
static SMALL_TALK_WORDS: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| t::tuples(M, "SMALL_TALK_WORDS"));
static CLAIM_ENDS: LazyLock<Vec<(usize, String)>> = LazyLock::new(|| {
    t::value(M, "CLAIM_ENDS").items().iter().map(|p| (p.items()[0].as_int().unwrap_or(0) as usize, p.items()[1].str().to_string())).collect()
});
/// A pattern in `_CHANGING`: a regex, or a named service with a day or time (`_AServiceOnADay`).
enum Changing {
    Re(&'static Re),
    Service(&'static Re, &'static Re),
}
impl Changing {
    /// `.search(text)`: the span of the match (the earlier of the two for a service).
    fn search(&self, text: &str) -> Option<(usize, usize)> {
        match self {
            Changing::Re(r) => r.search(text).map(|m| (m.start(), m.end())),
            Changing::Service(service, when) => {
                let s = service.search(text)?;
                let w = when.search(text)?;
                Some(if s.start() <= w.start() { (s.start(), s.end()) } else { (w.start(), w.end()) })
            }
        }
    }
}
static _CHANGING: LazyLock<Vec<(String, Changing)>> = LazyLock::new(|| {
    t::value(M, "_CHANGING")
        .items()
        .iter()
        .map(|p| {
            let what = p.items()[0].str().to_string();
            let v = &p.items()[1];
            let pattern = match v.as_re() {
                Some(r) => Changing::Re(r),
                None => Changing::Service(v.attr("service").and_then(|s| s.as_re()).unwrap(), v.attr("when").and_then(|s| s.as_re()).unwrap()),
            };
            (what, pattern)
        })
        .collect()
});
static RELATIONS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.topics.", "RELATIONS"));
static MEAN: LazyLock<&'static Re> = LazyLock::new(|| t::re("bot.memory.meant.", "MEAN"));
static LX_COMMON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.lexicon.", "COMMON"));
static LX_COMMON_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.lexicon.", "COMMON_NAMES"));
/// `_a_class_of_thing()`: every class word the ruled-out tables know, and their members.
static _A_CLASS_OF_THING: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let mut words = t::set("bot.compose.ruledout.", "RULED_OUT_CLASS");
    for (_, members) in t::value("bot.compose.ruledout.", "RULED_OUT_INSTANCES").pairs() {
        words.extend(members.strs());
    }
    words
});

// ---- helpers ----

/// `re.finditer(r'\b' + re.escape(cue) + r'\b', text)`'s pattern.
fn cue_rx(cue: &str) -> &'static Re {
    rx(&format!(r"\b{}\b", escape(cue)))
}
/// `word.strip('.,!?;:-|"\'').lower()` for every word of a text.
fn stripped_words(s: &str) -> Vec<String> {
    text::split_ref(s).into_iter().map(|w| text::strip_chars(w, ".,!?;:-|\"'").to_lowercase()).collect()
}
fn has(set: &HashSet<String>, w: Option<&str>) -> bool {
    w.is_some_and(|w| set.contains(w))
}
fn sv(norm: &[String]) -> HashSet<&str> {
    norm.iter().map(String::as_str).collect()
}
fn any_in(words: &HashSet<&str>, set: &HashSet<String>) -> bool {
    words.iter().any(|w| set.contains(*w))
}
fn any_of(words: &HashSet<&str>, list: &[&str]) -> bool {
    list.iter().any(|w| words.contains(w))
}

/// `_asks_for_help(text)`: "can you help me with ...".
fn _asks_for_help(text: &str) -> bool {
    if HELP_ASK_OWN_SUBJECT.is_match(text) {
        return false;
    }
    for m in HELP_ASK.finditer(text) {
        if HELP_ASK_NOT.is_match(&text[..m.start()]) {
            continue;
        }
        let tail = &text[m.end()..];
        if HELP_ASK_SUBJECT.is_match(tail) || HELP_ASK_TAIL_CLAUSE.is_match(tail) {
            continue;
        }
        return true;
    }
    false
}

/// `_asks_a_verdict(text)`: "is it worth it", "was i wrong".
fn _asks_a_verdict(text: &str) -> bool {
    let flat = rx("[^a-z0-9 ]+").sub(" ", &text.to_lowercase());
    let flat = rx(r"\s+").sub(" ", &flat);
    let flat = text::strip(&flat);
    if flat.is_empty() {
        return false;
    }
    let words = text::split_ref(flat);
    if words.len() > 30 {
        return false;
    }
    if VERDICT_ASK.fullmatch(flat).is_some() {
        return true;
    }
    for start in 1..words.len().saturating_sub(1) {
        if words.len() - start > 7 {
            continue;
        }
        if QUESTION_WORD.is_match(&words[..start].join(" ")) {
            return false;
        }
        if VERDICT_ASK.fullmatch(&words[start..].join(" ")).is_some() {
            return true;
        }
    }
    false
}

/// `_claim_forms(word)`.
fn _claim_forms(word: &str) -> HashSet<String> {
    let mut forms: HashSet<String> = HashSet::new();
    forms.insert(word.to_string());
    let len = text::len(word);
    for (cut, _suffix) in CLAIM_ENDS.iter() {
        if len > cut + 2 {
            forms.insert(text::slice(word, 0, Some(-(*cut as isize))));
        }
    }
    if word.ends_with("ing") {
        forms.insert(format!("{}e", text::slice(word, 0, Some(-3))));
    }
    for form in forms.clone() {
        let f: Vec<char> = form.chars().collect();
        if f.len() > 3 && f[f.len() - 1] == f[f.len() - 2] {
            forms.insert(f[..f.len() - 1].iter().collect());
        }
    }
    forms.into_iter().map(|f| if f.ends_with('s') && text::len(&f) > 3 { f[..f.len() - 1].to_string() } else { f }).collect()
}

/// `names_a_claim(word)`: the word names a claim (a refund, a complaint).
pub fn names_a_claim(word: &str) -> bool {
    let word = word.to_lowercase();
    if A_THING_NOT_A_CLAIM.contains(&word) {
        return true;
    }
    _claim_forms(&word).iter().any(|f| CLAIM_STEMS.contains(f))
}

/// `_ruled_out_of(text)`: "not the red one" -> the words ruled out.
fn _ruled_out_of(text: &str) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }
    let mut gone: Vec<String> = Vec::new();
    for m in RULED_OUT.finditer(text) {
        let said_before = &text[..m.start()];
        let all: Vec<String> = text::split_ref(said_before).iter().map(|w| text::strip_chars(w, " .,!?;:'\u{2019}").to_lowercase()).collect();
        let before = &all[all.len().saturating_sub(3)..];
        if !before.iter().any(|w| !w.is_empty() && !RULED_OUT_NO_THING.contains(w)) {
            continue;
        }
        let group: Vec<&str> = text::split_ref(m.str(1));
        if group.iter().any(|w| _A_CLASS_OF_THING.contains(&w.to_lowercase())) {
            continue;
        }
        let same_thing = group.last().map(|w| w.to_lowercase()).is_some_and(|w| w == "one" || w == "ones");
        if group.is_empty() {
            panic!("IndexError: list index out of range");
        }
        if !same_thing && !RULED_OUT_CORRECTION.is_match(said_before) {
            continue;
        }
        for word in group {
            let word = text::strip_chars(word, " .,!?;:'\u{2019}\u{201c}\u{201d}");
            if word.is_empty() {
                break;
            }
            if RULED_OUT_ENDS.contains(&word.to_lowercase()) {
                break;
            }
            gone.push(word.to_string());
            if gone.len() >= 4 {
                break;
            }
        }
    }
    gone
}

/// `_limit_of(text)`: "nothing with nuts" -> "nuts".
fn _limit_of(text: &str) -> Option<String> {
    if text.is_empty() {
        return None;
    }
    let mut ask = text::strip_chars(&text::squash(&text.replace('|', " ")), " .,!?").to_lowercase();
    for _ in 0..3 {
        let shorter = LIMIT_SOFT_BEFORE.sub("", &ask);
        if shorter != ask {
            ask = shorter;
        }
        let shorter = text::strip_chars(&LIMIT_SOFT_AFTER.sub("", &ask), " .,!?").to_string();
        if shorter != ask {
            ask = shorter;
        }
        let again = text::strip_chars(&LIMIT_SOFT_BEFORE.sub("", &LIMIT_SOFT_AFTER.sub("", &ask)), " .,!?").to_string();
        if ask == again {
            break;
        }
    }
    for (_name, shape) in LIMIT_SHAPES.iter() {
        let Some(m) = shape.search(&ask) else { continue };
        let mut thing = text::strip_chars(&text::squash(m.str(1)), " .,!?").to_string();
        let mut words: Vec<String> = text::split(&thing);
        if words.len() > 1 && matches!(words.last().unwrap().as_str(), "one" | "ones") {
            thing = words[..words.len() - 1].join(" ");
            words = text::split(&thing);
        }
        if words.is_empty() || words.len() as i64 > LIMIT_ROOM {
            continue;
        }
        if words.iter().any(|w| LIMIT_NOT_A_THING.contains(w)) || words.iter().any(|w| LIMIT_REFUSAL.contains(w)) {
            continue;
        }
        if LIMIT_NOT_A_START.contains(&words[0]) {
            continue;
        }
        if PAST_ENDS.match_(words.last().unwrap()).is_some() {
            continue;
        }
        if ask.starts_with("nothing") && names_a_claim(words.last().unwrap()) {
            continue;
        }
        if words.iter().any(|w| LIMIT_A_FACT.contains(w) || pyuni::is_digit(w)) {
            continue;
        }
        let before = text::strip_chars(&ask[..m.start()], " .,!?");
        if !before.is_empty() && !LIMIT_SOFT_BEFORE.sub("", &format!("{before} ")).is_empty() {
            continue;
        }
        return Some(thing);
    }
    None
}

/// `two_ends_of_one_scale(text)`: "sooner or later", "hot or cold" is not a choice.
pub fn two_ends_of_one_scale(text: &str) -> bool {
    if !(SCALE_ASK.match_(text).is_some() || text.contains("sooner or later")) {
        return false;
    }
    for pair in OPPOSITE_PAIRS.iter() {
        let (one, other) = (&pair[0], &pair[1]);
        for (first, second) in [(one, other), (other, one)] {
            if rx(&format!(r"\b(?:the |a |an )?{first}\s+or\s+(?:the |a |an )?{second}\b")).is_match(text) {
                return true;
            }
        }
    }
    false
}

/// `_turns_the_pair_down(text)`: "neither ... nor".
fn _turns_the_pair_down(text: &str) -> bool {
    let low = text.to_lowercase();
    let head = low.split(" or ").next().unwrap_or("");
    REFUSED_PAIR.is_match(head)
}

/// `_laughed_off(text, cue)`.
fn _laughed_off(text: &str, cue: &str) -> bool {
    cue_rx(cue).finditer(text).iter().any(|m| LAUGHED_OFF_AFTER.match_(text::strip(&text[m.end()..])).is_some())
}

/// `_a_person_they_like(norm)`.
fn _a_person_they_like(norm: &[String]) -> bool {
    norm.iter().enumerate().any(|(k, w)| TASTE_VERBS.contains(w) && norm[k + 1..].iter().any(|x| GENDER_WORDS.contains(x)))
}

/// `_a_thing_they_own(norm)`.
fn _a_thing_they_own(norm: &[String]) -> bool {
    let words = sv(norm);
    if !any_in(&words, &LEARNED_ON) {
        return false;
    }
    !words.iter().any(|w| SELF_ASK.contains(*w) && !LEARNED_ON.contains(*w))
}

/// `agreement_of(text)`: "fair enough", "sounds good" said as the whole line.
pub fn agreement_of(text: &str) -> Option<String> {
    let said = text::squash(&text.to_lowercase().replace(',', " "));
    let first = text::squash(said.split('|').next().unwrap_or(""));
    let flat = text::squash(&said.replace('|', " "));
    AGREEMENT_IDIOMS.iter().find(|p| **p == flat || **p == first).cloned()
}

/// `_has_any(text, phrases)`.
fn _has_any<S: AsRef<str>>(text: &str, phrases: &[S]) -> bool {
    phrases.iter().any(|p| text.contains(p.as_ref()))
}

/// `small_talk_only(norm)`: the line is a small-talk phrase and nothing else ("how are you").
pub fn small_talk_only(norm: &[String]) -> bool {
    if norm.is_empty() || norm.len() > 10 {
        return false;
    }
    for phrase in SMALL_TALK_WORDS.iter() {
        let size = phrase.len();
        if size > norm.len() {
            continue;
        }
        for at in 0..=(norm.len() - size) {
            if norm[at..at + size] != phrase[..] {
                continue;
            }
            if norm[..at].iter().chain(norm[at + size..].iter()).all(|w| SMALL_TALK_LEFTOVER.contains(w)) {
                return true;
            }
        }
    }
    false
}

/// `cue_about_person(text, cue)`: the feeling word is said of a person ("i'm sad", "so sad").
pub fn cue_about_person(text: &str, cue: &str) -> bool {
    if PARTICLE_STATES.contains(cue) {
        for found in cue_rx(cue).finditer(text) {
            if _a_direction(text, found.start()) {
                return false;
            }
        }
    }
    let last = text::split_ref(cue).last().map(|w| w.to_string()).unwrap_or_else(|| panic!("IndexError"));
    if CUE_FOLLOW.contains(&last) {
        return cue_rx(cue).is_match(text);
    }
    for m in cue_rx(cue).finditer(text) {
        if _a_verb_holds_it(text, m.start()) {
            continue;
        }
        let rest = text::lstrip(&text[m.end()..]);
        if rest.is_empty() || ".,!?;:-|".contains(rest.chars().next().unwrap()) {
            return true;
        }
        let following = text::strip_chars(text::split_ref(rest)[0], ".,!?;:-|");
        if following.is_empty() || CUE_FOLLOW.contains(&following.to_lowercase()) {
            return true;
        }
        if following.chars().next().is_some_and(|c| pyuni::ch_upper(c)) && !LX_COMMON.contains(&following.to_lowercase()) {
            return true;
        }
        if STATE_NOUNS.contains(&following.to_lowercase()) && _state_before(text, m.start()) {
            return true;
        }
        if PHRASE_OPENERS.contains(&following.to_lowercase()) && _clause_follows(rest) {
            return true;
        }
        if _speaker_before(text, m.start()) && _verb_after(rest) {
            return true;
        }
    }
    false
}

/// `_a_verb_holds_it(text, at)`: "stop being sad" - a verb before the cue takes it.
fn _a_verb_holds_it(text: &str, at: usize) -> bool {
    let words = stripped_words(&text[..at]);
    let Some(last) = words.last() else { return false };
    last.ends_with("ing") && !FEELING_CUES_BEFORE.contains(last) && !ING_THINGS.contains(last) && !ING_TAKES_NO_OBJECT.contains(last)
}

/// `_state_before(text, at)`.
fn _state_before(text: &str, at: usize) -> bool {
    let words = stripped_words(&text[..at]);
    STATE_TAIL.is_match(&words[words.len().saturating_sub(4)..].join(" "))
}

static RE_IM: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bi'?m\b", 0));
static RE_IVE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bi'?ve\b", 0));

/// `_speaker_before(text, at)`: the words just before `at` (a byte offset) make the speaker the
/// one in the state ("i am so", "i feel", "my mood is").
pub fn _speaker_before(text: &str, at: usize) -> bool {
    let mut words = stripped_words(&text[..at.min(text.len())]);
    while words.last().is_some_and(|w| FEELING_FILLER.contains(w)) {
        words.pop();
    }
    let tail = words[words.len().saturating_sub(5)..].join(" ");
    let tail = RE_IM.sub("i am", &tail);
    let tail = RE_IVE.sub("i have", &tail);
    if SPEAKER_TAIL.is_match(&tail) {
        return true;
    }
    let recent = &words[words.len().saturating_sub(4)..];
    if let Some(my) = recent.iter().position(|w| w == "my") {
        if recent[my + 1..].iter().any(|w| MOOD_NOUNS.contains(w)) {
            return true;
        }
    }
    false
}

/// `_verb_after(rest)`.
fn _verb_after(rest: &str) -> bool {
    for word in text::split_ref(rest).into_iter().take(5) {
        let word = text::strip_chars(word, ".,!?;:-|\"'").to_lowercase();
        if CLAUSE_HELPERS.contains(&word) || CLAUSE_VERBS.contains(&word) {
            return true;
        }
        if text::len(&word) > 3 && (word.ends_with("ed") || word.ends_with("ing")) {
            return true;
        }
    }
    false
}

/// `_clause_follows(rest)`.
fn _clause_follows(rest: &str) -> bool {
    let words: Vec<&str> = text::split_ref(rest).into_iter().map(|w| text::strip_chars(w, ".,!?;:-|'\"")).collect();
    let mut end = words.len();
    for (k, word) in words.iter().enumerate() {
        if word.chars().last().is_some_and(|c| ".,!?;:-|".contains(c)) {
            end = k + 1;
            break;
        }
    }
    let clause: Vec<String> = words[..end.min(8)].iter().filter(|w| !w.is_empty()).map(|w| w.to_lowercase()).collect();
    if clause.is_empty() {
        return false;
    }
    clause[1..].iter().any(|w| CUE_FOLLOW.contains(w) || CLAUSE_VERBS.contains(w) || w.ends_with("ed"))
}

/// `_a_body_part_before(text, cue)`: "my stomach is sick".
fn _a_body_part_before(text: &str, cue: &str) -> bool {
    for m in cue_rx(cue).finditer(text) {
        let before: Vec<String> = stripped_words(&text[..m.start()]).into_iter().filter(|w| !FEELING_FILLER.contains(w) && w != "is").collect();
        if before.last().is_some_and(|w| BODY_SUBJECT.contains(w)) {
            return true;
        }
    }
    false
}

/// `_takes_an_object(text, end)`: the cue at `end` takes an object ("sick of my job").
fn _takes_an_object(text: &str, end: usize) -> bool {
    let words = stripped_words(&text[end..]);
    for (pos, word) in words.iter().enumerate() {
        if word.is_empty() || TAIL_STOPS.contains(word) {
            return false;
        }
        if OBJECT_MARKERS.contains(word) {
            let after = words.get(pos + 1);
            if TIME_MARKERS.contains(word) && after.is_some_and(|a| TIME_FILLERS.contains(a)) {
                return false;
            }
            return true;
        }
    }
    false
}

/// `_object_after(text, cue)`.
fn _object_after(text: &str, cue: &str) -> bool {
    cue_rx(cue).finditer(text).iter().any(|m| _takes_an_object(text, m.end()))
}

/// `_state_cue(cue)`.
fn _state_cue(cue: &str) -> Option<String> {
    if SELF_ONLY_STATE.contains(cue) {
        return Some(cue.to_string());
    }
    let head = text::split_ref(cue).first().map(|w| w.to_string()).unwrap_or_else(|| panic!("IndexError"));
    if SELF_ONLY_STATE.contains(&head) { Some(head) } else { None }
}

/// `_a_copula_before(text, start)`.
fn _a_copula_before(text: &str, start: usize) -> bool {
    let mut words = stripped_words(&text[..start]);
    while words.last().is_some_and(|w| FEELING_FILLER.contains(w) || w == "both" || w == "all") {
        words.pop();
    }
    words.last().is_some_and(|w| COPULA_BEFORE.contains(w))
}

/// `_state_of_a_person(text, cue)`: a state word said of a person ("i'm down", not "down the road").
fn _state_of_a_person(text: &str, cue: &str) -> bool {
    for m in cue_rx(cue).finditer(text) {
        if SUPPLY_WORDS.contains(cue) && _about_a_supply(text, m.end()) {
            continue;
        }
        if VERB_ONLY_STATE.contains(cue) && !_a_copula_before(text, m.start()) {
            continue;
        }
        if _takes_an_object(text, m.end()) {
            continue;
        }
        let mut words = stripped_words(&text[..m.start()]);
        while words.last().is_some_and(|w| STATE_JOINERS.contains(w) || FEELING_FILLER.contains(w)) {
            words.pop();
        }
        let Some(subject) = words.last() else { return true };
        if PERSON_SUBJECT.contains(subject) || OTHER_PEOPLE.contains(subject) {
            return true;
        }
        let before_subject = &words[..words.len() - 1];
        if MOOD_NOUNS.contains(subject) && before_subject[before_subject.len().saturating_sub(3)..].iter().any(|w| w == "my") {
            return true;
        }
        if _speaker_before(text, m.start()) {
            return true;
        }
        return false;
    }
    false
}

/// `_said_of_the_speaker(text, cue)`.
fn _said_of_the_speaker(text: &str, cue: &str) -> bool {
    cue_rx(cue).finditer(text).iter().any(|m| _speaker_before(text, m.start()))
}

/// `_names_an_episode(text, cue)`: "a panic attack".
fn _names_an_episode(text: &str, cue: &str) -> bool {
    rx(&format!(r"\b{}\b\s+(\w+)", escape(cue))).finditer(text).iter().any(|m| EPISODE_NOUNS.contains(&m.str(1).to_lowercase()))
}

/// `a_broken_heart_shape(text)`: "my heart is broken", "heartbroken".
pub fn a_broken_heart_shape(text: &str) -> bool {
    let low = text::squash(&text.to_lowercase().replace('\u{2019}', "'"));
    HEART_IN_A_STATE.is_match(&low) || BROKEN_HEARTED.is_match(&low) || BROKE_A_HEART.is_match(&low) || A_BROKEN_HEART.is_match(&low)
}

/// `_somebody_elses_heart(text)`: "my sister is heartbroken".
fn _somebody_elses_heart(text: &str) -> bool {
    let low = text::squash(&text.to_lowercase().replace('\u{2019}', "'"));
    if !a_broken_heart_shape(&low) || _a_broken_heart_is_theirs(&low) {
        return false;
    }
    if A_POSSESSED_HEART.is_match(&low) {
        return false;
    }
    if A_PERSON_IS_THE_SUBJECT.match_(&low).is_some() {
        return true;
    }
    let words = text::split_ref(&low);
    if words.len() > 1 && matches!(words[0], "my" | "our" | "the") {
        let person = text::strip_chars(words[1], ".,!?'\"");
        let cut = text::slice(person, 0, Some(-1));
        if A_PERSON_ELSE.contains(person) || A_PERSON_ELSE.contains(&cut) {
            if matches!(words.get(2), Some(&"heart") | Some(&"hearts")) {
                return false;
            }
            return true;
        }
    }
    false
}

/// `_a_broken_heart_is_theirs(text)`: the speaker's own broken heart.
fn _a_broken_heart_is_theirs(text: &str) -> bool {
    let low = text::squash(&text.to_lowercase().replace('\u{2019}', "'"));
    if !a_broken_heart_shape(&low) {
        return false;
    }
    let words = text::split_ref(&low);
    let starts: Vec<usize> = [rx(r"\bhearts?\b").search(&low), rx(r"\bbroken\b").search(&low)].into_iter().flatten().map(|m| m.start()).collect();
    let at = match starts.iter().min() {
        Some(&s) => text::split_ref(&low[..s]).len(),
        None => 0,
    };
    let window = &words[at.saturating_sub(2)..at.min(words.len())];
    let owner = window.last().copied().unwrap_or("");
    if HAVE_A_BROKEN_HEART.is_match(&low) {
        return true;
    }
    if BROKE_A_HEART.is_match(&low) {
        return false;
    }
    if NOT_THEIR_OWN.contains(owner) || owner.ends_with("'s") || (owner.ends_with('s') && !matches!(owner, "is" | "was" | "this" | "its" | "us")) {
        return false;
    }
    if window.iter().any(|w| *w == "my" || *w == "our") {
        return true;
    }
    if matches!(owner, "a" | "an" | "the") {
        return false;
    }
    !rx(r"\b(?:her|his|their|your)\b").is_match(&low) && rx(r"^(?:i|im|i'm|ive|i've|we|we're|weve|we've|our)\b").match_(&low).is_some()
}

/// `_a_hedged_low(text)`: "not great tbh", "could be better".
fn _a_hedged_low(text: &str) -> bool {
    let mut words: Vec<String> = text::split_ref(&text.to_lowercase()).into_iter().map(|w| text::strip_chars(w, ".,!?;:-'\"|").to_string()).filter(|w| !w.is_empty()).collect();
    let mut doubted = false;
    for phrase in HEDGED_DOUBT.iter() {
        let count = text::split_ref(phrase).len();
        if words.len() > count && words[words.len() - count..].join(" ") == *phrase {
            words.truncate(words.len() - count);
            doubted = true;
            break;
        }
    }
    while words.first().is_some_and(|w| HEDGED_WRAPPER.contains(w)) {
        words.remove(0);
    }
    while words.last().is_some_and(|w| HEDGED_WRAPPER.contains(w)) {
        words.pop();
    }
    let answer = words.join(" ");
    HEDGED_LOW.contains(&answer) || (doubted && HEDGED_OK.contains(&answer))
}

/// `_copula_just_before(words, pos)`.
fn _copula_just_before(words: &[String], pos: usize) -> bool {
    for word in words[..pos].iter().rev() {
        if FEELING_FILLER.contains(word) {
            continue;
        }
        return FEELING_COPULA.contains(word);
    }
    false
}

/// `_state_as_feeling(text)`: "i'm a mess", "i'm all over the place".
fn _state_as_feeling(text: &str) -> bool {
    let words: Vec<String> = text::split_ref(&text.to_lowercase()).into_iter().map(|w| text::strip_chars(w, ".,!?;:-|\"'").to_string()).collect();
    for (pos, word) in words.iter().enumerate() {
        if !STATE_AS_FEELING.contains(word) {
            continue;
        }
        if _thing_turned_down(&words, pos) || !_copula_just_before(&words, pos) || _particle_is_a_place(&words, pos) {
            continue;
        }
        let after = words.get(pos + 1).map(String::as_str).unwrap_or("");
        if FEELING_NOT_MOOD.contains(after) {
            continue;
        }
        if matches!(_whose_state(&words, pos, None).as_deref(), Some("self") | Some("other")) {
            return true;
        }
    }
    false
}

/// `_works_on_the_speaker(text)`: "it stresses me out" -> stressed.
fn _works_on_the_speaker(text: &str) -> Option<String> {
    let words: Vec<String> = text::split_ref(&text.to_lowercase()).into_iter().map(|w| text::strip_chars(w, ".,!?;:-|\"'()").to_string()).collect();
    for (pos, word) in words.iter().enumerate() {
        let Some(feeling) = WORK_ON_ME.get(word).filter(|f| !f.is_empty()) else { continue };
        let mut lead: Option<&str> = None;
        let lo = pos as isize - 4;
        let mut back = pos as isize - 1;
        while back > lo.max(-1) {
            let w = &words[back as usize];
            if ACT_ON_FILLER.contains(w) || ACT_ON_LEAD.contains(w) {
                back -= 1;
                continue;
            }
            lead = Some(w);
            break;
        }
        if matches!(lead, Some("i" | "i'm" | "im" | "me" | "my" | "myself" | "we" | "our")) {
            continue;
        }
        let named = words[pos.saturating_sub(4)..pos].iter().filter(|w| !ACT_ON_FILLER.contains(*w));
        if named.into_iter().any(|w| PARTS_OF_ME.contains(w)) {
            continue;
        }
        if let Some(l) = lead.filter(|l| !l.is_empty()) {
            if l.ends_with("nt") || matches!(l, "not" | "never" | "no") {
                continue;
            }
        }
        let mut acted_on: Option<&str> = None;
        for ahead in pos + 1..(pos + 4).min(words.len()) {
            if ACT_ON_FILLER.contains(&words[ahead]) {
                continue;
            }
            acted_on = Some(&words[ahead]);
            break;
        }
        if !matches!(acted_on, Some("me" | "us")) {
            continue;
        }
        return Some(feeling.clone());
    }
    None
}

/// `_skeleton(word)`: the consonants of a word ("anxous" and "anxious" share one).
pub fn _skeleton(word: &str) -> String {
    word.to_lowercase().chars().filter(|c| pyuni::ch_alpha(*c) && !"aeiouy".contains(*c)).collect()
}

/// `_sound_cues()`: (skeleton -> feeling -> cues, every one-word cue).
fn _sound_cues() -> &'static (HashMap<String, BTreeMap<String, Vec<String>>>, HashSet<String>) {
    static CUES: LazyLock<(HashMap<String, BTreeMap<String, Vec<String>>>, HashSet<String>)> = LazyLock::new(|| {
        let mut index: HashMap<String, BTreeMap<String, Vec<String>>> = HashMap::new();
        let mut known = HashSet::new();
        for (name, cues) in EMOTION_WORDS.iter() {
            for cue in cues {
                if cue.contains(' ') {
                    continue;
                }
                known.insert(cue.clone());
                if text::len(cue) < 4 {
                    continue;
                }
                index.entry(_skeleton(cue)).or_default().entry(name.clone()).or_default().push(cue.clone());
            }
        }
        (index, known)
    });
    &CUES
}

/// `feeling_by_sound(word)`: the one feeling a misspelt word sounds like ("anxous" -> anxious).
pub fn feeling_by_sound(word: &str) -> Option<String> {
    let low = text::strip_chars(text::strip(word), ".,!?;:'\"()").to_lowercase();
    if text::len(&low) < 4 || !pyuni::is_alpha(&low) {
        return None;
    }
    let (index, known) = _sound_cues();
    if known.contains(&low) || LX_COMMON_NAMES.contains(&low) {
        return None;
    }
    let first: Option<char> = low.chars().next();
    let found: BTreeSet<&String> = match index.get(&_skeleton(&low)) {
        Some(matches) => matches.iter().filter(|(_, cues)| cues.iter().any(|c| c.chars().next() == first)).map(|(k, _)| k).collect(),
        None => BTreeSet::new(),
    };
    if found.len() == 1 { found.into_iter().next().cloned() } else { None }
}

/// `_a_feeling_can_end_here(text, end)`.
fn _a_feeling_can_end_here(text: &str, end: usize) -> bool {
    let rest = text::lstrip(&text[end..]);
    if rest.is_empty() || ".,!?;:".contains(rest.chars().next().unwrap()) {
        return true;
    }
    SOUND_FOLLOWS.contains(text::strip_chars(text::split_ref(rest)[0], ".,!?;:'\""))
}

/// `_feeling_said_as_a_sound(text)`: "i feel anxous" -> anxious.
fn _feeling_said_as_a_sound(text: &str) -> Option<String> {
    for m in FEELING_SOUND_FRAME.finditer(text) {
        if _takes_an_object(text, m.end()) || !_a_feeling_can_end_here(text, m.end()) {
            continue;
        }
        if let Some(name) = feeling_by_sound(m.str(1)) {
            return Some(name);
        }
    }
    None
}

/// `_a_period_of_their_own(text, cue)`: "my low days", "a bad week".
fn _a_period_of_their_own(text: &str, cue: &str) -> bool {
    for m in cue_rx(cue).finditer(text) {
        let mut after = stripped_words(&text[m.end()..]);
        let mut seen = 0;
        while !after.is_empty() && seen < 3 && OWN_PERIOD_FILLER.contains(&after[0]) {
            after.remove(0);
            seen += 1;
        }
        if after.is_empty() || !OWN_PERIOD_NOUNS.contains(&after[0]) {
            continue;
        }
        let before: Vec<String> = text::split_ref(&text[..m.start()]).into_iter().map(|w| text::strip_chars(w, ".,!?;:-|\"'()").to_lowercase()).collect();
        let reach = OWN_PERIOD_REACH as usize;
        for word in before[before.len().saturating_sub(reach)..].iter().rev() {
            if OWN_PERIOD_STOPS.contains(word) {
                break;
            }
            if OWN_PERIOD_HEADS.contains(word) {
                return true;
            }
        }
    }
    false
}

/// `_acts_on_the_speaker(text)`: "it drives me mad" -> the feeling it names.
fn _acts_on_the_speaker(text: &str) -> Option<String> {
    if LAUGH_AT_THE_END.is_match(text) {
        return None;
    }
    for pair in ACTS_ON_THE_SPEAKER.iter() {
        let (pattern, name) = (&pair[0], &pair[1]);
        for m in rx(&format!(r"\b(?:{pattern})")).finditer(text) {
            let before = text::split_ref(&text[..m.start()]);
            let before = &before[before.len().saturating_sub(2)..];
            if before.iter().any(|w| A_BODY_SUBJECT.contains(text::strip_chars(w, ".,!?;:'\""))) {
                continue;
            }
            return Some(name.clone());
        }
    }
    None
}

/// `emotion_of(text)`: the feeling the line says the speaker has, or None.
pub fn emotion_of(text: &str) -> Option<String> {
    let text = BETTER_OFF.sub(" ", text);
    let text = NO_BETTER.sub("worse", &text);
    let text = GETTING_WORSE.sub_fn(&text, |m| format!("{} worse", m.str(1)));
    let text = IDIOM_STATES.sub(" ", &text);
    let text = ABILITY_AT.sub(" ", &text);
    let text = text.as_str();
    for (name, cues) in EMOTION_WORDS.iter() {
        for cue in cues {
            if let Some(state) = _state_cue(cue) {
                if !_state_of_a_person(text, &state) {
                    continue;
                }
            }
            if BODY_STATE_CUES.contains(cue) && (_a_body_part_before(text, cue) || _object_after(text, cue)) {
                continue;
            }
            if let Some(idiom) = NOT_A_MOOD_THERE.get(cue) {
                if idiom.is_match(text) {
                    continue;
                }
            }
            if LABEL_ON_A_PERSON.contains(cue) && _said_of_the_speaker(text, cue) {
                continue;
            }
            if (SELF_ONLY_CUES.contains(cue) || ALWAYS_SELF_ONLY.contains(cue) || THING_OR_PERSON_CUES.contains(cue) || CONFUSED_SELF_ONLY.contains(cue) || A_COMPARISON.contains(cue))
                && !_said_of_the_speaker(text, cue)
            {
                continue;
            }
            if THING_OR_PERSON_CUES.contains(cue) && _laughed_off(text, cue) {
                continue;
            }
            if PERIOD_CUES.contains(cue) && A_PERIOD.is_match(text) && !A_LIMIT_ABOUT_PEOPLE.is_match(text) && (A_MONTH_PERIOD.is_match(text) || A_MONEY_SUBJECT.is_match(text)) {
                continue;
            }
            if SAID_AS_A_NEGATION.contains(cue) && !denied(text, cue) {
                continue;
            }
            if !GOOD_EMOTION_NAMES.contains(name) && !SAID_AS_A_NEGATION.contains(cue) && denied(text, cue) {
                continue;
            }
            if GOOD_EMOTION_NAMES.contains(name) && GOOD_WORD_MISSING.is_match(text) && !GOOD_WORD_COMPARED.is_match(text) && !GOOD_WORD_BY_COMPARISON.is_match(text) {
                return Some("sad".to_string());
            }
            if cue_about_person(text, cue) || _names_an_episode(text, cue) {
                return Some(name.clone());
            }
        }
    }
    if let Some(acted) = _acts_on_the_speaker(text) {
        return Some(acted);
    }
    if a_broken_heart_shape(text) {
        return Some("sad".to_string());
    }
    if let Some(sounded) = _feeling_said_as_a_sound(text) {
        return Some(sounded);
    }
    if let Some(worked) = _works_on_the_speaker(text) {
        return Some(worked);
    }
    if _state_as_feeling(text) {
        return Some("sad".to_string());
    }
    for (name, cues) in EMOTION_WORDS.iter() {
        if GOOD_EMOTION_NAMES.contains(name) {
            continue;
        }
        for cue in cues {
            if NOT_THE_SPEAKERS_DAY.contains(cue) || PERIOD_CUES.contains(cue) {
                continue;
            }
            if denied(text, cue) {
                continue;
            }
            if _a_period_of_their_own(text, cue) {
                return Some(name.clone());
            }
        }
    }
    None
}

/// `_capability_question(norm)`: "can you help with homework".
fn _capability_question(norm: &[String]) -> bool {
    let compact: Vec<&String> = norm.iter().filter(|w| !CAPABILITY_FILLER.contains(*w)).collect();
    for phrase in CAPABILITY.iter() {
        let wanted: Vec<&str> = text::split_ref(phrase);
        if wanted.len() > compact.len() {
            continue;
        }
        for start in 0..=(compact.len() - wanted.len()) {
            if !compact[start..start + wanted.len()].iter().map(|s| s.as_str()).eq(wanted.iter().copied()) {
                continue;
            }
            let tail = &compact[start + wanted.len()..];
            if tail.iter().any(|w| !CAPABILITY_CARRIER.contains(*w)) {
                return false;
            }
            if tail.iter().enumerate().any(|(k, w)| REFERRING.contains(*w) && k + 1 < tail.len() && POINTED_AT.contains(tail[k + 1])) {
                return false;
            }
            return true;
        }
    }
    false
}

/// `_asks_how_the_bot_sees_me(norm)`: "what do you think of me".
fn _asks_how_the_bot_sees_me(norm: &[String]) -> bool {
    let mut words: Vec<String> = norm.to_vec();
    let mut changed = true;
    while changed && !words.is_empty() {
        changed = false;
        for phrase in MOOD_TAIL.iter() {
            let p = text::split(phrase);
            let size = p.len();
            if size < words.len() && words[words.len() - size..] == p[..] {
                let k = words.len() - size;
                words.truncate(k);
                changed = true;
            }
        }
        while words.first().is_some_and(|w| ME_LEAD.contains(w)) {
            words.remove(0);
            changed = true;
        }
        while words.last().is_some_and(|w| ME_TAIL.contains(w)) {
            words.pop();
            changed = true;
        }
    }
    let ask: Vec<String> = if words.last().is_some_and(|w| w == "my") {
        let mut a = words[..words.len() - 1].to_vec();
        a.push("me".to_string());
        a
    } else {
        words.clone()
    };
    ME_ASKS.contains(&words.join(" ")) || ME_ASKS.contains(&ask.join(" "))
}

/// `_asks_who_this_is(norm)`: "who is this", "am i talking to a bot".
fn _asks_who_this_is(norm: &[String]) -> bool {
    let mut words: Vec<&str> = norm.iter().map(String::as_str).collect();
    while words.last().is_some_and(|w| WHO_TAIL.contains(*w)) {
        words.pop();
    }
    let w = words.as_slice();
    const WHOLE: &[&[&str]] = &[
        &["who", "is", "this"],
        &["who", "am", "i", "talking", "to"],
        &["who", "am", "i", "speaking", "to"],
        &["who", "am", "i", "speaking", "with"],
        &["who", "am", "i", "chatting", "to"],
        &["who", "am", "i", "chatting", "with"],
        &["who", "am", "i", "talking", "with"],
        &["who", "was", "i", "talking", "to"],
        &["who", "was", "i", "speaking", "to"],
        &["who", "was", "i", "speaking", "with"],
    ];
    if WHOLE.iter().any(|p| *p == w) {
        return true;
    }
    if w.len() >= 4 && w[..4] == ["who", "have", "i", "been"] && (w.len() == 6 || w.len() == 7) {
        return WHO_TALKING.contains(w[4]) && matches!(w[5], "to" | "with");
    }
    if w.len() >= 3 && (w[..3] == ["who", "did", "i"] || w[..3] == ["who", "was", "i"] || w[..3] == ["who", "am", "i"]) && (w.len() == 5 || w.len() == 6) {
        if w.len() == 6 && w[3] != "just" {
            return false;
        }
        let at = if w.len() == 6 { 1 } else { 0 };
        return WHO_TALKING.contains(w[3 + at]) && matches!(w[4 + at], "to" | "with");
    }
    if w.len() >= 3 && (w[..3] == ["am", "i", "talking"] || w[..3] == ["am", "i", "speaking"] || w[..3] == ["am", "i", "chatting"]) {
        if w.len() == 5 && matches!(w[3], "to" | "with") {
            return WHO_A_THING.contains(w[4]) || matches!(w[4], "a" | "an");
        }
        if w.len() == 6 && matches!(w[3], "to" | "with") && matches!(w[4], "a" | "an") {
            return WHO_A_THING.contains(w[5]);
        }
    }
    if w.len() >= 2 && (w[..2] == ["is", "this"] || w[..2] == ["is", "it"]) && (3..=5).contains(&w.len()) {
        let mut rest = &w[2..];
        if rest.first().is_some_and(|r| *r == "a" || *r == "an") {
            rest = &rest[1..];
        }
        if rest.first() == Some(&"real") {
            rest = &rest[1..];
        }
        return !rest.is_empty() && rest.iter().all(|r| WHO_A_THING.contains(*r));
    }
    false
}

/// `_asks_where_the_bot_is(norm)`: "where are you", "where do you live".
fn _asks_where_the_bot_is(norm: &[String]) -> bool {
    let words: Vec<&str> = norm.iter().map(String::as_str).collect();
    if words.is_empty() || !matches!(words[0], "where" | "whereabouts") {
        return false;
    }
    let rest = &words[1..];
    let rest = if rest.len() >= 2 && rest[..2] == ["are", "you"] {
        &rest[2..]
    } else if rest.len() >= 3 && rest[..2] == ["do", "you"] && rest[2] == "live" {
        &rest[3..]
    } else if rest.len() >= 2 && rest[..2] == ["you", "live"] {
        &rest[2..]
    } else {
        return false;
    };
    rest.iter().all(|w| WHERE_BOT_TAIL.contains(*w))
}

/// `a_stop_ask(line)`: "can we stop", "stop talking".
pub fn a_stop_ask(line: &str) -> bool {
    let low = line.to_lowercase().replace(',', " ");
    let mut words: Vec<&str> = text::split_ref(&low).into_iter().filter(|w| !w.is_empty() && *w != " ").collect();
    while words.first().is_some_and(|w| STOP_ASK_FILLER.contains(text::strip_chars(w, ".,!?"))) {
        words.remove(0);
    }
    STOP_ASK.fullmatch(text::strip_chars(&words.join(" "), " .!?")).is_some()
}

/// `_about_bot_kind(text, words, norm)`: which question about the bot itself the line asks.
fn _about_bot_kind(text_: &str, words: &HashSet<&str>, norm: &[String]) -> Option<&'static str> {
    let bare = format!(" {} ", text::strip(text_));
    let flat = text_.replace(" | ", " ").replace('|', " ");
    let joined = norm.join(" ");
    if WHILE_ASK.is_match(&flat) {
        return Some("while");
    }
    if KEEPING_ASK.is_match(&flat) || a_stop_ask(&flat) {
        return Some("keeping");
    }
    if _asks_where_the_bot_is(norm) {
        return Some("where");
    }
    if _asks_who_this_is(norm) {
        return Some("identity");
    }
    if ANSWERING_A_QUOTED_ASK.is_match(text_) && !_self_ask(norm) {
        return None;
    }
    if _asks_how_the_bot_sees_me(norm) {
        return Some("warmth");
    }
    if words.contains("remember") && any_of(words, &["you", "your"]) {
        return Some("memory");
    }
    if _capability_question(norm) {
        return Some("help");
    }
    if _asks_if_the_bot_is_a_product(norm) {
        return Some("model");
    }
    if _recalls_their_own_line(norm) {
        return None;
    }
    if _asks_what_is_new_with_the_bot(norm) || _asks_what_the_bot_has_been_doing(norm) || _asks_whether_the_bot_is_about(norm) || _asks_how_it_spends_its_time(norm) {
        return Some("doings");
    }
    if _asks_what_the_bot_is_doing(norm) && any_of(words, &["you", "u", "ya"]) && any_in(words, &ASKS_IF_ANYTHING) {
        return Some("doings");
    }
    if _asks_about_its_experience(norm) {
        return Some("doings");
    }
    if _self_ask(norm) {
        return Some(if any_of(words, &["remember", "recall", "forget"]) { "memory" } else { "identity" });
    }
    if _how_long_itself(norm) || _when_itself(norm) {
        return Some("memory");
    }
    if _counts_its_own_doings(norm) {
        return Some("memory");
    }
    if A_YOUR_KIN.is_match(&joined) || A_YOUR_OWN_THING.is_match(&joined) {
        return Some("life");
    }
    if A_HOW_MANY_OF_THE_BOTS_OWN.is_match(&joined) && norm.iter().any(|w| BOT_LIFE.contains(w)) {
        return Some("life");
    }
    if A_WHAT_ITS_LIKE.is_match(&joined) {
        return Some("identity");
    }
    if _asks_about_its_own_life(norm) {
        return Some("life");
    }
    if _has_any(&bare, &ABOUT_BOT) || text::ends_with_any(text::strip(text_), &ABOUT_BOT_END) {
        return Some("identity");
    }
    if any_of(words, &["you", "your"]) && any_in(words, &ROBOT_WORDS) && words.contains("are") {
        return Some("identity");
    }
    None
}

/// `_without_mood_tail(clause)`: "i am sad today though" -> without "today though".
fn _without_mood_tail(clause: &[String]) -> Vec<String> {
    let mut words = clause.to_vec();
    let mut changed = true;
    while changed && !words.is_empty() {
        changed = false;
        if MOOD_TAIL_WORDS.contains(words.last().unwrap()) {
            words.pop();
            changed = true;
            continue;
        }
        for phrase in MOOD_TAIL.iter() {
            let p = text::split(phrase);
            let size = p.len();
            if size >= words.len() || words[words.len() - size..] != p[..] {
                continue;
            }
            if MOOD_TAIL_TAKEN.contains(&words[words.len() - size - 1]) {
                continue;
            }
            let k = words.len() - size;
            words.truncate(k);
            changed = true;
            break;
        }
    }
    words
}

/// `_last_subject_pos(words)`.
fn _last_subject_pos(words: &[String]) -> Option<usize> {
    for pos in (0..words.len()).rev() {
        let word = &words[pos];
        if BEFORE_STATE.contains(word) {
            continue;
        }
        if text::len(word) > 4 && (word.ends_with("ly") || word.ends_with("ing")) {
            continue;
        }
        if pyuni::is_digit(&word.replace('.', "")) || NUMBER_WORDS.contains(word) {
            continue;
        }
        return Some(pos);
    }
    None
}

/// `_last_subject(words)`.
fn _last_subject(words: &[String]) -> Option<String> {
    _last_subject_pos(words).map(|p| words[p].clone())
}

/// `_particle_is_a_place(words, pos)`: "down the road".
fn _particle_is_a_place(words: &[String], pos: usize) -> bool {
    if pos > 0 && MOTION_WORDS.contains(&words[pos - 1]) {
        return true;
    }
    if words.len() < pos + 2 || !DETERMINERS.contains(&words[pos + 1]) {
        return false;
    }
    !words[(pos + 2).min(words.len())..(pos + 5).min(words.len())].iter().any(|w| TIME_AFTER_A_PARTICLE.contains(w))
}

/// `_a_direction(text, at)`.
fn _a_direction(text_: &str, at: usize) -> bool {
    let words: Vec<String> = text::split_ref(&text_.to_lowercase()).into_iter().map(|w| text::strip_chars(w, ".,!?;:-|\"'").to_string()).collect();
    let pos = text::split_ref(&text_[..at].to_lowercase()).len();
    pos < words.len() && _particle_is_a_place(&words, pos)
}

/// `_copula_before(words, pos)`.
fn _copula_before(words: &[String], pos: usize) -> bool {
    for word in words[..pos].iter().rev() {
        if COPULA.contains(word) {
            return true;
        }
        if BEFORE_STATE.contains(word) {
            continue;
        }
        return false;
    }
    true
}

/// `_noun_reading(words, pos)`.
fn _noun_reading(words: &[String], pos: usize) -> bool {
    if pos > 0 && DEGREE.contains(&words[pos - 1]) {
        return false;
    }
    words[pos.saturating_sub(2)..pos].iter().any(|w| DETERMINERS.contains(w))
}

/// `_thing_turned_down(words, pos)`.
fn _thing_turned_down(words: &[String], pos: usize) -> bool {
    if pos < 2 || !DETERMINERS.contains(&words[pos - 2]) {
        return false;
    }
    pos == 2 || MANIPULATE.contains(&words[pos - 3])
}

/// `_is_person(word)`.
fn _is_person(word: &str) -> bool {
    PERSON_SUBJECT.contains(word) || PERSON_NOUNS.contains(word) || MOOD_NOUNS.contains(word) || OTHER_PEOPLE.contains(word) || matches!(word, "that" | "this" | "it")
}

/// `_names_a_thing(words, at)`.
fn _names_a_thing(words: &[String], at: usize) -> bool {
    if EVENT_NOUNS.contains(&words[at]) {
        return false;
    }
    words[at.saturating_sub(3)..at].iter().any(|w| DETERMINERS.contains(w))
}

/// `_names_somebody_else(words, at)`: "my 5 year old is ...".
fn _names_somebody_else(words: &[String], at: usize) -> bool {
    words[at] == "old" && words[at.saturating_sub(3)..at].iter().any(|w| w == "year")
}

/// `_whose_state(words, pos, head=None)`: "self", "other" or "thing": whose state the word is.
fn _whose_state(words: &[String], pos: usize, head: Option<usize>) -> Option<String> {
    let following = &words[(pos + 1).min(words.len())..];
    let mut subject_at = _last_subject_pos(&words[..head.unwrap_or(pos)]);
    if let Some(s) = subject_at {
        if RELATIVE_SUBJECTS.contains(&words[s]) {
            if let Some(earlier) = _last_subject_pos(&words[..s]) {
                subject_at = Some(earlier);
            }
        }
    }
    let subject = subject_at.map(|s| words[s].as_str());
    let r = |s: &str| Some(s.to_string());
    if let (Some(subject), Some(sa)) = (subject, subject_at) {
        if _copula_before(words, pos) {
            if OTHER_PEOPLE.contains(subject) || _names_somebody_else(words, sa) {
                return r("other");
            }
            if _is_person(subject) {
                return r("self");
            }
            if _names_a_thing(words, sa) {
                return r("thing");
            }
            return r("self");
        }
    }
    if following.first().is_some_and(|f| PERSON_PREPOSITIONS.contains(f)) {
        if has(&OTHER_PEOPLE, subject) {
            return r("other");
        }
        if subject.is_some_and(|s| !_is_person(s)) {
            return r("thing");
        }
        return r("self");
    }
    for word in following {
        if CONJUNCTIONS.contains(word) {
            break;
        }
        if BETWEEN_STATE_AND_NOUN.contains(word) || TIME_FILLERS.contains(word) {
            continue;
        }
        if word.ends_with("ly") && text::len(word) > 4 {
            continue;
        }
        if DETERMINERS.contains(word) {
            continue;
        }
        if word.ends_with("ing") && text::len(word) > 4 {
            return r("self");
        }
        if OTHER_PEOPLE.contains(word) {
            return r("other");
        }
        if PERSON_NOUNS.contains(word) || MOOD_NOUNS.contains(word) {
            return r("self");
        }
        return r("thing");
    }
    let Some(subject) = subject else { return r("self") };
    if OTHER_PEOPLE.contains(subject) {
        return r("other");
    }
    if PERSON_SUBJECT.contains(subject) || PERSON_NOUNS.contains(subject) || MOOD_NOUNS.contains(subject) || matches!(subject, "that" | "this" | "it") {
        return r("self");
    }
    if _names_a_thing(words, subject_at.unwrap()) {
        return r("thing");
    }
    r("self")
}

/// `_echoed_state(words)`: "me too", "so is she".
fn _echoed_state(words: &[String]) -> Option<String> {
    if words.is_empty() || words.len() > 6 {
        return None;
    }
    let w: Vec<&str> = words.iter().map(String::as_str).collect();
    let carried = (w[0] == "so" && w.len() > 1 && BE_FORMS.contains(w[1]))
        || w.last() == Some(&"too")
        || (w.len() >= 2 && w[w.len() - 2..] == ["as", "well"])
        || w.last() == Some(&"also")
        || (w.len() >= 2 && w[..2] == ["same", "here"])
        || (w.len() == 2 && PERSON_SUBJECT.contains(w[0]) && BE_FORMS.contains(w[1]));
    if !carried {
        return None;
    }
    if w.len() >= 2 && (w[..2] == ["same", "here"] || w[..2] == ["me", "too"] || w[..2] == ["same", "same"]) {
        return Some("self".to_string());
    }
    if w.iter().any(|x| PERSON_SUBJECT.contains(*x)) {
        return Some("self".to_string());
    }
    if w.iter().any(|x| OTHER_PEOPLE.contains(*x)) {
        return Some("other".to_string());
    }
    None
}

/// `_carried_over(words, state_at)`.
fn _carried_over(words: &[String], state_at: Option<usize>) -> Option<String> {
    let mut tail = match state_at {
        Some(s) => &words[(s + 1).min(words.len())..],
        None => words,
    };
    while tail.first().is_some_and(|w| CONJUNCTIONS.contains(w)) {
        tail = &tail[1..];
    }
    _echoed_state(tail)
}

/// `_state_owner(clauses)`: whose state the line is about ("self", "other", "thing").
fn _state_owner(clauses: &[Vec<String>]) -> Option<String> {
    let mut owner: Option<String> = None;
    let mut carried: Option<String> = None;
    let whole = clauses.iter().map(|c| c.join(" ")).collect::<Vec<_>>().join(" ");
    if _works_on_the_speaker(&whole).is_some() {
        return Some("self".to_string());
    }
    for clause in clauses {
        let words = _without_mood_tail(clause);
        if owner.is_some() && !words.iter().any(|w| STATE_WORDS.contains(w)) {
            let echo = _echoed_state(&words);
            if echo.as_deref() == Some("self") {
                return Some("self".to_string());
            }
            if echo.is_some() {
                owner = echo;
            }
        }
        let mut state_at: Option<usize> = None;
        for (index, word) in words.iter().enumerate() {
            let mut pos = index;
            if !STATE_WORDS.contains(word) {
                continue;
            }
            state_at = Some(pos);
            if NOUN_STATES.contains(word) && _noun_reading(&words, pos) {
                continue;
            }
            if PARTICLE_STATES.contains(word) {
                if _thing_turned_down(&words, pos) {
                    owner = Some("thing".to_string());
                    continue;
                }
                if !_copula_before(&words, pos) {
                    continue;
                }
            }
            if COPULA_STATES.contains(word) && (pos == 0 || !_copula_before(&words, pos)) {
                continue;
            }
            let mut head = None;
            if word == "fed" {
                if words.get(pos + 1).map(String::as_str) != Some("up") {
                    continue;
                }
                head = Some(pos);
                pos += 1;
            }
            let mut whose = _whose_state(&words, pos, head);
            if whose.is_none() && carried.as_ref().is_some_and(|c| PERSON_SUBJECT.contains(c) || PERSON_NOUNS.contains(c)) {
                whose = Some("self".to_string());
            }
            if whose.as_deref() == Some("self") {
                return Some("self".to_string());
            }
            if whose.is_some() {
                owner = whose;
            }
        }
        if owner.is_some() {
            let echo = _carried_over(&words, state_at);
            if echo.as_deref() == Some("self") {
                return Some("self".to_string());
            }
            if echo.is_some() {
                owner = echo;
            }
        }
        carried = _last_subject(&words).filter(|s| !s.is_empty()).or(carried);
    }
    owner
}

/// `_about_a_supply(text, end)`: "low on milk".
fn _about_a_supply(text_: &str, end: usize) -> bool {
    let rest = stripped_words(&text_[end..]);
    if rest.len() < 2 || !SUPPLY_PREPOSITIONS.contains(&rest[0]) {
        return false;
    }
    let thing = &rest[1];
    !OTHER_PEOPLE.contains(thing) && !PERSON_NOUNS.contains(thing) && !PERSON_SUBJECT.contains(thing)
}

/// `shortage_of(text)`: "we're out of milk" -> milk.
pub fn shortage_of(text_: &str) -> Option<String> {
    let words = stripped_words(text_);
    for (k, word) in words.iter().enumerate() {
        if !SUPPLY_WORDS.contains(word) || k + 2 >= words.len() {
            continue;
        }
        if !SUPPLY_PREPOSITIONS.contains(&words[k + 1]) {
            continue;
        }
        let thing = &words[k + 2];
        if DETERMINERS.contains(thing) || OWN_RESERVES.contains(thing) || OTHER_PEOPLE.contains(thing) || PERSON_NOUNS.contains(thing) || PERSON_SUBJECT.contains(thing) {
            continue;
        }
        return Some(thing.clone());
    }
    let out = rx(r"\b(?:run|ran|running) out of ([a-z]+)").search(text_)?;
    let g = out.str(1);
    if !OWN_RESERVES.contains(g) && !DETERMINERS.contains(g) { Some(g.to_string()) } else { None }
}

/// `_how_long_itself(norm)`: "how long have you been around".
fn _how_long_itself(norm: &[String]) -> bool {
    let said = norm.join(" ");
    let Some(m) = ITSELF_LONG.match_(&said) else { return false };
    text::split_ref(&said[m.end()..]).iter().all(|w| ITSELF_LIFE.contains(*w))
}

/// `_when_itself(norm)`.
fn _when_itself(norm: &[String]) -> bool {
    let said = norm.join(" ");
    let Some(m) = ITSELF_WHEN.match_(&said) else { return false };
    text::split_ref(&said[m.end()..]).iter().all(|w| ITSELF_WHEN_TAIL.contains(*w))
}

/// `_counts_its_own_doings(norm)`: "how many people have you talked to today".
fn _counts_its_own_doings(norm: &[String]) -> bool {
    let mut at = 0;
    while at < norm.len() && COUNT_OPENERS.contains(&norm[at]) {
        at += 1;
    }
    let pair: Vec<&str> = norm[at.min(norm.len())..(at + 2).min(norm.len())].iter().map(String::as_str).collect();
    if pair != ["how", "many"] && pair != ["how", "much"] {
        return false;
    }
    let rest = &norm[at + 2..];
    let rs = sv(rest);
    if !any_in(&rs, &BOT_COUNTED) || !any_of(&rs, &["you", "u", "ya", "your", "ur"]) {
        return false;
    }
    rest.iter().enumerate().any(|(k, w)| (BOT_DOINGS.contains(w) || (BOT_HAVE.contains(w) && _a_count_of_chats(rest))) && _the_doer_is_the_bot(rest, k))
}

/// `_a_count_of_chats(rest)`.
fn _a_count_of_chats(rest: &[String]) -> bool {
    rest.iter().any(|w| BOT_CHATS.contains(w))
}

/// `_the_doer_is_the_bot(rest, doing)`.
fn _the_doer_is_the_bot(rest: &[String], doing: usize) -> bool {
    let mut at = doing as isize - 1;
    while at >= 0 && (COUNT_FILLERS.contains(&rest[at as usize]) || rest[at as usize].ends_with("ly")) {
        at -= 1;
    }
    at >= 0 && BOT_DOERS.contains(&rest[at as usize])
}

/// `_asks_about_its_experience(norm)`: "have you ever been to paris".
fn _asks_about_its_experience(norm: &[String]) -> bool {
    if norm.is_empty() {
        return false;
    }
    let said = norm.join(" ");
    let Some(m) = EXPERIENCE_ASK.match_(&said) else { return false };
    let rest = text::split_ref(&said[m.end()..]);
    if rest.is_empty() {
        return false;
    }
    if rest.iter().any(|w| THEIR_PROBLEM.contains(*w)) {
        return false;
    }
    rest.iter().all(|w| EXPERIENCE_WORDS.contains(*w) || BOT_LIFE.contains(*w) || BOT_LIFE_FILLER.contains(*w))
}

/// `_asks_about_its_own_life(norm)`: "do you have a family".
fn _asks_about_its_own_life(norm: &[String]) -> bool {
    if norm.is_empty() || !any_of(&sv(norm), &["you", "u", "ya", "your", "ur", "yours", "yourself"]) {
        return false;
    }
    let joined = norm.join(" ");
    let Some(m) = BOT_LIFE_ASK.match_(&joined) else { return false };
    let rest = text::split_ref(&joined[m.end()..]);
    if rest.is_empty() || rest.iter().any(|w| !BOT_LIFE.contains(*w) && !BOT_LIFE_FILLER.contains(*w)) {
        return false;
    }
    rest.iter().any(|w| BOT_LIFE.contains(*w))
}

/// `_the_bots_own_not(norm, i)`: "you're not a person" - the not is about the bot.
fn _the_bots_own_not(norm: &[String], i: usize) -> bool {
    if i + 1 < norm.len() && NOT_BEFORE_THE_BOT.contains(&norm[i + 1]) {
        return true;
    }
    if i >= 1 && NOT_AFTER_THE_BOT.contains(&norm[i - 1]) {
        return true;
    }
    i >= 2 && NOT_BEFORE_THE_BOT.contains(&norm[i - 2]) && NOT_AUX.contains(&norm[i - 1]) && (i < 3 || !CLAUSE_OPENERS.contains(&norm[i - 3]))
}

/// `_recalls_their_own_line(norm)`: "what did i say".
fn _recalls_their_own_line(norm: &[String]) -> bool {
    let said: Vec<&str> = norm.iter().map(String::as_str).filter(|w| !w.is_empty()).collect();
    _I_SAID_IT.match_(&said.join(" ")).is_some()
}

/// `_self_ask(norm)`: "what are you", "tell me about yourself".
fn _self_ask(norm: &[String]) -> bool {
    let words = sv(norm);
    if words.is_empty() || !any_of(&words, &["you", "your", "yourself", "yours"]) {
        return false;
    }
    if !any_in(&words, &SELF_ASK) {
        return false;
    }
    if _a_person_they_like(norm) || _a_thing_they_own(norm) {
        return false;
    }
    if !norm.is_empty() && JOIN_OPENERS.contains(&norm[0]) && any_in(&words, &MAKER_WORDS) {
        return false;
    }
    if (0..norm.len().saturating_sub(1)).any(|k| POSSESSIVE_WORDS.contains(&norm[k]) && A_PERSON_AFTER_A_POSSESSIVE.contains(&norm[k + 1])) {
        return false;
    }
    if (0..norm.len().saturating_sub(1)).any(|k| LEARNING_WORDS.contains(&norm[k]) && norm[k + 1] == "from") {
        return false;
    }
    if any_in(&words, &A_HAVING) && any_in(&words, &CHILD_NOUNS) {
        return false;
    }
    let mut left: HashSet<&str> = words.iter().copied().filter(|w| !SELF_WORDS.contains(*w) && !SELF_FILLER.contains(*w) && !matches!(*w, "you" | "your" | "yourself" | "yours")).collect();
    if words.contains("not") {
        if (1..norm.len()).any(|k| norm[k] == "not" && !_the_bots_own_not(norm, k)) {
            return false;
        }
        left.remove("not");
    }
    left.is_empty()
}

/// `_about_bot(text, words, norm)`.
fn _about_bot(text_: &str, words: &HashSet<&str>, norm: &[String]) -> bool {
    _about_bot_kind(text_, words, norm).is_some()
}

/// `_asks_if_the_bot_is_a_product(norm)`: "are you chatgpt".
fn _asks_if_the_bot_is_a_product(norm: &[String]) -> bool {
    if norm.len() < 2 || norm.len() as i64 > PRODUCT_ASK_WORDS || !matches!(norm[0].as_str(), "are" | "is") {
        return false;
    }
    if norm.iter().any(|w| ROBOT_WORDS.contains(w)) {
        return false;
    }
    let mut tail: Vec<&String> = norm.iter().collect();
    while tail.last().is_some_and(|w| PRODUCT_ASK_TAIL.contains(*w)) {
        tail.pop();
    }
    tail.len() > 1 && ASSISTANT_NAMES.contains(*tail.last().unwrap())
}

/// `_starts_with(words, phrases)`.
fn _starts_with<S: AsRef<str>>(words: &[String], phrases: &[S]) -> bool {
    let bare = words.join(" ");
    phrases.iter().any(|p| {
        let p = p.as_ref();
        bare == p || bare.starts_with(&format!("{p} "))
    })
}

/// `without_filler(norm)`: "oh thanks" -> "thanks".
pub fn without_filler(norm: &[String]) -> &[String] {
    if norm.len() > 1 && BEFORE_COURTESY.contains(&norm[0]) { &norm[1..] } else { norm }
}

/// `_a_thing_named_after_the_courtesy(norm)`.
pub fn _a_thing_named_after_the_courtesy(norm: &[String]) -> bool {
    _after_thanks(norm).is_some_and(|rest| !rest.is_empty() && COURTESY_THING.contains(&rest[0]))
}

/// THANKS sorted by length, longest first (Python's stable `sorted(key=len, reverse=True)`).
static THANKS_LONGEST: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| longest_first(&THANKS));
static GREETINGS_LONGEST: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| longest_first(&GREETINGS));
static FAREWELLS_LONGEST: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| longest_first(&FAREWELLS));
fn longest_first(phrases: &[String]) -> Vec<Vec<String>> {
    let mut v: Vec<&String> = phrases.iter().collect();
    v.sort_by(|a, b| text::len(b).cmp(&text::len(a)));
    v.into_iter().map(|p| text::split(p)).collect()
}
fn starts_with_words<'a>(norm: &'a [String], words: &[String]) -> Option<&'a [String]> {
    if norm.len() >= words.len() && norm[..words.len()] == words[..] { Some(&norm[words.len()..]) } else { None }
}

/// `_after_thanks(norm)`: the words after a thank-you (None when it does not open the line).
fn _after_thanks(norm: &[String]) -> Option<Vec<String>> {
    for words in THANKS_LONGEST.iter() {
        if let Some(rest) = starts_with_words(norm, words) {
            if !rest.is_empty() && COURTESY_THING.contains(&rest[0]) {
                return None;
            }
            return Some(rest.to_vec());
        }
    }
    None
}

/// `the_line_after_a_courtesy(norm)`: the words after a thank-you (empty when none).
pub fn the_line_after_a_courtesy(norm: &[String]) -> Vec<String> {
    _after_thanks(without_filler(norm)).unwrap_or_default()
}

/// `_a_fall(rest)`: the person fell.
fn _a_fall(rest: &[String]) -> bool {
    for (index, word) in rest.iter().enumerate() {
        if FALL.contains(word) || word.starts_with("collaps") {
            return true;
        }
        if word == "fell" {
            let after = rest.get(index + 1).map(String::as_str).unwrap_or("");
            if !matches!(after, "asleep" | "for" | "into" | "in" | "behind" | "out") {
                return true;
            }
            continue;
        }
        if matches!(word.as_str(), "tripped" | "slipped" | "slipping" | "stumbled" | "stumbling") {
            let after = &rest[(index + 1).min(rest.len())..(index + 7).min(rest.len())];
            if after.first().is_some_and(|a| SLIPPED_FROM.contains(a)) {
                continue;
            }
            if after.iter().any(|a| FOUND.contains(a)) {
                continue;
            }
            return true;
        }
    }
    false
}

/// `_after_greeting(norm)`.
fn _after_greeting(norm: &[String]) -> Option<Vec<String>> {
    GREETINGS_LONGEST.iter().find_map(|w| starts_with_words(norm, w).map(|r| r.to_vec()))
}

/// `after_greeting(norm)`.
pub fn after_greeting(norm: &[String]) -> Option<Vec<String>> {
    _after_greeting(norm)
}

/// `_after_farewell(norm)`.
fn _after_farewell(norm: &[String]) -> Option<Vec<String>> {
    FAREWELLS_LONGEST.iter().find_map(|w| starts_with_words(norm, w).map(|r| r.to_vec()))
}

/// `farewell_only(norm)`: "bye for now", "night night x".
pub fn farewell_only(norm: &[String]) -> bool {
    let Some(rest) = _after_farewell(norm) else { return false };
    if rest.iter().any(|w| NEEDS_ANSWER.contains(w)) {
        return false;
    }
    let left: Vec<&String> = rest.iter().filter(|w| !_WISH_WORDS.contains(*w)).collect();
    if left.len() < rest.len() && left.iter().all(|w| FAREWELL_TAIL.contains(*w) || POLITE.contains(*w)) {
        return true;
    }
    if rest.first().is_some_and(|w| _THANKS_WORDS.contains(w)) {
        return true;
    }
    rest.iter().all(|w| FAREWELL_TAIL.contains(w) || POLITE.contains(w) || _SAID_ON_ITS_OWN.contains(w) || TERMS_OF_ADDRESS.contains(w))
}

/// `greeting_only(norm)`: "hi there", "hello, i'm sam".
pub fn greeting_only(norm: &[String]) -> bool {
    let Some(rest) = _after_greeting(norm) else { return false };
    if rest.iter().all(|w| GREETING_TAIL.contains(w) || POLITE.contains(w)) {
        return true;
    }
    if rest.len() <= 2 && !rest.iter().any(|w| NEEDS_ANSWER.contains(w)) && !_says_what_they_are(&rest) {
        return true;
    }
    for lead in INTRODUCES.iter() {
        if rest.len() < lead.len() || rest[..lead.len()] != lead[..] {
            continue;
        }
        let tail: Vec<&String> = rest[lead.len()..].iter().filter(|w| !NAMING_WORDS.contains(*w)).collect();
        if !(1..=2).contains(&tail.len()) {
            return false;
        }
        return tail.iter().all(|w| _a_name(w));
    }
    false
}

/// `_says_what_they_are(rest)`: the fact store reads the words as a fact about the person.
fn _says_what_they_are(rest: &[String]) -> bool {
    crate::memory::facts::about_themselves(&rest.join(" ")).is_some_and(|f| !f.is_empty())
}

/// `_a_name(word)`: the fact store reads the word as a name.
fn _a_name(word: &str) -> bool {
    crate::memory::facts::reads_as_a_name(word).is_some_and(|n| !n.is_empty())
}

/// `_says_the_help_was_good(norm)`: "that helps", "ok that really helped".
fn _says_the_help_was_good(norm: &[String]) -> bool {
    let mut words: Vec<String> = norm.to_vec();
    while words.len() > 2 && BEFORE_HELP_WAS_GOOD.contains(&words[0]) {
        words.remove(0);
    }
    while words.len() > 2 {
        let said = words.join(" ");
        let mut tail: Option<Vec<String>> = AFTER_HELP_WAS_GOOD_SAID_OUT.iter().find(|p| said.ends_with(&format!(" {p}"))).map(|p| text::split(p));
        if tail.is_none() && AFTER_HELP_WAS_GOOD.contains(words.last().unwrap()) {
            tail = Some(vec![words.last().unwrap().clone()]);
        }
        match tail {
            Some(t) if words.len() >= t.len() + 2 => {
                let k = words.len() - t.len();
                words.truncate(k);
            }
            _ => break,
        }
    }
    HELP_WAS_GOOD.contains(&words.join(" "))
}

/// `_thanks_only(norm, clauses=None)`: a thank-you and nothing else to answer.
fn _thanks_only(norm: &[String], clauses: Option<&[Vec<String>]>) -> bool {
    let Some(rest) = _after_thanks(norm) else { return false };
    if rest.iter().any(|w| NEEDS_ANSWER.contains(w)) {
        return false;
    }
    if STILL_WRONG.is_match(&rest.join(" ")) {
        return false;
    }
    if _a_clause_of_its_own(&rest, clauses) {
        return false;
    }
    !_a_fall(&rest)
}

/// `_a_clause_of_its_own(rest, clauses=None)`: the thank-you carries a clause of its own.
fn _a_clause_of_its_own(rest: &[String], clauses: Option<&[Vec<String>]>) -> bool {
    if rest.is_empty() || THANKS_ABOUT.contains(&rest[0]) {
        return false;
    }
    if _starts_with(rest, &THANKS) {
        return false;
    }
    if rest.len() < 4 {
        return false;
    }
    if _a_step_done(rest) {
        return _the_step_went_wrong(rest);
    }
    if _agrees(rest) || rest.iter().any(|w| BETTER.contains(w)) {
        return false;
    }
    if _praises_the_thing(rest) {
        return false;
    }
    if rest.iter().any(|w| SAYS_ABOUT_YOU.contains(w)) {
        return false;
    }
    if _a_fall_that_is_not_one(rest) {
        return false;
    }
    if rest.iter().any(|w| w == "temperature") {
        return false;
    }
    if SIGNING_OFF.is_match(&rest.join(" ")) {
        return false;
    }
    for clause in clauses.unwrap_or(&[]).iter().skip(1) {
        if clause.len() >= 4 && !THANKS_ABOUT.contains(&clause[0]) {
            return true;
        }
    }
    false
}

/// `_a_step_done(rest)`: "i tried the ...", "we swapped it".
pub fn _a_step_done(rest: &[String]) -> bool {
    if rest.len() < 3 || !matches!(rest[0].as_str(), "i" | "we") {
        return false;
    }
    if !STEP_VERBS.contains(&rest[1]) && !(rest[1] == "gave" && rest[2..(4).min(rest.len())].iter().any(|w| w == "go")) {
        return false;
    }
    let after = &rest[2..];
    if matches!(rest[1].as_str(), "tried" | "swapped") {
        return after.first().map(String::as_str) == Some("it");
    }
    if STEP_ANY_OBJECT.contains(&rest[1]) && !matches!(after.first().map(String::as_str), Some("it") | Some("the")) {
        return false;
    }
    after.iter().any(|w| w == "the" || w == "my") || after.first().map(String::as_str) == Some("it")
}

/// `_the_step_went_wrong(rest)`.
pub fn _the_step_went_wrong(rest: &[String]) -> bool {
    rest.iter().skip(2).any(|w| STEP_WRONG.contains(w))
}

/// `_a_fall_that_is_not_one(rest)`: "fell asleep".
pub fn _a_fall_that_is_not_one(rest: &[String]) -> bool {
    if !rest.iter().any(|w| FELL_WORDS.contains(w)) {
        return false;
    }
    !_a_fall(rest)
}

/// `_praises_the_thing(rest)`.
fn _praises_the_thing(rest: &[String]) -> bool {
    if !(rest.iter().any(|w| PRAISING.contains(w) || PRAISING_VERB.contains(w)) || PRAISE_SAID.is_match(&rest.join(" "))) {
        return false;
    }
    !rest.iter().any(|w| TURNED_ROUND.contains(w))
}

/// `_agrees(rest)`: "you're right", "not a bad idea".
fn _agrees(rest: &[String]) -> bool {
    let Some(not_at) = rest.iter().position(|w| w == "not") else {
        return rest.iter().all(|w| AGREE_TAIL.contains(w));
    };
    let mut after = &rest[not_at + 1..];
    while after.first().is_some_and(|w| NOT_FILLER.contains(w)) {
        after = &after[1..];
    }
    if !after.first().is_some_and(|w| AGREE_AFTER_NOT.contains(w)) {
        return false;
    }
    after.iter().all(|w| AGREE_TAIL.contains(w) || AGREE_AFTER_NOT.contains(w))
}

/// `_ends_with_thanks(norm)`: "that's great, thanks".
fn _ends_with_thanks(norm: &[String]) -> bool {
    let mut said_to = norm.len();
    while said_to > 0 && ADDRESSED.contains(&norm[said_to - 1]) {
        said_to -= 1;
    }
    let line = &norm[..said_to];
    let named = said_to < norm.len();
    for words in THANKS_LONGEST.iter() {
        if line.len() > words.len() && line[line.len() - words.len()..] == words[..] {
            let rest: Vec<String> = line[..line.len() - words.len()].iter().filter(|w| !THANKS_NOISE.contains(*w)).cloned().collect();
            if rest.is_empty() {
                return true;
            }
            if rest.len() > 6 {
                continue;
            }
            if rest.iter().all(|w| AFFIRM.contains(w) || ACK.contains(w)) && !named {
                continue;
            }
            if matches!(rest[0].as_str(), "no" | "nope" | "nah" | "never" | "naw") {
                continue;
            }
            if !_agrees(&rest) {
                continue;
            }
            return true;
        }
    }
    false
}

/// `_talk_opening(text)`: "can we talk", "i need to talk to someone".
fn _talk_opening(text_: &str) -> bool {
    let mut words: Vec<String> = text::split(text_);
    let mut changed = true;
    while !words.is_empty() && changed {
        changed = false;
        if TALK_HEAD.contains(&words[0]) {
            words.remove(0);
            changed = true;
        }
        for tail in TALK_TAIL.iter() {
            if words.len() > tail.len() + 1 && words[words.len() - tail.len()..] == tail[..] {
                let k = words.len() - tail.len();
                words.truncate(k);
                changed = true;
                break;
            }
        }
    }
    !words.is_empty() && TALK_OPENING.fullmatch(&words.join(" ")).is_some()
}

/// `talk_frame(norm)`.
pub fn talk_frame(norm: &[String]) -> bool {
    let words: Vec<&str> = norm.iter().map(String::as_str).filter(|w| !w.is_empty()).collect();
    let text_ = words.join(" ");
    !text_.is_empty() && _talk_opening(&text_)
}

/// `opening_line(norm)`: "can i ask you something".
pub fn opening_line(norm: &[String]) -> bool {
    let words: Vec<&str> = norm.iter().map(String::as_str).filter(|w| !w.is_empty()).collect();
    if words.is_empty() {
        return false;
    }
    let text_ = words.join(" ");
    if _talk_opening(&text_) {
        return true;
    }
    if words.len() as i64 > OPENING_MAX {
        return false;
    }
    if matches!(words[0], "have" | "do" | "does" | "did") {
        return false;
    }
    if !words.iter().all(|w| OPENING_WORDS.contains(*w)) {
        return false;
    }
    OPENING_FRAME.is_match(&text_)
}

/// `_short_reply(text)`.
fn _short_reply(text_: &str) -> bool {
    SHORT_REPLIES.iter().any(|p| text_ == p || text_.starts_with(&format!("{p} ")))
}

/// `_cant_for_cannot(words)`.
fn _cant_for_cannot(words: &[String]) -> String {
    let mut joined: Vec<&str> = Vec::new();
    let mut index = 0;
    while index < words.len() {
        if words[index] == "can" && words.get(index + 1).map(String::as_str) == Some("not") {
            joined.push("cant");
            index += 2;
            continue;
        }
        joined.push(&words[index]);
        index += 1;
    }
    joined.join(" ")
}

/// `_agrees_to_try(norm)`: "i'll give it a go", "worth a try".
fn _agrees_to_try(norm: &[String]) -> bool {
    let words: Vec<String> = norm.iter().filter(|w| !w.is_empty() && *w != "|").cloned().collect();
    if words.is_empty() || words.len() > 9 {
        return false;
    }
    if words.iter().any(|w| QUESTION_WORDS.contains(w) || COMMIT_STOP.contains(w)) {
        return false;
    }
    let bare = words.join(" ");
    let mut hedged: &[String] = &words;
    while !hedged.is_empty() {
        if HEDGE_LEAD_ONE.contains(&hedged[0]) {
            hedged = &hedged[1..];
        } else if HEDGE_LEAD_TWO.contains(&hedged[..hedged.len().min(2)].join(" ")) {
            hedged = &hedged[2.min(hedged.len())..];
        } else {
            break;
        }
    }
    let mut said = _cant_for_cannot(hedged);
    let sw = text::split(&said);
    if sw.first().is_some_and(|w| NO_HARM_SUBJECT.contains(w)) {
        said = sw[1..].join(" ");
    }
    let n_said = text::split_ref(&said).len();
    if n_said > 0 && n_said <= 7 && COMMIT_NO_HARM.iter().any(|p| said == *p || said.starts_with(&format!("{p} "))) {
        return true;
    }
    if COMMIT_ALONE.contains(&bare) {
        return true;
    }
    if COMMIT_ALONE.iter().any(|p| bare.starts_with(&format!("{p} ")) && words.len() <= 5) {
        return true;
    }
    if COMMIT_ALONE_TAIL.iter().any(|p| bare == *p || bare.starts_with(&format!("{p} "))) {
        return true;
    }
    let mut lead: &[String] = strip_leaders(&words);
    if lead.first().is_some_and(|w| AFFIRM.contains(w)) {
        lead = &lead[1..];
        if lead.first().map(String::as_str) == Some("one") {
            lead = &lead[1..];
        }
    }
    if lead.first().is_some_and(|w| COMMIT_SUBJECT.contains(w)) {
        lead = &lead[1..];
    }
    if lead.first().is_some_and(|w| HEDGED_COMMIT.contains(w)) {
        let rest = lead[1..].join(" ");
        return HEDGED_ALONE.iter().any(|p| rest == *p || rest.starts_with(&format!("{p} ")));
    }
    if lead.first().is_some_and(|w| COMMIT_SUBJECT.contains(w)) {
        lead = &lead[1..];
    }
    if lead.is_empty() || !COMMIT_OPEN.contains(&lead[0]) {
        return false;
    }
    lead = &lead[1..];
    if lead.first().map(String::as_str) == Some("to") {
        lead = &lead[1..];
    }
    lead.first().is_some_and(|w| COMMIT_TRY.contains(w))
}

/// `_asks_the_room(stripped)`: "anyone know a good ...".
fn _asks_the_room(stripped: &[String]) -> bool {
    if stripped.len() < 2 || !ASK_THE_ROOM.contains(&stripped[0]) {
        return false;
    }
    if !ASK_THE_ROOM_VERB.contains(&stripped[1]) {
        return false;
    }
    if ASK_THE_ROOM_ANY.contains(&stripped[0]) {
        return true;
    }
    if !ASK_FOR_A_NAME.contains(&stripped[1]) {
        return false;
    }
    stripped.len() > 2 && ASK_THE_ROOM_WANTS.contains(&stripped[2])
}

/// `strip_leaders(clause) or clause`.
fn stripped_or_whole(clause: &[String]) -> &[String] {
    let s = strip_leaders(clause);
    if s.is_empty() { clause } else { s }
}

/// `_asks_at_the_end(clause)`.
fn _asks_at_the_end(clause: &[String]) -> bool {
    let stripped = stripped_or_whole(clause);
    if stripped.is_empty() {
        return false;
    }
    if _opens_question(stripped) {
        return true;
    }
    stripped.iter().any(|w| TAIL_ASKS.contains(w)) && stripped.iter().all(|w| TAIL_ASKS.contains(w) || TAIL_ASK_FILLER.contains(w))
}

/// `_says_nothing_but_the_floor(clause)`.
fn _says_nothing_but_the_floor(clause: &[String]) -> bool {
    stripped_or_whole(clause).iter().all(|w| SETUP_NOTHING.contains(w) || SETUP_CONNECTIVES.contains(w))
}

/// `the_setup(line)`: the clauses in front of the question ("i'm going to paris | what should i
/// see" -> "i am going to paris").
pub fn the_setup(line: &Line) -> Option<String> {
    let clauses: Vec<&Vec<String>> = line.clauses.iter().filter(|c| !c.is_empty()).collect();
    if clauses.len() < 2 {
        return None;
    }
    let ask_at = (1..clauses.len()).rev().find(|&k| _asks_at_the_end(clauses[k]))?;
    let before = &clauses[..ask_at];
    if before.iter().any(|c| _opens_question(c)) {
        return None;
    }
    if before.iter().all(|c| _says_nothing_but_the_floor(c)) {
        return None;
    }
    let setup = before.iter().map(|c| stripped_or_whole(c).join(" ")).collect::<Vec<_>>().join(" ");
    let setup = A_FRAME_IN_FRONT.sub("", &setup);
    let setup = text::strip_chars(&setup, " ,-").to_string();
    if setup.is_empty() {
        return None;
    }
    if text::split_ref(&setup).len() as i64 > SETUP_MOST_WORDS {
        return None;
    }
    if FRAME_NOT_A_SETUP.match_(&format!(" {setup}")).is_some() {
        return None;
    }
    if text::split_ref(&setup).iter().all(|w| SETUP_CONNECTIVES.contains(*w)) {
        return None;
    }
    Some(setup)
}

/// `_opens_question(clause)`: the clause opens like a question.
fn _opens_question(clause: &[String]) -> bool {
    let stripped = stripped_or_whole(clause);
    if _asks_the_room(stripped) {
        return true;
    }
    if stripped.is_empty() || !QUESTION_STARTS.contains(&stripped[0]) {
        return false;
    }
    if stripped.len() > 2 && NEGATIONS.contains(&stripped[1]) && !SUBJECT_NEGATIONS.contains(&stripped[1]) && !PRONOUNS.contains(&stripped[2]) {
        return false;
    }
    if stripped[0] == "do" && stripped.len() > 1 && QUESTION_WORDS.contains(&stripped[1]) {
        return false;
    }
    if YESNO_STARTS.contains(&stripped[0]) && stripped.len() > 1 && _verb_form(&stripped[1], &stripped[0]) {
        return false;
    }
    if stripped.len() > 1 && opens_without_subject(stripped, 0) {
        return false;
    }
    true
}

/// `_hands_a_question_to_the_speaker(norm)`: "ask me a question", "quiz me".
fn _hands_a_question_to_the_speaker(norm: &[String]) -> bool {
    let words: Vec<&String> = norm.iter().filter(|w| !w.is_empty() && *w != "|").collect();
    for at in 0..words.len().saturating_sub(1) {
        let word = words[at];
        if !HAND_OVER_VERBS.contains(word) {
            continue;
        }
        let next = words[at + 1];
        let handed_to = HAND_OVER_TO.contains(next) || ((next == "my" || next == "our") && at + 2 < words.len() && HAND_OVER_MINE.contains(words[at + 2]));
        if !handed_to {
            continue;
        }
        let before = &words[..at];
        if before.is_empty() {
            return true;
        }
        let modal = before.iter().any(|w| HAND_OVER_MODALS.contains(*w));
        if !before.iter().all(|w| HAND_OVER_LEADS.contains(*w) || HAND_OVER_MODALS.contains(*w) || (modal && HAND_OVER_SUBJECTS.contains(*w))) {
            continue;
        }
        if modal || !before.iter().any(|w| HAND_OVER_SUBJECTS.contains(*w)) {
            return true;
        }
    }
    false
}

/// `_asks_a_question_coming(norm, text)`: "can i ask you something".
fn _asks_a_question_coming(norm: &[String], text_: &str) -> bool {
    if !(1..=12).contains(&norm.len()) || text_.contains('?') || norm.iter().any(|w| ALREADY_ASKED.contains(w)) {
        return false;
    }
    if _hands_a_question_to_the_speaker(norm) {
        return false;
    }
    if matches!(norm[0].as_str(), "do" | "does" | "did" | "any") {
        return false;
    }
    if matches!(norm[0].as_str(), "have" | "has") && norm.len() > 1 && matches!(norm[1].as_str(), "you" | "u" | "anyone" | "anybody" | "any" | "they") {
        return false;
    }
    for (at, word) in norm.iter().take(5).enumerate() {
        if word == "asked" && at == 1 && matches!(norm[0].as_str(), "i" | "we") {
            continue;
        }
        if norm[..at].iter().any(|w| PAST_ASK.contains(w)) {
            continue;
        }
        let after = norm.get(at + 1).map(String::as_str).unwrap_or("");
        let asked_here = ASKING_VERBS.contains(word)
            && (ASK_TARGET.contains(after) || ASK_ABOUT.contains(after))
            && (at == 0 || norm[..at].iter().any(|b| ASK_MODALS.contains(b) || ASK_AUX.contains(b)) || ASK_TARGET.contains(after));
        let named = QUESTION_NOUNS.contains(word) || asked_here;
        if !named {
            continue;
        }
        if at == 0 || norm[..at].iter().any(|b| ASK_FRAMES.contains(b)) {
            return true;
        }
    }
    false
}

/// `_asks_for_tips(text, norm=())`: "any tips for ...".
fn _asks_for_tips(text_: &str, norm: &[String]) -> bool {
    let about = norm.iter().any(|w| !TIPS_FRAME.contains(w));
    for m in TIPS.finditer(text_) {
        let (before, after) = (&text_[..m.start()], &text_[m.end()..]);
        if TIPS_AMOUNT.is_match(before) {
            continue;
        }
        if TIPS_WANTED.is_match(before) {
            if about {
                return true;
            }
            continue;
        }
        if matches!(m.get(1), Some("tips" | "hints" | "pointers")) {
            if TIPS_SUBJECT.is_match(before) {
                continue;
            }
            if TIPS_ABOUT.match_(after).is_some() || text::strip_chars(after, " ?.!,").is_empty() {
                return true;
            }
        }
    }
    false
}

/// `_asks_the_bot(norm)`: "you know what ...".
fn _asks_the_bot(norm: &[String]) -> bool {
    if norm.len() < 3 {
        return false;
    }
    YOU_WORDS.contains(&norm[0]) && ASK_VERBS.contains(&norm[1]) && ASK_FOLLOW.contains(&norm[2])
}

/// `after_any_link(norm)`: "any idea how to ..." -> the words after the link (empty when none).
pub fn after_any_link(norm: &[String]) -> Vec<String> {
    if norm.is_empty() || !matches!(norm[0].as_str(), "any" | "anything") {
        return vec![];
    }
    if norm.len() < 2 || !ANY_ANSWER_WANTED.contains(&norm[1]) {
        return vec![];
    }
    for at in 2..norm.len() {
        if ANY_ANSWER_LINKS.contains(&norm[at]) {
            return norm[at + 1..].to_vec();
        }
    }
    vec![]
}

/// `_asks_after_any(norm)`.
fn _asks_after_any(norm: &[String]) -> bool {
    if norm.len() < 2 || ANY_ALREADY_ASKS.contains(&norm[1]) {
        return false;
    }
    let about = after_any_link(norm);
    !about.is_empty() && !_names_a_person(&about)
}

/// `_names_a_person(words)`.
fn _names_a_person(words: &[String]) -> bool {
    (0..words.len().saturating_sub(1)).any(|k| OWNS_A_PERSON.contains(&words[k]) && A_PERSON_NAMED.contains(&words[k + 1]))
}

/// `_ask_words(norm)`.
fn _ask_words(norm: &[String]) -> Vec<String> {
    let mut words: Vec<String> = norm.iter().map(|w| BOUT.get(w).cloned().unwrap_or_else(|| w.clone())).collect();
    while words.first().is_some_and(|w| FILLER_BEFORE.contains(w)) {
        words.remove(0);
    }
    while words.last().is_some_and(|w| FILLER_AFTER.contains(w)) {
        words.pop();
    }
    words
}

/// `_asks_it_back(norm)`: "and you?", "how about you".
fn _asks_it_back(norm: &[String]) -> bool {
    ASKS_IT_BACK.contains(&_ask_words(norm).join(" "))
}

/// `asks_how_i_am(norm)`: "how are you", "you ok?".
pub fn asks_how_i_am(norm: &[String]) -> bool {
    let mut norm = norm;
    while norm.len() > 2 && norm[norm.len() - 2] == "with" && GREETING_ASK_SECOND_PERSON.contains(&norm[norm.len() - 1]) {
        norm = &norm[..norm.len() - 2];
    }
    if norm.is_empty() || norm.len() > 8 {
        return false;
    }
    let words = sv(norm);
    if !any_in(&words, &GREETING_ASK_SUBJECT) {
        return false;
    }
    if norm[0] == "what" {
        return words.iter().all(|w| GREETING_WHAT_SAYING.contains(*w));
    }
    if norm[0] != "how" {
        if norm.len() > 4 || !any_in(&words, &GREETING_ASK_STATE) || !any_in(&words, &GREETING_ASK_SECOND_PERSON) {
            return false;
        }
        let at = norm.iter().position(|w| GREETING_ASK_STATE.contains(w)).unwrap();
        if !norm[..at].iter().any(|w| GREETING_ASK_SECOND_PERSON.contains(w)) {
            return false;
        }
        return !(norm.len() > at + 1 && GREETING_ASK_ABOUT_SOMETHING.contains(&norm[at + 1]));
    }
    words.iter().all(|w| GREETING_ASK_WORDS.contains(*w))
}

/// `_tag_tells_a_plan(norm)`.
fn _tag_tells_a_plan(norm: &[String]) -> bool {
    let body = &norm[..norm.len() - 1];
    if body.iter().any(|w| TAG_PLAN.contains(w)) {
        return true;
    }
    (0..body.len().saturating_sub(1)).any(|k| body[k] == "at" && TAG_TIME_NUMBER.contains(&body[k + 1]))
}

/// `_tag_checks_a_fact(norm)`.
fn _tag_checks_a_fact(norm: &[String]) -> bool {
    let body = &norm[..norm.len() - 1];
    for k in 0..body.len().saturating_sub(1) {
        if TAG_CHECKABLE.contains(&body[k]) && body[k + 1] == "be" {
            return false;
        }
    }
    body.iter().any(|w| TAG_CHECKABLE.contains(w))
}

/// `_asks_for_confirmation(norm)`: "..., right?", "..., isn't it".
fn _asks_for_confirmation(norm: &[String]) -> bool {
    if norm.len() < 3 || !TAG_ENDINGS.contains(norm.last().unwrap()) {
        return false;
    }
    if _opens_question(norm) || _asks_the_bot(norm) || _tag_tells_a_plan(norm) || !_tag_checks_a_fact(norm) {
        return false;
    }
    if norm.last().unwrap() == "right" {
        let before = &norm[norm.len() - 2];
        if before.ends_with("ing") || before.ends_with("ed") {
            return false;
        }
        return !TAG_RIGHT_NOT.contains(before);
    }
    norm.len() >= 4
}

/// `_asks_about_the_bot_day(norm)`: "how was your day".
fn _asks_about_the_bot_day(norm: &[String]) -> bool {
    let words = _ask_words(norm);
    if words.is_empty() {
        return false;
    }
    let line = words.join(" ");
    BOT_DAY_SHAPES.iter().any(|s| s.fullmatch(&line).is_some())
}

/// `_asks_how_it_spends_its_time(norm)`.
fn _asks_how_it_spends_its_time(norm: &[String]) -> bool {
    if norm.is_empty() || norm.len() > 12 {
        return false;
    }
    if !any_of(&sv(norm), &["you", "u", "ya", "your", "ur"]) {
        return false;
    }
    SPENDS_ITS_TIME.is_match(&norm.join(" "))
}

/// `_asks_whether_the_bot_is_about(norm)`: "are you there", "you free?".
fn _asks_whether_the_bot_is_about(norm: &[String]) -> bool {
    if norm.is_empty() || norm.len() > 6 {
        return false;
    }
    let words = sv(norm);
    if !any_of(&words, &["you", "u", "ya", "ur", "your"]) {
        return false;
    }
    if any_in(&words, &FIRST_PERSON) || any_in(&words, &MANNERS_WORDS) {
        return false;
    }
    any_in(&words, &ABOUT_AT_A_TIME) || any_in(&words, &FREE_OR_BUSY)
}

/// `the_manners_question(norm)`: "what are you up to".
pub fn the_manners_question(norm: &[String]) -> bool {
    let mut lead = 0;
    while lead < norm.len() && UP_TO_LEADS.contains(&norm[lead]) {
        lead += 1;
    }
    let norm = &norm[lead..];
    if norm.is_empty() || (!MANNERS_OPENERS.contains(&norm[0]) && !SAYS_WHAT_ARE_YOU.contains(&norm[0])) {
        return false;
    }
    let words = sv(norm);
    if !any_of(&words, &["you", "u", "ya"]) && !SAYS_WHAT_ARE_YOU.contains(&norm[0]) {
        return false;
    }
    if words.contains("been") && any_of(&words, &["do", "doing"]) {
        return false;
    }
    if words.contains("busy") && any_of(&words, &["keeps", "keeping"]) {
        return words.iter().all(|w| KEEPS_BUSY_SHAPE.contains(*w));
    }
    if !any_of(&words, &["up", "doing", "doin", "do", "at", "been"]) {
        return false;
    }
    if any_in(&words, &WHEN_IN_THE_ASK) && !any_of(&words, &["up", "doing", "doin", "at", "been"]) {
        return false;
    }
    norm.iter().all(|w| DOING_FILLER.contains(w) || WHEN_IN_THE_ASK.contains(w))
}

/// `_asks_what_is_new_with_the_bot(norm)`.
fn _asks_what_is_new_with_the_bot(norm: &[String]) -> bool {
    let words = sv(norm);
    if words.is_empty() || norm.len() > 9 {
        return false;
    }
    if !any_of(&words, &["you", "u", "ya", "your", "yours"]) || !any_in(&words, &NEW_FRAME) || any_in(&words, &NEW_ASK_AHEAD) {
        return false;
    }
    words.iter().all(|w| NEW_ASK_WORDS.contains(*w))
}

/// `_asks_what_the_bot_is_doing(norm)`.
fn _asks_what_the_bot_is_doing(norm: &[String]) -> bool {
    if norm.is_empty() || norm.len() > 10 {
        return false;
    }
    let words = sv(norm);
    if !(any_of(&words, &["doing", "do"]) || matches!(norm.last().unwrap().as_str(), "to" | "much" | "on") || words.contains("up")) {
        return false;
    }
    if words.contains("going") && words.contains("on") && any_in(&words, &GOING_ON_WHEN) {
        return false;
    }
    if norm.iter().all(|w| DOING_FILLER.contains(w)) {
        return true;
    }
    _asks_what_the_bot_has_been_doing(norm)
}

/// `_asks_what_the_bot_has_been_doing(norm)`.
fn _asks_what_the_bot_has_been_doing(norm: &[String]) -> bool {
    let words = sv(norm);
    if !any_of(&words, &["you", "u", "ya", "your", "yours"]) || any_in(&words, &STILL_COMING) || !any_in(&words, &BEEN_SAID) {
        return false;
    }
    if !any_of(&words, &["did", "done", "been", "were", "was", "had"]) && !any_in(&words, &BEEN_WHEN) {
        return false;
    }
    if words.contains("where") && any_in(&words, &SPENT_TIME) && any_of(&words, &["have", "has", "been", "were", "was"]) {
        return true;
    }
    if !any_of(&words, &["do", "does", "did", "done", "doing"]) {
        return false;
    }
    norm.iter().all(|w| DOING_FILLER.contains(w) || BEEN_WHEN.contains(w) || BEEN_SAID.contains(w))
}

/// `_wants_a_recommendation(norm, text)`: "best", "better" asked of a thing.
fn _wants_a_recommendation(norm: &[String], text_: &str) -> bool {
    if norm.iter().any(|w| COMPARATIVE.contains(w)) && COMPARISON.is_match(&format!(" {text_} ")) {
        return true;
    }
    for (index, word) in norm.iter().enumerate() {
        if !COMPARATIVE.contains(word) {
            continue;
        }
        let before = if index > 0 { norm[index - 1].as_str() } else { "" };
        let after = norm.get(index + 1).map(String::as_str).unwrap_or("");
        if A_PERSON_AFTER.contains(after) && !(matches!(after, "man" | "woman") && norm.get(index + 2).map(String::as_str) == Some("for")) {
            continue;
        }
        if !DESCRIBES.contains(before) && !AFTER_COMPARATIVE.contains(after) {
            return true;
        }
    }
    norm.iter().any(|w| RECOMMEND.contains(w) && !COMPARATIVE.contains(w))
}

/// `_the_thing_a_thank_you_is_for(norm)`: "thanks for the tips, but ..." -> the span thanked for.
fn _the_thing_a_thank_you_is_for(norm: &[String]) -> Option<(usize, usize)> {
    if norm.is_empty() || !_starts_with(norm, &THANKS) {
        return None;
    }
    let object = |w: &String| GRATITUDE_OBJECT.contains(w) || RECOMMEND.contains(w) || TIP_WORDS.contains(w);
    for (at, word) in norm.iter().take(5).enumerate() {
        if word != "for" {
            continue;
        }
        if norm[..at].iter().any(|w| !COURTESY_WORD.contains(w) && !GRATITUDE_OBJECT.contains(w)) {
            continue;
        }
        let mut end = at + 1;
        while end < norm.len() && object(&norm[end]) {
            if end > at + 1 && GRATITUDE_ASK_AGAIN.contains(&norm[end]) {
                break;
            }
            end += 1;
        }
        if end > at + 1 {
            while end + 1 < norm.len() && GRATITUDE_JOINERS.contains(&norm[end]) {
                let mut run = end + 1;
                while run < norm.len() && object(&norm[run]) {
                    run += 1;
                }
                if run == end + 1 {
                    break;
                }
                if run < norm.len() && norm[run] == "for" {
                    break;
                }
                end = run;
            }
            return Some((at, end));
        }
    }
    None
}

/// `_asks(norm, text)`: the ask labels, without what a thank-you was for.
fn _asks(norm: &[String], text_: &str) -> Vec<String> {
    if let Some((a, b)) = _the_thing_a_thank_you_is_for(norm) {
        let mut left: Vec<String> = norm[..a].to_vec();
        left.extend(norm[b..].iter().cloned());
        let t = format!(" {} ", left.join(" "));
        return _ask_labels(&left, &t);
    }
    _ask_labels(norm, text_)
}

/// `_a_bare_definition(text)`: "x meaning".
fn _a_bare_definition(text_: &str) -> bool {
    let Some(found) = BARE_DEFINITION.search(text_) else { return false };
    let head = found.str(1).to_lowercase();
    if head.ends_with("'s") || head.ends_with("\u{2019}s") || rx(r"\d").is_match(&head) {
        return false;
    }
    if rx(r"\s*s\b").match_(&text_[found.end()..]).is_some() {
        return false;
    }
    !NOT_A_DEFINITION_HEAD.contains(&head)
}

/// `_ask_labels(norm, text)`: what the line asks for ("how_long", "recommend", "steps", ...).
fn _ask_labels(norm: &[String], text_: &str) -> Vec<String> {
    let mut asks: BTreeSet<&str> = BTreeSet::new();
    let words = sv(norm);
    if _asks_the_room(norm) && ASK_FOR_A_NAME.contains(&norm[1]) {
        asks.insert("recommend");
    }
    if words.contains("how") {
        if words.contains("long") {
            asks.insert("how_long");
        } else if words.contains("much") || words.contains("many") {
            asks.insert("how_much");
        } else if words.contains("often") || words.contains("frequently") {
            asks.insert("how_often");
        } else {
            asks.insert("how");
        }
    }
    for label in ["what", "why", "when", "where", "who", "which"] {
        if words.contains(label) {
            asks.insert(label);
        }
    }
    if _wants_a_recommendation(norm, text_) || _has_any(text_, &RECOMMEND_PHRASES) || _asks_for_tips(text_, norm) {
        asks.insert("recommend");
    }
    if _has_any(text_, &STEPS) || STEPS_WORD.is_match(text_) || _asks_for_help(text_) {
        asks.insert("steps");
    }
    if _has_any(text_, &OPINION) {
        asks.insert("opinion");
    }
    if _has_any(text_, &COMPARE) || any_of(&words, &["vs", "versus"]) {
        asks.insert("compare");
    }
    if !ASKS_ABOUT_A_PERSON.is_match(text_) && (_has_any(text_, &DEFINITION) || _a_bare_definition(text_)) {
        asks.insert("definition");
    }
    let digits = rx(r"\d").is_match(&A_DATE_FIGURE.sub(" ", text_));
    if any_in(&words, &MATH_WORDS) || text_.contains('%') || (digits && any_of(&words, &["of", "each", "per", "split"])) {
        asks.insert("maths");
    }
    if text_.contains(" or ") && !two_ends_of_one_scale(text_) && !_turns_the_pair_down(text_) {
        asks.insert("choice");
    }
    if any_of(&words, &["list", "ideas", "options", "examples", "suggest"]) {
        asks.insert("list");
    }
    if any_of(&words, &["write", "word", "rewrite", "draft"]) {
        asks.insert("write");
    }
    if _asks_a_question_coming(norm, text_) {
        asks.insert("question_coming");
    }
    asks.into_iter().map(str::to_string).collect()
}

/// `_third_person(stem)`.
pub fn _third_person(stem: &str) -> String {
    let s: Vec<char> = stem.chars().collect();
    if s.len() > 2 && stem.ends_with('y') && !"aeiouy".contains(s[s.len() - 2]) {
        return format!("{}ies", &stem[..stem.len() - 1]);
    }
    if ["s", "sh", "ch", "x", "z", "o"].iter().any(|e| stem.ends_with(e)) {
        return format!("{stem}es");
    }
    format!("{stem}s")
}

/// `_verb_form(word, auxiliary='')`.
fn _verb_form(word: &str, auxiliary: &str) -> bool {
    if word.ends_with("ing") && !PAST_AUX.contains(auxiliary) {
        return false;
    }
    VERB_FORMS.contains(word)
}

/// `noun_phrase_request(line)`: "hotels in rome" said as a request.
pub fn noun_phrase_request(line: &Line) -> bool {
    let words: Vec<&String> = line.norm.iter().filter(|w| !w.is_empty()).collect();
    if words.is_empty() || words.len() as i64 > NOUN_REQUEST_MAX || line.signals.question_mark {
        return false;
    }
    if words.iter().any(|w| NOUN_REQUEST_SELF.contains(*w)) {
        return false;
    }
    if words.iter().enumerate().any(|(k, w)| VERBISH.contains(*w) && !(k > 0 && words[k - 1] == "to")) {
        return false;
    }
    words.len() >= 2 && words.iter().any(|w| PLACE_WORD.contains(*w))
}

/// `taking_up(line, read=None)`: "i've started running" -> what they took up.
pub fn taking_up(line: &Line, read: Option<&dyn ReadView>) -> Option<TakingUp> {
    let said = read.and_then(|r| r.clean()).filter(|c| !c.is_empty()).unwrap_or(&line.text);
    let said = rx("['\u{2019}]").sub("", said);
    let said = text::strip(&said);
    if said.is_empty() || format!(" {} ", said.to_lowercase()).contains(" not ") {
        return None;
    }
    if rx_i(r"\W*(?:am|is|are|was|were)\s+(?:i|we|you|they|he|she)\b").match_(said).is_some() {
        return None;
    }
    let m = INTEREST_HEAD.match_(said)?;
    let thing = INTEREST_TAIL.sub("", text::strip_chars(m.name_str("thing"), " .,!?"));
    let thing = text::strip(&thing).to_string();
    let words = text::split(&thing);
    if words.is_empty() || words.len() > 4 {
        return None;
    }
    if INTEREST_NOT.contains(&words[0]) || (words.len() > 1 && INTEREST_NOT.contains(&words[1])) {
        return None;
    }
    let thing = thing.to_lowercase();
    if thing.is_empty() {
        return None;
    }
    Some(TakingUp {
        text: thing,
        words,
        shape: if STARTED_OUT.is_match(said) { "started" } else { "doing" }.to_string(),
        back: BACK_INTO.is_match(said),
    })
}

/// A run-time regex with re.I.
fn rx_i(pattern: &str) -> &'static Re {
    crate::understanding::rx_flags(pattern, crate::py::re::I)
}

/// `_bare_subject(clause)`.
fn _bare_subject(clause: &[String]) -> bool {
    if clause.is_empty() {
        return false;
    }
    for word in clause {
        if VERBISH.contains(word) || (text::len(word) > 3 && (word.ends_with("ed") || word.ends_with("ing"))) {
            return false;
        }
    }
    if DETERMINERS.contains(&clause[0]) {
        return true;
    }
    clause.len() == 1 && !NOT_A_SUBJECT.contains(&clause[0])
}

/// `_continues_the_sentence(clauses, index)`: "my favourite food | what is yours".
fn _continues_the_sentence(clauses: &[Vec<String>], index: usize) -> bool {
    if index < 1 || !_bare_subject(&clauses[index - 1]) {
        return false;
    }
    let clause = &clauses[index];
    if clause[(1).min(clause.len())..(3).min(clause.len())].iter().any(|w| SUBJECT_OF_ITS_OWN.contains(w)) {
        return false;
    }
    if WH_CONTINUES.contains(&clause[0]) {
        return true;
    }
    if !YESNO_STARTS.contains(&clause[0]) {
        return false;
    }
    index >= 2 && _bare_subject(&clauses[index - 2])
}

/// `denied(text, cue)`: the cue is said with a denial in front ("not sad").
pub fn denied(text_: &str, cue: &str) -> bool {
    cue_rx(cue).finditer(text_).iter().any(|m| DENY_BEFORE.is_match(&text_[..m.start()]))
}

/// `_a_state_of_the_speaker(norm, index)`.
fn _a_state_of_the_speaker(norm: &[String], index: usize) -> bool {
    let mut verb = false;
    let mut back = index as isize - 1;
    while back >= 0 && index as isize - back <= 4 {
        let word = &norm[back as usize];
        if SPEAKER.contains(word) {
            return verb;
        }
        if STATING_VERB.contains(word) {
            verb = true;
        } else if !NEGATORS.contains(word) && !NEGATION_SKIP.contains(word) {
            return false;
        }
        back -= 1;
    }
    false
}

/// `_asks_for_it_rather_than_got_it(norm)`.
fn _asks_for_it_rather_than_got_it(norm: &[String]) -> bool {
    let Some(arrived) = norm.iter().position(|w| LETTER_RECEIVED.contains(w)) else { return false };
    norm[..=arrived].iter().any(|w| LETTER_ASKED_FOR.contains(w))
}

/// `_a_letter_someone_else_got(norm, verb)`.
fn _a_letter_someone_else_got(norm: &[String], verb: usize) -> bool {
    for (index, word) in norm[..verb.min(norm.len())].iter().enumerate() {
        if LETTER_SOMEONE_ELSES.contains(word) {
            let after = norm.get(index + 1).map(String::as_str).unwrap_or("");
            if !after.is_empty() && !LETTER_ITSELF.contains(after) {
                return true;
            }
        }
    }
    false
}

/// `summons_of(norm, kind)`: "the council wants to see me", "i got a letter from the court".
pub fn summons_of(norm: &[String], kind: &str) -> Option<Summons> {
    if norm.is_empty() || kind == "question" {
        return None;
    }
    let words = sv(norm);
    let joined_plain = norm.join(" ");
    let mut who: Option<String> = None;
    let mut at: Option<usize> = None;
    for phrase in SUMMONS_WHO_PHRASES.iter() {
        if joined_plain.contains(phrase.as_str()) {
            let first = text::split_ref(phrase)[0];
            who = Some(phrase.clone());
            at = Some(norm.iter().position(|w| w == first).unwrap_or_else(|| panic!("ValueError: {first} is not in list")));
            break;
        }
    }
    if who.is_none() {
        who = norm.iter().find(|w| SUMMONS_WHO.contains(*w)).cloned();
        at = who.as_ref().map(|w| norm.iter().position(|x| x == w).unwrap());
    }
    let who = who?;
    let at = at.unwrap();
    let mut verb = norm.iter().position(|w| SUMMONS_VERB.contains(w) && w != "has" && w != "have");
    if verb.is_none() {
        verb = norm.iter().position(|w| SUMMONS_VERB.contains(w));
    }
    let joined = format!(" {} ", norm.join(" "));
    let letter_first = verb.is_some_and(|v| {
        at > v
            && SUMMONS_WRITTEN.iter().any(|p| joined.contains(p.as_str()))
            && norm.iter().any(|w| LETTER_RECEIVED.contains(w))
            && !_asks_for_it_rather_than_got_it(norm)
            && !_a_letter_someone_else_got(norm, v)
    });
    let verb = verb?;
    if at > verb && !letter_first {
        return None;
    }
    if norm[verb.saturating_sub(3)..verb].iter().any(|w| SUMMONS_NOT.contains(w)) {
        return None;
    }
    if letter_first || SUMMONS_WRITTEN.iter().any(|p| joined.contains(p.as_str())) {
        return Some(Summons { who, shape: "written".to_string() });
    }
    if let Some(rang) = SUMMONS_RANG.iter().find(|p| joined.contains(p.as_str())) {
        let after = joined.split_once(rang.as_str()).map(|(_, b)| b).unwrap_or("");
        let gap: Vec<&str> = text::split_ref(after).into_iter().take(3).collect();
        if !gap.iter().any(|w| SUMMONS_CALL_DONE.contains(*w)) && !gap.iter().any(|w| SUMMONS_SAID.contains(*w)) {
            return Some(Summons { who, shape: "rang".to_string() });
        }
    }
    if !SUMMONS_WANTS.iter().any(|w| words.contains(w.as_str())) {
        return None;
    }
    if SUMMONS_SEE.iter().any(|p| joined.contains(p.as_str())) {
        return Some(Summons { who, shape: "see".to_string() });
    }
    None
}

/// `_said_to_somebody(norm, index)`: "thanks mate" - the address word ends a short line.
fn _said_to_somebody(norm: &[String], index: usize) -> bool {
    if index + 1 != norm.len() || !(2..=3).contains(&norm.len()) {
        return false;
    }
    ADDRESSED_LINES.contains(&norm[0])
}

/// `_a_body_not_a_mood(norm, index)`: "sick stomach".
fn _a_body_not_a_mood(norm: &[String], index: usize) -> bool {
    norm.get(index + 1).is_some_and(|f| STOMACH_WORDS.contains(f))
}

/// `_asks_what_was_meant(text)`: the whole line asks what the bot's last answer meant.
fn _asks_what_was_meant(text_: &str) -> bool {
    let asked = text::strip(text_);
    if asked.is_empty() {
        return false;
    }
    MEAN.match_(asked).is_some()
}

/// `_asks_back(norm)`: "you?", "and you".
pub fn _asks_back(norm: &[String]) -> bool {
    let words: Vec<&String> = norm.iter().filter(|w| !w.is_empty()).collect();
    let n = words.len();
    if n < 2 || !BACK_AT.contains(words[n - 1]) {
        return false;
    }
    if (n == 2 && BACK_JOIN.contains(words[0])) || (n == 3 && BACK_ASKING.contains(words[0]) && BACK_JOIN.contains(words[1])) {
        return true;
    }
    if n > 2 && BACK_NOT_A_SUBJECT.contains(words[n - 3]) {
        return false;
    }
    if words[..n - 1].iter().any(|w| BACK_AT.contains(*w) || *w == "youre" || *w == "your") {
        return false;
    }
    if words[n - 2] == "about" && !words[..n - 2].iter().any(|w| BACK_ASKING.contains(*w)) {
        return false;
    }
    if BACK_JOIN.contains(words[n - 2]) {
        let joined: Vec<&str> = words.iter().map(|w| w.as_str()).collect();
        return _polarity(norm) != 0 || emotion_of(&joined.join(" ")).is_some() || words[..n - 2].iter().any(|w| BACK_AFTER.contains(*w));
    }
    BACK_AFTER.contains(words[n - 2])
}

/// `says_how_they_are(norm)`: "i am fine", "im ok".
pub fn says_how_they_are(norm: &[String]) -> bool {
    norm.iter().enumerate().any(|(k, w)| SAYS_HOW_THEY_ARE.contains(w) && _a_state_of_the_speaker(norm, k))
}

/// `a_bad_verdict(norm, index)`: the word at `index` is a verdict on a thing ("the film was
/// rubbish"), not a feeling.
pub fn a_bad_verdict(norm: &[String], index: usize) -> bool {
    if index >= norm.len() || (!VERDICT_NOUNS.contains(&norm[index]) && !VERDICT_ADJECTIVES.contains(&norm[index])) {
        return false;
    }
    let mut back = index as isize - 1;
    while back >= 0 && VERDICT_FILLER.contains(&norm[back as usize]) {
        back -= 1;
    }
    if back < 0 || !COPULAS.contains(&norm[back as usize]) {
        return false;
    }
    let subject = back - 1;
    if subject < 0 {
        return false;
    }
    let s = &norm[subject as usize];
    !(PRONOUN_SUBJECT.contains(s) || ASKING_SUBJECT.contains(s) || COPULAS.contains(s) || NOT_A_THING.contains(s))
}

/// `verdicts_a_thing(clean)`.
pub fn verdicts_a_thing(clean: &str) -> bool {
    let said = text::squash(&clean.to_lowercase().replace('\u{2019}', "'").replace('\'', ""));
    let norm: Vec<String> = text::split_ref(&said).iter().map(|p| rx("[^a-z0-9]+").sub("", p)).filter(|w| !w.is_empty()).collect();
    (0..norm.len()).any(|k| a_bad_verdict(&norm, k))
}

/// `_polarity(norm)`: -1, 0 or 1: how the line feels.
fn _polarity(norm: &[String]) -> i64 {
    let mut score: i64 = 0;
    let mut negated = false;
    for (index, word) in norm.iter().enumerate() {
        if NEGATORS.contains(word) {
            negated = true;
            continue;
        }
        if TERMS_OF_ADDRESS.contains(word) && _said_to_somebody(norm, index) {
            negated = false;
            continue;
        }
        let feeling: Option<i64> = if OK_WORDS.contains(word) || SAYS_HOW_THEY_ARE.contains(word) {
            if _a_state_of_the_speaker(norm, index) { Some(1) } else { None }
        } else if POSITIVE.contains(word) {
            Some(1)
        } else if NEGATIVE.contains(word) {
            if !BODY_STATE_WORDS.contains(word) || !_a_body_not_a_mood(norm, index) { Some(-1) } else { None }
        } else if a_bad_verdict(norm, index) {
            Some(-1)
        } else {
            None
        };
        let Some(feeling) = feeling else {
            if !NEGATION_SKIP.contains(word) {
                negated = false;
            }
            continue;
        };
        score += if negated { -feeling } else { feeling };
        negated = false;
    }
    (score > 0) as i64 - (score < 0) as i64
}

/// `_thanks_still_asks(norm)`: "thanks, what about ...".
fn _thanks_still_asks(norm: &[String]) -> bool {
    for (index, word) in norm.iter().enumerate() {
        if !THANKS_ASK_WORDS.contains(word) {
            continue;
        }
        let back = if index > 0 { norm[index - 1].as_str() } else { "" };
        if GIVEN_ADVICE.contains(back) {
            continue;
        }
        return true;
    }
    for (index, word) in norm.iter().enumerate() {
        if !THANKS_ASK_NOUNS.contains(word) {
            continue;
        }
        let back = if index > 0 { norm[index - 1].as_str() } else { "" };
        if GIVEN_ADVICE.contains(back) {
            continue;
        }
        if index + 2 < norm.len() && THANKS_ASK_PREPS.contains(&norm[index + 1]) && pyuni::is_alpha(&norm[index + 2]) {
            return true;
        }
    }
    false
}

/// `_only_the_ask(norm, asks, text)`: "which one to pick" said without a question mark.
fn _only_the_ask(norm: &[String], asks: &[String], text_: &str) -> bool {
    let words: Vec<&String> = norm.iter().filter(|w| !POLITE.contains(*w)).collect();
    if words.is_empty() || words.len() > 8 {
        return false;
    }
    if words.iter().any(|w| SPEAKER.contains(*w) || SAYS_WHAT_IT_IS.contains(*w)) {
        return false;
    }
    if !OPENS_A_FRAME.contains(words[0]) {
        return false;
    }
    if !words.iter().any(|w| PICKING.contains(*w)) || !TO_A_VERB.is_match(text_) {
        return false;
    }
    !asks.is_empty() || words.iter().any(|w| PICKING_QUALITY.contains(*w))
}

/// `classify(line)`: what kind of line it is and what it asks for.
pub fn classify(line: &Line) -> Read {
    let text_ = line.norm_text.as_str();
    let norm = &line.norm;
    let words = sv(norm);
    let signals = &line.signals;
    let clauses = &line.clauses;
    let spaced = format!(" {text_} ");
    let opens_like_question = clauses.iter().enumerate().any(|(at, c)| _opens_question(c) && !_continues_the_sentence(clauses, at));
    let mut asks = _asks(norm, &spaced);
    let turned_back = _asks_it_back(norm) || _asks_about_the_bot_day(norm);
    let tag = _asks_for_confirmation(norm);
    let mut kind = "statement";
    let courtesy = without_filler(norm);
    if norm.is_empty() {
        kind = "empty";
    } else if signals.question_mark
        || _opens_question(norm)
        || opens_like_question
        || _asks_the_bot(norm)
        || _asks_for_help(text_)
        || tag
        || _asks_after_any(norm)
        || (!asks.is_empty() && _starts_with(norm, &THANKS) && (signals.question_mark || any_in(&words, &QUESTION_WORDS) || _thanks_still_asks(norm)))
    {
        kind = "question";
    } else if _only_the_ask(norm, &asks, text_) {
        kind = "question";
        let mut set: BTreeSet<String> = asks.into_iter().collect();
        set.insert("recommend".to_string());
        asks = set.into_iter().collect();
    } else if _starts_with(norm, &GREETINGS) && greeting_only(norm) {
        kind = "greeting";
    } else if (_starts_with(courtesy, &THANKS) && _thanks_only(courtesy, Some(clauses))) || _says_the_help_was_good(norm) {
        kind = "thanks";
    } else if _ends_with_thanks(courtesy) {
        kind = "thanks";
    }
    if turned_back {
        asks = vec![];
    }
    if kind != "question" {
        let positions: BTreeSet<usize> = background_clause(norm).union(&background_positions(norm)).copied().collect();
        let behind: HashSet<&str> = positions.into_iter().map(|k| norm[k].as_str()).collect();
        asks.retain(|a| !behind.contains(a.as_str()));
    }
    if kind == "question" && ((!turned_back && clauses.iter().any(|c| strip_leaders(c).first().is_some_and(|w| YESNO_STARTS.contains(w)))) || tag) {
        let mut set: BTreeSet<String> = asks.into_iter().collect();
        set.insert("yesno".to_string());
        asks = set.into_iter().collect();
    }
    let short = norm.len() <= 3;
    let bare: Vec<&String> = norm.iter().filter(|w| !POLITE.contains(*w)).collect();
    let bare_set: HashSet<&str> = bare.iter().map(|w| w.as_str()).collect();
    let denied_bad: HashSet<&str> = words.iter().copied().filter(|w| NEGATIVE.contains(*w) && denied(text_, w)).collect();
    let strip_norm = strip_leaders(norm);
    let intent: &str = if kind == "empty" {
        "empty"
    } else if signals.laughter && norm.len() <= 5 {
        "laugh"
    } else if _starts_with(strip_norm, &FAREWELLS) && farewell_only(strip_norm) {
        "farewell"
    } else if _about_bot(text_, &words, norm) {
        "about_bot"
    } else if kind == "greeting" {
        "greeting"
    } else if kind == "thanks" {
        "thanks"
    } else if _has_any(&spaced, &CORRECTION) && (words.contains("meant") || words.contains("mean")) {
        "correction"
    } else if _asks_it_back(norm) || _asks_about_the_bot_day(norm) || _asks_what_the_bot_is_doing(norm) || the_manners_question(norm) || asks_how_i_am(norm) {
        "smalltalk"
    } else if agreement_of(text_).is_some() {
        "ack"
    } else if _short_reply(text_) && norm.len() <= 6 {
        "ack"
    } else if kind == "statement" && _agrees_to_try(norm) {
        "ack"
    } else if matches!(norm[0].as_str(), "no" | "nah" | "nope" | "never") && norm.len() <= 5 && norm.get(1).map(String::as_str) != Some("one") {
        "deny"
    } else if ACK.contains(&norm[0]) && norm.len() <= 4 {
        "ack"
    } else if bare.len() <= 3 && bare_set.iter().all(|w| AFFIRM.contains(*w)) {
        "affirm"
    } else if bare.len() <= 3 && bare_set.iter().all(|w| DENY.contains(*w)) {
        "deny"
    } else if bare.len() <= 3 && (bare_set.iter().all(|w| ACK.contains(*w)) || bare_set.iter().all(|w| LAUGH.contains(*w))) {
        "ack"
    } else if small_talk_only(norm) {
        "smalltalk"
    } else if kind == "question" {
        "question"
    } else if asks.iter().any(|a| a == "recommend") || _asks_for_tips(text_, norm) || _has_any(text_, &STEPS) || any_of(&words, &["help", "need", "want"]) {
        "request"
    } else if matches!(norm[0].as_str(), "tell" | "explain" | "show" | "give" | "write" | "make") {
        "request"
    } else if words.contains("i") && (words.iter().any(|w| !denied_bad.contains(w) && (NEGATIVE.contains(*w) || POSITIVE.contains(*w))) || says_how_they_are(norm)) {
        "feeling"
    } else if (words.contains("i") || _acts_on_the_speaker(text_).is_some()) && emotion_of(text_).is_some() {
        "feeling"
    } else if _has_any(&spaced, &NEWS) {
        "news"
    } else if any_of(&words, &["i", "me", "my", "we", "our"]) && (any_in(&words, &EVENT_WORDS) || norm.len() <= 12) {
        "news"
    } else if _has_any(text_, &OPINION) || (words.contains("think") && kind == "question") {
        "opinion"
    } else {
        "statement"
    };
    let mut state_owner = _state_owner(clauses);
    let limit = _limit_of(text_);
    let suggestion = _a_suggestion_from_someone(norm, text_);
    let ruled_out = _ruled_out_of(text_);
    let mut emotion = None;
    if words.contains("i") || words.contains("im") || words.contains("my") || kind != "question" {
        emotion = emotion_of(text_);
    }
    let thingish = |o: &Option<String>| o.is_none() || o.as_deref() == Some("thing");
    if thingish(&state_owner) && _somebody_elses_heart(text_) {
        state_owner = Some("other".to_string());
    }
    if emotion.is_some() && thingish(&state_owner) && _a_broken_heart_is_theirs(text_) {
        state_owner = Some("self".to_string());
    }
    let hedge_text = if line.clean.is_empty() { text_ } else { line.clean.as_str() };
    if emotion.is_none() && _a_hedged_low(hedge_text) {
        emotion = Some("sad".to_string());
        if thingish(&state_owner) {
            state_owner = Some("self".to_string());
        }
    }
    let polarity = _polarity(norm);
    let shortage = shortage_of(text_);
    if shortage.is_some() && emotion.is_none() {
        state_owner = None;
    }
    let has_ask = |a: &str| asks.iter().any(|x| x == a);
    Read {
        kind: kind.to_string(),
        intent: intent.to_string(),
        tag,
        emotion,
        state_owner,
        shortage,
        polarity,
        short,
        first_person: any_of(&words, &["i", "me", "my", "we", "our", "us", "myself"]),
        asks_opinion: has_ask("opinion"),
        asks_recommendation: has_ask("recommend"),
        asks_steps: has_ask("steps"),
        help_ask: _asks_for_help(text_),
        asks_a_verdict: _asks_a_verdict(text_),
        about_bot: intent == "about_bot",
        about_bot_kind: if intent == "about_bot" { _about_bot_kind(text_, &words, norm).map(str::to_string) } else { None },
        counts_the_bots_own_day: _counts_its_own_doings(norm),
        replies_to_bot: matches!(intent, "affirm" | "deny" | "ack" | "laugh"),
        agrees_to_try: _agrees_to_try(norm),
        wants_a_quiz: _hands_a_question_to_the_speaker(norm),
        agreement: agreement_of(text_),
        summons: summons_of(norm, kind),
        asks_back: _asks_back(norm),
        asks_what_was_meant: _asks_what_was_meant(text_),
        excludes: limit.iter().filter(|l| !l.is_empty()).cloned().collect(),
        ruled_out,
        constraint: limit.is_some(),
        suggestion,
        asks,
    }
}

/// `shift_worked(text)`: "night", "late" or "early".
pub fn shift_worked(text_: &str) -> Option<String> {
    if SHIFT_NIGHT.is_match(text_) {
        return Some("night".to_string());
    }
    if SHIFT_LATE.is_match(text_) {
        return Some("late".to_string());
    }
    if SHIFT_EARLY.is_match(text_) {
        return Some("early".to_string());
    }
    None
}

/// `_a_suggestion_from_someone(norm, text)`: "my doctor said i should ...".
fn _a_suggestion_from_someone(norm: &[String], _text: &str) -> Option<Suggestion> {
    let said = format!(" {} ", norm.join(" "));
    let report = A_REPORTED_VERB.search(&said)?;
    if !(A_SUGGESTED_CLAUSE.search_at(&said, report.start()).is_some() || A_TOLD_TO.is_match(&said)) {
        return None;
    }
    if A_NOT_A_SUGGESTION.is_match(&said) {
        return None;
    }
    let before = &said[..report.start()];
    let mut named: Option<String> = None;
    for found in A_SAID_BY.finditer(before) {
        named = Some(text::strip(found.as_str()).to_string());
    }
    let who = match named.as_deref().filter(|n| !n.is_empty()) {
        Some(n) => {
            let mut words = text::split(n);
            if matches!(words[0].as_str(), "my" | "our") {
                words[0] = "your".to_string();
            }
            words.join(" ")
        }
        None => String::new(),
    };
    Some(Suggestion { who: text::upper_first(&who), source: named.unwrap_or_default() })
}

/// `is_a_routine(line, read=None)`: "i walk the dog every morning".
pub fn is_a_routine(line: &Line, read: Option<&dyn ReadView>) -> bool {
    if let Some(r) = read {
        if !matches!(r.kind(), Some("statement") | Some("news") | None) {
            return false;
        }
        if !r.asks().is_empty() {
            return false;
        }
    }
    let said = [line.plain.as_str(), line.clean.as_str(), line.text.as_str()].into_iter().find(|s| !s.is_empty()).unwrap_or("");
    let said = text::strip(said);
    if said.is_empty() || said.ends_with('?') {
        return false;
    }
    if _ROUTINE_SUBJECT.match_(said).is_none() {
        return false;
    }
    if ROUTINE_IS_A_FAULT.is_match(said) || ROUTINE_IN_THE_PAST.is_match(said) {
        return false;
    }
    ROUTINE_SHAPE.is_match(said)
}

/// `a_home_carried_forward(text, earlier_lines)`: "can i bring her anything" after a line about
/// mum in a care home.
pub fn a_home_carried_forward(text_: &str, earlier_lines: &[String]) -> Option<CareHome> {
    let line = rx("['\u{2019}]").sub("", text_);
    let line = text::strip(&line);
    if line.is_empty() || a_care_home(line).is_some() {
        return None;
    }
    if !(CARE_HOME_BRING_ASKS.is_match(line) && CARE_HOME_BRING_WORD.is_match(line) && CARE_HOME_BRING_WHO.is_match(line)) {
        return None;
    }
    for earlier in earlier_lines.iter().rev() {
        if let Some(mut read) = a_care_home(earlier) {
            read.shape = "bring".to_string();
            read.carried = Some(earlier.clone());
            return Some(read);
        }
    }
    None
}

/// `a_care_home(text)`: someone close in a care home, and how the line is about it.
pub fn a_care_home(text_: &str) -> Option<CareHome> {
    let line = rx("['\u{2019}]").sub("", text_);
    let line = text::strip(&line);
    if line.is_empty() || !CARE_HOME_PLACE.is_match(line) || CARE_HOME_NOT.is_match(line) {
        return None;
    }
    let lowered = line.to_lowercase();
    let mut who = String::new();
    let mut relation: Option<String> = None;
    for m in rx(r"\b(?:my|our|your)\s+([a-z][a-z-]*)").finditer(&lowered) {
        let g = m.str(1);
        if RELATIONS.contains(g) || CARE_HOME_KIN.contains(g) {
            relation = Some(g.to_string());
            who = text::squash(m.as_str());
            break;
        }
    }
    if who.is_empty() {
        for word in ["shes", "hes", "she", "he", "they", "theyre"] {
            if rx(&format!(r"\b{word}\b")).is_match(&lowered) {
                who = word.to_string();
                break;
            }
        }
    }
    if who.is_empty() {
        return None;
    }
    let rel = relation.as_deref().unwrap_or("\u{0}");
    const HE: &[&str] = &["husband", "father", "dad", "son", "brother", "uncle", "cousin", "grandad", "grandpa", "grandfather", "granddad", "partner", "boyfriend", "friend", "male friend", "nephew", "stepdad", "stepfather"];
    const SHE: &[&str] = &["wife", "mother", "mum", "mom", "daughter", "sister", "aunt", "gran", "granny", "grandma", "grandmother", "partner", "girlfriend", "friend", "niece", "stepmum", "stepmother", "nan", "nana", "mam", "mammy", "grannie", "mother-in-law", "sister-in-law", "daughter-in-law"];
    const THEY: &[&str] = &["kids", "children", "child", "parents", "friends", "mates", "mate"];
    let mut pronoun = if HE.contains(&rel) {
        "he"
    } else if SHE.contains(&rel) {
        "she"
    } else if THEY.contains(&rel) {
        "they"
    } else if who == "he" || who == "hes" {
        "he"
    } else if who == "she" || who == "shes" {
        "she"
    } else {
        "they"
    };
    if matches!(rel, "kids" | "children" | "parents" | "friends") {
        pronoun = "they";
    }
    let shape = if CARE_HOME_GUILT.is_match(&lowered) {
        "guilt"
    } else if CARE_HOME_VISIT.is_match(&lowered) {
        "visiting"
    } else if CARE_HOME_MOVED.is_match(&lowered) {
        "moved"
    } else {
        "in"
    };
    Some(CareHome {
        text: text::squash(CARE_HOME_PLACE.search(line).unwrap().as_str()),
        who,
        relation,
        pronoun: pronoun.to_string(),
        shape: shape.to_string(),
        carried: None,
    })
}

/// `_a_line_asks(read, said)`: the line asks something (for over_time).
fn _a_line_asks(read: &dyn ReadView, said: &str) -> bool {
    if read.kind() == Some("question") {
        return true;
    }
    let commas = said.replace(" ,", ",");
    if _A_REPORTS_THE_ASK.is_match(said) {
        return said.contains('?') || _ASKS_WITH_A_QUESTION_WORD.is_match(said) || _ASKS_WITH_AN_AUXILIARY.match_(text::strip(said)).is_some() || _ASKS_AFTER_A_COMMA_WORD.is_match(&commas);
    }
    if !read.asks().is_empty() {
        return true;
    }
    if read.intent() == Some("question") {
        return true;
    }
    if said.contains('?') {
        return true;
    }
    _ASKS_WITH_A_QUESTION_WORD.is_match(said) || _ASKS_MID_LINE.is_match(said) || _ASKS_AFTER_A_COMMA.is_match(&commas) || _ASKS_WITH_AN_AUXILIARY.match_(text::strip(said)).is_some()
}

/// `_as_typed(line)`.
fn _as_typed(line: &Line) -> String {
    let said = if line.text.is_empty() { &line.clean } else { &line.text };
    format!(" {} ", text::strip(said).to_lowercase())
}

/// `_carried_on(read, previous, words)`: a short follow-up carries on the last line's live ask.
fn _carried_on(read: &dyn ReadView, previous: Option<&Understood>, words: &[&str]) -> Option<(Option<bool>, Option<String>)> {
    let previous = previous?;
    if previous.over_time != Some(true) {
        return None;
    }
    if !read.followup() || words.len() as i64 > _A_FOLLOW_UP_WORDS {
        return None;
    }
    if !previous.text.is_empty() && Some(previous.text.as_str()) == read.text() {
        return None;
    }
    Some((Some(true), previous.over_time_what.clone()))
}

/// `_text(line)`.
fn _text(line: &Line) -> String {
    let said = if line.plain.is_empty() { &line.clean } else { &line.plain };
    format!(" {} ", text::strip(said).to_lowercase())
}

/// `over_time(line, read, previous=None, carrying_on=False)`: whether the answer changes over
/// time (prices, weather, opening hours) and what kind of live thing it is.
pub fn over_time(line: &Line, read: &dyn ReadView, previous: Option<&Understood>, carrying_on: bool) -> Option<(Option<bool>, Option<String>)> {
    let text_ = _text(line);
    let text_ = text_.as_str();
    let asking = _a_line_asks(read, text_);
    let words: Vec<&str> = text::split_ref(text_);
    let bare = words.len() as i64 <= _A_BARE_ASK;
    let no = Some((Some(false), None));
    let unknown = Some((None, None));
    let sealed = read.asks().iter().any(|a| _SEALED_ASKS.contains(a));
    if asking && _A_HOW_IT_WORKS.is_match(text_) && !_A_SAYS_NOW.is_match(text_) {
        return no;
    }
    if _A_ARITHMETIC.is_match(text_) || _A_ARITHMETIC.is_match(&_as_typed(line)) {
        return no;
    }
    if sealed && _A_HAS_A_NUMBER.is_match(text_) && !_A_AN_AMOUNT_OF_A_THING.is_match(text_) {
        return no;
    }
    if _A_PAST_YEAR.is_match(text_) || _A_HOW_OLD_IS_THE_WORLD.is_match(text_) || _A_HISTORY_ITSELF.is_match(text_) {
        return no;
    }
    if _A_A_DIFFERENCE.is_match(text_) {
        return no;
    }
    if _A_HOW_A_THING_MEASURES.is_match(text_) && !_A_WEATHER_HERE.is_match(text_) {
        return no;
    }
    if _A_A_TRAFFIC_LIGHT.is_match(text_) {
        return unknown;
    }
    if _A_A_PLAN_ABOUT_THE_WEATHER.is_match(text_) && !_A_WEATHER_HERE.is_match(text_) {
        return unknown;
    }
    if _A_HOW_MUCH_IT_HOLDS.is_match(text_) || _A_HOW_MUCH_IS_IN.is_match(text_) || _A_HOW_MUCH_ENOUGH.is_match(text_) || _A_HOW_MUCH_A_FEELING.is_match(text_) {
        return no;
    }
    if _A_AN_OCCASION_OF_THEIR_OWN.is_match(text_) {
        return unknown;
    }
    if !_MONEY.is_match(text_) && (_A_A_CONVERSION.is_match(text_) || _A_HOW_MUCH_OF_A_UNIT.is_match(text_)) {
        return no;
    }
    if _A_HOW_MUCH_I_SPENT.is_match(text_) || _A_MY_OWN_FIGURE.is_match(text_) {
        return unknown;
    }
    if _A_THE_CLOCK.is_match(text_) || _A_DID_YOU_SEE.is_match(text_) {
        return unknown;
    }
    if _A_BARE_RECORD.is_match(text_) && !asking {
        return no;
    }
    let fresh = _A_FRESHENING.sub(" ", text_);
    if _A_WORD_MEANS.is_match(text_) {
        return no;
    }
    for (what, pattern) in _A_BARE_LIVE.iter() {
        if pattern.is_match(&fresh) && !_A_REPORTS.is_match(text_) && words.len() as i64 <= _A_BARE_LIVE_WORDS {
            return Some((Some(true), Some(what.clone())));
        }
    }
    for (what, pattern) in _CHANGING.iter() {
        let Some((start, end)) = pattern.search(&fresh) else { continue };
        if matches!(what.as_str(), "hours" | "timetable" | "event") && _A_MY_OWN_PLAN.is_match(text_) {
            continue;
        }
        let head = text::strip(&fresh[..start]);
        if _A_DEFINITION.is_match(text_) && matches!(head, "what is" | "what are" | "whats" | "what's") && text::strip_chars(&fresh[end..], " ?!.,").is_empty() {
            return no;
        }
        if asking {
            return Some((Some(true), Some(what.clone())));
        }
        if bare && !_A_REPORTS.is_match(text_) {
            let tail = text::strip(&fresh[end..]);
            if _A_DETERMINER.contains(head) && (tail.is_empty() || _A_BARE_TAIL.match_(tail).is_some()) {
                return Some((Some(true), Some(what.clone())));
            }
        }
    }
    if carrying_on {
        return _carried_on(read, previous, &words);
    }
    if !asking {
        return unknown;
    }
    if sealed {
        return no;
    }
    if _SETTLED.iter().any(|p| p.is_match(text_)) {
        return no;
    }
    unknown
}

fn s_a_broken_heart_shape(_: &crate::data::FnRef, line: &str) -> bool {
    a_broken_heart_shape(line)
}

/// The figure-shape readers kinds.py defines that entries name (`Entry.shape`).
pub const SHAPES: &[(&str, crate::knowledge::registry::ShapeFn)] = &[("bot.understanding.kinds:a_broken_heart_shape:1676", s_a_broken_heart_shape)];


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

    /// Python: bot/understanding/kinds.py:377 `_a_class_of_thing()` (never called on the trace corpus)
    pub fn _a_class_of_thing() -> Value {
        crate::unported!("bot.understanding.kinds:_a_class_of_thing")
    }

}

