//! Port of `bot/understanding/normalize.py`: the line cleaned, respelled and cut into clauses.
//!
//! `normalise(text)` is the first thing the engine does to a line: accents off, glued and
//! run-on words split, chat-speak and contractions spelled out, typos mended where one reading
//! is clear, the line cut into clauses, and the clause that asks found. Every table is PyBot's
//! own, read from the data bundle by Python name.

#![allow(non_upper_case_globals)]

use crate::understanding::lexicon;
use crate::understanding::pyuni;
use crate::understanding::{rx, t};
use crate::understood::{Kept, Signals};
use crate::py::re::{Match, Re};
use crate::py::text;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::{LazyLock, RwLock};

const M: &str = "bot.understanding.normalize.";
const LX: &str = "bot.understanding.lexicon.";

/// What `normalise(text)` returns (Python's dict, same keys).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Line {
    pub text: String,
    pub clean: String,
    pub words: Vec<String>,
    pub plain: String,
    pub norm: Vec<String>,
    pub norm_text: String,
    pub fixes: IndexMap<String, String>,
    pub intensifier: Vec<String>,
    pub intensifier_left: Vec<String>,
    pub clauses: Vec<Vec<String>>,
    pub focus: Vec<String>,
    pub asking: Vec<String>,
    pub signals: Signals,
    pub dropped_subject: bool,
    pub change_of_subject: bool,
    pub kept: Option<Kept>,
}

// ---- PyBot's tables (bot.understanding.normalize.*) ----
static ABOUT_THE_FORM: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ABOUT_THE_FORM"));
static AFTER_AN_INVERTED_AUX: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AFTER_AN_INVERTED_AUX"));
static AFTER_A_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AFTER_A_STATE"));
static AFTER_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AFTER_A_THING"));
static AFTER_A_TIME: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AFTER_A_TIME"));
static AFTER_A_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AFTER_A_VERB"));
static AINT_FORMS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AINT_FORMS"));
static AINT_PLURAL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AINT_PLURAL"));
static AMOUNT_BEFORE_A_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AMOUNT_BEFORE_A_STATE"));
static APOSTROPHE_BASES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "APOSTROPHE_BASES"));
static APOSTROPHE_NOT: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| t::tuples(M, "APOSTROPHE_NOT"));
static APOSTROPHE_TAILS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "APOSTROPHE_TAILS"));
static ASKS_WHOSE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASKS_WHOSE"));
static ASK_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_NOUNS"));
static ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_WORDS"));
static AUX_OPENS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AUX_OPENS"));
static AUX_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AUX_WORDS"));
static A_CHOKE_SAID_THE_OTHER_WAY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_CHOKE_SAID_THE_OTHER_WAY"));
static A_COPULA: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_COPULA"));
static A_COUNT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_COUNT"));
static A_FEELING_CAN_BE_FOLLOWED: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_FEELING_CAN_BE_FOLLOWED"));
static A_GLASS_OR_A_PAN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_GLASS_OR_A_PAN"));
static A_JOB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_JOB"));
static A_KEEPING_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_KEEPING_A_THING"));
static A_LABEL_BEFORE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_LABEL_BEFORE"));
static A_LETTER_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_LETTER_AFTER"));
static A_NEGATION_BEFORE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_NEGATION_BEFORE"));
static A_NUMBER_OF_SOMEONES_DAY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_NUMBER_OF_SOMEONES_DAY"));
static A_PAIN_WORD: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "A_PAIN_WORD"));
static A_PARTS_OF_A_PERIOD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_PARTS_OF_A_PERIOD"));
static A_PART_A_PERSON_OWNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_PART_A_PERSON_OWNS"));
static A_PART_THE_PAIN_IS_IN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_PART_THE_PAIN_IS_IN"));
static A_PERIOD_FOLLOWS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_PERIOD_FOLLOWS"));
static A_PRONOUN_GLUED: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "A_PRONOUN_GLUED"));  // t, 7 items
static A_PRONOUN_STATE: LazyLock<IndexMap<String, HashSet<String>>> = LazyLock::new(|| t::map_set(M, "A_PRONOUN_STATE"));
static A_SIDE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_SIDE"));
static A_STATE_THEN_A_TIME: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_STATE_THEN_A_TIME"));
static A_STATE_YOU_CAN_FEEL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_STATE_YOU_CAN_FEEL"));
static A_THING_GOES_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_THING_GOES_AFTER"));
static A_THING_TYPED_AS_A_LITTLE_WORD: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "A_THING_TYPED_AS_A_LITTLE_WORD"));
static A_THING_WORN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_THING_WORN"));
static A_TIME_TO_LEAVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_TIME_TO_LEAVE"));
static A_VERB_CAN_FOLLOW: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_VERB_CAN_FOLLOW"));
static BARE_TO: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BARE_TO"));
static BASE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BASE"));
static BEEN_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BEEN_SUBJECT"));
static BEEN_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BEEN_WORDS"));
static BEFORE_A_FEELING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BEFORE_A_FEELING"));
static BEFORE_A_JOB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BEFORE_A_JOB"));
static BEFORE_A_PREPOSITION: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BEFORE_A_PREPOSITION"));
static BEFORE_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BEFORE_A_THING"));
static BEFORE_A_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BEFORE_A_VERB"));
static BEFORE_THE_LOSING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BEFORE_THE_LOSING"));
static BEFORE_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BEFORE_VERB"));
static BEFORE_WORD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BEFORE_WORD"));
static BE_BEFORE_A_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BE_BEFORE_A_STATE"));
static BE_FORMS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BE_FORMS"));
static BE_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BE_VERBS"));
static BE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BE_WORDS"));
static BIN_CONTAINERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BIN_CONTAINERS"));
static BREAKS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BREAKS"));
static CAN_FOLLOW_A_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CAN_FOLLOW_A_VERB"));
static CAN_NOT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CAN_NOT"));
static CAUSES_A_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CAUSES_A_STATE"));
static CAUSE_A_CLAUSE_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CAUSE_A_CLAUSE_VERB"));
static CAUSE_A_DEMONSTRATIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CAUSE_A_DEMONSTRATIVE"));
static CAUSE_A_SUBJECT_FOLLOWS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CAUSE_A_SUBJECT_FOLLOWS"));
static CAUSE_BEFORE_A_NOUN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CAUSE_BEFORE_A_NOUN"));
static CAUSE_BEFORE_A_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CAUSE_BEFORE_A_VERB"));
static CHANGE_OF_SUBJECT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CHANGE_OF_SUBJECT"));
static CHAT_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CHAT_NAMES"));
static CLAUSE_OPENERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CLAUSE_OPENERS"));
static CLIPPED_FAMILY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CLIPPED_FAMILY"));
static CLIPPED_THINGS: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| t::map_strs(M, "CLIPPED_THINGS"));
static CLIPPED_UNITS: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| t::map_strs(M, "CLIPPED_UNITS"));
static COMMON_RANK: LazyLock<IndexMap<String, i64>> = LazyLock::new(|| t::map_int(M, "COMMON_RANK"));
static COMPOUND_HEADS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COMPOUND_HEADS"));
static COMPOUND_TAILS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COMPOUND_TAILS"));
static CONTRACTION_NEXT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CONTRACTION_NEXT"));
static COPULAS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COPULAS"));
static CORE_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CORE_VERBS"));
static COUNTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COUNTS"));
static COUNT_UNITS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COUNT_UNITS"));
static CURRENCY_FACTOR: LazyLock<IndexMap<String, i64>> = LazyLock::new(|| t::map_int(M, "CURRENCY_FACTOR"));
static CURRENCY_SIGN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CURRENCY_SIGN"));
static CURRENCY_SIGNS: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "CURRENCY_SIGNS"));
static DAY_NAMED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DAY_NAMED"));
static DEMONSTRATIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DEMONSTRATIVE"));
static DETERMINERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DETERMINERS"));
static DETERMINER_IN_FRONT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DETERMINER_IN_FRONT"));
static DIGIT_WORDS: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "DIGIT_WORDS"));
static DOCS_ON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DOCS_ON"));
static DOCS_PAPER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DOCS_PAPER"));
static DOCS_SCREEN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DOCS_SCREEN"));
static DOCS_VISIT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DOCS_VISIT"));
static DOING_NOT_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DOING_NOT_THING"));
static DROPPED_SUBJECT_AUX: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DROPPED_SUBJECT_AUX"));
static EXCLAIMED_AFTER_A_QUESTION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "EXCLAIMED_AFTER_A_QUESTION"));
static EXCLAMATION_OPENS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "EXCLAMATION_OPENS"));
static EXCLAMATION_PHRASE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "EXCLAMATION_PHRASE"));
static E_KEPT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "E_KEPT_WORDS"));
static FAMILY_SHRT: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| t::map_strs(M, "FAMILY_SHRT"));
static FEELING_BEFORE_SICK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FEELING_BEFORE_SICK"));
static FINDING_A_TEAM: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FINDING_A_TEAM"));
static FIRST_PERSON_BE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FIRST_PERSON_BE"));
static FOOD_SLOT_LEAD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FOOD_SLOT_LEAD"));
static FOR_A: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FOR_A"));
static FOR_MEAL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FOR_MEAL"));
static FOR_OBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FOR_OBJECT"));
static FREQUENT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FREQUENT"));
static F_TYPED_FOR_TH: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "F_TYPED_FOR_TH"));
static GLUED_AFTER_A_DO: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GLUED_AFTER_A_DO"));
static GLUED_COLLAPSE: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| t::map_strs(M, "GLUED_COLLAPSE"));
static GLUED_DO: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GLUED_DO"));
static GLUED_LEFT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GLUED_LEFT"));
static GLUED_RIGHT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GLUED_RIGHT"));
static HAS_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HAS_AFTER"));
static HAS_FORMS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HAS_FORMS"));
static HAS_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HAS_SUBJECT"));
static HAS_SUBJECTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HAS_SUBJECTS"));
static HAVE_FORMS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HAVE_FORMS"));
static HEART_SAID_AS_A_FEELING: LazyLock<Vec<(&'static Re, String)>> = LazyLock::new(|| t::re_str_pairs(M, "HEART_SAID_AS_A_FEELING"));
static HELLO_SAID_SHORT: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| t::tuples(M, "HELLO_SAID_SHORT"));
static HELLO_TAIL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HELLO_TAIL"));
static HOW_DEGREE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HOW_DEGREE"));
static HOW_FOLLOWS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HOW_FOLLOWS"));
static HR_IS_THE_DEPARTMENT_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HR_IS_THE_DEPARTMENT_AFTER"));
static HV_HELPERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HV_HELPERS"));
static HV_SUBJECTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HV_SUBJECTS"));
static ILL_GOES_ON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ILL_GOES_ON"));
static ING_THINGS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ING_THINGS"));
static INTRODUCING_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "INTRODUCING_AFTER"));
static INTRODUCING_A_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "INTRODUCING_A_PERSON"));
static IN_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "IN_NOUNS"));
static IRREGULAR_PARTICIPLES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "IRREGULAR_PARTICIPLES"));
static I_GLUED: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "I_GLUED"));
static JOINED_WORDS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "JOINED_WORDS"));
static KIN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "KIN"));
static KIN_FILLERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "KIN_FILLERS"));
static KIN_HAS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "KIN_HAS"));
static KIN_IS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "KIN_IS"));
static KIN_SINGULAR: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "KIN_SINGULAR"));
static KNOW_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "KNOW_SUBJECT"));
static LEADERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LEADERS"));
static LEFT_OFF_CONTRACTION: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| t::map_strs(M, "LEFT_OFF_CONTRACTION"));
static LETTER_C_IS_SEE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LETTER_C_IS_SEE"));
static LEXICON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LEXICON"));
static LIKE_STARTS_A_CLAUSE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LIKE_STARTS_A_CLAUSE"));
static LONG_ENOUGH_TO_BE_A_VERB: LazyLock<IndexMap<String, i64>> = LazyLock::new(|| t::map_int(M, "LONG_ENOUGH_TO_BE_A_VERB"));
static LOOSE_FOR_LOSE: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "LOOSE_FOR_LOSE"));
static MEAL_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MEAL_WORDS"));
static MISSING_E_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MISSING_E_AFTER"));
static MI_BACK_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MI_BACK_VERBS"));
static MI_IS_A_NAME_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MI_IS_A_NAME_AFTER"));
static MI_IS_ME_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MI_IS_ME_AFTER"));
static MI_IS_ME_BEFORE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MI_IS_ME_BEFORE"));
static MODAL_FORMS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MODAL_FORMS"));
static MOTION_BEFORE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MOTION_BEFORE"));
static MRS_IS_THE_WIFE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MRS_IS_THE_WIFE"));
static NAMED_LETTER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NAMED_LETTER"));
static NAME_CALLS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NAME_CALLS"));
static NAME_FRAMES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NAME_FRAMES"));
static NAME_ROLL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NAME_ROLL"));
static NOMINAL_BEFORE_COPULA: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOMINAL_BEFORE_COPULA"));
static NOT_A_COMPOUND_HEAD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_COMPOUND_HEAD"));
static NOT_A_COMPOUND_TAIL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_COMPOUND_TAIL"));
static NOT_A_MEAL_TYPO: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_MEAL_TYPO"));
static NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_THING"));
static NOT_A_THING_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_THING_AFTER"));
static NOT_A_THING_ENDING: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "NOT_A_THING_ENDING"));  // t, 2 items
static NOT_A_THING_PAIR: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_THING_PAIR"));
static NOT_BEFORE_A_NOUN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_BEFORE_A_NOUN"));
static NOT_BEFORE_KNOW: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_BEFORE_KNOW"));
static NOT_COUNTED_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_COUNTED_AFTER"));
static NOT_IN_A_QUESTION_PHRASE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_IN_A_QUESTION_PHRASE"));
static NOT_MY_OWN_DAY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_MY_OWN_DAY"));
static NOT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_WORDS"));
static NO_ASK_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NO_ASK_AFTER"));
static NO_SWAP_TO: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NO_SWAP_TO"));
static NUMBERS_BEFORE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NUMBERS_BEFORE"));
static OBJECT_PRONOUN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OBJECT_PRONOUN"));
static ONE_OF_A_KIND: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ONE_OF_A_KIND"));
static ONE_PART_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ONE_PART_AFTER"));
static ONE_RELATION: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ONE_RELATION"));
static ONE_RELATION_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ONE_RELATION_VERBS"));
static ONE_WORD_FOODS: LazyLock<Vec<(&'static Re, String)>> = LazyLock::new(|| t::re_str_pairs(M, "ONE_WORD_FOODS"));
static OPENERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OPENERS"));
static OPERATORS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OPERATORS"));
static OWNER_WITHOUT_THE_DAY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "OWNER_WITHOUT_THE_DAY"));
static OWNING_A_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OWNING_A_PERSON"));
static OWNING_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OWNING_VERBS"));
const PART_REACH: i64 = 3;
static PART_THAT_IS_BAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PART_THAT_IS_BAD"));
static PAST_AS_PARTICIPLE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PAST_AS_PARTICIPLE"));
static PERIOD_COUNTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PERIOD_COUNTS"));
static PERSON_PREPS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PERSON_PREPS"));
static PERSON_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PERSON_WORDS"));
static PHRASES: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| t::tuples(M, "PHRASES"));
static PHRASE_MEASURES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PHRASE_MEASURES"));
static PLACE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLACE_WORDS"));
static PLURAL_SUBJECTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLURAL_SUBJECTS"));
static POSSESSIVES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "POSSESSIVES"));
static POSSESSIVE_IN_FRONT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "POSSESSIVE_IN_FRONT"));
static PRONOUN_BASES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PRONOUN_BASES"));
static PUNCT_BREAK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PUNCT_BREAK"));
static RAW_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RAW_WORD"));
static REASON_STARTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "REASON_STARTS"));
static RELATIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RELATIVE"));
static RELATIVE_ASKS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RELATIVE_ASKS"));
static REPEAT_ONCE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "REPEAT_ONCE"));
static REPEAT_ONCE_SUBJECTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "REPEAT_ONCE_SUBJECTS"));
static REPORTING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "REPORTING"));
static RUN_ON_LAST_PIECES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RUN_ON_LAST_PIECES"));
static RUN_ON_MINE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RUN_ON_MINE"));
static RUN_ON_OPENERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RUN_ON_OPENERS"));
static RUN_ON_PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RUN_ON_PEOPLE"));
static RUN_ON_SELF: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RUN_ON_SELF"));
const RUN_ON_SELF_LONGEST: i64 = 9;
const RUN_ON_SHORTEST: i64 = 8;
static RUN_ON_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RUN_ON_WORDS"));
static SAID_APART: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SAID_APART"));
static SAID_AS_A_FEELING: LazyLock<Vec<(&'static Re, String)>> = LazyLock::new(|| t::re_str_pairs(M, "SAID_AS_A_FEELING"));
static SAID_IT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SAID_IT"));
static SAID_OUT: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "SAID_OUT"));
static SENTENCE_END: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SENTENCE_END"));
static SHION_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SHION_WORDS"));
static SHRT_THINGS: LazyLock<IndexMap<String, HashSet<String>>> = LazyLock::new(|| t::map_set(M, "SHRT_THINGS"));
static SICK_AS_FEELING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SICK_AS_FEELING"));
static SOLFEGE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SOLFEGE"));
static SON_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SON_VERBS"));
static SORE_FILLERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SORE_FILLERS"));
static SORE_IN_FRONT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SORE_IN_FRONT"));
static SORE_PARTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SORE_PARTS"));
static SPELLING_ASK: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SPELLING_ASK"));
static SPELLING_ENDINGS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "SPELLING_ENDINGS"));  // t, 8 items
static SPLIT_CONTRACTION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SPLIT_CONTRACTION"));
static SPLIT_NOT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SPLIT_NOT"));
static STATES_OF_A_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STATES_OF_A_PERSON"));
static STOPPED_TH_STEMS: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "STOPPED_TH_STEMS"));
static SUBJECT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUBJECT_WORDS"));
static SUGAR_AFTER_ROSE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUGAR_AFTER_ROSE"));
static SUGAR_FIRST_PERSON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SUGAR_FIRST_PERSON"));
static SUGAR_LEVEL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUGAR_LEVEL"));
static SUGAR_PRICE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUGAR_PRICE"));
static SUGAR_PRICE_SHAPE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SUGAR_PRICE_SHAPE"));
static SUM_STAYS_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUM_STAYS_AFTER"));
static SUM_THE_NOUN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUM_THE_NOUN"));
static SUN_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUN_WORDS"));
static TAG_GUARD_BEFORE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TAG_GUARD_BEFORE"));
static TAG_PLEASE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TAG_PLEASE"));
static TAG_QUESTION_LEADS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TAG_QUESTION_LEADS"));
static TAG_THANKS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TAG_THANKS"));
static TAG_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TAG_WORDS"));
static TAIL_OF_A_DAY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TAIL_OF_A_DAY"));
static THE_HEART_SAID_THE_OTHER_WAY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THE_HEART_SAID_THE_OTHER_WAY"));
static THE_LOSING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THE_LOSING"));
static THE_ODD_PLURALS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THE_ODD_PLURALS"));
static THING_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THING_AFTER"));
static THING_DETERMINER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THING_DETERMINER"));
static THING_JOIN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THING_JOIN"));
static THING_PLACE_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THING_PLACE_AFTER"));
static THING_PLACE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THING_PLACE_WORDS"));
static THING_POSSESSIVES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THING_POSSESSIVES"));
static THING_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THING_SUBJECT"));
static THIRD_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THIRD_PERSON"));
static THIRD_PERSON_BE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THIRD_PERSON_BE"));
static THOUSANDS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "THOUSANDS"));
static THOUSANDS_SAID_WITH_A_SPACE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "THOUSANDS_SAID_WITH_A_SPACE"));
static TIMES_A_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TIMES_A_STATE"));
static TIME_AFTER_MEAL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TIME_AFTER_MEAL"));
static TIME_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TIME_WORDS"));
static TOKEN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TOKEN"));
static TOO_A: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TOO_A"));
static TOO_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TOO_AFTER"));
static TOO_A_GRADED: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TOO_A_GRADED"));
static TO_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TO_AFTER"));
static UP_TOO_ASKS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "UP_TOO_ASKS"));
static UP_TO_LEADS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "UP_TO_LEADS"));
static UP_TO_SPELLINGS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "UP_TO_SPELLINGS"));
static UP_TO_TAGS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "UP_TO_TAGS"));
static URGENT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "URGENT_WORDS"));
static VEGGIES_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "VEGGIES_AFTER"));
static VOWEL_SET: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "VOWEL_SET"));
static VOWEL_SET_OR_Y: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "VOWEL_SET_OR_Y"));
static WANT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WANT_WORDS"));
static WEARS_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WEARS_AFTER"));
static WHAT_ARE_YOU_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WHAT_ARE_YOU_AFTER"));
static WHAT_ARE_YOU_SPELLINGS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WHAT_ARE_YOU_SPELLINGS"));
static WH_ANY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WH_ANY"));
static WH_INTENSIFIER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WH_INTENSIFIER"));
static WH_OPENERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WH_OPENERS"));
static WH_QUANTIFIERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WH_QUANTIFIERS"));
static WH_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WH_WORDS"));
static YOU_ARE_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "YOU_ARE_AFTER"));
static _AFTER_A_DOING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_AFTER_A_DOING"));
static _AFTER_A_LEAD_IN_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_AFTER_A_LEAD_IN_SUBJECT"));
static _AFTER_PARTICIPLE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_AFTER_PARTICIPLE"));
static _ANY_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_ANY_WORD"));
static _A_CLIPPED_HEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_CLIPPED_HEAD"));
static _A_HEAD_IN_THE_IDIOM: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_HEAD_IN_THE_IDIOM"));
static _A_QUESTION_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_QUESTION_VERB"));
static _A_SHORT_WORD_IS_NAMED_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_SHORT_WORD_IS_NAMED_AFTER"));
static _A_SHORT_WORD_IS_NAMED_BEFORE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_SHORT_WORD_IS_NAMED_BEFORE"));
static _A_STATE_OF_THE_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_STATE_OF_THE_PERSON"));
static _A_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_WORDS"));
static _DAY_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_DAY_WORDS"));
static _EXCLAMATION_SUBJECTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_EXCLAMATION_SUBJECTS"));
static _FEELING_WORD: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "_FEELING_WORD"));
static _HAVE_ASKING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_HAVE_ASKING"));
static _HAVE_NEGATIONS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_HAVE_NEGATIONS"));
static _HAVE_PARTICIPLES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_HAVE_PARTICIPLES"));
static _HAVE_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_HAVE_THING"));
static _KEEPING_PLACES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_KEEPING_PLACES"));
static _KEPT_ALREADY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_KEPT_ALREADY"));
static _KEPT_THINGS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_KEPT_THINGS"));
static _LEADER_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_LEADER_WORD"));
static _NUMBER_SHAPE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_NUMBER_SHAPE"));
static _PLACE_LEADS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_PLACE_LEADS"));
static _PLACE_STOP: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_PLACE_STOP"));
static _PREDICATE_ONLY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_PREDICATE_ONLY"));
static _SMALL_NUMBERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_SMALL_NUMBERS"));
static _SPACED_PIECE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_SPACED_PIECE"));
static _SPACED_RUN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_SPACED_RUN"));
static _STATE_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_STATE_FILLER"));
static _STATE_TAIL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_STATE_TAIL"));
static _SUBJECTLESS_AUX: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_SUBJECTLESS_AUX"));
static _SUBJECTLESS_ONLY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_SUBJECTLESS_ONLY"));
static _SUBJECTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_SUBJECTS"));
static _SUBJECT_SAID: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_SUBJECT_SAID"));
static _TYPED_SURPRISE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_TYPED_SURPRISE"));
static _WERE_ARE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_WERE_ARE"));
static _WERE_DOING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_WERE_DOING"));
static _WERE_IN_THE_PAST: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_WERE_IN_THE_PAST"));
static _WERE_LEAD_IN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_WERE_LEAD_IN"));
static _WERE_PLACE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_WERE_PLACE"));
static _WERE_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_WERE_STATE"));
static _WH_QUESTIONS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_WH_QUESTIONS"));

// ---- bot.understanding.lexicon tables ----
static LX_A_STATE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "A_STATE"));
static LX_BACKGROUND_STARTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "BACKGROUND_STARTS"));
static LX_BODY_PARTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "BODY_PARTS"));
static LX_CHAT: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(LX, "CHAT"));
static LX_CLAUSE_BREAKS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "CLAUSE_BREAKS"));
static LX_COMMON_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "COMMON_NAMES"));
static LX_EXPAND: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(LX, "EXPAND"));
static LX_FEELINGS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "FEELINGS"));
static LX_JOINED_WORDS: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(LX, "JOINED_WORDS"));
static LX_LAUGHTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "LAUGHTER"));
static LX_MISSPELLINGS: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(LX, "MISSPELLINGS"));
static LX_NOT_A_FEELING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "NOT_A_FEELING"));
static LX_PAIN_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "PAIN_WORDS"));
static LX_PLACE_PREPS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "PLACE_PREPS"));
static LX_PLAIN_EXPANDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "PLAIN_EXPANDS"));
static LX_REAL_IN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "REAL_IN"));
static LX_SAID_SHORT: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(LX, "SAID_SHORT"));
static LX_SOUNDS_LIKE_A_FEELING: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(LX, "SOUNDS_LIKE_A_FEELING"));
static LX_START_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "START_VERBS"));
static LX_STATES_SAID_ONTO_IM: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "STATES_SAID_ONTO_IM"));
static LX_STATES_WITHOUT_G: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "STATES_WITHOUT_G"));
static LX_TYPED_CONTRACTIONS: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(LX, "TYPED_CONTRACTIONS"));
static LX_TYPED_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(LX, "TYPED_WORDS"));
static LX_URGENT_MISSPELLINGS: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(LX, "URGENT_MISSPELLINGS"));
static LX_US_SPELLINGS: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(LX, "US_SPELLINGS"));
/// `lexicon.FEELINGS | lexicon.A_STATE`.
static LX_FEELINGS_OR_STATES: LazyLock<HashSet<String>> = LazyLock::new(|| LX_FEELINGS.union(&LX_A_STATE).cloned().collect());

// ---- tables that are not plain lists ----
/// `VOWELS` (a string of letters).
static VOWELS: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "VOWELS"));
static S_CLUSTERS: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "S_CLUSTERS"));
/// `THING_WORDS` / `THING_WORDS_BY_LENGTH`: `learn_thing_words` adds to them at run time.
static THING_WORDS: LazyLock<RwLock<HashSet<String>>> = LazyLock::new(|| RwLock::new(t::set(M, "THING_WORDS")));
static THING_WORDS_BY_LENGTH: LazyLock<RwLock<HashMap<usize, Vec<String>>>> = LazyLock::new(|| {
    // Python iterates each length's set; only "exactly one fits" is read, so order is free.
    let mut m: HashMap<usize, Vec<String>> = HashMap::new();
    for (k, v) in t::value(M, "THING_WORDS_BY_LENGTH").pairs() {
        let mut words = v.strs();
        words.sort();
        m.insert(k.as_int().unwrap_or(0) as usize, words);
    }
    RwLock::new(m)
});
/// `LETTERS`: the translate table `_unaccent` applies first (code point -> text).
static LETTERS: LazyLock<HashMap<char, String>> = LazyLock::new(|| {
    t::value(M, "LETTERS").pairs().iter().filter_map(|(k, v)| Some((char::from_u32(k.as_int()? as u32)?, v.str().to_string()))).collect()
});
/// `SAID_APART_FORMS`: the pieces a word is typed apart into -> the word.
static SAID_APART_FORMS: LazyLock<HashMap<Vec<String>, String>> =
    LazyLock::new(|| t::value(M, "SAID_APART_FORMS").pairs().iter().map(|(k, v)| (k.strs(), v.str().to_string())).collect());
/// `_A_CLIP`: typed -> (the longer word it was read as, the cues that must be there).
static _A_CLIP: LazyLock<Vec<(String, String, HashSet<String>)>> = LazyLock::new(|| {
    t::value(M, "_A_CLIP").pairs().iter().map(|(k, v)| (k.str().to_string(), v.items()[0].str().to_string(), v.items()[1].str_set())).collect()
});
/// `URGENT_BONES`: consonant skeleton -> the urgent words with it (filled on first use).
static URGENT_BONES: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| {
    let mut m: HashMap<String, Vec<String>> = HashMap::new();
    for known in URGENT_WORDS.iter() {
        m.entry(_bones(known)).or_default().push(known.clone());
    }
    m
});
/// `frozenset(bot.knowledge.geography.COUNTRIES)` (`_PLACES_SAID_WHOLE`).
static COUNTRIES: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let v = crate::data::get("bot.knowledge.geography.COUNTRIES");
    match v {
        crate::data::Value::Dict(d) => d.iter().filter_map(|(k, _)| k.as_str().map(str::to_string)).collect(),
        other => other.str_set(),
    }
});
/// `unicodedata.normalize("NFKD", c)` without combining marks, per character, after `LETTERS`
/// (CPython's Unicode 15.0 tables, exported to `unaccent.txt`).
static UNACCENT: LazyLock<HashMap<char, String>> = LazyLock::new(|| {
    include_str!("unaccent.txt")
        .lines()
        .filter_map(|line| {
            let (cp, rest) = line.split_once('\t')?;
            let c = char::from_u32(u32::from_str_radix(cp, 16).ok()?)?;
            let s: String = rest.split(' ').filter(|p| !p.is_empty()).filter_map(|p| char::from_u32(u32::from_str_radix(p, 16).ok()?)).collect();
            Some((c, s))
        })
        .collect()
});

// ---- small Python-string helpers (words here are ASCII after TOKEN, but these stay char-safe) ----

/// `len(s)` in characters.
fn n(s: &str) -> usize {
    if s.is_ascii() { s.len() } else { s.chars().count() }
}
/// `s[a:b]` with Python's character indices (negative from the end, clamped).
fn sl(s: &str, a: isize, b: isize) -> String {
    text::slice(s, a, Some(b))
}
/// `s[a:]`.
fn from(s: &str, a: isize) -> String {
    text::slice(s, a, None)
}
/// `s[:b]`.
fn upto(s: &str, b: isize) -> String {
    text::slice(s, 0, Some(b))
}
/// `s[i]` (None where Python raises IndexError).
fn at(s: &str, i: isize) -> Option<char> {
    let len = n(s) as isize;
    let i = if i < 0 { len + i } else { i };
    if i < 0 || i >= len {
        return None;
    }
    if s.is_ascii() { Some(s.as_bytes()[i as usize] as char) } else { s.chars().nth(i as usize) }
}
/// `s[i]`, raising like Python when out of range.
fn ix(s: &str, i: isize) -> char {
    at(s, i).unwrap_or_else(|| panic!("IndexError: string index out of range"))
}
/// `x in "aeiou"` for a one-character string (`"" in s` is True in Python).
fn in_str(c: Option<char>, s: &str) -> bool {
    match c {
        Some(c) => s.contains(c),
        None => true,
    }
}
/// `word in SET` where word may be None.
fn has(set: &HashSet<String>, w: Option<&str>) -> bool {
    w.is_some_and(|w| set.contains(w))
}
/// `word.isalpha()`.
fn alpha(s: &str) -> bool {
    pyuni::is_alpha(s)
}
/// `word.isdigit()`.
fn digit(s: &str) -> bool {
    pyuni::is_digit(s)
}
fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}
/// `s.split()` as owned words.
fn words_of(s: &str) -> Vec<String> {
    text::split(s)
}
/// `COMMON_RANK.get(w, 10 ** 6)`.
fn rank(w: &str) -> i64 {
    COMMON_RANK.get(w).copied().unwrap_or(1_000_000)
}
fn is_vowel(c: char) -> bool {
    VOWELS.contains(c)
}

// ---- respelling urgent words ----

/// `respell_urgent(text)`: urgent words spelled right ("anaphalaxis", "suicdal"), and the family
/// clips put back ("my dd" is my daughter), before the safety module reads the line.
pub fn respell_urgent(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let put = |m: &Match| -> String {
        let word = m.as_str();
        let low = word.to_lowercase();
        match LX_URGENT_MISSPELLINGS.get(&low) {
            None => {
                let guessed = fix_typo(&low);
                match guessed {
                    Some(g) if URGENT_WORDS.contains(&g) => g,
                    _ => word.to_string(),
                }
            }
            Some(fix) => {
                let keep = word.chars().next().is_some_and(|c| pyuni::is_upper(&c.to_string()));
                if keep { text::capitalize(fix) } else { fix.clone() }
            }
        }
    };
    let first = _ANY_WORD.sub_fn(text, put);
    CLIPPED_FAMILY.sub_fn(&first, _family_put_back)
}

/// `_family_put_back(match)`.
fn _family_put_back(m: &Match) -> String {
    let rest_said = m.name("rest").unwrap_or("");
    let rest = words_of(rest_said);
    let after = rest.first().map(|w| w.to_lowercase());
    let after_after = rest.get(1).map(|w| w.to_lowercase());
    let word = m.name_str("word").to_lowercase();
    let owner = m.name_str("owner").to_lowercase();
    match _family_short(&word, Some(&owner), after.as_deref(), after_after.as_deref()) {
        None => m.as_str().to_string(),
        Some(family) => format!("{} {}{}", m.name_str("owner"), family.join(" "), rest_said),
    }
}

/// `_doubles_its_last_letter(word)`: "stop" doubles its p before -ing.
fn _doubles_its_last_letter(word: &str) -> bool {
    if n(word) < 3 || !alpha(word) {
        return false;
    }
    if in_str(at(word, -3), "aeiou") || !in_str(at(word, -2), "aeiou") || in_str(at(word, -1), "aeiouwxy") {
        return false;
    }
    rx("[aeiou]+").findall(word).len() == 1
}

/// `_drops_a_final_e(word)`.
fn _drops_a_final_e(word: &str) -> bool {
    word.ends_with('e') && n(word) > 2
}

/// `_keeps_its_e_before_ing(word)`.
fn _keeps_its_e_before_ing(word: &str) -> bool {
    n(word) > 2 && (word.ends_with("ee") || word.ends_with("ye") || word.ends_with("oe"))
}

/// `_ing_form(word)`.
fn _ing_form(word: &str) -> String {
    if _keeps_its_e_before_ing(word) { format!("{word}ing") } else { format!("{}ing", upto(word, -1)) }
}

/// `_inflections(word)`: the endings a base word takes.
pub fn _inflections(word: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    if n(word) < 3 || !alpha(word) {
        return out;
    }
    let y_for_i = word.ends_with('y') && !VOWEL_SET.contains(&at(word, -2).map(String::from).unwrap_or_default());
    out.insert(format!("{word}ly"));
    if !y_for_i {
        out.insert(format!("{word}s"));
    }
    let last = at(word, -1).unwrap();
    let cut = upto(word, -1);
    if _doubles_its_last_letter(word) {
        out.insert(format!("{word}{last}ing"));
        out.insert(format!("{word}{last}ed"));
        out.insert(format!("{word}{last}er"));
    } else if _drops_a_final_e(word) {
        out.insert(_ing_form(word));
        out.insert(format!("{cut}ed"));
        out.insert(format!("{cut}er"));
    } else {
        out.insert(format!("{word}ing"));
        if !y_for_i {
            out.insert(format!("{word}ed"));
            out.insert(format!("{word}er"));
        }
    }
    if word.ends_with('e') {
        out.insert(format!("{cut}ing"));
        out.insert(format!("{cut}ed"));
    }
    if word.ends_with('y') {
        out.insert(format!("{cut}ies"));
        out.insert(format!("{cut}ied"));
        out.insert(format!("{cut}ier"));
    }
    if word.ends_with('s') {
        out.insert(cut);
    }
    out
}

/// `learn_thing_words(words)`: the topic engine's thing words are things the typo mender knows.
pub fn learn_thing_words(words: &[String]) {
    let mut things = THING_WORDS.write().unwrap();
    let mut by_length = THING_WORDS_BY_LENGTH.write().unwrap();
    for word in words {
        if alpha(word) && n(word) > 2 && things.insert(word.clone()) {
            let list = by_length.entry(n(word)).or_default();
            if !list.contains(word) {
                list.push(word.clone());
                list.sort();
            }
        }
    }
}

/// `_double_variants(word)`: a consonant doubled, or a doubled one typed once.
fn _double_variants(word: &str) -> HashSet<String> {
    let w: Vec<char> = word.chars().collect();
    let mut out = HashSet::new();
    for (i, &ch) in w.iter().enumerate() {
        if is_vowel(ch) || ch == 'y' {
            continue;
        }
        if i + 1 < w.len() && w[i + 1] == ch {
            out.insert(w[..i].iter().chain(w[i + 1..].iter()).collect());
        } else {
            out.insert(w[..i].iter().chain(std::iter::once(&ch)).chain(w[i..].iter()).collect());
        }
    }
    out
}

/// `_a_doubled_letter_typed_once(word)`.
fn _a_doubled_letter_typed_once(word: &str) -> HashSet<String> {
    let w: Vec<char> = word.chars().collect();
    let mut out = HashSet::new();
    for (i, &ch) in w.iter().enumerate() {
        if pyuni::ch_alpha(ch) {
            out.insert(w[..i].iter().chain(std::iter::once(&ch)).chain(w[i..].iter()).collect());
        }
    }
    out
}

/// `_three_of_a_letter(word)`.
fn _three_of_a_letter(word: &str) -> bool {
    let w: Vec<char> = word.chars().collect();
    w.windows(3).any(|t| t[0] == t[1] && t[1] == t[2])
}

/// `_doubled_letter_words(word)`.
fn _doubled_letter_words(word: &str) -> Vec<String> {
    if LEXICON.contains(word) {
        return vec![];
    }
    let mut out: Vec<String> = _a_doubled_letter_typed_once(word)
        .into_iter()
        .filter(|w| (BASE.contains(w) || (LEXICON.contains(w) && !BASE.contains(&format!("{w}s")))) && !_three_of_a_letter(w))
        .collect();
    out.sort(); // Python's order is the set's; only the best-ranked one is read
    out
}

/// `_ie_variants(word)`: "recieve" -> "receive".
fn _ie_variants(word: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for (pair, other) in [("ie", "ei"), ("ei", "ie")] {
        let mut start = word.find(pair);
        while let Some(s) = start {
            out.insert(format!("{}{}{}", &word[..s], other, &word[s + 2..]));
            start = word[s + 1..].find(pair).map(|k| k + s + 1);
        }
    }
    out
}

/// `_missing_vowel(word)`.
fn _missing_vowel(word: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    if n(word) < 5 || LEXICON.contains(word) {
        return out;
    }
    let w: Vec<char> = word.chars().collect();
    for i in 1..=w.len() {
        let mut start = i;
        while start > 0 && !is_vowel(w[start - 1]) {
            start -= 1;
        }
        let mut end = i;
        while end < w.len() && !is_vowel(w[end]) {
            end += 1;
        }
        let cluster = &w[start..end];
        if cluster.len() < 3 || (cluster[0] == 's' && S_CLUSTERS.contains(cluster[1])) {
            continue;
        }
        for vowel in VOWELS.chars() {
            let candidate: String = w[..i].iter().chain(std::iter::once(&vowel)).chain(w[i..].iter()).collect();
            if BASE.contains(&candidate) {
                out.insert(candidate);
            }
        }
    }
    out
}

/// `_extra_vowel(word)`.
fn _extra_vowel(word: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let w: Vec<char> = word.chars().collect();
    if w.len() < 8 {
        return out;
    }
    for (i, &ch) in w.iter().enumerate() {
        if !(ch == 'e' || ch == 'i') || i + 1 == w.len() {
            continue;
        }
        let before_vowel = i > 0 && is_vowel(w[i - 1]);
        let before_empty = i == 0; // '' in VOWELS is True
        if before_empty || before_vowel || is_vowel(w[i + 1]) {
            out.insert(w[..i].iter().chain(w[i + 1..].iter()).collect());
        }
    }
    out
}

/// `_a_said_for_an_ending(word)`: "computa" -> "computer".
fn _a_said_for_an_ending(word: &str) -> Vec<String> {
    if _A_WORDS.contains(word) {
        return vec![];
    }
    let (stem, plural) = if word.ends_with("as") && n(word) >= 5 {
        (upto(word, -2), "s")
    } else if word.ends_with('a') && n(word) >= 4 {
        (upto(word, -1), "")
    } else {
        return vec![];
    };
    if in_str(at(&stem, -1), *VOWELS) {
        return vec![];
    }
    ["er", "or", "ar", "ow"].iter().map(|e| format!("{stem}{e}{plural}")).collect()
}

/// `_dropped_letter(word)`.
fn _dropped_letter(word: &str) -> Vec<String> {
    if n(word) < 3 || word.ends_with('w') {
        return vec![];
    }
    vec![format!("{word}w")]
}

/// `_an_ing_said_as_n(word)`: "goin" -> "going" for words "n" stands in for "ing".
fn _an_ing_said_as_n(word: &str) -> Option<String> {
    if n(word) < 5 || !word.ends_with('n') || LEXICON.contains(word) || LX_COMMON_NAMES.contains(word) {
        return None;
    }
    if sl(word, -3, -1).chars().any(is_vowel) {
        return None;
    }
    let stem = upto(word, -1);
    let mut forms = vec![format!("{stem}ing")];
    if n(&stem) > 1 && at(&stem, -1) == at(&stem, -2) {
        forms.push(format!("{}{}ing", upto(&stem, -1), ix(&stem, -1)));
    }
    let known: HashSet<String> = forms.into_iter().filter(|f| LEXICON.contains(f)).collect();
    if known.len() == 1 { known.into_iter().next() } else { None }
}

/// `_a_leading_h(word)`.
fn _a_leading_h(word: &str) -> Vec<String> {
    if n(word) < 4 || !in_str(at(word, 0), "aeiou") {
        return vec![];
    }
    if word.ends_with('s') && BASE.contains(&upto(word, -1)) {
        return vec![];
    }
    vec![format!("h{word}")]
}

/// `_e_kept_before_ing(word)`.
fn _e_kept_before_ing(word: &str) -> Vec<String> {
    if n(word) < 6 || !word.ends_with("eing") || E_KEPT_WORDS.contains(word) {
        return vec![];
    }
    let shorter = format!("{}ing", upto(word, -4));
    if LEXICON.contains(&shorter) { vec![shorter] } else { vec![] }
}

/// `_swapped_pair(word)`: two consonants typed the wrong way round.
fn _swapped_pair(word: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    if n(word) < 5 || _a_word_the_bot_writes(word) {
        return out;
    }
    let w: Vec<char> = word.chars().collect();
    for i in 0..w.len() - 1 {
        let (a, b) = (w[i], w[i + 1]);
        if a == b || is_vowel(a) || is_vowel(b) {
            continue;
        }
        let mut v = w.clone();
        v.swap(i, i + 1);
        out.insert(v.into_iter().collect());
    }
    out
}

/// `_two_letters_the_wrong_way_round(word)`: a vowel and a consonant swapped.
fn _two_letters_the_wrong_way_round(word: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    if n(word) < 5 || LEXICON.contains(word) || _a_word_the_bot_writes(word) {
        return out;
    }
    let w: Vec<char> = word.chars().collect();
    for i in 0..w.len() - 1 {
        let (a, b) = (w[i], w[i + 1]);
        if a == b || is_vowel(a) == is_vowel(b) {
            continue;
        }
        let mut v = w.clone();
        v.swap(i, i + 1);
        let made: String = v.into_iter().collect();
        if !NO_SWAP_TO.contains(&made) {
            out.insert(made);
        }
    }
    out
}

/// `_words_the_bot_writes()`: every word in the knowledge base's keys, replies and asks.
fn _words_the_bot_writes() -> &'static HashSet<String> {
    static WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| {
        use crate::knowledge::entry::Reply;
        use crate::knowledge::lookup::{self, KeyPart};
        let mut words: HashSet<String> = HashSet::new();
        for entry in lookup::all_entries() {
            let mut texts: Vec<&str> = Vec::new();
            match &entry.reply {
                Reply::Text(s) => texts.push(s),
                Reply::Choices(v) => texts.extend(v.iter().map(String::as_str)),
                Reply::Fn(_) => {}
            }
            for part in entry.ask.iter().chain(entry.key.iter()) {
                if let KeyPart::Word(s) = part {
                    texts.push(s);
                }
            }
            texts.push(&entry.id);
            texts.push(&entry.domain);
            for t in texts {
                words.extend(lookup::tokens(t));
            }
        }
        let vocabulary = lookup::_vocabulary(lookup::all_entry_refs());
        words.extend(vocabulary.known.iter().cloned());
        words.into_iter().filter(|w| alpha(w)).map(|w| w.to_lowercase()).collect()
    });
    &WORDS
}

/// `_a_word_the_bot_writes(word)`.
fn _a_word_the_bot_writes(word: &str) -> bool {
    LX_TYPED_WORDS.contains(word) || LX_COMMON_NAMES.contains(word) || _words_the_bot_writes().contains(word)
}

/// `_bones(word)`: the word without its vowels.
fn _bones(word: &str) -> String {
    word.chars().filter(|c| !is_vowel(*c)).collect()
}

/// `_longest_consonant_run(word)`.
fn _longest_consonant_run(word: &str) -> usize {
    let (mut run, mut best) = (0, 0);
    for c in word.chars() {
        if is_vowel(c) || c == 'y' {
            run = 0;
        } else {
            run += 1;
            best = best.max(run);
        }
    }
    best
}

/// `_an_emergency_word_left_out(word)`: an urgent word typed without its vowels.
fn _an_emergency_word_left_out(word: &str) -> Option<String> {
    if n(word) < 6 || LEXICON.contains(word) || _longest_consonant_run(word) < 3 {
        return None;
    }
    let found: Vec<&String> = URGENT_BONES.get(&_bones(word)).map(|v| v.iter().filter(|k| n(k) > n(word)).collect()).unwrap_or_default();
    if found.len() == 1 { Some(found[0].clone()) } else { None }
}

/// `_a_name_slot(before, before_before, word)`: the word is a name ("my name is jo").
fn _a_name_slot(before: Option<&str>, before_before: Option<&str>, word: &str) -> bool {
    if !alpha(word) || n(word) < 2 || LEXICON.contains(word) {
        return false;
    }
    if has(&NAME_FRAMES, before) {
        return true;
    }
    if has(&BE_WORDS, before) && before_before == Some("name") {
        return true;
    }
    if CHAT_NAMES.contains(word)
        && (has(&NAME_ROLL, before)
            || (matches!(before, Some("me" | "you" | "u")) && has(&NAME_CALLS, before_before))
            || (before == Some("not") && (has(&BE_WORDS, before_before) || matches!(before_before, Some("im" | "i")))))
    {
        return true;
    }
    LX_COMMON_NAMES.contains(word) && has(&KIN, before) && has(&POSSESSIVES, before_before)
}

/// `_dropped_g(word)`: "runnin" -> "running".
fn _dropped_g(word: &str) -> Option<String> {
    if n(word) < 4 || !word.ends_with("in") || LEXICON.contains(word) || LX_REAL_IN.contains(word) || LX_COMMON_NAMES.contains(word) {
        return None;
    }
    let longer = format!("{word}g");
    if LEXICON.contains(&longer) {
        return Some(longer);
    }
    let stem = upto(word, -2);
    if _drops_a_final_e(&stem) && n(&stem) > 2 {
        let dropped = _ing_form(&stem);
        if LEXICON.contains(&dropped) {
            return Some(dropped);
        }
    }
    let mut stem = upto(word, -2);
    if n(&stem) > 3 && at(&stem, -1) == at(&stem, -2) {
        stem = upto(&stem, -1);
    }
    if _doubles_its_last_letter(&stem) && (BASE.contains(&stem) || LEXICON.contains(&stem)) {
        let doubled = format!("{stem}{}ing", ix(&stem, -1));
        if LEXICON.contains(&doubled) {
            return Some(doubled);
        }
    }
    None
}

/// `_stopped_th(word)`: "somefing" -> "something".
fn _stopped_th(word: &str) -> Option<String> {
    if n(word) < 5 || LEXICON.contains(word) {
        return None;
    }
    let mut found_suffix = None;
    for suffix in ["fings", "fing", "finks", "fink", "fins", "fin"] {
        if word.ends_with(suffix) {
            found_suffix = Some(suffix);
            break;
        }
    }
    let suffix = found_suffix?;
    let stem = &word[..word.len() - suffix.len()];
    let plural = suffix.ends_with('s');
    let found = STOPPED_TH_STEMS.get(stem).filter(|f| !f.is_empty())?;
    Some(if plural { format!("{found}s") } else { found.clone() })
}

/// `_shun_variant(word)`: "informashun" -> "information".
fn _shun_variant(word: &str) -> Option<String> {
    if LEXICON.contains(word) {
        return None;
    }
    if word.ends_with("shuns") && n(word) > 7 {
        return _shun_variant(&upto(word, -1)).map(|s| format!("{s}s"));
    }
    let stem = if word.ends_with("kshun") {
        upto(word, -5)
    } else if word.ends_with("shun") {
        upto(word, -4)
    } else {
        return None;
    };
    if n(&stem) < 2 || !alpha(&stem) {
        return None;
    }
    let kept = format!("{stem}s");
    for spelled in [
        format!("{stem}shion"),
        format!("{stem}ssion"),
        format!("{stem}sion"),
        format!("{stem}tion"),
        format!("{kept}tion"),
        format!("{kept}ssion"),
        format!("{kept}sion"),
        format!("{stem}cian"),
    ] {
        if LEXICON.contains(&spelled) || SHION_WORDS.contains(&spelled) {
            return Some(spelled);
        }
    }
    Some(format!("{stem}tion"))
}

/// `_said_with_an_f(word)`: "fink" -> "think".
fn _said_with_an_f(word: &str) -> Option<String> {
    if let Some(v) = F_TYPED_FOR_TH.get(word) {
        return Some(v.clone()).filter(|v| !v.is_empty());
    }
    if !alpha(word) || !word.starts_with('f') || LEXICON.contains(word) {
        return None;
    }
    if n(word) < 4 {
        return None;
    }
    let spelled = format!("th{}", from(word, 1));
    if LEXICON.contains(&spelled) || LX_CHAT.contains_key(&spelled) || LX_EXPAND.contains_key(&spelled) {
        return Some(spelled);
    }
    for ending in ["s", "es", "d", "ed", "ing"] {
        let stem = upto(&spelled, -(ending.len() as isize));
        if spelled.ends_with(ending) && n(&stem) >= 5 && LEXICON.contains(&stem) {
            return Some(spelled);
        }
    }
    if spelled.ends_with("in") && LEXICON.contains(&format!("{spelled}g")) {
        return Some(format!("{spelled}g"));
    }
    None
}

/// `_said_with_an_f_anywhere(word)`: "nofing" -> "nothing".
fn _said_with_an_f_anywhere(word: &str) -> Option<String> {
    if n(word) < 3 || !alpha(word) || !word.contains('f') || LEXICON.contains(word) {
        return None;
    }
    if word.ends_with("ff") {
        if word.starts_with('w') {
            return None;
        }
        let spelled = format!("{}th", upto(word, -2));
        return if LEXICON.contains(&spelled) { Some(spelled) } else { None };
    }
    let w: Vec<char> = word.chars().collect();
    for (i, &letter) in w.iter().enumerate() {
        if letter != 'f' {
            continue;
        }
        let spelled: String = w[..i].iter().collect::<String>() + "th" + &w[i + 1..].iter().collect::<String>();
        if LEXICON.contains(&spelled) {
            return Some(spelled);
        }
    }
    None
}

/// `_a_thing_where_a_thing_goes(word, before)`: "my phn" after a possessive is my phone.
fn _a_thing_where_a_thing_goes(word: &str, before: Option<&str>) -> Option<String> {
    if !has(&POSSESSIVES, before) && !has(&A_THING_GOES_AFTER, before) {
        return None;
    }
    let thing = A_THING_TYPED_AS_A_LITTLE_WORD.get(word)?;
    if LEXICON.contains(thing) { Some(thing.clone()) } else { None }
}

/// `_missing_e(word)`: "tird" -> "tired".
fn _missing_e(word: &str) -> Option<String> {
    if n(word) < 4 || !word.ends_with('d') || LEXICON.contains(word) {
        return None;
    }
    if word.ends_with("yd") {
        let c = at(word, -3);
        if c.is_some_and(|c| is_vowel(c) || c == 'y') {
            return None;
        }
        let longer = format!("{}ied", upto(word, -2));
        return if LEXICON.contains(&longer) { Some(longer) } else { None };
    }
    if !MISSING_E_AFTER.contains(&at(word, -2).map(String::from).unwrap_or_default()) {
        return None;
    }
    let longer = format!("{}ed", upto(word, -1));
    if LEXICON.contains(&longer) { Some(longer) } else { None }
}

/// `_short_double(word)`: a common short word with a letter doubled or missing.
fn _short_double(word: &str) -> Option<String> {
    if n(word) < 3 || LEXICON.contains(word) {
        return None;
    }
    let mut variants = _double_variants(word);
    if at(word, -1) == at(word, -2) {
        variants.insert(upto(word, -1));
    }
    let mut keep: Vec<String> = variants.into_iter().filter(|v| LEXICON.contains(v) && FREQUENT.contains(v)).collect();
    keep.sort_by(|a, b| rank(a).cmp(&rank(b)).then_with(|| a.cmp(b)));
    keep.into_iter().next()
}

/// `_one_letter_apart(word, other)`: one vowel wrong, two letters swapped, or one letter more
/// or less (a vowel, when the typed word is the longer one).
fn _one_letter_apart(word: &str, other: &str) -> bool {
    let w: Vec<char> = word.chars().collect();
    let o: Vec<char> = other.chars().collect();
    if w.len().abs_diff(o.len()) > 1 {
        return false;
    }
    if w.len() == o.len() {
        let differ: Vec<usize> = (0..w.len()).filter(|&i| w[i] != o[i]).collect();
        if differ.len() == 1 {
            let i = differ[0];
            return VOWEL_SET_OR_Y.contains(&w[i].to_string()) && VOWEL_SET_OR_Y.contains(&o[i].to_string());
        }
        if differ.len() == 2 {
            let (i, j) = (differ[0], differ[1]);
            return j == i + 1 && w[i] == o[j] && w[j] == o[i];
        }
        return false;
    }
    let word_is_long = w.len() > o.len();
    let (short, long) = if w.len() < o.len() { (&w, &o) } else { (&o, &w) };
    let mut i = 0;
    while i < short.len() && short[i] == long[i] {
        i += 1;
    }
    if i == short.len() {
        return false;
    }
    if short[i..] != long[i + 1..] {
        return false;
    }
    if word_is_long {
        return VOWEL_SET.contains(&long[i].to_string());
    }
    true
}

/// `_one_letter_wrong(word, other)`.
fn _one_letter_wrong(word: &str, other: &str) -> bool {
    let w: Vec<char> = word.chars().collect();
    let o: Vec<char> = other.chars().collect();
    if w.len().abs_diff(o.len()) > 1 {
        return false;
    }
    if w.len() == o.len() {
        let differ: Vec<usize> = (0..w.len()).filter(|&i| w[i] != o[i]).collect();
        if differ.len() == 1 {
            return true;
        }
        if differ.len() == 2 {
            let (i, j) = (differ[0], differ[1]);
            return j == i + 1 && w[i] == o[j] && w[j] == o[i];
        }
        return false;
    }
    let (short, long) = if w.len() < o.len() { (&w, &o) } else { (&o, &w) };
    let mut i = 0;
    while i < short.len() && short[i] == long[i] {
        i += 1;
    }
    short[i..] == long[(i + 1).min(long.len())..]
}

/// `_a_word_the_reading_knows(word)`.
fn _a_word_the_reading_knows(word: &str) -> bool {
    if _known_word(word) {
        return true;
    }
    _stems_of(word).iter().any(|s| _known_word(s))
}

/// `_known_word(word)`.
fn _known_word(word: &str) -> bool {
    LEXICON.contains(word) || THING_WORDS.read().unwrap().contains(word)
}

/// `_stems_of(word)`.
fn _stems_of(word: &str) -> HashSet<String> {
    let mut stems = HashSet::new();
    let len = n(word);
    if word.ends_with("ing") && len > 5 {
        stems.insert(upto(word, -3));
        stems.insert(format!("{}e", upto(word, -3)));
    }
    if word.ends_with("ed") && len > 4 {
        stems.insert(upto(word, -2));
        stems.insert(upto(word, -1));
    }
    if word.ends_with("es") && len > 5 {
        stems.insert(upto(word, -2));
    }
    if word.ends_with('s') && len > 4 {
        stems.insert(upto(word, -1));
    }
    stems
}

/// `_a_thing_a_letter_short(word)`: a thing word with one letter left out ("tabe" -> "table").
fn _a_thing_a_letter_short(word: &str) -> Option<String> {
    if n(word) < 4 {
        return None;
    }
    if _a_word_the_bot_writes(word) || _a_word_the_reading_knows(word) {
        return None;
    }
    let by_length = THING_WORDS_BY_LENGTH.read().unwrap();
    let first = at(word, 0);
    let last = at(word, -1);
    let fits: Vec<&String> = by_length
        .get(&(n(word) + 1))
        .map(|v| v.iter().filter(|w| at(w, 0) == first && at(w, -1) == last && _one_letter_wrong(word, w)).collect())
        .unwrap_or_default();
    if fits.len() == 1 { Some(fits[0].clone()) } else { None }
}

/// `_a_core_verb_typed_wrong(word, before, after=None)`: "i fel sick" -> "feel".
fn _a_core_verb_typed_wrong(word: &str, before: Option<&str>, after: Option<&str>) -> Option<String> {
    if !alpha(word) || n(word) < 3 || n(word) > 7 {
        return None;
    }
    if !has(&BEFORE_A_VERB, before) {
        return None;
    }
    if LEXICON.contains(word) || LX_COMMON_NAMES.contains(word) || _words_the_bot_writes().contains(word) {
        return None;
    }
    let mut found: BTreeSet<&String> = CORE_VERBS.iter().filter(|v| _one_letter_wrong(word, v)).collect();
    if word.ends_with('d') {
        let cut = upto(word, -1);
        found.retain(|v| **v != cut);
    }
    if found.is_empty() && has(&AFTER_A_VERB, after) {
        found = CORE_VERBS.iter().filter(|v| _its_vowels_typed_wrong(word, v)).collect();
    }
    if found.len() == 1 { found.into_iter().next().cloned() } else { None }
}

/// `_its_vowels_typed_wrong(word, other)`.
fn _its_vowels_typed_wrong(word: &str, other: &str) -> bool {
    if word == other || at(word, 0) != at(other, 0) {
        return false;
    }
    if n(word).abs_diff(n(other)) > 1 {
        return false;
    }
    let consonants: String = word.chars().filter(|c| !"aeiou".contains(*c)).collect();
    !consonants.is_empty() && consonants == other.chars().filter(|c| !"aeiou".contains(*c)).collect::<String>()
}

/// `_near_a_part(word)`: a body part typed with one letter more or less.
fn _near_a_part(word: &str) -> Option<String> {
    if n(word) < 3 || !alpha(word) || LEXICON.contains(word) || LX_BODY_PARTS.contains(word) {
        return None;
    }
    let mut found: BTreeSet<&String> = BTreeSet::new();
    let w: Vec<char> = word.chars().collect();
    for part in LX_BODY_PARTS.iter() {
        let p: Vec<char> = part.chars().collect();
        if p.len() < 4 || p.len().abs_diff(w.len()) != 1 {
            continue;
        }
        let (short, long) = if w.len() < p.len() { (&w, &p) } else { (&p, &w) };
        let mut i = 0;
        while i < short.len() && short[i] == long[i] {
            i += 1;
        }
        if short[i..] == long[i + 1..] {
            found.insert(part);
        }
    }
    if found.len() == 1 { found.into_iter().next().cloned() } else { None }
}

/// `_a_game_named_another_way(norm, words)`: "five a side", "a kickabout" are football.
fn _a_game_named_another_way(norm: Vec<String>, words: Vec<String>) -> (Vec<String>, Vec<String>) {
    let flat = norm.join(" ");
    if ABOUT_THE_FORM.is_match(&flat) || FINDING_A_TEAM.is_match(&flat) {
        return (norm, words);
    }
    if words.iter().any(|w| LX_BODY_PARTS.contains(w)) && words.iter().any(|w| LX_PAIN_WORDS.contains(w)) {
        return (norm, words);
    }
    let mut out = Vec::new();
    let mut read = false;
    let mut index = 0;
    while index < norm.len() {
        if A_SIDE.contains(&norm[index]) && index + 2 < norm.len() && norm[index + 1] == "a" && norm[index + 2] == "side" {
            out.push("football".to_string());
            read = true;
            index += 3;
            continue;
        }
        if norm[index] == "kickabout" || (norm[index] == "kick" && norm.get(index + 1).map(String::as_str) == Some("about")) {
            out.push("football".to_string());
            read = true;
            index += if norm[index] == "kickabout" { 1 } else { 2 };
            continue;
        }
        out.push(norm[index].clone());
        index += 1;
    }
    if !read || words.iter().any(|w| w == "football") {
        return (out, words);
    }
    let mut words = words;
    words.push("football".to_string());
    (out, words)
}

/// `_parts_that_hurt(words, fixes)`: a body part typed one letter off beside a pain word.
fn _parts_that_hurt(words: Vec<String>, fixes: &mut Vec<(String, String)>) -> Vec<String> {
    let pain: Vec<usize> = words.iter().enumerate().filter(|(_, w)| LX_PAIN_WORDS.contains(*w)).map(|(i, _)| i).collect();
    if pain.is_empty() {
        return words;
    }
    let mut out = Vec::new();
    for (at_, word) in words.iter().enumerate() {
        let part = _near_a_part(word);
        let beside = (at_ > 0 && pain.contains(&(at_ - 1))) || pain.contains(&(at_ + 1));
        let mine = at_ > 0
            && matches!(words[at_ - 1].as_str(), "my" | "our")
            && pain.iter().any(|&other| (other as i64 - at_ as i64).abs() <= PART_REACH);
        match part {
            Some(part) if beside || mine => {
                fixes.push((word.clone(), part.clone()));
                out.push(part);
            }
            _ => out.push(word.clone()),
        }
    }
    out
}

/// `_where_a_state_stands(before, before_before, after, before_words=None)`: the words around
/// this one are where a feeling can be said ("im so ___", "he is ___").
fn _where_a_state_stands(before: Option<&str>, before_before: Option<&str>, after: Option<&str>, before_words: Option<&[String]>) -> bool {
    let words: Vec<&str> = match before_words {
        Some(b) => b.iter().map(String::as_str).collect(),
        None => [before_before, before].into_iter().flatten().collect(),
    };
    if let Some(last) = words.last() {
        if DETERMINER_IN_FRONT.contains(*last) {
            let done_to = POSSESSIVE_IN_FRONT.contains(*last) && words.iter().any(|w| CAUSES_A_STATE.contains(*w));
            if !done_to {
                return false;
            }
        }
    }
    let end = words.len() as isize - 1;
    let mut back = end;
    while back >= 0 && _measures_a_state(words[back as usize], false) {
        back -= 1;
    }
    if back < 0 {
        return _opens_a_state(after);
    }
    let b = back as usize;
    let stands = words[b];
    if BE_BEFORE_A_STATE.contains(stands) {
        return true;
    }
    if THIRD_PERSON_BE.contains(stands) {
        let opening = !(b >= 1 && (PERSON_WORDS.contains(words[b - 1]) || OBJECT_PRONOUN.contains(words[b - 1])));
        return _ends_a_state(after, opening);
    }
    if CAUSES_A_STATE.contains(stands) {
        return back < end || _ends_a_state(after, false);
    }
    if OBJECT_PRONOUN.contains(stands) && b >= 1 && CAUSES_A_STATE.contains(words[b - 1]) {
        return true;
    }
    if stands == "my" && b >= 1 && CAUSES_A_STATE.contains(words[b - 1]) {
        return _ends_a_state(after, false);
    }
    let mut wide = back;
    while wide >= 0 && _measures_a_state(words[wide as usize], true) {
        wide -= 1;
    }
    if wide < back && wide >= 0 && BE_BEFORE_A_STATE.contains(words[wide as usize]) {
        return true;
    }
    if wide < back && wide < 0 {
        return _opens_a_state(after);
    }
    false
}

/// `_ends_a_state(after, opening=False)`.
fn _ends_a_state(after: Option<&str>, opening: bool) -> bool {
    if after.is_none() || after == Some("|") || has(&AFTER_A_STATE, after) {
        return true;
    }
    if opening {
        return false;
    }
    has(&TIMES_A_STATE, after) || has(&AFTER_A_TIME, after) || has(&A_STATE_THEN_A_TIME, after)
}

/// `_opens_a_state(after)`.
fn _opens_a_state(after: Option<&str>) -> bool {
    _ends_a_state(after, true) || has(&NOT_A_THING, after)
}

/// `_a_measure_spelt_wrong(word)`: "absolutly", "realy".
fn _a_measure_spelt_wrong(word: &str) -> bool {
    if !(n(word) >= 6 && alpha(word)) {
        return false;
    }
    if AMOUNT_BEFORE_A_STATE.iter().any(|m| _one_letter_apart(word, m)) {
        return true;
    }
    if word.ends_with("aly") || word.ends_with("lye") || word.ends_with("yl") {
        return true;
    }
    n(word) >= 9 && word.ends_with("ley")
}

/// `_measures_a_state(word, wide=False)`: "so", "really", "absolutely" in front of a state.
fn _measures_a_state(word: &str, wide: bool) -> bool {
    if wide && PHRASE_MEASURES.contains(word) {
        return true;
    }
    AMOUNT_BEFORE_A_STATE.contains(word) || BARE_TO.contains(word) || (n(word) >= 5 && word.ends_with("ly")) || _a_measure_spelt_wrong(word)
}

/// `_a_letter_typed_twice(word)`: a feeling with one letter doubled ("sadd").
fn _a_letter_typed_twice(word: &str) -> Option<String> {
    if n(word) < 4 || LX_COMMON_NAMES.contains(word) {
        return None;
    }
    let w: Vec<char> = word.chars().collect();
    for place in 0..w.len() - 1 {
        if w[place] != w[place + 1] {
            continue;
        }
        let shorter: String = w[..place].iter().chain(w[place + 1..].iter()).collect();
        if LX_FEELINGS.contains(&shorter) {
            return Some(shorter);
        }
    }
    None
}

/// `_an_introduction(...)`: the word is someone being introduced, not a feeling.
fn _an_introduction(before: Option<&str>, before_before: Option<&str>, after: Option<&str>, before_words: Option<&[String]>, after_words: Option<&[String]>) -> bool {
    for word in before_words.unwrap_or(&[]) {
        if INTRODUCING_A_PERSON.contains(word) {
            return true;
        }
    }
    if has(&INTRODUCING_AFTER, after) {
        return true;
    }
    if before == Some("is") && matches!(before_before, Some("this" | "that")) {
        return true;
    }
    if after == Some("is") && matches!(before, None | Some("|")) {
        return true;
    }
    if matches!(after, Some("and" | "|")) || digit(after.unwrap_or("")) {
        for word in after_words.unwrap_or(&[]).iter().take(3) {
            if digit(word) {
                return true;
            }
        }
    }
    false
}

/// `_a_letter_wrong_in_a_state(word)`.
fn _a_letter_wrong_in_a_state(word: &str) -> Option<String> {
    if n(word) < 5 || LEXICON.contains(word) || LX_COMMON_NAMES.contains(word) {
        return None;
    }
    let first = upto(word, 1);
    let found: BTreeSet<&String> = STATES_OF_A_PERSON.iter().filter(|s| upto(s, 1) == first && _a_letter_apart(word, s)).collect();
    if found.len() == 1 { found.into_iter().next().cloned() } else { None }
}

/// `_a_letter_apart(word, other)`.
fn _a_letter_apart(word: &str, other: &str) -> bool {
    if _one_letter_apart(word, other) {
        return true;
    }
    if n(word).abs_diff(n(other)) != 1 {
        return false;
    }
    let (short, long) = if n(word) < n(other) { (word, other) } else { (other, word) };
    let l: Vec<char> = long.chars().collect();
    (1..l.len()).any(|i| l[..i].iter().chain(l[i + 1..].iter()).copied().eq(short.chars()))
}

/// `_near_a_feeling(...)`: a feeling word typed a letter off where a feeling is said.
fn _near_a_feeling(word: &str, before: Option<&str>, before_before: Option<&str>, after: Option<&str>, alone: bool, before_words: Option<&[String]>, after_words: Option<&[String]>) -> Option<String> {
    if !alone && !_where_a_state_stands(before, before_before, after, before_words) {
        return None;
    }
    if n(word) < 4 || LEXICON.contains(word) || LX_NOT_A_FEELING.contains(word) {
        return None;
    }
    if let Some(twice) = _a_letter_typed_twice(word) {
        if !_an_introduction(before, before_before, after, before_words, after_words) {
            return Some(twice);
        }
    }
    if n(word) == 4 {
        if after.is_some() && after != Some("|") {
            return None;
        }
        if LX_COMMON_NAMES.contains(word) {
            return None;
        }
    }
    if word.ends_with("cious") {
        let f = format!("{}xious", upto(word, -5));
        if LX_FEELINGS.contains(&f) {
            return Some(f);
        }
    }
    if word.ends_with("ctious") {
        let f = format!("{}xious", upto(word, -6));
        if LX_FEELINGS.contains(&f) {
            return Some(f);
        }
    }
    let wl = n(word);
    let found: BTreeSet<&String> = LX_FEELINGS
        .iter()
        .filter(|f| _one_letter_apart(word, f) && (wl >= 5 || n(f) == wl + 1) && !_a_consonant_left_out_of_an_ing_word(word, f))
        .collect();
    if found.len() == 1 {
        return found.into_iter().next().cloned();
    }
    if matches!(after, None | Some("|")) && _the_state_is_the_whole_frame(before, before_before, before_words) {
        return the_feeling_two_letters_are_gone_from(word);
    }
    None
}

/// `_a_consonant_left_out_of_an_ing_word(word, feeling)`.
fn _a_consonant_left_out_of_an_ing_word(word: &str, feeling: &str) -> bool {
    let w: Vec<char> = word.chars().collect();
    let f: Vec<char> = feeling.chars().collect();
    if !word.ends_with("ing") || f.len() != w.len() + 1 {
        return false;
    }
    let mut i = 0;
    while i < w.len() && w[i] == f[i] {
        i += 1;
    }
    w[i..] == f[i + 1..] && !VOWEL_SET.contains(&f[i].to_string())
}

/// `_the_state_is_the_whole_frame(before, before_before, before_words=None)`.
fn _the_state_is_the_whole_frame(before: Option<&str>, before_before: Option<&str>, before_words: Option<&[String]>) -> bool {
    let words: Vec<&str> = match before_words {
        Some(b) => b.iter().map(String::as_str).filter(|w| !w.is_empty()).collect(),
        None => [before_before, before].into_iter().flatten().filter(|w| !w.is_empty()).collect(),
    };
    let mut back = words.len() as isize - 1;
    while back >= 0 && _measures_a_state(words[back as usize], false) {
        back -= 1;
    }
    back == 0
}

/// `the_feeling_two_letters_are_gone_from(word)`: "exausted" -> "exhausted".
pub fn the_feeling_two_letters_are_gone_from(word: &str) -> Option<String> {
    let low = text::strip(word).to_lowercase();
    if n(&low) < 6 || LX_COMMON_NAMES.contains(&low) {
        return None;
    }
    let found: BTreeSet<&String> = LX_FEELINGS.iter().filter(|f| _two_letters_out(&low, f)).collect();
    if found.len() == 1 { found.into_iter().next().cloned() } else { None }
}

/// `_two_letters_out(word, feeling)`.
fn _two_letters_out(word: &str, feeling: &str) -> bool {
    let diff = n(feeling) as isize - n(word) as isize;
    if !(diff == 1 || diff == 2) || upto(word, 1) != upto(feeling, 1) {
        return false;
    }
    if _one_letter_apart(word, feeling) {
        return false;
    }
    let mut rest = feeling.chars();
    word.chars().all(|letter| rest.any(|c| c == letter))
}

/// `_feeling_without_its_ending(word)`: "stress" -> "stressed".
fn _feeling_without_its_ending(word: &str) -> Option<String> {
    if n(word) < 3 || !alpha(word) {
        return None;
    }
    if LX_FEELINGS_OR_STATES.contains(word) {
        return None;
    }
    for ending in ["ed", "d", "y", "s", "us", "e", "g", "ing", "l", "ty"] {
        let w = format!("{word}{ending}");
        if LX_FEELINGS_OR_STATES.contains(&w) {
            return Some(w);
        }
    }
    None
}

/// The word with Python's `.strip().strip('.,!?;:\'"()').lower()`.
fn bare_low(word: &str) -> String {
    text::strip_chars(text::strip(word), ".,!?;:'\"()").to_lowercase()
}

/// `reads_as_a_feeling(word)`: a word on its own that the engine reads as a feeling.
pub fn reads_as_a_feeling(word: &str) -> bool {
    let low = bare_low(word);
    if low.is_empty() || !alpha(&low) {
        return false;
    }
    if _feeling_without_its_ending(&low).is_some() || _a_letter_wrong_in_a_state(&low).is_some() {
        return true;
    }
    if LX_SOUNDS_LIKE_A_FEELING.contains_key(&low) {
        return true;
    }
    let mut fixes = Vec::new();
    if _token(&low, &mut fixes, None, None, None, true, None, None)
        .iter()
        .any(|w| LX_FEELINGS.contains(w) || LX_A_STATE.contains(w) || STATES_OF_A_PERSON.contains(w))
    {
        return true;
    }
    if LX_FEELINGS_OR_STATES.contains(&format!("{low}d")) {
        return true;
    }
    _a_vowel_left_out_of_a_state(&low).is_some()
}

/// `_a_vowel_left_out_of_a_state(word)`.
fn _a_vowel_left_out_of_a_state(word: &str) -> Option<String> {
    let len = n(word);
    if !(3..5).contains(&len) || !alpha(word) {
        return None;
    }
    if LEXICON.contains(word) || LX_COMMON_NAMES.contains(word) {
        return None;
    }
    let found: BTreeSet<&String> = LX_FEELINGS_OR_STATES.iter().filter(|s| n(s) == len + 1 && _one_vowel_less(s).contains(word)).collect();
    if found.len() == 1 { found.into_iter().next().cloned() } else { None }
}

/// `_one_vowel_less(word)`.
fn _one_vowel_less(word: &str) -> HashSet<String> {
    let w: Vec<char> = word.chars().collect();
    (0..w.len())
        .filter(|&i| VOWEL_SET.contains(&w[i].to_string()) && i > 0 && i + 1 < w.len())
        .map(|i| w[..i].iter().chain(w[i + 1..].iter()).collect())
        .collect()
}

/// `reads_as_a_sound(word)`: a misspelt feeling that sounds like one.
pub fn reads_as_a_sound(word: &str) -> bool {
    let low = bare_low(word);
    if low.is_empty() || !alpha(&low) {
        return false;
    }
    crate::understanding::kinds::feeling_by_sound(&low).is_some()
}

/// `repaired_tokens(word)`: what the engine reads a word on its own as.
pub fn repaired_tokens(word: &str) -> Vec<String> {
    let low = bare_low(word);
    if low.is_empty() {
        return vec![];
    }
    let mut fixes = Vec::new();
    _token(&low, &mut fixes, None, None, None, true, None, None)
}

/// `spelled(word)`: the words a chat spelling stands for.
pub fn spelled(word: &str) -> Vec<String> {
    let low = bare_low(word);
    if low.is_empty() {
        return vec![];
    }
    for table in [&*LX_CHAT, &*LX_EXPAND, &*LX_US_SPELLINGS, &*LX_MISSPELLINGS, &*LX_URGENT_MISSPELLINGS] {
        if let Some(v) = table.get(&low) {
            return words_of(v);
        }
    }
    vec![]
}

/// `_unsays_the_word(word, variant)`: dropping a doubled letter would undo an ending.
fn _unsays_the_word(word: &str, variant: &str) -> bool {
    if n(variant) >= n(word) {
        return false;
    }
    let Some(ending) = ["ing", "est", "ed", "er"].into_iter().find(|e| variant.ends_with(e)) else { return false };
    let stem = upto(variant, -(ending.len() as isize));
    if n(&stem) < 2 {
        return false;
    }
    let doubled = format!("{stem}{}", ix(&stem, -1));
    word == format!("{doubled}{ending}")
}

/// `_a_word_of_the_bots(word)`.
fn _a_word_of_the_bots(word: &str) -> bool {
    if NO_SWAP_TO.contains(word) {
        return false;
    }
    BASE.contains(word) || _words_the_bot_writes().contains(word)
}

/// `fix_typo(word)`: the one word a typo stands for, when the reading is clear.
pub fn fix_typo(word: &str) -> Option<String> {
    if !alpha(word) {
        return None;
    }
    if BASE.contains(word) || LX_TYPED_WORDS.contains(word) || _words_the_bot_writes().contains(word) || LX_NOT_A_FEELING.contains(word) || LX_COMMON_NAMES.contains(word) {
        return None;
    }
    if let Some(s) = _shun_variant(word) {
        return Some(s);
    }
    if let Some(s) = _stopped_th(word) {
        return Some(s);
    }
    if let Some(s) = _short_double(word) {
        return Some(s);
    }
    let doubled_once = _doubled_letter_words(word);
    if n(word) == 3 && !doubled_once.is_empty() {
        let best = doubled_once.iter().min_by(|a, b| rank(a).cmp(&rank(b))).unwrap();
        if rank(best) < 2000 {
            return Some(best.clone());
        }
    }
    if n(word) < 4 {
        return None;
    }
    if let Some(s) = _a_thing_a_letter_short(word) {
        return Some(s);
    }
    let mut candidates: Vec<String> = _double_variants(word).into_iter().filter(|w| _a_word_of_the_bots(w) && !_unsays_the_word(word, w)).collect();
    candidates.extend(doubled_once.iter().cloned());
    candidates.extend(_ie_variants(word).into_iter().filter(|w| _a_word_of_the_bots(w)));
    if let Some(d) = _dropped_g(word) {
        candidates.push(d);
    }
    if let Some(d) = _an_ing_said_as_n(word) {
        candidates.push(d);
    }
    candidates.extend(_dropped_letter(word).into_iter().filter(|w| _a_word_of_the_bots(w)));
    candidates.extend(_a_leading_h(word).into_iter().filter(|w| _a_word_of_the_bots(w)));
    candidates.extend(_extra_vowel(word).into_iter().filter(|w| _a_word_of_the_bots(w)));
    candidates.extend(_missing_vowel(word).into_iter().filter(|w| _a_word_of_the_bots(w)));
    candidates.extend(_a_said_for_an_ending(word).into_iter().filter(|w| BASE.contains(w) || (w.ends_with('s') && LEXICON.contains(w))));
    candidates.extend(_swapped_pair(word).into_iter().filter(|w| _a_word_of_the_bots(w) || URGENT_WORDS.contains(w)));
    candidates.extend(_two_letters_the_wrong_way_round(word).into_iter().filter(|w| _a_word_of_the_bots(w)));
    if let Some(d) = _missing_e(word) {
        candidates.push(d);
    }
    candidates.extend(_e_kept_before_ing(word).into_iter().filter(|w| LEXICON.contains(w)));
    if let Some(d) = _an_emergency_word_left_out(word) {
        candidates.push(d);
    }
    // _a_silent_t_put_back: "lisen" -> "listen", "whisle" -> "whistle"
    for ending in ["en", "le"] {
        if word.ends_with(ending) && n(word) > ending.len() + 1 {
            if at(word, -(ending.len() as isize) - 1).is_some_and(is_vowel) {
                continue;
            }
            let w = format!("{}t{}", upto(word, -(ending.len() as isize)), ending);
            if BASE.contains(&w) || LEXICON.contains(&w) {
                candidates.push(w);
            }
        }
    }
    let typed_is_a_word = BASE.contains(word);
    candidates.extend(_double_variants(word).into_iter().filter(|w| BASE.contains(w) && !(typed_is_a_word && _unsays_the_word(word, w))));
    let unique: BTreeSet<String> = candidates.into_iter().collect();
    let mut words: Vec<String> = unique.into_iter().collect();
    words.sort_by_key(|w| rank(w));
    if words.is_empty() {
        return None;
    }
    if words.len() == 1 {
        return Some(words.remove(0));
    }
    if rank(&words[0]) < 120 && rank(&words[1]) > 900 {
        return Some(words.remove(0));
    }
    None
}

/// `_slip(word, opener)`: "missing" or "extra" when the word is the opener one letter off.
fn _slip(word: &str, opener: &str) -> Option<&'static str> {
    if word == opener || upto(word, 1) != upto(opener, 1) || n(word).abs_diff(n(opener)) != 1 {
        return None;
    }
    let w: Vec<char> = word.chars().collect();
    let o: Vec<char> = opener.chars().collect();
    let word_is_short = w.len() < o.len();
    let (short, long) = if word_is_short { (&w, &o) } else { (&o, &w) };
    let mut i = 0;
    while i < short.len() && short[i] == long[i] {
        i += 1;
    }
    if short[i..] != long[i + 1..] {
        return None;
    }
    Some(if word_is_short { "missing" } else { "extra" })
}

/// `fix_opener(word)`: a question opener typed a letter off ("wat", "whn", "hows").
pub fn fix_opener(word: &str) -> Option<String> {
    if !alpha(word) || n(word) < 2 {
        return None;
    }
    let doubled = n(word) > 2 && at(word, -1) == at(word, -2);
    let short = n(word) == 2;
    let mut fits: Vec<(String, &str)> = Vec::new();
    let mut openers: Vec<&String> = OPENERS.iter().collect();
    openers.sort(); // Python iterates the set; only "exactly one fits" is read
    for opener in openers {
        let Some(shape) = _slip(word, opener) else { continue };
        if short && !WH_OPENERS.contains(opener) {
            continue;
        }
        if shape == "extra" && !doubled {
            continue;
        }
        fits.push((opener.clone(), shape));
    }
    let last = at(word, -1);
    let second_last = at(word, -2);
    for test in 0..2 {
        if fits.len() > 1 {
            let kept: Vec<(String, &str)> = fits
                .iter()
                .filter(|(o, _)| if test == 0 { at(o, -1) == last } else { doubled && at(o, -1) == second_last })
                .cloned()
                .collect();
            if kept.len() == 1 {
                fits = kept;
            }
        }
        if fits.len() == 1 {
            break;
        }
    }
    if fits.len() == 1 { Some(fits.remove(0).0) } else { None }
}

/// `_plain_word(word)`.
fn _plain_word(word: &str) -> bool {
    BASE.contains(word) || LX_CHAT.contains_key(word) || LX_MISSPELLINGS.contains_key(word) || LX_EXPAND.contains_key(word)
}

/// `_repair_openers(clauses, fixes)`: a misspelt opener at the head of a clause mended.
fn _repair_openers(mut clauses: Vec<Vec<String>>, fixes: &mut Vec<(String, String)>) -> Vec<Vec<String>> {
    for clause in clauses.iter_mut() {
        let lead = clause.len() - strip_leaders(clause).len();
        let starts: BTreeSet<usize> = [0, lead].into_iter().collect();
        for start in starts {
            if start >= clause.len() {
                continue;
            }
            let word = clause[start].clone();
            if _plain_word(&word) {
                continue;
            }
            if let Some(fixed) = fix_opener(&word) {
                fixes.push((word.clone(), fixed.clone()));
                let parts = _token(&fixed, fixes, None, None, None, false, None, None);
                clause.splice(start..start + 1, parts);
                break;
            }
        }
    }
    clauses
}

/// `_four_spells_for(word)`: "4ever" -> "forever".
fn _four_spells_for(word: &str) -> Option<String> {
    if n(word) < 4 || !word.starts_with('4') || !alpha(&from(word, 1)) {
        return None;
    }
    let rest = from(word, 1);
    if !in_str(at(&rest, 0), "aeioug") {
        return None;
    }
    if BASE.contains(&rest) || LX_CHAT.contains_key(&rest) || LX_MISSPELLINGS.contains_key(&rest) || LEXICON.contains(&rest) {
        return Some(format!("for{rest}"));
    }
    None
}

/// `_collapse(word)`: letters held down ("sooooo") collapsed to a known word.
fn _collapse(word: &str) -> Option<String> {
    if !rx(r"(.)\1\1").is_match(word) {
        return None;
    }
    let runs = rx(r"(.)\1+").finditer(word);
    if runs.len() > 3 {
        return None;
    }
    let choices: Vec<usize> = runs.iter().map(|m| n(m.as_str()).min(3)).collect();
    // itertools.product(*choices): the first run varies slowest
    let mut lengths: Vec<usize> = vec![1; runs.len()];
    loop {
        let mut variant = word.to_string();
        for (m, &len) in runs.iter().zip(lengths.iter()).rev() {
            variant = format!("{}{}{}", &variant[..m.start()], m.str(1).repeat(len), &variant[m.end()..]);
        }
        if variant != word && LEXICON.contains(&variant) {
            return Some(variant);
        }
        // next combination
        let mut k = runs.len();
        loop {
            if k == 0 {
                return None;
            }
            k -= 1;
            if lengths[k] < choices[k] {
                lengths[k] += 1;
                for l in lengths.iter_mut().skip(k + 1) {
                    *l = 1;
                }
                break;
            }
        }
    }
}

/// `_apostrophe(word)`: a contraction spelled out ("can't" -> can not, "i'm" -> i am).
fn _apostrophe(word: &str) -> Option<Vec<String>> {
    if !word.contains('\'') {
        return None;
    }
    let (base, _, rest) = text::partition(word, "'");
    if rest == "t" && word.ends_with("n't") {
        let stem = &word[..word.len() - 3];
        return match stem {
            "ca" => Some(strs(&["can", "not"])),
            "wo" => Some(strs(&["will", "not"])),
            "sha" => Some(strs(&["shall", "not"])),
            "ai" => Some(strs(&["is", "not"])),
            "" => None,
            s => Some(strs(&[s, "not"])),
        };
    }
    match rest {
        "m" => Some(strs(&[base, "am"])),
        "re" => Some(strs(&[base, "are"])),
        "ve" => Some(strs(&[base, "have"])),
        "ll" => Some(strs(&[base, "will"])),
        "d" => Some(if PRONOUN_BASES.contains(base) { strs(&[base, "would"]) } else { strs(&[base]) }),
        "s" => {
            if PRONOUN_BASES.contains(base) {
                return Some(strs(&[base, "is"]));
            }
            let read = LX_CHAT
                .get(base)
                .filter(|s| !s.is_empty())
                .or_else(|| LX_MISSPELLINGS.get(base).filter(|s| !s.is_empty()))
                .or_else(|| LX_URGENT_MISSPELLINGS.get(base).filter(|s| !s.is_empty()));
            if let Some(read) = read {
                if !read.contains(' ') {
                    return Some(vec![read.clone(), "is".to_string()]);
                }
            }
            Some(strs(&[base]))
        }
        _ => None,
    }
}

/// `_counts_a_compound(head, tail)`.
fn _counts_a_compound(head: &str, tail: &str) -> bool {
    if head.is_empty() || tail.is_empty() || head == "|" || tail == "|" {
        return false;
    }
    if digit(head) || digit(tail) {
        return false;
    }
    if NOT_COUNTED_AFTER.contains(head) || NOT_COUNTED_AFTER.contains(tail) {
        return false;
    }
    if NOT_A_COMPOUND_HEAD.contains(head) || NOT_A_COMPOUND_TAIL.contains(tail) {
        return false;
    }
    if head.ends_with("ing") || tail.ends_with("ing") {
        return false;
    }
    true
}

/// `_counts_a_thing(word)`.
fn _counts_a_thing(word: &str) -> bool {
    if word.is_empty() || word == "|" {
        return true;
    }
    let first = words_of(word).into_iter().next().unwrap_or_else(|| "|".to_string());
    let word = first.as_str();
    if digit(word) || COUNT_UNITS.contains(word) || NOT_COUNTED_AFTER.contains(word) {
        return true;
    }
    n(word) > 2 && word.ends_with('s') && !(word.ends_with("ss") || word.ends_with("us") || word.ends_with("is"))
}

/// The first (or last) word a chat spelling stands for (`(spelled.split() or ['|'])[0]`).
fn first_word(s: &str) -> String {
    text::split_ref(s).first().map(|w| w.to_string()).unwrap_or_else(|| "|".to_string())
}
fn last_word(s: &str) -> String {
    text::split_ref(s).last().map(|w| w.to_string()).unwrap_or_else(|| "|".to_string())
}
/// `lexicon.CHAT.get(w, w)`.
fn chat_or(w: &str) -> String {
    LX_CHAT.get(w).cloned().unwrap_or_else(|| w.to_string())
}

/// `_digit_word(token, before, after, after_after=None)`: "2" and "4" read as two/four,
/// to/too or for.
fn _digit_word(token: &str, before: Option<&str>, after: Option<&str>, after_after: Option<&str>) -> Option<String> {
    if token != "2" && token != "4" {
        return None;
    }
    let number = if token == "2" { "two" } else { "four" };
    if matches!(after.unwrap_or("").to_lowercase().as_str(), "yr" | "yrs") {
        return Some(number.to_string());
    }
    let before = before.filter(|b| !b.is_empty()).unwrap_or("|");
    let spelled_before = LX_CHAT.get(before).filter(|s| !s.is_empty()).or_else(|| LX_EXPAND.get(before).filter(|s| !s.is_empty())).cloned().unwrap_or_else(|| before.to_string());
    let spoken_before = last_word(&spelled_before);
    let after = match after {
        Some(a) if !a.is_empty() => first_word(&chat_or(a)),
        _ => "|".to_string(),
    };
    let after = after.as_str();
    if OPERATORS.contains(after) || OPERATORS.contains(before) {
        return Some(number.to_string());
    }
    if COUNTS.contains(after) && (BEFORE_VERB.contains(before) || TOO_AFTER.contains(before) || before == "|") {
        let tail = match after_after {
            Some(a) if !a.is_empty() => first_word(&chat_or(a)),
            _ => "|".to_string(),
        };
        if A_PERIOD_FOLLOWS.contains(&tail) || tail == "|" {
            return Some(number.to_string());
        }
        return Some("for".to_string());
    }
    if A_LABEL_BEFORE.contains(&spoken_before) && !digit(after) {
        return Some(number.to_string());
    }
    if token == "2" && MOTION_BEFORE.contains(before) && !A_PERIOD_FOLLOWS.contains(after) {
        return Some("to".to_string());
    }
    if DEMONSTRATIVE.contains(after) {
        let aa = match after_after {
            Some(a) if !a.is_empty() => first_word(&chat_or(a)),
            _ => "|".to_string(),
        };
        if A_PERIOD_FOLLOWS.contains(&aa) {
            return Some(number.to_string());
        }
    }
    if token == "2" {
        if TO_AFTER.contains(before) && (after == "|" || UP_TO_TAGS.contains(after)) {
            return Some("to".to_string());
        }
        if after == "|" {
            return Some(if TOO_AFTER.contains(before) { "too" } else { number }.to_string());
        }
        if TOO_A.contains(after) {
            return Some("too".to_string());
        }
        if BEFORE_WORD.contains(after) || BEFORE_VERB.contains(after) {
            return Some("to".to_string());
        }
        if TOO_A_GRADED.contains(after) && A_NEGATION_BEFORE.contains(&spoken_before) {
            return Some("too".to_string());
        }
        return Some(number.to_string());
    }
    if BE_VERBS.contains(before) {
        return Some(number.to_string());
    }
    if FOR_A.contains(after) {
        return Some("for".to_string());
    }
    if FOR_OBJECT.contains(after) && (before != "|" || FOR_MEAL.contains(after)) {
        return Some("for".to_string());
    }
    if digit(after) && !digit(before) {
        return Some("for".to_string());
    }
    if let Some(aa) = after_after {
        let tail = if aa.is_empty() { "|".to_string() } else { first_word(&chat_or(aa)) };
        if COMPOUND_HEADS.contains(after) && COMPOUND_TAILS.contains(&tail) {
            return Some(number.to_string());
        }
    }
    if NUMBERS_BEFORE.contains(before) && !_counts_a_thing(after) {
        return Some(number.to_string());
    }
    if let Some(aa) = after_after {
        let tail = if aa.is_empty() { "|".to_string() } else { first_word(&chat_or(aa)) };
        if _counts_a_compound(after, &tail) {
            return Some(number.to_string());
        }
    }
    if !_counts_a_thing(after) {
        return Some("for".to_string());
    }
    Some(number.to_string())
}

/// `_once_only(tokens)`: a word typed twice in a row kept once, for the words that never repeat.
pub fn _once_only(tokens: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for token in tokens {
        if out.last() == Some(token) && REPEAT_ONCE.contains(token) {
            continue;
        }
        out.push(token.clone());
    }
    out
}

/// `_only_leaders_before(words, index)`: only filler words between the clause start and here.
fn _only_leaders_before(words: &[String], index: usize) -> bool {
    let mut start = 0;
    for i in 0..index {
        if words[i] == "|" {
            start = i + 1;
        }
    }
    words[start..index].iter().all(|w| LEADERS.contains(w))
}

/// `_lookahead(words, index, n=3)`: the next words of the clause as the engine reads them.
fn _lookahead(words: &[String], index: usize, count: usize) -> Vec<String> {
    let mut out = Vec::new();
    for word in words.iter().skip(index + 1).take(count) {
        if word == "|" {
            break;
        }
        let mut fixes = Vec::new();
        let read = _token(word, &mut fixes, None, None, None, false, None, None);
        out.push(read.into_iter().next().unwrap_or_else(|| word.clone()));
    }
    out
}

/// `subject_missing(words, index)`.
pub fn subject_missing(words: &[String], index: usize) -> bool {
    DROPPED_SUBJECT_AUX.contains(&words[index]) && opens_without_subject(words, index)
}

/// `_opener_is(word)`: "whats" / "hows" typed as one word with its "is".
fn _opener_is(word: &str) -> Option<Vec<String>> {
    if n(word) < 4 || !word.ends_with(['s', 'z']) {
        return None;
    }
    let stem = upto(word, -1);
    if WH_OPENERS.contains(&stem) {
        return Some(vec![stem, "is".to_string()]);
    }
    let spelled = LX_CHAT
        .get(&stem)
        .filter(|s| !s.is_empty())
        .or_else(|| LX_EXPAND.get(&stem).filter(|s| !s.is_empty()))
        .or_else(|| LX_MISSPELLINGS.get(&stem).filter(|s| !s.is_empty()))
        .map(|s| words_of(s))
        .unwrap_or_default();
    if spelled.len() == 1 && WH_OPENERS.contains(&spelled[0]) {
        return Some(vec![spelled[0].clone(), "is".to_string()]);
    }
    None
}

/// `_cause_in_place_is_the_verb(before, after, after_words=None)`: "cause" is the verb here,
/// not "because".
fn _cause_in_place_is_the_verb(before: Option<&str>, after: Option<&str>, after_words: Option<&[String]>) -> bool {
    if has(&CAUSE_BEFORE_A_VERB, before) || has(&CAUSE_BEFORE_A_NOUN, before) {
        return true;
    }
    let after_words = after_words.unwrap_or(&[]);
    if has(&CAUSE_A_DEMONSTRATIVE, after) {
        let rest = after_words.get(1);
        return rest.is_none_or(|r| !CAUSE_A_SUBJECT_FOLLOWS.contains(r));
    }
    if has(&CAUSE_A_SUBJECT_FOLLOWS, after) {
        return false;
    }
    if after.is_none() {
        return false;
    }
    if after == Some("the") && after_words.iter().skip(1).take(3).any(|w| CAUSE_A_CLAUSE_VERB.contains(w)) {
        return false;
    }
    true
}

/// `_token(word, fixes, ...)`: one typed word as the words it stands for (chat-speak spelled
/// out, contractions opened, typos mended). Every mend is added to `fixes`.
#[allow(clippy::too_many_arguments)]
pub fn _token(word: &str, fixes: &mut Vec<(String, String)>, before: Option<&str>, before_before: Option<&str>, after: Option<&str>, alone: bool, before_words: Option<&[String]>, after_words: Option<&[String]>) -> Vec<String> {
    if word == "cause" && _cause_in_place_is_the_verb(before, after, after_words) {
        return vec![word.to_string()];
    }
    if before.is_some() {
        if let Some(thing) = _a_thing_where_a_thing_goes(word, before) {
            fixes.push((word.to_string(), thing.clone()));
            return vec![thing];
        }
    }
    if let Some(d) = DIGIT_WORDS.get(word) {
        return vec![d.clone()];
    }
    if let Some(c) = LX_CHAT.get(word) {
        return words_of(c);
    }
    if let Some(c) = LX_EXPAND.get(word) {
        return words_of(c);
    }
    if let Some(c) = LX_US_SPELLINGS.get(word) {
        return vec![c.clone()];
    }
    if let Some(c) = LX_MISSPELLINGS.get(word) {
        return words_of(c);
    }
    if let Some(c) = LX_URGENT_MISSPELLINGS.get(word) {
        fixes.push((word.to_string(), c.clone()));
        return vec![c.clone()];
    }
    if word.contains('\'') {
        if let Some(expanded) = _apostrophe(word) {
            if !expanded.is_empty() {
                return expanded;
            }
        }
    }
    if let Some(f) = _four_spells_for(word) {
        fixes.push((word.to_string(), f.clone()));
        return vec![f];
    }
    if let Some(c) = _collapse(word) {
        fixes.push((word.to_string(), c.clone()));
        return vec![c];
    }
    if LEXICON.contains(word) || !alpha(word) || n(word) < 2 {
        return vec![word.to_string()];
    }
    if let Some(opened) = _opener_is(word) {
        fixes.push((word.to_string(), opened[0].clone()));
        return opened;
    }
    if let Some(p) = _z_for_s(word) {
        fixes.push((word.to_string(), p.clone()));
        return vec![p];
    }
    if let Some(p) = _ache_for_aike(word) {
        fixes.push((word.to_string(), p.clone()));
        return vec![p];
    }
    if let Some(p) = fix_typo(word) {
        fixes.push((word.to_string(), p.clone()));
        return vec![p];
    }
    if let Some(p) = _said_with_an_f(word) {
        fixes.push((word.to_string(), p.clone()));
        return vec![p];
    }
    if let Some(p) = _said_with_an_f_anywhere(word) {
        fixes.push((word.to_string(), p.clone()));
        return vec![p];
    }
    if let Some(p) = _near_a_feeling(word, before, before_before, after, alone, before_words, after_words) {
        fixes.push((word.to_string(), p.clone()));
        return vec![p];
    }
    vec![word.to_string()]
}

/// `_one_word_food(clean)`: foods typed as two words joined ("ice cream" stays one food).
fn _one_word_food(clean: &str) -> String {
    let mut clean = clean.to_string();
    for (pattern, one) in ONE_WORD_FOODS.iter() {
        clean = pattern.sub(one, &clean);
    }
    clean
}

/// `_joined_words(clean)`: "fedup" -> "fed up".
fn _joined_words(clean: &str) -> String {
    if !JOINED_WORDS.is_match(clean) {
        return clean.to_string();
    }
    JOINED_WORDS.sub_fn(clean, |m| LX_JOINED_WORDS.get(m.as_str()).unwrap_or_else(|| panic!("KeyError: {}", m.as_str())).clone())
}

/// `_z_for_s(word)`: "thingz" -> "things".
fn _z_for_s(word: &str) -> Option<String> {
    if n(word) < 4 || !word.ends_with('z') {
        return None;
    }
    if LEXICON.contains(word) || _a_word_the_bot_writes(word) {
        return None;
    }
    let fixed = format!("{}s", upto(word, -1));
    if LEXICON.contains(&fixed) || _a_word_the_bot_writes(&fixed) { Some(fixed) } else { None }
}

/// `_ache_for_aike(word)`: "headaike" -> "headache".
fn _ache_for_aike(word: &str) -> Option<String> {
    for tail in ["aike", "ayke", "eake"] {
        if word.ends_with(tail) && n(word) > tail.len() {
            let fixed = format!("{}ache", &word[..word.len() - tail.len()]);
            if LEXICON.contains(&fixed) {
                return Some(fixed);
            }
        }
    }
    None
}

/// `_a_contraction_typed_with_a_space(clean)`: "i m", "don t" put back together.
fn _a_contraction_typed_with_a_space(clean: &str) -> String {
    if !SPLIT_CONTRACTION.is_match(clean) && !SPLIT_NOT.is_match(clean) {
        return clean.to_string();
    }
    let clean = SPLIT_CONTRACTION.sub_fn(clean, |m| format!("{}'{}", m.str(1), m.str(2)));
    SPLIT_NOT.sub_fn(&clean, |m| {
        if !NOT_WORDS.contains(&format!("{}t", m.str(1))) {
            return m.as_str().to_string();
        }
        format!("{}'t", m.str(1))
    })
}

/// `_the_ask_said_plainer(text)`: "what was it i said" -> "what did i say".
fn _the_ask_said_plainer(text: &str) -> String {
    SAID_IT.sub_fn(text, |one| match one.name("who") {
        Some(who) if !who.is_empty() => format!("what did {who} say"),
        _ => {
            let saying = one.name_str("saying");
            format!("what {} {} saying", if saying == "i" { "was" } else { "were" }, saying)
        }
    })
}

/// `_me_for_my_pass(text)`: "me head" -> "my head".
fn _me_for_my_pass(text: &str) -> String {
    let text = _A_HEAD_IN_THE_IDIOM.sub(" my head", text);
    _A_CLIPPED_HEAD.sub(" my head", &text)
}

/// `_phrase_pass(text)`: whole phrases replaced before words are split ("cn u", "b/c").
fn _phrase_pass(text: &str) -> String {
    let mut text = _me_for_my_pass(text);
    for (heart, feeling) in HEART_SAID_AS_A_FEELING.iter() {
        if heart.is_match(&text) {
            text = heart.sub(feeling, &text);
        }
    }
    for (pattern, feeling) in SAID_AS_A_FEELING.iter() {
        if pattern.is_match(&text) {
            text = pattern.sub(feeling, &text);
        }
    }
    for pair in PHRASES.iter() {
        let (phrase, value) = (&pair[0], &pair[1]);
        if !text.contains(phrase.as_str()) {
            continue; // the pattern is the literal phrase with lookarounds
        }
        let pattern = rx(&format!("(?<![a-z0-9]){}(?![a-z0-9])", crate::py::re::escape(phrase)));
        if pattern.is_match(&text) {
            text = pattern.sub(&format!(" {value} "), &text);
        }
    }
    let text = UP_TOO_ASKS.sub("\\1up to", &text);
    _the_ask_said_plainer(&text)
}

/// `'%d' % value` for a float (Python truncates toward zero, at any size).
fn percent_d(value: f64) -> String {
    let t = value.trunc();
    if t == 0.0 { "0".to_string() } else { format!("{t:.0}") }
}

/// `_the_figure(figure)`: "1,500.50" and "1.500,50" read as one number.
fn _the_figure(figure: &str) -> String {
    let mut figure = figure.to_string();
    if figure.contains(',') && figure.contains('.') {
        let decimal = if figure.rfind(',') > figure.rfind('.') { ',' } else { '.' };
        let thousands = if decimal == ',' { '.' } else { ',' };
        figure = figure.replace(thousands, "").replace(decimal, ".");
    } else if figure.contains(',') {
        let last = figure.split(',').next_back().unwrap_or("");
        figure = if n(last) == 3 { figure.replace(',', "") } else { figure.replace(',', ".") };
    }
    let value: f64 = py_float(&figure);
    if value == value.trunc() { percent_d(value) } else { format!("{value:.2}").trim_end_matches('0').to_string() }
}

/// Python's `float(text)` for the figures the bot reads (raises like Python on anything else).
fn py_float(s: &str) -> f64 {
    let t = text::strip(s);
    let ok = !t.is_empty()
        && t.chars().all(|c| c.is_ascii_digit() || c == '.')
        && t.matches('.').count() <= 1
        && t != ".";
    if !ok {
        panic!("ValueError: could not convert string to float: {s:?}");
    }
    t.parse().unwrap_or_else(|_| panic!("ValueError: could not convert string to float: {s:?}"))
}

/// `_a_sign_is_a_word(text)`: "£20" -> "20 quid".
fn _a_sign_is_a_word(text: &str) -> String {
    CURRENCY_SIGN.sub_fn(text, |m| {
        let sign = m.str(1);
        let figure = m.str(2);
        let scale = m.str(3).to_lowercase();
        let mut value = py_float(&_the_figure(figure).replace(',', ""));
        if !scale.is_empty() {
            value *= *CURRENCY_FACTOR.get(&scale).unwrap_or_else(|| panic!("KeyError: {scale}")) as f64;
        }
        let written = if value == value.trunc() { percent_d(value) } else { format!("{value:.2}") };
        format!("{written} {}", CURRENCY_SIGNS.get(sign).unwrap_or_else(|| panic!("KeyError: {sign}")))
    })
}

/// `_is_doing_something(word)`.
fn _is_doing_something(word: &str) -> bool {
    if DOING_NOT_THING.contains(word) {
        return true;
    }
    for ending in NOT_A_THING_ENDING.iter() {
        if word.ends_with(ending.as_str()) && n(word) as i64 >= *LONG_ENOUGH_TO_BE_A_VERB.get(ending).unwrap_or_else(|| panic!("KeyError: {ending}")) {
            return !ING_THINGS.contains(word);
        }
    }
    false
}

/// `possessive_me(tokens)`: "me phone" -> "my phone".
pub fn possessive_me(tokens: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token == "me" && index + 1 < tokens.len() {
            let thing = tokens[index + 1].as_str();
            let after = tokens.get(index + 2).map(String::as_str);
            let before = if index > 0 { Some(tokens[index - 1].as_str()) } else { None };
            let before_before = if index >= 2 { Some(tokens[index - 2].as_str()) } else { None };
            let asks_whose = has(&ASKS_WHOSE, before) || (has(&BE_WORDS, before) && has(&ASKS_WHOSE, before_before));
            let owns_a_person = KIN.contains(thing)
                && (after.is_none() || has(&AFTER_A_THING, after) || has(&BEFORE_A_THING, after) || has(&TIME_WORDS, after) || has(&PLACE_WORDS, after));
            let owns_the_thing = has(&AFTER_A_THING, after)
                || has(&OWNING_VERBS, before)
                || owns_a_person
                || (has(&LX_PAIN_WORDS, after) && (before.is_none() || has(&BEFORE_A_THING, before)))
                || (asks_whose && !NOT_A_THING_AFTER.contains(thing) && !(thing.ends_with("ly") || thing.ends_with("ing") || thing.ends_with("ed")))
                || ((before.is_none() || has(&BEFORE_A_THING, before))
                    && !NOT_A_THING_AFTER.contains(thing)
                    && !thing.ends_with("ly")
                    && !(has(&BEFORE_A_PREPOSITION, before) && _is_doing_something(thing)));
            let pair = format!("{thing} {}", after.unwrap_or("None"));
            let how_or_when = TIME_WORDS.contains(thing) || PLACE_WORDS.contains(thing) || has(&TIME_WORDS, after) || NOT_A_THING_PAIR.contains(&pair);
            if owns_the_thing && alpha(thing) && n(thing) > 2 && !NOT_A_THING.contains(thing) && (owns_a_person || !how_or_when) {
                out.push("my".to_string());
                continue;
            }
        }
        out.push(token.clone());
    }
    out
}

/// `a_son_not_the_sun(tokens)`: "my sun is ill" -> son.
pub fn a_son_not_the_sun(tokens: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if (token == "sun" || token == "suns") && index > 0 && OWNING_A_PERSON.contains(&tokens[index - 1]) {
            let after = tokens.get(index + 1).map(String::as_str);
            if !has(&SUN_WORDS, after) && has(&SON_VERBS, after) {
                out.push(if token == "sun" { "son" } else { "sons" }.to_string());
                continue;
            }
        }
        out.push(token.clone());
    }
    out
}

/// `_a_slip_that_is_a_word(tokens)`: a real word typed for another ("pane" for pain).
fn _a_slip_that_is_a_word(tokens: &[String]) -> (Vec<String>, IndexMap<String, String>) {
    let lowered: Vec<String> = tokens.iter().map(|t| t.to_lowercase()).collect();
    let mut out = tokens.to_vec();
    let mut slips = IndexMap::new();
    let get = |i: usize| lowered.get(i).map(String::as_str);
    for (index, token) in lowered.iter().enumerate() {
        let mut right: Option<String> = None;
        let before = if index > 0 { get(index - 1) } else { None };
        let after = get(index + 1);
        let after_that = get(index + 2);
        if let Some(fix) = A_PAIN_WORD.get(token) {
            let kept = has(&DETERMINERS, before) && index > 1 && A_KEEPING_A_THING.contains(&lowered[index - 2]);
            let mut after_body = false;
            if after == Some("in") {
                for place in index + 2..lowered.len().min(index + 5) {
                    let part = &lowered[place];
                    if A_PART_THE_PAIN_IS_IN.contains(part) {
                        after_body = true;
                        break;
                    }
                    if A_PART_A_PERSON_OWNS.contains(part) && OWNING_A_PERSON.contains(&lowered[place - 1]) {
                        after_body = true;
                        break;
                    }
                }
            }
            let before_body = has(&A_PART_THE_PAIN_IS_IN, before) || (has(&A_PART_A_PERSON_OWNS, before) && index > 1 && OWNING_A_PERSON.contains(&lowered[index - 2]));
            let feels = has(&A_STATE_YOU_CAN_FEEL, after)
                || has(&A_COPULA, after)
                || matches!(after, Some("hurt" | "hurts" | "ache" | "aches" | "aching" | "kill" | "killing" | "throb" | "throbbing"));
            let pain_frame = (before_body && feels) || after_body;
            if !kept && !lowered.iter().any(|w| A_GLASS_OR_A_PAN.contains(w)) && pain_frame {
                right = Some(fix.clone());
            }
        } else if token == "hart" && lowered.iter().any(|w| THE_HEART_SAID_THE_OTHER_WAY.contains(w)) {
            right = Some("heart".to_string());
        } else if (token == "chocked" || token == "chockes") && !has(&DETERMINERS, after) && lowered.iter().any(|w| A_CHOKE_SAID_THE_OTHER_WAY.contains(w)) {
            right = Some(if token == "chocked" { "choked" } else { "chokes" }.to_string());
        } else if (token == "ware" || token == "wared") && lowered.iter().any(|w| A_THING_WORN.contains(w)) {
            right = Some(if token == "ware" { "wear" } else { "wore" }.to_string());
        } else if token == "desert" && before == Some("for") {
            right = Some("dessert".to_string());
        } else if token == "live" && has(&BEFORE_A_JOB, after) && has(&A_JOB, after_that) && {
            let third = get(index + 3);
            match third {
                None => true,
                Some("|") => true,
                Some(w) => A_TIME_TO_LEAVE.contains(w),
            }
        } {
            right = Some("leave".to_string());
        } else if token == "saw" && has(&A_COPULA, before) && lowered.iter().any(|p| A_PART_THE_PAIN_IS_IN.contains(p)) {
            right = Some("sore".to_string());
        } else if token == "felling" && has(&A_STATE_YOU_CAN_FEEL, after) {
            right = Some("feeling".to_string());
        }
        if let Some(right) = right {
            if !right.is_empty() && &right != token {
                out[index] = right.clone();
                slips.insert(tokens[index].clone(), right);
            }
        }
    }
    (out, slips)
}

/// `loose_is_lose(tokens)`: "loose weight" -> "lose".
pub fn loose_is_lose(tokens: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if let Some(fixed) = LOOSE_FOR_LOSE.get(token).filter(|f| !f.is_empty()) {
            let mut broke = false;
            for step in 1..=4 {
                let Some(after) = tokens.get(index + step) else {
                    broke = true;
                    break;
                };
                if THE_LOSING.contains(after) {
                    out.push(fixed.clone());
                    broke = true;
                    break;
                }
                if !BEFORE_THE_LOSING.contains(after) && !digit(after) {
                    broke = true;
                    break;
                }
            }
            if !broke {
                out.push(token.clone());
                continue;
            }
            if out.last() == Some(fixed) {
                continue;
            }
        }
        out.push(token.clone());
    }
    out
}

/// `_the_thing_has_been(word)`.
fn _the_thing_has_been(word: Option<&str>) -> bool {
    let plain = match word {
        Some(w) if !w.is_empty() => _plain_chat(w),
        _ => String::new(),
    };
    if plain.is_empty() {
        return false;
    }
    if BEEN_WORDS.contains(&plain) || plain.ends_with("ing") || plain.ends_with("ed") {
        return true;
    }
    plain != "in" && plain.ends_with("in") && !IN_NOUNS.contains(&plain)
}

/// `_is_person(word)`.
fn _is_person(word: Option<&str>) -> bool {
    let Some(word) = word.filter(|w| !w.is_empty()) else { return false };
    let bare = word.replace(['\'', '\u{2019}'], "");
    if PERSON_WORDS.contains(&bare) || PERSON_WORDS.contains(bare.trim_end_matches('s')) {
        return true;
    }
    if PERSON_WORDS.contains(&_plain_chat(word)) {
        return true;
    }
    let spelled = LX_CHAT.get(word).filter(|s| !s.is_empty()).or_else(|| LX_EXPAND.get(word).filter(|s| !s.is_empty())).map(|s| words_of(s)).unwrap_or_default();
    !spelled.is_empty() && PERSON_WORDS.contains(&spelled[0])
}

/// `_plain_chat(word)`: the first word a chat spelling stands for.
fn _plain_chat(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }
    let spelled = LX_CHAT.get(word).filter(|s| !s.is_empty()).or_else(|| LX_EXPAND.get(word).filter(|s| !s.is_empty())).map(String::as_str).unwrap_or(word);
    text::split_ref(spelled).first().map(|w| w.to_string()).unwrap_or_else(|| panic!("IndexError: list index out of range"))
}
/// `_plain_chat(word or '')` for an optional word.
fn plain_chat(word: Option<&str>) -> String {
    _plain_chat(word.unwrap_or(""))
}

/// `_spells_a_helper(word)`.
fn _spells_a_helper(word: Option<&str>) -> bool {
    let Some(word) = word.filter(|w| !w.is_empty() && *w != "|") else { return false };
    let spelled = LX_CHAT.get(word).filter(|s| !s.is_empty()).or_else(|| LX_EXPAND.get(word).filter(|s| !s.is_empty())).map(String::as_str).unwrap_or(word);
    let Some(last) = text::split_ref(spelled).last().map(|s| s.to_string()) else { return false };
    MODAL_FORMS.contains(&last) || BE_FORMS.contains(&last) || HAVE_FORMS.contains(&last) || last == "to" || last == "not"
}

/// `_is_be(word)`.
fn _is_be(word: Option<&str>) -> bool {
    let Some(word) = word.filter(|w| !w.is_empty()) else { return false };
    if BE_FORMS.contains(word) {
        return true;
    }
    match LX_CHAT.get(word).filter(|s| !s.is_empty()).or_else(|| LX_EXPAND.get(word).filter(|s| !s.is_empty())) {
        Some(s) => BE_FORMS.contains(&text::split_ref(s).last().map(|w| w.to_string()).unwrap_or_else(|| panic!("IndexError"))),
        None => false,
    }
}

/// `_is_participle(word)`.
fn _is_participle(word: Option<&str>) -> bool {
    let Some(word) = word.filter(|w| !w.is_empty() && *w != "|" && *w != "or") else { return false };
    let plain = _plain_chat(word);
    plain.ends_with("ed") || IRREGULAR_PARTICIPLES.contains(&plain)
}

/// `_a_gerund_reads_as_a_verb(word, after_after)`.
fn _a_gerund_reads_as_a_verb(word: Option<&str>, after_after: Option<&str>) -> bool {
    let Some(word) = word else { return true };
    if !word.ends_with("ing") || n(word) < 5 {
        return true;
    }
    if matches!(after_after, None | Some("") | Some("|")) {
        return true;
    }
    has(&CAN_FOLLOW_A_VERB, after_after)
}

/// `_kin_needs_a_helper(token, before, after, opens_the_clause, after_after=None)`:
/// "my mums ill" -> my mum is ill.
fn _kin_needs_a_helper(token: &str, before: Option<&str>, after: Option<&str>, opens_the_clause: bool, after_after: Option<&str>) -> Option<Vec<String>> {
    if !opens_the_clause || before != Some("my") {
        return None;
    }
    let single = KIN_SINGULAR.get(token)?;
    if has(&KIN_HAS, after) {
        return Some(vec![single.clone(), "has".to_string()]);
    }
    if has(&KIN_IS, after) || after == Some("not") || digit(after.unwrap_or("")) {
        if has(&KIN_IS, after) && !_a_gerund_reads_as_a_verb(after, after_after) {
            return None;
        }
        return Some(vec![single.clone(), "is".to_string()]);
    }
    if has(&KIN_FILLERS, after) && (has(&KIN_IS, after_after) || has(&KIN_HAS, after_after) || after_after == Some("not")) {
        return Some(vec![single.clone(), if has(&KIN_HAS, after_after) { "has" } else { "is" }.to_string()]);
    }
    None
}

/// `_family_short(token, before, after, after_after)`: "my dd" -> my daughter.
fn _family_short(token: &str, before: Option<&str>, after: Option<&str>, after_after: Option<&str>) -> Option<Vec<String>> {
    let family = FAMILY_SHRT.get(token)?;
    if !_is_possessive(before) {
        return None;
    }
    if let Some(things) = SHRT_THINGS.get(token) {
        if !things.is_empty() && (has(things, after) || has(things, after_after)) {
            return None;
        }
    }
    Some(family.clone())
}

/// `_counts_a_period(word)`.
fn _counts_a_period(word: Option<&str>) -> bool {
    PERIOD_COUNTS.contains(&plain_chat(word))
}

/// `_is_a_count(word)`.
fn _is_a_count(word: Option<&str>) -> bool {
    let plain = plain_chat(word);
    digit(&plain) || A_COUNT.contains(&plain)
}

/// `_hr_is_the_department(after)`.
fn _hr_is_the_department(after: Option<&str>) -> bool {
    HR_IS_THE_DEPARTMENT_AFTER.contains(&plain_chat(after))
}

/// `_says_one(word)`.
fn _says_one(word: Option<&str>) -> bool {
    let plain = plain_chat(word);
    matches!(plain.as_str(), "1" | "a" | "an" | "one") || A_PARTS_OF_A_PERIOD.contains(&plain)
}

/// `_is_a_thing_here(before, after, after_after=None)`.
fn _is_a_thing_here(before: Option<&str>, after: Option<&str>, after_after: Option<&str>) -> bool {
    let said_after = after.unwrap_or("").to_lowercase();
    let said_after = said_after.trim_matches('|');
    if THING_AFTER.contains(said_after) {
        return true;
    }
    if THING_JOIN.contains(said_after) {
        return THING_SUBJECT.contains(&plain_chat(before));
    }
    if THING_PLACE_AFTER.contains(said_after) {
        if !THING_DETERMINER.contains(&plain_chat(before)) {
            return false;
        }
        let aa = after_after.unwrap_or("").to_lowercase();
        return THING_PLACE_WORDS.contains(aa.trim_matches('|'));
    }
    said_after.is_empty() && THING_SUBJECT.contains(&plain_chat(before))
}

/// `_clipped_thing(token)`.
fn _clipped_thing(token: &str) -> String {
    let pair = &CLIPPED_THINGS[token];
    if token.ends_with('s') { pair[1].clone() } else { pair[0].clone() }
}

/// `_is_before_a_noun(token, after)`.
fn _is_before_a_noun(token: &str, after: Option<&str>) -> bool {
    match after {
        _ if token.ends_with('s') => false,
        None | Some("") | Some("|") => false,
        Some(a) => !NOT_BEFORE_A_NOUN.contains(&_plain_chat(a)),
    }
}

/// `_is_said_of_a_person(word)`.
fn _is_said_of_a_person(word: Option<&str>) -> bool {
    let Some(word) = word.filter(|w| !w.is_empty() && *w != "|" && !PERSON_PREPS.contains(*w)) else { return false };
    let plain = _plain_chat(word);
    if n(&plain) > 3 && (plain.ends_with("ing") || plain.ends_with("ed") || plain.ends_with("in")) {
        return true;
    }
    IRREGULAR_PARTICIPLES.contains(&plain) || BEEN_WORDS.contains(&plain)
}

/// `_is_possessive(word)`.
fn _is_possessive(word: Option<&str>) -> bool {
    let Some(word) = word.filter(|w| !w.is_empty()) else { return false };
    POSSESSIVES.contains(&word.replace(['\'', '\u{2019}'], ""))
}

/// `_sounds_like_a_feeling(token, before, after, before_before=None)`: "im board" -> bored.
fn _sounds_like_a_feeling(token: &str, before: Option<&str>, after: Option<&str>, before_before: Option<&str>) -> Option<String> {
    let before = if has(&BEFORE_A_FEELING, before) { before_before } else { before };
    if !has(&FIRST_PERSON_BE, before) {
        return None;
    }
    let feeling = LX_SOUNDS_LIKE_A_FEELING.get(token)?;
    if after.is_some_and(|a| !a.is_empty()) && !has(&A_FEELING_CAN_BE_FOLLOWED, after) {
        return None;
    }
    Some(feeling.clone()).filter(|f| !f.is_empty())
}

/// `_were_for_another_word(before, after, after_after=None)`: "were is it" -> where.
fn _were_for_another_word(before: Option<&str>, after: Option<&str>, after_after: Option<&str>) -> Option<Vec<String>> {
    if !matches!(before, None | Some("|")) {
        return None;
    }
    if matches!(after, None | Some("|")) {
        return None;
    }
    if has(&_A_QUESTION_VERB, after) {
        return Some(strs(&["where"]));
    }
    if has(&_WERE_STATE, after) || has(&_WERE_DOING, after) || has(&_WERE_ARE, after) {
        return Some(strs(&["we", "are"]));
    }
    if has(&_WERE_LEAD_IN, after) {
        if has(&_WERE_STATE, after_after) || has(&_WERE_DOING, after_after) {
            return Some(strs(&["we", "are"]));
        }
        if after_after.is_some() && _a_doing(after_after) {
            return Some(strs(&["we", "are"]));
        }
        if has(&_AFTER_A_LEAD_IN_SUBJECT, after_after) {
            return Some(strs(&["were"]));
        }
        return None;
    }
    if _a_doing(after) && (after_after.is_none() || has(&_AFTER_A_DOING, after_after)) {
        return Some(strs(&["we", "are"]));
    }
    if has(&_WERE_PLACE, after) && !has(&_WERE_IN_THE_PAST, after_after) {
        return Some(strs(&["we", "are"]));
    }
    None
}

/// `_a_doing(word)`.
fn _a_doing(word: Option<&str>) -> bool {
    let Some(word) = word.filter(|w| !w.is_empty()) else { return false };
    let bare = word.replace('\'', "");
    if bare.ends_with("ing") {
        return LEXICON.contains(&bare) || LEXICON.contains(&upto(&bare, -3));
    }
    if bare.ends_with("in") && n(&bare) >= 6 {
        return LEXICON.contains(&bare) || LEXICON.contains(&upto(&bare, -2));
    }
    false
}

/// `_a_doubtful_split(pieces, after=None)`.
fn _a_doubtful_split(pieces: &[String], after: Option<&str>) -> bool {
    if pieces.len() > 1 && pieces[0] == pieces[1] {
        return true;
    }
    GLUED_DO.contains(&pieces[0]) && has(&GLUED_AFTER_A_DO, after)
}

/// `_a_place_said_whole(bare)`.
fn _a_place_said_whole(bare: &str) -> bool {
    LX_COMMON_NAMES.contains(bare) || COUNTRIES.contains(bare)
}

/// `_a_pronoun_said_together(token)`: "imtired" -> i'm tired.
fn _a_pronoun_said_together(token: &str) -> Option<Vec<String>> {
    let bare = token.replace(['\'', '\u{2019}'], "");
    if _a_place_said_whole(&bare) {
        return None;
    }
    for (who, after) in A_PRONOUN_STATE.iter() {
        if bare.starts_with(who.as_str()) && after.contains(&from(&bare, n(who) as isize)) {
            return Some(vec![who.clone(), from(&bare, n(who) as isize)]);
        }
    }
    for who in A_PRONOUN_GLUED.iter() {
        if bare.starts_with(who.as_str()) && n(&bare) > n(who) && I_GLUED.contains(&from(&bare, n(who) as isize)) {
            let rest = if token.starts_with(who.as_str()) { from(token, n(who) as isize) } else { from(&bare, n(who) as isize) };
            return Some(vec![who.clone(), rest]);
        }
    }
    None
}

/// `_a_state_typed_onto_im(token)`: "imtired".
fn _a_state_typed_onto_im(token: &str) -> Option<Vec<String>> {
    if n(token) < 4 || !token.starts_with("im") || !pyuni::is_lower(token) {
        return None;
    }
    let rest = from(token, 2);
    if LX_STATES_SAID_ONTO_IM.contains(&rest) {
        return Some(vec!["im".to_string(), rest]);
    }
    None
}

/// `_glued_words(token, after=None)`: two or three words typed without a space.
fn _glued_words(token: &str, after: Option<&str>) -> Option<Vec<String>> {
    if n(token) < 4 || LEXICON.contains(token) || !pyuni::is_lower(token) {
        return None;
    }
    if let Some(s) = _a_pronoun_said_together(token) {
        return Some(s);
    }
    if let Some(s) = _a_state_typed_onto_im(token) {
        return Some(s);
    }
    let len = n(token);
    let (half, odd) = (len / 2, len % 2);
    if odd == 0 && upto(token, half as isize) == from(token, half as isize) {
        return None;
    }
    for cut in [6usize, 5, 4, 3, 2] {
        let left = upto(token, cut as isize);
        if !GLUED_LEFT.contains(&left) {
            continue;
        }
        let rest = from(token, cut as isize);
        if GLUED_RIGHT.contains(&rest) {
            let pieces = vec![left, rest];
            return if _a_doubtful_split(&pieces, after) { None } else { Some(pieces) };
        }
        if let Some(collapse) = GLUED_COLLAPSE.get(&rest) {
            let mut pieces = vec![left];
            pieces.extend(collapse.iter().cloned());
            return Some(pieces);
        }
        for second in [5isize, 4, 3, 2] {
            let a = upto(&rest, second);
            let b = from(&rest, second);
            if GLUED_RIGHT.contains(&a) && GLUED_RIGHT.contains(&b) {
                let pieces = vec![left.clone(), a, b];
                return if _a_doubtful_split(&pieces, after) { None } else { Some(pieces) };
            }
        }
    }
    None
}

/// `glued_words(token, after=None)`.
pub fn glued_words(token: &str, after: Option<&str>) -> Option<Vec<String>> {
    _glued_words(token, after)
}

/// `_the_mrs_is_the_wife(text)`.
fn _the_mrs_is_the_wife(text: &str) -> String {
    MRS_IS_THE_WIFE.sub("\\1 wife", text)
}

/// `_chat_words_said_out(clean)`: chat words that stand for a phrase spelled out.
fn _chat_words_said_out(clean: &str) -> String {
    if clean.is_empty() {
        return clean.to_string();
    }
    let mut out: Vec<String> = Vec::new();
    for word in text::split_ref(clean) {
        let mut spelled = SAID_OUT.get(word).cloned().filter(|s| !s.is_empty());
        let before: Vec<String> = match out.last() {
            Some(last) => {
                let low = last.to_lowercase();
                words_of(SAID_OUT.get(&low).map(String::as_str).unwrap_or(&low))
            }
            None => vec![],
        };
        if let Some(sp) = &spelled {
            if !before.is_empty() && _SUBJECT_SAID.contains(before.last().unwrap()) && (sp.starts_with("i ") || sp.starts_with("I ")) {
                spelled = Some(sp.split_once(' ').map(|(_, r)| r.to_string()).unwrap_or_default());
            }
        }
        out.push(spelled.filter(|s| !s.is_empty()).unwrap_or_else(|| word.to_string()));
    }
    out.join(" ")
}

/// `_a_hello_said_short(clean)`: "hiya" opening a line is a hello, kept as the rest.
fn _a_hello_said_short(clean: &str) -> String {
    if clean.is_empty() {
        return clean.to_string();
    }
    for pair in HELLO_SAID_SHORT.iter() {
        let (typed, meant) = (&pair[0], &pair[1]);
        if clean == typed {
            return meant.clone();
        }
        if !(clean.starts_with(&format!("{typed} ")) || clean.starts_with(&format!("{typed},")) || clean.starts_with(&format!("{typed}!"))) {
            continue;
        }
        let mut words: Vec<&str> = text::split_ref(text::strip_chars(&clean[typed.len()..], " ,!.")).into_iter().collect();
        while words.first().is_some_and(|w| HELLO_TAIL.contains(text::strip_chars(w, ",.!"))) {
            words.remove(0);
        }
        let rest = words.join(" ");
        let rest = text::strip_chars(&rest, " ,!.");
        return if rest.is_empty() { meant.clone() } else { rest.to_string() };
    }
    clean.to_string()
}

/// `_glued_said_apart(clean)`.
fn _glued_said_apart(clean: &str) -> String {
    let words = text::split_ref(clean);
    let mut out: Vec<String> = Vec::new();
    for (index, token) in words.iter().enumerate() {
        match _glued_words(token, words.get(index + 1).copied()) {
            Some(mut pieces) if !pieces.is_empty() => {
                if let Some(spread) = LX_CHAT.get(&pieces[0]) {
                    if spread.contains(' ') {
                        let mut v = words_of(spread);
                        v.extend(pieces.drain(1..));
                        pieces = v;
                    }
                }
                out.extend(pieces);
            }
            _ => out.push(token.to_string()),
        }
    }
    out.join(" ")
}

/// `_said_run_on(token)`: a whole phrase typed as one word ("iamsotired").
fn _said_run_on(token: &str) -> Option<Vec<String>> {
    if !alpha(token) {
        return None;
    }
    let token = token.to_lowercase();
    let t: Vec<char> = token.chars().collect();
    if (t.len() as i64) < RUN_ON_SHORTEST {
        return None;
    }
    if LEXICON.contains(&token) {
        return None;
    }
    let mut best: HashMap<usize, Vec<String>> = HashMap::new();
    best.insert(0, vec![]);
    for start in 0..t.len() {
        let Some(pieces) = best.get(&start).cloned() else { continue };
        for end in (start + 1..=t.len()).rev() {
            let piece: String = t[start..end].iter().collect();
            if !RUN_ON_WORDS.contains(&piece) {
                continue;
            }
            let mut here = pieces.clone();
            here.push(piece);
            let better = match best.get(&end) {
                None => true,
                Some(there) => here.len() < there.len(),
            };
            if better {
                best.insert(end, here);
            }
        }
    }
    let split = best.get(&t.len())?.clone();
    if split.len() < 2 {
        return None;
    }
    if RUN_ON_LAST_PIECES.contains(split.last().unwrap()) {
        return None;
    }
    if split.iter().filter(|p| n(p) == 1).count() > 1 {
        return None;
    }
    if RUN_ON_SELF.contains(&split[0]) || RUN_ON_MINE.contains(&split[0]) {
        if (t.len() as i64) < RUN_ON_SELF_LONGEST || split.len() < 3 {
            return None;
        }
        return Some(split);
    }
    if RUN_ON_OPENERS.contains(&split[0]) {
        if !split.iter().any(|p| RUN_ON_PEOPLE.contains(p)) {
            return None;
        }
        return Some(split);
    }
    None
}

/// `_a_run_and_what_surrounds_it(token)`: (marks before, letters, marks after).
fn _a_run_and_what_surrounds_it(token: &str) -> (String, String, String) {
    let t: Vec<char> = token.chars().collect();
    let mut start = 0;
    while start < t.len() && !pyuni::ch_alpha(t[start]) {
        start += 1;
    }
    let mut end = t.len();
    while end > start && !pyuni::ch_alpha(t[end - 1]) {
        end -= 1;
    }
    (t[..start].iter().collect(), t[start..end].iter().collect(), t[end..].iter().collect())
}

/// `_typed_without_spaces(clean)`.
fn _typed_without_spaces(clean: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for token in text::split_ref(clean) {
        let (before, letters, after) = _a_run_and_what_surrounds_it(token);
        match _said_run_on(&letters) {
            Some(mut pieces) if !pieces.is_empty() => {
                if let Some(spread) = LX_CHAT.get(&pieces[0]) {
                    if spread.contains(' ') {
                        let mut v = words_of(spread);
                        v.extend(pieces.drain(1..));
                        pieces = v;
                    }
                }
                let first_at = out.len();
                out.extend(pieces);
                if !after.is_empty() {
                    let last = out.last_mut().unwrap();
                    last.push_str(&after);
                }
                if !before.is_empty() {
                    // Python: out[0] = before + out[0] (the line's first word, as written)
                    let _ = first_at;
                    out[0] = format!("{before}{}", out[0]);
                }
            }
            _ => out.push(token.to_string()),
        }
    }
    out.join(" ")
}

/// `_the_said_apart_forms()`: every way a contraction or short word is typed apart.
pub fn _the_said_apart_forms() -> IndexMap<Vec<String>, String> {
    let mut forms: IndexMap<Vec<String>, String> = IndexMap::new();
    for joined in LX_EXPAND.keys().chain(LX_TYPED_CONTRACTIONS.keys()) {
        if !alpha(joined) || !(2..=10).contains(&n(joined)) {
            continue;
        }
        for pieces in _the_ways_a_word_is_typed_apart(joined) {
            forms.entry(pieces).or_insert_with(|| joined.clone());
        }
    }
    let mut bases: Vec<&String> = APOSTROPHE_BASES.iter().collect();
    bases.sort();
    let mut tails: Vec<&String> = APOSTROPHE_TAILS.iter().collect();
    tails.sort();
    for base in bases {
        for tail in &tails {
            if APOSTROPHE_NOT.iter().any(|p| p.len() == 2 && &p[0] == base && &p[1] == *tail) {
                continue;
            }
            let one = format!("{base}'{tail}");
            if _apostrophe(&one).is_some_and(|v| !v.is_empty()) {
                forms.entry(vec![base.clone(), (*tail).clone()]).or_insert(one);
            }
        }
    }
    forms
}

/// `_the_ways_a_word_is_typed_apart(joined)`.
fn _the_ways_a_word_is_typed_apart(joined: &str) -> Vec<Vec<String>> {
    let j: Vec<char> = joined.chars().collect();
    let s = |a: usize, b: usize| -> String { j[a..b].iter().collect() };
    let mut ways = Vec::new();
    for cut in 1..j.len() {
        if cut == 1 || j.len() - cut == 1 {
            ways.push(vec![s(0, cut), s(cut, j.len())]);
        }
    }
    for first_cut in 1..j.len().saturating_sub(1) {
        for second_cut in first_cut + 1..j.len() {
            let pieces = vec![s(0, first_cut), s(first_cut, second_cut), s(second_cut, j.len())];
            if pieces.iter().any(|p| n(p) == 1) {
                ways.push(pieces);
            }
        }
    }
    ways
}

/// `_said_apart(tokens)`: "don t", "i m" typed apart put together.
fn _said_apart(tokens: Vec<String>) -> Vec<String> {
    if tokens.len() < 2 {
        return tokens;
    }
    let said = tokens.join(" ");
    if !SAID_APART.is_match(&said) {
        return tokens;
    }
    let joined = SAID_APART.sub_fn(&said, |m| {
        let before = text::split_ref(&said[..m.start()]);
        if before.last().is_some_and(|b| A_LETTER_AFTER.contains(*b)) {
            return m.as_str().to_string();
        }
        let key = words_of(m.as_str());
        SAID_APART_FORMS.get(&key).cloned().unwrap_or_else(|| panic!("KeyError: {key:?}"))
    });
    words_of(&joined)
}

/// `said_apart(line)`.
pub fn said_apart(line: &str) -> String {
    let words = words_of(line);
    if words.len() < 2 {
        return line.to_string();
    }
    _said_apart(words).join(" ")
}

/// `_ill_is_i_will_going(token, before, after)`.
fn _ill_is_i_will_going(token: &str, before: Option<&str>, after: Option<&str>) -> Option<Vec<String>> {
    if token != "ill" || !has(&ILL_GOES_ON, after) {
        return None;
    }
    if before.is_some() && before != Some("|") && !has(&LEADERS, before) {
        return None;
    }
    Some(strs(&["i", "will"]))
}

/// `_an_ill_that_is_i_will(clean)`: "ill be there" -> "i will be there".
fn _an_ill_that_is_i_will(clean: &str) -> String {
    let mut words = words_of(clean);
    let mut changed = false;
    for index in 0..words.len() {
        if words[index] != "ill" || index + 1 >= words.len() {
            continue;
        }
        if !ILL_GOES_ON.contains(&words[index + 1]) {
            continue;
        }
        let before = if index > 0 { Some(words[index - 1].as_str()) } else { None };
        if before.is_some() && before != Some("|") && !has(&LEADERS, before) {
            continue;
        }
        words[index] = "i will".to_string();
        changed = true;
    }
    if changed { words.join(" ") } else { clean.to_string() }
}

/// `_left_off_contraction(token, before, after, before_before)`: "dont" at the head of a clause.
fn _left_off_contraction(token: &str, before: Option<&str>, after: Option<&str>, before_before: Option<&str>) -> Option<Vec<String>> {
    let words = LEFT_OFF_CONTRACTION.get(token).filter(|w| !w.is_empty())?;
    if !has(&CONTRACTION_NEXT, after) {
        return None;
    }
    if before.is_some() && before != Some("|") {
        let leaders_only = has(&LEADERS, before) && (matches!(before_before, None | Some("|")) || has(&LEADERS, before_before));
        if !leaders_only {
            return None;
        }
    }
    Some(words.clone())
}

/// `_one_of_the_parts(token, before, after)`.
fn _one_of_the_parts(token: &str, before: Option<&str>, after: Option<&str>) -> Option<String> {
    if !token.ends_with('s') || n(token) < 4 || !ONE_OF_A_KIND.contains(&upto(token, -1)) {
        return None;
    }
    if !_is_possessive(before) {
        return None;
    }
    let a = after.unwrap_or("");
    if has(&ONE_PART_AFTER, after) || a.ends_with("ing") || a.ends_with("in") {
        return Some(upto(token, -1));
    }
    None
}

/// `_wers_word(before)`.
fn _wers_word(before: Option<&str>) -> Vec<String> {
    if matches!(before, None | Some("|")) || has(&LEADERS, before) {
        return strs(&["where", "is"]);
    }
    if WEARS_AFTER.contains(&plain_chat(before)) || _is_person(before) {
        return strs(&["wears"]);
    }
    strs(&["where", "is"])
}

/// `_sum_word(before, after=None)`.
fn _sum_word(before: Option<&str>, after: Option<&str>) -> Vec<String> {
    let spelled = if matches!(before, None | Some("|")) { String::new() } else { plain_chat(before) };
    if SUM_THE_NOUN.contains(&spelled) {
        return strs(&["sum"]);
    }
    let behind = if matches!(after, None | Some("|")) { String::new() } else { plain_chat(after) };
    if SUM_STAYS_AFTER.contains(&behind) {
        return strs(&["sum"]);
    }
    if (spelled == "in" || spelled == "to") && !SUM_STAYS_AFTER.contains(&behind) {
        return strs(&["sum"]);
    }
    strs(&["some"])
}

/// `_positional_word(token, before, after, ...)`: a word whose reading depends on its
/// neighbours ("2", "were", "bin", "mi", "ur").
fn _positional_word(token: &str, before: Option<&str>, after: Option<&str>, after_after: Option<&str>, before_before: Option<&str>, before_words: Option<&[String]>) -> Vec<String> {
    let end = |w: Option<&str>| matches!(w, None | Some("|"));
    if UP_TO_SPELLINGS.contains(token) && (end(before) || has(&UP_TO_LEADS, before)) && (end(after) || has(&UP_TO_TAGS, after)) && end(after_after) {
        return strs(&["what", "you", "up", "to"]);
    }
    if WHAT_ARE_YOU_SPELLINGS.contains(token) && has(&WHAT_ARE_YOU_AFTER, after) {
        return strs(&["what", "you"]);
    }
    if token == "w" {
        return strs(&["with"]);
    }
    if token == "c" && has(&LETTER_C_IS_SEE, after) {
        return strs(&["see"]);
    }
    if token == "wers" {
        return _wers_word(before);
    }
    if token == "sum" {
        return _sum_word(before, after);
    }
    if token == "were" {
        if let Some(s) = _were_for_another_word(before, after, after_after) {
            return s;
        }
    }
    if CLIPPED_THINGS.contains_key(token) && _is_a_thing_here(before, after, after_after) {
        return vec![_clipped_thing(token)];
    }
    if let Some(unit) = CLIPPED_UNITS.get(token) {
        if _is_a_count(before) || _says_one(before) {
            if token == "hr" && _hr_is_the_department(after) {
                return vec![token.to_string()];
            }
            if _says_one(before) || _is_before_a_noun(token, after) {
                return vec![unit[0].clone()];
            }
            return vec![unit[1].clone()];
        }
    }
    if let Some(c) = _left_off_contraction(token, before, after, before_before) {
        return c;
    }
    if let Some(g) = _ill_is_i_will_going(token, before, after) {
        return g;
    }
    if let Some(f) = _sounds_like_a_feeling(token, before, after, before_before) {
        return vec![f];
    }
    if let Some(r) = _digit_word(token, before, after, after_after) {
        return vec![r];
    }
    if token == "wdyt" && after.is_some_and(|a| !a.is_empty() && a != "|") {
        return strs(&["what", "do", "you", "think"]);
    }
    if let Some(p) = _one_of_the_parts(token, before, after) {
        return vec![p];
    }
    if token == "ow" && has(&HOW_FOLLOWS, after) {
        return strs(&["how"]);
    }
    if token == "bout" && !has(&DETERMINERS, before) {
        return strs(&["about"]);
    }
    if token == "no" && (has(&KNOW_SUBJECT, before) || has(&NOT_BEFORE_KNOW, before)) {
        let after_read = match after {
            Some(a) if !a.is_empty() && a != "|" => Some(_plain_chat(a)),
            other => other.map(str::to_string),
        };
        if end(after) || has(&A_VERB_CAN_FOLLOW, after) || has(&A_VERB_CAN_FOLLOW, after_read.as_deref()) {
            return strs(&["know"]);
        }
    }
    if token == "hv" {
        let before_read = before.filter(|b| !b.is_empty()).map(_plain_chat).unwrap_or_default();
        let after_read = after.filter(|a| !a.is_empty()).map(_plain_chat).unwrap_or_default();
        if DETERMINERS.contains(&before_read) {
            return vec![token.to_string()];
        }
        if HV_SUBJECTS.contains(&before_read) || HV_HELPERS.contains(&before_read) || after_read == "to" {
            return strs(&["have"]);
        }
        if HV_SUBJECTS.contains(&after_read) {
            return strs(&["have"]);
        }
    }
    if token == "bin" {
        if has(&HAVE_FORMS, before) || before.unwrap_or("").ends_with("ve") {
            return strs(&["been"]);
        }
        if _is_person(before) && (has(&PERSON_PREPS, after) || _is_participle(after) || _is_said_of_a_person(after)) {
            return strs(&["been"]);
        }
        let before_plain = before.filter(|b| !b.is_empty()).map(_plain_chat).unwrap_or_default();
        if has(&THING_POSSESSIVES, before_before) && !BIN_CONTAINERS.contains(&before_plain) && _the_thing_has_been(after) {
            return strs(&["been"]);
        }
        if (has(&BEEN_SUBJECT, before) || has(&BE_FORMS, before) || end(before))
            && (_is_said_of_a_person(after)
                || digit(after.unwrap_or(""))
                || _counts_a_period(after)
                || _counts_a_period(after_after)
                || (has(&BEEN_SUBJECT, before) && matches!(after, Some("a" | "an"))))
        {
            return strs(&["been"]);
        }
    }
    if token == "of" && MODAL_FORMS.contains(&plain_chat(before)) && _is_participle(after) && !has(&DETERMINERS, before_before) {
        return strs(&["have"]);
    }
    if AINT_FORMS.contains(token) {
        if _is_participle(after) || has(&PAST_AS_PARTICIPLE, after) {
            return if has(&HAS_SUBJECT, before) { strs(&["has", "not"]) } else { strs(&["have", "not"]) };
        }
        let b = plain_chat(before);
        if b == "i" {
            return strs(&["am", "not"]);
        }
        if AINT_PLURAL.contains(&b) {
            return strs(&["are", "not"]);
        }
        return strs(&["is", "not"]);
    }
    if token == "of" && after == Some("to") && _is_be(before) {
        return strs(&["off"]);
    }
    if token == "b" && _spells_a_helper(before) {
        return strs(&["be"]);
    }
    if end(after) && _where_a_state_stands(before, before_before, after, before_words) {
        if let Some(state) = _a_letter_wrong_in_a_state(token) {
            return vec![state];
        }
    }
    if has(&FIRST_PERSON_BE, before) || (has(&FIRST_PERSON_BE, before_before) && has(&AMOUNT_BEFORE_A_STATE, before)) {
        if let Some(f) = _feeling_without_its_ending(token) {
            return vec![f];
        }
    }
    let opens = before_before.is_none() || before_before == Some("|") || has(&LEADERS, before_before);
    if let Some(kin) = _kin_needs_a_helper(token, before, after, opens, after_after) {
        return kin;
    }
    if (token == "yr" || token == "yrs") && digit(before.unwrap_or("")) {
        return vec![if token.ends_with('s') || after == Some("old") { "years" } else { "year" }.to_string()];
    }
    if let Some(f) = _family_short(token, before, after, after_after) {
        return f;
    }
    if token == "veggies" && has(&VEGGIES_AFTER, before) {
        return strs(&["vegetables"]);
    }
    if (token == "ur" || token == "your") && after.is_some_and(|a| !a.is_empty()) && has(&YOU_ARE_AFTER, after) {
        return strs(&["you're"]);
    }
    if token == "mi" {
        if has(&MI_IS_ME_AFTER, after) {
            return strs(&["me"]);
        }
        if has(&MI_IS_A_NAME_AFTER, before) {
            return vec![token.to_string()];
        }
        if has(&MI_IS_ME_BEFORE, after) {
            return strs(&["me"]);
        }
        if after == Some("back") && has(&MI_BACK_VERBS, before) {
            return strs(&["me"]);
        }
        if end(after) {
            return strs(&["me"]);
        }
        if !(has(&SOLFEGE, before) && (end(after) || has(&SOLFEGE, after))) {
            if before.is_some() && before != Some("|") && !has(&LEXICON, before) {
                return vec![token.to_string()];
            }
            return strs(&["my"]);
        }
    }
    vec![token.to_string()]
}

/// `_join_chat_digits(tokens)`: "2" "moro" -> "2moro"; spaced-out letters joined.
fn _join_chat_digits(tokens: &[String]) -> Vec<String> {
    let mut joined = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let token = &tokens[index];
        if let Some(following) = tokens.get(index + 1) {
            let both = format!("{token}{following}");
            let digits = following.chars().next().is_some_and(pyuni::ch_digit) || digit(token);
            if digits && (LX_CHAT.contains_key(&both) || LX_MISSPELLINGS.contains_key(&both)) {
                joined.push(both);
                index += 2;
                continue;
            }
        }
        if let Some((word, next)) = _said_apart_as_one(tokens, index) {
            joined.push(word);
            index = next;
            continue;
        }
        joined.push(token.clone());
        index += 1;
    }
    joined
}

/// `_said_apart_as_one_word(clean)`: "t h x" -> "thx".
fn _said_apart_as_one_word(clean: &str) -> String {
    if !_SPACED_RUN.is_match(clean) {
        return clean.to_string();
    }
    _SPACED_RUN.sub_fn(clean, |m| {
        let pieces = words_of(m.str(1));
        for length in (3..=pieces.len()).rev() {
            let word = pieces[..length].concat().to_lowercase();
            if LX_CHAT.contains_key(&word) || LX_MISSPELLINGS.contains_key(&word) {
                let mut out = vec![word];
                out.extend(pieces[length..].iter().cloned());
                return out.join(" ");
            }
        }
        m.as_str().to_string()
    })
}

/// `_said_apart_as_one(tokens, start)`: (the word, where reading goes on), or None.
fn _said_apart_as_one(tokens: &[String], start: usize) -> Option<(String, usize)> {
    if !_SPACED_PIECE.is_match(&tokens[start]) {
        return None;
    }
    for length in [4, 3] {
        if start + length > tokens.len() {
            continue;
        }
        let pieces = &tokens[start..start + length];
        if !pieces.iter().all(|p| _SPACED_PIECE.is_match(p)) {
            continue;
        }
        let word = pieces.concat();
        if LX_CHAT.contains_key(&word) || LX_MISSPELLINGS.contains_key(&word) {
            return Some((word, start + length));
        }
    }
    None
}

/// `_docs_on_a_screen(words)`.
fn _docs_on_a_screen(words: &[String]) -> bool {
    if words.iter().any(|w| DOCS_SCREEN.contains(w)) {
        return true;
    }
    for (index, word) in words.iter().enumerate() {
        if matches!(word.as_str(), "phone" | "tablet" | "laptop" | "pc" | "screen" | "file") {
            if words[index.saturating_sub(2)..index].iter().any(|b| DOCS_ON.contains(b)) {
                return true;
            }
        }
    }
    false
}

/// `_near_a_meal(word, before, after=None)`: "for brekfast" -> breakfast.
fn _near_a_meal(word: &str, before: &str, after: Option<&str>) -> Option<String> {
    if word.is_empty() || n(word) < 3 || LEXICON.contains(word) {
        return None;
    }
    if !FOOD_SLOT_LEAD.contains(&_plain_chat(before)) {
        return None;
    }
    if let Some(a) = after {
        if !TIME_AFTER_MEAL.contains(&_plain_chat(a)) {
            return None;
        }
    }
    if NOT_A_MEAL_TYPO.contains(word) {
        return None;
    }
    let mut meals: Vec<&String> = MEAL_WORDS.iter().collect();
    meals.sort();
    for meal in &meals {
        if n(word) >= 4 && meal.starts_with(word) {
            return Some((*meal).clone());
        }
    }
    let found: Vec<&&String> = meals.iter().filter(|m| _one_letter_apart(word, m)).collect();
    if found.len() == 1 { Some((**found[0]).clone()) } else { None }
}

/// Rebuild `clean` with some of its RAW_WORD words replaced (by word index).
fn replace_words(clean: &str, places: &IndexMap<usize, String>) -> String {
    let mut pieces = String::new();
    let mut last = 0;
    for (index, m) in RAW_WORD.finditer(clean).into_iter().enumerate() {
        pieces.push_str(&clean[last..m.start()]);
        pieces.push_str(places.get(&index).map(String::as_str).unwrap_or(m.as_str()));
        last = m.end();
    }
    pieces.push_str(&clean[last..]);
    pieces
}

/// `_a_meal_word_misspelt(clean, words)`.
fn _a_meal_word_misspelt(clean: String, mut words: Vec<String>) -> (String, Vec<String>) {
    let mut places: IndexMap<usize, String> = IndexMap::new();
    for index in 1..words.len() {
        let after = words.get(index + 1).map(String::as_str);
        if let Some(meal) = _near_a_meal(&words[index], &words[index - 1], after) {
            places.insert(index, meal);
        }
    }
    if places.is_empty() {
        return (clean, words);
    }
    let clean = replace_words(&clean, &places);
    for (index, meal) in &places {
        if *index < words.len() {
            words[*index] = meal.clone();
        } else {
            panic!("IndexError: list assignment index out of range");
        }
    }
    (clean, words)
}

/// `_docs_doctor(clean, words)`: "the docs said" is the doctor.
fn _docs_doctor(clean: String, mut words: Vec<String>) -> (String, Vec<String>) {
    if !words.iter().any(|w| w == "doc" || w == "docs") {
        return (clean, words);
    }
    if words.iter().any(|w| DOCS_PAPER.contains(w)) || _docs_on_a_screen(&words) {
        return (clean, words);
    }
    let mut places: IndexMap<usize, String> = IndexMap::new();
    for (index, word) in words.iter().enumerate() {
        if word != "doc" && word != "docs" {
            continue;
        }
        let near = words[index.saturating_sub(4)..index].iter().chain(words[(index + 1).min(words.len())..(index + 3).min(words.len())].iter());
        if near.into_iter().any(|o| DOCS_VISIT.contains(o)) {
            places.insert(index, "doctor".to_string());
        }
    }
    if places.is_empty() {
        return (clean, words);
    }
    let clean = replace_words(&clean, &places);
    for index in places.keys() {
        words[*index] = "doctor".to_string();
    }
    (clean, words)
}

/// `a_sor_bit_is_sore(tokens)`.
pub fn a_sor_bit_is_sore(tokens: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token == "sor" {
            let before = if index > 0 { Some(tokens[index - 1].as_str()) } else { None };
            let after = tokens.get(index + 1).map(String::as_str);
            let fills = index > 1 && has(&SORE_FILLERS, before) && SORE_IN_FRONT.contains(&tokens[index - 2]);
            if (before.is_some() && (has(&SORE_IN_FRONT, before) || fills)) || (after.is_some() && has(&SORE_PARTS, after)) {
                out.push("sore".to_string());
                continue;
            }
        }
        out.push(token.clone());
    }
    out
}

/// `_one_relation_says(words)`: "my mums says" -> my mum says.
fn _one_relation_says(words: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for (index, word) in words.iter().enumerate() {
        let after = words.get(index + 1).map(String::as_str).filter(|a| !a.is_empty());
        if index > 0 && after.is_some() && word.ends_with('s') && ONE_RELATION.contains(&upto(word, -1)) && _is_possessive(Some(&words[index - 1])) {
            let read = _plain_chat(after.unwrap());
            if ONE_RELATION_VERBS.contains(&read) || HAVE_FORMS.contains(&read) || BE_FORMS.contains(&read) || MODAL_FORMS.contains(&read) {
                out.push(upto(word, -1));
                if HAS_FORMS.contains(&read) {
                    out.push("has".to_string());
                }
                continue;
            }
        }
        out.push(word.clone());
    }
    out
}

/// `_ask_start(clause)`: where a question starts inside a clause ("i was wondering what ...").
fn _ask_start(clause: &[String]) -> Option<usize> {
    if clause.is_empty() || WH_WORDS.contains(&clause[0]) || AUX_WORDS.contains(&clause[0]) {
        return None;
    }
    for start in 1..clause.len() {
        let token = &clause[start];
        let Some(following) = clause.get(start + 1) else { continue };
        let after = clause.get(start + 2).map(String::as_str);
        if WH_WORDS.contains(token) {
            if AUX_WORDS.contains(following) || _a_word_between_the_ask_and_its_aux(clause, start) {
                if _reported(clause, start) {
                    continue;
                }
                return Some(start);
            }
            if HOW_DEGREE.contains(following) || (ASK_NOUNS.contains(following) && has(&AUX_WORDS, after)) {
                if _reported(clause, start) {
                    continue;
                }
                return Some(start);
            }
        } else if AUX_WORDS.contains(token) && SUBJECT_WORDS.contains(following) {
            if NO_ASK_AFTER.contains(&clause[start - 1]) || SUBJECT_WORDS.contains(&clause[start - 1]) || BE_WORDS.contains(token) {
                continue;
            }
            return Some(start);
        }
    }
    None
}

/// `_a_word_between_the_ask_and_its_aux(clause, start)`: "what kind of x do ...".
fn _a_word_between_the_ask_and_its_aux(clause: &[String], start: usize) -> bool {
    let span = &clause[(start + 1).min(clause.len())..(start + 6).min(clause.len())];
    for (offset, word) in span.iter().enumerate() {
        if AUX_WORDS.contains(word) {
            let crossed: Vec<&String> = span[..offset].iter().filter(|w| !WH_QUANTIFIERS.contains(*w) && *w != "of" && *w != "for").collect();
            let subject = clause.get(start + 2 + offset).map(String::as_str);
            return crossed.len() <= 2 && !crossed.iter().any(|w| NOT_IN_A_QUESTION_PHRASE.contains(*w)) && has(&AFTER_AN_INVERTED_AUX, subject);
        }
    }
    false
}

/// `_inverted_ask(clause, start)`.
fn _inverted_ask(clause: &[String], start: usize) -> bool {
    let after = &clause[(start + 1).min(clause.len())..(start + 3).min(clause.len())];
    after.len() > 1 && RELATIVE_ASKS.contains(&after[0]) && SUBJECT_WORDS.contains(&after[1])
}

/// `_reported(clause, start)`: the question word is reported, not asked ("i know what i want").
fn _reported(clause: &[String], start: usize) -> bool {
    let before = &clause[start.saturating_sub(2)..start];
    if before.is_empty() {
        return false;
    }
    if before.iter().any(|w| REPORTING.contains(w)) {
        return true;
    }
    let last = &before[before.len() - 1];
    if RELATIVE.contains(&clause[start])
        && !_inverted_ask(clause, start)
        && !AUX_WORDS.contains(last)
        && !WH_WORDS.contains(last)
        && !REPORTING.contains(last)
        && (DETERMINERS.contains(last) || (before.len() > 1 && (DETERMINERS.contains(&before[0]) || NOMINAL_BEFORE_COPULA.contains(&before[0]))))
    {
        return true;
    }
    COPULAS.contains(last) && (before.len() < 2 || NOMINAL_BEFORE_COPULA.contains(&before[0]))
}

/// `_split_ask(clauses)`: a clause cut where its question starts.
fn _split_ask(clauses: Vec<Vec<String>>) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    for clause in clauses {
        match _ask_start(&clause) {
            Some(start) if start + 1 < clause.len() => {
                out.push(clause[..start].to_vec());
                out.push(clause[start..].to_vec());
            }
            _ => out.push(clause),
        }
    }
    out
}

/// `_opens_clause(token)`.
fn _opens_clause(token: &str) -> bool {
    CLAUSE_OPENERS.contains(token) || SUBJECT_WORDS.contains(token) || WH_WORDS.contains(token) || BREAKS.contains(token) || LEADERS.contains(token)
}

/// `_says_has(words)`: "she is got" -> has.
fn _says_has(mut words: Vec<String>) -> Vec<String> {
    if words.len() < 2 {
        return words;
    }
    for index in 1..words.len() - 1 {
        if words[index] != "is" || !HAS_SUBJECTS.contains(&words[index - 1]) {
            continue;
        }
        let after = &words[index + 1];
        if HAS_AFTER.contains(after) || (after == "gone" && index + 2 < words.len()) {
            words[index] = "has".to_string();
        }
    }
    words
}

/// `_clauses(tokens)`: the line cut at punctuation and at "and"/"but" before a new clause.
fn _clauses(tokens: &[String]) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    let mut current: Vec<String> = Vec::new();
    for (k, token) in tokens.iter().enumerate() {
        if token == "|" {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
            continue;
        }
        let following = tokens.get(k + 1).filter(|f| !f.is_empty());
        if BREAKS.contains(token) && current.len() >= 2 && following.is_some_and(|f| _opens_clause(f)) {
            out.push(std::mem::take(&mut current));
            continue;
        }
        current.push(token.clone());
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// `_a_leading_like_is_a_filler(clean)`: "like i dont know" -> "i dont know".
fn _a_leading_like_is_a_filler(clean: &str) -> String {
    let (head, _, rest) = text::partition(clean, " ");
    if text::strip_chars(head, ",.!?") != "like" || rest.is_empty() {
        return clean.to_string();
    }
    let after = text::strip_chars(rest.split(' ').next().unwrap_or(""), ",.!?");
    if !LIKE_STARTS_A_CLAUSE.contains(after) {
        return clean.to_string();
    }
    rest.to_string()
}

/// `without_a_leading_exclamation(clean)`.
pub fn without_a_leading_exclamation(clean: &str) -> String {
    _an_exclamation_in_front(clean)
}

/// `_an_exclamation_in_front(clean)`: "oh my god i lost my keys" -> "i lost my keys".
fn _an_exclamation_in_front(clean: &str) -> String {
    let mut clean = clean.to_string();
    for _ in 0..3 {
        if let Some(phrase) = EXCLAMATION_PHRASE.match_(&clean) {
            let rest = clean[phrase.end()..].to_string();
            if !rest.is_empty() && _EXCLAMATION_SUBJECTS.contains(text::strip_chars(rest.split(' ').next().unwrap_or(""), ",.!?;:-|")) {
                clean = rest;
                continue;
            }
            return clean;
        }
        let (head, _, rest) = text::partition(&clean, " ");
        if rest.is_empty() || !EXCLAMATION_OPENS.contains(text::strip_chars(head, ",.!?;:-|")) {
            return clean;
        }
        let after = text::strip_chars(rest.split(' ').next().unwrap_or(""), ",.!?;:-|");
        if _EXCLAMATION_SUBJECTS.contains(after) || EXCLAMATION_OPENS.contains(after) {
            clean = rest.to_string();
            continue;
        }
        return clean;
    }
    clean
}

/// `_an_exclamation_is_not_a_thing(clean)`.
fn _an_exclamation_is_not_a_thing(clean: &str) -> String {
    EXCLAIMED_AFTER_A_QUESTION.sub("\\1", clean)
}

/// `strip_leaders(clause)`: the clause without its leading filler words ("so", "well", ...),
/// always keeping the last word.
pub fn strip_leaders(clause: &[String]) -> &[String] {
    let mut start = 0;
    while start + 1 < clause.len() && LEADERS.contains(&clause[start]) {
        start += 1;
    }
    &clause[start..]
}

/// `without_leaders(text)`: the text without its leading filler words.
pub fn without_leaders(text: &str) -> String {
    let mut said = text.to_string();
    loop {
        let Some(m) = _LEADER_WORD.match_(&said) else { return said };
        if !LEADERS.contains(&m.str(1).to_lowercase()) {
            return said;
        }
        let rest = said[m.end()..].to_string();
        if text::strip(&rest).is_empty() {
            return said;
        }
        said = rest;
    }
}

/// `_clause_stop(norm, index)`: where the clause that starts at `index` ends.
fn _clause_stop(norm: &[String], index: usize) -> usize {
    let mut stop = index + 1;
    while stop < norm.len() && !LX_CLAUSE_BREAKS.contains(&norm[stop]) && !_WH_QUESTIONS.contains(&norm[stop]) {
        stop += 1;
    }
    stop
}

/// `background_clause(norm)`: the positions of the first background clause ("when i was ...").
pub fn background_clause(norm: &[String]) -> BTreeSet<usize> {
    for (index, word) in norm.iter().enumerate() {
        if index == 0 || !LX_BACKGROUND_STARTS.contains(word) {
            continue;
        }
        if REPORTING.contains(&norm[index - 1]) {
            continue;
        }
        return (index.._clause_stop(norm, index)).collect();
    }
    BTreeSet::new()
}

/// `reason_positions(norm)`: the positions of the first reason clause ("because ...").
pub fn reason_positions(norm: &[String]) -> BTreeSet<usize> {
    for (index, word) in norm.iter().enumerate() {
        if index == 0 || !REASON_STARTS.contains(word) {
            continue;
        }
        return (index.._clause_stop(norm, index)).collect();
    }
    BTreeSet::new()
}

/// `background_positions(norm)`: the place words inside the background of the line, which say
/// where it happened rather than what is asked.
pub fn background_positions(norm: &[String]) -> BTreeSet<usize> {
    let mut skip = BTreeSet::new();
    let mut spans: Vec<(usize, usize, bool)> = Vec::new();
    if norm.first().map(String::as_str) == Some("it") && norm.len() > 1 && LX_START_VERBS.contains(&norm[1]) {
        spans.push((0, _clause_stop(norm, 1), true));
    }
    let clause = background_clause(norm);
    if let (Some(lo), Some(hi)) = (clause.first(), clause.last()) {
        spans.push((*lo, hi + 1, false));
    }
    for (start, stop, anywhere) in spans {
        let mut index = start;
        while index < stop {
            if !LX_PLACE_PREPS.contains(&norm[index]) {
                index += 1;
                continue;
            }
            if !anywhere && index > 0 && !(_PLACE_LEADS.contains(&norm[index - 1]) || skip.contains(&(index - 1))) {
                index += 1;
                continue;
            }
            index += 1;
            let mut taken = 0;
            while index < stop && taken < 3 && !LX_CLAUSE_BREAKS.contains(&norm[index]) && !_PLACE_STOP.contains(&norm[index]) && !LX_PLACE_PREPS.contains(&norm[index]) {
                skip.insert(index);
                index += 1;
                taken += 1;
            }
        }
    }
    skip
}

/// `opens_without_subject(words, index)`: an auxiliary at `index` that opens a clause whose
/// subject was left out ("am going to the shops", "have been feeling low").
pub fn opens_without_subject(words: &[String], index: usize) -> bool {
    if index >= words.len() || words[index].is_empty() {
        return false;
    }
    let word = words[index].as_str();
    if !AUX_OPENS.contains(word) {
        return false;
    }
    let rest: Vec<&str> = words[index + 1..].iter().map(String::as_str).filter(|w| *w != "|").collect();
    if rest.is_empty() {
        return false;
    }
    let second = rest[0];
    if _SUBJECTS.contains(second) {
        return false;
    }
    if word == "am" {
        return true;
    }
    if _PREDICATE_ONLY.contains(second) {
        return true;
    }
    if _SUBJECTLESS_AUX.contains(word) {
        let clause: Vec<&str> = words[index + 1..].iter().map(String::as_str).take_while(|w| *w != "|").collect();
        let mut k = 0;
        while k < clause.len() && _STATE_FILLER.contains(clause[k]) {
            k += 1;
        }
        if k < clause.len() && _A_STATE_OF_THE_PERSON.contains(clause[k]) && (k + 1 == clause.len() || _STATE_TAIL.contains(clause[k + 1]) || _STATE_FILLER.contains(clause[k + 1])) {
            return true;
        }
    }
    if _HAVE_NEGATIONS.contains(second) && word == "have" {
        return true;
    }
    if word == "have" && _HAVE_PARTICIPLES.contains(second) && rest.len() > 2 {
        return _HAVE_THING.contains(rest[1]) && !_HAVE_ASKING.contains(rest[2]) && !rest[1..].iter().any(|w| TIME_WORDS.contains(*w));
    }
    if n(second) > 4 && (second.ends_with("ing") || second.ends_with("ed")) {
        if rest.len() < 2 {
            return true;
        }
        if _SUBJECTLESS_ONLY.contains(word) {
            return _AFTER_PARTICIPLE.contains(rest[1]);
        }
        let after = if rest.len() > 2 { rest[2] } else { "" };
        return _AFTER_PARTICIPLE.contains(rest[1]) && (after.ends_with("ing") || after.ends_with("ed"));
    }
    false
}

/// `_the_measure_clause(clause)`: "how long" / "how much" said as its own clause.
fn _the_measure_clause(clause: &[String]) -> bool {
    let stripped = strip_leaders(clause);
    stripped.len() > 1 && stripped[0] == "how" && HOW_DEGREE.contains(&stripped[1])
}

/// `_asking(clauses, focus)`: the focus, with a "how much" clause in front of it joined on.
/// `focus` is the index of the focus clause (Python finds it by identity).
fn _asking(clauses: &[Vec<String>], focus: Option<usize>) -> Vec<String> {
    let Some(at_) = focus else { return vec![] };
    let focus_clause = &clauses[at_];
    if focus_clause.is_empty() {
        return vec![];
    }
    if at_ > 0 && _the_measure_clause(&clauses[at_ - 1]) {
        let mut v = clauses[at_ - 1].clone();
        v.extend(focus_clause.iter().cloned());
        return v;
    }
    focus_clause.clone()
}

/// `_focus(clauses)`: the clause that asks (the last question clause, else the last that wants
/// something, else the last with words). Returns its index; None stands for Python's `[]`.
fn _focus(clauses: &[Vec<String>]) -> Option<usize> {
    for (i, clause) in clauses.iter().enumerate().rev() {
        let stripped = strip_leaders(clause);
        if !stripped.is_empty() && ASK_WORDS.contains(&stripped[0]) {
            return Some(i);
        }
    }
    for (i, clause) in clauses.iter().enumerate().rev() {
        if clause.iter().any(|t| WANT_WORDS.contains(t)) {
            return Some(i);
        }
    }
    for (i, clause) in clauses.iter().enumerate().rev() {
        if !strip_leaders(clause).is_empty() {
            return Some(i);
        }
    }
    if clauses.is_empty() { None } else { Some(clauses.len() - 1) }
}

/// `_unaccent(text)`: accents and ligatures off (NFKD without combining marks, after LETTERS).
pub fn _unaccent(text: &str) -> String {
    if text.is_ascii() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        let translated: String = match LETTERS.get(&c) {
            Some(s) => s.clone(),
            None => c.to_string(),
        };
        for d in translated.chars() {
            match UNACCENT.get(&d) {
                Some(s) => out.push_str(s),
                None => out.push(d),
            }
        }
    }
    out
}

/// `intensifier_positions(norm)`: "what on earth", "why the hell": the words after the
/// question word that only add force.
pub fn intensifier_positions(norm: &[String]) -> BTreeSet<usize> {
    let mut positions = BTreeSet::new();
    for (index, word) in norm.iter().enumerate() {
        if !WH_ANY.contains(word) {
            continue;
        }
        for size in [3usize, 2] {
            let phrase = &norm[(index + 1).min(norm.len())..(index + 1 + size).min(norm.len())];
            if phrase.len() == size && WH_INTENSIFIER.match_(&phrase.join(" ")).is_some() {
                positions.extend(index + 1..index + 1 + size);
                break;
            }
        }
    }
    positions
}

/// `_the_surprise_typed(text)`: the words after "what on earth" etc. as typed.
fn _the_surprise_typed(text: &str) -> Vec<String> {
    let flat = rx("[^a-z']+").sub(" ", &text.to_lowercase().replace('\u{2019}', "'"));
    let flat = rx(r"\b(?:wtf|wth|wtaf)\b").sub("what on earth", &flat);
    let mut words: Vec<String> = Vec::new();
    for m in _TYPED_SURPRISE.finditer(&flat) {
        for word in text::split_ref(m.as_str()).into_iter().skip(1) {
            let word = word.strip_suffix("'s").unwrap_or(word).to_string();
            if !words.contains(&word) {
                words.push(word);
            }
        }
    }
    words
}

/// `_without_the_intensifier(norm)`.
fn _without_the_intensifier(norm: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut index = 0;
    while index < norm.len() {
        let word = &norm[index];
        if WH_ANY.contains(word) && index + 1 < norm.len() {
            let phrases = intensifier_positions(norm);
            if phrases.contains(&(index + 1)) {
                let mut end = index + 1;
                while end < norm.len() && phrases.contains(&end) {
                    end += 1;
                }
                out.push(word.clone());
                index = end;
                continue;
            }
        }
        out.push(word.clone());
        index += 1;
    }
    out
}

/// `_a_clip_kept_only_with_its_cue(plain, norm, fixes, line)`: "mon" is "moon" only when the
/// line talks about the sky.
fn _a_clip_kept_only_with_its_cue(mut plain: Vec<String>, mut norm: Vec<String>, mut fixes: Vec<(String, String)>, line: &[String]) -> (Vec<String>, Vec<String>, Vec<(String, String)>) {
    let mut bent: HashMap<&str, &str> = HashMap::new();
    for (a, b) in &fixes {
        bent.insert(a.as_str(), b.as_str());
    }
    let bent: HashMap<String, String> = bent.into_iter().map(|(a, b)| (a.to_string(), b.to_string())).collect();
    for (typed, longer, cues) in _A_CLIP.iter() {
        if bent.get(typed) != Some(longer) || line.iter().any(|w| cues.contains(w)) {
            continue;
        }
        fixes.retain(|(a, _)| a != typed);
        plain = plain.into_iter().map(|w| if &w == longer { typed.clone() } else { w }).collect();
        norm = norm.into_iter().map(|w| if &w == longer { typed.clone() } else { w }).collect();
    }
    (plain, norm, fixes)
}

/// `_fix_auxiliaries(tokens)`: "i been" -> "i have been".
fn _fix_auxiliaries(tokens: Vec<String>) -> Vec<String> {
    if !tokens.iter().any(|t| t == "been") {
        return tokens;
    }
    let mut out: Vec<String> = Vec::new();
    for word in tokens {
        if word == "been" && !out.is_empty() {
            let last = out.last().unwrap().clone();
            if BE_FORMS.contains(&last) {
                let k = out.len();
                out[k - 1] = if k > 1 && PLURAL_SUBJECTS.contains(&out[k - 2]) { "have" } else { "has" }.to_string();
            } else if PLURAL_SUBJECTS.contains(&last) || THIRD_PERSON.contains(&last) {
                let before_it = if out.len() > 1 { Some(out[out.len() - 2].as_str()) } else { None };
                if !has(&HAVE_FORMS, before_it) && !has(&BE_FORMS, before_it) && !has(&MODAL_FORMS, before_it) && before_it != Some("not") {
                    out.push(if PLURAL_SUBJECTS.contains(&last) { "have" } else { "has" }.to_string());
                }
            }
        }
        out.push(word);
    }
    out
}

/// `_change_of_subject_off(clean)`: "changing the subject, ..." taken off the front.
fn _change_of_subject_off(clean: &str) -> (String, bool) {
    let Some(seen) = CHANGE_OF_SUBJECT.match_(clean) else { return (clean.to_string(), false) };
    let rest = &clean[seen.end()..];
    if RAW_WORD.findall(rest).len() < 2 {
        return (clean.to_string(), false);
    }
    (rest.to_string(), true)
}

/// `_without_a_tag_at_the_end(clean)`: "..., yeah?", "..., please", "... lol" taken off the end.
fn _without_a_tag_at_the_end(clean: &str) -> (String, Vec<String>) {
    let mut words: Vec<String> = words_of(clean);
    let bare = |w: &str| text::strip_chars(w, ".,!?").to_string();
    let asks = words.first().is_some_and(|w| TAG_QUESTION_LEADS.contains(&bare(w))) || clean.contains('?');
    let mut dropped: Vec<String> = Vec::new();
    while words.len() > 1 && TAG_WORDS.contains(&bare(words.last().unwrap())) {
        if TAG_GUARD_BEFORE.contains(&bare(&words[words.len() - 2])) {
            break;
        }
        dropped.insert(0, words.pop().unwrap());
    }
    while words.len() > 1 && TAG_PLEASE.contains(&bare(words.last().unwrap())) && asks {
        dropped.insert(0, words.pop().unwrap());
    }
    while words.len() > 1 && LX_LAUGHTER.contains(&bare(words.last().unwrap())) && asks {
        dropped.insert(0, words.pop().unwrap());
    }
    while words.len() > 3 && TAG_THANKS.contains(&bare(words.last().unwrap())) && asks {
        dropped.insert(0, words.pop().unwrap());
    }
    (words.join(" "), dropped)
}

/// `_spellings_of_one_word(one, other)`.
fn _spellings_of_one_word(one: &str, other: &str) -> bool {
    if one.is_empty() || other.is_empty() {
        return false;
    }
    if one == other {
        return true;
    }
    let (short, longer) = if n(one) <= n(other) { (one, other) } else { (other, one) };
    if n(short) < 4 || !longer.starts_with(short) {
        return false;
    }
    SPELLING_ENDINGS.iter().any(|e| *e == from(longer, n(short) as isize))
}

/// `_names_a_letter_beside_a_slip(words, fixed)`.
fn _names_a_letter_beside_a_slip(words: &[String], fixed: &IndexMap<String, String>) -> bool {
    if fixed.is_empty() {
        return false;
    }
    for (k, word) in words.iter().enumerate() {
        if !fixed.contains_key(word) {
            continue;
        }
        let near = words[k.saturating_sub(4)..(k + 5).min(words.len())].join(" ");
        if NAMED_LETTER.is_match(&near) {
            return true;
        }
    }
    false
}

/// `_a_short_word_named(short, before, after)`.
fn _a_short_word_named(short: &str, before: Option<&str>, after: Option<&str>) -> bool {
    if has(&_A_SHORT_WORD_IS_NAMED_BEFORE, before) || has(&_A_SHORT_WORD_IS_NAMED_AFTER, after) {
        return true;
    }
    after.is_some_and(|a| a.ends_with("'s")) && short == "u"
}

/// `_asks_the_spelling(words, fixed)`: the line asks how a word is spelled, so the words are
/// kept as typed.
fn _asks_the_spelling(words: &[String], fixed: &IndexMap<String, String>) -> bool {
    if fixed.is_empty() {
        return false;
    }
    if words.iter().any(|w| SPELLING_ASK.contains(w)) {
        return true;
    }
    if _names_a_letter_beside_a_slip(words, fixed) {
        return true;
    }
    for (k, word) in words.iter().enumerate() {
        if word != "or" {
            continue;
        }
        let before = if k > 0 { words[k - 1].as_str() } else { "" };
        let after = words.get(k + 1).map(String::as_str).unwrap_or("");
        if let Some(fb) = fixed.get(before) {
            if after == fb || after == before {
                return true;
            }
        }
        if let Some(fa) = fixed.get(after) {
            if before == fa || before == after {
                return true;
            }
        }
        if let Some(fb) = fixed.get(before) {
            if _spellings_of_one_word(fb, after) {
                return true;
            }
        }
        if let Some(fa) = fixed.get(after) {
            if _spellings_of_one_word(fa, before) {
                return true;
            }
        }
    }
    false
}

/// `_can_not_as_one_word(clean)`.
fn _can_not_as_one_word(clean: &str) -> String {
    CAN_NOT.sub("cant", clean)
}

/// `_the_day_left_out(text)`: "is it my birthday or my sons" -> "... my sons birthday".
fn _the_day_left_out(text: &str) -> String {
    let Some(named) = DAY_NAMED.search(text) else { return text.to_string() };
    let day = named.str(1).to_lowercase();
    let mut pieces = String::new();
    let mut any = false;
    let mut last = 0;
    for found in OWNER_WITHOUT_THE_DAY.finditer(text) {
        if found.start() < named.end() {
            continue;
        }
        let tail_text = rx("[,.;:!?]").split(&text[found.end()..]).into_iter().next().unwrap_or_default();
        if text::split_ref(&tail_text).iter().any(|w| !TAIL_OF_A_DAY.contains(&w.to_lowercase())) {
            continue;
        }
        pieces.push_str(&text[last..found.end()]);
        pieces.push(' ');
        pieces.push_str(&day);
        last = found.end();
        any = true;
    }
    if !any {
        return text.to_string();
    }
    pieces.push_str(&text[last..]);
    pieces
}

/// `_a_day_named_as_not_mine(text)`: "its not my birthday its my mums".
fn _a_day_named_as_not_mine(text: &str) -> String {
    let Some(negated) = NOT_MY_OWN_DAY.search(text) else { return text.to_string() };
    let day = negated.str(1).to_lowercase();
    let mut owner: Option<(usize, usize)> = None;
    for found in A_NUMBER_OF_SOMEONES_DAY.finditer(&text[..negated.start()]) {
        if KIN.contains(found.str(1).to_lowercase().trim_end_matches('s')) {
            owner = Some((found.start(), found.end()));
        }
    }
    let Some((_, owner_end)) = owner else { return text.to_string() };
    let whole = format!("{} {}{}{}", &text[..owner_end], day, &text[owner_end..negated.start()], &text[negated.end()..]);
    text::squash(&whole)
}

/// `_a_number(word)`.
fn _a_number(word: &str) -> bool {
    _NUMBER_SHAPE.is_match(word) || _SMALL_NUMBERS.contains(word)
}

/// `_kept_for(flat)`: how long a thing has been kept, and where ("in the fridge for 3 days").
fn _kept_for(flat: &[String]) -> Option<Kept> {
    if flat.is_empty() {
        return None;
    }
    let mut time: Option<String> = None;
    let mut at_: usize = 0;
    for (index, word) in flat.iter().enumerate() {
        if index + 1 < flat.len() && _a_number(word) && _DAY_WORDS.contains(&flat[index + 1]) {
            time = Some(format!("{} {}", word, flat[index + 1]));
            at_ = index;
            break;
        }
        if _KEPT_ALREADY.contains(word) {
            let after = flat.get(index + 1).map(String::as_str).unwrap_or("");
            time = Some(if word == "since" && !after.is_empty() { text::strip(&format!("{word} {after}")).to_string() } else { word.clone() });
            at_ = index;
            break;
        }
    }
    let time = time?;
    let mut place: Option<(usize, String)> = None;
    for (index, word) in flat.iter().enumerate() {
        if _KEEPING_PLACES.contains(word) {
            place = Some((index, word.clone()));
            break;
        }
    }
    if let Some((index, place)) = place {
        if (at_ as i64 - index as i64).abs() <= 10 {
            return Some(Kept { place: Some(place), time });
        }
        return None;
    }
    if flat.iter().any(|w| _KEPT_THINGS.contains(w)) {
        return Some(Kept { place: None, time });
    }
    None
}

/// `_only_doubles_a_letter(typed, mended)`.
fn _only_doubles_a_letter(typed: &str, mended: &str) -> bool {
    let m: Vec<char> = mended.chars().collect();
    if m.len() != n(typed) + 1 {
        return false;
    }
    (0..m.len() - 1).any(|k| m[k] == m[k + 1] && m[..k].iter().chain(m[k + 1..].iter()).copied().eq(typed.chars()))
}

/// `normalise(text)`: the line as the rest of the engine reads it.
pub fn normalise(text: &str) -> Line {
    let clean = _glued_said_apart(&text::squash(&_unaccent(text)));
    let clean = _typed_without_spaces(&clean);
    let clean = _said_apart_as_one_word(&clean);
    let clean = _the_mrs_is_the_wife(&clean);
    let clean = text::squash(&clean.to_lowercase());
    let clean = _can_not_as_one_word(&clean);
    let clean = _the_day_left_out(&clean);
    let clean = _a_day_named_as_not_mine(&clean);
    let clean = THOUSANDS.sub("\\1", &clean);
    let clean = THOUSANDS_SAID_WITH_A_SPACE.sub_fn(&clean, |m| format!("{}{}", m.str(1), m.str(2).replace([' ', '\u{a0}'], "")));
    let clean = _one_word_food(&clean);
    let clean = _a_leading_like_is_a_filler(&clean);
    let clean = _joined_words(&clean);
    let clean = _a_contraction_typed_with_a_space(&clean);
    let clean = _an_exclamation_in_front(&clean);
    let clean = _an_exclamation_is_not_a_thing(&clean);
    let clean = _a_hello_said_short(&clean);
    let clean = _chat_words_said_out(&clean);
    let clean = _an_ill_that_is_i_will(&clean);
    let (clean, tags_at_the_end) = _without_a_tag_at_the_end(&clean);
    let (clean, change_of_subject) = _change_of_subject_off(&clean);
    let raw = RAW_WORD.findall_str(&clean);
    let (clean, words) = _a_meal_word_misspelt(clean, raw);
    let (clean, _words) = _docs_doctor(clean, words);
    let said = _a_sign_is_a_word(&clean);
    let mut words = RAW_WORD.findall_str(&said);
    let mut fixes: Vec<(String, String)> = Vec::new();
    let mut norm: Vec<String> = Vec::new();
    let mut plain: Vec<String> = Vec::new();
    let mut fixed: IndexMap<String, String> = IndexMap::new();
    let mut subject_put_back = false;
    let sentence_ends = SENTENCE_END.sub(" | ", &_phrase_pass(&said));
    let tokens = TOKEN.findall_str(&PUNCT_BREAK.sub(" | ", &sentence_ends));
    let joined = _one_relation_says(&a_son_not_the_sun(&a_sor_bit_is_sore(&loose_is_lose(&possessive_me(&_said_apart(_join_chat_digits(&tokens)))))));
    let (joined, slips) = _a_slip_that_is_a_word(&joined);
    for (typed, meant) in &slips {
        fixes.push((typed.clone(), meant.clone()));
        fixed.insert(typed.clone(), meant.clone());
    }
    for (index, word) in joined.iter().enumerate() {
        let before = if index > 0 { Some(joined[index - 1].as_str()) } else { None };
        let after = joined.get(index + 1).map(String::as_str);
        let opens = before.is_none() || before == Some("|") || (has(&LEADERS, before) && _only_leaders_before(&joined, index));
        if opens && DROPPED_SUBJECT_AUX.contains(word) {
            let mut probe = vec![word.clone()];
            probe.extend(_lookahead(&joined, index, 3));
            if subject_missing(&probe, 0) {
                norm.push("i".to_string());
                norm.extend(_token(word, &mut fixes, before, None, None, false, None, None));
                plain.push(word.clone());
                subject_put_back = true;
                continue;
            }
        }
        let after_after = joined.get(index + 2).map(String::as_str);
        let before_before = if index >= 2 { Some(joined[index - 2].as_str()) } else { None };
        if _a_name_slot(before, before_before, word) {
            norm.push(word.clone());
            plain.push(word.clone());
            continue;
        }
        let before_words = &joined[index.saturating_sub(5)..index];
        let after_words = &joined[(index + 1).min(joined.len())..(index + 4).min(joined.len())];
        let verb = _a_core_verb_typed_wrong(word, before, after);
        if let Some(v) = &verb {
            fixes.push((word.clone(), v.clone()));
            fixed.insert(word.clone(), v.clone());
        }
        let spellings = match &verb {
            Some(v) => vec![v.clone()],
            None => _positional_word(word, before, after, after_after, before_before, Some(before_words)),
        };
        for spelled in spellings {
            let mut parts = _token(&spelled, &mut fixes, before, before_before, after, false, Some(before_words), Some(after_words));
            if !parts.is_empty() && norm.last() == Some(&parts[0]) && REPEAT_ONCE.contains(&parts[0]) {
                parts.remove(0);
            } else if parts.len() > 1 && !norm.is_empty() && parts[0] == "i" && parts[1] == "do" && REPEAT_ONCE_SUBJECTS.contains(norm.last().unwrap()) {
                parts.remove(0);
            }
            if parts.is_empty() {
                continue;
            }
            norm.extend(parts.iter().cloned());
            if parts.len() > 1 && LX_PLAIN_EXPANDS.contains(&spelled) {
                plain.push(parts.join(" "));
            } else if parts.len() == 1 && !DIGIT_WORDS.contains_key(&spelled) && !LX_US_SPELLINGS.contains_key(&spelled) {
                plain.push(parts[0].clone());
                if parts[0] != spelled
                    && (LX_MISSPELLINGS.contains_key(&spelled) || LX_URGENT_MISSPELLINGS.contains_key(&spelled) || (LX_SAID_SHORT.contains_key(&spelled) && !_a_short_word_named(&spelled, before, after)))
                {
                    fixed.insert(spelled.clone(), parts[0].clone());
                } else if &spelled != word && LX_STATES_WITHOUT_G.contains(word) {
                    fixed.insert(word.clone(), spelled.clone());
                } else if word == "bin" && parts[0] == "been" {
                    fixed.insert(word.clone(), parts[0].clone());
                }
            } else {
                plain.push(LX_TYPED_CONTRACTIONS.get(&spelled).cloned().unwrap_or_else(|| spelled.clone()));
            }
        }
    }
    let mut norm = _fix_auxiliaries(_says_has(norm));
    let mut intensifier = _the_surprise_typed(text);
    let intensifier_left: Vec<String> = intensifier_positions(&norm).into_iter().map(|k| norm[k].clone()).collect();
    if intensifier.is_empty() {
        intensifier = intensifier_left.clone();
    }
    if !intensifier_left.is_empty() {
        norm = _without_the_intensifier(&norm);
    }
    let (norm, new_words) = _a_game_named_another_way(norm, words);
    words = new_words;
    let norm = _parts_that_hurt(norm, &mut fixes);
    if norm.iter().any(|w| LX_PAIN_WORDS.contains(w)) {
        let mut mended: IndexMap<&str, &str> = IndexMap::new();
        for (a, b) in &fixes {
            mended.insert(a.as_str(), b.as_str());
        }
        plain = plain.into_iter().map(|w| mended.get(w.as_str()).map(|s| s.to_string()).unwrap_or(w)).collect();
    }
    let (plain, norm, mut fixes) = _a_clip_kept_only_with_its_cue(plain, norm, fixes, &joined);
    let clauses = _split_ask(_repair_openers(_clauses(&norm), &mut fixes));
    let flat: Vec<String> = clauses.iter().flatten().cloned().collect();
    let focus_at = _focus(&clauses);
    let asking = _asking(&clauses, focus_at);
    let focus = focus_at.map(|k| clauses[k].clone()).unwrap_or_default();
    let laugh_words = norm.iter().chain(tags_at_the_end.iter());
    let signals = Signals {
        question_mark: text.contains('?'),
        shout: text::split_ref(text).iter().any(|w| n(w) > 2 && pyuni::is_upper(w)),
        laughter: laugh_words.into_iter().any(|w| lexicon::is_a_laugh(w)) || rx(r"\b(?:ha\s*){2,}|\b(?:heh\s*){2,}").is_match(text),
        ellipsis: text.contains("...") || text.contains(".."),
        exclamation: text.contains('!'),
        empty: words.is_empty(),
    };
    let keep = _asks_the_spelling(&words, &fixed);
    let mut settled = fixed.clone();
    for (typed, mended) in &fixes {
        if typed != mended && !mended.contains(' ') && (fixed.contains_key(typed) || !_only_doubles_a_letter(typed, mended)) {
            settled.entry(typed.clone()).or_insert_with(|| mended.clone());
        }
    }
    let words_out = if keep { words.clone() } else { words.iter().map(|w| settled.get(w).cloned().unwrap_or_else(|| w.clone())).collect() };
    let plain_text = rx(r"\s*\|").sub(",", &plain.join(" "));
    let mut fixes_map: IndexMap<String, String> = IndexMap::new();
    for (a, b) in fixes {
        fixes_map.insert(a, b);
    }
    Line {
        text: text.to_string(),
        clean,
        words: words_out,
        plain: text::strip_chars(&plain_text, " ,").to_string(),
        norm_text: if clauses.is_empty() { String::new() } else { clauses.iter().map(|c| c.join(" ")).collect::<Vec<_>>().join(" | ") },
        norm: flat.clone(),
        fixes: fixes_map,
        intensifier,
        intensifier_left,
        focus,
        asking,
        signals,
        dropped_subject: subject_put_back,
        change_of_subject,
        kept: _kept_for(&flat),
        clauses,
    }
}

/// `_the_feeling_word(match)`.
fn _the_feeling_word(m: &Match) -> String {
    let said = text::squash(&m.str(1).to_lowercase());
    _FEELING_WORD.get(&said).cloned().unwrap_or_else(|| "upset".to_string())
}

/// `without_the_figure_of_speech(clean)`: "sick with worry" is worried, not ill.
pub fn without_the_figure_of_speech(clean: &str) -> String {
    let clean = FEELING_BEFORE_SICK.sub_fn(clean, |m| text::split_ref(m.as_str()).first().map(|w| w.to_string()).unwrap_or_else(|| panic!("IndexError")));
    SICK_AS_FEELING.sub_fn(&clean, _the_feeling_word)
}

/// `_the_part_that_is_bad(match)`: "my back is bad" -> "my back hurts".
fn _the_part_that_is_bad(m: &Match) -> String {
    let g1 = m.get(1).unwrap_or("None");
    let g2 = m.get(2).unwrap_or("None");
    let part = g2.to_lowercase();
    let plural = part.ends_with('s') || THE_ODD_PLURALS.contains(&part);
    let link = text::squash(&m.name("link").unwrap_or("").to_lowercase());
    if link == "has been" || link == "have been" {
        let verb = if plural { "have been hurting" } else { "has been hurting" };
        return format!("{g1} {g2} {verb}");
    }
    let verb = if plural { "hurt" } else { "hurts" };
    format!("{g1} {g2} {verb}")
}

/// `_the_part_that_hurts(clean)`.
pub fn _the_part_that_hurts(clean: &str) -> String {
    PART_THAT_IS_BAD.sub_fn(clean, _the_part_that_is_bad)
}

/// `_the_sugar_that_is_blood(clean)`: "my sugar is high" is blood sugar.
pub fn _the_sugar_that_is_blood(clean: &str) -> String {
    SUGAR_FIRST_PERSON.sub_fn(clean, |m| {
        let tail: Vec<String> = text::split_ref(m.str(2)).iter().map(|w| text::strip_chars(w, ".,!?").to_lowercase()).collect();
        let blood = || format!("{} blood sugar{}", m.get(1).unwrap_or("None"), m.get(2).unwrap_or("None"));
        if tail.iter().any(|w| SUGAR_PRICE_SHAPE.match_(w).is_some()) {
            return m.as_str().to_string();
        }
        if tail.iter().any(|w| w.chars().next().is_some_and(pyuni::ch_digit)) {
            return blood();
        }
        let near = &tail[..tail.len().min(3)];
        let mut level: Vec<&String> = near.iter().filter(|w| SUGAR_LEVEL.contains(*w)).collect();
        if level.len() == 1 && level[0] == "rose" {
            let rose = tail.iter().position(|w| w == "rose").unwrap();
            let following = &tail[rose + 1..];
            if !following.is_empty() && !SUGAR_AFTER_ROSE.contains(&following[0]) && !following[0].chars().next().is_some_and(pyuni::ch_digit) {
                level.clear();
            }
        }
        if !level.is_empty() {
            for (k, word) in tail.iter().enumerate() {
                if SUGAR_LEVEL.contains(word) && tail[(k + 1).min(tail.len())..(k + 4).min(tail.len())].iter().any(|w| SUGAR_PRICE.contains(w)) {
                    return m.as_str().to_string();
                }
            }
            return blood();
        }
        m.as_str().to_string()
    })
}
